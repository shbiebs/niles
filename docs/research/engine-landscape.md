# The Four-Workload Engine: Does It Exist?

*A citation-backed survey of transactional + analytical + graph + streaming execution in a single
database engine, and of the optimizer guarantees such an engine would need.*

Compiled September 2026. Sources are papers, standards, and vendor **documentation** (not marketing
pages) wherever a primary source exists. Every claim carries a citation marker `[n]` resolved in
§12. Confidence markers: **[V]** = verified against a primary source in this survey; **[S]** =
secondary source only; **[U]** = could not verify, see §11.

---

## 1. Executive summary — the direct answer

**No such engine exists, and one of the stated requirements is provably unachievable.**

Broken into parts:

1. **Transactional + analytical in one engine is solved** — genuinely, not just in marketing — but
   only at a specific scale and with a specific architecture. Umbra [1] and its commercial
   descendant CedarDB [21] are the existence proof: a single storage format (PAX-in-B-tree),
   a single MVCC scheme, a single compiler, no row/column seam. Every *other* HTAP system on the
   user's list keeps two physical representations and pays a freshness or isolation tax at the
   seam [16].

2. **Adding graph is solved as an execution problem and unsolved as a product problem.** The
   technical answer to "graph queries far more efficiently than SQL" is worst-case-optimal joins,
   and Freitag et al. [3] shipped exactly that inside Umbra with a hybrid optimizer, getting ~77x
   over a commercial RDBMS on 4-clique queries **with no regression on TPC-H or JOB**. So a
   relational engine *can* absorb graph performance. But the language layer (SQL/PGQ) only landed
   in PostgreSQL in March 2026 [12], without variable-length paths, and the leading columnar graph
   DBMS (Kùzu) was archived by its own company in October 2025 [10].

3. **Adding streaming to that same engine has not been done by anyone.** Every incremental-view
   engine surveyed (Materialize, Feldera, RisingWave, Noria/ReadySet, Flink) is explicitly
   *downstream* of an OLTP database. RisingWave's own docs state it "is not designed to replace
   PostgreSQL or MySQL for OLTP workloads" [26]. Noria — the closest architectural relative to what
   the user seems to want — is **eventually consistent** by design [27]. Nobody does all four.

4. **"The optimizer should never produce a bad join order or pick a wrong index" is impossible,**
   and this is not an engineering gap — it is a theorem, in fact three independent theorems.
   See §5 and §10. The strongest known result: even an optimizer that abandons compile-time
   estimation entirely and discovers selectivities by executing, has a **worst-case suboptimality of
   at least 4x, and no deterministic online algorithm can do better than 4x even in one
   dimension** [7]. Any system design that assumes plan optimality is a solved sub-problem is
   built on sand. The achievable goal is *bounded* badness, not *zero* badness.

5. **The user's SIMD claim is correct.** Kersten et al. [5] measure 8.4x SIMD speedup on a
   *microbenchmark* selection and **1.4x on TPC-H Q6**, ~1.1x on Q3/Q9. Their explanation:
   "most OLAP queries are bound by data access, which does not (yet) benefit much from SIMD."
   Anyone quoting 8x on real queries is quoting a microbenchmark.

**Closest system today: CedarDB.** It satisfies requirements 1, 2, 3, 6 (partially), 7, 8 and 9,
and is missing 4 (impossible), 5 (graph is roadmap-only; streaming absent). It is also 64 GB-capped
in the free tier and has no HA, replication, or failover as of its published roadmap [22].

---

## 2. Capability matrix

Workload columns: **T** = OLTP (high-concurrency short read-write transactions), **A** = OLAP
(scan/aggregate/join at scale), **G** = graph (multi-hop, cyclic patterns, variable-length paths),
**S** = streaming (continuous incremental view maintenance over unbounded input).

Scoring: ● first-class, ◐ real but with a documented seam or restriction, ○ absent/anti-goal.

| System | T | A | G | S | Storage seam? | Exec model | Isolation | PG/MySQL wire |
|---|---|---|---|---|---|---|---|---|
| **Umbra / CedarDB** | ● | ● | ◐ WCOJ + recursive CTE; SQL/PGQ on roadmap [22] | ○ | **None** — one PAX B-tree [1] | Adaptive: bytecode → LLVM [1] | Serializable MVCC [19] | PG wire + catalog [21] |
| PostgreSQL 19 | ● | ◐ | ◐ SQL/PGQ, fixed-length paths only [12] | ○ | None (row-only) | Interpreted | Serializable (SSI) | native |
| SAP HANA | ◐ | ● | ◐ separate graph engine | ○ | **Yes** — column main + row delta [16] | Compiled (L/HEX) | Serializable-ish [U] | ○ |
| SingleStore | ● | ● | ○ | ○ | **Yes** — rowstore + columnstore [17] | Compiled (MBC) | RC / RR [S] | MySQL wire |
| TiDB + TiFlash | ● | ● | ○ | ○ | **Yes** — Raft learner columnar replica [18] | Vectorized | **No SERIALIZABLE**; SI sold as RR [24] | MySQL wire |
| CockroachDB | ● | ◐ | ○ | ○ | None (row-only) | Vectorized + row | Serializable default | PG wire, gaps [25] |
| YugabyteDB | ● | ◐ | ○ | ○ | None | Postgres-derived | Serializable available [23] | PG wire (reuses PG) |
| Oracle In-Memory | ● | ● | ◐ SQL/PGQ 23ai [13] | ○ | **Yes** — dual format [16] | Compiled + SIMD | Serializable | ○ |
| SQL Server columnstore | ● | ● | ◐ graph tables | ○ | **Yes** — NCCI on rowstore [16] | Batch-mode vectorized | Serializable | ○ |
| Snowflake Unistore | ◐ 2 TB/db, ~16k ops/s [20] | ● | ○ | ○ | **Yes** — hybrid + FDN | Vectorized | Session-consistency [20] | ○ |
| DuckDB | ○ single-writer | ● | ◐ DuckPGQ ext., research status [14] | ○ | None (column-only) | Vectorized | Serializable (single writer) | ○ |
| ClickHouse | ○ | ● | ○ | ◐ mat. views, no IVM semantics | None | Vectorized | ○ no general txns | MySQL/PG *ports* only |
| Databricks Photon | ○ | ● | ○ | ○ | n/a (lakehouse) | **Vectorized by choice** [6] | n/a | ○ |
| Databricks Lakebase (2026) | ● (Neon PG) | ● (Photon) | ○ | ○ | **Yes** — pageserver + Parquet, conceded as two physical copies [28] | two engines | PG's | PG wire |
| Kùzu | ◐ single writer; **archived Oct 2025** [10] | ◐ | ● WCOJ + factorization [11] | ○ | None (columnar) | Vectorized + factorized | Serializable, 1 writer [11] | ○ |
| Neo4j / Memgraph / TigerGraph | ◐ | ○ | ● | ○ | n/a | interpreted/compiled | varies | ○ |
| Materialize | ○ writes restricted [29] | ◐ views only | ○ | ● | n/a | Timely/Differential | **Strict serializable default** [29] | PG wire |
| Feldera / DBSP | ○ | ◐ | ○ | ● | n/a | Rust circuits | n/a (deterministic IVM) [8] | ad-hoc PG-ish |
| RisingWave | ○ **explicit anti-goal** [26] | ◐ | ○ | ● | n/a | Vectorized | read-only txns only [26] | PG wire |
| Noria / ReadySet | ◐ | ○ | ○ | ● partial state | n/a | dataflow | **Eventually consistent** [27] | PG + MySQL wire |
| Apache Flink | ○ | ○ | ○ | ● | n/a | JVM | exactly-once *processing* | ○ |

