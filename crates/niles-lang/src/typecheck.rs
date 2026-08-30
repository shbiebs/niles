//! The body-level checker: where the currency-row solver and the effect calculus meet
//! actual source text.
//!
//! [`crate::resolve`] discharged the declaration-level rules. This phase walks function
//! and view bodies and discharges the rest: W4, W6, W7, W8, W14, W15, W16 and W17 of the
//! grammar's section 15, plus rung monotonicity.
//!
//! # What this checker is, and what it is not
//!
//! It is **not** a Hindley–Milner inference engine, and pretending otherwise would be
//! dishonest about the artifact. It is a *money-shaped* analysis: every expression is
//! classified into a small lattice — a monetary value in some currency, a linear ledger
//! half, or something else — and the interesting judgements are made over that
//! classification. That is sufficient for the four claims Contribution 4 rests on, because
//! all four are about money, currency, linearity and rung, and none is about polymorphic
//! recursion over user types.
//!
//! The boundary is stated rather than hidden: where the analysis cannot see through an
//! expression it reports *undecided* and discharges the obligation to the runtime, and
//! [`Report::runtime_obligations`] counts how often that happened. A checker that silently
//! assumed the good case for anything it could not follow would make the soundness theorem
//! vacuous.

use crate::ast::*;
use crate::currency_rows::{self as rows, Amount, Cur, Row as CurRow, Unifier, Verdict};
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::effects::{self, Effect, LinearKind, LinearValue, Row as EffRow, Rung};
use crate::lexer::Span;
use crate::resolve::Catalog;
use std::collections::HashMap;

/// The classification an expression is given. Small on purpose.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// A monetary value: a currency (possibly a variable) and a symbolic amount.
    Money(Cur, Amount),
    /// A linear ledger half or hold, which must be consumed exactly once.
    Linear(LinearKind),
    /// Anything else the checker does not need to look inside.
    Opaque,
}

/// What a check concluded, beyond the diagnostics.
#[derive(Debug, Default)]
pub struct Report {
    /// Conservation obligations the checker proved statically.
    pub conservation_proved: usize,
    /// Obligations it could not decide, and handed to the runtime. Reported, never hidden.
    pub runtime_obligations: usize,
    /// Effect rows inferred per function, for `nilesc effects` and for lowering.
    pub inferred_effects: HashMap<String, EffRow>,
    /// The weakest rung each view body reads at, which is the ceiling on what it may promise.
    pub view_rungs: HashMap<String, Option<Rung>>,
}

pub fn check_program(prog: &Program, cat: &Catalog) -> (Report, Diagnostics) {
    let mut cx = Cx {
        cat,
        d: Diagnostics::new(),
        report: Report::default(),
        unifier: Unifier::new(),
        next_symbol: 0,
    };
    for item in &prog.items {
        cx.item(item);
    }
    (cx.report, cx.d)
}

struct Cx<'a> {
    cat: &'a Catalog,
    d: Diagnostics,
    report: Report,
    unifier: Unifier,
    next_symbol: u32,
}

/// Per-scope state: the currency row being accumulated, the effects incurred, and the
/// linear values in flight.
#[derive(Default)]
struct Scope {
    row: CurRow,
    effects: EffRow,
    linear: Vec<LinearValue>,
    /// Set inside a `txn` or an `fx` leg, where a conservation obligation is open.
    conserving: bool,
    /// What each `let`-bound name and parameter is, in the money-shaped lattice.
    ///
    /// Without this the checker sees `debit(tenant, rent)` and cannot tell what currency
    /// `rent` is, so it invents a variable and then reports the function's declared
    /// `debit<usd>` as unsatisfied. The resulting diagnostic is technically true and
    /// completely useless, which is the usual shape of an under-powered checker's output.
    bindings: HashMap<String, Shape>,
}

