# ROADMAP — Nilestream, the instrument, phase by phase

**Read this first (cycle 16, C16-09).** This roadmap was written as "how Niles replaces SQL and Nilestream replaces PostgreSQL". Neither is the plan any longer. The product language is PostgreSQL's SQL and PL/pgSQL with a checked dialect, of which Niles is the specification and reference checker (decision DB-8); the product engine is PostgreSQL (DB-1); and Nilestream, whose phases this document sequences, is the thesis's instrument. The phases below are kept, with their gates and kill criteria, because they are cited by number; a phase for a stopped component is specification and future work, as the table says.

<!-- BEGIN:scope-summary.roadmap thesis/scope.toml#scopesummary -->

*Generated from `thesis/scope.toml`. Do not edit by hand.*

**Scope.** Of the 13 components this thesis describes, the **product** is: PostgreSQL as the engine, with the append-only ledger, the REV mechanism and the checks; PostgreSQL's SQL and PL/pgSQL with a checked dialect as the language; and GBS, a core-banking system whose product goal is to replace Loan IQ and Calypso. Kept as the theory's **instrument**: Nilestream's ledger write path, REV runtime and reference oracle; and Nilestream's PostgreSQL wire surface. Kept as the **reference** the product is tested against: Niles, its full grammar, type system and calculus. **Specification and future work** — designed, described in full, and not evaluated anywhere in this thesis: the MySQL wire listener; the native frame protocol; distributed execution, consensus and cross-shard commit; the self-hosting compiler; the native and WASM compilation targets; the online materialization optimizer of C5; and homomorphic commitments for confidential columns. Source: `thesis/scope.toml`, rendered here and into §1.9, §7, §8, `docs/ROADMAP.md` and the head of every chapter describing a stopped component.

<!-- END:scope-summary.roadmap -->

<!-- BEGIN:scope-table.roadmap thesis/scope.toml#scopetable -->

*Generated from `thesis/scope.toml`. Do not edit by hand.*

