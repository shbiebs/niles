//! **Stamp the commit that produced this binary into the binary.**
//!
//! F-29: the contract table was compared across two instances of one host class and read as a
//! regression — `report` 2.68× MET against 2.17× NOT MET — when the three arms measured on one
//! instance were within 8% of each other. The changes did not move the row; the instance did. A
//! ratio without a host, a session and a commit beside it is not a result, and the way to make
//! that impossible is to put them in the document the harness writes.
//!
//! Read at **build** time and not at run time, deliberately. `git rev-parse` in the running
//! process reports the working tree's HEAD, which is whatever the tree happens to be checked out
//! at when the binary is invoked — a benchmark built at one commit and run after a checkout would
//! stamp itself with code it does not contain. That is the same class of error as the stale
//! worktree that made a whole `run4.sh` re-run measure the wrong commit.
//!
//! `NILES_DIRTY` matters as much as the hash: a dirty tree means the commit does **not** identify
//! the code, so the document says so rather than printing a hash that cannot be checked out.

use std::process::Command;

fn main() {
    // Rebuild when HEAD moves, so the stamp cannot go stale behind an unchanged source file.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");

    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };

    let commit = git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    // `--porcelain` over any tracked change. Untracked files are excluded: this project's
    // working copies carry four of them by design, and treating those as "dirty" would mark
    // every run on the author's machine unidentifiable and train a reader to ignore the flag.
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    println!("cargo:rustc-env=NILES_COMMIT={commit}");
    println!("cargo:rustc-env=NILES_DIRTY={dirty}");
}
