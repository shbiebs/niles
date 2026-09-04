// The corpus builder takes one argument per dimension of a case (scale, cardinalities,
// nulls, correlation shape). Bundling them into a struct would move the same nine values
// one level down without making a call site clearer.
#![allow(clippy::too_many_arguments)]

//! **The unnesting corpus.**
//!
//! Twenty-four correlated queries, each built twice: once in nested form (an `Apply`) and
//! once as whatever `unnest` produces. Every one is checked three ways.
//!
//! 1. **Denotationally.** The two circuits are evaluated on the same dataset and the two
//!    Z-sets must be equal. This is the assertion that matters, and it is deliberately not
//!    "the plan contains a semi-join": a plan can contain a semi-join and compute the wrong
//!    thing, and a structural test would pass it.
//! 2. **Against a hand-written oracle**, for the eight `not in` cases. Denotational
//!    equivalence says the rewrite preserves the nested form's meaning; it says nothing
//!    about whether the nested form's meaning is *SQL's*. The oracle is written from the
//!    three-valued truth table directly, so a shared misunderstanding of `not in` cannot
//!    cancel between the two circuits.
//! 3. **Structurally**, but only as a cross-check: the unnested plan must hold no `Apply`,
//!    must pass `verify`, and must name the rewrite that fired.
//!
//! # The dataset
//!
//! One fixed dataset for every case, carrying the four things that break rewrites:
//! **duplicates** (so a semi-join that inflated would show), **retractions** (negative
//! weights, so a rewrite sound on bags but not on Z-sets would show), **nulls in the
//! probed column on each side**, and **empty correlation groups** (so a scalar subquery's
//! null-extension is exercised).
//!
//! # `not in` and the kill criterion
//!
//! The roadmap makes a single wrong `not in` case a kill criterion, so eight of the
//! twenty-four are `not in`: nulls on the left, nulls on the right, nulls on both, nulls
//! in a group that also matches, an empty subquery, an all-null subquery, the correlated
//! form and the uncorrelated one. Each is checked against the oracle as well as against
//! its own nested form.

use niles_ir::circuit::{internal_contract, Circuit, NodeId};
use niles_ir::eval::{add, row, run, Row, ZSet};
use niles_ir::operator::{Agg, ApplyKind, ColIdx, Op, Scalar, ScalarOp};
use niles_ir::value::Value;
use niles_ir::{verify, Consistency, Lineage, Materialize, Retention, ServeContract};
use nilestream_optimizer::unnest::{unnest, Rewrite};
use std::collections::BTreeMap;

// ── the dataset ──────────────────────────────────────────────────────────────────────

/// The dataset is **scale-parameterised**, and that turned out to matter.
///
/// The first version was one fixed instance of nine outer rows and ten inner ones, and
/// the `not in` cases did *more* counted work after unnesting than before. That is not a
/// defect in the rewrite: a dependent join costs `|outer| x |inner|`, and the unnested
/// `not in` is five operators — filter, anti-join, filter, project, distinct, anti-join —
/// each linear. At nine by ten the constant wins and the asymptote has not arrived.
///
/// Hiding that by loosening the assertion would have thrown away the actual result, which
/// is a *crossover*: the rewrite is a loss below a scale and a growing win above it, and
/// the scale at which it turns is a number worth reporting. So the corpus runs at several
/// `k` and E17 reports the curve.
///
/// Each block of `k` replicates the same group structure — duplicates, retractions, a null
/// in the probed column, an empty correlation group — under fresh keys, so the *shape* of
/// every case is scale-invariant and only the cardinality moves.
///
/// `orders(id, cust, amount)` — the outer relation. Column 1 is the correlation key and
/// column 2 the probe.
fn orders_at(k: usize) -> ZSet {
    let mut z = ZSet::new();
    for b in 0..k as i128 {
        let (c, o) = (b * 100, b * 10);
        add(&mut z, row(&[Some(o + 1), Some(c + 10), Some(100)]), 1);
        add(&mut z, row(&[Some(o + 2), Some(c + 10), Some(200)]), 1);
        add(&mut z, row(&[Some(o + 3), Some(c + 20), Some(100)]), 2); // a duplicate
        add(&mut z, row(&[Some(o + 4), Some(c + 20), None]), 1); // a null probe
        add(&mut z, row(&[Some(o + 5), Some(c + 30), Some(300)]), 1); // no matching group
        add(&mut z, row(&[Some(o + 6), Some(c + 40), Some(400)]), 1);
        add(&mut z, row(&[Some(o + 7), Some(c + 40), Some(999)]), 1);
        add(&mut z, row(&[Some(o + 8), Some(c + 50), Some(500)]), 3);
        add(&mut z, row(&[Some(o + 9), Some(c + 50), Some(500)]), -3); // cancels to nothing
    }
    z
}

