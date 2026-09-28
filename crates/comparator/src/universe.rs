//! The synthetic universe every arm is loaded from: one generator, one seed, one file of
//! transactions, identical on every arm and checked to be so.
//!
//! # Where each parameter comes from
//!
//! The author's answers of 2026-09-28 (`cycle-14-author-answers.md`), in the shapes they were
//! given — ranges and proportions, never data:
//!
//! * **Activity skew α = 0.6 by default** (W5: ~10% of activity in the busiest 1% of
//!   facilities, ~40% in the busiest 10%). Activity is drawn from a Zipf law over a seeded
//!   permutation of the accounts, so which accounts are hot is itself random per seed.
//! * **Multi-currency as a population, not an exception** (W11: more than 30% of facilities).
//!   With `multi_currency`, 35% of accounts also hold a second currency, and a transfer
//!   between two such accounts is in that currency half the time.
//! * **Back-valued postings 1–5%, up to a month** (W10): 3% of transactions carry a value
//!   day up to 22 business days before their booking day.
//! * **A flat intraday profile** (W9): transactions are spread evenly over batches, and a
//!   batch is one sealed epoch on every arm.
//!
//! Everything here is a pure function of [`Params`]; nothing reads a clock or the
//! environment, so the same parameters give the same bytes on every host, and
//! [`Universe::checksum`] proves it per arm.

/// The generator's knobs. Every one is written into E27's header.
#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub accounts: u64,
    pub seed: u64,
    /// Zipf exponent of account activity (and of the read mix, see `Sampler`).
    pub alpha: f64,
    pub multi_currency: bool,
    /// Transactions in the history loaded before any measurement.
    pub history_txns: u64,
    /// Transactions sealed together as one epoch during the load.
    pub batch: u64,
}

impl Params {
    /// The declared first run of §5.4: ten transactions of history per account, batches of
    /// 200 transactions.
    pub fn declared(accounts: u64, seed: u64, alpha: f64, multi_currency: bool) -> Params {
        Params {
            accounts,
            seed,
            alpha,
            multi_currency,
            history_txns: accounts * 10,
            batch: 200,
        }
    }
}

/// One leg: an account, a currency code (0 = usd, 1 = eur), an amount in minor units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Leg {
    pub acct: u64,
    pub cur: u32,
    pub amt: i64,
}

/// A two-leg, conserving transfer.
#[derive(Debug, Clone, PartialEq)]
pub struct Txn {
    /// Unique across the universe, starting at 1.
    pub id: u64,
    /// The batch (and so the epoch offset) it is sealed in, starting at 1.
    pub batch: u64,
    pub legs: [Leg; 2],
    /// The value day, in business days since the start of the history. Back-valued for 3%.
    pub value_day: i32,
}

/// SplitMix64 — small, fast, and fully determined by its seed.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// A Zipf(α) draw over accounts `1..=n`, hot accounts chosen by a seeded permutation.
#[derive(Debug, Clone)]
pub struct Sampler {
    cumulative: Vec<f64>,
    account_of_rank: Vec<u64>,
}

impl Sampler {
    pub fn new(n: u64, alpha: f64, seed: u64) -> Sampler {
        let mut account_of_rank: Vec<u64> = (1..=n).collect();
        let mut r = Rng::new(seed ^ 0x5A5A_5A5A);
        for i in (1..account_of_rank.len()).rev() {
            let j = r.below(i as u64 + 1) as usize;
            account_of_rank.swap(i, j);
        }
        let mut cumulative = Vec::with_capacity(n as usize);
        let mut total = 0.0;
        for rank in 1..=n {
            total += (rank as f64).powf(-alpha);
            cumulative.push(total);
        }
        for c in &mut cumulative {
            *c /= total;
        }
        Sampler {
            cumulative,
            account_of_rank,
        }
    }
    pub fn draw(&self, r: &mut Rng) -> u64 {
        let u = r.unit();
        let i = self.cumulative.partition_point(|&c| c < u);
        self.account_of_rank[i.min(self.account_of_rank.len() - 1)]
    }
    /// The account at activity rank `rank` (1 = the busiest).
    pub fn account_at(&self, rank: usize) -> u64 {
        self.account_of_rank[rank.clamp(1, self.account_of_rank.len()) - 1]
    }
    /// The share of total weight held by the top `k` ranks — reported in the header so a
    /// reader can check the skew against the author's ~10% / ~40%.
    pub fn top_share(&self, k: usize) -> f64 {
        if k == 0 {
            return 0.0;
        }
        self.cumulative[k.min(self.cumulative.len()) - 1]
    }
}

/// Whether an account holds the second currency: a fixed 35% of accounts, by hash.
pub fn holds_eur(acct: u64, multi_currency: bool) -> bool {
    multi_currency && (acct.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) % 100 < 35
}

/// The desk an account books to — for the desk-exposure query (q4).
pub fn desk_of(acct: u64) -> u64 {
    acct % 16
}

#[derive(Debug, Clone)]
pub struct Universe {
    pub params: Params,
    pub txns: Vec<Txn>,
    pub sampler: Sampler,
}

