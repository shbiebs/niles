//! **E18 — memory, counted rather than described.**
//!
//! Before this module the project had no memory instrument at all: no allocation counter, no
//! bytes-per-posting figure, no threshold in any gate. "Highly efficient in memory" was a
//! goal with nothing to measure it against, so a regression could not be caught and an
//! improvement could not be claimed.
//!
//! # Why allocation counts and not a profiler
//!
//! The counters here are **deterministic**. Two runs of the same scenario on the same build
//! allocate the same number of times and the same number of bytes, so a threshold can gate a
//! merge and a difference is a change in the program rather than in the machine. Wall clock
//! on a two-core container varies by tens of percent between invocations of the same probe;
//! nothing in this file reports it, and `results/E18-memory.md` therefore contains no number
//! that depends on the host.
//!
//! `valgrind --tool=dhat` and `--tool=massif` answer a different and also useful question —
//! where the bytes are, and how the heap moves over time — and the commands are recorded in
//! `docs/BENCHMARK.md`. They are not run in a gate because their output is neither
//! deterministic in the same way nor cheap.
//!
//! # What the counters mean
//!
//! * **allocations** — calls to `alloc` and `realloc`. A `Vec` that grows five times counts
//!   five, which is the point: it is the cost that scales with a row rather than with a query.
//! * **bytes** — requested bytes, summed over those calls. Not resident set: the allocator's
//!   own bookkeeping and its free lists are excluded, deliberately, because they are a
//!   property of the allocator and this measures the program.
//! * **live** — requested minus freed, at the moment of reading. The closest thing here to
//!   "how much does this data structure cost to hold".
//! * **peak live** — the high-water mark since the last reset. What decides whether a query
//!   fits, which a total-bytes figure cannot say.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

pub static ALLOCS: AtomicUsize = AtomicUsize::new(0);
pub static BYTES: AtomicUsize = AtomicUsize::new(0);
pub static LIVE: AtomicUsize = AtomicUsize::new(0);
pub static PEAK: AtomicUsize = AtomicUsize::new(0);

/// The system allocator, counted.
///
/// Installed by the *binary* — a library must not choose a program's allocator — under the
/// `count-alloc` feature, and by the budget test in its own crate. `installed()` is how a
/// caller finds out whether the counters mean anything, so a run without the feature refuses
/// rather than reporting zeros.
pub struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(l.size(), Relaxed);
        let live = LIVE.fetch_add(l.size(), Relaxed) + l.size();
        PEAK.fetch_max(live, Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Relaxed);
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(n, Relaxed);
        let live = LIVE.fetch_add(n, Relaxed) + n;
        LIVE.fetch_sub(l.size(), Relaxed);
        PEAK.fetch_max(live, Relaxed);
        System.realloc(p, l, n)
    }
}

/// Whether the counting allocator is the one this program is using.
///
/// Probed rather than assumed: without `--features count-alloc` the binary installs nothing,
/// every counter reads zero, and a table of zeros looks exactly like a program that allocates
/// nothing. E18 refuses to write results when this is false.
pub fn installed() -> bool {
    let before = ALLOCS.load(Relaxed);
    let v: Vec<u8> = Vec::with_capacity(4096);
    std::hint::black_box(&v);
    drop(v);
    ALLOCS.load(Relaxed) > before
}

/// A reading of the counters, taken around a region.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counted {
    pub allocations: usize,
    pub bytes: usize,
    /// Bytes still held when the region ended: what the region *kept*.
    pub live_delta: isize,
    /// High-water mark above the live bytes at entry.
    pub peak_delta: usize,
}

/// Count the allocations a closure performs.
///
/// `live_delta` is zero for a region that holds nothing and non-zero for one whose result is
/// retained, which is what separates "this query touched 30MB" from "this structure costs
/// 30MB to hold".
///
/// **The process must be effectively single-threaded across the region.** A
/// `#[global_allocator]` is process-wide, so another thread allocating during the region is
/// counted into it — silently, and by a different amount each run. The E18 binary is
/// single-threaded; the budget test runs under `--test-threads=1` and checks a known-quiet
/// region first, so contamination is an instruction rather than a wrong number.
pub fn count<T>(f: impl FnOnce() -> T) -> (T, Counted) {
    let (a0, b0, l0) = (
        ALLOCS.load(Relaxed),
        BYTES.load(Relaxed),
        LIVE.load(Relaxed),
    );
    PEAK.store(l0, Relaxed);
    let v = f();
    let counted = Counted {
        allocations: ALLOCS.load(Relaxed) - a0,
        bytes: BYTES.load(Relaxed) - b0,
        live_delta: LIVE.load(Relaxed) as isize - l0 as isize,
        peak_delta: PEAK.load(Relaxed).saturating_sub(l0),
    };
    (v, counted)
}

