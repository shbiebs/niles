# 9. Evaluation: Design, Measured Results, and What Remains Unmeasured

This chapter has two halves, and the boundary between them is stated before anything else so that no reader mistakes one for the other.

**§9.1–§9.4 report measurements that were actually taken.** They come from a research prototype (Appendix K) that implements the mechanisms this thesis is about — epoch-ordered commit under a per-currency zero-sum rule, hash chaining, per-key anchor indices, partial materialization over the absence lattice, anchored upqueries, eviction policies, and per-key checkpoints. Every number in these sections was produced by the run that reports it, on the platform of §9.1.2, and every artifact is in `results/` with the code that generated it.

**§9.5–§9.12 are the evaluation design for the parts not yet built.** They state protocols, baselines and pre-registered predictions with their theoretical sources. Every cell there is marked *to be measured*, and no number in this thesis is invented to fill one.

Three of the measured results **contradict hypotheses this thesis previously asserted**. They are reported as contradictions, in the sections where the hypotheses were stated, because a design document that only reports confirmations is not evidence of anything.

## 9.1 What Was Measured, and On What

### 9.1.1 Scope of the prototype, stated as a limitation first

The prototype is a single-node, in-memory, single-threaded Rust program of roughly 900 lines. It has **no durability** (no `fsync`, no write-ahead log), **no concurrency control** beyond being single-threaded, **no consensus**, **no query planner**, **no SQL surface**, and only two view shapes. It is not Nilestream; it is the smallest artifact that makes Nilestream's *mechanisms* measurable.

What follows from that scope is a discipline about what can be claimed. The prototype can support claims about **counted work** — base rows read, deltas applied, resident entries held over time, upqueries issued — because those are properties of the algorithms and the workload and are reproducible on any machine. It cannot support claims about throughput relative to a production DBMS, and none are made here. In particular, this chapter does **not** fill the comparison cells against TigerBeetle-class ledgers, distributed strictly-serializable SQL, or incremental-view engines: those remain *to be measured* in §9.5, and a prototype without durability or concurrency cannot honestly stand in for them.

This is the choice the systems-benchmarking literature calls for. Reporting a microbenchmark as if it characterized a system is a named benchmarking error, as is comparing a system only against itself; the design below answers both by keeping the unit machine-independent and by always comparing against a full-materialization baseline run on the identical workload and seed, plus — for eviction — against the randomized policy that the original partial-state work actually used.

### 9.1.2 Platform

| | |
|---|---|
| CPU | Intel Xeon @ 2.80 GHz, 2 vCPU (shared virtual machine) |
| Memory | 7 GiB |
| Kernel | Linux 6.18.44 x86-64 |
| Toolchain | rustc 1.95.0, release profile (optimized) |
| Storage | virtual disk; **not exercised** — the prototype is in-memory |

This is a modest, shared, virtualized environment. It is adequate for counted-work measurements, which do not depend on it, and it is *not* adequate for absolute-throughput claims, which is one more reason none are made. Wall-clock appears exactly once, in §9.4.4, with that caveat repeated.

### 9.1.3 Protocol

Five seeds throughout (1, 7, 42, 100, 2024), fixed in advance. Medians are reported with the observed range; no result rests on a single run. The pseudo-random generator is a stated linear congruential generator rather than a library default, so a seed is a reproducibility guarantee rather than a hope. Every experiment writes a CSV artifact; the tables below are transcriptions of those artifacts.

**Notation.** The skew parameter is the **Zipf rank exponent *s***, where the probability of the rank-*r* key is proportional to *r*⁻ˢ, so higher *s* means more skew. It is not the Pareto shape parameter, for which higher means a *thinner* tail and *less* skew. The two are routinely conflated in this literature, and a phase diagram whose axis can be read backwards is worse than no phase diagram; earlier drafts of this thesis used a bare "α" and were ambiguous in exactly this way.

## 9.2 Correctness Results (Measured)

### 9.2.1 Reconstruction equivalence, conservation, and the absence discipline

The core correctness experiment runs 10,000 balanced transfers over a closed book of 40 accounts with a deliberately punishing memory budget of 8 resident entries, so that eviction and reconstruction are exercised continuously rather than incidentally. Interleaved with the transfers are reads (each of which may trigger an upquery), forced evictions, and idempotent replays of previously used keys. After every read the value served by the partial view is compared against an independent fold at the anchor the answer carries.

Two things about that comparison were wrong in the previous revision and are worth stating, because they decide how much the zero in the divergence column is worth. The "independent fold" was the ledger's own `reconstruct_balance` — the same function the view had just called on a miss — so most of the reported comparisons were a value against itself. And every read was taken at the head, so Theorem 4.1's quantification over *every* anchor was tested at one. The oracle is now `crates/conservation-suite`, which shares no code with the engine and is written under Appendix F's rule that it is never optimised, and anchors are drawn from the whole retained history: about 3,330 of the roughly 3,340 reads per seed are at an anchor below the head. The result is unchanged and the evidence for it is not.

<!-- BEGIN:E1-correctness results/E1-correctness.md#table -->

*Generated from `results/E1-correctness.md`. Do not edit by hand.*

| Seed | Transfers | Upqueries | Evictions | Historical-anchor reads | Divergences | Conservation | Chain | Rebuild mismatches | Miss != 0 | Idempotent rejects |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 10000 | 3018 | 2712 | 3332 | **0** | OK | OK | **0** | OK | 103 |
| 7 | 10000 | 3023 | 2698 | 3333 | **0** | OK | OK | **0** | OK | 103 |
| 42 | 10000 | 3007 | 2686 | 3328 | **0** | OK | OK | **0** | OK | 103 |
| 100 | 10000 | 2990 | 2662 | 3331 | **0** | OK | OK | **0** | OK | 103 |
| 2024 | 10000 | 3017 | 2703 | 3332 | **0** | OK | OK | **0** | OK | 103 |

<!-- END:E1-correctness -->

*Table 9.1 — Correctness under adversarial interleaving of commit, read, evict, upquery and
replay. Every check passed on every seed.*

Each column corresponds to a named guarantee. **Divergences = 0** is the executable form of the reconstruction theorem (C1): across roughly 13,600 reconstructions, no reconstructed value ever differed from the independent fold at the same anchor. **Conservation = OK** is the per-currency system total remaining exactly zero at the end of every run, which is the conservation corollary under continuous eviction and refill. **Rebuild mismatches = 0** is the reconstruction-equivalence property of H-F4: the entire derived layer was wiped and rebuilt from the retained base alone, and all 40 balances matched their pre-wipe values exactly. **Miss ≠ 0 = OK** is the absence-lattice discipline: after a total wipe, reading a funded account returned its correct non-zero balance *via reconstruction*, not a silent zero from an empty slot. **Idempotent rejects = 103** confirms that every replayed key was refused rather than double-posted.

Separately, mutating a single committed posting broke chain verification, as required: tamper-evidence is detection relative to a retained digest, and the digest detected it.

### 9.2.2 What this does and does not establish

It establishes that the guarantees are mutually consistent and executable, and that the mechanisms implement them under adversarial schedules. It does **not** establish strict serializability, and the distinction is not pedantic: a published Jepsen analysis of a production distributed SQL system found that its bank tests "passed consistently" while a *causal reverse* anomaly demonstrated the system was serializable but **not** strictly serializable. A conservation-of-money suite is therefore strictly weaker than the top rung of the consistency ladder, and this thesis must not present one as evidence for the other.

The methodological consequence, adopted in §9.6, is that the correctness programme needs a second instrument: black-box anomaly inference over recorded histories in the style of Elle, which infers an Adya-style dependency graph and detects cycles, and is *sound* — an anomaly it reports is present in every interpretation of the observation. There is a pleasing structural fit here. Elle's power depends on *traceability*: a data model from which the complete version history can be recovered by reading, for which append-only list structures are the canonical example. An append-only, hash-chained ledger is precisely such a model. **The architecture this thesis proposes is unusually amenable to black-box verification, and that is an argument for it that neither the earlier draft nor the prior literature makes.**

### 9.2.3 Stream–relation duality (measured)

The duality of H-F2 was checked executably as well as proved. For each of five seeds, a 200-epoch changelog of signed Z-set deltas over 50 keys was generated; the state sequence was obtained by integration, then differentiated back to a changelog, then re-integrated, and the two state sequences were compared as canonical Z-sets — equal supports, equal weights — at *every* epoch.

**1,000 epoch-by-epoch comparisons; 0 mismatches.**

The deltas include negative weights, so the check covers retraction and not merely accumulation. This is corroboration of a theorem, not a substitute for it: the proof is in §3.8, and the operator identities it rests on are machine-checked in the DBSP development.

## 9.3 The Phase Diagram (Measured) — and the Hypothesis It Refutes

### 9.3.1 The cost model, and why it is swept rather than fixed

Comparing materialization strategies requires pricing three different things: memory held over time, maintenance work, and reconstruction work. The prototype counts each directly:

* **resident-entry-epochs** — the integral of residency over time, not peak residency, because charging peak for the whole run would overstate the cost of a strategy whose footprint grows gradually, which is exactly what full materialization does;
* **deltas applied** — per-key maintenance actually performed;
* **base rows read** — reconstruction work actually performed.

