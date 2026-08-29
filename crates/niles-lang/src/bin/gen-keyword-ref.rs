//! Generates `docs/keywords.md` — the normative per-keyword reference of thesis
//! Appendix B.19 — from the compiler's keyword registry.
//!
//! PostgreSQL generates its perfect-hash lookup, its parser token declarations *and* its
//! documentation from one `kwlist.h` table, so that none of the three can drift from the
//! others. This binary is the documentation arm of the same discipline. Run it with
//! `cargo run -p niles-lang --bin gen-keyword-ref`; `tests/keyword_ref.rs` fails if the
//! checked-in file differs from what this program would produce, which is what makes the
//! thesis's "generated from the compiler's keyword registry" a checkable statement rather
//! than an aspiration.

use niles_lang::keywords::{by_origin, Category, Keyword, Label, Origin, KEYWORDS};
use std::fmt::Write;

#[allow(dead_code)]
fn main() {
    let out = render();
    let path = std::env::args().nth(1).unwrap_or_else(|| "docs/keywords.md".to_string());
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).ok();
    std::fs::write(&path, out).expect("write keyword reference");
    eprintln!("wrote {path} ({} keywords)", KEYWORDS.len());
}

pub fn render() -> String {
    let mut s = String::new();
    let total = KEYWORDS.len();
    let reserved = KEYWORDS
        .iter()
        .filter(|k| matches!(k.category, Category::Reserved | Category::ReservedFuture))
        .count();
    let unreserved = KEYWORDS.iter().filter(|k| k.category == Category::Unreserved).count();

    s.push_str("# The Niles Keyword Reference\n\n");
    s.push_str(
        "<!-- GENERATED FILE — DO NOT EDIT.\n     \
         Produced by `cargo run -p niles-lang --bin gen-keyword-ref` from the keyword\n     \
         registry in `crates/niles-lang/src/keywords.rs`. `tests/keyword_ref.rs` fails if\n     \
         this file and the registry disagree, so the two cannot drift. -->\n\n",
    );
    let _ = writeln!(
        s,
        "This is the normative per-keyword reference of thesis Appendix B.19. It is \
         **generated** from the compiler's keyword registry, not maintained alongside it: \
         a keyword cannot exist in the lexer without an entry here, and an entry here \
         cannot describe a keyword the lexer does not have.\n"
    );
    let _ = writeln!(s, "**{total} keywords**: {unreserved} unreserved, {reserved} reserved (including reserved-for-future), \
         {} in the remaining two classes.\n", total - unreserved - reserved);

    s.push_str("## How to read the tables\n\n");
    s.push_str(
"Two orthogonal axes, following PostgreSQL's `kwlist.h`, which carries both because a \
language with a keyword-delimited grammar and an installed base of schemas needs both.\n\n\
**Category** — how far the word is reserved:\n\n\
| Category | Meaning |\n|---|---|\n\
| `unreserved` | No special status outside its own clause. Usable as any identifier, anywhere. |\n\
| `non-reserved (cannot be function or type name)` | Usable as a column or variable, but the word introduces built-in syntax that would be ambiguous with a call. |\n\
| `reserved (can be function or type name)` | Usable as a function or type name; a bare occurrence in column position would be read as a clause. |\n\
| `reserved` | Never an identifier without the `r#` escape. |\n\
| `reserved for future use` | Reserved with no meaning assigned. Using it is a hard error naming the reason. |\n\n\
**Label** — whether the word may be a bare output label without `as`. `select 55 check` \
is legal; `select 55 as from` is required. This is a second dimension, not a fifth \
category value.\n\n\
**Escape.** Any word, however reserved, is usable as an identifier as `r#word`.\n\n\
**Reservation policy.** A new keyword is `unreserved` unless a written justification \
records why the grammar cannot be written without reserving it. Reserved-word count is a \
function of parser technology, not of vocabulary size: Niles uses hand-written recursive \
descent with unbounded lookahead, which keeps words like `epoch`, `ledger`, `serve` and \
`budget` available as column names in a bank's existing schema. Every novel keyword in \
this reference is unreserved, and a test enforces it.\n\n",
    );

    for (origin, title, blurb) in [
        (
            Origin::Sql,
            "SQL-derived keywords",
            "Taken from SQL, with SQL's meaning wherever the meaning survives. Where it does \
             not — `update` and `delete` are legal against a `table` and meaningless against a \
             `ledger` — the difference is stated in the entry.",
        ),
        (
            Origin::Rust,
            "Rust-derived keywords",
            "Taken from Rust, because SQL has no equivalent construct. Restrictions apply in \
             query and transaction context: no ambient I/O, no wall clock, no `unsafe`, and \
             recursion only through a guarded `fixpoint`.",
        ),
        (
            Origin::Novel,
            "Novel Niles keywords",
            "These name concepts neither SQL nor Rust has: an immutable epoch-ordered base, a \
             conservation rule, a per-view consistency contract, a materialization mode, a \
             linear hold, an atomic cross-currency form, a confidentiality level. Every one of \
             them is unreserved.",
        ),
    ] {
        let n = by_origin(origin).filter(|k| k.category != Category::ReservedFuture).count();
        let _ = writeln!(s, "## {title} ({n})\n");
        let _ = writeln!(s, "{blurb}\n");
        s.push_str("| Keyword | Category | Label | Since | Description | Example |\n");
        s.push_str("|---|---|---|---|---|---|\n");
        for k in by_origin(origin).filter(|k| k.category != Category::ReservedFuture) {
            row(&mut s, k);
        }
        s.push('\n');
    }

    let future: Vec<&Keyword> =
        KEYWORDS.iter().filter(|k| k.category == Category::ReservedFuture).collect();
    let _ = writeln!(s, "## Reserved for future use ({})\n", future.len());
    s.push_str(
        "Reserved with no meaning assigned in this edition. Using one is a hard error that \
         names the reason, which is how a language keeps room to grow without a breaking \
         change later.\n\n| Keyword | Reason |\n|---|---|\n",
    );
    for k in future {
        let _ = writeln!(s, "| `{}` | {} |", k.word, k.doc);
    }

    s.push_str("\n## The normative reserved list (Appendix B.16)\n\n");
    let mut reserved_words: Vec<&str> = KEYWORDS
        .iter()
        .filter(|k| !k.category.usable_as_ident())
        .map(|k| k.word)
        .collect();
    reserved_words.sort_unstable();
    let _ = writeln!(s, "{} words. All require `r#` to be used as identifiers.\n", reserved_words.len());
    let _ = writeln!(s, "```\n{}\n```\n", reserved_words.join(" "));

    s.push_str("## Words that are *not* reserved, and why that matters\n\n");
    let mut free: Vec<&str> = KEYWORDS
        .iter()
        .filter(|k| k.category == Category::Unreserved)
        .map(|k| k.word)
        .collect();
    free.sort_unstable();
    let _ = writeln!(
        s,
        "{} words carry meaning in their own clause and are ordinary identifiers everywhere \
         else. A bank migrating a schema whose columns are called `epoch`, `ledger`, `posted`, \
         `serve`, `budget` or `scale` does not have to rename them.\n",
        free.len()
    );
    let _ = writeln!(s, "```\n{}\n```\n", free.join(" "));
    s
}

fn row(s: &mut String, k: &Keyword) {
    let label = match k.label {
        Label::Bare => "bare",
        Label::RequiresAs => "requires `as`",
    };
    let esc = |t: &str| t.replace('|', "\\|");
    let _ = writeln!(
        s,
        "| `{}` | {} | {} | {} | {} | `{}` |",
        k.word,
        k.category.as_str(),
        label,
        k.since,
        esc(k.doc),
        esc(k.example)
    );
}
