# Cycle 10 audit brief — **Claude Fable 5.1**

**You are auditing, not building.** Your single deliverable is a **work order**: a prioritised,
evidence-backed set of tasks that Claude Opus will execute in a later session. You write no
production code into either repository. Read everything, run the suites, run the compiler over the
corpus, run the gates, write throwaway probes outside the trees, and **keep testing in the cloud
container** — it is the one place in this project where a deterministic counter can be produced
without asking anyone. Commits are yours only for the work order itself, on a `c10/00-audit`
branch, bundled to the author as §10 describes.

The work order you produce is the only thing the executing agent sees. Assume it has none of your
context: every task carries its own preconditions, baseline, target, method, acceptance test, guard,
and guardrails, and must be executable **offline**.

**A second auditor — Astra — is reading the same two repositories independently, from a
companion brief.** Astra has no shell: it reads GitHub and the author's Mac through the desktop
bridge, and what the author pastes, and it asks the author to run things. Its brief leans on
claims-versus-code, the type system, the thesis text, and GBS's product model against Loan IQ and
Calypso. Yours leans empirical: you *measure* what Astra can only read, and you re-derive every
number the executor reported in cycle 9 before you build on it. The author reconciles the two work
orders afterwards, exactly as in cycle 9 (`docs/audit/cycle-9/work-order-9.md` §2A is what that
reconciliation looks like: every Astra finding carries a Fable verdict — *confirmed*,
*confirmed-narrowed*, *not re-read* — and the four disagreements are argued out by name). Three
consequences:

1. **Where you both look at the same thing and disagree, the disagreement is the valuable part.**
   State your evidence class on every finding. Never round toward the other auditor.
2. **Do not soften a finding because you expect Astra to catch it.**
3. **§8 lists things you must both check independently.** Do those even knowing they are duplicated.

**The prior cycle's work order was wrong about its headline task, and the executor measured that
it was wrong.** Cycle 9's C9-06 was built to recover a read-throughput collapse (94,038 → 20,507
reads/s across 6 → 12 connections) attributed to the view lock. The corrected measurement found
no collapse — the same build rises 147,306 → 160,838 — and found the tail is the **base** lock.
The change was kept on other grounds and the claim withdrawn. Read
`docs/audit/cycle-9/execution-report.md` §11 and `docs/audit/cycle-9/hostc/c9-pending-results.md`
before anything else, and treat every figure in `docs/audit/cycle-8/lc-23-attribution.md` as
**refuted unless re-measured**. Then treat everything in `docs/audit/cycle-9/` — including the
work order you and Astra wrote — the same way: claims to verify from the source and by measurement.

**Instruments first, and now instruments *second* as well.** Before you audit the code, run
`docs/audit/cycle-10/preflight.sh`. Then, for each instrument you intend to use, answer two
questions: *would it have caught the defect it is about to look for?* and — the question cycle 9
added — *could it have produced the finding it is about to refute?* The view-lock histogram was
added in cycle 8 to convict the view lock; run correctly, it exonerated it. An instrument that can
only confirm is not an instrument.

---

## 1. The three goals, and the question each auditor answers

The stated end goals are **a highly efficient query language (Niles)**, **a highly efficient query
engine (Nilestream)**, and **a highly efficient core-banking system (GBS)** — efficient in **memory
and speed**, poised to replace systems of the class of **Finastra Loan IQ** (commercial and
syndicated lending) and **Nasdaq Calypso** (capital markets: trading, risk, collateral, treasury,
clearing, post-trade) — without surrendering any commitment in §10. The thesis's claim is a theory;
the two systems are its instrument; the banking constructs are a domain layer on a fully general
relational core.

Cycle 9 answered the three framing questions and each answer moved the question:

- **Nilestream.** The repair the thesis describes for the fourth lattice state is built, guarded by
  a latch-driven differential, and reached over the wire on the reference host (median 14,377
  `pending_joins` at 12r/6w). It is worth **+1.8%**, not the predicted ≥ 25%, because the collapse
  it was aimed at was an artefact of an uncorrected harness. In the slowest-16 tables of *both*
  arms, at every level, `base wait` is the dominant term — one replicate prints 1098/1097,
  1093/1093, 1089/1089 µs total/base. `answer_from_view` holds the base **read** guard across the
  reconstruction, deliberately, because `RwLock` is not reentrant and a keyed read must take the
  base exactly once; a writer arriving behind that guard queues, and every later reader queues
  behind the writer. **Question one: what is the design that lets a reconstruction run over an
  immutable prefix without holding the base lock at all — and can the lock order O < B < P < V < C
  survive it?** The base is append-only; a fold over `[0, anchor]` needs a stable `head`, not a
  guard. That is a design question about the storage trait, not a tuning question, and §6.1 frames
  it. Beside it, a number the same measurement produced: **6.35% of reads land pinned against
  0.29% joined** — twenty-two reconstructions serve one anchor and are thrown away for every one
  that is shared. The deferred-delta merge was scoped as "a second commit" and is worth more than
  the first (§6.2).
