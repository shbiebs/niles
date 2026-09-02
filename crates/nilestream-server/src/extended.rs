//! The extended query protocol, and the design question that blocked it.
//!
//! # Why this was refused rather than shipped
//!
//! Earlier versions of this server answered `Parse`, `Bind` and `Execute` with
//! `0A000 feature_not_supported` and a stated reason:
//!
//! > *a prepared statement must be cached against the epoch it was planned at, because a
//! > plan valid at one visibility frontier need not be valid at another.*
//!
//! That reason was correct and the refusal was the right call at the time, because
//! shipping a plan cache that ignored the problem would have produced the worst failure
//! this system can have: a client executes a prepared statement, the server serves it from
//! a plan compiled against a schema that no longer exists, and returns a well-formed answer
//! computed from the wrong definition. Nothing in the answer would look wrong.
//!
//! # The answer
//!
//! The problem dissolves once you notice that **the thing a plan depends on is itself
//! versioned by an epoch**. A schema change in Nilestream is a ledger fact: it has an epoch,
//! and epochs are totally ordered. So a plan is valid exactly while no schema epoch has
//! occurred after the one it was compiled at:
//!
//! ```text
//!   plan compiled at schema epoch S is valid at read anchor A
//!     ⟺  no schema change occurred in (S, A]
//! ```
//!
//! That is a comparison of two integers. It needs no invalidation callbacks, no version
//! vectors, no cache-coherence protocol between sessions, and no distributed agreement —
//! the same shape of answer the distributed read path gets for the same reason, and for
//! once a conventional database's hardest cache problem is genuinely easier here rather
//! than merely differently phrased.
//!
//! Three consequences follow, and the third is the one that matters:
//!
//! 1. A cached plan carries `schema_epoch`, and a mismatch is detected by comparison rather
//!    than by notification.
//! 2. A stale plan is **recompiled**, not refused. The client's prepared statement name
//!    keeps working across a migration, which is the entire point of preparing it.
//! 3. **A plan is never silently reused across a schema change.** That is the failure the
//!    refusal was protecting against, and it is now prevented by a check rather than by
//!    declining to implement the feature.
//!
//! # What is still not here
//!
//! Parameter *types* are inferred as text and coerced at bind time; there is no
//! `ParameterDescription` negotiation, and a client asking for binary format gets an error
//! naming that rather than silently receiving text. Portals are supported without suspension:
//! `Execute` with a non-zero row limit returns everything and does not emit
//! `PortalSuspended`, because row-limited execution over a partially materialized view needs
//! a cursor over reconstruction order, which is a real design question and not one this
//! thesis has answered.

use crate::pg_wire::{self, Backend, Field};
use std::collections::HashMap;

/// The epoch at which a schema was read. A plan's validity is a comparison against this.
pub type SchemaEpoch = u64;

/// A parsed, planned statement.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub name: String,
    pub sql: String,
    /// **The epoch the schema was read at when this was planned.** The whole cache-validity
    /// mechanism is this field plus one comparison.
    pub schema_epoch: SchemaEpoch,
    /// Output columns, known at plan time.
    pub fields: Vec<Field>,
    /// How many parameters the statement takes.
    pub param_count: usize,
    /// How many times this plan has been executed, and how many times it was recompiled
    /// because the schema moved. Both are reported by `nilestreamd`'s introspection, because
    /// a plan cache whose hit rate cannot be observed is a plan cache nobody can tune.
    pub executions: u64,
    pub recompilations: u64,
}

/// A bound portal: a prepared statement plus its arguments.
#[derive(Debug, Clone)]
pub struct Portal {
    pub name: String,
    pub statement: String,
    pub params: Vec<Option<String>>,
    /// Row limit from `Execute`. Zero means unlimited.
    pub max_rows: u32,
}

/// Why an extended-protocol request failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtError {
    UnknownStatement(String),
    UnknownPortal(String),
    /// A parameter count mismatch between `Parse` and `Bind`.
    ParamCount {
        expected: usize,
        got: usize,
    },
    /// Binary format requested. Named rather than silently downgraded, because a client
    /// that asked for binary and received text will misparse every value.
    BinaryFormatUnsupported,
    /// The statement did not compile. Carries the compiler's own code.
    Compile {
        code: String,
        message: String,
    },
}

