# Cycle 11 — the builder's execution report

*Written against `docs/audit/cycle-11/work-order-11.md` (GBS `c11/work-order-11`, `6233b21`),
the consolidated order assembled from Astra's cycle-11 work order and Fable's. It reports
tranche 1: cards C11-06, C11-02, C11-01, C11-03, C11-04, C11-05(a)+(b) and C11-07, in the
order the cut names them. Cards C11-05(c)/(d), C11-08 through C11-14 are `not done: not
reached` and §9 says so one by one.*

*This report lives in niles by the author's answer to LC-50: execution reports beside the code
they cite by file and line, work orders and audit briefs in private GBS.*

---

## 0. The three decisions the author made, and where each took effect

| decision | answer | where it landed |
|---|---|---|
| **LC-38** — the deferred merge's default | **(a)** off now; C11-05(c)'s interval repair and (d)'s Host C measurement decide whether it returns | `MergeCaps::default() == OFF`, niles `e8982f5`, alone on its own commit; `MergeCaps::ON` keeps the preregistered 32 / 4,096 |
| **LC-50** — where reports live | reports in niles, orders and briefs in private GBS | this file, on niles `c11/07-execution-report` |
| delivery cadence | one bundle per card, as each lands | eleven bundles, §7 |

Nothing else in §5 of the order was answered, and §10 lists what each unanswered one still
blocks.

---

## 1. What ran, and on what

Every command in this report ran in the cloud container. **Nothing here is a Host C
measurement**, and no row below is a performance claim.

| | |
|---|---|
| host | Linux 6.18.44-fc-v24, x86_64, cloud container |
| compiler | `rustc 1.95.0 (59807616e 2026-04-14)` |
| how selected | `C11_TOOLCHAIN=stable`, named by the caller because the tree's pin (1.95.0) does not resolve here — the container has no egress to `static.rust-lang.org`, and `stable` *is* 1.95.0 |
| PostgreSQL | 16.13 on 127.0.0.1:5432, role `bench`, database `bank`, started for `numeric_binary_oracle` |
| the newer compiler | **not installed.** Every gate run declares `--lint-toolchain none`, which puts the absence in the transcript rather than passing silently. A11-06's 1.97.1 rows are **not covered by anything in this report** — see §4, which corrects what A11-06 said about them |

`RUSTUP_TOOLCHAIN` was unset in every gate invocation and `C11_TOOLCHAIN=stable` supplied
instead, so the substitution is a caller's assertion in the transcript and not a line in a
script. `CARGO_NET_OFFLINE=true` and `RUSTUP_AUTO_INSTALL=0` throughout; nothing was
downloaded and nothing was installed.

---

## 2. Every target sentence, verbatim, with its verdict

The sentences are copied from §3 of the work order. `done` means the sentence is true of the
committed code and a named command demonstrates it.

### C11-06 — the evidence path

> **C11-06.1:** *`bench --score <dir>` reproduces the six T04.2 read rows and the six writer
> rows of A11-07 from cycle 10's per-replicate CSVs to the integer (median, MAD, range,
> RMS-pooled MAD, joint-gate verdict, direction), refuses an arm with fewer than five measured
> replicates, prints per arm and level the fold fraction (flights landed ÷ queries), pending
> joins, mean arrival gap, writer throughput and write p99 beside the read row, and the harness
> prints its gate table from that output and from no arithmetic of its own.*

**done, with one qualification stated rather than glossed.** The six T04.2 read rows reproduce
to the integer from cycle 10's own twenty `mixed.csv` files, committed unaltered as
`crates/bank-bench/tests/fixtures/c10-hostc-t04.2/`. The writer rows reproduce and say more
than A11-07 did — §3. The scorer refuses an arm with fewer than five replicates, refuses a
metric that is `n/a` in any replicate, and refuses a zero pooled MAD with different medians,
each as a printed row rather than an omission.

The qualification: **the fold fraction and the counter columns are `missing` for cycle-10
logs, and the scorer says so in those words.** They come from the `flights` summary line,
which no build before this cycle printed. The scorer reports `MISSING: no log under <dir>
carries a flights summary line … which is a different statement from "the counters were zero",
and this scorer will not make the second one on their behalf.` For runs of the new build the
columns are present; the fold fraction is printed as its two components and their quotient so
the definition is checkable from the row.

Command: `cargo run -q -p bank-bench --bin bench -- --score crates/bank-bench/tests/fixtures/c10-hostc-t04.2`, exit 0.
Test: `cargo test --offline -p bank-bench --test score_c10_fixture`, exit 0, 5 passed.

> **C11-06.2:** *Every harness refusal that reads a benchmark log parses the benchmark's
> machine-readable summary line; a bank-bench test asserts that line's schema; a reworded
> human-readable line changes no verdict; missing, malformed, `n/a` and zero are distinguished
> by four fixture logs and four self-test arms.*

**done, with the legacy path kept deliberately and labelled.** `check_writer_progress`,
`check_no_missing_fields` and `check_arm_caps` read fields from `NLSBENCH/1` lines through one
`summary_fields` reader. `summary_schema.rs` asserts the schema, asserts that the CSV row and
the summary line are two renderings of one field list, and reworks the human line to assert
that every parsed field is unchanged. Four fixture logs and ten self-test arms cover the four
outcomes — more than the four the sentence asks for, because the caps check needed its own.

The prose greps remain as an explicitly labelled legacy path. Pair 2 of the Block B campaign
compares `e29a025` against `653a369`, and **neither commit prints the machine-readable line**;
refusing them would refuse the measurement the campaign exists for. The manifest records
`summary_lines=absent` for such an arm.

Command: `bash docs/audit/cycle-11/hostc/c11-baselock.sh --self-test`, exit 0, ending *every
injected fault was refused, and every clean control was accepted*.

