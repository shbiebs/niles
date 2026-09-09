# Cycle 10 — execution report

Executor: Opus 5, in the cloud container, against the consolidated work order at
`docs/audit/cycle-10/work-order-10.md` (commit `c075bae`).

Integration parent: `c075bae` on `c10/00-audit`. Working branch: `c10/01-instruments`.

This report is written as the work is landed and is incomplete until the tasks above the cut
are done. Sections are added in the order the tasks were executed, which is the order the
work order sets: **T00**, then **T00a**, then T01, T02, T04, T07.

---

## T00 — Correct instruments and measure the current baseline

Target sentences, quoted verbatim from the work order:

> **Target T00.1:** The current-build baseline script reports every requested phase and admissibility row at verified SHAs, rejects each harness fault injection, and produces a neutral A=A control before any performance task is scored.

> **Target T00.2:** The wire and raw traces separately report shared/exclusive B waits and holds, deferred_merges including zero, both V waits, joined-read outcomes, refusal reasons and phase-separated anchor gaps with counters that reconcile to their defined events.

### Status

**T00.2: landed.** **T00.1: landed, and smoke-run end to end in the container; the author's
measured run on Host C is the outstanding step.** No performance number in this section is a
result — the container is a 2-core VM and the transcript below exists to show that the
harness runs and that the instruments produce numbers a reader can act on, not to be scored.

### Commits

| SHA | What |
|---|---|
| `8d6bab9` | every phase of a keyed read is measured, and every counter is asked for by name |
| `44da088` | the wire's names and its values come from one list, not two |
| `3eebfa3` | a lock declares its modes, so the split sums to the aggregate by arithmetic |
| `a1b3e76` | the stats table is checked for the width defect the slow-read table had |
| `f78472f` | the stats snapshot no longer holds the view while it waits for the base (A10-01) |
| `caa1966` | the baseline harness refuses the faults it is supposed to refuse |
| `06b9be2` | the witness invocation passes its filters to the test binary |
| `1db11ec` | two instrument defects the harness's own first run exposed |
| `4488234` | a deleted output directory is a prune, not a mystery |

Workspace gate at `4488234`: **1,045 pass / 0 fail / 7 ignored**, `cargo clippy --workspace
--all-targets` clean, `cargo fmt --check` clean. Parent `c075bae` measured in a detached
worktree for comparison: **1,037 pass / 0 fail / 7 ignored**. The README's test-function count
guard moved 898 → 902 over these commits and was updated each time it fired, as it instructs.

### What T00.2 added

Every phase of `answer_from_view`, where three existed:

| column | what it is | why it was not there |
|---|---|---|
| `base_wait_us` | waiting for B | was there |
| `view_wait_us` | waiting for V, first acquisition | was there |
| `view_hold_us` | holding V across `begin_read` | was there, but summed with the second hold |
| `fold_us` | the reconstruction, V released, B held | never measured |
| `view_wait2_us` | waiting for V to install | never measured (A10-08) |
| `view_hold2_us` | holding V across `finish_fold` | never measured |
| `outcome` | `hit` / `fold_owned` / `fold_alone` / `joined` / `join_retried` | joined reads never appeared in the table at all |
| `gap_begin`, `gap_finish` | `applied − anchor` at each phase | never measured (A10-11) |
| `rounding_us` | the residual | computed against three phases while three more existed |

The base lock is one `RwLock` taken in two modes and was reported as one histogram, whose p99
belongs to neither. `base_read` and `base_write` are now reported beside `base`, which is
unchanged so that a number an earlier cycle quoted still means what it meant.

`nilestream_stats` gained `deferred_merges` (reported at zero — an unasked zero is A10-16),
`waiters_refused` (counted only against a genuinely full waiter list, not a settled flight),
`joins_answered` / `joins_retried`, the two gap totals with their two denominators, and
`flights_behind_at_begin` / `flights_that_fell_behind`.

### Guards, each proved by reverting the change it guards

Every transcript below is from a disposable `git worktree` at the commit under test, with the
named line reverted and nothing else. **No witness is a compile error.**

**G1 — `the_slow_read_table_declares_every_column_it_emits`.** Reverting the generated
`RowDescription` to a hand-written list of five:

```
assertion `left == right` failed: the row description must be generated from the column table
and not written out beside it
  left: ["rank", "total_us", "base_wait_us", "view_wait_us", "view_hold_us", "rounding_us"]
 right: ["rank", "total_us", ..., "gap_begin", "gap_finish", "rounding_us"]
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

Reverting the `DataRow` to one column narrower than the description:

```
assertion `left == right` failed: row 0 carries 11 values under 12 declared names — a client
would read every column after the first extra one as the wrong quantity
  left: 11
 right: 12
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

**The first version of this guard was vacuous and the reversion proved it.** It iterated the
emitted rows, and a run in which no keyed read reaches the view emits none, so dropping a
`DataRow` value left it green. The property was moved out of the test: `SLOW_READ_COLUMNS`
carries each column's name, wire type and value function, and both sides are generated from
it, so equal width is a fact about the code rather than about what else happened to run
(`44da088`).

**G2 — `the_lock_histograms_name_six_scopes_and_a_reset_empties_them`.** Reverting the scope
array to four:

```
assertion `left == right` failed: six scopes: the aggregate base, its two modes, then the
nested scopes outermost last
  left: ["base", "view", "statement", "wire"]
 right: ["base", "base_read", "base_write", "view", "statement", "wire"]
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

**G3 — the mode routing.** This guard was vacuous too, and twice over. Its first form summed
the two modes' acquisition counts and compared them to the aggregate's; deleting the
shared-mode recording from `TimedRead::drop` left it **green**, because the test drives a
`MemoryEngine` that never takes `ENGINE_LOCK` at all, and because `0 + n == n` holds for a
build that counts no shared acquisition. The property was moved into the type: `LockStats`
declares the histograms its modes are recorded into, `ENGINE_LOCK` is
`with_modes(&BASE_READ, &BASE_WRITE)`, and the `Drop` impls record into whatever the lock
declares. Reverting the routing:

```
assertion `left == right` failed: the shared acquisition reached `base_read`'s analogue
  left: 0
 right: 1
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

Reverting the declaration (`ENGINE_LOCK = LockStats::new()`):

```
panicked at session.rs: the base declares a shared-mode histogram
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

**G4 — `nilestream_stats_names_the_columns_the_benchmark_reads`.** Removing
`Field::int8("deferred_merges")`:

```
`select nilestream_stats` must name `deferred_merges` — the benchmark looks it up by name and
renders `n/a` when it is missing, so dropping it here would quietly unmeasure the column
rather than break anything.
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

Emitting one value with no declared name (the same defect class, in the table that still
keeps two lists):

```
assertion `left == right` failed: `select nilestream_stats` emitted 24 values under 23
declared names; a client reads every column after the first extra one as the wrong quantity
  left: 24
 right: 23
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

**G5 — `a_joined_reader_waits_outside_the_read_and_holds_nothing`.** Taking the base across
the wait:

```
the caller's `Wait` arm contains `self.base()`: the wait is only cheap because it is taken
holding nothing
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

This guard's slice ran to the first `}` in the arm's text and therefore stopped four lines
early once the arm contained a braced `if`; it now ends at the arm's own closing brace and
additionally forbids a lock being taken across the wait.

