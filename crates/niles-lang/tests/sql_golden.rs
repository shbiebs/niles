//! **The SQL fragment, checked by what it computes.**
//!
//! Thesis §4.7(c) and §9.7 claim golden-file α-equivalence tests for the SQL surface. There
//! were none: `grep -ri golden crates` returned nothing, and `sql_surface.rs` marked forms
//! `Lowered` or `Equivalent` on the strength of nobody having checked. Several of those
//! forms did not lower at all — `SELECT a, b FROM t` emitted no projection, `DISTINCT` and
//! the set operators were parsed and never read, a from-list of two tables silently used the
//! first, and `JOIN ... ON` resolved the condition against the left schema only, so it found
//! nothing and the join ran unconstrained.
//!
//! This file is the missing test, and it is stronger than α-equivalence of circuits. Two
//! circuits can be structurally different and denote the same Z-set, and structurally
//! identical while both being wrong. So each case states **what the query denotes** on a
//! fixed dataset, and the SQL and pipeline spellings are both evaluated with
//! `niles_ir::eval` and compared against that.
//!
//! # A case
//!
//! ```text
//! tests/golden/07_distinct.sql       select distinct k from t
//! tests/golden/07_distinct.niles     t.map(|r| r.k).distinct()
//! tests/golden/07_distinct.expected  k
//!                                    1 x1
//!                                    2 x1
//!                                    3 x1
//! ```
//!
//! A case whose `.expected` file says `REFUSED NL05xx` asserts the refusal instead, and that
//! is how the fragment narrows honestly: a form that cannot be lowered is named, with its
//! code, in a file, rather than lowering "approximately".
//!
//! The dataset is in `tests/golden/DATA.md`, written out so an `.expected` file can be read
//! without running anything.

use niles_ir::eval::{self, ZSet};
use niles_lang::{lower, parser, resolve};
use std::collections::BTreeMap;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

/// The fixed dataset. See `tests/golden/DATA.md`.
fn sources() -> BTreeMap<String, ZSet> {
    let mut m = BTreeMap::new();
    m.insert(
        "t".to_string(),
        eval::zset(&[
            (&[1, 10, 100], 1),
            (&[1, 10, i128::MIN], 1), // `n` is null; see `null_row` below
            (&[2, 20, 200], 1),
            (&[3, 30, i128::MIN], 1),
        ]),
    );
    // The two rows above with a null `n` cannot be written with `zset`, which takes
    // integers, so they are rewritten here. Done explicitly rather than by extending
    // `zset`, because a helper that accepted a sentinel for null is exactly how a null and
    // a number stop being distinguishable.
    let t = m.get_mut("t").expect("t");
    let nulled: Vec<(Vec<niles_ir::value::Value>, i128)> = t
        .iter()
        .map(|(r, w)| {
            let mut r = r.clone();
            if r[2] == niles_ir::value::Value::Int(i128::MIN) {
                r[2] = niles_ir::value::Value::Null;
            }
            (r, *w)
        })
        .collect();
    *t = nulled.into_iter().collect();

    m.insert(
        "u".to_string(),
        eval::zset(&[(&[1, 1], 1), (&[2, 2], 1), (&[4, 4], 1)]),
    );
    m.insert(
        "edges".to_string(),
        eval::zset(&[
            (&[1, 2], 1),
            (&[2, 3], 1),
            (&[3, 4], 1),
            (&[5, 6], 1),
            (&[6, 5], 1),
        ]),
    );
    m
}

struct Case {
    name: String,
    sql: Option<String>,
    niles: Option<String>,
    expected: String,
}

fn cases() -> Vec<Case> {
    let mut names: Vec<String> = std::fs::read_dir(DIR)
        .expect("golden dir")
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            (p.extension().and_then(|x| x.to_str()) == Some("expected"))
                .then(|| p.file_stem()?.to_str().map(str::to_string))
                .flatten()
        })
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| Case {
            expected: std::fs::read_to_string(format!("{DIR}/{name}.expected"))
                .expect("expected file"),
            sql: std::fs::read_to_string(format!("{DIR}/{name}.sql")).ok(),
            niles: std::fs::read_to_string(format!("{DIR}/{name}.niles")).ok(),
            name,
        })
        .collect()
}

