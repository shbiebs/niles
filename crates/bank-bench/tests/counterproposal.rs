//! **The Niles half of E14, counted from the corpus.**
//!
//! §6.10.3 prints a table scoring twelve defect classes by the stage at which each is
//! caught, and §11.5 says in prose that "Niles catches eleven at compile time and warns on
//! the twelfth". Both were typed beside a corpus that had eleven files, two of which the
//! SQL side had and the Niles side did not (`d7`, `d10`) — so "written twice" was true of
//! nine classes, and the counts in the table could not be derived from anything in the
//! repository.
//!
//! This test derives them. It runs `nilesc check` over every file in the corpus, classifies
//! each as refused / warned / silently accepted, and asserts the numbers §6.10.3 prints.
//! The PostgreSQL half needs a live server and stays in `run.sh`; this half needs only the
//! compiler, so it can be part of the gate — which is where a number a chapter depends on
//! belongs.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Refused,
    Warned,
    Accepted,
}

fn corpus() -> Vec<(String, Verdict)> {
    let dir = repo_root().join("crates/counterproposal/niles");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("niles"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            // `nilesc` lives in another crate, so `CARGO_BIN_EXE_` is not set for this
            // test binary. Driving it through `cargo run` is what `run.sh` does, and using
            // the same path keeps the two from diverging.
            let out = Command::new("cargo")
                .args(["run", "-q", "-p", "nilesc", "--", "check"])
                .arg(&p)
                .current_dir(repo_root())
                .output()
                .expect("run nilesc");
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            let verdict = if out.status.code() != Some(0) {
                Verdict::Refused
            } else if text.contains("warning[") {
                Verdict::Warned
            } else {
                Verdict::Accepted
            };
            (name, verdict)
        })
        .collect()
}

#[test]
fn both_sides_of_the_corpus_carry_the_same_defect_classes() {
    let names: Vec<String> = corpus().into_iter().map(|(n, _)| n).collect();
    for n in 1..=13 {
        assert!(
            names.iter().any(|f| f.starts_with(&format!("d{n}_"))),
            "the corpus has no `d{n}`: {names:?}. The SQL side numbers its cases D2..D11 and \
             the Niles side is scored against them; a gap on one side makes \"the same \
             classes written twice\" false of the pair."
        );
    }
}

#[test]
fn the_thesis_reports_the_verdicts_the_corpus_produces() {
    let v = corpus();
    let refused = v.iter().filter(|(_, x)| *x == Verdict::Refused).count();
    let warned = v.iter().filter(|(_, x)| *x == Verdict::Warned).count();
    let accepted = v.iter().filter(|(_, x)| *x == Verdict::Accepted).count();
    assert_eq!(
        refused + warned + accepted,
        v.len(),
        "every case has exactly one verdict"
    );

    let chapter = std::fs::read_to_string(repo_root().join("thesis/06-architecture-and-niles.md"))
        .expect("§6");
    assert!(
        chapter.contains(&format!(
            "| Compile time | 0 | {refused} (+{warned} warning)"
        )),
        "§6.10.3's table must say `{refused} (+{warned} warning)` for the Niles column; the \
         corpus produces {refused} refusals, {warned} warnings and {accepted} silent \
         acceptances. Verdicts: {v:?}"
    );

    let discussion =
        std::fs::read_to_string(repo_root().join("thesis/11-discussion.md")).expect("§11");
    let words = ["nine", "ten", "eleven", "twelve", "thirteen"];
    let word = words
        .get(refused.saturating_sub(9))
        .copied()
        .unwrap_or("some");
    assert!(
        discussion.contains(&format!("Niles catches {word} at compile time")),
        "§11.5 must say Niles catches `{word}` ({refused}) at compile time"
    );
}

/// The one case the compiler accepts in silence, named so that it cannot be forgotten.
///
/// `d6` writes a view predicate over a wall-clock helper. Niles has no `now()`, so the
/// defect has no direct spelling — but the file is accepted with *no diagnostic at all*,
/// which is not the same as being inexpressible: an unknown function in a view predicate
/// simply is not checked. That is a real gap and it is recorded here rather than counted as
/// a win.
#[test]
fn the_silently_accepted_case_is_the_one_the_thesis_names() {
    let silent: Vec<String> = corpus()
        .into_iter()
        .filter(|(_, v)| *v == Verdict::Accepted)
        .map(|(n, _)| n)
        .collect();
    assert_eq!(
        silent,
        vec!["d6_wall_clock_predicate".to_string()],
        "exactly one case is accepted with no diagnostic, and §6.10.3 says which"
    );
}
