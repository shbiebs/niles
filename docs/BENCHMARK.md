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

0. **The `fsync` ceiling, per host and per barrier.** Every `durable` figure is bounded by
   `barriers/s × transactions per barrier` (`SPEC-ENGINE.md` Part 0), and the first factor is
   a property of the storage *and* of which barrier was requested. Measured with the same
   probe across the hosts this project has run on:

   | host | filesystem | barrier | barriers/s | µs each |
   |---|---|---|--:|--:|
   | Linux container | ext4 on virtio | `fdatasync` | 4,961–5,825 | 172–202 |
   | Linux VM on a Mac | ext4 on NVMe | `fdatasync` | 500–959 | 1,043–2,001 |
   | macOS | APFS on NVMe | `F_FULLFSYNC` | 255 | 3,917 |
   | overlayfs, `fsync=volatile` | overlay | `fsync` | ~1,000,000 | ~1 |

   **The last row is not storage evidence.** The mount option makes the barrier a no-op, and
   a ceiling measured there is a measurement of the mount option. A number quoted from that
   host cannot be compared with any other, and a work order that inherits it will conclude
   that the storage is not the constraint for reasons that have nothing to do with storage.

   Two checks, neither sufficient alone: `make fsync-proof` shows the record barrier reaches
   the kernel as a syscall (the volatile-overlay row passes this), and `bench`'s ceiling
   probe shows it costs what a barrier costs (a wrongly-named barrier passes this on a host
   where `fsync` and `fdatasync` are the same call, which this container is).

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

   The fold itself is no longer a plausible suspect, which is worth stating because it was
   the obvious one. At 20,000 postings and 10,001 groups it costs 26.5 M instructions and
   4.3 ms in process, down from 97.9 M and 7.3 ms — see the callgrind section above for what
   moved and why.

   Nor is the reply, any longer. T-05 took the whole served path from 51.0 M instructions to
   27.8 M and from 95,035 allocations to 12,602, and T-06 took the top-ten shape from 50.7 M
   to 32.8 M. The effect is visible in the contract table: the `analytical` composite went
   from 0.66× PostgreSQL to **1.32×**, and all four common statements are now at or above the
   baseline — `group_by_cur` 4.47×, `sum_negative` 2.27×, `top_ten_by_sum` 1.18×,
   `group_by_acct` 1.04×, where the last was 0.48×.

   `group_by_acct` is the one at the margin, and what remains in it is producing and sending
   10,001 rows: the Z-set's own row vectors and the socket, not the aggregation. Its
   run-to-run figure also moves more than the others between sessions, so 1.04× should be
   read as parity rather than as a lead.
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
| `analytical` | NOT MET (still, against a 10–12× contract) — though the composite is now **above** 1.0× | no longer item 4 at all. After T-04, T-05 and T-06 the composite is 1.32× PostgreSQL rather than two thirds of it, and all four common statements are at or above the baseline. What remains between 1.32× and 10× is the protocol floor: a round trip is ~120µs here, so five statements per composite cannot be answered in the 0.25–0.30ms the contract's multiple implies whatever the engine does. See "Is this contract reachable" below. |

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

## Instruction counts (callgrind), and the one binary that makes them cheap

Allocation counts are what the E18 gate asserts, because they are deterministic. They do not
settle every question — whether a change moved *work*, or merely moved it from the heap to the
stack — and where they do not, the deterministic answer is an instruction count.

`crates/nilestream-server/examples/fold_ir.rs` serves one statement against the engine E18 is
configured for and does nothing else, so a profile takes seconds and describes one query
rather than eight scenarios mixed together:

```sh
cargo build --release -p nilestream-server --example fold_ir
valgrind --tool=callgrind --callgrind-out-file=/tmp/fold.out \
    ./target/release/examples/fold_ir \
    "select acct, sum(amt) from postings group by acct" 1
callgrind_annotate --inclusive=yes /tmp/fold.out | grep -E 'Folder::row|Folder::finish'
```

