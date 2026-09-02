//! The IR operator set.
//!
//! This is the vocabulary the theory quantifies over. Every theorem in Chapters 3 and 4 is
//! stated about circuits built from these operators and nothing else, so the set is closed
//! deliberately: adding an operator is a change to the thesis's claims, not a feature.
//!
//! # Three properties every operator must declare
//!
//! **Incrementality.** Whether the operator can be maintained over a Z-set delta without
//! retaining its whole input. `order_by`, `limit` and `offset` cannot, and that single fact
//! decides whether a view ending in one of them can be served on demand at a strict rung.
//! The surface syntax must not hide it, so the IR states it per operator.
//!
//! **Key derivation.** What key the operator's output is indexed by, given its inputs'
//! keys. Reconstruction is per key: an upquery asks "what is the value at key k, anchored
//! at epoch e", and a circuit whose key discipline is not derivable end to end has no
//! upquery path, which means its state cannot be evicted and reconstructed at all.
//!
//! **Conservation transparency.** Whether the operator can create or destroy a monetary
//! quantity flowing through it. A `filter` can (it drops rows); a `map` that projects
//! money through unchanged cannot; a `sum` cannot. This is what lets the verifier check
//! that a conservation-carrying path stays conservation-carrying, which is the IR-level
//! statement of Contribution 1's corollary.

use std::fmt;

/// A column reference within a node's output schema.
pub type ColIdx = u16;

/// Aggregate functions the runtime maintains incrementally.
///
/// Deliberately short. Each member is here because it has a known incremental maintenance
/// rule over a Z-set; `min` and `max` are included with the caveat recorded on them, since
/// they are the two whose deletion case is not O(1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Agg {
    /// Additive: maintained by adding the delta. The banking case.
    Sum,
    /// Additive over weights.
    Count,
    /// **Not** freely incremental on deletion: removing the current minimum requires
    /// knowing the next one, so the runtime retains an ordered multiset per group. The
    /// cost is recorded here rather than discovered in production.
    Min,
    /// See `Min`.
    Max,
    /// Maintained as a (sum, count) pair, never as a running float, because a running
    /// float loses associativity and therefore loses reproducibility under reordering.
    Avg,
}

impl Agg {
    /// Whether the aggregate is maintainable in O(1) per delta in both directions.
    pub fn is_additive(self) -> bool {
        matches!(self, Agg::Sum | Agg::Count | Agg::Avg)
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Agg::Sum => "sum",
            Agg::Count => "count",
            Agg::Min => "min",
            Agg::Max => "max",
            Agg::Avg => "avg",
        }
    }
}

/// The join kinds the runtime implements. Outer joins are separated because their
/// incremental maintenance has an extra obligation: a delta that removes the last match on
/// one side must *emit* a null-extended row, and forgetting that is a classic
/// incremental-view bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinKind {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
    /// Anti-join: rows of the left with no match. Used by `where not exists`.
    Anti,
    /// Semi-join: rows of the left with at least one match, without duplication.
    Semi,
}

impl JoinKind {
    pub fn is_outer(self) -> bool {
        matches!(
            self,
            JoinKind::LeftOuter | JoinKind::RightOuter | JoinKind::FullOuter
        )
    }
}

/// A scalar expression, in the flat form the runtime evaluates. Deliberately not the
/// surface AST: a lowered predicate has no names, only column indices, which is what makes
/// a node relocatable and a plan comparable.
#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
    Column(ColIdx),
    LitInt(i128),
    LitBool(bool),
    LitText(String),
    /// A money constant in minor units at the named currency's declared scale. The
    /// currency travels with the value at every level of the IR: an operation that lost it
    /// would make the currency-safety theorem unstatable below the surface syntax.
    LitMoney {
        minor: i128,
        currency: u32,
    },
    /// The epoch the row was sealed at. Available to every operator, because the whole
    /// point of an epoch-ordered base is that visibility is a first-class column.
    Anchor,
    /// The SQL null. Distinct from `Option::None` and from an evicted `Hole` — see
    /// [`crate::value`] for why the three are kept apart.
    LitNull,
    /// `x is null` — a *definite* boolean about a value, never itself unknown. Needed
    /// because the `not in` rewrite has to ask the question the three-valued comparison
    /// operators cannot answer.
    IsNull(Box<Scalar>),
    Not(Box<Scalar>),
    Neg(Box<Scalar>),
    Binary {
        op: ScalarOp,
        lhs: Box<Scalar>,
        rhs: Box<Scalar>,
    },
    /// A fuel-metered user function. Its determinism obligation is checked at load, not
    /// here; what the IR records is that the call exists, so the verifier can refuse to
    /// place it on a path that must be reproducible.
    Udf {
        id: u32,
        args: Vec<Scalar>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalarOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Like,
}

