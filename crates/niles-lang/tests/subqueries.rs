//! **Correlated subqueries, from source text to a circuit the optimizer can unnest.**
//!
//! `nilestream-optimizer::unnest` turns a dependent join into a set-at-a-time one, and
//! `results/E17-unnesting.md` measures what that is worth. None of it mattered while the
//! rewrite was unreachable: the surface had no `exists`, and `lower.rs` produced no
//! `Apply`, so the corpus built its circuits by hand and no query a user could write ever
//! arrived at the rewrite. This file is the path being closed.
//!
//! Three things are checked: that the four predicate forms parse, that they lower to an
//! `Apply` with the **correlation actually extracted**, and that a form which cannot be
//! lowered is refused rather than approximated.
//!
//! # The two defects this exercise found
//!
//! Writing `where t.z = 1` and reading the circuit turned up a defect that had nothing to
//! do with subqueries and was considerably worse than the feature being added.
//!
//! 1. **`=` in a SQL `where` clause parsed as an assignment.** Niles's two ancestries
//!    disagree about one character: in Rust `a = b` assigns, in SQL it compares, and SQL
//!    has no assignment expression at all. The parser took the Rust reading everywhere, so
//!    `select k from t where t.z = 1` produced an `Assign` node.
//! 2. **Lowering then silently discarded it.** The three predicate sites read
//!    `self.scalar(..).unwrap_or(Scalar::LitBool(true))`, so a predicate with no lowering
//!    became the constant `true`: the `where` clause vanished and the view returned every
//!    row. The query looked correct, the plan verified, and no answer-level test could
//!    catch it, because every row it returned was a real row.
//!
//! Together those are `Err(_) => 0` at plan level. Both are fixed and both are pinned
//! below, because the second one in particular is the kind of defect that comes back the
//! next time someone needs a "reasonable default".

use niles_lang::ast::*;
use niles_lang::{lower, parser, resolve};
use niles_ir::circuit::Circuit;
use niles_ir::operator::{ApplyKind, Op, Scalar, ScalarOp};

const SCHEMA: &str = "schema s {\n  \
    base t { k: i64, x: i64, z: i64 }\n  \
    base u { k: i64, y: i64 }\n";

fn circuit_of(query: &str) -> (Circuit, bool) {
    let src = format!("{SCHEMA}  view v = sql {{ {query} }};\n}}\n");
    let (p, pd) = parser::parse_program(&src);
    assert!(!pd.has_errors(), "`{query}` must parse: {:?}", pd.items);
    let (cat, _) = resolve::resolve_program(&p, 0);
    let (lo, d) = lower::lower_program(&p, &cat);
    (lo.circuit, d.has_errors())
}

fn the_apply(c: &Circuit) -> (&ApplyKind, &Vec<(u16, u16)>) {
    let n = c
        .nodes
        .iter()
        .find(|n| matches!(n.op, Op::Apply { .. }))
        .unwrap_or_else(|| panic!("no dependent join in:\n{}", c.explain()));
    match &n.op {
        Op::Apply { kind, correlation } => (kind, correlation),
        _ => unreachable!(),
    }
}

// ── the four predicate forms ─────────────────────────────────────────────────────────

#[test]
fn a_correlated_exists_lowers_to_a_dependent_join() {
    let (c, err) = circuit_of("select k from t where exists (select 1 from u where u.k = t.k)");
    assert!(!err);
    let (kind, corr) = the_apply(&c);
    assert!(matches!(kind, ApplyKind::Exists));
    assert_eq!(corr, &vec![(0u16, 0u16)], "the correlation must be extracted, not left as a filter");
}

#[test]
fn a_correlated_not_exists_lowers_to_a_dependent_join() {
    let (c, err) = circuit_of("select k from t where not exists (select 1 from u where u.k = t.k)");
    assert!(!err);
    let (kind, corr) = the_apply(&c);
    assert!(matches!(kind, ApplyKind::NotExists), "`not exists` is `not` over `exists`");
    assert_eq!(corr, &vec![(0u16, 0u16)]);
}

#[test]
fn an_in_subquery_carries_both_the_probe_and_the_correlation() {
    let (c, err) = circuit_of("select k from t where t.x in (select u.y from u where u.k = t.k)");
    assert!(!err);
    let (kind, corr) = the_apply(&c);
    // The probe is `t.x`, column 1 of the outer; the inner column is `u.y`, column 1 of u.
    assert!(matches!(kind, ApplyKind::In { probe: 1, inner: 1 }), "{kind:?}");
    assert_eq!(corr, &vec![(0u16, 0u16)]);
}

