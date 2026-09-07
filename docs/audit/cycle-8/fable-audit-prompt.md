# Cycle 8 audit brief — **Fable**

**You are auditing, not building.** Your single deliverable is a **work order**: a prioritised,
evidence-backed set of tasks that Claude Opus will execute in a later session. You write no
production code into either repository. Probe crates outside the trees, benchmark runs, test suites,
`strace`/`callgrind`/`dhat`/`massif`, throwaway scripts — all encouraged. Commits are not.

The work order you produce is the only thing the executing agent sees. Assume it has none of your
context: every task carries its own preconditions, baseline, target, method, acceptance test, guard,
and guardrails, and must be executable **offline**.

**You are the only auditor this cycle.** Cycle 7 was meant to run two audits — yours empirical, GPT
6 Astra's structural — and Astra's failed on a content classifier before it read a file. You ran both
halves, and the two briefs were consolidated into one work order. This cycle you own both halves from
the start: **claims-versus-code, the type system, the thesis text, GBS layering and the bootstrap are
yours as much as the locks, barriers and latency distributions are.** Where the two halves disagree —
a measurement that contradicts a sentence — that disagreement is the finding.

---

## 1. The three goals, and what "efficient" has to mean for each

The stated end goals are **a highly efficient query language (Niles)**, **a highly efficient query
engine (Nilestream)**, and **a highly efficient core-banking system (GBS)** — efficient in **memory
and speed**, poised to replace systems of the class of Loan IQ and Calypso, without surrendering any
commitment in §10.

Three framing questions, each to be answered with numbers or with a stated reason numbers cannot
yet be had:

- **Nilestream: is the next ceiling structural or another implementation defect?** Four cycles have
  found implementation ceilings — a lock, a barrier placement, a `&mut` taken for a counter, and in
  cycle 7 a read that reported the wrong end of its certification interval so that ~46% of keyed
  reads were discarded and rebuilt. Each was correct on every input and wrong in structure. Say
  whether what remains is that class again, or the design.
- **Niles: has anything been measured about the language at all?** Cycle 7 touched the compiler only
  to fix two lints. `callgrind` says compilation was 96% of the daemon's own work on one workload
  (`session.rs`, the plan-cache note) — that was a *daemon* finding. No cycle has asked what the
  compiler costs per statement class, whether the typed IR is the size it needs to be, or whether
  `niles-interp`'s bootstrap gates measure anything. Decide what an efficiency claim about a
  *language* would even be evidence of, and whether one is admissible yet.
- **GBS: is an efficiency task admissible yet?** Cycle 7's answer was no: with no product trace
  (F-24) and lifecycles that do not survive a restart (F-12), the only honest GBS task was the one
  making measurement possible, and it is below the cut line still (T-10). Re-decide with the
  cycle-7 tree in front of you. A refusal with a reason is a finding.

---

## 2. Access

### 2.1 GitHub

Both repositories are private, owned by **`shbiebs`**:

- `https://github.com/shbiebs/niles`
- `https://github.com/shbiebs/gbs`

**If you do not have credentials, ask, and ask for the narrowest thing that works.** Read access
only:

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
credentials are refused, audit what you can reach and say in your output which parts went unread.

### 2.2 The author's Mac — and the thing that will catch you out

You are authorised to audit the working copies at `/Users/checolino/Documents/niles` and
`/Users/checolino/Documents/GBS`. Both are at the heads in §4 and clean apart from the four untracked
files below.

**If you reach the Mac through the Claude desktop bridge, you do not get macOS.** You get an isolated
**Linux VM, aarch64, with `~/Documents` mounted over FUSE, and no `rustc`, no `cargo`, no `psql`,
no `valgrind`.** It is fine for reading, grepping and counting. It is **useless for storage
measurement** and it cannot build the workspace. Cycle 7 confirmed this again: the executor tried to
run `cargo +1.97.1 clippy` there and had to ask the author instead.

**Everything that needs the real machine is packaged as a shell script for the author to run in
Terminal.app.** The convention is `~/Documents/niles-hostc/<name>.sh`; `run4.sh` and `run5.sh` exist
and their results are under `~/Documents/niles-hostc/results4-*/` and `results5-*/`. Follow it, and:

