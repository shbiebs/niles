# Cycle 10 work order — Fable

**Auditor:** Claude Fable 5.1, container, from `docs/audit/cycle-10/fable-audit-prompt.md`.
**Trees audited:** `niles` at **`8548189`** (`c10/00-audit`, = `53b40f6` + the briefs) and `gbs` at
**`688919c`**, both in the container's clones; the author's Mac read through the bridge.
**Executor:** Claude Opus, in a later session, offline. This document is the only thing it sees.

GitHub was **not reachable from the container** (`GIT_TERMINAL_PROMPT=0 git ls-remote` →
"could not read Username"; the repositories are private and the container holds no credential).
No token was requested: the container's clones are the histories the bundles carry and the Mac's
trees were read directly, so nothing in this work order depends on GitHub. The author should
still confirm, on GitHub, that `c9/07-pending`, `c10/00-audit` and `c7/01-durable-rows` are at
`8548189` and `c7/00-adapter` at `688919c` — the second of those is **not yet true** (§0.2).

---

## 0. Preflight and admissibility

### 0.1 Container preflight, verbatim

```
cycle-10 audit preflight — 2026-09-08T16:05:54Z

=== A. host
uname            : Linux 6.18.44-fc-v24 x86_64 GNU/Linux
cores (nproc)    : 2
cpu model        : Intel(R) Xeon(R) Processor @ 2.80GHz
memtotal         : 7.8 GiB
cgroup cpu.max   : n/a
cgroup mem.max   : n/a

=== B. filesystem under the tree — THE decisive section
path             : /home/claude/work/niles
mount            : /dev/vda ext4   rw,relatime,resv_strict,resuid=65534,resgid=65534
free space       : 18G avail of 252G
barrier          : fdatasync
median           : 139.0 us  ->  7,195 barriers/s
spread           : 136.9-940.4 us
VERDICT          : plausible as a real barrier for THIS container. Still a
                   host-shaped number: usable for within-host ratios only.

=== C. toolchain
pin              : channel = "1.95.0" — does NOT resolve here — every probe uses RUSTUP_TOOLCHAIN=stable
rustc            : rustc 1.95.0 (59807616e 2026-04-14)   (toolchain: stable)
cargo            : cargo 1.95.0 (f2d3ce0bd 2026-03-21)
host triple      : x86_64-unknown-linux-gnu
1.97.1           : not installed here — the author runs that gate on the Mac
valgrind         : valgrind-3.22.0
python3          : Python 3.11.15

=== D. PostgreSQL
psql             : psql (PostgreSQL) 16.13
pg_isready       : /var/run/postgresql:5432 - accepting connections   (started by hand first: `sudo -n service postgresql start`)
bench PostgreSQL : 127.0.0.1:5433 - no response

=== E. network egress
https://github.com          : 400
https://crates.io           : 403
https://static.rust-lang.org: 000FAIL

=== F. trees
/home/claude/work/niles   HEAD 8548189  branch c10/00-audit   dirty 0
/home/claude/work/gbs     HEAD 688919c  branch c9/02-idem-key-only   dirty 0
```

### 0.2 The Mac, through the bridge (read-only)

| | value |
|---|---|
| `~/Documents/niles` | **`8548189`** on `c7/01-durable-rows`; branches `c9/07-pending`, `c10/00-audit` present; clean apart from the five protected files |
| protected files | `.DS_Store` 10,244 · `AGENTS.md` 16,639 · `niles/.DS_Store` 6,148 · `thesis/.DS_Store` 8,196 · `thesis/Niles-Thesis.pdf` 816,110 — **unchanged** |
| `~/Documents/GBS` | **`688919c`** on `c7/00-adapter` as of 16:10:42 UTC (reflog: two fast-forwards, 447544d then 688919c) and pushed. It was at `963e4d9` when this audit's first bridge read was taken at 16:05 UTC; the author synced it five minutes later. |
| `~/Documents/niles-sync/cycle-10/` | both briefs and `niles-c10-audit.bundle` present |

**A correction to this audit's own instruction.** The GBS sync block given at three cycle-9
landings and again in this work order's first draft named the ref `c7/00-adapter` on both
bundles. The bundles carry **`c9/01-batch-seq`** and **`c9/02-idem-key-only`** (`git bundle
list-heads`); a fetch by the name given fails with `couldn't find remote ref`. The author landed
the commits anyway at 16:10 UTC, and the block as written then reported `Already up to date` and
`cannot force update the branch used by worktree` — both harmless, both symptoms of a wrong
instruction. **Rule for the executor, added to §8:** a sync block names the ref the bundle
actually carries, checked with `git bundle list-heads` before the block is written, and never
`git branch -f`s the branch that is checked out.

The Mac preflight has been run (§0.2a); `c10-rwlock.sh` has been run (§0.2b).

### 0.2a Mac preflight, verbatim (the sections that carry facts)

```
cycle-10 audit preflight — 2026-09-08T16:32:46Z
=== A. host
uname            : Darwin 25.6.0 arm64
cores (nproc)    : 10
memtotal         : 16.0 GiB
=== B. filesystem under the tree
mount            : /dev/disk3s1s1 on / (apfs, sealed, local, read-only, journaled)
barrier          : fsync
median           : 15.7 us  ->  63,658 barriers/s
spread           : 13.1-48.2 us
VERDICT          : suspicious — verify the mount is not volatile before
                   publishing any durability figure.
=== C. toolchain
pin              : channel = "1.95.0" — resolves here
rustc            : rustc 1.95.0 (59807616e 2026-04-14)   (toolchain: 1.95.0)
1.97.1           : installed — the author's newer-lint gate
valgrind         : ABSENT — no callgrind/dhat/massif attribution
strace           : ABSENT — make fsync-proof cannot run
=== D. PostgreSQL
psql             : psql (PostgreSQL) 18.6 (Homebrew)
pg_isready       : /tmp:5432 - no response
bench PostgreSQL : 127.0.0.1:5433 - no response
=== E. network egress
https://github.com          : 200
=== F. trees
niles  HEAD 69488a81  branch c7/01-durable-rows  dirty 5 (the five protected files)
GBS    HEAD 688919c   branch c7/00-adapter       dirty 0
=== G. gate
  cargo test --offline --workspace   # niles: ~907 #[test] attributes; gbs: ~567
```

**Two of those lines are the instrument, not the machine** (F-10-13). Section B's "barrier:
fsync, 15.7 µs, *suspicious*" is Python's `os.fsync`, which on Darwin does **not** reach the
device — `F_FULLFSYNC` does, at ~255/s, and that is what the ledger's sink issues. The preflight
measures the wrong syscall on the one host whose storage figures are publishable, and reports
the reference host as suspicious. Its "mount … read-only" line is the sealed APFS system volume
that `df` resolves the path to through a firmlink, not the Data volume the tree lives on. Neither
changes any admissibility answer below — cycle 9's `c9-storage.sh` contract and the sink's own
probe are the storage evidence — but a preflight whose section B is wrong on Host C is a
preflight that would have passed a volatile mount on Host C too.

