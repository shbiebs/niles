# Cycle 7 audit brief — **GPT 6 Astra**

**You are auditing, not building.** Your single deliverable is a **work order**: a prioritised,
evidence-backed set of tasks that Claude Opus will execute in a later session. You write no
production code into either repository. Read everything, run the suites, run the compiler over the
corpus, write throwaway probes outside the trees. Commits are not yours to make.

The work order you produce is the only thing the executing agent sees. Assume it has none of your
context: every task carries its own preconditions, baseline, target, method, acceptance test, guard,
and guardrails, and must be executable **offline**.

**A second auditor — Fable — is reading the same two repositories independently, from a different
brief.** Its brief leans empirical: concurrency curves, memory, locks, storage, the harnesses. Yours
leans structural. The two work orders will be reconciled by the author afterwards. Three
consequences:

1. **Where you both look at the same thing and disagree, the disagreement is the valuable part.** In
   cycle 6 the two audits reported group-commit ratios of 4.1× and 8.2× for the same change; it
   resolved cleanly *because both stated their evidence class* — the ratio grows with the barrier's
   cost, and your host's overlay (`fsync=volatile`) made the serialised arm mutex-bound rather than
   fsync-bound, which was itself the finding. **State your evidence class. Never round toward the
   other auditor.**
2. **Do not soften a finding because you expect Fable to catch it.** Independent corroboration is
   why cycle 6 trusted F-12, F-13 and F-14.
3. **§8 lists items you must both check independently.** Do those even knowing they are duplicated.

Your demonstrated strength in cycle 6 was reading claims against code — you found that the
canonical-encoding guarantee was asserted in the very file that randomises its input, that no
instrument relates the engine benchmark's workload to the product suite's demand, that the E13/E19
depth claims contradict, and that checked type facts vanish before the wire. **That is the class of
defect this brief is built around.**

---

## 1. The three goals, and the two nobody has defined

The stated end goals are **a highly efficient query language (Niles)**, **a highly efficient query
engine (Nilestream)**, and **a highly efficient core-banking system (GBS)** — efficient in **memory
and speed**, without surrendering any commitment in §10.

Two of the three are under-specified in a way three cycles never resolved. **Resolving them is your
first assignment**, because an undefined goal cannot be audited and every subsequent task inherits
the ambiguity.

### 1.1 What is an efficient *language*?

Three audits have measured the engine and called it the language. A language's efficiency is not its
compiler's wall clock — compile time, startup and binary size have been refuted as priorities twice
and are not to be re-litigated. Candidate definitions, each of which implies a different instrument:

- **What the typed IR lets the engine avoid at run time.** The pipeline is source → resolve →
  typecheck → lower → `niles-ir` circuit → verifier → REV runtime. What does a *checked* fact buy?
- **Whether the type system's guarantees are exploited rather than merely checked.** A statically
  proven single-currency aggregate should not be re-checking currencies at run time; a linearity
  proof should let something be moved rather than copied. **Find the places where the compiler knows
  something the engine then re-establishes.** This is your highest-value structural sweep, and it
  bears directly on both memory and speed.
- **Whether the cost model the planner reads is calibrated against anything real.** `DPccp` to 12
  relations with a three-term cost model (flow + resident state + reconstruction depth) and
  reconstructibility pruned as a legality constraint. Is any coefficient measured?
- **The density the language mandates.** Money as `i128`; identities as `String` (LC-03); Z-set rows
  as `Vec<Value>`; `Value` as an enum over `Int(i128)`/`Null`. A language that mandates a
  representation owns its memory.

State a definition, defend it, derive measurable claims from it, and if you conclude the current
framing is unmeasurable, say so and propose the instrument.

### 1.2 What is an efficient core-banking system?

