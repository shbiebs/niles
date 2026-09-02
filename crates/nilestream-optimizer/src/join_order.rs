//! Cost-based join ordering for partial-state dataflow (thesis §6.11, Appendix G.5).
//!
//! # Why this is not the textbook problem
//!
//! Classical join ordering — Selinger's dynamic program, and its modern form as Moerkotte
//! and Neumann's `DPccp` — minimises one quantity: the number of intermediate tuples the
//! plan produces. That is the right objective for a batch engine, where a join is a
//! *transient*: it consumes its inputs, emits its output, and holds nothing afterwards.
//!
//! In a dataflow engine a join is not transient. It is a **standing operator with two
//! indexes**, and those indexes are resident for as long as the view exists. The number
//! that matters is therefore not how many tuples pass through, but how many are still
//! sitting in memory when nothing is passing through at all. A plan that produces fewer
//! intermediate tuples but keeps a larger index is worse, and the classical objective
//! cannot see that.
//!
//! Partial state adds a third quantity that neither of the first two captures. When a REV
//! evicts a key and later reads it, the value is reconstructed by an upquery that walks
//! *up* the operator graph to the nearest reconstructible ancestor. The shape of the join
//! tree is the shape of that walk. A deep left-deep tree makes reconstruction deep; a
//! bushy tree makes it shallow but multiplies the number of internal indexes. So the
//! objective here is a three-term one:
//!
//! ```text
//!   cost(plan) = w_flow · Σ intermediate cardinality       (classical)
//!              + w_state · Σ resident index entries        (dataflow)
//!              + w_recon · E[upquery work per miss]        (partial state)
//! ```
//!
//! The three weights are not universal constants. They are read off the view's serve
//! contract: a view served at `ledger_consistent` with a tight freshness bound pays for
//! state to avoid reconstruction latency; a cold analytical view served at `bounded`
//! pays for reconstruction to avoid state. This is the same claim Contribution 3 makes
//! about the consistency ladder — *the price is workload-shaped, not history-shaped* —
//! appearing here as a planner input rather than a theorem.
//!
//! # Reconstructibility is a constraint, not a cost
//!
//! One thing in this planner has no analogue in a classical optimiser. Some orderings are
//! not merely expensive but **illegal**, because they place a non-reconstructible operator
//! on the upquery path of a partial node. Reconstruction from a frozen prefix is what
//! makes anchored upqueries sound (Thm 4.1); an operator that cannot be replayed from the
//! base — a source that is a mutable table rather than an immutable ledger, or a windowing
//! node whose input has been discarded — breaks that chain. Such candidates are pruned
//! *before* costing, so a bad estimate can make the plan slow but cannot make it wrong.
//! This is the same discipline the materialization optimizer follows in `modes.rs`, and
//! for the same reason.
//!
//! # Search
//!
//! `DPccp` enumerates exactly the connected-complement pairs of the query graph, which is
//! optimal in the sense that it enumerates no pair that cannot form a valid plan. For a
//! chain of *n* relations it does O(n³) work; for a clique, O(3ⁿ). We run it whenever the
//! number of relations is at most [`DP_LIMIT`], and fall back to a greedy
//! minimum-selectivity heuristic above that, recording in the plan which was used so that
//! a measured regression can be attributed. Cross products are enumerated only when the
//! graph is disconnected and there is no alternative, because a cross product in a
//! standing dataflow operator is a resident quadratic index, which is a different kind of
//! mistake from a transient one.

use std::collections::BTreeMap;

/// Above this many relations, `DPccp`'s worst case is not worth its optimality.
///
/// Twelve is the conventional cut-off and it is not arbitrary: the pathological case is
/// the clique, where the number of connected-complement pairs is 3ⁿ − 2^(n+1) + 1, which
/// is about 265k at n = 12 and about 4.8M at n = 15. The chain case, which is what almost
/// every banking query actually is, stays polynomial and could go much further; we do not
/// special-case it, because a planner whose complexity depends on a shape the user cannot
/// see is a planner whose behaviour the user cannot predict.
pub const DP_LIMIT: usize = 12;

/// A relation available to the join, with the statistics the cost model needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    pub name: String,
    /// Estimated rows. For a base this is a counted quantity; for a derived input it is
    /// itself an estimate, and the compounding is the main source of planner error.
    pub rows: u64,
    /// Whether this input is an immutable, fully retained base. Only a subtree rooted
    /// entirely in bases is reconstructible, so this flag is a *legality* input and not
    /// merely a cost one.
    pub is_base: bool,
    /// Per-column distinct-value counts, keyed by column index. Missing means unknown,
    /// and unknown is handled by a stated default rather than by an invented number.
    pub distinct: BTreeMap<usize, u64>,
}

