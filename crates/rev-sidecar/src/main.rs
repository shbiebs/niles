//! `rev-sidecar` — the process of arms H3 and T. See the crate documentation.
//!
//! ```text
//! rev-sidecar --listen 127.0.0.1:5462 --upstream-port 5461 --budget 50 --loaded-head 50
//!             [--slot h3] [--publication h3]
//! rev-sidecar --listen 127.0.0.1:5465 --ledger driver:127.0.0.1:5464 --budget 50 --loaded-head 50
//! ```
//!
//! The second form is arm T: TigerBeetle is the ledger, reached through the line protocol of
//! `tools/arms/tigerbeetle/driver.py`, and its change stream arrives from RabbitMQ through
//! the same driver.

use nilestream_core::rev::{Policy, Runtime};
use rev_sidecar::base::PgBase;
use rev_sidecar::stream::{self, Shared};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// The view's name: the one Nilestream's daemon installs, from the same text.
const VIEW: &str = "__balance";

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn fail(msg: &str) -> ! {
    eprintln!("rev-sidecar: {msg}");
    std::process::exit(2)
}

/// The balance view, compiled from Niles text exactly as `nilestreamd` compiles its own
/// (`nilestream_server::rev_engine`): the default schema, `sum(amt)` per `(acct, cur)`,
/// `consistency: snapshot, materialize: auto`, a residency budget, LRU.
fn install(budget: u64) -> Runtime {
    const BALANCE: &str = "select acct, cur, sum(amt) from postings group by acct, cur";
    let program = format!(
        "{}\nview {VIEW} = sql {{ {BALANCE} }} serve {{ consistency: snapshot, materialize: auto }};\n",
        nilestream_server::daemon::DEFAULT_SCHEMA
    );
    let (prog, d) = niles_lang::parser::parse_program(&program);
    if d.has_errors() {
        fail(&format!("the view does not parse: {d:?}"));
    }
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    if rd.has_errors() {
        fail(&format!("the view does not resolve: {rd:?}"));
    }
    let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
    if ld.has_errors() || !niles_ir::verify::verify(&lowered.circuit).is_ok() {
        fail("the view does not lower and verify");
    }
    Runtime::install(lowered.circuit, Some(budget), Policy::Lru)
        .unwrap_or_else(|e| fail(&format!("the runtime refused the view: {}", e.explain())))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let listen = arg(&args, "--listen").unwrap_or_else(|| fail("--listen host:port"));
    // `--ledger driver:HOST:PORT` is arm T (TigerBeetle behind tools/arms/tigerbeetle's
    // driver); otherwise the ledger is PostgreSQL on `--upstream-port`.
    let driver = arg(&args, "--ledger").and_then(|l| l.strip_prefix("driver:").map(String::from));
    let port: u16 = match &driver {
        Some(_) => 0,
        None => arg(&args, "--upstream-port")
            .and_then(|p| p.parse().ok())
            .unwrap_or_else(|| fail("--upstream-port N or --ledger driver:HOST:PORT")),
    };
    let budget: u64 = arg(&args, "--budget")
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| fail("--budget N"));
    let loaded: u64 = arg(&args, "--loaded-head")
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| fail("--loaded-head E"));
    let slot = arg(&args, "--slot").unwrap_or_else(|| "h3".into());
    let publication = arg(&args, "--publication").unwrap_or_else(|| "h3".into());

    let mut rt = install(budget);
    let plan = rt.view(VIEW).expect("installed").plan().clone();
    if plan.group_key.len() != 2 {
        fail(&format!("the view's key is not (acct, cur): {plan}"));
    }
    let base = Arc::new(match &driver {
        Some(addr) => PgBase::over_driver(plan, addr, loaded),
        None => PgBase::new(plan, port, loaded),
    });
    // Nothing is resident, so the loaded epochs carry no deltas this view could need: the
    // view starts certified through the loaded head, and the base says its deltas begin
    // after it (`deltas_available_from`), so no merge can reach behind the stream's start.
    rt.advance(base.as_ref(), loaded);
    let shared = Arc::new(Shared {
        base,
        rt: Mutex::new(rt),
        view: VIEW.into(),
    });
    let s2 = Arc::clone(&shared);
    match driver {
        Some(addr) => {
            std::thread::spawn(move || {
                if let Err(e) = stream::run_driver(&addr, s2) {
                    fail(&format!("change stream: {e}"));
                }
                fail("change stream ended");
            });
        }
        None => {
            let conn = stream::open(port, &slot, &publication).unwrap_or_else(|e| fail(&e));
            std::thread::spawn(move || {
                if let Err(e) = stream::run(conn, s2) {
                    // A sidecar whose stream has failed would serve a view frozen at some
                    // epoch while the ledger moves on: stop, so every later read is a
                    // connection error.
                    fail(&format!("replication stream: {e}"));
                }
                fail("replication stream ended");
            });
        }
    }
    let listener =
        TcpListener::bind(&listen).unwrap_or_else(|e| fail(&format!("bind {listen}: {e}")));
    println!("rev-sidecar ready on {listen}, frontier #{loaded}");
    rev_sidecar::server::serve(listener, shared);
}
