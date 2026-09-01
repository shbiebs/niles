# HANDOFF — Critical review and execution blueprint

**Audience.** An execution model (Opus) working autonomously in two repositories:
`niles/` (language, engine, thesis) and `gbs/` (Global Banking System). Everything below was
written after reading the code, not the summaries of it; every finding cites a file and line
that exists at the commits named here, and every task names the file it changes.

**Commits reviewed.** `niles` at `6039913` (430 tests), `gbs` at `9c4a8b8` (258 tests + 6 in
the excluded adapter crate).

**The three final goals**, restated so that each task can be traced to one of them:

| Goal | Where it lives | What "done" means here |
|---|---|---|
| G1 — an ideal, highly efficient query language | `niles/crates/niles-lang`, `niles-ir`, `niles-interp`, `bootstrap/` | `docs/SPEC-LANGUAGE.md` L-1..L-24 all **Built**, each with its acceptance test passing |
| G2 — an ideal query engine | `niles/crates/nilestream-*` | `docs/SPEC-ENGINE.md` Part 0 targets **measured against PostgreSQL**, not asserted |
| G3 — a core banking system | `gbs/` | 29/29 product lines, running on Nilestream with durability on, conservation suite green under eviction |

---

## Part 1 — Critical review

Each finding has a **class** (Logic = gap or edge case in the findings; Bottleneck =
architectural limit in a proposed solution; Claim-gap = deliverable does not satisfy what
the thesis says), **evidence** (file:line, verified), a **severity**, and the task that
resolves it.

### F1 · Claim-gap · The performance contract has never been measured against its baseline

**Evidence.** `docs/SPEC-ENGINE.md` Part 0 commits to 5–10× on OLTP, 10–12× on analytical,
and parity on point lookups and durable commit — *relative to PostgreSQL*. Every experiment
in `results/` (E1–E12) reports **counted work inside `proto-engine`** — rows touched,
reconstructions, checkpoint bytes — except §9.4.4, the one wall-clock table, which the thesis
itself labels "in-memory, single-threaded, no durability, no consensus" and compares to nothing. `crates/bank-bench/src/` contains a generator, scenarios
and an analysis module, and **no code path that connects to PostgreSQL** (`grep -rn postgres
crates/bank-bench/src` is empty). E14 (`results/E14-minimal-counterproposal.md`) ran REVs
*inside* PostgreSQL 16 to test engine-independence; it did not time Nilestream against it.

**Why it matters.** The thesis's primary stated goal is speed. A speed claim whose only
evidence is counted operations in a prototype is, in the thesis's own vocabulary, an
*editorial* contribution. Until a wall-clock harness exists, the Part 0 table is a
prediction wearing the typography of a result.

**Severity.** Blocking for G2. → **Task 3.**

### F2 · Claim-gap · GBS has no durable path; Phase 8 cannot start

**Evidence.** `gbs/crates/gbs-nilestream/src/lib.rs`: `NilestreamLedger::append` returns
`LedgerError::NotDurable` unconditionally and `durable: false` is a constant. The Nilestream
segment writer it should call exists — `niles/crates/nilestream-ledger/src/segment.rs`:
`Segment::open(path, SyncPolicy)`, `Segment::append(payload) -> Record`, `Segment::sync()` —
and is never referenced from GBS.

**Consequence.** `ROADMAP.md` Phase 8's gate ("conservation suite under continuous eviction,
over the wire, with durability on") has no runnable predecessor. The kernel's own
`ledger.rs` doc-comment argues that "a port with one real implementation is a description,
not an interface". Today the port has one implementation that works and one that refuses.

**Severity.** Blocking for G3. → **Task 2.**

### F3 · Logic · Product structs cache derived state, contradicting the thesis's own rule

