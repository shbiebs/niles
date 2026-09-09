//! **Abandoned owners cannot permanently exhaust admission capacity for unrelated keys.**
//!
//! `begin_read` reaps the *incoming key's* cancelled flight and nothing else. Drop 256
//! installing `FoldTicket`s for 256 distinct cold keys and the flight table is full of dead
//! records; the 257th read, for a key that has no flight at all, sees
//! `in_flight.len() >= MAX_FLIGHTS`, counts `flights_refused`, and folds alone. Nothing ever
//! removes those records, because the only thing that removes the record for key `K` is a
//! read of `K` — so on a server where every read remains *correct*, sharing capacity stays
//! exhausted for the rest of the process's life, and the tail it costs shows up only as
//! latency.
//!
//! The contract the repair has to hold, from the card, is narrower than "clean up":
//!
//! * reclaim **only** a generation whose completion is cancelled — a live flight is not
//!   swept away because the table is under pressure;
//! * preserve the slot state the flight replaced, and **not** over a newer write;
//! * live caps stay enforced, so a table genuinely full of live flights still refuses;
//! * cancellation cannot change an anchor or a value a caller was handed;
//! * the refusal and the reclamation are separate counters, because load and abandonment are
//!   different problems that look identical in one number.
//!
//! Each of those is a test below, and the last two are the ones a repair is most likely to
//! get wrong in the direction that looks correct.

use niles_ir::circuit::Circuit;
use niles_ir::operator::{Agg, Op, Scalar};
use niles_ir::{Consistency, Lineage, Materialize, Retention, ServeContract};
use nilestream_core::absence::Epoch;
use nilestream_core::rev::{Base, Key, Policy, ReadMode, ReadOutcome, Runtime, Value};

/// The cap the runtime enforces. Not imported: it is private, and a test that read it from
/// the code under test would agree with a wrong value.
const MAX_FLIGHTS: usize = 256;

struct Rows;

impl Base for Rows {
    fn frontier(&self) -> Epoch {
        1_000
    }
    fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
        (key[0] as i128 * 10 + anchor as i128, 1)
    }
    fn deltas_at(&self, _e: Epoch) -> Vec<(Key, Value)> {
        Vec::new()
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
    // A budget above anything these tests touch: this file is about the flight table, and a
    // residency budget evicting underneath it would make every result about two mechanisms.
    Runtime::install(c, Some(100_000), Policy::Lru).expect("a supported fragment")
}

/// Take a ticket for `key` and drop it — an owner that died between authorisation and its
/// fold, which is what a cancelled connection looks like from here.
fn abandon(rt: &mut Runtime, key: i64) {
    let view = rt.view_mut("balance").expect("the view");
    match view.begin_read_with(&vec![key], 1, ReadMode::Alone) {
        ReadOutcome::Fold(t) => drop(t),
        _ => panic!("key {key} should have been authorised to fold"),
    }
}

#[test]
fn a_key_with_no_flight_is_admitted_after_two_hundred_and_fifty_six_owners_are_dropped() {
    let mut rt = runtime();
    for k in 0..MAX_FLIGHTS as i64 {
        abandon(&mut rt, k);
    }
    // The distinct key. Nothing about it was ever asked before, and its own reap — the only
    // reclamation the previous version did — finds nothing to reap.
    let view = rt.view_mut("balance").expect("the view");
    let outcome = view.begin_read_with(&vec![9_999], 1, ReadMode::Alone);
    assert!(
        matches!(outcome, ReadOutcome::Fold(ref t) if t.installs()),
        "a key with no flight of its own was refused admission because 256 unrelated owners \
         had been dropped"
    );
    let s = rt.stats();
    assert_eq!(
        s.flights_refused, 0,
        "no read here was refused for want of a live flight"
    );
    assert!(
        s.flights_reclaimed >= MAX_FLIGHTS as u64,
        "the dead records were reclaimed and counted as such, not as refusals: {}",
        s.flights_reclaimed
    );
}

