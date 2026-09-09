//! **The arm selector's contract: what a run gets when nobody names a policy.**
//!
//! `MergeCaps::default()` is `MergeCaps::OFF` as of C11-05(b), by the author's answer to
//! LC-38. This file pins that, and pins the *other* half with it — that turning the mechanism
//! back on is one value away and restores the preregistered 32 / 4,096 — because a decision
//! recorded only in a doc comment is a decision the next change can quietly reverse.
//!
//! # Why the default moved
//!
//! Cycle 10 measured the merge arm against the pinned control on Host C. It lost on all six
//! rows, three of them clearing the ≥10%-and-≥3-pooled-MADs gate, and cost about 30% of
//! writer throughput and 53–80% of write p99 at the partial working point. F-11-14 located
//! the mechanism rather than leaving it as a number: a merged landing installs at `applied`,
//! which no concurrent reader's anchor has reached, so its certification interval sits where
//! nobody is looking — while the pinned arm's point interval catches about nineteen of twenty
//! hot-key readers.
//!
//! So this is a **policy** change and not a mechanism change. The mechanism is unchanged,
//! tested, and one environment variable away; C11-05(c)'s interval repair and C11-05(d)'s
//! Host C measurement decide whether the default comes back. Shipping a default that carries
//! a measured regression would be a default nobody chose; deleting the mechanism because its
//! first measurement lost would publish a negative result about the weakest version of the
//! idea. Off-by-default is neither, and it is reversible in one line.

use nilestream_core::rev::MergeCaps;

#[test]
fn the_default_is_off_because_lc38_said_so() {
    assert_eq!(
        MergeCaps::default(),
        MergeCaps::OFF,
        "the deferred merge is off by default (LC-38, answered `off now, C11-05(c)/(d) \
         decide`). If this is being changed back, it should be because (d) measured a win on \
         Host C — and the change belongs on its own commit, as this one was."
    );
    assert_eq!(MergeCaps::default().max_epochs, 0);
    assert_eq!(MergeCaps::default().max_rows, 0);
}

#[test]
fn turning_it_back_on_restores_the_preregistered_caps_and_not_a_new_pair() {
    // The other half. A future change that flips the default by editing the numbers rather
    // than by selecting `ON` would silently re-tune the mechanism at the same time as
    // re-enabling it, and the measurement that followed would be about two changes.
    assert_eq!(MergeCaps::ON.max_epochs, 32);
    assert_eq!(MergeCaps::ON.max_rows, 4_096);
    assert_ne!(
        MergeCaps::ON,
        MergeCaps::OFF,
        "the two arms must be two arms"
    );
}

#[test]
fn an_explicit_choice_still_wins_over_the_default() {
    // The default is a default and not a lock: a run that names a policy gets it, which is
    // what makes the ablation a value rather than a `cfg`.
    let chosen = MergeCaps {
        max_epochs: 7,
        max_rows: 11,
    };
    assert_ne!(chosen, MergeCaps::default());
    assert_eq!(chosen.max_epochs, 7);
}
