# Work Order 7 — Fable's audit

**Auditor:** Fable (Claude Fable 5.1), cycle 7. **Executor:** Claude Opus, a later session.
**Audited:** Niles `2b52912` on `c6/audit-cycle-7` (engine identical to `d9c8699`), GBS `e803b7d`
on `c6/07-hold-index`. **Hosts:** A (this 2-core container) and C (the author's Apple M4, ten
cores) — see §0. Nothing was written into either repository; every probe lives outside the trees.
**Attribution for the executor:** taken from the executing session's own instructions; if none,
record "none required".

**Reconciliation note.** GPT 6 Astra is auditing the same commits from a structural brief. Where
this order and Astra's disagree, the evidence class is stated beside every number here so the
disagreement can be resolved rather than averaged.

---

## 0. Preflight and admissibility

### Host A — this container

```
uname            : Linux 6.18.44-fc-v24 x86_64 GNU/Linux
cores (nproc)    : 2            cgroup cpu.max: n/a (no cgroup v2 quota file; 2 is the grant)
memtotal         : 7.8 GiB
mount            : /dev/vda ext4 rw,relatime  — NOT an overlay, NOT volatile
barrier          : fdatasync, median 135.7 µs -> 7,371 barriers/s, spread 132–853 µs
                   VERDICT: plausible as a real barrier for THIS container; within-host only
rustc/cargo      : 1.95.0 (== the pin)   clippy/rustfmt present   valgrind 3.22   strace present
PostgreSQL       : 16.13, wal_sync_method=fdatasync, fsync=on, synchronous_commit=on
egress           : github 400 (proxy), crates.io 403, static.rust-lang.org FAIL — none needed
CARGO_TARGET_DIR : unset
trees            : niles 2b52912 c6/audit-cycle-7 (clean); gbs e803b7d c6/07-hold-index (clean)
```

Uptime at audit start was 15 minutes: **this is a fresh instance of the A class, not the instance
cycle 6 measured on.** That matters for §2 F-29.

