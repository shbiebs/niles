# Cycle 9 audit brief — **Astra**

**You are auditing, not building.** Your single deliverable is a **work order**: a prioritised,
evidence-backed set of tasks that Claude Opus will execute in a later session. You write no code
into either repository and you make no commits. You read, you compare claims against source, you
reason about designs, and where a number is needed that only a machine can produce, you ask the
author to run a named command and paste its output.

**You have no shell.** Everything you know about these repositories comes from reading them — on
GitHub, or as files the author attaches — and from output the author pastes back. Write every
request for a run as a complete, copy-pasteable block with the expected runtime, and say what you
will do with the result. If a request goes unanswered, say in your work order which conclusions
rest on it and are therefore provisional.

The work order you produce is the only thing the executing agent sees. Assume it has none of your
context: every task carries its own preconditions, baseline, target, method, acceptance test, guard,
and guardrails, and must be executable **offline**.

**A second auditor — Fable — is reading the same two repositories independently, from a companion
brief.** Fable has a container and measures; its brief leans empirical. Yours leans structural:
claims versus code, the type system and what it proves that the engine re-checks, the thesis text
against the runtime, GBS's product model against what Loan IQ's and Calypso's users would need.
The author reconciles the two work orders afterwards. Three consequences:

1. **Where you both look at the same thing and disagree, the disagreement is the valuable part.**
   State your evidence class on every finding — *read from source*, *inferred from a document*, or
   *measured by the author on request* — and never round toward the other auditor.
2. **Do not soften a finding because you expect Fable to catch it.**
3. **§8 lists things you must both check independently.** Do those even knowing they are duplicated.

Your predecessor's strength, in the one cycle it ran, was reading claims against code: it found a
guarantee asserted in the very file that randomises its input, a depth claim that contradicted
another, and checked facts that vanished before the wire. Cycle 8 found two more of that class
without you — a lattice state the theory names and the engine never enters, and a recovery that
returned a prefix with a success code. **That class is what this brief is built around.**

Everything in `docs/audit/cycle-8/` — the work order, the execution report, the LC-23 attribution —
was written by Claude sessions. Treat it as a set of claims to verify from the source, not as
facts to build on.

---

## 1. The three goals, and the question each auditor answers

The stated end goals are **a highly efficient query language (Niles)**, **a highly efficient query
engine (Nilestream)**, and **a highly efficient core-banking system (GBS)** — efficient in **memory
and speed**, poised to replace systems of the class of **Finastra Loan IQ** (commercial and
syndicated lending) and **Nasdaq Calypso** (capital markets: trading, risk, collateral, treasury,
clearing, post-trade) — without surrendering any commitment in §10. The thesis's claim is a theory;
the two systems are its instrument; the banking constructs are a domain layer on a fully general
relational core.

Cycle 8 answered the three framing questions with numbers and each answer opens the next one:

- **Nilestream.** The next ceiling was implementation again — view metadata went from 159 to 94 B
  per resident key and both idempotency indexes are bounded by a declared window — and then the
  attribution of the 12–13 ms read tail found the largest ceiling this project has measured:
  **read throughput falls as readers are added** (94,038/s at six connections, 48,310 at nine,
  20,507 at twelve; p50 37 → 176 → 596 µs) because a reconstruction runs while its reader holds
  the view mutex, and the lattice state that would let it run outside the lock — `Slot::Pending` —
  is constructed nowhere in the runtime. **Is the repair the design the thesis describes, and does
  it preserve the certification interval?** That is question one, and it is not a tuning question.
- **Niles.** The compiler has been measured once: 243 µs for a 158-line program, the IR verifier
  2.4% of it, nothing in the front end on the serving path — and the plan cache that keeps it off
  the serving path emptied itself to admit one statement until cycle 8 replaced that with FIFO.
  **What does a statement cost in the engine that the compiler already knew?** Where the type
  system proves a fact — single currency, conservation, linearity — and the runtime re-establishes
  it, the language is paying twice. Nobody has swept for that. That is question two.
- **GBS.** Still inadmissible for an efficiency claim: no product trace (F-24), lifecycles that do
  not survive a restart (F-12), six of twenty-nine product rows on the generic path only. Cycle 8
  made the one guarantee it had a column instead of a sentence. **What would Loan IQ's and
  Calypso's users need that GBS cannot yet express — and which of those needs is a new invariant
  for the ledger rather than a feature?** That is question three, and §6.5 gives you the material.

---

## 2. Access

### 2.1 GitHub

Both repositories are private, owned by **`shbiebs`**:

- `https://github.com/shbiebs/niles`
- `https://github.com/shbiebs/gbs`

