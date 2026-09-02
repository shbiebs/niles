//! The four workload classes of `SPEC-ENGINE.md` Part 0, as things that can be run.
//!
//! Each is a closed loop: prepare, run N operations against a target, report wall-clock and
//! latency percentiles. Nothing here interprets a result — that is `render`'s job — and
//! nothing here knows which target it is driving, which is what keeps the comparison fair.
//!
//! # Percentiles, and why p99 is reported alongside the median
//!
//! A median throughput figure hides the thing an operator actually feels. `SPEC-ENGINE`'s
//! point-lookup row is a *latency* claim, and a system with a good median and a bad tail
//! fails it while looking excellent in a single number. Both are recorded, from the same run,
//! so neither can be quoted without the other being available.

use crate::target::Target;
use crate::wire::WireError;
use std::time::{Duration, Instant};

/// A deterministic pseudo-random sequence.
///
/// SplitMix64: small, well-distributed, and — the reason it is here rather than a call to a
/// library — **reproducible from a seed across machines and runs**. A benchmark whose access
/// pattern differs between two runs is a benchmark whose two numbers cannot be compared, and
/// a phase diagram built from it would be measuring the sequence rather than the engine.
pub struct Rng(u64);

impl Rng {
    pub fn seeded(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A key in `1..=n`, uniform.
    pub fn key(&mut self, n: i64) -> i64 {
        1 + (self.next_u64() % n.max(1) as u64) as i64
    }

    /// A key in `1..=n`, **skewed**: `hot_share` of draws land in the hottest 1% of keys.
    ///
    /// Real banking traffic is not uniform, and the difference is not cosmetic: a skewed
    /// workload is where partial materialisation is supposed to win, so measuring only the
    /// uniform case would omit the case the thesis is about.
    pub fn skewed_key(&mut self, n: i64, hot_share: f64) -> i64 {
        let hot = (n / 100).max(1);
        if (self.next_u64() % 1_000_000) as f64 / 1_000_000.0 < hot_share {
            1 + (self.next_u64() % hot as u64) as i64
        } else {
            self.key(n)
        }
    }
}

/// What one workload run produced.
#[derive(Debug, Clone)]
pub struct Sample {
    pub workload: String,
    pub target: String,
    pub run: u32,
    pub operations: u64,
    pub wall: Duration,
    pub p50: Duration,
    pub p99: Duration,
    pub durable: bool,
    /// Non-empty when the run did not happen, and why.
    pub not_run: Option<String>,
}

impl Sample {
    pub fn ops_per_second(&self) -> f64 {
        if self.wall.as_secs_f64() <= 0.0 {
            return 0.0;
        }
        self.operations as f64 / self.wall.as_secs_f64()
    }

    /// One CSV line. The schema is fixed in `render.rs` and asserted by a test there.
    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{:.3},{:.1},{:.1},{:.1},{},{}",
            self.workload,
            self.target,
            self.run,
            self.operations,
            self.wall.as_secs_f64() * 1000.0,
            self.p50.as_nanos() as f64 / 1000.0,
            self.p99.as_nanos() as f64 / 1000.0,
            self.ops_per_second(),
            self.durable,
            self.not_run.as_deref().unwrap_or("")
        )
    }
}

/// A run that did not happen, and the reason.
///
/// Constructed rather than omitted. A results table with a missing row invites a reader to
/// assume the number was unremarkable; one with `NOT RUN` and a sentence does not.
pub fn skipped(workload: &str, target: &str, run: u32, reason: String) -> Sample {
    Sample {
        workload: workload.into(),
        target: target.into(),
        run,
        operations: 0,
        wall: Duration::ZERO,
        p50: Duration::ZERO,
        p99: Duration::ZERO,
        durable: false,
        not_run: Some(reason),
    }
}