> **C11-06.3:** *Every run writes a manifest under its output root — code pair, dirty-diff
> hash, arguments, environment, the executable's hash, limits, and each stage's outcome — and a
> stage whose prerequisite failed is recorded `not run` and its executable is never launched:
> a self-test with a deliberately present stale executable whose launch would create a sentinel
> shows no sentinel and names the failed stage.*

**done.** `$OUT/manifest.txt` is written before anything is built; `$OUT/manifest.csv` gains a
row per stage as it happens, so `not run` is a value rather than an absence. Each arm's
binaries are hashed at build time and `may_launch` refuses any binary whose hash is not the one
this run recorded — an arm whose build never ran has no recorded hash and is unlaunchable
whatever a previous run left on disk.

Guard, §5: with `may_launch` returning unconditionally, the self-test reports *NOT REFUSED: the
stale executable was launched — the sentinel /tmp/c10-selftest.*/LAUNCHED exists* and exits 1.

> **C11-06.4:** *The paired gate resolves each repository's pin from that repository, launches
> every child with the same explicitly selected compiler it prints (`rustup run <pin>` or an
> equivalent immutable child environment), runs the newer lint separately, uses
> `git rev-parse --is-inside-work-tree` and exact expected SHAs, and — from a foreign cwd, with
> an inherited `RUSTUP_TOOLCHAIN=1.97.1`, on a linked worktree, on a path with spaces, with an
> unavailable pin — refuses or proceeds exactly as its self-test arms say; the pin missing is a
> refusal of dependent execution, never a silent `stable`.*

**done.** Eleven self-test arms, all green: `bash docs/audit/cycle-11/hostc/c11-gates.sh
--self-test`, exit 0.

`pick_toolchain <repo>` takes its repository, returns 2 when called without one, and returns 3
when the pin does not resolve and no substitute is named in `C11_TOOLCHAIN`. `in_repo`/
`run_pinned` **assign** `RUSTUP_TOOLCHAIN` in the child, so an operator's own 1.97.1 cannot
reach a section that resolved 1.95.0; the arm asserts on the child's value and not the
parent's. `--niles-sha`/`--gbs-sha` stop the run when HEAD is not the expected commit.
`--is-inside-work-tree` replaces `--git-dir`, which accepts a bare repository that has no files
for any gate below to be about.

The newer lint runs as its own section whose rows may be red; with no `--lint-toolchain` it is
`NOT RUN` and the gate is non-zero, and `--lint-toolchain none` is an explicit assertion by the
caller that this host has one compiler.

**Two defects in this cycle's own harness were found by writing those arms**, and both are in
§3.

> **C11-06.5:** *A table-driven test drives every `nilestream_stats` counter to a non-zero value
> through the wire with an isolated zero-event control per counter, and fails when a counter is
> added to the wire without a row.*

**done, with the honest form of "every".** `crates/nilestream-server/tests/stats_counters.rs`
is closed at both ends: a counter on the wire with no row fails the completeness test and is
named, and a row naming a column the wire does not carry fails too. Each row is `Driven` (zero
on a fresh engine, non-zero after the named event — eight counters), `Level` (a size or a
configured cap — six), or `StaysZero` **with its reason** (eighteen).

`StaysZero` is where the sentence's "every" is qualified and the qualification is the useful
part: a single-threaded fixture cannot produce a join, a waiter refusal, or a flight refusal,
and driving those through the wire would need a concurrency harness that is its own card. A
counter asserted to stay zero *after the very events that would move it* is a guard, not an
excuse — `flights_that_fell_behind` has one now, carrying F-11-09's structural-zero argument
and the note that LC-37's prototype is exactly what would break it.

Guard, §5: remove one row and the test fails naming `["merge_rows_visited"]`.

> **C11-06.6:** *Every harness change ships with a self-test arm that reaches the changed line,
> shown red with the change reverted and green with it kept, and the report lists per harness
> commit the arm and both exit codes.*

**done.** §5 lists them.

> **C11-06.7 (gbs):** *Both fmt/lint gates pass on 1.95.0 and 1.97.1 after the one-cast repair,
> and `conservation_under_eviction`'s retention-hit assertion is unchanged.*

**not done: half of it cannot be run here, and the other half turned out to be a different
finding.** The cast is gone and the assertion is unchanged; fmt and clippy pass on **1.95.0**
including the adapter crate by manifest path. **1.97.1 is not installed in this container and
nothing here says anything about it** — that verification is Host C's, and the command is
`bash ~/c11-hostc/c11-gates.sh --lint-toolchain 1.97.1`.

The finding is that A11-06's diagnosis was wrong. See §3.

