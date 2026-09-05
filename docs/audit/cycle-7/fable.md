# Cycle 7 audit brief — **Fable**

**You are auditing, not building.** Your single deliverable is a **work order**: a prioritised,
evidence-backed set of tasks that Claude Opus will execute in a later session. You write no
production code into either repository. Probe crates outside the trees, benchmark runs, test suites,
`strace`/`callgrind`/`dhat`/`massif`, throwaway scripts — all encouraged. Commits are not.

The work order you produce is the only thing the executing agent sees. Assume it has none of your
context: every task carries its own preconditions, baseline, target, method, acceptance test, guard,
and guardrails, and must be executable **offline**.

**A second auditor — GPT 6 Astra — is reading the same two repositories independently, from a
different brief.** Its brief leans structural: claims-versus-code, the type system, the thesis text,
GBS layering, the bootstrap. Yours leans empirical. The two work orders will be reconciled by the
author afterwards. Three consequences:

1. **Where you both look at the same thing and disagree, the disagreement is the valuable part.**
   In cycle 6 the two audits reported group-commit ratios of 8.2× and 4.1× for the same change; the
   discrepancy resolved cleanly *because both stated their evidence class* — the ratio grows with
   the barrier's cost, and one host's overlay made the serialised arm mutex-bound rather than
   fsync-bound. State your evidence class. Never round toward the other auditor.
2. **Do not soften a finding because you expect the other to catch it.** Independent corroboration
   is why cycle 6 trusted F-12, F-13 and F-14.
3. **§8 lists items you must both check independently.** Do those even though you know they are
   duplicated.

---

## 1. The three goals, and the one you own

The stated end goals are **a highly efficient query language (Niles)**, **a highly efficient query
engine (Nilestream)**, and **a highly efficient core-banking system (GBS)** — efficient in **memory
and speed**, without surrendering any commitment in §10.

**Your territory is what can be measured**: concurrency, memory, locks, storage, allocation,
instruction counts, latency distributions, and the harnesses that produce them. Astra owns the
structural half. You are not forbidden from crossing — if you find a structural defect, report it —
but do not spend your budget re-deriving what a reading would give you when you are the only auditor
with a machine that can measure it.

Two framing questions you should answer with numbers rather than argument:

- **Is Nilestream's efficiency story bounded by design or by implementation?** Three cycles have
  found implementation ceilings — a lock, a barrier placement, a `&mut` taken for a counter. Say
  whether the next ceiling is another of those or something structural.
- **Can GBS's efficiency be measured at all yet?** One hold probe is the entire body of evidence.
  `G3` makes every sweep cold, records no product-internal reads and no performance. Decide whether
  a GBS efficiency task is admissible before F-24's product trace exists, and if not, say so — a
  refusal with a reason is a finding.

---

## 2. Access

### 2.1 GitHub

Both repositories are private, owned by **`shbiebs`**:

- `https://github.com/shbiebs/niles`
- `https://github.com/shbiebs/gbs`

They were pushed complete — 17 branches in `niles`, 9 in `gbs`, no tags — plus the cycle-7 branches
listed in §4.

**If you do not have credentials, ask, and ask for the narrowest thing that works.** You need read
access only:

- A **fine-grained personal access token**
- Repository access: **Only select repositories** → `niles` and `gbs`
- Repository permissions: **Contents: Read-only** (Metadata: Read-only is added automatically)
- Nothing else — no Administration, no Actions, no write

```
git clone https://github.com/shbiebs/niles.git
git clone https://github.com/shbiebs/gbs.git
```

Username `shbiebs`, password = the token. **Never push.** `Repository not found` on a private repo
is a token-scope problem, not a URL problem: say so and ask again rather than guessing at URLs. If
credentials are refused, audit what you can reach and say in your output which parts went unread. A
work order that guesses at unread code is worse than one that admits a gap.

### 2.2 The author's Mac — and the thing that will catch you out

You are authorised to audit the working copies at `/Users/checolino/Documents/niles` and
`/Users/checolino/Documents/GBS`.

**Read the next paragraph before planning a single measurement.**

If you reach the Mac through the Claude desktop bridge, **you do not get macOS.** You get an
isolated **Linux VM, aarch64, 4 cores, 4 GB RAM**, with `~/Documents` mounted over **FUSE**, and
**no `rustc`, no `cargo`, no `psql`, no `valgrind`.** That environment is fine for reading, grepping
and counting. It is **useless for storage measurement** — a FUSE mount's `fsync` is not APFS's — and
it cannot build the workspace.

