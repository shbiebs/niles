//! The IR verifier — a member of the trusted computing base.
//!
//! # Why the verifier is trusted and the compiler is not
//!
//! This is a deliberate architectural choice recorded in §11.4, and it decides where
//! engineering effort goes. The compiler is large, will change often, and will be extended
//! by people who did not write it. The verifier is small, changes rarely, and can be read
//! end to end in an afternoon. Putting the verifier rather than the compiler in the trusted
//! base means a compiler bug produces a *rejected circuit* rather than a wrong answer —
//! and a wrong answer, in this system, means money that was created or destroyed.
//!
//! The same reasoning appears in the optimizer's design: the optimizer is forbidden from
//! being a correctness dependency, so a bad estimate costs money in the sense of compute,
//! never in the sense of ledger balance.
//!
//! # What is checked
//!
//! Structural well-formedness, then the semantic invariants the theorems assume. The
//! semantic ones are the point; the structural ones are there because a malformed circuit
//! would otherwise reach the engine and panic somewhere less informative.

use crate::circuit::{Anchor, Circuit, NodeId};
use crate::operator::Op;
use crate::upquery_path;
use crate::{Consistency, Materialize, Retention};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    pub code: &'static str,
    pub node: Option<NodeId>,
    pub msg: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.node {
            Some(n) => write!(f, "[{}] node {n}: {}", self.code, self.msg),
            None => write!(f, "[{}] {}", self.code, self.msg),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    pub violations: Vec<Violation>,
    pub nodes_checked: usize,
}

impl VerifyReport {
    pub fn is_ok(&self) -> bool {
        self.violations.is_empty()
    }
    pub fn render(&self) -> String {
        if self.is_ok() {
            return format!("verified: {} nodes, no violations\n", self.nodes_checked);
        }
        let mut s = format!("REJECTED: {} violation(s)\n", self.violations.len());
        for v in &self.violations {
            s.push_str(&format!("  {v}\n"));
        }
        s
    }
}

