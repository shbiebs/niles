//! `SPEC-LANGUAGE.md`'s Part V is counted from its own sections.
//!
//! Part V was a hand-maintained table beside twenty sections that each declare a status, and
//! it had drifted three ways at once (F-35): it counted four requirements — L-3, L-4, L-7,
//! L-14 — as **Built** when no section for them existed anywhere in the document; its prose
//! said "Ten of twenty-four built" while its own rows summed to eleven; and L-8/L-9 read
//! `Specified` beside a section describing a checker that `ROADMAP.md` marks BUILT.
//!
//! A summary of sections that is not computed from the sections is a second document.

#[path = "../src/bin/gen-spec-conformance.rs"]
mod gen;

use std::path::PathBuf;

fn spec() -> (PathBuf, String) {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("the repository root is two levels above this crate")
        .join("docs/SPEC-LANGUAGE.md");
    let text =
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
    (p, text)
}

#[test]
fn part_five_is_what_the_sections_say() {
    let (path, doc) = spec();
    let block = gen::render(&doc).unwrap_or_else(|e| panic!("{e}"));
    let want = format!("{}\n\n{block}\n{}", gen::BEGIN, gen::END);
    let start = doc
        .find(gen::BEGIN)
        .unwrap_or_else(|| panic!("{} has no conformance marker", path.display()));
    let end = doc
        .find(gen::END)
        .unwrap_or_else(|| panic!("{} has no closing marker", path.display()));
    assert_eq!(
        &doc[start..end + gen::END.len()],
        want,
        "Part V is stale; run `cargo run -p niles-lang --bin gen-spec-conformance`"
    );
}

#[test]
fn every_requirement_the_summary_counts_has_a_section() {
    let (_, doc) = spec();
    let reqs = gen::parse(&doc).expect("the sections parse");
    let ids: Vec<String> = reqs.iter().flat_map(|(_, r)| r.ids.clone()).collect();
    // L-1 through L-24, each exactly once.
    for n in 1..=24 {
        let id = format!("L-{n}");
        let count = ids.iter().filter(|i| **i == id).count();
        assert_eq!(
            count, 1,
            "`{id}` appears in {count} section headings; every requirement needs exactly one, \
             and L-3, L-4, L-7 and L-14 had none while Part V counted them Built"
        );
    }
    assert_eq!(ids.len(), 24);
}

#[test]
fn a_section_with_no_status_is_an_error_rather_than_a_default() {
    // The negative control on the generator itself. Without it, a missing status could be
    // silently treated as `Specified` and the summary would look right while being invented.
    let doc = "## Part I — Nothing\n\n### L-1 A thing\n\nNo status here.\n";
    let e = gen::parse(doc).expect_err("a section with no status must be an error");
    assert!(e.contains("declares no `Status:`"), "{e}");
}

#[test]
fn the_status_words_are_the_four_the_summary_counts() {
    let (_, doc) = spec();
    let reqs = gen::parse(&doc).expect("the sections parse");
    for (_, r) in &reqs {
        assert!(
            ["Built", "Partial", "Specified", "Adopt"]
                .iter()
                .any(|w| r.status.eq_ignore_ascii_case(w)),
            "`{}` declares status `{}`, which Part V has no column for — a status outside \
             the four is a status the summary silently drops",
            r.ids.join(" / "),
            r.status
        );
    }
}