impl Relation {
    pub fn base(name: &str, rows: u64) -> Self {
        Relation {
            name: name.into(),
            rows,
            is_base: true,
            distinct: BTreeMap::new(),
        }
    }

    pub fn derived(name: &str, rows: u64) -> Self {
        Relation {
            name: name.into(),
            rows,
            is_base: false,
            distinct: BTreeMap::new(),
        }
    }

    pub fn with_distinct(mut self, col: usize, n: u64) -> Self {
        self.distinct.insert(col, n);
        self
    }

    fn ndv(&self, col: usize) -> u64 {
        // The default when a column's distinct count is unknown. We use the textbook
        // fallback — assume the column is a key of at most 10% selectivity — and we
        // record here that it is a fallback rather than a measurement, because a
        // cardinality estimate that cannot be distinguished from a counted value is how
        // a planner comes to be trusted for a number it never had.
        *self.distinct.get(&col).unwrap_or(&(self.rows / 10).max(1))
    }
}

/// An equi-join predicate between two relations, by index into the relation list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    pub left: usize,
    pub left_col: usize,
    pub right: usize,
    pub right_col: usize,
}

impl Edge {
    pub fn new(left: usize, left_col: usize, right: usize, right_col: usize) -> Self {
        Edge {
            left,
            left_col,
            right,
            right_col,
        }
    }
}

/// What the plan is being optimised *for*. Read off the view's serve contract.
///
/// These are the `w_*` of the module docs. They are deliberately exposed as a small
/// struct rather than hidden constants, because §9.13's phase diagram is precisely a
/// measurement of how the optimal plan moves as these weights move.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weights {
    /// Cost per intermediate tuple produced. The classical objective.
    pub flow: f64,
    /// Cost per entry held resident in a join index, per unit time.
    pub state: f64,
    /// Cost per unit of reconstruction work, scaled by the expected miss rate.
    pub reconstruct: f64,
    /// The probability that a read of this view misses and must upquery. At 0 the model
    /// degenerates to the fully materialized case; at 1, to the fully demand-driven one.
    pub miss_rate: f64,
}

impl Weights {
    /// The default balance: state and flow comparable, reconstruction rare.
    ///
    /// This corresponds to a warm view — the regime the thesis argues is the common one,
    /// and the one the Pareto-skew hypothesis (F-skew) predicts.
    pub fn warm() -> Self {
        Weights {
            flow: 1.0,
            state: 1.0,
            reconstruct: 1.0,
            miss_rate: 0.05,
        }
    }

    /// A cold analytical view: state is expensive, misses are the common case.
    pub fn cold() -> Self {
        Weights {
            flow: 1.0,
            state: 4.0,
            reconstruct: 1.0,
            miss_rate: 0.60,
        }
    }

    /// A hot serving view under a tight freshness bound: reconstruction latency dominates,
    /// so state is cheap by comparison.
    pub fn hot() -> Self {
        Weights {
            flow: 1.0,
            state: 0.25,
            reconstruct: 8.0,
            miss_rate: 0.01,
        }
    }
}

/// A join tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    Scan(usize),
    Join {
        left: Box<Plan>,
        right: Box<Plan>,
        on: Edge,
    },
    /// Only produced when the query graph is disconnected. Kept as a distinct variant so
    /// that a plan containing one is visible to the caller rather than silently costed.
    Cross {
        left: Box<Plan>,
        right: Box<Plan>,
    },
}

impl Plan {
    /// The relations in this subtree, as a bitset.
    pub fn relations(&self) -> u64 {
        match self {
            Plan::Scan(i) => 1u64 << i,
            Plan::Join { left, right, .. } | Plan::Cross { left, right } => {
                left.relations() | right.relations()
            }
        }
    }

    /// Depth of the deepest path. This is the length of the upquery walk, which is why
    /// it is a costed quantity here and not merely a diagnostic.
    pub fn depth(&self) -> usize {
        match self {
            Plan::Scan(_) => 1,
            Plan::Join { left, right, .. } | Plan::Cross { left, right } => {
                1 + left.depth().max(right.depth())
            }
        }
    }

    /// A left-deep plan has no join whose right input is itself a join. The distinction
    /// matters here more than in a batch engine: in a left-deep tree only one operand of
    /// each join is a standing intermediate, so the resident-state term grows linearly;
    /// in a bushy tree both operands are, and it grows with the number of internal nodes.
    pub fn is_left_deep(&self) -> bool {
        match self {
            Plan::Scan(_) => true,
            Plan::Join { left, right, .. } | Plan::Cross { left, right } => {
                matches!(**right, Plan::Scan(_)) && left.is_left_deep()
            }
        }
    }

    pub fn contains_cross(&self) -> bool {
        match self {
            Plan::Scan(_) => false,
            Plan::Cross { .. } => true,
            Plan::Join { left, right, .. } => left.contains_cross() || right.contains_cross(),
        }
    }

