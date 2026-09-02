# E16 — the pre-registered crossover band

**Written and committed before the durable run.** That ordering is the whole point of the
file: a band derived after seeing the measurement is not a prediction, and a threshold
adjusted to fit is the researcher degree of freedom this thesis's own §9 warns about. The
commit that adds this file is an ancestor of the commit that adds the durable CSVs, and
`git log` is the record.

## What Theorem 4.2(ii) asks for

> C_part = C_hot + miss(π, m) · c_u · w · (1 + Θ(Z))
>
> against C_full = C_apply(λ, |K|) + C_mem(|K|). For every (c_u, w, λ, Z) there is a
> threshold miss\* such that miss(π, m) ≥ miss\* implies C_part ≥ C_full.

So a band on miss\* needs five quantities. Four are measurable on this machine and are
stated below with the command that produced them. The fifth is not, and that is the finding
this file records.

## The constants, measured

| symbol | meaning | value | how |
|---|---|---|---|
| c_u · w | base rows touched per reconstruction, in counted work | median **20 467**, range 2 305 – 85 204 over 150 points | `results/e4_phase.csv`, column `rows_touched_partial` |
| C_apply(λ,\|K\|) | deltas applied under full materialization | median from the same file's `deltas_full` | `results/e4_phase.csv` |
| C_mem(\|K\|) | resident entry-epochs under full materialization | `resident_entry_epochs_full` | `results/e4_phase.csv` |
| fsync cost | the device's durability barrier | **77.9 µs**, ceiling 12 845 durable commits/s per connection | `bench --calibrate`, probing `/var/lib/postgresql/16/main` |

Counted work, not time. The distinction is the one §9 insists on everywhere else and it
matters here: a crossover stated in rows is machine-independent, and one stated in
microseconds is a statement about this container.

## Why there is no two-sided band: the constant inside Θ(Z)

**Θ(Z) has no stated constant anywhere in the thesis, and inventing one is the thing this
file exists not to do.**

Z is the delayed-hit factor — the number of inter-arrival times a reconstruction takes —
and the term `(1 + Θ(Z))` is written asymptotically. Asymptotic notation is the right form
for the *theorem*: clause (ii) says a threshold exists for every (c_u, w, λ, Z), and the
existence does not depend on the constant. It is the wrong form for a *prediction*, because
a band computed with the constant set to 1 and a band computed with it set to 10 differ by
an order of magnitude in exactly the region the experiment is measuring.

§4.3's corroboration cites Atre et al.'s Ω(kZ) competitive lower bound for deterministic
online algorithms, which has the same shape and also carries no constant. Appendix D's cost
model gives the structure and not the coefficient.

**So: no two-sided band is derivable.** What is derivable is one side of it.

## The one-sided prediction, pre-registered

Setting Θ(Z) = 0 — no delayed hits, every reconstruction serving one waiting reader — gives
the **most favourable case for partial materialization**, and therefore a *lower bound* on
the crossover:

> **Prediction P1.** With Θ(Z) = 0, partial materialization is cheaper than full whenever
>
>     miss · (c_u · w) < deltas_full + resident_entry_epochs_full − C_hot
>
> and on the E4 constants above this places miss\* **above 0.5** for every point whose
> `budget_frac` ≥ 0.05. Equivalently: at a miss rate below one in two, the single-key fold
> should be cheaper partial than full, in counted work, at every budget fraction of 5% or
> more.
>
> **P1 is falsified** if any E4 or E12 point with `budget_frac` ≥ 0.05 and `hit_rate` ≥ 0.5
> reports `rows_touched_partial` ≥ `deltas_full + resident_entry_epochs_full`.

> **Prediction P2.** The crossover is *not* observable in the E16 wall-clock `point`
> workload as configured, because that workload's budget and key count fix a single miss
> rate rather than sweeping one. E16 measures a point on the curve; E4 and E12 measure the
> curve. **P2 is falsified** if `results/E16-wallclock/point.csv` reports two runs at the
> same configuration with materially different miss rates.

> **Prediction P3, and the one that would matter most.** With Θ(Z) unstated, the *upper*
> side of the band cannot be predicted, so a measurement showing partial materialization
> losing at a miss rate below P1's threshold would **not** contradict Theorem 4.2 — it would
> locate the missing constant. If that happens the honest report is "Θ(Z)'s constant is at
> least X on this workload", derived from the measurement, and §4.3 gains it as a measured
> quantity rather than an asymptotic one.

## What this file is not

It is not a target. Nothing in `bench.rs`, `experiments/`, or the engine reads it, and no
run is tuned toward it. It is a statement made before the measurement about what the
measurement could show, so that afterwards there is something for the result to disagree
with.
