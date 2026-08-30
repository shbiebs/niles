//! One client session: the path from a wire query to a served answer.
//!
//! This module is where the adoption argument of §6.9 is either true or not. A PostgreSQL
//! client sends a string; that string is parsed as Niles's SQL surface, lowered to the same
//! IR as any other query, verified by the same verifier, and served from the same REV
//! runtime under the same contract. **There is no compatibility layer with its own
//! execution path**, which matters because a second execution path is exactly the seam this
//! thesis argues against everywhere else: two ways to compute an answer is two answers that
//! can disagree.
//!
//! # The session's anchor, and why it exists
//!
//! Every session holds a [`Session::anchor`], and it only ever moves forward. That single
//! `max` is consistency rung 1 — monotonic reads — and it is the cheapest rung on the
//! ladder for a reason: over an append-only base, a session's guarantee that it never sees
//! time run backwards costs one comparison. A session that read the live frontier on every
//! query would give a *weaker* guarantee for the same work, because two queries could
//! straddle nothing at all and still be served out of order under retry.

use crate::pg_wire::{self, Backend, Field, Frontend};
use niles_lang::diagnostics::Severity;
use std::collections::HashMap;

/// What the session can serve from. Kept as a trait so the session can be tested without a
/// running engine, and so the same code serves the in-memory prototype and a durable one.
pub trait Serving {
    /// The current visibility frontier.
    fn frontier(&self) -> u64;
    /// Read a view at an anchor. `None` means the view exists but has no entry for the key.
    fn read(&mut self, view: &str, key: &[i64], anchor: u64) -> Option<i128>;
    /// The views this server knows, and the scale each one's money column carries.
    fn views(&self) -> Vec<(String, u32)>;
}

pub struct Session {
    pub user: String,
    pub database: String,
    /// Rung 1: this only ever increases.
    pub anchor: u64,
    pub in_transaction: bool,
    pub failed: bool,
    /// The schema text this session's queries are compiled against.
    pub schema: String,
    pub queries_served: u64,
}

impl Session {
    pub fn new(user: String, database: String, schema: String) -> Session {
        Session {
            user,
            database,
            anchor: 0,
            in_transaction: false,
            failed: false,
            schema,
            queries_served: 0,
        }
    }

    /// The transaction-status byte a `ReadyForQuery` carries.
    pub fn status(&self) -> u8 {
        if self.failed {
            b'E'
        } else if self.in_transaction {
            b'T'
        } else {
            b'I'
        }
    }

    /// Advance the session anchor. Monotone by construction.
    pub fn observe(&mut self, frontier: u64) -> u64 {
        self.anchor = self.anchor.max(frontier);
        self.anchor
    }

    /// Handle one frontend message, producing the messages to send back.
    pub fn handle(&mut self, msg: Frontend, engine: &mut dyn Serving) -> Vec<Backend> {
        match msg {
            Frontend::Query(sql) => {
                self.queries_served += 1;
                let mut out = self.query(&sql, engine);
                out.push(Backend::ReadyForQuery(self.status()));
                out
            }
            Frontend::Terminate => Vec::new(),
            Frontend::Extended(tag) => {
                self.failed = true;
                vec![
                    pg_wire::unsupported(
                        &format!("the extended query protocol (message `{}`)", tag as char),
                        "a prepared statement must be cached against the epoch it was planned at, \
                         because a plan valid at one visibility frontier need not be valid at another; \
                         that design question is open, and shipping a version that ignored it would be \
                         worse than not shipping one",
                    ),
                    Backend::ReadyForQuery(self.status()),
                ]
            }
            Frontend::Password(_) => vec![Backend::AuthenticationOk, Backend::ReadyForQuery(self.status())],
            Frontend::Unknown(t) => {
                self.failed = true;
                vec![
                    pg_wire::unsupported(&format!("message type `{}`", t as char), "unrecognised"),
                    Backend::ReadyForQuery(self.status()),
                ]
            }
            Frontend::Startup { .. } | Frontend::SslRequest => Vec::new(),
        }
    }

