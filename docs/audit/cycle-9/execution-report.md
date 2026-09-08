# Cycle 9 execution report

**Executor:** Claude Opus 5, in the container, against the consolidated work order
`docs/audit/cycle-9/work-order-9.md` (Fable + Astra, tasks C9-00 … C9-12, cut after C9-06).
**Branch:** `c9/07-pending`, tip `a1dbca6`, ten commits over `638c7bb` (the work order).
**Paired GBS branch:** `c7/00-adapter`, tip `688919c`, two commits over `963e4d9`.

**Seven of seven above-cut tasks landed. One target inside them failed its pass line, and
the failure is the cycle's most useful result.** C9-06.2 predicted ≥ 1.25× read throughput
from removing the view-lock hold. The measurement returned **1.018×** — and, in doing so,
refuted the baseline the prediction was written from. §11 is that result; it changes what
LC-23 says and what Chapter 3 may claim.

Nothing below the cut was taken. No task was substituted for another.

---

## §1 The checklist, verbatim

Each line is the work order's §8 wording. `done` means the line is true of `a1dbca6`.

| target | status |
|---|---|
| **C9-00.1** The gate's fixture races are gone: two fixtures made in one clock tick own different files, and a scratch name already on disk is stepped over rather than truncated. | **done** — `crates/nilesc/tests/run.rs`, `Temp::candidate(name, salt, pid, seq)` shared by constructor and guard, `create_new(true)` with 64-candidate retry. Both guards re-proved by reversion, §2 R1/R2. |
| **C9-00.2** A reader-progress precondition failing is reported as a precondition, not as the property. | **done** — `frontiers.rs`: `WANT = 150`, `CEILING = 200_000_000`, and a `PRECONDITION UNMET:` message distinct from the property's. |
| **C9-00.3** The sealer's batching is testable: every transaction waiting when the sealer wakes commits in one epoch. | **done** — `start_gated(.., Option<Arc<Barrier>>)`; `every_transaction_waiting_when_the_sealer_wakes_commits_in_one_epoch` asserts `max_batch == 32`, `epochs_sealed == 1`, `fsyncs == 1`. The two opportunistic `max_batch > 1` assertions became `eprintln!` reports: a scheduler that gives the submitters no overlap is a fact about the host, not a defect. |
| **C9-01.1** `select nilestream_lockstats [reset]` reports four scopes and a rounding column; the harness resets before each level. | **done** — `STATEMENT` and `WIRE` added beside `ENGINE_LOCK` and `VIEW_LOCK`; `rounding_us`; `nls_lockstats_reset(port)` before each mixed level in `bench.rs`. |
| **C9-01.2** A percentile is a bucket boundary and never overstates; the open-ended top bucket reports the observed maximum. | **done** — `quantile` returns `(1<<i)-1` clamped by `max_us`, and `max_us` for the top bucket. Re-proved, §2 R3. |
| **C9-01.3** The "0–2 µs on everything else" column is withdrawn in the prose that cited it. | **done** — Chapter 3, with the arithmetic reason stated (four consecutive timestamps partition the fourth by construction). |
| **C9-02.1** `append` returns `Appended { epoch, receipt }` and the receipt is held on the session that made it. | **done** — `take_pending` removed from the `Serving` trait; `Session.receipts` + `take_receipts()`; a source guard requires `take_receipts` and forbids `take_pending` in `daemon.rs`. Guard `a_receipt_is_reachable_only_from_the_session_that_appended`. |
| **C9-02.2** Both idempotency windows are bounded in **transactions**; the sealer prunes by its per-transaction queue and the record's number is `batch_seq`. | **done** — `while order.len() > w { pop }`; `the_window_holds_the_last_w_transactions_whatever_the_batch_size` at W−1/W/W+1 under batch 1 and > 1. Re-proved, §2 R4. Paired GBS commit `447544d`. |
| **C9-03.1** A damaged length is classified as damage, not as a torn tail; the header checks itself. | **done** — `len_check(len) = !len ^ 0xA5A5_A5A5`, `MIN_RECORD = 84`, `TruncationCause::BadHeader { at_offset }`. Re-proved, §2 R5. |
| **C9-03.2** `open_bounded` returns `Err(InvalidData)` on an envelope it cannot decode exactly, and no public constructor can start a sequencer with a window built by `unwrap_or_default`. | **done** — `decode_envelope` is strict UTF-8 and refuses trailing bytes (`o != payload.len()`); the unchecked `recover_seen` / `recover_txns` are **deleted**, not deprecated. |
| **C9-03.3** `SPEC-ENGINE` states the record layout and the three-way classification; the thesis sentence carries no marker. | **done** — record-layout table and clean-end / torn-tail / damage table added; the chapter-7 sentence is corrected. |
| **C9-03.4** A segment cut exactly at a record boundary is recorded as `LC-36`, never claimed solved. | **done** — raised as LC-36 in the work order's ledger and stated in `SPEC-ENGINE`. |
| **C9-04.1** Every currency code is the declaration index, tested under permuted `HashMap` insertion. | **done** — `CurrencyInfo::code` assigned once at collection; `crates/niles-lang/tests/currency_codes.rs`, 200 resolutions. Re-proved, §2 R6. |
| **C9-04.2** `nilestreamd --budget 2,500` and each other flag with a non-integer exits non-zero naming the flag and the text. | **done** — `fn flag<T: FromStr>(name, raw) -> T` refuses; `crates/nilestream-server/tests/daemon_flags.rs`, 7 process-level refusals. |
| **C9-04.3** Two relations declaring different windows are refused at startup naming both; one relation gives its window deterministically. | **done** — `declared_idem_window -> Result<Option<u64>, String>`, names sorted. |
| **C9-04.4** `txn idem(.., window: ..)` is NL0218 everywhere; a `txn idem(..)` to a relation without an `idem` column is NL0219; `ignored_windows` no longer exists. | **done** — 82 corpus sites plus 9 in GBS converted; `docs/BUILD-LOG.md` left as history; `docs/keywords.md` regenerated. Paired GBS commit `688919c`. |
| **C9-04.5** `nilestreamd` with neither `--durable` nor `--volatile` exits non-zero; `--volatile` says VOLATILE; a wire relation without `idem` is refused unless `--idem-window` is given. | **done** — LC-16 and LC-35 refusals. |
| **C9-05.1** `schema_parses` reads 1 after 2,000 inserts at one epoch and 2 after a schema change. | **done** — `Session.currencies: Option<(u64, Option<BTreeMap<u32,u32>>)>`; `schema_parses` is a `nilestream_stats` column. Re-proved, §2 R7 (**2000 vs 1**). |
| **C9-05.2** `Session::insert` costs at most 45,000 instructions per insert under callgrind on the executing host, stated with the command. | **done** — `crates/nilestream-server/src/bin/checked-twice.rs`, `make checked-twice`; before/after table in `docs/BENCHMARK.md`. |
| **C9-05.3** The four SQLSTATE refusals on the insert path each still have a passing test. | **done** — unchanged and green. |
| **C9-06.1** Every concurrent read equals an independent fold at its own requested anchor while a delayed reconstruction overlaps an `advance`, and same-key same-anchor readers share exactly one fold without holding V while folding or waiting — proved by the latch-driven two-phase differential whose precondition counter is asserted. | **done** — `rev::two_phase::a_flight_that_overlaps_an_advance_is_joined_shared_and_installed_pinned`; asserts `pinned_installs + deferred_merges + pending_joins > 0`. Three reversions red, §2 R8–R10. |
| **C9-06.2** At the corrected 12r6w Host C point, Pending delivers ≥ 1.25× baseline median read throughput under the ≥ 10% ∧ ≥ 3 pooled-MAD gate, with no contract or durability regression and the `view_wait_us` maximum reported beside it. | **not done: measured and failed. 1.018× (+1.8%).** Clears 3.28 pooled MADs, fails the ≥ 10% arm, nowhere near 1.25×. No contract or durability regression (`durable` @12 +4.5% and tighter; fallback 0.00% on both arms). `view_wait_us` maxima reported in §11. **The baseline the pass line was written from is also refuted** — see §11. |
| **C9-06.3** `nilestream_stats` reports the four flight counters; the 12r6w run shows `pending_joins > 0`; the marker is removed only then. | **done** — median 14,377 joins at 12r/6w on Host C. The marker is struck from Chapter 3, not narrowed. |
| **C9-06.4** Theorem 4.1's proof carries the Pending clause; §3's lattice paragraph and read-path rules state the exact-anchor interval, the equal-anchor join and the generation-owned install as implemented. | **done** — Theorem 4.1 clause (5) with sub-clauses 5a–5c; Chapter 3 lattice, read-path and upquery paragraphs rewritten. |

