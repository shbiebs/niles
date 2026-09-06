# Work Order 7 (consolidated) — from two Fable passes over one commit pair

**Consolidates** the empirical pass (Hosts A and C, `run4.sh`/`run5.sh`, five probes) and the
structural pass (reading, three witnesses) over Niles `c6/audit-cycle-7` (engine `d9c8699`) and GBS
`e803b7d`. Both passes were Fable; the GPT 6 Astra brief was refused by that model's classifier and
its territory was reassigned. Independence is therefore weaker than cycle 6's, and the structural
pass compensated by treating every empirical finding as a claim to verify from the source: it
confirmed all it could check and **overturned one** (F-38). Where the two passes are cited below,
the evidence class is stated so nothing is averaged.

**Executor:** Claude Opus, a later session. **Attribution:** from the executing session's own
instructions, or "none required". **Nothing was written into either repository by either pass.**

---

## §1 Executive judgement

**Concurrency is real; durability is not; currency safety stops at the wire.**

On ten cores, T-06 turned the scan-shaped read from flat at 1.83× under one mutex into 3.13× at
four connections and 4.24× at eight — the ≥3.0× target met, the ≥4.5× target missed by 6% against a
plateau the four performance cores set. Point reads went from a 95,000/s ceiling to 140,000/s, which
refutes cycle 6's F-23 on the host it was measured on. The lock-order fix costs nothing measurable;
the lock order O < B < P < V < C (S a leaf) holds across all three crates; chunked storage is closed
on both arms of its trigger on both hosts; LC-14 stays retired. Memory against base size is
sub-linear at 173 B/row at two million rows, and most of that is the language's value model, not the
engine's choice.

Three findings outrank all of it, none visible to any test or any published row:

- **F-26.** `nilestreamd --durable` fsyncs the idempotency key and the epoch *number*, not the rows.
  After SIGKILL and restart, 25,416 acknowledged inserts were gone and a retry of one was accepted as
  new. Every `durable` row ever published measures a barrier on a stub. Durable-before-visible holds
  within a process and is void across a restart. Two further holes sit on the failure path (F-39,
  F-40) and must close with it.
- **F-41.** The wire accepted an `INSERT` in a currency the schema never declared, then served
  324 USD + 500 of currency 999 as "824", the balance of account 1. Contribution 4 is proved for
  Niles programs and undischarged for data; every benchmark row is data.
- **F-27.** Under concurrent writes 87.4% of keyed reads bypass the maintained view (0.0% without
  writers), because an epoch is applied before it is visible and the engine discards an answer the
  algebra certifies as exact. The fix is one condition in `Rev::read`, confirmed licensed by
  Theorem 4.1's own step (3). LC-17 closes.

Also: the contract table flips MET/NOT MET between two instances of one host class (F-29); T-06
changed a public trait and GBS no longer builds against Niles (F-47); the `30.days` idempotency
window reaches no crate below the compiler and is infinite (F-43); the eviction budget bounds
resident values but not two per-key metadata maps (F-44); the daemon's banner states two falsehoods
(F-42); and the hash chain is **not** a placeholder — the README and the thesis are what is stale,
and three audits believed them (F-38).

**GBS efficiency is not admissible this cycle.** No product trace (F-24), state that does not survive
a restart (F-12), and a build that fails against current Niles (F-47). The only GBS tasks here are
the ones that make measurement possible.

---

## §2 Findings — the register

Full text in `docs/audit/cycle-7/fable-work-order.md` (F-26…F-37) and
`fable-structural-work-order.md` (F-38…F-47). EV = impact × confidence ÷ cost.