**Everything that needs the real machine must be packaged as a shell script for the author to run in
Terminal.app.** That is the established convention: prior cycles left
`~/Documents/niles-hostc/run.sh`, `run2.sh`, `run3.sh` and read the results back. Follow it, and:

- Say **explicitly, at the point in your work order where it is needed**, "the author must now run
  `bash ~/Documents/niles-hostc/<name>.sh` and paste the output." Do not bury it.
- Make each script **idempotent, non-interactive, and bounded in time** — state the expected
  runtime. A script that needs a decision halfway through wastes a round trip.
- Have it write results to `~/Documents/niles-hostc/results-<stamp>/` and print a compact summary to
  stdout, so a paste-back carries the finding even if the files do not.
- **Never `chmod +x` and assume**: prior cycles hit `zsh: permission denied` because the file was
  not executable. Always instruct `bash <path>`, never `./<path>`.
- The author's zsh has `interactive_comments` **off** — a `#` typed at the prompt is executed as a
  command. Do not put comments in lines you ask them to paste.

The real Mac has Rust (`~/.cargo`, `~/.rustup`), PostgreSQL, and **10 cores (4 performance + 6
efficiency)** on Apple silicon with APFS. That is the only host in this project's history that can
answer the questions in §6.

**Read-only discipline on the Mac trees.** They are the author's live working copies. Do not commit,
stage, `git clean`, `git gc`, or check out a different branch without saying so first. `target/` is
acceptable collateral; nothing else is. **Four untracked files in `niles` must never be edited,
staged, deleted, moved or bundled:** `.DS_Store`, `AGENTS.md`, `thesis/.DS_Store`,
`thesis/Niles-Thesis.pdf`. Report their byte sizes at the end as proof they are intact.

Two environment traps that have each already cost a cycle:

1. **`CARGO_TARGET_DIR`.** Setting it made a nested `cargo run` contend with the outer `cargo test`
   on one build-directory lock: six spurious test failures and a *wrong verdict in a thesis table*.
   Cycle 6's T-03 removed every nested `cargo run` from the verdict tests. **Verify that on the Mac,
   with the variable set, at default parallelism** — it is the regression test for T-03 and it is
   the exact condition that failed.
2. **PostgreSQL's barrier on macOS.** Its default `wal_sync_method` there is a plain `fsync()`,
   which APFS does not turn into a drive-cache flush, while Rust's `sync_all` issues `F_FULLFSYNC`.
   Measured naively, PostgreSQL reports 13,458 durable commits/s against a **324/s** barrier — 41×
   its own storage — while reporting `fsync=on`. The harness now **refuses** the comparison unless
   `wal_sync_method=fsync_writethrough`. Do not work around the refusal.

---

## 3. Hosts, and what each may conclude

| host | what it is | admissible |
|---|---|---|
| **A** | 2-core Linux container, ext4 on virtio, `fdatasync` 4,961–5,825/s | counters, within-host ratios |
| **B** | the desktop bridge: 4-core aarch64 Linux VM, FUSE mount, no toolchain | reading, counting; **no storage or build evidence** |
| **C** | the Mac itself, Apple M4, **10 cores (4P+6E)**, APFS, `F_FULLFSYNC` **255/s** | connection curves, RSS under load, second-architecture runs, anything needing a real barrier or >3 cores |
| **D** | your own container, if on overlayfs `fsync=volatile` | **nothing about durability** — a ~1,000,000/s "fsync" is a mount option |

**Rules already violated once each:**

- No absolute wall-clock, throughput or fsync figure beside another host's without a host column.
- Every wall-clock threshold derived on the executing host in the same session. Nothing inherited.
- Deterministic counters (instructions, allocations, visited entries, `max_batch`, txns/fsync, lock
  buckets) gate first and need one run. Wall clock needs two warm-ups and **≥5 measured runs**,
  arms interleaved, median/MAD/range reported; a gate fires only on **≥10% and ≥3× the pooled MAD**,
  otherwise **`noise-limited`** — which is a result.
- `unsupported`, `blocked`, `not run`, `noise-limited` are results. Omission is not.
- Concurrency runs at 1/2/4/8; sealer probes add 16; ≥10-core hosts add 12 or 16. **A pass is a
  shape**: throughput non-decreasing through the available cores, p50 sub-linear in N, no
  resident-budget violation, bounded per-connection RSS slope, and **no rising correctness fallback
  hidden from the table.**
