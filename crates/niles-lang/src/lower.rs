//! Lowering: typed tree → circuit.
//!
//! This is the step that closes the loop. Above it, the language; below it, the engine.
//! The circuit is the stable contract between the two, so lowering's job is to lose
//! *syntax* and keep *semantics* — every anchor, contract, key and provenance annotation
//! that a theorem in Chapters 3–4 quantifies over has to survive, because an IR that
//! erased them would leave the proofs talking about a different object.
//!
//! # One IR, two surfaces
//!
//! The pipeline surface and the SQL surface lower to the *same* nodes. That is not an
//! implementation convenience — it is the generality claim made operational. If
//! `postings.group_by(..).sum(..)` and `select acct, sum(amt) from postings group by acct`
//! produced different circuits, then "the SQL surface is the same language" would be
//! marketing. A test asserts they produce identical node sequences.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::effects::Rung;
use crate::lexer::Span;
use crate::resolve::{Catalog, RelationInfo, Staleness};
use niles_ir::circuit::{Circuit, NodeId};
use niles_ir::operator::{Agg, ApplyKind, ColIdx, JoinKind as IrJoin, Op, Scalar, ScalarOp};
use niles_ir::{Consistency, Lineage, Materialize, Retention, ServeContract};
use std::collections::HashMap;

pub struct Lowered {
    pub circuit: Circuit,
    /// Column names per node, so `explain` can print names rather than indices.
    pub schemas: HashMap<NodeId, Vec<String>>,
}

pub fn lower_program(prog: &Program, cat: &Catalog) -> (Lowered, Diagnostics) {
    let mut lx = Lx {
        cat,
        circuit: Circuit::new(),
        schemas: HashMap::new(),
        d: Diagnostics::new(),
        sources: HashMap::new(),
    };
    for item in &prog.items {
        lx.item(item);
    }
    (
        Lowered {
            circuit: lx.circuit,
            schemas: lx.schemas,
        },
        lx.d,
    )
}

struct Lx<'a> {
    cat: &'a Catalog,
    circuit: Circuit,
    schemas: HashMap<NodeId, Vec<String>>,
    d: Diagnostics,
    /// Memoised source nodes: one node per relation, however many views read it. Sharing
    /// matters — two source nodes for one ledger would maintain the same deltas twice.
    sources: HashMap<String, NodeId>,
}

impl<'a> Lx<'a> {
    fn item(&mut self, item: &Item) {
        match item {
            Item::Schema(s) => {
                for si in &s.items {
                    if let SchemaItem::View(v) = si {
                        self.view(v);
                    }
                }
            }
            Item::Mod { items, .. } => items.iter().for_each(|i| self.item(i)),
            Item::View(v) => self.view(v),
            _ => {}
        }
    }

    fn view(&mut self, v: &ViewDecl) {
        let contract = self.contract_of(&v.name.text);
        let before = self.d.items.len();
        let Some(id) = self.expr(&v.body, contract) else {
            if self.d.items.len() > before {
                // Something more specific already said why. A second, vaguer message about
                // the same view is noise that pushes the useful one off the screen.
                return;
            }
            self.d.push(
                Diagnostic::error("NL0500", format!("view `{}` has no lowering", v.name.text))
                    .primary(v.body.span(), "this expression is not a pipeline over a declared relation")
                    .note("a view body must reduce to a chain of stages rooted in a base, ledger, table or another view"),
            );
            return;
        };
        // The view's own contract is attached to its output node, replacing the inherited
        // one. This is the node the engine consults when a read arrives.
        self.circuit.nodes[id as usize].contract =
            niles_ir::circuit::Checked::new("contract", contract);
        self.circuit.set_output(v.name.text.clone(), id);
    }

    fn contract_of(&self, view: &str) -> ServeContract {
        let Some(info) = self.cat.views.get(view) else {
            return niles_ir::circuit::internal_contract();
        };
        ServeContract {
            consistency: match info.rung {
                // The numbers the *source* gave, which is the whole content of a bounded
                // contract. This arm used to read `{ epochs: 4, millis: 1_000 }`
                // unconditionally, so a view written `bounded(epochs: 8)` was served at
                // four and the IR held no trace that eight had ever been asked for.
                Rung::Bounded => {
                    let s = info.staleness.unwrap_or(Staleness::UNPARAMETERISED);
                    Consistency::Bounded {
                        epochs: s.epochs,
                        millis: s.millis,
                    }
                }
                Rung::Monotonic => Consistency::Monotonic,
                Rung::ReadYourWrites => Consistency::ReadYourWrites,
                Rung::Snapshot => Consistency::Snapshot,
                Rung::Serializable => Consistency::Serializable,
                Rung::LedgerConsistent => Consistency::LedgerConsistent,
            },
            materialize: match info.materialize.as_str() {
                "absent" => Materialize::Absent,
                "demand" => Materialize::Demand,
                "full" => Materialize::Full,
                "spilled" => Materialize::Spilled,
                "tiered" => Materialize::Tiered,
                _ => Materialize::Auto,
            },
            retain: match info.retain.as_str() {
                "pinned" => Retention::Pinned,
                "forever" => Retention::Forever,
                _ => Retention::Evictable,
            },
            lineage: match info.lineage.as_str() {
                "key" => Lineage::Key,
                "full" => Lineage::Full,
                _ => Lineage::Off,
            },
        }
    }

