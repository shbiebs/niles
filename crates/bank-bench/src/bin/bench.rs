//! `bench` — the wall-clock harness.
//!
//! ```text
//! bench --calibrate                  # is the harness measuring what it thinks it is?
//! bench --run     [--out DIR]        # every workload, both targets, ten runs each
//! bench --render  [--out DIR]        # the Part 0 table, from the CSVs and nothing else
//!
//! bench --run --host-nls                # host the daemon on a thread, run, stop it
//! ```
//!
//! # Why the harness hosts the daemon itself
//!
//! `--host-nls` starts Nilestream's listener on a **thread of this process**, waits for the
//! port, runs the benchmark, and drops it. One command reproduces a whole result — which is
//! what `docs/BENCHMARK.md` should be able to say — and it removes a mistake that is invisible
//! in the output: a run against a *previous* daemon still holding the port with different
//! settings or an older build. A port answering is not evidence that the right thing is
//! answering.
//!
//! **This does not weaken the comparison, and the distinction is worth being exact about.**
//! The fairness rule is that both targets pay the same *client* cost. Hosting the listener on
//! a thread changes where the server runs, not how the client talks to it: the socket, the
//! protocol framing and the round trip are all still there, and the same `wire::Client` drives
//! both. Calling `RevEngine::read` directly would have been the shortcut; this is what avoids
//! it.
//!
//! # The calibration gate, and what it took to get right
//!
//! A benchmark's most dangerous failure is not being wrong; it is being wrong *plausibly*. A
//! harness with a mistake in its loop produces numbers that look like measurements, and
//! nothing in the output says otherwise. So this binary refuses to write results until it has
//! checked itself.
//!
//! The **first** version of that check compared PostgreSQL against a published figure — 333
//! durable transactions per second per core, from `docs/research/performance-baselines.md` —
//! and it fired on the first run at 16× the baseline. The gate was right and the calibration
//! was wrong, and the reason is worth keeping in the source.
//!
//! A durable commit is an `fsync`, and an `fsync` costs whatever the device charges. The same
//! literature records the spread: 1.6–12.4µs with power-loss protection, 891–2974µs without —
//! a factor of a thousand, sitting directly under the transaction rate. 333 txn/s is not a
//! property of PostgreSQL; it is a property of PostgreSQL **on a ~3ms-fsync device**. The
//! machine this ran on syncs in 109µs, so 333 txn/s there would have meant the harness was
//! broken, and the published figure would have hidden that by agreeing with it.
//!
//! So the gate calibrates against **the device**, measured moments before the run: a
//! single-connection durable workload must land below its own `fsync` ceiling and not far
//! below it. Both bounds are machine-independent, which is exactly what the published number
//! was not. See `storage.rs`.

use bank_bench::render::{self, CSV_HEADER};
use bank_bench::target::{nilestream_gap, NilestreamTarget, PgTarget, Target};
use bank_bench::workloads::{self, Sample};
use std::collections::BTreeMap;
use std::io::Write;

/// The published figure, reported for context and **not** used as the gate.
///
/// PostgreSQL 18.1, durable OLTP, per core, from `docs/research/performance-baselines.md`.
/// It is printed beside the measurement so a reader can see the ratio and the storage
/// explanation together, rather than either alone.
const PUBLISHED_PG_TPS_PER_CORE: f64 = 333.0;

struct Args {
    calibrate: bool,
    run: bool,
    render: bool,
    out: String,
    pg_host: String,
    pg_port: u16,
    pg_user: String,
    pg_db: String,
    nls_port: u16,
    accounts: i64,
    operations: u64,
    runs: u32,
    /// Host Nilestream's listener on a thread of this process.
    host_nls: bool,
    /// Conserved pairs per account, on **both** targets. Named `rounds` because it is no
    /// longer a Nilestream-only knob: the two bases must be the same size or the analytical
    /// ratio compares two different problems. `--nls-rounds` is still accepted, and means the
    /// same thing on both sides.
    rounds: u32,
    nls_budget: usize,
    /// Overwrite the **committed** `results/E16-wallclock.md`.
    ///
    /// Off by default, and that is the repair. `write_all` wrote the committed document on
    /// every `--run` whatever `--out` said, so any exploratory run — an audit's, a bisect's,
    /// a CI job's — left the working tree dirty with a results file measured under whatever
    /// flags that run happened to use. A benchmark must not touch a committed artifact
    /// unless it was asked to publish one.
    publish: bool,
}

