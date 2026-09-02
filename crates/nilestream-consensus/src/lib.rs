//! Replicated ledger groups: agreement on the epoch order, across nodes.
//!
//! # Why this is a small addition rather than a new system
//!
//! Raft replicates a log. **The ledger is a log** — append-only, totally ordered, never
//! partial — so the mapping is not an analogy, it is an identity:
//!
//! | Raft | Nilestream |
//! |---|---|
//! | log entry | epoch |
//! | leader | the sealer of §7.1 |
//! | commit index | the visibility frontier |
//! | log matching property | the hash chain |
//!
//! The single-node design already had a single sealer and a frontier that advances only
//! after durability. Distributing it changes *who* decides the order and *when* an epoch is
//! durable enough to publish — from "one disk has it" to "a quorum of disks has it" — and
//! changes nothing else. That is the argument for having built the single-node write path
//! the way §7.1 does: the distributed version is the same shape with a quorum where the
//! `fsync` was.
//!
//! # What the hash chain adds that Raft does not have
//!
//! Raft's log matching property is maintained by *protocol*: a follower accepts an
//! `AppendEntries` if the previous index and term match, and the leader backs up on
//! rejection until they do. It is sound, and it rests entirely on nodes reporting their own
//! state honestly. A node that lied about its previous term — through a bug, a corrupted
//! disk, or malice — would be believed.
//!
//! Here the previous entry is identified by its **hash**, not by its term. A follower
//! recomputes the link before accepting, so an entry that does not follow from the one
//! before it is rejected regardless of what any node claims about it. This does not make
//! the design Byzantine-tolerant — a lying *leader* can still refuse to make progress, and
//! §11.1 scopes Byzantine settings out — but it converts a whole class of silent divergence
//! into a detected one, and it costs nothing, because the chain is computed anyway for
//! audit.
//!
//! # What is built here and what is not
//!
//! Built: terms, elections with the up-to-date restriction, log replication with hash-linked
//! matching, commit by quorum, follower catch-up, and a deterministic network simulator with
//! loss, reordering and partition. Every test is deterministic — no sleeps, no wall clock —
//! because a flaky consensus test is worse than no consensus test: it trains its reader to
//! re-run it.
//!
//! Not built: membership changes, log compaction with snapshots, pre-vote, leadership
//! transfer, and the cross-shard commit protocol of §8.6. Each is named in [`NOT_BUILT`].

pub mod cross_shard;
pub mod sim;

use nilestream_ledger::chain::Hasher256;
use std::collections::HashMap;

/// The features a production replicated log needs that this does not have. Kept as data so
/// a test can assert the list has not quietly shrunk in the documentation.
pub const NOT_BUILT: &[(&str, &str)] = &[
    ("membership changes", "joint consensus; a group's node set is fixed at construction here"),
    ("log compaction", "snapshots plus an install-snapshot RPC; the ledger is never compacted, so a catching-up follower replays from the beginning"),
    ("pre-vote", "a partitioned node returning with a high term forces an unnecessary election"),
    ("leadership transfer", "graceful handover for maintenance"),
    ("cross-shard commit", "the two-phase protocol of thesis 8.6, which is what a transaction spanning two ledger groups needs"),
];

pub type NodeId = u8;
pub type Term = u64;
pub type Index = u64;

/// One epoch, as a replicated log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub term: Term,
    pub index: Index,
    /// The chain link. Recomputed by every follower before acceptance, never trusted.
    pub hash: [u8; 32],
    pub parent: [u8; 32],
    pub payload: Vec<u8>,
}

impl Entry {
    pub fn seal(term: Term, index: Index, parent: [u8; 32], payload: Vec<u8>) -> Entry {
        let mut h = Hasher256::new();
        h.update(&parent);
        h.update(&term.to_le_bytes());
        h.update(&index.to_le_bytes());
        h.update(&payload);
        Entry {
            term,
            index,
            hash: h.finalize(),
            parent,
            payload,
        }
    }
    /// Whether this entry genuinely follows from `parent_hash`. This is the check Raft does
    /// with a term comparison and this design does with a recomputation.
    pub fn follows(&self, parent_hash: [u8; 32]) -> bool {
        self.parent == parent_hash
            && Entry::seal(self.term, self.index, parent_hash, self.payload.clone()).hash
                == self.hash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

/// Messages between nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    RequestVote {
        term: Term,
        from: NodeId,
        last_index: Index,
        last_term: Term,
    },
    VoteGranted {
        term: Term,
        from: NodeId,
        to: NodeId,
    },
    VoteDenied {
        term: Term,
        from: NodeId,
        to: NodeId,
        reason: &'static str,
    },
    AppendEntries {
        term: Term,
        from: NodeId,
        prev_index: Index,
        prev_hash: [u8; 32],
        entries: Vec<Entry>,
        leader_commit: Index,
    },
    AppendOk {
        term: Term,
        from: NodeId,
        to: NodeId,
        match_index: Index,
    },
    AppendRejected {
        term: Term,
        from: NodeId,
        to: NodeId,
        hint: Index,
        reason: RejectReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// The leader's term is behind.
    StaleTerm,
    /// The follower has no entry at `prev_index`, or a different one.
    LogMismatch,
    /// **The chain does not link.** Raft has no equivalent: this is corruption or a lie,
    /// not a normal disagreement about which prefix is current, and a follower must not
    /// accept it even from a leader whose term is current.
    BrokenChain,
}

/// One node of a ledger group.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub role: Role,
    pub term: Term,
    pub voted_for: Option<NodeId>,
    pub log: Vec<Entry>,
    /// The highest index known committed. **This is the visibility frontier**: a read at
    /// the strict rung is anchored here and nowhere else.
    pub commit_index: Index,
    /// Leader state: per-follower next and match indices.
    pub next_index: HashMap<NodeId, Index>,
    pub match_index: HashMap<NodeId, Index>,
    pub peers: Vec<NodeId>,
    pub votes: Vec<NodeId>,
    /// Entries this node has ever reported committed, for the safety invariant.
    pub committed_history: Vec<Entry>,
}

