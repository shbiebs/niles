//! **The crash protocol, as a test rather than as a procedure someone ran once.**
//!
//! T-01's finding was that `nilestreamd --durable` fsynced the idempotency key and the epoch
//! *number*, not the rows. Eight writers, 25,416 acknowledged inserts, `SIGKILL`, restart: the
//! frontier was back at the seed and every account had no balance. A record that cannot
//! reconstruct the base is a receipt, not a log.
//!
//! The repair was verified out of process, by hand, and it passed. That is the weakest evidence
//! this project accepts, and it has been wrong twice in one cycle: `verify_chain` was checked
//! only by a ten-minute experiment and broke silently for every epoch on every seed, and
//! `run4.sh`'s "after" arm was checked only by a comment and measured the wrong commit for a
//! whole cycle. So the protocol runs here, on every `cargo test`, against the **shipped binary**.
//!
//! # Why the binary and not the library
//!
//! An in-process test can drop a `RevEngine` and reopen the segment, and three already do. What
//! it cannot do is die. `SIGKILL` is the whole point: no destructor runs, no buffer is flushed,
//! no `Drop` gets a chance to make the file consistent. The difference between "the sink was
//! closed" and "the process was shot" is exactly where a durability bug lives, and only a real
//! process can be shot. `env!("CARGO_BIN_EXE_nilestreamd")` is the binary this package builds.
//!
//! # Why `psql`
//!
//! For `psql_conformance.rs`'s reason: an in-repo client is written against the same
//! understanding of the protocol as the server, so the two halves of one misunderstanding agree
//! perfectly. Here it matters for a second reason — the acknowledgement is the thing under test.
//! `INSERT 0 2` from a real libpq client is the server saying *committed*, and the claim is that
//! anything it said that about survives the kill.

use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Accounts the writers post to. Small, so a whole-base check is cheap and exact.
const ACCOUNTS: i64 = 8;
/// The counterparty every transfer balances against.
const SINK: i64 = 999_999;

fn skip_marker() -> bool {
    std::env::var("NILES_SKIP_PSQL").as_deref() == Ok("1")
}

fn psql(port: u16, sql: &str) -> (bool, String) {
    let out = Command::new("psql")
        .args([
            "-h",
            "127.0.0.1",
            "-p",
            &port.to_string(),
            "-U",
            "bench",
            "-d",
            "bank",
            "--no-psqlrc",
            "-A",
            "-t",
            "-c",
            sql,
        ])
        .env("PGCONNECT_TIMEOUT", "10")
        .env("PGPASSWORD", "")
        .output()
        .unwrap_or_else(|e| panic!("cannot run `psql`: {e}"));
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), s)
}

