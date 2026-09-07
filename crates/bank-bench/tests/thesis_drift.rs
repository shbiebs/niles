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
        // **Not the bibliography.** A reference carries titles and product names that this
        // test's namespace has no claim over, and it did real damage: `[90] F1 Lightning:
        // HTAP as a service` was flagged here as a bare `F1` and *fixed* by prefixing it,
        // so the thesis cited a paper called "H-F1 Lightning" — a naming convention
        // rewriting a book title, which nothing downstream could have caught because the
        // citation apparatus was not checked at all until this cycle.
        if path.file_name().and_then(|s| s.to_str()) == Some("references.md") {
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
        declared.len() >= 13,
        "§1.6 should declare thirteen hypotheses; found {declared:?}"
    );
    for id in &declared {
        // **A withdrawn hypothesis keeps its paragraph and loses its entry.** H-S9 was
        // withdrawn when the crate that would have been its instrument turned out to be
        // three `pub mod` lines and was deleted: with no lineage mode there is no
        // independent variable, so there is nothing to hold constant and nothing to vary.
        // Deleting the paragraph would hide that a hypothesis was abandoned, which is the
        // opposite of what this file is for; the paragraph says *Withdrawn* and gives the
        // reason, and this check requires exactly that of any id with no entry.
        if !toml.contains(&format!("id = \"{id}\"")) {
            // The paragraph runs from the id's declaration to the next blank line.
            let at = intro.find(&format!("**{id} ")).unwrap_or(0);
            let para: String = intro[at..].lines().take_while(|l| !l.is_empty()).collect();
            assert!(
                para.contains("Withdrawn") || para.contains("withdrawn"),
                "{id} is declared in §1.6, has no entry in status.toml, and does not say it \
                 is withdrawn — so its status is whatever the prose happens to say"
            );
            continue;
        }
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
    // **The seeding is part of the recipe.** It was not, and that is how the two targets came
    // to hold 20,000 and 40,000 rows: `--nls-rounds` defaulted to 2 on one side and did not
    // exist on the other, appeared in neither this recipe nor the results header, and so
    // nothing could catch it. A parameter that changes what is measured and is not in the
    // command that reproduces it is a parameter nobody can check.
    let rounds = field(&results, "* Rounds per account:");

    for (flag, want) in [
        ("--accounts", &accounts),
        ("--operations", &operations),
        ("--runs", &runs),
        ("--rounds", &rounds),
    ] {
        let needle = format!("{flag} {want}");
        assert!(
            doc.contains(&needle),
            "docs/BENCHMARK.md's recipe does not carry `{needle}`, which is what the committed \
             `results/E16-wallclock.md` was produced with. A recipe whose parameters differ \
             from the run it documents reproduces something else."
        );
    }

    // The two targets held the same base, and the header says how big it was.
    let base = field(&results, "* Base rows per target:");
    let expected: i64 =
        accounts.parse::<i64>().expect("accounts") * 2 * rounds.parse::<i64>().expect("rounds");
    assert_eq!(
        base.parse::<i64>().ok(),
        Some(expected),
        "the results header says the base is {base} rows, but {accounts} accounts x {rounds} \
         rounds x 2 legs is {expected}. The header is what a reader checks the comparison \
         against; if it is not arithmetic, it is decoration."
    );
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

/// **Every identifier Appendix D names exists in the workspace.**
///
/// D.2 and D.3 listed an engine API in the present tense: `LedgerHandle::append_batch`,
/// `ReadModel::serve`, `StorageTier`, a `Rev::explain` returning a `Lineage`. None of them
/// were in `crates/`. An appendix is the last place a reader looks before believing
/// something is built, so this checks the backticked `Type::method` and `Type` names in it
/// against the source.
///
/// The generated blocks are the fix; this is the guard on the hand-written prose around
/// them, which is where the fictional names were.
#[test]
fn appendix_d_names_only_things_that_exist() {
    let appendix = thesis("appendix-d.md");
    let mut sources = String::new();
    let crates_dir = repo_root().join("crates");
    let mut stack = vec![crates_dir];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().and_then(|s| s.to_str()) == Some("target") {
                    continue;
                }
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                sources.push_str(&std::fs::read_to_string(&p).unwrap_or_default());
            }
        }
    }

    let mut missing = Vec::new();
    for token in appendix.split('`').skip(1).step_by(2) {
        // `Type::method` or `Type::CONST`: the shape the fictional API was written in.
        let Some((ty, item)) = token.split_once("::") else {
            continue;
        };
        if !ty.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            || !ty.chars().all(|c| c.is_ascii_alphanumeric())
            || item.is_empty()
            || !item.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            continue;
        }
        // The type must exist and the member must appear somewhere in the crates. A looser
        // check than "this method is on that type", and enough to have caught every one of
        // the names that were not there at all.
        let ty_present = sources.contains(&format!("struct {ty}"))
            || sources.contains(&format!("enum {ty}"))
            || sources.contains(&format!("trait {ty}"));
        if !ty_present || !sources.contains(item) {
            missing.push(token.to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "Appendix D names {} item(s) that are not in the workspace: {missing:?}\n\
         An appendix that describes an API nobody can call is a design note in a reference \
         manual's typography.",
        missing.len()
    );
}

