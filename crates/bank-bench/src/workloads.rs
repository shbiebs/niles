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
    /// Which PostgreSQL query protocol the client used: `simple` or `extended`.
    ///
    /// Recorded per row because **both targets must use the same one in a run**, and until
    /// now nothing said which either used. libpq's `PQexec` is the simple protocol and
    /// `PQexecParams` is the extended one; they differ by a round trip and by whether the
    /// server re-plans, so a comparison in which one side used each would be measuring the
    /// protocol rather than the engine.
    pub protocol_path: &'static str,
    /// The miss rate of the partial view over this run, where the target has one.
    ///
    /// `None` for PostgreSQL, which has no partial state to miss in. Reported because a run
    /// whose budget exceeded the key count never evicted anything, so it measured the hit
    /// path exclusively while being presented as a measurement of the mechanism.
    pub miss_rate: Option<f64>,
}

impl Sample {
    pub fn ops_per_second(&self) -> f64 {
        if self.wall.as_secs_f64() <= 0.0 {
            return 0.0;
        }
        self.operations as f64 / self.wall.as_secs_f64()
    }

    /// One CSV line. The schema is fixed in `render.rs` and asserted by a test there.
    ///
    /// `not_run` is a sentence and sentences contain commas, so it is escaped here. The
    /// reader splits on a fixed field count, and one comma inside a reason silently shifted
    /// the `protocol_path` and `miss_rate` columns of every row that carried one — which is
    /// exactly the row a reader most wants to trust.
    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{:.3},{:.1},{:.1},{:.1},{},{},{},{}",
            self.workload,
            self.target,
            self.run,
            self.operations,
            self.wall.as_secs_f64() * 1000.0,
            self.p50.as_nanos() as f64 / 1000.0,
            self.p99.as_nanos() as f64 / 1000.0,
            self.ops_per_second(),
            self.durable,
            self.not_run
                .as_deref()
                .unwrap_or("")
                .replace(',', ";")
                .replace('\n', " "),
            self.protocol_path,
            self.miss_rate
                .map(|m| format!("{m:.4}"))
                .unwrap_or_else(|| "n/a".into())
        )
    }
}

/// **The query protocol both targets use.**
///
/// One constant, so the two sides cannot differ. libpq's `PQexec` is the simple protocol and
/// `PQexecParams` is the extended one; they differ by a round trip and by whether the server
/// re-plans, so a comparison in which PostgreSQL used one and Nilestream the other would be
/// measuring the protocol. Nothing recorded which either used, so nothing could have said.
pub const PROTOCOL_PATH: &str = "simple";

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
        // A run that did not happen used the protocol nothing used. Recorded rather than
        // left blank so the column is never ambiguous between "simple" and "unknown".
        protocol_path: "none",
        miss_rate: None,
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
    // Asked after the run, so the rate covers the operations just measured.
    let miss_rate = t.miss_rate();
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
        protocol_path: PROTOCOL_PATH,
        miss_rate,
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
        let sql = if t.name() == "postgres" {
            format!(
                "insert into postings (txn, acct, cur, amt, epoch) values \
                 ('bench-{run}-{i}', {from}, 'USD', -{amount}, {epoch}), \
                 ('bench-{run}-{i}', {to}, 'USD', {amount}, {epoch})"
            )
        } else {
            // The same two legs, in the same statement, sealed as one epoch. Nilestream's
            // ledger *refuses* the half-applied version, which is the guarantee this
            // workload is meant to be measuring the cost of rather than the absence of.
            format!(
                "insert into postings values ({txn}, {from}, 0, -{amount}), ({txn}, {to}, 0, {amount})",
                txn = 1_000_000 + (run as i64) * 1_000_000 + i as i64
            )
        };
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
        protocol_path: PROTOCOL_PATH,
        miss_rate: None,
    })
}

