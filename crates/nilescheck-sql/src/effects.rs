//! **Effect annotations, the SQL spelling of NL0310** (cycle 15, C15-05; E30b′ design §5.1),
//! and **which functions write a ledger** (the input of NSQ002, design §5.3).
//!
//! # The annotation
//!
//! `comment on function f(…) is 'effects: append; debits usd; credits usd; reads@snapshot'`.
//! Each item is `append`, `debits C`, `credits C`, `holds C`, `authorizes C` or `reads@R`,
//! where `C` is a currency or `*` and `R` a rung. A function with no `effects:` comment is not
//! checked, as a Niles function with no declared row is inferred and reported rather than
//! held to anything.
//!
//! # What a body is inferred to do
//!
//! * An insert into a ledger is `append`, and each leg is `debits C` when its amount is
//!   syntactically negative, `credits C` when it is syntactically positive, and both when its
//!   sign is not syntactic. The sign is read at the amount's **head**: descending through a
//!   cast, a one-element `row(…)` and a binary operator's left operand, a unary minus or a
//!   negative literal is negative; a column, parameter, field or non-negative literal is
//!   positive; anything else (a `case`, a call, a subquery) has no syntactic sign. `C` is the
//!   typed column filled (representation R1) or the row's currency literal (R2), and `*` when
//!   neither is known.
//! * A call to a function commented `produces` is `holds C`, and to one commented `requires`
//!   is `authorizes C`, where `C` is the currency of the call's money-typed argument.
//! * A call to a function this script declares contributes that function's declared row when
//!   it has one, and otherwise its inferred effects, transitively. A cycle contributes nothing
//!   further.
//! * A `select` from a relation with a `serve_contract` row is `reads@` its rung; a `select`
//!   from a ledger is `reads@ledger_consistent`.
//!
//! # The check
//!
//! Every inferred effect must be permitted by the declared row, by Niles's rule
//! (`niles_lang::effects::declared_permits`): a currency is matched exactly or by `*`; a read is
//! matched exactly, because a read's rung is a statement about freshness, not a permission.

use crate::ast::*;
use crate::check::{walk_expr, Catalog, Diag};
use crate::check2::{query_exprs, visit, Item, Types};
use crate::conventions::Conventions;
use crate::lex::Span;
use crate::plpgsql::{Block, LoopKind, PlStmt};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Eff {
    Append,
    Debit(String),
    Credit(String),
    Hold(String),
    Authorize(String),
    Read(String),
}

impl fmt::Display for Eff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eff::Append => write!(f, "append"),
            Eff::Debit(c) => write!(f, "debits {c}"),
            Eff::Credit(c) => write!(f, "credits {c}"),
            Eff::Hold(c) => write!(f, "holds {c}"),
            Eff::Authorize(c) => write!(f, "authorizes {c}"),
            Eff::Read(r) => write!(f, "reads@{r}"),
        }
    }
}

/// Parse an `effects:` annotation. `Err` names the first item it cannot read.
pub fn parse_row(text: &str) -> Option<Result<BTreeSet<Eff>, String>> {
    let rest = text.trim().strip_prefix("effects:")?;
    let mut out = BTreeSet::new();
    for item in rest.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let mut w = item.split_whitespace();
        let head = w.next().unwrap_or("");
        let arg = w.next();
        let e = match (head, arg) {
            ("append", None) => Eff::Append,
            ("debits", Some(c)) => Eff::Debit(c.to_string()),
            ("credits", Some(c)) => Eff::Credit(c.to_string()),
            ("holds", Some(c)) => Eff::Hold(c.to_string()),
            ("authorizes", Some(c)) => Eff::Authorize(c.to_string()),
            (h, None) if h.starts_with("reads@") => Eff::Read(h["reads@".len()..].to_string()),
            _ => return Some(Err(item.to_string())),
        };
        out.insert(e);
    }
    Some(Ok(out))
}

