//! The authoritative ledger write path (thesis Ch. 6.2, Appendix D.2).
//!
//! Append-only, hash-chained, epoch-ordered, strictly serializable, never partial.

pub mod admission;
pub mod aead;
pub mod chain;
pub mod epoch;
pub mod frontiers;
pub mod random;
pub mod segment;
pub mod sequencer;
pub mod sidecar;

/// Exact minor-unit money. Floats never touch amounts (thesis 7.1).
pub type Minor = i128;

/// Epoch identifier on the global visibility timeline (thesis 3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Epoch(pub u64);