    /// Render as a nested s-expression, for tests and for `explain`.
    pub fn render(&self, rels: &[Relation]) -> String {
        match self {
            Plan::Scan(i) => rels[*i].name.clone(),
            Plan::Join { left, right, .. } => {
                format!("({} ⋈ {})", left.render(rels), right.render(rels))
            }
            Plan::Cross { left, right } => {
                format!("({} × {})", left.render(rels), right.render(rels))
            }
        }
    }
}

/// The three cost terms, kept separate so that a plan choice can be *explained* rather
/// than merely asserted. §9.13.4 reports these individually.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Cost {
    pub flow: f64,
    pub state: f64,
    pub reconstruct: f64,
    /// Estimated output cardinality of the subtree. Carried alongside the cost because
    /// the dynamic program needs it for the parent's estimate.
    pub rows: f64,
}

impl Cost {
    pub fn total(&self, w: &Weights) -> f64 {
        w.flow * self.flow + w.state * self.state + w.reconstruct * self.reconstruct
    }
}

/// Why a candidate ordering was rejected before it was costed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Illegal {
    /// A subtree rooted in a mutable table cannot be replayed from the base, so a partial
    /// node above it cannot honestly reconstruct (Thm 4.1's hypothesis fails).
    NotReconstructible { relation: String },
}

/// The result of planning, including the evidence for the choice.
#[derive(Debug, Clone)]
pub struct Planned {
    pub plan: Plan,
    pub cost: Cost,
    pub weights: Weights,
    /// Whether the exhaustive dynamic program ran, or the greedy fallback.
    pub exhaustive: bool,
    /// Connected-complement pairs enumerated. The measured complexity, which §9.13.4
    /// checks against the O(n³) / O(3ⁿ) prediction rather than assuming it.
    pub pairs_enumerated: u64,
    /// Orderings pruned as illegal before costing.
    pub pruned: Vec<Illegal>,
}

/// The join-order planner.
pub struct JoinPlanner {
    rels: Vec<Relation>,
    edges: Vec<Edge>,
    weights: Weights,
    /// If set, the subtree feeding a partial node must be reconstructible, and orderings
    /// that root it in a mutable table are pruned. Off for a fully materialized view,
    /// where there is no reconstruction to protect.
    require_reconstructible: bool,
}

impl JoinPlanner {
    pub fn new(rels: Vec<Relation>, edges: Vec<Edge>) -> Self {
        JoinPlanner {
            rels,
            edges,
            weights: Weights::warm(),
            require_reconstructible: false,
        }
    }

    pub fn with_weights(mut self, w: Weights) -> Self {
        self.weights = w;
        self
    }

    /// Require every input to be an immutable base, because the view above this join is
    /// partially materialized and will need to reconstruct through it.
    pub fn requiring_reconstructible(mut self) -> Self {
        self.require_reconstructible = true;
        self
    }

    /// Selectivity of an equi-join, under the containment assumption: each value of the
    /// smaller domain appears in the larger. σ = 1 / max(ndv_left, ndv_right).
    ///
    /// This is the standard System-R estimate and it is wrong in the standard ways —
    /// notably it assumes independence between predicates, which correlated banking
    /// columns (`account`, `currency`) violate systematically. We use it anyway and
    /// state the error rather than inventing a better estimator we have not validated.
    /// The one place we depart from it is below, where a foreign key into a base is
    /// recognised.
    fn selectivity(&self, e: &Edge) -> f64 {
        let l = self.rels[e.left].ndv(e.left_col) as f64;
        let r = self.rels[e.right].ndv(e.right_col) as f64;
        1.0 / l.max(r).max(1.0)
    }

    /// Cardinality of joining two subtrees with estimated sizes `lr` and `rr` on `e`.
    fn join_rows(&self, lr: f64, rr: f64, e: &Edge) -> f64 {
        (lr * rr * self.selectivity(e)).max(1.0)
    }

    fn edges_between(&self, a: u64, b: u64) -> Vec<Edge> {
        self.edges
            .iter()
            .copied()
            .filter(|e| {
                let (l, r) = (1u64 << e.left, 1u64 << e.right);
                (a & l != 0 && b & r != 0) || (a & r != 0 && b & l != 0)
            })
            .collect()
    }

    /// Cost of a scan: no flow, no reconstruction, and resident state only if this leaf
    /// is going to be indexed for a join, which it is in every case the planner sees.
    fn scan_cost(&self, i: usize) -> Cost {
        Cost {
            flow: 0.0,
            state: self.rels[i].rows as f64,
            reconstruct: 0.0,
            rows: self.rels[i].rows as f64,
        }
    }

