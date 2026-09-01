//! **The read path, served by the mechanism the thesis is about.**
//!
//! `nilestreamd` used to answer every query from a `HashMap` and say so in its startup
//! banner. That was honest but it made the server unmeasurable: when the E16 wall-clock
//! harness compared it to PostgreSQL, the `point` row read `PARITY` and the document had to
//! explain at length that the number was the protocol path rather than an engine result.
//!
//! This module closes that. A wire query is answered by
//! [`PartialView::read`](proto_engine::PartialView::read) over an immutable, hash-chained
//! [`Ledger`](proto_engine::Ledger) — partial materialisation, the absence lattice, an
//! anchored upquery on a miss — so the same harness now measures the thing the specification
//! makes claims about.
//!
//! # What is still not a database
//!
//! Everything `proto-engine`'s own module docs say: in-memory, single-threaded, no durability,
//! no consensus, no planner, two view shapes. That list has not shrunk and this module does
//! not pretend otherwise. What has changed is narrower and worth having: a latency measured
//! here is the latency of *a partial view answering an anchored read*, and a miss is a real
//! reconstruction over a real base, so the miss rate is reportable alongside it.
//!
//! # Why the miss rate is reported and not just the latency
//!
//! A parity result at a 0% miss rate and a parity result at a 40% miss rate are different
//! findings, and the phase diagram of thesis §9.3 is built from exactly that difference. A
//! server that reported only latency would let the more interesting of the two disappear, so
//! [`RevEngine::stats`] carries it out to the harness.

use proto_engine::{EvictionPolicy, Ledger, PartialView, Posting, Row, ViewMode};

/// What a read cost, in the units that distinguish one parity result from another.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadStats {
    pub reads: u64,
    pub hits: u64,
    pub misses: u64,
    /// Base rows touched by reconstructions. The counted-work figure the prototype was built
    /// to produce, now available alongside a wall-clock one for the same run.
    pub rows_touched: u64,
    pub resident: usize,
}

impl ReadStats {
    pub fn miss_rate(&self) -> f64 {
        if self.reads == 0 {
            return 0.0;
        }
        self.misses as f64 / self.reads as f64
    }
}

/// A ledger, a partial view over it, and the mapping from a wire query to a read.
pub struct RevEngine {
    ledger: Ledger,
    view: PartialView,
    /// The currency index every wire query uses. The demo schema declares one currency, and a
    /// server that silently answered in whichever currency it found first would be making the
    /// per-currency conservation rule invisible from the outside.
    currency: u32,
    views: Vec<(String, u32)>,
}

impl RevEngine {
    /// Build an engine holding `accounts` accounts, each seeded with `postings_per_account`
    /// balanced transfers against a house account.
    ///
    /// Seeded rather than empty because a benchmark against an empty server reports excellent
    /// latencies for queries that return nothing, and because a partial view over a base with
    /// no history has nothing to reconstruct — the miss path, which is the interesting one,
    /// would never run.
    pub fn seeded(
        accounts: i64,
        postings_per_account: u32,
        budget: usize,
        mode: ViewMode,
        policy: EvictionPolicy,
    ) -> RevEngine {
        let mut ledger = Ledger::new();
        let mut txn = 0u64;
        for round in 0..postings_per_account {
            for a in 1..=accounts as u64 {
                txn += 1;
                // Two conserved legs: the account and the house. The commit rule quantifies
                // over currency, so an unbalanced pair would be refused here rather than
                // discovered as a wrong answer later.
                let amt = 100 + (round as i128 * 7 + a as i128 % 13);
                let rows = vec![
                    Row::Post(Posting { txn, acct: a, cur: 0, amt, valid: round as i64 }),
                    Row::Post(Posting { txn, acct: 0, cur: 0, amt: -amt, valid: round as i64 }),
                ];
                let _ = ledger.submit(&format!("seed-{txn}"), rows);
            }
        }

        let mut view = PartialView::new(mode, budget, policy);
        // Bring the view up to the ledger's head, so a read at the frontier is answered from
        // maintained state rather than by reconstructing the entire history on first touch.
        for e in 1..=ledger.head() {
            view.apply_epoch(&ledger, e);
        }

        RevEngine {
            ledger,
            view,
            currency: 0,
            views: vec![("__wire_result".to_string(), 2)],
        }
    }