**Nothing in this table has ● in all four columns. Nothing has ● in three.** The best any system
achieves is two ● plus one ◐: Umbra/CedarDB (T, A, ◐G) and Kùzu (◐T, ◐A, G) — and Kùzu is dead.

---

## 3. Question 1 — HTAP: is transactional + analytical solved, and at what cost?

### 3.1 The physical tension is real and the literature names four ways of dodging it

Li & Zhang's SIGMOD 2022 tutorial [16] gives the canonical taxonomy, and it is a taxonomy of
*seams*:

| Class | Systems | Freshness | Scalability cost |
|---|---|---|---|
| Primary row store + in-memory column store | Oracle Dual-Format, SQL Server, DB2 BLU | high | analytics doesn't scale out |
| Distributed row store + column-store replica | TiDB, SingleStore | **degraded** — updates lag the columnar replica | scales both |
| Disk row store + distributed column store | MySQL HeatWave | medium | scales both |
| Primary column store + delta row store | SAP HANA | high | **transactions don't scale** — delta merge is the bottleneck |

The tutorial's conclusion is the important one: such systems "must balance the trade-off between
workload isolation and data freshness," and *no single storage format efficiently serves both*
[16]. Note what this claim actually is: it is an assertion about *dual-format* designs. It is not a
proof that a single format cannot serve both.

### 3.2 Umbra is the counterexample to the folklore

Umbra [1] does not keep two formats. Its architecture:

- Relations are stored in **B⁺-trees with PAX-layout leaf pages** — fixed-size attributes columnar
  at the page head, variable-size data packed at the tail [1]. One representation. Point lookups
  get the tree; scans get columnar locality inside each page.
- A **variable-size-page buffer manager** using pointer swizzling ("swips") with size classes
  mapped into separate reserved virtual-memory regions, avoiding external fragmentation. Measured
  overhead vs. pure in-memory: **under 6% on average** [1]. This is the result that makes
  "disk-based with in-memory performance" more than a slogan.
- **Adaptive compilation**: queries start as interpreted bytecode and are promoted to LLVM-compiled
  machine code only when the runtime justifies it [1].
- Measured: **3.0x geometric mean over HyPer on JOB, 1.8x on TPC-H**; 4.6x / 2.3x over MonetDB [1].

So the honest architectural answer to the user's question — "do these systems keep two formats and
sync them (a seam), or is there a genuine hybrid?" — is: **almost all of them keep a seam; Umbra
does not.** PAX is the resolution. It is a 1998 idea (Ailamaki et al.) whose modern application
inside a B-tree is what makes single-format HTAP work.

The cost Umbra pays instead is *scale*: it is a single-node system. There is no distributed
Umbra. The seam other systems accept is what buys them horizontal scale-out.

### 3.3 CedarDB: current state (checked September 2026)

- **Community Edition launched 14 May 2025**, free, capped at **64 GB of compressed tuple data**
  [21]. Enterprise adds HA, monitoring, native PG replication, S3 scaling, unlimited storage.
- Wire-level: implements "the Postgres wire protocol, SQL grammar, and system catalog" [21].
- **Roadmap gaps as published** [22]: read replication, automatic failover, automatic backups,
  point-in-time recovery, encryption at rest, resource limits — all *planned, not shipped*.
  Range types, schema evolution, vector support, `pg_dump` compatibility, logical replication — all
  *in progress*. "Enhanced graph query support" — *planned*. **Streaming / incremental view
  maintenance does not appear on the roadmap at all** [22].
- Robustness evidence is unusually good: on SQLStorm (18,251 LLM-generated StackOverflow queries),
  Umbra/CedarDB completed **18,165** vs PostgreSQL 15,731 and DuckDB 15,208, and reduced crashes
  from 116 to 0 during benchmark-driven hardening [9].

### 3.4 The 2026 entrants

**Databricks "LTAP" / Lakebase** (announced July 2026) is worth calling out because it is the
loudest current claim of unification. It is Neon-derived serverless PostgreSQL for the write path,
Parquet/Iceberg in object storage for the read path, and a component ("Reyden") that interprets
PostgreSQL pages directly from object store [28]. Marketed as "zero copies"; a Databricks engineer
in the same article conceded "technically two, since pageservers act as a cache or materialization
layer" [28]. **It is a seam architecture with good marketing, not a single engine.**

**Snowflake Unistore / Hybrid Tables** went GA in November 2024 [S]. Its documented limits are the
tell: **2 TB of hybrid-table data per database, ~16,000 ops/sec per database, 200 databases per
account, all hybrid tables in one transaction must be in the same database**, and no clustering
keys, materialized views, streams, Snowpipe, replication, or Search Optimization on them [20].
That is an OLTP *accessory*, not an OLTP engine.

### 3.5 Verdict on Q1

HTAP is solved **on a single node, at single-node scale, by one architecture** (PAX-in-B-tree +
single MVCC + adaptive compilation), demonstrated by Umbra/CedarDB. It is *not* solved in the
distributed case: every distributed HTAP system in the survey pays with a second physical copy and
a freshness lag. The cost of the seam is exactly the thing the user should care about, because a
ledger-plus-derived-views design is itself a seam architecture — just an explicit and principled
one.

---

## 4. Question 2 — Adding graph

### 4.1 Worst-case-optimal joins are the real answer, and they already work inside an RDBMS

This is the most important finding for anyone tempted to build a separate graph engine.

Freitag, Bandle, Schmidt, Kemper & Neumann, VLDB 2020 [3] integrated WCOJ into Umbra. What they
did that matters:

- A **hash trie** built in linear time with constant-time lookups, operating on *hashed* join keys
  and deferring comparisons to enumeration — no persistent index precomputation, unlike EmptyHeaded
  [3]. This removes the historical objection that WCOJ needs expensive sorted tries.
- A **hybrid optimizer** using post-order tree refinement that introduces a multi-way join only
  where the optimizer's own cardinality estimates say the join is *growing* (output cardinality
  exceeds input cardinalities) [3].
- Measured: 4-clique on Slashdot **0.18 s vs 13.95 s** for a commercial RDBMS (~77x); 3-clique on
  Wikipedia 0.03–0.04 s vs EmptyHeaded 0.43 s (13x, including precomputation); 3-clique on Twitter
  579 s where binary-join Umbra timed out [3].
- Critically: **no regression.** "No performance penalty on TPC-H queries where WCOJ offers no
  benefit"; on JOB, matches unmodified binary-join Umbra. The optimizer made **18 false positives
  and 82 false negatives across 1,285 joins** [3].

That last paragraph is the whole ballgame. It means the "relational engines are bad at graph
queries" premise is **obsolete as of 2020** — provided the engine has WCOJ *and* an optimizer
disciplined enough to only fire it on growing joins.

### 4.2 Who actually implements WCOJ

| Engine | WCOJ | Notes |
|---|---|---|
| Umbra / CedarDB | ● | hash trie, hybrid optimizer [3] |
| Kùzu | ● | multiway WCO via ℓ-way adjacency-list intersection, plus factorization and ASP-join [11] — **project archived** [10] |
| DuckPGQ | ◐ | DuckDB extension, research status [14] |
| Free Join (Wang et al., SIGMOD 2023) | ● | unifies WCOJ and binary joins; research |
| PostgreSQL, MySQL, SQL Server, Oracle, SingleStore, TiDB, ClickHouse | ○ | none |
| Neo4j, TigerGraph, Memgraph | ○ / proprietary | [U] |