- **Niles.** Cycle 9 closed one instance of "checked twice": the currency table was re-parsed on
  every `INSERT` (61% of the insert's instructions) and is now compiled once per schema epoch, with
  `schema_parses` as a wire column. That was one fact the compiler proves and the engine
  re-established. **Question two, unchanged and now with a method: where else does the engine
  re-establish a fact the type system already proved?** Conservation (`conserve per`), linearity
  (Q_lin), single currency on the read path, the idempotency key's presence, the window's unit.
  Cycle 9 gave you the instrument — per-function callgrind attribution on the `checked-twice`
  probe, stable where whole-process totals are not — and swept one function with it. Sweep the
  rest.
- **GBS.** Still inadmissible for an efficiency claim, and nothing in cycle 9 moved it: F-24 (no
  product trace the engine benchmark can be shaped by) and F-12 (lifecycles that do not survive a
  restart, `BLOCKED-T-09-lifecycle`) are both open; C9-G01, the 1,000-share syndicated payment
  trace, was below the cut and was not taken; six of twenty-nine rows are `GenericPathOnly` and
  one (Risk analytics) is `Blocked — this row posts nothing`. Two cross-repo commits landed
  (`447544d`, `688919c`) and — **check this first** — as of the end of cycle 9 they had been
  bundled to the Mac but **not fetched into `~/Documents/GBS`**, which was still at `963e4d9`.
  **Question three: which of Loan IQ's and Calypso's shapes is a new invariant for the ledger
  rather than a feature, and which one closes F-24 or F-12 while it is at it?** §6.5 carries fresh
  vendor material and the crosswalk cycle 9 built; extend it, do not restart it.

---

## 2. Access

### 2.1 GitHub

Both repositories are private, owned by **`shbiebs`**:

- `https://github.com/shbiebs/niles`
- `https://github.com/shbiebs/gbs`

**Try before you ask.** From the container: `git ls-remote https://github.com/shbiebs/niles`
says in one line whether you can read it. If you cannot, ask the author for the narrowest thing
that works: **read access only**, to those two repositories and nothing else. On GitHub that is a
fine-grained personal access token with *Only select repositories* → `niles`, `gbs`, and
*Contents: Read-only*; the author supplies it out of band, you use it once to clone, you never
write it into an output, and you never push. If access is refused or the repository reads as not
found, say so in your work order and audit what you can reach — the bundles in
`~/Documents/niles-sync/cycle-9/` and `cycle-10/` on the Mac are complete histories and the author
can attach them. **Never let a `git` command in the container reach for the network without
`GIT_TERMINAL_PROMPT=0`**: a prompt for credentials hangs a session that cannot answer it.

The heads to audit are in §4. Confirm them before reading anything else, and confirm that the two
branches on each remote (`c7/01-durable-rows` and `c9/07-pending` on niles; `c7/00-adapter` on
gbs) are at those heads — cycle 9 ended with one of them not.

### 2.2 The author's Mac

You are authorised to audit the working copies at `/Users/checolino/Documents/niles` and
`/Users/checolino/Documents/GBS`. Both are at the heads in §4 (after the author runs the §4 sync)
and clean apart from five untracked files that are the author's and must never be edited, staged,
deleted, moved or bundled: `.DS_Store`, `AGENTS.md`, `niles/.DS_Store`, `thesis/.DS_Store`,
`thesis/Niles-Thesis.pdf` — **10,244 / 16,639 / 6,148 / 8,196 / 816,110 bytes** at the end of
cycle 9, unchanged through the whole cycle. Report their sizes at the end; a discrepancy is
referred to the author, never corrected. **Read-only discipline**: no commits, no staging, no
`git clean`, no checkout of another branch without saying so first.

Everything that needs the real machine — a build, a benchmark, a storage probe, `psql` — is a
shell script the author runs in Terminal.app. Cycle 9's convention is the one to keep: scripts live
**in the tree** at `docs/audit/cycle-<n>/hostc/<name>.sh` (so they are versioned and land with the
bundle), and the author runs them with `bash`. `docs/audit/cycle-9/hostc/c9-pending.sh` is the
template: it builds each arm in its own detached worktree at an explicit SHA and refuses if the
retarget did not take; interleaves arms inside one session; discards warm-ups; runs one daemon per
replicate by letting `bench --host-nls` own it, so no replicate can measure a stale server on the
port; prints a status for every section; exits non-zero if any section did not complete; **refuses
`--publish`** rather than ignoring it; and names its candidate by *branch ref*, because a script
that writes its own SHA into itself changes that SHA. Say **explicitly, at the point in your work
order where it is needed**: "the author must now run `bash
~/Documents/niles/docs/audit/cycle-10/hostc/<name>.sh` and paste the output." Every script is
idempotent, non-interactive, bounded in time with the runtime stated, and never assumes `chmod +x`.

The author's zsh has `interactive_comments` **off**: a `#` at the prompt is a command. No comments
in pasted lines. `--ff-only` moves the branch that is checked out, so a `c10/*` name has to be
created with `git branch -f` at the merged commit before it can be pushed — every sync block in
§12 already says so. The author has run every cycle-9 sync block correctly on the first attempt;
the one that was *not* run is the GBS one, because it was given once and not repeated at a
landing. **Repeat an owed sync block at every subsequent landing until it is run.**

**Bridge traps, if you reach the Mac through the Claude desktop bridge.** You get a Linux VM over
FUSE, not macOS: no `rustc`, `cargo`, `psql` or `valgrind`; fine for reading, useless for
measuring. Any `git` command that touches the index leaves a `.git/index.lock` the bridge cannot
delete — use `git --no-optional-locks` for reads, and if one appears, tell the author to `rm
~/Documents/niles/.git/index.lock`. `git -C` on a worktree fails through the bridge because
worktree metadata records `/Users/...` paths the mount does not have. Files are delivered to the
Mac by writing them to `~/Documents/niles-sync/cycle-10/` through the bridge; a bundle written
there and a fetch pasted for the author is the whole delivery mechanism.

The real Mac is **Host C, the reference host**: Apple M4, 10 cores (4P + 6E), Darwin 25.6.0, APFS
on NVMe, every barrier `F_FULLFSYNC` at ~255/s, Rust 1.95.0 pinned with 1.97.1 also installed,
PostgreSQL 16 installed **but not running on 127.0.0.1:5433** — which still blocks the E16/E19
republish (C9-12). The only machine in this project that gives the same answer twice: in cycle 9
it gave the same read-throughput medians to within 2% across five interleaved replicates.

### 2.3 The container (yours alone)

A 2-core shared Xeon with real `fdatasync`. **PostgreSQL 16 is installed but not started**:
`sudo -n service postgresql start` first, or `make gate` is red on `numeric_binary_oracle` for an
environment reason — cycle 9 lost a diagnosis to exactly this, and the cycle-9 preflight table
recorded the server as present. `RUSTUP_TOOLCHAIN=stable` (stable *is* 1.95.0; the pin `1.95.0`
does not resolve), `RUSTUP_AUTO_INSTALL=0`, never let rustup attempt a download.

---

## 3. Hosts, and what each may conclude

| host | may conclude | may not |
|---|---|---|
| **C** (the Mac) | absolute wall-clock, durability, scaling to 10 cores, contract verdicts, anything published | — |
| **a cloud container** (2 cores, shared Xeon, real `fdatasync`) | deterministic counters (allocations, instructions **per function**, visited entries, `max_batch`, txns/fsync ratios, compiler verdicts, flight counters), within-container ratios, tests, guards, mutation runs | any absolute figure beside another host's; any durability row; any curve past 2 cores; anything published |
| **the desktop bridge** (a Linux VM over FUSE, no toolchain) | reading, grepping, counting, delivering files | building, measuring, anything with `git` that writes an index |

Rules the table decides: no absolute wall-clock, throughput or fsync figure beside another host's
without a host column; deterministic counters gate first and need one run; wall clock needs 2
warm-ups and ≥ 5 measured runs, arms interleaved, median/MAD/range, and a gate fires only on ≥ 10%
*and* ≥ 3× the pooled MAD, else "noise-limited"; `unsupported`, `blocked`, `not run` and
`noise-limited` are results and omission is not. **A pass line is written against a *measured*
baseline or it is not written**: C9-06.2's was written against a document, and the document was
wrong by 7.8×. If the baseline you need does not exist yet, the first task in your order is the
script that measures it, and every pass line downstream says "against the figure that script
produces".

---

## 4. Exactly where the trees stand

| repository | head | branch(es) | tests |
|---|---|---|---|
| `niles` | **`53b40f6`** — "C9: the execution report quotes its target lines and adds the load runs" | `c7/01-durable-rows` = `c9/07-pending`, both on GitHub once the author runs the block below | 916 `#[test]` attributes; **1,037 pass, 0 fail, 7 ignored** (`--no-fail-fast`, container) |
| `gbs` | **`688919c`** — "C9-04 (gbs): the idempotency window is declared on the column" | `c7/00-adapter`, on GitHub once the author runs the block below | 625 attributes |

Cycle 9's commits on `niles`, oldest first: `638c7bb` (the consolidated work order) · `366c338`
(C9-00) · `f7cdfdd` (C9-01) · `35484ca`, `c224cbd` (C9-02) · `ca8e5c9` (C9-03) · `6c755fb`
(C9-04) · `ebe7f8f` (C9-05) · `78acd64`, `4ac5032`, `a1dbca6` (C9-06 and its measurement) ·
`dc19a4a`, `53b40f6` (the execution report). The cycle-10 briefs and preflight are on `c10/00-audit`
above `53b40f6`. On `gbs`: `447544d` (C9-02) · `688919c` (C9-04).

**Two syncs may still be owed when you start.** Confirm on the Mac and on GitHub before reading
code; if either is behind, put this at the top of your work order and tell the author to run it
now:

```
cd ~/Documents/niles
git fetch ~/Documents/niles-sync/cycle-9/niles-07-pending.bundle c9/07-pending
git merge --ff-only FETCH_HEAD
git branch -f c9/07-pending HEAD
git push origin c9/07-pending c7/01-durable-rows
cd ~/Documents/GBS
git fetch ~/Documents/niles-sync/cycle-9/gbs-01-batch-seq.bundle c7/00-adapter
git merge --ff-only FETCH_HEAD
git fetch ~/Documents/niles-sync/cycle-9/gbs-02-idem-key-only.bundle c7/00-adapter
git merge --ff-only FETCH_HEAD
git branch -f c7/00-adapter HEAD
git push origin c7/00-adapter
RUSTUP_AUTO_INSTALL=0 cargo +1.97.1 clippy --offline --all-targets -- -D warnings
```

Gate, both trees, at these heads: `cargo fmt --all -- --check`, `cargo clippy --offline
--all-targets -- -D warnings` (1.95.0 in the container, 1.97.1 on the Mac), `cargo test --offline
--workspace`, `make reproduce` (exit 0 after commit; its final `git diff --exit-code` is red only
while regenerated files are uncommitted), `make fsync-proof` — all green. A red row is a result;
never fix it by deletion.

---

## 5. What cycle 9 changed, and what it found that it was not asked about

Read `docs/audit/cycle-9/execution-report.md` in full. The short form:

**Landed (C9-00 … C9-06, all above the cut).** The gate's fixture races and load-dependent
assertions; four lock-histogram scopes with `reset`, quantiles that never overstate, the "0–2 µs"
column withdrawn; durability receipts owned by the session that earned them (`Appended { epoch,
receipt }`) and one idempotency window counted in transactions with the record's number named
`batch_seq`; a self-checking record header (`len_check = !len ^ 0xA5A5_A5A5`) with a three-way
recovery classification and a 10-record/8,840-flip torn corpus (7,984 refused / 856
prefix-with-success, all inside the final record); currency codes as declaration indices, refusing
daemon flags, one declared window, NL0218/NL0219; the schema compiled once per epoch; and the
two-phase read (`begin_read` / fold with nothing held / `finish_fold`) with equal-anchor joins,
generation-owned installs, pinned installs below `applied`, cancellation that restores the exact
absence it replaced, bounded flights, and four wire counters.

**Measured, and refuted its own premise (C9-06.2).** §11 of the report. The two-phase read is
worth +1.8% at 12r/6w; the collapse it was built to recover does not exist on the corrected
harness; the tail is the base lock; LC-23 is reopened against the base. The change is kept for
completing the lattice and bounding the hold, and every sentence that claimed speed was rewritten.

**Three material facts the work order did not cover.** (1) `thesis/gen-appendix-d.py` builds the
public-API appendix by cutting each file at its **first `#[cfg(test)]`**, so a test helper placed
mid-`impl` silently deleted `Unsupported` and `Runtime` from the generated appendix, and `make
reproduce` surfaced the loss as a diff to accept. Worked around in `rev.rs`; the generator is
unrepaired. (2) `make gate` is red in a fresh container because PostgreSQL is not started, and the
preflight table said it was present. (3) The generation check in `finish_fold` **does not prevent a
wrong answer** — no value divergence is constructible, because the certification interval already
defeats a stale stamp — so the work order overstated its hazard class by a rung; what it prevents
is a resident stamp moving backwards and an orphaned `Pending` marker. All three are candidates
for your §9, and none of them is a task yet.

**Two smaller things worth carrying.** `crates/niles-interp/src/lib.rs:401` still names
`ignored_windows` in a doc comment though the field is gone (C9-04.4's "no longer exists" is true
of the code and false of the prose). And `MISMATCH-A9-F05` still stands at `SPEC-ENGINE.md:599` although C9-04.1 repaired it — a stale marker that C9-10 would have struck.

**Where `docs/audit/cycle-9/` was itself wrong**, so you do not build on it: the C9-06.2 pass
line's baseline (a document, 7.8× low); the reversion-3 hazard class (fact 3 above); the preflight's
PostgreSQL row (fact 2); and C9-02.4's target line, which asked for a witness of the
applied-undurable state *or* a "not reproduced" record with the invariant named — the executor
produced neither as its own artefact, reporting instead that C9-02.3 removes the gap by
construction. That is probably true and it is not what the line asked for; decide whether the
witness is still worth having.

---

## 6. Where your budget should go

Ordered by expected value. §6.1 and §6.2 are the cycle; §6.3 is the carried backlog in the order
cycle 9 left it; §6.4 and §6.5 are the language and GBS questions.

### 6.1 The base-lock tail, and a reconstruction that holds nothing (LC-23 reopened, LC-24)

The finding: `answer_from_view` (`crates/nilestream-server/src/rev_engine.rs`) takes `self.base()`
— a `TimedRead` guard over `RwLock<Ledger>` — for its entire body and folds the base under it. The
two-phase read released **V** across the fold and left **B** held, because the source guard
`a_keyed_read_takes_the_base_exactly_once` forbids a second `self.base()` and `RwLock` is not
reentrant. A read guard blocks no other reader; it blocks the appender, and once the appender is
queued, every reader that arrives after it queues too. That is the shape of every slowest-16
table on both arms of C9-06.2.

**The question is not "make the fold faster". It is: what does a reconstruction over an
immutable, append-only prefix actually need from the base?** It needs a stable `head` at or above
its anchor and read access to rows `[0, anchor]`, none of which can change. Candidates, for you to
measure and for Astra to reason about:

- **A snapshot handle.** `Base::reconstruct` receives a snapshot that pins `head` and reads rows
  without a guard — sound only if the row store is truly immutable behind the guard (segment bytes
  are; the in-memory `Vec<Posting>` may reallocate on push). Establish which.
- **Guard only the frontier read.** Take B to read `head` and drop it; fold with no lock, indexing
  into storage that never moves. The lock order gains nothing and loses a hold.
- **`RwLock` fairness.** Whether the base lock is writer-preferring on the platforms that matter
  decides whether a long read hold *actually* queues later readers. Measure it, do not assume it:
  the slowest-16 tables say readers waited on B, and the mechanism should be shown, not inferred.

**Measure before designing.** The cycle-9 harness now resets histograms per level and has
`ENGINE_LOCK` waits: the first Host C script of the cycle should print, per level, the base-wait
histogram beside the view-wait histogram on the *current* build, so the design has a number to be
measured against. Its pass line is written against that number and no other. Then the guard: a
differential in which a writer appends continuously while readers reconstruct, asserting the
certification interval and the lock order — the existing `every_answer_matches_an_independent_fold_at_its_own_anchor`
extended so that `reconstruct` takes no guard — and a source guard that `answer_from_view` holds
B for a bounded section. The reversion that must make it fail: put the fold back under B.

**What it must not give up:** the base exactly once (or zero times) per keyed read; the order
O < B < P < V < C with F a leaf; a read that is exact at its own anchor; no compensating branch.

### 6.2 The deferred-delta merge (22 : 1)

`pinned_installs` is 6.35% of reads and `pending_joins` 0.29% at 12r/6w. A pinned entry serves the
one anchor it was asked for and is rebuilt by the next reader that needs the key fresh; twenty-two
reconstructions are thrown away for every one that is shared. The sound alternative, stated in
Chapter 3's upquery rule and Theorem 4.1 clause 5c and counted by a `deferred_merges` field that
reads zero: when a flight anchored at `a` lands after the view has applied through `e > a`, fold
the deltas for that key in `(a, e]` into the landing value and install at `e`, unpinned. Linearity
(H-F2, Q_lin) is what makes that sound, and it is the same step Theorem 4.1 already relies on.

What to measure first, in the container, deterministically: the *distribution* of `e − a` at
landing under the mixed workload. If most flights land one epoch behind, the merge is a single
delta lookup and the win is the whole 6.35%; if they land hundreds behind, the merge is a fold and
the question is where its cost crosses a fresh reconstruction's. The differential guard is the
latched one from C9-06 with a fourth reversion: install at `e` without folding `(a, e]` → the
divergence C9-06's reversion 1 already produces.

### 6.3 The carried backlog, in cycle 9's order

C9-07 through C9-12 were below the cut and none was taken. Re-derive each one's baseline before
carrying its target; the specifics that were true at `638c7bb` may not be at `53b40f6`:

- **C9-07 — checkpoints in the served daemon** (F-64, A9-F12, LC-32). `nilestreamd` has no
  `--checkpoint-interval` flag; the C9-06.2 arms proved it by refusal. The design and the E18 row
  are in the cycle-9 work order; Astra's correction — the physical scan index is per account while
  checkpoints are per (account, currency) — stands.
- **C9-08 — holes compact to ⊥, and `Runtime::install` walks the graph** (F-66, A9-F11, A9-F15,
  LC-31). `MISMATCH-A9-F15` still stands at Theorem 4.1's open case: a `sum` over a join installs.
- **C9-09 — E18 on the production structures** (F-65, A9-F09, F-77).
- **C9-10 — the sentence ledger and the MISMATCH inventory** (F-73, F-74, F-78). Seventeen `MISMATCH` families
  live at `53b40f6`; the execution report §9 lists them with owners; one is stale.
- **C9-11 — the templated plan key**; **C9-12 — the C republish** (blocked on PostgreSQL 16 on
  5433, still).

Whether any of them outranks §6.1 and §6.2 is yours to argue with EV arithmetic; cycle 9's judgement
that durability and determinism outrank speed still holds, and none of the six is a durability task.

### 6.4 Niles: the checked-twice sweep, continued

C9-05 measured one fact re-established at runtime (61% of an insert) with per-function callgrind
on `checked-twice oltp` and removed it. The instrument exists (`make checked-twice`; four shapes:
`seed`, `oltp`, `point`, `point-cold`). Sweep the remaining facts the type system proves and the
engine may re-derive: the conservation obligation (`conserve per`) at `append`; `Agg::Sum |
Agg::Count` linearity at `Runtime::install` — which is also where A9-F15's missing graph walk
lives; single currency on the served read path (`currency_count() != 1` is checked per read); the
`idem` column's presence per statement; the window's unit. For each: the instruction share on the
probe, whether the check is a type fact or a data fact (a data fact — an undeclared currency on the
wire — must stay), and what the engine would trust instead. Cycle 9's rule stands: **the language
pays twice only where the runtime cannot trust the compiler, and every such place is either a
refusal on data or a task.**

### 6.5 GBS: Loan IQ and Calypso, as invariants rather than features

Research done for this brief (vendor material, September 2026; sources in §13) confirms and
extends cycle 9's list. **Loan IQ** (Finastra) describes itself as servicing "70% of the world's
syndicated loans", "from a few to over a thousand lenders in any given deal", across "SME,
commercial real estate, letter of credit, complex bilateral, agency, syndication, ABL, export
finance"; a "PIK module"; "club deal, non-pro rata and unitranche"; "complex pricing structures"
with a "tiered matrix based on … dates, balances, ratings"; "interest and fee option types";
"advanced collateral management" with "asset registration, unit details, rent rolls and invoices",
"cross-collateralization", "covenant monitoring / document tracking", "violation / warnings /
control"; "rich trading functionality" (the secondary market) with "real-time view of all
back-office transactions"; "multi-branch, multi-business line, general ledger/sub-ledger with
configurable account mapping"; "online accounting with real-time debits and credits"; "extensive
audit trail"; "full multi-currency and multi-branch"; "adjustments, amendments and reversals";
ESG and sustainability-linked lending; and an integration layer ("Loan IQ Nexus", "API's / SDK").
**Calypso** (Nasdaq): multi-asset trade capture over "shared market data, pricing curves, and
valuation models" — rates, FX, credit, equity, fixed income, repo and securities finance,
commodities, treasury instruments; "a single trade record that is shared across the enterprise";
risk — "market risk, counterparty credit exposure, liquidity positions, and trading limits", P&L,
XVA; collateral and margin — "margin calculation across ETD, OTC cleared, and OTC uncleared",
VM/IA/Grid, "certified ISDA SIMM", "AANA calculation, group threshold monitoring, back testing",
"exposure netting across multiple business lines", "optimal allocation and minimal deployment of
cash collateral", "supports any agreement type", triparty and Acadia connectivity, "intraday and
end-of-day" reconciliation; securities finance "full support for stock and cash corporate actions"
and SFTR reporting; clearing — "more than 60 exchanges and CCPs", methodologies "SPAN2, PRISMA,
IRM2, VaR-based margin, liquidity charges, and margin buffers", account structures "ISA, OSA,
NOSA, GOSA, and multi-branch", "exchange position limit monitoring", "24/7 clearing with
follow-the-sun", "cross-margining"; post-trade STP across "confirmation, clearing, settlement,
accounting, corporate actions, and lifecycle event management"; treasury — "funding & liquidity
management", "consolidated asset inventory", visibility across "banking, trading, and investment
books", "liquidity planning and stress testing", Basel-aligned submissions; regulatory reporting
under Dodd-Frank, EMIR, MiFID II, FRTB, SA-CCR, SFTR; and — new this year — "tokenized collateral
management across mainstream and digital asset markets".

**The admissibility rule is unchanged.** GBS's product rows exist to exercise the ledger's
invariants under real product shapes, not to reach feature parity. A product task is admissible in
cycle 10 only if it does at least one of: (i) exercises a conservation or certification invariant
in a way no existing row does; (ii) closes F-24 or F-12; (iii) moves a row from `GenericPathOnly`
to `ProductSpecific` with G3 evidence. Start from cycle 9's crosswalk (`work-order-9.md`, F-79,
LC-34 — the table with **P / G / A** and the invariant each absent shape would stress) and
**extend it with the items above that it does not have**, classified the same way. The ones this
brief expects to matter most, with cycle 9's positions where they exist:

- **Pro-rata distribution to a thousand lender shares** — C9-G01, proposed and not taken. Still
  the first product task: one payment, a thousand postings, one conservation obligation with a
  rounding remainder (`ParticipantSumMismatch` in `gbs-products/src/lending.rs`), the largest single-epoch batch
  GBS would seal, and a direct test of the batch envelope, the sealer's drain and E23's
  cost-per-row. It is (i) and (ii) at once.
- **Adjustments, amendments, reversals** — cycle 9 found neither `reverse(txn)` nor
  `amend(txn, valid: ..)` exists as a spelling, though `valid:` exists on `Posting`. A language
  task, admissible as bitemporal audit; decide whether it outranks C9-G01.
- **PIK capitalisation** — the ledger writing to itself on a schedule; Niles has no `at epoch` /
  `every` form and the temporal library is read-side. Blocked behind F-12 and it stays there.
- **Letters of credit** — provisionally answered by G3 (a hold with expiry; 55 reads, 6 wipes);
  an *amendment* of an undrawn LC is the bitemporal gap above.
- **Collateral: cross-collateralisation, covenants, "violation / warnings / control"** — a
  constraint over holds *across* accounts, which is `conserve per (…)`-shaped and which the
  language may not be able to state. Loan IQ's "rent rolls and invoices" are data, not invariants.
