//! **The dataset** (design §4): one small bank, generated deterministically from seed 1 and
//! committed as fixture files, so every surface loads the same rows and the oracle reads them
//! too.
//!
//! The shape is constructed rather than left to chance where a task needs it: accounts 19 and
//! 20 never post (Q04); every usd movement runs from the two treasury accounts 1 and 2 to a
//! customer, so exactly those two usd balances are negative (Q02); and accounts 17 and 18 each
//! receive one usd transfer of the same amount and nothing else in usd, so their balances tie
//! at the top of the ranking (Q05, Q08).

use std::fmt::Write as _;

/// A currency: name, and minor-unit scale.
pub const CURRENCIES: &[(&str, u32)] = &[("usd", 2), ("eur", 2), ("jpy", 0)];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    pub id: i64,
    pub parent: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: i64,
    pub owner: i64,
    pub desk: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub txn: i64,
    pub acct: i64,
    pub cur: &'static str,
    /// Signed minor units.
    pub amt: i64,
    pub epoch: i64,
    /// Days since the dataset's day 0.
    pub value_date: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hold {
    pub id: i64,
    pub acct: i64,
    pub cur: &'static str,
    pub amount: i64,
    pub open: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dataset {
    pub parties: Vec<Party>,
    pub accounts: Vec<Account>,
    pub postings: Vec<Posting>,
    pub holds: Vec<Hold>,
}

/// A small, fixed PRNG (xorshift64*), so the dataset is a function of the seed and this file.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % (hi - lo + 1) as u64) as i64
    }
}

pub fn generate(seed: u64) -> Dataset {
    let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    // Parties: a chain 4 → 3 → 2 → 1, a small tree under 5, a pair under 10, and roots.
    let parents = [
        None,
        Some(1),
        Some(2),
        Some(3),
        None,
        Some(5),
        Some(5),
        Some(6),
        None,
        None,
        Some(10),
        None,
    ];
    let parties = parents
        .iter()
        .enumerate()
        .map(|(i, p)| Party {
            id: i as i64 + 1,
            parent: *p,
        })
        .collect();
    let accounts = (1..=20)
        .map(|i| Account {
            id: i,
            owner: (i - 1) % 12 + 1,
            desk: (i - 1) % 4 + 1,
        })
        .collect();

    // Transactions, before epochs are assigned: (currency, from, to, amount).
    let mut txns: Vec<(&'static str, i64, i64, i64)> = Vec::new();
    for k in 0..58 {
        let from = 1 + (k % 2);
        let to = r.range(3, 16);
        txns.push(("usd", from, to, r.range(100, 99_999)));
    }
    txns.push(("usd", 1, 17, 2_000_000));
    txns.push(("usd", 2, 18, 2_000_000));
    for _ in 0..30 {
        let a = r.range(1, 18);
        let mut b = r.range(1, 18);
        while b == a {
            b = r.range(1, 18);
        }
        txns.push(("eur", a, b, r.range(100, 99_999)));
    }
    for _ in 0..10 {
        let a = r.range(3, 16);
        let mut b = r.range(3, 16);
        while b == a {
            b = r.range(3, 16);
        }
        txns.push(("jpy", a, b, r.range(10, 9_999)));
    }
    // A deterministic shuffle, so currencies interleave across epochs.
    for i in (1..txns.len()).rev() {
        let j = (r.next() % (i as u64 + 1)) as usize;
        txns.swap(i, j);
    }
    let mut postings = Vec::new();
    for (i, (cur, from, to, amt)) in txns.into_iter().enumerate() {
        let txn = i as i64 + 1;
        let epoch = 1 + i as i64 / 10;
        let mut vd = 4 * epoch - r.range(0, 3);
        // 3 of the 100 are back-valued, by 10 to 20 days.
        if i % 33 == 7 {
            vd = (vd - r.range(10, 20)).max(1);
        }
        postings.push(Posting {
            txn,
            acct: from,
            cur,
            amt: -amt,
            epoch,
            value_date: vd,
        });
        postings.push(Posting {
            txn,
            acct: to,
            cur,
            amt,
            epoch,
            value_date: vd,
        });
    }
    let holds = (1..=6)
        .map(|id| Hold {
            id,
            acct: r.range(3, 16),
            cur: if id % 2 == 0 { "eur" } else { "usd" },
            amount: r.range(100, 9_999),
            open: id <= 3,
        })
        .collect();
    Dataset {
        parties,
        accounts,
        postings,
        holds,
    }
}

impl Dataset {
    /// The fixture files, as (name, tab-separated text). Headers name the columns; a null is
    /// written `\N`, as PostgreSQL's `copy` reads it.
    pub fn files(&self) -> Vec<(&'static str, String)> {
        let mut parties = String::from("id\tparent\n");
        for p in &self.parties {
            let _ = writeln!(
                parties,
                "{}\t{}",
                p.id,
                p.parent
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "\\N".into())
            );
        }
        let mut accounts = String::from("id\towner\tdesk\n");
        for a in &self.accounts {
            let _ = writeln!(accounts, "{}\t{}\t{}", a.id, a.owner, a.desk);
        }
        let mut postings = String::from("txn\tacct\tcur\tamt\tepoch\tvalue_date\n");
        for p in &self.postings {
            let _ = writeln!(
                postings,
                "{}\t{}\t{}\t{}\t{}\t{}",
                p.txn, p.acct, p.cur, p.amt, p.epoch, p.value_date
            );
        }
        let mut holds = String::from("id\tacct\tcur\tamount\topen\n");
        for h in &self.holds {
            let _ = writeln!(
                holds,
                "{}\t{}\t{}\t{}\t{}",
                h.id, h.acct, h.cur, h.amount, h.open
            );
        }
        vec![
            ("parties.tsv", parties),
            ("accounts.tsv", accounts),
            ("postings.tsv", postings),
            ("holds.tsv", holds),
        ]
    }
}