/// `payments(cust, amount)` — the inner relation. Group `+40` carries a **null amount**,
/// which is the case `not in` turns on.
fn payments_at(k: usize) -> ZSet {
    let mut z = ZSet::new();
    for b in 0..k as i128 {
        let c = b * 100;
        add(&mut z, row(&[Some(c + 10), Some(100)]), 1);
        add(&mut z, row(&[Some(c + 10), Some(100)]), 1); // a duplicate, so a semi-join can inflate
        add(&mut z, row(&[Some(c + 10), Some(700)]), 1);
        add(&mut z, row(&[Some(c + 20), Some(100)]), 1);
        add(&mut z, row(&[Some(c + 40), None]), 1); // the null witness
        add(&mut z, row(&[Some(c + 40), Some(400)]), 1);
        add(&mut z, row(&[Some(c + 50), Some(500)]), 1);
        add(&mut z, row(&[Some(c + 60), Some(600)]), 1); // a group with no order
        add(&mut z, row(&[Some(c + 70), Some(700)]), 1);
        add(&mut z, row(&[Some(c + 70), Some(700)]), -1); // cancels
    }
    z
}

/// `flags(cust, amount)` — a second inner relation with **no nulls at all**, so the `not
/// in` cases can be run with and without a witness and the difference attributed.
fn flags_at(k: usize) -> ZSet {
    let mut z = ZSet::new();
    for b in 0..k as i128 {
        let c = b * 100;
        add(&mut z, row(&[Some(c + 10), Some(100)]), 1);
        add(&mut z, row(&[Some(c + 20), Some(999)]), 1);
        add(&mut z, row(&[Some(c + 40), Some(400)]), 1);
    }
    z
}

/// A relation that is entirely null in its probed column.
fn all_null_at(k: usize) -> ZSet {
    let mut z = ZSet::new();
    for b in 0..k as i128 {
        let c = b * 100;
        add(&mut z, row(&[Some(c + 10), None]), 1);
        add(&mut z, row(&[Some(c + 20), None]), 1);
        add(&mut z, row(&[Some(c + 40), None]), 1);
    }
    z
}

fn data_at(k: usize) -> BTreeMap<String, ZSet> {
    BTreeMap::from([
        ("orders".to_string(), orders_at(k)),
        ("payments".to_string(), payments_at(k)),
        ("flags".to_string(), flags_at(k)),
        ("all_null".to_string(), all_null_at(k)),
        ("empty".to_string(), ZSet::new()),
    ])
}

/// The unit instance, used by every correctness test. One block, so every structural
/// case — the duplicate, the retraction, the null, the empty group — is present exactly
/// once and a failure names one row rather than a hundred.
fn data() -> BTreeMap<String, ZSet> {
    data_at(1)
}

fn orders() -> ZSet {
    orders_at(1)
}

// ── building circuits ────────────────────────────────────────────────────────────────

fn served() -> ServeContract {
    ServeContract {
        consistency: Consistency::Snapshot,
        materialize: Materialize::Full,
        retain: Retention::Evictable,
        lineage: Lineage::Off,
    }
}

fn source(c: &mut Circuit, name: &str, arity: u16) -> NodeId {
    let id = c.add(
        Op::Source {
            relation: name.into(),
            is_base: true,
            anchor_key: vec![0],
            confidential: Vec::new(),
        },
        vec![],
        ServeContract {
            consistency: Consistency::LedgerConsistent,
            materialize: Materialize::Full,
            retain: Retention::Forever,
            lineage: Lineage::Off,
        },
        name,
    );
    // The reference sources have no rows until evaluation, so arity is declared here.
    c.nodes[id as usize].arity = arity;
    id
}

/// One corpus case: a nested circuit, and what the rewrite should be called.
pub struct Case {
    name: &'static str,
    nested: Circuit,
    expect: &'static str,
}

/// `outer |> apply(kind, corr) inner`, optionally with a filter on the inner side.
fn apply_case(
    name: &'static str,
    outer_rel: &str,
    outer_arity: u16,
    inner_rel: &str,
    inner_arity: u16,
    inner_filter: Option<Scalar>,
    kind: ApplyKind,
    correlation: Vec<(ColIdx, ColIdx)>,
    expect: &'static str,
) -> Case {
    let mut c = Circuit::new();
    let o = source(&mut c, outer_rel, outer_arity);
    let mut i = source(&mut c, inner_rel, inner_arity);
    if let Some(p) = inner_filter {
        i = c.add(
            Op::Filter { predicate: p },
            vec![i],
            internal_contract(),
            "inner filter",
        );
    }
    let a = c.add(Op::Apply { kind, correlation }, vec![o, i], served(), name);
    c.set_output("out", a);
    Case {
        name,
        nested: c,
        expect,
    }
}

fn gt(col: ColIdx, lit: i128) -> Scalar {
    Scalar::Binary {
        op: ScalarOp::Gt,
        lhs: Box::new(Scalar::Column(col)),
        rhs: Box::new(Scalar::LitInt(lit)),
    }
}

