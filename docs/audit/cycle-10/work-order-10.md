# Cycle 10 — consolidated work order (Astra + Fable)

Audit date: 2026-09-08. This is the sole audit deliverable and the controlling document for
cycle 10. Astra read source and existing evidence; Astra did not build, test, benchmark, modify
either repository, or make commits. Opus executes the tasks below offline. This consolidates the
updated Fable work order (preserved beside it as `work-order-10-fable.md`) with Astra's
independent audit and the author's subsequent complete gate results. The reconciliation in §1A
resolves scope and target conflicts; **the source work orders are evidence, not additional
instructions.**

## 0. Baseline, access, and admissibility

`N` means `/Users/checolino/Documents/niles`; `G` means `/Users/checolino/Documents/GBS`. Source
references below use these absolute roots and one-based lines. Numbers from archived runs are
explicitly historical evidence, not Cycle 10 performance baselines.

| Repository or ref | Verified state at initial audit/sync | Consequence |
|---|---|---|
| N audited source, then local `c7/01-durable-rows` and `c10/00-audit` | `85481890808c8b808c46a7534870abf40e8b0de5` | Cycle 10 starting commit. |
| N runtime predecessor, `c9/07-pending` | `53b40f6dcde3d4fe110f5c3d75e0018741f6768e` | Diff to 8548189 contains only the two Cycle 10 briefs and preflight, three files, 1,670 added lines; runtime source is identical. |
| G before author sync | `963e4d9ed8ba503b9ea629b5173f994f026f2d71` | Preflight's GBS row was behind; it was not used as the Cycle 10 implementation baseline. |
| G after requested author sync | `688919c87852c86d8e8c3abc0af2135db60a5197`, clean, `c7/00-adapter` | Correct baseline now present. |
| Author's `git ls-remote` | N c7 and c10 at 8548189; N c9 at 53b40f6; G c7 at 688919c | Both owed Cycle 9 changes are present remotely. N's differing refs are the verified documentation delta, not an outstanding runtime sync. |
| Author's G workspace Clippy | `cargo +1.97.1 clippy --offline --all-targets -- -D warnings`: exit 0 | This alone does not check the excluded Nilestream adapter; `make gate` includes it. |
| Astra GitHub probes | Noninteractive, bounded probes failed to resolve github.com | Agent network failure, not a finding of denied repository authorization. Author's network succeeded. Local source access sufficed. |

The brief's GBS sync block named bundle refs that do not exist and then attempted to force-update
the checked-out branch. The actual exports are `c9/01-batch-seq` at
`447544ded3e4306ceeed7300c6354e470cec8e9f` and `c9/02-idem-key-only` at 688919c. The author
fetched those refs, fast-forwarded the checked-out c7 branch, pushed it, and pasted successful
outputs. **The Cycle 9 GBS sync is closed; do not repeat the broken block.** Repeat every new,
still-owed Cycle 10 sync at each landing until acknowledged.

### Filled admissibility table

| Field | Cycle 10 verdict |
|---|---|
| Cores actually granted | Host C reports 10 native cores; no cgroup quota applies. This is not ten equal-performance cores or ten exclusively reserved cores. Record actual hardware and load with each run. |
| Barrier, median, rate | Author-requested ordinary `fsync`, 17.9 µs median, 55,944/s, 15.1–56.4 µs range. Seven probes; this is preflight evidence, not a benchmark passing the timing protocol. |
| Storage evidence | Incomplete. The printed sealed root mount does not establish the actual writable data volume backing the segment. Neither volatility nor power-loss persistence is established by this row. |
| Publish durability rows | **Blocked now** on verifying the actual segment path and the production Darwin barrier. The brief's historical ~255 F_FULLFSYNC/s is not this probe and is not a fresh baseline. |
| Publish >3-core curves | Host C can measure connection scaling on its 10-core hardware after provenance/load checks. A 12-reader/6-writer point is an oversubscribed connection count, not an 18-core claim. Publication remains a separate author-controlled action. |
| Newer toolchain | 1.97.1 installed; requested G workspace Clippy passed. Full paired gates have separate results below. Never equate `stable` with 1.95.0 without resolving it. |
| Valgrind attribution | Unsupported on current C setup; leave per-function instruction costs to an already-equipped execution host/Fable. No installation task. |
| PostgreSQL comparison | Blocked: no service on 5433; PATH identifies PG18.6, not PG16. An installed PG16 elsewhere is unverified. No wal_sync_method measured. |
| Container | 2 reported cores, stable resolving to 1.95.0, Valgrind 3.22, PG16.13 on 5432 but not 5433. No exclusive CPU grant proved. No container publication or durability/scaling extrapolation. |

**T00 is the script task that produces the current Host C snapshot/merge performance baseline**,
and all downstream performance pass lines use its figures.

The verbatim container preflight (16:05:54Z), Mac preflight (16:32:46Z) and `c10-rwlock.sh`
transcript (200/200 writer-preferring on Darwin) are recorded in `work-order-10-fable.md` §0.1,
§0.2a and §0.2b, and in this cycle's execution report.

## 1. Executive judgement

**Nilestream:** immutable-prefix reconstruction is a sound direction, but a stable integer head
cannot make a mutable `Vec` safe to read without its lock. Pin owned immutable rows, their lookup
structure and the relevant metadata; release B before folding. Keep the returned answer at the
requested anchor even if a separately certified result can be installed later. Before speed work,
repair the directly served V→B stats inversion and the report's split certification/copy, and
settle the mixed public-API wait cycle. These source-derived correctness/liveness risks outrank
throughput.

**Niles:** the next checked-twice saving is not permission to remove admission validation. Static
money types, declared schema/window metadata and supported query shape can be cached or represented
by a verified object. Conservation of incoming postings, representable numeric values, duplicates,
live balances and wire currencies remain data obligations. `conserve per` currently stores names
but does not carry arbitrary partition semantics into admission. The first action there is honest
refusal and a precise claim, not deleting the per-currency validator.