impl Scope {
    /// A child scope that inherits bindings but opens its own conservation obligation.
    /// Bindings flow in because a `txn` block sees the `let`s above it; the row does not,
    /// because the obligation belongs to the transaction.
    fn child_conserving(&self) -> Scope {
        Scope { conserving: true, bindings: self.bindings.clone(), ..Default::default() }
    }
}

impl<'a> Cx<'a> {
    fn fresh_symbol(&mut self) -> u32 {
        self.next_symbol += 1;
        self.next_symbol
    }

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
            Item::Fn(f) => self.function(f),
            Item::Impl(i) => {
                let fns = i.items.clone();
                fns.iter().for_each(|f| self.function(f));
            }
            _ => {}
        }
    }

    // ---------------- functions ----------------

    fn function(&mut self, f: &FnDecl) {
        let Some(body) = &f.body else { return };
        let mut sc = Scope::default();
        // Seed the scope from the signature. A parameter declared `Money<usd>` is money in
        // USD everywhere in the body, which is the point of writing the type.
        for p in &f.params {
            if let (Some(cur), Some(name)) = (money_currency_of(&p.ty), first_binding(&p.pat)) {
                let sym = self.fresh_symbol();
                let c = match cur {
                    Some(c) => Cur::Known(c),
                    None => self.unifier.fresh(),
                };
                sc.bindings.insert(name, Shape::Money(c, Amount::symbol(sym)));
            }
        }
        // Capabilities a function holds are the `Auth<E>` parameters it takes. They cannot
        // be constructed, only received or granted, which is what makes them unforgeable.
        let mut caps = EffRow::new();
        for p in &f.params {
            if let Ty::Path { path, args, .. } = &p.ty {
                if path.last().text == "Auth" {
                    if let Some(Ty::Path { path: e, args: eargs, .. }) = args.first() {
                        let names: Vec<String> = eargs
                            .iter()
                            .filter_map(|t| match t {
                                Ty::Path { path, .. } => Some(path.last().text.clone()),
                                _ => None,
                            })
                            .collect();
                        if let Some(eff) = Effect::parse(&e.last().text, None, &names) {
                            caps.add(eff, p.span);
                        }
                    }
                }
            }
        }
        self.block(body, &mut sc);

        let decl_span = f.effects.as_ref().map(|r| r.span).unwrap_or(f.name.span);
        // W14: the inferred row must be permitted by the declared one.
        if let Some(declared) = &f.effects {
            let declared_row = to_eff_row(declared);
            for diag in effects::check_declaration(&f.name.text, &sc.effects, &declared_row, decl_span) {
                self.d.push(diag);
            }
        } else {
            // Undeclared effects are a warning, not an error: inference exists, and forcing
            // an annotation on every helper would make the language tiresome for no safety
            // gain. But a function that appends or debits and does not say so is exactly
            // the function a reviewer needs to be able to find.
            let interesting: Vec<String> = sc
                .effects
                .iter()
                .filter(|e| !matches!(e, Effect::Read(_)))
                .map(|e| e.to_string())
                .collect();
            if !interesting.is_empty() {
                self.d.push(
                    Diagnostic::warning(
                        "NL0313",
                        format!("`{}` has undeclared effects {}", f.name.text, sc.effects),
                    )
                    .primary(f.name.span, "no effect row declared")
                    .note("an effect row is part of the signature: a caller should be able to see from the type that this function moves money")
                    .suggest(
                        f.name.span,
                        format!("! {{ {} }}", interesting.join(", ")),
                        "declare the inferred row",
                        crate::diagnostics::Applicability::MachineApplicable,
                    ),
                );
            }
        }

        // W15/W16: authority.
        for diag in effects::check_authority(&f.name.text, &sc.effects, &caps, decl_span) {
            self.d.push(diag);
        }
        // W8: linearity.
        for diag in effects::check_linearity(&sc.linear) {
            self.d.push(diag);
        }
        self.report.inferred_effects.insert(f.name.text.clone(), sc.effects);
    }

    // ---------------- views ----------------

    fn view(&mut self, v: &ViewDecl) {
        let mut sc = Scope::default();
        self.expr(&v.body, &mut sc);
        let Some(info) = self.cat.views.get(&v.name.text) else { return };
        // Rung monotonicity: the judgement that makes "two derived views of one ledger
        // disagreeing at the moment a decision is made" unspellable.
        for diag in effects::check_rung_monotonicity(
            &v.name.text,
            info.rung,
            &sc.effects,
            v.body.span(),
            info.contract_span,
        ) {
            self.d.push(diag);
        }
        self.report.view_rungs.insert(v.name.text.clone(), sc.effects.weakest_read());
    }

    // ---------------- statements ----------------

    fn block(&mut self, b: &Block, sc: &mut Scope) {
        for s in &b.stmts {
            self.stmt(s, sc);
        }
        if let Some(t) = &b.tail {
            self.expr(t, sc);
        }
    }

    fn stmt(&mut self, s: &Stmt, sc: &mut Scope) {
        match s {
            Stmt::Let { pat, ty, init, .. } => {
                let mut shape = init.as_ref().map(|e| self.expr(e, sc)).unwrap_or(Shape::Opaque);
                // An explicit annotation wins over inference: `let x: Money<jpy> = f();`
                // is the programmer telling the checker something it could not see.
                if let Some(Some(cur)) = ty.as_ref().map(money_currency_of) {
                    let sym = self.fresh_symbol();
                    let c = match cur {
                        Some(c) => Cur::Known(c),
                        None => self.unifier.fresh(),
                    };
                    shape = Shape::Money(c, Amount::symbol(sym));
                }
                if let Shape::Money(..) = &shape {
                    for n in pat.bindings() {
                        sc.bindings.insert(n.text.clone(), shape.clone());
                    }
                }
                if let Shape::Linear(kind) = shape {
                    for n in pat.bindings() {
                        sc.linear.push(LinearValue {
                            name: n.text.clone(),
                            kind,
                            bound_at: n.span,
                            uses: Vec::new(),
                        });
                    }
                }
            }
            Stmt::Expr(e) | Stmt::Semi(e) => {
                self.expr(e, sc);
            }
            Stmt::Item(i) => self.item(i),
            Stmt::Dml(dml) => self.dml(dml, sc),
            Stmt::Error(_) => {}
        }
    }

    fn dml(&mut self, dml: &Dml, sc: &mut Scope) {
        // W4: `update` and `delete` are legal against a `table` only. This is the single
        // most important static difference between a base and a table, and it is what
        // "the base is never partial" means at the level of a program.
        let mutation = match dml {
            Dml::Update { table, span, .. } => Some((table, *span, "update")),
            Dml::Delete { table, span, .. } => Some((table, *span, "delete")),
            Dml::Insert { table, span, .. } => Some((table, *span, "insert")),
            _ => None,
        };
        if let Some((name, span, verb)) = mutation {
            if let Some(rel) = self.cat.relations.get(&name.text) {
                if rel.is_base() {
                    let kind = if rel.kind == RelKind::Ledger { "ledger" } else { "base" };
                    let alt = if verb == "insert" {
                        "append to it inside a `txn { .. }`"
                    } else {
                        "append a compensating entry"
                    };
                    self.d.push(
                        Diagnostic::error("NL0230", format!("`{verb}` is not legal against {kind} `{}`", name.text))
                            .primary(span, format!("`{}` is immutable and fully retained", name.text))
                            .secondary(rel.span, format!("declared as a {kind} here"))
                            .note("history is the authority: a fact that can be edited is not evidence, and reconstruction over an edited base is not reproduction")
                            .note(format!("to change the world, {alt}")),
                    );
                }
            }
            sc.effects.add(if verb == "insert" { Effect::Append } else { Effect::Mutate }, span);
        }
        match dml {
            Dml::Grant { span, .. } | Dml::Revoke { span, .. } | Dml::Backfill { span, .. } => {
                sc.effects.add(Effect::Admin, *span)
            }
            Dml::Emit { span, .. } => sc.effects.add(Effect::Emit, *span),
            _ => {}
        }
    }

    // ---------------- expressions ----------------

    fn expr(&mut self, e: &Expr, sc: &mut Scope) -> Shape {
        match e {
            // ---- money ----
            Expr::Money { minor, scale, currency, span } => {
                // W6: the literal's own scale must equal the currency's declared scale.
                if let Some(info) = self.cat.currencies.get(&currency.text) {
                    if info.scale != *scale {
                        self.d.push(
                            Diagnostic::error(
                                "NL0240",
                                format!("this literal has {scale} decimal places but `{}` has scale {}", currency.text, info.scale),
                            )
                            .primary(*span, format!("written with {scale} decimal places"))
                            .secondary(info.span, format!("`{}` declared with scale {}", currency.text, info.scale))
                            .note("the scale is part of the type; rounding a money literal silently is how a reconciliation breaks months later with no evidence of where"),
                        );
                    }
                } else {
                    self.d.push(
                        Diagnostic::error("NL0241", format!("currency `{}` is not declared", currency.text))
                            .primary(*span, "unknown currency")
                            .note("declare it with its minor-unit scale: `currency xyz { scale: 2 }`"),
                    );
                }
                Shape::Money(Cur::Known(currency.text.clone()), Amount::constant(*minor))
            }

            // ---- the conserving forms ----
            Expr::Txn { body, span, .. } => {
                let mut inner = sc.child_conserving();
                self.block(body, &mut inner);
                sc.effects.union(&inner.effects);
                sc.effects.add(Effect::Append, *span);
                sc.linear.append(&mut inner.linear);
                let mut row = std::mem::take(&mut inner.row);
                self.discharge_conservation(&mut row, *span, None);
                Shape::Opaque
            }

            Expr::Fx { legs, rate, span } => {
                // Two rows, each required to be zero. There is no single currency in which
                // a cross-currency transaction sums to zero, which is precisely why this is
                // a form and not a pair of transfers.
                for (name, leg) in legs {
                    let mut inner = sc.child_conserving();
                    self.expr(leg, &mut inner);
                    sc.effects.union(&inner.effects);
                    sc.linear.append(&mut inner.linear);
                    let mut row = std::mem::take(&mut inner.row);
                    self.discharge_conservation(&mut row, name.span, Some(&name.text));
                }
                if let Some(r) = rate {
                    self.expr(r, sc);
                } else {
                    self.d.push(
                        Diagnostic::error("NL0242", "an `fx` form must record its rate")
                            .primary(*span, "no `rate:` given")
                            .note("the rate is what makes the two legs auditable as one conversion rather than two unrelated transactions"),
                    );
                }
                sc.effects.add(Effect::Append, *span);
                Shape::Opaque
            }

            // ---- calls, including the linear constructors ----
            Expr::Call { callee, args, span } => {
                let name = match &**callee {
                    Expr::Path(p) => p.last().text.clone(),
                    other => {
                        self.expr(other, sc);
                        String::new()
                    }
                };
                let shapes: Vec<Shape> = args.iter().map(|a| self.expr(&a.value, sc)).collect();
                self.call(&name, &shapes, args, *span, sc)
            }

            Expr::Hold { args, span } => {
                let shapes: Vec<Shape> = args.iter().map(|a| self.expr(&a.value, sc)).collect();
                let cur = shapes
                    .iter()
                    .find_map(|s| match s {
                        Shape::Money(c, _) => Some(c.to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "*".into());
                sc.effects.add(Effect::Hold(cur), *span);
                sc.effects.add(Effect::Append, *span);
                Shape::Linear(LinearKind::Hold)
            }

            Expr::Resolve { hold, outcome, span } => {
                self.consume_linear(hold, sc, *span);
                if let ResolveOutcome::Post(a) = outcome {
                    self.expr(a, sc);
                }
                sc.effects.add(Effect::Append, *span);
                Shape::Opaque
            }

            Expr::Authorize { args, span } => {
                let shapes: Vec<Shape> = args.iter().map(|a| self.expr(&a.value, sc)).collect();
                let cur = shapes
                    .iter()
                    .find_map(|s| match s {
                        Shape::Money(c, _) => Some(c.to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "*".into());
                // W16: the only construct that may reduce a balance below zero. That it
                // *requires* authority is checked in `effects::check_authority`; what
                // happens here is that the effect is recorded, which is what makes that
                // check possible at all.
                sc.effects.add(Effect::Authorize(cur), *span);
                Shape::Opaque
            }

            Expr::Declassify { args, span } => {
                args.iter().for_each(|a| {
                    self.expr(&a.value, sc);
                });
                sc.effects.add(Effect::Declassify, *span);
                Shape::Opaque
            }

            // ---- reads ----
            Expr::Stage { recv, kind, args, span, .. } => {
                self.expr(recv, sc);
                if let Expr::Path(p) = &**recv {
                    if let Some(rung) = self.cat.source_rung(&p.last().text) {
                        sc.effects.add(Effect::Read(rung), p.span);
                    }
                }
                for a in args {
                    self.expr(&a.value, sc);
                }
                // W17: a confidential column may not appear in a predicate, a key or an
                // aggregate — the engine cannot compute on ciphertext, so a filter over an
                // encrypted column would either leak through timing or silently not filter.
                if matches!(kind, StageKind::Where | StageKind::GroupBy | StageKind::Sum | StageKind::Having) {
                    self.check_confidential_use(recv, args, *span);
                }
                Shape::Opaque
            }

            Expr::Path(p) => {
                let name = &p.last().text;
                if let Some(rung) = self.cat.source_rung(name) {
                    sc.effects.add(Effect::Read(rung), p.span);
                }
                sc.bindings.get(name).cloned().unwrap_or(Shape::Opaque)
            }

            Expr::Fixpoint { recv, step, measure, .. } => {
                self.expr(recv, sc);
                self.expr(step, sc);
                self.expr(measure, sc);
                Shape::Opaque
            }

            // ---- arithmetic: where currency mismatch is caught ----
            Expr::Binary { op, lhs, rhs, span } => {
                let (a, b) = (self.expr(lhs, sc), self.expr(rhs, sc));
                self.binary(*op, a, b, *span, lhs.span(), rhs.span())
            }

            Expr::Unary { op, operand, .. } => {
                let s = self.expr(operand, sc);
                match (op, s) {
                    (UnOp::Neg, Shape::Money(c, a)) => Shape::Money(c, a.neg()),
                    (_, s) => s,
                }
            }

            // ---- everything else: recurse ----
            Expr::Block(b) => {
                self.block(b, sc);
                Shape::Opaque
            }
            Expr::If { cond, then, els, .. } => {
                self.expr(cond, sc);
                self.block(then, sc);
                if let Some(e) = els {
                    self.expr(e, sc);
                }
                Shape::Opaque
            }
            Expr::Case { arms, els, .. } => {
                for (c, v) in arms {
                    self.expr(c, sc);
                    self.expr(v, sc);
                }
                if let Some(e) = els {
                    self.expr(e, sc);
                }
                Shape::Opaque
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee, sc);
                for a in arms {
                    if let Some(g) = &a.guard {
                        self.expr(g, sc);
                    }
                    self.expr(&a.body, sc);
                }
                Shape::Opaque
            }
            Expr::While { cond, body, .. } => {
                self.expr(cond, sc);
                self.block(body, sc);
                Shape::Opaque
            }
            Expr::Loop { body, .. } => {
                self.block(body, sc);
                Shape::Opaque
            }
            Expr::For { iter, body, .. } => {
                self.expr(iter, sc);
                self.block(body, sc);
                Shape::Opaque
            }
            Expr::Closure { body, .. } => {
                self.expr(body, sc);
                Shape::Opaque
            }
            Expr::Try { expr, .. } | Expr::Cast { expr, .. } => self.expr(expr, sc),
            Expr::Field { base, .. } | Expr::Index { base, .. } => {
                self.expr(base, sc);
                Shape::Opaque
            }
            Expr::Tuple { elems, .. } | Expr::Array { elems, .. } => {
                elems.iter().for_each(|x| {
                    self.expr(x, sc);
                });
                Shape::Opaque
            }
            Expr::StructLit { fields, .. } => {
                fields.iter().for_each(|(_, v)| {
                    self.expr(v, sc);
                });
                Shape::Opaque
            }
            Expr::Assign { target, value, .. } => {
                self.expr(target, sc);
                self.expr(value, sc);
                Shape::Opaque
            }
            Expr::Return { value: Some(v), .. } => self.expr(v, sc),
            Expr::Explain { target, span } | Expr::Impact { target, span } => {
                self.expr(target, sc);
                sc.effects.add(Effect::Read(Rung::Snapshot), *span);
                Shape::Opaque
            }
            Expr::Reproduce { target, at, span } => {
                self.expr(target, sc);
                if let Some(a) = at {
                    self.expr(a, sc);
                }
                sc.effects.add(Effect::Read(Rung::LedgerConsistent), *span);
                Shape::Opaque
            }
            Expr::Sql { inner, .. } => self.expr(inner, sc),
            Expr::Select(s) => {
                self.select(s, sc);
                Shape::Opaque
            }
            _ => Shape::Opaque,
        }
    }

    fn select(&mut self, s: &SelectStmt, sc: &mut Scope) {
        for t in &s.from {
            self.table_ref(t, sc);
        }
        for (e, _) in &s.projections {
            self.expr(e, sc);
        }
        if let Some(f) = &s.filter {
            self.expr(f, sc);
        }
        for g in &s.group_by {
            self.expr(g, sc);
        }
        if let Some(h) = &s.having {
            self.expr(h, sc);
        }
        if let Some((_, next)) = &s.set_op {
            let n = next.clone();
            self.select(&n, sc);
        }
    }

    fn table_ref(&mut self, t: &TableRef, sc: &mut Scope) {
        match t {
            TableRef::Named { name, .. } => {
                if let Some(r) = self.cat.source_rung(&name.text) {
                    sc.effects.add(Effect::Read(r), name.span);
                }
            }
            TableRef::Join { left, right, on, .. } => {
                self.table_ref(left, sc);
                self.table_ref(right, sc);
                if let Some(o) = on {
                    self.expr(o, sc);
                }
            }
            TableRef::Sub { query, .. } => {
                let q = query.clone();
                self.select(&q, sc);
            }
        }
    }

    /// The library calls the checker knows the meaning of. Everything else is opaque, and
    /// contributes an undecided symbol rather than an assumption.
    fn call(&mut self, name: &str, shapes: &[Shape], args: &[Arg], span: Span, sc: &mut Scope) -> Shape {
        let money = shapes.iter().find_map(|s| match s {
            Shape::Money(c, a) => Some((c.clone(), a.clone())),
            _ => None,
        });
        match name {
            "debit" | "credit" => {
                let is_debit = name == "debit";
                let (c, a) = match money {
                    Some(x) => x,
                    None => {
                        let s = self.fresh_symbol();
                        (self.unifier.fresh(), Amount::symbol(s))
                    }
                };
                let eff = if is_debit { Effect::Debit(c.to_string()) } else { Effect::Credit(c.to_string()) };
                sc.effects.add(eff, span);
                if sc.conserving {
                    sc.row.movement(c, if is_debit { a.neg() } else { a }, span);
                }
                Shape::Linear(if is_debit { LinearKind::Debit } else { LinearKind::Credit })
            }
            "post" => {
                // `post` is what consumes the linear halves.
                for a in args {
                    self.consume_linear(&a.value, sc, span);
                }
                sc.effects.add(Effect::Append, span);
                Shape::Opaque
            }
            _ => {
                // An unknown call producing money contributes an *undecided* amount: the
                // checker cannot see the value, so it records a fresh symbol rather than
                // assuming zero. This is the mechanism that keeps `is_decided` honest.
                if let Some((c, _)) = money {
                    let s = self.fresh_symbol();
                    return Shape::Money(c, Amount::symbol(s));
                }
                Shape::Opaque
            }
        }
    }

    fn binary(&mut self, op: BinOp, a: Shape, b: Shape, span: Span, lsp: Span, rsp: Span) -> Shape {
        let (Shape::Money(ca, aa), Shape::Money(cb, ab)) = (&a, &b) else {
            return Shape::Opaque;
        };
        match op {
            BinOp::Add | BinOp::Sub => {
                // W7: no operator joins two distinct currencies. This is the "cannot
                // mismatch currencies" clause, decided by unification.
                match self.unifier.unify(ca, cb) {
                    Ok(c) => {
                        let amt = if op == BinOp::Add { aa.add(ab) } else { aa.add(&ab.neg()) };
                        Shape::Money(c, amt)
                    }
                    Err((x, y)) => {
                        self.d.push(
                            Diagnostic::error(
                                "NL0250",
                                format!("cannot {} `Money<{x}>` and `Money<{y}>`", if op == BinOp::Add { "add" } else { "subtract" }),
                            )
                            .primary(span, format!("`{x}` and `{y}` are different currencies"))
                            .secondary(lsp, format!("this is `Money<{x}>`"))
                            .secondary(rsp, format!("this is `Money<{y}>`"))
                            .note("money is closed under addition, subtraction and integer scaling *within* a currency; nothing joins two")
                            .note("to move value between currencies, use an `fx { leg .., leg .., rate: .. }` form, which conserves each currency separately and records the rate")
                            .suggest(span, "fx { leg a: .., leg b: .., rate: r }", "use an `fx` form: two conserved legs and a recorded rate", crate::diagnostics::Applicability::HasPlaceholders),
                        );
                        Shape::Opaque
                    }
                }
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if let Err((x, y)) = self.unifier.unify(ca, cb) {
                    self.d.push(
                        Diagnostic::error("NL0251", format!("cannot compare `Money<{x}>` with `Money<{y}>`"))
                            .primary(span, "different currencies are not ordered relative to each other")
                            .secondary(lsp, format!("`Money<{x}>`"))
                            .secondary(rsp, format!("`Money<{y}>`"))
                            .note("there is no exchange-rate-free ordering between currencies, and an implicit one would be a silent conversion"),
                    );
                }
                Shape::Opaque
            }
            _ => Shape::Opaque,
        }
    }

    fn consume_linear(&mut self, e: &Expr, sc: &mut Scope, at: Span) {
        if let Expr::Path(p) = e {
            let n = &p.last().text;
            if let Some(v) = sc.linear.iter_mut().find(|v| &v.name == n) {
                v.uses.push(at);
            }
        }
    }

    /// Close an open conservation obligation and report the verdict.
    fn discharge_conservation(&mut self, row: &mut CurRow, span: Span, leg: Option<&str>) {
        row.substitute(&self.unifier);
        if row.is_empty() {
            return;
        }
        for v in rows::check_conservation(row) {
            match &v {
                Verdict::Conserves => self.report.conservation_proved += 1,
                Verdict::Undecided { .. } => self.report.runtime_obligations += 1,
                Verdict::Violates { currency, .. } => {
                    let scale = self.cat.currencies.get(currency).map(|c| c.scale).unwrap_or(2);
                    if let Some(mut d) = rows::diagnose(&v, scale) {
                        if let Some(name) = leg {
                            d = d.secondary(span, format!("in leg `{name}` of this `fx` form"));
                            d = d.note("each leg of an `fx` form conserves independently; a rate relates them, it does not balance them");
                        }
                        // Point at the ledger's own rule — the other half of the two-span
                        // form these diagnostics exist for.
                        if let Some(rel) = self.cat.relations.values().find(|r| r.conserve_span.is_some()) {
                            if let Some(cs) = rel.conserve_span {
                                d = d.secondary(cs, format!("`conserve per (..)` declared on `{}` here", rel.name));
                            }
                        }
                        self.d.push(d);
                    }
                }
            }
        }
    }

    fn check_confidential_use(&mut self, recv: &Expr, args: &[Arg], span: Span) {
        let Some(rel) = self.root_relation(recv) else { return };
        let confidential: Vec<(String, String, Span)> = rel
            .columns
            .iter()
            .filter_map(|c| c.confidential.clone().map(|l| (c.name.clone(), l, c.span)))
            .collect();
        if confidential.is_empty() {
            return;
        }
        for a in args {
            let mut used = Vec::new();
            collect_fields(&a.value, &mut used);
            for (name, span_of_use) in used {
                if let Some((_, level, decl)) = confidential.iter().find(|(c, _, _)| *c == name) {
                    self.d.push(
                        Diagnostic::error("NL0260", format!("`{name}` is `@confidential({level})` and cannot be used here"))
                            .primary(span_of_use, "the engine cannot compute on this column")
                            .secondary(*decl, format!("declared `@confidential({level})` here"))
                            .secondary(span, "used in a predicate, key or aggregate")
                            .note("an encrypted column is opaque to the engine: a filter over it would either not filter, or leak through timing")
                            .note("to use it, `declassify(..)` it with a capability — which is audited, and is the point"),
                    );
                }
            }
        }
    }

    fn root_relation(&self, e: &Expr) -> Option<&crate::resolve::RelationInfo> {
        match e {
            Expr::Path(p) => self.cat.relations.get(&p.last().text),
            Expr::Stage { recv, .. } | Expr::Fixpoint { recv, .. } => self.root_relation(recv),
            _ => None,
        }
    }
}

fn collect_fields(e: &Expr, out: &mut Vec<(String, Span)>) {
    match e {
        Expr::Field { name, span, .. } => out.push((name.text.clone(), *span)),
        Expr::Closure { body, .. } => collect_fields(body, out),
        Expr::Binary { lhs, rhs, .. } => {
            collect_fields(lhs, out);
            collect_fields(rhs, out);
        }
        Expr::Unary { operand, .. } => collect_fields(operand, out),
        Expr::Tuple { elems, .. } | Expr::Array { elems, .. } => elems.iter().for_each(|x| collect_fields(x, out)),
        Expr::Call { args, .. } | Expr::Stage { args, .. } => args.iter().for_each(|a| collect_fields(&a.value, out)),
        _ => {}
    }
}

/// `Money<usd>` -> `Some(Some("usd"))`; `Money` -> `Some(None)`; anything else -> `None`.
fn money_currency_of(t: &Ty) -> Option<Option<String>> {
    match t {
        Ty::Path { path, args, .. } if path.last().text == "Money" => Some(match args.first() {
            Some(Ty::Path { path, .. }) => Some(path.last().text.clone()),
            _ => None,
        }),
        Ty::Ref { inner, .. } => money_currency_of(inner),
        _ => None,
    }
}

fn first_binding(p: &Pat) -> Option<String> {
    p.bindings().first().map(|n| n.text.clone())
}

fn to_eff_row(row: &EffectRow) -> EffRow {
    let mut out = EffRow::new();
    for e in &row.effects {
        let args: Vec<String> = e.args.iter().map(|a| a.text.clone()).collect();
        if let Some(eff) = Effect::parse(&e.name.text, e.at.as_ref().map(|a| a.text.as_str()), &args) {
            out.add(eff, e.span);
        }
    }
    out
}
