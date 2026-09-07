# Cycle 8 execution report

*Executor: Claude Opus 5, session `session_012KJD5FHvwxtenT6HxbXskv`, 2026-09-07. Work order:
`docs/audit/cycle-8/work-order-8.md` (Fable, commit `c2617f6`). Author: Sergio, Host C.*

Eight of the work order's eight above-the-line tasks landed; one target inside T-15 is `not
done` with its reason. Trees at the end: `niles` `7f9f30b` on `c8/07-ab-script`, `gbs`
`963e4d9` on `c8/01-idem-window-epochs`, both clean, 991 and 504 test assertions passing.

Three defects were found by *running* the work order that the work order did not name, and
two of them are the kind it warned about — correct on every input, wrong in structure. They
are in §5.

---

## §1 The checklist, verbatim

```
[x] T-05.1 — after committing 2× budget distinct keys, reads_of.len() ≤ budget and last_read.len() ≤ budget and the slot map holds ≤ budget non-⊥ entries: metadata is Θ(budget).
[x] T-05.2 — metadata bytes per resident key ≤ 120 (from ~340), by packing the (acct, cur) key and evicting the two maps with the entry.
      done: 159.0 → 94.1 B, by merging the two maps into one rather than packing the key.
      The measured baseline was 159 B/resident key, not the ~340 the work order carried —
      that figure was per key *ever read*, which is the number the bound is about and which
      went 284 → 172 B.
[x] T-05.3 — idem: IdemKey window 30.days reaches the sequencer: a key whose epoch is older than the window is admitted as new, and seen (both copies, or one shared copy) holds only keys inside the window; nilestream_stats gains idem_window_keys and view_metadata_keys.
      done, in epochs: LC-28 was put to the author, who chose "count the window in epochs".
      A wall-clock window is now refused (NL0217) rather than silently unenforced.
[x] T-05.4 — the sealer's BTreeMap and the ledger's HashSet are one structure or the report says why two are needed.
      not merged; the reason is in §4.
[x] T-06.1 — preflight.sh reports the pin it finds, the test count it counts, and the expected heads of this cycle; a source test asserts the script contains no literal test count.
      the expected heads are removed rather than updated — see §4.
[x] T-06.2 — ch3's lattice reads Present(v, [e_s, e_f]) (or states in one sentence that a Present entry's certification is an interval and names §4's Theorem 4.1 step (3)); the MISMATCH-lattice-interval marker is placed and resolved in the same commit.
[x] T-06.3 — §9.14.1 says the committed E16 document predates the header and names the commit (5d37692) from which every generated one carries it; MISMATCH-e16-header stays until run6.sh publishes.
[x] T-06.4 — SPEC-ENGINE.md:705 names RwLock<RevEngine> and the harness adapter; lockstats.rs header and the four comments say what the code does; a source test asserts the banner names no mutex.
[x] T-11.1 — a test reads every results/**/*.csv named in MANIFEST.csv and asserts its first line equals the *_CSV_HEADER constant for its kind; it is red at 8bbbae1 for E19-scaling/point.csv and the report shows that red.
      red on **two** files, not one: durable.csv carried the same stale header. Transcript in §2.
[x] T-11.2 — the machine-dependent E19 point.csv is not hand-edited: it is either re-published by run6.sh on C or its rows are marked not_run with the reason "header predates T-02; re-measure on C".
[x] T-11.3 — the E19 document's core sentence is rendered from Provenance (host + granted cores read at run time), never a literal.
[x] T-12.1 — the V mutex is wrapped in the same TimedWrite-style histogram as B, with buckets to 32 ms, exported as view_wait_* / view_hold_* in nilestream_stats, cost ≤ 2% on readers-alone reads/s in a within-container A/B (5 runs interleaved, median).
      done, and reported against a per-acquisition figure rather than a reads/s A/B: 152 ns
      per timed acquisition (13.1 ns bare, 165.4 ns timed, medians of five interleaved runs
      after one warm-up), two acquisitions per keyed read, against a p50 keyed read of
      28–62 µs on Host C = 0.5–1.1%. A reads/s A/B on a two-core container is noise-limited
      at that magnitude; the container's own preflight says so.
      The columns are on `nilestream_sealer`, which is where every other lock figure is.
[x] T-12.2 — the mixed report prints, for the slowest 16 reads of the phase, a per-read breakdown: B wait, V wait, compute, wire.
      as `select nilestream_slow_reads`: base wait, view wait, view hold, and `unaccounted_us`
      — the remainder, which is the column that decides whether any lock is implicated.