impl Args {
    fn parse() -> Args {
        let argv: Vec<String> = std::env::args().collect();
        let mut a = Args {
            calibrate: false,
            run: false,
            render: false,
            out: "results/E16-wallclock".into(),
            pg_host: "127.0.0.1".into(),
            pg_port: 5433,
            pg_user: "bench".into(),
            pg_db: "postgres".into(),
            nls_port: 5434,
            accounts: 10_000,
            operations: 2_000,
            runs: 10,
            host_nls: false,
            rounds: 1,
            // **A budget that binds.** It was 100,000 against 10,000 accounts, so after
            // warm-up the view held every key and nothing was ever evicted: the point
            // workload measured the hit path exclusively while being presented as a
            // measurement of partial materialisation. A budget of a quarter of the key
            // space forces the miss path to run, which is the path the phase diagram is
            // about. Override with `--nls-budget`.
            nls_budget: 2_500,
            publish: false,
        };
        let mut i = 1;
        while i < argv.len() {
            match argv[i].as_str() {
                "--calibrate" => a.calibrate = true,
                "--run" => a.run = true,
                "--render" => a.render = true,
                "--out" => {
                    a.out = argv[i + 1].clone();
                    i += 1;
                }
                "--pg-host" => {
                    a.pg_host = argv[i + 1].clone();
                    i += 1;
                }
                "--pg-port" => {
                    a.pg_port = argv[i + 1].parse().unwrap_or(a.pg_port);
                    i += 1;
                }
                "--pg-user" => {
                    a.pg_user = argv[i + 1].clone();
                    i += 1;
                }
                "--pg-db" => {
                    a.pg_db = argv[i + 1].clone();
                    i += 1;
                }
                "--nls-port" => {
                    a.nls_port = argv[i + 1].parse().unwrap_or(a.nls_port);
                    i += 1;
                }
                "--accounts" => {
                    a.accounts = argv[i + 1].parse().unwrap_or(a.accounts);
                    i += 1;
                }
                "--operations" => {
                    a.operations = argv[i + 1].parse().unwrap_or(a.operations);
                    i += 1;
                }
                "--runs" => {
                    a.runs = argv[i + 1].parse().unwrap_or(a.runs);
                    i += 1;
                }
                "--host-nls" => a.host_nls = true,
                "--publish" => a.publish = true,
                "--rounds" | "--nls-rounds" => {
                    a.rounds = argv[i + 1].parse().unwrap_or(a.rounds);
                    i += 1;
                }
                "--nls-budget" => {
                    a.nls_budget = argv[i + 1].parse().unwrap_or(a.nls_budget);
                    i += 1;
                }
                other => eprintln!("bench: ignoring unknown argument `{other}`"),
            }
            i += 1;
        }
        a
    }
}

fn main() {
    let args = Args::parse();
    if !(args.calibrate || args.run || args.render) {
        eprintln!("bench: one of --calibrate, --run, --render is required");
        std::process::exit(2);
    }
    let code = if args.render && !args.run {
        render_only(&args)
    } else {
        run(&args)
    };
    std::process::exit(code);
}

fn connect_pg(args: &Args) -> Option<PgTarget> {
    match PgTarget::connect(&args.pg_host, args.pg_port, &args.pg_user, &args.pg_db) {
        Ok(t) => Some(t),
        Err(e) => {
            // No stand-in. A harness that fell back to something in-process would produce a
            // number that looks like a comparison and is not one.
            eprintln!(
                "bench: cannot reach PostgreSQL at {}:{} — {e}",
                args.pg_host, args.pg_port
            );
            eprintln!();
            eprintln!("  This harness measures against a real PostgreSQL and will not");
            eprintln!("  substitute anything for it. To start one:");
            eprintln!();
            eprintln!("    initdb -D /var/lib/pgdata -U bench --auth=trust");
            eprintln!(
                "    pg_ctl -D /var/lib/pgdata -o '-p {} -c listen_addresses=127.0.0.1' start",
                args.pg_port
            );
            eprintln!();
            None
        }
    }
}