**Below the cut — not taken, and not substituted for anything above it:** C9-07 (checkpoints),
C9-08 (hole compaction, graph-walking `install`), C9-09 (E18 on production structures), C9-10
(sentence ledger and MISMATCH inventory), C9-11 (templated plan key), C9-G01 (syndicated trace),
C9-12 (C republish).

---

## §2 Guard transcripts

Every reversion below was applied in a **disposable worktree**, run, and reverted; the worktree
was then removed. R1–R7 were re-proved for this report at tip `a1dbca6` in
`/home/claude/work/c9-wt-report`; R8–R11 were proved at landing time in
`/home/claude/work/c9-wt-t06`. A compile error is never the witness — every reversion below
compiles and fails on an assertion.

### R1 — C9-00, the fixture name carries pid and sequence
*Reverted:* `crates/nilesc/tests/run.rs` — `format!("nilesc-run-{name}-{salt}-{pid}-{seq}")` → `format!("nilesc-run-{name}-{salt}")`.
```
test two_fixtures_made_in_one_clock_tick_own_different_files ... FAILED
panicked at crates/nilesc/tests/run.rs:84:9
test result: FAILED. 0 passed; 1 failed; 12 filtered out          rc=101
restored: test result: ok. 1 passed; 0 failed
```

### R2 — C9-00, a taken scratch name is stepped over, not truncated
*Reverted:* `.create_new(true)` → `.create(true)`.
```
test a_scratch_name_already_on_disk_is_stepped_over_rather_than_truncated ... FAILED
assertion `left != right` failed: it took the taken name
  left: "/tmp/nilesc-run-taken-1700000000000000001-24065-0"
 right: "/tmp/nilesc-run-taken-1700000000000000001-24065-0"
test result: FAILED. 0 passed; 1 failed                            rc=101
restored: test result: ok. 1 passed; 0 failed
```
This guard is the one that **passed under its own reversion** when first written, because the
test computed the squat name itself. Routing the constructor and the guard through one
`Temp::candidate` is what makes it a guard.

