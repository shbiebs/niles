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
use crate::operator::{ColIdx, JoinKind, Op, Scalar};
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

/// Whether a node is reachable from any named output — i.e. whether it is *served*.
///
/// An unreachable node is dead plan, and holding an `Apply` in dead plan is not a
/// correctness problem; holding one on a path a reader can query is.
fn reaches_output(c: &Circuit, id: NodeId) -> bool {
    c.live_nodes().contains(&id)
}

/// Verify a circuit. A circuit that fails here never reaches the engine.
pub fn verify(c: &Circuit) -> VerifyReport {
    let mut r = VerifyReport {
        violations: Vec::new(),
        nodes_checked: c.nodes.len(),
    };

    // ---- structural ----
    for n in &c.nodes {
        if n.inputs.len() != n.op.arity() {
            r.violations.push(Violation {
                code: "IR001",
                node: Some(n.id),
                msg: format!(
                    "`{}` takes {} input(s) but has {}",
                    n.op.name(),
                    n.op.arity(),
                    n.inputs.len()
                ),
            });
        }
        for i in &n.inputs {
            if *i as usize >= c.nodes.len() {
                r.violations.push(Violation {
                    code: "IR002",
                    node: Some(n.id),
                    msg: format!("input {i} does not exist"),
                });
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
            r.violations.push(Violation {
                code: "IR005",
                node: None,
                msg: format!("output `{name}` names a node that does not exist"),
            });
        }
    }

    // ---- every operator in the circuit has an implementation ----
    r.violations.extend(unevaluable(c, crate::eval::implements));

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
                msg: format!(
                    "contract {:?}/{:?}/{:?} is not serveable",
                    contract.consistency, contract.materialize, contract.retain
                ),
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
            && matches!(
                contract.materialize,
                Materialize::Demand | Materialize::Absent | Materialize::Tiered
            );
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
        if let Op::Source {
            relation,
            is_base: true,
            ..
        } = &n.op
        {
            if contract.retain != Retention::Forever {
                r.violations.push(Violation {
                    code: "IR014",
                    node: Some(n.id),
                    msg: format!(
                        "base `{relation}` must be `retain: forever`; the base is never partial"
                    ),
                });
            }
            if matches!(
                contract.materialize,
                Materialize::Absent | Materialize::Demand
            ) {
                r.violations.push(Violation {
                    code: "IR015",
                    node: Some(n.id),
                    msg: format!("base `{relation}` cannot be partially materialized"),
                });
            }
        }

        // **The duplicate-inflation rule.** A semi- or anti-join emits *left rows*, so its
        // output width is the left input's. A node claiming the sum of both widths is one
        // whose implementation concatenated — that is, an inner join wearing a semi-join's
        // label — and an inner join in a semi-join's place turns one outer row into as many
        // rows as matched it. In an `exists` over postings that is one account balance
        // reported three times, and no answer-level test catches it because every one of
        // the three is a correct row.
        if let Op::Join { kind, .. } = &n.op {
            if matches!(kind, JoinKind::Semi | JoinKind::Anti) {
                let left = n
                    .inputs
                    .first()
                    .and_then(|i| c.nodes.get(*i as usize))
                    .map(|x| x.arity);
                if let Some(w) = left {
                    if n.arity != w {
                        r.violations.push(Violation {
                            code: "IR017",
                            node: Some(n.id),
                            msg: format!(
                                "a {kind:?} join emits left rows and must have arity {w}, not {}; \
                                 a wider one is an inner join that will duplicate them",
                                n.arity
                            ),
                        });
                    }
                }
            }
        }

        // **A dependent join cannot be served.** `Apply` is the *nested* form of a
        // correlated subquery: one new outer row re-scans the inner relation, so there is
        // no delta rule and therefore no incremental maintenance. Unnesting is not an
        // optimisation that may be skipped when the optimizer is busy; it is the step that
        // makes the query maintainable at all, and a circuit that still holds an `Apply`
        // on a path to a named output has not had it.
        if let Op::Apply { kind, correlation } = &n.op {
            if reaches_output(c, n.id) {
                r.violations.push(Violation {
                    code: "IR018",
                    node: Some(n.id),
                    msg: format!(
                        "`apply({})` is reachable from a served output; a dependent join has \
                         no delta rule, so the circuit must be unnested before it is served",
                        kind.name()
                    ),
                });
            }
            let widths: Vec<u16> = n
                .inputs
                .iter()
                .filter_map(|i| c.nodes.get(*i as usize))
                .map(|x| x.arity)
                .collect();
            if widths.len() == 2 {
                for (o, i) in correlation {
                    if *o >= widths[0] || *i >= widths[1] {
                        r.violations.push(Violation {
                            code: "IR019",
                            node: Some(n.id),
                            msg: format!(
                                "correlation ({o}, {i}) is outside the inputs' widths ({}, {})",
                                widths[0], widths[1]
                            ),
                        });
                    }
                }
            }
        }

        // A conservation-transparent node whose inputs are not is a contradiction the
        // constructor should have prevented; catching it here is cheap and it is exactly
        // the kind of bug a hand-built circuit introduces.
        if transparent {
            for i in &n.inputs {
                if (*i as usize) < c.nodes.len()
                    && !*c.nodes[*i as usize].conservation_transparent.peek()
                {
                    r.violations.push(Violation {
                        code: "IR016",
                        node: Some(n.id),
                        msg: format!(
                            "claims conservation transparency over non-transparent input {i}"
                        ),
                    });
                }
            }
        }
    }

    // ---- confidentiality (IR022) ----
    for v in confidential_use(c) {
        r.violations.push(v);
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

/// **A circuit may not compute on a column the schema sealed.**
///
/// The rule the type checker enforces, enforced again here on the artefact the engine
/// actually runs — because for two years it was enforced on *one of two surfaces*:
/// `postings.where(|p| p.legal_name == x)` was refused and `select … where legal_name = 'x'`
/// compiled, so the one static confidentiality guarantee the design makes was false on the
/// surface every wire client uses. A guarantee that lives in a single checker is a guarantee
/// with a single point of silent failure, and this is the second.
///
/// Confidentiality is propagated through the circuit rather than read only at the source: a
/// `Map` output is sealed if its expression reads a sealed column, an `Aggregate` key column
/// inherits from the column it groups, and an aggregate's output is sealed if its argument
/// was — so laundering a sealed value through a projection and then filtering on it is
/// refused too. Carrying a sealed value to the output is the one legal use; every predicate,
/// key, join key and aggregate argument is a violation.
fn confidential_use(c: &Circuit) -> Vec<Violation> {
    use std::collections::BTreeMap;
    let mut sealed: BTreeMap<NodeId, Vec<bool>> = BTreeMap::new();
    let mut out = Vec::new();

    // Nodes are in dependency order except through `Delay`, which the structural rules above
    // already police, so one forward pass suffices.
    for n in &c.nodes {
        let input_sealed = |m: &BTreeMap<NodeId, Vec<bool>>, k: usize| -> Vec<bool> {
            n.inputs
                .get(k)
                .and_then(|i| m.get(i))
                .cloned()
                .unwrap_or_default()
        };
        let reads_sealed = |sc: &Scalar, cols: &[bool]| -> bool {
            let mut hit = false;
            scalar_columns(sc, &mut |i| {
                if cols.get(i as usize).copied().unwrap_or(false) {
                    hit = true;
                }
            });
            hit
        };
        let mut violate = |code: &'static str, msg: String| {
            out.push(Violation {
                code,
                node: Some(n.id),
                msg,
            })
        };

        let here: Vec<bool> = match &n.op {
            Op::Source {
                confidential,
                anchor_key: _,
                ..
            } => {
                let width = confidential
                    .iter()
                    .map(|i| *i as usize + 1)
                    .max()
                    .unwrap_or(0);
                let mut v = vec![false; width.max(n.arity as usize)];
                for i in confidential {
                    if (*i as usize) < v.len() {
                        v[*i as usize] = true;
                    }
                }
                v
            }
            Op::Filter { predicate } => {
                let cols = input_sealed(&sealed, 0);
                if reads_sealed(predicate, &cols) {
                    violate(
                        "IR022",
                        "a filter predicate reads a column declared `@confidential`; the \
                         engine cannot compute on ciphertext, so the filter would either not \
                         filter or leak through timing"
                            .into(),
                    );
                }
                cols
            }
            Op::Map { exprs } => {
                let cols = input_sealed(&sealed, 0);
                exprs.iter().map(|e| reads_sealed(e, &cols)).collect()
            }
            Op::Aggregate { group_key, aggs } => {
                let cols = input_sealed(&sealed, 0);
                for k in group_key {
                    if cols.get(*k as usize).copied().unwrap_or(false) {
                        violate(
                            "IR022",
                            format!(
                                "column {k} is declared `@confidential` and is a grouping key; \
                                 grouping is an equality test the engine cannot perform on a \
                                 value it cannot read"
                            ),
                        );
                    }
                }
                let mut v: Vec<bool> = group_key
                    .iter()
                    .map(|k| cols.get(*k as usize).copied().unwrap_or(false))
                    .collect();
                for (_, arg) in aggs {
                    if reads_sealed(arg, &cols) {
                        violate(
                            "IR022",
                            "an aggregate's argument reads a column declared `@confidential`"
                                .into(),
                        );
                    }
                    v.push(false);
                }
                v
            }
            Op::Index { key } => {
                let cols = input_sealed(&sealed, 0);
                for k in key {
                    if cols.get(*k as usize).copied().unwrap_or(false) {
                        violate(
                            "IR022",
                            format!("column {k} is declared `@confidential` and is an index key"),
                        );
                    }
                }
                cols
            }
            Op::OrderBy { keys } => {
                let cols = input_sealed(&sealed, 0);
                for (k, _) in keys {
                    if cols.get(*k as usize).copied().unwrap_or(false) {
                        violate(
                            "IR022",
                            format!(
                                "column {k} is declared `@confidential` and is an ordering key; \
                                 an order over ciphertext is an order over nothing the reader \
                                 asked for"
                            ),
                        );
                    }
                }
                cols
            }
            Op::Join {
                left_key,
                right_key,
                residual,
                ..
            } => {
                let l = input_sealed(&sealed, 0);
                let rgt = input_sealed(&sealed, 1);
                for k in left_key {
                    if l.get(*k as usize).copied().unwrap_or(false) {
                        violate("IR022", format!("left join key {k} is `@confidential`"));
                    }
                }
                for k in right_key {
                    if rgt.get(*k as usize).copied().unwrap_or(false) {
                        violate("IR022", format!("right join key {k} is `@confidential`"));
                    }
                }
                let mut both = l.clone();
                both.extend(rgt.iter().copied());
                if let Some(res) = residual {
                    if reads_sealed(res, &both) {
                        violate(
                            "IR022",
                            "a join's residual predicate reads a `@confidential` column".into(),
                        );
                    }
                }
                both
            }
            // Everything else passes its input's sealing through unchanged: these operators
            // move rows, they do not read columns.
            _ => input_sealed(&sealed, 0),
        };
        sealed.insert(n.id, here);
    }
    out
}

/// Every column index a scalar reads.
fn scalar_columns(s: &Scalar, f: &mut impl FnMut(ColIdx)) {
    match s {
        Scalar::Column(i) => f(*i),
        Scalar::Binary { lhs, rhs, .. } => {
            scalar_columns(lhs, f);
            scalar_columns(rhs, f);
        }
        Scalar::IsNull(x) | Scalar::Not(x) | Scalar::Neg(x) => scalar_columns(x, f),
        Scalar::Udf { args, .. } => args.iter().for_each(|a| scalar_columns(a, f)),
        _ => {}
    }
}

/// **Operators the circuit contains and the evaluator cannot evaluate.**
///
/// Takes the predicate rather than calling [`crate::eval::implements`] directly, so a test
/// can hold the *rule* — "an operator with no arm is refused" — rather than only the current
/// answer. With every variant implemented the rule is unobservable from outside, and a rule
/// nothing can observe is a rule nothing is holding.
///
/// The class this exists for: `RIGHT` and `FULL` joins once evaluated to nothing at all and
/// `CROSS` answered the equi-join, and all three parsed, lowered and passed this verifier.
/// Type coherence, effect rows, contract feasibility and guardedness were all checked; that
/// the operator had somewhere to go was not.
pub fn unevaluable(c: &Circuit, implemented: impl Fn(&Op) -> bool) -> Vec<Violation> {
    c.nodes
        .iter()
        .filter(|n| !implemented(&n.op))
        .map(|n| Violation {
            code: "IR021",
            node: Some(n.id),
            msg: format!(
                "`{}` has no arm in the reference evaluator, so a circuit containing it has \
                 no denotation to be checked against",
                n.op.name()
            ),
        })
        .collect()
}

#[cfg(test)]
mod confidentiality {
    use super::*;
    use crate::circuit::Circuit;
    use crate::operator::{Agg, Scalar};
    use crate::{Consistency, Materialize, Retention, ServeContract};

    fn contract() -> ServeContract {
        ServeContract {
            consistency: Consistency::Snapshot,
            materialize: Materialize::Auto,
            retain: Retention::Forever,
            lineage: crate::Lineage::Key,
        }
    }

    /// A source with column 1 sealed, and `f` applied to it.
    fn circuit_with(op: Op) -> Circuit {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "p".into(),
                is_base: true,
                anchor_key: vec![],
                confidential: vec![1],
            },
            vec![],
            contract(),
            "p",
        );
        let n = c.add(op, vec![src], contract(), "under test");
        c.outputs.insert("v".into(), n);
        c
    }

    fn codes(c: &Circuit) -> Vec<&'static str> {
        super::confidential_use(c)
            .into_iter()
            .map(|v| v.code)
            .collect()
    }

    /// **The rule the type checker enforces, enforced again on the circuit.**
    ///
    /// Written against hand-built circuits rather than through the compiler on purpose: the
    /// point of this rule is that it holds for a circuit *whatever produced it*, including
    /// one an optimizer rewrote or a future surface emitted. A guarantee that only holds for
    /// circuits the current front end happens to build is not a guarantee about the engine.
    #[test]
    fn a_circuit_may_not_compute_on_a_sealed_column() {
        assert_eq!(
            codes(&circuit_with(Op::Filter {
                predicate: Scalar::Binary {
                    op: crate::operator::ScalarOp::Eq,
                    lhs: Box::new(Scalar::Column(1)),
                    rhs: Box::new(Scalar::LitInt(1)),
                },
            })),
            vec!["IR022"],
            "a predicate over a sealed column"
        );
        assert_eq!(
            codes(&circuit_with(Op::Aggregate {
                group_key: vec![1],
                aggs: vec![(Agg::Count, Scalar::Column(0))],
            })),
            vec!["IR022"],
            "a sealed grouping key"
        );
        assert_eq!(
            codes(&circuit_with(Op::Aggregate {
                group_key: vec![0],
                aggs: vec![(Agg::Sum, Scalar::Column(1))],
            })),
            vec!["IR022"],
            "a sealed aggregate argument"
        );
        assert_eq!(
            codes(&circuit_with(Op::OrderBy {
                keys: vec![(1, true)],
            })),
            vec!["IR022"],
            "a sealed ordering key"
        );
        assert!(
            codes(&circuit_with(Op::Map {
                exprs: vec![Scalar::Column(0), Scalar::Column(1)],
            }))
            .is_empty(),
            "carrying a sealed value to the output is the one legal use"
        );
    }

    /// Laundering: project the sealed column through a `Map`, then filter on the copy.
    /// Refused, because confidentiality propagates with the value rather than staying on the
    /// column index it started at.
    #[test]
    fn a_sealed_value_does_not_become_readable_by_being_projected() {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "p".into(),
                is_base: true,
                anchor_key: vec![],
                confidential: vec![1],
            },
            vec![],
            contract(),
            "p",
        );
        let m = c.add(
            Op::Map {
                exprs: vec![Scalar::Column(1)],
            },
            vec![src],
            contract(),
            "launder",
        );
        let f = c.add(
            Op::Filter {
                predicate: Scalar::Binary {
                    op: crate::operator::ScalarOp::Eq,
                    lhs: Box::new(Scalar::Column(0)),
                    rhs: Box::new(Scalar::LitInt(1)),
                },
            },
            vec![m],
            contract(),
            "filter",
        );
        c.outputs.insert("v".into(), f);
        assert_eq!(codes(&c), vec!["IR022"]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::{Agg, Scalar};
    use crate::{Lineage, ServeContract};

    fn contract(cons: Consistency, m: Materialize, r: Retention) -> ServeContract {
        ServeContract {
            consistency: cons,
            materialize: m,
            retain: r,
            lineage: Lineage::Off,
        }
    }

    #[test]
    fn an_operator_with_no_evaluator_arm_is_refused_before_it_can_answer_wrongly() {
        // The rule, held against a predicate rather than against today's answer: every
        // variant is implemented, so with the real predicate this check can never fire and a
        // test of the real predicate would be a test of nothing.
        //
        // Why the rule earns its place: `RIGHT` and `FULL` joins once evaluated to *nothing
        // at all* and `CROSS` answered the equi-join, and all three parsed, lowered and
        // passed this verifier — which checked types, effects, contracts and guardedness,
        // and not whether the operator it was letting through had anywhere to go.
        let c = base_circuit(contract(
            Consistency::Snapshot,
            Materialize::Full,
            Retention::Forever,
        ));
        assert!(
            unevaluable(&c, crate::eval::implements).is_empty(),
            "every operator this compiler emits is implemented today"
        );

        let missing = unevaluable(&c, |op| !matches!(op, Op::Aggregate { .. }));
        assert_eq!(
            missing.len(),
            1,
            "an operator with no arm must be named, not passed"
        );
        assert_eq!(missing[0].code, "IR021");
        assert!(
            missing[0].msg.contains("no denotation"),
            "the reason a reader needs is *why* it matters: {}",
            missing[0].msg
        );

        // And it reaches `verify` rather than sitting in a helper nobody calls.
        assert!(verify(&c).is_ok());
    }

    fn base_circuit(view_contract: ServeContract) -> Circuit {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "postings".into(),
                is_base: true,
                anchor_key: vec![0, 1],
                confidential: Vec::new(),
            },
            vec![],
            contract(
                Consistency::LedgerConsistent,
                Materialize::Full,
                Retention::Forever,
            ),
            "postings",
        );
        let agg = c.add(
            Op::Aggregate {
                group_key: vec![0, 1],
                aggs: vec![(Agg::Sum, Scalar::Column(2))],
            },
            vec![src],
            view_contract,
            "balance",
        );
        c.set_output("balance", agg);
        c
    }

    #[test]
    fn a_well_formed_circuit_verifies() {
        let c = base_circuit(contract(
            Consistency::Snapshot,
            Materialize::Demand,
            Retention::Evictable,
        ));
        let r = verify(&c);
        assert!(r.is_ok(), "{}", r.render());
        assert_eq!(r.nodes_checked, 2);
    }

    #[test]
    fn a_base_may_not_be_partially_materialized() {
        // The premise every reconstruction theorem starts from, enforced rather than assumed.
        let mut c = Circuit::new();
        c.add(
            Op::Source {
                relation: "postings".into(),
                is_base: true,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            contract(
                Consistency::LedgerConsistent,
                Materialize::Demand,
                Retention::Forever,
            ),
            "",
        );
        let r = verify(&c);
        assert!(
            r.violations.iter().any(|v| v.code == "IR015"),
            "{}",
            r.render()
        );
    }

    #[test]
    fn a_base_must_be_retained_forever() {
        let mut c = Circuit::new();
        c.add(
            Op::Source {
                relation: "postings".into(),
                is_base: true,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            contract(
                Consistency::LedgerConsistent,
                Materialize::Full,
                Retention::Evictable,
            ),
            "",
        );
        let r = verify(&c);
        assert!(
            r.violations.iter().any(|v| v.code == "IR014"),
            "{}",
            r.render()
        );
    }

    #[test]
    fn evictable_state_over_a_mutable_table_is_rejected() {
        // Eviction over a source whose history is gone is data loss, not memory management.
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "staging".into(),
                is_base: false,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            contract(
                Consistency::Snapshot,
                Materialize::Full,
                Retention::Evictable,
            ),
            "",
        );
        c.add(
            Op::Aggregate {
                group_key: vec![0],
                aggs: vec![(Agg::Sum, Scalar::Column(1))],
            },
            vec![src],
            contract(
                Consistency::Snapshot,
                Materialize::Demand,
                Retention::Evictable,
            ),
            "",
        );
        let r = verify(&c);
        assert!(
            r.violations.iter().any(|v| v.code == "IR013"),
            "{}",
            r.render()
        );
        assert!(
            r.render().contains("history is not retained"),
            "{}",
            r.render()
        );
    }

    #[test]
    fn the_top_rung_cannot_be_served_from_a_pinned_anchor() {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source {
                relation: "p".into(),
                is_base: true,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            contract(
                Consistency::LedgerConsistent,
                Materialize::Full,
                Retention::Forever,
            ),
            "",
        );
        c.add(
            Op::AsOf { epoch: Some(4200) },
            vec![src],
            contract(
                Consistency::LedgerConsistent,
                Materialize::Full,
                Retention::Pinned,
            ),
            "",
        );
        let r = verify(&c);
        assert!(
            r.violations.iter().any(|v| v.code == "IR011"),
            "{}",
            r.render()
        );
    }

    #[test]
    fn an_infeasible_contract_is_rejected_at_the_ir_too() {
        // Already checked in the front end. Checked again here because the IR is the
        // stable contract and may be produced by something that is not this compiler.
        let c = base_circuit(contract(
            Consistency::LedgerConsistent,
            Materialize::Spilled,
            Retention::Evictable,
        ));
        let r = verify(&c);
        assert!(
            r.violations.iter().any(|v| v.code == "IR010"),
            "{}",
            r.render()
        );
    }

    #[test]
    fn a_dangling_input_is_caught_before_the_engine_panics() {
        let mut c = base_circuit(contract(
            Consistency::Snapshot,
            Materialize::Full,
            Retention::Pinned,
        ));
        c.nodes[1].inputs = vec![99];
        let r = verify(&c);
        assert!(
            r.violations.iter().any(|v| v.code == "IR002"),
            "{}",
            r.render()
        );
    }

    #[test]
    fn arity_mismatches_are_caught() {
        let mut c = base_circuit(contract(
            Consistency::Snapshot,
            Materialize::Full,
            Retention::Pinned,
        ));
        c.nodes[1].inputs = vec![0, 0];
        let r = verify(&c);
        assert!(
            r.violations.iter().any(|v| v.code == "IR001"),
            "{}",
            r.render()
        );
    }

    #[test]
    fn the_verifier_reads_every_semantic_field_it_is_meant_to() {
        // Self-check: if the verifier stopped consulting one of the checked fields, the
        // audit it runs at the end would report that field as unread — so this test also
        // guards the verifier against its own future drift.
        let c = base_circuit(contract(
            Consistency::Snapshot,
            Materialize::Full,
            Retention::Pinned,
        ));
        let r = verify(&c);
        assert!(
            !r.violations.iter().any(|v| v.code == "IR020"),
            "the verifier itself left a field unread:\n{}",
            r.render()
        );
    }
}