### 4.3 SQL/PGQ adoption status (checked September 2026)

- **PostgreSQL 19**: SQL/PGQ committed by Peter Eisentraut on **16 March 2026** — `GRAPH_TABLE`,
  property-graph DDL, catalogs, `\dG`, `pg_get_propgraphdef()`. **Variable-length path matching is
  not implemented**, and the security-definer view variant is missing [12]. Fixed-length patterns
  only is a substantial limitation — "friends of friends up to 6 hops" is exactly the query class
  people buy graph databases for.
- **Oracle Database 23ai**: SQL/PGQ implemented [13].
- **Google Spanner Graph**: ISO GQL/SQL-PGQ-aligned [S].
- **DuckPGQ**: SQL:2023 SQL/PGQ syntax, ~30 functions including shortest path, PageRank, WCC;
  explicitly still "part of an ongoing research project at CWI" [14].

### 4.4 The Kùzu event — a cautionary data point

Kùzu was the best-engineered columnar graph DBMS in the literature: disk-based CSR adjacency
indices, factorized intermediate results, ASP-join (accumulate–semijoin–probe), multiway WCOJ,
serializable transactions with a single writer [11]. On LDBC it beat DuckDB and Umbra by **>10x on
selective multi-hop acyclic queries**, and generally beat Umbra on cyclic queries; Neo4j was
excluded as "not competitive" [11].

Its GitHub repository was **archived without notice around 10–14 October 2025**; the company said
only that it was "working on something new" [10]. Community forks exist (Kineviz's `bighorn`), and
one observer noted "there's probably six people that actually understand the codebase" [10].

The lesson for a thesis that proposes a new engine: the technical quality of Kùzu was not the
binding constraint on its survival.

### 4.5 Verdict on Q2

Graph-in-relational is **technically solved at the execution layer** (WCOJ + factorization), **half
solved at the language layer** (SQL/PGQ shipping but without variable-length paths in PostgreSQL),
and **commercially unstable at the specialist layer**. A general relational core with WCOJ is the
right architecture; a separate graph engine is not.

---

## 5. Question 3 — Adding streaming

### 5.1 Every streaming engine surveyed is deliberately downstream of an OLTP database

| System | Does OLTP? | Consistency | Stated position |
|---|---|---|---|
| **Materialize** | ○ — writes restricted; Bounded Staleness forbids INSERT/UPDATE/DELETE [29] | **Strict Serializable is the default**; also Serializable, Bounded Staleness; RC/RR/RU accepted and silently upgraded to Serializable [29] | streaming analytics platform, not general OLTP [29] |
| **Feldera / DBSP** | ○ | deterministic IVM over a formally defined algebra [8] | incremental computation engine |
| **RisingWave** | ○ — **explicit anti-goal** | "supports read-only transactions but does not support read-write transaction processing… **not designed to replace PostgreSQL or MySQL for OLTP workloads**" [26] | sits downstream of an OLTP DB via CDC [26] |
| **Noria** | ◐ — supports INSERT/UPDATE/DELETE, but prototype lacks update/delete on non-PK predicates [27] | **"Noria operators and the contents of its external views are eventually-consistent"** [27] | web-app read scaling |
| **ReadySet** | ○ | cache in front of PG/MySQL | "a MySQL and Postgres wire-compatible caching layer that sits in front of existing databases" [S] |
| **Apache Flink / Arroyo / ksqlDB** | ○ | exactly-once *processing*, not database isolation | stream processors |

Two findings are worth flagging hard for a thesis in this space:

**(a) Materialize already provides strict serializability over incrementally maintained views, by
default, with a documented consistency ladder** [29]. Its four levels — Strict Serializable,
Serializable, Bounded Staleness, and legacy-compatible — are close to the "epoch-based consistency
ladder from bounded-staleness to strict serializability" a REV-style thesis would propose. This is
prior art that must be engaged with directly, not merely cited. The distinguishing move left
available is *partial* state: Materialize's views are fully materialized.

**(b) Noria is the only system that combines partial materialization with upqueries, and it
explicitly gives up strong consistency to do so** [27]. Its own words: "Eventual consistency is
attractive for performance and scalability, and is sufficient for many web applications."
Measured: 5x higher load than a hand-optimized MySQL baseline on Lobsters (5,000 vs ~1,000
pageviews/sec) [27].

That pairing — partial state ⇒ eventual consistency, in the one system that tried it — is the
single most load-bearing fact in this survey for anyone proposing partial materialization *with*
strict serializability. It is not a proof of impossibility. It is an unrefuted empirical
correlation with one data point, and it is exactly the kind of claim a formal frontier theorem
would be valuable for.

### 5.2 Does anyone do all four?

No. The nearest claims:

- **SAP HANA** has an OLTP path, a columnar OLAP engine, and a separate graph engine — three of
  four — but its streaming component (Smart Data Streaming) was a separate, since-deprecated
  product [U], and its graph engine is a distinct engine rather than the relational optimizer.
- **ClickHouse** has materialized views that fire on insert; these are not incremental view
  maintenance in the DBSP sense (no correct handling of deletes/retractions in general) and
  ClickHouse has no general transactions.
- No system in the survey has ● in three of the four columns.

### 5.3 Verdict on Q3

Streaming/IVM as a *first-class part of a transactional engine* is the genuinely unbuilt quadrant.
It is not obviously impossible. The DBSP algebra [8] provides the formal substrate; Materialize
proves strict serializability over IVM is implementable; Umbra proves single-format HTAP is
implementable. Nobody has put them in one process.

---

## 6. Question 4 — "The optimizer should not produce a bad join order or wrong indexes"

This section answers the user's most important question. **The requirement as stated is provably
unachievable.** Here is the chain, from empirical to theoretical.

### 6.1 Empirical: the problem is cardinality estimation, and it has not improved in a decade

Leis et al., VLDB 2015 [4] — the Join Order Benchmark paper. JOB: 113 queries from 33 templates,
3–16 joins (avg 8), over the real 3.6 GB / 21-table IMDB dataset. Findings:

- PostgreSQL base-table selectivity: median q-error 1.00, **95th percentile 6.10, max 207**.
- Join estimates exceeding 10x error: **16% at 1 join, 32% at 2 joins, 52% at 3 joins.** Errors
  grow with join count. A commercial system frequently estimated **exactly 1 row** for 3+ join
  queries.
- With PostgreSQL's estimates, **38% of queries ran >2x slower than optimal and 5.3% ran >100x
  slower** [4].
- The **ranking of causes is decisive**: cardinality estimation ≫ cost model ≫ plan enumeration.
  Cost-model parameter tuning moved median prediction error 38% → 30%; a *trivial* cost function
  was 34% faster than PostgreSQL's real one. Exhaustive DP vs. randomized Quickpick-1000: median
  1.05x with real estimates, ~1.00–1.07x with true cardinalities [4].
- And the finding that directly refutes "give it the right indexes": misestimation damage is
  **worse when more indexes are available**, because a bad estimate plus an index invites a
  catastrophic nested-loop plan [4].

Leis et al., VLDB 2025 — *Still Asking: How Good Are Query Optimizers, Really?* [2], the ten-year
retrospective. Findings:

- Estimation errors "of one order of magnitude or more for larger expressions occur routinely
  across **all** systems" [2].
- ~10% of JOB queries did not complete in reasonable time on PostgreSQL 9.4 because of cardinality
  errors [2].
- **"Learning-based techniques have yet to see widespread adoption in industry."** Microsoft
  reported limited production gains with Bao-style approaches, citing "noisy and expensive
  performance measurements" [2].
- The more-indexes-is-worse finding was independently confirmed by Microsoft on SQL Server [2].

SQLStorm, VLDB 2025 [9] — 18,251 LLM-generated queries over StackOverflow data, 12,384 unique query
plans (vs TPC-H's 30, TPC-DS's 256). **Cardinality estimation errors reached 14 orders of
magnitude.** Only 12,587 of 18,251 queries produced identical results in at least two systems —
**5,664 queries exposed cross-system semantic disagreement**, which is its own warning for anyone
promising wire-compatible drop-in replacement.

Guy Lohman's 2014 framing [15] remains exact: cardinality estimation is "the root of all evil, the
Achilles Heel of query optimization." Cost models introduce errors up to ~30%; cardinality
estimation produces "errors of many orders of magnitude" — he gives an example of 7 orders of
magnitude from redundant predicates. His three named root causes are the ones still unsolved: host
variables / parameter markers unknown at compile time, join-predicate selectivity, and column
correlation [15].

### 6.2 Theoretical result #1 — you cannot estimate accurately from a sample

Charikar, Chaudhuri, Motwani & Narasayya, PODS 2000. As restated by Freitag & Neumann [30]:

> "A powerful negative result due to Charikar et al. states that **any estimator which examines at
> most n rows of a table with N rows must incur an expected ratio error in Ω(√(N/n)) on some
> input.**"

Practical consequence, measured by Harmouch & Naumann's VLDB 2018 experimental survey [31]: to
reach **1% relative error, the GEE estimator must sample more than 90% of the dataset.** Sampling
does not scale; you must either scan everything (sketches) or accept unbounded error on adversarial
inputs. **[V]** — statement verified against two independent secondary sources quoting the theorem;
the original PODS PDF was paywalled (see §11).

This is the formal core of "you cannot always know the cardinality."

### 6.3 Theoretical result #2 — even abandoning estimation entirely, 4x is the floor

This is the result that settles the user's question, and it is the one most people in this debate
have never heard of.

Dutt & Haritsa's **Plan Bouquets** (SIGMOD 2014) [7] takes the radical position: throw away
compile-time selectivity estimation completely, and instead *discover* selectivities at runtime by
executing a geometrically-spaced sequence of partial plans under cost budgets. This buys a
**provable worst-case bound on suboptimality (MSO — maximum sub-optimality)**:

- **1D: MSO ≤ 4**, achieved with geometric cost progression at ratio r = 2.
- **And this is optimal: "No deterministic online algorithm can provide an MSO guarantee lower
  than 4 in the 1D scenario."** [7]
- Multi-dimensional: **MSO ≤ 4ρ**, where ρ is the maximum number of plans on an isocost surface
  (typically kept to ~10 by anorexic reduction).

Read that second bullet carefully. It is a **lower bound on plan quality that holds for any
deterministic strategy**, including one with a perfect cost model, unlimited compile time, and the
freedom to execute and observe. The best any optimizer can guarantee in the simplest possible
setting — one unknown selectivity — is **4x worse than the oracle-optimal plan**. In realistic
multi-predicate settings the guarantee degrades to 4ρ, i.e. roughly 40x with ρ≈10.

And Plan Bouquets *pays* for that guarantee: hours of compile-time POSP identification, multiple
partial executions per query (12–19 in their experiments), invalidation on database growth, and
inapplicability to updates and to queries lacking Plan Cost Monotonicity [7].

**So: "the engine should never produce a bad join order" is false in the strongest sense. It is not
merely hard. There is a proven 4x floor, and the only known technique that reaches even that floor
is unusable for OLTP.**

### 6.4 Theoretical result #3 — asymmetry, and what *is* achievable

The productive reframing, well established in the literature: **stop trying to be optimal; make
underestimation impossible.**

Suciu's group and others formalized this as *pessimistic cardinality estimation* — compute provable
**upper bounds** (AGM / polymatroid bounds, SafeBound, LpBound) rather than point estimates [32].
The asymmetry argument, in the SIGMOD blog's words: "systems that overestimate only need to be
accurate for *at least one* good plan, while systems that underestimate need to be accurate for
*every* bad query plan" [32]. Underestimation is what produces catastrophes (nested loops on
millions of rows); overestimation merely produces conservatism.