impl ExtError {
    pub fn to_backend(&self) -> Backend {
        match self {
            ExtError::UnknownStatement(n) => Backend::ErrorResponse {
                severity: "ERROR".into(),
                code: "26000".into(), // invalid_sql_statement_name
                message: format!("prepared statement `{n}` does not exist"),
                detail: None,
            },
            ExtError::UnknownPortal(n) => Backend::ErrorResponse {
                severity: "ERROR".into(),
                code: "34000".into(), // invalid_cursor_name
                message: format!("portal `{n}` does not exist"),
                detail: None,
            },
            ExtError::ParamCount { expected, got } => Backend::ErrorResponse {
                severity: "ERROR".into(),
                code: "08P01".into(), // protocol_violation
                message: format!(
                    "bind message supplies {got} parameters but the statement requires {expected}"
                ),
                detail: None,
            },
            ExtError::BinaryFormatUnsupported => pg_wire::unsupported(
                "the binary parameter and result format",
                "this server sends text only; a client that asked for binary and received text \
                 would misparse every value, so the request is refused rather than downgraded",
            ),
            ExtError::Compile { code, message } => pg_wire::diagnostic_error(code, message, None),
        }
    }
}

/// The per-session plan cache.
///
/// Keyed by statement name, as the protocol requires. The unnamed statement (`""`) is
/// special in the protocol — it is replaced by each `Parse` — and that is honoured, because
/// clients that never name their statements rely on it.
#[derive(Debug, Default)]
pub struct PlanCache {
    statements: HashMap<String, Prepared>,
    portals: HashMap<String, Portal>,
    pub hits: u64,
    pub misses: u64,
    pub recompilations: u64,
}

impl PlanCache {
    pub fn new() -> PlanCache {
        PlanCache::default()
    }

    /// `Parse`: compile and cache.
    pub fn parse(
        &mut self,
        name: &str,
        sql: &str,
        schema_epoch: SchemaEpoch,
        fields: Vec<Field>,
        param_count: usize,
    ) -> &Prepared {
        self.misses += 1;
        let p = Prepared {
            name: name.to_string(),
            sql: sql.to_string(),
            schema_epoch,
            fields,
            param_count,
            executions: 0,
            recompilations: 0,
        };
        self.statements.insert(name.to_string(), p);
        &self.statements[name]
    }

    /// Look a plan up **and check it against the current schema epoch**.
    ///
    /// Returns `Ok(plan)` if it is still valid, and `Err(stale_sql)` if it must be
    /// recompiled — which the caller does, rather than failing. A client's prepared
    /// statement surviving a migration is the entire reason to prepare it.
    pub fn lookup(
        &mut self,
        name: &str,
        current_schema: SchemaEpoch,
    ) -> Result<&Prepared, Option<String>> {
        match self.statements.get(name) {
            None => Err(None),
            Some(p) if p.schema_epoch == current_schema => {
                self.hits += 1;
                Ok(self.statements.get(name).unwrap())
            }
            Some(p) => {
                // The schema moved. The plan is not wrong to have existed; it is wrong to
                // use now, and the difference between those two is why this returns the SQL
                // to recompile rather than an error.
                let sql = p.sql.clone();
                Err(Some(sql))
            }
        }
    }

    /// Record that a stale plan was recompiled at a new schema epoch.
    pub fn recompiled(&mut self, name: &str, schema_epoch: SchemaEpoch, fields: Vec<Field>) {
        self.recompilations += 1;
        if let Some(p) = self.statements.get_mut(name) {
            p.schema_epoch = schema_epoch;
            p.fields = fields;
            p.recompilations += 1;
        }
    }

