# SPEC-ENGINE — the ideal DBMS

**Normative specification** for an engine that replaces PostgreSQL and MySQL and is faster,
on four workload classes simultaneously. Every performance requirement carries a **number**
and a **workload it applies to**, because a speed specification without both is a slogan.

Companion to `SPEC-LANGUAGE.md`. Evidence in `docs/research/performance-baselines.md` (33
primary sources) and `docs/research/engine-landscape.md` (40).

Status values as in the language specification: **Built** · **Partial** · **Specified** ·
**Adopt**.

---

## Part 0 — The performance contract

The single most important table in this document, because it says where a win is available
and where claiming one would be dishonest.

| Workload | PostgreSQL baseline | Target | Available? |
|---|---|---|---|
| **OLTP, durable, strictly serializable** | 333 txn/s/core (18.1, wh=1000, 48-core EPYC, **fsync off** — *literature, non-durable*; see `docs/research/performance-baselines.md`). The harness's own baseline is a measured PostgreSQL 16.13 at `synchronous_commit = on` and `fsync = on`. | **5–10×** | Yes, bounded |
| **Scan-heavy analytical** | ClickBench, indexed | **10–12×** geomean | Yes — but it is *storage layout*, not execution |
| **Point lookup by primary key** | at the achievable bound | **parity** | **No.** PostgreSQL wins 10 of 43 ClickBench queries, all of them these |
| **Highly selective indexed access** | at the achievable bound | **parity** | **No** |
| **Durable single-commit latency** | 1.6–12.4 µs (PLP NVMe) | **parity** | **No — this is a hardware constant** |
| **Repeated derived query (IVM)** | full re-execution | large, but it is a caching argument | Yes, rhetorically weak |
| **Graph-shaped multi-hop** | recursive CTE | 4.5–6.9× (WCOJ) | Yes, on graph queries only |
| **Memory under partial materialisation** | n/a | ~3× (Gjengset thesis) | Yes |

**The OLTP ladder, and why 5–10× is the honest number:**

The 333 figure carries its durability setting because it was quoted for years without one,
and a non-durable published rate compared against a durable measured one is not a comparison.
The harness therefore does not calibrate against it: it measures PostgreSQL on the same
machine, under the same durability settings, over the same protocol, and uses *that* as the
baseline. The published figure appears only as context, and `bench --calibrate` prints the
ratio so a reader can see how far apart the two machines are.

| | txn/s/core | durability |
|---|---|---|
| PostgreSQL 18.1 (literature) | 333 | **fsync off** |
| A well-built general-purpose in-memory DBMS | ~3 000 | — |
| Silo (in RAM, no network, no recovery, partition-local) | 21 875 | none |

PostgreSQL → a well-built in-memory system is ~9×. **The next 7× to Silo is bought entirely
with restrictions this engine cannot take**: no network round trip, no recovery, no think
time, partition-local workloads. Strict serializability, durability, retention and wire
compatibility are all in the requirements, so Silo's ceiling is not available and quoting it
would be dishonest.

**E-2 is therefore normative as:** the engine MUST meet or exceed 5× PostgreSQL on durable
strictly-serializable OLTP, MUST meet or exceed 10× on scan-heavy analytical work, and
**MUST NOT regress below parity on point lookups and selective indexed access.** The last
clause is the one that keeps the specification honest: a system that wins the first two by
losing the third has not replaced PostgreSQL.

### What bounds the OLTP row, stated as arithmetic

A durable commit is a barrier, so:

> **durable throughput ≤ (barriers per second the device sustains) × (transactions per
> barrier)**

Both factors are measured, neither is a constant, and the row cannot be read without them.

**The first factor is a property of the storage *and of which barrier was asked for*, and it
moves by four orders of magnitude across hosts this project has run on.** The same probe —
a 4 KiB write followed by `sync_data`, seven times, median reported — measured:

| host | filesystem | barrier | barriers/s | µs each |
|---|---|---|--:|--:|
| Linux container | ext4 on a virtio disk | `fdatasync` | 4,961–5,825 | 172–202 |
| Linux VM on a Mac | ext4 on NVMe | `fdatasync` | 500–959 | 1,043–2,001 |
| macOS | APFS on NVMe | **`F_FULLFSYNC`** | **255** | **3,917** |
| Linux container on overlayfs | overlay, `fsync=volatile` | `fsync` | ~1,000,000 | ~1 |

