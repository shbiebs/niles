//! A research prototype of a partially-materialized read-model engine over an immutable,
//! epoch-ordered, hash-chained ledger.
//!
//! SCOPE — read this before interpreting any measurement produced with this crate.
//!
//! This is a single-node, in-memory, single-threaded research prototype built to make the
//! thesis's *mechanisms* measurable: epoch-ordered commit with a per-currency zero-sum
//! commit rule, hash chaining, per-key anchor indices, partial materialization with an
//! absence lattice, anchored upqueries, and eviction policies. It is NOT a database. It has
//! no durability (no fsync, no WAL), no concurrency control beyond single-threaded
//! execution, no consensus, no query planner, no SQL surface, and only two view shapes
//! (per-key sum and grouped rollup).
//!
//! Consequently it can support claims about **counted work** — base rows touched, deltas
//! applied, resident entries, upqueries issued — which are properties of the algorithms and
//! the workload and are reproducible on any machine. It cannot support claims about
//! throughput relative to a production DBMS, and none are made.
//!
//! The counted-work discipline is deliberate: reporting a cost ratio in machine-independent
//! units avoids attributing to the design what is actually an artifact of a shared virtual
//! machine, and it is what makes the phase diagram in Chapter 9 reproducible by a reader on
//! different hardware.

pub mod ledger;
pub mod view;
pub mod policy;
pub mod workload;
pub mod cost;

pub use ledger::{Ledger, Posting, Reject, Row, Hold, Outcome};
pub use view::{PartialView, Slot, ViewMode, ViewStats};
pub use policy::EvictionPolicy;
pub use workload::Zipf;
pub use cost::CostModel;

/// Exact integer minor units at the currency's declared scale (never floating point).
pub type Minor = i128;

/// Account identifier.
pub type Acct = u64;

/// Currency index. Scale is per currency (ISO 4217 exponents range at least 0..=3), so no
/// scale is hard-coded anywhere in this prototype.
pub type Cur = u32;

/// A ledger epoch: the unit of visibility, versioning and hashing.
pub type Epoch = u64;
