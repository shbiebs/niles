# Cycle 7 audit brief — **Fable, structural pass**

**You are auditing, not building.** Your single deliverable is a **work order**: a prioritised,
evidence-backed set of tasks that Claude Opus will execute in a later session. You write no
production code into either repository. Read everything, run the suites, run the compiler over the
corpus, write throwaway probes outside the trees. Commits are not yours to make.

The work order you produce is the only thing the executing agent sees. Assume it has none of your
context: every task carries its own preconditions, baseline, target, method, acceptance test, guard,
and guardrails, and must be executable **offline**.

**This is the second of two passes over the same commits, and the first one already ran.** Cycle 7
was designed as two independent audits — an empirical one and a structural one, by different
models, reconciled afterwards. The structural brief was assigned to GPT 6 Astra and **was refused by
that model's content classifier before any work was done**. You are taking over its territory. The
empirical pass — also Fable — has finished; its work order is at `docs/audit/cycle-7/fable-work-order.md`
in the `niles` tree (and `claude/work-order-7-fable-audit.md` in the project), with the probes that
produced every number under `docs/audit/cycle-7/probes/`.

Three consequences, and they are different from the ones the original design had:

1. **Independence is weaker than planned, so compensate for it deliberately.** The empirical work
   order was written by the same model you are. Treat every finding in it as a *claim to verify from
   the source*, not as a fact to build on. Where you would have corroborated Astra, you now
   corroborate yourself — so do it against the code, with the evidence class stated, and say
   explicitly where you could not.
2. **Do not re-measure.** The ten-core curves, the crash test, the fallback count, the E16
   three-arm table, the memory curve — those exist. Your job with each is to ask whether the
   *reasoning* from the number to the finding is sound, whether the structural cause named is the
   real one, and what else has the same shape.
3. **Where you disagree with the empirical pass, the disagreement is the valuable part.** State
   your evidence class. Never round toward the earlier document.

The class of defect this brief is built around is the one that has produced every high-value finding
for three cycles: **things that are correct on every input and wrong in structure** — a verdict rule
that could not tell "the compiler refused" from "cargo did not run"; group commit built, tested and
unreachable; a `&mut` taken for a counter that serialised every read; a deadlock no correctness test
could catch; a guarantee asserted in the file that randomises its input; and now, from the
empirical pass, **a durable daemon that fsyncs the epoch number and not the rows, and a maintained
view that discards exact answers whenever a barrier is in flight.** Both of those were found by
reading, then confirmed by measurement. Find the next one by reading.

---

## 1. The three goals, and the two nobody has defined

The stated end goals are **a highly efficient query language (Niles)**, **a highly efficient query
engine (Nilestream)**, and **a highly efficient core-banking system (GBS)** — efficient in **memory
and speed**, without surrendering any commitment in §10.

The empirical pass declined to define the first and third and said so. **Defining them is your first
assignment**, because an undefined goal cannot be audited and every task inherits the ambiguity.

### 1.1 What is an efficient *language*?

Three audits have measured the engine and called it the language. A language's efficiency is not its
compiler's wall clock — compile time, startup and binary size have been refuted as priorities twice
and are not to be re-litigated. Candidate definitions, each implying a different instrument:

- **What the typed IR lets the engine avoid at run time.** Source → resolve → typecheck → lower →
  `niles-ir` circuit → verifier → REV runtime. What does a *checked* fact buy at run time?
- **Whether the type system's guarantees are exploited rather than merely checked.** A statically
  proven single-currency aggregate should not be re-checking currencies at run time (`answer_from_view`
  calls `currency_count()` on every keyed read — is that the compiler's job or the engine's?); a
  linearity proof should let something be moved rather than copied. **Find every place where the
  compiler knows something the engine then re-establishes.** This is your highest-value structural
  sweep and it bears on both memory and speed.
- **Whether the cost model the planner reads is calibrated against anything real.** `DPccp` to 12
  relations with a three-term cost model (flow + resident state + reconstruction depth). Is any
  coefficient measured? The empirical pass has a number the model should predict: a fold of 20,000
  postings into 10,000 groups costs 2.1 ms on an M4 core and ~6 ms on a Xeon vCPU.
- **The density the language mandates.** Money as `i128`; identities as `String` (LC-03); Z-set
  rows as `Vec<Value>`; `Value` as an enum over `Int(i128)`/`Null`. The empirical pass measured
  **173 bytes per base row at two million rows** against ~40 bytes of payload — a 4.3× overhead that
  *falls* with size. Say how much of that the language mandates and how much the engine chose.

