//! The "blessing" test, in rustc's `compiletest` style: the checked-in generated file
//! must equal what the generator would produce right now.
//!
//! Without this, "generated from the compiler's keyword registry" is a claim about how the
//! file was *once* produced. With it, the claim holds continuously, and a contributor who
//! adds a keyword and forgets to regenerate the reference is told so by CI rather than by
//! a reader of the thesis.

use std::path::PathBuf;

#[path = "../src/bin/gen-keyword-ref.rs"]
mod gen;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn the_checked_in_reference_matches_the_registry() {
    let path = repo_root().join("docs/keywords.md");
    let expected = gen::render();
    let actual = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        actual.trim_end(),
        expected.trim_end(),
        "\ndocs/keywords.md is out of date. Regenerate it:\n\
         \n    cargo run -p niles-lang --bin gen-keyword-ref\n"
    );
}

#[test]
fn every_keyword_has_an_entry() {
    let out = gen::render();
    for k in niles_lang::keywords::KEYWORDS {
        assert!(
            out.contains(&format!("| `{}` |", k.word)),
            "`{}` has no entry in the generated reference",
            k.word
        );
    }
}
