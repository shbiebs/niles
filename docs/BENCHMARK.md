# BENCHMARK — how to reproduce E16, and what it does and does not establish

`SPEC-ENGINE.md` Part 0 states four performance targets **relative to PostgreSQL**. Before
this harness existed, none of them had been measured against that baseline: every experiment
in `results/` reported counted work inside `proto-engine`, and the one wall-clock table in the
thesis (§9.4.4) was in-memory, single-threaded and compared to nothing. A speed claim whose
only evidence is counted operations in a prototype is, in the thesis's own grading, an
*editorial* contribution.

This document is what it takes to run the comparison and read it correctly.

---

## Running it

**One command runs everything.** It used to be four, one of which was a `cargo test`
invocation whose working directory is the *package* rather than the workspace — which is how
a second `results/` tree appeared under `crates/bank-bench/` and the table went on reporting
a missing row while a good measurement sat ten directories away.

```sh
# A real PostgreSQL. The harness will not substitute anything for it.
pg_ctlcluster 16 main start            # or: initdb -D … && pg_ctl -D … start

# Calibrate, run both targets on all four workloads, and render — in one process.
cargo run --release -p bank-bench --bin bench -- \
      --calibrate --run --render --publish \
      --pg-port 5432 --host-nls \
      --accounts 10000 --operations 500 --runs 5 --rounds 1
```

**`--publish` is what overwrites the committed `results/E16-wallclock.md`.** Without it a run
writes its document and CSVs under `--out` and leaves the repository alone. It used to write
the committed file on every run whatever `--out` said, so an exploratory run — an audit's, a
bisect's, a CI job's — left the working tree dirty with a table measured under whatever flags
that run happened to use. `publish::destinations` holds the rule and
`an_unpublished_run_writes_nothing_a_repository_tracks` fails the build if it is lost.

**Those are the parameters the committed results were produced with**, and they are read back
out of `results/E16-wallclock.md`'s own "How it was run" section by
`the_benchmark_recipe_reproduces_the_committed_numbers` in `crates/bank-bench/tests/thesis_drift.rs`.
This line said `--operations 2000 --runs 10` while the committed table came from 500 and 5, so
the one command a reader would type was not the command that produced the numbers underneath
it — which is the whole of what a reproduction recipe is for.

`--rounds` is in that list for the same reason and is the newer half of the finding. It sets
how many conserved pairs are seeded per account, and it used to be `--nls-rounds`, applied to
one target only, defaulted to `2`, and appeared in neither this recipe nor the results header.
PostgreSQL was therefore measured over 20,000 rows and Nilestream over 40,000, and every ratio
in the table was a ratio between two different bases. The flag now seeds **both** targets, the
run aborts if the two bases end up unequal, and `* Base rows per target:` is printed in the
results header so the check is one a reader can make without running anything.

`--host-nls` starts `nilestreamd` on a thread of the same process and drives it over TCP with
the same client, with a durable sink at `results/E16-wallclock/nilestream-bench.seg` under
`SyncPolicy::Always`. Without the sink the `durable` row would be measured against an
in-memory append; the harness refuses to start rather than run it.

`--render` alone re-renders the table from the CSVs without re-running anything, which is the
property that makes the numbers auditable: nothing in the binary contains a figure that
appears in the document.

### Why the two halves run differently

PostgreSQL is a server somebody else started, so the harness connects *out* to it. Nilestream
is this repository's own code, and the harness hosts its listener on a thread and drives it
over a TCP port — because environments differ in whether a benchmark binary may fork a
long-lived child or bind a listener, and a Nilestream column that read `NOT RUN` for
*sandbox* reasons would be worthless.

**The fairness rule is untouched**, and the distinction is exact: both targets pay the same
*client* cost — socket, protocol framing, round trip — because the same `wire::Client` drives
both. Hosting the listener on a thread changes where the server runs, not how the client talks
to it. Calling `RevEngine::read` directly would have been the shortcut; going through a port is
what avoids it.

