# E15 — The Appendix E Bootstrap Gates

*Can Niles express its own front end, and does the result agree with the reference
implementation?*

Reproduce with `cargo test -p niles-interp`. The Niles source under test is
`bootstrap/lexer.niles`; the gates are `crates/niles-interp/tests/bootstrap_stages.rs`.

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
analogue, self-application. A parser, a type-checker and a lowering pass in Niles are
unwritten.

**Does not establish, specifically.** Cross-target determinism. Appendix C.4's obligation
is byte-identical output on ARM64 *and* x86-64; this gate runs one target in one process
and can only check run-to-run determinism. That obligation remains unmet and E.19 says so.

**Does not establish.** Any trusting-trust property. A bit-identical self-build is what a
successful Thompson attack looks like; diverse double-compiling is what would address it,
and it is future work (§12).