The trailing `1` is the rounds — one conserved pair per account, 20,000 postings and 10,001
groups, matching E16's committed recipe. Omit it and the example seeds two, as E18's scenarios
do; an instruction count that did not say which is a figure for an unnamed base.

**The T-04 measurement**, at 20,000 postings and 10,001 groups:

| | `Folder::row` | `Folder::finish` | total | in-process |
|---|--:|--:|--:|--:|
| before | 57.99 M Ir | 39.89 M Ir | 97.9 M | 7.3 ms |
| after | 21.46 M Ir | 5.08 M Ir | **26.5 M** | **4.3 ms** |

Three costs went, and all three were the same mistake in different places: a one-column group
key is one integer, and it was being stored, compared and rebuilt as a boxed vector.

* The accumulator map is keyed by `Option<i128>` when the key is one column. `Value` has
  exactly two variants, so that mapping is total and needs no schema, no column kinds and no
  type inference — and the `match` that performs it is exhaustive, so a third variant would
  be a compile error rather than a silently wrong grouping.
* Every group's accumulators live end to end in one arena and the map holds an index. A
  `Vec<Acc>` per group was ten thousand allocations to hold, for almost every query in the
  fragment, a single running total.
* `finish` builds the output Z-set **in bulk**. Both accumulator maps iterate in ascending key
  order, the orderings agree, and two groups with different keys give two rows that differ in
  their first columns — so the rows arrive sorted and distinct, and `eval::add`'s
  get-then-insert was doing thirteen `Vec<Value>` comparisons down a rebalancing tree, ten
  thousand times, to find a slot whose position was already known. Collecting into the
  `BTreeMap` takes std's bulk path instead: sort (linear on ordered input), then build
  bottom-up with no rebalancing. That one change is 39.9 M → 5.1 M.

E18's `served_group_by_acct` budget moved from 105,000 to **80,000** allocations per query
(measured: 73,536, from 95,035). What remains is O(groups) and none of it is O(base rows): the
10,001 that stay are the output rows themselves, which are what the `ZSet` type *is*. Taking
those off the served path is T-05's job.

The 26.5 M is below the 30.9 M a hand-written reference fold needed for the same Z-set from
the same postings, which is the number this task was aimed at rather than a round figure.

### The reply (T-05)

With the fold down to 26.5 M, the rest of `query` was 24.5 M — and none of it was computing
anything. The same recipe, at the same 20,000 postings and 10,001 groups, one query per
process:

| | `RevEngine::query`, inclusive | of which the fold | everything else | in-process |
|---|--:|--:|--:|--:|
| after T-04 | 51.0 M Ir | 26.5 M | 24.5 M | 4.3 ms |
| after T-05 | **27.8 M Ir** | 27.0 M | **0.8 M** | **2.5 ms** |

The 24.5 M was two copies of an answer that already existed.

* The engine rendered the Z-set into `Vec<Vec<Option<String>>>` — a vector per row and a
  `String` per cell — and the wire layer then parsed those strings back into bytes. Both
  paths now read the Z-set where it lies: `Rows` carries a `RowSource`, the served answer
  hands over the Z-set and the anchor, and the framer writes each cell's integer straight into
  one reply buffer. The text form remains for the diagnostic statements, whose cells are
  genuinely strings, and as `Rows::text()` for in-process callers and tests.
* The folded aggregate was handed to the reference evaluator as a precomputed node and asked
  for the output. `try_run_node_with` returns a `Cow` and `into_owned` at its boundary
  **deep-clones a borrowed one**, so a plain `group by` with nothing above it copied ten
  thousand row vectors and rebuilt the tree to arrive at the value it started from. When the
  aggregate *is* the output, the fold has already answered. Where an `order by` or a `limit`
  sits above it, the reference still evaluates from the folded value, so those operators keep
  exactly one semantics.

Framing is the third saving and does not appear in the table above, because the example does
not go over a socket: a served row was a `DataRow(Vec<Option<String>>)` and `encode` allocated
a body vector and an output vector for each one. Rows are now framed into a single buffer
carried as one `Backend::Raw`, with `RowDescription` and `CommandComplete` still ordinary
messages either side of it — which is what keeps `Execute`'s "do not re-send the description"
filter working, and keeps a reply legible in a packet capture.