/// The twenty-four cases.
fn corpus() -> Vec<Case> {
    use ApplyKind::*;
    let corr = || vec![(1u16, 0u16)];
    vec![
        // --- exists / not exists ---
        apply_case(
            "exists, correlated",
            "orders",
            3,
            "payments",
            2,
            None,
            Exists,
            corr(),
            "exists-to-semi-join",
        ),
        apply_case(
            "exists, no matching group",
            "orders",
            3,
            "empty",
            2,
            None,
            Exists,
            corr(),
            "exists-to-semi-join",
        ),
        apply_case(
            "exists, with an inner filter",
            "orders",
            3,
            "payments",
            2,
            Some(gt(1, 300)),
            Exists,
            corr(),
            "exists-to-semi-join",
        ),
        apply_case(
            "exists, uncorrelated",
            "orders",
            3,
            "payments",
            2,
            None,
            Exists,
            vec![],
            "exists-to-semi-join",
        ),
        apply_case(
            "not exists, correlated",
            "orders",
            3,
            "payments",
            2,
            None,
            NotExists,
            corr(),
            "not-exists-to-anti-join",
        ),
        apply_case(
            "not exists, empty inner",
            "orders",
            3,
            "empty",
            2,
            None,
            NotExists,
            corr(),
            "not-exists-to-anti-join",
        ),
        apply_case(
            "not exists, with an inner filter",
            "orders",
            3,
            "payments",
            2,
            Some(gt(1, 300)),
            NotExists,
            corr(),
            "not-exists-to-anti-join",
        ),
        apply_case(
            "not exists, uncorrelated",
            "orders",
            3,
            "empty",
            2,
            None,
            NotExists,
            vec![],
            "not-exists-to-anti-join",
        ),
        // --- in ---
        apply_case(
            "in, correlated",
            "orders",
            3,
            "payments",
            2,
            None,
            In { probe: 2, inner: 1 },
            corr(),
            "in-to-semi-join",
        ),
        apply_case(
            "in, against a duplicated inner",
            "orders",
            3,
            "payments",
            2,
            None,
            In { probe: 2, inner: 1 },
            vec![],
            "in-to-semi-join",
        ),
        apply_case(
            "in, inner column all null",
            "orders",
            3,
            "all_null",
            2,
            None,
            In { probe: 2, inner: 1 },
            corr(),
            "in-to-semi-join",
        ),
        apply_case(
            "in, empty inner",
            "orders",
            3,
            "empty",
            2,
            None,
            In { probe: 2, inner: 1 },
            corr(),
            "in-to-semi-join",
        ),
        // --- not in: eight cases, because one wrong one is the kill criterion ---
        apply_case(
            "not in, correlated, nulls on the right",
            "orders",
            3,
            "payments",
            2,
            None,
            NotIn { probe: 2, inner: 1 },
            corr(),
            "not-in-to-anti-join-with-null-witness",
        ),
        apply_case(
            "not in, correlated, no nulls anywhere",
            "orders",
            3,
            "flags",
            2,
            None,
            NotIn { probe: 2, inner: 1 },
            corr(),
            "not-in-to-anti-join-with-null-witness",
        ),
        apply_case(
            "not in, uncorrelated, one null poisons everything",
            "orders",
            3,
            "payments",
            2,
            None,
            NotIn { probe: 2, inner: 1 },
            vec![],
            "not-in-to-anti-join-with-null-witness",
        ),
        apply_case(
            "not in, uncorrelated, no nulls",
            "orders",
            3,
            "flags",
            2,
            None,
            NotIn { probe: 2, inner: 1 },
            vec![],
            "not-in-to-anti-join-with-null-witness",
        ),
        apply_case(
            "not in, inner column entirely null",
            "orders",
            3,
            "all_null",
            2,
            None,
            NotIn { probe: 2, inner: 1 },
            corr(),
            "not-in-to-anti-join-with-null-witness",
        ),
        apply_case(
            "not in, empty inner is vacuously true",
            "orders",
            3,
            "empty",
            2,
            None,
            NotIn { probe: 2, inner: 1 },
            corr(),
            "not-in-to-anti-join-with-null-witness",
        ),
        apply_case(
            "not in, inner filtered to nothing",
            "orders",
            3,
            "payments",
            2,
            Some(gt(1, 100_000)),
            NotIn { probe: 2, inner: 1 },
            corr(),
            "not-in-to-anti-join-with-null-witness",
        ),
        apply_case(
            "not in, a group that both matches and has a null",
            "orders",
            3,
            "payments",
            2,
            Some(gt(0, 30)),
            NotIn { probe: 2, inner: 1 },
            corr(),
            "not-in-to-anti-join-with-null-witness",
        ),
        // --- correlated scalar subqueries ---
        apply_case(
            "scalar sum, correlated",
            "orders",
            3,
            "payments",
            2,
            None,
            Scalar {
                agg: Agg::Sum,
                expr: niles_ir::operator::Scalar::Column(1),
            },
            corr(),
            "scalar-to-outer-join-with-aggregate",
        ),
        apply_case(
            "scalar count over an empty group",
            "orders",
            3,
            "empty",
            2,
            None,
            Scalar {
                agg: Agg::Count,
                expr: niles_ir::operator::Scalar::Column(1),
            },
            corr(),
            "scalar-to-outer-join-with-aggregate",
        ),
        apply_case(
            "scalar min, with nulls in the group",
            "orders",
            3,
            "payments",
            2,
            None,
            Scalar {
                agg: Agg::Min,
                expr: niles_ir::operator::Scalar::Column(1),
            },
            corr(),
            "scalar-to-outer-join-with-aggregate",
        ),
        apply_case(
            "scalar max over a filtered inner",
            "orders",
            3,
            "payments",
            2,
            Some(gt(1, 200)),
            Scalar {
                agg: Agg::Max,
                expr: niles_ir::operator::Scalar::Column(1),
            },
            corr(),
            "scalar-to-outer-join-with-aggregate",
        ),
    ]
}

