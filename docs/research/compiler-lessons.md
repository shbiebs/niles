I have gathered sufficient primary sources. Here is the research document.

---

# Transferable Engineering Lessons from rustc and SQL Compilers, for the Niles Language and Nilestream

**Scope and method.** Every claim below is anchored to a primary source (compiler dev guides, RFCs, peer-reviewed papers, engine documentation, or source repositories) fetched during this research. Where I could not verify a figure or where the framing in the task brief is factually off, I say so explicitly in the **Correction** notes and in the consolidated §18.

---

## Part I — Lessons from the Rust compiler toolchain

### 1. Query-based incremental compilation (rustc_middle queries, red–green tracking)

**The design.** rustc is not a pipeline of passes. It is "a set of function-like *queries* [that] compute information about the input source" [1]. Queries are memoized: results are cached, and "when we execute a query, we also discover (at runtime!) what other queries it depends on" [1], which builds a dependency DAG *incidentally* rather than by declaration. `TyCtxt<'tcx>` is the query context every access goes through; because "every access from one query to another has to go through the query context, we can record these accesses and thus actually build this dependency graph in memory" [3].

Reuse is decided by the **red–green algorithm**: "if all the inputs to query Q are colored green, then the query Q **must** result in the same value as last time and hence need not be re-executed" [2]. When an input *is* red, the query is re-executed and its result hashed; if the hash matches, the node is still marked green, stopping the invalidation wavefront [2]. This is `try_mark_green` [3].

**Why it was chosen.** It replaces a fixed pass ordering with demand-driven evaluation, and it makes incrementality a property of the *architecture* rather than a bolt-on. It also works across crate boundaries [1].

**What it costs — and this is the part usually under-reported.** The dev guide is unusually candid:

- Result hashing must be *stable*: "whenever something is hashed that might change in between compilation sessions (e.g. a `DefId`), we instead hash its stable equivalent (e.g. the corresponding `DefPath`)" [3].
- "**Computing fingerprints is quite costly. It is the main reason why incremental compilation can be slower than non-incremental compilation.**" [3]
- Fingerprints are 128-bit hashes, so "there is a small possibility of hash collisions… the system would erroneously assume that the result hasn't changed, leading to a missed update" [3]. Incremental correctness is therefore *probabilistic*, not absolute.
- The on-disk cache cannot be updated in place: "it has to rewrite each file entirely in each compilation session. The overhead of doing so is a few percent of total compilation time" [3].
- Query result *shape* is a performance decision: "Especially 'span' information is very volatile, so including it in query result will increase the chance that the result won't be reusable" [3].
- Query providers must be pure; global mutable state is eliminated [4].
- LWN's assessment: the approach "introduces a lot of complexity compared to a simple pipeline-based approach" [4].

**Lesson transferable to Niles.** Model Nilestream's view-maintenance planner as a memoized query graph keyed on stable, epoch-independent identities (not on spans or physical offsets), and budget explicitly for the fingerprinting overhead — rustc's own guide says hashing, not recomputation, is the dominant incremental cost.

**Second lesson.** The red–green *"re-run then compare hashes"* fallback is structurally the same move as a Niles upquery that reconstructs evicted state and then discovers the reconstruction is identical to what was evicted. Both buy invalidation-wavefront truncation at the price of speculative recomputation. Cite [2], [3] as prior art for the cost accounting in Contribution 3.

---

### 2. The multi-IR pipeline: AST → HIR → THIR → MIR → LLVM-IR

**The design.** Seven representations, each narrower than the last [5]:

| Level | What it is | What it is *for* |
|---|---|---|
| Tokens (`rustc_lexer`) | type-tag + text slice | pure lexing |
| AST | "pretty much exactly what the user wrote" [5] | macro expansion, name resolution, early lints |
| HIR | desugared; elided lifetimes made explicit | "amenable to type checking" [5] |
| THIR | "fully typed and a bit more desugared" — autoref/autoderef explicit, operators become plain calls [6] | MIR construction, exhaustiveness checking, unsafety checking [6] |
| MIR | CFG of basic blocks, no nested expressions, all types explicit [7] | borrow check, dataflow, const eval, MIR opts |
| LLVM-IR | "typed assembly language with lots of annotations" [5] | backend optimization + codegen |

**Why MIR specifically.** RFC 1211 states the motivation bluntly: "The current compiler uses a single AST from the initial parse all the way to the final generation of bitcode. While this has some advantages, there are also a number of distinct downsides" [8]. The enumerated downsides are the argument for *any* mid-level IR, and every one of them applies to a query language:

1. Every pass must handle the full surface language rather than a reduced core.
2. Desugarings (closures, `for`, overloaded operators) get re-derived in multiple phases.
3. Control-flow reasoning overlaid on an AST is "awkward and doesn't guarantee analyses match the actual generated code."
4. "The wide distance between what's analyzed (AST) and executed (bitcode) undermines confidence in safety guarantees."
5. Rust-specific optimizations are hard without a semantic mid-level.
6. **Backend lock-in**: "LLVM migration is nearly impossible since compiler semantics are embedded in the trans step rather than in AST-to-MIR lowering" [8].

**Why THIR exists as a separate level.** It is the level at which *types exist but control flow has not yet been flattened* — the only place where pattern-match exhaustiveness and unsafety are naturally checkable. Notably, it is also *ephemeral*: "each body of THIR is only stored temporarily and is dropped as soon as it's no longer needed," to control peak memory [6].

**Lesson transferable to Niles.** Your thesis proposes "three surfaces but one intermediate representation." RFC 1211's downside list argues that *one* IR is too few if that IR must simultaneously (a) carry the surface's sugar for diagnostics, (b) be typed enough for the consistency-effect calculus, and (c) be flat enough to lower to a DBSP circuit. The rustc answer is: split by *what property becomes checkable*. Concretely, a defensible Niles pipeline is `AST → NHIR (desugared, effects still implicit) → NTHIR (fully typed; money/currency/bitemporality resolved; the level at which the Contribution-4 soundness theorem is stated) → NMIR (a DBSP-compatible operator DAG with epochs explicit; the level at which the versioned partial-state algebra is defined) → circuit`. State the soundness theorem over NTHIR and the reconstruction theorem over NMIR, and you get RFC 1211's point 4 for free: the analyzed object and the executed object are close.

**Second lesson (backend lock-in).** RFC 1211 point 6 is the strongest argument in your favor for putting a typed IR between Niles and Nilestream's runtime: without it, the DBSP-circuit choice becomes irreversible.

---

### 3. Interning, arenas, and identity schemes (`Symbol`, `NodeId`/`DefId`/`HirId`)

**Arenas and interning.** "Since A LOT of data structures are created during compilation, for performance reasons, we allocate them from a global memory pool" [9]. Types are interned so that "for each interned type `X`, we implemented `PartialEq` for `X`, so we can just compare pointers" [9]. The `'tcx` lifetime ties every arena-allocated value to the compilation session: "when compilation finishes, all the memory related to that buffer is freed" [9]. Identifiers get the same treatment — the higher-level lexer performs "string interning—storing only one immutable copy of each distinct string value" [5].

**Identity schemes.** Three distinct ID families, each with a different scope and stability contract [10]:

- **`DefId`** = `CrateNum` + `DefIndex`. "Identifies a particular definition, or top-level item, in a given crate." Cross-crate, and **stable across compilations** — this is what makes incremental caching possible. Not every expression has one.
- **`LocalDefId`** — crate-local `DefId`, using the type system to prevent passing a foreign definition where a local one is required.
- **`HirId`** = owner + `local_id`. Identifies any HIR node including fine-grained expressions, but is crate-local.
- **`BodyId`** — a `HirId` wrapper for executable bodies.

Crucially, the guide explains *why* HIR stores item contents out-of-band rather than nested in parents: (i) accessing a node through its ID lets the compiler track the dependency, and (ii) maps allow iterating all items without a full traversal [10]. And incrementality imposes a hard constraint: "ID assignment must be deterministic so the same code produces identical results across sessions" [3].

**Lesson transferable to Niles.** Design at least two ID families before writing the parser: a **stable, content-derived, cross-session identity** for schema-level definitions (tables, views, currencies, account types) — the analogue of `DefId`, and the thing a REV definition hashes into — and a **cheap, session-local, dense index** for expression nodes. Do not conflate them; rustc's incremental correctness rests on the distinction. The `local_id`-within-owner trick (`HirId`) is directly worth copying: it makes a node ID *relatively* stable, so editing one view definition does not renumber every node in the catalog.

**Second lesson.** Intern currency codes, account identifiers, and view names into a `Symbol` table so equality is a pointer/index compare. In a system where the conservation-of-money check runs per epoch, `Currency == Currency` being a `u32` compare rather than a string compare is not a micro-optimization.

---

### 4. Diagnostics as a first-class subsystem

**The design.** A `Diagnostic` is structured, not a string: severity level, error code, main message, a diagnostic window with spans, and sub-diagnostics [11]. `Span` "is the primary data structure in `rustc` used to represent a location in the code being compiled" [11]. Most errors carry a code (e.g. `E0308`) resolving to a long-form explanation via `--explain`; the error index currently spans roughly `E0001`–`E0806` [12].

**What actually made the errors good** — three things, and they are all *policy*, not technology:

1. **Suggestions are typed by confidence.** Every structured suggestion carries an `Applicability`: `MachineApplicable`, `HasPlaceholders`, `MaybeIncorrect`, or `Unspecified` [11]. Only `MachineApplicable` may be auto-applied by `rustfix`. This is the mechanism that makes automated migration safe (see §8).
2. **A written style contract.** "Write in plain simple English"; no capitalization or trailing period unless multi-sentence; identifiers in backticks; messages "matter of fact" [11].
3. **A span-sufficiency rule.** A primary span must include "enough text to describe the problem in such a way that if it were the only thing being displayed… it would still make sense" [11].

Also load-bearing: rustc distinguishes *lints* (configurable, allow/warn/deny/forbid) from *hard errors* (non-negotiable, e.g. borrowck) [11].

**Lesson transferable to Niles.** For a language whose selling point is that double-entry invariants, currency mismatch, and unauthorized overdraft are *statically* caught, the diagnostic is the product. Three concrete imports:

- Assign a Niles error-code namespace up front (`N0001`…) with `niles --explain N0142`, and make error-code assignment a review gate. This also gives your Contribution-4 soundness theorem a *testable surface*: each rejection rule in the calculus maps to a code, and the UI test suite (§9) pins the exact message.
- Adopt `Applicability` verbatim. A suggestion like "add `in USD`" is `MachineApplicable`; "this ledger entry may need a matching credit" is `MaybeIncorrect`. Without this taxonomy an auto-fixer is unsafe.
- Split the checks into *lints* (e.g. "this view has no eviction policy") and *hard errors* (e.g. "unbalanced double-entry transaction"). Your consistency-effect calculus should say explicitly which rung each check sits on.

---

### 5. Lexer/parser split: `rustc_lexer` as a reusable crate

**The design.** `rustc_lexer` states its own rationale in its crate docs:

> "The idea with `rustc_lexer` is to make a reusable library, by separating out pure lexing and rustc-specific concerns, like spans, error reporting, and interning. So, rustc_lexer operates directly on `&str`, produces simple tokens which are a pair of type-tag and a bit of original text, and does not report errors, instead storing them as flags on the token." [13]

> "Tokens produced by this lexer are not yet ready for parsing the Rust syntax. For that see `rustc_parse::lexer`, which converts this basic token stream into wide tokens used by actual parser." [13]

The lexer is **hand-written, not generated** [14]. The `rustc_parse` layer ("cooking") adds spans, interns identifiers into `Symbol`s, and joins multi-character punctuation.

**Why the split.** Errors-as-flags rather than errors-as-diagnostics is the key move: it makes the lexer a total function on any `&str`, which is what lets it be shared. rust-analyzer consumes it (republished as `ra-ap-rustc_lexer`) [15]. Nethercote documents the real cost of the layering: rustc actually has **three** token representations — `rustc_lexer`'s *split* form (`&&` is two `&`), `rustc_ast`'s *joined* form, and `proc_macro::TokenStream`'s split-with-`Spacing` form — and conversion between them requires re-splitting and re-joining. He calls it "a mixture of the two. It's complicated" [16]. The underlying cause: "Not until full parsing is done can this decision be made, based on the syntactic context" [16].

**Lesson transferable to Niles.** Ship `niles-lexer` as a standalone crate with no dependency on spans, interning, or diagnostics, storing lexical errors as flags on tokens. You get, free: a formatter, a syntax highlighter, an LSP server, and a fuzz target, all sharing one definition of "what a token is." **But** — heed [16] — decide *once*, in writing, whether compound operators are split or joined at the raw-token layer, and do not introduce a third representation for UDF macros. rustc's three-way conversion mess is a warning, not a model.

---

### 6. Trait solving, type inference, and the chalk reformulation

**Inference.** `InferCtxt` holds inference variables in several flavours — general type variables, integral variables (from `22`), float variables (from `22.0`), region variables, and const variables [17]. Equality is enforced via `infcx.at(...).eq(t, u)`; success returns `InferOk<()>` carrying trait *obligations* still to be discharged [17]. Snapshots allow "backtrack[ing], trying out multiple possibilities before settling on which path to take" [17].

**Regions are handled differently, and this is the important architectural fact.** rustc does *not* unify regions eagerly. It "simply collect[s] constraints as we go, but make (almost) no attempt to solve regions," and "region constraints are only solved at the very end of typechecking, once all other constraints are known and all other obligations have been proven" [17].

**Chalk.** The premise was that "the Rust trait system is basically a kind of logic, and it can be mapped onto standard logical inference rules," so it "recasts Rust's trait system explicitly in terms of logic programming" [18].

**Correction to the brief.** The framing "why the chalk reformulation was attempted" is right, but the *outcome* is commonly misreported. What shipped is **not** chalk. It is `rustc_next_trait_solver` / `-Znext-solver`, built by the Trait System Refactor Initiative — informed by chalk's ideas but implemented in-tree. As of August 2026 it was enabled on nightly after "nearly 4 years of active development," described by the initiative as "the largest single change to the Rust compiler since its initial release," fixing "more than 200 issues on GitHub" and unblocking Type Alias Impl Trait and Return Type Notation [19]. The dev-guide chalk chapter is dated May 2022 and does not reflect this [18]. Chalk-as-a-shipping-component should be described as *not adopted*; chalk-as-a-specification-vehicle should be described as influential.

**Lesson transferable to Niles.** Two, both strong:

- **The region model is the template for your consistency-effect calculus.** Consistency levels and retention/immutability obligations are exactly like lifetimes: they are *ordering/outlives constraints*, they are cheap to *collect* during type-checking and expensive to *solve*, and solving them early over-constrains. Collect consistency obligations as constraints during Niles type-checking, and discharge them in one late pass — this is precisely rustc's `'a: 'b` architecture, and citing [17] gives you a real-system precedent for Contribution 4.
- **The chalk history is a warning about rewrite scope.** A "recast the whole system in a cleaner formalism" project took ~4 years to land even at rustc's resourcing level, and the thing that landed was a *third* design, not the formalism as originally specified [18], [19]. If Niles's effect calculus is to be both proved and implemented, keep the proved fragment small and explicitly bounded.

---

### 7. The bootstrap: stage0/1/2(/3), determinism, and the OCaml history

**The design.** [20]

- **stage0** — "the pre-compiled compiler and standard library," typically a recent **beta**, downloaded automatically.
- **stage1** — built "from current code, by an earlier compiler." Compiled by stage0, so "not by the source in your working directory," which creates potential ABI mismatch.
- **stage2** — "the truly current compiler": stage1 rebuilds the compiler using the in-tree standard library, giving ABI consistency.
- **stage3** — optional verification: rebuild libraries with stage2; "the result ought to be identical to before, unless something has broken" [20].

**Why more than one stage.** "The compiler source code can't use some features until they reach beta (because otherwise the beta compiler doesn't support them)" [20] — the chicken-and-egg problem — plus the ABI argument above.

**History.** Rust was originally written in OCaml, in a compiler called **rustboot**; the modern chain descends by "backtrac[ing] through history, all the way back to the last version of rustboot" [21]. Users never do this: "one downloads a stage0 compiler as a binary snapshot to compile Rust from source" [21]. The chain worked by using rustboot to build the first Rust-written compiler as a snapshot, then "use this snapshot to build the next one, following the chain of snapshotted commits" [21]. The C++ dependency is LLVM only, not the snapshot mechanism [21].

**Correction to the brief.** Two points. (a) *Three stages is a slight mis-statement.* The functional chain is stage0 → stage1 → stage2; stage3 exists solely as the determinism check and is not built by default. (b) *The determinism check is not `x.py build --stage 2` compared against itself.* It is the stage2-vs-stage3 same-result test [20]: you compare artifacts produced by stage1 against artifacts produced by stage2 from identical source. (c) The "snapshot compiler" era is over — modern stage0 is a released beta, not a hand-blessed snapshot; the snapshot chain is history, not current practice.

**Lesson transferable to Niles (Appendix E).** Your three-stage bootstrap plan is architecturally correct and matches rustc, but state the stage properties precisely:

- **stage1 is not a valid determinism reference** (built by a different compiler, potentially different ABI).
- **The fixpoint test is stage2 ≡ stage3**, i.e. "compiler compiled by itself twice produces bit-identical output." This is the exact gate to specify in Appendix C's cross-target determinism obligation, and citing [20] gives it provenance.
- For the Rust host shim: rustc's own history shows the host language can be *fully retired* once the fixpoint holds, but only because a signed binary snapshot chain was retained. Plan for a published, reproducible stage0 artifact, not a "build it from Rust each time" story.

---

### 8. Editions as a language-evolution mechanism

**The design.** "When there are backwards-incompatible changes, they are pushed into the next edition. Since editions are opt-in, existing crates won't use the changes unless they explicitly migrate" [22]. The canonical case is new keywords: adding `async` unconditionally would break `let async = 1;` [22].

**The two guarantees that make this work.** From RFC 2052 [23]:

- **Forward compatibility of correct code**: "Warning-free code on edition N must compile on edition N+1 and have the same behavior."
- **A single core**: "The Rust compiler supports multiple editions, but must only support a single version of 'core Rust' (MIR and trait system)."

And from the edition guide: "All Rust code — regardless of edition — will ultimately compile down to the same internal representation," so edition differences are "skin deep" [22]. Interop is mandatory: "crates in one edition **must** seamlessly interoperate with those compiled with other editions" [22], so migration decisions stay private to a crate and the ecosystem cannot fork.

**Migration.** `cargo fix --edition` / `rustfix` automates it — e.g. rewriting a variable named `async` to `r#async` [22] — and RFC 2052 sets the bar: "changes that cannot be automated must be required only in a small minority of crates" [23]. The guide is honest that "Cargo's automatic migrations aren't perfect" [22]. Editions permitted so far: converting deprecations to hard errors, adding keywords, repurposing corner cases via deprecation→error; **forbidden**: anything touching trait coherence, standard-library protocols, or requiring non-crate-local edits [23].

