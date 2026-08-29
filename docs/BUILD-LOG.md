# Build log — from specification to instrument

Running record of the work that turns the thesis's *specified* language and engine into a
*built* one, with the reasoning behind each decision. Written as it happens, so that a
session that ends mid-way leaves something a later one can pick up. Newest section last.

## Why this file exists

The Turn-5 audit found the thesis and the repository contradicting each other. Appendix
B.14 said the full grammar "is maintained in the artifact at
`crates/niles-lang/grammar/niles.ebnf`"; the file had five lines. Appendix B.19 said the
keyword reference "is generated from the compiler's keyword registry"; there was no
registry and `docs/keywords.md` was a placeholder. Appendix E described the bootstrap in
the present tense. Chapter 7 correctly said "stub crates only", so the document
contradicted itself rather than merely overclaiming.

Two ways to fix that: weaken the claims, or build the artifacts. This log records the
second.

---

## Stage 0 — research before design

`docs/research/compiler-lessons.md` (12,000 words, primary sources, IEEE references)
collects transferable lessons from rustc and from SQL compilers. Five findings changed the
design that follows; two corrected assumptions this project held.

**Changed the design.**

1. **Split parsing from analysis on an epoch boundary.** PostgreSQL separates raw parsing
   from semantic analysis because "system catalog lookups can only be done within a
   transaction, and we do not wish to start a transaction immediately upon receiving a
   query string." Nilestream has the strictly stronger version of that constraint: a name
   resolves only relative to a *visibility frontier*, so a Niles parse must be epoch-free
   and total, and analysis epoch-anchored. This is not an imported style — it falls out of
   the consistency model, and it is now an argument in the thesis rather than an
   implementation note.
2. **The keyword registry is a data table with four axes, and everything generates from
   it.** PostgreSQL's `kwlist.h` is one table with no logic, consumed by whoever defines
   the macro, and it generates the perfect-hash lookup, the parser's token declarations
   *and* the documentation. Niles copies the pattern and adds two axes the thesis needs:
   `origin` (SQL / Rust / novel), which generates Appendix B.3's three tables
   mechanically, and `since`, which makes the edition mechanism enforceable.
3. **Reserved-word count is a function of parser technology, not vocabulary size.** SQL
   reserves heavily partly because LALR(1) cannot resolve context-dependent
   identifier/keyword ambiguity. Hand-written recursive descent with unbounded lookahead
   keeps `epoch`, `ledger`, `serve` and `budget` usable as column names — which is what
   makes Niles adoptable against a bank's existing schema. Measured outcome: 0 of 66
   novel keywords are reserved.
4. **Four IR levels, split by what becomes checkable at each.** Following RFC 1211's
   account of why MIR exists: surface CST, desugared, fully typed (where the
   consistency-effect calculus is stated), and a flat epoch-explicit operator DAG (where
   the partial-state algebra lives and materialization is decided).
5. **GoogleSQL's accessed-field discipline for the IR.** A mechanism that turns "the
   back-end silently ignored a semantic field" into a hard error is worth a great deal in
   a system whose central claim is that money cannot be created or destroyed.

**Corrected assumptions.** Chalk did not ship — what shipped is `rustc_next_trait_solver`,
influenced by it. The bootstrap gate is stage2-vs-stage3, not stage2 against itself, and
modern stage0 is a released beta, not a snapshot. Noria has no documented IR, so
"Nilestream has a typed IR and Noria did not" is a positioning asset, not a comparison.
And no authoritative production count exists for the SQL-92 or SQL:2016 BNF — so this
thesis cites none, and counts its own grammar instead.

---

## Stage 1 — the keyword registry (`crates/niles-lang/src/keywords.rs`, 470 lines)

174 keywords in one table, in ASCII order within four blocks. Each row carries spelling,
token, category, label status, origin, edition, a one-line description and a minimal
example. Six tests, including two that enforce policy rather than mechanics: the registry
must stay sorted (so a diff and a perfect-hash generator agree), and **no novel keyword may
be reserved without a written justification in the file**.

Measured: 174 keywords — 70 SQL-derived, 31 Rust-derived, 66 novel, 7 reserved-for-future.
95 unreserved, 66 reserved, the rest in the two intermediate classes. Every one of the 66
novel words is unreserved.