// ── the gate ─────────────────────────────────────────────────────────────────────────

#[test]
fn the_corpus_is_large_enough_and_weighted_toward_not_in() {
    let c = corpus();
    assert!(
        c.len() >= 20,
        "the roadmap asks for at least twenty, got {}",
        c.len()
    );
    let not_in = c.iter().filter(|x| x.name.starts_with("not in")).count();
    assert!(
        not_in >= 5,
        "at least five `not in` cases with nulls on each side, got {not_in}"
    );
}

#[test]
fn every_case_unnests_and_denotes_the_same_zset() {
    // The assertion that matters. Nested and unnested, same data, same answer — checked as
    // Z-sets, so a rewrite that got the *weights* wrong fails here even when the set of
    // rows is right.
    let src = data();
    let mut checked = 0;
    for case in corpus() {
        let (flat, report) = unnest(&case.nested);
        assert!(
            report.is_complete(),
            "`{}` was refused:\n{}",
            case.name,
            report.render()
        );
        assert_eq!(
            report.fired.len(),
            1,
            "`{}` should fire exactly one rewrite",
            case.name
        );
        assert_eq!(report.fired[0].name(), case.expect, "case `{}`", case.name);

        let (before, _) = run(&case.nested, "out", &src);
        let (after, _) = run(&flat, "out", &src);
        assert_eq!(
            before,
            after,
            "\ncase: {}\nnested:   {:?}\nunnested: {:?}\n\nnested plan:\n{}\nunnested plan:\n{}",
            case.name,
            before,
            after,
            case.nested.explain(),
            flat.explain()
        );
        checked += 1;
    }
    assert_eq!(checked, 24);
}

#[test]
fn the_unnested_plan_holds_no_apply_and_verifies() {
    for case in corpus() {
        let (flat, _) = unnest(&case.nested);
        assert!(
            !flat.nodes.iter().any(|n| matches!(n.op, Op::Apply { .. })),
            "`{}` still holds a dependent join",
            case.name
        );
        let r = verify::verify(&flat);
        // The access audit is not the subject here — a corpus circuit has no consumer that
        // reads every field — so only the structural and semantic codes are asserted.
        let real: Vec<_> = r.violations.iter().filter(|v| v.code != "IR020").collect();
        assert!(
            real.is_empty(),
            "`{}` failed verification: {real:?}",
            case.name
        );
    }
}

#[test]
fn the_nested_plan_is_refused_by_the_verifier() {
    // The other half: a circuit that still holds an `Apply` on a served path must not pass.
    // If it did, unnesting would be optional, and a view with no delta rule could be
    // served — which is the failure IR018 exists to prevent.
    for case in corpus() {
        let r = verify::verify(&case.nested);
        assert!(
            r.violations.iter().any(|v| v.code == "IR018"),
            "`{}`: a served dependent join must be rejected\n{}",
            case.name,
            r.render()
        );
    }
}

// ── the oracle: `not in`, written straight from the truth table ───────────────────────

/// SQL's `outer.probe NOT IN (select inner from R where correlation)`, computed directly.
///
/// Written from the three-valued rule rather than from either circuit, so that a shared
/// misunderstanding cannot cancel between the nested and unnested forms. Denotational
/// equivalence says the rewrite is faithful to the `Apply`; this says the `Apply` is
/// faithful to SQL.
fn not_in_oracle(
    outer: &ZSet,
    inner: &ZSet,
    correlation: &[(ColIdx, ColIdx)],
    probe: ColIdx,
    icol: ColIdx,
) -> ZSet {
    let mut out = ZSet::new();
    for (o, ow) in outer {
        let p = o[probe as usize];
        // `null NOT IN (…)` is unknown even against an empty subquery.
        if p.is_null() {
            continue;
        }
        let group: Vec<&Row> = inner
            .iter()
            .filter(|(_, w)| **w > 0)
            .filter(|(r, _)| {
                correlation.iter().all(|(a, b)| {
                    let (x, y) = (o[*a as usize], r[*b as usize]);
                    !x.is_null() && !y.is_null() && x == y
                })
            })
            .map(|(r, _)| r)
            .collect();
        let matched = group.iter().any(|r| r[icol as usize] == p);
        let any_null = group.iter().any(|r| r[icol as usize].is_null());
        // false (matched) and unknown (a null with no match) both drop the row.
        if !matched && !any_null {
            add(&mut out, o.clone(), *ow);
        }
    }
    out
}

