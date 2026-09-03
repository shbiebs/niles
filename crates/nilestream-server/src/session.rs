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

/// What the session can serve from. Kept as a trait so the session can be tested without a
/// running engine, and so the same code serves the in-memory prototype and a durable one.
pub trait Serving {
    /// The current visibility frontier.
    fn frontier(&self) -> u64;

    /// **Evaluate a compiled circuit and return its rows.**
    ///
    /// The signature is the finding. It used to be
    /// `read(&mut self, view: &str, key: &[i64], anchor: u64) -> Option<i128>` — one
    /// number, for one key, from a view named by a string — and the implementation ignored
    /// the string, took `key[0]`, and folded `sum(amt)` for currency 0. The session
    /// compiled the client's SQL, verified the circuit, and then threw it away.
    ///
    /// A trait that cannot *accept* a circuit cannot serve one, so the shape of this
    /// method is what made the shortcut invisible: nothing in the type said the answer was
    /// unrelated to the query.
    fn query(
        &mut self,
        circuit: &niles_ir::circuit::Circuit,
        output: &str,
        anchor: u64,
    ) -> Result<Rows, ServeError>;

    /// Append rows as one sealed epoch, returning the epoch **after** it is durable.
    fn append(&mut self, rows: Vec<proto_engine::Row>, txn_id: &str) -> Result<u64, ServeError>;

    /// The views this server knows, and the scale each one's money column carries.
    fn views(&self) -> Vec<(String, u32)>;

    /// What this server does with an append before returning: `"always"` when every epoch
    /// is on stable storage before it is acknowledged, `"none"` when it is not.
    ///
    /// Asked over the wire so a benchmark reports what the server *is* rather than what its
    /// harness believes. A `durable` row measured against a non-durable append is the
    /// single most common way a durability number is inflated.
    fn durability(&self) -> &'static str;

    /// `(reads, hits, misses, rows_touched, resident)` for the read model, or zeroes where
    /// the target has no partial state.
    fn read_stats(&self) -> (u64, u64, u64, u64, usize);
}

/// A served result: column names and rows of optional text.
///
/// `None` is a SQL null and is kept distinct from a zero all the way to the wire, because
/// the whole absence argument is worthless if the last layer collapses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rows {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
}

/// Why a query or an append could not be served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServeError {
    /// The circuit did not evaluate — a non-terminating fixpoint, most usefully.
    Eval(String),
    /// The ledger refused the rows because the set does not conserve, or names a hold
    /// that is not there.
    Rejected(String),
    /// This identity already committed. **Not the same failure**, and not the same code: a
    /// driver retries an integrity violation differently from a duplicate, and a retried
    /// payment that came back as "unbalanced" would be retried again.
    Duplicate(String),
    /// The rows were sealed and the sync did not return. The epoch is **not** published and
    /// the client is told so, because a caller that wanted durability and received an `Ok`
    /// would stop keeping its own copy and find out at the worst possible moment.
    NotDurable(String),
}

impl ServeError {
    /// The SQLSTATE a PostgreSQL client will interpret.
    ///
    /// Real codes rather than a single generic one: a client's retry logic reads this, and
    /// `40001` (serialization failure) means "retry" while `23505` (unique violation) means
    /// "you already did this". Answering `XX000` to both would make an idempotent retry
    /// look like an outage.
    pub fn sqlstate(&self) -> &'static str {
        match self {
            ServeError::Eval(_) => "22000", // data exception
            // The set does not conserve: an integrity constraint, and the constraint is
            // the one this whole system is about.
            ServeError::Rejected(_) => "23000", // integrity_constraint_violation
            ServeError::Duplicate(_) => "23505", // unique_violation: a repeated identity
            ServeError::NotDurable(_) => "58030", // io_error
        }
    }
    pub fn detail(&self) -> &str {
        match self {
            ServeError::Eval(m)
            | ServeError::Rejected(m)
            | ServeError::Duplicate(m)
            | ServeError::NotDurable(m) => m,
        }
    }
}

pub struct Session {
    pub user: String,
    pub database: String,
    /// Rung 1: this only ever increases.
    pub anchor: u64,
    pub in_transaction: bool,
    pub failed: bool,
    /// Rows accumulated inside a `BEGIN` … `COMMIT`, sealed as **one** epoch at commit.
    ///
    /// Held rather than appended per statement, because a transaction whose statements
    /// sealed independently would have an epoch in which half of it had happened — which is
    /// precisely what atomicity means here, and what the old `begin`/`commit` pair did not
    /// provide: it flipped a flag and said so in a notice.
    pending: Vec<proto_engine::Row>,
    /// The identities of the statements in the open transaction, joined into the sealed
    /// set's own idempotency key so a retried transaction is refused as a whole.
    pending_ids: Vec<String>,
    /// Prepared statements and portals, keyed by the schema epoch they were compiled at.
    pub plans: crate::extended::PlanCache,
    /// The schema text this session's queries are compiled against.
    pub schema: String,
    /// **Compiled circuits, keyed by statement text.**
    ///
    /// The simple query path compiled every statement afresh — parse, resolve, typecheck,
    /// lower, verify — and `callgrind` says what that costs: **177,064 instructions against
    /// 7,693 to actually serve a point read**, so 96% of the daemon's own work on that
    /// workload was recompiling a statement it had just compiled. The wall-clock share is
    /// much smaller (a round trip is ~120µs against ~22µs of compilation) which is why this
    /// was worth measuring before believing either number: "the compiler is 0.5% of the
    /// budget" and "the compiler is 96% of the engine" are both true, of different budgets.
    ///
    /// Keyed by `(schema, statement text)` and not by an epoch, because **a session compiles
    /// against one schema for its whole life**: `Session::schema` is set at construction and
    /// never assigned. A key carrying the schema makes that a property of the cache rather
    /// than a fact a reader has to go and check, and leaves the door open for a session whose
    /// schema can move.
    compiled: std::collections::HashMap<(u64, String), niles_lang::lower::Lowered>,
    /// Bounded, and cleared rather than evicted when it fills.
    ///
    /// A cache keyed by arbitrary client text is unbounded memory with a friendly name. LRU
    /// would be better and is not obviously worth the machinery here: a session issues a
    /// handful of statement shapes, and a session that issues more than this many distinct
    /// ones is not one a plan cache was going to help.
    compiled_cleared: u64,
    pub compile_hits: u64,
    pub compile_misses: u64,
    pub queries_served: u64,
}

/// How many compiled statements one session holds before the cache is emptied.
const PLAN_CACHE_LIMIT: usize = 256;

