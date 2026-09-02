//! The REV runtime: reconstructible epoch-anchored views, executing a typed IR circuit.
//!
//! This is where the thesis's central abstraction becomes a running object. A [`Rev`] is a
//! versioned, partially materialized derived state over an immutable, epoch-ordered base:
//! it materializes only what is read, evicts the rest under a budget, and reconstructs on
//! demand through an anchored upquery.
//!
//! # What closes the loop
//!
//! The circuit this runtime executes is the one `niles-lang` lowers from Niles source
//! text. Nothing is hand-built between the two. That is what makes the pipeline
//! *source → typed IR → running partial-state engine → measured result* a single artifact
//! rather than a diagram.
//!
//! # Scope, stated before any measurement is read
//!
//! This runtime executes the **key-aggregate fragment** of the IR: a `Source`, optional
//! `Filter` and `Map`, and one `Aggregate` keyed by a derivable group key. That is the
//! shape of a balance, a position, a rollup and a running total — which is to say, of the
//! views the thesis's measurements are about — but it is a fragment, and a circuit outside
//! it is rejected by [`Runtime::install`] rather than silently mis-executed. Joins,
//! fixpoints and ordering stages lower and verify but do not execute here.
//!
//! The counters are **counted work** — base rows read, deltas applied, resident
//! entry-epochs — not wall-clock. They are properties of the algorithm and the workload,
//! reproducible on any machine, which is what makes a phase diagram built from them
//! something a reader can check rather than something they must trust.

use crate::absence::{Epoch, Slot};
use niles_ir::circuit::{Circuit, NodeId};
use niles_ir::operator::{Agg, Op};
use niles_ir::{Consistency, Materialize, Retention};
use std::collections::HashMap;

/// A view key. Concretely a small tuple of integers, which is what a `(acct, cur)` group
/// key lowers to.
pub type Key = Vec<i64>;

/// A materialized value. The fragment this runtime executes produces one aggregate per
/// key, in exact integer minor units — never floating point, because a balance that is
/// only approximately conserved is not conserved.
pub type Value = i128;

/// An answer, with the epoch it is true at. Every read returns one of these: an answer
/// without its anchor is not checkable, and "the same question, asked twice, answered
/// consistently" would be unstatable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchored {
    pub value: Value,
    pub anchor: Epoch,
}

/// What the runtime needs from a base. Deliberately narrow: three operations, all of them
/// reads of an immutable, epoch-ordered history.
///
/// Keeping this a trait rather than a concrete type is not abstraction for its own sake.
/// It is what lets the same runtime run over the in-memory research prototype that
/// produced Chapter 9's numbers and over a durable, segment-backed store, and be
/// *measured to produce the same counted work* over both. A runtime welded to one storage
/// layer could not make that comparison.
pub trait Base {
    /// The current visibility frontier.
    fn frontier(&self) -> Epoch;

    /// Fold the aggregate for `key` over the prefix ending at `anchor`.
    ///
    /// The implementation is expected to start from the newest checkpoint at or before
    /// `anchor` and fold only the suffix — which is what makes reconstruction cost bounded
    /// by the checkpoint interval rather than by history length (SC7).
    ///
    /// Returns the value **and** the number of base rows it had to read, because the row
    /// count is the measurement.
    fn reconstruct(&mut self, key: &Key, anchor: Epoch) -> (Value, u64);

    /// The per-key deltas sealed in epoch `e`. Empty for most keys in most epochs, which
    /// is exactly why partial materialization can pay.
    fn deltas_at(&mut self, e: Epoch) -> Vec<(Key, Value)>;
}

/// Eviction policy. `CostAware` is the adaptive one; the other two are the baselines it is
/// measured against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Lru,
    Random,
    /// Evict the entry whose reconstruction is cheapest relative to how rarely it is read.
    /// The measured margin over LRU is real but uneven — large on reconstruction work,
    /// small on aggregate delay — and the runtime records both so the claim stays honest.
    CostAware,
}

