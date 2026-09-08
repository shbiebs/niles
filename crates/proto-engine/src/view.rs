//! Derived read models: partial by default, with the absence lattice, anchored upqueries,
//! and eviction.

use std::collections::{BTreeMap, BTreeSet};

use crate::policy::EvictionPolicy;
use crate::{Acct, Cur, Epoch, Ledger, Minor};

/// The absence lattice (thesis §3.3), as a type so that a state the theory does not
/// contemplate cannot be represented.
///
/// `Hole` retains the *version* an entry held when it was evicted — "honest absence".
/// It is what lets a reader tell "evicted" from "logically zero", which is the single most
/// important safety property on the read path: a missing balance must never read as zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Never computed. The runtime asserts nothing.
    Bottom,
    /// Evicted: value gone, version remembered.
    Hole(Epoch),
    /// Certified: equals the ideal view at this anchor.
    Present(Minor, Epoch),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Materialize everything ever touched; never evict.
    Full,
    /// Materialize on read; evict under a budget.
    Demand,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ViewStats {
    pub reads: u64,
    pub hits: u64,
    pub misses: u64,
    pub upqueries: u64,
    /// Base rows read by upqueries (the reconstruction cost unit).
    pub rows_touched: u64,
    /// Per-key delta applications performed by maintenance.
    pub deltas_applied: u64,
    /// Deltas discarded because the key was not resident (the saving partiality buys).
    pub deltas_skipped: u64,
    pub evictions: u64,
    /// Peak number of resident entries.
    pub peak_resident: u64,
    /// Accumulated resident-entry-epochs: the integral of residency over time. This is the
    /// honest memory unit — charging peak residency for the whole run would overstate the
    /// cost of a strategy whose footprint grows gradually, which is exactly what full
    /// materialization does.
    pub resident_entry_epochs: u64,
    /// Sum over reads of the number of requests that would queue during a reconstruction,
    /// i.e. the delayed-hit term. Reported only when a service time is configured.
    pub aggregate_delay: f64,
}

/// A per-(account, currency) balance view.
pub struct PartialView {
    pub mode: ViewMode,
    pub budget: usize,
    pub policy: EvictionPolicy,
    slots: BTreeMap<(Acct, Cur), Slot>,
    /// Bookkeeping for the eviction policy.
    meta: BTreeMap<(Acct, Cur), policy_meta::Meta>,
    pub applied: Epoch,
    /// Whether any epoch has been folded into this view yet.
    ///
    /// `applied: Epoch` alone cannot say so: epochs are numbered from zero here, so a
    /// fresh view and a view that has applied epoch 0 both read `applied == 0`. Folding
    /// "everything after `applied`" without this flag silently skips epoch 0 — the first
    /// epoch of the ledger, and the one that funds every account.
    applied_any: bool,
    /// Keys whose entry was installed at an anchor below `applied` — a historical read.
    /// Such an entry has not seen the deltas between its anchor and `applied`, so it may
    /// serve only reads at or below its own stamp and must receive no delta.
    pinned: BTreeSet<(Acct, Cur)>,
    pub stats: ViewStats,
    clock: u64,
    /// Maintained incrementally; counting residents on every budget check would make
    /// eviction quadratic and would measure the harness rather than the design.
    resident_count: usize,
}

pub mod policy_meta {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Meta {
        pub last_used: u64,
        pub uses: u64,
        /// Credit for the cost-aware policy: reconstruction cost, delay-weighted.
        pub credit: f64,
        /// Last measured reconstruction cost in base rows.
        pub recon_cost: f64,
    }
}

impl PartialView {
    pub fn new(mode: ViewMode, budget: usize, policy: EvictionPolicy) -> Self {
        Self {
            mode,
            budget,
            policy,
            slots: BTreeMap::new(),
            meta: BTreeMap::new(),
            applied: 0,
            applied_any: false,
            pinned: BTreeSet::new(),
            stats: ViewStats::default(),
            clock: 0,
            resident_count: 0,
        }
    }

    pub fn resident(&self) -> usize {
        self.resident_count
    }