| # | finding | class | EV | source | closed by |
|---|---|---|--:|---|---|
| **F-41** | undeclared currency accepted at the wire; `group by acct` sums across currencies | correctness, Contribution 4 | 25 | structural, witness on D | T-01b |
| **F-26** | the durable daemon loses every appended row on restart | correctness | 25 | empirical, crash probe on A | T-01 |
| **F-39** | a session anchors at the applied epoch before its barrier; read-your-own-failed-write | correctness | 20 | structural | T-01 |
| **F-47** | T-06 changed `Base` (`&mut self → &self`); GBS `make gate` red; no cross-repo check | correctness/process | 20 | structural, GBS gate on D | T-00 |
| **F-38** | two hash chains that never meet; "placeholder hasher" is stale in README/thesis — it is SHA-256 (ADR 0003) | claim-vs-code | 15 | structural | T-01, T-06 |
| **F-44** | `reads_of`/`last_read` never pruned — the budget has an unbounded shadow | memory | 15 | structural | T-05 |
| **F-27** | 87.4% keyed-read fallback under writes; applied > visible by design; discarded answers are exact | correctness/efficiency | 12.5 | empirical, exact counter | T-02 |
| **F-46** | C.5's determinism obligation has no engine-side test | instrument | 12 | structural | T-07 |
| **F-40** | a failed barrier is masked by the next success (`fetch_max` contiguity; sealer continues after I/O error) | correctness | 10 | structural | T-01, LC-21 |
| **F-43** | `idem window 30.days` never reaches the engine; window infinite; reopen O(history) | correctness/memory | 10 | structural | T-05 |
| **F-42** | banner and four comments assert the engine mutex and durability; both false | claim-vs-code | 10 | structural | T-06 |
| **F-33** | clippy red under rustc 1.97.1 on two `niles-ir` lints; pin says `stable`, means 1.95.0 | instrument | 10 | empirical, run5 on C | T-04a |
| **F-29** | `report` 2.68× MET on one A instance, 2.17× on another — and 2.30× there *before* cycle 6 | wrong-measurement | 8.3 | empirical, 3 arms on A | T-04 |
| **F-28** | T-06 on ten cores: fold 1.83× flat → 3.13×/4.24×; point 95k → 140k/s; trigger does not fire | instrument result | — | empirical, run4 on C | closed |
| **F-30** | lock order complete, three crates; fix costs nothing; guard's `.lock()` search is fragile | liveness | — | both | T-06 (guard) |
| **F-45** | plan cache bounded only by `Close` | memory, minor | 4 | structural | noted |
| F-31 | 266 → 173 B/row from 20k to 2M rows; T-15's trigger reads 0.65× | negative | — | empirical | closed |
| F-32 | a specified counter was dropped twice and nothing surfaced it | process | — | empirical | §8 checklist |
| F-34 | no writer starvation at 8:1 on C (LC-18) | negative | — | empirical | closed |
| F-35 | the segment survives a mid-batch SIGKILL; the daemon does not use it | instrument | — | empirical | T-01 |
| F-36 | bench-process RSS 74 MiB p50 under 16 threads on C | negative | — | empirical | closed |
| F-37 | T-03 confirmed on C; pre-T-03 code fails a different subset each run | verification | — | empirical, run5 | closed |

**Verified, not findings:** T-02's condition is Theorem 4.1's certification interval (pinned, `Full`
and first-delta paths all fall on the reconstruct side); the `append`→sealer channel is unbounded;
`e803b7d`'s consumed-map invariant holds at both writers; Appendix E is honest (the parser in Niles
exists; the type-checker does not); §3's data-race text is hedged but says "two lock disciplines"
against six locks; the arm64 corpus is discharged by `run4.sh` (`counterproposal` 4/4).

### `MISMATCH` register

| id | thesis | code |
|---|---|---|
| `MISMATCH-durability-restart` | 06:38, 07:19, 09:457 | true of `nilestream-ledger`, false of the daemon (F-26) |
| `MISMATCH-currency-at-wire` | Contribution 4 | undischarged at the wire (F-41) |
| `MISMATCH-contract-instance` | 09:686–702, absolute figures from an unnamed instance | flips across instances (F-29) |
| `MISMATCH-placeholder-hasher` | 07:69, `README.md:55` | SHA-256 since ADR 0003 (F-38) |
| `MISMATCH-idem-window` | §3.20 "a defined admission semantics" | never reaches the engine (F-43) |
| `MISMATCH-lock-count` | 03:231 "two lock disciplines" | six locks, four atomics |

