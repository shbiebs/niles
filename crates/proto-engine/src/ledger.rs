//! The authoritative base: append-only, epoch-ordered, hash-chained, never partial.
//!
//! Two structures matter for the experiments:
//!   * `epochs` — the immutable sequence of sealed epochs (the base itself);
//!   * `by_account` — a per-key **anchor index** mapping an account to the positions of its
//!     postings. This is the structure that makes a reconstruction cost proportional to a
//!     key's own update count rather than to the length of history, and it is what the
//!     history-independence experiment (E5) tests.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use nilestream_ledger::chain::Hasher256;

use crate::{Acct, Cur, Epoch, Minor};

#[derive(Debug, Clone)]
pub struct Posting {
    pub txn: u64,
    pub acct: Acct,
    pub cur: Cur,
    pub amt: Minor,
    /// Valid time (the world's calendar), distinct from the epoch (system time).
    pub valid: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Post(Minor),
    Void,
    Expire,
}

#[derive(Debug, Clone)]
pub struct Hold {
    pub id: u64,
    pub acct: Acct,
    pub cur: Cur,
    pub amount: Minor,
}

/// A base row. There is deliberately no update or delete variant.
#[derive(Debug, Clone)]
pub enum Row {
    Post(Posting),
    Hold(Hold),
    Resolve { hold: u64, outcome: Outcome },
}