State a definition, defend it, derive measurable claims from it, and if the current framing is
unmeasurable say so and propose the instrument.

### 1.2 What is an efficient core-banking system?

GBS has never been measured beyond one hold probe. `G3` makes every sweep cold, records no
product-internal reads and no performance, and **no trace bridges the engine benchmark's workload to
the product suite's demand** (F-24, open as T-14). GBS also carries correctness-first blockers ahead
of any efficiency claim: lifecycle transitions and hold placements do not survive a restart (F-12),
on paths that authorise money movement. The empirical pass concluded that no GBS efficiency task is
admissible before those exist and wrote none. **Confirm or overturn that with an argument**, and if
you confirm it, write the task that makes measurement possible.

---

## 2. Access

### 2.0 Your own cloud container — where this audit happens

**Work from a clone in your own container.** Both trees have **zero external dependencies**, so
`cargo test --offline --workspace` succeeds against a bare toolchain with no registry cache.

**Run the preflight before anything else, and paste its entire output at the top of your work
order.** It is committed at `docs/audit/cycle-7/preflight.sh`; the empirical pass's output for the
same script is at `docs/audit/cycle-7/probes/preflight-host-A.txt`, so the two can be laid side by
side:

```
git clone https://github.com/shbiebs/niles.git
git clone https://github.com/shbiebs/gbs.git
git -C niles checkout c6/audit-cycle-7
git -C gbs checkout c6/07-hold-index
bash niles/docs/audit/cycle-7/preflight.sh "$PWD/niles" "$PWD/gbs"
```

It reports the host and the **cgroup-granted** core count (not `nproc`), the filesystem and mount
options under the tree, a 4 KiB + `fdatasync` barrier probe with a verdict, the toolchain,
PostgreSQL, egress, both tree hashes, and an admissibility table for you to fill in. A barrier above
~100,000/s is a mount option, not a device — cycle 6's other container measured ~1,000,000/s on an
overlay mounted `fsync=volatile`, and every durability number taken there was void. `make
fsync-proof` passes on such a mount too; the two checks are necessary together.

Three container traps the preflight surfaces, each of which has already cost something:

1. **`nproc` is not your core count.** Read `/sys/fs/cgroup/cpu.max`.
2. **A newer `stable` turns the gate red for reasons unrelated to the code.** `rust-toolchain.toml`
   pins `channel = "stable"` and says its intended value is `1.95.0`. The empirical pass **confirmed
   this on the author's Mac**: rustc 1.97.1 fails `clippy -- -D warnings` on two lints in `niles-ir`
   that 1.95.0 never emitted. If your `stable` is newer and the gate is red, that is F-33, not a
   defect in the code — report it as such and check whether the two lints are the only ones.
3. **`numeric_binary_oracle` needs a running PostgreSQL** with a `bench` role and `PGPORT` set;
   without it the tree reads red for a missing service, not a defect.

### 2.1 GitHub

Both repositories are private, owned by **`shbiebs`**: `https://github.com/shbiebs/niles` and
`https://github.com/shbiebs/gbs`. **If you do not have credentials, ask for the narrowest thing that
works**: a fine-grained personal access token, repository access **Only select repositories** →
`niles` and `gbs`, permissions **Contents: Read-only** (Metadata is added automatically), nothing
else. Username `shbiebs`, password = the token. **Never push.** `Repository not found` on a private
repo is a token-scope problem, not a URL problem: say so and ask again. If credentials are refused,
audit what you can reach and say which parts went unread.

### 2.2 The author's Mac — reading only, for you

You are authorised to read the working copies at `/Users/checolino/Documents/niles` and
`/Users/checolino/Documents/GBS`. Through the desktop bridge you get a **4-core aarch64 Linux VM
over FUSE with no toolchain** — reading, grepping and cross-referencing only. Anything needing the
real ten-core macOS machine is packaged as a script the author runs in Terminal.app, results under
`~/Documents/niles-hostc/results-<stamp>/`, instructed as `bash <path>` never `./<path>`, with no
`#` comments in lines the author pastes (their zsh executes them). **The empirical pass already ran
two such scripts (`run4.sh`, `run5.sh`) and their results are read back and in its work order; you
should need at most one more — the arm64 corpus verdicts (§6.6) — and only if you cannot derive
them in your container.**

