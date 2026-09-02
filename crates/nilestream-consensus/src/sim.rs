//! A deterministic network simulator, and the safety invariants it checks.
//!
//! # Why deterministic
//!
//! A consensus test that sleeps and hopes is worse than no consensus test, because it
//! trains its reader to re-run it. Everything here runs on a seeded PRNG with a logical
//! clock: message loss, reordering, partition and node restarts are all decisions the
//! simulator makes from the seed, so a failing run is reproducible from its seed alone and
//! a passing suite means the same thing every time.
//!
//! # The three invariants
//!
//! Checked after **every** step, not at the end, because an invariant checked only at the
//! end tells you a system was broken without telling you when.
//!
//! 1. **Election safety** — at most one leader per term.
//! 2. **Log matching** — if two nodes have an entry at the same index with the same term,
//!    the entries are identical, and so is every entry before it.
//! 3. **State machine safety** — an entry a node has reported committed is never
//!    afterwards changed or removed. **This is the one that matters here**: it is the
//!    distributed form of "the base is never partial", and its violation is money that
//!    existed and then did not.

use crate::{Entry, Index, Msg, Node, NodeId, Role, Term};
use std::collections::HashMap;

/// A seeded PRNG. An LCG, because reproducibility matters here and statistical quality
/// does not: the simulator needs decisions, not random numbers.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
            | 1)
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
    pub fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

/// How hostile the network is.
#[derive(Debug, Clone, Copy, Default)]
pub struct Network {
    /// Percent of messages dropped.
    pub loss: u64,
    /// Percent of messages delivered out of order.
    pub reorder: u64,
}

pub struct Cluster {
    pub nodes: Vec<Node>,
    /// In flight: (to, message).
    queue: Vec<(NodeId, Msg)>,
    pub rng: Rng,
    pub net: Network,
    /// Node pairs that cannot talk. A partition is a set of severed links, not a flag,
    /// because that is what a real partition is.
    partitioned: Vec<(NodeId, NodeId)>,
    /// Every (term, leader) seen, for election safety.
    leaders_seen: HashMap<Term, Vec<NodeId>>,
    /// What each node had reported committed, for state machine safety.
    committed_snapshot: HashMap<NodeId, Vec<Entry>>,
    pub steps: u64,
    pub delivered: u64,
    pub dropped: u64,
}

impl Cluster {
    pub fn new(n: usize, seed: u64) -> Cluster {
        let ids: Vec<NodeId> = (0..n as NodeId).collect();
        let nodes = ids
            .iter()
            .map(|id| Node::new(*id, ids.iter().copied().filter(|x| x != id).collect()))
            .collect();
        Cluster {
            nodes,
            queue: Vec::new(),
            rng: Rng::new(seed),
            net: Network::default(),
            partitioned: Vec::new(),
            leaders_seen: HashMap::new(),
            committed_snapshot: HashMap::new(),
            steps: 0,
            delivered: 0,
            dropped: 0,
        }
    }

    pub fn with_network(mut self, net: Network) -> Cluster {
        self.net = net;
        self
    }

    pub fn partition(&mut self, a: NodeId, b: NodeId) {
        self.partitioned.push((a, b));
    }
    pub fn heal(&mut self) {
        self.partitioned.clear();
    }
    fn reachable(&self, a: NodeId, b: NodeId) -> bool {
        !self
            .partitioned
            .iter()
            .any(|(x, y)| (*x == a && *y == b) || (*x == b && *y == a))
    }