**GBS:** the author's correct tree is now available and the simple newer-lint gate passed. Neither
durable product lifecycle replay (F-12) nor a benchmark-shaping product trace (F-24) is closed. The
first product task remains a deterministic thousand-share syndicated distribution with one joint
durable/wire/restart witness. No GBS efficiency task is admitted yet.

The historical 12r6w medians yield `163753 / 160837.6 = 1.018126…`. This is neither the old
collapse nor the required 1.25× gain. At that point a budget of 2,500 exceeds the 2,000-account
single-currency working set; it is not evidence about quarter-resident eviction. Historical pinned
and join counters do not imply a 22× opportunity: they count different events, pinned values can
serve later reads at that same anchor, and the daemon currently throws joined results away before
retrying. Measure gap, reuse and actual saved work.

## 1A. Reconciliation of the two audits — controlling decisions

**Integration parent.** N's last verified local main head before this order is
`96b6d9e4d962bb35e64a13d1345d821ae271cb36`; compared with the audited 8548189 it changes only
audit documentation and adds `c10-baselock.sh`/`c10-rwlock.sh`. Runtime source is unchanged, so
the exact-8548189 paired gate results remain applicable to runtime. Execution starts from the
commit of this consolidated order atop that audit-only chain. Do not rewind c7 to 8548189 or
overwrite newer audit commits.

The Fable file's opening/executive language saying GBS is behind or cannot compile is superseded by
the author's successful sync and real adapter compile. **Compiles** and **passes all adapter
tests** remain different: the latter is false in the three-run evidence of §9.

### Finding-by-finding disposition

| Fable finding | Astra counterpart | Consolidated verdict and reason | Owner |
|---|---|---|---|
| F-10-01 stats lock inversion | A10-01 | **Confirmed independently in source; reproduced by Fable in container.** Ship deterministic latches plus a wire-progress guard. | T01 |
| F-10-02 shared/exclusive lock scope | A10-08/10 | **Confirmed-narrowed.** Separate read/write histograms required. Summing overlapping reader hold durations is reader-lock-time, not exclusive wall-clock occupancy; 2.82/0.98≈2.88 does not establish which activity causes every tail. | T00 |
| F-10-03 stale GBS pair | A10-16 | **Closed as current state.** Correct bundle refs synced; pair compiles. Preserve the historical bad-instruction finding. New adapter tests are separate red rows. | T00a |
| F-10-04 absent sibling passes | BLOCKED-adapter | **Confirmed independently in source.** Explicit opt-out must be reported as not-run and must not satisfy the paired audit gate. | T00a |
| F-10-05 gap mostly one | A10-11 | **Accepted as Fable-reported container measurement; inference narrowed.** Supports a capped-merge experiment, not guaranteed avoided reconstructions or constant-cost `deltas_at`. | T04 |
| F-10-06 SHA-256 dominates probe | A10-14 | **Accepted scoped attribution; design not accepted as a ready repair.** Parent-hash handoff and H ordering need their own design proof. | T10 |
| F-10-07 cfg(test) in doc comment | A10-13 | **Confirmed independently in source.** Column-zero cutoff alone still loses legitimate items after a real intermediate test item. | T00a minimal fix; T05 general policy |
| F-10-08 generation second hazard | A10-12 | **Confirmed.** Stale resident/result loss is established, permanent waiter stranding is not. Retain structural generation guard. | T01 |
| F-10-09 stale markers/comment | A9-F05 | **Confirmed.** Strike active repaired marker, remove obsolete `ignored_windows` prose; preserve historical reports with dated correction. | T00a/T05 |
| F-10-10 cold plan cost | — | **Accepted as Fable-reported scoped measurement.** 340.8M/1952≈174,590 inclusive instructions/miss. | T06/T08 |
| F-10-11 old baseline sweep | A10-09 | **Confirmed-narrowed.** Earlier audit-time measurements are legitimate historical evidence, but not new Cycle 10 baselines. | T00 registry |
| F-10-12 window negative witness | C9-02.4 | **Useful test; proposed target direction corrected.** The old gap was admission-new/sealer-duplicate. A correct expired retry is accepted by both, not refused by both. | T00a |
| F-10-13 Darwin storage probe | A10-17 | **Confirmed-narrowed.** Do not call all ordinary fsync "no barrier," assume a Python constant exists, or certify a Data mount merely by re-running `df` on a firmlink. | T00a |
| No counterpart | A10-02…07 | **Astra source findings retained:** split report certification, mixed legacy wait cycle, discarded joins/outer lock, abandoned flight capacity, whole-graph refusal, unimplemented conserve keys. | T01/T02 |
| No counterpart | A10-18/19/20 | **Author-measured after that order:** adapter tamper/guaranteed-hit guards red on all runs; N allocation budget red on both Mac toolchains. Binding gate evidence. | T00a |

### Resolved design and priority disagreements

1. **Snapshot landing versus smaller repairs first:** above-cut efficiency work is bounded merge
   and measured served checkpoints. The full storage snapshot is T03's below-cut design/prototype.
2. **Merge before snapshot:** allowed on the current daemon using its existing B acquisition, but
   never by calling an arbitrary internally-locking Base under V. Epoch cap 8 is a candidate
   starting policy, **not a row/allocation bound**: `Ledger::deltas_at` visits the entire record
   and allocates a key per posting. Bound rows/work too. No new lock or second B acquisition.
3. **Merge targets:** replace fixed rates with a deterministic eligible-landing contract and a
   measured current-control comparison; report rates as hypotheses, not pass requirements.
4. **Checkpoints:** T07 above the cut. Default 16 with 0 ablation; measure 0/16/64/256 and actual
   account-index visits. A C+1 target-key bound is not a bound on mixed-currency physical work;
   either change the index or state the limit. The p99 target is against the *immediate
   merge-enabled control*. A missed speed target is recorded, not repaired by another baseline.