Outputs: `results/E16-wallclock/{oltp,analytical,point,durable}.csv` and the rendered
`results/E16-wallclock.md`. **Nothing in the rendered file is typed in by hand** — that is why
`--render` exists rather than a paragraph explaining how to fill the table in.

## The calibration gate, and the mistake it caught

The gate runs first and refuses to write results if it fails.

Its **first** version compared PostgreSQL against a published figure: 333 durable transactions
per second per core, from `docs/research/performance-baselines.md`. It fired on the first run
at 16× the baseline — and the gate was right while the calibration was wrong.

A durable commit is an `fsync`, and an `fsync` costs whatever the device charges. The same
literature records the spread: **1.6–12.4µs** with power-loss protection, **891–2974µs**
without. A factor of a thousand, sitting directly underneath the transaction rate. 333 txn/s
is therefore not a property of PostgreSQL; it is a property of PostgreSQL *on a ~3ms-fsync
device*. On the machine below, which syncs in 93µs, holding PostgreSQL to 333 txn/s would have
meant the harness was broken — and the published figure would have concealed that by agreeing
with it.

So the gate calibrates against **the device**, measured moments before each run
(`bank-bench::storage`):

| Bound | What it catches |
|---|---|
| Rate **above** the device's `fsync` ceiling | Commits are not reaching storage. With one connection there is no group commit to explain it, so this is a `synchronous_commit = off` nobody mentioned — the commonest way a durability number is inflated |
| Rate **far below** the ceiling (< 5%) | The loop is not doing the work it claims, or every statement pays for something the benchmark did not mean to measure |

Both bounds are machine-independent, which is exactly what the published number was not.

## The machine these results came from

Recorded because a benchmark without its machine is a number without units.

| | |
|---|---|
| CPU | Intel Xeon @ 2.10GHz, **2 cores** |
| RAM | 7 GiB |
| Kernel | 6.18.44-fc-v22 |
| Storage | `/dev/vda`, **ext4**, virtualised block device, 252G |
| `fsync` cost | **77.9µs** per call — "fast NVMe or virtualised block device" |
| Implied ceiling | **12,845** durable commits/s per connection |
| Measured durable rate | 5,152 txn/s, **40% of the device ceiling** — consistent, and the gate's basis for passing |

Two cores. Every ratio below is a two-core result, and the OLTP contract's "5–10×" was
written against a 48-core baseline figure — which is one of the reasons that row reads NOT
MET, and it is named in the limitations list rather than left for a reader to work out.

**The probe now measures the device PostgreSQL's WAL is on.** It probed `temp_dir()`, which
on this container is a different filesystem from `$PGDATA`: a tmpfs `/tmp` reports an `fsync`
cost of about a microsecond and a ceiling of a million commits per second, against which
every real durable rate looks implausibly low and the plausibility gate is measuring the
wrong device. The server is asked `show data_directory`, and refuses to guess if it will not
say.

### The eight PostgreSQL settings, read from the running server

Read from `pg_settings` rather than from a config file, because the two can differ, and
printed into the results.

| setting | value |
|---|---|
| `server_version` | 16.13 (Ubuntu 16.13-0ubuntu0.24.04.1) |
| `synchronous_commit` | `on` |
| `fsync` | `on` |
| `full_page_writes` | `on` |
| `wal_level` | `replica` |
| `shared_buffers` | 16384 (8kB pages = 128 MB) |
| `work_mem` | 4096 (kB) |
| `max_wal_size` | 1024 (MB) |

PostgreSQL is left at its packaged defaults. Tuning it *down* to win would be the cheapest way
to fabricate a margin, so every setting the harness can see is read from the running server —
not from a config file, because the two can differ — and printed into the results.

## Fairness: one client, two servers

Both targets are driven over the **PostgreSQL wire protocol, through the same client**
(`bank-bench::wire`, ~450 lines of `std`). This is not incidental:

* A comparison where one side is called in-process and the other over TCP omits, from one side
  only, the syscalls, copies and round trip the other pays on every query — worth roughly the
  whole margin the specification claims, and invisible in the result.