The last row is **not storage evidence**: the mount option makes the call a no-op, and a
ceiling measured there says nothing about durability. `make fsync-proof` checks that the
record barrier reaches the kernel, which that row would also pass — the two checks are
necessary together and neither is sufficient alone.

**The second factor is a property of the engine, and it was 1 because of where one lock
ended.** The sealer group-commits to 4,096 and is tested at sixteen concurrent submitters, but
the daemon held its engine mutex across `Session::handle` *and* the fsync, so submitters
reached `submit` one at a time and `select nilestream_sealer` reported `max_batch = 1` and
1.00 transactions per barrier at every connection count. The write path now applies the rows
under the lock, submits the record **without waiting**, releases the lock, and only then waits
on the barrier before writing the reply — so the acknowledgement still follows the fsync while
the fsync no longer holds the next connection up. `rev_engine::visibility_tests` holds the
other half: an applied epoch is not visible to any reader until its barrier has returned.

Measured through the wire on the Linux container, sixteen `psql` clients appending 300
transactions each against one appending the same:

| connections | `max_batch` | txns per barrier | lock hold p50 |
|--:|--:|--:|--:|
| 1 | 1 | 1.00 | 16 µs |
| 16 | 15 | **6.02** | 4 µs |

The one-connection row is a definition and not a result: a single submitter has nothing to
batch with. Against the committed 4,519 ops/s PostgreSQL baseline, 5× requires 22,595 ops/s,
which needs **4.1** transactions per barrier at 5,500/s, **6.5** at a conservative 3,476/s,
and **89** at the macOS figure. So 6.02 clears the first, is just short of the second, and is
nowhere near the third — the macOS row remains storage-bound whatever the engine does. The
sealer driven directly reaches 9.65–14.68 per barrier depending on host, which is what the
wire figure is converging on rather than exceeding.

E19 measures the end of that chain: durable appends rise from 3,850 ops/s at one connection to
**22,049 at sixteen — 5.73×** on a two-core host. That is short of the 6× this repair was
aimed at, and the shortfall is reported rather than rounded: the remaining serialisation is
the single engine mutex the read path still shares, which is a separate repair and not this
one's.

So the OLTP row is no longer bounded by one transaction per barrier, and on no host measured
so far is it bounded by the storage. What bounds it now is the engine lock itself.

**The ratio is the contract, and it is only a ratio when both sides pay the same barrier.**
PostgreSQL's `wal_sync_method` is recorded beside every run and pre-registered: on macOS its
default is `fsync`, which APFS does not turn into a drive-cache flush, so a comparison there
must set `fsync_writethrough` or `bench` refuses it. Measured once without that check,
PostgreSQL reported 13,458 durable commits per second against a 324/s barrier — 41× its own
storage — while reporting `fsync=on`. Two systems durable against different failures do not
produce a comparison, however carefully the timing is done.

### The analytical contract, re-shaped: two asymptotic rows (DM-03)

The 10–12× figure above is a *geomean over ClickBench*, and it was carried into a harness
whose analytical set returns ten thousand rows per statement. That makes it unreachable for a
reason that has nothing to do with the engine: putting 10,001 rows on the wire costs both
sides the same milliseconds, and a fixed cost added to both terms of a ratio drags it toward
1.0× however fast the server is. The audit's own arithmetic gives a ceiling of about 2.7× in
that shape. A contract nothing can meet is not a demanding contract; it is an unread one.

So the shape changes rather than the number, and it changes to something a benchmark can
falsify. E23 measures the report statement along two axes and fits a slope with its standard
error on each:

| # | Row | Normative claim | Measured by |
|---|---|---|---|
| **E-2a** | **Per row of the answer** | Nilestream's cost per row of output MUST NOT exceed PostgreSQL's. **Parity, not a multiple** — a report has to put its answer on the wire, and a system claiming to beat that would be claiming to send fewer bytes than the answer contains. | `results/E23-scaling.md`, output axis |
| **E-2b** | **Per row of the base** | Nilestream's **warm** cost MUST be *o(1)* in the base: the slope's confidence interval MUST contain zero, while PostgreSQL's MUST be positive. This is F1 — a stream-first system's per-answer cost must not grow with accumulated input — narrowed to a report. | `results/E23-scaling.md`, base axis |

**E-2b is the row the whole architecture is for.** Everything else in this table can be won
by being faster at the same work; this one can only be won by not doing the work — by having
maintained the answer as the writes arrived rather than deriving it when the question was
asked. It is also the only row on which "matches PostgreSQL as the data grows to infinity" is
a statement that can come out false: a ratio at one size cannot distinguish a system that
scales from one that happens to be quick at twenty thousand rows.

Two guards keep these rows honest, and both are in the harness rather than in this document:

* **A slope is refused on fewer than three distinct sizes.** With two the fit is exact, the
  residual degrees of freedom are zero, and the standard error is 0/0.
* **A verdict is refused when the control's own slope is not distinguishable from zero.** The
  first run of E23's output axis held the base at 200,000 rows while the answer grew from 201
  to 1,001, so PostgreSQL's cost was dominated by a scan that did not change and its output
  slope came out *negative* — which a naive comparison reads as Nilestream winning. That row
  now reads `INCONCLUSIVE` and says to widen the answer's range.

**The old wording is not deleted.** `MISMATCH-A-01` in `docs/BUILD-LOG.md` records the 10–12×
claim, where it came from, and why the shape rather than the number was wrong. The
cold-reconstruction composite stays in E16 under its own name as the other end of the phase
diagram — a system that only ever reported the warm end would be hiding the trade the thesis
exists to characterise.

### The report row

Between the two: `report` in the E16 contract table is the same statement at one size,
answered from a maintained view **while a second connection appends to the base**, against
PostgreSQL running the identical statement over its identically growing table. Its target is
**≥ 2.6× PostgreSQL**, derived from the measured parts (frame, client, socket) rather than
chosen to be reachable.

It is stated as a multiple rather than parity deliberately: a maintained view that were
merely at parity with a recompute would be evidence *against* the structural claim, because
the entire argument for holding derived state is that reading it is cheaper than deriving it.
The row is refused — `NOT RUN`, with the reason — when the server says it would answer by the
fold rather than from the view (`explain` reports `report-from-view`), and when the appender
completed no appends, which would make "warm" mean "stale" instead of "maintained".

*Two figures previously cited here have been corrected against primary sources: the WCOJ
4-clique gain is 4.5–6.9× rather than 77× (that number does not appear in Freitag et al.),
and the 1.4× on TPC-H Q6 is the AVX-512 gain and not the compilation gain — compiled versus
vectorized on Q6 is a dead tie at 15 ms.*

---

## Part I — Storage

### E-6 Single-format HTAP storage

**Storage MUST NOT keep two physical copies of the same data for two workload classes.**

*Rationale.* Row-major suits OLTP and column-major suits OLAP, and the usual resolution is
to keep both and synchronise them — which is a seam, and a seam between two representations
of one truth is the thing this whole project argues against. It is also the mechanism by
which every distributed HTAP system pays for analytics with a second copy.

*Status: **Adopt**.* Umbra (Neumann & Freitag, CIDR 2020) is the counterexample to the
folklore: a B⁺-tree whose **leaf pages are in PAX layout**. One structure, one MVCC scheme,
one compiler, no seam. Measured: 3.0× geomean over HyPer on the Join Order Benchmark, with
buffer-manager overhead under 6% against pure in-memory.

**Do not invent storage.** This is solved, it has been buyable since May 2025, and the
thesis's contribution is elsewhere.

*Acceptance test.* A scan of one column MUST NOT read the other columns' bytes, measured in
pages faulted. A point lookup MUST touch O(log n) pages. Both MUST hold against the *same*
physical structure. **Status: Specified.** There is no storage crate. `nilestream-storage`
was a stub with no caller — a `lib.rs` of three `pub mod` lines over three one-line files —
and this sentence said it "has checkpointing and tiering". It has been deleted from the
workspace and recorded in `docs/ROADMAP.md`; checkpointing lives where it is measured, in
`proto-engine` and the compiled sweep.

