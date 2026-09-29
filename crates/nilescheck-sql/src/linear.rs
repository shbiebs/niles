//! **Linearity over PL/pgSQL** (cycle 14, R2-05): a value of a domain marked `linear <kind>`
//! is consumed exactly once on every path that commits — the rule Niles states as W8 and
//! enforces with NL0320/NL0321/NL0322, here over PostgreSQL's own procedural language.
//!
//! # What counts
//!
//! * **Made** — a call to a function that produces the kind (`comment … 'produces hold'`,
//!   or one whose return type is the linear domain).
//! * **Bound** — assigned to a variable of the linear domain (`h := hold(..)`, `select
//!   hold(..) into h`, a `declare h hold_ref := hold(..)` default).
//! * **Consumed** — passed to a function marked `consumes <kind>`.
//! * **Moved** — passed to a function whose parameter has the linear domain (that function
//!   must then consume it, and is checked for it), assigned to another linear variable, or
//!   returned from a function whose return type is the domain.
//! * **Dropped**, which is the defect: made and not bound (`perform hold(..)`, a `select`
//!   with no destination, an argument to an ordinary function, an assignment to a variable
//!   of another type); overwritten while live (`h := null`); live when the path leaves the
//!   function (`end`, `return`); live on one arm of an `if` and not the other.
//!
//! A path that raises an exception commits nothing — the transaction rolls back and the hold
//! with it — so it carries no obligation, which is how Niles treats an aborting path too.
//! Reading a linear value (`raise notice '%', h`, a comparison, an argument to a function
//! whose parameter is not the domain) is neither a consumption nor a drop.
//!
//! # Limits, stated
//!
//! Dynamic SQL (`execute`) is opaque: a linear value used only there is reported as never
//! consumed, which is the conservative direction. A loop that consumes a value bound before
//! it is refused (NL0321): it runs any number of times, including zero and more than one.
//! A value of the linear domain built by a cast (`42::hold_ref`) is not refused here: it
//! names an existing hold rather than making one, so it is a forging question — the one the
//! capability rule answers for capabilities — and this rule does not yet ask it of linear
//! domains.

use crate::ast::*;
use crate::check::{walk_expr, Diag};
use crate::conventions::{units, Conventions, Unit, UnitBody};
use crate::lex::Span;
use crate::plpgsql::{Block, LoopKind, PlStmt};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
enum St {
    /// Declared, holding nothing.
    Empty,
    /// Holding an unconsumed value, bound at this span.
    Live(Span),
    /// Consumed or moved at this span.
    Gone(Span),
}

#[derive(Debug, Clone)]
struct Var {
    kind: String,
    st: St,
    /// The loop depth the current value was bound at, for NL0322.
    depth: usize,
}

/// `None` is an unreachable path (after a `return` or a `raise exception`).
type State = Option<BTreeMap<String, Var>>;

struct Cx<'a> {
    conv: &'a Conventions,
    /// Diagnostics inside a `language sql` body are reported here (its spans are relative).
    at: Option<Span>,
    returns_kind: Option<String>,
    depth: usize,
    out: Vec<Diag>,
    /// (variable, span) pairs already reported, so a join does not repeat itself.
    seen: BTreeSet<(String, usize)>,
}

impl<'a> Cx<'a> {
    fn sp(&self, s: Span) -> Span {
        self.at.unwrap_or(s)
    }

    fn push(&mut self, code: &'static str, msg: String, span: Span) {
        let span = self.sp(span);
        if self.seen.insert((format!("{code}{msg}"), span.start)) {
            self.out.push(Diag {
                code,
                error: true,
                msg,
                span,
                defect: Some("d14"),
            });
        }
    }

    fn describe(kind: &str) -> String {
        format!("a {kind}")
    }