GBS has never been measured beyond a single hold probe. `G3` makes every sweep cold, records no
product-internal reads and no performance, and **no trace bridges the engine benchmark's workload to
the product suite's demand** — your own finding F-24, still open as T-14. Meanwhile GBS carries
**correctness-first blockers ahead of any efficiency claim**: lifecycle transitions and hold
placements do not survive a restart (F-12), on paths that authorise money movement, and the
confidentiality machinery is wired to nothing.

Decide, and say plainly: **is a GBS efficiency task admissible at all before T-08 and T-14 exist?**
A reasoned refusal is a finding.

---

## 2. Access

### 2.0 Your own cloud container — where most of this audit happens

**Work primarily from a clone in your own container.** It is faster than the bridge, it has a
toolchain, and nothing you do there can touch the author's files. The Mac is for the two things a
container cannot give you (§2.2). Nothing in this project needs the network at build time — **both
trees have zero external dependencies**, so `cargo test --offline --workspace` succeeds against a
bare toolchain with no registry cache and no vendoring.

**Run the preflight before anything else, and paste its entire output at the top of your work
order.** It is committed at `docs/audit/cycle-7/preflight.sh` in the `niles` tree, so you and Fable
run the *identical* script and your two environment manifests can be laid side by side when the
author reconciles the work orders:

```
git clone https://github.com/shbiebs/niles.git
git clone https://github.com/shbiebs/gbs.git
bash niles/docs/audit/cycle-7/preflight.sh "$PWD/niles" "$PWD/gbs"
```

It takes seconds, writes nothing outside its own scratch directory, and answers the one question
that decides what every later number means: **what may a measurement taken in this container be used
to claim?** It reports the host and the *cgroup-granted* core count (which is not always `nproc`),
the filesystem and mount options under the tree, a 4 KiB-write + `fdatasync` barrier probe with a
verdict, the toolchain, PostgreSQL, egress, both tree hashes, and an admissibility table for you to
fill in.

**Why this exists, in one paragraph.** Your cycle-6 container was an overlay mounted
`fsync=volatile`. It reported a barrier of **~1,000,000/s** — median 1,033,047, spread
576,243–1,460,600 — and that is not a storage measurement at all: the mount option makes the call a
no-op. Every durability conclusion drawn there was void, and the reason your group-commit ratio came
out 4.1× against Fable's 8.2× is that on a volatile overlay the serialised arm is *mutex-bound
rather than fsync-bound*, so batching has less to amortise. **That was a genuine finding, and it was
only legible because the mount option was recorded.** The probe now prints the verdict for you:

- **> 100,000 barriers/s** → `*** NOT STORAGE EVIDENCE ***`. This container cannot produce any
  durability number. Say so in your work order and route those measurements to Host C.
- **20,000–100,000/s** → suspicious; verify the mount before publishing anything durable.
- **below that** → plausible for *this* container, and still host-shaped: within-host ratios only.

Note also that `make fsync-proof` passes on a volatile overlay — it checks that the syscall reaches
the kernel, not that the kernel does anything with it. **The two checks are necessary together and
neither is sufficient alone.**

Three further container-specific traps the preflight will surface:

1. **`nproc` is not your core count.** Read `/sys/fs/cgroup/cpu.max`; if it is not `max`, the quota
   divided by the period is what you actually have. A concurrency curve plotted against a core count
   you do not own is a curve about nothing.
2. **A newer `stable` toolchain can turn the gate red for reasons unrelated to the code.**
   `rust-toolchain.toml` pins `channel = "stable"` with `rust-version = 1.95.0`, and says why: the
   machine it was built on had no egress to `static.rust-lang.org`, so rustup could not install a
   channel named by version (`BLOCKED-T-01-toolchain` in `docs/BUILD-LOG.md`). If your `stable` is
   newer and `clippy -- -D warnings` goes red, **that is a finding about the pin, not about the
   code** — report it as one, and consider whether "anyone with network access should set the
   version" is a task worth writing.
3. **`numeric_binary_oracle` needs a running PostgreSQL** with a `bench` role and `PGPORT` set.
   Without it the tree reads red for a missing service. Do not report that as a failing test; the
   preflight prints the two ways to provision one.