    /// `Bind`: attach arguments to a plan.
    pub fn bind(
        &mut self,
        portal: &str,
        statement: &str,
        params: Vec<Option<String>>,
        max_rows: u32,
        binary: bool,
    ) -> Result<(), ExtError> {
        if binary {
            return Err(ExtError::BinaryFormatUnsupported);
        }
        let Some(p) = self.statements.get(statement) else {
            return Err(ExtError::UnknownStatement(statement.to_string()));
        };
        if params.len() != p.param_count {
            return Err(ExtError::ParamCount {
                expected: p.param_count,
                got: params.len(),
            });
        }
        self.portals.insert(
            portal.to_string(),
            Portal {
                name: portal.to_string(),
                statement: statement.to_string(),
                params,
                max_rows,
            },
        );
        Ok(())
    }

    pub fn portal(&self, name: &str) -> Result<&Portal, ExtError> {
        self.portals
            .get(name)
            .ok_or_else(|| ExtError::UnknownPortal(name.to_string()))
    }

    /// `Describe` on a statement: the row shape, without executing.
    pub fn describe(&self, name: &str) -> Result<Vec<Field>, ExtError> {
        self.statements
            .get(name)
            .map(|p| p.fields.clone())
            .ok_or_else(|| ExtError::UnknownStatement(name.to_string()))
    }

    pub fn note_execution(&mut self, statement: &str) {
        if let Some(p) = self.statements.get_mut(statement) {
            p.executions += 1;
        }
    }

    /// `Close`.
    pub fn close_statement(&mut self, name: &str) {
        self.statements.remove(name);
        self.portals.retain(|_, p| p.statement != name);
    }

    pub fn close_portal(&mut self, name: &str) {
        self.portals.remove(name);
    }

    pub fn statement_count(&self) -> usize {
        self.statements.len()
    }

