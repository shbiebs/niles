//! **Typing of PL/pgSQL and `language sql` bodies** (cycle 15, C15-05; E30b′ design §5.2).
//!
//! PostgreSQL resolves a PL/pgSQL statement's operators and casts when the statement first
//! runs, so a body that adds a usd to an eur, or passes an eur where a usd parameter is
//! declared, is accepted by `create function` and fails, or does not fail, at run time.
//! `check2`'s NL0250 knew only the currency of *columns*; inside a body the money is mostly
//! parameters and variables, and their minor units.
//!
//! # The types
//!
//! A value is a currency composite `C`; an integer **`minor⟨C⟩`** that came from a `C`'s minor
//! units (`(m).minor` where `m: usd`), or `minor⟨?⟩` when it carries no currency (a literal, a
//! plain integer); null; or a type this reading does not follow. `+`, `-` and unary `-` keep
//! `minor⟨C⟩`; `*` and `/` by a plain integer keep it; `sum/min/max/coalesce/least/greatest`
//! of `T` are `T`; a call to a script function is its declared return type; a scalar
//! subquery is its one item's type.
//!
//! # What is refused
//!
//! * **NL0250** — `+`, `-` or a comparison between two currencies, `minor⟨C⟩` against
//!   `minor⟨D⟩` included.
//! * **NL0255** — a `C` where a `D` is required: a script function's argument, an insert into
//!   a column of type `D`, `row(e)::D` with `e: minor⟨C⟩`, and a ledger row whose currency
//!   literal names `D` while its money is `C` (R1: the typed column filled; R2: the amount's
//!   `minor⟨C⟩`).
//! * **NL0332** — an assignment, a declaration's default or a `return` whose value is `C`
//!   where the declared type is `D`.
//!
//! Whatever the reading cannot type is left alone: no rule here guesses.

use crate::ast::*;
use crate::check::{walk_expr, Diag};
use crate::check2::{query_exprs, Types};
use crate::effects::top_selects;
use crate::lex::Span;
use crate::plpgsql::{Block, LoopKind, PlStmt};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Cur(String),
    Minor(Option<String>),
    Null,
    Other,
}

const INTS: &[&str] = &[
    "bigint", "int8", "integer", "int", "int4", "smallint", "int2", "numeric", "decimal",
];

fn classify(types: &Types, t: &str) -> Ty {
    if types.currencies.contains_key(t) {
        Ty::Cur(t.to_string())
    } else if INTS.contains(&t) {
        Ty::Minor(None)
    } else {
        Ty::Other
    }
}

fn cur_of(t: &Ty) -> Option<&String> {
    match t {
        Ty::Cur(c) | Ty::Minor(Some(c)) => Some(c),
        _ => None,
    }
}

/// A script function's signature: parameter types and return type (last name parts).
type Sigs = BTreeMap<String, (Vec<String>, Option<String>)>;

struct Tx<'a> {
    types: &'a Types,
    sigs: &'a Sigs,
    env: BTreeMap<String, String>,
    returns: Option<String>,
    /// Where to report inside a re-parsed `language sql` body, whose spans are relative.
    at: Option<Span>,
    out: Vec<Diag>,
    seen: BTreeSet<(usize, usize, &'static str)>,
}