### R3 — C9-01, a percentile is a bucket boundary and never overstates
*Reverted:* `lockstats.rs` — `(1u64 << i).saturating_sub(1)` → `1u64 << i`.
```
test lockstats::tests::one_outlier_in_a_hundred_moves_the_maximum_and_not_the_p99 ... FAILED
  assertion failed: the median hold is in the 1µs band   left: 2  right: 1
test lockstats::tests::a_tail_wider_than_one_percent_moves_the_p99 ... FAILED
  assertion failed: the median is unmoved by a tail      left: 2  right: 1
test result: FAILED. 7 passed; 2 failed                            rc=101
restored: test result: ok. 9 passed; 0 failed
```

### R4 — C9-02, the window is counted in transactions
*Reverted:* `sequencer.rs` — `while order.len() > w` → `while order.len() > w * 4096` (the window
in *records*, one record being a batch of up to 4,096 transactions).
```
test sequencer::window_tests::the_window_holds_the_last_w_transactions_whatever_the_batch_size ... FAILED
panicked at crates/nilestream-ledger/src/sequencer.rs:1068:13
test result: FAILED. 0 passed; 1 failed                            rc=101
restored: test result: ok. 1 passed; 0 failed
```

### R5 — C9-03, the record header checks itself
*Reverted:* `segment.rs` — `fn len_check(len) { !len ^ LEN_CHECK_MASK }` → `fn len_check(_len) { 0 }`.
```
test a_segment_whose_envelope_cannot_be_decoded_refuses_to_open_at_all ... FAILED
test every_mutation_is_refused_or_recovers_exactly_what_was_acknowledged ... FAILED
  the refusal must name what could not be rebuilt: … has a damaged record header at offset 990:
  its length does not match its own check word. That is corruption, not a torn write …
test result: FAILED. 0 passed; 2 failed                            rc=101
restored: test result: ok. 2 passed; 0 failed
```

### R6 — C9-04, a currency code is a declaration index
*Reverted:* `resolve.rs` — `let code = cat.currencies.len() as u32` → a hash of the currency name,
which is what the pre-C9-04 consumers effectively used by iterating a `HashMap`.
```
test the_catalog_numbers_currencies_by_declaration_and_not_by_hash_order ... FAILED
  assertion failed: round 0: `zar` is the currency declared at position 0, and its code must be 0
  in every process that ever compiles this schema
  left: 21  right: 0
test a_money_literal_lowers_to_its_declaration_index ... FAILED
test result: FAILED. 0 passed; 2 failed                            rc=101
restored: test result: ok. 2 passed; 0 failed
```