    fn schema_of(&self, id: NodeId) -> &[String] {
        self.schemas.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    fn col_index(&self, id: NodeId, name: &str) -> Option<ColIdx> {
        self.schema_of(id)
            .iter()
            .position(|c| c == name)
            .map(|i| i as ColIdx)
    }

    fn expr(&mut self, e: &Expr, c: ServeContract) -> Option<NodeId> {
        match e {
            Expr::Path(p) => self.source(&p.last().text, c),
            Expr::Stage {
                recv,
                kind,
                name,
                args,
                ..
            } => {
                let input = self.expr(recv, c)?;
                self.stage(input, *kind, name, args, c)
            }
            Expr::Fixpoint { recv, measure, .. } => {
                let input = self.expr(recv, c)?;
                // The measure is what lets the runtime stop, so a measure the lowering
                // cannot express is the one thing a fixpoint may not be given a default
                // for. `LitInt(0)` is a measure that never decreases, which is a
                // termination argument that proves nothing — and a fixpoint that could
                // diverge stalls an epoch, and a stalled epoch stalls the visibility
                // timeline for every reader in the system.
                let Some(m) = self.scalar(input, measure) else {
                    self.d.push(
                        Diagnostic::error("NL0507", "this fixpoint's measure has no lowering")
                            .primary(measure.span(), "cannot be expressed in the circuit")
                            .note("the measure is the termination argument; a constant one is not an argument")
                            .note("guarded recursion is a requirement of the operator set, not a convenience: an unbounded fixpoint stalls the epoch and every reader waiting on it"),
                    );
                    return None;
                };
                let id = self.circuit.add(
                    Op::Fixpoint {
                        measure: m,
                        max_rounds: 1_000,
                    },
                    vec![input],
                    c,
                    "fixpoint",
                );
                self.schemas.insert(id, self.schema_of(input).to_vec());
                Some(id)
            }
            Expr::Sql { inner, .. } => self.expr(inner, c),
            Expr::Select(s) => self.select(s, c),
            _ => None,
        }
    }

    /// A source node, memoised. Views are sources too: reading a view is a read of its
    /// output node, which is how rung monotonicity has anything to be monotone over.
    fn source(&mut self, name: &str, _c: ServeContract) -> Option<NodeId> {
        if let Some(id) = self.sources.get(name) {
            return Some(*id);
        }
        if let Some(id) = self.circuit.outputs.get(name) {
            return Some(*id);
        }
        let rel: &RelationInfo = self.cat.relations.get(name)?;
        let cols: Vec<String> = rel.columns.iter().map(|c| c.name.clone()).collect();
        // The anchor index, translated into column indices. Its presence is what makes the
        // view upqueryable; its absence is a warning the resolver already emitted.
        let anchor_key: Vec<ColIdx> = rel
            .anchor_indices
            .first()
            .map(|ix| {
                ix.iter()
                    .filter_map(|c| cols.iter().position(|x| x == c).map(|i| i as ColIdx))
                    .collect()
            })
            .unwrap_or_default();
        let contract = ServeContract {
            // A base is never partial and is read at the frontier: it *is* the frontier.
            consistency: Consistency::LedgerConsistent,
            materialize: Materialize::Full,
            retain: if rel.is_base() {
                Retention::Forever
            } else {
                Retention::Pinned
            },
            lineage: Lineage::Key,
        };
        let id = self.circuit.add(
            Op::Source {
                relation: name.to_string(),
                is_base: rel.is_base(),
                anchor_key,
            },
            vec![],
            contract,
            name,
        );
        self.schemas.insert(id, cols);
        self.sources.insert(name.to_string(), id);
        Some(id)
    }

    fn stage(
        &mut self,
        input: NodeId,
        kind: StageKind,
        name: &Name,
        args: &[Arg],
        c: ServeContract,
    ) -> Option<NodeId> {
        let in_schema = self.schema_of(input).to_vec();
        let (op, out_schema): (Op, Vec<String>) = match kind {
            StageKind::Where | StageKind::Having => {
                // The pipeline surface, which used to read
                // `.unwrap_or(Scalar::LitBool(true))` — the same defect the SQL surface's
                // `predicate` helper was written to remove, left in place on the other
                // path. Thesis §11.5.7 claimed the site was pinned; only the SQL half was,
                // and `postings.where(|p| p.nonexistent == 1)` lowered to
                // `Filter{LitBool(true)}` and returned every posting.
                let clause = if kind == StageKind::Where {
                    "where"
                } else {
                    "having"
                };
                let arg = self.required_arg(args, name.span, clause, "a predicate")?;
                let p = self.predicate(input, &arg, clause)?;
                (Op::Filter { predicate: p }, in_schema.clone())
            }
            StageKind::Map | StageKind::Select => {
                let arg = self.required_arg(args, name.span, "map", "a projection")?;
                let exprs: Vec<Scalar> = self.scalar_list(input, &arg)?;
                let names = (0..exprs.len().max(1)).map(|i| format!("c{i}")).collect();
                (Op::Map { exprs }, names)
            }
            StageKind::GroupBy => {
                // A `group_by` alone is not an aggregate; it establishes the key, and the
                // aggregate that follows consumes it. Lowering it to an `Index` keeps the
                // two-stage surface honest without inventing an aggregate nobody wrote.
                let key = self.key_of(input, args);
                (Op::Index { key }, in_schema.clone())
            }
            StageKind::Sum
            | StageKind::Count
            | StageKind::Min
            | StageKind::Max
            | StageKind::Avg => {
                let agg = match kind {
                    StageKind::Sum => Agg::Sum,
                    StageKind::Count => Agg::Count,
                    StageKind::Min => Agg::Min,
                    StageKind::Max => Agg::Max,
                    _ => Agg::Avg,
                };
                // The grouping key comes from the `Index` node immediately upstream, if
                // there is one; otherwise this is a global aggregate.
                let group_key = match &self.circuit.node(input).op {
                    Op::Index { key } => key.clone(),
                    _ => self.circuit.node(input).key.clone().unwrap_or_default(),
                };
                // The aggregated expression. `unwrap_or(Scalar::Column(0))` summed
                // whatever happened to be in the first column — an account id, a
                // timestamp — and reported it as a balance.
                let arg =
                    self.required_arg(args, name.span, agg.as_str(), "the value to aggregate")?;
                let value = match self.scalar(input, &arg) {
                    Some(v) => v,
                    None => {
                        self.d.push(
                            Diagnostic::error(
                                "NL0502",
                                format!("the expression `{}` aggregates has no lowering", agg.as_str()),
                            )
                            .primary(arg.span(), "cannot be expressed in the circuit")
                            .note("there is no safe default: aggregating the first column instead would report some other quantity under this aggregate's name"),
                        );
                        return None;
                    }
                };
                let mut names: Vec<String> = group_key
                    .iter()
                    .filter_map(|i| in_schema.get(*i as usize).cloned())
                    .collect();
                names.push(agg.as_str().to_string());
                // Fold the aggregate into the upstream Index rather than stacking a node
                // on it: `group_by(k).sum(v)` is one operator, and emitting two would make
                // the SQL and pipeline surfaces produce different circuits for one query.
                if matches!(self.circuit.node(input).op, Op::Index { .. }) {
                    let real_input = self.circuit.node(input).inputs[0];
                    let id = self.circuit.add(
                        Op::Aggregate {
                            group_key,
                            aggs: vec![(agg, value)],
                        },
                        vec![real_input],
                        c,
                        name.text.clone(),
                    );
                    self.schemas.insert(id, names);
                    return Some(id);
                }
                (
                    Op::Aggregate {
                        group_key,
                        aggs: vec![(agg, value)],
                    },
                    names,
                )
            }
            StageKind::Join
            | StageKind::LeftJoin
            | StageKind::RightJoin
            | StageKind::FullOuterJoin
            | StageKind::CrossJoin => {
                let rhs = args.first().and_then(|a| self.expr(&a.value, c))?;
                let jk = match kind {
                    StageKind::LeftJoin => IrJoin::LeftOuter,
                    StageKind::RightJoin => IrJoin::RightOuter,
                    StageKind::FullOuterJoin => IrJoin::FullOuter,
                    _ => IrJoin::Inner,
                };
                let residual = args.get(1).and_then(|a| self.scalar(input, &a.value));
                let lk = self.circuit.node(input).key.clone().unwrap_or_default();
                // The right key defaulting to the left key joined two relations on
                // whatever column positions the left happened to use. Position agreement
                // is not key agreement, and the resulting join was silently wrong rather
                // than empty.
                let rk = match self.circuit.node(rhs).key.clone() {
                    Some(k) => k,
                    None => {
                        self.d.push(
                            Diagnostic::error("NL0503", "the right side of this join has no key")
                                .primary(name.span, "nothing says which columns to join on")
                                .note("the left side's key is not a default: two relations agreeing on a column *position* is not the same as agreeing on a column")
                                .note("declare an anchor index on the right relation, or group it by the join key first"),
                        );
                        return None;
                    }
                };
                let mut names = in_schema.clone();
                names.extend(self.schema_of(rhs).iter().cloned());
                let id = self.circuit.add(
                    Op::Join {
                        kind: jk,
                        left_key: lk,
                        right_key: rk,
                        residual,
                    },
                    vec![input, rhs],
                    c,
                    name.text.clone(),
                );
                self.schemas.insert(id, names);
                return Some(id);
            }
            StageKind::Union | StageKind::UnionAll => {
                let rhs = args.first().and_then(|a| self.expr(&a.value, c))?;
                let id = self
                    .circuit
                    .add(Op::Union, vec![input, rhs], c, name.text.clone());
                self.schemas.insert(id, in_schema.clone());
                if kind == StageKind::Union {
                    let d = self.circuit.add(Op::Distinct, vec![id], c, "distinct");
                    self.schemas.insert(d, in_schema);
                    return Some(d);
                }
                return Some(id);
            }
            StageKind::Except => {
                let rhs = args.first().and_then(|a| self.expr(&a.value, c))?;
                let neg = self.circuit.add(Op::Negate, vec![rhs], c, "negate");
                self.schemas.insert(neg, self.schema_of(rhs).to_vec());
                let id = self
                    .circuit
                    .add(Op::Union, vec![input, neg], c, name.text.clone());
                self.schemas.insert(id, in_schema);
                return Some(id);
            }
            StageKind::Intersect => {
                // Z-set intersection as a semi-join: keep the left rows that have a match,
                // without duplicating them. Expressing it as a join rather than a
                // dedicated operator keeps the operator set closed, which matters because
                // every theorem quantifies over that set.
                let rhs = args.first().and_then(|a| self.expr(&a.value, c))?;
                let lk = self.circuit.node(input).key.clone().unwrap_or_default();
                let rk = match self.circuit.node(rhs).key.clone() {
                    Some(k) => k,
                    None => {
                        self.d.push(
                            Diagnostic::error("NL0503", "the right side of this `intersect` has no key")
                                .primary(name.span, "nothing says which columns to match on")
                                .note("the left side's key is not a default: two relations agreeing on a column *position* is not the same as agreeing on a column"),
                        );
                        return None;
                    }
                };
                let id = self.circuit.add(
                    Op::Join {
                        kind: IrJoin::Semi,
                        left_key: lk,
                        right_key: rk,
                        residual: None,
                    },
                    vec![input, rhs],
                    c,
                    name.text.clone(),
                );
                self.schemas.insert(id, in_schema);
                return Some(id);
            }
            StageKind::Distinct | StageKind::DistinctBy => (Op::Distinct, in_schema.clone()),
            StageKind::OrderBy => {
                let keys = self
                    .key_of(input, args)
                    .into_iter()
                    .map(|k| (k, true))
                    .collect();
                (Op::OrderBy { keys }, in_schema.clone())
            }
            StageKind::Limit => {
                // A non-literal bound became `u64::MAX`, so `limit(n)` returned
                // everything. The row count a view returns is not a place for a default:
                // "all of them" is the answer least likely to be noticed as wrong and the
                // most expensive when it is.
                let n = self.literal_count(args, name.span, "limit")?;
                (
                    Op::Limit {
                        count: n,
                        offset: 0,
                    },
                    in_schema.clone(),
                )
            }
            StageKind::Offset => {
                let n = self.literal_count(args, name.span, "offset")?;
                (
                    Op::Limit {
                        count: u64::MAX,
                        offset: n,
                    },
                    in_schema.clone(),
                )
            }
            StageKind::AsOf => {
                // No argument means the frontier, which is a real reading. An argument the
                // lowering cannot read is not: it used to fall through to the frontier too,
                // so `as_of(#4200)` written with a typo silently became `as_of(now)` — the
                // one answer an as-of read must never give.
                let e = match args.first().map(|a| &a.value) {
                    None => None,
                    Some(Expr::Epoch(v, _)) => Some(*v),
                    Some(other) => {
                        self.d.push(
                            Diagnostic::error("NL0510", "`as_of` takes an epoch")
                                .primary(other.span(), "not an epoch literal")
                                .note("the system axis is ordered by epoch, and an epoch is written `#4200`")
                                .note("omitting the argument reads at the frontier; an argument that cannot be read does not")
                        );
                        return None;
                    }
                };
                (Op::AsOf { epoch: e }, in_schema.clone())
            }
            StageKind::ValidAt => {
                // This arm was `Op::ValidAt { instant: None }` unconditionally: whatever
                // date was written, the circuit pinned to nothing. The valid-time axis is
                // half of the bitemporality claim and it was being discarded at the door.
                let i = match args.first().map(|a| &a.value) {
                    None => None,
                    Some(Expr::Instant { text, span, .. }) => {
                        match niles_ir::value::days_since_epoch(text) {
                            Some(d) => Some(d),
                            None => {
                                self.d.push(
                                    Diagnostic::error("NL0509", format!("`{text}` is not a date"))
                                        .primary(*span, "expected `YYYY-MM-DD`"),
                                );
                                return None;
                            }
                        }
                    }
                    Some(other) => {
                        self.d.push(
                            Diagnostic::error("NL0510", "`valid_at` takes a date")
                                .primary(other.span(), "not a valid-time literal")
                                .note("the world axis is written `v@2026-08-01`"),
                        );
                        return None;
                    }
                };
                (Op::ValidAt { instant: i }, in_schema.clone())
            }
            StageKind::Get | StageKind::Range => {
                let key = self.circuit.node(input).key.clone().unwrap_or_default();
                (Op::Index { key }, in_schema.clone())
            }
            // `Fold` and `Fixpoint` reach here only as *stages*; the `Expr::Fixpoint` form
            // is lowered in `expr`. Neither has an operator, and the old arm returned the
            // input unchanged — an identity, so a `fold` that computed something and a
            // stage the parser did not recognise both produced a circuit that quietly did
            // less than the source said.
            StageKind::Fold | StageKind::Fixpoint | StageKind::Unknown => {
                self.d.push(
                    Diagnostic::error(
                        "NL0505",
                        format!("the stage `{}` has no lowering", name.text),
                    )
                    .primary(name.span, "not an operator the circuit has")
                    .note("this used to lower to the identity, which is a circuit that silently does less than the source says")
                    .note("the operator set is closed on purpose: every theorem in the thesis quantifies over it, so a stage outside it is a stage outside the results"),
                );
                return None;
            }
        };
        let id = self.circuit.add(op, vec![input], c, name.text.clone());
        self.schemas.insert(id, out_schema);
        Some(id)
    }

    /// The argument a stage requires, or a diagnostic saying which stage wanted what.
    ///
    /// Every caller used to write `args.first().and_then(..).unwrap_or(<something>)`, so a
    /// stage written with no argument at all lowered to a node with a fabricated one.
    fn required_arg(&mut self, args: &[Arg], at: Span, stage: &str, wanted: &str) -> Option<Expr> {
        match args.first() {
            Some(a) => Some(a.value.clone()),
            None => {
                self.d.push(
                    Diagnostic::error("NL0500", format!("`{stage}` takes {wanted}"))
                        .primary(at, "no argument given")
                        .note("there is no default for this: a stage with a fabricated argument computes something nobody wrote"),
                );
                None
            }
        }
    }

    /// A row-count bound, which must be a literal.
    fn literal_count(&mut self, args: &[Arg], at: Span, stage: &str) -> Option<u64> {
        match args.first().map(|a| &a.value) {
            Some(Expr::Int(v, _)) if *v >= 0 => Some(*v as u64),
            Some(Expr::Int(v, sp)) => {
                self.d.push(
                    Diagnostic::error("NL0504", format!("`{stage}` cannot take {v}"))
                        .primary(*sp, "a negative row count")
                        .note("a bound on rows is a natural number; a negative one has no reading"),
                );
                None
            }
            Some(other) => {
                self.d.push(
                    Diagnostic::error("NL0504", format!("`{stage}` requires a literal bound"))
                        .primary(other.span(), "not a literal")
                        .note("this used to become `u64::MAX`, so a `limit` the lowering could not read returned every row — the answer least likely to be noticed as wrong and the most expensive when it is")
                        .note("a view boundary must be reproducible: an evicted entry reconstructed against a different bound would hold a different number"),
                );
                None
            }
            None => {
                self.d.push(
                    Diagnostic::error("NL0504", format!("`{stage}` takes a bound"))
                        .primary(at, "no argument given"),
                );
                None
            }
        }
    }

    /// The column indices a `|r| (r.a, r.b)` closure names.
    fn key_of(&self, input: NodeId, args: &[Arg]) -> Vec<ColIdx> {
        let mut names = Vec::new();
        if let Some(a) = args.first() {
            collect_field_names(&a.value, &mut names);
        }
        names
            .iter()
            .filter_map(|n| self.col_index(input, n))
            .collect()
    }

    /// One scalar. Returns `None` for shapes the IR has no form for; the caller supplies a
    /// conservative default rather than silently dropping the expression, because a
    /// dropped predicate is a filter that does not filter.
    fn scalar(&mut self, input: NodeId, e: &Expr) -> Option<Scalar> {
        Some(match e {
            Expr::Closure { body, .. } => self.scalar(input, body)?,
            Expr::Int(v, _) => Scalar::LitInt(*v),
            Expr::Bool(v, _) => Scalar::LitBool(*v),
            Expr::Str(s, _) => Scalar::LitText(s.clone()),
            Expr::Money {
                minor, currency, ..
            } => {
                // The currency travels with the value into the IR. An index into the
                // catalog's currency list, so the engine never has to parse a string to
                // know which money it is holding.
                //
                // `unwrap_or(0)` made an undeclared currency into *whichever currency
                // happens to sort first* — so a literal written in an unknown currency
                // entered the circuit denominated in a real one. The typechecker reports
                // the undeclared currency (NL0241); this refuses to lower it, because a
                // circuit that survived the error would carry the wrong denomination.
                let Some(ix) = self.cat.currencies.keys().position(|k| k == &currency.text) else {
                    self.d.push(
                        Diagnostic::error(
                            "NL0506",
                            format!("currency `{}` is not declared, so this literal has no lowering", currency.text),
                        )
                        .primary(e.span(), "unknown currency")
                        .note("this used to become the first declared currency, which silently redenominated the literal"),
                    );
                    return None;
                };
                Scalar::LitMoney {
                    minor: *minor,
                    currency: ix as u32,
                }
            }
            // A date literal is an integer in the IR — days since 1970-01-01, read by the
            // one function both this and `valid_at` use. Without a form here,
            // `where p.value_date >= v@2026-08-01` had no lowering, and on the pipeline
            // surface that meant `Filter{LitBool(true)}`: the worked example's
            // `balance_as_of_2026_q1` returned every posting in the ledger and called
            // itself a balance as of March.
            Expr::Instant { text, span, .. } => match niles_ir::value::days_since_epoch(text) {
                Some(d) => Scalar::LitInt(d as i128),
                None => {
                    self.d.push(
                        Diagnostic::error("NL0509", format!("`{text}` is not a date"))
                            .primary(*span, "expected `YYYY-MM-DD`")
                            .note("dates are not read leniently: a literal parsed into the wrong day is worse than a compile error, because nothing downstream can tell"),
                    );
                    return None;
                }
            },
            Expr::Epoch(v, _) => Scalar::LitInt(*v as i128),
            Expr::Field { name, .. } => Scalar::Column(self.col_index(input, &name.text)?),
            Expr::Path(p) => Scalar::Column(self.col_index(input, &p.last().text)?),
            Expr::Unary {
                op: UnOp::Not,
                operand,
                ..
            } => Scalar::Not(Box::new(self.scalar(input, operand)?)),
            Expr::Unary {
                op: UnOp::Neg,
                operand,
                ..
            } => Scalar::Neg(Box::new(self.scalar(input, operand)?)),
            Expr::Binary { op, lhs, rhs, .. } => {
                let sop = match op {
                    BinOp::Add => ScalarOp::Add,
                    BinOp::Sub => ScalarOp::Sub,
                    BinOp::Mul => ScalarOp::Mul,
                    BinOp::Div => ScalarOp::Div,
                    BinOp::Rem => ScalarOp::Rem,
                    BinOp::Eq => ScalarOp::Eq,
                    BinOp::Ne => ScalarOp::Ne,
                    BinOp::Lt => ScalarOp::Lt,
                    BinOp::Le => ScalarOp::Le,
                    BinOp::Gt => ScalarOp::Gt,
                    BinOp::Ge => ScalarOp::Ge,
                    BinOp::And => ScalarOp::And,
                    BinOp::Or => ScalarOp::Or,
                    BinOp::Like => ScalarOp::Like,
                    _ => return None,
                };
                Scalar::Binary {
                    op: sop,
                    lhs: Box::new(self.scalar(input, lhs)?),
                    rhs: Box::new(self.scalar(input, rhs)?),
                }
            }
            Expr::Call { callee, args, .. } => {
                // An unresolved call becomes a UDF node rather than being dropped. It is
                // uncertified by default, which is what makes the upquery-path derivation
                // refuse to reconstruct through it.
                let name = match &**callee {
                    Expr::Path(p) => p.last().text.clone(),
                    _ => return None,
                };
                let id = name
                    .bytes()
                    .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
                Scalar::Udf {
                    id,
                    args: args
                        .iter()
                        .filter_map(|a| self.scalar(input, &a.value))
                        .collect(),
                }
            }
            _ => return None,
        })
    }

    /// Lower an expression that is being used as a **predicate**.
    ///
    /// Separate from [`Lx::scalar`] for one reason, and it is a correctness reason rather
    /// than a tidiness one. These three call sites used to read
    /// `self.scalar(cur, f).unwrap_or(Scalar::LitBool(true))`: a predicate the lowering
    /// could not express became the constant `true`, so the `where` clause was silently
    /// discarded and the view returned every row. The query looked correct, the plan
    /// verified, and no answer-level test could catch it, because every row it returned
    /// was a real row.
    ///
    /// That is the `Err(_) => 0` defect of §1.1.1 at plan level, and it was found by
    /// writing `where t.z = 1` and reading the circuit. There is no safe default here:
    /// `true` returns rows that should have been filtered out and `false` hides rows that
    /// exist, so the only honest behaviour is to refuse to lower the view and say which
    /// predicate could not be expressed.
    fn predicate(&mut self, input: NodeId, e: &Expr, clause: &str) -> Option<Scalar> {
        match self.scalar(input, e) {
            Some(p) => Some(p),
            None => {
                self.d.push(
                    Diagnostic::error(
                        "NL0501",
                        format!("this `{clause}` predicate has no lowering"),
                    )
                    .primary(e.span(), "cannot be expressed in the circuit")
                    .note(
                        "a predicate that cannot be lowered is not defaulted to `true` or \
                             `false`: one would return rows that should have been filtered out \
                             and the other would hide rows that exist",
                    ),
                );
                None
            }
        }
    }

    /// A projection list, all of it or none of it.
    ///
    /// The `filter_map` this replaces dropped every element it could not lower, so
    /// `map(|p| (p.acct, f(p.amt)))` produced a one-column projection and the money column
    /// was gone — with no diagnostic, and a circuit that verified.
    fn scalar_list(&mut self, input: NodeId, e: &Expr) -> Option<Vec<Scalar>> {
        match e {
            Expr::Closure { body, .. } => self.scalar_list(input, body),
            Expr::Tuple { elems, .. } => {
                let mut out = Vec::with_capacity(elems.len());
                for (i, x) in elems.iter().enumerate() {
                    match self.scalar(input, x) {
                        Some(s) => out.push(s),
                        None => {
                            self.d.push(
                                Diagnostic::error(
                                    "NL0508",
                                    format!("element {} of this projection has no lowering", i + 1),
                                )
                                .primary(x.span(), "cannot be expressed in the circuit")
                                .note("the whole projection is refused rather than the element dropped: a projection missing a column produces a view with a different schema from the one that was written"),
                            );
                            return None;
                        }
                    }
                }
                Some(out)
            }
            other => match self.scalar(input, other) {
                Some(s) => Some(vec![s]),
                None => {
                    self.d.push(
                        Diagnostic::error("NL0508", "this projection has no lowering")
                            .primary(other.span(), "cannot be expressed in the circuit"),
                    );
                    None
                }
            },
        }
    }

    // ---------------- the SQL surface ----------------

    /// Lower a `select` to the same nodes the pipeline surface produces.
    ///
    /// SQL's *written* order is not its evaluation order, and this is where that is
    /// resolved: `from`, then `where`, then `group by`/aggregate, then `having`, then
    /// `order by`, then `limit`. The pipeline surface writes that order out explicitly,
    /// which is the argument for it; the SQL surface writes SQL's order and is rearranged
    /// here.
    fn select(&mut self, s: &SelectStmt, c: ServeContract) -> Option<NodeId> {
        let mut cur = self.table_ref(s.from.first()?, c)?;
        if let Some(f) = &s.filter {
            // Subquery predicates become `Apply` nodes in the pipeline; whatever is left
            // becomes the `where` filter. Splitting first is what lets a correlated
            // `exists` reach the optimizer's unnesting rules at all.
            let mut subqueries = Vec::new();
            let residual = split_subqueries(f, &mut subqueries);
            for sq in subqueries {
                cur = self.apply(cur, &sq, c)?;
            }
            if let Some(r) = residual {
                let p = self.predicate(cur, &r, "where")?;
                let id = self
                    .circuit
                    .add(Op::Filter { predicate: p }, vec![cur], c, "where");
                self.schemas.insert(id, self.schema_of(cur).to_vec());
                cur = id;
            }
        }
        if !s.group_by.is_empty() {
            let in_schema = self.schema_of(cur).to_vec();
            let mut names = Vec::new();
            for g in &s.group_by {
                collect_field_names(g, &mut names);
            }
            let group_key: Vec<ColIdx> = names
                .iter()
                .filter_map(|n| self.col_index(cur, n))
                .collect();
            // The aggregate in the projection list.
            let mut aggs = Vec::new();
            for (e, _) in &s.projections {
                if let Expr::Call { callee, args, .. } = e {
                    if let Expr::Path(p) = &**callee {
                        let agg = match p.last().text.as_str() {
                            "sum" => Some(Agg::Sum),
                            "count" => Some(Agg::Count),
                            "min" => Some(Agg::Min),
                            "max" => Some(Agg::Max),
                            "avg" => Some(Agg::Avg),
                            _ => None,
                        };
                        if let Some(a) = agg {
                            let v = args
                                .first()
                                .and_then(|x| self.scalar(cur, &x.value))
                                .unwrap_or(Scalar::Column(0));
                            aggs.push((a, v));
                        }
                    }
                }
            }
            let mut out: Vec<String> = group_key
                .iter()
                .filter_map(|i| in_schema.get(*i as usize).cloned())
                .collect();
            out.extend(aggs.iter().map(|(a, _)| a.as_str().to_string()));
            let id = self
                .circuit
                .add(Op::Aggregate { group_key, aggs }, vec![cur], c, "group by");
            self.schemas.insert(id, out);
            cur = id;
        }
        if let Some(h) = &s.having {
            let p = self.predicate(cur, h, "having")?;
            let id = self
                .circuit
                .add(Op::Filter { predicate: p }, vec![cur], c, "having");
            self.schemas.insert(id, self.schema_of(cur).to_vec());
            cur = id;
        }
        if !s.order_by.is_empty() {
            let mut names = Vec::new();
            for (e, _) in &s.order_by {
                collect_field_names(e, &mut names);
            }
            let keys: Vec<(ColIdx, bool)> = s
                .order_by
                .iter()
                .zip(names.iter())
                .filter_map(|((_, asc), n)| self.col_index(cur, n).map(|i| (i, *asc)))
                .collect();
            let id = self
                .circuit
                .add(Op::OrderBy { keys }, vec![cur], c, "order by");
            self.schemas.insert(id, self.schema_of(cur).to_vec());
            cur = id;
        }
        if let Some(Expr::Int(n, _)) = &s.limit {
            let off = match &s.offset {
                Some(Expr::Int(o, _)) => *o as u64,
                _ => 0,
            };
            let id = self.circuit.add(
                Op::Limit {
                    count: *n as u64,
                    offset: off,
                },
                vec![cur],
                c,
                "limit",
            );
            self.schemas.insert(id, self.schema_of(cur).to_vec());
            cur = id;
        }
        Some(cur)
    }

    /// Lower one subquery predicate into an [`Op::Apply`] over `outer`.
    ///
    /// The correlation is read out of the subquery's own `where` clause: an equality whose
    /// two sides resolve, one in the outer schema and one in the inner, is a correlation
    /// pair; everything else stays as a filter on the inner side. That split is what makes
    /// the operator's `correlation` field meaningful, and it is deliberately *syntactic* —
    /// a semantic notion of "depends on the outer row" would need a solver.
    fn apply(&mut self, outer: NodeId, sq: &Subquery, c: ServeContract) -> Option<NodeId> {
        let inner_from = sq.query.from.first()?;
        let mut inner = self.table_ref(inner_from, c)?;

        let mut correlation = Vec::new();
        let mut local = Vec::new();
        if let Some(w) = &sq.query.filter {
            for conj in conjuncts(w) {
                match self.as_correlation(outer, inner, conj) {
                    Some(pair) => correlation.push(pair),
                    None => local.push(conj),
                }
            }
        }
        for l in local {
            let p = self.predicate(inner, l, "subquery where")?;
            let id = self.circuit.add(
                Op::Filter { predicate: p },
                vec![inner],
                c,
                "subquery where",
            );
            self.schemas.insert(id, self.schema_of(inner).to_vec());
            inner = id;
        }

        let kind = match &sq.kind {
            SubqueryKind::Exists => ApplyKind::Exists,
            SubqueryKind::NotExists => ApplyKind::NotExists,
            SubqueryKind::In(probe) | SubqueryKind::NotIn(probe) => {
                let p = self.col_of(outer, probe)?;
                // The subquery's single projection is the column being probed against. A
                // subquery projecting more than one column is a different construct (a row
                // comparison) and is refused rather than approximated.
                if sq.query.projections.len() != 1 {
                    self.d.push(
                        Diagnostic::error("NL0502", "an `in` subquery must project exactly one column")
                            .primary(sq.query.span, format!("this projects {}", sq.query.projections.len()))
                            .note("a multi-column `in` is a row comparison, which is a different construct"),
                    );
                    return None;
                }
                let icol = self.col_of(inner, &sq.query.projections[0].0)?;
                if matches!(sq.kind, SubqueryKind::In(_)) {
                    ApplyKind::In {
                        probe: p,
                        inner: icol,
                    }
                } else {
                    ApplyKind::NotIn {
                        probe: p,
                        inner: icol,
                    }
                }
            }
        };

        let label = format!("{} subquery", kind.name());
        let id = self.circuit.add(
            Op::Apply { kind, correlation },
            vec![outer, inner],
            c,
            label,
        );
        // An `Apply` in these four kinds preserves the outer row exactly, so it preserves
        // the outer schema. A scalar subquery would widen it; that surface form is not
        // reachable yet and would need this line to change with it.
        self.schemas.insert(id, self.schema_of(outer).to_vec());
        Some(id)
    }

    /// A correlation pair, if `e` is `outer.a = inner.b` (in either order).
    ///
    /// The qualifier decides, and it has to. `where u.k = t.k` is the commonest correlated
    /// predicate there is, and both sides are called `k`: an unqualified rule that asked
    /// "which schema does this name resolve in" would find it resolves in both, give up,
    /// and leave the equality as a *filter on the inner side* — where it lowers to
    /// `k = k`, which is always true. The subquery would then match every row and the
    /// `exists` would be a no-op. So the relation name is read off the side it belongs to,
    /// and the unqualified rule is only the fallback for when there is no qualifier to read.
    fn as_correlation(&self, outer: NodeId, inner: NodeId, e: &Expr) -> Option<(ColIdx, ColIdx)> {
        let Expr::Binary {
            op: BinOp::Eq,
            lhs,
            rhs,
            ..
        } = e
        else {
            return None;
        };
        let (ln, rn) = (leaf_name(lhs)?, leaf_name(rhs)?);
        let inner_rel = self.relation_of(inner);

        if let (Some(lq), Some(rq)) = (qualifier(lhs), qualifier(rhs)) {
            if Some(&lq) == inner_rel.as_ref() && Some(&rq) != inner_rel.as_ref() {
                return Some((self.col_index(outer, &rn)?, self.col_index(inner, &ln)?));
            }
            if Some(&rq) == inner_rel.as_ref() && Some(&lq) != inner_rel.as_ref() {
                return Some((self.col_index(outer, &ln)?, self.col_index(inner, &rn)?));
            }
            // Both sides name the same relation: a local predicate, not a correlation.
            return None;
        }

        // No qualifier on one or both sides. Fall back to resolution: each side must
        // resolve on exactly one of the two schemas, or "which relation did you mean" is a
        // question the lowering would be guessing the answer to.
        let (lo, li) = (self.col_index(outer, &ln), self.col_index(inner, &ln));
        let (ro, ri) = (self.col_index(outer, &rn), self.col_index(inner, &rn));
        match (lo, li, ro, ri) {
            (Some(o), None, None, Some(i)) => Some((o, i)),
            (None, Some(i), Some(o), None) => Some((o, i)),
            _ => None,
        }
    }

    /// The base relation a node reads from, following single-input operators down to the
    /// first `Source`. `None` for a join or a union, where "the relation" is not a
    /// question with one answer.
    fn relation_of(&self, mut id: NodeId) -> Option<String> {
        loop {
            let n = self.circuit.node(id);
            match &n.op {
                Op::Source { relation, .. } => return Some(relation.clone()),
                _ if n.inputs.len() == 1 => id = n.inputs[0],
                _ => return None,
            }
        }
    }

    fn col_of(&self, id: NodeId, e: &Expr) -> Option<ColIdx> {
        self.col_index(id, &leaf_name(e)?)
    }

    fn table_ref(&mut self, t: &TableRef, c: ServeContract) -> Option<NodeId> {
        match t {
            TableRef::Named { name, .. } => self.source(&name.text, c),
            TableRef::Sub { query, .. } => self.select(query, c),
            TableRef::Join {
                left,
                right,
                kind,
                on,
                ..
            } => {
                let l = self.table_ref(left, c)?;
                let r = self.table_ref(right, c)?;
                let jk = match kind {
                    JoinKind::Left => IrJoin::LeftOuter,
                    JoinKind::Right => IrJoin::RightOuter,
                    JoinKind::Full => IrJoin::FullOuter,
                    _ => IrJoin::Inner,
                };
                let residual = on.as_ref().and_then(|o| self.scalar(l, o));
                let lk = self.circuit.node(l).key.clone().unwrap_or_default();
                let rk = self
                    .circuit
                    .node(r)
                    .key
                    .clone()
                    .unwrap_or_else(|| lk.clone());
                let mut names = self.schema_of(l).to_vec();
                names.extend(self.schema_of(r).iter().cloned());
                let id = self.circuit.add(
                    Op::Join {
                        kind: jk,
                        left_key: lk,
                        right_key: rk,
                        residual,
                    },
                    vec![l, r],
                    c,
                    "join",
                );
                self.schemas.insert(id, names);
                Some(id)
            }
        }
    }
}

/// A subquery predicate lifted out of a `where` clause.
struct Subquery {
    kind: SubqueryKind,
    query: SelectStmt,
}

enum SubqueryKind {
    Exists,
    NotExists,
    /// The probe expression from the outer row.
    In(Expr),
    NotIn(Expr),
}

/// The conjuncts of an `and`-chain, left to right.
fn conjuncts(e: &Expr) -> Vec<&Expr> {
    match e {
        Expr::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            let mut v = conjuncts(lhs);
            v.extend(conjuncts(rhs));
            v
        }
        other => vec![other],
    }
}