**The bytes did not change**, and that is asserted rather than argued:
`a_framed_row_is_the_bytes_the_message_encoded` compares the framed form against the encoded
message for nulls, zero, negatives and `i128::MIN` — the value a formatter that negates before
converting overflows on — and the `psql` conformance transcript is byte-identical. The
server's tests decode rows with `pg_wire::decoded_rows` rather than matching on
`Backend::DataRow`, so they assert what reaches a client rather than which representation
carried it.

### The top-k step (T-06)

`limit 10` over a `group by acct` sorted ten thousand rows to keep ten, and cloned every one
of them into a vector before sorting. The rows are borrowed now, and only the prefix that can
reach the answer is ordered: `select_nth_unstable_by` partitions at `offset + count`, and the
sort runs over that prefix alone. The same recipe, on the top-ten statement:

| | `RevEngine::query`, inclusive | of which the reference | in-process | E18 |
|---|--:|--:|--:|--:|
| before | 50.7 M Ir | 20.5 M | 3.8 ms | 22,631 |
| after | **32.8 M Ir** | **2.5 M** | **2.4 ms** | **12,628** |

`served_top_ten` is now the fold's own cost plus 26 allocations — the ten rows and the
selection's one vector — where it used to be the fold's cost plus a second copy of all ten
thousand groups.

**The bound is `offset + count` rows, and it is safe** because every row in a Z-set is
distinct and carries a positive weight, so each fills at least one of the slots the limit has
to give away; a row outside the first `offset + count` in the ordering cannot reach the output
however large the weights ahead of it are. And the answer is *identical* rather than similar,
because `order_rows` falls back to comparing the rows themselves: the ordering is total over
distinct rows, so the prefix is unique and there are no ties for a selection algorithm to break
differently from a sort.

That last point is where the change could have gone wrong invisibly, so it is tested twice.
`a_bounded_limit_answers_exactly_what_a_full_sort_would` writes the replaced implementation out
again inside the test and compares the two over equal keys, negatives, nulls, weights above
one, offsets that start inside a repeated row, and limits at and past the end — the reference
evaluator cannot be its own judge here, because comparing it against itself would compare the
new code with the new code. Golden `65_top_k_negative_sums_and_ties` pins the same property in
the corpus: a `limit 3` that cuts into a three-way tie at a negative sum.

E18 after all three tasks:

| Scenario | before T-04 | after T-05 | budget |
|---|--:|--:|--:|
| `served_group_by_acct` | 95,035 | **12,602** | 14,000 |
| `served_top_ten` | 22,631 | **12,628** | 14,100 |
| `served_point` | 20 | **12** | 13 |
| `served_group_by_cur` | 25 | **13** | 15 |
| `served_sum_negative` | 23 | **12** | 14 |

`served_point` is not a reply-size story — it returns one row. Four of its eight came from
`scan_fold::plan`, which cloned the predicate out of the circuit to describe it; a plan is
built per query and thrown away at the end of one, so it now borrows, and cloning a
`Scalar::Binary` no longer clones its two boxed operands.

What is left in `served_group_by_acct` is the Z-set itself: one row vector per group, which is
what the type is, plus the tree holding it. Going below that needs a different `ZSet`, which is
not an efficiency task but a change to the reference semantics' representation.

## Knowing what a query costs before running it (T-10)

The engine answers a statement in one of four ways, and they differ by three orders of
magnitude:

| class | what it does | cost |
|---|---|---|
| `view` | a maintained REV entry, read at the requested anchor, reconstructed on a miss | no base rows on a hit |
| `index-fold` | one pass over **one account's** postings, through the anchor index | that account's history |
| `fold` | one pass over the base, per-group accumulator, no intermediate Z-set | base rows + groups |
| `materialise` | the base as a Z-set and the circuit evaluated over it | a row and an allocation per base row |

