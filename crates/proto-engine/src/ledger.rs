//! The authoritative base: append-only, epoch-ordered, hash-chained, never partial.
//!
//! Two structures matter for the experiments:
//!   * `epochs` — the immutable sequence of sealed epochs (the base itself);
//!   * `by_account` — a per-key **anchor index** mapping an account to the positions of its
//!     postings. This is the structure that makes a reconstruction cost proportional to a
//!     key's own update count rather than to the length of history, and it is what the
//!     history-independence experiment (E5) tests.

use std::collections::{HashMap, HashSet};

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
    idem: HashSet<String>,
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
    pub rows_touched: u64,
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

    fn chain(&self, parent: &[u8; 32], rows: &[Row]) -> [u8; 32] {
        if !self.chaining {
            return [0; 32];
        }
        let mut h = Hasher256::new();
        h.update(parent);
        for r in rows {
            match r {
                Row::Post(p) => {
                    h.update(b"P");
                    h.update(&p.txn.to_le_bytes());
                    h.update(&p.acct.to_le_bytes());
                    h.update(&p.cur.to_le_bytes());
                    h.update(&p.amt.to_le_bytes());
                    h.update(&p.valid.to_le_bytes());
                }
                Row::Hold(hd) => {
                    h.update(b"H");
                    h.update(&hd.id.to_le_bytes());
                    h.update(&hd.acct.to_le_bytes());
                    h.update(&hd.cur.to_le_bytes());
                    h.update(&hd.amount.to_le_bytes());
                }
                Row::Resolve { hold, outcome } => {
                    h.update(b"R");
                    h.update(&hold.to_le_bytes());
                    match outcome {
                        Outcome::Post(a) => {
                            h.update(b"p");
                            h.update(&a.to_le_bytes());
                        }
                        Outcome::Void => {
                            h.update(b"v");
                        }
                        Outcome::Expire => {
                            h.update(b"e");
                        }
                    }
                }
            }
        }
        h.finalize()
    }

    /// Admission: idempotency, then the commit rule (per (txn, currency) sum-zero), then
    /// hold-resolution validity; then seal the batch as one epoch.
    pub fn submit(&mut self, key: &str, rows: Vec<Row>) -> Result<Epoch, Reject> {
        if self.idem.contains(key) {
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
        let hash = self.chain(&parent, &rows);
        let id = self.epochs.len() as Epoch;

        for (i, r) in rows.iter().enumerate() {
            let rr = RowRef {
                epoch: id,
                idx: i as u32,
            };
            match r {
                Row::Post(p) => {
                    self.by_account.entry(p.acct).or_default().push(rr);
                    if self.checkpoint_interval > 0 {
                        let key = (p.acct, p.cur);
                        let run = self.running.entry(key).or_insert(0);
                        *run += p.amt;
                        let seen = self.posting_seen.entry(key).or_insert(0);
                        *seen += 1;
                        if (*seen).is_multiple_of(self.checkpoint_interval) {
                            self.checkpoints.entry(key).or_default().push((id, *run));
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

        self.epochs.push(EpochRec {
            id,
            parent,
            hash,
            rows,
        });
        self.idem.insert(key.to_string());
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
    pub fn reconstruct_balance(&mut self, acct: Acct, cur: Cur, anchor: Epoch) -> Minor {
        let refs = match self.by_account.get(&acct) {
            Some(v) => v,
            None => return 0,
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
        self.rows_touched += touched;
        total
    }

    /// Unindexed reconstruction: fold the whole prefix. Used only as the *ablation* that
    /// shows what the anchor index buys (E5), never on the serving path.
    pub fn reconstruct_balance_scan(&mut self, acct: Acct, cur: Cur, anchor: Epoch) -> Minor {
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
        self.rows_touched += touched;
        total
    }

    /// Unresolved holds on an account as of an anchor (the available-balance leg).
    pub fn unresolved_holds(&mut self, acct: Acct, cur: Cur, anchor: Epoch) -> Minor {
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
        self.rows_touched += end as u64;
        total
    }

    /// The number of postings on an account up to an anchor — the per-key update count that
    /// the cost law says reconstruction should depend on.
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
        for e in &self.epochs {
            if e.parent != parent || e.hash != self.chain(&parent, &e.rows) {
                return false;
            }
            parent = e.hash;
        }
        true
    }

    pub fn reset_counters(&mut self) {
        self.rows_touched = 0;
    }
}