/// Niles's permission rule, for SQL's effect vocabulary.
pub fn permits(declared: &BTreeSet<Eff>, e: &Eff) -> bool {
    if declared.contains(e) {
        return true;
    }
    let star = "*".to_string();
    match e {
        Eff::Read(_) | Eff::Append => false,
        Eff::Debit(_) => declared.contains(&Eff::Debit(star)),
        Eff::Credit(_) => declared.contains(&Eff::Credit(star)),
        Eff::Hold(_) => declared.contains(&Eff::Hold(star)),
        Eff::Authorize(_) => declared.contains(&Eff::Authorize(star)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Neg,
    Pos,
    Unknown,
}

/// A leg amount's syntactic sign, read at its head (module doc).
pub fn sign(e: &Expr) -> Sign {
    match e {
        Expr::Un(op, _, _) if op == "-" => Sign::Neg,
        Expr::Un(op, x, _) if op == "+" => sign(x),
        Expr::Lit(Literal::Int(v), _) | Expr::Lit(Literal::Num(v), _) => {
            if v.starts_with('-') {
                Sign::Neg
            } else {
                Sign::Pos
            }
        }
        Expr::Cast(x, _, _) => sign(x),
        Expr::Row(v, _) if v.len() == 1 => sign(&v[0]),
        Expr::Bin(a, _, _, _) => sign(a),
        Expr::Col(..) | Expr::Param(..) | Expr::Field(..) => Sign::Pos,
        _ => Sign::Unknown,
    }
}

/// The top-level selects of a query body: the operands of its set operations.
pub fn top_selects(b: &QueryBody) -> Vec<&Select> {
    match b {
        QueryBody::Select(s) => vec![s],
        QueryBody::SetOp { left, right, .. } => {
            let mut v = top_selects(left);
            v.extend(top_selects(right));
            v
        }
        QueryBody::Nested(q) => top_selects(&q.body),
        _ => Vec::new(),
    }
}

/// Every relation a query reads, subqueries in expressions included.
pub fn relations(q: &Query, out: &mut BTreeSet<String>) {
    fn from(fi: &FromItem, out: &mut BTreeSet<String>) {
        match fi {
            FromItem::Table { name, .. } => {
                out.insert(name.last().to_string());
            }
            FromItem::Sub { query, .. } => relations(query, out),
            FromItem::Join { left, right, .. } => {
                from(left, out);
                from(right, out);
            }
            _ => {}
        }
    }
    for c in &q.with {
        if let Stmt::Query(cq) = c.body.as_ref() {
            relations(cq, out);
        }
    }
    crate::check::walk_query(q, &mut |s| {
        for fi in &s.from {
            from(fi, out);
        }
    });
    let mut exprs = Vec::new();
    query_exprs(q, &mut exprs);
    for e in exprs {
        expr_relations(e, out);
    }
}

/// Relations read by subqueries inside an expression.
pub fn expr_relations(e: &Expr, out: &mut BTreeSet<String>) {
    walk_expr(e, &mut |x| match x {
        Expr::Sub(q, _) | Expr::Exists(q, _) | Expr::ArraySub(q, _) | Expr::InSub(_, q, _, _) => {
            // `walk_expr` visits the subquery's own expressions; its `from` is read here.
            let mut inner = BTreeSet::new();
            crate::check::walk_query(q, &mut |s| {
                for fi in &s.from {
                    if let FromItem::Table { name, .. } = fi {
                        inner.insert(name.last().to_string());
                    }
                }
            });
            out.extend(inner);
        }
        _ => {}
    });
}

/// The parameter and variable types of a function (last name part), by name.
pub fn env_of(f: &CreateFunction) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for (i, a) in f
        .args
        .iter()
        .filter(|a| a.mode.as_deref() != Some("out"))
        .enumerate()
    {
        let t = a.ty.name.last().to_string();
        if let Some(n) = &a.name {
            env.insert(n.clone(), t.clone());
        }
        env.insert(format!("${}", i + 1), t);
    }
    fn decls(b: &Block, env: &mut BTreeMap<String, String>) {
        for d in &b.decls {
            if let Some(t) = &d.ty {
                env.insert(d.name.clone(), t.name.last().to_string());
            }
        }
        fn stmts(v: &[PlStmt], env: &mut BTreeMap<String, String>) {
            for s in v {
                match s {
                    PlStmt::Block(b) => decls(b, env),
                    PlStmt::If {
                        branches,
                        otherwise,
                        ..
                    } => {
                        for (_, b) in branches {
                            stmts(b, env);
                        }
                        if let Some(o) = otherwise {
                            stmts(o, env);
                        }
                    }
                    PlStmt::Case {
                        whens, otherwise, ..
                    } => {
                        for (_, b) in whens {
                            stmts(b, env);
                        }
                        if let Some(o) = otherwise {
                            stmts(o, env);
                        }
                    }
                    PlStmt::Loop { body, .. } => stmts(body, env),
                    _ => {}
                }
            }
        }
        stmts(&b.body, env);
        for (_, h) in &b.handlers {
            stmts(h, env);
        }
    }
    if let Some(b) = &f.plpgsql {
        decls(b, &mut env);
    }
    env
}

/// The currency of an expression by the simplest reading: a variable or parameter of a
/// currency type, a cast to one, or a column of one. `*` otherwise.
fn currency_of(types: &Types, env: &BTreeMap<String, String>, e: &Expr) -> String {
    match e {
        Expr::Col(n, _) if n.0.len() == 1 => {
            if let Some(t) = env.get(n.last()) {
                if types.currencies.contains_key(t) {
                    return t.clone();
                }
            }
            types.currency_of(e).unwrap_or_else(|| "*".into())
        }
        Expr::Param(k, _) => env
            .get(&format!("${k}"))
            .filter(|t| types.currencies.contains_key(*t))
            .cloned()
            .unwrap_or_else(|| "*".into()),
        _ => types.currency_of(e).unwrap_or_else(|| "*".into()),
    }
}

/// The legs of one insert into a ledger: (currency, sign).
fn legs(types: &Types, i: &Insert) -> Vec<(String, Sign)> {
    let table = i.table.last().to_string();
    let columns: Vec<String> = if i.columns.is_empty() {
        types.table_columns.get(&table).cloned().unwrap_or_default()
    } else {
        i.columns.clone()
    };
    let r2 = types.ledger_cols.get(&table).and_then(|(cc, ac)| {
        Some((
            columns.iter().position(|c| c == cc)?,
            columns.iter().position(|c| c == ac)?,
        ))
    });
    let money: Vec<(usize, String)> = columns
        .iter()
        .enumerate()
        .filter_map(|(k, c)| {
            types
                .columns
                .get(&(table.clone(), c.clone()))
                .map(|cur| (k, cur.clone()))
        })
        .collect();
    let is_null = |e: &Expr| match e {
        Expr::Lit(Literal::Null, _) => true,
        Expr::Cast(x, _, _) => matches!(x.as_ref(), Expr::Lit(Literal::Null, _)),
        _ => false,
    };
    let row_legs = |row: &[&Expr], out: &mut Vec<(String, Sign)>| {
        if let Some((cur_at, amt_at)) = r2 {
            if let Some(a) = row.get(amt_at) {
                if !is_null(a) {
                    let cur = match row.get(cur_at) {
                        Some(Expr::Lit(Literal::Str(c), _))
                            if types.currencies.contains_key(c) =>
                        {
                            c.clone()
                        }
                        _ => "*".into(),
                    };
                    out.push((cur, sign(a)));
                }
            }
        }
        for (k, cur) in &money {
            if let Some(a) = row.get(*k) {
                if !is_null(a) {
                    out.push((cur.clone(), sign(a)));
                }
            }
        }
    };
    let mut out = Vec::new();
    match i.source.as_ref().map(|q| &q.body) {
        Some(QueryBody::Values(rows)) => {
            for r in rows {
                let v: Vec<&Expr> = r.iter().collect();
                row_legs(&v, &mut out);
            }
        }
        Some(body) => {
            for s in top_selects(body) {
                let v: Vec<&Expr> = s.items.iter().map(|it| &it.expr).collect();
                row_legs(&v, &mut out);
            }
        }
        None => {}
    }
    out
}

/// Every string literal in a PL/pgSQL block's dynamic statements (`execute`, `for … in
/// execute`, `return query execute`), which `check2::visit` reports only as a span.
fn dynamic_strings(b: &Block, out: &mut Vec<String>) {
    fn lits(e: &Expr, out: &mut Vec<String>) {
        walk_expr(e, &mut |x| {
            if let Expr::Lit(Literal::Str(s), _) = x {
                out.push(s.clone());
            }
        });
    }
    fn stmts(v: &[PlStmt], out: &mut Vec<String>) {
        for s in v {
            match s {
                PlStmt::Execute { command, using, .. } => {
                    lits(command, out);
                    using.iter().for_each(|u| lits(u, out));
                }
                PlStmt::ReturnQuery {
                    execute: Some(c), ..
                } => lits(c, out),
                PlStmt::Loop { kind, body, .. } => {
                    if let LoopKind::ForExecute { command, using, .. } = kind.as_ref() {
                        lits(command, out);
                        using.iter().for_each(|u| lits(u, out));
                    }
                    stmts(body, out);
                }
                PlStmt::If {
                    branches,
                    otherwise,
                    ..
                } => {
                    for (_, b) in branches {
                        stmts(b, out);
                    }
                    if let Some(o) = otherwise {
                        stmts(o, out);
                    }
                }
                PlStmt::Case {
                    whens, otherwise, ..
                } => {
                    for (_, b) in whens {
                        stmts(b, out);
                    }
                    if let Some(o) = otherwise {
                        stmts(o, out);
                    }
                }
                PlStmt::Block(b) => dynamic_strings(b, out),
                _ => {}
            }
        }
    }
    stmts(&b.body, out);
    for (_, h) in &b.handlers {
        stmts(h, out);
    }
}

fn contains_word(hay: &str, word: &str) -> bool {
    hay.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|w| w.eq_ignore_ascii_case(word))
}