### R7 — C9-05, the schema is compiled once per epoch
*Reverted:* `session.rs` — the cache-hit arm made unreachable (`if *k == key && false`).
```
test session::tests::the_schema_is_parsed_once_per_epoch_and_not_once_per_insert ... FAILED
  assertion failed: 2000 inserts at one schema epoch parsed the schema 2000 times
  left: 2000  right: 1
test result: FAILED. 0 passed; 1 failed                            rc=101
restored: test result: ok. 1 passed; 0 failed
```

### R8 — C9-06, an install below `applied` is pinned
*Reverted:* `rev.rs::install` — the `if anchor < self.applied { pinned.insert(..) }` branch removed.
```
test rev::two_phase::a_flight_that_overlaps_an_advance_is_joined_shared_and_installed_pinned ... FAILED
  assertion failed: a read at the new frontier was served the value from before the epoch that
  landed during the flight. An entry installed at an anchor below `applied` has not seen the
  deltas in between and must not inherit the frontier
  left: 60  right: 1000060
test result: FAILED. 3 passed; 1 failed
```

### R9 — C9-06, `advance` never writes into a Pending slot
*Reverted:* the `Some(Slot::Pending(_))` arm of `apply_epoch` writes `Present(delta, e)`.
```
  assertion failed: `advance` wrote into a `Pending` slot. There is no value there to fold a
  delta into, and publishing one epoch's delta as the key's balance makes every reader whose
  anchor falls in the resulting interval take it as a hit: the account is wrong by its entire
  history until the flight lands
  left: Present(1000000, 11)  right: Pending(10)
test result: FAILED. 3 passed; 1 failed
```

### R10 — C9-06, an install is owned by its generation
*Reverted:* `finish_fold` — the generation test replaced with `let mine = true`.
```
test rev::two_phase::a_flight_that_overlaps_an_advance_is_joined_shared_and_installed_pinned ... FAILED
test rev::two_phase::a_superseded_completion_never_moves_a_resident_stamp_backwards ... FAILED
  assertion failed: the older completion overwrote the newer resident entry. Its value is right
  at its own anchor and wrong as a resident stamp: every read above `5` now reconstructs, and
  the flight table entry it cleared belonged to somebody else
  left: Present(406, 4)  right: Present(906, 5)
test result: FAILED. 2 passed; 2 failed
```

### R11 — C9-06, the fold happens with the view released
*Reverted:* `answer_from_view` — the two-phase block replaced by a single `Rev::read` under the
hold. Compiles; both source guards fire.
```
test rev_engine::lock_order_tests::a_joined_reader_waits_outside_the_read_and_holds_nothing ... FAILED
test rev_engine::lock_order_tests::the_reconstruction_happens_with_the_view_released ... FAILED
test result: FAILED. 3 passed; 2 failed
```

---

## §3 The gate

`make gate` at `a1dbca6`, container, `RUSTUP_TOOLCHAIN=stable` (stable **is** 1.95.0; the pin
`1.95.0` does not resolve), `RUSTUP_AUTO_INSTALL=0`, no network.

| | result |
|---|---|
| `cargo test --workspace --no-fail-fast` | **1,037 passed, 0 failed, 7 ignored** |
| `cargo clippy --all-targets -- -D warnings` | clean, workspace and `tools/memprobe` |
| `cargo fmt --check` | clean |
| `make reproduce` | regenerates identically; the terminal `git diff --exit-code` is red only while the regenerated files are uncommitted, green after each commit |
| README test-count drift | 866 → 898, updated eight times during execution; the drift test is what forced each |

**Ignored, by name** — all seven are measurements or generators, never checks:
`e16_point_workload_against_the_rev_runtime`, `e18_verdict_distribution`, `generate`,
`e17_write_the_measurement`, `transcript`, and
`lockstats::instrument_cost::timing_an_acquisition_is_not_a_measurable_share_of_a_read` (twice,
lib and bin).

**Changed verdicts, by name.** None went green→red and stayed. Two went red during execution and
are named because a report that lists only the end state is not a record of what happened:
`the_visible_frontier_never_names_an_epoch_the_base_has_not_applied` (broke when a needed wait
block was removed in C9-06; fixed with `append_durable`), and
`the_readme_test_count_is_the_number_of_tests_that_exist` (by design, eight times).