### 0.2b `c10-rwlock.sh` on Host C, verbatim

```
=== c10-rwlock: 2026-09-08T16:32:33Z on Darwin 25.6.0 arm64 ===
toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
platform                                     : macos / std::sync::RwLock
second reader admitted AFTER the queued writer : 200/200  (writer-preferring)
second reader admitted BEFORE the queued writer: 0/200  (reader-preferring)
second reader's wait, us                      : p50 217 p90 222 max 269
```

**200 of 200.** On the reference host a queued writer blocks every later reader, without
exception and with less jitter than Linux (143/200 in the container, where the 57 were
scheduling races on two cores). The mechanism behind every slowest-16 table in C9-06.2 is
established on the machine that produced them: a reader folding under the base guard queues
the appender, and every reader arriving behind the appender waits for both.

### 0.3 Admissibility

| question | container | Host C |
|---|---|---|
| cores actually granted | 2 (no cgroup quota reported; `nproc` = 2) | 10 (`nproc`; no cgroup) — measured |
| barrier, median, rate | `fdatasync`, 139 µs, 7,195/s | preflight prints `fsync` 15.7 µs / 63,658/s, which is **not a barrier on Darwin** (F-10-13); the sink's `F_FULLFSYNC` is ~255/s (cycle 9's `c9-storage.sh` contract, to be re-run before any durability row) |
| storage evidence? | yes, ratios only | yes — from the sink's probe, not the preflight |
| may publish durability rows? | **no** | yes, after `c9-storage.sh`'s `F_FULLFSYNC` line is on record this cycle |
| may publish > 3-core curves? | **no** | yes |
| toolchain newer than 1.95.0? | no (stable = 1.95.0) | pin resolves; 1.97.1 for lint only — measured |
| valgrind attribution possible | yes, per function | **no** (absent) — measured |
| PostgreSQL comparison possible | 16.13 on 5432 after a manual start; nothing on 5433 | **PATH has 18.6 (Homebrew)**; nothing on 5432 or 5433; a PG16 on 5433 is still the C republish's precondition (C10-11) — measured |
| `make fsync-proof` | runs | **cannot** (no `strace`) — the Mac gate is fmt + clippy + tests + reproduce |

Every container figure below is a **count, a ratio, or a per-function instruction total**.
Nothing in this document is a wall-clock claim about Host C except where labelled *cycle 9,
Host C* and taken from `c9-pending-results.md`.

### 0.4 The gate, at these heads