5. **Partition semantics:** reject "already a prohibition". No admission use of arbitrary
   `conserve_keys`; repeated rules overwrite; per-group net zero is weaker than prohibiting paired
   cross-partition flow. T02 truthful refusal above cut.
6. **Generated API:** adopt the ZSet witness above cut but use an inclusion-aware fix.
7. **Hash off B / new H:** below-cut design only, T10/LC-42. No unreviewed lock-order change.
8. **Gate meaning:** neither Valgrind nor fsync-proof is a prerequisite of `make gate`. PG absence,
   the adapter assertion and the memory budget are its actual observed issues.

The first performance script is the already-delivered **c10-baselock.sh, repaired in T00**. Its
current default baseline `c7/01-durable-rows` moves to the candidate after a landing; first-run
A=B and reused stale worktrees are both possible. Resolve/freeze baseline once into a manifest, use
an explicit predecessor SHA for later arms, verify actual worktree HEAD, and enforce the §6 fault
corpus before taking any number from it.

## 2. Findings and structural evidence

Evidence labels: **S** = read from source; **D** = inferred from a document or re-derived from
archived data, not a new run; **A** = measured by the author on request. HI = independently
identified; HS = shaped by the brief or carried finding. EV = impact × confidence ÷ cost; each
input is 1–5.

| ID | Class; origin; evidence | Finding, source and cost | EV | Repair and what it gives up |
|---|---|---|---|---|
| A10-01 / F-10-01 | liveness; HI; S+container | `rev_engine.rs:992–1000` retains V across `self.base()`. `append:872,934` takes B→V. A stats request and append can deadlock. Source guard at 3159 lists only two functions. | 25 | Copy stats under a valid order or release V before B; document a non-atomic diagnostic snapshot. T01. |
| A10-02 | correctness; HI; S | `report_shape:1226–1233` checks full/applied=a under V; `report_from_view:1295–1328` reacquires V and copies current rows without rechecking. Values from e>a can be labelled a. | 25 | Certify and copy within the same V guard; otherwise fall back after dropping V. T01. |
| A10-03 | liveness; HI; S | `rev.rs:474–494`: synchronous `read(&mut self)` waits on an existing flight while the caller retains exclusive Rev/V. Owner requires that Rev for `finish_fold`. | 12.5 | Make the legacy read incapable of joining while holding Rev. T01. |
| A10-04 | guarantee-bounded; HI; S | Server `query:755–764` discards `w.wait()` and retries. One logical read can reacquire B; `RwLock<RevEngine>::query:1044–1052` retains outer O throughout the wait. | 10 | Consume the joined anchored value with sufficient metadata; clone an owned generation before waiting. T01. |
| A10-05 | guarantee-bounded; HI; S | `begin_read:517–550`, `reap_cancelled:730`, `WaitTicket:211`: cancelled flights are reaped only on a miss for their own key; dropped wait tickets never release their reservation. 256 abandoned keys can exhaust sharing capacity. | 10 | Bounded maintenance/cancellation and exact waiter release; separate refusal reasons. T01. |
| A10-06 | correctness; HS; S | `Runtime::install` checks output aggregates, not the reachable graph; thesis 4:37 retains A9-F15. | 12.5 | Refuse unsupported whole graphs at public install. T02. |
| A10-07 | correctness; HI; S | `parser.rs:646–664` accepts repeated conserve clauses; `resolve.rs:554–556` overwrites. `conserve_keys` has no consumer; `ledger.rs:384–410` enforces per-(txn,cur). | 12.5 | Reject unsupported/repeated keys until a real partition rule exists. T02, LC-41. |
| A10-08 / F-10-02 | instrument-gap; HI; S | `answer_from_view:1447,1461,1484` omits joins from slowest-16 and the second V acquisition from `view_wait_us`; aggregate ENGINE_LOCK combines readers/writers; `ReadStats` omits `deferred_merges`. | 10 | Measure logical-read phases, both V waits, mode and gap; expose all flight counters including zero. T00. |
| A10-09 | wrong-measurement; HI; D/S | Archive arithmetic used mean(MADs), not RMS pooling; `c9-pending.sh` does not implement all the retarget/deadline/refusal guarantees attributed to it. | 20 | Correct arithmetic and causal language, then measure a fresh baseline with an audited harness. T00. |
| A10-10 | guarantee-bounded; HS; S/D | `answer_from_view:1428–1477` folds under B; `Ledger:108,127,506–549` needs mutable-container-backed indices/rows. | 5 | Owned immutable prefix plus measured bounded capture. T03. |
| A10-11 / F-10-05 | instrument-gap; HS; D/S | No archived e−a distribution; Fable's container histogram exists; keyed suffix cost unmeasured. Pinning is exact and can be reused historically. | 5 | Preserve answer a; merge only a certified available suffix; pin when unavailable. T04. |
| A10-12 / F-10-08 | stale-claim; HS; S | Generation does not establish 5a; completion publication at `rev.rs:655` is outside `mine`. A retained Present means M is not always Pending during a flight. | 20 | Separate answer correctness, certification and ownership/cost claims. T01. |
| A10-13 / F-10-07 | instrument-gap; HS; S | `gen-appendix-d.py:70–79` truncates at the first `cfg(test)` **string**; `eval.rs:11` has it in a doc comment. `include-results.py:291–310` only recognises matched BEGIN/END pairs. | 10 | Inclusion policy with positive and omission mutants. T00a minimal; T05 general. |
| A10-14 | negative; HS; S | Conservation report retains undecided/may-violate obligations (`typecheck.rs:126–143,1977`); direct append has no trusted compiler certificate. | 10 | Cost the sweep first; cache proven schema/plan facts only. T06. |
| A10-15 | instrument-gap; HS; S/D | G3 uses separate wire/reopen evidence; lifecycle events remain in-memory. Its "reads equals upqueries in every row" is contradicted by term 6/5, trade loans 22/20, trading 16/14. | 6.67 | Joint product witness; repair generated prose. TG01. |
| A10-16 / F-10-03 | instrument-gap; HS; A/S | Broken brief sync refs; corrected author sync succeeded. | 15 | Validate exported bundle refs and the checked-out branch before every sync. |
| A10-17 / F-10-13 | negative; HS; S/A | PG absence is a host fact. Darwin `os.fsync` is not `F_FULLFSYNC`; the root mount line is a firmlink artefact. | 15 | Per-command status; probe the actual path and supported syscall. T00a. |
| A10-18 | instrument-gap; HI; A/S | G `a_tampered_record_is_refused_and_the_segment_is_not_truncated` fails: `gbs-nilestream/src/lib.rs:774` selects `buf.len()/2` despite saying third-record payload; the reader reports a damaged header at offset 660 and refuses without truncation; the assertion at 791 demands `damaged at offset`. **A real red test and a stale instrument, not accepted corruption.** | 20 | Decode record boundaries, select a proven interior payload byte and a separate header check-word byte. T00a. |
| A10-19 | guarantee-bounded; HI; A | N `make reproduce` exits nonzero for `rev_metadata_2x_budget`: 44,165 allocations on Mac against 34,165 committed, +2 allocations and +112 bytes per key, identical `live` and `peak`. Reproduced on both Mac toolchains. | 10 | Attribute with host/toolchain-labelled runs; repair with a reverting cost guard or record as unresolved without raising the budget. T00a. |
| A10-20 | instrument-gap; HI; A/S | All three G adapter runs fail `money_is_conserved_at_every_anchor_under_continuous_eviction:250` on the **hit-coverage** assertion, not a balance mismatch. Under `Policy::CostAware` an inserted cold key can be evicted immediately at a full budget. | 15 | Isolate a resident-hit phase with known capacity; preserve the continuous-eviction phase. T00a. |