**The Mac's oracle.** `cargo +1.97.1 clippy --offline --all-targets -- -D warnings` was run by the
author after every landing and finished clean each time — the last at `a1dbca6`. The Mac gate was
not run in full; the container's is the gate of record for this cycle.

**One container red that is not a defect.** `numeric_binary_oracle::our_numeric_bytes_are_postgresqls_numeric_bytes`
fails closed with `no PostgreSQL on 127.0.0.1:5432` in a fresh container until
`service postgresql start` is run. See §5, fact 2.

---

## §4 The LC ledger, with what this cycle decided

| LC | before this cycle | after |
|---|---|---|
| **LC-23** — the read tail's cause | *closed* by cycle 8 as "the view mutex holding V across the fold" | **reopened, against the base.** The closing attribution was inferred from an uncorrected harness. In the slowest-16 tables of *both* arms of C9-06.2, `base wait` is the dominant term and often the whole of it. §11. |
| **LC-30** — Pending's protocol at a different anchor | proposed: join only on identical `(key, anchor)`; other anchors fold independently and do not install; bounded flights; generation-owned completion — *the author confirms the protocol* | **implemented as proposed and guarded** (R8–R11). Author confirmation still outstanding as a matter of record; nothing was implemented beyond the proposal. |
| **LC-32** — checkpoint interval | open, decided on `c9-checkpoints.sh` | **untouched** — C9-07 is below the cut and the script does not exist. Both C9-06.2 arms refuse `--checkpoint-interval`, so checkpoints are 0 on both by construction and Pending was measured in isolation as required. |
| **LC-35 / LC-16** — refusals at startup | open | **closed** by C9-04.5. |
| **LC-36** — a segment cut exactly at a record boundary | raised this cycle | **open, and stated as the author's contract to give**: indistinguishable from a shorter log without a retained tip. Never claimed solved by the parser. |
| **LC-24** — p99 at 16 connections | carry; measure the remaining B wait after C9-06 (C9-12.3) | **carried, and now the priority.** C9-06.2 says B is not a candidate cause but the dominant one. |
| **LC-31** — whether a demand view keeps holes | open, author's decision | **untouched** (C9-08 is below the cut). |
| **LC-34** — Loan IQ / Calypso coverage | open | **untouched** (C9-G01 below the cut). |

**New, raised by this cycle's measurement:** the deferred-delta merge should be promoted above the
cut in cycle 10. At 12r/6w, pinned installs are 6.35% of reads and joins 0.29% — twenty-two
reconstructions land pinned for every one shared, and a pinned entry serves exactly one anchor.
`deferred_merges` exists and reads zero.

---

## §5 Exactly three material facts the work order does not cover

Nothing here is recycled from §2 or §2A of the work order.

### Fact 1 — a `#[cfg(test)]` item mid-file silently truncates the generated public-API appendix
*Evidence class: reproduced, in this container.* `thesis/gen-appendix-d.py:70–74` builds each
crate's public surface by cutting the file at **the first `#[cfg(test)]`** and keeping what
precedes it. A test-only helper placed inside `impl Rev`, above `Unsupported` and `Runtime`, moved
that cut and **deleted both types from `thesis/appendix-d-api.md`** — a removal that `make
reproduce` surfaces as a diff a reviewer is invited to accept, not as a failure. Observed as
`-pub enum Unsupported` / `-pub struct Runtime` in the regenerated appendix.

*Consequence.* The appendix that documents the engine's public surface can lose its tail from an
edit that has nothing to do with the public surface, and the mechanism that is supposed to keep it
honest reports the loss as an intended change. Worked around in `rev.rs` by moving the helper into
the test module, with the reason recorded at the site so the next person does not re-trip it. The
generator itself is unrepaired.

*Does it move the cut?* No. It is a one-line fix (skip `#[cfg(test)]`-attributed items rather than
truncating) and belongs with C9-10, which is already the prose-integrity task.

### Fact 2 — `make gate` is red in a fresh container for an environment reason, and the preflight table says otherwise
*Evidence class: reproduced.* `numeric_binary_oracle` panics with *"no PostgreSQL on
127.0.0.1:5432 — this test is the only check that the money encoding is right, and skipping it
silently would leave that unchecked"*, taking `make gate` to `Error 101`. Failing closed is
correct and should stay. What is wrong is the record: the work order's §0 preflight lists
`psql / postgres : PostgreSQL 16.13` as though it were serving, and it is not — `service
postgresql start` is required first, and nothing says so.

