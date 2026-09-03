# WORK-ORDER-3 — report

What was executed, what it measured, and what it did not do.

## 1. Host and hashes

| | |
|---|---|
| Host | `Linux 6.18.44-fc-v24 x86_64`, **2 cores**, 8,023 MB |
| Toolchain | cargo 1.95.0, PostgreSQL 16.13, valgrind 3.22.0 |
| `fsync` cost, measured at run time on `$PGDATA` | **145.5 µs** — ceiling 6,874 durable commits/s per connection |
| niles | `2090bc1` → **`cc7341a`** on `review/thesis` |
| gbs | `2671f97` → **`c0f985c`** on `review/F-18` |

**Adapter validation** (`crates/gbs-nilestream`, which path-depends on the niles checkout) was
run once against the final pair — niles `cc7341a` with gbs `3d3faa0` — and passed 40/0 (gbs's final commit, `c0f985c`, adds a document only). It is
the only cross-repo check and it names both hashes, as §5 requires.

Both trees were clean at `2090bc1` / `2671f97` and are clean at the final commits.
`git status --porcelain` is empty in both. **There were no pre-existing untracked files**, so
nothing in the diffs is inherited.

Tests: niles **778 passed / 0 failed / 5 ignored** (was 757/0/5); gbs **444 / 0 / 1** (was
443/0/1) plus the adapter's 40/0; `make g3` 29 rows, 0 failures.

The host is not steady. Across the session PostgreSQL — which nothing here touches — moved by
up to 20% between runs of the same benchmark, so every wall-clock figure below is quoted with
its within-run PostgreSQL control, and §6's variance policy is what decides whether a
difference is a result.

---

## 2. Before and after, per task

### Wall clock — E16, 5 runs, medians (ops/s)

Measured with the same command at both ends: `bench --calibrate --run --pg-port 5432
--host-nls --accounts 10000 --operations 500 --runs 5`. The "before" column is the repaired
harness (T-02) against the unmodified engine, so both columns compare the same statements;
the ratio column is what the contract is judged on.

| Statement / workload | PG before | NLS before | PG final | NLS final | NLS ÷ PG |
|---|--:|--:|--:|--:|--:|
| analytical (composite, common set) | 187.4 | 25.4 | 155.0 | 88.4 | **0.136 → 0.570** |
| `analytical:group_by_cur` | 248.1 | 27.2 | 249.0 | 663.5 | **0.110 → 2.665** |
| `analytical:sum_negative` | 503.1 | 31.0 | 481.2 | 586.1 | **0.062 → 1.218** |
| `analytical:group_by_acct` | 102.1 | 20.2 | 91.1 | 41.0 | 0.198 → 0.450 |
| `analytical:top_ten_by_sum` | 124.6 | *(refused)* | 113.1 | 61.1 | — → 0.540 |
| point | 7,879 | 9,164 | 7,908 | 13,001 | 1.163 → **1.644** |
| oltp | 3,032 | 2,886 | 2,676 | 2,800 | 0.952 → 1.046 |
| durable | 3,355 | 2,841 | 2,804 | 3,007 | 0.847 → 1.073 |

Spread, from the published run: analytical MAD 0.4–0.8% of median, point 2.8% (PG) and 11.1%
(NLS), oltp 4.4%/7.3%, durable 4.2%/6.5%.

The **point miss rate** moved from `1.0000 ± 0.0000` — a constant, not a measurement — to
**`0.3014 ± 0.0020`**.

### Memory — E18, deterministic counters

In-process, allocations exact. Never combined with the wire numbers above.

| Scenario | Before | Final | |
|---|--:|--:|---|
| `served_group_by_cur` | 173,363 allocations / 33.1 MB | **25 / 3.1 kB** | peak 23.9 MB → 1.8 kB |
| `served_sum_negative` | 156,692 / 33.5 MB | **23 / 3.1 kB** | peak 19.2 MB → 1.9 kB |
| `served_group_by_acct` | 266,706 / 38.9 MB | **95,035 / 8.1 MB** | peak 24.9 MB → 3.6 MB; O(groups), 10,001 of them |
| `served_point` | 33 / 5.4 kB | **20 / 644 B** | and no base rows touched on a hit |
| `zset_base_at` | 2.2 / row | **1.2 / row** | 400 → 240 B per row |
| `rev_read_hit` | 2 | 2 | unchanged |
| `ledger_seeded` | 2.3 / posting | 3.8 / posting | **worse**: the maintained view's cost |
| `append_in_memory` | 4 / txn | 7 / txn | **worse**: same cause |
| gbs `balance_fold_k20000` | 20,003 / fold | **4 / fold** | 1,466 µs → 532 µs; constant in `k` |
| gbs `balance_fold_k10` | 13 / fold | **4 / fold** | 1.09 µs → 0.42 µs |
| gbs `seal_and_append` | 21.1 / txn | 21.1 / txn | unchanged |

### Instructions — callgrind, deterministic

| | Ir per point query |
|---|--:|
| compile (parse, resolve, typecheck, lower, verify) | 177,064 |
| serve | 7,693 |
| compile share of the daemon's own work, before T-09 | **95.8%** |
| after T-09, amortised over 1,000 queries of one shape | ≈ 2.2% |

---

## 3. Every target, and whether it was met

| Task | Target | Result |
|---|---|---|
| T-01 | instrument reproduces the audit's baselines; `make reproduce` clean | **met** — every figure reproduced exactly (173,363 / 266,706 / 156,692 allocations; 8,418,859 B live) |
| T-02 | per-statement table, common-set composite, coverage row, no base drift | **met** — composite 0.118× → 0.136× on the same engine, i.e. the old number overstated the gap by 15% |
| T-03 | `order by sum(amt) desc limit 10` matches PostgreSQL's answer; alias resolves; unknown key refused | **met** — verified over the wire and by three golden cases |
| T-04 | `group by acct` ≤ 1.6 ms in-process; ≤ 10 allocations/statement; peak ≤ 1 MB | **partly met, and the target was wrong.** `group by cur` and `sum where` reach 25 and 23 allocations and ~1.5 ms; `group by acct` cannot reach 10 allocations *at all* — it returns 10,001 rows, and the order's target was derived from a reference fold that produced none. Held to O(groups) instead, which is the right invariant: 95,035 allocations for 10,001 groups, from 266,706 |
| T-05 | miss rate strictly between 0 and 1; hit ≤ 8 allocations | **met on the rate** (0.3014 ± 0.0020); **missed on allocations** — 20 per served point read, not 8, because the order's figure counted the runtime read (2) and not the `Rows` envelope the wire needs. The number is reported rather than the target quietly restated |
| T-06 | ≤ 3 + currencies-seen allocations per fold; ≤ 40 ns/entry | **met** — 4 allocations, constant in `k`; 26.6 ns/entry |
| T-07 | per-key checkpoints | **not done — blocked on LC-02.** See §5 |
| T-08 | `try_run` ≤ 45,000 allocations; `base_at` ≤ 47,000; `advance` ≤ 5 ms | **met** — 46,665 for `base_at` (the target was 47,000), `advance` 3.2 ms; wall clock **noise-limited** |
| T-09 | trigger ≥ 20% of daemon instructions | **fired at 95.8%**, against the order's own expectation that it would not. Implemented; point row +32% |
| T-10 | ≤ 19 allocations per append with empty narratives | **closed as measured-not-worth-doing.** `String::new()` allocates nothing, so an empty narrative already costs zero allocations; the premise was false. The only gain is 8 bytes of inline size on a 144-byte `Entry` whose live cost is 362 B — 2%, against a change touching the wire encoder |

Contract rows, from the published run: `oltp` 1.05× (NOT MET), `analytical` 0.57× (NOT MET),
`point` 1.22× p99 (PARITY), `durable` 1.07× (PARITY).

---

## 4. Guarantees checked

Every commitment in the order's §3 was checked. None was weakened, and this order authorised
none.

| Guarantee | Checked by | Held |
|---|---|---|
| No red tests | `cargo test --workspace`, both repos, `make g3`, adapter | yes — 778/0/5, 444/0/1, 40/0, 29 rows |
| Honest refusal over silent fallback | T-03: `order by` resolves or refuses NL0509, in both surfaces; `scan_fold::plan` refuses rather than approximating | yes — and one violation was **found and repaired** (§7) |
| Theorem 4.1 on Q_lin only | `scan_fold::plan` accepts exactly `sum`/`count`, the fragment `Runtime::install` accepts | yes |
| Denotation preserved (4.6(c)) | `the_fold_and_the_oracle_agree` — 14 shapes through both paths; `sql_golden` 61 cases; E12 sweep byte-identical | yes |
| A balance is a fold, never a field | T-06 changes the fold's allocation, not its shape; T-07 not done | yes |
| Self-describing amounts | untouched; LC-01 not decided, so no interning | yes |
| Products are pure | `gbs-products/tests/purity.rs` | yes |
| Full retention, no compaction | untouched | yes |
| Honest absence | `an_untouched_account_has_no_balance_on_the_view_path_either`, `a_sum_over_no_rows_is_absent_rather_than_zero` | yes |
| **No `unsafe`** | now asserted **repository-wide** in both repos, with one named exception | see MISMATCH-M-01 |
| No `unwrap_or`-to-default | untouched | yes |
| GBS layering | `gbs-products/tests/layering.rs` | yes |
| Benchmarks do not touch committed artifacts | `an_unpublished_run_writes_nothing_a_repository_tracks`; verified by running the harness with a clean tree afterwards | yes, now |
| No performance claim without a reproducible command | every number here is from `make memory`, `bench --run`, or a callgrind command recorded in `docs/BENCHMARK.md` | yes |

---

## 5. MISMATCH and BLOCKED

**MISMATCH-M-01 — "the workspace contains no `unsafe` block" is now stated with one named
exception, and the check is strictly stronger than the claim it replaces.**

`GlobalAlloc` has no safe implementation, so the memory instrument F-08 asked for cannot exist
without `unsafe`. The choice was between an instrument and an unqualified claim. What was done
instead of taking the exception quietly:

* the instrument is a package **outside** each workspace (`tools/memprobe`), excluded from it,
  built by nothing that ships;
* the niles drift test now scans the **whole repository** rather than only `crates/` — a file
  outside `crates/` could previously have used the keyword unpoliced;
* the gbs check, which read **one file, its own**, via `include_str!` — so the claim covered a
  single module of a twenty-thousand-line repository — is replaced by the same repository-wide
  walk;
* both permit exactly one file, **by name, with its reason**, and fail if that permission
  stops being needed.

Thesis §9.10 states this. The claim about the system is unchanged and better tested; what
changed is that the exception is written down where it has to be argued for.

**BLOCKED-LC-02 — per-key checkpoints in GBS (T-07).** Not implemented, per the order's
guardrail. Preparatory findings, so the decision can be made on evidence:

* the **purity test does not block it**: `crates/gbs-kernel/src/view.rs` is already on that
  test's `&mut self` allow-list "the journal itself", so a checkpoint map inside `Journal`
  passes as the code stands. The obstacle is the stated commitment, not the gate;
* the value is large and bounded: a fold is now 26.6 ns/entry and constant-allocation, so an
  account with 20,000 entries costs 532 µs. At `C = 64` a checkpointed fold reads ≤ 64 entries
  — **≈ 1.7 µs, about 300×** — and SC7 already specifies the rule, with the end-of-epoch
  hazard documented in `proto-engine`'s implementation;
* the decision is whether derived, recomputable `(epoch, running balance)` state inside
  `Journal` is compatible with "a balance is a fold, never a field". That is a trade between a
  guarantee and a factor of 300, and the order is right that an implementing agent should not
  make it alone.

No other `MISMATCH` or `BLOCKED` was raised.

---

## 6. Regression guards, with the proof each fails

Every guard below was reverted in a temporary edit and observed to fail. The failing output is
quoted from that run.

| Guard | Reverted by | Failure |
|---|---|---|
| `an_unpublished_run_writes_nothing_a_repository_tracks` | `destinations` always publishing | `an unpublished run reached the committed tree` |
| `a_run_starting_from_a_different_base_is_reported_with_both_numbers` | `base_drift` returning `None` | `a drift is reported` |
| `the_common_set_is_the_same_operation_on_both_sides` | pairing `group by acct` with the ordered statement | ``assertion failed: `group_by_acct` pairs an ordered statement with an unordered one`` |
| `58_order_by_aggregate` (golden) | restoring the `collect_field_names` + zip lowering | `expected: 1 20 / 3 30` `got: 1 20 / 2 20` — the wrong two rows |
| `60_order_by_unknown_column` (golden) | same | `must be refused with NL0509, but it compiled` |
| `an_operator_with_no_evaluator_arm_is_refused…` | `unevaluable` returning nothing | `an operator with no arm must be named, not passed` |
| E18 budgets (T-01) | one allocation per base row in `base_at` | `served_group_by_cur — 213,363 against a budget of 190,000` (+2 more) |
| E18 budgets (T-08) | restoring the `Op::Source` copy | `served_group_by_cur — 133,362 against a budget of 95,000` (+2 more) |
| E18 budgets (T-04) | `scan_fold::plan` refusing everything | `served_group_by_cur — 86,697 against a budget of 28` (+2 more) |
| `the_fold_and_the_oracle_agree` | same | `is inside the fragment and must be folded, or this case is comparing the materialising path with itself` |
| `a_single_account_read_is_served_by_the_maintained_view` | disabling the view path | `a warm key must hit: 0 of 51` |
| `a_repeated_statement_is_compiled_once` | bypassing the plan cache | ``twenty-one asks, one compilation — left: (0, 21) right: (20, 1)`` |
| gbs E18 budgets (T-06) | restoring the per-entry currency clone | `balance_fold_k20000 — 20,003.0 allocations per fold against a budget of 4.0` |

---

## 7. The validation table

| Command | Result |
|---|---|
| `cd niles && make gate` | **green** — 778/0/5, fmt, clippy, generated blocks, memory budget |
| `cd gbs && make gate` | **green** — 444/0/1, adapter 40/0, `g3` 29 rows / 0 failures, memory budget |
| `cargo test --manifest-path gbs/crates/gbs-nilestream/Cargo.toml` at niles `cc7341a` / gbs `3d3faa0` | **green**, 40/0 |
| `cd niles && make reproduce && git status --porcelain` | **green**, empty |
| `make memory` (both repos) | **green** |
| `the_fold_and_the_oracle_agree` and the T-05 extension | **green** — 14 shapes, plus 4 the planner must refuse |
| `bench --run` before/after each step | run five times; see §2. Two rows **noise-limited** — see below |
| callgrind on the point path | recorded in §2; drove T-09's decision |
| massif | **not run.** Peak heap is reported by E18's `peak` column from a deterministic counter, which answered the question at lower cost; the massif and dhat commands are recorded in `docs/BENCHMARK.md` |
| GBS fold allocation guard | **green**, proven to fail |

**Noise-limited results, reported as results.** T-08's wall clock: every Nilestream statement
moved between 0.82× and 1.14× while PostgreSQL — untouched — moved 0.80× to 0.97× on the same
host over the same interval. Normalised against that control the change is inside the spread,
so the verdict is *noise-limited* even though the deterministic counters show allocations
halved. Halving `malloc` calls did not change the time, because the cost of an unkeyed
`group by` was the fold and the B-tree descents. That is an argument for removing the
materialisation rather than for making it cheaper, and is why T-04 was worth doing.

T-09's analytical composite likewise: 0.64× → 0.55× is 13%, below the 20% the variance policy
lets wall clock gate on.

---

## 8. Tasks deliberately not done

* **T-07** — blocked on LC-02, an author's decision. §5.
* **T-10** — closed as measured-not-worth-doing, with the measurement that closed it. §3.
* **LC-01** (interning GBS identities) — not attempted; it is the author's trade and T-06 took
  the part that costs nothing.
* **LC-04** (multi-connection harness) — out of scope for this order, and the OLTP arithmetic
  now says why it is the *only* route to that contract.
* `extended.rs`'s `Prepared::schema_epoch` still uses `engine.frontier()`, so a prepared
  statement is invalidated by every **append**. Conservative and useless as a cache, and left
  alone deliberately: that field is the mechanism a migration story hangs on, and re-purposing
  it under an efficiency task would spend it. The new compiled-circuit cache is keyed
  separately.

---

## 9. Three things found while executing that this order did not cover

**1. The benchmark runs PostgreSQL's five runs before Nilestream's five, so host drift biases
the ratio.** Every `bench --run` measures one target completely, then the other. On a host that
moves by 20% over a few minutes — which this one does — a drift during the run appears as an
engine difference. It showed up as an anti-correlated pair in T-09's run: PostgreSQL's
analytical figure was the highest of the session and Nilestream's the lowest, in the same run,
from a change that cannot slow an analytical query. Interleaving the targets per run would
remove it, and would cost nothing. The per-statement table and the MAD column make it
*visible*; they do not remove it.

**2. The extended protocol's "plan cache" caches no plan.** `Prepared` holds the statement
text, the schema epoch, the output fields and two counters — and no circuit. `Execute`
therefore recompiles, which means the extended path pays the 177,064 instructions the simple
path used to. The cache's own documentation is about validity, which it does correctly; that
it stores nothing to be validated is not visible from the type. The new `compile_cached` is
used by both paths, so the effect is fixed, but the naming still invites the misreading.

**3. `Value` is 32 bytes for a 16-byte payload, and after T-04 that costs almost nothing.**
`niles-ir`'s `Value` is `{Null, Int(i128)}` — 32 bytes, doubling every row it appears in. F-13
said not to rewrite it, and the measurements say more than that: with the served path folding
posting records directly, the representation now costs only the reference evaluator and the
corpora. The audit's estimate of what a packed row would save is real and it is no longer on
any path a user's query takes. Revisit only if gate time on the corpora becomes the
bottleneck — it is currently 0.13 s.

---

## 10. What the next cycle should look at first

Ranked by what this cycle's measurements point at, not by what is left over.

1. **`group_by_acct` and `top_ten_by_sum` are row-production bound, not fold bound.** They are
   0.45× and 0.54× of PostgreSQL and cost 95,035 allocations for 10,001 groups — about nine
   per group, for the key, the accumulator, and the `Vec<Option<String>>` the wire wants. The
   wire representation is the target, not the engine.
2. **The daemon is single-threaded in its work** (`Arc<Mutex<RevEngine>>`). Nothing in this
   cycle could demonstrate the cost, because the harness drives one connection on two cores.
   LC-04's measurement is the prerequisite for knowing whether it matters.
3. **LC-02**, which is worth ~300× on long-history folds and is one sentence of decision.
4. **The two contracts whose shape puts them out of reach** (LC-03). Both NOT METs now end in
   arithmetic; the specification is where they end.
