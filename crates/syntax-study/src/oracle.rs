//! **The oracle** (design §5): one function per task over the dataset, sharing no code with any
//! checker or executor. Each doc comment is the task's specification, precise where the
//! design's one-line description was not; a program is correct when its answer equals the
//! oracle's.
//!
//! Signs: a leg's amount is signed minor units, and "a pays b m" is `a: −m, b: +m`.

use crate::answer::Answer;
use crate::data::{k, Dataset, Posting};
use std::collections::{BTreeMap, BTreeSet};

/// The arguments every surface's transaction program is called with.
pub mod args {
    /// T01–T05: payer and payee.
    pub const A: i64 = 3;
    pub const B: i64 = 4;
    /// T02: the fee account, the amount and the fee (usd minor units).
    pub const FEE_ACCT: i64 = 5;
    pub const AMOUNT: i64 = 12_345;
    pub const FEE: i64 = 345;
    /// T03: A pays B 100.00 usd; B pays A 90.00 eur.
    pub const FX_USD: i64 = 10_000;
    pub const FX_EUR: i64 = 9_000;
    /// T04/T05: a hold of 50.00 usd on A; T04 captures 30.00 of it to B.
    pub const HOLD: i64 = 5_000;
    pub const CAPTURE: i64 = 3_000;
    /// T06: borrower, three lenders and their shares of 9,000.00 usd (1/2, 1/3, 1/6).
    pub const BORROWER: i64 = 3;
    pub const LENDERS: [(i64, i64); 3] = [(5, 450_000), (6, 300_000), (7, 150_000)];
    /// T07: 12.34 usd accrued: the receivable +, the income account −.
    pub const RECEIVABLE: i64 = 8;
    pub const INCOME: i64 = 9;
    pub const ACCRUAL: i64 = 1_234;
    /// T08: the loan account, the payer and the unapplied account; the payment is the loan
    /// account's usd balance plus 50.00, so 50.00 is excess.
    pub const LOAN: i64 = 10;
    pub const PAYER: i64 = 11;
    pub const UNAPPLIED: i64 = 12;
    pub const EXCESS: i64 = 5_000;
    /// T09: an overdraft limit of 1,000.00 usd granted to A.
    pub const OVERDRAFT: i64 = 100_000;
    /// T10: the transaction reversed.
    pub const REVERSED_TXN: i64 = 1;
    /// B03: the correction: account 1 pays account 3 10.00 usd, value day 5, recorded at
    /// epoch 11 (after every epoch in the dataset).
    pub const CORR_FROM: i64 = 1;
    pub const CORR_TO: i64 = 3;
    pub const CORR_AMT: i64 = 1_000;
    pub const CORR_DAY: i64 = 5;
    pub const CORR_EPOCH: i64 = 11;
}

fn s<T: ToString>(v: T) -> String {
    v.to_string()
}

fn balances<'a>(ps: impl Iterator<Item = &'a Posting>) -> BTreeMap<(i64, &'static str), i64> {
    let mut m = BTreeMap::new();
    for p in ps {
        *m.entry((p.acct, p.cur)).or_insert(0) += p.amt;
    }
    m
}