Everything in §6 except §6.4's second-architecture check can be done in your container.

### 2.1 GitHub

Both repositories are private, owned by **`shbiebs`**:

- `https://github.com/shbiebs/niles`
- `https://github.com/shbiebs/gbs`

Pushed complete — 17 branches in `niles`, 9 in `gbs`, no tags — plus the cycle-7 branches in §4.

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
credentials are refused, audit what you can reach and say which parts went unread. A work order that
guesses at unread code is worse than one that admits a gap.

### 2.2 The author's Mac, and what the bridge actually gives you

You are authorised to audit the working copies at `/Users/checolino/Documents/niles` and
`/Users/checolino/Documents/GBS`.

**Read this before planning any measurement.** If you reach the Mac through the Claude desktop
bridge, **you do not get macOS.** You get an isolated **Linux VM, aarch64, 4 cores, 4 GB RAM**, with
`~/Documents` mounted over **FUSE**, and **no `rustc`, no `cargo`, no `psql`, no `valgrind`.** That
is enough for reading, grepping, counting and cross-referencing — which is most of your brief — and
it is **useless for storage measurement** and cannot build the workspace.

**Anything needing the real machine must be packaged as a shell script for the author to run in
Terminal.app.** Prior cycles left `~/Documents/niles-hostc/run.sh`, `run2.sh`, `run3.sh` and read the
results back. Follow that, and:

- Say **explicitly, at the point in your work order where it is needed**, "the author must now run
  `bash ~/Documents/niles-hostc/<name>.sh` and paste the output." Do not bury it.
- Make each script **idempotent, non-interactive, and time-bounded** — state the expected runtime.
- Write results to `~/Documents/niles-hostc/results-<stamp>/` and print a compact summary to stdout,
  so a paste-back carries the finding even if the files do not.
- Instruct `bash <path>`, never `./<path>` — a prior cycle hit `zsh: permission denied` on a file
  that was not executable.
- The author's zsh has `interactive_comments` **off**: a `#` typed at the prompt is executed as a
  command. Do not put comments in lines you ask them to paste.

The real Mac has Rust, PostgreSQL and **10 cores (4 performance + 6 efficiency)** on Apple silicon
with APFS — the only host in this project's history that can run a wide concurrency test. Fable owns
that territory; use it for the **second-architecture** checks that are yours (§6.4).

**Read-only discipline.** These are live working copies. Do not commit, stage, `git clean`, `git gc`,
or check out a different branch without saying so first. `target/` is acceptable collateral; nothing
else is. **Four untracked files in `niles` must never be edited, staged, deleted, moved or bundled:**
`.DS_Store`, `AGENTS.md`, `thesis/.DS_Store`, `thesis/Niles-Thesis.pdf`. Report their byte sizes at
the end.

Two traps that have each cost a cycle:

1. **`CARGO_TARGET_DIR`.** Setting it made a nested `cargo run` contend with the outer `cargo test`
   on one build-directory lock: six spurious failures and a *wrong verdict in a thesis table* — the
   corpus recorded `d13_missing_anchor` as **Refused** where you record it **Warned** (`NL0223`,
   exit 0). T-03 removed every nested `cargo run`. **Verify that on the Mac with the variable set,
   and re-derive the thirteen corpus verdicts on arm64** — the table must equal run 3's (11 Refused,
   `d13` Warned, `d6` Accepted).
2. **PostgreSQL's barrier on macOS** — default `wal_sync_method` is a plain `fsync()` that APFS does
   not flush, while Rust's `sync_all` issues `F_FULLFSYNC`. Measured naively PostgreSQL reports
   13,458 durable commits/s against a **324/s** barrier while reporting `fsync=on`. The harness
   refuses unless `fsync_writethrough`. Do not work around the refusal.

---

## 3. Hosts and admissibility