| tree | row | result |
|---|---|---|
| niles `8548189` | `make gate` | **exit 0** (fmt, clippy, workspace tests, memprobe budgets) |
| niles | `cargo test --workspace --no-fail-fast` | 1,037 passed, 0 failed, 7 ignored (named in cycle 9's report §3) |
| niles | three runs at `--test-threads=8` under two `yes >/dev/null &` hogs, one `taskset -c 0` at `--test-threads=1` (taken at `dc19a4a`, which differs from `8548189` only in `docs/`) | 1,037 / 1,037 / 1,037 / 1,037, **no changed verdict** |
| gbs `688919c` | fmt, clippy, workspace tests, `make reproduce` | all exit 0; **504 passed, 0 failed, 1 ignored** |
| cross-repo | `downstream_adapter` (compiles the real `gbs-nilestream` from the sibling checkout) | green **in the container**; **red on the Mac** until §0.2 is run — see F-10-03 |

Test-attribute counts, by the rule "occurrences of `#[test]` under `crates/` and `tools/`": niles
916, gbs 625. The preflight's own count (`crates/` only, one rule) prints ~904 / ~560. The
runner's pass count is the number that matters and is above.

---

## 1. Executive judgement

**Where the three artefacts stand.** Nilestream's write path is durable, receipt-owned and
strictly recoverable, and its read path now reaches all four states of the absence lattice under
load; those are the claims the thesis makes and the engine now supports. Its efficiency ceiling is
**one lock**: the base `RwLock<Ledger>`, held shared across every reconstruction and exclusively
across every append. Niles has been swept for "checked twice" and the answer is now *no*: what an
insert spends its instructions on is a reference SHA-256 (56%) and allocation (~10%), not a fact
the compiler already proved (§2, F-10-06). GBS is exactly where cycle 9 left it — inadmissible
for an efficiency claim, six rows generic, F-24 and F-12 open — and, today, **its checkout on the
reference host does not compile against the engine it adapts** (F-10-03).

**The three questions.**

1. *What lets a reconstruction run over an immutable prefix without holding the base lock?*
   **Today, nothing can.** `Ledger.epochs` is a `Vec<EpochRec>` and `by_account` is a
   `HashMap<Acct, Vec<RowRef>>`; both are mutated by `submit` and both move under an append (a
   `Vec` reallocates on push; a `HashMap` rehashes). A snapshot handle over the in-memory base
   is not a guard change, it is a storage change (chunked, append-only, `Arc`-shared) — LC-37,
   below the cut, with a spike. What *can* be done this cycle is to make the hold **short** and
   the lock **honest**: the base lock's histogram cannot tell a reader's hold from the writer's
   (F-10-02); split it, measure on Host C, and then bound the reader's hold with checkpoints
   (C10-04, which is C9-07 promoted for a new reason — a fold of ≤ C+1 rows is a hold of ≤ C+1
   rows). The container measures **readers holding the base 2.9× longer in aggregate than the
   writer** (2.82 s against 0.98 s of a 10-second level), reader holds reaching 3.2 ms, and a
   queued writer that then blocks later readers (143/200 on Linux `std`; Darwin to be measured by
   `c10-rwlock.sh`). That is the mechanism, shown rather than inferred. The lock order survives
   every candidate in this work order; the one path that violates it today is F-10-01.
2. *Where else does the engine re-establish a fact the type system proved?* **Nowhere that
   costs anything.** Per-function callgrind on `checked-twice oltp`: `Hasher256::compress` 56.3%
   of the process, SipHash/`hash_one` 7.0% (the idempotency, conservation-sum and account
   indexes), malloc/free ~10%, `Ledger::submit` self 2.4%, `Session::insert` self 1.0%. The
   conservation re-check in `submit` is a *data* fact (a wire row can be unbalanced) and stays;
   at ~360 instructions per epoch it is not a cost. The sweep is closed; what it found is that the
   hash chain is computed under the most contended lock in the engine (F-10-06).
3. *Which Loan IQ / Calypso shape is a new invariant, and which closes F-24 or F-12?* Unchanged:
   the thousand-share pro-rata payment (C9-G01 → C10-G01) is both (i) and (ii) and is still the
   only admissible product task. New this cycle from the vendor material: **partitioned
   conservation** (CCP account segregation) is probably expressible today by adding the partition
   column to `conserve per (…)`'s key list — a transaction must then balance within each partition
   — which makes it a new *use* of an old spelling and one GBS row rather than a language task;
   what the spelling cannot say is selective netting across some lines and not others (LC-41). Nothing in
   §6.5 of the brief is admissible ahead of C10-G01, and C10-G01 is below the cut because two
   liveness-or-gate findings and one instrument outrank it.

**Reachability arithmetic.** The only efficiency target this work order sets is scored against
`c10-baselock.sh --baseline-only`, which the author has not yet run. From cycle 9's Host C
transcript (5 replicates, 12r/6w): 160,838 reads/s median, pooled MAD 888; slowest-16 `base
wait` 400–1,100 µs; `pinned_installs` 311,921 per 30 s level (6.35% of reads). C10-03 removes
~99% of those pinned installs (the container measures the gap `applied − anchor` as exactly 1 in
**98.7%** of pinned installs on the served path); each is a fold that no longer happens. What that
is worth in reads/s on Host C is not predicted here — cycle 9's prediction was 1.25× and the
measurement was 1.018× — it is measured.

---

## 2. Findings

Class · EV = impact × confidence ÷ cost (each 1–5) · HI (integrity) / HS (speed) · evidence class.

### F-10-01 — `read_stats` takes V then B; `append` takes B then V: `select nilestream_stats` concurrent with an `INSERT` deadlocks the daemon
*liveness · EV 5×5÷1 = 25 · HI · measured in the container.*
`crates/nilestream-server/src/rev_engine.rs:991–995`: `let guard = self.runtime…map(|rt|
Timed::acquire(rt, &VIEW_LOCK)); let idem_keys = self.base().idem_window_keys()` — V is alive
when B is taken. `append` (`:872`, `:934`) takes B exclusively and then V. The lock-order source
guard (`the_base_is_acquired_before_the_view_on_every_path_that_takes_both`) checks exactly two
functions, `answer_from_view` and `append`; `read_stats` is not one of them, and no behavioural
test drives a stats query against a writer. **Reproduced**: a throwaway test with one thread
looping `append` and one looping `read_stats`, under a 5 s deadline, made **zero** progress on
either side (`PROBE: appends=0 stats=0 stalled_ms=1000`); each side alone completes 199/199 and
100/100. Reachable from the wire: every `select nilestream_stats` is `read_stats`. The benchmark
never hit it because it samples the counters before threads start and after they join.
*Repair:* take B before V, or take neither together — read `idem_window_keys` first and drop the
guard before V. *Gives up:* nothing; the two values are not read atomically today either.

### F-10-02 — the base lock's histogram cannot tell a reader's hold from the writer's
*instrument-gap · EV 4×5÷1 = 20 · HS · read from source, measured in the container.*
`lockstats.rs:203`: one `ENGINE_LOCK` for `TimedRead` (`rev_engine.rs:617`) and `TimedWrite`
(`:872`). The design question of §6.1 — is the base wait caused by readers' folds or the writer's
section — is unanswerable from the instrument that exists. With a probe split (throwaway, in a
worktree) at 2r/1w over 10 s in the container: **read holds 247,139 acquisitions, p50 3 µs, p99
127 µs, max 3,167 µs, total 2.82 s; write holds 27,853, p50 31 µs, p99 255 µs, max 4,058 µs,
total 0.98 s; read waits p99 255 µs, max 4,131 µs.** Readers occupy the base 2.9× longer than
the writer; a single reader hold reaches 3 ms (a fold). The combined histogram reports "p99 255,
max 4,131" and says nothing about which. *Repair:* `ENGINE_READ` and `ENGINE_WRITE` scopes,
both in `select nilestream_lockstats`, both reset per level, both printed by `bench` after each
mixed level. *Gives up:* one more row in a table.

### F-10-03 — the reference host's GBS does not compile against the reference host's niles, and the Mac gate has been red since C9-02 landed
*correctness (of the pair) · EV 4×5÷1 = 20 · HI · read from the Mac; reproduced in the container.*
`~/Documents/GBS` is at `963e4d9`; niles is at `8548189`. GBS `963e4d9`'s adapter reads
`rec.epoch` on `nilestream_ledger::segment::Record`, renamed `batch_seq` in C9-02: building it
against the current niles fails with `error[E0609]: no field 'epoch' on type
'&nilestream_ledger::segment::Record'` at `src/lib.rs:177` and `:335`. The cross-repo guard
`downstream_adapter.rs` finds `../GBS` beside the tree and compiles the real adapter, so `make
gate` on the Mac is red on that test today. The author has run `cargo +1.97.1 clippy` at each
landing, which does not run tests, so nothing has said so. The GBS sync block was given at three
landings with the **wrong ref name** (§0.2) — the cycle-9 brief's rule "repeat until confirmed"
was followed and the thing repeated was wrong. **Resolved at 16:10 UTC:** the Mac's GBS is at
`688919c` and pushed. **Recorded green on Host C**: `cargo test --offline -p nilestream-core --test
downstream_adapter` at niles `e168d2d` against GBS `688919c` — `1 passed` in 2.79 s, which is
a real compile of the adapter and not the skip. F-10-03 is closed as a state and stays as a
finding about the interval nobody could see (C9-02 landing → 16:10 UTC) and about the
instruction. What remains of it is F-10-04. The Mac's full workspace run stops at
`numeric_binary_oracle` (no PostgreSQL on 5432) unless `--no-fail-fast` is given; that is the
environment red §5 names.

### F-10-04 — the cross-repo guard passes when there is nothing to check
*instrument-gap · EV 3×5÷1 = 15 · HI · read from source.*
`downstream_adapter.rs:44–58`: with no GBS checkout it prints `SKIPPED: …` and **returns Ok**. On
a host where the sibling is missing the gate is green for a check that did not run — the cycle-6
class ("cannot tell refused from did not run"), in the one test that exists to catch the pair
breaking. *Repair:* skip only under an explicit `NILES_NO_GBS=1`; otherwise fail naming the path
looked for. Both hosts this project runs on have the sibling. *Gives up:* a clone with no GBS
beside it must set one variable to be green.

### F-10-05 — pinned installs land exactly one epoch behind, 98.7% of the time
*negative-turned-positive · EV 4×5÷2 = 10 · HS · measured in the container (deterministic gap
histogram on the served path, 2r/1w, 10 s).*
At every pinned install, `applied − anchor`: **1 → 67,514 (98.7%); 2–3 → 856 (1.3%); 4–7 → 1;
nothing beyond.** (In-process differential, for contrast: spread out to 1,024+, because that
harness has one advancer racing four readers over 24 keys.) On the served path the deferred merge
(§6.2 of the brief; Theorem 4.1 clause 5c; `deferred_merges`, zero today) is **one `deltas_at`
lookup**, not a fold. Cycle 9, Host C: pinned installs are 6.35% of reads at 12r/6w and each is a
reconstruction thrown away after one anchor. *Repair:* C10-03. *Gives up:* under V, one
`deltas_at(applied)` per landing that is one epoch behind — an allocation of that epoch's delta
vector, filtered to one key.

### F-10-06 — 56% of an insert is a reference SHA-256 computed under the exclusive base lock
*guarantee-bounded · EV 3×5÷3 = 5 · HS · measured in the container (callgrind, per function).*
`checked-twice oltp`, 2,000 inserts after seeding: process total 267.7 M instructions;
`nilestream_ledger::chain::Hasher256::compress` **150.7 M (56.3%)**, `finalize` 3.6%, `update`
3.0%; SipHash + `hash_one` 7.0%; malloc/free/`_int_*` ~10%; `Ledger::submit` self 2.4%;
`Session::insert` self 1.0%; `Rev::apply_epoch` 0.6%. The hash is `H(parent ‖ epoch ‖
canon(rows))` in `Ledger::chain` (`ledger.rs:298`), called from `submit` (`:416`) — inside the
write guard `append` holds (`rev_engine.rs:872`). The implementation is the FIPS reference by
design (`chain.rs:27`: no SIMD, no unrolling); a faster hash needs `core::arch` and `unsafe`, or
a crate — both forbidden. What is *not* forbidden is where it runs: `parent` is known the moment
the previous epoch's hash is, so the chain can be computed after B is released, serialised by a
small appender-only lock taken in epoch order. The segment sealer already hashes the same row
bytes a second time (`Record::seal`, over the payload that embeds this hash). *Repair:* C10-05,
below the cut — the container says readers, not the writer, dominate base occupancy (F-10-02),
so this is second. *Gives up:* the in-memory hash lags the append by one hand-off; a reader of
`epochs[e].hash` must wait for it, which today only replay does.

### F-10-07 — Appendix D omits the Z-set API, because a doc comment says `#[cfg(test)]`
*stale-claim · EV 3×5÷1 = 15 · HI · read from source; reproduced.*
`thesis/gen-appendix-d.py:70–74` cuts each file at the first occurrence of the string
`#[cfg(test)]`. `crates/niles-ir/src/eval.rs:11` contains that string **in a `//!` doc comment**
("…rather than living in a `#[cfg(test)]`…"), 930 lines before the real one. Every public item
in `eval.rs` — `pub type Row`, **`pub type ZSet`**, `eval_scalar`, 14 more — is absent from
`thesis/appendix-d-api.md` (0 occurrences of `ZSet`). Across the workspace, **34 public items in 7
files** follow their file's first `#[cfg(test)]` and are silently omitted; `niles-ir` is one of
the three crates the appendix exists to document. Cycle 9's fact 1 was this generator; this is
what it costs. *Repair:* C10-02 — cut at the first `#[cfg(test)]` **at column 0**, or skip
test-attributed items. *Gives up:* nothing.

### F-10-08 — cycle 9's execution report overstates the generation check's second hazard
*stale-claim · EV 2×5÷1 = 10 · HI · read from source.*
`docs/audit/cycle-9/execution-report.md` §5 fact 3 and `rev.rs::finish_fold`'s doc comment say
the generation check prevents "a stale owner clearing a flight record it does not own, which
strands the successor's `Pending` marker with nobody left to publish it". Traced: with the check
removed, the stale owner's `install()` writes `Present(v_old, a_old)` over the marker — the slot
is never left `Pending`, and the successor's waiters are published by the successor's own
`Arc<Completion>`, which it still holds. What is lost is the **successor's fresher result**
(uninstalled) and the stamp (backwards): a cost, not liveness. The report corrected the work order
by one rung and was itself one rung high. Theorem 4.1's clause (5) is unaffected (5a–5c do not
cite the generation; §8.3 below). *Repair:* C10-02 corrects both sentences.

