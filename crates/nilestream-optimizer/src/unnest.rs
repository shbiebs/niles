//! **Subquery unnesting: turning a dependent join into a set-at-a-time one.**
//!
//! A correlated subquery has two circuits. The nested one holds an [`Op::Apply`] — a
//! dependent join, whose meaning is a loop: for each outer row, scan the inner relation.
//! The unnested one holds a semi-, anti- or left-outer join over the whole relation at
//! once. This module turns the first into the second.
//!
//! # Why this is not an optimisation
//!
//! The usual framing — "unnesting makes correlated queries faster" — undersells it here.
//! A dependent join has **no delta rule**. One new outer row re-scans the inner relation,
//! so the operator cannot be incrementally maintained, so a view containing one cannot be
//! served at any rung and cannot be partially materialized at all. `verify` refuses it
//! (IR018). Unnesting is therefore the step that makes a correlated query *expressible as
//! a view*, and the speedup is a consequence rather than the purpose.
//!
//! # The five rewrites
//!
//! | Nested | Unnested |
//! |---|---|
//! | `exists (…)` | semi-join on the correlation |
//! | `not exists (…)` | anti-join on the correlation |
//! | `x in (…)` | semi-join on the correlation **and** `x = inner` |
//! | `x not in (…)` | anti-join, twice — see below |
//! | scalar subquery | left outer join against a grouped aggregate |
//!
//! Four of the five are one node each. The fifth is the one that is usually wrong.
//!
//! # `not in`, and the null witness
//!
//! `x NOT IN (select y from r where r.k = o.k)` is three-valued, and the rule is not
//! "anti-join". Writing `T`, `F`, `U` for true, false and unknown:
//!
//! * if `x` is null → `U` → the row is dropped, whatever `r` holds;
//! * if any in-scope `y` equals `x` → `F` → dropped;
//! * else if any in-scope `y` is null → `U` → **dropped**, even though nothing matched;
//! * else → `T` → kept.
//!
//! The third line is the one that surprises people, and it is why a plain anti-join is
//! wrong: an anti-join keeps exactly the rows that did not match, including the ones whose
//! verdict is unknown. A single null in the subquery's column makes `not in` return *no
//! rows at all* for every non-matching probe.
//!
//! The rewrite therefore emits two anti-joins:
//!
//! ```text
//! Filter(x is not null)                        -- drops the null probes
//!   |> AntiJoin(r, on correlation ∪ {x = y})   -- drops the matches
//!   |> AntiJoin(NullWitness(r), on correlation) -- drops the unknowns
//! ```
//!
//! where `NullWitness(r) = Distinct(Map(correlation-cols)(Filter(y is null)(r)))` — one row
//! per correlation group that contains a null. The uncorrelated case falls out of the same
//! shape without a special case: with an empty correlation the second anti-join has an
//! empty key, every pair matches, and the result is "keep the rows iff the witness relation
//! is empty" — which is precisely the rule.
//!
//! # What is refused, and why refusal is named
//!
//! Non-equi correlation, a `limit` or `order_by` below the apply, and an uncertified UDF in
//! the subquery are all refused with a [`Refusal`] naming the reason. A rewrite that
//! silently declined would leave an `Apply` in the plan and the failure would surface much
//! later as IR018, pointing at the wrong thing.

use niles_ir::circuit::{Circuit, NodeId};
use niles_ir::operator::{Agg, ApplyKind, ColIdx, JoinKind, Op, Scalar, ScalarOp};

/// Which rewrite fired at which node, so the gate can be checked in the IR rather than by
/// eye.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rewrite {
    /// `exists` → semi-join.
    ExistsToSemiJoin { apply: NodeId },
    /// `not exists` → anti-join.
    NotExistsToAntiJoin { apply: NodeId },
    /// `in` → semi-join with the probe folded into the key.
    InToSemiJoin { apply: NodeId },
    /// `not in` → the two anti-joins and the null witness.
    NotInToAntiJoinWithNullWitness { apply: NodeId },
    /// A correlated scalar subquery → left outer join against a grouped aggregate.
    ScalarToOuterJoinWithAggregate { apply: NodeId },
}