impl Universe {
    pub fn generate(p: &Params) -> Universe {
        let sampler = Sampler::new(p.accounts, p.alpha, p.seed);
        let mut r = Rng::new(p.seed);
        let mut txns = Vec::with_capacity(p.history_txns as usize);
        for i in 0..p.history_txns {
            let id = i + 1;
            txns.push(Self::one(p, &sampler, &mut r, id, i / p.batch + 1));
        }
        Universe {
            params: p.clone(),
            txns,
            sampler,
        }
    }

    /// One transfer, drawn from the universe's distributions. Also used for the measured
    /// write phase, with ids above the history's.
    pub fn one(p: &Params, s: &Sampler, r: &mut Rng, id: u64, batch: u64) -> Txn {
        let from = s.draw(r);
        let mut to = s.draw(r);
        if to == from {
            to = from % p.accounts + 1;
        }
        let cur = if holds_eur(from, p.multi_currency)
            && holds_eur(to, p.multi_currency)
            && r.unit() < 0.5
        {
            1
        } else {
            0
        };
        let amt = 100 + r.below(1_000_000) as i64;
        // Fifty batches to a business day; 3% back-valued by up to 22 business days.
        let day = (batch / 50) as i32;
        let value_day = if r.unit() < 0.03 {
            day - 1 - r.below(22) as i32
        } else {
            day
        };
        Txn {
            id,
            batch,
            legs: [
                Leg {
                    acct: from,
                    cur,
                    amt: -amt,
                },
                Leg { acct: to, cur, amt },
            ],
            value_day,
        }
    }

    pub fn batches(&self) -> u64 {
        self.txns.last().map(|t| t.batch).unwrap_or(0)
    }

    /// The legs every arm must hold, as the canonical multiset `acct,cur,amt` — sorted, one
    /// per line — and its SHA-256. `txn` is deliberately not in it: Nilestream's SQL surface
    /// reads `txn` as the start of a transaction block, so a column the served engine cannot
    /// return would make the check unrunnable on one arm; the multiset with its count is the
    /// strongest statement all arms can be asked for identically.
    pub fn checksum(&self) -> (u64, String) {
        let mut legs: Vec<Leg> = self.txns.iter().flat_map(|t| t.legs).collect();
        checksum_of(&mut legs)
    }
}

/// The checksum of any multiset of legs, in the one canonical form every arm is held to.
pub fn checksum_of(legs: &mut [Leg]) -> (u64, String) {
    legs.sort();
    let mut h = nilestream_ledger::chain::Hasher256::new();
    for l in legs.iter() {
        h.update(format!("{},{},{}\n", l.acct, l.cur, l.amt).as_bytes());
    }
    (
        legs.len() as u64,
        nilestream_ledger::chain::hex(&h.finalize()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_parameters_give_the_same_universe() {
        let p = Params::declared(1000, 42, 0.6, true);
        let a = Universe::generate(&p);
        let b = Universe::generate(&p);
        assert_eq!(a.txns, b.txns);
        assert_eq!(a.checksum(), b.checksum());
        let c = Universe::generate(&Params::declared(1000, 7, 0.6, true));
        assert_ne!(
            a.checksum(),
            c.checksum(),
            "a different seed is a different universe"
        );
    }

    #[test]
    fn every_transaction_conserves_per_currency() {
        let u = Universe::generate(&Params::declared(500, 1, 0.6, true));
        for t in &u.txns {
            assert_eq!(t.legs[0].cur, t.legs[1].cur);
            assert_eq!(t.legs[0].amt + t.legs[1].amt, 0);
            assert_ne!(t.legs[0].acct, t.legs[1].acct);
        }
    }

    #[test]
    fn alpha_point_six_matches_the_authors_shape_at_ten_thousand_accounts() {
        // W5: ~10% of activity in the busiest 1%, ~40% in the busiest 10%.
        let s = Sampler::new(10_000, 0.6, 1);
        let top1 = s.top_share(100);
        let top10 = s.top_share(1000);
        assert!((0.08..0.20).contains(&top1), "top 1% share {top1}");
        assert!((0.30..0.50).contains(&top10), "top 10% share {top10}");
    }

    #[test]
    fn the_multi_currency_population_is_about_a_third() {
        let n = 10_000u64;
        let eur = (1..=n).filter(|&a| holds_eur(a, true)).count() as f64 / n as f64;
        assert!((0.30..0.40).contains(&eur), "{eur}");
        assert_eq!((1..=n).filter(|&a| holds_eur(a, false)).count(), 0);
    }

    #[test]
    fn some_postings_are_back_valued_and_none_by_more_than_a_month() {
        let u = Universe::generate(&Params::declared(2000, 3, 0.6, false));
        let back = u
            .txns
            .iter()
            .filter(|t| t.value_day < (t.batch / 50) as i32)
            .count() as f64
            / u.txns.len() as f64;
        assert!((0.01..0.05).contains(&back), "{back}");
        assert!(u
            .txns
            .iter()
            .all(|t| (t.batch / 50) as i32 - t.value_day <= 22));
    }
}
