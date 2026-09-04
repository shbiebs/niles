//! The circuit: a DAG of operators, and the stable contract between language and engine.
//!
//! # The accessed-field discipline
//!
//! The most consequential design decision in this file is borrowed from GoogleSQL's
//! resolved AST, which marks fields as ignorable or non-ignorable and offers a
//! `CheckFieldsAccessed` call so that a back end which *silently skipped* a semantic
//! annotation fails loudly rather than producing a plausible wrong answer.
//!
//! For a system whose central claim is that money cannot be created or destroyed, that
//! mechanism is worth more per engineering hour than almost anything else available. The
//! dangerous failure here is not a crash. It is an engine that reads a circuit, does not
//! notice the `consistency: ledger_consistent` annotation on a node, serves the view from
//! stale state, and returns a number that looks exactly like the right one. No test of the
//! answer catches that, because the answer is well-formed; only a test of *whether the
//! annotation was read* catches it.
//!
//! So every semantically load-bearing field on a node is behind an accessor that records
//! the read, and [`Circuit::assert_all_accessed`] fails if any went unread. The engine
//! calls it after planning. This composes with the proofs rather than substituting for
//! them: the theorems say what a correct engine computes, and this says the engine at
//! least *looked at* every premise the theorems quantify over.
//!
//! # Four IR levels, and which one this is
//!
//! Following the account of why rustc has MIR — each level exists because something
//! becomes checkable at it that was not checkable before — Niles has four:
//!
//! | Level | What becomes checkable |
//! |---|---|
//! | Surface CST (`niles-lang::ast`) | syntax; formatting; IDE resilience |
//! | Resolved tree (`niles-lang::resolve`) | names, at an epoch; declaration well-formedness |
//! | Typed tree (`niles-lang::typecheck`) | conservation, currency, linearity, effects, rung |
//! | **Circuit (this file)** | **the partial-state algebra: keys, upquery paths, materialization, anchor discipline** |
//!
//! This is the level the theorems of Chapters 3 and 4 quantify over. An IR that erased the
//! anchors, the contracts or the provenance would leave those proofs talking about a
//! different object.

use crate::operator::{ColIdx, Op};
use crate::{Consistency, Lineage, Materialize, Retention, ServeContract};
use std::cell::Cell;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;

pub type NodeId = u32;

/// A node's anchor discipline: what epoch its output is stamped with.
///
/// Every answer in this system carries an epoch. That is not telemetry — it is what makes
/// "the same question, asked twice, answered consistently" a checkable statement, and it
/// is the field an engine is most likely to drop because nothing visibly breaks when it
/// does. Hence its place among the accessed-checked fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// Stamped with the visibility frontier at read time. The strict path.
    Frontier,
    /// Stamped with a fixed epoch: an `as_of` read, or a reconstruction from a frozen
    /// prefix. This is the discipline that dissolves the five partial-state anomalies —
    /// once the base a reconstruction reads is not moving, a double application or a
    /// skipped delta has no way to occur.
    Pinned(u64),
    /// Inherited from the node's inputs, taking the *minimum*: a result is no fresher than
    /// its stalest input.
    Inherited,
}

/// A semantic field that the consumer must read. Wraps the value with a read flag.
///
/// The `Cell` is deliberate: recording a read must not require a mutable borrow, or every
/// consumer would need `&mut Circuit` and the discipline would be abandoned within a week.
#[derive(Debug)]
pub struct Checked<T> {
    value: T,
    accessed: Cell<bool>,
    name: &'static str,
}

impl<T: Clone> Clone for Checked<T> {
    fn clone(&self) -> Self {
        Checked {
            value: self.value.clone(),
            accessed: Cell::new(self.accessed.get()),
            name: self.name,
        }
    }
}

