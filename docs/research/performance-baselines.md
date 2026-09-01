# Performance Baselines for Nilestream

**Purpose.** Establish citable, primary-source performance numbers against which Nilestream's
acceptance criteria can be set. Every figure below is tagged with a verification status.

**Compiled:** 2026-09-01

## Verification legend

| Tag | Meaning |
|---|---|
| **[V]** | Verified against the primary source (peer-reviewed paper, official result, or raw data I fetched and recomputed). |
| **[V-self]** | Verified in the primary source, but the source is the system's own authors comparing against their own prior system. |
| **[V-vendor]** | Verified as *stated*, but the source is a vendor blog with a commercial interest. Treat as an upper bound on the claim, not a measurement. |
| **[M]** | Measured by me in this report from published raw data. Reproduction steps given. |
| **[U]** | **Could not verify.** The figure is either absent from the cited source or the source does not exist as described. Do not cite. |

---

## 1. Summary table: the ratios that matter

| # | Ratio | Value | Source | Status |
|---|---|---|---|---|
| 1 | Umbra vs HyPer, Join Order Benchmark, geomean | **3.0×** | Neumann & Freitag, CIDR 2020 §4.1 | **[V-self]** |
| 2 | Umbra vs HyPer, TPC-H SF10, geomean | **1.8×** | Neumann & Freitag, CIDR 2020 §4.1 | **[V-self]** |
| 3 | Umbra vs MonetDB, JOB / TPC-H geomean | **4.6× / 2.3×** | Neumann & Freitag, CIDR 2020 §4.1 | **[V-self]** |
| 4 | DuckDB vs PostgreSQL-**indexed**, ClickBench, geomean of per-query hot ratios | **11.5×** (median 14.5×; **Postgres wins 10 of 43 queries**) | Recomputed from ClickBench raw JSON, c6a.4xlarge | **[M]** |
| 5 | DuckDB vs PostgreSQL-**unindexed**, same data | 1091× | Same | **[M]** — *see §8.1; this is a worst-case Postgres config, not a real ratio* |
| 6 | CedarDB vs DuckDB, ClickBench, geomean | **2.8×** (DuckDB wins 2 of 43) | Recomputed from ClickBench raw JSON | **[M]** |
| 7 | ClickHouse vs DuckDB, ClickBench, geomean | **1.76×** (DuckDB wins 5 of 43) | Recomputed from ClickBench raw JSON | **[M]** |
| 8 | Compiled (Typer) vs vectorized (Tectorwise), TPC-H SF1 | **0.66×–1.93×** — neither dominates | Kersten et al., PVLDB 11(13) 2018 | **[V]** |
| 9 | SIMD (AVX-512) benefit on TPC-H Q6 | **1.4×** — *this is the "1.4×" you had; it is SIMD, not vectorization-vs-compilation* | Kersten et al. 2018 §6 | **[V, reattributed]** |
| 10 | SIMD benefit on TPC-H join queries (Q3, Q9) | **~1.1×** (i.e. ~none) | Kersten et al. 2018 §6 | **[V]** |
| 11 | LLVM codegen latency, typical TPC-H query | **40–90 ms** | Neumann, PVLDB 14(12) 2021 | **[V]** |
| 12 | Direct x86 codegen latency, same queries | **1–2 ms** | Neumann, PVLDB 14(12) 2021 | **[V]** |
| 13 | Umbra Flying Start vs LLVM -O3 | **108× faster compile, 1.2× slower execution** | Kersten et al., VLDB J. 30 (2021) Table 3 | **[V-self]** |
| 14 | HyPer bytecode interpreter vs LLVM -O3 execution | **4.1× slower** | Kersten et al., VLDB J. 30 (2021) Table 3 | **[V-self]** |
| 15 | fsync latency, enterprise NVMe **with** power-loss protection, 16 KB | **1.6 µs – 12.4 µs** | Callaghan, fio measurement, 2026-01 | **[V]** |
| 16 | fsync latency, consumer NVMe **without** PLP, 16 KB | **891 µs – 2 974 µs** | Same | **[V]** |
| 17 | PostgreSQL 18.1 TPROC-C, 48-core EPYC, **fsync off** | **236 986 NOPM** (wh=4000); **431 648 NOPM** (wh=1000) | Callaghan, HammerDB, 2026-02 | **[V]** |
| 18 | MySQL 8.4.8 on same box | 229 511 NOPM (wh=4000); 435 095 (wh=1000) | Same | **[V]** |
| 19 | Silo, TPC-C, 32 cores, **with** logging | **~700 000 txn/s** (~750 000 without logging) | Tu et al., SOSP 2013 | **[V]** |
| 20 | HyPer, TPC-C, single thread / 5 threads | **126 576 / 380 868 txn/s** total mix | Kemper & Neumann, ICDE 2011 Fig. 9 | **[V]** |
| 21 | Noria essential state vs full materialization | **9 %** (73 MB of 789 MB, Lobsters) | Gjengset et al., OSDI 2018 §8.4 | **[V]** |
| 22 | Noria vs MariaDB, Lobsters, 16 vCPU, sub-100 ms p95 | **5×** (5 000 vs 1 000 pages/s) | Gjengset et al., OSDI 2018 §8.1 | **[V]** |
| 23 | WCOJ (Umbra) vs EmptyHeaded, 4-clique, small graphs | **4.5×–6.9×** | Freitag et al., PVLDB 13(12) 2020 Table 2 | **[V]** |
| 24 | WCOJ regression on TPC-H / JOB | **None** — "no slowdown over unmodified Umbra" | Freitag et al. 2020 §5.2.1 | **[V]** |
| 25 | "77× on 4-clique" | **Does not appear in the paper** | — | **[U] Do not cite** |
| 26 | Tuple-at-a-time interpretation: fraction of time doing real work | **<10 %** (62 % record navigation/copying, 28 % hashing) | Boncz et al., CIDR 2005 Table 2 | **[V]** |
| 27 | Audited TPC-C record | **707 M tpmC on 1 557 servers** (~455 k tpmC/node) | Xu et al., PVLDB 15(12) 2022 | **[V]** |

---

## 2. TPC-H / analytical baselines

### 2.1 The single most important structural fact