- **Margin and collateral (Calypso)** — a variation-margin call is a hold whose amount is a
  function of a price: a REV over a **non-ledger base**. First place the "fully general relational
  core" is tested against market data. ISDA SIMM, AANA and "group threshold monitoring" are all
  *derived views with a threshold* — the same shape.
- **CCP account structures (ISA / OSA / NOSA / GOSA) and "exposure netting across business
  lines"** — a partition of the key space with cross-partition conservation forbidden by rule.
  Netting *across* business lines is the same rule with the sign reversed. Does `conserve per`
  admit a partition, and does it admit a cross-partition *prohibition*?
- **Multi-book treasury (banking / trading / investment) and multi-branch GL mapping** — GBS's FX
  row exists; a *book* dimension does not, and Loan IQ's "configurable account mapping" is a view
  from postings to GL accounts — a derived relation the thesis's core should express for free.
- **The "single trade record" / "unified trade record"** — in Calypso this is a product claim; in
  this project it is the ledger, and the question is only whether every downstream artefact GBS
  produces (positions, P&L, margin, reports) is provably a view over it. Reports under EMIR /
  SFTR / SA-CCR are views with a *schedule*, and a scheduled view is PIK's problem again.
- **Tokenized collateral** — a settlement leg outside the ledger; out of scope unless someone shows
  it stresses an invariant.