If you do not have access, ask the author for the narrowest thing that works: **read access only**,
to those two repositories and nothing else. On GitHub that is a fine-grained personal access token
with *Only select repositories* → `niles`, `gbs`, and *Contents: Read-only*. The author supplies it
out of band; you never write it into an output, and you never push. If access is refused or the
repository reads as not found, say so in your output and audit what you can reach — the author can
also paste or attach individual files.

The heads to audit are in §4. Confirm them before reading anything else.

### 2.2 The author's Mac

You are authorised to audit the working copies at `/Users/checolino/Documents/niles` and
`/Users/checolino/Documents/GBS`. Both are at the heads in §4 and clean apart from five untracked
files that are the author's and must never be edited, staged, deleted, moved or bundled:
`.DS_Store`, `AGENTS.md`, `niles/.DS_Store`, `thesis/.DS_Store`, `thesis/Niles-Thesis.pdf` (the
fifth, `niles/.DS_Store`, appeared in cycle 8 and is reported rather than handled). Report their
sizes at the end. **Read-only discipline**: no commits, no staging, no `git clean`, no checkout of
another branch without saying so first.

Everything that needs the real machine — a build, a benchmark, a storage probe, `psql` — is
packaged as a shell script for the author to run in Terminal.app. The convention is
`~/Documents/niles-hostc/<name>.sh`; `run4.sh` through `run6.sh` exist and their results are under
`~/Documents/niles-hostc/results<n>-<stamp>/`. Say **explicitly, at the point in your work order
where it is needed**: "the author must now run `bash ~/Documents/niles-hostc/<name>.sh` and paste
the output." Every script is idempotent, non-interactive, bounded in time with the runtime stated,
never assumes `chmod +x`, and — **the rule cycle 7 and cycle 8 each learned once** — retargets
every worktree it measures to an explicit SHA and refuses to measure if the retarget did not take
or the worktree is dirty. `run6.sh` is the current template: it also starts its own `nilestreamd`
per section, writes non-published output outside the worktree, and checks for PostgreSQL once, up
front, with the commands to start one.

The author's zsh has `interactive_comments` **off**: a `#` at the prompt is a command. No comments
in pasted lines. `--ff-only` moves the branch that is checked out, so a `c9/*` name has to be
created with `git branch -f` at the merged commit before it can be pushed — every sync block in
§12 already says so.

**Bridge traps, if you reach the Mac through the Claude desktop bridge.** You get a Linux VM
over FUSE, not macOS: no `rustc`, `cargo`, `psql` or `valgrind`; fine for reading, useless for
measuring. Any `git` command that touches the index leaves a `.git/index.lock` the bridge cannot
delete — use `git --no-optional-locks` for reads, and if one appears, tell the author to `rm
~/Documents/niles/.git/index.lock`. `git -C` on a worktree fails through the bridge because
worktree metadata records `/Users/...` paths the mount does not have; read worktrees from the
main tree's `git worktree list` instead, and expect them to show as "prunable" — that is the
bridge's view, not the Mac's.

The real Mac is **Host C, the reference host**: Apple M4, 10 cores (4P + 6E), APFS on NVMe, every
barrier `F_FULLFSYNC` at ~255/s, Rust 1.95.0 pinned with 1.97.1 also installed, PostgreSQL 16
installed **but not running on 127.0.0.1:5433** — which is what still blocks `run6.sh` sections B
and C, and therefore T-15.3 and the E16/E19 republish. The only machine in this project that gives
the same answer twice.

---

## 3. Hosts, and what each may conclude

| host | may conclude | may not |
|---|---|---|
| **C** (the Mac) | absolute wall-clock, durability, scaling to 10 cores, contract verdicts, anything published | — |
| **a cloud container** (2 cores, shared Xeon, real `fdatasync` at a rate that varied 1.8–2.1× between instances in cycle 8) | deterministic counters (allocations, instructions where deterministic, visited entries, `max_batch`, txns/fsync ratios, compiler verdicts), within-container ratios, tests, guards, mutation runs | any absolute figure beside another host's; any durability row; any curve past 2 cores; anything published |
| **the desktop bridge** (a Linux VM over FUSE, no toolchain) | reading, grepping, counting | building, measuring, anything with `git` that writes an index |