/// The inner relation a case actually sees, after any filter below the apply.
fn effective_inner(c: &Circuit, src: &BTreeMap<String, ZSet>) -> ZSet {
    let apply = c
        .nodes
        .iter()
        .find(|n| matches!(n.op, Op::Apply { .. }))
        .expect("an apply");
    niles_ir::eval::run_node(c, apply.inputs[1], src).0
}

#[test]
fn every_not_in_case_matches_a_hand_written_three_valued_oracle() {
    let src = data();
    let mut checked = 0;
    for case in corpus()
        .into_iter()
        .filter(|c| c.name.starts_with("not in"))
    {
        let apply = case
            .nested
            .nodes
            .iter()
            .find(|n| matches!(n.op, Op::Apply { .. }))
            .unwrap();
        let (kind, correlation) = match &apply.op {
            Op::Apply { kind, correlation } => (kind.clone(), correlation.clone()),
            _ => unreachable!(),
        };
        let (probe, icol) = match kind {
            ApplyKind::NotIn { probe, inner } => (probe, inner),
            _ => unreachable!(),
        };
        let want = not_in_oracle(
            &orders(),
            &effective_inner(&case.nested, &src),
            &correlation,
            probe,
            icol,
        );

        let (nested, _) = run(&case.nested, "out", &src);
        assert_eq!(
            nested, want,
            "the *nested* form of `{}` is not SQL's `not in`",
            case.name
        );

        let (flat, _) = unnest(&case.nested);
        let (unnested, _) = run(&flat, "out", &src);
        assert_eq!(
            unnested, want,
            "the *unnested* form of `{}` is not SQL's `not in`",
            case.name
        );
        checked += 1;
    }
    assert_eq!(
        checked, 8,
        "eight `not in` cases, checked against the oracle"
    );
}

#[test]
fn one_null_in_an_uncorrelated_subquery_returns_no_rows() {
    // The rule stated as its own test, because it is the one the roadmap makes a kill
    // criterion and the one a reader will want to see named. `payments` has a null amount;
    // uncorrelated, that makes every non-matching probe unknown, so nothing survives.
    let src = data();
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "not in, uncorrelated, one null poisons everything")
        .unwrap();
    let (flat, _) = unnest(&case.nested);
    let (out, _) = run(&flat, "out", &src);
    assert!(
        out.is_empty(),
        "a single null in the subquery must return no rows, got {out:?}"
    );

    // And the contrast, so the test is about the null rather than about the data: the same
    // query against a relation with no nulls returns rows.
    let no_nulls = corpus()
        .into_iter()
        .find(|c| c.name == "not in, uncorrelated, no nulls")
        .unwrap();
    let (flat2, _) = unnest(&no_nulls.nested);
    let (out2, _) = run(&flat2, "out", &src);
    assert!(
        !out2.is_empty(),
        "without nulls the same query must return rows"
    );
}

#[test]
fn a_null_probe_is_dropped_even_against_an_empty_subquery() {
    // The other half of the three-valued rule, and the one a "not in = anti-join"
    // implementation gets wrong in the opposite direction: an anti-join would *keep* a
    // null probe, because it matched nothing.
    let src = data();
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "not in, empty inner is vacuously true")
        .unwrap();
    let (flat, _) = unnest(&case.nested);
    let (out, _) = run(&flat, "out", &src);
    assert!(
        !out.keys().any(|r| r[2].is_null()),
        "order 4 has a null amount and must not survive `not in`, got {out:?}"
    );
    // Everything else does survive, since the subquery is empty.
    assert_eq!(out.len(), orders().len() - 1);
}

#[test]
fn a_plain_anti_join_would_have_been_wrong_which_is_why_the_witness_exists() {
    // Guarding the guard. If the null witness were dropped, `not in` would become an
    // anti-join — and this test asserts that the two really do differ on this dataset, so
    // the witness is load-bearing rather than decorative.
    let src = data();
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "not in, uncorrelated, one null poisons everything")
        .unwrap();
    let (correct, _) = run(&unnest(&case.nested).0, "out", &src);

    // The naive rewrite: probe-not-null, then one anti-join, and no witness.
    let mut naive = Circuit::new();
    let o = source(&mut naive, "orders", 3);
    let p = source(&mut naive, "payments", 2);
    let f = naive.add(
        Op::Filter {
            predicate: Scalar::Not(Box::new(Scalar::IsNull(Box::new(Scalar::Column(2))))),
        },
        vec![o],
        internal_contract(),
        "probe not null",
    );
    let a = naive.add(
        Op::Join {
            kind: niles_ir::operator::JoinKind::Anti,
            left_key: vec![2],
            right_key: vec![1],
            residual: None,
        },
        vec![f, p],
        served(),
        "naive anti",
    );
    naive.set_output("out", a);
    let (wrong, _) = run(&naive, "out", &src);

    assert_ne!(
        correct, wrong,
        "the null witness made no difference on this dataset, so the corpus does not test it"
    );
    assert!(correct.is_empty());
    assert!(
        !wrong.is_empty(),
        "the naive rewrite returns rows SQL says do not exist"
    );
}