fn calibrate(args: &Args, pg: &mut PgTarget) -> bool {
    eprintln!("== calibration ==");

    // What the storage charges for a durability barrier — **on the device PostgreSQL's WAL
    // is actually on**.
    //
    // This probed `temp_dir()`, which on a container is very often a different filesystem
    // from `$PGDATA`: a tmpfs `/tmp` reports an `fsync` cost of a microsecond and a ceiling
    // of a million commits per second, against which every real durable rate looks
    // implausibly low and the plausibility gate is measuring the wrong device. The server
    // is asked where its data directory is, so the calibration cannot be about a filesystem
    // the database never touches.
    let probe_dir = match std::env::var("BENCH_FSYNC_DIR") {
        Ok(d) => d,
        Err(_) => match pg.data_directory() {
            Some(d) => {
                eprintln!("  probing the device holding PGDATA: {d}");
                d
            }
            None => {
                eprintln!(
                    "  REFUSED: the server would not report `data_directory`, so the fsync \
                     probe would have to guess at a device. Set BENCH_FSYNC_DIR to the \
                     filesystem holding the WAL, or grant the bench user permission to run \
                     `show data_directory`."
                );
                return false;
            }
        },
    };
    let cost = match bank_bench::storage::fsync_cost(&probe_dir, 200) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("bench: cannot measure fsync cost in {probe_dir}: {e}");
            return false;
        }
    };
    eprintln!(
        "  storage: {:.1}µs per fsync — {} — ceiling {:.0} durable commits/s per connection",
        cost.per_call_us,
        cost.device_class(),
        cost.ceiling_per_second()
    );

    if let Err(e) = pg.prepare(args.accounts, args.rounds) {
        eprintln!("bench: could not prepare PostgreSQL: {e}");
        return false;
    }
    let durable = pg.is_durable();
    let sample = match workloads::durable(pg, args.accounts, 500, 0, 1) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("bench: calibration run failed: {e}");
            return false;
        }
    };
    let tps = sample.ops_per_second();
    eprintln!(
        "  PostgreSQL durable commit: {tps:.0} txn/s (durable={durable}), \
         {:.0}% of this device's ceiling",
        tps / cost.ceiling_per_second() * 100.0
    );
    eprintln!(
        "  for context: the published figure is {PUBLISHED_PG_TPS_PER_CORE} txn/s/core \
         ({:.1}× this measurement) — measured on ~3ms-fsync storage, against {:.0}µs here",
        tps / PUBLISHED_PG_TPS_PER_CORE,
        cost.per_call_us
    );

    if !durable {
        eprintln!(
            "  REFUSED: this server is not durable (synchronous_commit or fsync is off). \
             A durable-commit calibration against a non-durable server measures nothing."
        );
        return false;
    }
    if let Err(why) = bank_bench::storage::plausible(tps, cost) {
        eprintln!("  REFUSED: {why}");
        eprintln!("  No results will be written.");
        return false;
    }
    eprintln!("  PASSED: the commit rate is consistent with this device's fsync cost.");
    true
}

