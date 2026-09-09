//! **The declared fragment, case by case, against an oracle that does not use the runtime.**
//!
//! `Runtime::install` admitted any circuit whose outputs were keyed `Sum` or `Count`
//! aggregates. It kept no executable plan for them, and `advance` handed the same `Base` to
//! every view — and a `Base` is a pre-aggregated oracle that is given a key and an epoch and
//! is told nothing about the graph. So this installed and answered:
//!
//! ```text
//! input rows      [10, 20]
//! sum(amount)  -> 30     correct
//! count(amount)-> 30     the sum, wearing count's name
//! ```
//!
//! No answer-level test could catch it, because 30 is exactly what the *other* output should
//! give. Chapter 4's claim that an installed circuit computes "exactly Q_lin" was contradicted
//! by the public API, and the contradiction was reachable in four lines.
//!
//! # What this file asserts
//!
//! Every row of the declared matrix, each labelled `correct` or `refused: <reason>`:
//!
//! * a supported case is executed through the runtime in **every state a REV entry can be
//!   in** — cold, hit, evicted, reconstructed after eviction, and after `advance` — and every
//!   answer is compared against a fold over the raw rows computed here, with no runtime, no
//!   circuit and no `Base` involved;
//! * an unsupported case is refused **at installation**, with the structured reason named in
//!   the table rather than "some error";
//! * a base whose plan is not the view's is rejected rather than served, including the case
//!   that looks most innocent: the same relation name with a different column numbering.
//!
//! The oracle is deliberately stupid — a loop over a `Vec` — because a clever oracle is a
//! second implementation of the thing under test, and the two would share a mistake.

use niles_ir::circuit::Circuit;
use niles_ir::operator::{Agg, JoinKind, Op, Scalar, ScalarOp};
use niles_ir::{Consistency, Lineage, Materialize, Retention, ServeContract};
use nilestream_core::absence::Epoch;
use nilestream_core::rev::{Base, BasePlan, Key, Policy, Runtime, Unsupported, Value};

// ── the fixture: raw rows, and two things that read them ───────────────────────────────────

/// One base row: the epoch it was sealed in, and its columns.
type Raw = (Epoch, Vec<i128>);

/// `(epoch, [key0, amount])` — the shape every circuit below aggregates.
///
/// Negative amounts and repeated keys are here on purpose: a delete in a Z-set is a negative
/// weight, and this base's `deltas_at` reports the signed sum for an epoch, so a key that is
/// credited and debited in the same epoch must net rather than appear twice.
fn fixture() -> Vec<Raw> {
    vec![
        (1, vec![1, 10]),
        (1, vec![2, 20]),
        (2, vec![1, 5]),
        (2, vec![1, -5]), // an insert and its delete in one epoch: nets to zero
        (3, vec![2, -30]),
        (4, vec![3, 0]), // a key whose only row is zero: present, and not absent
        (5, vec![1, -100]),
    ]
}

/// **The oracle.** A fold over the raw rows, with no runtime, no circuit and no `Base`.
fn oracle(rows: &[Raw], plan: &BasePlan, key: &Key, anchor: Epoch) -> Value {
    let input_col = match plan.input {
        Scalar::Column(c) => c as usize,
        ref other => panic!("the fixture only builds column inputs, not {other:?}"),
    };
    let mut acc: Value = 0;
    for (e, cols) in rows {
        if *e > anchor {
            continue;
        }
        let row_key: Key = plan
            .group_key
            .iter()
            .map(|c| cols[*c as usize] as i64)
            .collect();
        if row_key != *key {
            continue;
        }
        match plan.agg {
            Agg::Sum => acc += cols[input_col],
            Agg::Count => acc += 1,
            other => panic!("the fixture does not model {other:?}"),
        }
    }
    acc
}

/// A base over the raw rows that answers exactly the plan it is given.
struct RawBase {
    rows: Vec<Raw>,
    plan: BasePlan,
    head: Epoch,
}

impl RawBase {
    fn new(plan: BasePlan) -> RawBase {
        RawBase {
            rows: fixture(),
            head: 5,
            plan,
        }
    }
}

