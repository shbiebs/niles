//! **E18 — the memory instrument.**
//!
//! A library so that the published table and the budget gate call the same scenario
//! functions. Two copies of a measurement is two numbers that can disagree, and the one in
//! the gate would be the one nobody looked at.
//!
//! See `Cargo.toml` for why this package is outside the workspace.

pub mod alloc;
pub mod scenarios;

/// **The counting allocator, installed for this crate's own unit tests.**
///
/// Without it `cargo test --lib` runs with the system allocator, every counter reads zero,
/// and `a_reading_reports_what_a_region_kept_apart_from_what_it_touched` fails on
/// `transient.bytes > 0` — which is what it did, unnoticed, because `make memory` passed
/// `--ignored` (filtering every non-ignored test out) and `memprobe` is outside the
/// workspace, so `make test` never reached it either. A library must not choose a program's
/// allocator, and a test binary is this library's own program: `cfg(test)` is exactly the
/// scope where that is true.
#[cfg(test)]
#[global_allocator]
static COUNTING_IN_TESTS: alloc::Counting = alloc::Counting;
