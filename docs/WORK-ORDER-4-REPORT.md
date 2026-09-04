# Work Order 4 — what this cycle changed, measured, and declined

**Repository:** `niles`, on `review/thesis` from `23c7655`.
**Companion:** `gbs`, on `review/F-18` from `c0f985c` (`docs/WORK-ORDER-4.md` there).

Every number below is from `results/` or from a `callgrind` run whose recipe is in
`docs/BENCHMARK.md`. Nothing here is typed in from an estimate; where a figure is an estimate
it says so.

## The headline

The E16 contract table's `analytical` composite went from **0.66× PostgreSQL to 1.32×**, and
all four statements both engines can express are now at or above the baseline:

| statement | before (start of cycle) | after |
|---|--:|--:|
| `group_by_cur` | 4.06× | 4.47× |
| `group_by_acct` | **0.48×** | **1.04×** |
| `top_ten_by_sum` | **0.59×** | **1.18×** |
| `sum_negative` | 2.38× | 2.27× |
| **composite** | **0.66×** | **1.32×** |

None of that came from widening the fragment, relaxing a refusal, or changing what a query
means. Every step is compared against the reference evaluator over the same base, and the wire
bytes are byte-identical to what the previous cycle sent.

Two things had to be true before those numbers meant anything, and only one of them was.
**T-01 is the reason the table is a comparison at all**: PostgreSQL was being measured over
20,000 rows and Nilestream over 40,000, so every ratio in the previous table was a ratio
between two different problems.

## What was done

### T-01 — E16 became a comparison

`--nls-rounds` defaulted to `2`, applied to one target only, and appeared in neither the
results header nor the reproduction recipe. Six changes: equal seeding on both targets through
`Target::prepare(accounts, rounds)`; a `Target::base_rows()` check that turns an unequal base
into a `NOT RUN` row with both counts rather than a ratio; `* Rounds per account:` and
`* Base rows per target:` in the results header, read back by the drift test; the two targets
**interleaved** so host drift lands on both sides; a one-sided parity band (the contract is a
floor, and a target that beat it was being marked as missing it); at least 25 analytical
executions per run so a p99 is a percentile of something; and an empty answer that describes
the same columns as a non-empty one.

### T-02 — a concurrency instrument, and what it found

`bench --connections 1,2,4` drives a point read and a durable append from N threads, each with
its own socket, released from a barrier so the clock covers operations and not sockets;
latencies pooled across threads before the percentiles. Published as E19, in its own document
with its own type, so a 4-connection figure cannot reach the single-connection contract table.

| workload | target | 2 → 4 | reading |
|---|---|--:|---|
| point | postgres | 1.16× | rises |
| point | nilestream | 0.85× | falls |
| durable | postgres | 1.40× | rises |
| durable | nilestream | 0.98× | flat |

PostgreSQL keeps scaling at the widest level on both workloads; Nilestream goes flat on
appends and falls on reads. That is F-06's contention, measured for the first time.

### T-04 / T-05 / T-06 — the served path

Three tasks, one theme: the engine was doing work whose result it already had.

| | before | after |
|---|--:|--:|
| `served_group_by_acct` allocations | 95,035 | **12,602** |
| `RevEngine::query` instructions | 51.0 M | **27.8 M** |
| in-process, 10,001 groups | 7.3 ms | **2.5 ms** |
| `served_top_ten` allocations | 22,631 | **12,628** |
| top-ten instructions | 50.7 M | **32.8 M** |
| `served_point` allocations | 20 | **12** |

* **T-04**: a one-column group key is one integer. `Value` has exactly two variants, so a
  one-column key is always an `Option<i128>` — the specialisation needs no schema and no type
  inference, and its `match` is exhaustive, so a third variant would be a compile error rather
  than a silently wrong grouping. Accumulators moved into one arena. `finish` builds the Z-set
  in bulk, because the groups come out ascending and `eval::add` was doing thirteen
  `Vec<Value>` comparisons down a rebalancing tree to find a slot whose position was known.
* **T-05**: the reply is framed from the Z-set where it lies, into one buffer, with an integer
  formatter — instead of being rendered to `Vec<Vec<Option<String>>>` and parsed back. And the
  folded aggregate is no longer deep-cloned through the reference evaluator's `Cow` boundary
  when it *is* the output.
* **T-06**: `limit 10` keeps ten rows by bounded selection rather than sorting ten thousand.

### T-08 — the pipeline surface can sort descending

`order_by` mapped every key to ascending, so `t.order_by(|r| desc(r.amt)).limit(10)` returned
the ten *smallest*, silently — and the golden case that would have caught it could not be
written, because the surface had no spelling for the thing it got wrong. `desc` and `asc` have
been in the keyword registry with samples in exactly that form since it was written.

`tests/keyword_samples.rs` is the general guard: 143 of 174 samples now compile against a
fixture. See **Findings** below for the fifteen that do not.

### T-09 — one read model

`RevEngine` held a `proto_engine::PartialView` beside the REV runtime. The wire path read the
runtime; `read_point`, `stats()` and `evict_all` read the other one; nothing in the type said
which. Six tests warmed one and asserted about the other, and all six would have passed with
the served path disconnected. The second read model is gone and the tests ask the question the
way a client does.

### T-10 — the two cliffs, and `explain`

Three spellings of one question cost 20, 32 and 76,696 allocations. Adding a condition that
*narrows* a query made it a thousand times more expensive. Fixed by splitting conjuncts (sound
because every row surviving a filter satisfies every conjunct of its predicate) and by pushing
a group-key-only `having` below the aggregate. `explain` now names the serve path —
`view`, `index-fold`, `fold`, `materialise` — from `rev_engine::serve_path`, the function
`query` itself branches on.

