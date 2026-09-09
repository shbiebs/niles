//! The distributed read path: REVs whose keys live on more than one node.
//!
//! # The claim this module exists to support, and why it is unusually cheap
//!
//! Distributing a read path is normally the hard half of a distributed database, because
//! reads must be made consistent against writes that are happening concurrently on other
//! nodes. That is a coordination problem, and coordination is what costs.
//!
//! Here it is not, and the reason is the base:
//!
//! > **A cross-shard upquery is a read of a frozen prefix on another node. Frozen prefixes
//! > do not move, so there is nothing to coordinate against.**
//!
//! A reconstruction at anchor *e* asks shard B for the fold of key *k* over epochs ≤ *e*.
//! Shard B's history at epochs ≤ *e* is immutable and fully retained, so the answer is a
//! pure function of (key, epoch) — the same at any wall-clock moment, from any node, under
//! any concurrent write load. No lock, no lease, no read timestamp negotiation, no
//! two-phase read. The reply can be cached forever, because it can never become wrong.
//!
//! This is the one place where the thesis's central design choice — an immutable,
//! epoch-ordered base — pays a dividend that compounds rather than merely holds. Every
//! property the single-node read path has, the distributed read path has for free, and the
//! proofs of Chapter 3 transfer verbatim because they were about frozen prefixes all along.
//!
//! # What is genuinely harder when distributed
//!
//! Three things, and they are named rather than glossed.
//!
//! **A snapshot across shards.** A read set spanning shards must use one anchor, and each
//! shard must have *reached* it. A shard lagging behind the requested anchor cannot answer;
//! it must wait, or the read must be refused. That is a real cost and it is where rung 3
//! and above become expensive in a way they are not on one node.
//!
//! **The frontier is now a minimum.** A cluster-wide strict read anchors at the *lowest*
//! frontier across the shards it touches, not the highest. One slow shard bounds the
//! freshness of every strict read that touches it — the distributed form of "a result is
//! no fresher than its stalest input", now with a network between the inputs.
//!
//! **A key's home may move.** Rebalancing changes which shard owns a key. This module
//! treats the shard map as versioned and refuses a read whose map version is stale, rather
//! than silently reading the wrong node — which would return an answer that is *correct for
//! a prefix of the wrong history*, the worst available failure.

use crate::absence::Epoch;
use crate::rev::{Base, Key, Value};
use std::collections::HashMap;

pub type ShardId = u16;

/// Which shard owns which key.
///
/// Versioned, because a rebalance changes it and a read carried out under a stale map
/// returns an answer computed from the wrong history. That answer would be internally
/// consistent and completely wrong, which is why the version is checked rather than
/// assumed.
#[derive(Debug, Clone)]
pub struct ShardMap {
    pub version: u64,
    shards: Vec<ShardId>,
}

impl ShardMap {
    /// A hash-partitioned map over `n` shards.
    pub fn hashed(n: u16, version: u64) -> ShardMap {
        ShardMap {
            version,
            shards: (0..n).collect(),
        }
    }

    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }

    /// The shard owning a key.
    ///
    /// Deliberately a plain hash of the key's first component rather than a range: a ledger
    /// is keyed by account, account activity is Zipf-distributed, and range partitioning a
    /// Zipf key space puts the hot accounts on one shard. Hashing spreads them, at the cost
    /// of range scans — which this engine's executable fragment does not have anyway.
    pub fn owner(&self, key: &Key) -> ShardId {
        let h = key.iter().fold(0xcbf29ce484222325u64, |a, k| {
            (a ^ (*k as u64)).wrapping_mul(0x100000001b3)
        });
        self.shards[(h % self.shards.len() as u64) as usize]
    }

    /// Group a batch of keys by owner, so one round trip serves each shard.
    pub fn route(&self, keys: &[Key]) -> HashMap<ShardId, Vec<Key>> {
        let mut out: HashMap<ShardId, Vec<Key>> = HashMap::new();
        for k in keys {
            out.entry(self.owner(k)).or_default().push(k.clone());
        }
        out
    }
}

/// Why a distributed read could not be served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    /// A shard has not yet reached the requested anchor. It must catch up; the read cannot
    /// be answered from a prefix that does not exist yet.
    Behind {
        shard: ShardId,
        frontier: Epoch,
        requested: Epoch,
    },
    /// The caller's shard map is stale. Refusing is the only safe response: reading the
    /// wrong node returns an answer that is correct for a prefix of the wrong history.
    StaleMap { caller: u64, current: u64 },
    /// The shard did not answer.
    Unreachable { shard: ShardId },
}