/// What one function's body does, before its calls are followed: its own effects, the
/// functions it calls, whether it writes a ledger statically, and its string literals.
#[derive(Default, Debug, Clone)]
struct Local {
    effects: BTreeSet<Eff>,
    calls: BTreeSet<String>,
    writes_ledger: bool,
    strings: Vec<String>,
}

fn local(types: &Types, conv: &Conventions, contracts: &BTreeMap<String, String>, s: &Stmt) -> Local {
    let mut l = Local::default();
    let env = match s {
        Stmt::CreateFunction(f) => env_of(f),
        _ => BTreeMap::new(),
    };
    let on_expr = |e: &Expr, l: &mut Local| {
        walk_expr(e, &mut |x| match x {
            Expr::Func(fc) => {
                let name = fc.name.last().to_string();
                let money = fc
                    .args
                    .iter()
                    .map(|(_, a)| currency_of(types, &env, a))
                    .find(|c| c != "*")
                    .unwrap_or_else(|| "*".into());
                if conv.produces.contains_key(&name) {
                    l.effects.insert(Eff::Hold(money.clone()));
                }
                if conv.requires.contains_key(&name) {
                    l.effects.insert(Eff::Authorize(money));
                }
                l.calls.insert(name);
            }
            Expr::Lit(Literal::Str(s), _) => l.strings.push(s.clone()),
            _ => {}
        });
        let mut rels = BTreeSet::new();
        expr_relations(e, &mut rels);
        for r in rels {
            read_effect(types, contracts, &r, l);
        }
    };
    visit(s, None, &mut |item| match item {
        Item::Expr(e, _) => on_expr(e, &mut l),
        Item::Query(q, _) => {
            let mut exprs = Vec::new();
            query_exprs(q, &mut exprs);
            for e in exprs {
                on_expr(e, &mut l);
            }
            let mut rels = BTreeSet::new();
            relations(q, &mut rels);
            for r in rels {
                read_effect(types, contracts, &r, &mut l);
            }
        }
        Item::Stmt(st, _) => match st {
            Stmt::Insert(i) if types.ledgers.contains(i.table.last()) => {
                l.writes_ledger = true;
                l.effects.insert(Eff::Append);
                for (cur, sg) in legs(types, i) {
                    match sg {
                        Sign::Neg => {
                            l.effects.insert(Eff::Debit(cur));
                        }
                        Sign::Pos => {
                            l.effects.insert(Eff::Credit(cur));
                        }
                        Sign::Unknown => {
                            l.effects.insert(Eff::Debit(cur.clone()));
                            l.effects.insert(Eff::Credit(cur));
                        }
                    }
                }
            }
            Stmt::Update(u) if types.ledgers.contains(u.table.last()) => l.writes_ledger = true,
            Stmt::Delete(d) if types.ledgers.contains(d.table.last()) => l.writes_ledger = true,
            Stmt::Merge(m) if types.ledgers.contains(m.table.last()) => l.writes_ledger = true,
            Stmt::Call(e, _) => on_expr(e, &mut l),
            _ => {}
        },
        Item::Dynamic(_) => {}
    });
    if let Stmt::CreateFunction(f) = s {
        if let Some(b) = &f.plpgsql {
            dynamic_strings(b, &mut l.strings);
        }
    }
    if let Stmt::Do { block, .. } = s {
        dynamic_strings(block, &mut l.strings);
    }
    l
}

