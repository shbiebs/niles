//! Cross-shard commit: a transaction spanning two or more ledger groups.
//!
//! # The problem, stated precisely
//!
//! Each ledger group agrees on its own total order. A transaction touching accounts in two
//! groups must appear **atomic and at one point in time** to every reader of either group,
//! or the conservation invariant is observable as broken: a reader catching group A after
//! the debit and group B before the credit sees money that has left one account and
//! arrived nowhere.
//!
//! That is not hypothetical. It is the single most common way a sharded ledger loses the
//! property this entire thesis is about, and it is why §8.6 exists.
//!
//! # Why two-phase commit, and why its usual objection does not apply here
//!
//! The standard objection to 2PC is that it is *blocking*: if the coordinator fails after
//! prepare and before commit, participants hold their fragments indefinitely. That
//! objection assumes the coordinator's decision lives only in the coordinator.
//!
//! Here it does not. The coordinator **is a ledger group**, so its decision is a replicated
//! log entry committed by quorum before it is sent. A coordinator that dies after deciding
//! has already durably recorded the decision on a majority, and any successor reads it out
//! of the log rather than re-deciding. 2PC over replicated participants with a replicated
//! coordinator is not the blocking protocol the textbook describes — and the reason that
//! construction is available here is that the write path was already a replicated log for
//! reasons that had nothing to do with sharding.
//!
//! # What the epoch adds
//!
//! A distributed transaction needs a single point on a timeline that every participant
//! agrees on, so a read can be anchored *before* or *after* it and never inside it. Here
//! that point already exists and is already published: participants prepare at their own
//! epochs, the coordinator picks a **commit epoch** strictly greater than every prepared
//! epoch, and each participant seals the transaction at that epoch.
//!
//! The consequence is the property that matters: **a read anchored at any epoch sees either
//! all of a cross-shard transaction or none of it**, in every group, because the transaction
//! occupies the same epoch number in all of them. There is no window. This design needs no
//! synchronised-clock interval and no commit-wait to get that, because the ordering is by an
//! agreed integer rather than by a physical clock — which is the one place the epoch design
//! pays a dividend a timestamp design does not.
//!
//! # What is not here
//!
//! Participant recovery after coordinator loss is a decision *lookup*, not a full recovery
//! protocol with timeouts and coordinator election. The read-only and single-shard
//! one-phase optimisations are absent. Neither affects safety; both affect latency, and
//! pretending otherwise is what §7.5 exists to prevent.

use crate::{Index, Term};
use std::collections::{BTreeMap, BTreeSet};

/// A ledger group identifier.
pub type ShardId = u16;

/// A distributed transaction identifier.
pub type Xid = u64;

/// One shard's part of a cross-shard transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub shard: ShardId,
    /// The per-currency net of this fragment. **A fragment need not conserve on its own** —
    /// only the whole transaction must — which is exactly why the check has to be made by
    /// the coordinator across fragments rather than by each shard in isolation.
    pub net: BTreeMap<String, i128>,
    pub payload: Vec<u8>,
}

