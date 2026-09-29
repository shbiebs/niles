//! **The annotations SQL+C+L reads** (cycle 14, R2-05), all of them stock PostgreSQL comments,
//! so PostgreSQL accepts every annotated script unchanged:
//!
//! | comment | on | means |
//! |---|---|---|
//! | `linear <kind>` | a domain | every value of the domain is linear: consumed exactly once |
//! | `produces <kind>` | a function | it makes a fresh linear value (its body is trusted) |
//! | `consumes <kind>` | a function | it consumes the linear argument of that kind (trusted) |
//! | `capability <effect>` | a domain | a value of the domain is an unforgeable capability |
//! | `requires <effect>` | a function | a caller must hold a capability for `effect` |
//! | `grants <effect>` | a function | it may build a capability for `effect` (trusted) |
//!
//! Functions are keyed by name: an overloaded name carries one annotation, which the rules
//! apply to every overload. That is a stated limit of this reading, not an oversight.

use crate::ast::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone)]
pub struct Conventions {
    /// linear domain → kind.
    pub linear: BTreeMap<String, String>,
    /// function → the kind it produces.
    pub produces: BTreeMap<String, String>,
    /// function → the kind it consumes.
    pub consumes: BTreeMap<String, String>,
    /// capability domain → effect.
    pub capability: BTreeMap<String, String>,
    /// function → the effect it requires a capability for.
    pub requires: BTreeMap<String, String>,
    /// function → the effect it may grant.
    pub grants: BTreeMap<String, String>,
    /// function → its declared parameter types (last name part), in order.
    pub params: BTreeMap<String, Vec<String>>,
    /// function → its return type (last name part), when it has a single one.
    pub returns: BTreeMap<String, String>,
    /// Every domain the script declares.
    pub domains: BTreeSet<String>,
}

fn word_after<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let mut w = text.split_whitespace();
    if w.next()? == key {
        w.next()
    } else {
        None
    }
}

impl Conventions {
    pub fn of(stmts: &[Stmt]) -> Conventions {
        let mut c = Conventions::default();
        for s in stmts {
            match s {
                Stmt::CreateDomain { name, .. } => {
                    c.domains.insert(name.last().to_string());
                }
                Stmt::CreateFunction(f) => {
                    let n = f.name.last().to_string();
                    c.params.insert(
                        n.clone(),
                        f.args
                            .iter()
                            .filter(|a| a.mode.as_deref() != Some("out"))
                            .map(|a| a.ty.name.last().to_string())
                            .collect(),
                    );
                    if let Some(Returns::Type { setof: false, ty }) = &f.returns {
                        c.returns.insert(n, ty.name.last().to_string());
                    }
                }
                _ => {}
            }
        }
        for s in stmts {
            if let Stmt::Comment {
                kind,
                target,
                text: Some(text),
                ..
            } = s
            {
                let t = target.last().to_string();
                match kind.as_str() {
                    "domain" | "type" => {
                        if let Some(k) = word_after(text, "linear") {
                            c.linear.insert(t.clone(), k.to_string());
                        }
                        if let Some(e) = word_after(text, "capability") {
                            c.capability.insert(t, e.to_string());
                        }
                    }
                    "function" | "procedure" => {
                        for (key, map) in [
                            ("produces", &mut c.produces),
                            ("consumes", &mut c.consumes),
                            ("requires", &mut c.requires),
                            ("grants", &mut c.grants),
                        ] {
                            if let Some(k) = word_after(text, key) {
                                map.insert(t.clone(), k.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        c
    }

    /// The linear kind a value of type `ty` carries, if any.
    pub fn linear_kind(&self, ty: &str) -> Option<&String> {
        self.linear.get(ty)
    }

    /// The linear kind a call to `f` produces: a function marked `produces`, or one whose
    /// declared return type is a linear domain.
    pub fn produced_by(&self, f: &str) -> Option<String> {
        self.produces.get(f).cloned().or_else(|| {
            self.returns
                .get(f)
                .and_then(|t| self.linear.get(t))
                .cloned()
        })
    }

    /// Whether a function's body is a trusted primitive of a convention (a producer, a
    /// consumer, a granter, or a function a capability is required for).
    pub fn trusted(&self, f: &str) -> bool {
        self.produces.contains_key(f)
            || self.consumes.contains_key(f)
            || self.grants.contains_key(f)
            || self.requires.contains_key(f)
    }

    pub fn is_empty(&self) -> bool {
        self.linear.is_empty() && self.capability.is_empty()
    }
}

/// A body the R2-05 rules analyse: a function (PL/pgSQL or `language sql`) or a `do` block.
pub struct Unit<'a> {
    /// The function's name; `do` for an anonymous block.
    pub name: String,
    /// (parameter name, type), in order; unnamed parameters are `$n`.
    pub params: Vec<(String, String)>,
    pub returns: Option<String>,
    pub body: UnitBody<'a>,
    pub span: Span,
}

pub enum UnitBody<'a> {
    Pl(&'a crate::plpgsql::Block),
    /// A `language sql` body, re-parsed; diagnostics inside it are reported at `span`
    /// because its statements' spans are relative to the body text.
    Sql(Vec<Stmt>),
}

use crate::lex::Span;

pub fn units(stmts: &[Stmt]) -> Vec<Unit<'_>> {
    let mut out = Vec::new();
    for s in stmts {
        match s {
            Stmt::CreateFunction(f) => {
                let params = f
                    .args
                    .iter()
                    .filter(|a| a.mode.as_deref() != Some("out"))
                    .enumerate()
                    .map(|(i, a)| {
                        (
                            a.name.clone().unwrap_or_else(|| format!("${}", i + 1)),
                            a.ty.name.last().to_string(),
                        )
                    })
                    .collect();
                let returns = match &f.returns {
                    Some(Returns::Type { setof: false, ty }) => Some(ty.name.last().to_string()),
                    _ => None,
                };
                let body = if let Some(b) = &f.plpgsql {
                    Some(UnitBody::Pl(b))
                } else {
                    match &f.body {
                        Some(FuncBody::Text(text, _)) if f.language.as_deref() == Some("sql") => {
                            crate::parse(text).ok().map(|(v, _)| UnitBody::Sql(v))
                        }
                        Some(FuncBody::Atomic(v)) => Some(UnitBody::Sql(v.clone())),
                        _ => None,
                    }
                };
                if let Some(body) = body {
                    out.push(Unit {
                        name: f.name.last().to_string(),
                        params,
                        returns,
                        body,
                        span: f.span,
                    });
                }
            }
            Stmt::Do { block, span } => out.push(Unit {
                name: "do".into(),
                params: Vec::new(),
                returns: None,
                body: UnitBody::Pl(block),
                span: *span,
            }),
            _ => {}
        }
    }
    out
}

/// The top-level expressions of one SQL statement: select items, predicates, values rows,
/// `set` values, `returning` items — everything a rule over calls needs to see.
pub fn stmt_exprs(s: &Stmt) -> Vec<&Expr> {
    let mut out = Vec::new();
    match s {
        Stmt::Query(q) => crate::check2::query_exprs(q, &mut out),
        Stmt::Insert(i) => {
            if let Some(q) = &i.source {
                crate::check2::query_exprs(q, &mut out);
            }
            out.extend(i.returning.iter().map(|r| &r.expr));
        }
        Stmt::Update(u) => {
            out.extend(u.set.iter().map(|(_, e)| e));
            out.extend(u.where_.iter());
        }
        Stmt::Delete(d) => out.extend(d.where_.iter()),
        Stmt::Call(e, _) => out.push(e),
        _ => {}
    }
    out
}
