# Appendix E. The Niles Self-Hosting Compiler

## E.0 Status: What Is Built, and What This Appendix Describes

This appendix is written in the present tense throughout, and the reader is owed an explicit statement of what that tense refers to, because the answer differs by section and a document that blurred the difference would be overclaiming.

**Built and running** (`crates/niles-lang`, and reproducible by `cargo test -p niles-lang`): the stage-0 front end. The keyword registry of B.19; the normative grammar of B.15; a lossless two-layer lexer covering every literal form of B.2, including money with per-currency scale, both temporal axes, epochs and durations; a resilient hand-written recursive-descent parser with Pratt expression parsing, which is total on arbitrary input and recovers at item and statement boundaries; the surface AST; the diagnostics subsystem with stable `NL` codes, multi-span labels and machine-applicable suggestions; the epoch-anchored resolver and catalog; the currency-row solver; the consistency-effect calculus with the rung-monotonicity judgement; the linearity checker; and lowering to the typed IR of Appendix D. This front end compiles the worked program of B.20, which is kept compiling by a test so that the thesis's example cannot drift from the language it describes.

**Built and running, and new since the previous revision of this appendix** (`crates/niles-interp`, `bootstrap/lexer.niles`; reproducible by `cargo test -p niles-interp`): **stage-0 execution, and the first three bootstrap gates.** The earlier revision recorded that "stage 1 has no input yet, and no line of the Niles-written compiler has been written." Both halves of that are now false, and the correction turned on noticing that the appendix had imposed a requirement on itself that E.1 never stated. A bootstrap needs stage 0 to **evaluate** Niles programs; it does not need stage 0 to emit machine code. Reading "stage-0 compiler" as "native compiler" is what had made stage 1 unreachable, and it was a mistake of this document rather than of the design.

`niles-interp` is therefore a tree-walking interpreter for Niles's imperative subset — functions, `let` with patterns, `if`/`while`/`loop`/`for`/`match`, `break`/`continue`/`return`, blocks with tail values, checked integer arithmetic, strings, arrays, structs, tuple-payload enums and closures. It refuses the relational tier by name (`txn`, `hold`, `fx`, `fixpoint`, `view`, pipelines) rather than approximating it, because an interpreter that evaluated `txn` as a plain block would be one in which a program could appear to conserve money with nothing checking that it did. It also refuses float literals and money arithmetic: the first because cross-target determinism (C.4) and money-is-not-a-float are the same commitment, the second because the currency check lives in the type system and a second, unchecked path to the same operation is the seam §6.9 refuses. Environments are `BTreeMap`, so no hash seed can reach the output.

`bootstrap/lexer.niles` is a lexer **written in Niles** — the first line of the Niles compiler written in Niles, and stage 1's first input. It covers identifiers, keywords (all 174, generated from the registry), integers, money literals, epoch literals, strings, comments and the full punctuation table. It does *not* cover instants, durations, byte strings or raw identifiers; E.19 records that, and the equivalence gate excludes those constructs visibly rather than letting them pass.

`bootstrap/parser.niles` is a recursive-descent parser **written in Niles**, 1,647 lines, loaded with the lexer as one program. It builds a tree rather than emitting text — `enum Node { Atom(str), List([Node]) }`, walked afterwards by `render_node` — because the claim under test is that Niles can hold a syntax tree, not that it can concatenate strings in the right order; and it keeps the lexer's no-aliasing calling convention, since the subset has nothing that could hold a mutable cursor. It covers the imperative subset the interpreter executes, including the full operator table of B.10.1, and excludes the relational and SQL surfaces visibly. **The status line for E.4 and E.5 is therefore: lexer and parser self-hosted; type-checking and IR lowering are Rust.**

**Designed and specified, not built**: everything from E.6 onward — the self-hosted middle end, the WASM-hosted optimizer, instruction selection, register allocation, machine-code encoding, object emission and linking. The stage-0 front end still accepts a large but proper subset of the whole language: no trait solver, no monomorphisation, no native code generation. A **type-checker and a lowering pass** written in Niles remain unwritten, so what E.18's gates now cover is two front-end stages rather than a compiler.

The reader should read E.1–E.5 as a specification with a partial implementation behind it, and E.6–E.19 as a specification with none. Where a section makes a claim about an artifact — "adding the WSL profile required a record and linker flags, and no compiler-code change" in E.2 — that claim describes the *design's intent* and has not been demonstrated, because the back end it would be demonstrated in does not exist. §11.2 lists an unfinished instrument as the most likely failure mode of this project, and this appendix is where that risk is largest.

