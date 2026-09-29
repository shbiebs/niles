//! **Capabilities in SQL** (cycle 14, R2-05, the author's decision to attempt d9 rather than
//! assume it unreachable): the rule Niles states with NL0312 and NL0330, over annotated SQL.
//!
//! * **NL0312** — a function calls one marked `requires <effect>` without holding a
//!   parameter of a domain marked `capability <effect>`.
//! * **NL0330** — a capability is built rather than received: a cast to its domain, a
//!   variable of its domain assigned from anything but a capability it already holds, or a
//!   function returning the domain that is not marked `grants <effect>`.
//!
//! Together they are what "unforgeable" means in Niles, and no more: a capability arrives as
//! a parameter, or from a function the script marks as the one that grants it. What SQL
//! cannot add is the guarantee's reach outside the checked script — a console session can
//! still write `select authorize_overdraft(1, row(1)::usd)` — which is the same boundary the
//! Niles checker has at its own edge (it checks programs, not the people running them).

use crate::ast::*;
use crate::check::{walk_expr, Diag};
use crate::conventions::{stmt_exprs, units, Conventions, UnitBody};
use crate::lex::Span;
use crate::plpgsql::{Block, PlStmt};

fn diag(code: &'static str, msg: String, span: Span) -> Diag {
    Diag {
        code,
        error: true,
        msg,
        span,
        defect: Some("d9"),
    }
}

/// Every expression in a PL/pgSQL block, with the assignments into named variables.
fn block_exprs<'b>(
    b: &'b Block,
    exprs: &mut Vec<&'b Expr>,
    assigns: &mut Vec<(String, &'b Expr, Span)>,
) {
    for d in &b.decls {
        if let Some(e) = &d.default {
            exprs.push(e);
            assigns.push((d.name.clone(), e, d.span));
        }
    }
    for s in b
        .body
        .iter()
        .chain(b.handlers.iter().flat_map(|(_, h)| h.iter()))
    {
        pl_exprs(s, exprs, assigns);
    }
}

fn pl_exprs<'b>(
    s: &'b PlStmt,
    exprs: &mut Vec<&'b Expr>,
    assigns: &mut Vec<(String, &'b Expr, Span)>,
) {
    match s {
        PlStmt::Assign {
            target,
            value,
            span,
        } => {
            exprs.push(value);
            if let Expr::Col(n, _) = target {
                assigns.push((n.last().to_string(), value, *span));
            }
        }
        PlStmt::Sql { stmt, .. } => exprs.extend(stmt_exprs(stmt)),
        PlStmt::Perform(q, _) => crate::check2::query_exprs(q, exprs),
        PlStmt::Return(Some(e), _) | PlStmt::ReturnNext(Some(e), _) => exprs.push(e),
        PlStmt::ReturnQuery { query: Some(q), .. } => crate::check2::query_exprs(q, exprs),
        PlStmt::Raise { args, .. } => exprs.extend(args.iter()),
        PlStmt::Assert { cond, .. } => exprs.push(cond),
        PlStmt::If {
            branches,
            otherwise,
            ..
        } => {
            for (c, b) in branches {
                exprs.push(c);
                b.iter().for_each(|x| pl_exprs(x, exprs, assigns));
            }
            if let Some(o) = otherwise {
                o.iter().for_each(|x| pl_exprs(x, exprs, assigns));
            }
        }
        PlStmt::Case {
            operand,
            whens,
            otherwise,
            ..
        } => {
            exprs.extend(operand.iter());
            for (cs, b) in whens {
                exprs.extend(cs.iter());
                b.iter().for_each(|x| pl_exprs(x, exprs, assigns));
            }
            if let Some(o) = otherwise {
                o.iter().for_each(|x| pl_exprs(x, exprs, assigns));
            }
        }
        PlStmt::Loop { body, .. } => body.iter().for_each(|x| pl_exprs(x, exprs, assigns)),
        PlStmt::Block(b) => block_exprs(b, exprs, assigns),
        PlStmt::Execute { command, using, .. } => {
            exprs.push(command);
            exprs.extend(using.iter());
        }
        _ => {}
    }
}

