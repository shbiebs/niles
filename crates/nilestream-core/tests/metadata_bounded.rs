//! **Per-key policy metadata is bounded by the residency budget, on every read path.**
//!
//! `begin_read_with` created an eviction-policy entry for every distinct key it was ever
//! asked about, and only eviction of a *resident* victim removed one. So the map grew with
//! the history of questions rather than with the size of the answer, and the four ways a read
//! can end without installing all leaked: a refused admission, a competing flight at another
//! anchor, an abandoned ticket, an uninstalled fold.
//!
//! The measurement the cycle-11 foundations audit made: ten thousand distinct keys read once
//! each with a residency budget of one hundred left `metadata_len() == 10_000` and
//! `resident_count() == 0`. It is on the wire as `view_metadata_keys`, so an operator could
//! watch it grow and had nothing to compare it against.
//!
//! Neither policy can read such an entry. `Lru` and `CostAware` both filter to `resident()`
//! before consulting `meta`, so metadata for a non-resident key is unreachable by
//! construction — it was pure growth, not a cache of anything.
//!
//! # What this file asserts
//!
//! That every path which does not install leaves no entry, that installing creates exactly
//! one, and that eviction removes it — so the map's size is the residency budget and not the
//! number of keys the view has seen. The controls matter as much as the leaks: a policy that
//! cannot rank its residents is a policy, not a repair.

use niles_ir::circuit::Circuit;
use niles_ir::operator::{Agg, Op, Scalar};
use niles_ir::{Consistency, Lineage, Materialize, Retention, ServeContract};
use nilestream_core::absence::Epoch;
use nilestream_core::rev::{Base, BasePlan, Key, Policy, ReadMode, ReadOutcome, Runtime, Value};

/// A base with one unit for every key at every epoch, so any key can be reconstructed.
struct Flat {
    head: Epoch,
}

impl Base for Flat {
    fn answers(&self) -> &BasePlan {
        static PLAN: std::sync::OnceLock<BasePlan> = std::sync::OnceLock::new();
        PLAN.get_or_init(|| BasePlan::sum("postings", 1, vec![0]))
    }
    fn frontier(&self) -> Epoch {
        self.head
    }
    fn reconstruct(&self, _k: &Key, anchor: Epoch) -> (Value, u64) {
        (anchor as Value, 1)
    }
    fn deltas_at(&self, _e: Epoch) -> Vec<(Key, Value)> {
        Vec::new()
    }
    fn delta_rows_at(&self, _e: Epoch) -> u64 {
        0
    }
}

fn contract() -> ServeContract {
    ServeContract {
        consistency: Consistency::LedgerConsistent,
        materialize: Materialize::Demand,
        retain: Retention::Evictable,
        lineage: Lineage::Key,
    }
}

fn runtime(budget: u64, policy: Policy) -> Runtime {
    let mut c = Circuit::new();
    let src = c.add(
        Op::Source {
            relation: "postings".into(),
            is_base: true,
            anchor_key: vec![0],
            confidential: Vec::new(),
        },
        vec![],
        contract(),
        "postings",
    );
    let agg = c.add(
        Op::Aggregate {
            group_key: vec![0],
            aggs: vec![(Agg::Sum, Scalar::Column(1))],
        },
        vec![src],
        contract(),
        "balance",
    );
    c.set_output("balance", agg);
    Runtime::install(c, Some(budget), policy).expect("a supported fragment")
}

/// `(resident, metadata_len)` — the two numbers this file is about.
fn shape(rt: &mut Runtime) -> (u64, usize) {
    let v = rt.view_mut("balance").expect("the view");
    (v.resident_count(), v.metadata_len())
}

const BUDGET: u64 = 100;
const KEYS: i64 = 10_000;

#[test]
fn a_read_that_installs_leaves_metadata_bounded_by_the_budget() {
    let base = Flat { head: 1 };
    let mut rt = runtime(BUDGET, Policy::Lru);
    for k in 0..KEYS {
        rt.view_mut("balance").unwrap().read(&base, &vec![k], 1);
    }
    let (resident, meta) = shape(&mut rt);
    assert_eq!(resident, BUDGET, "the budget is the residency bound");
    assert_eq!(
        meta, BUDGET as usize,
        "installing reads left {meta} metadata entries for {resident} resident keys"
    );
}

