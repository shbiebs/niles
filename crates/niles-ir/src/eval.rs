//! **The reference semantics of a circuit.**
//!
//! A small, deliberately slow interpreter over Z-sets. It is not the engine and never will
//! be: it exists so that a rewrite can be checked *denotationally* — run the circuit before
//! and after, on the same data, and compare the answers — rather than structurally, by
//! asserting that the plan has the shape someone expected.
//!
//! Structural assertions are the weaker kind and it is worth saying why. "The plan contains
//! a semi-join" is satisfied by a plan that contains a semi-join computing the wrong thing.
//! "The two plans denote the same Z-set on this dataset" is not. The unnesting corpus uses
//! the second form, which is why this module is public rather than living in a `#[cfg(test)]`
//! block where the schedule catalogue's evaluator used to.
//!
//! # Z-sets, and why the weights are signed
//!
//! A collection is a multiset with signed multiplicities. Signs are not decoration: they
//! are how this IR expresses `except`, retractions, and outer-join maintenance, and a
//! rewrite can be sound on sets, unsound on bags, and unsound again on Z-sets. The
//! generators used by the corpora produce negative weights on purpose.
//!
//! The one rule that follows from this and is easy to get wrong: **a zero weight is not a
//! member**. Two equal Z-sets must compare equal, so a row whose weight cancels to zero is
//! removed rather than kept at zero — otherwise every denotation test would be a test of
//! the encoding instead.
//!
//! # Counted work
//!
//! [`Eval::work`] counts row-level operations: every source row read, every pair a join
//! considers, every group an aggregate touches. It is the unit E17 reports the
//! nested/unnested ratio in, and it is the honest unit for a *rewrite* gate — a rewrite
//! either does less work or it does not, and that is a property of the plan rather than of
//! the machine it runs on.

use crate::circuit::{Circuit, NodeId};
use crate::operator::{Agg, ApplyKind, ColIdx, JoinKind, Op, Scalar, ScalarOp};
use crate::value::{arith, compare, truth, Tri, Value};
use std::collections::BTreeMap;

pub type Row = Vec<Value>;
pub type ZSet = BTreeMap<Row, i128>;

/// Add `w` to `row`'s weight, removing the row if the weight cancels to zero.
pub fn add(z: &mut ZSet, row: Row, w: i128) {
    if w == 0 {
        return;
    }
    let e = z.entry(row.clone()).or_insert(0);
    *e += w;
    if *e == 0 {
        z.remove(&row);
    }
}

/// Build a Z-set from `(row, weight)` pairs, with integer rows for brevity.
pub fn zset(rows: &[(&[i128], i128)]) -> ZSet {
    let mut z = ZSet::new();
    for (r, w) in rows {
        add(&mut z, r.iter().map(|v| Value::Int(*v)).collect(), *w);
    }
    z
}

/// Build a row that may contain nulls, from `Option<i128>`.
pub fn row(vs: &[Option<i128>]) -> Row {
    vs.iter()
        .map(|v| v.map(Value::Int).unwrap_or(Value::Null))
        .collect()
}