/// The median and the 99th percentile, by **nearest rank on `(n-1)·q`**.
///
/// The convention is stated because percentile definitions differ by up to a whole sample at
/// small n, and a benchmark that did not say which one it used would be reporting a figure
/// nobody else could reproduce. No interpolation: every value reported is a latency that was
/// actually observed, which matters when the question is "how slow did a request get".
fn percentiles(mut latencies: Vec<Duration>) -> (Duration, Duration) {
    if latencies.is_empty() {
        return (Duration::ZERO, Duration::ZERO);
    }
    latencies.sort_unstable();
    let at = |q: f64| {
        let i = ((latencies.len() as f64 - 1.0) * q).round() as usize;
        latencies[i.min(latencies.len() - 1)]
    };
    (at(0.50), at(0.99))
}

/// **Point lookup.** One account's balance, at the frontier.
///
/// The workload the performance contract asks for *parity* on rather than a multiple, and the
/// one where an engine that is fast at scans can still be slow: it is a single key, so almost
/// all of the time is index descent, tuple visibility and protocol.
pub fn point(
    t: &mut dyn Target,
    accounts: i64,
    operations: u64,
    run: u32,
    seed: u64,
) -> Result<Sample, WireError> {
    if let Some(reason) = t.unsupported("point") {
        return Ok(skipped("point", t.name(), run, reason));
    }
    // **The simple protocol, on both targets.** An earlier version prepared a statement
    // first and then did not use it, which broke the Nilestream run outright: `nilestreamd`
    // implements the extended query protocol in `extended.rs` but does not wire it into its
    // connection loop, so `Parse` is refused.
    //
    // Removing the prepare was the right fix rather than a workaround. The rule this harness
    // exists to keep is that both targets pay the same client cost, and a run where one side
    // used prepared statements and the other could not would violate it in the direction that
    // flatters PostgreSQL. Preparing on both is a fair comparison and so is preparing on
    // neither; preparing on one is not.
    let mut rng = Rng::seeded(seed);
    let mut latencies = Vec::with_capacity(operations as usize);
    let started = Instant::now();
    for _ in 0..operations {
        let key = rng.skewed_key(accounts, 0.9);
        let sql = format!("select acct, sum(amt) from postings where acct = {key} group by acct");
        let at = Instant::now();
        t.run(&sql)?;
        latencies.push(at.elapsed());
    }
    let wall = started.elapsed();
    let (p50, p99) = percentiles(latencies);
    Ok(Sample {
        workload: "point".into(),
        target: t.name().into(),
        run,
        operations,
        wall,
        p50,
        p99,
        durable: false,
        not_run: None,
    })
}

/// **OLTP.** A transfer: two conserved legs, one transaction.
///
/// Deliberately an append of two postings rather than an update of two balances. The whole
/// architecture rests on a balance being a fold, so benchmarking an `UPDATE balances` would
/// be measuring a system this one is not.
pub fn oltp(
    t: &mut dyn Target,
    accounts: i64,
    operations: u64,
    run: u32,
    seed: u64,
) -> Result<Sample, WireError> {
    if let Some(reason) = t.unsupported("oltp") {
        return Ok(skipped("oltp", t.name(), run, reason));
    }
    let durable = t.is_durable();
    let mut rng = Rng::seeded(seed);
    let mut latencies = Vec::with_capacity(operations as usize);
    let started = Instant::now();
    for i in 0..operations {
        let from = rng.skewed_key(accounts, 0.5);
        let mut to = rng.key(accounts);
        if to == from {
            to = 1 + (to % accounts.max(1));
        }
        let amount = 1 + (rng.next_u64() % 10_000) as i64;
        let epoch = 1_000_000 + i as i64;
        // One statement, so one implicit transaction: a transfer that could be half-applied
        // is the failure the ledger design exists to prevent, and a benchmark that allowed it
        // would be measuring a weaker guarantee.
        let sql = format!(
            "insert into postings (txn, acct, cur, amt, epoch) values \
             ('bench-{run}-{i}', {from}, 'USD', -{amount}, {epoch}), \
             ('bench-{run}-{i}', {to}, 'USD', {amount}, {epoch})"
        );
        let at = Instant::now();
        t.run(&sql)?;
        latencies.push(at.elapsed());
    }
    let wall = started.elapsed();
    let (p50, p99) = percentiles(latencies);
    Ok(Sample {
        workload: "oltp".into(),
        target: t.name().into(),
        run,
        operations,
        wall,
        p50,
        p99,
        durable,
        not_run: None,
    })
}