#[test]
fn churn_far_beyond_the_cap_leaves_the_table_at_the_cap_and_not_above_it() {
    // Ten times the cap, all abandoned, then quiescence. The budget assertion from the card:
    // resident flight records ≤ 256, with no growth over the run.
    let mut rt = runtime();
    for k in 0..(10 * MAX_FLIGHTS as i64) {
        abandon(&mut rt, k);
    }
    let view = rt.view_mut("balance").expect("the view");
    assert!(
        view.in_flight_len() <= MAX_FLIGHTS,
        "{} flight records resident after 2,560 abandoned owners",
        view.in_flight_len()
    );
}

#[test]
fn a_table_full_of_live_flights_still_refuses() {
    // The control, and the one a repair is most likely to break: sweeping on capacity
    // pressure must not sweep flights whose owners are alive. The tickets are held in a
    // vector for exactly that reason.
    let mut rt = runtime();
    let mut held = Vec::new();
    for k in 0..MAX_FLIGHTS as i64 {
        let view = rt.view_mut("balance").expect("the view");
        match view.begin_read_with(&vec![k], 1, ReadMode::Alone) {
            ReadOutcome::Fold(t) => held.push(t),
            _ => panic!("key {k} should have been authorised to fold"),
        }
    }
    let view = rt.view_mut("balance").expect("the view");
    let outcome = view.begin_read_with(&vec![9_999], 1, ReadMode::Alone);
    assert!(
        matches!(outcome, ReadOutcome::Fold(ref t) if !t.installs()),
        "a table of 256 live flights must still refuse the 257th"
    );
    assert_eq!(
        rt.stats().flights_refused,
        1,
        "and it is a refusal, not a reclamation"
    );
    assert_eq!(
        rt.stats().flights_reclaimed,
        0,
        "nothing was dead, so nothing may be reported as reclaimed"
    );
    drop(held);
}

#[test]
fn a_reclaimed_flight_does_not_overwrite_a_value_written_while_it_was_in_the_air() {
    // **The window the sweep widened.** Reclamation puts back the slot the flight replaced.
    // With the sweep, that restore can happen an unbounded time after the cancellation and
    // for a key the incoming reader never named — so anything that could write the slot in
    // between now has much longer to do it.
    //
    // Nothing can, today: `apply_epoch` skips a `Pending` slot, eviction never chooses one
    // because `is_resident` is false for it, and there is at most one flight per key. This
    // test therefore **passes against both the guarded and the unguarded restore**, and it
    // is here as the case that would fail first if any of those three decisions changed, not
    // as a witness to a defect. Saying so is the point: a test whose failure mode is
    // unreachable today should say that rather than imply a bug it did not find.
    let mut rt = runtime();
    let key: Key = vec![7];

    // A read that installs, so the key holds a real value.
    let base = Rows;
    let want = {
        let view = rt.view_mut("balance").expect("the view");
        view.read(&base, &key, 1).value
    };

    // An owner takes a flight at a different anchor and dies. Its `prior` is the value above.
    {
        let view = rt.view_mut("balance").expect("the view");
        if let ReadOutcome::Fold(t) = view.begin_read_with(&key, 2, ReadMode::Alone) {
            drop(t);
        }
    }
    // Something else writes the slot while the dead flight's record is still in the table.
    rt.advance(&base, 3);

    // Now force a sweep by filling the table with other abandoned owners.
    for k in 100..(100 + MAX_FLIGHTS as i64) {
        abandon(&mut rt, k);
    }
    let view = rt.view_mut("balance").expect("the view");
    let after = view.read(&base, &key, 1).value;
    assert_eq!(
        after, want,
        "the answer at anchor 1 changed because a reclaimed flight put back what it had \
         replaced, over a slot something else had since written"
    );
}

#[test]
fn a_dropped_owner_never_changes_the_answer_another_reader_was_handed() {
    // The value and anchor a caller received are its own, whatever the view does afterwards.
    let mut rt = runtime();
    let base = Rows;
    let key: Key = vec![3];
    let first = {
        let view = rt.view_mut("balance").expect("the view");
        view.read(&base, &key, 5)
    };
    for k in 0..(2 * MAX_FLIGHTS as i64) {
        abandon(&mut rt, k);
    }
    let view = rt.view_mut("balance").expect("the view");
    let again = view.read(&base, &key, 5);
    assert_eq!(first.value, again.value);
    assert_eq!(first.anchor, again.anchor);
    assert_eq!(first.anchor, 5, "an answer is stamped at what was asked");
}
