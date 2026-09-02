//! The consistency ladder, and the one thing about it that is not a menu.
//!
//! The six rungs are ordered, and the ordering is load-bearing: it is what rung
//! monotonicity is defined over, and what makes "a computation is no fresher than its
//! stalest input" a statement with content.
//!
//! The rung a view is served at is **not** a free choice from a menu of qualities. The
//! lower rungs are in the highly-available class and the top rung is provably in the
//! unavailable one, so declaring `ledger_consistent` is a statement about what the view
//! gives up — availability under partition — and not merely a request for better data.
//!
//! Where the cost of a rung falls was itself a measured finding, and it is not where the
//! design originally assumed. Instrumenting the read path showed no difference between
//! rungs at all: hit ratios and miss counts were indistinguishable. The cost falls on
//! **maintenance** — how often the view must be brought forward — and scales as roughly
//! 1/k in the staleness bound k. The re-instrumentation that found this is reported in the
//! thesis as a null result and its diagnosis, because the null was informative.

use niles_ir::Consistency;

/// How often a view at this rung must be maintained, in epochs.
///
/// This is the whole cost model of the ladder, and it is one line because the finding was
/// that the cost is one thing.
pub fn maintenance_stride(c: Consistency) -> u64 {
    match c {
        Consistency::Bounded { epochs, .. } => epochs.max(1),
        _ => 1,
    }
}

/// Whether the rung is in the highly-available class.
pub fn is_highly_available(c: Consistency) -> bool {
    matches!(
        c,
        Consistency::Bounded { .. } | Consistency::Monotonic | Consistency::ReadYourWrites
    )
}

/// Whether a view served at `outer` may read a view served at `inner`.
///
/// Rung monotonicity, at the runtime. The compiler rejects a violating program, but the IR
/// is the stable contract and may be produced by something else, so the runtime checks too.
pub fn permits_reading(outer: Consistency, inner: Consistency) -> bool {
    rank(inner) >= rank(outer)
}

fn rank(c: Consistency) -> u8 {
    match c {
        Consistency::Bounded { .. } => 0,
        Consistency::Monotonic => 1,
        Consistency::ReadYourWrites => 2,
        Consistency::Snapshot => 3,
        Consistency::Serializable => 4,
        Consistency::LedgerConsistent => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bounded_rung_is_maintained_less_often_in_proportion_to_its_slack() {
        assert_eq!(
            maintenance_stride(Consistency::Bounded {
                epochs: 8,
                millis: 0
            }),
            8
        );
        assert_eq!(maintenance_stride(Consistency::LedgerConsistent), 1);
        // A zero-epoch bound is the strict rung wearing a different name, and must not
        // become a division by zero.
        assert_eq!(
            maintenance_stride(Consistency::Bounded {
                epochs: 0,
                millis: 0
            }),
            1
        );
    }

    #[test]
    fn the_top_rung_is_the_unavailable_one() {
        assert!(is_highly_available(Consistency::ReadYourWrites));
        assert!(!is_highly_available(Consistency::LedgerConsistent));
        assert!(!is_highly_available(Consistency::Serializable));
    }

    #[test]
    fn monotonicity_permits_the_safe_direction_only() {
        assert!(permits_reading(
            Consistency::Bounded {
                epochs: 4,
                millis: 0
            },
            Consistency::LedgerConsistent
        ));
        assert!(!permits_reading(
            Consistency::LedgerConsistent,
            Consistency::Bounded {
                epochs: 4,
                millis: 0
            }
        ));
        assert!(permits_reading(
            Consistency::Snapshot,
            Consistency::Snapshot
        ));
    }
}