impl Tx<'_> {
    fn push(&mut self, code: &'static str, msg: String, span: Span) {
        let span = self.at.unwrap_or(span);
        if self.seen.insert((span.start, span.end, code)) {
            self.out.push(Diag {
                code,
                error: true,
                msg,
                span,
                defect: Some("d2"),
            });
        }
    }

    fn column(&self, name: &str) -> Ty {
        let found: BTreeSet<&String> = self
            .types
            .column_types
            .iter()
            .filter(|((_, c), _)| c == name)
            .map(|(_, t)| t)
            .collect();
        if found.len() == 1 {
            classify(self.types, found.into_iter().next().expect("one"))
        } else {
            Ty::Other
        }
    }

    fn ty(&mut self, e: &Expr) -> Ty {
        match e {
            Expr::Lit(Literal::Int(_), _) | Expr::Lit(Literal::Num(_), _) => Ty::Minor(None),
            Expr::Lit(Literal::Null, _) => Ty::Null,
            Expr::Lit(..) => Ty::Other,
            Expr::Col(n, _) => {
                if n.0.len() == 1 {
                    if let Some(t) = self.env.get(n.last()) {
                        return classify(self.types, &t.clone());
                    }
                }
                self.column(n.last())
            }
            Expr::Param(k, _) => match self.env.get(&format!("${k}")) {
                Some(t) => classify(self.types, &t.clone()),
                None => Ty::Other,
            },
            Expr::Cast(x, tn, sp) => {
                let target = classify(self.types, tn.name.last());
                let inner = match x.as_ref() {
                    Expr::Row(v, _) if v.len() == 1 => self.ty(&v[0]),
                    other => self.ty(other),
                };
                match (&target, &inner) {
                    // `null::eur` is no money at all, not an eur.
                    (_, Ty::Null) => Ty::Null,
                    (Ty::Cur(d), Ty::Minor(Some(c))) | (Ty::Cur(d), Ty::Cur(c)) if c != d => {
                        self.push(
                            "NL0255",
                            format!("a `{d}` built from `{c}`: an amount in one currency relabelled as another"),
                            *sp,
                        );
                        target
                    }
                    (Ty::Minor(None), Ty::Minor(c)) => Ty::Minor(c.clone()),
                    _ => target,
                }
            }
            // `(x)` parses as a one-element row: it is `x`, parenthesised.
            Expr::Row(v, _) if v.len() == 1 => self.ty(&v[0]),
            Expr::Row(v, _) => {
                for x in v {
                    self.ty(x);
                }
                Ty::Other
            }
            Expr::Field(x, f, _) => {
                let t = self.ty(x);
                match (t, f.as_str()) {
                    (Ty::Cur(c), "minor") => Ty::Minor(Some(c)),
                    _ => Ty::Other,
                }
            }
            Expr::Un(op, x, _) => {
                let t = self.ty(x);
                match (op.as_str(), t) {
                    ("-" | "+", Ty::Minor(c)) => Ty::Minor(c),
                    _ => Ty::Other,
                }
            }
            Expr::Bin(a, op, b, sp) => {
                let (ta, tb) = (self.ty(a), self.ty(b));
                let op = op.as_str();
                match op {
                    "+" | "-" | "=" | "<>" | "!=" | "<" | ">" | "<=" | ">=" => {
                        if let (Some(c), Some(d)) = (cur_of(&ta), cur_of(&tb)) {
                            if c != d
                                && matches!(
                                    (&ta, &tb),
                                    (Ty::Minor(_), Ty::Minor(_)) | (Ty::Cur(_), Ty::Cur(_))
                                )
                            {
                                self.push(
                                    "NL0250",
                                    format!("`{op}` between `{c}` and `{d}`: amounts in different currencies"),
                                    *sp,
                                );
                            }
                        }
                        if matches!(op, "+" | "-") {
                            match (ta, tb) {
                                (Ty::Minor(c), Ty::Minor(d)) => Ty::Minor(c.or(d)),
                                (Ty::Minor(c), Ty::Null) | (Ty::Null, Ty::Minor(c)) => Ty::Minor(c),
                                (Ty::Cur(c), Ty::Cur(_)) => Ty::Cur(c),
                                _ => Ty::Other,
                            }
                        } else {
                            Ty::Other
                        }
                    }
                    "*" | "/" | "%" => match (ta, tb) {
                        (Ty::Minor(Some(c)), Ty::Minor(None))
                        | (Ty::Minor(None), Ty::Minor(Some(c))) => Ty::Minor(Some(c)),
                        (Ty::Minor(_), Ty::Minor(_)) => Ty::Minor(None),
                        _ => Ty::Other,
                    },
                    _ => Ty::Other,
                }
            }
            Expr::Func(f) => self.call(f),
            Expr::Case {
                operand,
                whens,
                otherwise,
                ..
            } => {
                if let Some(o) = operand {
                    self.ty(o);
                }
                let mut arms = Vec::new();
                for (c, r) in whens {
                    self.ty(c);
                    arms.push(self.ty(r));
                }
                if let Some(o) = otherwise {
                    arms.push(self.ty(o));
                }
                let known: Vec<Ty> = arms.into_iter().filter(|t| *t != Ty::Null).collect();
                match known.first() {
                    Some(t) if known.iter().all(|x| x == t) => t.clone(),
                    _ => Ty::Other,
                }
            }
            Expr::Sub(q, _) => {
                self.query(q);
                match &q.body {
                    QueryBody::Select(s) if s.items.len() == 1 => self.ty(&s.items[0].expr),
                    _ => Ty::Other,
                }
            }
            Expr::Exists(q, _) | Expr::ArraySub(q, _) => {
                self.query(q);
                Ty::Other
            }
            Expr::InSub(x, q, _, _) => {
                self.ty(x);
                self.query(q);
                Ty::Other
            }
            other => {
                // Not typed here; its parts still are.
                let mut parts: Vec<Expr> = Vec::new();
                let mut first = true;
                walk_expr(other, &mut |x| {
                    if first {
                        first = false;
                    } else {
                        parts.push(x.clone());
                    }
                });
                for p in &parts {
                    self.ty(p);
                }
                Ty::Other
            }
        }
    }

    fn call(&mut self, f: &FuncCall) -> Ty {
        let name = f.name.last().to_lowercase();
        let args: Vec<Ty> = f.args.iter().map(|(_, a)| self.ty(a)).collect();
        if let Some(x) = &f.filter {
            self.ty(x);
        }
        match name.as_str() {
            "sum" | "min" | "max" => args.first().cloned().unwrap_or(Ty::Other),
            // Not an operation between amounts: `coalesce` picks one, and R1 writes
            // `coalesce((amt_usd).minor, (amt_eur).minor, …)` over the one column a row fills.
            // So no NL0250 here (design §5.2 lists `+`, `-` and comparisons). The result has a
            // currency only when every argument that has one agrees.
            "coalesce" | "least" | "greatest" | "nullif" => {
                let curs: BTreeSet<&String> = args.iter().filter_map(cur_of).collect();
                let first = args.iter().find(|t| **t != Ty::Null).cloned();
                match (curs.len(), first) {
                    (0, Some(t)) => t,
                    (1, _) => args
                        .iter()
                        .find(|t| cur_of(t).is_some())
                        .cloned()
                        .unwrap_or(Ty::Other),
                    (_, Some(Ty::Minor(_))) => Ty::Minor(None),
                    _ => Ty::Other,
                }
            }
            _ => {
                let Some((params, ret)) = self.sigs.get(f.name.last()).cloned() else {
                    return Ty::Other;
                };
                for (k, (p, a)) in params.iter().zip(args.iter()).enumerate() {
                    if let (Ty::Cur(d), Some(c)) = (classify(self.types, p), cur_of(a)) {
                        if *c != d && matches!(a, Ty::Cur(_)) {
                            self.push(
                                "NL0255",
                                format!(
                                    "argument {} of `{}` is `{c}`, and the function declares `{d}`",
                                    k + 1,
                                    f.name.last()
                                ),
                                f.span,
                            );
                        }
                    }
                }
                ret.map(|r| classify(self.types, &r)).unwrap_or(Ty::Other)
            }
        }
    }

    fn query(&mut self, q: &Query) {
        let mut exprs = Vec::new();
        query_exprs(q, &mut exprs);
        for e in exprs {
            self.ty(e);
        }
    }

    fn insert(&mut self, i: &Insert) {
        let table = i.table.last().to_string();
        let columns: Vec<String> = if i.columns.is_empty() {
            self.types
                .table_columns
                .get(&table)
                .cloned()
                .unwrap_or_default()
        } else {
            i.columns.clone()
        };
        let col_ty: Vec<Ty> = columns
            .iter()
            .map(|c| {
                self.types
                    .column_types
                    .get(&(table.clone(), c.clone()))
                    .map(|t| classify(self.types, t))
                    .unwrap_or(Ty::Other)
            })
            .collect();
        let ledger = self.types.ledgers.contains(&table);
        let (cur_col, amt_col) = match self.types.ledger_cols.get(&table) {
            Some((c, a)) => (Some(c.clone()), Some(a.clone())),
            None => (Some("cur".to_string()), None),
        };
        let cur_at = cur_col.and_then(|c| columns.iter().position(|x| *x == c));
        let amt_at = amt_col.and_then(|c| columns.iter().position(|x| *x == c));
        let rows: Vec<Vec<Expr>> = match i.source.as_ref().map(|q| &q.body) {
            Some(QueryBody::Values(v)) => v.clone(),
            Some(body) => top_selects(body)
                .into_iter()
                .map(|s| s.items.iter().map(|it| it.expr.clone()).collect())
                .collect(),
            None => Vec::new(),
        };
        if let Some(q) = &i.source {
            if !matches!(q.body, QueryBody::Values(_)) {
                self.query(q);
            }
        }
        for row in rows {
            let tys: Vec<Ty> = row.iter().map(|e| self.ty(e)).collect();
            for (k, t) in tys.iter().enumerate() {
                if let (Some(Ty::Cur(d)), Ty::Cur(c)) = (col_ty.get(k), t) {
                    if c != d {
                        self.push(
                            "NL0255",
                            format!(
                                "a `{c}` inserted into `{table}.{}`, which is `{d}`",
                                columns[k]
                            ),
                            row[k].span(),
                        );
                    }
                }
            }
            if !ledger {
                continue;
            }
            // The currency the row is labelled with, when it is a literal.
            let label = cur_at.and_then(|k| match row.get(k) {
                Some(Expr::Lit(Literal::Str(s), _)) if self.types.currencies.contains_key(s) => {
                    Some(s.clone())
                }
                _ => None,
            });
            let Some(label) = label else { continue };
            // The currency the row's money is: R2 the amount's, R1 the typed column filled.
            let mut money: Vec<(String, Span)> = Vec::new();
            if let Some(k) = amt_at {
                if let Some(Ty::Minor(Some(c))) = tys.get(k) {
                    money.push((c.clone(), row[k].span()));
                }
            } else {
                for (k, ct) in col_ty.iter().enumerate() {
                    if let (Ty::Cur(c), Some(t)) = (ct, tys.get(k)) {
                        if *t != Ty::Null {
                            money.push((c.clone(), row[k].span()));
                        }
                    }
                }
            }
            for (c, sp) in money {
                if c != label {
                    self.push(
                        "NL0255",
                        format!("a ledger row labelled `{label}` whose money is `{c}`"),
                        sp,
                    );
                }
            }
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Insert(i) => self.insert(i),
            Stmt::Query(q) => self.query(q),
            Stmt::Update(u) => {
                let table = u.table.last().to_string();
                for (cols, e) in &u.set {
                    let t = self.ty(e);
                    if let (Some(col), Ty::Cur(c)) = (cols.first(), &t) {
                        if let Some(Ty::Cur(d)) = self
                            .types
                            .column_types
                            .get(&(table.clone(), col.clone()))
                            .map(|x| classify(self.types, x))
                        {
                            if *c != d {
                                self.push(
                                    "NL0255",
                                    format!("a `{c}` assigned to `{table}.{col}`, which is `{d}`"),
                                    e.span(),
                                );
                            }
                        }
                    }
                }
                if let Some(w) = &u.where_ {
                    self.ty(w);
                }
            }
            Stmt::Delete(d) => {
                if let Some(w) = &d.where_ {
                    self.ty(w);
                }
            }
            Stmt::Call(e, _) => {
                self.ty(e);
            }
            _ => {}
        }
    }

    fn assign(&mut self, name: &str, value: &Expr, span: Span) {
        let t = self.ty(value);
        let Some(declared) = self.env.get(name).cloned() else {
            return;
        };
        if let (Ty::Cur(d), Ty::Cur(c)) = (classify(self.types, &declared), &t) {
            if d != *c {
                self.push(
                    "NL0332",
                    format!("`{name}` is declared `{d}` and is given a `{c}`"),
                    span,
                );
            }
        }
    }

    fn block(&mut self, b: &Block) {
        for d in &b.decls {
            if let Some(v) = &d.default {
                self.assign(&d.name, v, d.span);
            }
        }
        self.stmts(&b.body);
        for (_, h) in &b.handlers {
            self.stmts(h);
        }
    }

    fn stmts(&mut self, v: &[PlStmt]) {
        for s in v {
            match s {
                PlStmt::Assign {
                    target,
                    value,
                    span,
                } => match target {
                    Expr::Col(n, _) if n.0.len() == 1 => self.assign(n.last(), value, *span),
                    _ => {
                        self.ty(value);
                    }
                },
                PlStmt::Sql { stmt, .. } => self.stmt(stmt),
                PlStmt::Perform(q, _) => self.query(q),
                PlStmt::Execute { command, using, .. } => {
                    self.ty(command);
                    for u in using {
                        self.ty(u);
                    }
                }
                PlStmt::If {
                    branches,
                    otherwise,
                    ..
                } => {
                    for (c, b) in branches {
                        self.ty(c);
                        self.stmts(b);
                    }
                    if let Some(o) = otherwise {
                        self.stmts(o);
                    }
                }
                PlStmt::Case {
                    operand,
                    whens,
                    otherwise,
                    ..
                } => {
                    if let Some(o) = operand {
                        self.ty(o);
                    }
                    for (cs, b) in whens {
                        for c in cs {
                            self.ty(c);
                        }
                        self.stmts(b);
                    }
                    if let Some(o) = otherwise {
                        self.stmts(o);
                    }
                }
                PlStmt::Loop { kind, body, .. } => {
                    match kind.as_ref() {
                        LoopKind::While(c) => {
                            self.ty(c);
                        }
                        LoopKind::ForQuery { query, .. } => self.query(query),
                        LoopKind::ForInt { lo, hi, by, .. } => {
                            self.ty(lo);
                            self.ty(hi);
                            if let Some(b) = by {
                                self.ty(b);
                            }
                        }
                        _ => {}
                    }
                    self.stmts(body);
                }
                PlStmt::Return(Some(e), span) => {
                    let t = self.ty(e);
                    if let (Some(r), Ty::Cur(c)) = (self.returns.clone(), &t) {
                        if let Ty::Cur(d) = classify(self.types, &r) {
                            if d != *c {
                                self.push(
                                    "NL0332",
                                    format!("the function returns `{d}` and this is `{c}`"),
                                    *span,
                                );
                            }
                        }
                    }
                }
                PlStmt::ReturnQuery { query: Some(q), .. } => self.query(q),
                PlStmt::Raise { args, .. } => {
                    for a in args {
                        self.ty(a);
                    }
                }
                PlStmt::Block(b) => self.block(b),
                _ => {}
            }
        }
    }
}