#[test]
fn an_uncorrelated_not_in_lowers_with_an_empty_correlation() {
    let (c, err) = circuit_of("select k from t where t.x not in (select u.y from u)");
    assert!(!err);
    let (kind, corr) = the_apply(&c);
    assert!(matches!(kind, ApplyKind::NotIn { probe: 1, inner: 1 }), "{kind:?}");
    assert!(corr.is_empty(), "no correlation was written, so none may be invented");
}

// ── the correlation, and the defect a naive rule would have caused ───────────────────

#[test]
fn the_qualifier_decides_which_side_a_correlation_column_is_on() {
    // `where u.k = t.k`: both sides are called `k`, and both names resolve in both
    // schemas. A rule that asked only "which schema does this name resolve in" would find
    // it resolves in both, give up, and leave the equality as a *filter on the inner
    // side* — where it lowers to `k = k`, always true, so the subquery matches everything
    // and the `exists` becomes a no-op. That is what the first version did.
    let (c, _) = circuit_of("select k from t where exists (select 1 from u where u.k = t.k)");
    let (_, corr) = the_apply(&c);
    assert_eq!(corr.len(), 1, "the correlation must be found even when both columns share a name");

    // And there must be no leftover filter on the inner side pretending to be the same
    // predicate. If one appeared, the correlation would be applied twice — once as a key
    // and once as a tautology.
    let filters: Vec<&Scalar> = c
        .nodes
        .iter()
        .filter_map(|n| match &n.op {
            Op::Filter { predicate } => Some(predicate),
            _ => None,
        })
        .collect();
    assert!(filters.is_empty(), "the correlation was also left behind as a filter: {filters:?}");
}

#[test]
fn a_local_predicate_in_the_subquery_stays_on_the_inner_side() {
    // The other half of the split: `u.y > 5` mentions only the inner relation, so it is a
    // filter under the apply rather than a correlation.
    let (c, err) = circuit_of("select k from t where exists (select 1 from u where u.k = t.k and u.y > 5)");
    assert!(!err);
    let (_, corr) = the_apply(&c);
    assert_eq!(corr, &vec![(0u16, 0u16)]);
    let has_local = c.nodes.iter().any(|n| {
        matches!(&n.op, Op::Filter { predicate: Scalar::Binary { op: ScalarOp::Gt, .. } })
    });
    assert!(has_local, "the local predicate must survive as a filter:\n{}", c.explain());
}

#[test]
fn a_conjunct_beside_a_subquery_becomes_an_ordinary_filter() {
    let (c, err) = circuit_of("select k from t where t.z > 1 and exists (select 1 from u where u.k = t.k)");
    assert!(!err);
    let (_, corr) = the_apply(&c);
    assert_eq!(corr.len(), 1);
    // `t.z` is column 2 of the outer, and the apply preserves the outer schema — so the
    // index must still be 2 after it. An apply that widened the row would shift this.
    let ok = c.nodes.iter().any(|n| {
        matches!(&n.op, Op::Filter { predicate: Scalar::Binary { op: ScalarOp::Gt, lhs, .. } }
            if **lhs == Scalar::Column(2))
    });
    assert!(ok, "the residual predicate's column index shifted:\n{}", c.explain());
}

// ── what is refused ──────────────────────────────────────────────────────────────────

#[test]
fn a_subquery_under_an_or_is_refused_rather_than_approximated() {
    // An `Apply` is a pipeline node; it cannot be one arm of a disjunction without first
    // becoming a semi-join and a union, which is a different rewrite and is not
    // implemented. Refusing loudly beats lowering something that is not the query written.
    let (c, err) = circuit_of("select k from t where t.z > 1 or exists (select 1 from u where u.k = t.k)");
    assert!(err, "this must be an error, not a silently different query");
    assert!(!c.nodes.iter().any(|n| matches!(n.op, Op::Apply { .. })));
}

#[test]
fn a_multi_column_in_subquery_is_refused_by_name() {
    let src = format!(
        "{SCHEMA}  view v = sql {{ select k from t where t.x in (select u.k, u.y from u) }};\n}}\n"
    );
    let (p, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&p, 0);
    let (_, d) = lower::lower_program(&p, &cat);
    assert!(d.items.iter().any(|x| x.code == "NL0502"), "{:?}", d.items);
}

// ── the two defects, pinned ──────────────────────────────────────────────────────────

#[test]
fn equals_in_a_sql_where_clause_is_a_comparison_and_not_an_assignment() {
    // Defect 1. Before the fix this parsed as `Expr::Assign`.
    let (p, _) = parser::parse_program("view v = sql { select k from t where t.z = 1 };");
    let Some(Item::View(v)) = p.items.first() else { panic!() };
    let Expr::Sql { inner, .. } = &v.body else { panic!() };
    let Expr::Select(s) = &**inner else { panic!() };
    assert!(
        matches!(s.filter, Some(Expr::Binary { op: BinOp::Eq, .. })),
        "`=` in a `where` clause is equality: {:?}",
        s.filter
    );
}