Do not propose more than one product task above the cut line unless it is also (ii). The refusal
"no GBS efficiency task is admissible until F-24 and F-12 close" stands until an auditor shows
otherwise with evidence. And check first — because cycle 9 could not — **whether the GBS tree the
author has is the one you are reading.**

### 6.6 What only you can do this cycle: measure in the container

Astra cannot run anything. You can, and these are deterministic, offline, and one run each:

- **The `e − a` landing distribution** (§6.2): instrument `finish_fold` in a throwaway probe to
  histogram `applied − anchor` at every pinned install under the in-process mixed workload. That
  number decides whether the deferred merge is a lookup or a fold.
- **`RwLock` fairness on the base** (§6.1): a probe with one writer and N readers on
  `std::sync::RwLock<Vec<_>>`, reader hold ~100 µs, measuring reader queueing behind a queued
  writer. Linux `std` is what the container has; say so, and ask the author for the Darwin run.
- **Base-wait attribution on the current build**: run `bench --host-nls --nls-only --scaling-only`
  at `--connections 2` and print `ENGINE_LOCK` beside `VIEW_LOCK` after `nilestream_lockstats
  reset`. Two cores make the absolute figures meaningless and the *ratio* of base to view wait is
  still a fact.
- **The checked-twice sweep** (§6.4): per-function callgrind on all four probe shapes, attributed to
  the functions named there.