- The lock histogram reports **bucket boundaries, not interpolated values**. `p99 ≤ 2048µs` is
  honest; `p99 = 1873µs` would not be.

**Run the preflight first, in whatever container you audit from, and paste its output at the top of
your work order.** It is committed at `docs/audit/cycle-7/preflight.sh`, and Astra runs the
*identical* script, so the author can lay your two environment manifests side by side when
reconciling. It reports the host and the **cgroup-granted** core count (not `nproc`), the filesystem
and mount options under the tree, a 4 KiB + `fdatasync` barrier probe with a verdict, the toolchain,
PostgreSQL, egress, both tree hashes, and an admissibility table to fill in:

```
bash niles/docs/audit/cycle-7/preflight.sh /path/to/niles /path/to/gbs
```

Two things it will tell you that are easy to get wrong. **`nproc` is not your core count** — read
`/sys/fs/cgroup/cpu.max` and divide quota by period. And **a barrier above ~100,000/s is a mount
option, not a device**: cycle 6's other auditor measured ~1,000,000/s on an overlay mounted
`fsync=volatile`, which voided every durability number taken there. `make fsync-proof` passes on
such a mount too — it checks the syscall reaches the kernel, not that the kernel honours it — so the
two checks are necessary together and neither is sufficient alone.

**Before you write a probe, check whether cycle 6 built it.** Your own `durabled`, `wireprobe`,
`lockprobe` and `fs.c` are largely subsumed: `nilestreamd --durable <segment>` exists,
`bench --run --scaling-only --nls-only` exists, `select nilestream_sealer` exposes the sealer and
lock counters, and `make fsync-proof` straces the barrier. Rebuild only what is genuinely missing,
and say what you found already present.

---

## 4. Exactly where the trees stand

**Niles** — base `4d9a457`, seven stacked branches, none merged to `master`:

| branch | head | task |
|---|---|---|
| `c6/01-evidence` | `79c2881` | T-01 evidence manifest, green tree |
| `c6/02-instruments` | `05eeef7` | T-02 durable daemon, `--nls-only`, sealer + lock counters |
| `c6/03-verdict-tests` | `a25d7ab` | T-03 nested `cargo run` removed from verdict tests |
| `c6/04-fsync-contract` | `d6e535b` | T-04 fsync ceiling per host and barrier |
| `c6/05-unlock` | `15425b5` | T-05 barrier released from the engine lock |
| `c6/06-rwlock-read` | `d681f0d` | T-06 reader-writer base, shared read path |
| `c6/06a-lock-order` | **`d9c8699`** | T-06a lock order **B < P < V < C**; the deadlock in §6.1 |

**GBS** — base `572fe97`:

| branch | head | task |
|---|---|---|
| `c6/01-evidence` | `e2cb34e` | T-01 GBS half; `g3` post-diffed |
| `c6/03-verdict-tests` | `e6418e0` | T-03 GBS half |
| `c6/07-hold-index` | **`e803b7d`** | T-07 index-backed hold resolution |

Gate at `d9c8699` / `e803b7d`: `cargo fmt --check` green, `cargo clippy --all-targets -- -D warnings`
green, Niles **801 test functions, 0 failed** (1 ignored), GBS **506 declared / 504 passing**,
`make reproduce` exit 0 with a clean diff, `make fsync-proof` green. `numeric_binary_oracle` needs a
running PostgreSQL with a `bench` role and `PGPORT` set; without it the tree reads red for a missing
service, not a defect.

---

## 5. What cycle 6 changed, and its measurements

Do not re-derive this. Do audit it — none of it has been reviewed by anyone.

**T-01** — 14 of 51 files under `results/` were regenerated by `make reproduce`; **37 were not**,
including `E16-wallclock.md` (the contract table) and `E19-scaling.md` (produced by no recipe at
all). `results/MANIFEST.csv` now classifies every file with its producing command, and a test
enforces that **`make reproduce` runs no machine-dependent producer** — otherwise the gate
overwrites an acquired measurement with a re-measurement. GBS ran `g3` *after* its diff, so the
product gate's own verdict was the one file the check could not see.

