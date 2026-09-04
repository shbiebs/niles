//! **Verified schedules.** A schedule is a sequence of catalogue rewrites, and this module
//! checks that each was applied where its side condition holds.
//!
//! # What this replaced, and why the earlier design could not have worked
//!
//! `SPEC-ENGINE.md` E-opt-3 and `ROADMAP.md` Phase 5 originally asked for a verifier that
//! *decides* whether a schedule "preserves the algorithm's denotation" over the operator set.
//! The architecture review recorded why that is unbuildable rather than merely hard.
//!
//! The operator set includes [`Op::Negate`](crate::Op::Negate) — Z-set negation, which is how
//! `except` and outer-join retraction are expressed — and [`Op::Fixpoint`]. Equivalence of
//! relational algebra **with difference** is undecidable (Trakhtenbrot's theorem, via
//! Abiteboul–Hull–Vianu §6.3). No amount of engineering effort produces a decision procedure
//! for it. The roadmap's own kill criterion — "if it needs a general theorem prover, narrow
//! the language" — would therefore have fired on the first day, because the fragment was never
//! fixed.
//!
//! Narrowing does not rescue the original shape either. Even for conjunctive queries,
//! equivalence is NP-complete (Chandra–Merlin 1977) — fine for a checker, since queries are
//! small, but worth stating. And Z-sets carry weights, so the relevant semantics is *bag*
//! rather than set: bag-equivalence of conjunctive queries is graph-isomorphism-hard, and bag
//! *containment* is a long-standing open problem. A "verifier" that answered "maybe" would
//! have rebuilt the query hint it was meant to replace.
//!
//! # The shape that does work
//!
//! Turn the problem around. Rather than proposing a plan and asking whether it is equivalent,
//! a schedule **names the rewrites that produced it**, drawn from a finite catalogue whose
//! members are individually proven equivalence-preserving under a stated side condition. The
//! checker then verifies each side condition — syntactically, in the IR — and applies the
//! rewrite itself.
//!
//! Equivalence is then true **by construction**, and the checker is a checker rather than a
//! prover: it never answers "maybe", because [`check`] returns
//! `Result<Circuit, ScheduleError>` and no variant of the error means "unknown". A step that
//! is not in the catalogue is [`ScheduleError::NotInCatalogue`], which is a refusal, not an
//! uncertainty.
//!
//! # What this is and is not a contribution
//!
//! It is not a decision procedure for plan equivalence, and this module does not pretend to
//! be one. What it is: **query hints become checked rewrites.** PostgreSQL has refused hints
//! for twenty-five years, and its objections reduce to the observation that a hint is an
//! unverified assertion the optimizer must trust. A catalogue step is not an assertion — it is
//! an operation whose precondition is checked before it is performed, so a wrong schedule is
//! rejected rather than silently obeyed.
//!
//! Nilestream is unusually well placed for it because the IR verifier is already in the
//! trusted base: this needed a new judgement, not a new subsystem, and — deliberately — no SMT
//! solver and no e-graph. A checker small enough to read is a checker somebody can audit.
//!
//! # The catalogue
//!
//! | Step | Side condition | Why it preserves denotation |
//! |---|---|---|
//! | [`Step::CommuteJoin`] | inner join, no residual | `A ⋈ B = B ⋈ A` up to column order, which the compensating [`Op::Map`] restores exactly |
//! | [`Step::PushFilterIntoLeft`] | inner join, predicate reads only left columns | selection distributes over inner join on the side it mentions |
//! | [`Step::PushFilterIntoRight`] | as above, right side | as above |
//! | [`Step::CommuteUnion`] | exactly two inputs | Z-set union is addition in an abelian group |
//! | [`Step::ElideDoubleNegate`] | `Negate` whose only input is a `Negate` | negation is an involution: `-(-z) = z` |
//!
//! Every one of those is a theorem about Z-sets, not a heuristic. Rules whose soundness needs
//! a *semantic* condition — projection pushdown needs functional-dependency inference, and
//! pushing a filter into an outer join's null-extended side is simply false — are deliberately
//! absent. Adding one means proving it and writing its side condition syntactically, which is
//! the bar this design exists to impose.

use crate::circuit::{Circuit, NodeId};
use crate::operator::{ColIdx, JoinKind, Op, Scalar};

/// One rewrite from the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Swap a join's inputs, restoring column order with a compensating projection.
    CommuteJoin { join: NodeId },
    /// Move a filter below a join, into its left input.
    PushFilterIntoLeft { filter: NodeId },
    /// Move a filter below a join, into its right input.
    PushFilterIntoRight { filter: NodeId },
    /// Swap a two-input union's inputs.
    CommuteUnion { union: NodeId },
    /// Remove a `Negate` directly above another `Negate`.
    ElideDoubleNegate { negate: NodeId },
}

