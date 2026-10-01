//! `e30c` — E30c's driver (design `docs/study/E30c-design.md`, cycle 16, C16-10).
//!
//! ```text
//! e30c run      results/E30c-dialect/outcomes.tsv: every case, twin and mechanism
//! e30c report   results/E30c-dialect.md, rendered from the rows
//! ```
//!
//! `run` needs PostgreSQL 16 with the provisioned role `bench`, a database `bank` (stock: no
//! event trigger) and a database `dialect` into which `tools/e30c-install.sh` has installed
//! `corpus/e30c/schema/dialect.sql` as the cluster's superuser. Every script runs as `bench`, in
//! `begin … rollback`, after `schema/base.sql`.

use std::path::{Path, PathBuf};

const BASE: &str = include_str!("../../corpus/e30c/schema/base.sql");
const CASES: &str = include_str!("../../corpus/e30c/cases.tsv");

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/e30c")
}

fn results() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results")
}

struct Case {
    id: String,
    class: String,
    expected: String,
}

fn cases() -> Vec<Case> {
    CASES
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            Case {
                id: f[0].into(),
                class: f[1].into(),
                expected: f[2].into(),
            }
        })
        .collect()
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\n'], " ")
}

/// The checker's outcome: `refused CODES` or `accepted`, over base + script.
fn check(script: &str, dialect: bool) -> String {
    let src = format!("{BASE}\n{script}\n");
    match nilescheck_sql::parse(&src) {
        Err(e) => format!("parse-error {}", clean(&e.msg)),
        Ok((stmts, _)) => {
            let d = if dialect {
                nilescheck_sql::check_dialect(&stmts)
            } else {
                nilescheck_sql::check_all(&stmts)
            };
            let mut codes: Vec<&str> = d.iter().filter(|x| x.error).map(|x| x.code).collect();
            codes.sort();
            codes.dedup();
            if codes.is_empty() {
                "accepted".into()
            } else {
                format!("refused {}", codes.join(" "))
            }
        }
    }
}