Until this cycle the only way to find out which one a statement took was to measure it, and
two statements that fall into the cheap classes were falling into the most expensive one:

```
select acct, sum(amt) from postings where acct = 4242 group by acct              20 allocs, 0.8µs
select acct, sum(amt) from postings where acct = 4242 and cur = 0 group by acct  32 allocs, 1,014µs
select acct, sum(amt) from postings group by acct having acct = 4242         76,696 allocs, 13,955µs
```

Three spellings of one question, and adding a condition that *narrows* it made it a thousand
times more expensive. The causes were separate and both were cliffs rather than costs: the
scan restriction understood only a filter whose entire predicate was `acct = k`, so an `and`
made it give up; and a `Filter` above the aggregate made the fold planner refuse the shape
outright, so a `having` materialised the base to select one group out of ten thousand.

Now: a **conjunct** `acct = k` anywhere in any filter restricts the scan, which is sound
because every row surviving a filter satisfies every conjunct of its predicate, so `acct = k`
is a necessary condition on every row that reaches the aggregate — and the filters are all
still applied, so the restriction can neither drop a row nor keep one. A disjunction is
deliberately *not* split: neither side of an `or` is necessary, so `acct = 1 or acct = 2`
stays one leaf and restricts nothing. And a `having` whose columns are all grouping keys is
rewritten onto the source's column indices and pushed below the aggregate; one that names an
aggregate's value is not, because that value does not exist before the aggregation.

| E18 scenario | before | after | budget |
|---|--:|--:|--:|
| `served_point_conjunct` | 32 | **15** | 17 |
| `served_having_on_key` | 76,696 | **21** | 24 |

`served_point_conjunct` is three above `served_point` rather than equal to it, and the three
are a correctness finding the tests produced. The **maintained view** may only answer when
`acct = k` is the *whole* restriction: it is keyed by account and currency and knows nothing
about a query's other conditions, so answering `where acct = 7 and cur = 99` from it returned
account 7's balance for a query that selects no rows. A scan may be restricted whenever the
conjunct is necessary; a view may only answer when it is sufficient. Those are different
questions and the code now asks both — `account_restriction` and `sole_account_filter`.

**`explain` says which class**, from `rev_engine::serve_path` — the function `query` itself
branches on, so the two cannot become different descriptions of the same engine:

```sh
psql -h 127.0.0.1 -p 5434 -U bench bank \
  -c "explain select acct, sum(amt) from postings group by acct having acct = 7"
   property    |                        value
---------------+------------------------------------------------------
 serve path    | index-fold
 what it costs | one pass over one account's postings, reached through…
 circuit nodes | 4
```

`nilesc explain FILE` prints the same class per view under the circuit dump. It is a *static*
answer: `view` additionally requires conditions a compiler cannot see — that the balance view
is installed, that the base holds one currency, and that the entry is true at the anchor that
was asked for — and each of those falls back to `index-fold`, so `explain` says "view" for a
shape that *can* be a view read rather than for one that certainly will be.

## The wire (T-32): binary formats, streamed replies, and a client that stopped charging both sides

Three things, and the third moved the contract table most.

**The reply's peak memory is a constant.** A served answer used to be assembled whole before
the first byte reached the socket. `Backend::Rows` carries the Z-set instead and `write_all`
renders it through a 64 KB buffer, flushing as it fills. E18, at two sizes an order of
magnitude apart:

| scenario | allocations | peak bytes |
|---|--:|--:|
| `wire_reply_10k` | 1 | 69,632 |
| `wire_reply_100k` | 1 | 69,632 |

Ten times the answer, the same cost to the byte. With the flush removed those read 4 / 835 KB
and 7 / 6.7 MB, which is the shape the budget refuses. What remains O(rows) is the Z-set — the
fold's own per-group accumulator, which PostgreSQL's hash aggregate has too.