**Why the appendix is retained in full despite that.** Because the boundary argument in E.19 — what belongs in the language and what stays in the host — is a design result that can be stated and criticised without the code, and because a phased programme whose later phases are undocumented cannot be evaluated for feasibility, which is what a committee is being asked to do. The honest position is to describe the design completely and label its status precisely, rather than to shorten the appendix and thereby hide how much remains.

## E.1 Self-Hosting Strategy: The Three-Stage Bootstrap

Stage 0: a compiler written in the host language, compiling the full Niles language (slowly, with unoptimized output). Stage 1: the Niles-written compiler sources, compiled by stage 0. Stage 2: stage 1 recompiling the same sources. Stage 3: an optional further self-build used as a consistency check.

Self-hosting is not vanity. It is the strongest available test of the language's expressiveness claim — a compiler is a large, pointer-heavy, performance-sensitive program with none of the shape of a banking workload — and it keeps the language's ergonomics answerable to authors who must live inside it. Section E.19 states precisely what the boundary is and why it holds.

## E.2 The Target Model as Data

All target knowledge lives in declarative records: word width, endianness, alignment rules, calling convention (parameter and return registers, caller/callee-saved sets, stack alignment), relocation kinds, object-format parameters, and platform shims. Back-end code is generic over the record, and encoder tables (E.15) are generated from it. Adding the WSL profile required a record and linker flags, and no compiler-code change — which is the intended proof of the design.

## E.3 The Host Shim

The self-hosted compiler runs on a deliberately tiny host shim: read file, write file, arguments, exit code, and (for the wasm-hosted optimizer) a fuelled execution loop. Everything else — including memory management through the language's own arena model — is Niles. The shim is small enough to audit by reading, which is what keeps the bootstrap's trusted base honest.

## E.4 Front-End in Niles

A hand-written lexer covering the literal forms of Appendix B.2 (money with per-currency scale checking, temporal and value-date literals, epochs, idempotency keys) and a recursive-descent parser with Pratt expression parsing, producing the AST datatypes stage 0 defines through a shared serialized form. Error recovery synchronizes at item and statement boundaries; every diagnostic carries a span, a stable code, and a fix hint where computable.

## E.5 Type-Checking and IR Lowering in Niles

The self-hosted checker re-implements: inference with local annotations; coherent trait resolution; the linearity checker for posting halves and holds; the effect-row checker; the currency-row solver; and confidentiality flow checking against static labels. Lowering produces the IR of Appendix D with provenance-derived upquery paths.

**Differential gate.** The stage-0 and stage-1 checkers must accept and reject an identical corpus with identical diagnostic *codes* (wording may differ). This is the check that catches the most dangerous class of bootstrap bug: a self-hosted checker that is subtly more permissive than the one that compiled it.

## E.6 Middle-End: Planner and Optimizer

The middle end holds the query planner — circuit construction, join ordering by cost model, delta-form selection, upquery-path derivation, and initial materialization-mode defaults handed to the runtime optimizer — and a scalar optimizer over an SSA form for imperative and UDF code. Planner decisions are recorded so that plans are explainable.

## E.7 The Aggressive Optimizer, Wasm-Hosted

The higher optimization level is itself a Niles program compiled to wasm32 and executed through the same sandboxed runtime UDFs use. Two reasons, and both are more than aesthetic: it dogfoods the UDF tier at the largest scale available, and it makes optimizer passes safely pluggable, since a buggy pass can exhaust fuel or trap but cannot corrupt the compiler. Its ambition is bounded and stated: reach respectable native performance on the compiler's own workloads, not re-create a mature optimizing back-end.

## E.8 Pass Pipeline

- **O0:** lower plus naive register allocation.
- **O1:** SSA construction, constant folding and propagation, dead-code elimination, control-flow simplification, size-bounded inlining, scalar replacement.
- **O2 (wasm-hosted):** global value numbering and common-subexpression elimination, loop-invariant code motion (E.10), bounded peephole superoptimization (E.11), bounded unrolling, tail-call formation, block layout by branch-frequency estimates.

## E.9 Optimizer Driver

The driver loads content-addressed pass modules, runs them over serialized SSA with per-pass fuel accounting, and emits pass reports consumed by the explain tooling. Passes are pure functions from SSA to SSA, and the determinism gate hashes the SSA after every pass on every target.

## E.10 Representative Pass: Loop-Invariant Code Motion