### A10-01, landed inside T00 because the baseline depends on it

`read_stats` acquired V and then reached for B underneath it; `append` takes B exclusively and
then V. Two connections close the cycle, and `select nilestream_stats` is an ordinary
statement — which the mixed sweep issues between levels. A baseline that can hang is not a
baseline, so this landed before anything was measured.

The repair is not nesting in the right order but **not nesting**: B is taken, read and
released, then V is acquired. The cost is stated in the code — the snapshot is no longer
atomic across the two locks, so a row can pair an idempotency-window size with view counters
one epoch newer.

**Witnessed, latched, and bounded.** A `cfg(test)`-only rendezvous inside `read_stats`, keyed
to one thread so a parallel test's stats query is untouched, parks the stats thread between
its two phases; the appender is started while it is parked and released afterwards. Reverting
the two statements:

```
the append never returned within 10s. It holds the base exclusively and is waiting for the
view; the stats snapshot holds the view and is waiting for the base. That is the cycle of
A10-01, and no answer is ever wrong on the way into it — the connection simply never replies.
test result: FAILED. 0 passed; 1 failed;   finished in 10.29s   EXIT=101
```

The source guard listed two functions by name and `read_stats` was the third. It is now a
derived scan over every function in the file that takes both locks, and it **strips comments
before reading positions** — because the first draft of this very repair carried a paragraph
naming `self.base()` above the acquisition, and the scan passed against the deadlocking build
on the strength of the prose describing the deadlock. Reverting, with comments stripped:

```
`read_stats` takes the view at 604 and the base at 695. Every path must take the base first
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

The scan is asserted to find at least three functions and to name `answer_from_view`, `append`
and `read_stats`, so a change in how the locks are spelled cannot empty it and leave a green
test asserting nothing. It currently finds exactly those three.

### What T00.1 added to `c10-baselock.sh`

The script it replaces defaulted its baseline to `c7/01-durable-rows` — a cycle-7 branch — so
a run with no flags measured a build three cycles old and labelled it "baseline". It reused a
leftover worktree at whatever commit that worktree happened to be at, swallowed warm-up
failures, never checked the port, never bounded a replicate, and never noticed a level with no
writer progress or a column the server does not have.

* The baseline is the checked-out build, resolved to a full SHA and printed. A repository
  with **modified tracked files** is refused; a worktree whose HEAD is not the SHA under test
  is refused. Untracked files are counted and named in the preflight but are not a refusal —
  each arm is built in a detached worktree at a resolved SHA, so an untracked file reaches no
  binary, and Host C's checkout permanently carries the five files the protocol forbids
  touching. A check that refused on those would have refused every run on the machine it was
  written for, and the obvious way out would have been to delete them.
* **Two working points per level, labelled.** The historical 2,000 accounts / 2,500 budget —
  a budget above the distinct key count, so the view is effectively full — and 2,000 / 400,
  which is genuinely partial. Cycle 9 measured only the first and wrote about partial
  materialisation.
* **A neutral A=A control under `--baseline-only`**: the same commit built twice in two
  worktrees and interleaved exactly as a real comparison would be. This stage carries no pass
  line, and it runs before any performance task is scored.
* **AB/BA order balancing**, so what the first arm of a pair pays is not charged to the same
  arm every time.
* A per-replicate deadline implemented without `timeout(1)`, a free-port check before each
  daemon, a writer-progress check, and a missing-column check that reads the benchmark's own
  warning rather than publishing a table of `n/a`.
* The bounded deadlock witness runs **first, before any build**, and a failure there reports
  the baseline as **blocked** rather than slow, and stops.
* `env -C` is GNU and Host C is a Mac; the working directory is set by a subshell.

### The fault injections — `--self-test`, run in the container

Every injected fault refused, **and every clean control accepted**, which is the half that
catches a check that refuses everything:

```
=== self-test: every refusal, against the fault it is for ===
  accepted   : a clean repository
  refused    : a dirty repository
  accepted   : a worktree at the commit under test
  refused    : a worktree at another commit
  accepted   : a worktree that does not exist yet
  accepted   : a free port
  refused    : an occupied port
  accepted   : a complete log
  refused    : a log missing a reported column
  refused    : a log whose flight counters are n/a
  accepted   : a log with writer progress
  refused    : a level with no writer progress
  refused    : a run with no mixed level at all
  accepted   : a command inside its deadline
  refused    : a command past its deadline (exit 124)

=== self-test: the flags ===
  refused    : `--nonsense`
  refused    : `--publish`
  refused    : no arms named at all