| host | what it is | admissible |
|---|---|---|
| **A** | 2-core Linux container, ext4 on virtio, `fdatasync` 4,961–5,825/s | counters, within-host ratios |
| **B** | the desktop bridge: 4-core aarch64 Linux VM, FUSE mount, no toolchain | reading, counting; **no storage or build evidence** |
| **C** | the Mac itself, Apple M4, 10 cores, APFS, `F_FULLFSYNC` 255/s | wide concurrency, real barrier, the second-architecture run |
| **D** | **your own container** — establish it with `preflight.sh`, do not assume it | whatever the preflight's admissibility table says, and nothing beyond it. Cycle 6's D was an overlay mounted `fsync=volatile` reporting ~1,000,000 barriers/s: **nothing about durability**, and that was your own finding |

Deterministic counters (instructions, allocations, visited entries, `max_batch`, txns/fsync, lock
buckets, **compiler verdicts**) gate first and need one run. Wall clock needs two warm-ups and ≥5
measured runs, arms interleaved, median/MAD/range; a gate fires only on **≥10% and ≥3× the pooled
MAD**, otherwise **`noise-limited`**. `unsupported`, `blocked`, `not run`, `noise-limited` are
results; omission is not. No absolute figure beside another host's without a host column.

**Your comparative advantage is that most of what you need is host-independent.** Compiler verdicts,
IR shapes, claim-versus-code contradictions, layering violations and drift do not need ten cores.

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

**GBS** — base `572fe97`: `c6/01-evidence` `e2cb34e`, `c6/03-verdict-tests` `e6418e0`,
`c6/07-hold-index` **`e803b7d`**.

Gate at `d9c8699` / `e803b7d`: fmt green, clippy `-D warnings` green, Niles **801 test functions, 0
failed** (1 ignored), GBS **506 declared / 504 passing**, `make reproduce` exit 0 clean,
`make fsync-proof` green. `numeric_binary_oracle` needs PostgreSQL with a `bench` role and `PGPORT`;
without it the tree reads red for a missing service, not a defect.

---

## 5. What cycle 6 changed

Do not re-derive. Do audit — none of it has been reviewed by anyone.

**T-01** — 14 of 51 `results/` files were regenerated by `make reproduce`; **37 were not**, including
`E16-wallclock.md` (the contract) and `E19-scaling.md` (produced by no recipe at all). Your estimate
was 22 of 39 counting top-level files; the true figure was worse. `results/MANIFEST.csv` now
classifies every file with its producing command, and a test enforces that **`make reproduce` runs
no machine-dependent producer**. GBS ran `g3` *after* its diff, so the product gate's own verdict was
the one file the check could not see.

**T-02** — `nilestreamd --durable <segment>`; `bench --scaling-only --nls-only`; `select
nilestream_sealer` exposing sealer counters and a bucketed lock wait/hold histogram.

**T-03** — both verdict suites classified a non-zero `cargo` exit as a *compiler refusal*.
`niles_schema.rs` asserts in both directions, so **the counterfactual tests proving a bad program is
rejected would have passed without the compiler ever seeing the program**. Verdicts now come from
`nilesc`'s own markers (`ok:` / `error[NL` / `warning[`); an absent compiler is `BLOCKED-nilesc`.
`error[NL` carries its prefix deliberately: a rustc failure while *building* the compiler also
prints `error[E….]`.

**T-04** — every durable result names `(host, filesystem, mount options, device class, barrier,
ceiling, spread)`; `wal_sync_method` pre-registered; `make fsync-proof` straces the barrier;
`SPEC-ENGINE.md` Part 0 states `durable throughput ≤ barriers/s × transactions per barrier`.

**T-05** — the daemon held one mutex across `Session::handle` **and the fsync inside it**, so group
commit — built, and tested at sixteen concurrent submitters — never formed a batch.
`submit_pending` now returns the reply channel; `append` applies under the lock and pushes the
barrier token to a pending list; `serve` takes the tokens, releases the lock, waits, **then** writes
the reply. A separate **`visible` frontier** is published by `fetch_max` when the barrier returns.
`max_batch` 1 → 15, txns per barrier 1.00 → **6.02**, durable appends 3,850 → 22,049 ops/s (**5.73×**,
short of the 6× target and reported as short).