impl Rewrite {
    pub fn name(&self) -> &'static str {
        match self {
            Rewrite::ExistsToSemiJoin { .. } => "exists-to-semi-join",
            Rewrite::NotExistsToAntiJoin { .. } => "not-exists-to-anti-join",
            Rewrite::InToSemiJoin { .. } => "in-to-semi-join",
            Rewrite::NotInToAntiJoinWithNullWitness { .. } => {
                "not-in-to-anti-join-with-null-witness"
            }
            Rewrite::ScalarToOuterJoinWithAggregate { .. } => "scalar-to-outer-join-with-aggregate",
        }
    }
    pub fn node(&self) -> NodeId {
        match self {
            Rewrite::ExistsToSemiJoin { apply }
            | Rewrite::NotExistsToAntiJoin { apply }
            | Rewrite::InToSemiJoin { apply }
            | Rewrite::NotInToAntiJoinWithNullWitness { apply }
            | Rewrite::ScalarToOuterJoinWithAggregate { apply } => *apply,
        }
    }
}

/// Why an apply was left nested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub apply: NodeId,
    pub reason: &'static str,
}

/// What `unnest` did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub fired: Vec<Rewrite>,
    pub refused: Vec<Refusal>,
}

impl Report {
    pub fn is_complete(&self) -> bool {
        self.refused.is_empty()
    }
    pub fn render(&self) -> String {
        let mut s = String::new();
        for f in &self.fired {
            s.push_str(&format!("  fired   {} at node {}\n", f.name(), f.node()));
        }
        for r in &self.refused {
            s.push_str(&format!("  refused node {}: {}\n", r.apply, r.reason));
        }
        s
    }
}

/// Rewrite every `Apply` in `c` into set-at-a-time joins, where the side conditions allow.
///
/// Returns a new circuit and a report. The input is not modified: a rewrite that mutated
/// in place would make "compare the answers before and after" an awkward thing to write,
/// and the whole gate rests on that comparison being easy.
pub fn unnest(c: &Circuit) -> (Circuit, Report) {
    let mut report = Report::default();
    let mut out = Circuit::new();
    out.certified_udfs = c.certified_udfs.clone();
    // Old node id → new node id. Every rewrite ends by mapping the apply's id to whatever
    // node now produces its output, so consumers need no special handling.
    let mut remap: Vec<NodeId> = vec![0; c.nodes.len()];

    for n in &c.nodes {
        let inputs: Vec<NodeId> = n.inputs.iter().map(|i| remap[*i as usize]).collect();
        let contract = *n.contract.peek();
        let new_id = match &n.op {
            Op::Apply { kind, correlation } => match refuse(c, n.id, kind, correlation) {
                Some(reason) => {
                    report.refused.push(Refusal {
                        apply: n.id,
                        reason,
                    });
                    out.add(n.op.clone(), inputs, contract, n.label.clone())
                }
                None => {
                    let outer_arity = c.nodes[n.inputs[0] as usize].arity;
                    let id = rewrite(
                        &mut out,
                        kind,
                        correlation,
                        inputs[0],
                        inputs[1],
                        outer_arity,
                        &contract,
                        &n.label,
                    );
                    report.fired.push(fired_for(kind, n.id));
                    id
                }
            },
            other => out.add(other.clone(), inputs, contract, n.label.clone()),
        };
        remap[n.id as usize] = new_id;
    }

    for (name, id) in &c.outputs {
        out.set_output(name.clone(), remap[*id as usize]);
    }
    (out, report)
}

fn fired_for(kind: &ApplyKind, apply: NodeId) -> Rewrite {
    match kind {
        ApplyKind::Exists => Rewrite::ExistsToSemiJoin { apply },
        ApplyKind::NotExists => Rewrite::NotExistsToAntiJoin { apply },
        ApplyKind::In { .. } => Rewrite::InToSemiJoin { apply },
        ApplyKind::NotIn { .. } => Rewrite::NotInToAntiJoinWithNullWitness { apply },
        ApplyKind::Scalar { .. } => Rewrite::ScalarToOuterJoinWithAggregate { apply },
    }
}

