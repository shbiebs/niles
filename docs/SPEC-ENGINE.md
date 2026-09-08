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

**Every ratio in this table is a same-session A/B on one instance, and is normative only as
that.** The `report` row read 2.68× MET on one instance of host class A and 2.17× NOT MET on
another; the three engine versions measured on the second instance were within 8% of each other,
and PostgreSQL itself ran 35% slower there. Both arms moved with the machine. A threshold sitting
inside the cross-instance variance of one host class is not a contract, so a figure quoted without
its host, instance, session, barrier and commit is not a claim this specification makes. **Host C**
is the reference (Apple M4, APFS on NVMe, `F_FULLFSYNC` at ~255 barriers/s); `docs/BENCHMARK.md`
gives the two-arm invocation, and every generated E16 document carries the header that makes the
comparison checkable.

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
aimed at, and the shortfall is reported rather than rounded: the remaining serialisation was
the single engine mutex the read path still shared, which the next paragraph removes. With
that mutex gone, the durable arm measured on its own at eight connections over five runs
reads **14,451 ops/s against 12,380** — the same repair helping a workload it was not aimed
at, because an append no longer waits behind a reader.

So the OLTP row is no longer bounded by one transaction per barrier, and on no host measured
so far is it bounded by the storage.

**The third factor was the engine lock, and it is gone.** The daemon took one mutex around
the whole of `Session::handle` — parse, plan, execute, frame — so a *read* excluded every
other read: two folds over a frozen prefix, which have nothing to say to each other, ran one
at a time. The cause was small and structural. `Base::reconstruct` took `&mut self` to
increment a row counter; that made every read of the base exclusive; that made
`Serving::query` exclusive; that made the mutex the only place to put it. The counter is now
an atomic, the base is behind a reader-writer lock taken **shared** by every read and
exclusively by every append, the read model has its own lock because a read of a partial view
installs into it, and `serve` acquires nothing at all.

Measured on the Linux container (two cores), medians of three runs, `bench --scaling-only
--nls-only`:

| workload | 1 conn | 2 | 4 | 8 |
|---|--:|--:|--:|--:|
| `fold` before | 164 (1.00×) | 270 (1.65×) | 227 (1.38×) | 209 (1.27×) |
| `fold` after | 165 (1.00×) | 278 (1.68×) | **259 (1.57×)** | **244 (1.48×)** |
| `point` before | 15,122 (1.00×) | 39,493 (2.61×) | 29,545 (1.95×) | 31,920 (2.11×) |
| `point` after | 15,857 (1.00×) | **79,814 (5.03×)** | **79,668 (5.02×)** | **57,479 (3.62×)** |

`fold` is an unkeyed `group by acct` the maintained view refuses — `is_full()` is false under
a budget below the key count — so it is the scan path every time, which is what makes it the
instrument for this. Two cores put the ceiling for a CPU-bound fold at about 2.0×, and 1.68×
at two connections is 84% of it; the figures at four and eight are the same two cores shared
more ways, not a fourth and eighth core doing nothing. **A two-core host cannot answer what
this change is worth on a wide one**, and the numbers above are not extrapolated to one.

The host-independent evidence is the lock counter. Four connections folding continuously,
`select nilestream_sealer`'s lock columns:

| | acquisitions | wait p50 | wait p99 | wait max | hold p50 |
|---|--:|--:|--:|--:|--:|
| before | 1,600 | ≤1 µs | ≤1,024 µs | 6,309 µs | ≤256 µs |
| after | 1,761 | ≤1 µs | **≤1 µs** | **2 µs** | ≤256 µs |

The hold is unchanged — the fold costs what it costs — and the wait fell by three orders of
magnitude, which is the whole claim: the folds were queueing, and they are not any more.
Waiting is now under one percent of counted request time, which closes the chunked-storage
question by measurement rather than by argument.

**Two locks means an order, and the first version of this had two.** A keyed read took the
read model and reached for the base underneath it; an append took the base and reached for the
read model underneath *that*. A point query and a concurrent insert could therefore each hold
what the other was waiting for. Nothing in the suite saw it, because every concurrency test in
the workspace drives one workload at a time — and no answer is ever wrong on the way into a
deadlock, so there is nothing for a correctness test to catch. The order is now stated and
tested: **base, then pending barriers, then read model, then currency set**, with every path
taking a subsequence of it. `rev_engine::lock_order_tests` holds it three ways — the source
order on both paths, the base acquired exactly once per read (an `RwLock` is not reentrant),
and four readers against a continuous appender under a deadline. On the inverted order that
last test hangs: two of five threads finish in thirty seconds.

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