---

### E-3 Indexes, and the anchor obligation

**Every key a view reconstructs by MUST have an anchor index. A view without one MUST NOT
compile.**

*Rationale.* This is not an optimisation. Without an anchor index a reconstruction scans the
ledger, and the SC7 bound on reconstruction cost does not hold — so a view without one is a
view whose cost model is false, not merely slow.

*Acceptance test.* the GBS schema (`niles/gbs.niles` in the GBS repository) declares five anchor indices and compiles; removing
one MUST fail. Reconstruction cost MUST be bounded at C/2+1 per SC7, measured flat at ≈8.5
base rows across a 64× increase in history. **Status: Built.**

---

### E-5 Cache-awareness

**Hot structures MUST be laid out for cache residency, and the layout MUST be measurable.**

*Calibration.* The literature's decomposition is more useful than any headline: Boncz (CIDR
2005) found that in a tuple-at-a-time interpreter, **real work is under 10% of time and 62%
is record navigation and copying.** The win is in removing the navigation, not in
micro-optimising the work.

*Acceptance test.* Batch sizes MUST default to L1-resident. Cache misses per tuple MUST be
reportable for a scan. **Status: Specified.**

---

### L-13 / E-storage Zero-copy from the ledger

**A read of ledger rows MUST perform O(1) allocations. A decode from a client buffer MUST
validate.**

The trust boundary is the hash chain (see `SPEC-LANGUAGE.md` L-13). **Status: Specified.**

---

## Part II — Execution

### E-exec-1 Adaptive tiering

**The engine MUST NOT impose interpretive overhead in steady state, and MUST NOT compile
unconditionally.**

Three tiers, promotion by measured cost:

| Tier | Entry | Compile cost | Execution |
|---|---|---|---|
| 1 — bytecode | unconditional | ~0 | 1.2× slower than compiled |
| 2 — direct machine code | on repetition | 1–2 ms | native |
| 3 — optimising back end | runtime > ~100 ms | 40–90 ms | best |

*Evidence.* Umbra's "Flying Start" is **108× faster to compile for 1.2× slower to execute**
(VLDB J. 2021). The crossover at ~100 ms is where tier 3 pays for itself.

*Acceptance test.* A point lookup MUST complete under 1 ms end to end — which is only
possible if it never reaches tier 3. The tier each query ran at MUST be reportable, so a
regression is attributable to tiering rather than guessed at. **Status: Specified.**

---

### E-exec-2 Vectorized execution

**Data-parallel operators SHOULD use SIMD over cache-resident batches.**

*Status: **Adopt**, as a parity requirement.* Measured gain 1.4× (TPC-H Q6), ~1.1× (Q3/Q9).
Compiled-versus-vectorized is a wash across the whole Kersten spread (0.66×–1.93×). The spec
requires it because its absence would be a regression, and forbids claiming it as a
differentiator.

---

### E-exec-3 Worst-case-optimal joins

**Cyclic join queries SHOULD be executed by a worst-case-optimal algorithm where the
optimizer's cost model selects it.**

*Status: **Adopt**, and calibrate.* Freitag et al. (VLDB 2020) show WCOJ inside a relational
engine with **zero regression** on TPC-H and JOB — which is the important part, because it
means adopting it costs nothing. The gain is 4.5–6.9× on 4-clique against EmptyHeaded.

**But: the hybrid optimizer chose WCOJ zero times out of 923 joins across TPC-H and JOB.**
This is the answer to "graph queries far more efficiently than SQL" *and* the reason a
separate graph engine is not needed — but on relational workloads it is a non-event.

---

### E-exec-4 Partial materialisation with anchored reconstruction

**Derived state MUST be evictable, and reconstruction MUST be anchored to a frozen prefix.**

