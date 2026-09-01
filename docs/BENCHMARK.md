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

```sh
# 1. A real PostgreSQL. The harness will not substitute anything for it.
initdb -D /var/lib/pgdata -U bench --auth=trust
pg_ctl -D /var/lib/pgdata -l /tmp/pg.log \
       -o '-p 5433 -c listen_addresses=127.0.0.1' start

# 2. Nilestream, on its own port.
cargo build --release -p nilestream-server
./target/release/nilestreamd --port 5434 --accounts 10000 &

# 3. Check the harness before trusting it, then run.
cargo run --release -p bank-bench --bin bench -- --calibrate --pg-port 5433
cargo run --release -p bank-bench --bin bench -- --run \
      --pg-port 5433 --nls-port 5434 --accounts 10000 --operations 2000 --runs 10

# 4. Re-render the table from the CSVs alone, without re-running anything.
cargo run --release -p bank-bench --bin bench -- --render
```

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
| Storage | `/dev/vda`, ext4, virtualised block device |
| `fsync` cost | **93µs** per call — "fast NVMe or virtualised block device" |
| Implied ceiling | ~10,700 durable commits/s per connection |
| PostgreSQL | 16.13, `synchronous_commit = on`, `fsync = on`, `shared_buffers = 128MB`, `full_page_writes = on`, `wal_level = replica` |
| Measured durable rate | ~5,100 txn/s, **39% of the device ceiling** — consistent, and the gate's basis for passing |

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

## What the results do not establish

The `point` row currently reads `PARITY`, and it is **not** evidence that the read-model
runtime reaches parity. `nilestreamd` serves reads from an in-memory demo engine — its own
startup banner says so — so that row measures the protocol path against PostgreSQL's full
stack. The honest reading is that Nilestream's wire path is not a bottleneck. Making it an
engine result means serving the row from `proto-engine`'s `PartialView` over a real ledger,
at which point the same harness measures the mechanism the thesis is about.
