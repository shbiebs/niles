//! Differential testing against the reference oracle, and the algebra's property tests.
//!
//! Thesis §3.11 states that the oracle *defines* correctness and that "all testing in
//! Chapter 9 is differential testing against 𝒪". That was not true: the oracle compiled,
//! passed its own unit tests, and was called by nothing. This module is the harness that
//! makes the sentence true, and the engines' test suites are its callers.
//!
//! The rule the harness exists to enforce is that a system under test is never compared
//! against *itself*. E1's divergence column compared `reconstruct_balance` with
//! `reconstruct_balance`, which is an identity dressed as a check; here the expected
//! value always comes from a fold this crate owns and the engine cannot reach.

use crate::oracle::{Acct, Cur, Minor, Oracle, Posting, Row};

/// Why a system could not answer. An honest refusal is not a divergence; a wrong answer
/// is. Keeping them apart is the whole point of the absence lattice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unavailable {
    /// The anchor is beyond what this system has.
    BeyondFrontier,
    /// The system declined for a reason it names.
    Refused(String),
}

/// An answer and the epoch it is true at. Every read returns one: an answer without its
/// anchor is not checkable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Answer {
    pub value: Minor,
    pub anchor: u64,
}

/// What the harness needs from a system under test: one anchored read.
///
/// Deliberately the narrowest possible surface. A system that can answer "what is the
/// balance of this key at this epoch" can be checked against the oracle, whatever it is
/// underneath — the in-memory research prototype, the REV runtime, or the banking
/// kernel's journal through the adapter.
///
/// **The returned anchor is not decoration.** A read means "at least as fresh as
/// `anchor`", so an engine may legitimately answer from a fresher entry and say so. The
/// property being checked is the certification invariant — the value is exact at the
/// anchor it is *served with* — and comparing against the oracle at the *requested*
/// anchor instead tests something no engine promises. The first version of this harness
/// made that mistake and reported two thousand units of phantom divergence.
pub trait Observable {
    fn balance_at(&mut self, acct: Acct, cur: Cur, anchor: u64) -> Result<Answer, Unavailable>;
}

/// One step of the transition system of thesis §3.11.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Open the book: fund every account from an issuance account, as one epoch.
    ///
    /// An explicit step rather than something the harness does behind the system's back,
    /// because a funding the two sides apply at different moments is a divergence the
    /// harness invented.
    Open {
        accounts: u64,
        cur: Cur,
        each: Minor,
    },
    /// A balanced transfer of `amt` from `from` to `to`, under an idempotency key.
    Submit {
        key: String,
        from: Acct,
        to: Acct,
        cur: Cur,
        amt: Minor,
    },
    /// Replay a key already submitted. Must be refused, never double-posted.
    Duplicate { key: String },
    /// Read a key at an anchor, and compare.
    Read { acct: Acct, cur: Cur, anchor: u64 },
    /// Ask the system under test to drop derived state.
    Evict,
    /// Crash and recover: the derived layer is rebuilt from the retained base alone.
    Crash,
}

/// A seeded schedule. The seed is a reproducibility guarantee, so the generator is a
/// stated linear congruential generator rather than a library default.
#[derive(Debug, Clone)]
pub struct Schedule {
    pub seed: u64,
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergence {
    pub acct: Acct,
    pub cur: Cur,
    pub anchor: u64,
    pub oracle: Minor,
    pub system: Result<Answer, Unavailable>,
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub compared: u64,
    pub divergences: Vec<Divergence>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.divergences.is_empty()
    }
}