| admissibility (A) | |
|---|---|
| cores actually granted | 2 |
| barrier | `fdatasync`, 7,371/s median (bench's own probe in the same session: 7,178/s) |
| storage evidence? | yes, for this container |
| may publish durability rows? | as within-host ratios only |
| may publish >3-core curves? | **no** |
| toolchain newer than 1.95.0? | no |
| valgrind attribution possible | yes |
| PostgreSQL comparison possible | yes, both sides under `fdatasync` on the same ext4 |

### Host C — the author's Mac, via `run4.sh` (run by the author in Terminal.app)

```
Darwin 25.6.0 arm64, Apple M4, 10 cores (4 performance + 6 efficiency), 16 GiB, macOS 26.6.2
rustc 1.97.1 / cargo 1.97.1   <-- NEWER than the 1.95.0 pin; see F-33
APFS, /dev/disk3s5, 27 GiB free of 460
barrier: F_FULLFSYNC — the durable arm ran at 255–267 ops/s single-connection, which is the ceiling
niles working copy: wo4/T-32-binary-wire 4d9a457, four untracked files intact
  (.DS_Store 10,244 B; AGENTS.md 16,639 B; thesis/.DS_Store 8,196 B; thesis/Niles-Thesis.pdf 816,110 B)
worktrees added under ~/Documents/niles-hostc/: wt-after 2b52912, wt-before 15425b5, wt-t06 d681f0d
```

| admissibility (C) | |
|---|---|
| cores | 10; a CPU-bound path plateaus at the 4 performance cores |
| barrier | `F_FULLFSYNC`, ~255/s |
| may publish >3-core curves? | **yes — the only host that can** |
| PostgreSQL comparison | not run (no PostgreSQL arm in run4; would need `fsync_writethrough`) |

Host B (the desktop bridge's 4-core Linux VM over FUSE) was used only to read files back.

**Rules this table decides**, as in the brief: no absolute figure beside another host's without a
host column; counters gate on one run; wall clock on ≥3–5 runs with median/MAD; a gate fires on
≥10% **and** ≥3× the pooled MAD, else `noise-limited`.

---

## 1. Executive judgement

**The engine's concurrency story is now real on a wide host, and its durability story is not.**

Cycle 6's two largest changes were measured on two cores and could not be assessed. On ten cores
they are: the scan-shaped read went from **flat at 1.83× from two connections** — the mutex, exactly
as F-14 predicted — to **3.13× at four and 4.24× at eight**, plateauing at 4.4× on the four
performance cores; point reads went from a 95,000/s ceiling to **140,000/s**; and the lock-order fix
that followed costs nothing measurable. T-06's Host C target at four connections is **met**, at eight
it is **missed by 6%** against a ceiling the hardware sets. The chunked-storage trigger does not fire
on either arm on either host, so that work is **closed as measured-not-worth-doing on two hosts**,
and LC-14 stays retired.

Two findings outrank all of that, and both were invisible to every test and every published row.

**F-26 — the daemon does not recover its rows.** `nilestreamd --durable` fsyncs a record that
carries the idempotency key and the epoch *number*. Not the rows. After a SIGKILL and a restart on the
same segment, 25,416 acknowledged inserts were gone, the frontier was back at the seed, every
writer's account had no balance, and a retry of an acknowledged transaction was accepted as new while
the sequencer counted it as a duplicate. Every `durable` row in E16 and E19 — including cycle 6's
5.73× and this cycle's 7.80× — measures the cost of a barrier on a record that cannot reconstruct
the ledger. **Durable-before-visible holds within one process life and is void across a restart.**
The segment layer underneath (`nilestream-ledger`) recovers payloads correctly and is tested for it;
the server writes it a stub. This is the most important thing in this order and it is T-01.

**F-27 — under concurrent writes, 87.4% of keyed reads bypass the maintained view.** Measured
exactly, with a scratch counter, on A: readers alone 0.0%, readers with two writers 87.4%. The cause
is structural and follows from T-05: an epoch is *applied* to the view before it is *visible*, so
whenever any barrier is in flight the view's effective version exceeds the reader's anchor and
`answer_from_view` discards an answer that is, in fact, exact. The thesis's mechanism is switched off
by the workload it exists for. The cost is masked at the benchmark's base size because the fallback
scan is anchor-indexed and the seeded history is two postings deep per account; on E23's base axis it
is 100× deeper. The fix is a one-condition change in `Rev::read`, the counter that work order 6
specified twice, and a mixed-workload row in E19 so the number can never hide again.

**The contract table is instance-fragile (F-29).** Re-measured on this instance, `report` reads
**2.17× NOT MET** against a committed **2.68× MET** — and so does the engine *before* T-05 on the
same instance (2.30×). The change did not regress it; the A class did. A contract row that flips on
two instances of one host class is not a result, and the arithmetic for every efficiency goal
inherits that fragility.

**On the goals.** Nilestream's speed under concurrency is now bounded by the hardware and by F-27,
not by a lock. Its memory against base size is sub-linear at 173 B per row at two million rows —
T-15's trigger does not fire. Its durability is not what the tables say. Niles-the-language was not
measured by this audit; Astra owns that definition. **GBS efficiency is not admissible yet**: with no
product trace (F-24) and lifecycles that do not survive a restart (F-12), the only honest GBS task in
this order is the one that makes measurement possible.

---

## 2. Findings

EV = impact × confidence ÷ cost, each 1–5; ties favour correctness. **HI** host-independent, **HS**
host-shaped.

### F-26 — The durable daemon loses every appended row on restart — EV 25 (5×5÷1) — correctness, HI

**Evidence.** `crates/nilestream-server/src/rev_engine.rs:615` — the payload handed to the sequencer
is `epoch.to_string().into_bytes()`; `:217` the same on the test path. `with_durable` (`:690–695`)
opens the sequencer — which recovers the segment's records for the idempotency window and the
frontier — and sets `visible = base.head()`, the **seeded** head. No code path replays a segment
into the in-memory `Ledger`. The segment layer's own recovery (`nilestream-ledger/src/segment.rs:345`,
twelve tests) is correct and unused for rows.

**Measured** (A, `probes/crash`, 8 writers, each posting `(id, acct, 0, 1), (id, 9999, 0, -1)` to its
own account, SIGKILL at t=2.0 s, restart on the same 1.3 MB segment):

| | before kill | after restart |
|---|--:|--:|
| acknowledged inserts (8 threads) | **25,416** | — |
| `select nilestream_frontier` | 2999 + 25,416 | **2999** |
| `sum(amt)` for each writer account | ≥ acked | **no row** (absent, not zero) |
| retry of an acknowledged txn id | — | **`INSERT 0 2`** — accepted as new |
| sealer `duplicates_absorbed` after that retry | — | 1 — the sequencer *knew* |

`interpret` maps `Rejected::Duplicate { at_epoch }` to `Ok(at_epoch)`, so the sink's "I have seen
this key" becomes the daemon's "committed" while the in-memory ledger, whose `seen` set is fresh,
takes the rows a second time. After a restart the two clocks — the sequencer's recovered epoch count
and the ledger's re-seeded one — also diverge, so `Pending::wait`'s `fetch_max` publishes numbers
from one against a frontier from the other.

**Cost.** Every `durable` row in `results/E16-wallclock.md` and `results/E19-scaling/`, cycle 6's
T-05 numbers, and this cycle's Host C durable curve measure a barrier on a stub. The daemon's banner
says "appends are durable"; `Serving::durability()` returns `"always"`, which is what `bench` records
as `durable=true`. Part 1 of cycle 6's report listed durable-before-visible as "preserved and
tested" — true within a process, and the tests only ever ran within a process.

**Repair.** The record's payload must carry the rows (a canonical encoding of the `Vec<Row>` — the
ledger crate already has framing for keyed batches); `with_durable` must replay the segment's
payloads through `Ledger::submit` in epoch order on open, rebuilding indexes and the hash chain
deterministically, and set `visible` to the recovered head; the sequencer's epoch and the ledger's
must be one number or the relationship must be stated and checked; `interpret` must not turn a
sequencer duplicate into a commit the ledger has not seen. **Gives up:** a larger record per epoch —
measure what it does to txns-per-barrier — and a slower open. Nothing else.