### Durability, and what the record contains

**A durable append records its rows.** The payload is `parent ‖ hash ‖ canon(rows)` — the epoch's
rows in a canonical encoding, and the ledger's own chain link over them. `with_durable` replays
every recovered transaction through `Ledger::submit` in commit order and **verifies** the
recomputed link against the one the record carries, refusing to open at the first disagreement.

Three things this fixes, each of which was true of every `durable` figure this project has
published:

- The payload was `epoch.to_string()`. A restart recovered the idempotency window and **no rows**:
  eight writers, 25,416 acknowledged inserts, `SIGKILL`, reopen — the frontier was back at the seed
  and every account had no balance. A record that cannot reconstruct the base is a receipt.
- The ledger's chain and the segment's were two hashes over two different things. They now share
  one formula, `H(parent ‖ epoch ‖ canon(rows))`, and one byte string, so a replay can compare
  rather than rebuild. **A chain rebuilt on replay and compared to nothing protects nothing.**
- A record is a **batch** and a batch is many transactions, so a record's epoch is not a
  transaction's index. The k-th recovered transaction is ledger epoch `seed_epochs + k`; counting
  records would place every recovered row at the wrong epoch under group commit — which is to say
  under every workload the sealer exists for.

**A barrier failure is fail-stop** (LC-21). The sealer stops, every later submit is refused with
`ShuttingDown`, and the daemon serves no further appends until it is reopened and recovered. It
used to answer `Io` and take the next batch, which left epoch *n* applied to every reader's base
with no record on disk while *n+1* committed — and the visible frontier publishes with `fetch_max`
over contiguous epochs, so publishing *n+1* made *n* visible. On replay that hole is either a gap
or a failed checksum, and recovery stops there, losing every acknowledged epoch after it. The trade
is availability under a storage fault, for a prefix that is whole.

**A session does not observe the epoch it just wrote.** `append` returns the *applied* epoch,
before its barrier; the anchor is raised from `frontier()`, which moves only when a barrier
returns. Otherwise a failed barrier — correctly answered `58030` — left the session anchored past
the visible frontier, and its next read found the rows in the base and served them.

Measured on the executing host, same session, 16 connections × 300 transactions:

| | fsyncs | txns per barrier | wall |
|---|--:|--:|--:|
| before T-01 (`2afb56e`) | 972 | 4.94 | 0.434 s |
| after T-01 | 640 | **7.50** | 0.362 s |

The larger record did not cost throughput; it batches better, because a payload that takes longer
to hand over leaves a wider drain window for the next submitter to join.

**Two things the first cut of this got wrong**, both found by gates rather than by review, and
both worth recording because they are the same shape:

- *The epoch number was read off the ledger's length rather than passed in.* That is correct in
  `submit` — the epoch about to be pushed sits at the current length — and wrong in
  `verify_chain`, which walks a finished ledger where the length is the total count. Every link
  recomputed to a different digest, so E1's chain column read `FAIL` on all five seeds while the
  workspace stayed green: `verify_chain` had **no unit test**, and the only thing exercising it
  was a ten-minute experiment reporting through a generated table. `chain` now takes the epoch as
  an argument, and `ledger::chain_tests` checks an honest ledger at every length from one to six,
  a row altered after sealing, and two sealed epochs swapped. *A hash only its writer can
  reproduce verifies nothing.*
- *The chain built a `Vec` it did not need.* `canon(rows)` was materialised and handed to the
  hasher, at two allocations and ~132 B per epoch — enough to put `ledger_seeded` and
  `append_in_memory` over their E18 budgets the moment the chain became row-shaped. The rows now
  stream into the hasher through `write_rows`, and `encode_rows` — which the record genuinely
  needs — is written in terms of it, so the payload's bytes and the digest's bytes are the same
  bytes by construction. Both scenarios returned to exactly their pre-T-01 figures, so the
  verified chain costs no allocation at all.

### The schema's currency premise, at the wire