**T-06** — `Base::reconstruct` took `&mut self` **only to increment a row counter**, which made every
read of the base exclusive, which made `Serving::query` exclusive, which left one mutex around
`Session::handle`. Counter → `AtomicU64`; base → `RwLock<Ledger>`; read model → its own `Mutex`,
because a read of a partial view *is* a write to it; `Serving` is a shared-borrow trait and `serve`
acquires nothing. Point reads 39,493 → 79,814 ops/s at 2 connections; folds 227 → 259 at 4; lock wait
p99 ≤1,024 µs → ≤1 µs. **All on two cores.**

**T-06a** — see §6.1.

**T-07** — proving a hold open scanned the account's entries, then *all* of global `hold.void`, then
all of `hold.expired`: 29 µs at 0 voids → **8,630 µs at 40,000**. Now indexed through the kernel's
existing `consumed` map: **26 µs and flat**, zero entries visited for an open hold.

---

## 6. Where your budget should go

### 6.1 A deadlock nobody's tests could catch — **audit the fix and generalise the class**

T-06 replaced one mutex with finer locks and did not say which comes first. `answer_from_view` took
the read model and reached for the base underneath it; `append` took the base exclusively and reached
for the read model underneath *that*. AB–BA, reachable in the shipped daemon the moment one client
read while another wrote. Demonstrated before fixing: four readers against a continuous appender
under a thirty-second deadline — **two of five threads finish** on the old order. `d9c8699` states
**B (base) < P (pending) < V (read model) < C (currency set)** and holds `answer_from_view` to one
base guard, because `RwLock` is not reentrant.

**Why this is in your brief and not only Fable's:** it is a *structural* defect that no correctness
test can catch, because **no answer is ever wrong on the way into a deadlock**. That is the same
shape as T-03's verdict rule and T-05's unreachable group commit. Your assignment is not to
re-measure it but to ask **what else has that shape**:

- The enumeration covered `nilestream-server` only. **`nilestream-ledger` and `nilestream-core` have
  never been examined**, and the sequencer has its own locks.
- **Audit the fix.** Written by the agent that wrote the defect, in the same session, reviewed by
  nobody.
- The guard is a source-text test. Is a stronger mechanism warranted — a lock hierarchy in types, a
  debug-build order assertion — given the **zero-external-dependency rule that forbids `loom` and
  `parking_lot`**?

### 6.2 A comment that was false on the correctness-critical path

`answer_from_view` claimed *"the write path holds the engine's lock, so the frontier cannot move
between `observe` and here and the two are always equal."* That reasoning died with the engine lock
in T-05; the comment survived until `d9c8699`. The `answered.anchor != anchor` check and its fold
fallback are unchanged and still correct, but the mismatch is now a **normal event** rather than a
defence against an impossible one.

**Your question is whether the check is still *sufficient*** — a question about the absence lattice,
the visible frontier, and what "the answer is true at the anchor asked for" means when the frontier
moves during a read. Then LC-17: is a fold fallback the right response when it costs ~100× the read
it replaces?

**And the general form:** how many other comments in this codebase justify a behaviour by an
invariant that T-05 or T-06 removed? The code is unusually heavily commented and the comments carry
the reasoning; a stale one is a defect that will be trusted.

### 6.3 The counter specified twice and shipped neither time

Work order 6 required T-02's stats surface to report **anchor-mismatch fold fallbacks**, and required
the cycle-6 report to state **the fallback rate after T-05**. Neither exists; `d9c8699` marks the
site `BLOCKED-fallback-rate` in a comment, and a comment is not a counter. A point read that falls
back to the fold is ~100× more expensive, so **a mixed read/write workload could collapse from 80,000
ops/s to fold speed with every test green and no number anywhere showing it.**