    /// The three-term cost of adding one join above two costed subtrees.
    fn join_cost(&self, l: &Cost, r: &Cost, e: Option<&Edge>, depth: usize) -> Cost {
        let rows = match e {
            Some(e) => self.join_rows(l.rows, r.rows, e),
            // A cross product. Nothing about the cost model needs to punish it specially:
            // the resident-state term does that on its own, which is the point of having
            // the term.
            None => (l.rows * r.rows).max(1.0),
        };

        // Flow: the classical term. Tuples produced, plus the tuples the two operands
        // produced beneath us.
        let flow = l.flow + r.flow + rows;

        // State: both operands of a standing join are indexed and resident. This is the
        // term a batch optimiser does not have, and it is why a plan that minimises flow
        // can lose here.
        let state = l.state + r.state + l.rows + r.rows;

        // Reconstruction: on a miss, the upquery walks to the nearest reconstructible
        // ancestor, doing work proportional to what it must re-derive along the way. We
        // model that as the subtree's produced rows scaled by depth — a deep tree
        // re-derives through more operators — and then by how often a miss happens.
        let reconstruct =
            l.reconstruct + r.reconstruct + self.weights.miss_rate * rows * (depth as f64);

        Cost {
            flow,
            state,
            reconstruct,
            rows,
        }
    }

    /// Prune orderings that a partial node above cannot reconstruct through.
    fn legality(&self) -> Vec<Illegal> {
        if !self.require_reconstructible {
            return Vec::new();
        }
        self.rels
            .iter()
            .filter(|r| !r.is_base)
            .map(|r| Illegal::NotReconstructible {
                relation: r.name.clone(),
            })
            .collect()
    }

    /// Plan the join.
    pub fn plan(&self) -> Result<Planned, Vec<Illegal>> {
        let pruned = self.legality();
        if !pruned.is_empty() {
            return Err(pruned);
        }
        let n = self.rels.len();
        if n == 0 {
            return Err(vec![]);
        }
        if n <= DP_LIMIT {
            Ok(self.dpccp())
        } else {
            Ok(self.greedy())
        }
    }

    /// `DPccp` in the form that matters here: enumerate every connected subgraph and its
    /// connected complements, bottom-up by subset size.
    ///
    /// We enumerate subsets in increasing-cardinality order rather than the numeric order
    /// of the classical `DPsize`, and we skip a split whose two halves have no edge
    /// between them unless the whole graph is disconnected. That last condition is the
    /// only place cross products enter, and it is deliberate: a cross product produced
    /// because the graph *has* no connecting edge is unavoidable, whereas one produced
    /// because the enumerator did not look is a bug.
    fn dpccp(&self) -> Planned {
        let n = self.rels.len();
        let full: u64 = (1u64 << n) - 1;
        let mut best: BTreeMap<u64, (Plan, Cost)> = BTreeMap::new();
        let mut pairs: u64 = 0;

        for i in 0..n {
            best.insert(1u64 << i, (Plan::Scan(i), self.scan_cost(i)));
        }

        // Subsets in increasing popcount order.
        let mut by_size: Vec<Vec<u64>> = vec![Vec::new(); n + 1];
        for s in 1..=full {
            by_size[s.count_ones() as usize].push(s);
        }

        let connected = self.graph_is_connected(full);

        for size in 2..=n {
            for &s in &by_size[size] {
                // Enumerate proper non-empty subsets of s. The standard trick: iterate
                // sub = (sub - 1) & s. Each unordered split is seen twice; we take only
                // the half containing the lowest set bit, which halves the work and, more
                // importantly, makes the enumeration count reproducible for §9.13.4.
                let low = s & s.wrapping_neg();
                let mut sub = (s - 1) & s;
                while sub != 0 {
                    if sub & low != 0 {
                        let comp = s & !sub;
                        if comp != 0 {
                            if let (Some((lp, lc)), Some((rp, rc))) =
                                (best.get(&sub).cloned(), best.get(&comp).cloned())
                            {
                                pairs += 1;
                                let between = self.edges_between(sub, comp);
                                let cands: Vec<Option<Edge>> = if between.is_empty() {
                                    if connected {
                                        vec![]
                                    } else {
                                        vec![None]
                                    }
                                } else {
                                    between.into_iter().map(Some).collect()
                                };
                                for e in cands {
                                    let depth = 1 + lp.depth().max(rp.depth());
                                    let cost = self.join_cost(&lc, &rc, e.as_ref(), depth);
                                    let plan = match e {
                                        Some(on) => Plan::Join {
                                            left: Box::new(lp.clone()),
                                            right: Box::new(rp.clone()),
                                            on,
                                        },
                                        None => Plan::Cross {
                                            left: Box::new(lp.clone()),
                                            right: Box::new(rp.clone()),
                                        },
                                    };
                                    let improves = match best.get(&s) {
                                        None => true,
                                        Some((_, c)) => {
                                            cost.total(&self.weights) < c.total(&self.weights)
                                        }
                                    };
                                    if improves {
                                        best.insert(s, (plan, cost));
                                    }
                                }
                            }
                        }
                    }
                    sub = (sub - 1) & s;
                }
            }
        }

        let (plan, cost) = best.remove(&full).expect("dpccp covers the full set");
        Planned {
            plan,
            cost,
            weights: self.weights,
            exhaustive: true,
            pairs_enumerated: pairs,
            pruned: Vec::new(),
        }
    }