impl Step {
    pub fn name(&self) -> &'static str {
        match self {
            Step::CommuteJoin { .. } => "commute-join",
            Step::PushFilterIntoLeft { .. } => "push-filter-into-left",
            Step::PushFilterIntoRight { .. } => "push-filter-into-right",
            Step::CommuteUnion { .. } => "commute-union",
            Step::ElideDoubleNegate { .. } => "elide-double-negate",
        }
    }

    /// The node this step names.
    pub fn target(&self) -> NodeId {
        match self {
            Step::CommuteJoin { join } => *join,
            Step::PushFilterIntoLeft { filter } | Step::PushFilterIntoRight { filter } => *filter,
            Step::CommuteUnion { union } => *union,
            Step::ElideDoubleNegate { negate } => *negate,
        }
    }
}

/// Why a schedule was refused.
///
/// Note what is absent: there is no `Unknown`, no `Maybe`, no `Unverified`. A checker that
/// could answer "I could not tell" would have rebuilt the unverified query hint this design
/// exists to replace, so the type makes that unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    /// The step names a node that does not exist.
    NoSuchNode { step: &'static str, node: NodeId },
    /// The step names a node of the wrong kind — `commute-join` on an aggregate, say.
    WrongOperator {
        step: &'static str,
        node: NodeId,
        found: &'static str,
        wanted: &'static str,
    },
    /// The operator is right but its side condition does not hold.
    ///
    /// The interesting refusal, and the one that carries the argument: it names the condition
    /// rather than reporting that something was wrong.
    SideCondition {
        step: &'static str,
        node: NodeId,
        condition: String,
    },
    /// A step outside the catalogue.
    ///
    /// Cannot arise from a [`Step`] value — the enum is the catalogue — and exists for a
    /// front end parsing a schedule from text, where a user may name a rewrite that has not
    /// been proven.
    NotInCatalogue { named: String },
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduleError::NoSuchNode { step, node } => {
                write!(f, "`{step}` names node {node}, which does not exist")
            }
            ScheduleError::WrongOperator {
                step,
                node,
                found,
                wanted,
            } => write!(
                f,
                "`{step}` applies to a {wanted}; node {node} is a {found}"
            ),
            ScheduleError::SideCondition {
                step,
                node,
                condition,
            } => write!(
                f,
                "`{step}` cannot be applied at node {node}: {condition}. The rewrite is only \
                 equivalence-preserving when that holds, so it is refused rather than performed"
            ),
            ScheduleError::NotInCatalogue { named } => write!(
                f,
                "`{named}` is not in the rewrite catalogue. A schedule may only name rewrites \
                 that have been proven equivalence-preserving; an unproven one would be a hint, \
                 which is what this replaces"
            ),
        }
    }
}

/// A schedule: the rewrites, in the order they are to be applied.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schedule {
    pub steps: Vec<Step>,
}

impl Schedule {
    pub fn new(steps: Vec<Step>) -> Schedule {
        Schedule { steps }
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }
}

/// Check and apply a whole schedule.
///
/// Returns the rewritten circuit, or the first refusal. **A refusal leaves nothing behind**:
/// the circuit is rewritten into a clone, so a schedule that fails at step seven does not
/// produce a plan that had six rewrites applied to it.
pub fn check(circuit: &Circuit, schedule: &Schedule) -> Result<Circuit, ScheduleError> {
    let mut c = circuit.clone();
    for step in &schedule.steps {
        c = apply(&c, step)?;
    }
    Ok(c)
}

/// Check and apply one step.
pub fn apply(circuit: &Circuit, step: &Step) -> Result<Circuit, ScheduleError> {
    match step {
        Step::CommuteJoin { join } => commute_join(circuit, *join),
        Step::PushFilterIntoLeft { filter } => push_filter(circuit, *filter, true),
        Step::PushFilterIntoRight { filter } => push_filter(circuit, *filter, false),
        Step::CommuteUnion { union } => commute_union(circuit, *union),
        Step::ElideDoubleNegate { negate } => elide_double_negate(circuit, *negate),
    }
}

fn node_at(circuit: &Circuit, id: NodeId, step: &'static str) -> Result<usize, ScheduleError> {
    circuit
        .nodes
        .iter()
        .position(|n| n.id == id)
        .ok_or(ScheduleError::NoSuchNode { step, node: id })
}

/// Every column index a scalar expression reads.
///
/// The basis of the filter-pushdown side condition, and deliberately **syntactic**: it walks
/// the expression and collects `Column` indices. A semantic condition — "this predicate does
/// not depend on the right side after simplification" — would need a solver, and a solver is
/// what this design refuses.
pub fn columns_read(s: &Scalar, out: &mut Vec<ColIdx>) {
    match s {
        Scalar::Column(c) => out.push(*c),
        Scalar::Not(inner) | Scalar::Neg(inner) | Scalar::IsNull(inner) => columns_read(inner, out),
        Scalar::Binary { lhs, rhs, .. } => {
            columns_read(lhs, out);
            columns_read(rhs, out);
        }
        Scalar::Udf { args, .. } => {
            for a in args {
                columns_read(a, out);
            }
        }
        Scalar::LitInt(_)
        | Scalar::LitBool(_)
        | Scalar::LitText(_)
        | Scalar::LitMoney { .. }
        | Scalar::LitNull
        | Scalar::Anchor => {}
    }
}

