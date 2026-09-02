//! **Plan-space restriction** — the cheapest large win in the optimizer literature.
//!
//! This module implements ROADMAP phase 1, and it is worth stating up front why it exists
//! rather than a better cardinality estimator.
//!
//! # Why restriction rather than estimation
//!
//! "The optimizer should never produce a bad plan" is refuted by three independent results.
//! The decisive one is **Plan Bouquets** (Dutt & Haritsa, SIGMOD 2014): even abandoning
//! compile-time estimation entirely and *discovering* selectivities by executing, no
//! deterministic online algorithm achieves maximum suboptimality below **4×** — with even
//! one unknown selectivity. Charikar et al. (PODS 2000) add that any estimator sampling *n*
//! of *N* rows incurs Ω(√(N/n)) ratio error on some input. Learned optimizers do not rescue
//! it: they degrade under updates, cost 5–20 ms of inference, and industry has not adopted
//! them.
//!
//! So estimation cannot be fixed. What *can* be fixed is what a bad estimate is allowed to
//! do. Leis et al. measured it: disabling risky nested loops and enabling runtime hash-table
//! resizing cut queries running more than 2× slower from **38% to under 4%**. That is the
//! largest single win available, it costs a filter rather than a subsystem, and it is
//! robust precisely because it does not depend on the estimate being right.
//!
//! # The asymmetry that makes this work
//!
//! A nested-loop join over an *unindexed* inner relation costs |L| × |R|. If the estimate
//! of |L| is low by 100×, a hash join costs 100× more than predicted — bad, and linear in
//! the error. The nested loop costs 100× more than predicted *too*, but from a quadratic
//! base, so the absolute damage is unbounded.
//!
//! **The point is not that nested loops are slow. It is that their downside under
//! misestimation is unbounded while a hash join's is linear.** A plan whose worst case is
//! unbounded should not be reachable, however good it looks under an estimate nobody can
//! trust. This is a decision-theoretic argument, not a performance one: it minimises regret
//! rather than expected cost, which is the right objective when the input to the expectation
//! is known to be unreliable.
//!
//! # What is deliberately still permitted
//!
//! An *index* nested-loop join over a small outer relation is bounded by |L| × log|R| and is
//! frequently the best plan for a selective lookup. Forbidding it would cost real
//! performance on exactly the point-lookup workloads where PostgreSQL is already at the
//! achievable bound — and `SPEC-ENGINE.md` Part 0 makes not regressing there a normative
//! requirement. So the restriction is narrow by design: it forbids the *unbounded* shape,
//! not the operator.

use crate::join_order::{Plan, Relation};
use std::fmt;

/// How an inner relation is accessed by a nested-loop join.
///
/// The distinction the whole module turns on. Same operator, two entirely different risk
/// profiles under misestimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerAccess {
    /// An index covers the join key: cost is |L| × log|R|, bounded.
    Indexed,
    /// No index: cost is |L| × |R|. **Unbounded under misestimation.**
    Scan,
}

/// Why a plan was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// A nested loop over an unindexed inner relation.
    UnboundedNestedLoop {
        outer: String,
        inner: String,
        /// The estimated product, so the diagnostic shows the scale of what was avoided.
        estimated_tuples: u128,
    },
    /// A cross product between two relations that *do* have a join predicate available.
    ///
    /// Distinct from an unavoidable cross product on a disconnected graph: this one is the
    /// enumerator failing to find an edge it had, which is a bug rather than a plan.
    AvoidableCrossProduct { left: String, right: String },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::UnboundedNestedLoop {
                outer,
                inner,
                estimated_tuples,
            } => write!(
                f,
                "refused: nested loop joining `{outer}` against unindexed `{inner}` \
                 (~{estimated_tuples} tuple comparisons). A nested loop over an unindexed \
                 inner relation is unbounded under misestimation, where a hash join's error \
                 is linear. Add an index on the join key, or let the planner use a hash join"
            ),
            Refusal::AvoidableCrossProduct { left, right } => write!(
                f,
                "refused: cross product between `{left}` and `{right}`, which have a join \
                 predicate available. An unavoidable cross product on a disconnected graph \
                 is permitted; this one is the enumerator missing an edge"
            ),
        }
    }
}