| Component | Status | Decision | What exists | Described in |
|---|---|---|---|---|
| The engine: PostgreSQL with the append-only ledger, the REV mechanism and the checks | **product** | DB-1 (cycle 16): PostgreSQL is the product engine, whatever the engine comparison (H-E1) finds; DB-11: extended in SQL and PL/pgSQL first, with a compiled extension only where a measurement shows SQL is the bottleneck | E14 rebuilt the REV mechanism in stock PostgreSQL 16 (§6.10.3), and it runs as E27's P+ arm (crates/comparator); crates/rev-sidecar is the hybrid H3 (a PostgreSQL ledger with a REV sidecar). No packaged PostgreSQL ledger schema ships yet | §6.10.3, §9.14, results/E27-comparator.md |
| The language: PostgreSQL's SQL and PL/pgSQL with a checked dialect | **product** | DB-8 (cycle 16, confirmed by the author 2026-10-01), on the measurements H-L1 (E14 counterproposal) and H-L2 (E30b') | crates/nilescheck-sql: a catalog checker reading annotations from PostgreSQL comments, with linearity, conservation through niles-lang's solver, capabilities, declared effects and body typing. Enforcement at DDL time is not built (cycle 16, C16-10) | §6.10.3, §9.11, results/E14-minimal-counterproposal.md, results/E30b-syntax.md |
| Niles: the full grammar, type system and consistency-effect calculus | **reference** | H-L1 retired the grammar to a specification; DB-8 made Niles the specification of the dialect's semantics and the reference checker it is tested against. It does not replace SQL | crates/niles-lang (the stage-0 front end, the checker and lowering to the IR), crates/nilesc, crates/niles-interp | §6, Appendix B, §4.5 |
| Nilestream: the ledger write path, the REV runtime and the reference oracle | **instrument** | A-3 kept it (2026-09-21); DB-1 made it the theory's instrument and reference REV implementation, not the product engine | crates/nilestream-ledger, crates/nilestream-core, crates/conservation-suite, crates/proto-engine | §7.1, §7.4, Appendix D, Appendix K |
| Nilestream's PostgreSQL wire surface | **instrument** | A-3 kept it (2026-09-21) | crates/nilestream-server: PostgreSQL protocol v3, simple and extended query paths, driven by psql in conformance tests; no binary result format and no SCRAM | §7.3, §7.5, Appendix D.4 |
| GBS: a core-banking system in Rust, whose product goal is to replace Loan IQ and Calypso | **product** | DB-4 (GBS's backend is PostgreSQL), DB-5 (replacing Loan IQ and Calypso is the product goal; the thesis evaluates only what is built), DB-10 (pricing and risk through ORE as a separate process) | a separate, private repository: the kernel, the mechanisms and the products, with a Nilestream adapter held by its G3 correctness suite. Its PostgreSQL backend and the G3p suite are cycle 16's C16-05, not yet run | §11.4 |
| The MySQL wire listener ("the MySQL replacement") | **specification and future work** | A-3 (2026-09-21) stopped the listener | a packet codec only (crates/nilestream-server/src/mysql_wire.rs, with tests); nothing calls it and no socket accepts MySQL clients | §7.3, §11.5.4, Appendix D.4 |
| The native Nilestream frame protocol | **specification and future work** | never scheduled; Appendix D.4 has described it as specification since cycle 9 | nothing | Appendix D.4 |
| Distributed execution, consensus and the cross-shard commit protocol | **specification and future work** | A-3 (2026-09-21) stopped consensus and distribution | a single-process deterministic simulator: crates/nilestream-consensus (a replicated ledger group and two-phase commit over groups, cross_shard.rs) and crates/nilestream-core/src/distributed.rs. None of it has run over a network | §8.5, §8.6, §8.7, Appendix G.8 |
| The self-hosting compiler | **specification and future work** | A-3 (2026-09-21) stopped the bootstrap | stage 0 and the first front-end stage only: crates/niles-interp (an interpreter for the imperative subset), bootstrap/lexer.niles and bootstrap/parser.niles. Nothing from Appendix E.6 onward | Appendix E |
| Compilation targets: native Linux and macOS on ARM64, Windows on x86-64, WSL and WASM | **specification and future work** | A-3 (2026-09-21), with cycle 14's D-3 keeping the chapter as specification | nothing: nilesc lowers to the typed IR and has no back end | Appendix C, Appendix G.6, Appendix G.7 |
| The online adaptive materialization optimizer (C5) | **specification and future work** | A-3 (2026-09-21) shrank the optimizer to the planner and stopped the online optimizer | the planner kept by A-3 (crates/nilestream-optimizer: unnesting, join ordering, three eviction rules); no estimator, no hysteresis schedule, no rent-or-buy break-even | §4.6, Appendix I |
| Committed (homomorphic) commitments for confidential columns | **specification and future work** | A-3 (2026-09-21): confidentiality labels stay and are checked; the commitments stop | the labels and their flow check in crates/niles-lang; no commitment scheme | §6.20, §6.21, §6.22, Appendix B.13 |

<!-- END:scope-table.roadmap -->

Speed is the primary goal, so this roadmap is **sequenced by measured value, not by
instinct**, and every phase carries a numeric gate and a kill criterion.

Three findings from `docs/research/performance-baselines.md` set the order, and all three point
away from where effort would naturally go:

| Instinct says | Measurement says |
|---|---|
| The optimizer's job is join ordering | Join ordering is worth **7%**. Subquery unnesting is worth **510×** |
| Compilation and SIMD are the speed story | Compiled vs vectorized is a **wash**; SIMD is **1.4×** on real queries |
| WCOJ makes graph queries fast | True, and it was chosen **0 times out of 923 joins** on relational workloads |

The single largest available win in the optimizer literature is not a better estimator. It
is **restricting the plan space**: Leis et al. cut queries running >2× slower from 38% to
under 4% by disabling risky nested loops. That is phase 1.

### The ordering rule, restated per regime — and what it says about this numbering

The rule above reads "sequenced by measured value", and taken flat it is inconsistent with
its own table: subquery unnesting is worth 510× and sits at phase 2, behind a phase-1 item
whose cited result is a *reliability* figure rather than a speed one. **The rule is not one
rule.** The three findings live in three regimes and are not commensurable:

| Regime | The measurement | What it orders |
|---|---|---|
| **Expressibility** | Unnesting is 510× — and, in this engine, a correlated subquery has no delta rule at all, so unnesting is what makes it *servable as a view* | First, always. A rewrite that changes what can be asked outranks one that changes how fast |
| **Tail reliability** | Leis: >2×-slower queries from 38% to under 4% | Second. It removes catastrophes rather than adding throughput, and a catastrophe is what a user reports |
| **Throughput** | Join ordering 7%; compiled-vs-vectorized a wash; SIMD 1.4× | Last, and by the margin the numbers show |

**The decision, recorded rather than silently applied:** the rule is restated per regime and
the phase *numbers* are left alone, because they are cited by number in the thesis, in
`SPEC-ENGINE.md` and in this document's own gates, and renumbering to satisfy an ordering
argument would break every citation to fix a table of contents. The order actually taken
already follows the corrected rule — **phase 2 is built and phase 1 is not** — which is the
clearest evidence that the flat reading was never the one being used.

---

## The eight phases

Each phase states its gate — the measurement that must hold to proceed — and its kill
criterion, the result that would end that line of work rather than prompt another attempt.

**Status vocabulary.** **BUILT** means the phase's build item exists with tests and its gate
has been measured. *Not runnable* means the build item may exist and the **gate cannot be
evaluated**, because the instrument it names does not. *Out of scope for this increment*
means neither is attempted and no claim about it is made anywhere.

### Phase 1 — Plan-space restriction · *the cheapest large win* · **NOT RUNNABLE**

**Not runnable, and the reason is the gate rather than the build.** The gate is stated on the
Join Order Benchmark, and this repository has neither JOB nor a surface that could run it: the
lowered fragment has no general scan-and-join query path exposed over the wire, and §9.14.1's
analytical row already records three PostgreSQL statements — `count(*)`, `count(distinct …)`,
`order by <aggregate>` — as outside it. A phase whose gate cannot be evaluated is not a phase
in progress; it is a phase waiting on an instrument, and this line says which one.

**Build.** Refuse any plan containing an unbounded nested loop. Enable runtime hash-table
resizing. Both go into `join_order.rs`, which already has `DPccp` and the three-term cost
model.

**Gate.** On the Join Order Benchmark, queries more than 2× slower than the best observed
plan MUST fall below 4% (Leis's measured result). Report the before figure too — if the
starting point is not near 38%, the benchmark is not being run faithfully.

**Kill criterion.** If restriction costs more than 10% on the queries it does not rescue,
the restriction is too blunt and the rule needs narrowing rather than the idea abandoning.

**Cost.** Days. It is a filter on an existing enumerator.

---

### Phase 2 — Subquery unnesting · **BUILT**

**Built.** `nilestream-optimizer::unnest` — five rewrites, each named in a `Rewrite` enum so
the gate can be checked in the IR: `exists` → semi-join, `not exists` → anti-join, `in` →
semi-join with the probe in the key, `not in` → filter + anti-join + **null witness**, and a
correlated scalar subquery → left outer join over a grouped aggregate. Refusals are named
too: a `limit` or `order_by` below the apply, an uncertified UDF, an uncorrelated scalar
subquery.

Three things had to exist first, and each is a result in its own right.

* **A nested form.** `Op::Apply { kind, correlation }` — a dependent join. It is *not
  incremental*: one new outer row re-scans the inner relation, so there is no delta rule,
  and `verify` refuses one on a path to a served output (IR018). That reframes the phase:
  unnesting is not an optimisation a planner may skip, it is what makes a correlated query
  expressible as a view.
* **A null.** `niles-ir::value` — `Value::{Null, Int}` and Kleene three-valued logic. The
  IR had no null, so `not in` was unstatable. The module keeps the thesis's three absences
  apart by name: `null`, `Option::None`, and the evicted `Hole`.
* **One semantics.** The reference evaluator moved out of a `#[cfg(test)]` block into
  `niles-ir::eval`, public, shared by the schedule catalogue and the unnesting corpus. Two
  copies of a semantics is two semantics.

**Gate — met.** A 24-case corpus, checked *denotationally* (nested and unnested denote the
same Z-set) rather than structurally, plus a hand-written three-valued oracle for the eight
`not in` cases so a shared misunderstanding cannot cancel between the two circuits.

**Kill criterion — not triggered.** No `not in` case is wrong. Eight of them carry nulls on
the left, on the right, on both, in a group that also matches, in an empty subquery, in an
all-null subquery, correlated and uncorrelated.

**Result.** Counted work, correlated regime: **1.44× at k=1, 4.29× at k=4, 15.71× at k=16,
61.39× at k=64** — quadrupling as `k` quadruples, which is the signature of removing a
quadratic. `results/E17-unnesting.md`.

The 510× figure was a target taken from a different measurement and is not the comparable
number: Dreseler's is a wall-clock geomean over TPC-H at one scale factor, and this is
counted work over a purpose-built corpus as a function of scale. What can be said is that
the ratio is unbounded in `k` rather than a constant, which is the property the phase was
after; a wall-clock comparison waits on the surface syntax below.

**The surface, and two more defects.** `exists`, `not exists`, `in (select …)` and
`not in (select …)` now parse and lower to an `Apply` with the correlation extracted, so a
query a user can write reaches the rewrite (`crates/niles-lang/tests/subqueries.rs`).
Closing that path turned up two defects that had nothing to do with subqueries: a
correlation whose two columns shared a name was left behind as the tautology `k = k`,
making `exists` a no-op; and `=` in a SQL `where` clause parsed as an *assignment*, which
lowering then replaced with `LitBool(true)` — so `where t.z = 1` returned every row. Both
fixed and pinned; `results/E17-unnesting.md` round 2.

**Still open.** A **scalar** subquery in a projection: the rewrite is built and in the
corpus, the projection path does not yet emit an `Apply` for one.

---

### Phase 2.5 — Serve the read path from `proto-engine` · **BUILT**

**Why it appeared.** The E16 harness could compare Nilestream to PostgreSQL over the wire and
immediately exposed that there was nothing to compare: `nilestreamd` served reads from a hash
map, so the point-lookup row measured the protocol path rather than the read-model runtime.

**Built.** `nilestream-server::rev_engine` implements `Serving` over a `proto_engine::Ledger`
and the **REV runtime**: partial materialisation, the absence lattice, an anchored upquery on
a miss. A historical read (`anchor < head`) reconstructs from the base rather than reusing a
fresher slot — a partial view's hit rule is right for a bounded-staleness rung and wrong for
the as-of read a dispute asks.

It held *two* read models for a while: the runtime the wire path reads, and a
`proto_engine::PartialView` beside it that `read_point`, `stats()` and `evict_all` read and
that no query ever consulted. Nothing in the type said which of the two a caller was looking
at, so a test could warm one and assert about the other — and six of them did. The second one
is gone; `proto_engine::PartialView` itself stays, because `nilestream sweep` and the E-series
experiments are built on it.

**Gate — met.** `results/E16-wallclock.md` reports the `point` row at **PARITY**: 130µs p99
against PostgreSQL's 128µs, at an **8–14% miss rate**, each miss a real reconstruction over a
real base. The miss rate is printed with every run, because a parity result at 0% says only
that a warm view is fast while one at 9% says reconstruction is — and the second is the claim.

**What the phase bought on the way.** Three defects, none findable by counting operations:

1. `pg_wire::write_all` issued one socket write per protocol message, so a four-message reply
   stalled on Nagle plus the peer's delayed-ACK timer — 23 lookups/s against PostgreSQL's
   13,600. One buffer, one write, `TCP_NODELAY`: ~14,700/s, a factor of 640.
2. The calibration gate's own baseline was wrong (see the phase table's note on `storage.rs`).
3. A measurement written to a path nothing read, because `cargo test` runs from the package
   directory.

**What remains.** The write path over the wire, and a scan-and-group-by read surface. Both are
`NOT RUN` rows in E16 today, with the reason printed under the table.

---

### Phase 3 — Adaptive tiering · *what makes OLTP possible at all* · **OUT OF SCOPE**

*Out of scope for this increment: the adaptive optimizer it depends on is specified and not built (thesis §4.6), and H-S7 has no runner.*

**Build.** Three tiers: bytecode unconditionally, direct machine-code emission on repetition,
optimising back end only past ~100 ms of measured runtime.

**Gate.** A point lookup MUST complete under 1 ms end to end. The tier each query ran at MUST
be reportable. Umbra's Flying Start is the target shape: 108× faster to compile for 1.2×
slower to execute.

**Kill criterion.** If tier 1 is more than 2× slower than tier 2 on the OLTP corpus, the
bytecode design is wrong — Umbra achieves 1.2× and a much worse ratio means the interpreter
is doing work the compiler was doing at build time.

**Why this is phase 3 and not phase 6.** Without it there is no OLTP path at all: unconditional
compilation costs 40–90 ms per query in the systems the literature measures, and at that price
every point lookup loses to PostgreSQL. This phase is a precondition for E-2's OLTP target, not
an optimisation of it.

**A measurement that narrows this phase.** Compiling one query against the current schema —
parse, resolve, type-check — measures **0.02 ms** on this codebase, not 40–90 ms. The published
figures are for optimising back ends emitting machine code; Niles's front end is nowhere near
that cost, and the E16 harness confirmed it directly. So tiering is still needed for the
*back-end* work Phase 3 describes, and the front end is not the thing to tier. Sequencing an
optimisation against a cost the codebase does not have would have been the expensive kind of
mistake.

---

### Phase 4 — Columnar storage: PAX leaves in the B⁺-tree · *the analytical order of magnitude* · **OUT OF SCOPE**

*Out of scope for this increment: no storage engine work is attempted, and §9.14.1's analytical NOT MET is attributed to an unkeyed `group by` materialising the whole base rather than to layout.*

**Build.** Adopt Umbra's layout. Not invent — adopt.

**Gate.** A scan of one column MUST NOT fault the other columns' pages, measured in pages.
A point lookup MUST still touch O(log n) pages **against the same physical structure**. Then
the ClickBench geomean against indexed PostgreSQL MUST reach 10×.

**Kill criterion.** If point-lookup latency regresses below PostgreSQL parity, the layout has
bought analytics with transactions and the seam has reappeared in a new place. Stop.

**Honesty note for the evaluation chapter.** The analytical win is **mostly storage layout,
not execution engine** — it comes from not reading unused columns. Attributing it to the
compiler or to SIMD would be attributing it to the wrong thing.

---

### Phase 5 — The schedule checker · **BUILT** *(and re-specified first)*

**What changed before anything was built.** The phase asked for a verifier that *decides*
denotational equivalence over the operator set. That operator set includes Z-set negation, and
equivalence of relational algebra with difference is **undecidable**. The kill criterion below
would have fired on day one — not because the work was hard, but because the fragment was never
fixed. Narrowing to conjunctive queries does not rescue it either: under the bag semantics
Z-sets actually have, equivalence is graph-isomorphism-hard and containment is open.

**Built instead.** `niles-ir::schedule`: a schedule names rewrites from a **finite catalogue**,
each proven equivalence-preserving under a side condition the checker verifies *syntactically*
before performing it. Five rules today — join commutativity with a compensating projection,
filter pushdown into either side of an inner join, union commutativity, double-negation
elision. Equivalence holds by construction, and `ScheduleError` has **no `Unknown` variant**,
so the checker cannot answer "maybe".

**Gate — met.** A schedule step whose side condition fails is rejected by `check`; a schedule
that is only slower is accepted (commuting a join twice is legal and strictly worse); the
negative control — pushing a predicate that spans both sides — is refused by name. Each rule
carries a **1,000-trial denotation test** against a reference Z-set interpreter on inputs
including negative weights, because a rewrite can be sound on sets, wrong on bags, and wrong
again on Z-sets.

**The kill criterion is deleted.** It cannot fire on a checker: there is no equivalence to
decide, only side conditions to verify.

**Why it matters, restated precisely.** PostgreSQL has refused query hints for twenty-five
years and every one of its objections reduces to *hints are unverified*. This does not decide
plan equivalence — nothing can. It makes a hint into an operation whose precondition is checked
before it is performed. **Nobody has built this**: Halide separates algorithm from schedule,
SaneQL and "Against SQL" both ask for the same separation, and none makes the schedule carry a
proof.

**What remains.** The catalogue is five rules. Projection pushdown needs functional-dependency
inference and is deliberately absent; semi-join introduction arrives with Phase 2's unnesting;
physical-operator choice belongs in the optimizer, where `plan_space::check` is already the
side condition.

---

### Phase 6 — WCOJ · *parity, not a differentiator* · **OUT OF SCOPE**

*Out of scope for this increment, and the roadmap's own table is the argument: WCOJ was chosen 0 times out of 923 joins on relational workloads.*

**Build.** Adopt Freitag's hybrid optimizer approach.

**Gate.** **Zero regression on TPC-H and JOB** — that is the load-bearing requirement, and it
is what Freitag demonstrates. Then 4.5–6.9× on 4-clique.

**Calibration, stated in the roadmap so it is not overclaimed later.** Freitag's own optimizer
chose WCOJ **zero times out of 923 joins** across TPC-H and JOB. This phase buys graph-query
parity and removes the need for a separate graph engine. It does not make relational queries
faster and the evaluation chapter must not imply it does.

---

### Phase 7 — Distributed execution over a real network · **STOPPED: specification and future work**

<!-- BEGIN:scope-distributed.roadmap thesis/scope.toml#scopeline -->

*Generated from `thesis/scope.toml`. Do not edit by hand.*

> **Scope: specification and future work.** Distributed execution, consensus and the cross-shard commit protocol. *Decision:* A-3 (2026-09-21) stopped consensus and distribution. *What exists:* a single-process deterministic simulator: crates/nilestream-consensus (a replicated ledger group and two-phase commit over groups, cross_shard.rs) and crates/nilestream-core/src/distributed.rs. None of it has run over a network. Nothing in this chapter is evaluated; it is a design.

<!-- END:scope-distributed.roadmap -->

*Out of scope for this increment. `distributed.rs` and `cross_shard.rs` are models — nothing in the server or the experiments calls either, and there is no network anywhere in this repository (thesis §11.5.3).*

**Build.** Nothing new. Deploy what exists.

**Gate.** The consensus, cross-shard commit and distributed read path have all been built and
tested in a deterministic simulator, and **not one has run over a network.** The gate is
partial failure, clock skew, and an operator. An Elle-style cycle check SHOULD find no anomaly (planned; no such checker is built)
— and this matters *even if the conservation suite passes*, because a published analysis
shows that combination is possible.

**Kill criterion.** A conservation violation traceable to the *model* rather than to an
implementation defect halts the programme (thesis K2).

**Expectation, stated in advance.** The first contact with a real network finds something.
Budgeting zero surprises here would be the least credible line in this document.

---

### Phase 8 — GBS on the engine

**Superseded as the product plan (cycle 16).** GBS's product backend is PostgreSQL (decision DB-4), and its correctness there will be held by a G3p suite (cycle 16's C16-05), which has not run yet. GBS on Nilestream, below, stays as the instrument's evidence: G3 is the correctness suite that holds it, and its latest run is 29 rows and 0 failures at GBS `966e259` (2026-10-01).

