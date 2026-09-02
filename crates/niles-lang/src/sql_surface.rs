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
    /// Both spellings denote the same Z-set on the golden corpus's dataset, checked by
    /// `tests/sql_golden.rs`. **Stronger than the circuit equality this used to mean:** two
    /// circuits can be structurally different and denote the same thing, and structurally
    /// identical while both being wrong.
    Equivalent,
    /// Lowered and evaluated, with a golden case fixing what it denotes — but written in
    /// one surface only, so there is no cross-surface claim to make.
    Lowered,
    /// **Refused**, with the diagnostic code that refuses it. This is what narrowing the
    /// fragment looks like from inside the code: a form nobody can write by accident,
    /// because the compiler says no and says which rule.
    Refused(&'static str),
    /// Lowered, and **nothing checks what it denotes**. The reason is recorded, and the
    /// status exists because the alternative was to call these `Lowered` alongside forms
    /// that have a corpus case behind them — which is how a table stops meaning anything.
    Untested(&'static str),
    /// In the fragment's specification but not implemented in stage 0.
    Specified,
    /// Deliberately outside the fragment, with the reason recorded.
    Excluded(&'static str),
}

/// **The stated fragment.** This is the normative list Appendix B.17 prints, and the thing
/// the generality result quantifies over. Nothing outside it is claimed.
pub static MAPPING: &[Mapping] = &[
    // Every status below is named by a case in `tests/golden/`, and
    // `the_status_of_every_form_is_backed_by_a_corpus_case` fails if one is not.
    Mapping { sql: "SELECT a, b FROM t", niles: "t.map(|r| (r.a, r.b))", status: Status::Equivalent },      // 02, 03
    Mapping { sql: "SELECT * FROM t", niles: "t", status: Status::Equivalent },                             // 01
    Mapping { sql: "SELECT * FROM t WHERE p", niles: "t.where(|r| p)", status: Status::Equivalent },        // 04, 05
    Mapping { sql: "SELECT k, sum(v) FROM t GROUP BY k", niles: "t.group_by(|r| r.k).sum(|r| r.v)", status: Status::Equivalent }, // 08
    Mapping { sql: "count / min / max / avg", niles: ".count(..) / .min(..) / .max(..) / .avg(..)", status: Status::Equivalent }, // 09, 10, 11, 40
    Mapping { sql: "SELECT sum(v) FROM t", niles: "(no pipeline spelling)", status: Status::Lowered },      // 26
    Mapping { sql: "GROUP BY k HAVING h", niles: ".group_by(|r| r.k).having(|g| h)", status: Status::Lowered },  // 12
    Mapping { sql: "JOIN u ON c", niles: ".join(u)", status: Status::Lowered },                             // 17
    Mapping { sql: "LEFT JOIN u ON c", niles: ".left_join(u)", status: Status::Lowered },                   // 18
    Mapping { sql: "RIGHT JOIN u ON c", niles: ".right_join(u)", status: Status::Equivalent },              // 50
    Mapping { sql: "FULL JOIN u ON c", niles: ".full_outer_join(u)", status: Status::Equivalent },          // 51
    // **Refused, and the reason is worth the row.** Every join operator in the IR joins on
    // a key and there is no product operator, so `cross join` lowered to the keyed inner
    // join and answered a different query -- four rows where twelve were asked for.
    Mapping { sql: "CROSS JOIN u", niles: "(none)", status: Status::Refused("NL0516") },                    // 52
    Mapping { sql: "JOIN u USING (k)", niles: "(none)", status: Status::Refused("NL0001") },                // 56
    Mapping { sql: "COUNT(DISTINCT x)", niles: "(none)", status: Status::Refused("NL0002") },               // 57
    Mapping { sql: "CASE WHEN .. THEN .. ELSE .. END", niles: "(none)", status: Status::Refused("NL0508") },// 53
    Mapping { sql: "WITH x AS (..) SELECT .. (non-recursive)", niles: "(none)", status: Status::Refused("NL0500") }, // 54, 55
    Mapping { sql: "FROM t, u", niles: "(no pipeline spelling)", status: Status::Lowered },                 // 19
    Mapping { sql: "UNION", niles: ".union(u)", status: Status::Lowered },                                  // 14
    Mapping { sql: "UNION ALL", niles: ".union_all(u)", status: Status::Lowered },                          // 13
    Mapping { sql: "EXCEPT", niles: ".except(u)", status: Status::Lowered },                                // 15
    Mapping { sql: "INTERSECT", niles: ".intersect(u)", status: Status::Lowered },                          // 16
    Mapping { sql: "DISTINCT", niles: ".distinct()", status: Status::Equivalent },                          // 07, 29
    Mapping { sql: "IS NULL / IS NOT NULL", niles: "is null / is not null", status: Status::Equivalent },   // 20, 21
    Mapping { sql: "NOT IN with nulls", niles: "(no pipeline spelling)", status: Status::Lowered },         // 39
    Mapping { sql: "EXISTS (correlated)", niles: "(no pipeline spelling)", status: Status::Lowered },       // 37
    Mapping { sql: "ORDER BY x LIMIT n [OFFSET m]", niles: ".order_by(|r| r.x).limit(n)", status: Status::Equivalent }, // 22, 23
    Mapping { sql: "ORDER BY <aggregate>", niles: "(no pipeline spelling: `order_by` has no descending form)", status: Status::Lowered }, // 58
    Mapping { sql: "ORDER BY <projection alias>", niles: "(no pipeline spelling: the stages have no aliases)", status: Status::Lowered }, // 59
    Mapping { sql: "ORDER BY <unknown column>", niles: ".order_by(|r| r.unknown)", status: Status::Refused("NL0509") }, // 60
    Mapping { sql: "LIMIT <non-literal>", niles: ".limit(<non-literal>)", status: Status::Refused("NL0504") }, // 24
    Mapping { sql: "a scalar subquery in the projection list", niles: "(none)", status: Status::Refused("NL0508") }, // 38
    Mapping { sql: "a set operation between different arities", niles: "(none)", status: Status::Refused("NL0512") }, // 27
    Mapping { sql: "SELECT with no FROM", niles: "(none)", status: Status::Refused("NL0511") },             // 35
    Mapping { sql: "WITH RECURSIVE", niles: ".fixpoint(|acc| ..) guard measure(m)", status: Status::Specified },  // 31, 36
    Mapping { sql: "CREATE MATERIALIZED VIEW", niles: "view .. serve { materialize: full }", status: Status::Lowered }, // 01
    Mapping { sql: "AS OF SYSTEM TIME", niles: ".as_of(#e)", status: Status::Lowered }, // 41
    Mapping { sql: "FOR SYSTEM_TIME", niles: ".recorded_at / .valid_at / bitemporal", status: Status::Specified },
    Mapping {
        sql: "INSERT / UPDATE / DELETE",
        niles: "same, on `table` only",
        status: Status::Untested(
            "these lower, and nothing in this repository checks what they compute. DML \
             denotation is out of the golden corpus's scope, which is queries; saying \
             `lowered` beside forms that have a case behind them would make the word \
             mean two things",
        ),
    },
    Mapping {
        sql: "BEGIN / COMMIT",
        niles: "txn { .. } (base) / begin..commit (table)",
        status: Status::Untested(
            "transaction control is a property of the write path, and the golden corpus \
             evaluates read models. What a `txn` does is tested in the kernel and in the \
             conservation suite, not here",
        ),
    },
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
            Status::Equivalent => {
                "**equivalent** — both surfaces denote the same Z-set on the golden corpus"
                    .to_string()
            }
            Status::Lowered => "lowered, with a golden case fixing what it denotes".to_string(),
            Status::Refused(code) => format!("**refused** — `{code}`"),
            Status::Untested(why) => format!("lowered, *untested* — {why}"),
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
        assert!(t.contains("**equivalent** — both surfaces denote the same Z-set"));
        assert!(t.contains("*excluded*"));
        assert!(
            t.contains("specified, not in stage 0"),
            "the table must not present unbuilt rows as built"
        );
        // And the status that did not exist before: a form the compiler refuses, named
        // with the code that refuses it. A fragment with no refusals is one nobody has
        // tried to leave.
        assert!(
            t.contains("**refused**"),
            "the table must say which forms are refused, and with what"
        );
    }
}
