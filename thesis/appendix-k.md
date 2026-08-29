# Appendix K. The Research Prototype and How to Reproduce Chapter 9

Chapter 9's measured results come from a prototype built for this thesis. This appendix states what it is, what it deliberately is not, how it is instrumented, and the exact commands that regenerate every number.

## K.1 What the prototype is

A single-node, in-memory, single-threaded Rust program of roughly 900 lines, in two crates:

* **`proto-engine`** — the mechanisms: an append-only, epoch-ordered, hash-chained ledger with a per-currency zero-sum commit rule and idempotent admission; per-key **anchor indices**; per-key **checkpoints**; a partial view over the **absence lattice** with anchored upqueries and three eviction policies; a full-materialization mode; a Zipf workload generator; and a counted-work cost model.
* **`experiments`** — the harness: ten experiments, each emitting a CSV artifact and a console summary.

The whole of it compiles with no `unsafe` blocks.

## K.2 What it deliberately is not

Stated as a list, because every item is a claim the prototype cannot support:

| Absent | Consequence for claims |
|---|---|
| Durability (`fsync`, WAL) | No durability or recovery-time claim; §9.4.4's write-path numbers are mechanism cost only |
| Concurrency | No contention result (§9.4.4 is a stated non-result); no isolation-level claim |
| Consensus / replication | No distributed claim |
| Query planner, SQL surface, compiler | No language or generality claim |
| Joins and general aggregation | Only per-key sums and rollups; a join-heavy view family may behave differently |
| Real workload traces | Synthetic Zipf only |

It is not Nilestream. It is the smallest artifact that makes Nilestream's mechanisms measurable, and it is presented as such.

## K.3 The measurement discipline

**Counted work is the primary unit.** Base rows read, deltas applied, resident-entry-epochs, upqueries issued. These are properties of the algorithms and the workload, so a reader on different hardware reproduces the same numbers. This is what allows a phase diagram measured on a shared two-vCPU virtual machine to mean something.

**Wall-clock appears once**, in §9.4.4, labelled with the platform and its limits.

**Memory is reported at the logical layer only.** The prototype reports resident *entries* and logical bytes, never process resident-set size. The three layers — application-requested bytes, allocator-resident bytes, and process RSS — measure different things, and the allocator's own documentation makes the inequality chain explicit (`allocated ≤ active ≤ resident ≤ mapped`, where resident includes metadata and unused dirty pages). Reporting a saving measured at the logical layer as though it were an RSS saving would credit the design with the allocator's behaviour; the thesis states which layer it means.

**Five seeds, medians with ranges.** No result rests on one run. The generator is a stated linear congruential generator, so seeds reproduce exactly rather than approximately.

**Baselines that are not the system itself.** Partial materialization is always compared against full materialization on the identical workload and seed; eviction is compared against LRU *and* against the randomized policy that the original partial-state system used. Comparing a system only against itself is a named benchmarking error, and so is presenting a microbenchmark as a system result — the second is answered by scoping (K.2) rather than by the measurement.

## K.4 The experiments

| ID | Question | Primary output |
|---|---|---|
| E1 | Does evict-then-reconstruct equal never-evict, under adversarial interleaving? | Divergences, conservation, rebuild mismatches, miss≠0, idempotency, tamper |
| E2 | Are integration and differentiation mutually inverse on epoch-indexed Z-sets? | Epoch-by-epoch canonical Z-set comparisons |
| E3 | How does resident state compare, partial vs full, across skew? | Resident ratio, hit rate |
| E4 | Where is the break-even between partial and full? | Cost ratio over (skew × budget), swept over memory price |
| E5 | Does reconstruction cost depend on history length? | Base rows per reconstruction vs. ledger length; indexed vs. unindexed |
| E6 | Which eviction policy wins under reconstruction latency? | Misses, base rows, aggregate delay for three policies |
| E7 | What do the commit rule and hash chain cost? | Postings/sec, chained vs unchained (wall-clock) |
| E8 | What does each consistency rung cost? | Misses, base rows, deltas applied, apply invocations |
| E9 | Does growing the key space restore history-independence? | Cost vs. history at fixed per-key ratio |
| E10 | Do per-key checkpoints bound reconstruction cost? | Cost vs. history for checkpoint intervals 256/64/16 |

E9 and E10 exist because E5 refuted a claim; E10 exists because E9 refuted its most plausible defence. That chain — claim, falsification, defence, falsification, mechanism, confirmation — is the appendix's main methodological point.

## K.5 Reproducing Chapter 9

```sh
cargo build --release -p experiments

./target/release/experiments e1 e2      # correctness + duality      (~1 s)
./target/release/experiments e3         # memory vs skew             (~75 s)
./target/release/experiments e4         # phase diagram              (~100 s)
./target/release/experiments e5 e6      # history + policies         (~110 s)
./target/release/experiments e7 e8      # write path + rungs         (~100 s)
./target/release/experiments e9         # refined history test       (~70 s)
./target/release/experiments e10        # checkpoints                (~115 s)

./target/release/experiments all        # everything
```

Artifacts land in `results/` as CSV, one per experiment, plus console transcripts. Chapter 9's tables are transcriptions of those files. Timings are for the platform in §9.1.2; the counted-work numbers do not depend on it, and a reader should obtain the same values on any machine.

## K.6 Instrumentation notes, including one that changed a result

**Counting reconstruction work.** The ledger increments a `rows_touched` counter by exactly the number of base rows a reconstruction reads, including the checkpoint row when one is used. The view attributes that delta to the read that caused it.

**Resident-entry-epochs, not peak residency.** The memory term accumulates the resident count once per applied epoch, giving the integral of residency over time. An earlier version charged *peak* residency for the whole run, which overstated the cost of a strategy whose footprint grows gradually — that is, of full materialization — and produced a phase diagram in which partial materialization won every cell. The correction is described in §9.3.1 and is recorded here because a cost model that encodes its own conclusion is easy to write and hard to notice.

**Deriving anchors rather than rewriting them.** Applying an epoch touches only entries that receive a delta; a resident entry that receives none is certified through the view's applied frontier by construction, and its effective anchor is computed on read. Rewriting every resident entry per epoch would have made maintenance proportional to residency and would have measured the harness rather than the design.

**The E8 null.** The first version of the consistency-rung experiment measured only misses and hit rate, and found no difference between rungs at all. The null was traced to instrumenting the read path when the rung's cost falls on the maintenance path; adding the applied-delta and apply-invocation counters revealed a 66× difference. §9.4.3 reports both the null and the diagnosis, because the sequence is where the finding actually came from.

## K.7 Honest gaps in the instrument

* **Contention is unmeasurable here.** Single-threaded execution means the hot-share sweep in §9.4.4 varies a parameter that cannot affect the outcome. Reported as a non-result rather than as evidence of an absence of contention.
* **The delayed-hit term is analytic.** E6 charges each reconstruction for the requests expected to queue behind it, computed from a service time and an arrival rate; it does not simulate a queue. Labelled as a model, not a measurement.
* **Only two view shapes.** Per-key sums and grouped rollups. Upquery paths for joins — where the partial-state literature's hardest problems live — are not exercised.
* **No fault injection against durable state**, because there is no durable state.

Each gap maps to an experiment in §9.9 and to a phase of the programme in Chapter 8.