/// Whether an expression mentions a UDF.
///
/// A UDF's determinism is certified at load rather than here, and pushing one below a join
/// changes **how many times it is called** — which is observable for anything that is not a
/// pure function, and is the reason this is a side condition rather than a footnote.
fn mentions_udf(s: &Scalar) -> bool {
    match s {
        Scalar::Udf { .. } => true,
        Scalar::Not(inner) | Scalar::Neg(inner) => mentions_udf(inner),
        Scalar::Binary { lhs, rhs, .. } => mentions_udf(lhs) || mentions_udf(rhs),
        _ => false,
    }
}

fn commute_join(circuit: &Circuit, id: NodeId) -> Result<Circuit, ScheduleError> {
    const STEP: &str = "commute-join";
    let i = node_at(circuit, id, STEP)?;
    let (kind, lk, rk, residual) = match &circuit.nodes[i].op {
        Op::Join {
            kind,
            left_key,
            right_key,
            residual,
        } => (*kind, left_key.clone(), right_key.clone(), residual.clone()),
        other => {
            return Err(ScheduleError::WrongOperator {
                step: STEP,
                node: id,
                found: other.name(),
                wanted: "join",
            })
        }
    };

    // **Inner only.** `A ⟕ B` is not `B ⟕ A`: an outer join preserves rows from one named
    // side, and swapping changes which. Semi and anti joins are asymmetric by definition.
    if kind != JoinKind::Inner {
        return Err(ScheduleError::SideCondition {
            step: STEP,
            node: id,
            condition: format!(
                "the join is {kind:?}, and only an inner join commutes — an outer join \
                 preserves rows from a named side, and swapping changes which side that is"
            ),
        });
    }
    // A residual predicate is written against the *combined* column layout, so swapping the
    // inputs would silently reindex it. Remapping is possible; refusing is honest and is what
    // a syntactic side condition looks like.
    if residual.is_some() {
        return Err(ScheduleError::SideCondition {
            step: STEP,
            node: id,
            condition: "the join carries a residual predicate written against the combined \
                        column layout, which swapping would reindex"
                .to_string(),
        });
    }
    if circuit.nodes[i].inputs.len() != 2 {
        return Err(ScheduleError::SideCondition {
            step: STEP,
            node: id,
            condition: format!(
                "a join has two inputs; this one has {}",
                circuit.nodes[i].inputs.len()
            ),
        });
    }

    let left = circuit.nodes[i].inputs[0];
    let right = circuit.nodes[i].inputs[1];
    let left_arity = arity_of(circuit, left);
    let right_arity = arity_of(circuit, right);

    let mut out = circuit.clone();
    out.nodes[i].inputs = vec![right, left];
    out.nodes[i].op = Op::Join {
        kind,
        left_key: rk,
        right_key: lk,
        residual: None,
    };

    // **The compensating projection.** Swapping puts the right input's columns first, which is
    // a different output schema — so the rewrite would be equivalence-preserving only "up to
    // column order", and a checker that accepted that would be accepting something weaker than
    // it claimed. Restoring the order exactly is what makes the equality hold on the nose.
    let restore: Vec<Scalar> = (0..left_arity)
        .map(|c| Scalar::Column(right_arity + c))
        .chain((0..right_arity).map(Scalar::Column))
        .collect();
    let compensator = out.nodes[i].clone_shell(
        next_id(&out),
        Op::Map { exprs: restore },
        vec![id],
        left_arity + right_arity,
        format!("restore column order after commuting {id}"),
    );
    let new_id = compensator.id;
    out.nodes.push(compensator);
    rewire_consumers(&mut out, id, new_id);
    Ok(out)
}