impl<T> Checked<T> {
    pub fn new(name: &'static str, value: T) -> Self {
        Checked {
            value,
            accessed: Cell::new(false),
            name,
        }
    }
    /// Read the field, recording that it was read.
    pub fn get(&self) -> &T {
        self.accessed.set(true);
        &self.value
    }
    /// Read without recording. For printing and debugging only — never for a decision,
    /// because a decision taken on a peeked field is exactly what this mechanism exists to
    /// catch.
    pub fn peek(&self) -> &T {
        &self.value
    }
    pub fn was_accessed(&self) -> bool {
        self.accessed.get()
    }
    pub fn reset(&self) {
        self.accessed.set(false);
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// One node of the circuit.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub op: Op,
    pub inputs: Vec<NodeId>,
    /// The output key. `None` means not upqueryable without an explicit `Index`.
    pub key: Option<Vec<ColIdx>>,
    /// Column count of the output schema.
    pub arity: u16,

    // ---- semantically load-bearing; access is checked ----
    /// The epoch discipline of this node's output.
    pub anchor: Checked<Anchor>,
    /// The serve contract in force at this node.
    pub contract: Checked<ServeContract>,
    /// Whether a monetary quantity entering this node necessarily leaves it. An engine
    /// that presents a non-transparent node's output as a control total is wrong in a way
    /// no answer-level test detects.
    pub conservation_transparent: Checked<bool>,
    /// The provenance mode. `Off` still stamps the anchor.
    pub lineage: Checked<Lineage>,
    /// A human name, for `explain`. Not semantically load-bearing, so not checked.
    pub label: String,
}

impl Node {
    /// Every checked field, for the audit.
    fn checked_fields(&self) -> Vec<(&'static str, bool)> {
        vec![
            (self.anchor.name(), self.anchor.was_accessed()),
            (self.contract.name(), self.contract.was_accessed()),
            (
                self.conservation_transparent.name(),
                self.conservation_transparent.was_accessed(),
            ),
            (self.lineage.name(), self.lineage.was_accessed()),
        ]
    }
    pub fn reset_access(&self) {
        self.anchor.reset();
        self.contract.reset();
        self.conservation_transparent.reset();
        self.lineage.reset();
    }

    /// A new node carrying this one's **semantic** fields, with a new operator and inputs.
    ///
    /// For a rewrite that inserts a node — the compensating projection a commuted join needs,
    /// or a filter moved below one. The anchor discipline, the serve contract, the lineage
    /// mode and conservation transparency are inherited deliberately rather than defaulted:
    /// a rewrite must not be able to *change* a node's epoch discipline as a side effect, and
    /// defaulting them would do exactly that while looking like housekeeping.
    ///
    /// Conservation transparency is the one that would bite hardest. A `Map` that reorders
    /// columns moves every monetary quantity that enters it, so it inherits the join's
    /// transparency; a node that defaulted to opaque would make the circuit's control totals
    /// unusable for a reason no answer-level test would find.
    pub fn clone_shell(
        &self,
        id: NodeId,
        op: Op,
        inputs: Vec<NodeId>,
        arity: u16,
        label: String,
    ) -> Node {
        Node {
            id,
            op,
            inputs,
            key: self.key.clone(),
            arity,
            anchor: Checked::new("anchor", *self.anchor.peek()),
            contract: Checked::new("contract", *self.contract.peek()),
            conservation_transparent: Checked::new(
                "conservation_transparent",
                *self.conservation_transparent.peek(),
            ),
            lineage: Checked::new("lineage", *self.lineage.peek()),
            label,
        }
    }
}

/// A whole circuit: the lowered form of one or more views.
#[derive(Debug, Clone, Default)]
pub struct Circuit {
    pub nodes: Vec<Node>,
    /// Named outputs: the views this circuit serves.
    pub outputs: HashMap<String, NodeId>,
    /// UDF ids the loader has certified deterministic. Empty until certification runs, so
    /// the default is the conservative one.
    pub certified_udfs: Vec<u32>,
}

/// What a field-access audit found.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessReport {
    pub total_fields: usize,
    pub unread: Vec<(NodeId, &'static str)>,
}

impl AccessReport {
    pub fn is_clean(&self) -> bool {
        self.unread.is_empty()
    }
}

impl Circuit {
    pub fn new() -> Self {
        Circuit::default()
    }

