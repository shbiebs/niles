//! **The manual's coverage map and decision catalogue hold together** (cycle 17, C17-13).
//!
//! * Every row of `docs/manual/coverage.csv` names a chapter that exists, or says `not yet`.
//! * Every id in `docs/manual/decision-ids.txt` (copied from the author's decision records, not
//!   from the chapter) has a card in `docs/manual/06-decisions.md`, and every card carries the
//!   four parts the catalogue promises: alternatives, criterion, evidence, and what would
//!   reverse it.

use std::collections::BTreeMap;
use std::path::PathBuf;

fn manual() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/manual")
}

fn read(name: &str) -> String {
    std::fs::read_to_string(manual().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn every_coverage_row_names_a_chapter_that_exists_or_says_not_yet() {
    let csv = read("coverage.csv");
    let mut lines = csv.lines();
    assert_eq!(lines.next(), Some("area,mechanism,chapter"), "the header");
    let mut rows = 0;
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let chapter = line.rsplit(',').next().expect("a chapter column").trim();
        rows += 1;
        if chapter == "not yet" {
            continue;
        }
        assert!(
            chapter.ends_with(".md") && manual().join(chapter).is_file(),
            "`{line}` names {chapter}, which is not a chapter of docs/manual"
        );
    }
    assert!(
        rows >= 30,
        "the map covers the language, the engine, GBS and the front end"
    );
    for area in ["language,", "engine,", "gbs,", "front end,"] {
        assert!(
            csv.lines().any(|l| l.starts_with(area)),
            "no row for {area}"
        );
    }
}

/// The catalogue's cards: id → the card's text, up to the next card or section.
fn cards() -> BTreeMap<String, String> {
    let text = read("06-decisions.md");
    let mut out = BTreeMap::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("### ") {
            if let Some((id, body)) = current.take() {
                out.insert(id, body);
            }
            let id = rest.split_whitespace().next().unwrap_or("").to_string();
            current = Some((id, String::new()));
        } else if line.starts_with("## ") {
            if let Some((id, body)) = current.take() {
                out.insert(id, body);
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((id, body)) = current {
        out.insert(id, body);
    }
    out
}

fn ids() -> Vec<String> {
    read("decision-ids.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

#[test]
fn every_decision_id_has_a_complete_card() {
    let cards = cards();
    let ids = ids();
    assert!(ids.len() >= 64, "the records' ids: {}", ids.len());
    for id in &ids {
        let card = cards
            .get(id)
            .unwrap_or_else(|| panic!("decision {id} has no card in 06-decisions.md"));
        for part in [
            "- **Alternatives:**",
            "- **Criterion:**",
            "- **Evidence:**",
            "- **What would reverse it:**",
        ] {
            assert!(card.contains(part), "the card for {id} lacks `{part}`");
        }
    }
    for id in cards.keys() {
        assert!(
            ids.contains(id),
            "06-decisions.md has a card {id} that decision-ids.txt does not list"
        );
    }
}