=== self-test: status ===
every injected fault was refused, and every clean control was accepted
EXIT=0
```

**The self-test caught two of its own checks.** The writer-progress check matched `0 writes/s`
as a *substring* of `900 writes/s` and refused a healthy level; and the port check had nothing
to be tested against, so it now starts a listener with whichever of `perl` or `python3` is
already on the machine and reports the case as not run — and fails — if neither is. Nothing is
installed.

Two further faults are exercised by the real path rather than by `--self-test`, and both fired
during the container smoke run: a stale-SHA/mismatched-worktree refusal, and the dirty-repo
refusal, which stopped the first smoke run at the preflight because the script itself was
uncommitted.

### The container smoke run — not a result

Configuration: `--baseline-only --warmups 0 --measured 1 --seconds 3` on a 2-core VM at
`4488234`. Exit 0, every section ran. It is reproduced here only to show what the instruments
now say. **Three things in it are worth the author's attention before the Host C run.**

**1. The tail is base wait, and the base is held by readers.** Every level, both working
points:

```
scope      | acquisitions | wait p99 | hold p99 | hold max | hold total
base       |        90132 |     1023 |      511 |     4016 |    2969996
base_read  |        77158 |     1023 |      511 |     4005 |    2874132
base_write |        12974 |     1023 |       63 |     4016 |      95864
view       |       140833 |      511 |       31 |      407 |     292098
```

`base_write`'s hold p99 is 63 µs against `base_read`'s 511 µs, and the exclusive mode's total
occupancy is 3% of the shared mode's. The aggregate `base` row — the only row cycle 9 had —
reports a p99 of 511 µs that belongs to neither mode. Reader-lock-time is not occupancy and
the table says so, but the *direction* is now separable, which is what A10-08 asked for.

**2. No flight ever fell behind — and that is a consequence, not a coincidence.**

```
anchor gap: begin mean 4.78 over 45681 flights, finish mean 4.78 over 45681 installs,
max at finish 17 (epochs); 45282 began behind, 0 fell further behind
```

`flights_that_fell_behind` is **0** in every level of every run, and the begin and finish means
are equal to two decimals. The mechanism is visible in the same transcript:
`answer_from_view` holds the base **shared for its whole body**, including across the fold, and
`append` needs it **exclusively**. No epoch can be applied while any keyed read is in flight,
so `applied` cannot move under a fold and no flight can fall behind one. This is A10-10's
finding measured rather than argued, and it means the deferred-merge decision of T04 is being
asked about a window that, on this build, is empty by construction. **The gap is entirely
`applied − anchor` at arrival — about four to five epochs, the lag between the reader's
sampled frontier and the view's applied point — and none of it is caused by folding.**

**3. `pinned_installs` is essentially every install.** 45,282 pinned against 45,681 installs,
with `pending_joins` in the hundreds. Nearly every keyed read arrives with an anchor already
below `applied` and installs pinned. `fallback` is 0.00% throughout, `deferred_merges` 0,
`waiters_refused` 0, `flights_refused` 0, and every join that was entered was answered —
`joins_retried` 0 at every level.

### Two instrument defects the smoke run itself exposed

Both were found by running the harness, not by reading the code, which is the argument for
running it in the container before the author does.

**A mean above its own maximum.** The first flight line divided both gap totals by
`pending_joins` — the nearest counter to hand — and printed `mean 386.8 at begin` beside
`max at finish 16`. A mean cannot exceed the maximum of the same quantity. The two totals
accumulate over different populations and `pending_joins` is neither; the runtime now reports
`gap_begin_samples` and `gap_finish_samples` and each mean is taken over its own denominator,
with `n/a` rather than a division when the denominator is zero.

**A hold that ended before the guard did.** `install_hold` was read while the second view
guard was still alive, so releasing it fell outside every phase and landed in the residual
column — one row showed **1,160 µs of "rounding"** on a table whose six integer divisions can
lose at most six. The guard is dropped explicitly and the elapsed time read after it.

Also corrected: the comment on the finish gap claimed it was recorded "installed or not",
which the `if ticket.install` beneath it has never done.

### What is outstanding for T00

The measured run on Host C. It is the author's, and the instruction is in the message that
accompanies this report.

---

## T00a — Resolve the newly observed gate failures before scoring speed

> **Target T00a.1:** The GBS tamper guard mutates verified record regions, distinguishes header and payload corruption, preserves all original bytes on refusal, and passes its clean-file negative control on both paired toolchains.

> **Target T00a.2:** The current E18 allocation-budget violation is attributed with same-source, host/toolchain-labelled runs and is either repaired with a reverting cost guard or recorded as an explicit unresolved gate failure without raising the budget to hide it.

> **Target T00a.3:** The GBS continuous-eviction guard preserves all value/conservation assertions and separately demonstrates an actual resident hit under an explicit retention precondition instead of assuming every new entry remains cached.

> **Target T00a.4:** The paired adapter guard refuses missing GBS unless an explicit opt-out is reported as not-run, Appendix D includes legitimate API items after comment and intermediate-test traps, and Darwin preflight names and checks its actual barrier and data-volume evidence.

> **Target T00a.5:** A named old-window-gap retry is accepted by both admission and sealer after expiry, an in-window duplicate is refused consistently, and no applied-but-undurable visibility or premature acknowledgement occurs under either retry schedule.

### Status

All five landed in the container. **T00a.1's "both paired toolchains" and T00a.4's Darwin
half are the author's**: the remote-devices shell that reaches Host C is a Linux VM, so
`platform.system()` there is `Linux` and the Darwin path cannot be exercised through it at
all. Host C's own terminal is the only place that runs it.

### Commits

| Repo | SHA | What |
|---|---|---|
| G | `20d001e` | the tamper guard aims at a region it has decoded (T00a.1) |
| G | `01e1d85` | the hit path is demonstrated where the entry is proved to survive (T00a.3) |
| N | `e29a025` | a flight nobody joined never takes its rendezvous (T00a.2) |
| N | `92f3861` | Appendix D stops losing its API to a sentence about testing (T00a.4) |
| N | `f50b100` | the paired gate refuses a missing GBS, the Darwin barrier is a barrier (T00a.4) |
| N | `61c757b` | the two idempotency windows are tested where they meet (T00a.5) |

Container gate at `61c757b` / `01e1d85`, through `c10-gates.sh`: niles **1,050 pass / 0 fail
/ 7 ignored**, GBS workspace **504 / 0 / 1**, GBS adapter **53 / 0 / 0**, generated documents
green, `make reproduce` green, and the paired adapter gate green rather than skipped.

### T00a.1 — the tamper guard was aimed at nothing in particular

`a_tampered_record_is_refused_and_the_segment_is_not_truncated` said it flipped "a byte inside
the third record's payload region" and flipped `buf.len()/2`. That lands in a record *header*:
the reader answered `damaged record header at offset 660`, and the assertion demanded the
substring `damaged at offset`, which is the message for damage in a body. A real red test and
a stale instrument at once — the reader was right, and the aim had never been checked against
the format.

`record_bounds` now walks the segment by its own length prefixes and **asserts the walk lands
exactly on the end of the file**: if the framing the test assumes is not the framing on disk,
that is a failure rather than a silently mis-aimed flip. Three tests where there was one,
because a header finding and a body finding have different remedies — a bad body is a record
that can be identified and quoted, a bad header is a record whose extent is unknowable and on
whose word nothing may be discarded (F-61). Both refusals assert byte-for-byte preservation
rather than an unchanged length, since a refusal that rewrote the file to the same size would
pass a length check and destroy the evidence anyway. The clean-file control is its own named
test, so "the refusals are not refusing everything" is a claim a reader can find in a
transcript.

**Reversions.** Restoring the flip to `buf.len()/2`:

```
a damaged payload must be reported as damage in a record with records after it, not as a
damaged header: ... "segment ... has a damaged record header at offset 660: its length does
not match its own check word ..."
test result: FAILED. 0 passed; 1 failed;   EXIT=101
```

That is A10-18 reproduced exactly. Disabling the reader's two mid-file refusals in the paired
`nilestream-ledger` checkout:

```
a segment tampered with in the middle must not open
a segment with a damaged record header must not open
test result: FAILED. 1 passed; 2 failed;
```

— both tamper tests fail while the clean control passes, which is the shape that says the
controls separate.

### T00a.2 — the E18 allocation gate, repaired rather than excused

`make reproduce` fails `rev_metadata_2x_budget` on Host C with 44,165 allocations against a
34,165 budget: 10,000 more over 5,000 keys, two per key, +112 bytes each, `live` and `peak`
identical. The mechanism is `Completion`. Rust's std lazily boxes a `pthread_mutex_t` (64
bytes) and a `pthread_cond_t` (48) on macOS at first use, and `finish_fold` published to every
flight's completion whether or not anyone was listening. 64 + 48 = 112, twice per uncontended
fold. Linux, where the budget was measured, uses futexes and allocates nothing — which is why
the budget could not see a cost paid on every fold.

**The budget is not raised.** A completion is marked when it is handed to a joining reader,
under the view lock, and `publish` returns without touching the mutex when it never was. The
cancellation flag moved out of the guarded state into an atomic for the same reason:
`reap_cancelled` asks every miss whether a flight is dead, and that read was taking the lock
the publish path had just been taught not to.

**The guard is a count, not a duration**, because the cost is a first-acquisition allocation
and every later acquisition is nearly free. It therefore fails identically on Linux, where the
same saving is two atomic operations:

```
assertion `left == right` failed: ten uncontended folds took 10 completion lock(s). Each
first acquisition costs two heap allocations and 112 bytes on macOS ...
  left: 10
 right: 0