Rules the table decides: no absolute wall-clock, throughput or fsync figure beside another host's
without a host column; deterministic counters gate first and need one run; wall clock needs 2
warm-ups and ≥5 measured runs, arms interleaved, median/MAD/range, and a gate fires only on ≥10%
*and* ≥3× the pooled MAD, else "noise-limited"; `unsupported`, `blocked`, `not run` and
`noise-limited` are results and omission is not. **A `callgrind` whole-process instruction count is
not deterministic at compiler granularity** (three runs of one binary over one file: 2,402,561 /
2,410,322 / 2,409,454 I-refs) — count what the program produces, not what the CPU executed.

---

## 4. Exactly where the trees stand

| repository | head | branch(es) | tests |
|---|---|---|---|
| `niles` | **`5eb0bb2`** — "T-12.3: LC-23 answered — the tail is the view mutex, and the lattice already has the repair" | `c7/01-durable-rows` = `c8/08-run6-fixes`, both on GitHub | 866 `#[test]` attributes; 991 assertions pass |
| `gbs` | **`963e4d9`** — "T-G1: G3 states the as-of counter instead of describing it — closes F-60" | `c7/00-adapter` = `c8/01-idem-window-epochs`, both on GitHub | 560 attributes; 504 pass |

Cycle 8's commits on `niles`, oldest first: `c2617f6` (the audit brief and `run6.sh`) · `c0fa77a`,
`34d0066`, `2792670` (T-05) · `18b5f49` (T-06) · `4a8f63d` (T-11) · `25a5de2` (T-12) · `9ee43e7`
(T-13) · `0338302` (T-14) · `7f9f30b` (T-15) · `90fc5fe` (the execution report) · `d8ad061` (the
`run6.sh` and publish-gate fixes) · `5eb0bb2` (LC-23). On `gbs`: `f9c2d66` (T-05) · `963e4d9`
(T-G1). Read `docs/audit/cycle-8/execution-report.md` and `lc-23-attribution.md` before anything
else: they are the executor's own account of what was done, what was not, and what was found that
the work order had not asked for.

Gate, both trees, at these heads: `cargo fmt --all -- --check`, `cargo clippy --offline
--all-targets -- -D warnings` (1.95.0 in the container, 1.97.1 on the Mac), `cargo test --offline
--workspace`, `make reproduce` (exit 0, clean diff), `make fsync-proof` — all green. A red row is a
result; never fix it by deletion.

---

## 5. What cycle 8 changed, and what it found that it was not asked about