    /// Bytes of resident *logical* state: the key and value words actually held.
    ///
    /// This is layer 1 of the three-layer memory-accounting discipline (logical bytes /
    /// allocator resident / process RSS). It is reported as logical bytes and never as RSS,
    /// because conflating the two would credit the design with savings that belong to the
    /// allocator — or blame it for overhead that does not.
    pub fn resident_logical_bytes(&self) -> usize {
        const ENTRY: usize = std::mem::size_of::<(Acct, Cur)>() + std::mem::size_of::<Slot>();
        self.resident() * ENTRY
    }

    /// Apply one sealed epoch. Only *resident* keys are advanced — a delta addressed to an
    /// evicted key is discarded with no downstream notification, because that key's future
    /// value will be reconstructed from the base rather than from the delta stream. This is
    /// the maintenance saving that partiality buys, and `deltas_skipped` counts it.
    pub fn apply_epoch(&mut self, ledger: &Ledger, e: Epoch) {
        let rec = &ledger.epochs[e as usize];
        for row in &rec.rows {
            if let crate::Row::Post(p) = row {
                let k = (p.acct, p.cur);
                if self.pinned.contains(&k) {
                    self.stats.deltas_skipped += 1;
                    continue;
                }
                match self.slots.get(&k) {
                    // Already included: an entry reconstructed ahead of the applied
                    // frontier carries this delta, and adding it again is the
                    // double-application anomaly.
                    Some(Slot::Present(_, stamp)) if *stamp >= e => {
                        self.stats.deltas_skipped += 1;
                    }
                    Some(Slot::Present(v, _)) => {
                        let nv = *v + p.amt;
                        self.slots.insert(k, Slot::Present(nv, e));
                        self.stats.deltas_applied += 1;
                    }
                    _ => {
                        self.stats.deltas_skipped += 1;
                    }
                }
            }
        }
        // Entries that received no delta are not rewritten: a resident entry is certified
        // through the view's applied frontier by construction, because it has been resident
        // for every epoch since its own anchor and every such epoch was applied to it. The
        // effective anchor is therefore derived on read (`effective_anchor`), which keeps
        // maintenance proportional to the number of deltas rather than to residency.
        self.stats.resident_entry_epochs += self.resident_count as u64;
        self.applied = e;
        self.applied_any = true;
    }

    /// Fold every epoch not yet applied, through `e`, in order.
    ///
    /// This is what a maintenance pass is. A bounded-staleness rung runs it less often
    /// than a strict one and therefore does fewer, larger passes — but it folds the same
    /// deltas, because the epochs it batched over still have to be applied. Applying only
    /// the boundary epoch leaves the view certified through a frontier whose deltas
    /// nobody folded, which is a wrong answer rather than a stale one.
    pub fn apply_through(&mut self, ledger: &Ledger, e: Epoch) {
        let from = if self.applied_any {
            self.applied + 1
        } else {
            0
        };
        for epoch in from..=e {
            self.apply_epoch(ledger, epoch);
        }
    }

    /// Read at an anchor. Returns `(value, anchor, was_hit)`.
    ///
    /// A miss triggers an upquery anchored at the requested epoch. Because the base is
    /// immutable, the reconstruction is a pure function of (key, anchor) and cannot race
    /// with maintenance.
    pub fn read(
        &mut self,
        ledger: &mut Ledger,
        acct: Acct,
        cur: Cur,
        anchor: Epoch,
        service_time: f64,
        arrivals_during_fill: f64,
    ) -> (Minor, Epoch, bool) {
        self.clock += 1;
        self.stats.reads += 1;
        let k = (acct, cur);

        if let Some(Slot::Present(v, a)) = self.slots.get(&k).copied() {
            let a = if self.pinned.contains(&k) {
                a
            } else {
                a.max(self.applied)
            };
            if a >= anchor {
                self.stats.hits += 1;
                let m = self.meta.entry(k).or_default();
                m.last_used = self.clock;
                m.uses += 1;
                // Refresh credit on hit (the "landlord" reset).
                m.credit = m.credit.max(m.recon_cost);
                return (v, a, true);
            }
        }

        // Miss: reconstruct at the requested anchor.
        self.stats.misses += 1;
        self.stats.upqueries += 1;
        // This fold's own visited count, from the fold, rather than a difference of the
        // ledger's process-global counter across the call (A9-F18).
        let (v, visited) = ledger.reconstruct_balance_counted(acct, cur, anchor);
        let cost = visited as f64;
        self.stats.rows_touched += visited;

        // Delayed hits: while a reconstruction is in flight, further requests for the same
        // key queue behind it. The aggregate-delay objective charges for those, which is
        // why a policy that ranks purely by reuse probability is the wrong policy here.
        if service_time > 0.0 {
            self.stats.aggregate_delay += service_time * (1.0 + arrivals_during_fill);
        }

        if !matches!(self.slots.get(&k), Some(Slot::Present(_, _))) {
            self.resident_count += 1;
        }
        // A reconstruction anchored below the applied frontier is a historical answer:
        // worth keeping, but not current, and never promoted to `applied`.
        if anchor < self.applied {
            self.pinned.insert(k);
        } else {
            self.pinned.remove(&k);
        }
        self.slots.insert(k, Slot::Present(v, anchor));
        {
            let m = self.meta.entry(k).or_default();
            m.last_used = self.clock;
            m.uses += 1;
            m.recon_cost = cost;
            m.credit = cost * (1.0 + arrivals_during_fill);
        }

        self.enforce_budget();
        let r = self.resident() as u64;
        if r > self.stats.peak_resident {
            self.stats.peak_resident = r;
        }
        (v, anchor, false)
    }