pub fn check(stmts: &[Stmt]) -> Vec<Diag> {
    let conv = Conventions::of(stmts);
    let mut out = Vec::new();
    if conv.capability.is_empty() {
        return out;
    }
    let casts_to_capability = |e: &Expr, at: Span, out: &mut Vec<Diag>| {
        walk_expr(e, &mut |x| {
            if let Expr::Cast(_, ty, sp) = x {
                if let Some(eff) = conv.capability.get(ty.name.last()) {
                    out.push(diag(
                        "NL0330",
                        format!("a capability for `{eff}` cannot be constructed: `{}` is received as a parameter or granted, never cast", ty.name.last()),
                        if at.start == usize::MAX { *sp } else { at },
                    ));
                }
            }
        });
    };
    // Statements outside any function (a migration's top level, a view): a cast builds a
    // capability, and a call that requires one has nowhere to hold it.
    for s in stmts {
        if matches!(s, Stmt::CreateFunction(_) | Stmt::Do { .. }) {
            continue;
        }
        for e in stmt_exprs(s) {
            casts_to_capability(
                e,
                Span {
                    start: usize::MAX,
                    end: 0,
                },
                &mut out,
            );
            walk_expr(e, &mut |x| {
                if let Expr::Func(f) = x {
                    if let Some(eff) = conv.requires.get(f.name.last()) {
                        out.push(diag(
                            "NL0312",
                            format!("the script calls `{}`, which requires a capability for `{eff}`, outside any function that could hold one", f.name.last()),
                            f.span,
                        ));
                    }
                }
            });
        }
        if let Stmt::CreateView(v) = s {
            let mut v2 = Vec::new();
            crate::check2::query_exprs(&v.query, &mut v2);
            for e in v2 {
                casts_to_capability(
                    e,
                    Span {
                        start: usize::MAX,
                        end: 0,
                    },
                    &mut out,
                );
            }
        }
    }
    for u in units(stmts) {
        let granting = conv.grants.get(&u.name);
        let trusted = conv.trusted(&u.name);
        // What this function holds: its capability parameters, by effect.
        let held: Vec<(String, String)> = u
            .params
            .iter()
            .filter_map(|(n, t)| conv.capability.get(t).map(|e| (n.clone(), e.clone())))
            .collect();
        // A function returning a capability is a factory unless it is the granter.
        if let Some(r) = &u.returns {
            if let Some(eff) = conv.capability.get(r) {
                if granting != Some(eff) {
                    out.push(diag(
                        "NL0330",
                        format!("`{}` returns a capability for `{eff}` but is not marked `grants {eff}`: it would mint one for any caller", u.name),
                        u.span,
                    ));
                }
            }
        }
        let mut exprs: Vec<&Expr> = Vec::new();
        let mut assigns: Vec<(String, &Expr, Span)> = Vec::new();
        let mut cap_locals: Vec<(String, String)> = Vec::new();
        let sql_at = matches!(u.body, UnitBody::Sql(_)).then_some(u.span);
        match &u.body {
            UnitBody::Pl(b) => {
                fn decls(b: &Block, conv: &Conventions, out: &mut Vec<(String, String)>) {
                    for d in &b.decls {
                        if let Some(eff) =
                            d.ty.as_ref()
                                .and_then(|t| conv.capability.get(t.name.last()))
                        {
                            out.push((d.name.clone(), eff.clone()));
                        }
                    }
                    fn walk(s: &PlStmt, conv: &Conventions, out: &mut Vec<(String, String)>) {
                        match s {
                            PlStmt::Block(b) => decls(b, conv, out),
                            PlStmt::If {
                                branches,
                                otherwise,
                                ..
                            } => {
                                branches
                                    .iter()
                                    .flat_map(|(_, v)| v.iter())
                                    .for_each(|x| walk(x, conv, out));
                                otherwise.iter().flatten().for_each(|x| walk(x, conv, out));
                            }
                            PlStmt::Loop { body, .. } => {
                                body.iter().for_each(|x| walk(x, conv, out))
                            }
                            _ => {}
                        }
                    }
                    b.body.iter().for_each(|x| walk(x, conv, out));
                }
                decls(b, &conv, &mut cap_locals);
                block_exprs(b, &mut exprs, &mut assigns);
            }
            UnitBody::Sql(v) => {
                for s in v {
                    exprs.extend(stmt_exprs(s));
                }
            }
        }
        if granting.is_none() {
            for e in &exprs {
                casts_to_capability(
                    e,
                    sql_at.unwrap_or(Span {
                        start: usize::MAX,
                        end: 0,
                    }),
                    &mut out,
                );
            }
            // A capability-typed local assigned from anything but a capability already held.
            for (name, value, span) in &assigns {
                let Some((_, eff)) = cap_locals.iter().find(|(n, _)| n == name) else {
                    continue;
                };
                let from_held = matches!(value, Expr::Col(n, _) if held.iter().any(|(h, e)| h == n.last() && e == eff));
                let null = matches!(value, Expr::Lit(Literal::Null, _));
                if !from_held && !null {
                    out.push(diag(
                        "NL0330",
                        format!("`{name}` is a capability for `{eff}` and is assigned from something that is not one: a capability is received or granted, never built"),
                        sql_at.unwrap_or(*span),
                    ));
                }
            }
        }
        if trusted {
            continue;
        }
        // NL0312: a call to a function that requires a capability this one does not hold.
        for e in &exprs {
            walk_expr(e, &mut |x| {
                let Expr::Func(f) = x else { return };
                let Some(eff) = conv.requires.get(f.name.last()) else {
                    return;
                };
                if !held.iter().any(|(_, h)| h == eff) {
                    out.push(diag(
                        "NL0312",
                        format!(
                            "`{}` calls `{}`, which requires a capability for `{eff}`, without holding one",
                            u.name,
                            f.name.last()
                        ),
                        sql_at.unwrap_or(f.span),
                    ));
                }
            });
        }
    }
    out.sort_by_key(|d| (d.span.start, d.code));
    out.dedup_by(|a, b| a.span.start == b.span.start && a.code == b.code);
    out
}
