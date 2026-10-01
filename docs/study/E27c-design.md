# E27c — the REV with its memory accounted for: design, future work

*Cycle 16, card C16-03; the author's decision DB-3(a), and DB-1. **Not run.** Under DB-1 the
product engine is PostgreSQL whatever the engine comparison says, so E27c is thesis evidence
about the reconstructible epoch-anchored view, not a product decision, and it waits for a cycle
that schedules it. This document records what it would measure and why, so that the question it
answers is not lost and its rules exist before any of its numbers. If it is ever run, it is
pre-registered by a commit that changes only this file's status line and §7's dates.*

## 1. Why E27b does not settle the question

E27b measured N as its design named it, with two faults the cycle-16 audit found in the theory
and not in the harness (`docs/audit/cycle-16`, DB-3):

1. **Holes.** An evicted key stayed in the view as `Hole(version)`, so the view's bytes grew
   with the keys ever read. LC-31 was decided in C16-03: an evicted key is compacted to ⊥. The
   engine E27b measured had holes; the engine after C16-03 does not.
2. **Checkpoints.** For SC7's time bound the ledger keeps a running balance and a posting count
   for every key, Θ(|K|), and a checkpoint every C postings per key, never pruned, Θ(H/C). E27b
   counts this state in N's view-state bytes (its deviations say so). By the restated cost model
   (§3.14, Theorem 3.7(iii)) it makes N's derived state at least full state in order.

So E27b compares P+ and H3 against a REV that pays full state for its aggregates plus history
for its checkpoints. It answers "is N as built justified"; it does not answer "is a REV that
holds only what is read justified", which is the thesis's question.

## 2. Arms

| arm | what |
|---|---|
| N | E27b's N at C16-03's head: holes compacted, checkpoints as before |
| N′-lazy | checkpoints built only for keys read: a key's first read folds it once and starts its checkpoints; no running aggregate for a key never read |
| N′-window | checkpoints pruned outside an anchor window of W epochs behind the head; an anchor older than the window folds from the oldest kept checkpoint or genesis, and says so |
| N′-off | C = 0: no checkpoints and no running aggregates; reconstruction is history-shaped |
| P+, H3 | as E27b, unchanged |

N′-disk (checkpoints in an on-disk structure) is named and not designed here: it moves the
Ω(H/C) of Proposition 3.7.2 off the heap, and its cost is I/O, which needs its own metric.

## 3. Metrics

- **Derived-state bytes per key read**, as E27b §4.1, with checkpoint state counted separately
  and in total, per arm.
- **p99 read latency under eviction**, as E27b's p99-engine shape, at head anchors and at
  historical anchors uniform between a key's first epoch and the head (SC7's distribution,
  which no experiment has sampled: item 14).
- **Base rows per reconstruction**, at the same two anchor distributions.

## 4. The axis E27b does not have: history at fixed key space

Proposition 3.7.2 and the restated Corollary 4.2.1 predict a crossover in **H**, not only in
size: at fixed |K|, as the ledger lengthens, N's checkpoint memory grows as H/C and passes full
state at H\* = C·(|K|(1 − σ) − m). E27c sweeps H at fixed |K| (4×, 16×, 64× the E27b history at
10⁴ keys), as E10 swept it for time. S-1 applies: a result that holds at one end and not the
other is two results with the crossover between them.

## 5. Predictions, stated before any run

1. N's derived-state bytes grow linearly in H at fixed |K|; N′-lazy's grow with the history of
   the keys read; N′-window's are flat in H; N′-off's are flat in H.
2. N′-off's historical-anchor p99 grows with H; N, N′-lazy and N′-window stay flat inside the
   window, and N′-window grows outside it.
3. No arm beats Proposition 3.7.2's bound: any arm with flat memory in H has historical-anchor
   reconstruction time that grows with H, for anchors outside what it keeps.

## 6. The rule, unchanged except for the arms

E27b §8, per size range, per history length and per anchor distribution. Refusal conditions as
E27b §9.

## 7. Status

Future work. Not pre-registered, not scheduled, not run.