Fourteen findings closed (F-43, F-44, F-49's instrument half, F-50, F-52–F-60) and one target
line structurally not done (T-15.1, two arms in one process). The execution report's §5 names three
facts the work order had not covered, and §11 the attribution that overturned the work order's own
hypothesis:

1. **The plan cache emptied itself** to admit one statement over its 256-plan limit: 1,000 distinct
   statements executed twice cost 2,000 compilations. Now FIFO. **Its hit rate has never been
   measured**, and neither has the size of one cached `Lowered` — 256 of them per session, times
   the connection count, is a memory term nobody has a number for.
2. **Recovery silently dropped acknowledged transactions** from a batch envelope that ended early:
   both decoders `break` on a short read and returned the prefix with a success code. Now refused,
   naming the record. **The recovery path has still never been fuzzed**: one truncation was tried,
   at one offset, by hand.
3. **Two tests passed or failed according to how busy the machine was** — one from cycle 7, one
   written in cycle 8 — both because their preconditions were fixed counts instead of properties
   of the run. The class is *load-dependent preconditions*, and nobody has swept the suite for it.
4. **A run without `--publish` wrote committed CSVs** (the default `--out` mapped to the committed
   directory whatever the run was doing). Gated now; the E16 documents were already gated, so the
   rule "committed artefacts only with `--publish`" was true of the documents and not their data.
5. **`run6.sh` failed twice before it ran**: it inherited a `bench` invocation that assumed a
   running daemon that a probe crate had previously spawned, and a `grep` made the empty section
   look like a section that ran. Both are the class "the audit's instrument was wrong in the same
   way as the code", and both were found only by running the script on the reference host.
6. **The tail (LC-23)**: view mutex first, base guard second, scheduler and wire not at all —
   `unaccounted_us` 0–2 µs in all 160 slowest reads. Read throughput *falls* with readers. The
   mechanism is `Rev::read` reconstructing under the view mutex; the repair is the lattice's own
   `Pending` state, never constructed. **`MISMATCH-pending-unreachable`** is marked in chapter 3.

Two things the executor wrote down as *given up*, which an auditor should weigh rather than accept:
a key evicted and read again restarts its `CostAware` count at one (T-05.2), and a retry older than
the declared window commits twice (T-05.3, by the language's own semantics; LC-28 decided in
epochs).

---

## 6. Where your budget should go

### 6.1 The view mutex, and the lattice's fourth state — the first task of cycle 9

`docs/audit/cycle-8/lc-23-attribution.md` is the evidence. What the executor did *not* do — on
purpose — is design the repair, because it touches the certification interval. Your work order must:

- specify `Slot::Pending(anchor)` end to end: mark, release the view, fold the base, re-acquire,
  install; what a second reader at the *same* key and anchor does (join); at a *different* anchor
  (its own reconstruction — say why, and what it costs); what `advance` does on meeting a `Pending`
  entry (skip, as for a hole — but state that the reconstruction is anchored and cannot be
  invalidated by an epoch it does not include); and how `install` after the fold interacts with
  `pinned` when `applied` moved during the fold;
- state the invariant the change may not weaken — every answer is exact at the anchor it was asked
  for, over `[stamp, effective]` — and the guard: the concurrent differential
  (`every_answer_matches_an_independent_fold_at_its_own_anchor`) *with a reconstruction long enough
  to overlap an `advance`*, which today it is not;
- give the baseline as the measured curve (94,038 / 48,310 / 20,507 reads/s at 6/9/12 connections,
  worst view wait 22,160 µs) and the target as a ratio at twelve connections, measured by `run6.sh
  --only D` on C before and after;
- decide, and say why, whether the base guard held across `advance` (3.3 ms on the worst 9r/5w
  read) is touched in the same task or left for the next measurement.

### 6.2 Two claims about partial state that the runtime does not use

**Honest absence carries information nothing reads.** Chapter 3: "eviction maps Present(v, e) to
Hole(e), never to ⊥ — absence of *value* never masquerades as absence of *history*", and Hole(e)
"knows from `e` exactly which prefix to fold". In `nilestream-core/src/rev.rs`, the runtime never
calls `.version()` outside its own tests; `Rev::read` folds `base.reconstruct(key, anchor)` from
the *anchor*, and `apply_epoch` treats a hole and ⊥ identically. So a hole's epoch is dead
information, and the slot map's retention of holes — ~112 B per key ever read, the last
Θ(history) term left in the view after T-05 — buys nothing today. Decide which of three things is
true and make the thesis say it: (a) reconstruction should use the version and does not; (b)
honest absence is an *audit* property (which epoch an entry was certified through when it left),
not the safety property the chapter calls it, since ⊥ also reconstructs and "miss ≠ zero" is
satisfied either way; (c) holes may be compacted to ⊥ under full retention with no loss, and the
view's memory becomes Θ(budget) outright. G3 wipes the whole view to ⊥ before every sweep and
conservation holds, which is evidence for (b) and (c).

**The served daemon does not use SC7.** `RevEngine::seeded` builds `Ledger::new()`, whose
`checkpoint_interval` is 0. E9 and E11 established that per-key checkpoints are what make
reconstruction cost "a function of a chosen interval instead of a function of the log's age", and
H-S3 claims rung cost is workload-shaped rather than history-shaped. Every E16, E19 and mixed figure
this project has published was measured from a daemon that folds a key's *entire* history on every
miss — and the within-level decay in cycle 8's data (94,038 → 84,384 reads/s over five runs as the
base grew) is the direct symptom. Establish whether §9 states which configuration its numbers came
from; specify the flag, the default, and the re-measurement; and say what the checkpoints cost in
memory per key, since T-05 just spent a cycle bounding exactly that kind of term.

### 6.3 Niles: what the compiler knew that the engine re-establishes

Cycle 8 put the compiler on the bench; nobody has yet asked what a *checked fact* is worth at run
time. Sweep the path source → resolve → typecheck → lower → `niles-ir` → verifier → `rev_engine` →
`session` for facts proved once and checked again: currency agreement on a statically
single-currency aggregate; the conservation obligation on a `txn` the checker discharged; a
linearity proof that should let a value move rather than clone (`Key` is a `Vec<i64>` cloned on
every read — T-05 halved that from two clones to one and stopped); scale per currency, declared in
the schema and re-parsed per `INSERT`. For each: where the fact is established, where it is
re-established, what the second check costs in an E18 row, and whether removing it would weaken a
refusal the wire relies on (SQLSTATEs 22023, 22000, 22003, 0A000 are contracts). This is the
language's efficiency question and it has no instrument yet.

Also on the language: the transaction-level `idem("k", window: 24.hours)` form is parsed and
**dropped** — no diagnostic, no lowering — while the column form is checked (NL0215) and its unit
enforced (NL0217). Two spellings of one concept with different rigor is a defect in the calculus,
not a style issue. And the plan cache: measure bytes per `Lowered`, the hit rate on E16's workload,
and whether 256 × sessions is a term the memory model must carry.