- Say **explicitly, at the point in your work order where it is needed**: "the author must now run
  `bash ~/Documents/niles-hostc/<name>.sh` and paste the output." Do not bury it.
- Make each script **idempotent, non-interactive, bounded in time**, with the expected runtime
  stated; results under `~/Documents/niles-hostc/results-<stamp>/` and a compact summary to stdout.
- **Never `chmod +x` and assume** — always instruct `bash <path>`.
- The author's zsh has `interactive_comments` **off**: a `#` at the prompt is a command. No comments
  in pasted lines.
- **A script's worktree must be retargeted every run, and the script must refuse to measure if the
  retarget did not take.** `run4.sh` created its worktrees on first use, reused the directories, and
  pinned arm A to a *branch name* — so the re-run cycle 7 asked for "after T-01 and T-02" silently
  re-measured the commit *before* T-00 and produced a complete, plausible set of numbers about the
  wrong code. It was caught because the mixed probe prints every column the server returns by name,
  and the column T-02 added was not there. `run4.sh` now derives arm A from `HEAD` and refuses on
  mismatch. **Every script you write inherits that rule.**

The real Mac has Rust (`~/.cargo`, `~/.rustup`, **1.95.0 pinned** by `rust-toolchain.toml`, with
1.97.1 also installed), PostgreSQL, and **10 cores (4P + 6E)** on Apple M4 with APFS, where every
barrier is `F_FULLFSYNC` at **~255/s**. It is **Host C, the reference host** (T-04): the only machine
in this project that gives the same answer twice.

**Read-only discipline on the Mac trees.** Do not commit, stage, `git clean`, `git gc`, or check out
a different branch without saying so first. **Four untracked files in `niles` must never be edited,
staged, deleted, moved or bundled:** `.DS_Store` (10,244 B), `AGENTS.md` (16,639 B),
`thesis/.DS_Store` (8,196 B), `thesis/Niles-Thesis.pdf` (816,110 B). Report their sizes at the end.

**Two bridge traps from cycle 7.** Any `git` command through the bridge that touches the index
leaves a `.git/index.lock` the bridge cannot delete — use `git --no-optional-locks` for reads, and if
one appears, tell the author to `rm ~/Documents/niles/.git/index.lock`. And `git -C` on a worktree
fails through the bridge because worktree metadata records `/Users/...` paths the mount does not
have; read worktrees from the main tree's `git worktree list` instead.

---

## 3. Hosts, and what each may conclude

| host | what it is | admissible |
|---|---|---|
| **A** | 2-core Linux container, ext4 on virtio, `fdatasync` 4,961–5,825/s, rustc 1.95.0 as `stable` only | counters, within-host ratios; **cannot install a toolchain by version** — build with `RUSTUP_TOOLCHAIN=stable` |
| **B** | the desktop bridge: aarch64 Linux VM, FUSE, no toolchain | reading, counting; **no storage or build evidence** |
| **C** | the Mac: Apple M4, **10 cores**, APFS, `F_FULLFSYNC` **255/s**; **the reference host** | connection curves, RSS under load, second-architecture runs, anything needing a real barrier or >3 cores, **every contract figure** |
| **D** | your own container, if on overlayfs `fsync=volatile` | **nothing about durability** |

The rules from cycle 7 stand: no absolute figure beside another host's without a host column; every
wall-clock threshold derived on the executing host in the same session; deterministic counters gate
first; wall clock needs two warm-ups and **≥5 measured runs**, arms interleaved, median/MAD/range,
a gate firing only on **≥10% and ≥3× the pooled MAD**, otherwise `noise-limited`; `unsupported`,
`blocked`, `not run`, `noise-limited` are results. **A contract ratio names its host, instance
(boot id, not hostname), session, barrier and commit, or it is not a contract ratio** — T-04 made
every E16 document carry that header, and the reason is in §9.

