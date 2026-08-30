//! The epoch sequencer: the concurrent write path.
//!
//! Many threads submit transactions; **one** thread seals them into epochs, appends them to
//! a durable segment, forces them to disk, and only then advances the visibility frontier.
//!
//! # Why a single sealer, and why that is not a bottleneck argument
//!
//! The base is an append-only, totally ordered history. A total order is a single sequence,
//! and producing one from many concurrent submitters requires exactly one point of
//! agreement. Distributing that point is what consensus is for, and is a separate problem
//! from this one — within a node the honest design is one sealer and a queue, because
//! pretending otherwise would mean either a partial order (which is not a ledger) or
//! hidden coordination (which is the same bottleneck with more moving parts).
//!
//! What *is* worth engineering is that the sealer amortises the expensive part. An `fsync`
//! costs the same whether it commits one transaction or five hundred, so the sealer drains
//! everything waiting, seals it as **one epoch**, and syncs once. Group commit is the
//! reason a single-sealer design is not the throughput ceiling it appears to be: the cost
//! per transaction falls as concurrency rises, which is the opposite of the usual
//! contention story and is why this shape is the one production ledgers converge on.
//!
//! # What each submitter is promised
//!
//! `submit` returns when the transaction's epoch is **durable and visible**, or with the
//! reason it was rejected. It never returns "probably". A caller that got `Ok(epoch)` can
//! tell a customer the money moved, and a crash one microsecond later does not change that.
//!
//! # What this is not
//!
//! Single node. Cross-node agreement is the consensus problem and is not solved here; the
//! `consensus` module states the interface that would carry this design across nodes and
//! is explicit that it is a specification, not an implementation.

use crate::frontiers::Frontier;
use crate::segment::{Segment, SyncPolicy};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// A submitted unit of work. Opaque bytes at this layer: the sequencer orders and commits;
/// it does not interpret. Admission rules — conservation, idempotency, hold resolution —
/// run in `admission` *before* a transaction reaches the queue, because a transaction that
/// will be rejected should never occupy a slot in an epoch.
#[derive(Debug, Clone)]
pub struct Txn {
    pub idem_key: String,
    pub payload: Vec<u8>,
}

/// Why a submission did not commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejected {
    /// This idempotency key already committed, at the epoch named. **Not an error**: the
    /// caller retried, and the correct response is the original outcome, not a second
    /// transaction. Returning the epoch is what makes a retry safe rather than doubling.
    Duplicate { at_epoch: u64 },
    /// The sealer stopped.
    ShuttingDown,
    /// The write path failed. The epoch did not commit and the frontier did not move.
    Io(String),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SequencerStats {
    pub epochs_sealed: u64,
    pub txns_committed: u64,
    pub fsyncs: u64,
    pub duplicates_absorbed: u64,
    /// The largest number of transactions sealed into one epoch. The group-commit win.
    pub max_batch: u64,
}

impl SequencerStats {
    /// Transactions per fsync. The figure that decides whether a single-sealer design is
    /// a bottleneck or a batching opportunity.
    pub fn txns_per_fsync(&self) -> f64 {
        if self.fsyncs == 0 {
            0.0
        } else {
            self.txns_committed as f64 / self.fsyncs as f64
        }
    }
}

struct Request {
    txn: Txn,
    reply: Sender<Result<u64, Rejected>>,
}

/// The handle callers hold.
pub struct Sequencer {
    tx: Option<Sender<Request>>,
    frontier: Arc<Frontier>,
    stats: Arc<Mutex<SequencerStats>>,
    sealer: Option<JoinHandle<()>>,
}