### 6.4 Durability: the recovery path has one truncation test

Cycle 8's T-14.3 tried one short envelope at one offset and found a silent drop. The path now
refuses that case. It has not been shown to refuse *every* case: specify a torn-segment corpus —
truncation at every byte offset of a small segment, single-bit alterations in header, hash and
payload, a duplicated record, a reordered pair — with one property: **refuse, or recover exactly
what was acknowledged; never a prefix with a success code.** Deterministic, offline, one run.
Together with T-14.2 (concurrent `submit_pending`) this is what makes the crash-protocol
integration test one witness among several rather than the only one.

### 6.5 GBS: Loan IQ and Calypso, as invariants rather than features

Research done for this brief (vendor material, September 2026; sources in §13) puts the two systems'
functional scope as follows. **Loan IQ** (Finastra): commercial, bilateral and syndicated lending on
one platform — deal → facility → outstanding (loan, letter of credit) with borrower, lender-share and
agent roles; "comprehensive agency servicing" for deals of "a few to over a thousand lenders";
drawdowns, repayment schedules, interest and fee accrual ("fee types & characteristics", "discount
loans", "tiered matrix" pricing by dates, balances and ratings); "lending operations — trades" (the
secondary market: assignments, participations, settlement); letters of credit, guarantees, export
finance, ABL, CRE; collateral with "cross collateralization", covenant monitoring and document
tracking; "adjustments, amendments and reversals"; "online accounting with real-time debits and
credits", multi-branch GL/sub-ledger mapping, "extensive audit trail"; PIK, unitranche and
non-pro-rata; sustainability-linked lending. **Calypso** (Nasdaq): multi-asset trade capture and
pricing over shared curves and market data — rates, FX, credit, equity, fixed income, repo and
securities financing, commodities; risk — market, counterparty credit exposure, limits, P&L, XVA;
collateral — margin calculation, CSA-style agreements, initial and variation margin, optimisation,
ISDA SIMM; clearing — CCP margin methodologies (SPAN2, PRISMA, IRM2), house and client account
structures (ISA, OSA, NOSA, GOSA), position limits; post-trade STP — confirmation, matching,
settlement, corporate actions, lifecycle events; treasury — cash positions, funding, liquidity
across banking, trading and investment books; regulatory reporting (EMIR, Dodd-Frank, MiFID II,
FRTB, SA-CCR); a "unified trade record".

**The admissibility rule for anything you derive from that list.** GBS's product rows exist to
exercise the ledger's invariants under real product shapes, not to reach feature parity. A product
task is admissible in cycle 9 only if it does at least one of: (i) exercises a conservation or
certification invariant in a way no existing row does; (ii) closes F-24 (a product trace the
engine benchmark can be shaped by) or F-12 (lifecycles that survive a restart); (iii) moves a row
from `GenericPathOnly` to `ProductSpecific` with G3 evidence. Map each capability above onto GBS's
29 rows and classify it: **present with G3 evidence / generic path only / absent**, and for the
absent ones say which invariant it would stress. The ones this brief expects to matter most:

- **Pro-rata distribution to a syndicate of a thousand lender shares**: one payment, a thousand
  postings, one conservation obligation — the largest single-epoch batch GBS would ever seal, and
  a direct test of the batch envelope, the sealer's drain, and E23's cost-per-row-of-answer.
- **Adjustments, amendments and reversals on an immutable ledger**: a reversal is a compensating
  posting, an amendment is a new valid-time interval, and neither may rewrite history. Which of
  Loan IQ's "adjustments, amendments and reversals" have a bitemporal spelling in Niles today, and
  which cannot be expressed?
- **PIK capitalisation and accrual**: interest that becomes principal is a posting the *schedule*
  creates, not a client — the first product where the ledger writes to itself. Does the language
  have a spelling for a scheduled, deterministic, epoch-anchored posting?
- **Letters of credit as contingent liabilities**: off-balance until drawn; a hold with an expiry is
  the nearest primitive. Is a hold the right primitive, or does an LC need a third state?
- **Margin and collateral (Calypso)**: a variation-margin call is a conditional hold whose amount
  is a *function of a price* — a derived view over market data — settled by a posting. That is a
  REV whose input is not the ledger, and the first place the "fully general relational core" claim
  is tested against a non-ledger base.
- **CCP netting and account structures**: house and client segregation is a partition of the
  ledger's key space with cross-partition conservation forbidden by rule — a `conserve per (…)`
  clause the language may or may not be able to state.
- **Multi-book treasury (banking, trading, investment) and FX with real-time debits and credits**:
  GBS's FX row exists; what it does not have is a *book* dimension.

