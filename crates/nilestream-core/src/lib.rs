//! Nilestream-Core: the partial-state read-model runtime (thesis Ch. 6.2, Appendix D.3).
//!
//! Home of the REV: reconstructible epoch-anchored views.

pub mod rev;
pub mod absence;
pub mod anchor_index;
pub mod upquery;
pub mod eviction;
pub mod ladder;

use nilestream_ledger::Epoch;

/// The lattice of absence (thesis 3.3): every (view, key) slot is exactly here.
#[derive(Debug, Clone)]
pub enum Slot<V> {
    /// Never computed; the runtime asserts nothing.
    Bottom,
    /// Evicted: value gone, version remembered (honest absence).
    Hole(Epoch),
    /// Upquery in flight toward this anchor.
    Pending(Epoch),
    /// Certified: value equals the ideal view at the anchor epoch.
    Present(V, Epoch),
}

/// A value stamped with the epoch it is correct as of (thesis 3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchored<T> {
    pub value: T,
    pub anchor: Epoch,
}