/// PostgreSQL's outcome, in database `db`: `refused <first line>` or `accepted`.
fn execute(db: &str, script: &str) -> String {
    let port = std::env::var("PGPORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5432);
    let mut c = match bank_bench::wire::Client::connect("127.0.0.1", port, "bench", db) {
        Ok(c) => c,
        Err(e) => return format!("blocked {}", clean(&e.to_string())),
    };
    let _ = c.simple("rollback");
    let out = c
        .simple("begin")
        .and_then(|_| c.simple(BASE))
        .and_then(|_| c.simple(script))
        .and_then(|_| c.simple("set constraints all immediate"));
    let _ = c.simple("rollback");
    match out {
        Ok(_) => "accepted".into(),
        Err(bank_bench::wire::WireError::Server { message, .. }) => {
            format!("refused {}", clean(message.lines().next().unwrap_or("")))
        }
        Err(e) => format!("blocked {}", clean(&e.to_string())),
    }
}

fn run() {
    let mut rows = String::from("case\tclass\texpected\ttwin\tmechanism\toutcome\n");
    for c in cases() {
        for twin in ["d", "c"] {
            let p = corpus().join(&c.id).join(format!("{twin}.sql"));
            let script = std::fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
            for (mech, outcome) in [
                ("event-trigger", execute("dialect", &script)),
                ("catalog-pass", check(&script, true)),
                ("baseline-sqlcl", check(&script, false)),
                ("stock-postgresql", execute("bank", &script)),
            ] {
                rows += &format!(
                    "{}\t{}\t{}\t{twin}\t{mech}\t{outcome}\n",
                    c.id, c.class, c.expected
                );
            }
        }
    }
    let dir = results().join("E30c-dialect");
    std::fs::create_dir_all(&dir).expect("results dir");
    std::fs::write(dir.join("outcomes.tsv"), rows).expect("write outcomes");
    println!("wrote {}", dir.join("outcomes.tsv").display());
}

fn report() {
    let tsv = std::fs::read_to_string(results().join("E30c-dialect/outcomes.tsv"))
        .expect("run `e30c run` first");
    let rows: Vec<Vec<String>> = tsv
        .lines()
        .skip(1)
        .map(|l| l.split('\t').map(String::from).collect())
        .collect();
    let get = |case: &str, twin: &str, mech: &str| -> String {
        rows.iter()
            .find(|r| r[0] == case && r[3] == twin && r[4] == mech)
            .map(|r| r[5].clone())
            .unwrap_or_else(|| "missing".into())
    };
    let refused = |o: &str| o.starts_with("refused");
    let blocked = rows.iter().any(|r| r[5].starts_with("blocked"));

    let mut md = String::from(
        "# E30c — can the dialect refuse schema-level omission?\n\n\
         **Generated by `cargo run -q -p syntax-study --bin e30c -- report` from \
         `results/E30c-dialect/outcomes.tsv`. Do not edit.** Design: \
         `docs/study/E30c-design.md` (pre-registered, `a038646`). Corpus: \
         `crates/syntax-study/corpus/e30c/`.\n\n",
    );
    if blocked {
        md += "**A run was blocked** (no server, role or database): the verdict below is not a \
               verdict, and the blocked cells say why.\n\n";
    }
    md += "## Every case\n\n| case | omission | predicted | twin | event trigger (DDL) | catalog pass | baseline SQL+C+L | stock PostgreSQL |\n|---|---|---|---|---|---|---|---|\n";
    let mut residual = Vec::new();
    let mut false_refusals = Vec::new();
    for c in cases() {
        for twin in ["d", "c"] {
            let et = get(&c.id, twin, "event-trigger");
            let cp = get(&c.id, twin, "catalog-pass");
            let bl = get(&c.id, twin, "baseline-sqlcl");
            let st = get(&c.id, twin, "stock-postgresql");
            md += &format!(
                "| {} | {} | {} | {} | {et} | {cp} | {bl} | {st} |\n",
                c.id,
                c.class,
                c.expected,
                if twin == "d" { "defective" } else { "correct" }
            );
            if twin == "d" && !refused(&et) && !refused(&cp) {
                residual.push(format!("{} ({})", c.id, c.class));
            }
            if twin == "c" && (refused(&et) || refused(&cp)) {
                false_refusals.push(format!(
                    "{} ({}{})",
                    c.id,
                    if refused(&et) { "event trigger" } else { "" },
                    if refused(&cp) {
                        if refused(&et) {
                            ", catalog pass"
                        } else {
                            "catalog pass"
                        }
                    } else {
                        ""
                    }
                ));
            }
        }
    }
    md += "\n## The verdict, by the design's rule (§1)\n\n";
    if blocked {
        md += "Not given: a run was blocked.\n";
    } else if residual.is_empty() {
        md += "**Every defective script is refused**, at DDL time or at check time. By the rule, \
               Niles's schema surface becomes specification only, on this corpus.\n";
    } else {
        md += &format!(
            "**Residual: {}.** These defective scripts were accepted by both the event trigger \
             and the catalog pass; the Niles grammar is kept for their classes.\n",
            residual.join(", ")
        );
    }
    md += &format!(
        "\n**False refusals** (a correct twin refused): {}.\n",
        if false_refusals.is_empty() {
            "none".to_string()
        } else {
            false_refusals.join(", ")
        }
    );
    md += "\n## What this does not show\n\n\
           Fourteen cases written by the builder of the rule show that a class can slip past a \
           mechanism, not how often one does in a bank's schema. A superuser can turn the event \
           trigger off; the catalog pass in the gate is the answer to that, and only when the \
           gate runs. Runtime cost of the trigger on DDL is not measured.\n";
    std::fs::write(results().join("E30c-dialect.md"), md).expect("write report");
    println!("wrote results/E30c-dialect.md");
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("run") => run(),
        Some("report") => report(),
        _ => {
            eprintln!("usage: e30c run | e30c report");
            std::process::exit(2);
        }
    }
}