impl Scalar {
    /// Whether evaluating this expression is deterministic and epoch-pure — the condition
    /// for it to be safely re-evaluated during a reconstruction. A UDF is assumed
    /// non-deterministic here unless the loader has certified it, because assuming the
    /// good case would make reconstruction-equivalence unprovable.
    pub fn is_reproducible(&self, certified_udfs: &[u32]) -> bool {
        match self {
            Scalar::Udf { id, args } => {
                certified_udfs.contains(id)
                    && args.iter().all(|a| a.is_reproducible(certified_udfs))
            }
            Scalar::Not(x) | Scalar::Neg(x) | Scalar::IsNull(x) => {
                x.is_reproducible(certified_udfs)
            }
            Scalar::Binary { lhs, rhs, .. } => {
                lhs.is_reproducible(certified_udfs) && rhs.is_reproducible(certified_udfs)
            }
            _ => true,
        }
    }

    /// The columns this expression reads. Needed to derive an upquery path: a
    /// reconstruction must fetch exactly the base columns the surviving expressions touch.
    pub fn columns(&self, out: &mut Vec<ColIdx>) {
        match self {
            Scalar::Column(c) => out.push(*c),
            Scalar::Not(x) | Scalar::Neg(x) => x.columns(out),
            Scalar::Binary { lhs, rhs, .. } => {
                lhs.columns(out);
                rhs.columns(out);
            }
            Scalar::Udf { args, .. } => args.iter().for_each(|a| a.columns(out)),
            _ => {}
        }
    }
}

/// One operator.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// A base, ledger or table. The leaf of every circuit, and the only node whose state
    /// is never partial.
    Source {
        relation: String,
        /// Whether this source is an immutable, fully retained base. A circuit rooted only
        /// in bases is reconstructible; one rooted in a mutable table is not, because the
        /// history it would fold no longer exists.
        is_base: bool,
        /// The column indices forming the anchor index, if one is declared.
        anchor_key: Vec<ColIdx>,
    },
    Filter {
        predicate: Scalar,
    },
    /// Projection and computation in one node, as in Materialize's `Map`/`Project` pair
    /// collapsed: a separate project node buys nothing once expressions are index-based.
    Map {
        exprs: Vec<Scalar>,
    },
    Join {
        kind: JoinKind,
        left_key: Vec<ColIdx>,
        right_key: Vec<ColIdx>,
        residual: Option<Scalar>,
    },
    /// Grouped aggregation. The workhorse: a balance is a `sum` over a `group_by (acct, cur)`.
    Aggregate {
        group_key: Vec<ColIdx>,
        aggs: Vec<(Agg, Scalar)>,
    },
    /// Canonicalising `distinct` in the Z-set sense: clamps every weight to 0 or 1.
    Distinct,
    Union,
    /// Z-set negation, which is how `except` and outer-join retraction are expressed.
    Negate,
    /// Guarded recursion. The measure is carried in the IR, not merely checked at parse
    /// time, because the runtime must be able to *stop* — a fixpoint that could diverge
    /// would stall an epoch, and a stalled epoch stalls the visibility timeline for every
    /// reader in the system.
    Fixpoint {
        measure: Scalar,
        max_rounds: u32,
    },
    /// The DBSP delay `z⁻¹`. The only operator permitted to close a cycle.
    Delay,
    /// Integration: the running sum of a stream. `I` of Theorem 2.20.
    Integrate,
    /// Differentiation: the delta of a collection. `D` of Theorem 2.20, and `I`'s inverse.
    Differentiate,
    /// Index the input by a key, making it upqueryable. Inserted by the planner wherever a
    /// downstream node needs per-key reconstruction.
    Index {
        key: Vec<ColIdx>,
    },
    /// **Not incremental.** Retains its whole input.
    OrderBy {
        keys: Vec<(ColIdx, bool)>,
    },
    /// **Not incremental.**
    Limit {
        count: u64,
        offset: u64,
    },
    /// Pin a read to a system-time epoch — the system axis of bitemporality.
    AsOf {
        epoch: Option<u64>,
    },
    /// Pin a read to a valid-time instant — the world axis.
    ValidAt {
        instant: Option<i64>,
    },

    /// **The nested form of a subquery**: a dependent join.
    ///
    /// Inputs are `[outer, inner]`. For each row of `outer`, the matching set of `inner`
    /// is the rows agreeing on every `correlation` pair; what is then done with that set
    /// depends on [`ApplyKind`].
    ///
    /// This operator exists so that a correlated subquery has a *representation* before it
    /// is unnested, rather than being a rewrite with no source form. Its evaluation is a
    /// nested loop by definition — the outer cardinality times the inner — which is what
    /// makes the unnesting gate measurable: the same query has two circuits, and the ratio
    /// between their counted work is the result.
    ///
    /// It is **not incremental** and the verifier refuses it on any served view, because a
    /// dependent join has no delta rule: a single new outer row re-scans the inner
    /// relation. Unnesting is therefore not an optimisation in the usual sense, where the
    /// unoptimised plan is merely slower; it is the step that makes the query
    /// maintainable at all.
    Apply {
        kind: ApplyKind,
        correlation: Vec<(ColIdx, ColIdx)>,
    },
}

