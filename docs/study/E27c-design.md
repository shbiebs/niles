# E27c — the REV with its memory accounted for: design, future work

*Cycle 16, card C16-03; the author's decision DB-3(a), and DB-1. **Pre-registered 2026-11-11 (cycle 17, C17-11; decision D17-2); not run.** Under DB-1 the
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

**Pre-registered 2026-11-11** (cycle 17, card C17-11; the author's decision D17-2 of 2026-10-10), before any E27c number. Not run.

- **Arms:** N, N′-lazy, P+, H3 and P-inc (cycle 17's C17-03). N′-window and N′-off are not run: D17-2 chose N′-lazy only.
- **The rule:** E27b's §8, unchanged, with P-inc added as an arm; refusals as E27b §9.
- **Sizes and skew:** 10⁴ to 10⁶ accounts; α ∈ {0.6, 1.1}.
- **Where:** M-NOISE (cycle 17, C17-04; `results/M-NOISE.md` on niles branch `c17/04-m-noise`) found the container's refusals a host effect, not a run-length one, so every E27c comparison at 10⁵ and above runs on the author's Mac (Host C, D17-7), and this container runs only the smoke. Run lengths are E27b's.
- **Before the run:** N′-lazy is built behind a daemon flag whose default leaves N as E27b measured it, and passes the reconstruction-equivalence and conservation suites under the lazy policy.

### 7.1 Run 3's choices (2026-11-12)

**Pre-registered 2026-11-12** (cycle 17, card C17-12 run 3; the author's decisions of 2026-11-12, recorded in the project's `cycle-17-decisions.md`), before any E27c point, before the harness changes for E27c and before its run script. Nothing above is changed; this entry fixes what §§2–7 left open.

1. **Where, and which series.**
   - **Host C** (the author's Mac, arm64, 16 GiB; D17-7): single-currency at 10⁵ and 10⁶, multi-currency at 10⁵. **Multi-currency at 10⁶ is not run**, because on 16 GiB it would swap. That is a stated limit of E27c, not a result.
   - **The container:** the history sweep of §4, single-currency at 10⁴ accounts, at 1×, 4×, 16× and 64× E27b's history (10, 40, 160 and 640 transactions per account; 1× is E27b's declared history and the sweep's reference). Its view-state bytes and base-row counts carry predictions 1 and 3 of §5. Its latency cells are subject to the container's noise (`results/M-NOISE.md`) and are reported with that beside them.
   - **Not run:** 10⁴ multi-currency, and 10³. They are not in the author's list of 2026-11-12.
2. **Skew:** α ∈ {0.6, 1.1} in every series, size and history length, as §7 fixed.
3. **Arms and builds.**
   - **N and N′-lazy** are the same metered daemon (`nilestreamd-metered`, built by `tools/memprobe`, as E27b's N). N′-lazy is started with `--checkpoint-policy lazy` (C17-11), on its own port and data directory.
   - **H3** is E27b's: `rev-sidecar-metered`, built by `tools/memprobe` on the host that runs it. The build is checked in the container first.
   - **P+ and P-inc** are as E27b and C17-03, each on its own cluster with §5.3's settings.
   - Every comparison is within one host.
4. **Shapes and run lengths:** E27b's, unchanged.
   - **The main shape:** §5 of E27b (3 warm-ups, 10 measured runs, 200 operations at one client with every tenth a two-leg transfer, then 2 and 4 clients over q1 and q2).
   - **The targeted p99 shape:** §6 of E27b (2,000 point reads at one client, q1 and q2 half each, no writes, then 4 × 500). On the multi-currency series at 10⁵ it asks q1 only, as E27b §6 does.
   - Budget 5% of keys, LRU, C = 16.
5. **Metrics, made precise.**
   - **Derived-state bytes per key read:** E27b §4.1's, per arm. **Checkpoint state** is reported separately:
     - N and N′-lazy: the daemon's `checkpoint_bytes` (the ledger's per-key running balances, posting counts and checkpoints, as a clone under the metering allocator);
     - P+: `pg_total_relation_size` of `arm.checkpoints` and `arm.key_counts`;
     - H3 and P-inc hold none.

     The total is E27b's view-state bytes, which includes it.
   - **Head and historical anchors:** q1 asks at the head. q2's anchor is drawn uniformly between the key's first epoch and the head (`crates/comparator/src/run.rs`, `draw_read`), which is §3's historical distribution. E27 and E27b already drew q2 this way, for latency. What neither recorded is base rows per reconstruction for each distribution, which is what item 14 lacks.
   - **Base rows per reconstruction:** after each main-shape point's measured runs, and outside every timed region, a counted pass runs on each arm.
     - It asks 200 q1 reads, then 200 q2 reads, with the same keys and anchors on every arm. Every answer is checked against the oracle; a divergence there refuses that arm, as §9 of E27b does.
     - The arm's `misses` and `rows_touched` counters are read before and after each half. Base rows per reconstruction is Δ`rows_touched` ÷ Δ`misses`, per half.
     - N, N′-lazy and H3 count these. P+ and P-inc do not, and are reported as *not counted*, never as zero.
6. **The rule (§6), read before any number.** E27b §8 with P-inc added. It is evaluated for each REV arm R ∈ {N, N′-lazy} separately, per size range, per history length and per anchor distribution (q1; q2):
   - **R is justified** iff it beats P+, H3 **and** P-inc by the joint gate, above the floor, on view-state bytes per key read **and** on p99 read latency under eviction. The latency is the p99 shape's query for that distribution, at 1 and 4 clients.
   - **H3 is recommended** iff it matches R within the gate on both. **P+** likewise iff it matches R on both, and **P-inc** iff it matches R on both.
   - Where the p99 shape asks no q2 (multi-currency at 10⁵), the historical distribution's verdict is **NOT RUN**, with the reason.
   - Refusals are E27b §9's. Under C17-02, an arm whose end-of-run check was not recorded is refused.
   - S-1 applies across sizes and across history lengths.
   - §5's predictions are checked descriptively beside the rule, from the points: checkpoint bytes and base rows per reconstruction against H, for N and N′-lazy.
7. **Host C protocol** (D17-7; work order §6.3).
   - The author quits Claude and its VM, keeps the Mac on mains power with other applications closed and no Time Machine backup, and runs the script under `caffeinate`.
   - The script refuses to start above a one-minute load average of 1.5, as runs 1 and 2 did.
   - It records the load averages and `vm.swapusage` before and after every point. **A point during which swap in use grew is labelled swap-affected.** Its latency and memory figures are not compared (as E27-hostc's were not), and the result says so.
   - **Order:** single 10⁵, then multi 10⁵, then single 10⁶. Within each: α 0.6, then 1.1; seeds 1, 7, 42, 100, 2024; per (α, seed), the main point, then the p99 point. The run resumes per point.
8. **Rendering.** The rule and the metrics are fixed by this entry. The renderer is written in the session that reads the points, from this entry, without changing it. Any presentation choice made after the points exist is recorded as a deviation.