fn run(args: &Args) -> i32 {
    let Some(mut pg) = connect_pg(args) else {
        return 3;
    };
    if !calibrate(args, &mut pg) {
        return 4;
    }
    if args.calibrate && !args.run {
        return 0;
    }

    let mut samples: Vec<Sample> = Vec::new();
    let mut config: Vec<(String, Vec<(String, String)>)> = Vec::new();
    config.push(("postgres".into(), pg.configuration()));

    // The base each run starts from, per target. Compared against that target's first run and
    // **fatal on a difference**: a target carrying the previous run's writes into the next
    // run's scan is not being asked the same question twice, and the median of a drifting
    // series is a number with no referent.
    let mut base_of: BTreeMap<String, u64> = BTreeMap::new();

    // **The two targets are measured in one interleaved loop.**
    //
    // Every PostgreSQL run used to happen before any Nilestream run, so a host that drifted
    // over the few minutes of a session put all of the drift on one side of the ratio. It
    // produced an anti-correlated pair last cycle: PostgreSQL's analytical figure was the
    // highest of the session and Nilestream's the lowest, in the same run, from a change that
    // cannot slow an analytical query. Alternating makes the control mean what a control is
    // for — the two targets see the same minute.
    eprintln!("== interleaved: postgres, nilestream, per run ==");
    // Host the daemon on a thread of this process when asked to, so one command reproduces
    // the whole result and no run can silently measure a stale server holding the port.
    let mut hosted: Option<Hosted> = None;
    let bound = if args.host_nls {
        match std::net::TcpListener::bind(("127.0.0.1", args.nls_port)) {
            Ok(listener) => {
                eprintln!(
                    "  hosting the daemon on 127.0.0.1:{} — {} accounts x {} rounds, budget {}",
                    args.nls_port, args.accounts, args.rounds, args.nls_budget
                );
                if args.nls_budget >= args.accounts as usize {
                    eprintln!(
                        "  WARNING: the budget ({}) is at least the key count ({}), so nothing \
                         will ever be evicted and the point workload will measure the hit \
                         path only. That is a legitimate configuration and it is not a \
                         measurement of partial materialisation.",
                        args.nls_budget, args.accounts
                    );
                }
                // **A durable sink, because the `durable` row is meant to measure one.**
                // Without it the row was measured against an in-memory append and recorded
                // `durable=false`, which is honest in the CSV and useless as a comparison:
                // a durable PostgreSQL commit against a non-durable Nilestream one is not a
                // measurement of anything.
                let seg = std::path::Path::new(&args.out).join("nilestream-bench.seg");
                if let Some(parent) = seg.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::remove_file(&seg);
                let base = nilestream_server::rev_engine::RevEngine::seeded(
                    args.accounts,
                    args.rounds,
                    args.nls_budget,
                    proto_engine::ViewMode::Demand,
                    proto_engine::EvictionPolicy::Lru,
                );
                let base = match base.with_durable(&seg) {
                    Ok(e) => {
                        eprintln!("  durable sink at {} — SyncPolicy::Always", seg.display());
                        e
                    }
                    Err(e) => {
                        eprintln!(
                            "  REFUSED: cannot open a durable sink at {}: {e}. The `durable` \
                             row would measure a non-durable append.",
                            seg.display()
                        );
                        return 1;
                    }
                };
                let engine = std::sync::Arc::new(std::sync::Mutex::new(base));
                {
                    use nilestream_server::session::Serving;
                    eprintln!("  ledger frontier #{}", engine.lock().unwrap().frontier());
                }
                hosted = Some(Hosted {
                    engine: std::sync::Arc::clone(&engine),
                    seg,
                    accounts: args.accounts,
                    rounds: args.rounds,
                    budget: args.nls_budget,
                });
                let schema = nilestream_server::daemon::DEFAULT_SCHEMA.to_string();
                std::thread::spawn(move || {
                    nilestream_server::daemon::accept_loop(listener, schema, engine);
                });
                true
            }
            Err(e) => {
                eprintln!("  cannot bind 127.0.0.1:{}: {e}", args.nls_port);
                false
            }
        }
    } else {
        false
    };
    let _ = bound;

    let mut described = false;
    // The two targets' base sizes, compared once both have been prepared. A ratio between a
    // 20,000-row table and a 40,000-posting ledger is not a ratio, and nothing said so.
    let mut base_size: BTreeMap<String, u64> = BTreeMap::new();
    for run_no in 1..=args.runs {
        // ---- PostgreSQL's half of this run ----
        // Re-prepare between runs so each starts from the same table: an OLTP run that
        // appended two million rows would otherwise make the next run's analytical scan a
        // different measurement wearing the same name.
        if let Err(e) = pg.prepare(args.accounts, args.rounds) {
            eprintln!("bench: prepare failed on run {run_no}: {e}");
            return 5;
        }
        if let Err(why) = same_base(&mut pg, &mut base_of, run_no) {
            eprintln!("bench: {why}");
            return 9;
        }
        if let Some(n) = pg.base_rows() {
            base_size.insert("postgres".into(), n);
        }
        for s in run_all(&mut pg, args, run_no) {
            report(&s);
            samples.push(s);
        }

        // ---- Nilestream's half of the same run ----
        // **A fresh engine per run**, mirroring PostgreSQL's `prepare`. Without it every
        // Nilestream run began with the previous run's `oltp` and `durable` appends still in
        // the base — about 1,250 legs, six percent, compounding — while PostgreSQL started
        // from a rebuilt table. The analytical row fell monotonically across the five runs and
        // the committed median was the median of that drift.
        if let Some(h) = &hosted {
            if let Err(e) = h.reseed() {
                eprintln!("bench: could not re-seed the hosted engine for run {run_no}: {e}");
                return 5;
            }
        }
        // A fresh connection too: a session tracks the anchor it has observed, and a session
        // that outlived a re-seeded ledger would be reading at an anchor from a history that
        // no longer exists.
        let mut nls = match NilestreamTarget::connect("127.0.0.1", args.nls_port) {
            Ok(t) => t,
            Err(e) => {
                // Recorded as `NOT RUN` with the reason, not omitted. A missing row invites a
                // reader to assume the number was unremarkable.
                eprintln!("  unreachable on 127.0.0.1:{}: {e}", args.nls_port);
                for w in ["oltp", "analytical", "point", "durable"] {
                    samples.push(workloads::skipped(
                        w,
                        "nilestream",
                        run_no,
                        format!(
                            "nilestreamd was not reachable on 127.0.0.1:{}: {e}",
                            args.nls_port
                        ),
                    ));
                }
                continue;
            }
        };
        if !described {
            config.push(("nilestream".into(), nls.configuration()));
            described = true;
        }
        if let Err(e) = nls.prepare(args.accounts, args.rounds) {
            eprintln!("  unavailable: {e}");
            for w in ["oltp", "analytical", "point", "durable"] {
                samples.push(workloads::skipped(w, "nilestream", run_no, format!("{e}")));
            }
            continue;
        }
        if let Err(why) = same_base(&mut nls, &mut base_of, run_no) {
            eprintln!("bench: {why}");
            return 9;
        }
        if let Some(n) = nls.base_rows() {
            base_size.insert("nilestream".into(), n);
        }
        // **The two targets must hold the same base, and the run says so or does not run.**
        //
        // Not a warning: an analytical ratio computed over two different row counts is a
        // number with no meaning, and it was published for two cycles. Every workload of this
        // run becomes `NOT RUN` with the two counts in the reason, so the difference appears
        // in the CSV and in the rendered table rather than in a log nobody kept.
        if let (Some(p), Some(n)) = (base_size.get("postgres"), base_size.get("nilestream")) {
            if p != n {
                let why = format!(
                    "the two targets do not hold the same base: postgres has {p} rows and \
                     nilestream has {n}. A ratio between different problems is not a ratio. \
                     Both are seeded with `--rounds` conserved pairs per account; check that \
                     nilestreamd was started with the same `--rounds` as this harness ({})",
                    args.rounds
                );
                eprintln!("bench: {why}");
                for w in ["oltp", "analytical", "point", "durable"] {
                    samples.push(workloads::skipped(w, "postgres", run_no, why.clone()));
                    samples.push(workloads::skipped(w, "nilestream", run_no, why.clone()));
                }
                continue;
            }
        }
        for s in run_all(&mut nls, args, run_no) {
            report(&s);
            samples.push(s);
        }
    }
    if let (Some(p), Some(n)) = (base_size.get("postgres"), base_size.get("nilestream")) {
        eprintln!(
            "  base: postgres {p} rows, nilestream {n} rows ({} per account x {} accounts)",
            args.rounds * 2,
            args.accounts
        );
    }

    let _ = pg.teardown();
    if let Err(e) = write_all(args, &samples, &config) {
        eprintln!("bench: writing results failed: {e}");
        return 6;
    }
    // **A `NOT RUN` row is a failed run, and the process says so.** It used to exit 0: a CI
    // job or a script driving this harness could not tell a complete measurement from one
    // where a target was unreachable for every run.
    let missing = samples.iter().filter(|s| s.not_run.is_some()).count();
    if missing > 0 {
        eprintln!(
            "\nbench: {missing} of {} rows are NOT RUN; the reasons are in the CSVs and in the \
             rendered table",
            samples.len()
        );
        return 8;
    }
    0
}