/// Evaluate a scalar against a row. Nulls propagate; comparisons are three-valued.
pub fn eval_scalar(s: &Scalar, r: &Row) -> Value {
    match s {
        Scalar::Column(c) => *r.get(*c as usize).unwrap_or(&Value::Null),
        Scalar::LitInt(v) => Value::Int(*v),
        Scalar::LitBool(b) => Value::Int(*b as i128),
        Scalar::LitMoney { minor, .. } => Value::Int(*minor),
        Scalar::LitText(_) | Scalar::Anchor => Value::Int(0),
        Scalar::LitNull => Value::Null,
        Scalar::IsNull(inner) => Tri::of(eval_scalar(inner, r).is_null()).definite(),
        Scalar::Not(inner) => match truth(eval_scalar(inner, r)) {
            Tri::Unknown => Value::Null,
            t => t.not().definite(),
        },
        Scalar::Neg(inner) => arith(eval_scalar(inner, r), Value::Int(0), |a, _| -a),
        Scalar::Udf { .. } => Value::Int(0),
        Scalar::Binary { op, lhs, rhs } => {
            let (a, b) = (eval_scalar(lhs, r), eval_scalar(rhs, r));
            match op {
                ScalarOp::Add => arith(a, b, |x, y| x + y),
                ScalarOp::Sub => arith(a, b, |x, y| x - y),
                ScalarOp::Mul => arith(a, b, |x, y| x * y),
                ScalarOp::Div => arith(a, b, |x, y| if y == 0 { 0 } else { x / y }),
                ScalarOp::Rem => arith(a, b, |x, y| if y == 0 { 0 } else { x % y }),
                ScalarOp::Eq => tri_value(compare(a, b, |x, y| x == y)),
                ScalarOp::Ne => tri_value(compare(a, b, |x, y| x != y)),
                ScalarOp::Lt => tri_value(compare(a, b, |x, y| x < y)),
                ScalarOp::Le => tri_value(compare(a, b, |x, y| x <= y)),
                ScalarOp::Gt => tri_value(compare(a, b, |x, y| x > y)),
                ScalarOp::Ge => tri_value(compare(a, b, |x, y| x >= y)),
                ScalarOp::And => tri_value(truth(a).and(truth(b))),
                ScalarOp::Or => tri_value(truth(a).or(truth(b))),
                ScalarOp::Like => tri_value(compare(a, b, |x, y| x == y)),
            }
        }
    }
}

/// A three-valued result, carried back into the value domain: unknown *is* null.
fn tri_value(t: Tri) -> Value {
    match t {
        Tri::Unknown => Value::Null,
        other => other.definite(),
    }
}

/// Whether a predicate keeps a row. Unknown discards, exactly as in `where`.
pub fn keeps(p: &Scalar, r: &Row) -> bool {
    truth(eval_scalar(p, r)).keeps()
}

/// The evaluator, carrying the work counter.
pub struct Eval<'a> {
    circuit: &'a Circuit,
    sources: &'a BTreeMap<String, ZSet>,
    /// Row-level operations performed. See the module docs.
    pub work: u64,
}

/// Evaluate a named output and report the counted work.
pub fn run(c: &Circuit, output: &str, sources: &BTreeMap<String, ZSet>) -> (ZSet, u64) {
    let id = *c
        .outputs
        .get(output)
        .unwrap_or_else(|| panic!("no output named `{output}`"));
    run_node(c, id, sources)
}

/// Evaluate one node and report the counted work.
pub fn run_node(c: &Circuit, id: NodeId, sources: &BTreeMap<String, ZSet>) -> (ZSet, u64) {
    let mut e = Eval {
        circuit: c,
        sources,
        work: 0,
    };
    let z = e.node(id);
    (z, e.work)
}