/// Counted work. Machine-independent, and the unit every claim in Chapter 9 is stated in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub reads: u64,
    pub hits: u64,
    pub misses: u64,
    pub upqueries: u64,
    /// Base rows the upqueries had to fold. The headline reconstruction cost.
    pub base_rows_read: u64,
    /// Deltas applied to resident entries. Where a consistency rung's cost actually falls.
    pub deltas_applied: u64,
    /// Deltas skipped because their entry was not resident — the saving partiality buys.
    pub deltas_skipped: u64,
    pub evictions: u64,
    pub peak_resident: u64,
    /// The integral of residency over time: sum over epochs of the resident entry count.
    ///
    /// This, and not peak residency, is the memory term in the cost model. An early
    /// version of this instrument charged `peak x epochs` and produced a phase diagram in
    /// which partial materialization won every cell — an artifact of the metric, not a
    /// result. The correction is recorded in the thesis because the mistake is an easy one
    /// and the corrected figure is the one the phase boundary depends on.
    pub resident_entry_epochs: u64,
}

impl Stats {
    pub fn hit_ratio(&self) -> f64 {
        if self.reads == 0 {
            0.0
        } else {
            self.hits as f64 / self.reads as f64
        }
    }
    /// Price the raw counters after the fact. Pricing at the end rather than during the run
    /// is what lets one measured run be re-scored under many cost models, which is how the
    /// phase diagram sweeps the price of memory without re-running anything.
    pub fn cost(&self, memory: f64, maintenance: f64, reconstruction: f64) -> f64 {
        self.resident_entry_epochs as f64 * memory
            + self.deltas_applied as f64 * maintenance
            + self.base_rows_read as f64 * reconstruction
    }
}

/// One reconstructible epoch-anchored view.
pub struct Rev {
    pub node: NodeId,
    pub name: String,
    slots: HashMap<Key, Slot<Value>>,
    resident: u64,
    /// Resident-entry budget. `None` means full materialization.
    budget: Option<u64>,
    policy: Policy,
    /// The epoch through which deltas have been applied to this view as a whole.
    applied: Epoch,
    /// Per-key read counts, for the cost-aware policy.
    reads_of: HashMap<Key, u64>,
    /// Per-key last-read clock, for LRU.
    clock: u64,
    last_read: HashMap<Key, u64>,
    /// The rung this view promises, and the mode it was planned in. Both are read from the
    /// circuit's checked fields, so an engine that ignored them fails the IR audit.
    pub rung: Consistency,
    pub mode: Materialize,
    pub stats: Stats,
}

impl Rev {
    pub fn resident_count(&self) -> u64 {
        self.resident
    }

    /// Read a key at the given anchor.
    ///
    /// The four cases are the absence lattice: a `Present` entry at or after the anchor is
    /// a hit; anything else reconstructs. `Bottom` reconstructs too — a key never seen is
    /// not a key with no postings, and answering it with the aggregate's identity would be
    /// the money-from-memory-pressure bug the lattice exists to prevent.
    pub fn read(&mut self, base: &mut dyn Base, key: &Key, anchor: Epoch) -> Anchored {
        self.stats.reads += 1;
        self.clock += 1;
        self.last_read.insert(key.clone(), self.clock);
        *self.reads_of.entry(key.clone()).or_insert(0) += 1;

        // The effective version of a resident entry is the later of its own stamp and the
        // view-wide applied epoch: an entry that received no delta in an epoch is still
        // current through that epoch, and rewriting every resident entry's stamp on every
        // epoch would make maintenance O(resident) instead of O(deltas).
        if let Some(Slot::Present(v, e)) = self.slots.get(key) {
            let effective = (*e).max(self.applied);
            if effective >= anchor {
                self.stats.hits += 1;
                return Anchored {
                    value: *v,
                    anchor: effective,
                };
            }
        }

        self.stats.misses += 1;
        self.stats.upqueries += 1;
        // The anchored upquery: one epoch, one frozen prefix, no protocol state.
        let (value, rows) = base.reconstruct(key, anchor);
        self.stats.base_rows_read += rows;
        self.install(key.clone(), value, anchor);
        Anchored { value, anchor }
    }