/// **One analytical statement, in each target's dialect.**
///
/// A table rather than two arrays, and that is the whole of the repair. The two arrays were
/// five statements against three, round-robined into *one composite sample*, so the reported
/// ratio compared a PostgreSQL set containing its cheapest statement (`count(*)`, 1.4ms)
/// against a Nilestream set that does not contain it. The gap was real; the number was not
/// the gap.
///
/// Pairing the dialects by *operation* makes the composite a like-for-like comparison and
/// leaves the coverage difference where it belongs: a statement Nilestream cannot express is
/// `nls: None`, is still run on PostgreSQL so its cost is on the record, and is excluded from
/// the ratio.
pub struct AnalyticalStatement {
    /// A stable identifier, used as the CSV's workload column (`analytical:<id>`) and as the
    /// key of `target::ANALYTICAL_BLOCKED`.
    pub id: &'static str,
    pub pg: &'static str,
    /// `None` when the construct is outside Nilestream's lowered fragment. The reason is in
    /// `target::ANALYTICAL_BLOCKED` under the same `id`, so a reader is never told only that
    /// something is missing.
    pub nls: Option<&'static str>,
}

/// The analytical statement set, paired by operation.
///
/// `group_by_acct` is the pairing that used to be absent: PostgreSQL ran
/// `group by acct order by sum(amt) desc limit 10` and Nilestream ran a plain `group by
/// acct`, and the two were averaged into one row as though they were the same statement.
/// They are now two entries — the plain grouping, which both targets run, and the ordered
/// top-ten, which is its own entry.
pub const ANALYTICAL_STATEMENTS: &[AnalyticalStatement] = &[
    AnalyticalStatement {
        id: "count_star",
        pg: "select count(*) from postings",
        nls: None,
    },
    AnalyticalStatement {
        id: "group_by_cur",
        pg: "select cur, sum(amt) from postings group by cur",
        nls: Some("select cur, sum(amt) from postings group by cur"),
    },
    AnalyticalStatement {
        id: "group_by_acct",
        pg: "select acct, sum(amt) from postings group by acct",
        nls: Some("select acct, sum(amt) from postings group by acct"),
    },
    AnalyticalStatement {
        id: "top_ten_by_sum",
        pg: "select acct, sum(amt) from postings group by acct order by sum(amt) desc limit 10",
        // In the common set since `order by <aggregate>` lowers. It was outside the fragment
        // for a reason that turned out to be a defect rather than a boundary: the key list
        // came out empty and the server answered with the *wrong ten rows*, silently. The
        // fragment did not widen to flatter this benchmark; a wrong answer was repaired and
        // the statement joined the set the ratio is computed over.
        nls: Some(
            "select acct, sum(amt) from postings group by acct order by sum(amt) desc limit 10",
        ),
    },
    AnalyticalStatement {
        id: "count_distinct_acct",
        pg: "select count(distinct acct) from postings",
        nls: None,
    },
    AnalyticalStatement {
        id: "sum_negative",
        pg: "select sum(amt) from postings where amt < 0",
        nls: Some("select sum(amt) from postings where amt < 0"),
    },
];

/// The statements both targets run — **the only ones the composite ratio is computed from**.
pub fn analytical_common() -> Vec<&'static AnalyticalStatement> {
    ANALYTICAL_STATEMENTS
        .iter()
        .filter(|s| s.nls.is_some())
        .collect()
}

/// **Analytical.** Scans and grouped roll-ups over the whole posting table.
///
/// Returns one sample per statement — `analytical:<id>`, so a composite gap can be attributed
/// to an operator rather than guessed at — followed by the composite `analytical` row, which
/// covers **only the statements both targets run**. A target's statements outside the common
/// set are measured and reported and do not enter the ratio.
pub fn analytical(
    t: &mut dyn Target,
    _accounts: i64,
    operations: u64,
    run: u32,
) -> Result<Vec<Sample>, WireError> {
    if let Some(reason) = t.unsupported("analytical") {
        return Ok(vec![skipped("analytical", t.name(), run, reason)]);
    }
    let is_pg = t.name() == "postgres";
    let common = analytical_common();
    // `operations` counts executions **of the common set**, so both targets run each common
    // statement the same number of times whatever else they can express.
    let rounds = (operations / common.len().max(1) as u64).max(1);

    let mut per_statement: Vec<(&AnalyticalStatement, Vec<Duration>)> = Vec::new();
    for st in ANALYTICAL_STATEMENTS {
        let sql = if is_pg { Some(st.pg) } else { st.nls };
        let Some(sql) = sql else { continue };
        let mut lat = Vec::with_capacity(rounds as usize);
        for _ in 0..rounds {
            let at = Instant::now();
            t.run(sql)?;
            lat.push(at.elapsed());
        }
        per_statement.push((st, lat));
    }

    let mut out = Vec::new();
    for (st, lat) in &per_statement {
        let wall: Duration = lat.iter().sum();
        let (p50, p99) = percentiles(lat.clone());
        out.push(Sample {
            workload: format!("analytical:{}", st.id),
            target: t.name().into(),
            run,
            operations: lat.len() as u64,
            wall,
            p50,
            p99,
            durable: false,
            not_run: None,
            protocol_path: PROTOCOL_PATH,
            miss_rate: None,
        });
    }

    // The composite: the common set only, and its wall clock is the sum of those statements'
    // latencies rather than the elapsed time of the whole loop — otherwise a target that can
    // express more statements would be charged for them in a ratio they are excluded from.
    let common_lat: Vec<Duration> = per_statement
        .iter()
        .filter(|(st, _)| st.nls.is_some())
        .flat_map(|(_, l)| l.iter().copied())
        .collect();
    let wall: Duration = common_lat.iter().sum();
    let (p50, p99) = percentiles(common_lat.clone());
    out.push(Sample {
        workload: "analytical".into(),
        target: t.name().into(),
        run,
        operations: common_lat.len() as u64,
        wall,
        p50,
        p99,
        durable: false,
        not_run: None,
        protocol_path: PROTOCOL_PATH,
        miss_rate: None,
    });
    Ok(out)
}

