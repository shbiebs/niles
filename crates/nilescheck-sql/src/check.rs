//! **The catalog checker**: the serve-contract family and its neighbours, checked over a
//! parsed SQL script — at check time, before any data exists — with the codes `niles-lang`
//! and `niles-ir` use, so the two checkers' outputs can be diffed.
//!
//! Where cycle 13's arm (Appendix A of its work order) read a live catalog with regular
//! expressions over `pg_get_viewdef`, this reads the script's own DDL through the parser, so a
//! rule sees the query's structure (a top-level `limit`, a call anywhere in a predicate, the
//! views a view reads) rather than the text.
//!
//! # The library the rules read
//!
//! PostgreSQL 16 refuses `create view … with (consistency = …)` (`unrecognized parameter`),
//! so a contract lives beside the view, as cycle 13 found:
//! * `serve_contract(view_name, consistency, materialize, retain, budget, lineage)` rows,
//!   written as `insert into serve_contract values (…)`;
//! * `comment on table t is 'ledger'` marks a ledger;
//! * `comment on column t.c is 'confidential:e2ee'` labels a column;
//! * `comment on index i is 'anchor'` marks an anchor index.

use crate::ast::*;
use crate::lex::Span;
use crate::plpgsql::{Block, LoopKind, PlStmt};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    pub code: &'static str,
    pub error: bool,
    pub msg: String,
    pub span: Span,
    /// The E14 defect class this corresponds to, when there is one.
    pub defect: Option<&'static str>,
}

/// The consistency ladder, in rung order (the same order as `niles-lang`'s).
pub const RUNGS: &[&str] = &[
    "bounded",
    "monotonic",
    "read_your_writes",
    "snapshot",
    "serializable",
    "ledger_consistent",
];

pub fn rung(c: &str) -> Option<usize> {
    RUNGS.iter().position(|r| *r == c)
}

#[derive(Debug, Clone, Default)]
pub struct Contract {
    pub consistency: String,
    pub materialize: String,
    pub retain: String,
    pub span: Span,
}

/// PostgreSQL 16's built-in functions whose result is not a function of their arguments
/// (`provolatile <> 'i'`) that a view predicate is likely to reach — the time and randomness
/// functions. A user function is judged by its declared volatility.
pub const NON_IMMUTABLE_BUILTINS: &[&str] = &[
    "now",
    "clock_timestamp",
    "statement_timestamp",
    "transaction_timestamp",
    "timeofday",
    "current_date",
    "current_time",
    "current_timestamp",
    "localtime",
    "localtimestamp",
    "random",
    "random_normal",
    "gen_random_uuid",
    "nextval",
    "currval",
    "lastval",
    "setseed",
    "txid_current",
    "pg_current_xact_id",
    "age",
];

#[derive(Debug, Default)]
pub struct Catalog {
    pub tables: BTreeMap<String, Vec<String>>,
    pub ledgers: BTreeSet<String>,
    /// (table, column) → label.
    pub labels: BTreeMap<(String, String), String>,
    pub views: BTreeMap<String, (Query, Span)>,
    pub contracts: BTreeMap<String, Contract>,
    /// index name → (table, columns as written).
    pub indexes: BTreeMap<String, (String, Vec<String>)>,
    pub anchor_indexes: BTreeSet<String>,
    /// function name → declared volatility (default `volatile`).
    pub functions: BTreeMap<String, String>,
}

fn base(n: &Name) -> String {
    n.last().to_string()
}