// ── refusals ─────────────────────────────────────────────────────────────────────────

#[test]
fn unnesting_through_a_limit_is_refused_with_a_reason() {
    // "The three cheapest payments *for this customer*" is not "three rows of the join".
    // A rewrite that did it anyway would change the answer, so it refuses — and names why,
    // because a silent refusal surfaces much later as IR018 pointing at the wrong node.
    let mut c = Circuit::new();
    let o = source(&mut c, "orders", 3);
    let p = source(&mut c, "payments", 2);
    let lim = c.add(
        Op::Limit {
            count: 3,
            offset: 0,
        },
        vec![p],
        internal_contract(),
        "top 3",
    );
    let a = c.add(
        Op::Apply {
            kind: ApplyKind::Exists,
            correlation: vec![(1, 0)],
        },
        vec![o, lim],
        served(),
        "exists",
    );
    c.set_output("out", a);

    let (_, report) = unnest(&c);
    assert!(!report.is_complete());
    assert_eq!(report.refused.len(), 1);
    assert!(
        report.refused[0].reason.contains("limit"),
        "{}",
        report.refused[0].reason
    );
}

#[test]
fn unnesting_through_an_uncertified_udf_is_refused() {
    let mut c = Circuit::new();
    let o = source(&mut c, "orders", 3);
    let p = source(&mut c, "payments", 2);
    let f = c.add(
        Op::Filter {
            predicate: Scalar::Udf {
                id: 7,
                args: vec![Scalar::Column(0)],
            },
        },
        vec![p],
        internal_contract(),
        "udf filter",
    );
    let a = c.add(
        Op::Apply {
            kind: ApplyKind::Exists,
            correlation: vec![(1, 0)],
        },
        vec![o, f],
        served(),
        "exists",
    );
    c.set_output("out", a);
    let (_, report) = unnest(&c);
    assert!(
        report.refused[0].reason.contains("certified"),
        "{}",
        report.refused[0].reason
    );

    // Certify it and the same rewrite proceeds — the refusal is about determinism, not
    // about UDFs.
    let mut c2 = c.clone();
    c2.certified_udfs = vec![7];
    let (_, report2) = unnest(&c2);
    assert!(report2.is_complete(), "{}", report2.render());
}

// ── counted work: the E17 measurement ────────────────────────────────────────────────

/// The nested and unnested work for one case, in row-operations.
pub fn work_of(case: &Case, src: &BTreeMap<String, ZSet>) -> (u64, u64) {
    let (_, nested) = run(&case.nested, "out", src);
    let (flat, _) = unnest(&case.nested);
    let (_, unnested) = run(&flat, "out", src);
    (nested, unnested)
}

/// Total nested and unnested work over the whole corpus at scale `k`.
fn totals_at(k: usize) -> (u64, u64) {
    let src = data_at(k);
    let (mut n, mut u) = (0, 0);
    for case in corpus() {
        let (a, b) = work_of(&case, &src);
        n += a;
        u += b;
    }
    (n, u)
}

/// The same, restricted to the regime where an asymptotic win exists.
///
/// The whole-corpus total grows *sub*-linearly, and the reason is worth stating rather
/// than smoothing: the uncorrelated cases stay quadratic in both plans, so as `k` rises
/// they come to dominate a sum that mixes the regimes. A ratio taken over a mixture of
/// asymptotics is a number without a meaning. This one is taken over the correlated cases
/// alone, where the claim — a quadratic became a linear — is the claim being tested.
fn correlated_totals_at(k: usize) -> (u64, u64) {
    let src = data_at(k);
    let (mut n, mut u) = (0, 0);
    for case in corpus() {
        if regime(&case, &src) != Regime::Correlated {
            continue;
        }
        let (a, b) = work_of(&case, &src);
        n += a;
        u += b;
    }
    (n, u)
}

/// Which regime a case is in, computed rather than declared.
///
/// The asymptotic win is `|outer| x |inner|` becoming `|outer| + |inner|`, so it exists
/// only when both factors grow. Two kinds of case have no win to have, and saying so is
/// more useful than quietly excluding them by name:
///
/// * **`EmptyInner`** — the subquery evaluates to nothing, so the nested loop's inner
///   iteration never runs. There is no quadratic to remove. These cases are in the corpus
///   for their *answers* (an empty `not in` is vacuously true; an empty scalar subquery is
///   null), not for their cost.
/// * **`Uncorrelated`** — with no correlation every outer row matches every inner row, so
///   the join is a cross product and both plans are quadratic.
#[derive(Debug, PartialEq, Eq)]
enum Regime {
    Correlated,
    Uncorrelated,
    EmptyInner,
}