    fn query(&mut self, sql: &str, engine: &mut dyn Serving) -> Vec<Backend> {
        let trimmed = sql.trim().trim_end_matches(';').trim();
        if trimmed.is_empty() {
            return vec![Backend::EmptyQueryResponse];
        }
        let lower = trimmed.to_ascii_lowercase();

        // Transaction control, and the honest limits of it. A `begin`/`commit` pair over a
        // *base* is not a table transaction: the base is appended to inside a `txn` block,
        // and there is no rollback of a sealed epoch. So the session tracks the state a
        // client expects and says plainly what it does not provide.
        match lower.as_str() {
            "begin" | "start transaction" => {
                self.in_transaction = true;
                return vec![
                    Backend::NoticeResponse {
                        message: "this session's transaction state is advisory: a sealed ledger epoch \
                                  has no rollback, and reads are served at a session anchor rather than \
                                  from a snapshot held open"
                            .into(),
                    },
                    Backend::CommandComplete("BEGIN".into()),
                ];
            }
            "commit" | "end" => {
                self.in_transaction = false;
                self.failed = false;
                return vec![Backend::CommandComplete("COMMIT".into())];
            }
            "rollback" | "abort" => {
                self.in_transaction = false;
                self.failed = false;
                return vec![Backend::CommandComplete("ROLLBACK".into())];
            }
            _ => {}
        }

        // Two introspection queries, because a client that cannot list what it can read is
        // not usable from `psql`.
        if lower == "\\d" || lower.starts_with("select * from nilestream_views") {
            let mut rows: Vec<Backend> = vec![Backend::RowDescription(vec![
                Field::text("view"),
                Field::int8("scale"),
            ])];
            for (name, scale) in engine.views() {
                rows.push(Backend::DataRow(vec![Some(name), Some(scale.to_string())]));
            }
            let n = rows.len() - 1;
            rows.push(Backend::CommandComplete(format!("SELECT {n}")));
            return rows;
        }
        if lower.starts_with("select") && lower.contains("nilestream_frontier") {
            let f = engine.frontier();
            self.observe(f);
            return vec![
                Backend::RowDescription(vec![Field::int8("frontier"), Field::int8("session_anchor")]),
                Backend::DataRow(vec![Some(f.to_string()), Some(self.anchor.to_string())]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }

        // The real path: compile the client's SQL as Niles, against this session's schema.
        let program = format!("{}\nview __wire_result = sql {{ {trimmed} }} serve {{ consistency: snapshot, materialize: auto }};\n", self.schema);
        let (prog, mut diags) = niles_lang::parser::parse_program(&program);
        let (cat, rd) = niles_lang::resolve::resolve_program(&prog, self.anchor);
        diags.extend(rd);
        let (_r, td) = niles_lang::typecheck::check_program(&prog, &cat);
        diags.extend(td);

        if diags.has_errors() {
            self.failed = true;
            let first = diags
                .sorted()
                .into_iter()
                .find(|d| d.severity == Severity::Error)
                .expect("has_errors implies one exists");
            let detail = first.notes.first().cloned();
            return vec![pg_wire::diagnostic_error(first.code, &first.msg, detail.as_deref())];
        }

        let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
        if ld.has_errors() {
            self.failed = true;
            return vec![pg_wire::diagnostic_error("NL0500", "this query has no lowering", None)];
        }
        // The verifier stands between the compiler and the engine on this path too. A
        // client cannot be given a way around it, or the trusted base has a hole in it
        // shaped like a network socket.
        let vr = niles_ir::verify::verify(&lowered.circuit);
        if !vr.is_ok() {
            self.failed = true;
            let first = vr.violations.first().map(|v| v.msg.clone()).unwrap_or_default();
            return vec![pg_wire::diagnostic_error("IR000", "the compiled circuit did not verify", Some(&first))];
        }

        // Serve it. The anchor is the session's, advanced to the frontier first, so a
        // client's reads never run backwards.
        let anchor = self.observe(engine.frontier());
        let Some(view) = pick_view(&lowered.circuit, &cat) else {
            self.failed = true;
            return vec![pg_wire::diagnostic_error("NL0501", "this query does not name a servable view", None)];
        };

        // The executable fragment is one keyed aggregate, so a query is answered per key.
        // A query without a key predicate would be a full scan of the view, which the
        // runtime can do but which is not what a partial-state engine is for — so it is
        // refused with an explanation rather than served slowly.
        let keys = extract_keys(trimmed);
        if keys.is_empty() {
            return vec![
                Backend::NoticeResponse {
                    message: "no key predicate: this engine serves a partially materialized view per key, \
                              so an unkeyed scan would defeat the mechanism being measured. Add a \
                              `where <key> = <n>` clause."
                        .into(),
                },
                Backend::RowDescription(vec![Field::int8("key"), Field::numeric("value"), Field::int8("anchor")]),
                Backend::CommandComplete("SELECT 0".into()),
            ];
        }

        let mut out = vec![Backend::RowDescription(vec![
            Field::int8("key"),
            // Money as `numeric`, never `float8`. Exactness that survived the type system
            // must survive the wire.
            Field::numeric("value"),
            Field::int8("anchor"),
        ])];
        let mut n = 0usize;
        for k in &keys {
            match engine.read(&view, &[*k], anchor) {
                Some(v) => {
                    out.push(Backend::DataRow(vec![
                        Some(k.to_string()),
                        Some(v.to_string()),
                        Some(anchor.to_string()),
                    ]));
                    n += 1;
                }
                // A view with no entry for this key is a NULL, not a zero. The absence
                // lattice's distinction reaches the client intact.
                None => {
                    out.push(Backend::DataRow(vec![Some(k.to_string()), None, Some(anchor.to_string())]));
                    n += 1;
                }
            }
        }
        out.push(Backend::CommandComplete(format!("SELECT {n}")));
        out
    }
}

fn pick_view(circuit: &niles_ir::circuit::Circuit, cat: &niles_lang::resolve::Catalog) -> Option<String> {
    if circuit.outputs.contains_key("__wire_result") {
        return Some("__wire_result".into());
    }
    let mut names: Vec<&String> = cat.views.keys().collect();
    names.sort();
    names.first().map(|s| s.to_string())
}

/// The integer keys a `where k = n` or `where k in (a, b)` clause names.
///
/// Deliberately crude, and the crudeness is bounded by the runtime's fragment rather than
/// by effort: the executable IR fragment is one keyed aggregate, so a key predicate is the
/// only predicate that changes what is served. A richer predicate parser would be building
/// for an engine that does not exist yet.
fn extract_keys(sql: &str) -> Vec<i64> {
    let lower = sql.to_ascii_lowercase();
    let Some(w) = lower.find(" where ") else { return Vec::new() };
    let tail = &lower[w + 7..];
    let mut keys = Vec::new();
    let mut num = String::new();
    for ch in tail.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
        } else {
            if !num.is_empty() {
                if let Ok(v) = num.parse::<i64>() {
                    keys.push(v);
                }
                num.clear();
            }
        }
    }
    if let Ok(v) = num.parse::<i64>() {
        keys.push(v);
    }
    keys
}