    /// Add a node, deriving its key from its inputs.
    pub fn add(
        &mut self,
        op: Op,
        inputs: Vec<NodeId>,
        contract: ServeContract,
        label: impl Into<String>,
    ) -> NodeId {
        let id = self.nodes.len() as NodeId;
        let input_keys: Vec<Option<Vec<ColIdx>>> = inputs
            .iter()
            .map(|i| self.nodes[*i as usize].key.clone())
            .collect();
        let key = op.derive_key(&input_keys);
        // Conservation transparency composes: a path is transparent only if every node on
        // it is. Taking the conjunction here, at construction, is what makes the property
        // a *path* property rather than a per-node curiosity.
        let transparent = op.is_conservation_transparent()
            && inputs
                .iter()
                .all(|i| *self.nodes[*i as usize].conservation_transparent.peek());
        let anchor = match &op {
            Op::AsOf { epoch: Some(e) } => Anchor::Pinned(*e),
            Op::Source { .. } => Anchor::Frontier,
            _ => Anchor::Inherited,
        };
        let width = |i: &NodeId| self.nodes[*i as usize].arity;
        let arity = match &op {
            Op::Map { exprs } => exprs.len() as u16,
            Op::Aggregate { group_key, aggs } => (group_key.len() + aggs.len()) as u16,
            // A join's output width is the sum of its inputs' — **except** for semi and
            // anti, which emit left rows only. Deriving this from the left input alone, as
            // this did before `Semi`/`Anti` had a reference semantics, made an inner join
            // claim its left width; nothing noticed, because no consumer read `arity` for
            // an inner join. The distinction is load-bearing now: an engine that widened a
            // semi-join is one that duplicated its left rows, which is the classic
            // unnesting defect, and `verify` rejects it on exactly this field.
            Op::Join { kind, .. } => match kind {
                crate::operator::JoinKind::Semi | crate::operator::JoinKind::Anti => {
                    inputs.first().map(width).unwrap_or(0)
                }
                _ => inputs.iter().map(width).sum(),
            },
            // An apply is the outer row, plus one column if a scalar subquery widened it.
            Op::Apply { kind, .. } => inputs.first().map(width).unwrap_or(0) + kind.widens() as u16,
            _ => inputs.first().map(width).unwrap_or(0),
        };
        let lineage = contract.lineage;
        self.nodes.push(Node {
            id,
            op,
            inputs,
            key,
            arity,
            anchor: Checked::new("anchor", anchor),
            contract: Checked::new("contract", contract),
            conservation_transparent: Checked::new("conservation_transparent", transparent),
            lineage: Checked::new("lineage", lineage),
            label: label.into(),
        });
        id
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    pub fn set_output(&mut self, name: impl Into<String>, id: NodeId) {
        self.outputs.insert(name.into(), id);
    }

    /// Nodes in dependency order. `None` if the graph has a cycle not mediated by a
    /// `Delay`, which is the only legal way to close one.
    pub fn topological_order(&self) -> Option<Vec<NodeId>> {
        let mut indegree: HashMap<NodeId, usize> = HashMap::new();
        for n in &self.nodes {
            indegree.entry(n.id).or_insert(0);
            for i in &n.inputs {
                if matches!(self.nodes[n.id as usize].op, Op::Delay) {
                    continue; // a delay breaks the cycle by construction
                }
                let _ = i;
                *indegree.entry(n.id).or_insert(0) += 1;
            }
        }
        let mut ready: Vec<NodeId> = indegree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(k, _)| *k)
            .collect();
        ready.sort_unstable();
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        while let Some(id) = ready.pop() {
            if !seen.insert(id) {
                continue;
            }
            out.push(id);
            for n in &self.nodes {
                if n.inputs.contains(&id) {
                    let d = indegree.get_mut(&n.id).unwrap();
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        ready.push(n.id);
                    }
                }
            }
        }
        if out.len() == self.nodes.len() {
            Some(out)
        } else {
            None
        }
    }

    /// The effective anchor of a node: `Inherited` resolves to the *minimum* of its
    /// inputs', because a result is no fresher than its stalest input.
    pub fn effective_anchor(&self, id: NodeId) -> Anchor {
        let n = self.node(id);
        match n.anchor.get() {
            Anchor::Inherited => {
                let mut pinned: Option<u64> = None;
                let mut any_frontier = false;
                for i in &n.inputs {
                    match self.effective_anchor(*i) {
                        Anchor::Pinned(e) => pinned = Some(pinned.map_or(e, |p: u64| p.min(e))),
                        Anchor::Frontier => any_frontier = true,
                        Anchor::Inherited => {}
                    }
                }
                match (pinned, any_frontier) {
                    (Some(e), _) => Anchor::Pinned(e),
                    (None, true) => Anchor::Frontier,
                    _ => Anchor::Inherited,
                }
            }
            other => *other,
        }
    }

