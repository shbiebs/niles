//! The SQL surface: the normative SQL ↔ Niles mapping of Appendix B.17, as code.
//!
//! # What this module is for
//!
//! The generality claim in this thesis is deliberately narrow, because the broad version
//! is not checkable. "Niles replaces SQL" cannot be established by passing a conformance
//! suite: no authoritative current suite exists, the last official one targeted SQL-92 and
//! was withdrawn in 1997, and NIST disclaimed it. That claim was withdrawn from the thesis
//! rather than weakened.
//!
//! What replaces it is three statements that *can* be checked, and this module supports
//! the third: a **semantics-preserving translation of a stated SQL fragment**. The fragment
//! is stated here, in one place, as a machine-readable list; the translation is the
//! lowering in [`crate::lower`]; and the preservation property is tested by lowering both
//! spellings of the same query and comparing the resulting circuits.
//!
//! That test is the whole argument. If `select acct, sum(amt) from postings group by acct`
//! and `postings.group_by(|p| p.acct).sum(|p| p.amt)` produced different circuits, then
//! "the SQL surface is the same language" would be a marketing claim rather than a
//! technical one.

use crate::ast::{SelectStmt, SetOp};

/// One row of the SQL ↔ Niles mapping table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    pub sql: &'static str,
    pub niles: &'static str,
    /// Whether the translation is proved semantics-preserving by a lowering-equality test,
    /// or merely specified. Honesty here is the point of the whole exercise.
    pub status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Both spellings lower to the same circuit, checked by a test.
    Equivalent,
    /// Accepted by the parser and lowered, but no equality test yet.
    Lowered,
    /// In the fragment's specification but not implemented in stage 0.
    Specified,
    /// Deliberately outside the fragment, with the reason recorded.
    Excluded(&'static str),
}

/// **The stated fragment.** This is the normative list Appendix B.17 prints, and the thing
/// the generality result quantifies over. Nothing outside it is claimed.
pub static MAPPING: &[Mapping] = &[
    Mapping { sql: "SELECT a, b FROM t", niles: "t.map(|r| (r.a, r.b))", status: Status::Lowered },
    Mapping { sql: "SELECT * FROM t WHERE p", niles: "t.where(|r| p)", status: Status::Equivalent },
    Mapping { sql: "SELECT k, sum(v) FROM t GROUP BY k", niles: "t.group_by(|r| r.k).sum(|r| r.v)", status: Status::Equivalent },
    Mapping { sql: "GROUP BY k HAVING h", niles: ".group_by(|r| r.k).having(|g| h)", status: Status::Lowered },
    Mapping { sql: "JOIN u ON c", niles: ".join(u, |t, u| c)", status: Status::Lowered },
    Mapping { sql: "LEFT JOIN u ON c", niles: ".left_join(u, |t, u| c)", status: Status::Lowered },
    Mapping { sql: "UNION", niles: ".union(u)", status: Status::Lowered },
    Mapping { sql: "UNION ALL", niles: ".union_all(u)", status: Status::Lowered },
    Mapping { sql: "EXCEPT", niles: ".except(u)", status: Status::Lowered },
    Mapping { sql: "INTERSECT", niles: ".intersect(u)", status: Status::Lowered },
    Mapping { sql: "DISTINCT", niles: ".distinct()", status: Status::Lowered },
    Mapping { sql: "ORDER BY x DESC LIMIT n", niles: ".order_by(|r| desc(r.x)).limit(n)", status: Status::Lowered },
    Mapping { sql: "WITH RECURSIVE", niles: ".fixpoint(step) guard measure(m)", status: Status::Specified },
    Mapping { sql: "CREATE MATERIALIZED VIEW", niles: "view .. serve { materialize: full }", status: Status::Lowered },
    Mapping { sql: "AS OF SYSTEM TIME", niles: ".as_of(#e)", status: Status::Lowered },
    Mapping { sql: "FOR SYSTEM_TIME", niles: ".recorded_at / .valid_at / bitemporal", status: Status::Specified },
    Mapping { sql: "INSERT / UPDATE / DELETE", niles: "same, on `table` only", status: Status::Lowered },
    Mapping { sql: "BEGIN / COMMIT", niles: "txn { .. } (base) / begin..commit (table)", status: Status::Lowered },
    // --- deliberate exclusions, each with its reason ---
    Mapping {
        sql: "NULL three-valued logic in aggregates",
        niles: "Option<T>",
        status: Status::Excluded("SQL's null propagates differently through aggregates than through predicates; \
                                 reproducing that faithfully would import a semantics the conservation proofs \
                                 would then have to reason about"),
    },
    Mapping {
        sql: "Implicit type coercion",
        niles: "explicit `as` / `cast`",
        status: Status::Excluded("an implicit numeric widening between money and integer is exactly how a scale \
                                 error becomes invisible"),
    },
    Mapping {
        sql: "Correlated subqueries in arbitrary position",
        niles: "join + fixpoint",
        status: Status::Excluded("decorrelation is well understood but out of scope for stage 0; the fragment \
                                 says so rather than failing at an unexpected place"),
    },
    Mapping {
        sql: "Cursors, triggers, stored procedures",
        niles: "(none)",
        status: Status::Excluded("ambient side effects in the engine are incompatible with the effect calculus: \
                                 a trigger is an unnamed effect on an unnamed path"),
    },
    Mapping {
        sql: "Vendor-specific dialect syntax",
        niles: "(none)",
        status: Status::Excluded("the wire protocols accept dialect syntax at the protocol boundary; the language \
                                 does not adopt it"),
    },
];