**Binary result formats, negotiated per column** in `Bind` as the protocol specifies, with a
`set nilestream.binary = on` extension for the simple protocol, which has no field to carry a
format code. `RowDescription` reports what was actually chosen, so a client is never told one
thing and sent another. `int8` is eight big-endian bytes; `numeric` is PostgreSQL's own
decimal encoding, and it is checked against a running PostgreSQL rather than against a decoder
of our own — see `crates/bank-bench/tests/numeric_binary_oracle.rs`. An encoding is defined by
one implementation, and the money boundary is not a place to trust one nobody compared.

**The harness client stopped charging both sides for work neither asked for.** The workloads
time statements and discard their answers; the client was building a `Vec` per row and a
`String` per cell anyway — about forty thousand allocations and 2.7 ms on a ten-thousand-row
reply. `simple_counted` reads the same bytes and skips the cells rather than copying them.
Used identically against both targets, which is the rule: a client lean against one server and
eager against the other would be measuring the client.

That last change is why the analytical figures moved again:

| statement | before T-32 | after |
|---|--:|--:|
| `group_by_acct` | 1.04× | **1.58×** |
| `top_ten_by_sum` | 1.18× | **1.77×** |
| composite | 1.32× | **1.88×** |

**The `durable` row fell out of the parity band in one run**, 0.87× → 0.78×, and chasing
that is what produced the ceiling probe described in the next section. Nothing in T-32
touches the append path — `durable` returns no rows, so the lean client cannot have moved
it — and within-run MAD was 2.9% and 7.6%, which does not cover a 10% ratio movement. The
answer turned out not to be in the engine at all.

## Three questions about the reply path, and the instrument that answers them (E21)

The wire work above raised three questions it did not answer, and the answers existed only in
a session's scrollback, which is the same as not existing. `results/E21-wire-cost.md` is
generated by

```
cargo run --release -p bank-bench --example wire_cost
```

It is **not** in `make reproduce`, for the reason `bench --run` is not: it is wall-clock and
machine-dependent, so a diff against a committed copy would fail on any machine but the one
that produced it. Re-run it when the reply path changes; read the committed copy otherwise.

**What would it cost to remove the flush?** Nothing, and at large replies it would cost
something to remove it. `put_rows` is the same renderer with a no-op flush — it builds the
whole reply and writes it once — so the two paths can be run against each other over a real
loopback socket. E21 asserts they are **byte for byte identical** before timing either, then
reports the median per-reply wall clock with its scatter. Across runs the ten-thousand-row
ratio moves between 0.96× and 1.06× and the hundred-thousand-row ratio sits near 0.9×;
neither is reliably separated from the replies' own noise. The mechanism explains the shape:
a small reply is a few buffer-fulls, so the extra `write` calls are a visible fraction of a
short wall clock, while a large one makes the assembled path grow and copy a buffer the size
of the whole answer. **The bound is therefore not a memory-for-speed trade the results are
tolerating.** It buys the constant peak E18 measures — 69,632 bytes at both sizes, against
835 KB and 6.7 MB assembled — for no measurable wall clock.

**Is it efficient for the harness client to discard its answers?** Yes, and the margin was
large enough to be inside every published ratio. `simple` builds a `Vec` per row and a
`String` per cell for an answer the workloads throw away; `simple_counted` walks the same
bytes for their lengths. On a ten-thousand-row reply that is about 0.8–1.0 ms, which E21
measures at roughly a quarter of the statement as the harness used to time it. The important
part is that this was charged to **both** targets: the harness drives PostgreSQL and
Nilestream through the same client, so removing it favoured neither side — what it did was
stop a large additive constant from sitting inside both terms of every ratio and dragging
them toward 1.0×. A ratio with the same constant added to numerator and denominator is not
the engines' ratio. The lean path skips no protocol: every byte is read and every cell length
parsed; only the copy into a `String` that is dropped unread is skipped. Statements whose
answers a workload actually uses — the conservation checks, the point lookups that assert a
balance — still use `simple`.

## How the `durable` row is kept honest: the device, not the engine

