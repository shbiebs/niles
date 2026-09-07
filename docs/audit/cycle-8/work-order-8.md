# Work order 8 — Fable's cycle-8 audit of Niles, Nilestream and GBS

*Auditor: Claude Fable 5.1, session `session_012KJD5FHvwxtenT6HxbXskv`, 2026-09-07. Executor: the
Opus session that takes this file. Author: Sergio (Host C, the reference host).*

Trees audited: `niles` at `8bbbae1` (`c7/01-durable-rows`; `9ae1319` plus the brief) and `gbs` at
`febb738` (`c7/00-adapter`). Both are identical on the Mac and on GitHub (`@{u}..HEAD` empty in both
working copies, checked 2026-09-07 through the bridge with `--no-optional-locks`). The four untracked
files in the Mac's `niles` are the author's and were not touched.

**How to read this file.** §0 is the preflight and what this container may conclude. §1 answers the
brief's three questions. §2 is the finding register, ordered by `impact × confidence ÷ cost`. §3 is
the task list with the cut line; every target line is a sentence to be marked `done` or `not done:
why`, and §10 copies them verbatim. §4–§9 are branch stacks, validation, Host C scripts, the LC
ledger and reporting. Nothing in §2 is fixed by this file; the executor fixes, the author runs the
Host C scripts at the marked moments, and the auditor's own instruments are audited in §2 first
(F-50, F-51, F-52), because cycle 7 showed they fail in the same way the code does.

---

## §0 Preflight, verbatim, and the admissibility table

Run in the auditing container at 2026-09-07T03:45:12Z with `docs/audit/cycle-8/preflight.sh`
(committed at `8bbbae1`). Reproduced exactly; the three lines it gets wrong about its own tree are
F-50 in §2.

```
cycle-7 audit preflight — 2026-09-07T03:45:12Z

=== A. host
uname            : Linux 6.18.44-fc-v24 x86_64 GNU/Linux
cores (nproc)    : 2
cpu model        : Intel(R) Xeon(R) Processor @ 2.10GHz
memtotal         : 7.8 GiB
cgroup cpu.max   : n/a
cgroup mem.max   : n/a
NOTE: nproc can exceed what the cgroup actually grants. If cpu.max is not 'max',
      divide the quota by the period — that, not nproc, is your core count.

=== B. filesystem under the tree — THE decisive section
path             : /home/claude/work/niles
mount            : /dev/vda ext4   rw,relatime,resv_strict,resuid=65534,resgid=65534
free space       : 18G avail of 252G

-- the barrier probe: 4 KiB write + fdatasync, 7 times, median reported
barrier          : fdatasync
median           : 98.0 us  ->  10,200 barriers/s
spread           : 83.1-859.1 us
VERDICT          : plausible as a real barrier for THIS container. Still a
                   host-shaped number: usable for within-host ratios only.

reminder: `make fsync-proof` checks the syscall REACHES THE KERNEL. It passes on a
volatile overlay too. The two checks are necessary together; neither is sufficient.

=== C. toolchain
rustc            : rustc 1.95.0 (59807616e 2026-04-14)
cargo            : cargo 1.95.0 (f2d3ce0bd 2026-03-21)
host triple      : x86_64-unknown-linux-gnu
rustfmt          : rustfmt 1.9.0-stable (59807616e1 2026-04-14)
clippy           : clippy 0.1.95 (59807616e1 2026-04-14)
valgrind         : valgrind-3.22.0
python3          : Python 3.11.15
git              : git version 2.43.0
strace           : present
CARGO_TARGET_DIR : <unset, which is correct>

the tree pins `channel = "stable"` with `rust-version = 1.95.0`, because the machine
it was built on had no egress to static.rust-lang.org and could not name a version.
Your `stable` is 1.95.0. If that is NEWER than 1.95.0, a new lint can turn
`clippy -- -D warnings` red for reasons unrelated to this code. **A red gate from a
newer toolchain is a finding about the pin, not about the code** — report it that way.

=== D. PostgreSQL
psql             : psql (PostgreSQL) 16.13 (Ubuntu 16.13-0ubuntu0.24.04.1)
postgres         : PostgreSQL 16.13 (Ubuntu 16.13-0ubuntu0.24.04.1)
pg_isready       : /var/run/postgresql:5432 - accepting connections

`numeric_binary_oracle` needs a running server with a `bench` role and PGPORT set.
Without it the tree reads RED for a missing service, not a defect — do not report that
as a failing test. Start one with `pg_ctlcluster 16 main start` or a private cluster:
  initdb -D <pgdata outside the repo> -U bench --auth=trust
  pg_ctl -D <pgdata> -o '-p 5433 -c listen_addresses=127.0.0.1' start
  createdb -h 127.0.0.1 -p 5433 -O bench bench
Record fsync / synchronous_commit / full_page_writes / wal_level / shared_buffers /
work_mem / max_wal_size AND **wal_sync_method** beside any comparison.

=== E. network egress
https://github.com          : 400
https://crates.io           : 403
https://static.rust-lang.org: 000FAIL

Egress here is YOURS, not the executing agent's. **No task you write may require
network access at execution time**: Opus's container has no egress to crates.io or
static.rust-lang.org, and no task may install anything. Both trees have **zero external
dependencies**, so a bare toolchain builds them offline — verify that with
`cargo test --offline --workspace` rather than assuming it.

=== F. trees
/home/claude/work/niles
  HEAD           : 8bbbae1 Cycle 8: the audit brief for Fable
  branch         : c7/01-durable-rows
  dirty          : 0 entries
/home/claude/work/gbs
  HEAD           : febb738 T-04a (2/n): a third 1.97.1 lint, in the repository F-33 never checked
  branch         : c7/00-adapter
  dirty          : 0 entries

EXPECTED at the start of cycle 7:
  niles  d9c8699  c6/06a-lock-order   (or 474f153 c6/audit-cycle-7, which adds only docs)
  gbs    e803b7d  c6/07-hold-index
If you are on the author's Mac, the FOUR untracked files in `niles` are his and must
never be edited, staged, deleted, moved or bundled: .DS_Store, AGENTS.md,
thesis/.DS_Store, thesis/Niles-Thesis.pdf. Anything else untracked is yours to explain.

=== G. gate — run these yourself and record the result
  cargo fmt --all -- --check
  cargo clippy --offline --all-targets -- -D warnings
  cargo test --offline --workspace          # niles: 801 test fns; gbs: 506 declared
  make reproduce                            # must exit 0 with a clean diff
  make fsync-proof                          # needs strace
A red row here is a RESULT. Record it; never fix it by deletion.

=== H. admissibility — fill this in and put it in your work order
  cores actually granted        : ____   (cgroup quota, not nproc)
  barrier + median + rate       : ____
  storage evidence?             : yes / NO (volatile mount)
  may publish durability rows?  : ____
  may publish >3-core curves?   : ____   (needs the cgroup answer above)
  toolchain newer than 1.95.0?  : ____
  valgrind attribution possible : ____
  PostgreSQL comparison possible: ____   (and under which wal_sync_method)

preflight complete.
```