Total cost is a weighted sum. Maintenance and reconstruction are fixed at 1 unit each, and **the memory price is swept**, expressed in units of "one base-row read, per resident entry, per epoch." Sweeping it rather than fixing it is the difference between a result and an artifact: at a memory price of zero the answer is trivially "materialize everything," and any single chosen weight would smuggle the conclusion into the premise. A reader can locate their own hardware on this axis by asking what one resident entry-epoch costs them relative to one base-row read.

An earlier version of this experiment charged peak residency times total epochs. That made the memory term dominate everything, and produced a "phase diagram" in which partial materialization won in every cell. It was discarded as an artifact of the weighting, and the correction is recorded here because the failure mode — a cost model that encodes its own conclusion — is easy to commit and hard to spot in a table of ratios.

### 9.3.2 Cost components

Workload: 10,000 accounts, 30,000 operations, 90% reads, budget = 5% of the key space, LRU eviction; medians over five seeds.

| Zipf *s* | Resident-entry-epochs (partial / full) | Base rows read (partial / full) | Deltas applied (partial / full) |
|---|---|---|---|
| 0.5 | 1,484,276 / 17,559,969 | 36,690 / 10,025 | 593 / 4,040 |
| 0.7 | 1,483,534 / 15,200,887 | 36,335 / 8,838 | 1,335 / 4,237 |
| 0.9 | 1,479,730 / 11,560,348 | 26,767 / 6,813 | 2,701 / 4,613 |
| 1.1 | 1,467,457 / 7,281,624 | 12,987 / 4,376 | 4,213 / 5,091 |
| 1.3 | 1,430,876 / 3,885,194 | 4,480 / 2,325 | 5,265 / 5,531 |

*Table 9.2 — The three cost components, measured. Partial materialization always holds less state and applies fewer deltas; it always reads more base rows.*

### 9.3.3 The measured phase diagram

Each cell is the median over five seeds of cost(partial) / cost(full) on identical workloads. Values below 1.00 favour partial materialization.

**Memory price = 0** (memory free)

| budget \ *s* | 0.5 | 0.7 | 0.9 | 1.1 | 1.3 |
|---|---|---|---|---|---|
| 1% | 3.11 | 5.33 | 7.47 | 6.36 | 3.67 |
| 2% | 2.97 | 4.05 | 4.49 | 3.38 | 1.96 |
| 5% | 2.65 | 2.88 | 2.57 | 1.81 | 1.25 |
| 10% | 2.32 | 2.26 | 1.83 | 1.33 | 1.06 |
| 25% | 1.80 | 1.56 | 1.25 | 1.04 | 1.00 |
| 50% | 1.30 | 1.14 | 1.02 | 1.00 | 1.00 |

**Memory price = 0.002**

| budget \ *s* | 0.5 | 0.7 | 0.9 | 1.1 | 1.3 |
|---|---|---|---|---|---|
| 1% | **0.90** | 1.61 | 2.49 | 2.54 | 1.86 |
| 2% | **0.87** | 1.24 | 1.49 | 1.36 | 1.05 |
| 5% | **0.82** | **0.93** | **0.93** | **0.83** | **0.81** |
| 10% | **0.79** | **0.81** | **0.76** | **0.75** | **0.86** |
| 25% | **0.80** | **0.79** | **0.80** | **0.91** | 1.00 |
| 50% | **0.90** | **0.91** | **0.97** | 1.00 | 1.00 |

**Memory price = 0.01**

| budget \ *s* | 0.5 | 0.7 | 0.9 | 1.1 | 1.3 |
|---|---|---|---|---|---|
| 1% | **0.25** | **0.44** | **0.69** | **0.77** | **0.67** |
| 2% | **0.25** | **0.36** | **0.44** | **0.45** | **0.45** |
| 5% | **0.27** | **0.32** | **0.35** | **0.38** | **0.51** |
| 10% | **0.33** | **0.36** | **0.39** | **0.50** | **0.74** |
| 25% | **0.51** | **0.55** | **0.64** | **0.85** | 1.00 |
| 50% | **0.78** | **0.84** | **0.96** | 1.00 | 1.00 |

*Table 9.3 — The measured phase diagram at three memory prices. Full tables, including prices 0.0005 and 0.05, are in `results/e4.log`.*

### 9.3.4 Three findings, one of which refutes a stated hypothesis

**Finding 1 — There is a real boundary, and it is set by the memory price, not by skew.** At memory price 0, partial materialization loses in every cell, by up to 7.5×. At 0.01 it wins in every cell but the saturated ones. The crossover sits between roughly 0.0005 and 0.002 in these units. That is the frontier the thesis predicts, located empirically for the first time.

```text
PROPOSED — MISMATCH-F-08. Not applied to the sentence above.

"That is the frontier the thesis predicts" claims more than the theorem supports, and H-S2
claims more still: it says the crossover falls "within the band predicted by the
Eviction–Consistency Frontier Theorem". Theorem 4.2(ii) has a Θ(Z) term and states no
constant for it, so **no band is derivable** — the theorem asserts that a threshold exists
for every (c_u, w, λ, Z), which is an existence claim, and an existence claim cannot be
agreed or disagreed with by a located crossover.

THE REPLACEMENT WORDING

  §9.12, Finding 1, final sentence:
    "A crossover in the memory price exists and was located, between roughly 0.0005 and
     0.002 in these units. Theorem 4.2(ii) predicts that such a crossover exists; it does
     not predict where, because the constant inside its Θ(Z) term is unstated. The located
     value is therefore a measurement the theorem is consistent with, not a confirmation of
     a predicted band."

  §1.6 H-S2, the claim:
    "As the demanded consistency level rises, and/or skew flattens, and/or the delayed-hit
     ratio Z grows, the measured cost of partial materialization crosses that of full
     materialization. The Eviction–Consistency Frontier Theorem predicts that the crossover
     exists; its band is not derivable from the theorem as stated, so this hypothesis is
     confirmed by locating a crossover and refuted by finding none."

  §1.6 H-S2, Status:
    "measured (§9.3, §9.13.2): a crossover exists and was located. The theorem's band was
     not derived and is not tested. results/E16-band.md, written and committed before the
     durable run, registers the one side of it that *is* derivable — a lower bound on miss*
     with Θ(Z) set to zero — and states which constant is missing and why."

  §11.3, the C2 row:
    C2 drops from a predictive claim to an existence claim. A measurement that located no
    crossover anywhere in the swept price range would refute it; a measurement that located
    one at an unexpected price would not.

WHAT IS NOW SATISFIED

  §5.5's rule — "a crossover without Z is not interpretable" — was not satisfied when this
  finding was written: neither e4_phase.csv nor e12_phase_compiled.csv had a Z column. Both
  do now (T-15), defined identically in the two writers as base rows per reconstruction over
  deltas per read, the counted-work analogue of the delayed-hit factor. E4 reports Z from 5.3
  to 307 across the grid, so the crossover above is interpretable in a way it was not.

  P3 of results/E16-band.md is the interesting outcome: a measurement showing partial
  materialization losing below the registered threshold would not contradict Theorem 4.2 —
  it would locate the missing constant, and §4.3 would gain it as a measured quantity rather
  than an asymptotic one.
```

**Finding 2 — The optimal budget is interior, not extremal.** At memory price 0.002 and *s* = 0.5 the ratio runs 0.90 (50% budget) → 0.80 (25%) → **0.79 (10%)** → 0.82 (5%) → 0.87 (2%) → 0.90 (1%). Too large a budget wastes memory; too small a budget thrashes, and reconstruction cost explodes faster than memory savings accrue. The existence of an interior optimum is what makes an adaptive materialization optimizer a necessity rather than an ornament — a fixed policy at either extreme is measurably wrong.

**Finding 3 — the refutation.** Hypothesis H-S1, as stated in Chapter 1 of the previous draft, held that *"partiality pays on skew"* and that the advantage *grows* with skew. **The measurements contradict this.** Table 9.2 shows why: as *s* rises, full materialization touches fewer distinct keys, so its own footprint shrinks — resident-entry-epochs for full fall from 17.6 M at *s* = 0.5 to 3.9 M at *s* = 1.3, while partial's stay near 1.45 M. The *ratio* therefore gets **worse** for partial as skew increases, from 12:1 down to 2.7:1 (Table 9.2), and the corresponding memory-ratio measurement in §9.3.5 shows the same monotone deterioration. Skew simultaneously reduces partial's reconstruction penalty (36,690 → 4,480 rows read), so the two effects oppose one another and the net result depends on the memory price — which is precisely why the diagram must be swept over price rather than plotted at one.

The corrected hypothesis, which the data support, is: **partial materialization pays when memory is expensive relative to reconstruction, at an interior budget, and skew determines the *shape* of the trade rather than its direction.** Chapter 1's H-S1 is restated accordingly, and the earlier phrasing is retained in Appendix J with the reason it was wrong, because a hypothesis quietly edited after the fact is not a hypothesis.

### 9.3.5 Resident state across skew

Independently measured, with a 10% budget over 20,000 accounts and 60,000 operations:

| Zipf *s* | resident(partial) / resident(full), median [range] | Hit rate |
|---|---|---|
| 0.5 | 0.116 [0.116, 0.116] | 0.179 |
| 0.7 | 0.130 [0.130, 0.131] | 0.332 |
| 0.9 | 0.167 [0.167, 0.167] | 0.567 |
| 1.0 | 0.206 [0.203, 0.207] | 0.690 |
| 1.1 | 0.268 [0.264, 0.269] | 0.795 |
| 1.3 | 0.533 [0.525, 0.540] | 0.922 |

*Table 9.4 — Memory advantage and hit rate move in opposite directions as skew rises.*

The seed-to-seed range is negligible, so the trend is not noise. The two columns moving in opposite directions is the whole story of this chapter in miniature: the thing skew helps (hit rate, hence reconstruction cost) and the thing skew hurts (relative memory advantage) are different things, and a design decision needs both.

## 9.4 Cost, Policy, and Mechanism Results (Measured)

### 9.4.1 Is the cost of a read history-shaped or workload-shaped? — a falsification, a refinement, and a fix

This is the thesis's cost-law claim (C3): *the marginal price is workload-shaped, not history-shaped*. It was tested in three stages, and the first two refuted it.

**Stage 1 (E5) — fixed key space.** With 2,000 accounts held fixed and total writes swept, base rows read per reconstruction were:

| Writes | Ledger epochs | Indexed reconstruction | Unindexed scan |
|---|---|---|---|
| 5,000 | 5,004 | 124.5 | 14,000 |
| 20,000 | 20,004 | 499.3 | 44,000 |
| 80,000 | 80,004 | 2,009.8 | 164,000 |
| 320,000 | 320,004 | 8,008.9 | 644,000 |

*Table 9.5 — Reconstruction cost grows linearly with history when the key space is fixed. The anchor index gives an 80–112× reduction over a full scan, but does not change the growth.*

Cost grew 64× as history grew 64×. **The claim is false in this form.** The anchor index — the structure the earlier draft credited with making cost history-independent — buys two orders of magnitude in constant factor and *nothing* in asymptotics.

**Stage 2 (E9) — key space grown in proportion.** The obvious defence is that a real institution opens accounts as it grows, so per-key update counts stay bounded. Tested by growing accounts with writes at a fixed ratio:

| Writes | Accounts | Rows read per reconstruction | Per-key updates |
|---|---|---|---|
| 5,000 | 500 | 210.4 | 210.4 |
| 20,000 | 2,000 | 499.3 | 499.3 |
| 80,000 | 8,000 | 1,245.9 | 1,245.9 |
| 320,000 | 32,000 | 3,303.7 | 3,303.7 |

*Table 9.6 — The defence fails. Cost still grows ~16× as history grows 64×.*

The reason is instructive and is a property of skew itself: under a Zipf distribution the *hottest* keys retain a roughly constant share of traffic, so their absolute update count grows with total traffic no matter how many new cold keys are added. Reconstruction reads are drawn from the same distribution, so they preferentially hit exactly those keys. Growing the key space dilutes the tail, not the head.

**Stage 3 (E10) — the fix, and its confirmation.** The mechanism the design was missing is a **per-key checkpoint**: every *C* postings on a key, record (epoch, running balance), so a reconstruction folds only the suffix since the newest checkpoint at or before the anchor. A checkpoint is derived state — recomputable from the base — so it costs nothing in immutability, retention or auditability.

| Writes | No checkpoint | C = 256 | C = 64 | C = 16 |
|---|---|---|---|---|
| 5,000 | 124.5 | 45.9 | 19.9 | 6.8 |
| 20,000 | 499.3 | 78.4 | 26.1 | 8.3 |
| 80,000 | 2,009.8 | 102.1 | 30.8 | 8.0 |
| 320,000 | 8,008.9 | 123.2 | 32.0 | **8.5** |

*Table 9.7 — Per-key checkpoints restore bounded reconstruction cost. At C = 16, cost is flat across a 64× increase in history and sits at ≈ C/2 + 1, exactly as theory predicts.*

At C = 16 the cost rises from 6.8 to 8.5 — a factor of 1.25 — while history grows 64×. The predicted value is the mean distance back to the previous checkpoint plus one checkpoint read, i.e. C/2 + 1 = 9, and the measurement lands just under it. At C = 64 the cost is 32.0 against a predicted 33. **The cost law holds, but only with checkpointing, and its constant is C/2 + 1 — a number the designer chooses.**

This is the most consequential empirical result in the thesis, because it converts an asserted asymptotic property into a *design obligation*. It also explains, retrospectively, why production ledgers maintain running balances rather than folding from the journal on demand, and why one purpose-built ledger offers a per-account flag to retain balance history at all: they are paying for the same bound by a different name. The formal statement is added to Chapter 3 as the **Bounded Reconstruction Theorem** and to Chapter 4 as contribution **SC7**, and the honest reading of the earlier draft is that it claimed the conclusion without the mechanism that makes it true.

### 9.4.2 Eviction policy under reconstruction latency

Three policies compared on identical workloads (10,000 accounts, *s* = 0.9, budget 5%, 40,000 operations), including the randomized policy that the original partial-state system used, so the comparison has a real baseline and not merely the system against itself. `service_time` is the modelled reconstruction service time; the aggregate-delay column charges each reconstruction for the requests that queue behind it.

<!-- BEGIN:table-9.8 results/e6_policies.csv#policytable -->

*Generated from `results/e6_policies.csv`. Do not edit by hand.*

| service_time | policy | misses (median) | base rows read (median) | aggregate delay (median) |
|---|---|---|---|---|
| 0 | random | 21,149 | 88,764 | — |
| 0 | LRU | 19,583 | 41,002 | — |
| 0 | cost-aware | 18,531 | 29,272 | — |
| 1 | random | 21,149 | 88,764 | 211,490 |
| 1 | LRU | 19,583 | 41,002 | 195,830 |
| 1 | cost-aware | 19,004 | 33,806 | 190,040 |
| 4 | random | 21,149 | 88,764 | 3,130,052 |
| 4 | LRU | 19,583 | 41,002 | 2,898,284 |
| 4 | cost-aware | 19,041 | 34,823 | 2,818,068 |

<!-- END:table-9.8 -->

*Table 9.8 — Medians over five seeds, generated from `results/e6_policies.csv`.*

Three observations, including one that qualifies the thesis's own claim. First, the gap between randomized eviction and LRU is large — 2.2× in base rows read — which is a measured argument that replacing randomized eviction is worth doing at all. Second, the cost-aware policy improves on LRU substantially on reconstruction work, and on misses. Third, and against expectation, its advantage on *aggregate delay* is modest. Both figures are generated from the file rather than typed beside it, because six cells of the table above had drifted from the run they named and the two comparisons in this sentence had drifted with them — 31% and 4.1% against a file that says:

<!-- BEGIN:policy-deltas results/e6_policies.csv#policydeltas -->

*Generated from `results/e6_policies.csv`. Do not edit by hand.*

Cost-aware against LRU, from `results/e6_policies.csv`: **28.6% fewer base rows** at service_time 0 (41,002 → 29,272), and **2.8% less aggregate delay** at service_time 4 (2,898,284 → 2,818,068).

<!-- END:policy-deltas -->

The delayed-hit weighting changes which entries it keeps, and that trade costs it some of its base-row advantage. The honest conclusion is that cost-awareness is clearly worth it for reconstruction work and only marginally so for latency in this configuration, and the thesis should not claim more.

### 9.4.3 What each consistency rung costs — a measurement that was measuring its own defect

Identical workload; the rung sets both the anchor a read demands and how far maintenance
may be batched.

<!-- BEGIN:E8-rungs results/E8-rungs.md#table -->

*Generated from `results/E8-rungs.md`. Do not edit by hand.*

| rung | deltas applied | maintenance passes | misses | base rows read | divergences |
|---|---|---|---|---|---|
| bounded(k=64) | 2171 | 61 | 26644 | 102624 | 0 |
| bounded(k=8) | 2514 | 446 | 24700 | 73811 | 0 |
| strict(k=0) | 3621 | 4017 | 19658 | 40869 | 0 |

<!-- END:E8-rungs -->

*Table 9.9 — Medians over five seeds. `divergences` compares every served value against an
independent fold at the anchor it was served with.*

**The previous version of this table was an artefact, and the artefact was the finding.**
It reported deltas applied of 55, 408 and 3,621 — a **66×** spread — with misses varying
by 0.3% and hit rate by 0.001 across rungs, and concluded that the price of freshness "is
not paid on the read path at all". Three of those statements do not survive a correct
instrument.

The mechanism was this. A bounded rung batches maintenance, and the harness implemented
batching by applying only the epoch at the stride boundary and then certifying the view
through it. The *k−1* epochs in between were never folded into any resident entry. The
66× was therefore not a saving; it was the count of deltas thrown away, and the entries
the view then served were wrong rather than stale. Because they were nevertheless marked
current, they registered as hits — which is why the read columns looked identical.

Corrected, so that a pass folds every epoch in its window:

* **The ratio in deltas applied is 1.67×, not 66×.** A bounded rung folds essentially the
  same deltas as a strict one, because the epochs it batched over still have to be applied.
* **What remains ~66× is maintenance *passes*** (61 against 4,017). That is a real saving
  and a materially weaker claim: the same work in fewer, larger passes, which amortizes
  per-pass overhead and lets an operator schedule maintenance rather than run it
  continuously.
