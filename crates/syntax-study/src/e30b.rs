//! **E30b′ — does the language need its own grammar?** (cycle 15, C15-05). The design,
//! pre-registered before any program or checker change, is `docs/study/E30b-design.md`; this
//! module is the part of the instrument E30's modules do not already provide: the second SQL
//! representation (R2), where each surface's programs live, the one mutation site pattern the
//! design adds, and the adversarial corpus.

use crate::data::Dataset;
use crate::mutate::Mutant;
use crate::tokens::{tokenize, Kind};
use std::path::{Path, PathBuf};

/// Representation R2's schema (design §3).
pub const SCHEMA_R2: &str = include_str!("../corpus/e30b/schema/r2.sql");
pub const SCHEMA_A01: &str = include_str!("../corpus/e30b/schema/a01.sql");
pub const SCHEMA_A02: &str = include_str!("../corpus/e30b/schema/a02.sql");
pub const SCHEMA_A02_NILES: &str = include_str!("../corpus/e30b/schema/a02.niles");

/// The surfaces E30b′ compares, in the design's order (§2).
pub const SURFACES: &[&str] = &["SQL", "SQL2", "NL", "RS"];

/// How a surface is named in the results.
pub fn label(surface: &str) -> &'static str {
    match surface {
        "SQL" => "SQL-R1",
        "SQL2" => "SQL-R2",
        "NL" => "NL",
        "RS" => "RS",
        _ => "?",
    }
}

/// The lexis a surface's programs are tokenised and mutated with: both SQL representations
/// are SQL.
pub fn lexis(surface: &str) -> &'static str {
    match surface {
        "SQL" | "SQL2" => "SQL",
        "NL" => "NL",
        _ => "RS",
    }
}

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

/// Where a surface's program for a task is (design §4.1): SQL-R1 is E30's program, or for
/// a transaction task the annotated copy in `e30b/r1/`; SQL-R2 is in `e30b/r2/`; NL and RS
/// are E30's, unchanged.
pub fn program_path(task: &str, surface: &str) -> PathBuf {
    let lower = task.to_lowercase();
    match surface {
        "SQL" if task.starts_with('T') => corpus().join(format!("e30b/r1/{task}/{lower}.sql")),
        "SQL" => corpus().join(format!("{task}/{lower}.sql")),
        "SQL2" => corpus().join(format!("e30b/r2/{task}/{lower}.sql")),
        s => corpus().join(crate::tasks::program_file(task, s)),
    }
}

/// A program the design itself declares not expressible, though a file exists (§4.1).
pub fn declared_not_expressible(task: &str, surface: &str) -> Option<&'static str> {
    match (task, surface) {
        ("T04", "NL" | "RS") => Some(
            "not expressible as the task states: a capture names no payee (`resolve h post amt` \
             posts nothing, Appendix B as written; the author's decision of 2026-09-30, design §6.2)",
        ),
        _ => None,
    }
}

/// The SQL that reads a transaction's effect back under R2.
pub fn sql_effect_r2(task: &str) -> &'static str {
    if task == "T09" {
        "select acct, cur, amount from overdraft_limits"
    } else {
        "select acct, cur, amt from postings where txn > 100"
    }
}

/// E30's mutants of a program, with R2's one extra M2 pattern for SQL-R2 (design §7): the
/// sign of the amount in a ledger insert's `amt` position is toggled.
pub fn mutants(src: &str, surface: &str) -> Vec<Mutant> {
    let mut v = crate::mutate::mutants(src, lexis(surface));
    if surface == "SQL2" {
        let next = v.iter().filter(|m| m.op == "M2").count();
        for (k, (start, end, with, line, what)) in m2_r2(src).into_iter().enumerate() {
            let mut text = String::with_capacity(src.len() + 4);
            text.push_str(&src[..start]);
            text.push_str(&with);
            text.push_str(&src[end..]);
            v.push(Mutant {
                op: "M2",
                site: next + k,
                line,
                what,
                text,
            });
        }
    }
    v
}