impl<'a> Eval<'a> {
    fn node(&mut self, id: NodeId) -> ZSet {
        let n = self
            .circuit
            .nodes
            .iter()
            .find(|n| n.id == id)
            .expect("node");
        match &n.op {
            Op::Source { relation, .. } => {
                let z = self.sources.get(relation).cloned().unwrap_or_default();
                self.work += z.len() as u64;
                z
            }
            Op::Filter { predicate } => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp {
                    self.work += 1;
                    if keeps(predicate, &r) {
                        add(&mut out, r, w);
                    }
                }
                out
            }
            Op::Map { exprs } => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp {
                    self.work += 1;
                    add(
                        &mut out,
                        exprs.iter().map(|e| eval_scalar(e, &r)).collect(),
                        w,
                    );
                }
                out
            }
            Op::Join {
                kind,
                left_key,
                right_key,
                residual,
            } => {
                let l = self.node(n.inputs[0]);
                let r = self.node(n.inputs[1]);
                self.join(*kind, left_key, right_key, residual.as_ref(), &l, &r)
            }
            Op::Aggregate { group_key, aggs } => {
                let inp = self.node(n.inputs[0]);
                self.aggregate(group_key, aggs, &inp)
            }
            Op::Distinct => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp {
                    self.work += 1;
                    if w > 0 {
                        add(&mut out, r, 1);
                    }
                }
                out
            }
            Op::Union => {
                let mut out = ZSet::new();
                for k in 0..n.inputs.len() {
                    let z = self.node(n.inputs[k]);
                    for (r, w) in z {
                        self.work += 1;
                        add(&mut out, r, w);
                    }
                }
                out
            }
            Op::Negate => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp {
                    self.work += 1;
                    add(&mut out, r, -w);
                }
                out
            }
            Op::Apply { kind, correlation } => {
                let outer = self.node(n.inputs[0]);
                let inner = self.node(n.inputs[1]);
                self.apply(kind, correlation, &outer, &inner)
            }
            // Pass-throughs at this level of abstraction: the reference semantics is about
            // *what* a circuit denotes, and these operators change when or how it is
            // computed rather than what comes out.
            Op::Index { .. }
            | Op::AsOf { .. }
            | Op::ValidAt { .. }
            | Op::Integrate
            | Op::Differentiate
            | Op::Delay => self.node(n.inputs[0]),
            other => panic!("the reference evaluator does not cover {}", other.name()),
        }
    }

    /// The key a row contributes to, or `None` if any key column is null.
    ///
    /// Null is not a key. `null = null` is unknown and unknown does not join, so a row
    /// with a null in a key column belongs to no bucket on either side — which is the same
    /// rule as the three-valued comparison, expressed once so the two cannot drift.
    fn key_of(r: &Row, cols: &[ColIdx]) -> Option<Row> {
        let mut k = Vec::with_capacity(cols.len());
        for c in cols {
            let v = *r.get(*c as usize).unwrap_or(&Value::Null);
            if v.is_null() {
                return None;
            }
            k.push(v);
        }
        Some(k)
    }

    /// An equi-join, executed by **building an index on the right and probing it**.
    ///
    /// This is not a performance detail of the reference evaluator; it is what makes
    /// [`Eval::work`] a meaningful number. The first version looped over every pair, which
    /// costs `|L| x |R|` — the same as a dependent join — and so reported that unnesting
    /// saved nothing. It was the evaluator that was wrong, not the rewrite: no engine
    /// executes an equi-join as a nested loop, and a cost model that says otherwise cannot
    /// distinguish the two plans the unnesting gate exists to compare.
    ///
    /// Counted work is therefore `|R|` to build, plus `|L|` to probe, plus one per pair
    /// actually produced. An `Apply` keeps its nested loop, because that is what a
    /// dependent join *is*, and the difference between the two is the result E17 reports.
    ///
    /// An empty key is the honest exception: every left row matches every right row, the
    /// single bucket holds the whole relation, and the work is quadratic again. That is
    /// correct — an empty-key join is a cross product — and it is why the corpus reports
    /// its uncorrelated cases separately.
    fn join(
        &mut self,
        kind: JoinKind,
        lk: &[ColIdx],
        rk: &[ColIdx],
        residual: Option<&Scalar>,
        l: &ZSet,
        r: &ZSet,
    ) -> ZSet {
        let width_r = r.keys().next().map(|x| x.len()).unwrap_or(0);
        let mut index: BTreeMap<Row, Vec<(&Row, i128)>> = BTreeMap::new();
        for (rrow, rw) in r {
            self.work += 1;
            if let Some(k) = Self::key_of(rrow, rk) {
                index.entry(k).or_default().push((rrow, *rw));
            }
        }
        let empty: Vec<(&Row, i128)> = Vec::new();

        let mut out = ZSet::new();
        for (lrow, lw) in l {
            self.work += 1;
            let bucket = match Self::key_of(lrow, lk) {
                Some(k) => index.get(&k).unwrap_or(&empty),
                // A null key probes nothing, on either side.
                None => &empty,
            };
            // Membership is decided by the matching set's *total weight*, not its row
            // count: a row inserted and then retracted is not there, and a semi-join that
            // counted rows would say it was.
            let mut total: i128 = 0;
            let mut any = false;
            for (rrow, rw) in bucket {
                self.work += 1;
                let mut combined = lrow.clone();
                combined.extend(rrow.iter().copied());
                if let Some(res) = residual {
                    if !keeps(res, &combined) {
                        continue;
                    }
                }
                total += rw;
                any = true;
                if matches!(kind, JoinKind::Inner | JoinKind::LeftOuter) {
                    // Weights multiply: the Z-set semantics of a join, and the reason a
                    // rewrite sound on sets can be unsound here.
                    add(&mut out, combined, lw * rw);
                }
            }
            let present = total > 0;
            match kind {
                // Semi and anti emit the *left row unchanged, at its own weight*. This is
                // the property that makes unnesting safe: an `exists` must never turn one
                // outer row into three because three inner rows matched.
                JoinKind::Semi if present => add(&mut out, lrow.clone(), *lw),
                JoinKind::Anti if !present => add(&mut out, lrow.clone(), *lw),
                JoinKind::LeftOuter if !any => {
                    let mut combined = lrow.clone();
                    combined.extend(std::iter::repeat(Value::Null).take(width_r));
                    add(&mut out, combined, *lw);
                }
                _ => {}
            }
        }
        out
    }

    /// Grouped aggregation, in two passes.
    ///
    /// Two rather than one because a Z-set is unordered and carries signed weights, so
    /// `min` is not a running fold: a retraction arriving after the current minimum would
    /// have to un-remove a row a one-pass fold has already forgotten. Collecting each
    /// group's contributions and folding at the end is slower and is the semantics the
    /// theorems quantify over; the engine's incremental version has to *agree* with this,
    /// which is what makes having a reference worth the duplication.
    fn aggregate(&mut self, group_key: &[ColIdx], aggs: &[(Agg, Scalar)], inp: &ZSet) -> ZSet {
        let mut raw: BTreeMap<Row, Vec<Vec<(Value, i128)>>> = BTreeMap::new();
        for (r, w) in inp {
            self.work += 1;
            let k: Row = group_key
                .iter()
                .map(|c| *r.get(*c as usize).unwrap_or(&Value::Null))
                .collect();
            let slot = raw.entry(k).or_insert_with(|| vec![Vec::new(); aggs.len()]);
            for (i, (_, e)) in aggs.iter().enumerate() {
                slot[i].push((eval_scalar(e, r), *w));
            }
        }
        let mut out = ZSet::new();
        for (k, per_agg) in raw {
            self.work += 1;
            let mut row_out = k;
            for (i, (a, _)) in aggs.iter().enumerate() {
                row_out.push(fold(*a, &per_agg[i]));
            }
            add(&mut out, row_out, 1);
        }
        out
    }

    fn apply(
        &mut self,
        kind: &ApplyKind,
        correlation: &[(ColIdx, ColIdx)],
        outer: &ZSet,
        inner: &ZSet,
    ) -> ZSet {
        let lk: Vec<ColIdx> = correlation.iter().map(|(a, _)| *a).collect();
        let rk: Vec<ColIdx> = correlation.iter().map(|(_, b)| *b).collect();
        let mut out = ZSet::new();
        for (orow, ow) in outer {
            // One charge for looking at the outer row, matching what a join pays per left
            // row. Without it the two plans would be compared on different accounting, and
            // an `Apply` over an empty inner relation would look free.
            self.work += 1;
            // Then the nested loop that gives the operator its name and its cost: every
            // outer row re-scans the inner relation, and that is what unnesting removes.
            let mut group: Vec<(&Row, i128)> = Vec::new();
            for (irow, iw) in inner {
                self.work += 1;
                let ok = match (Self::key_of(orow, &lk), Self::key_of(irow, &rk)) {
                    (Some(a), Some(b)) => a == b,
                    // A null on either side of the correlation is unknown, and unknown
                    // does not correlate.
                    _ => false,
                };
                if ok {
                    group.push((irow, *iw));
                }
            }
            let nonempty = group.iter().map(|(_, w)| *w).sum::<i128>() > 0;
            match kind {
                ApplyKind::Exists => {
                    if nonempty {
                        add(&mut out, orow.clone(), *ow);
                    }
                }
                ApplyKind::NotExists => {
                    if !nonempty {
                        add(&mut out, orow.clone(), *ow);
                    }
                }
                ApplyKind::In { probe, inner: icol } => {
                    let p = orow[*probe as usize];
                    // A null probe is unknown against everything, including an empty set.
                    if p.is_null() {
                        continue;
                    }
                    let found = group.iter().any(|(r, w)| {
                        *w > 0 && compare(p, r[*icol as usize], |a, b| a == b).keeps()
                    });
                    if found {
                        add(&mut out, orow.clone(), *ow);
                    }
                }
                ApplyKind::NotIn { probe, inner: icol } => {
                    let p = orow[*probe as usize];
                    if p.is_null() {
                        continue;
                    }
                    let mut verdict = Tri::True;
                    for (r, w) in &group {
                        if *w <= 0 {
                            continue;
                        }
                        // `p <> r` is unknown when `r` is null, and Kleene's `and` lets
                        // a definite `false` win over unknown — which is exactly why one
                        // matching row still rules the whole predicate false, while one
                        // null with no match leaves it unknown.
                        verdict = verdict.and(compare(p, r[*icol as usize], |a, b| a != b));
                    }
                    if verdict.keeps() {
                        add(&mut out, orow.clone(), *ow);
                    }
                }
                ApplyKind::Scalar { agg, expr } => {
                    let vals: Vec<(Value, i128)> = group
                        .iter()
                        .map(|(r, w)| (eval_scalar(expr, r), *w))
                        .collect();
                    // An empty matching set yields null — SQL's rule for a scalar
                    // subquery, and the reason the unnested form is a *left outer* join.
                    let v = if vals.is_empty() {
                        Value::Null
                    } else {
                        fold(*agg, &vals)
                    };
                    let mut r = orow.clone();
                    r.push(v);
                    add(&mut out, r, *ow);
                }
            }
        }
        out
    }
}

