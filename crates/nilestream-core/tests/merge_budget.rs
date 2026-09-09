//! **A merge that refuses a wide epoch must not have built it first.**
//!
//! `merge_suffix` walked the suffix calling `Base::deltas_at(e)`, added `rows.len()` to a
//! running total, and refused once the total passed `max_rows`. So the refusal happened
//! *after* the allocation it was refusing: with a cap of 2 against a 10,000-row epoch, ten
//! thousand rows were materialised and dropped, and `merge_rows_visited` reported ten
//! thousand — honest about the cost, and silent about the bound not being enforced.
//!
//! The audit's probe reported exactly that: `row cap=2; source rows materialized=10000;
//! reported rows visited=10000; answer preserved`. The last clause is why it is a resource
//! defect and not a correctness one, and why no answer-level test could find it.
//!
//! # What is asserted here
//!
//! The base counts its own materialisations, so "how many rows were built" is a measurement
//! and not an inference. Then:
//!
//! * refusing a wide epoch materialises **nothing**;
//! * a merge inside budget materialises at most `max_rows` rows over the whole walk;
//! * **no sentinel row** is inspected — the count is exact, so the decision never needs to
//!   look one row past the cap;
//! * every refusal preserves the answer the caller was going to get anyway;
//! * the epoch cap, the row cap, an unavailable prefix and an arithmetic overflow are four
//!   counters and not one, because they are four different things to do about it.

use niles_ir::circuit::Circuit;
use niles_ir::operator::{Agg, Op, Scalar};
use niles_ir::{Consistency, Lineage, Materialize, Retention, ServeContract};
use nilestream_core::absence::Epoch;
use nilestream_core::rev::{Base, Key, MergeCaps, Policy, ReadMode, ReadOutcome, Runtime, Value};
use std::cell::Cell;

/// A base whose epochs are as wide as asked for, and which **counts what it materialises**.
struct Wide {
    /// Rows per epoch.
    width: u64,
    head: Epoch,
    /// The key every row in the fixture belongs to, so a merge has something to accumulate.
    hot: i64,
    /// Rows actually built by `deltas_at`.
    materialised: Cell<u64>,
    /// Calls to `delta_rows_at`, which build nothing.
    counted: Cell<u64>,
    /// The delta each row carries.
    per_row: Value,
    /// The earliest epoch this base still holds.
    from: Epoch,
}

impl Wide {
    fn new(width: u64, head: Epoch) -> Wide {
        Wide {
            width,
            head,
            hot: 1,
            materialised: Cell::new(0),
            counted: Cell::new(0),
            per_row: 1,
            from: 0,
        }
    }
}