/// R2's M2 sites: in each `insert into postings (… amt …)`, the element at `amt`'s position
/// of every `values` tuple and of every top-level select item list.
// Token scanning moves between indices (a bracket's partner, a clause's end), as in `mutate.rs`.
#[allow(clippy::needless_range_loop)]
fn m2_r2(src: &str) -> Vec<(usize, usize, String, u32, String)> {
    let toks = tokenize(src, "SQL");
    let is = |i: usize, s: &str| toks.get(i).is_some_and(|t| t.text.eq_ignore_ascii_case(s));
    let closing = |open: usize| -> Option<usize> {
        let mut depth = 0i32;
        for (i, t) in toks.iter().enumerate().skip(open) {
            match t.text.as_str() {
                "(" | "[" => depth += 1,
                ")" | "]" => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 < toks.len() {
        if !(is(i, "insert") && is(i + 1, "into") && is(i + 2, "postings")) {
            i += 1;
            continue;
        }
        let end = (i..toks.len())
            .find(|&k| toks[k].text == ";")
            .unwrap_or(toks.len());
        let mut k = i + 3;
        let mut at = None;
        if is(k, "(") {
            let c = closing(k).unwrap_or(k);
            let cols: Vec<String> = toks[k + 1..c]
                .iter()
                .filter(|t| t.kind == Kind::Word)
                .map(|t| t.text.to_lowercase())
                .collect();
            at = cols.iter().position(|c| c == "amt");
            k = c + 1;
        }
        let Some(at) = at else {
            i = end;
            continue;
        };
        // Split a comma list at depth 0 between `from` and `to` (exclusive).
        let split = |from: usize, to: usize| -> Vec<(usize, usize)> {
            let mut elems = Vec::new();
            let mut depth = 0;
            let mut s = from;
            for x in from..to {
                match toks[x].text.as_str() {
                    "(" | "[" => depth += 1,
                    ")" | "]" => depth -= 1,
                    "," if depth == 0 => {
                        elems.push((s, x));
                        s = x + 1;
                    }
                    _ => {}
                }
            }
            elems.push((s, to));
            elems
        };
        let mut lists: Vec<(usize, usize)> = Vec::new();
        if is(k, "values") {
            let mut t = k + 1;
            while t < end && toks[t].text == "(" {
                let c = closing(t).unwrap_or(end);
                lists.push((t + 1, c));
                t = c + 1;
                if is(t, ",") {
                    t += 1;
                }
            }
        } else if is(k, "select") {
            // Each top-level select's item list: `select` to its `from` (or the statement's end).
            let mut t = k;
            while t < end {
                if is(t, "select") {
                    let mut depth = 0;
                    let mut stop = end;
                    for x in t + 1..end {
                        match toks[x].text.as_str() {
                            "(" => depth += 1,
                            ")" => depth -= 1,
                            _ if depth == 0
                                && (toks[x].text.eq_ignore_ascii_case("from")
                                    || toks[x].text.eq_ignore_ascii_case("union")) =>
                            {
                                stop = x;
                                break;
                            }
                            _ => {}
                        }
                    }
                    lists.push((t + 1, stop));
                    t = stop;
                }
                t += 1;
            }
        }
        for (from, to) in lists {
            let elems = split(from, to);
            let Some(&(a, b)) = elems.get(at) else {
                continue;
            };
            if a >= b {
                continue;
            }
            let (s, e) = (toks[a].start, toks[b - 1].end);
            let text = &src[s..e];
            if toks[a].text == "-" {
                out.push((
                    toks[a].start,
                    toks[a].end,
                    String::new(),
                    toks[a].line,
                    format!("the sign of `{text}` removed"),
                ));
            } else {
                out.push((
                    s,
                    e,
                    format!("-({text})"),
                    toks[a].line,
                    format!("`{text}` negated"),
                ));
            }
        }
        i = end;
    }
    out
}

/// Whether a mutant's one edit falls inside an effect annotation: SQL's `'effects: …'`
/// string, or a Niles function's declared row `! { … }` (design §7).
pub fn in_annotation(program: &str, mutant: &str, surface: &str) -> bool {
    let at = program
        .bytes()
        .zip(mutant.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(program.len().min(mutant.len()));
    let regions: Vec<(usize, usize)> = match lexis(surface) {
        "SQL" => {
            let mut v = Vec::new();
            let mut from = 0;
            while let Some(p) = program[from..].find("'effects:") {
                let s = from + p;
                let e = program[s + 1..]
                    .find('\'')
                    .map(|q| s + 1 + q)
                    .unwrap_or(program.len());
                v.push((s, e + 1));
                from = e + 1;
            }
            v
        }
        _ => {
            let mut v = Vec::new();
            let mut from = 0;
            while let Some(p) = program[from..].find("! {") {
                let s = from + p;
                let e = program[s..]
                    .find('}')
                    .map(|q| s + q)
                    .unwrap_or(program.len());
                v.push((s, e + 1));
                from = e + 1;
            }
            v
        }
    };
    regions.iter().any(|(s, e)| at >= *s && at < *e)
}

/// One adversarial case (design §4.2), as `corpus/e30b/adversarial/cases.tsv` gives it.
#[derive(Debug, Clone)]
pub struct Case {
    pub id: String,
    pub class: String,
    pub sql_schema: String,
    pub niles_schema: String,
    pub d_sql_args: String,
    pub c_sql_args: String,
    pub d_niles_args: String,
    pub c_niles_args: String,
    pub niles_defective: String,
    pub niles_correct: String,
}

pub fn adversarial_dir() -> PathBuf {
    corpus().join("e30b/adversarial")
}

pub fn cases() -> Vec<Case> {
    let text = std::fs::read_to_string(adversarial_dir().join("cases.tsv")).expect("cases.tsv");
    text.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let c: Vec<String> = l.split('\t').map(str::to_string).collect();
            assert_eq!(c.len(), 10, "cases.tsv row: {l}");
            Case {
                id: c[0].clone(),
                class: c[1].clone(),
                sql_schema: c[2].clone(),
                niles_schema: c[3].clone(),
                d_sql_args: c[4].clone(),
                c_sql_args: c[5].clone(),
                d_niles_args: c[6].clone(),
                c_niles_args: c[7].clone(),
                niles_defective: c[8].clone(),
                niles_correct: c[9].clone(),
            }
        })
        .collect()
}

/// The SQL schema a case is checked with, for R1 or R2.
pub fn sql_schema(case: &Case, r2: bool) -> &'static str {
    if r2 {
        return SCHEMA_R2;
    }
    match case.sql_schema.as_str() {
        "a01" => SCHEMA_A01,
        "a02" => SCHEMA_A02,
        _ => crate::pg::SCHEMA_SQL,
    }
}

pub fn niles_schema(case: &Case) -> &'static str {
    match case.niles_schema.as_str() {
        "a02" => SCHEMA_A02_NILES,
        _ => crate::niles::SCHEMA,
    }
}

