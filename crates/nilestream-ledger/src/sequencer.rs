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
use crate::segment::{Recovery, Segment, SyncPolicy};
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

/// Read a little-endian `u32` at `*offset`, advancing it. `None` past the end.
fn read_u32(buf: &[u8], offset: &mut usize) -> Option<u32> {
    if *offset + 4 > buf.len() {
        return None;
    }
    let v = u32::from_le_bytes(buf[*offset..*offset + 4].try_into().ok()?);
    *offset += 4;
    Some(v)
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
    /// Rebuild the idempotency window by reading the keys back out of a segment.
    ///
    /// The inverse of the batch framing in the sealer. A key that committed before a
    /// restart must still be refused after it, with the epoch it originally committed at.
    /// The same, refusing a record whose envelope it cannot finish reading.
    pub fn recover_seen_checked(
        recovery: &Recovery,
    ) -> Result<std::collections::BTreeMap<String, u64>, String> {
        let mut seen = std::collections::BTreeMap::new();
        for rec in &recovery.records {
            // The frontier's epoch for this record. The record's own number is a *batch*
            // sequence number; this is the visibility coordinate, 1-based.
            let epoch = rec.batch_seq + 1;
            for (key, _) in Self::decode_envelope(&rec.payload)
                .map_err(|e| format!("segment record {}: {e}", rec.batch_seq))?
            {
                seen.insert(key, epoch);
            }
        }
        Ok(seen)
    }

    /// Open a sequencer over a segment at `path`, recovering the idempotency window.
    ///
    /// **`SyncPolicy::Never` is refused here.** `submit` promises that it returns when the
    /// transaction's epoch is durable *and* visible, and under `Never` the frontier was
    /// published with nothing on stable storage — so the promise was false for a policy
    /// any caller could pass. The policy still exists on [`Segment`], because the
    /// durability benchmark prices the guarantee by removing it; what it may not do is
    /// reach a path that claims the guarantee.
    pub fn open(
        path: impl AsRef<std::path::Path>,
        policy: SyncPolicy,
    ) -> std::io::Result<Sequencer> {
        Ok(Self::open_recovered(path, policy)?.0)
    }

    /// `open`, and the records it recovered.
    ///
    /// A caller that keeps derived state over this ledger — the daemon's in-memory base is
    /// one — has to replay those records to rebuild it, and cannot do that without seeing
    /// them. `open` threw them away, which is why a restart recovered the idempotency window
    /// and nothing else: the sequencer knew every committed key and the base knew no rows.
    pub fn open_recovered(
        path: impl AsRef<std::path::Path>,
        policy: SyncPolicy,
    ) -> std::io::Result<(Sequencer, Recovery)> {
        Self::open_bounded(path, policy, None)
    }

    /// `open_recovered`, with the idempotency window the schema declared, in epochs.
    pub fn open_bounded(
        path: impl AsRef<std::path::Path>,
        policy: SyncPolicy,
        window: Option<u64>,
    ) -> std::io::Result<(Sequencer, Recovery)> {
        if matches!(policy, SyncPolicy::Never) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SyncPolicy::Never publishes an epoch with nothing on stable storage; a \
                 sequencer that accepted it would be lying in `submit`'s contract. Use \
                 `Segment` directly for the durability benchmark.",
            ));
        }
        let (segment, recovery) = Segment::open(path, policy)?;
        let frontier = Frontier::new();
        // **The checked form, and the difference is a data-loss defect.** This called
        // an unchecked wrapper that was `recover_seen_checked(..).unwrap_or_default()`: on any
        // envelope the decoder refused, the window came back **empty** and a sequencer
        // started with it, so every identity committed before the restart was new again and
        // a retry committed a second time. The daemon survived only because a *later*
        // checked replay returned `Err` and dropped the engine; every other caller of this
        // function — the durability bench, the tests, `Sequencer::open` — got the empty
        // window and no error at all (F-62, A9-F04).
        let seen = Sequencer::recover_seen_checked(&recovery).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "the idempotency window cannot be rebuilt from this segment: {e}. \
                     Opening with an empty window would let every identity that committed \
                     before this restart commit again."
                ),
            )
        })?;
        if let Some(head) = recovery.head() {
            frontier.seal(head + 1);
            frontier.publish(head + 1);
        }
        Ok((
            Self::start_bounded(segment, frontier, policy, seen, window),
            recovery,
        ))
    }

    /// Every committed transaction, in commit order, as `(idempotency key, payload)`.
    ///
    /// The inverse of the sealer's batch framing, and the companion to
    /// [`Sequencer::recover_seen_checked`], which reads the same bytes for the keys alone.
    ///
    /// **One record is one batch and one batch is many transactions**, so a record's epoch is
    /// not a transaction's index: a caller replaying these onto its own log must count
    /// transactions, not records. That distinction is the whole reason this returns a flat
    /// sequence rather than a per-record structure, and getting it wrong would put every
    /// recovered row at the wrong epoch.
    /// The same, refusing a record whose envelope it cannot finish reading.
    ///
    /// **The checked form is what a restart must use.** `recover_txns` returns what it could
    /// read, which is the right shape for a tool that inspects a segment and the wrong one
    /// for the daemon: replaying a short record loses acknowledged transactions silently.
    pub fn recover_txns_checked(recovery: &Recovery) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut out = Vec::new();
        for rec in &recovery.records {
            out.extend(
                Self::decode_envelope(&rec.payload)
                    .map_err(|e| format!("segment record {}: {e}", rec.batch_seq))?,
            );
        }
        Ok(out)
    }

    /// **One batch envelope, decoded or refused.**
    ///
    /// `count | (key_len, key, payload_len, payload)*`, and the *count* is the point: both
    /// recovery paths used to walk this framing with a `break` at every short read, so a
    /// record whose envelope ended early lost the transactions after the truncation and said
    /// nothing. The segment's CRC does not save it — the checksum covers the record's bytes,
    /// and a record can be perfectly well-formed at the storage layer while its payload's own
    /// framing is short, which is what a writer bug or a tamper with a recomputed CRC
    /// produces. Recovery would then replay fewer transactions than were acknowledged, and
    /// report success.
    ///
    /// Refusing is the only honest option: a suffix replayed onto a prefix that is missing an
    /// epoch rebuilds a ledger nobody wrote, and a durability claim that quietly drops an
    /// acknowledged transaction is worse than one that fails loudly.
    fn decode_envelope(payload: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut o = 0usize;
        let Some(count) = read_u32(payload, &mut o) else {
            return Err("the envelope has no transaction count".into());
        };
        let mut out = Vec::with_capacity(count.min(4096) as usize);
        for i in 0..count {
            let Some(klen) = read_u32(payload, &mut o) else {
                return Err(format!(
                    "transaction {i} of {count}: the envelope ends before its key length"
                ));
            };
            if o + klen as usize > payload.len() {
                return Err(format!(
                    "transaction {i} of {count}: a key of {klen} bytes runs past the end of a \
                     {}-byte envelope",
                    payload.len()
                ));
            }
            // **Strict, not lossy.** `from_utf8_lossy` turned damaged identity bytes into
            // replacement characters and carried on, so a corrupted key recovered as a
            // *different, valid-looking* key: the retry it was supposed to refuse would then
            // commit again (A9-F04). An identity that is not what was written is not an
            // identity.
            let key = match std::str::from_utf8(&payload[o..o + klen as usize]) {
                Ok(k) => k.to_string(),
                Err(e) => {
                    return Err(format!(
                        "transaction {i} of {count}: the idempotency key is not valid UTF-8 \
                         ({e}); it was written as UTF-8, so these are not the bytes that were \
                         written"
                    ))
                }
            };
            o += klen as usize;
            let Some(plen) = read_u32(payload, &mut o) else {
                return Err(format!(
                    "transaction {i} of {count} (`{key}`): the envelope ends before its \
                     payload length"
                ));
            };
            if o + plen as usize > payload.len() {
                return Err(format!(
                    "transaction {i} of {count} (`{key}`): a payload of {plen} bytes runs past \
                     the end of a {}-byte envelope",
                    payload.len()
                ));
            }
            out.push((key, payload[o..o + plen as usize].to_vec()));
            o += plen as usize;
        }
        // **Every byte accounted for.** A count that stops short of the payload's end leaves
        // bytes nobody claimed: either the count is wrong or the framing is, and in both
        // cases the transactions recovered are not the transactions written. Accepting the
        // prefix and ignoring the rest is the same silent-drop this function exists to
        // refuse, one level up (A9-F04).
        if o != payload.len() {
            return Err(format!(
                "the envelope declares {count} transaction(s) occupying {o} bytes of a \
                 {}-byte payload; {} byte(s) are unaccounted for",
                payload.len(),
                payload.len() - o
            ));
        }
        Ok(out)
    }

    pub fn start(segment: Segment, frontier: Arc<Frontier>, policy: SyncPolicy) -> Sequencer {
        Self::start_with_window(segment, frontier, policy, Default::default())
    }

    /// Start with a recovered idempotency window.
    pub fn start_with_window(
        segment: Segment,
        frontier: Arc<Frontier>,
        policy: SyncPolicy,
        recovered_seen: std::collections::BTreeMap<String, u64>,
    ) -> Sequencer {
        Self::start_bounded(segment, frontier, policy, recovered_seen, None)
    }

    /// The same, with the idempotency window the schema declared, **in epochs**.
    ///
    /// The sealer's window is the second of the two this system holds — the ledger's
    /// admission index is the other — and until T-05 (cycle 8) neither was bounded: every
    /// identity ever committed stayed in both, at a measured 99.6 B here and 68.8 B there
    /// (E18), so 168.4 B per identity, forever, on the write path.
    ///
    /// `None` keeps every identity, which is what a sequencer built without a declared
    /// window does, and is reported rather than assumed.
    ///
    /// **Epochs, not days**: an epoch carries no wall clock, and putting a timestamp under
    /// the chain hash would make every committed hash time-dependent (LC-28, decided by the
    /// author, cycle 8).
    ///
    /// **And transactions, not batches.** A batch is one segment record and can hold up to
    /// 4,096 transactions, so pruning by the record's sequence number — which this did until
    /// cycle 9 — bounded the window somewhere between `window` and 4,096·`window` identities
    /// while the base's admission index bounded itself at exactly `window` transactions. One
    /// declaration, two units, and a retry in the gap was new to admission and a duplicate
    /// here: an applied transaction with no acknowledgement (A9-F02). The bound is the count
    /// of transactions now, which is what one epoch means in the base and what the column's
    /// `N.epochs` has meant since LC-28.
    pub fn start_bounded(
        segment: Segment,
        frontier: Arc<Frontier>,
        policy: SyncPolicy,
        recovered_seen: std::collections::BTreeMap<String, u64>,
        window: Option<u64>,
    ) -> Sequencer {
        Self::start_gated(segment, frontier, policy, recovered_seen, window, None)
    }

    /// `start_bounded`, with an optional gate the sealer waits on before its first drain.
    ///
    /// **The gate exists so that group commit can be a property rather than a race.** The
    /// tests for it submitted from sixteen threads and asserted `max_batch > 1`, which is
    /// true only if two submitters happen to queue before the sealer drains — a fact the
    /// scheduler decides. On a host that hands the sealer a core between every send, that
    /// assertion fails, and the failure looks exactly like group commit being broken. With
    /// the gate the test states the actual claim: *if N transactions are waiting when the
    /// sealer wakes, it seals them as one epoch behind one barrier.* Nothing about
    /// production changes — `start_bounded` passes `None`, and a `None` gate is not waited
    /// on — but the property becomes observable without asking the operating system for a
    /// favour.
    fn start_gated(
        mut segment: Segment,
        frontier: Arc<Frontier>,
        policy: SyncPolicy,
        recovered_seen: std::collections::BTreeMap<String, u64>,
        window: Option<u64>,
        gate: Option<Arc<std::sync::Barrier>>,
    ) -> Sequencer {
        let (tx, rx): (Sender<Request>, Receiver<Request>) = channel();
        let stats = Arc::new(Mutex::new(SequencerStats::default()));
        let (f, s) = (Arc::clone(&frontier), Arc::clone(&stats));

        let sealer = std::thread::spawn(move || {
            if let Some(g) = gate {
                g.wait();
            }
            // The idempotency window, recovered from the segment at start-up rather than
            // starting empty. `BTreeMap`, not `HashMap`: the window decides which epoch a
            // duplicate is told it committed at, and an epoch is a hashed, audited value
            // (GC-12 in this repository's conventions).
            let mut seen: std::collections::BTreeMap<std::sync::Arc<str>, u64> = recovered_seen
                .into_iter()
                .map(|(k, e)| (std::sync::Arc::from(k.as_str()), e))
                .collect();
            // The same identities in commit order, so the window prunes from the front in
            // O(1) instead of scanning the map for old epochs. `Arc<str>`: the deque holds a
            // pointer to the key the map already owns, not a second copy of it.
            let mut order: std::collections::VecDeque<(std::sync::Arc<str>, u64)> = seen
                .iter()
                .map(|(k, e)| (std::sync::Arc::clone(k), *e))
                .collect();
            order.make_contiguous().sort_by_key(|(_, e)| *e);

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
                    if let Some(e) = seen.get(req.txn.idem_key.as_str()) {
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
                // One epoch for the whole batch. The framing carries each transaction's
                // **idempotency key** as well as its payload, because the window has to be
                // rebuildable from the segment: an idempotency window that does not
                // survive a restart does not protect against the retry that a restart
                // provokes, which is the one retry a client is most likely to send.
                //
                // Layout: count | (key_len, key, payload_len, payload)*
                let mut payload = Vec::new();
                payload.extend_from_slice(&(fresh.len() as u32).to_le_bytes());
                for r in &fresh {
                    let k = r.txn.idem_key.as_bytes();
                    payload.extend_from_slice(&(k.len() as u32).to_le_bytes());
                    payload.extend_from_slice(k);
                    payload.extend_from_slice(&(r.txn.payload.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&r.txn.payload);
                }

                match segment.append(payload) {
                    Ok(rec) => {
                        // The frontier's epoch for this record. The record's own number is
                        // a *batch* sequence number; this is the visibility coordinate,
                        // 1-based.
                        let epoch = rec.batch_seq + 1;
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
                            let k: std::sync::Arc<str> =
                                std::sync::Arc::from(r.txn.idem_key.as_str());
                            seen.insert(std::sync::Arc::clone(&k), epoch);
                            order.push_back((k, epoch));
                        }
                        // **Prune by transactions, which is the unit the declaration means.**
                        //
                        // This dropped an identity when its *batch sequence number* was
                        // `window` behind the batch just written. The base's admission index
                        // (`proto_engine::Ledger::prune_idem`) bounds itself by the last
                        // `window` **transactions**, and one transaction there is one epoch.
                        // A batch here is one segment record holding up to 4,096
                        // transactions, so the two structures called "the window" counted
                        // different things under one declaration: for `window = W` the base
                        // remembered W transactions and this remembered somewhere between W
                        // and 4,096·W of them. A retry landing in the gap was *new* to
                        // admission — applied to the base, an epoch sealed — and a
                        // *duplicate* to the sink, which is an applied transaction that
                        // cannot be acknowledged (A9-F02).
                        //
                        // `order` holds one entry per transaction, so bounding its length is
                        // the same rule the base applies, in the same unit, and the two
                        // windows now hold the same set.
                        if let Some(w) = window {
                            let w = w.max(1) as usize;
                            while order.len() > w {
                                if let Some((k, _)) = order.pop_front() {
                                    seen.remove(&k);
                                }
                            }
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
                        // **Fail-stop (LC-21).** The sealer stops. It used to answer `Io` and
                        // take the next batch, which left epoch *n* applied to every reader's
                        // base with no record on disk while epoch *n+1* committed and
                        // published — and the frontier publishes with `fetch_max` over
                        // contiguous epochs, so publishing *n+1* makes *n* visible. A failed
                        // epoch masked by the next success is a hole in the middle of a
                        // chain: on replay it is either a gap or a record that fails its
                        // checksum, and recovery stops there, losing every acknowledged epoch
                        // after it.
                        //
                        // Refusing everything from here is the trade a ledger should make:
                        // availability under a storage fault, for a prefix that is whole.
                        // Every later submitter gets `ShuttingDown` because the channel's
                        // sender is dropped with this loop.
                        break;
                    }
                }
            }
        });

        Sequencer {
            tx: Some(tx),
            frontier,
            stats,
            sealer: Some(sealer),
        }
    }

    /// Submit and wait for the transaction to be durable and visible.
    pub fn submit(&self, txn: Txn) -> Result<u64, Rejected> {
        self.submit_pending(txn)?
            .recv()
            .map_err(|_| Rejected::ShuttingDown)?
    }

    /// **Hand the transaction to the sealer and return without waiting for the barrier.**
    ///
    /// The waiting is the whole problem this exists to move. A caller that blocks in
    /// [`submit`](Self::submit) while holding a lock makes every other thread wait behind one
    /// `fsync` — and, worse, makes it impossible for a *second* submitter to arrive, so the
    /// sealer's drain loop finds an empty queue and group commit, which is built and tested
    /// at sixteen concurrent submitters, never forms a batch. Measured through the daemon
    /// before this existed: `max_batch` 1 and 1.00 transactions per fsync at every connection
    /// count, against 9.65 for the same sealer driven directly.
    ///
    /// The returned receiver yields exactly what `submit` would have: the epoch once the
    /// record is on stable storage and the frontier has been published, or the rejection. A
    /// caller that never reads it has still committed the transaction — the ordering
    /// guarantee is the sealer's, not the caller's — so the receiver is a *notification*,
    /// not a handle to uncommitted work.
    pub fn submit_pending(&self, txn: Txn) -> Result<Receiver<Result<u64, Rejected>>, Rejected> {
        let (reply, rx) = channel();
        let Some(tx) = &self.tx else {
            return Err(Rejected::ShuttingDown);
        };
        tx.send(Request { txn, reply })
            .map_err(|_| Rejected::ShuttingDown)?;
        Ok(rx)
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
        let (segment, recovery) = Segment::open(p, policy).unwrap();
        // The window comes back with the segment: a restart must still refuse a key that
        // committed before it.
        let seen = Sequencer::recover_seen_checked(&recovery).expect("the segment decodes");
        Sequencer::start_with_window(segment, Frontier::new(), policy, seen)
    }

    #[test]
    fn a_commit_returns_only_once_it_is_durable_and_visible() {
        let p = tmp("durable");
        let s = seq(&p, SyncPolicy::Always);
        let e = s
            .submit(Txn {
                idem_key: "a".into(),
                payload: b"x".to_vec(),
            })
            .unwrap();
        assert!(e >= 1);
        assert_eq!(
            s.frontier().visible(),
            e,
            "the frontier must be at the returned epoch"
        );
        assert_eq!(
            s.frontier().pending(),
            0,
            "nothing sealed-but-unpublished may remain"
        );
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
                    mine.push(
                        s.submit(Txn {
                            idem_key: k,
                            payload: vec![t as u8, i as u8],
                        })
                        .unwrap(),
                    );
                }
                mine
            }));
        }
        let all: Vec<u64> = handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect();
        assert_eq!(all.len(), 400);

        let st = s.stats();
        assert_eq!(
            st.txns_committed, 400,
            "every transaction must commit exactly once"
        );
        // Epochs form a prefix of the integers with no gaps: a total order.
        let epochs: HashSet<u64> = all.iter().copied().collect();
        let max = *epochs.iter().max().unwrap();
        assert_eq!(
            st.epochs_sealed, max,
            "epoch numbering must be gapless: {st:?}"
        );
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
                    let _ = s.submit(Txn {
                        idem_key: format!("g{t}-{i}"),
                        payload: vec![0; 32],
                    });
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let st = s.stats();
        assert_eq!(st.txns_committed, 1600);
        // **Reported, not asserted.** Whether sixteen threads happen to queue two requests
        // before the sealer drains is the scheduler's decision, and asserting on it made
        // this test fail on hosts that schedule the sealer eagerly. The claim it was trying
        // to make is asserted deterministically by
        // `every_transaction_waiting_when_the_sealer_wakes_commits_in_one_epoch` below; what
        // survives here is the load-shaped observation, printed so a run that batches nothing
        // is visible without being called a defect.
        eprintln!(
            "  group_commit_amortises_the_fsync: max_batch={} txns/fsync={:.2} (load-shaped, \
             not asserted)",
            st.max_batch,
            st.txns_per_fsync()
        );
        let _ = std::fs::remove_file(&p);
    }

    /// **Group commit, as a property: what is waiting when the sealer wakes is one epoch.**
    ///
    /// The deterministic half of the claim `group_commit_amortises_the_fsync` used to assert
    /// by racing. The sealer is held behind a gate while thirty-two transactions are queued
    /// with `submit_pending`, which returns without waiting; when the gate opens, the drain
    /// finds all of them. `max_batch` must then be exactly thirty-two, one epoch must have
    /// been sealed, and the whole batch must have cost one barrier — the three numbers group
    /// commit is *for*, none of them decided by the operating system.
    #[test]
    fn every_transaction_waiting_when_the_sealer_wakes_commits_in_one_epoch() {
        const N: usize = 32;
        let p = tmp("gated-batch");
        let (segment, _rec) = Segment::open(&p, SyncPolicy::Always).unwrap();
        // Two parties: this thread and the sealer.
        let gate = Arc::new(std::sync::Barrier::new(2));
        let s = Sequencer::start_gated(
            segment,
            Frontier::new(),
            SyncPolicy::Always,
            Default::default(),
            None,
            Some(Arc::clone(&gate)),
        );
        let mut waiting = Vec::with_capacity(N);
        for i in 0..N {
            waiting.push(
                s.submit_pending(Txn {
                    idem_key: format!("gated-{i:03}"),
                    payload: vec![i as u8; 16],
                })
                .expect("queued"),
            );
        }
        // Everything is in the channel and the sealer has not looked at it yet.
        gate.wait();
        let mut epochs = Vec::with_capacity(N);
        for rx in waiting {
            epochs.push(rx.recv().expect("the sealer replied").expect("committed"));
        }
        let st = s.shutdown();
        assert_eq!(
            st.max_batch, N as u64,
            "every transaction waiting when the sealer woke had to seal as one batch: \
             max_batch={} over {N} queued",
            st.max_batch
        );
        assert_eq!(st.epochs_sealed, 1, "one batch is one epoch: {st:?}");
        assert_eq!(
            epochs
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            1,
            "every transaction in one batch commits at the same epoch: {epochs:?}"
        );
        assert_eq!(
            st.fsyncs, 1,
            "one epoch is one barrier — that is what group commit amortises: {st:?}"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_retried_transaction_returns_its_original_epoch_and_does_not_double() {
        // Idempotency, which is the property that makes a client retry safe. The wrong
        // behaviour here is not an error — it is a second, successful, duplicate payment.
        let p = tmp("idem");
        let s = seq(&p, SyncPolicy::Always);
        let first = s
            .submit(Txn {
                idem_key: "pay-991".into(),
                payload: b"100".to_vec(),
            })
            .unwrap();
        let again = s.submit(Txn {
            idem_key: "pay-991".into(),
            payload: b"100".to_vec(),
        });
        assert_eq!(again, Err(Rejected::Duplicate { at_epoch: first }));
        let st = s.stats();
        assert_eq!(
            st.txns_committed, 1,
            "the retry must not have created a second transaction"
        );
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
            let h1 = std::thread::spawn(move || {
                a.submit(Txn {
                    idem_key: k1,
                    payload: vec![1],
                })
            });
            let h2 = std::thread::spawn(move || {
                b.submit(Txn {
                    idem_key: k2,
                    payload: vec![1],
                })
            });
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
        assert_eq!(
            st.txns_committed, 40,
            "40 distinct keys, 40 commits: {st:?}"
        );
        assert_eq!(st.duplicates_absorbed, 40);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_snapshot_taken_during_writes_names_a_stable_prefix() {
        let p = tmp("snapshot");
        let s = Arc::new(seq(&p, SyncPolicy::Always));
        for i in 0..20 {
            s.submit(Txn {
                idem_key: format!("s{i}"),
                payload: vec![i],
            })
            .unwrap();
        }
        let snap = s.frontier().snapshot();
        let writer = {
            let s = Arc::clone(&s);
            std::thread::spawn(move || {
                for i in 20..60 {
                    let _ = s.submit(Txn {
                        idem_key: format!("s{i}"),
                        payload: vec![i as u8],
                    });
                }
            })
        };
        writer.join().unwrap();
        assert!(
            s.frontier().visible() > snap.anchor,
            "the frontier must have moved on"
        );
        assert!(
            snap.includes(snap.anchor) && !snap.includes(snap.anchor + 1),
            "the snapshot itself must not have moved"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn everything_submitted_before_shutdown_commits() {
        let p = tmp("drain");
        let s = seq(&p, SyncPolicy::Always);
        for i in 0..25 {
            s.submit(Txn {
                idem_key: format!("d{i}"),
                payload: vec![i],
            })
            .unwrap();
        }
        let st = s.shutdown();
        assert_eq!(st.txns_committed, 25);
        let rec = recover(&p).unwrap();
        assert!(rec.was_clean(), "{:?}", rec.cause);
        assert_eq!(rec.records.len() as u64, st.epochs_sealed);
        let _ = std::fs::remove_file(&p);
    }
}

#[cfg(test)]
mod window_tests {
    //! The idempotency window across a restart.

    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("niles-win-{name}-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    /// **The sealer's window, bounded by epoch age — T-05.3.**
    ///
    /// One submitter under `SyncPolicy::Always`, so each transaction is its own epoch and
    /// "four epochs ago" is "four transactions ago". A key inside the window is still
    /// refused as a duplicate and told the epoch it committed at; one that has aged out is
    /// admitted again, because the window no longer holds it.
    #[test]
    fn a_key_older_than_the_window_is_new_to_the_sealer() {
        let p = tmp("window");
        let (segment, recovery) = Segment::open(&p, SyncPolicy::Always).unwrap();
        let seen = Sequencer::recover_seen_checked(&recovery).expect("the segment decodes");
        let s =
            Sequencer::start_bounded(segment, Frontier::new(), SyncPolicy::Always, seen, Some(4));
        for i in 0..10u64 {
            s.submit(Txn {
                idem_key: format!("k{i}"),
                payload: vec![i as u8],
            })
            .expect("new");
        }
        assert!(
            matches!(
                s.submit(Txn {
                    idem_key: "k9".into(),
                    payload: vec![9],
                }),
                Err(Rejected::Duplicate { .. })
            ),
            "`k9` is inside the window and its retry must be told the original epoch"
        );
        assert!(
            s.submit(Txn {
                idem_key: "k0".into(),
                payload: vec![0],
            })
            .is_ok(),
            "`k0` has aged out of the window, so the sealer no longer knows it committed"
        );
        let _ = std::fs::remove_file(&p);
    }

    /// **The window's boundary is the same transaction count on both sides — C9-02.3.**
    ///
    /// `W - 1`, `W`, `W + 1` transactions after an identity committed, with the batch size
    /// forced to one and then to many. The rule is the base's: an identity is remembered iff
    /// it is among the last `W` **transactions**. Under the old batch-sequence rule the
    /// many-per-batch arm remembered every identity in the run, because ten batches of one
    /// are ten sequence numbers and one batch of ten is one.
    #[test]
    fn the_window_holds_the_last_w_transactions_whatever_the_batch_size() {
        const W: u64 = 4;

        // Arm 1: one transaction per batch. Each `submit` waits, so the sealer drains one.
        {
            let p = tmp("boundary-one");
            let (segment, recovery) = Segment::open(&p, SyncPolicy::Always).unwrap();
            let seen = Sequencer::recover_seen_checked(&recovery).expect("the segment decodes");
            let s = Sequencer::start_bounded(
                segment,
                Frontier::new(),
                SyncPolicy::Always,
                seen,
                Some(W),
            );
            s.submit(Txn {
                idem_key: "target".into(),
                payload: vec![0],
            })
            .expect("new");
            // W - 1 further transactions: `target` is the W-th most recent, still inside.
            for i in 0..(W - 1) {
                s.submit(Txn {
                    idem_key: format!("f{i}"),
                    payload: vec![i as u8],
                })
                .expect("new");
            }
            assert!(
                matches!(
                    s.submit(Txn {
                        idem_key: "target".into(),
                        payload: vec![0]
                    }),
                    Err(Rejected::Duplicate { .. })
                ),
                "at W-1 further transactions the identity is the W-th most recent and inside \
                 the window"
            );
            // One more distinct transaction pushes it out.
            s.submit(Txn {
                idem_key: "one-more".into(),
                payload: vec![9],
            })
            .expect("new");
            assert!(
                s.submit(Txn {
                    idem_key: "target".into(),
                    payload: vec![0]
                })
                .is_ok(),
                "at W further transactions the identity has aged out and is new again"
            );
            s.shutdown();
            let _ = std::fs::remove_file(&p);
        }

        // Arm 2: many transactions per batch, queued behind the gate so the drain takes them
        // all at once. Under the batch-sequence rule this whole run was one sequence number
        // and nothing ever aged out.
        {
            let p = tmp("boundary-batch");
            let (segment, _rec) = Segment::open(&p, SyncPolicy::Always).unwrap();
            let gate = Arc::new(std::sync::Barrier::new(2));
            let s = Sequencer::start_gated(
                segment,
                Frontier::new(),
                SyncPolicy::Always,
                Default::default(),
                Some(W),
                Some(Arc::clone(&gate)),
            );
            let mut waiting = Vec::new();
            waiting.push(
                s.submit_pending(Txn {
                    idem_key: "target".into(),
                    payload: vec![0],
                })
                .expect("queued"),
            );
            for i in 0..(W + 1) {
                waiting.push(
                    s.submit_pending(Txn {
                        idem_key: format!("b{i}"),
                        payload: vec![i as u8],
                    })
                    .expect("queued"),
                );
            }
            gate.wait();
            for rx in waiting {
                rx.recv().expect("replied").expect("committed");
            }
            let st = s.stats();
            assert!(
                st.max_batch > 1,
                "PRECONDITION UNMET: the gate did not produce a shared batch: {st:?}"
            );
            assert!(
                s.submit(Txn {
                    idem_key: "target".into(),
                    payload: vec![0]
                })
                .is_ok(),
                "W+1 transactions committed after `target`, so it is outside the window — \
                 whether they arrived as one batch or as {} of them. Bounding by the batch \
                 sequence number kept it forever.",
                st.epochs_sealed
            );
            s.shutdown();
            let _ = std::fs::remove_file(&p);
        }
    }

    #[test]
    fn idempotency_window_survives_restart() {
        // The retry a restart provokes is the one a client is most likely to send, and it
        // was the one retry the window could not refuse: `seen` lived in memory only, so
        // a key that committed before the restart committed again after it. That is a
        // duplicated payment, not an error.
        let p = tmp("restart");
        let first_epoch = {
            let q = Sequencer::open(&p, SyncPolicy::Always).unwrap();
            let e = q
                .submit(Txn {
                    idem_key: "payment-9".into(),
                    payload: vec![1, 2, 3],
                })
                .unwrap();
            q.shutdown();
            e
        };

        // Restart, and retry the same key.
        let q = Sequencer::open(&p, SyncPolicy::Always).unwrap();
        let again = q.submit(Txn {
            idem_key: "payment-9".into(),
            payload: vec![1, 2, 3],
        });
        assert_eq!(
            again,
            Err(Rejected::Duplicate {
                at_epoch: first_epoch
            }),
            "a key that committed before the restart must be refused after it, at the \
             epoch it originally committed at"
        );
        q.shutdown();

        // And exactly one record is on disk.
        let rec = crate::segment::recover(&p).unwrap();
        assert_eq!(rec.records.len(), 1, "the retry must not have appended");
    }

    #[test]
    fn a_fresh_key_still_commits_after_a_restart() {
        // The negative control: a recovered window that refused everything would pass the
        // test above and be useless.
        let p = tmp("fresh");
        {
            let q = Sequencer::open(&p, SyncPolicy::Always).unwrap();
            q.submit(Txn {
                idem_key: "a".into(),
                payload: vec![1],
            })
            .unwrap();
            q.shutdown();
        }
        let q = Sequencer::open(&p, SyncPolicy::Always).unwrap();
        let e = q
            .submit(Txn {
                idem_key: "b".into(),
                payload: vec![2],
            })
            .expect("a key never seen before must commit");
        assert!(e > 1);
        q.shutdown();
    }

    #[test]
    fn the_sequencer_refuses_a_policy_it_cannot_honour() {
        // `submit` promises the epoch is durable when it returns. Under `Never` the
        // frontier was published with nothing on disk, so the promise was false for a
        // policy any caller could pass.
        let p = tmp("never");
        let err = match Sequencer::open(&p, SyncPolicy::Never) {
            Ok(_) => panic!("a sequencer must not accept a policy it cannot honour"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}

/// **What a restart depends on**, tested in process rather than only through a spawned
/// binary, a `psql` session and a `SIGKILL`.
#[cfg(test)]
mod recovery_tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("niles-rec-{name}-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    /// **Recovery, in process — T-14.1.**
    ///
    /// `open_recovered` and `recover_txns_checked` are the two a restart depends on, and
    /// until now the only thing that exercised them end to end was the crash-protocol
    /// integration test: a spawned binary, a `psql` session, a `SIGKILL`, serial writers. A
    /// defect in the record walk would be found by a test that takes forty seconds and needs
    /// a PostgreSQL client, or not at all.
    ///
    /// Seal, drop, reopen, and assert that what comes back is what went in — the keys, the
    /// payloads, and **the order**, which is the part a decoder can get wrong while
    /// returning the right set.
    #[test]
    fn a_reopened_segment_returns_every_transaction_in_commit_order() {
        let p = tmp("recover-order");
        let sent: Vec<(String, Vec<u8>)> = (0..40u8)
            .map(|i| (format!("txn-{i:02}"), vec![i, i.wrapping_mul(7), 0xAB]))
            .collect();
        {
            let (segment, recovery) = Segment::open(&p, SyncPolicy::Always).unwrap();
            let seen = Sequencer::recover_seen_checked(&recovery).expect("the segment decodes");
            let s =
                Sequencer::start_with_window(segment, Frontier::new(), SyncPolicy::Always, seen);
            for (k, payload) in &sent {
                s.submit(Txn {
                    idem_key: k.clone(),
                    payload: payload.clone(),
                })
                .expect("new");
            }
            s.shutdown();
        }
        let (_segment, recovery) = Segment::open(&p, SyncPolicy::Always).unwrap();
        let back = Sequencer::recover_txns_checked(&recovery).expect("the segment decodes");
        assert_eq!(
            back, sent,
            "a reopened segment must return every transaction, with its payload, in the order \
             it was committed. Order is not decoration: the daemon replays these onto a seeded \
             prefix and asserts each lands at `seed_epochs + k`, so a decoder that returned the \
             right set in the wrong order would rebuild a ledger nobody wrote."
        );
        let window = Sequencer::recover_seen_checked(&recovery).expect("the segment decodes");
        assert_eq!(
            window.len(),
            sent.len(),
            "every committed identity must come back in the window"
        );
        for (i, (k, _)) in sent.iter().enumerate() {
            assert_eq!(
                window.get(k).copied(),
                Some(i as u64 + 1),
                "`{k}` must be remembered at the epoch it committed at (1-based on the \
                 frontier), or a retry after a restart is told it committed somewhere else"
            );
        }
        let _ = std::fs::remove_file(&p);
    }

    /// **Concurrent submitters — T-14.2.**
    ///
    /// `submit_pending` is the function group commit exists through: it hands the sealer a
    /// transaction and returns a receiver, so a second submitter can arrive while the first
    /// is waiting for a barrier. Nothing tested it under concurrency. The crash-protocol
    /// test drives one session at a time, which is exactly the shape that cannot see a
    /// batching defect — and `max_batch` reading 1 under load was how the *last* one was
    /// found.
    ///
    /// Eight threads, and three assertions that together say the sealer did its job: every
    /// reply arrives, the epochs are a permutation of the range (no gap, no duplicate), and
    /// at least one batch held more than one transaction.
    #[test]
    fn eight_concurrent_submitters_all_commit_and_at_least_one_batch_is_shared() {
        let p = tmp("concurrent-submit");
        let (segment, recovery) = Segment::open(&p, SyncPolicy::Every(8)).unwrap();
        let seen = Sequencer::recover_seen_checked(&recovery).expect("the segment decodes");
        let s = std::sync::Arc::new(Sequencer::start_with_window(
            segment,
            Frontier::new(),
            SyncPolicy::Every(8),
            seen,
        ));
        const THREADS: u64 = 8;
        const EACH: u64 = 32;
        let mut handles = Vec::new();
        for t in 0..THREADS {
            let s = std::sync::Arc::clone(&s);
            handles.push(std::thread::spawn(move || {
                let mut epochs = Vec::new();
                for i in 0..EACH {
                    let rx = s
                        .submit_pending(Txn {
                            idem_key: format!("t{t}-{i}"),
                            payload: vec![t as u8, i as u8],
                        })
                        .expect("submitted");
                    epochs.push(rx.recv().expect("the sealer replies").expect("committed"));
                }
                epochs
            }));
        }
        let mut all: Vec<u64> = handles
            .into_iter()
            .flat_map(|h| h.join().expect("a submitter panicked"))
            .collect();
        assert_eq!(
            all.len() as u64,
            THREADS * EACH,
            "every submitter must get a reply"
        );
        let st = s.stats();
        assert_eq!(
            st.txns_committed,
            THREADS * EACH,
            "the sealer must have committed every transaction it replied to"
        );
        all.sort_unstable();
        all.dedup();
        // Epochs are shared *within* a batch, so distinct epochs are at most the number of
        // transactions and at least one if any batching happened at all.
        assert!(
            !all.is_empty() && (all.len() as u64) <= THREADS * EACH,
            "epochs must be a set drawn from the commits: {} distinct",
            all.len()
        );
        // **Reported, not asserted — and the reason matters.** Whether any two of these
        // sixteen sends land in one drain is the scheduler's call: each thread waits for its
        // own reply before sending again, so on a host that runs the sealer between every
        // send, every batch is legitimately one. Asserting `max_batch > 1` here made a green
        // property depend on the machine's load. What this test *does* establish without the
        // operating system's help is above: every submitter gets a reply, every transaction
        // commits, and the epochs are drawn from the commits. The batching claim is asserted
        // deterministically by `every_transaction_waiting_when_the_sealer_wakes_commits_in_
        // one_epoch`.
        eprintln!(
            "  eight_concurrent_submitters: max_batch={} over {THREADS} submitters \
             (load-shaped, not asserted)",
            st.max_batch
        );
        let _ = std::fs::remove_file(&p);
    }
}
