//! **Answering a keyed aggregate by folding the base, instead of copying it first.**
//!
//! `Serving::query` materialised the whole `postings` relation as a
//! `BTreeMap<Vec<Value>, i128>` and evaluated the circuit over it. For an unkeyed `group by`
//! over the benchmark's twenty thousand postings that is 86,697 allocations and 17MB of
//! transient garbage per query — after the two copies removed in T-08 — before the aggregate
//! has looked at a row. The Z-set is not what the query needs; it is what the *reference
//! evaluator* needs, and the reference evaluator says so about itself in its first paragraph.
//!
//! This module recognises the fragment `Source → (Filter | Map)* → Aggregate{sum, count}` —
//! the same fragment `nilestream_core::rev::Runtime::install` accepts, which is the shape of
//! a balance, a position, a rollup and a running total — and answers it in one pass over the
//! ledger's own posting records, with a per-group accumulator and no intermediate Z-set at
//! all. A circuit outside the fragment is **refused rather than approximated**: `plan`
//! returns `None` and the caller takes the materialising path, which still exists and is
//! still correct.
//!
//! # Why this cannot become a second semantics
//!
//! Three deliberate constraints.
//!
//! *The scalars are evaluated by `niles_ir::eval::eval_scalar` and the predicates by
//! `keeps`.* Not by a copy of them. Three-valued comparison, null propagation and the
//! keep-on-unknown rule are the reference's, and a second implementation of them would
//! disagree in the null cases some months from now.
//!
//! *The aggregate is folded by the same rule `eval::fold` states*, restricted to the two
//! aggregates that are linear in the Z-set weight — `sum` and `count`. `min`, `max` and `avg`
//! are refused here and left to the reference: `min` genuinely is not a running fold over a
//! signed multiset, because a retraction arriving after the current minimum would have to
//! un-remove a row a one-pass fold has already forgotten.
//!
//! *Everything above the aggregate is evaluated by the reference*, through
//! `eval::try_run_with`, which takes the folded aggregate as a given node. So `order by` and
//! `limit` keep exactly the semantics — including `Limit`'s stated tie-breaking rule — that
//! the golden corpus pins down. There is no second copy of them here to drift.
//!
//! And the agreement is tested rather than argued: `the_fold_and_the_oracle_agree` runs every
//! shape through both paths and compares the Z-sets.
//!
//! # What the weights mean here
//!
//! The base is a set of posting records, each contributing weight 1. Materialising it as a
//! Z-set merges records that are equal in every column into one row of weight *n*. `sum` and
//! `count` are linear in the weight, so folding *n* records of weight 1 and folding one row
//! of weight *n* give the same answer — which is what makes this shortcut sound rather than
//! merely faster. It is exactly why the fragment stops at the linear aggregates.

use niles_ir::circuit::{Circuit, NodeId};
use niles_ir::eval::{self, Row, ZSet};
use niles_ir::operator::{Agg, ColIdx, Op, Scalar};
use niles_ir::value::Value;

/// One stage of the row pipeline, in the order the circuit applies it.
#[derive(Debug, Clone)]
enum Step {
    Filter(Scalar),
    Map(Vec<Scalar>),
}

/// A keyed aggregate that can be answered by one pass over the base.
#[derive(Debug, Clone)]
pub struct FoldPlan {
    /// The base relation the source names.
    pub relation: String,
    /// The aggregate node, whose value this plan computes. The caller hands it to
    /// `eval::try_run_with` so the rest of the circuit is evaluated by the reference.
    pub node: NodeId,
    steps: Vec<Step>,
    group_key: Vec<ColIdx>,
    aggs: Vec<(Agg, Scalar)>,
}

impl FoldPlan {
    /// Columns in the output: the group key, then one per aggregate.
    pub fn width(&self) -> usize {
        self.group_key.len() + self.aggs.len()
    }

    /// The columns the aggregate groups by, in output order.
    pub fn group_key(&self) -> &[ColIdx] {
        &self.group_key
    }

    /// The aggregates, in output order.
    pub fn aggs(&self) -> &[(Agg, Scalar)] {
        &self.aggs
    }

