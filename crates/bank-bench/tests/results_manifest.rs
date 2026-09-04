//! Every file under `results/` is accounted for, exactly once.
//!
//! # Why this exists
//!
//! `make reproduce` ends in `git diff --exit-code -- results/`. That is a strong check for a
//! file the recipe regenerates and **no check at all** for one it does not: the file is
//! unchanged because nothing wrote it, and the diff is clean because the file is unchanged.
//! Measured at the time this was written, `make reproduce` regenerated **14 of 51** files
//! under `results/`, so the gate was passing vacuously for the other 37 — including
//! `E16-wallclock.md`, which carries the performance contract.
//!
//! The manifest does not make those files reproducible. It makes the *distinction* explicit
//! and enforced: a reader can see which numbers a green `make reproduce` actually vouches
//! for, and a file cannot enter or leave `results/` without someone saying which kind it is.
//!
//! # What is checked
//!
//! `results/MANIFEST.csv` and `.gitignore` must **together** account for every file on disk
//! under `results/`, and every manifest entry must exist. `.gitignore` is read rather than
//! duplicated, so the deliberately-untracked scratch copies (the benchmark's own `--out`
//! directory writes one beside its CSVs) stay exempt in exactly one place.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The four classes, and what each promises a reader.
const CLASSES: [&str; 4] = [
    // Same bytes on any host with the pinned toolchain. `make reproduce` regenerates it, so
    // the diff means something.
    "byte-deterministic",
    // Same bytes within one toolchain and target. Allocation *counts* are exact across
    // hosts; allocation *bytes* are not.
    "toolchain-scoped",
    // Acquired by measurement. Regenerating it elsewhere is expected to differ, so
    // `make reproduce` must NOT regenerate it and `--publish` is required to write it.
    "machine-dependent",
    // Produced once by something no longer wired, or by hand. Retained as evidence and
    // cited; not regenerable.
    "historical",
];

struct Entry {
    path: String,
    class: String,
    regenerated_by: String,
}

fn manifest() -> Vec<Entry> {
    let p = repo_root().join("results/MANIFEST.csv");
    let text =
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() || line.starts_with("path,") {
            continue;
        }
        // The command may contain commas (`--connections 1,2,4`), so split only twice.
        let mut it = line.splitn(3, ',');
        let (path, class, cmd) = (
            it.next().expect("path"),
            it.next()
                .unwrap_or_else(|| panic!("no class in manifest line: {line}")),
            it.next()
                .unwrap_or_else(|| panic!("no producer in manifest line: {line}")),
        );
        out.push(Entry {
            path: path.to_string(),
            class: class.to_string(),
            regenerated_by: cmd.to_string(),
        });
    }
    out
}

/// The `results/` paths `.gitignore` exempts, read from the file rather than copied.
fn ignored() -> BTreeSet<String> {
    let p = repo_root().join(".gitignore");
    std::fs::read_to_string(&p)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("results/"))
        .map(|l| l.trim_start_matches("results/").to_string())
        .collect()
}

fn walk(dir: &Path, base: &Path, into: &mut BTreeSet<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, base, into);
        } else if let Ok(rel) = p.strip_prefix(base) {
            into.insert(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn on_disk() -> BTreeSet<String> {
    let base = repo_root().join("results");
    let mut out = BTreeSet::new();
    walk(&base, &base, &mut out);
    out.remove("MANIFEST.csv");
    out
}

#[test]
fn every_results_file_is_classified_and_every_classified_file_exists() {
    let m = manifest();
    let listed: BTreeSet<String> = m.iter().map(|e| e.path.clone()).collect();
    assert_eq!(
        listed.len(),
        m.len(),
        "a path is listed twice in results/MANIFEST.csv"
    );

    let disk = on_disk();
    let exempt = ignored();

    let unlisted: Vec<&String> = disk
        .iter()
        .filter(|f| !listed.contains(*f) && !exempt.contains(*f))
        .collect();
    assert!(
        unlisted.is_empty(),
        "under results/ and accounted for by neither MANIFEST.csv nor .gitignore: {unlisted:?}.\n\
         Add each to the manifest with its class and the command that regenerates it, or to \
         .gitignore if it is scratch. A results file nobody has classified is a number nobody \
         can say the provenance of."
    );

    let missing: Vec<&String> = listed.iter().filter(|f| !disk.contains(*f)).collect();
    assert!(
        missing.is_empty(),
        "listed in results/MANIFEST.csv and absent from disk: {missing:?}.\n\
         Either the file was deleted and its entry should go with it, or a producer stopped \
         writing it."
    );
}

#[test]
fn every_class_is_one_of_the_four_and_carries_a_producer() {
    for e in manifest() {
        assert!(
            CLASSES.contains(&e.class.as_str()),
            "`{}` has class `{}`, which is not one of {CLASSES:?}",
            e.path,
            e.class
        );
        assert!(
            !e.regenerated_by.trim().is_empty(),
            "`{}` has no producer; write the exact command, or `none` if it is historical",
            e.path
        );
        // The two halves of the distinction this file exists to keep.
        if e.class == "historical" {
            assert_eq!(
                e.regenerated_by.trim(),
                "none",
                "`{}` is historical, so its producer must be `none` — a historical file with \
                 a command is either not historical or the command no longer works",
                e.path
            );
        } else {
            assert_ne!(
                e.regenerated_by.trim(),
                "none",
                "`{}` is `{}` rather than historical, so it must name the command that \
                 produces it",
                e.path,
                e.class
            );
        }
    }
}

/// **A machine-dependent file must not be regenerated by `make reproduce`.**
///
/// The rule the manifest's classes encode, checked against the recipe rather than trusted:
/// if `make reproduce` regenerated an acquired measurement it would overwrite evidence with
/// whatever this host happens to do, and the diff that follows would then compare a
/// published number against a re-measurement — which is how a contract row silently becomes
/// a property of the machine that last ran the gate.
#[test]
fn the_reproduce_recipe_regenerates_no_machine_dependent_result() {
    let mk = std::fs::read_to_string(repo_root().join("Makefile")).expect("Makefile");
    let recipe: String = mk
        .lines()
        .skip_while(|l| !l.starts_with("reproduce:"))
        .take_while(|l| l.starts_with("reproduce:") || l.starts_with('\t'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!recipe.is_empty(), "no `reproduce:` target in the Makefile");

    for e in manifest() {
        if e.class != "machine-dependent" {
            continue;
        }
        assert!(
            !recipe.contains(&e.regenerated_by),
            "`make reproduce` runs `{}`, which produces the machine-dependent `{}`. An \
             acquired measurement must be acquired by an explicit command, never by the \
             reproduction gate.",
            e.regenerated_by,
            e.path
        );
    }
    // `--publish` is what writes a committed machine-dependent artefact; the recipe must
    // not carry it.
    assert!(
        !recipe.contains("--publish"),
        "`make reproduce` passes `--publish`, so running the gate overwrites committed \
         measurements"
    );
}