impl Base for RawBase {
    fn answers(&self) -> BasePlan {
        self.plan.clone()
    }
    fn frontier(&self) -> Epoch {
        self.head
    }
    fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
        // The row count is the rows it *read*, which is every row at or below the anchor —
        // not the rows that matched, which is what a reader would want to believe.
        let read = self.rows.iter().filter(|(e, _)| *e <= anchor).count() as u64;
        (oracle(&self.rows, &self.plan, key, anchor), read)
    }
    fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
        let mut per: std::collections::BTreeMap<Key, Value> = Default::default();
        for (at, cols) in &self.rows {
            if *at != e {
                continue;
            }
            let k: Key = self
                .plan
                .group_key
                .iter()
                .map(|c| cols[*c as usize] as i64)
                .collect();
            let v = match self.plan.agg {
                Agg::Sum => match self.plan.input {
                    Scalar::Column(c) => cols[c as usize],
                    _ => unreachable!(),
                },
                Agg::Count => 1,
                other => panic!("{other:?}"),
            };
            *per.entry(k).or_insert(0) += v;
        }
        per.into_iter().collect()
    }
}

// ── circuit builders ───────────────────────────────────────────────────────────────────────

fn contract() -> ServeContract {
    ServeContract {
        consistency: Consistency::LedgerConsistent,
        materialize: Materialize::Demand,
        retain: Retention::Evictable,
        lineage: Lineage::Key,
    }
}

fn source(c: &mut Circuit, relation: &str) -> niles_ir::circuit::NodeId {
    c.add(
        Op::Source {
            relation: relation.into(),
            is_base: true,
            anchor_key: vec![0],
            confidential: Vec::new(),
        },
        vec![],
        contract(),
        relation,
    )
}

fn agg(
    c: &mut Circuit,
    input: niles_ir::circuit::NodeId,
    name: &str,
    a: Agg,
    col: u16,
    key: Vec<u16>,
) -> niles_ir::circuit::NodeId {
    let id = c.add(
        Op::Aggregate {
            group_key: key,
            aggs: vec![(a, Scalar::Column(col))],
        },
        vec![input],
        contract(),
        name,
    );
    c.set_output(name, id);
    id
}

/// One output: `agg(col) over postings group by [0]`.
fn one(a: Agg, col: u16) -> Circuit {
    let mut c = Circuit::new();
    let s = source(&mut c, "postings");
    agg(&mut c, s, "balance", a, col, vec![0]);
    c
}

// ── the supported half: every state, against the oracle ────────────────────────────────────

/// Drive one supported plan through cold, hit, evicted, reconstructed and advanced, comparing
/// every answer with the oracle.
fn check_supported(plan: BasePlan, circuit: Circuit) {
    let base = RawBase::new(plan.clone());
    let rows = fixture();
    // A budget of one against three keys, so every second read evicts the previous one and
    // the third state in the list below is reached by construction rather than by luck.
    let mut rt = Runtime::install(circuit, Some(1), Policy::Lru).expect("a supported fragment");
    let view = rt.view_mut("balance").expect("the installed view");

    let keys: Vec<Key> = vec![vec![1], vec![2], vec![3], vec![99]];

    // cold: every key, first touch, at several anchors.
    for anchor in [0u64, 1, 2, 3, 4, 5] {
        for k in &keys {
            let got = view.read(&base, k, anchor);
            assert_eq!(
                got.value,
                oracle(&rows, &plan, k, anchor),
                "{plan} at key {k:?} anchor {anchor}: cold read"
            );
            assert_eq!(
                got.anchor, anchor,
                "the answer is stamped at what was asked"
            );
        }
    }
    // hit: the same question again. The budget is one, so most of these are misses that
    // reconstruct — which is the point: the answer may not depend on whether it was resident.
    for k in &keys {
        let a = view.read(&base, k, 5);
        let b = view.read(&base, k, 5);
        assert_eq!(a.value, b.value, "{plan}: a repeat answered differently");
        assert_eq!(a.value, oracle(&rows, &plan, k, 5));
    }
    // advanced: maintenance applies the deltas, and the answers must not move.
    for e in 1..=5u64 {
        rt.advance(&base, e);
    }
    let view = rt.view_mut("balance").expect("the installed view");
    for k in &keys {
        assert_eq!(
            view.read(&base, k, 5).value,
            oracle(&rows, &plan, k, 5),
            "{plan} at key {k:?}: after advance"
        );
    }
    // A key nothing ever touched answers the same as the oracle does — which for a `sum` is
    // zero and for a `count` is zero, and in neither case is it a row that exists.
    assert_eq!(
        view.read(&base, &vec![99], 5).value,
        oracle(&rows, &plan, &vec![99], 5)
    );
}