```

The counter lives on the completion and is folded into the view's stats when the flight ends.
Its first draft was a process-wide static and **passed alone while failing under `cargo
test`'s parallelism**, because another test's joined flight incremented it between the two
reads — a counter a guard cannot isolate reports someone else's work.

The control is what makes the guard mean anything: a build that skipped the rendezvous
unconditionally would pass the count perfectly and strand every waiter. Reverting `mark_joined`
proved that by **hanging the container** until it was killed. A red test that never returns is
not a result, so the joined reader's wait is on its own thread behind a ten-second deadline,
and the reversion now fails with `the joined reader was never answered within 10s`.

**Cost, stated.** `size_of::<Stats>()` went 128 → 200 and `size_of::<Rev>()` 336 → 408 across
this cycle's nine new counters, measured against `c075bae`; `Completion` did not grow at all,
96 bytes both sides, its three new fields fitting in existing padding. `results/E18-memory`
moved by exactly 4 × 72 bytes on `ledger_seeded` with its allocation count unchanged at
150,254.

### T00a.3 — the hit path, where the entry is proved to survive

`money_is_conserved_at_every_anchor_under_continuous_eviction` ended by requiring a second read
of one key to be a hit, "or the hit path is untested everywhere in this repository". Against
that ledger — budget three, nine keys, `Policy::CostAware`, a view already full from sixty
operations — that is not a property of the engine: the victim chosen at a full budget can be
the entry the read just inserted (A10-20).

The claim is not deleted, because deleting it leaves the hit path unexercised. It moves whole
into `a_second_read_of_one_key_is_served_from_resident_state`, where the budget is above the
key space and the two reads are bracketed by an eviction count that must not move. An eviction
between them is reported as an **unmet precondition** — this run witnesses nothing about hits
— rather than as a hit that failed to happen. The eviction phase keeps every value and
conservation assertion, plus `evictions > 0`, `wipes >= 15` and `upqueries > 0`.

**The first attempt at the reversion was itself informative.** Moving the hit assertion to a
*fresh* ledger at budget three still passes: the failure needs a view already holding budget
entries. The finding is not "a small budget evicts a fresh insert" but "at a **full** budget it
can". Restoring the assertion to the end of the eviction loop reproduces A10-20 exactly, and
the new precondition is reachable too — at budget one with another key read between the two it
fires with `2 eviction(s) happened between the two reads`.

### T00a.4 — three checks that reported what they had not established

**The paired adapter gate passed when it did not run.** With no GBS checkout it printed
`SKIPPED` and returned, so `cargo test` said `ok`. A reader of a transcript could tell; a
reader of the gate could not, and the gate decides — which is how the pair stayed broken for a
whole cycle with both sides green. A missing checkout is a refusal now, naming `GBS_ROOT` and
the sibling path. `NILES_NO_GBS=1` remains for standalone development and produces a
`PAIRED ADAPTER GATE: NOT RUN` line rather than a pass; `c10-gates.sh` refuses a transcript
containing that line and refuses to run at all if the variable is set in its environment. All
three behaviours exercised in the container.

**Appendix D was losing its API to a sentence about testing.** The generator truncated each
file at the first occurrence of the *characters* `#[cfg(test)]`, anywhere. `eval.rs` explains
in its module documentation why the evaluator is public "rather than living in a
`#[cfg(test)]` block", and that sentence deleted `ZSet`, `eval_scalar` and every other public
item in the file. Two more shapes were wrong the same way: a test module in the *middle* of a
file deleted the API below it, and the attribute on a non-module item swallowed the rest of
the file. Across the workspace this understated the public surface by **32 items** —
`niles-ir` 40 → 58, `nilestream-server` 78 → 87, `bank-bench` 65 → 70 — silently, and in the
direction that looks correct.

The extractor masks comments and string literals (raw strings included) before looking for the
attribute, and each attribute covers its own item: to the matching brace for a block, to the
terminating semicolon otherwise. Guarded in two halves, because either alone is weak — a
`--self-test` over four synthetic traps, which fails if the extraction regresses, and drift
assertions on the *committed* appendix, which fail if the generator is fixed and the file never
regenerated. Both proved separately. The guard's own first draft asserted `pub struct ZSet`,
which is a `pub type`, and failed against an appendix that was already correct.

**The Darwin barrier probe was not measuring a barrier.** The preflight asked for
`os.fdatasync`, which macOS does not have, caught the `AttributeError` and fell back to
`os.fsync` — which returns before the drive has committed its own write cache. Every
durability verdict this preflight produced on Host C described a call that does not durably
store anything (A10-17). It uses `fcntl(fd, F_FULLFSYNC)` there now, checks the interface
exists rather than assuming, and reports an unavailable or failing barrier as NOT RUN with a
non-zero exit rather than substituting a call that is not one. The mount line came from
`mount | grep ' on / '`, which on macOS describes the read-only system volume reached through
a firmlink; the mount point now comes from `df` on the path itself.

**`c10-gates.sh`**, which the work order asks the author to run and which did not exist. Its
own first run in the container found a bug in itself: backticks inside a double-quoted `note`
executed `numeric_binary_oracle` as a command.

**`c10-rwlock.sh`** took no arguments and ignored everything, so a mistyped flag ran the
default probe and reported it as an answer to the question asked; it refuses unknown arguments
now and has a real `--deadline`. Its quantiles were read at index 100 and 180 of 200 sorted
samples and labelled p50 and p90 — nearest-rank is 99 and 179. Its one "toolchain" line named
the override it uses rather than the tree's pin, so a reader could not tell whether they agree
and the audit that ran it on two Mac toolchains had nothing to distinguish them by. Container
run at the repaired script: 170/200 admitted after the queued writer, p50 239 µs.

### T00a.5 — the two windows, tested where they meet

The window is enforced twice and nothing tested them together. **The target is accept, not
refuse**, and that distinction is the finding: an identity that has aged out of a declared
window is one the system has promised to forget, so a retry of it is a new transaction that
must commit. Both-refuse would turn a lost acknowledgement into a silently dropped payment.
The test asserts agreement in both directions — in-window refused by both, aged-out accepted
by both and landing durably — and checks durability in the segment's own coordinates, because
the first draft compared a base epoch against a record count and failed against a correct
build.

Three reversions, each firing a different assertion: the sealer pruning by batch sequence
again (the retry is refused, and the engine's own message reads "the sink has already committed
this idempotency key at epoch 1, while the base accepted it as new. The two idempotency windows
disagree"); admission never pruning (the precondition fails, the base holding 12 identities
against a declared window of 3); admission not refusing duplicates (the in-window retry reaches
the sink and returns a window disagreement rather than a clean duplicate).

---

## T01 — Repair served certification and wait ownership

> **Target T01.1:** Stats, append, full-report and mixed legacy/two-phase reads complete under their forced interleavings, and every returned row equals the independent fold at its stated visible anchor.

> **Target T01.2:** Every logical keyed read acquires B at most once, every joined reader waits with no O/B/P/V/C/S guard, and a successful join consumes its own exact-anchor result without a second reconstruction.

> **Target T01.3:** Cancelling owners or abandoning wait tickets releases bounded sharing capacity and counts every capacity refusal while preserving the exact prior absence and generation ownership.

> **Target T01.4:** The Pending proof distinguishes served-value correctness, certification and generation ownership, and each reversion transcript claims only the hazard it actually witnesses.

### Status

A10-02, A10-03, A10-04, A10-05 and A10-12 landed on `c10/02-read-safety`. A10-01 landed
earlier, inside T00, because the baseline depends on it. Container gate at `0ff16a4`: **1,059
pass / 0 fail / 7 ignored**, clippy and fmt clean, `make reproduce` exit 0.