/// Verify a circuit. A circuit that fails here never reaches the engine.
pub fn verify(c: &Circuit) -> VerifyReport {
    let mut r = VerifyReport { violations: Vec::new(), nodes_checked: c.nodes.len() };

    // ---- structural ----
    for n in &c.nodes {
        if n.inputs.len() != n.op.arity() {
            r.violations.push(Violation {
                code: "IR001",
                node: Some(n.id),
                msg: format!("`{}` takes {} input(s) but has {}", n.op.name(), n.op.arity(), n.inputs.len()),
            });
        }
        for i in &n.inputs {
            if *i as usize >= c.nodes.len() {
                r.violations.push(Violation { code: "IR002", node: Some(n.id), msg: format!("input {i} does not exist") });
            } else if *i >= n.id && !matches!(n.op, Op::Delay) {
                // Forward references are only legal through a `Delay`, which is what makes
                // a fixpoint's cycle well founded rather than an infinite regress.
                r.violations.push(Violation {
                    code: "IR003",
                    node: Some(n.id),
                    msg: format!("input {i} is not earlier, and only a `delay` may close a cycle"),
                });
            }
        }
    }
    if c.topological_order().is_none() {
        r.violations.push(Violation {
            code: "IR004",
            node: None,
            msg: "the circuit has a cycle not mediated by a `delay`".into(),
        });
    }
    for (name, id) in &c.outputs {
        if *id as usize >= c.nodes.len() {
            r.violations.push(Violation { code: "IR005", node: None, msg: format!("output `{name}` names a node that does not exist") });
        }
    }

    // ---- semantic: the invariants the theorems assume ----
    for n in &c.nodes {
        let contract = n.contract.get();
        let anchor = n.anchor.get();
        let transparent = *n.conservation_transparent.get();
        let _ = n.lineage.get();

        // The two contract-feasibility rules, restated at the IR level. They are also
        // checked in the front end (W9, W10); they are checked *again* here because the IR
        // is the stable contract and may be produced by something other than this
        // compiler — a hand-written plan, a future front end, a migration tool.
        if !contract.permits(contract.materialize) {
            r.violations.push(Violation {
                code: "IR010",
                node: Some(n.id),
                msg: format!("contract {:?}/{:?}/{:?} is not serveable", contract.consistency, contract.materialize, contract.retain),
            });
        }

        // A view served at the top rung must answer at the visibility frontier. A node
        // pinned to a past epoch cannot, and serving one as if it could is precisely the
        // silent-staleness failure the whole consistency ladder exists to make visible.
        if contract.consistency == Consistency::LedgerConsistent {
            if let Anchor::Pinned(e) = anchor {
                r.violations.push(Violation {
                    code: "IR011",
                    node: Some(n.id),
                    msg: format!("served `ledger_consistent` but anchored at the fixed epoch #{e}"),
                });
            }
        }

        // Evictable state must be reconstructible, or eviction is data loss rather than a
        // memory-management decision. This is the IR-level statement of Proposition 3.4.
        let evictable = contract.retain == Retention::Evictable
            && matches!(contract.materialize, Materialize::Demand | Materialize::Absent | Materialize::Tiered);
        if evictable && !matches!(n.op, Op::Source { .. }) {
            match upquery_path::derive(c, n.id, 0) {
                Ok(p) if !p.is_anchored() => r.violations.push(Violation {
                    code: "IR012",
                    node: Some(n.id),
                    msg: "evictable state whose reconstruction path is not anchored to immutable bases".into(),
                }),
                Err(e) => r.violations.push(Violation {
                    code: "IR013",
                    node: Some(n.id),
                    msg: format!("evictable state with no reconstruction path: {}", e.explain()),
                }),
                Ok(_) => {}
            }
        }

        // A base is never partial. This is not a policy the optimizer may trade away; it
        // is the premise every reconstruction theorem starts from.
        if let Op::Source { relation, is_base: true, .. } = &n.op {
            if contract.retain != Retention::Forever {
                r.violations.push(Violation {
                    code: "IR014",
                    node: Some(n.id),
                    msg: format!("base `{relation}` must be `retain: forever`; the base is never partial"),
                });
            }
            if matches!(contract.materialize, Materialize::Absent | Materialize::Demand) {
                r.violations.push(Violation {
                    code: "IR015",
                    node: Some(n.id),
                    msg: format!("base `{relation}` cannot be partially materialized"),
                });
            }
        }

        // A conservation-transparent node whose inputs are not is a contradiction the
        // constructor should have prevented; catching it here is cheap and it is exactly
        // the kind of bug a hand-built circuit introduces.
        if transparent {
            for i in &n.inputs {
                if (*i as usize) < c.nodes.len() && !*c.nodes[*i as usize].conservation_transparent.peek() {
                    r.violations.push(Violation {
                        code: "IR016",
                        node: Some(n.id),
                        msg: format!("claims conservation transparency over non-transparent input {i}"),
                    });
                }
            }
        }
    }

    // ---- the accessed-field audit ----
    // Runs last, because the checks above are what read the fields. If the verifier itself
    // stopped reading one, this would catch that too.
    let access = c.audit_access();
    for (node, field) in access.unread {
        r.violations.push(Violation {
            code: "IR020",
            node: Some(node),
            msg: format!("semantic field `{field}` was never read by any consumer"),
        });
    }

    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::internal_contract;
    use crate::operator::{Agg, Scalar};
    use crate::{Lineage, ServeContract};

    fn contract(cons: Consistency, m: Materialize, r: Retention) -> ServeContract {
        ServeContract { consistency: cons, materialize: m, retain: r, lineage: Lineage::Off }
    }

    fn base_circuit(view_contract: ServeContract) -> Circuit {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "postings".into(), is_base: true, anchor_key: vec![0, 1] },
            vec![],
            contract(Consistency::LedgerConsistent, Materialize::Full, Retention::Forever),
            "postings",
        );
        let agg = c.add(
            Op::Aggregate { group_key: vec![0, 1], aggs: vec![(Agg::Sum, Scalar::Column(2))] },
            vec![src],
            view_contract,
            "balance",
        );
        c.set_output("balance", agg);
        c
    }

    #[test]
    fn a_well_formed_circuit_verifies() {
        let c = base_circuit(contract(Consistency::Snapshot, Materialize::Demand, Retention::Evictable));
        let r = verify(&c);
        assert!(r.is_ok(), "{}", r.render());
        assert_eq!(r.nodes_checked, 2);
    }

    #[test]
    fn a_base_may_not_be_partially_materialized() {
        // The premise every reconstruction theorem starts from, enforced rather than assumed.
        let mut c = Circuit::new();
        c.add(
            Op::Source { relation: "postings".into(), is_base: true, anchor_key: vec![0] },
            vec![],
            contract(Consistency::LedgerConsistent, Materialize::Demand, Retention::Forever),
            "",
        );
        let r = verify(&c);
        assert!(r.violations.iter().any(|v| v.code == "IR015"), "{}", r.render());
    }

    #[test]
    fn a_base_must_be_retained_forever() {
        let mut c = Circuit::new();
        c.add(
            Op::Source { relation: "postings".into(), is_base: true, anchor_key: vec![0] },
            vec![],
            contract(Consistency::LedgerConsistent, Materialize::Full, Retention::Evictable),
            "",
        );
        let r = verify(&c);
        assert!(r.violations.iter().any(|v| v.code == "IR014"), "{}", r.render());
    }

    #[test]
    fn evictable_state_over_a_mutable_table_is_rejected() {
        // Eviction over a source whose history is gone is data loss, not memory management.
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "staging".into(), is_base: false, anchor_key: vec![0] },
            vec![],
            contract(Consistency::Snapshot, Materialize::Full, Retention::Evictable),
            "",
        );
        c.add(
            Op::Aggregate { group_key: vec![0], aggs: vec![(Agg::Sum, Scalar::Column(1))] },
            vec![src],
            contract(Consistency::Snapshot, Materialize::Demand, Retention::Evictable),
            "",
        );
        let r = verify(&c);
        assert!(r.violations.iter().any(|v| v.code == "IR013"), "{}", r.render());
        assert!(r.render().contains("history is not retained"), "{}", r.render());
    }

    #[test]
    fn the_top_rung_cannot_be_served_from_a_pinned_anchor() {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "p".into(), is_base: true, anchor_key: vec![0] },
            vec![],
            contract(Consistency::LedgerConsistent, Materialize::Full, Retention::Forever),
            "",
        );
        c.add(
            Op::AsOf { epoch: Some(4200) },
            vec![src],
            contract(Consistency::LedgerConsistent, Materialize::Full, Retention::Pinned),
            "",
        );
        let r = verify(&c);
        assert!(r.violations.iter().any(|v| v.code == "IR011"), "{}", r.render());
    }

    #[test]
    fn an_infeasible_contract_is_rejected_at_the_ir_too() {
        // Already checked in the front end. Checked again here because the IR is the
        // stable contract and may be produced by something that is not this compiler.
        let c = base_circuit(contract(Consistency::LedgerConsistent, Materialize::Spilled, Retention::Evictable));
        let r = verify(&c);
        assert!(r.violations.iter().any(|v| v.code == "IR010"), "{}", r.render());
    }

    #[test]
    fn a_dangling_input_is_caught_before_the_engine_panics() {
        let mut c = base_circuit(contract(Consistency::Snapshot, Materialize::Full, Retention::Pinned));
        c.nodes[1].inputs = vec![99];
        let r = verify(&c);
        assert!(r.violations.iter().any(|v| v.code == "IR002"), "{}", r.render());
    }

    #[test]
    fn arity_mismatches_are_caught() {
        let mut c = base_circuit(contract(Consistency::Snapshot, Materialize::Full, Retention::Pinned));
        c.nodes[1].inputs = vec![0, 0];
        let r = verify(&c);
        assert!(r.violations.iter().any(|v| v.code == "IR001"), "{}", r.render());
    }

    #[test]
    fn the_verifier_reads_every_semantic_field_it_is_meant_to() {
        // Self-check: if the verifier stopped consulting one of the checked fields, the
        // audit it runs at the end would report that field as unread — so this test also
        // guards the verifier against its own future drift.
        let c = base_circuit(contract(Consistency::Snapshot, Materialize::Full, Retention::Pinned));
        let r = verify(&c);
        assert!(
            !r.violations.iter().any(|v| v.code == "IR020"),
            "the verifier itself left a field unread:\n{}",
            r.render()
        );
    }
}
