# Cycle 11 audit brief — **Astra**

**You are auditing, not building.** Your single deliverable is a **work order**: a prioritised,
evidence-backed set of tasks — and, this cycle, a set of *decisions* — that a building model will
execute in a later session. You write no production code into either repository. You have no shell. Everything you know about these repositories comes from reading them — on GitHub, on the author's Mac through the desktop bridge, or as files the author attaches — and from output the author pastes back. Every request for a run is a complete copy-pasteable block with its expected runtime and what each possible result would mean, written *before* the result comes back. Your work order is a file the author places at `docs/audit/cycle-11/astra-work-order.md` and commits; you do not commit it.

The work order you produce is the only thing the executing agent sees. Assume it has none of your
context: every task carries its own preconditions, baseline, target, method, acceptance test,
guard, and guardrails, and must be executable **offline**.

**A second auditor — Fable — is reading the same two repositories independently, from a
companion brief.** Fable has a container and measures; its brief leans empirical. Yours leans structural: claims versus code, the type system and what it proves that the engine re-checks, the thesis text against the runtime, the lock order against the source, the decision record against what was actually decided, GBS's product model against what Loan IQ's and Calypso's users would need. The author reconciles the two work orders afterwards, as in cycles 9 and 10 (`docs/audit/cycle-10/work-order-10.md` §2 is what that looks like: every finding carries the other auditor's verdict — *confirmed*, *confirmed-narrowed*, *not re-read* — and the disagreements are argued out by name). Three consequences:

1. **Where you both look at the same thing and disagree, the disagreement is the valuable part.**
   State your evidence class on every finding. Never round toward the other auditor.
2. **Do not soften a finding because you expect Fable to catch it.**
3. **§9 lists things you must both check independently.** Do those even knowing they are duplicated.

**This cycle is different in kind.** Cycles 1–10 asked *what is wrong and what to fix next*. The
author has now asked a prior question, and it is the first section of your work order: **given
everything that has been built and measured, was the plan right, should any of it be undone, could
a better plan have been followed from the beginning, and does the knowledge and ability exist to
finish Niles, Nilestream and GBS at all?** Answer it before you propose a single task. §1 states
the question in full; §2 gives you the record to answer it from. A work order that proposes
cycle-11 tasks without first answering §1 is not the deliverable.

**Cycle 10's headline task was built and measured, and it lost.** The deferred merge (T04) was
motivated by a real, measured number — 99.1% of installs land pinned with a 4.6–5.7 epoch suffix —
and the mechanism, built and correct, costs **7–13% of read throughput at every level and both
working points**, 9–17 pooled MADs from zero. The executor's first explanation was refuted by its
own probe. Read `docs/audit/cycle-10/execution-report.md` in full before anything else — its T04
sections, its dated correction, and MF-1 through MF-14 — and treat every figure in
`docs/audit/cycle-10/work-order-10.md` as a claim to re-derive, including the ones this brief
repeats.

---

## 1. The question this cycle answers first

The stated end goals are **a highly efficient query language (Niles)**, **a highly efficient query
engine (Nilestream)**, and **a highly efficient core-banking system (GBS)** — efficient in **memory
and speed** — poised to replace systems of the class of **Finastra Loan IQ** (commercial and
syndicated lending) and **Nasdaq Calypso** (capital markets: trading, risk, collateral, treasury,
clearing, post-trade), **implementing their functionality on GBS**, without surrendering any
commitment in §11. The thesis's claim is a theory; the two systems are its instrument; the
banking constructs are a domain layer on a fully general relational core.

Ten cycles in, the author asks four things, and your work order opens with a section answering
each, in this order, with evidence:

**1.0 The fundamental questions, answered before the decisions are audited.** The author wants
to be *very sure* of four things, and every later answer in your work order is conditional on
these. Treat each as a hypothesis with a refutation condition, and test it against the strongest
existing alternative rather than against a straw one.

- **Is a new query language needed?** State what Niles proves statically that SQL — with the
  extensions a real bank would actually use (PostgreSQL's `CHECK`, `NUMERIC`, range types,
  triggers, row-level security; temporal tables in SQL:2011; Materialize's and Feldera's SQL
  over DBSP) — cannot, and *whether that gap could be closed by a library, a linter or a typed
  embedding* (a Rust DSL, a Datalog, a checker over SQL's AST) at a fraction of the cost of a
  language. The obligation catalogue in `crates/niles-lang/src/typecheck.rs` and
  `results/obligations.csv` is the evidence; count what is proved, what is discharged to the
  runtime, and what a competent SQL deployment re-checks at runtime anyway. If the honest answer
  is "a typed embedding over SQL would prove 80% of it", say so and cost the remaining 20%.
- **Is a new database management system needed?** State what the REV runtime does that Noria,
  Materialize, Feldera/DBSP, ReadySet, or PostgreSQL with logical replication into a
  view-maintenance layer cannot — *partial materialisation with epoch-anchored certification over
  an immutable ledger* is the claim — and whether each alternative could host the ledger as a
  table and the REVs as maintained views with the same guarantees. Name the guarantee each
  alternative fails, with a citation to its documentation or a reproducible probe. Then the
  harder half: is the *write path* (a sealer, hash-chained epochs, `F_FULLFSYNC` per batch)
  something an existing ledger-grade store (TigerBeetle; an append-only PostgreSQL table with
  a chain column; a log like Kafka with idempotent producers) already gives, and if so what is
  the engine *for*?
- **What is needed to build a core banking system with unbounded data and infinite-but-skewed
  derived views?** Not the thesis's answer — a first-principles list: the invariants (double
  entry, conservation per currency, idempotency, bitemporality, retention, auditability), the
  operational properties (durable-before-visible, restart without loss, a total order), the
  product surface (§6.4), the scale (accounts, postings per day, views per account, tail
  latency at the teller and at the regulator), and the *people and time*. Then mark which items
  the current trees demonstrate, design, name, or contradict. This list is the spine of §1.4.
