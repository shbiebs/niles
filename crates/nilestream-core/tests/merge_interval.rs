//! **A merged entry is stamped where the key's history ends, not where the view's does.**
//!
//! F-11-14. Both installs in cycle 10's A/B were point intervals. The pinned control installs
//! at the fold's own anchor `a` and certifies `[a, a]`; the merge installed at `applied` and
//! certified `[applied, applied]`. On Host C the readers arrive *behind* `applied` by the
//! arrival gap, so the pinned point caught roughly nineteen of twenty hot-key readers and the
//! merged point caught almost none — the merge arm folded 8.5x more, held the base shared 8.5x
//! more, and starved the writers, which is A11-07's rows.
//!
//! The value was never wrong. `[applied, applied]` is a true interval. It is simply the
//! *narrowest* true interval available, and nothing in the suite said which one was installed:
//! before this file, **no test anywhere asserted the stamp of a merged entry**, which is how a
//! merge could certify one epoch instead of thirty-two and pass every gate for a cycle.
//!
//! The walk already reads every epoch in `(a, applied]`. So it also knows the epoch `d` of the
//! last delta *for this key*, and that the value is unchanged from `d` through `applied` — that
//! is what "no delta in between" means. Installing at `d` un-pinned certifies `[d, applied]`,
//! which is the widest interval the walk's own evidence supports. Where the suffix moved other
//! keys but not this one, `d` does not exist and the fold's own anchor `a` is already the end
//! of this key's history: the entry certifies `[a, applied]`, the whole suffix.
//!
//! # The fixture the earlier file could not be
//!
//! `merge_budget.rs`'s base gives its hot key a delta in *every* epoch, so `d == applied`
//! always and the old install and the new one are the same stamp. That is why it could measure
//! the merge's cost and say nothing about its interval. `Stops` below gives the key its last
//! delta at a stated epoch and moves other keys after it — a key that stopped changing, which
//! is the ordinary case and the one the whole repair is about.

use niles_ir::circuit::Circuit;
use niles_ir::operator::{Agg, Op, Scalar};
use niles_ir::{Consistency, Lineage, Materialize, Retention, ServeContract};
use nilestream_core::absence::Epoch;
use nilestream_core::rev::{Base, Key, MergeCaps, Policy, ReadMode, ReadOutcome, Runtime, Value};

/// The hot key receives one unit in every epoch up to and including `stops`; after that the
/// epochs are real and busy but belong to other keys.
struct Stops {
    head: Epoch,
    stops: Epoch,
}

const HOT: i64 = 1;

impl Stops {
    fn new(head: Epoch, stops: Epoch) -> Stops {
        Stops { head, stops }
    }
    /// The oracle, computed from the fixture's definition rather than from the engine: the hot
    /// key's balance as of `anchor` is one unit per epoch up to `min(anchor, stops)`.
    fn truth(&self, anchor: Epoch) -> Value {
        anchor.min(self.stops) as Value
    }
}