/// **The report workload: the same statement, served from a view that is being maintained.**
///
/// The analytical row measures a *cold reconstruction* — the fold runs over the whole base
/// for every query. That is one end of the phase diagram and the thesis is about the other:
/// a derived view kept current by the write path, so a report is a read of state that is
/// already correct rather than a recomputation of it. Nothing on the wire had ever measured
/// that end.
///
/// **"Warm" here means maintained, not cached.** A number taken from a view over a base that
/// stopped moving is a number about a cache, and a cache is not what the thesis claims. So
/// this workload holds a second connection open that appends at the OLTP rate for the whole
/// measurement, and the report is timed on the primary connection against a base that is
/// growing underneath it. Both targets get the same treatment: PostgreSQL's recompute over
/// its own growing table is the control, and it is a fair one, because that is what a
/// database without maintained state has to do.
///
/// The appends are counted and returned, because "the base was moving" is a claim this
/// workload makes and a reader should be able to check it. A run whose appender managed no
/// appends measured a quiesced base, and the sample says so rather than looking identical to
/// one that did not.
pub struct Report {
    pub sample: Sample,
    /// Appends the second connection completed during the measurement.
    pub appends: u64,
    /// Appends per second the appender was **asked** for, and what it achieved.
    ///
    /// Both, because the first run of this workload let the appender go as fast as it could
    /// and PostgreSQL's side got 108 appends against Nilestream's 21 — five times the write
    /// pressure on one arm of a two-arm comparison, on a two-core host where that pressure
    /// is taken out of the thing being timed. A rate the harness *asked* for is the same on
    /// both sides; a rate it achieved is a fact about the run, and a run that could not hold
    /// the rate is a run whose report was measured under lighter load than it claims.
    pub append_rate_target: f64,
    pub append_rate_achieved: f64,
    /// What the server said it would do with the report statement, when it can be asked.
    ///
    /// `explain` on the Nilestream side; `None` for a target with no such surface. The
    /// report row asserts a *mechanism* — an answer read out of maintained state — and a
    /// latency alone cannot distinguish that from a fold that happened to be quick.
    pub serve_path: Option<String>,
    /// Base rows before and after, when the target can be asked. The pair a reader checks
    /// the appender against.
    pub base_before: Option<u64>,
    pub base_after: Option<u64>,
}

/// The statement a report is: every account's total, which is what the maintained view holds.
pub const REPORT_STATEMENT: AnalyticalStatement = AnalyticalStatement {
    id: "report_totals",
    pg: "select acct, sum(amt) from postings group by acct",
    nls: Some("select acct, sum(amt) from postings group by acct"),
};

