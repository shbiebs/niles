//! E13 — what durability and concurrency actually cost.
//!
//! Chapter 9 marks these cells *to be measured*, because the research prototype has neither
//! an fsync nor a thread. This measures them on the real write path: a durable, hash-chained,
//! epoch-ordered segment behind a concurrent sequencer.
//!
//! Two figures, and the second is the interesting one.
//!
//! * **The price of the guarantee.** `SyncPolicy::Always` against `Never`. `Never` is not a
//!   configuration anyone should run a ledger under; it is here to make the cost of the
//!   guarantee visible by removing it.
//! * **Whether a single sealer is a ceiling.** Transactions per fsync, as concurrency rises.
//!   If group commit works, this figure grows with load, and the per-transaction cost of
//!   durability falls exactly when the system is busiest.
//!
//! Unlike the counted-work figures elsewhere in this thesis, these are wall-clock and
//! therefore machine-dependent. They are reported as ratios wherever a ratio is meaningful.

use nilestream_ledger::frontiers::Frontier;
use nilestream_ledger::segment::{Segment, SyncPolicy};
use nilestream_ledger::sequencer::{Sequencer, Txn};
use std::sync::Arc;
use std::time::Instant;

fn run(threads: usize, per_thread: usize, policy: SyncPolicy, tag: &str) -> (f64, f64, u64) {
    let mut path = std::env::temp_dir();
    path.push(format!("niles-bench-{tag}-{}.log", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let (segment, _) = Segment::open(&path, policy).unwrap();
    let seq = Arc::new(Sequencer::start(segment, Frontier::new(), policy));

    let start = Instant::now();
    let mut handles = Vec::new();
    for t in 0..threads {
        let s = Arc::clone(&seq);
        handles.push(std::thread::spawn(move || {
            for i in 0..per_thread {
                // A realistic payload: two postings, sixteen bytes each.
                let _ = s.submit(Txn {
                    idem_key: format!("{t}-{i}"),
                    payload: vec![0u8; 32],
                });
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let elapsed = start.elapsed().as_secs_f64();
    let st = seq.stats();
    let _ = std::fs::remove_file(&path);
    (
        st.txns_committed as f64 / elapsed,
        st.txns_per_fsync(),
        st.max_batch,
    )
}

fn main() {
    let per_thread = 2_000;
    println!("# E13 — the cost of durability and the shape of group commit");
    println!();
    println!("Wall-clock, machine-dependent. Ratios are the transferable part.");
    println!();
    println!("| threads | policy | txns/sec | txns per fsync | largest batch |");
    println!("|---:|---|---:|---:|---:|");

    let mut always: Vec<f64> = Vec::new();
    let mut never: Vec<f64> = Vec::new();
    let concurrencies = [1usize, 2, 4, 8, 16];

    for &t in &concurrencies {
        let (tps, tpf, batch) = run(t, per_thread, SyncPolicy::Always, &format!("a{t}"));
        always.push(tps);
        println!("| {t} | Always (ledger-grade) | {tps:.0} | {tpf:.1} | {batch} |");
    }
    for &t in &concurrencies {
        let (tps, tpf, batch) = run(t, per_thread, SyncPolicy::Never, &format!("n{t}"));
        never.push(tps);
        println!("| {t} | Never (**not a ledger**) | {tps:.0} | {tpf:.1} | {batch} |");
    }

    println!();
    println!("## The price of the guarantee");
    println!();
    println!("| threads | cost of durability (x slower) |");
    println!("|---:|---:|");
    for (i, &t) in concurrencies.iter().enumerate() {
        println!("| {t} | {:.2}x |", never[i] / always[i].max(1.0));
    }
    println!();
    println!("If the ratio falls as threads rise, group commit is amortising the fsync and a");
    println!("single sealer is a batching opportunity rather than the ceiling it appears to be.");
    println!("If it stays flat, the fsync is being paid per transaction and the design needs");
    println!("revisiting — which is the outcome that would matter, so it is stated first.");
}