impl Base for Stops {
    /// The plan this fixture's runtime installs: a keyed `sum` of column 1 of
    /// `postings`, grouped by column 0. C11-01 requires the base to say what it
    /// answers so the view can refuse a base that answers a different question;
    /// stating it wrongly here would make `require_base` assert, which is the point.
    fn answers(&self) -> &nilestream_core::rev::BasePlan {
        static PLAN: std::sync::OnceLock<nilestream_core::rev::BasePlan> =
            std::sync::OnceLock::new();
        PLAN.get_or_init(|| nilestream_core::rev::BasePlan::sum("postings", 1, vec![0]))
    }
    fn frontier(&self) -> Epoch {
        self.head
    }
    fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
        if key == &vec![HOT] {
            (self.truth(anchor), anchor.min(self.stops))
        } else {
            (0, 0)
        }
    }
    fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
        if e == 0 || e > self.head {
            return Vec::new();
        }
        if e <= self.stops {
            vec![(vec![HOT], 1)]
        } else {
            // Busy, and none of it this key's. A walk that skipped these would still get the
            // right *value*; it would get the wrong *stamp*, which is the point here.
            vec![(vec![900 + e as i64], 1)]
        }
    }
    fn delta_rows_at(&self, _e: Epoch) -> u64 {
        1
    }
    fn deltas_available_from(&self) -> Epoch {
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

fn runtime() -> Runtime {
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
    // The merge is the subject, so it is armed explicitly. `MergeCaps::default()` is OFF by
    // C11-05(b) and a test that inherited its arm from the default would silently stop
    // testing anything the day the default changed — which is how four tests in this
    // repository came to need re-arming at once.
    rt.set_merge_caps(MergeCaps::ON);
    rt
}

/// Fold at `anchor`, advance the view to `advance_to`, then land the fold late. The returned
/// value is the owner's own answer, which no merge may alter.
fn land_late(rt: &mut Runtime, base: &Stops, key: &Key, anchor: Epoch, advance_to: Epoch) -> Value {
    let ticket = {
        let view = rt.view_mut("balance").expect("the view");
        match view.begin_read_with(key, anchor, ReadMode::Alone) {
            ReadOutcome::Fold(t) => t,
            _ => panic!("the read should have been authorised to fold"),
        }
    };
    let (value, rows) = base.reconstruct(ticket.key(), ticket.anchor());
    rt.advance(base, advance_to);
    let view = rt.view_mut("balance").expect("the view");
    view.finish_fold_merging(ticket, value, rows, Some(base as &dyn Base))
        .value
}

/// Did a read at `anchor` come out of the view, or did it have to fold? `begin_read_with`
/// answers exactly that question and is the only honest way to ask it — an `Anchored` looks
/// the same either way, which is the whole reason the fold fraction had to be counted.
fn is_hit(rt: &mut Runtime, key: &Key, anchor: Epoch) -> bool {
    let view = rt.view_mut("balance").expect("the view");
    matches!(
        view.begin_read_with(key, anchor, ReadMode::Alone),
        ReadOutcome::Hit(_)
    )
}

#[test]
fn a_merged_entry_is_stamped_at_the_last_delta_and_certifies_through_the_frontier() {
    // The key's history ends at 3; the view is advanced to 12. Before this change the merged
    // entry was stamped 12 and certified `[12, 12]`.
    let base = Stops::new(12, 3);
    let mut rt = runtime();
    let key: Key = vec![HOT];

    let owner = land_late(&mut rt, &base, &key, 1, 12);
    assert_eq!(
        owner,
        base.truth(1),
        "the owner keeps its own answer at its own anchor"
    );
    assert_eq!(rt.stats().deferred_merges, 1, "the merge was taken");

    // **The interval, read end to end.** Every anchor from the last delta to the frontier
    // must hit, and every one of them must hit with the right value.
    for a in 3..=12 {
        assert!(
            is_hit(&mut rt, &key, a),
            "anchor {a} is inside [3, 12] and must be served by the merged entry; before \
             C11-05(c) only anchor 12 was, which is F-11-14"
        );
    }
}

#[test]
fn the_stamp_is_the_last_delta_so_an_anchor_below_it_must_still_reconstruct() {
    // The lower bound is not decoration. An anchor below `d` is a read below the entry: a
    // delta landed in `(anchor, d]` that this anchor must not see.
    let base = Stops::new(12, 3);
    let mut rt = runtime();
    let key: Key = vec![HOT];
    land_late(&mut rt, &base, &key, 1, 12);

    assert!(
        !is_hit(&mut rt, &key, 2),
        "anchor 2 is below the last delta at 3 and must reconstruct; serving it would hand \
         back a value that includes the epoch-3 delta under an anchor that excludes it"
    );
}

#[test]
fn a_suffix_with_no_delta_for_this_key_certifies_the_whole_suffix_from_the_folds_own_anchor() {
    // The key stopped at 3 and the fold is anchored at 5, so the suffix `(5, 12]` moves other
    // keys and never this one. There is no `d`; the fold's own anchor is already the end of
    // this key's history, so the entry certifies `[5, 12]` — and must not be pinned, which
    // before this change it was, because `install` pins anything stamped below `applied`.
    let base = Stops::new(12, 3);
    let mut rt = runtime();
    let key: Key = vec![HOT];

    let owner = land_late(&mut rt, &base, &key, 5, 12);
    assert_eq!(owner, base.truth(5));
    assert_eq!(rt.stats().deferred_merges, 1, "the walk completed");
    assert_eq!(
        rt.stats().pinned_installs,
        0,
        "a merge that proved the key did not move is not a historical read, and counting it \
         as a pinned install was wrong before this change as well as after it"
    );

    for a in 5..=12 {
        assert!(
            is_hit(&mut rt, &key, a),
            "anchor {a} is inside [5, 12]; the walk read every epoch in it and found no delta \
             for this key, so the value is unchanged across all of it"
        );
    }
}

#[test]
fn every_anchor_the_view_answers_agrees_with_the_fixtures_own_oracle() {
    // **The differential oracle names no key.** The interval widened; if it widened past what
    // the deltas support, some anchor inside it now returns a value the fixture's own
    // definition disagrees with. Computed from `Stops::truth`, which is arithmetic over the
    // fixture's statement of itself and touches neither the runtime nor the view.
    for stops in [0u64, 1, 3, 7, 12] {
        for anchor in [1u64, 2, 5, 9] {
            let base = Stops::new(12, stops);
            let mut rt = runtime();
            let key: Key = vec![HOT];
            let owner = land_late(&mut rt, &base, &key, anchor, 12);
            assert_eq!(
                owner,
                base.truth(anchor),
                "owner's answer at anchor {anchor} with the key stopping at {stops}"
            );
            for a in 0..=12 {
                let view = rt.view_mut("balance").expect("the view");
                let got = view.read(&base as &dyn Base, &key, a);
                assert_eq!(
                    got.value,
                    base.truth(a),
                    "anchor {a}, key stops at {stops}, fold anchored at {anchor}"
                );
                assert_eq!(
                    got.anchor, a,
                    "an answer is stamped with the anchor asked for"
                );
            }
        }
    }
}

#[test]
fn the_arrival_gap_reader_hits_the_merged_entry() {
    // **The §2.2 diagnostic, promoted to a test.** A reader does not arrive at `applied`; it
    // arrives behind it by the arrival gap, because epochs land while it is in flight. Host C
    // measured that gap and the merged entry's point stamp sat above every one of them.
    //
    // Here the gap is three epochs and the key's own history ended long before: a reader at
    // `applied - 3` must be served. This is the assertion that fails at `FOLD` where it wants
    // `HIT` if the install goes back to `applied`.
    let base = Stops::new(20, 4);
    let mut rt = runtime();
    let key: Key = vec![HOT];
    land_late(&mut rt, &base, &key, 2, 20);

    for gap in 1..=8 {
        let anchor = 20 - gap;
        assert!(
            is_hit(&mut rt, &key, anchor),
            "a reader arriving {gap} epoch(s) behind the frontier folded instead of hitting; \
             this is exactly the population that made the merge arm fold 8.5x more than the \
             control it was meant to beat"
        );
    }
}

#[test]
fn the_frontier_cannot_move_between_the_walk_and_the_install_today() {
    // **A tripwire, and it is written down as one.** `merge_suffix` runs under the same
    // `&mut self` as the install that follows it, so `applied` cannot advance in between and
    // `merges_refused_moved` cannot fire. Asserting the zero is what makes it fail if that
    // ever stops being true — and LC-37 / C11-12, which move the walk out from under the
    // second view acquisition, are exactly the change that would make it fire.
    //
    // The guard for this arm does not go red on reversion, and the report says so rather than
    // claiming a defect that cannot be reproduced.
    let base = Stops::new(12, 3);
    let mut rt = runtime();
    let key: Key = vec![HOT];
    land_late(&mut rt, &base, &key, 1, 12);
    assert_eq!(
        rt.stats().merges_refused_moved,
        0,
        "the frontier moved under a walk that holds `&mut self`, which should not be reachable"
    );
    assert_eq!(
        rt.stats().deferred_merges,
        1,
        "and the merge was taken, not refused"
    );
}

#[test]
fn a_refused_merge_still_pins_and_the_owner_still_gets_its_own_answer() {
    // The control for all of the above: with the merge refused, nothing widens. The entry is
    // a historical read again, pinned at its own anchor, and an anchor above it reconstructs.
    let base = Stops::new(12, 3);
    let mut rt = runtime();
    rt.set_merge_caps(MergeCaps::OFF);
    let key: Key = vec![HOT];

    let owner = land_late(&mut rt, &base, &key, 1, 12);
    assert_eq!(
        owner,
        base.truth(1),
        "the owner's answer is not the merge's to change"
    );
    assert_eq!(rt.stats().deferred_merges, 0, "no merge was taken");
    assert_eq!(rt.stats().pinned_installs, 1);
    assert!(
        is_hit(&mut rt, &key, 1),
        "the pinned point is still a hit at its own anchor"
    );
    assert!(
        !is_hit(&mut rt, &key, 2),
        "and a point interval is a point: anchor 2 reconstructs"
    );
}