    pub fn stats(&self) -> ReadStats {
        let s = self.view.counters();
        ReadStats {
            reads: s.reads,
            hits: s.hits,
            misses: s.misses,
            rows_touched: s.rows_touched,
            resident: self.view.resident(),
        }
    }

    /// Drop everything the view holds, so the next reads all miss.
    ///
    /// The lever the phase diagram is swept with: measuring at a 0% miss rate and at a high
    /// one are different experiments, and a server that could only be measured warm would
    /// only ever produce the flattering half.
    pub fn evict_all(&mut self) {
        self.view.wipe();
    }

    pub fn head(&self) -> u64 {
        self.ledger.head()
    }
}

impl crate::session::Serving for RevEngine {
    fn frontier(&self) -> u64 {
        self.ledger.head()
    }

    fn read(&mut self, _view: &str, key: &[i64], anchor: u64) -> Option<i128> {
        let acct = *key.first()? as u64;
        if self.ledger.is_empty() {
            return None;
        }
        // Epochs in this ledger are **zero-based**: `head()` is `len() - 1`, so epoch 0 is a
        // real record and must not be treated as "nothing". Getting that wrong would have
        // made the ledger's first transaction invisible, which is the kind of off-by-one that
        // a conservation suite finds three months later.
        let head = self.ledger.head();
        let anchor = anchor.min(head);

        // An account the base has never posted to has no value, which is not zero. The
        // distinction is the absence lattice's, and this is the layer where it would be
        // quietest to lose: `Serving::read` returns an `Option`, and a fold over an empty
        // history has no value to return.
        if self.ledger.key_update_count(acct, anchor) == 0 {
            return None;
        }

        let value = if anchor < head {
            // **A historical read is an anchored reconstruction, not a cache hit.**
            //
            // `PartialView::read` treats a slot materialised at a *later* anchor as a hit for
            // an earlier one — it returns the value together with the anchor it is actually
            // true at, which is the bounded-staleness rung and is correct for a "no older
            // than X" read. It is not correct for "as of X", which is what a historical query
            // means and what a dispute asks.
            //
            // So an as-of read goes to the base: `reconstruct_balance` folds the account's
            // own entries up to `anchor` through the anchor index. That is exactly the
            // anchored upquery of thesis §3, spent where it is needed rather than avoided by
            // answering a different question.
            self.ledger.reconstruct_balance(acct, self.currency, anchor)
        } else {
            let (v, _at, _hit) =
                self.view.read(&mut self.ledger, acct, self.currency, anchor, 0.0, 0.0);
            v
        };
        Some(value)
    }

