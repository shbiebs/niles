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
      --calibrate --run --render \
      --pg-port 5432 --host-nls \
      --accounts 10000 --operations 2000 --runs 10
```

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
3. **A plan cache keyed by schema epoch, and nothing else.** `extended.rs` caches a compiled
   plan against the epoch it was planned at. The *simple* query path — which is what this
   benchmark uses on both targets — compiles every statement afresh: parse, resolve,
   typecheck, lower, verify, per query. That is a real per-query cost and it is paid on every
   row of this table.
4. **A full fold behind every scan.** `Serving::query` materialises the source as a Z-set and
   evaluates the circuit over it. A `where acct = k` predicate is pushed into the scan through
   the anchor index; an unkeyed `group by` is not, and materialises the whole base. **This is
   the cause of the `analytical` NOT MET**: 20,000 postings are re-read per query against
   PostgreSQL's sequential scan with an in-memory aggregate.
5. **No incremental maintenance on the served path.** The partially materialised view exists,
   is maintained, and is *not consulted by the wire path* — every point read is an anchored
   reconstruction. The `point` row's miss rate is therefore 1.00 by construction, which makes
   its parity result a claim about reconstruction rather than about a warm cache.
6. **Two cores.** The OLTP contract's "5–10×" was set against a 48-core baseline figure.
   **This is a contributing cause of the `oltp` NOT MET**, together with item 3.
7. **The budget setting.** The engine is hosted with residency for a quarter of the key
   space. A budget at or above the key count evicts nothing and would make the `point` row a
   measurement of a warm cache; the harness warns when it does not bind.

### Attributing this run's two NOT METs

| row | verdict | attributed to |
|---|---|---|
| `oltp` | NOT MET (≈1× against a 5–10× contract) | items 3 and 6 — per-query compilation on a two-core machine, against a contract written for 48 cores. Not to the ledger: the durable row shows the write path at parity with PostgreSQL's, on the same device and the same `fsync`. |
| `analytical` | NOT MET (≈0.13× against a 10–12× contract) | item 4 — a full fold per query. The engine is doing more work than PostgreSQL, not the same work more slowly, and the phase-diagram experiments are where that trade is characterised. |

Neither is attributed to the engine's correctness, and neither should be read as one. What
they are is a **measured baseline and a characterised gap**, which is the claim §9.14.1 can
support and the stronger claim it cannot.

### What the analytical row compares

Three of PostgreSQL's five analytical statements are outside Nilestream's lowered fragment,
so the row compares five statements against three:

| construct | why it is outside the fragment |
|---|---|
| `count(*)` | `*` is not a column, and the aggregate lowering resolves its argument as one (NL0502) |
| `count(distinct acct)` | `distinct` is a stage in this fragment, not an aggregate modifier |
| `order by sum(amt) desc` | `order by` resolves against the input schema — the choice that lets it name a column the query does not select |

Widening the fragment during a benchmark would be tuning the artifact to the measurement.

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
answering an anchored read and reconstructing on a miss. The measured runs sit at 8–14%
misses, each a real upquery touching real base rows, and the latency holds across them.

That combination is the claim worth having. A parity result at a 0% miss rate would only say a
warm cache is fast; at 9% it says *reconstruction* is, which is what the thesis argues. The
miss rate is printed with every run for exactly this reason, and a result quoted without it
should be treated as incomplete.

Two things it does not establish:

* **Nothing about durability or concurrency.** The read engine is in-memory and
  single-threaded. The durable path is measured separately, in GBS, where `make fsync-proof`
  counts the syscalls.
* **Not "faster than PostgreSQL".** The two are doing different work — an index scan and an
  aggregation against a maintained view plus occasional reconstruction. That difference is the
  point of partial materialisation rather than an unfair comparison, but it means the row says
  *a REV serves a point lookup as fast as an indexed aggregate*, which is a narrower and more
  useful statement.

The three `NOT RUN` rows are findings about the engine's surface: `nilestreamd` exposes no
write path over the wire, and its read surface serves per-key balances rather than scans. Those
are the next things to build, not caveats to argue away.
