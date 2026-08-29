//! Derived read models: partial by default, with the absence lattice, anchored upqueries,
//! and eviction.

use std::collections::HashMap;

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
    slots: HashMap<(Acct, Cur), Slot>,
    /// Bookkeeping for the eviction policy.
    meta: HashMap<(Acct, Cur), policy_meta::Meta>,
    pub applied: Epoch,
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
            slots: HashMap::new(),
            meta: HashMap::new(),
            applied: 0,
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
                match self.slots.get(&k) {
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
            let a = a.max(self.applied);
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
        let before = ledger.rows_touched;
        let v = ledger.reconstruct_balance(acct, cur, anchor);
        let cost = (ledger.rows_touched - before) as f64;
        self.stats.rows_touched += ledger.rows_touched - before;

        // Delayed hits: while a reconstruction is in flight, further requests for the same
        // key queue behind it. The aggregate-delay objective charges for those, which is
        // why a policy that ranks purely by reuse probability is the wrong policy here.
        if service_time > 0.0 {
            self.stats.aggregate_delay += service_time * (1.0 + arrivals_during_fill);
        }

        if !matches!(self.slots.get(&k), Some(Slot::Present(_, _))) {
            self.resident_count += 1;
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
            let victim = self.policy.choose_victim(&self.slots, &self.meta, self.clock);
            match victim {
                Some(k) => {
                    // Honest absence: Present(v, e) becomes Hole(e). The value is dropped;
                    // the version is kept, so absence of value never masquerades as absence
                    // of history.
                    if let Some(Slot::Present(_, a)) = self.slots.get(&k).copied() {
                        self.slots.insert(k, Slot::Hole(a.max(self.applied)));
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
        self.slots.clear();
        self.meta.clear();
        self.resident_count = 0;
    }

    pub fn slot(&self, acct: Acct, cur: Cur) -> Slot {
        self.slots.get(&(acct, cur)).copied().unwrap_or(Slot::Bottom)
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
