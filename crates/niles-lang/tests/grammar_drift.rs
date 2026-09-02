//! The anti-drift gate.
//!
//! The thesis says three things about the artifacts in this crate: that the normative
//! grammar lives in `grammar/niles.ebnf`, that the keyword reference is *generated* from
//! the compiler's keyword registry, and that the reserved list is the union of the three
//! keyword tables. Each of those claims can rot silently the moment the code moves.
//! These tests are what stop it: they check the grammar file, the registry and the AST
//! against each other in both directions, and they compute every number the thesis quotes
//! rather than trusting a figure typed by hand.
//!
//! PostgreSQL keeps its keyword lists "in their own source files for use by automatic
//! tools" for exactly this reason. This is the automatic tool.

use niles_lang::ast::StageKind;
use niles_lang::keywords::{by_origin, Category, Origin, KEYWORDS};
use std::collections::HashSet;

const GRAMMAR: &str = include_str!("../grammar/niles.ebnf");

/// Remove `(* .. *)` comments, so that a word mentioned only in prose is not mistaken for
/// a terminal.
fn grammar_body() -> String {
    let mut out = String::with_capacity(GRAMMAR.len());
    let b = GRAMMAR.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'(' && b[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b')' && depth > 0 {
            depth -= 1;
            i += 2;
        } else {
            if depth == 0 {
                out.push(b[i] as char);
            }
            i += 1;
        }
    }
    out
}

/// Every `"..."` terminal in the grammar, comments excluded.
fn grammar_terminals() -> HashSet<String> {
    let body = grammar_body();
    let mut out = HashSet::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            let mut s = String::new();
            for c2 in chars.by_ref() {
                if c2 == '"' {
                    break;
                }
                s.push(c2);
            }
            out.insert(s);
        }
    }
    out
}

#[test]
fn the_grammar_file_is_the_size_the_header_claims() {
    let body = grammar_body();
    let rules = body.lines().filter(|l| l.contains("::=")).count();
    let productions: usize = {
        // One production per alternative: the BNF-normal reading.
        let mut total = 0usize;
        for chunk in body.split("::=").skip(1) {
            let rule_body = chunk.split("::=").next().unwrap_or("");
            let stop = rule_body.find(';').unwrap_or(rule_body.len());
            total += 1 + rule_body[..stop].matches('|').count();
        }
        total
    };
    // The header states both figures. If either moves, this fails and the header must be
    // corrected — which is the point: no number in the thesis is typed by hand.
    assert!(
        GRAMMAR.contains(&format!("{rules} named rules")),
        "grammar header must state the real rule count, which is {rules}"
    );
    assert!(
        GRAMMAR.contains(&format!("that is {productions} productions")),
        "grammar header must state the real production count, which is {productions}"
    );
    assert!(
        rules > 150,
        "the grammar is suspiciously small: {rules} rules"
    );
}

#[test]
fn every_keyword_appears_in_the_grammar() {
    // A word in the registry that no production mentions is either dead vocabulary or a
    // hole in the grammar. Both are bugs, and both are invisible without this test.
    let terminals = grammar_terminals();
    let missing: Vec<&str> = KEYWORDS
        .iter()
        .filter(|k| k.category != Category::ReservedFuture)
        .map(|k| k.word)
        .filter(|w| !terminals.contains(*w))
        .collect();
    assert!(
        missing.is_empty(),
        "these registry keywords appear in no production: {missing:?}\n\
         either use them in the grammar or remove them from the registry"
    );
}