* **The read path is where the cost moved, not where it is absent.** A lax rung now reads
  **2.5× more base rows** (102,624 against 40,869) and misses 26,644 times against 19,658;
  its hit rate is 0.259, not the 0.455 previously reported. Its entries are genuinely less
  current, so more reads reconstruct.

The honest statement, which is less convenient than the one it replaces: **a bounded rung
buys fewer maintenance passes and pays for them in reconstruction.** The trade is visible
on both sides rather than free on one, and an operator provisioning for it must size the
reconstruction path as well as the maintenance path.

The first attempt at this experiment measured only misses and hit rate and found no
difference at all between rungs. That null was reported, investigated, and attributed to
instrumenting the wrong path. The attribution was itself wrong: the null was real, and it
was the *false certification* that made the read columns identical. Adding maintenance
counters found a difference in the right place for the wrong reason, and the sequence —
null, misattributed diagnosis, confident table, refutation — is recorded because it is a
more useful cautionary tale than the one this section used to tell. Appendix J.16 keeps
the refuted figures.

### 9.4.4 Write path: what the mechanisms cost (wall-clock, heavily caveated)

The only wall-clock measurement in this chapter. **In-memory, single-threaded, no durability, no consensus.** It measures what the commit rule and the hash chain cost, and nothing else. It is not a throughput claim and cannot be compared with any published database figure, all of which include durability.

| Configuration | Hot-account share | Postings/sec (median) |
|---|---|---|
| chained | 0.0 | 1,526,384 |
| chained | 0.5 | 1,739,657 |
| chained | 0.9 | 1,686,866 |
| unchained | 0.0 | 2,243,577 |
| unchained | 0.5 | 2,107,468 |
| unchained | 0.9 | 2,222,726 |

*Table 9.10 — Mechanism cost only. Hash chaining costs roughly 30% of admission throughput on this platform.*

Two readings. The useful one: **hash chaining costs about 30%** of the in-memory admission path — a real number for a real mechanism, and one an implementer can weigh. The important one: **hot-account share has no effect, and that is a limitation of the instrument, not a finding about the design.** Contention is a concurrency phenomenon and this prototype is single-threaded, so the flat column measures the absence of an experiment rather than the absence of a problem. The contention question is genuinely open here and is answered in the literature rather than by this prototype: two independent production write-ups report that the shared settlement account is a structural hot key in double-entry systems, one reporting a per-account ceiling of 3–4 update operations per second raised to about 30 by 250 ms batching windows, the other routing hot-account entries to an asynchronous path with a 60-second bound rather than sharding the account. Those are the numbers a reader should weigh; §9.9 states the concurrency experiment that would let this thesis contribute its own.

## 9.5 Evaluation Design for the Unmeasured Parts

Everything below is protocol and prediction. No cell is filled.

### 9.5.1 Baselines and the question each answers

| Baseline | Question it answers | Status |
|---|---|---|
| PostgreSQL / MySQL with trigger-maintained balances | Does the aligned design beat the industry-default shape? | *to be measured* |
| Total-materialization IVM engine (Materialize/Feldera class) | Isolates **partiality** as the single variable | *to be measured* |
| Nilestream with anchoring disabled | Prices this thesis's own machinery | *to be measured* |
| Purpose-built ledger | Is the write path competitive as a write path? | *to be measured* |
| Full materialization on identical workload | Read-side comparison | **measured, §9.3** |
| Randomized eviction | Is replacing it worth it? | **measured, §9.4.2** |

Two rules are fixed in advance. Vendor performance claims without published methodology, hardware and workload definition are excluded as baselines — including a widely-cited 150,080-transactions-per-second core-banking figure that discloses neither transaction definition, isolation level, nor durability setting. And any TPC-derived workload run here is unaudited by definition, so it will be reported as "TPC-C-derived" and never as a `tpmC` result, in line with the TPC's own comparability rules.

### 9.5.2 Predicted directions, with sources

* Ledger write path with durability: throughput within a small constant factor of a purpose-built baseline. *Source:* the write path performs the same work plus hash chaining, whose cost is now measured at ≈30% (§9.4.4). *Refuted by:* a gap larger than one order of magnitude.
* Reconstruction latency under load stays inside an authorization budget in the winning region of the phase diagram. *Source:* Table 9.7's bounded per-reconstruction work under checkpointing.
* Bounded-staleness views cost ~1/k in maintenance **passes**, and pay for it in reconstruction. *Source:* Table 9.9 as corrected in §9.4.3; the earlier "~1/k in deltas applied" was an instrument artefact and is retained in Appendix J.16. To be confirmed with concurrency and real coordination.

## 9.6 Correctness Evaluation Beyond Conservation

The conservation suite (§9.2) is necessary and insufficient (§9.2.2). The full programme adds: black-box anomaly inference over recorded histories, checking for the Adya phenomena G0, G1a/b/c, G-single and G2 including real-time and per-process edges; adversarial fault campaigns (crash at each protocol step class, eviction storms, duplicate and reordered delivery, recovery mid-upquery); and a mutant corpus of programs that must be *rejected* at compile time. Acceptance is zero surviving violations, with any violation diagnosed as either an implementation defect or a model gap and reported as such.

A model-checking strand is included with a stated precedent and a stated limit. Formal specification of a consistency ladder has found real defects in a shipped system: a TLA+ specification of a commercial database's five advertised consistency levels revealed that two of them were indistinguishable to a client performing individual reads and writes, and that under strongly-consistent writes the bounded-staleness rung was subject to no bound at all. That is a directly analogous precedent and a cautionary one: **a ladder can be advertised with five rungs and have four.** The limit is equally clear from the same literature: model checking establishes design-level safety, not that the implementation refines the design.

## 9.7 Generality and SQL Coverage — a Claim That Had to Be Restructured

The previous draft proposed to establish SQL-completeness partly by "passing an industry SQL-conformance suite." **That argument is not available, and the reason is a finding of this revision.**

The only officially-blessed SQL conformance suite ever produced targets SQL-92, was frozen in December 1996, and the validation programme that certified against it was terminated on 1 July 1997. NIST's own user guide states that it "would be incorrect for implementations to claim conformance … simply by virtue of correct performance of these tests." The widely-used modern alternative is a *differential* tester, comparing engines against one another rather than against the standard, and explicitly excludes transactional behaviour and concurrency from its scope. No authoritative, current, complete SQL:2016/2023 conformance corpus exists, and the standard's licensing obstructs building an open one.

The claim is therefore restructured into three parts, of which the primary one is a proof obligation rather than a test:

1. **Translation completeness (primary, a theorem).** A total, semantics-preserving compilation of a *stated* SQL fragment into the typed IR, proved by structural induction and gated by golden-file α-equivalence tests. This is stronger than any conformance run, because it quantifies over all programs in the fragment rather than over a corpus.
2. **Feature coverage (corroboration).** A per-feature evidence table against the normatively enumerated mandatory feature list in the standard's own Annex F.
3. **Differential equivalence (corroboration).** Agreement with established engines on a published corpus, reported as interoperability evidence and not as conformance.

Anything outside the stated fragment fails loudly with a named error rather than being approximated — the operational counterpart of fragment honesty.

## 9.8 Threats to Validity

**Construct.** The prototype implements the thesis's mechanisms but is not the system; its view shapes are simple, and a join-heavy view family could behave differently. The workload is synthetic; real banking traces are unobtainable for legal reasons. The skew premise is contested in the literature — production measurements range from *s* ≈ 0.55 to ≈ 2.5 across different systems — which is why §9.3 sweeps it rather than assuming it.

**Internal.** Counted-work units eliminate machine variance for the primary results but not modelling error in the cost weights; the sweep addresses this and the raw components are published so a reader can re-price everything. The cost-aware policy's parameters were developed on the same workload family they were evaluated on, which risks over-fitting; a held-out workload is required before claiming generality for it. The delayed-hit model is an analytic charge, not a measured queue, and is labelled as such.

**External.** One platform, one prototype, single-threaded, in-memory. The write-path numbers do not generalize past the mechanism they measure. The contention result is a non-result (§9.4.4).

**Theory-fidelity.** The prototype is not a proof; where it corroborates a theorem (§9.2.1, §9.2.3) it corroborates the theorem's *statement*, not its proof. Where it refutes a claim (§9.3.4, §9.4.1) the refutation is about the claim as operationalized, and §9.4.1 shows how much depends on that operationalization.

**Attribution.** Every literature figure in this thesis carries its original context. The partial-state memory figures from prior work describe one application at one scale under one latency target; the cache-skew figures describe those systems' workloads, not banking; the production ledger throughput figures describe those platforms' configurations. None is evidence about Nilestream.

## 9.9 Experiments the Prototype Cannot Run

Stated explicitly, because the gaps are as informative as the results.

* **Contention.** Needs concurrency, a real commit protocol and a lock or OCC discipline. The literature's quantitative anchor for what to expect is a measured ≈6.6× throughput collapse in an in-memory OLTP system as Zipf θ rises from 0.4 to 0.99 on a write-heavy workload — alongside the striking converse that on a read-heavy workload the same skew *increases* throughput through cache locality. That read/write asymmetry is precisely the shape this thesis's phase diagram should exhibit under concurrency, and testing it is the single highest-value next experiment.
* **Durability.** Needs a real write-ahead path. The honest framing is already fixed by §11.1: a single node with fsync-before-ack has RPO = 0 against process and OS crash and unbounded RPO against loss of the node or site.
* **Distribution.** Needs the cross-shard protocol of §8.6.
* **Language claims.** Need the compiler.