The 2025/2026 counterpart, xBound [33], adds provable **lower** bounds on join sizes from
lightweight statistics (ℓ-norms of degree sequences, min/max, Theta sketches, heavy-hitter
partitions). Its motivating measurement is striking: in Microsoft Fabric Data Warehouse,
**0.05% of extreme underestimates account for 95% of all CPU under-allocation** [33]. Results:
corrects 23.6% of underestimates, reduces P90 q-error on underestimated queries by **35.8x**,
speedups up to 20.1x, at 67–197 MB of statistics and <70 ms estimation time [33].

Note what these give you and what they don't. They give **one-sided guarantees** — "never below X,"
"never above Y." They do not give plan optimality. The blog states the position plainly: perfect
cardinality estimation is "impossible"; optimizers should "simply avoid catastrophically bad plans"
rather than find optimal ones [32].

### 6.5 Learned optimizers: do they actually beat traditional ones robustly?

**No, not robustly, and not in production.**

Wang et al., VLDB 2021, *Are We Ready For Learned Cardinality Estimation?* [34] — 5 learned methods
(Naru, MSCN, LW-XGB/NN, DeepDB, DQM) vs 9 traditional, on 4 real datasets:

- Accuracy: learned methods win big on static data — 28x, 51x, 938x, 1758x better max q-error than
  commercial systems on Census/Forest/Power/DMV [34].
- Training: seconds for DBMS statistics vs **minutes to 4+ hours** for Naru [34].
- Inference: data-driven methods need **5–20 ms per query, up to 20x slower than a DBMS estimator**
  — a hard blocker for OLTP [34].
- **Updates**: learned methods "cannot catch up with fast data updates" and often *underperform*
  traditional DBMSs as update frequency rises [34].
- **Correlation**: q-error of *all* estimators rises 10–100x when two columns become functionally
  dependent [34].
- **Logical consistency**: tested against monotonicity, consistency, stability and fidelity, most
  learned methods violate multiple rules. Only DeepDB satisfied all four; regression-based methods
  violated all but stability [34].
- Verdict: **not ready for deployment** [34].

*Query Optimization in the Wild* (2025) [35] confirms the industrial picture: production systems
still run System R / Starburst / Volcano / Cascades frameworks; "the industry remains hesitant to
deploy learned QO techniques in real-world production systems," blocked by explainability,
debuggability, regression risk, and training cost. What *has* shipped is the **"pacemaker
approach"** — ML augmenting an existing optimizer: SQL Server Cardinality Estimation Feedback and
Degree-of-Parallelism Feedback, Google BigQuery History-Based Optimizations (2025) [35].

### 6.6 What actually works in practice: the three defensible mechanisms

1. **QO–QE collaboration / adaptive execution.** Runtime filters (Bloom/bitmap), mid-execution
   reoptimization on observed cardinalities, fallback to proven plans [35]. Leis et al. found that
   simply **disabling risky nested-loop joins and enabling runtime hash-table resizing dropped
   >2x-slower queries from 38% to under 4%** [4]. That is the single highest-leverage intervention
   in the entire literature, and it is not an estimation improvement — it is a *plan-space
   restriction*.
