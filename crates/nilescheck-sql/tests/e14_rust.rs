//! **E14's Rust column for d14, measured in the run** (cycle 14, R2-05): round 1's class-B
//! probe, re-run on every gate rather than cited. Each file of
//! `crates/counterproposal/d14/rust/` is compiled by `rustc` with `#![deny(unused_must_use)]`
//! (in the file) and linted by `clippy-driver -D warnings`; the verdict is whether each
//! refuses it. A missing `rustc` or `clippy-driver` is a blocked test, never a verdict.

use std::path::{Path, PathBuf};
use std::process::Command;

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../counterproposal/d14/rust")
}

/// `Some(true)` refused, `Some(false)` accepted; panics when the tool did not run.
fn refused(tool: &str, file: &Path) -> bool {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "e14-{tool}-{}",
        file.file_stem().unwrap().to_string_lossy()
    ));
    let mut c = Command::new(tool);
    c.args(["--edition", "2021", "--crate-type", "bin", "-o"])
        .arg(&out)
        .arg(file)
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    if tool == "clippy-driver" {
        c.args(["-D", "warnings"]);
    }
    let o = c.output().unwrap_or_else(|e| {
        panic!("BLOCKED-{tool}: cannot run `{tool}`: {e}; the Rust column needs the toolchain's rustc and clippy")
    });
    let err = String::from_utf8_lossy(&o.stderr);
    if o.status.success() {
        return false;
    }
    assert!(
        err.contains("error"),
        "BLOCKED-{tool}: `{tool}` failed without a diagnostic on {}: {err}",
        file.display()
    );
    true
}

/// (file, refused by rustc, refused by clippy).
const EXPECTED: &[(&str, bool, bool)] = &[
    ("d14_1_bare_call", true, true),
    ("d14_2_discarded_binding", false, false),
    ("d14_3_bound_read_dropped", false, false),
    ("d14_4_explicit_ignore", false, false),
    // clippy's `forget_non_drop`: forgetting a value with no destructor is dropping it.
    ("d14_5_forget", false, true),
    // ... and with a destructor (a drop bomb) the lint no longer applies.
    ("d14_5b_forget_with_destructor", false, false),
    ("ok_resolved", false, false),
];

pub fn rust_line(v: &[(String, bool, bool)]) -> String {
    let five: Vec<&(String, bool, bool)> = v
        .iter()
        .filter(|(n, _, _)| n.starts_with("d14_") && n.len() > 5 && !n.contains("5b"))
        .collect();
    let r = five.iter().filter(|x| x.1).count();
    let c = five.iter().filter(|x| x.2).count();
    format!(
        "Rust, d14: rustc with `deny(unused_must_use)` refuses {r} of {}; `clippy -D warnings` refuses {c} of {}, and the `mem::forget` spelling only while `Hold` has no destructor.",
        five.len(),
        five.len()
    )
}

fn measured() -> Vec<(String, bool, bool)> {
    EXPECTED
        .iter()
        .map(|(n, _, _)| {
            let f = dir().join(format!("{n}.rs"));
            (
                n.to_string(),
                refused("rustc", &f),
                refused("clippy-driver", &f),
            )
        })
        .collect()
}

#[test]
fn rust_refuses_what_round_one_measured_and_no_more() {
    let got = measured();
    let want: Vec<(String, bool, bool)> = EXPECTED
        .iter()
        .map(|(n, a, b)| (n.to_string(), *a, *b))
        .collect();
    assert_eq!(got, want);
    let doc = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../results/E14-minimal-counterproposal.md"),
    )
    .unwrap();
    let line = rust_line(&got);
    assert!(
        doc.contains(&line),
        "E14's document must say, verbatim:\n  {line}"
    );
}
