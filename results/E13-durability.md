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

---

## A note on policy semantics (T-06, this branch)

The figures above stand; what changed is what the policies are allowed to do.

* **`SyncPolicy::Never` is refused by the sequencer.** It published an epoch with nothing
  on stable storage, while `submit`'s own contract says it returns when the transaction's
  epoch is durable and visible. The policy remains on `Segment`, because this benchmark
  prices the guarantee by removing it; what it may no longer do is reach a path that
  claims the guarantee. The `Never` column here is produced by driving `Segment` directly.
* **`Every(n)` no longer syncs twice.** The sequencer synced every epoch *and* `append`
  synced every n-th, so the batching policy was `Always` with an extra fsync.
* **A failed write is rolled back** rather than left in place, so a later fsynced,
  published, acknowledged epoch cannot sit behind a torn record and be discarded at the
  next recovery.
* **Damage anywhere but the tail refuses to open**, and truncates nothing.

None of these changes the cost of an fsync, which is what the table measures. They change
what is true when one fails.
