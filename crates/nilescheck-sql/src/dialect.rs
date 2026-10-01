//! **The dialect's schema rule at check time** (cycle 16, C16-10; `docs/study/E30c-design.md` §3).
//!
//! The event trigger in `corpus/e30c/schema/dialect.sql` refuses D1–D3 inside PostgreSQL. This
//! pass refuses the same from the script (K1), because a superuser, `event_triggers = off` or
//! single-user mode can bypass the trigger, and adds what no trigger can see at `CREATE` time:
//! a money-bearing `state` table not keyed by a linear or capability handle (K2), a function
//! role naming an undeclared kind or effect (K3), and a currency without a scale (K4).
//!
//! Placement declares: a composite type in schema `currency` is a currency; a table in `ledger`
//! is a ledger; a table in `state` holds unposted money; a domain in `linear` or `capability` is
//! a linear handle or a capability, its kind or effect being its name. A column is money-bearing
//! when its type is a currency, an array of one, or a domain over one; nothing else is money.

use crate::ast::*;
use crate::check::Diag;
use crate::lex::Span;
use std::collections::{BTreeMap, BTreeSet};

const MONEY_SCHEMAS: [&str; 2] = ["ledger", "state"];

#[derive(Debug, Clone, Default)]
struct Table {
    schema: String,
    /// Every column and its qualified type name.
    cols: Vec<(String, String)>,
    /// The primary key's columns.
    pk: Vec<String>,
}

#[derive(Default)]
struct Model {
    /// Qualified currency type → its declared scale, if any, and where it was created.
    currencies: BTreeMap<String, (Option<u32>, Span)>,
    /// Qualified domains whose base type is money.
    money_domains: BTreeSet<String>,
    linear: BTreeSet<String>,
    capability: BTreeSet<String>,
    /// Qualified table → its shape.
    tables: BTreeMap<String, Table>,
    /// Functions' role comments: (role, kind or effect, span).
    roles: Vec<(String, String, Span)>,
}

/// `schema.name`, with an unqualified name in `public`.
fn qual(n: &Name) -> (String, String) {
    let p = &n.0;
    let last = p.last().cloned().unwrap_or_default();
    let schema = if p.len() >= 2 {
        p[p.len() - 2].clone()
    } else {
        "public".to_string()
    };
    (schema, last)
}

fn dotted(n: &Name) -> String {
    let (s, l) = qual(n);
    format!("{s}.{l}")
}

/// A type's qualified name, as the model keys it. An unqualified built-in stays bare.
fn type_key(t: &TypeName) -> String {
    if t.name.0.len() >= 2 {
        dotted(&t.name)
    } else {
        t.name.last().to_string()
    }
}

impl Model {
    fn is_money(&self, ty: &str) -> bool {
        self.currencies.contains_key(ty) || self.money_domains.contains(ty)
    }

    fn money_cols(&self, t: &Table) -> Vec<String> {
        t.cols
            .iter()
            .filter(|(_, ty)| self.is_money(ty))
            .map(|(c, _)| c.clone())
            .collect()
    }

    /// The columns a query's result would have, as far as they can be known: every column of
    /// every table it reads. Conservative in the direction of refusing: a `select *` from a
    /// money table yields money, and so does a projection that names one of its columns, which
    /// this reading does not distinguish (stated limit).
    fn query_cols(&self, q: &Query) -> Vec<(String, String)> {
        let mut out = Vec::new();
        self.body_cols(&q.body, &mut out);
        out
    }

    fn body_cols(&self, b: &QueryBody, out: &mut Vec<(String, String)>) {
        match b {
            QueryBody::Select(s) => {
                for f in &s.from {
                    self.cols_of_from(f, out);
                }
            }
            QueryBody::Table(n) => {
                if let Some(t) = self.tables.get(&dotted(n)) {
                    out.extend(t.cols.iter().cloned());
                }
            }
            QueryBody::SetOp { left, right, .. } => {
                self.body_cols(left, out);
                self.body_cols(right, out);
            }
            QueryBody::Nested(q) => self.body_cols(&q.body, out),
            QueryBody::Values(_) => {}
        }
    }

