//! `proto-engine` against the reference oracle.
//!
//! This is the crate that produced every counted-work figure in the evaluation chapter,
//! and until now the only value check on it compared `reconstruct_balance` against
//! `reconstruct_balance`. The expected value here comes from a fold in
//! `conservation-suite`, which this crate cannot reach and does not share code with.
//!
//! Anchors are drawn from the whole retained history, because Theorem 4.1 quantifies over
//! every anchor and a suite that reads only at the head tests it at one.

use conservation_suite::faults::{inject, Fault};
use conservation_suite::oracle::{Acct as OAcct, Cur as OCur};
use conservation_suite::properties::{differential, schedule, Answer, Observable, Op, Unavailable};
use proto_engine::ledger::{Ledger, Posting, Row};
use proto_engine::policy::EvictionPolicy;
use proto_engine::view::{PartialView, ViewMode};
use proto_engine::{Acct, Cur, Minor};

const USD: Cur = 840;
const ISSUANCE: Acct = 9_999;

/// The system under test: a partial view over the prototype's ledger, with a budget that
/// binds so that eviction and reconstruction run continuously rather than incidentally.
struct Sut {
    ledger: Ledger,
    view: PartialView,
    txn: u64,
}

impl Sut {
    fn new(checkpoints: usize) -> Self {
        Sut {
            ledger: if checkpoints == 0 {
                Ledger::new()
            } else {
                Ledger::with_checkpoints(checkpoints)
            },
            // Three slots against sixty-four keys: almost every read reconstructs.
            view: PartialView::new(ViewMode::Demand, 3, EvictionPolicy::Lru),
            txn: 0,
        }
    }

    fn drive(&mut self, op: &Op) {
        match op {
            Op::Open {
                accounts,
                cur: _,
                each,
            } => {
                let mut rows = Vec::new();
                for a in 0..*accounts {
                    rows.push(Row::Post(Posting {
                        txn: 0,
                        acct: a,
                        cur: USD,
                        amt: *each as Minor,
                        valid: 0,
                    }));
                }
                rows.push(Row::Post(Posting {
                    txn: 0,
                    acct: ISSUANCE,
                    cur: USD,
                    amt: -(*each as Minor) * *accounts as Minor,
                    valid: 0,
                }));
                let _ = self.ledger.submit("open", rows);
                self.view.apply_through(&self.ledger, self.ledger.head());
            }
            Op::Submit {
                key,
                from,
                to,
                cur: _,
                amt,
            } => {
                self.txn += 1;
                let t = self.txn;
                let ok = self
                    .ledger
                    .submit(
                        key,
                        vec![
                            Row::Post(Posting {
                                txn: t,
                                acct: from.0,
                                cur: USD,
                                amt: -(*amt as Minor),
                                valid: 0,
                            }),
                            Row::Post(Posting {
                                txn: t,
                                acct: to.0,
                                cur: USD,
                                amt: *amt as Minor,
                                valid: 0,
                            }),
                        ],
                    )
                    .is_ok();
                if ok {
                    self.view.apply_through(&self.ledger, self.ledger.head());
                }
            }
            Op::Duplicate { key } => {
                // Must be refused, not double-posted.
                let r = self.ledger.submit(key, vec![]);
                assert!(r.is_err(), "a replayed key must be refused");
            }
            // Eviction and crash are the same operation one level apart: recovery is a
            // large eviction, which is Theorem 4.1's practical content.
            Op::Evict | Op::Crash => self.view.wipe(),
            Op::Read { .. } => {}
        }
    }
}

impl Observable for Sut {
    fn balance_at(&mut self, acct: OAcct, cur: OCur, anchor: u64) -> Result<Answer, Unavailable> {
        if cur.0 != USD {
            return Err(Unavailable::Refused("only USD in this fixture".into()));
        }
        if anchor > self.ledger.head() {
            return Err(Unavailable::BeyondFrontier);
        }
        let (value, served_at, _hit) =
            self.view
                .read(&mut self.ledger, acct.0, USD, anchor, 0.0, 0.0);
        Ok(Answer {
            value,
            anchor: served_at,
        })
    }
}