impl Sequencer {
    /// Start a sequencer over a segment.
    pub fn start(mut segment: Segment, frontier: Arc<Frontier>, policy: SyncPolicy) -> Sequencer {
        let (tx, rx): (Sender<Request>, Receiver<Request>) = channel();
        let stats = Arc::new(Mutex::new(SequencerStats::default()));
        let (f, s) = (Arc::clone(&frontier), Arc::clone(&stats));

        let sealer = std::thread::spawn(move || {
            // The idempotency window. In a durable deployment this is recovered from the
            // segment at start-up; here it is in memory, and that limitation is stated
            // rather than hidden, because an idempotency window that does not survive a
            // restart does not protect against the retry that a restart provokes.
            let mut seen: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

            while let Ok(first) = rx.recv() {
                // Drain everything already waiting: one fsync will commit all of it.
                let mut batch = vec![first];
                while let Ok(more) = rx.try_recv() {
                    batch.push(more);
                    if batch.len() >= 4096 {
                        break;
                    }
                }

                // Absorb duplicates before sealing. A retry must return the *original*
                // epoch: telling a caller "committed at a new epoch" would be a second
                // transaction wearing the first one's name.
                //
                // Two checks, not one, and the second is the one that is easy to omit.
                // `seen` catches a retry of an *already committed* key. But a client that
                // retries fast enough — or a client and a proxy retrying together — can put
                // both copies into the *same* drain, where neither is in `seen` yet. The
                // first version of this sealer checked only `seen` and committed both, and
                // the test for it failed roughly two runs in three: a genuine race, not a
                // flaky test, and the failure mode is a duplicated payment rather than an
                // error. Within-batch keys are therefore tracked as the batch is scanned.
                let mut fresh: Vec<Request> = Vec::with_capacity(batch.len());
                let mut in_batch: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                let mut deferred: Vec<(usize, Sender<Result<u64, Rejected>>)> = Vec::new();
                for req in batch {
                    if let Some(e) = seen.get(&req.txn.idem_key) {
                        // Account for it *before* replying. The reply is what makes the
                        // outcome observable to the caller, and a caller who can see the
                        // outcome must be able to see the state that produced it. This is
                        // the same ordering rule as durable-before-visible, one level up,
                        // and getting it backwards here made the idempotency test fail
                        // about one run in three — a real race, in the instrument rather
                        // than in the mechanism, but the same class of mistake.
                        s.lock().unwrap().duplicates_absorbed += 1;
                        let _ = req.reply.send(Err(Rejected::Duplicate { at_epoch: *e }));
                    } else if let Some(idx) = in_batch.get(&req.txn.idem_key) {
                        // A second copy of a key already in this batch. Its answer is the
                        // epoch the first copy is about to get, so it waits for it.
                        deferred.push((*idx, req.reply));
                        s.lock().unwrap().duplicates_absorbed += 1;
                    } else {
                        in_batch.insert(req.txn.idem_key.clone(), fresh.len());
                        fresh.push(req);
                    }
                }
                if fresh.is_empty() {
                    continue;
                }

                // One epoch for the whole batch.
                let mut payload = Vec::new();
                payload.extend_from_slice(&(fresh.len() as u32).to_le_bytes());
                for r in &fresh {
                    payload.extend_from_slice(&(r.txn.payload.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&r.txn.payload);
                }

                match segment.append(payload) {
                    Ok(rec) => {
                        let epoch = rec.epoch + 1; // epochs are 1-based on the frontier
                        f.seal(epoch);
                        // Force the epoch to stable storage before the frontier moves.
                        //
                        // Only under a *batching* policy: `Always` already synced inside
                        // `append`, and syncing again would pay the cost twice. The first
                        // version did exactly that, and the benchmark caught it — it read
                        // 0.5 transactions per fsync at one thread, which is a figure with
                        // no sensible interpretation and was the clue.
                        if matches!(policy, SyncPolicy::Every(_)) {
                            if let Err(e) = segment.sync() {
                                for (_, reply) in deferred {
                                    let _ = reply.send(Err(Rejected::Io(e.to_string())));
                                }
                                for r in fresh {
                                    let _ = r.reply.send(Err(Rejected::Io(e.to_string())));
                                }
                                continue;
                            }
                        }
                        // Durable. Now, and only now, publish.
                        f.publish(epoch);
                        {
                            let mut st = s.lock().unwrap();
                            st.epochs_sealed += 1;
                            st.txns_committed += fresh.len() as u64;
                            st.fsyncs = segment.fsyncs;
                            st.max_batch = st.max_batch.max(fresh.len() as u64);
                        }
                        for r in &fresh {
                            seen.insert(r.txn.idem_key.clone(), epoch);
                        }
                        // The deferred copies get the same epoch their original got, and
                        // are told it was a duplicate — which is what makes a retry safe.
                        for (_, reply) in deferred {
                            let _ = reply.send(Err(Rejected::Duplicate { at_epoch: epoch }));
                        }
                        for r in fresh {
                            let _ = r.reply.send(Ok(epoch));
                        }
                    }
                    Err(e) => {
                        for (_, reply) in deferred {
                            let _ = reply.send(Err(Rejected::Io(e.to_string())));
                        }
                        for r in fresh {
                            let _ = r.reply.send(Err(Rejected::Io(e.to_string())));
                        }
                    }
                }
            }
        });

        Sequencer { tx: Some(tx), frontier, stats, sealer: Some(sealer) }
    }

    /// Submit and wait for the transaction to be durable and visible.
    pub fn submit(&self, txn: Txn) -> Result<u64, Rejected> {
        let (reply, rx) = channel();
        let Some(tx) = &self.tx else { return Err(Rejected::ShuttingDown) };
        tx.send(Request { txn, reply }).map_err(|_| Rejected::ShuttingDown)?;
        rx.recv().map_err(|_| Rejected::ShuttingDown)?
    }

    pub fn frontier(&self) -> &Arc<Frontier> {
        &self.frontier
    }

    pub fn stats(&self) -> SequencerStats {
        *self.stats.lock().unwrap()
    }

    /// Stop the sealer and wait for it. Everything already submitted commits first.
    pub fn shutdown(mut self) -> SequencerStats {
        self.tx.take();
        if let Some(h) = self.sealer.take() {
            let _ = h.join();
        }
        *self.stats.lock().unwrap()
    }
}

impl Drop for Sequencer {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(h) = self.sealer.take() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::recover;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("niles-seq-{name}-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    fn seq(p: &PathBuf, policy: SyncPolicy) -> Sequencer {
        let (segment, _) = Segment::open(p, policy).unwrap();
        Sequencer::start(segment, Frontier::new(), policy)
    }