/// The opening epoch: every account funded from an issuance account, so the book is
/// closed and transfers cannot drive a balance negative by accident.
pub fn opening_rows(accounts: u64, cur: Cur, each: Minor) -> Vec<Row> {
    let mut rows = Vec::new();
    for a in 0..accounts {
        rows.push(Row::Post(Posting {
            txn: 0,
            acct: Acct(a),
            cur,
            amt: each,
            valid: 0,
        }));
    }
    rows.push(Row::Post(Posting {
        txn: 0,
        acct: Acct(9_999),
        cur,
        amt: -each * accounts as Minor,
        valid: 0,
    }));
    rows
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// Build a schedule that exercises the transitions the thesis names: commit, read at an
/// arbitrary anchor, evict, replay, crash.
///
/// Anchors are drawn uniformly from the whole retained history rather than taken at the
/// head. Reading only at the head tests reconstruction-equivalence at one anchor, and
/// Theorem 4.1 quantifies over every anchor.
pub fn schedule(seed: u64, accounts: u64, len: usize) -> Schedule {
    let mut r = Lcg(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1));
    let mut ops = Vec::with_capacity(len + 1);
    ops.push(Op::Open {
        accounts,
        cur: Cur(840),
        each: 1_000_000,
    });
    let mut epochs: u64 = 1;
    let mut keys: Vec<String> = Vec::new();
    for i in 0..len {
        let pick = r.next() % 100;
        if pick < 45 {
            let from = r.next() % accounts;
            let mut to = r.next() % accounts;
            if to == from {
                to = (to + 1) % accounts;
            }
            let key = format!("t-{seed}-{i}");
            keys.push(key.clone());
            ops.push(Op::Submit {
                key,
                from: Acct(from),
                to: Acct(to),
                cur: Cur(840),
                amt: 1 + (r.next() % 500) as Minor,
            });
            epochs += 1;
        } else if pick < 85 {
            ops.push(Op::Read {
                acct: Acct(r.next() % accounts),
                cur: Cur(840),
                anchor: if epochs == 0 { 0 } else { r.next() % epochs },
            });
        } else if pick < 92 {
            ops.push(Op::Evict);
        } else if pick < 97 {
            if let Some(k) = keys.get((r.next() as usize) % keys.len().max(1)) {
                ops.push(Op::Duplicate { key: k.clone() });
            }
        } else {
            ops.push(Op::Crash);
        }
    }
    Schedule { seed, ops }
}

