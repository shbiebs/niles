//! **The NL and RS executors** (design §2 and amendment A1): the Niles front end as `nilesc check`
//! runs it, a view evaluated by `niles_ir::eval` over the dataset with the as-of and valid-time
//! pins applied to the sources the way A1 fixes, and a function run by `niles-interp`.

use crate::answer::Answer;
use crate::data::{Dataset, Posting};
use crate::kinds::{render, Kind};
use niles_ir::eval::{self, ZSet};
use niles_ir::operator::Op;
use niles_ir::value::Value;
use niles_lang::diagnostics::Severity;
use niles_lang::{lower, parser, resolve, typecheck};
use std::collections::BTreeMap;

/// The schema every NL and RS program is checked after.
pub const SCHEMA: &str = include_str!("../corpus/schema/schema.niles");

/// A checker verdict: every diagnostic's code, and whether any is an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Verdict {
    pub fn refused(&self) -> bool {
        !self.errors.is_empty()
    }
}

fn front(
    src: &str,
) -> (
    niles_lang::ast::Program,
    resolve::Catalog,
    niles_lang::diagnostics::Diagnostics,
) {
    let (prog, mut d) = parser::parse_program(src);
    let (cat, r) = resolve::resolve_program(&prog, 0);
    d.extend(r);
    let (_, t) = typecheck::check_program(&prog, &cat);
    d.extend(t);
    (prog, cat, d)
}

/// `nilesc check`: parse, resolve, typecheck, lower.
pub fn check(program: &str) -> Verdict {
    let src = format!("{SCHEMA}\n{program}");
    let (prog, cat, mut d) = front(&src);
    let (_, l) = lower::lower_program(&prog, &cat);
    d.extend(l);
    let mut v = Verdict {
        errors: Vec::new(),
        warnings: Vec::new(),
    };
    for x in &d.items {
        match x.severity {
            Severity::Error => v.errors.push(x.code.to_string()),
            Severity::Warning => v.warnings.push(x.code.to_string()),
            Severity::Note => {}
        }
    }
    v.errors.sort();
    v.errors.dedup();
    v.warnings.sort();
    v.warnings.dedup();
    v
}

/// Days since 1970-01-01 of the dataset's day 0 (2026-01-01), the encoding `Date` lowers to.
pub fn day0() -> i64 {
    niles_ir::value::days_since_epoch("2026-01-01").expect("a date")
}

/// The pins a circuit carries, as the server reads them (`session.rs`, `pinned_anchor`):
/// conflicting as-of pins are refused, not reconciled.
fn pins(c: &niles_ir::circuit::Circuit) -> Result<(Option<u64>, Option<i64>), String> {
    let (mut asof, mut valid) = (None, None);
    for n in &c.nodes {
        match &n.op {
            Op::AsOf { epoch: Some(e) } => match asof {
                None => asof = Some(*e),
                Some(p) if p == *e => {}
                Some(p) => return Err(format!("conflicting as-of pins #{p} and #{e}")),
            },
            Op::ValidAt { instant: Some(i) } => match valid {
                None => valid = Some(*i),
                Some(p) if p == *i => {}
                Some(p) => return Err(format!("conflicting valid-time pins {p} and {i}")),
            },
            _ => {}
        }
    }
    Ok((asof, valid))
}

