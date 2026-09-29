//! **E14's SQL columns, counted from the run** (cycle 14, R2-05): for the thirteen classes of
//! `crates/counterproposal/sql-checked/` and the five d14 spellings of
//! `crates/counterproposal/d14/sql/`, the codes the catalog checker (R2-04's column,
//! `check_catalog`) and SQL+C+L (`check_all`) draw, each file checked as the preamble plus the
//! file. The totals lines are required verbatim in `results/E14-minimal-counterproposal.md`,
//! so the document cannot drift from the run.

use std::path::{Path, PathBuf};

fn cp() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../counterproposal")
}

type Checker = fn(&[nilescheck_sql::ast::Stmt]) -> Vec<nilescheck_sql::check::Diag>;

/// (file stem, sorted distinct codes, whether any is an error).
fn run(dir: &Path, prefix: &str, check: Checker) -> Vec<(String, Vec<&'static str>, bool)> {
    let pre = std::fs::read_to_string(cp().join("sql-checked/_preamble.sql")).unwrap();
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with(prefix))
        .collect();
    files.sort_by_key(|p| {
        let n = p.file_stem().unwrap().to_string_lossy().to_string();
        let num: String = n[1..].chars().take_while(|c| c.is_ascii_digit()).collect();
        (num.parse::<u32>().unwrap_or(0), n)
    });
    files
        .iter()
        .map(|f| {
            let src = format!("{pre}\n{}", std::fs::read_to_string(f).unwrap());
            let (stmts, _) = nilescheck_sql::parse(&src)
                .unwrap_or_else(|e| panic!("{}: {}", f.display(), e.msg));
            let d = check(&stmts);
            let error = d.iter().any(|x| x.error);
            let mut codes: Vec<&'static str> = d.into_iter().map(|x| x.code).collect();
            codes.sort();
            codes.dedup();
            (
                f.file_stem().unwrap().to_string_lossy().to_string(),
                codes,
                error,
            )
        })
        .collect()
}

fn classes(check: Checker) -> Vec<(String, Vec<&'static str>, bool)> {
    run(&cp().join("sql-checked"), "d", check)
}

fn d14(check: Checker) -> Vec<(String, Vec<&'static str>, bool)> {
    run(&cp().join("d14/sql"), "d14_", check)
}

const SQL_C_L: &[(&str, &[&str])] = &[
    ("d1_unbalanced", &["NL0300"]),
    ("d2_cross_currency_sum", &["NL0250"]),
    ("d3_wrong_scale", &["NL0240"]),
    ("d4_mixed_currency_txn", &["NL0300"]),
    ("d5_stale_authorization", &["NL0311"]),
    ("d6_wall_clock_predicate", &["IR013"]),
    ("d7_stale_read_after_boundary", &["NL0310"]),
    ("d8_confidential_predicate", &["NL0260"]),
    ("d9_unauthorized_overdraft", &["NL0312"]),
    ("d10_ledger_without_conserve", &["NL0211"]),
    ("d11_update_ledger", &["NL0230"]),
    ("d12_infeasible_contract", &["NL0220"]),
    ("d13_missing_anchor", &["NL0223"]),
    ("d14_1_bare_call", &["NL0320"]),
    ("d14_2_discarded_binding", &["NL0320"]),
    ("d14_3_bound_read_dropped", &["NL0320"]),
    ("d14_4_explicit_ignore", &["NL0320"]),
    ("d14_5_forget", &["NL0320"]),
];

pub fn sql_c_l_line(c: &[(String, Vec<&str>, bool)], s: &[(String, Vec<&str>, bool)]) -> String {
    let caught = c.iter().filter(|x| !x.1.is_empty()).count();
    let refused = c.iter().filter(|x| x.2).count();
    let d14 = s.iter().filter(|x| x.2).count();
    format!(
        "SQL+C+L: {caught} of {} classes caught at check time ({refused} refused, {} as a warning) and {d14} of {} d14 spellings refused.",
        c.len(),
        caught - refused,
        s.len()
    )
}

pub fn catalog_d14_line(s: &[(String, Vec<&str>, bool)]) -> String {
    let caught = s.iter().filter(|x| !x.1.is_empty()).count();
    format!(
        "PostgreSQL + checker, d14: {caught} of {} spellings caught (the catalog checker has no rule over a function's paths).",
        s.len()
    )
}

#[test]
fn sql_c_l_draws_exactly_its_codes_on_every_class_and_spelling() {
    let got: Vec<(String, Vec<&str>)> = classes(nilescheck_sql::check_all)
        .into_iter()
        .chain(d14(nilescheck_sql::check_all))
        .map(|(n, c, _)| (n, c))
        .collect();
    let want: Vec<(String, Vec<&str>)> = SQL_C_L
        .iter()
        .map(|(n, c)| (n.to_string(), c.to_vec()))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn the_catalog_checker_reaches_no_d14_spelling() {
    for (n, c, _) in d14(nilescheck_sql::check_catalog) {
        assert!(c.is_empty(), "{n}: {c:?}");
    }
}

#[test]
fn every_control_is_accepted_by_both_sql_checkers() {
    let pre = std::fs::read_to_string(cp().join("sql-checked/_preamble.sql")).unwrap();
    let ok = std::fs::read_to_string(cp().join("d14/sql/ok_resolved.sql")).unwrap();
    let (stmts, _) = nilescheck_sql::parse(&format!("{pre}\n{ok}")).unwrap();
    assert!(nilescheck_sql::check_all(&stmts).is_empty());
    assert!(nilescheck_sql::check_catalog(&stmts).is_empty());
}

#[test]
fn the_e14_document_carries_the_sql_totals() {
    let doc = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results/E14-minimal-counterproposal.md"),
    )
    .unwrap();
    let c = classes(nilescheck_sql::check_all);
    let s = d14(nilescheck_sql::check_all);
    for line in [
        sql_c_l_line(&c, &s),
        catalog_d14_line(&d14(nilescheck_sql::check_catalog)),
    ] {
        assert!(
            doc.contains(&line),
            "E14's document must say, verbatim:\n  {line}"
        );
    }
}
