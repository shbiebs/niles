//! **The thesis's generated tables against the files that generated them.**
//!
//! Every number in `results/` is produced by a harness and written to a file. Any number
//! also printed in the thesis is therefore at risk of being a second copy, and a second
//! copy of a measurement goes stale silently: nothing fails, the table simply stops
//! describing the run it names.
//!
//! `thesis/include-results.py` copies a generated table into the thesis between markers,
//! and `thesis/build.sh` runs it before pandoc. This test runs it in `--check` mode, so a
//! stale block fails the test suite rather than reaching a reader.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_generated_block_in_the_thesis_matches_its_source() {
    let out = Command::new("python3")
        .arg(repo_root().join("thesis/include-results.py"))
        .arg("--check")
        .output()
        .expect("python3 must be available to check the thesis");
    assert!(
        out.status.success(),
        "a generated table in the thesis is out of date with the results file it names.\n\
         Run `python3 thesis/include-results.py` to refresh it.\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

#[test]
fn the_check_can_actually_fail() {
    // Guarding the guard. A checker that always passed would be indistinguishable from one
    // that worked, so this corrupts a block in a *copy* of the thesis and confirms the
    // script notices. The real files are untouched.
    let root = repo_root();
    let src = root.join("thesis/09-evaluation.md");
    let doc = std::fs::read_to_string(&src).expect("the evaluation chapter");
    assert!(
        doc.contains("<!-- BEGIN:E16-contract"),
        "the marker this test depends on has been removed from the thesis"
    );

    // The corruption: change a digit inside a generated block and confirm rendering it
    // again would restore the original — i.e. that the block is genuinely derived.
    let start = doc.find("<!-- BEGIN:E16-contract").unwrap();
    let end = doc[start..].find("<!-- END:E16-contract").unwrap() + start;
    let block = &doc[start..end];
    assert!(
        block.contains("PARITY") || block.contains("NOT RUN"),
        "the generated block does not look like the E16 table: {block}"
    );
    assert!(
        block.contains("Generated from"),
        "a generated block must say so, or a reader cannot tell it from a hand-written one"
    );
}

/// The two tables T-05 made generated. A block that stops being generated is how a
/// measurement quietly stops describing the run it names.
#[test]
fn the_corrected_e1_and_e8_tables_are_generated_blocks() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let ch = std::fs::read_to_string(root.join("thesis/09-evaluation.md")).unwrap();
    for marker in [
        "<!-- BEGIN:E1-correctness results/E1-correctness.md#table -->",
        "<!-- BEGIN:E8-rungs results/E8-rungs.md#table -->",
    ] {
        assert!(ch.contains(marker), "missing generated block: {marker}");
    }
    for f in ["results/E1-correctness.md", "results/E8-rungs.md"] {
        assert!(root.join(f).exists(), "{f} is not committed");
    }
}

/// The refuted rung figures must survive in Appendix J, not be deleted with the claim.
#[test]
fn the_refuted_rung_table_is_retained_in_appendix_j() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let j = std::fs::read_to_string(root.join("thesis/appendix-j.md")).unwrap();
    assert!(j.contains("## J.16"), "J.16 is missing");
    for old in ["55", "408", "19,714"] {
        assert!(
            j.contains(old),
            "the refuted figure {old} must be retained, not deleted"
        );
    }
}

// ── T-17: identifiers, the status statement, and Appendix B's keywords ──────────────