/// Run a schedule against the oracle and a system under test, comparing every read.
///
/// `evict` and `crash` are the system's own hooks: the harness cannot know how a given
/// engine drops derived state, only that dropping it must not change an answer.
pub fn differential<S: Observable>(
    sut: &mut S,
    schedule: &Schedule,
    mut apply: impl FnMut(&mut S, &Op),
) -> Report {
    let mut oracle = Oracle::new();
    let mut report = Report::default();

    for op in &schedule.ops {
        // The oracle side: submit and duplicate change it; read compares; evict and
        // crash are meaningless to a system that materializes nothing, which is exactly
        // why it is the oracle.
        match op {
            Op::Open {
                accounts,
                cur,
                each,
            } => {
                let _ = oracle.submit("open", opening_rows(*accounts, *cur, *each));
            }
            Op::Submit {
                key,
                from,
                to,
                cur,
                amt,
            } => {
                let _ = oracle.submit(
                    key,
                    vec![
                        Row::Post(Posting {
                            txn: 1,
                            acct: *from,
                            cur: *cur,
                            amt: -amt,
                            valid: 0,
                        }),
                        Row::Post(Posting {
                            txn: 1,
                            acct: *to,
                            cur: *cur,
                            amt: *amt,
                            valid: 0,
                        }),
                    ],
                );
            }
            Op::Duplicate { key } => {
                // A replay must be refused by both sides. The oracle's refusal is the
                // specification; the system under test is checked by its own suite.
                let r = oracle.submit(key, vec![]);
                debug_assert!(r.is_err(), "a replayed key must be refused");
            }
            Op::Read { acct, cur, anchor } => {
                let got = sut.balance_at(*acct, *cur, *anchor);
                report.compared += 1;
                // Compared at the anchor the answer *carries*, and separately checked for
                // not being older than the anchor asked for. Those are two different
                // obligations and a single comparison would conflate them.
                let ok = match &got {
                    Ok(a) => {
                        a.anchor >= *anchor
                            && a.value == oracle.ledger_balance(*acct, *cur, a.anchor)
                    }
                    Err(_) => false,
                };
                if !ok {
                    report.divergences.push(Divergence {
                        acct: *acct,
                        cur: *cur,
                        anchor: *anchor,
                        oracle: oracle.ledger_balance(*acct, *cur, *anchor),
                        system: got,
                    });
                }
            }
            Op::Evict | Op::Crash => {}
        }
        apply(sut, op);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A system that folds the oracle's own history. Trivially correct, and here to check
    /// that the harness reports zero divergences when there are none — a harness that
    /// cannot pass is as useless as one that cannot fail.
    struct Mirror {
        inner: Oracle,
    }

    impl Observable for Mirror {
        fn balance_at(&mut self, acct: Acct, cur: Cur, anchor: u64) -> Result<Answer, Unavailable> {
            Ok(Answer {
                value: self.inner.ledger_balance(acct, cur, anchor),
                anchor,
            })
        }
    }

    /// A system that answers zero for a key it has not seen — the `Err(_) => 0` defect
    /// this whole architecture exists to make unrepresentable. The harness must catch it.
    struct MissIsZero {
        inner: Oracle,
        blind_after: u64,
    }

    impl Observable for MissIsZero {
        fn balance_at(&mut self, acct: Acct, cur: Cur, anchor: u64) -> Result<Answer, Unavailable> {
            if anchor > self.blind_after {
                return Ok(Answer { value: 0, anchor });
            }
            Ok(Answer {
                value: self.inner.ledger_balance(acct, cur, anchor),
                anchor,
            })
        }
    }

    fn drive(o: &mut Oracle, op: &Op) {
        if let Op::Open {
            accounts,
            cur,
            each,
        } = op
        {
            let _ = o.submit("open", opening_rows(*accounts, *cur, *each));
        }
        if let Op::Submit {
            key,
            from,
            to,
            cur,
            amt,
        } = op
        {
            let _ = o.submit(
                key,
                vec![
                    Row::Post(Posting {
                        txn: 1,
                        acct: *from,
                        cur: *cur,
                        amt: -amt,
                        valid: 0,
                    }),
                    Row::Post(Posting {
                        txn: 1,
                        acct: *to,
                        cur: *cur,
                        amt: *amt,
                        valid: 0,
                    }),
                ],
            );
        }
    }

    #[test]
    fn the_harness_passes_a_system_that_is_right() {
        let s = schedule(42, 8, 400);
        let mut m = Mirror {
            inner: Oracle::new(),
        };
        let r = differential(&mut m, &s, |m, op| drive(&mut m.inner, op));
        assert!(r.compared > 100, "the schedule must actually read");
        assert!(r.ok(), "{:?}", r.divergences);
    }

    #[test]
    fn the_harness_catches_a_system_that_answers_zero_for_a_miss() {
        // The negative control. A harness nobody has watched fail is not evidence.
        let s = schedule(7, 8, 400);
        let mut m = MissIsZero {
            inner: Oracle::new(),
            blind_after: 3,
        };
        let r = differential(&mut m, &s, |m, op| drive(&mut m.inner, op));
        assert!(
            !r.ok(),
            "a system returning zero for a miss must be reported as divergent"
        );
    }

    #[test]
    fn anchors_are_drawn_from_the_whole_history_not_only_the_head() {
        // Theorem 4.1 quantifies over every anchor. A schedule that reads only at the
        // head tests it at one.
        let s = schedule(1, 8, 600);
        let anchors: Vec<u64> = s
            .ops
            .iter()
            .filter_map(|o| match o {
                Op::Read { anchor, .. } => Some(*anchor),
                _ => None,
            })
            .collect();
        assert!(anchors.len() > 50);
        let distinct: std::collections::BTreeSet<u64> = anchors.iter().copied().collect();
        assert!(
            distinct.len() > 20,
            "reads must span the history: {} distinct anchors",
            distinct.len()
        );
    }

    #[test]
    fn conservation_holds_under_arbitrary_interleavings() {
        // The global property gate: whatever order commits, reads, evictions and replays
        // arrive in, the per-currency system total is zero at every epoch.
        for seed in [1u64, 7, 42, 100, 2024] {
            let s = schedule(seed, 12, 500);
            let mut o = Oracle::new();
            for op in &s.ops {
                drive(&mut o, op);
            }
            for anchor in 0..=s.ops.len() as u64 / 4 {
                assert!(
                    o.conservation_ok(anchor),
                    "seed {seed}: conservation failed at anchor {anchor}"
                );
            }
        }
    }

    #[test]
    fn a_replayed_key_is_refused_rather_than_double_posted() {
        let mut o = Oracle::new();
        let rows = vec![
            Row::Post(Posting {
                txn: 1,
                acct: Acct(1),
                cur: Cur(840),
                amt: -10,
                valid: 0,
            }),
            Row::Post(Posting {
                txn: 1,
                acct: Acct(2),
                cur: Cur(840),
                amt: 10,
                valid: 0,
            }),
        ];
        o.submit("k", rows.clone()).unwrap();
        let before = o.ledger_balance(Acct(2), Cur(840), 0);
        assert!(o.submit("k", rows).is_err());
        assert_eq!(o.ledger_balance(Acct(2), Cur(840), 0), before);
    }
}