    fn cols_of_from(&self, f: &FromItem, out: &mut Vec<(String, String)>) {
        match f {
            FromItem::Table { name, .. } => {
                if let Some(t) = self.tables.get(&dotted(name)) {
                    out.extend(t.cols.iter().cloned());
                }
            }
            FromItem::Sub { query, .. } => self.body_cols(&query.body, out),
            FromItem::Join { left, right, .. } => {
                self.cols_of_from(left, out);
                self.cols_of_from(right, out);
            }
            FromItem::Func { .. } => {}
        }
    }
}

fn diag(code: &'static str, msg: String, span: Span) -> Diag {
    Diag {
        code,
        error: true,
        msg,
        span,
        defect: Some("schema-level omission"),
    }
}

/// D1 and D2 for one table, from its placement and its columns (K1).
fn placed(m: &Model, name: &str, t: &Table, span: Span, d: &mut Vec<Diag>) {
    let money = m.money_cols(t);
    if !money.is_empty() && !MONEY_SCHEMAS.contains(&t.schema.as_str()) {
        d.push(diag(
            "NSQ110",
            format!(
                "`{name}` holds money ({}) in schema `{}`; money is stored only in `ledger` or \
                 `state` (E30c D1, K1)",
                money.join(", "),
                t.schema
            ),
            span,
        ));
    }
    if t.schema == "ledger" {
        if money.is_empty() {
            d.push(diag(
                "NSQ111",
                format!(
                    "`{name}` is in schema `ledger` and has no money column: an amount must be a \
                     currency type, not a bare number (E30c D2, K1)"
                ),
                span,
            ));
        }
        if !t.cols.iter().any(|(c, _)| c == "txn") {
            d.push(diag(
                "NSQ111",
                format!("`{name}` is in schema `ledger` and has no `txn` column (E30c D2, K1)"),
                span,
            ));
        }
    }
}