fn read_effect(types: &Types, contracts: &BTreeMap<String, String>, rel: &str, l: &mut Local) {
    if let Some(r) = contracts.get(rel) {
        l.effects.insert(Eff::Read(r.clone()));
    } else if types.ledgers.contains(rel) {
        l.effects.insert(Eff::Read("ledger_consistent".into()));
    }
}

/// The whole-script analysis both rules need.
pub struct Analysis {
    /// function → its own effects, calls and literals.
    locals: BTreeMap<String, Local>,
    /// function → its declared row, when it has an `effects:` comment.
    pub declared: BTreeMap<String, BTreeSet<Eff>>,
    /// function → the unreadable item of its annotation.
    pub unreadable: BTreeMap<String, String>,
    /// functions attached to a trigger on a ledger.
    ledger_triggers: BTreeSet<String>,
    ledgers: BTreeSet<String>,
}

impl Analysis {
    pub fn of(stmts: &[Stmt]) -> Analysis {
        let types = Types::of(stmts);
        let conv = Conventions::of(stmts);
        let contracts: BTreeMap<String, String> = Catalog::of(stmts)
            .contracts
            .into_iter()
            .map(|(k, c)| (k, c.consistency))
            .collect();
        let mut locals = BTreeMap::new();
        let mut declared = BTreeMap::new();
        let mut unreadable = BTreeMap::new();
        let mut ledger_triggers = BTreeSet::new();
        for s in stmts {
            match s {
                Stmt::CreateFunction(f) => {
                    locals.insert(
                        f.name.last().to_string(),
                        local(&types, &conv, &contracts, s),
                    );
                }
                Stmt::Comment {
                    kind,
                    target,
                    text: Some(text),
                    ..
                } if kind == "function" || kind == "procedure" => match parse_row(text) {
                    Some(Ok(row)) => {
                        declared.insert(target.last().to_string(), row);
                    }
                    Some(Err(item)) => {
                        unreadable.insert(target.last().to_string(), item);
                    }
                    None => {}
                },
                Stmt::CreateTrigger(t) if types.ledgers.contains(t.table.last()) => {
                    ledger_triggers.insert(t.function.last().to_string());
                }
                _ => {}
            }
        }
        Analysis {
            locals,
            declared,
            unreadable,
            ledger_triggers,
            ledgers: types.ledgers,
        }
    }