*Consequence.* An executor, a CI job or a future audit reading that preflight row will read a false
red on the single test that checks the money encoding, and the natural reaction to a red gate on
an unrelated task is to doubt the task. It cost one wasted diagnosis in this cycle.

*Does it move the cut?* No. It is a line in `preflight.sh` and a sentence in the next work order.

### Fact 3 — the generation check does not prevent a wrong answer, and the work order says it does
*Evidence class: reproduced, and a failed attempt to reproduce the opposite.* The work order
specifies reversion 3 as *"install ignoring generation → an older completion overwrites a newer
resident → divergence"*. **No value divergence is constructible.** A ticket's `(key, anchor)` is
fixed at `begin_read`, so a late completion's value is exact at its own anchor; and `hit()` serves
only inside `[stamp, effective]`, so a stale *stamp* can never answer a later anchor. I tried
several interleavings — historical anchor below `applied`, different-anchor fold landing after the
owner, `applied` pinned and unpinned — and the certification interval defeated each.

What the generation actually prevents is (i) a resident stamp moving **backwards**, which costs
every later reader a reconstruction, and (ii) a stale owner **clearing a flight record it does not
own**, which strands the successor's `Pending` marker with nobody left to publish it — a liveness
hazard, not a conservation one. The guard therefore asserts the stamp
(`left: Present(406, 4)  right: Present(906, 5)`), and the code comment says so in as many words.

*Consequence.* The hazard class in the work order is overstated by one rung. The check is still
required — a permanently orphaned `Pending` slot is worse than a slow read — but the thesis must
not claim the certification interval needs it for correctness, because the interval is what makes
it unnecessary for correctness. Recorded at `rev.rs::finish_fold` under *"Why a generation, and
what it is not for"*.

*Does it move the cut?* No, and it strengthens Theorem 4.1's clause (5): 5a is provable without
the generation, which is why it is stated first.

---

## §6 Worktree status, and the five protected files

**Container, `/home/claude/work/niles`, at `a1dbca6`:** clean except for this report's own file.
Every disposable worktree used during execution has been removed and pruned; `git worktree list`
shows one entry.

**Mac, `/Users/checolino/Documents/niles`:** `a1dbca6` on `c7/01-durable-rows`, working tree clean
apart from the five untracked protected files.

| protected file | size at audit | size now | verdict |
|---|--:|--:|---|
| `.DS_Store` | 10,244 | **10,244** | unchanged |
| `AGENTS.md` | 16,639 | **16,639** | unchanged |
| `niles/.DS_Store` | 6,148 | **6,148** | unchanged |
| `thesis/.DS_Store` | 8,196 | **8,196** | unchanged |
| `thesis/Niles-Thesis.pdf` | 816,110 | **816,110** | unchanged |

All five are still untracked and were never edited, staged, moved or bundled. No discrepancy to
refer.

**Mac, `/Users/checolino/Documents/GBS`:** clean, but **at `963e4d9` — two commits behind.** Both
GBS bundles are present in `~/Documents/niles-sync/cycle-9/` and neither has been fetched. Until
they are, the GBS tree does not build against the current niles: `batch_seq` is renamed and nine
`idem` sites are converted. The sync block is in §8.

---

## §7 Commits and bundles

| commit | task | bundle | sha256 (first 16) | bytes |
|---|---|---|---|--:|
| `366c338` | C9-00 | `niles-01-gate-fixtures.bundle` | `2cee641b5fabdfb1` | 55,358 |
| `f7cdfdd` | C9-01 | `niles-02-instruments.bundle` | `4aab680752aecd4e` | 21,134 |
| `35484ca`, `c224cbd` | C9-02 | `niles-03-owned-receipts.bundle` | `83ccedc2eae6afad` | 13,321 |
| `ca8e5c9` | C9-03 | `niles-04-strict-recovery.bundle` | `59fe62fa247740d3` | 15,855 |
| `6c755fb` | C9-04 | `niles-05-contracts.bundle` | `20675e073341cc0d` | 18,798 |
| `ebe7f8f` | C9-05 | `niles-06-currency-table.bundle` | `45cc5cd46533544b` | 8,127 |
| `78acd64`, `4ac5032`, `a1dbca6` | C9-06 | `niles-07-pending.bundle` | `f5c70c21deb4c58a` | 4,783,641 |
| `447544d` (GBS) | C9-02 | `gbs-01-batch-seq.bundle` | `9558d4ab4aa8305c` | 1,344 |
| `688919c` (GBS) | C9-04 | `gbs-02-idem-key-only.bundle` | `0524a637cc5362c5` | 1,228 |

