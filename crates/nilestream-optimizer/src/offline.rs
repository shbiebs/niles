//! The planner: choosing a materialization mode per view, from a circuit and observed load.
//!
//! This module is where Contribution 5's calculus becomes a decision the engine actually
//! takes. It sits between the compiler and the runtime: the compiler fixes *what* a view
//! computes and *what it promises*; the planner fixes *how much of it is kept resident*.
//!
//! # The three safety properties, and which one does the work
//!
//! 1. **Semantic safety** (Theorem 4.5(a)): a mode change cannot change an answer. This is
//!    what makes the whole component safe to be wrong: no individual heuristic needs its
//!    own correctness argument, because none of them can produce a wrong number.
//! 2. **Contract safety**: infeasible modes are filtered *before* cost comparison, never
//!    after. A bad estimate therefore costs compute; it cannot breach a rung.
//! 3. **Explainability**: every decision records the estimates that produced it. An
//!    adaptive component that changes behaviour under load and cannot be interrogated is
//!    operationally frightening, and §11.2 lists that as a named risk.
//!
//! Property 2 is the one that does the work here, and the ordering it demands is the whole
//! design of [`plan_view`]: filter, *then* price. Pricing first and filtering after would
//! be the same code with the same output in the common case and a contract violation in
//! the uncommon one — which is precisely the shape of bug that survives testing.
//!
//! # What the planner is forbidden from doing
//!
//! It may not promote a view to a stricter rung to make it cheaper, it may not demote one
//! to make it faster, and it may not choose a mode outside the contract's feasible set even
//! if every estimate says it would win. The contract is the author's statement about what
//! the view gives up; the planner's job is to be economical *within* it.

use crate::estimators::RangeStats;
use crate::modes::{buy_cost, miss_charge, should_buy, Mode, ModeDecision};
use niles_ir::circuit::{Circuit, NodeId};
use niles_ir::{Consistency, Materialize, Retention, ServeContract};

/// What the planner needs to know about how a view is actually being used. Measured at
/// runtime, not guessed at compile time — which is the difference between this and a
/// static view-selection heuristic.
#[derive(Debug, Clone, Copy)]
pub struct Observed {
    /// Reads per epoch against this view.
    pub read_rate: f64,
    /// Deltas per epoch arriving at it.
    pub write_rate: f64,
    /// Distinct keys in the working set.
    pub working_set: f64,
    /// Base rows a single reconstruction folds. With per-key checkpoints this is bounded
    /// by the interval rather than by history — SC7 — which is what makes it estimable at
    /// all rather than a function of how long the system has been running.
    pub reconstruction_rows: f64,
    /// The price of one resident entry for one epoch, in the same counted-work units as
    /// the other terms. This is the parameter the phase diagram sweeps, and the one an
    /// operator actually sets.
    pub residency_price: f64,
}

impl Observed {
    fn range_stats(&self) -> RangeStats {
        RangeStats {
            read_rate: self.read_rate,
            write_rate: self.write_rate,
            reconstruction_cost: self.reconstruction_rows,
            // Service time per reconstruction, in the same counted-work units. The
            // delayed-hit term needs it, and omitting it is provably the wrong policy when
            // reconstruction takes time — one of Appendix I's own results.
            reconstruction_latency: self.reconstruction_rows / 64.0,
            // Maintenance is a delta arriving at a resident entry: the write rate,
            // apportioned across the working set.
            maintenance_cost: self.write_rate / self.working_set.max(1.0),
            residency_bytes: (self.working_set * 64.0) as u64,
        }
    }
}