---

## §3 Guarantees, checked

| commitment | status |
|---|---|
| durable-before-visible | within a process: yes; across a restart: **no** (F-26); on the failure path: **no** (F-39, F-40) |
| one sealer per ledger | yes |
| one total epoch order | in memory yes; after a restart the two clocks diverge (F-26) |
| full retention | **no** across a restart (F-26) |
| honest absence | yes (post-restart query returned no row, not zero) |
| honest refusal over silent fallback | **no** at the wire: an undeclared currency gets `INSERT 0 2` (F-41) |
| cannot mismatch currencies (Contribution 4) | programs: yes; data: **no** (F-41) |
| no `unsafe`, zero external dependencies, no default-on-error | yes |
| apply-before-publish | yes — and it is the cause of F-27 |
| stated lock order | yes, all three crates |
| GBS layering, product purity, fold-never-field | not exercised; **GBS does not build against Niles** (F-47) |

---

## §4 Global constraints

Unchanged from work order 6, plus, from this cycle: **a stated lock order O < B < P < V < C, S a
leaf, and any new lock placed in it; the executor's report carries every requirement of this order
verbatim with a status beside it (F-32); public traits with out-of-tree implementors (`Base`,
`Serving`) are listed in `SPEC-ENGINE.md` and a change to one is a cross-repository commit (F-47);
the gate is run on 1.95.0 and on the newest stable the author has, and a lint that only the newer
one emits is fixed rather than pinned away (F-33).**

---

## §5 Tasks

Dependency order. **Cut line after T-04a.** Every task: guard proved failing on the reverted change
in a disposable worktree, transcript in the report. Baselines measured on the executing host in the
same session; targets are ratios against them. **LC-21 must be decided before T-01 starts.**

### T-00 — GBS builds against Niles again, and a check exists — closes F-47

*Files:* `gbs-nilestream/src/evicting.rs` (`&self` on `reconstruct` and `deltas_at`; `&base` at
`:457`); a Niles test `adapter_builds_when_present` gated on `GBS_ROOT`; `docs/SPEC-ENGINE.md`
(public traits with downstreams). *Target:* GBS `make gate` green against the named Niles SHA; the
Niles test skips **by name** when `GBS_ROOT` is unset and fails when the adapter does not compile.
*Guard:* revert the adapter signature → the Niles test fails with the E0053 text. *Merge:* both
halves in one window; the GBS commit records Niles's full SHA.

### T-01 — Durable means the rows come back — closes F-26, F-35, F-38 (chain), F-39, F-40

*Files:* `nilestream-server/src/rev_engine.rs` (`append`, `with_durable`, `DurableSink`,
`Pending::wait`), `session.rs` (`insert`, `observe`), `daemon.rs` (`serve`),
`nilestream-ledger/src/{sequencer,segment}.rs`, `proto-engine/src/ledger.rs` (chain), the banner,
`SPEC-ENGINE.md`.
*Baseline:* `probes/crash` — 8 writers, SIGKILL at 2 s, restart: frontier returns to the seed, 0 of
~25k acknowledged inserts present, a retried id accepted as new.
*Target:* after restart, for every writer account `sum(amt) ≥ acked` and `sum(amt) − acked ≤` the
largest batch in flight; frontier ≥ the last acknowledged epoch; a retry of an acknowledged id is
refused as `Duplicate` **by the daemon**; txns-per-barrier at 16 connections on the executing host
within 10% of the same-session pre-task figure.
*Method:* **(a)** the payload is a canonical encoding of the epoch's `Vec<Row>`; **(b) one chain** —
the ledger's `EpochRec.hash` *is* the segment record's hash over that encoding, and replay
**verifies** each recomputed hash against the record before applying, refusing at the first
mismatch; **(c)** `with_durable` replays every recovered record through `Ledger::submit` in order
and sets `visible` to the recovered head; **(d)** the invariant `record.epoch == ledger_epoch −
seed_head` is stated and asserted on replay; **(e)** the session observes only after `wait()`
succeeds; **(f)** a barrier failure is **fail-stop** — the sealer stops, later submits get
`ShuttingDown`, the daemon refuses appends until reopened (per LC-21); **(g)** `interpret` never
turns a sequencer `Duplicate` into a commit the ledger has not seen.
*Acceptance:* the crash protocol as a `#[test]` (spawn the binary, SIGKILL, restart, verify); the
visibility, recovery and idempotency suites green; `make fsync-proof` green.
*Guards, each proved by reversion:* payload back to the epoch string → the restart test fails on
the first account; tamper one byte of one record's payload → reopen refuses at that record rather
than rebuilding past it; inject a barrier failure at *n* then succeed at *n+1* → a read at the
visible frontier must not see *n*'s rows (fails on `fetch_max` alone); observe-before-wait → the
failed-barrier session reads its own rows.
*Guardrails:* `SyncPolicy::Always` only; one sealer; no wire change; `MISMATCH-durability-restart`
resolved in the thesis or left marked.