Note the meta-finding: an explicitly specified deliverable was silently dropped by the executing
agent **twice**, and the cycle-6 report did not say so. Consider whether the work-order format needs
a mechanism that makes a dropped requirement visible.

### 6.4 Claims against code — your sweep

The following are all "asserted somewhere, unverified anywhere". Each is a candidate finding:

- **The hash chain is a placeholder.** `README.md` records "built with a **placeholder hasher**
  (ADR 0002); API is drop-in", while a hand-written SHA-256 exists elsewhere and was measured at ~30%
  of the daemon's seeding instructions. `docs/HANDOFF-OPUS.md` notes **the specs do not carry the
  caveat forward**. Every durability and throughput number includes a fake chaining cost. Does the
  *thesis* carry it? Does any tamper-detection claim depend on it?
- **Cross-target claims exceed cross-target evidence.** Appendix C claims native Linux, macOS ARM64,
  Windows x86-64, WSL and WASM, with a **cross-target determinism obligation and test**. Only
  x86_64-linux and arm64-macOS have been exercised, and only for the compiler corpus. Say what is
  actually supported and what the determinism obligation currently proves.
- **The bootstrap (Appendix E) has gates but no audit.** A tree-walking interpreter for the
  imperative subset, a lexer written in Niles, and gates for run / equivalence against the reference
  lexer / self-application / fixpoint. Nobody has reviewed the interpreter for correctness. The
  parser and type-checker in Niles are unwritten — is the three-stage bootstrap claim still honest?
- **Confidentiality.** F-13's canonical-encoding claim was scoped correctly at `encode.rs:306` and
  wrongly at `encode.rs:1`, `ARCHITECTURE.md:316` and `BUILD-LOG.md:557`. LC-12's B′ was accepted.
  T-12 is unstarted. Meanwhile the E2EE chapter's property is unshipped. Check whether any *other*
  guarantee is stated at file-header scope and true only at function scope.
- **The thesis has not been reconciled with the code since cycle 6.** `SPEC-ENGINE.md` Part 0 was
  rewritten *by the executing agent*, which is exactly the arrangement that produces
  self-consistent-but-unreviewed prose. Chapters 4, 6 and 9 were not checked. Every divergence is a
  `MISMATCH-<id>`.
- **`no unsafe` is not data-race freedom.** The thesis claims immutability and data-race freedom as a
  *result*. Nothing has checked it against the new relaxed counters, the `AcqRel` visible frontier,
  and the `fetch_max` publication protocol. This is a proof obligation, not a benchmark.
- **`e803b7d` made an invariant load-bearing that was previously incidental** — *if the tag map says
  a tag was consumed at an epoch, an entry in that epoch consumes it*. Where the old code walked on,
  the new code reports absence, and **a settled hold reported open keeps encumbering funds**. Check
  that the invariant holds everywhere the map is written, not only where it is read.

### 6.5 GBS, where correctness gates efficiency

- **F-12 / T-08.** `lifecycle.rs:310` pushes to an in-memory `Vec<Event>`; no serialiser or rebuilder
  exists in either workspace. `tradefinance.rs:249-251` and `:311-313` call `transition` **before**
  `ps.seal`. `HoldBook` is a `BTreeMap<HoldId, Hold>`; `Hold::place` posts nothing; four products
  place holds; `encumbered_total` folds the in-memory book while `hold.rs:20-21` states the opposite.
  **After a restart every available balance is overstated by every open hold.** The project filed
  this as a *decision* (`BLOCKED-T-09-lifecycle`) and pinned the wrong behaviour with a test. Unstarted.
- **F-24 / T-14.** No trace bridges the engine benchmark's workload to product demand. Until it
  exists, no optimisation can claim product relevance.
- **LC-03.** `AccountId` as `String` across ~245 sites. The census that would decide it does not
  exist.
- **GBS layering is law**, and `make gate` end to end was never run in cycle 6.

