//! **Appendix F is the oracle, byte for byte.**
//!
//! The appendix opens by calling this "the one component of the system that is implemented
//! and passing tests today", and then printed a paraphrase of it — a fold whose signature
//! had drifted from the real one, and a `Scale` type declared in the listing and used
//! nowhere in it. A reader auditing the semantics by reading the appendix was auditing
//! something that would not compile.
//!
//! The appendix now carries the region of `src/oracle.rs` between its `BEGIN:appendix-f` and
//! `END:appendix-f` markers, extracted by `thesis/gen-appendix-f.py`. This test runs the
//! extractor and compares, so a change to the oracle that is not reflected in the appendix
//! fails the build rather than leaving a reader with the old semantics.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_appendix_carries_the_oracle_the_suite_runs() {
    let out = Command::new("python3")
        .arg("thesis/gen-appendix-f.py")
        .current_dir(repo_root())
        .output()
        .expect("run the extractor");
    assert!(
        out.status.success(),
        "the extractor failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let extracted = String::from_utf8_lossy(&out.stdout).into_owned();

    let appendix = std::fs::read_to_string(repo_root().join("thesis/appendix-f-core.md"))
        .expect("thesis/appendix-f-core.md");
    assert_eq!(
        appendix.trim(),
        extracted.trim(),
        "the appendix and the oracle have diverged. Run `make reproduce`."
    );

    // And the extraction is not empty or trivially small, which a broken marker would make
    // it while leaving the equality above satisfied.
    assert!(
        extracted.lines().count() > 40,
        "the extracted core is {} lines; the markers are probably in the wrong place",
        extracted.lines().count()
    );
    assert!(
        extracted.contains("pub fn ledger_balance"),
        "the fold that defines a balance must be inside the excerpt"
    );
}
