# C9-06.2 — the section-D measurement, and what it refuted

**Host C, 2026-09-08T14:40:12Z.** Darwin 25.6.0 arm64 (MacBook Pro, 10 cores), rustc 1.95.0.
`bash docs/audit/cycle-9/hostc/c9-pending.sh` — baseline `ebe7f8f` (C9-05) against candidate
`4ac5032` (C9-06), arms interleaved inside one session, one daemon per replicate, 2 warm-ups
discarded and 5 measured replicates per arm, 30-second mixed levels, 2,000 accounts × 8 rounds
at a residency budget of 2,500. `--checkpoint-interval` refused by both arms, so checkpoints
are 0 on both by construction. Every section ran; 5/5 replicates completed on each arm.

---

## 1. The verdict

**C9-06.2 is not met.** The pass line was *candidate median ≥ 1.25× baseline median at 12r6w,
under a ≥ 10% **and** ≥ 3 pooled-MAD gate*. The measurement gives **1.018×**.

| level | baseline median reads/s | candidate median reads/s | ratio | Δ | pooled MAD | Δ in MADs | gate |
|---|--:|--:|--:|--:|--:|--:|---|
| 6r/3w | 147,306 | 151,185 | 1.026 | +2.6% | 3,118 | 1.24 | **fails both arms** |
| 9r/5w | 156,686 | 159,499 | 1.018 | +1.8% | 1,083 | 2.60 | **fails both arms** |
| 12r/6w | 160,838 | 163,753 | **1.018** | **+1.8%** | 888 | 3.28 | **fails the ≥ 10% arm** |

The 12r6w difference clears three pooled MADs, which says the +1.8% is probably real. It is not
the result that was predicted, and clearing one arm of a two-arm gate is not clearing the gate.

**No contract or durability regression.** `durable` @12 conn: 1,410 → 1,473 median (+4.5%), and
markedly tighter — the candidate's five replicates span 1,466–1,474 against the baseline's
1,216–1,476. `fold` @12: 5,377 → 5,526 (+2.8%). `point` @12: 160,225 → 157,863 (−1.5%), inside
its own spread. `fallback` is 0.00% on every level of both arms.

## 2. The measurement refuted its own premise

The pass line was written against the cycle-8 curve **94,038 / 48,310 / 20,507 reads/s** with a
view wait reaching **22,160 µs**, and the work order was explicit that this curve is *inferred
from a document* rather than measured. The corrected harness does not reproduce it:

| | 6r/3w | 9r/5w | 12r/6w | shape |
|---|--:|--:|--:|---|
| cycle-8 document (inferred) | 94,038 | 48,310 | 20,507 | **collapses** |
| C9-05 baseline, measured here | 147,306 | 156,686 | 160,838 | **rises** |
| ratio | 1.6× | 3.2× | **7.8×** | |

The baseline at twelve connections is **7.8× the number the task existed to recover**, and the
curve rises with concurrency instead of falling. The slowest-read view wait tells the same
story: the document's 22,160 µs against a measured baseline maximum of **788 µs**, 28× smaller,
and a median-across-replicates maximum of 321 µs.

**There was no collapse to recover.** A change cannot deliver 1.25× against a baseline that was
never at 20,507/s.

What differs between the two harnesses is exactly what C9-00…C9-01 repaired: the cycle-8 run
used one daemon for a whole session with `ENGINE_LOCK` and `VIEW_LOCK` histograms never reset,
so level *n*'s figures contained levels 1…n−1 (F-75); its counters were lifetime totals rather
than level-local; and the certification-interval repair (T-02) had not yet taken the fallback
rate from 87.4% to 0.00%, so nearly every keyed read was folding the base twice. The falling
curve and the 22 ms wait are properties of that harness, not of the engine.

## 3. Where the time actually goes

In the slowest-16 tables, on **both** arms and at every level, `base wait` is the dominant term
and frequently the whole of it. Two rows, quoted as they printed:

```
candidate 12r/6w, replicate 4        baseline 12r/6w, replicate 5
rank total base view hold            rank total base view hold
   0  1098 1097    0    0               0  1112  794  313    3
   1  1093 1093    0    0               1   927  136  788    1
   2  1089 1089    0    0               2   908  829   70    9
```

`answer_from_view` takes the base read guard for its whole body — in both arms, deliberately,
because `RwLock` is not reentrant and a keyed read must take the base exactly once. A read guard
does not block another reader, but a writer arriving behind it does: the appender queues, and
every reader arriving after the appender queues behind *that*. The tail this engine has at
twelve connections is a **base-lock** tail, and it is not the one C9-06 was aimed at.

**LC-23 is therefore reopened.** Its closing attribution — "the tail is the view mutex holding V
across the fold" — was inferred from an uncorrected harness and is not supported by the corrected
one. The candidate's view waits are not zero either (12r6w maxima 377 / 26 / 77 / 683 / 1,589 µs
across the five replicates, against the baseline's 321 / 297 / 542 / 299 / 788), which is what a
handful of reads contending on two short holds looks like rather than one long one.

## 4. C9-06.3 is met, at the wire, on the reference host

`pending_joins` at 12r/6w, five candidate replicates:

| replicate | pending_joins | uninstalled_folds | pinned_installs | flights_refused |
|--:|--:|--:|--:|--:|
| 1 | 14,245 | 495 | 320,255 | 0 |
| 2 | 13,731 | 492 | 311,921 | 0 |
| 3 | 14,401 | 554 | 303,653 | 0 |
| 4 | 18,334 | 810 | 307,637 | 0 |
| 5 | 14,377 | 487 | 318,865 | 0 |

The absence lattice's fourth state is reached by the served daemon under load on the reference
host. `MISMATCH-pending-unreachable` is **removed**, not narrowed.

The other two columns are the finding. Against ~4.9 M reads in a 30-second level, a joined read
is **0.29%** and a pinned install is **6.35%**: under a moving frontier, twenty-two
reconstructions land pinned for every one that is shared. A pinned entry serves the single
anchor it was asked for and is rebuilt by the next reader. The deferred merge (`deferred_merges`,
zero here) is what would let those land current, and these ratios say it is worth far more than
the join is.

## 5. What is kept, and on what grounds

The two-phase read stays, and its justification in the thesis changes. It is kept because it
makes the lattice's fourth state reachable and counted (a claim the thesis makes and the engine
could not support), and because it bounds a hold that had no bound. It is **not** kept as a
throughput result, and no phase-diagram cell may be drawn from it: on this host, at these
levels, it is worth +1.8% and that is all it is worth.

## 6. Reproduce

```
bash ~/Documents/niles/docs/audit/cycle-9/hostc/c9-pending.sh
```

Raw logs and per-replicate CSVs: `~/c9-pending-out/` on Host C. The script never writes a
tracked artefact and refuses `--publish`; every figure above was transcribed from its transcript.
