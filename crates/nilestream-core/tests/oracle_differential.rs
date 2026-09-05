//! The REV runtime against the reference oracle.
//!
//! `proto-engine` is the research prototype; this is the runtime that executes a compiled
//! circuit. Both are checked against the same fold in `conservation-suite`, which neither
//! can reach — so a defect in one cannot be hidden by the same defect in the other, which
//! is precisely what a shared reconstruction routine would have allowed.

use conservation_suite::faults::{inject, Fault};
use conservation_suite::oracle::{Acct as OAcct, Cur as OCur};
use conservation_suite::properties::{differential, schedule, Answer, Observable, Op, Unavailable};
use niles_ir::circuit::Circuit;
use niles_ir::operator::{Agg, Op as IrOp};
use niles_ir::{Consistency, Materialize, Retention, ServeContract};
use nilestream_core::absence::Epoch;
use nilestream_core::rev::{Base, Key, Policy, Runtime, Value};
use std::collections::BTreeMap;

/// A base the runtime can read: an append-only history, folded on demand.
///
/// Deliberately as dumb as the oracle. Its job is to be a *base*, not to be fast; the
/// thing under test is the partial view above it.
#[derive(Default)]
struct History {
    /// (epoch, key, delta)
    rows: Vec<(Epoch, Key, Value)>,
    head: Epoch,
    started: bool,
}

impl History {
    fn seal(&mut self, deltas: Vec<(Key, Value)>) {
        if self.started {
            self.head += 1;
        } else {
            self.started = true;
        }
        for (k, d) in deltas {
            self.rows.push((self.head, k, d));
        }
    }
}

impl Base for History {
    fn frontier(&self) -> Epoch {
        self.head
    }
    fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
        let mut total = 0;
        let mut rows = 0;
        for (e, k, d) in &self.rows {
            if *e <= anchor && k == key {
                total += d;
                rows += 1;
            }
        }
        (total, rows)
    }
    fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
        let mut acc: BTreeMap<Key, Value> = BTreeMap::new();
        for (ep, k, d) in &self.rows {
            if *ep == e {
                *acc.entry(k.clone()).or_insert(0) += d;
            }
        }
        acc.into_iter().collect()
    }
}

fn contract(rung: Consistency) -> ServeContract {
    ServeContract {
        consistency: rung,
        materialize: Materialize::Demand,
        retain: Retention::Forever,
        lineage: niles_ir::Lineage::Off,
    }
}

fn circuit(rung: Consistency) -> Circuit {
    let mut c = Circuit::new();
    let src = c.add(
        IrOp::Source {
            relation: "postings".into(),
            is_base: true,
            anchor_key: vec![0],
            confidential: Vec::new(),
        },
        vec![],
        contract(Consistency::LedgerConsistent),
        "postings",
    );
    let agg = c.add(
        IrOp::Aggregate {
            group_key: vec![0],
            aggs: vec![(Agg::Sum, niles_ir::operator::Scalar::Column(1))],
        },
        vec![src],
        contract(rung),
        "balance",
    );
    c.outputs.insert("balance".into(), agg);
    c
}

struct Sut {
    base: History,
    rt: Runtime,
}

impl Sut {
    fn new(budget: Option<u64>) -> Self {
        Sut {
            base: History::default(),
            rt: Runtime::install(circuit(Consistency::Snapshot), budget, Policy::Lru).unwrap(),
        }
    }

    fn drive(&mut self, op: &Op) {
        match op {
            Op::Open {
                accounts,
                cur: _,
                each,
            } => {
                let mut d = Vec::new();
                for a in 0..*accounts {
                    d.push((vec![a as i64], *each as Value));
                }
                self.base.seal(d);
                let head = self.base.frontier();
                self.rt.advance(&self.base, head);
            }
            Op::Submit { from, to, amt, .. } => {
                self.base.seal(vec![
                    (vec![from.0 as i64], -(*amt as Value)),
                    (vec![to.0 as i64], *amt as Value),
                ]);
                let head = self.base.frontier();
                self.rt.advance(&self.base, head);
            }
            // A duplicate never reaches the base: admission refuses it upstream, and the
            // runtime has no admission path of its own.
            Op::Duplicate { .. } | Op::Read { .. } => {}
            Op::Evict | Op::Crash => {
                self.rt.view_mut("balance").unwrap().wipe();
            }
        }
    }
}