/// Measure the report while a second connection appends to the base.
///
/// `open` yields a fresh connection to the *same* target — the appender must not share the
/// measured connection, or the appends would be serialised into the latency being timed and
/// the workload would be measuring a mixture rather than a report under load.
pub fn report(
    t: &mut dyn Target,
    operations: u64,
    run: u32,
    append_rate: f64,
    open: &(dyn Fn() -> Result<crate::wire::Client, WireError> + Sync),
    append: &(dyn Fn(u32, u64) -> String + Sync),
) -> Result<Report, WireError> {
    if let Some(reason) = t.unsupported("report") {
        return Ok(Report {
            sample: skipped("report", t.name(), run, reason),
            appends: 0,
            append_rate_target: append_rate,
            append_rate_achieved: 0.0,
            serve_path: None,
            base_before: None,
            base_after: None,
        });
    }
    let is_pg = t.name() == "postgres";
    let sql = if is_pg {
        REPORT_STATEMENT.pg
    } else {
        REPORT_STATEMENT
            .nls
            .expect("the report statement is expressible on both targets")
    };

    // **Warm the view before the clock starts.** On the Nilestream side the first report
    // installs the maintained view's keys; charging that to the measurement would report the
    // cost of becoming warm as the cost of being warm. PostgreSQL gets the same two
    // untimed executions, which warm its buffer cache — the same courtesy, for the same
    // reason.
    t.run(sql)?;
    t.run(sql)?;

    // Ask the server what it will do, before doing it. A `report` row that claims a
    // maintained view and cannot say so is a latency with a story attached.
    let serve_path = t.serve_path(sql);

    let base_before = t.base_rows();
    let stop = std::sync::atomic::AtomicBool::new(false);
    let done = std::sync::atomic::AtomicU64::new(0);
    let gap = if append_rate > 0.0 {
        Duration::from_secs_f64(1.0 / append_rate)
    } else {
        Duration::ZERO
    };

    let (latencies, wall) = std::thread::scope(|scope| {
        let stop = &stop;
        let done = &done;
        scope.spawn(move || {
            // A failure to open is not a failure of the run: it is recorded as zero appends,
            // and the caller's `appends` column says the base did not move. Silently
            // measuring a quiesced base is the failure this counter exists to prevent.
            let Ok(mut client) = open() else { return };
            let mut i = 0u64;
            let from = Instant::now();
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                if client.simple(&append(run, i)).is_err() {
                    return;
                }
                i += 1;
                done.store(i, std::sync::atomic::Ordering::Relaxed);
                // **Paced against the wall clock rather than by sleeping a fixed gap.** A
                // fixed sleep between statements gives a rate of `1/(gap + service time)`,
                // which is a different rate on a target whose service time is different —
                // which is exactly the two targets here. Sleeping until the wall clock
                // reaches `i * gap` gives the same rate on both, or falls behind visibly.
                let due = gap * i as u32;
                if let Some(left) = due.checked_sub(from.elapsed()) {
                    std::thread::sleep(left);
                }
            }
        });

        let mut lat = Vec::with_capacity(operations as usize);
        let started = Instant::now();
        for _ in 0..operations {
            let at = Instant::now();
            match t.run(sql) {
                Ok(_) => lat.push(at.elapsed()),
                Err(e) => {
                    // Release the appender before returning, or its thread outlives the
                    // scope's join and the failure becomes a hang.
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                    return (Err(e), Duration::ZERO);
                }
            }
        }
        let wall = started.elapsed();
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        (Ok(lat), wall)
    });

    let latencies = latencies?;
    let appends = done.load(std::sync::atomic::Ordering::Relaxed);
    let achieved = if wall.as_secs_f64() > 0.0 {
        appends as f64 / wall.as_secs_f64()
    } else {
        0.0
    };
    let base_after = t.base_rows();
    let (p50, p99) = percentiles(latencies.clone());
    Ok(Report {
        sample: Sample {
            workload: "report".into(),
            target: t.name().into(),
            run,
            operations: latencies.len() as u64,
            wall,
            p50,
            p99,
            durable: false,
            not_run: None,
            protocol_path: PROTOCOL_PATH,
            miss_rate: None,
        },
        appends,
        append_rate_target: append_rate,
        append_rate_achieved: achieved,
        serve_path,
        base_before,
        base_after,
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
        // The same operation in each target's dialect: one durable append of a posting that
        // conserves. Nilestream's ledger refuses a set that does not, so the amount is zero
        // — a set of one row summing to zero — which is also what PostgreSQL's row carries,
        // so the two are the same write and not merely similarly named ones.
        let sql = if t.name() == "postgres" {
            format!(
                "insert into postings (txn, acct, cur, amt, epoch) values \
                 ('durable-{run}-{i}', {acct}, 'USD', 0, {})",
                2_000_000 + i as i64
            )
        } else {
            // **The run number belongs in the identity.** Without it the second run
            // replays the first's transaction numbers and the ledger refuses every one of
            // them as a duplicate — which is the idempotency guarantee working exactly as
            // designed, reported as "a workload failed". A benchmark that re-submits the
            // same transaction is not measuring throughput.
            format!(
                "insert into postings values ({}, {acct}, 0, 0)",
                2_000_000 + (run as i64) * 1_000_000 + i as i64
            )
        };
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
        protocol_path: PROTOCOL_PATH,
        miss_rate: None,
    })
}

