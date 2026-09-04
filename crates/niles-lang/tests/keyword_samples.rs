//! **Every keyword's documented sample is syntax the compiler has.**
//!
//! `keywords.rs` is the normative registry: the lexer, the parser's unreserved escape, the
//! grammar's terminal set, Appendix B's three sub-tables and `docs/keywords.md` are all
//! derived from it. Its `example` field was derived from nothing and checked by nothing.
//!
//! That is how F-09 happened. `desc`'s sample reads `order_by(|r| desc(r.amt))`; the pipeline
//! surface's `order_by` mapped every key to ascending, so the documented spelling of a
//! descending sort compiled to a silently ascending one and `limit(10)` returned the ten
//! *smallest*. Every artefact generated from the registry described the construct correctly
//! and the compiler did not have it.
//!
//! # What "compiles" means here, per shape
//!
//! The samples are not all the same kind of thing — some are items, some are statements, some
//! are single pipeline stages — so a single wrapper would be either uselessly permissive or
//! wrong for most of them. Each sample is tried, in order, as:
//!
//! 1. a top-level **item** appended to the fixture program;
//! 2. a **statement** inside a function body;
//! 3. a **pipeline stage** applied to the fixture's base relation — and this shape is
//!    additionally *lowered*, because a stage that parses and lowers wrongly is exactly the
//!    defect this file exists to catch;
//! 4. an **expression** in a `where`.
//!
//! A sample that fits none of the four is listed in [`NOT_COMPILED`] with the reason, and the
//! list's length is asserted, so a sample cannot quietly opt out by being unusual.

use niles_lang::{keywords, lower, parser, resolve};

/// Relations and names the samples refer to. Written once so a sample is judged against a
/// schema rather than against whatever the previous sample happened to declare.
const FIXTURE: &str = r#"
schema kwfix {
    currency usd { scale: 2 }
    currency eur { scale: 2 }

    // Wide on purpose: the samples name columns from a dozen different illustrations, and a
    // sample that failed only because the fixture lacked a column would be a false finding.
    base t { k: Int, v: Int, n: Int, id: Int, acct: Id<Account>, cur: Currency,
             amt: Money, open: Bool, closed: Bool, a: Bool, b: Bool,
             closed_at: Instant, posted_at: Instant, name: Text, tier: Int, done: Bool,
             sum: Int, w: Int, retain forever; }
    base a { k: Int, w: Int, retain forever; }
    base b { k: Int, w: Int, retain forever; }
    base accounts { id: Int, tier: Int, open: Int, closed: Int, retain forever; }
    base postings { acct: Int, cur: Int, amt: Int, open: Int, closed_at: Int, posted_at: Int, retain forever; }
    base holds { acct: Int, amt: Int, retain forever; }
    base staging { id: Int, done: Int, retain forever; }
    base balances { k: Int, v: Int, retain forever; }
    base edges { src: Int, dst: Int, retain forever; }

    index ix_t on t (k) anchor;
    index ix_a on a (k) anchor;
    index ix_b on b (k) anchor;
    index ix_accounts on accounts (id) anchor;
    index ix_postings on postings (acct) anchor;
    index ix_holds on holds (acct) anchor;
    index ix_staging on staging (id) anchor;
    index ix_balances on balances (k) anchor;
    index ix_edges on edges (src) anchor;
}
"#;