### 2.1 Concrete schedules

**Stats cycle:** S locks V in `read_stats`; A locks B(write) in `append`; A blocks on V; S blocks
on B(read). Reachable from the wire (`session.rs:899`). A test must latch those acquisitions,
observe the cycle without hanging the suite, and prove the repaired path completes.

**Report race:** initialise a full view at `a`; let `report_shape` finish its certification; append
a balanced transaction changing the value at `e>a`; resume `report_from_view`. The loop copies the
new value and stores `anchor: a`.

**Legacy wait cycle:** A begins a miss and owns ticket T, releases V. B locks V and calls
`Rev::read` at T's key/anchor, joining T. A needs V to finish T. B waits on A while holding V.

Rust's standard `RwLock` does not promise a particular scheduling policy; the deadlocks above do
not depend on it.

### 2.3 Theorem 4.1 and hazard-class audit

| Clause / former hazard | Verdict | Required guard |
|---|---|---|
| 5a served answer | Correct without generation under the fold precondition. | Independent prefix oracle for every returned answer, including a stale-generation completion. |
| 5b Pending admits no delta | Slot-kind handling prevents it; the generation test is unrelated. | Force pending across advance; require a value/certification witness where the mutation causes one. |
| 5c moved-frontier install | `install` pins `anchor<applied` independently of generation. | Unpin a stale fold then query a genuinely changed key at e: value divergence. |
| Generation ownership | Prevents superseded installs and removing a successor's entry. Resident regression is established; permanent stranded-waiter liveness is **not**. | Reversion must produce a stale overwrite/removal, not a fabricated wrong-value transcript. |
| "Between begin and finish M=Pending" | **False** for a pre-existing Present unable to answer this anchor: `prior=None` retains it. | Exercise retained Present plus an owned flight; state slot and flight metadata separately. |
| "Deferred merge is strictly better" | Can add more work than saved, evict a useful historical answer, or race beyond covered h. | Cost and reuse measurements; no unconditional benefit theorem. |
| Cancel restores exact prior | Restoration occurs at reap, not at owner Drop; untouched cancelled keys retain markers/capacity. | Immediate waiter release plus eventual bounded cleanup. |

### 2.4 Instruments, arithmetic and falsification

Pooled MAD is `sqrt((MAD_A² + MAD_B²)/2)`.

| Host / readers,writers | Baseline median; MAD | Candidate median; MAD | Ratio | Δ / pooled MAD | Verdict |
|---|---|---|---:|---:|---|
| C archive / 6,3 | 147305.6; 3709.0 | 151185.0; 2527.3 | 1.026336 | 3879.4 / 3173.636 = 1.222 | noise-limited |
| C archive / 9,5 | 156685.6; 1396.2 | 159498.9; 768.9 | 1.017955 | 2813.3 / 1127.072 = 2.496 | noise-limited |
| C archive / 12,6 | 160837.6; 1119.0 | 163753.0; 656.8 | 1.018126 | 2915.4 / 917.482 = 3.178 | fails ≥10% and ≥25% target |

The report's ~3118/~1083/~888 pools are arithmetic averages of the MADs. Correcting them changes
none of the overall decisions. The script runs A then B on every iteration rather than alternating,
leaving an order effect.

For 12r6w, archived per-run pinned/read percentages are 6.4913, 6.3303, 6.3313, 6.5888, 6.4891:
median **6.4891%**, not the reported ~6.35% (a ratio of medians). Median joins 14,377 gives
pinned/joins ≈ 21.695. Denominators include core read attempts/retries; these are not a measured
fraction of client RPCs that can be saved.

**Instrument falsifiability.** For each histogram, counter and table, the executor states what
result would falsify the hypothesis it was added for, and whether the harness can produce it. The
document-baseline sweep treats every prior cycle's timing target as a candidate until its raw
transcript is identified; T00's baseline registry enumerates each inherited target line with its
raw-data path or `not run`.

### 2.6 Partition conservation is two questions