    fn consumers(&self, kind: &str) -> String {
        let v: Vec<String> = self
            .conv
            .consumes
            .iter()
            .filter(|(_, k)| *k == kind)
            .map(|(f, _)| format!("`{f}`"))
            .collect();
        if v.is_empty() {
            "a function marked `consumes`".into()
        } else {
            v.join(" or ")
        }
    }

    /// The linear kind of a call expression that makes one.
    fn made(&self, e: &Expr) -> Option<(String, Span)> {
        if let Expr::Func(f) = e {
            return self.conv.produced_by(f.name.last()).map(|k| (k, f.span));
        }
        if let Expr::Cast(inner, ty, _) = e {
            // `hold(..)::hold_ref` is still the made value.
            if self.conv.linear_kind(ty.name.last()).is_some() {
                return self.made(inner);
            }
        }
        None
    }

    fn var_of(e: &Expr) -> Option<&str> {
        match e {
            Expr::Col(n, _) if n.0.len() == 1 => Some(n.last()),
            _ => None,
        }
    }

    /// Evaluate one expression for its linear effects. `sunk` is a call this expression's
    /// context binds (the right-hand side of an assignment into a linear variable), which
    /// is therefore not a drop.
    fn eval(&mut self, e: &Expr, st: &mut State, sunk: Option<*const Expr>) {
        let Some(vars) = st.as_mut() else { return };
        let mut absorbed: BTreeSet<*const Expr> = BTreeSet::new();
        if let Some(p) = sunk {
            absorbed.insert(p);
        }
        let mut events: Vec<(String, Span, &'static str)> = Vec::new();
        // Pass 1: calls that consume or move their arguments.
        walk_expr(e, &mut |x| {
            let Expr::Func(f) = x else { return };
            let name = f.name.last();
            let consumes = self.conv.consumes.get(name);
            let params = self.conv.params.get(name);
            for (i, (_, a)) in f.args.iter().enumerate() {
                let param_kind = params
                    .and_then(|p| p.get(i))
                    .and_then(|t| self.conv.linear_kind(t));
                let takes = consumes.is_some() || param_kind.is_some();
                if !takes {
                    continue;
                }
                if let Some(v) = Self::var_of(a) {
                    if vars.contains_key(v) {
                        events.push((
                            v.to_string(),
                            f.span,
                            if consumes.is_some() {
                                "consume"
                            } else {
                                "move"
                            },
                        ));
                    }
                } else if self.made(a).is_some() {
                    absorbed.insert(a as *const Expr);
                }
            }
        });
        for (v, span, _how) in events {
            let var = vars.get_mut(&v).expect("present");
            match var.st.clone() {
                St::Live(_) => {
                    if var.depth < self.depth {
                        let msg = format!(
                            "`{v}` is consumed inside a loop, so it is consumed once per iteration: none, once or many times"
                        );
                        self.out.push(Diag {
                            code: "NL0321",
                            error: true,
                            msg,
                            span: self.at.unwrap_or(span),
                            defect: Some("d14"),
                        });
                    }
                    var.st = St::Gone(span);
                }
                St::Gone(first) => {
                    let msg = format!(
                        "`{v}` is consumed twice, but must be consumed exactly once (first at byte {})",
                        first.start
                    );
                    self.out.push(Diag {
                        code: "NL0321",
                        error: true,
                        msg,
                        span: self.at.unwrap_or(span),
                        defect: Some("d14"),
                    });
                }
                St::Empty => {}
            }
        }
        // Pass 2: every made value nothing absorbed is dropped where it is made.
        let mut drops = Vec::new();
        walk_expr(e, &mut |x| {
            if absorbed.contains(&(x as *const Expr)) {
                return;
            }
            if let Some((kind, span)) = self.made(x) {
                drops.push((kind, span));
            }
        });
        // A cast wrapping a made value reports once, at the call.
        drops.dedup_by_key(|(_, s)| s.start);
        for (kind, span) in drops {
            let what = Self::describe(&kind);
            let msg = format!(
                "{what} is made here and never bound: it must be consumed exactly once, by {}",
                self.consumers(&kind)
            );
            self.push("NL0320", msg, span);
        }
    }

    /// Bind `target := value`.
    fn assign(&mut self, target: &str, value: Option<&Expr>, span: Span, st: &mut State) {
        let Some(vars) = st.as_mut() else { return };
        let target_kind = vars.get(target).map(|v| v.kind.clone());
        let made = value.and_then(|v| self.made(v));
        let moved_from = value
            .and_then(Self::var_of)
            .filter(|v| vars.contains_key(*v) && *v != target)
            .map(String::from);
        // Evaluate the right-hand side, with the made call absorbed when the target can hold it.
        if let Some(v) = value {
            // A made value on the right is bound here, well or badly: into a linear
            // variable, or (reported below) into one of another type.
            let sunk = if made.is_some() {
                Some(v as *const Expr)
            } else {
                None
            };
            let sunk = sunk.or_else(|| {
                // `h := hold(..)::hold_ref` — absorb the inner call too.
                if target_kind.is_some() {
                    if let Expr::Cast(inner, _, _) = v {
                        return Some(inner.as_ref() as *const Expr);
                    }
                }
                None
            });
            self.eval(v, st, sunk);
        }
        let Some(vars) = st.as_mut() else { return };
        match target_kind {
            Some(kind) => {
                if let Some(Var {
                    st: St::Live(_), ..
                }) = vars.get(target)
                {
                    let msg = format!(
                        "`{target}` is overwritten while it still holds {}: the value it held is dropped",
                        Self::describe(&kind)
                    );
                    self.push("NL0320", msg, span);
                }
                let incoming = if made.is_some() {
                    Some(span)
                } else if let Some(src) = &moved_from {
                    let live = matches!(vars.get(src).map(|v| &v.st), Some(St::Live(_)));
                    if live {
                        vars.get_mut(src).expect("present").st = St::Gone(span);
                        Some(span)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let depth = self.depth;
                let var = vars.get_mut(target).expect("present");
                var.st = match incoming {
                    Some(s) => St::Live(s),
                    None => St::Empty,
                };
                var.depth = depth;
            }
            None => {
                // A linear variable copied into one of another type is only read: the copy
                // is untracked, and the original is still owed.
                if let (Some((kind, _)), Some(v)) = (made, value) {
                    let msg = format!(
                        "{} is bound to `{target}`, which is not of its linear domain: the value is dropped",
                        Self::describe(&kind)
                    );
                    self.push("NL0320", msg, v.span());
                }
            }
        }
    }

    /// Everything live when a path leaves the function.
    fn leave(&mut self, st: &State, span: Span, how: &str) {
        let Some(vars) = st else { return };
        let live: Vec<(String, Span, String)> = vars
            .iter()
            .filter_map(|(n, v)| match v.st {
                St::Live(b) => Some((n.clone(), b, v.kind.clone())),
                _ => None,
            })
            .collect();
        for (n, bound, kind) in live {
            let msg = if how.is_empty() {
                format!(
                    "`{n}` is never consumed: {} must be consumed exactly once, by {}",
                    Self::describe(&kind),
                    self.consumers(&kind)
                )
            } else {
                format!("`{n}` is not consumed on this {how} path")
            };
            let at = if how.is_empty() { bound } else { span };
            self.push("NL0320", msg, at);
        }
    }

    /// Join two paths. A value live on one and not the other is consumed on some paths only.
    fn join(&mut self, a: State, b: State, span: Span) -> State {
        match (a, b) {
            (None, x) | (x, None) => x,
            (Some(mut a), Some(b)) => {
                for (n, vb) in b {
                    let va = a.get(&n).cloned();
                    match va {
                        None => {
                            a.insert(n, vb);
                        }
                        Some(va) => {
                            let la = matches!(va.st, St::Live(_));
                            let lb = matches!(vb.st, St::Live(_));
                            if la != lb {
                                let msg = format!(
                                    "`{n}` is consumed on some paths through this branch and not on others; it must be consumed exactly once on every path"
                                );
                                self.push("NL0320", msg, span);
                                a.get_mut(&n).expect("present").st = St::Gone(span);
                            }
                        }
                    }
                }
                Some(a)
            }
        }
    }

    fn block(&mut self, b: &Block, st: &mut State) {
        let entry = st.clone();
        let mut declared = Vec::new();
        for d in &b.decls {
            let ty = d.ty.as_ref().map(|t| t.name.last().to_string());
            if let Some(kind) = ty
                .as_deref()
                .and_then(|t| self.conv.linear_kind(t))
                .cloned()
            {
                if let Some(vars) = st.as_mut() {
                    vars.insert(
                        d.name.clone(),
                        Var {
                            kind,
                            st: St::Empty,
                            depth: self.depth,
                        },
                    );
                }
                declared.push(d.name.clone());
                if let Some(e) = &d.default {
                    self.assign(&d.name, Some(e), d.span, st);
                }
            } else if let Some(e) = &d.default {
                // A made value as the default of a variable of another type is dropped.
                let name = d.name.clone();
                if st.as_ref().is_some_and(|v| !v.contains_key(&name)) {
                    self.assign(&name, Some(e), d.span, st);
                }
            }
        }
        self.stmts(&b.body, st);
        // An exception handler runs instead of the rest of the body: an alternative path
        // from the block's entry, where nothing the body bound exists yet.
        for (_, h) in &b.handlers {
            let mut hs = entry.clone();
            self.stmts(h, &mut hs);
            let joined = self.join(st.take(), hs, b.span);
            *st = joined;
        }
        // A variable declared here goes out of scope at the block's end.
        if let Some(vars) = st.as_mut() {
            let mut leaving = BTreeMap::new();
            for n in &declared {
                if let Some(v) = vars.remove(n) {
                    leaving.insert(n.clone(), v);
                }
            }
            if !leaving.is_empty() {
                self.leave(&Some(leaving), b.span, "");
            }
        }
    }

    fn stmts(&mut self, v: &[PlStmt], st: &mut State) {
        for s in v {
            self.stmt(s, st);
        }
    }

    fn stmt(&mut self, s: &PlStmt, st: &mut State) {
        if st.is_none() {
            return;
        }
        match s {
            PlStmt::Assign {
                target,
                value,
                span,
            } => match Self::var_of(target) {
                Some(t) => self.assign(t, Some(value), *span, st),
                None => self.eval(value, st, None),
            },
            PlStmt::Sql { stmt, into, .. } => {
                // `select hold(..) into h`: an assignment from the one select item.
                if let (Stmt::Query(q), [target]) = (stmt.as_ref(), into.as_slice()) {
                    if let QueryBody::Select(sel) = &q.body {
                        if sel.items.len() == 1 && sel.from.is_empty() {
                            self.assign(target.last(), Some(&sel.items[0].expr), q.span, st);
                            return;
                        }
                    }
                }
                for e in crate::conventions::stmt_exprs(stmt) {
                    self.eval(e, st, None);
                }
            }
            PlStmt::Perform(q, _) => {
                let mut v = Vec::new();
                crate::check2::query_exprs(q, &mut v);
                for e in v {
                    self.eval(e, st, None);
                }
            }
            PlStmt::Return(e, span) => {
                if let Some(e) = e {
                    // Returning a linear value of the function's own return kind moves it out.
                    let moves = self.returns_kind.is_some();
                    if moves {
                        if let Some(v) = Self::var_of(e) {
                            if let Some(var) = st.as_mut().and_then(|m| m.get_mut(v)) {
                                var.st = St::Gone(*span);
                            }
                        } else {
                            self.eval(e, st, self.made(e).map(|_| e as *const Expr));
                        }
                    } else {
                        self.eval(e, st, None);
                    }
                }
                let snapshot = st.clone();
                self.leave(&snapshot, *span, "return");
                *st = None;
            }
            PlStmt::ReturnNext(Some(e), _) => self.eval(e, st, None),
            PlStmt::ReturnQuery { query: Some(q), .. } => {
                let mut v = Vec::new();
                crate::check2::query_exprs(q, &mut v);
                for e in v {
                    self.eval(e, st, None);
                }
            }
            PlStmt::Raise { level, args, .. } => {
                for a in args {
                    self.eval(a, st, None);
                }
                // `raise` with no level, and `raise exception`, abort the transaction: the
                // path commits nothing and owes nothing.
                if level.as_deref().is_none_or(|l| l == "exception") {
                    *st = None;
                }
            }
            PlStmt::Assert { cond, .. } => self.eval(cond, st, None),
            PlStmt::If {
                branches,
                otherwise,
                span,
            } => {
                let mut acc: State = None;
                let mut first = true;
                let mut fallthrough = st.clone();
                for (c, body) in branches {
                    self.eval(c, &mut fallthrough, None);
                    let mut b = fallthrough.clone();
                    self.stmts(body, &mut b);
                    acc = if first { b } else { self.join(acc, b, *span) };
                    first = false;
                }
                let mut o = fallthrough;
                if let Some(ob) = otherwise {
                    self.stmts(ob, &mut o);
                }
                *st = self.join(acc, o, *span);
            }
            PlStmt::Case {
                operand,
                whens,
                otherwise,
                span,
            } => {
                if let Some(e) = operand {
                    self.eval(e, st, None);
                }
                let entry = st.clone();
                let mut acc: State = None;
                let mut first = true;
                for (conds, body) in whens {
                    let mut b = entry.clone();
                    for c in conds {
                        self.eval(c, &mut b, None);
                    }
                    self.stmts(body, &mut b);
                    acc = if first { b } else { self.join(acc, b, *span) };
                    first = false;
                }
                // A `case` with no `else` raises CASE_NOT_FOUND when nothing matches: that
                // path aborts, so it joins as unreachable.
                let o = match otherwise {
                    Some(ob) => {
                        let mut o = entry.clone();
                        self.stmts(ob, &mut o);
                        o
                    }
                    None => None,
                };
                *st = if first { o } else { self.join(acc, o, *span) };
            }
            PlStmt::Loop {
                kind, body, span, ..
            } => {
                match kind.as_ref() {
                    LoopKind::While(c) => self.eval(c, st, None),
                    LoopKind::ForInt { lo, hi, by, .. } => {
                        self.eval(lo, st, None);
                        self.eval(hi, st, None);
                        if let Some(b) = by {
                            self.eval(b, st, None);
                        }
                    }
                    LoopKind::ForQuery { query, .. } => {
                        let mut v = Vec::new();
                        crate::check2::query_exprs(query, &mut v);
                        for e in v {
                            self.eval(e, st, None);
                        }
                    }
                    LoopKind::Foreach { array, .. } => self.eval(array, st, None),
                    _ => {}
                }
                let before = st.clone();
                self.depth += 1;
                let mut b = st.clone();
                self.stmts(body, &mut b);
                // Anything bound in this iteration and still live at its end is dropped.
                if let Some(vars) = b.as_ref() {
                    let d = self.depth;
                    let stale: Vec<(String, Span)> = vars
                        .iter()
                        .filter(|(_, v)| v.depth >= d)
                        .filter_map(|(n, v)| match v.st {
                            St::Live(s) => Some((n.clone(), s)),
                            _ => None,
                        })
                        .collect();
                    for (n, s) in stale {
                        self.out.push(Diag {
                            code: "NL0322",
                            error: true,
                            msg: format!(
                                "`{n}` is not consumed by the end of the loop's iteration"
                            ),
                            span: self.at.unwrap_or(s),
                            defect: Some("d14"),
                        });
                    }
                }
                self.depth -= 1;
                // After the loop: what was bound before it, with anything the body consumed
                // marked consumed (already reported by `eval` as NL0321).
                let mut after = before;
                if let (Some(a), Some(bv)) = (after.as_mut(), b.as_ref()) {
                    for (n, v) in a.iter_mut() {
                        if let Some(x) = bv.get(n) {
                            if matches!(x.st, St::Gone(_)) {
                                v.st = x.st.clone();
                            }
                        }
                    }
                }
                let _ = span;
                *st = after;
            }
            PlStmt::Exit { when, span, .. } => {
                if let Some(c) = when {
                    self.eval(c, st, None);
                }
                // Leaving an iteration early: anything bound in it and still live is lost.
                if let Some(vars) = st.as_ref() {
                    let d = self.depth;
                    let stale: Vec<String> = vars
                        .iter()
                        .filter(|(_, v)| d > 0 && v.depth >= d && matches!(v.st, St::Live(_)))
                        .map(|(n, _)| n.clone())
                        .collect();
                    for n in stale {
                        self.out.push(Diag {
                            code: "NL0322",
                            error: true,
                            msg: format!("`{n}` is not consumed on this exit path"),
                            span: self.at.unwrap_or(*span),
                            defect: Some("d14"),
                        });
                    }
                }
                if when.is_none() {
                    *st = None;
                }
            }
            PlStmt::Block(b) => self.block(b, st),
            PlStmt::Execute { command, using, .. } => {
                self.eval(command, st, None);
                for u in using {
                    self.eval(u, st, None);
                }
            }
            _ => {}
        }
    }
}

fn check_unit(conv: &Conventions, u: &Unit, out: &mut Vec<Diag>) {
    if conv.trusted(&u.name) {
        return;
    }
    let mut cx = Cx {
        conv,
        at: None,
        returns_kind: u
            .returns
            .as_deref()
            .and_then(|t| conv.linear_kind(t))
            .cloned(),
        depth: 0,
        out: Vec::new(),
        seen: BTreeSet::new(),
    };
    // Linear parameters arrive live: the function they were moved into must consume them.
    let mut vars = BTreeMap::new();
    for (n, t) in &u.params {
        if let Some(kind) = conv.linear_kind(t) {
            vars.insert(
                n.clone(),
                Var {
                    kind: kind.clone(),
                    st: St::Live(u.span),
                    depth: 0,
                },
            );
        }
    }
    let mut st: State = Some(vars);
    match &u.body {
        UnitBody::Pl(b) => {
            cx.block(b, &mut st);
        }
        UnitBody::Sql(stmts) => {
            cx.at = Some(u.span);
            let n = stmts.len();
            for (i, s) in stmts.iter().enumerate() {
                // The last statement of a `language sql` function is its result: a made value
                // there, in a function returning the linear domain, is moved out.
                let last = i + 1 == n && cx.returns_kind.is_some();
                for e in crate::conventions::stmt_exprs(s) {
                    let sunk = if last {
                        cx.made(e).map(|_| e as *const Expr)
                    } else {
                        None
                    };
                    if last {
                        if let Some(v) = Cx::var_of(e) {
                            if let Some(var) = st.as_mut().and_then(|m| m.get_mut(v)) {
                                var.st = St::Gone(u.span);
                            }
                        }
                    }
                    cx.eval(e, &mut st, sunk);
                }
            }
        }
    }
    // Falling off the end of the function.
    let end = st.clone();
    cx.leave(&end, u.span, "");
    out.extend(cx.out);
}

/// Every linearity diagnostic in the script. Nothing is reported when the script declares no
/// linear domain.
pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let conv = Conventions::of(stmts);
    let mut out = Vec::new();
    if conv.linear.is_empty() {
        return out;
    }
    for u in units(stmts) {
        check_unit(&conv, &u, &mut out);
    }
    out
}