### §0.1 The gate, run by the auditor at `8bbbae1` / `febb738`

`RUSTUP_TOOLCHAIN=stable` throughout (the pin `channel = "1.95.0"` does not resolve here; `stable`
*is* 1.95.0, so this is the pinned compiler under another name). Every row green:

| gate | niles | gbs |
|---|---|---|
| `cargo fmt --all -- --check` | clean | clean |
| `cargo clippy --offline --all-targets -- -D warnings` | clean (1.95.0) | clean (1.95.0) |
| `cargo test --offline --workspace` | **841** test functions, 0 failed, PostgreSQL 16.13 up | 506, 0 failed |
| `make reproduce` | exit 0, `git diff --exit-code -- results/` clean | n/a |
| `make fsync-proof` | fdatasync reaches the kernel | n/a |
| author's `cargo +1.97.1 clippy`, both trees | GREEN (reported by the author 2026-09-06) | GREEN |

### §0.2 Admissibility

| question | answer for this container |
|---|---|
| cores actually granted | **2** (`cpu.max` n/a — no cgroup v2 file; `nproc` 2 is the only figure; the shared-Xeon steal seen in cycle 7 still applies) |
| barrier, median, rate | fdatasync, **98.0 µs**, **10,200/s** (spread 83–859 µs). Cycle 7's preflights on the same host class read 4,961–5,825/s: a **1.8–2.1× spread between instances of one host class**, which is the reason the host column exists |
| storage evidence | yes (`/dev/vda` ext4, `resv_strict`; `make fsync-proof` reaches the kernel). Real barrier, unknown device beneath it |
| may publish durability rows | **no**. Never from a container (constraint); within-container ratios only |
| may publish >3-core curves | **no** (2 cores) |
| toolchain newer than 1.95.0 | no; equal. The author's 1.97.1 gate is the newer-toolchain check, and it is green |
| valgrind attribution possible | yes (3.22.0) — deterministic counters admissible |
| PostgreSQL comparison possible | yes, 16.13, `wal_sync_method = fdatasync`, `synchronous_commit = on`, `fsync = on` — **as a same-session A/B only**, never as an absolute |

Rule for everything below: a figure from this container is a *within-container ratio* or a
*deterministic counter*; every wall-clock number that decides anything is Host C's, produced by a
script in §6 at a moment marked **the author must run this now**.

---

## §1 Executive judgement

**Where the three artefacts stand.** Cycle 7 closed the two claims that were not true (durability
that did not restore rows; a keyed read that rebuilt ~46% of its answers) and made the mixed
workload a row. What is left is measured on Host C at `8d229ea` (run 4, 2026-09-06): readers alone
82–92k reads/s at p50 28–62 µs; mixed 85–95k reads/s with **0 fallbacks** at every shape; writers
256–844/s at p50 4–8 ms, which is the Mac's `F_FULLFSYNC` and not the engine; `txns_per_fsync` 1.03
at one writer rising to 2.30 at four. Against the contract (SPEC-ENGINE Part 0) the last committed
A/B is oltp 1.07×, analytical 1.93×, report 2.17–2.68× — all **NOT MET** but `point` — and it was
measured before T-02 and before the counter change, so it is a *stale* verdict, not a current one
(F-53). Nothing in this container can move that verdict; only `run6.sh` (§6) can.

### Q1 — Nilestream: is the next ceiling structural or another implementation defect?

**Another implementation defect, and this audit names two of the same class, plus one that is
design.** The class — *correct on every input, wrong in structure* — recurs in:

1. **The metadata that outlives the budget** (F-43/F-44, measured this cycle). The eviction budget
   bounds *values*; `reads_of`, `last_read` and the version-only `slots` grow with every key ever
   read: **~340 B per key ever read, linear, 33 MiB at 100k distinct keys against a budget of 2,500
   entries — 40× the budget's own footprint** (probe in §2, F-43). And the idempotency window is two
   unbounded copies, not one: `Ledger::seen` (`HashSet<String>`, measured 67.5 B/key at 100k and
   108 B/key at 1M, ~357 B per identity end-to-end) and the sealer's `seen: BTreeMap<String, u64>`
   (`sequencer.rs:250`), which nobody has measured. Every input is answered correctly; memory is
   Θ(history) where the design promises Θ(budget). *This* is the next ceiling for the memory goal
   and it is implementation, not design: T-05.
2. **The 12–13 ms read tail** (F-49, LC-23). Attributable to nothing yet instrumented: the base
   lock's `lock_wait_max_us` is 548–1,655 µs across the three shapes, so the base is ruled out; the
   view mutex (V) has no histogram; `submit_pending` is a channel send so no fsync sits under any
   lock the reader takes (`sequencer.rs:421–429`). Read max equals write max (12,405/13,067 µs,
   13,091/13,033, 12,955/13,023) to within 1%, which is the shape of *one reader per run* landing
   behind the writer's barrier — through the OS scheduler, since no lock chain reaches the fsync.
   Until V is instrumented that is a hypothesis; T-12 instruments it and `run6.sh` measures it. If
   it holds, the tail is the Mac's `F_FULLFSYNC` and a core count, not the engine.
3. **Design, and it stays**: `append` holds the base **write** guard across `advance` on the view
   (`rev_engine.rs:776–780`) so a reader waits on a writer's apply. That is apply-before-publish
   and the lock order O < B < P < V < C; it costs the reader at most the apply of one batch of
   resident entries. It is a *bound*, not a defect, and LC-23 should decide whether the guard could
   be downgraded only after T-12 says the tail is or is not there.

Arithmetic for the reachability claim that matters — **oltp 5–10× PostgreSQL**: on C, PostgreSQL's
oltp arm is 2,706–4,519 ops/s (two instances). Nilestream's write path is barrier-bound at 1.03
txns/fsync with one writer; at `F_FULLFSYNC` ≈ 4–8 ms that is 125–250 writes/s per writer, and
the contract is only reachable through group commit — `max_batch` 8 and 2.30 txns/fsync at four
writers on C. **5× PostgreSQL requires ≥ 5 × 2,706 ≈ 13,500 committed txns/s, i.e. ≈ 54–108
txns per fsync at C's barrier.** The sealer drains up to 4,096, so the design allows it; the
measurement says the harness has never offered it more than 8. The oltp verdict is therefore a
*harness* question before it is an engine one: the E16 oltp arm runs one connection. That is
F-54, and T-15's A/B script runs oltp at 1/4/16 writers so the verdict is taken where the design
says it should be taken.

### Q2 — Niles: has anything been measured about the language at all?