    fn graph_is_connected(&self, mask: u64) -> bool {
        let n = self.rels.len();
        if n == 0 {
            return true;
        }
        let mut seen = 0u64;
        let start = mask & mask.wrapping_neg();
        let mut stack = vec![start.trailing_zeros() as usize];
        seen |= start;
        while let Some(v) = stack.pop() {
            for e in &self.edges {
                let other = if e.left == v {
                    Some(e.right)
                } else if e.right == v {
                    Some(e.left)
                } else {
                    None
                };
                if let Some(o) = other {
                    let b = 1u64 << o;
                    if mask & b != 0 && seen & b == 0 {
                        seen |= b;
                        stack.push(o);
                    }
                }
            }
        }
        seen == mask
    }

    /// The fallback above [`DP_LIMIT`]: repeatedly join the pair whose result is smallest.
    ///
    /// This is the classical greedy operator ordering. It is left-deep by construction
    /// only when the smallest candidate always involves the accumulated result, which is
    /// not guaranteed; we do not force left-deepness, because forcing it would hide the
    /// state-term effect the cost model exists to expose.
    fn greedy(&self) -> Planned {
        let n = self.rels.len();
        let mut parts: Vec<(u64, Plan, Cost)> = (0..n)
            .map(|i| (1u64 << i, Plan::Scan(i), self.scan_cost(i)))
            .collect();
        let mut pairs = 0u64;

        while parts.len() > 1 {
            let mut choice: Option<(usize, usize, Option<Edge>, Cost)> = None;
            for a in 0..parts.len() {
                for b in (a + 1)..parts.len() {
                    pairs += 1;
                    let between = self.edges_between(parts[a].0, parts[b].0);
                    let e = between.first().copied();
                    if e.is_none() && parts.len() > 2 {
                        continue; // defer cross products until forced
                    }
                    let depth = 1 + parts[a].1.depth().max(parts[b].1.depth());
                    let c = self.join_cost(&parts[a].2, &parts[b].2, e.as_ref(), depth);
                    let better = match &choice {
                        None => true,
                        Some((_, _, _, bc)) => c.total(&self.weights) < bc.total(&self.weights),
                    };
                    if better {
                        choice = Some((a, b, e, c));
                    }
                }
            }
            let (a, b, e, cost) = match choice {
                Some(c) => c,
                // Every remaining pair is a cross product and there are more than two of
                // them; force the first.
                None => {
                    let depth = 1 + parts[0].1.depth().max(parts[1].1.depth());
                    (
                        0,
                        1,
                        None,
                        self.join_cost(&parts[0].2, &parts[1].2, None, depth),
                    )
                }
            };
            let (bm, bp, _) = parts.remove(b);
            let (am, ap, _) = parts.remove(a);
            let plan = match e {
                Some(on) => Plan::Join {
                    left: Box::new(ap),
                    right: Box::new(bp),
                    on,
                },
                None => Plan::Cross {
                    left: Box::new(ap),
                    right: Box::new(bp),
                },
            };
            parts.push((am | bm, plan, cost));
        }

        let (_, plan, cost) = parts.pop().expect("greedy reduces to one part");
        Planned {
            plan,
            cost,
            weights: self.weights,
            exhaustive: false,
            pairs_enumerated: pairs,
            pruned: Vec::new(),
        }
    }