**Negative check the executor must make:** whether the thesis's durability chapter describes the
daemon or the ledger crate. If the former, `MISMATCH-durability-restart`.

### F-27 — 87.4% of keyed reads fall back under concurrent writes, structurally — EV 12.5 (5×5÷2) — correctness/efficiency, HI

**Evidence.** A scratch counter placed at `rev_engine.rs` `answer_from_view`'s
`answered.anchor != anchor` branch, in a throwaway worktree, driven by `probes/mixed` (4 readers on a
0.9-skewed key set, 2 writers, 10,000 accounts, budget 2,500, durable):

| phase | keyed reads | answered from view | fell back to scan | fallback rate |
|---|--:|--:|--:|--:|
| readers alone | 149,452 | 149,452 | 0 | **0.0%** |
| mixed | 57,080 | 7,211 | 49,869 | **87.4%** |

**Mechanism.** `Rev::read` (`nilestream-core/src/rev.rs:193–233`) returns a hit's *effective*
version `max(stamp, applied)`. Under T-05, `append` calls `advance` before the barrier returns, so
`applied` runs ahead of the visible frontier whenever a barrier is in flight — which, at 1,600
writes/s on A or a 3.9 ms barrier on C, is nearly always. A reader's anchor is the visible frontier.
So `effective > anchor` for **every resident key**, not only the written ones, and the engine's
strict equality check discards the answer. Yet a resident, unpinned entry with `stamp ≤ anchor ≤
applied` has received **no delta in `(anchor, applied]`** — `apply_epoch` restamps on every delta
(`:311`) — so its value **is** the value at `anchor`, exactly. The engine throws away a correct answer
and re-derives it from the base.

**Why nobody saw it.** The `hits` counter counts these as hits (the view *did* hit before the engine
discarded it), so `miss_rate` reads optimistically under concurrency. The latency signal is muted
because the fallback goes through `scan(restricted = Some(acct))` — the anchor index — and the
seeded history is two postings per account. On Host C, with a 3.9 ms barrier keeping `applied >
visible` essentially continuously, read p50 in the mixed phase was **unchanged** (24–67 µs): the
fallback costs what a hit costs at this base size. On E23's base axis it costs 100× more, and that is
where this will surface as a slope nobody predicted.

**Repair.** (a) In `Rev::read`, when `Slot::Present(v, stamp)`, not pinned, and `stamp ≤ anchor ≤
effective`, return `Anchored { value: v, anchor }` — the requested anchor — because the value is
exact there; keep the reconstruct path for `stamp > anchor` (a delta landed after the anchor) and for
`anchor > effective`. This is the reconstruction theorem's own clause, not a relaxation. (b) The
anchor-mismatch counter on the stats surface, as work order 6 specified in T-02 and required in the
T-05 report — its absence twice is itself a finding (F-32). (c) A `mixed` row in E19 (§4, T-03).
**Gives up:** nothing; a test that a key *with* a delta in `(anchor, applied]` still reconstructs is
the guard.

**Corroboration requested from Astra:** whether the algebra's statement of the reconstruction
theorem licenses (a) as written, and LC-17's other options in light of it.

### F-28 — T-06 on ten cores: the fold scales, the target at eight is missed by 6% against a hardware ceiling — instrument result, HS (C)

`run4.sh`, arm A (`2b52912`) against arm B (`15425b5`, one mutex, fold workload patched into its
harness in the scratch worktree), medians of three runs, MAD in the raw CSVs:

| fold, ops/s | 1 | 2 | 4 | 8 | 12 | 16 |
|---|--:|--:|--:|--:|--:|--:|
| before (mutex) | 465 | 843 (1.81×) | 850 (1.83×) | 853 (1.83×) | 856 (1.84×) | 852 (1.83×) |
| after (T-06+06a) | 454 | 858 (1.89×) | **1,421 (3.13×)** | **1,924 (4.24×)** | 2,000 (4.40×) | 2,006 (4.42×) |
| fold p50, µs | 2,174 | 2,272 | 2,675 | **3,965 (1.82×)** | 5,175 | 5,452 |
| before p50 | 2,148 | 2,329 | 4,663 | 9,315 (4.34×) | 13,975 | 18,695 |

| point, ops/s | 1 | 2 | 4 | 8 | 12 | 16 |
|---|--:|--:|--:|--:|--:|--:|
| before | 32,252 | 58,012 | 81,458 (2.53×) | 96,061 (2.98×) | 94,946 | 92,964 |
| after | 29,803 | 55,079 | 94,369 (3.17×) | **125,092 (4.20×)** | **139,601 (4.68×)** | 136,300 |

**Against T-06's targets:** report/fold ≥3.0× at 4 — **met** (3.13×); ≥4.5× at 8 — **missed**
(4.24×; the curve plateaus at 4.4× at 12–16, which is the four performance cores); p50 at 8 ≤3× the
1-connection p50 — **met** (1.82×); point curve not regressed — **exceeded** (+30% at 8, +47% at 12).
The mutex arm is flat at 1.83× from two connections on ten cores, which is F-14's "1/(fold time)"
prediction made exact. **F-23 ("point reads are not lock-bound") is refuted on the host it was
measured on:** the mutex capped point reads at ~95k/s; without it they reach ~140k/s.

**The chunked-storage trigger, both arms, both hosts:** arm 1 (lock wait ≥20% of counted request
time) — A: wait p99 ≤1 µs against hold p50 256 µs; C: wait p99 ≤1–16 µs against hold p99 128 µs —
under 1% on both. Arm 2 (RwLock arm <1.5× baseline at 8) — C: 1,924 / 853 = **2.26×**. Neither fires.
**Closed as measured-not-worth-doing on two hosts.** LC-14 stays retired.

### F-30 — The lock-order fix is complete and costs nothing measurable; the order holds across all three crates — liveness, HI

**Enumeration.** Every lock in `nilestream-server`, `nilestream-ledger`, `nilestream-core`:

| | lock | crate | taken by |
|---|---|---|---|
| **O** | `RwLock<RevEngine>` (harness adapter only) | server | `Serving for RwLock<RevEngine>`: read per call; `reseed`: write |
| **B** | `RevEngine::ledger: RwLock<Ledger>` | server | `base()` shared on every read path; `append` exclusive |
| **P** | `RevEngine::pending: Mutex<Vec<Pending>>` | server | `append` (under B), `take_pending` (alone) |
| **V** | `RevEngine::runtime: Mutex<Runtime>` | server | `append` (under B, P); `answer_from_view` (under B); `report_shape`, `report_from_view`, `read_stats` (alone) |
| **C** | `RevEngine::currencies: RwLock<BTreeSet<u32>>` | server | `append` write (under B, P, V); `currency_count`/`sole_currency` read, released before V |
| **S** | `Sequencer::stats: Mutex<SequencerStats>` | ledger | the sealer thread alone; `sealer_stats()` with nothing held |
| — | `Frontier::{visible, sealed}` | ledger | atomics |
| — | none | core | `Runtime`/`Rev` are plain structs; the server wraps them in V |

Per-function acquisition sequences (from the source, every site): `append` B→P→V→C;
`answer_from_view` B→V; `report_from_view` V; `read_stats` V; everything else one lock. The channel
between `append` and the sealer is unbounded, so `append` never blocks on it under B. **Total order:
O < B < P < V < C, with S a leaf. No inversion remains.**

**The fix.** One base guard held across `answer_from_view`; the later second `self.base()` was
removed (an `RwLock` read behind a queued writer would have self-deadlocked). Verified by re-running
the three guards on the reverted order in a scratch worktree: the source guard fails, the
exactly-once guard fails, and the deadline test **hangs with 2 of 5 threads finishing** — the defect
was real, not theoretical. **Cost on C, arm A vs arm C (`d681f0d`, T-06 without the fix):** fold
1,421/1,389 at 4, 1,924/1,982 at 8, 2,000/1,875 at 12; point 125k/128k at 8, 139.6k/141.4k at 12 —
all inside MAD: **noise-limited, no cost.**