#[test]
fn assignment_still_exists_outside_a_sql_statement() {
    // The counterpart, so the fix did not simply delete assignment. Inside a function body
    // Niles is Rust and `a = b` assigns.
    let (p, _) = parser::parse_program("fn f() { a = 1; }");
    let rendered = niles_lang::sexpr::program(&p);
    assert!(rendered.contains("(assign (path a) (int 1))"), "{rendered}");
}

#[test]
fn a_where_clause_reaches_the_circuit_as_a_predicate() {
    // Defect 1 and 2 together, at the level a user would notice: the filter must actually
    // filter. Before the fix this circuit held `Filter { predicate: LitBool(true) }` and
    // the view returned every row.
    let (c, err) = circuit_of("select k from t where t.z = 1");
    assert!(!err);
    let preds: Vec<&Scalar> = c
        .nodes
        .iter()
        .filter_map(|n| match &n.op {
            Op::Filter { predicate } => Some(predicate),
            _ => None,
        })
        .collect();
    assert_eq!(preds.len(), 1, "{}", c.explain());
    assert!(
        matches!(preds[0], Scalar::Binary { op: ScalarOp::Eq, .. }),
        "the `where` clause was discarded: {:?}",
        preds[0]
    );
    assert_ne!(*preds[0], Scalar::LitBool(true), "the silent-true fallback is gone");
}

#[test]
fn an_unlowerable_predicate_is_an_error_and_not_a_default() {
    // Defect 2, stated as its own rule. There is no safe default: `true` returns rows that
    // should have been filtered out and `false` hides rows that exist, so the only honest
    // behaviour is to refuse the view and say which predicate could not be expressed.
    let src = format!("{SCHEMA}  view v = sql {{ select k from t where t.z > 1 or exists (select 1 from u) }};\n}}\n");
    let (p, _) = parser::parse_program(&src);
    let (cat, _) = resolve::resolve_program(&p, 0);
    let (lo, d) = lower::lower_program(&p, &cat);
    assert!(d.items.iter().any(|x| x.code == "NL0501"), "{:?}", d.items);
    assert!(
        !lo.circuit.nodes.iter().any(|n| matches!(&n.op, Op::Filter { predicate: Scalar::LitBool(true) })),
        "an unlowerable predicate must not become `true`:\n{}",
        lo.circuit.explain()
    );
    // And only one message for one problem: the specific diagnostic replaces the generic
    // "this view has no lowering", rather than joining it.
    assert!(!d.items.iter().any(|x| x.code == "NL0500"), "{:?}", d.items);
}

// ── the whole path, end to end ───────────────────────────────────────────────────────

#[test]
fn a_query_a_user_can_write_reaches_the_unnesting_rewrite() {
    // The point of the exercise. Source text in, a semi-join out — which is what makes
    // E17's measurement a statement about the *language* rather than about a corpus of
    // hand-built circuits.
    let (c, err) = circuit_of("select k from t where exists (select 1 from u where u.k = t.k)");
    assert!(!err);
    let (flat, report) = nilestream_optimizer::unnest::unnest(&c);
    assert!(report.is_complete(), "{}", report.render());
    assert_eq!(report.fired.len(), 1);
    assert_eq!(report.fired[0].name(), "exists-to-semi-join");
    assert!(flat.nodes.iter().any(|n| matches!(
        &n.op,
        Op::Join { kind: niles_ir::operator::JoinKind::Semi, .. }
    )));
    assert!(!flat.nodes.iter().any(|n| matches!(n.op, Op::Apply { .. })));
}

#[test]
fn all_four_surface_forms_reach_a_rewrite() {
    let cases = [
        ("select k from t where exists (select 1 from u where u.k = t.k)", "exists-to-semi-join"),
        ("select k from t where not exists (select 1 from u where u.k = t.k)", "not-exists-to-anti-join"),
        ("select k from t where t.x in (select u.y from u where u.k = t.k)", "in-to-semi-join"),
        ("select k from t where t.x not in (select u.y from u)", "not-in-to-anti-join-with-null-witness"),
    ];
    for (q, want) in cases {
        let (c, err) = circuit_of(q);
        assert!(!err, "`{q}` must lower");
        let (_, report) = nilestream_optimizer::unnest::unnest(&c);
        assert_eq!(report.fired.first().map(|r| r.name()), Some(want), "`{q}`");
    }
}
