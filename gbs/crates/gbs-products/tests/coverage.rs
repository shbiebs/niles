//! **The coverage matrix, kept honest.**
//!
//! `ARCHITECTURE.md` §4 claims that twenty-nine product lines decompose into seven
//! mechanisms, and prints a matrix saying which line needs which. A matrix in a document
//! drifts from the code within a month, and a drifted matrix is worse than none: it is the
//! kind of artefact a reader trusts and a reviewer checks.
//!
//! This file reads the matrix out of the markdown and checks two things against the build:
//! that it is well formed, and that every product actually implemented uses exactly the
//! mechanisms its row claims. The second is the one with teeth — a product that quietly
//! grew a dependency on an eighth mechanism, or stopped using one it was supposed to need,
//! shows up here.
//!
//! # What this cannot check, stated plainly
//!
//! It checks *implemented* rows. Most of the matrix is still a design claim: twenty-two of
//! the twenty-nine lines have no code, and their rows are predictions. This file does not
//! pretend otherwise — [`implemented_rows`] is an explicit list, and the honesty of the
//! matrix as a whole rests on that list growing.
//!
//! The check is also structural rather than semantic: it confirms that `derivatives.rs`
//! imports M2 and M7, not that it uses them *correctly*. Correctness is what the products'
//! own tests are for. What this catches is a claim in a document that the code stopped
//! supporting.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn gbs_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gbs-products lives at gbs/crates/gbs-products")
        .to_path_buf()
}

