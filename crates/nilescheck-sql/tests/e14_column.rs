//! **E14's "PostgreSQL + checker" column, counted from the run** (R2-04).
//!
//! Each defect class of `crates/counterproposal/sql-checked/` is checked as the preamble plus
//! the defect, and its verdict is the stage it is caught at: `check` with the codes, or not
//! caught — with the reason this arm cannot reach it named, not guessed. The totals line is
//! required, verbatim, in `results/E14-minimal-counterproposal.md`, so the document cannot
//! drift from the run.

use std::path::Path;

fn dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../counterproposal/sql-checked")
}

fn verdicts() -> Vec<(String, Vec<String>)> {
    let pre = std::fs::read_to_string(dir().join("_preamble.sql")).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(dir())
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with('d'))
        .collect();
    files.sort_by_key(|p| {
        let n = p.file_name().unwrap().to_string_lossy().to_string();
        n[1..].split('_').next().unwrap().parse::<u32>().unwrap()
    });
    files
        .iter()
        .map(|f| {
            let src = format!("{pre}\n{}", std::fs::read_to_string(f).unwrap());
            let (stmts, _) = nilescheck_sql::parse(&src)
                .unwrap_or_else(|e| panic!("{}: {}", f.display(), e.msg));
            let mut codes: Vec<String> = nilescheck_sql::check_all(&stmts)
                .into_iter()
                .map(|d| d.code.to_string())
                .collect();
            codes.sort();
            codes.dedup();
            (f.file_stem().unwrap().to_string_lossy().to_string(), codes)
        })
        .collect()
}

/// What this arm is expected to reach, and for the three it cannot, why.
const EXPECTED: &[(&str, &[&str])] = &[
    ("d1_unbalanced", &[]), // conservation needs the solver over the postings (R2-05)
    ("d2_cross_currency_sum", &["NL0250"]),
    ("d3_wrong_scale", &["NL0240"]),
    ("d4_mixed_currency_txn", &[]), // the solver again (R2-05); PostgreSQL's trigger catches it at COMMIT
    ("d5_stale_authorization", &["NL0311"]),
    ("d6_wall_clock_predicate", &["IR013"]),
    ("d7_stale_read_after_boundary", &["NL0310"]),
    ("d8_confidential_predicate", &["NL0260"]),
    ("d9_unauthorized_overdraft", &[]), // an authority is a capability; SQL has no type to carry one
    ("d10_ledger_without_conserve", &["NL0211"]),
    ("d11_update_ledger", &["NL0230"]),
    ("d12_infeasible_contract", &["NL0220"]),
    ("d13_missing_anchor", &["NL0223"]),
];

pub fn totals_line(v: &[(String, Vec<String>)]) -> String {
    let caught = v.iter().filter(|(_, c)| !c.is_empty()).count();
    format!(
        "PostgreSQL + checker: {caught} of {} caught at check time, {} not caught (d1 and d4 need the conservation solver, R2-05; d9 needs a capability SQL cannot type).",
        v.len(),
        v.len() - caught
    )
}

#[test]
fn the_preamble_alone_is_clean() {
    let pre = std::fs::read_to_string(dir().join("_preamble.sql")).unwrap();
    let (stmts, _) = nilescheck_sql::parse(&pre).unwrap();
    let d = nilescheck_sql::check_all(&stmts);
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn each_defect_class_draws_exactly_its_codes() {
    let v = verdicts();
    assert_eq!(v.len(), 13);
    for ((name, codes), (want_name, want)) in v.iter().zip(EXPECTED) {
        assert_eq!(name, want_name);
        assert_eq!(
            codes,
            &want.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "{name}"
        );
    }
    eprintln!("{}", totals_line(&v));
}

#[test]
fn the_e14_document_carries_the_counted_totals() {
    let doc = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results/E14-minimal-counterproposal.md"),
    )
    .unwrap();
    let line = totals_line(&verdicts());
    assert!(
        doc.contains(&line),
        "E14 must carry the checker column's counted totals:\n  {line}"
    );
}