### T-01b — The wire enforces the schema's currency premise — closes F-41

*Files:* `session.rs` (`parse_insert`, `insert`), `rev_engine.rs` (the fold path's refusal,
`serve_path_at`), `niles-lang` diagnostics (a named code).
*Baseline:* the F-41 witness — `INSERT 0 2` for currency 999; "824" for account 1.
*Target:* an `INSERT` naming an undeclared currency is refused by name (SQLSTATE `22023`); an `amt`
outside the declared scale is refused; `sum(amt) group by acct` over a base holding more than one
currency is **refused** at the engine with the compiler's own diagnostic, never folded; `explain`
reports the path `query` will actually take.
*Acceptance:* the witness as a test — the second `select` refuses; the `group by acct, cur` form
answers both rows.
*Guard:* remove the ingress check → the witness test fails on "824".
*Guardrails:* no change to the compiler's solver; the schema is the single source of declared
currencies. **Sequenced before T-02 and T-03**, so the mixed row never measures a cross-currency fold.

### T-02 — The view answers exactly at the anchor when it can, and says how often it could not — closes F-27, F-32

*Files:* `nilestream-core/src/rev.rs` (`read`), `rev_engine.rs` (`answer_from_view`,
`read_stats`), `session.rs` (`nilestream_stats`), the E19 renderer.
*Baseline:* 87.4% fallback under 4r/2w on A; 0.0% readers alone.
*Target:* fallback rate under 4r/2w **≤ 5%** (the residual is keys with a delta in
`(anchor, applied]`, which must still reconstruct); `hits` counts only answers served, the
discarded-hit count is its own column; a `fallbacks` column on `select nilestream_stats`; mixed-phase
read p50 within 1.2× of readers-alone.
*Method:* in `Rev::read`, for a resident unpinned `Present(v, stamp)` with `stamp ≤ anchor ≤
effective`, return `Anchored { v, anchor }` — Theorem 4.1's certification interval; count the
remaining mismatches at the engine.
*Acceptance:* a `nilestream-core` test that a key **with** a delta in `(anchor, applied]` still
reconstructs; a pinned entry at `anchor > stamp` still reconstructs; `oracle_differential` under
concurrent append/read at random anchors; the deadline test green.
*Guard:* revert the condition → the ≤5% test fails at ~87%; revert the counter → the renderer
refuses the row.
*Guardrails:* pinned entries keep reconstructing; no change to eviction; LC-20 is not decided here.

### T-03 — The mixed workload becomes a row nobody can omit

*Files:* `bank-bench/src/bin/bench.rs` (a `mixed` level in E19), `workloads.rs`, the renderer,
`results/MANIFEST.csv`. *Target:* E19 gains `mixed` rows per connection level (readers N, writers
⌈N/2⌉) reporting reads/s, read p50/p99, writes/s, write p99, **fallback rate**, `max_batch`, lock
wait p99; `--nls-only` supports it; the renderer refuses a `mixed` row without the fallback column;
on the executing host the row reproduces `probes/mixed`'s shape within MAD. *Method:* lift
`probes/mixed`'s three phases into `workloads::concurrent`. *Guard:* the renderer's refusal,
reverted → a row publishes without the column. *Guardrails:* never `--publish` from the container;
`NOT RUN` without T-02.