#[test]
fn a_keyed_sum_matches_the_raw_row_oracle_in_every_state() {
    check_supported(BasePlan::sum("postings", 1, vec![0]), one(Agg::Sum, 1));
}

#[test]
fn a_keyed_count_matches_the_raw_row_oracle_in_every_state() {
    // The case the defect made unobservable: with a `count` base this is a `count`, and the
    // answers differ from the sum's on every key. Before the repair the runtime could not
    // tell the two apart, so a `count` view over a `sum` base answered the sum.
    let plan = BasePlan {
        relation: "postings".into(),
        agg: Agg::Count,
        input: Scalar::Column(1),
        group_key: vec![0],
    };
    check_supported(plan, one(Agg::Count, 1));
}

#[test]
fn the_sum_and_the_count_of_the_same_rows_are_different_numbers() {
    // The assertion that makes the two tests above worth having. If these ever coincide the
    // fixture has stopped being able to see the defect.
    let rows = fixture();
    let sum = BasePlan::sum("postings", 1, vec![0]);
    let count = BasePlan {
        relation: "postings".into(),
        agg: Agg::Count,
        input: Scalar::Column(1),
        group_key: vec![0],
    };
    for k in [vec![1i64], vec![2], vec![3]] {
        assert_ne!(
            oracle(&rows, &sum, &k, 5),
            oracle(&rows, &count, &k, 5),
            "key {k:?}"
        );
    }
}

// ── the refused half: every shape, with the reason it is refused for ───────────────────────