* The client is on the measured path, so an external crate's version would become part of every
  number reported. The workspace has no external dependencies and a benchmark is a poor place
  to start.
* It exercises Nilestream's own `pg_wire` implementation from the other side, which found the
  defect below.

Neither target uses prepared statements. An earlier version prepared on PostgreSQL and could
not on Nilestream (`nilestreamd` implements the extended query protocol in `extended.rs` but
does not wire it into its connection loop), which would have flattered PostgreSQL. Preparing on
both is fair and preparing on neither is fair; preparing on one is not.

## A third defect, worth recording because it is the quietest

The Nilestream half runs under `cargo test`, whose working directory is the **package**
directory rather than the workspace root. A relative output path therefore created a second
`results/` tree under `crates/bank-bench/`, where `bench --render` never looked — so the table
went on reporting `NOT RUN` for a workload that had just been measured perfectly well, ten
directories away. Nothing failed; a good number simply went somewhere nobody read.

## The defect the harness found in its first hour

The first Nilestream run measured **23 point lookups per second** against PostgreSQL's 13,600.
Not the engine, and not the compiler — per-query compilation measures 0.02ms, so the whole
"compile every query" concern was worth 0.5% of the budget. `pg_wire::write_all` issued **one
socket write per protocol message**, so a four-message reply (`RowDescription`, `DataRow`,
`CommandComplete`, `ReadyForQuery`) was held by Nagle's algorithm pending the peer's delayed-ACK
timer. 43ms per query is that timer wearing a database's clothes.

Batching the reply into one buffer and setting `TCP_NODELAY` took the same workload to ~14,700
lookups per second — a factor of 640.

**No counted-work benchmark could have found this.** The engine did the right amount of work,
in the right order, and then waited. It is the clearest argument in this repository for
measuring wall-clock against a baseline rather than counting operations, and it is the reason
`ROADMAP.md` now carries an item to move the read path onto `proto-engine` before any
Nilestream number is quoted as an engine result.

## Known limitations of the Nilestream path

**Every NOT MET must be attributable to an item on this list or to the engine**, and saying
which is the point of having the list. A gap with no named cause is a result nobody can act
on; a gap attributed to "it is a prototype" is not attributed at all.

1. **One mutex over the whole engine.** `daemon::accept_loop` hands every connection an
   `Arc<Mutex<RevEngine>>`, so reads serialise against each other and against writes. The
   benchmark drives one connection, so this does not affect these numbers — and it is the
   first thing that would, at any concurrency.
2. **A thread per connection.** Right at this scale and stated rather than defended; not a
   design for thousands of connections.
3. **A compiled-circuit cache, per session.** The simple query path compiled every statement
   afresh — parse, resolve, typecheck, lower, verify — on every query. `callgrind` puts that
   at **177,064 instructions against 7,693 to serve a point read**: 96% of the daemon's own
   work on that workload was recompiling something it had just compiled. Statements are now
   compiled once per session and keyed by `(schema, statement text)`, which is sound because
   a lowering does not depend on the anchor it was compiled at — `Catalog::epoch` is carried
   into `lower` and never read by it, and a test holds that rather than a comment.

   Worth stating precisely, because both of the things previously said about this cost are
   true of different budgets. Compilation is *0.5% of a wall-clock budget* dominated by a
   ~120µs round trip, and *96% of the engine's own instructions*. Measuring first is what
   distinguished them: the point row moved 9,213 → 12,195 ops/s, +32%.

   `extended.rs`'s `Prepared` still keys on `engine.frontier()` as its schema epoch, which
   invalidates every prepared statement on every *append* — conservative, and useless as a
   cache, since a schema does not change when a posting is written. Left alone deliberately:
   that field is the mechanism a migration story is meant to hang on, and re-purposing it
   under an efficiency task would spend it.