    fn views(&self) -> Vec<(String, u32)> {
        self.views.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Serving;

    fn engine() -> RevEngine {
        RevEngine::seeded(100, 3, 1_000, ViewMode::Demand, EvictionPolicy::Lru)
    }

    #[test]
    fn a_read_is_answered_by_the_partial_view_over_the_ledger() {
        let mut e = engine();
        // Epochs are zero-based, so 300 transfers put the head at 299.
        assert_eq!(e.frontier(), 299, "one epoch per seeded transfer, zero-based");
        let v = e.read("__wire_result", &[7], e.frontier()).expect("account 7 has postings");
        assert!(v > 0, "a real fold, not a placeholder: {v}");
        assert!(e.stats().reads > 0);
    }

    #[test]
    fn an_account_the_base_never_touched_has_no_value_rather_than_zero() {
        // The absence lattice reaching the wire. A server that answered zero here would make
        // "this account does not exist" indistinguishable from "this account has no money",
        // which is the §1.1.1 defect at the protocol boundary.
        let mut e = engine();
        assert_eq!(e.read("__wire_result", &[9_999], e.frontier()), None);
        assert!(e.read("__wire_result", &[1], e.frontier()).is_some());
    }

    #[test]
    fn a_read_at_a_past_anchor_answers_what_was_true_then() {
        let mut e = engine();
        let late = e.read("__wire_result", &[5], e.frontier()).unwrap();
        let early = e.read("__wire_result", &[5], 5).unwrap();
        assert!(early < late, "the past holds less: {early} vs {late}");

        // And it is stable: asking again gives the same answer, because the prefix cannot
        // change and the reconstruction is a pure function of (key, anchor).
        for _ in 0..20 {
            assert_eq!(e.read("__wire_result", &[5], 5), Some(early));
        }
    }

    #[test]
    fn eviction_forces_a_reconstruction_and_the_answer_is_unchanged() {
        // The property Contribution 1 is about, exercised through the wire path: eviction
        // followed by reconstruction can neither create nor destroy money.
        let mut e = engine();
        let warm: Vec<Option<i128>> =
            (1..=20).map(|a| e.read("__wire_result", &[a], e.frontier())).collect();
        let hits_before = e.stats().hits;

        e.evict_all();
        assert_eq!(e.stats().resident, 0);

        let cold: Vec<Option<i128>> =
            (1..=20).map(|a| e.read("__wire_result", &[a], e.frontier())).collect();
        assert_eq!(warm, cold, "eviction changed an answer");
        assert!(e.stats().misses > 0, "and the cold reads really did reconstruct");
        assert!(e.stats().rows_touched > 0, "touching base rows to do it");
        assert!(e.stats().hits >= hits_before);
    }

    #[test]
    fn the_miss_rate_is_reportable_because_a_parity_result_depends_on_it() {
        // A parity result at a 0% miss rate and one at a 40% miss rate are different
        // findings. A server that reported only latency would let the more interesting of
        // the two disappear.
        let mut e = engine();
        let head = e.frontier();
        let pass = |e: &mut RevEngine| {
            for a in 1..=50 {
                e.read("__wire_result", &[a], head);
            }
        };

        // Pass one is cold: in demand mode a key is materialised when it is first read, so
        // every one of these misses and reconstructs.
        pass(&mut e);
        let cold = e.stats();
        assert_eq!(cold.misses, 50, "every first touch is a miss");
        assert!((cold.miss_rate() - 1.0).abs() < 1e-9);

        // Pass two is warm: the same keys, now resident, so the cumulative rate halves.
        pass(&mut e);
        let warm = e.stats();
        assert_eq!(warm.misses, 50, "no new misses");
        assert_eq!(warm.hits, 50, "and fifty hits");
        assert!((warm.miss_rate() - 0.5).abs() < 1e-9, "{}", warm.miss_rate());

        // Eviction puts it back: the third pass reconstructs everything again.
        e.evict_all();
        pass(&mut e);
        let after = e.stats();
        assert!(after.miss_rate() > warm.miss_rate(), "eviction raises the miss rate");
        assert_eq!(after.misses, 100);
        assert!(after.rows_touched > cold.rows_touched, "and it paid for it in base rows");
        assert!((0.0..=1.0).contains(&after.miss_rate()));
    }

    #[test]
    fn an_anchor_beyond_the_head_is_answered_at_the_head_rather_than_invented() {
        let mut e = engine();
        let at_head = e.read("__wire_result", &[3], e.frontier());
        assert_eq!(e.read("__wire_result", &[3], u64::MAX), at_head);
    }

    #[test]
    fn an_empty_ledger_answers_nothing_rather_than_zero() {
        let mut e = RevEngine::seeded(0, 0, 10, ViewMode::Demand, EvictionPolicy::Lru);
        assert_eq!(e.read("__wire_result", &[1], 0), None);
    }

    #[test]
    fn epoch_zero_is_a_real_record_and_is_readable() {
        // Zero-based epochs, and an engine that treated 0 as "nothing" would make the
        // ledger's first transaction invisible — an off-by-one a conservation suite finds
        // three months later.
        let mut e = RevEngine::seeded(3, 1, 100, ViewMode::Demand, EvictionPolicy::Lru);
        assert_eq!(e.frontier(), 2, "three transfers, epochs 0..=2");
        assert!(e.read("__wire_result", &[1], 0).is_some(), "account 1 posted at epoch 0");
        assert_eq!(e.read("__wire_result", &[2], 0), None, "account 2 has not yet");
        assert!(e.read("__wire_result", &[2], 1).is_some());
    }

    #[test]
    fn a_historical_read_reconstructs_rather_than_reusing_a_fresher_slot() {
        // `PartialView` treats a slot anchored later as a hit for an earlier read — correct
        // for a "no older than X" rung, wrong for "as of X", which is what a dispute asks.
        // Warming the view at the head must not change what a past anchor answers.
        let mut e = engine();
        let head = e.frontier();
        let past = 5;
        let cold = e.read("__wire_result", &[5], past).unwrap();

        for a in 1..=50 {
            e.read("__wire_result", &[a], head);
        }
        assert_eq!(
            e.read("__wire_result", &[5], past),
            Some(cold),
            "a warm view answered a historical query with a fresher value"
        );
        assert_ne!(e.read("__wire_result", &[5], head), Some(cold), "and the head differs");
    }
}
