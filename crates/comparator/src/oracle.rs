//! The answers every arm is held to, computed from the generated transactions by a fold
//! this crate owns — independent of Nilestream, of PostgreSQL, and of the E14 mechanism.
//!
//! An arm that disagrees with this on any checked answer is not measured: E27 refuses the
//! comparison (§3's third refusal condition), because a latency of a wrong answer is not a
//! measurement of anything.

use crate::universe::{Txn, Universe};
use std::collections::BTreeMap;

/// Balance key: (account, currency).
pub type Key = (u64, u32);

/// The applied history, indexed for the questions the arms are asked.
pub struct Oracle {
    /// Every transaction applied so far, in order, with its epoch **index** — the batch for
    /// the history, `batches + w` for the w-th measured write. Arms map an index to their
    /// own epoch numbering; this is the only numbering the oracle knows.
    applied: Vec<(u64, Txn)>,
    /// Per key, the (epoch index, amount, value day) of every leg, in epoch order.
    by_key: BTreeMap<Key, Vec<(u64, i64, i32)>>,
    head: u64,
}

impl Oracle {
    pub fn of(u: &Universe) -> Oracle {
        let mut o = Oracle {
            applied: Vec::new(),
            by_key: BTreeMap::new(),
            head: 0,
        };
        for t in &u.txns {
            o.apply(t.batch, t.clone());
        }
        o
    }

    /// Record a transaction sealed at epoch index `idx` (monotone).
    pub fn apply(&mut self, idx: u64, t: Txn) {
        assert!(idx >= self.head, "epochs are applied in order");
        for l in t.legs {
            self.by_key
                .entry((l.acct, l.cur))
                .or_default()
                .push((idx, l.amt, t.value_day));
        }
        self.head = idx;
        self.applied.push((idx, t));
    }

    /// The checksum of every leg applied so far, history and measured writes, in the one
    /// canonical form (`universe::checksum_of`). Every arm is held to it after the run as
    /// well as after the load, so an arm that dropped or doubled a write cannot finish.
    pub fn checksum(&self) -> (u64, String) {
        let mut legs: Vec<crate::universe::Leg> =
            self.applied.iter().flat_map(|(_, t)| t.legs).collect();
        crate::universe::checksum_of(&mut legs)
    }

    pub fn head(&self) -> u64 {
        self.head
    }

    /// The balance of `key` through epoch index `at` (inclusive). `None` if the key has no
    /// leg at or before `at` — absent, not zero.
    pub fn balance(&self, key: Key, at: u64) -> Option<i128> {
        let legs = self.by_key.get(&key)?;
        let n = legs.partition_point(|&(e, _, _)| e <= at);
        if n == 0 {
            return None;
        }
        Some(legs[..n].iter().map(|&(_, a, _)| a as i128).sum())
    }

    /// Every key's balance at the head.
    pub fn report(&self) -> BTreeMap<Key, i128> {
        self.by_key
            .iter()
            .map(|(k, legs)| (*k, legs.iter().map(|&(_, a, _)| a as i128).sum()))
            .collect()
    }

    /// The ten largest balances in currency `cur` at the head, ties broken by account so the
    /// answer is a set with one order. Arms are compared on the multiset of balances, since
    /// `order by sum desc limit 10` does not fix which of two equal balances comes first.
    pub fn top10(&self, cur: u32) -> Vec<i128> {
        let mut v: Vec<i128> = self
            .report()
            .into_iter()
            .filter(|((_, c), _)| *c == cur)
            .map(|(_, b)| b)
            .collect();
        v.sort_by(|a, b| b.cmp(a));
        v.truncate(10);
        v
    }

    /// q3, one account's statement: the number of legs on `key` whose value day is in
    /// `[from, to]`, and their sum (`None` when there are none — absent, not zero).
    pub fn statement(&self, key: Key, from: i32, to: i32) -> (u64, Option<i128>) {
        let Some(legs) = self.by_key.get(&key) else {
            return (0, None);
        };
        let hit: Vec<i128> = legs
            .iter()
            .filter(|&&(_, _, d)| d >= from && d <= to)
            .map(|&(_, a, _)| a as i128)
            .collect();
        let n = hit.len() as u64;
        (n, if n == 0 { None } else { Some(hit.iter().sum()) })
    }

    /// q4, desk exposure by currency: per currency, the sum of every balance booked to `desk`.
    pub fn desk(&self, desk: u64) -> BTreeMap<u32, i128> {
        let mut out = BTreeMap::new();
        for ((acct, cur), legs) in &self.by_key {
            if crate::universe::desk_of(*acct) == desk {
                *out.entry(*cur).or_default() +=
                    legs.iter().map(|&(_, a, _)| a as i128).sum::<i128>();
            }
        }
        out
    }

