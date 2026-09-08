# LC-23, answered: where the mixed-read tail goes

*Measured on Host C (Apple M4, 10 cores, APFS on NVMe, `F_FULLFSYNC`), 2026-09-07, arm A at
`d8ad061`, by `run6.sh --only D`. Raw output in `~/Documents/niles-hostc/results6-20260907-195628/`.*

LC-23 asked where a mixed workload's 12–13 ms read maximum goes, given that the base lock's
longest wait was 1.7 ms. Work order 8 offered a hypothesis and this measurement **refutes
it**. It said:

> Read max equals write max to within 1%, which is the shape of *one reader per run* landing
> behind the writer's barrier — through the OS scheduler, since no lock chain reaches the
> fsync. […] If it holds, the tail is the Mac's `F_FULLFSYNC` and a core count, not the
> engine.

It does not hold. `unaccounted_us` — the part of a server-side keyed read that is not waiting
for the base guard, not waiting for the view mutex, and not holding the view — is **0 to 2 µs
in every one of the 160 slowest reads recorded**, at every shape. The scheduler and the wire
are not where the time goes. It is all inside the engine's two locks.

## What the levels did

| level | reads/s (run 1 → run 5) | read p50 | writes/s |
|---|---|--:|--:|
| 6r/3w | 94,038 → 84,384 | 37 → 51 µs | 520–567 |
| 9r/5w | 48,310 → 29,843 | 176 → 297 µs | 732–828 |
| 12r/6w | 20,507 → 15,847 | 596 → 795 µs | 879–908 |

**Read throughput falls as readers are added** — 94k at six connections, 48k at nine, 20k at
twelve — and p50 rises sixteenfold across the same range. Within each level it decays run over
run as the base grows. That is not a saturating curve; it is a queue.

## Where the slowest reads waited

Worst read per shape, from the tables `bench` printed:

| shape | worst total | base wait | view wait | view hold | unaccounted |
|---|--:|--:|--:|--:|--:|
| 6r/3w | 591 µs | 0 | **496** | 95 | 0 |
| 9r/5w | 3,419 µs | **3,340** | 67 | 12 | 0 |
| 12r/6w | 22,775 µs | 442 | **22,160** | 172 | 1 |

Two populations, and both are locks. Some reads wait on the base guard (`append` holds it
exclusively while it applies an epoch); more wait on the view mutex. At twelve connections
both reach ~21 ms.

The third column is the one that explains the rest. Several reads **hold** the view for over
a millisecond — 1,412, 1,477, 1,132, 1,119 µs — where a keyed read against resident state is
microseconds.

## The mechanism

`Rev::read` reconstructs on a miss:

```rust
self.stats.misses += 1;
self.stats.upqueries += 1;
let (value, rows) = base.reconstruct(key, anchor);   // a fold over the base
self.install(key.clone(), value, anchor);
```

and `answer_from_view` calls it **while holding the view mutex**. So an upquery — whose cost
is proportional to the key's history — runs inside `V`, and every other reader queues behind
it. With a residency budget of 2,500 against 10,000 accounts, three keys in four are evicted
at any moment, so this is the steady state and not a rare event.

That accounts for all three observations together: the millisecond view *holds* are single
reconstructions; the twenty-millisecond view *waits* are queues of readers behind them; and
the decay within a level is each reconstruction folding a longer base as the run proceeds.

## The lattice already has the answer, and the runtime never reaches it

Chapter 3 defines four states, ⊥ ⊏ Hole(e) ⊏ Pending(e) ⊏ Present(v, e), and says of the
third:

> `Pending(e)` exists because reconstruction is not instantaneous. A second reader arriving
> during an upquery must join it rather than start a second one, and must not see an absence
> and conclude anything.

**`Slot::Pending` is never constructed anywhere in the runtime.** It appears in `absence.rs`,
in that module's own unit tests, and nowhere else. The state whose entire purpose is to let a
reconstruction happen *outside* the lock while other readers join it is unreachable, so the
engine implements three states and the thesis describes four. Marked
`MISMATCH-pending-unreachable`.

That is also the repair, and it is a design change rather than a tuning knob: mark the slot
`Pending(anchor)`, release the view, fold the base, re-acquire, install; a second reader for
the same key at the same anchor joins the first instead of starting its own. It needs its own
task, its own guards, and a before-and-after on this host — the certification interval is the
central object of the thesis and it must not be weakened to buy throughput.

## What this changes elsewhere

* **T-08 / F-28 — the fold plateau at 8–16 connections, and LC-24's unstable p99.** Both are
  below the cut line and both now have a candidate cause that is measurable rather than
  speculative. They should be re-read against this before any new experiment is designed.
* **The base guard is not the first thing to change.** Work order 8 wondered whether `append`
  holding the base write guard across `advance` was the problem. It contributes — the 9r/5w
  worst read waited 3.3 ms on the base — but the view mutex is both larger and structurally
  avoidable, and touching the lock order to chase the smaller of the two would be the wrong
  order of work.
* **The E19 `point` scaling curve was read as a scaling result.** If the same serialisation is
  underneath it, the curve is a property of this lock and not of the engine's concurrency.