**No open-source database has ever published an audited TPC-H result.** I checked the TPC-H
results register directly. The systems present are EXASOL, Microsoft SQL Server, Alibaba
Hologres, StarRocks, OceanBase, Altibase, and Ant Group Explorer. PostgreSQL, MySQL, DuckDB and
ClickHouse are all absent.
([TPC-H results register](https://www.tpc.org/tpch/results/tpch_results5.asp?version=4)) **[V]**

Consequence for the spec: **there is no authoritative PostgreSQL TPC-H number to target.** Every
PostgreSQL TPC-H figure in circulation is an unaudited community or vendor measurement. The spec
must therefore define its own harness and publish it, rather than cite a "PostgreSQL TPC-H
baseline" as if one existed.

For scale context, the top audited figures are 43 903 QphH @100 GB (Altibase), 6 145 628 QphH
@1 TB (EXASOL 6.2, HPE DL325 Gen10), and 54 803 403 QphH @100 TB (Ant Group Explorer, cluster).
These are cluster results with heavy price/performance engineering and are not a meaningful
single-node target.

### 2.2 PostgreSQL vs DuckDB — measured on identical hardware

Because no peer-reviewed head-to-head exists at the scale factors you asked for, I recomputed the
ratio myself from ClickBench's published raw result files, which have the virtue of being run on
**the same AWS instance type (c6a.4xlarge, 16 vCPU, 32 GB)** for every system.

Method (reproducible):

```bash
git clone --depth 1 https://github.com/ClickHouse/ClickBench.git
# hot time per query = min(run2, run3), per ClickBench's own rule
# systems: postgresql/20250310, postgresql-indexed/20250310,
#          duckdb/20260511, clickhouse/20260901, cedardb/20260815,
#          mysql/20250711  — all results/<snapshot>/c6a.4xlarge.json
```

| System | Sum of 43 hot query times | Geomean per query | Load time | On-disk size |
|---|---|---|---|---|
| PostgreSQL (no indexes) | 11 895.9 s | 273.8 s | 937 s | 106.5 GB |
| PostgreSQL (indexed) | 4 085.5 s | 2.88 s | 10 357 s | 124.4 GB |
| MySQL | 21 962.3 s | 206.7 s | 10 004 s | 94.4 GB |
| DuckDB | 26.25 s | 0.251 s | 126 s | 20.5 GB |
| ClickHouse | 18.09 s | 0.142 s | 269 s | 15.3 GB |
| CedarDB | 28.99 s | 0.090 s | 187 s | 8.5 GB |

**[M]**

Geomean of per-query speedups (only meaningful comparison is against the *indexed* Postgres):

| Comparison | Geomean | Median | Min | Max | Queries where Postgres wins |
|---|---|---|---|---|---|
| DuckDB vs PostgreSQL-indexed | **11.5×** | 14.5× | 0.01× | 4087× | **10 / 43** |
| DuckDB vs PostgreSQL-unindexed | 1091× | 762× | 41× | 9233× | 0 / 43 |

**This bimodality is the single most important result in this document.** Against a properly
indexed PostgreSQL, DuckDB's advantage is roughly one order of magnitude *in the geometric mean*,
but the distribution spans five orders of magnitude in both directions. Postgres with a usable
index beats DuckDB outright on 10 of 43 queries — those are the point-lookup and highly selective
queries where B-tree access dominates and columnar scanning is pure overhead. DuckDB's 4087×
maximum is on queries where Postgres has no usable index and must scan 100 M rows row-wise.

The often-quoted three-and-four-digit "DuckDB is 1000× faster than Postgres" figure is an artifact
of comparing against a Postgres instance with no indexes at all. See §8.1.

### 2.3 Umbra / CedarDB vs HyPer and MonetDB — the strongest academic ratios

From Neumann & Freitag, *Umbra: A Disk-Based System with In-Memory Performance*, CIDR 2020, §4.1
([PDF](https://db.in.tum.de/~freitag/papers/p29-neumann-cidr20.pdf)) **[V-self]**:

- **JOB: Umbra 3.0× geomean over HyPer.** Confirmed — your figure is correct.
- **TPC-H SF10: Umbra 1.8× geomean over HyPer.**
- JOB: Umbra 4.6× geomean over MonetDB; TPC-H 2.3×.
- Buffer manager overhead: **<2 % on average**, **<6 %** when bypassing the buffer manager
  entirely. Read throughput 1.13 GiB/s with buffer manager vs 1.15 GiB/s without.
- Hardware: Intel i7-7820X, 8 physical / 16 logical cores @3.6 GHz, 64 GiB RAM, Samsung 960 EVO.
- **The paper contains no PostgreSQL comparison at all.** There is no Umbra-vs-PostgreSQL ratio in
  the primary literature.

Caveat: CIDR is a vision/short-paper venue with lighter review than PVLDB, and this is TUM
comparing Umbra against TUM's own HyPer. The 3.0× is credible but it is a self-comparison.

### 2.4 Cross-system single-threaded TPC-H SF10

Dreseler et al., *Quantifying TPC-H Choke Points and Their Optimizations*, PVLDB 13(8) 2020
([PDF](https://www.vldb.org/pvldb/vol13/p1206-dreseler.pdf)) **[V]** evaluate Hyrise against
HyPer, MonetDB, DuckDB and Umbra at SF10, single-threaded, on a Xeon 8180 NUMA node. This is the
cleanest peer-reviewed multi-system TPC-H comparison available — but note it does **not** include
PostgreSQL.

Its most useful result for a spec is not the ranking but the *decomposition*: which optimizations
actually buy the speedups (§4 below).

### 2.5 DuckDB absolute anchors

Useful for sanity-checking your own harness:

- **TPC-H SF100, total runtime:** 166.2 s on AWS r6id.xlarge (4 vCPU, 32 GB); 570.8 s on r6id.large
  (2 vCPU, 16 GB); 235.0 s on a Samsung Galaxy S24 Ultra.
  ([DuckDB blog, 2024-12-06](https://duckdb.org/2024/12/06/duckdb-tpch-sf100-on-mobile)) **[V-vendor]**
  — vendor-published, but the configuration is fully disclosed and the claim is unflattering
  (it's a stunt about running on phones), so bias risk is low.
- **TPC-H SF1000 (265 GB), geomean per query:** 12 s on a 2023 M3 Max MacBook Pro; 218 s on a 2012
  MacBook Pro — a ~20× improvement over a decade, of which ~7× is raw CPU.
  ([DuckDB, *The Lost Decade of Small Data?*](https://duckdb.org/2025/05/19/the-lost-decade-of-small-data)) **[V-vendor]**

### 2.6 PostgreSQL absolute anchors (community, unaudited)

From the [pgtpc](https://github.com/Vonng/pgtpc) community repository **[V-vendor / community]**:

| Scale factor | Time | Hardware |
|---|---|---|
| SF1 | 8 s | Apple M1 Max, 10 cores, 64 GB |
| SF10 | 56 s | Apple M1 Max, 10 cores, 64 GB |
| SF50 | 1 327 s | Apple M1 Max, 10 cores, 64 GB |
| SF100 | 4 835 s | Apple M1 Max, 10 cores, 64 GB |
| SF1 | 13.51 s | z1d.2xlarge, 8 cores, 64 GB |
| SF10 | 133.35 s | z1d.2xlarge, 8 cores, 64 GB |

Note the superlinear blow-up from SF10 → SF50 → SF100 (56 s → 1 327 s → 4 835 s): this is
PostgreSQL falling out of memory and into disk-bound hash joins. That knee is real and is a
legitimate target — but it is a *memory-hierarchy* result, not an execution-engine result.

**Do not construct a "PostgreSQL SF100 = 4835 s vs DuckDB SF100 = 166 s → 29×" ratio from these
two rows.** Different hardware, different thread counts, unaudited, and one is a vendor post. If
you want that ratio, measure it yourself.

---

## 3. TPC-C / transactional baselines

### 3.1 The realistic single-node PostgreSQL ceiling

The best available primary measurement is Mark Callaghan's HammerDB TPROC-C runs (he is a
long-standing, independent, methodologically careful benchmarker; these are blog posts but they
are original measurements with disclosed configuration).

**Hardware:** Hetzner 48-core AMD EPYC 9454P, SMT disabled, 128 GB RAM, 2× Intel D7-P5520 NVMe in
RAID 1, ext4, Ubuntu. **fsync on commit disabled** for both systems.
([Small Datum, 2026-02](http://smalldatum.blogspot.com/2026/02/hammerdb-tproc-c-on-large-server.html)) **[V]**

| Config | PostgreSQL 18.1 | MySQL 8.4.8 |
|---|---|---|
| vu=40, wh=4000 (largest, most I/O) | **236 986 NOPM** | 229 511 NOPM |
| vu=40, wh=1000 (fits in memory) | **431 648 NOPM** | 435 095 NOPM |

Callaghan's summary: *"Postgres and MySQL have similar throughput for the largest warehouse count
(wh=4000). Otherwise Postgres gets between 1.4X and 2X more throughput (NOPM)."* And on
efficiency: at wh=4000, *"MySQL 8.4.8 uses about 2X more CPU per transaction and does more than 2X
more context switches per transaction compared to Postgres 18.1."*

**So: the realistic single-node PostgreSQL ceiling on a 48-core server is ~230 k–430 k NOPM with
durability turned off.** With durability on, it is bounded by fsync latency and group-commit
batching (§5.3).

Version-over-version, PostgreSQL has been roughly flat: *"There are small regressions in versions
16, 17 and 18"* with a small improvement in 19 beta1 relative to 18
([Small Datum, 2026-06](http://smalldatum.blogspot.com/2026/06/hammerdb-tproc-c-on-large-server.html)) **[V]**.
This matters for your spec: **you are not chasing a fast-moving target on the OLTP side.**

### 3.2 What the research systems claim, and the caveats

| System | Claim | Caveats disclosed by the authors |
|---|---|---|
| **Silo** (Tu et al., SOSP 2013, [PDF](https://wzheng.github.io/silo.pdf)) **[V]** | ~700 k txn/s at 32 cores **with** logging; ~750 k without (only 1.16× gap) | In-memory only; *"our clients do not currently use the network"* — authors estimate network would cost ~23 %; *"our performance is higher than a full system would observe"*; no checkpointing or recovery implemented; no client think time; #warehouses = #workers with NUMA-local assignment. Beat a commercial main-memory DB doing ≤3 000 txn/s/core on the same hardware. |
| **HyPer** (Kemper & Neumann, ICDE 2011, [PDF](https://cs.brown.edu/courses/cs227/archives/2012/papers/olap/hyper.pdf)) **[V]** | 126 576 txn/s single-threaded; 380 868 txn/s with 5 OLTP threads (full 5-transaction mix) | Only **12 warehouses / ~1 GB** database — fits entirely in cache-friendly memory; workload is partitionable by warehouse; redo logging to a storage server was on. VoltDB comparison numbers were *"extracted from the product overview brochure"*, not measured, and were for a 6-node cluster vs HyPer's single server. |
| **VoltDB** | ~1.6 M txn/s at >300 cores across a 39-server projection | The widely cited analysis ([Percona/Schwartz](https://www.percona.com/blog/is-voltdb-really-as-scalable-as-they-claim/)) **[V]** is a *Universal Scalability Law extrapolation*, not a measurement at that size; the author states *"I did not audit or repeat Tim's benchmarks."* The workload is the "voter" benchmark, not TPC-C. Single-partition-heavy workloads only. |
| **CedarDB** | — | **I found no published CedarDB TPC-C or CH-benCHmark throughput number.** They document how to *run* CH-benCHmark but do not publish results. **[U] — do not cite a CedarDB OLTP figure.** |

### 3.3 Audited tpmC is a different unit — do not mix it in

The audited TPC-C record is **707 million tpmC** on **1 557 servers** (~130 728 cores, ~455 k
tpmC/node, ~5 400 tpmC/core), OceanBase, PVLDB 15(12) 2022
([PDF](https://vldb.org/pvldb/vol15/p3385-xu.pdf)) **[V]**.

Audited tpmC differs from HammerDB NOPM in ways that make the numbers non-comparable:

- **Keying and think time are mandatory.** Each emulated terminal idles for seconds between
  transactions. OceanBase used **559 440 000 emulated terminals** to reach 707 M tpmC. A HammerDB
  run with 40 virtual users and no think time is a completely different regime.
- **Database size scales with throughput** (55 944 000 warehouses here) — you cannot keep the
  working set artificially small.
- **8-hour steady-state measurement** with <2 % throughput variance required (they achieved <1 %).
- **Full ACID durability and replication** are audited requirements — here, three replicas
  (full + data + log) across three zones with Paxos.

HammerDB itself is explicit that TPROC-C results *"can only be used for official audited TPC-C
benchmarks"* if audited, and that *"if a benchmark claiming a tpmC metric has not been audited and
approved by the TPC-Council, then it is invalid."*
([HammerDB blog](https://www.hammerdb.com/blog/uncategorized/how-to-understand-tpc-c-tpmc-and-tproc-c-nopm-and-what-is-good-performance/)) **[V]**

**Spec guidance:** report NOPM as NOPM. Never write "tpmC" for a number you did not have audited.

---

## 4. Join Order Benchmark

### 4.1 The original: Leis et al., PVLDB 9(3) 2015

[PDF](https://www.vldb.org/pvldb/vol9/p204-leis.pdf) **[V]**

Dataset: real IMDB data, 3.6 GB as CSV; `cast_info` 36 M rows, `movie_info` 15 M rows.
**113 queries** across 33 query structures. Hardware: 2× Xeon X5570 (8 cores total), 64 GB RAM,
`work_mem=2GB`, `shared_buffers=4GB`, PostgreSQL 9.4.

Headline findings:

- **PostgreSQL cardinality estimation degrades fast with join count:** *"16 % of the estimates for
  1 join are wrong by a factor of 10 or more. This percentage increases to 32 % with 2 joins, and
  to 52 % with 3 joins."*
- **Base-table selection estimates, 95th percentile error factor** (Table 1): PostgreSQL 6.10,
  DBMS A 1.98, DBMS B 30.2, DBMS C 5367, HyPer 8.00.
- *"For all systems we routinely observe misestimates by a factor of 1000 or more"* on joins.
- **The cost model barely matters compared to cardinality.** *"Our tuned cost model yields 41 %
  faster runtimes than the standard PostgreSQL model, but even a simple [main-memory cost model]
  makes queries 34 % faster."* Contrast with Lohman's quote the authors endorse: cardinality
  errors are *"many orders of magnitude"* while cost model errors are *"at most 30 %."*
- **Join enumeration algorithm matters much less than you'd think** (Table 3, normalized to
  optimal, using PostgreSQL estimates): dynamic programming median 1.03 / max 4.79; Quickpick-1000
  median 1.05 / max 7.29; Greedy Operator Ordering median 1.19 / max 2.36.
- With true cardinalities and nested-loop joins disabled, *"less than 4 % of the queries are off by
  more than 2×."*

**This is the single most spec-relevant result in the whole document for a query optimizer:
exhaustive join enumeration is nearly worthless if your cardinality estimates are bad, and nearly
unnecessary if they are good.**

### 4.2 The 2025 retrospective — contains no new numbers

Leis & Neumann, *Still Asking: How Good Are Query Optimizers, Really?*, PVLDB 18
([PDF](http://www.vldb.org/pvldb/vol18/p5531-viktor.pdf)) **[V]**

I fetched and checked this twice. **It is a retrospective essay and contains no new experimental
measurements.** It does not benchmark PostgreSQL against DuckDB, Umbra or CedarDB. Its substantive
claims are qualitative: progress over the decade has been limited; learned cardinality estimators
*"have yet to see widespread adoption in industry"*; the promising direction is a *"combination of
static and run-time optimization."*

**Do not cite this paper for numbers.** It is worth citing for the claim that the 2015 problem is
still open.

### 4.3 The PostgreSQL-vs-modern JOB comparison you wanted does not exist in the literature

I could not find a peer-reviewed head-to-head of PostgreSQL against Umbra/CedarDB/DuckDB on JOB.
The Umbra CIDR paper compares only against HyPer and MonetDB. **If you need this ratio, it is an
experiment you must run.** Note that ~10 % of JOB queries reportedly failed to complete in
reasonable time on PostgreSQL 9.4 due to estimation errors, so a modern rerun would need a
timeout policy and would need to state it.

---

## 5. Micro-costs a spec should target

### 5.1 Interpretation overhead — the classic decomposition

Boncz, Zukowski & Nes, *MonetDB/X100: Hyper-Pipelining Query Execution*, CIDR 2005 **[V]**

TPC-H Q1, SF1, on a 1533 MHz AthlonMP:

| System | Q1 time |
|---|---|
| MySQL 4.1 | 26.6 s |
| "DBMS X" | 28.1 s |
| MonetDB/MIL | 3.7 s |
| MonetDB/X100 | 0.50 s |
| Hand-coded baseline | 0.22 s |

The important number is not the 53× — it is the **profile decomposition (Table 2)**: in MySQL's
tuple-at-a-time engine, the five operations that do the actual arithmetic account for **<10 % of
total execution time**; ~62 % goes to record navigation and data copying and ~28 % to hash-table
operations. A single addition (`Item_func_plus::val`) costs **38 instructions at IPC 0.8**
(~49 cycles).

**Caveat: this is 2005 hardware and MySQL 4.1.** The *structural* claim (interpretation dominates
in a tuple-at-a-time engine) is still true and is the standard citation. The *magnitude* (53×)
should not be presented as a current PostgreSQL-vs-compiled ratio. Modern PostgreSQL has
expression JIT and much better executor code.

### 5.2 Compiled vs vectorized — and a correction to your figure

Kersten, Leis, Kemper, Neumann, Pavlo, Boncz, *Everything You Always Wanted to Know About Compiled
and Vectorized Queries But Were Afraid to Ask*, PVLDB 11(13) 2018 **[V]**

The authors built two engines — **Typer** (data-centric compiled) and **Tectorwise** (vectorized) —
sharing algorithms and data structures, so the comparison isolates the execution paradigm.

TPC-H SF1, single-threaded, Intel i9-7900X:

| Query | Typer (compiled) | Tectorwise (vectorized) | Winner |
|---|---|---|---|
| Q1 | 44 ms | 85 ms | Typer 1.93× |
| Q3 | 47 ms | 44 ms | Tectorwise 1.07× |
| Q6 | 15 ms | 15 ms | tie |
| Q9 | 126 ms | 111 ms | Tectorwise 1.14× |
| Q18 | 90 ms | 154 ms | Typer 1.71× |

The paper's own summary: *"the relative performance ranges from Typer being faster by 74 % (Q1) to
Tectorwise being faster by 32 % (Q9)"* and *"these are not large differences."* Conclusion:
*"neither paradigm is clearly dominated by the other which makes both viable options."*

Microarchitectural mechanism (Table 1, per tuple): Tectorwise executes **up to 2.4× more
instructions** and more L1 misses, but achieves higher IPC (Q3: 1.8 vs 0.8; Q9: 1.3 vs 0.6) because
its simple loops let the out-of-order engine hide memory latency. Compiled wins on
compute-bound queries (values stay in registers); vectorized wins on memory-bound ones.

Vectorized interpretation overhead is essentially nil: **<1.5 % of runtime is in the interpreted
part; 98.5 % is in the primitives.**

Multi-threaded at SF100 on 10 cores/20 threads, the gap *narrows further*.

> ### Correction to the "1.4× on TPC-H Q6" figure
>
> **The 1.4× is the AVX-512 SIMD speedup on Q6, not a compiled-vs-vectorized or
> vectorization-vs-scalar-engine result.** In the Typer-vs-Tectorwise comparison Q6 is a **tie**
> (15 ms vs 15 ms).
>
> The paper's SIMD results (§6):
>
> | Measurement | Speedup |
> |---|---|
> | Selection microbenchmark, dense input | 8.4× |
> | Selection with selection vectors (sparse) | 2.7× |
> | Hash-join hashing in isolation | 2.3× |
> | **TPC-H Q6, realistic query** | **1.4×** |
> | Gather (sparse memory access) | 1.1× |
> | **TPC-H Q3 / Q9 (join queries)** | **~1.1× — essentially nothing** |
>
> Authors' conclusion: *"For the more complicated TPC-H queries, the performance gains are quite
> small (around 10 % for join queries)"* — *"most OLAP queries are bound by data access, which
> does not (yet) benefit much from SIMD."*
>
> **Spec implication: do not budget a large win from hand-vectorizing operators.** The measured
> return on real queries is 10–40 %, and it collapses once working sets exceed cache.

### 5.3 Compilation latency

| Backend | Latency | Source |
|---|---|---|
| LLVM codegen, typical TPC-H query | **40–90 ms** | Neumann, PVLDB 14(12) 2021 **[V]** |
| Direct x86-64 generation, same queries | **1–2 ms** | Neumann, PVLDB 14(12) 2021 **[V]** |
| Umbra total query preparation, TPC-H SF0.01, geomean | **0.66 ms** (plan 0.25 + codegen 0.20 + x86 0.21) | Kersten et al., VLDB J. 30 (2021) Table 2 **[V-self]** |
| HyPer total preparation, same | 1.33 ms | Same |
| **Pathological query (2 000 joins, 108 k IR instructions)** | Flying Start **<0.04 s**; LLVM FastISel **4 s**; LLVM standard **150 s** | Same **[V-self]** |
| Stress test (10 000 joins) | Umbra ~5 s total; **LLVM does not terminate within two hours** | Neumann 2021 **[V]** |

**Execution cost of the fast tier:**

- Umbra Flying Start vs LLVM -O3: **108× faster compilation, 1.2× slower execution.**
- HyPer's bytecode interpreter vs LLVM -O3: **4.1× slower execution.**
- LLVM -O0 vs -O3: 1.3× slower execution.

**This is the cleanest design guidance in the document.** A bytecode-first tier costs you ~20 %
execution speed (Umbra's own IR) or ~4× (a naive bytecode interpreter), and buys you two orders of
magnitude on compile latency. For a system serving many short queries, that trade is
overwhelmingly correct. The crossover is roughly: if a query runs longer than ~100 ms, LLVM's
40–90 ms is worth paying; below that it is not.

A third-party observation puts CedarDB's compilation overhead at **50–100 ms per query**
([Tinybird](https://www.tinybird.co/blog/clickhouse-vs-cedardb)) **[V-vendor, competitor]** — note
the source is a managed-ClickHouse vendor, so treat as a hostile estimate rather than a
measurement.

### 5.4 fsync cost on modern NVMe

Callaghan's fio measurements (O_DIRECT, 1 job, 5-minute runs)
([Small Datum, 2026-01](http://smalldatum.blogspot.com/2026/01/ssds-power-loss-protection-and-fsync.html)) **[V]**

| Drive | PLP? | fsync, 16 KB | fdatasync, 16 KB | fsync, 2 MB |
|---|---|---|---|---|
| Samsung PM9a3 | Yes | **1.6 µs** | 0.7 µs | 139.1 µs |
| Intel D7-P5520 / Solidigm | Yes | **12.4 µs** | 9.8 µs | 58.2 µs |
| Crucial T500 | Claimed | **891.1 µs** | 447.4 µs | 980.1 µs |
| Samsung 990 Pro | No | **2 974.2 µs** | 2 783.2 µs | 5 396.8 µs |

Corroborating independent measurement from CedarDB using `bench-fio` on a consumer Crucial T700:
non-durable writes complete in **~10 µs** (~100 000 inserts/s/thread), but with `dsync` latency
rises to **~500 µs typical with a >1 ms tail** (~2 000 values/s) — a **25–50× penalty**
([CedarDB](https://cedardb.com/blog/ssd_latency/)) **[V-vendor]**.

**Spec implication, and it is a big one.** The presence or absence of a capacitor-backed write
cache changes durable commit latency by **~1000×** (1.6 µs vs 2 974 µs). Any ledger-commit latency
target in the Nilestream spec is meaningless unless it states the storage class. On PLP-equipped
enterprise NVMe, a single-threaded durable append can reach ~10⁵–10⁶ commits/s before fsync is the
bottleneck; on a consumer drive it is ~300–1 000/s. **Specify PLP NVMe as a hardware precondition,
or your durability numbers will not reproduce.**

### 5.5 Group commit batching factors

PostgreSQL's documentation
([WAL configuration](https://www.postgresql.org/docs/current/wal-configuration.html)) **[V]**
describes the mechanism but **publishes no batching factor**. Key points:

- `commit_delay` makes a group-commit leader sleep N microseconds inside `XLogFlush` so followers
  queue behind it.
- Recommended starting value: *"half of the average time [`pg_test_fsync`] reports it takes to
  flush after a single 8 kB write operation."*
- With `commit_delay = 0` (the default), group commit still occurs opportunistically: a group
  consists only of sessions that arrive during the window in which the previous flush is running.
- No sleep occurs if `fsync` is off or fewer than `commit_siblings` sessions are active.

**There is no citable "typical batching factor."** It is determined by (flush latency) ×
(arrival rate). Derive it for your own target hardware: with a 12 µs fsync and a 200 k txn/s
arrival rate, the opportunistic window batches ~2–3 transactions; with a 900 µs fsync at the same
rate it batches ~180. **This is a derived quantity, not a constant — write it into the spec as a
formula, not a number.**

---

## 6. Worst-case-optimal joins

Freitag, Bandle, Schmidt, Kemper, Neumann, *Adopting Worst-Case Optimal Joins in Relational
Database Systems*, PVLDB 13(12) 2020 ([PDF](https://www.vldb.org/pvldb/vol13/p1891-freitag.pdf)) **[V]**

Hardware: 2× Intel Xeon E5-2680 v4, 28 cores / 56 threads, 256 GiB RAM.

### 6.1 The "77×" figure is not in this paper

I searched the paper specifically for it. **No 77× speedup is reported anywhere.** The largest
claim is *"outperforming the remaining systems by up to two orders of magnitude"* on graph pattern
queries. **[U] — remove 77× from your notes.**

The actual 4-clique numbers (Table 2, `UmbraOHT` = Umbra with the hybrid optimizer + optimized
hash trie):

| Dataset | UmbraOHT | EmptyHeaded | Speedup |
|---|---|---|---|
| Wikipedia | 0.10 s | 0.55 s | 5.5× |
| Epinions | 0.23 s | 1.04 s | 4.5× |
| Slashdot | 0.18 s | 1.24 s | 6.9× |

Against "DBMS X" the 4-clique gaps are larger: 0.10 s vs 1.66 s on Wikipedia (~17×), 0.23 s vs
6.53 s on Epinions (~28×).

3-clique on large graphs (Table 3):

| Dataset | UmbraOHT | EmptyHeaded | Speedup |
|---|---|---|---|
| Google+ | 7.70 s | 18.67 s | 2.4× |
| Orkut | 15.25 s | 309.14 s | **20.3×** |
| Twitter | 579.07 s | timeout | — |

### 6.2 The regression story — this is the important half

- **TPC-H and JOB: no regression at all.** *"UmbraOHT exhibits no slowdown over the unmodified
  version of Umbra"* (§5.2.1, Fig. 5).
- **JOB without filters:** the hybrid optimizer *"matches or improves over the performance of
  Umbra"* (§5.2.2, Fig. 6). One query improves 1.9× over Umbra and 4.2× over the eager
  always-multiway variant.
- **The eager variant (`UmbraEAG`, always use WCOJ) is the cautionary tale:** it beats MonetDB and
  DBMS X but *"falls short of binary join plans if the latter do not incur any redundant work."*
  A commercial DBMS that relies on WCOJ unconditionally is beaten by Umbra *"by up to four orders
  of magnitude"* on TPC-H and JOB (§2.2).
- **Hybrid optimizer decision accuracy (Table 4)** is the number a spec should internalize:

  | Workload | True positives | False negatives |
  |---|---|---|
  | TPC-H | **0 of 59 joins** | — |
  | JOB | **0 of 864 joins** | — |
  | JOB (no filters) | 19 | 75 |
  | Graph queries | 48 | **0** |

- **Cost of guessing wrong:** false positives *"increase the absolute query runtime by up to 3.7×"*
  on their synthetic benchmark; false negatives *"only miss a potential further speedup of up to
  1.6×"* on 3 of 32 JOB queries.

**Spec implication: WCOJ is a strictly graph-pattern feature.** On TPC-H and JOB the optimizer
correctly chose it **zero times out of 923 joins**. Adopting it costs nothing if you gate it behind
a hybrid optimizer, and buys nothing on relational workloads. The asymmetry (3.7× downside on a
false positive vs 1.6× upside forgone on a false negative) argues for a conservative gate.

---

## 7. Streaming / IVM and partial materialization

### 7.1 DBSP: the paper has no evaluation

Budiu et al., *DBSP: Automatic Incremental View Maintenance for Rich Query Languages*, PVLDB 16(7)
2023 ([PDF](https://www.vldb.org/pvldb/vol16/p1601-budiu.pdf)) **[V]**

I fetched and checked this. **The paper contains no quantitative evaluation.** §8 states
explicitly: *"The scope of this paper is the DBSP theory of IVM, so we only briefly touch upon the
implementation aspects. We defer a full description and evaluation of the system to a future
paper."*

What it does provide: the complexity claim that operators run in **O(|ΔDB[t]|)** time, a Rust
implementation, a SQL-to-DBSP compiler passing 7 M SQL Logic Tests, and a **Lean mechanization of
the core theory (~5 k lines)**.

**Cite DBSP for the theory and the O(|Δ|) bound. Do not cite it for throughput.** Since Nilestream
is DBSP-compatible by design, the Lean mechanization is the more useful thing to reference anyway.

### 7.2 Feldera — vendor self-benchmark, transparently caveated

[Feldera vs Flink on Nexmark](https://www.feldera.com/blog/nexmark-vs-flink) **[V-vendor]**

Hardware: Threadripper 3990X, 64 cores / 128 threads, 256 GB RAM. 100 M Nexmark events. Feldera
16 worker threads; Flink 8 task managers × 2 cores.

| Metric | Result vs Flink |
|---|---|
| Throughput | **2.2× geomean**, up to 6.2× (q7); q13 slightly slower |
| Memory | **0.24× geomean** (0.03×–2.6×) |
| CPU time | **0.8× geomean** (0.2×–2.3×) |

Disclosed omissions: q6 (no Flink implementation), q10 (Flink implementation failed), q11 and q21
(Feldera lacks session windows / UDFs). This is a vendor benchmark, but the disclosure is unusually
honest — they report the query where they lose and the queries they cannot run.

**Note the shape of the result: 2.2× on throughput, but 4× on memory.** For a partially-stateful
design the memory number is the more relevant one, and it is the one a vendor has least incentive
to inflate against a JVM baseline.

### 7.3 Materialize — vendor benchmark, architecturally unfair

[Materialize vs Aurora PostgreSQL](https://materialize.com/blog/performance-benchmark-aurora-postgresql-materialize/) **[V-vendor]**

Claimed: 588× throughput / 590× latency at 1 connection; 355× / 750× at 10 connections; Aurora
crashed at 100 connections. Headline: *"100× greater throughput with 1000× lower latency."*

**Why this is not a usable target.** The workload is a single complex CTE/window-function query
over 1 GB, and the comparison is Materialize *serving a precomputed incrementally-maintained view*
against Aurora *executing the full query on every request*. That is a comparison between
materialization and no materialization, not between two engines. PostgreSQL was given indexes but
no materialized view. A fair baseline would be a PostgreSQL materialized view with a refresh
strategy, or a cache.

The honest version of this claim: **incremental maintenance converts a repeated O(query) cost into
an O(Δ) cost, so the "speedup" is unbounded and equals the query's re-execution cost.** That is a
statement about caching, not about engine quality — and it is exactly the claim Nilestream should
make carefully, because it is trivially true and therefore rhetorically weak.

### 7.4 DBToaster

[DBToaster performance page](https://dbtoaster.github.io/home_performance.html) **[V-vendor/academic]**

Claim: generated engines *"regularly outperform the commercial systems by 3–6 orders of magnitude"*
on realtime-warehousing queries derived from TPC-H, measured as *"the rate at which each system can
produce up-to-date (fresh) views."* Hardware: Xeon E5-2620, 6 cores, 128 GB.

**Caveats that make this uncitable as a target:** the baselines are anonymous ("DBX", "SPY") per
licensing restrictions, so the comparison is unfalsifiable; the primary scale factor is **0.1
(100 MB)**; and, as with Materialize, the comparison is IVM against full recomputation.

The one hard number worth keeping: Noria measured DBToaster at **520 k writes/s** vs Noria's 240 k
(fully-populated) / 1 M (fully-evicted), with DBToaster using **17 GB vs Noria's 6.2 GB** — i.e.
Noria used **36 % of DBToaster's memory**. That is a third-party measurement of DBToaster and is
more trustworthy than DBToaster's own page.

### 7.5 Noria — your figures verified, with a caveat you need

Gjengset et al., *Noria: dynamic, partially-stateful data-flow for high-performance web
applications*, OSDI 2018 ([PDF](https://cs.brown.edu/people/malte/pub/papers/2018-osdi-noria.pdf)) **[V]**

**Memory (§8.4) — your "~9 %" is correct:**
- Full state across all operators: **789 MB** (Lobsters). Essential partial state: **73 MB**.
  *"Noria's essential memory requirement for Lobsters therefore amounts to 9 % of total state."*
- 35 of 60 stateful operators could use partial materialization.
- At production scale the working set is 525 MB (60 % of total); at 10× scale, 2.6 GB of 7 GB
  (38 %). **The 9 % is the *essential* floor, not the steady-state working set.** The working-set
  numbers (38–60 %) are the honest ones to quote for capacity planning.
- Total overhead ~3× base table size.

**Throughput (§8.1) — your "5× at 16 vCPUs, sub-100 ms p95" is correct:**
- MariaDB: 1 000 pages/s (saturating 16 cores). Noria with baseline queries: 2 300 pages/s.
  Noria with "natural" queries: **5 000 pages/s = 5× MariaDB**, at sub-100 ms 95th-percentile.
- Hardware: EC2 c5.4xlarge, 16 vCPUs. Dataset: 9.2 k users, 40 k stories, 120 k comments.
- MariaDB without pre-computation managed *"just 20 pages/sec."*

**§8.2 single-query comparison:** Noria 14 M req/s (4 shards, Zipfian s=1.08) vs MariaDB and
MariaDB+memcached at ~100–200 k req/s. 2 M vs ~20 k on a 50/50 read/write mix.
**§8.3:** 3 M req/s on one machine, 30 M across ten.

> **The caveat you must carry with the 5×.** The authors state (§8.1) that MariaDB was
> *"configured to use a thread pool, to avoid flushing to disk after transactions, and to store the
> database on a ramdisk."* **Durability was off and the database was in RAM.** The System Z
> comparison used **read-uncommitted** isolation. The memcached-only configuration is described by
> the authors themselves as *"unrealistic: it does not store individual votes or stories, is not
> persistent, and cannot prevent double-voting."*
>
> Also §8.5: Noria *"only guarantees eventual consistency"*, eviction is randomized, it is
> inefficient for sharded queries requiring shuffles, and it lacks some SQL. **A 5× throughput win
> bought with eventual consistency is not a 5× win for a strictly-serializable ledger system.**
> This is the central honesty problem for Nilestream's read side and it should be confronted in
> the spec, not buried.

### 7.6 Newer partial-materialization measurements

Gjengset's MIT PhD thesis, *Partial State in Dataflow-Based Materialized Views* (2021)
([PDF](https://pdos.csail.mit.edu/papers/jfrg:thesis.pdf)) **[V]** is the only substantial follow-up
I found. Its abstract makes two claims that **differ from the OSDI paper and are worth noting**:

- *"Partial state also reduces memory use by up to 2/3 compared to traditional materialized views."*
  — i.e. **~3× reduction, not the ~11× implied by "9 % of full state."** The 2/3 figure is the more
  conservative and probably the more defensible one; the 9 % is an essential-state floor on one
  workload.
- *"...increases supported application load by up to 20× over MySQL"* — larger than the OSDI
  paper's 5×, presumably a different configuration.
- Acknowledged limitations: *"it is eventually consistent, supports only a subset of SQL, increases
  memory use, and reduces write performance."*

I was not able to extract specific upquery latency figures in milliseconds or the quantified
write-throughput cost of partial state (§6.8) from the thesis PDF. **If those numbers matter to
your spec — and the cost of an upquery on the read path is arguably the most important single
number for a partially-stateful design — they need to be read out of the thesis directly, or
measured.** I flag this as an open gap rather than guessing.

**I found no partial-materialization measurements newer than 2021.** This is a genuinely thin
area, which is good news for a thesis contribution and bad news for baseline-setting.

---

## 8. Caveats: what these numbers do not mean

### 8.1 The ClickBench PostgreSQL configuration is close to worst-case

The 1091× DuckDB-over-PostgreSQL figure I computed in §2.2 is real arithmetic on real data and it
is **not a meaningful engine comparison.** Unindexed PostgreSQL takes ~258 s on nearly every one of
the 43 queries — a flat line, because every query is a full sequential scan of a 100 M-row, 100 GB
row-major table. It measures "row store scans 100 GB", nothing more.

The indexed configuration (11.5× geomean) is the fair one, and even it flatters the columnar
systems: ClickBench's dataset is **a single flat denormalized table**, which is ClickHouse's home
turf. The ClickBench maintainers say so themselves: *"The dataset is a single flat table"* unlike
normalized warehouse schemas, *"potentially disadvantaging traditional systems."*

Additional ClickBench limitations, in the maintainers' own words
([README](https://github.com/ClickHouse/ClickBench)):

- 99 997 497 records is *"rather small by modern standards."*
- *"The benchmark runs its queries one after another and does not test workloads with concurrent
  queries."* — **no concurrency at all.**
- Mostly single-node.
- *"It is not possible to test the efficiency of storage used for in-memory databases, or the time
  of data loading for stateless query engines."*
- Their own disclaimer: *"All Benchmarks Are ~~Bastards~~ Liars."*
- **And: ClickBench is maintained by ClickHouse Inc.**, a vendor whose product is one of the
  systems ranked. The raw data is public and the harness reproducible, which mitigates this a
  great deal, but the *choice of dataset and query set* is a vendor's choice.

Note also the load-time asymmetry the table in §2.2 exposes: PostgreSQL-indexed took **10 357 s to
load** versus DuckDB's 126 s. If you include load time, the comparison changes character entirely.
Most published comparisons quietly exclude it.

### 8.2 TPC-H is not representative, and the literature says so precisely

Dreseler et al., PVLDB 13(8) 2020 **[V]**, is the best citation for this:

- **Uniform, uncorrelated data.** *"Linear scaling of non-fact tables, its homogeneous data
  distribution"*, and a 3NF rather than star schema. No realistic skew.
- **No NULLs, no outer joins** (except Q13), all joins are equi-joins.
- **Artificial correlations that are exploitable.** `l_shipdate` and `l_receiptdate` are always
  within 30 days, so *"when a query accesses the lineitem or orders table, a third of data does not
  have to be accessed"* by partition pruning. That is a benchmark artifact.
- **Handcrafted queries** *"non-comparable with the auto-generated queries of actual workloads."*

And the decomposition that should reshape your priorities:

| Optimization | Geomean contribution |
|---|---|
| **Subquery flattening** | **~510×** |
| Predicate pushdown / ordering | ~4× (Q22: 7368 %) |
| Physical locality / partition pruning | up to 3.35× |
| **Join ordering** | **~7 %** |
| Dependent group-by keys | ~24 % on Q10 only |

**Subquery flattening is worth ~510× and join ordering is worth ~7 %.** Combined predicate
placement + subquery flattening is *"almost 30× improvement"* across the benchmark. Q1, Q13, Q16
and Q18 are optimization-resistant — *"even the most significant improvement did not double
performance"* — so those four are pure execution-engine tests.

The paper also supplies the perfect example of honest-but-misleading benchmarking: **Hyrise appears
7× faster than MonetDB on Q6, but *"without this optimization [partition pruning], Hyrise is only
1.8× faster."*** Both statements are true. Only one is informative.

### 8.3 Benchmarking crimes: the standard checklist

Raasveldt, Boncz, Mühleisen et al., *Fair Benchmarking Considered Difficult*, DBTest 2018
([PDF](https://hannes.muehleisen.org/publications/DBTEST2018-performance-testing.pdf)) **[V]**

They cite a survey finding *"easily preventable benchmarking crimes... with various issues found in
96 % of papers."* Their pitfalls, with the magnitudes they measured:

| Pitfall | Measured distortion |
|---|---|
| Unoptimized build (debug assertions on) | MonetDB TPC-H Q1 SF1: **1.58 s vs 0.87 s** — ~2× from a compile flag |
| Untuned configuration | PostgreSQL TPC-H Q9: **0.47 s vs 0.27 s** — 74 % from settings alone |
| Apples vs oranges (hand-coded vs DBMS) | Hand-written Q1: **0.03 s vs MonetDB 0.87 s** — ~30×, entirely from omitting error checking and transaction logic |
| Hidden schema choices | MariaDB ranked *both fastest and slowest* depending on DOUBLE vs DECIMAL — both TPC-H-compliant |
| Cold vs hot, cold vs warm | OS page cache silently converts "cold" runs into warm ones |
| Ignoring preprocessing | Index build and load time excluded, favouring expensive-index systems |
| Incorrect code | A bug can make a system *faster* by touching less data |

**The "apples vs oranges" row is directly relevant to Nilestream.** A from-scratch engine without
full SQL coverage, without multi-user transaction machinery, and without complete error handling
will beat PostgreSQL by a wide margin on any benchmark, and that margin is not a real win. The spec
must state what functionality is present when the number is taken.

### 8.4 OLTP-specific caveats

- **Durability off is the default in almost every number above.** Callaghan disabled fsync on
  commit for both PostgreSQL and MySQL; Noria put MariaDB on a ramdisk with flushing off; Silo's
  best number excludes checkpointing and recovery and does not touch the network; VoltDB's cited
  results were run *"without any logging or replication."* **A durable number and a non-durable
  number differ by the fsync latency of §5.4 divided by the batching factor of §5.5 — which is to
  say, by up to three orders of magnitude on the wrong hardware.**
- **In-memory research systems assume the working set fits.** HyPer's 126 k txn/s single-threaded
  used **12 warehouses / ~1 GB.** Silo set #warehouses = #workers with NUMA-local assignment. These
  are partition-friendly configurations. Cross-partition transaction rates are where the numbers
  collapse — Silo's own comparison against Partitioned-Store shows a 2.98× advantage at 60 %
  cross-partition, which is the interesting regime.
- **No think time.** Research TPC-C runs closed-loop with zero keying/think time; audited TPC-C
  mandates it. The two measure different things.
- **NOPM ≠ tpmC.** See §3.3.

### 8.5 Vendor benchmarks: specific things I would not pass along

| Claim | Source | Why not |
|---|---|---|
| pg_duckdb **1500×** / **18 000×** over PostgreSQL | [MotherDuck](https://motherduck.com/blog/pgduckdb-beta-release-duckdb-postgres/) | **Single query** (TPC-DS Q1), **no indexes** on PostgreSQL (acknowledged in the post), SF1 and SF10. Q1 is a correlated-subquery query PostgreSQL handles badly. This is a worst case dressed as a average. |
| Materialize **100× throughput / 1000× latency** vs Aurora | [Materialize](https://materialize.com/blog/performance-benchmark-aurora-postgresql-materialize/) | Compares a maintained view against on-demand full query execution. Architecturally not a like-for-like test. |
| DBToaster **3–6 orders of magnitude** | [DBToaster](https://dbtoaster.github.io/home_performance.html) | Anonymous baselines (unfalsifiable), SF0.1, IVM vs recomputation. |
| AlloyDB **117×** over PostgreSQL | quoted in [CedarDB](https://cedardb.com/blog/ode_to_postgres/) | Second-hand quotation of a Google marketing figure; the same passage notes it drops to 20× with GROUP BY and **2.6× with joins**. Quote the 2.6× if you quote anything. |
| CedarDB "faster than ClickHouse" | ClickBench leaderboard | True on the geomean I computed (2.8× over DuckDB, and it edges ClickHouse on geomean) — **but** CedarDB's *total* time (28.99 s) is worse than ClickHouse's (18.09 s), because it loses badly on Q18 and Q32. Geomean and sum disagree. A competitor also notes CedarDB has no distributed query support ([Tinybird](https://www.tinybird.co/blog/clickhouse-vs-cedardb)). |

### 8.6 Figures in the original brief that I could not verify

| Figure as given | Status |
|---|---|
| "77× on 4-clique" (Freitag et al. VLDB 2020) | **[U] Not in the paper.** No 77× appears. Actual 4-clique vs EmptyHeaded: 4.5×–6.9×. Largest single ratio in the paper: 20.3× (3-clique, Orkut, vs EmptyHeaded), ~28× vs DBMS X (4-clique, Epinions), and a general "up to two orders of magnitude" claim. |
| "1.4× on TPC-H Q6 from vectorization" (Kersten et al. 2018) | **Number correct, attribution wrong.** 1.4× is the **AVX-512 SIMD** gain on Q6. Compiled vs vectorized on Q6 is a **tie** (15 ms vs 15 ms). |
| "Noria: memory to ~9 % of full state" | **[V] Correct** — 73 MB of 789 MB, Lobsters, *essential* state. Steady-state working set is 38–60 %. |
| "Noria: 5× the load of hand-optimized MySQL at 16 vCPUs, sub-100 ms p95" | **[V] Correct** — 5 000 vs 1 000 pages/s, c5.4xlarge. But the MariaDB baseline had **durability off and ran on a ramdisk**, and Noria is **eventually consistent**. |
| "Umbra 3.0× geomean over HyPer on JOB" | **[V-self] Correct** — CIDR 2020 §4.1. Self-comparison; no PostgreSQL baseline exists in that paper. |

---

## 9. Where a win is actually available

This section is the analytical payload: a normalization that decides which targets are honest.

### 9.1 OLTP, normalized to transactions/sec/core

HammerDB NOPM is New-Order per *minute*, and New-Order is 45 % of the TPC-C mix. Converting
everything to total transactions/sec/core:

| System | Config | Total txn/s | Cores | **txn/s/core** |
|---|---|---|---|---|
| PostgreSQL 18.1 | wh=4000, fsync off | 8 777 | 48 | **183** |
| PostgreSQL 18.1 | wh=1000, fsync off | 15 987 | 48 | **333** |
| MySQL 8.4.8 | wh=4000, fsync off | 8 500 | 48 | 177 |
| Commercial in-memory DBMS | Silo's own baseline, same HW as Silo | — | — | **3 000** |
| Silo | 32 warehouses in RAM, logging on, no network | 700 000 | 32 | **21 875** |
| HyPer | 12 warehouses / 1 GB, 5 threads | 380 868 | 5 | 76 174 |

**[M]** — derived from **[V]** sources in §3.

Read this ladder carefully, because each rung is bought with a different currency:

- **PostgreSQL → commercial in-memory DBMS: ~9×.** This is the honest, achievable figure for a
  *general-purpose* system: full SQL, a wire protocol, real durability, unpartitioned workloads.
  It is the gap attributable to architecture (in-memory structures, better concurrency control,
  less buffer-manager and MVCC overhead) rather than to benchmark restrictions.
- **Commercial in-memory → Silo: another ~7×.** This rung is bought entirely with restrictions.
  Silo's clients *do not use the network* (authors estimate ~23 % cost to add it), there is no
  checkpointing or recovery, no client think time, and #warehouses = #workers with NUMA-local
  assignment so cross-partition traffic is minimal.
- **HyPer's 76 k/core is not comparable to anything.** 12 warehouses / ~1 GB fits in cache. Note
  that its own scaling is poor — 1 thread gives 126 576 txn/s and 5 threads only 380 868, i.e.
  3.0× from 5 cores. The single-thread number is a cache-residency artifact.

**Conclusion for the spec: budget ~5–10× over PostgreSQL on OLTP, not 50×.** The 50–65× figures in
the literature are real measurements of systems that gave up the network, recovery, general SQL and
unpartitioned workloads. Nilestream's requirements (strict serializability, durability,
auditability, retention, a MySQL/PostgreSQL wire protocol) place it on the *general-purpose* rung.
A claim above ~10× on OLTP would require explicitly disclosing which of Silo's restrictions you
have adopted.

### 9.2 Analytics, and where PostgreSQL is genuinely far from the bound

The analytical side is the opposite story, and it is where the defensible large win lives.

| Workload shape | PostgreSQL's position | Available win |
|---|---|---|
| Full-scan aggregation over a wide table (ClickBench-like) | Row-major storage forces reading every column; no vectorization; limited intra-query parallelism | **10²–10³×** — but most of it is *storage format*, not execution engine |
| Multi-join analytics with correlated predicates (JOB-like) | Cardinality errors ≥10× on 52 % of 3-join estimates; ~10 % of JOB queries historically failed to complete | **Large but optimizer-shaped**, not engine-shaped |
| Repeated queries over slowly-changing data | Full re-execution every time | **Unbounded** — this is IVM, and the "speedup" is just the re-execution cost you avoided |
| Point lookups and highly selective indexed access | Already near-optimal; B-tree access dominates | **None.** PostgreSQL beat DuckDB on **10 of 43** ClickBench queries |
| Single-query OLTP-style writes with durability | Bounded by fsync (§5.4), which is a hardware constant | **None available in software** |

### 9.3 The decomposition that should drive the spec's priorities

Ordering the measured contributions from the literature, largest first:

| Lever | Measured effect | Source |
|---|---|---|
| Columnar storage + not reading unused columns | 10²–10³× on scan-heavy analytics | §2.2 **[M]** |
| Subquery flattening | **~510× geomean** on TPC-H | Dreseler §8.2 **[V]** |
| Avoiding tuple-at-a-time interpretation | Real work is **<10 %** of time in an interpreted engine | Boncz 2005 **[V]** |
| Incremental maintenance vs re-execution | Unbounded (equals re-execution cost) | §7 |
| Predicate pushdown / ordering | ~4× | Dreseler **[V]** |
| Partial materialization (memory, not speed) | **~3×** (thesis) to 9 % essential floor (OSDI) | §7.5–7.6 **[V]** |
| Compilation tier choice (bytecode-first vs LLVM) | **108× compile latency**, 1.2× execution cost | §5.3 **[V-self]** |
| Compiled vs vectorized execution | **0.66×–1.93×** — a wash | Kersten 2018 **[V]** |
| Join *ordering* algorithm (given good estimates) | **~7 %** | Dreseler **[V]** |
| SIMD on realistic join queries | **~1.1×** | Kersten 2018 **[V]** |
| WCOJ on relational workloads | **0 of 923 joins** chosen by the optimizer | Freitag 2020 **[V]** |

**The top of this list is storage layout, algebraic rewriting, and avoiding interpretation. The
bottom of it — SIMD, compiled-vs-vectorized, join enumeration, WCOJ — is where engine projects
usually spend their effort and where the measured returns are 7 %–2×.**

For a spec whose thesis is *partially-materialized state over an immutable ledger*, the honest
positioning follows directly: the defensible claims are (a) memory, via partial materialization
(~3×, well-supported), and (b) latency on repeated derived queries, via IVM (unbounded, but a
caching argument). The *engine* claims — vectorization, compilation, WCOJ — should be specified as
**parity requirements, not differentiators**, because the literature says the spread between good
implementations there is under 2×.

---

## 10. References

1. Boncz, Zukowski, Nes. *MonetDB/X100: Hyper-Pipelining Query Execution.* CIDR 2005.
   https://www.cidrdb.org/cidr2005/papers/P19.pdf
2. Budiu et al. *DBSP: Automatic Incremental View Maintenance for Rich Query Languages.* PVLDB 16(7), 2023.
   https://www.vldb.org/pvldb/vol16/p1601-budiu.pdf
3. Callaghan, M. *SSDs, power loss protection and fsync latency.* Small Datum, Jan 2026.
   http://smalldatum.blogspot.com/2026/01/ssds-power-loss-protection-and-fsync.html
4. Callaghan, M. *HammerDB tproc-c on a large server, Postgres and MySQL.* Small Datum, Feb 2026.
   http://smalldatum.blogspot.com/2026/02/hammerdb-tproc-c-on-large-server.html
5. Callaghan, M. *HammerDB tproc-c on a large server, Postgres 14 to 19 beta1.* Small Datum, Jun 2026.
   http://smalldatum.blogspot.com/2026/06/hammerdb-tproc-c-on-large-server.html
6. CedarDB. *Why Your SSD (Probably) Sucks and What Your Database Can Do About It.*
   https://cedardb.com/blog/ssd_latency/
7. CedarDB. *An ode to PostgreSQL, and why it is still time to start over.*
   https://cedardb.com/blog/ode_to_postgres/
8. ClickHouse. *ClickBench: a Benchmark For Analytical Databases.* https://github.com/ClickHouse/ClickBench
9. Dreseler et al. *Quantifying TPC-H Choke Points and Their Optimizations.* PVLDB 13(8), 2020.
   https://www.vldb.org/pvldb/vol13/p1206-dreseler.pdf
10. DuckDB. *Running TPC-H SF100 on Mobile Phones.* Dec 2024. https://duckdb.org/2024/12/06/duckdb-tpch-sf100-on-mobile
11. DuckDB. *The Lost Decade of Small Data?* May 2025. https://duckdb.org/2025/05/19/the-lost-decade-of-small-data
12. Feldera. *Feldera performance on Nexmark versus Flink.* https://www.feldera.com/blog/nexmark-vs-flink
13. Freitag, Bandle, Schmidt, Kemper, Neumann. *Adopting Worst-Case Optimal Joins in Relational Database Systems.* PVLDB 13(12), 2020.
    https://www.vldb.org/pvldb/vol13/p1891-freitag.pdf
14. Gjengset et al. *Noria: dynamic, partially-stateful data-flow for high-performance web applications.* OSDI 2018.
    https://cs.brown.edu/people/malte/pub/papers/2018-osdi-noria.pdf
15. Gjengset, J. *Partial State in Dataflow-Based Materialized Views.* PhD thesis, MIT, 2021.
    https://pdos.csail.mit.edu/papers/jfrg:thesis.pdf
16. HammerDB. *How to understand TPC-C tpmC and TPROC-C NOPM.*
    https://www.hammerdb.com/blog/uncategorized/how-to-understand-tpc-c-tpmc-and-tproc-c-nopm-and-what-is-good-performance/
17. Kemper, Neumann. *HyPer: A hybrid OLTP&OLAP main memory database system based on virtual memory snapshots.* ICDE 2011.
    https://cs.brown.edu/courses/cs227/archives/2012/papers/olap/hyper.pdf
18. Kersten, Leis, Kemper, Neumann, Pavlo, Boncz. *Everything You Always Wanted to Know About Compiled and Vectorized Queries But Were Afraid to Ask.* PVLDB 11(13), 2018.
    https://dl.acm.org/doi/10.14778/3275366.3284966
19. Kersten, Leis, Neumann. *Tidy Tuples and Flying Start: fast compilation and fast execution of relational queries in Umbra.* VLDB Journal 30, 2021.
    https://db.in.tum.de/~kersten/Tidy%20Tuples%20and%20Flying%20Start%20Fast%20Compilation%20and%20Fast%20Execution%20of%20Relational%20Queries%20in%20Umbra.pdf
20. Leis et al. *How Good Are Query Optimizers, Really?* PVLDB 9(3), 2015. https://www.vldb.org/pvldb/vol9/p204-leis.pdf
21. Leis, Neumann. *Still Asking: How Good Are Query Optimizers, Really?* PVLDB 18, 2025. (Retrospective; no new measurements.)
    http://www.vldb.org/pvldb/vol18/p5531-viktor.pdf
22. Materialize. *Performance Benchmark: Aurora PostgreSQL vs. Materialize.*
    https://materialize.com/blog/performance-benchmark-aurora-postgresql-materialize/
23. MotherDuck. *pg_duckdb beta release.* https://motherduck.com/blog/pgduckdb-beta-release-duckdb-postgres/
24. Neumann, Freitag. *Umbra: A Disk-Based System with In-Memory Performance.* CIDR 2020.
    https://db.in.tum.de/~freitag/papers/p29-neumann-cidr20.pdf
25. Neumann, T. *Evolution of a Compiling Query Engine.* PVLDB 14(12), 2021. http://www.vldb.org/pvldb/vol14/p3207-neumann.pdf
26. PostgreSQL Global Development Group. *WAL Configuration*, PostgreSQL 18 documentation.
    https://www.postgresql.org/docs/current/wal-configuration.html
27. Raasveldt, Holanda, Gubner, Mühleisen. *Fair Benchmarking Considered Difficult: Common Pitfalls In Database Performance Testing.* DBTest 2018.
    https://hannes.muehleisen.org/publications/DBTEST2018-performance-testing.pdf
28. Schwartz, B. *Is VoltDB really as scalable as they claim?* Percona.
    https://www.percona.com/blog/is-voltdb-really-as-scalable-as-they-claim/
29. Tinybird (Archer, C.). *ClickHouse vs CedarDB: Is Cedar really faster?* https://www.tinybird.co/blog/clickhouse-vs-cedardb
30. Transaction Processing Performance Council. *TPC-H Results.* https://www.tpc.org/tpch/results/tpch_results5.asp?version=4
31. Tu, Zheng, Kohler, Liskov, Madden. *Speedy Transactions in Multicore In-Memory Databases.* SOSP 2013.
    https://wzheng.github.io/silo.pdf
32. Vonng. *pgtpc: PostgreSQL TPC benchmark results.* https://github.com/Vonng/pgtpc
33. Xu et al. *OceanBase: A 707 Million tpmC Distributed Relational Database System.* PVLDB 15(12), 2022.
    https://vldb.org/pvldb/vol15/p3385-xu.pdf