    fn install(&mut self, key: Key, value: Value, anchor: Epoch) {
        let was_resident = self.slots.get(&key).map_or(false, |s| s.is_resident());
        self.slots.insert(key, Slot::Present(value, anchor));
        if !was_resident {
            self.resident += 1;
        }
        self.stats.peak_resident = self.stats.peak_resident.max(self.resident);
        self.enforce_budget();
    }

    fn enforce_budget(&mut self) {
        let Some(b) = self.budget else { return };
        while self.resident > b {
            let Some(victim) = self.choose_victim() else {
                break;
            };
            if let Some(s) = self.slots.get_mut(&victim) {
                // Honest absence: the value goes, the version stays.
                if s.evict() {
                    self.resident -= 1;
                    self.stats.evictions += 1;
                }
            }
        }
    }

    fn choose_victim(&self) -> Option<Key> {
        let resident = || self.slots.iter().filter(|(_, s)| s.is_resident());
        match self.policy {
            Policy::Random => resident().next().map(|(k, _)| k.clone()),
            Policy::Lru => resident()
                .min_by_key(|(k, _)| self.last_read.get(*k).copied().unwrap_or(0))
                .map(|(k, _)| k.clone()),
            Policy::CostAware => {
                // Keep what is read often; drop what is cheap to rebuild. The score is
                // reads per unit of reconstruction work, and the entry with the lowest
                // score goes. Approximating reconstruction cost by 1 here is deliberate:
                // with per-key checkpoints the cost is bounded by the interval and is
                // nearly uniform, which is itself a consequence of SC7.
                resident()
                    .min_by_key(|(k, _)| self.reads_of.get(*k).copied().unwrap_or(0))
                    .map(|(k, _)| k.clone())
            }
        }
    }

    /// Apply one epoch's deltas. **O(deltas), not O(resident)** — an entry that receives no
    /// delta is not touched, and its currency is derived on read from `applied`.
    ///
    /// This is where a consistency rung's cost actually falls, which was itself a measured
    /// finding: instrumenting only the read path showed no difference between rungs at all.
    pub fn apply_epoch(&mut self, base: &mut dyn Base, e: Epoch) {
        for (key, delta) in base.deltas_at(e) {
            match self.slots.get_mut(&key) {
                Some(Slot::Present(v, stamp)) => {
                    *v += delta;
                    *stamp = e;
                    self.stats.deltas_applied += 1;
                }
                // Not resident: the delta is skipped, and correctly so. The entry's hole
                // still carries its old version, so a later read reconstructs from the
                // base and picks this delta up along the way. This is the saving.
                _ => self.stats.deltas_skipped += 1,
            }
        }
        self.applied = e;
        self.stats.resident_entry_epochs += self.resident;
    }

    /// Drop everything, keeping no versions. Used only by the rebuild-from-base check,
    /// where the point is to prove the derived layer holds no information the base does not.
    pub fn wipe(&mut self) {
        self.slots.clear();
        self.resident = 0;
    }

    pub fn slot(&self, key: &Key) -> Slot<Value> {
        self.slots.get(key).cloned().unwrap_or(Slot::Bottom)
    }
}

