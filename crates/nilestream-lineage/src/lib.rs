//! Lineage and provenance (thesis 3.10, 6.15, Appendix D).
//!
//! Provenance rides the same circuits as values: Z-sets are the (Z,+,·) instance of a
//! commutative semiring, and the factorization theorem means the most general annotation
//! specializes to every other semantics by homomorphism. Three declarable modes.

pub mod semiring;
pub mod explain;
pub mod impact;

/// Per-view lineage mode (thesis 3.10). `Off` still stamps every answer with its anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineageMode {
    /// Anchors only.
    Off,
    /// Which base keys contributed.
    Key,
    /// The how-provenance polynomial, retained.
    Full,
}