    /// Whether the chain from the base is filters and nothing else.
    ///
    /// The question a caller asks before answering from a maintained view instead of the
    /// base: a `Map` between the source and the aggregate changes what is being aggregated,
    /// so a view maintained over the unmapped rows is not an answer to this query.
    pub fn filters_only(&self) -> bool {
        self.steps.iter().all(|s| matches!(s, Step::Filter(_)))
    }
}

/// Find the aggregate this circuit's output rests on, if the path down to the base is inside
/// the fragment.
///
/// Conservative by construction. Anything unrecognised — a join, a fixpoint, a second
/// source, an aggregate this cannot fold — returns `None`, and the caller materialises. A
/// planner that guessed would be trading a wrong answer for a fast one.
pub fn plan(c: &Circuit, output: &str) -> Option<FoldPlan> {
    let out = *c.outputs.get(output)?;
    // Walk down from the output through the stages that sit *above* an aggregate, looking
    // for one. `order by` and `limit` are evaluated by the reference from the folded value,
    // so they may sit here; anything else means this is not the shape.
    let mut id = out;
    loop {
        let n = c.nodes.iter().find(|n| n.id == id)?;
        match &n.op {
            Op::Aggregate { group_key, aggs } => {
                if aggs.is_empty() || !aggs.iter().all(|(a, _)| matches!(a, Agg::Sum | Agg::Count))
                {
                    return None;
                }
                let steps = chain(c, *n.inputs.first()?)?;
                return Some(FoldPlan {
                    relation: steps.0,
                    node: n.id,
                    steps: steps.1,
                    group_key: group_key.clone(),
                    aggs: aggs.clone(),
                });
            }
            Op::OrderBy { .. } | Op::Limit { .. } | Op::Distinct => id = *n.inputs.first()?,
            _ => return None,
        }
    }
}

/// The `Source → (Filter | Map)*` chain feeding an aggregate, in application order.
fn chain(c: &Circuit, from: NodeId) -> Option<(String, Vec<Step>)> {
    let mut steps = Vec::new();
    let mut id = from;
    loop {
        let n = c.nodes.iter().find(|n| n.id == id)?;
        match &n.op {
            Op::Source {
                relation, is_base, ..
            } => {
                if !is_base {
                    // A derived source is a view this engine does not hold; the
                    // materialising path knows how to get it and this one does not.
                    return None;
                }
                steps.reverse();
                return Some((relation.clone(), steps));
            }
            Op::Filter { predicate } => {
                steps.push(Step::Filter(predicate.clone()));
                id = *n.inputs.first()?;
            }
            Op::Map { exprs } => {
                steps.push(Step::Map(exprs.clone()));
                id = *n.inputs.first()?;
            }
            _ => return None,
        }
    }
}

/// A group's running accumulators, one per aggregate.
#[derive(Clone)]
struct Acc {
    /// `sum` and `count` are both an integer accumulator; `any` distinguishes "no non-null
    /// value was seen" from "the values summed to zero", which `eval::fold` reports as null
    /// and zero respectively. Losing that distinction is §1.1.1's defect in aggregate form.
    total: i128,
    any: bool,
}

/// **The fold, fed one base record at a time.**
///
/// A pushing interface rather than a pulling one, and the reason is measurable: pulling
/// requires one iterator type for the whole-base scan and another for the anchor-index scan,
/// which in practice means collecting both into a buffer first. Buffering twenty thousand
/// five-column rows costs 10MB of `Vec` growth per query — more than the Z-set this module
/// exists to avoid building. Pushed, the scan streams and the fold allocates once per
/// *group*.
pub struct Folder<'p> {
    plan: &'p FoldPlan,
    /// Two row buffers for the whole query rather than two allocations per row. A `Map`
    /// writes into the spare and the two swap, so a chain of maps costs nothing.
    cur: Vec<Value>,
    spare: Vec<Value>,
    key: Row,
    groups: Groups,
    /// Every group's accumulators, `plan.aggs.len()` of them per group, in the order the
    /// groups were first seen. The map holds the group's index into this.
    accs: Vec<Acc>,
    work: u64,
}