**Lesson transferable to Niles.** This is the single most directly copyable mechanism in the whole brief, because Niles is a *keyword-hungry* language (money literals, epoch keywords, consistency annotations, confidentiality annotations, all the banking domain vocabulary — Appendix B's novel keyword list is large and will grow).

- Ship an edition mechanism in v1, before you have users, and make **"one core IR, many surfaces"** an invariant: RFC 2052's "single version of core Rust" maps exactly onto your "three surfaces but one intermediate representation."
- Encode the *forbidden* list too: a Niles edition may add keywords and tighten checks, but must **never** change ledger semantics, epoch ordering, or on-disk representation. That is the difference between a language edition and a storage-format migration, and conflating them is how a ledger system loses auditability.
- Make automated migration a *gate on the design*, not an afterthought: a proposed edition change is admissible only if it is `MachineApplicable` (§4) for the overwhelming majority of programs.

---

### 9. Testing: compiletest, UI blessing, pretty-print round-trips, fuzzing

**compiletest.** "The main test harness of the Rust test suite," organizing thousands of annotated source files with parallel execution [24]. Suites include `ui` / `ui-fulldeps` (stdout/stderr snapshots), `mir-opt`, `codegen-llvm` / `assembly` (via FileCheck), **`incremental`**, `debuginfo`, `run-make`, and `coverage` [24]. It supports *revisions* (multiple configurations from one file) and *compare-modes* (re-running a suite under different compiler flags) [24].

**UI tests and blessing.** UI tests "check the stdout/stderr snapshots from the compilation and/or running the resulting executable," and `x test --bless` regenerates the expected files [24]. This is what makes §4's diagnostics durable: every message is pinned by a checked-in `.stderr` file, so a regression in wording is a failing test.

**Pretty-print round-trip — confirmed, and more interesting than the brief suggests.** compiletest's `pretty` mode does *two* things [25]:

- **Converging mode (default, 2 rounds):** pretty-print the source, then pretty-print *that* output, and assert round 0 == round 1 — i.e. the printer reaches a fixpoint.
- **Exact mode (`pp_exact`, 1 round):** compare against a checked-in reference.
- **Then, in both modes**, it type-checks the final pretty-printed output — the source comment reads: *"Finally, let's make sure it actually appears to remain valid code"* [25].

**Correction to the brief.** `cargo fuzz` is **not** how rustc is fuzzed. `cargo-fuzz` targets Rust *libraries*. The dev guide's fuzzing chapter names three tools [26]:

- **`fuzz-rustc`** — a libFuzzer harness;
- **`icemaker`** — "runs rustc and other tools on a large number of source files with a variety of flags to catch ICEs";
- **`tree-splicer`** — generates new sources by recombining existing ones while preserving syntactic validity.

The guide also gives operational rules: minimize inputs; use `--emit=mir` to skip LLVM; try `-Zmir-opt-level=4` and `-Zvalidate-mir`; and avoid seeding from known crashes because "a fuzzer that is seeded with such tests is more likely to generate bugs with the same root cause" [26].

**Lesson transferable to Niles.** Four:

- **Build the harness before the language.** compiletest-style annotated-source tests with `--bless` are the only economical way to maintain thousands of pinned diagnostics — and your Appendix B keyword catalogue implies thousands.
- **Copy the `incremental` suite idea directly.** rustc has a whole test suite whose job is verifying that incremental reuse produces the same answer as from-scratch compilation [24]. Nilestream needs the exact analogue: a suite asserting *evict-then-upquery-then-read ≡ never-evicted read*, per epoch. This is the executable form of your reconstruction theorem, and compiletest's `incremental` suite is the citable precedent for how to organize it.
- **Copy `compare-mode`.** Run the entire correctness suite once per consistency rung. Any test whose result differs between rungs is either a genuine staleness-visible test or a bug — and that partition *is* your consistency ladder's empirical content.
- **Copy the pretty-print fixpoint test.** For Niles: `parse → print → parse → print` must converge, and the final printed program must still type-check [25]. For a language that will have three surface syntaxes lowering to one IR, add the stronger cross-surface gate: `lower(rust_surface) ≡ lower(sql_surface)` on the IR, structurally.
- **Fuzz with `tree-splicer`-style structural recombination, not byte mutation.** For SQL-shaped languages, byte fuzzing mostly finds lexer bugs; recombining valid query fragments finds analyzer and planner bugs.

---

## Part II — Lessons from SQL compilers and parsers

### 10. PostgreSQL: dumb parse, separate analysis, and the `kwlist.h` registry

**The pipeline.** Five stages, verbatim from the docs [27]: connection → **parser stage** ("checks the query… for correct syntax and creates a *query tree*") → **rewrite system** (applies rules from the system catalogs to the query tree) → **planner/optimizer** ("first creating all possible *paths* leading to the same result… Next the cost for the execution of each path is estimated and the cheapest path is chosen") → **executor** (recursively steps the plan tree).

**Why parsing is deliberately dumb.** This is the sharpest single sentence in the whole research set, and it is the one to quote in your thesis:

> "The parser stage creates a parse tree using only fixed rules about the syntactic structure of SQL. It does not make any lookups in the system catalogs, so there is no possibility to understand the detailed semantics of the requested operations." [28]

And the reason, which is *operational*, not aesthetic:

> "The reason for separating raw parsing from semantic analysis is that system catalog lookups can only be done within a transaction, and we do not wish to start a transaction immediately upon receiving a query string." [28]

The transformation stage then "does the semantic interpretation needed to understand which tables, functions, and operators are referenced," producing the query tree. The canonical example: "a `FuncCall` node in the parse tree represents something that looks syntactically like a function call. This might be transformed to either a `FuncExpr` or `Aggref` node depending on whether the referenced name turns out to be an ordinary function or an aggregate function" [28].

**Scale.** `gram.y` (bison) is **21,016 lines / ~528 KB** [29]; `scan.l` (flex) handles identifiers and keywords [28].

**The `kwlist.h` registry — the pattern the brief wants.** The file is a single table with no code, consumed by whoever defines the macro [30]:

```c
PG_KEYWORD("abort", ABORT_P, UNRESERVED_KEYWORD, BARE_LABEL)
PG_KEYWORD("all",   ALL,     RESERVED_KEYWORD,   BARE_LABEL)
PG_KEYWORD("array", ARRAY,   RESERVED_KEYWORD,   AS_LABEL)
```

Header comment: "The keyword lists are kept in their own source files for use by automatic tools. The exact representation of a keyword is determined by the `PG_KEYWORD` macro, which is not defined in this file; it can be defined by the caller for special purposes" [31]. The file must be in ASCII order because `gen_keywordlist.pl` builds a perfect-hash lookup from it [30]. Parallel files exist for PL/pgSQL (`pl_reserved_kwlist.h`, `pl_unreserved_kwlist.h`), ECPG, and the C parser — same macro, different tables [32].

**The category taxonomy** (user-facing form) [33]:

1. **unreserved** — no special status outside its own context; usable as any identifier.
2. **non-reserved (cannot be function or type name)** — e.g. `BETWEEN`, `COALESCE`, `EXTRACT`, `POSITION`. "Most of these words represent built-in functions or data types with special syntax. The function or type is still available but it cannot be redefined by the user."
3. **reserved (can be function or type name)** — e.g. `AUTHORIZATION`, `CROSS`, `FULL`, `INNER`, `JOIN`, `LEFT`, `NATURAL`, `RIGHT`.
4. **reserved** — e.g. `SELECT`, `FROM`, `WHERE`.

And the crucial pragmatic escape hatch: **"Even reserved key words are not completely reserved in PostgreSQL, but can be used as column labels (for example, `SELECT 55 AS CHECK`, even though `CHECK` is a reserved key word)"** [33].

**Correction / addition to the brief.** Your four categories are right, but there is a **fifth, orthogonal axis** you did not mention and should: the `BARE_LABEL` / `AS_LABEL` field. It records whether the keyword can appear as a bare column label without `AS` [30], [31], and the user docs mark ~25 keywords as "requires AS" (`ARRAY`, `CHAR`, `EXCEPT`, `FETCH`, `FROM`, `GROUP`, `LIMIT`, `ON`, `ORDER`, `OVER`, …) [33]. This is a second dimension, not a fifth value on the first — a keyword has a *category* **and** a *label-status*.

**Correction (my own uncertainty).** Two automated reads of `kwlist.h` disagreed: one reported "approximately 900 `PG_KEYWORD` entries," the other reported the file at **538 lines / 535 loc** [31]. Since there is exactly one `PG_KEYWORD` per line and the header is ~18 lines, the file cannot contain 900 entries. **The defensible statement is ~500–520 keywords in current master; I could not obtain an exact verified count in this session and you should count it yourself before citing a number.** The 538-line figure is the one I can source [31].

**Lesson transferable to Niles.** This is your keyword-registry blueprint, and it is better than you framed it:

- **One table file, no logic.** `niles_kwlist.h`-equivalent: a `NILES_KEYWORD(spelling, token, category, label_status, origin, since_edition)` table. Extend Postgres's two axes with two more you need: `origin` ∈ {SQL_DERIVED, RUST_DERIVED, NILES_NOVEL} — which directly generates Appendix B's three keyword sub-sections — and `since_edition`, which makes §8's edition mechanism mechanically enforceable.
- **Generate everything from it.** Postgres generates the perfect-hash lookup, the bison token declarations, *and* documentation from the same table. You should generate: the lexer's keyword recognizer, the parser's `unreserved_keyword`-style nonterminal alternatives, the Appendix B normative reserved list, the `--explain` keyword reference, and the syntax-highlighter grammars. A hand-maintained Appendix B will drift from the implementation within one release; a generated one cannot.
- **Reserve as little as possible, and add the label escape hatch.** Postgres's `SELECT 55 AS CHECK` trick [33] exists because reserving a word breaks existing schemas. Niles will want words like `epoch`, `ledger`, `money`, `posted`, `valid`, `serve`, `evict` — every one of which is a plausible existing column name in a bank's schema. Default new keywords to *unreserved*, and force a written justification for each promotion to reserved.
- **Take the "dumb parser" split seriously and for the same reason.** Postgres separates raw-parse from analyze because *catalog lookups need a transaction* [28]. Nilestream has a strictly stronger version of that constraint: catalog lookups need an **epoch**. A Niles parse must not resolve names, because name resolution is only meaningful relative to a visibility frontier. Splitting `parse` (epoch-free, total, pure) from `analyze` (epoch-anchored, catalog-reading) is therefore not a stylistic import — it falls out of your own consistency model. This is a genuinely strong argument to make in the thesis, and [28] is the citation.

---

### 11. SQL keyword reservation classes and grammar size

**What I can verify.** SQL's own standard distinction: "According to the standard, reserved key words are the only real key words; they are never allowed as identifiers. Non-reserved key words only have a special meaning in particular contexts and can be used as identifiers in other contexts" [33]. PostgreSQL's docs then note that real implementations need finer classes: "In the PostgreSQL parser, life is a bit more complicated. There are several different classes of tokens ranging from those that can never be used as an identifier to those that have absolutely no special status in the parser" [33]. PostgreSQL's Appendix C cross-tabulates every keyword against SQL:2023 / SQL:2016 / SQL-92 with reserved/non-reserved markers [33].

SQL-92 was published November 1992 and "grew about five times compared to SQL-89," though the growth was mostly "more precise specifications of existing features," with new features accounting for only 1.5–2× [34]. Machine-readable BNF for SQL-92, SQL-99, SQL-2003 and SQL-2016 is maintained in Ron Savage's public grammar repository, in `.bnf`, hyperlinked `.bnf.html`, and `.ebnf` forms [35].

**Correction — I could not verify the numbers you asked for.** I found **no authoritative source** for a production count of the SQL-92 or SQL:2016 BNF, and none for a page count of SQL:2016. Do not cite a number for these unless you count the productions in [35] yourself and report your own method. Widely circulated figures for these ("SQL-92 has N productions", "SQL:2016 is N pages") are, as far as I could establish in this session, uncited folklore. **Flagging this is more useful than giving you a plausible-looking number.**

**Why SQL has so many reserved words — the honest causal story.** It is not that the standard's authors were careless. Three compounding causes, all evidenced above: (i) SQL's grammar is *keyword-delimited* rather than punctuation-delimited, so every new clause consumes vocabulary; (ii) the grammar is LALR (bison/yacc), and LALR(1) resolves far fewer context-dependent identifier/keyword ambiguities than an unbounded-lookahead or hand-written parser, so words get reserved to remove conflicts [36]; (iii) each standard revision adds features but essentially never removes reservations, and implementations must keep old reservations for compatibility.

**Lesson transferable to Niles.** Cause (ii) is actionable and is an argument for §14's approach: a hand-written recursive-descent parser with unbounded lookahead and backtracking can keep words unreserved that an LALR parser would be forced to reserve. If Niles wants a large novel vocabulary *and* a small reserved set, the parser technology choice is what buys it. Say this explicitly in Appendix B's design-principles section: **"reserved-word count is a function of parser technology, not of vocabulary size."**

---

### 12. Apache Calcite: templated parser, validator, `RelNode` algebra, Volcano planning

**The parser.** Generated by JavaCC from `Parser.jj`, which is itself **a FreeMarker template**: "Parser.jj is actually an Apache FreeMarker template that contains variables that can be substituted" [37]. Downstream projects supply their own `config.fmpp` and `parserImpls.ftl` to inject grammar without forking: "The parser in calcite-core instantiates the template with default values of the variables, typically empty, but you can override" [37]. `calcite-server` uses this to add DDL [37]. The template is **9,837 lines / 269 KB**, using `${parser.class}`, `${parser.package}`, `<#list …>` and `<@pp.changeOutputFile …>` directives [38]. Keyword categorization is consulted at parse time via `getMetadata().isNonReservedKeyword(token)` [38].

**The validator as its own phase.** `SqlValidator` "validates the parse tree of a SQL statement, and provides semantic information about the parse tree" [39]. Its structure is the part worth stealing: **two orthogonal abstractions**, `SqlValidatorScope` (what tables/columns are visible *at a point*) and `SqlValidatorNamespace` (what a *data source* is — `IdentifierNamespace`, `SelectNamespace`, `SetopNamespace`, …) [39]. The validator also derives types (`deriveType()`), expands `*` (`expandStar()`), and inserts implicit casts [39]. Scope visibility differs per clause: FROM sees only base tables; WHERE/GROUP BY see joined tables; SELECT/HAVING see more; **ORDER BY uniquely sees column aliases**, supporting ordinal and alias ordering [39].

**The algebra IR.** "Every query is represented as a tree of relational operators," and "planner rules transform expression trees using mathematical identities that preserve semantics" [40]. `RelBuilder` is "the simplest way to build a relational expression" — a fluent API [40]. Critically, the documentation makes the SqlNode/RelNode boundary a *two-entrance* design: "You can translate from SQL to relational algebra, or you can build the tree directly" [40]. The optimizer works **only** on `RelNode`, which is what lets Calcite support many SQL dialects behind one optimizer [40].

**The planner.** `VolcanoPlanner` "optimizes queries by transforming expressions selectively according to a dynamic programming algorithm" [41]. Calcite also ships `HepPlanner` for deterministic heuristic rule application.

**Correction to the brief.** You wrote "Volcano/Cascades cost-based optimization." The javadoc says only "dynamic programming algorithm" and does **not** cite Graefe [41]. Calcite's Volcano planner is Cascades-style in *character* (top-down, memo, rule-driven, branch-and-bound), and the Calcite paper describes a cost-based optimizer, but if you attribute Cascades specifically, cite Graefe's own papers rather than Calcite's javadoc, which does not make the claim.

**Lesson transferable to Niles — three, and the third is the most important.**

- **Templated grammar with an extension seam.** Given Niles's domain layer over a general relational core, a templated parser lets the banking vocabulary live in a *separate grammar fragment* injected into the core template. This mechanically enforces your "banking constructs are a domain layer on a fully general relational core" claim: if the banking rules are a separate `.ftl`-equivalent, the core parser demonstrably does not depend on them.
- **Scope and Namespace as separate abstractions.** Copy this verbatim. Niles needs a third axis Calcite lacks: **epoch/visibility**. Model it as `NilesScope` (name visibility) × `NilesNamespace` (data source) × `NilesFrontier` (which epoch a name resolves *as of*). Keeping these three orthogonal is what will make bitemporal name resolution tractable rather than a special case bolted into every clause. Also note the ORDER BY alias-visibility quirk [39] — the SQL clause visibility rules are irregular, and if Niles inherits SQL's pipelined clause order (as Appendix B plans), it inherits the irregularity and must specify it.
- **The algebra is the contract, and it must have two entrances.** Calcite's optimizer touching only `RelNode` [40] is exactly the discipline that makes your "three surfaces, one IR" claim real rather than aspirational. And `RelBuilder` — the ability to construct the IR *without going through the surface syntax* — is worth building early: it is what makes the IR independently testable, and it is what the self-hosted compiler in Appendix E will need.

**Reference paper.** Begoli, Camacho-Rodríguez, Hyde, Mior, Lemire, "Apache Calcite: A Foundational Framework for Optimized Query Processing Over Heterogeneous Data Sources," SIGMOD 2018, arXiv:1802.10233 [42].

---

### 13. DuckDB: forking the Postgres parser

**The design and the stated reason.** The SIGMOD 2019 demo paper: "The SQL parser is derived from Postgres' SQL parser that has been stripped down as much as possible" [43]. The rationale is explicitly *pragmatic*: it provides "a full-featured and stable parser." The parse tree is then immediately converted into DuckDB's own C++ structures "to limit Postgres data structure dependencies" [43] — i.e. the fork is quarantined at the boundary, not spread through the system.

The downstream pipeline: "The binder resolves all expressions referring to schema objects such as tables or views with their column names and types. The logical plan generator then transforms the parse tree into a tree of basic logical query operators" [43] — the same raw-parse / bind / logical-plan split as Postgres. The optimizer does dynamic-programming join ordering with a greedy fallback, arbitrary subquery flattening, CSE and constant folding [43]; execution is vectorized-interpreted with 1024-tuple vectors [43].

**The cost they later admitted.** In a 2024 post, Mühleisen and Raasveldt describe their own parser as "a fork of the Postgres YACC parser" and write that "the most advanced SQL systems of 2024 use parser technology from the 1960s" [36]. Two specific complaints:

- **No runtime extensibility.** "Parser construction is a compile-time activity" requiring grammar edits, recompilation, and resolving shift/reduce conflicts — impractical for extensions [36].
- **All-or-nothing errors.** YACC parsers accept or reject a whole query, "making error messages unhelpful" [36].

Their prototype PEG parser fixes both — "PEG parsers do not require a compilation step where the grammar is converted to… a finite state automaton," making it "feasible to re-create a parser at runtime," and PEG recovery rules give "more than a single error" with "meaningful error messages" — at a cost of "approximately 10× slower" parsing, which they judge negligible against analytical query durations [36].

**Lesson transferable to Niles.** Three:

- **Forking is a legitimate schedule decision when the fork is quarantined.** DuckDB got a complete, battle-tested SQL surface for near-zero design cost, and paid for it by converting to native structures immediately at the boundary [43]. If Nilestream needs a PostgreSQL-wire-compatible *legacy SQL* path (your MySQL/PostgreSQL wire-compatibility goal), forking a parser for that path *only*, behind an immediate conversion to Niles IR, is a defensible engineering choice — and DuckDB is the citation that makes it defensible rather than lazy.
- **But do not fork for Niles itself.** DuckDB's own 2024 retrospective enumerates exactly the two properties Niles most needs and a YACC fork cannot give: runtime grammar extensibility (for your UDF tier and domain library) and multi-error recovery (for the IDE experience in §17) [36].
- **The 10× parse-cost finding is directly usable.** [36] gives you a published data point that parser speed is not the binding constraint in a database system. If reviewers push back on a hand-written or PEG parser for Niles on performance grounds, this is the counter-evidence.

---

### 14. `sqlparser-rs`: hand-written recursive descent + Pratt, and the dialect trait

**The design.** "The core expression parser uses the Pratt Parser design… while the surrounding SQL statement parser is a traditional, hand-written recursive descent parser" [44]. Dialects are a trait; the contribution rule is explicit: "Any SQL feature that is dialect specific should be parsed by *both* the relevant `Dialect` as well as `GenericDialect`" [44].

**The scope boundary — this is the most transferable single decision.** "This crate provides only a syntax parser, and tries to avoid applying any SQL semantics." It deliberately accepts statements most databases reject (duplicate column names, for instance) because semantic validation "varies drastically between dialects" [44]. And a fidelity guarantee: "The original SQL text can be generated from the AST" (modulo comments, whitespace normalization, keyword capitalization) [44].

**Why hand-written over generated.** The maintainers give four reasons [44]:

1. "Code is simple to write"
2. "Performance is generally better"
3. "Debugging is much easier"
4. "It is far easier to extend and make dialect-specific extensions"

**Lessons on dialects.** The `GenericDialect` rule [44] is the interesting one: a permissive superset dialect that accepts everything, plus strict per-dialect parsers. This gives you a *tool* dialect (accept anything, for linting/lineage/migration analysis) and *engine* dialects (reject what the engine won't run) from one codebase.

**Lesson transferable to Niles.**

- **Pratt for expressions, recursive descent for statements** is the right default. SQL/Niles expression grammars are precedence-heavy and statement grammars are keyword-driven; Pratt handles the first with a precedence table (which can be *generated from the keyword registry* of §10), recursive descent handles the second readably. This is also, per §11, what lets you keep the reserved-word set small.
- **The "syntax only, no semantics" boundary is the same boundary Postgres draws for a different reason.** §10 gives you an epoch-based justification; §14 gives you a *dialect-independence* justification. Both point at the same architecture. Draw the line once, in the IR: `parse` produces a Niles CST/AST that is *total* on well-formed token streams and knows nothing about the catalog, epochs, money, or currencies.
- **Round-trip fidelity as a spec'd property.** `sqlparser-rs` guarantees SQL text regenerates from the AST [44]. Combine with rustc's pretty-print fixpoint test (§9, [25]) — the round-trip is not just a nice property, it is a testable invariant with an established harness pattern.
- **On dialects for Niles:** you have three surfaces (Rust-first, SQL-fallback, novel). Treat these as *dialects of one AST*, not three parsers. The `GenericDialect` pattern says: build one permissive parser that accepts the union, plus per-surface strictness gates that reject out-of-surface constructs after parsing. That preserves the "one IR" invariant and makes the surface-equivalence test of §9 mechanical.

---

### 15. ZetaSQL / GoogleSQL: the resolved AST as an engine contract

**Correction to the brief, first.** ZetaSQL has been renamed/rehosted as **GoogleSQL** (`google/googlesql`, "GoogleSQL (formerly ZetaSQL)") [45], [46]. Use the current name or note the rename.

**The design.** GoogleSQL "defines a SQL language (grammar, types, data model, semantics, and function library) and implements parsing and analysis for that language as a reusable component," "intended to be used by multiple engines, to provide consistent language and behavior (name resolution, type checking, implicit casting, etc.)" — deployed across BigQuery, Spanner, F1, BigTable, Dremel and Procella [45]. The analyzer emits a **Resolved AST**: fully typed, fully name-resolved, with node families `ResolvedArgument`, `ResolvedExpr`, `ResolvedScan`, `ResolvedStatement` [46]. The classes are **generated**: "The generated classes are specified in `gen_resolved_ast.py`" with "bazel genrules" producing both C++ and Java from one specification [46]. Feature availability is gated by `LanguageOptions` flags (`FEATURE_MULTILEVEL_AGGREGATION`, `FEATURE_AGGREGATE_FILTERING`, …), so engines declare which constructs they support [46]. Behaviour is validated by a shared compliance suite: "GoogleSQL's compliance test suite can be used to validate query engine implementations are correct and consistent" [45].

**The mechanism the brief is reaching for — and it is better than "versioned/stable."** I could **not** verify an explicit versioning-and-stability promise for the Resolved AST. What I *did* verify is a sharper mechanism that solves the same problem: **accessed-field tracking.** From `resolved_node.h` [47]:

> If an engine reads a field containing a value it doesn't understand, it should raise an unimplemented error. But if an engine fails to read a **non-ignorable** field entirely, the query could be interpreted incorrectly with no error raised.

Three methods address it [47]:

- `CheckFieldsAccessed()` — verifies "no non-ignorable fields are unaccessed" and errors if features are not being interpreted;
- `ClearFieldsAccessed()` — resets access markers across the node tree;
- `MarkFieldsAccessed()` — marks all fields accessed when a node "has no semantic effect on the query" (e.g. an unused WITH clause).

Fields are annotated ignorable / non-ignorable in the generator spec. So: an engine that upgrades to a GoogleSQL version with a new semantic field and silently ignores it gets a **runtime error**, not a silently wrong answer.

**Lesson transferable to Niles — and this is, in my judgement, the single most valuable import in the entire brief for a *financial* system.**

Nilestream will have a typed IR consumed by multiple back-ends (the interpreter, the DBSP circuit builder, the WASM UDF tier, the wire-compat shims, and eventually third-party engines). The catastrophic failure mode is not "back-end crashes on an unknown node"; it is "back-end silently ignores the `settlement_currency` field and produces a plausible, wrong, *auditable-looking* number." `CheckFieldsAccessed` converts that class of failure from silent to loud, mechanically, without proofs.

Concretely for Niles:
- Generate the IR node classes from one specification (as GoogleSQL does [46]) — this also gives Appendix D's IR reference for free and keeps it in sync.
- Annotate every IR field `ignorable` or `non_ignorable`, with **money-, currency-, epoch-, retention- and confidentiality-bearing fields non-ignorable by default**.
- Make `check_fields_accessed()` a *required* post-lowering assertion in every back-end, enforced in the test harness.
- Use `LanguageOptions`-style feature flags [46] so a back-end declares its supported subset and unsupported constructs fail at analysis time, not at execution time.

This is a citable, deployed, industrial mechanism [47] that directly serves your auditability and conservation-of-money obligations, and it costs far less than proving each back-end correct.

---

### 16. SQL → incremental dataflow IRs: Materialize, Feldera, RisingWave, Noria

#### 16.1 Materialize — HIR / MIR / LIR

**Your presumption is correct, and confirmed by a primary source.** Michael Greenberg (Materialize, 30 Jan 2025):

> "Materialize compiles SQL through a series of intermediate languages: a high-level intermediate language (HIR), a mid-level intermediate language (MIR), and a low-level intermediate language (LIR)." [48]

And the pipeline: "A SQL query is translated to an HIR query," "which is then translated into one or more MIR queries," and "the compiler then lowers MIR into LIR, our final intermediate representation" [48]. Where the work happens: **"Our optimizer does the bulk of its decision making in MIR: planning joins, removing redundancies, and identifying patterns"** [48].

The IR levels are user-visible through `EXPLAIN`, which supports `RAW PLAN` ("closest to the original SQL"), `DECORRELATED PLAN` ("the decorrelated but not-yet-optimized plan"), `LOCALLY OPTIMIZED PLAN` ("before view inlining and access path selection"), `OPTIMIZED PLAN`, and `PHYSICAL PLAN` ("corresponds to the operators shown in `mz_introspection.mz_lir_mapping`") [49]. The overall shape is "SQL ⇒ raw plan ⇒ decorrelated plan ⇒ optimized plan ⇒ physical plan ⇒ dataflow" [49].

**Caveat on the mapping.** `RAW PLAN` = `HirRelationExpr` and `PHYSICAL PLAN` = `LirRelationExpr` are solid (the latter is confirmed by the `mz_lir_mapping` reference [49]). The exact IR level of `DECORRELATED PLAN` and `LOCALLY OPTIMIZED PLAN` was reported inconsistently by my sources; **decorrelation is the HIR→MIR lowering**, so `DECORRELATED PLAN` should be MIR, but I flag this rather than assert it. Verify against the `mz-expr`/`mz-sql` crate docs before citing precisely.

**The architecture around it.** Materialize splits into storage (partial time-varying collections via the Persist library), adapter (PostgreSQL protocol, SQL parsing, catalog, **timestamp selection**), and compute (Timely/Differential Dataflow), across `environmentd` (control plane) and `clusterd` (data plane) [50]. "Compute then transforms the IR according to several optimization passes, and finally compiles it into a Differential Dataflow program" [50].

**Lesson transferable to Niles.** Materialize is the closest existing system to Nilestream's read side, and it independently arrived at rustc's three-level answer. Two specific imports:

- **Put the optimizer's decisions at exactly one level.** Materialize is explicit that MIR is where join planning, redundancy removal, and pattern identification happen [48]. Do the same: Nilestream's materialization/eviction decisions — the thing your phase diagram characterizes — should live at exactly one IR level, and it should be the one where operators are relational and epochs are explicit but physical dataflow layout is not yet fixed.
- **Make every IR level `EXPLAIN`-able.** [49] is the model. For a thesis whose deliverable is a *characterization*, having `EXPLAIN RAW / LOWERED / OPTIMIZED / PHYSICAL PLAN FOR <niles query>` is not a convenience feature — it is your primary instrument for showing *where* on the phase diagram a query falls and *why*. Note also that Materialize exposes the LIR↔runtime-operator mapping as a queryable system table (`mz_introspection.mz_lir_mapping`) [49]; the equivalent for Nilestream is an introspection view mapping REVs to their materialized/evicted extents per epoch.
- **Note the adapter's job.** "Timestamp selection" sits in the adapter [50] — i.e. Materialize also separates *which version of the world you read* from *what the query computes*. That is your epoch/visibility-frontier separation, in a shipping system.

#### 16.2 Feldera / DBSP — the circuit as the IR

**The theory.** DBSP models streams as infinite sequences over an **abelian group** (so that both insertion and *deletion* are expressible), and relations as **Z-sets** — "a table where each row has an associated weight," with negative weights for deletions [51]. The central object is the incrementalization operator:

$$Q^\Delta := \mathcal{D} \circ \uparrow\! Q \circ \mathcal{I}$$

where $\mathcal{I}$ integrates and $\mathcal{D}$ differentiates; "the incremental version of a query is a stateful streaming operator which computes directly on changes and produces changes" [51]. The algebraic laws that make it compositional: the **chain rule** $(Q_1 \circ Q_2)^\Delta = Q_1^\Delta \circ Q_2^\Delta$; linear operators are their own incremental versions (Thm 3.3); joins are bilinear, $\Delta(a \times b) = \Delta a \times \Delta b + a \times \Delta b + \Delta a \times b$ [51].

**The compiler.** Algorithm 4.6 mechanically converts a relational query to an incremental circuit in five steps — build the circuit, eliminate redundant `distinct`s algebraically, lift to streams, wrap with $\mathcal{I}$/$\mathcal{D}$, then apply the chain rule recursively. The process "is deterministic and its running time is proportional to the number of operators in the query" [51]. Complexity: $O(|\Delta DB[t]|)$ for most operators. It extends to recursive queries (Datalog, transitive closure) and to windows/streaming constructs. The implementation "passe[s] all 7 million SQL Logic Tests" [51].

**The modularity claim, which is the one to engage with.** "New operators automatically benefit from incrementalization theory once expressed as DBSP circuits" [51].

**Citations.** Budiu, Chajed, McSherry, Ryzhyk, Tannen, PVLDB 16(7), 2023 [51]; SIGMOD Record Research Highlight, 2024 [52]; extended VLDB Journal version, 2025 [53]; a mathematical specification [54]; and — notable for your proof ladder — a **Lean formalization** of the theory by Tej Chajed [55].

**Lesson transferable to Niles.** Three, and one is a warning:

- **DBSP gives you your baseline compilation strategy for free, and it is *deterministic*.** Algorithm 4.6 is mechanical and linear-time [51]. Your Contribution 1 (versioned partial-state algebra) should be positioned as *what Algorithm 4.6 does not do*: DBSP incrementalizes assuming state is fully retained. The step from "$Q^\Delta$ over full state" to "$Q^\Delta$ over *partial* state with eviction and upquery reconstruction" is precisely your gap, and it is cleanly statable in DBSP's own vocabulary.
- **The Lean formalization [55] is a template for your verification-status section.** You plan a "verification status of the proof ladder." Chajed's Lean development is the existing, citable precedent for mechanizing exactly this class of stream-algebra result, and it sets the bar (and gives you a possible foundation to build on rather than reproduce).
- **The warning.** The modularity claim [51] means: any operator you add to Niles must be expressible as a DBSP circuit or it does not get incrementalization. Your banking domain library (FX conversion, atomic cross-currency conservation, bitemporal `balance-as-of`) must therefore be shown to be DBSP-expressible — or explicitly carved out as a non-incremental escape hatch with a stated cost. Do this early; it constrains the language design.

**Note.** Feldera's SQL front-end builds on Apache Calcite; the `sql-to-dbsp-compiler` component is in the Feldera repository [56], but its README was not fetchable in this session (robots.txt), so I have not verified the Calcite integration details from a primary source. Verify before citing specifics.

#### 16.3 RisingWave

**The design.** SQL MV definitions become "a logical plan which consists of logical operators encoding the dataflow" [57]. Then: "the stream fragmenter at the meta service breaks the generated logical stream plan into stream fragments, and duplicates such fragments when necessary" [57]; fragments are scheduled to compute nodes which instantiate **actors**. Execution is actor-based rather than pipeline-parallel: "each actor reacts to its own input message, including both data update and control signal" [57].

The correctness argument is compositional: "we build a set of executors where each executor corresponds to a relational operator (including base table)," so per-operator correctness yields end-to-end MV correctness by recursive change propagation [57]. And a unification worth noting for a system with your durability requirements: RisingWave treats "every object in our internal storage as both a logical table and an internal state" [57].

**Lesson transferable to Niles.** Two:

- **Fragmentation is a distinct IR-lowering step**, separate from logical optimization and separate from physical scheduling [57]. Nilestream's distributed phase should name this step explicitly rather than folding it into planning — it is where sharding, cross-shard commit boundaries, and replica placement are decided, and your cross-shard commit protocol will need a plan-level object to attach to.
- **"State is a table" is directly aligned with your F2 hypothesis.** RisingWave's storage unification [57] is a shipping instance of stream-relation duality: operator state and user-visible relations are the same kind of object. That is an engineering corroboration of F2 you can cite, and it also suggests the design where a REV's materialized extent is *itself* queryable as a relation — which is what makes your introspection and audit story cheap.

#### 16.4 Noria

**The design.** Gjengset et al., OSDI '18: Noria accepts "a relational schema and a set of parameterized queries" and compiles them "into a data-flow program that pre-computes results for reads and incrementally applies writes" [58]. The novel contribution is a streaming model supporting "eviction and reconstruction of data-flow state on demand," and "partial statefulness helps Noria limit its in-memory state without prior data-flow systems' restriction to windowed state," while also enabling online schema and query changes [58].

**Upqueries.** From Gjengset's MIT PhD thesis (Feb 2021): "Partial state lets entries in materialized views be marked as missing, and introduces upqueries to compute such missing state on-demand," and "upqueries flow 'up' this dataflow, in the opposite direction of the data, and trigger the retransmission of past state in the case of a cache miss" [59].

**Correction to the brief.** The brief asks "how each turns SQL into an incremental dataflow IR" for Noria. **The thesis does not name or formalize an intermediate representation.** It describes compiling "all the application queries into a joint dataflow program" forming "a directed acyclic graph of relational operators such as aggregations, joins, and filters," and notes "Noria focuses entirely on relational operators," unlike graph/iterative dataflow systems [59]. So: Noria's SQL→dataflow lowering is *implemented*, not *specified as an IR*. (The commercial descendant, ReadySet, does carry a `readyset-mir` crate [60] — evidence that an IR emerged in practice — but I could not retrieve documentation describing it, so treat that as a code-structure observation, not a documented design.)

**Lesson transferable to Niles.** This gap is *your opportunity*, and it is worth stating in the thesis exactly this way:

> Noria established partial statefulness and upqueries as a working mechanism [58], [59], and DBSP established a formal algebra for incrementalization over fully-retained state [51]. Neither provides a formal algebra of *partial* state. The REV, the versioned partial-state algebra, and the epoch-anchored reconstruction theorem occupy precisely that gap.

Two further points: (i) Noria's SQL→dataflow lowering being unspecified is *why* it cannot state a reconstruction theorem — you cannot prove eviction-and-reconstruction preserves anything if the object being evicted has no formal definition; naming the REV is therefore load-bearing, not cosmetic. (ii) Noria's "online schema and query changes" [58] is a capability you will need to either match or explicitly disclaim, since a ledger system's derived views will change more often than its base.

---

### 17. Error-recovery parsing for IDE-grade experience

**The tree design.** rust-analyzer uses **Rowan**, a red-green tree library after Roslyn, in three layers: `GreenNode` (immutable, purely functional, holds the data), `SyntaxNode`/red node (adds parent pointers and identity), and typed AST wrappers over untyped syntax nodes [61].

Three stated invariants [61]:
1. **Lossless / full-fidelity**: "Syntax trees are lossless, or full fidelity. All comments and whitespace get preserved."
2. **Semantic-less**: "Syntax trees are semantic-less. They describe *strictly* the structure of a sequence of characters, they don't have hygiene, name resolution or type information attached."
3. **Resilient**: "even if the input is invalid, parser tries to see as much syntax tree fragments in the input as it can."

Errors are wrapped in error nodes treated like normal nodes, and crucially — **"Parser errors are not a part of syntax tree"** — they are reported on the side, so the tree stays stable across refactors [61]. And the architectural separation: "Keep the parser and the syntax tree isolated from each other, such that they can vary independently," achieved with abstract `TokenSource`/`TreeSink` traits so neither crate depends on the other [61].

**The recovery techniques.** From Kladov's resilient-LL tutorial (21 May 2023) [62]:

1. **Error nodes** — wrap unexpected tokens in an error node rather than aborting; parsing continues and recovers downstream.
2. **Recovery sets / synchronizing tokens** — when a parse loop fails, check the next token against a recovery set that means "stop here, break to the parent." For function parameters the set includes `->`, `{`, `fn`.
3. **FIRST-set guards** — for expressions, check against `EXPR_FIRST` so recovery does not swallow keywords like `let`.
4. **Homogeneous (dynamically-shaped) trees** — "This structure does not enforce any constraints on the shape of the syntax tree at all, and so it naturally accommodates errors anywhere."
5. **LL over LR for incomplete input** — "code is written top-down and left-to-right, LL seems to have an advantage for typical patterns of incomplete code," because LL naturally recognizes valid *prefixes*.
6. **The progress invariant** — every parse-loop iteration must consume at least one token, preventing infinite loops during recovery.

**Corroboration from the SQL side.** DuckDB independently identified the same problem: YACC parsers are "all-or-nothing," while PEG with annotated recovery rules gives "more than a single error" and "meaningful error messages" [36]. Two very different communities converging on the same diagnosis is strong evidence.

**Lesson transferable to Niles.** This is the item that determines whether Niles is *adoptable*, and it is a design decision that cannot be retrofitted — the tree representation and the parser's recovery strategy have to be right from the first commit.

- **Adopt a lossless CST with typed AST views over it.** Niles needs both: the CST for the LSP, formatter, and migration tooling (which must preserve comments and layout), and the typed AST for the compiler. Rowan's three-layer design [61] gives both from one parse.
- **Keep errors out of the tree.** [61]'s point — errors reported alongside, not embedded — is what makes incremental reparsing tractable.
- **Point 4 is the crucial one and conflicts with a naive typed-AST-first design.** A homogeneous tree "naturally accommodates errors anywhere" [62]. If you define Niles's AST as a strongly-typed Rust enum tree from the outset, you will have no place to put a malformed `serve` contract or a half-typed money literal, and you will end up with an unrecoverable parser. Parse into a homogeneous CST; project typed views out of it.
- **Design recovery sets from the keyword registry.** §10's registry can *generate* the synchronizing-token sets: statement-initial keywords are natural recovery points. This is a concrete payoff from making the registry a data table rather than scattered constants.
- **Point 5 argues for LL/recursive-descent over LALR**, which agrees with §14 (`sqlparser-rs`) and §13 (DuckDB's retrospective) and §11 (fewer forced reservations). Four independent lines of evidence converge on hand-written recursive descent for Niles.

---

## 18. Consolidated corrections to the brief

Ranked by how much they change the thesis text:

1. **`cargo fuzz` is not how rustc is fuzzed.** The dev guide names `fuzz-rustc`, `icemaker`, and `tree-splicer` [26]. `cargo-fuzz` fuzzes Rust libraries, not the compiler. Fix the sentence or you will be corrected in review.
2. **Chalk did not ship.** What shipped is `rustc_next_trait_solver` / `-Znext-solver` from the Trait System Refactor Initiative — influenced by chalk, not chalk [18], [19]. Describe chalk as an influential reformulation *attempt*.
3. **The bootstrap is not "three stages" in the sense implied, and the determinism check is not `x.py build --stage 2` compared against itself.** stage0(beta) → stage1 → stage2 is the chain; stage3 is the optional same-result verification, and the gate is stage2 vs stage3 [20]. Also, the "snapshot compiler" is history — modern stage0 is a released beta [20], [21].
4. **Noria has no documented IR.** The thesis compiles SQL directly to a dataflow DAG and never names an intermediate representation [59]. `readyset-mir` exists in the descendant codebase [60] but is undocumented. Rephrase item 16's premise for Noria — and note this gap is an asset to your positioning.
5. **ZetaSQL is now GoogleSQL** [45], and I could **not verify** an explicit "versioned/stable" promise for the Resolved AST. The real, verifiable mechanism serving that goal is `CheckFieldsAccessed` / ignorable-vs-non-ignorable field annotations [47] plus `LanguageOptions` feature gating [46]. This is a *better* fact for your thesis than the one you assumed.
6. **PostgreSQL's keyword taxonomy has a second, orthogonal axis** you did not mention: `BARE_LABEL` vs `AS_LABEL` [30], [31], [33]. Four categories × two label statuses.
7. **I could not verify any production count for SQL-92 or SQL:2016 BNF, nor a page count for SQL:2016.** Do not cite a number you have not counted. Machine-readable BNF is available at [35] if you want to count it and report your method.
8. **`kwlist.h` keyword count: sources disagreed** (one automated read said ~900; the file is 538 lines with one keyword per line [31], implying ~500–520). Count it before citing.
9. **Calcite's `VolcanoPlanner` javadoc does not claim Cascades or cite Graefe** — it says "dynamic programming algorithm" [41]. Attribute Cascades to Graefe directly if you make that claim.
10. **Materialize's HIR/MIR/LIR: you were right** [48], [49] — confirmed by primary sources. But the exact IR level of `DECORRELATED PLAN` / `LOCALLY OPTIMIZED PLAN` was reported inconsistently across my sources; verify before asserting the full mapping.
11. **DuckDB's parser choice has a published retrospective critique from its own authors** [36] — worth citing alongside the original decision [43], because it turns "DuckDB forked Postgres" from an endorsement into a nuanced cost/benefit you can reason about.
12. **Unverified in this session:** Feldera's `sql-to-dbsp-compiler` README (robots-blocked) [56]; the "Introducing MIR" blog post (redirect only — but RFC 1211 [8] is a stronger source anyway).

---

## 19. Synthesis: the five design decisions this research most strongly recommends

1. **Split parsing from analysis on an epoch boundary.** Postgres splits on a *transaction* boundary because catalog lookups need a transaction [28]. Nilestream's version is stronger and more principled: name resolution is only meaningful relative to a visibility frontier, so `parse` must be epoch-free and total, and `analyze` epoch-anchored. This is a thesis argument, not just an implementation note.
2. **Make the keyword registry a generated data table with four axes** — spelling, token, reservation category, label status, plus `origin` and `since_edition` [30], [31], [33] — and generate the lexer, the grammar's keyword nonterminals, Appendix B, and the recovery sets from it. Ship an edition mechanism with the RFC 2052 guarantees (forward compatibility of warning-free code; one core IR; mandatory cross-edition interop; `MachineApplicable` migration) before you have users [22], [23].
3. **Four IR levels, split by what becomes checkable**, per RFC 1211's downside list [8]: surface AST/CST (lossless, resilient), a desugared level, a fully-typed level where the consistency-effect calculus and Contribution-4 soundness are stated, and a flat epoch-explicit operator DAG where the partial-state algebra and reconstruction theorem live and where materialization decisions are made. Make every level `EXPLAIN`-able, as Materialize does [48], [49].
4. **Adopt GoogleSQL's accessed-field discipline for the IR** [47]. For a system whose central claim is that money cannot be created or destroyed, a mechanism that turns "back-end silently ignored a semantic field" into a hard error is worth more per engineering hour than almost anything else on this list — and it composes with, rather than substitutes for, your proofs.
5. **Hand-written resilient recursive descent, Pratt expressions, lossless CST, errors outside the tree** [44], [61], [62] — supported independently by rustc, rust-analyzer, sqlparser-rs, and DuckDB's own retrospective [36]. This is also what keeps the reserved-word set small despite a large novel vocabulary (§11), which matters enormously for a language meant to be adopted against existing bank schemas.

---

## References

[1] Rust Project, "Queries: demand-driven compilation," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/query.html

[2] Rust Project, "Incremental compilation," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html

[3] Rust Project, "Incremental compilation in detail," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html

[4] D. Alden, "Rust's incremental compiler architecture," *LWN.net*, Dec. 3, 2024. [Online]. Available: https://lwn.net/Articles/997784/

[5] Rust Project, "Overview of the compiler," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/overview.html

[6] Rust Project, "The THIR," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/thir.html

[7] Rust Project, "The MIR (Mid-level IR)," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/mir/index.html

[8] N. Matsakis, "RFC 1211: Mid-level IR (MIR)," *The Rust RFC Book*, 2015. [Online]. Available: https://rust-lang.github.io/rfcs/1211-mir.html

[9] Rust Project, "Memory management in rustc," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/memory.html

[10] Rust Project, "The HIR," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/hir.html

[11] Rust Project, "Errors and lints," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/diagnostics.html

[12] Rust Project, "Rust Compiler Error Index," 2026. [Online]. Available: https://doc.rust-lang.org/error_codes/error-index.html

[13] Rust Project, "Crate `rustc_lexer`," *rustc API documentation*, 2026. [Online]. Available: https://doc.rust-lang.org/stable/nightly-rustc/rustc_lexer/index.html

[14] Rust Project, "Lexing and parsing," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/the-parser.html

[15] rust-analyzer Project, "Crate `ra_ap_rustc_lexer`," *docs.rs*, 2026. [Online]. Available: https://docs.rs/ra-ap-rustc_lexer/latest/ra_ap_rustc_lexer/

[16] N. Nethercote, "Quirks of Rust's token representation," Oct. 5, 2022. [Online]. Available: https://nnethercote.github.io/2022/10/05/quirks-of-rusts-token-representation.html

[17] Rust Project, "Type inference," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/type-inference.html

[18] Rust Project, "Chalk-based trait solving," *Rust Compiler Development Guide*, May 2022. [Online]. Available: https://rustc-dev-guide.rust-lang.org/traits/chalk.html

[19] lcnr and the Rustc Trait System Refactor Initiative, "Enabling the next-generation trait solver on nightly," *Rust Blog*, Aug. 21, 2026. [Online]. Available: https://blog.rust-lang.org/2026/08/21/enabling-next-solver-on-nightly/

[20] Rust Project, "What bootstrapping does," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/building/bootstrapping/what-bootstrapping-does.html

[21] rust-dev mailing list, "How is Rust bootstrapped?," Mozilla, Jun. 2014. [Online]. Available: https://mail.mozilla.org/pipermail/rust-dev/2014-June/010222.html

[22] Rust Project, "What are editions?," *The Rust Edition Guide*, 2026. [Online]. Available: https://doc.rust-lang.org/edition-guide/editions/index.html

[23] A. Turon and N. Matsakis, "RFC 2052: Epochs (editions)," *The Rust RFC Book*, 2017. [Online]. Available: https://rust-lang.github.io/rfcs/2052-epochs.html

[24] Rust Project, "compiletest," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/tests/compiletest.html

[25] Rust Project, "compiletest `runtest/pretty.rs` source," *rustc API documentation*, 2026. [Online]. Available: https://doc.rust-lang.org/stable/nightly-rustc/src/compiletest/runtest/pretty.rs.html

[26] Rust Project, "Fuzzing," *Rust Compiler Development Guide*, 2026. [Online]. Available: https://rustc-dev-guide.rust-lang.org/fuzzing.html

[27] PostgreSQL Global Development Group, "The path of a query," *PostgreSQL 18 Documentation*, 2026. [Online]. Available: https://www.postgresql.org/docs/current/query-path.html

[28] PostgreSQL Global Development Group, "The parser stage," *PostgreSQL 18 Documentation*, 2026. [Online]. Available: https://www.postgresql.org/docs/current/parser-stage.html

[29] PostgreSQL Global Development Group, "`src/backend/parser/gram.y`," PostgreSQL source repository (master), 2026. [Online]. Available: https://github.com/postgres/postgres/blob/master/src/backend/parser/gram.y

[30] PostgreSQL Global Development Group, "`kwlist.h` source listing," *PostgreSQL Source Code (doxygen, master)*, 2026. [Online]. Available: https://doxygen.postgresql.org/kwlist_8h_source.html

[31] PostgreSQL Global Development Group, "`src/include/parser/kwlist.h`," PostgreSQL source repository (master), 2026. [Online]. Available: https://github.com/postgres/postgres/blob/master/src/include/parser/kwlist.h

[32] PostgreSQL Global Development Group, "`pl_reserved_kwlist.h` / `pl_unreserved_kwlist.h` / `ecpg_kwlist.h` source listings," *PostgreSQL Source Code (doxygen)*, 2026. [Online]. Available: https://doxygen.postgresql.org/pl__reserved__kwlist_8h_source.html

[33] PostgreSQL Global Development Group, "Appendix C. SQL Key Words," *PostgreSQL 18 Documentation*, 2026. [Online]. Available: https://www.postgresql.org/docs/current/sql-keywords-appendix.html

[34] Wikipedia contributors, "SQL-92," *Wikipedia*, 2026. [Online]. Available: https://en.wikipedia.org/wiki/SQL-92

[35] R. Savage, "SQL: BNF grammars for SQL standards (SQL-92, SQL-99, SQL-2003, SQL-2016)," GitHub repository. [Online]. Available: https://github.com/ronsavage/SQL

[36] H. Mühleisen and M. Raasveldt, "Runtime-Extensible SQL Parsers Using PEG," *DuckDB Blog*, Nov. 22, 2024. [Online]. Available: https://duckdb.org/2024/11/22/runtime-extensible-parsers

[37] Apache Software Foundation, "Adapters — Extending the parser," *Apache Calcite Documentation*, 2026. [Online]. Available: https://calcite.apache.org/docs/adapter.html

[38] Apache Software Foundation, "`core/src/main/codegen/templates/Parser.jj`," Apache Calcite source repository (main), 2026. [Online]. Available: https://github.com/apache/calcite/blob/main/core/src/main/codegen/templates/Parser.jj

[39] Apache Software Foundation, "Interface `SqlValidator`," *Apache Calcite API Documentation*, 2026. [Online]. Available: https://calcite.apache.org/javadocAggregate/org/apache/calcite/sql/validate/SqlValidator.html

[40] Apache Software Foundation, "Algebra," *Apache Calcite Documentation*, 2026. [Online]. Available: https://calcite.apache.org/docs/algebra.html

[41] Apache Software Foundation, "Class `VolcanoPlanner`," *Apache Calcite API Documentation*, 2026. [Online]. Available: https://calcite.apache.org/javadocAggregate/org/apache/calcite/plan/volcano/VolcanoPlanner.html

[42] E. Begoli, J. Camacho-Rodríguez, J. Hyde, M. J. Mior, and D. Lemire, "Apache Calcite: A Foundational Framework for Optimized Query Processing Over Heterogeneous Data Sources," in *Proc. 2018 ACM SIGMOD Int. Conf. Management of Data*, 2018, pp. 221–230. arXiv:1802.10233. [Online]. Available: https://arxiv.org/abs/1802.10233

[43] M. Raasveldt and H. Mühleisen, "DuckDB: an Embeddable Analytical Database," in *Proc. 2019 ACM SIGMOD Int. Conf. Management of Data (Demo)*, Amsterdam, 2019. [Online]. Available: https://duckdb.org/pdf/SIGMOD2019-demo-duckdb.pdf

[44] Apache Software Foundation / DataFusion Project, "datafusion-sqlparser-rs: Extensible SQL Lexer and Parser for Rust," GitHub repository, 2026. [Online]. Available: https://github.com/apache/datafusion-sqlparser-rs

[45] Google, "GoogleSQL (formerly ZetaSQL) — Analyzer Framework for SQL," GitHub repository, 2026. [Online]. Available: https://github.com/google/googlesql

[46] Google, "ResolvedAST," *GoogleSQL/ZetaSQL documentation*, 2026. [Online]. Available: https://github.com/google/zetasql/blob/master/docs/resolved_ast.md

[47] Google, "`resolved_ast/resolved_node.h`," GoogleSQL source repository (master), 2026. [Online]. Available: https://github.com/google/googlesql/blob/master/googlesql/resolved_ast/resolved_node.h

[48] M. Greenberg, "Source Mapping and Introspection: Debugging Materialize with Materialize," *Materialize Blog*, Jan. 30, 2025. [Online]. Available: https://materialize.com/blog/debugging-query-performance/

[49] Materialize Inc., "EXPLAIN PLAN," *Materialize Documentation*, 2026. [Online]. Available: https://materialize.com/docs/sql/explain-plan/

[50] B. Vincent, "The Software Architecture of Materialize," *Materialize Blog*, Feb. 23, 2023. [Online]. Available: https://materialize.com/blog/materialize-architecture/

[51] M. Budiu, T. Chajed, F. McSherry, L. Ryzhyk, and V. Tannen, "DBSP: Automatic Incremental View Maintenance for Rich Query Languages," *Proc. VLDB Endowment*, vol. 16, no. 7, pp. 1601–1614, Aug. 2023. [Online]. Available: https://docs.feldera.com/vldb23.pdf

[52] M. Budiu, T. Chajed, F. McSherry, L. Ryzhyk, and V. Tannen, "DBSP: Incremental Computation on Streams and Its Applications to Databases," *ACM SIGMOD Record*, vol. 53, Mar. 2024. [Online]. Available: https://dl.acm.org/doi/10.1145/3665252.3665271

[53] M. Budiu et al., "DBSP: automatic incremental view maintenance for rich query languages," *The VLDB Journal*, vol. 34, no. 39, Apr. 2025. [Online]. Available: https://link.springer.com/article/10.1007/s00778-025-00922-y

[54] M. Budiu, "DBSP mathematical specification." [Online]. Available: https://mihaibudiu.github.io/work/dbsp-spec.pdf

[55] T. Chajed, "database-stream-processing-theory: a Lean formalization of DBSP," GitHub repository, 2022. [Online]. Available: https://github.com/tchajed/database-stream-processing-theory

[56] Feldera Inc., "`sql-to-dbsp-compiler`," Feldera source repository (main), 2026. [Online]. Available: https://github.com/feldera/feldera/tree/main/sql-to-dbsp-compiler

[57] RisingWave Labs, "Streaming Engine," *RisingWave Developer Guide*, 2026. [Online]. Available: https://risingwavelabs.github.io/risingwave/design/streaming-overview.html

[58] J. Gjengset, M. Schwarzkopf, J. Behrens, L. T. Araújo, M. Ek, E. Kohler, M. F. Kaashoek, and R. Morris, "Noria: dynamic, partially-stateful data-flow for high-performance web applications," in *Proc. 13th USENIX Symp. Operating Systems Design and Implementation (OSDI '18)*, 2018. [Online]. Available: https://www.usenix.org/conference/osdi18/presentation/gjengset

[59] J. F. R. Gjengset, "Partial State in Dataflow-Based Materialized Views," Ph.D. dissertation, Massachusetts Institute of Technology, Feb. 2021. [Online]. Available: https://jon.thesquareplanet.com/papers/phd-thesis.pdf

[60] ReadySet Technology, "readyset," GitHub repository, 2026. [Online]. Available: https://github.com/readysettech/readyset

[61] rust-analyzer Project, "Syntax," *rust-analyzer Book — Contributing*, 2026. [Online]. Available: https://rust-analyzer.github.io/book/contributing/syntax.html

[62] A. Kladov (matklad), "Resilient LL Parsing Tutorial," May 21, 2023. [Online]. Available: https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html

---

**Sources:**
[Rust Compiler Development Guide — Queries](https://rustc-dev-guide.rust-lang.org/query.html) · [Incremental compilation in detail](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html) · [LWN: Rust's incremental compiler architecture](https://lwn.net/Articles/997784/) · [rustc overview](https://rustc-dev-guide.rust-lang.org/overview.html) · [THIR](https://rustc-dev-guide.rust-lang.org/thir.html) · [MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html) · [RFC 1211](https://rust-lang.github.io/rfcs/1211-mir.html) · [rustc memory management](https://rustc-dev-guide.rust-lang.org/memory.html) · [HIR](https://rustc-dev-guide.rust-lang.org/hir.html) · [Diagnostics](https://rustc-dev-guide.rust-lang.org/diagnostics.html) · [Error index](https://doc.rust-lang.org/error_codes/error-index.html) · [rustc_lexer](https://doc.rust-lang.org/stable/nightly-rustc/rustc_lexer/index.html) · [The parser](https://rustc-dev-guide.rust-lang.org/the-parser.html) · [ra_ap_rustc_lexer](https://docs.rs/ra-ap-rustc_lexer/latest/ra_ap_rustc_lexer/) · [Nethercote on token representation](https://nnethercote.github.io/2022/10/05/quirks-of-rusts-token-representation.html) · [Type inference](https://rustc-dev-guide.rust-lang.org/type-inference.html) · [Chalk](https://rustc-dev-guide.rust-lang.org/traits/chalk.html) · [Next-gen trait solver on nightly](https://blog.rust-lang.org/2026/08/21/enabling-next-solver-on-nightly/) · [What bootstrapping does](https://rustc-dev-guide.rust-lang.org/building/bootstrapping/what-bootstrapping-does.html) · [rust-dev: how is Rust bootstrapped](https://mail.mozilla.org/pipermail/rust-dev/2014-June/010222.html) · [Edition guide](https://doc.rust-lang.org/edition-guide/editions/index.html) · [RFC 2052](https://rust-lang.github.io/rfcs/2052-epochs.html) · [compiletest](https://rustc-dev-guide.rust-lang.org/tests/compiletest.html) · [compiletest pretty.rs](https://doc.rust-lang.org/stable/nightly-rustc/src/compiletest/runtest/pretty.rs.html) · [rustc fuzzing](https://rustc-dev-guide.rust-lang.org/fuzzing.html) · [PostgreSQL query path](https://www.postgresql.org/docs/current/query-path.html) · [PostgreSQL parser stage](https://www.postgresql.org/docs/current/parser-stage.html) · [PostgreSQL gram.y](https://github.com/postgres/postgres/blob/master/src/backend/parser/gram.y) · [PostgreSQL kwlist.h](https://github.com/postgres/postgres/blob/master/src/include/parser/kwlist.h) · [kwlist.h doxygen](https://doxygen.postgresql.org/kwlist_8h_source.html) · [PostgreSQL SQL Key Words appendix](https://www.postgresql.org/docs/current/sql-keywords-appendix.html) · [Wikipedia SQL-92](https://en.wikipedia.org/wiki/SQL-92) · [ronsavage/SQL BNF grammars](https://github.com/ronsavage/SQL) · [DuckDB PEG parsers](https://duckdb.org/2024/11/22/runtime-extensible-parsers) · [Calcite adapters](https://calcite.apache.org/docs/adapter.html) · [Calcite Parser.jj](https://github.com/apache/calcite/blob/main/core/src/main/codegen/templates/Parser.jj) · [Calcite SqlValidator](https://calcite.apache.org/javadocAggregate/org/apache/calcite/sql/validate/SqlValidator.html) · [Calcite algebra](https://calcite.apache.org/docs/algebra.html) · [Calcite VolcanoPlanner](https://calcite.apache.org/javadocAggregate/org/apache/calcite/plan/volcano/VolcanoPlanner.html) · [Calcite paper (arXiv)](https://arxiv.org/abs/1802.10233) · [DuckDB SIGMOD 2019](https://duckdb.org/pdf/SIGMOD2019-demo-duckdb.pdf) · [datafusion-sqlparser-rs](https://github.com/apache/datafusion-sqlparser-rs) · [google/googlesql](https://github.com/google/googlesql) · [ZetaSQL resolved_ast docs](https://github.com/google/zetasql/blob/master/docs/resolved_ast.md) · [Materialize debugging query performance](https://materialize.com/blog/debugging-query-performance/) · [Materialize EXPLAIN PLAN](https://materialize.com/docs/sql/explain-plan/) · [Materialize architecture](https://materialize.com/blog/materialize-architecture/) · [DBSP VLDB 2023](https://docs.feldera.com/vldb23.pdf) · [Feldera publications](https://docs.feldera.com/literature/papers/) · [DBSP Lean formalization](https://github.com/tchajed/database-stream-processing-theory) · [RisingWave streaming engine](https://risingwavelabs.github.io/risingwave/design/streaming-overview.html) · [Noria OSDI '18](https://www.usenix.org/conference/osdi18/presentation/gjengset) · [Gjengset PhD thesis](https://jon.thesquareplanet.com/papers/phd-thesis.pdf) · [readysettech/readyset](https://github.com/readysettech/readyset) · [rust-analyzer syntax](https://rust-analyzer.github.io/book/contributing/syntax.html) · [Resilient LL parsing tutorial](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html)agentId: a04286db9c5d6bb0c (use SendMessage with to: 'a04286db9c5d6bb0c', summary: '<5-10 word recap>' to continue this agent)
<usage>subagent_tokens: 155021
tool_uses: 102
duration_ms: 972571</usage>