/// **No `unsafe`, asserted rather than observed.**
///
/// §3.17, §5.8 and §9.10 describe a three-layer verification programme of which one layer
/// exists: the type-level one, discharged by the compiler. "The prototype contains no
/// `unsafe` blocks" was a sentence about a state of the tree at some past moment, which is
/// the kind of claim one line makes false. It is the only part of that programme this thesis
/// can currently stand behind, so it is the part that gets a test.
/// **No `unsafe` anywhere in the repository, with one named exception.**
///
/// §9.10 claims the workspace contains no `unsafe` block. This scans the **whole
/// repository** rather than only `crates/`, which is strictly stronger and is what makes the
/// one exception reviewable instead of hidden behind a scope: a file outside `crates/` could
/// previously have used `unsafe` and nothing would have said.
///
/// The exception is `tools/memprobe`, the E18 memory instrument. `GlobalAlloc` cannot be
/// implemented without `unsafe`, and the alternative to this exception was to weaken §9.10's
/// claim to "no unsafe except…", which trades a guarantee about the system for the
/// convenience of a tool that measures it. The package is outside the workspace, is built by
/// nothing that ships, and is allowed here by *name* — so a second file wanting the same
/// licence has to be added to this list by someone who has thought about it.
#[test]
fn the_only_unsafe_in_the_repository_is_the_measurement_tool() {
    /// Files permitted to contain the keyword, with why.
    const ALLOWED: &[(&str, &str)] = &[(
        "tools/memprobe/src/alloc.rs",
        "the counting global allocator: `GlobalAlloc` has no safe implementation, and this \
         package is outside the workspace and linked into nothing that ships",
    )];

    let root = repo_root();
    let mut offenders = Vec::new();
    let mut allowed_seen = 0usize;
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(name, "target" | ".git" | "node_modules") {
                    continue;
                }
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            // This file names the keyword in order to look for it, and `keywords.rs`
            // registers it as reserved-and-refused in Niles. Both are mentions, not uses.
            if p.ends_with("tests/thesis_drift.rs") || p.ends_with("src/keywords.rs") {
                continue;
            }
            let rel = p
                .strip_prefix(&root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            let keyword = "un".to_string() + "safe";
            let uses = text.lines().enumerate().filter(|(_, line)| {
                let t = line.trim();
                t.starts_with(&format!("{keyword} ")) || t.contains(&format!(" {keyword} {{"))
            });
            match ALLOWED.iter().find(|(f, _)| *f == rel) {
                Some(_) => {
                    if uses.count() > 0 {
                        allowed_seen += 1;
                    }
                }
                None => {
                    for (n, line) in uses {
                        offenders.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                    }
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "§9.10 says the repository has no `unsafe` block outside the measurement tool, and \
         that this is the one layer of the memory-model programme that is discharged. \
         Found:\n  {}",
        offenders.join("\n  ")
    );
    // An exception that stopped being used should stop being granted: a licence nobody needs
    // is a licence the next file inherits without argument.
    assert_eq!(
        allowed_seen,
        ALLOWED.len(),
        "an entry in the allow-list no longer contains the keyword it was granted for; \
         remove it rather than leaving a standing exception"
    );
}

/// **Every document in `docs/` is reachable from the README.**
///
/// `docs/VALIDATION-RUN.md` — the validation protocol of an earlier cycle, run, with the two
/// items it did not meet — was linked from nothing and named by nothing. A document nobody
/// can find is a document nobody reads, and its two unmet items were as good as unrecorded.
#[test]
fn no_document_is_reachable_from_nothing() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).expect("README.md");
    let mut orphans = Vec::new();
    for e in std::fs::read_dir(repo_root().join("docs"))
        .expect("docs/")
        .flatten()
    {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if !readme.contains(&format!("docs/{name}")) {
            orphans.push(name);
        }
    }
    orphans.sort();
    assert!(
        orphans.is_empty(),
        "{orphans:?} are in `docs/` and named nowhere in the README. Add a row to the \
         document table, or delete the file."
    );
}

