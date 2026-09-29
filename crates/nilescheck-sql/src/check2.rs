//! **The rules that need every statement, not only the contracted views**: money typed by
//! currency (NL0250, NL0240), materialisation of a non-reproducible definition (NL0310),
//! conservation removed from a ledger (NL0211), mutation of a ledger by `truncate`, `merge`
//! or a rule (NL0230, extending `check.rs`'s update/delete), and a predicate on a
//! confidential column anywhere — a view, a function, an anonymous block (NL0260).
//!
//! # The money discipline these rules read
//!
//! A currency is a composite type of one minor-unit field, declared a currency by comment:
//! `create type usd as (minor bigint); comment on type usd is 'currency scale=2';`. With it,
//! PostgreSQL itself refuses `usd + eur` in a view (no such operator) — but only where it
//! resolves operators at DDL time; inside PL/pgSQL it resolves them when the statement first
//! runs. The checker infers the currency of column references, casts and aggregates and
//! refuses the mixed operation wherever it is written.

use crate::ast::*;
use crate::check::{walk_expr, walk_query, Diag, NON_IMMUTABLE_BUILTINS};
use crate::lex::Span;
use crate::plpgsql::{Block, LoopKind, PlStmt};
use std::collections::{BTreeMap, BTreeSet};

/// What the typed rules know about the script.
#[derive(Debug, Default)]
pub struct Types {
    /// currency type → scale.
    pub currencies: BTreeMap<String, u32>,
    /// (table, column) → currency type.
    pub columns: BTreeMap<(String, String), String>,
    pub ledgers: BTreeSet<String>,
    /// (table, column) → label.
    pub labels: BTreeMap<(String, String), String>,
    /// view → (query, materialized).
    pub views: BTreeMap<String, (Query, bool)>,
    pub functions: BTreeMap<String, String>,
}

fn last(n: &Name) -> String {
    n.last().to_string()
}

impl Types {
    pub fn of(stmts: &[Stmt]) -> Types {
        let mut t = Types::default();
        let mut composite: BTreeSet<String> = BTreeSet::new();
        for s in stmts {
            match s {
                Stmt::CreateType {
                    name,
                    kind,
                    attributes,
                    ..
                } if kind == "composite" && attributes.len() == 1 => {
                    composite.insert(last(name));
                }
                Stmt::Comment {
                    kind,
                    target,
                    text: Some(text),
                    ..
                } => match kind.as_str() {
                    "type" => {
                        if let Some(rest) = text.strip_prefix("currency") {
                            let scale = rest
                                .split_whitespace()
                                .find_map(|w| w.strip_prefix("scale="))
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(2);
                            if composite.contains(&last(target)) {
                                t.currencies.insert(last(target), scale);
                            }
                        }
                    }
                    "table" if text == "ledger" => {
                        t.ledgers.insert(last(target));
                    }
                    "column" => {
                        let p = &target.0;
                        if p.len() >= 2 {
                            t.labels.insert(
                                (p[p.len() - 2].clone(), p[p.len() - 1].clone()),
                                text.clone(),
                            );
                        }
                    }
                    _ => {}
                },
                Stmt::CreateView(v) => {
                    t.views
                        .insert(last(&v.name), ((*v.query).clone(), v.materialized));
                }
                Stmt::CreateFunction(f) => {
                    t.functions.insert(
                        last(&f.name),
                        f.volatility.clone().unwrap_or_else(|| "volatile".into()),
                    );
                }
                _ => {}
            }
        }
        for s in stmts {
            if let Stmt::CreateTable(ct) = s {
                for c in &ct.columns {
                    if t.currencies.contains_key(c.ty.name.last()) {
                        t.columns.insert(
                            (last(&ct.name), c.name.clone()),
                            c.ty.name.last().to_string(),
                        );
                    }
                }
            }
        }
        t
    }