/// The study's constants: the anchors, dates and arguments every surface's program uses.
pub mod k {
    /// B01, B04, B05: the system-time anchor.
    pub const EPOCH: i64 = 5;
    /// B02, B04: the valid-time instant (a day).
    pub const DAY: i64 = 20;
    /// B05: the statement's account and value-date range, and V03's month start.
    pub const STMT_ACCT: i64 = 3;
    pub const STMT_FROM: i64 = 10;
    pub const STMT_TO: i64 = 30;
    pub const MONTH_START: i64 = 31;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn usd_balances(d: &Dataset) -> BTreeMap<i64, i64> {
        let mut m = BTreeMap::new();
        for p in d.postings.iter().filter(|p| p.cur == "usd") {
            *m.entry(p.acct).or_insert(0) += p.amt;
        }
        m
    }

    #[test]
    fn the_shape_the_tasks_need_is_there() {
        let d = generate(1);
        assert_eq!(d.postings.len(), 200);
        // Q04: exactly accounts 19 and 20 never post.
        let silent: Vec<i64> = d
            .accounts
            .iter()
            .filter(|a| !d.postings.iter().any(|p| p.acct == a.id))
            .map(|a| a.id)
            .collect();
        assert_eq!(silent, vec![19, 20]);
        // Q02: exactly the two treasury accounts are negative in usd.
        let neg: Vec<i64> = usd_balances(&d)
            .into_iter()
            .filter(|(_, b)| *b < 0)
            .map(|(a, _)| a)
            .collect();
        assert_eq!(neg, vec![1, 2]);
        // Q05: 17 and 18 tie at the top.
        let b = usd_balances(&d);
        assert_eq!(b[&17], b[&18]);
        assert!(b.values().all(|v| *v <= b[&17]));
        // Every transaction conserves per currency.
        let mut t: BTreeMap<(i64, &str), i64> = BTreeMap::new();
        for p in &d.postings {
            *t.entry((p.txn, p.cur)).or_insert(0) += p.amt;
        }
        assert!(t.values().all(|v| *v == 0));
        // Three back-valued transactions, and three open holds.
        assert_eq!(d.holds.iter().filter(|h| h.open).count(), 3);
    }

    #[test]
    fn the_dataset_is_a_function_of_the_seed() {
        assert_eq!(generate(1), generate(1));
        assert_ne!(generate(1), generate(2));
    }
}