**Build.** Run the twenty-nine implemented product lines against Nilestream rather than
against the in-memory harness. *(This said "eleven … and add the remaining eighteen" while
GBS's own `implemented_rows()` listed twenty-nine. The count was stale, not the plan.)*

**Gate.** The conservation suite MUST pass under continuous eviction, over the wire, with
durability on. Then the core-banking benchmark against PostgreSQL, reporting per workload
class — and reporting parity where parity is the honest result.

**Status: the first clause of the gate is met; the second is not run.**

`gbs/Makefile`'s `g3` target runs it and writes `gbs/results/G3-verdict.md`. Every matrix row
is driven through a `Session<EvictingLedger>` — GBS's own session type over a durable
Nilestream segment at `SyncPolicy::Always`, with balances served by a REV whose budget is a
quarter of the key space and which is wiped before every anchor sweep. The result is
**23 product-evidenced + 6 generic-path-only of 29, 0 failures**, with replay and
crash-recovery verdicts per row. Two shapes — `transfer` and `drawdown` — cross the
PostgreSQL wire into the daemon, driven by `psql`; the other twenty-seven are in-process and
the verdict table says so.

The gate's second sentence — the core-banking benchmark against PostgreSQL — is **not run by
this target**, and no parity claim about GBS on Nilestream is made anywhere. `bank-bench`
measures the engines directly; see `docs/BENCHMARK.md` and `results/`.

---

## The thing this roadmap will not do

**It will not build a matching engine into the ledger.**

An exchange matching engine runs single-threaded on a pinned core with no allocation in the
hot path, kernel-bypass networking, and a latency budget in hundreds of nanoseconds. A
durable ledger is `fsync`-bound and lives in microseconds to milliseconds. **These differ by
three to four orders of magnitude**, and no storage engine serves both. A system claiming to
would be lying about one of them.

The separation, and why the thesis's own model is the right interface:

| Tier | What it is | Latency | Durability |
|---|---|---|---|
| **1 — matching** | Deterministic, in-memory, pinned core, no allocation | ~100 ns – 1 µs | **None in the hot path** |
| **2 — ledger** | Ingests tier 1's sequenced stream as pre-agreed epochs | µs – ms | `fsync`, hash-chained |
| **3 — REVs** | Positions, risk, P&L, regulatory roll-ups | per rung | derived |

**A matching engine's output is already a total order — which is exactly what an epoch is.**
So tier 2's admission is a *validation* rather than a sequencing decision, which is the same
observation Calvin and Aria make about deterministic execution and which the thesis already
cites. The interface between the two tiers is the one place this architecture is unusually
well suited to a problem it was not designed for.

---

## Sequencing, and what it costs

| Phase | Value | Effort | Risk |
|---|---|---|---|
| 1 Plan-space restriction | 38% → <4% of queries badly planned | Days | Low |
| 2 **Subquery unnesting** | **~510× geomean** | Weeks | Low |
| 2.5 **Read path on `proto-engine`** | **Built** — E16 `point` is PARITY at a 9% miss rate | Done | — |
| 3 Adaptive tiering | Makes the OLTP target reachable at all | Weeks | Medium |
| 4 PAX columnar storage | 10× analytical | Months | Medium |
| 5 **Schedule checker** | **Built** — five proven rewrites, 1,000-trial denotation tests | Done | — |
| 6 WCOJ | Graph parity | Months | Low |
| 7 Distributed over a network | Validates what is built | Months | High |
| 8 GBS on the engine | The end-to-end claim | Ongoing | Medium |

**Phases 1 and 2 are the whole first quarter and they are worth more than phases 4 and 6
combined.** That is the finding this roadmap exists to act on: two weeks of unnesting rules
beats months of storage work, measured, and the instinct that says otherwise is wrong.

---

## What would change this roadmap

Stated so it is falsifiable rather than a plan that survives contact with any evidence.

* **If unnesting measures under 10× on our corpus**, Dreseler's 510× does not transfer and
  phase 2 should be reordered behind phase 4.
* **If plan-space restriction costs more than 10%** on unrescued queries, phase 1's rule is
  too blunt.
* **If PAX leaves regress point lookups below PostgreSQL**, phase 4 stops: the seam has
  reappeared and the single-format claim is false for our implementation.
* **The schedule verifier did need a general theorem prover**, and this is what changed:
  equivalence over an operator set with Z-set negation is undecidable, so the phase was
  re-specified as a *checker* over a finite catalogue of proven rewrites before any of it was
  built. The prediction held and the response was to narrow, exactly as written.
* **If the distributed protocols fail over a real network** in a way the simulator could not
  have caught, the simulator's fault model is wrong and needs rebuilding before the protocols
  do.

## Deleted from the workspace, and why

Three crates were carried in the workspace as `pub mod` lines over one-line files: no
function, no type, no caller. They appeared in Appendix D's component map, in
`docs/SPEC-ENGINE.md`'s status lines and in the thesis's chapters as though they were
components, which is the only thing a reader could have concluded from finding them in
`crates/`.

A stub is a plan. Kept in the build it is a plan that looks like a component, and every
count of "how much of this exists" was wrong by three. They are removed and recorded here:

* **`nilestream-storage`** — tiering, cold storage, the migration boundary, checkpoint
  persistence. Appendix D.5 and SPEC-ENGINE §PAX describe the design; nothing implements it.
  Checkpointing as a *mechanism* is built and measured, in `proto-engine` and through the
  compiled sweep (SC7, §9.13.1); what is missing is the durable tier beneath it.
* **`niles-stdlib`** — `std::bank`, `std::temporal`, money and the builtin catalogue.
  The builtins that exist live in `niles-interp` and `niles-lang`'s registry; the library
  *over* the core that §6.6 argues for is the unbuilt part, and H-S8 is the hypothesis it
  bears on.
* **`nilestream-lineage`** — provenance annotation, `explain`, `impact`, the semiring
  machinery. Its absence is why H-S9 is withdrawn rather than merely unmeasured: there is no
  lineage mode to switch between, so there is no completeness figure and no overhead figure
  to report (Remark 4.1.2, §12).

Restoring any of them means writing the crate, not un-deleting the file.