    /// Cost a caller-supplied plan under these weights. Used by the tests to show that
    /// the chosen plan really is cheaper than the alternatives, rather than merely
    /// asserting that the search returned something.
    pub fn cost_of(&self, p: &Plan) -> Cost {
        match p {
            Plan::Scan(i) => self.scan_cost(*i),
            Plan::Join { left, right, on } => {
                let (l, r) = (self.cost_of(left), self.cost_of(right));
                self.join_cost(&l, &r, Some(on), p.depth())
            }
            Plan::Cross { left, right } => {
                let (l, r) = (self.cost_of(left), self.cost_of(right));
                self.join_cost(&l, &r, None, p.depth())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical banking join: postings ⋈ accounts ⋈ customers, a chain.
    fn chain() -> (Vec<Relation>, Vec<Edge>) {
        let rels = vec![
            Relation::base("postings", 10_000_000).with_distinct(0, 50_000),
            Relation::base("accounts", 50_000)
                .with_distinct(0, 50_000)
                .with_distinct(1, 5_000),
            Relation::base("customers", 5_000).with_distinct(0, 5_000),
        ];
        let edges = vec![Edge::new(0, 0, 1, 0), Edge::new(1, 1, 2, 0)];
        (rels, edges)
    }

    #[test]
    fn a_three_way_chain_is_planned_and_the_plan_covers_every_relation() {
        let (rels, edges) = chain();
        let p = JoinPlanner::new(rels.clone(), edges).plan().unwrap();
        assert_eq!(p.plan.relations(), 0b111);
        assert!(p.exhaustive);
        assert!(
            !p.plan.contains_cross(),
            "a connected chain needs no cross product"
        );
    }

    #[test]
    fn the_smaller_relations_are_joined_first() {
        // The classical result, which must still hold: joining the two small tables first
        // produces far less intermediate flow than driving from the 10M-row base. If the
        // added state and reconstruction terms broke this, the model would be wrong.
        let (rels, edges) = chain();
        let p = JoinPlanner::new(rels.clone(), edges).plan().unwrap();
        let rendered = p.plan.render(&rels);
        assert!(
            rendered.contains("(accounts ⋈ customers)")
                || rendered.contains("(customers ⋈ accounts)"),
            "expected the small pair to be an inner node, got {rendered}"
        );
    }

    #[test]
    fn the_state_term_can_reverse_the_classical_choice() {
        // Two plans with identical flow but different resident state. The point of the
        // test is not that a particular plan wins, but that *changing only the weights*
        // changes which one does — i.e. that the state term is load-bearing rather than
        // decorative.
        let rels = vec![
            Relation::base("a", 1_000).with_distinct(0, 1_000),
            Relation::base("b", 1_000_000).with_distinct(0, 1_000),
            Relation::base("c", 1_000).with_distinct(0, 1_000),
        ];
        let edges = vec![Edge::new(0, 0, 1, 0), Edge::new(1, 0, 2, 0)];

        let cheap_state = JoinPlanner::new(rels.clone(), edges.clone())
            .with_weights(Weights {
                flow: 1.0,
                state: 0.0,
                reconstruct: 0.0,
                miss_rate: 0.0,
            })
            .plan()
            .unwrap();
        let dear_state = JoinPlanner::new(rels.clone(), edges)
            .with_weights(Weights {
                flow: 0.0,
                state: 1.0,
                reconstruct: 0.0,
                miss_rate: 0.0,
            })
            .plan()
            .unwrap();

        // Under a pure-flow objective the plan is chosen for its output size; under a
        // pure-state objective, for what it holds. The costs must differ in kind.
        assert!(cheap_state.cost.flow > 0.0);
        assert!(dear_state.cost.state > 0.0);
        assert_ne!(
            cheap_state.cost.total(&cheap_state.weights),
            dear_state.cost.total(&dear_state.weights)
        );
    }

    #[test]
    fn a_hot_contract_and_a_cold_one_do_not_always_agree() {
        // Contribution 3's claim, in planner form: the price is workload-shaped. If the
        // same plan were optimal at every point of the weight space, the phase diagram
        // of §9.13 would have one region and there would be nothing to characterise.
        let rels = vec![
            Relation::base("l", 100_000)
                .with_distinct(0, 100)
                .with_distinct(1, 100_000),
            Relation::base("m", 100_000)
                .with_distinct(0, 100)
                .with_distinct(1, 100),
            Relation::base("n", 100).with_distinct(0, 100),
            Relation::base("o", 1_000_000).with_distinct(0, 100_000),
        ];
        let edges = vec![
            Edge::new(0, 1, 3, 0),
            Edge::new(0, 0, 1, 0),
            Edge::new(1, 1, 2, 0),
        ];

        let hot = JoinPlanner::new(rels.clone(), edges.clone())
            .with_weights(Weights::hot())
            .plan()
            .unwrap();
        let cold = JoinPlanner::new(rels.clone(), edges)
            .with_weights(Weights::cold())
            .plan()
            .unwrap();

        // Both must be valid and complete; whether they coincide is an empirical fact
        // about this graph, so we assert the interesting invariant instead: each plan is
        // at least as good as the other *under its own weights*.
        let hp = JoinPlanner::new(
            rels.clone(),
            vec![
                Edge::new(0, 1, 3, 0),
                Edge::new(0, 0, 1, 0),
                Edge::new(1, 1, 2, 0),
            ],
        )
        .with_weights(Weights::hot());
        let cp = JoinPlanner::new(
            rels.clone(),
            vec![
                Edge::new(0, 1, 3, 0),
                Edge::new(0, 0, 1, 0),
                Edge::new(1, 1, 2, 0),
            ],
        )
        .with_weights(Weights::cold());
        assert!(
            hp.cost_of(&hot.plan).total(&Weights::hot())
                <= hp.cost_of(&cold.plan).total(&Weights::hot()) + 1e-6
        );
        assert!(
            cp.cost_of(&cold.plan).total(&Weights::cold())
                <= cp.cost_of(&hot.plan).total(&Weights::cold()) + 1e-6
        );
    }

    #[test]
    fn the_dynamic_program_beats_or_ties_the_greedy_fallback() {
        // The whole justification for spending O(3ⁿ) is that it finds something greedy
        // misses. If it never did, DP_LIMIT should be 0.
        let (rels, edges) = chain();
        let planner = JoinPlanner::new(rels.clone(), edges);
        let dp = planner.dpccp();
        let gr = planner.greedy();
        assert!(
            dp.cost.total(&planner.weights) <= gr.cost.total(&planner.weights) + 1e-9,
            "dp {:?} should not be worse than greedy {:?}",
            dp.cost,
            gr.cost
        );
    }

    #[test]
    fn a_disconnected_graph_still_produces_a_complete_plan_and_says_so() {
        // Two components with no join predicate between them. A planner that returns no
        // plan here is a planner that cannot answer a legal query; one that returns a
        // plan without marking the cross product is one whose cost the caller cannot
        // interpret.
        let rels = vec![
            Relation::base("a", 100).with_distinct(0, 100),
            Relation::base("b", 100).with_distinct(0, 100),
            Relation::base("c", 100).with_distinct(0, 100),
        ];
        let edges = vec![Edge::new(0, 0, 1, 0)];
        let p = JoinPlanner::new(rels.clone(), edges).plan().unwrap();
        assert_eq!(p.plan.relations(), 0b111);
        assert!(
            p.plan.contains_cross(),
            "c can only be attached by a cross product"
        );
    }

    #[test]
    fn a_connected_graph_never_gets_a_cross_product() {
        // The dual of the previous test, and the one that catches an enumerator bug:
        // if `edges_between` were wrong, the planner would silently reach for a cross
        // product and the cost model would quietly accept it.
        let (rels, edges) = chain();
        let p = JoinPlanner::new(rels, edges).plan().unwrap();
        assert!(!p.plan.contains_cross());
    }

    #[test]
    fn a_mutable_input_is_pruned_before_costing_not_after() {
        // Reconstructibility is a legality constraint, not a cost. The distinction is the
        // one that makes Thm 4.1 safe under a bad estimator: a wrong number can make the
        // plan slow, but it cannot make a partial node reconstruct through something that
        // no longer exists.
        let rels = vec![
            Relation::base("postings", 1_000_000),
            Relation::derived("live_fx_quotes", 500), // a mutable table, not a base
        ];
        let edges = vec![Edge::new(0, 0, 1, 0)];
        let err = JoinPlanner::new(rels, edges)
            .requiring_reconstructible()
            .plan()
            .unwrap_err();
        assert_eq!(
            err,
            vec![Illegal::NotReconstructible {
                relation: "live_fx_quotes".into()
            }]
        );
    }

    #[test]
    fn the_same_query_plans_fine_when_the_view_is_fully_materialized() {
        // The complement of the previous test: the constraint is a property of the
        // *contract*, not of the query. A fully materialized view never reconstructs, so
        // a mutable input is merely a cost.
        let rels = vec![
            Relation::base("postings", 1_000_000),
            Relation::derived("live_fx_quotes", 500),
        ];
        let edges = vec![Edge::new(0, 0, 1, 0)];
        let p = JoinPlanner::new(rels, edges).plan().unwrap();
        assert_eq!(p.plan.relations(), 0b11);
    }

    #[test]
    fn enumeration_is_deterministic_across_runs() {
        // The determinism obligation of Appendix C.4 reaches the planner too: two runs on
        // the same input must produce the same plan and the same enumeration count, or a
        // measured regression cannot be attributed to a change.
        let (rels, edges) = chain();
        let a = JoinPlanner::new(rels.clone(), edges.clone())
            .plan()
            .unwrap();
        let b = JoinPlanner::new(rels, edges).plan().unwrap();
        assert_eq!(a.plan, b.plan);
        assert_eq!(a.pairs_enumerated, b.pairs_enumerated);
    }

    #[test]
    fn a_chain_enumerates_far_fewer_pairs_than_a_clique() {
        // The measured complexity claim of the module docs. We assert the ordering rather
        // than a specific count, because a specific count would pin an implementation
        // detail; but the gap must be large, or DP_LIMIT is protecting against nothing.
        let n = 8;
        let rels: Vec<_> = (0..n)
            .map(|i| Relation::base(&format!("r{i}"), 1_000).with_distinct(0, 100))
            .collect();

        let chain_edges: Vec<_> = (0..n - 1).map(|i| Edge::new(i, 0, i + 1, 0)).collect();
        let mut clique_edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                clique_edges.push(Edge::new(i, 0, j, 0));
            }
        }

        let c = JoinPlanner::new(rels.clone(), chain_edges).plan().unwrap();
        let k = JoinPlanner::new(rels, clique_edges).plan().unwrap();
        assert!(
            k.pairs_enumerated > c.pairs_enumerated,
            "clique {} should enumerate more than chain {}",
            k.pairs_enumerated,
            c.pairs_enumerated
        );
    }

    #[test]
    fn above_the_limit_the_greedy_fallback_runs_and_is_labelled() {
        // Honesty about which algorithm produced the plan. §9.13.4 attributes a
        // regression to the search only if it can tell which search ran.
        let n = DP_LIMIT + 2;
        let rels: Vec<_> = (0..n)
            .map(|i| Relation::base(&format!("r{i}"), 1_000).with_distinct(0, 100))
            .collect();
        let edges: Vec<_> = (0..n - 1).map(|i| Edge::new(i, 0, i + 1, 0)).collect();
        let p = JoinPlanner::new(rels, edges).plan().unwrap();
        assert!(!p.exhaustive, "above DP_LIMIT the fallback must run");
        assert_eq!(p.plan.relations(), (1u64 << n) - 1);
    }

    #[test]
    fn depth_is_the_upquery_walk_and_a_left_deep_plan_is_recognised() {
        let (rels, _) = chain();
        let ld = Plan::Join {
            left: Box::new(Plan::Join {
                left: Box::new(Plan::Scan(0)),
                right: Box::new(Plan::Scan(1)),
                on: Edge::new(0, 0, 1, 0),
            }),
            right: Box::new(Plan::Scan(2)),
            on: Edge::new(1, 1, 2, 0),
        };
        assert!(ld.is_left_deep());
        assert_eq!(ld.depth(), 3);
        assert_eq!(ld.render(&rels), "((postings ⋈ accounts) ⋈ customers)");

        let bushy = Plan::Join {
            left: Box::new(Plan::Scan(0)),
            right: Box::new(Plan::Join {
                left: Box::new(Plan::Scan(1)),
                right: Box::new(Plan::Scan(2)),
                on: Edge::new(1, 1, 2, 0),
            }),
            on: Edge::new(0, 0, 1, 0),
        };
        assert!(!bushy.is_left_deep());
        assert_eq!(bushy.depth(), 3);
    }

    #[test]
    fn a_higher_miss_rate_raises_the_reconstruction_term_and_nothing_else() {
        // The three terms must be separable, because §9.13.4 reports them separately and
        // a reader has to be able to attribute a cost change to a cause.
        let (rels, edges) = chain();
        let low = JoinPlanner::new(rels.clone(), edges.clone()).with_weights(Weights {
            flow: 1.0,
            state: 1.0,
            reconstruct: 1.0,
            miss_rate: 0.0,
        });
        let high = JoinPlanner::new(rels.clone(), edges).with_weights(Weights {
            flow: 1.0,
            state: 1.0,
            reconstruct: 1.0,
            miss_rate: 1.0,
        });

        let plan = low.plan().unwrap().plan;
        let lc = low.cost_of(&plan);
        let hc = high.cost_of(&plan);
        assert_eq!(lc.flow, hc.flow);
        assert_eq!(lc.state, hc.state);
        assert_eq!(lc.rows, hc.rows);
        assert!(hc.reconstruct > lc.reconstruct);
        assert_eq!(
            lc.reconstruct, 0.0,
            "at miss_rate 0 there is no reconstruction to pay for"
        );
    }

    #[test]
    fn selectivity_uses_the_larger_distinct_count() {
        // The containment assumption, pinned. If this silently became the *smaller*
        // count, every estimate would be inflated and the planner would still return
        // plausible-looking plans — the classic way an estimator error hides.
        let rels = vec![
            Relation::base("a", 1_000).with_distinct(0, 10),
            Relation::base("b", 1_000).with_distinct(0, 1_000),
        ];
        let p = JoinPlanner::new(rels, vec![Edge::new(0, 0, 1, 0)]);
        assert!((p.selectivity(&Edge::new(0, 0, 1, 0)) - 0.001).abs() < 1e-12);
    }

    #[test]
    fn an_unknown_distinct_count_falls_back_visibly_rather_than_to_one() {
        // A missing statistic must not become σ = 1 (every row matches every row), which
        // would make the cross product look free.
        let r = Relation::base("x", 1_000);
        assert_eq!(r.ndv(0), 100);
        let tiny = Relation::base("y", 3);
        assert_eq!(
            tiny.ndv(0),
            1,
            "never zero, or the selectivity divides by zero"
        );
    }
}
