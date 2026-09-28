//! **E27 — the engine measured against its alternatives** (cycle 14, card R2-02).
//!
//! The comparator the cycle-11, -12 and -13 work orders specified and none ran. One host,
//! one generated universe per (size, seed), every arm loaded from it and proved to hold the
//! same legs, the same script run on every arm interleaved, every answer checked against an
//! oracle this crate owns, and verdicts by the joint gate with a resolution floor from
//! warm-up runs. The specification is §5 of `claude/cycle-14-round-2-work-order.md`.
//!
//! Modules: [`universe`] (the generator), [`oracle`] (the answers), [`arms`] (N, P+, P, M and
//! the probe's M+), [`pgcluster`] (one PostgreSQL cluster per arm), [`run`] (one size and
//! seed), [`stats`] (the gate), [`store`] (one file per point), [`render`] (E27), [`probe`]
//! (the five anomalies).

pub mod arms;
pub mod oracle;
pub mod pgcluster;
pub mod probe;
pub mod render;
pub mod run;
pub mod stats;
pub mod store;
pub mod universe;
