//! **The falsification test.**
//!
//! `ARCHITECTURE.md` §1 makes a claim that the thesis states and does not test: banking is a
//! *library over a fully general relational core*, and no banking product requires a change
//! to the kernel. Thesis §11.3 gives its refutation condition:
//!
//! > A banking product from §6.24 that cannot be expressed without a kernel change would
//! > falsify the "banking as a library" claim and, with it, part of the generality thesis.
//!
//! A claim of that shape is worthless if the only thing stopping a violation is somebody
//! noticing in review. This file reads the Cargo manifests and fails the build if the
//! layering is breached, which converts the refutation condition into a build failure: a
//! product that needs to reach past the mechanisms layer **cannot be written** without
//! breaking a named test whose failure message says exactly which claim it falsifies.
//!
//! The layering, from `ARCHITECTURE.md` §5:
//!
//! ```text
//! gbs-products     compositions only — MUST NOT depend on anything below gbs-mechanisms
//!       ▼
//! gbs-mechanisms   M2..M7, as patterns of M1
//!       ▼
//! gbs-kernel       M1: postings, per-currency conservation, the chart
//!       ▼
//! nilestream-*     ledger, REV runtime, consensus  ·  niles-lang, niles-ir
//! ```
//!
//! # Why this is a test and not a lint
//!
//! A lint is advisory and a `cargo deny` configuration is a separate artifact somebody has
//! to remember to run. This is in the same `cargo test` that everything else is in, so the
//! claim is checked by the same command that checks conservation. That matters more than
//! the mechanism: a falsification condition nobody executes is a promise, not a test.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Crates below the mechanisms layer. A product depending on any of these has reached past
/// the library boundary, which is the falsification.
const BELOW_THE_LINE: &[&str] = &[
    "nilestream-ledger",
    "nilestream-core",
    "nilestream-storage",
    "nilestream-server",
    "nilestream-consensus",
    "nilestream-optimizer",
    "nilestream-lineage",
    "niles-ir",
    "niles-lang",
    "niles-interp",
    "proto-engine",
];

fn gbs_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is `gbs/crates/gbs-products`; the GBS root is two levels up.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gbs-products lives at gbs/crates/gbs-products")
        .to_path_buf()
}

/// The `[dependencies]` section of a manifest, as crate names.
///
/// A deliberately small parser rather than a TOML dependency: this test exists to police
/// the dependency list, and giving it a dependency of its own to police would be a poor
/// joke. It reads the `[dependencies]` table and stops at the next section header.
fn dependencies_of(crate_dir: &Path) -> BTreeSet<String> {
    let manifest = crate_dir.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", manifest.display()));

    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            // Catch dev- and build-dependencies too. A product that reaches past the line
            // only in its tests has still reached past it, and the fixture it built there
            // would be evidence that the layering does not hold.
            in_deps = matches!(
                t,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
            );
            continue;
        }
        if !in_deps || t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = t.split_once('=') {
            deps.insert(name.trim().trim_matches('"').to_string());
        }
    }
    deps
}

#[test]
fn products_depend_on_nothing_below_the_mechanisms_layer() {
    // **The test the architecture stands or falls on.**
    let deps = dependencies_of(&gbs_root().join("crates/gbs-products"));

    let breaches: Vec<&String> =
        deps.iter().filter(|d| BELOW_THE_LINE.contains(&d.as_str())).collect();

    assert!(
        breaches.is_empty(),
        "\n\
         LAYERING BREACH — this falsifies the claim of ARCHITECTURE.md §1.\n\n\
         `gbs-products` now depends on: {breaches:?}\n\n\
         Products are compositions of mechanisms. A product that must reach past\n\
         `gbs-mechanisms` is a product that cannot be expressed as a library over the\n\
         general core, which is the refutation condition thesis §11.3 states for the\n\
         \"banking as a library\" claim.\n\n\
         Two honest responses, in order of preference:\n\
           1. Express what the product needs as a *mechanism* (M2..M7 or a new one), and\n\
              record the addition in ARCHITECTURE.md §3 — an eighth mechanism is a design\n\
              change worth noting, not a routine extension.\n\
           2. If it genuinely cannot be a mechanism, record the falsification in the\n\
              thesis and amend the claim. Do not delete this test.\n"
    );

    // And the positive half: a product must actually be built on the mechanisms, or the
    // constraint above is satisfied vacuously by a crate that does nothing.
    assert!(
        deps.contains("gbs-mechanisms"),
        "gbs-products must be built on the mechanisms layer; it depends on {deps:?}"
    );
}