/// The dataset as sources, in each relation's declared column order.
fn sources(
    cat: &resolve::Catalog,
    d: &Dataset,
    extra: &[Posting],
    asof: Option<u64>,
    valid: Option<i64>,
) -> Result<BTreeMap<String, ZSet>, String> {
    let cur = |c: &str| -> Value {
        Value::Int(cat.currencies.get(c).map(|x| x.code as i128).unwrap_or(-1))
    };
    let mut out = BTreeMap::new();
    for (name, rel) in &cat.relations {
        let cols: Vec<&str> = rel.columns.iter().map(|c| c.name.as_str()).collect();
        let mut z: ZSet = ZSet::new();
        let mut add = |row: Vec<Value>| {
            *z.entry(row).or_insert(0) += 1;
        };
        let pick = |get: &dyn Fn(&str) -> Option<Value>| -> Result<Vec<Value>, String> {
            cols.iter()
                .map(|c| get(c).ok_or_else(|| format!("no value for {name}.{c}")))
                .collect()
        };
        match name.as_str() {
            "parties" => {
                for p in &d.parties {
                    add(pick(&|c| match c {
                        "id" => Some(Value::Int(p.id as i128)),
                        "parent" => Some(
                            p.parent
                                .map(|x| Value::Int(x as i128))
                                .unwrap_or(Value::Null),
                        ),
                        _ => None,
                    })?);
                }
            }
            "accounts" => {
                for a in &d.accounts {
                    add(pick(&|c| match c {
                        "id" => Some(Value::Int(a.id as i128)),
                        "owner" => Some(Value::Int(a.owner as i128)),
                        "desk" => Some(Value::Int(a.desk as i128)),
                        _ => None,
                    })?);
                }
            }
            "postings" => {
                for p in d.postings.iter().chain(extra) {
                    if asof.is_some_and(|e| p.epoch as u64 > e) {
                        continue;
                    }
                    if valid.is_some_and(|v| day0() + p.value_date > v) {
                        continue;
                    }
                    add(pick(&|c| match c {
                        "txn" | "idem" => Some(Value::Int(p.txn as i128)),
                        "acct" => Some(Value::Int(p.acct as i128)),
                        "cur" => Some(cur(p.cur)),
                        "amt" => Some(Value::Int(p.amt as i128)),
                        "value_date" => Some(Value::Int((day0() + p.value_date) as i128)),
                        _ => None,
                    })?);
                }
            }
            "holds" => {
                for h in &d.holds {
                    add(pick(&|c| match c {
                        "id" => Some(Value::Int(h.id as i128)),
                        "acct" => Some(Value::Int(h.acct as i128)),
                        "cur" => Some(cur(h.cur)),
                        "amount" => Some(Value::Int(h.amount as i128)),
                        "open" => Some(Value::Int(h.open as i128)),
                        _ => None,
                    })?);
                }
            }
            other => return Err(format!("the dataset has no relation `{other}`")),
        }
        out.insert(name.clone(), z);
    }
    Ok(out)
}

/// Evaluate the views named `outputs` of a program, each over the dataset plus `extra`
/// postings, and concatenate their rows (each prefixed by `tags[i]` when given).
pub fn eval_views(
    program: &str,
    outputs: &[(&str, Option<&str>)],
    kinds: &[Kind],
    d: &Dataset,
    extra: &[Posting],
    like: &Answer,
) -> Result<Answer, String> {
    let src = format!("{SCHEMA}\n{program}");
    let (prog, cat, mut diags) = front(&src);
    let (lowered, l) = lower::lower_program(&prog, &cat);
    diags.extend(l);
    if diags.has_errors() {
        return Err("the program does not check".into());
    }
    let code_to_cur: BTreeMap<i128, String> = cat
        .currencies
        .values()
        .map(|c| (c.code as i128, c.name.clone()))
        .collect();
    let mut rows = Vec::new();
    for (view, tag) in outputs {
        // Pins are per view: evaluate each view's own sub-circuit with its own pins.
        let (a, v) = pins_of_output(&lowered.circuit, view)?;
        let src = sources(&cat, d, extra, a, v)?;
        let (z, _) = eval::try_run(&lowered.circuit, view, &src).map_err(|e| e.to_string())?;
        let tag_kinds: &[Kind] = if tag.is_some() { &kinds[1..] } else { kinds };
        // A Z-set has no order; the order a view presents is its `OrderBy`'s, applied here
        // exactly as the server applies it (`eval::presentation_order`).
        let keys = eval::presentation_order(&lowered.circuit, view);
        let mut ordered: Vec<(&Vec<Value>, &i128)> = z.iter().collect();
        ordered.sort_by(|a, b| eval::presented_cmp(a.0, b.0, &keys));
        for (r, w) in ordered {
            if *w < 0 {
                return Err(format!("negative weight {w} in `{view}`"));
            }
            for _ in 0..*w {
                let mut out: Vec<String> = tag.iter().map(|t| t.to_string()).collect();
                if r.len() != tag_kinds.len() {
                    return Err(format!(
                        "`{view}` returns {} columns; the task has {}",
                        r.len(),
                        tag_kinds.len()
                    ));
                }
                for (v, k) in r.iter().zip(tag_kinds) {
                    out.push(render(*v, *k, &code_to_cur, day0()));
                }
                rows.push(out);
            }
        }
    }
    Ok(like.like(rows))
}