- **Can the skew be measured in a specific data system or universe?** The phase diagram's whole
  claim rests on a Pareto-skewed family of derived views; the benchmark assumes Zipf with s=0.9
  (`crates/bank-bench`). Propose a measurement of the *actual* skew — of account access, of view
  reads, of key popularity over time — on at least one real or realistic universe: a public
  payments or transaction dataset, a synthetic generator calibrated to published statistics, or
  an instrumented replay of GBS's own product traces. State the estimator (exponent fit, Hill
  estimator, head-share at 1%/10%), the sample size needed, and what result would *refute* the
  thesis's premise (a skew too shallow for demand materialisation to pay is a phase-diagram
  boundary, and the thesis must be able to land on either side of it). If no such universe is
  reachable offline, say what the author must obtain and how the benchmark should be labelled
  until then.

**1.1 Were the right decisions made?** Every decision in the record — D-1 through D-8, LC-01
through LC-42, the cut lines of cycles 7–10, the choices to keep C9-06 after its speed claim died
and to build T04 after its negative-experiment framing was withdrawn — is a hypothesis to be
tested against what was measured afterwards. For each decision that mattered, say: what it
decided, what evidence it rested on at the time, what was measured since, and whether it stands,
stands-narrowed, or is refuted. A decision refuted by later measurement is a first-class finding.
Do not confine yourself to the ledger's numbering: the largest decisions in this project were never
given a number — *build a language rather than extend SQL*, *build an engine rather than adopt
one*, *a two-repository split with GBS as a consumer*, *epochs rather than wall-clock*, *Rust with
zero dependencies*, *a three-model pipeline with a builder that cannot push*. Each gets a row.

**1.2 Should any commits be undone?** Produce a **revert ledger**: for every commit on
`c9/*` and `c10/*` (and any earlier commit you find reason to name), one of *keep*, *keep but
default off*, *rework*, *revert*, with the evidence. Candidates this brief already knows about,
for you to confirm or refuse: the deferred merge (`7b6b704`) — a correct mechanism with a measured
cost and no measured benefit; the two-phase read (`78acd64…a1dbca6`, cycle 9) — kept "for
completing the lattice" after +1.8% replaced a predicted ≥25%; the branch collapse of cycle 10 (T02
and T04 landed on `c10/02-read-safety` rather than on the `c10/03-…`/`c10/04-…` branches the work
order specified — is a stack that can no longer be reverted task-by-task a defect?). A revert is a
task with a guard like any other: it must be proven to restore the measured baseline, not assumed
to.