    /// Every key's balance through epoch index `at` — the report a replica that had applied
    /// exactly the epochs through `at` would give. Keys with no leg by then are absent.
    pub fn report_at(&self, at: u64) -> BTreeMap<Key, i128> {
        self.by_key
            .keys()
            .filter_map(|k| self.balance(*k, at).map(|b| (*k, b)))
            .collect()
    }

    /// q5 through epoch index `at`.
    pub fn top10_at(&self, cur: u32, at: u64) -> Vec<i128> {
        let mut v: Vec<i128> = self
            .report_at(at)
            .into_iter()
            .filter(|((_, c), _)| *c == cur)
            .map(|(_, b)| b)
            .collect();
        v.sort_by(|a, b| b.cmp(a));
        v.truncate(10);
        v
    }

    /// q3 through epoch index `at`.
    pub fn statement_at(&self, key: Key, from: i32, to: i32, at: u64) -> (u64, Option<i128>) {
        let Some(legs) = self.by_key.get(&key) else {
            return (0, None);
        };
        let hit: Vec<i128> = legs
            .iter()
            .filter(|&&(e, _, d)| e <= at && d >= from && d <= to)
            .map(|&(_, a, _)| a as i128)
            .collect();
        let n = hit.len() as u64;
        (n, if n == 0 { None } else { Some(hit.iter().sum()) })
    }

    /// q4 through epoch index `at`.
    pub fn desk_at(&self, desk: u64, at: u64) -> BTreeMap<u32, i128> {
        let mut out = BTreeMap::new();
        for (acct, cur) in self.by_key.keys() {
            if crate::universe::desk_of(*acct) == desk {
                if let Some(b) = self.balance((*acct, *cur), at) {
                    *out.entry(*cur).or_default() += b;
                }
            }
        }
        out
    }

    /// The epoch index of `key`'s first leg — the earliest anchor at which it has a balance.
    pub fn first_epoch(&self, key: Key) -> Option<u64> {
        self.by_key.get(&key)?.first().map(|&(e, _, _)| e)
    }

    /// The amounts of `key`'s legs sealed at or before epoch index `at` — what a delta that
    /// was applied twice, or skipped, would add to or remove from an answer.
    pub fn legs_through(&self, key: Key, at: u64) -> Vec<i64> {
        self.by_key
            .get(&key)
            .map(|legs| {
                legs.iter()
                    .take_while(|&&(e, _, _)| e <= at)
                    .map(|&(_, a, _)| a)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The keys of one account.
    pub fn keys_of(&self, acct: u64) -> Vec<Key> {
        self.by_key
            .range((acct, 0)..=(acct, u32::MAX))
            .map(|(k, _)| *k)
            .collect()
    }

    /// How many (account, currency) keys exist.
    pub fn key_count(&self) -> usize {
        self.by_key.len()
    }

    /// Keys that exist at the head, for drawing reads that have an answer.
    pub fn keys(&self) -> Vec<Key> {
        self.by_key.keys().copied().collect()
    }

    /// Conservation: every currency nets to zero over the whole ledger.
    pub fn conserves(&self) -> bool {
        let mut per_cur: BTreeMap<u32, i128> = BTreeMap::new();
        for ((_, c), b) in self.report() {
            *per_cur.entry(c).or_default() += b;
        }
        per_cur.values().all(|&v| v == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::{Params, Universe};

    #[test]
    fn the_oracle_conserves_and_answers_history_and_head_consistently() {
        let u = Universe::generate(&Params::declared(300, 5, 0.6, true));
        let o = Oracle::of(&u);
        assert!(o.conserves());
        let head = o.head();
        assert_eq!(o.report_at(head), o.report());
        assert_eq!(o.top10_at(0, head), o.top10(0));
        assert_eq!(o.desk_at(3, head), o.desk(3));
        let k0 = o.keys()[0];
        assert_eq!(
            o.statement_at(k0, -100, 1000, head),
            o.statement(k0, -100, 1000)
        );
        // Before the last batch, some key's balance differs: the `_at` forms really do stop.
        assert_ne!(o.report_at(head - 1), o.report());
        for k in o.keys().into_iter().take(50) {
            assert_eq!(o.balance(k, head), o.report().get(&k).copied());
            // The balance at epoch 0 is absent: nothing is sealed before batch 1.
            assert_eq!(o.balance(k, 0), None);
        }
    }
}