---

## 7. Where *not* to spend the cycle

Refuted, twice in some cases: compile time, startup, binary size, idle RSS; connection memory at this
scale (peak RSS 4,836 KiB at 16 connections on A, 23,312 KiB at 12 on C); reply streaming (1.04× /
0.94× at 10k / 100k rows, within scatter, peak reply memory 69,632 B) **except** the slow-consumer
measurement that F-25 said was needed and never happened; crypto dependencies (settled: none);
point-read sharding unless T-06's Host C trigger fires.

---

## 8. Check these independently, even knowing Fable also will

1. **The T-06a lock-order fix** (§6.1) — complete? Right guard, given the zero-dependency rule?
2. **Is the `answered.anchor != anchor` check still sufficient**, and is a fold fallback right
   (LC-17)?
3. **Is any published number now stale** after T-05, T-06 and T-06a?
4. **What does "efficient" mean for each of the three artefacts**, stated so it can be measured?

---

## 9. Potential misses in every cycle so far

Confirm or refute each; a refutation with evidence is as valuable as a finding.

1. **Nothing has ever been measured under a mixed workload.** Every row in every results file is one
   workload at a time — which is precisely why §6.1's deadlock survived three audits and a full test
   suite. The deadline test in `d9c8699` is the project's first mixed-workload exercise and it exists
   to catch a hang, not to measure.
2. **Lock order had never been analysed until §6.1**, and only one crate is done.
3. **Crash recovery has never been exercised under concurrency**, now that group commit forms batches
   of fifteen. Durable-before-visible is the project's central claim.
4. **The eviction budget has one shape** — 10,000 accounts, 2,500-entry budget — and the phase
   diagram is swept in the prototype, not through the daemon.
5. **The idempotency window** `idem: IdemKey window 30.days` has unmeasured memory as history grows —
   which is the "memory against base size" question the thesis is about.
6. **The plan cache is untested under statement diversity.**
7. **Cross-target claims exceed evidence** (§6.4).
8. **The bootstrap has gates but no audit** (§6.4).
9. **Thesis-versus-code drift since cycle 6** (§6.4).
10. **Slow-consumer behaviour is unmeasured.**
11. **`no unsafe` ≠ data-race freedom** (§6.4).
12. **GBS has no efficiency instrument at all** (§1.2, §6.5).

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
- **A stated lock order.** B < P < V < C; any new lock must be placed in it.
- **Every optimisation ships its guard and threshold in the same commit**, and the executor proves
  the guard *fails* on the reverted optimisation, transcript in the report. Where a repair is
  invisible to behaviour — T-03's, T-06's and T-06a's all were — **the guard must read the source**.
- **No performance claim without a reproducible command**, raw results under `results/`, prose
  generated from them, host class and barrier name in the header.
- **Benchmarks touch committed artefacts only with `--publish`.**
- **Micro-benchmark gains are never contract results.** Only E16/E19 rows are.
- **No task may require network access at execution time.** No egress to crates.io or
  `static.rust-lang.org`. Rust 1.95.0, valgrind 3.22, and a startable PostgreSQL 16 are
  preconditions, not steps. **No task installs anything.**
- **Attribution** is taken from the executing session's own instructions; record "none required" if
  there is none. Never bake in a trailer.

---

## 11. Open questions to carry, restate or close

Settled in cycle 6, **not to be reopened**: LC-01, LC-02 (answered A and implemented), LC-04, LC-08,
LC-09, LC-10, LC-11, LC-12.

- **LC-03** — `AccountId` String→u64: full conversion (~245 sites, 17 files, best memory) vs a
  boundary newtype vs no change until a census attributes ≥10% to identity strings. **This one is
  yours to frame**: it is a language-representation question before it is an engineering one.
- **LC-05** — rendered-NULL, seed depth, SQL refusal boundaries. Broad acceptance widens the
  benchmark surface and the test burden; narrow honest refusal narrows both.
