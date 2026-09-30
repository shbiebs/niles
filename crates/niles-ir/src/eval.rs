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
use std::borrow::Cow;
use std::collections::BTreeMap;

pub type Row = Vec<Value>;
pub type ZSet = BTreeMap<Row, i128>;

/// Add `w` to `row`'s weight, removing the row if the weight cancels to zero.
///
/// The row is **moved**, not cloned. `z.entry(row.clone()).or_insert(0)` kept the original
/// alive only so the cancel-to-zero case could `remove(&row)` afterwards, and paid a heap
/// allocation for every row inserted into every intermediate Z-set to do it — around twenty
/// thousand of them per analytical query over the benchmark's base, on top of the twenty
/// thousand the row itself costs.
///
/// The cancellation case now pays the extra lookup instead, which is the right way round: a
/// weight cancelling to zero is rare, and an insert is not.
pub fn add(z: &mut ZSet, row: Row, w: i128) {
    if w == 0 {
        return;
    }
    match z.get_mut(&row) {
        Some(e) => {
            *e += w;
            if *e == 0 {
                z.remove(&row);
            }
        }
        None => {
            z.insert(row, w);
        }
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
///
/// Takes a slice rather than a `Row`, so an engine folding the base can evaluate a predicate
/// against a row it holds on the stack instead of allocating a `Vec` per posting to satisfy
/// a signature. `&Vec<Value>` coerces, so every existing caller is unchanged — and the point
/// is that there is still exactly one scalar evaluator: a second one would be a second
/// three-valued logic, which is the kind of duplication that disagrees in the null cases
/// four months later.
pub fn eval_scalar(s: &Scalar, r: &[Value]) -> Result<Value, EvalError> {
    Ok(match s {
        Scalar::Column(c) => *r.get(*c as usize).unwrap_or(&Value::Null),
        Scalar::LitInt(v) => Value::Int(*v),
        Scalar::LitBool(b) => Value::Int(*b as i128),
        // The currency travels with the value (decision 8). It used to be dropped here, so
        // `sum(amt) < 0.00 eur` over usd rows compared the numbers (E30's F13).
        Scalar::LitMoney { minor, currency } => Value::Money {
            minor: *minor,
            currency: *currency,
        },
        // A string is a string (decision 6). It used to be `0`, the first currency's code
        // (E30's F11), and then was refused at lowering unless it named a currency.
        Scalar::LitText(t) => Value::text(t),
        Scalar::Anchor => Value::Int(0),
        Scalar::InCurrency { amount, currency } => {
            match (eval_scalar(amount, r)?, eval_scalar(currency, r)?) {
                (Value::Null, _) => Value::Null,
                (Value::Int(m), Value::Int(c)) => Value::Money {
                    minor: m,
                    currency: c as u32,
                },
                // Already money: its own currency stands.
                (v @ Value::Money { .. }, _) => v,
                (a, c) => {
                    return Err(EvalError::Mismatch {
                        op: "currency",
                        why: crate::value::Mismatch::Kinds(a.kind(), c.kind()),
                    })
                }
            }
        }
        Scalar::LitNull => Value::Null,
        Scalar::IsNull(inner) => Tri::of(eval_scalar(inner, r)?.is_null()).definite(),
        Scalar::Not(inner) => match truth(eval_scalar(inner, r)?) {
            Tri::Unknown => Value::Null,
            t => t.not().definite(),
        },
        Scalar::Neg(inner) => match eval_scalar(inner, r)? {
            Value::Money { minor, currency } => Value::Money {
                minor: crate::arith::neg(minor)?,
                currency,
            },
            Value::Text(_) => {
                return Err(EvalError::Mismatch {
                    op: "-",
                    why: crate::value::Mismatch::Kinds("text", "a sign"),
                })
            }
            v => arith(v, Value::Int(0), |a, _| crate::arith::neg(a))?,
        },
        Scalar::Udf { .. } => Value::Int(0),
        Scalar::Binary { op, lhs, rhs } => {
            let (a, b) = (eval_scalar(lhs, r)?, eval_scalar(rhs, r)?);
            let mismatch = |op: &'static str| move |why| EvalError::Mismatch { op, why };
            let num = |op: &'static str,
                       additive: bool,
                       f: fn(i128, i128) -> Result<i128, crate::arith::ArithError>|
             -> Result<Value, EvalError> {
                Ok(crate::value::arith_values(a, b, additive, f).map_err(mismatch(op))??)
            };
            let cmp =
                |op: &'static str, f: fn(std::cmp::Ordering) -> bool| -> Result<Value, EvalError> {
                    Ok(tri_value(
                        crate::value::compare_values(a, b, f).map_err(mismatch(op))?,
                    ))
                };
            match op {
                ScalarOp::Add => num("+", true, crate::arith::add)?,
                ScalarOp::Sub => num("-", true, crate::arith::sub)?,
                ScalarOp::Mul => num("*", false, crate::arith::mul)?,
                ScalarOp::Div => num("/", false, crate::arith::div)?,
                ScalarOp::Rem => num("%", false, crate::arith::rem)?,
                ScalarOp::Eq => cmp("=", |o| o.is_eq())?,
                ScalarOp::Ne => cmp("<>", |o| o.is_ne())?,
                ScalarOp::Lt => cmp("<", |o| o.is_lt())?,
                ScalarOp::Le => cmp("<=", |o| o.is_le())?,
                ScalarOp::Gt => cmp(">", |o| o.is_gt())?,
                ScalarOp::Ge => cmp(">=", |o| o.is_ge())?,
                ScalarOp::And => tri_value(truth(a).and(truth(b))),
                ScalarOp::Or => tri_value(truth(a).or(truth(b))),
                // SQL's `like`: `%` any run, `_` one character, no escape. Text only.
                ScalarOp::Like => match (a, b) {
                    (Value::Null, _) | (_, Value::Null) => Value::Null,
                    (Value::Text(_), Value::Text(_)) => {
                        let (s, p) = (a.as_text().expect("text"), b.as_text().expect("text"));
                        Tri::of(like(&s, &p)).definite()
                    }
                    (x, y) => {
                        return Err(EvalError::Mismatch {
                            op: "like",
                            why: crate::value::Mismatch::Kinds(x.kind(), y.kind()),
                        })
                    }
                },
            }
        }
    })
}

/// SQL `like`: `%` matches any run of characters, `_` exactly one; everything else itself.
pub fn like(s: &str, pattern: &str) -> bool {
    let s: Vec<char> = s.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    // Iterative matching with one backtrack point, linear in practice and never recursive.
    let (mut i, mut j) = (0usize, 0usize);
    let (mut star, mut mark) = (None::<usize>, 0usize);
    while i < s.len() {
        if j < p.len() && (p[j] == '_' || p[j] == s[i]) {
            i += 1;
            j += 1;
        } else if j < p.len() && p[j] == '%' {
            star = Some(j);
            mark = i;
            j += 1;
        } else if let Some(st) = star {
            j = st + 1;
            mark += 1;
            i = mark;
        } else {
            return false;
        }
    }
    while j < p.len() && p[j] == '%' {
        j += 1;
    }
    j == p.len()
}

/// A three-valued result, carried back into the value domain: unknown *is* null.
fn tri_value(t: Tri) -> Value {
    match t {
        Tri::Unknown => Value::Null,
        other => other.definite(),
    }
}

/// Whether a predicate keeps a row. Unknown discards, exactly as in `where`.
///
/// Fallible for the same reason `eval_scalar` is: a `where` clause can contain arithmetic,
/// and a predicate that answered `false` because its division failed would filter rows out
/// of an answer and report nothing — which is the defaulting defect wearing a boolean.
pub fn keeps(p: &Scalar, r: &[Value]) -> Result<bool, EvalError> {
    Ok(truth(eval_scalar(p, r)?).keeps())
}

/// The evaluator, carrying the work counter.
pub struct Eval<'a> {
    circuit: &'a Circuit,
    sources: &'a BTreeMap<String, ZSet>,
    /// Row-level operations performed. See the module docs.
    pub work: u64,
    /// The accumulator a `Delay` reads inside a running fixpoint, innermost last.
    ///
    /// A stack rather than a field, because a fixpoint may appear inside a fixpoint and the
    /// inner one's `Delay` must read the inner accumulator. One shared slot would silently
    /// give the inner recursion the outer's rows.
    fix_stack: Vec<ZSet>,
    /// Nodes whose value the caller already has.
    ///
    /// The engine's fast paths compute a keyed aggregate by folding the base directly,
    /// without ever materialising it as a Z-set. Everything *above* that aggregate —
    /// `order by`, `limit` — must still mean what this evaluator says it means, and the way
    /// to guarantee that is to let this evaluator compute it, from the value the fast path
    /// produced. Re-implementing `Limit`'s tie-breaking rule beside this one would be two
    /// semantics for one operator, which is the seam the whole design argues against.
    precomputed: &'a BTreeMap<NodeId, ZSet>,
    /// Set when a fixpoint did not converge inside its round budget. Carried rather than
    /// panicked, so a caller can report it as the diagnostic it is.
    /// **The first error this evaluation hit, whatever kind.**
    ///
    /// This slot was `non_terminating` and held only the fixpoint's. It now holds an
    /// arithmetic refusal too, because the alternative — making `node`, `aggregate` and
    /// `apply` all fallible — would thread `?` through sixteen recursive call sites to reach
    /// the same place: `try_run_node_with`, which is the only function that returns anything
    /// to a caller.
    ///
    /// **First writer wins, and the traversal continues with `Value::Null`.** Continuing is
    /// safe only because every entry point either returns this error or panics with it, so no
    /// caller ever sees the partial Z-set; the nulls exist to let the walk finish, and they
    /// are nulls rather than zeroes so that a future path which *did* leak one would leak an
    /// absence rather than a plausible balance. That is the same distinction the whole
    /// absence lattice is about, applied to this evaluator's own failure mode.
    error: Option<EvalError>,
}