`durable` is an `fsync` rate, and an `fsync` rate has a ceiling that belongs to the storage.
`storage::ceiling` probes the device several times, spaced, and reports the median, the MAD,
the extremes and the **spread** — highest over lowest. On this container's storage the spread
has been measured at 1.05×, 1.11×, 1.49× and 1.58× in probes taken minutes apart, with no
database involved. **The device moves more than the `durable` row was being asked to explain.**

So a movement in an absolute durable rate between sessions is not evidence about the engine,
and the harness stopped publishing one as though it were. `bench --run` probes the ceiling in
the same session as the run and E16 publishes each target's rate as a **fraction of it**:

| Target | Durable commits/s | Fraction of the device ceiling |
|---|--:|--:|
| postgres | 4668 | 48% |
| nilestream | 4595 | 47% |

A fraction is machine-independent in the way a rate is not. "This engine gets 47% of the
`fsync`s its storage can deliver" survives being run somewhere else; "4,595 durable commits
per second" does not. Both targets are measured against the same ceiling in the same session
and the runs are interleaved, so whatever the device is doing it is doing to both. E16 also
prints whether the device held still (`Ceiling::steady`, a spread within 1.1×) so a reader is
told when an absolute figure from that run is worth quoting at all, rather than left to
assume it is.

Three tests hold this: `a_ceiling_reports_the_devices_spread_and_refuses_to_call_a_moving_
device_steady`, `the_fraction_of_the_ceiling_is_what_survives_a_change_of_machine` — which
asserts that the same engine on two devices a factor of two apart reports the same
efficiency, the whole point — and `probing_the_real_device_returns_a_distribution_rather_
than_a_point`.

## The report row, and E23: the two rows the architecture is actually for (T-33)

Every row above measures the engine doing the same work as PostgreSQL, faster or slower.
None of them measures the thing the thesis is about, which is *not doing the work* — holding
a derived view current as the writes arrive, so that a report is a read of state that is
already correct rather than a recomputation of it. Two additions close that.

### `report` — the same statement, out of a maintained view, under concurrent appends

The `analytical` row is a **cold reconstruction**: the fold runs over the whole base for
every query. That is one end of the phase diagram, and it stays in the table under that name.
`report` is the other end — `select acct, sum(amt) from postings group by acct`, answered
from a view whose contract is `materialize: full`, read in key order at the anchor it is true
at, touching no base row.

Three things keep it from being a cache measurement wearing a maintained view's name:

**A second connection appends throughout.** A number taken from a view over a base that
stopped moving is a number about a cache. So the workload holds an appender open for the
whole measurement, writing conserved pairs at a fixed rate, and PostgreSQL gets the identical
treatment against its identically growing table — its recompute *is* the control, and a fair
one, because that is what a database without maintained state has to do. The appends are
counted and printed; a run whose appender managed none is `NOT RUN` with that as the reason.

The pacing is against the wall clock rather than a fixed sleep between statements, and the
reason is a defect this row had on its first run: unpaced, PostgreSQL's side took 108 appends
and Nilestream's took 21 — five times the write pressure on one arm of a two-arm comparison,
on a two-core host where that pressure comes out of the thing being timed. A fixed gap gives
a rate of `1/(gap + service time)`, which is a different rate on a target whose service time
is different. Sleeping until the clock reaches `i × gap` gives the same rate on both, or
falls visibly behind — and both the asked-for rate and the achieved one are recorded.

**The server is asked which path it will take, and the row is refused if it is the wrong
one.** `explain` reports `report-from-view` only when the view is full *and* has been
advanced to the anchor in question; anything else and the harness records `NOT RUN` with the
mismatch rather than publishing a fold's number under the maintained view's name. This
needed a repair: `serve_path` classified a statement from the compiled circuit, which is
right for four of its five classes and wrong for this one — a view's contract and how far the
write path has advanced it are not in the circuit — so `explain` had been answering `fold`
for statements the engine answered without reading a single base row.

**Two daemons, not one.** The engine holds one runtime, and the `point` row needs a budget
that binds while a report needs a view that never evicts. Making one budget serve both would
delete the very thing the point row measures, so `bench --host-nls` starts a second daemon at
`--nls-report-port` (one past the first by default) with the same seed and a full view, and
the results file names which port served which row.

