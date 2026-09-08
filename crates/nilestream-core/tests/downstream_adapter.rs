//! **`Base` has an implementor outside this workspace, and nothing here could see it.**
//!
//! `nilestream_core::rev::Base` is implemented by `gbs-nilestream::JournalBase` in the GBS
//! repository, which builds against this tree by path dependency. T-06 changed
//! `reconstruct` and `deltas_at` from `&mut self` to `&self` — a correct change, made for a
//! measured reason — and the adapter kept the old signature. Both gates were green: this
//! workspace never builds the adapter, and GBS's gate was not run for two cycles. The pair
//! was broken for the length of a cycle and no check on either side could report it.
//!
//! A public trait with an out-of-tree implementor is an API. This test is the only thing in
//! this repository that knows that.
//!
//! # Why this is allowed to shell out when the verdict tests are not
//!
//! T-03 removed `cargo run` from the verdict suites because a build failure there was read as
//! a *compiler refusal* — the subprocess's exit code answered a question about the language.
//! Here the subprocess's exit code answers a question about the build, which is the thing
//! being asked. The hazard T-03 names still applies in one direction, and is handled: a cargo
//! that cannot run at all is `BLOCKED`, never a failure, and the two are told apart by
//! looking for rustc's own `error[E` marker in the output rather than by the exit code alone.

use std::path::PathBuf;
use std::process::Command;

/// The GBS checkout, from `GBS_ROOT` or a sibling directory. `None` means "not here".
fn gbs_root() -> Option<PathBuf> {
    if let Ok(raw) = std::env::var("GBS_ROOT") {
        let p = PathBuf::from(&raw);
        return p
            .join("crates/gbs-nilestream/Cargo.toml")
            .exists()
            .then_some(p);
    }
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = here.parent()?.parent()?;
    for name in ["GBS", "gbs"] {
        let p = root.parent()?.join(name);
        if p.join("crates/gbs-nilestream/Cargo.toml").exists() {
            return Some(p);
        }
    }
    None
}

#[test]
fn the_downstream_adapter_still_compiles_against_this_trees_public_traits() {
    let Some(gbs) = gbs_root() else {
        // **A missing GBS is a refusal, not a skip.**
        //
        // This printed `SKIPPED` and returned, so `cargo test` reported `ok`. A reader of a
        // *transcript* could tell the check had not run; a reader of the gate could not, and
        // the gate is what decides. That is precisely how the pair stayed broken for a whole
        // cycle: this workspace never builds the adapter, GBS's own gate was not run, and
        // both sides were green.
        //
        // Standalone development on a machine with no GBS checkout is a real need, so there
        // is an opt-out — but it must be *asked for*, and what it produces is a NOT RUN
        // line that the audit harness refuses to accept as a pass.
        if std::env::var("NILES_NO_GBS").is_ok() {
            println!(
                "PAIRED ADAPTER GATE: NOT RUN (NILES_NO_GBS is set). This is an opt-out for \
                 standalone development and it does not satisfy a paired audit gate. The \
                 audit harness treats this line as a section that did not run."
            );
            return;
        }
        panic!(
            "PAIRED ADAPTER GATE: no GBS checkout found, so the only check that notices when \
             a change to a public trait breaks its out-of-tree implementor did not run — and \
             a check that does not run must not report `ok`.\n\n\
             Point it at one with `GBS_ROOT=/path/to/GBS`, or place a checkout at ../GBS \
             beside this repository.\n\n\
             To develop without one, set `NILES_NO_GBS=1`. That is an opt-out, not a pass: \
             it prints a NOT RUN line and the audit harness refuses a transcript containing \
             it."
        );
    };

    // **By manifest path, not by `-p`.** `gbs-nilestream` is deliberately not a default
    // member of the GBS workspace — it needs a Nilestream checkout beside it, and making that
    // mandatory would turn an optional dependency into a required one. So `-p gbs-nilestream`
    // from the workspace root fails with "did not match any packages", which is a cargo
    // complaint and not an adapter defect. GBS's own `make adapter` uses `--manifest-path`
    // and so does this.
    //
    // A target directory of its own. `CARGO_TARGET_DIR`, if set, belongs to *this* build; a
    // nested cargo sharing it contends for the build-directory lock, which is precisely the
    // failure T-03 removed from the verdict suites.
    let manifest = gbs.join("crates/gbs-nilestream/Cargo.toml");
    let target = gbs.join("crates/gbs-nilestream/target");
    // **`--all-targets`, because the library was never the whole downstream surface.**
    //
    // This check built the adapter crate's *library* and reported it green while GBS's own
    // gate was red: `tests/over_the_wire.rs` wrapped the engine in a `Mutex`, which T-06 had
    // replaced with an `RwLock`, and a test target is as much a consumer of these traits as
    // the library is. A downstream integration test is in fact the *more* likely place for a
    // break, because it is where the daemon is assembled rather than merely linked against.
    //
    // `build --all-targets` compiles tests and benches without running them, which is the
    // right amount: this check answers "does the API still fit", and running GBS's suite here
    // would answer a question GBS's own gate already asks, slowly, with its own fixtures.
    let out = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .arg("build")
        .arg("--all-targets")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(&manifest)
        .current_dir(&gbs)
        .env("CARGO_TARGET_DIR", &target)
        .env_remove("RUSTFLAGS")
        .output();

    let out = match out {
        Ok(o) => o,
        Err(e) => panic!(
            "BLOCKED-adapter: cargo could not be spawned in {}: {e}. This is not a statement \
             about the adapter.",
            gbs.display()
        ),
    };
    if out.status.success() {
        return;
    }

    let stderr = String::from_utf8_lossy(&out.stderr);
    // **A compiler error is the finding; anything else is BLOCKED.** `error[E….]` is rustc
    // saying the code is wrong. A missing registry, a locked target directory or a broken
    // toolchain produce a non-zero exit with no such marker, and reporting those as a broken
    // adapter is the T-03 defect in a new file.
    assert!(
        !stderr.contains("error[E"),
        "the downstream adapter no longer compiles against this tree's public traits. This \
         workspace changed something `gbs-nilestream` implements — `Base` and `Serving` are \
         the two — and the change needs a matching commit in GBS recording this tree's SHA. \
         rustc said:\n{}",
        stderr
            .lines()
            .filter(|l| l.contains("error") || l.starts_with("  -->"))
            .take(12)
            .collect::<Vec<_>>()
            .join("\n")
    );
    panic!(
        "BLOCKED-adapter: cargo exited {} with no rustc error marker, so this says nothing \
         about the adapter. Last lines:\n{}",
        out.status,
        stderr.lines().rev().take(6).collect::<Vec<_>>().join("\n")
    );
}