**Run the preflight first, in whatever container you audit from, and paste its output at the top of
your work order.** It is at `docs/audit/cycle-8/preflight.sh` (identical to cycle 7's):

```
bash niles/docs/audit/cycle-8/preflight.sh /path/to/niles /path/to/gbs
```

`nproc` is not your core count — read `/sys/fs/cgroup/cpu.max`. A barrier above ~100,000/s is a
mount option. `make fsync-proof` passes on such a mount too; the two checks are necessary together.

**Before you write a probe, check whether a prior cycle built it.** Now present: `nilestreamd
--durable`; `bench --run --scaling-only --nls-only` with the **`mixed` level** (T-03) reporting
fallback rate, `max_batch`, lock wait p99 and `base_epochs`; `select nilestream_stats` with
**`view_answers` and `fallbacks`** (T-02); `select nilestream_sealer`; `make fsync-proof`; a
**crash-protocol `#[test]`** that spawns the shipped binary and `SIGKILL`s it
(`nilestream-server/tests/crash_recovery.rs`); a **concurrent oracle differential**
(`nilestream-core`, `concurrent_differential`); and `probes/mixed`, `probes/crash`, `probes/mem`
under `docs/audit/cycle-7/probes/`.

---

## 4. Exactly where the trees stand

**Niles** — `master` untouched; the cycle-6 stack ends at `c6/audit-cycle-7` = `cea8a1e`; cycle 7 is
one stacked branch on top of it. All branches are on GitHub.

| branch | head | what |
|---|---|---|
| `c7/00-adapter-check` | `2afb56e` | T-00 alone, as its own reviewable unit |
| **`c7/01-durable-rows`** | **`9ae1319`** | T-00 through T-04a, the closures, and the records below |

The eleven commits on `c7/01-durable-rows`, oldest first: `2afb56e` T-00 · `9318847` T-01 ·
`6d00f94` T-01b · `ec34783` T-02 · `8d229ea` T-02 follow-up (postcondition, adapter
`--all-targets`) · `0835b1a` run-4 record · `a7e5a74` T-03 · `5d37692` T-04 · `1f9c38a` T-04a ·
`1d0574c` `MISMATCH-durability-restart` resolved · `9ae1319` three acceptance lines closed.

**GBS** — `c7/00-adapter` at **`febb738`**: `8c814f6` T-00 · `9d398d7` the as-of limit removed and
two breaks the adapter check missed · `3eaaae0` toolchain pin · `febb738` a third 1.97.1 lint.

**Gate at `9ae1319` / `febb738`:** `cargo fmt --check` green; `cargo clippy --all-targets -D warnings`
green under **both 1.95.0 and 1.97.1, both repositories** — verified by the author on the Mac;
Niles **841 test functions, 62 suites, 0 failed**; GBS `make gate` 29 rows, 0 failures; `make
reproduce` exit 0 with a clean diff; `make fsync-proof` green; the downstream adapter check green
with `--all-targets`. `numeric_binary_oracle` and `crash_recovery` need `psql`; the first also needs
a running PostgreSQL with a `bench` role.

**Toolchain:** `rust-toolchain.toml` in both repositories pins `channel = "1.95.0"` (the author set
it; T-04a). A container with no egress to `static.rust-lang.org` cannot install a channel by version
and **must build with `RUSTUP_TOOLCHAIN=stable`**, which the file says. Without that a fresh
container fails on a download it cannot make.

Records for every task are under `docs/audit/cycle-7/`: `run4-after-t02.md`, `t03-mixed-row.md`,
`t04-contract-provenance.md`, and the two work-order files. The requirements checklist — every §5
line of work order 7 with `done` / `not done: why` — is in the project as
`cycle-7-requirements-checklist.md`. **Read it first.** It is the document F-32 exists because
nobody wrote, and it names what was dropped.

---

## 5. What cycle 7 changed, and its measurements

None of this has been reviewed by anyone but the agent that wrote it. Audit it.

**T-00** — a downstream adapter check: `nilestream-core/tests/downstream_adapter.rs` builds GBS's
adapter when `GBS_ROOT` or a sibling checkout is present, **skips by name** when it is not, and
tells a compiler error from a cargo failure. It shipped building the *library* only and was green
while GBS's own gate was red twice (a `Mutex` where T-06 put an `RwLock`; two stale `mut`s). It
builds `--all-targets` now. `SPEC-ENGINE.md` Part III½ names the two traits implemented out of tree.

**T-01** — the durable record was `epoch.to_string()`. It is now `parent ‖ hash ‖ canon(rows)`;
`with_durable` replays every recovered transaction through `Ledger::submit` and **verifies** the
recomputed chain link against the record, refusing at the first mismatch; a barrier failure is
**fail-stop** (LC-21, decided by the author); a session observes `frontier()` and never the epoch it
just applied. Same session, 16 connections × 300 transactions on A: 972 → 640 fsyncs, **4.94 → 7.50
transactions per barrier** — the larger record batches *better*. Out of process: 8 writers, 25,416
acks, `SIGKILL`, reopen — every account recovered at or above its acks, the retry refused as
`Duplicate` by the daemon. **Two defects in the first cut, both caught by gates and not by review**:
`chain` read the epoch number off `self.epochs.len()`, correct when sealing and wrong in
`verify_chain`, so **E1's chain column read FAIL on all five seeds while the workspace was green** —
`verify_chain` had no unit test; and the chain built a `Vec` only to hash it, putting two E18
scenarios over budget. `write_rows` streams into the hasher; both scenarios returned to *exactly*
their prior figures.

**T-01b** — a currency's wire code is its position among the schema's `currency` declarations. An
undeclared currency is refused at ingress (`22023`); `sum(amt) group by acct` over a multi-currency
base is refused (`22000`) rather than folded, naming both remedies; `explain` reports
`refused-cross-currency`. In the closures: a decimal `amt` is accepted at the currency's declared
scale and refused beyond it (`22003`), *rather than rounded*.

**T-02** — `Rev::read` returned `anchor: effective` and every caller in the project had written the
same branch: *if the anchor came back different, throw the answer away and rebuild*. It returns the
anchor asked for whenever `stamp ≤ anchor ≤ effective`. **Over the wire on Host C**, three shapes:

| shape | fallback before | after | base rows folded |
|---|--:|--:|--:|
| 4 readers, 2 writers | 45.8% | **0.2%** | 20.6M → 7.9M (−61%) |
| 8 readers, 4 writers | 43.7% | **0.2%** | 42.9M → 16.6M (−61%) |
| 8 readers, 1 writer | 45.3% | **0.2%** | 20.8M → 1.7M (−92%) |

The "before" column is **derived** — the counter did not exist to take it — from the excess of
reported `reads` over queries issued under the old double-counting; three shapes agree without a
reason to. **Wall clock did not move** (−0.4%, +3.9%, +0.3%; p50 23 µs both sides): a keyed fallback
folds ~22 rows through the anchor index against a ~47 µs round trip. Counted work fell 61–92% and a
wire benchmark could not see it, which is how F-27 survived three cycles. The lower bound closed a
**correctness** hazard: `anchor < stamp` was served with `effective` attached, safe only because
callers discarded it. GBS's `as_of_reconstructions` branch — documented in `G3-verdict.md` as a
standing limit — is unreachable now and its meaning is inverted to "upstream regressed".

**T-03** — E19's `mixed` level: readers N, writers ⌈N/2⌉, both roles for `--mixed-seconds`. The
renderer **refuses** a row without a fallback rate; the median uses `unwrap_or(0.0)` deliberately so
that deleting the refusal produces the *wrong publishable row* and not a crash. **A mixed level's
read rate is a function of accumulated history** — 23,232 → 18,998 → 12,208 reads/s across 6,925 →
20,436 → 38,466 sealed epochs with nothing about the engine changed — so the row carries
`base_epochs`. Against `probes/mixed` the *shape* reproduces; the absolute read rate is **21% apart
at matched base size, outside either instrument's MAD**, and the residual is warm state.

**T-04** — every E16 document carries host, instance (boot id), session, barrier and the commit that
built the binary — stamped at **build** time by `build.rs`, since `git rev-parse` at run time labels
a binary with whatever the tree is checked out at when invoked. `--baseline <commit>` records which
A/B a run belongs to and does not execute it. **Host C is the reference.** The F-29 three-arm table is
recorded; `MISMATCH-contract-instance` is resolved.

**T-04a** — two 1.97.1 lints in `niles-ir` fixed at the source; **a third in GBS that F-33 never
named**, because `run5.sh` §D only ever ran clippy in the Niles tree. Both toolchains, both
repositories, green.

**The run-4 re-run** (Host C, `8d229ea`): durable **7.80×** at 16 connections against a target of
within 10% of 7.80×; F-28 reproduced (fold 1.83× → 4.57×, point 95.8k → 138.8k/s); F-34 holds
(write p50 3,991 → 3,932 µs at 8:1). Peak RSS 85.4 → 79.2 MiB.

---

## 6. Where your budget should go

### 6.1 Below the cut line — the tasks work order 7 specified and did not reach

Re-audit each against the cycle-7 tree and either carry it with a fresh baseline or close it with a
measurement showing it is moot:

- **T-05 — window and budget (F-43, F-44).** `idem: IdemKey window 30.days` reaches no crate below
  the compiler and the window is infinite; the eviction budget bounds resident values but not the
  two per-key metadata maps (`reads_of`, `last_read`). Nobody has measured either cost as history
  accumulates. **Measure before specifying** — the idempotency index at 1M / 10M identities, and the
  metadata maps at 2× budget distinct keys.
- **T-06 — say what is true (F-42 remainder, F-38 docs, F-30 guard).** T-01 fixed the banner and the
  README's placeholder-hasher row. Not done: `lockstats.rs`'s header, the four comments, `thesis/07:69`,
  and a source test that the banner names no mutex. **And three new stale claims from cycle 7 itself
  are in §9.**
- **T-07 — the determinism fixture (F-46).** E1's rendered output hashed on x86_64 and asserted on
  arm64. Nothing cross-architecture has been asserted yet, and now there is a second SHA-256 stream
  (`write_rows`) whose bytes must be identical on both.
- **T-08 — the fold plateau on C (F-28's residual).** 4.4–4.6× at 8–16 connections on ten cores:
  the four performance cores, or `report_from_view`'s V hold? `fold` p99 at 16 connections is also
  **unstable** — 101,551 µs in one run against 17,017 and 24,795 µs in its siblings, same spread the
  prior run showed. Attribute both.
- **T-09 — E24: RSS under load at 20k / 200k / 2M on C**, via a `run6.sh`.
- **T-10 — GBS lifecycle and holds on the ledger (F-12)**, sharing T-01's encoding discipline.

### 6.2 The tail nobody has attributed

Mixed-phase read **max** is 12–13 ms at every shape on Host C, before and after T-02, against
431–756 µs readers-alone; p50 and p99 are unaffected. A small number of reads wait on something —
most likely the base write guard held across an apply, since `append` applies rows and advances the
view under the exclusive guard. That is LC-18's neighbourhood and it is unmeasured. **Attribute it**
with a lock histogram at finer buckets or a per-read trace, on C.

### 6.3 What the counter changed underneath the history

`nilestream_stats.reads` no longer double-counts a fallback (T-02). **Every prior `reads`, `hits`
and `misses` figure in `results/` and the thesis was produced under the old counting**, in which a
discarded answer was a hit and a fallback was two reads. Decide which published figures are
invalidated, and whether the E16/E19 committed files need re-measuring on C under the new meaning
before anything compares against them.

### 6.4 The thesis has not been reconciled since cycle 6

Chapters 4, 6 and 9 and `SPEC-ENGINE.md` were edited by the executing agent in cycles 6 and 7 and
checked by nobody. `thesis_drift` guards generated blocks, not prose. Specifically: does chapter 4's
Theorem 4.1 say what `Rev::read` now does (the certification interval as a *postcondition*, every
answer stamped with the anchor it was asked for)? Does §9.14.1's new prose agree with the E16 header
it describes? Does chapter 6's durability sentence — now marked resolved — match `with_durable`'s
actual refusal behaviour?

### 6.5 Niles, the language, has never been on the bench

See §1. At minimum: what a statement costs to compile per class (point, fold, report, insert) in
instructions; whether the plan cache (`extended.rs`, epoch-keyed) is invalidated correctly and what
it costs under thousands of distinct statements; whether the typed IR's verifier is on the hot path;
and whether Appendix E's bootstrap gates assert anything a regression would fail.

### 6.6 GBS after the as-of change

`as_of_reconstructions` should now read zero everywhere. Confirm across the whole `G3` sweep, and
audit whether any *other* GBS branch compensates for an upstream behaviour that has since changed —
the pattern was "a limit documented downstream that was a defect upstream", and it is unlikely to
have had one instance.

---

## 7. Where *not* to spend the cycle

Refuted, some twice. Reopen only with evidence that overturns the measurement: compile time,
startup, binary size, idle RSS; connection memory at this scale; reply streaming (except a
slow-consumer measurement, still unmade); crypto dependencies; point-read sharding unless T-08's
attribution says V; **the mixed row's absolute agreement with `probes/mixed`** — the 21% is warm
state and matching it would change what the other E19 levels measure.

---

## 8. Check these regardless

1. **Is `Rev::read`'s postcondition — every answer carries the anchor it was asked for — actually
   sufficient for LC-19**, strict serializability at the wire, now that no caller checks?
2. **Is the crash-protocol test the audit's protocol?** It drives ~960 acknowledged transfers
   through **one** `psql` session, serially. The audit's protocol used **eight concurrent writers**,
   so group commit formed batches and a kill could land mid-batch. The test is weaker on exactly the
   case durable-before-visible is about. Decide whether it needs N concurrent sessions.
3. **Is every published number now stale** after T-01 (record size), T-02 (counter semantics), and
   the toolchain pin (different codegen than the figures were taken under)?
4. **Does the derived "before" fallback column stand?** It rests on one reading of how the old
   `read_stats` summed its counters. Re-derive it from the old code (`cea8a1e`) rather than from the
   commit message.

---

## 9. Potential misses in every cycle so far

Confirm or refute each with evidence. Cycle 7's are first because they are the freshest and because
they share one shape: **a check that cannot fail is not a check.**

1. **The adapter check was green and wrong** — library only, while GBS's own gate was red. Fixed;
   ask what *else* is checked at a narrower scope than the thing it claims to cover.
2. **`verify_chain` had no unit test.** The only thing exercising it was a ten-minute experiment
   reporting through a generated table. It broke for every epoch on every seed and the workspace
   stayed green. **Enumerate every function whose only check is an experiment.**
3. **`run4.sh` measured the wrong commit for a whole cycle**, and every number it produced was
   plausible. Audit `run5.sh` and every other script for the same reuse pattern.
4. **F-33's scope was one repository.** "Green on both toolchains" was never a claim about GBS.
   Audit every cross-repository claim for the same asymmetry.
5. **`probes/mixed`'s `FOLD_FACTOR` proxy was structurally inert.** It reported 0.03% fold-shaped
   reads while ~46% of keyed reads were falling back, because a keyed fallback is ~22 rows through
   the anchor index — nowhere near 20× the p50. An audit instrument that could not see the thing it
   was built for. **What other proxy in this project measures something adjacent to its claim?**
6. **Host A's 87.4% was reported as if universal.** Host C measured ~46% before T-02. A finding
   stated without its host was carried into a work order as a target ("fails at ~87%").
7. **A mixed row's read rate depends on accumulated history** (1.9× spread with no engine change).
   Any prior comparison of two throughput figures at different base sizes is suspect.
8. **`bench.rs` still hard-codes "> This host has 2 cores" into every E19 document** — including the
   ones produced on Host C's ten. A T-06-class defect introduced under the executor's nose, and the
   T-04 provenance header sits a page above it. The core count is in the preflight; it should be in
   the document.
9. **`MONEY_SCALE = 2` is still hard-coded for the *reported* numeric type modifier**
   (`session.rs:970`). The closures made *parsing* scale-aware; the wire still tells every client
   every money column is scale 2. A schema declaring `currency jpy { scale: 0 }` parses correctly
   and reports wrongly.
10. **`SPEC-ENGINE.md` Part III½ says `Serving` has no out-of-tree implementor** — while GBS's
    `over_the_wire.rs` assembles the daemon from `accept_loop` and `RwLock<RevEngine>`, which is why
    it broke. The table is stale one cycle after it was written.
11. **A guard that did not fire was nearly taken as a pass.** `minor_units` had two paths to one
    answer; deleting the check changed nothing because an underflow produced the same error.
    **Every guard in cycle 7 was proved by in-place reversion, not in a disposable worktree**, and one
    reversion silently failed to apply. Both are in the checklist as not done.
12. **Three acceptance lines were dropped from tasks reported done**, and found only because the
    §10 checklist was written. They are closed now; the class is not.
13. **The executor's tooling ate string literals.** Python heredocs with `\` line continuations
    collapsed Rust string literals three times; one shipped to a running daemon before being caught
    (`DETAIL:` text with runs of spaces). Instruct the executor to edit by line or with a real editor
    tool, and to `grep` for collapsed literals before every commit.
14. **A claim can be true of a component and false of the system containing it** —
    `MISMATCH-durability-restart` was true of `nilestream-ledger` and false of the daemon for three
    cycles; GBS's as-of limit was a downstream description of an upstream defect. Look for the class.

From earlier cycles, still open or partially so: crash recovery under **concurrent** writers with
group commit mid-batch (§8.2); the eviction budget has one shape; the idempotency window is unbounded
and unmeasured (T-05); the plan cache is untested under diversity; cross-target claims exceed
cross-target evidence (Appendix C: five targets, two exercised); the bootstrap has gates but no audit;
slow-consumer behaviour is unmeasured; `no unsafe` ≠ data-race freedom against the relaxed counters,
the `AcqRel` frontier and `fetch_max`; GBS has no efficiency instrument; E24 is unbuilt.

---

## 10. Constraints the executing agent is bound by — carry them into the work order

Non-negotiable; no task weakens one without an explicit decision routed to the author:

durable-before-visible; one sealer per ledger; one total epoch order; full retention; honest absence;
fold-never-field; product purity; self-describing amounts; GBS layering; **no `unsafe`**; **no
default-on-error**; **zero external dependencies**; no fabricated results; the thesis follows the
code with `MISMATCH-<id>` at every divergence; a red test is a result and is never fixed by
deletion; honest refusal over silent fallback; `BLOCKED-<id>` on ambiguity; apply-before-publish;
the lock order **O < B < P < V < C**, S a leaf; every optimisation ships its guard in the same commit
and the executor **proves the guard fails on the reverted change in a disposable `git worktree`**,
transcript in the report; no performance claim without a reproducible command and a provenance
header; benchmarks touch committed artefacts only with `--publish`, never from a container;
micro-benchmark gains are never contract results; **a currency's wire code is its declaration
index**; **`Rev::read` answers at the anchor it was asked for** — no caller may reintroduce a
compensating branch.

Environment: **no task may require network access at execution time; no task installs anything.**
Rust 1.95.0 (`RUSTUP_TOOLCHAIN=stable` where the pin cannot resolve), valgrind 3.22, `psql` and a
startable PostgreSQL 16 are preconditions. Attribution is taken from the executing session's own
instructions; never bake in a trailer.

**Sync at the end.** The executor has no remote and cannot push. It ships each commit as a `git
bundle` written to `~/Documents/niles-sync/cycle-8/` on the Mac, and tells the author — **at the
moment it is needed, not at the end** — exactly what to run:

```
cd ~/Documents/niles
git fetch ~/Documents/niles-sync/cycle-8/<bundle> c8/<branch>
git merge --ff-only FETCH_HEAD && git push origin c8/<branch>
```

`merge --ff-only` and not a branch-to-branch fetch, because the branch is checked out on the Mac. A
local uncommitted edit the author made is discarded with `git checkout -- <file>` only when the
incoming commit contains it byte for byte, and the executor says so.

---

## 11. Open questions to carry, restate or close

Settled, **not to be reopened**: LC-01, LC-02, LC-04, LC-08, LC-09, LC-10, LC-11, LC-12, LC-14
(retired), LC-15, **LC-21 (fail-stop; the author)**.

Carried from cycle 7, restate or close with evidence: **LC-03** `AccountId` String→u64; **LC-05**
rendered-NULL and refusal boundaries; **LC-06** the process/allocator boundary for memory ratios;
**LC-07** E23's slope; **LC-13** release checks; **LC-16** durable by default; **LC-19** what the
reader-writer split and T-02's postcondition together claim at the wire; **LC-20** not decided.

**Close with evidence, proposed:** **LC-17** — the anchor mismatch no longer occurs; the branch is
unreachable in both consumers (T-02). **LC-18** — no writer starvation at 8:1 on C (p50 flat, p99
1.5×); the RwLock stays, and §6.2's tail is the remaining question.

New, from cycle 7 — raise, do not answer alone:

- **LC-22** — should the wire report the schema's declared scale per money column (§9.9), and what
  does that do to `psql_conformance`'s golden output?
- **LC-23** — the 12–13 ms read tail (§6.2): attribute before deciding whether the base write guard
  should be held across the apply.
- **LC-24** — the fold p99 instability at 16 connections on C (§6.1 T-08).
- **LC-25** — should the crash test drive N concurrent sessions (§8.2)? It costs runtime in every
  `cargo test`.
- **LC-26** — a same-session A/B script for Host C: T-04 documented the two-arm invocation and no
  script runs both arms. Without one, "MET" on C is still one arm at a time.
- **LC-27** — which committed `results/` figures the T-02 counter change invalidates (§6.3), and
  whether they are re-measured or marked.

---

## 12. What you must produce

A single work order containing:

0. **The preflight output, verbatim, with its admissibility table filled in.**
1. **An executive judgement** — where the three artefacts stand against the three efficiency goals,
   with the arithmetic for any reachability claim spelled out, and an explicit answer to each of
   §1's three questions.
2. **Findings**, each with a class (wrong-measurement, correctness, liveness, guarantee-bounded,
   instrument-gap, negative, **stale-claim**), an **EV score** (`impact × confidence ÷ cost`, each
   1–5, ties favour correctness), **HI** or **HS**, evidence with file and line, the measured cost,
   the proposed repair, and **what the repair gives up**. Negative findings are first-class.
3. **A task list in dependency order** — each task carrying what it closes, its files, its
   **baseline measured on the executing host**, its **target as a ratio**, its method, its
   acceptance test, its **guard and the reversion that must make it fail**, and its guardrails. Mark
   a **cut line** for one agent in one cycle. **Every target line must be a sentence the executor can
   mark `done` or `not done: why`** — the §10 checklist is copied from it verbatim.
4. **Branch stacks and merge order** for both repositories from `9ae1319` / `febb738`.
5. **A validation protocol**: commands, what green means, what a red row means.
6. **Every Host C script**, named, with its expected runtime, its worktree-retarget-and-refuse
   clause, and an explicit "**the author must run this now**" marker where its result is needed.
7. **The open-questions ledger** (§11), updated.
8. **Reporting requirements** for the executing agent: the **§10 checklist with every target line
   verbatim and its status**; guard transcripts naming the disposable worktree; worktree status
   separating the four untracked files from task changes; the SHAs; **exactly three material facts
   found while executing that the work order did not cover**, negative findings included; and a
   sync section listing every bundle and the author's commands.

Order by `impact × confidence ÷ cost`, not by interest. And weigh three things:

- The stated goal is efficiency; the **binding constraint** is correctness. A liveness bug outranks
  a factor of two. A durability claim that is not true outranks both.
- Four cycles have found that **the highest-value defects were invisible to the test suite and
  correct on every input** — a verdict rule that could not tell "refused" from "did not run"; group
  commit structurally unreachable; a `&mut` for a counter; a deadlock no test could catch; and a
  read that reported the wrong end of its own interval. **Look for the class, not the instance.**
- Cycle 7 found that **the audit's own instruments** — a worktree pinned to a branch name, a latency
  proxy for a row-count phenomenon, a lint gate run in one of two repositories, a finding stated
  without its host — were wrong in the same way. Audit your instruments before you audit the code.
