# Does it exist? And if not, what to build.

Two questions were asked. Both have clear answers, and the answers change what Niles and
Nilestream should try to be. This document gives the verdict, corrects the parts of the
requirement list that are incoherent rather than merely hard, and sets out the roadmap for
the language, the engine, and GBS.

The evidence is in two companion surveys, each built from primary sources:
`language-landscape.md` (~11k words) and `engine-landscape.md` (~60 KB, 40 references).

---

## Part I — The verdicts

### Does such a language exist?

**No — and three of the requirements are incoherent rather than unsolved.** Nothing scores
above roughly six of the thirteen requirement clusters.

First, an attribution, because it turns out to be the single most useful fact here. The
first half of the list is close to verbatim from **"Why We Created Julia"** (Bezanson,
Karpinski, Shah, Edelman, February 2012) — "the speed of C with the dynamism of Ruby…
homoiconic with true macros like Lisp, but with obvious familiar mathematical notation like
Matlab… usable for general programming as Python, as easy for statistics as R, as natural
for string processing as Perl, as powerful for linear algebra as Matlab, as good at gluing
programs together as the shell." They called their own list "greedy" and "ungracious."

**"No runtime, no garbage collector, memory-safe" is not in that post.** Those three are
the additions — and they are precisely the additions that make the list impossible. Julia
is the fourteen-year controlled experiment in whether the rest of the list is achievable
*with* a GC and a runtime, and the answer is largely yes. Remove them and the manifesto
half collapses.

The current AOT story is the evidence and it is unambiguous: Julia 1.12's `juliac --trim`
buys ahead-of-time compilation by **prohibiting dynamic dispatch** — it deletes the
dynamism to get the binary. LWN's assessment is that the binaries are "not exactly small,
not exactly standalone, and severely limited in scope"; hello-world is 1.7 MB plus 91 MB of
libraries, and "programs cannot read from files or from the terminal." A 2026 high-energy-
physics case study produced a ~300 MB binary against 8 MB for C++, had to strip try/catch,
logging and CPU-feature queries, and concluded there is still no solid production solution.

The three closest candidates and what each is missing:

| | Meets | Missing |
|---|---|---|
| **Julia** | The manifesto half: dynamism, notation, statistics, glue, near-C speed on type-stable code | A non-removable generational GC; a large runtime; not memory-safe in Rust's sense (the core developers are explicit that there is no data-race freedom, and `@inbounds` has no guardrails); no query layer |
| **Rust** | Native, no GC, memory-safe, vectorized, zero-copy | Dynamism; extensible infix notation. It *hosts* query engines (DataFusion, Polars) rather than being one |
| **Mojo** (v1.0, open-sourced Aug 2026) | Real ownership and exclusivity checking, deterministic destruction, no GC of its own | **It is not a Python superset.** Its dynamism comes from embedding CPython unmodified — which re-imports CPython's reference counting and cycle detector into the process. The GC did not disappear; it moved |
| **Lean 4** | The best existing answer to two tensions at once: extensible mixfix syntax *and* hygienic macros over a typed `Syntax` tree, with reference counting rather than tracing GC, compiling to C | Not a performance-engineering language; no vectorization; no query layer; cannot collect cycles |

### Does such an engine exist?

**No. Nothing in the survey scores first-class on even three of the four workload classes** —
and one requirement, *"the optimizer should not produce a bad join order or wrong indexes,"*
is refuted by three independent theorems rather than being an engineering gap.