| SHA | What |
|---|---|
| `12d87dd` | three read-path hazards, each witnessed rather than argued (A10-02/03/05) |
| `c7d3955` | a joined read uses the answer it waited for, and waits holding nothing (A10-04) |
| `0ff16a4` | A10-12's claim separated, and three defects Host C's run exposed |

### A10-02 — a report certified at one anchor and copied at another

The split is along the line between what moves and what does not: a plan's shape is a property
of the circuit and is decided without a lock; whether the view is full and how far it has been
advanced changes on every append, so `report_certified_at` is called only with the guard held
and its answer used without releasing it.

**Two tests, and they witness different things — T01.4.** The unlatched one shows that a build
deciding the path from `is_full()` alone claims `report-from-view` for an anchor the view has
passed; it does **not** witness a value divergence, because in that build the certification
simply fails and the fold answers correctly. The latched one does. `report_race_hook` parks the
reporting thread between the shape decision and the guard, and the append lands while it is
parked:

```
assertion `left == right` failed: the report answered at anchor 15 with the values the view
holds AFTER an append that moved it past 15 ...
  left: [.., [Some("1"), Some("777209"), Some("15")], [Some("2"), Some("-776789"), Some("15")], ..]
 right: [.., [Some("1"), Some("209"),    Some("15")], [Some("2"), Some("211"),     Some("15")], ..]
```

A 777,000 transfer visible inside a reply stamped with an anchor before it.

### A10-03 — the legacy read could join a flight it was blocking

`Rev::read` takes `&mut self` for its whole body; waiting on another reader's flight parks a
thread holding the view exclusively, and the owner needs that same `&mut Rev` to publish. The
old code looped on `Join` with a comment arguing the case was unreachable — true of a
single-threaded caller, false of a public function reached through a shared `Mutex<Runtime>`.
`ReadMode::Alone` says the caller cannot wait, and the `Join` arm is **gone rather than made
unreachable**, because "cannot happen on this path" was the claim that was wrong. Reverting:

```
`Rev::read` did not return within 10s ... it is waiting for a `finish_fold` that needs the
very view it is holding — the deadlock of A10-03. It must fold alone instead.: Timeout
```

### A10-04 — the joined answer was discarded, and the outer lock was held across the wait

The `Wait` arm threw away `w.wait()`'s value and went round the loop; the retry re-entered
`answer_from_view`, which acquires the base — one logical read, two acquisitions of B, and a
join costing a wait *plus* a full second pass. Using the answer needs one thing it did not
carry: a key the base has never posted to must produce **no row**, not a row containing zero,
and a joined reader that received only the value would have to go back to the base to learn
which. A flight now publishes `Joined { answer, base_rows }`.

The second half was going to be a deletion. `Serving for RwLock<RevEngine>` holds O shared
across the wait, and `main.rs` serves from `Arc<RevEngine>` — so I removed it as dead and
corrected `SPEC-ENGINE.md`, which claims that type "is what the daemon serves every session
from". **The build corrected me**: `bench.rs` constructs one for the hosted daemon, and the
sweep calls `reseed`, which takes the same lock exclusively. The hazard is live. So the retry
loop moved out of `query` into its callers — `query_step` is one attempt and returns a join
rather than waiting on it — and the wrapper takes its guard, runs a step, drops the guard, and
only then waits.

Three reversions, each on a different assertion: discarding the answer fails the source guard
on `resolve_join`; not publishing `base_rows` makes an account with no history come back as a
row containing zero; taking the wrapper's guard before the wait fails the scan that requires
the guard's scope to close first.

### A10-05 — an abandoned wait ticket kept its place

`begin_read` reserves a place under the completion's lock and `join` gives it back; a dropped
ticket gave nothing back, so `MAX_WAITERS` abandoned readers refuse every later join against a
queue that is empty. Taking that lock in a destructor is safe here and nowhere else in this
engine — **F is a leaf** and is never held while anything is acquired, which is why
`FoldTicket`'s destructor deliberately does not take the view. A declined join is counted under
its own reason, `joins_declined_by_caller`: an overloaded flight table, a full waiter list and a
caller whose lock discipline forbids the wait are three findings with three remedies.

### A10-12 — the Pending claim, separated

"Between `begin_read` and `finish_fold` the slot is `Pending`" is a property of one case. A key
whose entry is `Present` at an anchor that cannot answer this reader keeps that entry, so the
slot holds a value during the flight and a third reader inside that interval is served a hit
while the reconstruction is out. The three claims are now asserted separately: the slot, the
certification (a hit equal to an independent fold), and the flight's own answer.

---

## The Host C baseline — the A=A control

Run at `e29a0256`, 5 measured replicates per arm, both working points, AB/BA balanced, 28
replicates at about 105 s each. Every section ran; both arms completed 5/5.

| point | level | baseline | control | Δ | rel | pooled MAD | MADs |
|---|---|--:|--:|--:|--:|--:|--:|
| full | 6r3w | 152,826 | 153,485 | +659 | 0.43% | 637 | 1.04 |
| full | 9r5w | 160,955 | 160,843 | −112 | −0.07% | 415 | 0.27 |
| full | 12r6w | 163,752 | 163,982 | +230 | 0.14% | 1,436 | 0.16 |
| partial | 6r3w | 143,367 | 143,114 | −253 | −0.18% | 872 | 0.29 |
| partial | 9r5w | 145,439 | 146,640 | +1,201 | 0.83% | 1,732 | 0.69 |
| partial | 12r6w | 149,104 | 148,705 | −399 | −0.27% | 733 | 0.54 |

**The harness's noise floor on Host C is 0.83% at worst, about one pooled MAD.** The gate fires
at ≥10% *and* ≥3 MADs, so it has roughly an order of magnitude of headroom. This stage has no
pass line; the table above is what every later C10 result is scored against.

Three findings from the run itself:

* **`flights_that_fell_behind` is 0 in every level of every replicate**, on a 10-core machine
  as in the container. `answer_from_view` holds the base shared across the fold and `append`
  needs it exclusively, so no epoch can be applied while a keyed read is in flight. The anchor
  gap is entirely `applied − anchor` at arrival, 4.6–5.7 epochs, and none of it is caused by
  folding. **T04's deferred merge is being asked about a window that is empty by construction
  on this build**, which is a result the phase diagram has to carry.
* **`c10-rwlock` on Darwin: 200/200 writer-preferring**, against 170/200 in the container.
* **The partial point contends differently.** At the full point the tails are `b wait`; at
  partial 12r6w they are `v wait` — one replicate has seven consecutive `hit` rows waiting
  3,500–5,900 µs on the view with zero base wait. Invisible at the 2,500 budget, which is the
  only point cycle 9 measured.

**A10-19 is confirmed repaired on the machine that found it**: `rev_metadata_2x_budget` reads
34,165 allocations on Host C, exactly the committed budget, against 44,165 before.

---

## Material facts not covered by the work order

Recorded as they are found; these are not restatements of A10-01…20 or F-10-01…13.

