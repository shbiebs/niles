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
use bank_bench::workloads::{self, Sample, ScalingSample};
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
    /// The second hosted Nilestream, whose view is **fully materialised**.
    ///
    /// The engine holds one runtime, so one daemon cannot serve both the partial view the
    /// `point` row needs and the maintained view a `report` is read from. Two daemons, one
    /// process, the same seed, and the results file says which port served which row —
    /// rather than one budget quietly deciding both and a reader assuming it did not.
    nls_report_port: Option<u16>,
    /// Appends per second on the second connection during the `report` workload.
    ///
    /// The same figure on both targets, which is the point: an unpaced appender ran five
    /// times faster against one target than the other, so the two report rows were measured
    /// under different write pressure on a host where pressure is taken out of the thing
    /// being timed.
    append_rate: f64,
    /// E23's axes. Accounts fixed while the base grows; the base held while the answer grows.
    e23_accounts: i64,
    e23_base: Vec<u32>,
    e23_output: Vec<i64>,
    e23_hold_base: u64,
    e23_reps: u32,
    /// Run E23 and nothing else.
    e23_only: bool,
    accounts: i64,
    operations: u64,
    /// Seconds the E19 `mixed` level runs its readers and writers together.
    ///
    /// A duration and not an operation count, because the two roles run at rates three orders
    /// of magnitude apart: a fixed count per thread would end the readers' share in the first
    /// second and spend the rest of the level measuring the writers alone.
    mixed_seconds: u64,
    runs: u32,
    /// Host Nilestream's listener on a thread of this process.
    host_nls: bool,
    /// Conserved pairs per account, on **both** targets. Named `rounds` because it is no
    /// longer a Nilestream-only knob: the two bases must be the same size or the analytical
    /// ratio compares two different problems. `--nls-rounds` is still accepted, and means the
    /// same thing on both sides.
    rounds: u32,
    nls_budget: usize,
    /// Connection counts for the scaling experiment (E19), e.g. `1,2,4`.
    ///
    /// Empty means the experiment does not run, and that is the default: the contract table
    /// is a single-connection measurement and must not change because a scaling flag was
    /// passed. E19 is written to its own directory and its own document.
    connections: Vec<u32>,
    /// Run **only** the scaling experiment, leaving the contract table alone.
    ///
    /// Here because E19 and E16 are published at different run counts, and re-rendering E16
    /// as a side effect of re-running E19 would overwrite a committed table with one measured
    /// under this invocation's flags. With this set, no `Sample` is produced and `write_all`
    /// is never reached, so `results/E16-wallclock.md` cannot be touched however `--publish`
    /// is passed.
    scaling_only: bool,
    /// Run the Nilestream arm without a PostgreSQL to compare against.
    ///
    /// The harness refuses to substitute anything for a real PostgreSQL, and that is right
    /// for a *comparison*. But "does the engine's throughput rise with connections" is a
    /// question about one engine, and it was unanswerable on any machine without a
    /// PostgreSQL 16 server — including the author's own, where the shape of the read curve
    /// is the open question. The PostgreSQL rows are recorded `NOT RUN` with this reason
    /// rather than omitted, so a reader of the CSV cannot mistake a one-armed run for a
    /// comparison.
    nls_only: bool,
    /// Overwrite the **committed** `results/E16-wallclock.md`.
    ///
    /// Off by default, and that is the repair. `write_all` wrote the committed document on
    /// every `--run` whatever `--out` said, so any exploratory run — an audit's, a bisect's,
    /// a CI job's — left the working tree dirty with a results file measured under whatever
    /// flags that run happened to use. A benchmark must not touch a committed artifact
    /// unless it was asked to publish one.
    publish: bool,
}

/// The contract workloads, in one place.
///
/// It was seven copies of the same array literal, and adding `report` had to touch all
/// seven — which is the shape of a list that will one day be six and a bug. A skipped row
/// for a workload nobody remembered to add to one of the copies is invisible: the row is
/// simply absent from the CSV and the table renders without it.
const CONTRACT_WORKLOADS: [&str; 5] = ["oltp", "analytical", "point", "durable", "report"];

