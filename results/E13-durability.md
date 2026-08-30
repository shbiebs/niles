# E13 — the cost of durability and the shape of group commit

Wall-clock, machine-dependent. Ratios are the transferable part.

| threads | policy | txns/sec | txns per fsync | largest batch |
|---:|---|---:|---:|---:|
| 1 | Always (ledger-grade) | 4450 | 1.0 | 1 |
| 2 | Always (ledger-grade) | 5197 | 1.0 | 2 |
| 4 | Always (ledger-grade) | 11225 | 2.4 | 4 |
| 8 | Always (ledger-grade) | 20104 | 4.6 | 8 |
| 16 | Always (ledger-grade) | 30440 | 8.8 | 16 |
| 1 | Never (**not a ledger**) | 25457 | 0.0 | 1 |
| 2 | Never (**not a ledger**) | 47110 | 0.0 | 2 |
| 4 | Never (**not a ledger**) | 80415 | 0.0 | 4 |
| 8 | Never (**not a ledger**) | 96913 | 0.0 | 8 |
| 16 | Never (**not a ledger**) | 144558 | 0.0 | 16 |

## The price of the guarantee

| threads | cost of durability (x slower) |
|---:|---:|
| 1 | 5.72x |
| 2 | 9.06x |
| 4 | 7.16x |
| 8 | 4.82x |
| 16 | 4.75x |

If the ratio falls as threads rise, group commit is amortising the fsync and a
single sealer is a batching opportunity rather than the ceiling it appears to be.
If it stays flat, the fsync is being paid per transaction and the design needs
revisiting — which is the outcome that would matter, so it is stated first.