### T-04 — The contract is a same-session A/B, and C is the reference — closes F-29

*Files:* `docs/SPEC-ENGINE.md` Part 0, `docs/BENCHMARK.md`, `bench.rs` (`--baseline <commit>` or
a documented two-arm invocation), `results/MANIFEST.csv`, thesis 09 §E16. *Target:* every contract
ratio carries `(host, instance id, session, barrier, baseline commit)`; `MET` only against a
baseline measured in the same session on the same host; C named as the reference host with its
script; the three-arm table from F-29 recorded as the demonstration; `MISMATCH-contract-instance`
resolved or marked. *Guard:* a test that the rendered E16 header names a baseline commit.
*Guardrails:* no target relaxation.

### T-04a — Two lints and one pin — closes F-33

*Files:* the two `niles-ir` sites `run5.sh` §D names; `rust-toolchain.toml`. *Target:* clippy green
under 1.95.0 **and** 1.97.1; the toolchain file pins `1.95.0` — **the author's step, needs network
once** — and its explanatory paragraph is deleted as it asks. *Guard:* the gate on both toolchains,
both recorded.

---
*Cut line.*

### T-05 — The window is a window; the budget bounds everything — closes F-43, F-44

Carry `window` through the IR to the sequencer and the ledger's `seen`; prune by epoch age; evict
`reads_of`/`last_read` with the entry; stats columns for both retained sets. *Guards:* commit 2×
budget distinct keys → metadata maps' length ≤ budget; a key older than the window is admitted as
new; reverted, each fails.

### T-06 — Say what is true — closes F-42, F-38 (docs), F-30 (guard), the `MISMATCH` register

Banner, `lockstats.rs` header, the four comments; `README.md:55`; `thesis/07:69`; the lock-order
guard names the lock (`runtime` + `.lock()`); each `MISMATCH` resolved in prose or left as a marked
sentence. *Guard:* a source test that the banner names no mutex and does not say "durable" unless
T-01 has landed.

### T-07 — The engine's determinism fixture — closes F-46

E1's rendered output hashed, committed from x86_64, asserted on arm64 by the next Host C script.

### T-08 — Fold plateau attribution on C — F-28's residual

`sample`/Instruments on the fold at 8–16 connections; establish whether the 4.4× plateau is the four
performance cores or `report_from_view`'s V hold. Only if the latter: shard the resident iteration.

### T-09 — E24 proper: RSS under load at 20k / 200k / 2M on C — via a `run6.sh`.

### T-10 — GBS: lifecycle and holds on the ledger — F-12, as F-26's twin (work order 6's T-08)

After T-01, so both replay designs share one encoding discipline. **No GBS efficiency task before
this and T-14 (product trace).**

---

## §6 Branch stacks and merge order