/// The accumulator table, in one of two shapes.
///
/// **The scalar shape is the whole of T-04.** A single-column group key was still stored as a
/// `Vec<Value>` of length one, so a `group by acct` over ten thousand accounts allocated a
/// one-element heap vector per group to look one up and another to keep it, and then `finish`
/// allocated a second one-element vector per group to build the output row and reached it
/// through `eval::add`'s lookup. That is three allocations and two tree descents per group
/// where the key is one integer sitting inside a box.
///
/// `Value` has exactly two variants, so **a one-column key is always an `Option<i128>`** —
/// `None` for `Null`, `Some(x)` for `Int(x)`. No schema, no column kinds, no type inference:
/// the specialisation is sound by the shape of `Value` itself, and the `match` in
/// [`Folder::scalar_key`] is exhaustive, so a third variant would be a compile error here
/// rather than a silently wrong grouping.
///
/// The orderings agree, which is what lets `finish` emit straight into the Z-set. `Value`
/// derives `Ord` with `Null` declared first, so `Null < Int(_)`; `Option` orders `None`
/// before `Some`; and two rows whose first column differs are ordered by that column alone.
/// A `BTreeMap<Option<i128>, _>` therefore yields groups in exactly the order the output
/// `ZSet` wants them, and each row can be `insert`ed rather than `add`ed.
/// The map holds an **index**, not a vector of accumulators. Every group's accumulators live
/// end to end in one arena, so a group costs no allocation of its own at all: `Vec<Acc>` per
/// group was the second of the three, and ten thousand two-word vectors is ten thousand
/// allocations to hold what is, for almost every query in the fragment, a single integer.
enum Groups {
    /// One integer (or null) per group.
    Scalar(std::collections::BTreeMap<Option<i128>, u32>),
    /// Anything else: no key columns at all, or two or more.
    Wide(std::collections::BTreeMap<Row, u32>),
}

impl<'p> Folder<'p> {
    pub fn new(plan: &'p FoldPlan) -> Self {
        Folder {
            plan,
            cur: Vec::with_capacity(8),
            spare: Vec::with_capacity(8),
            key: Vec::with_capacity(plan.group_key.len()),
            // The specialisation is chosen once, from the plan, and never re-decided per
            // row: a fold that could change shape half way through would be two folds.
            groups: if plan.group_key.len() == 1 {
                Groups::Scalar(Default::default())
            } else {
                Groups::Wide(Default::default())
            },
            accs: Vec::new(),
            work: 0,
        }
    }

    /// The one-column key as an `Option<i128>`.
    ///
    /// Exhaustive on purpose. `Value` has two variants today and this mapping is total; if a
    /// third is ever added, this match stops compiling and whoever adds it has to decide what
    /// it means for a group key, rather than discovering that some values silently grouped
    /// together.
    fn scalar_key(v: Value) -> Option<i128> {
        match v {
            Value::Null => None,
            Value::Int(x) => Some(x),
        }
    }

    /// Feed one base record, in the source's column order.
    pub fn row(&mut self, base: &[Value]) {
        self.work += 1;
        self.cur.clear();
        self.cur.extend_from_slice(base);
        for step in &self.plan.steps {
            match step {
                Step::Filter(p) => {
                    if !eval::keeps(p, &self.cur) {
                        return;
                    }
                }
                Step::Map(exprs) => {
                    self.spare.clear();
                    for e in exprs {
                        self.spare.push(eval::eval_scalar(e, &self.cur));
                    }
                    std::mem::swap(&mut self.cur, &mut self.spare);
                }
            }
        }
        let plan = self.plan;
        let n = plan.aggs.len();
        // Look the group up (or open it) and come away with an **index**, so the borrow of
        // the map ends before the arena is touched. A group costs no allocation of its own:
        // on the scalar path not even a key, since `Option<i128>` is `Copy` and lives in the
        // tree node.
        let next = (self.accs.len() / n.max(1)) as u32;
        let at = match &mut self.groups {
            Groups::Scalar(g) => {
                let k = Self::scalar_key(
                    self.cur
                        .get(plan.group_key[0] as usize)
                        .copied()
                        .unwrap_or(Value::Null),
                );
                match g.get(&k) {
                    Some(i) => *i,
                    None => *g.entry(k).or_insert(next),
                }
            }
            Groups::Wide(g) => {
                self.key.clear();
                for c in &plan.group_key {
                    self.key
                        .push(self.cur.get(*c as usize).copied().unwrap_or(Value::Null));
                }
                // The key is cloned only the first time its group is seen. That is the whole
                // difference between O(base) and O(groups).
                match g.get(&self.key) {
                    Some(i) => *i,
                    None => *g.entry(self.key.clone()).or_insert(next),
                }
            }
        };
        if at == next {
            self.accs.resize(
                self.accs.len() + n,
                Acc {
                    total: 0,
                    any: false,
                },
            );
        }
        let slot = &mut self.accs[at as usize * n..at as usize * n + n];
        for (i, (a, e)) in plan.aggs.iter().enumerate() {
            match a {
                // `count` counts the weight whatever the expression evaluates to, which is
                // `eval::fold`'s rule and is why a null does not reduce a count.
                Agg::Count => {
                    slot[i].total += 1;
                    slot[i].any = true;
                }
                Agg::Sum => {
                    if let Value::Int(x) = eval::eval_scalar(e, &self.cur) {
                        slot[i].total += x;
                        slot[i].any = true;
                    }
                }
                // `plan` refuses these, so this is unreachable. A refusal rather than a
                // default, because a default here would answer `min` with a sum.
                Agg::Min | Agg::Max | Agg::Avg => unreachable!("plan() accepts sum and count"),
            }
        }
    }

