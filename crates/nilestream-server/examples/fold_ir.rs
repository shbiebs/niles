//! **One served `group by acct`, and nothing else, so `callgrind` can count it.**
//!
//! Allocation counts are what the E18 gate asserts, because they are deterministic. Where
//! they do not settle a question — whether a change moved *work* rather than moving it from
//! the heap to the stack — the deterministic answer is an instruction count, and that needs a
//! process that does one thing.
//!
//! `tools/memprobe`'s binary runs eight scenarios over forty thousand postings; under
//! `valgrind` that is minutes, and the profile mixes the fold with the ledger seeding. This
//! serves a single statement against the same engine E16 and E18 are configured for, so the
//! numbers in `docs/BENCHMARK.md` refer to something a reader can re-run in seconds.
//!
//! ```sh
//! cargo build --release -p nilestream-server --example fold_ir
//! valgrind --tool=callgrind --callgrind-out-file=/tmp/fold.out \
//!     ./target/release/examples/fold_ir
//! callgrind_annotate --inclusive=yes /tmp/fold.out | grep -E 'Folder::row|Folder::finish'
//! ```

use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Serving;
use proto_engine::{EvictionPolicy, ViewMode};

/// E18's committed configuration, so one profile describes one engine. The rounds are
/// overridable as a second argument because E16's committed recipe seeds one pair per account
/// and E18's scenarios seed two, and an instruction count that did not say which is a figure
/// for an unnamed base.
const ACCOUNTS: i64 = 10_000;
const ROUNDS: u32 = 2;
const BUDGET: usize = 2_500;

fn main() {
    let sql = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "select acct, sum(amt) from postings group by acct".into());
    let program = format!(
        "{}\nview __wire_result = sql {{ {sql} }} serve {{ consistency: snapshot, materialize: auto }};\n",
        nilestream_server::daemon::DEFAULT_SCHEMA
    );
    let (prog, d) = niles_lang::parser::parse_program(&program);
    assert!(!d.has_errors(), "{:?}", d.items);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    assert!(!rd.has_errors(), "{:?}", rd.items);
    let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
    assert!(!ld.has_errors(), "{:?}", ld.items);

    let rounds = std::env::args()
        .nth(2)
        .and_then(|r| r.parse().ok())
        .unwrap_or(ROUNDS);
    let mut e = RevEngine::seeded(
        ACCOUNTS,
        rounds,
        BUDGET,
        ViewMode::Demand,
        EvictionPolicy::Lru,
    );
    let anchor = e.frontier();
    // Once to warm whatever the allocator and the page tables need, then the timed one. The
    // wall figure is an advisory — the gate asserts allocations, which are deterministic —
    // but an in-process time is what says whether an instruction count moved anything a
    // client would feel.
    let _ = e.query(&lowered.circuit, "__wire_result", anchor);
    let at = std::time::Instant::now();
    let rows = e
        .query(&lowered.circuit, "__wire_result", anchor)
        .expect("serves");
    let took = at.elapsed();
    // Printed so the optimiser cannot delete the query, and so a reader can see the profile
    // covered the shape they think it did.
    println!(
        "{} rows, {} columns, {} accounts x {rounds} rounds, {:.2} ms in process",
        rows.rows.len(),
        rows.columns.len(),
        ACCOUNTS,
        took.as_secs_f64() * 1000.0
    );
}