### F-10-09 — `MISMATCH-A9-F05` stands at a rule C9-04.1 repaired; `ignored_windows` survives in prose
*stale-claim · EV 2×5÷1 = 10 · HI · read from source.* `docs/SPEC-ENGINE.md:599` and
`crates/niles-interp/src/lib.rs:401`. *Repair:* C10-02.

### F-10-10 — the plan-cache miss costs ~170,000 instructions, and `point-cold` misses 1,952 of 2,000
*guarantee-bounded · EV 3×4÷3 = 4 · HS · measured in the container.* `checked-twice point-cold`:
`compile_cached` 340.8 M inclusive over 1,952 misses = **174 k instructions per miss**, of which
`parse_program` 206.7 M (35% of the process). `point` (warm, 100 misses) costs 257.5 M against
`point-cold`'s 590.4 M for the same statements. The templated key (C9-11 → C10-10) is what turns
1,952 misses into ~1; this is its baseline, deterministic, and the first one it has had.

### F-10-11 — baselines inferred from documents: one instance, already known
*negative · EV — · swept.* Every pass line in `work-order-9.md` §8 was checked for a figure taken
from `docs/audit/` rather than a transcript. One: C9-06.2, flagged by the work order itself.
C9-05.2's 113,500 and C9-08.1's 14.8× were audit-time container measurements; C9-07.1's "at
most 17 rows" is design-derived (C + 1 at C = 16) and stays a design statement until measured.
Nothing to repair; the rule in §10 of the brief now forbids the class.

### F-10-12 — C9-02.4's witness was not produced, and is worth producing
*instrument-gap · EV 3×3÷2 = 4.5 · HI · read from the cycle-9 report.* The line asked for a
reproduction of the applied-undurable state or a "not reproduced" record naming the invariant.
The invariant is now nameable — admission and sealer prune the same per-transaction queue — and a
test that asserts it at W−1/W/W+1 exists (`the_window_holds_the_last_w_transactions_whatever_
the_batch_size`). What does not exist is the *negative*: a test that retries an identity in the
old gap and shows it is refused by both. *Repair:* C10-02 adds that one test; it is the "not
reproduced" record with teeth.

### F-10-13 — the preflight's storage section measures the wrong syscall on Darwin and reports the reference host as suspicious
*instrument-gap · EV 3×5÷1 = 15 · HI · measured by the author on request.*
`docs/audit/cycle-10/preflight.sh` §B (inherited from cycle 9): Python `os.fdatasync` falls
back to `os.fsync` on Darwin, which does not flush the device; the honest barrier there is
`fcntl(fd, F_FULLFSYNC)`. The probe prints 63,658 barriers/s and the verdict *suspicious* on
Host C, whose real barrier is ~255/s. The mount line resolves to the sealed system volume through
a firmlink. The section that exists to catch a volatile mount cannot tell Host C from one.
*Repair:* C10-02 — `fcntl.fcntl(fd, fcntl.F_FULLFSYNC)` when `sys.platform == "darwin"`, with the
syscall named in the output; the mount line taken from the Data volume (`df -P` of the path's
real parent). *Gives up:* nothing.

---

## 3. Tasks, in dependency order

Every task: branch, what it closes, files, **baseline** (measured, with host), **target lines**
(each a sentence the executor marks `done` / `not done: why` — the §8 checklist is these lines
verbatim), method, acceptance, **guard and the reversion that must make it fail**, guardrails.
Guards are proved in a disposable `git worktree`; a compile error is never the witness.

### C10-00 — the stats query and the append take the two locks in one order (F-10-01) — `c10/01-stats-order`

