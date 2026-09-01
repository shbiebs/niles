# Are the goals logical and consistent?

**Yes — substantially. This list is satisfiable, and the previous one was not.**

That is a different answer from the one given three weeks ago, and the reason is that the
list changed. Four requirements were dropped, and they were precisely the four that made the
earlier version impossible. What remains has three residual problems, all of them
restatements rather than contradictions, and one of them is a category error that costs
nothing to fix.

This document is the consistency analysis. `SPEC-LANGUAGE.md` and `SPEC-ENGINE.md` are the
specifications that follow from it; `ROADMAP.md` is how Niles and Nilestream get there.

---

## 1. What changed, and why it matters

| Previous list | Revised list | Effect |
|---|---|---|
| "the dynamism of Ruby" | **removed**, replaced by *"mathematically functional"* | **This is the change that makes the rest possible** |
| "homoiconic, with true macros like Lisp" | removed | Removes the s-expressions-versus-infix contradiction |
| "obvious familiar mathematical notation like Matlab" | narrowed to *"powerful for algebra"* | Removes the demand for *extensible* surface notation |
| "as easy for statistics as R, as natural for string processing as Perl, as good at gluing programs together as the shell" | removed | Removes three scope commitments that pull toward a general-purpose scripting language |

The first row is the whole story. The previous analysis concluded that **"dynamism of Ruby"
+ "no runtime" + "no garbage collector" is self-cancelling** — not unsolved, but
contradictory — and that Julia is the fourteen-year controlled experiment proving it: Julia
1.12's `juliac --trim` obtains ahead-of-time compilation by *prohibiting dynamic dispatch*.
It deletes the dynamism to get the binary.

Replacing dynamism with **"mathematically functional"** does not weaken the list. It
strengthens it, because a functional core is exactly what makes the other requirements
compose:

* **Referential transparency is what lets the optimizer "do more."** An expression with no
  side effects can be reordered, fused, memoised, or evaluated in parallel without proof
  obligations. This is the mechanism behind requirement L-8 ("a better way to express
  queries that lets the engine do more"), and it is unavailable in a language where a query
  fragment might mutate something.
* **Static types are what make deterministic reclamation possible.** Without dynamic
  dispatch on arbitrary runtime types, ownership is decidable, and "no tracing GC" stops
  being a wish.
* **A functional core is what makes the deductive and the relational layers the same
  thing.** Datalog is a functional language over relations; a `select` is a comprehension.
  Requiring both is a coherent demand only if the base language is functional.

And critically for this project: **Ruby-level dynamism would have destroyed Contribution 4.**
Conservation of money, currency non-mismatch and authorised overdraw are checkable *because*
types are known before the program runs. The previous list asked for the language to be
dynamic and for the type system to prove financial invariants statically. This one does not.

---

## 2. The revised requirements, assessed

Twenty-four requirements, numbered here and carried into the specifications with the same
numbers. **Verdict key:** ✅ consistent and achievable · ⚠️ achievable but needs restating ·
🔷 achievable and already solved elsewhere (adopt, do not invent).

### The language

| # | Requirement | Verdict | Note |
|---|---|---|---|
| L-1 | Speed of C | ✅ | Rust demonstrates it. Achieved by compiling to native code with no runtime dispatch on the hot path |
| L-2 | Mathematically functional | ✅ | And it is what makes L-8, L-10, L-14, L-19 mutually consistent |
| L-3 | Usable for general programming | ✅ | Already true of the imperative subset — `bootstrap/lexer.niles` is 450 lines of it |
| L-4 | Powerful for algebra | ✅ | Relational algebra is the core; linear algebra is a typed library over arrays, not a syntax demand |
| L-5 | Replaces SQL | ⚠️ | Achievable as *supersedes*, not as *proves-SQL-inadequate*. §3.1 |
| L-6 | Compiled, no runtime, no GC, memory-safe | ⚠️ | **"No *tracing* GC."** Partial state with eviction needs epoch-scoped reclamation, which is a collector. §3.2 |
| L-7 | Query, filter, join, aggregate large datasets | ✅ | |
| L-8 | Express queries so the engine can do more | ✅ | Follows from L-2 |
| L-9 | More explicit control to avoid pathological plans | ⚠️ | In tension with L-8 *within one syntactic layer*. Dissolved by algorithm/schedule separation. §3.3 |
| L-10 | Set-at-a-time semantics | ✅ | |
| L-11 | Iterative graphs, recursive analytics, ML without impedance mismatch | ✅ | Guarded recursion + WCOJ + typed arrays, one type system |
| L-12 | Strict serializability | ❌→⚠️ | **Category error.** Not a language property. §3.4 |
| L-13 | No unnecessary type conversion or copying | ⚠️ | Zero-copy is bounded by trust. §3.5 — and the ledger's hash chain resolves it |
| L-14 | Compose clean functional code | ✅ | |
| L-15 | Nested subqueries and CTEs | ✅ | And **unnesting them is where the optimizer's real win is.** §4 |
| L-16 | Compiles to machine code without an interpreter | ⚠️ | Contradicts L-5 for short queries. §3.6 |
| L-17 | Vectorized execution, SIMD, cache-resident batches | 🔷 | Solved. But the measured gain is ~1.4×, not 8×. §4 |
| L-18 | Dataframe/array language | ✅ | |
| L-19 | Deductive and logic language | ✅ | |
| L-20 | DSLs that compile to bare metal | ✅ | This is `comptime`, and it is where the dynamism belongs |
| L-21 | I/O-bound reads, CPU-bound compute, respect fundamental complexity | ✅ | A discipline, not a feature. Testable as a cost model |
| L-22 | Eliminate interpreter cost, bad plans, copies, cache misses | ⚠️ | Three of four are achievable. "Bad plans" is refuted. §3.7 |
| L-23 | Better semantics than SQL, explicit control, composability | ✅ | |
| L-24 | Follow the thesis's enabling and cumulative findings | ✅ | The organising constraint of both specifications |

**Nineteen of twenty-four are consistent as written. Four need restating. One is a category
error.** None is self-cancelling. That is the answer to the question.

### The engine

| # | Requirement | Verdict | Note |
|---|---|---|---|
| E-1 | Replaces PostgreSQL and MySQL | ✅ | Wire compatibility, already built |
| E-2 | Fast | ⚠️ | Needs a number. §5 gives honest ones, per workload |
| E-3 | Indexes | ✅ | |
| E-4 | Query optimization | ✅ | |
| E-5 | Cache-aware | ✅ | |
| E-6 | Disk-aware data structures | 🔷 | Solved: PAX-layout leaf pages in a B⁺-tree (Umbra). Adopt |
| E-7 | Never a bad join order or wrong index | ❌→⚠️ | **Refuted by three theorems.** Restated in §3.7 as a bounded claim |
| E-8 | Transactional + analytical + graph + streaming simultaneously | ✅ | Open, not impossible. The pieces exist separately |

---

## 3. The six restatements

Each of these is a requirement that becomes achievable when said precisely. None requires
abandoning what was asked for.

### 3.1 "Replaces SQL" — as supersedes, not as proves-inadequate

SQL cannot be shown inadequate, because it is relationally complete. What can be shown is
that a *coherent type system spanning money, effects, consistency, confidentiality and
retention* has not been built, is not obtainable by composing five independent extensions
because the analyses are mutually constraining, and that Niles is an existence proof that it
can be built.

That is a real contribution and it is the honest form of the claim. **Replacement happens
by supersession**: the SQL surface lowers to the same IR (already built, §6.9), so
migration is incremental rather than a rewrite, and the new capabilities are available to a
program that has not yet been ported.

### 3.2 "No garbage collector" — no *tracing* garbage collector

Partially-stateful state with eviction has extents decided at runtime by the read workload.
There is no lexical scope to tie them to, and every real engine that does this uses
epoch-based reclamation — which *is* a collector.

Restated as **"no tracing GC; deterministic, epoch-scoped reclamation,"** the requirement is
achievable and, better, it is *free*: the epoch-ordered ledger already defines the
reclamation domain. The visibility timeline and the memory-reclamation domain are the same
object, which is a small bonus result the thesis can claim.

### 3.3 "Set-at-a-time *and* explicit plan control" — algorithm/schedule separation

These are genuinely opposed **within one syntactic layer**, which is why PostgreSQL has
refused query hints for twenty-five years. They are not opposed across two layers.

Halide (PLDI 2013) separates *what* to compute (the algorithm) from *how* (the schedule).
Neumann and Leis reach for the same move independently in SaneQL — "a version of the
language with additional operator hints could also be used to represent physical query
plans" — and Brandon asks for it in "Against SQL."

**The contribution available here is verified schedules.** Nobody has made the schedule
carry a *proof* that it preserves the algorithm's denotation. Every one of PostgreSQL's six
stated objections to hints reduces to *hints are unverified*; a verified schedule eliminates
five of the six. Nilestream is unusually well placed to do this, because the IR verifier is
already in the trusted base — the machinery exists and needs a new judgement, not a new
subsystem.

### 3.4 "Strict serializability" — an engine property the language *types*

Strict serializability is Papadimitriou's serializability plus Herlihy–Wing real-time order:
a property of a concurrency-control protocol over a durable store. The mechanisms that
produce it — a sequencer, a WAL, a coordinator — *are* a runtime. A language cannot provide
it.

What a language can do is what Niles already does: **type isolation as an effect, and
statically reject a program that reads below the rung it promises.** `NL0311` is that check,
and it fires — a view promising `ledger_consistent` while reading a `bounded` one does not
compile. The guarantee is the engine's; the *discipline* is the language's.

This also corrects the thesis: Contribution 4's soundness theorem should say what it proves
— that a well-typed program cannot *request* an unsound combination — and attribute the
guarantee to where it lives.

### 3.5 "No copying" — bounded by trust, and the ledger is a trusted writer

Of {zero-copy, memory-safe, untrusted input} you get any two. Zero-copy *within* your own
memory is solved and safe (Arrow, Polars, in safe Rust). Zero-copy *from external bytes*
asserts unchecked type invariants, and validation is O(n) — `rkyv` acknowledges exactly this
with its optional `bytecheck`.

The resolution here is specific to this design and it is clean: **a hash-chained,
append-only ledger written only by the engine is a trusted writer.** The chain *is* the
validation, amortised per epoch rather than per read. So zero-copy from the ledger is safe
and zero-copy from a client's wire buffer is not, and the boundary is exactly where the
hash chain ends.

### 3.6 "No interpreter" — contradicts being a PostgreSQL replacement

Measured, not argued. LLVM code generation costs **40–90 ms per query**; direct x86 emission
costs 1–2 ms; Umbra's "Flying Start" bytecode tier compiles **108× faster for 1.2× slower
execution**. At 100 MB, an interpreter finishes *every* query before a compiler has finished
compiling.

A system that compiles unconditionally cannot serve an OLTP workload, and an OLTP workload
is most of what PostgreSQL and MySQL are used for. **Restated: no interpretive overhead
*in steady state*, via adaptive tiering** — bytecode first, compile on promotion. That is
Umbra's answer and it is the only one that satisfies both L-16 and E-1.

### 3.7 "Never a bad plan" — a bounded claim

Three independent barriers, and the second is decisive:

1. **Charikar et al. (PODS 2000):** any estimator sampling *n* of *N* rows incurs
   Ω(√(N/n)) ratio error on some input.
2. **Plan Bouquets (Dutt & Haritsa, SIGMOD 2014):** even abandoning compile-time estimation
   and *discovering* selectivities by execution, **no deterministic online algorithm
   achieves maximum suboptimality below 4×**, with even one unknown selectivity.
3. Learned optimizers do not rescue it — they degrade under updates, cost 5–20 ms of
   inference, and industry has not adopted them.

**Restated, and this is the version to build:** *no plan containing an unbounded nested
loop, and no plan worse than k× the oracle plan for a stated k ≥ 4.*

The first half is achievable and is empirically the largest single win available in the
literature: Leis et al. cut queries running more than 2× slower from **38% to under 4%**
simply by disabling risky nested loops and enabling runtime hash-table resizing.
**Restricting the plan space beats improving the estimates**, and it is cheap.

---

## 4. What the measurements say about where to spend effort

The performance survey (`docs/research/performance-baselines.md`, 33 primary sources) produced
one finding that reorders the entire optimizer roadmap, and two that prevent overclaiming.

**Subquery unnesting is worth ~510× geomean. Join ordering is worth ~7%.**
(Dreseler et al., PVLDB 2020.) Every instinct says the optimizer's job is join enumeration;
the measurement says the optimizer's job is *unnesting*. `join_order.rs` is built and its
`DPccp` is correct, but the next optimizer investment should be the unnesting rules, by two
orders of magnitude.

**SIMD buys ~1.4× on TPC-H Q6 and ~1.1× on Q3/Q9 — not 8×.** The 8.4× is a microbenchmark
selection. And **compiled versus vectorized is a wash**: on Q6 the two are a dead tie at
15 ms, and Kersten's whole spread is 0.66×–1.93×, which the authors call "not large
differences." Both belong in the spec as *parity requirements, not differentiators*.

**Worst-case-optimal joins were chosen zero times out of 923 joins** across TPC-H and JOB by
Freitag's own hybrid optimizer. WCOJ is a parity requirement for graph-shaped queries and a
non-event for relational ones. (The 4-clique speedup is 4.5–6.9× against EmptyHeaded, not
the 77× previously cited here — that figure does not appear in the paper and has been
removed.)

---

## 5. What "faster than PostgreSQL" can honestly mean

Speed is the primary goal, so the specification needs numbers rather than an aspiration.
The survey normalised every OLTP result to transactions per second per core:

| | txn/s/core |
|---|---|
| PostgreSQL 18.1 (1000 warehouses, fsync off, 48-core EPYC) | **333** |
| A well-built general-purpose in-memory DBMS | **~3 000** |
| Silo (32 warehouses in RAM, no network, no recovery, partition-local) | **21 875** |

**PostgreSQL to a well-built in-memory system is about 9×. The next 7× to Silo is bought
entirely with restrictions Nilestream cannot take** — no network round trip, no recovery, no
think time, partition-local workloads. Given strict serializability, durability, retention
and wire compatibility, **the honest OLTP budget is 5–10×.**

On analytics the available win is much larger — **11.5× geomean against *indexed*
PostgreSQL** on ClickBench, recomputed from raw JSON on identical hardware — but two things
must be said about it. It is **mostly storage layout, not execution engine**: it comes from
columnar storage and not reading unused columns. And it is **bimodal**: PostgreSQL wins
outright on 10 of 43 queries, every one of them a point lookup or a highly selective indexed
access, where it is already at the achievable bound.

**Where claiming a win would be dishonest:** point lookups, indexed selective access, and
durable single-commit latency — that last one is a hardware constant (1.6–12.4 µs on
power-loss-protected NVMe against 891–2974 µs without, a ~1000× swing that belongs to the
storage device and not to any database).

The specifications state targets per workload class, and state explicitly where the target
is *parity*.

---

## 6. The answer, in one paragraph

The goals are logical and consistent. Nineteen of twenty-four language requirements hold as
written; four need precise restatement and one is a category error that costs nothing to
correct. Removing Ruby-dynamism removed the only genuine contradiction, and replacing it
with "mathematically functional" made the remainder mutually reinforcing rather than merely
compatible. The engine requirements are consistent except for "never a bad plan," which is
refuted by theorem and must become a bounded claim. **No language and no engine meeting this
set exists** — the two landscape surveys establish that against ~50 primary sources — and
nothing in the set is impossible. What is being asked for is buildable, and the honest
speed claim is 5–10× on transactional work, an order of magnitude on scan-heavy analytical
work, and parity where PostgreSQL is already at the bound.
