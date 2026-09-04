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

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Refused,
    Warned,
    Accepted,
}

/// **The compiler binary, found once — not `cargo run` per case.**
///
/// This used to spawn `cargo run -q -p nilesc` for every corpus file and classify
/// `status.code() != Some(0)` as [`Verdict::Refused`]. A cargo that could not run — a build
/// directory lock held by the outer `cargo test`, which is what happens whenever
/// `CARGO_TARGET_DIR` is set, as most CI sets it — was therefore indistinguishable from a
/// compiler that refused a program. On a ten-core machine with that variable set, this test
/// reported twelve refusals and zero warnings against the eleven and one the corpus actually
/// produces, and §6.10.3's table would have been "corrected" to a number no compiler ever
/// emitted.
///
/// Two changes close it. The binary is invoked directly, so the exit status is the
/// compiler's own; and the verdict is read from the compiler's markers rather than from an
/// exit code, so *no output at all* is a distinct outcome (`BLOCKED-nilesc`) instead of
/// being silently counted as a refusal.
fn nilesc_binary() -> &'static PathBuf {
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let root = repo_root();
        // Honour `CARGO_TARGET_DIR`; the cause of the defect above is also where the binary
        // lands when it is set.
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target"));
        for profile in ["release", "debug"] {
            let p = target.join(profile).join("nilesc");
            if p.is_file() {
                return p;
            }
        }
        // Build it once, and only if it is absent. A build failure is reported as
        // `BLOCKED-nilesc` by the caller when the binary still does not exist — never as a
        // verdict about the language.
        let _ = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .args(["build", "--quiet", "-p", "nilesc"])
            .current_dir(&root)
            .status();
        for profile in ["release", "debug"] {
            let p = target.join(profile).join("nilesc");
            if p.is_file() {
                return p;
            }
        }
        panic!(
            "BLOCKED-nilesc: no `nilesc` binary under {} and `cargo build -p nilesc` did not \
             produce one. These verdicts are what §6.10.3 reports, so running them without a \
             compiler would put a number in the thesis that no compiler emitted. Build it and \
             re-run.",
            target.display()
        )
    })
}

/// The compiler's verdict on one file, read from its own markers.
///
/// `nilesc check` prints `ok: …` on stdout when it accepts, and `error[NLnnnn]` on stderr
/// when it refuses. Exactly one of those is present whenever the compiler ran. Neither being
/// present means it did not run, which is a blocked test and not a refusal — the distinction
/// the old exit-code rule could not make.
fn verdict_of(file: &std::path::Path) -> Verdict {
    let bin = nilesc_binary();
    let out = Command::new(bin)
        .arg("check")
        .arg(file)
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|e| panic!("BLOCKED-nilesc: cannot run {}: {e}", bin.display()));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    let accepted = stdout.trim_start().starts_with("ok:");
    let refused = stderr.contains("error[");
    assert!(
        accepted || refused,
        "BLOCKED-nilesc: `{} check {}` produced neither an `ok:` line nor an `error[` \
         diagnostic (exit {:?}). The compiler did not run, or ran and said nothing; either \
         way this is not a verdict about the program.\nstdout: {stdout}\nstderr: {stderr}",
        bin.display(),
        file.display(),
        out.status.code(),
    );
    assert!(
        !(accepted && refused),
        "`{}` both accepted and refused {}; the marker rule cannot classify that",
        bin.display(),
        file.display()
    );

    if refused {
        Verdict::Refused
    } else if stderr.contains("warning[") {
        Verdict::Warned
    } else {
        Verdict::Accepted
    }
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
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            let verdict = verdict_of(&p);
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

/// **The verdict may not be read from an exit code, and this test says so in the source.**
///
/// The repair above is invisible to any assertion about behaviour: a test that spawns
/// `cargo run` and one that runs the binary produce identical verdicts on a machine where
/// cargo happens to work, which is exactly why the defect survived. So the guard is
/// structural — it fails if a `cargo run` subprocess comes back into this file.
#[test]
fn the_corpus_verdicts_do_not_come_from_a_nested_cargo() {
    let src = include_str!("counterproposal.rs");
    // The `.args([…"run"…])` form, in any spelling, spawning cargo.
    let spawns_cargo_run = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .any(|l| l.contains("\"run\"") && (l.contains("args") || l.contains("arg(")));
    assert!(
        !spawns_cargo_run,
        "this file spawns `cargo run` again. A cargo that cannot run — a build-directory \
         lock, which is what happens whenever CARGO_TARGET_DIR is set — is then \
         indistinguishable from a compiler that refused the program, and §6.10.3's table \
         gets a number no compiler emitted. Invoke the binary and read its markers."
    );
}
