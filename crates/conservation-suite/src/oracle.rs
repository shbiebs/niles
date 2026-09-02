//! The runnable reference oracle O (thesis 3.11, Appendix F).
//!
//! This program *defines* correctness for the whole system: Nilestream is correct iff
//! every observable trace of Nilestream is an observable trace of this fold, under the
//! declared per-view contracts.
//!
//! Design rules, deliberately obeyed (thesis F.1):
//!   1. No optimization of any kind — the base is a Vec, every read folds a prefix.
//!   2. No shared mutable state — single-threaded, message in, answer out.
//!   3. Exact integer money at the currency's own scale; never floats, never a hard-coded
//!      scale of 2 (real ISO 4217 minor units range 0..=3, and some codes have none).
//!   4. It implements the transition system of thesis 3.11 and nothing else.
//!
//! Holds are rows, not fields: a reservation is an appended row and its resolution is
//! another appended row, so the oracle has no update path at all, and the regulator's
//! distinction between the ledger balance and the available balance is two folds over the
//! same base rather than two stored numbers.

use std::collections::{HashMap, HashSet};

use nilestream_ledger::chain::Hasher256;

/// Exact minor units at the currency's declared scale.
pub type Minor = i128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cur(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Acct(pub u64);

/// A currency's declared scale (ISO 4217 minor-unit exponent): 0 for JPY, 2 for USD,
/// 3 for KWD. A `Money` type fixed at scale 2 is provably wrong (thesis 3.2).
///
/// **Declared and unused, deliberately, and this comment is the record of that.** The oracle
/// holds amounts in minor units and never renders them, so it needs no scale; the type is
/// here because Appendix F's design rule 3 says the oracle must not hard-code a scale of
/// two, and a reader checking that rule against the code should find the concept named
/// rather than infer its absence. Every amount that reaches this oracle has had its scale
/// checked by whatever produced it — `CurrencySums` in GBS, the currency row in the
/// checker — and the oracle's job is to fold, not to re-derive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale(pub u8);

#[derive(Debug, Clone)]
pub struct Posting {
    pub txn: u64,
    pub acct: Acct,
    pub cur: Cur,
    pub amt: Minor,
    /// Valid time (the world's calendar); system time is the epoch index.
    pub valid: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Captured for this amount; any remainder is released.
    Post(Minor),
    Void,
    Expire,
}

/// A base row. There is no update variant, by design.
#[derive(Debug, Clone)]
pub enum Row {
    Post(Posting),
    Hold {
        id: u64,
        acct: Acct,
        cur: Cur,
        amount: Minor,
        valid: i64,
    },
    Resolve {
        hold: u64,
        outcome: Outcome,
        valid: i64,
    },
}

#[derive(Debug, Clone)]
pub struct EpochRec {
    pub id: u64,
    pub parent: [u8; 32],
    pub hash: [u8; 32],
    pub rows: Vec<Row>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reject {
    /// The idempotency key was already admitted within the declared window.
    Duplicate,
    /// The commit rule failed: some (txn, currency) group does not sum to zero.
    Unbalanced,
    /// A resolution referred to a hold that does not exist or is already resolved.
    BadResolution,
}

#[derive(Default)]
pub struct Oracle {
    pub epochs: Vec<EpochRec>,
    idem: HashSet<String>,
}

fn chain(parent: &[u8; 32], rows: &[Row]) -> [u8; 32] {
    let mut h = Hasher256::new();
    h.update(parent);
    for r in rows {
        match r {
            Row::Post(p) => {
                h.update(b"P");
                h.update(&p.txn.to_le_bytes());
                h.update(&p.acct.0.to_le_bytes());
                h.update(&p.cur.0.to_le_bytes());
                h.update(&p.amt.to_le_bytes());
                h.update(&p.valid.to_le_bytes());
            }
            Row::Hold {
                id,
                acct,
                cur,
                amount,
                valid,
            } => {
                h.update(b"H");
                h.update(&id.to_le_bytes());
                h.update(&acct.0.to_le_bytes());
                h.update(&cur.0.to_le_bytes());
                h.update(&amount.to_le_bytes());
                h.update(&valid.to_le_bytes());
            }
            Row::Resolve {
                hold,
                outcome,
                valid,
            } => {
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
                h.update(&valid.to_le_bytes());
            }
        }
    }
    h.finalize()
}

impl Oracle {
    pub fn new() -> Self {
        Self::default()
    }

    // BEGIN:appendix-f
    //
    // Everything between these markers is reproduced verbatim in Appendix F.2 of the thesis
    // by `thesis/gen-appendix-f.py`, and `tests/appendix_f.rs` fails if the two differ. The
    // appendix used to carry a hand-written paraphrase that no longer compiled: its fold
    // signature had drifted from this one and it declared a type nothing used. An appendix
    // presenting itself as "the one component implemented and passing tests today" has to be
    // the code that is passing them.
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

    // END:appendix-f

    /// The total of holds placed but not yet resolved, as of `anchor`.
    pub fn unresolved_holds(&self, a: Acct, c: Cur, anchor: u64) -> Minor {
        let resolved: HashSet<u64> = self
            .rows_upto(anchor)
            .filter_map(|r| match r {
                Row::Resolve { hold, .. } => Some(*hold),
                _ => None,
            })
            .collect();

        self.rows_upto(anchor)
            .filter_map(|r| match r {
                Row::Hold {
                    id,
                    acct,
                    cur,
                    amount,
                    ..
                } if *acct == a && *cur == c && !resolved.contains(id) => Some(*amount),
                _ => None,
            })
            .sum()
    }

    /// Available balance = posted minus unresolved holds.
    ///
    /// Two views over one base, differing only in which rows their folds admit. The gap
    /// between them at a decision point is exactly the hazard supervisory guidance
    /// describes (thesis 3.19).
    pub fn available_balance(&self, a: Acct, c: Cur, anchor: u64) -> Minor {
        self.ledger_balance(a, c, anchor) - self.unresolved_holds(a, c, anchor)
    }

    /// "What did we believe at system epoch `sys` about valid time <= `valid_upto`?"
    /// Two filters, one per time axis — the entire implementation of bitemporality.
    pub fn bitemporal(&self, a: Acct, c: Cur, sys: u64, valid_upto: i64) -> Minor {
        self.rows_upto(sys)
            .filter_map(|r| match r {
                Row::Post(p) if p.acct == a && p.cur == c && p.valid <= valid_upto => Some(p.amt),
                _ => None,
            })
            .sum()
    }

    /// The global invariant: per currency, everything sums to zero.
    pub fn conservation_ok(&self, anchor: u64) -> bool {
        let mut sums: HashMap<Cur, Minor> = HashMap::new();
        for r in self.rows_upto(anchor) {
            if let Row::Post(p) = r {
                *sums.entry(p.cur).or_insert(0) += p.amt;
            }
        }
        sums.values().all(|s| *s == 0)
    }

    /// Tamper evidence: recompute the chain forward and compare.
    ///
    /// Note the guarantee this provides, stated precisely (thesis 1.9): detection by a
    /// verifier holding a previously obtained digest — not prevention.
    pub fn verify_chain(&self) -> bool {
        let mut parent = [0u8; 32];
        self.epochs.iter().all(|e| {
            let ok = e.parent == parent && e.hash == chain(&parent, &e.rows);
            parent = e.hash;
            ok
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USD: Cur = Cur(840);
    const MXN: Cur = Cur(484);
    const JPY: Cur = Cur(392);

    fn p(txn: u64, acct: u64, cur: Cur, amt: Minor) -> Row {
        Row::Post(Posting {
            txn,
            acct: Acct(acct),
            cur,
            amt,
            valid: 0,
        })
    }

    fn pv(txn: u64, acct: u64, cur: Cur, amt: Minor, valid: i64) -> Row {
        Row::Post(Posting {
            txn,
            acct: Acct(acct),
            cur,
            amt,
            valid,
        })
    }

    #[test]
    fn balanced_transfer_conserves() {
        let mut o = Oracle::new();
        let e = o
            .submit("t1", vec![p(1, 1, USD, -500), p(1, 2, USD, 500)])
            .unwrap();
        assert_eq!(o.ledger_balance(Acct(1), USD, e), -500);
        assert_eq!(o.ledger_balance(Acct(2), USD, e), 500);
        assert!(o.conservation_ok(e));
        assert!(o.verify_chain());
    }

    #[test]
    fn unbalanced_is_rejected() {
        let mut o = Oracle::new();
        assert_eq!(
            o.submit("t1", vec![p(1, 1, USD, -500), p(1, 2, USD, 499)]),
            Err(Reject::Unbalanced)
        );
        assert!(o.epochs.is_empty());
    }

    #[test]
    fn duplicate_idempotency_key_is_rejected() {
        let mut o = Oracle::new();
        o.submit("t1", vec![p(1, 1, USD, -1), p(1, 2, USD, 1)])
            .unwrap();
        assert_eq!(
            o.submit("t1", vec![p(2, 1, USD, -1), p(2, 2, USD, 1)]),
            Err(Reject::Duplicate)
        );
        assert_eq!(o.epochs.len(), 1);
    }

    #[test]
    fn fx_legs_balance_per_currency_separately() {
        let mut o = Oracle::new();
        // FX-atomic (thesis Def. 3.7): each currency's postings balance within the txn.
        let e = o
            .submit(
                "fx1",
                vec![
                    p(1, 1, USD, -100),
                    p(1, 9, USD, 100), // usd position account
                    p(1, 9, MXN, -1850),
                    p(1, 2, MXN, 1850),
                ],
            )
            .unwrap();
        assert!(o.conservation_ok(e));
    }

    #[test]
    fn mixed_currency_imbalance_rejected_even_though_total_is_zero() {
        let mut o = Oracle::new();
        assert_eq!(
            o.submit("bad", vec![p(1, 1, USD, -100), p(1, 2, MXN, 100)]),
            Err(Reject::Unbalanced)
        );
    }

    #[test]
    fn zero_scale_currency_is_representable() {
        // JPY has no minor unit; a Money type hard-coded to scale 2 could not express this.
        let mut o = Oracle::new();
        let e = o
            .submit("jpy", vec![p(1, 1, JPY, -1000), p(1, 2, JPY, 1000)])
            .unwrap();
        assert_eq!(o.ledger_balance(Acct(2), JPY, e), 1000);
        assert!(o.conservation_ok(e));
    }

    #[test]
    fn bitemporal_answers_are_total() {
        let mut o = Oracle::new();
        o.submit("a", vec![pv(0, 1, USD, -1, 0), pv(0, 2, USD, 1, 0)])
            .unwrap();
        // A backdated correction: recorded later, valid earlier.
        let e1 = o
            .submit("b", vec![pv(1, 1, USD, -10, 5), pv(1, 2, USD, 10, 5)])
            .unwrap();
        // As of system epoch 0 we did not yet believe the backdated posting:
        assert_eq!(o.bitemporal(Acct(1), USD, 0, 100), -1);
        // As of e1 we do, even for the old valid time:
        assert_eq!(o.bitemporal(Acct(1), USD, e1, 100), -11);
    }

    #[test]
    fn hold_reduces_available_but_not_ledger_balance() {
        let mut o = Oracle::new();
        o.submit("fund", vec![p(1, 1, USD, 10_000), p(1, 99, USD, -10_000)])
            .unwrap();
        let e = o
            .submit(
                "auth",
                vec![Row::Hold {
                    id: 7,
                    acct: Acct(1),
                    cur: USD,
                    amount: 2_500,
                    valid: 0,
                }],
            )
            .unwrap();

        // The regulator's two balances, over one base.
        assert_eq!(o.ledger_balance(Acct(1), USD, e), 10_000);
        assert_eq!(o.available_balance(Acct(1), USD, e), 7_500);
        assert!(o.conservation_ok(e));
    }

    #[test]
    fn partial_capture_releases_the_remainder() {
        let mut o = Oracle::new();
        o.submit("fund", vec![p(1, 1, USD, 10_000), p(1, 99, USD, -10_000)])
            .unwrap();
        o.submit(
            "auth",
            vec![Row::Hold {
                id: 7,
                acct: Acct(1),
                cur: USD,
                amount: 2_500,
                valid: 0,
            }],
        )
        .unwrap();

        // Capture less than was held: the settled posting is the captured amount, and the
        // hold is resolved, so the remainder becomes available again.
        let e = o
            .submit(
                "capture",
                vec![
                    Row::Resolve {
                        hold: 7,
                        outcome: Outcome::Post(1_800),
                        valid: 0,
                    },
                    p(2, 1, USD, -1_800),
                    p(2, 50, USD, 1_800),
                ],
            )
            .unwrap();

        assert_eq!(o.ledger_balance(Acct(1), USD, e), 8_200);
        assert_eq!(o.unresolved_holds(Acct(1), USD, e), 0);
        assert_eq!(o.available_balance(Acct(1), USD, e), 8_200);
        assert!(o.conservation_ok(e));
    }

    #[test]
    fn voided_hold_restores_availability_without_touching_the_ledger_balance() {
        let mut o = Oracle::new();
        o.submit("fund", vec![p(1, 1, USD, 500), p(1, 99, USD, -500)])
            .unwrap();
        o.submit(
            "auth",
            vec![Row::Hold {
                id: 3,
                acct: Acct(1),
                cur: USD,
                amount: 200,
                valid: 0,
            }],
        )
        .unwrap();
        let e = o
            .submit(
                "void",
                vec![Row::Resolve {
                    hold: 3,
                    outcome: Outcome::Void,
                    valid: 0,
                }],
            )
            .unwrap();

        assert_eq!(o.ledger_balance(Acct(1), USD, e), 500);
        assert_eq!(o.available_balance(Acct(1), USD, e), 500);
    }

    #[test]
    fn expired_hold_is_resolved_like_any_other() {
        let mut o = Oracle::new();
        o.submit("fund", vec![p(1, 1, USD, 500), p(1, 99, USD, -500)])
            .unwrap();
        o.submit(
            "auth",
            vec![Row::Hold {
                id: 4,
                acct: Acct(1),
                cur: USD,
                amount: 120,
                valid: 0,
            }],
        )
        .unwrap();
        let e = o
            .submit(
                "exp",
                vec![Row::Resolve {
                    hold: 4,
                    outcome: Outcome::Expire,
                    valid: 0,
                }],
            )
            .unwrap();
        assert_eq!(o.available_balance(Acct(1), USD, e), 500);
    }

    #[test]
    fn double_resolution_is_rejected() {
        let mut o = Oracle::new();
        o.submit(
            "auth",
            vec![Row::Hold {
                id: 9,
                acct: Acct(1),
                cur: USD,
                amount: 10,
                valid: 0,
            }],
        )
        .unwrap();
        o.submit(
            "r1",
            vec![Row::Resolve {
                hold: 9,
                outcome: Outcome::Void,
                valid: 0,
            }],
        )
        .unwrap();
        assert_eq!(
            o.submit(
                "r2",
                vec![Row::Resolve {
                    hold: 9,
                    outcome: Outcome::Void,
                    valid: 0
                }]
            ),
            Err(Reject::BadResolution)
        );
    }

    #[test]
    fn resolution_of_unknown_hold_is_rejected() {
        let mut o = Oracle::new();
        assert_eq!(
            o.submit(
                "r",
                vec![Row::Resolve {
                    hold: 404,
                    outcome: Outcome::Void,
                    valid: 0
                }]
            ),
            Err(Reject::BadResolution)
        );
    }

    #[test]
    fn available_balance_identity_holds_at_every_anchor() {
        let mut o = Oracle::new();
        o.submit("fund", vec![p(1, 1, USD, 1_000), p(1, 99, USD, -1_000)])
            .unwrap();
        o.submit(
            "h1",
            vec![Row::Hold {
                id: 1,
                acct: Acct(1),
                cur: USD,
                amount: 100,
                valid: 0,
            }],
        )
        .unwrap();
        o.submit(
            "h2",
            vec![Row::Hold {
                id: 2,
                acct: Acct(1),
                cur: USD,
                amount: 250,
                valid: 0,
            }],
        )
        .unwrap();
        o.submit(
            "v",
            vec![Row::Resolve {
                hold: 1,
                outcome: Outcome::Void,
                valid: 0,
            }],
        )
        .unwrap();

        for anchor in 0..o.epochs.len() as u64 {
            let ledger = o.ledger_balance(Acct(1), USD, anchor);
            let held = o.unresolved_holds(Acct(1), USD, anchor);
            assert_eq!(o.available_balance(Acct(1), USD, anchor), ledger - held);
        }
    }

    #[test]
    fn conservation_holds_at_every_epoch_of_a_mixed_history() {
        let mut o = Oracle::new();
        o.submit("a", vec![p(1, 1, USD, 100), p(1, 99, USD, -100)])
            .unwrap();
        o.submit(
            "b",
            vec![Row::Hold {
                id: 1,
                acct: Acct(1),
                cur: USD,
                amount: 40,
                valid: 0,
            }],
        )
        .unwrap();
        o.submit(
            "c",
            vec![
                Row::Resolve {
                    hold: 1,
                    outcome: Outcome::Post(40),
                    valid: 0,
                },
                p(2, 1, USD, -40),
                p(2, 2, USD, 40),
            ],
        )
        .unwrap();
        o.submit("d", vec![p(3, 2, USD, -15), p(3, 3, USD, 15)])
            .unwrap();

        for anchor in 0..o.epochs.len() as u64 {
            assert!(
                o.conservation_ok(anchor),
                "conservation failed at epoch {anchor}"
            );
        }
    }

    #[test]
    fn tamper_is_self_announcing() {
        let mut o = Oracle::new();
        o.submit("a", vec![p(0, 1, USD, -1), p(0, 2, USD, 1)])
            .unwrap();
        o.submit("b", vec![p(1, 1, USD, -2), p(1, 2, USD, 2)])
            .unwrap();
        assert!(o.verify_chain());

        // The forbidden eraser.
        if let Row::Post(ref mut post) = o.epochs[0].rows[0] {
            post.amt = -1_000_000;
        }
        assert!(!o.verify_chain());
    }

    #[test]
    fn prefix_reads_are_stable_as_history_grows() {
        // The property that makes anchored reconstruction meaningful: an answer at an
        // anchor never changes, however much history is appended afterwards.
        let mut o = Oracle::new();
        let e0 = o
            .submit("a", vec![p(1, 1, USD, 100), p(1, 99, USD, -100)])
            .unwrap();
        let before = o.ledger_balance(Acct(1), USD, e0);
        for i in 0..25u64 {
            o.submit(
                &format!("x{i}"),
                vec![p(10 + i, 1, USD, 5), p(10 + i, 99, USD, -5)],
            )
            .unwrap();
        }
        assert_eq!(o.ledger_balance(Acct(1), USD, e0), before);
    }
}