/// Split a `where` clause into subquery predicates and whatever is left.
///
/// Only **top-level conjuncts** are lifted. A subquery under an `or` or under a `not` that
/// is not `not exists` stays where it is, and will then fail to lower with NL0501 naming
/// it — which is the right outcome: an `Apply` is a pipeline node and cannot be one arm of
/// a disjunction without first being turned into a semi-join and a union, which is a
/// different rewrite and is not implemented. Refusing loudly beats lowering something that
/// is not the query that was written.
fn split_subqueries(e: &Expr, out: &mut Vec<Subquery>) -> Option<Expr> {
    let mut residual: Option<Expr> = None;
    for conj in conjuncts(e) {
        match conj {
            Expr::Exists { query, .. } => out.push(Subquery {
                kind: SubqueryKind::Exists,
                query: (**query).clone(),
            }),
            Expr::Unary {
                op: UnOp::Not,
                operand,
                ..
            } => match &**operand {
                Expr::Exists { query, .. } => out.push(Subquery {
                    kind: SubqueryKind::NotExists,
                    query: (**query).clone(),
                }),
                _ => residual = Some(and_with(residual, conj)),
            },
            Expr::Binary {
                op: op @ (BinOp::In | BinOp::NotIn),
                lhs,
                rhs,
                ..
            } => match &**rhs {
                Expr::Select(q) => {
                    let probe = (**lhs).clone();
                    let kind = if *op == BinOp::In {
                        SubqueryKind::In(probe)
                    } else {
                        SubqueryKind::NotIn(probe)
                    };
                    out.push(Subquery {
                        kind,
                        query: (**q).clone(),
                    });
                }
                // `x in (1, 2, 3)` is a list membership test, not a subquery, and lowers
                // as an ordinary scalar.
                _ => residual = Some(and_with(residual, conj)),
            },
            other => residual = Some(and_with(residual, other)),
        }
    }
    residual
}

fn and_with(acc: Option<Expr>, e: &Expr) -> Expr {
    match acc {
        None => e.clone(),
        Some(a) => Expr::Binary {
            op: BinOp::And,
            span: a.span().to(e.span()),
            lhs: Box::new(a),
            rhs: Box::new(e.clone()),
        },
    }
}

/// The relation a qualified field names: `t.k` gives `t`, and a bare `k` gives nothing.
fn qualifier(e: &Expr) -> Option<String> {
    match e {
        Expr::Field { base, .. } => match &**base {
            Expr::Path(p) if p.segments.len() == 1 => Some(p.segments[0].text.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// The name a path or field expression ends in: `t.k` and `k` both give `k`.
fn leaf_name(e: &Expr) -> Option<String> {
    match e {
        Expr::Field { name, .. } => Some(name.text.clone()),
        Expr::Path(p) => Some(p.last().text.clone()),
        _ => None,
    }
}

fn collect_field_names(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Field { name, .. } => out.push(name.text.clone()),
        Expr::Path(p) => out.push(p.last().text.clone()),
        Expr::Tuple { elems, .. } => elems.iter().for_each(|x| collect_field_names(x, out)),
        Expr::Closure { body, .. } => collect_field_names(body, out),
        _ => {}
    }
}