#[test]
fn mechanisms_depend_on_the_kernel_and_not_on_the_engine() {
    // The same discipline one level down. A mechanism is a *pattern of posting sets*, so it
    // needs M1 and no engine access whatever. If a mechanism acquired a ledger dependency,
    // a product could reach the engine through it and this file's main test would pass
    // while the architecture had already been breached — the exact way a layering check
    // becomes theatre.
    let deps = dependencies_of(&gbs_root().join("crates/gbs-mechanisms"));

    let breaches: Vec<&String> =
        deps.iter().filter(|d| BELOW_THE_LINE.contains(&d.as_str())).collect();

    assert!(
        breaches.is_empty(),
        "\n\
         `gbs-mechanisms` depends on {breaches:?}, which is below its layer.\n\n\
         A mechanism is a pattern of posting sets. It needs the kernel and nothing else —\n\
         if it needs the engine, the thing it is doing belongs in the engine.\n\n\
         This matters even though `gbs-products` still passes its own check: a product can\n\
         reach the engine *through* a mechanism, so a breach here makes the test above\n\
         vacuous.\n"
    );

    assert!(deps.contains("gbs-kernel"), "mechanisms are built on the kernel: {deps:?}");
}

#[test]
fn the_kernel_sits_on_the_ledger_and_on_nothing_else() {
    // The kernel is the one layer that may see the engine, and it may see exactly one part
    // of it. Every dependency added here makes the general core less general, which is why
    // the list is asserted rather than merely kept short.
    let deps = dependencies_of(&gbs_root().join("crates/gbs-kernel"));

    let engine: BTreeSet<&String> =
        deps.iter().filter(|d| BELOW_THE_LINE.contains(&d.as_str())).collect();

    assert_eq!(
        engine.len(),
        1,
        "the kernel should see exactly one engine crate; it sees {engine:?}"
    );
    assert!(
        deps.contains("nilestream-ledger"),
        "and that one crate is the ledger: {deps:?}"
    );

    // In particular: not the REV runtime. A kernel that could read a materialized view
    // would be a kernel with a second source of truth about balances, which is the thing
    // this whole design exists to not have.
    assert!(
        !deps.contains("nilestream-core"),
        "the kernel must not see the read-model runtime; a balance is a fold, not a lookup"
    );
}

#[test]
fn the_layering_check_can_actually_fail() {
    // Guarding the guard. A layering test that could not detect a breach would pass
    // forever and prove nothing — and this is precisely the failure the Appendix E
    // bootstrap gate had on its first run, where the scope filter silently excluded the
    // hardest cases. Feed the parser a manifest that *does* breach the layering and
    // confirm it is seen.
    let dir = std::env::temp_dir().join("gbs-layering-negative-control");
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\n\
         name = \"pretend-product\"\n\n\
         [dependencies]\n\
         gbs-mechanisms = { path = \"../gbs-mechanisms\" }\n\
         nilestream-core = { path = \"../../../crates/nilestream-core\" }\n",
    )
    .expect("write control manifest");

    let deps = dependencies_of(&dir);
    assert!(deps.contains("gbs-mechanisms"));
    assert!(
        deps.iter().any(|d| BELOW_THE_LINE.contains(&d.as_str())),
        "the check must detect a manifest that reaches past the line"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dev_dependencies_are_policed_too() {
    // A product that reaches past the line only in its tests has still reached past it.
    // The fixture it built there would be evidence that the layering does not hold in
    // practice, and a check that ignored `[dev-dependencies]` would be easy to route
    // around without meaning to.
    let dir = std::env::temp_dir().join("gbs-layering-dev-control");
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\n\
         name = \"pretend\"\n\n\
         [dependencies]\n\
         gbs-mechanisms = { path = \"x\" }\n\n\
         [dev-dependencies]\n\
         nilestream-ledger = { path = \"y\" }\n",
    )
    .expect("write control manifest");

    let deps = dependencies_of(&dir);
    assert!(
        deps.contains("nilestream-ledger"),
        "a dev-dependency below the line must be seen: {deps:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_crate_named_below_the_line_actually_exists() {
    // Otherwise the list decays: a crate renamed in the workspace would silently stop being
    // policed, and the test would keep passing while checking one fewer thing every time.
    let workspace = gbs_root().parent().expect("gbs sits in the workspace root").to_path_buf();
    for name in BELOW_THE_LINE {
        let dir = workspace.join("crates").join(name);
        assert!(
            dir.join("Cargo.toml").exists(),
            "`{name}` is on the below-the-line list but there is no crate at {}. \
             Either the crate was renamed — in which case update this list, or the check \
             silently stops policing it — or the entry is stale.",
            dir.display()
        );
    }
}