impl Session {
    pub fn new(user: String, database: String, schema: String) -> Session {
        Session {
            user,
            database,
            anchor: 0,
            in_transaction: false,
            failed: false,
            pending: Vec::new(),
            pending_ids: Vec::new(),
            plans: crate::extended::PlanCache::new(),
            schema,
            compiled: std::collections::HashMap::new(),
            compiled_cleared: 0,
            compile_hits: 0,
            compile_misses: 0,
            queries_served: 0,
        }
    }

    /// The identity of the schema a plan was compiled against.
    ///
    /// A hash of the text rather than a ledger epoch. The extended path uses
    /// `engine.frontier()` as its `schema_epoch`, which invalidates every prepared statement
    /// on every *append* — conservative, and useless as a cache, since the schema does not
    /// change when a posting is written. That is left alone here and reported rather than
    /// changed under an efficiency task, because `Prepared::schema_epoch` is the mechanism a
    /// future migration story is meant to hang on.
    fn schema_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.schema.hash(&mut h);
        h.finish()
    }

    /// **Compile a statement, or return the circuit compiled for it earlier.**
    ///
    /// One entry point, used by the simple path and by the extended one, so the two cannot
    /// come to disagree about what a statement means — which is the compatibility-layer
    /// failure this crate's own module docs argue against, in miniature.
    fn compile_cached(&mut self, sql: &str) -> Result<&niles_lang::lower::Lowered, Vec<Backend>> {
        let key = (self.schema_key(), sql.to_string());
        if self.compiled.contains_key(&key) {
            self.compile_hits += 1;
            return Ok(&self.compiled[&key]);
        }
        self.compile_misses += 1;
        let program = format!(
            "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n",
            self.schema
        );
        let (prog, mut diags) = niles_lang::parser::parse_program(&program);
        let (cat, rd) = niles_lang::resolve::resolve_program(&prog, self.anchor);
        diags.extend(rd);
        let (_r, td) = niles_lang::typecheck::check_program(&prog, &cat);
        diags.extend(td);
        if diags.has_errors() {
            let first = diags
                .sorted()
                .into_iter()
                .find(|d| d.severity == Severity::Error)
                .expect("has_errors implies one exists");
            let detail = first.notes.first().cloned();
            self.failed = true;
            return Err(vec![pg_wire::diagnostic_error(
                first.code,
                &first.msg,
                detail.as_deref(),
            )]);
        }
        let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
        if ld.has_errors() {
            // The lowering's own code and message, not a generic one. A client told "this
            // query has no lowering" cannot tell a `limit` it cannot read from a stage that
            // does not exist.
            let first = ld
                .sorted()
                .into_iter()
                .find(|d| d.severity == Severity::Error)
                .expect("has_errors implies one exists");
            let detail = first.notes.first().cloned();
            self.failed = true;
            return Err(vec![pg_wire::diagnostic_error(
                first.code,
                &first.msg,
                detail.as_deref(),
            )]);
        }
        // **The verifier stands between the compiler and the engine on this path too**, and
        // it runs before the plan is cached — so a circuit that does not verify is never
        // stored, and a cache hit is a hit on something that passed the gate. A client cannot
        // be given a way around it, or the trusted base has a hole in it shaped like a
        // network socket.
        let vr = niles_ir::verify::verify(&lowered.circuit);
        if !vr.is_ok() {
            let first = vr
                .violations
                .first()
                .map(|v| v.msg.clone())
                .unwrap_or_default();
            self.failed = true;
            return Err(vec![pg_wire::diagnostic_error(
                "IR000",
                "the compiled circuit did not verify",
                Some(&first),
            )]);
        }
        if self.compiled.len() >= PLAN_CACHE_LIMIT {
            self.compiled.clear();
            self.compiled_cleared += 1;
        }
        Ok(self.compiled.entry(key).or_insert(lowered))
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
                // **A simple query string may hold several statements**, and PostgreSQL
                // executes them in order with one `ReadyForQuery` at the end. The server
                // used to treat the whole string as one query, so
                // `psql -c "begin; insert …; commit"` — the ordinary way anyone scripts a
                // transaction — came back as a syntax error on the word `begin`. The
                // transaction machinery existed and there was no way to reach it from a
                // real client.
                let mut out = Vec::new();
                for stmt in split_statements(&sql) {
                    self.queries_served += 1;
                    out.extend(self.query(&stmt, engine));
                    // PostgreSQL abandons the rest of a multi-statement string after an
                    // error, which is what makes `begin; …; commit` safe to send in one
                    // go: a failing statement does not leave the following `commit` to
                    // seal a half-built transaction.
                    if self.failed {
                        break;
                    }
                }
                if out.is_empty() {
                    out.push(Backend::EmptyQueryResponse);
                }
                out.push(Backend::ReadyForQuery(self.status()));
                out
            }
            Frontend::Terminate => Vec::new(),
            // **The extended protocol, served by the plan cache that was written for it.**
            //
            // `extended.rs` is a complete module with eleven tests and a written answer to
            // the epoch-invalidation question — a plan is valid exactly while no schema
            // epoch has occurred after the one it was compiled at, which is a comparison of
            // two integers. It was reachable from nothing: the decoder discarded the
            // message body, so the statement text never arrived, and every `Parse` was
            // answered with a refusal citing a design question the module beside it had
            // already answered.
            Frontend::Extended(tag, body) => self.extended(tag, &body, engine),
            Frontend::Password(_) => vec![
                Backend::AuthenticationOk,
                Backend::ReadyForQuery(self.status()),
            ],
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
                self.pending.clear();
                self.pending_ids.clear();
                return vec![
                    Backend::NoticeResponse {
                        message:
                            "reads in this session are served at a session anchor rather than \
                                  from a snapshot held open; writes are buffered and sealed as one \
                                  epoch at COMMIT"
                                .into(),
                    },
                    Backend::CommandComplete("BEGIN".into()),
                ];
            }
            // **One epoch, or none.** The buffered rows are sealed together, so there is no
            // moment at which half the transaction is visible. This used to flip a flag and
            // return, which is why the notice above had to apologise for it.
            "commit" | "end" => {
                self.in_transaction = false;
                let rows = std::mem::take(&mut self.pending);
                let ids = std::mem::take(&mut self.pending_ids);
                if rows.is_empty() {
                    self.failed = false;
                    return vec![Backend::CommandComplete("COMMIT".into())];
                }
                let txn = ids.join("+");
                return match engine.append(rows, &txn) {
                    Ok(epoch) => {
                        self.observe(epoch);
                        self.failed = false;
                        vec![Backend::CommandComplete(format!("COMMIT {epoch}"))]
                    }
                    Err(e) => {
                        self.failed = true;
                        vec![pg_wire::sqlstate_error(
                            e.sqlstate(),
                            "the transaction did not commit",
                            Some(e.detail()),
                        )]
                    }
                };
            }
            // A rollback discards the buffer. Nothing was sealed, so there is nothing to
            // compensate — which is the whole reason the rows are held rather than appended.
            "rollback" | "abort" => {
                self.in_transaction = false;
                self.failed = false;
                let n = self.pending.len();
                self.pending.clear();
                self.pending_ids.clear();
                return vec![Backend::CommandComplete(format!("ROLLBACK {n}"))];
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
        // Whether this server's appends are durable, asked rather than assumed. The
        // benchmark used to hard-code `false` for this target with a comment explaining
        // that the read side is an in-memory demo — which was true, and meant a `durable`
        // row could be *measured* against a non-durable append with nothing but a comment
        // standing between that and a fabricated durability number.
        // The read-model counters, so a benchmark can record the miss rate beside the
        // latency. A parity result at a zero miss rate says a warm view is fast; one at a
        // real miss rate says *reconstruction* is, which is the claim the thesis makes —
        // and the CSV had `n/a` in that column because nothing could ask.
        if lower.starts_with("select") && lower.contains("nilestream_stats") {
            let (reads, hits, misses, rows, resident) = engine.read_stats();
            return vec![
                Backend::RowDescription(vec![
                    Field::int8("reads"),
                    Field::int8("hits"),
                    Field::int8("misses"),
                    Field::int8("rows_touched"),
                    Field::int8("resident"),
                ]),
                Backend::DataRow(vec![
                    Some(reads.to_string()),
                    Some(hits.to_string()),
                    Some(misses.to_string()),
                    Some(rows.to_string()),
                    Some(resident.to_string()),
                ]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }
        if lower.starts_with("select") && lower.contains("nilestream_durability") {
            let mode = engine.durability();
            return vec![
                Backend::RowDescription(vec![Field::text("durability")]),
                Backend::DataRow(vec![Some(mode.to_string())]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }
        if lower.starts_with("select") && lower.contains("nilestream_frontier") {
            let f = engine.frontier();
            self.observe(f);
            return vec![
                Backend::RowDescription(vec![
                    Field::int8("frontier"),
                    Field::int8("session_anchor"),
                ]),
                Backend::DataRow(vec![Some(f.to_string()), Some(self.anchor.to_string())]),
                Backend::CommandComplete("SELECT 1".into()),
            ];
        }

        // **A write.** `INSERT INTO postings VALUES (txn, acct, cur, amt)` becomes a
        // sealed epoch, and inside a transaction it is buffered until `COMMIT`. The old
        // path wrapped the statement in a view and rejected it, so the wire surface was
        // read-only and the §6.9 claim covered half a database.
        if lower.starts_with("insert") {
            return self.insert(trimmed, engine);
        }
        if lower.starts_with("update") || lower.starts_with("delete") {
            self.failed = true;
            return vec![pg_wire::sqlstate_error(
                "0A000",
                "`update` and `delete` are not legal against a ledger",
                Some(
                    "history is the authority: a fact that can be edited is not evidence, and a \
                     reconstruction over an edited base is not a reproduction. Append a \
                     compensating entry.",
                ),
            )];
        }

        // The anchor is observed *before* compiling, and the ordering is not arbitrary: a
        // cached circuit was compiled at some earlier anchor, so the cache is sound only
        // because a lowering does not depend on one. `Catalog::epoch` is carried and never
        // read by `lower`; `as_of` takes a literal epoch. `a_cached_plan_does_not_depend_on
        // _the_anchor_it_was_compiled_at` holds that, because it is the assumption the whole
        // cache rests on.
        let anchor = self.observe(engine.frontier());

        // The real path: compile the client's SQL as Niles, against this session's schema —
        // or take the circuit compiled for it earlier. `compile_cached` runs the verifier
        // before it stores anything, so a hit is a hit on a circuit that passed the gate.
        //
        // Serving it means **evaluating that circuit**, which is the whole of F-16: the
        // three lines this replaced picked a view by name, scraped integers out of the query
        // text with a digit scanner, and asked the engine for `sum(amt)` on the first of
        // them. The compiler ran, the verifier ran, and neither had any bearing on the
        // answer.
        let (served, named) = match self.compile_cached(trimmed) {
            Ok(lowered) => {
                let served = engine.query(&lowered.circuit, "__wire_result", anchor);
                // **The column names come from the lowering**, which is the only place that
                // knows them: a circuit carries indices, so the engine can only name columns
                // positionally. Taking them from `Lowered::schemas` means a client sees the
                // names it wrote rather than `c0`, `c1`.
                let named: Vec<String> = lowered
                    .circuit
                    .outputs
                    .get("__wire_result")
                    .and_then(|id| lowered.schemas.get(id))
                    .cloned()
                    .unwrap_or_default();
                (served, named)
            }
            Err(e) => return e,
        };
        let rows = match served {
            Ok(r) => r,
            Err(e) => {
                self.failed = true;
                return vec![pg_wire::sqlstate_error(
                    e.sqlstate(),
                    "this query could not be evaluated",
                    Some(e.detail()),
                )];
            }
        };
        let mut rows = rows;
        if named.len() + 1 == rows.columns.len() {
            rows.columns = named;
            rows.columns.push("anchor".into());
        }

        let mut out = vec![Backend::RowDescription(
            rows.columns
                .iter()
                .map(|c| {
                    // Money as `numeric`, never `float8`. Exactness that survived the type
                    // system must survive the wire.
                    if c == "anchor" {
                        Field::int8("anchor")
                    } else {
                        Field::numeric(c)
                    }
                })
                .collect(),
        )];
        let n = rows.rows.len();
        for r in rows.rows {
            out.push(Backend::DataRow(r));
        }
        out.push(Backend::CommandComplete(format!("SELECT {n}")));
        out
    }

    /// Serve one extended-protocol message.
    ///
    /// The messages are the standard ones and the state machine is the standard one: a
    /// `Parse` compiles and caches, a `Bind` makes a portal, an `Execute` serves it, and a
    /// `Sync` ends the implicit transaction and emits `ReadyForQuery`. Only `Sync` emits
    /// `ReadyForQuery`, which is the rule that distinguishes the extended protocol from the
    /// simple one and the one a client notices immediately if it is broken.
    fn extended(&mut self, tag: u8, body: &[u8], engine: &mut dyn Serving) -> Vec<Backend> {
        use crate::extended::ExtError;
        let mut at = 0usize;
        match tag {
            // Parse: statement name, query, parameter type OIDs.
            b'P' => {
                let name = pg_wire::get_cstr(body, &mut at);
                let sql = pg_wire::get_cstr(body, &mut at);
                let epoch = engine.frontier();
                // The fields are not known until the query is compiled, and compiling it
                // here is what makes a `Describe` before `Execute` answerable.
                match self.compile(&sql) {
                    Ok(fields) => {
                        let n = sql.matches('$').count();
                        self.plans.parse(&name, &sql, epoch, fields, n);
                        vec![Backend::ParseComplete]
                    }
                    Err(b) => {
                        self.failed = true;
                        vec![b]
                    }
                }
            }
            // Bind: portal, statement, parameters.
            b'B' => {
                let portal = pg_wire::get_cstr(body, &mut at);
                let statement = pg_wire::get_cstr(body, &mut at);
                match self.plans.bind(&portal, &statement, Vec::new(), 0, false) {
                    Ok(()) => vec![Backend::BindComplete],
                    Err(e) => {
                        self.failed = true;
                        vec![e.to_backend()]
                    }
                }
            }
            b'D' => {
                let _kind = if body.is_empty() { b'S' } else { body[0] };
                at = 1;
                let name = pg_wire::get_cstr(body, &mut at);
                match self.plans.describe(&name) {
                    Ok(fields) => vec![Backend::RowDescription(fields)],
                    Err(e) => {
                        self.failed = true;
                        vec![e.to_backend()]
                    }
                }
            }
            b'E' => {
                let portal = pg_wire::get_cstr(body, &mut at);
                let sql = match self.plans.portal(&portal) {
                    Ok(p) => {
                        let stmt = p.statement.clone();
                        match self.plans.lookup(&stmt, engine.frontier()) {
                            Ok(prep) => prep.sql.clone(),
                            // A stale plan is **recompiled**, not refused: the client's
                            // prepared name keeps working across a migration, which is the
                            // entire point of having prepared it.
                            Err(Some(sql)) => sql,
                            Err(None) => {
                                self.failed = true;
                                return vec![ExtError::UnknownStatement(stmt).to_backend()];
                            }
                        }
                    }
                    Err(e) => {
                        self.failed = true;
                        return vec![e.to_backend()];
                    }
                };
                self.plans.note_execution(&portal);
                self.queries_served += 1;
                // The rows only: an `Execute` does not re-send `RowDescription`, which the
                // client already got from `Describe`.
                self.query(&sql, engine)
                    .into_iter()
                    .filter(|m| !matches!(m, Backend::RowDescription(_)))
                    .collect()
            }
            b'S' => vec![Backend::ReadyForQuery(self.status())],
            b'C' => {
                let kind = if body.is_empty() { b'S' } else { body[0] };
                at = 1;
                let name = pg_wire::get_cstr(body, &mut at);
                if kind == b'P' {
                    self.plans.close_portal(&name);
                } else {
                    self.plans.close_statement(&name);
                }
                vec![Backend::CloseComplete]
            }
            b'H' => Vec::new(), // Flush: everything is written eagerly.
            other => {
                self.failed = true;
                vec![pg_wire::unsupported(
                    &format!("extended message `{}`", other as char),
                    "not part of the protocol's message set",
                )]
            }
        }
    }

    /// Compile a statement far enough to know its output columns, without serving it.
    fn compile(&mut self, sql: &str) -> Result<Vec<Field>, Backend> {
        let program = format!(
            "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n",
            self.schema
        );
        let (prog, mut diags) = niles_lang::parser::parse_program(&program);
        let (cat, rd) = niles_lang::resolve::resolve_program(&prog, self.anchor);
        diags.extend(rd);
        let (_r, td) = niles_lang::typecheck::check_program(&prog, &cat);
        diags.extend(td);
        let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
        diags.extend(ld);
        if diags.has_errors() {
            let first = diags
                .sorted()
                .into_iter()
                .find(|d| d.severity == Severity::Error)
                .expect("has_errors implies one exists");
            return Err(pg_wire::diagnostic_error(
                first.code,
                &first.msg,
                first.notes.first().map(|x| x.as_str()),
            ));
        }
        let mut fields: Vec<Field> = lowered
            .circuit
            .outputs
            .get("__wire_result")
            .and_then(|id| lowered.schemas.get(id))
            .map(|cols| cols.iter().map(|c| Field::numeric(c)).collect())
            .unwrap_or_default();
        fields.push(Field::int8("anchor"));
        Ok(fields)
    }

    /// `INSERT INTO postings VALUES (txn, acct, cur, amt), (…)` — the write path.
    ///
    /// Parsed here rather than through the lowering because an insert is not a query and has
    /// no circuit: the IR describes read models. The statement's shape is narrow on purpose
    /// and the narrowness is *stated* — a wider parser would be inventing a DML surface the
    /// language has not specified, and thesis §11.3's rule is to narrow publicly.
    fn insert(&mut self, sql: &str, engine: &mut dyn Serving) -> Vec<Backend> {
        let Some(rows) = parse_insert(sql) else {
            self.failed = true;
            return vec![pg_wire::sqlstate_error(
                "0A000",
                "this `insert` is outside the supported form",
                Some(
                    "the form is `INSERT INTO postings VALUES (txn, acct, cur, amt)[, …]`, with \
                     integer literals. Anything else is refused rather than partly understood.",
                ),
            )];
        };
        if rows.is_empty() {
            return vec![Backend::CommandComplete("INSERT 0 0".into())];
        }
        let n = rows.len();
        // The identity: the transaction numbers in the statement. An insert with no
        // identity would not be idempotent, and a retried insert over a wire that dropped
        // an acknowledgement is the ordinary case rather than the exotic one.
        let txn = rows
            .iter()
            .map(|r| match r {
                proto_engine::Row::Post(p) => p.txn.to_string(),
                _ => "x".into(),
            })
            .collect::<Vec<_>>()
            .join("-");

        if self.in_transaction {
            self.pending.extend(rows);
            self.pending_ids.push(txn);
            return vec![Backend::CommandComplete(format!("INSERT 0 {n}"))];
        }
        match engine.append(rows, &txn) {
            Ok(epoch) => {
                self.observe(epoch);
                vec![Backend::CommandComplete(format!("INSERT 0 {n}"))]
            }
            Err(e) => {
                self.failed = true;
                vec![pg_wire::sqlstate_error(
                    e.sqlstate(),
                    "the insert did not commit",
                    Some(e.detail()),
                )]
            }
        }
    }
}

/// Split a simple-query string into statements at top-level semicolons.
///
/// Quotes and parentheses are tracked, because a `;` inside a string literal or a value
/// list is not a statement boundary and splitting there would cut a query in half — which
/// is worse than not splitting at all, since the halves might each parse.
fn split_statements(sql: &str) -> Vec<String> {
    let (mut out, mut cur) = (Vec::new(), String::new());
    let (mut in_str, mut depth) = (false, 0i32);
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                // A doubled quote inside a literal is an escaped quote, not a close.
                if in_str && chars.peek() == Some(&'\'') {
                    cur.push(c);
                    cur.push(chars.next().expect("peeked"));
                    continue;
                }
                in_str = !in_str;
                cur.push(c);
            }
            '(' if !in_str => {
                depth += 1;
                cur.push(c);
            }
            ')' if !in_str => {
                depth -= 1;
                cur.push(c);
            }
            ';' if !in_str && depth <= 0 => {
                if !cur.trim().is_empty() {
                    out.push(cur.trim().to_string());
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// The rows an `INSERT INTO postings VALUES (…)` names.
///
/// `None` for anything outside the form. Deliberately not lenient: a partly-understood
/// insert would put rows in the ledger that do not match what was written, and the ledger is
/// the one place in this system where that cannot be corrected by an update.
fn parse_insert(sql: &str) -> Option<Vec<proto_engine::Row>> {
    let lower = sql.to_ascii_lowercase();
    let into = lower.find("into")?;
    let values = lower.find("values")?;
    let target = lower[into + 4..values].trim();
    // The column list, if written, is accepted only in the declared order — an insert
    // naming columns in another order would be silently permuted otherwise.
    let target = target.split('(').next()?.trim();
    if target != "postings" {
        return None;
    }
    let mut out = Vec::new();
    let mut rest = &sql[values + 6..];
    while let Some(open) = rest.find('(') {
        let close = rest[open..].find(')')? + open;
        let fields: Vec<i128> = rest[open + 1..close]
            .split(',')
            .map(|f| f.trim().parse::<i128>().ok())
            .collect::<Option<Vec<_>>>()?;
        if fields.len() != 4 {
            return None;
        }
        out.push(proto_engine::Row::Post(proto_engine::Posting {
            txn: u64::try_from(fields[0]).ok()?,
            acct: u64::try_from(fields[1]).ok()?,
            cur: u32::try_from(fields[2]).ok()?,
            amt: fields[3],
            valid: 0,
        }));
        rest = &rest[close + 1..];
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
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

    /// A test double that **evaluates the circuit**, like the real engine.
    ///
    /// The double it replaces held a `HashMap<(view, key), value>` and answered from it, so
    /// the session's tests could not have caught F-16: a session that ignored the circuit
    /// and a double that ignored the circuit agreed perfectly.
    struct MemoryEngine {
        frontier: u64,
        postings: niles_ir::eval::ZSet,
        views: Vec<(String, u32)>,
        appended: Vec<(String, usize)>,
    }

    impl Serving for MemoryEngine {
        fn frontier(&self) -> u64 {
            self.frontier
        }
        fn query(
            &mut self,
            circuit: &niles_ir::circuit::Circuit,
            output: &str,
            anchor: u64,
        ) -> Result<Rows, ServeError> {
            let mut src = std::collections::BTreeMap::new();
            src.insert("postings".to_string(), self.postings.clone());
            let (z, _) = niles_ir::eval::try_run(circuit, output, &src)
                .map_err(|e| ServeError::Eval(e.to_string()))?;
            let width = z.keys().next().map(|r| r.len()).unwrap_or(0);
            let mut columns: Vec<String> = (0..width).map(|i| format!("c{i}")).collect();
            columns.push("anchor".into());
            let mut rows = Vec::new();
            for (r, w) in &z {
                for _ in 0..(*w).max(0) {
                    let mut cells: Vec<Option<String>> = r
                        .iter()
                        .map(|v| match v {
                            niles_ir::value::Value::Null => None,
                            niles_ir::value::Value::Int(i) => Some(i.to_string()),
                        })
                        .collect();
                    cells.push(Some(anchor.to_string()));
                    rows.push(cells);
                }
            }
            Ok(Rows { columns, rows })
        }
        fn append(
            &mut self,
            rows: Vec<proto_engine::Row>,
            txn_id: &str,
        ) -> Result<u64, ServeError> {
            if self.appended.iter().any(|(t, _)| t == txn_id) {
                return Err(ServeError::Duplicate(format!(
                    "`{txn_id}` already committed"
                )));
            }
            self.appended.push((txn_id.to_string(), rows.len()));
            for r in rows {
                if let proto_engine::Row::Post(p) = r {
                    niles_ir::eval::add(
                        &mut self.postings,
                        vec![
                            niles_ir::value::Value::Int(p.txn as i128),
                            niles_ir::value::Value::Int(p.acct as i128),
                            niles_ir::value::Value::Int(p.cur as i128),
                            niles_ir::value::Value::Int(p.amt),
                            niles_ir::value::Value::Null,
                        ],
                        1,
                    );
                }
            }
            self.frontier += 1;
            Ok(self.frontier)
        }
        fn views(&self) -> Vec<(String, u32)> {
            self.views.clone()
        }
        fn durability(&self) -> &'static str {
            "none"
        }
        fn read_stats(&self) -> (u64, u64, u64, u64, usize) {
            (0, 0, 0, 0, 0)
        }
    }

    fn engine() -> MemoryEngine {
        // Two accounts, so a query naming one must not answer for the other.
        let postings = niles_ir::eval::zset(&[
            (&[1, 1001, 0, 85_000, 0], 1),
            (&[2, 2002, 0, 12_500, 0], 1),
            (&[3, 1001, 0, 500, 0], 1),
        ]);
        let mut postings: niles_ir::eval::ZSet = postings;
        // The `idem` column is text in the schema and has no integer form; it is null.
        let nulled: Vec<(Vec<niles_ir::value::Value>, i128)> = postings
            .iter()
            .map(|(r, w)| {
                let mut r = r.clone();
                r[4] = niles_ir::value::Value::Null;
                (r, *w)
            })
            .collect();
        postings = nulled.into_iter().collect();
        MemoryEngine {
            frontier: 4200,
            postings,
            views: vec![("ledger_balance".into(), 2)],
            appended: Vec::new(),
        }
    }

    fn session() -> Session {
        Session::new("ada".into(), "bank".into(), SCHEMA.into())
    }

    /// The data rows in a reply.
    fn rows_of(out: &[Backend]) -> Vec<Vec<Option<String>>> {
        out.iter()
            .filter_map(|m| match m {
                Backend::DataRow(r) => Some(r.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_query_compiles_through_the_same_front_end_as_any_other() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
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
        let out = s.handle(
            Frontend::Query("select nope from nonexistent_table".into()),
            &mut e,
        );
        let Some(Backend::ErrorResponse { message, .. }) = out.first() else {
            panic!("{out:?}")
        };
        assert!(
            message.starts_with("[NL"),
            "the NL code must survive to the wire: {message}"
        );
        assert_eq!(
            s.status(),
            b'E',
            "and the session must enter the failed state"
        );
    }

    #[test]
    fn the_session_anchor_only_moves_forward() {
        // Rung 1, monotonic reads, in one `max`. A session that read the live frontier each
        // time would give a weaker guarantee for the same work.
        let mut s = session();
        assert_eq!(s.observe(100), 100);
        assert_eq!(
            s.observe(50),
            100,
            "a lower frontier must not move the session back"
        );
        assert_eq!(s.observe(200), 200);
    }

    #[test]
    fn a_missing_key_is_null_and_not_zero() {
        // The absence lattice reaching the client intact. A view with no entry for a key is
        // not a view whose balance is zero, and a wire format that conflated them would
        // undo the distinction the whole engine maintains.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 9999 group by acct".into(),
            ),
            &mut e,
        );
        // **The old assertion was wrong, and its wrongness is the finding.** It said this
        // query returns one row whose value is NULL. It does not: a grouped aggregate over
        // an account with no postings produces *no group*, and SQL returns no row. The old
        // server manufactured one — it took the key out of the query text and emitted a row
        // for it whether or not the data had one, which is a fabricated result row wearing
        // the absence argument's clothes.
        //
        // The distinction the old test was defending is real and is enforced one layer
        // down, where it belongs: `RevEngine::read_point` returns `Option`, and
        // `a_key_the_base_has_never_seen_has_no_value` holds it.
        let rows: Vec<&Vec<Option<String>>> = out
            .iter()
            .filter_map(|m| match m {
                Backend::DataRow(r) => Some(r),
                _ => None,
            })
            .collect();
        assert!(
            rows.is_empty(),
            "an account with no postings forms no group, so there is no row to be null: {out:?}"
        );
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t == "SELECT 0")),
            "{out:?}"
        );
    }

    #[test]
    fn every_answer_carries_the_anchor_it_was_true_at() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &mut e,
        );
        let Some(Backend::RowDescription(fields)) = out.first() else {
            panic!()
        };
        assert_eq!(
            fields[2].name, "anchor",
            "the anchor is a column, not a footnote"
        );
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
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &mut e,
        );
        let Some(Backend::RowDescription(fields)) = out.first() else {
            panic!()
        };
        assert_eq!(
            fields[1].type_oid, 1700,
            "the money column must be numeric, not float8"
        );
    }

    #[test]
    fn an_unkeyed_query_is_answered_by_the_scan_surface() {
        // **Inverted, and the inversion is the finding.** This used to assert that an
        // unkeyed query was *refused*, on the reasoning that "scanning defeats the
        // mechanism being measured". That is a benchmark's reason, not a database's: a
        // server that refuses `group by` because the answer would be uninteresting to an
        // experiment cannot answer the analytical half of its own Part 0 table.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("select acct, sum(amt) from postings group by acct".into()),
            &mut e,
        );
        let rows: Vec<&Vec<Option<String>>> = out
            .iter()
            .filter_map(|m| match m {
                Backend::DataRow(r) => Some(r),
                _ => None,
            })
            .collect();
        assert_eq!(rows.len(), 2, "both accounts: {out:?}");
        assert!(
            !out.iter()
                .any(|m| matches!(m, Backend::NoticeResponse { .. })),
            "and no apology"
        );
    }

    #[test]
    fn the_extended_protocol_parses_binds_and_executes() {
        // **Inverted, and the inversion is the finding.** This used to assert that `Parse`
        // was refused with "that design question is open" — while `extended.rs`, in the
        // same crate, contained the answer, a plan cache keyed by schema epoch, and eleven
        // tests. The module was reachable from nothing.
        let (mut s, mut e) = (session(), engine());
        let mut parse = Vec::new();
        push_cstr(&mut parse, "st1");
        push_cstr(
            &mut parse,
            "select acct, sum(amt) from postings where acct = 1001 group by acct",
        );
        parse.extend_from_slice(&0u16.to_be_bytes());
        let out = s.handle(Frontend::Extended(b'P', parse), &mut e);
        assert!(
            matches!(out.first(), Some(Backend::ParseComplete)),
            "{out:?}"
        );

        let mut bind = Vec::new();
        push_cstr(&mut bind, "po1");
        push_cstr(&mut bind, "st1");
        let out = s.handle(Frontend::Extended(b'B', bind), &mut e);
        assert!(
            matches!(out.first(), Some(Backend::BindComplete)),
            "{out:?}"
        );

        // Describe answers from the plan, before any row is fetched — which is the whole
        // point of having compiled at `Parse` time.
        let mut desc = vec![b'S'];
        push_cstr(&mut desc, "st1");
        let out = s.handle(Frontend::Extended(b'D', desc), &mut e);
        let Some(Backend::RowDescription(fields)) = out.first() else {
            panic!("{out:?}")
        };
        assert_eq!(
            fields.last().map(|f| f.name.as_str()),
            Some("anchor"),
            "every answer carries the moment it is true at"
        );

        let mut exec = Vec::new();
        push_cstr(&mut exec, "po1");
        exec.extend_from_slice(&0u32.to_be_bytes());
        let out = s.handle(Frontend::Extended(b'E', exec), &mut e);
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::DataRow(r) if r[1] == Some("85500".into()))),
            "{out:?}"
        );
        assert!(
            !out.iter().any(|m| matches!(m, Backend::RowDescription(_))),
            "an Execute does not re-send the row description the client already has"
        );

        // Only `Sync` emits `ReadyForQuery`. A client notices immediately if this is wrong.
        let out = s.handle(Frontend::Extended(b'S', Vec::new()), &mut e);
        assert!(
            matches!(out.first(), Some(Backend::ReadyForQuery(_))),
            "{out:?}"
        );
    }

    #[test]
    fn a_prepared_statement_survives_a_schema_epoch_moving() {
        // The answer `extended.rs` was written around: a plan is valid exactly while no
        // schema epoch has occurred after the one it was compiled at. A stale plan is
        // recompiled, not refused, so the client's prepared name keeps working.
        let (mut s, mut e) = (session(), engine());
        let mut parse = Vec::new();
        push_cstr(&mut parse, "st1");
        push_cstr(
            &mut parse,
            "select acct, sum(amt) from postings where acct = 1001 group by acct",
        );
        parse.extend_from_slice(&0u16.to_be_bytes());
        s.handle(Frontend::Extended(b'P', parse), &mut e);
        let mut bind = Vec::new();
        push_cstr(&mut bind, "po1");
        push_cstr(&mut bind, "st1");
        s.handle(Frontend::Extended(b'B', bind), &mut e);

        // The frontier moves, so the plan's schema epoch no longer matches.
        e.frontier += 10;
        let mut exec = Vec::new();
        push_cstr(&mut exec, "po1");
        exec.extend_from_slice(&0u32.to_be_bytes());
        let out = s.handle(Frontend::Extended(b'E', exec), &mut e);
        assert!(
            out.iter().any(|m| matches!(m, Backend::DataRow(_))),
            "a plan whose schema epoch moved is recompiled, not refused: {out:?}"
        );
    }

    fn push_cstr(b: &mut Vec<u8>, s: &str) {
        b.extend_from_slice(s.as_bytes());
        b.push(0);
    }

    #[test]
    fn transaction_control_says_what_it_does_and_does_not_provide() {
        // The notice no longer apologises for a rollback that does not exist — writes are
        // buffered and sealed as one epoch at `COMMIT`, so a `ROLLBACK` discards them and
        // there is nothing to compensate. What it still says is the part that remains true:
        // reads are served at a session anchor rather than from a snapshot held open.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(Frontend::Query("begin".into()), &mut e);
        assert!(
            out.iter().any(|m| matches!(m, Backend::NoticeResponse { message } if message.contains("session anchor"))),
            "a client must be told how its reads are anchored: {out:?}"
        );
        assert!(
            out.iter().any(|m| matches!(m, Backend::NoticeResponse { message } if message.contains("one epoch at COMMIT"))),
            "and how its writes are sealed: {out:?}"
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

    /// **A statement is compiled once per session, and the second ask is a cache hit.**
    ///
    /// `callgrind` on the point path: 177,064 instructions to compile a statement against
    /// 7,693 to serve it, so 96% of the daemon's own work on that workload was recompiling
    /// something it had just compiled. Both of the things previously said about this are
    /// true and neither is the whole story — compilation is 0.5% of a *wall-clock* budget
    /// dominated by a 120µs round trip, and 96% of the engine's own — which is why the
    /// decision to build this was gated on measuring it rather than on either intuition.
    #[test]
    fn a_repeated_statement_is_compiled_once() {
        let (mut s, mut e) = (session(), engine());
        let sql = "select acct, sum(amt) from postings where acct = 1001 group by acct";
        let first = s.handle(Frontend::Query(sql.into()), &mut e);
        assert!(
            !first
                .iter()
                .any(|m| matches!(m, Backend::ErrorResponse { .. })),
            "{first:?}"
        );
        assert_eq!((s.compile_hits, s.compile_misses), (0, 1));

        for _ in 0..20 {
            let again = s.handle(Frontend::Query(sql.into()), &mut e);
            assert_eq!(
                rows_of(&again),
                rows_of(&first),
                "a cached plan must answer what the compiled one did"
            );
        }
        assert_eq!(
            (s.compile_hits, s.compile_misses),
            (20, 1),
            "twenty-one asks, one compilation"
        );

        // A *different* statement is a different key, and still compiles.
        let other = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 9 group by acct".into(),
            ),
            &mut e,
        );
        assert!(!other
            .iter()
            .any(|m| matches!(m, Backend::ErrorResponse { .. })));
        assert_eq!((s.compile_hits, s.compile_misses), (20, 2));
    }

    /// **A statement that does not compile is not cached, and says so every time.**
    ///
    /// The failure a plan cache invites: storing something the verifier refused, or
    /// answering the second ask from a cache entry that should not exist. A refusal is not a
    /// plan.
    #[test]
    fn a_refused_statement_is_refused_again_and_never_cached() {
        let (mut s, mut e) = (session(), engine());
        for _ in 0..3 {
            let mut fresh = session();
            let out = fresh.handle(
                Frontend::Query("select nope from nonexistent_table".into()),
                &mut e,
            );
            assert!(
                out.iter()
                    .any(|m| matches!(m, Backend::ErrorResponse { .. })),
                "a statement that does not compile must be refused: {out:?}"
            );
            assert_eq!(fresh.compile_hits, 0, "a refusal is not a plan");
        }
        let _ = &mut s;
    }

    /// **The assumption the cache rests on**: a lowering does not depend on the anchor it
    /// was compiled at.
    ///
    /// The cache is keyed by `(schema, statement text)` and not by the session anchor, which
    /// would be a different key on every write and no cache at all. That is sound only
    /// because `Catalog::epoch` is carried into the lowering and never read by it — `as_of`
    /// takes a *literal* epoch. If that ever stops being true, this fails here rather than
    /// by serving a stale plan.
    #[test]
    fn a_cached_plan_does_not_depend_on_the_anchor_it_was_compiled_at() {
        let sql = "select acct, sum(amt) from postings where acct = 1001 group by acct";
        let circuit_at = |anchor: u64| {
            let program = format!(
                "{SCHEMA}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n"
            );
            let (prog, d) = niles_lang::parser::parse_program(&program);
            assert!(!d.has_errors());
            let (cat, _) = niles_lang::resolve::resolve_program(&prog, anchor);
            let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
            assert!(!ld.has_errors());
            format!("{:?}", lowered.circuit)
        };
        assert_eq!(
            circuit_at(0),
            circuit_at(4_200),
            "the same statement lowered to different circuits at two anchors, so a plan \
             cached at one is not valid at another and this cache is unsound"
        );
        assert_eq!(circuit_at(4_200), circuit_at(u64::MAX / 2));
    }

    #[test]
    fn two_queries_over_one_key_return_different_answers() {
        // **The test that would have caught F-16, and did not exist.** The served answer
        // used to be `sum(amt)` for `key[0]`, whatever was asked: `pick_view` returned a
        // constant, `extract_keys` scraped digit runs out of the query *text*, and the
        // circuit the compiler had just verified was dropped on the floor. Two different
        // questions about account 1001 came back with the same number.
        let (mut s, mut e) = (session(), engine());
        let rows = |out: &[Backend]| -> Vec<Vec<Option<String>>> {
            out.iter()
                .filter_map(|m| match m {
                    Backend::DataRow(r) => Some(r.clone()),
                    _ => None,
                })
                .collect()
        };
        let sum = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &mut e,
        );
        let count = s.handle(
            Frontend::Query(
                "select acct, count(amt) from postings where acct = 1001 group by acct".into(),
            ),
            &mut e,
        );
        let (a, b) = (rows(&sum), rows(&count));
        assert_eq!(a.len(), 1, "{sum:?}");
        assert_eq!(b.len(), 1, "{count:?}");
        assert_eq!(a[0][1], Some("85500".into()), "the sum of 85000 and 500");
        assert_eq!(b[0][1], Some("2".into()), "two postings on that account");
        assert_ne!(a, b, "two queries, two answers");
    }

    #[test]
    fn a_query_naming_one_account_does_not_answer_for_another() {
        // The other half: the key comes from the *predicate the compiler lowered*, not from
        // a digit scanner over the query text.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 2002 group by acct".into(),
            ),
            &mut e,
        );
        let rows: Vec<&Vec<Option<String>>> = out
            .iter()
            .filter_map(|m| match m {
                Backend::DataRow(r) => Some(r),
                _ => None,
            })
            .collect();
        assert_eq!(rows.len(), 1, "{out:?}");
        assert_eq!(rows[0][0], Some("2002".into()));
        assert_eq!(rows[0][1], Some("12500".into()));
    }

    #[test]
    fn an_insert_appends_and_is_visible_to_the_next_read() {
        // The wire surface used to be read-only: an `INSERT` was wrapped in a view and
        // rejected, so half a database was unreachable over the protocol §6.9's adoption
        // argument rests on.
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("insert into postings values (4, 3003, 0, 700)".into()),
            &mut e,
        );
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t == "INSERT 0 1")),
            "{out:?}"
        );
        let read = s.handle(
            Frontend::Query(
                "select acct, sum(amt) from postings where acct = 3003 group by acct".into(),
            ),
            &mut e,
        );
        assert!(
            read.iter()
                .any(|m| matches!(m, Backend::DataRow(r) if r[1] == Some("700".into()))),
            "{read:?}"
        );
    }

    #[test]
    fn a_transaction_seals_as_one_epoch_and_a_rollback_seals_nothing() {
        let (mut s, mut e) = (session(), engine());
        s.handle(Frontend::Query("begin".into()), &mut e);
        s.handle(
            Frontend::Query("insert into postings values (10, 4004, 0, 100)".into()),
            &mut e,
        );
        s.handle(
            Frontend::Query("insert into postings values (11, 4004, 0, -100)".into()),
            &mut e,
        );
        let before = e.frontier();
        s.handle(Frontend::Query("commit".into()), &mut e);
        assert_eq!(
            e.frontier(),
            before + 1,
            "two statements, one epoch: there is no moment in which half of it happened"
        );
        assert_eq!(e.appended.len(), 1);
        assert_eq!(e.appended[0].1, 2, "both rows in the one sealed set");

        // And the rollback: buffered rows are discarded, and nothing was sealed to
        // compensate for.
        s.handle(Frontend::Query("begin".into()), &mut e);
        s.handle(
            Frontend::Query("insert into postings values (12, 5005, 0, 1)".into()),
            &mut e,
        );
        let f = e.frontier();
        let out = s.handle(Frontend::Query("rollback".into()), &mut e);
        assert_eq!(e.frontier(), f, "a rollback seals nothing");
        assert!(
            out.iter()
                .any(|m| matches!(m, Backend::CommandComplete(t) if t == "ROLLBACK 1")),
            "and says how many rows it discarded: {out:?}"
        );
    }

    #[test]
    fn a_repeated_transaction_identity_is_refused_with_a_unique_violation() {
        // Idempotency over a wire that can drop an acknowledgement. The SQLSTATE matters:
        // a driver reads it, and `23505` says "you already did this" while the `42P01`
        // this server used to answer to everything says "no such table".
        let (mut s, mut e) = (session(), engine());
        s.handle(
            Frontend::Query("insert into postings values (7, 6006, 0, 5)".into()),
            &mut e,
        );
        let again = s.handle(
            Frontend::Query("insert into postings values (7, 6006, 0, 5)".into()),
            &mut e,
        );
        let Some(Backend::ErrorResponse { code, .. }) = again.first() else {
            panic!("{again:?}")
        };
        assert_eq!(code, "23505");
    }

    #[test]
    fn an_update_against_the_ledger_is_refused_with_feature_not_supported() {
        let (mut s, mut e) = (session(), engine());
        let out = s.handle(
            Frontend::Query("update postings set amt = 0 where acct = 1001".into()),
            &mut e,
        );
        let Some(Backend::ErrorResponse { code, detail, .. }) = out.first() else {
            panic!("{out:?}")
        };
        assert_eq!(code, "0A000");
        assert!(detail
            .as_deref()
            .unwrap_or_default()
            .contains("compensating entry"));
    }

    #[test]
    fn a_diagnostic_carries_a_sqlstate_from_its_own_family() {
        // Every Niles diagnostic used to reach the client as `42P01` — *undefined_table* —
        // so a driver's retry logic was told the same untrue thing about a syntax error, a
        // currency mismatch and a rung violation alike.
        assert_eq!(pg_wire::sqlstate_for("NL0001"), "42601");
        assert_eq!(pg_wire::sqlstate_for("NL0250"), "42804");
        assert_eq!(pg_wire::sqlstate_for("NL0300"), "23000");
        assert_eq!(
            pg_wire::sqlstate_for("NL0312"),
            "42501",
            "a missing capability is a privilege failure, not an integrity one: a driver \
             retries the second and must not retry the first"
        );
        assert_eq!(pg_wire::sqlstate_for("NL0400"), "42P20");
        assert_eq!(pg_wire::sqlstate_for("NL0501"), "0A000");
        assert_eq!(pg_wire::sqlstate_for("IR013"), "58000");
        assert_eq!(
            pg_wire::sqlstate_for("ZZ9999"),
            "XX000",
            "an unclassified code is an internal error, not a guess"
        );
    }
}