    fn enforce_budget(&mut self) {
        if self.mode == ViewMode::Full {
            return;
        }
        while self.resident() > self.budget {
            let victim = self
                .policy
                .choose_victim(&self.slots, &self.meta, self.clock);
            match victim {
                Some(k) => {
                    // Honest absence: Present(v, e) becomes Hole(e). The value is dropped;
                    // the version is kept, so absence of value never masquerades as absence
                    // of history.
                    if let Some(Slot::Present(_, a)) = self.slots.get(&k).copied() {
                        let version = if self.pinned.remove(&k) {
                            a
                        } else {
                            a.max(self.applied)
                        };
                        self.slots.insert(k, Slot::Hole(version));
                        self.resident_count -= 1;
                        self.stats.evictions += 1;
                        // Decay all credits (budget pressure), per the cost-aware policy.
                        if let EvictionPolicy::CostAware = self.policy {
                            let dec = self.meta.get(&k).map(|m| m.credit).unwrap_or(0.0);
                            for m in self.meta.values_mut() {
                                m.credit = (m.credit - dec).max(0.0);
                            }
                        }
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
    }

    /// Wipe the entire derived layer (the rebuild-from-base experiment).
    pub fn wipe(&mut self) {
        self.pinned.clear();
        self.slots.clear();
        self.meta.clear();
        self.resident_count = 0;
    }

    pub fn slot(&self, acct: Acct, cur: Cur) -> Slot {
        self.slots
            .get(&(acct, cur))
            .copied()
            .unwrap_or(Slot::Bottom)
    }

    /// Raw counters, for cost models applied after the fact.
    pub fn counters(&self) -> ViewStats {
        self.stats
    }

    /// Total work charged so far under a cost model, for the phase diagram.
    pub fn total_cost(&self, cm: &crate::CostModel) -> f64 {
        cm.memory * (self.stats.resident_entry_epochs as f64)
            + cm.maintenance * (self.stats.deltas_applied as f64)
            + cm.reconstruction * (self.stats.rows_touched as f64)
    }
}

#[cfg(test)]
mod cert_tests {
    //! The certification invariant, and the three defects that violated it.
    //!
    //! `proto-engine` is the instrument that produced every counted-work figure in the
    //! evaluation chapter, and until this module it had no tests at all. A measurement
    //! whose instrument is unchecked is a measurement of the instrument.

    use super::*;
    use crate::ledger::{Ledger, Posting, Reject, Row};

    const USD: Cur = 840;

    fn seal(l: &mut Ledger, key: &str, acct: Acct, amt: Minor) {
        // Balanced against a contra account, because the commit rule is per (txn, currency).
        l.submit(
            key,
            vec![
                Row::Post(Posting {
                    txn: 1,
                    acct,
                    cur: USD,
                    amt,
                    valid: 0,
                }),
                Row::Post(Posting {
                    txn: 1,
                    acct: 999,
                    cur: USD,
                    amt: -amt,
                    valid: 0,
                }),
            ],
        )
        .expect("balanced");
    }

    /// An independent fold: the rows, filtered and summed in the test. Deliberately not
    /// `Ledger::reconstruct_balance` — comparing a value against the function that
    /// produced it is what made Table 9.1's divergence column an identity.
    fn truth(l: &Ledger, acct: Acct, cur: Cur, anchor: Epoch) -> Minor {
        let mut total = 0;
        for (i, e) in l.epochs.iter().enumerate() {
            if i as Epoch > anchor {
                break;
            }
            for r in &e.rows {
                if let Row::Post(p) = r {
                    if p.acct == acct && p.cur == cur {
                        total += p.amt;
                    }
                }
            }
        }
        total
    }

    #[test]
    fn f01_stale_hit_is_not_promoted() {
        let mut l = Ledger::new();
        seal(&mut l, "a", 1, 10);
        seal(&mut l, "b", 1, 20);
        let mut v = PartialView::new(ViewMode::Demand, 8, EvictionPolicy::Lru);
        v.apply_epoch(&l, 1);

        let (val, anch, _) = v.read(&mut l, 1, USD, 0, 0.0, 0.0);
        assert_eq!(val, truth(&l, 1, USD, anch));

        let (val, anch, _) = v.read(&mut l, 1, USD, 1, 0.0, 0.0);
        assert_eq!(anch, 1);
        assert_eq!(
            val,
            truth(&l, 1, USD, 1),
            "a read at the applied frontier must not be answered from an older entry"
        );
    }

    #[test]
    fn f02_delta_is_applied_once() {
        let mut l = Ledger::new();
        seal(&mut l, "a", 1, 10);
        seal(&mut l, "b", 1, 20);
        let mut v = PartialView::new(ViewMode::Demand, 8, EvictionPolicy::Lru);
        // Reconstruct ahead of the applied frontier: the entry already carries both epochs.
        let (ahead, _, _) = v.read(&mut l, 1, USD, 1, 0.0, 0.0);
        assert_eq!(ahead, truth(&l, 1, USD, 1));
        v.apply_epoch(&l, 0);
        v.apply_epoch(&l, 1);
        let (after, anch, _) = v.read(&mut l, 1, USD, 1, 0.0, 0.0);
        assert_eq!(
            after,
            truth(&l, 1, USD, anch),
            "folding a delta the value already carries doubles the money"
        );
    }

    #[test]
    fn cert_holds_under_interleaved_reads_evictions_and_maintenance() {
        for seed in [1u64, 7, 42, 100, 2024] {
            let mut lcg = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let mut next = || {
                lcg = lcg
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                lcg >> 33
            };
            let mut l = Ledger::new();
            for i in 0..100u64 {
                seal(&mut l, &format!("s{seed}-{i}"), i % 6, 1 + (i % 4) as Minor);
            }
            // A budget that binds, so eviction runs continuously.
            let mut v = PartialView::new(ViewMode::Demand, 3, EvictionPolicy::Lru);
            let mut checked = 0;
            for e in 0..100u64 {
                v.apply_epoch(&l, e);
                for _ in 0..3 {
                    let acct = next() % 6;
                    let anchor = next() % (e + 1);
                    let (val, anch, _) = v.read(&mut l, acct, USD, anchor, 0.0, 0.0);
                    assert!(anch >= anchor);
                    assert_eq!(
                        val,
                        truth(&l, acct, USD, anch),
                        "seed {seed}: served value disagrees with the fold at its own anchor"
                    );
                    checked += 1;
                }
            }
            assert!(checked >= 300);
        }
    }

    #[test]
    fn eviction_is_reproducible_from_the_seed_for_every_policy() {
        for policy in [
            EvictionPolicy::Lru,
            EvictionPolicy::Random,
            EvictionPolicy::CostAware,
        ] {
            let run = || {
                let mut l = Ledger::new();
                for i in 0..60u64 {
                    seal(&mut l, &format!("k{i}"), i % 8, 1);
                }
                let mut v = PartialView::new(ViewMode::Demand, 3, policy);
                for e in 0..60u64 {
                    v.apply_epoch(&l, e);
                    v.read(&mut l, (e * 7) % 8, USD, e, 1.0, 1.0);
                }
                let resident: Vec<(Acct, Cur)> = v
                    .slots
                    .iter()
                    .filter(|(_, s)| matches!(s, Slot::Present(_, _)))
                    .map(|(k, _)| *k)
                    .collect();
                (v.stats.rows_touched, v.stats.evictions, resident)
            };
            let (a, b, c) = (run(), run(), run());
            assert_eq!(a, b, "{policy:?} is not reproducible from its seed");
            assert_eq!(b, c, "{policy:?} is not reproducible from its seed");
        }
    }

    #[test]
    fn the_first_maintenance_pass_folds_epoch_zero() {
        // Epochs are numbered from zero here, so a fresh view and a view that has applied
        // epoch 0 both read `applied == 0`. Folding "everything after applied" without
        // distinguishing them skips the ledger's first epoch — the one that funds the
        // accounts — and every balance is short by its opening entry.
        let mut l = Ledger::new();
        seal(&mut l, "open", 1, 100);
        seal(&mut l, "more", 1, 5);
        let mut v = PartialView::new(ViewMode::Demand, 8, EvictionPolicy::Lru);
        v.read(&mut l, 1, USD, 0, 0.0, 0.0); // resident, anchored at epoch 0
        let head = l.head();
        v.apply_through(&l, head);
        let (val, anch, hit) = v.read(&mut l, 1, USD, head, 0.0, 0.0);
        assert!(hit, "the entry was resident and maintained");
        assert_eq!(val, truth(&l, 1, USD, anch));
        assert_eq!(val, 105);
    }

    #[test]
    fn a_batched_pass_folds_the_same_deltas_as_an_unbatched_one() {
        // The corrected statement of what a bounded rung buys: fewer passes, identical
        // deltas. If batching applied fewer deltas the view would be serving a value the
        // ledger does not have.
        let build = || {
            let mut l = Ledger::new();
            for i in 0..40u64 {
                seal(&mut l, &format!("k{i}"), i % 4, 1);
            }
            l
        };
        let mut l1 = build();
        let mut strict = PartialView::new(ViewMode::Demand, 8, EvictionPolicy::Lru);
        for k in 0..4 {
            strict.read(&mut l1, k, USD, 0, 0.0, 0.0);
        }
        for e in 0..=l1.head() {
            strict.apply_through(&l1, e);
        }

        let mut l2 = build();
        let mut lax = PartialView::new(ViewMode::Demand, 8, EvictionPolicy::Lru);
        for k in 0..4 {
            lax.read(&mut l2, k, USD, 0, 0.0, 0.0);
        }
        let mut e = 0;
        while e <= l2.head() {
            lax.apply_through(&l2, e.min(l2.head()));
            e += 8;
        }
        lax.apply_through(&l2, l2.head());

        assert_eq!(
            lax.stats.deltas_applied, strict.stats.deltas_applied,
            "batching must not lose deltas"
        );
        let (h1, h2) = (l1.head(), l2.head());
        for k in 0..4 {
            let (a, _, _) = strict.read(&mut l1, k, USD, h1, 0.0, 0.0);
            let (b, _, _) = lax.read(&mut l2, k, USD, h2, 0.0, 0.0);
            assert_eq!(a, b);
            assert_eq!(a, truth(&l1, k, USD, h1));
        }
    }

    #[test]
    fn overflow_is_refused() {
        // Conservation is stated over the integers. In release builds Rust's arithmetic
        // wraps, so a set summing to zero only by wraparound would pass the commit rule
        // and create money out of a machine word.
        let mut l = Ledger::new();
        let r = l.submit(
            "wrap",
            vec![
                Row::Post(Posting {
                    txn: 1,
                    acct: 1,
                    cur: USD,
                    amt: Minor::MAX,
                    valid: 0,
                }),
                Row::Post(Posting {
                    txn: 1,
                    acct: 2,
                    cur: USD,
                    amt: Minor::MAX,
                    valid: 0,
                }),
                Row::Post(Posting {
                    txn: 1,
                    acct: 3,
                    cur: USD,
                    amt: 2,
                    valid: 0,
                }),
            ],
        );
        assert_eq!(r, Err(Reject::Overflow));
    }
}
