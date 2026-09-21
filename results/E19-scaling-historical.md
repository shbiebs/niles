# E19 — the mutex-era rows, kept

**Not regenerable, and not to be regenerated.** These are the rows `results/E19-scaling.md`
carried from cycle 7 until cycle 13. They are preserved here because deleting a measurement
when a newer one replaces it hides the thing worth knowing: that a published curve described
an engine the repository no longer had.

## What they were measured on

Committed by `fa012ed` (T-02) on **2026-09-04**. The daemon's engine became
`RwLock<RevEngine>` in `d681f0d` (cycle 6, T-06) on **2026-09-05** — one day later. So every
row below was measured with `daemon::serve` holding one `Arc<Mutex<RevEngine>>`, and the
engine has not had that lock since. Three normative places went on describing the mutex as
current for seven cycles: this file's own instrument note in `status.toml` (H-S10),
`thesis/09-evaluation.md`, and `docs/BENCHMARK.md`.

`f7cdfdd` (C9-01) and `4a8f63d` (T-11) touched the document afterwards, adding disclosures
and replacing the raw CSVs with one `not_run` row each. Neither re-measured: the rows below
are `fa012ed`'s, unchanged, and their raw per-run values exist only in that commit's history.

Two further caveats travelled with them, and both still apply to the numbers here:
**`MISMATCH-daemon-checkpoints`** — the daemon behind every row ran with
`checkpoint_interval = 0`, the configuration §9.4.1 excludes from the cost law; and
**F-72** — the `durable` and `mixed` levels' writes were a single leg of amount zero until
cycle 9's C9-01, so no view value moved under the readers of a mixed level.

## The rows

> **This host has 2 cores.** These rows say whether throughput rises from 1 to 2 to 4 connections *on two cores*. They say nothing about 16 or 48, and a reader who extrapolates them to a server-class machine is reading a number this experiment did not measure. The saturation point of a 2-core host is a property of the host.

| Workload | Target | Connections | Median ops/s | MAD | vs 1 connection | Median p99 |
|---|---|---|---|---|---|---|
| durable | nilestream | 1 | 4717 | 124 | 1.00× | 314 µs |
| durable | nilestream | 2 | 7053 | 192 | 1.50× | 476 µs |
| durable | nilestream | 4 | 6923 | 344 | 1.47× | 819 µs |
| durable | postgres | 1 | 6834 | 879 | 1.00× | 282 µs |
| durable | postgres | 2 | 8689 | 427 | 1.27× | 535 µs |
| durable | postgres | 4 | 12176 | 864 | 1.78× | 1282 µs |
| point | nilestream | 1 | 21607 | 312 | 1.00× | 101 µs |
| point | nilestream | 2 | 47783 | 97 | 2.21× | 128 µs |
| point | nilestream | 4 | 40415 | 1860 | 1.87× | 267 µs |
| point | postgres | 1 | 15839 | 124 | 1.00× | 97 µs |
| point | postgres | 2 | 44744 | 6600 | 2.82× | 87 µs |
| point | postgres | 4 | 52122 | 1995 | 3.29× | 156 µs |

### The top step, as it read then

| Workload | Target | Top step | Ratio | Reading |
|---|---|---|---|---|
| durable | nilestream | 2 → 4 | 0.98× | **flat** — the added connection buys nothing measurable |
| durable | postgres | 2 → 4 | 1.40× | **rises** — the added connection is doing work |
| point | nilestream | 2 → 4 | 0.85× | **falls** — the added connection costs more than it brings |
| point | postgres | 2 → 4 | 1.16× | **rises** — the added connection is doing work |

There was no `fold` workload in the scaling experiment at this date.

## What may and may not be concluded by comparing these with the current rows

**Not a controlled comparison.** The two sets were measured in different sessions on
different container instances, and `render::Provenance::render` carries the warning this
repository learned the hard way: *a ratio in this table may be compared only with another
taken in the same session on the same instance* — E16's `report` row once read 2.68× MET on
one instance of a host class and 2.17× NOT MET on another, with the code unmoved. Anyone
who subtracts a 2026-09-04 figure from a 2026-09-21 one and attributes the difference to the
lock is reading host variance.

**What survives that objection** is the *within-session* comparison against the PostgreSQL
arm, which is the same software in both eras and absorbs the instance. At the 2 → 4 step of
the `point` workload:

| era | nilestream 2 → 4 | postgres 2 → 4 (control) | nilestream relative to its control |
|---|--:|--:|--:|
| mutex (2026-09-04) | 0.85× | 1.16× | **0.73** |
| `RwLock` (2026-09-21) | 1.30× | 0.97× | **1.34** |

In the mutex era the engine scaled at roughly three-quarters of what the same host gave
PostgreSQL over the same step; on the current engine it scales at roughly a third more. That
is a real change in sign — from *below* the control to *above* it — and it is the strongest
statement these two sets of rows support. It is **not** an attribution: cycles 6 through 12
changed more than the lock, and no experiment here holds the rest fixed. An experiment that
did would build both engine versions and run them interleaved in one session, and this
project has not run it.