- **The load-dependent sweep** (§8.1): three runs at `--test-threads=8` under two `yes >/dev/null &`
  hogs, then one pinned to one core at `--test-threads=1`. Cycle 9's execution ran this at C9-00 and
  it is the class that comes back.
- **The generator** (§5, fact 1): count how many `#[cfg(test)]` occurrences in the workspace precede
  a `pub` item in the same file. Each one is a public item the appendix silently omits today.
- **`gbs` against `niles` at `688919c` / `53b40f6`**: build it, run its gate, and say whether the
  cross-repo guard (`crates/nilestream-core/tests/downstream_adapter.rs`) actually compiles the
  GBS adapter or a copy of its shape.

Where a container figure is a ratio, say so; where it is a count, it travels.

---

## 7. Where *not* to spend the cycle

Not on compile time, startup or binary size as priorities (measured; refuted twice; measured again
in cycle 8 and found sub-millisecond). Not on a distributed protocol, consensus or cross-shard
commit. Not on the wire's binary formats beyond LC-22. Not on E2EE. Not on re-litigating LC-21
(fail-stop) or LC-28 (epochs). Not on `unsafe` — there is none outside the measurement tool and a
test asserts it. Not on a feature for GBS that stresses no invariant. **Not on the view lock**: it
was measured and exonerated; a task that touches `VIEW_LOCK` needs a number that contradicts
`c9-pending-results.md` §3 first.

---

## 8. Check these regardless

Both auditors, independently, and state your evidence class for each:

1. **Run the gate yourself** on both trees at the heads in §4 and record every row (PostgreSQL 16
   started first; report its absence as an environment fact, never as a failing test). Then
   `cargo test --offline --workspace --no-fail-fast -- --test-threads=8` **three times** under two
   `yes >/dev/null &` hogs and once pinned to one core at `--test-threads=1`, and record any test
   that changes verdict.
2. **Read `answer_from_view` against the lock order and the slowest-16 tables** in
   `c9-pending-results.md` §3 and say, from source, why a base *read* guard queues later readers —
   or that it does not, and where the base wait then comes from.
3. **Read `Rev::finish_fold` and Theorem 4.1 clause (5)** and say whether 5a–5c are each proved
   without the generation check, as the execution report's fact 3 claims.
4. **Re-derive the C9-06.2 arithmetic** from the transcript in `c9-pending-results.md` — medians,
   pooled MADs, the 1.018× — and say whether the gate's two arms were applied as stated.
5. **The GBS tree on the Mac** — its head, and whether it builds against the niles head.
6. **Every `MISMATCH` and `BLOCKED` marker in both trees**, in one table with its status — seventeen
   `MISMATCH` and five `BLOCKED` families in niles (source and prose together), five and seven in
   gbs, at the heads in §4 — and
   say which have been repaired without their marker being struck.

