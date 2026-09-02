//! **E16, the Nilestream half.** The point-lookup workload against the read-model runtime.
//!
//! # Why this is a test rather than a flag on the benchmark binary
//!
//! The harness drives PostgreSQL by connecting *out* to a server somebody else started. It
//! cannot start Nilestream the same way in every environment — a sandbox that reaps
//! background processes, or forbids a binary from binding a listener, leaves the Nilestream
//! column permanently `NOT RUN` for a reason that has nothing to do with the engine.
//!
//! So the Nilestream half runs here: the listener is hosted on a thread of the test process,
//! and the workload is driven through **the same `wire::Client`** the benchmark uses for
//! PostgreSQL. `cargo test -p bank-bench` reproduces it anywhere the repository builds.
//!
//! **The fairness rule is unchanged, and the distinction matters.** Both targets pay the same
//! *client* cost: socket, protocol framing, round trip. Hosting the listener on a thread
//! changes where the server runs, not how the client talks to it. Calling `RevEngine::read`
//! directly would have been the shortcut — this is precisely what avoids it, and it is why the
//! test goes through a TCP port rather than a function call.
//!
//! The samples are written to `results/E16-wallclock/point.csv` alongside PostgreSQL's, and
//! `bench --render` builds the table from both.

use bank_bench::wire::Client;
use nilestream_server::daemon;
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Serving;
use proto_engine::{EvictionPolicy, ViewMode};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Host the daemon on a thread and return the port it is listening on.
///
/// Port 0 asks the kernel for a free one, which matters more than it looks: a fixed port makes
/// a test fail when a previous run's server is still holding it, and — worse — makes it *pass*
/// against that stale server, measuring an older build without saying so.
type Hosted = (u16, u64, Arc<Mutex<RevEngine>>);

fn host(accounts: i64, rounds: u32, budget: usize) -> Hosted {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind an ephemeral port");
    let port = listener.local_addr().expect("addr").port();
    let engine = Arc::new(Mutex::new(RevEngine::seeded(
        accounts,
        rounds,
        budget,
        ViewMode::Demand,
        EvictionPolicy::Lru,
    )));
    let frontier = engine.lock().unwrap().frontier();
    let schema = daemon::DEFAULT_SCHEMA.to_string();
    let observer = Arc::clone(&engine);
    std::thread::spawn(move || daemon::accept_loop(listener, schema, engine));

    // Wait for the listener rather than sleeping a fixed time.
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    (port, frontier, observer)
}

fn percentiles(mut v: Vec<Duration>) -> (Duration, Duration) {
    v.sort_unstable();
    let at = |q: f64| v[(((v.len() as f64 - 1.0) * q).round() as usize).min(v.len() - 1)];
    (at(0.50), at(0.99))
}

/// Run the point workload and return `(ops_per_second, p50, p99)`.
fn point_workload(
    port: u16,
    accounts: i64,
    operations: u64,
    seed: u64,
) -> (f64, Duration, Duration) {
    let mut c = Client::connect("127.0.0.1", port, "bench", "bank").expect("connect");
    let mut rng = bank_bench::workloads::Rng::seeded(seed);
    let mut latencies = Vec::with_capacity(operations as usize);
    let started = Instant::now();
    for _ in 0..operations {
        // The same access pattern the benchmark's `point` workload uses: 90% of lookups in
        // the hottest 1% of accounts, seeded and reproducible.
        let key = rng.skewed_key(accounts, 0.9);
        let sql = format!("select acct, sum(amt) from postings where acct = {key} group by acct");
        let at = Instant::now();
        c.simple(&sql).expect("query");
        latencies.push(at.elapsed());
    }
    let wall = started.elapsed();
    let (p50, p99) = percentiles(latencies);
    (operations as f64 / wall.as_secs_f64(), p50, p99)
}

#[test]
fn the_read_path_answers_over_the_wire_from_a_partial_view() {
    // The claim this whole file exists to make measurable: a wire query is answered by the
    // REV mechanism, not by a hash map, and the answer is a real fold over a real ledger.
    let (port, frontier, _) = host(1_000, 2, 100_000);
    assert!(frontier > 0, "the ledger was seeded");

    let mut c = Client::connect("127.0.0.1", port, "bench", "bank").expect("connect");
    let rows = c
        .simple("select acct, sum(amt) from postings where acct = 42 group by acct")
        .expect("query");
    let value = rows.by_name("value").expect("account 42 has postings");
    assert!(value != 0, "a real balance: {value}");

    // And an account the base never touched comes back as a **NULL**, not a zero and not a
    // missing row. That distinction is the absence lattice reaching the wire intact, and it is
    // stronger than what PostgreSQL's `group by` does with the same query: PostgreSQL returns
    // no row at all, which tells a client "nothing matched" and leaves it to decide whether
    // that means the account is empty or absent. Nilestream answers the question that was
    // asked — this key, at this anchor, has no value — which is the §1.1.1 defect closed at
    // the protocol boundary where it would otherwise be quietest to lose.
    let missing = c
        .simple("select acct, sum(amt) from postings where acct = 999999 group by acct")
        .expect("query");
    assert_eq!(
        missing.rows.len(),
        1,
        "the key is answered, not silently dropped"
    );
    assert_eq!(missing.rows[0][1], None, "and the answer is NULL, not 0");
    assert_eq!(
        missing.by_name("value"),
        None,
        "which the client reads as no value"
    );
}