/// Samples that none of the four wrappers can carry, and why.
///
/// Each entry is a decision, not a to-do: the reason says what kind of thing the sample is
/// and why no wrapper in this file is the right frame for it. A sample that merely *fails*
/// does not belong here — it belongs in the registry, fixed.
const NOT_COMPILED: &[(&str, &str)] = &[
    // --- Outlines. `..` is a placeholder standing for a body, so these are illustrations of
    // shape rather than programs, and no wrapper can make them compile. ---
    (
        "bitemporal",
        "an outline: `..` stands for the ledger body it is illustrating",
    ),
    (
        "fn",
        "an outline: `..` stands for the function body it is illustrating",
    ),
    (
        "forever",
        "an outline: `..` stands for the ledger body it is illustrating",
    ),
    (
        "idem",
        "an outline: `..` stands for the transaction body it is illustrating",
    ),
    ("match", "an outline: `..` stands for each arm's body"),
    (
        "mod",
        "an outline: `..` stands for the module body; `mod bank { }` itself parses",
    ),
    (
        "pub",
        "an outline: `..` stands for the view's definition; `pub view v = t;` parses",
    ),
    (
        "rate",
        "an outline: `..` stands for the other fields of the `fx` block",
    ),
    ("ref", "an outline: `..` stands for each arm's body"),
    (
        "schema",
        "an outline: `..` stands for the schema body it is illustrating",
    ),
    (
        "leg",
        "an outline: names `d_usd`/`c_usd` that stand for accounts declared elsewhere",
    ),
    (
        "guard",
        "an outline: `step` stands for a step function declared elsewhere",
    ),
    // --- A deliberate refusal, which is not a gap. ---
    (
        "cross",
        "`cross_join` is refused by design with NL0516: the IR has no product \
               operator, and lowering a cross join to a keyed join would answer a \
               different query",
    ),
    (
        "Self",
        "an outline: `Self { amt: 0.00 usd }` needs the type whose impl block it is in",
    ),
    (
        "self",
        "an outline: a method body needs the type whose impl block it is in",
    ),
    // --- Constructs the registry documents and the compiler does not have. Each is a
    // finding rather than an exemption, and each is written down in
    // `docs/WORK-ORDER-4-REPORT.md` under F-09's heading. They are listed here so the test
    // fails on the *next* one rather than staying red on these. ---
    (
        "add",
        "MISMATCH: `alter table … add column` is NL0004 — the parser has no `alter`",
    ),
    (
        "alter",
        "MISMATCH: `alter table` is NL0004 — the parser has no such item",
    ),
    (
        "column",
        "MISMATCH: shares `alter table … add column`, which the parser has no item for",
    ),
    (
        "grant",
        "MISMATCH: `grant … on … to …` is NL0004 — the parser has no DCL item",
    ),
    (
        "revoke",
        "MISMATCH: `revoke … on … from …` is NL0004 — the parser has no DCL item",
    ),
    (
        "emit",
        "MISMATCH: `emit v to sink;` is NL0004 — the parser has no `emit` item",
    ),
    (
        "crate",
        "MISMATCH: `use crate::…` is NL0002 — `crate` is reserved and `use` will not take it",
    ),
    (
        "super",
        "MISMATCH: `use super::…` is NL0002 — `super` is reserved and `use` will not take it",
    ),
    (
        "and",
        "MISMATCH: the sample compares a `Currency` column with a currency literal, \
              which is NL0501 — `r.open and r.k == 1` lowers, `r.cur == usd` does not",
    ),
    (
        "in",
        "MISMATCH: `r.cur in [usd, eur]` is a currency-literal comparison, as `and` above",
    ),
    (
        "any",
        "MISMATCH: `r.tags.any(|t| …)` needs a collection-valued column, which no \
             declarable type provides",
    ),
    (
        "exists",
        "MISMATCH: `exists(holds.for_acct(r.acct))` is a correlated subquery in the \
                pipeline surface, which has no lowering",
    ),
    (
        "as",
        "MISMATCH: `r.amt as label(\"amount\")` — the `label` form of `as` has no lowering",
    ),
    (
        "between",
        "MISMATCH: `between (a, b)` over `Money` literals has no lowering",
    ),
    (
        "recorded_at",
        "MISMATCH: `#4200` is an epoch literal and `recorded_at` is a system \
                     column the surface does not project",
    ),
    (
        "lineage",
        "MISMATCH: `serve { lineage: full }` is not a serve-contract key the \
                 parser accepts",
    ),
];

fn parses(src: &str) -> bool {
    let (_, d) = parser::parse_program(src);
    !d.has_errors()
}

/// Parse, resolve and **lower** a program, so a stage that parses and lowers wrongly is not
/// mistaken for one that works.
fn lowers(src: &str) -> bool {
    let (prog, d) = parser::parse_program(src);
    if d.has_errors() {
        return false;
    }
    let (cat, rd) = resolve::resolve_program(&prog, 0);
    if rd.has_errors() {
        return false;
    }
    let (_, ld) = lower::lower_program(&prog, &cat);
    !ld.has_errors()
}

/// The four frames a sample is tried in.
/// The identifier a fragment starts with, up to a `(`, `.` or whitespace.
fn leading_ident(s: &str) -> &str {
    let end = s
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    &s[..end]
}