#[derive(Debug, Clone)]
pub struct EpochRec {
    pub id: Epoch,
    pub parent: [u8; 32],
    pub hash: [u8; 32],
    pub rows: Vec<Row>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reject {
    Duplicate,
    Unbalanced,
    BadResolution,
    /// A per-(txn, currency) sum that does not fit in `Minor`.
    ///
    /// Refused rather than wrapped. In release builds Rust's integer arithmetic wraps
    /// silently, so `[i128::MAX, i128::MAX, 2]` sums to zero and a transaction that
    /// creates money passes the commit rule. The conservation guarantee is stated over
    /// the integers, and this is where the implementation stops pretending its integers
    /// are unbounded.
    Overflow,
}

/// Where a posting lives: (epoch index, row index within the epoch).
#[derive(Debug, Clone, Copy)]
pub struct RowRef {
    pub epoch: Epoch,
    pub idx: u32,
}

#[derive(Default)]
pub struct Ledger {
    pub epochs: Vec<EpochRec>,
    /// The idempotency window: which identities have committed, and at which epoch.
    ///
    /// A `HashSet<String>` before T-05 (cycle 8), and unbounded: every identity the ledger
    /// had ever admitted stayed in it forever, at a measured 68.8 B each (E18,
    /// `idem_admission_index`), so the write path's memory was linear in history. A
    /// declared window — `idem: IdemKey window N.epochs` — is what the language always
    /// said this was, and nothing below the compiler had it.
    ///
    /// `Arc<str>`, so [`idem_order`](Self::idem_order) can hold the same identity in commit
    /// order without a second copy of the string: sixteen bytes for the pointer against
    /// sixty-eight for the key. A `HashMap` and not a `BTreeMap` because this index is
    /// looked up by key and never iterated in an order anything depends on — the one place
    /// order matters, [`with_idem_window`](Self::with_idem_window), sorts by epoch — and a
    /// `BTreeMap` here measured 26 B per identity more (E18 `ledger_seeded`), on the
    /// structure whose size this task exists to bound.
    idem: HashMap<Arc<str>, Epoch>,
    /// The same identities in commit order, so pruning is O(1) at the front rather than a
    /// scan for old epochs. One transaction is one epoch, so the deque's length *is* the
    /// number of epochs the window holds.
    idem_order: VecDeque<Arc<str>>,
    /// How many epochs an identity stays in the window. `None` is the old behaviour —
    /// forever — and is what a ledger built without a declared window still does; it is
    /// reported as such rather than assumed, over the wire as `idem_window_keys`.
    idem_window: Option<u64>,
    /// Anchor index: account -> positions of that account's postings, in epoch order.
    by_account: HashMap<Acct, Vec<RowRef>>,
    /// Anchor index for holds, keyed by account.
    holds_by_account: HashMap<Acct, Vec<RowRef>>,
    resolved_holds: HashSet<u64>,
    hold_index: HashMap<u64, RowRef>,
    /// Whether to compute the hash chain (used to price chaining in E7).
    pub chaining: bool,
    /// Per-key checkpoints: every `checkpoint_interval` postings on a key, record
    /// (epoch, running balance) so a later reconstruction can fold from the checkpoint
    /// rather than from genesis.
    ///
    /// This mechanism is not decoration: experiment E9 shows that without it the
    /// reconstruction cost of a *hot* key grows linearly with total history even when the
    /// key space grows in proportion, because a Zipf-hot key's share of traffic is roughly
    /// constant while traffic itself grows. Checkpoints are what make the reconstruction
    /// path's cost a function of a chosen interval instead of a function of the log's age.
    /// A checkpoint is derived state (it is recomputable from the base), so it does not
    /// weaken the immutability or retention argument.
    pub checkpoint_interval: usize,
    checkpoints: HashMap<(Acct, Cur), Vec<(Epoch, Minor)>>,
    posting_seen: HashMap<(Acct, Cur), usize>,
    running: HashMap<(Acct, Cur), Minor>,
    /// Instrumentation: base rows touched by reconstruction since last reset.
    ///
    /// **An atomic, and that is the point.** This counter was the only thing reconstruction
    /// mutated, and a `&mut self` taken for a counter is a `&mut self` taken for the whole
    /// ledger: it made `Base::reconstruct` exclusive, which made every read of the base
    /// exclusive, which made a fold of twenty thousand postings serialise against every
    /// other fold. Counting through an atomic costs one relaxed add per reconstruction and
    /// buys `&self` on the read path. Read it with [`Ledger::rows_touched`].
    rows_touched: std::sync::atomic::AtomicU64,
}

/// A destination for the canonical row encoding: a buffer when the bytes are wanted, a
/// hasher when only their digest is.
pub trait ByteSink {
    fn put(&mut self, bytes: &[u8]);
}

impl ByteSink for Vec<u8> {
    fn put(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

impl ByteSink for Hasher256 {
    fn put(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

/// Write the canonical encoding of `rows` into `sink`, one field at a time.
///
/// **The buffer is the thing worth not building.** [`encode_rows`] exists because the durable
/// record genuinely carries these bytes; `chain` only ever hands them to a hash, and building
/// a `Vec` to do that cost two allocations and ~132 B per epoch — enough to put `ledger_seeded`
/// and `append_in_memory` over their E18 budgets the moment T-01 made the chain row-shaped.
/// Streaming to a sink costs neither, and because both callers go through this one function
/// the digest and the payload cannot drift apart. `encode_rows_matches_the_streamed_form`
/// holds them to that.
pub fn write_rows(rows: &[Row], sink: &mut impl ByteSink) {
    sink.put(&(rows.len() as u32).to_le_bytes());
    for r in rows {
        match r {
            Row::Post(p) => {
                sink.put(b"P");
                sink.put(&p.txn.to_le_bytes());
                sink.put(&p.acct.to_le_bytes());
                sink.put(&p.cur.to_le_bytes());
                sink.put(&p.amt.to_le_bytes());
                sink.put(&p.valid.to_le_bytes());
            }
            Row::Hold(h) => {
                sink.put(b"H");
                sink.put(&h.id.to_le_bytes());
                sink.put(&h.acct.to_le_bytes());
                sink.put(&h.cur.to_le_bytes());
                sink.put(&h.amount.to_le_bytes());
            }
            Row::Resolve { hold, outcome } => {
                sink.put(b"R");
                sink.put(&hold.to_le_bytes());
                match outcome {
                    Outcome::Post(a) => {
                        sink.put(b"p");
                        sink.put(&a.to_le_bytes());
                    }
                    Outcome::Void => sink.put(b"v"),
                    Outcome::Expire => sink.put(b"e"),
                }
            }
        }
    }
}

/// **The canonical byte string for an epoch's rows.**
///
/// One row set, one encoding, on every machine and every run: fixed little-endian widths, a
/// one-byte tag per variant, a `u32` count first. It is the *same* field order and the same
/// tags the hash chain has always used — `chain` now hashes exactly these bytes, so a chain
/// over an encoding and a chain over the rows cannot disagree, which they could while the two
/// were written out separately.
///
/// It exists because durability needed it. Until T-01 the durable record carried
/// `epoch.to_string()` — the epoch *number* — so a restart recovered the idempotency window
/// and nothing else, and every acknowledged row was gone. A record that cannot reconstruct
/// the base is a receipt, not a log.
pub fn encode_rows(rows: &[Row]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + rows.len() * 40);
    write_rows(rows, &mut out);
    out
}

/// The inverse of [`encode_rows`]. `None` for anything that is not exactly its output.
///
/// Strict on purpose: a trailing byte, a short field or an unknown tag is a corrupt record,
/// and a decoder that guessed would replay a ledger nobody wrote.
pub fn decode_rows(b: &[u8]) -> Option<Vec<Row>> {
    let mut o = 0usize;
    let take = |o: &mut usize, n: usize| -> Option<&[u8]> {
        let s = b.get(*o..*o + n)?;
        *o += n;
        Some(s)
    };
    let count = u32::from_le_bytes(take(&mut o, 4)?.try_into().ok()?) as usize;
    let mut rows = Vec::with_capacity(count.min(1 << 16));
    for _ in 0..count {
        match take(&mut o, 1)?[0] {
            b'P' => rows.push(Row::Post(Posting {
                txn: u64::from_le_bytes(take(&mut o, 8)?.try_into().ok()?),
                acct: Acct::from_le_bytes(take(&mut o, 8)?.try_into().ok()?),
                cur: Cur::from_le_bytes(take(&mut o, 4)?.try_into().ok()?),
                amt: Minor::from_le_bytes(take(&mut o, 16)?.try_into().ok()?),
                valid: i64::from_le_bytes(take(&mut o, 8)?.try_into().ok()?),
            })),
            b'H' => rows.push(Row::Hold(Hold {
                id: u64::from_le_bytes(take(&mut o, 8)?.try_into().ok()?),
                acct: Acct::from_le_bytes(take(&mut o, 8)?.try_into().ok()?),
                cur: Cur::from_le_bytes(take(&mut o, 4)?.try_into().ok()?),
                amount: Minor::from_le_bytes(take(&mut o, 16)?.try_into().ok()?),
            })),
            b'R' => {
                let hold = u64::from_le_bytes(take(&mut o, 8)?.try_into().ok()?);
                let outcome = match take(&mut o, 1)?[0] {
                    b'p' => Outcome::Post(Minor::from_le_bytes(take(&mut o, 16)?.try_into().ok()?)),
                    b'v' => Outcome::Void,
                    b'e' => Outcome::Expire,
                    _ => return None,
                };
                rows.push(Row::Resolve { hold, outcome });
            }
            _ => return None,
        }
    }
    (o == b.len()).then_some(rows)
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            chaining: true,
            ..Default::default()
        }
    }

    pub fn with_checkpoints(interval: usize) -> Self {
        Self {
            chaining: true,
            checkpoint_interval: interval,
            ..Default::default()
        }
    }

    pub fn without_chaining() -> Self {
        Self {
            chaining: false,
            ..Default::default()
        }
    }

    /// The link for the epoch that will sit at index `epoch`.
    ///
    /// **The epoch number is an argument and not `self.epochs.len()`.** It was the length when
    /// T-01 put the epoch into the digest, which is correct at seal time — the epoch about to
    /// be pushed sits at the current length — and wrong everywhere else. `verify_chain` walks
    /// a *finished* ledger, where the length is the total count rather than the position of
    /// the epoch under test, so every link recomputed to a different digest and E1's chain
    /// column went to FAIL for all five seeds. A hash that only its writer can reproduce
    /// verifies nothing, and the failure was loud only because E1 checks the chain
    /// independently of the code that writes it.
    fn chain(&self, epoch: u64, parent: &[u8; 32], rows: &[Row]) -> [u8; 32] {
        if !self.chaining {
            return [0; 32];
        }
        // `H(parent ‖ epoch ‖ canon(rows))` — the segment's own formula
        // (`nilestream_ledger::segment::Record::seal`), over the same canonical bytes the
        // durable payload carries. The two chains were separate hashes of separate things:
        // this one over rows, the segment's over whatever it was handed, which until T-01 was
        // the epoch number. Sharing the formula and the bytes is what lets a replay *verify*
        // an epoch rather than merely rebuild it.
        let mut h = Hasher256::new();
        h.update(parent);
        h.update(&epoch.to_le_bytes());
        write_rows(rows, &mut h);
        h.finalize()
    }

    /// Set the idempotency window, in epochs, and prune to it.
    ///
    /// **Epochs, not days.** An epoch carries no wall clock: the durable record is
    /// `parent ‖ hash ‖ canon(rows)` and adding a timestamp under the chain would make every
    /// committed hash time-dependent. The window is therefore counted in the coordinate the
    /// ledger actually has (LC-28, decided by the author, cycle 8), and a declaration in
    /// days is refused by the compiler rather than silently rounded here.
    ///
    /// What it gives up, stated where it is chosen: **a retry of a transaction older than
    /// the window commits a second time.** That is what a window *is* — an idempotency key
    /// without one is a uniqueness constraint on all of history, which is the unbounded
    /// structure this replaces — and it is the reason the language requires the window to be
    /// declared rather than defaulted.
    pub fn with_idem_window(mut self, epochs: u64) -> Self {
        self.idem_window = Some(epochs);
        // A window declared over an already-populated ledger has no commit order to prune
        // by — the index was built without one — so it starts from what is there. Every
        // identity committed from here is ordered, and the window binds once the deque
        // holds more than it.
        if self.idem_order.is_empty() && !self.idem.is_empty() {
            let mut by_epoch: Vec<(Epoch, Arc<str>)> =
                self.idem.iter().map(|(k, e)| (*e, Arc::clone(k))).collect();
            by_epoch.sort_by_key(|(e, _)| *e);
            self.idem_order = by_epoch.into_iter().map(|(_, k)| k).collect();
        }
        self.prune_idem();
        self
    }

    /// How many identities the window holds. `nilestream_stats.idem_window_keys`.
    pub fn idem_window_keys(&self) -> usize {
        self.idem.len()
    }

    /// The declared window, in epochs; `None` if this ledger was built without one.
    pub fn idem_window(&self) -> Option<u64> {
        self.idem_window
    }

    fn remember_identity(&mut self, key: &str, at: Epoch) {
        let k: Arc<str> = Arc::from(key);
        // **The commit-order index exists only for a declared window.** With no window
        // nothing is ever pruned, so the deque would be a second pointer to every identity
        // in the ledger's history for no purpose — measured at +12% bytes per transaction
        // (E18 `append_in_memory`) before this branch existed. A cost that buys nothing is
        // not a cost this path should pay.
        if self.idem_window.is_some() {
            self.idem_order.push_back(Arc::clone(&k));
        }
        self.idem.insert(k, at);
        self.prune_idem();
    }

    /// Drop identities that have fallen out of the window.
    ///
    /// One transaction is one epoch, so the deque's length is the number of epochs held and
    /// the bound is exact rather than amortised. Popping the front is O(1): the deque is in
    /// commit order because epochs are, which is the one thing an append-only ledger can
    /// always be relied on for.
    fn prune_idem(&mut self) {
        let Some(w) = self.idem_window else { return };
        let w = w.max(1) as usize;
        while self.idem_order.len() > w {
            if let Some(old) = self.idem_order.pop_front() {
                self.idem.remove(&old);
            }
        }
    }

    /// Admission: idempotency, then the commit rule (per (txn, currency) sum-zero), then
    /// hold-resolution validity; then seal the batch as one epoch.
    pub fn submit(&mut self, key: &str, rows: Vec<Row>) -> Result<Epoch, Reject> {
        if self.idem.contains_key(key) {
            return Err(Reject::Duplicate);
        }

        // The commit rule. Note the quantification over currency: a transaction whose
        // amounts cancel only by mixing currencies is rejected.
        let mut sums: HashMap<(u64, Cur), Minor> = HashMap::new();
        for r in &rows {
            if let Row::Post(p) = r {
                let slot = sums.entry((p.txn, p.cur)).or_insert(0);
                *slot = match slot.checked_add(p.amt) {
                    Some(v) => v,
                    None => return Err(Reject::Overflow),
                };
            }
        }
        if sums.values().any(|s| *s != 0) {
            return Err(Reject::Unbalanced);
        }

        for r in &rows {
            if let Row::Resolve { hold, .. } = r {
                if !self.hold_index.contains_key(hold) || self.resolved_holds.contains(hold) {
                    return Err(Reject::BadResolution);
                }
            }
        }

        let parent = self.epochs.last().map(|e| e.hash).unwrap_or([0; 32]);
        let hash = self.chain(self.epochs.len() as u64, &parent, &rows);
        let id = self.epochs.len() as Epoch;

        // Definition 3.9: a checkpoint is `(e, V*(e)[k])` — the key's value at the *end*
        // of epoch e. Recording one mid-epoch, at the running value after some posting,
        // makes the pair a lie: reconstruction resumes strictly after `cp_epoch` and so
        // skips every later posting on that key within the same epoch. A transaction with
        // two legs on one (account, currency) — a fee beside a principal, a self-transfer,
        // any batched epoch — is enough to lose one.
        let mut crossed: Vec<(Acct, Cur)> = Vec::new();
        for (i, r) in rows.iter().enumerate() {
            let rr = RowRef {
                epoch: id,
                idx: i as u32,
            };
            match r {
                Row::Post(p) => {
                    self.by_account.entry(p.acct).or_default().push(rr);
                    // `None` means checkpointing is off, not "guard a division": making
                    // the interval's non-zero-ness a type fact says which.
                    if let Some(interval) = std::num::NonZeroUsize::new(self.checkpoint_interval) {
                        let key = (p.acct, p.cur);
                        let run = self.running.entry(key).or_insert(0);
                        *run += p.amt;
                        let seen = self.posting_seen.entry(key).or_insert(0);
                        let before = *seen / interval;
                        *seen += 1;
                        if *seen / interval > before && !crossed.contains(&key) {
                            crossed.push(key);
                        }
                    }
                }
                Row::Hold(h) => {
                    self.holds_by_account.entry(h.acct).or_default().push(rr);
                    self.hold_index.insert(h.id, rr);
                }
                Row::Resolve { hold, .. } => {
                    self.resolved_holds.insert(*hold);
                }
            }
        }

        // One checkpoint per key per epoch, written after every row of the epoch has been
        // folded, so the recorded value is the end-of-epoch value.
        crossed.sort_unstable();
        for key in crossed {
            let run = *self.running.get(&key).unwrap_or(&0);
            self.checkpoints.entry(key).or_default().push((id, run));
        }

        self.epochs.push(EpochRec {
            id,
            parent,
            hash,
            rows,
        });
        self.remember_identity(key, id);
        Ok(id)
    }

    pub fn head(&self) -> Epoch {
        self.epochs.len().saturating_sub(1) as Epoch
    }

    pub fn len(&self) -> usize {
        self.epochs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.epochs.is_empty()
    }

    /// Total postings retained (the base never forgets, so this only grows).
    pub fn posting_count(&self) -> usize {
        self.epochs.iter().map(|e| e.rows.len()).sum()
    }

    /// **Indexed** reconstruction of one account's balance at an anchor epoch.
    ///
    /// This is the upquery path. Cost is proportional to the number of postings on *that
    /// account* up to the anchor — a workload property — not to the length of history.
    /// `rows_touched` is incremented by exactly the number of base rows read, which is the
    /// machine-independent cost unit the experiments report.
    pub fn reconstruct_balance(&self, acct: Acct, cur: Cur, anchor: Epoch) -> Minor {
        self.reconstruct_balance_counted(acct, cur, anchor).0
    }

    /// The same fold, returning **this call's own** visited-row count as well as the value.
    ///
    /// `rows_touched` is one process-global atomic, and per-operation cost was read off it
    /// as a difference across the call. Under a shared base guard two reconstructions run at
    /// once, so each one's "cost" included whatever the other visited: the figure is a sum
    /// over overlapping folds wearing the name of a single read (A9-F18). Single-threaded
    /// rows — E1 through E12, E18 — are unaffected; every `rows_touched`-derived number taken
    /// under concurrency was not.
    ///
    /// The count is accumulated in a local and published to the global total once, so the
    /// aggregate still means what it did and the per-call figure is now the caller's own.
    pub fn reconstruct_balance_counted(&self, acct: Acct, cur: Cur, anchor: Epoch) -> (Minor, u64) {
        let refs = match self.by_account.get(&acct) {
            Some(v) => v,
            None => return (0, 0),
        };
        let mut total: Minor = 0;
        let mut touched = 0u64;
        // Positions are appended in epoch order, so a binary search bounds the scan to the
        // prefix at the anchor: this is the "anchor index" of the thesis made concrete.
        let end = refs.partition_point(|r| r.epoch <= anchor);

        // Start from the newest checkpoint at or before the anchor, if one exists. A
        // checkpoint read costs one row.
        let mut start = 0usize;
        if self.checkpoint_interval > 0 {
            if let Some(cps) = self.checkpoints.get(&(acct, cur)) {
                let ci = cps.partition_point(|(e, _)| *e <= anchor);
                if ci > 0 {
                    let (cp_epoch, cp_val) = cps[ci - 1];
                    total = cp_val;
                    touched += 1;
                    start = refs.partition_point(|r| r.epoch <= cp_epoch);
                }
            }
        }

        for rr in &refs[start..end] {
            let row = &self.epochs[rr.epoch as usize].rows[rr.idx as usize];
            touched += 1;
            if let Row::Post(p) = row {
                if p.cur == cur {
                    total += p.amt;
                }
            }
        }
        self.count_rows(touched);
        (total, touched)
    }

    /// Unindexed reconstruction: fold the whole prefix. Used only as the *ablation* that
    /// shows what the anchor index buys (E5), never on the serving path.
    pub fn reconstruct_balance_scan(&self, acct: Acct, cur: Cur, anchor: Epoch) -> Minor {
        let mut total: Minor = 0;
        let mut touched = 0u64;
        for e in self.epochs.iter().take(anchor as usize + 1) {
            for row in &e.rows {
                touched += 1;
                if let Row::Post(p) = row {
                    if p.acct == acct && p.cur == cur {
                        total += p.amt;
                    }
                }
            }
        }
        self.count_rows(touched);
        total
    }

    /// Unresolved holds on an account as of an anchor (the available-balance leg).
    pub fn unresolved_holds(&self, acct: Acct, cur: Cur, anchor: Epoch) -> Minor {
        let refs = match self.holds_by_account.get(&acct) {
            Some(v) => v,
            None => return 0,
        };
        // A hold counts if placed at or before the anchor and not resolved at or before it.
        let mut resolved_upto: HashSet<u64> = HashSet::new();
        for e in self.epochs.iter().take(anchor as usize + 1) {
            for row in &e.rows {
                if let Row::Resolve { hold, .. } = row {
                    resolved_upto.insert(*hold);
                }
            }
        }
        let end = refs.partition_point(|r| r.epoch <= anchor);
        let mut total = 0;
        for rr in &refs[..end] {
            if let Row::Hold(h) = &self.epochs[rr.epoch as usize].rows[rr.idx as usize] {
                if h.cur == cur && !resolved_upto.contains(&h.id) {
                    total += h.amount;
                }
            }
        }
        self.count_rows(end as u64);
        total
    }

    /// The number of postings on an account up to an anchor — the per-key update count that
    /// the cost law says reconstruction should depend on.
    /// One account's postings, in epoch order, up to and including `anchor`.
    ///
    /// Through the **anchor index**, which is the mechanism the whole thesis is about: the
    /// alternative is a scan of history, and the difference between the two is the
    /// measured two orders of magnitude of §9.4.1.
    ///
    /// Exposed because the server needs it to restrict a circuit's source scan to the key a
    /// predicate names. That is predicate pushdown, not a second answer: filtering the
    /// source by a predicate the circuit itself applies cannot change what the circuit
    /// denotes, and `the_pushdown_and_the_full_scan_agree` holds the two together.
    pub fn postings_for(&self, acct: Acct, anchor: Epoch) -> Vec<Posting> {
        let Some(refs) = self.by_account.get(&acct) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for r in refs {
            if r.epoch > anchor {
                break;
            }
            if let Some(Row::Post(p)) = self
                .epochs
                .get(r.epoch as usize)
                .and_then(|e| e.rows.get(r.idx as usize))
            {
                out.push(p.clone());
            }
        }
        out
    }

    pub fn key_update_count(&self, acct: Acct, anchor: Epoch) -> usize {
        self.by_account
            .get(&acct)
            .map(|v| v.partition_point(|r| r.epoch <= anchor))
            .unwrap_or(0)
    }

    /// Independent oracle: the per-currency system total. Must be zero on a closed book.
    pub fn conservation(&self) -> HashMap<Cur, Minor> {
        let mut sums: HashMap<Cur, Minor> = HashMap::new();
        for e in &self.epochs {
            for r in &e.rows {
                if let Row::Post(p) = r {
                    *sums.entry(p.cur).or_insert(0) += p.amt;
                }
            }
        }
        sums
    }

    pub fn verify_chain(&self) -> bool {
        if !self.chaining {
            return true;
        }
        let mut parent = [0u8; 32];
        for (i, e) in self.epochs.iter().enumerate() {
            if e.parent != parent || e.hash != self.chain(i as u64, &parent, &e.rows) {
                return false;
            }
            parent = e.hash;
        }
        true
    }

    pub fn reset_counters(&mut self) {
        self.rows_touched
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Base rows touched by reconstruction since the last [`Ledger::reset_counters`].
    pub fn rows_touched(&self) -> u64 {
        self.rows_touched.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Add to the reconstruction counter from a shared borrow.
    ///
    /// `Relaxed` because nothing orders anything by this value: it is read after the work
    /// it counts has finished, by a caller that has already synchronised with the readers
    /// through the lock it holds. A stronger ordering would buy a guarantee no reader of
    /// this number uses.
    fn count_rows(&self, n: u64) {
        self.rows_touched
            .fetch_add(n, std::sync::atomic::Ordering::Relaxed);
    }
}

/// **This ledger, presented to the REV runtime.**
///
/// One implementation, here beside the ledger, rather than one per binary that wants it.
/// There were two — `nilestream/src/main.rs` had a `LedgerBase` wrapper and the daemon had
/// nothing, which is why the daemon's wire path never used the runtime at all — and the copy
/// in the binary searched `epochs` linearly for an epoch whose id *is* its index, making
/// every sweep quadratic.
///
/// # The key is `(account, currency)`, and that is not a detail
///
/// A balance is per currency. A one-component key would have to pick a currency to fold, and
/// the only available choices are "the first one seen" and "a constant" — both of which make
/// the per-currency conservation rule invisible from outside, which is the failure this
/// system exists to make impossible. A key with fewer than two components reconstructs
/// nothing and reports having read nothing, rather than answering about a currency nobody
/// named.
impl nilestream_core::rev::Base for Ledger {
    fn frontier(&self) -> Epoch {
        self.head()
    }

    fn reconstruct(&self, key: &nilestream_core::rev::Key, anchor: Epoch) -> (i128, u64) {
        let (Some(acct), Some(cur)) = (key.first(), key.get(1)) else {
            return (0, 0);
        };
        // **This call's own count, not a difference of a process-global counter.** The old
        // body read `rows_touched` before and after; under a shared base guard a concurrent
        // reconstruction's rows landed in between and were attributed here (A9-F18).
        self.reconstruct_balance_counted(*acct as Acct, *cur as Cur, anchor)
    }

    /// The posting count of a sealed epoch, read from the record rather than built from it.
    ///
    /// `rows` is a `Vec` the epoch already owns, so this is a length and no allocation. An
    /// epoch this ledger does not hold reports zero rows, which is the same answer
    /// `deltas_at` gives it and is why `deltas_available_from` exists separately: "no rows"
    /// and "no longer retained" are different facts and only the second is a refusal.
    fn delta_rows_at(&self, e: Epoch) -> u64 {
        self.epochs
            .get(e as usize)
            .filter(|r| r.id == e)
            .map(|rec| {
                rec.rows
                    .iter()
                    .filter(|r| matches!(r, Row::Post(_)))
                    .count() as u64
            })
            .unwrap_or(0)
    }

    fn deltas_at(&self, e: Epoch) -> Vec<(nilestream_core::rev::Key, i128)> {
        // Indexed rather than searched: an epoch's id is its position, assigned by `submit`.
        let Some(rec) = self.epochs.get(e as usize).filter(|r| r.id == e) else {
            return Vec::new();
        };
        rec.rows
            .iter()
            .filter_map(|r| match r {
                Row::Post(p) => Some((vec![p.acct as i64, p.cur as i64], p.amt)),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod checkpoint_tests {
    //! Theorem 3.7 clause (i): checkpointing changes cost and not value.
    //!
    //! Clause (ii) — the C/2 + 1 bound — is what E10 and E11 measure. Clause (i) had no
    //! test at all, and the two experiments that exercise checkpoints both run one
    //! posting per key per epoch, which is exactly the shape that cannot see the defect
    //! this module pins.

    use super::*;

    const USD: Cur = 840;

    fn post(txn: u64, acct: Acct, amt: Minor) -> Row {
        Row::Post(Posting {
            txn,
            acct,
            cur: USD,
            amt,
            valid: 0,
        })
    }

    /// **Two folds at once each report their own rows — A9-F18.**
    ///
    /// `rows_touched` is a process-global atomic, and per-read cost used to be read off it
    /// as a difference across the call. Two reconstructions under the shared base guard —
    /// the ordinary case for a mixed workload — then each reported the sum of both. This
    /// runs two folds with very different costs concurrently, many times, and requires each
    /// to report the count its own key implies, with the global total still the sum.
    #[test]
    fn two_concurrent_folds_each_report_their_own_visited_rows() {
        use std::sync::Arc;
        let mut l = Ledger::new();
        // Account 1 gets many postings; account 2 gets few. Their costs must not blend.
        for e in 0..200u64 {
            l.submit(&format!("many{e}"), vec![post(e, 1, 1), post(e, 999, -1)])
                .expect("balanced");
        }
        for e in 0..5u64 {
            l.submit(
                &format!("few{e}"),
                vec![post(1_000 + e, 2, 1), post(1_000 + e, 999, -1)],
            )
            .expect("balanced");
        }
        let head = l.head();
        let l = Arc::new(l);
        let before_global = l.rows_touched();
        const ROUNDS: u64 = 200;
        let a = {
            let l = Arc::clone(&l);
            std::thread::spawn(move || {
                let mut counts = Vec::with_capacity(ROUNDS as usize);
                for _ in 0..ROUNDS {
                    counts.push(l.reconstruct_balance_counted(1, USD, head).1);
                }
                counts
            })
        };
        let b = {
            let l = Arc::clone(&l);
            std::thread::spawn(move || {
                let mut counts = Vec::with_capacity(ROUNDS as usize);
                for _ in 0..ROUNDS {
                    counts.push(l.reconstruct_balance_counted(2, USD, head).1);
                }
                counts
            })
        };
        let (ca, cb) = (a.join().expect("a"), b.join().expect("b"));
        let (fa, fb) = (ca[0], cb[0]);
        assert_eq!(fa, 200, "account 1 has 200 postings at the head");
        assert_eq!(fb, 5, "account 2 has 5");
        assert!(
            ca.iter().all(|&c| c == fa),
            "the busy fold's count must not vary with what another thread was doing: {ca:?}"
        );
        assert!(
            cb.iter().all(|&c| c == fb),
            "the cheap fold's count must not vary with what another thread was doing: {cb:?}"
        );
        assert_eq!(
            l.rows_touched() - before_global,
            ROUNDS * (fa + fb),
            "the global total is still the sum of every fold's own rows"
        );
    }

    #[test]
    fn f04_checkpoint_mid_epoch_does_not_skip_postings() {
        // Three postings on one (account, currency) inside one epoch, with an interval
        // that falls between them. A checkpoint written at the running value after the
        // second posting, and resumed strictly after its epoch, loses the third.
        let mut l = Ledger::with_checkpoints(2);
        l.submit(
            "batch",
            vec![
                post(1, 1, 5),
                post(1, 1, 5),
                post(1, 1, 5),
                post(1, 999, -15),
            ],
        )
        .expect("balanced");
        let anchor = l.head();
        let indexed = l.reconstruct_balance(1, USD, anchor);
        let scan = l.reconstruct_balance_scan(1, USD, anchor);
        assert_eq!(
            indexed, scan,
            "checkpointed reconstruction must return the same value as the full fold"
        );
        assert_eq!(indexed, 15);
    }

    #[test]
    fn checkpointed_reconstruction_equals_the_full_fold_at_every_anchor() {
        // Theorem 3.7(i) as a property: random histories with several postings per key
        // per epoch, four intervals, every anchor.
        for interval in [1usize, 2, 3, 16] {
            for seed in [1u64, 7, 42, 100, 2024] {
                let mut lcg = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let mut next = || {
                    lcg = lcg
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    lcg >> 33
                };
                let mut l = Ledger::with_checkpoints(interval);
                for e in 0..40u64 {
                    // One to four postings on one key in this epoch, plus the contra.
                    let acct = next() % 5;
                    let n = 1 + next() % 4;
                    let mut rows = Vec::new();
                    let mut total: Minor = 0;
                    for j in 0..n {
                        let amt = ((next() % 9) as Minor) - 4;
                        total += amt;
                        rows.push(post(e, acct, amt));
                        let _ = j;
                    }
                    rows.push(post(e, 999, -total));
                    l.submit(&format!("s{seed}-{e}"), rows).expect("balanced");
                }
                for anchor in 0..=l.head() {
                    for acct in 0..5u64 {
                        let indexed = l.reconstruct_balance(acct, USD, anchor);
                        let scan = l.reconstruct_balance_scan(acct, USD, anchor);
                        assert_eq!(
                            indexed, scan,
                            "C={interval} seed={seed} acct={acct} anchor={anchor}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_bound_holds_on_the_e10_workload() {
        // Theorem 3.7(ii): the *expected* number of base rows read is at most C/2 + 1,
        // where the expectation is over an anchor falling uniformly between two
        // checkpoints. Anchors are therefore sampled across the history rather than
        // taken at the head: reading only at the head measures one fixed offset into the
        // interval, which for 1,000 postings at C = 64 is 40 rather than the mean 32, and
        // reports a bound violation that is an artefact of where the head happens to sit.
        for interval in [16usize, 64] {
            let mut l = Ledger::with_checkpoints(interval);
            for e in 0..4000u64 {
                let acct = e % 4;
                l.submit(&format!("k{e}"), vec![post(e, acct, 1), post(e, 999, -1)])
                    .expect("balanced");
            }
            let head = l.head();
            let before = l.rows_touched();
            let mut reads = 0u64;
            let mut lcg = 12345u64;
            for i in 0..400u64 {
                lcg = lcg
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let anchor = (lcg >> 33) % (head + 1);
                l.reconstruct_balance(i % 4, USD, anchor);
                reads += 1;
            }
            let mean = (l.rows_touched() - before) as f64 / reads as f64;
            let bound = interval as f64 / 2.0 + 1.0;
            assert!(
                mean < bound + 1.0,
                "C={interval}: mean {mean:.2} base rows per reconstruction against a \
                 bound of {bound:.1}"
            );
        }
    }
}

#[cfg(test)]
mod row_encoding_tests {
    //! **One encoding, two consumers.**
    //!
    //! The durable record carries `encode_rows`; the chain hashes `write_rows`. They are the
    //! same bytes today because the first is written in terms of the second, and this module
    //! is what will notice if that ever stops being true — a divergence would not fail a
    //! build, it would make every replayed epoch refuse to verify against a record that is
    //! perfectly honest, which is the worst shape a durability bug can take.

    use super::*;

    fn sample() -> Vec<Row> {
        vec![
            Row::Post(Posting {
                txn: 7,
                acct: 3,
                cur: 1,
                amt: -250,
                valid: 42,
            }),
            Row::Hold(Hold {
                id: 9,
                acct: 4,
                cur: 0,
                amount: 1_000,
            }),
            Row::Resolve {
                hold: 9,
                outcome: Outcome::Post(600),
            },
            Row::Resolve {
                hold: 11,
                outcome: Outcome::Void,
            },
            Row::Resolve {
                hold: 12,
                outcome: Outcome::Expire,
            },
        ]
    }

    #[test]
    fn encode_rows_matches_the_streamed_form() {
        let rows = sample();
        let buffered = encode_rows(&rows);
        let mut streamed = Vec::new();
        write_rows(&rows, &mut streamed);
        assert_eq!(
            buffered, streamed,
            "the payload's bytes and the chain's bytes must be the same bytes"
        );
    }

    #[test]
    fn the_hash_is_the_hash_of_those_bytes() {
        let rows = sample();
        let mut direct = Hasher256::new();
        direct.update(&encode_rows(&rows));
        let mut streamed = Hasher256::new();
        write_rows(&rows, &mut streamed);
        assert_eq!(
            direct.finalize(),
            streamed.finalize(),
            "streaming into the hasher must not change the digest — if it did, every ledger \
             written before this change would stop verifying"
        );
    }

    #[test]
    fn every_variant_round_trips() {
        let rows = sample();
        let bytes = encode_rows(&rows);
        let back = decode_rows(&bytes).expect("the decoder must accept what the encoder writes");
        // Compared through the encoding rather than through `PartialEq`, which `Row` does not
        // have: re-encoding equal bytes is the property the chain actually depends on, since
        // what a replay compares is the digest and not the values.
        assert_eq!(
            encode_rows(&back),
            bytes,
            "a decoded row must re-encode to the bytes it came from, for every variant"
        );
        assert_eq!(back.len(), rows.len(), "and must not lose or invent a row");
    }
}

#[cfg(test)]
mod chain_tests {
    //! **`verify_chain` had no unit test, and that is how it broke.**
    //!
    //! The chain was checked by E1 — which is a ten-minute experiment run, is not part of
    //! `cargo test`, and reports through a generated table nobody re-reads on every commit.
    //! So when T-01 put the epoch number into the digest and `chain` took it from
    //! `self.epochs.len()`, the seal path stayed correct and the verify path silently stopped
    //! agreeing with it, for every epoch, on every seed. The workspace was green.
    //!
    //! These tests are the cheap version of that check: a few epochs, in-process, on the two
    //! properties the chain has to have — an honest ledger verifies, and a ledger whose rows
    //! were changed afterwards does not.

    use super::*;

    fn transfer(txn: u64, from: Acct, to: Acct, amt: Minor) -> Vec<Row> {
        vec![
            Row::Post(Posting {
                txn,
                acct: from,
                cur: 0,
                amt: -amt,
                valid: 0,
            }),
            Row::Post(Posting {
                txn,
                acct: to,
                cur: 0,
                amt,
                valid: 0,
            }),
        ]
    }

    fn built(n: u64) -> Ledger {
        let mut l = Ledger {
            chaining: true,
            ..Default::default()
        };
        for i in 1..=n {
            l.submit(&format!("k-{i}"), transfer(i, 1, 2, 10))
                .expect("balanced");
        }
        l
    }

    #[test]
    fn an_honestly_built_ledger_verifies_at_every_length() {
        // Every length, because the defect this pins is an off-by-position: a formula that
        // reads the ledger's *current* length agrees with itself at exactly one epoch, and a
        // single-epoch ledger is the one place it cannot be caught.
        for n in 1..=6 {
            let l = built(n);
            assert!(
                l.verify_chain(),
                "a ledger of {n} epoch(s), built by this code and verified by this code, must \
                 verify — if it does not, the seal and the check are computing different \
                 hashes and neither is authoritative"
            );
        }
    }

    #[test]
    fn changing_a_row_after_the_fact_breaks_the_chain() {
        let mut l = built(4);
        assert!(l.verify_chain());
        match &mut l.epochs[1].rows[0] {
            Row::Post(p) => p.amt += 1,
            _ => unreachable!("the first row of a transfer is a posting"),
        }
        assert!(
            !l.verify_chain(),
            "a row altered after its epoch was sealed must break the link — this is the whole \
             claim the hash chain makes"
        );
    }

    #[test]
    fn moving_an_epoch_breaks_the_chain() {
        // The epoch number is in the digest so that two epochs with identical rows are not
        // interchangeable. Swapping them keeps every row, every parent field and every hash
        // field intact, so nothing but the position has changed.
        let mut l = built(4);
        l.epochs.swap(1, 2);
        assert!(
            !l.verify_chain(),
            "reordering sealed epochs must break the chain, or the ledger's order is not \
             part of what it commits to"
        );
    }
}

/// **The idempotency window: what it bounds, and what it gives up.**
///
/// Its own module because it is its own mechanism — not a checkpoint, not the commit rule —
/// and the two tests below are the pair the design has to be judged on together: the index
/// is bounded, *and* a retry older than the bound commits again.
#[cfg(test)]
mod idem_window_tests {
    use super::*;

    const USD: Cur = 840;

    fn post(txn: u64, acct: Acct, amt: Minor) -> Row {
        Row::Post(Posting {
            txn,
            acct,
            cur: USD,
            amt,
            valid: 0,
        })
    }

    /// **A key older than the window is admitted as new — T-05.3.**
    ///
    /// The window is what makes the idempotency index bounded, and this is the behaviour it
    /// buys and pays for in the same breath: inside the window a retry is refused and told
    /// nothing committed twice; outside it, the same key commits again, because the ledger
    /// no longer holds the evidence that it ever committed. An `IdemKey` without a window is
    /// a uniqueness constraint over all of history, which is exactly the structure that grew
    /// at 68.8 B per identity forever.
    #[test]
    fn a_key_older_than_the_window_is_new() {
        const WINDOW: u64 = 4;
        let mut l = Ledger::default().with_idem_window(WINDOW);
        for i in 0..10u64 {
            l.submit(&format!("k{i}"), vec![post(i, 1, 5), post(i, 999, -5)])
                .expect("balanced and new");
        }
        assert_eq!(
            l.idem_window_keys(),
            WINDOW as usize,
            "the window must bound the index: {} identities held against a window of {WINDOW}",
            l.idem_window_keys()
        );
        assert_eq!(
            l.submit("k9", vec![post(99, 1, 5), post(99, 999, -5)]),
            Err(Reject::Duplicate),
            "`k9` is inside the window and its retry must be refused"
        );
        assert!(
            l.submit("k0", vec![post(100, 1, 5), post(100, 999, -5)])
                .is_ok(),
            "`k0` has fallen out of the window, so the ledger no longer knows it committed \
             and admits it as new. That is what a window is, and why it is declared."
        );
    }

    /// Without a declared window nothing is pruned — stated as a test rather than as an
    /// absence, so that the default is visible and a change to it is a failing assertion.
    #[test]
    fn no_declared_window_keeps_every_identity() {
        let mut l = Ledger::default();
        for i in 0..10u64 {
            l.submit(&format!("k{i}"), vec![post(i, 1, 5), post(i, 999, -5)])
                .expect("balanced and new");
        }
        assert_eq!(l.idem_window(), None);
        assert_eq!(l.idem_window_keys(), 10);
        assert_eq!(
            l.submit("k0", vec![post(100, 1, 5), post(100, 999, -5)]),
            Err(Reject::Duplicate),
            "with no window, the first identity is still known"
        );
    }
}
