# Host C, run 4, re-run after T-01 and T-02 — `results4-20260906-092323`

Apple M4, 10 cores (4 performance, 6 efficiency), 16 GiB, macOS 26.6.2, rustc 1.97.1.
APFS on NVMe, so every barrier is `F_FULLFSYNC` at ~3.9 ms — **255 barriers/s**, the slowest
storage this project measures on and the one where the durable row is device-bound by
arithmetic rather than by engine defect.

Arms: **A** = `8d229ea` (T-00, T-01, T-01b, T-02), **B** = `15425b5` (one mutex, before T-06),
**C** = `d681f0d` (T-06 without the lock-order fix).

## The first re-run measured the wrong commit

`run4.sh` created its worktrees on first use and reused the directories thereafter, with arm A
pinned to the *branch* `c6/audit-cycle-7`. That branch stopped meaning "after" the moment T-00
landed a commit on top of it, so the re-run this work order asked for — *after T-01 and T-02
land* — silently re-measured `cea8a1e`, the commit before T-00. The working copy was on
`8d229ea` throughout; the arms never look at it.

Two independent tells, both in the log:

* `git worktree list` reported `wt-after cea8a1e [c6/audit-cycle-7]`;
* the mixed probe prints every column the server returns, **by name**, and `view_answers` and
  `fallbacks` were absent — T-02 adds both.

The script now derives arm A from the working copy's `HEAD` (override with `ARM_A=<rev>`),
re-checks it out every run, and **refuses to measure** if the retarget did not take. A stale
arm is worse than no run: it produces a full set of plausible numbers about the wrong commit,
and every one of them would have been reported as a cycle-7 result.

## T-02 over the wire — the number F-27 was about

Three reader/writer shapes, 8 s per phase, durable sink, 10,000 accounts at budget 2,500:

| shape | fallback before | after | base rows folded | mixed reads/s before → after |
|---|--:|--:|--:|--:|
| 4 readers, 2 writers | 45.8% | **0.2%** | 20.6M → 7.9M (−61%) | 85,788 → 85,445 |
| 8 readers, 4 writers | 43.7% | **0.2%** | 42.9M → 16.6M (−61%) | 85,976 → 89,289 |
| 8 readers, 1 writer | 45.3% | **0.2%** | 20.8M → 1.7M (−92%) | 94,415 → 94,707 |

E19's `point` rows read `fallback 0.0%` at every connection count from 1 to 16; `fold` and
`durable` read `n/a`, which is correct — neither consults the view, and a rate that was not
measured must not render as zero.

**The "before" column is derived**, because the counter did not exist to take it directly. Under
the old counting a fallback incremented the view's `reads` *and* the scan surface's `served`,
and `read_stats` summed them, so the reported total exceeded the queries the probe issued by
exactly the number of fallbacks; issued is `reads/s × 8 s` over the two read phases. After the
change the excess is 0.2% and `view_answers` equals `reads` exactly. Before, it is 43.7–45.8%
on three shapes that have no reason to agree unless the derivation is sound.

Not the 87.4% F-27 reported on Host A — a different machine, and a figure this run neither
confirms nor refutes. Close to half of all keyed reads, on this one.

**It bought nothing in wall clock, and that is the result rather than a disappointment.** Read
throughput moved −0.4%, +3.9% and +0.3%; read p50 is 23 µs before and after. A keyed fallback
folds one account's postings through the anchor index — about 22 rows — against a ~47 µs wire
round trip, so it was roughly 2% of a read's cost and 46% of the reads. Counted work fell
61–92%; the wall clock could not see it.

This is the counted-work methodology earning its place. A wire benchmark on this host would
have reported the mechanism as healthy while nearly half its keyed reads bypassed it — which is
how F-27 survived three audit cycles, and why the fix had to be gated on a counter rather than
on a latency.

## The acceptance checks

| check | required | measured |
|---|---|---|
| durable curve at 16 connections | within 10% of 7.80× | **7.80×** (260 → 2,027 ops/s) |
| E19 `mixed` row carries a fallback column | the number cannot hide | `fallback 0.0%` on every point row |
| T-02 fallback rate under writers | ≤ 5% | **0.2%** on three shapes |
| F-28 (T-06 on ten cores) | reproduced | fold 1.83× → **4.57×**; point 95.8k → **138.8k/s** |
| F-34 (no writer starvation at 8:1) | negative result holds | write p50 3,991 → 3,932 µs (flat) |
| T-03 regression under `CARGO_TARGET_DIR` | green | 4 passed, 16 passed |

T-01 grew the durable record from an 8-byte epoch string to `parent ‖ hash ‖ canon(rows)`, and
the durable arm did not move: 7.80× against B's 7.62× and C's 7.66×, all within the spread. On
this host that is expected and says little — `F_FULLFSYNC` dominates whatever the record holds.
The Linux-container measurement is where the record size showed up, and there it *helped*
(4.94 → 7.50 transactions per barrier), because a payload that takes longer to hand over leaves
a wider drain window for the next submitter to join.

Peak RSS through arm A fell from 85.4 MiB to 79.2 MiB, consistent with T-01's `write_rows`
removing two allocations per epoch from the chain.

## Carried, not closed

* **A long read tail under concurrent writes.** Mixed-phase read *max* is 12–13 ms against
  431–756 µs readers-alone, at every shape, before and after T-02. p50 and p99 are unaffected,
  so this is a small number of reads waiting on something — most likely the base write guard
  held across an apply. Pre-existing and unrelated to T-02; it belongs to whatever addresses
  LC-18's neighbourhood.
* **`fold` p99 at 16 connections is unstable**: 101,551 µs in one run against 17,017 and
  24,795 µs in the others, and the prior run showed the same spread. The median is steady, so
  this is a tail-behaviour question and not a throughput one. T-08 owns the fold plateau.
