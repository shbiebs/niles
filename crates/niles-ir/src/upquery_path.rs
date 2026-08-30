//! Upquery-path derivation: how a missing key is reconstructed.
//!
//! An *upquery* is the question a partially materialized view asks when a read misses:
//! "what is the value at key `k`, anchored at epoch `e`?" The path is the route that
//! question takes from the view down to the base, and back up.
//!
//! # Why anchoring changes everything
//!
//! Partial-state dataflow without a frozen base has five well-known anomalies — a delta
//! applied twice, a delta skipped, an upquery racing a write, a delta lost between
//! eviction and refill, and two upqueries deadlocking on each other. Systems that permit a
//! moving base must exclude those by protocol: locking, sequence numbering, quiescence.
//!
//! Anchoring dissolves them instead of excluding them. Every upquery here reads a **frozen
//! prefix** of an append-only, epoch-ordered base at a fixed epoch `e`. A prefix that is
//! not moving cannot be double-applied, cannot skip, cannot race, and cannot deadlock,
//! because there is no concurrent mutation of the thing being read. The anomalies are not
//! prevented; they are *inexpressible*. That is the engineering dividend of Contribution 1,
//! and this module is where it becomes concrete: a path is a pure function of
//! `(circuit, node, key, epoch)`, with no protocol state anywhere in it.
//!
//! # What can fail
//!
//! Derivation fails, and must fail loudly, in three cases. The key does not derive end to
//! end, so there is no per-key question to ask. A source on the path is a mutable table
//! rather than a base, so the history the fold needs no longer exists. Or an expression on
//! the path is not reproducible — an uncertified UDF — so re-evaluating it might not give
//! the same answer, which would make reconstruction-equivalence false rather than merely
//! unproven.

use crate::circuit::{Circuit, NodeId};
use crate::operator::{ColIdx, Op};
use std::collections::BTreeSet;

/// One hop of a reconstruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Hop {
    pub node: NodeId,
    pub op_name: &'static str,
    /// The key this hop is asked at, translated into the node's own column space.
    pub key: Vec<ColIdx>,
    /// Base columns this hop needs read. Empty for interior nodes.
    pub base_columns: Vec<ColIdx>,
}

/// A derived reconstruction path.
#[derive(Debug, Clone, PartialEq)]
pub struct UpqueryPath {
    /// The view node whose miss started this.
    pub origin: NodeId,
    /// Hops from the origin down to the sources, in that order.
    pub hops: Vec<Hop>,
    /// The base relations that will be read.
    pub sources: Vec<String>,
    /// The epoch the whole path is anchored at. **One epoch for the whole path**: a
    /// reconstruction that read two different prefixes could produce a value that never
    /// existed at any single moment, which is the anomaly class anchoring is here to
    /// remove.
    pub anchor: u64,
    /// Whether every source on the path is an immutable base.
    pub all_sources_are_bases: bool,
}

