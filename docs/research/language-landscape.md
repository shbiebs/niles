# The Language Landscape: Does the Required Language Exist?

*A citation-backed survey against a 17-point requirement list.*
Compiled 2026-09-01. Sources are primary where available; every unverified claim is flagged
in [§7 What I Could Not Verify](#7-what-i-could-not-verify).

---

## 0. The requirement list, normalized

The list, restated as eleven testable clusters so that languages can be scored against it.

| ID | Cluster | Requirements folded in |
|----|---------|------------------------|
| **C1** | **Native performance** | speed of C; compiles to machine code; no interpreter; eliminates interpreter costs |
| **C2** | **No runtime, no GC** | fast, compiled, *no runtime*, *no garbage collector* |
| **C3** | **Memory safety** | memory-safe (in the strong, Rust-like sense) |
| **C4** | **Runtime dynamism** | dynamism of Ruby; general-purpose like Python; glue like the shell |
| **C5** | **Metaprogramming** | homoiconic; true Lisp macros; DSLs that compile to bare metal; specialized code per query shape |
| **C6** | **Notation & array/dataframe** | obvious familiar mathematical notation like MATLAB; linear algebra; dataframe/array language with explicit composable analytics ops |
| **C7** | **Domain breadth** | statistics as R; string processing as Perl; glue as shell |
| **C8** | **Relational / SQL replacement** | replaces SQL; query/filter/join/aggregate large datasets; set-at-a-time semantics; nested subqueries and CTEs; clean functional composition |
| **C9** | **Plan control** | a better way to express queries so the engine can do more, **and** more explicit control to avoid pathological plans |
| **C10** | **Deductive / recursive** | logic language; recursive and graph queries far more efficiently than SQL; iterative graph algorithms, recursive analytics, ML without impedance mismatch |
| **C11** | **Vectorized execution** | columns in cache-fitting batches, SIMD, not row-by-row; no cache misses |
| **C12** | **Zero-copy** | avoids type conversion and copying |
| **C13** | **Strict serializability** | strict serializability |

(Thirteen, not eleven — C9 and C13 turn out to be the load-bearing ones.)

---

## 1. Executive summary — the direct answer

### **No. No such language exists.**

Stronger, and more useful: **the list as written cannot be satisfied by any language, because at
least three of its requirements are not "hard" but *incoherent* — either category errors or
mutual contradictions.** The remaining requirements are individually achieved somewhere, and the
best existing systems cover perhaps 60–70% of the list, but no artifact covers more than about
half, and two of the gaps are not engineering gaps.

Three findings, in order of importance:

**Finding 1 — "Strict serializability" is a category error at the language layer (C13).**
Strict serializability is serializability (Papadimitriou 1979) plus a real-time order constraint
in the sense of Herlihy & Wing's linearizability (1990). It is a property of a *concurrency-control
protocol over a durable store*, not of a programming language. No general-purpose programming
language provides it, and none can: it requires a commit protocol, a timestamp authority, and
durable storage — i.e. **a substantial runtime**. So C13 directly contradicts C2 ("no runtime").
A language can *express* transactions, *type* isolation levels, and *statically reject* programs
that would violate them — which is exactly what the Niles consistency-effect calculus is for — but
the guarantee is discharged by the engine, not the language. This is the single most important
correction to the list.

**Finding 2 — "Dynamism of Ruby" + "no runtime" is a contradiction, and Julia is the experimental
proof.** Julia was founded on precisely the first cluster of this list (attribution confirmed,
[§4](#4-attribution-check)), and pursued it for fourteen years with substantial funding. As of
Julia 1.12 (stable release 1.12.4 as of early 2026), the `juliac` AOT compiler produces standalone
binaries **only by prohibiting dynamic dispatch** — that is, only by deleting the dynamism. LWN's
review of 1.12: the binaries "are not exactly small, not exactly standalone, and severely limited
in scope"; hello-world yields "a 1.7MB binary and a directory of library files that occupied a
further 91MB"; "programs cannot read from files or from the terminal." A 2026 HEP case study
compiling `JetReconstruction.jl` produced a ~300 MB binary versus 8 MB for the C++ equivalent, had
to strip try/catch, logging and CPU-feature queries, and still concluded the authors "don't yet
have a solid production solution." This is not Julia failing; this is the requirement being
self-cancelling. Late binding requires *something* at run time to dispatch and to reclaim values
whose extents are not statically known. "No runtime" and "runtime dynamism" name the same resource
with opposite signs.

**Finding 3 — the two tensions that look fundamental are actually solved, and the solution is the
same architectural move in both cases: split the language in two.**
- C9 (declarative + explicit plan control) is dissolved by Halide's separation of *algorithm* from
  *schedule* (Ragan-Kelley et al., PLDI 2013) — two languages, one interface, full declarativity in
  one and total control in the other. Neumann & Leis independently reach for the same move in SaneQL
  (CIDR 2024): "a version of the language with additional operator hints could also be used to
  represent physical query plans."
- C5 (homoiconic + infix notation) is dissolved by Lean 4 and Racket — give up *homoiconicity* (the
  identity of program text and data), keep *macro power* (extensible surface syntax + hygienic
  procedural macros over a typed AST). Lean 4's `Syntax` inductive type with `notation` /
  `macro_rules` / `elab` demonstrates full Lisp macro power over mixfix notation (Ullrich & de Moura,
  ITP 2020).

And one tension that is *not* a tension at all: C1+C11 (compiled *and* vectorized) was empirically
settled by Kersten et al. (VLDB 2018) — the two paradigms perform within noise of each other on
OLAP, and InkFuse (Wagner, Kohn, Boncz, Leis) shows they can be unified in a single engine.

### Closest candidates, ranked

| Rank | Language | Clusters met | The gap |
|------|----------|--------------|---------|
| 1 | **Julia** | C1(~), C4, C6, C7, C10(~) | Has a GC and a runtime; not memory-safe in the Rust sense; AOT story still experimental; no query layer; no plan control |
| 2 | **Rust** | C1, C2, C3, C11, C12(~), C5(~) | No dynamism; no extensible infix notation; not a query language (it *hosts* them: DataFusion, Polars); no set-at-a-time semantics |
| 3 | **Mojo** | C1, C2, C3(~), C6, C11 | Dynamism only via embedded CPython (re-importing CPython's GC); no macros/homoiconicity; no query layer; ecosystem young |
| 4 | **Lean 4** | C2(~ RC not tracing GC), C5 (best in class), C1(~) | Not a performance-engineering language; no vectorization; no query layer; RC cannot collect cycles; proof assistant first |
| 5 | **Racket** | C5 (gold standard), C4, C7 | Fails C1, C2, C3, C11 entirely — Chez-backed, GC'd, not AOT-to-machine-code in the required sense |
| — | **Terra** | C1, C2, C5, C4 (in the Lua meta-layer) | *The architecture is right* — two layers, dynamic meta / static object — but no memory safety, no query layer, minimal maintenance |
| — | **Soufflé** | C10 (best in class), C1 | Not general-purpose; no transactions; no incrementality; no dynamism |

**No candidate exceeds ~6 of 13 clusters.** The union of Rust + Terra's staging model + Lean 4's
macro system + Soufflé's Datalog backend + a Halide-style schedule language covers most of the
coherent subset — which is a design brief, not a language that exists.

---

## 2. Scoring table (languages × requirement clusters)

Legend: **✓** meets · **~** partial / with caveats · **✗** does not meet · **n/a** out of scope by design

| Language | C1 native | C2 no rt/GC | C3 mem-safe | C4 dynamism | C5 macros | C6 notation/array | C7 breadth | C8 relational | C9 plan ctrl | C10 deductive | C11 vectorized | C12 zero-copy | C13 strict-ser |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **Julia** | ~ | ✗ | ~ | ✓ | ~ | ✓ | ✓ | ~ | ✗ | ~ | ~ | ~ | ✗ |
| **Rust** | ✓ | ✓ | ✓ | ✗ | ~ | ~ | ~ | ✗ | n/a | ✗ | ✓ | ~ | ✗ |
| **Zig** | ✓ | ✓ | ✗ | ~ | ~ | ~ | ~ | ✗ | n/a | ✗ | ✓ | ✓ | ✗ |
| **Nim** | ✓ | ~ | ~ | ~ | ✓ | ~ | ~ | ✗ | n/a | ✗ | ~ | ~ | ✗ |
| **Mojo** | ✓ | ✓ | ~ | ~ | ✗ | ✓ | ~ | ✗ | n/a | ✗ | ✓ | ✓ | ✗ |
| **Terra + Lua** | ✓ | ✓ | ✗ | ✓ | ✓ | ~ | ~ | ✗ | n/a | ✗ | ~ | ✓ | ✗ |
| **Common Lisp (SBCL)** | ~ | ✗ | ~ | ✓ | ✓ | ✗ | ✓ | ✗ | n/a | ✗ | ✗ | ✗ | ✗ |
| **Racket** | ✗ | ✗ | ~ | ✓ | ✓ | ~ | ✓ | ✗ | n/a | ~ | ✗ | ✗ | ✗ |
| **Clojure** | ✗ | ✗ | ~ | ✓ | ✓ | ✗ | ~ | ~ | ✗ | ~ | ✗ | ✗ | ✗ |
| **Lean 4** | ~ | ~ | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | n/a | ~ | ✗ | ✗ | ✗ |
| **Koka** | ~ | ~ | ✓ | ✗ | ~ | ✗ | ✗ | ✗ | n/a | ✗ | ✗ | ✗ | ✗ |
| **Vale** | ~ | ✓ | ✓ | ~ | ✗ | ✗ | ✗ | ✗ | n/a | ✗ | ✗ | ~ | ✗ |
| **Austral** | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | n/a | ✗ | ✗ | ✓ | ✗ |
| **Futhark** | ✓ | ~ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ~ | ✗ | ✓ | ~ | n/a |
| **Halide** | ✓ | ✓ | ~ | ✗ | ✓ | ✓ | ✗ | ✗ | **✓** | ✗ | ✓ | ✓ | n/a |
| **Chapel** | ✓ | ✗ | ~ | ~ | ~ | ✓ | ~ | ✗ | ~ | ✗ | ~ | ~ | ✗ |
| **ATS** | ✓ | ✓ | ✓ | ✗ | ~ | ✗ | ✗ | ✗ | n/a | ✗ | ✗ | ✓ | ✗ |
| **K / q / kdb+** | ✓ | ~ | ~ | ✓ | ✗ | ✓ | ~ | ✓ | ~ | ✗ | ✓ | ✓ | ~ |
| **APL / BQN / J** | ~ | ✗ | ~ | ✓ | ~ | ✓ | ~ | ✗ | n/a | ✗ | ~ | ✗ | ✗ |
| **R** | ✗ | ✗ | ~ | ✓ | ~ | ✓ | ✓ | ~ | ✗ | ✗ | ✗ | ✗ | ✗ |
| **MATLAB** | ✗ | ✗ | ~ | ✓ | ✗ | ✓ | ~ | ✗ | ✗ | ✗ | ~ | ✗ | ✗ |
| **Soufflé** | ✓ | ✓ | ~ | ✗ | ~ | ✗ | ✗ | ~ | ~ | **✓** | ~ | ~ | ✗ |
| **DDlog (archived)** | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ~ | ✗ | ✓ | ✗ | ~ | ✗ |
| **DBSP / Feldera** | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ~ | ~ | ~ |
| **Rel (RelationalAI)** | ? | ✗ | ? | ~ | ~ | ~ | ~ | ✓ | ✗ | ✓ | ? | ? | ? |
| **PRQL / Malloy / Logica** | ✗ | ✗ | n/a | ✗ | ~ | ~ | ✗ | ~ | ✗ | ~ | ✗ | ✗ | ✗ |
| **EdgeQL** | ✗ | ✗ | n/a | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ~ | ✗ | ✗ | ~ |
| **SaneQL** | n/a | n/a | n/a | ✗ | ~ | ✗ | ✗ | ✓ | ~(proposed) | ✗ | n/a | n/a | n/a |
| **Morel** | ✗ | ✗ | ~ | ✗ | ~ | ~ | ✗ | ✓ | ✗ | ~ | ✗ | ✗ | ✗ |
| **Flix** | ✗ | ✗ | ✓ | ✗ | ~ | ✗ | ~ | ~ | ✗ | ✓ | ✗ | ✗ | ✗ |
| **Gleam / Roc / Carp** | ~ | ~ | ✓ | ✗ | ~ | ✗ | ~ | ✗ | n/a | ✗ | ✗ | ✗ | ✗ |

Two things the table makes visible:

1. **The C1–C3 column block and the C4 column are almost perfectly anti-correlated.** Every language
   that scores ✓ on native/no-GC/memory-safe scores ✗ on dynamism, and vice versa. The only
   entries that get both are the *two-layer* systems (Terra+Lua) — which cheat by being two
   languages.
2. **No row has a ✓ in both C8 and C2.** Every system that genuinely replaces SQL carries a runtime.
   This is not an accident; see [§3.7](#37-relational-semantics-vs-no-gc).

---

## 3. The tensions analysis

This is the substantive part. For each, I state the verdict: **fundamental** (incoherent or provably
costly), **structural** (real, but dissolvable at a known architectural price), or **contingent**
(merely unsolved, or already solved).

### 3.1 (a) "Dynamism of Ruby" + "no GC" + "AOT compiled, no runtime"

**Verdict: two distinct claims, one FUNDAMENTAL and one CONTINGENT. The requirement as written
conflates them.**

Decompose:

- **(a1) dynamism + "no *tracing* GC"** — *contingent, and largely solved.* You can have automatic
  memory management without a tracing collector. The frontier:
  - **Perceus** (Reinking, Xie, de Moura, Leijen; PLDI 2021) — *precise* reference counting with
    reuse analysis, formalized over a linear resource calculus λ₁. Koka's purely functional
    red-black tree lands "within 10% of the performance" of C++'s `std::map`, and beats OCaml,
    Haskell, Swift and Java on allocation-heavy benchmarks with 3–10× lower memory than Java.
  - **Lean 4** — same lineage (de Moura is on both papers). RC with borrowing analysis to elide
    increments, thread-partitioned counts (non-shared values use cheap non-atomic RC), and
    opportunistic in-place update via `isShared` checks in the IR. Compiles to C.
  - **Swift ARC**, **Nim ARC/ORC** — production-grade versions of the same idea. Nim's ORC = ARC
    plus a cycle collector, and is Nim's default.
  - **Vale generational references** (Verdagon/Evan Ovadia) — a pointer plus a remembered
    generation number, checked on deref. Measured **+10.84%** over unsafe, versus **+25.29%** for
    reference counting. Allows mutable aliasing (observers, callbacks) that borrow checking
    rejects, at the price of a runtime check that regions and linear style can eliminate.
  - **Rust** — compile-time only, zero runtime cost, at the price of rejecting exactly the aliasing
    patterns dynamism depends on.

  **The catch, and it is a real one:** Perceus and Lean 4 both *assume acyclic data*. Lean's manual
  is explicit: "Because the verifiable fragment of Lean cannot create cyclic data, the Lean runtime
  does not have a technique to detect it." Perceus lists cycles as "a known limitation," requiring
  "explicit clearing" by the programmer. **Ruby-style dynamism produces cycles routinely** —
  mutable object graphs, closures capturing each other, observer registrations, parent/child
  back-pointers. So the honest statement is: *RC-without-tracing works for the functional,
  acyclic fragment; the moment you add Ruby's object model you need either a cycle collector
  (Nim ORC — which is a partial tracing GC), weak references by hand (Swift), or generational
  references (Vale).* The cheapest general-purpose answer today is Vale's, and Vale is a research
  language.

- **(a2) dynamism + "no runtime"** — **FUNDAMENTAL. This is a contradiction.** "Dynamism" *means*
  decisions deferred to run time; the machinery that makes those decisions *is* a runtime. You
  cannot have method dispatch on a type not known until run time without a dispatch mechanism
  present at run time. The strongest available evidence:
  - **Julia**: `juliac`'s "main limitation is the prohibition of dynamic dispatch." The AOT path
    is bought by removing the very property the manifesto asked for. And it is still not
    "no runtime" — the 1.12 hello-world ships 91 MB of libraries alongside a 1.7 MB binary; the
    HEP case study's binary was ~300 MB.
  - **Mojo** (v1.0, fully open-sourced under Apache-2.0-with-LLVM-exceptions on 2026-08-18):
    ownership + argument-exclusivity checking + deterministic destruction, **no GC**. But Mojo's
    Python dynamism is obtained by *embedding CPython*: the docs describe using "the CPython
    runtime without modification for full compatibility with existing Python libraries," requiring
    Python 3.10–3.14. The dynamism is real; it is provided by a runtime with a reference-counting
    collector and a cycle detector, running in-process. The GC did not disappear; it moved.
  - **Terra**: the only design that is honest about the split — a *dynamic* Lua meta-layer that
    runs at compile time, and a *static, GC-free, LLVM-compiled* Terra object layer. You get
    dynamism where dynamism is cheap (staging) and none where it is expensive (execution).

  **The frontier, precisely stated:** you can have (i) full dynamism with a runtime and RC or
  generational references and no *tracing* GC — Vale, Swift, Nim-ORC, Mojo-with-CPython; or
  (ii) zero runtime with dynamism confined to a *compile-time* meta-stage — Terra, Zig `comptime`,
  Rust proc-macros, Julia's generated functions. **What nobody has is dynamism at run time with
  nothing at run time, and nobody will.**

  *Corollary that matters for this thesis:* C4 is also in tension with the project's own
  Contribution 4. Statically checkable money conservation, currency non-mismatch, and
  authorization requires static type information. Ruby-level dynamism destroys precisely the
  information the soundness theorem needs. The requirement list and the thesis goals disagree with
  each other here, not just internally.

### 3.2 (b) "Homoiconic with Lisp macros" + "familiar infix mathematical notation"

**Verdict: STRUCTURAL, and solved — but only by abandoning homoiconicity, which turns out to be
unnecessary.**

Homoiconicity in the strict sense means program *text* is written in the notation of the language's
own primary data structure — s-expressions. Any surface syntax that is not s-expressions is,
definitionally, not homoiconic. So the requirement as written is self-contradictory: *"homoiconic"
and "obvious familiar mathematical notation like MATLAB" cannot both hold.*

But nobody actually needs homoiconicity. What they need is **macro power**, which decomposes into
four independent capabilities:

| | (i) structured program representation | (ii) quasiquotation | (iii) hygiene | (iv) user-extensible surface syntax |
|---|---|---|---|---|
| **Common Lisp** | ✓ (identical to source) | ✓ | ✗ (unhygienic; `gensym` by hand) | ✓ (reader macros) |
| **Scheme / Racket** | ✓ | ✓ | ✓ | ✓ (`#lang`, replaceable reader) |
| **Lean 4** | ✓ (`Syntax` inductive type) | ✓ | ✓ (scope-tagged `Name`s) | ✓ (`notation`, `syntax`, `macro_rules`, `elab`) |
| **Julia** | ✓ (`Expr` objects) | ✓ (`quote` / `:()`) | ✓ (mostly; `esc` to break it) | **✗** — the parser and operator-precedence table are fixed |
| **Nim** | ✓ (typed AST) | ✓ | ✓ | ~ (templates/macros over existing syntax; no new precedence levels) |
| **Rust** | ~ (`proc_macro::TokenStream` is deliberately *not* an AST; `syn` reconstructs one out of band) | ✓ (`quote!`) | ~ (mixed — `macro_rules!` is partially hygienic, proc macros are not) | ~ (only inside `mac!(...)` delimiters) |
| **Elixir** | ✓ (quoted `{op, meta, args}` triples) | ✓ | ✓ | ~ (fixed operator set, can be re-bound) |

**The both-at-once answer exists, twice.**

- **Lean 4** is the cleanest demonstration. Its `Syntax` type is
  `node (kind) (args) | atom | ident | missing` — a general tree, not s-expressions — and the
  system provides three graded levels (`notation` for pattern sugar, `macro_rules` for procedural
  transformers, `syntax` + `elab` for full metaprograms), with the paper stating "there is no
  distinction between pattern-based and procedural macros." Hygiene is implemented by embedding
  macro scopes in identifier `Name`s, explicitly designed to avoid the quadratic scope
  proliferation of Racket's approach. Crucially, Lean keeps a *concrete* syntax tree pre-expansion,
  which is what makes IDE completion and refactoring possible — something homoiconic Lisps
  historically lack. And Lean's `notation` command lets you define arbitrary mixfix operators with
  arbitrary precedence, so `A * B`, `∑`, `⟦x⟧` are all user-definable.
- **Racket** is the other, from the opposite direction: `#lang` lets you replace the *reader*, so an
  infix mathematical surface language and a full hygienic macro system coexist by construction
  (Felleisen et al., "A Programmable Programming Language," CACM 2018).

**Julia is the interesting negative result** for this requirement: it has (i)–(iii) with infix
notation and is widely described as homoiconic, but it *fails (iv)*. Julia's parser is fixed; you
can overload the meaning of the finite set of operator characters, and you can define macros
invoked with `@`, but you cannot add a new infix operator at a new precedence level. So the
"homoiconic like Lisp *and* familiar notation like MATLAB" claim, at Julia specifically, is a
partial delivery: notation ✓, macros ✓, *extensible* notation ✗.

**Recommendation:** replace "homoiconic" in the requirement list with **"user-extensible mixfix
surface syntax over a typed, quotable, hygienic AST."** That is achievable, has two working
precedents, and is what was actually wanted.

### 3.3 (c) "Set-at-a-time declarative, let the engine do more" + "explicit control to avoid pathological plans"

**Verdict: STRUCTURAL — genuinely opposed *within a single syntactic layer*, dissolved by
two-layer separation. The price is that you must give up the idea that one language does both.**

Stated in one layer, these requirements are opposed by construction: the freedom the optimizer needs
to rewrite is exactly the freedom that explicit control removes. Every hint you honour is a rewrite
you forbid. This is not a slogan — it is the reason the PostgreSQL project has refused hints for
25+ years. Their stated objections:

> Hints embedded in queries demand extensive code refactoring during updates; beneficial hints often
> become performance liabilities after database upgrades; [they encourage] applying quick fixes
> rather than addressing root causes; hints optimized for small tables prove ineffective as data
> grows; the optimizer typically outperforms hint-based approaches; users with hints rarely report
> optimizer problems upstream.

Note what those objections are actually *about*: they are objections to hints that are (1)
syntactically embedded in the query, (2) unverifiable, (3) absolute rather than advisory, and (4)
not versioned against the schema/statistics they were tuned for. They are **not** objections to
giving the user control over the plan. Oracle, SQL Server (plan guides / Query Store forced plans)
and MySQL all ship hint mechanisms; the survey literature treats them as necessary evils rather than
design errors.

**The resolution is Halide's.** Ragan-Kelley, Barnes, Adams, Paris, Durand, Amarasinghe (PLDI 2013)
decouple *what to compute* (the algorithm — pure, declarative, order-free) from *how to compute it*
(the schedule — total control over tiling, vectorization, parallelism, fusion, storage). Their
framing:

> "We address this challenge by raising the level of abstraction, and decoupling the algorithm
> definition from its execution strategy, to improve portability and composability."
> "Halide's representation of image processing algorithms avoids imposing constraints on the order
> of execution and placement of data."

The empirical payoff they report: 52 lines of Halide beating a 262-line expert hand-tuned local
Laplacian filter by 1.7×. The same architecture is now standard in the compiler world — TVM, Exo,
Taichi, and TensorComprehensions all copy it.

**Critically, the database community is converging on the same move independently.** Neumann & Leis,
in SaneQL (CIDR 2024), close with exactly this: *"a version of the language with additional operator
hints could also be used to represent physical query plans."* And Jamie Brandon's "Against SQL"
lists, among the requirements for a post-SQL language, **"Exposed APIs for plans and hints (binary +
human-readable encodings)"** — i.e. the plan as a first-class, inspectable, writable value.

**So the honest formulation for the requirement list is:**
> *Two languages over one IR: a declarative relational algorithm language with no ordering
> commitments, and a separate schedule/plan language whose terms are first-class values that can be
> named, stored, diffed, version-pinned against statistics, and — this is the part hints lack —
> **checked for semantic equivalence to the algorithm.***

That last clause is the research contribution available here: Postgres's objections all reduce to
"hints are unverified." A schedule language whose terms are *proven* to preserve the algorithm's
denotation eliminates five of the six objections. This is a strictly better position than either
"trust the optimizer" or "trust the hint."

### 3.4 (d) "Vectorized execution" + "compile DSLs to bare metal"

**Verdict: CONTINGENT — and already settled. These are not alternatives; they are combinable, and
the leading systems combine them.**

The canonical source is exactly the one asked for: **Timo Kersten, Viktor Leis, Alfons Kemper,
Thomas Neumann, Andrew Pavlo, Peter Boncz, "Everything You Always Wanted to Know About Compiled and
Vectorized Queries But Were Afraid to Ask," PVLDB 11(13), 2018.**

What it actually concluded — and it is more deflationary than the reputation suggests:

> "To our surprise, the performance of vectorized and data-centric compiled query execution is quite
> similar in OLAP workloads."

Their measured range across single-threaded TPC-H was compiled (Typer) 74% faster on Q1 to
vectorized (Tectorwise) 32% faster on Q9 — small next to the gap between either and a traditional
engine. The mechanism behind the split:

- **Compiled wins** on computation-heavy queries with few cache misses, because intermediates stay
  in registers. *"Typer is more efficient for computational queries with few cache misses."*
- **Vectorized wins** on memory-bound hash operations (joins, aggregations), because a batch of
  independent lookups exposes memory-level parallelism the compiler cannot manufacture.
  *"Tectorwise is slightly better at hiding cache miss latency."*

And a finding directly relevant to the C11 requirement as stated: **SIMD is much less important than
the requirement assumes.** AVX-512 micro-benchmarks showed 8.4× speedups; real TPC-H queries saw
~1.4×, because *"most OLAP queries are bound by data access, which does not (yet) benefit much from
SIMD."* Their overall recommendation was that since neither paradigm dominates, *"other factors like
OLTP performance or implementation effort... may be of greater importance."*

What modern systems actually do:

| System | Approach |
|---|---|
| **DuckDB** | Vectorized interpretation, no JIT |
| **Photon** (Databricks) | Vectorized *by deliberate choice over* code generation |
| **Umbra** (TUM) | Compiled, but *adaptive and multi-tier*: custom Umbra IR → direct x86-64/ARM64 machine-code emission for linear-time compilation, escalating to LLVM in the background for long-running queries; morsel-driven parallelism provides the switching points |
| **HyPer** | Hybrid: vectorization for base-table selections and decompression, data-centric codegen for everything else |
| **InkFuse** (Wagner, Kohn, Boncz, Leis) | Explicit unification — "Incremental Fusion" decomposes operators into *suboperators* satisfying an **Enumeration Invariant** (finitely many instantiations, enumerable ahead of time), so the *same* codegen stack produces both a pre-compiled vectorized interpreter and fused compiled pipelines, with a hybrid backend switching at runtime on measured throughput |

Photon's justification for choosing vectorization is worth quoting because it is *not* about
performance:

> "Code generation typically eliminates interpretation and function call overheads by collapsing and
> inlining operators into a small number of pipelined functions. Although this is great for
> performance, it makes observability difficult."

and on engineering cost: two months to prototype aggregation with codegen, "a couple weeks with the
vectorized engine."

**The real, residual tension is not vectorized-vs-compiled. It is compile latency vs. peak
throughput.** Per-query-shape specialization pays only when the query runs long enough to amortize
compilation. Umbra's answer (bytecode → custom IR → LLVM, chosen adaptively) and InkFuse's answer
(pre-enumerated primitives, fuse incrementally) are the two known solutions, and both require
**a single typed IR with two backends** — which is precisely the architecture the Niles thesis
already proposes. This requirement is safe; budget for the second backend.

### 3.5 (e) "Memory-safe" + "no copies, no type conversion"

**Verdict: BOUNDED-FUNDAMENTAL. Safe zero-copy over *trusted* bytes is solved. Safe zero-copy over
*untrusted* bytes is impossible without an O(n) validation pass — which is the copy you were trying
to avoid, in scan form.**

Two separate situations, routinely conflated:

**(1) Zero-copy within the program's own memory** — Rust solves this. `&[T]`, slices, `Cow`,
lifetime-parameterized borrows, `bytemuck`/`zerocopy` for POD reinterpretation. The friction is
ergonomic, not soundness-level: lifetimes must thread through every struct that holds a borrow, and
self-referential structures need `Pin` or `unsafe`. Real but survivable — Apache Arrow's Rust
implementation, Polars and DataFusion are existence proofs that a complete columnar analytics stack
can be built this way.

**(2) Zero-copy from external bytes — mmap'd pages, network frames, on-disk ledger segments — is
where the guarantee genuinely breaks.** Casting a byte range to a typed view asserts that the bytes
satisfy the type's invariants (valid discriminants for enums, non-null pointers, valid UTF-8, no
overlapping offsets, in-bounds lengths). If the bytes came from anywhere you do not control —
another process, a corrupted page, a malicious client — that assertion is unchecked and the cast is
`unsafe` in the technical sense: violating it is undefined behaviour, not a wrong answer. `rkyv`
acknowledges exactly this and ships an *optional* `bytecheck` validation layer; validation is O(n)
over the accessed structure.

**So the sharp statement is a pick-two:**

> Of { **zero-copy**, **memory-safe**, **untrusted input** } you may have any two.
> - zero-copy + memory-safe ⟹ input must be trusted (your own process, or a verified writer)
> - memory-safe + untrusted ⟹ validate, which is O(n) touch even if not O(n) allocation
> - zero-copy + untrusted ⟹ unsafe

For a ledger-anchored system this is actually a *usable* result rather than a problem, and it
interacts well with the thesis's existing commitments: **a hash-chained, append-only ledger written
only by the engine itself is precisely a "trusted writer."** The hash chain *is* the validation, and
it is amortized per-epoch rather than per-read. Zero-copy reads over sealed, hash-verified epochs
are sound; zero-copy reads over client-supplied bytes are not. That boundary is worth drawing
explicitly in the architecture.

A second, smaller point on "avoids type conversion": columnar layouts and the row-oriented
double-entry domain model want different physical representations. Some conversion is not
accidental — it is the price of having both C6 (array/dataframe) and C8 (relational rows) in one
system. Arrow's answer is to make the columnar form canonical and pay conversion only at the
boundary; that is the right default.

### 3.6 (f) "Strict serializability" — the category error

**Verdict: FUNDAMENTAL, and a category error. Directly contradicts C2.**

Strict serializability = serializability (a *transactional* property: some serial order exists that
explains the observed history) + linearizability's real-time constraint (Herlihy & Wing, 1990: if
transaction A completes before B begins in wall-clock time, A precedes B in the serial order). It is
the strongest of the standard consistency models (Jepsen's hierarchy puts it at the top).

It is a property of a **history of operations against a store**, produced by a **concurrency-control
protocol**. Programming languages do not have histories. The mechanisms that produce strict
serializability — two-phase commit, a timestamp oracle or TrueTime-style bounded clock uncertainty,
a WAL, a coordinator — are, collectively, a runtime. **"No runtime" and "strict serializability" are
therefore in direct contradiction.**

What a *language* can contribute, and this is a real contribution rather than a consolation prize:

- **Express** transaction boundaries as syntax with defined semantics.
- **Type** isolation/consistency levels as effects, so a function that reads at bounded-staleness
  cannot be called from a context that promised strict serializability. (This is exactly the
  consistency-effect calculus the thesis proposes; the effect-system machinery for it exists — Koka,
  Flix, Eff — and is well understood.)
- **Statically reject** programs whose effect signature exceeds the serve contract of the view they
  target.
- **Nothing else.** The guarantee is discharged by the engine.

The closest a language has come to owning a transactional guarantee is **STM** — Haskell's `STM`
monad and Clojure's refs — which give *opacity* over in-memory state, not durability and not
real-time order across processes. That is a strictly weaker property in a strictly smaller domain.

### 3.7 Relational semantics vs. "no GC"

**Verdict: FUNDAMENTAL as stated, but the requirement is mis-stated and the correct version is
achievable — and happens to align with the thesis's own epoch model.**

Not on the user's list explicitly, but it falls out of the combination of C2 and C8, and it is the
one that will actually bite during implementation.

Relational values are *sets*. Their lifetimes are determined by dataflow, not by lexical scope.
Under partially-stateful materialization with eviction and upqueries — the core of this thesis —
the *extent* of live state is decided at run time by the read workload. There is no lexical scope
whose end coincides with "no reader will ever ask for this row again." That is the textbook
definition of a workload requiring dynamic reclamation.

Real engines solve it with **epoch-based reclamation** (EBR): readers pin an epoch, writers retire
objects into an epoch's garbage list, and reclamation happens once all readers have advanced past
it. `crossbeam-epoch`, RCU in the Linux kernel, and hazard pointers are the standard implementations.
**EBR is a garbage collector** — a deferred, non-tracing, non-moving, cooperatively-scheduled one,
but a collector.

So the achievable version of C2 is:

> **"No language-level *tracing* GC. Reclamation is explicit, epoch-scoped, and deterministic; no
> stop-the-world pauses; memory extent is bounded by the epoch frontier rather than by a heuristic
> collector."**

That is a meaningful, defensible, testable property. "No GC" full stop, alongside evictable derived
state, is not. And note the pleasing coincidence: **the thesis already has epochs.** The epoch-ordered
ledger that defines the visibility timeline is also, exactly, the right reclamation domain. Reclaim at
epoch boundaries, and memory management becomes a corollary of the consistency model rather than a
separate subsystem. This is arguably a small additional contribution worth naming.

### 3.8 Summary of verdicts

| Tension | Verdict | Cost of the resolution |
|---|---|---|
| dynamism + no **tracing GC** | contingent, solved-ish | RC/generational refs; cycles need explicit breaking or a cycle collector |
| dynamism + **no runtime** | **FUNDAMENTAL** | must confine dynamism to a compile-time meta-stage (Terra/Zig `comptime` model) |
| homoiconic + infix notation | structural, **solved** | drop "homoiconic"; adopt extensible mixfix over a typed AST (Lean 4 / Racket) |
| declarative + plan control | structural, **solved** | two languages over one IR (Halide model); add equivalence checking to beat hints |
| vectorized + compiled | contingent, **settled** | one typed IR, two backends; adaptive tier selection (Umbra / InkFuse) |
| memory-safe + zero-copy | **bounded-fundamental** | pick two of {zero-copy, safe, untrusted}; trusted-writer ledger makes this tractable |
| **strict serializability** in a language | **FUNDAMENTAL (category error)** | move it to the engine; the language types the *effect*, not the guarantee |
| set-at-a-time + no GC | **FUNDAMENTAL as stated** | restate as "no tracing GC, epoch-scoped reclamation"; then achievable |

---

## 4. Attribution check

### 4.1 The Julia manifesto — **CONFIRMED**

Source: Jeff Bezanson, Stefan Karpinski, Viral B. Shah, Alan Edelman, *"Why We Created Julia,"*
julialang.org blog, February 2012. The blog post contains, verbatim, the material the requirement
list paraphrases:

> "We want the speed of C with the dynamism of Ruby."
> "We want something that is homoiconic, with true macros like Lisp, but with obvious, familiar
> mathematical notation like Matlab."
> "We want something as usable for general programming as Python, as easy for statistics as R, as
> natural for string processing as Perl, as powerful for linear algebra as Matlab, as good at
> gluing programs together as the shell."
> "We want to write simple scalar loops that compile down to tight machine code... We want to write
> `A*B` and launch a thousand computations on a thousand machines."

The post explicitly acknowledges the maximalism: *"we are greedy: we want more"* and refers to the
list as their *"ungracious demands."* **The first cluster of the requirement list is the Julia
manifesto, near-verbatim.** Note that "no runtime, no garbage collector" and "memory-safe" are
**not** in the Julia post — those are the user's additions, and they are the additions that make the
list impossible.

### 4.2 "Against SQL" — **CONFIRMED**

Source: Jamie Brandon, *"Against SQL,"* scattered-thoughts.net, 2021-07-09. The second cluster of
the requirement list echoes it accurately.

Brandon's argument, in his structure:

1. **Inexpressiveness** — no sum types, no true recursion, no first-class functions or values. New
   capabilities (JSON, XML, windowing) must be added to the *spec* rather than written as libraries.
   *"If your database query language is not the right tool for querying data, that seems like a
   problem."*
2. **Incompressibility** — you cannot name a scalar without materializing it in the result, cannot
   pass tables as arguments (with rare recent exceptions), cannot abbreviate foreign-key joins,
   cannot abstract over column names or types. His worked example: adding one column to a
   "most highly paid employee" query requires restructuring half of it.
3. **Non-porousness** — extension mechanisms are non-standard across databases; queries are
   submitted as unstructured text with no standardized plan format or metadata encoding, *"making
   it harder than necessary to build any kind of tooling outside of the database."*

On complexity drag: SQL:2016 Part 2 alone is 1,732 pages of 9 parts, with 411 instances of
implementation-defined behaviour including basic arithmetic; *"No current version of any database
claims full conformance to Core SQL:2016."* His implementation-burden datum is striking and directly
relevant to this project: **Materialize's SQL parsing alone required ~27 kloc, more than the entirety
of differential dataflow (~16 kloc).** He argues this is how academic research on incremental
maintenance, parallel execution and provenance dies between toy demo and shippable system.

**What he says a replacement needs** (this is the part the requirement list is drawing on):

- *Expressiveness*: everything is an expression; few keywords, most things stdlib functions; first-class
  functions, relations, sum types, true recursion; unified scalar/table type system.
- *Compressibility*: functions taking relations and functions as arguments; polymorphism over
  relations without fixed schemas; column names, orderings, collations and window specifications as
  **first-class values**; compact foreign-key join syntax.
- *Porousness*: a **WebAssembly-based extension system** with standardized calling conventions;
  **exposed APIs for plans and hints, in both binary and human-readable encodings**; ergonomic
  nested-structure returns; embeddable parsing/planning/compilation libraries.
- *Simplicity*: simple denotational semantics for the core; a spec that completely specifies type
  inference and error semantics; *"it should be possible for an experienced engineer to throw
  together a slow but correct interpreter in a week or two."*

He closes on Stonebraker: *"All the annoying features of the language have endured to this day.
SQL will be the COBOL of 2020."*

**Three notes for this thesis.** (1) Brandon's "plans and hints as exposed APIs" is the same
architectural move as Halide's schedule language — corroboration for §3.3. (2) His WASM extension
recommendation matches the thesis's UDF tier. (3) His "interpreter in a week or two" simplicity
criterion is a *hard* constraint that argues against the requirement list's maximalism: a language
meeting all 13 clusters cannot have a one-week reference interpreter.

**One caution on relying on Brandon here:** "Against SQL" is a critique of *SQL the language*, not a
brief for imperative plan control. He asks for plans to be *inspectable and expressible*, not for the
optimizer to be overridden inline. The requirement-list phrasing "MORE explicit control to avoid
pathological plans" is a stronger claim than Brandon makes.

---

## 5. Per-language detail

### 5.1 Julia — the reality check

The most important entry, because Julia is the controlled experiment for the first half of the list.

**Does it have a garbage collector? Yes.** Per the manual: a **non-moving generational** collector
("Objects are not relocated in memory during garbage collection"; "Younger objects are collected
more frequently than older ones"), **parallel and concurrent** ("The GC can use multiple threads and
run concurrently with your program"), and **mostly precise** — precise for pure Julia code, with
"conservative scanning APIs for users calling Julia from C." Small objects (currently ≤ 2032 bytes)
go through a per-thread pool allocator; larger ones through system `malloc`. `GC.enable(false)`
exists but the manual warns it "can lead to memory exhaustion." As of early 2026 the allocator is
being overhauled, with a proposal to switch to **mimalloc** as the primary GC-object allocator —
motivated partly by Windows pathologies where freeing 8 GB could hang for up to two minutes.

**Does it have a runtime? Yes, a large one** — `libjulia`, the type system, the JIT, the dispatch
machinery, and the GC.

**Does it compile AOT to standalone binaries? Partially, experimentally, and only by giving up
dynamism.** Current state as of Julia 1.12 (stable 1.12.4; 1.13 in beta; 1.14-dev):

- `juliac` + `--trim` "attempts to slice out unused routines from the standard library, unneeded
  parts of the Julia runtime, metadata, and code from the user's program that it can determine is
  unreachable."
- LWN's assessment: the binaries "are not exactly small, not exactly standalone, and severely
  limited in scope." Hello-world = a 1.7 MB binary **plus 91 MB of library files**; the 93 MB
  bundle is what you must ship. Compilation takes about a minute; startup is then instant.
- **"The main limitation is the prohibition of dynamic dispatch."** Most packages therefore fail,
  because they contain dynamic dispatch in non-performance-critical paths.
- **"Programs cannot read from files or from the terminal; the only way to provide input is through
  command-line arguments."**
- LWN's characterization: *"really a kind of proof of concept: a demonstration of how things will
  eventually work."*
- Independent corroboration (ACAT 2025 / arXiv 2026, `JetReconstruction.jl` HEP case study): binary
  ≈ **300 MB** vs 8 MB for the FastJet C++ equivalent; to attempt `--trim=safe` the authors had to
  remove try/catch error handling, remove logging, remove CPU-feature queries (incompatible with
  `LoopVectorization`), and fix type instabilities — and *"two errors remain that prevent a trimmed
  build."* Conclusion: *"don't yet have a solid production solution."*
- Ecosystem-level blocker as of early 2026: "type stability — juliac requires types to be known at
  compile time" conflicts with TOML, CSV and DataFrames, which construct types dynamically.
- Historical alternatives: `PackageCompiler.jl` (sysimages — fast startup, but ships the whole
  runtime, ~GB), `StaticCompiler.jl` / `StaticTools.jl` (genuinely small static binaries, but only
  for a GC-free, allocation-free subset with `llvmcall` for everything else).

**Is it memory-safe in the Rust sense? No.** Julia core developers are direct about this. Mosè
Giordano: Julia "should also be memory safe (apart from bugs in the implementation of internal
functions)" but "doesn't guarantee safety in the presence of data-races." Jameson Nash: *"Unlike
Rust, there are not strict guardrails against unsafe mutation, but it is generally discouraged
style-wise."* Concretely: `@inbounds` disables bounds checks with no verification (there is an open
issue proposing to rename it `@unsafe_inbounds`), `unsafe_load`/`unsafe_store!`/`unsafe_wrap` exist,
`ccall` is unchecked, and there is **no data-race freedom**. Julia is memory-safe in the *Java*
sense (GC + bounds checks by default), not the *Rust* sense (no data races, no aliasing-mutation, no
UB without an `unsafe` block).

**Latency / time-to-first-plot.** Substantially improved since the 1.6–1.9 era (better invalidation
hygiene, `PrecompileTools.jl`, native-code caching in precompilation files, ongoing compiler
micro-optimization in 1.13-dev), but *not eliminated*: JIT latency is intrinsic to the design, and
the two escape hatches are sysimages (large) and AOT (restricted). I could not verify current
quantitative TTFX numbers for a standard `Plots.jl` workload on 1.12/1.13 — see [§7](#7-what-i-could-not-verify).

**Is it homoiconic with Lisp macros? Partially.** `Expr` objects, `quote`/`:()`, `@macro`
definitions, mostly-hygienic expansion with `esc` to break hygiene — yes. But the parser is fixed:
you cannot add new infix operators at new precedence levels. So Julia delivered *macros over infix
notation* but not *extensible* infix notation.

**What Julia did deliver, and it is a lot:** C-comparable speed on type-stable numeric code via
multiple dispatch + type inference + LLVM; genuinely excellent mathematical notation and linear
algebra; a real statistics ecosystem; multiple dispatch as a composition mechanism that is arguably
better than anything on this list at C7; and macros over infix syntax.

**What Julia proves for this thesis:** the manifesto's first cluster is *mostly* achievable — if you
accept a GC and a runtime. Adding "no GC, no runtime, memory-safe" to that list is not an increment;
it inverts the design. Julia's fourteen-year AOT effort is the empirical measurement of how much it
inverts it.

### 5.2 Rust

- **C1 ✓ C2 ✓ C3 ✓.** LLVM AOT to native code; no GC; a minimal runtime (panic machinery, stack
  probes, `std` init) that `#![no_std]` removes almost entirely. Memory safety and data-race freedom
  via ownership + borrow checking, with `unsafe` as the explicit escape hatch.
- **C5 ~.** `macro_rules!` is declarative and partially hygienic; procedural macros operate on
  `proc_macro::TokenStream`, which is deliberately *not* a stable AST — `syn` reconstructs one
  out-of-band and every proc-macro crate depends on it. Macros are invocable only inside
  `name!(...)` delimiters, so you cannot introduce free-standing infix syntax. Genuinely powerful,
  genuinely not homoiconic.
- **C6 ~.** Operator overloading via a fixed trait set (`Add`, `Mul`, …); no custom operators; array
  ergonomics are library-level (`ndarray`, `nalgebra`) and notably worse than Julia/MATLAB.
- **C11 ✓.** `std::simd` (portable SIMD, still nightly at last check — see §7), plus architecture
  intrinsics; and the empirical case is strong: **DataFusion, Polars and Arrow-rs are a complete
  vectorized columnar analytics stack in safe Rust.**
- **C12 ~.** Zero-copy within the program is idiomatic; zero-copy from untrusted external bytes hits
  the §3.5 boundary.
- **C4 ✗, C8 ✗.** No dynamism. Not a query language — it is the best available *host* for one.

Rust is the strongest single answer to C1–C3+C11+C12, i.e. the "systems half" of the list, and is
the obvious implementation substrate. It answers none of the "language half."

### 5.3 Mojo — current state (checked, September 2026)

- **v1.0 reached ~2026-08-12**; **fully open-sourced under Apache 2.0 with LLVM exceptions on
  2026-08-18** (compiler, tooling, infrastructure). Modular states they "aren't ready to take
  contributions to the compiler and tooling" yet, targeting end-2026.
- **No GC. ✓ C2.** Ownership model: "Every value has only one owner at a time"; "When the lifetime
  of the owner ends, Mojo destroys the value." Argument conventions: default (immutable reference),
  `mut`, `var` (ownership transfer, with `^` transfer sigil), `ref` (parametric mutability), plus
  `out`/`deinit`. Deterministic destruction, no collector.
- **C3 ~.** There *is* a checker: "argument exclusivity enforcement, preventing mutable references
  from aliasing other references," and "a lifetime checker that ensures that values are not
  destroyed when there are outstanding references." Analogous to Rust's borrow checker but operating
  at function-argument boundaries rather than through pervasive lifetime annotations. Whether this
  yields Rust-equivalent guarantees (particularly for data-race freedom across threads) I could not
  verify — see §7.
- **C4 ~ — and this is the decisive finding.** **Mojo is not a Python superset.** The docs describe
  "a Pythonic syntax" and *bidirectional interoperability*, not language equivalence: the plan is
  "to provide full compatibility with the Python ecosystem," described as aspirational. Python
  dynamism is obtained by **embedding CPython** — "the CPython runtime without modification for full
  compatibility with existing Python libraries," requiring Python 3.10–3.14. **So Mojo's dynamism
  arrives with CPython's reference-counting collector and cycle detector running in-process.** This
  is the cleanest available demonstration of §3.1(a2): the no-GC guarantee holds exactly over the
  Mojo-typed fragment and stops at the CPython boundary.
- **C6 ✓, C11 ✓.** SIMD is a first-class parametric type; MLIR-based, designed for heterogeneous
  hardware.
- **C5 ✗** (compile-time parameters and metaprogramming, but no macro system in the Lisp/Lean
  sense), **C8 ✗, C10 ✗, C13 ✗.**

Mojo is the most *deliberate* attempt at the C1+C2+C4+C6+C11 subset of this list. It delivers the
static half cleanly and buys the dynamic half from CPython.

### 5.4 Lean 4

- **C5 ✓ — best in class alongside Racket,** and the direct answer to tension (b). See §3.2.
- **C2 ~.** Reference counting, *not* tracing GC. Borrowing analysis elides RC ops; values are
  partitioned into thread-local (cheap non-atomic RC) and thread-shared. Opportunistic in-place
  update via `isShared`. **Cannot collect cycles** — by design, since "the verifiable fragment of
  Lean cannot create cyclic data." Compiles to **C** (not LLVM directly), with a documented
  extension point: "users can implement support for backends other than C by writing Lean programs
  that import `Lean.Compiler.IR`."
- **C1 ~.** Claims in the system-description paper: "preliminary experimental results demonstrate our
  new compiler produces competitive code that often outperforms the code generated by
  high-performance compilers such as `ocamlopt` and GHC." That is *fast for a functional language*,
  not *speed of C*.
- **C3 ✓** for the pure fragment; dependent types give far more than memory safety.
- **C4 ✗, C7 ✗, C8 ✗, C11 ✗, C13 ✗.** Positioned "primarily as a theorem prover with programming
  capabilities for proof automation — not as a general-purpose systems language."

**Why Lean 4 matters here disproportionately:** it is the single best demonstration that requirement
(b) is satisfiable, and its FBIP/Perceus lineage is also the best demonstration that (a1) is
satisfiable. For a thesis that wants *statically checkable* money conservation, Lean 4's
combination — dependent types, extensible notation, hygienic macros, RC not GC, C backend — is the
closest existing artifact to the Niles design brief, and is worth studying as a design source even
though it is not a candidate implementation substrate.

### 5.5 Racket

The gold standard for C5 and for language-oriented programming generally (Felleisen et al.,
"A Programmable Programming Language," CACM 2018; 2018 SIGPLAN Programming Languages Software Award).
`#lang` allows replacing the *reader*, so infix mathematical surface syntax and full hygienic macros
coexist by construction. It is the existence proof that "one project, many languages, one toolchain"
works.

It fails C1, C2, C3 and C11 outright — Racket CS runs on Chez Scheme with a generational GC and is
not AOT-to-machine-code in the required sense. **Racket is a source of architecture, not a
candidate.**

### 5.6 Terra (+ Lua)

Architecturally the most interesting entry on the list, and underrated.

Terra is "a low-level system programming language that is embedded in and meta-programmed by the
Lua programming language." Two layers with a clean seam:

- **Lua meta-layer** — dynamic, garbage-collected, runs at *compile* time. Terra functions are
  first-class Lua values that can be stored, passed and introspected.
- **Terra object-layer** — statically typed, manually memory-managed (C/C++-like), compiled through
  LLVM to machine code, **no GC**.
- Staging operators: escape `[]` (evaluate Lua at compile time, splice the result into Terra) and
  quotation (backtick — generate Terra expressions in Lua). The canonical demo is a Brainfuck
  compiler written in Lua that emits optimized native Terra code.

**This is exactly the structure that resolves tensions (a) and (c) simultaneously**: dynamism where
it is free (staging), none where it is expensive (execution); and it is the natural home for "DSLs
that compile to bare metal" and "specialized code per query shape." Liszt was ported to Terra on
precisely this basis.

Against it: **not memory-safe** (Terra is C-like), tiny ecosystem, low apparent maintenance activity,
no query layer. Its value here is as a *proof of the architecture*, not a substrate.

### 5.7 Zig

`comptime` is a genuinely elegant metaprogramming story — arbitrary compile-time execution of
ordinary Zig, with types as values — and gives most of what "DSLs compile to bare metal" wants
without a separate macro language. No GC, no hidden allocations, explicit allocators everywhere,
excellent C interop. **But Zig is not memory-safe** (no borrow checker; use-after-free and
double-free are runtime concerns addressed by debug allocators and safety-checked UB in Debug/
ReleaseSafe modes, not statically prevented). Fails C3 hard, which is a stated requirement.

### 5.8 Nim

`ARC` (deterministic RC with move semantics) and `ORC` (ARC + cycle collector, now the default) give
a middle path on C2 — deterministic destruction with a fallback for cycles. Nim's AST macros operate
on a typed AST and are genuinely powerful (C5 ~✓, better than Rust's token streams, short of Lean's
extensible syntax). Compiles via C/C++/JS. Memory safety is partial. Nim is the best evidence that
"RC + cycle collector" is a shippable answer to (a1), at the cost that ORC's cycle collector *is* a
tracing collector for the cyclic subgraph.

### 5.9 Vale, Koka, Austral — the memory-safety-without-GC frontier

- **Vale** — generational references: pointer + remembered generation, checked on deref. Measured
  **+10.84%** over unsafe vs **+25.29%** for RC. Permits mutable aliasing (observers, callbacks)
  that borrow checking rejects; with 64-bit generations, "the odds of an invalid access happening
  undetected is always 1/2^64." Regions and linear style can drive checks toward zero; a "first
  regions prototype" reportedly showed "no observable overhead when using linear style and regions."
  **Research maturity** — stack allocation and inline data were still unimplemented at the time of
  the cited writing.
- **Koka** — the Perceus reference implementation (PLDI 2021). Garbage-free RC + reuse analysis +
  FBIP. Requires explicit control flow (all effects compiled into a core language with explicit
  propagation) and assumes acyclicity. Within 10% of `std::map` on RB-trees.
- **Austral** — linear types enforcing exactly-once use; a deliberately minimal language with
  compile-time resource safety and no GC. Very strong on C2/C3/C12, essentially nothing else.

Collectively these define the honest answer to (a1): **automatic memory management without tracing
GC is a solved research problem with three working strategies (RC+reuse, RC+cycle-collector,
generational references), each with a documented cost and a documented cycle-handling caveat.**

### 5.10 Futhark, Halide, Chapel — the array/schedule axis

- **Futhark** — "statically typed, data-parallel, purely functional array language" targeting GPU
  (CUDA/HIP/OpenCL) and multicore CPU. Uniqueness types allow in-place array update while preserving
  purity. **Explicitly not general-purpose:** "Futhark is not intended to replace existing
  general-purpose languages"; intended for "relatively small but compute-intensive parts of an
  application"; not standalone (generates C/Python modules for FFI); described as "an ongoing
  research project." Excellent C6/C11; disqualified on C4/C7/C8.
- **Halide** — the schedule-language precedent, discussed at length in §3.3. Also relevant as an
  example of a DSL that compiles to bare metal with per-shape specialization (C5+C11).
- **Chapel** — productive parallel language with first-class multidimensional domains/arrays,
  locality control (`on`/`Locales`), and a real distributed story. **Has a GC** (for classes;
  `owned`/`shared`/`borrowed` provide Rust-ish alternatives). Best-in-class C6 + distributed
  parallelism; fails C2.

### 5.11 K / q / kdb+

The most interesting entry for C8+C11+C6 combined, and consistently under-cited in this discourse.
A single terse array language that *is* the query language, the analytics language and the
programming language, over a columnar store, with genuinely exceptional performance on time-series
workloads and a very small binary. Memory management is **reference counting**, with a `.Q.gc[]`
operator to return freed pages to the OS — so C2 is ~, not ✓. Fails C3 (not memory-safe in any
strong sense), C5 (no macro system), C7 (string processing and glue are weak), and readability is
famously hostile to "obvious, familiar mathematical notation." But as an existence proof that
**one language can be simultaneously an array language, a dataframe language and a query language
with set-at-a-time semantics and vectorized execution**, kdb+/q is the strongest data point on the
board, and deserves more weight in the thesis's related-work than it usually gets.

### 5.12 The deductive / Datalog family — best answer to C10

- **Soufflé** — "synthesizes a native parallel C++ program from a logic specification." This is
  literally C10 + C1 + C5 (a DSL compiling to bare metal, specialized per program shape). Used at
  scale for static program analysis over millions of LoC. Aggregates, records, ADTs, user-defined
  functors, choice domains, subsumption. **Not general-purpose; no transactions; no incrementality;
  no dynamism.** The single strongest artifact for "recursive and graph queries far more efficiently
  than SQL."
- **DDlog / Differential Datalog** (VMware Research) — incremental Datalog compiled to Rust over
  differential dataflow. **Archived** (`vmware-archive/differential-datalog`). Historically the best
  demonstration of incremental deductive computation; no longer maintained.
- **DBSP / Feldera** — the live successor line and the one directly relevant to this thesis. Budiu
  et al., "DBSP: Automatic Incremental View Maintenance for Rich Query Languages," PVLDB 16 (2023),
  with a SIGMOD Record 2024 version and a VLDB Journal 2025 extension; Feldera is the open-source
  engine. This is the substrate the thesis already names, and it is the correct choice: it is the
  only one of these with a *proved* incrementalization theory rather than an implementation.
- **Datafrog** — a lightweight embedded Datalog engine in Rust (used in Polonius); a library, not a
  language.
- **Flix** — the most interesting *language-design* entry: first-class Datalog constraints embedded
  in an effect-oriented functional/imperative language, with algebraic effects and handlers, traits,
  higher-kinded types, region-based local mutation, and a whole-program optimizing compiler
  (monomorphization, inlining, tree shaking). **Compiles to JVM bytecode and is garbage-collected** —
  so it fails C1/C2 — but "Programming with First-Class Datalog Constraints" (OOPSLA 2020) and
  "Flix: A Design for Language-Integrated Datalog" (PACMPL 2025) are the right prior art for
  *how to put Datalog inside a general-purpose language without impedance mismatch*, which is C10
  stated precisely.

### 5.13 The SQL-alternative family — and why none of them replace SQL

A structural point that applies to the whole group: **PRQL, Malloy, Logica and (in prototype) SaneQL
all compile *to* SQL.** They are front-ends. They improve C8's *ergonomics* — composability, pipelined
left-to-right reading order, functions over relations — while inheriting the target engine's
optimizer wholesale. That means they contribute **nothing** to C9 (plan control), C11 (execution
model) or C13, and they are not general-purpose languages. They are useful evidence about *syntax*,
not about *systems*.

- **PRQL** — pipelined relational syntax; compiles to SQL. Good evidence for composability-by-piping.
- **Malloy** (Google) — semantic modelling + nested/hierarchical results as a first-class output
  shape; compiles to SQL. Directly addresses Brandon's "ergonomic nested-structure returns."
- **Logica** (Google) — Datalog-flavoured; compiles to SQL/BigQuery. C10-flavoured syntax over a
  non-recursive engine.
- **SaneQL** (Neumann & Leis, CIDR 2024) — the most rigorous of the group, and by the Umbra authors.
  UFCS pipelining (`nation.filter(...).join(...).group(...)`), everything an expression, scalar
  *and table* parameters and unevaluated expressions as first-class values, minimal keywords (against
  SQL:2023's 409 reserved words). Empirical grounding: 130,998 real student queries, 38% erroring.
  Prototype translates to SQL. Explicitly a **query language only**, with future work noted on host-
  language embedding and on **using the language to represent physical plans with operator hints**.
- **EdgeQL** (EdgeDB/Gel) — genuinely more than a front-end: object-relational schema language,
  composable nested-shape queries, first-class links. Still engine-coupled; not general-purpose.
- **Morel** — Standard ML extended with relational algebra; the cleanest demonstration that
  functional composition and relational query can share one type system (addressing C8's "composes
  clean functional code"). Research-scale.
- **Rel** (RelationalAI; Aref, Guagliardo, Kastrinis, Libkin et al., arXiv 2504.10323, 2025) — the
  most ambitious: explicitly aims to escape the "sublanguage" paradigm and be a *general-purpose*
  language in which the entire application is relational, eliminating impedance mismatch. Datalog
  foundations extended with first-order formulas and aggregation; everything is a relation
  (functions included); integrity constraints first-class; and it makes exactly Brandon's argument
  against SQL's growth (invoking Steele's "small core plus libraries" principle against SQL's 4000+
  spec pages). **Rel is the closest existing artifact to the "replace SQL with a general-purpose
  relational language" half of the requirement list.** Its compilation target, execution model,
  memory management and plan-control story I could not verify from the paper — see §7.
- **dida / imp** (Jamie Brandon) — his own experiments: `imp` is a relational-programming language
  exploration, `dida` a differential-dataflow reimplementation in Zig. Both are research artifacts
  and, notably, Brandon's own follow-through on "Against SQL" did **not** produce a shipped
  replacement — which is itself evidence about difficulty.

### 5.14 Briefly: the rest

- **Common Lisp / SBCL** — C5 ✓ (unhygienic but total, including reader macros), C4 ✓, C7 ✓, C1 ~
  (SBCL is genuinely fast for a dynamic language, not C-fast), C2 ✗ (generational GC), C6 ✗
  (s-expressions are the opposite of "familiar mathematical notation" — this is precisely tension (b)
  in its original form).
- **Clojure** — C5 ✓, C4 ✓; JVM GC; STM (refs) gives in-memory transactions (a weak relative of
  C13); Datomic/Datalog connection is the C10 link. Fails C1/C2/C11.
- **Scheme** — see Racket; R7RS-large plus Chez gives fast native compilation but with GC.
- **ATS** — dependent types + linear types over C; extraordinary C2/C3/C12 guarantees, brutal
  ergonomics, no ecosystem.
- **APL / BQN / J** — the origin of C6's "explicit composable analytics operations" and the direct
  ancestors of K/q. Interpreted, GC'd; BQN and Dyalog have compilation stories but not to the C1/C2
  bar.
- **R, MATLAB** — the *referents* of C7 and C6 respectively, not candidates: interpreted, GC'd,
  copy-on-write semantics that violate C12 by design.
- **Gleam, Roc, Carp** — Gleam is BEAM/JS-targeted with a GC (fails C1/C2). Roc targets
  no-tracing-GC via RC+opportunistic-mutation (a Perceus relative) and is worth watching for (a1),
  but is pre-1.0 and platform-oriented. Carp is a Lisp with static memory management via ownership —
  interesting as an existence proof of "Lisp macros + no GC" (relevant to (a1)+(b)), tiny.

---

## 6. What this means for the Niles design

Compressed, because it follows from §3.

1. **Delete C13 from the language requirements.** Strict serializability is an engine property.
   Keep it as a *typed effect* — the consistency-effect calculus is the correct home for it and is
   already Contribution 4.
2. **Restate C2** as *"no tracing GC; deterministic, epoch-scoped reclamation; no stop-the-world
   pauses."* Then it is achievable, and the epoch-ordered ledger doubles as the reclamation domain —
   arguably a bonus result worth naming in the thesis.
3. **Restate C4.** Full runtime dynamism is incompatible with C2 *and* with the static checkability
   Contribution 4 requires. Adopt the **Terra/Zig model**: a dynamic, expressive *compile-time*
   meta-stage (which is where "specialized code per query shape" lives anyway), with a static,
   GC-free execution layer. This gets the felt benefit of dynamism at the point where the thesis
   actually needs it — query-shape specialization — at zero runtime cost.
4. **Delete "homoiconic."** Adopt Lean 4's model: extensible mixfix notation over a typed, quotable
   `Syntax` tree with hygiene. This is the single highest-leverage borrowing available, and it
   directly serves the Appendix B goal of "novel syntax/semantics/primitives for the new concepts."
5. **Split C8 from C9 into two languages over one IR** (Halide model). The differentiating research
   contribution available here is **verified schedules**: a schedule/plan term proven to preserve the
   algorithm's denotation. That answers five of PostgreSQL's six objections to hints and is a
   defensible novel claim.
6. **C1+C11 is safe.** Plan on one typed IR with two backends (pre-enumerated vectorized primitives
   à la InkFuse + fused compiled pipelines), with adaptive tier selection à la Umbra. Budget the
   second backend explicitly; and note Kersten's finding that **SIMD buys ~1.4× on real OLAP, not
   8×** — do not over-claim in the evaluation chapter.
7. **C12 boundary is drawable.** Zero-copy over the engine's own hash-verified sealed epochs is
   sound; zero-copy over client bytes is not. State the pick-two explicitly in the architecture.
8. **The related-work chapter should add**: Halide (schedule separation), Lean 4 / Racket (extensible
   syntax + hygiene), Perceus/Koka + Vale + Nim ORC (memory management without tracing GC),
   Soufflé + Flix (deductive), InkFuse + Umbra + Photon (execution paradigm unification), SaneQL +
   Rel + Morel (SQL replacement), and kdb+/q (the one-language array+query existence proof).

---

## 7. What I could not verify

Stated explicitly, as requested.

1. **Exact verbatim wording of the Julia post.** Direct `curl` to `julialang.org` was blocked by the
   egress proxy (HTTP 403 on CONNECT). The quotations in §4.1 come via the fetch-and-summarize path,
   which returned them as quoted strings and which I cross-checked against multiple independent
   secondary reproductions. I am confident in the substance and in the phrases in quotation marks,
   but **I did not obtain the raw HTML** and cannot certify punctuation or clause ordering.
2. **Julia TTFX quantitative status on 1.12/1.13.** I confirmed qualitatively that latency work
   continues (compiler micro-optimizations in 1.13-dev; `--trim` error-message improvements) but
   found **no current benchmark numbers** for a standard `Plots.jl` first-plot on 1.12/1.13. Do not
   cite a figure.
3. **Whether `juliac` binaries still link `libjulia` and retain the GC.** The ACAT/arXiv case study
   "does not explicitly state whether the binary retains dependencies on libjulia or the garbage
   collector." The 91 MB companion library directory and the ~300 MB binary strongly imply the
   runtime is bundled, and `--trim` is described as slicing out *unneeded parts of* the runtime
   (implying the needed parts remain), but **I did not find an authoritative statement** that GC
   code is or is not present in a trimmed binary. This is worth resolving directly if the thesis
   cites Julia's AOT story as evidence.
4. **The arXiv HTML for 2607.24321** returned HTTP 429 on three attempts. My account of that paper
   comes from the corresponding **ACAT 2025 slide deck** (indico.cern.ch) by the same authors and
   from search-result metadata. Treat the ~300 MB figure and the "two errors remain" quote as
   sourced to the slides, not the paper.
5. **Whether Mojo's exclusivity checking yields Rust-equivalent guarantees**, particularly data-race
   freedom across threads. The docs describe argument exclusivity and a lifetime checker but I found
   no statement of a formal safety claim, no `unsafe`-equivalent boundary documented, and no
   soundness argument. **Do not assert Mojo is memory-safe in the Rust sense.**
6. **Mojo's compilation target details** (MLIR→LLVM specifics, AOT vs JIT defaults, whether the
   no-GC guarantee is documented anywhere as a guarantee rather than an implementation property) —
   the ownership documentation "contains no information about garbage collection"; I inferred
   "no GC" from deterministic ownership-based destruction. Reasonable, but it is an inference.
7. **Rust `std::simd` stabilization status as of September 2026.** I did not check; it was nightly
   at my last reliable knowledge. Verify before citing.
8. **Rel's implementation** — compilation target, execution model, memory management, and whether it
   exposes any plan control. The arXiv paper "does not specify compilation targets or execution
   mechanisms in the provided excerpt." Its row in the scoring table carries `?` accordingly. I also
   did **not** verify the commonly repeated claim that RelationalAI's engine is implemented in Julia;
   I have deliberately omitted it rather than assert it.
9. **The optimizer-hints survey** (ResearchGate 395112547) returned HTTP 429; I could not confirm its
   authors, venue or date. §3.3 therefore rests on the PostgreSQL wiki's stated position (primary,
   verified) rather than on the survey.
10. **Vale's current maturity.** The generational-references article is undated in my fetch; stack
    allocation and inline data were unimplemented at the time of writing and may have shipped since.
    Treat the +10.84% / +25.29% figures as from that article's own microbenchmarks, which explicitly
    "didn't compare against C++ or Rust."
11. **Terra's current maintenance status** — the site "doesn't specify" and I did not check commit
    activity. Do not describe it as actively maintained.
12. **Current status of Morel, Logica, dida/imp, Datafrog and DDlog's successors.** Only DDlog's
    archived status (`vmware-archive/`) was directly confirmed. The others were not checked for
    2026 activity.
13. **Chapel, ATS, Austral, Carp, Roc, Gleam, BQN** were assessed from established knowledge without
    2026 verification. Their table rows are lower-confidence than the top ten.
14. **Kersten et al. quotations** come from the PVLDB PDF via fetch-and-summarize. The conclusions
    are stated confidently in the source and match the paper's well-known reputation, but I did not
    read the full PDF text directly.

---

## 8. Sources

**Attribution**
- Bezanson, Karpinski, Shah, Edelman. "Why We Created Julia." <https://julialang.org/blog/2012/02/why-we-created-julia/>
- Brandon, Jamie. "Against SQL." 2021-07-09. <https://www.scattered-thoughts.net/writing/against-sql/>
- Brandon, Jamie. <https://www.scattered-thoughts.net/> · imp: <https://github.com/jamii/imp>

**Julia**
- LWN, "Julia 1.12 brings progress on standalone binaries and more." <https://lwn.net/Articles/1044280/>
- Julia manual, Memory Management and Garbage Collection. <https://docs.julialang.org/en/v1/manual/memory-management/>
- "This Month in Julia World (January 2026)." <https://julialang.org/blog/2026/02/this-month-in-julia-world/>
- Fila et al., "Static compilation of Julia packages… JetReconstruction.jl." <https://arxiv.org/html/2607.24321v1> · slides: <https://indico.cern.ch/event/1488410/contributions/6562804/attachments/3130224/5553086/2025-09-08_acat2025_static-compilation-julia_fila.pdf>
- "Is Julia safe?" Julia Discourse. <https://discourse.julialang.org/t/is-julia-safe/121006>
- StaticTools.jl <https://github.com/brenhinkeller/StaticTools.jl> · StaticCompiler.jl <https://github.com/tshort/StaticCompiler.jl>
- Blackburn et al., "Reconsidering Garbage Collection in Julia." ISMM 2025. <https://www.steveblackburn.org/pubs/papers/julia-ismm-2025.pdf>

**Query execution**
- Kersten, Leis, Kemper, Neumann, Pavlo, Boncz. "Everything You Always Wanted to Know About Compiled and Vectorized Queries But Were Afraid to Ask." PVLDB 11(13), 2018. <https://www.vldb.org/pvldb/vol11/p2209-kersten.pdf>
- Behm et al. "Photon: A Fast Query Engine for Lakehouse Systems." SIGMOD 2022. <https://people.eecs.berkeley.edu/~matei/papers/2022/sigmod_photon.pdf>
- Wagner, Kohn, Boncz, Leis. "Incremental Fusion: Unifying Compiled and Vectorized Query Execution." <https://www.cs.cit.tum.de/fileadmin/w00cfj/dis/papers/inkfuse.pdf>
- Kersten, Neumann et al. "Tidy Tuples and Flying Start… Umbra." VLDB Journal 2021. <https://db.in.tum.de/~kersten/Tidy%20Tuples%20and%20Flying%20Start%20Fast%20Compilation%20and%20Fast%20Execution%20of%20Relational%20Queries%20in%20Umbra.pdf>
- Leis & Neumann. "Database Query Compilation: Our Journey." HYTRADBOI 2025. <https://www.hytradboi.com/2025/slides/leis-neumann-compilation.pdf>
- PostgreSQL wiki, OptimizerHintsDiscussion. <https://wiki.postgresql.org/wiki/OptimizerHintsDiscussion>

**Languages**
- Ragan-Kelley et al. "Halide." PLDI 2013. <https://people.csail.mit.edu/jrk/halide-pldi13.pdf>
- Ullrich & de Moura. "Beyond Notations: Hygienic Macro Expansion for Theorem Proving Languages." ITP 2020. <https://pp.ipd.kit.edu/uploads/publikationen/ullrich20beyond.pdf>
- de Moura & Ullrich. "The Lean 4 Theorem Prover and Programming Language." <https://lean-lang.org/papers/lean4.pdf> · RC reference: <https://lean-lang.org/doc/reference/latest/Run-Time-Code/Reference-Counting/>
- Reinking, Xie, de Moura, Leijen. "Perceus: Garbage Free Reference Counting with Reuse." PLDI 2021. <https://xnning.github.io/papers/perceus.pdf>
- Ovadia. "Vale's Memory Safety Strategy: Generational References and Regions." <https://verdagon.dev/blog/generational-references>
- Nim, "Introduction to ARC/ORC in Nim." <https://nim-lang.org/blog/2020/10/15/introduction-to-arc-orc-in-nim.html> · <https://nim-lang.org/docs/mm.html>
- Modular, "Mojo is now open source" (2026-08-18). <https://www.modular.com/blog/mojo-open-source> · ownership: <https://mojolang.org/docs/manual/values/ownership/> · Python interop: <https://mojolang.org/docs/manual/python/>
- Terra. <https://terralang.org/>
- Futhark. <https://futhark-lang.org/> · Henriksen et al., PLDI 2017: <https://futhark-lang.org/publications/pldi17.pdf>
- Flix. <https://flix.dev/> · "Programming with First-Class Datalog Constraints," OOPSLA 2020: <https://dl.acm.org/doi/pdf/10.1145/3428193> · "Flix: A Design for Language-Integrated Datalog," PACMPL 2025: <https://dl.acm.org/doi/10.1145/3763126>
- Soufflé. <https://souffle-lang.github.io/> · Scholz et al., CC 2016: <https://souffle-lang.github.io/pdf/cc.pdf>
- Rust procedural macros reference. <https://doc.rust-lang.org/reference/procedural-macros.html>
- kdb+/q memory management. <https://code.kx.com/q4m3/13_Commands_and_System_Variables/>
- Felleisen et al. "A Programmable Programming Language." CACM 2018.

**Query languages**
- Neumann & Leis. "A Critique of Modern SQL And A Proposal Towards A Simple and Expressive Query Language" (SaneQL). CIDR 2024. <https://www.cidrdb.org/cidr2024/papers/p48-neumann.pdf>
- Aref, Guagliardo, Kastrinis, Libkin et al. "Rel: A Programming Language for Relational Data." arXiv 2504.10323, 2025. <https://arxiv.org/html/2504.10323v1>
- Budiu et al. "DBSP: Automatic Incremental View Maintenance for Rich Query Languages." PVLDB 16, 2023. <https://docs.feldera.com/vldb23.pdf> · Feldera: <https://github.com/feldera/feldera>
- DDlog (archived). <https://github.com/vmware-archive/differential-datalog>
- PRQL. <https://prql-lang.org/>

**Consistency**
- Kingsbury. "Strong Serializability." Jepsen consistency models. <https://jepsen.io/consistency/models/strong-serializable>
- Bailis. "Linearizability versus Serializability." <https://www.bailis.org/blog/linearizability-versus-serializability/>
- Kingsbury. "Serializability, linearizability, and locality." <https://aphyr.com/posts/333-serializability-linearizability-and-locality>