fn run(seed: u64, checkpoints: usize, len: usize) -> conservation_suite::properties::Report {
    let s = schedule(seed, 8, len);
    let mut sut = Sut::new(checkpoints);
    differential(&mut sut, &s, |sut, op| sut.drive(op))
}

#[test]
fn the_partial_view_agrees_with_the_oracle_at_randomly_sampled_anchors() {
    let mut total = 0u64;
    for seed in [1u64, 7, 42, 100, 2024] {
        let r = run(seed, 0, 900);
        assert!(
            r.ok(),
            "seed {seed}: {} divergences, first {:?}",
            r.divergences.len(),
            r.divergences.first()
        );
        total += r.compared;
    }
    assert!(
        total >= 1000,
        "the suite must actually compare: {total} comparisons"
    );
}

#[test]
fn checkpointed_reconstruction_agrees_with_the_oracle_too() {
    // SC7's mechanism, checked for *value* rather than only for cost. Until now no
    // experiment ran the checkpointed path against an independent fold at all.
    for seed in [1u64, 42, 2024] {
        for c in [1usize, 2, 16] {
            let r = run(seed, c, 600);
            assert!(
                r.ok(),
                "seed {seed}, C={c}: {} divergences, first {:?}",
                r.divergences.len(),
                r.divergences.first()
            );
        }
    }
}

#[test]
fn the_answers_survive_a_crash_campaign() {
    // Recovery is a large eviction, so the campaign is the strongest form of the
    // reconstruction-equivalence claim the prototype can carry.
    for seed in [1u64, 7, 2024] {
        let base = schedule(seed, 8, 500);
        let campaign = inject(&base, Fault::Crash, 9);
        let mut sut = Sut::new(16);
        let r = differential(&mut sut, &campaign, |sut, op| sut.drive(op));
        assert!(r.compared > 100);
        assert!(r.ok(), "seed {seed}: {:?}", r.divergences.first());
    }
}

#[test]
fn the_answers_survive_an_eviction_storm() {
    for seed in [7u64, 100] {
        let base = schedule(seed, 8, 400);
        let campaign = inject(&base, Fault::EvictionStorm { times: 6 }, 5);
        let mut sut = Sut::new(0);
        let r = differential(&mut sut, &campaign, |sut, op| sut.drive(op));
        assert!(r.ok(), "seed {seed}: {:?}", r.divergences.first());
    }
}

#[test]
fn the_answers_survive_duplicate_delivery() {
    for seed in [42u64, 2024] {
        let base = schedule(seed, 8, 400);
        let campaign = inject(&base, Fault::DuplicateDelivery, 6);
        let mut sut = Sut::new(0);
        let r = differential(&mut sut, &campaign, |sut, op| sut.drive(op));
        assert!(r.ok(), "seed {seed}: {:?}", r.divergences.first());
    }
}

#[test]
fn the_differential_would_catch_a_wrong_engine() {
    // The negative control for this file. A suite nobody has watched fail proves only
    // that it compiles.
    struct Broken(Sut);
    impl Observable for Broken {
        fn balance_at(
            &mut self,
            acct: OAcct,
            cur: OCur,
            anchor: u64,
        ) -> Result<Answer, Unavailable> {
            // The defect this architecture exists to prevent: an absence answered with
            // the aggregate's identity.
            if anchor > 2 {
                return Ok(Answer { value: 0, anchor });
            }
            self.0.balance_at(acct, cur, anchor)
        }
    }
    let s = schedule(1, 8, 400);
    let mut b = Broken(Sut::new(0));
    let r = differential(&mut b, &s, |b, op| b.0.drive(op));
    assert!(!r.ok(), "the harness must catch a miss answered as zero");
}