**Read-only discipline.** Live working copies: no commits, staging, `git clean`, `git gc`, or
checkouts. Three scratch worktrees exist under `~/Documents/niles-hostc/` from the empirical pass;
leave them. **Four untracked files in `niles` must never be edited, staged, deleted, moved or
bundled:** `.DS_Store` (10,244 B), `AGENTS.md` (16,639 B), `thesis/.DS_Store` (8,196 B),
`thesis/Niles-Thesis.pdf` (816,110 B). Report their sizes at the end.

---

## 3. Hosts and admissibility

| host | what it is | admissible |
|---|---|---|
| **A** | the empirical pass's 2-core container, ext4, `fdatasync` 7,371/s | its counters and within-host ratios — already taken |
| **B** | the desktop bridge VM | reading only |
| **C** | the Mac, Apple M4, 10 cores, APFS, `F_FULLFSYNC` 255/s | already measured by `run4.sh`/`run5.sh`; one more script at most |
| **D** | **your container** — establish it with the preflight | whatever its admissibility table says |

**Your comparative advantage is that almost everything you need is host-independent.** Compiler
verdicts, IR shapes, claim-versus-code contradictions, layering violations, drift, and the soundness
of an argument do not need ten cores. Deterministic counters gate on one run; wall clock on ≥5 with
median/MAD and the ≥10%-and-≥3×-MAD rule; `unsupported`, `blocked`, `not run`, `noise-limited` are
results.

---

## 4. Exactly where the trees stand

**Niles** — base `4d9a457`; eight stacked branches, none merged to `master`:

| branch | head | task |
|---|---|---|
| `c6/01-evidence` | `79c2881` | T-01 evidence manifest |
| `c6/02-instruments` | `05eeef7` | T-02 durable daemon, `--nls-only`, sealer + lock counters |
| `c6/03-verdict-tests` | `a25d7ab` | T-03 nested `cargo run` removed |
| `c6/04-fsync-contract` | `d6e535b` | T-04 fsync ceiling per host and barrier |
| `c6/05-unlock` | `15425b5` | T-05 barrier released from the engine lock |
| `c6/06-rwlock-read` | `d681f0d` | T-06 reader-writer base |
| `c6/06a-lock-order` | `d9c8699` | T-06a lock order B < P < V < C |
| `c6/audit-cycle-7` | **`5ff365c`** | cycle-7 briefs, preflight, the empirical work order and probes — **docs only; the engine is `d9c8699`'s** |

**GBS** — base `572fe97`: `c6/01-evidence` `e2cb34e`, `c6/03-verdict-tests` `e6418e0`,
`c6/07-hold-index` **`e803b7d`**.

Gate at `5ff365c` / `e803b7d` on rustc 1.95.0: fmt green, clippy green, Niles **801 test functions,
0 failed** (1 ignored), GBS 506 declared / 504 passing, `make reproduce` exit 0, `make fsync-proof`
green. **On rustc 1.97.1 clippy is red in `niles-ir` (F-33).**

---

## 5. What is now known, and what the empirical pass left for you

Read `docs/audit/cycle-7/fable-work-order.md` in full before continuing. What follows is the
structural residue of each of its findings — the part a reading has to settle.

**F-26 — the durable daemon loses every appended row on restart.** `rev_engine.rs:615` hands the
sequencer `epoch.to_string().into_bytes()` as the payload; `with_durable` never replays a segment;
after SIGKILL and restart, 25,416 acknowledged inserts were gone and a retry of an acknowledged id
was accepted as new. The segment layer (`nilestream-ledger/src/segment.rs`) recovers payloads
correctly and is tested for it. **Yours:** (a) does the thesis's durability text describe the daemon
or the ledger crate — if the daemon, name the `MISMATCH`; (b) the task the empirical pass wrote
(T-01) encodes rows into the payload and replays on open — **audit that design against the hash
chain**: replaying through `Ledger::submit` recomputes the chain from the seed; is the seed itself
durable, and is the chain then *verified* against anything, or merely rebuilt? (c) the sequencer's
epoch numbering and the ledger's are two clocks; state the invariant that relates them and where it
is checked; (d) `interpret` maps `Rejected::Duplicate` to `Ok` — find every other place a refusal is
turned into a success on the way up.