## 9.10 Immutability and Memory Safety

One of the three layers of §5.8 is done, one is half done, and one is not started, and the honest summary is that the strongest thing this thesis can say about data races is a compiler's.

**Discharged, and tested.** The workspace contains **no `unsafe` block**, and sealed epochs are reached only through shared references. A drift test asserts the first at every build, in both repositories, because "no unsafe" is the kind of claim that is true until one line makes it false.

**Half done.** Crash-recovery drills run (§9.14 and GBS's G3), and verify that a reopened ledger's read model agrees both with the live process and with a fold of the recovered journal.

**Not started.** Exhaustive-interleaving model checking of the two mutable structures, and sanitizer runs over a concurrency suite. There is no `loom` or `shuttle` dependency in either workspace and no sanitizer job in the gate.

The residual surface is also smaller than "two mutable structures" implies, and for a reason that is a limitation rather than an achievement: the daemon serves every query under a single `Arc<Mutex<RevEngine>>` (§9.14.1), so the engine is concurrent in its connections and serial in its work. There is at present one lock to reason about, and the claim that the surface is small is not yet a claim that it has been checked. Acceptance for the full system remains zero races at all three layers.

## 9.11 Language Scope

Constructive tests: the banking portfolio implemented in the domain library with the kernel change log audited; a non-financial conserved-quantity domain; representative workloads per class; and the negative corpus of programs that must be rejected. The negative corpus is the mutant suite (§9.14.2). Two of the others are now measured and are reported here; the rest is *to be measured*.

### 9.11.1 The non-financial domain (H-S8's falsifier)

`examples/inventory.niles`: stock movements between warehouses, where units of a SKU are conserved exactly as money is and nothing is money. Two SKUs at scale 0, a movement ledger with `conserve per (txn, sku)`, a keyed-sum view of stock on hand, and two movement functions. Run by `crates/conservation-suite/tests/inventory.rs`.

**The result is a qualified pass, and the qualification is the interesting half.**

*What passed.* The domain checks; both movement functions are **proved** to conserve by the currency-row solver, not discharged to the seal; `nilesc run` posts two cancelling legs carrying `WIDGET` as their grade; a movement of two unrelated quantities is `Undecided` and the seal refuses it, naming the grade and the residual exactly as it does for money. **Nothing in the compiler, the IR or the runtime was changed**, and a test asserts that: `grep`ping `niles-lang`, `niles-ir` and `nilestream-core` for `inventory`, `widget`, `sprocket`, `warehouse` and `sku` returns nothing. The machinery §6.6 calls general — graded rows, linear halves, effect rows, declared commit rules — is general.

*What did not.* Every quantity in that file arrives as a *parameter*, because it has to. A literal in this domain does not lex:

```rust
let looks_like_currency = word.len() == 3
    && word.bytes().all(|b| b.is_ascii_lowercase())
    && keywords::lookup(word).is_none();
```

A grade's name must be exactly three lowercase letters — ISO 4217's shape, in the lexer. `15000 jpy` lexes; `5 widget` does not. So a non-financial grade can be named in a type, a schema, an effect row and an argument, and not in a literal.

*What that measures.* H-S8's refutation condition (§11.3) is a domain that cannot be expressed without a kernel change. This domain is expressible, and the kernel is untouched — so the hypothesis is **not** refuted. But the honest status is `partly measured` rather than `measured`, because the notation carries a banking assumption the machinery does not, and one plausible reading of "one language serves every workload class" is that a reader can write down a quantity. The one-line widening is recorded as `MISMATCH-T-22-currency-literal` in the example itself and deliberately **not applied**: changing the language to make a hypothesis pass is not a measurement of the language.

### 9.11.2 What the checker discharges (H-S6, static half)

`crates/niles-lang/tests/corpus_obligations.rs` runs `nilesc report-obligations` over every Niles program in both repositories and writes `results/obligations.csv`. Over four files — the GBS schema, the two examples and the inventory domain — **16 of 16 conservation obligations are proved statically and none is discharged to the runtime seal.**

Two cautions belong beside that number. It is a small corpus, and it is a corpus of programs written *by* people who knew what the solver proves, which is exactly the selection effect §11.5 raises about a checker whose `Undecided` verdict could make it true and useless. And it measures the wrong half of H-S6 for anyone asking about cost: the hypothesis says static checking subsumes runtime policing "at no measurable runtime cost", and no measurement of runtime cost exists — that needs a trigger-based SQL baseline and a ported corpus, neither of which is built. `status.toml` records H-S6 as `partly measured` for that reason.

## 9.12 Results Analysis by Conjecture

| Claim | Status after this chapter |
|---|---|
| C1 reconstruction equivalence | **Corroborated** — 0 divergences against an *independent* oracle at ~3,330 historical anchors per seed, 0 rebuild mismatches, 5 seeds (§9.2.1) |
| Conservation under eviction/refill | **Corroborated** — per-currency total exactly 0, 5 seeds (§9.2.1) |
| H-F2 stream–relation duality | **Corroborated** — 1,000 epochs, 0 mismatches (§9.2.3) |
| Absence discipline (miss ≠ 0) | **Corroborated** (§9.2.1) |
| C2 frontier exists | **Corroborated and located** — crossover between memory prices 0.0005 and 0.002 (§9.3.3) |
| H-S1 "partiality pays on skew" | **Refuted as stated**; restated as a memory-price condition with an interior optimum (§9.3.4) |
| C3 cost is workload- not history-shaped | **Refuted as stated; restored under checkpointing** with constant C/2 + 1 (§9.4.1) |
| Consistency rung cost | **Refuted as stated; re-measured.** The reported 66× in deltas applied was the count of deltas a defective batching loop discarded. Corrected: ~66× in maintenance *passes*, 1.67× in deltas, and **2.5× more base rows read** on the lax rung — the tax is not absent from the read path (§9.4.3, Appendix J.16) |
| Cost-aware eviction beats LRU | **Partly corroborated** — 28.6% on reconstruction work, 2.8% on aggregate delay (§9.4.2, generated from `results/e6_policies.csv`) |
| Hot-account contention | **Not measured**; instrument cannot (§9.4.4, §9.9) |
| ℓ₄ (serializable) price | **Not measured.** ℓ₄ coincides with ℓ₃ on read-only traces (§3.8), and every workload in this chapter is read-only at the read path, so its O(contended keys) term is never exercised |
| Strict serializability | **Not tested**; conservation is strictly weaker (§9.2.2) |
| SQL-completeness by conformance suite | **Withdrawn**; no such suite exists (§9.7) |
| Everything in §9.5 | *to be measured* |

## 9.13 Results Through the Compiled Path

Everything in §§9.1–9.4 was measured with `proto-engine`, a hand-written harness with no compiler in it. That was the honest thing to build first — a measurement is worth more than a compiler that produces no measurements — but it left the strongest available check on those findings unused. The language, the type system, the IR and the verifier were all specified and none of them was in the measurement path.

They are now. `crates/nilestream` compiles a Niles program with `niles-lang`, verifies the resulting circuit with `niles-ir::verify`, installs it on the `nilestream-core` REV runtime, and runs a workload against a hash-chained, epoch-ordered ledger. Nothing between those stages is hand-built. Two of the headline findings were re-measured along that path, and a third — durability — was measured for the first time, on a write path that has an `fsync` in it.

This is a different kind of confirmation from re-running the same harness with a different seed. It shares the workload generator and the ledger with §§9.1–9.4, but the view definition, the key derivation, the materialization decision, the anchor discipline and the eviction budget all now come from a compiled artifact rather than from hand-written Rust. A finding that survived that translation is a finding about the mechanism rather than about one implementation of it.

### 9.13.1 SC7 through the compiler (E11)

Dense keys, so each key accumulates real history: 200 accounts, Zipf *s* = 1.1, 4,000 reads, budget 20 entries. The figure is base rows read per upquery.

| Epochs | *C* = 0 (no checkpoints) | *C* = 16 | *C* = 64 |
|---:|---:|---:|---:|
| 5,000 | 58.5 | **6.9** | 19.1 |
| 20,000 | 234.0 | **8.3** | 28.6 |
| 80,000 | 931.5 | **8.4** | 31.7 |
| **growth over a 16× history increase** | **15.9×** | **1.22×** | 1.66× |

Without checkpoints the fold is history-length: cost grows 15.9× as history grows 16×, which is the linear behaviour §9.4.1 reported and which refuted the anchor-index claim. With *C* = 16 it is flat at 8.4 against the Theorem 3.7 bound of *C*/2 + 1 = 9. With *C* = 64 it is bounded and still climbing toward its own predicted 33, which it has not reached by 80,000 epochs.

The hand-written harness measured 6.8 → 8.3 → 8.0 → 8.5 for *C* = 16. The compiled path measures 6.9 → 8.3 → 8.4. **SC7 is confirmed through the compiler, not merely around it.**

### 9.13.2 The phase diagram through the compiler (E12)

20,000 epochs, 20,000 reads, 20,000 accounts, Zipf *s* = 1.1. Budget swept against the price of memory; cost is `resident_entry_epochs × price + deltas_applied + base_rows_read`, in counted-work units.

| Budget | 0.0001 | 0.0005 | 0.002 | 0.01 | 0.05 |
|---:|---:|---:|---:|---:|---:|
| 250 | 75,541 | 77,517 | 84,926 | **124,443** | **322,027** |
| 500 | 51,746 | 55,636 | **70,223** | 148,021 | 537,007 |
| 1,000 | 40,818 | **48,308** | 76,399 | 226,214 | 975,292 |
| 2,000 | **38,619** | 52,145 | 102,871 | 373,406 | 1,726,084 |
| 4,000 | 39,435 | 58,648 | 130,698 | 514,960 | 2,436,271 |
| 8,000 | 39,439 | 58,663 | 130,752 | 515,226 | 2,437,599 |
| full | 39,439 | 58,663 | 130,752 | 515,226 | 2,437,599 |

Three things are visible, and the third is the one that matters.

First, **the optimum is strictly interior wherever it is not at the swept boundary**: at price 0.0001 the best budget is 2,000, not 250 and not full.

```text
PROPOSED — MISMATCH-F-09. Not applied to the sentence above.

"strictly interior wherever it is not at the swept boundary" cannot be refuted by any
measurement: an optimum at an extreme *is* at the swept boundary by definition, so the
sentence excludes its own counterexample. The E12 table shows extremal optima at both price
ends, and the restated hypothesis has to survive that rather than define it away.

THE REPLACEMENT WORDING

  §1.6 H-S1, final clause:
    "…and there is a memory-price interval within which the cost-minimizing budget is
     strictly interior — neither the smallest swept nor full materialization. On the E12
     sweep that interval is **[0.0005, 0.002]** in units of one base-row read per resident
     entry per epoch, read from results/e12_phase_compiled.csv; outside it the optimum is
     extremal, and that is expected rather than excluded: at a memory price of zero,
     residency is free and full materialization is optimal by construction."

  The falsifier, stated so it can fire:
    "H-S1 is refuted by a **monotone** cost-versus-budget curve at any price inside
     [0.0005, 0.002] — that is, by a price in the stated interval at which the cheapest
     budget is the smallest swept or full materialization. An extremal optimum outside the
     interval refutes nothing."

  §9.13.2, this sentence:
    "At price 0.0001 the best budget is 2,000 — neither the smallest swept (250) nor full.
     At prices outside [0.0005, 0.002] the optimum moves to a boundary, which is the
     behaviour the restated H-S1 expects rather than the behaviour it excludes."

WHY THE INTERVAL AND THE CROSSOVER ARE THE SAME NUMBERS

  They are the same measurement read two ways: the price range in which partial and full
  materialization are within a factor of each other is the range in which the optimum can be
  interior. Stating it once and citing it twice is the honest form; deriving the second from
  the first without saying so would make one measurement look like two.
``` That is the corrected claim §9.3.4 arrived at after the first phase diagram turned out to be an artifact of charging peak residency rather than the residency integral.

Second, **the boundary sits between 0.0005 and 0.002**, which is the band the hand-written harness located.

Third, **full materialization is never uniquely optimal at any price tested**, and is indistinguishable from a budget of 8,000 — because at 8,000 the budget stops binding. That is the shape the theory predicts: above the working-set size, partiality is not a different strategy, it is the same strategy with the constraint slack.

### 9.13.3 The cost of durability, measured for the first time (E13)

§9.5 marked durability *to be measured*, because the research prototype has no `fsync` and no thread. The write path now has both. `SyncPolicy::Always` is the ledger-grade setting; `Never` is not a configuration anyone should run a ledger under and exists here to make the price of the guarantee visible by removing it. 2,000 transactions per thread, 32-byte payloads. Wall-clock and therefore machine-dependent; the ratios are the transferable part.

| Threads | Always (tx/s) | Never (tx/s) | Cost of durability | Txns per fsync (Always) |
|---:|---:|---:|---:|---:|
| 1 | 4,450 | 25,457 | 5.72× | 1.0 |
| 2 | 5,197 | 47,110 | 9.06× | 1.0 |
| 4 | 11,225 | 80,415 | 7.16× | 2.4 |
| 8 | 20,104 | 96,913 | 4.82× | 4.6 |
| 16 | 30,440 | 144,558 | 4.75× | 8.8 |

**The cost of durability falls as concurrency rises.** That is the result. A single-sealer design looks like a bottleneck and is routinely rejected as one, but an `fsync` costs the same whether it commits one transaction or five hundred, so the sealer drains what is waiting, seals it as one epoch, and syncs once. Transactions per fsync rises 1.0 → 8.8 across 1–16 threads, and the durability penalty falls from 5.7× to 4.8×. Throughput under **full durability** scales 4,450 → 30,440 tx/s, a 6.8× improvement from a 16× increase in concurrency.

The honest reading is narrow. This says a single sealer with group commit is a batching opportunity rather than a hard ceiling *at this scale, on this machine, with this payload*. It says nothing about a distributed commit, nothing about contention on a hot account (the sealer serialises everything, so there is no contention to observe), and nothing about how either figure compares to a production database. Those cells remain *to be measured*.

**And this table measures the ledger crate, not the server.** The threads here drive `nilestream-ledger` directly. The daemon that serves the wire protocol holds an `Arc<Mutex<RevEngine>>` and takes it once per query, so a client-side version of this experiment would measure the mutex rather than the sealer, and the scaling above would not appear. The two are different artefacts and the distinction is load-bearing: the group-commit result is about the write path's *design*, and the server's concurrency is an implementation limitation recorded in §9.14.1.

### 9.13.4 What building the compiler found in the thesis

Running the checker over this thesis's own worked program — the one printed in Appendix B.20 — produced four errors. Each is reported here because each is a case of the instrument catching something the argument had missed, which is the only reason to build an instrument.

**One.** `holds` was declared a `ledger`. A ledger must carry a conservation rule, and a hold does not conserve: it is a one-sided encumbrance, not a double-entry movement. Declaring it a ledger promised a double-entry invariant that no hold satisfies. It is a `base` — immutable and fully retained, but not conserved. The distinction had been made correctly in Chapter 3 and lost in the example.

**Two, and this is the important one.** `available_balance` was defined over `ledger_balance` and served at `ledger_consistent`, while `ledger_balance` is served at `read_your_writes`. The effect calculus rejected it under rung monotonicity. The reasoning is the one §4.6 gives: an availability decision built on a read-your-writes balance is at most read-your-writes fresh however strict the downstream contract claims to be, because the staleness entered one hop upstream and no annotation downstream removes it. **This is precisely the failure supervisory guidance describes** — two derived views of one ledger disagreeing at the moment a decision is made — and it was present in this thesis's own worked example, written by the author of the calculus, until the calculus was run over it. It is the single strongest piece of evidence in this thesis that the check is worth having.

**Three.** `settle_fx` declared `! { append, debit<usd>, credit<eur> }`, as though a conversion debited one currency and credited the other. It cannot: that conserves neither currency. An `fx` form is two conserved legs and therefore debits *and* credits in both. The declaration described a transaction that could not exist.

**Four.** The IR verifier rejected `statement_mtd`, whose predicate was `p.value_date >= month_start()`. `month_start()` reads the wall clock, so it is not reproducible, so a reconstruction of an evicted entry could disagree with the value it replaced. This is not a lint. It would make the reconstruction-equivalence theorem **false** for that view rather than merely unproven: a balance recomputed after an eviction could legitimately differ from the one evicted, and no audit could distinguish that from a defect. A view boundary must be a value, not a moment.

Three further defects were found in the engine by its own tests, and are recorded because each would have been a money bug in production rather than a crash.

* **Two copies of one idempotency key arriving in the same commit batch both committed.** The check consulted committed history but not the batch being assembled, and a client retrying quickly — or a client and a proxy retrying together — lands both copies in one drain. The failure mode is not an error; it is a duplicated payment. Caught by a test that submits the same key from two threads.
* **A duplicate was accounted for after its reply was sent**, so a caller could observe an outcome before the state that produced it was visible. The same ordering rule as durable-before-visible, one level up. It made the idempotency test fail about one run in three, which is exactly the frequency at which a race gets dismissed as flakiness.
* **`SyncPolicy::Always` synced twice per epoch** — once in `append`, once in the sealer. The benchmark surfaced it as 0.5 transactions per fsync, a figure with no sensible interpretation, which was the clue. Removing the second sync raised single-threaded throughput from 3,003 to 4,450 tx/s.

### 9.13.5 What Writing Niles Found in Niles

Building a compiler tests a language; *writing programs in it* tests the language differently, and finds different things. Two files were written in Niles after the compiler existed — the banking domain layer (`niles/std/bank.niles`) and Nilestream's own telemetry views (`niles/nilestream/observability.niles`) — and a third, `examples/available_balance.niles`, was written *before* it. All three are now kept compiling by a test.

**The reserved set was wrong, and only real code showed it.** The banking layer's first function is `fn transfer(from: Id<Account>, to: Id<Account>, ..)`, and it did not compile: `from` was reserved. That is the most natural name for the source account in a transfer, and a language that forbids it is a language a bank rewrites its code for. The registry's own stated policy — *a keyword is unreserved unless a written justification records why the grammar cannot be written without reserving it* — had been applied conscientiously to the novel vocabulary and not at all to the inherited SQL vocabulary, where it was simply assumed that SQL's reservations were necessary.

They are not, and the reason is §B.19's: **reserved-word count is a function of parser technology.** SQL reserves `from` because an LALR(1) grammar cannot otherwise resolve the projection/clause boundary. A hand-written parser with unbounded lookahead resolves it by position. Seven words — `from`, `group`, `order`, `limit`, `offset`, `values`, `end` — were demoted from *reserved* to *non-reserved*, and **not a single test broke**, because SQL clause parsing never went through the identifier path in the first place. The reserved set fell from 66 words to 59.

A second correction of the same kind: `lineage: full` did not parse, because `full` is reserved for `full outer join`. Contract keys and values now accept any keyword, since a `serve { .. }` block has exactly one reading and reserving a word there buys nothing.

Neither correction was discoverable from the grammar or the test suite. Both required someone to write a program they actually wanted to write.

**A pre-compiler example used seven constructs that do not exist.** `examples/available_balance.niles` was written as an illustration before the compiler did. Run through it, the file used a `.minus()` stage, `.value_date_between(..)`, `.window(sliding(..))`, `lineage` as a relation-level rule, an `idem"..."` adjacent-string literal, a `?placeholder`, and `Enum<Product>`. None was a typo. Each was a plausible-looking form that read correctly, would have passed a reviewer, and denoted nothing.

That is the ordinary fate of a language specified only in prose, and it is a specific argument against the way language proposals are usually evaluated: the example in the paper is not evidence that the language can express the example. It also, independently, contained the same rung-monotonicity violation as Appendix B.20 — `available_balance` derived from `ledger_balance` — which means the author made that error twice, in two files, having formulated the rule that forbids it.

**Where the checks were silent.** Worth recording symmetrically. The confidentiality check (W17) fired only once across all three files, and only because a test provoked it; the linearity checker fired on holds but never on posting halves, because `post(d, c)` consumes both in one call and there is no idiom in which they are dropped. Neither is evidence the checks are unnecessary, but both are evidence that the *frequency* claims one might be tempted to make about them are unsupported by this sample. Three files is not a corpus.

### 9.13.6 A Defect in the Instrument

Finally, one defect in the instrument itself, recorded under the same discipline as the phase-diagram artifact of §9.3.1. The end-to-end runner initially advanced the runtime with its loop counter rather than with the epoch the ledger actually sealed, and the ledger numbers epochs from zero. **The failure mode was not a wrong answer; it was a silent null.** Every maintenance counter read zero, the reconstruction figures looked entirely plausible, and the run would have been reported as evidence that maintenance costs nothing. The runner now refuses to return a result from a run in which no delta was observed at all, on the principle that a null which looks like a measurement is worse than an error.


## 9.14 Results Over the Wire, Through the Rewrite, and Through the Checker

§9.13 put the compiler in the measurement path. Three further experiments put, respectively, a *baseline*, a *rewrite* and a *corpus* in it. Each answers a question the preceding sections could not, and each is reported with the defects it found, because in every case the instrument caught something before the result did.

### 9.14.1 The first wall-clock comparison against the baseline (E16)

`results/E16-wallclock.md`, generated by `bank-bench --run`; nothing in it is typed by hand.

Every measurement in §§9.1–9.4 and §9.13 reported *counted work* inside the prototype, and the one wall-clock table (§9.4.4) was in-memory, single-threaded and compared to nothing. §7's performance contract states four targets relative to PostgreSQL. They were predictions in the typography of results. E16 is the first measurement against the baseline the specification names.

<!-- BEGIN:E16-contract results/E16-wallclock.md#contract -->

*Generated from `results/E16-wallclock.md`. Do not edit by hand.*

| Workload | Contract (SPEC-ENGINE Part 0) | PostgreSQL | Nilestream | Ratio | Verdict |
|---|---|---|---|---|---|
| oltp | 5–10× PostgreSQL | 4518 ops/s | 4189 ops/s | 0.93× | **NOT MET** |
| analytical | 10–12× PostgreSQL | 330.0 ops/s | 44.4 ops/s | 0.13× | **NOT MET** |
| point | parity with PostgreSQL | 122.1 µs p99 | 131.1 µs p99 | 0.93× | **PARITY** |
| durable | parity with PostgreSQL | 4654 ops/s | 3820 ops/s | 0.82× | **PARITY** |

<!-- END:E16-contract -->

Both sides are driven over the PostgreSQL wire protocol *through the same client*, over the same protocol path (recorded per CSV row), so neither is spared the protocol cost the other pays. **All four rows are measured on both targets.** They were not: three of them read `NOT RUN` against reasons — "no write surface over the wire", "a scan-and-group-by surface is not exposed" — that were true when they were written and had stopped being true, so three quarters of this table reported a gap in the engine that was a gap in the harness's beliefs about it.

**Two rows say NOT MET, and that is the result.** The engine is 0.93× PostgreSQL on durable OLTP against a contract of 5–10×, and 0.13× on the analytical workload against a contract of 10–12×. Both are attributed in `docs/BENCHMARK.md`'s enumerated *Known limitations of the Nilestream path*, and neither is attributed to the engine's correctness:

* **OLTP** — items 3 and 6, and a third that belongs on the record. The simple query path compiles every statement afresh (parse, resolve, typecheck, lower, verify), and this machine has two cores against a contract written for a 48-core baseline figure. The third is structural: **the daemon serves every query under one mutex.** `daemon.rs` holds an `Arc<Mutex<RevEngine>>` and takes it per query, on a thread-per-connection model, so the engine is concurrent in its connections and serial in its work — a design that cannot use a second core for reads over an immutable base, which is precisely the coordination-free read §7.5 claims. A 5–10× multiple against a 48-core baseline is not reachable through a global lock, and that is a property of this implementation rather than of the theory. It is listed here rather than in the future work because a limitation that explains a NOT MET belongs beside the NOT MET.

  Not the ledger, in any case: the `durable` row shows the write path at parity with PostgreSQL's, on the same device, at the same `fsync` cost, with `synchronous_commit = on` on one side and `SyncPolicy::Always` on the other.
* **Analytical** — item 4. An unkeyed `group by` materialises the whole base per query. The engine is doing *more work* than PostgreSQL rather than the same work more slowly, and where that trade pays is what §9.3's phase diagram characterises. Three of PostgreSQL's five analytical statements are also outside the lowered fragment — `count(*)`, `count(distinct …)`, `order by <aggregate>` — so the row compares five statements against three, and each missing construct is named with its reason rather than the fragment being widened during a benchmark.

What this table supports is therefore **an engine with a measured baseline and a characterized gap**, and not a performance claim. §7's contract remains the target; two of its four rows are unmet at this commit, by a factor named against a listed cause.

The `point` row is an engine result rather than a protocol one, and it is a **stronger** result than the one previously reported here. The wire path evaluates the compiled circuit over a source scan through the anchor index and does not consult the partially materialised view at all, so **every point read is an anchored reconstruction: the measured miss rate is 1.00**. Parity with PostgreSQL's indexed aggregate while reconstructing every read is a different claim from parity with a warm cache.

The sentence this replaces said "the measured runs sit at 8–14% misses". They did not. The harness configured a residency budget of 100,000 against 10,000 accounts, so after warm-up nothing was ever evicted and the true rate was zero; the figure was typed into the renderer's prose and no column carried it. The budget now binds at a quarter of the key space, the harness warns when it does not, and `miss_rate` is a column of every CSV row.

Two defects, and neither was in the engine.

**A 640× stall in the wire path.** The first run measured 23 point lookups per second against PostgreSQL's 13,600. Per-query compilation measures 0.02 ms, so the "compile every query" concern is 0.5% of the budget. The cause was that `pg_wire::write_all` issued one socket write per protocol message, so a four-message reply sat waiting on Nagle's algorithm and the peer's delayed-ACK timer: 43 ms per query is that timer wearing a database's clothes. One buffer, one write, `TCP_NODELAY`, and the same workload reached about 14,700/s. A counted-work benchmark could not have found this, because no unit of counted work was being spent.

**A mis-calibrated plausibility gate.** The harness refuses a throughput figure inconsistent with the device's measured `fsync` cost, on the principle that a durable-write number above the storage ceiling means the commits are not reaching storage. The gate fired at 16× over its ceiling — and the gate was right and the *published* figure was wrong: 333 txn/s/core had been carried from a device with power-loss protection, where an `fsync` costs 1.6–12.4 µs, to one without, where it costs 891–2,974 µs. The calibration is now measured per device at run time rather than quoted.

### 9.14.2 Subquery unnesting (E17)

`results/E17-unnesting.md`. Dreseler et al. measured subquery unnesting at roughly 510× geometric mean against join ordering's 7%, which is the finding that reorders an optimizer's priorities. §6.11's join ordering was built first; this is the rewrite that measurement says is worth two orders of magnitude more.

Three pieces of the IR had to exist before the rewrite could.

**A nested form.** A correlated subquery had no representation: no `exists` in the surface, no dependent join in the IR, so "unnesting" was a rewrite with no input. `Op::Apply` is that input, and it is deliberately **not incremental** — one new outer row re-scans the inner relation, so a dependent join has no delta rule and the verifier refuses one on a path to a served output. That reframes the whole exercise: unnesting is not an optimisation a busy planner may skip, it is the step that makes a correlated query expressible as a view at all, and the speedup is a consequence.

**A null.** The IR's value model was `i128`, so `not in` was unstatable. The IR now carries `Value::{Null, Int}` and Kleene three-valued logic, and keeps §3.3's three absences apart by name: `null` (unknown in the data), `Option::None` (absent in a program), and the evicted `Hole` (absent from memory, and reconstructible). Collapsing any pair is a defect with a name, and the third collapse is the `Err(_) => 0` of §1.1.1.

**One semantics.** The reference evaluator moved out of a test module and became public, shared by §6.13's schedule catalogue and by this corpus. Two copies of a semantics is two semantics, and a rewrite could then be "correct" under the copy its own author wrote.

Twenty-four correlated queries, each built nested and unnested, checked **denotationally** — the two circuits must denote the same Z-set on a dataset carrying duplicates, retractions, nulls on each side and empty groups. Deliberately not "the unnested plan contains a semi-join", which is satisfied by a plan containing a semi-join that computes the wrong thing. The eight `not in` cases are additionally checked against an oracle written straight from the three-valued truth table, because denotational equivalence establishes that the rewrite is faithful to the `Apply` and says nothing about whether the `Apply` is faithful to SQL.

**`not in` is the case worth stating.** A row survives `where` only when the predicate is *true*, so: a null probe is unknown and drops, even against an empty subquery; a match is false and drops; and a null anywhere in the subquery's column makes every non-matching probe unknown, so it drops too. A single null therefore makes `not in` return **no rows at all**. A plain anti-join keeps exactly the rows that did not match — unknown ones included — and is wrong in both directions. The rewrite is a filter, an anti-join, and a second anti-join against a *null witness*: one row per correlation group containing a null. The uncorrelated case needs no special handling, because with an empty correlation the second anti-join has an empty key and reduces to "keep the rows iff the witness is empty".

<!-- BEGIN:E17-curve results/E17-unnesting-table.md#curve -->

*Generated from `results/E17-unnesting-table.md`. Do not edit by hand.*

| k | outer rows | inner rows | corpus nested | corpus unnested | corpus ratio | correlated ratio |
|---|---|---|---|---|---|---|
| 1 | 9 | 7 | 1396 | 1101 | 1.27x | **1.44x** |
| 4 | 36 | 28 | 15736 | 5388 | 2.92x | **4.29x** |
| 16 | 144 | 112 | 225376 | 37116 | 6.07x | **15.71x** |
| 64 | 576 | 448 | 3500416 | 397308 | 8.81x | **61.39x** |

<!-- END:E17-curve -->

The correlated-regime ratio quadruples as *k* quadruples, which is the signature of removing a quadratic rather than shaving a constant. The whole-corpus ratio grows more slowly, and the reason is reported rather than smoothed: the corpus holds three regimes and only one of them has an asymptotic win to give — the uncorrelated cases stay quadratic in both plans, and the empty-inner ones have no inner loop to remove. A ratio taken over a mixture of asymptotics is a number without a meaning, so the regime is computed per case rather than declared, and a test guards the partition so a case that failed to speed up cannot be quietly reclassified.

The 510× figure is *not* the comparable number and is not claimed: that is a wall-clock geometric mean over TPC-H at one scale factor, and this is counted work over a purpose-built corpus as a function of scale. What can be said is that the ratio is unbounded in *k* rather than a constant.

Two corrections the measurement forced, both recorded under the discipline of §9.3.1.

**The instrument was wrong first.** The reference evaluator executed every equi-join as a nested loop, at |L|×|R| — the same cost as a dependent join — and therefore reported that unnesting saved nothing. No engine executes an equi-join that way, and a cost model that says otherwise cannot distinguish the two plans the experiment exists to compare. It now builds an index on the right and probes it.

**There is a crossover, and it is reported rather than smoothed away.** At the original nine-by-ten dataset the `not in` cases did *more* work after unnesting, because the unnested form is six operators and at that size the constant beats the asymptote. Loosening the assertion would have discarded the actual result, which is a phase boundary of exactly the kind §9.3 is about. The corpus is now scale-parameterised, the crossover is measured per case, and three cases have a ratio permanently below 1 — the ones whose subquery is empty, where there is no quadratic to remove and the extra nodes are pure cost. A planner could act on that; it is in the table rather than in a footnote.

**And two defects that had nothing to do with subqueries.** Closing the surface path — making a query a user can write reach the rewrite — turned both up within minutes.

First, a correlation whose two columns shared a name was silently left behind as a tautology. `where u.k = t.k` is the commonest correlated predicate there is, and both sides are called `k`; a rule that asked only "which schema does this name resolve in" found it resolved in both, gave up, and left the equality as a filter on the inner side, where it lowered to `k = k`. Always true, so the subquery matched every row and the `exists` became a no-op returning the whole outer relation.

Second, and worse: **`select k from t where t.z = 1` returned every row.** Two faults compounded. Niles's two ancestries disagree about one character — in Rust `a = b` assigns, in SQL's `where` clause it compares, and SQL has no assignment expression at all — and the parser took the Rust reading everywhere, producing an `Assign` node no `where` clause can contain. Lowering then read `self.scalar(..).unwrap_or(Scalar::LitBool(true))`, so a predicate with no lowering became the constant `true` and the clause vanished. The query looked correct, the plan verified, and no answer-level test could catch it, because every row it returned was a real row.

That is the third time in this programme that a *reasonable default for an absence* has turned out to be a wrong answer wearing a plausible shape, after `Err(_) => 0` in the kernel and `sum` over an empty group. The fix offers no default at all: `true` returns rows that should have been filtered out and `false` hides rows that exist, so an unlowerable predicate is now a diagnostic naming itself, and the view does not lower. §11.5.7 names the pattern.

### 9.14.3 Is the soundness theorem about code anyone writes? (E18)

`results/E18-solver-verdicts.md`. §11.5's argument for a checkable property requires that the check be sound *and* that its incompleteness be characterised rather than merely admitted. E14 establishes that Niles catches eleven of twelve deliberate defect classes at compile time where PostgreSQL catches none. The objection that raises is sharp: a checker with an `Undecided` verdict can do that and still be undecided on ordinary code, in which case Contribution 4's soundness theorem is true and applies to a fragment nobody writes.

Forty correct banking functions — transfers, fee sets, syndicated allocations, symbolic amounts, multi-currency legs, holds, and guarded paths — run one at a time through the front end. **Nothing is undecided.** Thirty-five are proved to conserve; five carry no conservation obligation at all, being holds. Five deliberately defective functions are the negative control, and the split between them is the honest part: three are reported as `Violates` where the arithmetic is straight-line and provably wrong, and two as `MayViolate` where the same arithmetic is reached across a control-flow merge. An accusation must be a must-statement; a checker that says "this transaction cannot balance" about a program it could not follow teaches its users to switch it off.

The guarded group is the surprise, and it sharpens the claim rather than merely supporting it. The undecidability behind the fourth verdict — Müller-Olm and Seidl's reduction from Post's Correspondence Problem — is about deciding whether an *affine relation holds at a program point*, and a limit check is exactly such a relation, so guarded transfers were expected to be undecided. They are proved, because **conservation asks a different question**: whether the net is zero on every path, not whether the guard is true. A guard that gates *whether* a balanced transaction happens does not threaten conservation at all. The verdict remains forced in general — a guard that made the *amounts* depend on an affine relation would reach it — and that is not what ordinary banking code looks like.

One correction, and it went the other way from the usual. The corpus initially reported the branch-defect case as `Undecided`, because the row join at a control-flow merge goes to top. The response was not to weaken the test: the checker now judges each arm *before* the join and downgrades a straight-line `Violates` to `MayViolate`, so a defect on one branch is reported as a defect on one branch. The corpus made the analysis more precise rather than the assertion more forgiving.

### 9.14.4 The bootstrap gates (E15)

`results/E15-bootstrap-gates.md`; the argument is Appendix E's, and only the evaluation-relevant part is stated here. Two front-end stages are now written in Niles — a lexer and a recursive-descent parser — and each is compared against the reference implementation over a corpus and over its own source, the parser node for node across 1,200 lines including its own.

The parser round is the one that produced a result about the *reference*. Assignment was left-associative, contradicting both Rust's rule and the comment sitting directly above the code, and it had survived every test in the workspace because associativity is invisible in a token stream: the lexer round could not have found it, and nothing else was looking. A second implementation of the same grammar found it within minutes of existing. That is the argument for stage-1 equivalence stated as a result rather than as a hope.

Two further findings came with it. Appendix B had no operator precedence table, so §6.25's rule that the appendix wins had nothing to win with; B.10.1 now states precedence and associativity normatively, and a test reads the table out of the markdown and checks every level against the compiler. And the stage-0 interpreter was spending about 95 KB of host stack per interpreted call frame in a debug build — a `match` over thirty expression variants compiles, unoptimised, to a frame holding the union of every arm's locals — which put a recursive-descent parser out of reach and whose failure mode was a process abort with no diagnostic. The cold arms moved behind `#[inline(never)]`, and a call-depth counter now turns the remaining limit into an ordinary error carrying a span.