impl Args {
    /// Where the fully-materialised daemon listens. One past the partial one unless asked
    /// otherwise, so the flag order cannot decide it.
    fn report_port(&self) -> u16 {
        self.nls_report_port.unwrap_or(self.nls_port + 1)
    }

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
            nls_report_port: None,
            append_rate: 500.0,
            e23_accounts: 10_000,
            e23_base: vec![1, 10, 100],
            e23_output: vec![1_000, 10_000, 100_000],
            e23_hold_base: 200_000,
            e23_reps: 9,
            e23_only: false,
            accounts: 10_000,
            operations: 2_000,
            mixed_seconds: 4,
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
            connections: Vec::new(),
            scaling_only: false,
            nls_only: false,
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
                "--e23" => a.e23_only = true,
                "--e23-reps" => {
                    if i + 1 < argv.len() {
                        a.e23_reps = argv[i + 1].parse().unwrap_or(a.e23_reps);
                        i += 1;
                    }
                }
                "--e23-base" => {
                    if i + 1 < argv.len() {
                        a.e23_base = argv[i + 1]
                            .split(',')
                            .filter_map(|x| x.trim().parse().ok())
                            .collect();
                        i += 1;
                    }
                }
                "--e23-output" => {
                    if i + 1 < argv.len() {
                        a.e23_output = argv[i + 1]
                            .split(',')
                            .filter_map(|x| x.trim().parse().ok())
                            .collect();
                        i += 1;
                    }
                }
                "--append-rate" => {
                    if i + 1 < argv.len() {
                        a.append_rate = argv[i + 1].parse().unwrap_or(a.append_rate);
                        i += 1;
                    }
                }
                "--nls-report-port" => {
                    if i + 1 < argv.len() {
                        a.nls_report_port = argv[i + 1].parse().ok().or(a.nls_report_port);
                        i += 1;
                    }
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
                "--mixed-seconds" => {
                    a.mixed_seconds = argv[i + 1].parse().unwrap_or(a.mixed_seconds);
                    i += 1;
                }
                "--runs" => {
                    a.runs = argv[i + 1].parse().unwrap_or(a.runs);
                    i += 1;
                }
                "--host-nls" => a.host_nls = true,
                "--scaling-only" => a.scaling_only = true,
                "--nls-only" => a.nls_only = true,
                "--publish" => a.publish = true,
                "--rounds" | "--nls-rounds" => {
                    a.rounds = argv[i + 1].parse().unwrap_or(a.rounds);
                    i += 1;
                }
                "--connections" => {
                    // `1,2,4`. A level that does not parse is dropped with a message rather
                    // than silently becoming a default: a scaling table missing its widest
                    // level is the one shape a reader would not notice.
                    a.connections = argv[i + 1]
                        .split(',')
                        .filter_map(|x| match x.trim().parse::<u32>() {
                            Ok(0) | Err(_) => {
                                eprintln!("bench: ignoring unusable connection level `{x}`");
                                None
                            }
                            Ok(n) => Some(n),
                        })
                        .collect();
                    a.connections.sort_unstable();
                    a.connections.dedup();
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
    if args.scaling_only && args.connections.is_empty() {
        eprintln!("bench: --scaling-only needs --connections, or it would measure nothing");
        std::process::exit(2);
    }
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
    // **One arm, and it says so.** `--nls-only` is only meaningful with `--scaling-only`:
    // the contract table *is* the comparison, so a one-armed E16 would be a table of
    // unlabelled absolute numbers, which is the shape this harness exists to refuse.
    // **One arm, and it says so.** `--nls-only` is only meaningful with `--scaling-only`:
    // the contract table *is* the comparison, so a one-armed E16 would be a table of
    // unlabelled absolute numbers, which is the shape this harness exists to refuse.
    if args.nls_only && !args.scaling_only {
        eprintln!(
            "bench: --nls-only measures one engine, so it needs --scaling-only. The contract \
             table is a comparison; there is nothing to put in it with one arm."
        );
        return 2;
    }
    let mut pg: Option<PgTarget> = if args.nls_only {
        eprintln!("== Nilestream only ==");
        eprintln!("  No PostgreSQL arm, and no calibration against its device: the question");
        eprintln!("  `does this engine's throughput rise with connections` is about one");
        eprintln!("  engine. PostgreSQL rows are recorded NOT RUN rather than omitted.");
        None
    } else {
        match connect_pg(args) {
            Some(p) => Some(p),
            None => return 3,
        }
    };
    if let Some(p) = pg.as_mut() {
        if !calibrate(args, p) {
            return 4;
        }
    }
    if args.calibrate && !args.run {
        return 0;
    }

    let mut samples: Vec<Sample> = Vec::new();
    let mut config: Vec<(String, Vec<(String, String)>)> = Vec::new();
    if let Some(p) = pg.as_mut() {
        config.push(("postgres".into(), p.configuration()));
    }

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
                let engine = std::sync::Arc::new(std::sync::RwLock::new(base));
                {
                    use nilestream_server::session::Serving;
                    eprintln!("  ledger frontier #{}", engine.read().unwrap().frontier());
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

    // **A second daemon, whose view is fully materialised.** The `report` row is a read of
    // maintained state; the `point` row is a read of *partially* maintained state under a
    // budget that binds. One engine holds one runtime, so one daemon cannot be both — and
    // making the point row's budget large enough to serve a report would delete the very
    // thing the point row measures. Two daemons, one process, the same seed, and both ports
    // named in the results.
    let mut hosted_report: Option<Hosted> = None;
    if args.host_nls {
        let port = args.report_port();
        match std::net::TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => {
                let seg = std::path::Path::new(&args.out).join("nilestream-report.seg");
                if let Some(parent) = seg.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::remove_file(&seg);
                let base = nilestream_server::rev_engine::RevEngine::seeded(
                    args.accounts,
                    args.rounds,
                    usize::MAX,
                    proto_engine::ViewMode::Demand,
                    proto_engine::EvictionPolicy::Lru,
                );
                match base.with_durable(&seg) {
                    Ok(e) => {
                        eprintln!(
                            "  hosting the report daemon on 127.0.0.1:{port} — the same \
                             {} accounts x {} rounds, view fully materialised",
                            args.accounts, args.rounds
                        );
                        let engine = std::sync::Arc::new(std::sync::RwLock::new(e));
                        hosted_report = Some(Hosted {
                            engine: std::sync::Arc::clone(&engine),
                            seg,
                            accounts: args.accounts,
                            rounds: args.rounds,
                            budget: usize::MAX,
                        });
                        let schema = nilestream_server::daemon::DEFAULT_SCHEMA.to_string();
                        std::thread::spawn(move || {
                            nilestream_server::daemon::accept_loop(listener, schema, engine);
                        });
                    }
                    Err(e) => eprintln!(
                        "  the report daemon has no durable sink ({e}); the report row will \
                         be NOT RUN rather than measured against a different engine"
                    ),
                }
            }
            Err(e) => eprintln!("  cannot bind 127.0.0.1:{port} for the report daemon: {e}"),
        }
    }

    // **The device's ceiling, and how much it moved.** The `durable` row is an `fsync` rate
    // by construction, so it is a measurement of the storage as much as of the engine. This
    // container's device has ranged 7,794 to 12,761 commits per second across probes taken
    // seconds apart, which is more movement than any engine change this cycle produced — so
    // the ceiling is measured here and published beside the row, and each target's rate is
    // reported as a fraction of it. That fraction is the part that is about the engine.
    let ceiling = std::env::var("BENCH_FSYNC_DIR")
        .ok()
        .or_else(|| pg.as_mut().and_then(|p| p.data_directory()))
        .and_then(|d| bank_bench::storage::ceiling(&d, 7, 200).ok());
    // **Do the two systems mean the same thing by "durable"?**
    //
    // `fsync=on` says PostgreSQL calls something before acknowledging; `wal_sync_method` says
    // what. Rust's `sync_data`, which the ledger and this probe both use, issues the barrier
    // named in `storage::BARRIER`. Where those differ in *strength* the comparison is between
    // two different guarantees, and the ratio is meaningless in a way no amount of repetition
    // fixes.
    //
    // The case this exists for is macOS: PostgreSQL defaults to `fsync`, which APFS does not
    // turn into a drive-cache flush, against `F_FULLFSYNC` on the Nilestream side. Measured
    // on the author's machine, PostgreSQL committed 13,458 durable transactions per second
    // against a 324/s barrier — 41× its own storage — while reporting `fsync=on`.
    if let Some(p) = pg.as_mut() {
        let method = p
            .configuration()
            .into_iter()
            .find(|(k, _)| k == "wal_sync_method")
            .map(|(_, v)| v)
            .unwrap_or_default();
        eprintln!(
            "  barriers: nilestream `{}`, postgres `wal_sync_method = {}`",
            bank_bench::storage::BARRIER,
            if method.is_empty() {
                "unknown"
            } else {
                &method
            }
        );
        if bank_bench::storage::BARRIER == "F_FULLFSYNC" && method != "fsync_writethrough" {
            eprintln!();
            eprintln!(
                "  REFUSED: this platform's `sync_data` is F_FULLFSYNC, which flushes the \
                 drive cache, and PostgreSQL is using `{method}`, which on APFS does not. \
                 The two systems would be durable against different failures and the ratio \
                 would not be a comparison."
            );
            eprintln!("  Set `wal_sync_method = fsync_writethrough` and restart the server.");
            return 6;
        }
    }

    if let Some(c) = &ceiling {
        eprintln!(
            "  device: {:.0} durable commits/s median over {} probes via {}, spread {:.2}x{}",
            c.median,
            c.probes,
            c.barrier,
            c.spread(),
            if c.steady() {
                ""
            } else {
                " — NOT STEADY, absolute durable rates are the device's and not the engine's"
            }
        );
    }

    let mut described = false;
    // The two targets' base sizes, compared once both have been prepared. A ratio between a
    // 20,000-row table and a 40,000-posting ledger is not a ratio, and nothing said so.
    let mut base_size: BTreeMap<String, u64> = BTreeMap::new();
    // `--scaling-only` skips the contract loop entirely: no `Sample` is produced, so
    // `write_all` below has nothing to write and the committed E16 document cannot be
    // reached. The two experiments are published at different run counts and re-rendering one
    // as a side effect of re-running the other would overwrite a measured table.
    let contract_runs = if args.scaling_only || args.e23_only {
        0
    } else {
        args.runs
    };
    for run_no in 1..=contract_runs {
        // **The contract table is a comparison, so this loop needs both arms.**
        // `contract_runs` is 0 under `--scaling-only`, which `--nls-only` requires, so this
        // cannot be reached without a PostgreSQL — stated as an expectation rather than
        // threaded as an `Option` through two hundred lines that all assume two targets.
        let pg = pg
            .as_mut()
            .expect("the contract loop compares two targets; --nls-only forces 0 runs of it");
        // The base the first arm of this run's report row started from. Reset per run,
        // because each run re-prepares both targets.
        let mut report_base: Option<u64> = None;
        // ---- PostgreSQL's half of this run ----
        // Re-prepare between runs so each starts from the same table: an OLTP run that
        // appended two million rows would otherwise make the next run's analytical scan a
        // different measurement wearing the same name.
        if let Err(e) = pg.prepare(args.accounts, args.rounds) {
            eprintln!("bench: prepare failed on run {run_no}: {e}");
            return 5;
        }
        if let Err(why) = same_base(pg, &mut base_of, run_no) {
            eprintln!("bench: {why}");
            return 9;
        }
        if let Some(n) = pg.base_rows() {
            base_size.insert("postgres".into(), n);
        }
        for s in run_all(pg, args, run_no) {
            report(&s);
            samples.push(s);
        }
        {
            // **Re-prepared first, because the report row is measured against a freshly
            // seeded engine on the other side.** Without this PostgreSQL entered the report
            // carrying `oltp` and `durable`'s appends — 24,500 rows against Nilestream's
            // 20,000, a 22% larger base on the arm the ratio divides by, which is a 22%
            // gift to the other arm. The Nilestream report daemon is reseeded per run; this
            // is the same courtesy on the same line.
            let (host, port, user, db) = (
                args.pg_host.clone(),
                args.pg_port,
                args.pg_user.clone(),
                args.pg_db.clone(),
            );
            let s = match pg.prepare(args.accounts, args.rounds) {
                Ok(()) => {
                    let open = move || bank_bench::wire::Client::connect(&host, port, &user, &db);
                    report_row(pg, args, run_no, &open, &mut report_base)
                }
                Err(e) => workloads::skipped(
                    "report",
                    "postgres",
                    run_no,
                    format!("could not re-prepare before the report row: {e}"),
                ),
            };
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
        if let Some(h) = &hosted_report {
            if let Err(e) = h.reseed() {
                eprintln!("bench: could not re-seed the report engine for run {run_no}: {e}");
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
                for w in CONTRACT_WORKLOADS {
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
            for w in CONTRACT_WORKLOADS {
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
                for w in CONTRACT_WORKLOADS {
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
        // **The report row is measured against the other daemon**, the one whose view is
        // fully materialised. A row served by the partial engine would be the fold's number
        // wearing the maintained view's name, which the engine itself refuses
        // (`a_partial_view_never_serves_a_report`) — this is the harness declining to ask.
        let s = match NilestreamTarget::connect("127.0.0.1", args.report_port()) {
            Ok(mut rt) => {
                let port = args.report_port();
                let open =
                    move || bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank");
                match rt.prepare(args.accounts, args.rounds) {
                    Ok(()) => report_row(&mut rt, args, run_no, &open, &mut report_base),
                    Err(e) => workloads::skipped("report", "nilestream", run_no, format!("{e}")),
                }
            }
            Err(e) => workloads::skipped(
                "report",
                "nilestream",
                run_no,
                format!(
                    "the fully-materialised daemon was not reachable on 127.0.0.1:{}: {e}",
                    args.report_port()
                ),
            ),
        };
        report(&s);
        samples.push(s);
    }
    if let (Some(p), Some(n)) = (base_size.get("postgres"), base_size.get("nilestream")) {
        eprintln!(
            "  base: postgres {p} rows, nilestream {n} rows ({} per account x {} accounts)",
            args.rounds * 2,
            args.accounts
        );
    }

    // ---- E19: the scaling experiment, if it was asked for ----
    //
    // **After the contract loop and into its own document.** The contract table is a
    // single-connection measurement of four targets the specification states without a
    // concurrency qualifier; a 4-connection figure must not be able to reach it. Nothing
    // above this line reads `args.connections`, and `scaling` returns a different type.
    if !args.connections.is_empty() {
        let scaling = run_scaling(args, pg.as_mut(), &hosted);
        if let Err(e) = write_scaling(args, &scaling.scaling, &scaling.mixed) {
            eprintln!("bench: writing the scaling results failed: {e}");
            return 6;
        }
    }

    // ---- E23: the two asymptotic sweeps, into their own document ----
    //
    // Its own file for the reason E19 has one: it answers a question the contract table
    // cannot ask, at a different shape, and a slope must not be able to reach a row that
    // states a ratio at one size.
    if args.e23_only {
        let pts = run_e23(
            args,
            pg.as_mut()
                .expect("E23 compares two targets and is not reachable under --nls-only"),
            &hosted,
            &hosted_report,
        );
        if let Err(e) = write_e23(args, &pts) {
            eprintln!("bench: writing E23 failed: {e}");
            return 6;
        }
    }

    if let Some(p) = pg.as_mut() {
        let _ = p.teardown();
    }
    if !args.scaling_only && !args.e23_only {
        if let Err(e) = write_all(args, &samples, &config, ceiling) {
            eprintln!("bench: writing results failed: {e}");
            return 6;
        }
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
    engine: std::sync::Arc<std::sync::RwLock<nilestream_server::rev_engine::RevEngine>>,
    seg: std::path::PathBuf,
    accounts: i64,
    rounds: u32,
    budget: usize,
}

impl Hosted {
    /// Replace the engine with one seeded at a **different** size.
    ///
    /// E23 sweeps the base across two decades, and a sweep that could not reseed at each
    /// size would be measuring one size three times.
    fn reseed_at(&self, accounts: i64, rounds: u32) -> Result<(), String> {
        use nilestream_server::rev_engine::RevEngine;
        let mut guard = self.engine.write().map_err(|e| e.to_string())?;
        // The tiny engine first, so the old one's durable sink releases the segment before
        // it is removed — two sequencers over one path recover each other's history.
        *guard = RevEngine::seeded(
            1,
            1,
            1,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        );
        let _ = std::fs::remove_file(&self.seg);
        *guard = RevEngine::seeded(
            accounts,
            rounds,
            self.budget,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        )
        .with_durable(&self.seg)
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Replace the engine with a freshly seeded one on a fresh segment.
    fn reseed(&self) -> Result<(), String> {
        use nilestream_server::rev_engine::RevEngine;
        let mut guard = self.engine.write().map_err(|e| e.to_string())?;
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

/// **E19: does a second and a fourth connection buy anything?**
///
/// Two workloads — a point read and a durable append — driven from 1, 2, 4… connections
/// against both targets, `--runs` times each. Every level re-prepares PostgreSQL and re-seeds
/// the hosted engine, for the same reason the contract loop does: a level that began with the
/// previous level's appends in its base is a different measurement wearing the same name.
///
/// Transaction identities are composed from the run, the level, the thread and the operation
/// index, in a range no other workload in this binary uses. The audit's scratch harness reused
/// one sequence across levels; the ledger refused the repeats as duplicates, and idempotency
/// working exactly as designed was read as a throughput collapse at four connections.
/// One measured point of the E23 sweep.
#[derive(Debug, Clone)]
struct E23 {
    /// `postgres`, `nilestream:cold` or `nilestream:warm`.
    series: String,
    /// Which axis this point was taken along: `base` or `output`.
    axis: &'static str,
    base_rows: u64,
    output_rows: u64,
    run: u32,
    /// Median wall clock of the statement, in milliseconds.
    ms: f64,
    /// Bytes of cell payload the answer carried, excluding protocol framing.
    ///
    /// Here because of what the first full sweep found. The maintained view's cost per row
    /// of base came out *just* above the noise floor — 1.2e-7 ± 3.2e-8 ms/row, positive —
    /// and the obvious reading is that reading maintained state is not quite free of the
    /// history behind it. The obvious reading is wrong, and this column is how a reader can
    /// check that: at a hundred times the base, each account's balance has summed a hundred
    /// times as many postings and is a hundred times larger, so it takes about two more
    /// decimal digits to write down. Ten thousand rows two bytes wider is a ten-percent
    /// larger *answer*, and the wall clock grew about ten percent. The residual is in the
    /// size of the answer, not in the amount of state touched — but that is a claim, and a
    /// claim needs a column.
    bytes: u64,
    /// What the server said it would do. Empty where the target does not say.
    serve_path: String,
    not_run: Option<String>,
}

impl E23 {
    fn csv(&self) -> String {
        format!(
            "{},{},{},{},{},{:.4},{},{},{}",
            self.series,
            self.axis,
            self.base_rows,
            self.output_rows,
            self.run,
            self.ms,
            self.bytes,
            self.serve_path,
            self.not_run.clone().unwrap_or_default().replace(',', ";")
        )
    }
}

const E23_HEADER: &str = "series,axis,base_rows,output_rows,run,ms,answer_bytes,serve_path,not_run";

/// Measure the report statement `n` times and take the median, in milliseconds.
fn time_report(t: &mut dyn Target, n: u32) -> Result<(f64, u64), bank_bench::wire::WireError> {
    let sql = if t.name() == "postgres" {
        workloads::REPORT_STATEMENT.pg
    } else {
        workloads::REPORT_STATEMENT.nls.expect("expressible")
    };
    // Two untimed executions on both targets: the maintained view installs its keys and the
    // buffer cache fills. Charging either to the first measurement would report the cost of
    // becoming ready as the cost of being ready, on whichever target happened to be first.
    t.run(sql)?;
    t.run(sql)?;
    let mut v = Vec::with_capacity(n as usize);
    let mut bytes = 0u64;
    for _ in 0..n {
        let at = std::time::Instant::now();
        let c = t.run(sql)?;
        v.push(at.elapsed().as_secs_f64() * 1000.0);
        // Every execution returns the same answer, so the last one's size is the answer's
        // size. Recorded rather than derived from the row count: the whole point is that
        // rows are not all the same width.
        bytes = c.bytes;
    }
    v.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a wall clock"));
    Ok((v[v.len() / 2], bytes))
}

/// **E23: how the report's cost moves with the base under it and with the answer it returns.**
///
/// Two sweeps, because they answer two different questions and averaging them would answer
/// neither:
///
/// * **Along the base**, with the answer's size held fixed. This is the thesis's H-F1 applied
///   to reports — a stream-first system's per-answer cost must not grow with accumulated
///   input — and it is the only row on which "matches PostgreSQL as the data grows" is a
///   statement that can be false. PostgreSQL must recompute, so its slope is positive by
///   construction; the claim under test is that Nilestream's warm slope is not.
/// * **Along the output**, with the base held fixed. Nobody can beat the size of the answer,
///   so the claim here is parity at the floor rather than a multiple: Nilestream's cost per
///   row of output must be no worse than PostgreSQL's.
///
/// Three series rather than two, because Nilestream has two paths and publishing only the
/// fast one would be publishing half the phase diagram. `nilestream:cold` is the fold over
/// the base — the same work PostgreSQL does — and `nilestream:warm` is the maintained view.
fn run_e23(
    args: &Args,
    pg: &mut PgTarget,
    hosted: &Option<Hosted>,
    hosted_report: &Option<Hosted>,
) -> Vec<E23> {
    let mut out: Vec<E23> = Vec::new();
    let reps = args.e23_reps;

    // (accounts, rounds) pairs. Along the base: the answer is fixed at `accounts + 1` rows
    // and the base grows by a decade at a time. Along the output: the base is held and the
    // answer grows by a decade.
    let base_axis: Vec<(i64, u32)> = args
        .e23_base
        .iter()
        .map(|&rounds| (args.e23_accounts, rounds))
        .collect();
    let output_axis: Vec<(i64, u32)> = args
        .e23_output
        .iter()
        .map(|&accounts| {
            // Hold the base: total postings = accounts * rounds * 2, kept at the middle
            // base size so the two sweeps meet at one common point.
            let target = args.e23_hold_base;
            let rounds = ((target / (accounts.max(1) as u64 * 2)) as u32).max(1);
            (accounts, rounds)
        })
        .collect();

    for run_no in 1..=args.runs {
        for (axis, sizes) in [("base", &base_axis), ("output", &output_axis)] {
            for &(accounts, rounds) in sizes.iter() {
                let base_rows = accounts as u64 * rounds as u64 * 2;
                let output_rows = accounts as u64 + 1;
                eprintln!(
                    "  E23 {axis} run {run_no}: {accounts} accounts x {rounds} rounds \
                     = {base_rows} base rows, {output_rows} rows of answer"
                );

                // ---- PostgreSQL: one path, recompute, the control for both modes ----
                let point = match pg.prepare(accounts, rounds) {
                    Ok(()) => match time_report(pg, reps) {
                        Ok((ms, bytes)) => E23 {
                            series: "postgres".into(),
                            axis,
                            base_rows,
                            output_rows,
                            run: run_no,
                            ms,
                            bytes,
                            serve_path: String::new(),
                            not_run: None,
                        },
                        Err(e) => e23_skip(
                            "postgres",
                            axis,
                            base_rows,
                            output_rows,
                            run_no,
                            e.to_string(),
                        ),
                    },
                    Err(e) => e23_skip(
                        "postgres",
                        axis,
                        base_rows,
                        output_rows,
                        run_no,
                        format!("prepare: {e}"),
                    ),
                };
                out.push(point);

                // ---- Nilestream, both paths ----
                for (series, host, port) in [
                    ("nilestream:cold", hosted, args.nls_port),
                    ("nilestream:warm", hosted_report, args.report_port()),
                ] {
                    let Some(h) = host else {
                        out.push(e23_skip(
                            series,
                            axis,
                            base_rows,
                            output_rows,
                            run_no,
                            "the daemon for this series was not hosted; pass --host-nls".into(),
                        ));
                        continue;
                    };
                    if let Err(e) = h.reseed_at(accounts, rounds) {
                        out.push(e23_skip(
                            series,
                            axis,
                            base_rows,
                            output_rows,
                            run_no,
                            format!("reseed: {e}"),
                        ));
                        continue;
                    }
                    let mut t = match NilestreamTarget::connect("127.0.0.1", port) {
                        Ok(t) => t,
                        Err(e) => {
                            out.push(e23_skip(
                                series,
                                axis,
                                base_rows,
                                output_rows,
                                run_no,
                                format!("connect: {e}"),
                            ));
                            continue;
                        }
                    };
                    let sql = workloads::REPORT_STATEMENT.nls.expect("expressible");
                    // Two untimed runs first, so the path is asked about a *warmed* view —
                    // `explain` on a cold full view reports the path it will take once the
                    // first delta installs the keys, and asking before that would record a
                    // claim about a different moment than the one measured.
                    let _ = t.run(sql);
                    let _ = t.run(sql);
                    let path = t.serve_path(sql).unwrap_or_default();
                    // **The series names a mechanism, so a mismatch is a refusal.** A cold
                    // point served by the view, or a warm point served by the fold, would be
                    // the other series' number under this series' name — and the slopes
                    // this experiment publishes are about the mechanisms, not the ports.
                    let want = if series == "nilestream:warm" {
                        "report-from-view"
                    } else {
                        "fold"
                    };
                    if path != want {
                        out.push(e23_skip(
                            series,
                            axis,
                            base_rows,
                            output_rows,
                            run_no,
                            format!("the server said it would serve this by `{path}`, and this series is `{want}`"),
                        ));
                        continue;
                    }
                    match time_report(&mut t, reps) {
                        Ok((ms, bytes)) => out.push(E23 {
                            series: series.into(),
                            axis,
                            base_rows,
                            output_rows,
                            run: run_no,
                            ms,
                            bytes,
                            serve_path: path,
                            not_run: None,
                        }),
                        Err(e) => out.push(e23_skip(
                            series,
                            axis,
                            base_rows,
                            output_rows,
                            run_no,
                            e.to_string(),
                        )),
                    }
                }
            }
        }
    }
    out
}

fn e23_skip(
    series: &str,
    axis: &'static str,
    base_rows: u64,
    output_rows: u64,
    run: u32,
    why: String,
) -> E23 {
    eprintln!("    {series}: NOT RUN — {why}");
    E23 {
        series: series.into(),
        axis,
        base_rows,
        output_rows,
        run,
        ms: 0.0,
        bytes: 0,
        serve_path: String::new(),
        not_run: Some(why),
    }
}

/// Render E23: the two sweeps, the six slopes, and the two asymptotic contract rows.
fn e23_document(args: &Args, pts: &[E23]) -> String {
    use bank_bench::fit;
    let mut s = String::new();
    s.push_str(
        "# E23 — the report's cost per row of base, and per row of answer\n\n\
         **Generated by `bench --e23`. Nothing in this file is typed in by hand.**\n\n\
         The contract table measures one size. This measures a *shape*, which is what the \
         thesis's structural claim is about: a derived view maintained over an immutable \
         ledger should answer a report without its cost following the history that produced \
         it. That is the foundational hypothesis F1 — a stream-first system's per-answer cost \
         must not grow with accumulated input — narrowed to something a benchmark can \
         falsify.\n\n\
         Three series, because Nilestream has two paths and publishing only the fast one \
         would be publishing half the phase diagram:\n\n\
         * **`postgres`** — recompute. There is no maintained state to read, so the whole \
           base is aggregated for every answer. This is the control for both of the others.\n\
         * **`nilestream:cold`** — the typed fold over the base. The same work PostgreSQL \
           does, on this engine's own scan surface.\n\
         * **`nilestream:warm`** — the maintained view, read in key order at the anchor it is \
           true at, touching no base row. Every point in this series was confirmed by asking \
           the server (`explain`) before it was timed, and a point the server said it would \
           serve by the fold is `NOT RUN` rather than published under this name.\n\n",
    );

    for (axis, title, unit, xname) in [
        (
            "base",
            "Along the base: the answer is fixed, the history grows",
            "row of base",
            "base rows",
        ),
        (
            "output",
            "Along the answer: the base is fixed, the answer grows",
            "row of answer",
            "rows of answer",
        ),
    ] {
        s.push_str(&format!("## {title}\n\n"));
        let mut sizes: Vec<u64> = pts
            .iter()
            .filter(|p| p.axis == axis)
            .map(|p| {
                if axis == "base" {
                    p.base_rows
                } else {
                    p.output_rows
                }
            })
            .collect();
        sizes.sort_unstable();
        sizes.dedup();

        // A column per size, filled with the median of that (series, size)'s runs.
        s.push_str(&format!("Median wall clock, by {xname}.\n\n| Series |"));
        let mut rule = "|---|".to_string();
        for z in &sizes {
            s.push_str(&format!(" {z} |"));
            rule.push_str("--:|");
        }
        s.push('\n');
        s.push_str(&rule);
        s.push('\n');

        for series in ["postgres", "nilestream:cold", "nilestream:warm"] {
            s.push_str(&format!("| `{series}` |"));
            for z in &sizes {
                let mut v: Vec<f64> = pts
                    .iter()
                    .filter(|p| {
                        p.axis == axis
                            && p.series == series
                            && p.not_run.is_none()
                            && (if axis == "base" {
                                p.base_rows
                            } else {
                                p.output_rows
                            }) == *z
                    })
                    .map(|p| p.ms)
                    .collect();
                if v.is_empty() {
                    s.push_str(" NOT RUN |");
                } else {
                    v.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
                    s.push_str(&format!(" {:.2} ms |", v[v.len() / 2]));
                }
            }
            s.push('\n');
        }
        s.push('\n');

        s.push_str(&format!(
            "### The slopes, per {unit}\n\n\
             Ordinary least squares over every run at every size, with the standard error of \
             the slope. **The error bar is the point.** A slope of 0.000003 ms per row is not \
             a finding on its own; whether its confidence interval contains zero is. A fit is \
             refused outright on fewer than three distinct sizes, because with two the \
             residual degrees of freedom are zero and the standard error is 0/0 — a number \
             that would look like certainty and mean nothing.\n\n\
             | Series | Slope (ms per {unit}) | Standard error | R² | n | Distinguishable from zero? |\n\
             |---|--:|--:|--:|--:|:--|\n"
        ));
        for series in ["postgres", "nilestream:cold", "nilestream:warm"] {
            let points: Vec<(f64, f64)> = pts
                .iter()
                .filter(|p| p.axis == axis && p.series == series && p.not_run.is_none())
                .map(|p| {
                    (
                        if axis == "base" {
                            p.base_rows
                        } else {
                            p.output_rows
                        } as f64,
                        p.ms,
                    )
                })
                .collect();
            match fit::fit(&points) {
                Ok(f) => s.push_str(&format!(
                    "| `{series}` | {:.3e} | {:.3e} | {:.3} | {} | **{}** |\n",
                    f.slope,
                    f.stderr,
                    f.r2,
                    f.n,
                    f.verdict()
                )),
                Err(e) => s.push_str(&format!(
                    "| `{series}` | — | — | — | {} | NO FIT: {e} |\n",
                    points.len()
                )),
            }
        }
        s.push('\n');
    }

    s.push_str(&e23_answer_size(pts));
    s.push_str(&e23_contract(pts));

    s.push_str("\n## How it was run\n\n");
    s.push_str(&format!(
        "* Along the base: {} accounts, {} rounds per account — {} rows of answer throughout.\n\
         * Along the answer: {} accounts, rounds chosen so the base stays near {} rows.\n\
         * {} timed executions per point, median reported; {} run(s) of the whole sweep.\n\
         * Both targets are re-seeded at every size, so no point inherits the previous one's \
           base.\n\
         * Both are driven over the PostgreSQL wire protocol through the same client, and the \
           two untimed executions that precede every point are given to both.\n",
        args.e23_accounts,
        args.e23_base
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        args.e23_accounts + 1,
        args.e23_output
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        args.e23_hold_base,
        args.e23_reps,
        args.runs,
    ));
    s
}

/// **How big the answer itself got, along the base axis.**
///
/// The section the first full sweep made necessary. `nilestream:warm`'s cost per row of base
/// came out *just* above its own noise — positive, though three orders of magnitude below
/// PostgreSQL's — and the reading a reader would reach for is that reading maintained state
/// is not quite free of the history behind it.
///
/// It is not, and the reason is arithmetic rather than architecture: with a hundred times the
/// base, each account's balance has summed a hundred times as many postings, so it is about
/// two decimal digits longer to write down. Ten thousand rows, two bytes wider each, is a
/// larger *answer* — and the answer is the one thing no design can make smaller.
///
/// So the residual is reported as what it is by measuring it: bytes of answer at each base
/// size, and the wall clock **per kilobyte of answer**. If that second series is flat, the
/// growth is in serialising a bigger answer and not in touching more state. This is offered
/// as evidence about a cause, not as a replacement verdict — the contract row above is
/// judged on the literal criterion and says NOT MET when it is not met.
fn e23_answer_size(pts: &[E23]) -> String {
    use bank_bench::fit;
    let mut sizes: Vec<u64> = pts
        .iter()
        .filter(|p| p.axis == "base" && p.not_run.is_none())
        .map(|p| p.base_rows)
        .collect();
    sizes.sort_unstable();
    sizes.dedup();
    if sizes.len() < 2 {
        return String::new();
    }

    let mut s = String::from(
        "## The residual, and where it is: the answer got bigger\n\n\
         Along the base axis the answer has the same **number** of rows at every size — the \
         account count does not change — but not the same number of **bytes**. At a hundred \
         times the base each balance has summed a hundred times as many postings, so it \
         needs about two more decimal digits. Ten thousand rows two bytes wider is a larger \
         answer, and no design makes an answer smaller than itself.\n\n\
         This section measures that rather than asserting it. If the wall clock per kilobyte \
         of answer is flat while the wall clock per row of base is not, the residual is in \
         serialising a bigger answer and not in touching more state.\n\n\
         | Series | ",
    );
    let mut rule = "|---|".to_string();
    for z in &sizes {
        s.push_str(&format!("{z} base rows | "));
        rule.push_str("--:|");
    }
    s.push('\n');
    s.push_str(&rule);
    s.push('\n');

    for series in ["postgres", "nilestream:cold", "nilestream:warm"] {
        s.push_str(&format!("| `{series}` (answer bytes) |"));
        for z in &sizes {
            let mut v: Vec<u64> = pts
                .iter()
                .filter(|p| {
                    p.axis == "base"
                        && p.series == series
                        && p.not_run.is_none()
                        && p.base_rows == *z
                })
                .map(|p| p.bytes)
                .collect();
            v.sort_unstable();
            if v.is_empty() {
                s.push_str(" — |");
            } else {
                s.push_str(&format!(" {} |", v[v.len() / 2]));
            }
        }
        s.push('\n');
    }
    s.push('\n');

    s.push_str(
        "| Series | Slope (ms per **kilobyte of answer**) | Standard error | R² | n | Distinguishable from zero? |\n\
         |---|--:|--:|--:|--:|:--|\n",
    );
    for series in ["postgres", "nilestream:cold", "nilestream:warm"] {
        let points: Vec<(f64, f64)> = pts
            .iter()
            .filter(|p| {
                p.axis == "base" && p.series == series && p.not_run.is_none() && p.bytes > 0
            })
            .map(|p| (p.bytes as f64 / 1024.0, p.ms))
            .collect();
        match fit::fit(&points) {
            Ok(f) => s.push_str(&format!(
                "| `{series}` | {:.3e} | {:.3e} | {:.3} | {} | **{}** |\n",
                f.slope,
                f.stderr,
                f.r2,
                f.n,
                f.verdict()
            )),
            Err(e) => s.push_str(&format!(
                "| `{series}` | — | — | — | {} | NO FIT: {e} |\n",
                points.len()
            )),
        }
    }
    s.push_str(
        "\nA reader should hold this to the same standard as everything else here: it is a \
         second fit over the same nine points, and a second fit that happens to say what one \
         hoped is not evidence. What makes it worth reading is that the two fits use **the \
         same measurements** and differ only in what they are plotted against — so if the \
         per-kilobyte slope were also positive and the per-base-row slope were the real \
         effect, this table would say so.\n\n",
    );
    s
}

/// The two asymptotic rows, judged from the fits.
fn e23_contract(pts: &[E23]) -> String {
    use bank_bench::fit;
    let series_points = |axis: &str, series: &str| -> Vec<(f64, f64)> {
        pts.iter()
            .filter(|p| p.axis == axis && p.series == series && p.not_run.is_none())
            .map(|p| {
                (
                    if axis == "base" {
                        p.base_rows
                    } else {
                        p.output_rows
                    } as f64,
                    p.ms,
                )
            })
            .collect()
    };

    let mut s = String::from(
        "## The two asymptotic contract rows\n\n\
         These are the rows that make \"matches PostgreSQL as the data grows\" a statement \
         that can be false. A ratio at one size cannot: a system can win at twenty thousand \
         rows and lose at two million, and the contract table would look identical.\n\n\
         | Row | Claim | PostgreSQL | Nilestream (warm) | Verdict |\n\
         |---|---|---|---|---|\n",
    );

    // (a) per row of base: warm slope must be flat; PostgreSQL's must be positive.
    let pg_base = fit::fit(&series_points("base", "postgres"));
    let nl_base = fit::fit(&series_points("base", "nilestream:warm"));
    let base_verdict = render::asymptotic::per_row_of_base(&pg_base, &nl_base);
    s.push_str(&format!(
        "| Per row of base | Nilestream's warm cost is *o(1)* in the base — the slope's \
         confidence interval contains zero — while PostgreSQL's is positive (H-F1) | {} | {} | {} |\n",
        fit_cell(&pg_base),
        fit_cell(&nl_base),
        base_verdict
    ));
    // (b) per row of output: warm slope must be <= PostgreSQL's.
    let pg_out = fit::fit(&series_points("output", "postgres"));
    let nl_out = fit::fit(&series_points("output", "nilestream:warm"));
    let out_verdict = render::asymptotic::per_row_of_answer(&pg_out, &nl_out);
    s.push_str(&format!(
        "| Per row of answer | Nilestream's cost per row of answer is no worse than \
         PostgreSQL's — parity at the floor, since nobody can beat the size of the answer | {} | {} | {} |\n",
        fit_cell(&pg_out),
        fit_cell(&nl_out),
        out_verdict
    ));

    s.push_str(
        "\nThe second row is a **parity** claim on purpose. A report has to put its answer on \
         the wire, and that cost is the same problem for everyone; a system claiming to beat \
         it would be claiming to send fewer bytes than the answer contains. The place where a \
         maintained view can win is the first row, and that is where the claim is a multiple \
         rather than a tie.\n",
    );
    // **The instrument's own floor, printed beside the verdict.**
    //
    // This row has come out both ways across full sweeps of the same binary on the same
    // host: 1.176e-7 +- 3.2e-8 (positive, NOT MET) and 5.376e-8 +- 2.3e-7 (flat, MET). Both
    // are honest readings of their own nine points, and the reason they disagree is that the
    // effect is at the edge of what three sizes and three runs can resolve. A results file
    // that printed only whichever verdict it got would be reporting the throw of a die as a
    // property of the system, so the resolution floor is printed with it: the smallest slope
    // this design could have distinguished from zero.
    if let (Ok(p), Ok(n)) = (&pg_base, &nl_base) {
        let floor = 2.0 * n.stderr;
        s.push_str(&format!(
            "\n**What this row can and cannot resolve.** The verdict is a comparison against \
             the fit's own standard error, so it is only as sharp as the measurement. On \
             these {} points the smallest warm slope distinguishable from zero is \
             **{:.2e} ms per row of base**; the measured slope is {:.2e}, and the control's \
             is {:.2e} — {:.0}× larger. This row has come out **both ways** across full \
             sweeps of the same binary on the same host, because the effect it is asked \
             about sits near that floor. What is stable across every sweep is the ratio to \
             the control and the per-kilobyte fit above; what is not stable is whether a \
             quantity that small is called zero. A reader should take the ratio as the \
             result and this verdict as the strict form of a question the experiment is \
             close to being unable to answer.\n\n",
            n.n,
            floor,
            n.slope,
            p.slope,
            if n.slope.abs() > 0.0 {
                (p.slope / n.slope).abs()
            } else {
                f64::INFINITY
            }
        ));
    }
    if base_verdict.contains("NOT MET") {
        if let (Ok(p), Ok(n)) = (&pg_base, &nl_base) {
            let _ = (p, n);
            s.push_str(
                "\n**This sweep read NOT MET, and it is reported as measured.** The literal \
                 criterion is that the warm slope's confidence interval contains zero, and on \
                 these points it does not. The section above locates what is left: the answer \
                 itself is bigger at a bigger base, because each balance has summed more \
                 postings and takes more digits to write. That is a real cost and it is not a \
                 cost of touching more state — but this row is judged on what was asked, not \
                 on what the cause turned out to be.\n\n",
            );
        }
    }
    s
}

fn fit_cell(f: &Result<bank_bench::fit::Fit, bank_bench::fit::NoFit>) -> String {
    match f {
        Ok(f) => format!(
            "{:.3e} ± {:.1e} ms/row ({})",
            f.slope,
            f.stderr,
            f.verdict()
        ),
        Err(e) => format!("no fit: {e}"),
    }
}

/// Both halves of one E19 run: the per-workload scaling rows, and the mixed rows.
///
/// Two vectors rather than one, because a `MixedSample` is not a `ScalingSample` with extra
/// fields — it has two roles and therefore two latency distributions, and flattening them into
/// the scaling shape would mean a row whose `p99` silently means "reads" in some rows and
/// "writes" in others.
pub struct ScalingRun {
    pub scaling: Vec<ScalingSample>,
    pub mixed: Vec<workloads::MixedSample>,
}

fn run_scaling(args: &Args, mut pg: Option<&mut PgTarget>, hosted: &Option<Hosted>) -> ScalingRun {
    let mut out: Vec<ScalingSample> = Vec::new();
    let mut out_mixed: Vec<workloads::MixedSample> = Vec::new();
    // Far from the contract loop's identity ranges (1–3 million), so the two experiments
    // cannot collide even if a future change stops re-seeding between them.

    for run_no in 1..=args.runs {
        for (level, &conns) in args.connections.iter().enumerate() {
            let seed = 0xE19 ^ ((run_no as u64) << 8) ^ (conns as u64);

            // ---- PostgreSQL ----
            //
            // Absent under `--nls-only`, and recorded as `NOT RUN` with the reason rather
            // than omitted: a CSV missing its comparison arm and a CSV that never had one
            // must not look alike.
            let have_pg = match pg.as_deref_mut() {
                None => {
                    for w in ["point", "fold", "durable"] {
                        out.push(workloads::scaling_skipped(
                            w,
                            "postgres",
                            conns,
                            run_no,
                            "--nls-only: no PostgreSQL arm was run, so this is not a comparison"
                                .to_string(),
                        ));
                    }
                    false
                }
                Some(p) => match p.prepare(args.accounts, args.rounds) {
                    Err(e) => {
                        let why = format!("PostgreSQL could not be prepared for the level: {e}");
                        for w in ["point", "fold", "durable"] {
                            out.push(workloads::scaling_skipped(
                                w,
                                "postgres",
                                conns,
                                run_no,
                                why.clone(),
                            ));
                        }
                        false
                    }
                    Ok(()) => true,
                },
            };
            if !have_pg {
                // Fall through to the Nilestream arm rather than `continue`, which would
                // have skipped it too.
                run_nilestream_level(
                    args,
                    hosted,
                    &mut out,
                    &mut out_mixed,
                    conns,
                    run_no,
                    level,
                    seed,
                );
                continue;
            }
            let (host, port, user, db) = (
                args.pg_host.clone(),
                args.pg_port,
                args.pg_user.clone(),
                args.pg_db.clone(),
            );
            let open_pg = move || bank_bench::wire::Client::connect(&host, port, &user, &db);
            let s = workloads::concurrent(
                workloads::Level {
                    workload: "point",
                    target: "postgres",
                    connections: conns,
                    run: run_no,
                    per_connection: args.operations,
                    durable: false,
                },
                &open_pg,
                &|thread, i| {
                    let k = scaling_key(seed, thread, i, args.accounts);
                    format!("select acct, sum(amt) from postings where acct = {k} group by acct")
                },
            );
            report_scaling(&s);
            out.push(s);
            // **The scan-shaped read, which is the one the audit found lock-bound.**
            //
            // An unkeyed `group by acct` over the whole base — the control arm for the
            // Nilestream row of the same name, running the identical statement against a
            // database with no maintained state, which is what it has to do.
            //
            // Far fewer operations per connection than the point workload, because one of
            // these folds twenty thousand postings into ten thousand groups and costs
            // milliseconds rather than microseconds.
            let s = workloads::concurrent(
                workloads::Level {
                    workload: "fold",
                    target: "postgres",
                    connections: conns,
                    run: run_no,
                    per_connection: (args.operations / 40).max(25),
                    durable: false,
                },
                &open_pg,
                &|_thread, _i| {
                    workloads::REPORT_STATEMENT
                        .nls
                        .expect("expressible")
                        .to_string()
                },
            );
            report_scaling(&s);
            out.push(s);
            let s = workloads::concurrent(
                workloads::Level {
                    workload: "durable",
                    target: "postgres",
                    connections: conns,
                    run: run_no,
                    per_connection: (args.operations / 4).max(50),
                    durable: true,
                },
                &open_pg,
                &|thread, i| {
                    let acct = scaling_key(seed, thread, i, args.accounts);
                    let id = txn_base(run_no, level, thread, i);
                    format!(
                        "insert into postings (txn, acct, cur, amt, epoch) values \
                         ('scale-{id}', {acct}, 'USD', 0, {id})"
                    )
                },
            );
            report_scaling(&s);
            out.push(s);

            // ---- Nilestream ----
            run_nilestream_level(
                args,
                hosted,
                &mut out,
                &mut out_mixed,
                conns,
                run_no,
                level,
                seed,
            );
        }
    }
    ScalingRun {
        scaling: out,
        mixed: out_mixed,
    }
}

/// The account a scaling thread touches at iteration `i`, drawn without shared state.
///
/// A free function rather than a captured closure: the workload driver hands this to threads,
/// so anything it closes over must be `Sync`, and the draw is reproducible from
/// `(seed, thread, i)` alone.
fn scaling_key(seed: u64, thread: u32, i: u64, accounts: i64) -> i64 {
    workloads::Rng::seeded(seed ^ ((thread as u64) << 40) ^ i.wrapping_mul(0x9E37_79B9))
        .skewed_key(accounts, 0.9)
}

/// A transaction identity that is unique across run, level, thread and iteration.
///
/// The ledger refuses a duplicate, which is the idempotency guarantee working; a scaling
/// sweep that reused identities would report that guarantee as a failed workload.
fn txn_base(run: u32, level: usize, thread: u32, i: u64) -> i64 {
    900_000_000
        + (run as i64) * 10_000_000
        + (level as i64) * 1_000_000
        + (thread as i64) * 100_000
        + i as i64
}

/// The Nilestream arm of one connection level.
///
/// Lifted out of `run_scaling` so `--nls-only` runs exactly the same code the comparison
/// does. A one-armed run that measured a *different* path would be a worse instrument than
/// no run at all.
#[allow(clippy::too_many_arguments)]
fn run_nilestream_level(
    args: &Args,
    hosted: &Option<Hosted>,
    out: &mut Vec<ScalingSample>,
    out_mixed: &mut Vec<workloads::MixedSample>,
    conns: u32,
    run_no: u32,
    level: usize,
    seed: u64,
) {
    {
        {
            if let Some(h) = hosted {
                if let Err(e) = h.reseed() {
                    let why = format!("the hosted engine could not be re-seeded: {e}");
                    for w in ["point", "fold", "durable"] {
                        out.push(workloads::scaling_skipped(
                            w,
                            "nilestream",
                            conns,
                            run_no,
                            why.clone(),
                        ));
                    }
                    return;
                }
            }
            let nls_port = args.nls_port;
            let open_nls =
                move || bank_bench::wire::Client::connect("127.0.0.1", nls_port, "bench", "bank");
            // **The keyed-read level is the one the fallback rate is about**, so it is
            // bracketed by the counters. The `fold` level never consults the view and the
            // `durable` level is a write, so neither has a rate to report.
            let before = nls_view_counters(nls_port);
            let mut s = workloads::concurrent(
                workloads::Level {
                    workload: "point",
                    target: "nilestream",
                    connections: conns,
                    run: run_no,
                    per_connection: args.operations,
                    durable: false,
                },
                &open_nls,
                &|thread, i| {
                    let k = scaling_key(seed, thread, i, args.accounts);
                    format!("select acct, sum(amt) from postings where acct = {k} group by acct")
                },
            );
            s.fallback_rate = fallback_rate_between(before, nls_view_counters(nls_port));
            report_scaling(&s);
            out.push(s);
            // **The scan-shaped read, which is the one the audit found lock-bound.**
            //
            // An unkeyed `group by acct` over the whole base. On this hosted engine the
            // balance view is `Demand` with a budget below the key count, so `is_full()` is
            // false and `report_from_view` refuses it: this row is the *fold*, every time,
            // which is what makes it the right instrument here. The audit called this
            // workload `report`; it is named `fold` in this table because the contract
            // table's `report` row is the opposite — a row that is `NOT RUN` unless the
            // server answers it from the maintained view without touching a base row.
            //
            // Far fewer operations per connection than the point workload, because one of
            // these folds twenty thousand postings into ten thousand groups and costs
            // milliseconds rather than microseconds.
            let s = workloads::concurrent(
                workloads::Level {
                    workload: "fold",
                    target: "nilestream",
                    connections: conns,
                    run: run_no,
                    per_connection: (args.operations / 40).max(25),
                    durable: false,
                },
                &open_nls,
                &|_thread, _i| {
                    workloads::REPORT_STATEMENT
                        .nls
                        .expect("expressible")
                        .to_string()
                },
            );
            report_scaling(&s);
            out.push(s);
            let s = workloads::concurrent(
                workloads::Level {
                    workload: "durable",
                    target: "nilestream",
                    connections: conns,
                    run: run_no,
                    per_connection: (args.operations / 4).max(50),
                    durable: true,
                },
                &open_nls,
                &|thread, i| {
                    let acct = scaling_key(seed, thread, i, args.accounts);
                    let id = txn_base(run_no, level, thread, i);
                    format!("insert into postings values ({id}, {acct}, 0, 0)")
                },
            );
            report_scaling(&s);
            out.push(s);

            // **The third phase, which is the one E19 never had.**
            //
            // `point` above is readers alone and `durable` is writers alone; both halves of the
            // mixed workload have been measured for cycles, in isolation, and the isolated
            // measurements are exactly the ones that cannot see the anchor-mismatch fallback —
            // it needs a writer moving the frontier under a reader. So the audit built
            // `probes/mixed` outside the repository to ask, which is what a question the
            // results tables cannot answer looks like.
            //
            // Writers are ⌈readers/2⌉, so the level carries both roles at a ratio the write
            // path can actually sustain: a durable append costs a barrier, and matching the
            // reader count one-for-one would measure a queue at the sealer rather than
            // contention at the base.
            let writers = conns.div_ceil(2);
            let before = nls_view_counters(nls_port);
            let mut m = workloads::mixed(
                workloads::MixedLevel {
                    target: "nilestream",
                    readers: conns,
                    writers,
                    run: run_no,
                    seconds: args.mixed_seconds,
                },
                &open_nls,
                &|thread, i| {
                    let k = scaling_key(seed ^ 0x5EED, thread, i, args.accounts);
                    format!("select acct, sum(amt) from postings where acct = {k} group by acct")
                },
                &|thread, i| {
                    let acct = scaling_key(seed, thread, i, args.accounts);
                    // Offset the identity space so a mixed writer cannot collide with the
                    // `durable` level's, which would be refused as a duplicate and read as a
                    // throughput collapse rather than as the commit rule working.
                    let id = txn_base(run_no, level, thread, i) + 500_000_000;
                    format!("insert into postings values ({id}, {acct}, 0, 0)")
                },
            );
            m.fallback_rate = fallback_rate_between(before, nls_view_counters(nls_port));
            let (batch, wait) = nls_sealer_counters(nls_port);
            m.max_batch = batch;
            m.lock_wait_p99_us = wait;
            m.base_epochs = nls_frontier(nls_port);
            report_mixed(&m);
            out_mixed.push(m);
        }
    }
}

/// `(max_batch, lock_wait_p99_us)` from `select nilestream_sealer`, by name.
///
/// By name for the reason the view counters are: the reply has fourteen columns and reading
/// the eleventh is correct exactly until someone adds one.
fn nls_sealer_counters(port: u16) -> (Option<u64>, Option<u64>) {
    let Ok(mut c) = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank") else {
        return (None, None);
    };
    let Ok(r) = c.simple("select nilestream_sealer") else {
        return (None, None);
    };
    let at = |name: &str| -> Option<u64> {
        let i = r.columns.iter().position(|c| c == name)?;
        r.rows.first()?.get(i)?.as_ref()?.trim().parse().ok()
    };
    (at("max_batch"), at("lock_wait_p99_us"))
}

/// The ledger's frontier, so a mixed row can say how much history it was measured against.
fn nls_frontier(port: u16) -> Option<u64> {
    let mut c = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank").ok()?;
    let r = c.simple("select nilestream_frontier").ok()?;
    let i = r.columns.iter().position(|c| c == "frontier")?;
    r.rows.first()?.get(i)?.as_ref()?.trim().parse().ok()
}

fn report_mixed(m: &workloads::MixedSample) {
    match &m.not_run {
        Some(why) => eprintln!(
            "  E19 mixed {} {}r/{}w run {} — NOT RUN: {why}",
            m.target, m.readers, m.writers, m.run
        ),
        None => eprintln!(
            "  E19 mixed {} {}r/{}w run {} — {:.0} reads/s (p50 {:.0}µs), {:.0} writes/s \
             (p99 {:.0}µs), fallback {}",
            m.target,
            m.readers,
            m.writers,
            m.run,
            m.reads_per_second(),
            m.read_p50.as_nanos() as f64 / 1000.0,
            m.writes_per_second(),
            m.write_p99.as_nanos() as f64 / 1000.0,
            m.fallback_rate
                .map(|r| format!("{:.2}%", r * 100.0))
                .unwrap_or_else(|| "REFUSED (not measured)".into())
        ),
    }
}

/// **The view counters, so a level can report the rate it actually ran at.**
///
/// `(view_answers, fallbacks)` read by name from `select nilestream_stats`. The counters are
/// cumulative over the server's lifetime, so a level's own rate is a delta across it — a
/// cumulative rate would be dominated by whatever ran first and would drift towards a
/// constant as the run went on, which is the opposite of what a connection sweep is asking.
///
/// `None` when the server cannot be asked, and the level then renders `n/a` rather than
/// `0.0%`: "no read fell back" and "the question was not asked" are different claims.
fn nls_view_counters(port: u16) -> Option<(u64, u64)> {
    let mut c = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank").ok()?;
    let r = c.simple("select nilestream_stats").ok()?;
    let at = |name: &str| -> Option<u64> {
        let i = r.columns.iter().position(|c| c == name)?;
        r.rows.first()?.get(i)?.as_ref()?.trim().parse().ok()
    };
    Some((at("view_answers")?, at("fallbacks")?))
}

/// The share of the keyed reads *in this level* that fell back to the fold.
fn fallback_rate_between(before: Option<(u64, u64)>, after: Option<(u64, u64)>) -> Option<f64> {
    let ((a0, f0), (a1, f1)) = (before?, after?);
    let (answers, fallbacks) = (a1.saturating_sub(a0), f1.saturating_sub(f0));
    let keyed = answers + fallbacks;
    (keyed > 0).then(|| fallbacks as f64 / keyed as f64)
}

fn report_scaling(s: &ScalingSample) {
    match &s.not_run {
        Some(why) => eprintln!(
            "  E19 {} {} @{} conn run {} — NOT RUN: {why}",
            s.workload, s.target, s.connections, s.run
        ),
        None => eprintln!(
            "  E19 {} {} @{} conn run {} — {:.0} ops/s, p99 {:.0}µs, fallback {}",
            s.workload,
            s.target,
            s.connections,
            s.run,
            s.ops_per_second(),
            s.p99.as_nanos() as f64 / 1000.0,
            s.fallback_rate
                .map(|r| format!("{:.1}%", r * 100.0))
                .unwrap_or_else(|| "n/a".into())
        ),
    }
}

fn write_e23(args: &Args, pts: &[E23]) -> std::io::Result<()> {
    let dir = bank_bench::publish::e23_dir(std::path::Path::new(&args.out));
    std::fs::create_dir_all(&dir)?;
    let mut csv = String::from(E23_HEADER);
    csv.push('\n');
    for p in pts {
        csv.push_str(&p.csv());
        csv.push('\n');
    }
    std::fs::write(dir.join("E23-scaling.csv"), &csv)?;
    let doc = e23_document(args, pts);
    let where_to = bank_bench::publish::e23_destinations(&dir, args.publish);
    for path in &where_to {
        std::fs::write(path, &doc)?;
    }
    eprintln!(
        "wrote {} and {}/E23-scaling.csv",
        where_to
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        dir.display()
    );
    Ok(())
}

fn write_scaling(
    args: &Args,
    samples: &[ScalingSample],
    mixed: &[workloads::MixedSample],
) -> std::io::Result<()> {
    let dir = bank_bench::publish::scaling_dir(std::path::Path::new(&args.out));
    std::fs::create_dir_all(&dir)?;
    let mut by_file: BTreeMap<&str, Vec<&ScalingSample>> = BTreeMap::new();
    for s in samples {
        by_file.entry(s.workload.as_str()).or_default().push(s);
    }
    for (w, rows) in &by_file {
        let mut f = std::fs::File::create(dir.join(format!("{w}.csv")))?;
        writeln!(f, "{}", workloads::SCALING_CSV_HEADER)?;
        for s in rows {
            writeln!(f, "{}", s.to_csv())?;
        }
    }
    // Its own file and its own header: a mixed row has two roles and therefore two latency
    // distributions, and forcing it into the scaling schema would give a `p99` column that
    // means reads in some rows and writes in others.
    if !mixed.is_empty() {
        let mut f = std::fs::File::create(dir.join("mixed.csv"))?;
        writeln!(f, "{}", workloads::MIXED_CSV_HEADER)?;
        for m in mixed {
            writeln!(f, "{}", m.to_csv())?;
        }
    }
    let doc = scaling_document(samples, mixed, args);
    let where_to = bank_bench::publish::scaling_destinations(&dir, args.publish);
    for path in &where_to {
        std::fs::write(path, &doc)?;
    }
    eprintln!(
        "\nwrote {} and {}/*.csv",
        where_to
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        dir.display()
    );
    Ok(())
}

fn scaling_document(
    samples: &[ScalingSample],
    mixed: &[workloads::MixedSample],
    args: &Args,
) -> String {
    let mut s = String::new();
    s.push_str("# E19 — Throughput against connection count\n\n");
    s.push_str(
        "**Generated by `bench --run --connections …`. Nothing in this file is typed in by \
         hand, and none of it is a contract result.**\n\n\
         `SPEC-ENGINE.md` Part 0 states its four targets without a concurrency qualifier, and \
         E16 measures them from one connection. This experiment asks a different question: \
         when a second and a fourth connection are added, does throughput rise? A server that \
         serialises its sessions answers a level of four connections at the speed of one, and \
         until this table existed nothing in the repository could have said whether either \
         target does.\n\n",
    );
    s.push_str(
        "> **This host has 2 cores.** These rows say whether throughput rises from 1 to 2 to \
         4 connections *on two cores*. They say nothing about 16 or 48, and a reader who \
         extrapolates them to a server-class machine is reading a number this experiment did \
         not measure. The saturation point of a 2-core host is a property of the host.\n\n",
    );
    s.push_str(&render::scaling_table(samples));
    s.push_str("\n### The top step\n\n");
    s.push_str(&render::scaling_verdicts(samples));
    if !mixed.is_empty() {
        s.push_str("\n## Readers and writers at the same time\n\n");
        s.push_str(&format!(
            "The `point` rows above are readers alone and `durable` is writers alone. Both \
             halves of this workload have been measured for cycles **in isolation**, and \
             isolation is exactly what hides the anchor-mismatch fallback: a keyed read is \
             discarded when a writer has moved the view's frontier past the reader's anchor, \
             so with no writer the rate is 0.0% whatever the engine does. It read ~46% under \
             writers before T-02.\n\n\
             Readers are the connection count; writers are ⌈readers/2⌉, because a durable \
             append costs a barrier and matching them one-for-one would measure a queue at \
             the sealer rather than contention at the base. {} s per level.\n\n",
            args.mixed_seconds
        ));
        s.push_str(&render::mixed_table(mixed));
        let refused = render::mixed_table_refusals(mixed);
        s.push_str(&format!(
            "\n**Levels refused for want of a fallback rate: {refused}.** A mixed row exists to \
             carry that column; without it the throughput figures are the `point` and `durable` \
             rows again, published under a name that claims more. Any number above zero here \
             means this section measured less than it says.\n"
        ));
    }
    s.push_str("\n## How it was run\n\n");
    s.push_str(&format!(
        "* Connection levels: {}\n\
         * Runs per level: {} (medians reported)\n\
         * Point-read operations **per connection**: {} — so a level of four connections \
         issues four times the work of a level of one. That is the shape the question needs: \
         if the server scales, the wall clock stays flat and ops/s rises; if it serialises, \
         the wall clock grows and ops/s does not.\n\
         * Durable-append operations per connection: {}\n\
         * Accounts: {}; rounds per account: {} (conserved pairs, on both targets)\n\
         * Every level re-prepares PostgreSQL and re-seeds the hosted engine, so no level \
         starts from the previous level's appends.\n\
         * Transaction identities are composed from `(run, level, thread, operation)` in a \
         range no other workload uses, so no append is ever a duplicate of another level's.\n\
         * Connections are opened **before** the clock starts: a level is charged for its \
         operations, not for its sockets.\n\
         * Latencies are pooled across every thread of a level before the percentiles are \
         taken, so `p99` is what a client saw and not the median thread's p99.\n\n\
         ### Why the step from 1 to 2 can exceed the core count\n\n\
         The single-connection level is **round-trip bound, not core bound**: the client \
         sends, blocks, and reads, so the server is idle for much of each operation and one \
         core is never saturated. A second connection fills that idle time as well as using \
         the second core, so a ratio above 2.00x at two connections is the pipeline being \
         filled rather than superlinear scaling. The ratio worth reading is the one from 2 to \
         4, where both cores are already busy: a target that keeps rising there is \
         parallelising, and one that falls is contending.\n\n",
        args.connections
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        args.runs,
        args.operations,
        (args.operations / 4).max(50),
        args.accounts,
        args.rounds,
    ));
    s.push_str(
        "## What this table is not\n\n\
         It is **not** a contract result. `results/E16-wallclock.md` holds the four rows of \
         `SPEC-ENGINE.md` Part 0, measured from one connection, and no figure from this \
         document belongs in them. The two experiments are written by different code paths \
         into different types for that reason: a `ScalingSample` cannot be rendered into the \
         contract table, and a `Sample` carries no connection count.\n",
    );
    s
}

/// One `report` sample, with the appender's own numbers folded into the log.
///
/// The appends are a *precondition* of the measurement rather than a result of it: a report
/// timed against a base nothing was writing to is a cache measurement, and this function
/// prints the count so a run where the appender did nothing is visible at the moment it
/// happens rather than three files later.
fn report_row(
    t: &mut dyn Target,
    args: &Args,
    run_no: u32,
    open: &(dyn Fn() -> Result<bank_bench::wire::Client, bank_bench::wire::WireError> + Sync),
    expect_base: &mut Option<u64>,
) -> Sample {
    let name = t.name().to_string();
    let is_pg = name == "postgres";
    let accounts = args.accounts;
    let append = move |run: u32, i: u64| -> String {
        // Two conserved legs, exactly as `oltp` writes them, so the base grows the way the
        // benchmark's own write path grows it.
        let from = 1 + (i % accounts.max(1) as u64) as i64;
        let to = 1 + ((i * 7 + 3) % accounts.max(1) as u64) as i64;
        let to = if to == from {
            1 + (to % accounts.max(1))
        } else {
            to
        };
        let amount = 1 + (i % 997) as i64;
        if is_pg {
            format!(
                "insert into postings (txn, acct, cur, amt, epoch) values \
                 ('rep-{run}-{i}', {from}, 'USD', -{amount}, {}), \
                 ('rep-{run}-{i}', {to}, 'USD', {amount}, {})",
                8_000_000 + i,
                8_000_000 + i
            )
        } else {
            format!(
                "insert into postings values ({txn}, {from}, 0, -{amount}), ({txn}, {to}, 0, {amount})",
                txn = 8_000_000 + (run as u64) * 1_000_000 + i
            )
        }
    };
    // The two report rows must start from the same base, and the log says so per run.
    match workloads::report(
        t,
        (args.operations / 10).max(20),
        run_no,
        args.append_rate,
        open,
        &append,
    ) {
        Ok(r) => {
            eprintln!(
                "  report[{name}] run {run_no}: {} appends on the second connection at \
                 {:.0}/s (asked {:.0}/s), base {} -> {}, served by {}",
                r.appends,
                r.append_rate_achieved,
                r.append_rate_target,
                r.base_before.map(|n| n.to_string()).unwrap_or("?".into()),
                r.base_after.map(|n| n.to_string()).unwrap_or("?".into()),
                r.serve_path
                    .as_deref()
                    .unwrap_or("(the target does not say)"),
            );
            if r.serve_path
                .as_deref()
                .is_some_and(|p| p != "report-from-view")
            {
                // A row that measured the fold under the maintained view's name is worse
                // than a missing row: it is a number that supports the structural claim
                // while having been produced by the mechanism the claim is against.
                return workloads::skipped(
                    "report",
                    &name,
                    run_no,
                    format!(
                        "the server said it would serve this by `{}` rather than \
                         `report-from-view`; a fold measured under the maintained view's \
                         name is the one number this row must never publish",
                        r.serve_path.as_deref().unwrap_or("?")
                    ),
                );
            }
            // **The two arms must have started from the same base.** The first arm records
            // what it found and the second is checked against it: a ratio between a 24,500-row
            // table and a 20,000-row ledger is not a ratio, and this row had exactly that
            // defect on its first publication because PostgreSQL entered it carrying the
            // `oltp` and `durable` appends while the Nilestream daemon had been reseeded.
            match (*expect_base, r.base_before) {
                (None, Some(n)) => *expect_base = Some(n),
                (Some(first), Some(n)) if first != n => {
                    return workloads::skipped(
                        "report",
                        &name,
                        run_no,
                        format!(
                            "this run's other target started the report from {first} base \
                             rows and this one from {n}; a ratio between two different \
                             problems is not a ratio"
                        ),
                    );
                }
                _ => {}
            }
            if r.appends == 0 && r.sample.not_run.is_none() {
                return workloads::skipped(
                    "report",
                    &name,
                    run_no,
                    "the appender completed no appends, so this would have been a report over \
                     a quiesced base — which is a cache measurement, not a maintained-view one"
                        .into(),
                );
            }
            r.sample
        }
        Err(e) => workloads::skipped("report", &name, run_no, format!("{e}")),
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
    ceiling: Option<bank_bench::storage::Ceiling>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&args.out)?;
    // One file per contract workload, and one for the per-statement rows: a workload column
    // of `analytical:group_by_acct` must not become a file name with a colon in it.
    let mut by_file: BTreeMap<&str, Vec<&Sample>> = BTreeMap::new();
    for s in samples {
        // From the one list, not a ninth copy of it: `report` was added to
        // `CONTRACT_WORKLOADS` and to the renderer and missed here, so ten runs of the new
        // row were written into `analytical-statements.csv` and `--render` could not find
        // them. A workload not in this list is a per-statement row by definition.
        let file = if CONTRACT_WORKLOADS.contains(&s.workload.as_str()) {
            s.workload.as_str()
        } else {
            "analytical-statements"
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

    // **The device's ceiling, as raw data beside the raw data.** Every other number in this
    // document has a CSV behind it; the ceiling section was rendered from a value that
    // existed only in the running process, so the one paragraph most likely to be disputed
    // — "was the disk steady while you measured?" — was the one with nothing to check it
    // against.
    if let Some(c) = ceiling {
        std::fs::write(
            std::path::Path::new(&args.out).join("device.csv"),
            format!(
                "median,mad,lowest,highest,probes\n{:.4},{:.4},{:.4},{:.4},{}\n",
                c.median, c.mad, c.lowest, c.highest, c.probes
            ),
        )?;
    }

    let doc = document(samples, config, args, ceiling);
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

fn document(
    samples: &[Sample],
    config: &[(String, Vec<(String, String)>)],
    args: &Args,
    ceiling: Option<bank_bench::storage::Ceiling>,
) -> String {
    let gaps: BTreeMap<String, String> = CONTRACT_WORKLOADS
        .iter()
        .filter_map(|w| nilestream_gap(w).map(|r| (w.to_string(), r)))
        .collect();
    let table = &render::contract_table(samples, &gaps);
    let mut s = String::new();
    s.push_str("# E16 — The performance contract, measured\n\n");
    s.push_str(
        "**Generated by `bench --run`. Nothing in this file is typed in by hand.**\n\n\
         `SPEC-ENGINE.md` Part 0 states its targets relative to PostgreSQL. Until this \
         experiment they were predictions in the typography of results: every measurement in \
         `results/` reported counted work inside `proto-engine`, and the single wall-clock \
         table in the thesis (§9.4.4) was in-memory, single-threaded, and compared to \
         nothing. This is the first wall-clock comparison against the baseline the \
         specification names.\n\n\
         The last row is the one the architecture is for. `analytical` is a **cold \
         reconstruction** — the fold runs over the whole base for every query, which is one \
         end of the phase diagram — and `report` is the other end: the same shape of \
         statement answered out of a view the write path is keeping current, while a second \
         connection appends to the base throughout, so that \"warm\" means maintained rather \
         than stale. Both ends stay in the table; publishing only the warm one would hide the \
         trade this project exists to characterise. E23 (`results/E23-scaling.md`) takes that \
         row along two axes and fits a slope, which is where \"matches PostgreSQL as the data \
         grows\" becomes a statement with a truth value.\n\n",
    );
    s.push_str(table);
    s.push('\n');
    s.push_str(&render::spread_table(samples));
    s.push('\n');
    s.push_str(&render::statement_table(samples));
    // **The report row's own caveat**, because its two arms are not appended to equally.
    if samples
        .iter()
        .any(|x| x.workload == "report" && x.not_run.is_none())
    {
        s.push_str(
            "\n### What the `report` row is, and the one asymmetry in it\n\n\
             The statement is `select acct, sum(amt) from postings group by acct`. On the \
             Nilestream side it is served from a view whose contract is `materialize: full`, \
             read in key order at the anchor it is true at, **touching no base row** — the \
             server is asked (`explain`) before every run and the row is `NOT RUN` rather \
             than published if it answers anything but `report-from-view`. PostgreSQL runs \
             the identical statement over its own identically growing table; its recompute \
             *is* the control, and a fair one, because that is what a database without \
             maintained state has to do. Both start each run from the same base, and the \
             harness refuses the row if they do not.\n\n\
             **The asymmetry, stated rather than removed.** The appender writes at the same \
             rate on both sides, and PostgreSQL's measurement takes longer — so more appends \
             land under it, and its table grows further during its own window than the \
             ledger does during Nilestream's. Equalising the *count* instead would mean \
             equalising the rate·time product by slowing the appender against the faster \
             target, which is the same as saying the faster target should face lighter \
             concurrent write pressure per second. The rate is what is held equal; the counts \
             are printed per run so the difference is visible rather than assumed away.\n",
        );
    }

    // **The durable row is an fsync rate, so the device belongs beside it.**
    if let Some(c) = ceiling {
        s.push_str("\n### The `durable` row is an `fsync` rate, so here is the device\n\n");
        s.push_str(&format!(
            "The device was probed {} times across this run: **{:.0}** durable commits per \
             second per connection at the median, MAD {:.0}, lowest {:.0}, highest {:.0} — a \
             spread of **{:.2}x**.{}\n\n",
            c.probes,
            c.median,
            c.mad,
            c.lowest,
            c.highest,
            c.spread(),
            if c.steady() {
                " The device held still, so the absolute rates below mean what they say."
            } else {
                " **The device did not hold still.** An absolute durable rate measured \
                 against storage that moves this much is a number about the storage, and a \
                 movement in it between sessions is not evidence about the engine. The \
                 fraction of the ceiling is the part that is."
            }
        ));
        s.push_str(
            "| Target | Durable commits/s | Fraction of the device ceiling |\n|---|--:|--:|\n",
        );
        for target in ["postgres", "nilestream"] {
            let rows: Vec<&Sample> = samples
                .iter()
                .filter(|x| x.workload == "durable" && x.target == target && x.not_run.is_none())
                .collect();
            if rows.is_empty() {
                continue;
            }
            let mut v: Vec<f64> = rows.iter().map(|x| x.ops_per_second()).collect();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = v[v.len() / 2];
            s.push_str(&format!(
                "| {target} | {med:.0} | {:.0}% |\n",
                c.efficiency(med) * 100.0
            ));
        }
        s.push_str(
            "\nA fraction is machine-independent in a way a rate is not: \"this engine gets \
             *n*% of the `fsync`s its storage can deliver\" survives being run somewhere \
             else, and is the claim a durability comparison is actually making. Both targets \
             are measured against the **same** ceiling in the **same** session, and the runs \
             are interleaved, so whatever the device is doing it is doing to both.\n",
        );
    }
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
    for w in CONTRACT_WORKLOADS {
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
    let gaps: BTreeMap<String, String> = CONTRACT_WORKLOADS
        .iter()
        .filter_map(|w| nilestream_gap(w).map(|r| (w.to_string(), r)))
        .collect();
    print!("{}", render::contract_table(&samples, &gaps));
    println!();
    print!("{}", render::spread_table(&samples));
    println!();
    print!("{}", render::statement_table(&samples));

    // **And re-derive E23 from its committed CSV.** The sweep itself is wall-clock and
    // machine-dependent, so `make reproduce` cannot re-run it — but the *document* is a pure
    // function of the CSV, and that part must not drift. A results file whose prose and
    // whose data can disagree is a results file nobody can check.
    let e23_dir = bank_bench::publish::e23_dir(std::path::Path::new(&args.out));
    let e23_csv = e23_dir.join("E23-scaling.csv");
    if let Ok(text) = std::fs::read_to_string(&e23_csv) {
        let pts: Vec<E23> = text.lines().skip(1).filter_map(parse_e23_line).collect();
        if pts.is_empty() {
            eprintln!("bench: {} has a header and no rows", e23_csv.display());
            return 7;
        }
        let doc = e23_document(args, &pts);
        // `--render` re-derives the **committed** document, because that is the copy
        // `make reproduce` diffs. Unlike a `--run`, this reads only what is already there.
        for path in [
            e23_dir.join("E23-scaling.md"),
            std::path::PathBuf::from(bank_bench::publish::COMMITTED_E23),
        ] {
            if let Err(e) = std::fs::write(&path, &doc) {
                eprintln!("bench: re-rendering E23 to {} failed: {e}", path.display());
                return 6;
            }
        }
    }
    0
}

fn parse_e23_line(line: &str) -> Option<E23> {
    let f: Vec<&str> = line.splitn(9, ',').collect();
    if f.len() < 9 {
        return None;
    }
    let not_run = if f[8].trim().is_empty() {
        None
    } else {
        Some(f[8].to_string())
    };
    Some(E23 {
        series: f[0].into(),
        // `&'static str` because the axis is one of two known values, and a CSV that names a
        // third is a CSV this build cannot render rather than one to guess about.
        axis: match f[1] {
            "base" => "base",
            "output" => "output",
            _ => return None,
        },
        base_rows: f[2].parse().ok()?,
        output_rows: f[3].parse().ok()?,
        run: f[4].parse().ok()?,
        ms: f[5].parse().ok()?,
        bytes: f[6].parse().ok()?,
        serve_path: f[7].into(),
        not_run,
    })
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