/// The side conditions, each with the reason it exists.
fn refuse(
    c: &Circuit,
    apply: NodeId,
    kind: &ApplyKind,
    correlation: &[(ColIdx, ColIdx)],
) -> Option<&'static str> {
    let n = c.node(apply);
    let inner = n.inputs.get(1).copied()?;

    // Unnesting moves the subquery's evaluation from once-per-outer-row to once. Under a
    // `limit` or an `order_by` those are different queries: "the three cheapest matching
    // rows *for this account*" is not "three rows of the join". Refuse rather than change
    // the answer.
    if let Some(bad) = find_below(c, inner, &|op| {
        matches!(op, Op::Limit { .. } | Op::OrderBy { .. })
    }) {
        let _ = bad;
        return Some(
            "the subquery is under a `limit` or `order_by`, where per-outer-row and \
                     set-at-a-time evaluation are different queries",
        );
    }

    // An uncertified UDF may not be deterministic, and unnesting changes how many times it
    // is called. For a certified one that does not matter, which is why the certification
    // list exists rather than a blanket refusal.
    if find_below(c, inner, &|op| match op {
        Op::Filter { predicate } => !predicate.is_reproducible(&c.certified_udfs),
        Op::Map { exprs } => exprs.iter().any(|e| !e.is_reproducible(&c.certified_udfs)),
        _ => false,
    })
    .is_some()
    {
        return Some(
            "the subquery calls a UDF the loader has not certified deterministic, and \
                     unnesting changes how many times it is called",
        );
    }

    // A scalar subquery must produce exactly one value per outer row. With no correlation
    // it produces one value for *all* of them, which is a legal query but a different
    // rewrite (a cross join against a one-row relation), and this module does not do it.
    if matches!(kind, ApplyKind::Scalar { .. }) && correlation.is_empty() {
        return Some(
            "an uncorrelated scalar subquery needs a cross join against a one-row \
                     relation, which is a different rewrite",
        );
    }
    None
}

/// Whether any operator at or below `id` satisfies `p`.
fn find_below(c: &Circuit, id: NodeId, p: &dyn Fn(&Op) -> bool) -> Option<NodeId> {
    let mut stack = vec![id];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(x) = stack.pop() {
        if !seen.insert(x) || x as usize >= c.nodes.len() {
            continue;
        }
        let n = &c.nodes[x as usize];
        if p(&n.op) {
            return Some(x);
        }
        stack.extend(n.inputs.iter().copied());
    }
    None
}

fn eq(a: Scalar, b: Scalar) -> Scalar {
    Scalar::Binary {
        op: ScalarOp::Eq,
        lhs: Box::new(a),
        rhs: Box::new(b),
    }
}