fn regime(case: &Case, src: &BTreeMap<String, ZSet>) -> Regime {
    if effective_inner(&case.nested, src).is_empty() {
        return Regime::EmptyInner;
    }
    let apply = case
        .nested
        .nodes
        .iter()
        .find(|n| matches!(n.op, Op::Apply { .. }))
        .unwrap();
    match &apply.op {
        Op::Apply { correlation, .. } if correlation.is_empty() => Regime::Uncorrelated,
        _ => Regime::Correlated,
    }
}

#[test]
fn the_three_regimes_are_all_represented_and_none_is_empty() {
    // Guarding the exclusion. If every case that failed to speed up were quietly reclassified,
    // the gate below would prove nothing, so the partition is asserted here.
    let src = data_at(8);
    let mut n = (0, 0, 0);
    for case in corpus() {
        match regime(&case, &src) {
            Regime::Correlated => n.0 += 1,
            Regime::Uncorrelated => n.1 += 1,
            Regime::EmptyInner => n.2 += 1,
        }
    }
    assert!(
        n.0 >= 12,
        "most of the corpus must be in the regime the gate measures, got {}",
        n.0
    );
    assert!(
        n.1 >= 3 && n.2 >= 5,
        "the two exempt regimes must be represented: {n:?}"
    );
    assert_eq!(n.0 + n.1 + n.2, 24);
}

#[test]
fn unnesting_is_asymptotically_cheaper_and_the_crossover_is_measured() {
    // The gate, stated as what is actually true rather than as what would be tidier.
    //
    // A dependent join costs |outer| x |inner|; the unnested form is a constant number of
    // linear passes. So the ratio grows with scale, and *below* some scale the constant
    // loses — which is a fact about the rewrite worth reporting rather than a failure to
    // engineer around. Three claims, each checked:
    //
    //   1. the ratio is monotone in scale,
    //   2. by k = 8 every correlated case is cheaper unnested,
    //   3. the ratio grows at least linearly, which is the signature of removing a
    //      quadratic rather than shaving a constant.
    let ratio = |k: usize| {
        let (n, u) = correlated_totals_at(k);
        n as f64 / u as f64
    };
    let (r1, r4, r16, r64) = (ratio(1), ratio(4), ratio(16), ratio(64));
    assert!(
        r1 < r4 && r4 < r16 && r16 < r64,
        "the ratio must grow with scale: {r1:.2} {r4:.2} {r16:.2} {r64:.2}"
    );
    // k quadruples at each step, so a linear-in-k ratio quadruples too. Asserting a
    // trebling leaves room for the constant terms while still excluding a merely-constant
    // improvement, which is the thing being distinguished.
    assert!(
        r16 > 3.0 * r4,
        "growth should be linear in k, not constant: {r4:.2} → {r16:.2}"
    );
    assert!(
        r64 > 3.0 * r16,
        "and it must keep growing: {r16:.2} → {r64:.2}"
    );

    // Every case in the measured regime, individually, at k = 8.
    let src = data_at(8);
    for case in corpus() {
        if regime(&case, &src) != Regime::Correlated {
            continue;
        }
        let (nested, unnested) = work_of(&case, &src);
        assert!(
            unnested < nested,
            "`{}` is not cheaper unnested at k=8: {nested} → {unnested}",
            case.name
        );
    }
}

#[test]
fn the_crossover_scale_is_where_the_measurement_says_it_is() {
    // The scale at which each case turns from a loss into a win. Pinned so that a future
    // change to the rewrite — an extra node, a fused pair — shows up here as a moved
    // number rather than as a quietly worse plan.
    let src8 = data_at(8);
    let mut worst = 1usize;
    for case in corpus() {
        if regime(&case, &src8) != Regime::Correlated {
            continue;
        }
        let mut k = 1;
        while k <= 64 {
            let (n, u) = work_of(&case, &data_at(k));
            if u < n {
                break;
            }
            k *= 2;
        }
        assert!(k <= 64, "`{}` never becomes cheaper by k=64", case.name);
        worst = worst.max(k);
    }
    assert!(
        worst <= 8,
        "the last case to cross over does so at k={worst}; if that has grown, the rewrite \
         gained a node and the small-input regime got worse"
    );
}

