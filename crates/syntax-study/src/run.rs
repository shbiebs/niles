//! **One program, checked and executed**: the verdict the design's classification reads (§6):
//! refused by its checker, raised when run, not executable by any executor (A1), or an answer.

use crate::answer::Answer;
use crate::data::{Dataset, Posting};
use crate::ext;
use crate::kinds::{self, Kind};
use crate::niles;
use crate::pg;
use crate::tasks::{self, Arg, Class};
use bank_bench::wire::Client;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The surface's checker refused it: the codes (or the checker's first error line).
    Static(Vec<String>),
    /// Accepted, and it raised when run.
    Runtime(String),
    /// Accepted, and no executor for this surface can run it (A1).
    Unexecuted(String),
    /// Accepted and ran.
    Answer(Answer),
    /// The harness could not run at all: never a verdict.
    Blocked(String),
}

pub struct Ctx {
    pub data: Dataset,
    pub pg: Option<Client>,
    pub data_typed: String,
    pub data_plain: String,
    pub facts: PathBuf,
    pub work: PathBuf,
    pub prqlc: Option<PathBuf>,
    pub souffle: Option<PathBuf>,
}

impl Ctx {
    pub fn new(data: Dataset, work: PathBuf) -> Ctx {
        let facts = work.join("facts");
        let _ = ext::write_facts(&data, &facts);
        Ctx {
            data_typed: pg::data_sql(&data, true),
            data_plain: pg::data_sql(&data, false),
            pg: pg::connect().ok(),
            data,
            facts,
            work,
            prqlc: ext::prqlc(),
            souffle: ext::souffle(),
        }
    }
}

fn sql_rows(a: &Answer, rows: Vec<Vec<String>>, kinds: &[Kind]) -> Answer {
    // PostgreSQL renders dates as ISO text and currencies as the text they are: nothing to do
    // but keep only what the task's kinds say is there.
    let _ = kinds;
    a.like(rows)
}

fn dl_rows(a: &Answer, rows: Vec<Vec<String>>, kinds: &[Kind]) -> Answer {
    let rows = rows
        .into_iter()
        .map(|r| {
            r.into_iter()
                .zip(kinds.iter().chain(std::iter::repeat(&Kind::Text)))
                .map(|(v, k)| match k {
                    Kind::Date => v.parse::<i64>().map(kinds::iso).unwrap_or(v),
                    _ => v,
                })
                .collect()
        })
        .collect();
    a.like(rows)
}

fn niles_args(v: &[Arg]) -> Option<Vec<niles::Arg>> {
    v.iter()
        .map(|a| match a {
            Arg::Acct(i) => Some(niles::Arg::Acct(*i)),
            Arg::Int(i) => Some(niles::Arg::Int(*i)),
            Arg::Money(m, c) => Some(niles::Arg::Money(*m, c)),
            // The interpreter has no way to be handed an `Auth`: it refuses `authorize` anyway.
            Arg::Cap => None,
        })
        .collect()
}

/// Check a program with its surface's checker: `Some(codes)` when refused.
pub fn check(ctx: &Ctx, surface: &str, program: &str) -> Result<Option<Vec<String>>, String> {
    match surface {
        "SQL" => Ok(check_sql(pg::SCHEMA_SQL, program)),
        // E30b′'s representation R2 (cycle 15, design §3): the same checker, R2's schema.
        "SQL2" => Ok(check_sql(crate::e30b::SCHEMA_R2, program)),
        "NL" | "RS" => {
            let v = niles::check(program);
            Ok(v.refused().then_some(v.errors))
        }
        "PRQL" => {
            let bin = ctx.prqlc.as_ref().ok_or("prqlc is not installed")?;
            match ext::prql_compile(bin, program) {
                Ok(_) => Ok(None),
                Err(e) => Ok(Some(vec![first_line(&e)])),
            }
        }
        "DL" => {
            let bin = ctx.souffle.as_ref().ok_or("souffle is not installed")?;
            // The front end runs before evaluation; an empty fact directory makes the check
            // independent of the data.
            let empty = ctx.work.join("nofacts");
            let _ = std::fs::create_dir_all(&empty);
            match ext::souffle_run(bin, &empty, &ctx.work.join("check"), program) {
                Err(ext::DlErr::Refused(e)) => Ok(Some(vec![first_line(&e)])),
                _ => Ok(None),
            }
        }
        other => Err(format!("no surface {other}")),
    }
}