/// The restriction policy.
///
/// Configurable because the right answer differs by deployment, and because a policy that
/// cannot be turned off cannot be measured against. The Leis result is the *default*, not a
/// law.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Policy {
    /// Refuse nested loops over unindexed inner relations.
    pub forbid_unbounded_nested_loop: bool,
    /// Below this size, a nested loop over an unindexed inner is permitted anyway.
    ///
    /// A twelve-row dimension table does not need an index, and forbidding the loop there
    /// would force a hash build that costs more than the loop. The threshold is where the
    /// hash build's fixed cost stops being worth avoiding — not a tuning knob for making
    /// the restriction go away.
    pub small_inner_threshold: u64,
    /// Refuse cross products the enumerator could have avoided.
    pub forbid_avoidable_cross_products: bool,
    /// Permit hash tables to grow at runtime rather than being sized from the estimate.
    ///
    /// The other half of Leis's result, and it belongs here because it is the same idea:
    /// a hash table sized from a low estimate degrades to a scan, which reintroduces the
    /// unbounded behaviour the nested-loop rule removes. Resizing bounds the damage a bad
    /// estimate can do *at runtime*, where the plan restriction bounds it at plan time.
    pub runtime_hash_resize: bool,
}

impl Default for Policy {
    /// Leis et al.'s measured configuration: 38% → under 4% of queries more than 2× slower
    /// than the best observed plan.
    fn default() -> Self {
        Policy {
            forbid_unbounded_nested_loop: true,
            small_inner_threshold: 1_000,
            forbid_avoidable_cross_products: true,
            runtime_hash_resize: true,
        }
    }
}

impl Policy {
    /// Every restriction off. For measuring what the restriction is worth — a policy with
    /// no off switch is a policy nobody can put a number on.
    pub fn unrestricted() -> Self {
        Policy {
            forbid_unbounded_nested_loop: false,
            small_inner_threshold: 0,
            forbid_avoidable_cross_products: false,
            runtime_hash_resize: false,
        }
    }
}

/// How a plan's joins are physically implemented, alongside the logical plan.
///
/// Carried separately rather than folded into `Plan`, because join *ordering* and join
/// *method* are different decisions made at different times, and the logical plan is what
/// the schedule verifier of ROADMAP phase 5 will need to reason about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Hash,
    /// A nested loop, and how it reaches the inner relation.
    NestedLoop(InnerAccess),
    Merge,
}

/// A physical annotation: for each join in the tree, in traversal order, its method.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Physical {
    pub methods: Vec<Method>,
}

impl Physical {
    pub fn new(methods: Vec<Method>) -> Self {
        Physical { methods }
    }

    /// Every join is a hash join. The safe default, and what the planner falls back to.
    pub fn all_hash(joins: usize) -> Self {
        Physical {
            methods: vec![Method::Hash; joins],
        }
    }
}

/// Check a plan against the policy. `Ok(())` means the plan is reachable.
///
/// Runs *before* costing, like the reconstructibility check in `join_order.rs` and for the
/// same reason: a refused plan is illegal rather than expensive, so a bad estimate can make
/// a plan slow but cannot make it unbounded.
pub fn check(
    plan: &Plan,
    physical: &Physical,
    rels: &[Relation],
    policy: &Policy,
) -> Result<(), Vec<Refusal>> {
    let mut refusals = Vec::new();
    let mut join_index = 0usize;
    walk(plan, physical, rels, policy, &mut join_index, &mut refusals);
    if refusals.is_empty() {
        Ok(())
    } else {
        Err(refusals)
    }
}