#[test]
#[ignore = "measurement; writes results/E17-unnesting-table.md"]
fn e17_write_the_measurement() {
    let scales = [1usize, 4, 16, 64];
    let mut s = String::new();

    s.push_str("### Per case, at k = 16\n\n");
    s.push_str("| # | case | regime | rewrite | nested | unnested | ratio |\n|---|---|---|---|---|---|---|\n");
    let src = data_at(16);
    let (mut tn, mut tu) = (0u64, 0u64);
    for (i, case) in corpus().into_iter().enumerate() {
        let (_, report) = unnest(&case.nested);
        let (nested, unnested) = work_of(&case, &src);
        let reg = match regime(&case, &src) {
            Regime::Correlated => "correlated",
            Regime::Uncorrelated => "uncorrelated",
            Regime::EmptyInner => "empty inner",
        };
        tn += nested;
        tu += unnested;
        s.push_str(&format!(
            "| {} | {} | {reg} | `{}` | {nested} | {unnested} | {:.2}x |\n",
            i + 1,
            case.name,
            report.fired.first().map(|r| r.name()).unwrap_or("—"),
            nested as f64 / unnested.max(1) as f64
        ));
    }
    s.push_str(&format!(
        "\n**Corpus total at k = 16:** {tn} → {tu} row-operations, {:.2}x.\n",
        tn as f64 / tu.max(1) as f64
    ));

    s.push_str("\n### The curve\n\n");
    s.push_str(
        "Whole corpus, and the correlated regime alone. The corpus total grows \
sub-linearly because the uncorrelated cases stay quadratic in both plans and come to \
dominate a sum that mixes regimes; the correlated column is the one the claim is about.\n\n",
    );
    s.push_str("| k | outer rows | inner rows | corpus nested | corpus unnested | corpus ratio | correlated ratio |\n|---|---|---|---|---|---|---|\n");
    for k in scales {
        let (n, u) = totals_at(k);
        let (cn, cu) = correlated_totals_at(k);
        s.push_str(&format!(
            "| {k} | {} | {} | {n} | {u} | {:.2}x | **{:.2}x** |\n",
            orders_at(k).len(),
            payments_at(k).len(),
            n as f64 / u.max(1) as f64,
            cn as f64 / cu.max(1) as f64,
        ));
    }

    s.push_str("\n### Crossover, per case\n\n");
    s.push_str("The smallest power-of-two scale at which the unnested plan does less work.\n\n");
    s.push_str("| case | crossover k |\n|---|---|\n");
    let src8 = data_at(8);
    for case in corpus() {
        if regime(&case, &src8) != Regime::Correlated {
            continue;
        }
        let mut k = 1;
        while k <= 64 {
            let (n, u) = work_of(&case, &data_at(k));
            if u < n {
                break;
            }
            k *= 2;
        }
        s.push_str(&format!(
            "| {} | {} |\n",
            case.name,
            if k <= 64 { k.to_string() } else { ">64".into() }
        ));
    }

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../results/E17-unnesting-table.md");
    std::fs::write(&path, s).expect("write the table");
    eprintln!("wrote {}", path.display());
}

#[test]
fn every_rewrite_in_the_enum_is_exercised_by_the_corpus() {
    // A corpus that never fires a rewrite is a corpus that does not test it.
    let mut seen: Vec<&'static str> = corpus()
        .iter()
        .map(|c| {
            let (_, r) = unnest(&c.nested);
            r.fired[0].name()
        })
        .collect();
    seen.sort_unstable();
    seen.dedup();
    let all = [
        "exists-to-semi-join",
        "in-to-semi-join",
        "not-exists-to-anti-join",
        "not-in-to-anti-join-with-null-witness",
        "scalar-to-outer-join-with-aggregate",
    ];
    assert_eq!(seen, all, "every rewrite must fire somewhere in the corpus");
    // And the enum has no member the corpus is unaware of.
    let _exhaustive = |r: Rewrite| match r {
        Rewrite::ExistsToSemiJoin { .. }
        | Rewrite::NotExistsToAntiJoin { .. }
        | Rewrite::InToSemiJoin { .. }
        | Rewrite::NotInToAntiJoinWithNullWitness { .. }
        | Rewrite::ScalarToOuterJoinWithAggregate { .. } => (),
    };
}

#[test]
fn a_semi_join_that_widened_is_rejected_as_duplicate_inflation() {
    // IR017, stated as its own test. The bug it catches is an inner join wearing a
    // semi-join's label; the checkable symptom is the output width.
    let mut c = Circuit::new();
    let o = source(&mut c, "orders", 3);
    let p = source(&mut c, "payments", 2);
    let j = c.add(
        Op::Join {
            kind: niles_ir::operator::JoinKind::Semi,
            left_key: vec![1],
            right_key: vec![0],
            residual: None,
        },
        vec![o, p],
        served(),
        "semi",
    );
    c.set_output("out", j);
    assert!(
        verify::verify(&c)
            .violations
            .iter()
            .all(|v| v.code != "IR017"),
        "a correct semi-join passes"
    );

    // Now widen it by hand, as a mis-implementation would.
    c.nodes[j as usize].arity = 5;
    let r = verify::verify(&c);
    assert!(
        r.violations.iter().any(|v| v.code == "IR017"),
        "a widened semi-join must be rejected:\n{}",
        r.render()
    );
}

#[test]
fn the_evaluator_and_the_verifier_agree_that_a_semi_join_does_not_inflate() {
    // Belt and braces: the verifier checks the declared width, the evaluator checks the
    // actual weights. Customer 10 has three payment rows, one of them a duplicate; the
    // `exists` over it must yield each order once.
    let src = data();
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "exists, correlated")
        .unwrap();
    let (flat, _) = unnest(&case.nested);
    let (out, _) = run(&flat, "out", &src);
    let for_ten: i128 = out
        .iter()
        .filter(|(r, _)| r[1] == Value::Int(10))
        .map(|(_, w)| *w)
        .sum();
    assert_eq!(
        for_ten, 2,
        "two orders for customer 10, each once — not once per payment"
    );
}