**Niles**, from `c6/audit-cycle-7`: `c7/00-adapter-check` → `c7/01-durable-rows` →
`c7/01b-currency-at-wire` → `c7/02-exact-at-anchor` → `c7/03-mixed-row` → `c7/04-contract-ab` →
`c7/04a-lints` → (below the line) `c7/05-window` → `c7/06-say-what-is-true` → `c7/07-determinism`.
**GBS**, from `c6/07-hold-index`: `c7/00-adapter` (records Niles's SHA) → `c7/10-lifecycle-ledger`.
T-00's halves merge together; T-01b before T-02 and T-03; the adapter is re-validated after every
Niles commit that touches `Base` or `Serving`.

---

## §7 Validation protocol

| command | green means | red means |
|---|---|---|
| `bash docs/audit/cycle-7/preflight.sh` | environment stated | nothing later interpretable |
| fmt; clippy on 1.95.0 **and** the newest stable | the gate is about the code | it is about the calendar (F-33) |
| `cargo test --offline --workspace`, both trees, serial, `PGPORT` set | semantics | no performance claim |
| GBS `make gate` against the named Niles SHA | the pair interoperates | cross-repo claims blocked (F-47) |
| the T-01 crash test; the tamper test; the injected-failure test | rows survive; the chain is verified; a failed epoch is never visible | every `durable` row is void |
| the F-41 witness test | the wire enforces the schema | Contribution 4 is program-only |
| the T-02 ≤5% fallback test | the view serves under writes | the mechanism is off under load |
| E19 `mixed` row with a fallback column | the number cannot hide | F-27's class is open |
| E16 with a same-session baseline | the contract is a ratio | it is an instance's coin |
| `make reproduce` clean; `make fsync-proof` | reproducible; the barrier reaches the kernel | stale; volatile |
| `run4.sh` re-run on C after T-01/T-02 | durable curve within 10% of 7.80× at 16; mixed rows within MAD of F-28/F-34 | the record grew past the barrier's budget |

Variance, concurrency and host policies are cycle 6's, unchanged.

---

## §8 Open questions

Settled, not reopened: LC-01, 02, 04, 08, 09, 10, 11, 12, 14, **17** (closed by T-02's algebra
verdict), **18** (no starvation at 8:1 on C; Linux 16:1 re-check once T-03 exists).

- **LC-21 (new, gates T-01) — fail-stop on barrier failure?** Accepted: (A) fail-stop — the sealer
  stops, the daemon refuses appends until reopened; simplest, and what a ledger should do; costs
  availability under a storage fault. (B) a marked-void epoch with a tombstone record in the segment
  and `visible` skipping it; keeps serving; costs a second kind of record and a non-contiguous
  visibility rule. *Recommended: A.* Reversal cost: the sealer's error arm and one test.
- **LC-15** — sharpened by F-29: a same-session A/B against a named baseline commit on the executing
  host; C is the reference.
- **LC-16** — moot until T-01; then durable by default with `--volatile` for benchmarks.
- **LC-19** — three clauses for the thesis: within a process the split preserves the wire claim;
  across a restart it is void until T-01; on the failure path it is void until F-39/F-40 close.
- **LC-20** — to the thesis: a bounded-staleness rung anchored at *applied* is ℓ₀ with K = in-flight
  batch depth; the ladder has the rung, nothing lets a session ask for it. Not an engineering task.
- **LC-03** — one term of the language's mandated bytes-per-row; measure against 173 B/row.
- **LC-06** — quote allocator-level bytes per row; report process RSS per host as context only.
- LC-05, LC-07 (F-27's cost surfaces on E23's base axis), LC-13 — unchanged.

---

## §9 Host C scripts

`run4.sh` and `run5.sh` have run; results under `~/Documents/niles-hostc/results4-…` and
`results5-…`. **The author must run `bash ~/Documents/niles-hostc/run4.sh` again after T-01 and
T-02 land**, and the executor compares the durable curve and mixed rows against F-28/F-34. No other
script is required by this order.

---

## §10 Reporting requirements

Cycle 6's, plus:

- **The requirements checklist:** every line of §5 above copied verbatim, with `done` / `not done:
  <why>` beside it. A dropped line is a finding against the executor (F-32).
- The `MISMATCH` register, each entry resolved or carried.
- The crash protocol, the tamper test and the injected-failure transcript before and after T-01.
- The F-41 witness before and after T-01b.
- The fallback rate under 4r/2w before and after T-02.
- GBS `make gate` against the named Niles SHA.
- Both toolchains' clippy output.
- Worktree status separating the four untracked files (`.DS_Store` 10,244 B, `AGENTS.md` 16,639 B,
  `thesis/.DS_Store` 8,196 B, `thesis/Niles-Thesis.pdf` 816,110 B) from task changes; PostgreSQL
  stopped and its log kept; the attribution instruction or "none required".
- **Exactly three material facts found while executing that this order did not cover.**