/// `acct:3 usd:12345 int:1` → the Niles harness's arguments.
pub fn niles_args(spec: &str) -> Vec<crate::niles::Arg> {
    use crate::niles::Arg;
    spec.split_whitespace()
        .map(|w| {
            let (k, v) = w.split_once(':').expect("kind:value");
            let n: i64 = v.parse().expect("an integer");
            match k {
                "acct" => Arg::Acct(n),
                "int" => Arg::Int(n),
                "usd" => Arg::Money(n, "usd"),
                "eur" => Arg::Money(n, "eur"),
                "jpy" => Arg::Money(n, "jpy"),
                other => panic!("no argument kind {other}"),
            }
        })
        .collect()
}

/// Run a SQL correct twin on PostgreSQL: it must run and the deferred conservation check
/// must pass. `Ok(n)` is the number of postings after it ran.
pub fn run_sql_twin(
    c: &mut bank_bench::wire::Client,
    d: &Dataset,
    schema: &str,
    r2: bool,
    program: &str,
    call: &str,
) -> Result<String, String> {
    let data = crate::pg::data_sql(d, !r2);
    let steps = vec![
        ("exec", format!("select task({call})")),
        ("exec", "set constraints all immediate".to_string()),
        ("read", "select count(*) from postings".to_string()),
    ];
    match crate::pg::run(c, schema, &data, program, &steps) {
        Ok(reads) => Ok(reads
            .first()
            .and_then(|r| r.first())
            .and_then(|r| r.first())
            .cloned()
            .unwrap_or_default()),
        Err(crate::pg::SqlErr::Server {
            at,
            sqlstate,
            message,
        }) => Err(format!(
            "{at}: [{sqlstate}] {}",
            message.lines().next().unwrap_or("")
        )),
        Err(crate::pg::SqlErr::Blocked(e)) => Err(format!("blocked: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_m2_toggles_the_amount_sign_in_values_and_selects() {
        let src = "insert into postings (txn, acct, cur, amt, epoch, value_date) values\n    (t, a, 'usd', -(m).minor, e, current_date),\n    (t, b, 'usd', (m).minor, e, current_date);\ninsert into postings (txn, acct, cur, amt, epoch, value_date)\n    select t, acct, cur, -amt, e, current_date from postings where txn = r;";
        let whats: Vec<String> = m2_r2(src).into_iter().map(|x| x.4).collect();
        assert_eq!(
            whats,
            [
                "the sign of `-(m).minor` removed",
                "`(m).minor` negated",
                "the sign of `-amt` removed"
            ]
        );
    }

    #[test]
    fn an_edit_inside_an_annotation_is_recognised() {
        let p = "create function f() returns void as $$ $$;\ncomment on function f() is 'effects: debits usd';\n";
        let m = p.replace("debits usd", "debits eur");
        assert!(in_annotation(p, &m, "SQL"));
        let m2 = p.replace("void", "text");
        assert!(!in_annotation(p, &m2, "SQL"));
        let n = "fn f() -> R ! { debit<usd> } { post(debit(a, m)?) }";
        assert!(in_annotation(n, &n.replacen("usd", "eur", 1), "NL"));
        assert!(!in_annotation(n, &n.replace("debit(a", "credit(a"), "NL"));
    }

    #[test]
    fn every_case_has_both_sql_twins_and_names_its_niles_twins() {
        let cs = cases();
        assert_eq!(cs.len(), 14);
        for c in &cs {
            let dir = adversarial_dir().join(&c.id);
            assert!(
                dir.join("d.sql").exists() && dir.join("c.sql").exists(),
                "{}",
                c.id
            );
            for (f, spec) in [
                ("d.nl.niles", &c.niles_defective),
                ("c.nl.niles", &c.niles_correct),
            ] {
                assert_eq!(
                    dir.join(f).exists(),
                    !spec.starts_with("n/a"),
                    "{} {f}: file and cases.tsv disagree",
                    c.id
                );
            }
        }
    }
}