2. **Plan pinning / workload memory.** SQL Server Query Store forced plans; Oracle SQL Plan
   Management baselines [35]. These do not make plans good; they make plans *stable*, which is what
   operators actually want.
3. **One-sided bounds.** Pessimistic upper bounds [32] and lower bounds [33] to clip the estimator
   and eliminate the catastrophic tail.

### 6.7 Verdict on Q4 — the straight answer

**"The optimizer should never produce a bad plan" is not achievable, and a system design that
assumes it is will fail.** Concretely:

- You cannot estimate reliably from samples: Ω(√(N/n)) ratio error is unavoidable [30].
- You cannot escape by measuring instead of estimating: MSO ≥ 4 for *any* deterministic online
  strategy, even in one dimension [7].
- You cannot escape by learning: learned estimators are more accurate on static data and worse
  under updates, 20x slower at inference, and logically inconsistent [34]; industry has not
  adopted them [2][35].
- The damage is *amplified*, not reduced, by having more indexes available [4][2].

What **is** achievable, and what a thesis should claim instead:

- **Bounded** suboptimality, at a stated cost (Plan Bouquets: 4ρ) [7].
- **No catastrophic underestimation**, via provable one-sided bounds [32][33].
- **Plan stability** across executions, via pinning and workload memory [35].
- **Restricted plan spaces** that exclude the pathological operators — this is empirically the
  biggest single win available [4].

If the Niles/Nilestream design needs an optimizer property to lean on, lean on *"no plan worse than
kx the oracle plan, for a stated k"* or *"no plan containing an unbounded nested loop"*. Never on
*"no bad plan."*

---

## 7. Question 5 — Compiled vs vectorized, and the true value of SIMD

### 7.1 What each system actually does

| System | Model | Detail |
|---|---|---|
| **Umbra / CedarDB** | **Adaptive: both** | Custom IR → bytecode interpreter first ("Flying Start"), promoted to LLVM-compiled machine code when runtime justifies it [1] |
| **HyPer** | Compiled | LLVM, data-centric operator fusion, tuples in registers |
| **DuckDB** | Vectorized | pull-based vectorized interpretation |
| **Photon** | **Vectorized by explicit choice** | C++; reasons documented below [6] |
| **Velox** | Vectorized | C++ library, Meta's shared execution engine |
| **InkFuse** | **Both from one abstraction** | suboperator IR; the vectorized interpreter is a *byproduct* of the compiler [36] |

### 7.2 Kersten et al., VLDB 2018 — there is no winner

Two systems built on the same codebase (Typer = compiled, Tectorwise = vectorized) [5]:

- **Compiled wins on computation-bound queries**: TPC-H Q1 **74% faster** — "keeps data in
  registers and thus needs to execute fewer instructions."
- **Vectorized wins on memory-bound queries**: Q3 and Q9 (hash-join-heavy) **4–32% faster**,
  growing to **40% at larger scale** — "vectorization is better at hiding cache miss latency."
- Conclusion: "these are not large differences, especially when compared to the performance gap to
  other systems." Choose on **non-performance grounds** — compiled for OLTP and language support,
  vectorized for profiling, adaptivity, and lower compile time [5].

### 7.3 The SIMD number — the user's claim is CORRECT

From the same paper [5]:

| Setting | SIMD speedup |
|---|---|
| Microbenchmark: selection | **8.4x** |
| Microbenchmark: hash computation | 2.3x |
| **Real TPC-H Q6 (selection)** | **1.4x** |
| **Real TPC-H Q3 / Q9 (joins)** | **~1.1x** |

The paper's explanation: "most OLAP queries are bound by data access, which does not (yet) benefit
much from SIMD, and not by computation" [5].

**Verified: ~1.4x on real TPC-H, not 8x.** The 8x figure is a microbenchmark result and quoting it
as an engine-level speedup is a category error.

Photon's numbers are not a contradiction: its **4x average across TPC-H (max 23x)** [6] is measured
against Databricks' *JVM-based* previous engine, and bundles C++ rewrite, memory management,
adaptivity and SIMD together. Its component-level SIMD wins (3x on ASCII uppercase, 3.5x on hash
join probes, 5.7x on grouped aggregation [6]) are kernel-level, not query-level.

### 7.4 Photon's rationale for rejecting code generation — worth quoting for a design decision

Databricks chose vectorized interpretation over codegen for three documented reasons [6]:

1. **Debuggability** — "The interpreted approach was 'just C++', for which existing tools are highly
   tailored. Techniques such as print debugging were also much easier."
2. **Observability** — codegen "makes observability difficult" because operator boundaries collapse
   into fused loops; the vectorized model preserves per-operator metrics.
3. **Adaptivity** — dynamic dispatch supports runtime adaptation naturally; codegen would need to
   "compile a prohibitive number of branches at runtime or re-compile parts of the query
   dynamically."

Given §6, point 3 is not a minor engineering preference. Runtime adaptivity is the *primary*
defence against bad plans, and codegen makes it harder. A design that mandates "compiles queries
to machine code without an interpreter" is trading away the main mitigation for the optimizer
problem it also claims to solve.

### 7.5 InkFuse resolves the dichotomy

Incremental Fusion [36] introduces a **suboperator IR below relational algebra**. Because
suboperators satisfy an "enumeration invariant" (parametrised over finite sets), the engine
pre-generates the complete vectorized interpreter by running its own JIT over every operator
instantiation — "the vectorized interpreter becomes a free byproduct of carefully choosing the
right abstraction for code generation" [36].

Measured [36]:
- At SF 0.1 (100 MB): vectorized interpreter finishes all queries in <20 ms; traditional compilation
  needs >40 ms just to compile.
- At SF 100 (100 GB): compilation overhead negligible; which paradigm wins is query-dependent.
- Hybrid backend: start interpreted, codegen on background threads, switch at **morsel granularity**,
  spending only **5% of morsels** benchmarking each backend.
- Competes with DuckDB and Umbra across SF 0.1–100 without Umbra's compilation infrastructure.

**Design implication:** "compiles to machine code without an interpreter" is the wrong requirement.
The state of the art — Umbra, InkFuse — is *both*, chosen adaptively, with the interpreter as the
low-latency entry path. Requiring "no interpreter" costs you short-query latency and adaptivity for
no measured throughput gain.

---

## 8. Question 6 — Strict serializability at speed

### 8.1 Who actually provides it

Distinguish three things carefully: **serializability** (equivalent to *some* serial order),
**strict serializability** (equivalent to a serial order *consistent with real time*), and
**snapshot isolation** (not serializable; permits write skew).

| System | Claimed | Actual | Note |
|---|---|---|---|
| Spanner | strict serializable | ✅ | TrueTime commit-wait; pays 2ε latency on every commit |
| CockroachDB | serializable (default) | ✅ serializable; single-key linearizable | uses hybrid logical clocks + uncertainty intervals rather than TrueTime |
| FoundationDB | strict serializable | ✅ | unbundled OCC; the resolver is the serialization point |
| Calvin / FaunaDB | strict serializable | ✅ | deterministic pre-ordering — sequencing *before* execution |
| **TiDB** | "REPEATABLE-READ" | ❌ **Snapshot Isolation**, advertised as RR for MySQL compatibility. **SERIALIZABLE is not supported.** Permits phantoms (P3) and write skew [24] | this is the exact "SI marketed as serializable" case the user asked about |
| YugabyteDB | Serializable / Snapshot / Read Committed | ✅ Serializable available (provisional records for reads); docs do **not** claim strict serializable / linearizable [23] | |
| **Materialize** | **Strict Serializable — default** | ✅ [29] | over incrementally maintained views |
| Umbra / HyPer | serializable MVCC | ✅ serializable [19] | single node; strict serializability follows trivially from single-node commit ordering **[U]** — not stated in the paper |
| ClickHouse | — | ❌ no general transactions | |
| DuckDB | serializable | ✅ but single-writer | |