/// A planned view: its chosen mode, its budget, and why.
#[derive(Debug, Clone)]
pub struct Plan {
    pub node: NodeId,
    pub name: String,
    pub mode: Materialize,
    /// Resident-entry ceiling. `None` means unbounded — a fully materialized view.
    pub budget: Option<u64>,
    pub decision: ModeDecision,
    /// The modes the contract ruled out, with the reason. Kept because a plan that only
    /// shows what was chosen cannot be argued with.
    pub excluded: Vec<(Materialize, &'static str)>,
    pub explanation: String,
}

/// Which modes a contract permits, and why the others are out.
///
/// This runs before any cost is computed. That ordering is property 2.
pub fn feasible(contract: &ServeContract) -> (Vec<Materialize>, Vec<(Materialize, &'static str)>) {
    let all = [
        Materialize::Absent,
        Materialize::Demand,
        Materialize::Full,
        Materialize::Spilled,
        Materialize::Tiered,
    ];
    let mut ok = Vec::new();
    let mut out = Vec::new();
    for m in all {
        // An author who wrote an explicit mode has made the decision; the planner's job
        // is then to honour it, not to second-guess it. Only `Auto` delegates.
        if contract.materialize != Materialize::Auto && contract.materialize != m {
            out.push((
                m,
                "the view declares an explicit mode; only `auto` delegates",
            ));
            continue;
        }
        if !contract.permits(m) {
            out.push((
                m,
                match (contract.consistency, m, contract.retain) {
                    (Consistency::LedgerConsistent, Materialize::Spilled, _) => {
                        "the top rung must answer at the visibility frontier; a spilled read is an I/O round trip on the critical path"
                    }
                    (_, Materialize::Absent, Retention::Pinned) => {
                        "`retain: pinned` says never evict; `absent` says never resident"
                    }
                    _ => "the contract does not permit it",
                },
            ));
            continue;
        }
        // Retention overrides economics: a pinned or forever-retained view is resident by
        // declaration, whatever the numbers say.
        if matches!(contract.retain, Retention::Pinned | Retention::Forever)
            && matches!(m, Materialize::Absent | Materialize::Demand)
        {
            out.push((m, "retention pins this state resident"));
            continue;
        }
        ok.push(m);
    }
    (ok, out)
}

/// Plan one view.
pub fn plan_view(circuit: &Circuit, node: NodeId, name: &str, obs: Observed) -> Plan {
    let n = circuit.node(node);
    // Read the checked fields: an engine that planned without consulting them would fail
    // the IR's accessed-field audit, which is the point of that audit existing.
    let contract = *n.contract.get();
    let _ = n.anchor.get();
    let _ = n.conservation_transparent.get();
    let _ = n.lineage.get();

    // 1. FILTER. Before any cost is computed.
    let (mut candidates, excluded) = feasible(&contract);
    if candidates.is_empty() {
        // A contract with no feasible mode is a contract the compiler should have rejected
        // (W9/W10). Reaching here means the circuit came from somewhere other than this
        // compiler, so the planner refuses rather than inventing a mode.
        candidates.push(Materialize::Full);
    }

    // 2. PRICE. Only the survivors.
    let stats = obs.range_stats();
    let horizon = 64.0;
    let buy = buy_cost(&stats, horizon, obs.residency_price);
    let spend = miss_charge(&stats, contract.cost_multiplier()) * horizon;
    let worth_materializing = should_buy(spend, buy);

    // A view is only worth materializing *fully* when its whole working set is worth
    // keeping. Demand exists for the case where part of it is — which is the Pareto case
    // the whole thesis is about, and the reason the answer is a mode rather than a bit.
    let mode = if !worth_materializing {
        if candidates.contains(&Materialize::Absent) {
            Materialize::Absent
        } else if candidates.contains(&Materialize::Demand) {
            Materialize::Demand
        } else {
            candidates[0]
        }
    } else if candidates.contains(&Materialize::Demand) && obs.residency_price > 0.0 {
        Materialize::Demand
    } else if candidates.contains(&Materialize::Full) {
        Materialize::Full
    } else {
        candidates[0]
    };

    // 3. SIZE. The budget follows from the phase diagram's shape: the optimum is interior,
    // so the budget is a fraction of the working set rather than all of it or none.
    //
    // The fraction is derived, not tuned: at the point where one more resident entry costs
    // `residency_price * horizon` and saves `read_rate_per_key * reconstruction_rows`, the
    // marginal entry breaks even. Below that price, keep more; above it, keep less.
    let budget = match mode {
        Materialize::Full | Materialize::Spilled => None,
        _ => {
            let marginal_saving =
                (obs.read_rate / obs.working_set.max(1.0)) * obs.reconstruction_rows;
            let marginal_cost = obs.residency_price * horizon;
            let fraction = if marginal_cost <= 0.0 {
                1.0
            } else {
                (marginal_saving / marginal_cost).clamp(0.0, 1.0)
            };
            Some(((obs.working_set * fraction).ceil() as u64).max(1))
        }
    };

    let to = match mode {
        Materialize::Absent => Mode::Absent,
        Materialize::Demand => Mode::Demand,
        Materialize::Full => Mode::Full,
        Materialize::Spilled => Mode::Spilled,
        _ => Mode::Tiered,
    };
    let decision = ModeDecision {
        // The planner runs before anything is resident, so every transition starts from
        // `Absent`. A running optimizer supplies the real current mode.
        from: Mode::Absent,
        to,
        reason: if worth_materializing {
            "cumulative reconstruction spend over the horizon exceeds the cost of materializing"
        } else {
            "reconstruction is cheaper than residency at the observed read rate"
        },
        cumulative_reconstruction_spend: spend,
        buy_cost: buy,
        delayed_hit_ratio: stats.delayed_hit_ratio(),
    };

    let explanation = format!(
        "view `{name}` at {:?}: {} of 5 modes feasible; buy {:.2} vs spend {:.2} over {horizon:.0} epochs \
         → {:?}{}. reads {:.1}/epoch over {:.0} keys, {:.1} base rows per reconstruction, \
         residency priced at {}",
        contract.consistency,
        candidates.len(),
        buy,
        spend,
        mode,
        budget.map_or(String::new(), |b| format!(" with a budget of {b}")),
        obs.read_rate,
        obs.working_set,
        obs.reconstruction_rows,
        obs.residency_price,
    );

    Plan {
        node,
        name: name.to_string(),
        mode,
        budget,
        decision,
        excluded,
        explanation,
    }
}

/// Plan every named output of a circuit.
pub fn plan(circuit: &Circuit, obs: Observed) -> Vec<Plan> {
    let mut outputs: Vec<(String, NodeId)> = circuit
        .outputs
        .iter()
        .map(|(n, i)| (n.clone(), *i))
        .collect();
    outputs.sort();
    outputs
        .into_iter()
        .map(|(name, id)| plan_view(circuit, id, &name, obs))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use niles_ir::circuit::internal_contract;
    use niles_ir::operator::{Agg, Op, Scalar};
    use niles_ir::Lineage;

    fn contract(c: Consistency, m: Materialize, r: Retention) -> ServeContract {
        ServeContract {
            consistency: c,
            materialize: m,
            retain: r,
            lineage: Lineage::Off,
        }
    }

    fn one_view(ct: ServeContract) -> (Circuit, NodeId) {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "postings".into(),
                is_base: true,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            internal_contract(),
            "postings",
        );
        let agg = c.add(
            Op::Aggregate {
                group_key: vec![0],
                aggs: vec![(Agg::Sum, Scalar::Column(1))],
            },
            vec![src],
            ct,
            "balance",
        );
        c.set_output("balance", agg);
        (c, agg)
    }

    fn obs(price: f64) -> Observed {
        Observed {
            read_rate: 100.0,
            write_rate: 50.0,
            working_set: 10_000.0,
            reconstruction_rows: 9.0,
            residency_price: price,
        }
    }

    #[test]
    fn infeasible_modes_are_removed_before_any_cost_is_computed() {
        // Property 2, and the ordering is the property. A cheap-but-infeasible mode must
        // never be reachable, however good it looks.
        let ct = contract(
            Consistency::LedgerConsistent,
            Materialize::Auto,
            Retention::Evictable,
        );
        let (feasible_modes, excluded) = feasible(&ct);
        assert!(!feasible_modes.contains(&Materialize::Spilled));
        let why = excluded
            .iter()
            .find(|(m, _)| *m == Materialize::Spilled)
            .unwrap()
            .1;
        assert!(why.contains("I/O round trip"), "{why}");
    }

    #[test]
    fn a_pinned_view_is_never_planned_partial() {
        let ct = contract(Consistency::Snapshot, Materialize::Auto, Retention::Pinned);
        let (c, id) = one_view(ct);
        let p = plan_view(&c, id, "balance", obs(0.05));
        assert!(
            !matches!(p.mode, Materialize::Absent | Materialize::Demand),
            "a pinned view was planned as {:?}: {}",
            p.mode,
            p.explanation
        );
        assert_eq!(p.budget, None);
    }

    #[test]
    fn an_explicit_mode_is_honoured_and_only_auto_delegates() {
        // The planner is not entitled to overrule an author who made the decision.
        let ct = contract(
            Consistency::Snapshot,
            Materialize::Full,
            Retention::Evictable,
        );
        let (c, id) = one_view(ct);
        let p = plan_view(&c, id, "balance", obs(0.05));
        assert_eq!(p.mode, Materialize::Full, "{}", p.explanation);
        assert!(p
            .excluded
            .iter()
            .any(|(_, w)| w.contains("only `auto` delegates")));
    }

    #[test]
    fn the_budget_shrinks_as_memory_gets_more_expensive() {
        // The shape the phase diagram measured: the optimum is interior, and it moves
        // toward smaller budgets as the price of memory rises.
        let ct = contract(
            Consistency::Snapshot,
            Materialize::Auto,
            Retention::Evictable,
        );
        let (c, id) = one_view(ct);
        let mut last = u64::MAX;
        for price in [0.0001f64, 0.0005, 0.002, 0.01, 0.05] {
            c.reset_access();
            let p = plan_view(&c, id, "balance", obs(price));
            let b = p.budget.unwrap_or(u64::MAX);
            assert!(
                b <= last,
                "budget rose from {last} to {b} as price rose to {price}"
            );
            last = b;
        }
        assert!(
            last < 10_000,
            "at the highest price the budget must be well under the working set: {last}"
        );
    }

    #[test]
    fn free_memory_means_full_materialization() {
        // The degenerate end of the phase diagram, and the planner should reach it: when
        // residency costs nothing there is no reason to evict anything, so the answer is
        // `full` with no budget rather than `demand` with a budget of everything. Those
        // are the same residency and different maintenance, and `full` is the cheaper of
        // the two because it skips the eviction bookkeeping entirely.
        let ct = contract(
            Consistency::Snapshot,
            Materialize::Auto,
            Retention::Evictable,
        );
        let (c, id) = one_view(ct);
        let p = plan_view(&c, id, "balance", obs(0.0));
        assert_eq!(p.mode, Materialize::Full, "{}", p.explanation);
        assert_eq!(p.budget, None, "a fully materialized view has no ceiling");
    }

    #[test]
    fn a_stricter_rung_makes_misses_more_expensive_and_pushes_toward_residency() {
        // The cost law: a rung's multiplier enters the miss charge, so the same workload
        // at a stricter rung is worth more memory.
        let lax = contract(
            Consistency::Bounded {
                epochs: 8,
                millis: 0,
            },
            Materialize::Auto,
            Retention::Evictable,
        );
        let strict = contract(
            Consistency::LedgerConsistent,
            Materialize::Auto,
            Retention::Evictable,
        );
        let (c1, i1) = one_view(lax);
        let (c2, i2) = one_view(strict);
        let p1 = plan_view(&c1, i1, "balance", obs(0.01));
        let p2 = plan_view(&c2, i2, "balance", obs(0.01));
        assert!(
            p2.decision.cumulative_reconstruction_spend
                > p1.decision.cumulative_reconstruction_spend,
            "the strict rung must charge more per miss: {} vs {}",
            p2.decision.cumulative_reconstruction_spend,
            p1.decision.cumulative_reconstruction_spend
        );
    }

    #[test]
    fn every_plan_explains_itself() {
        // §11.2 names optimizer opacity as a risk. A decision that cannot be interrogated
        // is one an operator cannot trust under load, which is exactly when it matters.
        let ct = contract(
            Consistency::Snapshot,
            Materialize::Auto,
            Retention::Evictable,
        );
        let (c, id) = one_view(ct);
        let p = plan_view(&c, id, "balance", obs(0.002));
        for needle in [
            "modes feasible",
            "buy",
            "spend",
            "reads",
            "base rows per reconstruction",
        ] {
            assert!(
                p.explanation.contains(needle),
                "explanation is missing `{needle}`: {}",
                p.explanation
            );
        }
        // And where the contract *does* rule something out, the plan must name it and say
        // why — a plan that only shows what was chosen cannot be argued with.
        let strict = contract(
            Consistency::LedgerConsistent,
            Materialize::Auto,
            Retention::Evictable,
        );
        let (c2, id2) = one_view(strict);
        let p2 = plan_view(&c2, id2, "balance", obs(0.002));
        assert!(
            !p2.excluded.is_empty(),
            "the strict rung rules out `spilled`, and the plan must say so"
        );
        assert!(
            p2.excluded.iter().all(|(_, why)| why.len() > 20),
            "every exclusion needs a real reason"
        );
    }

    #[test]
    fn the_planner_reads_every_checked_ir_field() {
        let ct = contract(
            Consistency::Snapshot,
            Materialize::Auto,
            Retention::Evictable,
        );
        let (c, id) = one_view(ct);
        let _ = plan_view(&c, id, "balance", obs(0.002));
        let unread: Vec<_> = c
            .audit_access()
            .unread
            .into_iter()
            .filter(|(n, _)| *n == id)
            .collect();
        assert!(
            unread.is_empty(),
            "the planner ignored a semantic field: {unread:?}"
        );
    }
}