**MF-1 — a source guard was reading another guard's source, and passing on it.**
`daemon.rs`'s `the_read_path_takes_the_base_shared_and_the_append_takes_it_exclusively` split
`rev_engine.rs` on `"fn append(&self, rows: Vec<Row>"`. `rustfmt` broke that signature across
four lines long ago, so the only occurrence of that string in the file was inside the
*lock-order test's own marker list*. The split therefore landed in a test module and the
`contains("TimedWrite::acquire")` beneath it was satisfied by that module's prose. The arm has
asserted nothing about `append` since the signature was reformatted, and it went on passing
because a guard that reads source can be satisfied by the source of another guard. It surfaced
only because cycle 10 replaced the marker list with a derived scan and the string disappeared.
Repaired at `f78472f` with the real multi-line marker, and with the body required to be
located rather than defaulted to the rest of the file by `unwrap_or(len)` — a default that
turns "the function was not found" into "the whole file, which certainly contains the string I
am looking for". Proved by reverting `append` to an untimed write guard.

**MF-2 — the benchmark had been reading a column the server renamed in cycle 9.**
`report_slow_reads` asked for `unaccounted_us`; the server has called it `rounding_us` since
cycle 9. The lookup helper defaulted a missing name to `0`, so **every cycle-9 slow-read table
printed a zero remainder**, and that zero was then read as "nothing outside the measured
phases" — the exact claim A9-F07 had already retired, arrived at a second time by a different
route. A name the benchmark asks for and the server does not have is now `n/a` with a warning,
and the harness refuses a run whose log contains that warning.

**MF-3 — a source-level lock-order guard can be satisfied by a comment describing the bug.**
The derived scan, before comment stripping, passed against the reverted deadlocking build
because the repair's own explanatory paragraph named `self.base()` above the acquisition and
moved the position the scan compares. Any guard in this project that locates code by
`str::find` over `include_str!` has this failure mode; the two in `rev_engine.rs` now strip
`//` comments before reading positions, and the rest have not been audited for it.

**MF-4 — three guards written this cycle were vacuous, and each was found by reversion rather
than by review.** The slow-read width check iterated rows and there were none; the lock-mode
reconciliation summed `0 + n == n` inside a test that drives an engine which never takes the
lock at all; the completion-lock counter was a process-wide static and passed alone while
failing under `cargo test`'s parallelism. Two were repaired by moving the property out of the
test — into a generated column table and into the lock's own declaration — and the third by
scoping the counter to the view under test. The general shape: **a guard that reads a counter
must be able to isolate it, and a guard that iterates a collection must assert the collection
is non-empty**, or it reports on whatever else the process did. Nothing in this repository
currently enforces either rule.

**MF-5 — a portability defect that only the machine it was written for could find.** The
harness self-test's occupied-port case passed in the Linux container and failed on Host C. The
cause was the test, not the check: it made two connections to a listener that never calls
`accept`, and on Darwin a connect to a socket whose backlog is full is refused, so the second
connect failed and the check correctly reported a port it could not reach as free. The same
class caught the executor twice more in one session — the deadlock witness's reverted form
hung rather than failing, and the completion control's reverted form hung too. Each was code
written against the platform in front of it and true only there.

**MF-7 — a toolchain override that is invisible where it is written.** Both Host C scripts
exported `RUSTUP_TOOLCHAIN=stable`. In the cloud container that is a no-op: the pinned 1.95.0
cannot be resolved without egress, and `stable` *is* 1.95.0. On Host C the pin resolves and
`stable` is 1.97.1, so the override silently swapped the compiler — **the whole cycle-10
baseline and every gate the author ran were built with a toolchain the tree does not pin**, and
no line of output said so. The general shape: an environment override written on the machine
where it changes nothing is untested by construction, and the first host it reaches is the one
it changes. All three scripts now prefer the pin, fall back to `stable`, and print which.

**MF-8 — `make reproduce` compared columns its own manifest says are not comparable.**
`results/MANIFEST.csv` classifies E18 as `toolchain-scoped` and defines that as "allocation
*counts* are exact across hosts, allocation *bytes* are not". The gate is
`git diff --exit-code -- results/`, which compares every byte of every file, so Host C failed on
the byte columns every time it ran — masked, until this cycle, by a louder failure in the same
file. The counts are now in a `byte-deterministic` file that is diffed; the byte columns stay,
labelled with their host, excluded. **A classification nothing enforces is a comment.**

**MF-9 — the executor wrote the same class of bug it was repairing.** The A10-12 test matched
on `view(&rt)...begin_read(..)` directly. A temporary in a `match` scrutinee lives to the end
of the whole `match`, so the view guard was held inside every arm, and an arm that took the
view again self-deadlocked on a non-reentrant `Mutex`. It hung the container suite until it was
killed. `read_split`, three hundred lines above it, binds the outcome to a local first for
exactly this reason and says so. Recorded because the repair is not "be more careful": it is
that a `match` on a lock-taking expression is a hazard the codebase has already met, and
nothing in the tree prevents the next one.

**MF-6 — the device bridge to Host C is a Linux VM, so it cannot validate Darwin behaviour.**
The `mcp__remote-devices__device_bash` shell reports `platform.system() == "Linux"` and
`hasattr(fcntl, "F_FULLFSYNC") == False` while sitting on the author's Mac with its folders
mounted. It is excellent for reading files, checking sizes and verifying bundle hashes — all
of which it did in this cycle — and it is **not** a Darwin execution environment. Any future
work order that assigns a Darwin-specific check to the executor rather than to the author is
assigning it to a machine that cannot run it.


---

## T02 — Make public query and conservation claims truthful

Branch `c10/02-read-safety`, commit `73fe782`. Closes A10-06 and A10-07; narrows LC-41.

**Target T02.1 (verbatim):** *Runtime installation refuses every unsupported reachable graph in
the guard corpus while all supported keyed balance circuits still agree with an independent fold
after advance and eviction.* — **done.**
`nilestream_core::rev::tests::every_unsupported_reachable_graph_is_refused_at_install` and
`a_supported_keyed_balance_still_agrees_with_an_independent_fold_after_advance_and_eviction`.

**Target T02.2 (verbatim):** *Every accepted conserve declaration has the grouping semantics
admission actually enforces, and unsupported or conflicting partition declarations are refused
with a named diagnostic instead of being silently overwritten or ignored.* — **done.**
`niles_lang::resolve::conserve_grouping_tests`, nine cases.

### What A10-06 actually was

`Runtime::install` read `circuit.outputs` and stopped. An output that was a keyed `sum` with a
key installed, **whatever fed it**.

The reason this is not an ordinary missing refusal is what happens after `install` returns:
nothing reads the circuit again. Every answer comes from `Base::reconstruct`, whose whole
signature is a key and an epoch — the production implementation
(`proto_engine::ledger`) sums every posting for `(acct, cur)` and has no way to be told about a
predicate. So a `Source → Filter → Aggregate` circuit did not execute the filter badly. It did
not execute it at all, and the view answered as though the filter were absent: no error, no
approximation, the unfiltered sum. **That is exactly the number an answer-level assertion
expects**, which is why the defect survived four audit cycles with the whole suite green.

`install` now walks every node reachable from each output and accepts only one keyed
`sum`/`count` directly over one immutable base source. Two new refusals name the node and the
reason: `Unsupported::Upstream` for a stage that would be dropped, `Unsupported::DerivedSource`
for a source this runtime does not hold.

**A documented claim that the code did not make true.** `thesis_drift::theorem_4_1_names_its_fragment`
asserts that Theorem 4.1's fragment is a property of the artifact, and its evidence is that
`Runtime::install` refuses a join. It read `install`'s doc comment, which said so. The function
refused a join *as an output* and installed one *under an aggregate*. The comment was true of
what the function meant; the test was checking the comment.