**Evidence.** `gbs/docs/ARCHITECTURE.md` §6 and thesis §6.7 ("balance as a function of
time") state that **no balance field and no status column** exist: every derived quantity is
a fold over the ledger anchored at an epoch. The code stores derived state in five places:

```
gbs-mechanisms/src/hold.rs:155        resolution: Option<(Outcome, Stamp)>
gbs-products/src/lending.rs:96        drawn: Amount
gbs-products/src/matching.rs:146      last_sequence: u64
gbs-products/src/securities.rs:195    units_outstanding: Amount
gbs-products/src/tradefinance.rs:112  drawn: Amount
```

Each is a running total that is *also* recoverable from postings the same struct emitted.
Two sources of truth for one quantity is exactly the defect class the thesis says REVs
eliminate; here it is reintroduced one layer above the kernel. The edge case is concrete:
`Facility::drawn` after a crash-and-replay of the ledger is whatever the struct was last
constructed with, not what the ledger says.

**Severity.** High — it is a *logical* contradiction, not a performance issue. → **Task 1.**

### F4 · Logic · Two parallel implementations of every product

**Evidence.** `gbs/crates/gbs-products/src/lending.rs::drawdown` and
`gbs/niles/gbs.niles::syndicated_drawdown` implement the same operation, one in Rust and one
in Niles. Nothing checks they agree. `tests/niles_schema.rs` checks that the Niles file
*compiles* and that its obligations are discharged statically; it never executes a
transaction through both and compares postings.

**Why it matters.** The thesis argues (§6.9, "three surfaces, one IR") *against* seams of
exactly this kind. A conformance gap here means the schema's "5 conservation obligations
proved statically" is a proof about a program that is not the one running.

**Severity.** High for G3's credibility. → **Task 9.**

### F5 · Logic · The schedule verifier is specified over an undecidable problem

**Evidence.** `ROADMAP.md` Phase 5 and `SPEC-ENGINE.md` E-opt-3 require the verifier to
decide that a schedule "preserves the algorithm's denotation" for "the operator set". The
operator set in `niles-ir` includes difference/negation (`Circuit` supports Z-set
subtraction). Equivalence of relational algebra with difference is undecidable (Trakhtenbrot;
Abiteboul–Hull–Vianu §6.3). The roadmap's kill criterion — "if it needs a general theorem
prover, narrow the language" — would fire on day one because the fragment was never fixed.

**Edge case the current text misses.** Even within conjunctive queries, equivalence is
NP-complete (Chandra–Merlin 1977), which is fine for a verifier (queries are small) but must
be stated; and bag semantics (which Z-sets have) makes CQ equivalence *harder* than set
semantics — equivalence of CQs under bag semantics is graph-isomorphism-hard and containment
is open. The verifier must therefore be specified over **schedule rewrites drawn from a
finite catalogue of proven-equivalent transformations** (join commutativity/associativity,
predicate pushdown under stated conditions, projection pushdown), not over denotational
equivalence of arbitrary plans.

**Severity.** High — it is the roadmap's "novel contribution". → **Task 5.**

### F6 · Claim-gap · The hash chain is a placeholder, and L-13 leans on it

**Evidence.** `niles/crates/nilestream-ledger/src/chain.rs:7`: "256-bit FNV-1a-style
placeholder hasher. NOT collision resistant." `docs/SPEC-LANGUAGE.md` L-13 and
`REQUIREMENTS.md` restate zero-copy as "bounded by trust — the hash chain is the validation".
A validation built on a non-collision-resistant hash validates nothing an adversary cares
about. ADR 0002 records this honestly; the specs do not carry the caveat forward.

**Severity.** Medium now, blocking before any "auditability" claim is evaluated. → **Task 7.**

### F7 · Logic · The currency-row solver's `Undecided` rate is unknown

**Evidence.** `niles/crates/niles-lang/src/currency_rows.rs:452` defines
`Verdict::Undecided`; the surrounding comments explain correctly that the Karr-style domain
cannot decide affine guards. Thesis §11.5 claims "11 of 12 defect classes" are caught. No
corpus measures how often real banking code lands in `Undecided`. If typical revolving-credit
logic (guards like `if drawn + amount <= limit`) is mostly `Undecided`, the soundness theorem
is true and useless.

**Severity.** Medium for G1; it is a measurement gap, not a design error. → **Task 6.**

### F8 · Claim-gap · Bootstrap proves a lexer, and the roadmap has no parser phase

**Evidence.** `bootstrap/lexer.niles` and `crates/niles-interp/tests/bootstrap_stages.rs` (14
gates) establish stage 0→1→2→3 fixpoint **for the lexer only**. Appendix E describes a
self-hosted front-end, type-checker, and IR lowering. The gap between "lexer in Niles" and
"front-end in Niles" is the entire parser and AST, and no roadmap phase names it.

**Severity.** Medium for G1. → **Task 8.** **CLOSED.** `bootstrap/parser.niles` (~1,050
lines) plus `crates/niles-interp/tests/bootstrap_parser.rs` (16 gates) carry the bootstrap
from a lexer to a front end: stage-1 equivalence over a 48-case corpus, stage-2
self-application over both bootstrap files (127,165 bytes of tree, identical to the
reference), stage-3 fixpoint, and four negative controls. The round rejected three defects
— a 95 KB-per-frame stack cost in the stage-0 interpreter that aborted the process rather
than reporting, a **left-associative assignment in the reference parser**, and a missing
operator table in Appendix B — recorded in `results/E15-bootstrap-gates.md` round 2 and
thesis §E.19.2.

### F9 · Bottleneck · Linear scans where the thesis mandates anchors

**Evidence.**

* `gbs/crates/gbs-kernel/src/ledger.rs:186` `MemoryLedger::seen` scans a `Vec` → ingest is
  O(n²) in transactions.
* `gbs/crates/gbs-mechanisms/src/signal.rs:161` `Signal::at` iterates every entry to compute
  one account's balance at one anchor. `SPEC-ENGINE.md` E-3 makes an anchor index an
  *obligation*; GBS's own signal ignores it.
* `gbs/crates/gbs-products/src/matching.rs:241` `level.remove(0)` on a `Vec` is O(n) per fill
  at a deep price level — in the one component whose interface exists to make a latency
  argument.

**Severity.** Medium; none is wrong, all are the wrong asymptotics in code that documents
asymptotics as a claim. → **Task 1** (signal, seen) and **Task 10** (matching).

### F10 · Bottleneck · GBS has no concurrency model

**Evidence.** `grep -rn "Mutex\|RwLock\|Arc<\|Send\|Sync" gbs/crates/*/src` returns nothing.
Every mutation is `&mut self`. The thesis is careful that strict serializability is the
*engine's* property; but GBS products holding their own mutable state (F3) means two
concurrent drawdowns on one facility are serialised by nothing but the borrow checker on a
single thread. Once F3 is fixed the products become pure functions from (ledger prefix,
request) to postings, and the engine's admission order *is* the serialisation — the fix to
F3 is the fix to F10, which is why they are one task.

**Severity.** High. → **Task 1.**

### F11 · Logic · Determinism is verified on one target

**Evidence.** Appendix C §C.4 states a cross-target determinism *obligation*;
`niles-interp::determinism_gate` runs the same program N times on the host and compares. A
run-to-run check on one machine cannot detect the failures the obligation is about (float
formatting, hash iteration order, endianness in canonical encoding).

**Severity.** Low until WASM/ARM64 targets exist; recorded so the claim is not overstated.
→ **Validation Protocol V-11** (no build task; a wording task in Task 11).

### F12 · Logic · Coverage is structural, not semantic

**Evidence.** `gbs/crates/gbs-products/tests/coverage.rs` checks that each product file's
`use` lines match the mechanisms the matrix says it needs. It cannot detect a product that
imports M3 and never calls it. 13 of 29 rows are predictions with no code behind them.

**Severity.** Low; the test is honest about what it checks. → **Task 9** strengthens it.

### F13 · Logic · `check_generator` is only as strong as its sample

**Evidence.** `gbs/crates/gbs-mechanisms/src/schedule.rs:386` checks the generator over
occurrences × a supplied set of observations. Nothing specifies how observations are
sampled; a generator balanced on the supplied observations may be unbalanced on an
observation outside them (e.g. a negative rate, a rate at the scale boundary). The thesis
calls this a declaration-time proof; it is a declaration-time *test*.

**Severity.** Medium. → **Task 1** (add boundary sampling and rename the guarantee).

### F14 · Claim-gap · Contribution 2 is a paper proof over an informal cost model

**Evidence.** Thesis §3.15 and §4.3 (Eviction–Consistency Frontier). §3.15's verification
status says so honestly. It is listed here because the *evaluation* chapter's phase
diagram (§9.3) is the empirical shadow of that theorem, and the phase diagram is populated
from `proto-engine` counts — the same limitation as F1. When Task 3's harness exists, the
phase diagram must be regenerated from wall-clock data and the theorem's predicted boundary
overlaid; only then does Contribution 2 have an empirical test.

**Severity.** Structural; folded into **Task 3** and **Task 11**.

### Summary table

| # | Class | Severity | Goal | Task |
|---|---|---|---|---|
| F1 | Claim-gap | Blocking | G2 | 3 |
| F2 | Claim-gap | Blocking | G3 | 2 |
| F3 | Logic | High | G3 | 1 |
| F4 | Logic | High | G3 | 9 |
| F5 | Logic | High | G2 | 5 |
| F6 | Claim-gap | Medium | G2 | 7 |
| F7 | Logic | Medium | G1 | 6 |
| F8 | Claim-gap | Medium | G1 | 8 |
| F9 | Bottleneck | Medium | G3 | 1, 10 |
| F10 | Bottleneck | High | G3 | 1 |
| F11 | Logic | Low | G1 | 11 |
| F12 | Logic | Low | G3 | 9 |
| F13 | Logic | Medium | G3 | 1 |
| F14 | Claim-gap | Structural | G2 | 3, 11 |

**What the review did *not* find.** The layering test, the `Sealed` unforgeability, the
absence-lattice semantics, the anchored-upquery design, the honest `NotDurable` refusal, the
per-currency conservation check, and the rung-monotonicity type rule are all sound as
built. The extraction of GBS revealed a two-type dependency and the port abstraction that
replaced it is correct. Nothing below asks Opus to redesign those.

---

## Part 2 — The Opus handoff blueprint

### Executive summary for Opus

1. **Fix the logic first.** Remove every cached derived quantity from GBS (F3/F10/F13) so
   products are pure folds over a ledger prefix; give the kernel and signal the indexes the
   thesis mandates (F9).
2. **Make the claims measurable.** Wire GBS to Nilestream's segment writer with real fsync
   (F2); build a wall-clock benchmark harness that runs the *same* workload against
   PostgreSQL and Nilestream and emits the SPEC-ENGINE Part 0 table from data (F1/F14);
   replace the placeholder hasher with SHA-256 against FIPS test vectors (F6).
3. **Close the language gaps on a decidable footing.** Restate the schedule verifier over a
   finite catalogue of proven rewrites (F5), measure the `Undecided` rate on a corpus (F7),
   write the parser in Niles and extend the bootstrap gates to it (F8), and run one
   transaction through both the Rust products and the Niles schema to prove they agree (F4).
4. **Then expand**: the 13 unimplemented product lines, and ROADMAP Phase 2 (unnesting),
   which is worth more than everything else in the optimizer combined.
5. **Finally, correct the thesis text** wherever a task changed what is true (F11/F14), and
   never the other way round.

### Global constraints

These apply to every task. A task that cannot be completed within them is stopped and the
reason written to `docs/BUILD-LOG.md`, not worked around.

- **GC-1 · No fabricated numbers.** A figure appears in a `results/*.md`, a spec, or the
  thesis only if a command in this repository produced it, and the command, its commit, and
  the raw CSV are named next to the figure. Predictions are labelled *predicted* in the
  sentence that contains them.
- **GC-2 · Tests before claims.** Every task's Expected Output includes tests. A task is not
  done while any test in either workspace is red, and a task never deletes or `#[ignore]`s an
  existing test to become green. Test counts must be monotone: `niles` ≥ 430, `gbs` ≥ 258.
- **GC-3 · The layering test is law.** `gbs/crates/gbs-products/tests/layering.rs` must pass
  unchanged. `gbs-kernel`, `gbs-mechanisms`, `gbs-products` acquire **no** dependencies. Only
  `gbs-nilestream` (excluded from the workspace) may depend on Niles crates, by path.
- **GC-4 · No `unsafe` outside `nilestream-storage` and `nilestream-ledger/src/segment.rs`**,
  and there only with a `// SAFETY:` comment naming the invariant. `matching.rs` stays free of
  `unsafe` and of spin loops (its existing test asserts this).
- **GC-5 · Kernel vocabulary is fixed.** `Amount`, `Currency`, `Epoch(u64)`, `Minor = i128`,
  `Stamp`, `Sealed`, `LedgerSink` keep their names and signatures. New capability goes in new
  types or new trait methods with default implementations.
- **GC-6 · Honest refusal over silent fallback.** A function that cannot answer returns
  `Err`/`Unavailable`/`Undecided`; it never returns zero, a default, or a stale value.
  `Err(_) => 0` must not typecheck anywhere.
- **GC-7 · Determinism.** Any new computation that feeds a hash, an epoch, a posting, or a
  benchmark figure iterates in a defined order (`BTreeMap`/sorted `Vec`), never `HashMap`.
- **GC-8 · Commit discipline.** One commit per task, message = task title, body = the
  finding IDs it resolves, trailer:
  ```
  Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_012KJD5FHvwxtenT6HxbXskv
  ```
  Never rewrite history. Never commit `target/`.
- **GC-9 · Thesis edits follow code.** A thesis section is edited only in Task 11 or when a
  task's Expected Output names the section. Cross-references are updated by the script in
  `thesis/build.sh`; the `.docx` is rebuilt at the end of Task 11 only.
- **GC-10 · Order.** Tasks run in the order given. Task N+1 does not start until Task N's
  validation items (Part 3) are green. Tasks 6, 7 and 8 are independent of each other and
  may run in any order *after* Task 5.
- **GC-11 · Scope.** Do not build the matching engine hot path, kernel-bypass networking, a
  distributed deployment over a real network (Phase 7), or LLVM tiering (Phase 3). Those are
  out of scope for this handoff and are listed so they are not started by accident.

### Sequential task list

---

#### Task 1 — Pure products: derived state as folds over the ledger

**Resolves.** F3, F9 (signal, `seen`), F10, F13.

**Objective.** No struct in `gbs-mechanisms` or `gbs-products` stores a quantity that is
recoverable from postings or events. Every such quantity becomes a function of
`(events, anchor: Epoch)`. Products become pure: `fn drawdown(&self, ledger_view, request,
at: Stamp) -> Result<PostingSet, _>` with no `&mut self`.

**Input files.**
- `gbs/crates/gbs-mechanisms/src/hold.rs` (`resolution` at 155, `resolve` at 215)
- `gbs/crates/gbs-mechanisms/src/signal.rs` (`Signal::at` at 151)
- `gbs/crates/gbs-mechanisms/src/schedule.rs` (`check_generator` at 386)
- `gbs/crates/gbs-products/src/lending.rs` (`drawn` at 96, `drawdown` at 149)
- `gbs/crates/gbs-products/src/tradefinance.rs` (`drawn` at 112, `state_at` at 139)
- `gbs/crates/gbs-products/src/securities.rs` (`units_outstanding` at 195, `subscribe` 279, `redeem` 314)
- `gbs/crates/gbs-products/src/matching.rs` (`last_sequence` at 146 — replace with the
  sequence of the last *event in the input*, computed, not stored)
- `gbs/crates/gbs-kernel/src/ledger.rs` (`MemoryLedger::seen` at 186)

**Expected output.**
1. A new kernel type `gbs-kernel/src/view.rs`:
   ```rust
   /// A read-only prefix of a ledger, frozen at `anchor`. Everything a product may read.
   pub struct LedgerView<'a> { entries: &'a [(Epoch, Entry)], anchor: Epoch, index: &'a AnchorIndex }
   pub struct AnchorIndex { by_account: BTreeMap<AccountId, Vec<(Epoch, usize)>> }
   impl<'a> LedgerView<'a> {
       pub fn balance(&self, account: &AccountId) -> Reading;   // O(log n + k_account)
       pub fn entries_for(&self, account: &AccountId) -> impl Iterator<Item=&Entry>;
       pub fn anchor(&self) -> Epoch;
   }
   ```
   `Reading` is `Present(Amount) | Absent | Unavailable`, reusing `signal::Reading` moved
   into the kernel (mechanisms re-export it, so no caller changes).
2. `MemoryLedger` gains `seen_index: BTreeMap<TxnId, Epoch>` maintained in `append`; `seen`
   becomes a lookup. `frontier()` unchanged.
3. `Hold` loses `resolution`; a `HoldLedger` (mechanisms) records `HoldEvent::{Placed,
   Resolved(Outcome)}` as ledger entries in a dedicated hold-event relation, and
   `Hold::resolution_at(view) -> Option<(Outcome, Stamp)>` folds them. "Resolved exactly
   once" becomes a *ledger* invariant checked in `PostingSet::seal` for hold relations:
   two `Resolved` events for one hold id in one prefix is `KernelError::DoubleResolution`.
4. `Facility::drawn(view)`, `LetterOfCredit::drawn(view)`, `Fund::units_outstanding(view)`
   are folds via `view.balance` on the facility's own drawn/units account. Constructors lose
   the corresponding parameters.
5. `Schedule::check_generator` takes an `ObservationSpace` — a set of named observation keys
   each with `{min, max, scale}` — and samples the corners (all-min, all-max, each-min-others-max,
   zero where allowed, one-minor-unit above zero and below max). The doc comment and the
   ARCHITECTURE.md M2 row say "checked at declaration over the corner set of the declared
   observation space", not "proved".
6. Every product's mutating method becomes `&self` and returns postings; a `Session`
   helper in `gbs-products/src/session.rs` composes `view → product → seal → sink.append`
   for tests.
7. Tests: for each product, a **replay test** — apply N random operations, serialise the
   ledger, rebuild a fresh view from the entries alone, assert every derived quantity equals
   what the product computed live. A **crash test**: truncate the entry list at a random
   epoch and assert derived quantities equal the live values at that epoch. A **concurrency
   test**: two `drawdown`s computed from the same view both seal, the second to be appended
   after the first is re-validated against the new view and refused if it would overdraw
   (this is the engine's admission order doing serialisation, demonstrated).

**Architectural guardrails.**
- Do not keep a cache "for speed" behind a `RefCell`. The index is the speed.
- Do not make `LedgerView` own its data; it borrows a prefix, so a view can never outlive
  the anchor it was frozen at.
- `balance` must return `Unavailable` when the anchor is beyond the view's frontier —
  never `Absent`, never zero.
- The hold-event relation is a *kernel* relation (it needs the double-resolution check in
  `seal`), but its *meaning* stays in mechanisms. Add only the check, not hold semantics, to
  the kernel.
- `matching.rs`: `last_sequence` must be derived from the replayed input; the `OrderBook`
  may keep its price levels (they are the *algorithm's* state, not a derived quantity).
  Task 10 changes their representation; do not do it here.

---

#### Task 2 — A durable ledger: wire `gbs-nilestream` to the segment writer

**Resolves.** F2.

**Objective.** `NilestreamLedger::append` writes a hash-chained record to a Nilestream
segment, fsyncs according to policy, and returns the epoch only after `sync()` returns.
`is_durable()` becomes true and truthful.

**Input files.**
- `gbs/crates/gbs-nilestream/src/lib.rs`, `gbs/crates/gbs-nilestream/Cargo.toml`
- `niles/crates/nilestream-ledger/src/segment.rs` (read only; `Segment::open`, `append`,
  `sync`, `recover`, `Record::seal`)
- `niles/crates/nilestream-ledger/src/frontiers.rs` (read only; the seal-then-publish
  protocol at lines 55–60)

**Expected output.**
1. `NilestreamLedger::open(path, SyncPolicy) -> io::Result<Self>` that calls `recover` and
   rebuilds `seen: BTreeMap<TxnId, Epoch>` by scanning the recovered records' payloads.
2. `append`: canonical-encode the `Sealed` set (define `gbs-kernel/src/encode.rs` with a
   length-prefixed, big-endian, field-ordered encoding — **no serde**, GC-3), call
   `Segment::append`, then `Segment::sync()`, then publish (mirror the frontiers protocol:
   sealed-before-sync, visible-after-sync), then return `Epoch(record.epoch)`.
3. `frontier()` = the last *published* epoch, not the last appended.
4. Payload carries `txn_id` first so `recover` can rebuild `seen` without decoding the rest.
5. Tests (in the adapter crate, run with `cargo test -p gbs-nilestream` from the adapter
   directory): append 1 000 sets, drop the ledger, reopen, assert `frontier` and every `seen`
   match; append with a duplicate `txn_id` after reopen is refused with
   `Duplicate{txn, at}`; a crash test that truncates the segment file mid-record and asserts
   `was_clean() == false` and that the frontier is the last complete record; a
   chain-verification test that re-walks the segment and recomputes each `parent` hash.
6. `Makefile` target `durable-test` that runs the adapter tests; `README.md` "Two
   implementations ship" paragraph updated to say both are durable-capable and which one is.

**Architectural guardrails.**
- Never return an epoch before `sync()` returns `Ok`. If `sync` fails, the record is
  *sealed but not visible*; return `LedgerError::NotDurable` and leave the frontier
  unchanged — do **not** retry silently.
- Do not add a `serde` dependency to any GBS crate. Hand-written encoding, with a round-trip
  test.
- The adapter must not reach into `Segment` internals; if a needed operation is missing,
  add it to `segment.rs` in the Niles repository as a `pub fn` with its own test, in a
  separate commit to that repository.

---

#### Task 3 — The wall-clock benchmark harness against PostgreSQL

**Resolves.** F1, F14.

**Objective.** One binary, `niles/crates/bank-bench/src/bin/bench.rs`, runs an identical
workload against PostgreSQL (via the `postgres` crate, wire protocol) and against Nilestream
(via its own `pg_wire` server, so the client path is the same), and writes
`results/E16-wallclock/<workload>.csv` plus a rendered `results/E16-wallclock.md` whose table
has *exactly* the rows of `SPEC-ENGINE.md` Part 0.

**Input files.**
- `niles/crates/bank-bench/src/{generator,scenarios,analysis,lib}.rs`
- `niles/crates/nilestream-server/src/{main,pg_wire,session}.rs` (the server the harness
  drives)
- `niles/docs/SPEC-ENGINE.md` Part 0 (the table shape)
- `niles/docs/research/performance-baselines.md` (the PostgreSQL 18.1 = 333 txn/s/core
  figure to reproduce first, as the calibration)

**Expected output.**
1. Four workload classes, each a module under `bank-bench/src/workloads/`: `oltp.rs`
   (transfer-heavy, TPC-C-like New-Order/Payment mix over accounts), `analytical.rs`
   (ClickBench-shaped scans and group-bys over postings), `point.rs` (single-key balance
   lookups at the frontier), `durable.rs` (commit latency with fsync on both sides).
2. A `Target` trait `{ connect, prepare(schema), run(txn) -> Duration, teardown }` with two
   impls: `PgTarget` and `NilestreamTarget`. Both use the PostgreSQL wire protocol so the
   client cost is identical.
3. **Calibration gate**, run first and recorded: `PgTarget` on the `oltp` workload must
   land within ±30% of 333 txn/s/core on the harness machine, or the harness is wrong and
   the run stops with that message.
4. CSV schema: `workload,target,run,txns,wall_ms,p50_us,p99_us,cpu_cores,fsync_policy,commit`.
   Ten runs each, medians reported, min/max shown.
5. `results/E16-wallclock.md` generated by `bank-bench --render`, containing the Part 0
   table with a **Measured** column beside the **Target** column and a **Verdict** column
   with values `MET`, `NOT MET`, `PARITY`, or `NOT RUN`. Nothing is hand-edited into it.
6. The phase diagram (`thesis/09-evaluation.md` §9.3) regenerated from the analytical and
   point workloads at three eviction rates, with the Contribution-2 predicted boundary drawn
   from `nilestream-optimizer/src/eviction.rs` cost terms as an overlay line — this is the
   first empirical test of the Frontier theorem and must be labelled as such.
7. A `docs/BENCHMARK.md` stating the machine, PostgreSQL version and configuration
   (`shared_buffers`, `synchronous_commit`, `fsync`), and the exact command line.

**Architectural guardrails.**
- If PostgreSQL is not installed the binary prints how to install it and exits non-zero. It
  never substitutes an in-process stand-in.
- Nilestream's server is driven **over the wire**, not called in-process — otherwise the
  comparison silently omits the protocol cost that PostgreSQL pays.
- Report parity as parity. A target "NOT MET" is a result, and goes in the thesis as one
  (Task 11).
- Do not tune PostgreSQL below its documented production defaults to win; record every
  setting.
- Run `durable.rs` with `synchronous_commit = on` and `SyncPolicy::EveryCommit` on both
  sides; also report the `off`/`Never` pair separately, clearly labelled as non-durable.

---

#### Task 4 — ROADMAP Phase 2: subquery unnesting

**Resolves.** the roadmap's highest-value gate; no review finding, but it is the
prerequisite for any analytical target in Task 3 being met.

**Objective.** Correlated `exists` → semi-join; correlated scalar subquery → left outer join
with aggregation; `in`/`not in` → semi/anti-join with correct three-valued logic for
`not in` with NULLs.

**Input files.**
- `niles/crates/niles-lang/src/{lower,sql_surface}.rs` (where SQL surface reaches IR)
- `niles/crates/niles-ir/src/circuit.rs` (add `SemiJoin`, `AntiJoin` operators if absent)
- `niles/crates/nilestream-optimizer/src/lib.rs` (a new `unnest.rs` module)
- `niles/docs/ROADMAP.md` Phase 2 (gate text)

**Expected output.**
1. `nilestream-optimizer/src/unnest.rs` with `pub fn unnest(plan) -> Plan` and a
   `Rewrite` enum naming which rule fired, so the gate can be checked in the IR.
2. `niles-ir::verify` extended to accept the new operators and to reject a `SemiJoin` whose
   right side is not key-preserving (the classic duplicate-inflation bug).
3. A corpus `niles/crates/nilestream-optimizer/tests/unnest_corpus/` of ≥ 20 correlated
   queries with expected IR shape and expected results on a fixed dataset, including at
   least five `not in` cases with NULLs on each side.
4. Gate measurement: `results/E17-unnesting.md` reporting the nested/unnested ratio per
   query **in the proto-engine's counted work** (this is a rewrite gate, so counted work is
   the right unit here) and, once Task 3 exists, wall-clock on the `analytical` workload.

**Architectural guardrails.**
- `not in (subquery)` with a NULL in the subquery result yields **no rows**; a NULL on the
  left yields no row for that tuple. A single wrong case is the roadmap's kill criterion —
  stop and fix before continuing.
- Do not unnest through a `limit` or a non-deterministic function; refuse with a named
  reason.
- The rewrite must be **effect-preserving** in the Niles sense: a view at rung ℓ before
  unnesting is at rung ℓ after (rung monotonicity NL0311 must still pass on the rewritten
  circuit).

---

#### Task 5 — The schedule verifier, restated over a finite rewrite catalogue

**Resolves.** F5.

**Objective.** Replace "the verifier decides denotational equivalence" with "the verifier
checks that a schedule is a sequence of catalogue rewrites, each applied where its side
condition holds". Equivalence is then *by construction*, and the verifier is a checker, not
a prover.

**Input files.**
- `niles/docs/SPEC-ENGINE.md` E-opt-3, `niles/docs/SPEC-LANGUAGE.md` L-8/L-9,
  `niles/docs/ROADMAP.md` Phase 5
- `niles/crates/niles-ir/src/verify.rs` (`verify(&Circuit) -> VerifyReport`)
- new: `niles/crates/niles-ir/src/schedule.rs`

**Expected output.**
1. A written **fragment statement** in SPEC-ENGINE E-opt-3: the catalogue is (a) join
   commutativity, (b) join associativity, (c) selection pushdown below a join when the
   predicate references one side only, (d) projection pushdown when the dropped columns are
   unreferenced above, (e) semi-join introduction from Task 4's rules, (f) physical-operator
   choice (hash/merge/nested-loop with `InnerAccess::Indexed`) which is denotation-neutral.
   Anything else is **not a schedule** and is refused with `ScheduleError::NotInCatalogue`.
2. `schedule.rs`: `enum Step { Commute(JoinId), Assoc(JoinId, JoinId), PushSelect(..),
   PushProject(..), SemiJoin(..), Physical(..) }`, `fn apply(circuit, step) -> Result<Circuit,
   ScheduleError>` that checks the side condition *before* rewriting, and `fn check(circuit,
   schedule) -> Result<Circuit, ScheduleError>` = fold of `apply`.
3. The roadmap gate reworded: "A schedule step whose side condition fails MUST be rejected
   by `check`; a schedule that is only slower MUST be accepted; the negative control is
   `PushSelect` of a predicate referencing both sides." The kill criterion is deleted — it
   cannot fire on a checker.
4. Tests: every catalogue rule has a positive and a negative test on a small circuit, plus a
   **denotation test**: evaluate the original and scheduled circuits in `proto-engine` on
   random Z-set inputs (including negative weights) and assert equal outputs, 1 000 trials
   per rule.
5. The thesis does not yet mention verified schedules at all (they live only in
   `SPEC-ENGINE.md`/`ROADMAP.md`). Add a subsection §6.13 "Verified Schedules" (renumber the
   sections after it, as §6.11/§6.12 were) and a Related Work paragraph in §10; both must
   cite Chandra–Merlin for CQ equivalence and state plainly that bag/Z-set equivalence in general is not decided here
   — the contribution is that **hints become checked rewrites**, not that plan equivalence
   is decided.

**Architectural guardrails.**
- Do not add an SMT or e-graph dependency. The whole point is that the checker is tiny and
  in the trusted base.
- Side conditions must be **syntactic** (column references, key-preservation flags already
  in the IR). If a rule needs a semantic condition (e.g. functional dependency inference),
  leave it out of the catalogue and note it as future work.
- `Physical` steps must not be able to introduce `NestedLoop(Scan)` on a large inner — reuse
  `plan_space::check` from Phase 1 as the side condition.

---

#### Task 6 — Measure the currency-row solver's `Undecided` rate

**Resolves.** F7.

**Objective.** A corpus of realistic banking functions in Niles, run through the currency-row
solver, with the verdict distribution reported.

**Input files.**
- `niles/crates/niles-lang/src/currency_rows.rs`
- `gbs/niles/gbs.niles` (seed corpus: its 6 functions)
- new: `niles/crates/niles-lang/tests/solver_corpus/` (≥ 40 functions)

**Expected output.**
1. Corpus of ≥ 40 functions covering: unconditional transfers; guards of the form `if a +
   b <= limit`; loops over participants (M4); rate application (M7) with each rounding mode;
   FX conversion with two currencies; a hold placement/resolution pair; and 5 deliberately
   defective functions (currency mix, lost minor unit, double posting).
2. `results/E18-solver-verdicts.md`: per function, `Proved | Refuted | Undecided`, and the
   overall rates. For every `Undecided`, one sentence on which construct caused it.
3. If `Undecided` > 25% on the *non-defective* set, a follow-up item is added to
   `ROADMAP.md` (not built in this handoff): affine guard tracking via a sign-domain
   product, with citation to Miné's octagon domain.
4. Thesis §11.5 "11 of 12 defect classes" gets the measured rate beside it.

**Architectural guardrails.**
- Do not weaken the solver to reduce `Undecided`. An `Undecided` that becomes `Proved`
  without a new inference rule is a soundness bug.
- The five defective functions must all be `Refuted` — that is the negative control.

---

#### Task 7 — SHA-256 for the hash chain

**Resolves.** F6.

**Objective.** Replace `Hasher256` with a from-scratch SHA-256 (FIPS 180-4) passing the NIST
test vectors, and re-derive every hash-dependent test fixture.

**Input files.**
- `niles/crates/nilestream-ledger/src/chain.rs`
- `niles/crates/nilestream-ledger/src/segment.rs` (uses `chain_hash`)
- `niles/docs/adr/0002-*.md` (ADR to supersede), `niles/docs/SPEC-LANGUAGE.md` L-13,
  `niles/docs/REQUIREMENTS.md` (zero-copy paragraph)

**Expected output.**
1. `chain.rs`: `Sha256` with `update`/`finalize`, the 64 round constants, padding, and
   big-endian length; **no crate dependency** (the ledger crate stays dependency-free for
   the same reason the GBS kernel is).
2. Tests: NIST vectors for `""`, `"abc"`, the 56-byte and 112-byte messages, and one million
   `'a'`; a test that the old placeholder's output is *not* produced (so a stale build is
   caught).
3. ADR 0003 superseding 0002. L-13 and REQUIREMENTS reworded: "the chain is SHA-256 over the
   canonical body; zero-copy reads are validated by recomputing the chain over the read
   range."
4. Any stored fixture hash in `results/` or tests is regenerated, and `E13-durability.md`
   gets a line noting the hash change and the commit.

**Architectural guardrails.**
- Constant-time is not required (hashes are public); collision resistance is. Do not
  "optimise" the compression function by hand in this task.
- The canonical body encoding must be **stated** in `chain.rs` doc-comments field by field;
  it is part of the audit claim.

---

#### Task 8 — The parser in Niles and bootstrap stages for it

**Resolves.** F8.

**Objective.** `bootstrap/parser.niles`: a recursive-descent parser over the token stream
from `lexer.niles`, producing the same AST as `niles-lang/src/parser.rs` for the
non-relational subset the interpreter supports; the 14 bootstrap gates extended to the parser.

**Input files.**
- `niles/bootstrap/lexer.niles`, `niles/crates/niles-lang/src/{parser,ast}.rs`
- `niles/crates/niles-interp/src/lib.rs` (may need `Variant` pattern matching depth or a
  `Vec` push builtin — add builtins, do not add language features)
- `niles/crates/niles-interp/tests/bootstrap_stages.rs`
- `niles/thesis/appendix-e.md` §E.5

**Expected output.**
1. `parser.niles` covering: `fn`, `let`, `if/else`, `match`, `while`, struct/variant
   declarations, calls, operators with the precedence table from Appendix B, and literals the
   interpreter accepts. Relational forms are refused with `NotInSubset`, exactly as the
   interpreter refuses them.
2. An AST *serialisation* in both Rust and Niles (S-expression text) so the equivalence gate
   compares strings, not structures.
3. Gates: stage-1 parse of a 30-program corpus equals the Rust parser's output; stage 2:
   `parser.niles` parses `lexer.niles` and itself; stage 3: fixpoint on the serialised AST;
   negative controls (a missing brace, a keyword as identifier, precedence trap `a - b - c`).
4. `results/E15-bootstrap-gates.md` extended with the parser rows; Appendix E §E.5 gets a
   status line: *lexer and parser self-hosted; type-checker and lowering are Rust*.

**Architectural guardrails.**
- Precedence and associativity come from the *table in Appendix B*; if the Rust parser
  disagrees with the appendix, the appendix wins and the Rust parser gets a failing test.
- No aliasing: the `Step{tok, next}` discipline of the lexer carries over — the parser
  threads a position, it never mutates a shared cursor.
- Fuel: the interpreter's budget must be large enough for stage 2 on a laptop; report the
  fuel consumed in the results file.

---

#### Task 9 — Conformance: one transaction, two implementations, identical postings

**Resolves.** F4, F12.

**Objective.** A test that executes a transaction through the Rust product *and* through the
Niles schema function and asserts the sealed posting sets are byte-identical after
canonical encoding (Task 2's encoder).

**Input files.**
- `gbs/crates/gbs-products/tests/niles_schema.rs` (extend), `gbs/niles/gbs.niles`
- `niles/crates/nilesc/src/` (needs a `nilesc run <file> <fn> <args-json>` subcommand that
  evaluates a schema function via `niles-interp` over an in-memory ledger and prints the
  canonical posting encoding)
- `gbs/crates/gbs-products/tests/coverage.rs`

**Expected output.**
1. `nilesc run` subcommand, with its own test in the Niles repository.
2. `tests/conformance.rs` in `gbs-products`: for each Niles function in `gbs.niles`
   (`syndicated_drawdown` and the other five), a fixture `(inputs, expected canonical
   postings)` produced by the Rust product; the test invokes `nilesc run` and compares.
   Skips loudly without a Niles checkout, like the existing schema tests.
3. `coverage.rs` strengthened: for each mechanism a product's row claims, assert that the
   product's *test module* has at least one test whose name contains that mechanism's
   canonical tag (`m3_hold`, `m7_valuation`, …) — call-level coverage without a coverage
   tool, and a naming convention documented in ARCHITECTURE.md.
4. ARCHITECTURE.md §7 records: "N of 6 schema functions conform" with the commit.

**Architectural guardrails.**
- The fixture is produced by the Rust side and *committed*; the test does not regenerate it,
  so a change to either side is visible as a diff.
- If a function cannot conform because Niles lacks a construct, record it as a **language
  gap** in `docs/BUILD-LOG.md` with the construct named; do not paper over it by changing
  the Rust side.

---

#### Task 10 — Matching-engine interface: correct asymptotics

**Resolves.** F9 (matching).

**Objective.** Price levels as `VecDeque<Order>` (O(1) pop-front) inside the
`BTreeMap<i128, _>`, and `replay` verified as O(events log levels).

**Input files.** `gbs/crates/gbs-products/src/matching.rs` (line 241 and the `OrderBook`
definition at ~146).

**Expected output.** `level.pop_front()`; a test that submits 100 000 orders at one price and
asserts the fill loop's *counted* operations are linear (count via a test-only counter,
not timing); the existing determinism and no-`unsafe` tests unchanged.

**Architectural guardrails.** Do not introduce timing assertions (flaky) and do not build
the hot path (GC-11).

---

#### Task 11 — Thesis reconciliation

**Resolves.** F11, F14, and every wording change the tasks above produced.

**Objective.** The thesis says what the repositories do, and nothing more.

**Input files.** `niles/thesis/{06-architecture-and-niles,07-implementation,08-phased-program,
09-evaluation,11-discussion,appendix-c,appendix-e}.md`, `thesis/build.sh`.

**Expected output.**
1. §9: E16 (wall-clock), E17 (unnesting), E18 (solver verdicts) sections, each with its CSV
   named, and the Part 0 table with Measured/Verdict columns copied from
   `results/E16-wallclock.md` by script (`thesis/build.sh` gains an `--include-results`
   step; no hand-copying).
2. §9.3 phase diagram regenerated (Task 3 item 6) and captioned as the first empirical test
   of Contribution 2.
3. §6.13 verified schedules: added per Task 5 item 5; §6.14 onward renumbered and
   cross-references fixed.
4. Appendix C §C.4: "determinism is verified run-to-run on the host; cross-target
   verification is pending WASM and ARM64 builds" — stated, not implied.
5. §11.5.7 defect audit extended with F1–F3, F5, F6 and the tasks that resolved them.
6. `Niles-Thesis.docx` rebuilt; word count recorded in `docs/BUILD-LOG.md`.

**Architectural guardrails.** No new claims. If a number in the thesis has no CSV behind
it after this task, it is either labelled *predicted* or deleted.

---

#### Task 12 — The remaining thirteen product lines (expansion)

**Objective.** Implement the 13 rows in `gbs/docs/ARCHITECTURE.md` §4 not in
`implemented_rows()`, as compositions only, each with replay/crash tests (Task 1's pattern)
and a schema function in `gbs.niles` with a conformance fixture (Task 9's pattern).

**Input files.** `gbs/crates/gbs-products/src/` (new modules), `gbs/niles/gbs.niles`,
`gbs/docs/ARCHITECTURE.md` §4 matrix, `tests/coverage.rs::implemented_rows`.

**Expected output.** 29/29 rows; the coverage and conformance tests extended; ARCHITECTURE
§7 updated. If any row requires a kernel change, **stop**: that is the thesis's refutation
condition, and it is written up in `docs/BUILD-LOG.md` and thesis §11.5 as a finding, not
worked around.

**Architectural guardrails.** GC-3 and GC-5. A new *mechanism* (M8) is permitted only if it
is expressible as a pattern of M1 and ≥ 3 product rows need it; document it in the matrix
before writing code.

---

## Part 3 — Validation protocol

Run in this order after each task; the whole list after Task 12. A red item stops the
sequence.

| ID | Proves | Command / check | Pass criterion |
|---|---|---|---|
| V-0 | GC-2 baseline | `cd niles && cargo test --workspace`; `cd gbs && cargo test --workspace` | ≥ 430 and ≥ 258 pass, 0 fail, 0 ignored added |
| V-0b | GC-3 | `cargo test -p gbs-products --test layering` | pass, file unchanged (`git diff --stat` empty for it) |
| V-1a | F3 gone | `grep -rnE "^\s+(drawn|units_outstanding|resolution|last_sequence):" gbs/crates/*/src` | **no matches** |
| V-1b | F10 gone | `grep -rn "&mut self" gbs/crates/gbs-products/src gbs/crates/gbs-mechanisms/src` | matches only in `OrderBook` (algorithm state) and `Session` |
| V-1c | F9 seen | `cargo test -p gbs-kernel seen_is_a_lookup` | a test that appends 100 000 sets and asserts counted comparisons in `seen` ≤ 20 |
| V-1d | F9 signal | `cargo test -p gbs-kernel balance_touches_only_the_accounts_entries` | counted entries visited = that account's entries |
| V-1e | F13 | `cargo test -p gbs-mechanisms generator_unbalanced_at_a_corner_is_refused` | a generator balanced at the sample mean but not at max is refused |
| V-1f | crash | `cargo test -p gbs-products replay_` and `crash_` | all pass |
| V-2a | F2 | `cd gbs/crates/gbs-nilestream && cargo test` | reopen test, duplicate-after-reopen, truncation, chain re-walk all pass |
| V-2b | F2 honesty | `grep -n "NotDurable" gbs/crates/gbs-nilestream/src/lib.rs` | appears only in the `sync` failure branch |
| V-2c | fsync real | `strace -e fsync,fdatasync -c cargo test -p gbs-nilestream append_is_durable` (Linux) | ≥ 1 fsync per committed set under `EveryCommit` |
| V-3a | F1 calibration | `bank-bench --target pg --workload oltp --calibrate` | within ±30% of 333 txn/s/core, recorded in `docs/BENCHMARK.md` |
| V-3b | F1 table | `bank-bench --render && git diff results/E16-wallclock.md` | table has Measured and Verdict columns for every Part 0 row; no `NOT RUN` |
| V-3c | F14 | `ls results/E16-wallclock/phase_diagram.csv` and the overlay figure in §9.3 | present, generated, captioned |
| V-3d | GC-1 | `grep -rn "×" thesis/09-evaluation.md` | every ratio has a CSV name within 3 lines or the word *predicted* |
| V-4a | unnesting fires | `cargo test -p nilestream-optimizer corpus_` | every corpus query's `Rewrite` matches expected |
| V-4b | not-in NULL | `cargo test -p nilestream-optimizer not_in_with_null_` | all 5+ cases produce the SQL-standard result |
| V-4c | rungs preserved | `cargo test -p niles-lang nl0311_after_unnest` | rung monotonicity holds on rewritten circuits |
| V-5a | F5 fragment | `grep -n "NotInCatalogue" niles/crates/niles-ir/src/schedule.rs` | present; and `grep -n "kill criterion" docs/ROADMAP.md` shows Phase 5's removed |
| V-5b | denotation | `cargo test -p niles-ir schedule_denotation_` | 1 000 random Z-set trials per rule, equal outputs |
| V-5c | negative | `cargo test -p niles-ir push_select_across_both_sides_is_refused` | pass |
| V-6a | F7 | `cat results/E18-solver-verdicts.md` | ≥ 40 functions; rates present; 5 defective all `Refuted` |
| V-6b | no weakening | `cargo test -p niles-lang currency_rows` | unchanged test set passes; PCP-forced `Undecided` test still `Undecided` |
| V-7a | F6 | `cargo test -p nilestream-ledger sha256_nist_` | all vectors pass |
| V-7b | stale build | `cargo test -p nilestream-ledger placeholder_is_gone` | pass |
| V-7c | no dep | `grep -c "^\[dependencies\]" -A5 niles/crates/nilestream-ledger/Cargo.toml` | no external crate added |
| V-8a | F8 | `cargo test -p niles-interp --test bootstrap_stages` | lexer gates (14) + parser gates (≥ 6) pass |
| V-8b | fixpoint | the stage-3 parser gate | serialised AST identical across stages |
| V-9a | F4 | `NILES_ROOT=../niles cargo test -p gbs-products --test conformance` | 6/6 (then 29/29 after Task 12) byte-identical |
| V-9b | F12 | `cargo test -p gbs-products --test coverage` | naming-convention check passes for every claimed mechanism |
| V-10 | F9 matching | `cargo test -p gbs-products fill_loop_is_linear` | counted ops linear; no `remove(0)` in `matching.rs` |
| V-11a | F11 | `grep -n "run-to-run" thesis/appendix-c.md` | the pending-cross-target sentence present |
| V-11b | docx | `thesis/build.sh` | exits 0; word count logged |
| V-12 | expansion | `cargo test -p gbs-products --test coverage implemented_rows` | 29 rows; layering unchanged |
| V-Σ | totals | both workspaces | test counts recorded in `docs/BUILD-LOG.md`, monotone vs. the start |

**Logical checks that are not tests** (Opus writes the answer into `docs/BUILD-LOG.md`):

1. After Task 1: *Is there any quantity a product returns that could differ from a fold over
   the ledger at the same anchor?* The answer must be "no", with the reasoning that every
   field remaining in every product struct is either an identifier, a declared term, or
   algorithm state (`OrderBook` levels).
2. After Task 3: *Does the harness measure what Part 0 promises?* For each row, name the
   workload module and the metric column that produced the Measured value.
3. After Task 5: *Can `check` ever answer "maybe"?* It must not be able to; the return type
   is `Result<Circuit, ScheduleError>` and no variant means "unknown".
4. After Task 12: *Did any product need a kernel change?* If yes, the thesis claim in
   §1.1/§6.7 is refuted and §11.5 says so.