/// The pins on the path to one output, found by walking its inputs.
fn pins_of_output(
    c: &niles_ir::circuit::Circuit,
    output: &str,
) -> Result<(Option<u64>, Option<i64>), String> {
    let Some(root) = c.outputs.get(output) else {
        return Err(format!("no view `{output}`"));
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut stack = vec![*root];
    let mut sub = niles_ir::circuit::Circuit::default();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(n) = c.nodes.iter().find(|n| n.id == id) {
            sub.nodes.push(n.clone());
            stack.extend(n.inputs.iter().copied());
        }
    }
    pins(&sub)
}

/// A value the interpreter is called with.
pub enum Arg {
    Acct(i64),
    Money(i64, &'static str),
    Int(i64),
}

/// Why a function did not produce a posting set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunErr {
    /// The interpreter refuses a form it has no semantics for (A1): not a verdict on the
    /// program.
    NotInSubset(String),
    /// It ran and failed: a refused commit, a failed `?`, anything the program itself did.
    Failed(String),
}

/// Run function `f` of a program on the interpreter and return the legs it seals, as
/// (acct, cur, amt). Opening balances are the dataset's.
pub fn run_fn(program: &str, f: &str, args: &[Arg], d: &Dataset) -> Result<Answer, RunErr> {
    use niles_interp::{Interp, Value as V};
    let src = format!("{SCHEMA}\n{program}");
    let (prog, cat, diags) = front(&src);
    if diags.has_errors() {
        return Err(RunErr::Failed("the program does not check".into()));
    }
    let scale = |c: &str| {
        crate::data::CURRENCIES
            .iter()
            .find(|x| x.0 == c)
            .map(|x| x.1)
            .unwrap_or(2)
    };
    // **The call boundary.** A caller hands `task` its arguments through a typed boundary —
    // the server decodes a call's arguments by the declared parameter types, as PostgreSQL
    // resolves `select task(…)` against the function's signature — and the interpreter's
    // `call` does not check them. So a parameter whose declared currency differs from the
    // argument's is refused here, as the call PostgreSQL refuses ("function … does not
    // exist") is refused there: a run-time failure on both surfaces.
    if let Some(decl) = find_fn(&prog.items, f) {
        for (n, (p, a)) in decl.params.iter().zip(args).enumerate() {
            if let (Some(declared), Arg::Money(_, c)) = (money_currency(&p.ty), a) {
                if !declared.eq_ignore_ascii_case(c) {
                    return Err(RunErr::Failed(format!(
                        "argument {} is Money<{c}> and `{f}` declares Money<{declared}>: no typed caller can make this call",
                        n + 1
                    )));
                }
            }
        }
    }
    let mut it = Interp::new();
    it.load(&prog);
    let mut open: BTreeMap<(i64, &str), i64> = BTreeMap::new();
    for p in &d.postings {
        *open.entry((p.acct, p.cur)).or_insert(0) += p.amt;
    }
    for ((a, c), m) in open {
        it.ledger
            .open_balance(&a.to_string(), &c.to_uppercase(), m as i128);
    }
    let vs: Vec<V> = args
        .iter()
        .map(|a| match a {
            Arg::Acct(i) => V::Str(std::rc::Rc::new(i.to_string())),
            Arg::Int(i) => V::Int(*i as i128),
            Arg::Money(m, c) => V::Money {
                minor: *m as i128,
                scale: scale(c),
                currency: c.to_uppercase(),
            },
        })
        .collect();
    match it.call(f, vs) {
        Ok(_) => {}
        Err(niles_interp::Error::NotInSubset { form, .. }) => {
            return Err(RunErr::NotInSubset(form.to_string()))
        }
        Err(e) => {
            // The interpreter has no relational tier: a view or relation read inside a
            // function reaches it as an unbound name. That is the executor's limit, not the
            // program's error, so it is kept apart from a failure (design deviation D1).
            let m = e.message();
            let unbound = m
                .strip_prefix('`')
                .and_then(|r| r.split_once("` is not bound here"))
                .map(|(n, _)| n.to_string());
            if let Some(n) = unbound {
                if cat.relations.contains_key(&n) || cat.views.contains_key(&n) {
                    return Err(RunErr::NotInSubset(format!(
                        "a read of `{n}` (the interpreter has no relational tier)"
                    )));
                }
            }
            return Err(RunErr::Failed(m));
        }
    }
    let mut rows = Vec::new();
    for set in &it.ledger.sealed {
        for l in &set.legs {
            rows.push(vec![
                l.account.clone(),
                l.currency.to_lowercase(),
                l.minor.to_string(),
            ]);
        }
    }
    Ok(Answer::set(rows))
}