    /// A function's inferred effects, its calls followed.
    pub fn inferred(&self, f: &str) -> BTreeSet<Eff> {
        let mut seen = BTreeSet::new();
        self.inferred_in(f, &mut seen)
    }

    fn inferred_in(&self, f: &str, seen: &mut BTreeSet<String>) -> BTreeSet<Eff> {
        if !seen.insert(f.to_string()) {
            return BTreeSet::new();
        }
        let Some(l) = self.locals.get(f) else {
            return BTreeSet::new();
        };
        let mut out = l.effects.clone();
        for c in &l.calls {
            if c == f {
                continue;
            }
            if let Some(d) = self.declared.get(c) {
                out.extend(d.iter().cloned());
            } else if self.locals.contains_key(c) {
                out.extend(self.inferred_in(c, seen));
            }
        }
        out
    }

    /// Whether a function writes a ledger (design §5.3): statically, through a call, as a
    /// ledger trigger's function, or by naming a ledger in a string literal.
    pub fn writes_ledger(&self, f: &str) -> bool {
        let mut seen = BTreeSet::new();
        self.writes_in(f, &mut seen)
    }

    fn writes_in(&self, f: &str, seen: &mut BTreeSet<String>) -> bool {
        if !seen.insert(f.to_string()) {
            return false;
        }
        if self.ledger_triggers.contains(f) {
            return true;
        }
        let Some(l) = self.locals.get(f) else {
            return false;
        };
        l.writes_ledger
            || self.names_a_ledger(&l.strings)
            || l.calls.iter().any(|c| self.writes_in(c, seen))
    }