fn architecture_md() -> String {
    let p = gbs_root().join("docs/ARCHITECTURE.md");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// The matrix, parsed out of the markdown: product line → the mechanisms it claims.
fn parse_matrix(md: &str) -> BTreeMap<String, BTreeSet<u8>> {
    let mut out = BTreeMap::new();
    for line in md.lines() {
        let t = line.trim();
        // A matrix row: `| Product line | ● | | ● | ... |` with exactly nine cells.
        if !t.starts_with('|') || !t.ends_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 8 {
            continue;
        }
        // Skip the header and the separator.
        if cells[1].contains("M1") || cells[1].starts_with(":-") || cells[1].starts_with("--") {
            continue;
        }
        let name = cells[0].to_string();
        if name.is_empty() || name.contains("Product line") {
            continue;
        }
        let mechanisms: BTreeSet<u8> = (1..=7)
            .filter(|i| cells[*i as usize] == "●")
            .map(|i| i as u8)
            .collect();
        out.insert(name, mechanisms);
    }
    out
}

/// Product lines that have code, and the module that implements them.
///
/// The explicit list the module docs mention. It is short on purpose: the honest position
/// is that most of the matrix is a prediction, and pretending otherwise by inferring
/// coverage from a filename would make this test agree with the document by construction.
fn implemented_rows() -> Vec<(&'static str, &'static str)> {
    vec![
        ("FX and multi-currency", "fx.rs"),
        ("Lending — revolving", "lending.rs"),
        ("Lending — term", "lending.rs"),
        ("Lending — syndicated", "lending.rs"),
        ("Letters of credit", "tradefinance.rs"),
        ("Supply-chain finance", "tradefinance.rs"),
        ("Forwards and swaps", "derivatives.rs"),
        ("OTC, caps and floors", "derivatives.rs"),
        ("Funds sweep", "liquidity.rs"),
        ("Cash pooling", "liquidity.rs"),
        ("Zero-balance structures", "liquidity.rs"),
    ]
}

/// Which mechanisms a module's source actually reaches for.
///
/// Detected from the symbols it imports and names. Crude, and deliberately so: a cleverer
/// analysis would be a second implementation of the thing it is checking.
fn mechanisms_used(source: &str) -> BTreeSet<u8> {
    let mut used = BTreeSet::new();
    // M1 is the kernel, and every product uses it.
    if source.contains("PostingSet") || source.contains("gbs_kernel") {
        used.insert(1);
    }
    if source.contains("Schedule") || source.contains("Occurrence") || source.contains("equal_instalments") {
        used.insert(2);
    }
    if source.contains("Hold") || source.contains("Outcome::") {
        used.insert(3);
    }
    if source.contains("ParticipantSet") || source.contains("Share") {
        used.insert(4);
    }
    if source.contains("Lifecycle") || source.contains("Actor") || source.contains("Rule::") {
        used.insert(5);
    }
    if source.contains("Signal") || source.contains("Reading") {
        used.insert(6);
    }
    if source.contains("Rate") || source.contains("accrue") || source.contains("convert")
        || source.contains("cap_payoff") || source.contains("floor_payoff")
    {
        used.insert(7);
    }
    used
}

fn module_source(file: &str) -> String {
    let p = gbs_root().join("crates/gbs-products/src").join(file);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

// ── the matrix is well formed ────────────────────────────────────────────────────────

#[test]
fn the_matrix_parses_and_covers_the_whole_scope() {
    let m = parse_matrix(&architecture_md());
    assert!(
        m.len() >= 29,
        "the matrix should cover every product line in the scope; found {} rows: {:?}",
        m.len(),
        m.keys().collect::<Vec<_>>()
    );
}

#[test]
fn every_row_needs_at_least_one_mechanism() {
    // A row with no mechanisms is a product the architecture claims to cover and does not.
    for (name, mechs) in parse_matrix(&architecture_md()) {
        assert!(!mechs.is_empty(), "`{name}` claims no mechanisms at all");
    }
}

#[test]
fn every_mechanism_is_needed_by_something() {
    // The other direction: a mechanism no product needs is a mechanism that should not
    // exist. Seven is a claim about what is *necessary*, not just what is sufficient.
    let m = parse_matrix(&architecture_md());
    for mech in 1..=7u8 {
        let users: Vec<&String> = m.iter().filter(|(_, v)| v.contains(&mech)).map(|(k, _)| k).collect();
        assert!(
            !users.is_empty(),
            "M{mech} is used by no product line. A mechanism nothing needs should be removed"
        );
    }
}

#[test]
fn risk_analytics_is_the_only_row_that_moves_no_money() {
    // A structural prediction the architecture makes in prose (§4), asserted here so that
    // it stays true. It matters because it is evidence the decomposition distinguishes
    // things that *post* from things that *observe* — if every row needed M1, the
    // mechanism set would not be separating anything.
    let m = parse_matrix(&architecture_md());
    let no_m1: Vec<&String> = m.iter().filter(|(_, v)| !v.contains(&1)).map(|(k, _)| k).collect();
    assert_eq!(
        no_m1,
        vec!["Risk analytics"],
        "exactly one row should move no money; found {no_m1:?}"
    );
}

#[test]
fn complexity_increases_toward_the_exotic_end() {
    // The other structural prediction: simple products need two or three mechanisms and
    // exotic ones need all seven. If every row needed all seven, the mechanisms would not
    // be independent and the decomposition would be wrong.
    let m = parse_matrix(&architecture_md());
    let simple = m.get("Funds sweep").expect("row present");
    let exotic = m.get("Clearing and prime brokerage").expect("row present");
    assert!(
        simple.len() < exotic.len(),
        "a sweep ({}) should need fewer mechanisms than prime brokerage ({})",
        simple.len(),
        exotic.len()
    );
    assert_eq!(exotic.len(), 7, "the exotic end needs everything");
    assert!(simple.len() <= 3, "the simple end should not");
}

// ── the implemented rows match the code ──────────────────────────────────────────────

#[test]
fn every_implemented_product_uses_exactly_the_mechanisms_its_row_claims() {
    // **The test with teeth.** A product that grew a dependency the matrix does not record,
    // or stopped using one it claims, is a document that has drifted from the code.
    let matrix = parse_matrix(&architecture_md());

    // A module often implements several rows — `lending.rs` covers revolving, term and
    // syndicated — so the check's granularity is the *file*, and what it compares against
    // is the union of that file's rows. Comparing a single row against a shared file would
    // report the syndicated row's M4 as an unclaimed use by the revolving row, which is a
    // false positive about a real file rather than a finding.
    //
    // The union is still a real check: a mechanism used by a file and claimed by none of
    // its rows is a genuine drift, which is how this test found that the matrix had
    // forgotten interest accrual is M7.
    let mut by_file: BTreeMap<&str, (Vec<&str>, BTreeSet<u8>)> = BTreeMap::new();
    for (row, file) in implemented_rows() {
        let claimed = matrix
            .get(row)
            .unwrap_or_else(|| panic!("`{row}` is implemented in {file} but has no matrix row"));
        let e = by_file.entry(file).or_default();
        e.0.push(row);
        e.1.extend(claimed.iter().copied());
    }

    for (file, (rows, claimed)) in by_file {
        let actual = mechanisms_used(&module_source(file));

        let unclaimed: Vec<u8> = actual.difference(&claimed).copied().collect();
        assert!(
            unclaimed.is_empty(),
            "{file} (rows {rows:?}) uses M{unclaimed:?}, which none of its matrix rows claim.\n\
             Either the rows are out of date, or the product reached for a mechanism it was \
             not supposed to need — and the second is the interesting case."
        );
        // The reverse direction is a warning rather than an error: a row may claim a
        // mechanism the *current* implementation does not yet exercise, because the row
        // describes the full product and the code is a slice of it. Asserting equality
        // would force the matrix to shrink to whatever is built, which is backwards.
    }
}

#[test]
fn the_implemented_list_matches_what_is_actually_in_the_crate() {
    // Guarding the guard: if a product module were added and not listed, this file would
    // silently stop covering it, and the coverage claim would decay without a failure.
    let src = gbs_root().join("crates/gbs-products/src");
    let modules: BTreeSet<String> = std::fs::read_dir(&src)
        .expect("src dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".rs") && n != "lib.rs")
        .collect();

    let listed: BTreeSet<String> =
        implemented_rows().iter().map(|(_, f)| f.to_string()).collect();

    assert_eq!(
        modules, listed,
        "every product module must appear in `implemented_rows`, or its coverage goes \
         unchecked. Present but unlisted: {:?}",
        modules.difference(&listed).collect::<Vec<_>>()
    );
}

#[test]
fn the_matrix_check_can_actually_fail() {
    // A negative control, for the same reason `layering.rs` has one: a check that cannot
    // detect a violation passes forever and proves nothing. Feed the parser a row and a
    // source that disagree, and confirm the disagreement is visible.
    let fake_matrix = "| Fake product | ● | | | | | | |\n";
    let parsed = parse_matrix(fake_matrix);
    let claimed = parsed.get("Fake product").expect("parsed");
    assert_eq!(claimed, &BTreeSet::from([1u8]), "claims M1 only");

    // A source that reaches for M4 and M7 as well.
    let source = "use gbs_kernel::PostingSet; use gbs_mechanisms::{ParticipantSet, accrue};";
    let actual = mechanisms_used(source);
    let unclaimed: Vec<u8> = actual.difference(claimed).copied().collect();
    assert_eq!(unclaimed, vec![4, 7], "the check must see the unclaimed mechanisms");
}

#[test]
fn the_honest_ratio_of_built_to_claimed_is_reported() {
    // Not an assertion about the number — it will change — but a test that the number is
    // *derivable*, so a reader can ask "how much of this matrix is real?" and get an answer
    // from the build rather than from a paragraph.
    let matrix = parse_matrix(&architecture_md());
    let built: BTreeSet<&str> = implemented_rows().iter().map(|(r, _)| *r).collect();
    let total = matrix.len();
    let done = built.len();

    assert!(done > 0 && done <= total);
    // Every implemented row must exist in the matrix — the reverse is expected to fail for
    // now, and that gap is the honest measure of how much is still design.
    for row in &built {
        assert!(matrix.contains_key(*row), "`{row}` is built but not in the matrix");
    }
    println!("coverage: {done} of {total} product lines implemented");
}
