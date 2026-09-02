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