Per-group zero sums prohibit **net imbalance** across groups. They do not prohibit every
cross-group flow: two opposite cross-book transfers in one transaction can cancel within each book.
Strict segregation needs an explicit admissible-participant/edge rule, or an atomic transfer
identity and partition constraint, with a defined allowance for FX/linked legs. Cross-line netting
needs a declared permitted netting set. LC-41 therefore contains a missing implementation of
declared grouping **and** a potentially new provenance/admission invariant. First refuse
unsupported declarations; the author chooses the richer policy before it is built.

## 3. Executable tasks, dependency order and cut

All tasks inherit §0 exact refs, §5 validation and §6 script contracts. All changes include their
meaningful guard in the same commit; the executor proves a reverted-change **runtime/assertion
failure** in a disposable worktree. A compile failure is an invalid mutation trial. The target
sentences prefixed **Target** are copied verbatim into §8's report checklist.

### T00 — Correct instruments and measure the current baseline

Closes A10-08/09/16/17; establishes the LC-23/24/38/39 baseline. Branch `c10/01-instruments`.
Files: `docs/audit/cycle-10/hostc/c10-baselock.sh` and its support code,
`crates/nilestream-server/src/{lockstats,rev_engine,session}.rs`, benchmark instrumentation,
baseline registry. The first commit repairs the existing harness; an instrumentation commit then
adds missing phases/counters, with neutral comparisons before it is used to score a runtime
change. Do not put a functional optimisation into the instrument control arm.

Because A10-01 exists, query diagnostic stats at quiescent boundaries in the baseline; separately
run the bounded deadlock witness. If the baseline cannot complete safely, mark it blocked and run
the correctness repair before scoring any performance task.

Record 6r3w, 9r5w, 12r6w at the historical 2,000-account/2,500-budget point **and** a genuinely
partial point with budget below distinct keys, labelled separately. Two warm-ups and ≥5 interleaved
measured arms, AB/BA order balanced. The first baseline stage has no performance pass line; neutral
A=A is the falsification control. Record `(a, applied_begin, applied_finish, readable_head,
visible)` gaps and actual keyed suffix rows before proposing merge thresholds.

**Target T00.1:** The current-build baseline script reports every requested phase and admissibility row at verified SHAs, rejects each harness fault injection, and produces a neutral A=A control before any performance task is scored.

**Target T00.2:** The wire and raw traces separately report shared/exclusive B waits and holds, deferred_merges including zero, both V waits, joined-read outcomes, refusal reasons and phase-separated anchor gaps with counters that reconcile to their defined events.

Guard: inject stale worktree/daemon, mismatched SHA, dirty worktree, failed warm-up, missing field,
port collision, timeout, bad flag, zero writer progress, and a deliberate second-V/join delay. Each
must fail or populate the intended column.

**The author must now run `bash ~/Documents/niles/docs/audit/cycle-10/hostc/c10-baselock.sh --baseline-only` and paste the output**, after its instrument commit is bundled and synced. Expected 25–40 minutes, cap 50.

### T00a — Resolve the newly observed gate failures before scoring speed

Above the cut; after T00's harness delivery and before T01–T04 performance scoring. N branch
`c10/01a-gate-evidence`; G branch `c10/00-adapter-guard` from 688919c.

**Target T00a.1:** The GBS tamper guard mutates verified record regions, distinguishes header and payload corruption, preserves all original bytes on refusal, and passes its clean-file negative control on both paired toolchains.

**Target T00a.2:** The current E18 allocation-budget violation is attributed with same-source, host/toolchain-labelled runs and is either repaired with a reverting cost guard or recorded as an explicit unresolved gate failure without raising the budget to hide it.

**Target T00a.3:** The GBS continuous-eviction guard preserves all value/conservation assertions and separately demonstrates an actual resident hit under an explicit retention precondition instead of assuming every new entry remains cached.

**Target T00a.4:** The paired adapter guard refuses missing GBS unless an explicit opt-out is reported as not-run, Appendix D includes legitimate API items after comment and intermediate-test traps, and Darwin preflight names and checks its actual barrier and data-volume evidence.

**Target T00a.5:** A named old-window-gap retry is accepted by both admission and sealer after expiry, an in-window duplicate is refused consistently, and no applied-but-undurable visibility or premature acknowledgement occurs under either retry schedule.

`NILES_NO_GBS=1` may support standalone development but cannot satisfy this paired audit gate. For
Appendix D, assert ZSet/eval_scalar plus a legitimate post-test item and intentionally excluded
helpers; a raw-string cutoff must fail. Darwin must use a supported `F_FULLFSYNC` interface, check
availability, identify the actual writable path/volume, and preserve failure status. Correct
`c10-rwlock.sh`'s argument refusal, actual toolchain, deadline and arrival-order labels.

The old state was **admission forgot the identity while the record-counted sealer retained it**;
the both-refuse target reverses the expiry remedy. Never manufacture a both-refuse outcome for an
expired identity.

**After T00a is synced, the author runs `bash ~/Documents/niles/docs/audit/cycle-10/hostc/c10-gates.sh` and `bash ~/Documents/niles/docs/audit/cycle-10/hostc/c10-rwlock.sh` and pastes their summaries.**

### T01 — Repair served certification and wait ownership

Closes A10-01/02/03/04/05/12. Branch `c10/02-read-safety`, after T00.

**Target T01.1:** Stats, append, full-report and mixed legacy/two-phase reads complete under their forced interleavings, and every returned row equals the independent fold at its stated visible anchor.

**Target T01.2:** Every logical keyed read acquires B at most once, every joined reader waits with no O/B/P/V/C/S guard, and a successful join consumes its own exact-anchor result without a second reconstruction.

**Target T01.3:** Cancelling owners or abandoning wait tickets releases bounded sharing capacity and counts every capacity refusal while preserving the exact prior absence and generation ownership.