4. **A materialising path, for the shapes outside the keyed-aggregate fragment.**
   `Serving::query` recognises `Source → (Filter | Map)* → Aggregate{sum, count}` — the same
   fragment the REV runtime accepts — and answers it by folding the ledger's own posting
   records in one pass, with a per-group accumulator and no intermediate Z-set. Everything
   above the aggregate (`order by`, `limit`) is evaluated by the reference evaluator from the
   folded value, so those operators keep one semantics. Everything *outside* the fragment —
   joins, `min`, `max`, `avg`, `distinct`, a bare projection — still materialises the whole
   base as a Z-set and evaluates the circuit over it, which is the honest cost of an
   arbitrary query against a partial-state engine.

   This used to be the path for *every* query, and it was the cause of the `analytical`
   NOT MET. Folding instead of materialising moved the four common statements by 25x, 19x,
   3.4x and 2.4x, and two of them are now faster than PostgreSQL. The remaining gap on
   `group_by_acct` and `top_ten_by_sum` is not the fold: those return 10,001 groups, and the
   cost is producing and framing 10,001 rows.
5. **Incremental maintenance, on the served path.** A single account's balance is answered
   by the REV runtime — partial materialisation under a budget, the absence lattice, an
   anchored upquery on a miss — over the circuit the daemon's own schema compiles to. The
   `point` row's miss rate is therefore **a measurement**: 0.30 at a budget of a quarter of
   the key space with 90% of reads in the hottest 1%.

   It was 1.00 *by construction*: the wire path evaluated the compiled circuit over a source
   scan and never consulted the view, and `read_stats` returned `hits = 0, misses = reads`,
   so the column could not have reported anything else whatever the engine did. The
   mechanism the phase diagram characterises was measured by nothing the daemon ran.

   The maintenance is not free and the cost is on the record: advancing the view on every
   append costs about three allocations per transaction and, measured against PostgreSQL
   within the same run, moved the `oltp` row from 1.16× to 1.00× of the baseline. That is
   the maintain-versus-reconstruct trade this system exists to characterise, paid on the
   write path so that a read of a hot key touches no base rows at all.

   Two conditions are checked rather than assumed, because each is a way the view could be a
   *different* answer from the one asked for: the base must hold exactly one currency (the
   view is keyed `(account, currency)` and a query naming no currency would otherwise have
   the server pick one, which is how a per-currency conservation rule becomes invisible), and
   the answer must be true at the anchor that was requested rather than merely at least as
   fresh. Either failing falls back to the fold.
6. **Two cores.** The OLTP contract's "5–10×" was set against a 48-core baseline figure.
   **This is a contributing cause of the `oltp` NOT MET**, together with item 3.
7. **The budget setting.** The engine is hosted with residency for a quarter of the key
   space. A budget at or above the key count evicts nothing and would make the `point` row a
   measurement of a warm cache; the harness warns when it does not bind.

### Attributing this run's two NOT METs

| row | verdict | attributed to |
|---|---|---|
| `oltp` | NOT MET (≈1× against a 5–10× contract) | item 6, and the arithmetic below. Not item 3: an `INSERT` does not go through the compiler at all, and not the ledger — the durable row shows the write path at parity with PostgreSQL's, on the same device at the same `fsync` cost. |
| `analytical` | NOT MET (still, against a 10–12× contract) | no longer item 4 for the keyed shapes: the fold answers `group by cur` at 2.7× PostgreSQL and `sum where` at 1.2×. What remains is the two statements that return 10,001 groups, where the cost is producing and sending the rows, and the protocol floor — a round trip is ~120µs here, so five statements per composite cannot be answered in the 0.25–0.30ms the contract's multiple implies whatever the engine does. See "Is this contract reachable" below. |

Neither is attributed to the engine's correctness, and neither should be read as one. What
they are is a **measured baseline and a characterised gap**, which is the claim §9.14.1 can
support and the stronger claim it cannot.

### What the analytical row compares

**A common set, paired by operation.** The row used to be five PostgreSQL statements against
three Nilestream ones, round-robined into a single composite sample — so the reported ratio
put PostgreSQL's *cheapest* statement (`count(*)`, 1.4 ms) on one side of a comparison and
not the other, and paired `group by acct order by sum(amt) desc limit 10` against a plain
`group by acct` as though the two were one statement. The gap it reported was real; the
number was not the gap.