**Cost, recorded rather than tuned away.** The walk allocates a `BTreeSet` and a `Vec` per
output: `ledger_seeded` moves 150,254 → 150,263 allocations in `E18-counts.csv`, nine per
install. Per operation it is unchanged at 3.8. Both figures reproduce exactly — HEAD measured
150,254 three times in a detached worktree, this tree 150,263 three times.

### What A10-07 actually was

`conserve per (..)` was parsed, its columns name-checked against the relation, and then
`conserve_keys` reached **no consumer anywhere below the compiler**. Admission conserves per
(transaction, currency) unconditionally. So `conserve per (txn, desk)` compiled clean, reads as
a promise of per-desk segregation, and bought nothing at all.

A repeated clause was worse. `relation_info` assigned each `Conserve` rule over the last, so
`conserve per (txn, cur); conserve per (desk);` compiled and the rule a reader would name as
the ledger's was the one discarded — silently, with no diagnostic and no record.

- **NL0224** refuses a second `conserve` clause and points at the first.
- **NL0225** refuses a grouping the seal does not enforce, and says what it does enforce.

The accepted set is the enforced set: exactly the relation's sole `TxnId` column and its sole
`Currency` column, compared **as a set**, because `(cur, txn)` names the same partition and
refusing it would be a statement about writing order. A ledger with no `Currency` column, or
with two, is refused by shape rather than by key — the keys may be exactly what the author
meant on a relation that cannot carry the rule.

**The check is on types, not spellings, and that is load-bearing.** The H-S8 falsifier of
§9.11.1 conserves a non-monetary quantity through a `Currency` column that is not called `cur`.
A rule keyed on the literal name would refuse the one file in the corpus that tests whether
this machinery is general. The fixture in `conserve_grouping_tests` is deliberately
domain-neutral for the same reason: the falsifier's standing claim is that no source file under
`crates/niles-lang`, `crates/niles-ir` or `crates/nilestream-core` contains a word from its
domain, and the first draft of that test broke it. The existing guard caught it.

**This refuses; it does not implement segregation.** Per-group zero sums are weaker than
forbidding cross-group flow — two opposite cross-group legs in one transaction cancel inside
each group and the partition is still violated. A real rule needs an admissible-edge policy
with a stated allowance for FX and linked legs, which is LC-41 and the author's to choose. The
work order's instruction was followed exactly: refuse the unsupported declarations first, and
do not invent a policy.

### Revert witnesses

Every guard was proved to fail on the reverted change in a disposable `git worktree`, and every
witness is an assertion failure rather than a compile error.

| Reverted | Guard | What it printed |
|---|---|---|
| the graph walk in `install` | `every_unsupported_reachable_graph_is_refused_at_install` | `installed a circuit with a filter between the source and the aggregate` |
| W3b (the grouping check) | four `conserve_grouping_tests` | `` `conserve per (txn, acct)` … must not compile. got [] `` |
| the silent overwrite restored | `a_second_conservation_rule_is_refused…` | `` got ["NL0225"] `` — NL0224 goes silent and the *wrong* rule is checked |
| the derived exclusion hand-written | `the_reproduce_diff_excludes_exactly_what_the_manifest_says_is_incomparable` | three separate refusals (no variable; literal `:!`; class unselected) |

---

## Correction to §"1. Flights never fall behind" — dated 2026-09-08

**The paragraph above that says the deferred-merge window "is empty by construction" is wrong,
and this correction supersedes it.** The original sentence is left standing as written; this is
the amendment, per the audit protocol.

The error was reading two counters as one.

- **`flights_that_fell_behind`** increments when the view's `applied` moves *between* a
  flight's begin and its landing.
- **`pinned_installs`** increments when the landing anchor is below `applied` **at all**, which
  includes every flight that *began* behind and never moved.

The suffix a deferred merge folds is `(anchor, applied]` **at landing** — the second counter,
not the first. The zero is real and structural, and it means no flight falls behind *while
folding*. It says nothing about whether there is a suffix to merge.

Fact 3 of this same report already recorded the number and the earlier paragraph did not carry
it: **45,282 pinned installs against 45,681 installs — 99.1% — with an arrival gap of four to
five epochs.** The merge window is open on essentially every install. It is opened by *arrival
lag*, not by fold duration: readers sample a frontier a few epochs behind the view's applied
point. Different mechanism, different cost model, and a real non-empty suffix.

So **T04.1 is buildable** and the disposition offered to the author on the strength of the
original paragraph — record T04 as a negative experiment — was withdrawn before it was acted
on.

**One thing the original zero could not have told anyone, and now can.** Nothing in the tree
ever asserted `flights_that_fell_behind` could be non-zero. An instrument reading zero because
it is dead reads exactly like an instrument reading zero because the event does not occur, and
the baseline reported that zero across 28 replicates on two hosts with no evidence of which it
was. `nilestream_core::rev::deferred_merge_window_tests` closes that: one test drives the
interleaving through the two-phase API directly and requires the counter to move; the other
states the served path's exclusion as arithmetic over `applied` rather than as an observed
schedule. Both land in this commit, and they are worth having whatever T04 becomes.

---

## T04 — Bounded, certified deferred merge

Branch `c10/02-read-safety`, commits `7b6b704`, `dfac93f`, `bca7907`. **The author chose to
build it, after the correction above withdrew the negative-experiment disposition.**

**Target T04.1 (verbatim):** *Every eligible generation-owned late landing within the
preregistered epoch and physical-work caps merges exactly the keyed deltas in (a,e], every
owner and same-anchor waiter still receives the answer at a, and unavailable or over-budget
suffixes remain honestly pinned or uninstalled.* — **done.**
`nilestream_core::rev::deferred_merge_tests`, nine cases, plus
`a_merged_entry_answers_the_frontier_it_landed_at` beside the pin it replaces.

**Target T04.2 (verbatim):** *Against the measured pinned-control figures from c10-merge.sh,
deferred merging reports its gap, visited-row, reuse, writer and memory costs and makes a
performance claim only when the preregistered ≥10% and ≥3 pooled-MAD gate passes.* —
**instrument landed and smoke-run end to end; the measurement is the author's.** No
performance claim is made in this report.

### What the merge is

A flight folds the prefix ending at its anchor `a`. If the view advanced to `e > a` while it
was out, the landing entry has not seen `(a, e]`, so `install` **pins** it: it serves only
`[a, a]`, and the next reader at the frontier reconstructs the whole key. The merge folds this
key's deltas over `(a, e]` into the landing value and installs at `e`, so the entry lands
current.

**What does not change is what anybody is told.** The owner and every same-anchor waiter
receive the fold's own value at `a`. The merge writes the view; it does not answer the reader.
Reverting that single line fails four tests, one of them — `f01_stale_hit_is_not_promoted` —
several cycles older than this work.

### Preregistered bounds, and three refusals kept apart

`MergeCaps` is fixed in code before anything was measured: **32 epochs** (the measured arrival
gap is 4.6–5.7 with a maximum of 17, so this covers the distribution about twice over and
still refuses a genuinely historical read) and **4,096 rows** (an epoch is not a constant
amount of reading — a delta list is the whole epoch's rows across every key — so the physical
work is bounded separately).

Over either cap the landing **pins**. It is not partially merged: a partial merge installed at
`e` is a value missing part of its own history under a current stamp, which is exactly what
the pin exists to prevent.