    /// The audit: which semantic fields were never read?
    ///
    /// The engine calls this after planning. A non-empty result is a hard error, not a
    /// warning: an unread annotation means some part of the plan was made without a
    /// premise the theorems assume was consulted.
    pub fn audit_access(&self) -> AccessReport {
        let mut unread = Vec::new();
        let mut total = 0usize;
        for n in &self.nodes {
            for (name, read) in n.checked_fields() {
                total += 1;
                if !read {
                    unread.push((n.id, name));
                }
            }
        }
        AccessReport {
            total_fields: total,
            unread,
        }
    }

    /// The hard form. Panics with a message naming every unread field.
    pub fn assert_all_accessed(&self) {
        let r = self.audit_access();
        assert!(
            r.is_clean(),
            "the consumer of this circuit ignored {} semantic field(s): {:?}\n\
             every one of these is a premise some theorem in Chapters 3-4 quantifies over; \
             a plan made without reading one is not a plan the proofs cover",
            r.unread.len(),
            r.unread
        );
    }

    pub fn reset_access(&self) {
        self.nodes.iter().for_each(|n| n.reset_access());
    }

    /// Every node reachable from a named output. Lowering can leave an orphan behind — an
    /// `Index` whose aggregate was folded into it, for example — and printing orphans in
    /// `explain` makes a plan look more complicated than it is.
    pub fn live_nodes(&self) -> BTreeSet<NodeId> {
        let mut live = BTreeSet::new();
        let mut stack: Vec<NodeId> = self.outputs.values().copied().collect();
        while let Some(id) = stack.pop() {
            if !live.insert(id) {
                continue;
            }
            if (id as usize) < self.nodes.len() {
                stack.extend(self.nodes[id as usize].inputs.iter().copied());
            }
        }
        live
    }

    /// `EXPLAIN`, at this IR level. Every level is explainable, as in Materialize, because
    /// a plan a reader cannot see is a plan a reader cannot review.
    pub fn explain(&self) -> String {
        let mut s = String::new();
        let order = self
            .topological_order()
            .unwrap_or_else(|| (0..self.nodes.len() as NodeId).collect());
        let live = self.live_nodes();
        for id in order {
            if !self.outputs.is_empty() && !live.contains(&id) {
                continue;
            }
            let n = self.node(id);
            let _ = writeln!(
                s,
                "  {:>3}  {:<40} in={:?} key={:?} anchor={:?} rung={:?} conserving={} {}",
                n.id,
                n.op.to_string(),
                n.inputs,
                n.key,
                n.anchor.peek(),
                n.contract.peek().consistency,
                n.conservation_transparent.peek(),
                if n.label.is_empty() {
                    String::new()
                } else {
                    format!("// {}", n.label)
                }
            );
        }
        for (name, id) in &self.outputs {
            let _ = writeln!(s, "  output {name} = node {id}");
        }
        s
    }
}

