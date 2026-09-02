//! The adaptive materialization optimizer (thesis Contribution 5, Appendix I).
//!
//! Chooses, per view and key range, a point of the materialization mode lattice,
//! subject to each view's consistency and freshness contract.
//!
//! Three safety properties hold regardless of estimator quality:
//!
//! 1. **Semantic safety** (Thm 4.5(a)): a mode change cannot change any answer, so no
//!    individual heuristic needs its own correctness argument.
//! 2. **Contract safety**: infeasible modes are filtered out *before* cost comparison,
//!    so a bad estimate can cost money but cannot breach a contract.
//! 3. **Explainability**: every transition records the estimates that caused it.

pub mod estimators;
pub mod eviction;
pub mod join_order;
pub mod modes;
pub mod offline;
pub mod plan_space;
pub mod unnest;

pub use modes::{Mode, ModeDecision};

/// The materialization mode lattice (thesis Def. 4.2).
///
/// Ordered by resident cost and inversely by per-read reconstruction cost.
pub mod mode_lattice {
    /// Keep nothing; reconstruct on every read.
    pub const ABSENT: &str = "absent";
    /// Materialize on read; evict under pressure (partial state).
    pub const DEMAND: &str = "demand";
    /// Materialize the whole range; maintain eagerly.
    pub const FULL: &str = "full";
    /// Materialized but resident on secondary storage: pay I/O, not reconstruction.
    pub const SPILLED: &str = "spilled";
    /// Hot prefix resident, remainder spilled, with a promotion rule.
    pub const TIERED: &str = "tiered";
}