**1.3 Could a better plan have been followed from the beginning?** Not a lament — a comparison.
Write the plan you would have written at cycle 1 knowing what is known now, in the same shape as a
work order (dependency order, cut lines), and diff it against the trajectory actually taken.
Where the diff is large, say what it would cost to move to the better plan *now* — including the
option of a **reset** of some component (a rewrite of the read-model runtime, of the compiler's
lowering, of GBS's product layer) — with the cost stated in cycles and the risk stated in
correctness. **Be explicit about what a reset would throw away that is currently proved**: 939
test functions, a torn-record corpus, a conservation suite, a lock-order guard, fourteen recorded
material facts. A reset that discards proved properties has to re-prove them, and that is part of
its cost.

**1.4 Does the knowledge and ability exist to build these three things?** A reachability
judgement, per artefact, against the full stated scope — including the parts no cycle has touched:
the self-hosting compiler and three-stage bootstrap (Appendix E), native targets for Linux/macOS
ARM64, Windows x86-64, WSL and WASM (Appendix C), the UDF WebAssembly ABI, end-to-end encryption,
distributed execution and the cross-shard commit protocol, and the product list (FX, derivatives,
forwards, swaps, OTC caps and floors, revolving/term/syndicated lending, trade finance, letters of
credit, supply-chain finance, sweeps, cash pooling, zero-balance structures). For each: what has
been *demonstrated* (a test, a measurement, a transcript), what has been *designed* (a chapter, a
spec), what is *named only*, and what is *contradicted by evidence*. Then the two questions the
thesis itself sets as kill criteria — **the ledger floor** and **the read-model break-even** — with
the numbers that exist and the numbers that are missing. Then, plainly: with the pipeline as it is
(one builder, two auditors, one human who runs scripts), how many cycles to a defensible thesis,
how many to a system a bank could pilot, and which of the two the author should be optimising for.
If the honest answer is "the thesis is reachable and the pilot is not", say so and say what the
pilot would additionally need. **Do not round toward encouragement.**

**1.5 Which model should audit, and which should build?** Evidence, not preference. The record
contains attributable errors by every participant. The executor recorded its own: MF-9 (writing
the class of bug it was repairing), MF-10 (writing MF-4's bug having written MF-4), MF-12 (an edit
script that lost a repair whose commit message described it), MF-13's twenty misleading refusals,
the T04 counter conflation that produced a withdrawn recommendation, and the branch collapse.
The auditors recorded theirs: cycle 9's headline task was wrong and the executor measured it;
cycle 10's §6.2 framed the merge on a 22:1 ratio that was correct and still led to a regression.
For each participant model, count the findings that **survived measurement** and the errors that
**were caught by the harness rather than by the model**, and recommend, with criteria: which model
should write work orders, which should execute them, whether the same model should ever do both
in one session (this cycle's execution was finished by the auditing model after a mid-session
switch — say whether that shows in the record), and what the human's irreducible role is. Name
models by the identifiers the sessions report, not by reputation.

---

## 2. The record you answer §1 from

Everything is in the two trees. In reading order:

- `docs/audit/cycle-10/execution-report.md` — cycle 10 end to end, 1,100 lines: T00, T00a, T01,
  T02, T04, the Host C A=A noise floor, the T04.2 regression table, the dated correction of its own
  T04 paragraph, and MF-1 … MF-14.
- `docs/audit/cycle-10/work-order-10.md` (`c075bae`) — the consolidated cycle-10 order; §2 is the
  reconciliation of the two cycle-10 audits; §7 is the open-question ledger as it stood.
- `docs/audit/cycle-9/execution-report.md` §11 and `docs/audit/cycle-9/hostc/c9-pending-results.md`
  — where the view-lock attribution died.
- `docs/audit/cycle-8/lc-23-attribution.md` — refuted; read it to see what a wrong attribution
  looked like from the inside.
- The thesis, `thesis/*.md`, with its `MISMATCH-*` and `BLOCKED-*` markers — the map from claim
  to code. `docs/SPEC-ENGINE.md`, `docs/SPEC-LANGUAGE.md`, `docs/BUILD-LOG.md`.
- `results/MANIFEST.csv` — every results file with its class; the five classes tell you which
  numbers a green `make reproduce` vouches for and which it does not.
- The Host C scripts under `docs/audit/cycle-10/hostc/` — `c10-baselock.sh` (with `--self-test`,
  `--baseline-only`, `--merge-arms`), `c10-gates.sh`, `c10-rwlock.sh`, `c10-merge.sh` — and what
  each one's `--self-test` proves about itself.

### 2.1 What cycle 10 did

| task | status | where |
|---|---|---|
| T00 instruments + Host C baseline | done; A=A noise floor **0.83% worst, ~1 pooled MAD** at 5×2×3 | report §T00, `c10-baselock.sh --baseline-only` |
| T00a gate failures | done (5 targets) | report §T00a |
| T01 served certification, wait ownership | done (A10-01…05, A10-12) | report §T01 |
| T02 truthful install and conserve | done (A10-06, A10-07; NL0224/NL0225) | report §T02 |
| T04.1 bounded certified merge | done (nine cases, five witnesses) | report §T04 |
| T04.2 merge measured | **done, negative: −7% to −13%, 9–17 MADs, no claim** | report §T04.2 |
| T07 served checkpoints | **not taken** — written against "the immediate merge-enabled control", which now loses to the pinned one | — |
| below cut: T03, T05, T06, T08, T09, T10, TG01 | not taken | work order §3 |

Two things the work order did not anticipate and you must weigh. **The branch plan was not
followed**: T02 and T04 landed on `c10/02-read-safety` (`73fe782`, `7b6b704`, …) instead of the
specified `c10/03-verified-contracts` and `c10/04-deferred-merge`, so cycle 10's stack is one
branch and cannot be reverted task by task. And **the results directory grew a fifth class**
(`host-scoped`) and the reproduction diff now derives its exclusions from the manifest — decide
whether that is the right end state or a patch on a patch.

### 2.2 The fourteen material facts, as a class

MF-1 … MF-14 are in the report with evidence. Read them as a family, because they are one:

| | fact | the shape |
|---|---|---|
| MF-1 | a source guard satisfied by another guard's source | a check that can be satisfied by prose |
| MF-2 | the benchmark read a column renamed a cycle earlier and defaulted it to 0 | a missing input rendered as a zero |
| MF-3 | a lock-order scan passed on the comment describing the deadlock | a check that can be satisfied by prose |
| MF-4 | three guards written in the cycle were vacuous | a guard that cannot isolate what it tests |
| MF-5 | a port check true on Linux, false on Darwin | code true on the platform in front of it |
| MF-6 | the desktop bridge is a Linux VM, not a Mac | an environment fact recorded as a host fact |
| MF-7 | a toolchain override invisible where it was written | an override untested by construction |
| MF-8 | `make reproduce` compared columns its manifest called incomparable | a classification nothing enforced |
| MF-9 | the executor wrote the bug it was repairing (a `match` on a lock-taking scrutinee) | a hazard the tree already met and nothing prevents |
| MF-10 | the executor wrote MF-4's bug having written MF-4 | as MF-4 |
| MF-11 | a probe that measured the value it was about to overwrite | a fix for MF-7 with MF-7's shape |
| MF-12 | an edit script that asserts late and writes late lost a repair | a method with no verification step |
| MF-13 | a refused build followed by a full sweep against the refused binary | a refusal that does not stop what it refuses |
| MF-14 | a toolchain probe run in the wrong directory | a fix for MF-11 with MF-7's shape |

Three of these (MF-7, MF-11, MF-14) are one defect repaired three times, each repair carrying the
shape of the last. Two (MF-4, MF-10) are the same defect written twice by the same process. Three
(MF-1, MF-3, and cycle 9's paired adapter gate) are a check passing on words. Whatever you
conclude in §1.5, this table is evidence in it. It is also the strongest argument in the record
that **the harness — the self-tests, the revert witnesses, the manifest, the arm-label refusal —
has caught more than any participant has**, and that the next cycle's highest-value work may be
on the harness rather than on the engine.

### 2.3 Where the trees stand

| repository | head | branch | tests |
|---|---|---|---|
| `niles` | **`653a369`** on `c10/02-read-safety` (after the author runs §4) | `c10/00-audit` = `96b6d9e`, `c10/01-instruments` = `8e708db`, `c10/02-read-safety` = `653a369`; cycle base `c075bae` | 939 test functions; **1,087 pass / 0 fail / 8 ignored** (container) |
| `gbs` | **`01e1d85`** on `c10/00-adapter-guard`, from `688919c` | | 504 / 0 / 1 workspace; adapter 53 / 0 / 0 |

Cycle 10's commits on `c10/02-read-safety`, oldest first: `12d87dd` `c7d3955` `0ff16a4` (T01) ·
`8fd2477` (report) · `73fe782` `f3a668e` (T02, report) · `7b6b704` `dfac93f` `bca7907` `d680c83`
(T04.1, T04.2 instrument, toolchain, report) · `d301296` `5ff860e` (fatal build) · `3b4f7d9`
`67c313b` `653a369` (toolchain probe, T04.2 result, README). The cycle-11 briefs are on
`c11/00-audit` above `653a369`.

Gate, both trees: `cargo fmt --all -- --check`; `cargo clippy --offline --all-targets -- -D
warnings`; `cargo test --offline --workspace`; `make reproduce` (green at `67c313b` in the
container; on Host C, green once PostgreSQL is on 5432 — `numeric_binary_oracle` is a host fact
otherwise); `bash docs/audit/cycle-10/hostc/c10-gates.sh` on the Mac runs all of it across both
trees and the paired adapter. A red row is a result; never fix it by deletion.

---

## 3. Access

### 3.1 GitHub

Both repositories are private, owned by **`shbiebs`**:

- `https://github.com/shbiebs/niles`
- `https://github.com/shbiebs/gbs`

**Try before you ask.** Ask the author to confirm, or read through the GitHub web interface if your session has it. If you cannot, ask the author for the narrowest thing that works: **read access only**, to those two repositories and nothing else — a fine-grained personal access token with *Only select repositories* → `niles`, `gbs`, and *Contents: Read-only*; the author supplies it out of band, you never write it into an output, and you never push. If access is refused, say so in your work order and audit what you can reach: the bundles under `~/Documents/niles-sync/` on the Mac are complete histories and the author can attach them. 

The heads to audit are in §2.3. Confirm them on the remote before reading anything else, and
confirm that `c10/02-read-safety` on niles and `c10/00-adapter-guard` on gbs are at those heads.
Cycle 10 ended with every branch pushed; cycle 9 did not.

### 3.2 The author's Mac

You are authorised to audit the working copies at `/Users/checolino/Documents/niles` and
`/Users/checolino/Documents/GBS`. Both are at the heads in §2.3 after the author runs §4, and
clean apart from **five untracked files that are the author's and must never be edited, staged,
deleted, moved or bundled**: `.DS_Store`, `AGENTS.md`, `niles/.DS_Store`, `thesis/.DS_Store`,
`thesis/Niles-Thesis.pdf` — **10,244 / 16,639 / 6,148 / 8,196 / 816,110 bytes**, unchanged through
cycles 9 and 10. Report their sizes at the end; a discrepancy is referred to the author, never
corrected. **Read-only discipline**: no commits, no staging, no `git clean`, no checkout of another
branch without saying so first.

Everything that needs the real machine — a build, a benchmark, a storage probe, `psql` — is a
shell script the author runs in Terminal.app, living **in the tree** at
`docs/audit/cycle-11/hostc/<name>.sh`. `docs/audit/cycle-10/hostc/c10-baselock.sh` is the template
and it is a better one than cycle 9's: it has a `--self-test` that injects every fault it refuses
and requires the refusal *and* the clean control; it refuses a dirty repository but waives the
files its manifest says are host-shaped, by name; it refuses a stale worktree and **stops**
(MF-13); it labels its arms from what the daemon reported, not from the flag it was given; it
probes the toolchain from inside the repository and prints the declared pin beside what resolved
(MF-14). Every new script inherits all of that or says why not. Say **explicitly, at the point in
your work order where it is needed**: "the author must now run `bash
~/Documents/niles/docs/audit/cycle-11/hostc/<name>.sh` and paste the output." Every script is
idempotent, non-interactive, bounded in time with the runtime stated, and never assumes `chmod +x`.

**The author's zsh has `interactive_comments` off**: a `#` at the prompt is a command. No comments
in pasted lines. `git pull --ff-only <bundle> <ref>` moves the branch that is **checked out** and
never creates one — cycle 10 lost a landing to exactly that; a new branch is created with
`git branch <name> <sha>` and checked out *before* anything is force-moved. Never `git branch -f`
the branch that is checked out. A sync block names **the ref the bundle actually carries**, checked
with `git bundle list-heads` before the block is written. Repeat every owed sync block at each
landing until confirmed. And **the author runs scripts from wherever they are standing** — a script
that depends on its working directory has to set it (MF-14).

**Bridge traps, if you reach the Mac through the Claude desktop bridge.** You get a Linux VM over
FUSE, not macOS: no `rustc`, `cargo`, `psql` or `valgrind`; `platform.system()` says `Linux`;
fine for reading, useless for measuring, and **it cannot validate Darwin behaviour** (MF-6). Any
`git` command that touches the index leaves a `.git/index.lock` the bridge cannot delete — use
`git --no-optional-locks` for reads. Files are delivered to the Mac by writing them to
`~/Documents/niles-sync/cycle-11/` through the bridge; a bundle written there and a fetch pasted
for the author is the whole delivery mechanism. The author's connected folder is `~/Documents`.

The real Mac is **Host C, the reference host**: Apple M4, 10 cores, Darwin 25.6.0, APFS on NVMe,
`F_FULLFSYNC` at ~205 barriers/s (measured this cycle; ~255/s in cycle 9), Rust **1.95.0 pinned
and resolving** with 1.97.1 also installed, PostgreSQL 16 installed but **not running on 5432**
in the author's shell (so `numeric_binary_oracle` is red as a host fact; the wire transcript needs
only the `psql` client, which is 18.6 Homebrew). It gives the same answer twice: A=A worst
separation 0.83% across 28 replicates.

### 3.3 What you cannot do, and what you do instead

You cannot build, run a test, or measure. Fable's container can, and the two of you are read side
by side. When a claim needs a number, write the block for the author and say what each possible
result would mean before it comes back — a request whose interpretation is written after the
answer is a request that cannot be wrong. The Mac's gate is the author's oracle; the container's
is Fable's. Yours is the source, the thesis, and the decision record.

---

## 4. Sync owed at the start

**The author must run this now**, before either auditor reads code. The bundle
`niles-c10-close.bundle` carries `refs/heads/c10/02-read-safety` at `653a369` and
`refs/heads/c11/00-audit` above it; the author's Mac is at `5ff860e` on `c10/02-read-safety`.

```
cd ~/Documents/niles
git bundle list-heads ~/Documents/niles-sync/cycle-11/niles-c10-close.bundle
git pull --ff-only ~/Documents/niles-sync/cycle-11/niles-c10-close.bundle c10/02-read-safety
git fetch ~/Documents/niles-sync/cycle-11/niles-c10-close.bundle c11/00-audit
git branch c11/00-audit FETCH_HEAD
git push origin c10/02-read-safety c11/00-audit
git rev-parse --short HEAD c11/00-audit
```

Expected: `653a369` and the `c11/00-audit` head printed by `list-heads`. GBS owes nothing:
`c10/00-adapter-guard` = `01e1d85` is on the Mac and on GitHub.

---

## 5. Hosts, and what each may conclude

| host | may conclude | may not |
|---|---|---|
| **C** (the Mac) | absolute wall-clock, durability, scaling to 10 cores, contract verdicts, anything published | — |
| **a cloud container** (2 cores) | deterministic counters, within-container ratios, tests, guards, mutation runs, the shape of a mechanism | any absolute figure beside another host's; any durability row; any curve past 2 cores; anything published |
| **the desktop bridge** (Linux VM over FUSE) | reading, grepping, counting, delivering files | building, measuring, Darwin behaviour, anything with `git` that writes an index |

A gate fires only on ≥10% **and** ≥3× the pooled MAD (`sqrt((MAD_A²+MAD_B²)/2)`), else
"noise-limited", which is a result. **A pass line is written against a measured baseline, never a
document.** The noise floor on Host C is measured: 0.83% worst, about one pooled MAD. `unsupported`,
`blocked`, `not run`, `noise-limited` and `negative` are results and omission is not.

---

## 6. Where your budget should go, after §1

§1 decides the shape of the cycle. Whatever it decides, these are the live questions and each one
is a task or a decision in your order.

### 6.1 LC-38 reopened: the merge lost, and nobody knows why yet

The merge is correct (nine cases, five revert witnesses, one of them a cycles-old invariant test),
cheap in the units it spends (9.9 rows and 5 epochs per merge, zero refusals), a 95–98% hit rate
single-threaded against 0% pinned — and 7–13% slower served. The executor's hypothesis: the suffix
is walked **inside the second view hold** (`finish_fold_merging` is called under V by
`answer_from_view`), so ~34 M delta rows per level are read under the one lock every reader and
every append needs; view hold total 3.68 M µs merged against 1.40 M µs pinned at full 6r3w. The
suffix needs only the base, which the reader already holds shared before it re-acquires V.

The author was given three options and has not chosen: **(A)** move the walk outside V and
re-measure; **(B)** record the regression and stop; **(C)** ship `MergeCaps::OFF` as the default
and keep the mechanism behind the flag. **Argue it**, from source and from the transcript
(`~/c10-baselock-out/` on the Mac holds every replicate's log and CSVs). Then measure what you can: an in-container two-arm run of `c10-merge.sh` with the walk moved out of V in a throwaway worktree gives a *ratio* the Mac can confirm or refute, and `the_merge_against_the_arrival_gap` (ignored test, `--nocapture`) is the single-threaded control. And ask the question underneath it, which is LC-37 / A10-10 again: **the read path holds the base shared across the fold, and every cycle's tail is the base lock.** T03's immutable-prefix design is below every cut so far. Say whether it should still be.

### 6.2 T07 and the rest of the cycle-10 backlog, re-baselined

T07 (served checkpoints, LC-32) was written against "the immediate merge-enabled control"; that
control lost, so its baseline is the pinned arm of `c10-merge.sh` and its target sentence needs
rewriting. T03 (LC-37), T05 (LC-40), T06 (checked-twice sweep), T08 (LC-31/33), T09 (LC-19/22/27),
T10 (LC-42, design only), TG01 (F-12/F-24, LC-34): re-derive each baseline at `653a369`, and rank
them with EV arithmetic against §1's answer. Cycle 10's judgement — correctness outranks speed,
and no speed task is scored without a measured baseline — stands until an auditor shows otherwise.

### 6.3 Niles: what the compiler proves and the engine re-establishes

A10-14 and T06 are untouched. Cycle 10 closed two "proved and then ignored" defects (the graph
walk, the conserve grouping) and both were the *opposite* shape — the compiler proved nothing and
the engine trusted it. Sweep both directions: type facts re-derived at runtime (cost, with
callgrind on `checked-twice`), and runtime facts the compiler claims to prove and does not
(A10-06/07 are the two found; LC-41's partition prohibition is the one that is missing by design).

### 6.4 GBS: Loan IQ and Calypso, researched again, and a roadmap rather than a crosswalk

Cycle 10's briefs carried vendor material and a crosswalk (F-79, LC-34) built on the admissibility
rule — a product task is admissible only if it exercises an invariant no row does, closes F-24 or
F-12, or moves a row to `ProductSpecific` with evidence. **That rule stands for cycle-11 tasks.**
But the author's stated goal is broader — *implement their functionality on GBS* — and the two
must be reconciled explicitly rather than by letting the rule quietly narrow the goal. So:

1. **Research both products afresh** (web, September 2026 vendor material and independent
   descriptions; cite what you read). Loan IQ: syndication and agency, pro-rata to a thousand
   shares, PIK, tiered pricing, collateral and covenants, secondary trading, multi-branch GL,
   adjustments/amendments/reversals, ESG-linked lending, the Nexus integration layer. Calypso:
   multi-asset trade capture over shared curves, the single trade record, risk and XVA, collateral
   and margin (VM/IA, SIMM, AANA, thresholds), securities finance and corporate actions, clearing
   (SPAN2/PRISMA/IRM2, ISA/OSA/NOSA/GOSA), post-trade STP, treasury and liquidity, regulatory
   reporting (EMIR, SFTR, SA-CCR, FRTB), tokenized collateral. Extend the crosswalk with anything
   the cycle-10 briefs did not have.
2. **Tier it by invariant, not by feature**: for every capability, (i) which ledger invariant it
   stresses or which new invariant it needs, (ii) whether Niles can *state* it today (LC-41 says
   partitioned conservation cannot be), (iii) what GBS row it would be, (iv) the evidence that
   would make the row `ProductSpecific`.
3. **Cost feature parity honestly.** The author wants to know whether GBS can replace these
   systems. Count the capabilities, count the invariants, and give a cycle estimate for a
   defensible *subset* (syndicated lending end to end, say) and for the whole. That number
   belongs in §1.4.
4. **The product roadmap is tiered by what it proves**: tier 1 closes F-12 and F-24 (a lifecycle
   that survives restart; a product trace the benchmark is shaped by — TG01 is still the
   candidate); tier 2 is each capability that needs a new invariant the language must state;
   tier 3 is everything that is a view over the ledger and therefore "free" only once tier 2 has
   the primitives.

Check first — because cycle 9 could not and cycle 10 did — **whether the GBS tree the author has
is the one you are reading.**

### 6.5 What only you can do this cycle: read

- **The decision record against what was measured** (§1.1), row by row, from the documents and the
  transcripts, with the date each decision was made and the date its evidence changed.
- **The lock order, from source, for every path** — and specifically whether `merge_suffix` under
  V (§6.1) violates anything stated, or only costs.
- **`Base` as a trait against its implementations**, again, because §6.1 and LC-37 both turn on
  whether the row store behind the guard is truly immutable.
- **Theorem 4.1 clause (5) and the merge**: clause 5c says the merge "is also sound by (H-F2) and
  is a strictly better install". The measurement says it is a strictly worse one *served*. Is the
  theorem's "better" a claim about certification or about cost, and which chapter needs a sentence?
- **The type facts the engine re-derives** (§6.3), from `typecheck.rs`'s obligations outward.
- **Loan IQ and Calypso onto the rows** (§6.4), with the tiering, before Fable costs it.
- **The plan-from-cycle-1** (§1.3), which is a reading task before it is anything else.

---

## 7. Where *not* to spend the cycle

Not on compile time, startup or binary size (measured; refuted thrice). Not on the wire's binary
formats beyond LC-22. Not on E2EE, distribution or cross-shard commit *as tasks* — they belong in
§1.4's reachability judgement and nowhere else this cycle. Not on re-litigating LC-21 (fail-stop)
or LC-28 (epochs). Not on `unsafe` — there is none outside the measurement tool and a test asserts
it. Not on the view lock as a *tuning* target — but note that §6.1's hypothesis is about work done
*under* it, which is a different claim from the one cycle 9 exonerated. Not on a feature for GBS
that stresses no invariant, unless §6.4's roadmap says it is tier 1.

---

## 8. Potential misses in every cycle so far — extended

Ten cycles have found that the highest-value defects were **correct on every input and wrong in
structure**. Cycle 10 added a new family: **the harness caught what the participants wrote.** Look
for the class, not the instance.

- **Two counters read as one.** `flights_that_fell_behind` (view outran the fold) and
  `pinned_installs` (landed behind at all) were conflated into "the merge window is empty by
  construction"; the report carries the dated correction. Every counter pair in `Stats` and
  `ReadStats` that shares a noun is a candidate: `hits`/`view_answers`, `misses`/`fallbacks`,
  `gap_at_begin`/`gap_at_finish`, `uninstalled_folds`/`flights_refused`.
- **Instruments never asserted non-zero.** `flights_that_fell_behind` read 0 across 28 replicates
  on two hosts and *nothing in the tree could distinguish that from dead code* until this cycle.
  For every counter on the wire, is there a test that makes it move?
- **Refusals that do not refuse** (MF-13, the paired adapter gate, `make reproduce`'s manifest):
  every `skip`, every `SKIPPED`, every `unwrap_or`, every default in a harness.
- **Probes that measure the wrong thing, place, or moment** (MF-7, MF-11, MF-14): every
  `rustc --version`, every `which`, every `pg_isready`, every `uname` in a script — from where,
  with what environment, before or after the thing it decides.
- **Edits that assert late and write late** (MF-12): the executor's Python edit method has no
  verification step. Is there a rule that would have caught it?
- **A plan not followed** (the branch collapse): what in the harness would have refused a commit
  landing on the wrong branch?
- **A mechanism built on a correct number that still lost** (T04): the 99.1% pinned figure was
  right and the mechanism was right and the result was a regression. What question, asked before
  building, would have predicted it? (The executor's own §6.2 in cycle 10 asked for the `e−a`
  distribution; it did not ask *where the merge's work would be done*.)
- **Results files that are host-shaped**: the `host-scoped` class now exists; sweep `results/` for
  anything else that is a fact about a machine — `E13-durability.md`, `E21-wire-cost.md`, the
  E16/E19 CSVs — and say whether each is classed right.
- **Generators that omit** (LC-40, A10-13): still unswept; T05 is below the cut.
- All of cycle 10's §9 — baselines inferred from documents, instruments that can only confirm,
  hazard classes overstated by a rung, environment facts recorded as tree facts, syncs given
  once, load-dependent preconditions, prose contradicting tables, sizes nobody has a number for,
  two spellings of one concept — **none swept to completion**.

---

## 9. Check these regardless

Both auditors, independently, with evidence class:

1. **The gate on both trees at the heads in §2.3** — ask the author for `bash ~/Documents/niles/docs/audit/cycle-10/hostc/c10-gates.sh` and paste; record every row; PostgreSQL's absence is an environment fact.
2. **Re-derive the T04.2 arithmetic** from the transcript — the six medians, six pooled MADs, and
   which rows clear which half of the gate — and say whether the arms were what the transcript
   says they were (`caps 32/4096` against `caps 0/0`, from the daemon's own line).
3. **Read `finish_fold_merging` and `merge_suffix` against the lock order** and say, from source,
   what is held while the suffix is walked, and what would be held if the walk moved before the
   second V acquisition.
4. **Read the dated correction in the execution report** and say whether the *original* paragraph
   should have been caught by either cycle-10 auditor from the counters' own definitions.
5. **The GBS tree on the Mac** — its head, and whether it builds against `653a369`.
6. **Every `MISMATCH` and `BLOCKED` marker in both trees**, in one table with status, and which
   have been repaired without their marker being struck. A10-06 repaired `MISMATCH-A9-F15`'s
   substance; say whether the marker still stands.
7. **The revert ledger** (§1.2): produce one each; the author reads them side by side.
8. **The model recommendation** (§1.5): produce one each, with the counts.

---

## 10. Instruments first — and audit the harness as a codebase

Before you audit the engine, audit the harness. `docs/audit/cycle-10/hostc/c10-baselock.sh` is
~1,000 lines with 24 self-test arms. It has caught more this cycle than any reader has. Ask of
every refusal in it: *what fault would it not catch?* Ask of every self-test arm: *is the injected
fault the fault that actually happened, or a neighbour of it?* (MF-5: the port arm passed on
Linux and failed on Darwin because the injected fault was a neighbour.) And ask whether the
harness should become a **crate** — tested by `cargo test`, run on both hosts, its refusals
first-class — rather than a shell script that has grown a test suite of its own. That is a
design decision for §1.3.

---

## 11. Constraints the executing agent is bound by — carry them into the work order

Non-negotiable; no task weakens one without an explicit decision routed to the author:

durable-before-visible; one sealer per ledger; one total epoch order; full retention; honest
absence (a `Hole` keeps its version, a cancelled flight restores the exact slot it replaced, a `⊥`
means "no cached assertion", never zero); fold-never-field; product purity; self-describing
amounts; GBS layering; **no `unsafe`**; **no default-on-error**; **zero external dependencies**;
no fabricated results; the thesis follows the code with `MISMATCH-<id>` at every divergence; a red
test is a result and is never fixed by deletion; honest refusal over silent fallback;
`BLOCKED-<id>` on ambiguity; apply-before-publish; the lock order **O < B < P < V < C**, S a leaf,
**F a leaf below V**; every optimisation ships its guard in the same commit and the executor
**proves the guard fails on the reverted change in a disposable `git worktree`** — a compile error
is never the witness; no performance claim without a reproducible command and a provenance
header; **a pass line is written against a measured baseline, never a document**; **benchmarks
touch committed artefacts only with `--publish`, never from a container**; a currency's wire code
is its declaration index; the idempotency window is counted in transactions; `Rev::read` and
`Rev::begin_read` answer at the anchor asked; a keyed read takes the base at most once; a joined
reader holds nothing while it waits; flights and waiters are bounded and a refusal is counted; an
install is owned by its generation; **the merge writes the view and never the answer**; every
refusal is counted under its own cause; **a refusal stops what it refuses**; every script probes
from where the thing it decides about lives; `deferred_merges` and every merge counter are
reported even at zero; the results manifest is the only classification and the diff derives from
it; **the branch plan in the work order is followed or the deviation is a `BLOCKED-<id>`**.

Environment: **no task may require network access at execution time; no task installs anything.**
Rust 1.95.0 (`RUSTUP_TOOLCHAIN=stable` where the pin does not resolve, `RUSTUP_AUTO_INSTALL=0`),
valgrind, `psql` and a startable PostgreSQL 16 (started explicitly) are preconditions. **Edits to
Rust source through Python are exact-string, single-occurrence replacements asserted before
writing, and every multi-edit script either writes each edit as it makes it or verifies afterwards
that every edit is present** (MF-12). Attribution is taken from the executing session's own
instructions; never bake in a trailer.

**Sync at every landing.** The executor has no remote and cannot push. It ships each commit as a
`git bundle` written to `~/Documents/niles-sync/cycle-11/` on the Mac (or to `~/Documents/`
directly through the bridge, as cycle 10 did), and tells the author — **at the moment it is needed,
and again at every landing until confirmed run** — exactly what to run, naming the ref the bundle
carries:

```
cd ~/Documents/niles
git bundle list-heads ~/Documents/niles-sync/cycle-11/<bundle>
git pull --ff-only ~/Documents/niles-sync/cycle-11/<bundle> <branch>
git push origin <branch>
git rev-parse --short HEAD
```

(`cd ~/Documents/GBS` for GBS.) For a branch that does not yet exist locally: `git fetch <bundle>
<branch>` then `git branch <branch> FETCH_HEAD`, never `--ff-only`. A lint the container's 1.95.0
cannot see is fixed from the author's pasted output, never guessed.

---

## 12. Open questions to carry, restate or close

Settled, **not to be reopened**: LC-01, 02, 04, 08–12, 14, 15, 17, 18, 21, 28. Decided in cycle 9:
LC-16, LC-35. Decided in cycle 10 by the author: LC-38 → build (option A of the cycle-10 decision);
the results manifest derives the diff; the baselock waiver reads the manifest.

**Reopened with a measured answer and an undecided consequence:** **LC-38** (§6.1) — merge or
pin, and the A/B/C decision the author has not made. **LC-37** — the base hold across the fold,
which every tail and now the merge's cost point at; T03 was below the cut in cycles 9 and 10.

Carried: **LC-03, 05, 06, 07, 13, 19, 20** (`BLOCKED-LC20-definition`, still the author's), **22,
25, 26, 27, 29, 30** (implemented and guarded; the author has not confirmed the protocol in so many
words — ask, and close it or not), **31, 32** (T07, re-baselined), **33, 34** (TG01, still the
first product), **36** (`BLOCKED-recovery-tip`, the author's contract), **39** (is
`pinned_installs / reads` a phase-diagram input? — it now has a *cost* attached, which changes the
question), **40** (T05), **41** (refused honestly at T02; the partition policy is the author's to
choose and no auditor may invent it), **42** (T10, design only).

New, raise and position — the author decides:

- **LC-43** — the merge's cost model: is the regression intrinsic to merging on the read path, or
  is it the cost of where the work is done? Decided by measurement (§6.1 A), not by argument.
- **LC-44** — the harness as a crate (§10): shell script with a self-test, or a tested Rust
  binary that both hosts run.
- **LC-45** — the branch plan as a constraint: does a deviation from the work order's stack need
  a `BLOCKED-<id>`, and is a single-branch cycle acceptable?
- **LC-46** — feature parity versus invariant coverage for GBS (§6.4): the author's stated goal
  and the admissibility rule disagree; one of them yields.
- **LC-47** — the pipeline's model assignment (§1.5), with the human's role stated.
- **LC-48** — reset or continue (§1.3), per component, with the cost of each.

---

## 13. What you must produce

A single work order, `docs/audit/cycle-11/astra-work-order.md`, containing:

0. **The gate output** for both trees at the heads in §2.3, labelled by host.
1. **§1 answered in full** — 1.0 the four fundamental questions with their refutation
   conditions and the strongest alternatives named, 1.1 the decision audit, 1.2 the revert ledger, 1.3 the plan-from-
   cycle-1 and its diff with reset costs, 1.4 the reachability judgement per artefact with the kill
   criteria' numbers, 1.5 the model recommendation with counts. This section comes first and it is
   allowed to be the longest.
2. **Findings**, each with a class (wrong-measurement, correctness, liveness, guarantee-bounded,
   instrument-gap, negative, stale-claim, **decision-refuted**), an **EV score**
   (`impact × confidence ÷ cost`, 1–5 each, ties to correctness), evidence class, file and line,
   measured cost, proposed repair, and **what the repair gives up**.
3. **A task list in dependency order** — each with what it closes, files, **baseline measured on
   the executing host or by a named script the author has run**, target as a sentence the executor
   can mark `done` or `not done: why`, method, acceptance test, **guard and the reversion that
   must make it fail**, guardrails; a **cut line** for one agent in one cycle; and **the branch
   each task lands on**, which the executor is now bound to (LC-45).
4. **Branch stacks and merge order** for both repositories from `c11/00-audit` (niles) /
   `01e1d85` (gbs), named `c11/*`.
5. **A validation protocol**: commands, what green means, what a red row means, which reds are
   environment.
6. **Every Host C script**, under `docs/audit/cycle-11/hostc/`, inheriting `c10-baselock.sh`'s
   refusals and self-test or saying why not, with its runtime and its "**the author must run this
   now**" marker where its result is needed. **The first script measures the baseline the first
   task is scored against.**
7. **The GBS roadmap** (§6.4) tiered by invariant, with the parity cost estimate.
8. **The open-questions ledger** (§12), updated, with LC-43…48 positioned.
9. **Reporting requirements** for the executing agent: the checklist with every target line
   verbatim; guard transcripts naming the disposable worktree with exit codes; timing tables with
   a host column; the MISMATCH/BLOCKED inventory; worktree status with the five untracked files'
   sizes; every SHA and bundle hash; **exactly three material facts the work order did not cover**;
   and a sync section repeated at every landing until confirmed.

Order tasks by `impact × confidence ÷ cost`, not by interest; but §1 is not ordered by EV — it is
ordered by the author's questions, and it is answered before anything is proposed. Weigh three
things: the stated goal is efficiency; the **binding constraint** is correctness; and **the
record now shows the harness catching more than the participants**, which is itself evidence
about where the next cycle's value lies. And of every instrument, ask whether it could ever have
said no — cycle 10's merge measurement could, and did.