impl Base for Wide {
    fn frontier(&self) -> Epoch {
        self.head
    }
    fn reconstruct(&self, _key: &Key, anchor: Epoch) -> (Value, u64) {
        // One row per epoch belongs to the hot key, so the fold at `anchor` is `anchor`.
        (anchor as Value, anchor)
    }
    fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
        if e == 0 || e > self.head {
            return Vec::new();
        }
        self.materialised.set(self.materialised.get() + self.width);
        // The hot key first, then cold filler, so a walk that stopped early would still have
        // found the interesting row — a fixture that hid it behind the filler would make a
        // partial walk look correct.
        let mut v = vec![(vec![self.hot], self.per_row)];
        for i in 1..self.width {
            v.push((vec![1_000 + i as i64], 1));
        }
        v
    }
    fn delta_rows_at(&self, e: Epoch) -> u64 {
        self.counted.set(self.counted.get() + 1);
        if e == 0 || e > self.head {
            0
        } else {
            self.width
        }
    }
    fn deltas_available_from(&self) -> Epoch {
        self.from
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

fn runtime(caps: MergeCaps) -> Runtime {
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
    let mut rt = Runtime::install(c, Some(100_000), Policy::Lru).expect("a supported fragment");
    rt.set_merge_caps(caps);
    rt
}

/// Fold at `anchor`, let the view advance past it, and land the fold late — which is the only
/// way to reach the merge at all.
fn land_late(rt: &mut Runtime, base: &Wide, key: &Key, anchor: Epoch, advance_to: Epoch) -> Value {
    let ticket = {
        let view = rt.view_mut("balance").expect("the view");
        match view.begin_read_with(key, anchor, ReadMode::Alone) {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the read should have been authorised to fold"),
        }
    };
    let (value, rows) = base.reconstruct(ticket.key(), ticket.anchor());
    rt.advance(base, advance_to);
    // **Zeroed here, so the counts below are the merge's and not maintenance's.**
    // `advance` calls `deltas_at` for every epoch it applies, which is a real and legitimate
    // materialisation by `apply_epoch`; leaving it in the total would make every assertion in
    // this file about two mechanisms and would have hidden the one it is for.
    base.materialised.set(0);
    base.counted.set(0);
    let view = rt.view_mut("balance").expect("the view");
    view.finish_fold_merging(ticket, value, rows, Some(base as &dyn Base))
        .value
}

#[test]
fn refusing_a_wide_epoch_materialises_nothing() {
    // The probe's numbers: a cap of 2 rows against a 10,000-row epoch.
    let base = Wide::new(10_000, 4);
    let mut rt = runtime(MergeCaps {
        max_epochs: 32,
        max_rows: 2,
    });
    let key: Key = vec![1];
    let answer = land_late(&mut rt, &base, &key, 1, 3);

    assert_eq!(
        base.materialised.get(),
        0,
        "the merge refused a suffix it had already built: {} rows materialised against a cap \
         of 2",
        base.materialised.get()
    );
    assert!(
        base.counted.get() >= 1,
        "the refusal must have cost a count, or it did not look at the suffix at all"
    );
    // **The refusal preserves the answer.** This is why the defect is a resource defect: the
    // number was always right.
    assert_eq!(answer, 1, "the owner's own fold, at its own anchor");
    let s = rt.stats();
    assert_eq!(s.merges_refused_rows, 1);
    assert_eq!(s.merges_refused_epochs, 0);
    assert_eq!(s.merges_refused_unavailable, 0);
    assert_eq!(s.merges_refused_overflow, 0);
    assert_eq!(s.deferred_merges, 0);
}

#[test]
fn a_merge_within_budget_materialises_at_most_the_cap_and_no_sentinel() {
    // Two epochs of three rows each against a cap of six: exactly at the boundary, so a walk
    // that inspected one row past the cap would show seven.
    let base = Wide::new(3, 4);
    let mut rt = runtime(MergeCaps {
        max_epochs: 32,
        max_rows: 6,
    });
    let key: Key = vec![1];
    let answer = land_late(&mut rt, &base, &key, 1, 3);
    assert_eq!(
        base.materialised.get(),
        6,
        "two epochs of three rows is six, and not one more"
    );
    // The merge landed: the owner's fold at anchor 1 was 1, and epochs 2 and 3 each carry one
    // row for the hot key.
    assert_eq!(
        answer, 1,
        "the owner is answered at its own anchor either way"
    );
    let s = rt.stats();
    assert_eq!(s.deferred_merges, 1, "the merge was taken");
    assert_eq!(s.merge_rows_visited, 6);
    assert_eq!(s.merges_refused_rows, 0);
}

#[test]
fn one_row_over_the_cap_is_refused_and_still_materialises_nothing() {
    // The boundary from the other side: seven rows against a cap of six.
    let base = Wide::new(7, 4);
    let mut rt = runtime(MergeCaps {
        max_epochs: 32,
        max_rows: 6,
    });
    land_late(&mut rt, &base, &vec![1], 1, 3);
    assert_eq!(base.materialised.get(), 0);
    assert_eq!(rt.stats().merges_refused_rows, 1);
}

#[test]
fn a_cap_of_zero_rows_refuses_before_reading_anything() {
    let base = Wide::new(1, 4);
    let mut rt = runtime(MergeCaps {
        max_epochs: 32,
        max_rows: 0,
    });
    land_late(&mut rt, &base, &vec![1], 1, 3);
    assert_eq!(base.materialised.get(), 0);
    assert_eq!(rt.stats().merges_refused_rows, 1);
}

#[test]
fn the_pinned_arm_reads_no_epoch_at_all() {
    // `caps 0 / 0` is the pinned control this cycle measures against, and it must remain a
    // valid arm: the epoch cap refuses first, so not even a count is taken.
    let base = Wide::new(10_000, 4);
    let mut rt = runtime(MergeCaps::OFF);
    let answer = land_late(&mut rt, &base, &vec![1], 1, 3);
    assert_eq!(base.materialised.get(), 0);
    assert_eq!(base.counted.get(), 0, "not even a count");
    assert_eq!(answer, 1);
    let s = rt.stats();
    assert_eq!(s.merges_refused_epochs, 1, "the epoch cap, not the row cap");
    assert_eq!(s.merges_refused_rows, 0);
}

#[test]
fn a_suffix_that_reaches_below_what_the_base_retains_is_its_own_refusal() {
    let mut base = Wide::new(1, 6);
    base.from = 4; // epochs 0..3 are no longer retained
    let mut rt = runtime(MergeCaps {
        max_epochs: 32,
        max_rows: 1_000,
    });
    land_late(&mut rt, &base, &vec![1], 1, 5);
    let s = rt.stats();
    assert_eq!(
        s.merges_refused_unavailable, 1,
        "an unretained prefix is a retention question, not a tuning one"
    );
    assert_eq!(s.merges_refused_rows, 0);
    assert_eq!(base.materialised.get(), 0);
}

#[test]
fn an_accumulation_that_would_overflow_is_refused_and_counted_apart() {
    // `delta += d` was unchecked, so this wrapped in release and panicked in debug — the
    // profile split C11-02 removed from the query evaluator, in the merge.
    let mut base = Wide::new(1, 4);
    base.per_row = i128::MAX;
    let mut rt = runtime(MergeCaps {
        max_epochs: 32,
        max_rows: 1_000,
    });
    let answer = land_late(&mut rt, &base, &vec![1], 1, 3);
    let s = rt.stats();
    assert_eq!(
        s.merges_refused_overflow, 1,
        "an arithmetic fact about the data, not a budget"
    );
    assert_eq!(s.merges_refused_rows, 0);
    assert_eq!(s.merges_refused_epochs, 0);
    assert_eq!(answer, 1, "and the caller's own answer is untouched");
}

#[test]
fn the_four_refusals_are_four_counters() {
    // A repair that reported every refusal under one name would pass every assertion above
    // taken one at a time. This is the assertion that they are distinguishable.
    let mut seen = Vec::new();
    for (name, caps, base) in [
        (
            "epochs",
            MergeCaps {
                max_epochs: 0,
                max_rows: 1_000,
            },
            Wide::new(1, 4),
        ),
        (
            "rows",
            MergeCaps {
                max_epochs: 32,
                max_rows: 0,
            },
            Wide::new(1, 4),
        ),
    ] {
        let mut rt = runtime(caps);
        land_late(&mut rt, &base, &vec![1], 1, 3);
        let s = rt.stats();
        seen.push((
            name,
            s.merges_refused_epochs,
            s.merges_refused_rows,
            s.merges_refused_unavailable,
            s.merges_refused_overflow,
        ));
    }
    assert_eq!(seen[0], ("epochs", 1, 0, 0, 0));
    assert_eq!(seen[1], ("rows", 0, 1, 0, 0));
}