### E23 — the two asymptotic rows

`results/E23-scaling.md`, generated by

```
cargo run --release -p bank-bench --bin bench -- --run --e23 --host-nls --runs 3 --e23-reps 7
```

The sweep is wall-clock and is not re-run by `make reproduce`. The **document**, however, is
a pure function of the committed CSV, and `bench --render` re-derives it — so the prose and
the data cannot drift apart even though the measurement cannot be repeated on another
machine.

Two axes, because they answer different questions and averaging them would answer neither.
Three series, because Nilestream has two paths and publishing only the fast one would be
publishing half the phase diagram.

**Along the base**, with the answer fixed at 10,001 rows, 20k → 200k → 2M postings:

| Series | 20,000 | 200,000 | 2,000,000 | Slope per row of base |
|---|--:|--:|--:|--:|
| `postgres` | 6.23 ms | 32.51 ms | 176.52 ms | 8.4e-5 ± 1.9e-6 — **positive** |
| `nilestream:cold` | 3.98 ms | 14.39 ms | 118.51 ms | 5.8e-5 ± 2.6e-7 — **positive** |
| `nilestream:warm` | 2.11 ms | 2.37 ms | 2.48 ms | 2.5e-7 ± 1.4e-7 — **flat** |

A hundredfold increase in accumulated history moves the maintained view's answer by about a
third of a millisecond, where both recomputing series grow by two orders of magnitude. **This
is the row the architecture exists for**, and it is the only row in this document on which
"matches PostgreSQL as the data grows" is a statement with a truth value: a ratio at one size
cannot tell a system that scales from one that happens to be quick at twenty thousand rows.

**The strict verdict on this row is not stable across sweeps, and the results file says so.**
Two full runs of the same binary on the same host read the warm slope as 1.18e-7 ± 3.2e-8
(positive — NOT MET) and 5.4e-8 ± 2.3e-7 (flat — MET). Both are honest readings of their own
nine points, and they disagree because the effect is near the resolution floor: the smallest
slope this design can distinguish from zero is about 4.5e-7 ms per row of base, which is
larger than the effect. Re-running until the wanted verdict appears would be reporting a die
throw as a property of the system, so `e23_contract` prints the floor and the ratio to the
control beside the verdict — and the ratio, at three orders of magnitude, is what is stable.

**Where the residual is, measured rather than argued.** Along this axis the answer has the
same *number* of rows at every base size, but not the same number of *bytes*: at a hundred
times the base each balance has summed a hundred times as many postings and takes about two
more decimal digits. The harness records the answer's measured size at every point, and the
same nine measurements plotted against kilobytes of answer instead of rows of base give:

| Series | answer bytes at 20k / 200k / 2M | ms per KB of answer |
|---|---|--:|
| `postgres` | 68,903 / 78,904 / 88,905 | 9.01 ± 1.41 — positive |
| `nilestream:cold` | 108,907 / 128,909 / 148,911 | 2.94 ± 0.53 — positive |
| `nilestream:warm` | 108,907 / 128,909 / 148,911 | 0.0091 ± 0.0123 — **flat** |

So what is left of the warm series' growth is the cost of putting a bigger answer on the
wire, which no design makes smaller. This is evidence about a cause and not a replacement
verdict: the contract row is judged on the criterion as written, and reports `NOT MET` when
the sweep it was taken from says so.

**Along the answer**, with the base held at 200,000 postings, 1k → 10k → 100k rows:

| Series | 1,001 | 10,001 | 100,001 | Slope per row of answer |
|---|--:|--:|--:|--:|
| `postgres` | 17.94 ms | 32.60 ms | 105.71 ms | 8.6e-4 ± 2.5e-5 |
| `nilestream:cold` | 9.62 ms | 15.20 ms | 46.73 ms | 3.8e-4 ± 1.1e-5 |
| `nilestream:warm` | 0.30 ms | 2.29 ms | 25.47 ms | 2.5e-4 ± 3.2e-6 |