/// **A phrase that names an unbuilt thing must be near a word that says so.**
///
/// The failure this catches is the one that recurs: a chapter written when something was
/// planned, left in the present tense after the plan changed. Appendix I described an
/// optimizer as implemented; §7 described a MySQL listener and storage tiers as built; §13
/// said all four contributions were proved *and measured*. Each was true of an intention.
///
/// The check is deliberately crude — a phrase, and a window of 240 characters around it that
/// must contain one of a small set of hedges — because the alternative is a reviewer
/// noticing, and a reviewer noticing is what produced this list.
#[test]
fn nothing_unbuilt_is_described_in_the_present_tense() {
    // Each phrase names something that does not exist in this repository at this commit.
    let unbuilt = [
        "MySQL listener",
        "lineage mode",
        "storage tier",
        "adaptive optimizer",
    ];
    // `Elle` and `model checking` are *not* in that list, deliberately. Both are named
    // legitimately all over the thesis — as a tool someone else built, as a methodology, as
    // a thing §12 says should be done — and a check that flagged every mention would be
    // switched off within a week. What was wrong was one sentence in §3.11 claiming the
    // programme *adds* an Elle-style checker; that sentence is gone and the assertion below
    // is what keeps it gone.
    // A sentence may name one of them if it also says what it is.
    let hedges = [
        "absent",
        "would",
        "no MySQL listener",
        "planned",
        "Planned",
        "not built",
        "unbuilt",
        "withdrawn",
        "Withdrawn",
        "future work",
        "does not exist",
        "specified and not built",
        "Specification",
        "specification",
        "not run",
        "has not been done",
        "no such",
        "nothing implements",
        "would be",
        "not measured",
        "deleted",
    ];
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(repo_root().join("thesis")).expect("thesis/") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        // Appendix J records positions as they were held, marked as withdrawn where they
        // were; the bibliography carries titles.
        if name == "references.md" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("readable");
        for phrase in unbuilt {
            let mut from = 0;
            while let Some(i) = text[from..].find(phrase) {
                let at = from + i;
                let lo = at.saturating_sub(240);
                let hi = (at + phrase.len() + 240).min(text.len());
                let window = &text[text.floor_char_boundary(lo)..text.floor_char_boundary(hi)];
                if !hedges.iter().any(|h| window.contains(h)) {
                    let line = text[..at].matches('\n').count() + 1;
                    offenders.push(format!(
                        "{name}:{line}: `{phrase}` with nothing to say it is not built"
                    ));
                }
                from = at + phrase.len();
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{} present-tense claim(s) about unbuilt components:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );

    // The specific sentence, rather than the word.
    for file in [
        "03-theoretical-framework.md",
        "05-research-design.md",
        "09-evaluation.md",
    ] {
        let t = thesis(file);
        assert!(
            !t.contains("adds black-box anomaly inference"),
            "{file} says the correctness programme *adds* an Elle-style checker. It does \
             not have one."
        );
    }
}

/// **The README's test count is the source's test count.**
///
/// It was neither. The status line said 201 and the reproduce block said 430, in the same
/// file, while the workspace held 736 — two figures that disagreed with each other and with
/// the artifact, in a repository whose stated discipline is that a document which drifts from
/// the code fails a build rather than being noticed in review.
///
/// Counted as `#[test]` functions in the source rather than as lines from `cargo test`,
/// deliberately. The runner prints 821 because `nilestream-server` is compiled as both a
/// library and a binary and its module tests are executed under each, so the runner's figure
/// counts several dozen tests twice. A reader who wants to know how much is tested wants the
/// number of distinct tests, and a test cannot shell out to the test runner anyway.
#[test]
fn the_readme_test_count_is_the_number_of_tests_that_exist() {
    let root = repo_root();
    let mut found = 0usize;
    let mut stack = vec![root.join("crates")];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let text = std::fs::read_to_string(&p).unwrap_or_default();
                // Whole lines only. Counting substrings would count this test's own doc
                // comment and the pattern it searches for, which it did: the first run
                // reported three tests that do not exist.
                found += text
                    .lines()
                    .filter(|l| l.trim() == concat!("#[", "test]"))
                    .count();
            }
        }
    }

    let readme = std::fs::read_to_string(root.join("README.md")).expect("README.md");
    let parts: Vec<&str> = readme.split("test function").collect();
    let claimed: Vec<usize> = parts[..parts.len().saturating_sub(1)]
        .iter()
        .filter_map(|before| {
            let digits: String = before
                .chars()
                .rev()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            digits.chars().rev().collect::<String>().parse().ok()
        })
        .collect();

    assert!(
        !claimed.is_empty(),
        "the README no longer states a test count; it stated two, and both were wrong"
    );
    for c in &claimed {
        assert_eq!(
            *c, found,
            "the README claims {c} test functions and the workspace has {found}. Update the \
             README — both places it says so — rather than the assertion."
        );
    }
}