    /// Cache hit ratio, for introspection. A plan cache whose hit rate cannot be observed
    /// is one nobody can tune.
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// The reply to `Execute` when a row limit was requested.
///
/// The protocol allows `PortalSuspended`, and this server does not send it: a row-limited
/// execution over a partially materialized view needs a cursor over *reconstruction order*,
/// and what that order should be — insertion, key, or the order the upqueries happened to
/// complete in — is an open design question rather than an implementation detail. Returning
/// everything and saying so is more honest than suspending a portal whose resumption
/// semantics are undefined.
pub fn row_limit_notice(max_rows: u32) -> Option<Backend> {
    if max_rows == 0 {
        return None;
    }
    Some(Backend::NoticeResponse {
        message: format!(
            "row limit {max_rows} ignored: this server returns the whole portal. Resuming a \
             suspended portal over a partially materialized view requires a cursor over \
             reconstruction order, which is not yet defined."
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields() -> Vec<Field> {
        vec![Field::int8("acct"), Field::numeric("balance")]
    }

    #[test]
    fn a_plan_is_reused_while_the_schema_stands_still() {
        let mut c = PlanCache::new();
        c.parse(
            "s1",
            "select acct, sum(amt) from postings where acct = $1 group by acct",
            100,
            fields(),
            1,
        );
        for _ in 0..10 {
            assert!(c.lookup("s1", 100).is_ok());
        }
        assert_eq!(c.hits, 10);
        assert!(c.hit_ratio() > 0.9);
    }

    #[test]
    fn a_plan_is_never_silently_reused_across_a_schema_change() {
        // The failure the original refusal was protecting against: a well-formed answer
        // computed from a definition that no longer exists. Nothing about it would look
        // wrong, which is why it is checked rather than hoped about.
        let mut c = PlanCache::new();
        c.parse("s1", "select x from t", 100, fields(), 0);
        let stale = c.lookup("s1", 101);
        assert!(
            matches!(stale, Err(Some(_))),
            "a moved schema must invalidate the plan"
        );
        assert_eq!(c.hits, 0, "and must not count as a hit");
    }

    #[test]
    fn a_stale_plan_is_recompiled_rather_than_refused() {
        // A prepared statement surviving a migration is the entire reason to prepare it.
        let mut c = PlanCache::new();
        c.parse("s1", "select x from t", 100, fields(), 0);
        let Err(Some(sql)) = c.lookup("s1", 200) else {
            panic!("expected a recompile request")
        };
        assert_eq!(
            sql, "select x from t",
            "the SQL comes back so the caller can recompile it"
        );
        c.recompiled("s1", 200, fields());
        assert!(
            c.lookup("s1", 200).is_ok(),
            "and the client's statement name keeps working"
        );
        assert_eq!(c.recompilations, 1);
    }

    #[test]
    fn validity_is_one_integer_comparison() {
        // The design point: no invalidation callbacks, no version vectors, no coherence
        // protocol between sessions. A schema change is a ledger fact with an epoch, and
        // epochs are totally ordered.
        let mut c = PlanCache::new();
        c.parse("s", "select 1", 50, fields(), 0);
        assert!(c.lookup("s", 50).is_ok());
        assert!(
            c.lookup("s", 49).is_err(),
            "an older schema is also a mismatch, not a match"
        );
        assert!(c.lookup("s", 51).is_err());
    }

    #[test]
    fn bind_checks_the_parameter_count() {
        let mut c = PlanCache::new();
        c.parse("s", "select $1, $2", 1, fields(), 2);
        assert_eq!(
            c.bind("p", "s", vec![Some("1".into())], 0, false),
            Err(ExtError::ParamCount {
                expected: 2,
                got: 1
            })
        );
        assert!(c
            .bind("p", "s", vec![Some("1".into()), None], 0, false)
            .is_ok());
    }

    #[test]
    fn binary_format_is_refused_by_name_rather_than_downgraded() {
        // A client that asked for binary and received text misparses every value, so this
        // is one of the few places where an error is strictly better than a best effort.
        let mut c = PlanCache::new();
        c.parse("s", "select 1", 1, fields(), 0);
        let e = c.bind("p", "s", vec![], 0, true).unwrap_err();
        assert_eq!(e, ExtError::BinaryFormatUnsupported);
        let Backend::ErrorResponse { code, detail, .. } = e.to_backend() else {
            panic!()
        };
        assert_eq!(code, "0A000");
        assert!(detail.unwrap().contains("misparse"));
    }

    #[test]
    fn binding_an_unknown_statement_is_the_right_sqlstate() {
        let mut c = PlanCache::new();
        let e = c.bind("p", "nope", vec![], 0, false).unwrap_err();
        let Backend::ErrorResponse { code, .. } = e.to_backend() else {
            panic!()
        };
        assert_eq!(
            code, "26000",
            "invalid_sql_statement_name, not a generic error"
        );
    }

    #[test]
    fn closing_a_statement_closes_the_portals_bound_to_it() {
        // Otherwise a portal outlives its plan and executes against a statement that no
        // longer exists.
        let mut c = PlanCache::new();
        c.parse("s", "select 1", 1, fields(), 0);
        c.bind("p1", "s", vec![], 0, false).unwrap();
        c.bind("p2", "s", vec![], 0, false).unwrap();
        assert!(c.portal("p1").is_ok());
        c.close_statement("s");
        assert!(c.portal("p1").is_err());
        assert!(c.portal("p2").is_err());
        assert_eq!(c.statement_count(), 0);
    }

    #[test]
    fn describe_reports_the_row_shape_without_executing() {
        let mut c = PlanCache::new();
        c.parse(
            "s",
            "select acct, sum(amt) from postings group by acct",
            1,
            fields(),
            0,
        );
        let f = c.describe("s").unwrap();
        assert_eq!(f.len(), 2);
        assert_eq!(f[1].type_oid, 1700, "money is still numeric on this path");
        assert_eq!(
            c.statements["s"].executions, 0,
            "describing must not execute"
        );
    }

    #[test]
    fn the_unnamed_statement_is_replaced_by_each_parse() {
        // Clients that never name their statements rely on this.
        let mut c = PlanCache::new();
        c.parse("", "select 1", 1, fields(), 0);
        c.parse("", "select 2", 1, fields(), 0);
        assert_eq!(c.statement_count(), 1);
        assert_eq!(c.lookup("", 1).unwrap().sql, "select 2");
    }

    #[test]
    fn a_row_limit_is_declined_with_the_open_question_named() {
        assert!(row_limit_notice(0).is_none(), "no limit, no notice");
        let Some(Backend::NoticeResponse { message }) = row_limit_notice(50) else {
            panic!()
        };
        assert!(
            message.contains("reconstruction order"),
            "the refusal must name why: {message}"
        );
    }
}