/// Start `nilestreamd --durable` on `port` over `segment`, and wait until it answers.
///
/// **A daemon that exits is reported as an exit, not as a timeout.** The first version of this
/// waited sixty seconds for a port that was never going to open and then said so — which is
/// true, useless, and slow. The interesting case is precisely a refusal to start: `with_durable`
/// is *supposed* to refuse a segment it cannot verify, so on the reverted payload the daemon
/// exits at once with its reason on stderr. Polling `try_wait` turns that from a minute of
/// nothing into the message the process actually printed.
fn start(port: u16, segment: &std::path::Path) -> Child {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nilestreamd"))
        .args([
            "--port",
            &port.to_string(),
            "--accounts",
            &ACCOUNTS.to_string(),
            "--rounds",
            "1",
            "--durable",
            segment.to_str().expect("utf-8 path"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the daemon binary must be built before this test runs");

    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("poll the daemon") {
            let mut why = String::new();
            if let Some(mut e) = child.stderr.take() {
                use std::io::Read;
                let _ = e.read_to_string(&mut why);
            }
            panic!(
 "the daemon exited ({status}) instead of serving on port {port}. It refused to open the segment, which is what `with_durable` does when a record does not verify — so this is a durability failure and not a startup one:\n{why}"
            );
        }
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok()
            // Listening is not the same as serving: the engine seeds and, on a reopen,
            // replays before the first query is answered.
            && psql(port, "select nilestream_frontier").0
        {
            return child;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    // The timeout path leaves a live process. Kill and reap it before failing, or the suite
    // leaks a daemon holding a port and the *next* test fails for a reason that is not its own.
    let _ = child.kill();
    let _ = child.wait();
    panic!("the daemon did not start serving on port {port} within 60s, and did not exit");
}

fn frontier(port: u16) -> u64 {
    let (ok, out) = psql(port, "select nilestream_frontier");
    assert!(ok, "frontier query failed: {out}");
    out.split('|')
        .next()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| panic!("cannot read a frontier from {out:?}"))
}

fn balance(port: u16, acct: i64) -> i128 {
    let (ok, out) = psql(
        port,
        &format!("select acct, cur, sum(amt) from postings where acct = {acct} group by acct, cur"),
    );
    assert!(ok, "balance query for {acct} failed: {out}");
    // `acct|cur|sum|anchor`, one row per currency; the seed and the writers both use currency 0.
    out.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| l.split('|').nth(2))
        .filter_map(|s| s.trim().parse::<i128>().ok())
        .sum()
}

/// An ephemeral port, taken by binding and releasing. Racy in principle; the window is
/// microseconds and the alternative is a fixed port that collides with a parallel test.
fn free_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

/// **The protocol.** Write until the daemon has acknowledged a few hundred transfers, record
/// what it acknowledged, `SIGKILL` it, restart on the same segment, and check the ledger against
/// the acknowledgements.
#[test]
fn acknowledged_rows_survive_a_sigkill() {
    if skip_marker() {
        panic!("NILES_SKIP_PSQL=1: the crash protocol did not run");
    }
    let dir = std::env::temp_dir().join(format!("niles-crash-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    let segment = dir.join("crash.seg");
    let _ = std::fs::remove_file(&segment);

    let port = free_port();
    let mut child = start(port, &segment);
    let seeded: Vec<i128> = (0..ACCOUNTS).map(|a| balance(port, a)).collect();

    // Drive the writes through one `psql` session so the acknowledgements are serial and the
    // last one is unambiguous: everything before it was acknowledged too.
    const ROUNDS: u64 = 120;
    let mut script = String::new();
    for i in 1..=ROUNDS {
        for a in 0..ACCOUNTS {
            let txn = 700_000_000 + i * 100 + a as u64;
            script.push_str(&format!(
                "insert into postings values ({txn}, {a}, 0, 1), ({txn}, {SINK}, 0, -1);\n"
            ));
        }
    }
    let mut p = Command::new("psql")
        .args([
            "-h",
            "127.0.0.1",
            "-p",
            &port.to_string(),
            "-U",
            "bench",
            "-d",
            "bank",
            "--no-psqlrc",
            "-A",
            "-t",
            "-v",
            "ON_ERROR_STOP=1",
            "-f",
            "-",
        ])
        .env("PGCONNECT_TIMEOUT", "10")
        .env("PGPASSWORD", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("psql");
    p.stdin
        .as_mut()
        .expect("stdin")
        .write_all(script.as_bytes())
        .expect("write the script");
    let out = p.wait_with_output().expect("psql finishes");
    let acks = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.starts_with("INSERT 0"))
        .count() as i128;
    assert!(
        acks > 100,
        "the protocol needs a body of acknowledged writes to be about anything: {acks} \
         acknowledged. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Every acknowledged statement posted +1 to each account in round-robin, so the expected
    // delta per account is the number of rounds that completed for it. Read it back from the
    // running server *before* the kill: that is what the client was told.
    let acked: Vec<i128> = (0..ACCOUNTS)
        .map(|a| balance(port, a) - seeded[a as usize])
        .collect();
    let live_frontier = frontier(port);
    assert!(
        acked.iter().all(|d| *d > 0),
        "every account must have been credited before the kill: {acked:?}"
    );

    // **The kill.** `Child::kill` is `SIGKILL` on Unix: no destructor, no flush, no `Drop`.
    child.kill().expect("kill the daemon");
    let _ = child.wait();

    // Reopen the same segment with a second process.
    let port2 = free_port();
    let mut child2 = start(port2, &segment);

    let recovered: Vec<i128> = (0..ACCOUNTS)
        .map(|a| balance(port2, a) - seeded[a as usize])
        .collect();

    for (a, (ack, rec)) in acked.iter().zip(recovered.iter()).enumerate() {
        assert!(
            rec >= ack,
            "account {a}: {rec} recovered against {ack} acknowledged. An acknowledged write \
             that is not in the reopened ledger is the defect T-01 exists to close — the \
             client was told `INSERT 0 2` and the row is gone."
        );
        // The other side: recovery must not invent rows. The slack is one in-flight batch,
        // which is bounded by the writes a single sealer batch can hold.
        assert!(
            rec - ack <= 64,
            "account {a}: {rec} recovered against {ack} acknowledged. More than a batch of \
             slack means replay applied something twice."
        );
    }

    assert!(
        frontier(port2) >= live_frontier,
        "the reopened frontier ({}) must be at or beyond the last one the running server \
         reported ({live_frontier})",
        frontier(port2)
    );

    // **A retry of an acknowledged identity is refused by the daemon**, not silently accepted
    // as new. Before T-01 the idempotency window survived a restart and the rows did not, so
    // this passed while the balances were empty — it is asserted *after* them for that reason.
    // The first identity the script wrote: round 1, account 0.
    let txn = 700_000_000 + 100;
    let (ok, out) = psql(
        port2,
        &format!("insert into postings values ({txn}, 0, 0, 1), ({txn}, {SINK}, 0, -1)"),
    );
    assert!(
        !ok && out.contains("already committed"),
        "a retried identity must be refused as a duplicate by the daemon: ok={ok} {out}"
    );

    child2.kill().expect("kill");
    let _ = child2.wait();
    let _ = std::fs::remove_dir_all(&dir);
}
