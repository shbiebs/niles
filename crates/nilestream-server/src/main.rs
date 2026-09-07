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
// The engine-mutex instrument. `daemon` times every acquisition through it; the binary
// itself never calls it, which is why it needs the allow.
#[allow(dead_code)]
#[path = "lockstats.rs"]
mod lockstats;
// The extended query protocol's plan cache. Reachable from the session as of this
// change; before it, the module existed and no listener referred to it.
mod extended;
#[path = "pg_wire.rs"]
mod pg_wire;
#[path = "rev_engine.rs"]
mod rev_engine;
// The keyed-aggregate fast path `rev_engine` takes before materialising anything.
#[path = "scan_fold.rs"]
mod scan_fold;
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
use std::sync::Arc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 5433u16;
    let mut schema_path: Option<String> = None;
    let mut accounts = 1000i64;
    let mut rounds = 3u32;
    let mut budget = 100_000usize;
    let mut full = false;
    let mut durable: Option<String> = None;
    let mut i = 1;
    while i + 1 < args.len() {
        match args[i].as_str() {
            "--port" => port = args[i + 1].parse().unwrap_or(port),
            "--schema" => schema_path = Some(args[i + 1].clone()),
            "--accounts" => accounts = args[i + 1].parse().unwrap_or(accounts),
            "--rounds" => rounds = args[i + 1].parse().unwrap_or(rounds),
            "--budget" => budget = args[i + 1].parse().unwrap_or(budget),
            "--mode" => full = args[i + 1] == "full",
            "--durable" => durable = Some(args[i + 1].clone()),
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
    // **`--durable <segment>` makes the shipped binary what the benchmark has been hosting.**
    //
    // Until this flag existed, `nilestreamd` could not attach a durable sink at all — the
    // only durable engine in the project was the one `bank-bench` builds in-process, and it
    // refuses to run without a PostgreSQL to compare against. So the shipped server's
    // durable throughput was undefined, and "how does Nilestream scale with connections"
    // could not be asked on a machine without PostgreSQL 16. Both of those are instrument
    // gaps rather than engine defects, and this is half of closing them.
    //
    // `SyncPolicy::Always` is the only policy `DurableSink::open` accepts, so there is no
    // flag here that quietly buys throughput by weakening the guarantee.
    // **The declared idempotency window, in epochs, from the schema this daemon compiled.**
    //
    // `idem: IdemKey window N.epochs` reached nothing below the compiler until T-05 (cycle
    // 8): both idempotency indexes kept every identity ever committed, at a measured 68.8 B
    // and 99.6 B each (E18), so the write path's memory was a function of history. A schema
    // that declares no window still keeps everything — the banner says which, because a
    // memory bound nobody can see is a memory bound nobody can act on.
    let idem_window = crate::session::declared_idem_window(&schema);
    match idem_window {
        Some(w) => eprintln!("  idempotency window: {w} epochs, from the schema"),
        None => eprintln!(
            "  idempotency window: none declared — every identity is kept, and the index \
             grows with history"
        ),
    }
    let base = RevEngine::seeded(
        accounts,
        rounds,
        budget,
        mode,
        proto_engine::EvictionPolicy::Lru,
    )
    .with_idem_window(idem_window);
    let base = match &durable {
        None => base,
        Some(path) => match base.with_durable_bounded(path, idem_window) {
            Ok(e) => {
                eprintln!("  durable sink at {path} — SyncPolicy::Always, fsync before publish");
                e
            }
            // Refuse rather than fall back to a volatile engine. A server that was asked for
            // durability and silently served without it is the single most direct way to
            // fabricate a durability number.
            Err(e) => {
                eprintln!("nilestreamd: cannot open a durable sink at {path}: {e}");
                std::process::exit(2);
            }
        },
    };
    let engine = Arc::new(base);

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
        eprintln!(
            "  read path: partial view ({:?}, budget {budget}) over a hash-chained ledger, \
             frontier #{}",
            mode,
            engine.frontier()
        );
    }
    if durable.is_none() {
        eprintln!("  NOTE: the read side is in-memory and single-threaded, with no durability and");
        eprintln!(
            "        no consensus. It serves the real REV mechanism -- partial state, honest"
        );
        eprintln!(
            "        absence, anchored reconstruction -- and it is not a production database."
        );
    } else {
        // **Two claims that were false when they were printed.** "Durable" meant the epoch
        // number reached stable storage, not the rows, so a restart recovered nothing (T-01);
        // and the engine mutex had been replaced by a reader-writer split two commits before
        // (T-06). A banner is the first thing an operator reads and the last thing anyone
        // re-checks.
        eprintln!("  NOTE: appends are durable -- rows are recorded and replayed on reopen --");
        eprintln!("        and reads run concurrently over the base. There is no consensus.");
        eprintln!("        `select nilestream_sealer` reports batching and lock contention.");
    }
    eprintln!("  try:  psql -h 127.0.0.1 -p {port} -U anyone bank");

    daemon::accept_loop(listener, schema, engine);
}