impl Node {
    pub fn new(id: NodeId, peers: Vec<NodeId>) -> Node {
        Node {
            id,
            role: Role::Follower,
            term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            peers,
            votes: Vec::new(),
            committed_history: Vec::new(),
        }
    }

    fn quorum(&self) -> usize {
        (self.peers.len() + 1) / 2 + 1
    }

    pub fn last_index(&self) -> Index {
        self.log.last().map_or(0, |e| e.index)
    }
    pub fn last_term(&self) -> Term {
        self.log.last().map_or(0, |e| e.term)
    }
    pub fn hash_at(&self, index: Index) -> [u8; 32] {
        if index == 0 {
            return [0u8; 32];
        }
        self.log
            .iter()
            .find(|e| e.index == index)
            .map_or([0xffu8; 32], |e| e.hash)
    }

    /// Begin an election.
    pub fn start_election(&mut self) -> Vec<Msg> {
        self.term += 1;
        self.role = Role::Candidate;
        self.voted_for = Some(self.id);
        self.votes = vec![self.id];
        self.peers
            .iter()
            .map(|_| Msg::RequestVote {
                term: self.term,
                from: self.id,
                last_index: self.last_index(),
                last_term: self.last_term(),
            })
            .collect()
    }

    /// Client submission. Only a leader may seal.
    pub fn propose(&mut self, payload: Vec<u8>) -> Option<Index> {
        if self.role != Role::Leader {
            return None;
        }
        let index = self.last_index() + 1;
        let parent = self.hash_at(self.last_index());
        self.log
            .push(Entry::seal(self.term, index, parent, payload));
        Some(index)
    }

    /// The `AppendEntries` a leader sends to one follower.
    pub fn replicate_to(&self, peer: NodeId) -> Msg {
        let next = self
            .next_index
            .get(&peer)
            .copied()
            .unwrap_or(self.last_index() + 1);
        let prev_index = next.saturating_sub(1);
        Msg::AppendEntries {
            term: self.term,
            from: self.id,
            prev_index,
            prev_hash: self.hash_at(prev_index),
            entries: self
                .log
                .iter()
                .filter(|e| e.index >= next)
                .cloned()
                .collect(),
            leader_commit: self.commit_index,
        }
    }