/// `(name, build, the reason installation must give)`.
type Case = (&'static str, fn() -> Circuit, fn(&Unsupported) -> bool);

fn refused_cases() -> Vec<Case> {
    vec![
        (
            "two outputs, a sum and a count",
            || {
                let mut c = Circuit::new();
                let s = source(&mut c, "postings");
                agg(&mut c, s, "a_sum", Agg::Sum, 1, vec![0]);
                agg(&mut c, s, "b_count", Agg::Count, 1, vec![0]);
                c
            },
            |u| matches!(u, Unsupported::MixedPlans { .. }),
        ),
        (
            "two outputs summing different columns",
            || {
                let mut c = Circuit::new();
                let s = source(&mut c, "postings");
                agg(&mut c, s, "a", Agg::Sum, 1, vec![0]);
                agg(&mut c, s, "b", Agg::Sum, 2, vec![0]);
                c
            },
            |u| matches!(u, Unsupported::MixedPlans { .. }),
        ),
        (
            "two outputs with different group keys",
            || {
                let mut c = Circuit::new();
                let s = source(&mut c, "postings");
                agg(&mut c, s, "a", Agg::Sum, 1, vec![0]);
                agg(&mut c, s, "b", Agg::Sum, 1, vec![0, 1]);
                c
            },
            |u| matches!(u, Unsupported::MixedPlans { .. }),
        ),
        (
            "one output with two aggregates",
            || {
                let mut c = Circuit::new();
                let s = source(&mut c, "postings");
                let id = c.add(
                    Op::Aggregate {
                        group_key: vec![0],
                        aggs: vec![
                            (Agg::Sum, Scalar::Column(1)),
                            (Agg::Count, Scalar::Column(1)),
                        ],
                    },
                    vec![s],
                    contract(),
                    "both",
                );
                c.set_output("both", id);
                c
            },
            |u| matches!(u, Unsupported::MultipleAggregates { .. }),
        ),
        (
            "an aggregate this runtime does not maintain",
            || one(Agg::Min, 1),
            |u| matches!(u, Unsupported::Aggregate { .. }),
        ),
        (
            "a filter between the source and the aggregate",
            || {
                let mut c = Circuit::new();
                let s = source(&mut c, "postings");
                let f = c.add(
                    Op::Filter {
                        predicate: Scalar::Binary {
                            op: ScalarOp::Gt,
                            lhs: Box::new(Scalar::Column(1)),
                            rhs: Box::new(Scalar::LitInt(0)),
                        },
                    },
                    vec![s],
                    contract(),
                    "kept",
                );
                agg(&mut c, f, "balance", Agg::Sum, 1, vec![0]);
                c
            },
            |u| matches!(u, Unsupported::Upstream { .. }),
        ),
        (
            "two sources joined under the aggregate",
            || {
                let mut c = Circuit::new();
                let l = source(&mut c, "postings");
                let r = source(&mut c, "other");
                let j = c.add(
                    Op::Join {
                        kind: JoinKind::Inner,
                        left_key: vec![0],
                        right_key: vec![0],
                        residual: None,
                    },
                    vec![l, r],
                    contract(),
                    "joined",
                );
                agg(&mut c, j, "balance", Agg::Sum, 1, vec![0]);
                c
            },
            |u| matches!(u, Unsupported::Upstream { .. }),
        ),
    ]
}

#[test]
fn every_unsupported_shape_is_refused_at_installation_with_its_own_reason() {
    let mut table = Vec::new();
    for (name, build, want) in refused_cases() {
        match Runtime::install(build(), Some(4), Policy::Lru) {
            Ok(_) => panic!(
                "`{name}` installed. Every output of an installed runtime is served by one \
                 `Base`, which answers one question, so this runtime would answer some of its \
                 outputs with another output's number."
            ),
            Err(u) => {
                assert!(
                    want(&u),
                    "`{name}` was refused for the wrong reason: {u:?} — {}",
                    u.explain()
                );
                table.push(format!("{name}: refused: {}", u.explain()));
            }
        }
    }
    assert_eq!(table.len(), 7);
}

// ── the base contract: a mismatched oracle is rejected, not served ─────────────────────────

fn mismatch_panics(view_plan: BasePlan, base_plan: BasePlan, circuit: Circuit) -> String {
    let base = RawBase::new(base_plan);
    let mut rt = Runtime::install(circuit, Some(4), Policy::Lru).expect("installs");
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let view = rt.view_mut("balance").expect("the view");
        view.read(&base, &vec![1], 5)
    }));
    match r {
        Ok(a) => panic!(
            "a view installed for {view_plan} accepted a base that answers something else and \
             returned {}. That number is well formed and wrong, which is the whole defect.",
            a.value
        ),
        Err(e) => e
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    }
}

#[test]
fn a_base_that_answers_another_question_is_rejected() {
    let hush = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    // A `sum` view handed a `count` oracle: the shape the probe found, one layer out.
    let msg = mismatch_panics(
        BasePlan::sum("postings", 1, vec![0]),
        BasePlan {
            relation: "postings".into(),
            agg: Agg::Count,
            input: Scalar::Column(1),
            group_key: vec![0],
        },
        one(Agg::Sum, 1),
    );
    assert!(msg.contains("was installed for"), "{msg}");

    // **The same relation with a different column numbering.** This is the innocent-looking
    // one: both plans say `sum over postings`, and they are questions about different
    // columns. A check on the relation name alone would pass it.
    let msg = mismatch_panics(
        BasePlan::sum("postings", 1, vec![0]),
        BasePlan::sum("postings", 2, vec![0]),
        one(Agg::Sum, 1),
    );
    assert!(msg.contains("was installed for"), "{msg}");

    // And a permuted key, which reconstructs positionally: `(acct, cur)` and `(cur, acct)`
    // have the same components and are not the same question.
    let msg = mismatch_panics(
        BasePlan::sum("postings", 2, vec![0, 1]),
        BasePlan::sum("postings", 2, vec![1, 0]),
        {
            let mut c = Circuit::new();
            let s = source(&mut c, "postings");
            agg(&mut c, s, "balance", Agg::Sum, 2, vec![0, 1]);
            c
        },
    );
    assert!(msg.contains("was installed for"), "{msg}");

    std::panic::set_hook(hush);
}