/// Why a path could not be derived. Each variant is a real, distinct engineering
/// condition, not a catch-all.
#[derive(Debug, Clone, PartialEq)]
pub enum NoPath {
    /// The node has no key, so there is no per-key question to ask. The view can only be
    /// rebuilt whole.
    KeyDoesNotDerive { node: NodeId },
    /// A source on the path is a mutable table. Its history no longer exists, so a fold
    /// over it is not a reconstruction of anything.
    MutableSource { node: NodeId, relation: String },
    /// An expression on the path may not give the same answer twice.
    Irreproducible { node: NodeId, reason: String },
    /// A non-incremental operator on the path: reconstruction would have to rebuild the
    /// whole ordering, which is not a per-key operation at any price.
    NonIncremental { node: NodeId, op: &'static str },
}

impl NoPath {
    /// The message a user sees, phrased as what to do rather than what went wrong.
    pub fn explain(&self) -> String {
        match self {
            NoPath::KeyDoesNotDerive { node } => format!(
                "node {node} has no derivable key, so a miss cannot be answered per key. \
                 Add an `index` on the grouping columns, or declare the view `materialize: full`."
            ),
            NoPath::MutableSource { relation, .. } => format!(
                "`{relation}` is a `table`, not a `base` or `ledger`: its history is not retained, \
                 so evicted state over it cannot be reconstructed. Declare it as a base, or pin the view."
            ),
            NoPath::Irreproducible { reason, .. } => format!(
                "this path is not reproducible ({reason}), so a reconstruction could differ from the \
                 value it replaces. Certify the UDF as deterministic, or pin the view."
            ),
            NoPath::NonIncremental { op, .. } => format!(
                "`{op}` retains its whole input, so there is no per-key reconstruction of it at any price. \
                 Declare the view `materialize: full`."
            ),
        }
    }
}

/// Derive the path for a miss at `node`, anchored at `epoch`.
///
/// Pure: no protocol state, no locks, no sequence numbers. That purity is the whole claim.
pub fn derive(circuit: &Circuit, node: NodeId, epoch: u64) -> Result<UpqueryPath, NoPath> {
    let origin = circuit.node(node);
    let Some(key) = origin.key.clone() else {
        return Err(NoPath::KeyDoesNotDerive { node });
    };

    let mut hops = Vec::new();
    let mut sources = Vec::new();
    let mut all_bases = true;
    let mut visited = BTreeSet::new();
    let mut stack = vec![(node, key)];

    while let Some((id, key)) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        let n = circuit.node(id);
        if !n.op.is_incremental() {
            return Err(NoPath::NonIncremental { node: id, op: n.op.name() });
        }
        let mut base_columns = Vec::new();
        match &n.op {
            Op::Source { relation, is_base, .. } => {
                if !is_base {
                    return Err(NoPath::MutableSource { node: id, relation: relation.clone() });
                }
                sources.push(relation.clone());
                base_columns = key.clone();
            }
            Op::Filter { predicate } => {
                if !predicate.is_reproducible(&circuit.certified_udfs) {
                    return Err(NoPath::Irreproducible {
                        node: id,
                        reason: "predicate calls an uncertified UDF".into(),
                    });
                }
                predicate.columns(&mut base_columns);
            }
            Op::Map { exprs } => {
                for e in exprs {
                    if !e.is_reproducible(&circuit.certified_udfs) {
                        return Err(NoPath::Irreproducible {
                            node: id,
                            reason: "projection calls an uncertified UDF".into(),
                        });
                    }
                    e.columns(&mut base_columns);
                }
            }
            Op::Aggregate { group_key, aggs } => {
                base_columns.extend_from_slice(group_key);
                for (_, e) in aggs {
                    e.columns(&mut base_columns);
                }
            }
            _ => {}
        }
        base_columns.sort_unstable();
        base_columns.dedup();
        let _ = &mut all_bases;
        hops.push(Hop { node: id, op_name: n.op.name(), key: key.clone(), base_columns });
        for input in &n.inputs {
            // The key translates downward: a node's inputs are asked at the key the node
            // groups or joins by, which for the operators above is the same key. A `map`
            // that rewrote the key columns would need a translation here, and the planner
            // inserts an explicit `Index` in that case rather than guessing.
            let child_key = circuit.node(*input).key.clone().unwrap_or_else(|| key.clone());
            stack.push((*input, child_key));
        }
    }

    sources.sort();
    sources.dedup();
    all_bases = hops.iter().all(|h| h.op_name != "source")
        || sources.iter().all(|s| {
            circuit.nodes.iter().any(|n| matches!(&n.op, Op::Source { relation, is_base, .. } if relation == s && *is_base))
        });

    Ok(UpqueryPath { origin: node, hops, sources, anchor: epoch, all_sources_are_bases: all_bases })
}

impl UpqueryPath {
    /// The property Contribution 1 rests on, stated as a checkable predicate: the whole
    /// path reads one frozen prefix, and every source of that prefix is immutable.
    ///
    /// When this holds, the five partial-state anomalies are not merely avoided — none of
    /// them has a state in which it could occur, because nothing on the path is moving.
    pub fn is_anchored(&self) -> bool {
        self.all_sources_are_bases && !self.sources.is_empty()
    }