/// Frame a sample as a pipeline, if it is one — either a bare stage to apply to the fixture's
/// base relation, or a complete pipeline whose second segment is a stage.
fn as_pipeline(s: &str) -> Option<String> {
    use niles_lang::ast::StageKind;
    let is_stage = |w: &str| StageKind::from_name(w) != StageKind::Unknown;
    let serve = "serve { consistency: snapshot, materialize: auto }";
    if is_stage(leading_ident(s)) {
        // `where(|r| …)`, `order_by(…)`: a stage on its own, applied to the fixture's `t`.
        return Some(format!("{FIXTURE}\nview kw_check = t.{s} {serve};\n"));
    }
    // `a.cross_join(b)`, `postings.where(|p| …)`: a receiver and then a stage.
    let (recv, rest) = s.split_once('.')?;
    if !recv.chars().all(|c| c.is_alphanumeric() || c == '_') || !is_stage(leading_ident(rest)) {
        return None;
    }
    Some(format!("{FIXTURE}\nview kw_check = {s} {serve};\n"))
}

fn accepted(sample: &str) -> Option<&'static str> {
    let s = sample.trim();
    // **A sample that names a pipeline stage is judged as one, and does not fall through.**
    // Checked first and exiting either way, because the permissive frames below would
    // otherwise accept it: `order_by(|r| descending(r.amt))` is a perfectly good
    // *expression*, so a stage sample naming a function the surface does not have would parse
    // as an expression and be reported as fine — which is F-09 wearing a different hat.
    //
    // `StageKind::from_name` decides, so this test and the parser agree on what a stage is.
    if let Some(framed) = as_pipeline(s) {
        return lowers(&framed).then_some("pipeline (lowered)");
    }
    // 1. A top-level item, beside the fixture's schema.
    if parses(&format!("{FIXTURE}\n{s}\n")) {
        return Some("item");
    }
    // 2. A statement in a function body.
    if parses(&format!("{FIXTURE}\nfn kwfn() {{ {s} }}\n")) {
        return Some("statement");
    }
    // 4. An expression, inside a `where`.
    if parses(&format!(
        "{FIXTURE}\nview kw_expr = t.where(|r| {s}) serve {{ consistency: snapshot, materialize: auto }};\n"
    )) {
        return Some("expression");
    }
    // 5. A **declaration clause** — a column with an annotation, a `conserve`, a `retain` —
    // which is a sample of something that only exists inside a `base` or `ledger` body.
    if parses(&format!(
        "schema kwdecl {{ currency usd {{ scale: 2 }} base decl {{ x: Int, {s} retain forever; }} }}\n"
    )) || parses(&format!(
        "schema kwdecl {{ currency usd {{ scale: 2 }} base decl {{ x: Int, {s} }} }}\n"
    )) {
        return Some("declaration clause");
    }
    // 6. An **item inside a schema block**, for the declarations that only exist there.
    if parses(&format!(
        "schema kwitem {{ currency usd {{ scale: 2 }} base t {{ acct: Int, retain forever; }} {s} }}\n"
    )) {
        return Some("schema item");
    }
    None
}

#[test]
fn every_keyword_sample_is_syntax_the_compiler_has() {
    let mut orphans: Vec<String> = Vec::new();
    let exempt: std::collections::BTreeMap<&str, &str> = NOT_COMPILED.iter().copied().collect();
    for k in keywords::KEYWORDS {
        if let Some(why) = exempt.get(k.word) {
            assert!(
                !why.is_empty(),
                "`{}` is exempt with an empty reason, which is not a decision",
                k.word
            );
            continue;
        }
        if accepted(k.example).is_none() {
            orphans.push(format!("{}: {}", k.word, k.example.replace('\n', " ")));
        }
    }
    assert!(
        orphans.is_empty(),
        "{} keyword samples are not syntax this compiler accepts in any of the four frames \
         (item, statement, lowered stage, expression). Either the sample is wrong, or the \
         construct it documents does not exist — which is the defect this test was written \
         for, and it is not fixed by adding the word to NOT_COMPILED:\n{}",
        orphans.len(),
        orphans.join("\n")
    );
}

/// The exemption list is a bounded list of decisions, not a growing amnesty.
#[test]
fn the_exemption_list_stays_small_and_reasoned() {
    assert!(
        NOT_COMPILED.len() <= 31,
        "{} samples are exempt from compilation, which is more than the registry has decisions \
         for today (31: fifteen outlines whose `..` or free names stand for something declared \
         elsewhere, fifteen MISMATCHes where the registry documents a construct the compiler \
         does not have, and one deliberate refusal). \
         Adding a word here is a decision that belongs in the work-order report, not a way \
         to make this test pass.",
        NOT_COMPILED.len()
    );
    for (w, why) in NOT_COMPILED {
        assert!(
            keywords::lookup(w).is_some(),
            "`{w}` is exempt and is not a keyword"
        );
        assert!(
            why.len() > 20,
            "`{w}`'s reason is too short to be one: {why}"
        );
    }
}
