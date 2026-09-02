# Appendix I. The Adaptive Materialization Optimizer: Algorithms and Analysis

This appendix gives Contribution 5 as implementable algorithms, with estimators, guarantees, and the offline procedure against which the online algorithm is graded.

## I.1 The Decision, Stated

For each (view V, key range R), choose a mode μ ∈ {absent, demand, full, spilled, tiered} minimizing expected total cost subject to (a) V's consistency and freshness contract, (b) the global memory budget, and (c) pinning constraints.

**Why key *ranges* rather than keys.** Per-key decisions cost more to track than they save for cold keys, and per-view decisions are too coarse for a skewed distribution where a view's hot head and cold tail want different modes. Ranges are formed by the key's natural order (account identifier prefix, time bucket, hash bucket) and split or merged when their internal statistics diverge or converge — the same adaptive-refinement idea that adaptive indexing applies to physical layout, applied here to residency decisions.

## I.2 Estimators

Per range R, maintained with exponentially weighted moving averages and periodic decay:

| Symbol | Meaning | Update |
|---|---|---|
| λ_r(R) | Read arrival rate | EWMA over a sliding window |
| λ_w(R) | Write/update rate touching R | EWMA |
| ĉ_u(R) | Reconstruction cost (upquery path evaluation work) | EWMA of measured upqueries |
| L̂(R) | Reconstruction *latency* | EWMA of measured upquery service time |
| **Ẑ(R)** | **Delayed-hit ratio = L̂(R) · λ_r(R)** | Derived |
| ĉ_a(R) | Per-epoch maintenance cost if materialized | EWMA of measured application |
| s(R) | Residency bytes | Measured |
| d̂(R) | Reuse-distance estimate | Sketch |
| Φ(ℓ) | Contract multiplier | From the view's contract (§3.14) |

Ẑ is the parameter the classical caching results omit and the delayed-hit literature identifies as decisive; it is measured directly rather than assumed, and it is reported with every experimental result (Section 5.5).

## I.3 Mode Selection: Rent-or-Buy with a Contract Multiplier

```
on read-miss(R):
    spend[R] += ĉ_u(R) · (1 + Ẑ(R)) · Φ(ℓ_V)
    if spend[R] ≥ buy_cost(R):
        propose(R, full)                       # break-even rule
        spend[R] = 0

buy_cost(R) = ĉ_a(R) · horizon + s(R) · residency_price

on periodic review(R):
    if materialized(R) and utility(R) < downgrade_threshold and dwell(R) ≥ min_dwell:
        propose(R, demand)
```

**Guarantee (Theorem 4.5(d)).** With `buy_cost` fixed and costs stationary, the deterministic break-even rule is 2-competitive against the offline optimum for the two-mode decision, and a randomized variant achieves a ratio approaching e/(e−1) ≈ 1.58, which is optimal for this class. The randomized variant is implemented behind a flag and compared in H-S7.

**Honest scope.** The guarantee is for the two-mode (demand versus full) decision under stationary costs. It does not survive non-stationarity, and it does not extend as stated to the five-mode lattice; I.6 says what does.

## I.4 Eviction Within `demand`: Cost-, Size- and Delay-Aware

Plain LRU is wrong here for two independent reasons: entries differ in reconstruction cost (upquery depth varies by circuit), and reconstruction takes time during which further requests queue.

```
# Landlord-style credit, extended with the delayed-hit weighting
on admit(k):        credit[k] = ĉ_u(range(k)) · (1 + Ẑ(range(k))) · Φ(ℓ_V)
on pressure(δ):     for all resident k: credit[k] -= δ · s(k)
                    evict all k with credit[k] ≤ 0
on hit(k):          credit[k] = min(credit_max, ĉ_u · (1 + Ẑ) · Φ)   # refresh
```

**Guarantee.** With uniform Ẑ, this is the file-caching discipline whose competitive bound is the resource-augmented k/(k−h+1) form — the correct generalization of the paging bound to heterogeneous size and retrieval cost. With non-uniform Ẑ the objective becomes aggregate delay, where the literature establishes that hit-rate-optimal policies (including the offline optimum for hit rate) are not latency-optimal, and reports a deterministic competitive lower bound of Ω(kZ) attributed to parallel work. **This thesis therefore claims no optimality in the delayed-hit regime** — only that a policy ignoring Ẑ is provably wrong, and that weighting by expected aggregate delay is the correct direction.