TiDB is the headline finding for this question: a major "NewSQL" system whose strongest isolation
level is snapshot isolation wearing MySQL's `REPEATABLE-READ` label [24].

### 8.2 The measured and theoretical cost

**Theoretical bound #1 — Attiya & Welch (1994).** For a linearizable read/write register on a
network with message-delay uncertainty *u*, there are lower bounds of roughly **u/4 on read latency
and u/2 on write latency**; sequential consistency has no such bound [37]. Linearizability is
strictly more expensive than sequential consistency when clocks are not perfectly synchronised.
This is why Spanner needs TrueTime: it converts clock uncertainty into an explicit, bounded
commit-wait rather than an unbounded one.

**Theoretical bound #2 — the SNOW theorem (Lu et al., OSDI 2016).** No read-only transaction
algorithm can simultaneously provide all four of [38]:

- **S** — Strict serializability
- **N** — Non-blocking operations
- **O** — One response per server (one round, one version)
- **W** — compatibility with conflicting Write transactions

"No read-only transaction algorithm provides all of the SNOW properties" [38]. In plain terms:
**strict serializability plus latency-optimal reads plus concurrent conflicting writes is
impossible.** For a design whose selling point is fast reads from derived views *while* writes are
landing on a ledger, this is directly binding: your read path must give up either strict
serializability, or non-blocking, or single-round.

**Practical cost, single node.** Neumann, Mühlbauer & Kemper, SIGMOD 2015 [19] is the important
positive result: serializable MVCC via *precision locking* over the undo buffers, at close to
snapshot-isolation cost, on a single node. This is the concurrency control Umbra inherits. The
lesson is that **serializability is cheap in-process and expensive across a network** — the cost is
almost entirely coordination, not validation.

### 8.3 Strict serializability *combined with* columnar analytics

Systems that credibly do both:

- **Umbra / CedarDB** — serializable MVCC [19] over PAX storage with full OLAP execution [1]. The
  strongest instance in the survey. Single node.
- **Materialize** — strict serializable by default [29], but over *derived views only*; it is not a
  general read-write store.
- **SingleStore, TiDB, Oracle, SQL Server, HANA** — analytics yes, but each with a seam, and TiDB
  without serializability at all.

**There is no distributed system in this survey that provides strict serializability over a
columnar analytical path with a single copy of the data.** That is a genuinely open engineering
target.

---

## 9. Question 7 — Wire compatibility as an adoption path

### 9.1 Who has done it

| System | Protocol | Approach |
|---|---|---|
| CockroachDB | PG wire | reimplemented from scratch |
| YugabyteDB | PG wire | **reuses the actual PostgreSQL query layer** — highest fidelity of any entry here |
| Neon / Databricks Lakebase | PG wire | real PostgreSQL, storage disaggregated [28] |
| CedarDB | PG wire + SQL grammar + system catalog [21] | reimplemented |
| Materialize | PG wire | reimplemented; *view* semantics, not table semantics |
| RisingWave | PG wire | reimplemented; read-only transactions [26] |
| TiDB | MySQL protocol | reimplemented |
| SingleStore | MySQL protocol | reimplemented |
| ReadySet | PG **and** MySQL wire | proxy in front of the real database [S] |

### 9.2 "Compatibility is not semantics" — the concrete gap

CockroachDB's own compatibility documentation is the best available evidence, because it is honest
and enumerated [25]. Unsupported: `CREATE DOMAIN`, range types, events, dropping primary keys,
XML functions, column-level privileges, XA syntax, database templating, single-partition deletion,
foreign data wrappers; advisory locks are **no-ops**. Behavioural divergences that will silently
change application results:

| Behaviour | PostgreSQL | CockroachDB |
|---|---|---|
| Float overflow | error | returns `+Inf` |
| Integer division `1/2` | `0` | `0.5` |
| Unary `~` precedence | low | high |
| Bitwise operator precedence | all equal | `&` > `#` > `\|` |
| Shift argument | modulo applied | no modulo |
| Subquery column naming | outer names preserved | `?column?` |
| CHECK with `INSERT ON CONFLICT` | validates input rows | validates result rows |

Note the class of these: they are not missing features you discover at deploy time. `1/2` returning
`0.5` instead of `0` is a **silent numerical difference**, and in a financial ledger it is a
correctness incident. Advisory locks becoming no-ops is a **silent loss of mutual exclusion**.

The second, less appreciated gap comes from SQLStorm [9]: across PostgreSQL, Umbra, DuckDB, Hyper
and two commercial systems, **only 12,587 of 18,251 queries produced identical results in at least
two systems**. 5,664 queries — 31% — exposed semantic disagreement *between existing systems that
all claim to implement SQL*. Wire compatibility gets you connected; SQL semantics compatibility is
a much larger and mostly unmeasured surface.

### 9.3 What the compatibility path actually costs

- **Protocol**: weeks. It is a documented message format.
- **Grammar + catalog**: months. CedarDB did grammar and `pg_catalog` [21].
- **Semantics**: years, and never complete. Type coercion rules, NULL/collation/ordering semantics,
  overflow, division, `NUMERIC` behaviour, error codes, `SQLSTATE` values, extensions, `pg_dump`
  round-tripping (still "in progress" for CedarDB in 2026 [22]).
- **Ecosystem**: unbounded. ORMs probe `pg_catalog` in undocumented ways; connection poolers,
  logical replication, CDC tools each have their own expectations.

---

## 10. Verdict: impossible vs unsolved vs already solved

### 10.1 Provably impossible (do not design around these)

1. **"The optimizer never produces a bad join order or picks a wrong index."**
   Three independent barriers:
   - **Estimation**: any estimator examining n of N rows incurs Ω(√(N/n)) expected ratio error on
     some input [30]; 1% relative error needs >90% sampling [31].
   - **Adaptation**: even discarding estimation and discovering selectivities by execution,
     **no deterministic online algorithm achieves MSO < 4** in 1D; realistic multi-dimensional
     bound is 4ρ [7].
   - **Learning**: learned estimators degrade under updates, cost 5–20 ms inference, and violate
     monotonicity/consistency [34]; industry has not adopted them [2][35].

2. **Strict serializability + latency-optimal, non-blocking, single-round reads + concurrent
   conflicting writes.** SNOW theorem [38]. You must give up one of the four.

3. **Linearizable operations at zero latency cost under clock uncertainty.** Attiya–Welch lower
   bounds of ~u/4 (read) and ~u/2 (write) [37]. Spanner's commit-wait is the tax made visible.

4. **Exact-result guarantees across "wire-compatible" engines.** Not a theorem, but as close to
   settled as empirical results get: 31% of a large realistic query corpus disagrees across
   existing SQL systems [9]. "Drop-in replacement" cannot mean "bit-identical results."

### 10.2 Unsolved but not impossible (this is where the contribution is)

1. **All four workload classes in one engine.** No theorem forbids it. Nobody has done it. The
   pieces exist separately: PAX-in-B-tree for T+A [1], WCOJ for G [3], DBSP for S [8].

2. **Partial materialization *with* strict serializability.** Noria demonstrated partial state and
   chose eventual consistency [27]; Materialize demonstrated strict serializability and chose full
   materialization [29]. Nobody has demonstrated both. **This is a real and well-posed open
   problem, and it is precisely the frontier a formal impossibility/cost result would settle.**
   The survey found no existing theorem bounding the cost of consistency under eviction.