| | Closest on | Missing |
|---|---|---|
| **Umbra / CedarDB** | The only genuine **single-format** HTAP engine: a B⁺-tree with PAX-layout leaf pages, so there is no rowstore/columnstore seam, one MVCC scheme, one compiler. 3.0× geomean over HyPer on the Join Order Benchmark; buffer-manager overhead under 6% against pure in-memory. Ships **worst-case-optimal joins** with a hybrid optimizer — 4.5–6.9× on a 4-clique against EmptyHeaded, **with zero regression on TPC-H and JOB** | Streaming and incremental view maintenance entirely — not even on the roadmap. Graph is recursive CTEs plus WCOJ; SQL/PGQ only "planned". Single node. Free tier caps at 64 GB with no HA or failover shipped |
| **Materialize** | **Strict serializability is its default isolation level**, over incrementally maintained views, with a published ladder (Strict Serializable → Serializable → Bounded Staleness). This is close prior art for an epoch-based consistency ladder and the thesis must engage it directly | General OLTP writes; columnar analytics; graph. Its views are **fully** materialized |
| **Noria / ReadySet** | The only system pairing partial materialization with upqueries | **Explicitly eventually consistent** — "Noria operators and the contents of its external views are eventually-consistent." No OLAP, no graph |
| **Kùzu** | Best columnar graph DBMS in the literature: factorisation, ASP-join, multiway WCOJ, >10× over DuckDB and Umbra on selective multi-hop | It is **dead** — archived by its own company in October 2025. Single-writer, no OLAP or streaming. A cautionary data point: technical excellence was not the binding survival constraint |

---

## Part II — What is impossible, what is open, and what is already solved

This is the part that should change the design, so it is separated from the survey.

### Provably impossible — stop asking for these

**"Never a bad plan."** Three independent barriers, and the second is decisive:

1. **Charikar et al. (PODS 2000):** any estimator examining *n* of *N* rows incurs
   **Ω(√(N/n)) ratio error on some input**. Harmouch measured that 1% relative error
   requires sampling over 90% of the data.
2. **Plan Bouquets (Dutt & Haritsa, SIGMOD 2014):** even if compile-time estimation is
   abandoned entirely and selectivities are *discovered by executing*, **no deterministic
   online algorithm achieves maximum suboptimality below 4×, with even one unknown
   selectivity.** The realistic multi-dimensional bound is around 4ρ (≈40×). And the
   technique pays for that with repeated partial executions and cannot handle updates.
3. Learned optimizers do not rescue it: they degrade under updates, cost 5–20 ms of
   inference, violate monotonicity and consistency, and industry has not adopted them.

**Strict serializability + non-blocking + one-round reads + conflicting writes** — the SNOW
theorem (OSDI 2016). This binds directly on any design serving fast reads from derived
views while writes land.

**Linearizability at zero latency cost** — Attiya–Welch: roughly *u*/4 on reads and *u*/2 on
writes under clock uncertainty *u*.

**Ruby-level dynamism with no runtime.** Not unsolved — self-cancelling.

### Open, and this is where the contributions are

**Partial materialisation *with* strict serializability.** Noria took partial state and gave
up consistency. Materialize took strict serializability and gave up partiality. Nobody has
both — and the survey **found no existing theorem bounding the cost of consistency under
eviction.**

That is an independent confirmation, from a survey that was instructed to be adversarial,
that **Contribution 2 (the Eviction-Consistency Frontier Theorem) addresses a genuinely open
frontier.** It is the strongest external evidence the thesis has yet received for its
central claim, and it arrived from a search designed to find prior art rather than to
confirm the premise.

Two others: all four workload classes in one engine (no theorem forbids it; the pieces
exist separately), and distributed strict serializability over single-copy columnar storage
(every distributed HTAP system today pays with a second physical copy; no proof that is
necessary).

### Already solved — adopt, do not reinvent

* **Seamless HTAP.** PAX-in-B-tree, since 2020, commercially available since May 2025.
* **Graph speed inside a relational engine.** Worst-case-optimal joins, VLDB 2020, with no
  regression on conventional workloads. **A separate graph engine is not needed** — which
  removes a whole subsystem from the Nilestream roadmap. But note the size of the prize:
  Freitag's own hybrid optimizer chose WCOJ **zero times out of 923 joins** across TPC-H and
  JOB. It is a *parity* requirement for graph-shaped queries, not a differentiator on
  relational ones.
* **Serializability at snapshot-isolation cost** on a single node — precision-locking MVCC,
  SIGMOD 2015.
* **Vectorised versus compiled is not a dichotomy.** Kersten et al. (VLDB 2018) found them
  "quite similar in OLAP workloads"; InkFuse unifies them via suboperators; Umbra tiers
  adaptively. Two numbers the evaluation chapter must not overclaim. **SIMD buys about 1.4× on real
  TPC-H Q6 and ~1.1× on Q3/Q9 — not 8×**; the 8.4× figure is a microbenchmark. And
  **compiled versus vectorized is a wash**: on Q6 the two are a dead tie at 15 ms, and the
  paper's whole spread is 0.66×–1.93×, which its authors describe as "not large
  differences." The 1.4× is the AVX-512 gain, not the compilation gain, and conflating them
  would be citing the right number for the wrong claim.
