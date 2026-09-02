//! The absence lattice.
//!
//! Four states, ordered ⊥ ⊏ Hole(e) ⊏ Pending(e) ⊏ Present(v, e), and the whole partial-
//! state theory rests on the distinction between the first two.
//!
//! # Honest absence
//!
//! When an entry is evicted, the runtime does **not** forget that the key exists. It keeps
//! the key and its *version* — the epoch through which deltas had been applied — and drops
//! only the value. That is a `Hole(e)`, and it is categorically different from `⊥`, which
//! means "this key has never been seen".
//!
//! The reason is the single most important safety property of partial state, and it has a
//! name: **miss ≠ zero**. A cache that forgets a key entirely cannot distinguish "the
//! balance is zero" from "I evicted the balance", and a system that confuses those two has
//! a mechanism for creating money out of memory pressure. Keeping the version means a
//! reconstruction knows exactly which prefix it must fold, and a read that finds a hole
//! knows it must reconstruct rather than answer zero.
//!
//! `Pending(e)` exists because reconstruction is not instantaneous. A second reader
//! arriving during an upquery must join it rather than start a second one, and must not
//! see an absence and conclude anything.

use std::fmt;

/// An epoch: the unit of visibility, versioning and hashing.
pub type Epoch = u64;

/// A slot in a partially materialized view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot<V> {
    /// Never seen. Distinct from a hole: nothing is known, not even a version.
    Bottom,
    /// **Evicted, honestly.** The value is gone; the version is kept. A read here must
    /// reconstruct, and knows from `e` exactly which prefix to fold.
    Hole(Epoch),
    /// A reconstruction is in flight, started at this epoch. A second reader joins it.
    Pending(Epoch),
    /// Materialized: a value, and the epoch through which deltas have been applied.
    Present(V, Epoch),
}

impl<V> Slot<V> {
    /// The lattice height, for the ordering.
    pub fn rank(&self) -> u8 {
        match self {
            Slot::Bottom => 0,
            Slot::Hole(_) => 1,
            Slot::Pending(_) => 2,
            Slot::Present(..) => 3,
        }
    }

    /// The version, if the slot has one. `Bottom` has none — that is what makes it
    /// different from a hole, and the difference is the safety property.
    pub fn version(&self) -> Option<Epoch> {
        match self {
            Slot::Bottom => None,
            Slot::Hole(e) | Slot::Pending(e) | Slot::Present(_, e) => Some(*e),
        }
    }

    pub fn value(&self) -> Option<&V> {
        match self {
            Slot::Present(v, _) => Some(v),
            _ => None,
        }
    }

    pub fn is_resident(&self) -> bool {
        matches!(self, Slot::Present(..))
    }

    /// Whether a read of this slot must reconstruct before it can answer.
    ///
    /// **`Bottom` must reconstruct too.** It is tempting to answer an unknown key with the
    /// identity of the aggregate — zero, for a sum — and that is exactly the bug: a key
    /// that has never been read is not a key that has no postings. The engine cannot tell
    /// those apart without asking the base, and it must ask.
    pub fn needs_reconstruction(&self) -> bool {
        !matches!(self, Slot::Present(..))
    }

    /// Evict: drop the value, keep the version. The operation this whole module exists for.
    pub fn evict(&mut self) -> bool {
        match self {
            Slot::Present(_, e) => {
                *self = Slot::Hole(*e);
                true
            }
            _ => false,
        }
    }

    /// The lattice join. Used when a reconstruction result meets concurrent delta
    /// application: the more-defined, more-recent state wins, and the epoch is the maximum,
    /// which is what makes application idempotent under retry.
    pub fn join(self, other: Slot<V>) -> Slot<V>
    where
        V: Clone,
    {
        match (self.version(), other.version()) {
            (Some(a), Some(b)) if a > b => self,
            (Some(a), Some(b)) if b > a => other,
            _ if self.rank() >= other.rank() => self,
            _ => other,
        }
    }
}

impl<V: fmt::Debug> fmt::Display for Slot<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Bottom => write!(f, "⊥"),
            Slot::Hole(e) => write!(f, "Hole(#{e})"),
            Slot::Pending(e) => write!(f, "Pending(#{e})"),
            Slot::Present(v, e) => write!(f, "Present({v:?}, #{e})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lattice_is_ordered() {
        let rungs: [Slot<i64>; 4] = [
            Slot::Bottom,
            Slot::Hole(1),
            Slot::Pending(1),
            Slot::Present(0, 1),
        ];
        for w in rungs.windows(2) {
            assert!(
                w[0].rank() < w[1].rank(),
                "{} must sit below {}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn eviction_keeps_the_version_and_drops_the_value() {
        // The definition of honest absence.
        let mut s = Slot::Present(500i64, 42);
        assert!(s.evict());
        assert_eq!(s, Slot::Hole(42));
        assert_eq!(s.version(), Some(42), "the version must survive eviction");
        assert_eq!(s.value(), None);
    }

    #[test]
    fn a_hole_is_not_bottom_and_that_is_the_safety_property() {
        // A cache that cannot tell these apart has a mechanism for creating money out of
        // memory pressure: it would answer an evicted balance with the aggregate's
        // identity. Both must reconstruct, but only a hole knows which prefix to fold.
        let hole: Slot<i64> = Slot::Hole(42);
        let bottom: Slot<i64> = Slot::Bottom;
        assert_ne!(hole, bottom);
        assert_eq!(hole.version(), Some(42));
        assert_eq!(bottom.version(), None);
        assert!(hole.needs_reconstruction() && bottom.needs_reconstruction());
    }

    #[test]
    fn an_unknown_key_is_never_answered_with_zero() {
        // Stated as its own test because it is the single easiest optimisation to make and
        // the single worst: a key that has never been read is not a key with no postings.
        let unknown: Slot<i64> = Slot::Bottom;
        assert!(unknown.needs_reconstruction());
        assert_eq!(
            unknown.value(),
            None,
            "there is no value to hand back, and no identity to invent"
        );
    }

    #[test]
    fn eviction_is_idempotent_and_holes_do_not_evict() {
        let mut s: Slot<i64> = Slot::Hole(7);
        assert!(
            !s.evict(),
            "evicting a hole is a no-op, not a version reset"
        );
        assert_eq!(s, Slot::Hole(7));
    }

    #[test]
    fn the_join_takes_the_later_epoch() {
        // What makes delta application idempotent under retry: applying an older result
        // over a newer one must not move the state backwards.
        let newer = Slot::Present(10i64, 9);
        let older = Slot::Present(5i64, 4);
        assert_eq!(newer.clone().join(older.clone()), Slot::Present(10, 9));
        assert_eq!(older.join(newer), Slot::Present(10, 9));
    }

    #[test]
    fn at_equal_epochs_the_more_defined_state_wins() {
        let present = Slot::Present(3i64, 5);
        let hole: Slot<i64> = Slot::Hole(5);
        assert_eq!(present.clone().join(hole.clone()), Slot::Present(3, 5));
        assert_eq!(hole.join(present), Slot::Present(3, 5));
    }

    #[test]
    fn pending_prevents_a_second_upquery_for_the_same_key() {
        let s: Slot<i64> = Slot::Pending(11);
        assert!(s.needs_reconstruction());
        assert_eq!(
            s.version(),
            Some(11),
            "a joiner learns which reconstruction it is joining"
        );
        assert!(!s.is_resident());
    }
}