Standard LICM — dominator tree and loop forest, invariance by reaching definitions, hoisting to a preheader with speculation forbidden — with one domain-specific twist: **anchor reads and effectful calls are never hoisted across an epoch-boundary intrinsic**, and the legality predicate consults the effect rows to decide. It is a small but satisfying demonstration that the effect system pays for itself inside the compiler, not only in user code.

## E.11 Representative Pass: Bounded Peephole Superoptimization

Over windows of a few instructions in hot blocks: enumerate replacement candidates from an algebraic identity table and a target cost table; verify semantic equality by exhaustive checking over small bit widths and bounded solving for full widths; replace only when strictly cheaper. Verified rewrites are cached content-addressed, so the expensive search amortizes across builds rather than being repeated.

## E.12 O2 Adds No Native Surface

An invariant, enforced mechanically: enabling the higher optimization level changes *performance only*. The wasm-hosted optimizer's output passes the same verifiers as unoptimized output; no pass may introduce operations outside the verified instruction set; and the determinism gate runs the conservation suite at every level demanding identical answer bytes. Optimization is therefore outside the trusted base — a property worth more than any speedup it delivers.

## E.13 Optimization Levels and the Remaining Gap

The measured gap between the self-hosted back-end and the established optional back-end is published per release rather than hidden. The honesty rule: numbers in Chapter 9 come from the self-hosted back-end; the gap is tracked as an engineering metric, not presented as a scientific result.

## E.14 Code Generation: Instruction Selection and Register Allocation

Selection is tree-pattern matching over SSA with target tables from E.2, using maximal munch with explicit tie-break order. Register allocation is linear scan over live intervals with interval splitting and spill-cost heuristics weighted by loop depth, with callee-saved registers allocated last to minimize prologue cost — the classical algorithm, chosen because it is simple enough to be self-hosted and verified rather than because it is the best available.

## E.15 Machine-Code Encoding and Object Assembly

Encoders are table-generated from the target model: fixed-width encodings for the RISC target, and the modifier/prefix machinery for the CISC target. Object writers emit the three platform formats with relocations, symbol tables, unwind skeletons, and minimal line tables for debugging. An exhaustive encoder test compares output against a reference disassembler corpus per release.

## E.16 Static Linking and the WSL Profile

The self-hosted linker performs static linking: symbol resolution across objects and the compiled standard library, relocation application, section layout, and entry-point synthesis calling the host shim's runtime initialization. The WSL profile is the Linux target with fully static linking and path-normalization shims, chosen so that a Windows/WSL user gets one binary with no distribution friction. It exists as a target record plus a linker profile, exercising E.2's claim that targets are data.

## E.17 Optional Host-Side Paths

Two escape hatches, both outside the bootstrap: an established optimizing back-end for release builds and for the gap measurement of E.13, and system dynamic linking where platform integration demands it. Neither participates in stage identity or the determinism gates, and both re-verify against the conservation suite when used.

## E.18 Building and Self-Verifying the Bootstrap

The bootstrap target runs stage 0 (pinned host toolchain) → stage 1 → stage 2 → stage 3, with gates:

- **G-a** stage 2 and stage 3 are bit-identical per target.
- **G-b** the cross-compilation square: building target B's compiler on A and natively on B yields identical binaries.
- **G-c** the compiled compiler passes the full language corpus with diagnostic-code equality against stage 0.
- **G-d** the conservation suite produces identical output hashes at every optimization level.

## E.19 What the Gates Actually Prove — and What They Do Not

This section exists because the natural claim here is wrong, and stating it wrongly would be a real vulnerability.

**What a bit-identical stage-2/stage-3 build proves.** That the compiler is a **fixed point**: compiling the compiler with itself reproduces itself. This detects *accidental* non-determinism — hash-map iteration order, timestamps, path leakage, address-layout-dependent code generation — and some classes of miscompilation. That is genuinely valuable, and it is why the gate exists. It is also, precisely, what the mature language toolchains claim for the analogous check: their own documentation presents the extra stage as a sanity check to detect breakage, explicitly optional, and does not claim it establishes trustworthiness.

**What it does not prove.** It does not detect a trusting-trust attack. A self-reproducing trojan is a fixed point *by construction* — that is the entire mechanism Thompson described — so a bit-identical self-build is exactly what a successful attack looks like [Thompson, CACM 1984]. Self-consistency and trustworthiness are different properties, and only reasoning that introduces *diversity* addresses the second.