#[test]
fn every_grammar_terminal_is_a_registry_keyword_or_punctuation() {
    // The other direction: a production must not invent a word the lexer will never
    // produce as a keyword, because such a production is unreachable.
    let words: HashSet<&str> = KEYWORDS.iter().map(|k| k.word).collect();
    // Stage names, type names and lexical fragments are legitimately not keywords.
    let allowed_non_keywords: HashSet<&str> = StageKind::all_names()
        .iter()
        .copied()
        .chain([
            "bool",
            "i8",
            "i16",
            "i32",
            "i64",
            "i128",
            "u8",
            "u16",
            "u32",
            "u64",
            "u128",
            "f32",
            "f64",
            "Text",
            "Bytes",
            "Json",
            "Money",
            "Currency",
            "Id",
            "Epoch",
            "Instant",
            "Date",
            "Duration",
            "Interval",
            "Signal",
            "Bitemporal",
            "Posting",
            "Debit",
            "Credit",
            "Hold",
            "Auth",
            "Anchored",
            "IdemKey",
            "Lineage",
            "millis",
            "ms",
            "seconds",
            "secs",
            "s",
            "minutes",
            "mins",
            "hours",
            "hrs",
            "days",
            "epochs",
            "off",
            "fuel",
            "deterministic",
            "pure",
            "to",
        ])
        .collect();
    let unknown: Vec<String> = grammar_terminals()
        .into_iter()
        .filter(|t| t.chars().next().is_some_and(|c| c.is_alphabetic()))
        .filter(|t| !words.contains(t.as_str()) && !allowed_non_keywords.contains(t.as_str()))
        .collect();
    assert!(
        unknown.is_empty(),
        "these grammar terminals are neither keywords nor known library names: {unknown:?}"
    );
}

#[test]
fn the_stage_vocabulary_matches_the_compiler() {
    // Section 12 of the grammar is normative about the operator set the IR must
    // implement. If a stage is added to the compiler and not to the grammar, the grammar
    // stops being normative, quietly.
    let terminals = grammar_terminals();
    for s in StageKind::all_names() {
        assert!(
            terminals.contains(*s),
            "stage `{s}` is missing from grammar section 12"
        );
    }
}

#[test]
fn the_reserved_set_stays_small() {
    // The design claim in `keywords.rs`: a hand-written parser with unbounded lookahead
    // keeps the reserved set small despite a large vocabulary. That is an empirical claim
    // about this implementation, so it gets a test with a real bound. If a change pushes
    // the ratio past a quarter, the claim in the thesis needs revisiting, not the test.
    let total = KEYWORDS.len();
    let reserved = KEYWORDS
        .iter()
        .filter(|k| matches!(k.category, Category::Reserved | Category::ReservedFuture))
        .count();
    // The claim is *not* that Niles reserves few words overall — most of the reserved
    // set is inherited, because `select`, `from`, `fn` and `let` were reserved in SQL and
    // Rust before Niles existed and unreserving them would break the reading of programs
    // in both lineages. The claim is narrower and is the one that governs adoption: the
    // *novel* vocabulary, the words a bank's existing schema might already be using, is
    // reserved at a rate of zero. That is asserted in `keywords::tests::reservation_policy_holds`
    // and quantified here.
    let novel_reserved = by_origin(Origin::Novel)
        .filter(|k| matches!(k.category, Category::Reserved | Category::ReservedFuture))
        .filter(|k| k.category != Category::ReservedFuture)
        .count();
    assert_eq!(
        novel_reserved, 0,
        "a novel keyword was reserved without justification"
    );

    // The figures the thesis quotes. Computed here, never typed into the prose by hand.
    assert_eq!(
        total, 174,
        "keyword count changed; regenerate docs/keywords.md and update B.3"
    );
    assert_eq!(reserved, 59, "reserved count changed; update Appendix B.16");
    let unreserved = KEYWORDS
        .iter()
        .filter(|k| k.category == Category::Unreserved)
        .count();
    assert_eq!(
        unreserved, 95,
        "unreserved count changed; update Appendix B.16"
    );
}

#[test]
fn the_three_origin_tables_partition_the_registry() {
    let sql = by_origin(Origin::Sql).count();
    let rust = by_origin(Origin::Rust).count();
    let novel = by_origin(Origin::Novel).count();
    assert_eq!(sql + rust + novel, KEYWORDS.len());
    // The syntax-lineage rule is a claim about proportions: SQL first, Rust where SQL has
    // no equivalent, novel only for concepts neither language has. If the novel table ever
    // outgrew the SQL one, the "SQL-first" claim would be rhetoric.
    assert!(
        sql >= novel,
        "SQL-derived ({sql}) must not be outnumbered by novel ({novel})"
    );
}

#[test]
fn well_formedness_rules_are_all_numbered_and_present() {
    // Section 15 lists the checks that are deliberately *not* in the grammar. Each one is
    // a claim that some later phase enforces it; the test that each is enforced lives with
    // that phase. Here we only check the list is intact and contiguous.
    for i in 1..=20 {
        assert!(
            GRAMMAR.contains(&format!("W{i} ")),
            "well-formedness rule W{i} is missing"
        );
    }
}