#[test]
fn an_abandoned_ticket_leaves_no_metadata_at_all() {
    // **The leak the audit measured.** `begin_read_with` then drop: the read never installs,
    // and before this repair each one left an entry behind for ever.
    let mut rt = runtime(BUDGET, Policy::Lru);
    for k in 0..KEYS {
        let v = rt.view_mut("balance").unwrap();
        if let ReadOutcome::Fold(t) = v.begin_read_with(&vec![k], 1, ReadMode::Alone) {
            drop(t);
        }
    }
    let (resident, meta) = shape(&mut rt);
    assert_eq!(resident, 0, "nothing installed, so nothing is resident");
    assert_eq!(
        meta, 0,
        "{KEYS} abandoned reads left {meta} metadata entries with {resident} residents; the \
         map is supposed to be bounded by the budget ({BUDGET}), not by the number of keys \
         this view has ever been asked about"
    );
}

#[test]
fn a_refused_admission_leaves_no_metadata() {
    // Fill the flight table with live owners, then read past it. The refused reads fold
    // alone and install nothing.
    let mut rt = runtime(BUDGET, Policy::Lru);
    let mut held = Vec::new();
    for k in 0..256i64 {
        let v = rt.view_mut("balance").unwrap();
        if let ReadOutcome::Fold(t) = v.begin_read_with(&vec![k], 1, ReadMode::Alone) {
            held.push(t);
        }
    }
    let before = shape(&mut rt).1;
    for k in 100_000..100_000 + KEYS {
        let v = rt.view_mut("balance").unwrap();
        let _ = v.begin_read_with(&vec![k], 1, ReadMode::Alone);
    }
    let (resident, meta) = shape(&mut rt);
    assert_eq!(resident, 0);
    assert_eq!(
        meta,
        before,
        "{KEYS} refused admissions added {} metadata entries",
        meta - before
    );
    drop(held);
}

#[test]
fn an_uninstalled_fold_leaves_no_metadata() {
    // A fold that completes and installs nothing: the second reader of a key arrives at a
    // *different* anchor, so it folds independently rather than joining (LC-30), and its
    // completion is counted `uninstalled_folds` and writes no slot. This is the path the
    // counter table names for that counter.
    let base = Flat { head: 8 };
    let mut rt = runtime(BUDGET, Policy::Lru);
    let mut owners = Vec::new();
    for k in 0..1_000i64 {
        let key: Key = vec![k];
        let owner = {
            let v = rt.view_mut("balance").unwrap();
            match v.begin_read_with(&key, 1, ReadMode::Alone) {
                ReadOutcome::Fold(t) => t,
                _ => panic!("the first read of a cold key folds"),
            }
        };
        let competing = {
            let v = rt.view_mut("balance").unwrap();
            match v.begin_read_with(&key, 2, ReadMode::Alone) {
                ReadOutcome::Fold(t) => t,
                _ => panic!("a second anchor folds independently"),
            }
        };
        let v = rt.view_mut("balance").unwrap();
        v.finish_fold_merging(competing, 2, 1, Some(&base as &dyn Base));
        owners.push(owner);
    }
    let uninstalled = rt.stats().uninstalled_folds;
    let (resident, meta) = shape(&mut rt);
    assert!(
        uninstalled > 0,
        "this fixture is supposed to produce uninstalled folds and produced none"
    );
    assert_eq!(resident, 0, "no fold installed");
    assert_eq!(
        meta, 0,
        "{uninstalled} uninstalled fold(s) left {meta} metadata entries"
    );
    drop(owners);
}