**T-02** — `nilestreamd --durable <segment>`; `bench --scaling-only --nls-only`; `select
nilestream_sealer` exposing `epochs_sealed`, `txns_committed`, `fsyncs`, `duplicates_absorbed`,
`max_batch`, `txns_per_fsync`, and a bucketed lock wait/hold histogram (power-of-two µs, relaxed
atomics).

**T-03** — both verdict suites classified a non-zero `cargo` exit as a *compiler refusal*.
`niles_schema.rs` asserts in both directions, so the counterfactual tests proving a bad program is
rejected would have passed **without the compiler ever seeing the program**. Verdicts now come from
`nilesc`'s own markers; an absent compiler is `BLOCKED-nilesc`, never `Refused`.

**T-04** — every durable result names `(host, filesystem, mount options, device class, barrier,
ceiling, spread)`; `bench` pre-registers PostgreSQL's `wal_sync_method`; `make fsync-proof` straces
one durable append and asserts an `fdatasync` reaches the kernel per epoch. `SPEC-ENGINE.md` Part 0
states `durable throughput ≤ barriers/s × transactions per barrier`.

**T-05** — the daemon held one mutex across `Session::handle` **and the `fsync` inside it**, so a
second submitter could not reach the sealer and group commit — built, and tested at sixteen
concurrent submitters — never formed a batch. `Sequencer::submit_pending` now returns the reply
channel instead of blocking; `append` applies under the lock and pushes the barrier token to a
pending list; `serve` takes the tokens, releases the lock, waits, **then** writes the reply. A
separate **`visible` frontier** is published by `fetch_max` when the barrier returns, so an
applied-but-not-yet-durable epoch is invisible to every reader.

| connections | `max_batch` | txns per barrier | lock hold p50 |
|--:|--:|--:|--:|
| 1 | 1 | 1.00 | 16 µs |
| 16 | 15 | 6.02 | 4 µs |

Durable appends **3,850 → 22,049 ops/s** from 1 to 16 connections: **5.73×**.

**T-06** — `Base::reconstruct` took `&mut self` **only to increment a row counter**. That `&mut`
propagated: every read of the base exclusive → `Serving::query` exclusive → one mutex around
`Session::handle`. The counter is now an `AtomicU64`; the base is an `RwLock<Ledger>`, shared for
reads and exclusive for appends; the read model has its own `Mutex` because a read of a partial view
*is* a write to it; `Serving` is a shared-borrow trait and `serve` acquires nothing.

| workload | 1 | 2 | 4 | 8 |
|---|--:|--:|--:|--:|
| fold before | 164 | 270 (1.65×) | 227 (1.38×) | 209 (1.27×) |
| fold after | 165 | 278 (1.68×) | 259 (1.57×) | 244 (1.48×) |
| point before | 15,122 | 39,493 (2.61×) | 29,545 (1.95×) | 31,920 (2.11×) |
| point after | 15,857 | 79,814 (5.03×) | 79,668 (5.02×) | 57,479 (3.62×) |

Lock counter, four connections folding: wait p99 ≤1,024 µs → **≤1 µs**, max 6,309 µs → **2 µs**,
hold p50 unchanged at ≤256 µs. **All of it on two cores**, which is why §6 exists.

**T-06a** — see §6.1.

**T-07** — proving a hold open scanned the account's entries, then *all* of global `hold.void`, then
all of `hold.expired`: 29 µs at 0 voids → **8,630 µs at 40,000**. Now indexed through the kernel's
existing `consumed` map: **26 µs and flat**, zero entries visited for an open hold.

---

## 6. Where your budget should go

### 6.1 The measurement that has never been taken — **your primary assignment**

**T-06's real targets were never evaluated.** They are Host C targets: the scan-shaped read
(`fold` workload) at **4 connections ≥3.0×** its 1-connection figure and at **8 connections ≥4.5×**,
with p50 at 8 ≤3× the 1-connection p50, and the point curve not regressed. The executing container
had **two cores**, where a CPU-bound fold's ceiling is ~2.0×, so the numbers above are 84% of what
that host could give and say nothing about a wide one.

This is the single largest open measurement in the project, it is why you have Mac access, and it
determines whether the reader-writer split was worth its complexity or whether the next step is the
chunked lock-free storage that cycle 6 closed on one arm of a two-arm trigger:

- **Arm 1** (evaluated on A, does not fire): lock wait after the RwLock is **<1%** of counted
  request time against a 20% threshold.