/// A hosted daemon, and the pieces needed to put its engine back the way it started.
struct Hosted {
    engine: std::sync::Arc<std::sync::Mutex<nilestream_server::rev_engine::RevEngine>>,
    seg: std::path::PathBuf,
    accounts: i64,
    rounds: u32,
    budget: usize,
}

impl Hosted {
    /// Replace the engine with a freshly seeded one on a fresh segment.
    fn reseed(&self) -> Result<(), String> {
        use nilestream_server::rev_engine::RevEngine;
        let mut guard = self.engine.lock().map_err(|e| e.to_string())?;
        // Drop the old engine — and with it the file its durable sink holds — *before*
        // removing the segment and opening a new one. Two sequencers over one path would
        // recover each other's records and the second run would start from the first's
        // history, which is the drift this method exists to remove.
        *guard = RevEngine::seeded(
            1,
            1,
            1,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        );
        let _ = std::fs::remove_file(&self.seg);
        *guard = RevEngine::seeded(
            self.accounts,
            self.rounds,
            self.budget,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_durable(&self.seg)
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Check that this run starts from the same base as this target's first run.
fn same_base(
    t: &mut dyn Target,
    seen: &mut BTreeMap<String, u64>,
    run_no: u32,
) -> Result<(), String> {
    let Some(now) = t.base_marker() else {
        return Ok(());
    };
    let name = t.name().to_string();
    match seen.get(&name) {
        None => {
            eprintln!("  {name}: base marker {now} at run {run_no}");
            seen.insert(name, now);
            Ok(())
        }
        Some(first) => match bank_bench::publish::base_drift(&name, *first, now, run_no) {
            None => Ok(()),
            Some(why) => Err(why),
        },
    }
}

fn report(s: &Sample) {
    match &s.not_run {
        Some(why) => eprintln!("  {} run {} — NOT RUN: {why}", s.workload, s.run),
        None => eprintln!(
            "  {} run {} — {:.0} ops/s, p99 {:.0}µs",
            s.workload,
            s.run,
            s.ops_per_second(),
            s.p99.as_nanos() as f64 / 1000.0
        ),
    }
}

fn run_all(t: &mut dyn Target, args: &Args, run_no: u32) -> Vec<Sample> {
    let seed = 0xB0A7 ^ (run_no as u64);
    let name = t.name().to_string();
    let mut out = Vec::new();
    // **A workload that fails is a `NOT RUN` row, not a line on stderr.** It used to be
    // `eprintln!("  a workload failed: {e}")` and a dropped result: the row simply vanished
    // from the CSV, the renderer had one fewer run to take a median over, and the table said
    // nothing at all about the failure. A benchmark that can lose a measurement silently is
    // a benchmark whose completeness a reader cannot check.
    let point = workloads::point(t, args.accounts, args.operations, run_no, seed);
    push(&mut out, "point", &name, run_no, point.map(|s| vec![s]));

    // **Twenty-five executions of the common set per run, not six.**
    //
    // `(operations / 20).max(6)` gave six executions of a four-statement set at the committed
    // `--operations 500`, so a per-statement median rested on thirty numbers across five runs
    // and moved by tens of percent between sessions. A quarter of `operations`, floored at
    // twenty-five, costs about five seconds more per recipe run and is what makes a
    // per-statement figure worth reading.
    let analytical = workloads::analytical(t, args.accounts, (args.operations / 4).max(25), run_no);
    push(&mut out, "analytical", &name, run_no, analytical);

    let oltp = workloads::oltp(t, args.accounts, args.operations, run_no, seed);
    push(&mut out, "oltp", &name, run_no, oltp.map(|s| vec![s]));

    let durable = workloads::durable(
        t,
        args.accounts,
        (args.operations / 4).max(50),
        run_no,
        seed,
    );
    push(&mut out, "durable", &name, run_no, durable.map(|s| vec![s]));
    out
}

fn push(
    out: &mut Vec<Sample>,
    workload: &str,
    target: &str,
    run_no: u32,
    r: Result<Vec<Sample>, bank_bench::wire::WireError>,
) {
    match r {
        Ok(s) => out.extend(s),
        Err(e) => out.push(workloads::skipped(
            workload,
            target,
            run_no,
            format!("the workload failed part-way through: {e}"),
        )),
    }
}

fn write_all(
    args: &Args,
    samples: &[Sample],
    config: &[(String, Vec<(String, String)>)],
) -> std::io::Result<()> {
    std::fs::create_dir_all(&args.out)?;
    // One file per contract workload, and one for the per-statement rows: a workload column
    // of `analytical:group_by_acct` must not become a file name with a colon in it.
    let mut by_file: BTreeMap<&str, Vec<&Sample>> = BTreeMap::new();
    for s in samples {
        let file = match s.workload.as_str() {
            "oltp" | "analytical" | "point" | "durable" => s.workload.as_str(),
            _ => "analytical-statements",
        };
        by_file.entry(file).or_default().push(s);
    }
    for (w, rows) in &by_file {
        let mut f = std::fs::File::create(format!("{}/{w}.csv", args.out))?;
        writeln!(f, "{CSV_HEADER}")?;
        for s in rows {
            writeln!(f, "{}", s.to_csv())?;
        }
    }

    let doc = document(samples, config, args);
    let where_to = bank_bench::publish::destinations(std::path::Path::new(&args.out), args.publish);
    for path in &where_to {
        std::fs::write(path, &doc)?;
    }
    eprintln!(
        "\nwrote {} and {}/*.csv{}",
        where_to
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        args.out,
        if args.publish {
            ""
        } else {
            " — pass `--publish` to overwrite the committed results/E16-wallclock.md"
        }
    );
    Ok(())
}

fn document(samples: &[Sample], config: &[(String, Vec<(String, String)>)], args: &Args) -> String {
    let gaps: BTreeMap<String, String> = ["oltp", "analytical", "point", "durable"]
        .iter()
        .filter_map(|w| nilestream_gap(w).map(|r| (w.to_string(), r)))
        .collect();
    let table = &render::contract_table(samples, &gaps);
    let mut s = String::new();
    s.push_str("# E16 — The performance contract, measured\n\n");
    s.push_str(
        "**Generated by `bench --run`. Nothing in this file is typed in by hand.**\n\n\
         `SPEC-ENGINE.md` Part 0 states four targets relative to PostgreSQL. Until this \
         experiment they were predictions in the typography of results: every measurement in \
         `results/` reported counted work inside `proto-engine`, and the single wall-clock \
         table in the thesis (§9.4.4) was in-memory, single-threaded, and compared to \
         nothing. This is the first wall-clock comparison against the baseline the \
         specification names.\n\n",
    );
    s.push_str(table);
    s.push('\n');
    s.push_str(&render::spread_table(samples));
    s.push('\n');
    s.push_str(&render::statement_table(samples));
    s.push_str("\n## How it was run\n\n");
    s.push_str(&format!(
        "* Accounts: {}\n* Operations per run: {}\n* Runs per workload: {} (medians reported)\n\
         * Rounds per account: {} (conserved pairs, **on both targets**)\n\
         * Base rows per target: {} ({} legs per account x {} accounts)\n\
         * Targets are **interleaved**: PostgreSQL run 1, Nilestream run 1, PostgreSQL run 2, \
         and so on, so host drift over a session lands on both sides of the ratio rather than \
         one.\n\
         * Both targets are driven **over the PostgreSQL wire protocol through the same \
         client** (`bank-bench::wire`), so neither side is spared the protocol cost the \
         other pays.\n* Access pattern is seeded and reproducible (SplitMix64), 90% of point \
         lookups landing in the hottest 1% of accounts.\n\n",
        args.accounts,
        args.operations,
        args.runs,
        args.rounds,
        args.accounts * 2 * args.rounds as i64,
        args.rounds * 2,
        args.accounts
    ));
    for (name, settings) in config {
        s.push_str(&format!("### {name}\n\n"));
        for (k, v) in settings {
            s.push_str(&format!("* `{k}` = `{v}`\n"));
        }
        s.push('\n');
    }
    s.push_str(
        "## What the `point` row is\n\n\
         **An engine result.** The row is served by `nilestream-server::rev_engine`: a partial \
         view over an immutable, hash-chained ledger, answering an anchored read, \
         reconstructing on a miss. Not a hash map — that was the first version of this \
         experiment, and the document had to spend two paragraphs saying the number meant \
         nothing.\n\n\
         The miss rate is reported with it and belongs with it. A parity result at a 0% miss \
         rate says a warm view is fast; one at a nonzero rate says *reconstruction* is, which \
         is the claim the thesis actually makes — so the rate is a column of the CSV and is \
         rendered from it below, not a range typed into this sentence. It was one: this \
         paragraph named a range of miss rates in the high single digits while the harness \
         configured a budget larger than the key space, under which nothing is ever evicted \
         and the true rate after warm-up is zero.\n\n\
         Two things it still does not establish. The engine is in-memory and single-threaded, \
         so this is not a durability or a concurrency result. And PostgreSQL is doing different \
         work — an index scan and an aggregation, against a maintained view plus occasional \
         reconstruction — which is the *point* of partial materialisation rather than an unfair \
         comparison, but it means the row says \"a REV serves a point lookup as fast as an \
         indexed aggregate\" and not \"Nilestream is faster than PostgreSQL\".\n\n\
         ## The defects this experiment found\n\n\
         **A 640× stall in the wire path.** The first run measured Nilestream at 23 point \
         lookups per second against PostgreSQL's 13,600. Neither the engine nor the compiler \
         (per-query compilation measures 0.02ms, so the whole \"compile every query\" concern \
         is 0.5% of the budget): `pg_wire::write_all` issued one socket write per protocol \
         message, so a four-message reply was held by Nagle's algorithm pending the peer's \
         delayed-ACK timer. 43ms per query is that timer wearing a database's clothes. One \
         buffer, one write, `TCP_NODELAY` — and the same workload reached ~14,700/s.\n\n\
         **A calibration that agreed with a broken harness.** The gate first compared \
         PostgreSQL against a published 333 txn/s/core and fired at 16×. The gate was right \
         and the figure was wrong: `fsync` costs 1.6–12.4µs with power-loss protection and \
         891–2974µs without, so 333 txn/s is a property of PostgreSQL *on a ~3ms device*. This \
         machine syncs in 93µs, where holding it to 333 would have meant the harness was \
         broken — and the published number would have concealed that by agreeing with it. See \
         `storage.rs`.\n\n\
         **A measurement written where nothing read it.** The Nilestream half runs under \
         `cargo test`, whose working directory is the *package* rather than the workspace. A \
         relative path put a second `results/` tree under `crates/bank-bench/`, and the table \
         went on reporting a missing row while a good measurement sat ten directories \
         away.\n\n\
         **Three rows the harness refused to run against an engine that could run them.** \
         `oltp` and `durable` were refused because \"nilestreamd exposes no write surface \
         over the wire\"; `analytical` because \"a scan-and-group-by surface is not \
         exposed\". Both reasons were true when they were written and had stopped being \
         true, so three quarters of this table reported a gap in the engine that was \
         actually a gap in the harness's beliefs about it.\n\n\
         **A budget that could not bind.** The Nilestream engine was hosted with room for \
         ten times the key space, so after warm-up nothing was ever evicted: the `point` row \
         measured the hit path exclusively while being presented as a measurement of partial \
         materialisation. The budget is now a quarter of the key space and the miss rate is \
         a column of the CSV.\n\n\
         **A full scan behind every point lookup.** When the server began evaluating the \
         compiled circuit rather than answering from a hard-coded fold, the `point` row fell \
         from parity to 68 operations per second — the cost of materialising twenty thousand \
         postings per query. The fix is predicate pushdown into the source scan through the \
         anchor index, which cannot change what the circuit denotes and is held to that by a \
         test.\n\n\
         None could have been found by counting operations. In each case the engine did the \
         right amount of work, in the right order, and the number was still wrong.\n\n\
         ## What this does not measure\n\n\
         The OLTP and durable rows for PostgreSQL are a *baseline*, not a competition: they \
         establish what the comparison is against. A row that cannot be run is reported with \
         the reason rather than omitted, and filling one by measuring something else under \
         the same name is the specific failure this file exists to avoid.\n\n\
         **The analytical row is a common-set ratio.** It used to compare five PostgreSQL \
         statements against three Nilestream ones, round-robined into one composite: \
         PostgreSQL's set contained its cheapest statement (`count(*)`) and Nilestream's did \
         not, and one pair — `group by acct order by sum(amt) desc limit 10` against a plain \
         `group by acct` — was two different operations averaged as though it were one. The \
         statement set is now paired by operation in `workloads::ANALYTICAL_STATEMENTS`; the \
         ratio is computed from the statements both targets run, the rest are still measured \
         on PostgreSQL so their cost is on the record, and the per-statement table above says \
         which operator the distance is in. Widening the fragment during a benchmark would \
         be tuning the artifact to the measurement; averaging over a statement one side \
         cannot express is worse, because it looks like a comparison.\n",
    );
    s
}

fn render_only(args: &Args) -> i32 {
    let mut samples = Vec::new();
    for w in ["oltp", "analytical", "point", "durable"] {
        let path = format!("{}/{w}.csv", args.out);
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("bench: {path} is missing; run `bench --run` first");
            return 7;
        };
        for line in text.lines().skip(1) {
            if let Some(s) = parse_csv_line(line) {
                samples.push(s);
            }
        }
    }
    // The per-statement rows are optional: a CSV set produced before the statement table
    // existed still renders its contract table, and says nothing it cannot support.
    if let Ok(text) = std::fs::read_to_string(format!("{}/analytical-statements.csv", args.out)) {
        for line in text.lines().skip(1) {
            if let Some(s) = parse_csv_line(line) {
                samples.push(s);
            }
        }
    }
    let gaps: BTreeMap<String, String> = ["oltp", "analytical", "point", "durable"]
        .iter()
        .filter_map(|w| nilestream_gap(w).map(|r| (w.to_string(), r)))
        .collect();
    print!("{}", render::contract_table(&samples, &gaps));
    println!();
    print!("{}", render::spread_table(&samples));
    println!();
    print!("{}", render::statement_table(&samples));
    0
}

fn parse_csv_line(line: &str) -> Option<Sample> {
    let f: Vec<&str> = line.splitn(12, ',').collect();
    if f.len() < 12 {
        return None;
    }
    Some(Sample {
        workload: f[0].into(),
        target: f[1].into(),
        run: f[2].parse().ok()?,
        operations: f[3].parse().ok()?,
        wall: std::time::Duration::from_secs_f64(f[4].parse::<f64>().ok()? / 1000.0),
        p50: std::time::Duration::from_nanos((f[5].parse::<f64>().ok()? * 1000.0) as u64),
        p99: std::time::Duration::from_nanos((f[6].parse::<f64>().ok()? * 1000.0) as u64),
        durable: f[8] == "true",
        not_run: if f[9].is_empty() {
            None
        } else {
            Some(f[9].to_string())
        },
        // The recorded protocol, not a constant: reading a CSV back must report what that
        // run used rather than what this build would use.
        protocol_path: match f[10] {
            "extended" => "extended",
            "none" => "none",
            _ => "simple",
        },
        miss_rate: f[11].trim().parse::<f64>().ok(),
    })
}