## I.5 Spilled and Tiered

`spilled` and `tiered` enter when a range's residency cost is dominated by *size* rather than by maintenance:

```
cost_demand(R) = λ_r(R) · miss(R) · ĉ_u(R) · (1 + Ẑ(R)) · Φ(ℓ)
cost_full(R)   = λ_w(R) · ĉ_a(R) + s(R) · residency_price
cost_spilled(R)= λ_w(R) · ĉ_a(R) + s(R) · storage_price + λ_r(R) · io_cost(R)
cost_tiered(R) = hot_share(R) · cost_full(R_hot) + (1 − hot_share(R)) · cost_spilled(R_cold)
choose argmin subject to contract feasibility
```

Feasibility is a hard filter, not a penalty term: a mode that cannot meet the view's rung — for example `spilled` where the I/O latency would breach an ℓ₅ authorization path's budget — is removed from the candidate set before comparison. This is what makes Theorem 4.5(a)'s safety clause hold operationally as well as formally.

## I.6 Hardness, and What Replaces the Classical Guarantee

**Hardness.** The offline problem is NP-hard: restricted to two modes with unit residency and a linear per-read cost model it is view selection under a space budget, NP-complete by reduction from set cover.

**The negative result.** The classical greedy guarantee of (1 − 1/e) for view selection assumes a cost model in which answering a query costs the size of the view used. Partial materialization violates that assumption — cost depends on miss rate and reconstruction depth — so benefit is not submodular in general and the guarantee does not transfer. This is reported as a finding rather than elided; it is the most-cited guarantee in the view-selection literature and it does not survive the move to partial state.

**Two recoveries.**

1. *Planning time.* With miss rates held fixed per range (estimated from a workload model rather than updated online) **and range benefits independent** — no two ranges sharing an upquery path, so that no range's benefit depends on which others are resident — the benefit function is additive, hence submodular, and greedy recovers the (1 − 1/e) bound. Both hypotheses are needed; fixing the miss rates alone does not give submodularity. This would be used to compute initial mode assignments at deploy time; nothing computes them, because the optimizer is not built.
2. *Runtime.* The online per-range analysis of I.3 and I.4 replaces the global guarantee, trading a bound on the *assignment* for bounds on each *decision*.

## I.7 Hysteresis, Thrash Avoidance, and Safety

```
min_dwell(mode)             # minimum time in a mode before leaving it
sustained_signal_window     # a proposal must persist across the window to apply
budget_reserve              # headroom so promotion never forces immediate demotion
pinned ranges excluded
```

Three safety properties hold regardless of estimator quality:

- **Semantic safety.** A mode change cannot change any answer (Theorem 4.5(a)) — proved once, so no individual heuristic needs a correctness argument.
- **Contract safety.** Infeasible modes are filtered before comparison, so a bad estimate can cost money but cannot breach a contract.
- **Explainability.** Every transition is logged with the estimator values that caused it, so a surprising decision is diagnosable. An adaptive component that cannot be interrogated is operationally unacceptable, whatever its average-case behaviour.

## I.8 The Offline Optimum (for Grading)

H-S7 measures the online algorithm against an optimum computed offline over the recorded trace:

```
# States: (mode assignment for each range at each decision epoch)
# Transitions: mode changes with their switching costs
# Cost: measured per-operation costs from the trace
# Solve by dynamic programming over epochs, per range, with a
# Lagrangian relaxation of the global budget coupling; sweep the
# multiplier to trace the budget-feasible frontier.
```

The relaxation is necessary because the budget couples ranges; the sweep gives a lower bound on achievable cost at each budget level, which is what the ratio in H-S7 is computed against. Where the relaxation gap is non-zero it is reported, so that a ratio is never presented as tighter than the bound supports.

## I.9 What Would Falsify Contribution 5

- A fixed policy beating the optimizer across the whole workload suite would mean the estimators cost more than they save, reducing C5 to a planning-time heuristic (and this is stated in Section 11.3 as a mind-changing result).
- Mode thrashing under phase changes despite hysteresis would indicate the signal model is wrong, not merely mistuned.
- A measured competitive ratio outside the analytical bound in the *stationary, no-latency* regime would indicate an error in the analysis, since that is the regime where the classical results apply directly.
- An observable behaviour change across a mode transition would falsify Theorem 4.5(a) and would be the most serious failure in this appendix, since every other safety property rests on it.