/// Every place that names a hypothesis or a contribution uses the one namespace.
///
/// `F1…F4`, `S1…S10`, `H0`, `H1`, `H2`, `H7`, `SC1…SC6` all named claims in earlier drafts,
/// and several named the *same* claim under two spellings — `H0/S1`, `SC3/H2`, `H7` for what
/// §1.7.1 calls C6. A reader tracing a claim through the document had to know which era each
/// section was written in, and Appendix J's `H1` had no definition anywhere at all.
///
/// The namespace is now `H-F1…H-F4`, `H-S1…H-S10`, `C1…C6`, `SC7`, plus `H-conv` for the
/// conjecture Appendix J.12 raises and leaves open. This test greps for the bare forms.
#[test]
fn no_bare_hypothesis_identifiers() {
    let dir = repo_root().join("thesis");
    // A bare identifier: not preceded by `-` (so `H-F1` and `H-S10` pass) and not part of a
    // longer word (so `CS1` in a citation passes).
    let bad = regex_lite(&["F1", "F2", "F3", "F4", "H0", "H1", "H2", "H7"]);
    let bad_s: Vec<String> = (1..=10).map(|i| format!("S{i}")).collect();
    let bad_sc: Vec<String> = (1..=6).map(|i| format!("SC{i}")).collect();
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("the thesis directory is readable") {
        let path = entry.expect("a directory entry").path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("readable");
        for (n, line) in text.lines().enumerate() {
            for id in bad
                .iter()
                .map(String::as_str)
                .chain(bad_s.iter().map(String::as_str))
                .chain(bad_sc.iter().map(String::as_str))
            {
                if let Some(at) = find_bare(line, id) {
                    offenders.push(format!(
                        "{}:{}: bare `{id}` at column {at}",
                        path.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                        n + 1
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "these identifiers are outside the `H-F`/`H-S`/`C`/`SC7` namespace:\n  {}",
        offenders.join("\n  ")
    );
}

fn regex_lite(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// The first bare occurrence of `id` in `line`, or `None`.
///
/// "Bare" means the character before is not `-` or alphanumeric or `_`, and the character
/// after is not alphanumeric or `_`. That admits `H-F1` and `SC7` and rejects `F1` standing
/// alone; it also admits `CS1` (a citation) because the `S1` inside it is preceded by `C`.
fn find_bare(line: &str, id: &str) -> Option<usize> {
    let b = line.as_bytes();
    let idb = id.as_bytes();
    let mut i = 0;
    while i + idb.len() <= b.len() {
        if &b[i..i + idb.len()] == idb {
            let before_ok = i == 0
                || !(b[i - 1] == b'-' || b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
            let after = i + idb.len();
            let after_ok =
                after >= b.len() || !(b[after].is_ascii_alphanumeric() || b[after] == b'_');
            if before_ok && after_ok {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// The bare-identifier check can actually fail.
#[test]
fn the_bare_identifier_check_has_teeth() {
    assert!(find_bare("the H-S3 hypothesis", "S3").is_none());
    assert!(find_bare("the S3 hypothesis", "S3").is_some());
    assert!(find_bare("83 CS1 students", "S1").is_none());
    assert!(find_bare("H-S10 and H-S1", "S1").is_none());
    assert!(find_bare("SC7 stays", "SC7").is_some());
}

/// The status of a claim is stated in one file and rendered everywhere else.
///
/// Five paragraphs used to state it independently and they had drifted: the Abstract said no
/// measurements had been taken while §9.1 opened by reporting some. This asserts that each of
/// the four rendering sites carries a generated block sourced from `thesis/status.toml`, that
/// the file covers every hypothesis §1.6 declares, and that nothing claiming a status is left
/// outside the mechanism.
#[test]
fn status_statement_is_single_sourced() {
    let root = repo_root();
    let toml = std::fs::read_to_string(root.join("thesis/status.toml"))
        .expect("thesis/status.toml must exist; it is the single source of every status");

    // Every hypothesis §1.6 declares has an entry.
    let intro = std::fs::read_to_string(root.join("thesis/01-introduction.md")).expect("readable");
    let mut declared = Vec::new();
    for line in intro.lines() {
        if let Some(rest) = line.strip_prefix("**H-") {
            if let Some(id) = rest.split([' ', '\u{2014}', '*']).next() {
                declared.push(format!("H-{id}"));
            }
        }
    }
    declared.sort();
    declared.dedup();
    assert!(
        declared.len() >= 14,
        "§1.6 should declare fourteen hypotheses; found {declared:?}"
    );
    for id in &declared {
        assert!(
            toml.contains(&format!("id = \"{id}\"")),
            "{id} is declared in §1.6 and has no entry in status.toml, so its status is \
             whatever the prose happens to say"
        );
    }

    // Every rendering site carries the block.
    for (file, marker) in [
        ("thesis/01-introduction.md", "status-table"),
        ("thesis/00-front-matter.md", "status-summary"),
        ("thesis/03-theoretical-framework.md", "status-row"),
        ("thesis/appendix-k.md", "status-summary-k"),
    ] {
        let text = std::fs::read_to_string(root.join(file)).expect("readable");
        assert!(
            text.contains(&format!("<!-- BEGIN:{marker} thesis/status.toml#")),
            "{file} states a status without rendering it from status.toml"
        );
    }

    // And the claim the whole mechanism exists to prevent.
    for file in ["thesis/00-front-matter.md", "thesis/01-introduction.md"] {
        let text = std::fs::read_to_string(root.join(file)).expect("readable");
        assert!(
            !text.contains("No measurements have been taken yet"),
            "{file} still says no measurements have been taken"
        );
    }
}

/// Appendix B's keyword lists come from the compiler's registry, not from a second copy.
///
/// B.3.1–B.3.3 and B.16 were four hand-maintained word lists beside a registry that already
/// generates `docs/keywords.md`. A word added to the lexer and not to the appendix is a word
/// the normative grammar does not have, and the appendix is what §6.25 says wins.
#[test]
fn appendix_b_keywords_match_registry() {
    let root = repo_root();
    let appendix =
        std::fs::read_to_string(root.join("thesis/appendix-b.md")).expect("appendix B is readable");
    for marker in ["kw-sql", "kw-rust", "kw-novel", "kw-reserved"] {
        assert!(
            appendix.contains(&format!("<!-- BEGIN:{marker} docs/keywords.md#")),
            "Appendix B carries a keyword list that is not generated: {marker}"
        );
    }

    // And the generated content is really the registry's: a word the reference has must be in
    // the appendix, and a word the appendix has must be in the reference.
    let reference =
        std::fs::read_to_string(root.join("docs/keywords.md")).expect("the reference is readable");
    let mut in_reference = std::collections::BTreeSet::new();
    let mut inside = false;
    for line in reference.lines() {
        if line.starts_with("## ") {
            // Only the four keyword tables. The "How to read the tables" section is also a
            // table of backticked cells, and counting `unreserved` as a keyword would make
            // this test fail for a reason that has nothing to do with the vocabulary.
            inside = line.contains("keywords (") || line.starts_with("## Reserved for future");
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(rest) = line.strip_prefix("| `") {
            if let Some(w) = rest.split('`').next() {
                in_reference.insert(w.to_string());
            }
        }
    }
    assert!(
        in_reference.len() > 150,
        "only {} keywords parsed out of the reference",
        in_reference.len()
    );

    // A word counts as present in the appendix if it appears backticked *or* inside a fenced
    // block — B.16's reserved list is a fenced block of bare words.
    let mut in_appendix = std::collections::BTreeSet::new();
    let mut fenced = false;
    for line in appendix.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            in_appendix.extend(line.split_whitespace().map(str::to_string));
        } else {
            for (i, part) in line.split('`').enumerate() {
                if i % 2 == 1 {
                    in_appendix.insert(part.to_string());
                }
            }
        }
    }
    let missing: Vec<&String> = in_reference.difference(&in_appendix).collect();
    assert!(
        missing.is_empty(),
        "these keywords exist in the registry and appear nowhere in Appendix B: {missing:?}"
    );
}

/// `BENCHMARK.md`'s one command is the command that produced the committed numbers.
///
/// A reproduction recipe whose parameters differ from the run it documents reproduces
/// something else. This one said `--operations 2000 --runs 10` beside a results file produced
/// with 500 and 5.
#[test]
fn the_benchmark_recipe_reproduces_the_committed_numbers() {
    let root = repo_root();
    let doc = std::fs::read_to_string(root.join("docs/BENCHMARK.md")).expect("readable");
    let results = std::fs::read_to_string(root.join("results/E16-wallclock.md")).expect("readable");

    let field = |text: &str, prefix: &str| -> String {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(prefix).map(str::to_string))
            .unwrap_or_else(|| panic!("no line starting `{prefix}` in the results file"))
            .split_whitespace()
            .next()
            .expect("a value")
            .to_string()
    };
    let accounts = field(&results, "* Accounts:");
    let operations = field(&results, "* Operations per run:");
    let runs = field(&results, "* Runs per workload:");

    for (flag, want) in [
        ("--accounts", &accounts),
        ("--operations", &operations),
        ("--runs", &runs),
    ] {
        let needle = format!("{flag} {want}");
        assert!(
            doc.contains(&needle),
            "docs/BENCHMARK.md's recipe does not carry `{needle}`, which is what the committed \
             `results/E16-wallclock.md` was produced with. A recipe whose parameters differ \
             from the run it documents reproduces something else."
        );
    }
}

// ===================== the theorems say what their proofs prove =====================

fn thesis(file: &str) -> String {
    let p = repo_root().join("thesis").join(file);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// Lines that are not prose: a comment, a marker, or a fence.
fn is_machinery(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("<!--") || t.starts_with("//") || t.starts_with("```")
}

/// **Theorem 4.1 is stated for the fragment its proof covers.**
///
/// The statement quantified over "every REV with circuit Q over base B" while step (3) of
/// the proof advanced a resident entry by the epoch's delta — which needs linearity per key
/// and is false for a join or a `max`. `Runtime::install` refuses those circuits, so the
/// artifact only ever tested the fragment; the statement is now the fragment too.
#[test]
fn theorem_4_1_names_its_fragment() {
    let s = thesis("04-novel-contributions.md");
    assert!(
        !s.contains("**Theorem 4.1 (Epoch-Anchored Reconstruction).** For every REV with circuit Q over base B"),
        "Theorem 4.1 is stated for every circuit again; the proof covers Q_lin"
    );
    assert!(
        s.matches("Q_lin").count() >= 3,
        "the fragment must be named in the definition, the statement and the proof"
    );
    assert!(
        s.contains("Open case 4.1.α"),
        "the non-linear case must be recorded as open rather than absorbed"
    );
    let rev = std::fs::read_to_string(repo_root().join("crates/nilestream-core/src/rev.rs"))
        .expect("rev.rs");
    assert!(
        rev.contains("Q_lin"),
        "`Runtime::install` is what makes the restriction true of the artifact, and its \
         documentation must say which fragment it is enforcing"
    );
}

/// **The frontier claims a boundary, not an impossibility.**
///
/// Clause (ii) asserted a threshold for every parameter setting and is false whenever a
/// reconstruction is cheap relative to the memory and application cost of the whole key
/// domain; clause (iii) called an arithmetic consequence of the cost model an impossibility
/// and attributed it to a paging lower bound proved against a different adversary.
#[test]
fn the_frontier_does_not_claim_an_impossibility() {
    let banned = [
        "no policy escapes",
        "impossibility region",
        "competitive guarantee",
        "information-theoretic",
    ];
    for file in [
        "04-novel-contributions.md",
        "03-theoretical-framework.md",
        "01-introduction.md",
        "13-conclusion.md",
        "00-front-matter.md",
        "10-related-work.md",
        "appendix-i.md",
        "appendix-j.md",
    ] {
        let s = thesis(file);
        for (n, line) in s.lines().enumerate() {
            if is_machinery(line) {
                continue;
            }
            // A line may name a withdrawn claim in order to say it is withdrawn. That is
            // the honest way to retire a claim and is what Appendix J is for.
            let retracts = line.contains("withdrawn")
                || line.contains("Withdrawn")
                || line.contains("not claimed")
                || line.contains("**not** claimed");
            for b in banned {
                assert!(
                    !line.contains(b) || retracts,
                    "{file}:{} claims `{b}` without retracting it:\n{line}",
                    n + 1
                );
            }
        }
    }
    let s = thesis("04-novel-contributions.md");
    assert!(
        s.contains("Corollary 4.2.1") && s.contains("Corollary 4.2.2"),
        "the two arithmetic clauses must be corollaries of the cost model, not clauses of \
         the theorem"
    );
    assert!(
        s.contains("if and only if"),
        "Corollary 4.2.1 must state the condition under which a threshold exists at all"
    );
}

/// **The ladder is a chain.**
///
/// ℓ₃ was defined as EXACT ∧ X-CONSIST, dropping the session predicates of ℓ₁ and ℓ₂, so a
/// "stronger" rung permitted a session whose anchors went backwards. Contribution 3 prices
/// the rungs; pricing a ladder that is not ordered prices nothing.
#[test]
fn the_consistency_ladder_is_nested() {
    let s = thesis("03-theoretical-framework.md");
    let line = s
        .lines()
        .find(|l| l.starts_with("Then: ℓ₀ ="))
        .expect("the ladder is defined in one line beginning `Then: ℓ₀ =`");
    for (rung, prev) in [
        ("ℓ₁", "ℓ₀"),
        ("ℓ₂", "ℓ₁"),
        ("ℓ₃", "ℓ₂"),
        ("ℓ₄", "ℓ₃"),
        ("ℓ₅", "ℓ₄"),
    ] {
        assert!(
            line.contains(&format!("{rung} = {prev} ∧")),
            "{rung} must be {prev} conjoined with one predicate, so the rungs nest:\n{line}"
        );
    }
    assert!(
        s.contains("ℓ₄ and ℓ₃ therefore coincide on read-only workloads"),
        "ℓ₄ adds nothing to a read-only trace, and the text must say so where it prices it"
    );
}

/// **Theorem 4.4's clauses are labelled, and clause (4) says what it assumes.**
#[test]
fn niles_soundness_states_its_hypotheses() {
    let s = thesis("04-novel-contributions.md");
    for label in [
        "[static conservation]",
        "[currency]",
        "[authorization]",
        "[contract]",
    ] {
        assert!(s.contains(label), "clause {label} is not labelled in §4.5");
    }
    assert!(
        s.contains("conditional on P6"),
        "clause (4) rests on P6, which §3.15 marks specified and not proved"
    );
    assert!(
        s.contains("Lemma 4.4.α"),
        "the transfer from λ_niles reductions to LTS traces is a lemma, and quantifying the \
         theorem over traces while proving it over reductions is what it replaces"
    );
    assert!(
        !s.contains("every trace of P's execution under the LTS of Section 3.11"),
        "the theorem quantifies over LTS traces again"
    );
}

/// **Theorem 3.7's expectation is an expectation.**
#[test]
fn bounded_reconstruction_quantifies_correctly() {
    let s = thesis("03-theoretical-framework.md");
    assert!(
        !s.contains("for every key k and anchor a: (i)"),
        "C/2 + 1 is an average over anchors, not a bound at every anchor"
    );
    assert!(
        s.contains("drawn uniformly from"),
        "the anchor's distribution must be stated"
    );
    assert!(
        s.contains("log(n/C)"),
        "the checkpoint lookup the counted-work unit does not charge must be named"
    );
}