/// Why an evaluation could not produce an answer.
///
/// One variant so far, and it is the one that matters: a recursion that does not converge.
/// A fixpoint that could diverge stalls an epoch, and a stalled epoch stalls the visibility
/// timeline for every reader in the system — so "ran out of rounds" is a result the caller
/// must see, never a value it can mistake for an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    NonTerminating {
        rounds: u32,
        /// The size of the accumulator at each of the last few rounds, so a reader can see
        /// whether it was still growing or oscillating.
        tail: Vec<usize>,
    },
    /// **An integer operation that has no answer.**
    ///
    /// Division or remainder by zero, or a result outside `i128`. Before this variant existed
    /// the first of those answered `Value::Int(0)` — a plausible balance, returned over the
    /// wire, to a client with no way to know the division had not happened — and the second
    /// wrapped in release and panicked in debug, so the same expression meant one thing under
    /// `cargo test` and another in a benchmark.
    ///
    /// The operands travel with the error because the row that produced them is gone by the
    /// time the refusal reaches a client, and "this query overflowed somewhere" is not a
    /// message anyone can act on.
    Arithmetic(crate::arith::ArithError),
    /// **Two values an operator cannot relate** (cycle 15, decisions 6 and 8): money in two
    /// currencies, or text against a number. Refused, as the checker refuses the same thing
    /// statically wherever it can see the currencies; this is where it cannot.
    Mismatch {
        op: &'static str,
        why: crate::value::Mismatch,
    },
    /// **A system-time read with no system time to read** (cycle 15, C15-05b). The circuit
    /// reads `R@recorded_at` and the caller supplied no such source. Answering from an
    /// empty relation would be a wrong answer that looks like an empty one.
    MissingSystemTime { relation: String },
    /// **A window over a collection that is not a bag** (cycle 15, C15-05b): a row with a
    /// negative weight has no position in a partition's order.
    Window { why: &'static str },
}

impl From<crate::arith::ArithError> for EvalError {
    fn from(e: crate::arith::ArithError) -> EvalError {
        EvalError::Arithmetic(e)
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::NonTerminating { rounds, tail } => write!(
                f,
                "the fixpoint did not converge in {rounds} rounds; the accumulator held {tail:?} rows over the last rounds"
            ),
            EvalError::Arithmetic(e) => write!(f, "{e}"),
            EvalError::Mismatch { op, why } => match why {
                crate::value::Mismatch::Currencies(a, b) => write!(
                    f,
                    "`{op}` between amounts in two currencies (catalog codes {a} and {b})"
                ),
                crate::value::Mismatch::Kinds(a, b) => {
                    write!(f, "`{op}` between {a} and {b}")
                }
            },
            EvalError::MissingSystemTime { relation } => write!(
                f,
                "the circuit reads `{relation}`, and no system-time rows were supplied for it"
            ),
            EvalError::Window { why } => write!(f, "a window function: {why}"),
        }
    }
}

impl std::error::Error for EvalError {}