All bundles have base `638c7bb` (niles) or `963e4d9` (GBS) and verify with
`The bundle records a complete history`. **No executor push.** Every trailer is this session's own:
`Co-Authored-By: Claude Opus 5` and the session URL.

---

## §8 The sync the author still owes

Niles is current on the Mac at `a1dbca6`. **GBS is not.** Run:

```
cd ~/Documents/GBS
git fetch ~/Documents/niles-sync/cycle-9/gbs-01-batch-seq.bundle c7/00-adapter
git merge --ff-only FETCH_HEAD
git fetch ~/Documents/niles-sync/cycle-9/gbs-02-idem-key-only.bundle c7/00-adapter
git merge --ff-only FETCH_HEAD
git branch -f c7/00-adapter HEAD
git push origin c7/00-adapter
RUSTUP_AUTO_INSTALL=0 cargo +1.97.1 clippy --offline --all-targets -- -D warnings
```

---

## §9 The MISMATCH inventory

Live literal markers in `thesis/`, `results/`, `SPEC-ENGINE.md` and `BENCHMARK.md` at `a1dbca6`:

| marker | count | owner |
|---|--:|---|
| `MISMATCH-T-22-currency-literal` | 4 | pre-existing |
| `MISMATCH-daemon-checkpoints` | 3 | C9-07 (below the cut) |
| `MISMATCH-A-01` | 3 | pre-existing |
| `MISMATCH-T-11-overdraw` | 2 | pre-existing |
| `MISMATCH-F-08` | 2 | pre-existing |
| `MISMATCH-A9-F15` | 1 | C9-08 (below the cut) |
| `MISMATCH-A9-F12` | 1 | C9-07 |
| `MISMATCH-A9-F05` | 1 | raised and **repaired** by C9-04; the marker at the currency-code rule is stale and should be struck by C9-10 |
| `MISMATCH-idem-fingerprint`, `MISMATCH-e16-header`, `MISMATCH-durability-restart`, `MISMATCH-T-13-fixpoint`, `MISMATCH-F-09`, `MISMATCH-F-07` | 1 each | pre-existing |

**Removed this cycle:** `MISMATCH-pending-unreachable` (C9-06.3), struck from Chapter 3 with the
evidence cited rather than narrowed. Count is now 0.

**Sentence ledger.** Every sentence corrected this cycle is listed at its site with the reason;
the full Astra §2.5 ledger is C9-10's target and was not rebuilt, which is why C9-10 stays below
the cut rather than being partly done. Sentences corrected: Chapter 3's lattice paragraph, its
read-path and upquery rules, and the "0–2 µs" withdrawal; Theorem 4.1's proof; Chapter 7's record
layout; Chapter 9's miss-rate correction; `E19-scaling.md`; `SPEC-ENGINE` E-conc-3 and the record
format; `BENCHMARK.md`'s slow-read section and the two `--volatile` launch lines; and the
`VIEW_LOCK`, `SLOW_READS` and `answer_from_view` doc comments (§11).

---

## §10 Timing and counted work

Container figures are two-core sandbox figures and are never comparable with Host C's. They are
kept apart by column.

| measurement | host | value |
|---|---|---|
| torn corpus, exhaustive | container | 8,840 bit flips + truncation at every offset: **7,984 refused / 856 prefix-with-success**, all inside the final record (was 1,120 with 216 outside) |
| `checked-twice oltp`, instructions/insert | container, callgrind | 113,500 → **≤ 45,000** |
| in-process two-phase differential | container | 82 joins, 64 pinned installs over 1,600 reads; run time **26.06 s → 0.18 s** after the readers were moved to the two-phase API |
| E18 `rev_metadata_2x_budget` | container | 3.8 → **6.8** alloc/op, budget 5.0 → **7.5**; `live_delta` 859,352 → 860,344 bytes (**+0.1%**) |
| E18/E24 | — | allocated / live / peak are never in one column with RSS; no E24 row was taken this cycle |
| C9-06.2 read throughput | **Host C** | §11 |