* **The highest-leverage optimizer fix is not better estimation.** Leis et al. cut queries
  running more than 2× slower from **38% to under 4%** by disabling risky nested loops and
  enabling runtime hash-table resizing. **Restricting the plan space beats improving the
  estimates**, and it is cheap.

### Three requirements that must be restated to be achievable

**1. "Strict serializability" is not a language property.** It is a property of a
concurrency-control protocol over a durable store — Papadimitriou's serializability plus
Herlihy–Wing real-time order. The mechanisms that produce it (a sequencer, a WAL, a
coordinator) *are* a runtime. A language can express transactions, type isolation levels as
effects, and statically reject a program that reads below the rung it promises. It cannot
provide the guarantee.

**This is a correction the thesis owes itself.** Contribution 4's soundness theorem should
say what it actually proves — that a well-typed program cannot *request* an unsound
combination — and the guarantee should be attributed to the engine, where it lives.

**2. "No garbage collector" should be "no *tracing* garbage collector."** Partially stateful
state with eviction and upqueries has extents decided at runtime by the read workload;
there is no lexical scope to tie them to. Real engines use epoch-based reclamation, which
*is* a collector. Restated as **deterministic, epoch-scoped reclamation**, it is achievable —
and the epoch-ordered ledger is already exactly the right reclamation domain. That is a
small bonus result the thesis can claim: the visibility timeline and the reclamation domain
are the same object.

**3. "Compiles to machine code without an interpreter" contradicts "replaces
PostgreSQL/MySQL."** InkFuse measures it: at 100 MB, an interpreter finishes *all* queries
in under 20 ms while compilation needs over 40 ms just to compile. Umbra's answer is an
interpreter — bytecode first, LLVM on promotion. Photon rejected code generation partly
*because* it obstructs the runtime adaptivity that is the main defence against bad plans.
The requirement as written forbids the known solution.

### And one tension between the requirement list and the thesis itself

Ruby-level dynamism destroys precisely the static type information Contribution 4 depends
on. Conservation of money, currency non-mismatch and authorised overdraw are checked
*because* types are known before the program runs. You cannot have both.

The resolution is the Terra/Zig `comptime` model: **put the dynamism in a compile-time
meta-stage** — which is where "specialised code per query shape" belongs anyway — and keep
the execution layer static and free of tracing collection. This is not a compromise; it is
where every system that has both properties has landed.

---

## Part III — The roadmap

### Niles: five reframed goals

| Requirement, as asked | Achievable form | Model to follow |
|---|---|---|
| Dynamism of Ruby | A `comptime` meta-stage; static execution tier | Terra, Zig |
| Homoiconic with Lisp macros *and* infix maths | Extensible **mixfix** surface syntax + hygienic procedural macros over a typed `Syntax` tree | Lean 4, Racket |
| Declarative *and* explicit plan control | **Algorithm/schedule separation** — and the novel part below | Halide (PLDI 2013) |
| No GC | No *tracing* GC; deterministic epoch-scoped reclamation | The ledger's own epochs |
| Strict serializability in the language | Isolation as an effect; the engine provides the guarantee | Correct the claim |

The one worth calling a contribution: **verified schedules.** Halide separates *what* to
compute from *how*, and Neumann & Leis independently reach for the same move in SaneQL —
"a version of the language with additional operator hints could also be used to represent
physical query plans." Brandon asks for the same thing in "Against SQL."

Nobody has made the schedule *verified* — a plan term carrying a proof that it preserves the
algorithm's denotation. PostgreSQL has refused query hints for twenty-five years, and every
one of its six stated objections reduces to *hints are unverified*. A verified schedule
eliminates five of the six. Nilestream is unusually well placed to do this, because **the IR
verifier is already in the trusted base** — the machinery exists and needs a new judgement,
not a new subsystem.

### Nilestream: four moves, three of them adoptions