/// **Whether this evaluator has an arm for `op`.**
///
/// An exhaustive match, and that is the whole mechanism: adding a variant to [`Op`] does not
/// compile until someone has said, here, whether the reference semantics can evaluate it.
/// [`crate::verify`] refuses a circuit containing an operator this returns `false` for, so
/// the answer is a *gate* rather than a fact about this file that a reader has to go and
/// check.
///
/// The audit that asked for this found three operators the evaluator answered wrongly rather
/// than not at all — `RIGHT` and `FULL` joins evaluated to nothing, `CROSS` answered the
/// equi-join — and every one of them parsed, lowered and **passed the verifier**. The
/// verifier checked types, effects, contracts and guardedness, and nothing about whether an
/// operator it was letting through had an implementation on the other side. A circuit whose
/// operator has no arm is not a slow query or a wrong plan: it is a wrong answer with a
/// clean bill of health, which is the most expensive defect this repository can ship.
pub fn implements(op: &Op) -> bool {
    match op {
        Op::Source { .. }
        | Op::Filter { .. }
        | Op::Map { .. }
        | Op::Join { .. }
        | Op::Aggregate { .. }
        | Op::Distinct
        | Op::Union
        | Op::Negate
        | Op::Apply { .. }
        | Op::Fixpoint { .. }
        | Op::Delay
        | Op::OrderBy { .. }
        | Op::Limit { .. }
        | Op::Window { .. }
        | Op::Index { .. }
        | Op::AsOf { .. }
        | Op::ValidAt { .. }
        | Op::Integrate
        | Op::Differentiate => true,
    }
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
    match try_run_node(c, id, sources) {
        Ok(v) => v,
        Err(e) => panic!("{e}"),
    }
}

/// Evaluate one node, reporting a non-terminating fixpoint rather than panicking on it.
pub fn try_run_node(
    c: &Circuit,
    id: NodeId,
    sources: &BTreeMap<String, ZSet>,
) -> Result<(ZSet, u64), EvalError> {
    try_run_node_with(c, id, sources, &BTreeMap::new())
}

/// Evaluate a named output, taking some nodes' values **as given**.
///
/// The entry point an engine uses when it has computed part of a circuit by a route this
/// evaluator does not take — a fold over the base rather than a Z-set materialisation — and
/// needs the rest of the circuit to mean exactly what the reference says it means.
pub fn try_run_with(
    c: &Circuit,
    output: &str,
    sources: &BTreeMap<String, ZSet>,
    precomputed: &BTreeMap<NodeId, ZSet>,
) -> Result<(ZSet, u64), EvalError> {
    let id = *c
        .outputs
        .get(output)
        .unwrap_or_else(|| panic!("no output named `{output}`"));
    try_run_node_with(c, id, sources, precomputed)
}

/// [`try_run_node`], with precomputed nodes.
pub fn try_run_node_with(
    c: &Circuit,
    id: NodeId,
    sources: &BTreeMap<String, ZSet>,
    precomputed: &BTreeMap<NodeId, ZSet>,
) -> Result<(ZSet, u64), EvalError> {
    let mut e = Eval {
        circuit: c,
        sources,
        work: 0,
        fix_stack: Vec::new(),
        error: None,
        precomputed,
    };
    let z = e.node(id);
    match e.error {
        Some(err) => Err(err),
        // `into_owned` at the boundary: a caller gets a Z-set it owns, and the copy happens
        // exactly once — for a circuit that is a bare source and for nothing else.
        None => Ok((z.into_owned(), e.work)),
    }
}

/// Evaluate a named output, reporting a non-terminating fixpoint.
pub fn try_run(
    c: &Circuit,
    output: &str,
    sources: &BTreeMap<String, ZSet>,
) -> Result<(ZSet, u64), EvalError> {
    let id = *c
        .outputs
        .get(output)
        .unwrap_or_else(|| panic!("no output named `{output}`"));
    try_run_node(c, id, sources)
}

impl<'a> Eval<'a> {
    /// Record an error, first writer wins.
    ///
    /// First rather than last, because the first is the one a reader can act on: a division
    /// by zero in a projection will usually be followed by more of the same on every
    /// remaining row, and reporting the last of ten thousand identical refusals names a row
    /// chosen by iteration order.
    fn note_error(&mut self, e: EvalError) {
        if self.error.is_none() {
            self.error = Some(e);
        }
    }

    /// Whether a predicate keeps a row, recording a refusal and **discarding the row**.
    ///
    /// Discarding rather than keeping, and it makes no difference to what a caller sees —
    /// the recorded error replaces the Z-set — but it makes a difference to what happens
    /// next: a kept row goes on to be projected and aggregated, so keeping it would run more
    /// arithmetic on data the query has already failed on and could turn one honest refusal
    /// into a second, later one that names the wrong operator.
    fn keeps_row(&mut self, p: &Scalar, r: &[Value]) -> bool {
        match keeps(p, r) {
            Ok(k) => k,
            Err(err) => {
                self.note_error(err);
                false
            }
        }
    }

    /// Evaluate a scalar, recording a refusal and yielding null rather than a value.
    ///
    /// The null never reaches a caller: `try_run_node_with` returns the recorded error
    /// instead of the Z-set, and `run_node` panics with it. It exists so the traversal can
    /// finish without threading `?` through sixteen recursive borrows of `Cow<ZSet>`.
    fn scalar(&mut self, e: &Scalar, r: &[Value]) -> Value {
        match eval_scalar(e, r) {
            Ok(v) => v,
            Err(err) => {
                self.note_error(err);
                Value::Null
            }
        }
    }

