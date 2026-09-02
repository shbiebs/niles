//! **E18 — what the currency-row solver actually decides.**
//!
//! # The question, and why nothing could answer it before
//!
//! Contribution 4's soundness theorem says a well-typed Niles program conserves money. The
//! analysis behind it has four verdicts: `Conserves`, `Violates`, `MayViolate`, and
//! `Undecided`. The last is honest and — the module's own doc comment is careful about this —
//! *forced*: with affine equality guards in the language, deciding whether an affine relation
//! holds at a program point is undecidable, by Müller-Olm and Seidl's reduction from Post's
//! Correspondence Problem.
//!
//! The architecture review's objection was not that `Undecided` exists. It was that **nobody
//! had measured how often it fires on code somebody would write**. A soundness theorem whose
//! analysis is undecided on an ordinary limit check is true and useless, and the thesis's
//! "11 of 12 defect classes" claim would be about a fragment nobody lives in.
//!
//! So: forty-five functions, forty of them correct and five deliberately defective, run
//! through the real front end — parse, resolve, type-check — one at a time, with the verdicts
//! counted.
//!
//! # Why one function at a time
//!
//! A whole file gives one report, and a rate computed from it would be an average over
//! functions of very different shapes. Running each function against the same schema attributes
//! a verdict to a *construct*, which is what makes the result actionable: "guarded transfers
//! are undecided" is a finding, and "the corpus is 12% undecided" is a number.

use niles_lang::typecheck::Report;

/// One function's verdict counts.
#[derive(Debug, Clone, PartialEq)]
struct Outcome {
    name: String,
    proved: usize,
    undecided: usize,
    may_violate: usize,
    violates: usize,
    /// Diagnostics the front end emitted at error severity. A function that does not compile
    /// is not a measurement of the solver, so this is checked rather than ignored.
    errors: usize,
}

impl Outcome {
    /// The verdict this function is filed under.
    ///
    /// Ordered by severity rather than by count: a function with one must-violation and nine
    /// proved rows is a defective function, and averaging would hide it.
    fn verdict(&self) -> &'static str {
        if self.violates > 0 {
            "Refuted"
        } else if self.may_violate > 0 {
            "MayViolate"
        } else if self.undecided > 0 {
            "Undecided"
        } else if self.proved > 0 {
            "Proved"
        } else {
            "NoObligation"
        }
    }
}

fn corpus_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/solver_corpus")
}

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

/// Split a corpus file into individual functions, keeping each one's leading comment.
///
/// Textual rather than syntactic, and that is a deliberate limitation: a splitter that parsed
/// would need the parser to be right about the thing being measured. A blank-line-separated
/// `fn` at column zero is unambiguous in this corpus and the test below asserts the count, so
/// a splitter that silently lost a function fails rather than reports a better rate.
fn split_functions(source: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in source.lines() {
        if line.starts_with("fn ") && !current.is_empty() {
            let text = current.join("\n");
            if text.contains("\nfn ") || text.starts_with("fn ") {
                out.push(text);
            }
            current.clear();
        }
        current.push(line);
    }
    if !current.is_empty() {
        out.push(current.join("\n"));
    }

    out.into_iter()
        .filter(|t| t.contains("fn "))
        .map(|text| {
            let name = text
                .lines()
                .find(|l| l.starts_with("fn "))
                .and_then(|l| l.strip_prefix("fn "))
                .and_then(|l| l.split('(').next())
                .unwrap_or("<unnamed>")
                .trim()
                .to_string();
            (name, text)
        })
        .collect()
}

/// Run one function against the schema and report what the solver concluded.
fn check_one(preamble: &str, name: &str, body: &str) -> Outcome {
    let program = format!("{preamble}\n\n{body}\n");
    let (prog, mut d) = niles_lang::parser::parse_program(&program);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (report, td): (Report, _) = niles_lang::typecheck::check_program(&prog, &cat);
    d.extend(td);

    Outcome {
        name: name.to_string(),
        proved: report.conservation_proved,
        undecided: report.undecided,
        may_violate: report.may_violate,
        violates: report.violates,
        errors: if d.has_errors() { 1 } else { 0 },
    }
}

/// Split the interprocedural corpus into cases at its `// --- case:` markers.
///
/// A case is several functions checked **together**, which is the whole point: the question
/// is what the solver concludes about the *caller*, and a splitter that separated the caller
/// from its callee would be measuring the state of affairs this group was written to end.
fn split_cases(source: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in source.lines() {
        if let Some(name) = line.trim().strip_prefix("// --- case:") {
            out.push((name.trim().to_string(), String::new()));
        } else if let Some(last) = out.last_mut() {
            last.1.push_str(line);
            last.1.push('\n');
        }
    }
    out
}

