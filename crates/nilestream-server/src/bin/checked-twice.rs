//! **A counting tool: what the engine re-establishes that the compiler already proved.**
//!
//! Not a benchmark. It prints nothing but a statement count and drives a fixed, deterministic
//! workload through `Session::handle` so that `callgrind` can attribute *instructions per
//! function* to it. Whole-process instruction totals move with allocator luck; per-function
//! inclusive counts do not, which is why this exists as its own binary rather than as a flag
//! on `bench`.
//!
//! The question it was built for (F-68): the compiler resolves a schema, proves what it
//! proves, and hands the session a lowering — and then `Session::insert` used to run
//! `parse_program` and `resolve_program` over the whole schema text *again*, on every
//! `INSERT`, to learn the declared currencies. 74,500 of 113,500 instructions per insert.
//!
//! ```sh
//! cargo build --release -p nilestream-server --bin checked-twice
//! valgrind --tool=callgrind --callgrind-out-file=cg.out \
//!     ./target/release/checked-twice oltp 2000
//! callgrind_annotate --inclusive=yes cg.out | head -40
//! ```
//!
//! Three shapes, because they exercise different halves:
//!
//! * `oltp` — a two-leg transfer, the write path. E16's `oltp` statement.
//! * `point` — a keyed read over a hot set of a hundred accounts, so the plan cache hits.
//! * `point-cold` — the same read over the whole key space, so it misses and the front end
//!   runs. The difference between the two is what a cached plan is worth.
//!
//! Subtract the `seed` shape's count (the same binary with zero statements) to get the
//! workload's own instructions rather than the process's.

use nilestream_server::pg_wire::Frontend;
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

fn main() {
    let shape = std::env::args().nth(1).unwrap_or_else(|| "point".into());
    let n: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2_000);

    // E16's committed configuration, so a count here and a wall-clock figure there describe
    // the same engine.
    let engine = RevEngine::seeded(10_000, 1, 2_500, ViewMode::Demand, EvictionPolicy::Lru);
    let mut session = Session::new(
        "bench".into(),
        "bank".into(),
        nilestream_server::daemon::DEFAULT_SCHEMA.into(),
    );

    // A fixed LCG: the same statements in the same order on every host and every run.
    let mut x = 0x9E37u64;
    let mut messages = 0usize;
    for i in 0..n {
        x = x
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let sql = match shape.as_str() {
            "seed" => break,
            "point" => {
                let k = 1 + (x >> 33) % 100;
                format!("select acct, sum(amt) from postings where acct = {k} group by acct")
            }
            "point-cold" => {
                let k = 1 + (x >> 33) % 10_000;
                format!("select acct, sum(amt) from postings where acct = {k} group by acct")
            }
            "oltp" => {
                let from = 1 + (x >> 33) % 100;
                let to = 1 + (x >> 40) % 100;
                let txn = 1_000_000 + i;
                format!("insert into postings values ({txn}, {from}, 0, -7), ({txn}, {to}, 0, 7)")
            }
            other => {
                eprintln!(
                    "checked-twice: `{other}` is not a shape. One of: seed, point, \
                     point-cold, oltp."
                );
                std::process::exit(2);
            }
        };
        messages += session.handle(Frontend::Query(sql), &engine).len();
        // The receipts belong to this session and there is no durable sink here, so there is
        // nothing to wait on; taking them keeps the loop honest about the shape of `serve`.
        let _ = session.take_receipts();
    }
    println!(
        "{shape}: {n} statements, {messages} backend messages, compile misses {}, hits {}, \
         schema parses {}",
        session.compile_misses, session.compile_hits, session.schema_parses
    );
}