**One weakness, for the executor.** `the_base_is_acquired_before_the_view_on_every_path_that_takes_both`
finds the view by the first `.lock()` in the function body; a future `pending.lock()` placed earlier
would be mistaken for it. Search for `runtime` followed by `.lock()` instead, or name the lock.

### F-29 — The contract table flips MET/NOT MET between two instances of one host class — EV 8.3 (5×5÷3) — wrong-measurement, HS

E16 re-measured on this A instance, five runs, PostgreSQL 16.13 on 5432 under `fdatasync`:

| row | committed (cycle 5, an A instance) | before T-05 (`05eeef7`) here | after T-05 (`15425b5`) here | after T-06a here |
|---|--:|--:|--:|--:|
| oltp | 1.07× NOT MET | 1.02× | 1.15× | 1.10× |
| analytical | 1.93× NOT MET | 2.04× | 2.13× | 2.04× |
| point p99 | 1.29× PARITY | — | — | 1.39× PARITY |
| durable | 0.84× PARITY | — | — | 0.96× PARITY |
| **report** | **2.68× MET** | **2.30× NOT MET** | 2.12× | **2.17× NOT MET** |

PostgreSQL itself runs 35% slower on this instance (oltp 4,519 → 2,706; report 149 → 98). The three
arms on this instance are within 8% of each other with MADs ≤4% — **the changes did not move the
report row; the instance did.** A 2.6× threshold sitting inside cross-instance variance of the same
host class is not a contract; it is a coin. **Repair:** the contract must be stated as a same-session
A/B against a pinned baseline commit on the executing host, never as an absolute ratio inherited from
another instance; and the reference host should be C, the only stable one the project has. **Gives
up:** the ability to quote "2.68× MET" without a host and a session beside it, which was never
legitimate.

### F-31 — Memory against base size is sub-linear; T-15's trigger does not fire — negative, HS (A, allocator-level)

`probes/mem`, RSS from `/proc/self/status` around `RevEngine::seeded`, budget 2,500:

| base rows | ΔRSS | bytes per base row |
|--:|--:|--:|
| 19,998 | 5.2 MiB | 266 |
| 199,998 | 39.8 MiB | 209 |
| 1,999,998 | 330 MiB | **173** |

A posting is ~40 B packed; 173 B/row is a 4.3× index-and-chain overhead that **falls** with size.
T-15's trigger (bytes/row at 2M > 1.5× the 20k figure) reads 0.65×. **Close T-15's layout work
without code.** Not measured: the idempotency window's share of that, and RSS under load at these
sizes on C — both stay open (§5).

### F-32 — A specified deliverable was dropped twice and nothing surfaced it — process, HI

Work order 6 required the anchor-mismatch counter in T-02 and its rate in the T-05 report. Neither
shipped; the cycle-6 report did not say so; F-27 is what it would have shown. The executor's report
format needs a **requirements checklist** copied verbatim from the order with a status per line —
`done` / `not done: <why>` — so a dropped line cannot be silent.

### F-33 — The gate is red on the Mac's 1.97.1 toolchain, for two lints the code never had a chance to see — EV 10 (2×5÷1) — instrument, HI

`run5.sh` §D: `cargo clippy --all-targets -- -D warnings` under rustc 1.97.1 fails in `niles-ir` on
two lints that 1.95.0 does not emit — *"you seem to want to iterate on a map's values"* and *"this
pattern is unneeded as the `..` pattern can match that element"*. All three arms **built** cleanly;
only the lint gate is red. `rust-toolchain.toml` pins `channel = "stable"` and says in its own
comment that the intended value is `1.95.0`, unset only because the build machine had no egress to
name a version. **The gate is therefore a function of which day you run it on.** Repair: the author
sets `channel = "1.95.0"` (one line, needs network once — the executor cannot), *and* the executor
fixes the two lints so the code is clean on both, because a pin that hides a lint is a pin that
hides the next real one. **Gives up:** nothing.

### F-37 — T-03 is confirmed on Host C, and the reversion proved itself — closes the §2.2 verification, HI