`workloads::ANALYTICAL_STATEMENTS` now holds one entry per statement with both dialects. The
composite ratio is computed from the entries both targets run and from nothing else; the rest
are still executed on PostgreSQL, so their cost is on the record, and are excluded from the
ratio. `the_common_set_is_the_same_operation_on_both_sides` fails the build on a pair whose
two dialects are different operations.

The coverage difference is printed under the results table, derived from the same list:

| construct | why it is outside the fragment |
|---|---|
| `count(*)` | `*` is not a column, and the aggregate lowering resolves its argument as one (NL0502) |
| `count(distinct acct)` | `distinct` is a stage in this fragment, not an aggregate modifier |

`order by sum(amt) desc` used to be a third row here, and its removal is a defect report
rather than a widening. The stated reason — "`order by` resolves against the input schema" —
described the code truthfully and described its behaviour falsely: the key list came out
*empty*, and `nilestreamd` answered the query with the wrong ten rows and no diagnostic. The
benchmark skipped a statement the server was quietly getting wrong. `order by` now resolves
against the output schema (so an aggregate and a projection alias both work) and refuses
NL0509 when a key resolves to nothing, in both surfaces.

Widening the fragment during a benchmark would be tuning the artifact to the measurement.
Averaging over a statement one side cannot express is worse, because it looks like a
comparison.

### Both targets start each run from the same base

PostgreSQL is re-prepared between runs; the hosted Nilestream engine is **re-seeded** between
runs, on a fresh segment. It was not, and the asymmetry was worth about six percent per run
and compounding: each Nilestream run began with the previous run's `oltp` and `durable`
appends — 500×2 + 125×2 = 1,250 legs — still in its ledger, so the analytical row declined
monotonically across the five runs and the committed median was the median of that drift.

`Target::base_marker` is read at the start of every run and compared against that target's
first run — rows for PostgreSQL, epochs for Nilestream, never compared across targets. A
difference **aborts the run** rather than being noted, because two runs of one target that
started from different bases are not two measurements of one thing.

## Memory (E18), and the profilers that are not in a gate

`results/E18-memory.md` is generated by the instrument in `tools/memprobe`:

```sh
cargo run --release --manifest-path tools/memprobe/Cargo.toml          # regenerate E18
make memory                                                            # the budget gate
```

Allocation counts are deterministic within a build profile, which is why the gate asserts
them and reports bytes without asserting them: a `Vec`'s growth policy moves bytes in ways
that are not a regression, and wall clock on this host moves by tens of percent between
invocations of the same probe. The gate runs `--release` and `--test-threads=1` for reasons
the test's own header gives.

The tool is **outside the workspace**: `GlobalAlloc` has no safe implementation, and §9.10's
"no `unsafe`" claim is about the system rather than about the tools that measure it. The
repository-wide drift test allows that one file by name.

For the questions counters cannot answer — where the bytes are, and how the heap moves over
time — the two commands, not run in any gate because their output is neither deterministic
in the same way nor cheap:

```sh
valgrind --tool=dhat   --dhat-out-file=/tmp/dhat.json \
      ./target/release/nilestreamd --port 5434 --accounts 10000 --rounds 2 --budget 2500
valgrind --tool=massif --massif-out-file=/tmp/massif.out \
      ./target/release/nilestreamd --port 5434 --accounts 10000 --rounds 2 --budget 2500
ms_print /tmp/massif.out | head -40
```

### Is the OLTP contract reachable on this architecture?

The same arithmetic, and the answer is sharper. The contract asks for 5–10× PostgreSQL on a
**durable** path. A durable commit is an `fsync`, and one `fsync` per transaction caps a
single writer at `1/fsync` — `results/E13-durability.md` measures 4,450 ops/s at batch 1 on
this device. PostgreSQL measures ~3,000 here, so 5× is 15,000 and 10× is 30,000: **both are
above the per-connection fsync ceiling**, and no engine change reaches them.