/// A trivial in-memory `Serving`, for tests and for `nilestreamd --demo`.
pub struct MemoryEngine {
    pub frontier: u64,
    pub data: HashMap<(String, i64), i128>,
    pub views: Vec<(String, u32)>,
}

impl Serving for MemoryEngine {
    fn frontier(&self) -> u64 {
        self.frontier
    }
    fn read(&mut self, view: &str, key: &[i64], _anchor: u64) -> Option<i128> {
        self.data.get(&(view.to_string(), key[0])).copied()
    }
    fn views(&self) -> Vec<(String, u32)> {
        self.views.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA: &str = "\
schema bank {
    currency usd { scale: 2 }
    ledger postings { txn: TxnId, acct: Id<A>, cur: Currency, amt: Money,
        idem: IdemKey window 1.days, conserve per (txn, cur); retain forever; }
    index ix on postings (acct) anchor;
}";

    fn engine() -> MemoryEngine {
        let mut data = HashMap::new();
        data.insert(("__wire_result".to_string(), 1001i64), 85000i128);
        MemoryEngine { frontier: 4200, data, views: vec![("ledger_balance".into(), 2)] }
    }

    fn session() -> Session {
        Session::new("ada".into(), "bank".into(), SCHEMA.into())
    }

    #[test]
    fn a_query_compiles_through_the_same_front_end_as_any_other() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("select acct, sum(amt) from postings where acct = 1001 group by acct".into()),
            &mut e,
        );
        assert!(matches!(out[0], Backend::RowDescription(_)), "{out:?}");
        assert!(out.iter().any(|m| matches!(m, Backend::DataRow(_))));
        assert!(matches!(out.last(), Some(Backend::ReadyForQuery(b'I'))));
    }

    #[test]
    fn a_compiler_error_reaches_the_client_with_its_code_intact() {
        // The point of not having a compatibility layer: the client gets the real
        // diagnostic, not a generic syntax error that says the wrong thing.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(Frontend::Query("select nope from nonexistent_table".into()), &mut e);
        let Some(Backend::ErrorResponse { message, .. }) = out.first() else { panic!("{out:?}") };
        assert!(message.starts_with("[NL"), "the NL code must survive to the wire: {message}");
        assert_eq!(s.status(), b'E', "and the session must enter the failed state");
    }

    #[test]
    fn the_session_anchor_only_moves_forward() {
        // Rung 1, monotonic reads, in one `max`. A session that read the live frontier each
        // time would give a weaker guarantee for the same work.
        let mut s = session();
        assert_eq!(s.observe(100), 100);
        assert_eq!(s.observe(50), 100, "a lower frontier must not move the session back");
        assert_eq!(s.observe(200), 200);
    }

    #[test]
    fn a_missing_key_is_null_and_not_zero() {
        // The absence lattice reaching the client intact. A view with no entry for a key is
        // not a view whose balance is zero, and a wire format that conflated them would
        // undo the distinction the whole engine maintains.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("select acct, sum(amt) from postings where acct = 9999 group by acct".into()),
            &mut e,
        );
        let row = out.iter().find_map(|m| match m {
            Backend::DataRow(cols) => Some(cols.clone()),
            _ => None,
        });
        assert_eq!(row.unwrap()[1], None, "an absent key must be NULL");
    }

    #[test]
    fn every_answer_carries_the_anchor_it_was_true_at() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("select acct, sum(amt) from postings where acct = 1001 group by acct".into()),
            &mut e,
        );
        let Some(Backend::RowDescription(fields)) = out.first() else { panic!() };
        assert_eq!(fields[2].name, "anchor", "the anchor is a column, not a footnote");
        let row = out.iter().find_map(|m| match m {
            Backend::DataRow(c) => Some(c.clone()),
            _ => None,
        });
        assert_eq!(row.unwrap()[2], Some("4200".into()));
    }

    #[test]
    fn money_is_described_as_numeric_on_this_path_too() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("select acct, sum(amt) from postings where acct = 1001 group by acct".into()),
            &mut e,
        );
        let Some(Backend::RowDescription(fields)) = out.first() else { panic!() };
        assert_eq!(fields[1].type_oid, 1700, "the money column must be numeric, not float8");
    }

    #[test]
    fn an_unkeyed_query_is_refused_with_an_explanation() {
        // The engine can scan, but scanning defeats the mechanism being measured, so the
        // refusal says what to add rather than returning a slow answer.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(Frontend::Query("select acct, sum(amt) from postings group by acct".into()), &mut e);
        assert!(out.iter().any(|m| matches!(m, Backend::NoticeResponse { message } if message.contains("key predicate"))), "{out:?}");
    }

    #[test]
    fn the_extended_protocol_is_refused_with_the_open_design_question_named() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(Frontend::Extended(b'P'), &mut e);
        let Some(Backend::ErrorResponse { detail, code, .. }) = out.first() else { panic!("{out:?}") };
        assert_eq!(code, "0A000");
        assert!(detail.as_ref().unwrap().contains("visibility frontier"), "the refusal must name the reason");
    }

    #[test]
    fn transaction_control_says_what_it_does_not_provide() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(Frontend::Query("begin".into()), &mut e);
        assert!(
            out.iter().any(|m| matches!(m, Backend::NoticeResponse { message } if message.contains("no rollback"))),
            "a client must be told that a sealed epoch cannot be rolled back: {out:?}"
        );
        assert_eq!(s.status(), b'T');
        s.handle(Frontend::Query("commit".into()), &mut e);
        assert_eq!(s.status(), b'I');
    }

    #[test]
    fn an_empty_query_gets_the_empty_response_not_an_error() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(Frontend::Query("   ".into()), &mut e);
        assert!(matches!(out[0], Backend::EmptyQueryResponse), "{out:?}");
    }

    #[test]
    fn key_extraction_reads_equality_and_in_lists() {
        assert_eq!(extract_keys("select x from t where acct = 1001"), vec![1001]);
        assert_eq!(extract_keys("select x from t where acct in (1, 2, 3)"), vec![1, 2, 3]);
        assert!(extract_keys("select x from t").is_empty());
    }
}