Here the claim is **parity**, deliberately: a report has to put its answer on the wire, and a
system claiming to beat that would be claiming to send fewer bytes than the answer contains.
Nilestream is 3.4× below PostgreSQL's cost per row of output, which meets a parity claim with
room, and is not the interesting result — the flat base slope is.

### Why the error bars, and the two refusals that come with them

A slope near zero always *looks* like the answer H-F1 wants, so a point estimate is not
evidence. `crates/bank-bench/src/fit.rs` fits by ordinary least squares and reports the
slope's standard error; "flat" means the confidence interval contains zero at two standard
errors, and the tests include one where a genuine thirty-fold growth must come out
**positive**, so the criterion is not a rubber stamp.

Two things it refuses rather than reports:

* **A fit on fewer than three distinct sizes.** With two the fit is exact, the residual
  degrees of freedom are zero, and the standard error is 0/0 — a number that would look like
  certainty and mean nothing.
* **A verdict whose control has no slope of its own.** E23's first output-axis run held the
  base at 200,000 rows while the answer grew only from 201 to 1,001, so PostgreSQL's cost was
  dominated by an unchanging scan and its output slope came out *negative* — which the
  comparison read as parity being met. It now reads `INCONCLUSIVE` and says to widen the
  answer's range.

`MISMATCH-A-01` in `docs/BUILD-LOG.md` records what these rows replaced: the analytical
contract's 10–12×, a geomean over ClickBench queries whose answers are a handful of rows,
carried into a workload that returns ten thousand — where a fixed serialisation cost sits in
both terms of the ratio and drags it toward 1.0× however fast the server is.

## Concurrency (E19), and why it is a separate document

`SPEC-ENGINE.md` Part 0 states its four targets without a concurrency qualifier, and E16
measures them **from one connection**. E19 asks the question E16 cannot: when a second and a
fourth connection are added, does throughput rise?

```sh
cargo run --release -p bank-bench --bin bench -- \
      --run --render --scaling-only --publish \
      --pg-port 5432 --host-nls \
      --accounts 10000 --operations 500 --runs 3 --connections 1,2,4
```

`--scaling-only` is what keeps the two experiments apart. They are published at different run
counts, and without it a re-run of E19 would re-render `results/E16-wallclock.md` under this
invocation's flags — overwriting a measured table as a side effect of measuring something
else. With it, no `Sample` is produced at all, so `write_all` is never reached and the
committed contract document cannot be touched however `--publish` is passed.

The separation is structural rather than a convention. A scaling level produces a
`ScalingSample`, which carries a connection count and has its own CSV schema; a contract row
produces a `Sample`, which has none. Neither renderer accepts the other's type, so a
4-connection figure cannot reach the contract table by anyone's mistake.

**Reading the table.** The step from 1 to 2 connections is not the interesting one: a
single-connection level is round-trip bound — the client sends, blocks, and reads, so the
server is idle for much of every operation — and a second connection fills that idle time as
well as using the second core. A ratio above 2.00× there is a pipeline being filled, not
superlinear scaling. The **top step** is the finding, and E19 renders it as its own table: at
4 connections on a 2-core host both cores are already busy, so a target that keeps rising is
parallelising and one that falls is contending. The bands are ±10%, for the same reason the
contract table's parity band is ±20%: a noisy host makes a difference smaller than the spread
not a finding.

**Every level starts from the same base.** PostgreSQL is re-prepared and the hosted engine is
re-seeded before each level, so no level begins with the previous level's appends. Transaction
identities are composed from `(run, level, thread, operation)` in a range no other workload in
this binary uses. That last part is a repair of a real mistake: the audit's scratch harness
reused one sequence across levels, so the four-connection level re-submitted the
one-connection level's transaction ids, the ledger refused every one of them as a duplicate,
and idempotency working exactly as designed was read as a throughput collapse.

**The 2-core caveat is printed in the document itself**, not left to this page: these rows say
whether throughput rises from 1 to 2 to 4 connections *on two cores*. They say nothing about
16 or 48.

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