/// The pass.
pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let mut m = Model::default();
    let mut d = Vec::new();
    for s in stmts {
        match s {
            Stmt::CreateType {
                name, kind, span, ..
            } if kind == "composite" && qual(name).0 == "currency" => {
                m.currencies.insert(dotted(name), (None, *span));
            }
            Stmt::Comment {
                kind,
                target,
                text: Some(text),
                span,
            } => {
                let key = dotted(target);
                if kind == "type" {
                    if let Some(c) = m.currencies.get_mut(&key) {
                        if let Some(n) = text
                            .strip_prefix("currency scale=")
                            .and_then(|v| v.trim().parse::<u32>().ok())
                        {
                            c.0 = Some(n);
                        }
                    }
                }
                if kind == "function" || kind == "procedure" {
                    let mut w = text.split_whitespace();
                    if let (Some(role), Some(what)) = (w.next(), w.next()) {
                        if ["produces", "consumes", "requires", "grants"].contains(&role) {
                            m.roles.push((role.to_string(), what.to_string(), *span));
                        }
                    }
                }
            }
            Stmt::CreateDomain { name, ty, .. } => {
                let (schema, last) = qual(name);
                if schema == "linear" {
                    m.linear.insert(last.clone());
                }
                if schema == "capability" {
                    m.capability.insert(last.clone());
                }
                if m.is_money(&type_key(ty)) {
                    m.money_domains.insert(dotted(name));
                }
            }
            Stmt::CreateTable(ct) => {
                let (schema, _) = qual(&ct.name);
                let mut t = Table {
                    schema,
                    ..Table::default()
                };
                for c in &ct.columns {
                    t.cols.push((c.name.clone(), type_key(&c.ty)));
                    if c.primary_key {
                        t.pk.push(c.name.clone());
                    }
                }
                for (_, c) in &ct.constraints {
                    if let TableConstraint::PrimaryKey(cols) = c {
                        t.pk.extend(cols.iter().cloned());
                    }
                }
                for src in ct.like.iter().chain(ct.partition_of.iter()) {
                    if let Some(s) = m.tables.get(&dotted(src)) {
                        t.cols.extend(s.cols.iter().cloned());
                        if t.pk.is_empty() {
                            t.pk = s.pk.clone();
                        }
                    }
                }
                if let Some(q) = &ct.as_query {
                    t.cols.extend(m.query_cols(q));
                }
                let name = dotted(&ct.name);
                placed(&m, &name, &t, ct.span, &mut d);
                m.tables.insert(name, t);
            }
            Stmt::CreateView(v) if v.materialized => {
                let (schema, _) = qual(&v.name);
                let t = Table {
                    schema,
                    cols: m.query_cols(&v.query),
                    pk: vec![],
                };
                let name = dotted(&v.name);
                placed(&m, &name, &t, Span::default(), &mut d);
                m.tables.insert(name, t);
            }
            Stmt::Query(q) => {
                if let QueryBody::Select(sel) = &q.body {
                    for target in &sel.into {
                        let (schema, _) = qual(target);
                        let t = Table {
                            schema,
                            cols: m.query_cols(q),
                            pk: vec![],
                        };
                        let name = dotted(target);
                        placed(&m, &name, &t, q.span, &mut d);
                        m.tables.insert(name, t);
                    }
                }
            }
            Stmt::Alter {
                kind,
                name,
                action,
                span,
            } if kind == "table" => {
                let key = dotted(name);
                let words: Vec<&str> = action.split_whitespace().collect();
                let Some(mut t) = m.tables.get(&key).cloned() else {
                    continue;
                };
                if let ["set", "schema", to, ..] = words.as_slice() {
                    let to = to.trim_end_matches(';').to_string();
                    let moved = format!("{to}.{}", qual(name).1);
                    t.schema = to;
                    placed(&m, &moved, &t, *span, &mut d);
                    m.tables.remove(&key);
                    m.tables.insert(moved, t);
                    continue;
                }
                if let Some(i) = words.iter().position(|w| *w == "add") {
                    let rest = &words[i + 1..];
                    let rest = if rest.first() == Some(&"column") {
                        &rest[1..]
                    } else {
                        rest
                    };
                    if let [col, ty, ..] = rest {
                        if !["constraint", "primary", "unique", "check", "foreign"].contains(col) {
                            t.cols.push((
                                col.to_string(),
                                ty.trim_end_matches([';', ',']).to_string(),
                            ));
                            placed(&m, &key, &t, *span, &mut d);
                            m.tables.insert(key.clone(), t.clone());
                        }
                    }
                }
                if let Some(i) = words.iter().position(|w| *w == "disable") {
                    if words.get(i + 1) == Some(&"trigger") && t.schema == "ledger" {
                        let trg = words
                            .get(i + 2)
                            .copied()
                            .unwrap_or("")
                            .trim_end_matches(';');
                        if trg.starts_with("dialect_") || trg == "all" {
                            d.push(diag(
                                "NSQ112",
                                format!(
                                    "`{trg}` on ledger table `{key}` is the dialect's and may not be \
                                     disabled (E30c D3, K1)"
                                ),
                                *span,
                            ));
                        }
                    }
                }
            }
            Stmt::Drop {
                kind, names, span, ..
            } if kind == "trigger" => {
                if let [trg, table, ..] = names.as_slice() {
                    let key = dotted(table);
                    let ledger = m.tables.get(&key).is_some_and(|t| t.schema == "ledger");
                    if ledger && trg.last().starts_with("dialect_") {
                        d.push(diag(
                            "NSQ112",
                            format!(
                                "`{}` on ledger table `{key}` is the dialect's and may not be \
                                 dropped (E30c D3, K1)",
                                trg.last()
                            ),
                            *span,
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    // K2: unposted money is reached through a handle.
    for (name, t) in &m.tables {
        if t.schema != "state" || m.money_cols(t).is_empty() {
            continue;
        }
        let handle = !t.pk.is_empty()
            && t.pk.iter().all(|c| {
                t.cols.iter().any(|(n, ty)| {
                    n == c && (ty.starts_with("linear.") || ty.starts_with("capability."))
                })
            });
        if !handle {
            d.push(diag(
                "NSQ113",
                format!(
                    "`{name}` holds unposted money and its primary key is not a `linear` or \
                     `capability` domain, so nothing makes a row a resource handed out once \
                     (E30c K2)"
                ),
                Span::default(),
            ));
        }
    }
    // K3: a role names a declared kind or effect.
    for (role, what, span) in &m.roles {
        let (declared, schema) = match role.as_str() {
            "produces" | "consumes" => (m.linear.contains(what), "linear"),
            _ => (m.capability.contains(what), "capability"),
        };
        if !declared {
            d.push(diag(
                "NSQ114",
                format!(
                    "a function `{role} {what}`, and no domain `{schema}.{what}` declares that \
                     {}: the role constrains nothing (E30c K3)",
                    if schema == "linear" {
                        "kind"
                    } else {
                        "capability"
                    }
                ),
                *span,
            ));
        }
    }
    // K4: every currency has a scale.
    for (name, (scale, span)) in &m.currencies {
        if scale.is_none() {
            d.push(diag(
                "NSQ115",
                format!("currency `{name}` has no `currency scale=N` comment (E30c K4)"),
                *span,
            ));
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(src: &str) -> Vec<&'static str> {
        let (stmts, _) = crate::parse(src).expect("parses");
        check(&stmts).into_iter().map(|d| d.code).collect()
    }

    const BASE: &str = "create type currency.usd as (minor bigint);\n\
        comment on type currency.usd is 'currency scale=2';\n\
        create table ledger.postings (txn bigint, acct bigint, amt currency.usd);\n";

    #[test]
    fn placement_is_the_declaration() {
        assert!(codes(BASE).is_empty(), "{:?}", codes(BASE));
        assert_eq!(
            codes(&format!(
                "{BASE}create table public.j (txn bigint, amt currency.usd);"
            )),
            ["NSQ110"]
        );
        assert_eq!(
            codes(&format!(
                "{BASE}create table ledger.f (txn bigint, amount bigint);"
            )),
            ["NSQ111"]
        );
        assert_eq!(
            codes(&format!(
                "{BASE}create table public.c (like ledger.postings);"
            )),
            ["NSQ110"]
        );
        assert_eq!(
            codes(&format!(
                "{BASE}create table public.s as select * from ledger.postings;"
            )),
            ["NSQ110"]
        );
        assert_eq!(
            codes(&format!(
                "{BASE}select * into public.d from ledger.postings;"
            )),
            ["NSQ110"]
        );
        assert_eq!(
            codes(&format!(
                "{BASE}alter table ledger.postings set schema public;"
            )),
            ["NSQ110"]
        );
        assert_eq!(
            codes(&format!(
                "{BASE}drop trigger dialect_conserve on ledger.postings;"
            )),
            ["NSQ112"]
        );
        assert_eq!(
            codes(&format!(
                "{BASE}alter table ledger.postings disable trigger dialect_append_only;"
            )),
            ["NSQ112"]
        );
    }

    #[test]
    fn the_catalog_only_clauses() {
        assert_eq!(
            codes("create type currency.gbp as (minor bigint);"),
            ["NSQ115"]
        );
        let held = format!("{BASE}create domain public.h as bigint;\ncreate table state.holds (id public.h primary key, amt currency.usd);");
        assert_eq!(codes(&held), ["NSQ113"]);
        let ok = format!("{BASE}create domain linear.hold as bigint;\ncreate table state.holds (id linear.hold primary key, amt currency.usd);");
        assert!(codes(&ok).is_empty(), "{:?}", codes(&ok));
        let role = format!("{BASE}create function f(a bigint) returns void language sql as $$ select 1 $$;\ncomment on function f(bigint) is 'requires overdraft';");
        assert_eq!(codes(&role), ["NSQ114"]);
        let declared = format!("create domain capability.overdraft as bigint;\n{role}");
        assert!(codes(&declared).is_empty(), "{:?}", codes(&declared));
    }
}