**What would.** Diverse double-compiling: compile the compiler's source with a second, independently sourced trusted compiler, then recompile the source with that result; bit-for-bit identical output implies source and executable correspond, and an attacker must then have subverted *every* compiler used [Wheeler, ACSAC 2005]. Its stated requirement is that the parent compiler be deterministic when compiling the compiler under test — which is to say that **reproducibility is a precondition for the real defence, not a substitute for it**. Reproducible builds are defined by exactly that property: given the same source, environment and instructions, any party can recreate bit-identical artifacts.

**The honest position.** The gates of E.18 deliver reproducibility and self-consistency, which are prerequisites and are worth having. Executing a genuine diverse double-compilation of the Niles bootstrap is future work (Chapter 12), and the thesis claims no trusting-trust guarantee until it is done.

### E.19.1 The Gates That Now Run, and What Each Establishes

`crates/niles-interp/tests/bootstrap_stages.rs` runs fourteen tests over the lexer; `crates/niles-interp/tests/bootstrap_parser.rs` runs sixteen more over the parser. Their scope is a *front end*, not a compiler, and the tables say so in each row.

| Gate | What it runs | What it establishes | What it does not |
|---|---|---|---|
| **Stage 1 runs** | `bootstrap/lexer.niles` under `niles-interp` | Stage 0 executes a ~450-line Niles program end to end | Nothing about the rest of the language |
| **Stage 1 equivalence** | Niles lexer vs. Rust lexer over a 29-case corpus | The two agree on every token's kind, span and text | Only over the declared scope; four literal forms are excluded |
| **Stage 2 self-application** | Niles lexer over its own source | It lexes itself, identically to the reference | Not "recompiles itself" — a lexer compiles nothing |
| **Stage 3 fixpoint** | Repeated runs, and repeated self-application | Byte-identical output run to run | **Run-to-run on one target only**; cross-target determinism (C.4) remains unmet |

And, one stage up:

| Gate | What it runs | What it establishes | What it does not |
|---|---|---|---|
| **Stage 1 runs** | `bootstrap/parser.niles` under `niles-interp` | Stage 0 executes a ~1,050-line Niles program that builds and walks a recursive tree | Nothing about type-checking |
| **Stage 1 equivalence** | Niles parser vs. Rust parser over a 48-case corpus | The two agree on **every node of every tree** | Spans are excluded by design — token offsets are the lexer's gate |
| **Stage 2 self-application** | Niles front end over its own two source files | It parses 2,068 lines of Niles, including itself, identically to the reference (127,165 bytes of tree) | Still not "recompiles itself": no code is generated |
| **Stage 3 fixpoint** | Repeated runs, and repeated self-application | Byte-identical output run to run | As above |
| **Negative controls** | A missing brace; a token that cannot begin an expression; `a - b - c`; `a = b = c` | Both parsers recover to the *same* tree, and associate the way B.10.1 says | The Niles parser has no diagnostic channel; see below |

Comparison is on a rendering, not on structures: the two parsers share no types, so a structural comparison would need an adapter, and an adapter is the one thing a gate must not be, since a defect in it cancels a defect in either side. Both sides render independently to the S-expression form defined in `niles_lang::sexpr`, in which optional slots are never elided — an absent `else` renders `(none)` — so two different trees cannot produce one string.

The corpus is chosen for the decisions a hand-written lexer actually gets wrong: longest-match punctuation (`::` before `:`, `|>` and `||` before `|`, `=>` and `==` before `=`), the sigil rule that distinguishes `#4200` from `#[attr]`, the rule that distinguishes `10.00 usd` from `10` followed by `.days`, escapes inside strings, and **unterminated constructs at end of input**. That last class is deliberately *not* excluded: a malformed input is exactly where two lexers most easily disagree, because each must decide independently where the broken construct ends, and dropping those cases would have removed the hardest evidence in the name of tidiness.

**The gate can fail, and did — four times, on its first run.** This is worth recording, because a gate that has never rejected anything is a gate with no demonstrated discriminating power.

1. The Niles lexer's keyword table listed `evict` and `conserves`. Neither exists; the registry's words are `evictable` and `conserve`.
2. It omitted `from` and `post`, which do exist.
3. It matched keywords **case-sensitively**. Niles inherits case-insensitive keywords from SQL, so `Epoch`, `EPOCH` and `epoch` are one word — a rule a lexer written by someone thinking in Rust gets wrong, because Rust's half of Niles's ancestry is case-sensitive. The two lineages disagree here and the registry settles it.
4. The gate's own scope filter initially excluded any input the reference lexer reported an error on, which silently removed the two unterminated-construct cases. That was a defect in the gate, not in either lexer, and it is the kind a self-written gate is most likely to have.