/// SQL+C+L over `schema` followed by the program: `Some(codes)` when it refuses.
pub fn check_sql(schema: &str, program: &str) -> Option<Vec<String>> {
    let src = format!("{schema}\n{program}");
    match nilescheck_sql::parse(&src) {
        Err(e) => Some(vec![format!("parse: {}", e.msg)]),
        Ok((stmts, _)) => {
            let mut e: Vec<String> = nilescheck_sql::check_all(&stmts)
                .into_iter()
                .filter(|d| d.error)
                .map(|d| d.code.to_string())
                .collect();
            e.sort();
            e.dedup();
            (!e.is_empty()).then_some(e)
        }
    }
}

fn first_line(s: &str) -> String {
    s.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .chars()
        .take(160)
        .collect()
}

/// Execute an accepted program and read its answer.
pub fn execute(ctx: &mut Ctx, task: &str, surface: &str, program: &str, like: &Answer) -> Outcome {
    let kinds = kinds::of(task);
    match surface {
        "SQL" | "SQL2" | "PRQL" => {
            let (schema, data, text) = if surface == "PRQL" {
                let Some(bin) = ctx.prqlc.clone() else {
                    return Outcome::Blocked("prqlc is not installed".into());
                };
                match ext::prql_compile(&bin, program) {
                    Ok(sql) => (pg::SCHEMA_PLAIN, ctx.data_plain.clone(), sql),
                    Err(e) => return Outcome::Static(vec![first_line(&e)]),
                }
            } else if surface == "SQL2" {
                (
                    crate::e30b::SCHEMA_R2,
                    ctx.data_plain.clone(),
                    program.to_string(),
                )
            } else {
                (pg::SCHEMA_SQL, ctx.data_typed.clone(), program.to_string())
            };
            let Some(c) = ctx.pg.as_mut() else {
                return Outcome::Blocked("no PostgreSQL".into());
            };
            let (definition, steps): (String, Vec<(&'static str, String)>) =
                match tasks::class(task) {
                    Class::Q => (String::new(), vec![("read", text)]),
                    Class::B if task != "B03" => (String::new(), vec![("read", text)]),
                    Class::T => {
                        let a: Vec<String> =
                            tasks::args(task, &ctx.data).iter().map(Arg::sql).collect();
                        (
                            text,
                            vec![
                                ("exec", format!("select task({})", a.join(", "))),
                                ("exec", "set constraints all immediate".into()),
                                (
                                    "read",
                                    if surface == "SQL2" {
                                        crate::e30b::sql_effect_r2(task).into()
                                    } else {
                                        tasks::sql_effect(task).into()
                                    },
                                ),
                            ],
                        )
                    }
                    Class::V => (text, vec![("read", "select * from answer".into())]),
                    Class::B => {
                        let a: Vec<String> =
                            tasks::correction_args().iter().map(Arg::sql).collect();
                        (
                            text,
                            vec![
                                ("exec", format!("select correct({})", a.join(", "))),
                                ("exec", "set constraints all immediate".into()),
                                ("read", "select 'valid', * from answer_valid".into()),
                                ("read", "select 'system', * from answer_system".into()),
                            ],
                        )
                    }
                };
            match pg::run(c, schema, &data, &definition, &steps) {
                Ok(reads) => {
                    Outcome::Answer(sql_rows(like, reads.into_iter().flatten().collect(), kinds))
                }
                Err(pg::SqlErr::Server {
                    at,
                    sqlstate,
                    message,
                }) => Outcome::Runtime(format!("{at}: [{sqlstate}] {}", first_line(&message))),
                Err(pg::SqlErr::Blocked(e)) => Outcome::Blocked(e),
            }
        }
        "DL" => {
            let Some(bin) = ctx.souffle.clone() else {
                return Outcome::Blocked("souffle is not installed".into());
            };
            match ext::souffle_run(&bin, &ctx.facts, &ctx.work.join("run"), program) {
                Ok(rows) => Outcome::Answer(dl_rows(like, rows, kinds)),
                Err(ext::DlErr::Refused(e)) => Outcome::Static(vec![first_line(&e)]),
                Err(ext::DlErr::Failed(e)) => Outcome::Runtime(first_line(&e)),
            }
        }
        "NL" | "RS" => match tasks::class(task) {
            Class::T => {
                let Some(args) = niles_args(&tasks::args(task, &ctx.data)) else {
                    return Outcome::Unexecuted(
                        "the interpreter cannot be handed a capability".into(),
                    );
                };
                match niles::run_fn(program, "task", &args, &ctx.data) {
                    Ok(a) => Outcome::Answer(a),
                    Err(niles::RunErr::NotInSubset(f)) => {
                        Outcome::Unexecuted(format!("niles-interp refuses `{f}`"))
                    }
                    Err(niles::RunErr::Failed(e)) => Outcome::Runtime(first_line(&e)),
                }
            }
            _ if task == "B03" => {
                let args = niles_args(&tasks::correction_args()).expect("no capability");
                let legs = match niles::run_fn(program, "correct", &args, &ctx.data) {
                    Ok(a) => a,
                    Err(niles::RunErr::NotInSubset(f)) => {
                        return Outcome::Unexecuted(format!("niles-interp refuses `{f}`"))
                    }
                    Err(niles::RunErr::Failed(e)) => return Outcome::Runtime(first_line(&e)),
                };
                // The interpreter's legs carry no value date: B03's correction is back-valued
                // by the program, which this executor cannot see. Recorded, not guessed.
                let _ = legs;
                Outcome::Unexecuted(
                    "niles-interp's legs carry no value date, so the back-valued correction cannot be read back".into(),
                )
            }
            _ => {
                let extra: Vec<Posting> = Vec::new();
                match niles::eval_views(
                    program,
                    &tasks::outputs(task),
                    kinds,
                    &ctx.data,
                    &extra,
                    like,
                ) {
                    Ok(a) => Outcome::Answer(a),
                    Err(e) => Outcome::Runtime(first_line(&e)),
                }
            }
        },
        other => Outcome::Blocked(format!("no surface {other}")),
    }
}

/// Check, then execute: the full verdict on one program.
pub fn verdict(ctx: &mut Ctx, task: &str, surface: &str, program: &str, like: &Answer) -> Outcome {
    match check(ctx, surface, program) {
        Err(e) => Outcome::Blocked(e),
        Ok(Some(codes)) => Outcome::Static(codes),
        Ok(None) => execute(ctx, task, surface, program, like),
    }
}

/// A mutant's class (design §6), in the design's order: refused by the checker (`static`),
/// raised when run (`runtime`), ran with a different answer from the unmutated program's
/// (`silent`), ran with the same answer (`equivalent`) — or, accepted when its program has no
/// executor (`base = None`) or it has none itself, `unexecuted` (A1). With the evidence.
pub fn classify(
    ctx: &mut Ctx,
    task: &str,
    surface: &str,
    mutant: &str,
    base: Option<&Answer>,
    like: &Answer,
) -> (&'static str, String) {
    match check(ctx, surface, mutant) {
        Err(e) => panic!("{task} {surface}: the checker could not run: {e}"),
        Ok(Some(codes)) => ("static", codes.join(" ")),
        Ok(None) => {
            let Some(base) = base else {
                return ("unexecuted", "its program has no executor".into());
            };
            match execute(ctx, task, surface, mutant, like) {
                Outcome::Static(c) => ("static", c.join(" ")),
                Outcome::Runtime(e) => ("runtime", e),
                Outcome::Unexecuted(e) => ("unexecuted", e),
                Outcome::Answer(a) => match base.diff(&a) {
                    None => ("equivalent", String::new()),
                    Some(d) => ("silent", d),
                },
                Outcome::Blocked(e) => panic!("{task} {surface}: blocked: {e}"),
            }
        }
    }
}
