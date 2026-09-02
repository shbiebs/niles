//! Fault campaigns: crash schedules, eviction storms, duplicate and reordered delivery.
//!
//! Thesis §9.6 lists these as the correctness programme beyond conservation, and §5.5
//! names fault injection as an instrument. This module was a one-line stub, so the
//! campaigns existed as a plan rather than as a thing that runs.
//!
//! What a fault campaign is here: a schedule of ordinary operations with a fault
//! interleaved at a chosen point, run against a system that must answer the same as the
//! oracle afterwards. Recovery, in this architecture, is a large eviction — Theorem 4.1
//! is what makes that sentence true rather than a hope — so a crash and an eviction storm
//! are the same test with different labels, and the labels are kept because the failures
//! they provoke are read differently by a person.

use crate::properties::{Op, Schedule};

/// Where a fault is injected into a schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// Drop all derived state and rebuild from the retained base.
    Crash,
    /// Evict repeatedly, so that almost every read reconstructs.
    EvictionStorm { times: usize },
    /// Deliver a previously admitted key again.
    DuplicateDelivery,
}

/// Insert `fault` after every `every` operations of `base`.
///
/// Injecting at a fixed stride rather than at random points is deliberate: a campaign
/// whose fault positions move with the seed cannot be re-run against a specific failure.
pub fn inject(base: &Schedule, fault: Fault, every: usize) -> Schedule {
    assert!(every > 0, "a stride of zero would inject forever");
    let mut ops = Vec::with_capacity(base.ops.len() * 2);
    let mut last_key: Option<String> = None;
    for (i, op) in base.ops.iter().enumerate() {
        if let Op::Submit { key, .. } = op {
            last_key = Some(key.clone());
        }
        ops.push(op.clone());
        if (i + 1).is_multiple_of(every) {
            match fault {
                Fault::Crash => ops.push(Op::Crash),
                Fault::EvictionStorm { times } => {
                    for _ in 0..times {
                        ops.push(Op::Evict);
                    }
                }
                Fault::DuplicateDelivery => {
                    if let Some(k) = &last_key {
                        ops.push(Op::Duplicate { key: k.clone() });
                    }
                }
            }
        }
    }
    Schedule {
        seed: base.seed,
        ops,
    }
}

/// Reorder the reads of a schedule without moving its writes.
///
/// Reads are the observations; writes are the history. Permuting observations must not
/// change any answer, because every read names the anchor it wants. A system whose
/// answers depend on the order its reads arrive in has state it should not have.
pub fn reorder_reads(base: &Schedule) -> Schedule {
    let mut reads: Vec<Op> = base
        .ops
        .iter()
        .filter(|o| matches!(o, Op::Read { .. }))
        .cloned()
        .collect();
    reads.reverse();
    let mut it = reads.into_iter();
    let ops = base
        .ops
        .iter()
        .map(|o| match o {
            Op::Read { .. } => it.next().unwrap_or_else(|| o.clone()),
            other => other.clone(),
        })
        .collect();
    Schedule {
        seed: base.seed,
        ops,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::schedule;

    #[test]
    fn a_crash_campaign_injects_at_a_reproducible_stride() {
        let s = schedule(42, 8, 100);
        let f = inject(&s, Fault::Crash, 10);
        let crashes = f.ops.iter().filter(|o| **o == Op::Crash).count();
        let before = s.ops.iter().filter(|o| **o == Op::Crash).count();
        assert!(crashes >= before + 10);
        // Reproducible: the same base and stride give the same campaign.
        assert_eq!(f.ops, inject(&s, Fault::Crash, 10).ops);
    }

    #[test]
    fn an_eviction_storm_evicts_many_times_per_injection() {
        let s = schedule(7, 8, 60);
        let f = inject(&s, Fault::EvictionStorm { times: 5 }, 6);
        let evictions = f.ops.iter().filter(|o| **o == Op::Evict).count();
        let before = s.ops.iter().filter(|o| **o == Op::Evict).count();
        assert_eq!(evictions, before + 5 * (60 / 6));
    }

    #[test]
    fn duplicate_delivery_replays_a_key_that_was_actually_admitted() {
        let s = schedule(1, 8, 80);
        let f = inject(&s, Fault::DuplicateDelivery, 7);
        let submitted: Vec<&String> = s
            .ops
            .iter()
            .filter_map(|o| match o {
                Op::Submit { key, .. } => Some(key),
                _ => None,
            })
            .collect();
        let replayed: Vec<&String> = f
            .ops
            .iter()
            .filter_map(|o| match o {
                Op::Duplicate { key } => Some(key),
                _ => None,
            })
            .collect();
        assert!(!replayed.is_empty());
        for k in &replayed {
            assert!(
                submitted.contains(k),
                "a duplicate campaign must replay a key that was admitted, not invent one"
            );
        }
    }

    #[test]
    fn reordering_reads_preserves_the_writes_exactly() {
        let s = schedule(100, 8, 120);
        let r = reorder_reads(&s);
        let writes = |sch: &Schedule| -> Vec<Op> {
            sch.ops
                .iter()
                .filter(|o| !matches!(o, Op::Read { .. }))
                .cloned()
                .collect()
        };
        assert_eq!(writes(&s), writes(&r));
        assert_eq!(s.ops.len(), r.ops.len());
    }

    #[test]
    fn reordering_reads_actually_changes_the_read_order() {
        // A no-op reorder would make every test built on it vacuous.
        let s = schedule(2024, 8, 200);
        let r = reorder_reads(&s);
        let reads = |sch: &Schedule| -> Vec<Op> {
            sch.ops
                .iter()
                .filter(|o| matches!(o, Op::Read { .. }))
                .cloned()
                .collect()
        };
        assert_ne!(reads(&s), reads(&r));
    }

    #[test]
    fn a_campaign_is_still_a_schedule_the_harness_can_run() {
        let s = schedule(7, 8, 50);
        for fault in [
            Fault::Crash,
            Fault::EvictionStorm { times: 3 },
            Fault::DuplicateDelivery,
        ] {
            let f = inject(&s, fault, 5);
            assert!(f.ops.len() >= s.ops.len());
            assert!(matches!(f.ops.first(), Some(Op::Open { .. })));
        }
    }
}