fn find_fn<'a>(items: &'a [niles_lang::ast::Item], f: &str) -> Option<&'a niles_lang::ast::FnDecl> {
    use niles_lang::ast::Item;
    items.iter().find_map(|i| match i {
        Item::Fn(d) if d.name.text == f => Some(d),
        Item::Mod { items, .. } => find_fn(items, f),
        _ => None,
    })
}

/// `Money<c>` → `c`.
fn money_currency(t: &niles_lang::ast::Ty) -> Option<String> {
    match t {
        niles_lang::ast::Ty::Path { path, args, .. } if path.last().text == "Money" => {
            match args.first() {
                Some(niles_lang::ast::Ty::Path { path, .. }) => Some(path.last().text.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

/// The rows the Niles executor is handed, rendered canonically (design §4's load check):
/// the relations exactly as [`sources`] builds them for every evaluation, decoded back
/// through the catalog. A posting's epoch is system time here, not a column (A1), so the
/// postings carry no epoch.
pub fn loaded_rows(d: &Dataset) -> BTreeMap<String, Vec<String>> {
    let (_, cat, _) = front(SCHEMA);
    let src = sources(&cat, d, &[], None, None).expect("sources");
    let name_of: BTreeMap<i128, String> = cat
        .currencies
        .iter()
        .map(|(n, c)| (c.code as i128, n.clone()))
        .collect();
    let mut out = BTreeMap::new();
    for (rel, z) in src {
        let cols: Vec<String> = cat.relations[&rel]
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect();
        let want: &[&str] = match rel.as_str() {
            "parties" => &["id", "parent"],
            "accounts" => &["id", "owner", "desk"],
            "postings" => &["txn", "acct", "cur", "amt", "value_date"],
            "holds" => &["id", "acct", "cur", "amount", "open"],
            _ => continue,
        };
        let mut rows = Vec::new();
        for (row, w) in &z {
            let cell = |c: &str| -> String {
                let i = cols.iter().position(|x| x == c).expect("column");
                match (c, &row[i]) {
                    (_, Value::Null) => "null".into(),
                    ("cur", Value::Int(k)) => name_of.get(k).cloned().unwrap_or_default(),
                    ("value_date", Value::Int(k)) => crate::kinds::iso(*k as i64),
                    (_, Value::Int(k)) => k.to_string(),
                }
            };
            let line = want.iter().map(|c| cell(c)).collect::<Vec<_>>().join("|");
            for _ in 0..*w {
                rows.push(line.clone());
            }
        }
        out.insert(rel, rows);
    }
    out
}