**Target T01.4:** The Pending proof distinguishes served-value correctness, certification and generation ownership, and each reversion transcript claims only the hazard it actually witnesses.

Guard cleanups use a subprocess deadline and tracked owned children so an expected deadlock cannot
hang the suite. A joined value of zero is not evidence the requested account exists. This task does
not claim a V-speed improvement.

### T02 — Make public query and conservation claims truthful

Closes A10-06/07 and active A9-F15; narrows LC-41. Branch `c10/03-verified-contracts`, after T01.

**Target T02.1:** Runtime installation refuses every unsupported reachable graph in the guard corpus while all supported keyed balance circuits still agree with an independent fold after advance and eviction.

**Target T02.2:** Every accepted conserve declaration has the grouping semantics admission actually enforces, and unsupported or conflicting partition declarations are refused with a named diagnostic instead of being silently overwritten or ignored.

Use hand-built `Circuit` input as well as compiler-produced input, because the public boundary
cannot assume the compiler ran. Do not invent a segregation policy on the author's behalf.

### T04 — Bounded, certified deferred merge

Closes A10-11/F-10-05 and LC-38/39; after T00/T00a/T01/T02. Branch `c10/04-deferred-merge`.

**Target T04.1:** Every eligible generation-owned late landing within the preregistered epoch and physical-work caps merges exactly the keyed deltas in (a,e], every owner and same-anchor waiter still receives the answer at a, and unavailable or over-budget suffixes remain honestly pinned or uninstalled.

**Target T04.2:** Against the measured pinned-control figures from c10-merge.sh, deferred merging reports its gap, visited-row, reuse, writer and memory costs and makes a performance claim only when the preregistered ≥10% and ≥3 pooled-MAD gate passes.

Bound physical rows visited and allocations as well as epoch distance. Large gap, unavailable
suffix, failed ownership or stale coverage must produce an exact pinned/uninstalled result, never a
counterfeit current certificate. **The author must now run `bash ~/Documents/niles/docs/audit/cycle-10/hostc/c10-merge.sh` and paste the output** after syncing T04. If current gaps make merge
unattractive, record the negative result and keep the bounded pin policy; T04.1 is then
`not done: negative experiment`.

### T07 — Served checkpoints with honest scan bounds

Branch `c10/05-checkpoints`; after bounded merge, against its immediate control.

**Target T07.1:** The served daemon proposes default checkpoint interval 16 with explicit 0 ablation, a head-read guard of at most 17 entries for the single-currency fixture, and 0/16/64/256 measurements of physical account-index visits, target-currency visits and memory with exact provenance.

**Target T07.2:** Against the immediate merge-enabled control measured by c10-baselock.sh, the checkpoint candidate targets at least a 50% reduction in shared-hold p99, reports whether it meets the significance gate and its maximum/throughput, and records a missed target without substituting another baseline.

The at-most-17 guard uses a 1,024-posting single-currency head read; do not transfer it to the
mixed-currency index without proof. **The author must now run `bash ~/Documents/niles/docs/audit/cycle-10/hostc/c10-checkpoints.sh` and paste the output** after delivery.

### — Consolidated cut line —

The ceiling for one cycle is T00, T00a, T01, T02, T04 and T07. If correctness consumes the cycle,
leave T04/T07 not done; no compensating speed task.

### Below the cut

**T03** immutable-prefix design and unmerged prototype (LC-37); **T05** generated-document
integrity and the complete sentence/marker ledger (LC-40); **T06** costed checked-twice sweep;
**T08** production memory, honest holes and cache usefulness (LC-31/33); **T10** ordered
hash-handoff design only, no H landing (LC-42); **TG01** the thousand-share syndicated trace
(F-12/F-24, LC-34); **T09** contract republication preparation (LC-19/22/27).

**Target T03.1:** The snapshot design and unmerged prototype demonstrate exact reconstruction outside engine guards through append/reallocation with bounded one-B capture, account for all Base implementors and retained-handle memory, and present the public-trait choice for the author's LC-37 decision.

**Target T05.1:** Every generated document has an explicit inclusion policy and an omission mutant that fails, and the complete marker inventory distinguishes active, repaired, conditional, historical and template occurrences without deleting evidence.

**Target T06.1:** Every checker obligation family has a costed runtime counterpart or an explicit no-duplicate/unsupported row, and any removed recheck is replaced by a schema-bound verified fact while all data refusals remain intact.

**Target T08.1:** Production memory probes separate full-retention base, absence metadata, idempotency, plans and active snapshots, and any compaction or plan-key optimisation preserves exact absence and answers while failing its measured cost guard when reverted.

**Target T10.1:** The hash-handoff design either proves ordered parent-hash availability, byte-identical chains and durable-before-visible publication without waiting under B, or records the blocking invariant and declines the new H lock.

**Target TG01.1:** A thousand-share syndicated payment produces a deterministic conserved posting set with an explicit rounding remainder, and the same trace proves wire acknowledgement, durable restart, eviction and product-lifecycle replay without joining unrelated fixtures into one verdict.

**Target T09.1:** E16/E19 contract evidence is regenerated only on an admissible Host C with exact PostgreSQL and storage provenance, and every unsupported or unrun row remains explicit until the author separately authorizes publication.

## 4. Branch stacks, delivery and merge order

Use paired sibling worktrees; G's adapter path dependencies resolve into `../niles`. Record both
full SHAs for every adapter build.

```text
c10/01-instruments         T00
c10/01a-gate-evidence      T00a N
c10/02-read-safety         T01
c10/03-verified-contracts  T02
c10/04-deferred-merge      T04
c10/05-checkpoints         T07
---- consolidated cut ----
c10/07-snapshot-design  T03 note; prototype unmerged
c10/06-document-integrity T05
c10/07-checked-facts    T06
c10/09-production-memory T08
c10/11-hash-design      T10 note; no H landing
c10/10-contract-evidence T09
```