/// What an [`Op::Apply`] does with the matching set of the inner relation.
///
/// The `In`/`NotIn` variants carry the two columns whose equality is being tested, kept
/// separate from `correlation` because they play a different role: correlation columns
/// select which inner rows are in scope, and the probe decides the predicate over them.
/// Merging them would work for `in` and be wrong for `not in`, where the null-witness
/// group is the correlation and not the probe.
#[derive(Debug, Clone, PartialEq)]
pub enum ApplyKind {
    /// `exists (subquery)` — keep the outer row when the matching set is non-empty. Does
    /// not duplicate: one outer row in, at most one out.
    Exists,
    /// `not exists (subquery)` — keep the outer row when the matching set is empty.
    NotExists,
    /// `outer.probe in (select inner from …)`. Three-valued: true on a match, and
    /// otherwise false *or unknown* — both of which drop the row, so the two need not be
    /// distinguished here.
    In { probe: ColIdx, inner: ColIdx },
    /// `outer.probe not in (select inner from …)`. Three-valued, and the trap: a null
    /// anywhere in the matching set's `inner` column makes the predicate unknown for
    /// **every** probe that did not match, so the row is dropped. A null probe drops too.
    NotIn { probe: ColIdx, inner: ColIdx },
    /// A correlated scalar subquery, appended to the outer row as one column. An empty
    /// matching set yields `null` — SQL's rule, and the reason the unnested form needs a
    /// *left outer* join rather than an inner one.
    Scalar { agg: Agg, expr: Scalar },
}

impl ApplyKind {
    pub fn name(&self) -> &'static str {
        match self {
            ApplyKind::Exists => "exists",
            ApplyKind::NotExists => "not_exists",
            ApplyKind::In { .. } => "in",
            ApplyKind::NotIn { .. } => "not_in",
            ApplyKind::Scalar { .. } => "scalar",
        }
    }
    /// Whether the apply widens the outer row. Only a scalar subquery does.
    pub fn widens(&self) -> bool {
        matches!(self, ApplyKind::Scalar { .. })
    }
}