**No — and one hypothesis in the thesis names "compile time" as a dependent variable that has no
experiment.** H-S6 (`thesis/01-introduction.md:174–175`): *DV: runtime overhead; residual violation
count; compile time.* E14 measures the first two; nothing measures the third. Probed here:
`nilesc check` on the three `examples/*.niles` (121–158 lines) takes 3.5–4.1 ms wall of which the
container's fork/exec floor is 3.9 ms (`/bin/true`), so compilation is **sub-millisecond per
program and unresolved below that** — the instrument is too coarse, not the cost too small to
matter. What an efficiency claim about a *language* would be evidence of, in this thesis: (a) the
static checks H-S6 promises cost less at compile time than the runtime policing they replace — a
counted claim (instructions per statement class under `callgrind`, byte-deterministic); (b) the
typed IR is not on the hot path — the verifier runs once per plan-cache fill, never per read; (c)
the bootstrap gates (Appendix E, E15) fail on a regression. (a) and (b) are admissible now with
`callgrind` in this container; (c) needs one mutation test. That is T-13, above the cut line
because a DV with no experiment is a stale claim in the hypothesis table, and cheap.

### Q3 — GBS: is an efficiency task admissible yet?

**Still no, with one narrower reason than cycle 7's.** F-24 (no product trace) and F-12 (lifecycles
do not survive a restart) stand; T-10 is unchanged below the line. New this cycle: GBS's
`as_of_reconstructions` is asserted zero in exactly one test
(`conservation_under_eviction.rs:753`, "no as-of read yet" — before any as-of read), the G3
verdict document *says* it should read zero (`results/G3-verdict.md:86`) and **no G3 row carries
it**. The compensating branch is documented as unreachable and nothing over the sweep proves it.
That is a five-line task (T-G1) and it is the only GBS task above the line. No efficiency figure
for GBS is admissible until a G3 run states, in a column, that every answer came from the view at
the anchor it asked for.

---

## §2 Findings — the register

Class ∈ {wrong-measurement, correctness, liveness, guarantee-bounded, instrument-gap, negative,
stale-claim}. EV = impact × confidence ÷ cost, each 1–5. HI = hypothesis-independent (a defect
whatever the thesis claims); HS = hypothesis-shaped. Ordered by EV; ties to correctness.

| id | class | finding | EV | HI/HS | evidence | closes in |
|---|---|---|--:|---|---|---|
| **F-43** | guarantee-bounded | view metadata is Θ(keys ever read), not Θ(budget): ~340 B/key, 40× budget at 100k | 5×5÷2 = **12.5** | HI | `rev.rs` `reads_of`/`last_read`/`slots`; probe §2.1 | T-05 |
| **F-44** | guarantee-bounded | idempotency window infinite, and **two copies**: `Ledger::seen` (108 B/key at 1M) + sealer `seen: BTreeMap<String,u64>` (unmeasured) | 5×5÷2 = **12.5** | HI | `ledger.rs` `seen`; `sequencer.rs:250,287,293` | T-05 |
| **F-50** | instrument-gap | the cycle-8 preflight misreports its own tree: "pins `channel = "stable"`" (it pins `1.95.0`), "801 test fns" (841), expected heads are cycle 7's | 3×5÷1 = **15** | HI | `docs/audit/cycle-8/preflight.sh` §C, §F, §G | T-06 |
| **F-51** | instrument-gap | `run5.sh` reuses `wt-gbs` with `[ -d … ] \|\| git worktree add` — no retarget, no refuse; the exact pattern that made run 4 measure `cea8a1e` | 4×5÷1 = **20** | HI | `~/Documents/niles-hostc/run5.sh:14` | `run6.sh` §6 |
| **F-52** | stale-claim | committed `results/E19-scaling/point.csv` has the **pre-T-02 header** (no `fallback_rate`); nothing tests a committed CSV header against its `*_CSV_HEADER` constant | 4×5÷1 = **20** | HI | `results/E19-scaling/point.csv:1` vs `workloads.rs` header | T-11 |
| **F-53** | stale-claim | `thesis/09:§9.14.1` says every generated E16 document "now carries a header naming its host…"; the committed `results/E16-wallclock.md` has none (cycle-5 run) and the committed contract verdicts predate T-02 | 4×5÷1 = **20** | HS | `results/E16-wallclock.md:1–12`; `thesis/09-evaluation.md` §9.14.1 | T-06 (mark), `run6.sh` (re-measure) |
| **F-54** | wrong-measurement | the oltp contract arm is measured at one connection, where the sealer is structurally at 1.03 txns/fsync; the contract is only reachable under group commit | 5×4÷2 = **10** | HS | `bench.rs` oltp workload; run 4 `txns_per_fsync` | T-15 |
| **F-55** | stale-claim | ch3's absence lattice defines `Present(v, e′)` at a *point* (`thesis/03:41`); the runtime certifies over an *interval* `[stamp, effective]` and Theorem 4.1 step (3) states the interval. The lattice and the theorem disagree on the type of `Present` | 4×4÷1 = **16** | HS | `thesis/03-theoretical-framework.md:38–44`; `thesis/04:27–65`; `rev.rs` `read` | T-06 |
| **F-56** | instrument-gap | H-S6's DV "compile time" has no experiment; `nilesc` cost is below the wall-clock floor and uncounted | 3×5÷1 = **15** | HS | `thesis/01:174–175`; probe §2.2 | T-13 |
| **F-57** | instrument-gap | recovery has no unit test: `Sequencer::open_recovered`, `recover_txns`, `submit_pending`, `RevEngine::open_recovered` are reached only by the crash-protocol integration test (serial writers) and by experiments | 4×5÷2 = **10** | HI | sweep §2.3 | T-14 |
| **F-49** | instrument-gap | the 12–13 ms mixed read tail (LC-23) is unattributed: V has no histogram; base ruled out (`lock_wait_max_us` ≤ 1,655) | 3×4÷2 = **6** | HI | run 4 `mixed-*.txt` (§2.4) | T-12 + `run6.sh` |
| **F-58** | stale-claim | `bench.rs:2298` hard-codes "> This host has 2 cores." into every E19 document, including Host C's ten-core runs | 3×5÷1 = **15** | HI | `crates/bank-bench/src/bin/bench.rs:2298` | T-11 |
| **F-59** | stale-claim | `docs/SPEC-ENGINE.md:705` lists `Serving` implementors as "none today"; `RwLock<RevEngine>` implements it (`rev_engine.rs:884`) | 2×5÷1 = **10** | HI | as cited | T-06 |
| **F-60** | instrument-gap (GBS) | `as_of_reconstructions` asserted zero once, before any as-of read; G3's verdict prose says "should read zero"; no G3 row carries the counter | 3×5÷1 = **15** | HI | `gbs …/conservation_under_eviction.rs:753`; `results/G3-verdict.md:86` | T-G1 |
| **F-61** | negative | **no lock-order defect.** Both paths take B before V (`the_base_is_acquired_before_the_view_on_every_path_that_takes_both`); `answer_from_view` takes B exactly once; `submit_pending` holds no lock across the fsync | — | HI | `rev_engine.rs:2885–2935`; `sequencer.rs:421` | closed |
| **F-62** | negative | **no fallbacks under any mixed shape on C** at `8d229ea`: 4r2w, 8r1w, 8r4w all `fallbacks 0`, `view_answers` 1.34–1.49M — T-02's postcondition holds under concurrency, not only in the unit test | — | HS | run 4 `mixed-*.txt` | closes LC-17 |
| **F-63** | negative | ch6's durability sentence matches the code: recovery verifies each link and the daemon **exits 2** rather than serve volatile when the sink cannot open (`main.rs:150–153`) | — | HS | as cited | closed |
| **F-64** | negative | ch4 Theorem 4.1 step (3) states the certification interval as the invariant `Rev::read` now implements; no thesis text describes the old "fresher entry" rule | — | HS | `thesis/04:27–65` | closed |
| **F-46** (carried) | instrument-gap | no cross-architecture determinism assertion; two SHA-256 streams now (`chain`, `write_rows`) | 3×4÷2 = 6 | HI | WO7 T-07 | T-07 (below line) |
| **F-28** (carried) | wrong-measurement | fold plateau 4.4–4.6× at 8–16 conns on C; p99 unstable at 16 (101,551 vs 17,017/24,795 µs) | 3×3÷3 = 3 | HS | run 4 arms | T-08 (below line) |
| **F-12/F-24** (carried) | guarantee-bounded (GBS) | lifecycles do not survive restart; no product trace | 4×5÷5 = 4 | HI | WO6/WO7 | T-10 (below line) |