/// One row of `results/E18-memory.md`.
#[derive(Debug, Clone)]
pub struct Row {
    /// A stable identifier — the key a threshold is written against.
    pub scenario: &'static str,
    /// What one operation is, for the per-operation columns.
    pub unit: &'static str,
    pub operations: u64,
    pub counted: Counted,
}

impl Row {
    pub fn allocations_per_op(&self) -> f64 {
        self.counted.allocations as f64 / self.operations.max(1) as f64
    }
    pub fn bytes_per_op(&self) -> f64 {
        self.counted.bytes as f64 / self.operations.max(1) as f64
    }
    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{:.1},{:.1}",
            self.scenario,
            self.unit,
            self.operations,
            self.counted.allocations,
            self.counted.bytes,
            self.counted.live_delta,
            self.counted.peak_delta,
            self.allocations_per_op(),
            self.bytes_per_op()
        )
    }
}

pub const CSV_HEADER: &str =
    "scenario,unit,operations,allocations,bytes,live_delta,peak_delta,allocations_per_op,bytes_per_op";

/// **The budget each scenario is held to**, as allocations per operation.
///
/// Allocation counts are exact, so these are exact bounds rather than tolerances: a build
/// that allocates once more per row than the number here has changed something, and the
/// change should be deliberate. Bytes and live figures are reported and **not** asserted —
/// they move with a `Vec`'s growth policy in ways that are not a regression.
///
/// Each is the measured figure plus about ten percent: tight enough that a cost paid *per
/// row* cannot arrive unnoticed — that is the class of regression worth catching, and it
/// arrives in the thousands — and loose enough that a refactor costing one more allocation
/// per query does not fail a build on its own. A number here is a decision, so each carries
/// what the code does today and what it would have to stop doing to breach it.
pub fn budget(scenario: &str) -> Option<f64> {
    Some(match scenario {
        // Two `Vec`s per posting — the row, and the tree node holding it — plus the tree's
        // own growth. T-04 takes the Z-set off the served path; this row keeps measuring the
        // reference evaluator's base, which stays and should stay.
        "zset_base_at" => 1.4,
        // A seeded ledger: one epoch record, one idempotency string and one index entry per
        // transaction, amortised over two postings.
        "ledger_seeded" => 2.6,
        // **The three served analytical statements, and the numbers are still not typos.**
        // An unkeyed `group by` materialises the whole base as a `BTreeMap<Vec<Value>,
        // i128>` and folds it, so a query over 20,000 postings allocates three to eight
        // times *per posting*. It was eight to thirteen: removing the evaluator's
        // whole-source copy and `add`'s key clone halved these, and the halves that remain
        // are the base materialisation itself, which a streaming fold removes rather than
        // shrinks. These are a ratchet at what it costs today.
        "served_group_by_cur" => 95_000.0,
        "served_group_by_acct" => 187_000.0,
        "served_sum_negative" => 77_000.0,
        // A served point read goes through the anchor index, so its cost is the account's
        // own postings and the reply — not the base.
        "served_point" => 30.0,
        // The REV runtime's hit path: the key is cloned into the recency map, and the answer
        // is a `Copy` struct.
        "rev_read_hit" => 2.2,
        // An in-memory append: the row vector, the idempotency key, the index push.
        "append_in_memory" => 4.4,
        _ => return None,
    })
}

/// The scenarios E18 reports, in order.
pub const SCENARIOS: &[&str] = &[
    "ledger_seeded",
    "zset_base_at",
    "served_group_by_cur",
    "served_group_by_acct",
    "served_sum_negative",
    "served_point",
    "rev_read_hit",
    "append_in_memory",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scenario_that_is_measured_has_a_budget() {
        // A row with no budget is a row nothing can regress against, which is the state this
        // whole module exists to leave.
        for s in crate::alloc::SCENARIOS {
            assert!(
                budget(s).is_some(),
                "`{s}` is measured and has no budget: a number nobody has decided on cannot \
                 fail a build"
            );
        }
    }

    #[test]
    fn a_reading_reports_what_a_region_kept_apart_from_what_it_touched() {
        // The distinction the table rests on: a query that allocates 30MB and frees it has a
        // large `bytes` and a zero `live_delta`; a structure that is retained has both.
        let (_, transient) = count(|| {
            let v: Vec<u64> = (0..1000).collect();
            std::hint::black_box(&v);
        });
        assert!(transient.bytes > 0);
        assert_eq!(transient.live_delta, 0, "a dropped vector keeps nothing");
        assert!(transient.peak_delta > 0, "and its peak is still visible");

        let (kept, retained) = count(|| (0..1000).collect::<Vec<u64>>());
        assert!(retained.live_delta > 0, "a returned vector is held");
        drop(kept);
    }
}