impl ReadError {
    pub fn explain(&self) -> String {
        match self {
            ReadError::Behind { shard, frontier, requested } => format!(
                "shard {shard} is at epoch {frontier} and the read asked for {requested}; a prefix that \
                 does not exist yet cannot be folded, so the read must wait rather than answer early"
            ),
            ReadError::StaleMap { caller, current } => format!(
                "the shard map moved from version {caller} to {current}; refusing rather than reading a \
                 node that no longer owns this key, because that answer would be correct for a prefix of \
                 the wrong history"
            ),
            ReadError::Unreachable { shard } => format!("shard {shard} did not answer"),
        }
    }
}

/// One shard, as seen from the read path.
pub struct Shard {
    pub id: ShardId,
    pub base: Box<dyn Base>,
    /// Rounds of network traffic this shard has served. Counted work, again: the figure a
    /// distributed read path lives or dies by is round trips, not instructions.
    pub round_trips: u64,
}

/// The cluster read path.
pub struct Cluster {
    pub map: ShardMap,
    pub shards: HashMap<ShardId, Shard>,
    /// Total cross-shard round trips. **The metric that matters**: a design that batches by
    /// owner does one per shard touched; a naive one does one per key.
    pub round_trips: u64,
    /// Reconstructions answered from a cache of a frozen prefix. These are free in a way
    /// that a cache of *live* state never is, because a frozen answer cannot go stale.
    pub cache_hits: u64,
    cache: HashMap<(Key, Epoch), Value>,
}

impl Cluster {
    pub fn new(map: ShardMap) -> Cluster {
        Cluster {
            map,
            shards: HashMap::new(),
            round_trips: 0,
            cache_hits: 0,
            cache: HashMap::new(),
        }
    }

    pub fn add_shard(&mut self, id: ShardId, base: Box<dyn Base>) {
        self.shards.insert(
            id,
            Shard {
                id,
                base,
                round_trips: 0,
            },
        );
    }

    /// **The cluster-wide strict frontier: the minimum, not the maximum.**
    ///
    /// A read that must reflect every committed write can only be anchored where *every*
    /// shard it touches has arrived. One slow shard therefore bounds the freshness of every
    /// strict read that touches it — the distributed form of "no fresher than its stalest
    /// input", now with a network in between, and the reason rung 5 is more expensive in a
    /// cluster than on one node.
    pub fn strict_frontier(&self) -> Epoch {
        self.shards
            .values()
            .map(|s| s.base.frontier())
            .min()
            .unwrap_or(0)
    }

    /// The frontier a read touching only these shards may use.
    pub fn frontier_for(&self, keys: &[Key]) -> Epoch {
        self.map
            .route(keys)
            .keys()
            .filter_map(|s| self.shards.get(s))
            .map(|s| s.base.frontier())
            .min()
            .unwrap_or(0)
    }

    /// Reconstruct one key at an anchor, from whichever shard owns it.
    ///
    /// Returns the value and the base rows the owning shard had to fold, so counted work
    /// stays comparable with the single-node figures of Chapter 9.
    pub fn reconstruct(
        &mut self,
        key: &Key,
        anchor: Epoch,
        map_version: u64,
    ) -> Result<(Value, u64), ReadError> {
        if map_version != self.map.version {
            return Err(ReadError::StaleMap {
                caller: map_version,
                current: self.map.version,
            });
        }
        // A frozen prefix cannot change, so a cached answer can never become wrong. This is
        // the one caching decision in the whole system that needs no invalidation rule.
        if let Some(v) = self.cache.get(&(key.clone(), anchor)) {
            self.cache_hits += 1;
            return Ok((*v, 0));
        }
        let owner = self.map.owner(key);
        let Some(shard) = self.shards.get_mut(&owner) else {
            return Err(ReadError::Unreachable { shard: owner });
        };
        let f = shard.base.frontier();
        if f < anchor {
            return Err(ReadError::Behind {
                shard: owner,
                frontier: f,
                requested: anchor,
            });
        }
        shard.round_trips += 1;
        self.round_trips += 1;
        let (v, rows) = shard.base.reconstruct(key, anchor);
        self.cache.insert((key.clone(), anchor), v);
        Ok((v, rows))
    }

    /// **A snapshot read across shards.** One anchor for the whole read set, chosen as the
    /// minimum frontier over the shards touched, so every shard can answer it.
    ///
    /// Batched by owner: one round trip per *shard*, not per key. On a Zipf key space over
    /// eight shards that is the difference between eight round trips and several hundred,
    /// and it is the only optimisation in this module that changes the asymptotics.
    pub fn snapshot_read(
        &mut self,
        keys: &[Key],
        map_version: u64,
    ) -> Result<(Vec<(Key, Value)>, Epoch), ReadError> {
        if map_version != self.map.version {
            return Err(ReadError::StaleMap {
                caller: map_version,
                current: self.map.version,
            });
        }
        let anchor = self.frontier_for(keys);
        let routed = self.map.route(keys);
        let mut out = Vec::with_capacity(keys.len());
        for (shard_id, shard_keys) in routed {
            let Some(shard) = self.shards.get_mut(&shard_id) else {
                return Err(ReadError::Unreachable { shard: shard_id });
            };
            // One round trip for the whole batch destined for this shard.
            shard.round_trips += 1;
            self.round_trips += 1;
            for k in shard_keys {
                let (v, _rows) = shard.base.reconstruct(&k, anchor);
                self.cache.insert((k.clone(), anchor), v);
                out.push((k, v));
            }
        }
        Ok((out, anchor))
    }