fn run_interprocedural() -> Vec<Outcome> {
    let preamble = std::fs::read_to_string(corpus_dir().join("preamble.niles")).expect("preamble");
    let src = std::fs::read_to_string(corpus_dir().join("interprocedural.niles"))
        .expect("interprocedural corpus");
    split_cases(&src)
        .into_iter()
        .map(|(name, body)| check_one(&preamble, &name, &body))
        .collect()
}

fn run_corpus() -> (Vec<Outcome>, Vec<Outcome>) {
    let preamble = std::fs::read_to_string(corpus_dir().join("preamble.niles")).expect("preamble");
    let sound = std::fs::read_to_string(corpus_dir().join("sound.niles")).expect("sound corpus");
    let defective =
        std::fs::read_to_string(corpus_dir().join("defective.niles")).expect("defective corpus");

    let run = |src: &str| -> Vec<Outcome> {
        split_functions(src)
            .into_iter()
            .map(|(name, body)| check_one(&preamble, &name, &body))
            .collect()
    };
    (run(&sound), run(&defective))
}

#[test]
fn the_corpus_is_large_enough_to_measure_anything() {
    // Guarding the guard. A splitter that silently lost functions would produce a *better*
    // rate from a smaller corpus, which is the failure mode that looks like success.
    let (sound, defective) = run_corpus();
    assert!(
        sound.len() >= 40,
        "the sound corpus should hold at least forty functions; the splitter found {}",
        sound.len()
    );
    assert_eq!(defective.len(), 5, "five deliberate defects");

    // And every function should be distinct: a splitter that duplicated one would inflate
    // whichever verdict that function happens to get.
    let mut names: Vec<&str> = sound.iter().map(|o| o.name.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(
        before,
        names.len(),
        "duplicate function names in the corpus"
    );
}

#[test]
fn every_corpus_function_compiles() {
    // **The load-bearing guard on the whole measurement.** A function that fails to parse
    // produces an all-zero report, which the tally files under `NoObligation` — so a corpus
    // that had quietly stopped compiling would report a *better* undecided rate than one that
    // works. That is the failure mode which looks like success, and it is the only one that
    // could make E18's headline number a fiction.
    let (sound, defective) = run_corpus();
    for o in &sound {
        assert_eq!(
            o.errors, 0,
            "`{}` is a correct function and the front end rejected it: {o:?}",
            o.name
        );
    }

    // The defective ones are *expected* to produce an error diagnostic — that is the checker
    // doing its job — so the assertion runs the other way, and it is the stronger of the two:
    // a defect that compiled cleanly would be a defect nobody is told about.
    for o in &defective {
        let must_object = o.violates > 0;
        if must_object {
            assert_eq!(
                o.errors, 1,
                "`{}` is a must-violation and should be an error, not a warning: {o:?}",
                o.name
            );
        }
    }
}

#[test]
fn a_guard_does_not_make_conservation_undecidable() {
    // The finding E18 exists to have produced, and the one the review predicted would go the
    // other way.
    //
    // The undecidability result behind `Undecided` — Müller-Olm and Seidl's reduction from
    // Post's Correspondence Problem — is about deciding whether an *affine relation holds at a
    // program point*. A limit check is exactly such a relation, so the expectation was that
    // guarded transfers would be undecided and the soundness theorem would apply to a fragment
    // nobody writes.
    //
    // They are proved, and the reason is that conservation asks a different question.
    // Conservation asks whether the net is zero **on every path**, not whether the guard is
    // true. A guard that gates *whether* a balanced transaction happens does not threaten
    // conservation at all: both arms conserve, the join agrees on zero, and the row is proved
    // without the guard ever being decided.
    //
    // Locked in as a test because it is a claim about the analysis, and a claim about an
    // analysis that nothing checks decays.
    let (sound, _) = run_corpus();
    for guarded in [
        "transfer_if_sufficient",
        "revolver_drawdown_within_limit",
        "fee_waived_for_premium",
        "overdraft_permitted_or_refused",
        "settlement_only_on_business_day",
    ] {
        let o = sound
            .iter()
            .find(|o| o.name == guarded)
            .unwrap_or_else(|| panic!("{guarded} missing from the corpus"));
        assert_eq!(
            o.verdict(),
            "Proved",
            "`{guarded}` is guarded by an affine condition and conservation is still \
             decidable: {o:?}"
        );
    }
}

#[test]
fn every_deliberate_defect_is_caught() {
    // **The negative control**, and the reason the sound corpus's numbers mean anything. A
    // checker that returned `Conserves` unconditionally would score perfectly on the forty
    // correct functions and fail here.
    let (_, defective) = run_corpus();
    for o in &defective {
        assert!(
            o.violates > 0 || o.may_violate > 0,
            "`{}` is a deliberate defect and the checker did not object: {o:?}",
            o.name
        );
    }
}

#[test]
fn a_straight_line_defect_is_an_accusation_and_a_branched_one_is_an_alarm() {
    // The distinction that makes the analysis honest. `Violates` is a must-statement — every
    // execution of this transaction moves money that does not balance — and it is only sound
    // where the body is straight-line and abort-free. The same arithmetic reached across a
    // merge is `MayViolate`, because whether that path is taken is not something the checker
    // can decide.
    //
    // A checker that accused a program it could not follow would teach its users to switch it
    // off, which is the failure mode this test exists to prevent.
    let (_, defective) = run_corpus();
    let by = |name: &str| {
        defective
            .iter()
            .find(|o| o.name == name)
            .unwrap_or_else(|| panic!("{name}"))
    };

    for straight_line in [
        "transfer_with_lost_cent",
        "syndicated_residue",
        "doubled_credit",
    ] {
        let o = by(straight_line);
        assert!(
            o.violates > 0,
            "`{straight_line}` is straight-line and provably unbalanced, so it must be an \
             accusation rather than an alarm: {o:?}"
        );
    }

    let branched = by("unbalanced_on_one_branch");
    assert!(
        branched.may_violate > 0 && branched.violates == 0,
        "a defect on one branch of a conditional is an alarm, not a proof: {branched:?}"
    );
}

/// The measurement. Writes `results/E18-solver-verdicts.md`.
///
/// Ignored by default, like E16's: it is a measurement rather than a check, and a test suite
/// that runs on every build should not be writing results files.
///
/// ```sh
/// cargo test -p niles-lang --test solver_verdicts -- --ignored --nocapture
/// ```
#[test]
#[ignore = "a measurement, not a check: run explicitly with --ignored"]
fn e18_verdict_distribution() {
    let (sound, defective) = run_corpus();

    let mut doc = String::new();
    doc.push_str("# E18 — What the currency-row solver decides\n\n");
    doc.push_str(
        "**Generated by `cargo test -p niles-lang --test solver_verdicts -- --ignored`.**\n\n\
         Contribution 4's soundness theorem says a well-typed Niles program conserves money. \
         The analysis behind it has four verdicts, and `Undecided` is not a weakness to be \
         apologised for: with affine equality guards in the language, deciding whether an \
         affine relation holds at a program point is **undecidable** — Müller-Olm and Seidl \
         reduce Post's Correspondence Problem to it — so the fourth verdict is forced by the \
         problem rather than conceded by the implementation.\n\n\
         What was never measured is **how often it fires on code somebody would write**. A \
         soundness theorem whose analysis is undecided on an ordinary limit check is true and \
         applies to a fragment nobody lives in. This is that measurement: forty-five \
         functions, run one at a time through the real front end against one schema.\n\n",
    );

    let tally = |set: &[Outcome]| {
        let mut counts = std::collections::BTreeMap::new();
        for o in set {
            *counts.entry(o.verdict()).or_insert(0usize) += 1;
        }
        counts
    };

    let sound_counts = tally(&sound);
    doc.push_str("## The forty correct functions\n\n");
    doc.push_str("| Verdict | Functions | Share |\n|---|---|---|\n");
    for (verdict, n) in &sound_counts {
        doc.push_str(&format!(
            "| {verdict} | {n} | {:.0}% |\n",
            *n as f64 / sound.len() as f64 * 100.0
        ));
    }
    let undecided_share =
        sound_counts.get("Undecided").copied().unwrap_or(0) as f64 / sound.len() as f64 * 100.0;
    doc.push_str(&format!(
        "\n**{undecided_share:.0}% of correct functions are `Undecided`.**\n\n"
    ));

    doc.push_str("### By function\n\n| Function | Verdict | Proved | Undecided | MayViolate | Violates |\n|---|---|---|---|---|---|\n");
    for o in &sound {
        doc.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            o.name,
            o.verdict(),
            o.proved,
            o.undecided,
            o.may_violate,
            o.violates
        ));
    }

    doc.push_str("\n## The five deliberate defects — the negative control\n\n");
    doc.push_str("| Function | Verdict | The defect |\n|---|---|---|\n");
    let described = [
        (
            "transfer_with_lost_cent",
            "three legs, off by one minor unit",
        ),
        ("syndicated_residue", "33.33 three ways against 100.00"),
        (
            "currency_mix",
            "sums to zero only if the currency is ignored",
        ),
        ("doubled_credit", "a copy-pasted leg"),
        (
            "unbalanced_on_one_branch",
            "the `else` branch does not balance",
        ),
    ];
    for o in &defective {
        let why = described
            .iter()
            .find(|(n, _)| *n == o.name)
            .map(|(_, w)| *w)
            .unwrap_or("");
        doc.push_str(&format!("| `{}` | {} | {why} |\n", o.name, o.verdict()));
    }
    doc.push_str(
        "\nAll five are caught. Without this control the numbers above would establish only \
         that the checker is quiet — a property a checker returning `Conserves` \
         unconditionally would also have.\n\n\
         The split between `Refuted` and `MayViolate` is the analysis's honesty rather than a \
         detail: an accusation is a **must**-statement, sound only where the body is \
         straight-line and abort-free, and the same arithmetic reached across a merge is an \
         alarm. A checker that accused a program it could not follow would teach its users to \
         switch it off.\n\n",
    );

    // ── the interprocedural group ─────────────────────────────────────────────────────
    let inter = run_interprocedural();
    let (sound_inter, defect_inter): (Vec<&Outcome>, Vec<&Outcome>) =
        inter.iter().partition(|o| !o.name.starts_with("DEFECT_"));
    doc.push_str("## The interprocedural group\n\n");
    doc.push_str(
        "**This group did not exist before, and could not have.** Until the checker computed \
         function summaries, a call to another function in the same program contributed \
         nothing at all to its caller — not its effects, not its money. A caller whose callee \
         posted one half of a transfer had *no* conservation obligation: not a violation, not \
         an alarm, not a runtime obligation. The ten dollars were not counted.\n\n\
         So a multi-function case would have measured the corpus splitter rather than the \
         solver, and every case in the two groups above is a single function. These are the \
         shapes banking code is actually written in: a transfer helper called by a product, a \
         fee routine called by three, a recursive amortisation.\n\n",
    );
    doc.push_str("| Case | Verdict | Proved | Undecided | MayViolate | Violates |\n|---|---|---|---|---|---|\n");
    for o in &inter {
        doc.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            o.name,
            o.verdict(),
            o.proved,
            o.undecided,
            o.may_violate,
            o.violates
        ));
    }
    let proved_inter = sound_inter.iter().filter(|o| o.proved > 0).count();
    doc.push_str(&format!(
        "\n{proved_inter} of {} conserving cases are proved across the call boundary, and both \
         deliberate defects are caught. The two that are not proved are the two the analysis \
         is honest about: an amount computed twice by a call the solver cannot see through, \
         and a self-recursive helper whose summary is marked `havoc` because its row appears \
         on both sides of its own definition. Neither is silently assumed to conserve, which \
         is the distinction the `havoc` flag exists to keep visible — a fresh symbol with no \
         havoc record would look exactly like a clean answer.\n\n",
        sound_inter.len()
    ));
    let _ = &defect_inter;

    doc.push_str("## The finding\n\n");
    doc.push_str(
        "**A guard does not make conservation undecidable, and that was not the expectation.**\n\n\
         The undecidability result behind the `Undecided` verdict — Müller-Olm and Seidl's \
         reduction from Post's Correspondence Problem — is about deciding whether an *affine \
         relation holds at a program point*. A limit check is exactly such a relation, so the \
         architecture review predicted that guarded transfers would land in `Undecided` and \
         that Contribution 4's soundness theorem would therefore apply to a fragment nobody \
         writes.\n\n\
         All five guarded functions in the corpus are **proved**. The reason is that \
         conservation asks a different question from the one the reduction is about: it asks \
         whether the net is zero *on every path*, not whether the guard is true. A guard that \
         gates **whether** a balanced transaction happens does not threaten conservation — both \
         arms conserve, the join agrees on zero, and the row is proved without the guard ever \
         being decided.\n\n\
         `Undecided` is still forced in general, and the module is right to have it: a guard \
         that made the *amounts* depend on an affine relation would reach it. What this \
         measurement establishes is that the ordinary shape of banking code — a limit check \
         around a conserving posting — is not that case.\n\n\
         ## A change this measurement caused\n\n\
         The corpus's fifth defect puts an unbalanced posting on one branch of a conditional. \
         It came back `Undecided`: the join goes to top where two arms disagree, which is sound \
         and is the right default, and it **discarded a finding the checker already had**. The \
         user was handed a runtime obligation instead of being told that one path is fifty \
         cents short.\n\n\
         `merge_branches` now judges each arm *before* the join and reports a decided non-zero \
         residue as a **may**-violation — true by construction, since some path is unbalanced, \
         and never an accusation, since which path executes is not decidable there. That is the \
         case `Verdict::MayViolate` was designed for and, until this corpus existed, the case \
         it never saw.\n\n\
         ## Reading this\n\n",
    );
    doc.push_str(&format!(
        "The corpus is grouped by the *shape* of the arithmetic rather than by banking \
         product, so a verdict is attributable to a construct. Straight-line transfers, \
         three-legged fee sets, many-legged allocations and symbolic amounts are the shapes \
         production code is mostly made of; guarded paths are the shape the review predicted \
         would be hardest, because a limit check is an affine guard and that is exactly what \
         the undecidability result is about.\n\n\
         Measured on {} correct functions and {} deliberate defects.\n",
        sound.len(),
        defective.len()
    ));

    let path = workspace_root().join("results/E18-solver-verdicts.md");
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(&path, doc).expect("write E18");

    eprintln!("=== E18 ===");
    for (v, n) in &sound_counts {
        eprintln!("  {v}: {n}/{}", sound.len());
    }
    for o in &defective {
        eprintln!("  defect `{}` -> {}", o.name, o.verdict());
    }
    eprintln!("wrote {}", path.display());
}