fn push_filter(circuit: &Circuit, id: NodeId, into_left: bool) -> Result<Circuit, ScheduleError> {
    let step: &'static str = if into_left {
        "push-filter-into-left"
    } else {
        "push-filter-into-right"
    };
    let i = node_at(circuit, id, step)?;
    let predicate = match &circuit.nodes[i].op {
        Op::Filter { predicate } => predicate.clone(),
        other => {
            return Err(ScheduleError::WrongOperator {
                step,
                node: id,
                found: other.name(),
                wanted: "filter",
            })
        }
    };
    if circuit.nodes[i].inputs.len() != 1 {
        return Err(ScheduleError::SideCondition {
            step,
            node: id,
            condition: "a filter has exactly one input".to_string(),
        });
    }
    let join_id = circuit.nodes[i].inputs[0];
    let j = node_at(circuit, join_id, step)?;
    let kind = match &circuit.nodes[j].op {
        Op::Join { kind, .. } => *kind,
        other => {
            return Err(ScheduleError::SideCondition {
                step,
                node: id,
                condition: format!("the filter's input is a {}, not a join", other.name()),
            })
        }
    };
    // **Inner only, and this one is not a technicality.** Pushing a predicate into the
    // null-extended side of an outer join changes the answer: rows that would have been
    // preserved with NULLs are removed before the join can preserve them. It is the classic
    // wrong rewrite, and it is refused by name.
    if kind != JoinKind::Inner {
        return Err(ScheduleError::SideCondition {
            step,
            node: id,
            condition: format!(
                "the join is {kind:?}. Pushing a predicate into the null-extended side of an \
                 outer join removes rows the join would have preserved"
            ),
        });
    }
    if mentions_udf(&predicate) {
        return Err(ScheduleError::SideCondition {
            step,
            node: id,
            condition: "the predicate calls a user function, and pushing it below the join \
                        changes how many times that function is called"
                .to_string(),
        });
    }

    let left = circuit.nodes[j].inputs[0];
    let right = circuit.nodes[j].inputs[1];
    let left_arity = arity_of(circuit, left);

    let mut read = Vec::new();
    columns_read(&predicate, &mut read);

    // The side condition proper: the predicate must mention only the side it is pushed into.
    // Columns are positional, so "the left side" is `0..left_arity` and the right side is
    // everything above — and pushing right also **reindexes** the predicate, since the right
    // input's own column 0 is the combined layout's `left_arity`.
    let (target, rewritten) = if into_left {
        if let Some(bad) = read.iter().find(|c| **c >= left_arity) {
            return Err(ScheduleError::SideCondition {
                step,
                node: id,
                condition: format!(
                    "the predicate reads column {bad}, which belongs to the right input \
                     (columns 0..{left_arity} are the left)"
                ),
            });
        }
        (left, predicate.clone())
    } else {
        if let Some(bad) = read.iter().find(|c| **c < left_arity) {
            return Err(ScheduleError::SideCondition {
                step,
                node: id,
                condition: format!(
                    "the predicate reads column {bad}, which belongs to the left input \
                     (columns {left_arity}.. are the right)"
                ),
            });
        }
        (right, shift_columns(&predicate, -(left_arity as i64)))
    };

    let mut out = circuit.clone();
    // Insert the filter above `target`, and remove the original above the join.
    let pushed = out.nodes[i].clone_shell(
        next_id(&out),
        Op::Filter {
            predicate: rewritten,
        },
        vec![target],
        arity_of(circuit, target),
        format!("pushed below join {join_id}"),
    );
    let pushed_id = pushed.id;
    out.nodes.push(pushed);
    // The join now reads the filtered side.
    let jj = node_at(&out, join_id, step)?;
    if into_left {
        out.nodes[jj].inputs[0] = pushed_id;
    } else {
        out.nodes[jj].inputs[1] = pushed_id;
    }
    // And the original filter is elided: its consumers read the join directly.
    rewire_consumers(&mut out, id, join_id);
    out.nodes.retain(|n| n.id != id);
    Ok(out)
}

fn commute_union(circuit: &Circuit, id: NodeId) -> Result<Circuit, ScheduleError> {
    const STEP: &str = "commute-union";
    let i = node_at(circuit, id, STEP)?;
    match &circuit.nodes[i].op {
        Op::Union => {}
        other => {
            return Err(ScheduleError::WrongOperator {
                step: STEP,
                node: id,
                found: other.name(),
                wanted: "union",
            })
        }
    }
    if circuit.nodes[i].inputs.len() != 2 {
        return Err(ScheduleError::SideCondition {
            step: STEP,
            node: id,
            condition: format!(
                "this step swaps two inputs; the node has {}",
                circuit.nodes[i].inputs.len()
            ),
        });
    }
    // Z-set union is addition in an abelian group, so this is commutative on the nose — no
    // compensating projection, because both inputs share the output schema.
    let mut out = circuit.clone();
    out.nodes[i].inputs.swap(0, 1);
    Ok(out)
}

fn elide_double_negate(circuit: &Circuit, id: NodeId) -> Result<Circuit, ScheduleError> {
    const STEP: &str = "elide-double-negate";
    let i = node_at(circuit, id, STEP)?;
    match &circuit.nodes[i].op {
        Op::Negate => {}
        other => {
            return Err(ScheduleError::WrongOperator {
                step: STEP,
                node: id,
                found: other.name(),
                wanted: "negate",
            })
        }
    }
    if circuit.nodes[i].inputs.len() != 1 {
        return Err(ScheduleError::SideCondition {
            step: STEP,
            node: id,
            condition: "a negate has exactly one input".to_string(),
        });
    }
    let inner_id = circuit.nodes[i].inputs[0];
    let k = node_at(circuit, inner_id, STEP)?;
    if !matches!(circuit.nodes[k].op, Op::Negate) {
        return Err(ScheduleError::SideCondition {
            step: STEP,
            node: id,
            condition: format!(
                "the input is a {}, not a second negate; a single negation is not the identity",
                circuit.nodes[k].op.name()
            ),
        });
    }
    if circuit.nodes[k].inputs.len() != 1 {
        return Err(ScheduleError::SideCondition {
            step: STEP,
            node: id,
            condition: "the inner negate has no single input".to_string(),
        });
    }
    // `-(-z) = z`: negation is an involution in a Z-set, so both nodes disappear and their
    // consumers read what the inner negate read.
    let grandparent = circuit.nodes[k].inputs[0];
    let mut out = circuit.clone();
    rewire_consumers(&mut out, id, grandparent);
    // The inner negate stays only if something else reads it.
    let inner_still_used = out
        .nodes
        .iter()
        .any(|n| n.id != id && n.inputs.contains(&inner_id));
    out.nodes
        .retain(|n| n.id != id && (inner_still_used || n.id != inner_id));
    for (name, target) in out.outputs.iter_mut() {
        let _ = name;
        if *target == id {
            *target = grandparent;
        }
    }
    Ok(out)
}