/// **Analytical.** Scans and grouped roll-ups over the whole posting table.
///
/// The workload the contract claims an order of magnitude on, and — per the roadmap's own
/// honesty note — the one where the win comes mostly from storage layout rather than from
/// execution. Reported so that the attribution can be checked rather than assumed.
pub fn analytical(
    t: &mut dyn Target,
    _accounts: i64,
    operations: u64,
    run: u32,
) -> Result<Sample, WireError> {
    if let Some(reason) = t.unsupported("analytical") {
        return Ok(skipped("analytical", t.name(), run, reason));
    }
    let queries = [
        "select count(*) from postings",
        "select cur, sum(amt) from postings group by cur",
        "select acct, sum(amt) from postings group by acct order by sum(amt) desc limit 10",
        "select count(distinct acct) from postings",
        "select sum(amt) from postings where amt < 0",
    ];
    let mut latencies = Vec::new();
    let started = Instant::now();
    for i in 0..operations {
        let sql = queries[(i as usize) % queries.len()];
        let at = Instant::now();
        t.run(sql)?;
        latencies.push(at.elapsed());
    }
    let wall = started.elapsed();
    let (p50, p99) = percentiles(latencies);
    Ok(Sample {
        workload: "analytical".into(),
        target: t.name().into(),
        run,
        operations,
        wall,
        p50,
        p99,
        durable: false,
        not_run: None,
    })
}

