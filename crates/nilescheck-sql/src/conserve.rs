//! **Conservation over a function's postings** (cycle 14, R2-05): the adapter that hands the
//! postings a PL/pgSQL or `language sql` body inserts into a ledger to `niles-lang`'s
//! currency-row solver (`niles_lang::currency_rows`), unchanged — the same judgement, the
//! same three verdicts, the same NL0300/NL0301.
//!
//! # What the adapter reads
//!
//! A ledger is a table commented `ledger`; a leg is one row of an `insert … values` into it.
//! Its currency is the currency type of the money column the row fills (`amt_usd usd` → usd),
//! and its amount is read as the solver reads Niles's: a literal (`row(-10000)::usd` is
//! −10000 minor units), a parameter or variable (a symbol, the same one wherever it
//! appears, so `row(-(m).minor)::usd` and `m` cancel), a sum, difference or integer multiple
//! of those, and otherwise a fresh symbol — an amount the checker cannot see through, which
//! makes the verdict *undecided* rather than an accusation. Legs are grouped into
//! transactions by the text of their `txn` value: `conserve per (txn, cur)`.
//!
//! Control flow is the solver's own: the two arms of an `if` are joined with `Row::join`, a
//! loop's body is iterated with `Row::iterate`, and a path that raises an exception commits
//! nothing and is dropped. An `insert … select` is undecided in every currency it fills.

use crate::ast::*;
use crate::check::Diag;
use crate::check2::Types;
use crate::conventions::{units, Unit, UnitBody};
use crate::lex::Span;
use crate::plpgsql::{Block, PlStmt};
use niles_lang::currency_rows::{self as cr, Amount, Cur, Row};
use std::collections::BTreeMap;

type NSpan = niles_lang::lexer::Span;

fn nspan(s: Span) -> NSpan {
    NSpan::new(s.start, s.end)
}

/// Transactions of one path: txn key → its row. `None` is an unreachable path.
type Rows = Option<BTreeMap<String, Row>>;

struct Cx<'a> {
    types: &'a Types,
    /// Parameter and variable names → symbol.
    symbols: BTreeMap<String, u32>,
    next: u32,
    /// Where each transaction first appeared, for the report.
    first: BTreeMap<String, Span>,
    at: Option<Span>,
}

impl<'a> Cx<'a> {
    fn symbol(&mut self, name: &str) -> Amount {
        let id = match self.symbols.get(name) {
            Some(i) => *i,
            None => {
                self.next += 1;
                self.symbols.insert(name.to_string(), self.next);
                self.next
            }
        };
        Amount::symbol(id)
    }

    fn fresh(&mut self) -> Amount {
        self.next += 1;
        // Offset so a fresh symbol never collides with a named one.
        Amount::symbol(1_000_000 + self.next)
    }

    /// The minor-unit amount of an integer expression.
    fn minor(&mut self, e: &Expr) -> Amount {
        match e {
            Expr::Lit(Literal::Int(v), _) => match v.parse::<i128>() {
                Ok(n) => Amount::constant(n),
                Err(_) => self.fresh(),
            },
            Expr::Un(op, x, _) if op == "-" => self.minor(x).neg(),
            Expr::Un(op, x, _) if op == "+" => self.minor(x),
            Expr::Field(x, f, _) if f == "minor" => match x.as_ref() {
                Expr::Col(n, _) if n.0.len() == 1 => self.symbol(n.last()),
                Expr::Row(v, _) if v.len() == 1 => match &v[0] {
                    Expr::Col(n, _) if n.0.len() == 1 => self.symbol(n.last()),
                    _ => self.fresh(),
                },
                _ => self.fresh(),
            },
            Expr::Row(v, _) if v.len() == 1 => self.minor(&v[0]),
            Expr::Col(n, _) if n.0.len() == 1 => self.symbol(&format!("int {}", n.last())),
            Expr::Bin(a, op, b, _) if op == "+" || op == "-" => {
                let (x, y) = (self.minor(a), self.minor(b));
                if op == "+" {
                    x.add(&y)
                } else {
                    x.add(&y.neg())
                }
            }
            Expr::Bin(a, op, b, _) if op == "*" => match (a.as_ref(), b.as_ref()) {
                (Expr::Lit(Literal::Int(k), _), x) | (x, Expr::Lit(Literal::Int(k), _)) => {
                    match k.parse::<i128>() {
                        Ok(k) => self.minor(x).scale(k),
                        Err(_) => self.fresh(),
                    }
                }
                _ => self.fresh(),
            },
            _ => self.fresh(),
        }
    }