Findings 1 and 2 are the argument for the keyword registry (§6.4) arriving as evidence rather than assertion: two implementations of one language drifted apart within a day of the second one existing. The response was not to fix the table but to remove the possibility — the Niles-side table is now **generated from `keywords.rs`**, and `keyword_table_matches_the_registry_exactly` fails the build if the two ever diverge again. The single source of truth crosses the bootstrap boundary; it does not stop at the host language.

### E.19.2 The Parser Round, and the Three Defects It Rejected

The parser gates failed in three different artefacts before passing, and the three are worth separating because only one of them is the kind of defect the exercise was designed to find.

1. **In the stage-0 interpreter.** A tree-walking interpreter spends host stack in proportion to the interpreted program's call depth, and a `match` over thirty expression variants compiles, unoptimised, to a frame holding the union of every arm's locals. Measured: ~95 KB of host stack per Niles call frame in a debug build against 4.8 KB in a release build — a twentyfold build-mode penalty — which put a recursive-descent parser out of reach on any ordinary stack, and whose failure mode was a **process abort with no diagnostic**, because a stack overflow in Rust does not unwind. Moving the cold arms behind `#[inline(never)]` brought debug to 32 KB, and a call-depth counter now turns the remaining limit into an ordinary error carrying a span. The default ceiling is the depth that fits a 1 MB stack in the widest build; the first value tried assumed a 2 MB thread stack and aborted the test process, which is how the number came to be measured rather than assumed.

2. **In the reference parser.** Assignment was **left**-associative. `expr_bp` recursed for the right-hand side at binding power 1, which placed assignment outside its own `min_bp == 0` guard, so `a = b = c` parsed as `(a = b) = c` — the opposite of Rust's rule and of the comment sitting directly above the code. This is the finding the exercise exists for. It had survived every test in the workspace because associativity is invisible in a token stream: the lexer round could not have found it, and nothing else was looking. It surfaced within minutes of a second implementation of the same grammar existing.

3. **In this thesis.** §6.25 makes Appendix B normative — where the appendix and the implementation disagree, the appendix wins — but B had no operator precedence table, so the rule had nothing to adjudicate with and finding 2 was arguable rather than decidable. B.10.1 now states precedence and associativity normatively, and `crates/niles-lang/tests/precedence.rs` reads the table out of the markdown and checks every level against the compiler's own `BinOp::precedence`, so the two cannot drift and the lineage rule is enforced rather than declared.

Finding 2 is the one to carry forward as a claim. The argument for stage-1 equivalence has until now been made in the future tense — that a second implementation *would* catch disagreements a single implementation cannot see. It has now caught one, in the reference implementation, in a construct every program uses.

**One gap, declared rather than closed.** `bootstrap/parser.niles` has no *diagnostic* channel. Where the reference parser builds an error node, the Niles parser builds the same error node and the trees agree; where the reference parser reports a message and builds an ordinary node — a reserved word used as an identifier is the case that arises — the Niles parser is silent. A test pins both halves, so the gap stays known rather than becoming a surprise. Diagnostics in Niles are part of the same work as the type-checker, and neither is written.

**What the gates do not establish, stated plainly.** They do not establish that Niles can express a compiler. They establish that it can express a lexer and a parser, that both agree with an independent implementation — the parser node for node, over a corpus and over 2,068 lines of real source including its own — and that both are deterministic. The distance from here to E.1's stage 2 — "stage 1 recompiling the same sources" — is a type-checker and a lowering pass, neither of which is written. The bootstrap has gone from *no input* to *two front-end stages, verified against the reference implementation*. That is two rungs, and calling it a ladder would be exactly the overclaim this appendix's E.0 exists to prevent.

## E.20 Thesis Consistency: What Is in Niles and Why the Boundary Holds

After bootstrap, written in Niles: the front end, the checker, the planner, the middle end, the higher optimizer (as wasm), the back-ends, and the linker — that is, the compiler. Still in the host language: the engine (ledger, core runtime, server) and the host shim.

The boundary is principled rather than accidental. The engine is the *object* the theory speaks about, and it must stay inside the audited memory-model claims of Sections 3.17 and 5.8, where an ownership-typed host language with a machine-checked soundness result is the instrument of that audit. The compiler is a *user* of the language, and is precisely where self-hosting has scientific value as expressiveness evidence. Migrating engine components into Niles is future work gated on the language growing a systems-programming tier — a gate Section 6.12 keeps deliberately closed, because opening it would enlarge the trusted base the soundness theorem depends on.
