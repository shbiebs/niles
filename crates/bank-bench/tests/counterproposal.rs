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
        // **Build first, always, and take the profile this test is itself running in.**
        //
        // This used to take whatever binary was already lying in `target/`, preferring
        // `release`, and build only if none existed. Cycle 13 found the consequence:
        // `target/release/nilesc` in this container was dated 2026-09-07 while HEAD was
        // 2026-09-21, so every verdict the test had produced for six cycles came from a
        // cycle-8 compiler. It scored a fossil and the suite was green, because a stale
        // binary is a *present* binary and presence was the only thing checked.
        //
        // That is worse than the defect above it. A blocked test announces itself; a test
        // scoring an old compiler reports confident numbers about a language state that no
        // longer exists — and those numbers are §6.10.3's table and §11.5's sentence.
        //
        // An unconditional `cargo build` is cheap when it is a no-op and correct when it is
        // not. The profile is the one this test binary was compiled in, so a `cargo test`
        // and a `cargo test --release` each score the compiler they just built rather than
        // the other profile's leftovers.
        let _ = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .args(if cfg!(debug_assertions) {
                ["build", "--quiet", "-p", "nilesc"].as_slice()
            } else {
                ["build", "--quiet", "--release", "-p", "nilesc"].as_slice()
            })
            .current_dir(&root)
            .status();
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let p = target.join(profile).join("nilesc");
        if p.is_file() {
            return p;
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
fn the_corpus_is_thirteen_classes_and_ten_of_them_run_a_sql_case() {
    // **Thirteen classes are scored; ten of them run an executed SQL case.**
    //
    // The Niles side is `d1`..`d13`. `defects.sql` runs `D2`..`D11`, and the other three are
    // scored on the PostgreSQL side without a case of their own: `D1`'s verdict comes from
    // the `postings_conserve` constraint trigger in `schema.sql` — runtime, at COMMIT —
    // which is exactly what `D10` drops in order to show what a runtime check is worth; and
    // `D12` (an infeasible serve contract) and `D13` (a missing anchor index) are scored
    // against constructs SQL has no spelling for.
    //
    // The distinction is asserted rather than left to prose because the totals depend on it:
    // "thirteen written twice" and "ten executed cases" are both true of different things,
    // and a reader who conflates them gets PostgreSQL's `never` count wrong by two.
    let names: Vec<String> = corpus().into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        names.len(),
        13,
        "the Niles corpus is thirteen files: {names:?}"
    );
    for n in 1..=13 {
        assert!(
            names.iter().any(|f| f.starts_with(&format!("d{n}_"))),
            "the corpus has no `d{n}`: {names:?}"
        );
    }

    let sql = std::fs::read_to_string(repo_root().join("crates/counterproposal/sql/defects.sql"))
        .expect("defects.sql");
    let paired: Vec<usize> = (1..=13)
        .filter(|n| sql.contains(&format!("### D{n} ")))
        .collect();
    assert_eq!(
        paired,
        vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        "`defects.sql` runs D2..D11. If that changed, §6.10.3's paragraph about which three \
         are scored without an executed case has to change with it."
    );
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

/// **E14's own document reports the counts the corpus produces.**
///
/// `results/E14-minimal-counterproposal.md` is classed `historical` and is written by hand,
/// so nothing regenerated it and nothing compared it. Its Totals line said *11 at compile
/// time, 1 warning, 1 with no spelling in the language* while the corpus produced 12, 1 and
/// 0 — and two of its per-case cells named diagnostics the compiler no longer emits. A
/// hand-written results document beside a harness that produces different numbers is the
/// defect this whole file exists to prevent, one level out.
#[test]
fn the_e14_document_reports_the_counts_the_corpus_produces() {
    let v = corpus();
    let refused = v.iter().filter(|(_, x)| *x == Verdict::Refused).count();
    let warned = v.iter().filter(|(_, x)| *x == Verdict::Warned).count();
    let accepted = v.iter().filter(|(_, x)| *x == Verdict::Accepted).count();
    let doc = std::fs::read_to_string(repo_root().join("results/E14-minimal-counterproposal.md"))
        .expect("E14's document");
    let want = format!(
        "Niles: **{refused} at compile time, {warned} as a compile-time warning, \
         {accepted} accepted in silence,"
    );
    assert!(
        doc.contains(&want),
        "E14's document must carry the corpus's own totals. Expected a line containing:\n  \
         {want}\nThe corpus produces {refused} refusals, {warned} warnings and {accepted} \
         silent acceptances."
    );
}

/// **No case is accepted in silence, and the one that was is how the gap got closed.**
///
/// `d6` writes a view predicate over a wall-clock helper, `month_start()`. Niles has no
/// `now()`, so the defect has no direct spelling — but the file used to be accepted with *no
/// diagnostic at all*, which is a different thing from being inexpressible: an unknown
/// function in a view predicate simply was not checked. This test named it so it could not
/// be forgotten, and §6.10.3 reported it as the one silent acceptance.
///
/// Cycle 13's L-1 closed it. `month_start` is now NL0205, *cannot find function
/// `month_start` in this scope* — not because anyone went looking for `d6`, but because the
/// checker began resolving its names. The gap this test was holding open was one instance
/// of a hole in the type discipline, and the instance fell when the hole did.
///
/// The test stays, inverted. A future silent acceptance is a regression in exactly the way
/// `d6` was, and it should fail here by name rather than be noticed in a table.
#[test]
fn no_case_is_accepted_in_silence() {
    let silent: Vec<String> = corpus()
        .into_iter()
        .filter(|(_, v)| *v == Verdict::Accepted)
        .map(|(n, _)| n)
        .collect();
    assert!(
        silent.is_empty(),
        "these cases are accepted with no diagnostic at all: {silent:?}. That is not the \
         same as being inexpressible — it is the compiler not looking. `d6` was in this \
         position until L-1 taught the checker to resolve names; if something is here \
         again, §6.10.3's table is wrong and so is §11.5's sentence."
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