**Closes:** F-10-01. **Files:** `crates/nilestream-server/src/rev_engine.rs` (`read_stats`, the
`lock_order_tests` module).
**Baseline:** the probe in F-10-01 — zero progress under a 5 s deadline, container.
**Method.** In `read_stats`, read `idem_window_keys` through `self.base()` **before** any view
guard exists and drop it, then take V; or restructure so no function holds both. Generalise the
source guard: enumerate every `fn` in `rev_engine.rs` whose body contains both a base acquisition
(`self.base()`, `TimedRead::acquire`, `TimedWrite::acquire`) and a view acquisition (`VIEW_LOCK`,
`.lock()` on the runtime), and assert base-before-view for each — not a fixed list of two. Add the
behavioural test as a committed one (`a_stats_query_and_a_concurrent_append_do_not_deadlock`,
deadline-bounded, progress-checked) and its wire form: two connections against the daemon's real
bytes, one `INSERT`ing in a loop, one issuing `select nilestream_stats` in a loop, 3 s, both make
progress.
**Targets.**
- C10-00.1 *`read_stats` holds no view guard while it acquires the base, and the lock-order source
  guard enumerates every function in `rev_engine.rs` that takes both locks rather than naming two.*
- C10-00.2 *One connection inserting continuously and one issuing `select nilestream_stats`
  continuously against the daemon's real bytes both make progress for 3 s, tested with a
  deadline; the same in-process with `append` and `read_stats`.*
**Guard/reversion.** Reversion: restore the V-then-B order in `read_stats` → both new tests red
(progress 0 under the deadline) and the generalised source guard red naming `read_stats`.
**Guardrails.** No new lock. The counters `read_stats` reads are relaxed atomics and need no
guard at all; only `rt.view(BALANCE_VIEW)` needs V and only `idem_window_keys` needs B.

### C10-01 — the base lock reports its two holds, and the harness prints them (F-10-02) — `c10/02-base-split`

**Closes:** F-10-02; gives every later task its baseline. **Files:** `lockstats.rs`,
`rev_engine.rs:616–617, 872`, `session.rs` (`nilestream_lockstats` columns), `bank-bench/src/
bin/bench.rs` (print the full lockstats row after each mixed level, level-local), `docs/
BENCHMARK.md`, `docs/audit/cycle-10/hostc/c10-baselock.sh` (exists; prints what the bench prints).
**Baseline:** the container split in F-10-02 (ratios); **Host C: none yet.**
**Method.** `ENGINE_READ` and `ENGINE_WRITE` replace the single `ENGINE_LOCK` (keep the name as
the sum if anything reads it); `TimedRead` records to the first, `TimedWrite` to the second; both
in `select nilestream_lockstats [reset]` with `read_`/`write_` column prefixes; `bench` prints the
row after each mixed level beside the flights line. Then **the author must now run `bash
~/Documents/niles/docs/audit/cycle-10/hostc/c10-baselock.sh --baseline-only` and paste the
output** — that transcript is the baseline for C10-03 and C10-04.
**Targets.**
- C10-01.1 *`select nilestream_lockstats` reports the base lock's shared and exclusive
  acquisitions as separate scopes with their own wait and hold quantiles, both reset by `reset`,
  and a test asserts a `TimedRead` lands in the shared scope only and a `TimedWrite` in the
  exclusive scope only.*
- C10-01.2 *Every mixed level of `bench` prints the full lockstats row, level-local, and
  `c10-baselock.sh --baseline-only` has been run on Host C with its transcript pasted into the
  execution report as the cycle's baseline.*
**Guard/reversion.** Reversion: route `TimedWrite` to the shared scope → the scope test red.
**Guardrails.** `nilestream_stats_names_the_columns_the_benchmark_reads` extended to the new
columns; no histogram is cumulative across levels.

### C10-02 — the gate and the prose stop saying things that are not so (F-10-03, F-10-04, F-10-07, F-10-08, F-10-09, F-10-12) — `c10/03-honest-gate`

**Closes:** the five small stale-claim and instrument findings. **Files:**
`crates/nilestream-core/tests/downstream_adapter.rs`; `thesis/gen-appendix-d.py`;
`thesis/appendix-d-api.md` (regenerated); `docs/SPEC-ENGINE.md:599`;
`crates/niles-interp/src/lib.rs:401`; `crates/nilestream-core/src/rev.rs` (`finish_fold` doc);
`docs/audit/cycle-9/execution-report.md` §5 fact 3 (**a correction appended, the original left
in place** — audit evidence is not rewritten); `crates/nilestream-ledger/src/sequencer.rs` (the
window-gap negative test); `docs/audit/cycle-10/preflight.sh` §B (F-10-13).
**Baseline:** 34 public items in 7 files absent from Appendix D; `ZSet` 0 occurrences.
**Targets.**
- C10-02.1 *`downstream_adapter` fails, naming the path it looked for, when no GBS checkout is
  found and `NILES_NO_GBS` is unset; it skips only when that variable is set; and the author's
  `cargo test --offline --workspace` on the Mac's niles at the landing shows it green against
  `~/Documents/GBS` at `688919c`.*
- C10-02.5 *`preflight.sh` §B issues `F_FULLFSYNC` on Darwin and names the syscall it issued;
  its mount line is the Data volume; run on Host C it reports the barrier at the rate the sink's
  own probe reports, not 63,658/s.*
- C10-02.2 *`gen-appendix-d.py` cuts a file only at a `#[cfg(test)]` that begins a line, and the
  regenerated `appendix-d-api.md` names `ZSet`, `eval_scalar` and every other public item of
  `niles-ir::eval`; a test in the workspace asserts that no `pub` item at column 0 in the three
  documented crates is absent from the generated appendix.*
- C10-02.3 *`MISMATCH-A9-F05` is struck from `SPEC-ENGINE.md` with C9-04.1 cited; no doc comment
  in the workspace names `ignored_windows`; `finish_fold`'s comment and an appended correction
  to the cycle-9 report's fact 3 state the generation check's second hazard as a discarded
  fresher result and a stamp moving backwards, not as an orphaned marker.*
- C10-02.4 *A test retries an identity that the old record-counted window would have admitted
  and the transaction-counted one refuses, and shows both admission and sealer refuse it — the
  "not reproduced" record C9-02.4 asked for, with the invariant named in its message.*
**Guard/reversion.** Reversion for .2: restore `text.find("#[cfg(test)]")` → the appendix test
red on `ZSet`. Reversion for .1: restore the early `return` → a run with `GBS_ROOT=/nonexistent`
and no sibling passes, which the new test's own precondition check turns red. Reversion for .4:
restore `while order.len() > w * 4096` in a worktree → red.
**Guardrails.** `docs/audit/*` is evidence: the correction to the cycle-9 report is an appended,
dated paragraph, never an edit of the original sentence.

### C10-03 — the deferred-delta merge: a flight that lands one epoch behind lands current (F-10-05, LC-38) — `c10/04-deferred-merge`