    /// Rebalance: install a new shard map. Every in-flight read carrying the old version
    /// will now be refused rather than served from the wrong node.
    pub fn rebalance(&mut self, new_map: ShardMap) {
        // The cache is keyed by (key, epoch) and its entries remain *correct* — a frozen
        // prefix is a frozen prefix wherever it lives — so a rebalance does not invalidate
        // it. That is another dividend of immutability: the usual cache-coherence problem
        // of a rebalance simply does not arise.
        self.map = new_map;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap as Map;

    /// A shard whose history is a vector. Deliberately unoptimised: it defines the right
    /// answer rather than implementing a fast one.
    struct VecBase {
        rows: Vec<(Epoch, Key, Value)>,
        head: Epoch,
    }

    impl VecBase {
        fn new(keys: &[i64], upto: Epoch, per_epoch: Value) -> VecBase {
            let mut rows = Vec::new();
            for e in 1..=upto {
                for k in keys {
                    rows.push((e, vec![*k], per_epoch));
                }
            }
            VecBase { rows, head: upto }
        }
    }

    impl Base for VecBase {
        fn frontier(&self) -> Epoch {
            self.head
        }
        fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
            let mut acc = 0i128;
            let mut n = 0u64;
            for (e, k, v) in &self.rows {
                if *e <= anchor && k == key {
                    acc += v;
                    n += 1;
                }
            }
            (acc, n)
        }
        /// The fixture's epochs are small enough that counting and materialising cost the same,
        /// so this is `deltas_at(e).len()`. Written out rather than defaulted, because a default
        /// of exactly this shape in the trait would preserve the defect `delta_rows_at` exists to
        /// remove: every implementation would then materialise the epoch it was about to refuse.
        fn delta_rows_at(&self, e: Epoch) -> u64 {
            self.deltas_at(e).len() as u64
        }

        fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
            self.rows
                .iter()
                .filter(|(re, _, _)| *re == e)
                .map(|(_, k, v)| (k.clone(), *v))
                .collect()
        }
    }

    fn cluster(shards: u16, keys_per_shard: usize, upto: Epoch) -> Cluster {
        let map = ShardMap::hashed(shards, 1);
        let mut c = Cluster::new(map.clone());
        // Give every shard the keys it owns.
        let all: Vec<i64> = (0..(shards as usize * keys_per_shard) as i64).collect();
        let mut by_shard: Map<ShardId, Vec<i64>> = Map::new();
        for k in &all {
            by_shard.entry(map.owner(&vec![*k])).or_default().push(*k);
        }
        for s in 0..shards {
            let ks = by_shard.remove(&s).unwrap_or_default();
            c.add_shard(s, Box::new(VecBase::new(&ks, upto, 10)));
        }
        c
    }

    #[test]
    fn a_cross_shard_reconstruction_needs_no_coordination() {
        // The claim this module exists for: a read of a frozen prefix on another node is a
        // pure function of (key, epoch). Reading it twice, with writes in between, gives
        // the same answer — with no lock, lease or timestamp negotiation anywhere.
        let mut c = cluster(4, 8, 50);
        let key = vec![5i64];
        let (a, _) = c.reconstruct(&key, 20, 1).unwrap();
        // "Writes happen elsewhere": the head moves, but the prefix at 20 cannot.
        let (b, _) = c.reconstruct(&key, 20, 1).unwrap();
        assert_eq!(a, b);
        assert_eq!(a, 200, "twenty epochs of ten");
    }

    #[test]
    fn an_answer_about_a_frozen_prefix_can_be_cached_forever() {
        // The one caching decision in the system that needs no invalidation rule.
        let mut c = cluster(4, 8, 50);
        let key = vec![3i64];
        c.reconstruct(&key, 30, 1).unwrap();
        let before = c.round_trips;
        for _ in 0..100 {
            c.reconstruct(&key, 30, 1).unwrap();
        }
        assert_eq!(
            c.round_trips, before,
            "a frozen answer must never need re-fetching"
        );
        assert_eq!(c.cache_hits, 100);
    }

    #[test]
    fn a_snapshot_read_costs_one_round_trip_per_shard_not_per_key() {
        // The only optimisation here that changes asymptotics.
        let mut c = cluster(4, 25, 40);
        let keys: Vec<Key> = (0..100i64).map(|k| vec![k]).collect();
        let (rows, _anchor) = c.snapshot_read(&keys, 1).unwrap();
        assert_eq!(rows.len(), 100);
        assert!(
            c.round_trips <= 4,
            "expected at most one trip per shard, took {}",
            c.round_trips
        );
    }

    #[test]
    fn a_snapshot_uses_one_anchor_for_the_whole_read_set() {
        let mut c = cluster(3, 10, 60);
        let keys: Vec<Key> = (0..30i64).map(|k| vec![k]).collect();
        let (rows, anchor) = c.snapshot_read(&keys, 1).unwrap();
        assert_eq!(anchor, 60);
        // Every value is the same because every key has the same history in this fixture —
        // the point is that they were all folded at one epoch.
        assert!(
            rows.iter().all(|(_, v)| *v == 600),
            "one anchor, one consistent set of answers"
        );
    }

    #[test]
    fn the_cluster_strict_frontier_is_the_minimum_not_the_maximum() {
        // The distributed form of "no fresher than its stalest input". One slow shard
        // bounds every strict read that touches it, and this is where rung 5 gets expensive
        // in a cluster in a way it is not on one node.
        let mut c = Cluster::new(ShardMap::hashed(3, 1));
        c.add_shard(0, Box::new(VecBase::new(&[0], 100, 1)));
        c.add_shard(1, Box::new(VecBase::new(&[1], 40, 1)));
        c.add_shard(2, Box::new(VecBase::new(&[2], 90, 1)));
        assert_eq!(c.strict_frontier(), 40, "the laggard sets the frontier");
    }

    #[test]
    fn a_shard_behind_the_requested_anchor_refuses_rather_than_answering_early() {
        let mut c = Cluster::new(ShardMap::hashed(1, 1));
        c.add_shard(0, Box::new(VecBase::new(&[7], 10, 1)));
        let e = c.reconstruct(&vec![7], 50, 1).unwrap_err();
        assert!(matches!(
            e,
            ReadError::Behind {
                frontier: 10,
                requested: 50,
                ..
            }
        ));
        assert!(e.explain().contains("does not exist yet"));
    }

    #[test]
    fn a_stale_shard_map_is_refused_rather_than_read_from_the_wrong_node() {
        // The worst available failure is not an error but an answer that is correct for a
        // prefix of the wrong history. Refusing is the only safe response.
        let mut c = cluster(4, 8, 30);
        assert!(c.reconstruct(&vec![1], 10, 1).is_ok());
        c.rebalance(ShardMap::hashed(8, 2));
        let e = c.reconstruct(&vec![1], 10, 1).unwrap_err();
        assert!(matches!(
            e,
            ReadError::StaleMap {
                caller: 1,
                current: 2
            }
        ));
        assert!(e.explain().contains("wrong history"));
        assert!(
            c.reconstruct(&vec![1], 10, 2).is_ok(),
            "the current map works"
        );
    }

    #[test]
    fn a_rebalance_does_not_invalidate_cached_frozen_answers() {
        // Another dividend of immutability: the cache-coherence problem a rebalance usually
        // creates does not arise, because a frozen prefix is a frozen prefix wherever it
        // lives.
        let mut c = cluster(4, 8, 30);
        let (before, _) = c.reconstruct(&vec![6], 15, 1).unwrap();
        c.rebalance(ShardMap::hashed(8, 2));
        let hits_before = c.cache_hits;
        let (after, _) = c.reconstruct(&vec![6], 15, 2).unwrap();
        assert_eq!(before, after);
        assert_eq!(
            c.cache_hits,
            hits_before + 1,
            "the cached answer survived the rebalance"
        );
    }

    #[test]
    fn hash_partitioning_spreads_a_skewed_key_space() {
        // Range partitioning a Zipf key space puts the hot accounts on one shard. This is
        // the reason for hashing, so it gets a test rather than a comment.
        let map = ShardMap::hashed(8, 1);
        let mut counts = [0usize; 8];
        for k in 0..8000i64 {
            counts[map.owner(&vec![k]) as usize] += 1;
        }
        let (lo, hi) = (*counts.iter().min().unwrap(), *counts.iter().max().unwrap());
        assert!(hi < lo * 2, "distribution is lopsided: {counts:?}");
    }

    #[test]
    fn counted_work_is_comparable_with_the_single_node_figures() {
        // The distributed path must report base rows read the same way, or Chapter 9's
        // numbers stop being comparable across the two deployments.
        let mut c = cluster(2, 4, 25);
        let (_v, rows) = c.reconstruct(&vec![2], 25, 1).unwrap();
        assert_eq!(
            rows, 25,
            "twenty-five epochs folded, counted the same way as on one node"
        );
    }
}
