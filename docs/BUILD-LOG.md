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

---

## Stage 7 — the semantic phases (`resolve`, `typecheck`, `currency_rows`, `effects`)

Four modules, and the split between them is the parse/analyse boundary taken seriously.

`resolve.rs` builds the catalog **at an epoch**. That is not ceremony: in this system a
name resolves only relative to a visibility frontier, because a migration is itself a
ledger fact and `postings` at epoch 4,200 may not have the columns it has at 9,000. So
`Catalog::at(epoch)` carries the epoch, and the same code serves the offline compiler and
the server path. PostgreSQL splits parse from analyse because catalog lookups need a
transaction; this is the stronger version of the same constraint, and it is an argument
the thesis can make rather than a style it imported.

`currency_rows.rs` (11 tests) is the machinery behind the first half of Contribution 4. A
transaction's net effect is not a number, it is a *row*: a map from currency to signed
amount. `conserve per (txn, cur)` says every entry of that row is zero — not that the
total is zero, which would let 10 USD cancel 10 EUR. Amounts are linear combinations of
opaque symbols plus a constant, so `debit(a, x); credit(b, x)` is provably conserving for
an unknown `x`, and a currency variable with union-find makes `fn move<C>(..)` conserving
for every `C` without monomorphising by currency.

The design decision worth recording is the **third verdict**. A row can be *conserving*,
*violating*, or **undecided** — the checker cannot see through the amount. Undecided is
not an error; it is an obligation handed to the runtime and counted. A checker that
accused every program it could not follow would be unusable, and one that assumed the good
case would make the soundness theorem vacuous. The third verdict is what keeps the first
two honest.

`effects.rs` (12 tests) is the second half. The novel judgement is **rung monotonicity**: a
view served at rung ℓ may not read a view served below ℓ. Everything else in the calculus
is a currency-flavoured restatement of a known effect system; this one is not, and it is
the one that earned its keep (see Stage 9).

## Stage 8 — the IR (`niles-ir`, 39 tests)

Four levels, split by what becomes checkable at each — the reason rustc has MIR, applied
here. The circuit level is where the partial-state algebra lives, and it carries two things
a conventional relational IR does not.

**The accessed-field discipline**, borrowed from GoogleSQL's resolved AST. Every
semantically load-bearing field on a node sits behind an accessor that records the read,
and `assert_all_accessed` fails if any went unread. The failure this prevents is not a
crash. It is an engine that does not notice a `ledger_consistent` annotation, serves from
stale state, and returns a number that looks exactly right. No answer-level test catches
that; only a test of whether the annotation was *read* does.

**Conservation transparency**, composed along paths. A `filter` drops rows, so money can
leave a filtered view without leaving the ledger. That is a legitimate thing to want and
it is not a control total, and the IR is where the difference becomes checkable.

`upquery_path.rs` derives reconstruction routes and is where anchoring stops being a slogan.
A path is a pure function of `(circuit, node, key, epoch)` — no locks, no sequence numbers,
no protocol state — because the prefix it reads is not moving. The five partial-state
anomalies are not prevented here; they have no state in which they could occur.

`verify.rs` is in the trusted base and the compiler is not. That is deliberate: the
compiler is large and will be extended by people who did not write it; the verifier is
small and can be read in an afternoon. A compiler bug therefore produces a *rejected
circuit* rather than a wrong answer, and a wrong answer here means money.

## Stage 9 — closing the loop, and what it found

`crates/nilestream` compiles a `.niles` file, verifies the circuit, installs it on the REV
runtime, and runs a workload against a real ledger. Two Chapter 9 findings reproduce
through that path rather than around it: SC7 (flat at 8.4 base rows across a 16× history
increase, predicted 9) and the phase diagram (strictly interior optimum, boundary between
memory prices 0.0005 and 0.002).

**Then the compiler was run over the thesis's own worked program, and rejected it.** Four
errors, of which one matters most: `available_balance` promised `ledger_consistent` while
reading a `read_your_writes` view. That is the two-derived-views-disagreeing failure
Chapter 1 opens with, written into the example by the person who formulated the rule that
forbids it. Nothing else in this project argues as well for having a compiler.

The other three: `holds` declared as a `ledger` when it does not conserve; `settle_fx`
declaring an effect row describing a transaction that could not exist; and a view predicate
calling `month_start()`, which reads the wall clock — the IR verifier rejected it because a
reconstruction could then legitimately differ from the value it replaced, which would make
reconstruction-equivalence *false* rather than unproven.

## Stage 10 — durability and concurrency (`nilestream-ledger`, 21 tests)

Length-prefixed, CRC-checked records; the hash chain recomputed on read rather than taken
on trust; recovery that truncates at the first bad record and reports how much it dropped;
a single-sealer sequencer with group commit that publishes the frontier only after fsync.

The ordering rule is the entire guarantee: **durable, then visible**. Publishing first
would let a reader observe an epoch a crash then erases, and in a ledger that is not a
stale read — it is a transaction the customer watched succeed and that no longer exists.

Three defects found by the tests, each a money bug rather than a crash:

1. **Two copies of one idempotency key in the same batch both committed.** The dedup
   consulted committed history but not the batch being assembled. A client retrying fast —
   or a client and a proxy retrying together — lands both in one drain. The result is a
   duplicated payment, not an error.
2. **A duplicate was accounted for after its reply was sent**, so a caller could observe an
   outcome before the state that produced it. The same ordering rule as durable-before-
   visible, one level up. It failed about one run in three, which is exactly the frequency
   at which a race gets written off as flakiness.
3. **`Always` synced twice per epoch.** The benchmark surfaced it as 0.5 transactions per
   fsync — a figure with no sensible interpretation, which was the clue. Fixing it raised
   single-threaded throughput 3,003 → 4,450 tx/s.

E13 then measured what Chapter 9 had marked *to be measured*: durability costs 5.7× at one
thread and 4.8× at sixteen, because group commit amortises the fsync. Transactions per
fsync rise 1.0 → 8.8. **A single sealer is a batching opportunity, not the ceiling it looks
like** — which was the design's most likely objection, and is now answered with a number.

## Where this leaves the thesis

The three contradictions from the Turn-5 audit are closed by building the artifacts rather
than by weakening the claims. Chapter 7 now states what is built and what is not with equal
precision, §7.5 lists each gap with the claim it withholds, and Appendix E.0 labels its own
tense. §6.10.1 re-assesses the language-creation gate against the built compiler and
**narrows G1**, because a gate that always passes is not a gate.

Still unbuilt, and stated as such: distribution, consensus, wire protocols, a cost-based
planner, and the self-hosted bootstrap. The executable IR fragment is narrower than the IR
the compiler emits, and the runtime rejects what it cannot run rather than mis-executing it.
