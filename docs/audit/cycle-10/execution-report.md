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

* The baseline is the checked-out build, resolved to a full SHA and printed. A dirty
  repository is refused; a worktree whose HEAD is not the SHA under test is refused.
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