#[test]
fn eviction_does_not_change_an_answer_served_over_the_wire() {
    // Contribution 1, exercised end to end: eviction followed by anchored reconstruction can
    // neither create nor destroy money. Checked through the protocol rather than in-process,
    // because that is where a caching layer would hide the difference.
    let (port, _, _) = host(500, 2, 50); // a budget far below the key count, so eviction bites
    let mut c = Client::connect("127.0.0.1", port, "bench", "bank").expect("connect");

    let read = |c: &mut Client, a: i64| -> Option<i128> {
        c.simple(&format!(
            "select acct, sum(amt) from postings where acct = {a} group by acct"
        ))
        .expect("query")
        .by_name("value")
    };

    let first: Vec<Option<i128>> = (1..=200).map(|a| read(&mut c, a)).collect();
    // Touch every other key, forcing the budget to evict much of what was just read.
    for a in 201..=500 {
        read(&mut c, a);
    }
    let second: Vec<Option<i128>> = (1..=200).map(|a| read(&mut c, a)).collect();
    assert_eq!(
        first, second,
        "eviction and reconstruction changed an answer"
    );
}

/// The measurement itself. Writes `results/E16-wallclock/point.csv` rows for Nilestream.
///
/// Ignored by default: it takes seconds rather than milliseconds, and a test suite that runs
/// on every build should not be a benchmark. Run it explicitly:
///
/// ```sh
/// cargo test --release -p bank-bench --test e16_nilestream -- --ignored --nocapture
/// ```
#[test]
#[ignore = "a measurement, not a check: run explicitly with --ignored"]
fn e16_point_workload_against_the_rev_runtime() {
    const ACCOUNTS: i64 = 10_000;
    const OPERATIONS: u64 = 2_000;
    const RUNS: u32 = 10;

    let (port, frontier, engine) = host(ACCOUNTS, 2, 100_000);
    eprintln!("hosting on 127.0.0.1:{port}, ledger frontier #{frontier}");

    let mut lines = Vec::new();
    for run in 1..=RUNS {
        let before = engine.lock().unwrap().stats();
        let (ops, p50, p99) = point_workload(port, ACCOUNTS, OPERATIONS, 0xB0A7 ^ run as u64);
        let after = engine.lock().unwrap().stats();
        // **The miss rate belongs beside the latency.** A parity result at a 0% miss rate and
        // one at a 40% miss rate are different findings — the first says a warm view is fast,
        // the second says reconstruction is — and the phase diagram of thesis §9.3 is built
        // from exactly that difference. Reporting only the latency would let the more
        // interesting of the two disappear.
        let reads = (after.reads - before.reads).max(1);
        let misses = after.misses - before.misses;
        let rows = after.rows_touched - before.rows_touched;
        eprintln!(
            "  point run {run} — {ops:.0} ops/s, p99 {:.0}µs, miss {:.1}% ({misses}/{reads}), \
             {rows} base rows touched, {} resident",
            p99.as_nanos() as f64 / 1000.0,
            misses as f64 / reads as f64 * 100.0,
            after.resident
        );
        lines.push(format!(
            "point,nilestream,{run},{OPERATIONS},{:.3},{:.1},{:.1},{:.1},false,",
            OPERATIONS as f64 / ops * 1000.0,
            p50.as_nanos() as f64 / 1000.0,
            p99.as_nanos() as f64 / 1000.0,
            ops
        ));
    }

    // Merge into the CSV beside PostgreSQL's rows, replacing any previous Nilestream rows so
    // a re-run does not accumulate stale measurements under the same name.
    //
    // Resolved from `CARGO_MANIFEST_DIR` rather than from the working directory, because
    // `cargo test` runs with the *package* directory as its cwd. Writing to a relative path
    // put a second `results/` tree under `crates/bank-bench/`, where `bench --render` never
    // looked — so the table went on reporting `NOT RUN` while a perfectly good measurement sat
    // ten directories away. A quiet failure, and exactly the kind this file exists to prevent.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("the workspace root is two levels above this crate")
        .to_path_buf();
    let path = root.join("results/E16-wallclock/point.csv");
    let path = path.as_path();
    let existing = std::fs::read_to_string(path)
        .unwrap_or_else(|_| format!("{}\n", bank_bench::render::CSV_HEADER));
    let mut out = String::new();
    for line in existing.lines() {
        if !line.starts_with("point,nilestream,") {
            out.push_str(line);
            out.push('\n');
        }
    }
    for l in &lines {
        out.push_str(l);
        out.push('\n');
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(path, out).expect("write point.csv");
    eprintln!(
        "wrote {} Nilestream rows to {}",
        lines.len(),
        path.display()
    );
}