The third refusal is the one that would have been silent. **`deltas_at` reports an epoch the
base no longer holds as *empty*, which is indistinguishable from an epoch in which nothing
happened**, so a merge across a compacted prefix would drop real money and report success.
`Base::deltas_available_from` makes it a refusal instead. It defaults to `0` because every
base in this repository is fully retained — which is what F4 promises — and a base that
compacts must override it.

The three are counted apart: an over-budget suffix is a tuning question, an unretained one a
retention question, and merging switched off is neither.

### The ablation is one build and two settings

`NILESTREAM_MERGE_CAPS` selects the arm, so a difference between the arms cannot be a
difference between two compilations. That is also the risk — nothing about the *build*
distinguishes them — so `--merge-arms` refuses any replicate whose transcript does not carry
the merge line with the caps that replicate was launched with. **A run whose arm rests on the
flag having been typed correctly is not an ablation.** The selector refuses an unparseable
value rather than defaulting, for the same reason.

`c10-merge.sh` is seven lines and a page of reasons: every other refusal the measurement needs
is already in `c10-baselock.sh` and already proved by its `--self-test`, and a second script
would be a second source of truth for questions that have one answer. Four new self-test arms
cover the merge line — the merging log, the control's log, a caps mismatch, and a build that
does not report the merge at all.

### The container smoke run — not a result

`--warmups 0 --measured 1 --seconds 3` on a 2-core VM. Exit 0, both arms labelled, every
section ran. The counters are what it exists to show:

```
merge arm   : 28219 merged of 28219 late landing(s) (100.0%); rows visited 236816,
              epochs merged 118408; refused 0/0/0; caps 32 epochs / 4096 rows
                per merge: 8.4 rows, 4.2 epochs
              flights: pinned_installs 0, deferred_merges 28219
pinned arm  : 0 merged of 38946 late landing(s); refused 38946 over-epochs;
              caps 0 epochs / 0 rows  [PINNED CONTROL: merging off]
```

Three things a reader should take from it, none of them a performance claim:

* **Every late landing is eligible.** 100% merged, zero refusals of any kind, and
  `pinned_installs` falls to **0** in the merge arm. The suffix is 4.2 epochs against a cap of
  32, so the epoch cap is not binding on this workload — which is the cap doing its job, since
  it exists to refuse the historical read rather than the late-arriving one.
* **The merge is cheap in the units it spends.** 8.4 delta rows per merge, which is 4.2 epochs
  at the two conserved legs each epoch seals. That is the number to compare against a
  reconstruction, and the comparison is the author's run.
* **The price is paid under V.** The suffix is walked inside the second view hold, which is
  why `install_hold` is measured around it. `v hold2` in the slow-read table is where a cost
  would appear.

### Revert witnesses

Five, each an assertion failure in a disposable worktree, none a compile error.

| Reverted | What fired |
|---|---|
| the merge not attempted | four merge tests; `the landing was eligible` |
| the merged value returned to the caller | both clause-2 tests, `a_merged_entry_answers…`, **and `f01_stale_hit_is_not_promoted`** |
| the row cap removed | `a_suffix_over_the_row_cap_pins_and_still_reports_what_it_read` |
| the retention boundary removed | `a_suffix_below_the_retention_boundary_pins_rather_than_dropping_money` |
| ownership ignored | `a landing that owns nothing must not merge` |

### A test that changed, and why that is a finding rather than an adjustment

`a_pinned_entry_read_above_its_stamp_still_reconstructs` went red when the merge landed. On
its fixture the landing is two epochs behind, so it now merges: the entry installs current and
the following read is served from it. **The value is right in both arms** — 100 at `a`, 500 at
`e` — so this is a change of policy, not of correctness, and the fixture was written when
pinning was the only landing. It now sets `MergeCaps::OFF` and asserts the pin it is named
for; `a_merged_entry_answers_the_frontier_it_landed_at` is the same fixture with merging on.
The two together are the smallest statement of what T04 does.

---

## Further material facts

**MF-10 — the executor wrote MF-4's bug, having written MF-4.** The arm selector's first test
set `NILESTREAM_MERGE_CAPS` to each bad value in turn and read it back through the function
under test. `cargo test` runs in parallel, so an unrelated test built a `RevEngine`, saw
`"8:64:2"`, and was killed by the refusal — which was behaving correctly. MF-4 states the rule
this broke: *a guard that reads process-wide state must be able to isolate it.* The repair is
structural rather than careful — the environment is read in exactly one place and the decision
is a pure function of a string — because "be more careful" is not a repair.

**MF-11 — a probe that measured the value it was about to overwrite.** `pick_toolchain`, added
this cycle to fix MF-7, asked `rustc --version` with the *inherited* environment and, on
success, concluded that the tree's pin resolves and unset `RUSTUP_TOOLCHAIN`. A shell that
already had the override set — which is how the container builds at all, the pinned 1.95.0
being unfetchable without egress — reported "the tree's pin resolves on this host", removed
the override that made it true, and every `cargo` below failed with `toolchain 1.95.0 is not
installed`. **On Host C the reported answer is unchanged, so only the container could find
it**, which is the mirror image of MF-7, where only Host C could. A fix for a
host-dependence defect acquired a host-dependence of its own in the opposite direction.

**MF-12 — an edit script that asserts late and writes late loses the edits before the
assertion.** The repair for MF-11 was written in the same script as an unrelated correction;
that second edit's assertion failed, the script exited before its single `write`, and the
first repair was silently lost. The commit describing the repair therefore did not contain it,
and the container reproduced the original failure with the original message. Recorded because
the shape is general and this cycle's edits are all of this form: **a multi-edit script must
write each edit as it makes it, or verify afterwards that every edit is present.** Nothing in
the working method currently requires either.

**MF-13 — a refusal that did not stop the thing it refused, found by Host C at the cost of a
40-minute run.** `c10-merge.sh`'s first real invocation met a leftover worktree from the
earlier `--baseline-only` sweep, still at `e29a0256`. `build_arm` refused it and printed the
exact command to remove it — and the script then ran every warm-up and all five measured
replicates against the binary already sitting in that directory. `e29a0256` predates the merge
by two commits, so the arm-label check refused all twenty replicates for reporting no merge
counters. **Every one of those twenty messages named the wrong cause**, and the right one was
two hundred lines above them.

Two things are worth separating here.

*The check added for T04.2 worked.* Nothing was scored, and nothing was scored *silently* — a
harness without the arm-label refusal would have measured a two-commit-old binary in both arms
and reported a clean null result with five replicates behind it. That is the failure the
refusal exists to prevent and it prevented it.

*The harness was still wrong.* `skip` records a section and returns, which is right when the
missing section leaves the rest of the transcript meaningful — a checkpoint probe that could
not run does not invalidate the replicates below it — and wrong for anything the measurement
is made of. This is the paired adapter gate's shape a third time: **a refusal that does not
stop the thing it refuses is a note, and notes do not gate.** `fatal` now stops, and the two
are distinguished where they are defined.

The guard is in two halves because either alone is weak: `fatal` really exits, and *no build
failure anywhere in the script is still wired to `skip`* — a behavioural test alone would pass
on the next `|| skip` somebody writes. Witnessed end to end in the container against a
deliberately stale worktree: the run now stops at section 1 with one refusal and one cause.