impl Catalog {
    pub fn of(stmts: &[Stmt]) -> Catalog {
        let mut c = Catalog::default();
        for s in stmts {
            match s {
                Stmt::CreateTable(t) => {
                    c.tables.insert(
                        base(&t.name),
                        t.columns.iter().map(|x| x.name.clone()).collect(),
                    );
                }
                Stmt::CreateView(v) => {
                    c.views.insert(base(&v.name), ((*v.query).clone(), v.span));
                }
                Stmt::CreateIndex {
                    name: Some(n),
                    table,
                    columns,
                    ..
                } => {
                    let cols = columns
                        .iter()
                        .map(|o| match &o.expr {
                            Expr::Col(n, _) => n.last().to_string(),
                            other => format!("{other:?}"),
                        })
                        .collect();
                    c.indexes.insert(n.clone(), (base(table), cols));
                }
                Stmt::CreateFunction(f) => {
                    c.functions.insert(
                        base(&f.name),
                        f.volatility.clone().unwrap_or_else(|| "volatile".into()),
                    );
                }
                Stmt::Comment {
                    kind, target, text, ..
                } => {
                    let text = text.clone().unwrap_or_default();
                    match kind.as_str() {
                        "table" if text == "ledger" => {
                            c.ledgers.insert(base(target));
                        }
                        "column" if !text.is_empty() => {
                            let parts = &target.0;
                            if parts.len() >= 2 {
                                c.labels.insert(
                                    (
                                        parts[parts.len() - 2].clone(),
                                        parts[parts.len() - 1].clone(),
                                    ),
                                    text,
                                );
                            }
                        }
                        "index" if text == "anchor" => {
                            c.anchor_indexes.insert(base(target));
                        }
                        _ => {}
                    }
                }
                Stmt::Insert(i) if base(&i.table) == "serve_contract" => {
                    if let Some(q) = &i.source {
                        if let QueryBody::Values(rows) = &q.body {
                            for row in rows {
                                let s = |k: usize| match row.get(k) {
                                    Some(Expr::Lit(Literal::Str(v), _)) => v.clone(),
                                    _ => String::new(),
                                };
                                c.contracts.insert(
                                    s(0),
                                    Contract {
                                        consistency: s(1),
                                        materialize: s(2),
                                        retain: if s(3).is_empty() {
                                            "evictable".into()
                                        } else {
                                            s(3)
                                        },
                                        span: i.span,
                                    },
                                );
                            }
                        }
                    }
                }
                Stmt::Alter {
                    kind, name, action, ..
                } if kind == "table" => {
                    // `alter table t add column c …` extends the table.
                    if let Some(rest) = action.strip_prefix("add column") {
                        let col = rest
                            .trim()
                            .trim_start_matches("if not exists")
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_string();
                        c.tables.entry(base(name)).or_default().push(col);
                    }
                }
                _ => {}
            }
        }
        c
    }

    fn volatile(&self, f: &str) -> bool {
        match self.functions.get(f) {
            Some(v) => v != "immutable",
            None => NON_IMMUTABLE_BUILTINS.contains(&f),
        }
    }
}

// ── walking the tree ──────────────────────────────────────────────────────────────────────

/// Every function name called anywhere in an expression.
fn calls_in(e: &Expr, out: &mut Vec<(String, Span)>) {
    walk_expr(e, &mut |x| {
        if let Expr::Func(f) = x {
            out.push((f.name.last().to_string(), f.span));
        }
    });
}

