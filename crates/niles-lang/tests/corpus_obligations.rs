//! **How much of the conservation burden the checker actually carries (H-S6).**
//!
//! H-S6 says static checkability subsumes runtime policing. The mutant corpus addresses one
//! half — that the checker refuses what it should. The other half asks what *fraction* of a
//! real program's conservation obligations it discharges without a runtime check, and until
//! this file existed nothing counted: `status.toml` recorded "no measurement of any kind".
//!
//! This counts, over every Niles program in both repositories, using `nilesc
//! report-obligations`. It is a measurement rather than a threshold — the number is written
//! to `results/obligations.csv` and reported in §9.11 — with one assertion that keeps it
//! honest: every file must *check*, so the fraction is over the whole corpus rather than
//! over the files that happened to compile.
//!
//! **What it does not measure** is the clause H-S6 states about runtime *cost* — "at no
//! measurable runtime cost". That needs a trigger-based SQL baseline and a ported corpus,
//! neither of which exists, and `status.toml` says so rather than letting this file stand in
//! for it.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// **The GBS corpus, and a refusal when it is not there.**
///
/// `../gbs/niles/gbs.niles` carries 11 of the corpus's 16 obligations — more than the rest of
/// the repository put together. The directory walk below `continue`s past a root it cannot
/// read, so in a checkout with no sibling GBS this test used to write a table of 5 and call
/// it the measurement: `TOTAL,,16,0` became `TOTAL,,5,0`, `results/MANIFEST.csv` still
/// classed the file `byte-deterministic`, and `make reproduce` failed on the diff with
/// nothing saying why. A missing corpus that produces a *smaller* answer looks exactly like a
/// correct one — the Appendix D failure mode, in a second place.
///
/// So the pair is required, with the same opt-out `downstream_adapter` uses: `NILES_NO_GBS`
/// is for standalone development, and it suppresses the *write* rather than shrinking the
/// number, because a fraction measured over part of the corpus is not this fraction.
fn gbs_corpus() -> std::path::PathBuf {
    let p = repo_root().join("../gbs/niles");
    if !p.is_dir() && std::env::var("NILES_NO_GBS").is_err() {
        panic!(
            "the obligation corpus includes the GBS programs beside this checkout and \
             {} is not a directory. Eleven of sixteen obligations live there, so counting \
             without it would report a smaller fraction as though it were this one. Put a \
             GBS checkout beside this repository, or set NILES_NO_GBS=1 to measure the \
             partial corpus without publishing it.",
            p.display()
        );
    }
    p
}

/// Every `.niles` program in this repository and the GBS checkout beside it.
///
/// The corpus is deliberately *all* of them and not a selection: choosing which programs to
/// count is choosing the answer.
fn corpus() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let roots = [
        repo_root().join("examples"),
        repo_root().join("niles"),
        gbs_corpus(),
    ];
    for r in roots {
        let Ok(entries) = std::fs::read_dir(&r) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("niles") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn obligations(path: &std::path::Path) -> (bool, u64, u64) {
    let out = Command::new("cargo")
        .args(["run", "-q", "-p", "nilesc", "--", "report-obligations"])
        .arg(path)
        .current_dir(repo_root())
        .output()
        .expect("run nilesc");
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let field = |k: &str| -> u64 {
        line.split_whitespace()
            .find_map(|f| f.strip_prefix(k))
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("no `{k}` in {line:?}"))
    };
    (field("ok=") == 1, field("static="), field("runtime="))
}

#[test]
fn the_corpus_reports_what_the_checker_discharges() {
    let files = corpus();
    assert!(
        files.len() >= 3,
        "the corpus should hold at least the examples and the GBS schema; found {}",
        files.len()
    );

    let mut csv = String::from("file,checks,static,runtime\n");
    let (mut total_static, mut total_runtime) = (0u64, 0u64);
    let mut failing = Vec::new();
    for f in &files {
        let (ok, s, r) = obligations(f);
        if !ok {
            failing.push(f.display().to_string());
        }
        total_static += s;
        total_runtime += r;
        let name = f
            .strip_prefix(repo_root())
            .unwrap_or(f)
            .display()
            .to_string();
        csv.push_str(&format!("{name},{},{s},{r}\n", u8::from(ok)));
    }
    let total = total_static + total_runtime;
    csv.push_str(&format!("TOTAL,,{total_static},{total_runtime}\n"));

    assert!(
        failing.is_empty(),
        "these corpus files do not check, so the fraction below would be over a subset: \
         {failing:?}"
    );
    assert!(
        total > 0,
        "the corpus states no conservation obligations at all, so there is nothing to \
         report; a fraction of zero obligations is not a measurement"
    );

    if std::env::var("NILES_NO_GBS").is_ok() && !gbs_corpus().is_dir() {
        println!(
            "NILES_NO_GBS: the GBS half of the corpus is absent, so the count above is over \
             a subset and results/obligations.csv is NOT written. The committed file stays \
             the measurement it was."
        );
        return;
    }
    let path = repo_root().join("results/obligations.csv");
    std::fs::create_dir_all(path.parent().expect("results/")).ok();
    std::fs::write(&path, &csv).expect("write results/obligations.csv");

    println!(
        "H-S6, static half: {total_static} of {total} conservation obligations proved \
         statically ({:.0}%), {total_runtime} discharged to the runtime seal, over {} files",
        total_static as f64 / total as f64 * 100.0,
        files.len()
    );
}
