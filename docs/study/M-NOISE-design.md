# M-NOISE: why the refusal rule refuses (C17-04): pre-registered design

**Status: pre-registered, 2026-11-11. Not run.** Committed before the harness change and before any M-NOISE number exists. A change after the first run goes in §8 as a dated deviation, never as an edit to §§1–7.

## 1. The question

E27b refused 351 of its 807 verdict cells (43%). Most refusals are the warm-up rule's: a cell is refused when either arm's warm-up runs have a MAD above 15% of their median (`crates/comparator/src/stats.rs`, `WARMUP_MAD_LIMIT`). Two causes fit that: **the runs are too short** (too few samples per run for a stable percentile), or **the host is noisy** (a 2-core container whose timing varies whatever the run length). They predict different things when the runs get longer, so the question is tested rather than read off the numbers.

## 2. The cell

| what | value |
|---|---|
| study settings | E27b's main shape (`Config::declared`): budget 5% of keys, checkpoint interval 16, α = 0.6, 10 measured runs, arms interleaved and rotated per run |
| series, size | single currency, 10⁴ accounts |
| seeds | 1, 7, 42, 100, 2024 |
| arms | P+ and P-inc (C17-03), the pair (P+, P-inc) |
| metrics | q1 p50 and p99 at 1, 2 and 4 clients: `q1_p50_us`, `q1_p99_us`, `c2_q1_p50_us`, `c2_q1_p99_us`, `c4_q1_p50_us`, `c4_q1_p99_us` |

## 3. The two run lengths

| | operations per run at 1 client | reads per client per concurrency level | warm-up runs | measured runs |
|---|---|---|---|---|
| **L1** (E27b's) | 200 | 60 | 3 | 10 |
| **L2** (2× with 2× warm-up) | 400 | 120 | 6 | 10 |

"2× with 2× warm-up" is read as: every per-run sample count doubled (so each percentile has twice the samples), and twice the warm-up runs. The warm-up MAD at L2 is therefore taken over six values rather than three; that is part of the treatment, not corrected for. The mix, the write share and every other setting are unchanged. The harness gains one option, `--length-factor N`, which multiplies `ops`, `conc_reads` and `warmups` by N; without it nothing changes.

**Order:** for each seed in the order above, the L1 point, then the L2 point, so the two lengths share the host's drift rather than one getting the first half of the session. Each point loads its own universe from scratch, as every comparator point does.

## 4. What is counted

For each length, each seed and each metric, the per-seed verdict of the pair (P+, P-inc) by `stats::judge`, the same function E27b's cells use. That is 6 metrics × 5 seeds = **30 cells per length**. A cell is **refused** when `judge` returns REFUSED, whatever the reason. Reported beside the count, descriptive only: the refusals by reason, and each arm's warm-up MAD as a share of its warm-up median, per metric and seed.

## 5. The rule

Let r₁ and r₂ be the refused cells out of 30 at L1 and L2.

1. **If r₁ = 0**, there is nothing at this cell to explain: the verdict is **inconclusive**, and no later design changes because of M-NOISE.
2. **Else, if r₂ ≤ r₁ / 2**, the run length is the cause: **every later design on this host uses L2's lengths (2×)**.
3. **Else**, the host is the cause: **comparisons at 10⁵ and above move to the Mac** (D17-7), and this host keeps 10³–10⁴.

No other outcome is defined, and the rule is applied as written.

## 6. Outputs

| file | class | what |
|---|---|---|
| `results/M-NOISE/L1/single-10000-<seed>.tsv`, `results/M-NOISE/L2/…` | machine-dependent | the ten points, as the comparator writes them |
| `results/M-NOISE.md` | machine-dependent | the counts, the reasons, the warm-up ratios and the verdict, rendered from the points by `comparator m-noise` |

Re-rendering from the points is byte-identical.

## 7. Host

This container (2 cores, 8 GB). Every point records its host, commit and toolchain, as the comparator's points do. **No build or gate runs beside an M-NOISE run.**

## 8. Deviations

None yet.
