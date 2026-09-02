*Extracted from `crates/conservation-suite/src/oracle.rs`. Do not edit by hand.*

```rust
    /// Admission: idempotency, then the commit rule (per (txn, currency) balance), then
    /// seal immediately. The oracle's epoch is one batch; the epoch period is a
    /// performance knob in the real engine and is irrelevant to the semantics here.
    pub fn submit(&mut self, key: &str, rows: Vec<Row>) -> Result<u64, Reject> {
        if self.idem.contains(key) {
            return Err(Reject::Duplicate);
        }

        // The commit rule: every (txn, currency) group sums to zero. Note the
        // quantification over currency — a transaction moving -100 in one currency and
        // +100 in another nets to zero arithmetically and is still rejected.
        let mut sums: HashMap<(u64, Cur), Minor> = HashMap::new();
        for r in &rows {
            if let Row::Post(p) = r {
                *sums.entry((p.txn, p.cur)).or_insert(0) += p.amt;
            }
        }
        if sums.values().any(|s| *s != 0) {
            return Err(Reject::Unbalanced);
        }

        // A resolution must refer to a hold that exists and is not already resolved.
        for r in &rows {
            if let Row::Resolve { hold, .. } = r {
                if !self.hold_exists(*hold) || self.hold_is_resolved(*hold) {
                    return Err(Reject::BadResolution);
                }
            }
        }

        let parent = self.epochs.last().map(|e| e.hash).unwrap_or([0; 32]);
        let hash = chain(&parent, &rows);
        let id = self.epochs.len() as u64;
        self.epochs.push(EpochRec {
            id,
            parent,
            hash,
            rows,
        });
        self.idem.insert(key.to_string());
        Ok(id)
    }

    fn rows_upto(&self, anchor: u64) -> impl Iterator<Item = &Row> {
        // `saturating_add`: an anchor of `u64::MAX` means "everything retained", and
        // panicking on it would make the oracle refuse the one question it exists to
        // answer. The oracle is the definition of correctness; it does not get to abort.
        self.epochs
            .iter()
            .take((anchor as usize).saturating_add(1))
            .flat_map(|e| &e.rows)
    }

    fn hold_exists(&self, id: u64) -> bool {
        self.epochs
            .iter()
            .flat_map(|e| &e.rows)
            .any(|r| matches!(r, Row::Hold { id: h, .. } if *h == id))
    }

    fn hold_is_resolved(&self, id: u64) -> bool {
        self.epochs
            .iter()
            .flat_map(|e| &e.rows)
            .any(|r| matches!(r, Row::Resolve { hold, .. } if *hold == id))
    }

    /// The ledger balance: settled postings only. This is the regulator's "ledger
    /// balance" — computed from settled transactions, taking no account of holds.
    ///
    /// Every read is a fold of a prefix. No caches, no indexes, no cleverness.
    pub fn ledger_balance(&self, a: Acct, c: Cur, anchor: u64) -> Minor {
        self.rows_upto(anchor)
            .filter_map(|r| match r {
                Row::Post(p) if p.acct == a && p.cur == c => Some(p.amt),
                _ => None,
            })
            .sum()
    }
```