3. **Distributed strict serializability over single-copy columnar storage.** Every distributed HTAP
   system pays with a second physical representation [16][28]. No proof this is necessary.

4. **Variable-length path matching in standard SQL/PGQ.** Committed in PostgreSQL 19 without it
   [12]; open across the standard-conforming implementations.

5. **Bounded-suboptimality optimization at OLTP latencies.** Plan Bouquets gets the bound but needs
   multiple partial executions and cannot handle updates [7]. A cheap MSO-bounded optimizer is
   unbuilt.

### 10.3 Already solved — the user may not know

1. **Single-engine HTAP without a row/column seam.** Umbra, since CIDR 2020 [1]; commercially
   available as CedarDB since May 2025 [21]. PAX inside B-tree leaf pages. <6% buffer-manager
   overhead vs pure in-memory [1].

2. **Graph queries at graph-database speed inside a relational engine.** WCOJ with hash tries and a
   growing-join-detecting hybrid optimizer: ~77x on 4-clique, **zero regression on TPC-H/JOB**,
   VLDB 2020 [3]. You do not need a graph engine to get graph performance.

3. **Serializability at snapshot-isolation cost, single node.** Precision-locking MVCC, SIGMOD 2015
   [19].

4. **Strict serializability over incrementally maintained views, with a consistency ladder.**
   Materialize ships Strict Serializable / Serializable / Bounded Staleness as user-selectable
   isolation levels, today [29]. Any thesis proposing an epoch-based consistency ladder must
   position against this explicitly.

5. **The compiled-vs-vectorized argument is over.** Neither wins [5]; Umbra runs both adaptively
   [1]; InkFuse derives both from one IR [36]. "No interpreter" is a self-inflicted constraint that
   costs short-query latency and runtime adaptivity.

6. **SIMD is worth ~1.4x on real OLAP, not 8x** [5]. Budget accordingly.

7. **The highest-leverage optimizer fix is not better estimation.** Disabling risky nested-loop
   joins and enabling runtime hash-table resizing cut >2x-slower queries from **38% to under 4%**
   [4]. Restricting the plan space beats improving the estimates.

### 10.4 Adversarial notes on the premise

- **"FAST" is not one axis.** The requirement set contains a direct internal contradiction:
  compiled-to-machine-code-without-interpreter optimises peak throughput on long queries, while
  OLTP replacement for PostgreSQL/MySQL requires sub-millisecond latency on trivial queries where
  compilation *is* the dominant cost. InkFuse measures this: at 100 MB, interpretation finishes all
  queries in <20 ms while compilation needs >40 ms just to compile [36]. Umbra resolves it with an
  interpreter. The requirement as written forbids the known solution.
- **"Cache-aware and disk-aware data structures" plus "optimized for all four workloads" is a
  physical tension, not a software one.** OLTP wants row locality; OLAP wants column locality;
  graph traversal wants adjacency locality; streaming wants append locality. PAX resolves the first
  two within a page. Nothing in the literature resolves all four in one layout — Kùzu used CSR
  adjacency indices *in addition to* columnar property files [11].
- **More indexes make the optimizer problem worse, not better** [4][2]. "Provides indexes" and
  "never picks the wrong index" are in tension with each other.
- **The specialist-engine survivorship signal is bad.** Kùzu — technically excellent, MIT-licensed,
  outperforming DuckDB and Umbra on its target queries [11] — was archived by its own company
  within ~2.5 years [10]. Technical merit was not sufficient.

---

## 11. What I could not verify