Do not propose more than one product task above the cut line unless it is also (ii). The refusal
"no GBS efficiency task is admissible until F-24 and F-12 close" stands until an auditor shows
otherwise with evidence.

### 6.6 What only you can do this cycle: read what a measurement cannot

Fable will measure. You read, and three readings are yours in particular:

- **Chapter 3 and Chapter 4 against `rev.rs`, `absence.rs` and `rev_engine.rs`, line by line**, for
  every sentence that states what the runtime does — the lattice, monotone anchoring, honest
  absence, the certification interval, the inheritance rule for `effective`, `Pending`. Each
  sentence is true, false, or true of a component and false of the system; say which, with the
  line. Cycle 7 found the durability sentence was the third kind; cycle 8 found the lattice's
  fourth state was the second.
- **The type system's obligations against the wire's refusals.** For each SQLSTATE the server
  emits (22023, 22000, 22003, 0A000), the checker either proved the condition cannot arise for a
  compiled program or it did not; for each obligation the checker discharges statically, the wire
  either re-checks it or it does not. Build the four-cell table. The cells "proved and re-checked"
  are §6.3's cost; the cells "not proved and not checked" are a defect.
- **GBS's twenty-nine rows against §6.5's capability lists**, as a classification table with one
  line of reasoning per row, and the invariant each absent capability would stress. This is the
  one deliverable of the cycle that needs a domain reading more than a machine.

### 6.7 Below cycle 8's cut line, carried

**T-07** cross-architecture determinism for both SHA-256 streams (`chain`, `write_rows`) — needs C.
**T-08 / LC-24** the fold plateau and the unstable p99 at 16 connections — re-read against LC-23's
attribution before designing anything; the same serialisation is the likeliest cause. **T-09**
E24 RSS under load on C. **T-10** GBS lifecycles and holds on the ledger (F-12). **T-15.1** two
arms in one process — the design (`--baseline-port`, a second target name through the sample
stream, a two-column contract table) is in `docs/BENCHMARK.md`. **T-15.3** the E16/E19 republish on
C, blocked on PostgreSQL 16 at 127.0.0.1:5433.

---

## 7. Where *not* to spend the cycle

Not on compile time, startup or binary size as priorities (measured; refuted twice; measured again
in cycle 8 and found sub-millisecond). Not on a distributed protocol, consensus or cross-shard
commit. Not on the wire's binary formats beyond LC-22. Not on E2EE. Not on re-litigating LC-21
(fail-stop) or LC-28 (epochs). Not on `unsafe` — there is none outside the measurement tool and a
test asserts it. Not on a feature for GBS that stresses no invariant.

---

## 8. Check these regardless

Both auditors, independently, and state your evidence class for each:

1. **Have the gate run for you.** Ask the author, in one block, for both trees at the heads in
   §4: `cargo fmt --all -- --check`, `cargo +1.97.1 clippy --all-targets -- -D warnings`, `cargo
   test --workspace`, and — three times — `cargo test --workspace -- --test-threads=8`, with the
   result lines pasted back. Record any test that changes verdict between runs: the load-dependent
   class from §5.3 has been swept exactly twice, and reading cannot find it.
2. **Read `Rev::read` against chapter 3 and Theorem 4.1** with §6.2 in hand, and say whether the
   theorem's step (3) is stated for the four-state lattice or the three the runtime has.
3. **Re-derive the E18 apportionment yourself** from `tools/memprobe/src/scenarios.rs` — the rows
   T-05 added (`rev_metadata_per_key`, `rev_metadata_2x_budget`, `idem_admission_index`,
   `idem_window_sealer`) — and say whether each measures what its name says.
4. **The idempotency window's default.** A schema that declares no window keeps every identity and
   the daemon says so in its banner. Decide whether that should be a refusal, beside LC-16.
5. **Read the three `MISMATCH` markers still open** — `e16-header`, `pending-unreachable`, and
   whatever §6.2 produces — and say what closes each.

---

## 9. Potential misses in every cycle so far

Eight cycles have found that the highest-value defects were **correct on every input and wrong in
structure**, invisible to a test suite that never asked: a verdict rule that could not tell
"refused" from "did not run"; group commit structurally unreachable; a `&mut` for a counter; a
deadlock no test could catch; a read reporting the wrong end of its own interval; a plan cache
that emptied itself; a recovery that returned a prefix with a success code; a lattice state that
exists in the theory and nowhere in the engine. **Look for the class, not the instance.** Candidates
no cycle has swept:

- **Load-dependent preconditions** across the whole suite (§5.3): any assertion on a count that the
  scheduler decides.
- **Prose beside a table that could contradict it**: T-G1 found one ("should read zero"); every
  generated document has sentences the generator did not compute. Enumerate them.
- **Things measured from one configuration and stated for the design** (§6.2's checkpoints): E13,
  E16, E19, E23, the mixed rows — for each, which flags the daemon ran with, and whether the thesis
  says.
- **Instruments that reset, or do not**: `SLOW_READS` was cumulative for a whole run and made ten
  tables of which six were identical. `ENGINE_LOCK` and `VIEW_LOCK` are also process-global and
  never reset — every histogram `run6.sh` prints is cumulative across levels.
- **Sizes nobody has a number for**: one `Lowered` in the plan cache; one `Rev` slot map of holes
  at 100k keys read; one session's total state × 16 connections; the sealer's window at its declared
  1,000,000 epochs (≈ 100 MB at 99.6 B each — is that the intended default?).
- **The two spellings of one concept** (§6.3, `idem` column vs `idem(...)` call) — and any other
  concept the language spells twice: `expires:` on a hold versus `window` on a key; `conserve per`
  versus the typechecker's obligation; `retain forever` versus the window.
- **What the audit's own instruments assumed**: cycle 8's `run6.sh` assumed a running daemon, a
  PostgreSQL, and a publish gate that turned out not to exist for CSVs. `preflight.sh` now measures
  its tree instead of describing it. Nothing has audited `run4.sh`'s mixed probe crate — which is a
  second, hand-written harness whose numbers were quoted for two cycles.

---

## 10. Constraints the executing agent is bound by — carry them into the work order

Non-negotiable; no task weakens one without an explicit decision routed to the author:

durable-before-visible; one sealer per ledger; one total epoch order; full retention; honest
absence (as chapter 3 will state it after §6.2); fold-never-field; product purity; self-describing
amounts; GBS layering; **no `unsafe`**; **no default-on-error**; **zero external dependencies**; no
fabricated results; the thesis follows the code with `MISMATCH-<id>` at every divergence; a red
test is a result and is never fixed by deletion; honest refusal over silent fallback;
`BLOCKED-<id>` on ambiguity; apply-before-publish; the lock order **O < B < P < V < C**, S a leaf;
every optimisation ships its guard in the same commit and the executor **proves the guard fails on
the reverted change in a disposable `git worktree`** — a sibling directory for GBS, whose manifest
reaches `../../../niles` — transcript in the report; no performance claim without a reproducible
command and a provenance header; **benchmarks touch committed artefacts only with `--publish`, never
from a container — CSVs included, since cycle 8**; micro-benchmark gains are never contract
results; a currency's wire code is its declaration index; the idempotency window is counted in
epochs; `Rev::read` answers at the anchor it was asked for — no caller may reintroduce a
compensating branch.

Environment: **no task may require network access at execution time; no task installs anything.**
Rust 1.95.0 (`RUSTUP_TOOLCHAIN=stable` where the pin cannot resolve), valgrind 3.22, `psql` and a
startable PostgreSQL 16 are preconditions. Edits to Rust source through Python must be line-based —
a `\`-continued heredoc collapsed string literals three times in cycle 7. Attribution is taken from
the executing session's own instructions; never bake in a trailer.

**Sync at the end, and at every landing.** The executor has no remote and cannot push. It ships
each commit as a `git bundle` written to `~/Documents/niles-sync/cycle-9/` on the Mac, and tells the
author — **at the moment it is needed, not at the end** — exactly what to run:

```
cd ~/Documents/niles
git fetch ~/Documents/niles-sync/cycle-9/<bundle> c9/<branch>
git merge --ff-only FETCH_HEAD
git branch -f c9/<branch> HEAD
git push origin c9/<branch> c7/01-durable-rows
cargo +1.97.1 clippy --all-targets -- -D warnings
```

(`c7/00-adapter` for GBS.) A lint the container's 1.95.0 cannot see is fixed from the author's
pasted output, never guessed.

---

## 11. Open questions to carry, restate or close

Settled, **not to be reopened**: LC-01, LC-02, LC-04, LC-08 to LC-12, LC-14, LC-15, LC-17, LC-18,
LC-21 (fail-stop), LC-28 (epochs).

Carried, restate or close with evidence: **LC-03** `AccountId` String→u64 (T-05's key packing is
the same decision at the view); **LC-05** rendered-NULL and refusal boundaries; **LC-06** the
allocator/process boundary; **LC-07** E23's slope, never re-fit; **LC-13** release checks;
**LC-16** durable by default — now decidable, and §8.4 sits beside it; **LC-19** the wire claim's
three clauses, now all true, to be folded into one sentence in chapter 6; **LC-20** not decided;
**LC-22** declared scale on the wire, deferred to this cycle with the transcript diff shown first;
**LC-24** the unstable p99 at 16 (see T-08); **LC-25** concurrent crash test — position was no,
T-14.2 covers it in-process; **LC-26** the A/B script exists, the in-process form does not
(T-15.1); **LC-27** E19's CSVs are marked `not_run` until republished; **LC-29** oltp's verdict
concurrency belongs in SPEC-ENGINE Part 0.

**Answered in cycle 8, to be closed formally:** **LC-23** — the view mutex; the repair is §6.1.

New, raise and position — the author decides:

- **LC-30** — `Slot::Pending`'s join semantics: same key, different anchor.
- **LC-31** — whether a hole is an audit record, a cost hint, or compactable (§6.2).
- **LC-32** — the served daemon's checkpoint interval: what default, and does it become a schema
  declaration like the window did.
- **LC-33** — the plan cache's policy and limit, decided by a measured hit rate.
- **LC-34** — which of Loan IQ's and Calypso's shapes is the first product task, by the rule in §6.5.
- **LC-35** — whether an undeclared idempotency window is a refusal.

---

## 12. What you must produce

A single work order containing:

0. **The preflight output, verbatim, with its admissibility table filled in.** You cannot run it;
   ask the author, first, for `bash ~/Documents/niles/docs/audit/cycle-9/preflight.sh
   ~/Documents/niles ~/Documents/GBS` on the Mac, and paste what comes back at the top of your
   work order. Fill in the table from it and say which rows you inferred.
1. **An executive judgement** — where the three artefacts stand against the three goals, with the
   arithmetic for any reachability claim, and an explicit answer to each of §1's three questions.
2. **Findings**, each with a class (wrong-measurement, correctness, liveness, guarantee-bounded,
   instrument-gap, negative, stale-claim), an **EV score** (`impact × confidence ÷ cost`, each
   1–5, ties to correctness), **HI** or **HS**, evidence with file and line, the measured cost, the
   proposed repair, and **what the repair gives up**. Negative findings are first-class.
3. **A task list in dependency order** — each task carrying what it closes, its files, its
   **baseline measured on the executing host** (or the C figure it inherits), its **target as a
   ratio or a sentence**, its method, its acceptance test, its **guard and the reversion that must
   make it fail**, and its guardrails. Mark a **cut line** for one agent in one cycle. **Every
   target line must be a sentence the executor can mark `done` or `not done: why`** — the
   reporting checklist is copied from it verbatim.
4. **Branch stacks and merge order** for both repositories from `5eb0bb2` / `963e4d9`, named `c9/*`.
5. **A validation protocol**: commands, what green means, what a red row means.
6. **Every Host C script**, named, with its expected runtime, its retarget-and-refuse clause, its
   own daemon and its PostgreSQL check where needed, and an explicit "**the author must run this
   now**" marker where its result is needed. You may specify a script in prose for the executor to
   write; `docs/audit/cycle-8/run6.sh` is the template it will follow.
7. **The open-questions ledger** (§11), updated.
8. **Reporting requirements** for the executing agent: the checklist with every target line
   verbatim and its status; guard transcripts naming the disposable worktree; worktree status
   separating the five untracked files from task changes; the SHAs; **exactly three material facts
   found while executing that the work order did not cover**, negative findings included; and a
   sync section listing every bundle and the author's commands.

Order by `impact × confidence ÷ cost`, not by interest. And weigh three things: the stated goal is
efficiency; the **binding constraint** is correctness — a liveness bug outranks a factor of two, and
a durability claim that is not true outranks both. The highest-value defects have been invisible to
the suite and correct on every input: look for the class. And **audit your instruments before you
audit the code** — every cycle so far has had one that was wrong in the same way the code was.

---

## 13. Sources for §6.5

Vendor material read for this brief, September 2026: Finastra's Loan IQ solution page and brochure
(`finastra.com/lending/solutions/loan-iq`, the 2023 brochure PDF, the Loan IQ Academy role-based
training infographic listing the functional topics quoted above, and Luxoft's partner overview);
Nasdaq's Calypso solution page and its clearing page (`nasdaq.com/solutions/fintech/nasdaq-calypso`,
`nasdaq.com/products/fintech/calypso/clearing`), and Quinnox's platform guide. None of these is a
specification; each is a vendor's description of scope, and the capability list is to be read as
"what a buyer of these systems expects to exist", not as a requirements document.