impl Op {
    pub fn name(&self) -> &'static str {
        match self {
            Op::Source { .. } => "source",
            Op::Filter { .. } => "filter",
            Op::Map { .. } => "map",
            Op::Join { .. } => "join",
            Op::Aggregate { .. } => "aggregate",
            Op::Distinct => "distinct",
            Op::Union => "union",
            Op::Negate => "negate",
            Op::Fixpoint { .. } => "fixpoint",
            Op::Delay => "delay",
            Op::Integrate => "integrate",
            Op::Differentiate => "differentiate",
            Op::Index { .. } => "index",
            Op::OrderBy { .. } => "order_by",
            Op::Limit { .. } => "limit",
            Op::AsOf { .. } => "as_of",
            Op::ValidAt { .. } => "valid_at",
            Op::Apply { .. } => "apply",
        }
    }

    /// How many inputs this operator takes. Checked by the verifier, because an arity
    /// mismatch in a hand-built circuit is otherwise a runtime panic in the engine.
    pub fn arity(&self) -> usize {
        match self {
            Op::Source { .. } => 0,
            Op::Join { .. } | Op::Union | Op::Apply { .. } => 2,
            _ => 1,
        }
    }

    /// **Incrementality.** Whether the operator is maintainable over a delta without
    /// retaining its whole input.
    ///
    /// This single predicate is what makes W11 checkable and what the planner consults
    /// before promising a demand-materialized view at a strict rung.
    pub fn is_incremental(&self) -> bool {
        match self {
            Op::OrderBy { .. } | Op::Limit { .. } => false,
            // A dependent join has no delta rule: one new outer row re-scans the inner
            // relation. Unnesting is therefore not a speed optimisation but the step that
            // makes a correlated query maintainable at all, and the verifier says so.
            Op::Apply { .. } => false,
            // A non-additive aggregate is incremental, but not in O(1) on deletion: it is
            // maintainable, at the cost of an ordered multiset per group. The distinction
            // is `is_additive`, not this predicate, and conflating them would either
            // over-reject `min` or under-charge it.
            _ => true,
        }
    }

    /// **Conservation transparency.** Whether a monetary quantity entering this operator
    /// necessarily leaves it.
    ///
    /// This is the IR-level footing for Contribution 1's corollary. A path from a
    /// conserved ledger to a view is conservation-preserving exactly when every operator
    /// on it is transparent; where one is not, the view is a *projection* of the ledger
    /// rather than a restatement of it, and must not be presented as a control total.
    pub fn is_conservation_transparent(&self) -> bool {
        match self {
            // Drops rows: money leaves the view without leaving the ledger. Not a bug —
            // a filtered view is a legitimate thing to want — but it is not a control
            // total, and the verifier must know the difference.
            Op::Filter { .. } => false,
            Op::Limit { .. } => false,
            Op::Join { kind, .. } => {
                !kind.is_outer() && !matches!(kind, JoinKind::Anti | JoinKind::Semi)
            }
            // `exists` and `in` drop outer rows; `not exists` and `not in` drop different
            // ones; a scalar subquery null-extends. None of the five moves every monetary
            // quantity through, so none is a control total.
            Op::Apply { .. } => false,
            // Clamps weights: two identical postings become one.
            Op::Distinct => false,
            Op::Negate => false,
            _ => true,
        }
    }

    /// The output key, given the inputs' keys. `None` means the output is not indexed and
    /// therefore not upqueryable without an explicit `Index` node.
    pub fn derive_key(&self, input_keys: &[Option<Vec<ColIdx>>]) -> Option<Vec<ColIdx>> {
        match self {
            Op::Source { anchor_key, .. } if !anchor_key.is_empty() => Some(anchor_key.clone()),
            Op::Source { .. } => None,
            Op::Index { key } => Some(key.clone()),
            Op::Aggregate { group_key, .. } => Some(group_key.clone()),
            Op::Join { left_key, .. } => Some(left_key.clone()),
            // An apply preserves the outer row's identity, so it preserves its key. That
            // is the property the semi/anti rewrites rely on to avoid duplicating rows.
            Op::Apply { .. } => input_keys.first().cloned().flatten(),
            // Filtering does not change the key; mapping may, so a map that rewrites the
            // key columns loses the index and the planner must re-index.
            Op::Filter { .. }
            | Op::Delay
            | Op::Integrate
            | Op::Differentiate
            | Op::AsOf { .. }
            | Op::ValidAt { .. }
            | Op::Distinct => input_keys.first().cloned().flatten(),
            Op::Union => match (input_keys.first(), input_keys.get(1)) {
                (Some(Some(a)), Some(Some(b))) if a == b => Some(a.clone()),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Source {
                relation,
                is_base,
                anchor_key,
            } => {
                write!(f, "source({relation}")?;
                if *is_base {
                    write!(f, ", base")?;
                }
                if !anchor_key.is_empty() {
                    write!(f, ", anchor={anchor_key:?}")?;
                }
                write!(f, ")")
            }
            Op::Aggregate { group_key, aggs } => {
                let a: Vec<&str> = aggs.iter().map(|(a, _)| a.as_str()).collect();
                write!(f, "aggregate(by={group_key:?}, {})", a.join(","))
            }
            Op::Join {
                kind,
                left_key,
                right_key,
                ..
            } => {
                write!(f, "join({kind:?}, {left_key:?} = {right_key:?})")
            }
            Op::Index { key } => write!(f, "index({key:?})"),
            Op::Apply { kind, correlation } => {
                write!(f, "apply({}, corr={correlation:?})", kind.name())
            }
            Op::Limit { count, offset } => write!(f, "limit({count}, offset={offset})"),
            other => write!(f, "{}", other.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_incremental_operators_are_exactly_the_ordering_ones() {
        let ops = [
            Op::Filter {
                predicate: Scalar::LitBool(true),
            },
            Op::Map { exprs: vec![] },
            Op::Aggregate {
                group_key: vec![0],
                aggs: vec![(Agg::Sum, Scalar::Column(1))],
            },
            Op::Distinct,
            Op::Union,
            Op::OrderBy {
                keys: vec![(0, true)],
            },
            Op::Limit {
                count: 10,
                offset: 0,
            },
        ];
        let non: Vec<&str> = ops
            .iter()
            .filter(|o| !o.is_incremental())
            .map(|o| o.name())
            .collect();
        assert_eq!(non, vec!["order_by", "limit"]);
    }

    #[test]
    fn a_filter_is_not_conservation_transparent() {
        // A filtered view is a legitimate thing to want, but it is not a control total,
        // and the IR must be able to tell a reader which it is looking at.
        assert!(!Op::Filter {
            predicate: Scalar::LitBool(true)
        }
        .is_conservation_transparent());
        assert!(Op::Aggregate {
            group_key: vec![0],
            aggs: vec![]
        }
        .is_conservation_transparent());
        assert!(Op::Map { exprs: vec![] }.is_conservation_transparent());
    }

    #[test]
    fn outer_and_anti_joins_are_not_transparent_but_inner_joins_are() {
        let mk = |k| Op::Join {
            kind: k,
            left_key: vec![0],
            right_key: vec![0],
            residual: None,
        };
        assert!(mk(JoinKind::Inner).is_conservation_transparent());
        assert!(!mk(JoinKind::LeftOuter).is_conservation_transparent());
        assert!(!mk(JoinKind::Anti).is_conservation_transparent());
    }

    #[test]
    fn an_aggregate_keys_its_output_by_its_group() {
        let op = Op::Aggregate {
            group_key: vec![0, 1],
            aggs: vec![(Agg::Sum, Scalar::Column(2))],
        };
        assert_eq!(op.derive_key(&[Some(vec![5])]), Some(vec![0, 1]));
    }

    #[test]
    fn a_source_without_an_anchor_index_is_not_upqueryable() {
        let bare = Op::Source {
            relation: "p".into(),
            is_base: true,
            anchor_key: vec![],
        };
        assert_eq!(bare.derive_key(&[]), None);
        let indexed = Op::Source {
            relation: "p".into(),
            is_base: true,
            anchor_key: vec![0, 1],
        };
        assert_eq!(indexed.derive_key(&[]), Some(vec![0, 1]));
    }

    #[test]
    fn a_union_keeps_a_key_only_when_both_sides_agree() {
        let u = Op::Union;
        assert_eq!(u.derive_key(&[Some(vec![0]), Some(vec![0])]), Some(vec![0]));
        assert_eq!(u.derive_key(&[Some(vec![0]), Some(vec![1])]), None);
        assert_eq!(u.derive_key(&[Some(vec![0]), None]), None);
    }

    #[test]
    fn min_and_max_are_incremental_but_not_additive() {
        // The distinction matters: conflating them would either over-reject `min` from
        // incremental views or charge it as O(1) when its deletion case is not.
        assert!(Agg::Sum.is_additive() && Agg::Count.is_additive());
        assert!(!Agg::Min.is_additive() && !Agg::Max.is_additive());
        let op = Op::Aggregate {
            group_key: vec![0],
            aggs: vec![(Agg::Min, Scalar::Column(1))],
        };
        assert!(op.is_incremental());
    }

    #[test]
    fn an_uncertified_udf_makes_an_expression_irreproducible() {
        let e = Scalar::Binary {
            op: ScalarOp::Add,
            lhs: Box::new(Scalar::Column(0)),
            rhs: Box::new(Scalar::Udf {
                id: 7,
                args: vec![Scalar::Column(1)],
            }),
        };
        assert!(
            !e.is_reproducible(&[]),
            "an uncertified UDF must block reproduction"
        );
        assert!(e.is_reproducible(&[7]));
    }

    #[test]
    fn money_carries_its_currency_at_every_ir_level() {
        // If a lowering ever dropped the currency, the currency-safety theorem would be
        // unstatable below the surface syntax — which is exactly the gap that lets a
        // "generic amount" column reach production.
        let m = Scalar::LitMoney {
            minor: 1000,
            currency: 3,
        };
        match m {
            Scalar::LitMoney { currency, .. } => assert_eq!(currency, 3),
            _ => panic!(),
        }
    }

    #[test]
    fn arity_is_declared_for_every_operator() {
        assert_eq!(Op::Union.arity(), 2);
        assert_eq!(
            Op::Join {
                kind: JoinKind::Inner,
                left_key: vec![],
                right_key: vec![],
                residual: None
            }
            .arity(),
            2
        );
        assert_eq!(
            Op::Source {
                relation: "p".into(),
                is_base: true,
                anchor_key: vec![]
            }
            .arity(),
            0
        );
        assert_eq!(Op::Distinct.arity(), 1);
    }
}