**F-27 — 87.4% of keyed reads bypass the maintained view under concurrent writes.** Cause: T-05
applies an epoch to the view before it is visible, so `Rev::read`'s `effective = max(stamp,
applied)` exceeds every reader's anchor while any barrier is in flight, and `answer_from_view`
discards the answer. The empirical pass argues the discarded answer is *exact* whenever `stamp ≤
anchor ≤ applied`, because `apply_epoch` restamps on every delta (`rev.rs:311`), and proposes a
one-condition change in `Rev::read` (T-02). **Yours, and it is the most important reading in this
brief:** is that argument licensed by the algebra as the thesis states it? Walk the reconstruction
theorem and the definition of a hit's version. Check the pinned-entry path, the `Materialize::Full`
path, and a key whose *first* delta lands in `(anchor, applied]` for a resident-but-empty slot.
Then LC-17: with the fix, what remains of "retry / serve later / block"? And LC-20 (new): should a
session be able to *ask* for the applied frontier as a bounded-staleness rung, so the view serves
without any fallback? That is a consistency-ladder question and belongs to the thesis.

**F-28 / F-30 — T-06 on ten cores, and the lock order across all three crates.** Fold 1.83× flat →
3.13× at 4 and 4.24× at 8; point 95k → 140k/s; the fix costs nothing measurable; total order
**O < B < P < V < C with S a leaf**, `nilestream-core` holds no locks. **Yours:** the enumeration was
by grep and by reading each site once. Verify it from the source independently; check the channel
between `append` and the sealer is unbounded (a bounded one under B would be a new edge); and answer
the guard question — the source-text test finds the view by the first `.lock()` in the function,
which a future `pending.lock()` placed earlier would fool. Is a stronger mechanism warranted — a
lock hierarchy in types, a debug-build order assertion — given that **`loom` and `parking_lot` are
forbidden** by the zero-dependency rule?

**F-29 — the contract table flips MET/NOT MET between two instances of one host class.** `report`
2.68× on cycle 5's container, 2.17× on the empirical pass's — and 2.30× on the latter *before* any of
cycle 6's changes. **Yours:** what does the thesis say the contract *is*? If it is stated as an
absolute ratio, that statement is now falsified by two instances of the same class and needs a
`MISMATCH`; the empirical pass's T-04 restates it as a same-session A/B — check that restatement
against the methodology chapter's own rules for control variables.

**F-32 — a specified deliverable was dropped twice and nothing surfaced it.** The anchor-mismatch
counter, required in T-02 and in the T-05 report of work order 6, shipped neither time. **Yours:**
propose the mechanism — a requirements checklist copied verbatim into the executor's report is the
empirical pass's suggestion; say whether that is enough or whether the work-order *format* needs a
machine-checkable manifest.

**F-33 / F-37 — toolchain and T-03.** T-03 is confirmed on the Mac; the pre-T-03 code fails a
*different* subset on each run under `CARGO_TARGET_DIR`. Clippy is red under 1.97.1. Nothing
structural remains except to check that the two `niles-ir` lints are the only ones and that fixing
them changes no semantics.

---

## 6. Your territory — the structural sweep

### 6.1 Claims against code

Each is "asserted somewhere, unverified anywhere". Each is a candidate finding with a file and line.

- **The hash chain is a placeholder.** `README.md`: "built with a **placeholder hasher** (ADR 0002);
  API is drop-in". A hand-written SHA-256 exists elsewhere and was measured at ~30% of the daemon's
  seeding instructions. `docs/HANDOFF-OPUS.md` notes the specs do not carry the caveat forward. Every
  durability and throughput number includes a fake chaining cost. Does the *thesis* carry the
  caveat? Does any tamper-detection claim (E1's "chain broke on mutation") depend on the placeholder
  being collision-resistant? **And now F-26 makes this sharper:** a chain that is rebuilt on replay
  rather than verified protects nothing across a restart whatever the hasher.
- **Cross-target claims exceed cross-target evidence.** Appendix C claims native Linux, macOS ARM64,
  Windows x86-64, WSL and WASM with a cross-target determinism obligation and test. Only
  x86_64-linux and arm64-macOS have been exercised, and only for the compiler corpus. Say what is
  actually supported and what the obligation currently proves.
- **The bootstrap (Appendix E) has gates but no audit.** A tree-walking interpreter for the
  imperative subset, a lexer in Niles, gates for run / equivalence / self-application / fixpoint.
  Nobody has reviewed the interpreter. The parser and type-checker in Niles are unwritten — is the
  three-stage bootstrap claim still honest?
- **Confidentiality's scoping.** F-13's canonical-encoding claim was scoped correctly at
  `encode.rs:306` and wrongly at `encode.rs:1`, `ARCHITECTURE.md:316`, `BUILD-LOG.md:557`. T-12 is
  unstarted. **Check whether any other guarantee is stated at file-header scope and true only at
  function scope** — that is a class, and F-13 was one instance.
- **The thesis has not been reconciled with the code since cycle 6.** `SPEC-ENGINE.md` Part 0 was
  rewritten *by the executing agent* — the arrangement that produces self-consistent-but-unreviewed
  prose — and now carries T-05's and T-06's numbers. Chapters 4, 6 and 9 were not checked. Every
  divergence is a `MISMATCH-<id>`; F-26 and F-29 each imply at least one.
- **`no unsafe` is not data-race freedom.** The thesis claims immutability and data-race freedom as
  a *result*. Check the claim against the new relaxed counters, the `AcqRel` visible frontier, and
  the `fetch_max` publication protocol — a proof obligation, not a benchmark. Note that `Pending::wait`
  publishes with `fetch_max` an epoch numbered by the *ledger* after a barrier confirmed by the
  *sequencer* (F-26's two clocks).
- **`e803b7d` made an invariant load-bearing that was previously incidental** — *if the tag map
  says a tag was consumed at an epoch, an entry in that epoch consumes it*. Where the old code walked
  on, the new code reports absence, and a settled hold reported open keeps encumbering funds. Check
  that the invariant holds everywhere the map is *written*, not only where it is read.
- **Stale reasoning in comments.** The codebase carries its reasoning in comments, and one on the
  correctness-critical path was false for two commits after T-05 removed the invariant it cited.
  **How many others justify a behaviour by an invariant that T-05, T-06 or T-06a removed?** A stale
  comment in this codebase is a defect that will be trusted.

### 6.2 What the compiler knows and the engine re-establishes

The sweep §1.1 asks for, made concrete. For each of: currency uniqueness, conservation per
`(txn, cur)`, idempotency-key uniqueness within the window, the serve contract's consistency rung,
and `materialize: full` versus `auto` — find where the checker proves it and where the engine
re-checks, re-derives or ignores it at run time. Each pair is either a run-time cost the language
already paid for, or a proof the engine does not trust — and the second is the more interesting
finding.

### 6.3 GBS, where correctness gates efficiency

- **F-12 / T-08.** `lifecycle.rs:310` pushes to an in-memory `Vec<Event>`; no serialiser or
  rebuilder exists; `tradefinance.rs:249-251` and `:311-313` call `transition` **before** `ps.seal`;
  `HoldBook` is in memory while `hold.rs:20-21` says the opposite. After a restart every available
  balance is overstated by every open hold. Filed as a *decision* (`BLOCKED-T-09-lifecycle`) with the
  wrong behaviour pinned by a test. **This is F-26's twin one layer up, and the two should be one
  task family.**
- **F-24 / T-14.** No trace bridges the engine benchmark to product demand.
- **LC-03.** `AccountId` as `String` across ~245 sites; the census does not exist. Yours to frame:
  it is a language-representation question before an engineering one, and §1.1's 173 B/row figure
  is the number it should be measured against.
- **GBS layering is law**, and `make gate` end to end on GBS was never run in cycles 6 or 7.

### 6.4 Idempotency, plan cache, eviction shapes — reading, not measuring

- `idem: IdemKey window 30.days`: find where the window is enforced and where it is *pruned*. If it
  is never pruned, the memory question is structural before it is empirical.
- The plan cache is epoch-keyed and invalidated by schema epoch. Read its eviction; say what a
  workload of ten thousand distinct statements does to it.
- The eviction budget has one measured shape. Read the policy's `CostAware` arm and say whether its
  inputs are ever populated on the wire path.

### 6.5 The class hunt

For each of the six structural defects named at the top of this brief, write down the *shape* in one
sentence, then search the code for the shape rather than the instance. Two shapes to start with:
**"a guarantee delegated to a layer that discards the thing the guarantee is about"** (F-26: durability
delegated to a sequencer that is given the epoch number) and **"a check that compares against the
wrong clock"** (F-27: the view's applied frontier against the session's visible one). There are
others.

### 6.6 The second-architecture corpus

Re-derive the thirteen compiler-corpus verdicts on arm64 — the table must equal cycle 6 run 3's
(11 Refused, `d13` Warned, `d6` Accepted). If your container is arm64, do it there; otherwise this is
the one Host C script you may need, and the author runs it.

---

## 7. Where *not* to spend the cycle

Refuted, some twice; do not reopen without evidence that overturns the measurement: compile time,
startup, binary size, idle RSS; connection memory (74 MiB p50 for a bench process hosting the daemon
under 16 client threads on C); reply streaming, except the slow-consumer measurement F-25 asked for;
crypto dependencies (settled: none); chunked lock-free storage and point-read sharding (closed on
both hosts by the empirical pass; LC-14 stays retired); memory layout work (T-15's trigger reads
0.65× against a 1.5× threshold).

---

## 8. Constraints the executing agent is bound by — carry them into the work order

Non-negotiable; no task weakens one without an explicit decision routed to the author:

durable-before-visible; one sealer per ledger; one total epoch order; full retention; honest absence;
fold-never-field; product purity; self-describing amounts; GBS layering; **no `unsafe`**; **no
default-on-error**; **zero external dependencies**; no fabricated results; the thesis follows the
code with `MISMATCH-<id>` at every divergence; a red test is a result, never fixed by deletion;
honest refusal over silent fallback; `BLOCKED-<id>` on ambiguity.

Added in cycles 6 and 7 and now load-bearing: **apply-before-publish**; **the stated lock order
O < B < P < V < C, S a leaf**; every optimisation ships its guard and threshold in the same commit,
with the reversion transcript; where a repair is invisible to behaviour the guard reads the source;
no performance claim without a reproducible command, raw results, generated prose, host and barrier
in the header; `--publish` only from a stated host; micro-benchmarks are never contract results;
**no task requires network at execution time and no task installs anything**; attribution from the
executing session's own instructions, or "none required"; and — from F-32 — **the executor's report
carries every requirement of the order verbatim with a status beside it**.

---

## 9. Open questions

Settled, not reopened: LC-01, 02, 04, 08, 09, 10, 11, 12, 14. The empirical pass's positions on
LC-15 through LC-19 and its new LC-20 are in its §6; **confirm, sharpen or overturn each from the
source and the thesis text**, and treat LC-19 — does the reader-writer split change what "strictly
serializable" is claimed at the wire — as the question that matters most, now with F-26's finding
that the claim is void across a restart and F-27's that reads at the visible anchor were being
served from the base rather than the view.

Still open and yours: **LC-03** (frame it), **LC-05** (rendered-NULL, seed depth, refusal
boundaries), **LC-06** (process/allocator boundary — the empirical pass has allocator-level bytes
per row and process RSS on two hosts; say which the thesis should quote), **LC-07** (E23's slope —
and note that F-27's cost surfaces on exactly E23's base axis, which nobody has re-run since T-05),
**LC-13** (release checks), **LC-16** (durable by default — moot until T-01, then recommend).

---

## 10. What you must produce

A single work order containing, in this order:

0. **The preflight output, verbatim, with its admissibility table filled in.**
1. **An executive judgement** — where the three artefacts stand against the three goals, with
   **your definitions from §1**, and a one-paragraph verdict on the empirical work order: which of
   its findings you confirmed from the source, which you could not, and which you overturn.
2. **Findings**, each with a class (correctness, liveness, guarantee-bounded, claim-versus-code,
   instrument-gap, process, negative), an **EV score** (`impact × confidence ÷ cost`, 1–5 each, ties
   favour correctness), **HI/HS**, evidence with file and line, the argued cost, the repair, and
   **what the repair gives up**. Continue the numbering from **F-38**.
3. **A task list in dependency order** — integrated with the empirical pass's T-01…T-07, not beside
   them: say where yours slot in, which of theirs you would reorder and why. Each task carries what
   it closes, dependencies, files, baseline, target, method, acceptance, **guard and the reversion
   that must make it fail**, guardrails. Mark the **cut line**.
4. **Branch stacks and merge order** from `5ff365c` / `e803b7d`.
5. **A validation protocol.**
6. **Any Host C script** you need, with an explicit "**the author must run this now**" marker —
   expected to be at most one (§6.6).
7. **The open-questions ledger**, updated.
8. **Reporting requirements** for the executor, including the verbatim-requirements checklist, every
   missed target, guard transcripts, worktree status separating the four untracked files, and
   **exactly three material facts found while executing that the order did not cover.**

Order by `impact × confidence ÷ cost`. The binding constraint is correctness: a durability claim
that is not true outranks a liveness bug, which outranks a factor of two. And weigh one thing above
the rest: **two of this cycle's three largest findings were found by reading a single line** —
`epoch.to_string().into_bytes()` and `answered.anchor != anchor` — and confirmed afterwards by a
probe. There is no reason to believe those were the last two.