Group commit does: E13 measures 30,440 ops/s at batch 16, a 6.8× recovery of the fsync tax.
But a batch of 16 needs 16 *concurrent* writers, and this harness drives **one synchronous
connection** — so the workload it measures can never form a batch whatever the engine
implements. The contract is unreachable under the harness's own workload shape.

Reaching it needs three things together: a group-commit sequencer, a concurrent client, and
more than two cores. PostgreSQL gains from concurrency too, so the ratio afterwards is not
predictable from these numbers — which is the honest reason this is recorded rather than
attempted.

### Is the analytical contract reachable on this architecture?

Worth the arithmetic, because "NOT MET" invites the reader to assume the gap is all engine.

The contract asks for 10–12× PostgreSQL. PostgreSQL answers the common set at about 160
composites per second here, so the target is 1,600–1,920 composites per second: 0.52–0.63ms
for four round-tripped statements, or **130–156µs per statement including the round trip**.
A round trip on this host measures about 120µs — `select 1` two thousand times through
`psql` takes 0.24–0.26s — which leaves 10–36µs per statement for everything the engine does,
against a base of 20,000 postings.

A single-key read fits in that (the engine answers one in about 2µs in-process). A grouped
aggregate over the whole base does not, and neither does sending 10,001 rows. **The
analytical contract as written is not reachable over a per-statement request/response
protocol at this base size, whatever the engine does**, and that is a property of the
contract's shape rather than of the implementation. What would make it reachable: state it
per statement, or over a pipelined or batched workload — either of which also helps
PostgreSQL, so the ratio after the change is not predictable from these numbers. That is a
decision for the specification and not for an implementation, and it is recorded rather than
quietly worked around.

## How to read the verdicts

| Verdict | Meaning |
|---|---|
| `MET` | The measured ratio reaches the stated multiple |
| `NOT MET` | It does not — reported as a result, not hidden |
| `PARITY` | Within ±20% of the baseline. **Overshooting a parity target is `NOT MET`**, because a claim that overshoots is still not the claim that was written down |
| `NOT RUN` | The target cannot serve this workload, with the reason printed below the table |

`NOT RUN` is a finding about the engine's surface, not a gap in the harness. Filling such a row
by measuring something else under the same name is the specific failure this whole apparatus
exists to avoid, and `render.rs` enforces it: a declared gap wins over any measurement, even
one that exists.

## What the results do and do not establish

The `point` row reads `PARITY` and it **is** an engine result: served by
`nilestream-server::rev_engine`, a partial view over an immutable hash-chained ledger,
answering an anchored read. **Its measured miss rate is 1.00**, and that is not a tuning
choice: the wire path evaluates the compiled circuit over a source scan through the anchor
index and does not consult the partially materialised view at all (item 5 below), so every
point read is an anchored reconstruction.

That is a stronger claim than the one this paragraph used to make and a different one. It
said "the measured runs sit at 8–14% misses"; they did not — the figure was typed into prose
while the harness ran a residency budget larger than the key space, under which nothing is
ever evicted and the true rate was zero. The rate is now a column of every CSV row, and a
parity result quoted without it should be treated as incomplete.

Two things it does not establish:

* **Nothing about durability or concurrency.** The read engine is in-memory and
  single-threaded. The durable path is measured separately, in GBS, where `make fsync-proof`
  counts the syscalls.
* **Not "faster than PostgreSQL".** The two are doing different work — an index scan and an
  aggregation against a maintained view plus occasional reconstruction. That difference is the
  point of partial materialisation rather than an unfair comparison, but it means the row says
  *a REV serves a point lookup as fast as an indexed aggregate*, which is a narrower and more
  useful statement.

**There are no `NOT RUN` rows in this table**, and the sentence that used to stand here said
there were three — "`nilestreamd` exposes no write path over the wire, and its read surface
serves per-key balances rather than scans". Both statements had stopped being true. A row that
cannot be run is still reported with its reason rather than omitted, and a run that produces
one now exits non-zero, so a script driving this harness can tell a complete measurement from
a partial one.