/// **Durable commit.** The `fsync` cost, measured rather than assumed.
///
/// One tiny transaction at a time, so the number is dominated by the commit rather than by
/// the work. `durable` is recorded per sample: a run against a target whose
/// `synchronous_commit` is off is a different measurement, and the CSV must not let the two
/// be confused.
pub fn durable(
    t: &mut dyn Target,
    accounts: i64,
    operations: u64,
    run: u32,
    seed: u64,
) -> Result<Sample, WireError> {
    if let Some(reason) = t.unsupported("durable") {
        return Ok(skipped("durable", t.name(), run, reason));
    }
    let is_durable = t.is_durable();
    let mut rng = Rng::seeded(seed);
    let mut latencies = Vec::with_capacity(operations as usize);
    let started = Instant::now();
    for i in 0..operations {
        let acct = rng.key(accounts);
        let sql = format!(
            "insert into postings (txn, acct, cur, amt, epoch) values \
             ('durable-{run}-{i}', {acct}, 'USD', 0, {})",
            2_000_000 + i as i64
        );
        let at = Instant::now();
        t.run(&sql)?;
        latencies.push(at.elapsed());
    }
    let wall = started.elapsed();
    let (p50, p99) = percentiles(latencies);
    Ok(Sample {
        workload: "durable".into(),
        target: t.name().into(),
        run,
        operations,
        wall,
        p50,
        p99,
        durable: is_durable,
        not_run: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sequence_is_reproducible_from_its_seed() {
        // The property a benchmark's access pattern must have. Two runs that touched
        // different keys produce two numbers that cannot be compared, and a phase diagram
        // built from them would be measuring the sequence.
        let a: Vec<u64> = (0..50).map(|_| Rng::seeded(7).next_u64()).collect();
        assert!(
            a.windows(2).all(|w| w[0] == w[1]),
            "same seed, same first draw"
        );

        let mut x = Rng::seeded(7);
        let mut y = Rng::seeded(7);
        for _ in 0..1_000 {
            assert_eq!(x.next_u64(), y.next_u64());
        }
        assert_ne!(Rng::seeded(7).next_u64(), Rng::seeded(8).next_u64());
    }

    #[test]
    fn keys_stay_in_range_including_at_the_boundaries() {
        let mut r = Rng::seeded(1);
        for _ in 0..10_000 {
            let k = r.key(100);
            assert!((1..=100).contains(&k), "{k}");
            let s = r.skewed_key(100, 0.9);
            assert!((1..=100).contains(&s), "{s}");
        }
        assert_eq!(
            Rng::seeded(1).key(1),
            1,
            "a single-account ledger still works"
        );
    }

    #[test]
    fn a_skewed_draw_concentrates_on_the_hot_keys_and_a_uniform_one_does_not() {
        // The distribution the thesis is about. If `skewed_key` were uniform, the workload
        // where partial materialisation is supposed to win would never be exercised.
        let mut r = Rng::seeded(42);
        let hot = |k: i64| k <= 10;
        let skewed_hits = (0..10_000)
            .filter(|_| hot(r.skewed_key(1_000, 0.9)))
            .count();
        let uniform_hits = (0..10_000).filter(|_| hot(r.key(1_000))).count();
        assert!(
            skewed_hits > 8_000,
            "90% should land in the hot 1%: {skewed_hits}"
        );
        assert!(
            uniform_hits < 500,
            "and a uniform draw should not: {uniform_hits}"
        );
    }

    #[test]
    fn percentiles_are_order_statistics_and_p99_is_not_the_maximum() {
        // Nearest rank on (n-1)·q, stated in the function's docs: for 1..=100 that puts p50
        // at index 50 (the 51st sample) and p99 at index 98. No interpolation, so every
        // number reported is a latency something actually took.
        let l: Vec<Duration> = (1..=100).map(Duration::from_micros).collect();
        let (p50, p99) = percentiles(l);
        assert_eq!(p50, Duration::from_micros(51));
        assert_eq!(p99, Duration::from_micros(99));
        assert!(p99 < Duration::from_micros(100), "p99 is not the maximum");

        // With a long tail, p50 must be unmoved by the outliers — which is the reason both
        // are reported: a good median with a bad tail fails a latency claim while looking
        // excellent in a single number.
        let mut l: Vec<Duration> = (0..980).map(|_| Duration::from_micros(10)).collect();
        l.extend((0..20).map(|_| Duration::from_secs(1)));
        let (p50, p99) = percentiles(l);
        assert_eq!(p50, Duration::from_micros(10));
        assert!(
            p99 >= Duration::from_secs(1),
            "the tail is visible: {p99:?}"
        );

        // And the boundary case, which is worth pinning down rather than discovering in a
        // results table: with *exactly* 1% of samples slow, p99 sits on the boundary and
        // reports the fast value. That is correct — 99% of requests really were fast — and it
        // is why a p99 alone is not a statement about the worst case.
        let mut l: Vec<Duration> = (0..990).map(|_| Duration::from_micros(10)).collect();
        l.extend((0..10).map(|_| Duration::from_secs(1)));
        let (_, p99) = percentiles(l);
        assert_eq!(p99, Duration::from_micros(10));
    }

    #[test]
    fn percentiles_of_nothing_are_zero_rather_than_a_panic() {
        assert_eq!(percentiles(Vec::new()), (Duration::ZERO, Duration::ZERO));
    }

    #[test]
    fn a_skipped_run_carries_its_reason_into_the_csv() {
        let s = skipped(
            "oltp",
            "nilestream",
            3,
            "no write surface over the wire".into(),
        );
        let line = s.to_csv();
        assert!(line.starts_with("oltp,nilestream,3,0,"), "{line}");
        assert!(line.ends_with("no write surface over the wire"), "{line}");
        assert_eq!(
            s.ops_per_second(),
            0.0,
            "and reports no throughput rather than infinity"
        );
    }

    #[test]
    fn a_csv_line_has_the_declared_number_of_columns() {
        let s = Sample {
            workload: "point".into(),
            target: "postgres".into(),
            run: 1,
            operations: 1_000,
            wall: Duration::from_millis(250),
            p50: Duration::from_micros(180),
            p99: Duration::from_micros(900),
            durable: true,
            not_run: None,
        };
        assert_eq!(s.to_csv().split(',').count(), 10);
        assert!(
            (s.ops_per_second() - 4_000.0).abs() < 1.0,
            "{}",
            s.ops_per_second()
        );
    }
}