fn arity_of(circuit: &Circuit, id: NodeId) -> ColIdx {
    circuit
        .nodes
        .iter()
        .find(|n| n.id == id)
        .map(|n| n.arity)
        .unwrap_or(0)
}

fn next_id(circuit: &Circuit) -> NodeId {
    circuit
        .nodes
        .iter()
        .map(|n| n.id)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0)
}

fn rewire_consumers(circuit: &mut Circuit, from: NodeId, to: NodeId) {
    for n in circuit.nodes.iter_mut() {
        if n.id == to {
            continue;
        }
        for input in n.inputs.iter_mut() {
            if *input == from {
                *input = to;
            }
        }
    }
    for (_, target) in circuit.outputs.iter_mut() {
        if *target == from {
            *target = to;
        }
    }
}

/// Shift every column reference in an expression by a signed offset.
///
/// Needed by right-side filter pushdown: the right input's own column 0 is the combined
/// layout's `left_arity`, so a predicate that read column 5 of the join must read column
/// `5 - left_arity` once it sits below it. Doing this wrong produces a plan that runs and
/// answers a different question, which is the failure mode a checker exists to make
/// impossible.
fn shift_columns(s: &Scalar, by: i64) -> Scalar {
    match s {
        Scalar::Column(c) => Scalar::Column(((*c as i64) + by).max(0) as ColIdx),
        Scalar::Not(inner) => Scalar::Not(Box::new(shift_columns(inner, by))),
        Scalar::Neg(inner) => Scalar::Neg(Box::new(shift_columns(inner, by))),
        Scalar::Binary { op, lhs, rhs } => Scalar::Binary {
            op: *op,
            lhs: Box::new(shift_columns(lhs, by)),
            rhs: Box::new(shift_columns(rhs, by)),
        },
        Scalar::Udf { id, args } => Scalar::Udf {
            id: *id,
            args: args.iter().map(|a| shift_columns(a, by)).collect(),
        },
        other => other.clone(),
    }
}

/// Parse a schedule step named in text, refusing anything outside the catalogue.
///
/// The surface a user touches. `NotInCatalogue` is the whole point: naming a rewrite that has
/// not been proven is refused here, rather than being trusted the way a query hint is.
pub fn step_from_name(name: &str, node: NodeId) -> Result<Step, ScheduleError> {
    match name {
        "commute-join" => Ok(Step::CommuteJoin { join: node }),
        "push-filter-into-left" => Ok(Step::PushFilterIntoLeft { filter: node }),
        "push-filter-into-right" => Ok(Step::PushFilterIntoRight { filter: node }),
        "commute-union" => Ok(Step::CommuteUnion { union: node }),
        "elide-double-negate" => Ok(Step::ElideDoubleNegate { negate: node }),
        other => Err(ScheduleError::NotInCatalogue {
            named: other.to_string(),
        }),
    }
}