    /// Evaluate a node, **borrowing a base relation rather than copying it**.
    ///
    /// `Op::Source` used to `.cloned()` the whole source. On the benchmark's base that is a
    /// twenty-thousand-row `BTreeMap<Vec<Value>, i128>` rebuilt per query — 46,665
    /// allocations and 9.6MB — before any operator had looked at a row. The operators that
    /// *read* their input and never own it (`Aggregate`, `Join`, `Apply`) then dropped the
    /// copy untouched: the analytical shape `Source -> Aggregate` paid for a duplicate of
    /// the base in order to fold it.
    ///
    /// A `Cow` says which arms need ownership. The consuming ones (`Filter`, `Map`,
    /// `Distinct`, `Union`, `Negate`) call `into_owned`, which copies only when the input is
    /// borrowed — exactly what they paid before, never more. The reading ones take a
    /// reference and pay nothing.
    fn node(&mut self, id: NodeId) -> Cow<'a, ZSet> {
        // A value the caller already has costs nothing to produce and nothing to copy.
        if let Some(z) = self.precomputed.get(&id) {
            self.work += z.len() as u64;
            return Cow::Borrowed(z);
        }
        let n = self
            .circuit
            .nodes
            .iter()
            .find(|n| n.id == id)
            .expect("node");
        match &n.op {
            Op::Source { relation, .. } => match self.sources.get(relation) {
                Some(z) => {
                    self.work += z.len() as u64;
                    Cow::Borrowed(z)
                }
                None => {
                    if crate::operator::system_time_of(relation).is_some() {
                        self.note_error(EvalError::MissingSystemTime {
                            relation: relation.clone(),
                        });
                    }
                    Cow::Owned(ZSet::new())
                }
            },
            // **Filtered by reference, and a kept row cloned once.** Consuming the input
            // would copy every row of a borrowed source in order to discard most of them,
            // which is the wrong way round for the operator whose job is to discard.
            Op::Filter { predicate } => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp.iter() {
                    self.work += 1;
                    if self.keeps_row(predicate, r) {
                        add(&mut out, r.clone(), *w);
                    }
                }
                Cow::Owned(out)
            }
            // A projection builds a new row for every input row, so it never needs to own
            // the input either.
            Op::Map { exprs } => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp.iter() {
                    self.work += 1;
                    add(
                        &mut out,
                        exprs.iter().map(|e| self.scalar(e, r)).collect(),
                        *w,
                    );
                }
                Cow::Owned(out)
            }
            Op::Join {
                kind,
                left_key,
                right_key,
                residual,
            } => {
                let l = self.node(n.inputs[0]);
                let r = self.node(n.inputs[1]);
                Cow::Owned(self.join(*kind, left_key, right_key, residual.as_ref(), &l, &r))
            }
            // **The arm the copy was for.** An aggregate reads its input and keeps none of
            // it, so over a base source this is now a fold with no duplicate at all.
            Op::Aggregate { group_key, aggs } => {
                let inp = self.node(n.inputs[0]);
                Cow::Owned(self.aggregate(group_key, aggs, &inp))
            }
            Op::Distinct => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp.iter() {
                    self.work += 1;
                    if *w > 0 {
                        add(&mut out, r.clone(), 1);
                    }
                }
                Cow::Owned(out)
            }
            Op::Union => {
                let mut out = ZSet::new();
                for k in 0..n.inputs.len() {
                    let z = self.node(n.inputs[k]);
                    for (r, w) in z.iter() {
                        self.work += 1;
                        add(&mut out, r.clone(), *w);
                    }
                }
                Cow::Owned(out)
            }
            Op::Negate => {
                let inp = self.node(n.inputs[0]);
                let mut out = ZSet::new();
                for (r, w) in inp.iter() {
                    self.work += 1;
                    add(&mut out, r.clone(), -*w);
                }
                Cow::Owned(out)
            }
            Op::Apply { kind, correlation } => {
                let outer = self.node(n.inputs[0]);
                let inner = self.node(n.inputs[1]);
                Cow::Owned(self.apply(kind, correlation, &outer, &inner))
            }
            // Pass-throughs at this level of abstraction: the reference semantics is about
            // *what* a circuit denotes, and these operators change when or how it is
            // computed rather than what comes out.
            // **Guarded recursion, evaluated.** This arm used to be absent, so the
            // reference evaluator panicked on `Fixpoint` — which meant C6(b)'s
            // "fixpoint completeness" claim had no runnable witness at all.
            //
            // The rule is the least-fixpoint one: start from the seed, apply the step to
            // the accumulator, add what it produced, and stop when nothing new arrives.
            // `Op::Distinct` semantics are used for the accumulator (weights clamped),
            // because a transitive closure is a *set* of reachable pairs and a Z-set that
            // kept multiplicities would grow without bound on a cyclic graph and never
            // converge — the recursion would be non-terminating for a reason that has
            // nothing to do with the query.
            Op::Fixpoint {
                measure,
                max_rounds,
            } => {
                let seed = self.node(n.inputs[0]);
                let mut acc: ZSet = seed
                    .as_ref()
                    .iter()
                    .filter(|(_, w)| **w > 0)
                    .map(|(r, _)| (r.clone(), 1))
                    .collect();
                let mut sizes = Vec::new();
                let mut converged = false;
                for _ in 0..*max_rounds {
                    self.fix_stack.push(acc.clone());
                    let produced = self.node(n.inputs[1]).into_owned();
                    self.fix_stack.pop();
                    let before = acc.len();
                    for (r, w) in &produced {
                        self.work += 1;
                        if *w > 0 {
                            acc.entry(r.clone()).or_insert(1);
                        }
                    }
                    sizes.push(acc.len());
                    if acc.len() == before {
                        converged = true;
                        break;
                    }
                }
                if !converged {
                    // The measure is carried in the IR precisely so a reader can be told
                    // *what* was supposed to decrease. It is reported rather than
                    // evaluated: the guard is a static obligation, and an evaluator that
                    // re-derived it would be checking the wrong thing here.
                    let _ = measure;
                    let tail = sizes.split_off(sizes.len().saturating_sub(4));
                    self.note_error(EvalError::NonTerminating {
                        rounds: *max_rounds,
                        tail,
                    });
                }
                Cow::Owned(acc)
            }
            // Inside a running fixpoint, the delay *is* the accumulator: it is the back
            // edge, and reading it is how the step sees what the previous round produced.
            // Outside one it is the identity, which is what it means at the top level of a
            // circuit with no cycle.
            // Owned rather than borrowed: the accumulator lives in `self`, not in the
            // sources, so a reference to it would borrow the evaluator for the rest of the
            // round. One copy per fixpoint round, which is what it was before.
            Op::Delay => match self.fix_stack.last() {
                Some(acc) => {
                    self.work += acc.len() as u64;
                    Cow::Owned(acc.clone())
                }
                None => self.node(n.inputs[0]),
            },
            // **Ordering is not part of a Z-set's denotation.** A Z-set is a map from rows
            // to weights; it has no order to change. `OrderBy` is therefore the identity
            // here, and that is a statement about the semantics rather than an omission:
            // what `order by` decides is how a *result set* is presented, which is a
            // question one level below this one.
            Op::OrderBy { .. } => self.node(n.inputs[0]),
            Op::Window {
                partition,
                order,
                func,
            } => {
                let inp = self.node(n.inputs[0]);
                Cow::Owned(self.window(partition, order, func, &inp))
            }

            // **`limit`, and the choice it forces.** "The first n rows" of an unordered
            // collection is not a denotation, and in SQL a `LIMIT` without an `ORDER BY`
            // genuinely has no defined answer. The reference semantics has to pick one or
            // refuse the operator, and it picks — because a fragment that cannot say what
            // `limit 2` means cannot claim `limit` at all.
            //
            // The choice, stated so an engine can be held to it: rows are taken in the
            // order given by the nearest upstream `OrderBy`'s keys, and where there is
            // none, in the rows' own lexicographic order. Multiplicities are consumed one
            // at a time, so `limit 2` over a row of weight 3 yields that row twice.
            Op::Limit { count, offset } => {
                let inp = self.node(n.inputs[0]);
                let keys = self.upstream_order(n.inputs[0]);
                // **Bounded selection, not a full sort.**
                //
                // `limit 10` over a `group by acct` sorted ten thousand rows to keep ten,
                // and cloned every one of them into a vector first. The rows here are
                // borrowed and only the prefix that can reach the answer is ordered.
                //
                // The bound is `offset + count` **rows**, and it is safe because every row
                // in a Z-set is distinct and carries a positive weight, so it fills at least
                // one of the slots the limit has to give away. A row outside the first
                // `offset + count` in the ordering cannot reach the output however large the
                // weights ahead of it are.
                //
                // This produces the same answer as the sort it replaces and not merely a
                // similar one: `order_rows` falls back to comparing the rows themselves, so
                // it is a *total* order over distinct rows and the prefix is unique. There
                // are no ties to break arbitrarily, which is exactly why the fallback is
                // there.
                let mut rows: Vec<(&Row, i128)> = inp
                    .iter()
                    .filter(|(_, w)| **w > 0)
                    .map(|(r, w)| (r, *w))
                    .collect();
                let want = (offset.saturating_add(*count)).min(rows.len() as u64) as usize;
                if want < rows.len() {
                    rows.select_nth_unstable_by(want, |a, b| order_rows(a.0, b.0, &keys));
                    rows.truncate(want);
                }
                rows.sort_unstable_by(|a, b| order_rows(a.0, b.0, &keys));
                let mut out = ZSet::new();
                let (mut skipped, mut taken) = (0u64, 0u64);
                for (r, w) in rows {
                    for _ in 0..w {
                        self.work += 1;
                        if skipped < *offset {
                            skipped += 1;
                            continue;
                        }
                        if taken >= *count {
                            return Cow::Owned(out);
                        }
                        add(&mut out, r.clone(), 1);
                        taken += 1;
                    }
                }
                Cow::Owned(out)
            }
            Op::Index { .. }
            | Op::AsOf { .. }
            | Op::ValidAt { .. }
            | Op::Integrate
            | Op::Differentiate => self.node(n.inputs[0]),
        }
    }

    /// The ordering keys of the nearest upstream `OrderBy`, if there is one.
    ///
    /// Only through operators that neither reorder nor reshape rows, which is why the walk
    /// stops at anything else: an `order by` on the far side of an aggregate is an ordering
    /// of a different relation.
    fn upstream_order(&self, mut id: NodeId) -> Vec<(ColIdx, bool)> {
        loop {
            let Some(n) = self.circuit.nodes.iter().find(|n| n.id == id) else {
                return Vec::new();
            };
            match &n.op {
                Op::OrderBy { keys } => return keys.clone(),
                Op::Filter { .. } | Op::Distinct | Op::AsOf { .. } | Op::ValidAt { .. } => {
                    match n.inputs.first() {
                        Some(i) => id = *i,
                        None => return Vec::new(),
                    }
                }
                _ => return Vec::new(),
            }
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

        // For the right-preserving kinds: which right rows found a partner. A right row is
        // padded exactly when nothing on the left matched it, and "matched" has to mean the
        // same thing on both sides — the residual included — or a `full` join would emit a
        // row twice, once padded and once joined.
        let mut matched_right: BTreeMap<&Row, bool> = BTreeMap::new();
        let preserves_right = matches!(kind, JoinKind::RightOuter | JoinKind::FullOuter);
        if preserves_right {
            for rrow in r.keys() {
                matched_right.insert(rrow, false);
            }
        }
        let width_l = l.keys().next().map(|x| x.len()).unwrap_or(0);

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
                    if !self.keeps_row(res, &combined) {
                        continue;
                    }
                }
                total += rw;
                any = true;
                if preserves_right {
                    matched_right.insert(*rrow, true);
                }
                if matches!(
                    kind,
                    JoinKind::Inner
                        | JoinKind::LeftOuter
                        | JoinKind::RightOuter
                        | JoinKind::FullOuter
                ) {
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
                JoinKind::LeftOuter | JoinKind::FullOuter if !any => {
                    let mut combined = lrow.clone();
                    combined.extend(std::iter::repeat_n(Value::Null, width_r));
                    add(&mut out, combined, *lw);
                }
                _ => {}
            }
        }

        // **The right-preserving pass.** Without it `RightOuter` and `FullOuter` were
        // *accepted by the parser, lowered, verified, and evaluated to the wrong answer* —
        // a right join returned nothing at all, because the matched pairs were emitted only
        // for the two kinds named in the loop above and the unmatched right rows were
        // emitted for none. Silence rather than a refusal, in the oracle that Theorem
        // 4.6(c)'s golden cases compare against, and the corpus had no case for either kind
        // so nothing said so.
        if preserves_right {
            for (rrow, rw) in r {
                self.work += 1;
                if matched_right.get(rrow).copied().unwrap_or(false) {
                    continue;
                }
                let mut combined: Row = std::iter::repeat_n(Value::Null, width_l).collect();
                combined.extend(rrow.iter().copied());
                add(&mut out, combined, *rw);
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
                slot[i].push((self.scalar(e, r), *w));
            }
        }
        let mut out = ZSet::new();
        for (k, per_agg) in raw {
            self.work += 1;
            let mut row_out = k;
            for (i, (a, _)) in aggs.iter().enumerate() {
                let v = match try_fold(*a, &per_agg[i]) {
                    Ok(v) => v,
                    Err(e) => {
                        self.note_error(e);
                        Value::Null
                    }
                };
                row_out.push(v);
            }
            add(&mut out, row_out, 1);
        }
        out
    }

    /// **A window function, by its definition.** Each partition's rows are put in order
    /// (the `OrderBy` order: the keys, then the row), expanded by weight — a row of weight
    /// `w` is `w` identical rows — and split into peer groups, rows equal on the ordering
    /// keys. Every row of a peer group sees the same frame: the partition's rows up to the
    /// end of its group, or the whole partition when there is no ordering.
    fn window(
        &mut self,
        partition: &[ColIdx],
        order: &[(ColIdx, bool)],
        func: &crate::operator::WindowFn,
        inp: &ZSet,
    ) -> ZSet {
        use crate::operator::WindowFn;
        let mut parts: BTreeMap<Row, Vec<(&Row, i128)>> = BTreeMap::new();
        for (r, w) in inp {
            self.work += 1;
            if *w < 0 {
                self.note_error(EvalError::Window {
                    why: "a row with a negative weight has no place in a partition's order",
                });
                return ZSet::new();
            }
            if *w == 0 {
                continue;
            }
            let k: Row = partition
                .iter()
                .map(|c| *r.get(*c as usize).unwrap_or(&Value::Null))
                .collect();
            parts.entry(k).or_default().push((r, *w));
        }
        let peers = |a: &Row, b: &Row| {
            order.iter().all(|(c, _)| {
                a.get(*c as usize).copied().unwrap_or(Value::Null)
                    == b.get(*c as usize).copied().unwrap_or(Value::Null)
            })
        };
        let mut out = ZSet::new();
        for (_, mut rows) in parts {
            rows.sort_by(|a, b| order_rows(a.0, b.0, order));
            // Peer groups, as index ranges into `rows`. With no ordering the whole partition
            // is one group: every row is every other row's peer.
            let groups: Vec<(usize, usize)> = if order.is_empty() {
                vec![(0, rows.len())]
            } else {
                let mut g = Vec::new();
                let mut start = 0;
                for i in 1..=rows.len() {
                    if i == rows.len() || !peers(rows[start].0, rows[i].0) {
                        g.push((start, i));
                        start = i;
                    }
                }
                g
            };
            let mut before: i128 = 0; // rows (with multiplicity) before the current group
            let mut frame: Vec<(Value, i128)> = Vec::new();
            for (g, (lo, hi)) in groups.iter().enumerate() {
                self.work += (hi - lo) as u64;
                let in_group: i128 = rows[*lo..*hi].iter().map(|(_, w)| *w).sum();
                match func {
                    WindowFn::RowNumber => {
                        let mut n = before;
                        for (r, w) in &rows[*lo..*hi] {
                            for _ in 0..*w {
                                n += 1;
                                let mut row = (*r).clone();
                                row.push(Value::Int(n));
                                add(&mut out, row, 1);
                            }
                        }
                    }
                    WindowFn::Rank | WindowFn::DenseRank => {
                        let v = if matches!(func, WindowFn::Rank) {
                            before + 1
                        } else {
                            g as i128 + 1
                        };
                        for (r, w) in &rows[*lo..*hi] {
                            let mut row = (*r).clone();
                            row.push(Value::Int(v));
                            add(&mut out, row, *w);
                        }
                    }
                    WindowFn::Running(a, e) => {
                        for (r, w) in &rows[*lo..*hi] {
                            frame.push((self.scalar(e, r), *w));
                        }
                        let v = match try_fold(*a, &frame) {
                            Ok(v) => v,
                            Err(err) => {
                                self.note_error(err);
                                Value::Null
                            }
                        };
                        for (r, w) in &rows[*lo..*hi] {
                            let mut row = (*r).clone();
                            row.push(v);
                            add(&mut out, row, *w);
                        }
                    }
                }
                before += in_group;
            }
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
                        .map(|(r, w)| (eval_scalar(expr, r).unwrap_or(Value::Null), *w))
                        .collect();
                    // An empty matching set yields null — SQL's rule for a scalar
                    // subquery, and the reason the unnested form is a *left outer* join.
                    let v = if vals.is_empty() {
                        Value::Null
                    } else {
                        match try_fold(*agg, &vals) {
                            Ok(v) => v,
                            Err(e) => {
                                self.note_error(e);
                                Value::Null
                            }
                        }
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
    try_fold(a, vals).unwrap_or_else(|e| panic!("{e}"))
}

/// [`fold`], refusing what has no answer (cycle 15, decision 8): a `sum`, `min`, `max` or
/// `avg` over money in more than one currency, or over text for `sum`/`avg`. Money in one
/// currency folds to money in that currency; `min`/`max` of text are by content.
pub fn try_fold(a: Agg, vals: &[(Value, i128)]) -> Result<Value, EvalError> {
    let op = match a {
        Agg::Count => return Ok(Value::Int(vals.iter().map(|(_, w)| *w).sum())),
        Agg::Sum => "sum",
        Agg::Min => "min",
        Agg::Max => "max",
        Agg::Avg => "avg",
    };
    // One currency across every money contribution, and no text where a number is summed.
    let mut currency: Option<u32> = None;
    for (v, _) in vals {
        match v {
            Value::Money { currency: c, .. } => match currency {
                None => currency = Some(*c),
                Some(k) if k != *c => {
                    return Err(EvalError::Mismatch {
                        op,
                        why: crate::value::Mismatch::Currencies(k, *c),
                    })
                }
                _ => {}
            },
            Value::Text(_) if matches!(a, Agg::Sum | Agg::Avg) => {
                return Err(EvalError::Mismatch {
                    op,
                    why: crate::value::Mismatch::Kinds("text", "a sum"),
                })
            }
            _ => {}
        }
    }
    if matches!(a, Agg::Min | Agg::Max)
        && vals
            .iter()
            .any(|(v, w)| *w > 0 && matches!(v, Value::Text(_)))
    {
        let live = vals
            .iter()
            .filter(|(v, w)| *w > 0 && !v.is_null())
            .map(|(v, _)| *v);
        return Ok(if a == Agg::Min {
            live.min()
        } else {
            live.max()
        }
        .unwrap_or(Value::Null));
    }
    let tag = |v: Value| match (v, currency) {
        (Value::Int(m), Some(c)) => Value::Money {
            minor: m,
            currency: c,
        },
        (v, _) => v,
    };
    Ok(tag(fold_numbers(a, vals)))
}

fn fold_numbers(a: Agg, vals: &[(Value, i128)]) -> Value {
    match a {
        Agg::Count => Value::Int(vals.iter().map(|(_, w)| *w).sum()),
        Agg::Sum => {
            let mut acc = 0i128;
            let mut any = false;
            for (v, w) in vals {
                if let Some(x) = v.int() {
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

/// Compare two rows by an ordering key, falling back to the whole row.
///
/// The fallback is what makes `limit` deterministic where the query does not say. It is a
/// *choice*, and it is written down here rather than left to whatever order a hash map
/// happened to produce — an engine that answered `limit 2` differently from this would be
/// disagreeing with the reference semantics, which is a thing a test can catch.
/// **The order a result set is presented in**, if its query asked for one.
///
/// Ordering is not part of a Z-set's denotation (see `Op::OrderBy` in [`Evaluator`]), so
/// the answer the engine returns is a map with no order; `order by` decides how the *result
/// set* is presented, one level below. That level is the wire, and it needs the keys: the
/// ones of the nearest `OrderBy` beneath the output, reached only through operators that
/// neither reorder nor reshape rows — the same walk `limit` uses. Empty when there is none,
/// or when a projection or anything else stands between, because keys that index the
/// `OrderBy`'s input do not index a reshaped output.
///
/// Until cycle 14 nothing called this and the wire sent every reply in the rows' own
/// lexicographic order, so `order by sum(amt) desc limit 10` returned the right ten rows in
/// ascending account order — found by E27's correctness pass (R2-02).
pub fn presentation_order(c: &Circuit, output: &str) -> Vec<(ColIdx, bool)> {
    let Some(mut id) = c.outputs.get(output).copied() else {
        return Vec::new();
    };
    loop {
        let Some(n) = c.nodes.iter().find(|n| n.id == id) else {
            return Vec::new();
        };
        match &n.op {
            Op::OrderBy { keys } => return keys.clone(),
            Op::Limit { .. }
            | Op::Filter { .. }
            | Op::Distinct
            | Op::AsOf { .. }
            | Op::ValidAt { .. } => match n.inputs.first() {
                Some(i) => id = *i,
                None => return Vec::new(),
            },
            _ => return Vec::new(),
        }
    }
}

/// The comparison [`presentation_order`]'s keys mean — the one `limit` selects by, so the
/// rows a limit kept and the order they are shown in cannot disagree.
pub fn presented_cmp(a: &Row, b: &Row, keys: &[(ColIdx, bool)]) -> std::cmp::Ordering {
    order_rows(a, b, keys)
}

fn order_rows(a: &Row, b: &Row, keys: &[(ColIdx, bool)]) -> std::cmp::Ordering {
    for (c, asc) in keys {
        let (x, y) = (
            a.get(*c as usize).copied().unwrap_or(Value::Null),
            b.get(*c as usize).copied().unwrap_or(Value::Null),
        );
        let ord = if *asc { x.cmp(&y) } else { y.cmp(&x) };
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    a.cmp(b)
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
                confidential: Vec::new(),
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

    /// **The bounded `limit` is the full sort, on every case that could tell them apart.**
    ///
    /// T-06 replaced "clone every row, sort all of them, keep n" with a bounded selection.
    /// That is only a performance change if the answer is identical, and the reference
    /// evaluator cannot be its own judge here — comparing it against itself would compare the
    /// new implementation with the new implementation. So the old one is written out again in
    /// this test and the two are compared over cases chosen for where a selection algorithm
    /// differs from a sort: equal ordering keys resolved by the row fallback, negative values,
    /// weights above one that consume more slots than they occupy rows, an `offset` that
    /// starts inside a repeated row, and limits at and beyond the end.
    #[test]
    fn a_bounded_limit_answers_exactly_what_a_full_sort_would() {
        /// The implementation this replaced: clone everything, sort everything, take n.
        fn by_full_sort(z: &ZSet, keys: &[(ColIdx, bool)], count: u64, offset: u64) -> ZSet {
            let mut rows: Vec<(Row, i128)> = z
                .iter()
                .filter(|(_, w)| **w > 0)
                .map(|(r, w)| (r.clone(), *w))
                .collect();
            rows.sort_by(|a, b| order_rows(&a.0, &b.0, keys));
            let mut out = ZSet::new();
            let (mut skipped, mut taken) = (0u64, 0u64);
            for (r, w) in rows {
                for _ in 0..w {
                    if skipped < offset {
                        skipped += 1;
                        continue;
                    }
                    if taken >= count {
                        return out;
                    }
                    add(&mut out, r.clone(), 1);
                    taken += 1;
                }
            }
            out
        }

        let v = |a: i128, b: i128| vec![Value::Int(a), Value::Int(b)];
        // Sums that tie (three rows at 50), a negative, a null in the ordered column, and
        // weights above one.
        let mut z = ZSet::new();
        for (row, w) in [
            (v(1, 50), 1i128),
            (v(2, 50), 3),
            (v(3, 50), 1),
            (v(4, -20), 2),
            (v(5, 100), 1),
            (v(6, 0), 1),
            (vec![Value::Int(7), Value::Null], 1),
        ] {
            z.insert(row, w);
        }

        for keys in [
            vec![],
            vec![(1u16, false)],
            vec![(1u16, true)],
            vec![(1u16, false), (0u16, true)],
            vec![(0u16, true)],
        ] {
            for count in [0u64, 1, 2, 3, 5, 10, 100] {
                for offset in [0u64, 1, 3, 4, 20] {
                    let mut c = Circuit::new();
                    let s = src(&mut c, "t");
                    let o = c.add(
                        Op::OrderBy { keys: keys.clone() },
                        vec![s],
                        internal_contract(),
                        "ord",
                    );
                    let l = c.add(
                        Op::Limit { count, offset },
                        vec![o],
                        internal_contract(),
                        "lim",
                    );
                    c.outputs.insert("out".into(), l);
                    let sources = BTreeMap::from([("t".to_string(), z.clone())]);
                    let (got, _) = run(&c, "out", &sources);
                    assert_eq!(
                        got,
                        by_full_sort(&z, &keys, count, offset),
                        "keys {keys:?}, limit {count} offset {offset}"
                    );
                }
            }
        }
    }

    #[test]
    fn like_matches_sql_patterns() {
        assert!(like("alice", "al%"));
        assert!(like("alice", "%ice"));
        assert!(like("bob", "_ob"));
        assert!(like("a%b", "a%b"));
        assert!(!like("alice", "_ob"));
        assert!(!like("al", "al_"));
        assert!(like("", "%"));
    }

    #[test]
    fn a_sum_over_two_currencies_is_refused_and_one_keeps_its_currency() {
        let m = |minor, currency| Value::Money { minor, currency };
        assert_eq!(
            try_fold(Agg::Sum, &[(m(3, 0), 1), (m(4, 0), 2)]),
            Ok(m(11, 0))
        );
        assert!(matches!(
            try_fold(Agg::Sum, &[(m(3, 0), 1), (m(4, 1), 1)]),
            Err(EvalError::Mismatch { .. })
        ));
        assert_eq!(
            try_fold(Agg::Max, &[(Value::text("x"), 1), (Value::text("y"), 1)]),
            Ok(Value::text("y"))
        );
    }
}

/// **Window functions, by their definition** (cycle 15, C15-05b; the author's decision 4).
#[cfg(test)]
mod window {
    use super::*;
    use crate::circuit::internal_contract;
    use crate::operator::WindowFn;

    /// `rows(part, ord, v)` over one source, with `func` appended.
    fn run(
        rows: &[(&[i128], i128)],
        order: Vec<(ColIdx, bool)>,
        func: WindowFn,
    ) -> Result<Vec<String>, EvalError> {
        let z = eval(rows, order, func)?;
        let mut out: Vec<String> = z
            .iter()
            .map(|(r, w)| {
                let cells: Vec<String> = r.iter().map(|v| v.to_string()).collect();
                format!("{} x{w}", cells.join(" "))
            })
            .collect();
        out.sort();
        Ok(out)
    }

    fn eval(
        rows: &[(&[i128], i128)],
        order: Vec<(ColIdx, bool)>,
        func: WindowFn,
    ) -> Result<ZSet, EvalError> {
        let mut c = Circuit::new();
        let s = c.add(
            Op::Source {
                relation: "r".into(),
                is_base: true,
                anchor_key: vec![],
                confidential: Vec::new(),
            },
            vec![],
            internal_contract(),
            "r",
        );
        let w = c.add(
            Op::Window {
                partition: vec![0],
                order,
                func,
            },
            vec![s],
            internal_contract(),
            "window",
        );
        c.outputs.insert("v".into(), w);
        let mut src = BTreeMap::new();
        src.insert("r".to_string(), zset(rows));
        Ok(try_run(&c, "v", &src)?.0)
    }

    // Partition 1: ord 10 (twice, as one row of weight 2), 20, 20 (a second row, a tie on
    // `ord` with a different `v`), 30. Partition 2: one row.
    const ROWS: &[(&[i128], i128)] = &[
        (&[1, 10, 5], 2),
        (&[1, 20, 7], 1),
        (&[1, 20, 8], 1),
        (&[1, 30, 1], 1),
        (&[2, 10, 100], 1),
    ];

    #[test]
    fn a_running_sum_includes_the_current_rows_peers() {
        // 10: 5+5 = 10; 20: 10+7+8 = 25 for both tied rows; 30: 26.
        assert_eq!(
            run(
                ROWS,
                vec![(1, true)],
                WindowFn::Running(Agg::Sum, Scalar::Column(2))
            )
            .unwrap(),
            [
                "1 10 5 10 x2",
                "1 20 7 25 x1",
                "1 20 8 25 x1",
                "1 30 1 26 x1",
                "2 10 100 100 x1"
            ]
        );
    }

    #[test]
    fn without_an_order_the_frame_is_the_whole_partition() {
        assert_eq!(
            run(
                ROWS,
                vec![],
                WindowFn::Running(Agg::Count, Scalar::Column(2))
            )
            .unwrap(),
            [
                "1 10 5 5 x2",
                "1 20 7 5 x1",
                "1 20 8 5 x1",
                "1 30 1 5 x1",
                "2 10 100 1 x1"
            ]
        );
    }

    #[test]
    fn rank_skips_after_a_tie_and_dense_rank_does_not() {
        assert_eq!(
            run(ROWS, vec![(1, true)], WindowFn::Rank).unwrap(),
            [
                "1 10 5 1 x2",
                "1 20 7 3 x1",
                "1 20 8 3 x1",
                "1 30 1 5 x1",
                "2 10 100 1 x1"
            ]
        );
        assert_eq!(
            run(ROWS, vec![(1, true)], WindowFn::DenseRank).unwrap(),
            [
                "1 10 5 1 x2",
                "1 20 7 2 x1",
                "1 20 8 2 x1",
                "1 30 1 3 x1",
                "2 10 100 1 x1"
            ]
        );
        // Descending: 30 first.
        assert_eq!(
            run(ROWS, vec![(1, false)], WindowFn::Rank).unwrap(),
            [
                "1 10 5 4 x2",
                "1 20 7 2 x1",
                "1 20 8 2 x1",
                "1 30 1 1 x1",
                "2 10 100 1 x1"
            ]
        );
    }

    #[test]
    fn row_number_numbers_each_copy_of_a_repeated_row() {
        assert_eq!(
            run(ROWS, vec![(1, true)], WindowFn::RowNumber).unwrap(),
            [
                "1 10 5 1 x1",
                "1 10 5 2 x1",
                "1 20 7 3 x1",
                "1 20 8 4 x1",
                "1 30 1 5 x1",
                "2 10 100 1 x1"
            ]
        );
    }

    #[test]
    fn a_negative_weight_is_refused() {
        match run(&[(&[1, 10, 5], -1)], vec![(1, true)], WindowFn::Rank) {
            Err(EvalError::Window { .. }) => {}
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[test]
    fn a_running_sum_of_money_keeps_its_currency_and_refuses_two() {
        let money = |currency: u32| Scalar::InCurrency {
            amount: Box::new(Scalar::Column(2)),
            currency: Box::new(Scalar::LitInt(currency as i128)),
        };
        let one = eval(ROWS, vec![(1, true)], WindowFn::Running(Agg::Sum, money(0))).unwrap();
        let last = |r: &Row| r.last().copied();
        assert!(
            one.keys().any(|r| last(r)
                == Some(Value::Money {
                    minor: 26,
                    currency: 0
                })),
            "the running sum is money in currency 0: {one:?}"
        );
        // Two currencies in one partition: the currency is column 1 here, 10/20/30.
        let two = Scalar::InCurrency {
            amount: Box::new(Scalar::Column(2)),
            currency: Box::new(Scalar::Column(1)),
        };
        match run(ROWS, vec![(1, true)], WindowFn::Running(Agg::Sum, two)) {
            Err(EvalError::Mismatch { .. }) => {}
            other => panic!("expected a currency mismatch, got {other:?}"),
        }
    }

    #[test]
    fn a_system_time_read_with_nothing_supplied_is_refused() {
        let mut c = Circuit::new();
        let s = c.add(
            Op::Source {
                relation: crate::operator::system_time_relation("postings"),
                is_base: true,
                anchor_key: vec![],
                confidential: Vec::new(),
            },
            vec![],
            internal_contract(),
            "postings",
        );
        c.outputs.insert("v".into(), s);
        match try_run(&c, "v", &BTreeMap::new()) {
            Err(EvalError::MissingSystemTime { .. }) => {}
            other => panic!("expected a refusal, got {:?}", other.map(|(z, _)| z.len())),
        }
    }
}