fn bal_rows(m: BTreeMap<(i64, &'static str), i64>) -> Vec<Vec<String>> {
    m.into_iter()
        .map(|((a, c), b)| vec![s(a), s(c), s(b)])
        .collect()
}

fn usd_balances(d: &Dataset) -> BTreeMap<i64, i64> {
    let mut m = BTreeMap::new();
    for p in d.postings.iter().filter(|p| p.cur == "usd") {
        *m.entry(p.acct).or_insert(0) += p.amt;
    }
    m
}

fn legs(v: &[(i64, &str, i64)]) -> Answer {
    Answer::set(v.iter().map(|(a, c, x)| vec![s(a), s(c), s(x)]).collect())
}

/// The expected answer of a task, by id.
pub fn expected(task: &str, d: &Dataset) -> Answer {
    use args::*;
    match task {
        // (acct, cur, balance) for every (account, currency) with a posting.
        "Q01" | "V01" => Answer::set(bal_rows(balances(d.postings.iter()))),
        // (acct, balance): usd balances below zero.
        "Q02" => Answer::set(
            usd_balances(d)
                .into_iter()
                .filter(|(_, b)| *b < 0)
                .map(|(a, b)| vec![s(a), s(b)])
                .collect(),
        ),
        // (desk, cur, total): postings summed by the desk of their account.
        "Q03" => {
            let desk: BTreeMap<i64, i64> = d.accounts.iter().map(|a| (a.id, a.desk)).collect();
            let mut m: BTreeMap<(i64, &str), i64> = BTreeMap::new();
            for p in &d.postings {
                *m.entry((desk[&p.acct], p.cur)).or_insert(0) += p.amt;
            }
            Answer::set(
                m.into_iter()
                    .map(|((k, c), v)| vec![s(k), s(c), s(v)])
                    .collect(),
            )
        }
        // (acct): accounts with no posting at all.
        "Q04" => Answer::set(
            d.accounts
                .iter()
                .filter(|a| !d.postings.iter().any(|p| p.acct == a.id))
                .map(|a| vec![s(a.id)])
                .collect(),
        ),
        // (acct, balance): the five largest usd balances, largest first, ties by account id.
        "Q05" => {
            let mut v: Vec<(i64, i64)> = usd_balances(d).into_iter().collect();
            v.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
            Answer::ordered(
                v.into_iter()
                    .take(5)
                    .map(|(a, b)| vec![s(a), s(b)])
                    .collect(),
            )
        }
        // (acct, n): distinct currencies each account with a posting has posted in.
        "Q06" => {
            let mut m: BTreeMap<i64, BTreeSet<&str>> = BTreeMap::new();
            for p in &d.postings {
                m.entry(p.acct).or_default().insert(p.cur);
            }
            Answer::set(m.into_iter().map(|(a, c)| vec![s(a), s(c.len())]).collect())
        }
        // (acct, cur, txn, running): per (account, currency), in (epoch, txn) order, the
        // balance after each posting.
        "Q07" => {
            let mut ps: Vec<&Posting> = d.postings.iter().collect();
            ps.sort_by_key(|p| (p.acct, p.cur, p.epoch, p.txn));
            let mut run: BTreeMap<(i64, &str), i64> = BTreeMap::new();
            let mut rows = Vec::new();
            for p in ps {
                let r = run.entry((p.acct, p.cur)).or_insert(0);
                *r += p.amt;
                rows.push(vec![s(p.acct), s(p.cur), s(p.txn), s(*r)]);
            }
            Answer::set(rows)
        }
        // (acct, desk, rank): usd balance ranked within the desk, largest first; equal
        // balances share a rank and the next rank skips (SQL's `rank()`).
        "Q08" => {
            let desk: BTreeMap<i64, i64> = d.accounts.iter().map(|a| (a.id, a.desk)).collect();
            let b = usd_balances(d);
            let mut rows = Vec::new();
            for (a, bal) in &b {
                let r = 1 + b
                    .iter()
                    .filter(|(x, v)| desk[*x] == desk[a] && *v > bal)
                    .count();
                rows.push(vec![s(a), s(desk[a]), s(r)]);
            }
            Answer::set(rows)
        }
        // (party, ancestor): every ancestor of every party, through any number of parents.
        "Q09" => {
            let parent: BTreeMap<i64, Option<i64>> =
                d.parties.iter().map(|p| (p.id, p.parent)).collect();
            let mut rows = Vec::new();
            for p in &d.parties {
                let mut cur = p.parent;
                while let Some(a) = cur {
                    rows.push(vec![s(p.id), s(a)]);
                    cur = parent[&a];
                }
            }
            Answer::set(rows)
        }
        // (acct, cur, available): posted balance less open holds, over every (account,
        // currency) with a posting or an open hold.
        "Q10" | "V02" => {
            let mut m = balances(d.postings.iter());
            for h in d.holds.iter().filter(|h| h.open) {
                *m.entry((h.acct, h.cur)).or_insert(0) -= h.amount;
            }
            Answer::set(bal_rows(m))
        }
        "T01" => legs(&[(A, "usd", -AMOUNT), (B, "usd", AMOUNT)]),
        "T02" => legs(&[
            (A, "usd", -AMOUNT),
            (B, "usd", AMOUNT - FEE),
            (FEE_ACCT, "usd", FEE),
        ]),
        "T03" => legs(&[
            (A, "usd", -FX_USD),
            (B, "usd", FX_USD),
            (B, "eur", -FX_EUR),
            (A, "eur", FX_EUR),
        ]),
        "T04" => legs(&[(A, "usd", -CAPTURE), (B, "usd", CAPTURE)]),
        // Voiding a hold posts nothing.
        "T05" => legs(&[]),
        "T06" => {
            let total: i64 = LENDERS.iter().map(|(_, x)| x).sum();
            let mut v = vec![(BORROWER, "usd", total)];
            v.extend(LENDERS.iter().map(|(a, x)| (*a, "usd", -x)));
            legs(&v)
        }
        "T07" => legs(&[(RECEIVABLE, "usd", ACCRUAL), (INCOME, "usd", -ACCRUAL)]),
        "T08" => {
            let owed = usd_balances(d).get(&LOAN).copied().unwrap_or(0);
            legs(&[
                (PAYER, "usd", -(owed + EXCESS)),
                (LOAN, "usd", owed),
                (UNAPPLIED, "usd", EXCESS),
            ])
        }
        // The grant itself: (acct, cur, limit). It posts nothing.
        "T09" => Answer::set(vec![vec![s(A), s("usd"), s(OVERDRAFT)]]),
        // The legs of the reversed transaction, negated.
        "T10" => legs(
            &d.postings
                .iter()
                .filter(|p| p.txn == REVERSED_TXN)
                .map(|p| (p.acct, p.cur, -p.amt))
                .collect::<Vec<_>>(),
        ),
        // (acct, cur, total) over value days from the month's start.
        "V03" => Answer::set(bal_rows(balances(
            d.postings.iter().filter(|p| p.value_date >= k::MONTH_START),
        ))),
        // (acct, n): postings per account.
        "V04" => {
            let mut m: BTreeMap<i64, i64> = BTreeMap::new();
            for p in &d.postings {
                *m.entry(p.acct).or_insert(0) += 1;
            }
            Answer::set(m.into_iter().map(|(a, n)| vec![s(a), s(n)]).collect())
        }
        // (cur, total): the trial balance, zero in every currency by conservation.
        "V05" => {
            let mut m: BTreeMap<&str, i64> = BTreeMap::new();
            for p in &d.postings {
                *m.entry(p.cur).or_insert(0) += p.amt;
            }
            Answer::set(m.into_iter().map(|(c, v)| vec![s(c), s(v)]).collect())
        }
        "B01" => Answer::set(bal_rows(balances(
            d.postings.iter().filter(|p| p.epoch <= k::EPOCH),
        ))),
        "B02" => Answer::set(bal_rows(balances(
            d.postings.iter().filter(|p| p.value_date <= k::DAY),
        ))),
        // After the correction: ("valid", acct, cur, balance) at value day DAY, which
        // includes it, and ("system", acct, cur, balance) as of EPOCH, which does not.
        "B03" => {
            let mut all = d.postings.clone();
            for (a, x) in [(CORR_FROM, -CORR_AMT), (CORR_TO, CORR_AMT)] {
                all.push(Posting {
                    txn: 101,
                    acct: a,
                    cur: "usd",
                    amt: x,
                    epoch: CORR_EPOCH,
                    value_date: CORR_DAY,
                });
            }
            let mut rows = Vec::new();
            for r in bal_rows(balances(all.iter().filter(|p| p.value_date <= k::DAY))) {
                let mut v = vec![s("valid")];
                v.extend(r);
                rows.push(v);
            }
            for r in bal_rows(balances(all.iter().filter(|p| p.epoch <= k::EPOCH))) {
                let mut v = vec![s("system")];
                v.extend(r);
                rows.push(v);
            }
            Answer::set(rows)
        }
        "B04" => Answer::set(bal_rows(balances(
            d.postings
                .iter()
                .filter(|p| p.value_date <= k::DAY && p.epoch <= k::EPOCH),
        ))),
        // (txn, value_date, cur, amt): one account's legs over a value-day range, as of EPOCH.
        "B05" => Answer::set(
            d.postings
                .iter()
                .filter(|p| {
                    p.acct == k::STMT_ACCT
                        && p.value_date >= k::STMT_FROM
                        && p.value_date <= k::STMT_TO
                        && p.epoch <= k::EPOCH
                })
                .map(|p| {
                    vec![
                        s(p.txn),
                        crate::kinds::day_iso(p.value_date),
                        s(p.cur),
                        s(p.amt),
                    ]
                })
                .collect(),
        ),
        other => panic!("no task {other}"),
    }
}

/// The thirty tasks, in the design's order.
pub const TASKS: &[&str] = &[
    "Q01", "Q02", "Q03", "Q04", "Q05", "Q06", "Q07", "Q08", "Q09", "Q10", "T01", "T02", "T03",
    "T04", "T05", "T06", "T07", "T08", "T09", "T10", "V01", "V02", "V03", "V04", "V05", "B01",
    "B02", "B03", "B04", "B05",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_task_has_an_answer_and_the_constructed_ones_are_not_empty() {
        let d = crate::data::generate(1);
        for t in TASKS {
            let a = expected(t, &d);
            if *t != "T05" {
                assert!(!a.rows.is_empty(), "{t} is empty");
            }
        }
        // The trial balance is zero in every currency.
        assert!(expected("V05", &d).rows.iter().all(|r| r[1] == "0"));
        // Q05 is the tie, first.
        let q5 = expected("Q05", &d);
        assert_eq!(q5.rows[0][0], "17");
        assert_eq!(q5.rows[1][0], "18");
        assert_eq!(q5.rows[0][1], q5.rows[1][1]);
    }
}
