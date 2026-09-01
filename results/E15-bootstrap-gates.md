# E15 — The Appendix E Bootstrap Gates

*Can Niles express its own front end, and does the result agree with the reference
implementation?*

Reproduce with `cargo test -p niles-interp`. The Niles source under test is
`bootstrap/lexer.niles` and `bootstrap/parser.niles`; the gates are
`crates/niles-interp/tests/bootstrap_stages.rs` (lexer, 14 tests) and
`crates/niles-interp/tests/bootstrap_parser.rs` (parser, 16 tests).

This file records two rounds. **Round 1** was the lexer. **Round 2**, below, is the
parser, and it found three defects the lexer round could not: one in the stage-0
interpreter, one in the *reference* parser, and one hole in Appendix B.

## Method

Appendix E.1 defines a three-stage bootstrap. Its blocker was recorded in E.0 as "stage 1
has no input", on the reasoning that stage 0 must compile the whole language before a
Niles-written compiler can run. That reasoning was wrong: a bootstrap needs stage 0 to
**evaluate** Niles programs, not to emit machine code. Reading "stage-0 compiler" as
"native compiler" is what made stage 1 unreachable.

Two artefacts were therefore built:

* `crates/niles-interp` — a tree-walking interpreter for Niles's imperative subset. It
  refuses the relational tier by name rather than approximating it, refuses float
  literals, and refuses money arithmetic (the currency check lives in the type system;
  a second unchecked path to the same operation is the seam §6.9 refuses). Environments
  are `BTreeMap`, so no hash seed can reach the output. 28 tests.
* `bootstrap/lexer.niles` — a lexer **written in Niles**, ~450 lines, covering
  identifiers, all 174 keywords, integers, money literals, epoch literals, strings,
  comments and the full punctuation table. Not covered, and excluded from the gate
  visibly: instants, durations, byte strings, raw identifiers.

Four gates, 14 tests.

## Results

| Gate | Result |
|---|---|
| Stage 1 — the Niles lexer parses and runs under stage 0 | **pass** |
| Stage 1 equivalence — vs. the Rust lexer over a 29-case corpus | **pass**, every token's kind, span and text |
| Stage 2 — self-application: the Niles lexer lexes its own source | **pass**, identically to the reference |
| Stage 3 — fixpoint: repeated runs and repeated self-application | **pass**, byte-identical |

Corpus coverage is chosen for the decisions a hand-written lexer actually gets wrong:
longest-match punctuation (`::` before `:`, `|>`/`||` before `|`, `=>`/`==` before `=`),
the sigil rule separating `#4200` from `#[attr]`, the money rule separating `10.00 usd`
from `10` followed by `.days`, string escapes, and **unterminated constructs at end of
input**. The last class is deliberately included rather than excluded: a malformed input
is where two lexers most easily disagree, since each must independently decide where the
broken construct ends.

## The gate rejected four defects on its first run

A gate that has never rejected anything has no demonstrated discriminating power. This one
failed four times before passing.

| # | Defect | Cause |
|---|---|---|
| 1 | Keyword table listed `evict`, `conserves` | Neither exists; the registry's words are `evictable`, `conserve` |
| 2 | Keyword table omitted `from`, `post` | Both exist |
| 3 | Keyword matching was case-sensitive | Niles inherits **case-insensitive** keywords from SQL; `Epoch` = `epoch`. Rust — the other half of Niles's ancestry — is case-sensitive, and a lexer written by someone thinking in Rust gets this wrong |
| 4 | The gate's scope filter excluded any input the reference lexer errored on | Silently removed the two unterminated-construct cases. A defect in the **gate**, not in either lexer |

Defects 1 and 2 are the keyword-registry argument (§6.4) arriving as evidence: two
implementations of one language drifted apart within a day of the second existing. The fix
was not to correct the table but to remove the possibility — the Niles-side table is
**generated from `keywords.rs`**, and `keyword_table_matches_the_registry_exactly` fails
the build if the two ever diverge. The single source of truth crosses the bootstrap
boundary.