/// What a participant answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vote {
    /// Prepared and durable at this epoch. The participant now holds the fragment and will
    /// honour either outcome.
    Prepared { shard: ShardId, at_epoch: Index },
    /// Refused, with the reason. A refusal is as durable as a preparation: a participant
    /// that said no must not later say yes, because the coordinator may already have
    /// aborted on its word.
    Refused { shard: ShardId, reason: RefuseReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefuseReason {
    /// The idempotency key already committed here.
    Duplicate,
    /// The shard has no quorum and cannot make its prepare durable.
    NoQuorum,
    /// The fragment's admission rules rejected it locally.
    Inadmissible,
}

/// The coordinator's decision. **Recorded in the coordinator's own replicated log before it
/// is sent**, which is what removes the blocking objection to 2PC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Commit { xid: Xid, at_epoch: Index },
    Abort { xid: Xid, why: AbortReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbortReason {
    /// At least one participant refused.
    ParticipantRefused,
    /// **The fragments do not conserve when summed.** The check only the coordinator can
    /// make: each shard sees a fragment that may legitimately be unbalanced, and only the
    /// union has to be zero.
    DoesNotConserve { currency: String, net: i128 },
    /// A participant was unreachable within the prepare round.
    Unreachable { shard: ShardId },
}

/// The coordinator's state machine.
#[derive(Debug, Clone)]
pub struct Coordinator {
    pub xid: Xid,
    pub term: Term,
    pub participants: BTreeSet<ShardId>,
    pub fragments: Vec<Fragment>,
    votes: BTreeMap<ShardId, Vote>,
    decided: Option<Decision>,
    /// Whether the decision is durable on the coordinator's own quorum. **No decision may
    /// be sent before this is true**, and [`Coordinator::send_decision`] enforces it.
    decision_durable: bool,
}

impl Coordinator {
    pub fn new(xid: Xid, term: Term, fragments: Vec<Fragment>) -> Coordinator {
        Coordinator {
            xid,
            term,
            participants: fragments.iter().map(|f| f.shard).collect(),
            fragments,
            votes: BTreeMap::new(),
            decided: None,
            decision_durable: false,
        }
    }

    /// **The conservation check across fragments.**
    ///
    /// The static solver of Contribution 4 proves conservation within a program's
    /// transaction. Sharding splits that transaction across machines, and this is where the
    /// same invariant is re-established at runtime over the pieces. Omitting it would let a
    /// sharded deployment silently lose the property the single-node design proves, which
    /// makes this the most important function in the module.
    pub fn conserves(&self) -> Result<(), (String, i128)> {
        let mut total: BTreeMap<String, i128> = BTreeMap::new();
        for f in &self.fragments {
            for (cur, amt) in &f.net {
                *total.entry(cur.clone()).or_insert(0) += amt;
            }
        }
        for (cur, net) in total {
            if net != 0 {
                return Err((cur, net));
            }
        }
        Ok(())
    }

    /// Record a vote. Idempotent: a repeated vote from a participant is the same vote.
    pub fn record(&mut self, v: Vote) {
        let shard = match &v {
            Vote::Prepared { shard, .. } | Vote::Refused { shard, .. } => *shard,
        };
        self.votes.entry(shard).or_insert(v);
    }

    pub fn all_voted(&self) -> bool {
        self.participants.iter().all(|s| self.votes.contains_key(s))
    }

    /// Decide, once every participant has voted.
    ///
    /// The commit epoch is **strictly greater than every prepared epoch**, so no participant
    /// can have published anything at the commit epoch before the decision existed. That is
    /// what makes "a read at any epoch sees all or none" true rather than merely likely.
    pub fn decide(&mut self) -> Option<Decision> {
        if self.decided.is_some() {
            return self.decided.clone();
        }
        if !self.all_voted() {
            return None;
        }
        // Conservation first: an unbalanced transaction must abort even if every
        // participant is individually willing, because each shard only saw its own piece.
        let d = if let Err((cur, net)) = self.conserves() {
            Decision::Abort { xid: self.xid, why: AbortReason::DoesNotConserve { currency: cur, net } }
        } else if self.votes.values().any(|v| matches!(v, Vote::Refused { .. })) {
            Decision::Abort { xid: self.xid, why: AbortReason::ParticipantRefused }
        } else {
            let at = self
                .votes
                .values()
                .filter_map(|v| match v {
                    Vote::Prepared { at_epoch, .. } => Some(*at_epoch),
                    _ => None,
                })
                .max()
                .unwrap_or(0)
                + 1;
            Decision::Commit { xid: self.xid, at_epoch: at }
        };
        self.decided = Some(d.clone());
        Some(d)
    }

    /// Make the decision durable on the coordinator's own quorum. In a deployment this is
    /// an ordinary proposal on the coordinator's ledger group.
    pub fn persist_decision(&mut self) {
        if self.decided.is_some() {
            self.decision_durable = true;
        }
    }

    /// Hand the decision to the participants.
    ///
    /// Refuses to do so before it is durable. The same ordering rule the single-node write
    /// path enforces between `fsync` and the frontier, one level up: **decide, persist, then
    /// tell anyone.** A decision announced before it is recorded is a decision that can be
    /// forgotten and re-made differently — which is how a distributed transaction commits on
    /// one shard and aborts on another.
    pub fn send_decision(&self) -> Option<Decision> {
        if !self.decision_durable {
            return None;
        }
        self.decided.clone()
    }

    /// A successor coordinator reads the decision out of the log rather than re-deciding.
    /// This is the whole answer to the blocking objection.
    pub fn recover(log: &[Decision], xid: Xid) -> Option<Decision> {
        log.iter()
            .find(|d| match d {
                Decision::Commit { xid: x, .. } | Decision::Abort { xid: x, .. } => *x == xid,
            })
            .cloned()
    }
}

/// A participant shard's state for one cross-shard transaction.
#[derive(Debug, Clone)]
pub struct Participant {
    pub shard: ShardId,
    pub xid: Xid,
    /// Set once prepared. A prepared fragment is durable and immovable: the participant has
    /// promised to honour whichever decision arrives.
    prepared_at: Option<Index>,
    refused: Option<RefuseReason>,
    outcome: Option<Decision>,
    seen_idem: BTreeSet<String>,
}

impl Participant {
    pub fn new(shard: ShardId, xid: Xid) -> Participant {
        Participant {
            shard,
            xid,
            prepared_at: None,
            refused: None,
            outcome: None,
            seen_idem: BTreeSet::new(),
        }
    }

    pub fn with_committed_keys(mut self, keys: &[&str]) -> Participant {
        self.seen_idem = keys.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Prepare: make the fragment durable locally at this shard's own next epoch, and
    /// promise to honour either outcome.
    pub fn prepare(&mut self, idem_key: &str, local_epoch: Index, has_quorum: bool) -> Vote {
        // A participant that already answered must answer the same way. A prepare that
        // could be retracted is not a prepare.
        if let Some(e) = self.prepared_at {
            return Vote::Prepared { shard: self.shard, at_epoch: e };
        }
        if let Some(r) = self.refused {
            return Vote::Refused { shard: self.shard, reason: r };
        }
        let reason = if self.seen_idem.contains(idem_key) {
            Some(RefuseReason::Duplicate)
        } else if !has_quorum {
            Some(RefuseReason::NoQuorum)
        } else {
            None
        };
        match reason {
            Some(r) => {
                self.refused = Some(r);
                Vote::Refused { shard: self.shard, reason: r }
            }
            None => {
                self.prepared_at = Some(local_epoch);
                Vote::Prepared { shard: self.shard, at_epoch: local_epoch }
            }
        }
    }

    /// Apply the coordinator's decision. Idempotent, because the decision may be delivered
    /// more than once and a participant that applied it twice would double the movement.
    pub fn apply(&mut self, d: Decision) {
        if self.outcome.is_none() {
            self.outcome = Some(d);
        }
    }

    pub fn is_prepared(&self) -> bool {
        self.prepared_at.is_some()
    }

    /// The epoch this shard sealed the transaction at. `None` while undecided or aborted.
    pub fn committed_at(&self) -> Option<Index> {
        match self.outcome {
            Some(Decision::Commit { at_epoch, .. }) => Some(at_epoch),
            _ => None,
        }
    }

    /// **The visibility rule.** A read anchored at `anchor` sees this transaction iff its
    /// commit epoch is at or below the anchor. Because every participant uses the *same*
    /// commit epoch, this answer is identical on every shard — which is precisely what makes
    /// a cross-shard transaction atomic to a reader.
    pub fn visible_at(&self, anchor: Index) -> bool {
        self.committed_at().map_or(false, |e| e <= anchor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frag(shard: ShardId, cur: &str, net: i128) -> Fragment {
        Fragment {
            shard,
            net: [(cur.to_string(), net)].into_iter().collect(),
            payload: vec![shard as u8],
        }
    }

    fn run(
        fragments: Vec<Fragment>,
        epochs: &[(ShardId, Index)],
        quorum: bool,
    ) -> (Coordinator, Vec<Participant>) {
        let mut c = Coordinator::new(1, 1, fragments);
        let mut ps: Vec<Participant> =
            c.participants.iter().map(|s| Participant::new(*s, 1)).collect();
        for p in &mut ps {
            let e = epochs.iter().find(|(s, _)| *s == p.shard).map(|(_, e)| *e).unwrap_or(10);
            c.record(p.prepare("k", e, quorum));
        }
        c.decide();
        c.persist_decision();
        if let Some(d) = c.send_decision() {
            for p in &mut ps {
                p.apply(d.clone());
            }
        }
        (c, ps)
    }

    #[test]
    fn a_balanced_cross_shard_transaction_commits_at_one_epoch_everywhere() {
        // The property that makes it atomic to a reader: the same epoch on every shard.
        let (_c, ps) = run(vec![frag(1, "usd", -100), frag(2, "usd", 100)], &[(1, 10), (2, 40)], true);
        let epochs: Vec<Option<Index>> = ps.iter().map(|p| p.committed_at()).collect();
        assert!(epochs.iter().all(|e| e.is_some()), "both shards must commit: {epochs:?}");
        assert_eq!(epochs[0], epochs[1], "and at the SAME epoch, or a reader can see half of it");
        assert_eq!(epochs[0], Some(41), "strictly above every prepared epoch");
    }

    #[test]
    fn a_read_sees_all_of_it_or_none_of_it_at_every_anchor() {
        // Swept across the whole epoch range: there is no anchor at which one shard shows
        // the transaction and the other does not. That window is the failure this protocol
        // exists to remove, so it gets an exhaustive check rather than a spot one.
        let (_c, ps) = run(vec![frag(1, "usd", -100), frag(2, "usd", 100)], &[(1, 7), (2, 55)], true);
        for anchor in 0..80 {
            let seen: Vec<bool> = ps.iter().map(|p| p.visible_at(anchor)).collect();
            assert!(
                seen.iter().all(|x| *x) || seen.iter().all(|x| !*x),
                "at anchor {anchor} the shards disagree: {seen:?}"
            );
        }
    }

    #[test]
    fn fragments_that_do_not_sum_to_zero_abort_even_when_every_shard_agrees() {
        // The check only the coordinator can make. Each shard sees a fragment that is
        // individually fine; only the union is unbalanced. Without this, sharding would
        // silently lose the invariant the single-node design proves.
        let (c, ps) = run(vec![frag(1, "usd", -100), frag(2, "usd", 60)], &[(1, 10), (2, 10)], true);
        match c.send_decision() {
            Some(Decision::Abort { why: AbortReason::DoesNotConserve { currency, net }, .. }) => {
                assert_eq!(currency, "usd");
                assert_eq!(net, -40);
            }
            other => panic!("expected a conservation abort, got {other:?}"),
        }
        assert!(ps.iter().all(|p| p.committed_at().is_none()), "nothing may commit");
    }

    #[test]
    fn conservation_is_checked_per_currency_across_shards() {
        // Two shards, two currencies, netting to zero *in total* but not per currency — the
        // same mistake the single-node solver rejects, made across machines.
        let mut a = frag(1, "usd", -100);
        a.net.insert("eur".into(), 0);
        let b = frag(2, "eur", 100);
        let (c, _) = run(vec![a, b], &[(1, 5), (2, 5)], true);
        assert!(matches!(
            c.send_decision(),
            Some(Decision::Abort { why: AbortReason::DoesNotConserve { .. }, .. })
        ));
    }

    #[test]
    fn one_refusal_aborts_the_whole_transaction() {
        let mut c = Coordinator::new(9, 1, vec![frag(1, "usd", -50), frag(2, "usd", 50)]);
        let mut p1 = Participant::new(1, 9);
        let mut p2 = Participant::new(2, 9).with_committed_keys(&["k"]);
        c.record(p1.prepare("k", 10, true));
        c.record(p2.prepare("k", 10, true));
        c.decide();
        c.persist_decision();
        assert!(matches!(
            c.send_decision(),
            Some(Decision::Abort { why: AbortReason::ParticipantRefused, .. })
        ));
    }

    #[test]
    fn a_decision_is_never_sent_before_it_is_durable() {
        // The same ordering rule as durable-before-visible, one level up: decide, persist,
        // then tell anyone. A decision announced before it is recorded can be forgotten and
        // re-made differently, which is how a distributed transaction commits on one shard
        // and aborts on another.
        let mut c = Coordinator::new(3, 1, vec![frag(1, "usd", -1), frag(2, "usd", 1)]);
        let mut p1 = Participant::new(1, 3);
        let mut p2 = Participant::new(2, 3);
        c.record(p1.prepare("k", 4, true));
        c.record(p2.prepare("k", 4, true));
        assert!(c.decide().is_some(), "the decision exists");
        assert!(c.send_decision().is_none(), "but must not be sendable before it is persisted");
        c.persist_decision();
        assert!(c.send_decision().is_some());
    }

    #[test]
    fn a_prepared_participant_cannot_change_its_answer() {
        let mut p = Participant::new(1, 1);
        let first = p.prepare("k", 12, true);
        // A retry, or a duplicate delivery, must give the same answer at the same epoch. A
        // prepare that could be retracted is not a prepare.
        let again = p.prepare("k", 99, false);
        assert_eq!(first, again);
        assert_eq!(p.committed_at(), None);
        assert!(p.is_prepared());
    }

    #[test]
    fn a_refusal_is_as_durable_as_a_preparation() {
        let mut p = Participant::new(1, 1).with_committed_keys(&["k"]);
        let first = p.prepare("k", 5, true);
        assert!(matches!(first, Vote::Refused { reason: RefuseReason::Duplicate, .. }));
        // Even with the duplicate condition gone, a participant that said no must not later
        // say yes: the coordinator may already have aborted on its word.
        p.seen_idem.clear();
        assert_eq!(p.prepare("k", 5, true), first);
    }

    #[test]
    fn applying_a_decision_twice_does_not_double_the_movement() {
        let mut p = Participant::new(1, 1);
        p.prepare("k", 3, true);
        p.apply(Decision::Commit { xid: 1, at_epoch: 9 });
        p.apply(Decision::Commit { xid: 1, at_epoch: 77 });
        assert_eq!(p.committed_at(), Some(9), "a redelivered decision is the same decision");
    }

    #[test]
    fn a_successor_coordinator_reads_the_decision_rather_than_re_deciding() {
        // The answer to 2PC's blocking objection: the coordinator IS a ledger group, so its
        // decision is a replicated log entry that a successor reads out.
        let log = vec![
            Decision::Abort { xid: 5, why: AbortReason::ParticipantRefused },
            Decision::Commit { xid: 7, at_epoch: 42 },
        ];
        assert_eq!(Coordinator::recover(&log, 7), Some(Decision::Commit { xid: 7, at_epoch: 42 }));
        assert!(matches!(Coordinator::recover(&log, 5), Some(Decision::Abort { .. })));
        assert_eq!(Coordinator::recover(&log, 99), None, "an unknown xid has no decision to recover");
    }

    #[test]
    fn a_shard_without_quorum_refuses_rather_than_promising_what_it_cannot_keep() {
        let mut p = Participant::new(4, 1);
        assert!(matches!(
            p.prepare("k", 8, false),
            Vote::Refused { reason: RefuseReason::NoQuorum, .. }
        ));
        assert!(!p.is_prepared());
    }

    #[test]
    fn the_commit_epoch_clears_every_prepared_epoch_however_skewed_the_shards() {
        // Shards run at wildly different rates; the commit epoch must clear all of them, or
        // a fast shard could already have published something at the commit epoch.
        for spread in [(1u64, 2u64), (1, 1_000), (999, 1_000), (5, 5)] {
            let (_c, ps) = run(
                vec![frag(1, "usd", -1), frag(2, "usd", 1)],
                &[(1, spread.0), (2, spread.1)],
                true,
            );
            let at = ps[0].committed_at().unwrap();
            assert!(at > spread.0 && at > spread.1, "commit epoch {at} does not clear {spread:?}");
        }
    }
}