/// Fold one aggregate over a group's `(value, weight)` contributions.
///
/// Weights matter: `count` sums them, `sum` multiplies, and `min`/`max` ignore rows whose
/// weight is not positive because a retracted row is not in the collection.
pub fn fold(a: Agg, vals: &[(Value, i128)]) -> Value {
    match a {
        Agg::Count => Value::Int(vals.iter().map(|(_, w)| *w).sum()),
        Agg::Sum => {
            let mut acc = 0i128;
            let mut any = false;
            for (v, w) in vals {
                if let Value::Int(x) = v {
                    acc += x * w;
                    any = true;
                }
            }
            // `sum` over no non-null rows is null, not zero. The distinction is the
            // §1.1.1 defect in aggregate form.
            if any {
                Value::Int(acc)
            } else {
                Value::Null
            }
        }
        Agg::Min | Agg::Max => {
            let live: Vec<i128> = vals
                .iter()
                .filter(|(_, w)| *w > 0)
                .filter_map(|(v, _)| v.int())
                .collect();
            match if a == Agg::Min {
                live.iter().min()
            } else {
                live.iter().max()
            } {
                Some(x) => Value::Int(*x),
                None => Value::Null,
            }
        }
        Agg::Avg => {
            let live: Vec<(i128, i128)> = vals
                .iter()
                .filter(|(_, w)| *w > 0)
                .filter_map(|(v, w)| v.int().map(|x| (x, *w)))
                .collect();
            let n: i128 = live.iter().map(|(_, w)| *w).sum();
            if n == 0 {
                Value::Null
            } else {
                Value::Int(live.iter().map(|(x, w)| x * w).sum::<i128>() / n)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::internal_contract;
    use crate::operator::Agg;

    fn src(c: &mut Circuit, name: &str) -> NodeId {
        c.add(
            Op::Source {
                relation: name.into(),
                is_base: true,
                anchor_key: vec![0],
            },
            vec![],
            internal_contract(),
            name,
        )
    }

    fn sources(pairs: &[(&str, ZSet)]) -> BTreeMap<String, ZSet> {
        pairs
            .iter()
            .map(|(n, z)| (n.to_string(), z.clone()))
            .collect()
    }

    #[test]
    fn a_zero_weight_row_is_not_a_member() {
        let mut z = ZSet::new();
        add(&mut z, row(&[Some(1)]), 3);
        add(&mut z, row(&[Some(1)]), -3);
        assert!(
            z.is_empty(),
            "a cancelled row must leave no trace, or equality is encoding-dependent"
        );
    }

    #[test]
    fn a_semi_join_does_not_duplicate_its_left_rows() {
        // The defect the whole exercise is about. Three matching right rows must yield one
        // left row, not three — an inner join followed by a projection would give three.
        let mut c = Circuit::new();
        let l = src(&mut c, "l");
        let r = src(&mut c, "r");
        let j = c.add(
            Op::Join {
                kind: JoinKind::Semi,
                left_key: vec![0],
                right_key: vec![0],
                residual: None,
            },
            vec![l, r],
            internal_contract(),
            "semi",
        );
        c.set_output("out", j);
        let s = sources(&[
            ("l", zset(&[(&[1, 100], 1)])),
            ("r", zset(&[(&[1, 7], 1), (&[1, 8], 1), (&[1, 9], 1)])),
        ]);
        let (out, _) = run(&c, "out", &s);
        assert_eq!(out, zset(&[(&[1, 100], 1)]));

        // And the inner join for contrast, so the test shows the difference rather than
        // asserting the absence of one.
        let mut c2 = Circuit::new();
        let l2 = src(&mut c2, "l");
        let r2 = src(&mut c2, "r");
        let j2 = c2.add(
            Op::Join {
                kind: JoinKind::Inner,
                left_key: vec![0],
                right_key: vec![0],
                residual: None,
            },
            vec![l2, r2],
            internal_contract(),
            "inner",
        );
        c2.set_output("out", j2);
        let (out2, _) = run(&c2, "out", &s);
        assert_eq!(
            out2.values().sum::<i128>(),
            3,
            "an inner join does duplicate; a semi-join must not"
        );
    }

    #[test]
    fn a_retracted_row_does_not_satisfy_exists() {
        // Membership is decided by total weight, not by row count. A right row inserted
        // and then retracted is not there, and a semi-join that counted rows would say it
        // was — which in a ledger is the difference between an account that has a hold on
        // it and one that had one.
        let mut c = Circuit::new();
        let l = src(&mut c, "l");
        let r = src(&mut c, "r");
        let j = c.add(
            Op::Join {
                kind: JoinKind::Semi,
                left_key: vec![0],
                right_key: vec![0],
                residual: None,
            },
            vec![l, r],
            internal_contract(),
            "semi",
        );
        c.set_output("out", j);
        let s = sources(&[
            ("l", zset(&[(&[1], 1)])),
            ("r", zset(&[(&[1], 1), (&[1], -1)])),
        ]);
        let (out, _) = run(&c, "out", &s);
        assert!(
            out.is_empty(),
            "a cancelled right row must not satisfy `exists`"
        );
    }

    #[test]
    fn a_null_key_joins_to_nothing_including_another_null() {
        let mut c = Circuit::new();
        let l = src(&mut c, "l");
        let r = src(&mut c, "r");
        let j = c.add(
            Op::Join {
                kind: JoinKind::Inner,
                left_key: vec![0],
                right_key: vec![0],
                residual: None,
            },
            vec![l, r],
            internal_contract(),
            "j",
        );
        c.set_output("out", j);
        let mut ls = ZSet::new();
        add(&mut ls, row(&[None]), 1);
        let mut rs = ZSet::new();
        add(&mut rs, row(&[None]), 1);
        let (out, _) = run(&c, "out", &sources(&[("l", ls), ("r", rs)]));
        assert!(
            out.is_empty(),
            "null = null is unknown, and unknown does not join"
        );
    }

    #[test]
    fn a_left_outer_join_null_extends_rather_than_dropping() {
        let mut c = Circuit::new();
        let l = src(&mut c, "l");
        let r = src(&mut c, "r");
        let j = c.add(
            Op::Join {
                kind: JoinKind::LeftOuter,
                left_key: vec![0],
                right_key: vec![0],
                residual: None,
            },
            vec![l, r],
            internal_contract(),
            "j",
        );
        c.set_output("out", j);
        let s = sources(&[
            ("l", zset(&[(&[1], 1), (&[2], 1)])),
            ("r", zset(&[(&[1, 50], 1)])),
        ]);
        let (out, _) = run(&c, "out", &s);
        let mut want = ZSet::new();
        add(&mut want, row(&[Some(1), Some(1), Some(50)]), 1);
        add(&mut want, row(&[Some(2), None, None]), 1);
        assert_eq!(out, want);
    }

    #[test]
    fn sum_over_no_rows_is_null_and_count_is_zero() {
        // The aggregate form of the §1.1.1 defect. `sum` of nothing is not 0, because 0 is
        // an answer and "there was nothing to add" is not.
        assert_eq!(fold(Agg::Sum, &[]), Value::Null);
        assert_eq!(fold(Agg::Count, &[]), Value::Int(0));
        assert_eq!(fold(Agg::Sum, &[(Value::Null, 1)]), Value::Null);
        assert_eq!(
            fold(Agg::Sum, &[(Value::Int(3), 2), (Value::Int(4), 1)]),
            Value::Int(10)
        );
    }

    #[test]
    fn min_ignores_retracted_rows() {
        assert_eq!(
            fold(Agg::Min, &[(Value::Int(1), -1), (Value::Int(5), 1)]),
            Value::Int(5)
        );
        assert_eq!(
            fold(Agg::Max, &[(Value::Int(1), 1), (Value::Int(5), 1)]),
            Value::Int(5)
        );
    }

    #[test]
    fn work_is_counted_and_a_nested_apply_costs_the_product() {
        let mut c = Circuit::new();
        let l = src(&mut c, "l");
        let r = src(&mut c, "r");
        let a = c.add(
            Op::Apply {
                kind: ApplyKind::Exists,
                correlation: vec![(0, 0)],
            },
            vec![l, r],
            internal_contract(),
            "apply",
        );
        c.set_output("out", a);
        let ls = zset(&[(&[1], 1), (&[2], 1), (&[3], 1)]);
        let rs = zset(&[(&[1], 1), (&[2], 1), (&[9], 1), (&[8], 1)]);
        let (out, work) = run(&c, "out", &sources(&[("l", ls), ("r", rs)]));
        assert_eq!(out, zset(&[(&[1], 1), (&[2], 1)]));
        // 3 source rows + 4 source rows + 3 outer-row charges + 3x4 nested-loop
        // comparisons. The per-outer-row charge matches what a join pays per left row, so
        // the two plans are compared on the same accounting — without it an `Apply` over an
        // empty inner relation would look free.
        assert_eq!(work, 3 + 4 + 3 + 12);

        // And the contrast that makes the number mean something: the same query as a
        // semi-join costs a build, a probe and the pairs, not the product.
        let mut c2 = Circuit::new();
        let l2 = src(&mut c2, "l");
        let r2 = src(&mut c2, "r");
        let j = c2.add(
            Op::Join {
                kind: JoinKind::Semi,
                left_key: vec![0],
                right_key: vec![0],
                residual: None,
            },
            vec![l2, r2],
            internal_contract(),
            "semi",
        );
        c2.set_output("out", j);
        let ls2 = zset(&[(&[1], 1), (&[2], 1), (&[3], 1)]);
        let rs2 = zset(&[(&[1], 1), (&[2], 1), (&[9], 1), (&[8], 1)]);
        let (out2, work2) = run(&c2, "out", &sources(&[("l", ls2), ("r", rs2)]));
        assert_eq!(out2, out, "the rewrite must not change the answer");
        assert!(
            work2 < work,
            "the set-at-a-time form must cost less: {work} vs {work2}"
        );
    }
}