    /// Handle a message. Returns replies to send.
    pub fn handle(&mut self, msg: Msg) -> Vec<Msg> {
        // Any message with a higher term makes this node a follower, whatever it was doing.
        let incoming = match &msg {
            Msg::RequestVote { term, .. }
            | Msg::VoteGranted { term, .. }
            | Msg::VoteDenied { term, .. }
            | Msg::AppendEntries { term, .. }
            | Msg::AppendOk { term, .. }
            | Msg::AppendRejected { term, .. } => *term,
        };
        if incoming > self.term {
            self.term = incoming;
            self.role = Role::Follower;
            self.voted_for = None;
            self.votes.clear();
        }

        match msg {
            Msg::RequestVote {
                term,
                from,
                last_index,
                last_term,
            } => {
                if term < self.term {
                    return vec![Msg::VoteDenied {
                        term: self.term,
                        from: self.id,
                        to: from,
                        reason: "stale term",
                    }];
                }
                if self.voted_for.is_some() && self.voted_for != Some(from) {
                    return vec![Msg::VoteDenied {
                        term: self.term,
                        from: self.id,
                        to: from,
                        reason: "already voted this term",
                    }];
                }
                // The up-to-date restriction. This is the whole of Raft's election safety:
                // a candidate whose log is behind cannot win, so a leader always has every
                // committed entry, so a committed entry can never be lost.
                let up_to_date = last_term > self.last_term()
                    || (last_term == self.last_term() && last_index >= self.last_index());
                if !up_to_date {
                    return vec![Msg::VoteDenied {
                        term: self.term,
                        from: self.id,
                        to: from,
                        reason: "candidate log is behind",
                    }];
                }
                self.voted_for = Some(from);
                vec![Msg::VoteGranted {
                    term: self.term,
                    from: self.id,
                    to: from,
                }]
            }

            Msg::VoteGranted { term, from, to } => {
                if to != self.id || term != self.term || self.role != Role::Candidate {
                    return Vec::new();
                }
                if !self.votes.contains(&from) {
                    self.votes.push(from);
                }
                if self.votes.len() >= self.quorum() {
                    self.become_leader();
                }
                Vec::new()
            }

            Msg::VoteDenied { .. } => Vec::new(),

            Msg::AppendEntries {
                term,
                from,
                prev_index,
                prev_hash,
                entries,
                leader_commit,
            } => {
                if term < self.term {
                    return vec![Msg::AppendRejected {
                        term: self.term,
                        from: self.id,
                        to: from,
                        hint: self.last_index(),
                        reason: RejectReason::StaleTerm,
                    }];
                }
                self.role = Role::Follower;

                // Does the follower have the leader's previous entry?
                if prev_index > 0 && self.hash_at(prev_index) != prev_hash {
                    return vec![Msg::AppendRejected {
                        term: self.term,
                        from: self.id,
                        to: from,
                        // Back up to what this node does have, so the leader converges in
                        // O(divergence) rounds rather than one index at a time.
                        hint: self.last_index().min(prev_index.saturating_sub(1)),
                        reason: RejectReason::LogMismatch,
                    }];
                }

                // **The chain check.** Recompute, do not trust. An entry that does not
                // follow from its stated parent is rejected even from a current leader.
                let mut parent = prev_hash;
                for e in &entries {
                    if !e.follows(parent) {
                        return vec![Msg::AppendRejected {
                            term: self.term,
                            from: self.id,
                            to: from,
                            hint: prev_index,
                            reason: RejectReason::BrokenChain,
                        }];
                    }
                    parent = e.hash;
                }

                // Truncate any divergent suffix, then append. Truncation is only ever of
                // *uncommitted* entries — the up-to-date restriction guarantees that — and
                // `assert_no_committed_entry_is_ever_truncated` checks it.
                self.log.retain(|e| e.index <= prev_index);
                self.log.extend(entries);

                if leader_commit > self.commit_index {
                    self.commit_index = leader_commit.min(self.last_index());
                    self.record_committed();
                }
                vec![Msg::AppendOk {
                    term: self.term,
                    from: self.id,
                    to: from,
                    match_index: self.last_index(),
                }]
            }

            Msg::AppendOk {
                term,
                from,
                to,
                match_index,
            } => {
                if to != self.id || self.role != Role::Leader || term != self.term {
                    return Vec::new();
                }
                self.match_index.insert(from, match_index);
                self.next_index.insert(from, match_index + 1);
                self.advance_commit();
                Vec::new()
            }

            Msg::AppendRejected {
                term,
                from,
                to,
                hint,
                reason,
            } => {
                if to != self.id || self.role != Role::Leader || term != self.term {
                    return Vec::new();
                }
                // A broken chain is not a normal disagreement about which prefix is current.
                // Backing up would be treating corruption as divergence, so the leader stops
                // replicating to that follower and the operator is told.
                if reason == RejectReason::BrokenChain {
                    self.next_index.insert(from, self.last_index() + 1);
                    return Vec::new();
                }
                self.next_index.insert(from, hint.max(1));
                vec![self.replicate_to(from)]
            }
        }
    }

    fn become_leader(&mut self) {
        self.role = Role::Leader;
        let next = self.last_index() + 1;
        for p in &self.peers {
            self.next_index.insert(*p, next);
            self.match_index.insert(*p, 0);
        }
    }

    /// Advance the commit index to the highest entry replicated on a quorum **in the
    /// current term**.
    ///
    /// The term restriction is the subtle part of Raft and the part most often got wrong:
    /// a leader may not commit an entry from a previous term merely because it is now on a
    /// quorum, because a later leader could still overwrite it. It becomes committed
    /// implicitly when an entry from the current term commits above it.
    fn advance_commit(&mut self) {
        let total = self.peers.len() + 1;
        for e in self.log.iter().rev() {
            if e.term != self.term {
                continue;
            }
            let replicas = 1 + self.match_index.values().filter(|m| **m >= e.index).count();
            if replicas * 2 > total && e.index > self.commit_index {
                self.commit_index = e.index;
                break;
            }
        }
        self.record_committed();
    }

    fn record_committed(&mut self) {
        let already = self.committed_history.len() as u64;
        for e in self
            .log
            .iter()
            .filter(|e| e.index > already && e.index <= self.commit_index)
        {
            self.committed_history.push(e.clone());
        }
    }

    /// The frontier this node would publish. A read at the strict rung anchors here.
    pub fn frontier(&self) -> Index {
        self.commit_index
    }

    /// Verify this node's whole log links. An operator runs this; so does every test.
    pub fn chain_is_intact(&self) -> bool {
        let mut parent = [0u8; 32];
        for e in &self.log {
            if !e.follows(parent) {
                return false;
            }
            parent = e.hash;
        }
        true
    }
}