*This is the engine's one genuinely novel requirement*, and the landscape survey confirms it
is open: **no existing theorem bounds the cost of consistency under eviction.** Noria took
partial state and gave up consistency ("Noria operators and the contents of its external
views are eventually-consistent"); Materialize took strict serializability and fully
materialises. Nobody has both.

*Calibration of the memory claim.* Gjengset's thesis reports **~3×**, which is the number to
use. The OSDI paper's "9% of full state" is an *essential-state floor*, with the steady-state
working set at 38–60%.

*Acceptance test.* Reconstruction after eviction MUST produce the value that was evicted —
0 divergences across 50 keys, verified. Reconstruction MUST be bounded by checkpoint
interval. **Status: Built** — `nilestream-core`, and reproduced in stock PostgreSQL (E14),
which is what shows the theory is not engine-specific.

---

## Part III — The optimizer

### E-7 No unbounded nested loop, and a stated suboptimality bound

**The optimizer MUST NOT produce a plan containing an unbounded nested loop. It MUST NOT
claim to produce an optimal plan.**

*Why the original requirement is refuted.* "Never a bad join order or wrong index" fails
against three independent results:

1. **Charikar et al. (PODS 2000)** — any estimator sampling *n* of *N* rows incurs
   Ω(√(N/n)) ratio error on some input. Harmouch measured that 1% relative error needs >90%
   sampling.
2. **Plan Bouquets (Dutt & Haritsa, SIGMOD 2014)** — even abandoning compile-time estimation
   and discovering selectivities *by executing*, **no deterministic online algorithm achieves
   maximum suboptimality below 4×**, with one unknown selectivity. Realistic
   multi-dimensional bound ≈ 4ρ.
3. Learned optimizers do not rescue it: degradation under updates, 5–20 ms inference,
   violated monotonicity, no industry adoption.

*What is achievable, and it is the biggest single win in the literature.* Leis et al. cut
queries running >2× slower from **38% to under 4%** by disabling risky nested loops and
enabling runtime hash-table resizing. **Restricting the plan space beats improving the
estimates.**

*Normative form:* the engine MUST refuse a plan whose worst case is unbounded, and MUST
publish a suboptimality bound *k* ≥ 4 rather than claiming optimality.

*A note on indexes.* "Provides indexes" and "never picks the wrong index" are in tension:
Leis (2015) and Microsoft on SQL Server both find misestimation damage is *worse* when more
indexes are available. More indexes means a larger plan space means more room for an
estimate to be wrong in.

*Acceptance test.* Over the Join Order Benchmark, the fraction of queries more than 2× slower
than the best observed plan MUST be under 4%. No plan may contain an unbounded nested loop.
**Status: Partial** — `join_order.rs` implements `DPccp` with a three-term cost model and
prunes non-reconstructible inputs as *legality*; the nested-loop restriction is specified.

---

### E-opt-2 Subquery unnesting — the top priority

**Correlated subqueries MUST be unnested where a semantics-preserving rewrite exists.**

*The finding that reorders the roadmap:*

| Optimization | Measured value (Dreseler et al., PVLDB 2020) |
|---|---|
| **Subquery unnesting** | **~510× geomean** |
| Join ordering | ~7% |

Two orders of magnitude apart, in the opposite direction from where instinct puts the effort.
`join_order.rs` is built and correct; **the unnesting rules are the next optimizer
investment and are worth ~70× more.**

*Acceptance test.* A correlated `exists` MUST lower to a semi-join. A benchmark of the nested
and unnested forms MUST report the ratio. **Status: Specified — top priority.**

---

### E-opt-3 Verified schedules

**A schedule attached to a query MUST name only rewrites from the catalogue below, and each
MUST be applied where its stated side condition holds.**

**Status: Built** — `niles-ir::schedule`.

#### Why the earlier wording was unimplementable

This requirement previously read: *a schedule MUST be verified to preserve the algorithm's
denotation*. That cannot be built, and the architecture review recorded why rather than
treating it as difficult.

The operator set includes `Op::Negate` — Z-set negation, which is how `except` and outer-join
retraction are expressed — and `Op::Fixpoint`. **Equivalence of relational algebra with
difference is undecidable** (Trakhtenbrot; Abiteboul–Hull–Vianu §6.3). No engineering effort
produces a decision procedure for it, so `ROADMAP.md` Phase 5's kill criterion — "if it needs a
general theorem prover, narrow the language" — would have fired on the first day, because the
fragment was never fixed.

Narrowing does not rescue the original shape either. Equivalence of conjunctive queries is
NP-complete (Chandra–Merlin 1977) — acceptable for a checker, since queries are small, and
worth stating. But Z-sets carry weights, so the relevant semantics is **bag** rather than set:
bag-equivalence of conjunctive queries is graph-isomorphism-hard, and bag *containment* is
open. A verifier that answered "maybe" would have rebuilt the query hint it was meant to
replace.

#### The requirement as built

Turn the problem around. A schedule **names the rewrites that produced the plan**, drawn from a
finite catalogue whose members are individually proven equivalence-preserving under a stated
side condition. The checker verifies each side condition *syntactically*, in the IR, and
performs the rewrite itself. Equivalence is then true **by construction**.

| Rewrite | Side condition (syntactic) | Why it preserves denotation |
|---|---|---|
| `commute-join` | inner join, no residual predicate | `A ⋈ B = B ⋈ A` up to column order, which a compensating `Map` restores exactly |
| `push-filter-into-left` | inner join, predicate reads only left columns, no UDF | selection distributes over inner join on the side it mentions |
| `push-filter-into-right` | as above, right side; predicate is reindexed | as above |
| `commute-union` | exactly two inputs | Z-set union is addition in an abelian group |
| `elide-double-negate` | a `Negate` whose only input is a `Negate` | negation is an involution: `−(−z) = z` |

Anything else is refused with `ScheduleError::NotInCatalogue`. **The error type has no
`Unknown` variant**, which is the requirement's teeth: a checker that could answer "I could not
tell" would be a hint generator with extra steps.

#### What is deliberately absent

Rewrites whose soundness needs a *semantic* condition. Projection pushdown requires
functional-dependency inference; pushing a filter into an outer join's null-extended side is
simply false and is refused by name. Adding a rule means proving it and expressing its side
condition syntactically — the bar this design exists to impose.

#### Conformance

`cargo test -p niles-ir schedule` runs, per rule, a **1,000-trial denotation test**: the
original and rewritten circuits are evaluated by a reference Z-set interpreter on generated
inputs *including negative weights*, and must agree. Signed weights are not a detail — a
rewrite can be sound on sets, wrong on bags, and wrong again on Z-sets, and this IR has the
third semantics. Each rule also carries a negative control: the outer-join refusals, the
both-sides predicate, the residual join, the UDF predicate, the single negation.

#### What this is, as a contribution

Not a decision procedure for plan equivalence. **Query hints become checked rewrites.**
PostgreSQL has refused hints for twenty-five years and its objections reduce to the observation
that a hint is an unverified assertion the optimizer must trust. A catalogue step is not an
assertion — it is an operation whose precondition is checked before it is performed — so a
wrong schedule is rejected rather than silently obeyed. Nilestream is unusually well placed
because the IR verifier is already in the trusted base: this needed a new judgement, not a new
subsystem, and deliberately no SMT solver and no e-graph.

---

## Part IV — Concurrency and durability

### E-conc-1 Strict serializability

**The engine MUST provide strict serializability as an available isolation level, and MUST
name the level actually in force.**

*Prior art to engage directly.* **Materialize's default isolation level is strict
serializable**, over incrementally maintained views, with a published ladder. That is close
prior art for the consistency ladder and the thesis must engage it rather than claim novelty
for the ladder itself. The novelty is the ladder *over partial state*.

*The bounds this cannot escape.* **SNOW (OSDI 2016)**: strict serializability +
non-blocking + one-round reads + conflicting writes is impossible — binding directly on a
design serving fast reads from derived views while writes land. **Attiya–Welch**:
linearizability costs ~u/4 on reads and ~u/2 on writes under clock uncertainty *u*.

*Acceptance test.* An Elle-style cycle check SHOULD find no anomaly. **Planned: no such checker exists in this repository**, so this row is an acceptance criterion with no instrument, and §9.12 records strict serializability as not tested rather than as passed. **This matters even if the
conservation suite passes**, because a published analysis shows exactly that combination is
possible. **Status: Partial** — consensus and cross-shard commit are built and simulated;
neither has run over a network.

---

### E-conc-2 Durability, and the honest latency statement

**A commit MUST be durable before it is acknowledged. The engine MUST NOT claim a latency
that belongs to the storage device.**

*The 1000× swing that is not the database's:*

| | fsync latency |
|---|---|
| Enterprise NVMe with power-loss protection | **1.6–12.4 µs** |
| Without PLP | **891–2974 µs** |

Every durable-commit target is meaningless unless it names the storage class. This
specification therefore states commit latency **as a function of the device**, not as a
number.

*What the engine can improve, and does.* Group commit: an `fsync` costs the same for one
transaction or five hundred. Measured in this repository: the cost of durability *falls* as
concurrency rises — 5.7× at one thread, 4.8× at sixteen — with transactions per fsync rising
1.0 → 8.8.

*Acceptance test.* Crash recovery MUST truncate at the first bad record and MUST detect
tampering and splicing. **Status: Built** — `nilestream-ledger`, CRC-checked, hash-chained,
length-prefixed segments.

---

## Part V — The four workloads, simultaneously

### E-8 One engine, four classes

**Transactional, analytical, graph and streaming MUST be served from one semantics.**

*Status: open, not impossible.* No system in the survey scores first-class on even three.
The pieces exist separately — PAX-in-B-tree for HTAP, WCOJ for graph, DBSP for streaming —
and no theorem forbids the combination.

*The architecture that makes it one system rather than four:*

| Class | What it is here |
|---|---|
| **Transactional** | The ledger write path |
| **Analytical** | A REV over the ledger, served at a loose rung |
| **Streaming** | A REV over the ledger, served at a tight rung |
| **Graph** | WCOJ over the same relational core |

**Analytical and streaming differ in *rung*, not in *kind*.** That is the claim that makes
four workloads one engine rather than four subsystems, and it is the thesis's architecture
restated as an engine requirement.

*Acceptance test.* One schema MUST serve all four with no second execution path and no
second copy of the data. **Status: Partial** — the ledger, REV runtime and rungs are built;
columnar storage and WCOJ are not.

---

### E-1 Wire compatibility

**PostgreSQL and MySQL clients MUST connect and MUST receive correct results. There MUST NOT
be a second execution path.**

*The commitment.* A wire protocol is a **surface, not a semantics**. Two ways to compute an
answer is two answers that can disagree.

*The gap that compatibility does not close.* Compatibility is not semantics: a client
expecting PostgreSQL's exact `NULL` ordering, collation, or error codes will find
differences, and the specification's obligation is to *publish* the differences rather than
imply there are none.

*Acceptance test.* `psql` connects and queries. Money MUST be `NUMERIC`/`NEWDECIMAL`, never
a float. **Status: Built** — v3 simple and extended query paths, MySQL packet layer, TLS
negotiation with the cryptography delegated.

---

## Part VI — Conformance summary

| Area | Built | Partial | Specified | Adopt |
|---|---|---|---|---|
| Storage | 1 (E-3) | 0 | 2 | 1 (E-6) |
| Execution | 1 (E-exec-4) | 0 | 1 | 2 |
| Optimizer | 0 | 1 (E-7) | 2 | 0 |
| Concurrency | 1 (E-conc-2) | 1 (E-conc-1) | 0 | 0 |
| Workloads | 1 (E-1) | 1 (E-8) | 0 | 0 |

**Four built, three partial, five specified, three adopt.**

### What this specification refuses to claim

1. **A win on point lookups or selective indexed access.** PostgreSQL is at the bound.
2. **Better than 10× on durable strictly-serializable OLTP.** Silo's ceiling is bought with
   restrictions this engine cannot take.
3. **Any latency win on a single durable commit.** That is the storage device's number.
4. **An order of magnitude from SIMD, from compilation, or from join ordering.** 1.4×, a
   wash, and 7%.
5. **Optimality.** Refuted by Plan Bouquets; the claim is a bounded *k* ≥ 4.
6. **That the distributed protocols work.** They are simulated. Not one has run over a
   network, and the first contact with a real network finds something.