- **Arm 2** (never evaluated): the RwLock arm is **<1.5× baseline throughput at 8 connections**.

Re-evaluate **both** on Host C. LC-14 (eviction budget under sharding) stays retired unless one
fires. Note also that F-23 — "point reads are not lock-bound" — was measured on ten cores, and on
two cores the mutex was costing point reads 2.6×. Establish which reading survives at 10 cores after
T-06.

### 6.2 A lock-order deadlock, found and fixed — **audit the fix**

T-06 replaced one mutex with finer locks and did not say which comes first. `answer_from_view` took
the read model and reached for the base underneath it; `append` took the base exclusively and
reached for the read model underneath *that*. AB–BA, reachable in the shipped daemon the moment one
client read while another wrote. **No test saw it**, because every concurrency test in the workspace
drives one workload at a time — and a correctness test cannot catch it in any case: no answer is
ever wrong on the way into a deadlock.

It was demonstrated before being fixed: four readers with a budget guaranteeing misses against a
continuous appender, under a thirty-second deadline — **two of five threads finish** on the old
order. `d9c8699` states the order **B (base) < P (pending barriers) < V (read model) < C (currency
set)**, makes every path take a subsequence, and holds `answer_from_view` to **one** base guard for
the whole function rather than two in the right order, because `RwLock` is not reentrant.

**Three things remain open and they are yours:**

- The enumeration covered `nilestream-server` only. **`nilestream-ledger` and `nilestream-core` have
  never been examined for lock order**, and the sequencer has its own.
- **Audit the fix.** It was written by the agent that wrote the defect, in the same session, reviewed
  by nobody.
- Under load on 10 cores, does the fix cost anything? The base read guard is now held across the
  whole keyed read. Measure it.

### 6.3 The counter specified twice and shipped neither time

Work order 6 required T-02's stats surface to report **anchor-mismatch fold fallbacks**, and required
the cycle-6 report to state **the fallback rate after T-05**. Neither exists. This matters more now:
the comment justifying the check claimed the frontier could not move during a read *because the
write path held the engine lock* — false since T-05, corrected in `d9c8699`. The mismatch is now a
**normal event**, and a point read that falls back to the fold is ~100× more expensive. **A mixed
read/write workload could collapse from 80,000 ops/s to fold speed with every test green and no
number anywhere showing it.** Build the counter, then build the workload that exercises it.

### 6.4 The mixed-workload harness that does not exist

Every row in every results file is one workload at a time. Reads concurrent with writes is the shape
a bank runs, the shape §6.3's fallback needs to fire, and the shape §6.2's deadlock needed. The
deadline test added in `d9c8699` is the project's **first** mixed-workload exercise and it exists to
catch a hang, not to measure anything. **Building a real mixed harness is probably the highest-value
instrument this cycle can produce**, and it is squarely yours.

### 6.5 Everything else measurable that is open

- **E16, the contract table, has not been re-measured since T-05 or T-06.** The published contract
  rows predate the two largest changes in the project.
- **Crash recovery has never been exercised under concurrency.** Group commit now forms batches of
  fifteen; a crash mid-batch with a partially-written segment has been tested single-threaded only.
  Durable-before-visible is the project's central claim.