    /// The amount a money-typed value contributes, or `None` for a SQL null (no leg).
    fn money(&mut self, e: &Expr) -> Option<Amount> {
        match e {
            Expr::Lit(Literal::Null, _) => None,
            Expr::Cast(inner, _, _) => match inner.as_ref() {
                Expr::Row(v, _) if v.len() == 1 => Some(self.minor(&v[0])),
                Expr::Lit(Literal::Null, _) => None,
                Expr::Col(n, _) if n.0.len() == 1 => Some(self.symbol(n.last())),
                _ => Some(self.fresh()),
            },
            Expr::Row(v, _) if v.len() == 1 => Some(self.minor(&v[0])),
            Expr::Col(n, _) if n.0.len() == 1 => Some(self.symbol(n.last())),
            _ => Some(self.fresh()),
        }
    }

    fn insert(&mut self, i: &Insert, rows: &mut Rows) {
        let table = i.table.last().to_string();
        if !self.types.ledgers.contains(&table) {
            return;
        }
        let Some(map) = rows.as_mut() else { return };
        let span = self.at.unwrap_or(i.span);
        // Money columns of this ledger, by position in the insert's column list.
        let cols: Vec<(usize, String)> = i
            .columns
            .iter()
            .enumerate()
            .filter_map(|(k, c)| {
                self.types
                    .columns
                    .get(&(table.clone(), c.clone()))
                    .map(|cur| (k, cur.clone()))
            })
            .collect();
        let txn_at = i.columns.iter().position(|c| c == "txn");
        // **Representation R2** (E30b′ design §3): one plain amount column beside a currency
        // text, named by the ledger's comment. A leg's currency is the row's currency value
        // when it is a declared currency's name as a string literal, and otherwise unknown
        // (its amount is then undecided); its amount is the amount expression, read as minor
        // units exactly as R1's composites are.
        let r2 = self.types.ledger_cols.get(&table).and_then(|(cc, ac)| {
            Some((
                i.columns.iter().position(|c| c == cc)?,
                i.columns.iter().position(|c| c == ac)?,
            ))
        });
        if let (Some((cur_at, amt_at)), Some(QueryBody::Values(values))) =
            (r2, i.source.as_ref().map(|q| &q.body))
        {
            for r in values {
                let rendered = txn_at.and_then(|k| r.get(k)).map(render);
                let (key, opaque) = match rendered {
                    Some(Some(k)) => (k, false),
                    Some(None) => (format!("(txn at byte {})", i.span.start), true),
                    None => ("(no txn column)".to_string(), false),
                };
                self.first.entry(key.clone()).or_insert(span);
                let Some(amt) = r.get(amt_at) else { continue };
                if matches!(amt, Expr::Lit(Literal::Null, _)) {
                    continue;
                }
                let known = match r.get(cur_at) {
                    Some(Expr::Lit(Literal::Str(c), _)) if self.types.currencies.contains_key(c) => {
                        Some(c.clone())
                    }
                    _ => None,
                };
                let a = self.minor(amt);
                let (cur, a) = match known {
                    Some(c) => (c, if opaque { a.add(&self.fresh()) } else { a }),
                    None => ("(a currency this reading cannot name)".to_string(), self.fresh()),
                };
                map.entry(key).or_default().movement(Cur::Known(cur), a, nspan(span));
            }
            return;
        }
        match i.source.as_ref().map(|q| &q.body) {
            Some(QueryBody::Values(values)) => {
                for r in values {
                    let rendered = txn_at.and_then(|k| r.get(k)).map(render);
                    let (key, opaque) = match rendered {
                        Some(Some(k)) => (k, false),
                        // A txn value this reading cannot render: its own group, undecided.
                        Some(None) => (format!("(txn at byte {})", i.span.start), true),
                        None => ("(no txn column)".to_string(), false),
                    };
                    self.first.entry(key.clone()).or_insert(span);
                    let mut legs = Vec::new();
                    for (k, cur) in &cols {
                        if let Some(e) = r.get(*k) {
                            if let Some(a) = self.money(e) {
                                let a = if opaque { a.add(&self.fresh()) } else { a };
                                legs.push((cur.clone(), a));
                            }
                        }
                    }
                    let row = map.entry(key).or_default();
                    for (cur, a) in legs {
                        row.movement(Cur::Known(cur), a, nspan(span));
                    }
                }
            }
            _ => {
                // `insert … select`: whatever it inserts, the checker cannot see the amounts.
                let key = format!("(insert … select at byte {})", i.span.start);
                self.first.entry(key.clone()).or_insert(span);
                let mut fresh: Vec<(String, Amount)> = cols
                    .iter()
                    .map(|(_, c)| (c.clone(), self.fresh()))
                    .collect();
                if r2.is_some() {
                    fresh.push((
                        "(a currency this reading cannot name)".to_string(),
                        self.fresh(),
                    ));
                }
                let row = map.entry(key).or_default();
                for (cur, a) in fresh {
                    row.movement(Cur::Known(cur), a, nspan(span));
                }
            }
        }
    }