    /// Whether any of these strings names a ledger as a whole word.
    pub fn names_a_ledger(&self, strings: &[String]) -> bool {
        strings
            .iter()
            .any(|s| self.ledgers.iter().any(|l| contains_word(s, l)))
    }

    /// The same question for an anonymous block, which has no name to call it by.
    pub fn block_writes_ledger(&self, types_stmt: &Stmt, stmts: &[Stmt]) -> bool {
        let types = Types::of(stmts);
        let conv = Conventions::of(stmts);
        let l = local(&types, &conv, &BTreeMap::new(), types_stmt);
        let mut seen = BTreeSet::new();
        l.writes_ledger
            || self.names_a_ledger(&l.strings)
            || l.calls.iter().any(|c| self.writes_in(c, &mut seen))
    }
}

fn span_of_function(stmts: &[Stmt], name: &str) -> Span {
    stmts
        .iter()
        .find_map(|s| match s {
            Stmt::CreateFunction(f) if f.name.last() == name => Some(f.span),
            _ => None,
        })
        .unwrap_or_default()
}

/// NL0310 for every annotated function whose body does something its row does not permit;
/// NSQ003 (a warning) for an annotation item that cannot be read.
pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let a = Analysis::of(stmts);
    let mut out = Vec::new();
    for (f, row) in &a.declared {
        for e in a.inferred(f) {
            if !permits(row, &e) {
                let declared: Vec<String> = row.iter().map(|x| x.to_string()).collect();
                out.push(Diag {
                    code: "NL0310",
                    error: true,
                    msg: format!(
                        "`{f}` has the effect `{e}`, which its declaration does not permit (declared: {})",
                        declared.join("; ")
                    ),
                    span: span_of_function(stmts, f),
                    defect: None,
                });
            }
        }
    }
    for (f, item) in &a.unreadable {
        out.push(Diag {
            code: "NSQ003",
            error: false,
            msg: format!("`{f}`'s effects annotation has an item this checker cannot read: `{item}`"),
            span: span_of_function(stmts, f),
            defect: None,
        });
    }
    out
}