#[test]
fn eviction_removes_the_entry_with_the_slot() {
    let base = Flat { head: 1 };
    let mut rt = runtime(BUDGET, Policy::Lru);
    for k in 0..(BUDGET as i64 * 10) {
        rt.view_mut("balance").unwrap().read(&base, &vec![k], 1);
    }
    let (resident, meta) = shape(&mut rt);
    assert_eq!(resident, BUDGET);
    assert_eq!(
        meta, resident as usize,
        "one entry per resident key, exactly"
    );
}

#[test]
fn both_policies_can_still_rank_their_residents() {
    // **The control.** A repair that bounded the map by removing the information the policy
    // needs would pass every assertion above and break eviction. Read one key far more often
    // than the rest and require that it survives.
    for policy in [Policy::Lru, Policy::CostAware] {
        let base = Flat { head: 1 };
        let mut rt = runtime(8, policy);
        let hot: Key = vec![7];
        for round in 0..40i64 {
            rt.view_mut("balance").unwrap().read(&base, &hot, 1);
            rt.view_mut("balance")
                .unwrap()
                .read(&base, &vec![1_000 + round], 1);
        }
        let v = rt.view_mut("balance").unwrap();
        assert!(
            matches!(
                v.begin_read_with(&hot, 1, ReadMode::Alone),
                ReadOutcome::Hit(_)
            ),
            "{policy:?} evicted the key it was read most often; the policy can no longer \
             rank its residents, which is a worse defect than the one being repaired"
        );
    }
}

#[test]
fn the_wire_counter_reports_the_bounded_number() {
    // `view_metadata_keys` is what an operator watches. It was the growing number and had
    // nothing to compare against; it is now the budget.
    let base = Flat { head: 1 };
    let mut rt = runtime(BUDGET, Policy::Lru);
    for k in 0..KEYS {
        rt.view_mut("balance").unwrap().read(&base, &vec![k], 1);
        let v = rt.view_mut("balance").unwrap();
        if let ReadOutcome::Fold(t) = v.begin_read_with(&vec![k + 500_000], 1, ReadMode::Alone) {
            drop(t);
        }
    }
    let (_, meta) = shape(&mut rt);
    assert!(
        meta <= BUDGET as usize,
        "after {KEYS} installing reads interleaved with {KEYS} abandoned ones the view holds \
         {meta} metadata entries against a budget of {BUDGET}"
    );
}

#[test]
fn the_slot_map_is_not_bounded_by_the_budget_and_this_records_how_much() {
    // **A finding, asserted at its present size rather than left to drift.**
    //
    // The policy metadata above is bounded now. The *slots* are not: an abandoned read of a
    // cold key leaves `Slot::Pending`, `reap_cancelled` only runs when that key is read
    // again, and for a cold key it has no `prior` to restore anyway. Eviction never selects a
    // marker because `is_resident` is false for one — which is right while the flight is
    // live and unbounded once it is gone.
    //
    // This is not the card's repair and does not pretend to be. It is here so the number
    // cannot change without someone noticing, and so the next card has its before-value.
    let mut rt = runtime(BUDGET, Policy::Lru);
    for k in 0..KEYS {
        let v = rt.view_mut("balance").unwrap();
        if let ReadOutcome::Fold(t) = v.begin_read_with(&vec![k], 1, ReadMode::Alone) {
            drop(t);
        }
    }
    let v = rt.view_mut("balance").unwrap();
    let (slots, meta, flights, resident) = (
        v.slots_len(),
        v.metadata_len(),
        v.in_flight_len(),
        v.resident_count(),
    );
    println!("slots={slots} meta={meta} in_flight={flights} resident={resident}");

    assert_eq!(meta, 0, "the metadata half of this card");
    assert_eq!(resident, 0, "nothing installed");
    assert!(
        flights <= 256,
        "C11-04 bounds the flight table at MAX_FLIGHTS and it holds {flights}"
    );
    assert_eq!(
        slots, KEYS as usize,
        "**the unrepaired leak, recorded**: {KEYS} abandoned reads left {slots} slots with \
         {resident} residents against a budget of {BUDGET}. If this number has fallen, the \
         marker leak has been repaired and this test should become the assertion that it \
         stays repaired; if it has risen, something else is retaining markers too."
    );
}