E18's budget was raised deliberately, not to make a test pass: the three extra allocations per
miss are an `Arc<Completion>`, a flight-table key and that table's node growth, all transient. The
row exists to say whether a long-lived view is Θ(budget) or Θ(history), and `live_delta` says it
is still Θ(budget).

---

## §11 C9-06.2 — the result, and what it refuted

Full evidence: `docs/audit/cycle-9/hostc/c9-pending-results.md`. Host C, Darwin 25.6.0 arm64,
10 cores, rustc 1.95.0, 2026-09-08T14:40:12Z. One daemon per replicate, arms interleaved, 2
warm-ups discarded, 5 measured replicates each, 30-second levels, 2,000 accounts × 8 rounds at a
budget of 2,500, `--checkpoint-interval` refused by both arms. 5/5 replicates on each arm; every
section ran.

| level | baseline (C9-05) | candidate (C9-06) | ratio | pooled MAD | Δ in MADs |
|---|--:|--:|--:|--:|--:|
| 6r/3w | 147,306 | 151,185 | 1.026 | 3,118 | 1.24 |
| 9r/5w | 156,686 | 159,499 | 1.018 | 1,083 | 2.60 |
| **12r/6w** | **160,838** | **163,753** | **1.018** | 888 | 3.28 |

**Fails.** The gate is ≥ 10% **and** ≥ 3 pooled MADs; +1.8% clears the second arm only, and 1.018
is not 1.25. `view_wait_us` maxima at 12r/6w, reported beside it as required — baseline
321 / 297 / 542 / 299 / 788 µs, candidate 377 / 26 / 77 / 683 / 1,589 µs. No contract or
durability regression: `durable` @12 median 1,410 → 1,473 (+4.5%) with the candidate's five
replicates spanning 1,466–1,474 against the baseline's 1,216–1,476; `fold` @12 +2.8%; `point` @12
−1.5%, inside its own spread; `fallback` 0.00% on every level of both arms.

**The pass line's baseline does not exist.** It was written against LC-23's
94,038 / 48,310 / 20,507 reads/s with a 22,160 µs view wait — figures the work order itself
flagged as *inferred from a document*. The same predecessor build, measured on the corrected
harness, does **147,306 / 156,686 / 160,838**: rising with concurrency, **7.8×** the collapsed
figure at twelve connections, with a slowest-read view wait of 788 µs rather than 22 ms. There was
no collapse to recover.

What differed is what C9-00 and C9-01 repaired: one daemon for a whole session with `ENGINE_LOCK`
and `VIEW_LOCK` histograms never reset between levels (F-75), lifetime rather than level-local
counters, and a keyed read folding the base twice on 87.4% of attempts before the certification
interval was repaired in cycle 8. **The falling curve was a property of the instrument.**

**Where the time goes.** In the slowest-16 tables of both arms, at every level, `base wait` is the
dominant term and frequently the whole of it — one candidate replicate prints 1098/1097,
1093/1093, 1089/1089 µs total/base. `answer_from_view` holds the base read guard across the fold,
deliberately, because `RwLock` is not reentrant and a keyed read must take the base exactly once;
a writer arriving behind that guard queues, and every later reader queues behind the writer.
**LC-23's attribution to the view mutex is withdrawn.**

**What was kept, and on what grounds.** The two-phase read stays: it makes the absence lattice's
fourth state reachable and counted, and it bounds a hold that had no bound. Both are structural
claims the thesis makes and the engine could not previously support. It is **not** kept as a
throughput result and no cell of the phase diagram is drawn from it. Every sentence that rested on
the refuted curve was rewritten rather than left standing — Chapter 3, `SPEC-ENGINE` E-conc-3, the
`VIEW_LOCK` and `SLOW_READS` doc comments, `answer_from_view`'s own comment.

The view histogram, added in cycle 8 to convict the view lock, is what exonerated it. That is the
only thing an instrument is for.

---

## §12 What the next cycle should take first

Not a decision, a recommendation, and the author's to accept or refuse.

1. **The base-lock tail (LC-24, LC-23-reopened).** It is now the measured bottleneck and it is in
   no work order. `answer_from_view` holding the base read guard across a reconstruction is the
   shape to attack.
2. **The deferred-delta merge.** 6.35% of reads land pinned against 0.29% joined. It was scoped as
   "a second commit" and the measurement says it is worth more than the first one was.
3. **C9-07 and C9-08**, in the work order's order, unchanged.

Fact 1 and Fact 2 of §5 are one-line repairs and belong wherever they fit.