---

## 9. Potential misses in every cycle so far

Nine cycles have found that the highest-value defects were **correct on every input and wrong in
structure**, invisible to a test suite that never asked: a verdict rule that could not tell
"refused" from "did not run"; group commit structurally unreachable; a `&mut` for a counter; a
deadlock no test could catch; a read reporting the wrong end of its own interval; a plan cache that
emptied itself; a recovery that returned a prefix with a success code; a lattice state that exists
in the theory and nowhere in the engine; **and, in cycle 9, an instrument whose numbers were
inferred rather than measured and which sent a whole task after the wrong lock.** Look for the
class, not the instance. Candidates no cycle has swept, with cycle 9's additions first:

- **Baselines inferred from documents.** C9-06.2 is the instance. Every pass line in every prior
  work order that cites a figure from `docs/audit/cycle-<n>/` rather than from a script's transcript
  is a candidate; enumerate them.
- **Instruments that can only confirm.** The view-lock histogram was added to convict; it acquitted
  only because the harness was corrected around it. For each histogram, counter and table in
  `lockstats.rs` and `nilestream_stats`, say what result would falsify the hypothesis it was added
  for, and whether the harness can produce that result.
- **Hazard classes overstated by a rung.** Fact 3 (§5): a correctness guard that is really a
  liveness-or-cost guard. Sweep every "→ divergence" in cycle 9's guard specifications and every
  "cannot" in Chapter 3 for the same shape.
- **Generators that truncate or omit.** Fact 1: `gen-appendix-d.py` cuts at the first
  `#[cfg(test)]`. Every generated document has a rule for what it includes; each rule is a place
  where a true thing can silently stop being said. `include-results.py`, `gen-appendix-d.py`, the
  E18/E19 renderers, `docs/keywords.md`.
- **Environment facts recorded as tree facts.** Fact 2. The preflight table is a claim about a
  machine, dated; anything in it that a later container inherits as true is a miss waiting.
- **Syncs that were given once.** The GBS block. Any instruction to the author that is not repeated
  until confirmed.
- **Load-dependent preconditions**, **prose beside a table that could contradict it**, **things
  measured from one configuration and stated for the design**, **sizes nobody has a number for**,
  **the two spellings of one concept** — all still open from cycle 9's §9, none swept to completion.
- **What the audit's own instruments assumed.** Cycle 9's `c9-pending.sh` assumed
  `--checkpoint-interval` would be refused by both arms and probed it; it did not assume PostgreSQL.
  Whatever your first script assumes, probe it.

---

## 10. Constraints the executing agent is bound by — carry them into the work order

Non-negotiable; no task weakens one without an explicit decision routed to the author:

durable-before-visible; one sealer per ledger; one total epoch order; full retention; honest
absence (as chapter 3 now states it: a `Hole` keeps its version, a cancelled flight restores the
exact slot it replaced, a `⊥` means "no cached assertion", never zero); fold-never-field; product
purity; self-describing amounts; GBS layering; **no `unsafe`**; **no default-on-error**; **zero
external dependencies**; no fabricated results; the thesis follows the code with `MISMATCH-<id>` at
every divergence; a red test is a result and is never fixed by deletion; honest refusal over silent
fallback; `BLOCKED-<id>` on ambiguity; apply-before-publish; the lock order **O < B < P < V < C**,
S a leaf, **F (a flight's completion) a leaf below V**; every optimisation ships its guard in the
same commit and the executor **proves the guard fails on the reverted change in a disposable
`git worktree`** — a compile error is never the witness — transcript in the report; no performance
claim without a reproducible command and a provenance header; **a pass line is written against a
measured baseline, never a document**; **benchmarks touch committed artefacts only with
`--publish`, never from a container**; micro-benchmark gains are never contract results; a
currency's wire code is its declaration index; the idempotency window is counted in transactions;
`Rev::read` and `Rev::begin_read` answer at the anchor asked — no caller may reintroduce a
compensating branch; a keyed read takes the base at most once; a joined reader holds nothing while
it waits; flights and waiters are bounded and a refusal is counted; an install is owned by its
generation; `deferred_merges` is reported even while it reads zero.

Environment: **no task may require network access at execution time; no task installs anything.**
Rust 1.95.0 (`RUSTUP_TOOLCHAIN=stable`, `RUSTUP_AUTO_INSTALL=0`), valgrind 3.22, `psql` and a
startable PostgreSQL 16 (started explicitly) are preconditions. Edits to Rust source through Python
are exact-string, single-occurrence replacements asserted before writing. Attribution is taken
from the executing session's own instructions; never bake in a trailer.

**Sync at the end, and at every landing.** The executor has no remote and cannot push. It ships
each commit as a `git bundle` written to `~/Documents/niles-sync/cycle-10/` on the Mac, and tells
the author — **at the moment it is needed, not at the end, and again at every landing until it is
confirmed run** — exactly what to run:

```
cd ~/Documents/niles
git fetch ~/Documents/niles-sync/cycle-10/<bundle> c10/<branch>
git merge --ff-only FETCH_HEAD
git branch -f c10/<branch> HEAD
git push origin c10/<branch> c7/01-durable-rows
RUSTUP_AUTO_INSTALL=0 cargo +1.97.1 clippy --offline --all-targets -- -D warnings
```

(`cd ~/Documents/GBS` and `c7/00-adapter` for GBS.) A lint the container's 1.95.0 cannot see is
fixed from the author's pasted output, never guessed.

---

## 11. Open questions to carry, restate or close

Settled, **not to be reopened**: LC-01, 02, 04, 08–12, 14, 15, 17, 18, 21 (fail-stop), 28
(epochs). **Decided in cycle 9**: LC-16 (durable by default; `--volatile` explicit), LC-35
(undeclared window refused).

**Reopened in cycle 9, and the cycle's first question:** **LC-23** — the tail is the *base* lock,
not the view; the closing attribution was inferred from an uncorrected harness. **LC-24** (p99 at
16) is the same question.

Carried, restate or close with evidence: **LC-03** `AccountId` String→u64; **LC-05** rendered
NULL / refusal boundaries; **LC-06** allocator / process boundary; **LC-07** E23 slope; **LC-13**
release checks; **LC-19** the wire claim's three clauses (fold into one chapter-6 sentence after a
product trace exists); **LC-20** still `BLOCKED-LC20-definition` — the author supplies the subject
or the number is retired; **LC-22** declared scale on the wire, transcript first; **LC-25**
concurrent crash; **LC-26** same-process A/B; **LC-27** E19 CSVs `not_run` until republished;
**LC-29** oltp verdict concurrency into SPEC-ENGINE Part 0; **LC-30** Pending at a different
anchor — **implemented as proposed and guarded; the author has not yet confirmed the protocol in so
many words**; **LC-31** what a hole is (C9-08); **LC-32** checkpoint interval (C9-07); **LC-33**
plan-cache policy (C9-09/C9-11); **LC-34** first product — C9-G01, still; **LC-36** a damaged
final record after a clean shutdown, `BLOCKED-recovery-tip`, the author's contract to give.