    /// The aggregate's Z-set, and the counted work in the unit `eval` counts: one per row
    /// read, one per group emitted.
    pub fn finish(self) -> (ZSet, u64) {
        let mut work = self.work;
        let aggs = &self.plan.aggs;
        // The aggregated cells of one group, appended to a row that already holds its key.
        let close = |row: &mut Row, accs: &[Acc]| {
            for (i, (a, _)) in aggs.iter().enumerate() {
                row.push(match a {
                    Agg::Count => Value::Int(accs[i].total),
                    // `sum` over no non-null rows is null, not zero.
                    Agg::Sum if accs[i].any => Value::Int(accs[i].total),
                    _ => Value::Null,
                });
            }
        };
        let n = aggs.len();
        let arena = &self.accs;
        let of = |i: u32| &arena[i as usize * n..i as usize * n + n];
        // **The rows come out in ascending order, so the Z-set is built in bulk.**
        //
        // Both accumulator maps are `BTreeMap`s, so they iterate in ascending key order; the
        // orderings agree (see [`Groups`]); and two groups with different keys give two rows
        // that differ in their first columns. So the rows arrive sorted and distinct, and
        // `eval::add`'s get-then-insert — thirteen `Vec<Value>` comparisons down a tree that
        // is being rebalanced as it grows, ten thousand times — is doing a search whose
        // answer is already known.
        //
        // `collect` into a `BTreeMap` takes std's bulk path: it sorts (linear on input that
        // is already ordered) and then builds the tree bottom-up with no rebalancing at all.
        // The intermediate `Vec` costs a handful of allocations from its growth, against ten
        // thousand tree insertions.
        let mut rows: Vec<(Row, i128)> = Vec::with_capacity(match &self.groups {
            Groups::Scalar(g) => g.len(),
            Groups::Wide(g) => g.len(),
        });
        match &self.groups {
            Groups::Scalar(g) => {
                for (k, i) in g {
                    work += 1;
                    let mut row = Row::with_capacity(1 + n);
                    row.push(match k {
                        None => Value::Null,
                        Some(x) => Value::Int(*x),
                    });
                    close(&mut row, of(*i));
                    rows.push((row, 1));
                }
            }
            Groups::Wide(g) => {
                for (k, i) in g {
                    work += 1;
                    let mut row = k.clone();
                    close(&mut row, of(*i));
                    rows.push((row, 1));
                }
            }
        }
        (rows.into_iter().collect(), work)
    }
}

/// Fold an iterator of base records. The pulling form, for callers that have one.
pub fn fold<'r, I>(plan: &FoldPlan, rows: I) -> (ZSet, u64)
where
    I: IntoIterator<Item = &'r [Value]>,
{
    let mut f = Folder::new(plan);
    for r in rows {
        f.row(r);
    }
    f.finish()
}