pub fn walk_expr(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(e);
    match e {
        Expr::Bin(a, _, b, _) | Expr::AtTimeZone(a, b, _) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        Expr::Un(_, a, _)
        | Expr::Is(a, _, _)
        | Expr::Cast(a, _, _)
        | Expr::Field(a, _, _)
        | Expr::Collate(a, _, _) => walk_expr(a, f),
        Expr::Func(c) => {
            for (_, a) in &c.args {
                walk_expr(a, f);
            }
            if let Some(x) = &c.filter {
                walk_expr(x, f);
            }
            for o in c.order_by.iter().chain(&c.within_group) {
                walk_expr(&o.expr, f);
            }
        }
        Expr::Case {
            operand,
            whens,
            otherwise,
            ..
        } => {
            if let Some(o) = operand {
                walk_expr(o, f);
            }
            for (c, r) in whens {
                walk_expr(c, f);
                walk_expr(r, f);
            }
            if let Some(o) = otherwise {
                walk_expr(o, f);
            }
        }
        Expr::Between { e, lo, hi, .. } => {
            walk_expr(e, f);
            walk_expr(lo, f);
            walk_expr(hi, f);
        }
        Expr::InList(a, v, _, _) => {
            walk_expr(a, f);
            for x in v {
                walk_expr(x, f);
            }
        }
        Expr::InSub(a, q, _, _) => {
            walk_expr(a, f);
            walk_query(q, &mut |qq| walk_select_exprs(qq, f));
        }
        Expr::Quantified(a, _, _, b, _) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        Expr::Exists(q, _) | Expr::Sub(q, _) | Expr::ArraySub(q, _) => {
            walk_query(q, &mut |qq| walk_select_exprs(qq, f));
        }
        Expr::Array(v, _) | Expr::Row(v, _) => {
            for x in v {
                walk_expr(x, f);
            }
        }
        Expr::Subscript(a, lo, hi, _, _) => {
            walk_expr(a, f);
            if let Some(x) = lo.as_ref() {
                walk_expr(x, f);
            }
            if let Some(x) = hi.as_ref() {
                walk_expr(x, f);
            }
        }
        Expr::Lit(..) | Expr::Col(..) | Expr::Star(_) | Expr::Param(..) | Expr::Default(_) => {}
    }
}

/// Every `Select` in a query, CTEs and set operations and subqueries in `from` included.
pub fn walk_query(q: &Query, f: &mut dyn FnMut(&Select)) {
    for c in &q.with {
        if let Stmt::Query(cq) = c.body.as_ref() {
            walk_query(cq, f);
        }
    }
    walk_body(&q.body, f);
}

fn walk_body(b: &QueryBody, f: &mut dyn FnMut(&Select)) {
    match b {
        QueryBody::Select(s) => {
            f(s);
            for fi in &s.from {
                walk_from(fi, f);
            }
        }
        QueryBody::SetOp { left, right, .. } => {
            walk_body(left, f);
            walk_body(right, f);
        }
        QueryBody::Nested(q) => walk_query(q, f),
        QueryBody::Values(_) | QueryBody::Table(_) => {}
    }
}

fn walk_from(fi: &FromItem, f: &mut dyn FnMut(&Select)) {
    match fi {
        FromItem::Sub { query, .. } => walk_query(query, f),
        FromItem::Join { left, right, .. } => {
            walk_from(left, f);
            walk_from(right, f);
        }
        _ => {}
    }
}

fn walk_select_exprs(s: &Select, f: &mut dyn FnMut(&Expr)) {
    for i in &s.items {
        walk_expr(&i.expr, f);
    }
    for x in s
        .where_
        .iter()
        .chain(s.having.iter())
        .chain(s.group_by.iter())
    {
        walk_expr(x, f);
    }
    for fi in &s.from {
        join_conditions(fi, f);
    }
}

fn join_conditions(fi: &FromItem, f: &mut dyn FnMut(&Expr)) {
    if let FromItem::Join {
        left, right, on, ..
    } = fi
    {
        if let Some(e) = on {
            walk_expr(e, f);
        }
        join_conditions(left, f);
        join_conditions(right, f);
    }
}

/// The relations a query reads by name.
fn relations(q: &Query) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    fn from(fi: &FromItem, out: &mut BTreeSet<String>) {
        match fi {
            FromItem::Table { name, .. } => {
                out.insert(name.last().to_string());
            }
            FromItem::Join { left, right, .. } => {
                from(left, out);
                from(right, out);
            }
            _ => {}
        }
    }
    walk_query(q, &mut |s| {
        for fi in &s.from {
            from(fi, &mut out);
        }
    });
    if let QueryBody::Table(n) = &q.body {
        out.insert(n.last().to_string());
    }
    out
}