#[test]
fn a_base_that_answers_the_view_is_accepted() {
    // The control. A check that rejected every base would pass the test above.
    let base = RawBase::new(BasePlan::sum("postings", 1, vec![0]));
    let mut rt = Runtime::install(one(Agg::Sum, 1), Some(4), Policy::Lru).expect("installs");
    let view = rt.view_mut("balance").expect("the view");
    assert_eq!(
        view.read(&base, &vec![1], 5).value,
        oracle(
            &fixture(),
            &BasePlan::sum("postings", 1, vec![0]),
            &vec![1],
            5
        )
    );
}

/// **Astra's probe, kept as a guard rather than as a paragraph.**
///
/// The original is four lines: install a circuit with a `sum(amount)` output and a
/// `count(amount)` output, seed the rows `[10, 20]`, read both. It answered
/// `sum = 30, count = 30`.
///
/// This test is written to hold in all three worlds, which is what makes it a guard rather
/// than a restatement of the repair:
///
/// * **as repaired**, installation refuses with `MixedPlans` and the assertion is on that
///   exact reason;
/// * **with the refusal removed** but the base check kept, the `count` view is handed a `sum`
///   oracle and `require_base` rejects it — also a pass, and the message says which check
///   caught it;
/// * **with descriptor propagation reverted entirely** — no plan on the view, no refusal at
///   installation, no base check — the reads reach the oracle comparison below and it fails
///   with `count` answering 30 where 2 is the answer. That is the witness the audit reported,
///   reproduced from this file, at a runtime assertion and not at a compile error.
#[test]
fn the_probe_cannot_answer_thirty_for_a_count_of_two_rows() {
    let two_rows: Vec<Raw> = vec![(1, vec![7, 10]), (1, vec![7, 20])];
    let sum_plan = BasePlan::sum("postings", 1, vec![0]);
    let count_plan = BasePlan {
        relation: "postings".into(),
        agg: Agg::Count,
        input: Scalar::Column(1),
        group_key: vec![0],
    };
    let key: Key = vec![7];
    assert_eq!(oracle(&two_rows, &sum_plan, &key, 1), 30, "the probe's sum");
    assert_eq!(
        oracle(&two_rows, &count_plan, &key, 1),
        2,
        "the probe's count"
    );

    let mut c = Circuit::new();
    let s = source(&mut c, "postings");
    agg(&mut c, s, "a_sum", Agg::Sum, 1, vec![0]);
    agg(&mut c, s, "b_count", Agg::Count, 1, vec![0]);

    let mut rt = match Runtime::install(c, Some(4), Policy::Lru) {
        Err(u) => {
            assert!(
                matches!(u, Unsupported::MixedPlans { .. }),
                "refused, but for the wrong reason: {u:?}"
            );
            return;
        }
        Ok(rt) => rt,
    };

    // Reached only when the installation refusal has been removed. The base is a `sum`
    // oracle, because that is what the probe had: one runtime, one base.
    let base = RawBase {
        rows: two_rows.clone(),
        plan: sum_plan.clone(),
        head: 1,
    };
    let hush = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let answers = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let sum = rt
            .view_mut("a_sum")
            .expect("the sum view")
            .read(&base, &key, 1);
        let count = rt
            .view_mut("b_count")
            .expect("the count view")
            .read(&base, &key, 1);
        (sum.value, count.value)
    }));
    std::panic::set_hook(hush);

    let (sum, count) = match answers {
        // The base check caught it: the `count` view was handed a `sum` oracle.
        Err(_) => return,
        Ok(v) => v,
    };
    assert_eq!(sum, 30, "the sum output");
    assert_eq!(
        count, 2,
        "**this is the defect**: the `count` output answered the sum. A runtime holds one \
         `Base`, a `Base` answers one question, and nothing bound the installed circuit to \
         what the base could execute."
    );
}