/// Emit the unnested form and return the node that now produces the apply's output.
#[allow(clippy::too_many_arguments)]
fn rewrite(
    out: &mut Circuit,
    kind: &ApplyKind,
    correlation: &[(ColIdx, ColIdx)],
    outer: NodeId,
    inner: NodeId,
    outer_arity: u16,
    contract: &niles_ir::ServeContract,
    label: &str,
) -> NodeId {
    let lk: Vec<ColIdx> = correlation.iter().map(|(a, _)| *a).collect();
    let rk: Vec<ColIdx> = correlation.iter().map(|(_, b)| *b).collect();

    match kind {
        ApplyKind::Exists => out.add(
            Op::Join {
                kind: JoinKind::Semi,
                left_key: lk,
                right_key: rk,
                residual: None,
            },
            vec![outer, inner],
            *contract,
            format!("{label} (exists → semi)"),
        ),
        ApplyKind::NotExists => out.add(
            Op::Join {
                kind: JoinKind::Anti,
                left_key: lk,
                right_key: rk,
                residual: None,
            },
            vec![outer, inner],
            *contract,
            format!("{label} (not exists → anti)"),
        ),
        ApplyKind::In { probe, inner: icol } => {
            // The probe joins the key. A null probe never equals anything — the reference
            // semantics makes key equality three-valued and discards unknown — so no
            // explicit filter is needed, and adding one would only hide where the rule lives.
            let mut lk = lk;
            let mut rk = rk;
            lk.push(*probe);
            rk.push(*icol);
            out.add(
                Op::Join {
                    kind: JoinKind::Semi,
                    left_key: lk,
                    right_key: rk,
                    residual: None,
                },
                vec![outer, inner],
                *contract,
                format!("{label} (in → semi)"),
            )
        }
        ApplyKind::NotIn { probe, inner: icol } => {
            // 1. Drop the null probes. `null not in (…)` is unknown even against an empty
            //    subquery, so this cannot be folded into the anti-join below: an anti-join
            //    would *keep* a null probe that matched nothing.
            let not_null = Scalar::Not(Box::new(Scalar::IsNull(Box::new(Scalar::Column(*probe)))));
            let probed = out.add(
                Op::Filter {
                    predicate: not_null,
                },
                vec![outer],
                *contract,
                format!("{label} (probe is not null)"),
            );

            // 2. Drop the matches.
            let mut alk = lk.clone();
            let mut ark = rk.clone();
            alk.push(*probe);
            ark.push(*icol);
            let unmatched = out.add(
                Op::Join {
                    kind: JoinKind::Anti,
                    left_key: alk,
                    right_key: ark,
                    residual: None,
                },
                vec![probed, inner],
                *contract,
                format!("{label} (not in → anti)"),
            );

            // 3. Drop the unknowns. The null witness: one row per correlation group that
            //    contains a null in the probed column. With an empty correlation the
            //    anti-join below has an empty key, so every pair matches and the whole
            //    left side is dropped iff the witness is non-empty — which is exactly the
            //    uncorrelated rule, without a special case for it.
            let nulls = out.add(
                Op::Filter {
                    predicate: Scalar::IsNull(Box::new(Scalar::Column(*icol))),
                },
                vec![inner],
                *contract,
                format!("{label} (null witness)"),
            );
            let witness_cols: Vec<Scalar> = if rk.is_empty() {
                // A one-column constant, so the projection is well formed and the group is
                // "the whole relation".
                vec![Scalar::LitInt(0)]
            } else {
                rk.iter().map(|c| Scalar::Column(*c)).collect()
            };
            let projected = out.add(
                Op::Map {
                    exprs: witness_cols,
                },
                vec![nulls],
                *contract,
                format!("{label} (witness key)"),
            );
            let witness = out.add(
                Op::Distinct,
                vec![projected],
                *contract,
                format!("{label} (witness distinct)"),
            );
            let witness_key: Vec<ColIdx> = (0..rk.len() as ColIdx).collect();
            out.add(
                Op::Join {
                    kind: JoinKind::Anti,
                    left_key: lk,
                    right_key: witness_key,
                    residual: None,
                },
                vec![unmatched, witness],
                *contract,
                format!("{label} (drop unknowns)"),
            )
        }
        ApplyKind::Scalar { agg, expr } => {
            // Group the subquery by its correlation columns once, then join. The join must
            // be a *left outer*: an outer row with no matching group takes null, which is
            // SQL's value for a scalar subquery over an empty set. An inner join would
            // silently drop those rows — the same class of defect as `Err(_) => 0`, one
            // level up.
            let grouped = out.add(
                Op::Aggregate {
                    group_key: rk.clone(),
                    aggs: vec![(*agg, expr.clone())],
                },
                vec![inner],
                *contract,
                format!("{label} (subquery aggregate)"),
            );
            let joined = out.add(
                Op::Join {
                    kind: JoinKind::LeftOuter,
                    left_key: lk,
                    right_key: (0..rk.len() as ColIdx).collect(),
                    residual: None,
                },
                vec![outer, grouped],
                *contract,
                format!("{label} (scalar → left outer)"),
            );
            // The join appended the group key *and* the aggregate; the apply appends only
            // the aggregate. Project back, or every downstream column index would shift —
            // which is the kind of off-by-k that a structural test passes and a
            // denotational one does not.
            let keep: Vec<Scalar> = (0..outer_arity)
                .map(Scalar::Column)
                .chain(std::iter::once(Scalar::Column(
                    outer_arity + rk.len() as u16,
                )))
                .collect();
            out.add(
                Op::Map { exprs: keep },
                vec![joined],
                *contract,
                format!("{label} (drop the group key)"),
            )
        }
    }
}

/// A convenience for the corpus: the predicate `col = lit`, since every fixture needs it.
pub fn col_eq(col: ColIdx, lit: i128) -> Scalar {
    eq(Scalar::Column(col), Scalar::LitInt(lit))
}

/// The aggregate a scalar subquery most often carries, named so the corpus reads.
pub const SUBQUERY_SUM: Agg = Agg::Sum;