New, raise and position — the author decides:

- **LC-37** — the base-lock hold across a reconstruction: snapshot handle, frontier-only guard, or
  something else; and whether `Base` grows a method for it.
- **LC-38** — the deferred-delta merge: when a flight lands behind `applied`, fold or pin; and the
  `e − a` distribution that decides it.
- **LC-39** — whether `pinned_installs / reads` is a phase-diagram input: it is a measured
  property of the workload under a moving frontier, and no cell of the diagram currently sees it.
- **LC-40** — generated-document integrity: what each generator is allowed to omit, stated once,
  with a test.
- **LC-41** — partitioned conservation: can `conserve per (…)` state a partition of the key space
  and a *prohibition* on cross-partition flow (CCP account segregation), and is that a new
  invariant or a new spelling.

---

## 12. What you must produce

A single work order containing:

0. **The preflight output, verbatim, with its admissibility table filled in** — `bash
   docs/audit/cycle-10/preflight.sh <niles> <gbs>` in the container, and ask the author to run it on
   the Mac and paste the output; both, labelled.
1. **An executive judgement** — where the three artefacts stand against the three goals, with the
   arithmetic for any reachability claim, and an explicit answer to each of §1's three questions.
2. **Findings**, each with a class (wrong-measurement, correctness, liveness, guarantee-bounded,
   instrument-gap, negative, stale-claim), an **EV score** (`impact × confidence ÷ cost`, each
   1–5, ties to correctness), **HI** or **HS**, evidence with file and line, the measured cost, the
   proposed repair, and **what the repair gives up**. Negative findings are first-class; a finding
   that a prior cycle's claim is false is first-class.
3. **A task list in dependency order** — each task carrying what it closes, its files, its
   **baseline measured on the executing host or by a named script the author has run** (never a
   document), its **target as a ratio or a sentence**, its method, its acceptance test, its **guard
   and the reversion that must make it fail**, and its guardrails. Mark a **cut line** for one agent
   in one cycle. **Every target line must be a sentence the executor can mark `done` or `not done:
   why`** — the reporting checklist is copied from it verbatim, and the executor is told so.
4. **Branch stacks and merge order** for both repositories from `c10/00-audit` (niles) / `688919c` (gbs), named `c10/*`.
5. **A validation protocol**: commands, what green means, what a red row means, and which reds are
   environment.
6. **Every Host C script**, in the tree under `docs/audit/cycle-10/hostc/`, named, with its expected
   runtime, its retarget-and-refuse clause, its own daemon where needed, its PostgreSQL check where
   needed, a `--publish` refusal, and an explicit "**the author must run this now**" marker in the
   work order where its result is needed. **The first script measures the baseline the first task
   is scored against.**
7. **The open-questions ledger** (§11), updated.
8. **Reporting requirements** for the executing agent: the checklist with every target line
   verbatim and its status; guard transcripts naming the disposable worktree, with exit codes;
   timing tables with a host column; the MISMATCH/BLOCKED inventory; worktree status separating the
   five untracked files (sizes) from task changes; every SHA and bundle hash; **exactly three
   material facts found while executing that the work order did not cover**; and a sync section
   listing every bundle and the author's commands, **repeated at every landing until confirmed**.

Order by `impact × confidence ÷ cost`, not by interest. And weigh three things: the stated goal is
efficiency; the **binding constraint** is correctness — a liveness bug outranks a factor of two, and
a durability claim that is not true outranks both. The highest-value defects have been invisible to
the suite and correct on every input: look for the class. And **audit your instruments before you
audit the code, and ask of each whether it could ever have said no** — cycle 9's could, once the
harness was fixed, and that is the only reason the cycle's largest finding exists.

---

## 13. Sources for §6.5

Vendor material read for this brief, September 2026 — none of it a specification; each is a
vendor's description of scope, to be read as "what a buyer of these systems expects to exist":
Finastra, *Loan IQ* solution page (`finastra.com/lending/solutions/loan-iq`) and the 2023 brochure
(`resource-loan-iq-brochure.pdf`), the Loan IQ solution overview
(`finastra.com/viewpoints/brochure/loan-iq-solution-overview`), and Luxoft's partner overview
(`luxoft.com/files/pdfs/finance/Loan-IQ.pdf`); Nasdaq, *Calypso* collateral, margin and securities
finance (`nasdaq.com/solutions/fintech/nasdaq-calypso/collateral-margin-securities-finance`),
clearing (`…/clearing`) and treasury (`…/treasury`) pages; and Quinnox's platform guide
(`quinnox.com/blogs/nasdaq-calypso-guide/`). Cycle 9's §13 sources stand beneath these.