G starts at `688919c`:

```text
c10/00-adapter-guard     T00a tamper/hit repairs, above cut
---- below cut ----
c10/01-syndicate-trace   TG01 trace and joint witness
c10/02-lifecycle-replay  TG01 durable product transitions
```

At **each landing**, deliver the bundle under `~/Documents/niles-sync/cycle-10/` and print a
complete author block with the actual bundle and **the ref the bundle actually carries**, checked
with `git bundle list-heads` before the block is written. Never `git branch -f` the branch that is
checked out. Verify expected current branch and protected status before merging; verify full SHA
and remote refs after. A divergence is `BLOCKED-sync-divergence`, never a forced reset. Repeat
every owed bundle/command at the next landing until its sync is confirmed.

## 5. Validation and immutable constraints

`CARGO_NET_OFFLINE=true`, `RUSTUP_AUTO_INSTALL=0`, `GIT_TERMINAL_PROMPT=0`. Use 1.95.0 explicitly;
if a host only exposes `stable`, prove `rustc -vV` resolves to 1.95.0 or record blocked. No
network, installs, or new external dependencies.

Required matrix, both repositories at exact paired heads: `cargo fmt --all -- --check`;
`cargo clippy --offline --all-targets -- -D warnings`; `cargo test --offline --workspace`;
`make gate`; three `cargo test --offline --workspace --no-fail-fast -- --test-threads=8`;
`make reproduce`; `make fsync-proof`. Also run the G adapter with
`--manifest-path crates/gbs-nilestream/Cargo.toml --no-fail-fast` three times.

Green means exit 0 with the required targets and no hidden skip. Missing PG16/service/strace,
missing companion checkout, unresolved pin or missing Valgrind is `blocked` or `unsupported`.
Assertion failures and budget violations are **red results to investigate**, not environmental.
`not run`, `unsupported`, `blocked` and `noise-limited` are separate report values.

Timing protocol: two discarded warm-ups per arm, ≥5 measured replicates, interleaved in one session
with balanced AB/BA order. Report median, MAD, min/max, sample count, host, full SHA, toolchain and
actual row/transaction/reader/writer counts. Pooled MAD is `sqrt((dA²+dB²)/2)`; a benefit gate
requires **both** ≥10% relative improvement and absolute median difference ≥3 pooled MADs.
Preregister the primary outcome; do not search many cells and report only the winner.

Invariants carried intact: durable-before-visible and before acknowledgement; one sealer per
ledger; one total epoch order; apply-before-publish; full retention; honest absence — a Hole keeps
its version, cancellation restores the exact prior slot, Bottom asserts nothing and never means
zero; fold-never-field; product purity and GBS layering; self-describing amounts; no `unsafe`
outside the pre-existing measurement exception; no default-on-error; zero external dependencies; no
fabricated results; honest refusal instead of silent fallback. Currency wire code is the
declaration index; idempotency windows count **transactions**. LC-21 fail-stop and the settled
window/durability choices are not reopened.

Lock order is O < B < P < V < C; S is a leaf; flight completion F is a leaf below V. Every logical
keyed read takes B at most once, every join waits without an ancestor lock, flights and waiters are
bounded, every capacity refusal is counted, an install belongs to its generation, and
`deferred_merges` is reported even when zero. `Rev::read`/`begin_read` answer at the requested
anchor, without a compensating caller branch. **No performance result may compensate for a wrong
answer, lost wakeup or acknowledgement before durability.**

Every code/thesis divergence gets a `MISMATCH-<id>` at the claim until resolved; ambiguities get a
specific `BLOCKED-<id>`. Never remove a red test, weaken an oracle, expand normalised fields or
accept a new known divergence to make the gate green. Python edits to Rust use exact-string
single-occurrence replacement with the count asserted before writing.

## 6. Host C scripts: required implementation contracts

Opus writes these inside N at `docs/audit/cycle-10/hostc/`, versions them with their task, and
delivers them before requesting a run. The author always invokes `bash`. Each script has a
`--help`.

| Script | Required experiment | Expected / cap | Dependencies |
|---|---|---|---|
| `c10-baselock.sh` | unchanged-current baseline; instrument control; all mode/gap/falsification rows; A=A control | 25–40 / 50 min | T00 |
| `c10-merge.sh` | current pinned predecessor versus bounded merge; phase gaps, suffix visits, reuse | 20–35 / 45 min | T04 |
| `c10-checkpoints.sh` | 0/16/64/256 served intervals, actual physical visits and memory | 20–35 / 45 min | T07 |
| `c10-gates.sh` | paired gate rows, three complete repeats and memory/adapter continuations | 15–35 / 90 min; ≤12 min/command | isolated checkouts |
| `c10-rwlock.sh` | bounded arrival/acquisition-order probe with pinned compiler and explicit limits | 1–3 / 5 min | repaired in T00a |
| `c10-memory-cache.sh`, `c10-syndicate.sh`, `c10-contract.sh`, `c10-snapshot.sh` | below-cut experiments | per §3 | their tasks |

Common requirements: parse all arguments before doing work and **always reject `--publish`
explicitly**; resolve each candidate ref once with `rev-parse --verify <ref>^{commit}` and freeze
the SHA in a manifest; create unique paired detached worktrees outside the author's working copies
and refuse foreign or dirty ones; probe actual options against the release binary and distinguish
an unknown-flag parse failure from every other failed row; own the daemon per replicate on a unique
port with verified readiness; supervise with a Python standard-library process-group supervisor
(`start_new_session=True`, `wait(timeout=…)`, TERM then bounded KILL) because GNU `timeout` does not
exist on Darwin; write unique raw logs with one status row per planned section; verify sample count
and mandatory fields before computing statistics; record host, cores, load, the **actual segment
file's** device/mount, compiler versions and barrier name with its limitations; run the script's own
fault corpus, including a neutral arm and a deliberately delayed non-B phase so the instrument can
acquit B.