/// NL0250, NL0255 and NL0332 inside every function body and anonymous block.
pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let types = Types::of(stmts);
    let mut sigs: Sigs = BTreeMap::new();
    for s in stmts {
        if let Stmt::CreateFunction(f) = s {
            let params = f
                .args
                .iter()
                .filter(|a| a.mode.as_deref() != Some("out"))
                .map(|a| a.ty.name.last().to_string())
                .collect();
            let ret = match &f.returns {
                Some(Returns::Type { setof: false, ty }) => Some(ty.name.last().to_string()),
                _ => None,
            };
            sigs.insert(f.name.last().to_string(), (params, ret));
        }
    }
    let mut out = Vec::new();
    for s in stmts {
        match s {
            Stmt::CreateFunction(f) => {
                let mut tx = Tx {
                    types: &types,
                    sigs: &sigs,
                    env: crate::effects::env_of(f),
                    returns: sigs.get(f.name.last()).and_then(|(_, r)| r.clone()),
                    at: None,
                    out: Vec::new(),
                    seen: BTreeSet::new(),
                };
                if let Some(b) = &f.plpgsql {
                    tx.block(b);
                } else {
                    match &f.body {
                        Some(FuncBody::Text(text, span))
                            if f.language.as_deref() == Some("sql") =>
                        {
                            if let Ok((inner, _)) = crate::parse(text) {
                                tx.at = Some(*span);
                                for x in &inner {
                                    tx.stmt(x);
                                }
                            }
                        }
                        Some(FuncBody::Atomic(v)) => {
                            for x in v {
                                tx.stmt(x);
                            }
                        }
                        Some(FuncBody::Return(e)) => {
                            tx.ty(e);
                        }
                        _ => {}
                    }
                }
                out.extend(tx.out);
            }
            Stmt::Do { block, .. } => {
                let mut tx = Tx {
                    types: &types,
                    sigs: &sigs,
                    env: BTreeMap::new(),
                    returns: None,
                    at: None,
                    out: Vec::new(),
                    seen: BTreeSet::new(),
                };
                tx.block(block);
                out.extend(tx.out);
            }
            _ => {}
        }
    }
    out
}