    fn join(a: Rows, b: Rows, at: Span) -> Rows {
        match (a, b) {
            (None, x) | (x, None) => x,
            (Some(mut a), Some(mut b)) => {
                let keys: Vec<String> = a.keys().chain(b.keys()).cloned().collect();
                let mut out = BTreeMap::new();
                for k in keys {
                    if out.contains_key(&k) {
                        continue;
                    }
                    let x = a.remove(&k).unwrap_or_default();
                    let y = b.remove(&k).unwrap_or_default();
                    out.insert(k, x.join(y, nspan(at)));
                }
                Some(out)
            }
        }
    }

    fn block(&mut self, b: &Block, rows: &mut Rows) {
        let entry = rows.clone();
        self.stmts(&b.body, rows);
        for (_, h) in &b.handlers {
            // A handler runs after the body's statements were rolled back to the block's
            // savepoint: the alternative path starts from the block's entry.
            let mut hr = entry.clone();
            self.stmts(h, &mut hr);
            *rows = Self::join(rows.take(), hr, b.span);
        }
    }

    fn stmts(&mut self, v: &[PlStmt], rows: &mut Rows) {
        for s in v {
            if rows.is_none() {
                return;
            }
            self.stmt(s, rows);
        }
    }

    fn stmt(&mut self, s: &PlStmt, rows: &mut Rows) {
        match s {
            PlStmt::Sql { stmt, .. } => {
                if let Stmt::Insert(i) = stmt.as_ref() {
                    self.insert(i, rows);
                }
            }
            PlStmt::If {
                branches,
                otherwise,
                span,
            } => {
                let entry = rows.clone();
                let mut acc: Rows = None;
                let mut first = true;
                for (_, body) in branches {
                    let mut r = entry.clone();
                    self.stmts(body, &mut r);
                    acc = if first { r } else { Self::join(acc, r, *span) };
                    first = false;
                }
                let mut o = entry;
                if let Some(ob) = otherwise {
                    self.stmts(ob, &mut o);
                }
                *rows = Self::join(acc, o, *span);
            }
            PlStmt::Case {
                whens,
                otherwise,
                span,
                ..
            } => {
                let entry = rows.clone();
                let mut acc: Rows = None;
                let mut first = true;
                for (_, body) in whens {
                    let mut r = entry.clone();
                    self.stmts(body, &mut r);
                    acc = if first { r } else { Self::join(acc, r, *span) };
                    first = false;
                }
                let o = match otherwise {
                    Some(ob) => {
                        let mut o = entry.clone();
                        self.stmts(ob, &mut o);
                        o
                    }
                    None => None,
                };
                *rows = if first { o } else { Self::join(acc, o, *span) };
            }
            PlStmt::Loop { body, span, .. } => {
                // The body's own net, iterated a symbolic number of times.
                let mut inner: Rows = Some(BTreeMap::new());
                self.stmts(body, &mut inner);
                if let (Some(outer), Some(inner)) = (rows.as_mut(), inner) {
                    for (k, r) in inner {
                        let it = r.iterate(nspan(*span));
                        outer.entry(k).or_default().merge(&it);
                    }
                }
            }
            PlStmt::Block(b) => self.block(b, rows),
            PlStmt::Return(..) => {
                // What was inserted before the return commits: keep the rows, end the path
                // by folding them into the function's result below.
            }
            PlStmt::Raise { level, .. } if level.as_deref().is_none_or(|l| l == "exception") => {
                *rows = None;
            }
            _ => {}
        }
    }
}