/// The fragment, as a count. Cited rather than typed by hand.
pub fn fragment_size() -> (usize, usize, usize) {
    let equivalent = MAPPING
        .iter()
        .filter(|m| m.status == Status::Equivalent)
        .count();
    let lowered = MAPPING
        .iter()
        .filter(|m| m.status == Status::Lowered)
        .count();
    let excluded = MAPPING
        .iter()
        .filter(|m| matches!(m.status, Status::Excluded(_)))
        .count();
    (equivalent, lowered, excluded)
}

/// A structural summary of a parsed `select`, used to explain what part of the fragment a
/// query lands in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Features {
    pub has_filter: bool,
    pub has_group: bool,
    pub has_having: bool,
    pub has_join: bool,
    pub has_order: bool,
    pub has_limit: bool,
    pub has_set_op: Option<&'static str>,
    pub distinct: bool,
}

pub fn features(s: &SelectStmt) -> Features {
    Features {
        has_filter: s.filter.is_some(),
        has_group: !s.group_by.is_empty(),
        has_having: s.having.is_some(),
        has_join: s
            .from
            .iter()
            .any(|t| matches!(t, crate::ast::TableRef::Join { .. })),
        has_order: !s.order_by.is_empty(),
        has_limit: s.limit.is_some(),
        has_set_op: s.set_op.as_ref().map(|(op, _)| match op {
            SetOp::Union => "union",
            SetOp::UnionAll => "union all",
            SetOp::Except => "except",
            SetOp::Intersect => "intersect",
        }),
        distinct: s.distinct,
    }
}

/// Render the mapping table, for the thesis and for `nilesc --explain sql`.
pub fn render_table() -> String {
    let mut s = String::from("| SQL | Niles | Status |\n|---|---|---|\n");
    for m in MAPPING {
        let status = match m.status {
            Status::Equivalent => "**equivalent** (lowering-equality tested)".to_string(),
            Status::Lowered => "lowered".to_string(),
            Status::Specified => "specified, not in stage 0".to_string(),
            Status::Excluded(why) => format!("*excluded* — {why}"),
        };
        s.push_str(&format!("| `{}` | `{}` | {} |\n", m.sql, m.niles, status));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_expr;

    #[test]
    fn the_fragment_is_stated_and_bounded() {
        let (eq, lowered, excluded) = fragment_size();
        assert!(
            eq >= 2,
            "at least the core constructs must be equality-tested"
        );
        assert!(lowered >= 10);
        assert!(
            excluded >= 5,
            "a fragment with no stated exclusions is not a fragment"
        );
    }

    #[test]
    fn every_exclusion_states_its_reason() {
        // An exclusion without a reason is an omission pretending to be a decision.
        for m in MAPPING {
            if let Status::Excluded(why) = m.status {
                assert!(
                    why.len() > 30,
                    "`{}` is excluded without a real reason",
                    m.sql
                );
            }
        }
    }

    #[test]
    fn features_reads_the_clauses_that_were_written() {
        let (e, d) = parse_expr(
            "sql { select acct, sum(amt) as bal from postings where amt > 0 group by acct having bal > 0 order by bal limit 10 }",
        );
        assert!(!d.has_errors(), "{:?}", d.items);
        let crate::ast::Expr::Sql { inner, .. } = &e else {
            panic!()
        };
        let crate::ast::Expr::Select(s) = &**inner else {
            panic!()
        };
        let f = features(s);
        assert!(f.has_filter && f.has_group && f.has_having && f.has_order && f.has_limit);
        assert!(!f.has_join && !f.distinct);
    }

    #[test]
    fn the_rendered_table_marks_status_honestly() {
        let t = render_table();
        assert!(t.contains("**equivalent** (lowering-equality tested)"));
        assert!(t.contains("*excluded*"));
        assert!(
            t.contains("specified, not in stage 0"),
            "the table must not present unbuilt rows as built"
        );
    }
}