Defect 3 is a genuine finding about the language rather than about the code: Niles's two
lineages disagree on case sensitivity, and only the registry settles it. Appendix B should
state the rule normatively; it currently states it only by implication.

## What this does and does not establish

**Establishes.** That Niles can express a lexer; that the lexer agrees with an independent
implementation over a corpus including malformed input; that it is deterministic run to
run; and that stage 0 can execute a non-trivial Niles program end to end.

**Does not establish.** That Niles can express a *compiler*. E.1's stage 2 is "stage 1
recompiling the same sources", and a lexer compiles nothing; what is checked here is its
analogue, self-application. A type-checker and a lowering pass in Niles are unwritten.
(A parser was, at the time of round 1; round 2 below closes that half.)

**Does not establish, specifically.** Cross-target determinism. Appendix C.4's obligation
is byte-identical output on ARM64 *and* x86-64; this gate runs one target in one process
and can only check run-to-run determinism. That obligation remains unmet and E.19 says so.

**Does not establish.** Any trusting-trust property. A bit-identical self-build is what a
successful Thompson attack looks like; diverse double-compiling is what would address it,
and it is future work (§12).


---

# Round 2 — the parser

*The same four gates, one stage further up.*

`bootstrap/parser.niles` is a recursive-descent parser **written in Niles**, ~1,050 lines,
loaded together with `lexer.niles` as one program and run by the stage-0 interpreter. It
builds a tree — `enum Node { Atom(str), List([Node]) }` — and renders it afterwards; it is
not a syntax-directed translator emitting text, because the claim under test is that Niles
can *hold* a syntax tree, not that it can concatenate strings in the right order.

Both sides render to the S-expression format in `niles_lang::sexpr`, independently, and
the gate compares strings. Spans are excluded: token offsets are the lexer's gate, already
checked there, so a disagreement here is a disagreement about structure and nothing else.

## Scope

Covered: `fn`, `struct`, `enum`, `use`, `let`, `if`/`else`, `match` with guards, `while`,
`loop`, `for`, `return`, `break`, `continue`, closures, calls, pipeline stages, indexing,
field access, struct literals, tuples, arrays, casts, `?`, effect rows, generics,
visibility, and the full operator table of Appendix B.10.1.