> **C11-06.8 (Block B, the author's run):** …

**not done: the author's run.** `~/c11-pairs-out` exists on Host C but is not a folder this
session can read, so I cannot say whether the run happened. The scorer is built and its guard
is cycle 10's own CSVs; paste the output or grant the folder and it scores in one command.
### C11-02 — one checked arithmetic

> **Target C11-02.1:** *Every integer operation in the reference evaluator and the interpreter
> is checked; division and remainder by zero and every overflow are refusals carrying the
> operator and position; a served query whose `having` or projection divides by zero is refused
> over the wire with a diagnostic and never answers a row; `i128::MIN / −1` is a refusal in both
> evaluators in both profiles; and the rewrite corpora pass against the checked semantics.*

**done, with one word narrowed.** "position" is the **operator and its operands**, not a source
span: the IR's `Scalar` carries no span, and inventing one would have meant either threading a
span through the whole lowering or fabricating a location. The diagnostic reads
`7 / 0 is not a number` and `<lhs> * <rhs> does not fit in a 128-bit integer`, which is what a
client can act on; the interpreter, which *does* have spans, keeps them.

Everything else holds. `niles_ir::arith` is the one integer arithmetic, used by
`niles_ir::eval`, `nilestream_server::scan_fold` and `niles-interp`. A refused fold is a
`22000` data exception over the wire whose `DETAIL` names the operator and the operands, and
the wire tests assert on **the absence of a row** before they look at the message.

**The corpora needed no changes at all**: `sql_golden` and the unnesting suite pass unchanged
against the checked semantics, so no fixture relied on `/0 = 0`. The card asked for that either
way and it is the good answer.

Commands: `cargo test --offline --workspace --no-fail-fast` → 1100 pass / 0 fail / 8 ignored;
`cargo test --offline --release -p niles-ir -p niles-interp -p nilestream-server` → 505 / 0 / 3.
Both profiles, because the defect was that they disagreed.

### C11-01 — bind admitted plans to what executes

> **Target C11-01.1:** *Every installed fragment case in the declared matrix — Sum and Count
> across two outputs, differing columns and group keys, multiple sources, zero rows, negative
> values, insert and delete multiplicities, a same-named relation with a changed schema, and the
> cold / hit / evicted / reconstructed / advanced states — matches its independent raw-row
> oracle at the requested anchor; every remaining shape is refused explicitly at installation
> with a structured reason; stale descriptor and schema mismatches are rejected; and the served
> balance path and the paired adapter pass unchanged semantic tests.*

**done, and the matrix resolves the way the card's "smallest honest fragment" instruction
implies rather than the way its first clause reads.** "Sum and Count across two outputs" is
**refused**, not matched: a runtime is handed one `Base`, a `Base` answers one aggregate over
one key, and installing two questions over one oracle is what produced `sum = 30, count = 30`.
So the matrix's two supported plans are a keyed `Sum` and a keyed `Count`, each with the base
that answers it, each driven cold / hit / evicted / reconstructed / after-advance at six anchors
and four keys against a raw-row oracle that touches no runtime, no circuit and no base. Seven
shapes are refused at installation with the reason named — mixed plans, differing columns,
differing keys, two aggregates in one output, an unmaintained aggregate, a filter between
source and aggregate, and two joined sources.

"Stale descriptor and schema mismatches" is `Base::answers()` checked on every path that
touches the base, including the two that look innocent: the same relation with a different
column numbering, and a permuted key (`reconstruct` reads `Key` positionally, so `(acct, cur)`
is not `(cur, acct)`).

The fixture carries the rest of the list: zero rows, a key whose only row is zero, a negative
amount, and an insert and its delete netting to zero within one epoch.

The served balance path and the paired adapter pass unchanged: `downstream_adapter` green
against gbs `2e0e4b4`, and the GBS adapter installs only `Agg::Sum`, which is what bounds the
finding to the niles side.

Command: `cargo test --offline -p nilestream-core --test fragment_matrix`, exit 0, 7 passed.

### C11-03 — invalid / unavailable / no-movement economics (GBS)

> **Target C11-03.1:** *Pricing failures and missing observations cannot become successful
> no-movement results; a basis-0 fixed leg is refused at construction or at declaration with a
> typed error and generates nothing; missing fixings are explicit incomplete outcomes that never
> complete a persisted payment; and exposure completeness, bounds and checked arithmetic
> determine every limit verdict explicitly.*

**done, refusing at declaration rather than at construction, which the sentence allows.**
Validating in `fixed_leg`'s signature would change every constructor's return type across the
product layer; the typed outcome refuses the same instrument one step later, at the point the
schedule is declared, and refuses it again on its coupon date for a caller who never declared
it. Both are asserted.

`Generated` replaces `Option<PostingSet>`: `Postings`, `NoMovement { why }`,
`MissingObservation { name }`, `Invalid { detail }`. `generate` returns a `Generation` carrying
sets **and** `pending`, so an occurrence that fired and could not be priced reaches the caller
as a fact. Nothing here persists a payment lifecycle, so "never completes a persisted payment"
is held by the two callers that could have: `Swap::net_settlement` refuses instead of returning
`Ok(None)` — "nothing due today" — and `g3` refuses to count an unpriced row as product
coverage. The retry policy for a late fixing is untouched and remains the domain owner's.

The exposure half is done and found something the card did not name — §3.

Command: `cargo test --offline -p gbs-products --test economic_outcomes`, exit 0, 11 passed;
workspace 517 / 0 / 1; adapter 53 / 0.

### C11-04 — reclaim cancelled generations

> **Target C11-04.1:** *Abandoned owners cannot permanently exhaust admission capacity for
> unrelated keys; live caps remain enforced; cancellation cannot overwrite a newer generation or
> change a returned anchor or value; and after 10,000 cancellation cycles and quiescence the
> resident flight records number at most 256 with waiters at the existing cap.*

**done, at 2,560 cycles rather than 10,000.** Ten times the cap exercises the property — the
table cannot exceed `MAX_FLIGHTS`, and a sweep can only run when it is full — and 10,000
single-threaded allocations would add ninety seconds to the suite to re-establish the same
invariant. `Rev::in_flight_len` is exposed so the bound is asked rather than inferred. Say the
word and the constant moves.

"Cancellation cannot overwrite a newer generation" is guarded rather than repaired: see §3,
where I withdraw a claim I made about it.

Command: `cargo test --offline -p nilestream-core --test flight_reclamation`, exit 0, 5 passed.

### C11-05(a) and (b) — the merge

> **Target C11-05.1:** *Refusing a wide epoch cannot materialise an unbounded suffix — at most
> `max_rows` rows are materialised, at most `max_rows + 1` inspected if a sentinel is needed, at
> most `max_epochs` epochs visited, and any unavoidable discovery work is bounded and stated;
> every refusal preserves the original certified answer; and bounded merge records its real scan
> and materialisation work.*

**done, and the sentinel case does not arise.** `Base::delta_rows_at` returns an exact count, so
the decision never needs to look one row past the cap: at most `max_rows` are materialised and
`max_rows` inspected. The discovery work is the count itself, which reads no rows, and it is
stated where the method is declared along with why the two-pass shape carries no race — an
epoch at or below the frontier is sealed, so the count and the extraction observe the same rows
by construction.

Four refusals, four counters: over-epochs, over-rows, unavailable prefix, and the new
`merges_refused_overflow`, because `delta += d` was unchecked and a wrapped merge would install
a balance made of arithmetic that did not happen. Every refusal preserves the owner's own
answer, asserted in every case.

Command: `cargo test --offline -p nilestream-core --test merge_budget`, exit 0, 8 passed.

**(b)** `MergeCaps::default() == OFF`, alone on `e8982f5`. `MergeCaps::ON` carries the
preregistered pair. Two const assertions pin both at compile time where no test filter can skip
them; `tests/merge_default.rs` pins `default() == OFF` (`Default::default` is not `const fn`)
and pins that re-enabling must select `ON` rather than edit the numbers, or a future re-enable
would silently re-tune at the same time.

> **Target C11-05.2 / C11-05.3** — the interval install and its Host C measurement.

**not done: not reached.** They are below the tranche-1 line and depend on (a) and (b) having
landed, which they now have.

### C11-07 — torn-corpus fixtures

> **Target C11-07.1:** *The torn corpus owns isolated fixtures; a forced name collision cannot
> delete another test's file; the controlled collision case either reproduces the Mac failure
> against the old helper or the report says it did not; and three complete eight-thread target
> runs retain every corruption and torn-tail assertion with exit 0.*

**done, and the controlled case reproduces it.** The old scheme collides within one clock tick
under a shared tag and then deletes the other party's fixture; the test supplies the clock, so
it reproduces with no threads and no scheduler luck. Three eight-thread runs: exit 0, 6 passed,
0 failed, each time. No ledger implementation changed and no corruption or torn-tail assertion
moved.

One of these tests was passing vacuously until the guard exposed it — §3.

Command: `cargo test --offline -p nilestream-ledger --test torn_corpus -- --test-threads=8`,
three times, exit 0.
---

## 3. Every material surprise, including the ones that are about my own work

The order asks for all of them with their EV and no quota. There are nine. Four are findings
about the code, two correct an auditor, and three are mistakes of mine that a guard or a test
caught — those are in because a report that only lists other people's errors is not a record of
what happened.

EV is impact × confidence ÷ cost on the order's 1–5 scales, used only to rank.

### S-1 — A11-06 is not a toolchain finding, and the gate has a hole (EV 20)

**A11-06 reported the `u64 as u64` cast as a lint error on 1.97.1 and clean on the pin. It is
an error on the pin too.** `cargo clippy --offline --manifest-path
crates/gbs-nilestream/Cargo.toml --all-targets -- -D warnings` fails on **1.95.0** with
`casting to the same type is unnecessary`.

What made it look compiler-dependent: `gbs-nilestream` is `exclude`d from the GBS workspace —
correctly, because it needs a niles checkout beside it and requiring one to run the suite would
make an optional dependency mandatory by the back door. So **every `cargo clippy --workspace`
run in that repository skips it**, this gate's included. The crate was *tested* by manifest path
since cycle 7 and never *linted* by one, so the only runs that ever reached the cast were
hand-typed, and those happened to be on 1.97.1.

The repair is one section in `c11-gates.sh` (niles `90e420b`) that lints the adapter crate by
manifest path under the repository's own selector, and again under `--lint-toolchain` when one
is given. A missing adapter crate is `NOT RUN` there, not a silent pass.

*What it changes.* A11-06's repair was right; its diagnosis was not, and any reasoning that
treated it as a reason to move off 1.95.0 was reasoning from a hole in the gate.

### S-2 — the same hole again, in the other direction (EV 16)

`make reproduce` failed on `tools/memprobe` needing `delta_rows_at`, and `cargo build
--workspace` had said nothing: memprobe is outside the workspace too, because `GlobalAlloc`
cannot be implemented without `unsafe` and this repository claims to contain none.

Two instances in one cycle of the same shape — **a crate excluded from a workspace for a good
reason is a crate no workspace-scoped command checks** — is worth a general rule rather than two
patches. The rule I would propose for the manual: every crate excluded from a workspace is
listed with the command that covers it, and the gate runs that command; an exclusion with no
covering command is a red row.

### S-3 — A11-07's writer regression is a partial-point effect (EV 15)

Scoring cycle 10's own CSVs with `bench --score` reproduces the six T04.2 read rows to the
integer and adds two things the audit did not have.

**At the `full` working point, not one writer row clears the gate** — including `12r/6w` at
−18.9% writer throughput, which is over the relative half at **1.4 pooled MADs**. That is
exactly the row a reader quoting percentages would have called a regression. A11-07's ~30%
throughput and 53–80% write-p99 losses are real and are **entirely at `partial`**.

And **`read_p50_us` fires on four of six rows at +11% to +17%** — a cleaner signal on the same
data than throughput, and one nobody had looked at because the transcript printed p50 and the
hand analysis used the rate.

Both are assertions in `score_c10_fixture.rs` now, not sentences.

### S-4 — `check_limit` was unsound, not merely unchecked (EV 15)

`Book::total` summed with `+=` on `i128` and stamped the **caller's** scale onto constituents
that carry their own, so a scale-0 amount could be added to a scale-2 one and the total looked
authoritative. Each is now a named gap and never an addend.

The larger one: `check_limit` reported a partial total over the limit as `Breached`, on the
argument that exposure is monotone in what has been read, so no unread constituent can bring it
back under. **The argument is valid and its premise is false in this type**: `Reading::Present`
carries a *signed* amount, and a book of exposures contains credit positions.
`an_exposure_reading_is_signed_so_a_partial_total_is_not_a_lower_bound` builds the
counterexample — a partial total of 400,000 above a complete one of 250,000.

The verdict is split into `BreachedOnReadablePart`, which says both true things and certifies
neither direction it cannot. **Restoring the definite breach is a business rule** — a
nonnegative exposure type, or a proven lower bound on the unread part — and belongs with LC-41.
The pre-existing test that asserted the unsound behaviour is rewritten rather than deleted, and
the rewrite is the finding.

*Operational note.* A caller matching on `Breached` would stop seeing partial breaches. There
is no such caller in either repository today; if one is written before the rule is decided, it
must match both variants.

### S-5 — two defects in this cycle's own harness, found by writing its self-test arms (EV 12)

**`c11-gates.sh` fabricated its toolchain line.** It called `pick_toolchain` with no argument;
the function read an ambient `REPO` the script never sets; under `set -u` both reads died; and
the preflight printed `RUSTUP_TOOLCHAIN=stable, because the pin (none declared) does not resolve
here` for a tree whose `rust-toolchain.toml` says 1.95.0 on its first line. Every gate then ran
under whatever `stable` is on that machine and reported it as the tree's own choice. **That is
MF-7 again, one cycle after MF-7, inside the script written to prevent MF-7.**

**`$(dirname "$0")` is relative when the script is invoked by a relative path**, so the arm that
runs it from `/` failed until `HERE` was resolved once at the top. `TOOLCHAIN_INHERITED` moved
out of the function for the same class of reason: read after the first call, it reported the
script's own selector back as the value it had inherited.

Both were found by arms written to test something else, which is the argument for writing them.

### S-6 — `check_arm_caps` watched the first level only (EV 10)

F-11-20 made the caps check run in every *mode*; it still read one `merge at` line per log. A
replicate whose first level agreed and whose second did not would have passed. It now checks
every level, and checks whenever the daemon reported caps at all — a structured transcript
reporting caps 0/0 for an arm launched at `default` passed the previous version, and the arm
named `a structured pinned arm launched at default` is what caught it.

### S-7 — my wire test was vacuous, and the controls said so (EV 8)

`checked_arithmetic.rs`'s first draft counted only `DataRow`s. A served answer is framed once
into `Backend::Rows`, so **every control reported zero rows** and every refusal assertion passed
for the wrong reason. The controls — "arithmetic that has an answer still has it" — are what
failed and exposed it. Its first `i128::MIN` was also written `0 - (MAX - 1)`, which is
`MIN + 2`, so the boundary case passed against the unfixed code.

Both are in the commit message. A test suite whose negative cases pass and whose controls fail
is telling you the negatives are meaningless.

### S-8 — I claimed a defect I could not reproduce, and withdrew it (EV 8)

In C11-04 I tightened the slot restore so a reclaimed flight can only put its value back over
the marker it installed, and wrote it up as a second defect. **When I ran the guard, the test
passed against the unguarded version too.** `apply_epoch` skips a `Pending` slot, `is_resident`
is false for one so eviction never chooses it, and there is at most one flight per key: the case
is unreachable today.

The guard is still worth having, because the sweep widens the window from "the incoming reader
just found this" to "this has sat here for an unbounded time", and a rule resting on three
decisions in three other functions is worth making local. The commit and the test now say that
plainly. The general point is the one worth keeping: **a guard that passes against the
reversion has not demonstrated anything, and running it is the only way to find out.**

### S-9 — one of my own new tests was passing vacuously, and its guard exposed that too (EV 7)

`a_taken_name_is_stepped_over_and_its_contents_are_untouched` squats the name the fixture
allocator will try next. With a single global counter, that name depended on how many
allocations *other* tests had happened to make first, so it was usually not the next one and the
test asserted nothing — it passed against a deliberately broken allocator. The counter is per
tag now.

That is the same cross-test coupling C11-07 exists to remove, in the test written to remove it.
Two of the nine surprises here are "my test did not test what it said"; both were found by
running the reversion, and neither would have been found by reading.

---

## 4. What this cycle changes about earlier findings

| finding | what it said | what this cycle establishes |
|---|---|---|
| **A11-06** | the lint row is red on 1.97.1, green on the pin | red on the pin too; the row looked toolchain-dependent because `--workspace` never lints the adapter crate (S-1) |
| **A11-07** | the merge arm costs ~30% writer throughput and 53–80% write p99 | true, and **only at the partial working point**; at `full` no writer row clears the gate, including one at −18.9% and 1.4 pooled MADs (S-3) |
| **A11-05** | `check_generator` ignores `None`, so invalid economics validate | confirmed and repaired; the analytics half also contained an **unsound** verdict, not merely an unchecked one (S-4) |
| **A11-01** | `sum` and `count` over one base both answer the sum | confirmed; the fragment is narrowed so the shape is refused at installation, and the GBS adapter is unaffected because it installs only `Agg::Sum` |
| **A11-02** | a refused merge materialises the epoch it refuses | confirmed with the probe's own numbers, and repaired by an exact preflight count |
| **A11-03** | 256 dropped owners exhaust admission for unrelated keys | confirmed and repaired |
| **F-11-19** | two evaluators, three arithmetic semantics | confirmed and repaired; **no corpus fixture depended on the defaulting**, which the card asked to be reported either way |
| **F-11-01/12/13/20** | the harness's own defects | all still repaired, and three more found on top (S-5, S-6) |
---

## 5. Guard transcripts

Every landing ships a guard, proved red on reversion in a **disposable git worktree** under the
session scratchpad, removed afterwards. In no case is the witness a compile error.

| card / commit | reversion applied | what fired | exit |
|---|---|---|---|
| C11-06.1 `0a3ef89` | `pooled_mad` → `(a + b) / 2.0` | `full 9r/5w: pooled MAD 1877.7, published 1908. The mean of the two MADs would be a different number and this is the assertion that says which one this scorer computes.` | 101 |
| C11-06.3 `49c6ca9` | `may_launch`'s first check → `if false` | *(nothing — the hash check caught it, so the reversion was widened)* | 101 |
| C11-06.3 `49c6ca9` | `may_launch` → `return 0` unconditionally | `NOT REFUSED: the stale executable was launched — the sentinel /tmp/c10-selftest.o0MRbb/LAUNCHED exists` | 1 |
| C11-06.5 `adab975` | one row removed from `TABLE` | `these counters are on the nilestream_stats wire and no row in this table says what they are or how they move: ["merge_rows_visited"]` | 101 |
| C11-02 `8c68499` | `arith::div` → `Ok(0)` on zero; `arith::mul` → `wrapping_mul` | `the query answered 1 row(s). Before C11-02 the first of them held a zero, which a client cannot tell from a balance of zero: [[Some("Int(3)"), Some("Int(0)")]]`, and in **release** the overflow test observed `Int(-110)` | 101 |
| C11-01 `fbae068` | the `MixedPlans` refusal removed **and** `require_base` returning early | `**this is the defect**: the count output answered the sum. left: 30, right: 2` | 101 |
| C11-03 `3c34eb9` | *(not run as a worktree reversion)* | the repair is a type change; the witness is that the pre-existing declaration test **failed** when the two check rules were conflated, and `a_basis_of_zero_is_refused_at_declaration_and_generates_nothing` fails on the old `.ok()?` by construction | — |
| C11-04 `b33177f` | the sweep on admission → `if false` | `a key with no flight of its own was refused admission because 256 unrelated owners had been dropped` | 101 |
| C11-04 `b33177f` | the restore guard → unconditional | **nothing fired** — the case is unreachable today; withdrawn as a defect, kept as a guard (S-8) | 0 |
| C11-05(a) `c4bad72` | `delta_rows_at` → `deltas_at(e).len()` | `the merge refused a suffix it had already built: 10000 rows materialised against a cap of 2. left: 10000, right: 0` | 101 |
| C11-07 `32d5822` | a taken name cleared with `remove_dir_all` and then taken | `the allocator must not take a name it did not create. left: "/tmp/niles-torn-17154-squat-1"` | 101 |

Two rows record a guard that **did not** fire. The first is a widening — the reversion I chose
was caught by a second check, so it proved nothing about the first — and the second is S-8. A
table that only listed the guards that worked would be a table that had never been run.

Per harness commit, the self-test arm and both exit codes:

| harness commit | arm | with the change | reverted |
|---|---|---|---|
| `49c6ca9` | `a stale executable whose arm never built, and it was not launched` | 0 | 1 |
| `49c6ca9` | `a structured pinned arm launched at default` | 0 | 1 (as F-11-20's own arm) |
| `49c6ca9` | `an arm whose caps change between levels of one replicate` | 0 | — (new check, no prior version) |
| `90e420b` | *(gate section; its own self-test has no arm for a section that shells out)* | 0 | — |

---

## 6. The scored table, from cycle 10's own CSVs

`bench --score crates/bank-bench/tests/fixtures/c10-hostc-t04.2` — **Host C, October, not this
container**. Reproduced here because it is the fixture the scorer is guarded by, and because
three rows of it are new.

| point | level | metric | pinned | merge | rel | pooled MAD | MADs | verdict |
|---|---|---|--:|--:|--:|--:|--:|---|
| full | 6r/3w | reads/s | 145,466 | 132,430 | −9.0% | 1,312 | 9.94 | noise-limited |
| full | 9r/5w | reads/s | 153,087 | 135,738 | −11.3% | 1,909 | 9.09 | **fires** |
| full | 12r/6w | reads/s | 155,821 | 143,118 | −8.2% | 1,363 | 9.32 | noise-limited |
| partial | 6r/3w | reads/s | 133,341 | 123,860 | −7.1% | 1,024 | 9.26 | noise-limited |
| partial | 9r/5w | reads/s | 136,482 | 118,986 | −12.8% | 1,047 | 16.71 | **fires** |
| partial | 12r/6w | reads/s | 136,161 | 118,949 | −12.6% | 1,197 | 14.38 | **fires** |
| partial | 6r/3w | writes/s | 572.7 | 400.4 | −30.1% | | 6.64 | **fires** |
| partial | 9r/5w | writes/s | 917.5 | 627.8 | −31.6% | | 11.35 | **fires** |
| partial | 12r/6w | writes/s | 1,001.4 | 754.0 | −24.7% | | 2.58 | noise-limited |
| full | 12r/6w | writes/s | 921.7 | 747.2 | −18.9% | | 1.41 | noise-limited |
| partial | 6r/3w | write p99 | 7,953 | 12,156 | +52.8% | | 3.02 | **fires** |
| partial | 9r/5w | write p99 | 8,018 | 14,393 | +79.5% | | 6.00 | **fires** |
| partial | 12r/6w | write p99 | 8,061 | 13,971 | +73.3% | | 3.75 | **fires** |
| full | 9r/5w | read p50 | 55.3 | 61.4 | +11.0% | | 13.47 | **fires** |
| partial | 9r/5w | read p50 | 59.0 | 69.1 | +17.1% | | 24.50 | **fires** |
| partial | 12r/6w | read p50 | 77.5 | 88.8 | +14.6% | | 38.76 | **fires** |

Every read row reproduces the audit's §0.3 table to the integer. The counter columns — fold
fraction, pending joins, arrival gap — read `MISSING` for this fixture, because no build of that
vintage printed the machine-readable line, and the scorer says so rather than printing zeros.

---

## 7. Every SHA, every bundle

**niles** — public, all from `c11/00-audit` @ `ff4658c` or `c10/02-read-safety` @ `653a369`:

| branch | head | base | bundle sha256 |
|---|---|---|---|
| `c11/05-evidence` | `90e420b` | `ff4658c` | `c3461b5811b9b0b796fa0b38b2d06c916826b1e1cc757f1b6cefb7fd57cc693e` (v2, 6 commits) |
| `c11/02-arith` | `671eaf0` | `653a369` | `861c6ed3ada19d2a8e00c1bca418ae48ea8ed998b96c47b0e2fcb4d770401989` |
| `c11/01-fragment` | `fbae068` | `671eaf0` | `ebe52f6c4670e678a13920825f12a427f8bfc3fb86ca0606c4ee2929f3130170` |
| `c11/03-flights` | `b33177f` | `671eaf0` | `7bf38560b2e1640f76547382fa498c651ff2cb193c843edab69f2716e8620c35` |
| `c11/04-merge` | `14fd117` | `b33177f` | `7bc415f20f9821b81c819c02befd2a706ad63a9881f8fcd5a15622b393739794` |
| `c11/06-torn` | `32d5822` | `653a369` | `7bb4c7478cbc116c1f90b72af1fbb2c77de7cea9cbc0e372e2a344e76b4b88f0` |

The first delivery of `c11/05-evidence` was `1d242b477086d1e2a34234e3cdee637bbbd5a7b076b9d6b6ab13a7848c316f42`
at `7efb68c` (5 commits); v2 supersedes it and fast-forwards from it.

**GBS** — private, all from `c10/00-adapter-guard` @ `01e1d85`:

| branch | head | base | bundle sha256 |
|---|---|---|---|
| `c11/02-adapter-lint` | `c66d9cf` | `01e1d85` | `8e3f6f947dcb42804986025019103d4305fc63bf83619835b925c51aaeeb4218` |
| `c11/03-plan-contract` | `2e0e4b4` | `c66d9cf` | `99c2b285d54937cc7fb3ed94f9704652e09ba97f31b0a10ab05cabf2591c6d7a` |
| `c11/01-economics` | `3c34eb9` | `c66d9cf` | `8a5819ac0c90f214cf49ddd3ed9b707ff817701f2d32d82b90cf569d51e32cdf` |
| `c11/04-delta-count` | `c653fdf` | `c66d9cf` | `6c9549d1a5b4bf9a730cba578e351d4e67c64c5239806e0da43e1d03003e2fa6` |

**A deviation from the branch plan, stated as the order requires.** The plan bases every GBS
branch on `01e1d85`. Three of the four are based on `c11/02-adapter-lint` instead, because
C11-06.4's new gate section lints the adapter crate for the first time and that lint is red
without the cast fix — so a branch that touches `gbs-nilestream` and is based on `01e1d85`
cannot pass its own gate. One chain to merge in order rather than branches that only pass
together. This is `BLOCKED-branch-C11-03` in the order's vocabulary; it is recorded here rather
than decided silently.

**Pairing.** A GBS branch that implements a niles trait names the niles SHA it was tested
against. `c11/03-plan-contract` requires niles `fbae068`; `c11/04-delta-count` requires niles
`14fd117`. Pairing either with an earlier niles branch fails `downstream_adapter`, which I hit
once and which is the gate doing its job.

---

## 8. Container validation, per landing

`bash c11-gates.sh --lint-toolchain none --niles-sha <sha> --gbs-sha <sha>`, with
`C11_TOOLCHAIN=stable`:

| niles | gbs | result |
|---|---|---|
| `7efb68c` (C11-06) | `c66d9cf` | every gate ran and every gate is green |
| `671eaf0` (C11-02) | `c66d9cf` | every gate ran and every gate is green |
| `90e420b` (C11-06.4) | `c66d9cf` | every gate ran and every gate is green |
| `b33177f` (C11-04) | `c66d9cf` | every gate ran and every gate is green |
| `14fd117` (C11-05) | `c653fdf` | every gate ran and every gate is green |
| `32d5822` (C11-07) | `c66d9cf` | every gate ran and every gate is green |

Each run covers: the niles workspace suite, the paired adapter gate, the GBS workspace suite
and its adapter crate by manifest path, the generated documents, `make reproduce`, fmt and
clippy for both trees **and** the adapter crate by manifest path.

`numeric_binary_oracle` is green in every run because PostgreSQL 16 was started; without it, it
is a red row and a host fact, exactly as §6.1 of the order says.

**MF-12 as practice.** Every source edit in this cycle was made by an exact-string
single-occurrence Python replacement that asserts its own match count before writing, and every
multi-edit script writes once per edit or verifies afterwards. Three edits failed their
assertion and were rewritten rather than forced: the `pick_toolchain` source line (two
occurrences, not one), the plan-extraction move (the first slice swallowed the graph walk), and
the `MergeCaps` default block (the replacement produced unbalanced braces and `bash -n`/`cargo
build` caught it immediately).
---

## 9. What did not run, and why

Every card below is `not done: not reached` unless the reason is more specific.

| card | status |
|---|---|
| **C11-05(c)** the interval install | **not done: not reached.** It is below the tranche-1 line and starts only when (a) and (b) have landed and C11-06's scorer exists. Both now hold, so it is the first thing cycle 12 can take. |
| **C11-05(d)** Host C measurement | **not done: the author's run**, and it cannot start before (c). |
| **C11-06.8** Block B | **not done: the author's run.** `~/c11-pairs-out` exists on Host C and is not readable from this session. |
| **C11-08** one skew generator | not done: not reached. |
| **C11-09** the manual foundation | **not done: not reached, and blocked besides.** LC-20's definition and the D-1…D-8 relabelling are both unanswered (§10), so the decision catalogue and the marker strike cannot be written honestly. |
| **C11-10** the two design notes | not done: not reached. |
| **C11-11** durable syndicated lifecycle | not done: below the cut, and LC-46/34 and LC-41 are unanswered. |
| **C11-12** owned immutable prefix | not done: below the cut; depends on C11-05(d). |
| **C11-13** the staged backlog | not done: below the cut. |
| **C11-14** independent manual replay | not done: below the cut; needs C11-09. |
| the matched existing-store comparator | not done: needs the author's prepared PostgreSQL environment and C11-08. |

**Nothing was compressed to fit.** Where a target's wording could not be met exactly, §2 says
which word was narrowed and why, rather than reporting the sentence as done: C11-02's
"position", C11-01's "Sum and Count across two outputs", C11-03's "at construction",
C11-04's 10,000 cycles, C11-05.1's sentinel, C11-06.1's fold fraction, C11-06.5's "every", and
C11-06.7's 1.97.1 half.

---

## 10. Decisions still owed, and what each blocks now

| decision | blocks |
|---|---|
| **LC-20** — is the cycle-7 text the definition? | C11-09's marker strike. Astra's `BLOCKED-LC20-definition` still stands; nothing was invented. |
| **D-1…D-8** — confirm they are historical incidents, not decisions | C11-09's decision catalogue. |
| **the exposure monotonicity rule** (new, from S-4) | whether `BreachedOnReadablePart` can ever be certified as `Breached`. Needs a nonnegative exposure type or a proven lower bound on the unread part; sits beside LC-41 and is an accounting reviewer's call, not mine. |
| **LC-30** — `Slot::Pending`'s protocol at a different anchor | nothing. C11-04 preserves it either way, and `a_dropped_owner_never_changes_the_answer_another_reader_was_handed` is the assertion that it does. |
| **LC-36, LC-41, LC-46/34, LC-48, LC-49/51, LC-52, K0b's threshold, the comparator environment, LC-47** | as the order says; none blocked tranche 1. |

**One question of mine.** C11-04 puts `flights_reclaimed` on the `nilestream_stats` wire and
C11-05(a) puts `merges_refused_overflow` there. C11-06's counter-table test requires every wire
column to have a row, and the branches are independent — so merging them without adding two
rows fails `every_wire_column_has_a_row_in_this_table_and_every_row_names_a_column` naming them.
That is the test working. Do you want the reconciliation commit now, or at merge time? It is
two `StaysZero` rows and a sentence each.

---

## 11. Known limitations of this report

* **No Host C number is in it.** Every timing figure quoted is cycle 10's, re-scored from cycle
  10's own files. Nothing here measures this cycle's changes for performance, and no card in
  tranche 1 makes a performance claim.
* **1.97.1 is untested.** The container has one compiler. Every gate run declares
  `--lint-toolchain none`, so the absence is in the transcript; A11-06's rows are not covered.
* **E18's byte totals move by 32 bytes and are not committed.** `Stats` gained two counters.
  The allocation counts — the portable half — are unchanged at 150,263. The byte rows are
  `toolchain-scoped` and belong to the host that produced them; `results/E18-memory.csv` and
  `.md` are deliberately left as they are and **want re-measuring on Host C**.
* **The counter table's "every" is qualified.** Eighteen of thirty-two counters are asserted to
  *stay* zero with a stated reason rather than driven non-zero, because a single-threaded
  fixture cannot produce a join, a waiter refusal or a flight refusal. Driving those is a
  concurrency harness and its own card.
* **C11-03's basis validation is at declaration, not construction.** Both are asserted; the
  constructor's signature is unchanged.
* **The fold fraction's definition is committed to.** `folds_from_counters = uninstalled_folds +
  pinned_installs + deferred_merges`, divided by the level's `reads`. The scorer prints all
  three so a reader can check it rather than take it. If the audit's 87%-of-3.88M figure was
  computed differently, the two definitions need reconciling before any row is compared across
  them.

---

## 12. The five protected files

Read through the bridge with `GIT_OPTIONAL_LOCKS=0` at every check, never staged, never
modified, never bundled:

| file | bytes |
|---|--:|
| `.DS_Store` | 10,244 |
| `AGENTS.md` | 16,639 |
| `niles/.DS_Store` | 6,148 |
| `thesis/.DS_Store` | 8,196 |
| `thesis/Niles-Thesis.pdf` | 816,110 |

Unchanged at every check, including the last one taken while writing this report. Nothing in
this cycle stages, switches, resets or cleans on Host C; every git command run there is in the
author's own Run-now blocks. The container's own working trees are clean at every commit and
carry no untracked files at all — the five exist only on Host C.

---

## 13. Where the rest of the evidence is

| what | where |
|---|---|
| every commit's reasoning | the commit messages, which are the long form of §2 and §3 |
| the guard transcripts in full | this session's transcript; the disposable worktrees are removed |
| the six green gate runs | `/tmp/gates-*.log` in the container, copied to the session scratchpad; not committed, because they are host-scoped and this container is not a reference host |
| cycle 10's raw CSVs | `crates/bank-bench/tests/fixtures/c10-hostc-t04.2/`, committed unaltered on `c11/05-evidence` |
| the audit that asked for all this | GBS `c11/work-order-11` @ `6233b21`, and the two orders it consolidates |
