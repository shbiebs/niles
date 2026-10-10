# The universe: what every comparison is loaded with, and where each number comes from

*Cycle 16, card C16-11 (carrying cycle 15's C15-06). This document states the provenance of every
parameter the synthetic generator uses and the format in which real read traces will be
recorded. It contains no data.*

> **The confidentiality line.** Shapes and ranges only: no data, no identifiers, no
> counterparties, no amounts, and nothing attributable to an employer. The two reference
> institutions are recorded as size bands, with no country and no rank (the author's answer
> W-C3).

## 1. Where a parameter can come from

Every parameter below carries one of four provenances, and the one it carries is the strongest
claim made about it.

| provenance | means |
|---|---|
| **author** | the author's answer to the workload questions of 2026-09-28 (W1–W12, W-C1–W-C3, S-1), in the shape he gave it. These are one practitioner's ranges, not a survey. |
| **public** | a published dataset or paper, cited. Public calibration is **not fetched yet**: the fetch script is the author's to run, because the container's proxy blocks the sources (§5). |
| **assumed** | a value the generator needs and no source gives. It is declared here so a reader can disagree with it, and it is swept where a result depends on it. |
| **swept** | not a value but a range: every comparison reports its verdict across it. |

## 2. Two profiles, and a sweep between them

GBS must serve small banks with few operations and big banks with many (the author, 2026-09-28).
So the universe has two named profiles, and **every comparison is a size sweep** that passes
through both and locates crossovers between them (decision S-1).

| profile | facilities or accounts | activity | provenance |
|---|---|---|---|
| **a small lender** | ~10³ active facilities, ~1.2×10⁴ outstanding loans (≈12 per facility) | ~5×10² manual events a month; system-generated events (accruals, rate settings, scheduled payments) many more | author (W1, W-C1) |
| **a very large lender** | the A-2 scale row: 10⁶ accounts, 10⁷ postings a day, 50 views per account | ~10⁴ scheduled items per business day (a ~5×10⁴-row weekly scheduled-activity report) | author (A-2, W1, W-C2) |
| **the sweep** | 10³, 10⁴, 10⁵ accounts on the container; 10⁶ as counted work only, on Host C (the author's decision), not yet run | per the generator below | swept (S-1) |

## 3. The parameters

### 3.1 What the comparator's generator uses today

`crates/comparator/src/universe.rs`, behind E27 and E27b. Every parameter is written into those
results' headers.

| parameter | value | provenance |
|---|---|---|
| activity skew α (Zipf rank exponent over accounts) | **0.6** by default | author (W5: ~10% of activity in the busiest 1% of facilities, ~40% in the busiest 10%: mild skew, the hardest case for partial materialisation) |
| α, swept | 0.6 to 1.6 | swept (decision D-5) |
| which accounts are hot | a seeded permutation, so it varies per seed | assumed |
| multi-currency share | 35% of accounts hold a second currency; a transfer between two such accounts is in it half the time | author (W11: more than 30% of facilities are multi-currency); the 35% and the half are assumed |
| back-valued postings | 3% of transactions, up to 22 business days back | author (W10: 1–5%, up to a month); 3% and 22 are assumed within it |
| intraday profile | flat: transactions spread evenly over batches, one batch one epoch | author (W9: fairly flat) |
| month-end concentration | not modelled | author (W9: ~1.5× at month-end); **not yet in the generator** |
| history | 10 transactions per account before measurement, in batches of 200 | assumed |
| the read mix | reads drawn from the same Zipf law as activity | **assumed**, and the assumption matters (§4) |

### 3.2 What the generator does not yet have (C16-11's code half)

| parameter | shape | provenance | status |
|---|---|---|---|
| facilities per borrower | 1–10 | author (W1) | not in the generator |
| outstandings per facility | heavy-tailed, median a few, tail 50+; supply-chain-finance programmes with dozens per borrower, up to ~90% of loans by count in some regions | author (W1) | not in the generator |
| guarantors per borrower | tail near 100 | author (W1) | not in the generator |
| lenders per facility | median ~8, tail 100+ | author (W2) | not in the generator |
| active facilities per week | more than 50% | author (W3) | not in the generator |
| events per active facility per month | 3–10, median ~5 | author (W4) | not in the generator |
| reads of a closed facility | a few a year, bursty (audit, regulator, disputes) | author (W6) | not in the generator |
| screens per user per day; facilities per user | ~150 screens over 10–20 facilities | author (W7) | not in the generator |
| the view families read | outstandings and positions; cash flows and scheduled activity; facility and deal details; accounting and GL entries; other | author (W8) | not in the generator |
| **β: skew across view definitions** | which of a key's views are read; swept like α | **assumed**, swept | `comparator::universe::view_sampler` (cycle 16, C16-11); no study reads it yet (C15-03 is held, DB-7) |
| latency targets | 20 ms p99 lookup, 5 s p99 report | author (W12, A-2) | targets, not parameters |

## 4. Read skew is a hypothesis, not a premise

The economics of partial materialisation depend on **reads** concentrating, on few keys and on
few views. No published measurement of read concentration over accounts, facilities or views in
operational banking has been found (searches of 2026-09-30 and 2026-10-01). What exists is
evidence about **activity** and **transfer** skew, and it is cited as that and nothing more:

* the Rabobank intra-bank transaction network, 2010–2020, about 1.6 million nodes: counterparty
  degree follows a power law, in-degree γ ≈ 1.43 and out-degree γ ≈ 1.13
  (Saxena et al., arXiv 2109.10703) — counterparties from transfers, not reads;
* LDBC FinBench (PVLDB 18), profiled on an Ant Group production graph: transfer degree power
  laws, and a read-to-write query ratio of 20:1, without a model of which accounts are read;
* TPC-E's 20/60/20 customer tiers, Fedwire's concentration among its ten largest participants,
  and the author's own mild activity skew (W5).

So the generator's read mix — reads drawn from the activity law — is an **assumption**, the
comparisons sweep α rather than fix it, and β is a second swept parameter. The measurement that
would replace the assumption is a real operator-day read trace, recorded in the format of §6
once GBS has screens (decision DA-9).

## 5. Public calibration (not yet fetched)

`tools/fetch-public-calibration.sh` is to fetch, on the author's Mac, DTCC's CFTC swap data
(rates and FX), SEC ABS-EE samples and the BIS Triennial survey tables, and to write
`results/calibration/` with each file's source and licence; `tools/skew/` is to fit them with a
Clauset–Shalizi–Newman estimator (maximum likelihood, `xmin` chosen by Kolmogorov–Smirnov, a
bootstrap confidence interval from 200 resamples) into `results/E29-skew.md`. **Neither exists at
this revision.** Until they do, no parameter above has public provenance, and the table says so.

## 6. The read-trace format

One record per read, for DA-9's operator-day replay. Specified now; recorded when screens exist.

A trace is a UTF-8 text file. Its first line is `# gbs-read-trace v1`; the second is the
tab-separated header below; every further line is one read, in the order the reads were issued.

| column | type | meaning |
|---|---|---|
| `t_ms` | integer ≥ 0 | milliseconds since the trace began. **An offset, never a wall-clock time**: a time of day is information about an institution |
| `role` | one of `operator`, `checker`, `manager`, `auditor`, `regulator`, `system` | the reading principal's role, never its identity |
| `screen` | one of `positions`, `cashflows`, `facility`, `accounting`, `other` | the view family (W8) |
| `view` | a synthetic view id, `v1`, `v2`, … | which view definition was read, for β |
| `key` | a synthetic key: `f` and an integer for a facility, `a` and an integer for an account | the key read; pseudonymised **per trace**, so no key can be joined across traces |
| `epoch` | integer | the ledger's frontier when the read was issued |
| `anchor` | integer ≤ `epoch` | the epoch the read asked for; equal to `epoch` for a head read, lower for an as-of read |

What a trace never contains: names, account or facility numbers, counterparties, amounts,
currencies, dates or times of day, free text, or anything that would let a key be mapped back to
a real one. A trace that needs any of them is not a read trace and is not recorded.