### §2.1 F-43 / F-44 — measured, this container, one run (deterministic counters)

Scratch crate over `nilestream-core::rev` and `proto-engine::ledger` (not committed; the executor
rebuilds it as T-05's baseline test, §3). Allocation bytes via a counting global allocator;
counts are exact, bytes are toolchain-scoped.

| structure | keys | bytes/key | note |
|---|--:|--:|---|
| `Ledger::seen` (`HashSet<String>`, 36-char keys) | 100,000 | 67.5 | table + string heap |
| same | 1,000,000 | 108 | after the last table doubling; 0.73–1.89 s per 1M inserts |
| whole ledger, per committed identity (rows + seen + epochs) | 1,000,000 | ~357 | the memory goal's unit |
| view metadata per key ever read (`reads_of` + `last_read` + `slots` version-only) | 100,000 | **~340** | 33 MiB with `budget = 2,500`; **linear in keys read, flat in budget** |
| — apportioned | | 98 / 98 / 112 | two `HashMap<Vec<i64>, …>` and the slot map; the `Vec<i64>` key is ~50 B of it each time |
| — with a packed `(i64, i64)` key | | ~50 total | what T-05's target is derived from |
| sealer `seen: BTreeMap<String, u64>` | | **not measured** | second copy of every identity; T-05's baseline must include it |

What the repair gives up: a key older than the window is admitted *as new* — a replay after 30 days
commits twice. That is the language's own semantics (`idem: IdemKey window 30.days`) and the thesis
says so; T-05 makes the window real rather than infinite.

### §2.2 F-56 — `nilesc` probe

| program | lines | `check` wall | `explain` wall | floor (`/bin/true`) |
|---|--:|--:|--:|--:|
| `examples/available_balance.niles` | 120 | 3,487 µs | — | 3,860 µs |
| `examples/demo_bank.niles` | 158 | 4,082 µs | 4,312–5,044 µs | |
| `examples/inventory.niles` | 121 | 4,033 µs | — | |

Below the floor: no figure. Instruction counts under `callgrind` are the admissible instrument.

### §2.3 F-57 — public functions no test region references (by name)

`nilestream-core/src/rev.rs`: `applied_through`, `iter_resident`. `nilestream-ledger/src/sequencer.rs`:
`open_recovered`, `recover_txns`, `submit_pending`. `nilestream-server/src/rev_engine.rs`:
`describe_class`, `open_recovered`, `serve_path_of`. `proto-engine/src/ledger.rs`:
`key_update_count`, `posting_count`, `reset_counters`, `without_chaining`. The recovery trio is the
one that matters: the only thing that exercises it end to end is `tests/crash_recovery.rs`, whose
writers are serial (§8.2 of the brief) and which runs a binary through `psql`. The sweep is a
name-reference heuristic (`#[cfg(test)]` regions plus `tests/`), so a function reached through a
wrapper is invisible to it; the executor confirms each with `cargo llvm-cov` if present, else by
a deliberate `todo!()` in a disposable worktree.

### §2.4 F-49 — run 4 (`results4-20260906-092323`, arm A = `8d229ea`, Host C, macOS 25.6 arm64)

| shape | readers-alone reads/s · p50 · max | mixed reads/s · p50 · p99 · **max** | writes/s · p50 · **max** | fold-shaped | `lock_wait_max_us` | fallbacks |
|---|---|---|---|---|--:|--:|
| 4r2w | 82,101 · 28 · 431 | 85,445 · 23 · 182 · **12,405** | 356 · 5,991 · **13,067** | 0.02% | 548 | 0 |
| 8r1w | 91,641 · 62 · 755 | 94,707 · 60 · 246 · **13,091** | 302 · 3,932 · **13,033** | 0.08% | 884 | 0 |
| 8r4w | 90,270 · 61 · 756 | 89,289 · 53 · 264 · **12,955** | 844 · 5,958 · **13,023** | 0.18% | 1,655 | 0 |

Read max tracks write max to within 1% at every shape; p99 is untouched. One reader per phase
lands behind one barrier. The lock that is instrumented cannot explain it; the one that is not (V)
and the scheduler can. T-12 tells them apart.

### `MISMATCH` register (thesis text vs code, to be resolved in T-06)

| id | where | divergence |
|---|---|---|
| `MISMATCH-lattice-interval` | `thesis/03:41` | `Present(v, e′)` is a point; the runtime and Theorem 4.1 certify `[stamp, effective]` |
| `MISMATCH-e16-header` | `thesis/09:§9.14.1` | "now carries a header" — true of the generator since `5d37692`, false of the committed document until the next `--publish` on C |
| `MISMATCH-serving-impl` | `docs/SPEC-ENGINE.md:705` | "none today" |
| `MISMATCH-two-cores` | `bench.rs:2298` → every `E19-scaling.md` | host core count typed into prose |

---

## §3 Guarantees, checked at `8bbbae1`

Durable-before-visible (`observe(frontier())`, fail-stop on barrier failure, verified recovery,
`exit 2` on an unopenable sink) — holds. One sealer, one epoch order — holds. Honest absence —
holds in the code; the *lattice text* is one point short (F-55). Fold-never-field, product purity,
self-describing amounts (scale per currency, four SQLSTATEs) — hold. `Rev::read` at the anchor
asked for — holds, and under concurrency on C (F-62). Lock order O < B < P < V < C — holds, source-
guarded (F-61). No `unsafe` — `grep -rn "unsafe" crates/ --include=*.rs` returns only the
allocator probe in `tools/memprobe` (declared). Zero external dependencies — `cargo test --offline`
green in both trees.

---

## §4 Global constraints (carried verbatim from the brief's §10; binding on every task)

durable-before-visible; one sealer per ledger; one total epoch order; full retention; honest
absence; fold-never-field; product purity; self-describing amounts; GBS layering; **no `unsafe`**;
**no default-on-error**; **zero external dependencies**; no fabricated results; the thesis follows
the code with `MISMATCH-<id>` at every divergence; a red test is a result and is never fixed by
deletion; honest refusal over silent fallback; `BLOCKED-<id>` on ambiguity; apply-before-publish;
the lock order **O < B < P < V < C**, S a leaf; every optimisation ships its guard in the same
commit and the executor **proves the guard fails on the reverted change in a disposable `git
worktree`**, transcript in the report; no performance claim without a reproducible command and a
provenance header; benchmarks touch committed artefacts only with `--publish`, never from a
container; micro-benchmark gains are never contract results; a currency's wire code is its
declaration index; `Rev::read` answers at the anchor it was asked for — no caller may reintroduce
a compensating branch.

Environment: **no task may require network access at execution time; no task installs anything.**
`RUSTUP_TOOLCHAIN=stable` on every cargo/make invocation in the container. Attribution comes from
the executing session's own instructions. Edits to Rust source through Python must be line-based —
a `\`-continued heredoc collapsed string literals three times in cycle 7 and one reached a live
daemon's `DETAIL` text.

---

## §5 Tasks, in dependency order

Format per task: closes · files · baseline (this container unless marked C) · target (a ratio or a
sentence) · method · acceptance · **guard and its reversion** · guardrails. **Target lines are the
§10 checklist, verbatim.**

### T-05 — The window is a window; the budget bounds everything — closes F-43, F-44

*Files:* `crates/nilestream-core/src/rev.rs`; `crates/proto-engine/src/ledger.rs`;
`crates/nilestream-ledger/src/sequencer.rs`; `crates/niles-ir` (carry `window` from `idem:` to the
plan); `crates/nilestream-server/src/{rev_engine,session}.rs` (`nilestream_stats` columns);
`docs/SPEC-ENGINE.md`; `thesis/06`.

*Baseline (this container, deterministic):* metadata ~340 B per key ever read, 33 MiB at 100k keys
under `budget = 2,500`; `Ledger::seen` 108 B/key at 1M; sealer `seen` **measure first** — that number
is T-05's first commit and goes in the report whatever the rest of the task does.

*Targets:*
- **T-05.1** — after committing 2× budget distinct keys, `reads_of.len() ≤ budget` and
  `last_read.len() ≤ budget` and the slot map holds ≤ budget non-`⊥` entries: metadata is
  Θ(budget). *(done / not done: why)*
- **T-05.2** — metadata bytes per resident key ≤ 120 (from ~340), by packing the `(acct, cur)` key
  and evicting the two maps with the entry. *(done / not done: why)*
- **T-05.3** — `idem: IdemKey window 30.days` reaches the sequencer: a key whose epoch is older than
  the window is admitted as new, and `seen` (both copies, or one shared copy) holds only keys inside
  the window; `nilestream_stats` gains `idem_window_keys` and `view_metadata_keys`. *(done / not done:
  why)*
- **T-05.4** — the sealer's `BTreeMap` and the ledger's `HashSet` are one structure or the report
  says why two are needed. *(done / not done: why)*

*Method:* evict metadata in the same call that evicts the value; prune `seen` by epoch age at seal
time (epoch → wall clock through the epoch's recorded time, which the durable record must carry —
if it does not, `BLOCKED-idem-clock` and stop, do not invent a clock).

*Acceptance:* the four targets as tests; E18 memory rows unchanged for the budget-bound workloads.

*Guard and reversion:* test `metadata_is_bounded_by_the_budget` (commit 2×budget keys, assert the
three lengths) — in a disposable worktree, revert the eviction of the maps and show it fails;
`a_key_older_than_the_window_is_new` — revert the pruning and show it fails.

*Guardrails:* honest absence — evicting metadata must map `Present → Hole(e)`, never `⊥`: the slot
map keeps the *version* (that is the 112 B/key that must shrink, not vanish). A key evicted from
`seen` inside the window is a correctness bug; the window is the only thing that may shrink `seen`.

### T-06 — Say what is true — closes F-50, F-53, F-55, F-59, the `MISMATCH` register, WO7's T-06 remainder

*Files:* `docs/audit/cycle-8/preflight.sh`; `thesis/03-theoretical-framework.md:38–44`;
`thesis/09-evaluation.md` §9.14.1; `docs/SPEC-ENGINE.md:705`; `crates/nilestream-server/src/lockstats.rs`
header; `thesis/07:69`; `README.md`; the four comments WO7 T-06 lists.

*Targets:*
- **T-06.1** — `preflight.sh` reports the pin it finds (`grep channel rust-toolchain.toml`), the test
  count it counts (`grep -rc "#\[test\]"` summed), and the expected heads of *this* cycle; a
  source test asserts the script contains no literal test count. *(done / not done: why)*
- **T-06.2** — ch3's lattice reads `Present(v, [e_s, e_f])` (or states in one sentence that a
  Present entry's certification is an interval and names §4's Theorem 4.1 step (3)); the
  `MISMATCH-lattice-interval` marker is placed and resolved in the same commit. *(done / not done:
  why)*
- **T-06.3** — §9.14.1 says the committed E16 document predates the header and names the commit
  (`5d37692`) from which every generated one carries it; `MISMATCH-e16-header` stays until
  `run6.sh` publishes. *(done / not done: why)*
- **T-06.4** — `SPEC-ENGINE.md:705` names `RwLock<RevEngine>` and the harness adapter; `lockstats.rs`
  header and the four comments say what the code does; a source test asserts the banner names no
  mutex. *(done / not done: why)*

*Guard and reversion:* the two source tests; revert one line each and show red.

### T-11 — Committed CSVs carry their headers, and E19's prose stops naming a core count — closes F-52, F-58

*Files:* `crates/bank-bench/tests/results_manifest.rs` (or a sibling `results_headers.rs`);
`crates/bank-bench/src/bin/bench.rs:2298`; `results/E19-scaling/point.csv`; `results/MANIFEST.csv`.

*Baseline:* `point.csv` header lacks `fallback_rate`; `E19-scaling.md` says "2 cores" on C.

*Targets:*
- **T-11.1** — a test reads every `results/**/*.csv` named in `MANIFEST.csv` and asserts its first
  line equals the `*_CSV_HEADER` constant for its kind; it is red at `8bbbae1` for
  `E19-scaling/point.csv` and the report shows that red. *(done / not done: why)*
- **T-11.2** — the machine-dependent E19 `point.csv` is not hand-edited: it is either re-published by
  `run6.sh` on C or its rows are marked `not_run` with the reason "header predates T-02; re-measure
  on C" — the executor does the second and the author's `run6.sh` does the first. *(done / not
  done: why)*
- **T-11.3** — the E19 document's core sentence is rendered from `Provenance` (host + granted cores
  read at run time), never a literal. *(done / not done: why)*

*Guard and reversion:* T-11.1 itself is the guard; its reversion is the committed tree.

### T-12 — Instrument V and the read phases; attribute the tail — closes F-49's instrument half; feeds LC-23

*Files:* `crates/nilestream-server/src/{lockstats,rev_engine}.rs`; `crates/bank-bench/src/bin/bench.rs`
(mixed report); `docs/BENCHMARK.md`.

*Baseline (C, run 4):* read max 12.4–13.1 ms tracks write max within 1%; `lock_wait_max_us` on B
≤ 1,655; V uninstrumented.

*Targets:*
- **T-12.1** — the V mutex is wrapped in the same `TimedWrite`-style histogram as B, with buckets
  to 32 ms, exported as `view_wait_*` / `view_hold_*` in `nilestream_stats`, cost ≤ 2% on
  readers-alone reads/s in a within-container A/B (5 runs interleaved, median). *(done / not done:
  why)*
- **T-12.2** — the mixed report prints, for the slowest 16 reads of the phase, a per-read breakdown:
  B wait, V wait, compute, wire — so the tail is attributed to a name, not a guess. *(done / not
  done: why)*
- **T-12.3** — `run6.sh` §6 reproduces run 4's three shapes with the breakdown; the report states
  which of {B, V, scheduler/barrier, wire} the ≥ 10 ms reads sit in. **Needs the author.** *(done /
  not done: why)*

*Guard and reversion:* a test that a read holding V for 5 ms (test-only hook, `cfg(test)`) shows in
`view_hold_max_us ≥ 5,000`; revert the histogram and it fails.

*Guardrails:* no change to the lock order; no downgrade of the base write guard in this task —
that is LC-23's decision after T-12.3 and not before.

### T-13 — The language on the bench: compile cost per statement class, counted — closes F-56

*Files:* `crates/niles-lang` (`nilesc time FILE`), `crates/experiments` (new `e26`),
`results/E26-compile-cost.{csv,md}`, `results/MANIFEST.csv`, `thesis/01:175`, `thesis/09`.

*Baseline:* nothing; `nilesc check` is below the container's exec floor.

*Targets:*
- **T-13.1** — `nilesc time FILE` prints, per item (view / fn / insert / report / point), lexing,
  parsing, resolving, typing, lowering and verifying in *instructions* when run under
  `valgrind --tool=callgrind` and in µs otherwise, one line per item, one summary line.
  *(done / not done: why)*
- **T-13.2** — `e26` runs it over `examples/*.niles` and the E14 defect corpus and writes a
  byte-deterministic instruction-count CSV (class: toolchain-scoped) with a per-class row; §9 cites
  it and H-S6's DV "compile time" points at it. *(done / not done: why)*
- **T-13.3** — one sentence, measured: the IR verifier runs once per plan-cache fill and never on a
  cached statement — a test drives 1,000 distinct statements and 1,000 repeats and asserts verifier
  invocations = 1,000. *(done / not done: why)*
- **T-13.4** — one bootstrap gate (Appendix E / E15) is mutated in a disposable worktree and shown
  red; the report names the gate and the mutation. *(done / not done: why)*

*Guard and reversion:* T-13.3's test; revert the cache key and it fails.

*What it gives up:* nothing in the engine; ~300 lines of instrumentation in the compiler front-end.

### T-14 — Recovery has unit tests — closes F-57; feeds LC-25

*Files:* `crates/nilestream-ledger/src/sequencer.rs` tests; `crates/nilestream-server/src/rev_engine.rs`
tests.

*Targets:*
- **T-14.1** — `Sequencer::open_recovered` + `recover_txns` are tested in-process: seal N
  transactions, drop the sequencer, reopen, assert the window and the `(idem_key, payload)` list
  are identical and in order. *(done / not done: why)*
- **T-14.2** — `submit_pending` is tested with 8 threads submitting concurrently: every reply
  arrives, epochs are a permutation of `0..N`, `max_batch > 1` at least once. *(done / not done:
  why)*
- **T-14.3** — `RevEngine::open_recovered` over a segment with a forged record refuses (already
  covered by `chain_verification_tests` — cite it, do not duplicate) and over a truncated last
  record refuses with a message naming the record index. *(done / not done: why)*

*Guard and reversion:* each test; revert the recovery ordering (`seed_epochs + k`) to `k` and
T-14.1 fails.

### T-15 — The same-session A/B script, and oltp at the connection count the design needs — closes F-54, LC-26

*Files:* `docs/audit/cycle-8/run6.sh` (§6, written by the auditor; the executor extends it if T-12
adds flags); `crates/bank-bench/src/bin/bench.rs` (`--oltp-connections 1,4,16`); `docs/BENCHMARK.md`.

*Targets:*
- **T-15.1** — `bench --run --baseline <sha> --baseline-bin <path>` runs both arms in one process,
  interleaved, ≥ 5 measured runs after 2 warm-ups, and the E16 document's contract table has an
  *A* and a *B* column with the ratio between them (today's `--baseline` is a label and the arms
  run back to back; `docs/BENCHMARK.md:127–145`). *(done / not done: why)*
- **T-15.2** — oltp is measured at 1, 4 and 16 writers and the verdict is taken at the best
  `txns_per_fsync`, with that figure printed beside it. *(done / not done: why)*
- **T-15.3** — `run6.sh` on C publishes E16 and E19 under the new headers (`--publish` **only on
  C**), and the author pushes the result. **The author must run this now** is marked in §6. *(done /
  not done: why)*

*Guard and reversion:* `results_manifest`/T-11.1 stay green after publish; the E16 provenance test
(`the_e16_document_renders_the_provenance_block_under_its_title`) is the guard on the header.

### T-G1 — GBS: G3 states the as-of counter — closes F-60

*Files (gbs):* `crates/gbs-nilestream/src/g3.rs`; `results/G3-verdict.md`; the G3 renderer.

*Target:*
- **T-G1.1** — every G3 verdict row carries `as_of_reconstructions`, the sweep asserts it is 0 at
  the end of every run, and the verdict document's "should read zero" sentence is replaced by the
  column. *(done / not done: why)*

*Guard and reversion:* in a disposable worktree, re-introduce the old fresher-entry rule in a
scratch `Rev::read` (or force `answered.anchor = anchor + 1` in the test hook) and show the
assertion fires.

### ——— cut line — one agent, one cycle ———

Below the line, carried with fresh baselines; none is specified further here:

- **T-07** — determinism fixture across x86_64/arm64 for both SHA-256 streams (F-46). Needs C.
- **T-08** — fold plateau and the unstable p99 at 16 connections on C (F-28). Needs T-12 first.
- **T-09** — E24 RSS under load on C (20k / 200k / 2M).
- **T-10** — GBS lifecycles and holds on the ledger (F-12), sharing T-01's encoding discipline.
  **No GBS efficiency task before T-10 and a product trace (F-24).**

---

## §6 Branch stacks and merge order

Stacks start from `8bbbae1` (niles — `9ae1319` plus the brief, docs only) and `febb738` (gbs).
Branch names carry the cycle: `c8/…`. One task per branch, each stacked on the previous so a
single `--ff-only` merge lands the cycle.

```
niles   8bbbae1 ─ c8/00-audit (this file, run6.sh, LC ledger)        ← auditor, now
                 └ c8/01-window-budget    (T-05)
                    └ c8/02-say-what-is-true (T-06)
                       └ c8/03-csv-headers    (T-11)
                          └ c8/04-view-histogram (T-12)
                             └ c8/05-nilesc-time  (T-13)
                                └ c8/06-recovery-tests (T-14)
                                   └ c8/07-ab-script     (T-15)
gbs     febb738 ─ c8/00-g3-asof            (T-G1)
```

Merge order is stack order. `c7/01-durable-rows` and `c7/00-adapter` are the parents; the author
merges each `c8/*` with `--ff-only` and pushes; nothing is rebased once bundled.

---

## §7 Validation protocol

Container, every task, before its commit (`RUSTUP_TOOLCHAIN=stable` on each line):

```
cargo fmt --all -- --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline --workspace                 # PostgreSQL 16 up: see §0 D
make reproduce                                    # exit 0, clean `git diff -- results/`
make fsync-proof
```

Green means: every row exits 0 and `make reproduce` leaves `results/` unchanged. A red row is a
result: record it with its output in the report; it is never fixed by deleting the test, the row,
or the manifest entry. T-11.1 is *expected* red on the first run and its red transcript is part of
the deliverable.

Guards: for every task, in a disposable worktree
(`git worktree add -d "$S/wt-<task>" HEAD~1` or a `git revert -n` of the change), run the guard
test and paste the failing assertion into the report. A guard that passes on the reverted change
is not a guard, and the task is `not done`.

Author's gate on the Mac after each bundle: `cargo +1.97.1 clippy --all-targets -- -D warnings` in
both trees. A lint the container's 1.95.0 cannot see is fixed by the executor from the author's
pasted output — never guessed (cycle 7's lesson).

---

## §8 Host C scripts

One script this cycle, `docs/audit/cycle-8/run6.sh` (copied by the author to
`~/Documents/niles-hostc/run6.sh`). It supersedes `run5.sh`'s reuse pattern (F-51).

**Retarget-and-refuse clause (mandatory in every Host C script from now on):** each arm's worktree
is created `--detach` at an explicit SHA; on every run the script `git -C <wt> checkout --detach
<sha>` and then compares `git -C <wt> rev-parse HEAD` with the SHA it was told; on mismatch it
prints `REFUSING: <wt> is at <have>, wanted <want>` and exits 3 before building anything. It also
refuses if `git -C <wt> status --porcelain` is non-empty, except for the four named untracked files
in the main tree (never in a worktree).

Arms: **A** = the head of `c8/07-ab-script` (or whichever `c8/*` the author has merged when running);
**B** = `8d229ea` (run 4's arm A, the last C measurement) — the `--baseline` argument. PostgreSQL
16 on C under its default `wal_sync_method` (recorded in `device.csv`).

Sections and expected runtime on C (ten cores; ~25 minutes total):

| section | what | when the author must run it |
|---|---|---|
| A | preflight for C (barrier, cores, toolchain) | at the start |
| B | E16 same-session A/B, 2 warm-ups + 5 runs interleaved, oltp at 1/4/16 writers, `--publish` (T-15.3) | **after T-15 is bundled and merged** |
| C | E19 scaling 1/2/4/8/12/16, `--publish` — republishes `point.csv` under the T-02 header (T-11.2) | same run |
| D | mixed 4r2w / 8r1w / 8r4w with the per-read breakdown (T-12.3) | same run; needs T-12 merged |
| E | gate at 1.97.1 in both trees | same run |

**The author must run this now** markers, in order, as they will appear in the executor's report:

1. After `c8/00-audit` is bundled — fetch/merge/push (commands in §9). No script.
2. After `c8/04-view-histogram` is bundled — fetch/merge/push; **optionally** run
   `run6.sh --only D` to give T-12.3 its data early (≈ 4 min).
3. After `c8/07-ab-script` is bundled — fetch/merge/push, then **`bash
   ~/Documents/niles-hostc/run6.sh`** (≈ 25 min), then bundle nothing: the script commits
   `results/` on C under `c8/08-hostc-run6`; the author pushes that branch and pastes the tail
   of `run6.log` into the session.
4. After T-G1 is bundled — fetch/merge/push in `GBS`.

---

## §9 The open-questions ledger

Settled, not reopened: LC-01, 02, 04, 08, 09, 10, 11, 12, 14, 15, 21.

**Closed this cycle, with evidence:**
- **LC-17** — closed. The anchor mismatch branch is unreachable in both consumers (T-02), and on C
  under concurrency `fallbacks = 0` at 4r2w, 8r1w and 8r4w over 1.34–1.49M view answers (F-62).
- **LC-18** — closed. No writer starvation at 8:1 on C: write p50 3,932 µs at 8r1w against
  3,991 µs writers-alone; the RwLock stays. The tail is LC-23, a separate question.

**Carried, restated:**
- **LC-03** — `AccountId` String→u64: one term of bytes-per-row; measure after T-05 (the key
  packing there is the same decision at the view).
- **LC-05** — rendered-NULL and refusal boundaries: unchanged; `mixed_table_refusals` renders a
  refusal as a row, which is the rule.
- **LC-06** — allocator bytes per row are the figure; RSS is context. T-05's probe follows it.
- **LC-07** — E23's slope: unchanged; re-fit after `run6.sh`.
- **LC-13** — release checks: unchanged.
- **LC-16** — durable by default: **now decidable** — T-01 landed, the daemon refuses volatile
  fallback. Proposal to the author: default `--durable <path>` required, `--volatile` explicit, in
  cycle 9. Not an executor decision.
- **LC-19** — the wire claim across the reader-writer split: within a process preserved; across a
  restart **now holds** (T-01: verified replay); on the failure path **now holds** (LC-21 fail-stop,
  F-39/F-40 closed). Proposal: fold the three clauses into one sentence in ch6 in T-06.
- **LC-20** — not decided; still a thesis sentence, not an engineering task.

**New, raised in cycle 7, here with the audit's position — the author decides:**
- **LC-22** — declared scale per money column on the wire: yes in principle (self-describing
  amounts), but it changes `psql_conformance`'s golden transcript; defer to cycle 9 with the
  transcript diff shown first.
- **LC-23** — the 12–13 ms tail: **attribute before deciding** (T-12); the audit's hypothesis is
  scheduler-behind-barrier, not the base guard (§1 Q1, §2.4).
- **LC-24** — fold p99 instability at 16 on C: below the line (T-08), after T-12.
- **LC-25** — concurrent crash test: T-14.2 tests concurrent `submit_pending` in-process instead,
  at unit-test cost; the crash protocol stays serial. Position: no.
- **LC-26** — the A/B script exists as `run6.sh` (§8); T-15 makes `bench` itself two-armed.
- **LC-27** — figures invalidated by the T-02 counter change: E16's committed `miss_rate` survives
  (single connection, zero fallbacks); **E19's committed `point.csv` does not** (old header, F-52)
  and is marked in T-11.2 until `run6.sh` republishes; every `reads/hits/misses` in the thesis
  drawn from E19 is cited as "pre-T-02 counting" until then.

**New this cycle:**
- **LC-28** — the idempotency window's clock: the epoch has no wall time in the durable record; a
  30-day window needs one. Author decides whether the record carries a timestamp (one more field
  under the hash) or the window is counted in epochs. `BLOCKED-idem-clock` in T-05 until decided.
- **LC-29** — oltp's verdict connection count (F-54): the contract sentence in SPEC-ENGINE Part 0
  should say at what concurrency 5–10× is claimed.

---

## §10 Reporting requirements for the executing agent

The report is `docs/audit/cycle-8/execution-report.md` (committed on the last branch) and a copy
in the project as `claude/cycle-8-execution-report.md`. It contains, in this order:

1. **The checklist, every target line verbatim, with its status** — copy the block below and
   replace each `[ ]`:

```
[ ] T-05.1 — after committing 2× budget distinct keys, reads_of.len() ≤ budget and last_read.len() ≤ budget and the slot map holds ≤ budget non-⊥ entries: metadata is Θ(budget).
[ ] T-05.2 — metadata bytes per resident key ≤ 120 (from ~340), by packing the (acct, cur) key and evicting the two maps with the entry.
[ ] T-05.3 — idem: IdemKey window 30.days reaches the sequencer: a key whose epoch is older than the window is admitted as new, and seen (both copies, or one shared copy) holds only keys inside the window; nilestream_stats gains idem_window_keys and view_metadata_keys.
[ ] T-05.4 — the sealer's BTreeMap and the ledger's HashSet are one structure or the report says why two are needed.
[ ] T-06.1 — preflight.sh reports the pin it finds, the test count it counts, and the expected heads of this cycle; a source test asserts the script contains no literal test count.
[ ] T-06.2 — ch3's lattice reads Present(v, [e_s, e_f]) (or states in one sentence that a Present entry's certification is an interval and names §4's Theorem 4.1 step (3)); the MISMATCH-lattice-interval marker is placed and resolved in the same commit.
[ ] T-06.3 — §9.14.1 says the committed E16 document predates the header and names the commit (5d37692) from which every generated one carries it; MISMATCH-e16-header stays until run6.sh publishes.
[ ] T-06.4 — SPEC-ENGINE.md:705 names RwLock<RevEngine> and the harness adapter; lockstats.rs header and the four comments say what the code does; a source test asserts the banner names no mutex.
[ ] T-11.1 — a test reads every results/**/*.csv named in MANIFEST.csv and asserts its first line equals the *_CSV_HEADER constant for its kind; it is red at 8bbbae1 for E19-scaling/point.csv and the report shows that red.
[ ] T-11.2 — the machine-dependent E19 point.csv is not hand-edited: it is either re-published by run6.sh on C or its rows are marked not_run with the reason "header predates T-02; re-measure on C".
[ ] T-11.3 — the E19 document's core sentence is rendered from Provenance (host + granted cores read at run time), never a literal.
[ ] T-12.1 — the V mutex is wrapped in the same TimedWrite-style histogram as B, with buckets to 32 ms, exported as view_wait_* / view_hold_* in nilestream_stats, cost ≤ 2% on readers-alone reads/s in a within-container A/B (5 runs interleaved, median).
[ ] T-12.2 — the mixed report prints, for the slowest 16 reads of the phase, a per-read breakdown: B wait, V wait, compute, wire.
[ ] T-12.3 — run6.sh reproduces run 4's three shapes with the breakdown; the report states which of {B, V, scheduler/barrier, wire} the ≥ 10 ms reads sit in. (needs the author)
[ ] T-13.1 — nilesc time FILE prints, per item, lexing, parsing, resolving, typing, lowering and verifying in instructions under callgrind and in µs otherwise, one line per item, one summary line.
[ ] T-13.2 — e26 runs it over examples/*.niles and the E14 defect corpus and writes a byte-deterministic instruction-count CSV (class: toolchain-scoped) with a per-class row; §9 cites it and H-S6's DV "compile time" points at it.
[ ] T-13.3 — the IR verifier runs once per plan-cache fill and never on a cached statement — a test drives 1,000 distinct statements and 1,000 repeats and asserts verifier invocations = 1,000.
[ ] T-13.4 — one bootstrap gate (Appendix E / E15) is mutated in a disposable worktree and shown red; the report names the gate and the mutation.
[ ] T-14.1 — Sequencer::open_recovered + recover_txns are tested in-process: seal N transactions, drop, reopen, assert the window and the (idem_key, payload) list are identical and in order.
[ ] T-14.2 — submit_pending is tested with 8 threads submitting concurrently: every reply arrives, epochs are a permutation of 0..N, max_batch > 1 at least once.
[ ] T-14.3 — RevEngine::open_recovered over a forged record refuses (cite chain_verification_tests) and over a truncated last record refuses with a message naming the record index.
[ ] T-15.1 — bench --run --baseline <sha> --baseline-bin <path> runs both arms in one process, interleaved, ≥ 5 measured runs after 2 warm-ups, and the E16 contract table has an A and a B column with the ratio between them.
[ ] T-15.2 — oltp is measured at 1, 4 and 16 writers and the verdict is taken at the best txns_per_fsync, with that figure printed beside it.
[ ] T-15.3 — run6.sh on C publishes E16 and E19 under the new headers (--publish only on C), and the author pushes the result.
[ ] T-G1.1 — every G3 verdict row carries as_of_reconstructions, the sweep asserts it is 0 at the end of every run, and the verdict document's "should read zero" sentence is replaced by the column.
```

2. **Guard transcripts** — for every task, the disposable worktree's path, the reverted change
   (SHA or `git revert -n` line), and the failing assertion, verbatim.
3. **Worktree status** at the end: `git status --short` of both trees, with the four untracked
   files (`.DS_Store`, `AGENTS.md`, `thesis/.DS_Store`, `thesis/Niles-Thesis.pdf` — Mac only)
   listed separately from task changes; the container's trees must be clean.
4. **The SHAs** — every commit on every `c8/*` branch, in order, with its one-line message.
5. **Exactly three material facts** found while executing that this work order did not cover;
   negative findings count. Not two, not four.
6. **The gate table** (§7) per task, including the T-11.1 red.
7. **The sync section** — every bundle written to `~/Documents/niles-sync/cycle-8/`, its branch and
   head SHA, and the author's commands for each:

```
cd ~/Documents/niles      # or ~/Documents/GBS
git fetch ~/Documents/niles-sync/cycle-8/<bundle> c8/<branch>
git merge --ff-only FETCH_HEAD && git push origin c8/<branch>
```

   plus, where an author-side uncommitted edit is contained byte-for-byte in the incoming commit,
   the `git checkout -- <file>` line and the statement that it is.

8. **What was not done and why**, one line per unreached task, in the words of its target line.
