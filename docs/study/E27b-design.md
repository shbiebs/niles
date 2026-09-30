# E27b — a fair Nilestream, a fair metric: design, pre-registered

*Cycle 15, round 2, cards C15-02 (this document's part 1), C15-03 (part 2: k views per account and
the spill arm) and C15-04 (part 3: the 10⁶ point on Host C); the author's decision DA-3. This
document is committed on its own, before the engine is touched. It fixes the metrics, the arms,
the size sweep, the k-view axis, the engine rule, the refusal conditions and the changes that are
made to Nilestream before it is measured. A change to any of them after this commit is a
deviation, listed in `results/E27b-comparator.md` with its reason, and never made after seeing a
number it would move.*

## 1. Why E27 is measured again

E27 (`results/E27-comparator.md`, `results/E27-p99-engine.md`; specification §5 of the cycle-14
round-2 work order, binding here except where this document changes it) evaluated the engine rule
and found N justified at no size. The cycle-15 audit (DA-3) found three ways in which that
measurement was not of the engine as designed:

1. **Multi-currency reads never used the view.** `answer_from_view` refuses whenever the base
   holds more than one currency, including for a read grouped by `(acct, cur)`, which the
   `(account, currency)`-keyed balance view can answer. Every keyed read of N on the `multi`
   series folded the base, and nothing was resident (E27's own Deviations say so).
2. **The memory clause compared unlike things.** It set N's process PSS, which includes N's
   in-memory base, against the PSS of a PostgreSQL cluster whose base is on disk.
3. **q3 and q4 could not run on N**: its served relation had no value day and no desk.

E27 is not overwritten. E27b is a new result beside it.

## 2. What changes in Nilestream before it is measured (and nothing else)

Each lands as its own commit after this one, with a guard test and, where the work order asks, a
sabotage control. These are the only engine changes the card makes.

| # | change | guard |
|---|---|---|
| E1 | **The multi-currency view.** A read grouped by `(acct, cur)`, or restricted to one currency by its predicate, is answered from the `(acct, cur)`-keyed view when the base holds more than one currency. A read that sums one account over several currencies stays refused, with a message saying why. | a two-currency `(acct, cur)` read answered from the view (`explain` path and the resident count); sabotage: the old refusal put back fails it |
| E2 | **View-state bytes** exposed over the wire (§4.1), on N and on H3's sidecar, which hosts the same runtime | the figure is zero before any read, grows with installs, stays bounded by the budget under eviction |
| E3 | **Value dates.** N's served relation gains `value_date` (an integer day in the universe's calendar, as the PostgreSQL arms' `value_day`), written through the wire's insert path and stored by the ledger with each posting. A back-valued correction is a posting with an earlier value date and a later epoch. The valid-time read on the SQL surface is a predicate on `value_date` (the pipeline's `.valid_at(@date)` is not the served surface, and the SQL surface has no valid-time clause) | a back-valued correction changes a valid-time read at the head and does not change an `as of` read at an epoch before it |
| E4 | **Desk.** N's served relation gains `desk` (integer), the same way, so q3 and q4 run on N | q3 and q4 answer on N and match the oracle |

The inserts N receives gain the two columns: `insert into postings values (txn, acct, cur, amt,
value_date, desk), …`. A four-column insert stays accepted (value date and desk null), so no
existing client breaks.

Everything else in N (the fold, the scan restriction, the view, eviction, checkpoints, durability)
is as it is at this commit's parent, `49bd011`, which carries C15-05b's changes (the author's
decisions 4, 5, 6, 8, R2-d and R2-e), made before this card by decision R2-c so that E27b measures
the engine once.

## 3. Arms

**Main sweep (part 1):** N, P+, P, M and H3, as E27 §"The arms" defines them, with N as changed in
§2 and nothing else changed. No arm is tuned.

- **H1 (ReadySet), H2 (pg_ivm) and T (TigerBeetle) are not re-run.** H1 and H2 do not enter the
  engine rule; T informs the write floor only, and E27's T figures stand, cited and never pooled.
  This keeps the 10⁵ points within the 8 GB host, where E27 had to drop arms.
- **Targeted p99 re-run:** N, P+ and H3 with E27-p99-engine's shape (§6), because the engine rule's
  p99 clause is judged there, as in E27.
- **Part 2 (C15-03):** the same arms plus the spill arm (Feldera) if the author approves the
  download, with k views per account (§7).
- **Part 3 (C15-04):** N and P+ at 10⁶ on Host C, counted work only, PostgreSQL 16.

## 4. Metrics

### 4.1 The memory metric: view-state bytes per key read

**View-state bytes** are the bytes of the derived state an arm holds so that it can answer reads:
resident entries, their metadata, and the per-key checkpoints reconstruction starts from. They are
**not** the base postings. Per arm:

| arm | view state | instrument |
|---|---|---|
| N | the balance view's REV (its resident slots, per-key metadata and in-flight reconstructions) **plus** the ledger's per-key checkpoints | **in-process allocation accounting**: the daemon runs under a counting global allocator; on request, it clones those structures under the view lock and reports the bytes the clone holds live, then drops it. A clone reproduces each B-tree's node structure exactly and allocates each vector at its length, so the figure **excludes spare vector capacity** and is a lower bound by that slack. |
| H3 | the sidecar's REV (the same runtime; H3 has no checkpoints) | the same instrument, in `rev-sidecar` |
| P+ | tables `arm.rev`, `arm.rev_meta`, `arm.key_counts`, `arm.checkpoints` | `pg_total_relation_size` of each (heap, indexes, TOAST, free-space and visibility maps), summed; it includes dead tuples and free space, which are that arm's real cost |
| P, M | the materialised view `arm.balances` | `pg_total_relation_size` |

**View-state bytes per key read** = view-state bytes at the end of a seed's last measured run ÷
the distinct keys read by then. It is judged **across seeds** (the joint gate over the five
per-seed values, no warm-up floor), as E27 judged its PSS metric, because the per-run values trend
with the keys read so far and are not replicates. Reported beside it, descriptive: view-state
bytes, resident keys, and bytes per resident key.

**Process PSS** (E27's metric) is kept as a **secondary** metric, labelled *includes base storage*
for N and H3's sidecar (whose base is PostgreSQL's) and *the cluster* for PostgreSQL arms. It does
not enter the engine rule.

### 4.2 Everything else

As E27 (§5.5 of the cycle-14 order, E27's `render.rs` metric list): p50/p99 per query at 1, 2 and
4 clients, the two-leg transfer's p50/p99, commits/s at fsync, base rows per reconstruction,
oracle divergences. **p99 read latency under eviction** for the rule is q1 and q2 at 1 and 4
clients from the targeted re-run (§6), as in E27.

## 5. Data, sweep and statistics

Unchanged from E27: the declared synthetic universe (α = 0.6; ten transactions of history per
account in batches of 200, one epoch per batch; 3% back-valued up to 22 business days; `multi`:
35% of accounts hold a second currency, transfers in it half the time), sizes 10³, 10⁴, 10⁵,
seeds {1, 7, 42, 100, 2024}, both series, budget 5% of keys, LRU, C = 16 on N and P+, the run
shape (3 warm-ups, 10 measured runs, 200 operations at 1 client with every tenth a two-leg
transfer, then 2 and 4 concurrent clients over q1/q2), the read mix, interleaving rotated per
run, the joint gate (≥ 10% and ≥ 3 pooled MADs), the floor from warm-up (3 × pooled MAD), and the
size-verdict rule of `crates/comparator/src/stats.rs`. The only change to the workload is that
N's inserts carry the value date and desk every other arm already receives (§2), and that q3 and
q4 are asked of N:

| query | N's text |
|---|---|
| q3 one account's month statement | `select count(amt), sum(amt) from postings where acct = {a} and cur = {c} and value_date between {d0} and {d1}` |
| q4 desk exposure by currency | `select cur, sum(amt) from postings where desk = {d} group by cur` |

If either text does not lower after E3/E4, the smallest spelling that does is used and listed as
a deviation, before any measured run.

## 6. The targeted p99 re-run

E27-p99-engine's shape, unchanged: 3 warm-ups and 10 measured runs per (arm, size, seed),
interleaved; a run is 2,000 point reads at 1 client (q1 and q2 half each, no writes), then
4 clients × 500 reads. Arms N, P+, H3. On the `multi` series at 10⁵, q1 only, as in E27 (the
author's decision of 2026-09-28).

## 7. The k-view axis (part 2, fixed here, measured in C15-03)

k ∈ {1, 10, 50} views per account over one base, drawn from the four view families of W8
(outstandings and positions; cashflows and scheduled activity; facility and deal details;
accounting and GL). Reads choose a view by a skew parameter **β** across view definitions,
separate from the per-key α, from C15-06's generator. The added metric is **maintenance cost per
write as a function of k**, in counted work and wall clock. The rule of §8 is evaluated per size
range **and per k**. The view definitions and β values are fixed in C15-03's own pre-registration
commit, before any of its runs.

## 8. The engine rule, unchanged (cycle-15 round-2 work order §6)

Evaluated at the declared universe, on one host, interleaved, **per size range and per k**.

- **N is justified as a product** iff it beats P+ **and** H3 by the joint gate, above the floor,
  on **view-state bytes per key read** **and** on **p99 read latency under eviction**.
- **The hybrid (PostgreSQL ledger + REV sidecar)** is the recommended product iff H3 matches N
  within the gate on both.
- **PostgreSQL with the E14 mechanism** is the recommended product iff P+ matches N on both.
- **The spill arm:** if it beats N on memory and matches it on p99, the report says full state
  plus spill is the simpler answer in that range.
- **T** informs the write floor only.

**Scale (S-1).** A recommendation that holds at one end of the sweep and not the other is two
recommendations, with the crossover between them. It is never averaged into one.

## 9. Refusal conditions

Stop and report, never adjust, if:

- the loaded-data checksums differ between arms (row count and SHA-256 per arm, per seed and size);
- the warm-up MAD exceeds 15% of the median;
- any arm diverges from the oracle.

A comparison below the resolution floor is BELOW FLOOR with no direction. Never tune an arm after
seeing another arm's numbers.

## 10. Output

- `results/E27b-comparator.md` and its point files under `results/E27b-comparator/`, rendered by
  the comparator; `results/E27b-p99-engine.md` for §6. Each carries the provenance header (commit,
  host, toolchain, date, configuration including `checkpoint_interval`, the universe name, α and
  k) and is classified in `results/MANIFEST.csv`.
- Claim **H-E1**'s text and status are updated in `thesis/status.toml` from the table and
  rendered, never typed.

## 11. What this design does not do, and says so

- It does not measure H1, H2 or T again (§3).
- The N instrument of §4.1 is a lower bound by spare vector capacity; P+'s includes PostgreSQL's
  page slack. The asymmetry is stated in the results beside every figure.
- Allocations per read on N (E27 §5.5) stay unmeasured on the served binary; E18 measures them
  in-process.
- The 10⁶ point is part 3's (Host C); the container holds 2 cores and 8 GB.