**A currency's code is its position among the schema's `currency` declarations, counting from
zero.** The wire carries a currency as an integer (`Cur = u32`) and the language declares one by
name, so something has to relate the two; this is the narrowest rule that invents no syntax, and
it is what the seeded base has always used. Order comes from the declaration's span rather than
from `HashMap` iteration, because a currency code is written into the ledger, where it is
permanent, and one that depended on hash order would differ between runs of the same binary on
the same schema.

**`MISMATCH-A9-F05`: that is true of the wire and false of the compiler.**
`session::declared_currencies` does sort by declaration span. `niles_lang::lower` does not: a
money literal's code is `cat.currencies.keys().position(..)` over a `HashMap<String,
CurrencyInfo>` with the standard hasher, so a compiled circuit names a currency by an order
that changes between processes. With one declared currency — every schema this project serves
today — the two agree at zero and nothing has seen it; with two, a `LitMoney` in the IR and
the same currency on the wire are different integers, and which ones depends on the run. The
rule above is the intended one; cycle 9's C9-04 makes the catalog carry the declaration index
so both consumers read it instead of deriving it.

Two obligations follow, and neither was discharged:

- **An `insert` naming an undeclared currency is refused** with `22023`
  (`invalid_parameter_value`), naming the currency and listing the declared codes. It used to
  answer `INSERT 0 2` for currency 999 against a schema declaring only `usd`. A currency with no
  declaration has no scale, so its amounts have no meaning, and `conserve per (txn, cur)` is
  vacuous over it. A schema that does not resolve refuses too: *unknown* is not *permitted*.
- **A `sum(amt)` that groups without `cur` over a base holding more than one currency is
  refused** with `22000` (`data_exception`) — not folded. It used to serve 324 USD + 500 of
  currency 999 as **"824"**, the balance of account 1. Adding two currencies is not a slow or
  imprecise answer; it is a number that denotes nothing. Both remedies are named in the error:
  add `cur` to the `group by`, or restrict with `cur = k`, and both still answer.

The refusal asks what the **base holds**, not what the schema declares — a schema may declare
three currencies over a base holding one, and refusing that fold would refuse the ordinary case
for a hypothetical. It sits on the fold rather than on the view because the fold is what the view
falls back *to*: `report_from_view` already refused this shape and fell through.

`explain` reports it as `refused-cross-currency`. A serve path is a claim about what the next
execution will do, and "it will refuse" is as much a fact as "it will fold"; promising `fold` for
a statement about to raise `22000` is the exact drift the one-function-decides rule exists to
prevent.

This is Contribution 4 on the **data** side. The compiler discharges "cannot mismatch currencies"
for Niles programs, and every benchmark row is data.

### The certification interval, and the answer that was thrown away

**A resident entry is exact at every anchor in `[stamp, effective]`, and `Rev::read` now says
so.** The entry holds the key's value as of `stamp` and received no delta between `stamp` and
`effective` — that is what `effective` means — so the value is unchanged across the whole closed
interval and is the correct answer at every anchor inside it. This is Theorem 4.1's own step (3);
nothing new is claimed here, and that is the point.

The read used to return `anchor: effective`: honest, and useless to its caller. `answer_from_view`
wants an answer true at the snapshot it was asked about, an answer stamped later includes writes
that snapshot excludes *as far as the caller can tell*, and so the engine discarded a correct
value and folded the base instead. The value was right the whole time; the engine could not tell,
because the read reported the wrong end of the interval.

Measured over the wire on a 10-core M4, three reader/writer shapes, 8 s per phase, durable sink,
10,000 accounts at budget 2,500:

| shape | fallback rate before | after | base rows folded |
|---|--:|--:|--:|
| 4 readers, 2 writers | 45.8% | **0.2%** | 20.6M → 7.9M (−61%) |
| 8 readers, 4 writers | 43.7% | **0.2%** | 42.9M → 16.6M (−61%) |
| 8 readers, 1 writer | 45.3% | **0.2%** | 20.8M → 1.7M (−92%) |

The residual 0.2% is measurement slop, not fallbacks: `view_answers` equals `reads` exactly in all
three shapes after the change.

**The "before" column is derived, and the derivation is worth stating because the counter did not
exist to take it directly.** Under the old counting a fallback incremented the view's `reads` *and*
the scan surface's `served`, and `read_stats` added them, so the reported total exceeded the
queries the probe actually issued by exactly the number of fallbacks. Issued is `reads/s × 8 s`
over the two read phases. After the change the excess is 0.2% and before it is 44–46%, on three
shapes independently — which is also a check on the derivation, since nothing forces those three
to agree.

The in-process test that gates this in CI reads 24.3% → 0.0% at 4r/2w, with 5,961 of 6,000 keyed
reads served from a resident entry and 39 reconstructed. That last check is the one that matters:
a fallback traded for a reconstruction would move the cost rather than remove it, and a rate alone
cannot tell those apart.

**It cost nothing in wall clock, and that is the finding rather than a disappointment.** Mixed-phase
read throughput moved −0.4%, +3.9% and +0.3% across the three shapes; read p50 is 23 µs before and
after. A keyed fallback folds *one account's* postings through the anchor index — about 22 rows —
against a wire round trip of roughly 47 µs, so it was some 2% of a read's cost and 46% of the
reads. Counted work fell by 61–92%; the wall clock could not see it. This is the counted-work
methodology earning its place: a wire benchmark run on this host would have reported the mechanism
as healthy while nearly half of its keyed reads bypassed it, which is precisely how F-27 survived
three audit cycles.

**The lower bound is not decoration, and it closed a correctness hazard rather than a cost one.**
`anchor < stamp` is a read *below* the entry: a delta landed in `(anchor, stamp]` that this anchor
must not see. The old condition served it — with `effective` attached — and was safe only because
every caller compared the two anchors and threw the answer away. A caller that trusted the value
would have been served the present at a historical anchor. `a_key_with_a_delta_after_the_anchor_still_reconstructs`
fails on the old condition by returning 500 where 100 is right, which is the shape of the bug that
was one careless caller away.

It also made `hits` a count of *answers* rather than of answers **served**. Those two numbers had
been reported as one since the runtime was wired to the wire path.

**What is counted now.** `select nilestream_stats` gains `view_answers` and `fallbacks` — the pair,
not a ratio, because they are counted at different places. `fallbacks` counts only the anchor
mismatch, not the shapes the view is never asked about, which are `serve_path`'s business.
`BLOCKED-fallback-rate` was raised against this twice: the behaviour was correct and the cost was
invisible, so an audit had to infer it from a latency distribution.

E19's `point` level is bracketed by the counters and reports a per-level rate, because the
counters are cumulative and a cumulative rate drifts towards a constant as a run goes on — the
opposite of what a connection sweep asks. The benchmark reads both columns **by name**; the reader
had been counting positions (`row.first()`, `row.get(2)`), which is correct exactly until a column
is added. A server that cannot be asked renders `n/a` and never `0.0%`: *no read fell back* and
*the question was not asked* are different claims, and only one of them is a result.

## Part III½ — The public surface, and who is downstream of it

**Two traits in this workspace are implemented outside it**, so a change to either is an API
change and needs a matching commit in the other repository recording this tree's SHA:

| trait | crate | out-of-tree implementor |
|---|---|---|
| `nilestream_core::rev::Base` | `nilestream-core` | `gbs-nilestream::JournalBase` (GBS) |
| `nilestream_server::session::Serving` | `nilestream-server` | none out of tree today. In tree: `RevEngine` itself, which is what **the daemon** serves every session from (`main.rs` builds an `Arc<RevEngine>`), and `RwLock<RevEngine>`, which is what **the benchmark's hosted daemon** uses because its sweep reseeds the engine between levels. Listed because any embedder implements it, and because the in-tree implementors are what a change to the trait actually breaks |

**A correction, cycle 10.** This row said `RwLock<RevEngine>` "is what the daemon serves every
session from". It is not: `main.rs` builds an `Arc<RevEngine>`, and the only `RwLock<RevEngine>`
in the workspace is the benchmark's, which needs the exclusive acquisition for `reseed`. The
distinction is not bookkeeping — it decides whether a wait inside `query` is taken under a
lock somebody else may need exclusively, which is A10-04. An implementor that wraps this trait
in a lock must run one `query_step` per acquisition and wait with the guard dropped; the
`RwLock` implementation shows the shape.

This section exists because the pair was broken for a cycle and neither gate could see it. T-06
turned `Base::reconstruct` and `deltas_at` from `&mut self` to `&self` — the right change, for a
measured reason — and GBS's adapter kept the old signature. This workspace never built the adapter;
GBS's gate was not run. Both were green.
`nilestream-core/tests/downstream_adapter.rs` now builds the adapter when a GBS checkout is
present (`GBS_ROOT`, or a sibling directory) and **skips by name** when it is not, so a log
distinguishes "the adapter is fine" from "nobody looked". It tells a compiler error apart from a
cargo failure by rustc's own `error[E` marker rather than by the exit code, which is the same
distinction T-03 drew in the verdict suites and for the same reason.

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

### E-conc-3 The keyed read holds the view for a decision, not for a reconstruction

**A read of a partially materialised view MUST NOT hold the view's lock across the
reconstruction it may need. The lattice's `Pending` state MUST be reachable, and what a
reader does when it finds one MUST be decided by the anchor.**

*What was built.* `Rev::read` took the view, tested the certification interval, missed, folded
the base **inside the hold**, installed and returned. Every other keyed read in the process
queued behind that fold whether or not it wanted the same key, and the hold had no bound.
`Slot::Pending` was constructed nowhere, and could not be: there was no moment at which a
second reader could arrive, because no second reader could run.

*What was measured, and what it refuted.* This requirement was written against LC-23's figures
— a mixed read throughput falling 94,038 → 48,310 → 20,507/s across 6r3w / 9r5w / 12r6w with a
view wait reaching 22,160 µs — which were **inferred from a document, not measured**. The
corrected two-arm run (`c9-pending.sh`, one daemon per replicate, histograms reset per level,
level-local counters, interleaved arms, 5 replicates) measures the *same predecessor build* at
**147,306 / 156,686 / 160,838** reads/s: rising with concurrency, 7.8× the collapsed figure at
twelve, slowest-read view wait 788 µs. There was no collapse. The two-phase read is worth
**+1.8%** at 12r/6w against a 1.25× pass line.

In the slowest-read tables of **both** arms, `base wait` is the dominant term and frequently the
whole of it. `answer_from_view` holds the base read guard across the fold — deliberately, since
`RwLock` is not reentrant and a keyed read must take the base exactly once — and a writer
arriving behind that guard queues, after which every later reader queues behind the writer. **The
tail is a base-lock tail.** LC-23's attribution to the view mutex is withdrawn and reopened
against the base. This requirement stands on the two grounds below, and **no performance claim
rests on it**.

*The protocol.* Three calls, and the lock is held for two of them.

| phase | lock | what it does |
|---|---|---|
| `begin_read(k, a)` | **V held** | hit test on `[stamp, effective]`; reap a cancelled flight; join, fold-alone, or take the flight and mark `Pending(a)` |
| the fold | **nothing** (the base guard the caller already holds) | `reconstruct(k, a)` over the frozen prefix |
| `finish_fold(t, v, rows)` | **V held** | install iff the view still holds this key's flight at this generation |

*The rules, normatively.*

1. A reader joins an in-flight reconstruction **iff its anchor is equal** to the flight's. A
   flight at another anchor folds a different prefix; joining it would serve an answer the
   reader's snapshot excludes.
2. A reader that does not own the key's flight **installs nothing**. Its answer is exact at
   its own anchor and is returned; the slot belongs to the owner.
3. `apply` **skips** a `Pending` slot. There is no value there to fold a delta into, and
   writing one epoch's delta as the key's value would publish a balance short by its entire
   history to every reader whose anchor lands in the resulting interval.
4. A flight that lands below the view's `applied` frontier installs **pinned**: it keeps its
   own anchor, does not inherit `applied`, and serves only reads at that anchor. Folding the
   intervening deltas into the landing value instead is sound and better, is reported as
   `deferred_merges`, and is not done here.
5. An install is conditioned on an unreused **generation**. A superseded completion answers
   its own caller and writes nothing: overwriting a newer resident entry moves its stamp
   backwards and clears a flight record it does not own.
6. Cancellation restores the **exact** slot the marker replaced, version and all. A `Hole`
   never degrades to `⊥`.
7. Flights and waiters are **bounded** (`MAX_FLIGHTS`, `MAX_WAITERS`). Over the bound a read
   folds alone, exactly, and `flights_refused` is incremented. There is no silent fold under
   the lock and no second path with different rules.

*The lock order gains one rung.* **O < B < P < V < C**, with a flight's completion condvar
**F strictly below all of them**, never held while anything is acquired. A joined reader
waits in the *caller*, holding nothing: `answer_from_view` returns the ticket rather than
resolving it, because waiting inside the read would park a thread under the base guard and
block every append in the process behind another reader's fold.

*Surfaces.* `select nilestream_stats` reports `pending_joins`, `uninstalled_folds`,
`pinned_installs` and `flights_refused`. `select nilestream_lockstats` reports `view_hold_us`
as the sum of the two holds and not the fold between them.

*The two grounds.* (i) The lattice's fourth state is reachable and counted — on the reference
host at 12r/6w the served daemon records a median 14,377 joins, 495 uninstalled folds and
311,921 pinned installs, so the chapter that describes joining describes something the engine
does. (ii) The hold is bounded: a keyed read owns the view for a decision and an install, never
for a reconstruction, and that is a structural property rather than a measured one.

*And one measured ratio that is a finding.* Against ~4.9 M reads in a 30-second level, joins are
0.29% and pinned installs 6.35% — twenty-two reconstructions land pinned for every one shared.
A pinned entry serves one anchor and is rebuilt by the next reader that needs the key fresh. The
deferred merge (rule 4 above, `deferred_merges`, zero today) is worth considerably more than the
join, and this is the evidence for it.

*Acceptance test.* The latch-driven differential
(`rev::two_phase::a_flight_that_overlaps_an_advance_is_joined_shared_and_installed_pinned`)
constructs the interleaving rather than waiting for it, and asserts the precondition
`pinned_installs + deferred_merges + pending_joins > 0`. **Status: Complete.** The protocol is
implemented, guarded and measured at the wire; the throughput claim it was expected to carry did
not survive its own measurement and has been withdrawn rather than restated
(`docs/audit/cycle-9/hostc/c9-pending-results.md`).

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

#### The record, and the three things recovery can conclude

A record is, little-endian:

```
len: u32 | len_check: u32 | batch_seq: u64 | parent: [u8;32] | hash: [u8;32] | payload | crc: u32
```

`len_check = !len ^ 0xA5A5_A5A5`, and it is the version marker as well as the check: a segment
written before cycle 9 has the batch sequence number's low four bytes where the check belongs,
fails it, and is refused by name rather than misread. `crc` covers the body — everything after
the eight-byte header — and `hash` is the chain link over `parent ‖ batch_seq ‖ payload`.
`batch_seq` is the record's sequence number in this segment and **not** a ledger epoch: one
record is a batch of up to 4,096 transactions, and reading it as an epoch is how two
idempotency windows came to count different things (A9-F02).

Recovery classifies the first thing it cannot read into exactly one of three, and the
classification decides what may be discarded:

| what it found | what it means | what happens |
|---|---|---|
| fewer than 8 bytes of header, or a body that runs past the end of a validated length | a crash caught mid-write | trimmed: a torn tail, and nothing beyond it can exist |
| a header whose `len_check` fails | **damage** — the length says nothing | refused, and nothing is truncated, unless fewer bytes remain than the smallest possible record (84), in which case no committed record can be among them and the tail is trimmed |
| a bad `crc`, an out-of-order `batch_seq`, or a broken chain link, with further bytes behind it | damage in the middle | refused, and nothing is truncated |

The middle row is the one that was missing. The checksum does not cover the length, so a single
flipped bit in a length prefix made the record look longer than the file; that was classified as
a short tail, the damaged prefix was then asked whether anything followed it, and its own
corrupted answer said no. A ten-record segment with one bit flipped in record zero reopened
**empty, with a success code** — ten acknowledged, fsynced epochs discarded. The corpus in
`crates/nilestream-ledger/tests/torn_corpus.rs` is exhaustive over every byte offset and every
bit of a ten-record fixture and asserts the property directly: *refuse, or recover exactly what
was acknowledged; never a prefix with a success code.*

**`LC-36` / `BLOCKED-recovery-tip`.** One class remains and cannot be closed from inside the
file. Damage to the **last** record is indistinguishable from a crash during its write: both
leave a record that does not verify with nothing after it, and trimming is the correct response
to one and data loss under the other. 856 of the corpus's 8,840 single-bit flips fall in that
record and are trimmed. Deciding them needs something the segment does not contain — a
clean-close marker written at shutdown, or a tip retained outside the segment and compared on
open. Which of those, and what an operator does when they disagree, is the author's contract to
give; until then the corpus asserts the boundary rather than pretending it is closed.

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