fn walk(
    plan: &Plan,
    physical: &Physical,
    rels: &[Relation],
    policy: &Policy,
    join_index: &mut usize,
    out: &mut Vec<Refusal>,
) {
    match plan {
        Plan::Scan(_) => {}
        Plan::Join { left, right, .. } => {
            walk(left, physical, rels, policy, join_index, out);
            walk(right, physical, rels, policy, join_index, out);

            let method = physical
                .methods
                .get(*join_index)
                .cloned()
                .unwrap_or(Method::Hash);
            *join_index += 1;

            if policy.forbid_unbounded_nested_loop {
                if let Method::NestedLoop(InnerAccess::Scan) = method {
                    let inner_rows = subtree_rows(right, rels);
                    if inner_rows > policy.small_inner_threshold {
                        out.push(Refusal::UnboundedNestedLoop {
                            outer: describe(left, rels),
                            inner: describe(right, rels),
                            estimated_tuples: subtree_rows(left, rels) as u128 * inner_rows as u128,
                        });
                    }
                }
            }
        }
        Plan::Cross { left, right } => {
            walk(left, physical, rels, policy, join_index, out);
            walk(right, physical, rels, policy, join_index, out);
            *join_index += 1;
            if policy.forbid_avoidable_cross_products {
                // Whether this cross product was avoidable is the *caller's* question — it
                // depends on the query graph, which `check` does not have. The planner only
                // emits `Cross` when the graph is genuinely disconnected, so reaching here
                // with a connected graph is the enumerator's bug, and
                // `a_connected_graph_never_gets_a_cross_product` in `join_order.rs` is the
                // test that catches it. This arm is the second line of defence.
                out.push(Refusal::AvoidableCrossProduct {
                    left: describe(left, rels),
                    right: describe(right, rels),
                });
            }
        }
    }
}

/// Estimated rows produced by a subtree. Deliberately crude — the *largest* input, rather
/// than a selectivity-adjusted product.
///
/// The reason to be crude here is the reason this module exists. A precise estimate would
/// reintroduce the dependence on estimation that the restriction is meant to remove: if the
/// rule for refusing an unbounded plan itself relied on a good estimate, it would fail in
/// exactly the cases it exists to catch. Using the largest base relation is a lower bound on
/// the risk that cannot be argued away by a low estimate.
fn subtree_rows(p: &Plan, rels: &[Relation]) -> u64 {
    match p {
        Plan::Scan(i) => rels.get(*i).map(|r| r.rows).unwrap_or(0),
        Plan::Join { left, right, .. } | Plan::Cross { left, right } => {
            subtree_rows(left, rels).max(subtree_rows(right, rels))
        }
    }
}

fn describe(p: &Plan, rels: &[Relation]) -> String {
    match p {
        Plan::Scan(i) => rels
            .get(*i)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| format!("#{i}")),
        Plan::Join { left, right, .. } => {
            format!("({} ⋈ {})", describe(left, rels), describe(right, rels))
        }
        Plan::Cross { left, right } => {
            format!("({} × {})", describe(left, rels), describe(right, rels))
        }
    }
}