/// **One level of a scaling run**: a workload, a target, a connection count, one run.
///
/// A separate type from [`Sample`] on purpose. The contract table is a **single-connection**
/// measurement — `SPEC-ENGINE.md` Part 0 states its four targets without a concurrency
/// qualifier — and a row from a four-connection level must never be able to reach it. Two
/// types with two CSV schemas and two documents is what makes that a compile-time property
/// rather than a convention someone has to remember at the call site.
///
/// The threads are **pooled**: `operations` is the total across every connection, `wall` is
/// the elapsed time of the level as a whole, and the percentiles are taken over every
/// thread's latencies together. A per-thread median averaged across threads would report the
/// latency of a typical connection while the question is what the *client* saw.
#[derive(Debug, Clone)]
pub struct ScalingSample {
    pub workload: String,
    pub target: String,
    /// How many connections drove this level. **The column that makes the row mean
    /// something**: without it a scaling table is four unlabelled throughput figures.
    pub connections: u32,
    pub run: u32,
    /// Operations across **all** connections.
    pub operations: u64,
    /// Elapsed time of the level, measured from the barrier release — connection setup is
    /// outside it, so a level is not charged for opening its own sockets.
    pub wall: Duration,
    pub p50: Duration,
    pub p99: Duration,
    pub durable: bool,
    pub not_run: Option<String>,
}

/// The scaling CSV's schema, in one place, asserted by `render`'s tests.
pub const SCALING_CSV_HEADER: &str =
    "workload,target,connections,run,operations,wall_ms,p50_us,p99_us,ops_per_second,durable,not_run";

impl ScalingSample {
    pub fn ops_per_second(&self) -> f64 {
        if self.wall.as_secs_f64() <= 0.0 {
            return 0.0;
        }
        self.operations as f64 / self.wall.as_secs_f64()
    }

    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{:.3},{:.1},{:.1},{:.1},{},{}",
            self.workload,
            self.target,
            self.connections,
            self.run,
            self.operations,
            self.wall.as_secs_f64() * 1000.0,
            self.p50.as_nanos() as f64 / 1000.0,
            self.p99.as_nanos() as f64 / 1000.0,
            self.ops_per_second(),
            self.durable,
            self.not_run
                .as_deref()
                .unwrap_or("")
                .replace(',', ";")
                .replace('\n', " ")
        )
    }
}

/// A level that did not run, and why — the scaling table's equivalent of [`skipped`].
pub fn scaling_skipped(
    workload: &str,
    target: &str,
    connections: u32,
    run: u32,
    reason: String,
) -> ScalingSample {
    ScalingSample {
        workload: workload.into(),
        target: target.into(),
        connections,
        run,
        operations: 0,
        wall: Duration::ZERO,
        p50: Duration::ZERO,
        p99: Duration::ZERO,
        durable: false,
        not_run: Some(reason),
    }
}

/// Which level of a scaling run is about to be driven.
///
/// A struct rather than six positional parameters: `(u32, u32, u64)` in a row is three
/// numbers a caller can transpose silently, and a level that ran `run` connections for
/// `connections` operations would produce a plausible table nobody could see was wrong.
#[derive(Debug, Clone, Copy)]
pub struct Level<'a> {
    pub workload: &'a str,
    pub target: &'a str,
    pub connections: u32,
    pub run: u32,
    /// Operations **per connection**. A level of four connections therefore issues four times
    /// the work of a level of one, which is the shape the question needs: a server that
    /// scales keeps the wall clock flat and raises ops/s, and one that serialises does not.
    pub per_connection: u64,
    pub durable: bool,
}