#[test]
fn the_interprocedural_group_reaches_the_caller() {
    // **What this group is for.** Every case here is at least two functions, and the
    // obligation belongs to the caller. Before summaries existed, a call contributed
    // nothing at all: `caller_of_a_half_poster` had *no* conservation obligation — not a
    // violation, not an alarm, not a runtime obligation — because the ten dollars its
    // callee moved were never counted.
    let cases = run_interprocedural();
    assert!(
        cases.len() >= 8,
        "the interprocedural group should hold at least eight cases; found {}",
        cases.len()
    );
    let by = |name: &str| {
        cases
            .iter()
            .find(|o| o.name == name)
            .unwrap_or_else(|| panic!("no case `{name}`"))
    };

    // The two deliberate defects must accuse, and they are the only ones that may.
    for defect in [
        "DEFECT_caller_of_a_half_poster",
        "DEFECT_helper_credits_twice",
    ] {
        let o = by(defect);
        assert!(
            o.violates > 0,
            "`{defect}` moves a literal amount across a call and does not balance, so the \
             residue is exact and this must be an accusation: {o:?}"
        );
    }
    for o in &cases {
        if !o.name.starts_with("DEFECT_") {
            assert_eq!(
                o.violates, 0,
                "`{}` conserves; accusing it would be a false positive, which is the one \
                 failure that teaches users to switch a checker off: {o:?}",
                o.name
            );
            assert_eq!(o.errors, 0, "`{}` must compile clean: {o:?}", o.name);
        }
    }

    // The proofs the summaries buy. Each of these was previously *nothing* — the caller's
    // transaction had no obligation to prove.
    for proved in [
        "helper_posts_both_halves",
        "helper_takes_the_amount",
        "two_helpers_one_transaction",
        "generic_currency_helper",
        "three_deep_chain",
        "helper_branches_and_both_arms_balance",
    ] {
        let o = by(proved);
        assert!(
            o.proved > 0,
            "`{proved}` conserves across the call and the solver should say so: {o:?}"
        );
    }

    // And the two honest boundaries. An amount the solver cannot see through, and a
    // recursive helper it cannot summarise, are both *discharged to the runtime* rather
    // than proved — which is the distinction `havoc` exists to keep visible.
    for undecided in ["helper_computes_the_amount", "recursive_amortisation"] {
        let o = by(undecided);
        assert!(
            o.undecided > 0 && o.violates == 0,
            "`{undecided}` is beyond the analysis and must be handed to the runtime, not \
             proved and not accused: {o:?}"
        );
    }
}