/// A predicate position: `where`, `having`, a join condition — anything that decides which
/// rows exist, as opposed to what a surviving row shows.
fn predicates(q: &Query) -> Vec<Expr> {
    let mut v = Vec::new();
    walk_query(q, &mut |s| {
        v.extend(s.where_.iter().cloned());
        v.extend(s.having.iter().cloned());
        for fi in &s.from {
            join_conditions(fi, &mut |e| v.push(e.clone()));
        }
    });
    v
}

// ── the rules ─────────────────────────────────────────────────────────────────────────────

pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let cat = Catalog::of(stmts);
    let mut d = Vec::new();

    for (view, c) in &cat.contracts {
        let Some((q, vspan)) = cat.views.get(view) else {
            d.push(Diag {
                code: "NL0100",
                error: true,
                msg: format!("a serve contract names `{view}`, which is not a view in this script"),
                span: c.span,
                defect: None,
            });
            continue;
        };
        let (r, span) = (rung(&c.consistency), *vspan);
        // NL0220: the strictest rung cannot be served from spilled state.
        if c.consistency == "ledger_consistent" && c.materialize == "spilled" {
            d.push(Diag {
                code: "NL0220",
                error: true,
                msg: format!("view `{view}` cannot be `ledger_consistent` and `spilled`"),
                span,
                defect: None,
            });
        }
        // NL0222: a non-incremental tail — `order by`/`limit`/`offset` on the view's own
        // query, not inside a subquery — cannot be demand-materialised at serializable or
        // above.
        let tail = !q.order_by.is_empty() || q.limit.is_some() || q.offset.is_some();
        if tail && c.materialize == "demand" && r.is_some_and(|r| r >= 4) {
            d.push(Diag {
                code: "NL0222",
                error: true,
                msg: format!(
                    "view `{view}` ends in a non-incremental stage and cannot be demand-materialized at `{}`",
                    c.consistency
                ),
                span,
                defect: None,
            });
        }
        // NL0223: an evictable view over a base table whose group key no anchor index covers.
        let evictable = c.retain != "pinned" && c.materialize != "full";
        if evictable {
            if let QueryBody::Select(s) = &q.body {
                let keys: Vec<String> = s
                    .group_by
                    .iter()
                    .filter_map(|g| match g {
                        Expr::Col(n, _) => Some(n.last().to_string()),
                        _ => None,
                    })
                    .collect();
                for t in relations(q)
                    .into_iter()
                    .filter(|t| cat.tables.contains_key(t))
                {
                    if keys.is_empty() {
                        continue;
                    }
                    let covered = cat.indexes.iter().any(|(i, (it, cols))| {
                        it == &t
                            && cat.anchor_indexes.contains(i)
                            && keys.iter().all(|k| cols.contains(k))
                    });
                    if !covered {
                        d.push(Diag {
                            code: "NL0223",
                            error: false,
                            msg: format!(
                                "no anchor index on `{t}` covers ({}) — view `{view}` cannot reconstruct an evicted key by index",
                                keys.join(", ")
                            ),
                            span,
                            defect: Some("d13"),
                        });
                    }
                }
            }
        }
        // NL0311: a view may not promise more than the views it reads.
        for parent in relations(q) {
            if let (Some(pc), Some(r)) = (cat.contracts.get(&parent), r) {
                if rung(&pc.consistency).is_some_and(|pr| r > pr) {
                    d.push(Diag {
                        code: "NL0311",
                        error: true,
                        msg: format!(
                            "view `{view}` is served at `{}` but reads `{parent}`, served at `{}`: staleness enters upstream",
                            c.consistency, pc.consistency
                        ),
                        span,
                        defect: Some("d5"),
                    });
                }
            }
        }
        // IR013: a non-immutable call in a predicate leaves no reproducible reconstruction path.
        for p in predicates(q) {
            let mut calls = Vec::new();
            calls_in(&p, &mut calls);
            for (f, s) in calls {
                if cat.volatile(&f) {
                    d.push(Diag {
                        code: "IR013",
                        error: true,
                        msg: format!(
                            "view `{view}` has no reproducible reconstruction path: its predicate calls `{f}`, which is not immutable"
                        ),
                        span: s,
                        defect: Some("d6/d7"),
                    });
                }
            }
        }
        // NL0260 is `check2.rs`'s: a predicate on a confidential column anywhere, not only
        // in a contracted view.
    }

    // NL0230: an update or delete against a ledger, anywhere — top level, in a function
    // body (SQL or PL/pgSQL), or in an anonymous block.
    let mut dml = Vec::new();
    for s in stmts {
        collect_dml(s, &mut dml);
    }
    for (verb, table, span) in dml {
        if cat.ledgers.contains(&table) {
            d.push(Diag {
                code: "NL0230",
                error: true,
                msg: format!("`{verb}` against `{table}`, a ledger: history is append-only"),
                span,
                defect: Some("d11"),
            });
        }
    }
    d
}