/// Wrap a query body in a view over the shared schema and lower it.
fn compile(body: &str, surface: &str) -> Result<(niles_ir::circuit::Circuit, ZSet), Vec<String>> {
    let schema = std::fs::read_to_string(format!("{DIR}/schema.niles")).expect("schema");
    let src = match surface {
        "sql" => format!(
            "{schema}\nview golden = sql {{ {body} }} serve {{ consistency: snapshot, materialize: auto }};\n"
        ),
        _ => format!(
            "{schema}\nview golden = {body}\n    serve {{ consistency: snapshot, materialize: auto }};\n"
        ),
    };
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, rd) = resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (lowered, ld) = lower::lower_program(&prog, &cat);
    d.extend(ld);
    let errors: Vec<String> = d
        .items
        .iter()
        .filter(|x| x.severity == niles_lang::diagnostics::Severity::Error)
        .map(|x| x.code.to_string())
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    match eval::try_run(&lowered.circuit, "golden", &sources()) {
        Ok((z, _work)) => Ok((lowered.circuit, z)),
        Err(e) => Err(vec![format!("EVAL {e}")]),
    }
}

/// A Z-set as the `.expected` files write it: one row per line, sorted, `<cols> x<weight>`.
fn render(z: &ZSet) -> String {
    let mut lines: Vec<String> = z
        .iter()
        .filter(|(_, w)| **w != 0)
        .map(|(r, w)| {
            let cols: Vec<String> = r
                .iter()
                .map(|v| match v {
                    niles_ir::value::Value::Null => "null".to_string(),
                    niles_ir::value::Value::Int(i) => i.to_string(),
                })
                .collect();
            format!("{} x{w}", cols.join(" "))
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

fn expected_body(e: &str) -> String {
    e.lines()
        .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_corpus_is_large_enough_to_be_a_corpus() {
    let cs = cases();
    assert!(
        cs.len() >= 30,
        "the work order asks for at least 30 golden cases; there are {}",
        cs.len()
    );
    // Every case must have at least one spelling, or the file pair is a typo that would
    // otherwise pass by testing nothing.
    for c in &cs {
        assert!(
            c.sql.is_some() || c.niles.is_some(),
            "`{}` has an .expected and neither a .sql nor a .niles",
            c.name
        );
    }
}

#[test]
fn every_case_denotes_what_its_expected_file_says() {
    let mut failures = Vec::new();
    for c in cases() {
        let want = expected_body(&c.expected);
        for (surface, body) in [("sql", &c.sql), ("niles", &c.niles)] {
            let Some(body) = body else { continue };
            let body = body.trim();
            match (compile(body, surface), want.strip_prefix("REFUSED ")) {
                // A case the file says is refused, and it is — with the named code.
                (Err(codes), Some(want)) => {
                    // Every code the file names must be present. A file naming two codes is
                    // saying the query is refused for two reasons, and a check that
                    // accepted "at least one of them" would pass while half the refusal
                    // quietly stopped happening.
                    for code in want.split_whitespace() {
                        if !codes.iter().any(|c| c == code) {
                            failures.push(format!(
                                "{}[{surface}]: expected refusal {code}, got {codes:?}",
                                c.name
                            ));
                        }
                    }
                }
                (Ok(_), Some(code)) => failures.push(format!(
                    "{}[{surface}]: must be refused with {code}, but it compiled",
                    c.name
                )),
                (Err(codes), None) => failures.push(format!(
                    "{}[{surface}]: must compile, but was refused with {codes:?}",
                    c.name
                )),
                (Ok((_, z)), None) => {
                    let got = render(&z);
                    if got != want {
                        failures.push(format!(
                            "{}[{surface}]:\n  expected:\n{}\n  got:\n{}",
                            c.name,
                            indent(&want),
                            indent(&got)
                        ));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} golden case(s) disagree with their expected file:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn indent(s: &str) -> String {
    s.lines()
        .map(|l| format!("    {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_two_surfaces_denote_the_same_zset_wherever_both_are_written() {
    // **The generality claim, made operational.** Where a case has both spellings, they must
    // produce the same answer — not merely both compile. This is the property that would be
    // marketing if it were only asserted: `select acct, sum(amt) from postings group by
    // acct` and `postings.group_by(..).sum(..)` are the same query or they are not.
    let mut pairs = 0;
    let mut skipped: Vec<String> = Vec::new();
    for c in cases() {
        let (Some(sql), Some(niles)) = (&c.sql, &c.niles) else {
            // Written in one surface only. Most of these are SQL forms with no pipeline
            // spelling (`FROM t, u`, a correlated `EXISTS`), and the mapping table of
            // `sql_surface.rs` says so per row; a few are refusals where a second spelling
            // would test nothing.
            skipped.push(format!("{}: one surface only", c.name));
            continue;
        };
        if expected_body(&c.expected).starts_with("REFUSED") {
            // A refusal is checked by `every_golden_case_denotes_what_its_file_says`, in
            // both surfaces, against the named code. Comparing the *denotations* of two
            // queries that do not compile would compare nothing.
            skipped.push(format!("{}: refused in both surfaces", c.name));
            continue;
        }
        let a = compile(sql.trim(), "sql");
        let b = compile(niles.trim(), "niles");
        match (a, b) {
            (Ok((_, x)), Ok((_, y))) => {
                assert_eq!(
                    render(&x),
                    render(&y),
                    "`{}`: the SQL and pipeline spellings denote different Z-sets",
                    c.name
                );
                pairs += 1;
            }
            (a, b) => panic!(
                "`{}`: one surface compiled and the other did not: sql={:?} niles={:?}",
                c.name,
                a.map(|_| "ok"),
                b.map(|_| "ok")
            ),
        }
    }
    assert!(
        pairs >= 18,
        "at least eighteen cases should be written in both surfaces; {pairs} were"
    );
    // **What is skipped, and by how much.** A `continue` inside a loop is how a corpus
    // quietly shrinks: the count goes up, the coverage does not, and nothing says which
    // cases stopped being compared. The list is printed and its length is bounded, so
    // adding a case that silently opts out of the comparison fails here.
    assert!(
        skipped.len() <= 34,
        "{} cases are skipped by this comparison, which is more than the corpus leaves \
         uncompared today (34: thirty-two written in one surface, two refused in both). The \
         two most recent are `58_order_by_aggregate` and `59_order_by_alias`, and the reason \
         is a gap in the *pipeline* surface rather than in the corpus: its `order_by` stage \
         has no descending spelling — `key_of` maps every key to `(k, true)` — so the case \
         that discriminates a working `order by` from a silently empty one cannot be \
         written there. An ascending one cannot: the broken lowering and a correct \
         ascending sort pick the same rows:\n{}",
        skipped.len(),
        skipped.join("\n")
    );
}

#[test]
fn a_non_terminating_fixpoint_is_refused_rather_than_answered() {
    // The negative control for the fixpoint evaluator. A step that adds a new row every
    // round never converges, and the evaluator must say so rather than return whatever it
    // had reached when the budget ran out — an answer that looks like an answer.
    //
    // Built directly as a circuit, because the surface refuses an unguarded fixpoint and
    // this is about what happens when the *guard is wrong*, which is a runtime fact.
    use niles_ir::circuit::internal_contract;
    use niles_ir::circuit::Circuit;
    use niles_ir::operator::{Op, Scalar, ScalarOp};
    use niles_ir::ServeContract;
    let c: ServeContract = internal_contract();
    let mut circuit = Circuit::new();
    let src = circuit.add(
        Op::Source {
            relation: "edges".into(),
            is_base: true,
            anchor_key: vec![0],
        },
        vec![],
        c,
        "edges",
    );
    let delay = circuit.add(Op::Delay, vec![src], c, "acc");
    // `(src + 1, dst + 1)` — a step that produces a row nobody has seen, every round.
    let grow = circuit.add(
        Op::Map {
            exprs: vec![
                Scalar::Binary {
                    op: ScalarOp::Add,
                    lhs: Box::new(Scalar::Column(0)),
                    rhs: Box::new(Scalar::LitInt(1)),
                },
                Scalar::Binary {
                    op: ScalarOp::Add,
                    lhs: Box::new(Scalar::Column(1)),
                    rhs: Box::new(Scalar::LitInt(1)),
                },
            ],
        },
        vec![delay],
        c,
        "grow",
    );
    let fix = circuit.add(
        Op::Fixpoint {
            measure: Scalar::Column(0),
            max_rounds: 50,
        },
        vec![src, grow],
        c,
        "fixpoint",
    );
    circuit.set_output("diverges", fix);

    let err = eval::try_run(&circuit, "diverges", &sources())
        .expect_err("a step that grows every round must not report an answer");
    match err {
        eval::EvalError::NonTerminating { rounds, tail } => {
            assert_eq!(rounds, 50);
            assert!(
                tail.windows(2).all(|w| w[1] > w[0]),
                "the report should show the accumulator still growing: {tail:?}"
            );
        }
    }

    // And the control: the same shape with a converging step reaches closure and returns.
    let mut circuit = Circuit::new();
    let src = circuit.add(
        Op::Source {
            relation: "edges".into(),
            is_base: true,
            anchor_key: vec![0],
        },
        vec![],
        c,
        "edges",
    );
    let delay = circuit.add(Op::Delay, vec![src], c, "acc");
    let same = circuit.add(
        Op::Map {
            exprs: vec![Scalar::Column(0), Scalar::Column(1)],
        },
        vec![delay],
        c,
        "identity",
    );
    let fix = circuit.add(
        Op::Fixpoint {
            measure: Scalar::Column(0),
            max_rounds: 50,
        },
        vec![src, same],
        c,
        "fixpoint",
    );
    circuit.set_output("converges", fix);
    let (z, _) = eval::try_run(&circuit, "converges", &sources()).expect("this one converges");
    assert_eq!(z.len(), 5, "the five edges, and nothing new");
}

/// Print what every case denotes, for writing the `.expected` files.
///
/// Ignored: it is a generator, not a check. Every file it helps write is then read against
/// the SQL standard by hand — a corpus whose expectations were produced by the thing under
/// test proves only that the thing is consistent with itself.
///
/// ```sh
/// cargo test -p niles-lang --test sql_golden -- --ignored --nocapture generate
/// ```
#[test]
#[ignore = "a generator, not a check"]
fn generate() {
    for c in cases() {
        for (surface, body) in [("sql", &c.sql), ("niles", &c.niles)] {
            let Some(body) = body else { continue };
            println!("===== {} [{surface}] {}", c.name, body.trim());
            match compile(body.trim(), surface) {
                Ok((_, z)) => println!("{}", render(&z)),
                Err(codes) => println!("REFUSED {}", codes.join(" ")),
            }
        }
    }
}

#[test]
fn the_status_of_every_form_is_backed_by_a_corpus_case() {
    // **The check that keeps the mapping table honest.** Its statuses used to be typed in:
    // `SELECT a, b FROM t` was marked `Lowered` while emitting no projection at all, and
    // `DISTINCT` was `Lowered` while being parsed and ignored. A status nobody can check is
    // a claim, and this file exists to stop the table making claims.
    //
    // Every non-excluded, non-specified row must name at least one corpus case in a comment
    // on its own line, and every case it names must exist.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sql_surface.rs"))
        .expect("sql_surface.rs");
    let names: Vec<String> = cases().into_iter().map(|c| c.name).collect();

    let mut unbacked = Vec::new();
    let mut missing = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        // One-line entries only; the multi-line ones are the `Excluded` rows, whose reason
        // is their justification and which have nothing to compute.
        if !t.starts_with("Mapping {") || !t.contains("status:") {
            continue;
        }
        if t.contains("Status::Excluded")
            || t.contains("Status::Specified")
            || t.contains("Status::Untested")
        {
            continue;
        }
        let Some((_, comment)) = t.split_once("// ") else {
            unbacked.push(t.to_string());
            continue;
        };
        let refs: Vec<&str> = comment
            .split(',')
            .map(str::trim)
            .filter(|x| x.chars().all(|c| c.is_ascii_digit()) && !x.is_empty())
            .collect();
        if refs.is_empty() {
            unbacked.push(t.to_string());
            continue;
        }
        for r in refs {
            if !names.iter().any(|n| n.starts_with(&format!("{r}_"))) {
                missing.push(format!("{r} (named by `{t}`)"));
            }
        }
    }
    assert!(
        unbacked.is_empty(),
        "these rows claim a status with no corpus case behind it:\n  {}",
        unbacked.join("\n  ")
    );
    assert!(
        missing.is_empty(),
        "these rows name a corpus case that does not exist:\n  {}",
        missing.join("\n  ")
    );
}