1. **Charikar et al. (PODS 2000), exact theorem statement.** The ACM PDF returned HTTP 403.
   The Ω(√(N/n)) form is quoted verbatim from Freitag & Neumann, CIDR 2019 §3.1 [30], and
   corroborated qualitatively by Harmouch & Naumann [31] ("for every estimator based on a small
   sample, there is a dataset where the ratio … is arbitrarily large") and by Motwani et al., who
   state the bound is tight. **The constants and the probability qualifier are unverified.** Cite
   the original paper directly; do not cite my formula.

2. **SingleStore SIGMOD 2022 paper text.** ACM returned 403; details in the matrix come from
   lecture slides [S] and the tutorial taxonomy [16]. In particular I could not confirm whether
   SingleStore Universal Storage physically stores one copy or two — the slide deck says "two
   representations… not as separate copies," which is ambiguous. Verify before citing.

3. **Umbra/CedarDB strict serializability.** Umbra provides serializable MVCC [19]. Whether the
   product documents *strict* serializability (real-time ordering) is not stated in any source I
   could reach; CedarDB's docs do not publish an isolation-level page. Marked [U] in §8.1.

4. **SAP HANA's current isolation guarantees and the status of its streaming component.** No
   primary source reached. HANA's graph engine is a separate engine, but I could not verify whether
   it shares the relational optimizer.

5. **Neo4j / TigerGraph / Memgraph internals.** All vendor-benchmark material; no independent
   primary source found within scope. Their absence of columnar OLAP is inferred, not documented.

6. **Analyzing the Impact of Cardinality Estimation on Execution Plans in Microsoft SQL Server
   (VLDB 2023).** DOI returned 403. Its finding (misestimation damage rises with index count) is
   cited here at second hand via Leis et al. 2025 [2].

7. **"Distributed Transactional Systems Cannot Be Fast."** ResearchGate returned 429. Not used as a
   load-bearing citation; SNOW [38] and Attiya–Welch [37] carry that argument instead.

8. **Whether Materialize's strict serializability holds under all view topologies**, and its
   throughput cost, are undocumented in the sources reached. The isolation-level page states the
   guarantee and the latency tradeoff qualitatively only [29].

9. **CedarDB's `pg_catalog` fidelity in practice** — claimed [21], not independently tested.

---

## 12. References

[1] T. Neumann and M. J. Freitag, "Umbra: A Disk-Based System with In-Memory Performance," *CIDR*, 2020. https://db.in.tum.de/~freitag/papers/p29-neumann-cidr20.pdf
[2] V. Leis et al., "Still Asking: How Good Are Query Optimizers, Really?," *PVLDB*, vol. 18, p. 5531, 2025. http://www.vldb.org/pvldb/vol18/p5531-viktor.pdf
[3] M. J. Freitag, M. Bandle, T. Schmidt, A. Kemper, and T. Neumann, "Adopting Worst-Case Optimal Joins in Relational Database Systems," *PVLDB*, vol. 13, no. 12, pp. 1891–1904, 2020. https://www.vldb.org/pvldb/vol13/p1891-freitag.pdf
[4] V. Leis, A. Gubichev, A. Mirchev, P. Boncz, A. Kemper, and T. Neumann, "How Good Are Query Optimizers, Really?," *PVLDB*, vol. 9, no. 3, pp. 204–215, 2015. https://www.vldb.org/pvldb/vol9/p204-leis.pdf
[5] T. Kersten, V. Leis, A. Kemper, T. Neumann, A. Pavlo, and P. Boncz, "Everything You Always Wanted to Know About Compiled and Vectorized Queries But Were Afraid to Ask," *PVLDB*, vol. 11, no. 13, 2018. https://dl.acm.org/doi/10.14778/3275366.3284966
[6] A. Behm et al., "Photon: A Fast Query Engine for Lakehouse Systems," *SIGMOD*, 2022. https://people.eecs.berkeley.edu/~matei/papers/2022/sigmod_photon.pdf
[7] A. Dutt and J. R. Haritsa, "Plan Bouquets: Query Processing without Selectivity Estimation," *SIGMOD*, 2014. https://dsl.cds.iisc.ac.in/publications/conference/bouquet.pdf
[8] M. Budiu, T. Chajed, F. McSherry, L. Ryzhyk, and V. Tannen, "DBSP: Automatic Incremental View Maintenance for Rich Query Languages," *PVLDB*, vol. 16, no. 7, 2023. https://www.vldb.org/pvldb/vol16/p1601-budiu.pdf
[9] T. Schmidt et al., "SQLStorm: Taking Database Benchmarking into the LLM Era," *PVLDB*, 2025. https://db.in.tum.de/~schmidt/papers/sqlstorm.pdf
[10] The Register, "KuzuDB graph database abandoned, community mulls options," 14 Oct. 2025. https://www.theregister.com/2025/10/14/kuzudb_abandoned/
[11] X. Feng, G. Jin, Z. Chen, et al., "KÙZU Graph Database Management System," *CIDR*, 2023. https://www.cidrdb.org/cidr2023/papers/p48-jin.pdf
[12] H. Dombrovskaya / depesz, "Waiting for PostgreSQL 19 – SQL Property Graph Queries (SQL/PGQ)," 31 Jul. 2026 (commit by P. Eisentraut, 16 Mar. 2026). https://www.depesz.com/2026/07/31/waiting-for-postgresql-19-sql-property-graph-queries-sql-pgq/
[13] Oracle, "Property Graphs in Oracle Database 23ai: The SQL/PGQ Standard." https://blogs.oracle.com/database/property-graphs-in-oracle-database-23ai-the-sql-pgq-standard
[14] DuckDB Community Extensions, "duckpgq." https://duckdb.org/community_extensions/extensions/duckpgq ; D. ten Wolde et al., "DuckPGQ: Bringing SQL/PGQ to DuckDB," *PVLDB*, 2023. https://dl.acm.org/doi/10.14778/3611540.3611614
[15] G. Lohman, "Is Query Optimization a 'Solved' Problem?," *ACM SIGMOD Blog*, 2014. https://dsf.berkeley.edu/cs286/papers/queryopt-sigmodblog2014.pdf
[16] G. Li and C. Zhang, "HTAP Databases: What is New and What is Next," *SIGMOD* (tutorial), 2022. https://dbgroup.cs.tsinghua.edu.cn/ligl/papers/sigmod22-tutorial-paper.pdf
[17] A. Prout et al., "Cloud-Native Transactions and Analytics in SingleStore," *SIGMOD*, 2022. https://dl.acm.org/doi/10.1145/3514221.3526055
[18] D. Huang et al., "TiDB: A Raft-based HTAP Database," *PVLDB*, vol. 13, no. 12, 2020. https://www.cs.purdue.edu/homes/csjgwang/CloudNativeDB/TiDBVLDB20.pdf
[19] T. Neumann, T. Mühlbauer, and A. Kemper, "Fast Serializable Multi-Version Concurrency Control for Main-Memory Database Systems," *SIGMOD*, 2015. https://dl.acm.org/doi/10.1145/2723372.2749436
[20] Snowflake, "Limitations and unsupported features for hybrid tables." https://docs.snowflake.com/en/user-guide/tables-hybrid-limitations
[21] CedarDB, product site and "Announcing the CedarDB Community Edition," 14 May 2025. https://cedardb.com/blog/launch/
[22] CedarDB, "Feature Roadmap for CedarDB." https://cedardb.com/docs/roadmap/
[23] YugabyteDB, "Isolation levels." https://docs.yugabyte.com/stable/explore/transactions/isolation-levels/
[24] PingCAP, "TiDB Transaction Isolation Levels." https://docs.pingcap.com/tidb/stable/transaction-isolation-levels/
[25] Cockroach Labs, "PostgreSQL Compatibility," v26.2. https://docs.cockroachlabs.com/docs/v26.2/postgresql-compatibility
[26] RisingWave, "When to use RisingWave." https://docs.risingwave.com/faq/faq-when-to-use-risingwave
[27] J. Gjengset, M. Schwarzkopf, J. Behrens, et al., "Noria: dynamic, partially-stateful data-flow for high-performance web applications," *OSDI*, 2018. https://pdos.csail.mit.edu/papers/noria:osdi18.pdf
[28] The Register, "Databricks unifies OLTP and OLAP, depending on what counts as a copy," 3 Jul. 2026. https://www.theregister.com/databases/2026/07/03/databricks-unifies-oltp-and-olap-depending-on-what-counts-as-a-copy/5265733
[29] Materialize, "Isolation levels." https://materialize.com/docs/get-started/isolation-level/
[30] M. J. Freitag and T. Neumann, "Every Row Counts: Combining Sketches and Sampling for Accurate Group-By Result Estimates," *CIDR*, 2019, §3.1 — quoting M. Charikar, S. Chaudhuri, R. Motwani, and V. Narasayya, "Towards estimation error guarantees for distinct values," *PODS*, 2000, pp. 268–279. https://db.in.tum.de/~freitag/papers/p23-freitag-cidr19.pdf
[31] H. Harmouch and F. Naumann, "Cardinality Estimation: An Experimental Survey," *PVLDB*, vol. 11, no. 4, 2018. https://www.vldb.org/pvldb/vol11/p499-harmouch.pdf
[32] "The Case for Cardinality Bounds: Principled Conservatism in Query Optimization," *ACM SIGMOD Blog*. https://wp.sigmod.org/?p=3707 ; W. Cai, M. Balazinska, and D. Suciu, "Pessimistic Cardinality Estimation: Tighter Upper Bounds for Intermediate Join Cardinalities," *SIGMOD*, 2019. https://homes.cs.washington.edu/~suciu/sigmod-2019-pessimistic.pdf
[33] "The Case for Cardinality Lower Bounds" (xBound), arXiv:2601.13117. https://arxiv.org/html/2601.13117
[34] X. Wang, C. Qu, W. Wu, J. Wang, and Q. Zhou, "Are We Ready For Learned Cardinality Estimation?," *PVLDB*, vol. 14, no. 9, 2021. https://www.vldb.org/pvldb/vol14/p1640-wang.pdf
[35] "Query Optimization in the Wild: Realities and Trends," arXiv:2510.20082, 2025. https://arxiv.org/html/2510.20082v2
[36] B. Wagner, A. Kemper, and T. Neumann, "Incremental Fusion: Unifying Compiled and Vectorized Query Execution" (InkFuse). https://www.cs.cit.tum.de/fileadmin/w00cfj/dis/papers/inkfuse.pdf
[37] H. Attiya and J. L. Welch, "Sequential consistency versus linearizability," *ACM TOCS*, vol. 12, no. 2, pp. 91–122, 1994. https://dl.acm.org/doi/10.1145/176575.176576
[38] H. Lu, C. Hodsdon, K. Ngo, S. Mu, and W. Lloyd, "The SNOW Theorem and Latency-Optimal Read-Only Transactions," *OSDI*, 2016. https://www.usenix.org/system/files/conference/osdi16/osdi16-lu.pdf
[39] M. Marcus et al., "Bao: Making Learned Query Optimization Practical," *SIGMOD*, 2021. https://people.csail.mit.edu/tatbul/publications/bao_sigmod21.pdf
[40] F. Wolf et al., "Robustness metrics for relational query execution plans," *PVLDB*, vol. 11, no. 11, 2018. https://dx.doi.org/10.14778/3236187.3236191