/// **Drive one level from `connections` threads and pool what they saw.**
///
/// `open` returns a fresh client per thread — every connection is its own socket, because a
/// shared one would serialise in the harness and the number would be measuring this file.
/// `statement` is handed `(thread, operation)` and returns the SQL that thread should send;
/// composing an identity out of both is how appends stay distinct. The audit's scratch
/// harness reused one sequence across levels, so the four-connection level re-submitted the
/// one-connection level's transaction ids and the ledger refused every one of them as a
/// duplicate — idempotency working exactly as designed, reported as a throughput collapse.
///
/// Every thread opens its connection, then waits on a barrier. The clock starts when the
/// barrier releases, so a level is charged for its operations and not for its sockets.
pub fn concurrent(
    level: Level<'_>,
    open: &(dyn Fn() -> Result<crate::wire::Client, WireError> + Sync),
    statement: &(dyn Fn(u32, u64) -> String + Sync),
) -> ScalingSample {
    use std::sync::Barrier;
    let Level {
        workload,
        target,
        connections,
        run,
        per_connection,
        durable,
    } = level;
    let n = connections.max(1);
    // `n + 1`: the main thread waits too, and takes the clock the moment the gate opens.
    let gate = Barrier::new(n as usize + 1);
    let (per_thread, wall) = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for thread in 0..n {
            let gate = &gate;
            handles.push(scope.spawn(move || -> Result<Vec<Duration>, WireError> {
                // Open before the gate: connection setup is not part of the measurement.
                let mut client = match open() {
                    Ok(c) => {
                        gate.wait();
                        c
                    }
                    Err(e) => {
                        // Still release the gate, or the other threads and the main thread
                        // block forever on a failure this function is meant to report.
                        gate.wait();
                        return Err(e);
                    }
                };
                let mut lat = Vec::with_capacity(per_connection as usize);
                for i in 0..per_connection {
                    let sql = statement(thread, i);
                    let at = Instant::now();
                    client.simple(&sql)?;
                    lat.push(at.elapsed());
                }
                Ok(lat)
            }));
        }
        gate.wait();
        let started = Instant::now();
        let mut per_thread: Vec<Result<Vec<Duration>, WireError>> = Vec::new();
        for h in handles {
            per_thread.push(h.join().unwrap_or_else(|_| {
                Err(WireError::Protocol("a connection thread panicked".into()))
            }));
        }
        (per_thread, started.elapsed())
    });

    let mut pooled = Vec::new();
    for r in per_thread {
        match r {
            Ok(l) => pooled.extend(l),
            Err(e) => {
                return scaling_skipped(
                    workload,
                    target,
                    connections,
                    run,
                    format!("a connection failed part-way through the level: {e}"),
                )
            }
        }
    }
    let operations = pooled.len() as u64;
    let (p50, p99) = percentiles(pooled);
    ScalingSample {
        workload: workload.into(),
        target: target.into(),
        connections,
        run,
        operations,
        wall,
        p50,
        p99,
        durable,
        not_run: None,
    }
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
        // The reason this test used to name — "no write surface over the wire" — is no
        // longer true of this server, so the example is a different one. A skipped run is
        // still a row rather than an omission: a results table with a missing row invites a
        // reader to assume the number was unremarkable.
        let s = skipped(
            "oltp",
            "postgresql",
            3,
            "no PostgreSQL server on this host".into(),
        );
        let line = s.to_csv();
        assert!(line.starts_with("oltp,postgresql,3,0,"), "{line}");
        assert!(line.contains("no PostgreSQL server on this host"), "{line}");
        assert!(
            line.ends_with(",none,n/a"),
            "a run that did not happen used no protocol and has no miss rate to report: {line}"
        );
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
            protocol_path: PROTOCOL_PATH,
            miss_rate: Some(0.37),
        };
        assert_eq!(s.to_csv().split(',').count(), 12);
        assert!(
            s.to_csv().ends_with(",simple,0.3700"),
            "the protocol and the miss rate are the last two columns: {}",
            s.to_csv()
        );
        assert!(
            (s.ops_per_second() - 4_000.0).abs() < 1.0,
            "{}",
            s.ops_per_second()
        );
    }
}