Durations are planning estimates, not measurements of these scripts. A hard cap must actually kill
and reap owned child process groups and exit nonzero.

## 7. Open-question ledger

Settled, not reopened: LC-01, 02, 04, 08–12, 14, 15, 17, 18, 21, 28. Decided in C9: LC-16, LC-35.

| LC | Disposition |
|---|---|
| 03, 05, 06, 07, 13 | carried; below cut or task-local |
| 19 | one chapter-6 sentence only after the joint product trace; open TG01/T09 |
| 20 | `BLOCKED-LC20-definition`: author supplies the subject or the number is retired |
| 22 | declared-scale wire transcript first; PG client/server versions distinguished; T09 |
| **23 / 24** | reopened B-tail attribution; T00 first. Correctness cycles are independently sourced. No renewed V-speed premise. |
| 25, 26, 27, 29 | carried per §7 of the source orders |
| 30 | implemented and guarded; explicit author confirmation still outstanding |
| 31 | Hole version is provenance, not a value; deferred cleanup and optional compaction are distinct. T01/T08 |
| 32 | checkpoint interval: proposed 16 awaits 0/16/64/256 evidence, T07 |
| 33 | plan-cache memory and usefulness remeasured; T06/T08 |
| 34 | thousand-share syndicated trace; below cut; F-12 and F-24 both required before efficiency |
| 36 | `BLOCKED-recovery-tip` remains the author's contract |
| **37** | owned immutable prefix including index/metadata, not a naked head. Public `Base` shape is the author's decision after T00 and T03 |
| **38** | gap-at-begin versus fold-induced gap, keyed suffix cost, bounded pin/merge decision; T00 then T04. No assumed 22× gain |
| **39** | pin rate may be a phase input only with a defined logical-read denominator and reuse/cost |
| **40** | generated inclusion policy and omission mutants; T05 |
| **41** | grouping syntax lacks admission semantics, and net-zero per partition is weaker than forbidden cross-partition flow. T02 honest refusal first |
| **42** | H is not approved. T10 first proves ordered parent-hash/visibility without waiting under B |

## 8. Executor report contract and checklist

Deliver `docs/audit/cycle-10/execution-report.md` with every §3 target sentence **verbatim**, one
row each with `done` or `not done: why`, plus evidence link, exact command/exit, host/SHA/toolchain
and guard reversion. A conditional negative performance outcome is a valid reported result where
the target explicitly permits it; it is not permission to mark an unimplemented value contract
done. Below-cut targets are listed as `not done: below cut`.

Report all baseline/candidate numbers beside a host column with derivation and raw-data location.
Keep A/S/D evidence classes and distinguish source-derived schedules from witnessed interleavings.
Include a reconciliation row for each Fable agreement/disagreement with **confirmed**,
**confirmed-narrowed**, **refuted** or **not independently verified**.

For each optimisation, list the disposable worktree's absolute path, parent/candidate/reversion
SHAs, the exact mutant diff, compiled command, test command and exit codes. No compile-error
witness. Attach the complete per-occurrence marker inventory and sentence ledger.

Report tracked worktree changes separately from the author's five untracked files, with their final
byte sizes (10,244 / 16,639 / 6,148 / 8,196 / 816,110 at audit) and any discrepancy referred to the
author, never corrected. Record both paired repo heads, every branch, every bundle/exported ref and
SHA-256, and all author sync statuses.

Include **exactly three material facts discovered during execution that this work order did not
cover**, with evidence class and consequence. Do not recycle A10-01…20 or F-10-01…13. If fewer than
three are found, report the shortfall rather than inventing facts.

## 9. Author-run Cycle 10 gate addendum

Full logs: `~/Documents/niles-sync/cycle10-gates-xojp4gqs/`. Isolated paired detached worktrees at
N 8548189 / G 688919c. `make gate` used 1.97.1; three workspace repeats and reproduce used 1.95.0.
This is **A** evidence.

| Command, Host C | N result | G result |
|---|---|---|
| `make gate` | exit 2 at `numeric_binary_oracle` | exit 2 at the adapter tamper test |
| fmt + full Makefile lint, 1.97.1 | passed before the failing test | passed, including adapter/memprobe lint |
| workspace tests, 1.95.0, runs 1/2/3 | **1,036 pass, 1 fail, 7 ignored** each; exit 101 | **504 pass, 0 fail, 1 ignored** each; exit 0 |
| failed N test | `our_numeric_bytes_are_postgresqls_numeric_bytes`, all three — missing PostgreSQL service | — |
| failed G adapter test in gate | — | `a_tampered_record_is_refused_and_the_segment_is_not_truncated`: 24 pass, 1 fail |
| `make reproduce` | exit 2: `rev_metadata_2x_budget` OVER BUDGET | exit 0, clean diff |
| `make fsync-proof` | exit 2, strace absent | exit 2, strace absent |
| G adapter `--no-fail-fast`, 1.95.0, ×3 | — | **48 passed, 2 failed**, exit 101 each |
| N `make memory`, 1.95.0 and 1.97.1 | exit 2; 44,165 / 5,000 = **8.833 alloc/key**, budget **7.5**, both toolchains | — |
| G `make memory`, 1.97.1 | — | exit 0 |

N isolated `reproduce` changed only `results/E18-memory.csv`, `results/E18-memory.md`,
`results/wire-protocol-session.md`; the wire difference is psql 18.6 Homebrew versus committed
16.13 Ubuntu. Fable reports N's memory gate green on Linux at source-equivalent code: a
**host-shaped discrepancy to attribute**, not permission to discard either result.

Final read-only verification: N `96b6d9e4d962bb35e64a13d1345d821ae271cb36` with only the five
protected untracked files at **10,244 / 16,639 / 6,148 / 8,196 / 816,110 bytes**, unchanged; G
`688919c87852c86d8e8c3abc0af2135db60a5197`, clean.