- **LC-06** — process/allocator boundary for comparative memory.
- **LC-07** — E23's H-F1 slope and floor.
- **LC-13** — release checks mandatory or advisory.
- **LC-14** — eviction budget under sharding. **Retired**, reopened only if T-06's Host C trigger
  fires.
- **LC-15** — what the OLTP contract is measured against. Accepted: the ratio against PostgreSQL on
  the same storage under the same barrier.
- **LC-16** — should `nilestreamd` be durable by default? Still unanswered. A DBMS whose shipped
  binary is volatile by default carries a documentation obligation at minimum.

New, from cycle 6 — raise, do not answer alone:

- **LC-17** — is a fold fallback the right response to an anchor mismatch, now that it fires in
  normal operation and costs ~100× the read it replaces? Retry at the observed anchor; serve at the
  later anchor and report it; block briefly. **Each trades a different consistency property — that
  analysis is yours.**
- **LC-18** — is `std::sync::RwLock` the right primitive for the base? Not writer-preferring on every
  platform; a sustained fold stream could starve appends, which on a ledger is a durability latency
  problem.
- **LC-19** — **does the reader-writer split change what "strictly serializable" is claimed at the
  wire?** A session's anchor is monotone and the visible frontier moves only after a barrier. Argue
  explicitly that the new lock structure preserves the claim, or say what must change. **This is the
  most important question in this brief**: it is the thesis's central guarantee, and the two largest
  changes of cycle 6 were made underneath it without anyone restating the argument.

---

## 12. What you must produce

A single work order containing:

0. **The preflight output, verbatim, with its admissibility table filled in** (§2.0). Every number
   later in the document is read against it, and an unstated environment is how cycle 6 nearly
   published a durability figure measured on a mount option.
1. **An executive judgement** — where the three artefacts stand against the three efficiency goals,
   with the arithmetic for any reachability claim spelled out, and **your definitions from §1**.
2. **Findings**, each with a class (wrong-measurement, correctness, liveness, guarantee-bounded,
   instrument-gap, claim-versus-code, negative), an **EV score** (`impact × confidence ÷ cost`, each
   1–5, ties favour correctness), **HI** or **HS**, evidence with file and line, the measured or
   argued cost, the proposed repair, and **what the repair gives up**. Negative findings are
   first-class.
3. **A task list in dependency order**, each task carrying what it closes, what it depends on,
   whether it is executable or gated, its files, its baseline, its target as a ratio against that
   baseline, its method, its acceptance test, its **guard and the reversion that must make the guard
   fail**, and its guardrails. Mark a **cut line** for one agent in one cycle.
4. **Branch stacks and merge order** for both repositories from the current heads.
5. **A validation protocol**: commands, what green means, what a red row means for the claims that
   depend on it.
6. **Every Host C script you need the author to run**, named, with expected runtime, and an explicit
   "**the author must run this now**" marker at the point where the result is needed.
7. **The open-questions ledger** (§11), updated, with LC-19 answered or sharpened.
8. **Reporting requirements** for the executing agent, including **every target it missed**, guard
   transcripts, worktree status separating the four untracked files from task changes, and **exactly
   three material facts found while executing that the work order did not cover**, negative findings
   included.

Order by `impact × confidence ÷ cost`, not by interest. And weigh two things:

- The stated goal is efficiency; the **binding constraint** is correctness. A liveness bug outranks a
  factor of two; a durability claim that is not true outranks both; and GBS's restart loss (§6.5)
  outranks every efficiency task in that repository.
- Three cycles have found that **the highest-value defects were invisible to the test suite** — a
  verdict rule that could not tell "the compiler refused" from "cargo did not run"; group commit
  built, tested and structurally unreachable; a `&mut` taken for a counter that serialised every
  read; a deadlock no correctness test could catch; and a guarantee asserted in the file that
  randomises its input. **Look for the class, not the instance: what else is correct on every input
  and wrong in structure?**
