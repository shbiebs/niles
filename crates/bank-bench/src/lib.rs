//! NilesBank: the core-banking benchmark generator and harness (thesis §9.1, Appendix G.9).
//!
//! # What was added, and why it changes what this crate is for
//!
//! This crate used to hold a generator and a scenario list. It now holds a **wall-clock
//! harness** that drives PostgreSQL and Nilestream over the same wire protocol and emits the
//! `SPEC-ENGINE.md` Part 0 table from the data it collected.
//!
//! The reason is a finding from the architecture review. The specification commits to 5–10×
//! on OLTP and 10–12× on analytical *relative to PostgreSQL*, and no measurement in this
//! repository had ever compared the two: every experiment reported counted work inside
//! `proto-engine`, and the one wall-clock table in the thesis was in-memory, single-threaded
//! and compared to nothing. A speed claim whose only evidence is counted operations in a
//! prototype is, in the thesis's own grading, an editorial contribution.
//!
//! ```text
//! cargo run -p bank-bench --bin bench -- --calibrate
//! cargo run -p bank-bench --bin bench -- --run
//! ```

pub mod analysis;
pub mod generator;
pub mod publish;
pub mod render;
pub mod scenarios;
pub mod storage;
pub mod target;
pub mod wire;
pub mod workloads;