`run5.sh`: GBS `e803b7d` (which includes T-03) runs `niles_schema` **16/16 green with and without
`CARGO_TARGET_DIR`** at default parallelism on the Mac — the exact condition that failed in cycle 5.
The pre-T-03 working copy (`572fe97`) under the same variable fails **3 of 15** — and `run4.sh` had
it failing a *different* **2 of 15** twenty minutes earlier. That nondeterminism is the cycle-5
defect reproduced on demand, and it is the natural reversion proof for T-03 without touching a
tracked file. (`run4.sh`'s T-03 line tested the working copy by my mistake; `run5.sh` corrected it.)

### F-34 — LC-18, measured: no writer starvation at 8 readers on ten cores — negative, HS (C)

`probes/mixed`, 8 readers / 1 writer on C: writer alone 258/s, p50 3,977 µs, p99 4,985 µs; with 8
readers 271/s, p50 3,975 µs, p99 7,889 µs, max 17.8 ms. Throughput and p50 unchanged; p99 rose to two
barriers. Reads fell 5% (93,741 → 89,482). `std::sync::RwLock` on this platform does not starve the
appender at this ratio. **LC-18 can close as "not observed at ≤8:1 on macOS; re-check on Linux at
16:1 when the mixed row exists."**

### F-35 — Crash recovery under group commit: the segment survives, the daemon does not use it — instrument, HI

The SIGKILL in F-26 landed mid-batch (`max_batch` was 4–15 at the time). The segment recovered
cleanly — the sequencer reported its idempotency keys back — so `nilestream-ledger`'s recovery under
group commit **works**. The finding is F-26's, not the segment's.

### F-36 — Negative: connection memory under load on ten cores is small — HS (C)

Bench process RSS through arm A (10,000 accounts, 2,500 budget, up to 16 client threads in the same
process): p50 74 MiB, max 81 MiB. Confirms F-22 on C. Do not spend the cycle here.

---

## 3. Guarantees, checked

| commitment | held? |
|---|---|
| durable-before-visible | **within a process life: yes (T-05 tests). Across a restart: NO (F-26).** |
| one sealer per ledger | yes |
| one total epoch order | in memory yes; **after a restart the sequencer's and the ledger's numbering diverge (F-26)** |
| full retention | **no, across a restart (F-26)** |
| honest absence | yes — the post-restart query returned *no row*, not zero |
| fold-never-field, product purity, self-describing amounts, GBS layering | not exercised by this audit |
| no `unsafe`, zero external dependencies, no default-on-error | yes (grep; `cargo test --offline` on a bare toolchain) |
| apply-before-publish | yes — and it is the cause of F-27 |
| stated lock order | yes, extended to all three crates (F-30) |

---

## 4. Tasks

In dependency order. **Cut line after T-04.** Every task: prove the guard fails on the reverted
change in a disposable worktree, transcript in the report. Baselines are measured on the executing
host in the same session; targets are ratios against them.

### T-01 — Durable means the rows come back — closes F-26, F-35

*Files:* `nilestream-server/src/rev_engine.rs` (`append`, `with_durable`, `DurableSink`),
`nilestream-ledger` (a row-batch payload encoding, if none fits), the daemon banner, `SPEC-ENGINE.md`.
*Baseline:* `probes/crash` protocol — 8 writers, SIGKILL at 2 s, restart: frontier returns to seed,
0 of ~25k acknowledged inserts present.
*Target:* after restart, for every writer account `sum(amt) ≥ acked`, and `sum(amt) − acked ≤` the
largest batch in flight; frontier ≥ the last acknowledged epoch; a retry of an acknowledged id is
refused as `Duplicate` by **the daemon**; the sequencer's epoch and the ledger's are one number.
Txns-per-barrier at 16 connections on A within 10% of today's 6.02 (the record grows).
*Method:* encode `Vec<Row>` canonically into the payload; on `with_durable`, replay every recovered
payload through `Ledger::submit` in epoch order before serving; set `visible` to the recovered head;
make `interpret` return `Duplicate` as a refusal when the ledger has not seen the key.
*Acceptance:* the crash protocol as a `#[test]` (spawn the daemon binary, SIGKILL, restart, verify);
the existing visibility and recovery suites green; `make fsync-proof` green.
*Guard:* revert the payload to the epoch string → the restart test fails on the first account.
*Guardrails:* `SyncPolicy::Always` only; one sealer; no change to the wire; `MISMATCH-durability-restart`
if the thesis text describes the daemon as durable today.

### T-02 — The view answers exactly at the anchor when it can, and says how often it could not — closes F-27, F-32

*Files:* `nilestream-core/src/rev.rs` (`read`), `nilestream-server/src/rev_engine.rs`
(`answer_from_view`, `read_stats`), `session.rs` (`nilestream_stats`), the E19 renderer.
*Baseline:* the scratch counter's 87.4% under 4r/2w on A; 0.0% readers alone.
*Target:* fallback rate under 4r/2w on A **≤ 5%** (the residual is keys with a delta in
`(anchor, applied]`, which must still reconstruct); `hits` no longer counts a discarded hit; a
`fallbacks` column on `select nilestream_stats`; read p50 in the mixed phase within 1.2× of readers
alone on A.
*Method:* in `Rev::read`, for a resident unpinned `Present(v, stamp)` with `stamp ≤ anchor ≤
effective`, return `Anchored { v, anchor }`; count the remaining mismatches at the engine.
*Acceptance:* a `nilestream-core` test that a key **with** a delta in `(anchor, applied]` still
reconstructs (this is the guard's twin); `oracle_differential` under concurrent append/read at random
anchors; the deadline test still green.
*Guard:* revert the condition → the fallback-rate test (≤5%) fails at ~87%. Revert the counter → the
column is absent and the renderer refuses the row.
*Guardrails:* pinned (historical) entries keep reconstructing; no change to eviction; LC-17's other
options are not taken without a decision.

### T-03 — The mixed workload becomes a row nobody can omit — closes the §6.4 instrument gap

*Files:* `bank-bench/src/bin/bench.rs` (a `mixed` level in E19), `workloads.rs`, the renderer,
`results/MANIFEST.csv`.
*Baseline:* none — `probes/mixed` is the only mixed measurement in the project's history.
*Target:* E19 gains `mixed` rows at each connection level (readers = N, writers = N/2 rounded up),
reporting reads/s, read p50/p99, writes/s, write p99, **fallback rate** (from T-02's counter),
`max_batch`, and lock wait p99; `--nls-only` supports it; the renderer refuses a `mixed` row
without the fallback column. On A the row reproduces this audit's shape within MAD.
*Method:* lift `probes/mixed`'s three-phase design into `workloads::concurrent`'s framework;
`readers-alone` and `writers-alone` are the existing rows, so only the mixed phase is new.
*Guard:* the renderer's refusal, reverted → a row publishes without the fallback column.
*Guardrails:* never `--publish` from the executing container; the row is `NOT RUN` without T-02.

### T-04 — The contract is a same-session A/B, and C is the reference — closes F-29

*Files:* `docs/SPEC-ENGINE.md` Part 0, `docs/BENCHMARK.md`, `bench.rs` (`--baseline <commit>`
or a documented two-arm invocation), `results/MANIFEST.csv`.
*Baseline:* the committed E16 rows carry no host instance and no session.
*Target:* every contract ratio is published with `(host, instance id, session, barrier, baseline
commit)`; a row is `MET` only against a baseline measured in the same session on the same host;
the reference host for the thesis's contract table is named as C with its script; the three-arm
table from this audit (§2 F-29) is recorded as the demonstration.
*Guard:* a test that the rendered E16 header names a baseline commit; reverted, it fails.
*Guardrails:* no target relaxation; `report`'s 2.6× stays 2.6×.

### T-04a — Two lints and one pin — closes F-33

*Files:* `crates/niles-ir/src/…` (the two sites `run5.sh` §D names), `rust-toolchain.toml`.
*Target:* clippy green under both 1.95.0 and 1.97.1; the toolchain file pins `1.95.0` (author's
step — needs network) and its explanatory paragraph is deleted as it asks.
*Guard:* none needed beyond the gate itself on two toolchains; record both in the report.

---
*Cut line.*

### T-05 — Fold plateau attribution on C — F-28's residual

Callgrind is Linux-only; on C use `sample`/Instruments or `--per-connection` counters. Establish
whether the 4.4× plateau is the four performance cores (expected) or contention in
`report_from_view`'s V hold. Only if the latter: shard the resident iteration.

### T-06 — Idempotency-window memory over time — §6.5

Seed, append at a fixed rate for a simulated 30 days of epochs, sample RSS; report bytes per retained
key and whether the window is ever pruned. HS on A; HI counters on allocations.

### T-07 — E24 proper: RSS under load at 20k / 200k / 2M on C

`probes/mem`'s allocator figure with the daemon serving 8 connections, on C, via a `run6.sh`.

---

## 5. Validation protocol

| command | green means | red means for the claims |
|---|---|---|
| `bash docs/audit/cycle-7/preflight.sh` | environment stated | nothing later is interpretable |
| `cargo fmt --all -- --check`, `cargo clippy --offline --all-targets -- -D warnings` | baseline | nothing merges |
| `cargo test --offline --workspace` (serial, `PGPORT` set) | semantics | no performance claim |
| the T-01 crash test | **rows survive a restart** | every `durable` row is void |
| the T-02 fallback-rate test (≤5%) | the view serves under writes | the mechanism is off under load |
| E19 `mixed` row present with a fallback column | the number cannot hide | F-27's class is open again |
| E16 with a same-session baseline | the contract is a ratio | it is an instance's coin |
| `make reproduce` clean; `make fsync-proof` | reproducible; barrier reaches the kernel | stale evidence; volatile mount |
| `run4.sh` re-run after T-01 on C | durable curve within 10% of 7.80× at 16 | the record grew past the barrier's budget |
| clippy under 1.95.0 **and** 1.97.1 | the gate is about the code | it is about the calendar (F-33) |

---

## 6. Open questions

Settled, not reopened: LC-01, 02, 04, 08, 09, 10, 11, 12. **LC-14 stays retired** (F-28, two hosts).

- **LC-15 — sharpened by F-29.** The OLTP contract must be a same-session A/B on the executing host
  against a named baseline commit; C is the reference host. *Reversal cost:* prose.
- **LC-16 — durable by default?** F-26 makes this moot until T-01: today the flag buys idempotency
  recovery, not data recovery. After T-01, recommend durable by default with `--volatile` for
  benchmarks; the banner already exists.
- **LC-17 — closed by T-02 as far as this audit can see:** the mismatch was the engine refusing an
  exact answer, not a consistency trade. What remains is the residual (keys with a delta after the
  anchor), which reconstructs today and is correct. Retry/serve-later/block are unnecessary for that
  residual. *Astra to confirm against the algebra.*
- **LC-18 — no starvation at 8:1 on C (F-34).** Close, with a Linux 16:1 re-check once T-03 exists.
- **LC-19 — the reader-writer split does not change the wire claim; F-26 does.** Within a process:
  a session's anchor is monotone, the visible frontier moves only on barrier return, reads at the
  visible anchor are exact (T-02 makes them cheap). Across a restart the claim is void until T-01.
- **New — LC-20:** should the anchor a session observes be the *applied* frontier for reads that do
  not require durability (a bounded-staleness rung), so that the view can serve at `applied` without
  a fallback at all? This is a consistency-ladder question and a thesis question, not an engineering
  one. Do not decide it in a task.

---

## 7. Host C scripts

Both ran during this audit; the executor re-runs `run4.sh` after T-01 and T-02.

- `~/Documents/niles-hostc/run4.sh` — three arms of E19 at 1/2/4/8/12/16 × 3 runs, the mixed probe
  at 4r/2w, 8r/4w, 8r/1w, RSS sampling, the T-03 regression. ~30 minutes. Results:
  `results4-20260904-223831/`.
- `~/Documents/niles-hostc/run5.sh` — the T-03 regression against the right GBS commit, and clippy
  under 1.97.1. ~1 minute. Results: `results5-20260904-224638/` (F-33, F-37).

**The author must run this now**, after T-01 and T-02 land: `bash ~/Documents/niles-hostc/run4.sh`,
and compare the durable curve and the mixed rows against §2 F-28 and F-34.

---

## 8. Reporting requirements for the executor

The cycle-6 requirements, plus:

- **A requirements checklist** (F-32): every line of §4 copied verbatim with `done` / `not done:
  <why>`. A dropped line is a finding against the executor.
- The fallback rate under 4r/2w on A, before and after T-02.
- The crash protocol's transcript, before and after T-01.
- `run4.sh` re-run on C after T-01; the durable curve beside this audit's.
- `run5.sh` is resolved (F-37, F-33): T-03 confirmed on C; clippy red under 1.97.1 is the pin. The
  executor fixes the two `niles-ir` lints; **the author sets `channel = "1.95.0"` in
  `rust-toolchain.toml`** — that needs network once and is not the executor's to do.
- Worktree status separating the four untracked files from task changes; PostgreSQL stopped and its
  log kept; the attribution instruction or "none required".
- **Exactly three material facts found while executing that this order did not cover.**

---

## 9. Three things this audit found that its brief did not cover

1. **The daemon is not durable across a restart (F-26).** The brief asked whether crash recovery had
   been exercised under concurrency; the answer is that it had never been exercised through the
   daemon at all, and the segment beneath it works.
2. **The fallback is masked by the benchmark's base depth (F-27).** Two postings per account make a
   full reconstruction cost what a hit costs; the number that will show it is E23's base axis, which
   nobody has re-run since T-05.
3. **The contract flips between instances of one host class (F-29).** The brief warned about hosts;
   it did not anticipate that two containers of the *same* class would disagree by 20% on a ratio
   with 4% MADs on each side.