/// A canonical text for an expression, to group legs by transaction; `None` when the
/// expression is one this reading does not render, and the legs are then undecided rather
/// than grouped by a guess.
fn render(e: &Expr) -> Option<String> {
    Some(match e {
        Expr::Lit(Literal::Int(v), _) | Expr::Lit(Literal::Num(v), _) => v.clone(),
        Expr::Lit(Literal::Str(v), _) => format!("'{v}'"),
        Expr::Col(n, _) => n.dotted(),
        Expr::Param(k, _) => format!("${k}"),
        Expr::Un(op, x, _) => format!("{op}({})", render(x)?),
        Expr::Bin(a, op, b, _) => format!("({} {op} {})", render(a)?, render(b)?),
        Expr::Cast(x, ty, _) => format!("{}::{}", render(x)?, ty.name.dotted()),
        Expr::Func(f) => {
            let mut args = Vec::new();
            for (_, a) in &f.args {
                args.push(render(a)?);
            }
            format!("{}({})", f.name.dotted(), args.join(", "))
        }
        _ => return None,
    })
}

fn check_unit(types: &Types, u: &Unit, out: &mut Vec<Diag>) {
    let mut cx = Cx {
        types,
        symbols: BTreeMap::new(),
        next: 0,
        first: BTreeMap::new(),
        at: None,
    };
    let mut rows: Rows = Some(BTreeMap::new());
    match &u.body {
        UnitBody::Pl(b) => cx.block(b, &mut rows),
        UnitBody::Sql(stmts) => {
            cx.at = Some(u.span);
            for s in stmts {
                if let Stmt::Insert(i) = s {
                    cx.insert(i, &mut rows);
                }
            }
        }
    }
    let Some(map) = rows else { return };
    for (key, row) in map {
        for v in cr::check_conservation(&row) {
            let scale = match &v {
                cr::Verdict::Violates { currency, .. }
                | cr::Verdict::MayViolate { currency, .. } => {
                    types.currencies.get(currency).copied().unwrap_or(2)
                }
                _ => 2,
            };
            if let Some(d) = cr::diagnose_with(&v, scale, row.provenance) {
                let span = cx.first.get(&key).copied().unwrap_or(u.span);
                let detail = d
                    .labels
                    .first()
                    .map(|l| format!(" — {}", l.msg))
                    .unwrap_or_default();
                out.push(Diag {
                    code: d.code,
                    error: matches!(d.severity, niles_lang::diagnostics::Severity::Error),
                    msg: format!("`{}`: {}{detail}", u.name, d.msg),
                    span,
                    defect: Some("d1"),
                });
            }
        }
    }
}

/// Every conservation diagnostic in the script: NL0300 (a transaction that cannot balance)
/// and NL0301 (one that may not, across a branch).
pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let types = Types::of(stmts);
    let mut out = Vec::new();
    if types.ledgers.is_empty() {
        return out;
    }
    for u in units(stmts) {
        check_unit(&types, &u, &mut out);
    }
    out
}