## Stage 2 — the lexer (`src/lexer.rs`, 640 lines, 10 tests)

Two layers, following rustc's split of a dependency-free `rustc_lexer` from a cooking
`StringReader`. The raw layer knows characters, not keywords. **The stream is lossless** —
every byte of input belongs to exactly one token or one trivium, checked by a test — which
is what a formatter, an IDE and a pretty-print round-trip need.

Four literal forms exist in neither parent language, and each needed a disambiguation
decision:

* **Money** `10.00 usd`. The literal's *own* scale is counted from the digits and kept
  separate from the currency, so `10.001 usd` becomes a scale error naming both numbers
  rather than a silent rounding. A number followed by an ordinary word is not money: only
  a three-letter lowercase non-keyword closes the literal.
* **Epoch** `#4200`, disambiguated from the attribute prefix `#[` by one character.
* **Two time axes** `@2026-03-01` and `v@2026-03-01`. Two prefixes, because bitemporality
  has two axes and one spelling would make the axis invisible at the point of use. An
  ISO-8601 body always begins with a digit, which is how `@confidential` and
  `read@snapshot` share the sigil without ambiguity.
* **Duration** `7.days`, checked before the fraction rule.

## Stage 3 — AST and diagnostics (`src/ast.rs` 720 lines, `src/diagnostics.rs` 250 lines)

The AST is the *raw parse* level: a `Name` is a string and a span, never a resolved id.
`StageKind::Unknown` exists so an unrecognised pipeline stage parses and is diagnosed with
a suggestion, and `Expr::Error` / `Item::Error` are ordinary variants.

Diagnostics are structured values with a stable `NLnnnn` code, a primary span, labelled
secondary spans, notes and machine-applicable suggestions. The reason is specific to this
thesis: the interesting errors are conservation, currency, linearity, effect and contract
errors, and every one of them must point at two places at once — the money that was
created *and* the rule that forbids it; the hold *and* its second resolution; the view's
declared rung *and* the effect that exceeds it. A one-span error cannot say that.

## Stage 4 — the parser (`src/parser.rs`, 1,300 lines, 15 tests)

Hand-written recursive descent with a Pratt loop. Every `parse_*` returns a node, error
nodes included; every recovery point names a synchronising set and always consumes at
least one token. Two properties have tests of their own: **a bad declaration between two
good ones must not destroy either**, and **parsing is total** — twenty adversarial inputs
including empty, unbalanced and non-UTF-8-shaped text must all terminate with a tree.

One design point worth recording: after `.` or `|>`, and in struct-literal field position,
**every** keyword is legal, because those positions have exactly one reading. That is why
`p.where(..)`, `p.select`, `p.order` and `p.check` all work without reserving anything.

The worked program of thesis Appendix B.20 is now `examples/demo_bank.niles` and parses
clean, under test. Where the thesis's printed example used forms the language does not
have — an `idem"..."` adjacent-string literal, `acct!(1001)` macros — the example was
corrected to the real syntax rather than the syntax being bent to the prose.

## Stage 5 — the normative grammar (`grammar/niles.ebnf`, 600 lines)

205 named rules; 496 productions in BNF-normal form. **Both figures are computed by
`tests/grammar_drift.rs` and asserted against the header**, so neither can drift. The file
carries fifteen sections, ending with twenty numbered well-formedness rules (W1–W20) that
are deliberately *not* syntax — a grammar that tried to encode "a hold is consumed exactly
once" would be neither readable nor decidable — each of which is a claim that some later
phase enforces it.

Four drift tests run in both directions: every registry keyword must appear in some
production, every alphabetic grammar terminal must be a registry keyword or a known
library name, the stage vocabulary must match `StageKind::all_names()`, and the W-rules
must be contiguous.

## Stage 6 — the generated keyword reference (`docs/keywords.md`)

`cargo run -p niles-lang --bin gen-keyword-ref` renders the registry into the normative
per-keyword reference of Appendix B.19. `tests/keyword_ref.rs` is a blessing test in
`compiletest`'s style: it fails if the checked-in file differs from what the generator
would produce now. That is what turns "generated from the compiler's keyword registry"
from a claim about how the file was once produced into one that holds continuously.

**Status at this point: 45 tests passing in `niles-lang`. The three thesis/repo
contradictions identified in the Turn-5 audit are closed by construction rather than by
weakening the prose.**