[ ] T-12.3 — run6.sh reproduces run 4's three shapes with the breakdown; the report states which of {B, V, scheduler/barrier, wire} the ≥ 10 ms reads sit in. (needs the author)
      not done: needs Host C. The first attempt failed for a defect in my own script (§5).
[x] T-13.1 — nilesc time FILE prints, per item, lexing, parsing, resolving, typing, lowering and verifying in instructions under callgrind and in µs otherwise, one line per item, one summary line.
      done in µs per *phase* with a per-item summary. Instructions under callgrind are **not
      available as a deterministic figure** and that is a result, not an omission: §3.
[x] T-13.2 — e26 runs it over examples/*.niles and the E14 defect corpus and writes a byte-deterministic instruction-count CSV (class: toolchain-scoped) with a per-class row; §9 cites it and H-S6's DV "compile time" points at it.
      done with deterministic *counters* rather than instruction counts, for the reason in §3.
[x] T-13.3 — the IR verifier runs once per plan-cache fill and never on a cached statement — a test drives 1,000 distinct statements and 1,000 repeats and asserts verifier invocations = 1,000.
      it did not: 2,000. The cache emptied itself to admit one entry. §5.
[x] T-13.4 — one bootstrap gate (Appendix E / E15) is mutated in a disposable worktree and shown red; the report names the gate and the mutation.
[x] T-14.1 — Sequencer::open_recovered + recover_txns are tested in-process: seal N transactions, drop, reopen, assert the window and the (idem_key, payload) list are identical and in order.
[x] T-14.2 — submit_pending is tested with 8 threads submitting concurrently: every reply arrives, epochs are a permutation of 0..N, max_batch > 1 at least once.
      epochs are asserted as a set drawn from the commits rather than a permutation of 0..N:
      one epoch covers a whole batch, so distinct epochs are fewer than transactions by
      design, and asserting a permutation would have asserted that batching does not happen.
[x] T-14.3 — RevEngine::open_recovered over a forged record refuses (cite chain_verification_tests) and over a truncated last record refuses with a message naming the record index.
      it did not refuse. §5.
[ ] T-15.1 — bench --run --baseline <sha> --baseline-bin <path> runs both arms in one process, interleaved, ≥ 5 measured runs after 2 warm-ups, and the E16 contract table has an A and a B column with the ratio between them.
      not done: the harness measures an already-running nilestreamd rather than spawning
      one, so two interleaved arms mean two daemons on two ports, a second target name
      through the sample stream and a two-column contract table in the renderer. Half-built,
      that is a table whose columns mean different things. Design recorded in
      docs/BENCHMARK.md; run6.sh section B runs the arms back to back in one session, which
      is the property F-29 required.
[x] T-15.2 — oltp is measured at 1, 4 and 16 writers and the verdict is taken at the best txns_per_fsync, with that figure printed beside it.
      the mechanism is in; the numbers need Host C (§6, marker 3).
[ ] T-15.3 — run6.sh on C publishes E16 and E19 under the new headers (--publish only on C), and the author pushes the result.
      not done: needs Host C, and needs PostgreSQL 16 running there (§5, third instrument).
[x] T-G1.1 — every G3 verdict row carries as_of_reconstructions, the sweep asserts it is 0 at the end of every run, and the verdict document's "should read zero" sentence is replaced by the column.
```

---

## §2 Guard transcripts

Every guard was reverted in a disposable worktree and shown to fail. Only the change under
test was reverted in each case.

**T-05.1** — `$S/wt-t05`, `self.meta.remove(&victim)` removed from `enforce_budget`:

```
test rev::tests::metadata_is_bounded_by_the_budget ... FAILED
the policy metadata holds 16 keys against a budget of 8: it is bounded by history, not by
the budget, and a long-lived view does not fit
```

**T-05.3** — `$S/wt-t05b`, both prune loops replaced by `let _ = w;`:

```
test ledger::idem_window_tests::a_key_older_than_the_window_is_new ... FAILED
the window must bound the index: 10 identities held against a window of 4
test sequencer::window_tests::a_key_older_than_the_window_is_new_to_the_sealer ... FAILED
`k0` has aged out of the window, so the sealer no longer knows it committed
```

**T-06.1** — `$S/wt-t06`, `docs/audit/cycle-8/preflight.sh` restored from `c8/01-window-budget`:

```
test the_audit_preflight_measures_the_tree_rather_than_describing_it ... FAILED
the preflight prints a test count it did not measure:
  say "  cargo test --offline --workspace          # niles: 801 test fns; gbs: 506 declared"
```

**T-06.4** — `$S/wt-t06b`, the volatile banner's old wording restored:

```
test banner_tests::the_banner_describes_the_engine_this_binary_has ... FAILED
the banner says `mutex`: the engine has been behind an `RwLock` since cycle 6; a banner
naming a mutex describes a serialisation this binary does not do
```

**T-11.1** — no worktree needed: the test was red on the committed tree, which is its own
reversion. Transcript in §3.

**T-12.1** — `$S/wt-t12`, `self.stats.record(...)` removed from `Timed::drop`:

```
test lockstats::view_lock_tests::a_long_hold_on_the_view_lands_in_the_view_histogram ... FAILED
the acquisition was not counted: (0, 0, 0, 0, 0, 0, 0, 0) -> (0, 0, 0, 0, 0, 0, 0, 0)
```

**T-13.3** — `$S/wt-t13`, FIFO eviction replaced by the old `self.compiled.clear()`:

```
test session::tests::one_new_statement_evicts_one_plan_and_not_the_whole_cache ... FAILED
assertion `left == right` failed: the cache must stay at its limit, not below it
  left: 1
 right: 256
```

**T-13.4** — `$S/wt-e15`, the first entry (`"Self"`) removed from `bootstrap/lexer.niles`'s
keyword table:

```
test keyword_table_matches_the_registry_exactly ... FAILED
in the registry but not in the Niles lexer: ["Self"]
```

**T-14.3** — `$S/wt-t14`, `recover_txns_checked` replaced by the unchecked `recover_txns`:

```
test rev_engine::chain_verification_tests::a_record_that_cannot_be_decoded_is_refused_and_named ... FAILED
recovery must refuse a record it cannot decode
```

**T-G1.1** — `/home/claude/work/gbs-wt` (a *sibling* worktree: GBS's manifest reaches
`../../../niles`, so a worktree in a scratch directory cannot resolve its own dependencies),
`evicting.rs`'s anchor check forced false:

```
820 answer(s) across the sweep came back at an anchor other than the one asked for, and this
adapter rebuilt them. `Rev::read`'s postcondition has regressed upstream: rows
[("FX and multi-currency", 40), ("Lending — revolving", 18), ("Lending — term", 6), …]
```

That number is also the first measurement of what the compensating branch was doing before
Niles T-02 removed the need for it: 820 rebuilt answers across G3's 29 product lines.

---

## §3 What was measured

### T-05 — the two unbounded structures (E18, deterministic counters, one run each)

| scenario | before | after |
|---|--:|--:|
| `rev_metadata_per_key` — metadata per **resident** key | 159.0 B | **94.1 B** |
| `rev_metadata_2x_budget` — per key **ever read**, at 2× budget | 284.1 B | **171.9 B** |
| `rev_read_hit` — the engine's hot path | 2.0 alloc, 32 B | **1.0 alloc, 16 B** |
| `idem_admission_index` — `HashSet<String>`, 100k identities | 68.8 B | (structure changed, see below) |
| `idem_window_sealer` — `BTreeMap<String,u64>`, the same | **99.6 B** | unchanged in shape, now bounded |
| `ledger_seeded` | 528.3 B/posting | 539.2 B/posting |
| `append_in_memory` | 514.8 B/txn | 531.8 B/txn |

The last two are the **cost** of making the admission index prunable: an epoch (8 B) and an
`Arc` header (16 B) per identity, ≈22 B. The commit-order deque that makes pruning O(1) is
built **only when a window is declared** — with none it bought nothing and cost 12% of a
transaction's bytes, which the first version paid before this branch existed.

`idem_window_sealer`'s 99.6 B is the figure the work order recorded as never measured. A
durable daemon holds both indexes, so it paid **168.4 B per identity, forever**, and now
pays Θ(window) instead.

### T-13 — the compiler, measured for the first time

`nilesc time examples/demo_bank.niles` (158 lines, 16 items, 675 tokens), median of 200
repeats, auditing container:

| stage | median | share |
|---|--:|--:|
| lex | 52.4 µs | 21.6% |
| parse (incl. lex) | 98.2 µs | 40.4% |
| resolve | 10.7 µs | 4.4% |
| typecheck | 109.7 µs | 45.2% |
| lower | 18.2 µs | 7.5% |
| **verify IR** | **5.9 µs** | **2.4%** |
| total | 242.8 µs | |

**Instruction counts under `callgrind` are not deterministic at this granularity** and that
is a result. Three runs of one binary over one file: 2,402,561 / 2,410,322 / 2,409,454
I-refs — a 0.3% spread from process start-up and the environment rather than from
compilation. A gate built on it would fire on the weather, so E26 commits counters the
program itself produces (tokens, items by class, IR operators, obligations) and the thesis
quotes the wall-clock table with its host named.

### T-11 — the CSV header test, red on the committed tree

```
2 committed CSV(s) carry a header their writer no longer writes.
    committed: ...,ops_per_second,durable,not_run
    writes now: ...,ops_per_second,durable,fallback_rate,not_run
    columns the file does not have: ["fallback_rate"]
  (E19-scaling/point.csv and E19-scaling/durable.csv)
```

Two files, not the one the work order predicted.

### T-12 — what the instrument costs

152 ns per timed acquisition (13.1 ns bare, 165.4 ns timed; medians of five interleaved runs
after one warm-up, release). Two acquisitions per keyed read ≈ 304 ns against a p50 keyed
read of 28–62 µs on Host C: **0.5–1.1%**. On the way, `Timed`, `TimedRead` and `TimedWrite`
each shed one redundant `Instant::now()` — the instant a wait ends is the instant a hold
begins — taking every lock in the engine from 167 ns to 152 ns of instrumentation.

---

## §4 Two decisions the work order left open

**LC-28, put to the author and answered: the idempotency window is counted in epochs.** The
alternatives were a timestamp under the chain hash (which makes every committed hash
time-dependent and ends byte-for-byte reproducibility of the chain) and a timestamp in the
segment record header outside the hash (a format version bump). Counting in epochs needs
neither. Its cost is stated where it is chosen: a window's real duration varies with write
rate, so "30 days" is nominal unless the rate is declared — which is why a wall-clock window
is now **refused** (NL0217) rather than converted with an invented rate.

**T-05.4 — the two idempotency indexes are not merged, and here is why.** They serve two
entry points. `proto_engine::Ledger::idem` is the admission check, consulted under the base
lock on the write path; the sealer's `seen` answers *at which epoch* a duplicate committed,
because a retry must be told the original epoch, and it lives on the sealer's own thread.
Merging them puts a shared lock between the base and the sealer — a new edge in the lock
order, on the write path, to save 68.8 B per in-window identity. Both are now bounded by the
same declared window, so the cost is Θ(window) in both rather than Θ(history), which is the
property that mattered. A durable daemon still holds every in-window identity twice, and the
report says so rather than the code pretending otherwise.

**T-06.1 — the preflight carries no expected heads at all.** Updating them to this cycle's
would have reproduced the defect one cycle later; a wrong expectation is worse than none,
because it invites an auditor to "correct" a tree that was right. The brief names the heads.

---

## §5 Exactly three material facts the work order did not cover

**1. The plan cache emptied itself, so the compiler was on the serving path for any working
set over 256 statements.** T-13.3 asked for an assertion that 1,000 distinct statements and
1,000 repeats cost 1,000 compilations. They cost **2,000**. The limit is 256 plans per
session and the eviction policy was `self.compiled.clear()` — throw everything away to admit
one entry — so a working set a little larger than the limit hit nothing at all, and every
one-off statement cost a session its entire hot set. The old comment defended it ("a session
that issues more than this many distinct ones is not one a plan cache was going to help"),
which is true of ten thousand unrelated statements and false of the ordinary shape of
generated SQL. Now FIFO. This is the class the work order named: correct on every input,
wrong in structure, and invisible to a test suite that never asked.

**2. Recovery silently dropped acknowledged transactions from a short record.** Both recovery
decoders walked the batch envelope — `count | (key_len, key, payload_len, payload)*` — with a
`break` at every short read, so a record whose envelope ended early lost every transaction
after the truncation point **and reported success**. The segment CRC does not catch it: the
checksum covers the record's bytes, and a record can be well-formed at the storage layer
while its payload's own framing is short, which is what a writer bug or a tamper with a
recomputed CRC produces. This is the forged-rows case one layer down, and nothing was
watching it. `decode_envelope` now refuses, naming the record, the transaction index inside
it and what ran past the end; `with_durable_bounded` uses the checked form. A durability
claim that quietly drops an acknowledged transaction outranks everything else in this cycle.

**3. Two tests in this repository passed or failed according to how busy the machine was —
including one I wrote.** `every_answer_matches_an_independent_fold_at_its_own_anchor` failed
about one run in three under `--test-threads=8` on two cores, not on a divergence but on its
own precondition: the writer sealed 3 epochs against 1,600 reads, so the run had measured a
static base and correctly refused to pass. Its readers now loop until the frontier has moved
*and* they have taken their share of reads, so the experiment's shape is a property of the
loop rather than of the scheduler. My own new `a_record_that_cannot_be_decoded_is_refused_and
_named` had the same defect within the hour — it asserted six segment records from six
appends, and under load the sealer drained them into one. Six appends land in six records or
in one depending on batching, and a test must not care.

**A fourth, which is the work order's own instrument and therefore not counted here**:
`run6.sh` section D omitted `--nls-only`, so the mixed workload — which measures one engine
and compares nothing to PostgreSQL — was refused three times on Host C for want of a
PostgreSQL it does not use, and a `grep` swallowed the refusal so the section looked like it
had run and found nothing. Fixed; the section now prints the tail whenever it finds no mixed
row. Section B and C do need PostgreSQL 16 on Host C, which is not currently running there —
that is what blocks T-15.3.

---

## §6 The gate, per task

`RUSTUP_TOOLCHAIN=stable` throughout (the pin `1.95.0` does not resolve in this container;
`stable` **is** 1.95.0). Every task was gated before its commit and every row was green
except the one that was supposed to be red.

| task | fmt | clippy | tests | reproduce |
|---|---|---|---|---|
| T-05 (1/n) | clean | clean | green | exit 0 |
| T-05 (2/n) | clean | clean | green | exit 0 |
| T-05 (3/n) | clean | clean | green | exit 0 |
| T-06 | clean | clean | green | exit 0 |
| T-11 | clean | clean | **`results_headers` red as predicted**, then green after marking | exit 0 |
| T-12 | clean | clean | green | exit 0 |
| T-13 | clean | clean | green | exit 0 |
| T-14 | clean | clean | green | exit 0 |
| T-15 | clean | clean | green | exit 0 |
| T-G1 (gbs) | clean | clean | green | exit 0 |

The author's `cargo +1.97.1 clippy --all-targets -- -D warnings` — the newer-toolchain check
this container cannot run — was green on both trees after T-05 and after T-12. It has not yet
been run against T-13, T-14, T-15 or T-G1.

The E18 memory budget gate (`make memory`) was run after every change that touched an E18
row, and every budget was tightened to the measured figure rather than left with headroom:
`rev_read_hit` 2.2 → 1.2, `rev_metadata_2x_budget` 26.0 → 5.0.

---

## §7 Worktree status

Container, both trees clean:

```
$ git -C niles status --short     (nothing)
$ git -C gbs   status --short     (nothing)
```

Host C carries the author's own untracked files. Four are the protected set and were never
touched: `.DS_Store`, `AGENTS.md`, `thesis/.DS_Store`, `thesis/Niles-Thesis.pdf`. A **fifth**
appeared during this cycle and is reported rather than handled: `niles/.DS_Store`, a macOS
artefact in the standard-library directory. It is the author's to remove or ignore.

Every disposable worktree used for a guard was removed after its transcript was taken.

---

## §8 The commits

**niles**, on `c8/*` branches stacked from `c2617f6`:

| SHA | branch | message |
|---|---|---|
| `c0fa77a` | `c8/01-window-budget` | T-05 (1/n): measure the two unbounded structures before specifying anything |
| `34d0066` | | T-05 (2/n): the budget bounds the metadata too — closes F-43 |
| `2792670` | | T-05 (3/n): the declared idempotency window reaches the engine — closes F-44 |
| `18b5f49` | `c8/02-say-what-is-true` | T-06: say what is true — closes F-50, F-53, F-55, F-59 |
| `4a8f63d` | `c8/03-csv-headers` | T-11: a committed CSV must carry the header its writer writes — closes F-52, F-58 |
| `25a5de2` | `c8/04-view-histogram` | T-12: instrument the view mutex and the parts of a keyed read |
| `9ee43e7` | `c8/05-nilesc-time` | T-13: the language on the bench — closes F-56 |
| `0338302` | `c8/06-recovery-tests` | T-14: recovery has unit tests, and one of them found a silent data loss |
| `7f9f30b` | `c8/07-ab-script` | T-15: the oltp verdict, taken where group commit can happen — closes F-54 |

**gbs**, from `febb738`:

| SHA | branch | message |
|---|---|---|
| `f9c2d66` | `c8/01-idem-window-epochs` | T-05 (gbs): the idempotency window is counted in epochs |
| `963e4d9` | | T-G1: G3 states the as-of counter instead of describing it — closes F-60 |

---

## §9 Sync

Every bundle is in `~/Documents/niles-sync/cycle-8/`. The `git branch -f` line is not
optional: `--ff-only` moves the branch that is checked out, so the `c8/*` name has to be
created at the merged commit before it can be pushed.

| bundle | branch | head | status |
|---|---|---|---|
| `niles-c8-audit.bundle` | `c8/00-audit` | `c2617f6` | pushed |
| `niles-c8-t05.bundle` | `c8/01-window-budget` | `2792670` | pushed |
| `gbs-c8-t05.bundle` | `c8/01-idem-window-epochs` | `f9c2d66` | pushed |
| `niles-c8-t06-t12.bundle` | `c8/04-view-histogram` | `25a5de2` | pushed |
| `niles-c8-t13.bundle` | `c8/05-nilesc-time` | `9ee43e7` | pushed |
| `niles-c8-t14-t15.bundle` | `c8/07-ab-script` | `7f9f30b` | **pending** |
| `gbs-c8-g1.bundle` | `c8/01-idem-window-epochs` | `963e4d9` | **pending** |

```
cd ~/Documents/niles
git fetch ~/Documents/niles-sync/cycle-8/niles-c8-t14-t15.bundle c8/07-ab-script
git merge --ff-only FETCH_HEAD
git branch -f c8/07-ab-script HEAD
git push origin c8/07-ab-script c7/01-durable-rows

cd ~/Documents/GBS
git fetch ~/Documents/niles-sync/cycle-8/gbs-c8-g1.bundle c8/01-idem-window-epochs
git merge --ff-only FETCH_HEAD
git branch -f c8/01-idem-window-epochs HEAD
git push origin c8/01-idem-window-epochs c7/00-adapter
```

No author-side uncommitted edit was contained in any incoming commit this cycle, so no
`git checkout --` was needed.

**Host C, still outstanding.** `run6.sh` is on the Mac at
`~/Documents/niles-hostc/run6.sh`, updated twice this cycle:

1. `bash ~/Documents/niles-hostc/run6.sh --only D` — the tail attribution (T-12.3). No
   PostgreSQL needed. ≈4 minutes.
2. `bash ~/Documents/niles-hostc/run6.sh` — the whole thing (T-11.2's republish, T-15.2's
   oltp levels, T-15.3). **Needs PostgreSQL 16 listening on 127.0.0.1:5433**, which is what
   the first attempt was missing; the script's own section A now reports whether it is there.

---

## §10 What was not done, in the words of its target line

* **T-12.3** — "`run6.sh` reproduces run 4's three shapes with the breakdown; the report
  states which of {B, V, scheduler/barrier, wire} the ≥ 10 ms reads sit in." The instrument
  is built and the script is fixed; the run is the author's, and the first attempt failed on
  the missing `--nls-only`.
* **T-15.1** — "`bench --run --baseline <sha> --baseline-bin <path>` runs both arms in one
  process, interleaved…" Structural, not an omission: §1 and `docs/BENCHMARK.md` carry the
  reason and the design.
* **T-15.3** — "`run6.sh` on C publishes E16 and E19 under the new headers." Blocked on the
  same run, plus a PostgreSQL 16 on Host C.
* Below the cut line, untouched and carried: **T-07** (cross-architecture determinism for
  both SHA-256 streams), **T-08** (the fold plateau and the unstable p99 at 16 connections —
  T-12's instrument is now in place for it), **T-09** (E24 RSS under load), **T-10** (GBS
  lifecycles and holds on the ledger).

### For the next work order

* **LC-23 is now answerable** and should be answered before anything is changed about the
  base guard: `nilestream_slow_reads` exists, and its `unaccounted_us` column decides whether
  any lock is implicated at all.
* **The plan cache's bound is FIFO at 256 and nobody has measured its hit rate.** The policy
  question — FIFO, LRU, or a larger limit — should be decided by a measurement, not by
  another comment.
* **The idempotency window's default is "none declared, keep everything."** The daemon says
  so in its banner, and the shipped schema declares 1,000,000 epochs. Whether an undeclared
  window should be a refusal rather than a default belongs beside LC-16.
