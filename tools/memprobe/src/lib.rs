//! **E18 — the memory instrument.**
//!
//! A library so that the published table and the budget gate call the same scenario
//! functions. Two copies of a measurement is two numbers that can disagree, and the one in
//! the gate would be the one nobody looked at.
//!
//! See `Cargo.toml` for why this package is outside the workspace.

pub mod alloc;
pub mod scenarios;