**Closes:** F-10-05, LC-38; makes `deferred_merges` non-zero. **Files:**
`crates/nilestream-core/src/rev.rs` (`finish_fold`, `install`, `Stats`),
`crates/nilestream-server/src/rev_engine.rs` (`answer_from_view` passes the base),
`thesis/03-theoretical-framework.md` (upquery rule), `thesis/04-novel-contributions.md` (5c).
**Baseline:** container, served path, 2r/1w: 98.7% of pinned installs at gap 1, 1.3% at 2–3,
none beyond 7. Host C, cycle 9: 6.35% of reads pinned, 0.29% joined, 12r/6w. **The Host C
baseline for the pass line is C10-01.2's transcript.**
**Method.** `finish_fold` gains the base: `finish_fold(ticket, value, rows, base: &dyn Base)`.
When `ticket.anchor < self.applied` and `self.applied − ticket.anchor ≤ MERGE_CAP` (start at 8;
the container says 1 covers 98.7%), fold `base.deltas_at(e)` for `e` in `(anchor, applied]`
filtered to `ticket.key` into `value`, install at `applied` **unpinned**, count
`deferred_merges`; beyond the cap, pin as today. The caller holds B across `finish_fold` already
(B < V; the base guard spans `answer_from_view`), so the read is in order. Sound by H-F2 /
Q_lin: it is Theorem 4.1's step (3) applied to the landing value.
**Targets.**
- C10-03.1 *A flight that lands with `applied − anchor ≤ MERGE_CAP` installs at `applied`,
  unpinned, with the key's deltas in `(anchor, applied]` folded in, and `deferred_merges` counts
  it; one that lands further behind installs pinned as before; both proved by the latched
  differential, which now asserts `deferred_merges > 0` on its merged arm and reads exactly at
  `applied` afterwards.*
- C10-03.2 *At 12r/6w on Host C, `pinned_installs` is below 0.5% of reads and `deferred_merges`
  is above 5% of reads, on `c10-baselock.sh --candidate c10/04-deferred-merge` against the
  C10-01.2 baseline; read throughput is reported beside it under the ≥ 10% ∧ ≥ 3 pooled-MAD gate
  and is **not** a pass condition.*
- C10-03.3 *Chapter 3's upquery rule and Theorem 4.1 clause 5c state the merge as implemented,
  including the cap and that a landing beyond it pins.*
**Guard/reversion.** Reversion 1: install at `applied` **without** folding `(anchor, applied]` →
the differential's exact-read assertion red (the divergence C9-06's reversion 1 produced).
Reversion 2: fold but install pinned → `deferred_merges` assertion red. Each red, each
transcript. No timing assertion in a unit test.
**Guardrails.** Under V the merge reads the base the caller already holds; it must not take a
lock. `MERGE_CAP` is a constant with the container histogram beside it, not a flag. The
uninstalled path and the join path are untouched.
**Host C:** *the author must now run `bash ~/Documents/niles/docs/audit/cycle-10/hostc/
c10-baselock.sh --candidate c10/04-deferred-merge` and paste the output.*

### C10-04 — checkpoints in the served daemon, promoted as the base-lock repair (C9-07; F-64, A9-F12, LC-32; F-10-02's reader holds) — `c10/05-checkpoints`

**Closes:** C9-07's three targets and bounds the reader's hold on B. **Files:**
`crates/nilestream-server/src/main.rs` (`--checkpoint-interval N`, default 16, `0` allowed and
banner-stated), `rev_engine.rs` (`seeded` / construction passes it; provenance header),
`proto-engine/src/ledger.rs` (the index Astra corrected: keyed by `(acct, cur)` or the bound
restated), `bank-bench` results headers, `tools/memprobe` (the E18 row `checkpoints_per_posting`),
`thesis/09-evaluation.md` §9.14.1/§9.14.5, `results/E19-scaling.md`, `docs/SPEC-ENGINE.md`.
**Baseline:** Host C `ENGINE_READ` hold p99 / max from C10-01.2 (to be measured); container:
read holds p99 127 µs, max 3,167 µs at C = 0.
**Method.** Cycle 9's C9-07 design, with Astra's two corrections adopted (the scan index is per
account while checkpoints are per (account, currency): key the index by the reconstruction key or
narrow the bound to target-key rows with the adversary case committed; checkpoint memory is
O(K + N/C) and is measured in an E18 row, not assumed). The daemon default is 16; `0` is the
explicit ablation and both `c10-baselock.sh` arms print which they ran at.
**Targets.**
- C10-04.1 *`nilestreamd` without flags serves with `checkpoint_interval = 16`, prints it, carries
  it in every provenance header, and a head read of a 1,024-posting key touches at most 17 base
  rows.*
- C10-04.2 *A `(a, usd)` read on an account with 1,000 `eur` postings and 16 `usd` postings
  visits at most 17 rows — or the thesis bound is restated as target-key rows and the adversary
  case is a committed test that reports the visited count.*
- C10-04.3 *An E18 row measures checkpoint bytes per posting at C = 16, and §9.14.1, §9.14.5 and
  `E19-scaling.md` state the interval their tables were measured at.*
- C10-04.4 *At 12r/6w on Host C, the base lock's shared-hold p99 on `c10-baselock.sh
  --candidate c10/05-checkpoints` is at most half the C10-01.2 baseline's, and its maximum is
  reported beside it; read throughput is reported under the gate and is not a pass condition.*
**Guard/reversion.** Revert the constructor's default → 1,025 rows: red. Revert the index key →
C10-04.2 red. Revert the header → the provenance test red.
**Guardrails.** Definition 3.9 (end-of-epoch checkpoints only) is already tested
(`checkpoint_tests`); it must stay green. The two `c10-baselock.sh` arms differ in the
checkpoint flag **only** for this task and the transcript says so.
**Host C:** *the author must now run `bash ~/Documents/niles/docs/audit/cycle-10/hostc/
c10-baselock.sh --candidate c10/05-checkpoints` and paste the output.*

### ——— cut line ———

C10-00 … C10-04 are the cycle. Below the line, in order, only when all five are green; none is
taken to compensate for an unresolved one above.

### C10-05 — the hash chain leaves the exclusive base hold (F-10-06) — `c10/06-hash-off-lock`
Design as in F-10-06: `submit` assigns `id` and `parent` under B and stores the rows with the
hash pending; the chain is computed after B is released under an appender-only lock **H** taken
in epoch order (H is below B and never held with V; the order becomes O < B < H < P < V < C, S
and F leaves); `append` builds the durable payload after the hash lands; replay verifies exactly
as today. **Baseline:** Host C `ENGINE_WRITE` hold p50/p99 from C10-01.2. **Targets:** the
write-hold p50 halves at 12r/6w (gate applies); the chain is byte-identical on the shipped
determinism fixture (`chain` and `write_rows` streams, C9-12.1's corpus) before and after; the
lock-order guard learns H. **Guard:** hash under B again → the hold assertion is a Host C
measurement, so the unit guard is the determinism fixture plus a source guard that `Ledger::
chain` is not called inside `submit`. Reversion → source guard red.

### C10-06 — a snapshot the fold can hold instead of the base (LC-37) — `c10/07-snapshot-spike`
A **spike, not a landing**: a design note with a measured prototype in a worktree, never merged.
Chunked, append-only epoch storage (`Vec<Arc<[EpochRec]>>`-shaped, chunks sealed at a fixed
size and never moved) and a per-account index that is likewise chunked, so `Base::reconstruct`
can run over an `Arc` snapshot with **no** base guard. Deliverable: the storage layout, the new
`Base` method, the lock order, the E18 cost per epoch and per account, and a container
measurement of reader hold → 0 on the snapshot path. The author decides LC-37 on it.

### C10-07 = C9-08 (holes compact to ⊥; `install` walks the graph; `MISMATCH-A9-F15`) — `c10/08-bounded-view`
### C10-08 = C9-09 (E18 on production structures; the plan-cache trace table) — `c10/09-e18-rows`
### C10-09 = C9-10 (the sentence ledger; every `MISMATCH`/`BLOCKED` in one table; LC-40's generator test generalised to every generator) — `c10/10-prose`
### C10-10 = C9-11 (the templated plan key; baseline F-10-10: 174 k instructions per miss, 1,952 misses of 2,000 on `point-cold`) — `c10/11-plan-key`
### C10-11 = C9-12 (the C republish; still blocked on PostgreSQL 16 on 5433) — `c10/12-republish`
### C10-G01 = C9-G01 (the 1,000-share syndicated payment trace; F-24, F-12, LC-34) — GBS `c10/g01-syndicated`
### C10-12 — `conserve per (…)` and a partition prohibition (LC-41) — a language design note, no code

Each carries cycle 9's task text (`work-order-9.md` §3) unchanged except that **every baseline is
re-derived before its target is carried** (the brief's §6.3), and C10-09 additionally strikes
every marker repaired without being struck.

---

## 4. Branch stacks and merge order

**niles**, from `c10/00-audit` (`8548189`):
```
c10/00-audit
  → c10/01-stats-order      C10-00
  → c10/02-base-split       C10-01   (baseline script run here)
  → c10/03-honest-gate      C10-02
  → c10/04-deferred-merge   C10-03   (Host C two-arm run)
  → c10/05-checkpoints      C10-04   (Host C two-arm run)
  ——— cut ———
  → c10/06-hash-off-lock … c10/12-republish, in order