    /// The currency of an expression, when it has exactly one.
    pub fn currency_of(&self, e: &Expr) -> Option<String> {
        match e {
            Expr::Col(n, _) => {
                let col = n.last();
                let mut found: BTreeSet<&String> = BTreeSet::new();
                for ((_, c), cur) in &self.columns {
                    if c == col {
                        found.insert(cur);
                    }
                }
                if found.len() == 1 {
                    found.into_iter().next().cloned()
                } else {
                    None
                }
            }
            Expr::Cast(_, ty, _) if self.currencies.contains_key(ty.name.last()) => {
                Some(ty.name.last().to_string())
            }
            Expr::Func(f)
                if ["sum", "min", "max", "coalesce"].contains(&f.name.last())
                    && !f.args.is_empty() =>
            {
                self.currency_of(&f.args[0].1)
            }
            Expr::Bin(a, op, b, _) if op == "+" || op == "-" => {
                let (x, y) = (self.currency_of(a), self.currency_of(b));
                if x == y {
                    x
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn volatile(&self, f: &str) -> bool {
        match self.functions.get(f) {
            Some(v) => v != "immutable",
            None => NON_IMMUTABLE_BUILTINS.contains(&f),
        }
    }

    /// Whether a view's definition — its own predicates, or those of any view it reads —
    /// calls a function that is not immutable.
    fn non_reproducible(&self, view: &str, seen: &mut BTreeSet<String>) -> Option<String> {
        if !seen.insert(view.to_string()) {
            return None;
        }
        let (q, _) = self.views.get(view)?;
        let mut hit = None;
        walk_query(q, &mut |s| {
            for p in s.where_.iter().chain(s.having.iter()) {
                walk_expr(p, &mut |x| {
                    if let Expr::Func(f) = x {
                        if hit.is_none() && self.volatile(f.name.last()) {
                            hit = Some(f.name.last().to_string());
                        }
                    }
                });
            }
        });
        if hit.is_some() {
            return hit;
        }
        let mut rels = Vec::new();
        walk_query(q, &mut |s| {
            for fi in &s.from {
                if let FromItem::Table { name, .. } = fi {
                    rels.push(last(name));
                }
            }
        });
        rels.into_iter()
            .find_map(|r| self.non_reproducible(&r, seen))
    }
}

/// Everything a statement contains that a rule may look at, with the span to report at.
pub enum Item<'a> {
    Query(&'a Query, Span),
    Expr(&'a Expr, Span),
    Stmt(&'a Stmt, Span),
    /// A dynamic statement (`execute`, `return query execute`, `for … in execute`): parsed as
    /// an expression, but the SQL it will run exists only at run time.
    Dynamic(Span),
}

/// Visit every statement, query and top-level expression in a statement, descending into
/// function bodies (`language sql` text and PL/pgSQL), anonymous blocks, rules and CTEs.
/// `at` is the span a diagnostic inside a re-parsed `language sql` body is reported at.
pub fn visit(s: &Stmt, at: Option<Span>, f: &mut dyn FnMut(Item)) {
    let sp = |x: Span| at.unwrap_or(x);
    f(Item::Stmt(s, sp(s.span())));
    match s {
        Stmt::Query(q) => f(Item::Query(q, sp(q.span))),
        Stmt::Insert(i) => {
            if let Some(q) = &i.source {
                f(Item::Query(q, sp(q.span)));
            }
            for c in &i.with {
                visit(&c.body, at, f);
            }
        }
        Stmt::Update(u) => {
            for (_, e) in &u.set {
                f(Item::Expr(e, sp(u.span)));
            }
            if let Some(w) = &u.where_ {
                f(Item::Expr(w, sp(u.span)));
            }
        }
        Stmt::Delete(x) => {
            if let Some(w) = &x.where_ {
                f(Item::Expr(w, sp(x.span)));
            }
        }
        Stmt::CreateView(v) => f(Item::Query(&v.query, sp(v.span))),
        Stmt::CreateFunction(func) => {
            if let Some(b) = &func.plpgsql {
                visit_block(b, at, f);
            }
            match &func.body {
                Some(FuncBody::Text(text, span)) if func.language.as_deref() == Some("sql") => {
                    if let Ok((inner, _)) = crate::parse(text) {
                        for x in &inner {
                            visit(x, Some(*span), f);
                        }
                    }
                }
                Some(FuncBody::Atomic(v)) => {
                    for x in v {
                        visit(x, at, f);
                    }
                }
                Some(FuncBody::Return(e)) => f(Item::Expr(e, sp(func.span))),
                _ => {}
            }
        }
        Stmt::Do { block, .. } => visit_block(block, at, f),
        Stmt::CreateRule { actions, .. } => {
            for x in actions {
                visit(x, at, f);
            }
        }
        Stmt::Explain { stmt, .. } => visit(stmt, at, f),
        _ => {}
    }
}

pub fn visit_block(b: &Block, at: Option<Span>, f: &mut dyn FnMut(Item)) {
    for d in &b.decls {
        if let Some(e) = &d.default {
            f(Item::Expr(e, at.unwrap_or(d.span)));
        }
    }
    for s in b
        .body
        .iter()
        .chain(b.handlers.iter().flat_map(|(_, h)| h.iter()))
    {
        visit_pl(s, at, f);
    }
}

fn visit_pl(s: &PlStmt, at: Option<Span>, f: &mut dyn FnMut(Item)) {
    let sp = |x: Span| at.unwrap_or(x);
    match s {
        PlStmt::Sql { stmt, .. } => visit(stmt, at, f),
        PlStmt::Perform(q, span) => f(Item::Query(q, sp(*span))),
        PlStmt::Assign { value, span, .. } => f(Item::Expr(value, sp(*span))),
        PlStmt::Return(Some(e), span) | PlStmt::ReturnNext(Some(e), span) => {
            f(Item::Expr(e, sp(*span)))
        }
        PlStmt::ReturnQuery {
            query: Some(q),
            span,
            ..
        } => f(Item::Query(q, sp(*span))),
        PlStmt::If {
            branches,
            otherwise,
            span,
        } => {
            for (c, b) in branches {
                f(Item::Expr(c, sp(*span)));
                b.iter().for_each(|x| visit_pl(x, at, f));
            }
            if let Some(o) = otherwise {
                o.iter().for_each(|x| visit_pl(x, at, f));
            }
        }
        PlStmt::Case {
            whens, otherwise, ..
        } => {
            for (_, b) in whens {
                b.iter().for_each(|x| visit_pl(x, at, f));
            }
            if let Some(o) = otherwise {
                o.iter().for_each(|x| visit_pl(x, at, f));
            }
        }
        PlStmt::Loop {
            kind, body, span, ..
        } => {
            match kind.as_ref() {
                LoopKind::While(c) => f(Item::Expr(c, sp(*span))),
                LoopKind::ForQuery { query, .. } => f(Item::Query(query, sp(*span))),
                LoopKind::ForExecute { .. } => f(Item::Dynamic(sp(*span))),
                _ => {}
            }
            body.iter().for_each(|x| visit_pl(x, at, f));
        }
        PlStmt::Block(b) => visit_block(b, at, f),
        PlStmt::Execute { span, .. }
        | PlStmt::ReturnQuery {
            execute: Some(_),
            span,
            ..
        } => f(Item::Dynamic(sp(*span))),
        _ => {}
    }
}

/// Every expression inside a query: select items, predicates, grouping, join conditions.
pub(crate) fn query_exprs<'q>(q: &'q Query, out: &mut Vec<&'q Expr>) {
    fn from_on<'q>(fi: &'q FromItem, out: &mut Vec<&'q Expr>) {
        if let FromItem::Join {
            left, right, on, ..
        } = fi
        {
            if let Some(e) = on {
                out.push(e);
            }
            from_on(left, out);
            from_on(right, out);
        }
    }
    let mut sels: Vec<&'q Select> = Vec::new();
    fn collect<'q>(b: &'q QueryBody, sels: &mut Vec<&'q Select>) {
        match b {
            QueryBody::Select(s) => {
                sels.push(s);
                for fi in &s.from {
                    if let FromItem::Sub { query, .. } = fi {
                        collect(&query.body, sels);
                    }
                }
            }
            QueryBody::SetOp { left, right, .. } => {
                collect(left, sels);
                collect(right, sels);
            }
            QueryBody::Nested(q) => collect(&q.body, sels),
            _ => {}
        }
    }
    collect(&q.body, &mut sels);
    for s in sels {
        out.extend(s.items.iter().map(|i| &i.expr));
        out.extend(s.where_.iter());
        out.extend(s.having.iter());
        out.extend(s.group_by.iter());
        for fi in &s.from {
            from_on(fi, out);
        }
    }
    if let QueryBody::Values(rows) = &q.body {
        for r in rows {
            out.extend(r.iter());
        }
    }
}

/// The predicate expressions of a query (where, having, join conditions), for NL0260.
fn predicates<'q>(q: &'q Query) -> Vec<(&'q Expr, BTreeSet<String>)> {
    let mut v = Vec::new();
    fn rels(fi: &FromItem, out: &mut BTreeSet<String>) {
        match fi {
            FromItem::Table { name, .. } => {
                out.insert(name.last().to_string());
            }
            FromItem::Join { left, right, .. } => {
                rels(left, out);
                rels(right, out);
            }
            _ => {}
        }
    }
    let mut stack: Vec<&'q QueryBody> = vec![&q.body];
    while let Some(b) = stack.pop() {
        match b {
            QueryBody::Select(s) => {
                let mut r = BTreeSet::new();
                for fi in &s.from {
                    rels(fi, &mut r);
                    if let FromItem::Sub { query, .. } = fi {
                        stack.push(&query.body);
                    }
                }
                for p in s.where_.iter().chain(s.having.iter()) {
                    v.push((p, r.clone()));
                }
            }
            QueryBody::SetOp { left, right, .. } => {
                stack.push(left);
                stack.push(right);
            }
            QueryBody::Nested(q) => stack.push(&q.body),
            _ => {}
        }
    }
    v
}

pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let t = Types::of(stmts);
    let mut d = Vec::new();
    let mut seen_spans: BTreeSet<(usize, usize, &'static str)> = BTreeSet::new();
    let mut push = |d: &mut Vec<Diag>, x: Diag| {
        if seen_spans.insert((x.span.start, x.span.end, x.code)) {
            d.push(x);
        }
    };

    for s in stmts {
        let mut found: Vec<Diag> = Vec::new();
        visit(s, None, &mut |item| {
            let exprs: Vec<(&Expr, Span)> = match &item {
                Item::Expr(e, sp) => vec![(*e, *sp)],
                Item::Query(q, sp) => {
                    let mut v = Vec::new();
                    query_exprs(q, &mut v);
                    v.into_iter().map(|e| (e, *sp)).collect()
                }
                Item::Stmt(..) => vec![],
                Item::Dynamic(sp) => {
                    // A named limit, not a pass: the text `execute` runs is built at run time.
                    found.push(Diag {
                        code: "NSQ001",
                        error: false,
                        msg: "dynamic SQL: `execute` is parsed, but the statement it runs is a run-time string and no rule here has checked it".into(),
                        span: *sp,
                        defect: None,
                    });
                    vec![]
                }
            };
            for (e, sp) in exprs {
                walk_expr(e, &mut |x| {
                    // NL0250: an arithmetic or comparison operator across two currencies.
                    if let Expr::Bin(a, op, b, _) = x {
                        if ["+", "-", "<", ">", "=", "<=", ">=", "<>"].contains(&op.as_str()) {
                            if let (Some(ca), Some(cb)) = (t.currency_of(a), t.currency_of(b)) {
                                if ca != cb {
                                    found.push(Diag {
                                        code: "NL0250",
                                        error: true,
                                        msg: format!("`{op}` between `{ca}` and `{cb}`: amounts in different currencies"),
                                        span: sp,
                                        defect: Some("d2"),
                                    });
                                }
                            }
                        }
                    }
                    // NL0240: a currency value built from a literal that is not a whole number
                    // of minor units — PostgreSQL would round it into the bigint silently.
                    if let Expr::Cast(inner, ty, _) = x {
                        if let Some(scale) = t.currencies.get(ty.name.last()) {
                            let lit = match inner.as_ref() {
                                Expr::Row(v, _) if v.len() == 1 => Some(&v[0]),
                                other => Some(other),
                            };
                            if let Some(Expr::Lit(Literal::Num(n), _)) = lit {
                                if n.split_once('.')
                                    .is_some_and(|(_, f)| !f.trim_end_matches('0').is_empty())
                                {
                                    found.push(Diag {
                                        code: "NL0240",
                                        error: true,
                                        msg: format!(
                                            "`{n}` minor units of `{}` (scale {scale}): a currency value is a whole number of minor units",
                                            ty.name.last()
                                        ),
                                        span: sp,
                                        defect: Some("d3"),
                                    });
                                }
                            }
                        }
                    }
                });
            }
            // NL0260: a predicate on a confidential column, in any query.
            if let Item::Query(q, sp) = &item {
                for (p, rels) in predicates(q) {
                    walk_expr(p, &mut |x| {
                        if let Expr::Col(n, _) = x {
                            for ((tbl, col), label) in &t.labels {
                                if col == n.last()
                                    && rels.contains(tbl)
                                    && label.starts_with("confidential")
                                {
                                    found.push(Diag {
                                        code: "NL0260",
                                        error: true,
                                        msg: format!(
                                            "a predicate on `{tbl}.{col}`, which is `{label}`"
                                        ),
                                        span: *sp,
                                        defect: Some("d8"),
                                    });
                                }
                            }
                        }
                    });
                }
            }
            // NL0230 (extended): truncate, merge that updates or deletes, a rule rewriting
            // writes on a ledger.
            if let Item::Stmt(st, sp) = &item {
                match st {
                    Stmt::Truncate { tables, .. } => {
                        for tb in tables {
                            if t.ledgers.contains(tb.last()) {
                                found.push(Diag {
                                    code: "NL0230",
                                    error: true,
                                    msg: format!(
                                        "`truncate` of `{}`, a ledger: history is append-only",
                                        tb.last()
                                    ),
                                    span: *sp,
                                    defect: Some("d11"),
                                });
                            }
                        }
                    }
                    Stmt::Merge(m)
                        if t.ledgers.contains(m.table.last())
                            && m.whens.iter().any(|w| {
                                matches!(w.action, MergeAction::Update(_) | MergeAction::Delete)
                            }) =>
                    {
                        found.push(Diag {
                            code: "NL0230",
                            error: true,
                            msg: format!(
                                "`merge` that updates or deletes `{}`, a ledger",
                                m.table.last()
                            ),
                            span: *sp,
                            defect: Some("d11"),
                        });
                    }
                    Stmt::CreateRule {
                        table,
                        event,
                        instead,
                        ..
                    } if t.ledgers.contains(table.last()) && (*instead || event != "select") => {
                        found.push(Diag {
                            code: "NL0230",
                            error: true,
                            msg: format!("a rule on `{event}` to `{}`, a ledger: a rule can turn an append into something else", table.last()),
                            span: *sp,
                            defect: Some("d11"),
                        });
                    }
                    _ => {}
                }
            }
        });
        for x in found {
            push(&mut d, x);
        }
    }

    // NL0310: a materialized view whose definition is not reproducible.
    for (v, (_, materialized)) in &t.views {
        if *materialized {
            if let Some(func) = t.non_reproducible(v, &mut BTreeSet::new()) {
                let span = stmts
                    .iter()
                    .find_map(|s| match s {
                        Stmt::CreateView(cv) if cv.name.last() == v => Some(cv.span),
                        _ => None,
                    })
                    .unwrap_or_default();
                push(&mut d, Diag {
                    code: "NL0310",
                    error: true,
                    msg: format!("materialized view `{v}` stores the result of a definition that calls `{func}`, which is not immutable: the stored rows are true only when they were computed"),
                    span,
                    defect: Some("d7"),
                });
            }
        }
    }

    // NL0211: a ledger left with no conservation trigger by a drop or disable in the script.
    let mut triggers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for s in stmts {
        match s {
            Stmt::CreateTrigger(tr) if tr.constraint && t.ledgers.contains(tr.table.last()) => {
                triggers
                    .entry(tr.table.last().to_string())
                    .or_default()
                    .insert(tr.name.clone());
            }
            Stmt::Drop {
                kind, names, span, ..
            } if kind == "trigger" && names.len() >= 2 => {
                let (trig, table) = (names[0].last().to_string(), names[1].last().to_string());
                if t.ledgers.contains(&table) {
                    let set = triggers.entry(table.clone()).or_default();
                    if set.remove(&trig) && set.is_empty() {
                        push(&mut d, Diag {
                            code: "NL0211",
                            error: true,
                            msg: format!("`drop trigger {trig}` leaves `{table}`, a ledger, with no conservation trigger"),
                            span: *span,
                            defect: Some("d10"),
                        });
                    }
                }
            }
            Stmt::Alter {
                kind,
                name,
                action,
                span,
            } if kind == "table" && t.ledgers.contains(name.last()) => {
                if let Some(rest) = action.trim().strip_prefix("disable trigger") {
                    let which = rest.trim().trim_end_matches(';').to_string();
                    let set = triggers.entry(name.last().to_string()).or_default();
                    let gone = if which == "all" || which == "user" {
                        !std::mem::take(set).is_empty()
                    } else {
                        set.remove(&which) && set.is_empty()
                    };
                    if gone {
                        push(&mut d, Diag {
                            code: "NL0211",
                            error: true,
                            msg: format!("`disable trigger {which}` leaves `{}`, a ledger, with no conservation trigger", name.last()),
                            span: *span,
                            defect: Some("d10"),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    for l in &t.ledgers {
        let declared = stmts
            .iter()
            .any(|s| matches!(s, Stmt::CreateTrigger(tr) if tr.constraint && tr.table.last() == l));
        if !declared {
            let span = stmts
                .iter()
                .find_map(|s| match s {
                    Stmt::CreateTable(ct) if ct.name.last() == l => Some(ct.span),
                    _ => None,
                })
                .unwrap_or_default();
            push(
                &mut d,
                Diag {
                    code: "NL0211",
                    error: true,
                    msg: format!("`{l}` is a ledger with no conservation constraint trigger"),
                    span,
                    defect: Some("d10"),
                },
            );
        }
    }
    d
}