1. **Adopt PAX-in-B-tree storage** (Umbra). Solves single-format HTAP. Do not invent storage.
2. **Adopt worst-case-optimal joins** (Freitag et al., VLDB 2020). This is the graph answer;
   it removes a graph subsystem from the roadmap and has zero measured regression.
3. **Adopt adaptive tiering** — bytecode interpreter first, compile on promotion. Required
   for the OLTP path to exist at all.
4. **Replace "no bad plan" with a bounded claim.** Two candidates, and the second is the one
   to make: *no plan worse than k× the oracle plan* (floor of 4, per Plan Bouquets), or
   **no plan containing an unbounded nested loop** — achievable, and empirically the single
   biggest win in the literature. `join_order.rs` should implement the plan-space
   restriction now, since it is cheap and the evidence is strong.

The four-workload claim then has an architecture behind it rather than an aspiration:
transactional is the ledger write path; analytical and streaming are both REVs over it
(they differ in rung, not in kind); graph is WCOJ over the same relational core. One
semantics, four workloads — which is the thesis's argument, stated as an engine roadmap.

### GBS: what the scope actually decomposes into

The full product scope — FX, derivatives, lending, trade finance, liquidity, securities,
asset management, ETFs, risk, portfolio management, trading, investment banking, distressed
portfolios, real estate, markets and securities services, wealth, trusts, philanthropy,
specialised financing, advisory and self-directed — decomposes into **seven mechanisms**
over one kernel. `ARCHITECTURE.md` has the full derivation and the coverage matrix; the
mechanisms are: balanced posting sets, contingent schedules, holds and commitments,
fractional participations, capability-gated state machines, position signals, and
rate-indexed valuation.

That decomposition is the falsifiable claim, and the build is its test: `tests/layering.rs`
fails if any product acquires a dependency below the mechanisms layer, so a product needing
a kernel change cannot be written without breaking the build.

### The newly added scope needs an honest separation

Matching engines, ultra-low-latency connectivity, and market-access infrastructure are **a
different regime from a ledger, and designing one system for both would be a mistake.**

An exchange matching engine runs single-threaded on a pinned core with no allocation in the
hot path, kernel-bypass networking, and a latency budget in hundreds of nanoseconds. A
durable ledger is `fsync`-bound and lives in milliseconds. These numbers differ by four
orders of magnitude; no single storage engine serves both, and a system claiming to would
be lying about one of them.

The correct architecture separates them and — usefully — the thesis's own model is the right
interface. **A matching engine's output is already a total order.** That is exactly what an
epoch is. So:

* **Tier 1, the matching engine:** deterministic, in-memory, no durability in the hot path,
  emitting a sequenced event stream. Rust, no allocation, pinned cores.
* **Tier 2, the ledger:** ingests that stream as pre-sequenced epochs. Because the order is
  already agreed, admission is a validation rather than a sequencing decision — which is the
  same observation Calvin and Aria make about deterministic execution, and which the thesis
  already cites.
* **Tier 3, the REVs:** positions, risk, P&L, regulatory roll-ups, each at its own rung.

Two further honesty notes on the newly named systems:

* **Loan IQ** (syndicated lending) and **Calypso** (capital markets and treasury) are within
  scope of the mechanism set — M4 and M2 respectively are what they are made of. These are
  credible targets.
* **Bloomberg is mostly a data business, not a backend.** Replacing its backend means
  replacing market-data ingestion, entitlement and distribution — which is a licensing and
  content problem before it is a technical one. The *analytics* and *portfolio* surfaces are
  addressable by M6 and M7; the data itself is not something a better engine obtains.

---

## What this document does not claim

The surveys list what could not be verified — fourteen items on the language side, a
comparable set on the engine side. Two are worth naming here because they bear on claims
above: whether trimmed `juliac` binaries still contain GC code is not authoritatively
stated anywhere found, so Julia's AOT story is cited from LWN and a case study rather than
from a definitive source; and Mojo's formal safety guarantees should not be asserted to be
memory-safe in Rust's sense without a primary source.

And the two reference videos could not be retrieved — YouTube returned 429 on every attempt.
Their titles and channels are confirmed; the content of the second one is not. §0 of
`ARCHITECTURE.md` records this rather than inventing a summary.