```
Each branch is a fast-forward of the previous; `c7/01-durable-rows` follows the tip at each
landing, as in every cycle.

**gbs**, from `c7/00-adapter` (`688919c`, **after §0.2**): no GBS commit is required above the
cut. C10-02.1's Mac verification is a run, not a commit. `c10/g01-syndicated` below the cut.

---

## 5. Validation protocol

Per landing, container: `make gate` (starting PostgreSQL first: `sudo -n service postgresql
start`); `cargo test --offline --workspace --no-fail-fast`; the task's guard proved by reversion
in a disposable worktree with the transcript captured; `make reproduce` after commit.
Per landing, Mac: the sync block, then `RUSTUP_AUTO_INSTALL=0 cargo +1.97.1 clippy --offline
--all-targets -- -D warnings`. The Mac cannot run `make gate` in full (no `strace` for
`fsync-proof`; no `valgrind`); its gate is fmt + clippy + `cargo test --offline --workspace` +
`make reproduce`. **Now, and again after C10-02:** `cd ~/Documents/niles && cargo test --offline
--workspace` on the Mac, so the cross-repo guard's verdict on the reference host is recorded
rather than expected.

Green means: exit 0 and no changed verdict against the previous landing's run. A red row is a
result. **Environment reds**, which are recorded and are not defects: `numeric_binary_oracle`
without PostgreSQL started; `downstream_adapter` on a host with no GBS sibling (after C10-02, a
refusal naming the path, which is the intended red).

The Mac preflight was run at 16:32 UTC (§0.2a) and its facts are in §0.3; its §B is wrong on
Darwin (F-10-13) and is repaired by C10-02.5.

---

## 6. Host C scripts

Both in the tree at `docs/audit/cycle-10/hostc/`, both refuse `--publish`, neither writes a
tracked artefact, both print a status per section and exit non-zero if a section did not
complete.

| script | what | runtime | run when |
|---|---|---|---|
| `c10-baselock.sh --baseline-only` | one arm, current build (`c7/01-durable-rows`): 6r3w / 9r5w / 12r6w × 30 s, 2 warm-ups + 5 measured; throughput, slowest-16 with base/view split, flights, and — after C10-01 — the lockstats row with read/write holds | ≈ 12 min / cap 20 | **after C10-01 lands** — this is the cycle's baseline |
| `c10-baselock.sh --candidate <ref>` | two arms interleaved, same protocol | 25–35 min / cap 45 | after C10-03; after C10-04; after C10-05 |
| `c10-rwlock.sh` | Darwin `std::sync::RwLock` fairness: 200 trials, second reader vs queued writer | < 1 min | **run, 16:32 UTC: 200/200 writer-preferring** (§0.2b) |

The `--checkpoint-interval` probe in `c10-baselock.sh` §2 must read the same on both arms except
when C10-04 is the candidate.

---

## 7. The open-questions ledger

**Settled, not reopened:** LC-01, 02, 04, 08–12, 14, 15, 17, 18, 21, 28. **Decided in cycle 9:**
LC-16, LC-35.

| LC | position after this audit |
|---|---|
| **23 / 24** | reopened against the base; **mechanism measured**: readers' holds 2.9× the writer's in aggregate (container); a queued writer blocks later readers **200/200 on Darwin** (Host C, `c10-rwlock.sh`) and 143/200 on Linux; repair is C10-04 (short holds) now and C10-06 (no hold) as a spike |
| 03, 05, 06, 07, 13, 19, 22, 25, 26, 27, 29 | carried unchanged from cycle 9 |
| 20 | `BLOCKED-LC20-definition` — retire the number at the end of this cycle if the author has not supplied a subject |
| **30** | implemented and guarded; **the author confirms in one sentence** or names the change |
| 31 | C10-07 |
| **32** | C10-04: default 16, `0` explicit; not a schema declaration this cycle |
| 33 | C10-08 / C10-10, with F-10-10 as the first measured miss cost |
| 34 | C10-G01, below the cut, unchanged |
| 36 | `BLOCKED-recovery-tip`, the author's contract |
| **37 (new)** | the snapshot: **a storage change, not a guard change** (`Vec` and `HashMap` move under append); C10-06 spike |
| **38 (new)** | the deferred merge: gap = 1 in 98.7%; **fold, capped**; C10-03 |
| **39 (new)** | `pinned_installs / reads` as a phase-diagram input — after C10-03 the number changes by an order of magnitude; decide then |
| **40 (new)** | generator integrity: C10-02.2 for Appendix D now, every generator in C10-09 |
| **41 (new)** | partitioned conservation: the grammar admits `conserve per (k₁, k₂, …)` as a key list (`ast.rs:256`, `resolve.rs:555`), and **adding the partition column to that list is already a cross-partition prohibition** — a transaction must then balance within each `(cur, book)` and cannot move value between books. So CCP segregation (`conserve per (txn, cur, account_class)`) is probably a new *use* of an old spelling, not a new invariant; what the spelling cannot say is the *opposite* — netting permitted across lines A and B but not C. A language note (C10-12) and one GBS row; Astra confirms the reading |
| **42 (new)** | the lock order gains **H** if C10-05 lands: O < B < H < P < V < C; the author confirms |

Markers: `MISMATCH-A9-F05` struck by C10-02; `MISMATCH-daemon-checkpoints`, `-A9-F12` by
C10-04; `-A9-F15` by C10-07; the pre-existing families by C10-09.

---

## 8. Reporting requirements for the executor

One execution report, `docs/audit/cycle-10/execution-report.md`, results only. At the top: the
audit baseline `8548189`, the integration parent, every task tip, both trees' states, the Mac's
GBS head. Evidence class on every figure.

**The checklist — every target line verbatim, with `done` / `not done: why`.**

| id | target |
|---|---|
| C10-00.1 | `read_stats` holds no view guard while it acquires the base, and the lock-order source guard enumerates every function in `rev_engine.rs` that takes both locks rather than naming two. |
| C10-00.2 | One connection inserting continuously and one issuing `select nilestream_stats` continuously against the daemon's real bytes both make progress for 3 s, tested with a deadline; the same in-process with `append` and `read_stats`. |
| C10-01.1 | `select nilestream_lockstats` reports the base lock's shared and exclusive acquisitions as separate scopes with their own wait and hold quantiles, both reset by `reset`, and a test asserts a `TimedRead` lands in the shared scope only and a `TimedWrite` in the exclusive scope only. |
| C10-01.2 | Every mixed level of `bench` prints the full lockstats row, level-local, and `c10-baselock.sh --baseline-only` has been run on Host C with its transcript pasted into the execution report as the cycle's baseline. |
| C10-02.1 | `downstream_adapter` fails, naming the path it looked for, when no GBS checkout is found and `NILES_NO_GBS` is unset; it skips only when that variable is set; and the author's `cargo test --offline --workspace` on the Mac's niles at the landing shows it green against `~/Documents/GBS` at `688919c`. |
| C10-02.2 | `gen-appendix-d.py` cuts a file only at a `#[cfg(test)]` that begins a line, and the regenerated `appendix-d-api.md` names `ZSet`, `eval_scalar` and every other public item of `niles-ir::eval`; a test in the workspace asserts that no `pub` item at column 0 in the three documented crates is absent from the generated appendix. |
| C10-02.3 | `MISMATCH-A9-F05` is struck from `SPEC-ENGINE.md` with C9-04.1 cited; no doc comment in the workspace names `ignored_windows`; `finish_fold`'s comment and an appended correction to the cycle-9 report's fact 3 state the generation check's second hazard as a discarded fresher result and a stamp moving backwards, not as an orphaned marker. |
| C10-02.4 | A test retries an identity that the old record-counted window would have admitted and the transaction-counted one refuses, and shows both admission and sealer refuse it — the "not reproduced" record C9-02.4 asked for, with the invariant named in its message. |
| C10-02.5 | `preflight.sh` §B issues `F_FULLFSYNC` on Darwin and names the syscall it issued; its mount line is the Data volume; run on Host C it reports the barrier at the rate the sink's own probe reports, not 63,658/s. |
| C10-03.1 | A flight that lands with `applied − anchor ≤ MERGE_CAP` installs at `applied`, unpinned, with the key's deltas in `(anchor, applied]` folded in, and `deferred_merges` counts it; one that lands further behind installs pinned as before; both proved by the latched differential, which now asserts `deferred_merges > 0` on its merged arm and reads exactly at `applied` afterwards. |
| C10-03.2 | At 12r/6w on Host C, `pinned_installs` is below 0.5% of reads and `deferred_merges` is above 5% of reads, on `c10-baselock.sh --candidate c10/04-deferred-merge` against the C10-01.2 baseline; read throughput is reported beside it under the ≥ 10% ∧ ≥ 3 pooled-MAD gate and is **not** a pass condition. |
| C10-03.3 | Chapter 3's upquery rule and Theorem 4.1 clause 5c state the merge as implemented, including the cap and that a landing beyond it pins. |
| C10-04.1 | `nilestreamd` without flags serves with `checkpoint_interval = 16`, prints it, carries it in every provenance header, and a head read of a 1,024-posting key touches at most 17 base rows. |
| C10-04.2 | A `(a, usd)` read on an account with 1,000 `eur` postings and 16 `usd` postings visits at most 17 rows — or the thesis bound is restated as target-key rows and the adversary case is a committed test that reports the visited count. |
| C10-04.3 | An E18 row measures checkpoint bytes per posting at C = 16, and §9.14.1, §9.14.5 and `E19-scaling.md` state the interval their tables were measured at. |
| C10-04.4 | At 12r/6w on Host C, the base lock's shared-hold p99 on `c10-baselock.sh --candidate c10/05-checkpoints` is at most half the C10-01.2 baseline's, and its maximum is reported beside it; read throughput is reported under the gate and is not a pass condition. |
| — | *cut line* |
| C10-05 … C10-12, C10-G01 | cycle 9's target lines for C9-07 … C9-12 and C9-G01, verbatim from `work-order-9.md` §8, plus C10-05's three and C10-06's deliverable list above |