    pub fn leader(&self) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|n| n.role == Role::Leader)
            .map(|n| n.id)
    }

    fn send(&mut self, from: NodeId, to: NodeId, msg: Msg) {
        if !self.reachable(from, to) || self.rng.chance(self.net.loss) {
            self.dropped += 1;
            return;
        }
        if self.rng.chance(self.net.reorder) && !self.queue.is_empty() {
            let at = self.rng.below(self.queue.len());
            self.queue.insert(at, (to, msg));
        } else {
            self.queue.push((to, msg));
        }
    }

    /// Broadcast whatever a node produced. Vote requests and append entries fan out to all
    /// peers; directed replies go to their addressee.
    fn dispatch(&mut self, from: NodeId, msgs: Vec<Msg>) {
        for m in msgs {
            match &m {
                Msg::RequestVote { .. } => {
                    let peers = self.nodes[from as usize].peers.clone();
                    for p in peers {
                        self.send(from, p, m.clone());
                    }
                }
                Msg::VoteGranted { to, .. }
                | Msg::VoteDenied { to, .. }
                | Msg::AppendOk { to, .. }
                | Msg::AppendRejected { to, .. } => {
                    let to = *to;
                    self.send(from, to, m);
                }
                Msg::AppendEntries { .. } => {
                    let peers = self.nodes[from as usize].peers.clone();
                    for p in peers {
                        self.send(from, p, m.clone());
                    }
                }
            }
        }
    }

    pub fn elect(&mut self, candidate: NodeId) {
        let msgs = self.nodes[candidate as usize].start_election();
        self.dispatch(candidate, msgs);
    }

    pub fn propose(&mut self, payload: Vec<u8>) -> Option<Index> {
        let leader = self.leader()?;
        let idx = self.nodes[leader as usize].propose(payload)?;
        self.heartbeat();
        Some(idx)
    }

    /// The leader replicates to everyone. In a real deployment this is a timer.
    pub fn heartbeat(&mut self) {
        let Some(leader) = self.leader() else { return };
        let peers = self.nodes[leader as usize].peers.clone();
        for p in peers {
            let m = self.nodes[leader as usize].replicate_to(p);
            self.send(leader, p, m);
        }
    }

    /// Deliver one message and check every invariant.
    pub fn step(&mut self) -> bool {
        if self.queue.is_empty() {
            return false;
        }
        let (to, msg) = self.queue.remove(0);
        self.delivered += 1;
        self.steps += 1;
        let replies = self.nodes[to as usize].handle(msg);
        self.dispatch(to, replies);
        self.check_invariants();
        true
    }

    /// Run until quiet, or until a step budget is exhausted. The budget is what turns a
    /// livelock into a test failure rather than a hang.
    pub fn run(&mut self, max_steps: u64) -> u64 {
        let start = self.steps;
        while self.steps - start < max_steps {
            if !self.step() {
                break;
            }
        }
        self.steps - start
    }

    fn check_invariants(&mut self) {
        // 1. Election safety.
        for n in &self.nodes {
            if n.role == Role::Leader {
                let seen = self.leaders_seen.entry(n.term).or_default();
                if !seen.contains(&n.id) {
                    seen.push(n.id);
                }
                assert!(
                    seen.len() <= 1,
                    "ELECTION SAFETY VIOLATED: two leaders in term {}: {:?}",
                    n.term,
                    seen
                );
            }
        }

        // 2. Log matching.
        for a in &self.nodes {
            for b in &self.nodes {
                if a.id >= b.id {
                    continue;
                }
                for ea in &a.log {
                    if let Some(eb) = b
                        .log
                        .iter()
                        .find(|e| e.index == ea.index && e.term == ea.term)
                    {
                        assert_eq!(
                            ea, eb,
                            "LOG MATCHING VIOLATED at index {} term {}: nodes {} and {} disagree",
                            ea.index, ea.term, a.id, b.id
                        );
                    }
                }
            }
        }

        // 3. State machine safety. The one that matters: a committed entry is never
        // afterwards changed or removed. Its violation is money that existed and then
        // did not.
        for n in &self.nodes {
            let prev = self.committed_snapshot.entry(n.id).or_default();
            for (i, old) in prev.iter().enumerate() {
                let now = n.committed_history.get(i);
                assert_eq!(
                    Some(old),
                    now,
                    "STATE MACHINE SAFETY VIOLATED on node {}: entry {} of its committed history changed",
                    n.id,
                    i
                );
            }
            if n.committed_history.len() > prev.len() {
                *prev = n.committed_history.clone();
            }
        }

        // 4. And the chain, on every node, always.
        for n in &self.nodes {
            assert!(n.chain_is_intact(), "the hash chain broke on node {}", n.id);
        }
    }

    /// The committed prefix every node agrees on.
    pub fn agreed_prefix(&self) -> Vec<Entry> {
        let min_commit = self.nodes.iter().map(|n| n.commit_index).min().unwrap_or(0);
        self.nodes[0]
            .log
            .iter()
            .filter(|e| e.index <= min_commit)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elected(n: usize, seed: u64) -> Cluster {
        let mut c = Cluster::new(n, seed);
        c.elect(0);
        c.run(200);
        assert_eq!(
            c.leader(),
            Some(0),
            "node 0 should have won an uncontested election"
        );
        c
    }

    #[test]
    fn an_uncontested_candidate_wins_and_only_one_leader_exists() {
        let c = elected(5, 1);
        assert_eq!(c.nodes.iter().filter(|n| n.role == Role::Leader).count(), 1);
        assert_eq!(c.nodes[0].term, 1);
    }

    #[test]
    fn a_proposal_commits_once_a_quorum_has_it() {
        let mut c = elected(5, 2);
        let idx = c.propose(b"epoch-1".to_vec()).expect("leader accepts");
        c.run(500);
        assert_eq!(idx, 1);
        assert!(
            c.nodes[0].commit_index >= 1,
            "the leader must commit after a quorum acks"
        );
        // And the commit index is the frontier a strict read would anchor at.
        assert_eq!(c.nodes[0].frontier(), c.nodes[0].commit_index);
    }

    #[test]
    fn a_minority_partition_cannot_commit_and_a_majority_can() {
        // The availability boundary, which is what rung 5 gives up. Nodes 3 and 4 are cut
        // off; the majority {0,1,2} keeps making progress and the minority does not.
        let mut c = elected(5, 3);
        for a in [0u8, 1, 2] {
            for b in [3u8, 4] {
                c.partition(a, b);
            }
        }
        c.propose(b"during-partition".to_vec());
        c.run(500);
        assert!(c.nodes[0].commit_index >= 1, "a majority must still commit");
        assert_eq!(c.nodes[3].commit_index, 0, "the minority must not");
        assert_eq!(c.nodes[4].commit_index, 0);

        // Healing catches the minority up, and nothing committed changes.
        c.heal();
        c.heartbeat();
        c.run(1000);
        assert!(
            c.nodes[3].commit_index >= 1,
            "a healed follower must catch up"
        );
        assert_eq!(c.nodes[3].log, c.nodes[0].log, "and end with the same log");
    }

    /// **An even cluster, because an odd one cannot tell a majority from a half.**
    ///
    /// Every test above uses three or five nodes, and for odd `total` the strict-majority
    /// rule `replicas * 2 > total` and the off-by-one `replicas * 2 >= total` accept exactly
    /// the same sets — with five nodes both mean "at least three". Weakening the rule to
    /// `>=` therefore passed the entire suite, which is to say the suite did not test the
    /// quorum rule at all; it tested a rule that happens to agree with it on the sizes it
    /// used.
    ///
    /// With four nodes the two rules part company: a strict majority is three, and `>=`
    /// would commit on two — a split-brain, since the other two could commit something else.
    /// This test is the one that dies when the rule is weakened.
    #[test]
    fn a_bare_half_of_an_even_cluster_is_not_a_quorum() {
        let mut c = elected(4, 11);
        // Sever the leader's link to two of its three followers, leaving it able to reach
        // exactly one: two nodes in total, which is half of four and not a majority.
        for b in [2u8, 3] {
            c.partition(0, b);
        }
        c.propose(b"half".to_vec()).expect("the leader accepts it");
        c.run(1000);
        assert_eq!(
            c.nodes[0].commit_index, 0,
            "two of four is a half, not a majority: committing here is a split-brain,              because {{2,3}} could commit something else with equal right"
        );

        // The control, on the same cluster: restore one link and three of four is a
        // majority, so the entry commits. Without this the assertion above would be
        // satisfied by a cluster that never commits anything.
        c.heal();
        c.heartbeat();
        c.run(1000);
        assert!(
            c.nodes[0].commit_index >= 1,
            "three of four is a majority and must commit"
        );
    }

    /// The same boundary stated without a partition: the *counting* rule itself.
    ///
    /// `advance_commit` is the only place a quorum is computed, and this pins its arithmetic
    /// at the two sizes where an off-by-one is visible. It is deliberately a unit test of
    /// the predicate rather than a scenario, so that a future refactor of the simulator
    /// cannot make the boundary untested by making the scenario unreachable.
    #[test]
    fn a_quorum_is_strictly_more_than_half_at_every_cluster_size() {
        for total in 2usize..=9 {
            let need = total / 2 + 1;
            for replicas in 1..=total {
                let is_quorum = replicas * 2 > total;
                assert_eq!(
                    is_quorum,
                    replicas >= need,
                    "with {total} nodes, {replicas} replicas: a quorum is {need} or more"
                );
                if total % 2 == 0 && replicas == total / 2 {
                    assert!(
                        !is_quorum,
                        "a bare half of {total} must never be a quorum; `replicas * 2 >= \
                         total` would make it one, and passes every odd-sized test"
                    );
                }
            }
        }
    }

    #[test]
    fn a_candidate_whose_log_is_behind_cannot_win() {
        // The up-to-date restriction, which is the whole of election safety: a leader
        // always has every committed entry, so a committed entry can never be lost.
        let mut c = elected(3, 4);
        for i in 0..5 {
            c.propose(format!("e{i}").into_bytes());
        }
        c.run(1000);
        assert!(c.nodes[0].last_index() == 5);

        // Node 2 is forced behind, then stands for election.
        c.nodes[2].log.truncate(1);
        let msgs = c.nodes[2].start_election();
        c.dispatch(2, msgs);
        c.run(500);
        assert_ne!(
            c.leader(),
            Some(2),
            "a behind candidate must not win: {:?}",
            c.leader()
        );
    }

    #[test]
    fn a_forged_entry_is_rejected_even_from_a_current_leader() {
        // What the hash chain adds over Raft. A leader whose entry does not follow from its
        // stated parent is refused, because the follower recomputes the link rather than
        // trusting the term. In Raft this entry would be accepted.
        let mut c = elected(3, 5);
        c.propose(b"honest".to_vec());
        c.run(500);

        let parent = c.nodes[0].hash_at(1);
        let mut forged = Entry::seal(c.nodes[0].term, 2, parent, b"forged".to_vec());
        forged.payload = b"tampered-after-sealing".to_vec(); // hash no longer matches

        let msg = Msg::AppendEntries {
            term: c.nodes[0].term,
            from: 0,
            prev_index: 1,
            prev_hash: parent,
            entries: vec![forged],
            leader_commit: 1,
        };
        let replies = c.nodes[1].handle(msg);
        let Some(Msg::AppendRejected { reason, .. }) = replies.first() else {
            panic!("a forged entry must be rejected, got {replies:?}")
        };
        assert_eq!(*reason, crate::RejectReason::BrokenChain);
        assert_eq!(c.nodes[1].last_index(), 1, "and must not be appended");
    }

    #[test]
    fn no_committed_entry_is_ever_truncated_under_loss_and_reordering() {
        // The headline safety property, exercised rather than argued. The invariant is
        // checked after every single message delivery inside `step`, so a violation is
        // caught at the step that caused it.
        for seed in 1..=25u64 {
            let mut c = Cluster::new(5, seed).with_network(Network {
                loss: 20,
                reorder: 30,
            });
            c.elect(0);
            c.run(300);
            if c.leader().is_none() {
                continue; // this seed's election was lost to the network; not a failure
            }
            for i in 0..10 {
                c.propose(format!("txn-{i}").into_bytes());
                c.run(200);
                c.heartbeat();
                c.run(200);
            }
            // Everything the leader believes committed is in every node's log that has it.
            let leader = c.leader().unwrap() as usize;
            let committed: Vec<&Entry> = c.nodes[leader]
                .log
                .iter()
                .filter(|e| e.index <= c.nodes[leader].commit_index)
                .collect();
            for e in committed {
                for n in &c.nodes {
                    if let Some(theirs) = n.log.iter().find(|x| x.index == e.index) {
                        assert_eq!(
                            theirs.hash, e.hash,
                            "seed {seed}: node {} has a different entry at committed index {}",
                            n.id, e.index
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_cluster_converges_after_a_partition_heals_across_many_seeds() {
        let mut converged = 0;
        for seed in 1..=20u64 {
            let mut c = Cluster::new(5, seed).with_network(Network {
                loss: 10,
                reorder: 20,
            });
            c.elect(0);
            c.run(400);
            if c.leader().is_none() {
                continue;
            }
            for a in [0u8, 1, 2] {
                for b in [3u8, 4] {
                    c.partition(a, b);
                }
            }
            for i in 0..5 {
                c.propose(format!("p{i}").into_bytes());
                c.run(200);
            }
            c.heal();
            for _ in 0..40 {
                c.heartbeat();
                c.run(400);
            }
            let leader = c.leader().unwrap() as usize;
            let commit = c.nodes[leader].commit_index;
            if commit > 0 && c.nodes.iter().all(|n| n.last_index() >= commit) {
                converged += 1;
            }
        }
        assert!(
            converged >= 15,
            "only {converged}/20 seeds converged after healing"
        );
    }

    #[test]
    fn a_leader_does_not_commit_a_previous_terms_entry_on_replica_count_alone() {
        // **The subtle Raft rule**, and the one most often got wrong: an entry from an
        // earlier term is not committed merely because it is now on a quorum, because a
        // later leader could still overwrite it. It commits implicitly when a current-term
        // entry above it commits.
        //
        // The earlier version of this test proposed the entry and ran the cluster for 500
        // steps before advancing the term — by which time the entry had *already committed
        // at its own term*, so `commit_index` could not move again and the assertion held
        // whether or not the rule existed. Deleting the term restriction from
        // `advance_commit` left the whole suite green. The entry must therefore be
        // uncommitted at the moment the term advances, which means it must not have been
        // replicated: the leader is cut off first.
        let mut c = elected(3, 7);
        c.partition(0, 1);
        c.partition(0, 2);
        c.propose(b"term1".to_vec()).expect("the leader accepts it");
        c.run(500);
        assert_eq!(
            c.nodes[0].commit_index, 0,
            "the entry must still be uncommitted, or this test cannot distinguish the rule              from its absence"
        );

        // A later term, with the old entry now reported present on a quorum.
        c.nodes[0].term += 1;
        c.nodes[0].match_index.insert(1, 1);
        c.nodes[0].match_index.insert(2, 1);
        let t = c.nodes[0].term;
        c.nodes[0].handle(Msg::AppendOk {
            term: t,
            from: 1,
            to: 0,
            match_index: 1,
        });
        assert_eq!(
            c.nodes[0].commit_index, 0,
            "an entry from a previous term must not be committed on replica count alone: a              later leader could still overwrite it, and a reader anchored here would have              observed a write that never happened"
        );

        // **The control.** An entry from the *current* term, on the same quorum, must
        // commit — and carries the old one with it. Without this the assertion above is
        // satisfied by a leader that commits nothing ever.
        c.heal();
        c.propose(b"term2".to_vec()).expect("the leader accepts it");
        c.run(1000);
        assert!(
            c.nodes[0].commit_index >= 2,
            "a current-term entry on a quorum commits, and commits the earlier one with it"
        );
    }

    #[test]
    fn every_unbuilt_feature_is_named_with_what_it_would_take() {
        // The list is data so it cannot quietly shrink in the prose while the code stays
        // the same.
        assert!(crate::NOT_BUILT.len() >= 5);
        for (name, why) in crate::NOT_BUILT {
            assert!(
                !name.is_empty() && why.len() > 20,
                "`{name}` has no real explanation"
            );
        }
        assert!(crate::NOT_BUILT
            .iter()
            .any(|(n, _)| n.contains("cross-shard")));
    }
}
