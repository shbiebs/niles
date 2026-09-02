// `Tri::not` is Kleene three-valued negation and is named after the logic, not after
// `std::ops::Not`; implementing the trait would give `!` a meaning on a value where
// `Unknown` is a third answer rather than a flipped bit.
#![allow(clippy::should_implement_trait)]

//! The typed intermediate representation (thesis 6.9, Appendix D.4).
//!
//! The IR is the *stable contract between the language and the engine*: a DBSP-style
//! circuit language whose types carry anchors, effects, contracts and provenance. The
//! theorems quantify over this object, not over any surface syntax — an IR that erased
//! those annotations would leave the proofs talking about something else.

pub mod circuit;
pub mod eval;
pub mod operator;
pub mod schedule;
pub mod upquery_path;
pub mod value;
pub mod verify;

/// The consistency ladder (thesis 3.7), declarable per view.
///
/// Not a free menu: the lower rungs are in the highly-available class and the top rung is
/// provably in the unavailable one, so a contract is a statement about what a view gives up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Consistency {
    /// l0: anchored at e >= vis - K epochs, and no older than T — whichever binds first.
    Bounded { epochs: u64, millis: u64 },
    /// l1: per session, anchors never decrease.
    Monotonic,
    /// l2: reads observe the session's own committed writes.
    ReadYourWrites,
    /// l3: one anchor per read set, across views.
    Snapshot,
    /// l4: read anchors form a serial order with transactions.
    Serializable,
    /// l5: anchored at vis(t) exactly; reflects every committed write preceding the read
    /// in real time. The authorization path.
    LedgerConsistent,
}

/// Materialization mode (thesis Def. 4.2). `Auto` delegates to the optimizer within the
/// other contract constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Materialize {
    Absent,
    Demand,
    Full,
    Spilled,
    Tiered,
    Auto,
}

/// Retention of *derived* state. The base is always retained: Proposition 3.4 makes full
/// retention necessary, not merely sufficient, for reconstructibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retention {
    Evictable,
    Pinned,
    Forever,
}

/// Lineage mode (thesis 3.10). `Off` still stamps every answer with its anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lineage {
    Off,
    Key,
    Full,
}

/// The per-view serve contract (thesis B.8, Appendix D.7).
#[derive(Debug, Clone, Copy)]
pub struct ServeContract {
    pub consistency: Consistency,
    pub materialize: Materialize,
    pub retain: Retention,
    pub lineage: Lineage,
}

impl ServeContract {
    /// The rung multiplier Phi(l) of the cost law (thesis 3.14): anchor bookkeeping is
    /// O(1) at the session rungs, while the top rung pays a freshness term on every miss.
    pub fn cost_multiplier(&self) -> f64 {
        match self.consistency {
            Consistency::Bounded { .. } | Consistency::Monotonic | Consistency::ReadYourWrites => {
                1.0
            }
            Consistency::Snapshot => 1.2,
            Consistency::Serializable => 1.6,
            Consistency::LedgerConsistent => 2.5,
        }
    }

    /// A mode is feasible only if it can meet the rung. Infeasible modes are filtered
    /// *before* cost comparison, which is what makes a bad estimate cost money rather
    /// than breach a contract (thesis I.5, I.7).
    pub fn permits(&self, m: Materialize) -> bool {
        match (self.consistency, m) {
            // Strict serving cannot be backed by a mode whose read path is an I/O round
            // trip on the critical path.
            (Consistency::LedgerConsistent, Materialize::Spilled) => false,
            // A pinned view is by definition resident.
            _ => !(self.retain == Retention::Pinned && matches!(m, Materialize::Absent)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract(c: Consistency, r: Retention) -> ServeContract {
        ServeContract {
            consistency: c,
            materialize: Materialize::Auto,
            retain: r,
            lineage: Lineage::Off,
        }
    }

    #[test]
    fn stricter_rungs_cost_more() {
        let lax = contract(
            Consistency::Bounded {
                epochs: 2,
                millis: 5_000,
            },
            Retention::Evictable,
        );
        let strict = contract(Consistency::LedgerConsistent, Retention::Evictable);
        assert!(strict.cost_multiplier() > lax.cost_multiplier());
    }

    #[test]
    fn strict_rung_rejects_spilled_state() {
        let c = contract(Consistency::LedgerConsistent, Retention::Evictable);
        assert!(!c.permits(Materialize::Spilled));
        assert!(c.permits(Materialize::Demand));
    }

    #[test]
    fn pinned_views_cannot_be_absent() {
        let c = contract(Consistency::Snapshot, Retention::Pinned);
        assert!(!c.permits(Materialize::Absent));
        assert!(c.permits(Materialize::Full));
    }

    #[test]
    fn ladder_is_ordered() {
        assert!(Consistency::ReadYourWrites < Consistency::LedgerConsistent);
        assert!(Consistency::Snapshot < Consistency::Serializable);
    }
}
