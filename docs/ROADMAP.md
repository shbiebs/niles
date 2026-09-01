# ROADMAP — how Niles replaces SQL and Nilestream replaces PostgreSQL

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

---

## The eight phases

Each phase states its gate — the measurement that must hold to proceed — and its kill
criterion, the result that would end that line of work rather than prompt another attempt.

### Phase 1 — Plan-space restriction · *the cheapest large win*

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

### Phase 2 — Subquery unnesting · *the 510× line*

**Build.** The unnesting rules: correlated `exists` → semi-join, correlated scalar subquery →
outer join with aggregation, `in`/`not in` → semi/anti-join with the three-valued-logic care
that `not in` requires.

**Gate.** A correlated `exists` MUST lower to a semi-join, verified in the IR rather than
in the timing. Then the ratio between nested and unnested forms MUST be reported on a corpus
of at least twenty correlated queries. Dreseler measured ~510× geomean; a result within an
order of magnitude of that is a pass, and anything near 1× means the rewrite is not firing.

**Kill criterion.** If unnesting produces a *wrong* answer on any `not in` with NULLs, stop
and fix the three-valued logic before proceeding. This is the classic unnesting bug and it is
a correctness failure, not a performance one.

**Cost.** Weeks. **Worth ~70× more than the join-ordering work already done.**

---

### Phase 2.5 — Serve the read path from `proto-engine` · **BUILT**

**Why it appeared.** The E16 harness could compare Nilestream to PostgreSQL over the wire and
immediately exposed that there was nothing to compare: `nilestreamd` served reads from a hash
map, so the point-lookup row measured the protocol path rather than the read-model runtime.

**Built.** `nilestream-server::rev_engine` implements `Serving` over
`proto_engine::{Ledger, PartialView}`: partial materialisation, the absence lattice, an
anchored upquery on a miss. A historical read (`anchor < head`) reconstructs from the base
rather than reusing a fresher slot — the view's hit rule is right for a bounded-staleness rung
and wrong for the as-of read a dispute asks.

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

### Phase 3 — Adaptive tiering · *what makes OLTP possible at all*

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

### Phase 4 — Columnar storage: PAX leaves in the B⁺-tree · *the analytical order of magnitude*

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

### Phase 5 — The schedule verifier · *the novel contribution*

**Build.** A schedule term attached to a query, and a verifier judgement proving it preserves
the algorithm's denotation.

**Gate.** A schedule that changes the result MUST be rejected *by the verifier*, not merely
produce a different plan. A schedule that is only slower MUST be accepted. The negative
control — a schedule reordering a non-commutative operation — MUST fail.

**Kill criterion.** If the verifier cannot decide denotational equivalence for the operator
set without a general theorem prover, narrow the schedule language until it can. A verifier
that says "maybe" is a hint, and hints are what this is meant to replace.

**Why it matters.** PostgreSQL has refused query hints for twenty-five years, and every one
of its six objections reduces to *hints are unverified*. A verified schedule eliminates five
of six. **Nobody has built this** — Halide separates algorithm from schedule, SaneQL and
"Against SQL" both ask for the same separation, and none makes the schedule carry a proof.
Nilestream is unusually well placed because the IR verifier is already in the trusted base:
this needs a new judgement, not a new subsystem.

---

### Phase 6 — WCOJ · *parity, not a differentiator*

**Build.** Adopt Freitag's hybrid optimizer approach.

**Gate.** **Zero regression on TPC-H and JOB** — that is the load-bearing requirement, and it
is what Freitag demonstrates. Then 4.5–6.9× on 4-clique.

**Calibration, stated in the roadmap so it is not overclaimed later.** Freitag's own optimizer
chose WCOJ **zero times out of 923 joins** across TPC-H and JOB. This phase buys graph-query
parity and removes the need for a separate graph engine. It does not make relational queries
faster and the evaluation chapter must not imply it does.

---

### Phase 7 — Distributed execution over a real network

**Build.** Nothing new. Deploy what exists.

**Gate.** The consensus, cross-shard commit and distributed read path have all been built and
tested in a deterministic simulator, and **not one has run over a network.** The gate is
partial failure, clock skew, and an operator. An Elle-style cycle check MUST find no anomaly
— and this matters *even if the conservation suite passes*, because a published analysis
shows that combination is possible.

**Kill criterion.** A conservation violation traceable to the *model* rather than to an
implementation defect halts the programme (thesis K2).

**Expectation, stated in advance.** The first contact with a real network finds something.
Budgeting zero surprises here would be the least credible line in this document.

---

### Phase 8 — GBS on the engine

**Build.** Run the eleven implemented product lines against Nilestream rather than against
the in-memory harness, and add the remaining eighteen.

**Gate.** The conservation suite MUST pass under continuous eviction, over the wire, with
durability on. Then the core-banking benchmark against PostgreSQL, reporting per workload
class — and reporting parity where parity is the honest result.

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
| 5 **Schedule verifier** | **The novel contribution** | Months | High |
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
* **If the schedule verifier needs a general theorem prover**, phase 5 narrows its language
  until it does not, or is abandoned. A verifier that answers "maybe" has rebuilt hints.
* **If the distributed protocols fail over a real network** in a way the simulator could not
  have caught, the simulator's fault model is wrong and needs rebuilding before the protocols
  do.