/// The default contract for an internal node: inherits, decides nothing.
pub fn internal_contract() -> ServeContract {
    ServeContract {
        consistency: Consistency::Snapshot,
        materialize: Materialize::Auto,
        retain: Retention::Evictable,
        lineage: Lineage::Off,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::{Agg, Scalar};

    fn source(c: &mut Circuit, name: &str, anchor_key: Vec<ColIdx>) -> NodeId {
        c.add(
            Op::Source {
                relation: name.into(),
                is_base: true,
                anchor_key,
                confidential: Vec::new(),
            },
            vec![],
            internal_contract(),
            name,
        )
    }

    fn balance_circuit() -> (Circuit, NodeId) {
        let mut c = Circuit::new();
        let src = source(&mut c, "postings", vec![0, 1]);
        let agg = c.add(
            Op::Aggregate {
                group_key: vec![0, 1],
                aggs: vec![(Agg::Sum, Scalar::Column(2))],
            },
            vec![src],
            ServeContract {
                consistency: Consistency::LedgerConsistent,
                materialize: Materialize::Demand,
                retain: Retention::Evictable,
                lineage: Lineage::Key,
            },
            "ledger_balance",
        );
        c.set_output("ledger_balance", agg);
        (c, agg)
    }

    #[test]
    fn an_ignored_semantic_field_is_caught() {
        // The whole point: an engine that never reads the contract produces a plausible
        // wrong answer, and no answer-level test detects it. This one does.
        let (c, _) = balance_circuit();
        let r = c.audit_access();
        assert_eq!(r.total_fields, 8, "two nodes x four checked fields");
        assert_eq!(r.unread.len(), 8, "nothing has been read yet");
        assert!(!r.is_clean());
    }

    #[test]
    fn a_consumer_that_reads_everything_passes_the_audit() {
        let (c, _) = balance_circuit();
        for n in &c.nodes {
            let _ = n.anchor.get();
            let _ = n.contract.get();
            let _ = n.conservation_transparent.get();
            let _ = n.lineage.get();
        }
        assert!(c.audit_access().is_clean());
        c.assert_all_accessed();
    }

    #[test]
    fn peeking_does_not_count_as_reading() {
        // Otherwise `explain` would launder the discipline: printing a field is not
        // deciding anything with it.
        let (c, _) = balance_circuit();
        let _ = c.explain();
        assert_eq!(
            c.audit_access().unread.len(),
            8,
            "explain must not mark fields as read"
        );
    }

    #[test]
    fn conservation_transparency_composes_along_a_path() {
        let mut c = Circuit::new();
        let src = source(&mut c, "postings", vec![0, 1]);
        assert!(*c.node(src).conservation_transparent.peek());
        let filtered = c.add(
            Op::Filter {
                predicate: Scalar::LitBool(true),
            },
            vec![src],
            internal_contract(),
            "recent_only",
        );
        assert!(
            !*c.node(filtered).conservation_transparent.peek(),
            "a filter breaks transparency"
        );
        // ... and everything downstream of it inherits the break, which is the property
        // that makes this a path judgement rather than a per-node one.
        let agg = c.add(
            Op::Aggregate {
                group_key: vec![0],
                aggs: vec![(Agg::Sum, Scalar::Column(2))],
            },
            vec![filtered],
            internal_contract(),
            "filtered_total",
        );
        assert!(
            !*c.node(agg).conservation_transparent.peek(),
            "a total over a filtered ledger is not a control total, and the IR must say so"
        );
    }

    #[test]
    fn a_result_is_no_fresher_than_its_stalest_input() {
        let mut c = Circuit::new();
        let live = source(&mut c, "postings", vec![0]);
        let pinned = c.add(
            Op::AsOf { epoch: Some(4200) },
            vec![live],
            internal_contract(),
            "historic",
        );
        let joined = c.add(
            Op::Join {
                kind: crate::operator::JoinKind::Inner,
                left_key: vec![0],
                right_key: vec![0],
                residual: None,
            },
            vec![live, pinned],
            internal_contract(),
            "mixed",
        );
        assert_eq!(c.effective_anchor(pinned), Anchor::Pinned(4200));
        assert_eq!(
            c.effective_anchor(joined),
            Anchor::Pinned(4200),
            "joining live data against a pinned read yields a pinned answer, not a live one"
        );
    }

    #[test]
    fn keys_derive_end_to_end_or_the_view_is_not_upqueryable() {
        let (c, agg) = balance_circuit();
        assert_eq!(c.node(agg).key, Some(vec![0, 1]));

        // Without an anchor index on the source, the key does not derive, and the view
        // cannot be reconstructed per key — it would have to be rebuilt whole.
        let mut c2 = Circuit::new();
        let src = source(&mut c2, "postings", vec![]);
        let f = c2.add(
            Op::Filter {
                predicate: Scalar::LitBool(true),
            },
            vec![src],
            internal_contract(),
            "",
        );
        assert_eq!(c2.node(f).key, None);
    }

    #[test]
    fn topological_order_covers_every_node() {
        let (c, _) = balance_circuit();
        let o = c.topological_order().expect("acyclic");
        assert_eq!(o.len(), c.nodes.len());
    }

    #[test]
    fn explain_prints_every_node_and_output() {
        let (c, _) = balance_circuit();
        let e = c.explain();
        assert!(e.contains("source(postings"), "{e}");
        assert!(e.contains("aggregate(by=[0, 1], sum)"), "{e}");
        assert!(e.contains("output ledger_balance"), "{e}");
        assert!(
            e.contains("LedgerConsistent"),
            "the rung must be visible in explain:\n{e}"
        );
    }
}