/// Every rewrite the catalogue contains, for `explain` and for documentation that cannot
/// drift from the code.
pub fn catalogue() -> Vec<&'static str> {
    vec![
        "commute-join",
        "push-filter-into-left",
        "push-filter-into-right",
        "commute-union",
        "elide-double-negate",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Anchor, Checked, Node};
    use crate::operator::ScalarOp;
    use crate::{Consistency, Lineage, Materialize, Retention, ServeContract};
    use std::collections::BTreeMap;

    // ── the reference evaluator ─────────────────────────────────────────────────────
    //
    // The catalogue's entries are stated as theorems about Z-sets, and a checker that
    // applied them without ever *evaluating* anything would be trusting its own
    // documentation. So each is checked denotationally, against `crate::eval`.
    //
    // That evaluator used to live here, as a private copy. It moved out when the unnesting
    // corpus needed the same semantics, because two copies of a semantics is two
    // semantics: a rewrite could then be "correct" under the copy that its author also
    // wrote. One implementation, used by both corpora, is the point.

    use crate::eval::{add, run_node as eval_at, Row, ZSet};

    // ── building circuits ───────────────────────────────────────────────────────────

    fn node(id: NodeId, op: Op, inputs: Vec<NodeId>, arity: u16) -> Node {
        Node {
            id,
            op,
            inputs,
            key: None,
            arity,
            anchor: Checked::new("anchor", Anchor::Inherited),
            contract: Checked::new(
                "contract",
                ServeContract {
                    consistency: Consistency::Snapshot,
                    materialize: Materialize::Demand,
                    retain: Retention::Evictable,
                    lineage: Lineage::Off,
                },
            ),
            conservation_transparent: Checked::new("conservation_transparent", true),
            lineage: Checked::new("lineage", Lineage::Off),
            label: String::new(),
        }
    }

    fn source(id: NodeId, name: &str, arity: u16) -> Node {
        node(
            id,
            Op::Source {
                relation: name.into(),
                is_base: true,
                anchor_key: vec![0],
                confidential: Vec::new(),
            },
            vec![],
            arity,
        )
    }

    /// `filter(p) over (L join R on L.0 = R.0)` — the shape most of the catalogue touches.
    fn join_circuit(kind: JoinKind, predicate: Scalar, residual: Option<Scalar>) -> Circuit {
        let mut c = Circuit::default();
        c.nodes.push(source(0, "l", 2));
        c.nodes.push(source(1, "r", 2));
        c.nodes.push(node(
            2,
            Op::Join {
                kind,
                left_key: vec![0],
                right_key: vec![0],
                residual,
            },
            vec![0, 1],
            4,
        ));
        c.nodes.push(node(3, Op::Filter { predicate }, vec![2], 4));
        c.outputs.insert("out".into(), 3);
        c
    }

    fn cells(vs: &[i128]) -> Row {
        vs.iter().map(|v| crate::value::Value::Int(*v)).collect()
    }

    /// Deterministic inputs including **negative weights**, because a rewrite can be sound on
    /// sets and unsound on Z-sets.
    fn inputs(seed: u64) -> BTreeMap<String, ZSet> {
        let mut rng = seed;
        let mut next = || {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((rng >> 33) % 7) as i128
        };
        let mut l = ZSet::new();
        let mut r = ZSet::new();
        for _ in 0..8 {
            let w = next() - 3; // −3..=3, so retractions occur
            if w != 0 {
                add(&mut l, cells(&[next(), next()]), w);
            }
            let w = next() - 3;
            if w != 0 {
                add(&mut r, cells(&[next(), next()]), w);
            }
        }
        BTreeMap::from([("l".to_string(), l), ("r".to_string(), r)])
    }

    fn output_of(c: &Circuit) -> NodeId {
        *c.outputs.get("out").expect("named output")
    }

    /// The property every catalogue entry claims: same inputs, same answer.
    fn denotation_holds(before: &Circuit, after: &Circuit, trials: u64) {
        for seed in 0..trials {
            let src = inputs(seed);
            let a = eval_at(before, output_of(before), &src).0;
            let b = eval_at(after, output_of(after), &src).0;
            assert_eq!(a, b, "the rewrite changed the answer on seed {seed}");
        }
    }

    // ── the catalogue, one positive and one negative test each ──────────────────────

    #[test]
    fn commuting_an_inner_join_preserves_the_denotation_over_a_thousand_z_sets() {
        let before = join_circuit(JoinKind::Inner, Scalar::LitBool(true), None);
        let after = apply(&before, &Step::CommuteJoin { join: 2 }).expect("commutes");
        denotation_holds(&before, &after, 1_000);
    }

    #[test]
    fn commuting_a_join_without_the_compensating_projection_would_have_been_wrong() {
        // The rewrite is only equivalence-preserving *up to column order*, and a checker that
        // accepted that would be accepting something weaker than it claims. This asserts the
        // compensator is really there and really necessary.
        let before = join_circuit(JoinKind::Inner, Scalar::LitBool(true), None);
        let after = apply(&before, &Step::CommuteJoin { join: 2 }).unwrap();
        assert!(
            after.nodes.iter().any(|n| matches!(n.op, Op::Map { .. })),
            "no compensating projection was inserted"
        );

        // Without it — swap the inputs by hand and evaluate — the answers differ.
        let mut naive = before.clone();
        naive.nodes[2].inputs.swap(0, 1);
        let src = inputs(1);
        assert_ne!(
            eval_at(&before, output_of(&before), &src).0,
            eval_at(&naive, output_of(&naive), &src).0,
            "swapping without compensating happened to agree, so this test proves nothing"
        );
    }

    #[test]
    fn an_outer_join_does_not_commute_and_the_refusal_says_why() {
        for kind in [
            JoinKind::LeftOuter,
            JoinKind::RightOuter,
            JoinKind::Anti,
            JoinKind::Semi,
        ] {
            let c = join_circuit(kind, Scalar::LitBool(true), None);
            let e = apply(&c, &Step::CommuteJoin { join: 2 }).unwrap_err();
            match e {
                ScheduleError::SideCondition { condition, .. } => {
                    assert!(condition.contains("inner join"), "{condition}");
                }
                other => panic!("expected a side-condition refusal for {kind:?}, got {other}"),
            }
        }
    }

    #[test]
    fn a_join_with_a_residual_is_refused_rather_than_reindexed() {
        let residual = Scalar::Binary {
            op: ScalarOp::Gt,
            lhs: Box::new(Scalar::Column(1)),
            rhs: Box::new(Scalar::Column(3)),
        };
        let c = join_circuit(JoinKind::Inner, Scalar::LitBool(true), Some(residual));
        let e = apply(&c, &Step::CommuteJoin { join: 2 }).unwrap_err();
        assert!(e.to_string().contains("residual"), "{e}");
    }

    #[test]
    fn pushing_a_left_only_filter_into_the_left_input_preserves_the_denotation() {
        // `l.1 > 2` mentions only left columns, so it distributes over the inner join.
        let p = Scalar::Binary {
            op: ScalarOp::Gt,
            lhs: Box::new(Scalar::Column(1)),
            rhs: Box::new(Scalar::LitInt(2)),
        };
        let before = join_circuit(JoinKind::Inner, p, None);
        let after = apply(&before, &Step::PushFilterIntoLeft { filter: 3 }).expect("pushes");
        denotation_holds(&before, &after, 1_000);
        assert!(
            after.nodes.iter().all(|n| n.id != 3),
            "the original filter was elided"
        );
    }

    #[test]
    fn pushing_a_right_only_filter_reindexes_it_and_preserves_the_denotation() {
        // `r.1 > 2` is column 3 of the combined layout and column 1 of the right input. The
        // reindexing is where this rewrite is most easily got wrong: a plan that forgot it
        // would run and answer a different question.
        let p = Scalar::Binary {
            op: ScalarOp::Gt,
            lhs: Box::new(Scalar::Column(3)),
            rhs: Box::new(Scalar::LitInt(2)),
        };
        let before = join_circuit(JoinKind::Inner, p, None);
        let after = apply(&before, &Step::PushFilterIntoRight { filter: 3 }).expect("pushes");
        denotation_holds(&before, &after, 1_000);

        let pushed = after
            .nodes
            .iter()
            .find(|n| matches!(&n.op, Op::Filter { .. }))
            .expect("a filter survives");
        match &pushed.op {
            Op::Filter { predicate } => {
                let mut cols = Vec::new();
                columns_read(predicate, &mut cols);
                assert_eq!(cols, vec![1], "column 3 became column 1 below the join");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn a_predicate_spanning_both_sides_is_refused_in_both_directions() {
        // The negative control the roadmap names. `l.1 = r.1` cannot be pushed into either
        // side, and a checker that allowed it would produce a plan computing something else.
        let p = Scalar::Binary {
            op: ScalarOp::Eq,
            lhs: Box::new(Scalar::Column(1)),
            rhs: Box::new(Scalar::Column(3)),
        };
        let c = join_circuit(JoinKind::Inner, p, None);
        for step in [
            Step::PushFilterIntoLeft { filter: 3 },
            Step::PushFilterIntoRight { filter: 3 },
        ] {
            let e = apply(&c, &step).unwrap_err();
            match e {
                ScheduleError::SideCondition { condition, .. } => {
                    assert!(condition.contains("belongs to the"), "{condition}");
                }
                other => panic!("expected a side-condition refusal, got {other}"),
            }
        }
    }

    #[test]
    fn pushing_a_filter_into_an_outer_join_is_refused_because_it_removes_preserved_rows() {
        let p = Scalar::Binary {
            op: ScalarOp::Gt,
            lhs: Box::new(Scalar::Column(1)),
            rhs: Box::new(Scalar::LitInt(2)),
        };
        let c = join_circuit(JoinKind::LeftOuter, p, None);
        let e = apply(&c, &Step::PushFilterIntoLeft { filter: 3 }).unwrap_err();
        assert!(e.to_string().contains("null-extended"), "{e}");
    }

    #[test]
    fn a_predicate_calling_a_udf_is_not_pushed_because_the_call_count_would_change() {
        let p = Scalar::Udf {
            id: 7,
            args: vec![Scalar::Column(1)],
        };
        let c = join_circuit(JoinKind::Inner, p, None);
        let e = apply(&c, &Step::PushFilterIntoLeft { filter: 3 }).unwrap_err();
        assert!(e.to_string().contains("user function"), "{e}");
    }

    #[test]
    fn union_commutes_and_needs_no_compensating_projection() {
        let mut before = Circuit::default();
        before.nodes.push(source(0, "l", 2));
        before.nodes.push(source(1, "r", 2));
        before.nodes.push(node(2, Op::Union, vec![0, 1], 2));
        before.outputs.insert("out".into(), 2);

        let after = apply(&before, &Step::CommuteUnion { union: 2 }).expect("commutes");
        assert_eq!(after.nodes[2].inputs, vec![1, 0]);
        assert_eq!(
            after.nodes.len(),
            before.nodes.len(),
            "both inputs share a schema, so nothing needs restoring"
        );
        denotation_holds(&before, &after, 1_000);
    }

    #[test]
    fn a_double_negation_is_the_identity_and_a_single_one_is_not() {
        let mut before = Circuit::default();
        before.nodes.push(source(0, "l", 2));
        before.nodes.push(node(1, Op::Negate, vec![0], 2));
        before.nodes.push(node(2, Op::Negate, vec![1], 2));
        before.outputs.insert("out".into(), 2);

        let after = apply(&before, &Step::ElideDoubleNegate { negate: 2 }).expect("elides");
        denotation_holds(&before, &after, 500);
        assert!(
            after.nodes.iter().all(|n| !matches!(n.op, Op::Negate)),
            "both negations should be gone"
        );

        // And a single negation is refused, because it is not the identity.
        let mut single = Circuit::default();
        single.nodes.push(source(0, "l", 2));
        single.nodes.push(node(1, Op::Negate, vec![0], 2));
        single.outputs.insert("out".into(), 1);
        let e = apply(&single, &Step::ElideDoubleNegate { negate: 1 }).unwrap_err();
        assert!(
            e.to_string()
                .contains("single negation is not the identity"),
            "{e}"
        );
    }

    // ── the checker's own properties ────────────────────────────────────────────────

    #[test]
    fn a_schedule_that_is_merely_slower_is_accepted() {
        // The roadmap's gate, in both directions: a schedule that changes the result must be
        // rejected, and one that only changes the plan must not be. Commuting a join twice
        // returns to the original denotation while leaving two extra projections behind — a
        // strictly worse plan, and a legal one.
        let before = join_circuit(JoinKind::Inner, Scalar::LitBool(true), None);
        let once = apply(&before, &Step::CommuteJoin { join: 2 }).unwrap();
        let twice = apply(&once, &Step::CommuteJoin { join: 2 }).expect("still legal");
        denotation_holds(&before, &twice, 500);
        assert!(
            twice.nodes.len() > before.nodes.len(),
            "and it really is a worse plan"
        );
    }

    #[test]
    fn the_checker_can_never_answer_maybe() {
        // Stated as a test because it is a property of the *type*: `check` returns
        // `Result<Circuit, ScheduleError>`, and no variant of the error means "unknown". A
        // checker that could answer "maybe" would have rebuilt the unverified hint this
        // design replaces.
        let variants = [
            ScheduleError::NoSuchNode { step: "s", node: 0 },
            ScheduleError::WrongOperator {
                step: "s",
                node: 0,
                found: "a",
                wanted: "b",
            },
            ScheduleError::SideCondition {
                step: "s",
                node: 0,
                condition: "c".into(),
            },
            ScheduleError::NotInCatalogue { named: "n".into() },
        ];
        for v in &variants {
            let text = v.to_string().to_lowercase();
            assert!(!text.contains("maybe"), "{text}");
            assert!(!text.contains("unknown"), "{text}");
            assert!(!text.contains("could not determine"), "{text}");
        }
    }

    #[test]
    fn a_rewrite_outside_the_catalogue_is_refused_by_name() {
        let e = step_from_name("magic-reordering", 4).unwrap_err();
        match e {
            ScheduleError::NotInCatalogue { named } => assert_eq!(named, "magic-reordering"),
            other => panic!("{other}"),
        }
        assert!(step_from_name("commute-join", 4).is_ok());
        assert_eq!(catalogue().len(), 5);
        for name in catalogue() {
            assert!(
                step_from_name(name, 0).is_ok(),
                "{name} is named but not parseable"
            );
        }
    }

    #[test]
    fn a_step_naming_the_wrong_operator_is_refused_with_both_kinds() {
        let c = join_circuit(JoinKind::Inner, Scalar::LitBool(true), None);
        let e = apply(&c, &Step::CommuteUnion { union: 2 }).unwrap_err();
        match e {
            ScheduleError::WrongOperator { found, wanted, .. } => {
                assert_eq!(wanted, "union");
                assert_eq!(found, "join");
            }
            other => panic!("{other}"),
        }
    }

    #[test]
    fn a_step_naming_a_node_that_does_not_exist_is_refused() {
        let c = join_circuit(JoinKind::Inner, Scalar::LitBool(true), None);
        assert!(matches!(
            apply(&c, &Step::CommuteJoin { join: 99 }),
            Err(ScheduleError::NoSuchNode { .. })
        ));
    }

    #[test]
    fn a_schedule_that_fails_partway_leaves_the_original_untouched() {
        // Otherwise a schedule refused at step seven would hand back a plan with six rewrites
        // applied — a plan nobody asked for and nobody checked as a whole.
        let before = join_circuit(JoinKind::Inner, Scalar::LitBool(true), None);
        let schedule = Schedule::new(vec![
            Step::CommuteJoin { join: 2 },
            Step::CommuteUnion { union: 2 }, // wrong operator: refused
        ]);
        assert!(check(&before, &schedule).is_err());
        assert_eq!(before.nodes.len(), 4, "the input circuit is unchanged");
        assert!(before.nodes.iter().all(|n| !matches!(n.op, Op::Map { .. })));
    }

    #[test]
    fn an_empty_schedule_is_the_identity() {
        let before = join_circuit(JoinKind::Inner, Scalar::LitBool(true), None);
        let after = check(&before, &Schedule::default()).expect("legal");
        denotation_holds(&before, &after, 100);
        assert_eq!(after.nodes.len(), before.nodes.len());
    }

    #[test]
    fn the_reference_evaluator_takes_signed_weights_seriously() {
        // Guarding the guard: if the evaluator ignored negative weights, every denotation test
        // above would be checking set semantics and would pass for rewrites that are wrong on
        // Z-sets — which is the semantics this IR actually has.
        let mut z = crate::eval::zset(&[(&[1, 1], 2), (&[2, 2], -3)]);
        assert_eq!(z.get(&cells(&[2, 2])), Some(&-3));
        add(&mut z, cells(&[1, 1]), -2);
        assert_eq!(
            z.get(&cells(&[1, 1])),
            None,
            "a zero weight is not a member"
        );

        let src = inputs(3);
        assert!(
            src["l"].values().any(|w| *w < 0) || src["r"].values().any(|w| *w < 0),
            "the generator must produce retractions, or the tests are weaker than they look"
        );
    }
}
