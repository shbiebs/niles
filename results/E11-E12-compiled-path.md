# E11-E12 — results measured through the compiled path

Every number here was produced by `crates/nilestream`, which compiles a Niles program
with `niles-lang`, verifies the resulting circuit with `niles-ir::verify`, installs it on
the `nilestream-core` REV runtime, and runs a workload against an immutable,
epoch-ordered, hash-chained ledger. Nothing between those stages is hand-built.

That matters because Chapter 9's original numbers came from `proto-engine`, a
hand-written harness with no compiler in it. These runs reproduce two of its headline
findings along a completely different path — through the language, the type system, the
IR and the verifier — which is a far stronger check on the findings than re-running the
same harness with a different seed.

Reproduce:

```sh
cargo build --release -p nilestream
./target/release/nilestream run   examples/demo_bank.niles ledger_balance --checkpoint 16
./target/release/nilestream sweep examples/demo_bank.niles ledger_balance
```

## E11 — SC7 (Bounded Reconstruction) through the compiled path

Dense keys (200 accounts), so each key accumulates real history. `--reads 4000`,
`--budget 20`, Zipf *s* = 1.1. Reported figure is base rows read per upquery.

| epochs | C = 0 (no checkpoints) | C = 16 | C = 64 |
|---:|---:|---:|---:|
| 5,000 | 58.5 | **6.9** | 19.1 |
| 20,000 | 234.0 | **8.3** | 28.6 |
| 80,000 | 931.5 | **8.4** | 31.7 |
| growth over 16x history | **15.9x** | **1.22x** | 1.66x |

Without checkpoints the fold is history-length: cost grows 15.9x as history grows 16x,
which is linear and is the refutation Chapter 9 reports. With C = 16 it is flat at 8.4
against a predicted bound of C/2 + 1 = 9. With C = 64 it is bounded and rising toward its
own predicted 33, which it has not reached at 80,000 epochs.

This is the same result the hand-written harness produced (C = 16: 6.8 -> 8.3 -> 8.0 -> 8.5),
obtained through the compiler instead of around it.

## E12 — the phase diagram through the compiled path

Budget swept against the price of memory, in counted-work units. Cost is
`resident_entry_epochs x price + deltas_applied + base_rows_read`. 20,000 epochs,
20,000 reads, 20,000 accounts, Zipf *s* = 1.1.

| budget | price 0.0001 | price 0.0005 | price 0.002 | price 0.01 | price 0.05 |
|---:|---:|---:|---:|---:|---:|
| 250 | 75,541 | 77,517 | 84,926 | 124,443 | 322,027 |
| 500 | 51,746 | 55,636 | 70,223 | 148,021 | 537,007 |
| 1000 | 40,818 | 48,308 | 76,399 | 226,214 | 975,292 |
| 2000 | 38,619 | 52,145 | 102,871 | 373,406 | 1,726,084 |
| 4000 | 39,435 | 58,648 | 130,698 | 514,960 | 2,436,271 |
| 8000 | 39,439 | 58,663 | 130,752 | 515,226 | 2,437,599 |
| full | 39,439 | 58,663 | 130,752 | 515,226 | 2,437,599 |

**Optimal budget by memory price:**

| memory price | optimal budget |
|---:|---:|
| 0.0001 | 2000 |
| 0.0005 | 1000 |
| 0.002 | 500 |
| 0.01 | 250 |
| 0.05 | 250 |

The optimum is **strictly interior** at every price where it is not at the swept boundary:
neither the smallest budget nor full materialization wins at 0.0001, 0.0005 or 0.002. Full
materialization is never uniquely optimal at any price tested. The boundary between the
partial-materialization regime and the full-materialization regime sits between memory
prices 0.0005 and 0.002, which is the same band the hand-written harness located.

## What these runs do not show

The runtime is single-threaded, has no durability and no concurrency control, so nothing
here speaks to throughput, isolation, contention or recovery. The executable IR fragment
is source -> optional filter/map -> one keyed aggregate; joins, fixpoints and ordering
stages compile and verify but do not execute, and the runtime rejects them rather than
mis-executing them. Those cells of Chapter 9 remain marked *to be measured*.

## An off-by-one worth recording

The first version of the runner advanced the runtime with the loop counter rather than the
epoch the ledger actually sealed, and the ledger numbers epochs from zero. The failure mode
was not a wrong answer. It was a **silent null**: every maintenance counter read zero, the
reconstruction figures looked entirely plausible, and the run would have been reported as
evidence that maintenance costs nothing. The runner now refuses to return a result from a
run in which no delta was observed at all, because a null that looks like a measurement is
worse than an error.