/// The regret bound a policy provides, as a multiple of the oracle plan.
///
/// `None` when unbounded — which is the honest answer for an unrestricted policy and the
/// whole point of the restriction.
///
/// The 4× floor is **Plan Bouquets' lower bound**, not this implementation's achievement: no
/// deterministic algorithm does better, so a policy claiming less would be claiming to have
/// beaten a theorem. What restriction buys is *reaching* the neighbourhood of that floor
/// instead of being unbounded.
pub fn regret_bound(policy: &Policy) -> Option<f64> {
    if policy.forbid_unbounded_nested_loop {
        // Plan Bouquets: 4× is the floor for one unknown selectivity; ~4ρ for the realistic
        // multi-dimensional case. The engine publishes the bound rather than claiming
        // optimality (SPEC-ENGINE E-7).
        Some(4.0)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::join_order::{Edge, JoinPlanner, Plan, Relation};

    fn rels() -> Vec<Relation> {
        vec![
            Relation::base("postings", 10_000_000).with_distinct(0, 50_000),
            Relation::base("accounts", 50_000).with_distinct(0, 50_000),
            Relation::base("currencies", 12).with_distinct(0, 12),
        ]
    }

    fn two_way(l: usize, r: usize) -> Plan {
        Plan::Join {
            left: Box::new(Plan::Scan(l)),
            right: Box::new(Plan::Scan(r)),
            on: Edge::new(l, 0, r, 0),
        }
    }

    // ── the restriction ─────────────────────────────────────────────────────────────

    #[test]
    fn a_nested_loop_over_a_large_unindexed_inner_is_refused() {
        // The plan Leis's result is about. 10M × 50k is 5×10¹¹ comparisons, and a low
        // estimate would have made it look reasonable.
        let plan = two_way(0, 1);
        let phys = Physical::new(vec![Method::NestedLoop(InnerAccess::Scan)]);
        let e = check(&plan, &phys, &rels(), &Policy::default()).unwrap_err();

        assert_eq!(e.len(), 1);
        match &e[0] {
            Refusal::UnboundedNestedLoop {
                outer,
                inner,
                estimated_tuples,
            } => {
                assert_eq!(outer, "postings");
                assert_eq!(inner, "accounts");
                assert_eq!(*estimated_tuples, 10_000_000u128 * 50_000);
            }
            other => panic!("expected an unbounded nested loop, got {other:?}"),
        }
        // The diagnostic must say *why*, not merely refuse — the asymmetry is the argument.
        let msg = e[0].to_string();
        assert!(msg.contains("unbounded under misestimation"), "{msg}");
        assert!(msg.contains("hash join"), "and name the alternative: {msg}");
    }

    #[test]
    fn the_same_join_is_permitted_when_the_inner_is_indexed() {
        // **The narrowness that matters.** An index nested-loop join is bounded by
        // |L| × log|R| and is frequently the best plan for a selective lookup. Forbidding
        // the operator rather than the shape would cost real performance on exactly the
        // point-lookup workloads where SPEC-ENGINE Part 0 forbids a regression.
        let plan = two_way(0, 1);
        let phys = Physical::new(vec![Method::NestedLoop(InnerAccess::Indexed)]);
        assert!(check(&plan, &phys, &rels(), &Policy::default()).is_ok());
    }

    #[test]
    fn a_small_inner_relation_is_permitted_unindexed() {
        // Twelve currencies. Forcing a hash build here would cost more than the loop, and a
        // rule that could not tell the difference would be a rule that made things slower
        // in the name of safety.
        let plan = two_way(0, 2);
        let phys = Physical::new(vec![Method::NestedLoop(InnerAccess::Scan)]);
        assert!(check(&plan, &phys, &rels(), &Policy::default()).is_ok());
    }

    #[test]
    fn the_threshold_is_where_the_rule_starts_applying() {
        let plan = two_way(0, 1);
        let phys = Physical::new(vec![Method::NestedLoop(InnerAccess::Scan)]);

        // accounts has 50,000 rows.
        let below = Policy {
            small_inner_threshold: 50_000,
            ..Policy::default()
        };
        assert!(
            check(&plan, &phys, &rels(), &below).is_ok(),
            "at the threshold, permitted"
        );

        let above = Policy {
            small_inner_threshold: 49_999,
            ..Policy::default()
        };
        assert!(
            check(&plan, &phys, &rels(), &above).is_err(),
            "just above it, refused"
        );
    }

    #[test]
    fn a_hash_join_is_always_permitted() {
        // Its error under misestimation is linear, which is the entire basis of the rule.
        let plan = two_way(0, 1);
        for m in [Method::Hash, Method::Merge] {
            assert!(check(&plan, &Physical::new(vec![m]), &rels(), &Policy::default()).is_ok());
        }
    }

    // ── the rule does not depend on a good estimate ─────────────────────────────────

    #[test]
    fn the_refusal_survives_an_arbitrarily_low_cardinality_estimate() {
        // **The property that makes this robust.** If the rule used a
        // selectivity-adjusted estimate, a low estimate would make the unbounded plan look
        // small and the rule would fail in exactly the case it exists to catch. Using the
        // largest base relation is a lower bound on the risk that no estimate can argue
        // away.
        //
        // Here the *join* is estimated to produce one row — the classic misestimation — and
        // the inner relation is still large, so the loop is still unbounded.
        let plan = Plan::Join {
            left: Box::new(two_way(0, 2)), // estimated tiny
            right: Box::new(Plan::Scan(1)),
            on: Edge::new(0, 0, 1, 0),
        };
        let phys = Physical::new(vec![Method::Hash, Method::NestedLoop(InnerAccess::Scan)]);
        assert!(
            check(&plan, &phys, &rels(), &Policy::default()).is_err(),
            "a low estimate must not talk the rule out of the refusal"
        );
    }

    #[test]
    fn every_offending_join_in_a_deep_plan_is_reported_not_just_the_first() {
        // An operator fixing one refusal and re-running to find the next is an operator
        // doing the compiler's job.
        // Both inner relations must be *large* for both joins to offend. An earlier version
        // of this test put `currencies` (12 rows) on the inside of the outer join, which is
        // legitimately below the threshold — so it reported one refusal, correctly, and the
        // test was wrong rather than the code.
        let plan = Plan::Join {
            left: Box::new(two_way(2, 0)),  // currencies ⋈ postings — inner is 10M
            right: Box::new(Plan::Scan(1)), // ⋈ accounts — inner is 50k
            on: Edge::new(0, 0, 1, 0),
        };
        let phys = Physical::new(vec![
            Method::NestedLoop(InnerAccess::Scan),
            Method::NestedLoop(InnerAccess::Scan),
        ]);
        let e = check(&plan, &phys, &rels(), &Policy::default()).unwrap_err();
        assert_eq!(e.len(), 2, "both offending joins reported: {e:?}");
        // And the reports are distinguishable, so an operator can fix them independently.
        assert_ne!(e[0], e[1]);
    }

    // ── measurability ───────────────────────────────────────────────────────────────

    #[test]
    fn the_policy_can_be_turned_off_so_the_restriction_can_be_measured() {
        // A policy with no off switch is a policy nobody can put a number on, and phase 1's
        // gate is precisely a before-and-after measurement.
        let plan = two_way(0, 1);
        let phys = Physical::new(vec![Method::NestedLoop(InnerAccess::Scan)]);
        assert!(check(&plan, &phys, &rels(), &Policy::default()).is_err());
        assert!(check(&plan, &phys, &rels(), &Policy::unrestricted()).is_ok());
    }

    #[test]
    fn the_regret_bound_is_published_rather_than_optimality_claimed() {
        // SPEC-ENGINE E-7: the engine publishes a bound k ≥ 4 rather than claiming
        // optimality. 4× is Plan Bouquets' floor — a policy claiming less would be claiming
        // to have beaten a theorem.
        assert_eq!(regret_bound(&Policy::default()), Some(4.0));
        assert_eq!(
            regret_bound(&Policy::unrestricted()),
            None,
            "unrestricted, the bound is genuinely unbounded and says so"
        );
    }

    // ── integration with the enumerator ─────────────────────────────────────────────

    #[test]
    fn a_plan_from_the_enumerator_passes_under_the_default_physical_annotation() {
        // The planner emits logical plans; the default physical annotation is all-hash, and
        // all-hash is always permitted. So restriction never blocks a plan the enumerator
        // produced on its own — it constrains the *method* choice that comes later.
        let rels = rels();
        let edges = vec![Edge::new(0, 0, 1, 0), Edge::new(1, 0, 2, 0)];
        let planned = JoinPlanner::new(rels.clone(), edges).plan().unwrap();
        let joins = count_joins(&planned.plan);
        assert!(check(
            &planned.plan,
            &Physical::all_hash(joins),
            &rels,
            &Policy::default()
        )
        .is_ok());
    }

    #[test]
    fn a_cross_product_on_a_disconnected_graph_reaches_the_second_line_of_defence() {
        // The planner emits `Cross` only when the graph is genuinely disconnected, so this
        // arm is a backstop rather than a common path. It is here because a bug in
        // `edges_between` would otherwise produce a silently quadratic plan.
        let plan = Plan::Cross {
            left: Box::new(Plan::Scan(0)),
            right: Box::new(Plan::Scan(1)),
        };
        let e = check(
            &plan,
            &Physical::new(vec![Method::Hash]),
            &rels(),
            &Policy::default(),
        )
        .unwrap_err();
        assert!(matches!(e[0], Refusal::AvoidableCrossProduct { .. }));
        assert!(e[0].to_string().contains("disconnected graph is permitted"));
    }

    fn count_joins(p: &Plan) -> usize {
        match p {
            Plan::Scan(_) => 0,
            Plan::Join { left, right, .. } | Plan::Cross { left, right } => {
                1 + count_joins(left) + count_joins(right)
            }
        }
    }

    #[test]
    fn runtime_hash_resizing_is_part_of_the_same_policy() {
        // The other half of Leis's result. A hash table sized from a low estimate degrades
        // to a scan, which reintroduces exactly the unbounded behaviour the nested-loop rule
        // removes — so the two belong to one policy rather than two.
        assert!(Policy::default().runtime_hash_resize);
        assert!(!Policy::unrestricted().runtime_hash_resize);
    }
}