/// **The audit preflight may not carry a figure about the tree it is run against.**
///
/// The cycle-8 preflight told its auditor the tree pinned `channel = "stable"` (it pinned
/// `1.95.0`), that the workspace held 801 test functions (843), and that the heads to expect
/// were the previous cycle's. Every one of those was true when it was written and false when
/// it was read — the same defect class the audit it opens exists to find, in the instrument
/// that opens it.
///
/// A number about the tree belongs in the preflight's *output*, measured when it runs. This
/// asserts the script contains no literal test count; the pin and the heads are checked by
/// reading, since they are no longer written down at all.
#[test]
fn the_audit_preflight_measures_the_tree_rather_than_describing_it() {
    let src = std::fs::read_to_string(repo_root().join("docs/audit/cycle-8/preflight.sh"))
        .expect("the cycle-8 preflight");
    for line in src.lines() {
        let l = line.trim_start();
        // Only the lines that *print*: a comment may quote a historical figure, and the
        // shell arithmetic that counts may name a bound.
        if !l.starts_with("say ") {
            continue;
        }
        assert!(
            !l.contains("test fns") && !l.contains("test functions"),
            "the preflight prints a test count it did not measure:\n  {line}\n\
             count it from the tree at run time — a figure typed into an audit instrument is \
             stale the first time the instrument is reused"
        );
    }
    assert!(
        src.contains("rust-toolchain.toml"),
        "the preflight must read the pin out of rust-toolchain.toml rather than describe it"
    );
    assert!(
        !src.contains("channel = \\\"stable\\\"`"),
        "the preflight still describes the pin instead of reading it"
    );
}