/// Every update/delete in a statement, with the verb, target table and span; recurses into
/// function bodies, `do` blocks, CTEs.
pub fn collect_dml(s: &Stmt, out: &mut Vec<(&'static str, String, Span)>) {
    match s {
        Stmt::Update(u) => {
            out.push(("update", base(&u.table), u.span));
            for c in &u.with {
                collect_dml(&c.body, out);
            }
        }
        Stmt::Delete(x) => {
            out.push(("delete", base(&x.table), x.span));
            for c in &x.with {
                collect_dml(&c.body, out);
            }
        }
        Stmt::Insert(i) => {
            for c in &i.with {
                collect_dml(&c.body, out);
            }
        }
        Stmt::Query(q) => {
            for c in &q.with {
                collect_dml(&c.body, out);
            }
        }
        Stmt::CreateFunction(f) => {
            if let Some(b) = &f.plpgsql {
                collect_block(b, out);
            }
            match &f.body {
                Some(FuncBody::Atomic(v)) => {
                    for x in v {
                        collect_dml(x, out);
                    }
                }
                Some(FuncBody::Text(text, span)) if f.language.as_deref() == Some("sql") => {
                    // A `language sql` body is SQL text: parse it and look inside.
                    if let Ok((inner, _)) = crate::parse(text) {
                        for x in &inner {
                            let mut sub = Vec::new();
                            collect_dml(x, &mut sub);
                            out.extend(sub.into_iter().map(|(v, t, _)| (v, t, *span)));
                        }
                    }
                }
                _ => {}
            }
        }
        Stmt::Do { block, .. } => collect_block(block, out),
        _ => {}
    }
}

pub fn collect_block(b: &Block, out: &mut Vec<(&'static str, String, Span)>) {
    for s in &b.body {
        collect_pl(s, out);
    }
    for (_, h) in &b.handlers {
        for s in h {
            collect_pl(s, out);
        }
    }
}

fn collect_pl(s: &PlStmt, out: &mut Vec<(&'static str, String, Span)>) {
    match s {
        PlStmt::Sql { stmt, .. } => collect_dml(stmt, out),
        PlStmt::If {
            branches,
            otherwise,
            ..
        } => {
            for (_, b) in branches {
                b.iter().for_each(|x| collect_pl(x, out));
            }
            if let Some(o) = otherwise {
                o.iter().for_each(|x| collect_pl(x, out));
            }
        }
        PlStmt::Case {
            whens, otherwise, ..
        } => {
            for (_, b) in whens {
                b.iter().for_each(|x| collect_pl(x, out));
            }
            if let Some(o) = otherwise {
                o.iter().for_each(|x| collect_pl(x, out));
            }
        }
        PlStmt::Loop { body, kind, .. } => {
            let _ = matches!(kind.as_ref(), LoopKind::Plain);
            body.iter().for_each(|x| collect_pl(x, out));
        }
        PlStmt::Block(b) => collect_block(b, out),
        _ => {}
    }
}