    #[test]
    fn a_commit_returns_only_once_it_is_durable_and_visible() {
        let p = tmp("durable");
        let s = seq(&p, SyncPolicy::Always);
        let e = s.submit(Txn { idem_key: "a".into(), payload: b"x".to_vec() }).unwrap();
        assert!(e >= 1);
        assert_eq!(s.frontier().visible(), e, "the frontier must be at the returned epoch");
        assert_eq!(s.frontier().pending(), 0, "nothing sealed-but-unpublished may remain");
        s.shutdown();
        // And it really is on disk.
        let rec = recover(&p).unwrap();
        assert_eq!(rec.records.len(), 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn concurrent_submitters_all_commit_and_the_order_is_total() {
        // The concurrency claim, tested rather than asserted: many threads, every
        // transaction committed exactly once, and one linear order over epochs.
        let p = tmp("concurrent");
        let s = Arc::new(seq(&p, SyncPolicy::Always));
        let mut handles = Vec::new();
        for t in 0..8u64 {
            let s = Arc::clone(&s);
            handles.push(std::thread::spawn(move || {
                let mut mine = Vec::new();
                for i in 0..50u64 {
                    let k = format!("t{t}-{i}");
                    mine.push(s.submit(Txn { idem_key: k, payload: vec![t as u8, i as u8] }).unwrap());
                }
                mine
            }));
        }
        let all: Vec<u64> = handles.into_iter().flat_map(|h| h.join().unwrap()).collect();
        assert_eq!(all.len(), 400);

        let st = s.stats();
        assert_eq!(st.txns_committed, 400, "every transaction must commit exactly once");
        // Epochs form a prefix of the integers with no gaps: a total order.
        let epochs: HashSet<u64> = all.iter().copied().collect();
        let max = *epochs.iter().max().unwrap();
        assert_eq!(st.epochs_sealed, max, "epoch numbering must be gapless: {st:?}");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn group_commit_amortises_the_fsync() {
        // The reason a single sealer is not the ceiling it looks like. Under concurrency
        // the sealer drains what is waiting and syncs once, so transactions-per-fsync rises
        // with load instead of falling.
        let p = tmp("group");
        let s = Arc::new(seq(&p, SyncPolicy::Always));
        let mut handles = Vec::new();
        for t in 0..16u64 {
            let s = Arc::clone(&s);
            handles.push(std::thread::spawn(move || {
                for i in 0..100u64 {
                    let _ = s.submit(Txn { idem_key: format!("g{t}-{i}"), payload: vec![0; 32] });
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let st = s.stats();
        assert_eq!(st.txns_committed, 1600);
        assert!(
            st.max_batch > 1,
            "under 16 concurrent submitters the sealer must have batched at least once: {st:?}"
        );
        assert!(
            st.txns_per_fsync() > 1.0,
            "group commit must beat one transaction per fsync: {:.2}",
            st.txns_per_fsync()
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_retried_transaction_returns_its_original_epoch_and_does_not_double() {
        // Idempotency, which is the property that makes a client retry safe. The wrong
        // behaviour here is not an error — it is a second, successful, duplicate payment.
        let p = tmp("idem");
        let s = seq(&p, SyncPolicy::Always);
        let first = s.submit(Txn { idem_key: "pay-991".into(), payload: b"100".to_vec() }).unwrap();
        let again = s.submit(Txn { idem_key: "pay-991".into(), payload: b"100".to_vec() });
        assert_eq!(again, Err(Rejected::Duplicate { at_epoch: first }));
        let st = s.stats();
        assert_eq!(st.txns_committed, 1, "the retry must not have created a second transaction");
        assert_eq!(st.duplicates_absorbed, 1, "stats were {st:?}");
        s.shutdown();
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn two_copies_of_one_key_in_the_same_batch_commit_once() {
        // The race the first version of this sealer lost. Both copies arrive before
        // either is recorded as committed, so a check against committed history alone
        // lets both through — and the result is not an error, it is a duplicated payment.
        let p = tmp("batch-idem");
        let s = Arc::new(seq(&p, SyncPolicy::Always));
        for _round in 0..40 {
            let key = format!("race-{_round}");
            let (a, b) = (Arc::clone(&s), Arc::clone(&s));
            let (k1, k2) = (key.clone(), key.clone());
            let h1 = std::thread::spawn(move || a.submit(Txn { idem_key: k1, payload: vec![1] }));
            let h2 = std::thread::spawn(move || b.submit(Txn { idem_key: k2, payload: vec![1] }));
            let (r1, r2) = (h1.join().unwrap(), h2.join().unwrap());
            let committed = [&r1, &r2].iter().filter(|r| r.is_ok()).count();
            assert_eq!(committed, 1, "exactly one copy may commit: {r1:?} / {r2:?}");
            // And the duplicate must be told the epoch the original got.
            let epoch = r1.or(r2.clone()).unwrap_or_else(|_| match r2 {
                Err(Rejected::Duplicate { at_epoch }) => at_epoch,
                other => panic!("{other:?}"),
            });
            assert!(epoch >= 1);
        }
        let st = s.stats();
        assert_eq!(st.txns_committed, 40, "40 distinct keys, 40 commits: {st:?}");
        assert_eq!(st.duplicates_absorbed, 40);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_snapshot_taken_during_writes_names_a_stable_prefix() {
        let p = tmp("snapshot");
        let s = Arc::new(seq(&p, SyncPolicy::Always));
        for i in 0..20 {
            s.submit(Txn { idem_key: format!("s{i}"), payload: vec![i] }).unwrap();
        }
        let snap = s.frontier().snapshot();
        let writer = {
            let s = Arc::clone(&s);
            std::thread::spawn(move || {
                for i in 20..60 {
                    let _ = s.submit(Txn { idem_key: format!("s{i}"), payload: vec![i as u8] });
                }
            })
        };
        writer.join().unwrap();
        assert!(s.frontier().visible() > snap.anchor, "the frontier must have moved on");
        assert!(snap.includes(snap.anchor) && !snap.includes(snap.anchor + 1),
            "the snapshot itself must not have moved");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn everything_submitted_before_shutdown_commits() {
        let p = tmp("drain");
        let s = seq(&p, SyncPolicy::Always);
        for i in 0..25 {
            s.submit(Txn { idem_key: format!("d{i}"), payload: vec![i] }).unwrap();
        }
        let st = s.shutdown();
        assert_eq!(st.txns_committed, 25);
        let rec = recover(&p).unwrap();
        assert!(rec.was_clean(), "{:?}", rec.cause);
        assert_eq!(rec.records.len() as u64, st.epochs_sealed);
        let _ = std::fs::remove_file(&p);
    }
}