impl Observable for Sut {
    fn balance_at(&mut self, acct: OAcct, _cur: OCur, anchor: u64) -> Result<Answer, Unavailable> {
        if anchor > self.base.frontier() {
            return Err(Unavailable::BeyondFrontier);
        }
        let key = vec![acct.0 as i64];
        let a = self
            .rt
            .view_mut("balance")
            .unwrap()
            .read(&self.base, &key, anchor);
        Ok(Answer {
            value: a.value,
            anchor: a.anchor,
        })
    }
}

/// The oracle's issuance account is 9,999 and is not among the keys the schedule reads,
/// so the runtime never sees it. The comparison is over the funded accounts.
#[test]
fn the_rev_runtime_agrees_with_the_oracle_at_randomly_sampled_anchors() {
    let mut total = 0u64;
    for seed in [1u64, 7, 42, 100, 2024] {
        let s = schedule(seed, 8, 900);
        // Three slots against eight keys: eviction runs continuously.
        let mut sut = Sut::new(Some(3));
        let r = differential(&mut sut, &s, |sut, op| sut.drive(op));
        assert!(
            r.ok(),
            "seed {seed}: {} divergences, first {:?}",
            r.divergences.len(),
            r.divergences.first()
        );
        total += r.compared;
    }
    assert!(total >= 1000, "{total} comparisons");
}

#[test]
fn full_materialization_agrees_too() {
    // Mode is cost, not semantics (Theorem 4.5(a)). Two budgets, one answer.
    for seed in [1u64, 42] {
        let s = schedule(seed, 8, 500);
        let mut partial = Sut::new(Some(2));
        let mut full = Sut::new(None);
        let a = differential(&mut partial, &s, |sut, op| sut.drive(op));
        let b = differential(&mut full, &s, |sut, op| sut.drive(op));
        assert!(a.ok() && b.ok(), "seed {seed}");
        assert_eq!(a.compared, b.compared);
    }
}

#[test]
fn the_answers_survive_a_crash_campaign() {
    for seed in [1u64, 7, 2024] {
        let base = schedule(seed, 8, 500);
        let campaign = inject(&base, Fault::Crash, 9);
        let mut sut = Sut::new(Some(3));
        let r = differential(&mut sut, &campaign, |sut, op| sut.drive(op));
        assert!(r.compared > 100);
        assert!(r.ok(), "seed {seed}: {:?}", r.divergences.first());
    }
}

#[test]
fn the_answers_survive_an_eviction_storm() {
    for seed in [7u64, 100] {
        let base = schedule(seed, 8, 400);
        let campaign = inject(&base, Fault::EvictionStorm { times: 6 }, 5);
        let mut sut = Sut::new(Some(2));
        let r = differential(&mut sut, &campaign, |sut, op| sut.drive(op));
        assert!(r.ok(), "seed {seed}: {:?}", r.divergences.first());
    }
}

#[test]
fn the_differential_would_catch_a_wrong_runtime() {
    struct Broken(Sut);
    impl Observable for Broken {
        fn balance_at(
            &mut self,
            acct: OAcct,
            cur: OCur,
            anchor: u64,
        ) -> Result<Answer, Unavailable> {
            if anchor > 2 {
                return Ok(Answer { value: 0, anchor });
            }
            self.0.balance_at(acct, cur, anchor)
        }
    }
    let s = schedule(1, 8, 400);
    let mut b = Broken(Sut::new(Some(3)));
    let r = differential(&mut b, &s, |b, op| b.0.drive(op));
    assert!(!r.ok());
}