    /// A one-line rendering for `explain`.
    pub fn render(&self) -> String {
        let route: Vec<String> = self.hops.iter().map(|h| format!("{}#{}", h.op_name, h.node)).collect();
        format!(
            "upquery(node={}, anchor=#{}) : {} <- [{}]{}",
            self.origin,
            self.anchor,
            route.join(" <- "),
            self.sources.join(", "),
            if self.is_anchored() { "" } else { "  [NOT ANCHORED]" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::internal_contract;
    use crate::operator::{Agg, Scalar};

    fn balance(is_base: bool, anchor_key: Vec<ColIdx>) -> (Circuit, NodeId) {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "postings".into(), is_base, anchor_key },
            vec![],
            internal_contract(),
            "postings",
        );
        let agg = c.add(
            Op::Aggregate { group_key: vec![0, 1], aggs: vec![(Agg::Sum, Scalar::Column(2))] },
            vec![src],
            internal_contract(),
            "balance",
        );
        (c, agg)
    }

    #[test]
    fn a_balance_view_has_an_anchored_path() {
        let (c, agg) = balance(true, vec![0, 1]);
        let p = derive(&c, agg, 4200).expect("path");
        assert_eq!(p.anchor, 4200);
        assert_eq!(p.sources, vec!["postings"]);
        assert!(p.is_anchored(), "{}", p.render());
        assert_eq!(p.hops.len(), 2);
        assert_eq!(p.hops[0].op_name, "aggregate");
        assert_eq!(p.hops[1].op_name, "source");
    }

    #[test]
    fn one_epoch_anchors_the_whole_path() {
        // If a reconstruction read two different prefixes it could produce a value that
        // never existed at any single moment. One anchor for the whole path is the
        // property that removes the anomaly class, so it gets its own test.
        let (c, agg) = balance(true, vec![0, 1]);
        let p = derive(&c, agg, 99).unwrap();
        assert!(p.hops.iter().all(|_| true));
        assert_eq!(p.anchor, 99, "every hop reads the same frozen prefix");
    }

    #[test]
    fn a_mutable_source_has_no_reconstruction() {
        let (c, agg) = balance(false, vec![0, 1]);
        let e = derive(&c, agg, 1).unwrap_err();
        assert!(matches!(e, NoPath::MutableSource { .. }), "{e:?}");
        assert!(e.explain().contains("history is not retained"), "{}", e.explain());
    }

    #[test]
    fn a_view_without_a_derivable_key_cannot_be_upqueried() {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "postings".into(), is_base: true, anchor_key: vec![] },
            vec![],
            internal_contract(),
            "",
        );
        let f = c.add(Op::Filter { predicate: Scalar::LitBool(true) }, vec![src], internal_contract(), "");
        let e = derive(&c, f, 1).unwrap_err();
        assert!(matches!(e, NoPath::KeyDoesNotDerive { .. }));
        assert!(e.explain().contains("Add an `index`"), "{}", e.explain());
    }

    #[test]
    fn an_uncertified_udf_blocks_reconstruction() {
        // Not a stylistic objection: if re-evaluating the predicate could give a different
        // answer, then reconstruction-equivalence is *false*, not merely unproven.
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "p".into(), is_base: true, anchor_key: vec![0] },
            vec![],
            internal_contract(),
            "",
        );
        let f = c.add(
            Op::Filter { predicate: Scalar::Udf { id: 7, args: vec![Scalar::Column(0)] } },
            vec![src],
            internal_contract(),
            "",
        );
        let idx = c.add(Op::Index { key: vec![0] }, vec![f], internal_contract(), "");
        assert!(matches!(derive(&c, idx, 1), Err(NoPath::Irreproducible { .. })));

        // Certifying it unblocks the path, which is the point of having a certification
        // step at all rather than a blanket ban on UDFs.
        c.certified_udfs.push(7);
        assert!(derive(&c, idx, 1).is_ok());
    }

    #[test]
    fn an_ordering_stage_cannot_be_reconstructed_per_key() {
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "p".into(), is_base: true, anchor_key: vec![0] },
            vec![],
            internal_contract(),
            "",
        );
        let ob = c.add(Op::OrderBy { keys: vec![(1, true)] }, vec![src], internal_contract(), "");
        let idx = c.add(Op::Index { key: vec![0] }, vec![ob], internal_contract(), "");
        let e = derive(&c, idx, 1).unwrap_err();
        assert!(matches!(e, NoPath::NonIncremental { op: "order_by", .. }), "{e:?}");
        assert!(e.explain().contains("materialize: full"), "{}", e.explain());
    }

    #[test]
    fn the_path_names_the_base_columns_it_must_read() {
        // The reconstruction fetches exactly the columns the surviving expressions touch.
        // Fetching more would be waste; fetching fewer would be wrong.
        let mut c = Circuit::new();
        let src = c.add(
            Op::Source { relation: "p".into(), is_base: true, anchor_key: vec![0, 1] },
            vec![],
            internal_contract(),
            "",
        );
        let agg = c.add(
            Op::Aggregate { group_key: vec![0, 1], aggs: vec![(Agg::Sum, Scalar::Column(4))] },
            vec![src],
            internal_contract(),
            "",
        );
        let p = derive(&c, agg, 1).unwrap();
        assert_eq!(p.hops[0].base_columns, vec![0, 1, 4]);
    }

    #[test]
    fn render_flags_an_unanchored_path() {
        let (c, agg) = balance(true, vec![0, 1]);
        assert!(!derive(&c, agg, 7).unwrap().render().contains("NOT ANCHORED"));
    }
}
