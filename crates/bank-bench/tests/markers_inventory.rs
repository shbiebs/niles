//! **The committed marker inventory equals what the generator prints, or this fails.**
//!
//! `docs/audit/cycle-11/markers-653a369.txt` listed 34 distinct markers at a commit an
//! unrestricted grep finds 50 in. Nothing compared the file to the tree, so the gap survived a
//! cycle and was found by an auditor counting by hand.
//!
//! A generated document that nothing regenerates is a document that drifts — the rule this
//! repository already applies to Appendix D, the SQL surface and the keyword reference. This
//! applies it to the inventory: `docs/audit/cycle-12/markers-653a369.txt` is regenerated at
//! the commit it names and compared byte for byte.
//!
//! The commit is fixed and historical, so this test is deterministic and says nothing about
//! the current head. An inventory *of* the current head cannot be committed at that head — it
//! would have to contain its own hash — which is why the artifact names the baseline it
//! describes rather than the commit it happens to sit on.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `docs/audit/cycle-12/markers-653a369.txt`, and the commit it claims to describe.
const INVENTORY: &str = "docs/audit/cycle-12/markers-653a369.txt";
const COMMIT: &str = "653a369";

fn generator_output(commit: &str) -> String {
    let out = Command::new("python3")
        .current_dir(repo_root())
        .args(["docs/audit/tools/markers.py", commit])
        .output()
        .expect("run docs/audit/tools/markers.py");
    assert!(
        out.status.success(),
        "the marker generator failed at {commit}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("the generator writes utf-8")
}

#[test]
fn the_generator_passes_its_own_self_test() {
    let out = Command::new("python3")
        .current_dir(repo_root())
        .args(["docs/audit/tools/markers.py", "--self-test"])
        .output()
        .expect("run the generator's self-test");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "the marker generator's self-test failed:\n{text}"
    );
    // The three faults that made the previous inventory smaller each have a case, and a
    // self-test whose cases were deleted would still exit 0.
    for case in [
        "truncated at the first underscore",
        "docs/audit is in scope",
        "untracked files are out of scope",
    ] {
        assert!(
            text.contains(case),
            "the self-test no longer covers `{case}`:\n{text}"
        );
    }
}

#[test]
fn the_committed_inventory_is_what_the_generator_prints() {
    let path = repo_root().join(INVENTORY);
    let committed =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let generated = generator_output(COMMIT);

    // The committed copy carries a `SUPERSEDES` note the generator does not print, because
    // the file it supersedes is evidence and stays where it is. Everything below the header
    // block is the generator's, byte for byte.
    let body = |s: &str| {
        s.lines()
            .skip_while(|l| l.starts_with('#') || l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        body(&committed),
        body(&generated),
        "{INVENTORY} is not what `markers.py {COMMIT}` prints. Regenerate it:\n    \
         python3 docs/audit/tools/markers.py {COMMIT} --write {INVENTORY}\n(keeping the \
         SUPERSEDES header), rather than editing the file by hand."
    );
}

#[test]
fn the_inventory_is_larger_than_the_one_it_supersedes() {
    // Not a vanity check. The cycle-11 file is *smaller* than the tree, and a smaller
    // inventory looks exactly like a complete one — the Appendix D failure mode. If a future
    // change to the generator's scope quietly shrinks it back, this says so.
    let count = |s: &str| {
        s.lines()
            .filter(|l| !l.starts_with('#') && l.contains(" occ"))
            .count()
    };
    let new = count(&std::fs::read_to_string(repo_root().join(INVENTORY)).expect(INVENTORY));
    let old = std::fs::read_to_string(repo_root().join("docs/audit/cycle-11/markers-653a369.txt"))
        .map(|s| s.lines().filter(|l| l.contains(" occ")).count())
        .unwrap_or(0);
    assert!(
        new > old,
        "the corrected inventory lists {new} markers and the one it supersedes lists {old}. \
         The correction was that the old one was missing markers; if this is no longer true \
         the generator's scope has changed and the SUPERSEDES note is now wrong."
    );
    assert!(
        new >= 50,
        "an unrestricted grep at {COMMIT} finds 50 distinct markers and this lists {new}"
    );
}