**Also required.** (1) The gate matrix per landing, container and Mac, with the Mac's `make gate`
result after C10-02. (2) Every guard transcript: worktree, SHA, reverted lines, red assertion,
restored green, exit codes. (3) Timing tables with a host column; the two `c10-baselock.sh`
transcripts in full; E18 and RSS never in one column. (4) The updated MISMATCH/BLOCKED table and
LC ledger with the author's answers to LC-30, LC-37, LC-42. (5) **Exactly three material facts
found while executing that this work order does not cover** — nothing from §2 recycled; fewer
than three reported as `not done: only N`. (6) Worktree status before and after with the five
protected files' sizes (10,244 / 16,639 / 6,148 / 8,196 / 816,110 at audit; a discrepancy is
referred, never corrected); every SHA and bundle hash, base and tip; the sync block for every
landing, **naming the ref the bundle carries (`git bundle list-heads` first), never
`branch -f`-ing the checked-out branch, and repeated in every subsequent landing's message until
the author's paste shows it run**; the author's 1.97.1 result. No executor push; attribution is the executing session's own.

---

## 9. What this audit could not do, and says so

- **GitHub:** unreachable without a credential; not requested; not needed for any finding.
- **Host C:** the preflight and the RwLock probe have been run (§0.2a, §0.2b); no throughput
  or lock-hold measurement of the current build exists yet, and every such figure here is cycle
  9's. The first thing the executor lands (C10-01) is the instrument, and the first thing the
  author runs after it is the baseline.
- **Astra:** the §8 checks 2, 3, 5 and 6 were done here from source and are for Astra to do
  independently; the reconciliation table (cycle 9's §2A shape) is the author's to commission.
- **The three material facts** of cycle 9 were each turned into a finding here (F-10-07, the
  PostgreSQL row in §0.1, F-10-08); none was recycled as new.