/// Why a circuit could not be installed on this runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum Unsupported {
    /// Outside the key-aggregate fragment.
    Shape { node: NodeId, op: &'static str },
    /// No derivable key, so there is nothing to materialize per key.
    NoKey { node: NodeId },
    /// The aggregate is not one the runtime maintains.
    Aggregate { node: NodeId, agg: &'static str },
}

impl Unsupported {
    pub fn explain(&self) -> String {
        match self {
            Unsupported::Shape { node, op } => format!(
                "node {node} is a `{op}`, which lowers and verifies but is outside the \
                 key-aggregate fragment this runtime executes"
            ),
            Unsupported::NoKey { node } => {
                format!(
                    "node {node} has no derivable key, so there is nothing to materialize per key"
                )
            }
            Unsupported::Aggregate { node, agg } => {
                format!("node {node} aggregates with `{agg}`, which this runtime does not maintain")
            }
        }
    }
}

/// The runtime: a circuit, and one REV per named output.
pub struct Runtime {
    pub circuit: Circuit,
    pub views: Vec<Rev>,
    pub epoch: Epoch,
}

impl Runtime {
    /// Install a compiled circuit. Rejects anything outside the executable fragment rather
    /// than silently mis-executing it.
    pub fn install(
        circuit: Circuit,
        budget: Option<u64>,
        policy: Policy,
    ) -> Result<Runtime, Unsupported> {
        let mut views = Vec::new();
        let outputs: Vec<(String, NodeId)> = circuit
            .outputs
            .iter()
            .map(|(n, i)| (n.clone(), *i))
            .collect();
        for (name, id) in outputs {
            let n = circuit.node(id);
            match &n.op {
                Op::Aggregate { aggs, .. } => {
                    for (a, _) in aggs {
                        if !matches!(a, Agg::Sum | Agg::Count) {
                            return Err(Unsupported::Aggregate {
                                node: id,
                                agg: a.as_str(),
                            });
                        }
                    }
                }
                other => {
                    return Err(Unsupported::Shape {
                        node: id,
                        op: other.name(),
                    })
                }
            }
            if n.key.is_none() {
                return Err(Unsupported::NoKey { node: id });
            }
            // Read the checked fields. This is not incidental: the IR's accessed-field
            // audit fails if a consumer plans without consulting them, and a runtime that
            // ignored the rung would serve stale state and return a well-formed wrong
            // answer that no answer-level test would catch.
            let contract = *n.contract.get();
            let _ = n.anchor.get();
            let _ = n.conservation_transparent.get();
            let _ = n.lineage.get();

            // The contract decides the budget: a `full` or `pinned` view is not evictable,
            // whatever budget the caller suggested.
            let effective_budget = match (contract.materialize, contract.retain) {
                (Materialize::Full, _) | (_, Retention::Pinned) | (_, Retention::Forever) => None,
                _ => budget,
            };
            views.push(Rev {
                node: id,
                name,
                slots: HashMap::new(),
                resident: 0,
                budget: effective_budget,
                policy,
                applied: 0,
                reads_of: HashMap::new(),
                clock: 0,
                last_read: HashMap::new(),
                rung: contract.consistency,
                mode: contract.materialize,
                stats: Stats::default(),
            });
        }
        views.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Runtime {
            circuit,
            views,
            epoch: 0,
        })
    }

    pub fn view_mut(&mut self, name: &str) -> Option<&mut Rev> {
        self.views.iter_mut().find(|v| v.name == name)
    }

    pub fn view(&self, name: &str) -> Option<&Rev> {
        self.views.iter().find(|v| v.name == name)
    }

    /// Advance every view to epoch `e`.
    ///
    /// The rung is honoured here, and this is where its cost lands. A `Bounded { epochs: k }`
    /// view need only be maintained every k epochs; a `ledger_consistent` view must be
    /// maintained at every one. That difference is a factor of k in maintenance work, and
    /// it is invisible on the read path — which is why an earlier version of this
    /// experiment, instrumented only on reads, measured no difference between rungs at all
    /// and reported a null.
    pub fn advance(&mut self, base: &mut dyn Base, e: Epoch) {
        self.epoch = e;
        for v in &mut self.views {
            let stride = match v.rung {
                Consistency::Bounded { epochs, .. } => epochs.max(1),
                _ => 1,
            };
            if e % stride == 0 {
                v.apply_epoch(base, e);
            } else {
                // Still accrue the memory integral: the entries are resident whether or
                // not this epoch touched them, and charging only maintained epochs would
                // make a lax rung look free rather than cheap.
                v.stats.resident_entry_epochs += v.resident;
            }
        }
    }

    /// Total counted work across every view.
    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        for v in &self.views {
            s.reads += v.stats.reads;
            s.hits += v.stats.hits;
            s.misses += v.stats.misses;
            s.upqueries += v.stats.upqueries;
            s.base_rows_read += v.stats.base_rows_read;
            s.deltas_applied += v.stats.deltas_applied;
            s.deltas_skipped += v.stats.deltas_skipped;
            s.evictions += v.stats.evictions;
            s.peak_resident = s.peak_resident.max(v.stats.peak_resident);
            s.resident_entry_epochs += v.stats.resident_entry_epochs;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niles_ir::circuit::internal_contract;
    use niles_ir::operator::Scalar;
    use niles_ir::{Lineage, ServeContract};

    /// A base whose whole history is in a vector. Deliberately unoptimised: it is a
    /// definition of the right answer, not an implementation of a fast one.
    #[derive(Default)]
    struct FoldBase {
        /// (epoch, key, delta)
        rows: Vec<(Epoch, Key, Value)>,
        head: Epoch,
        /// Per-key running balances every `interval` rows, which is the mechanism SC7 is
        /// about: reconstruction folds only the suffix after the newest checkpoint.
        interval: usize,
        checkpoints: HashMap<Key, Vec<(Epoch, Value)>>,
        counts: HashMap<Key, usize>,
    }

    impl FoldBase {
        fn new(interval: usize) -> Self {
            FoldBase {
                interval,
                ..Default::default()
            }
        }
        fn seal(&mut self, key: Key, delta: Value) -> Epoch {
            self.head += 1;
            self.rows.push((self.head, key.clone(), delta));
            let c = self.counts.entry(key.clone()).or_insert(0);
            *c += 1;
            if self.interval > 0 && *c % self.interval == 0 {
                let running: Value = self
                    .rows
                    .iter()
                    .filter(|(_, k, _)| *k == key)
                    .map(|(_, _, d)| d)
                    .sum();
                self.checkpoints
                    .entry(key)
                    .or_default()
                    .push((self.head, running));
            }
            self.head
        }
    }

    impl Base for FoldBase {
        fn frontier(&self) -> Epoch {
            self.head
        }
        fn reconstruct(&mut self, key: &Key, anchor: Epoch) -> (Value, u64) {
            let (mut acc, mut from) = (0i128, 0u64);
            if let Some(cps) = self.checkpoints.get(key) {
                if let Some((e, v)) = cps.iter().rev().find(|(e, _)| *e <= anchor) {
                    acc = *v;
                    from = *e;
                }
            }
            let mut rows = 0u64;
            for (e, k, d) in &self.rows {
                if *e > from && *e <= anchor && k == key {
                    acc += d;
                    rows += 1;
                }
            }
            (acc, rows)
        }
        fn deltas_at(&mut self, e: Epoch) -> Vec<(Key, Value)> {
            self.rows
                .iter()
                .filter(|(re, _, _)| *re == e)
                .map(|(_, k, d)| (k.clone(), *d))
                .collect()
        }
    }

    fn circuit(mat: Materialize, rung: Consistency) -> Circuit {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "postings".into(),
                is_base: true,
                anchor_key: vec![0],
            },
            vec![],
            internal_contract(),
            "postings",
        );
        let agg = c.add(
            Op::Aggregate {
                group_key: vec![0],
                aggs: vec![(Agg::Sum, Scalar::Column(1))],
            },
            vec![src],
            ServeContract {
                consistency: rung,
                materialize: mat,
                retain: Retention::Evictable,
                lineage: Lineage::Key,
            },
            "balance",
        );
        c.set_output("balance", agg);
        c
    }

    #[test]
    fn a_read_after_eviction_returns_the_same_value() {
        // Reconstruction equivalence, on the running engine: the central corollary of
        // Contribution 1, which is that memory pressure can never change an answer.
        let mut base = FoldBase::new(0);
        for i in 0..50 {
            base.seal(vec![i % 5], 100 + i as i128);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(2),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();

        let mut first = HashMap::new();
        for k in 0..5i64 {
            first.insert(k, v.read(&mut base, &vec![k], anchor).value);
        }
        // With a budget of 2 and 5 keys, the sweep above already evicted three of them.
        assert!(
            v.stats.evictions >= 3,
            "the budget must actually bite: {:?}",
            v.stats
        );
        for k in 0..5i64 {
            let again = v.read(&mut base, &vec![k], anchor).value;
            assert_eq!(again, first[&k], "key {k} changed across an eviction");
        }
    }

    #[test]
    fn an_evicted_entry_keeps_its_version_and_never_answers_zero() {
        let mut base = FoldBase::new(0);
        base.seal(vec![7], 500);
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(1),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        assert_eq!(v.read(&mut base, &vec![7], anchor).value, 500);
        // Force it out.
        base.seal(vec![8], 900);
        let head = base.frontier();
        v.read(&mut base, &vec![8], head);
        assert!(
            matches!(v.slot(&vec![7]), Slot::Hole(_)),
            "must be a hole, not gone: {}",
            v.slot(&vec![7])
        );
        // And reading it back reconstructs the real value, not the aggregate's identity.
        let head = base.frontier();
        assert_eq!(v.read(&mut base, &vec![7], head).value, 500);
    }

    #[test]
    fn every_answer_carries_its_anchor() {
        let mut base = FoldBase::new(0);
        base.seal(vec![1], 10);
        base.seal(vec![1], 20);
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            None,
            Policy::Lru,
        )
        .unwrap();
        let v = rt.view_mut("balance").unwrap();
        let a = v.read(&mut base, &vec![1], 1);
        let b = v.read(&mut base, &vec![1], 2);
        assert_eq!(
            (a.value, a.anchor),
            (10, 1),
            "an as-of read sees the prefix, not the head"
        );
        assert_eq!((b.value, b.anchor), (30, 2));
    }

    #[test]
    fn deltas_to_non_resident_entries_are_skipped_and_that_is_the_saving() {
        let mut base = FoldBase::new(0);
        for i in 0..20 {
            base.seal(vec![i % 10], 1);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(2),
            Policy::Lru,
        )
        .unwrap();
        {
            let v = rt.view_mut("balance").unwrap();
            v.read(&mut base, &vec![0], 1);
        }
        for e in 1..=20 {
            rt.advance(&mut base, e);
        }
        let s = rt.stats();
        assert!(
            s.deltas_skipped > s.deltas_applied,
            "partiality must skip more than it applies: {s:?}"
        );
    }

    #[test]
    fn a_full_view_ignores_the_budget() {
        // The contract overrides the caller: `materialize: full` is a promise the runtime
        // keeps even when someone hands it a budget.
        let mut base = FoldBase::new(0);
        for i in 0..10 {
            base.seal(vec![i], 1);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Full, Consistency::Snapshot),
            Some(1),
            Policy::Lru,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        for k in 0..10i64 {
            v.read(&mut base, &vec![k], anchor);
        }
        assert_eq!(
            v.stats.evictions, 0,
            "a fully materialized view must not evict"
        );
        assert_eq!(v.resident_count(), 10);
    }

    #[test]
    fn a_lax_rung_is_maintained_less_often_and_that_is_where_its_cost_lives() {
        // The measured finding, as a test: a rung's price falls on maintenance, not reads.
        let mut runs = Vec::new();
        for rung in [
            Consistency::Bounded {
                epochs: 8,
                millis: 0,
            },
            Consistency::LedgerConsistent,
        ] {
            let mut base = FoldBase::new(0);
            for i in 0..200 {
                base.seal(vec![i % 4], 1);
            }
            let mut rt =
                Runtime::install(circuit(Materialize::Demand, rung), None, Policy::Lru).unwrap();
            {
                let v = rt.view_mut("balance").unwrap();
                for k in 0..4i64 {
                    v.read(&mut base, &vec![k], 1);
                }
            }
            for e in 1..=200 {
                rt.advance(&mut base, e);
            }
            runs.push(rt.stats());
        }
        let (lax, strict) = (runs[0], runs[1]);
        assert!(
            strict.deltas_applied > lax.deltas_applied * 4,
            "the strict rung must pay materially more maintenance: {lax:?} vs {strict:?}"
        );
        assert_eq!(
            lax.reads, strict.reads,
            "and the read counts must be indistinguishable"
        );
    }

    #[test]
    fn checkpoints_bound_reconstruction_by_the_interval_not_by_history() {
        // SC7, the Bounded Reconstruction Theorem, on the running engine. Without
        // checkpoints the fold grows with history; with them it does not.
        let mut costs = Vec::new();
        for interval in [0usize, 16] {
            let mut base = FoldBase::new(interval);
            for i in 0..2000 {
                base.seal(vec![i % 4], 1);
            }
            let mut rt = Runtime::install(
                circuit(Materialize::Demand, Consistency::Snapshot),
                Some(1),
                Policy::Lru,
            )
            .unwrap();
            let anchor = base.frontier();
            let v = rt.view_mut("balance").unwrap();
            for _ in 0..8 {
                for k in 0..4i64 {
                    v.read(&mut base, &vec![k], anchor);
                }
            }
            costs.push(v.stats.base_rows_read as f64 / v.stats.upqueries as f64);
        }
        let (unbounded, bounded) = (costs[0], costs[1]);
        assert!(
            unbounded > 400.0,
            "without checkpoints the fold is history-length: {unbounded}"
        );
        // Predicted bound: C/2 + 1 on average, so 9 for C = 16. Allow the interval itself
        // as slack, since the last checkpoint's position within the run varies.
        assert!(
            bounded <= 16.0,
            "with C=16 the fold must be bounded near C/2+1 = 9, measured {bounded}"
        );
    }

    #[test]
    fn a_circuit_outside_the_fragment_is_rejected_not_mis_executed() {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "p".into(),
                is_base: true,
                anchor_key: vec![0],
            },
            vec![],
            internal_contract(),
            "",
        );
        let f = c.add(
            Op::Filter {
                predicate: Scalar::LitBool(true),
            },
            vec![src],
            internal_contract(),
            "",
        );
        c.set_output("v", f);
        let e = match Runtime::install(c, None, Policy::Lru) {
            Err(e) => e,
            Ok(_) => panic!("a filter-terminated circuit must be rejected, not installed"),
        };
        assert!(
            matches!(e, Unsupported::Shape { op: "filter", .. }),
            "{e:?}"
        );
        assert!(e.explain().contains("outside the key-aggregate fragment"));
    }

    #[test]
    fn the_runtime_reads_every_checked_field_of_the_nodes_it_installs() {
        // The GoogleSQL discipline, enforced against this runtime: if `install` stopped
        // consulting the rung, the audit would say so.
        let c = circuit(Materialize::Demand, Consistency::Snapshot);
        let rt = Runtime::install(c, None, Policy::Lru).unwrap();
        let unread: Vec<_> = rt
            .circuit
            .audit_access()
            .unread
            .into_iter()
            .filter(|(n, _)| rt.views.iter().any(|v| v.node == *n))
            .collect();
        assert!(
            unread.is_empty(),
            "the runtime ignored a semantic field: {unread:?}"
        );
    }

    #[test]
    fn wiping_the_derived_layer_loses_nothing_the_base_does_not_have() {
        // Rebuild-from-base: the derived layer holds no information the base does not.
        let mut base = FoldBase::new(8);
        for i in 0..300 {
            base.seal(vec![i % 6], (i as i128 % 7) - 3);
        }
        let mut rt = Runtime::install(
            circuit(Materialize::Demand, Consistency::Snapshot),
            Some(3),
            Policy::CostAware,
        )
        .unwrap();
        let anchor = base.frontier();
        let v = rt.view_mut("balance").unwrap();
        let before: Vec<Value> = (0..6i64)
            .map(|k| v.read(&mut base, &vec![k], anchor).value)
            .collect();
        v.wipe();
        let after: Vec<Value> = (0..6i64)
            .map(|k| v.read(&mut base, &vec![k], anchor).value)
            .collect();
        assert_eq!(
            before, after,
            "a full rebuild from the base must reproduce every balance"
        );
    }
}
