# L-EFF — does the language buy efficiency? Design

**Status: pre-registered, not run.** Cycle 17, card C17-16. This document is committed on its own,
before any program or harness of the study exists. A change to it after this commit is a
deviation, listed with its reason in the results file, never made after seeing a number it
would move. No run is scheduled this round.

## 1. The question and the rule

Every language study so far asked about expressiveness and safety (E30, E30b′, E30c). None asked
whether Niles's plans are faster or leaner than what a competent engineer writes in PostgreSQL by
hand. DB-1 and R2-b made annotated SQL the language and Niles its specification; L-EFF asks the
question that would reopen that on efficiency grounds alone.

**The rule, fixed here:** Niles's plans are justified for efficiency only if, at 10⁵ accounts,
they beat hand-written PostgreSQL by the joint gate (§6) on **at least half of the task classes**
(two of four), on the class's primary latency metric (§5), with no class where hand-written
PostgreSQL beats them on memory by the joint gate. Otherwise efficiency is not a reason for
Niles's plans, and the thesis says so.

**The second, smaller prediction:** annotated SQL+C+L and hand-written PostgreSQL run the same
statements, because the annotations are comments the server ignores, so their run-time
difference is **zero** by construction. L-EFF checks this rather than assuming it: any cell where
the two differ by the joint gate is a finding about the harness or the annotations, reported as
such.

## 2. The arms

| arm | what runs | how it is written |
|---|---|---|
| **PG** | PostgreSQL 16, hand-written SQL and PL/pgSQL | by the builder, as a competent engineer would, with the indexes the task needs, before any Niles plan is timed |
| **SQL+C+L** | the same text as PG, with the dialect's annotations | PG's text plus annotations; checked by `nilescheck-sql`; nothing else changes |
| **Niles** | Niles lowered to the IR and served by Nilestream | E30's Niles programs where they exist; written in the same style where E30 has none |

PG is written **first**, and frozen before Niles is timed, so the baseline cannot be tuned
against the result.

## 3. The corpus

E30's thirty tasks, in its four classes (`docs/study/E30-syntax-design.md` §3): Q (10 queries), T
(10 banking transactions), V (5 view contracts) and B (5 temporal and bitemporal reads). A task an
arm cannot express is recorded as **not expressible** for that arm, as E30 recorded it, and does
not enter that class's comparison for that arm.

## 4. The data and sizes

E30's generator (`crates/syntax-study`), scaled: 10³, 10⁴ and 10⁵ accounts, the history at 20
postings per account, five seeds (1, 7, 42, 100, 2024). Every arm loads the same rows, and the
row count and SHA-256 of the canonical rows are asserted equal per arm before any timing. Per
M-NOISE (cycle 17, C17-04: the container's refusals are a host effect), 10⁵ runs on the author's
Mac (Host C); 10³ and 10⁴ may run in the container.

## 5. The metrics

| metric | what | primary for |
|---|---|---|
| latency p50 and p99 | client-observed, over the PostgreSQL wire, simple protocol, per task, 1 and 4 clients | Q, V, B: read latency; T: commit latency |
| memory | PostgreSQL: the arm's derived state (`pg_total_relation_size` of indexes and views the task adds). Nilestream: view-state bytes (E27b §4.1) | every class |
| plan shape | PostgreSQL: `EXPLAIN (ANALYZE, BUFFERS)` node types and rows; Nilestream: the IR operator chain and rows touched | reported, never judged |

Every answer of every arm is checked against E30's oracle for that task; any divergence refuses
the cell.

## 6. Statistics

E27b's, unchanged: per (task, size, seed), medians and MADs over the measured runs; the joint gate
(at least 10% and at least 3 pooled MADs, with the warm-up floor); refused on oracle divergence,
a missing metric, or warm-up MAD above 15% of the median. A class's verdict at a size: an arm
when at least three seeds give it and none gives the other; otherwise as E27b's §8 aggregates.
Run lengths are E27b's.

## 7. Not measured, and said so

Build time, binary size, and developer effort. Concurrency beyond 4 clients. Durability cost
(every arm runs with its durable configuration; the comparison does not vary it).

## 8. Output

`results/L-EFF.md`, rendered from per-point files, with its manifest rows, when the study runs.