## What was declined, and why

### T-07 — closed as measured-not-worth-doing

The trigger was: proceed only if E19 shows Nilestream point throughput at 2 connections
**< 1.5×** its 1-connection throughput. The audit predicted 0.8–1.08×, so it expected to fire.

The properly instrumented measurement says **2.21×** (21,607 → 47,783 ops/s). The trigger does
not fire and no code was written. The audit's figure came from a scratch harness; T-02's is
from an instrument with a barrier, per-thread sockets, distinct transaction identities per
level, and a re-seeded engine between levels.

**This is not "no contention exists."** E19's top step says Nilestream *falls* from 2 to 4
connections (0.85×) where PostgreSQL rises (1.16×). The contention is real and it is at 4, not
at 2. A future task should be triggered by the **2 → 4** row rather than the 1 → 2 one, and
should carry the lock-hold instrument T-07 specified; the objective (compile outside the
engine lock) is still the right first move.

### T-12 — closed as measured-not-worth-doing, with the arithmetic

The trigger required a **single** record-layout change to reduce `ledger_seeded` live bytes per
posting by ≥ 10% (≥ 21.1 B of 210.7). Measured sizes: `Posting` 48 B, `Hold` 48 B, `Row` 64 B,
`EpochRec` 96 B, `RowRef` 16 B. At 40,000 postings in 20,000 epochs over 10,000 accounts:

| candidate | saving per posting | share of 210.7 B |
|---|--:|--:|
| drop the duplicated `parent` hash (32 B per `EpochRec`, 2 postings each) | 16.0 B | 7.6% |
| box the `Hold` variant (`Row` 64 → 56 B: the largest remaining payload is `Posting`'s 48 B plus the tag) | 8.0 B | 3.8% |
| reserve the epoch `Vec` | ≈ 0 B | ≈ 0% |

The third is zero rather than the ≈ 13 B the audit estimated, and that is worth recording: the
seeded path builds each epoch's rows with `vec![a, b]`, which allocates capacity exactly 2, so
there is no growth slack to reclaim. The `Vec` slack the audit saw belongs to a different
construction path.

No single change reaches 10%. Closed without code, as the audit expected.

### T-13 — blocked on DM-02

The trigger is "proceed only if DM-02 is decided **against** per-key checkpoints". DM-02 is
still open. **This is a decision the author still owes**, and it is the only one blocking a
task in this cycle.

## Findings this cycle produced that were not in the audit

1. **The maintained view was answering queries it had no right to answer.** A scan may be
   restricted whenever `acct = k` is a *necessary* condition; the balance view may only answer
   when it is *sufficient*. The two were the same function, so `where acct = 7 and cur = 99` —
   which selects no rows — was answered with account 7's balance the moment T-10 taught the
   restriction to see conjuncts. Found by a test comparing against the reference evaluator, not
   by reading. Now `account_restriction` and `sole_account_filter` ask the two questions
   separately.

2. **Fifteen constructs the keyword registry documents and the compiler does not have.** Every
   artefact the thesis generates from that registry — Appendix B's three sub-tables, the
   grammar's terminal set, `docs/keywords.md` — describes these as part of the language:

   | keyword(s) | sample | what happens |
   |---|---|---|
   | `alter`, `add`, `column` | `alter table accounts add column tier: i32;` | NL0004 — the parser has no `alter` item |
   | `grant`, `revoke` | `grant debit<usd> on postings to teller;` | NL0004 — no DCL item |
   | `emit` | `emit v to sink;` | NL0004 — no `emit` item |
   | `crate`, `super` | `use crate::bank::transfer;` | NL0002 — `use` will not take the reserved word |
   | `and`, `in` | `where(|r| r.cur == usd)` | NL0501 — a currency-literal comparison has no lowering |
   | `as` | `select(|r| (r.amt as label("amount")))` | NL0501 |
   | `between` | `where(|r| r.amt between (0.00 usd, 100.00 usd))` | NL0501 |
   | `exists` | `where(|r| exists(holds.for_acct(r.acct)))` | NL0501 — a correlated subquery |
   | `any` | `where(|r| r.tags.any(|t| t == "vip"))` | no collection-valued column type |
   | `lineage` | `serve { lineage: full }` | not a serve-contract key the parser accepts |
   | `recorded_at` | `where(|r| r.recorded_at <= #4200)` | not a projectable system column |

   Each is recorded in `NOT_COMPILED` in `tests/keyword_samples.rs` with its reason, and the
   list's length is asserted, so the next one fails the build rather than joining them
   silently. **Appendix B's scope claim should be read against this table**: the language
   *reserves* these words and the compiler does not implement them. Either the samples are
   aspirational and should be marked as such in the registry, or the constructs are owed.

3. **`memprobe`'s own budget test could not have failed** (found before the compaction, recorded
   here for completeness): the counting allocator was not installed in the test binary, so every
   budget was compared against zeros. Fixed with a `#[global_allocator]` under `cfg(test)` and a
   `memory::installed()` assertion at the top of the test.

## Gates

`make gate` and `make reproduce` are green on every commit of this branch. The `psql`
conformance transcript is byte-identical to the one committed before T-05, which is the check
that the framer changed the cost and not the protocol.

## What is not done

Part A's **T-32** (binary wire format, streamed constant-memory replies, a lean client for both
targets) and **T-33/T-34/T-35** (the warm-report row, E23 report scaling, E24 memory and storage
scaling) are not started. So are **Part B** (T-15–T-22, erasure) and **Part C** (T-23–T-31,
general-purpose ledger, SQL import, entity resolution). Each is a multi-day task in its own
right; none is blocked by anything in this cycle except T-33's dependency on T-32.
