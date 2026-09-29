//! **PostgreSQL is the validator** (R2-04): every text this parser accepts, PostgreSQL 16's
//! parser accepts, and every text it refuses, PostgreSQL refuses with a syntax error.
//!
//! "Accepts" means *no syntax error* (SQLSTATE 42601): a statement about a table that does not
//! exist parses in both and fails later in PostgreSQL with 42P01, which is not a disagreement
//! about the grammar. Each text runs inside `begin … rollback` in a scratch schema, so nothing
//! it creates survives. PostgreSQL is the one the gate provisions (`tools/pg-provision.sh`).

mod corpus;

use bank_bench::wire::{Client, WireError};

fn port() -> u16 {
    std::env::var("PGPORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5432)
}

fn client() -> Client {
    Client::connect("127.0.0.1", port(), "bench", "bank").unwrap_or_else(|e| {
        panic!(
            "no PostgreSQL reachable as `bench` on 127.0.0.1:{}: {e}; run tools/pg-provision.sh",
            port()
        )
    })
}

/// PostgreSQL's verdict on a text's syntax: `Ok` when it parsed (whatever happened next).
fn postgres_parses(c: &mut Client, sql: &str) -> Result<(), String> {
    c.simple("begin").map_err(|e| e.to_string())?;
    let _ = c.simple("create schema if not exists nilescheck_validate");
    let _ = c.simple("set local search_path = nilescheck_validate");
    let r = c.simple(sql);
    let _ = c.simple("rollback");
    match r {
        Ok(_) => Ok(()),
        Err(WireError::Server { sqlstate, message }) if sqlstate == "42601" => {
            Err(format!("syntax error: {message}"))
        }
        Err(_) => Ok(()),
    }
}

#[test]
fn postgresql_accepts_what_this_parser_accepts_and_refuses_what_it_refuses() {
    let mut c = client();
    let mut disagreements = Vec::new();
    let accepted = corpus::QUERIES
        .iter()
        .chain(corpus::DML)
        .chain(corpus::DDL)
        .copied()
        .chain([corpus::PLPGSQL]);
    let mut n = 0;
    for sql in accepted {
        assert!(
            nilescheck_sql::parse(sql).is_ok(),
            "this parser refuses {sql}"
        );
        if let Err(e) = postgres_parses(&mut c, sql) {
            disagreements.push(format!("accepted here, refused by PostgreSQL ({e}): {sql}"));
        }
        n += 1;
    }
    for sql in corpus::REFUSED.iter().copied().chain([corpus::ADJACENT]) {
        assert!(
            nilescheck_sql::parse(sql).is_err(),
            "this parser accepts {sql}"
        );
        if postgres_parses(&mut c, sql).is_ok() {
            disagreements.push(format!("refused here, accepted by PostgreSQL: {sql}"));
        }
        n += 1;
    }
    assert!(
        disagreements.is_empty(),
        "{} of {n} texts disagree:\n{}",
        disagreements.len(),
        disagreements.join("\n")
    );
}

#[test]
fn the_checker_fixture_is_valid_postgresql() {
    let mut c = client();
    let src = include_str!("fixtures/contracts.sql");
    assert!(nilescheck_sql::parse(src).is_ok());
    postgres_parses(&mut c, src).expect("PostgreSQL parses the checker's fixture");
}

/// **What PostgreSQL alone does with the checked corpus**: every defect class of
/// `crates/counterproposal/sql-checked/` is accepted at definition time — the cross-currency
/// sum inside PL/pgSQL, the fractional yen in a SQL function, the dropped conservation
/// trigger, the update in a function body. That is the column the checker is measured
/// against; if PostgreSQL started refusing one of them, E14's "PostgreSQL" cell for it would
/// be stale, and this test says which.
#[test]
fn postgresql_alone_accepts_every_checked_defect_at_definition_time() {
    let mut c = client();
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../counterproposal/sql-checked");
    let pre = std::fs::read_to_string(dir.join("_preamble.sql")).unwrap();
    let mut refused = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    for f in files
        .iter()
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with('d'))
    {
        let src = format!("{pre}\n{}", std::fs::read_to_string(f).unwrap());
        c.simple("begin").unwrap();
        let _ = c.simple("create schema if not exists nilescheck_e14");
        let _ = c.simple("set local search_path = nilescheck_e14");
        let r = c.simple(&src);
        let _ = c.simple("rollback");
        if let Err(e) = r {
            refused.push(format!("{}: {e}", f.display()));
        }
    }
    assert!(
        refused.is_empty(),
        "PostgreSQL refused at definition time:\n{}",
        refused.join("\n")
    );
}

/// **E14's PostgreSQL column for d14** (R2-05): each spelling of
/// `crates/counterproposal/d14/sql/` is created and run on PostgreSQL 16 with the preamble,
/// and the hold's state is read back. PostgreSQL refuses none at definition time, raises
/// nothing at run time, and leaves the hold open — `never`, in E14's terms. The control
/// resolves it. Inside `begin … rollback`, in a scratch schema.
#[test]
fn postgresql_runs_every_d14_spelling_and_leaves_the_hold_open() {
    let mut c = client();
    let cp = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../counterproposal");
    let pre = std::fs::read_to_string(cp.join("sql-checked/_preamble.sql")).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(cp.join("d14/sql"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    let mut seen = Vec::new();
    for f in &files {
        let n = f.file_stem().unwrap().to_string_lossy().to_string();
        let call = if n == "ok_resolved" {
            "select f(1, true)"
        } else {
            "select f(1)"
        };
        c.simple("begin").unwrap();
        let _ = c.simple("create schema if not exists nilescheck_d14");
        let _ = c.simple("set local search_path = nilescheck_d14");
        let created = c.simple(&format!("{pre}\n{}", std::fs::read_to_string(f).unwrap()));
        let ran = created.as_ref().ok().map(|_| c.simple(call));
        let state = c
            .simple("select string_agg(state, ',' order by id) from holds")
            .ok()
            .and_then(|r| {
                r.rows
                    .first()
                    .and_then(|row| row.first().cloned().flatten())
            });
        let _ = c.simple("rollback");
        assert!(
            created.is_ok(),
            "{n}: PostgreSQL refused it at definition time: {created:?}"
        );
        let ran = ran.unwrap();
        assert!(ran.is_ok(), "{n}: PostgreSQL raised at run time: {ran:?}");
        seen.push((n, state.unwrap_or_default()));
    }
    let want: Vec<(String, String)> = [
        ("d14_1_bare_call", "open"),
        ("d14_2_discarded_binding", "open"),
        ("d14_3_bound_read_dropped", "open"),
        ("d14_4_explicit_ignore", "open"),
        ("d14_5_forget", "open"),
        ("ok_resolved", "posted"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect();
    assert_eq!(seen, want);
    let doc = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../results/E14-minimal-counterproposal.md"),
    )
    .unwrap();
    let open = seen
        .iter()
        .filter(|(n, s)| n.starts_with("d14_") && s == "open")
        .count();
    let line = format!(
        "PostgreSQL, d14: 0 of {open} spellings refused at definition time or at run time; each runs and leaves the hold open."
    );
    assert!(
        doc.contains(&line),
        "E14's document must say, verbatim:\n  {line}"
    );
}
