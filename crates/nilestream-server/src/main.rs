// See lib.rs: the wire surface is incomplete by construction today (no write path, the
// extended query protocol unwired), so several items have no caller yet. Kept rather than
// deleted so the gap stays visible.
#![allow(dead_code)]
// `Backend::BackendKeyData` is the PostgreSQL message name.
#![allow(clippy::enum_variant_names)]

//! `nilestreamd` — the Nilestream daemon.
//!
//! Listens on TCP, speaks the PostgreSQL wire protocol, compiles each query as Niles, and
//! serves it from the REV runtime over a durable ledger. One thread per connection, which
//! is the right shape at this scale and is stated rather than defended: the write path is
//! serialised through a single sealer anyway, so connection-level concurrency is about
//! *reads*, and reads over an immutable base need no coordination at all.
//!
//! ```text
//! nilestreamd [--port N] [--schema FILE] [--accounts N] [--rounds N]
//!             [--budget N] [--mode demand|full]
//! ```
//!
//! # What serves a read
//!
//! Reads are answered by a **partial view over an immutable, hash-chained ledger**
//! (`rev_engine`): partial materialisation, the absence lattice, an anchored upquery on a
//! miss. The daemon used to answer from a `HashMap` and say so in this banner, which was
//! honest and made it unmeasurable — the E16 wall-clock harness could compare it to
//! PostgreSQL but the resulting `PARITY` measured the protocol path rather than the engine.
//!
//! `--budget` and `--mode` are the levers the phase diagram is swept with: a parity result at
//! a 0% miss rate and one at a 40% miss rate are different findings, and a server that could
//! only be run warm would only ever produce the flattering half.
//!
//! Point `psql -h 127.0.0.1 -p 5433 -U anyone bank` at it.

#[path = "daemon.rs"]
mod daemon;
#[path = "pg_wire.rs"]
mod pg_wire;
#[path = "rev_engine.rs"]
mod rev_engine;
#[path = "session.rs"]
mod session;
// The daemon uses one policy (`insecure`) and one negotiation, so most of `tls.rs` is dead
// code *in this binary* while being live in the library and exercised by its 17 tests. The
// alternative — trimming the module to what the binary happens to call — would delete the
// policy layer that exists so a deployment can turn TLS on with one line.
#[allow(dead_code)]
#[path = "tls.rs"]
mod tls;

use rev_engine::RevEngine;
use session::Serving;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 5433u16;
    let mut schema_path: Option<String> = None;
    let mut accounts = 1000i64;
    let mut rounds = 3u32;
    let mut budget = 100_000usize;
    let mut full = false;
    let mut i = 1;
    while i + 1 < args.len() {
        match args[i].as_str() {
            "--port" => port = args[i + 1].parse().unwrap_or(port),
            "--schema" => schema_path = Some(args[i + 1].clone()),
            "--accounts" => accounts = args[i + 1].parse().unwrap_or(accounts),
            "--rounds" => rounds = args[i + 1].parse().unwrap_or(rounds),
            "--budget" => budget = args[i + 1].parse().unwrap_or(budget),
            "--mode" => full = args[i + 1] == "full",
            _ => {}
        }
        i += 2;
    }

    let schema = match &schema_path {
        Some(p) => std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("nilestreamd: cannot read {p}: {e}");
            std::process::exit(2);
        }),
        None => daemon::DEFAULT_SCHEMA.to_string(),
    };

    // Refuse to start on a schema that does not compile. A server that accepted a broken
    // schema would fail on the first query instead, at which point the operator is
    // debugging a connection rather than a file.
    let (prog, mut d) = niles_lang::parser::parse_program(&schema);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    d.extend(rd);
    if d.has_errors() {
        eprint!(
            "{}",
            d.render(&schema, schema_path.as_deref().unwrap_or("<default>"))
        );
        eprintln!("nilestreamd: the schema does not compile; refusing to start");
        std::process::exit(1);
    }

    // The read side: a partial view over an immutable ledger, seeded with `rounds` balanced
    // transfers per account. Seeded rather than empty because a benchmark against an empty
    // server reports excellent latencies for queries that return nothing, and because a view
    // over a base with no history never exercises the miss path — which is the interesting
    // one.
    let mode = if full {
        proto_engine::ViewMode::Full
    } else {
        proto_engine::ViewMode::Demand
    };
    let engine = Arc::new(Mutex::new(RevEngine::seeded(
        accounts,
        rounds,
        budget,
        mode,
        proto_engine::EvictionPolicy::Lru,
    )));

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("nilestreamd: cannot bind 127.0.0.1:{port}: {e}");
            std::process::exit(2);
        }
    };
    eprintln!("nilestreamd 0.1 — PostgreSQL wire protocol on 127.0.0.1:{port}");
    eprintln!(
        "  schema: {} ({} view(s), {} relation(s))",
        schema_path.as_deref().unwrap_or("<default>"),
        cat.views.len(),
        cat.relations.len()
    );
    {
        let e = engine.lock().unwrap();
        eprintln!(
            "  read path: partial view ({:?}, budget {budget}) over a hash-chained ledger, \
             frontier #{}",
            mode,
            e.frontier()
        );
    }
    eprintln!("  NOTE: the read side is in-memory and single-threaded, with no durability and");
    eprintln!("        no consensus. It serves the real REV mechanism -- partial state, honest");
    eprintln!("        absence, anchored reconstruction -- and it is not a production database.");
    eprintln!("  try:  psql -h 127.0.0.1 -p {port} -U anyone bank");

    daemon::accept_loop(listener, schema, engine);
}