- **Residency and memory under concurrency** (F-22's instrument half) and **memory against base
  size** (the unbuilt E24) remain unmeasured. Bytes per base row at 20k / 200k / 2M is the question
  the thesis is actually about.
- **The idempotency window.** `idem: IdemKey window 30.days` — nobody has measured what that index
  costs as history accumulates.
- **The eviction budget has only ever been measured in one shape**: 10,000 accounts, 2,500-entry
  budget, and the phase diagram is swept in the prototype rather than through the daemon.
- **The plan cache under statement diversity** — epoch-keyed, invalidated by schema epoch, untested
  against thousands of distinct statements.
- **Slow-consumer streaming.** F-25 concluded bounded reply streaming should be left alone *and that
  a slow-consumer measurement was needed first*. The second half never happened.
- **The hash chain is a placeholder** (`README.md`, ADR 0002) while a hand-written SHA-256 exists
  elsewhere and was measured at ~30% of the daemon's seeding instructions. Every durability and
  throughput number therefore includes a fake chaining cost. Bound what a real hasher costs on the
  write path and say which published figure changes.
- **`no unsafe` is not data-race freedom.** The thesis claims immutability and data-race freedom as a
  *result*. Nothing has checked it against the new relaxed counters, the `AcqRel` visible frontier,
  and the `fetch_max` publication protocol.
- **T-05's misses**: durable ≥6× at 16 measured **5.73×**; p50 at 16 ≤2× the 1-connection p50 not
  met and not re-measured after T-06; "≥80% of the same-host concurrent sequencer arm" is not
  evaluable as written because the probe and the wire path do not share a client — restate the target
  or build the comparison.
- **`make gate` end to end on GBS** was never run in cycle 6.

---

## 7. Where *not* to spend the cycle

Refuted, twice in some cases. Reopen only with evidence that overturns the measurement:

- Compile time, startup, binary size, idle RSS.
- Connection memory at this scale — peak RSS of the durable daemon is 4,836 KiB at 16 connections
  (A) and 23,312 KiB at 12 (C).
- Reply streaming — 1.04× / 0.94× streamed-versus-assembled at 10k / 100k rows, within scatter,
  while peak reply memory stays at 69,632 B instead of 835 KB / 6.7 MB. **Except** the slow-consumer
  measurement in §6.5.
- Crypto dependencies (settled: none).
- Point-read sharding, unless §6.1's trigger fires.

---

## 8. Check these independently, even knowing Astra also will

Corroboration is what made cycle 6's findings trustworthy:

1. **The T-06a lock-order fix** (§6.2) — is it complete, and is the guard the right one given the
   zero-dependency rule that forbids `loom`?
2. **Is the `answered.anchor != anchor` check still *sufficient*** under the new concurrency, and is
   a fold fallback the right response (LC-17)?
3. **Is any published number now stale** after T-05, T-06 and T-06a?
4. **What does "efficient" mean for each of the three artefacts**, stated so it can be measured?

---

## 9. Potential misses in every cycle so far

Confirm or refute each; a refutation with evidence is as valuable as a finding.

1. Nothing has ever been measured under a mixed workload (§6.4).
2. Lock order had never been analysed until §6.2, and only one crate is done.
3. Crash recovery under group commit is untested.
4. The eviction budget has one shape.
5. The idempotency window's memory is unbounded in practice and unmeasured.
6. The plan cache is untested under diversity.
7. Cross-target claims exceed cross-target evidence — Appendix C claims Linux, macOS ARM64, Windows
   x86-64, WSL and WASM; only x86_64-linux and arm64-macOS have been exercised, and only for the
   compiler corpus.
8. The bootstrap (Appendix E) has gates but no audit.
9. The thesis text has not been reconciled with the code since cycle 6 — `SPEC-ENGINE.md` Part 0 was
   rewritten by the executing agent; chapters 4, 6 and 9 were not checked.
10. Slow-consumer behaviour is unmeasured.
11. `no unsafe` ≠ data-race freedom, now that there are new atomics and three locks.
12. GBS has no efficiency instrument at all, and F-24's product trace is the precondition for any
    GBS efficiency claim.

---

## 10. Constraints the executing agent is bound by — carry them into the work order

Non-negotiable; no task weakens one without an explicit decision routed to the author:

durable-before-visible; one sealer per ledger; one total epoch order; full retention; honest absence
(the three-valued `Reading` lattice); fold-never-field; product purity; self-describing amounts; GBS
layering; **no `unsafe`**; **no default-on-error**; **zero external dependencies**; no fabricated
results; the thesis follows the code with `MISMATCH-<id>` at every divergence; a red test is a result
and is never fixed by deletion; honest refusal over silent fallback; `BLOCKED-<id>` on ambiguity.

Added in cycle 6 and now load-bearing:

- **Apply-before-publish.** The frontier may not advertise an epoch whose barrier has not returned.
- **A stated lock order.** B < P < V < C, and any new lock must be placed in it.
- **Every optimisation ships its guard and threshold in the same commit**, and the executor proves
  the guard *fails* on the reverted optimisation, transcript in the report. Where a repair is
  invisible to behaviour — T-03's, T-06's and T-06a's all were — the guard must read the source.
- **No performance claim without a reproducible command**, raw results under `results/`, prose
  generated from them, host class and barrier name in the header.
- **Benchmarks touch committed artefacts only with `--publish`.** `--scaling-only` with a custom
  `--out` is the safe shape.
- **Micro-benchmark gains are never contract results.** Only E16/E19 rows are.
- **No task may require network access at execution time.** No egress to crates.io or
  `static.rust-lang.org`. Rust 1.95.0, valgrind 3.22 with callgrind/dhat/massif, and a startable
  PostgreSQL 16 are preconditions, not steps. **No task installs anything.**
- **Attribution** is taken from the executing session's own instructions; record "none required" if
  there is none. Never bake in a trailer.

---

## 11. Open questions to carry, restate or close

Settled in cycle 6, **not to be reopened**: LC-01, LC-02 (answered A and implemented), LC-04, LC-08,
LC-09, LC-10, LC-11, LC-12.

- **LC-03** — `AccountId` String→u64: full conversion (~245 sites, 17 files, best memory) vs a
  boundary newtype vs no change until a census attributes ≥10% to identity strings.
- **LC-05** — rendered-NULL, seed depth, SQL refusal boundaries.
- **LC-06** — process/allocator boundary for comparative memory: whole-process RSS (fair, noisy) vs
  allocator accounting (deterministic, excludes PostgreSQL internals) vs withdrawing the ratios.
- **LC-07** — E23's H-F1 slope and floor.
- **LC-13** — release checks mandatory or advisory.
- **LC-14** — eviction budget under sharding. **Retired**, reopened only if §6.1's trigger fires.
- **LC-15** — what the OLTP contract is measured against. Accepted: the ratio against PostgreSQL on
  the same storage under the same barrier.
- **LC-16** — should `nilestreamd` be durable by default? Still unanswered.

New, from cycle 6 — raise, do not answer alone:

- **LC-17** — is a fold fallback the right response to an anchor mismatch, now that the mismatch
  fires in normal operation and costs ~100× the read it replaces? Alternatives: retry at the
  observed anchor; serve at the later anchor and report it; block briefly.
- **LC-18** — is `std::sync::RwLock` the right primitive for the base? It is not writer-preferring
  on every platform; a sustained fold stream could starve appends, which on a ledger is a durability
  latency problem. **Measure before deciding — this one is yours.**
- **LC-19** — does the reader-writer split change what "strictly serializable" is claimed at the
  wire?

---

## 12. What you must produce

A single work order containing:

0. **The preflight output, verbatim, with its admissibility table filled in** (§3). Every number
   later in the document is read against it.
1. **An executive judgement** — where the three artefacts stand against the three efficiency goals,
   with the arithmetic for any reachability claim spelled out.
2. **Findings**, each with a class (wrong-measurement, correctness, liveness, guarantee-bounded,
   instrument-gap, negative), an **EV score** (`impact × confidence ÷ cost`, each 1–5, ties favour
   correctness), **HI** (host-independent) or **HS** (host-shaped), evidence with file and line, the
   measured cost, the proposed repair, and **what the repair gives up**. Negative findings — "not
   worth doing, here is the measurement" — are first-class and are what stop the next cycle wasting
   a week.
3. **A task list in dependency order**, each task carrying what it closes, what it depends on,
   whether it is executable or gated, its files, its **baseline measured on the executing host**,
   its **target as a ratio against that baseline**, its method, its acceptance test, its **guard and
   the reversion that must make the guard fail**, and its guardrails. Mark a **cut line** for one
   agent in one cycle.
4. **Branch stacks and merge order** for both repositories from the current heads.
5. **A validation protocol**: commands, what green means, what a red row means for the claims that
   depend on it.
6. **Every Host C script you need the author to run**, named, with its expected runtime, and an
   explicit "**the author must run this now**" marker at the point in the document where the result
   is needed.
7. **The open-questions ledger** (§11), updated.
8. **Reporting requirements** for the executing agent, including **every target it missed**, guard
   transcripts, worktree status separating the four untracked files from task changes, and **exactly
   three material facts found while executing that the work order did not cover**, negative findings
   included.

Order by `impact × confidence ÷ cost`, not by interest. And weigh two things:

- The stated goal is efficiency; the **binding constraint** is correctness. A liveness bug outranks
  a factor of two. A durability claim that is not true outranks both.
- Three cycles have now found that **the highest-value defects were invisible to the test suite** —
  a verdict rule that could not tell "the compiler refused" from "cargo did not run"; group commit
  built, tested and structurally unreachable; a `&mut` taken for a counter that serialised every
  read; and a deadlock that no correctness test could ever catch. **Look for the class, not the
  instance: what else is correct on every input and wrong in structure?**