Not covered, and excluded **visibly**: `schema`, `view`, `trait`, `impl`, `mod`, `const`,
`static`, `type`, attributes, the SQL surface, the novel relational forms, and (already
outside the lexer's scope) floats, instants, durations and byte strings. The Rust renderer
emits `(unsupported <form>)` for each, and `in_scope` refuses any input whose rendering
contains one. A second exclusion is narrower and stated for the same reason: a non-ASCII
character inside a *string literal*, because the Niles side rebuilds a literal's decoded
value byte by byte through `chr`. Comments are unaffected, which is why both bootstrap
files — full of em-dashes — are in scope.

One gap is a gap rather than an exclusion: the Niles parser has no **diagnostic** channel.
Where the reference parser builds an error node, this one builds the same error node and
the trees agree; where the reference parser reports a message and builds an ordinary node —
a reserved word used as an identifier — this one is silent. Pinned by
`the_reserved_word_gap_is_real_and_declared` so it stays known.

## Results

| Gate | Result |
|---|---|
| Stage 1 — the Niles parser parses and runs under stage 0 | **pass** |
| Stage 1 equivalence — vs. the Rust parser over a 48-case corpus | **pass**, every node of every tree |
| Stage 2 — the Niles parser parses `lexer.niles` | **pass**, identically to the reference |
| Stage 2 — the Niles **front end parses its own two source files** | **pass**, 127,165 bytes of tree, identical |
| Stage 3 — fixpoint: repeated runs and repeated self-application | **pass**, byte-identical |
| Negative control — a missing brace | both parsers recover to the same tree; reference reports |
| Negative control — a token that cannot begin an expression | both emit `(eerr)` and resynchronise identically |
| Negative control — `a - b - c` associates left, `a = b = c` right | **pass** after a fix; see below |
| Table drift — Appendix B.10.1 vs `BinOp::precedence` | **pass**, `crates/niles-lang/tests/precedence.rs` |

The corpus is chosen for what a hand-written parser actually gets wrong: precedence and
associativity at every adjacent pair of levels, the struct-literal/block ambiguity in
`if`/`while`/`match`/`for` heads, the statement-versus-tail-expression split, optional
slots that must not be elided, and the places where two spellings must produce one tree
(`and`/`&&`, `q.map(f)`/`q |> map(f)`).

## The gate rejected three defects, in three different artefacts

| # | Where | Defect |
|---|---|---|
| 1 | `crates/niles-interp` | ~95 KB of host stack per Niles call frame in a debug build; a recursive-descent parser was unrunnable, and the failure mode was a **process abort** with no diagnostic |
| 2 | `crates/niles-lang/src/parser.rs` | Assignment was **left**-associative, contradicting Rust's rule and the comment directly above the code |
| 3 | `thesis/appendix-b.md` | There was no operator precedence table, so there was no arbiter for defect 2 |

**Defect 1** is the interesting one because it is invisible until something recurses. A
`match` over thirty expression variants compiles, unoptimised, to a frame holding the union
of every arm's locals; measured, that was 95 KB per Niles call in debug and 4.8 KB in
release — a 20x build-mode penalty. Moving the fat arms behind `#[inline(never)]` brought
debug to 32 KB (release 3.3 KB), and a call-depth counter now turns the remaining limit
into an `Error::TooDeep` carrying a span instead of a SIGSEGV. `run_with_stack` pairs a
raised ceiling with a stack sized for it. The default ceiling is 24, which is the number
that fits a 1 MB stack in the widest build; the first value tried was 64, on the assumption
that a spawned thread gets 2 MB, and it aborted the test process — which is how the number
became measured rather than assumed, and a small demonstration of why the counter had to
exist.

| Build | KB of host stack per Niles call frame | Max depth on 8 MB |
|---|---|---|
| debug, before | 95.5 | 84 |
| debug, after | 32.4 | 251 |
| release, before | 4.82 | 1,699 |
| release, after | 3.35 | 2,446 |

**Defect 2** is the one the exercise was for. `expr_bp` recursed for an assignment's
right-hand side at binding power 1, which put assignment outside its own `min_bp == 0`
guard and made `a = b = c` parse as `(a = b) = c`. No test had asked: associativity is
invisible in a token stream, so the lexer round could not have found it, and nothing else
was looking. It surfaced within minutes of a second implementation of the same grammar
existing — which is the argument for stage-1 equivalence stated as a result rather than as
a hope.

**Defect 3** is what made defect 2 arguable rather than decidable. §6.25 makes the appendix
normative, but B had no operator table, so "the appendix wins" had nothing to win with.
B.10.1 now states precedence and associativity, and `precedence.rs` reads the table out of
the markdown and checks every level against `BinOp::precedence` — so the two can no longer
drift, and the rule is enforced rather than declared.

## Cost

Stage 2 runs a parser written in Niles, under a tree-walking interpreter, over 1,200 lines
of Niles: ~30 s in a debug build, ~13 s in release. It is the most expensive test in the
workspace. It is the only one that executes the front end against itself.

## What round 2 does and does not establish

**Establishes.** That Niles can express a *parser*, including a recursive tree type and the
recursion over it; that the parser agrees with an independent implementation node for node
over a 48-case corpus and over 1,200 lines of real source, including its own; that it
recovers from malformed input to the same tree the reference does; and that it is
deterministic run to run.

**Does not establish.** That Niles can express a type-checker or a lowering pass. Those
remain Rust, and §E.19 says so. The front end is self-hosted; the middle end is not.

**Does not establish.** Cross-target determinism, or any trusting-trust property. Unchanged
from round 1.
