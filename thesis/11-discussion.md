# 11. Discussion

## 11.1 When Is Niles Not the Answer

A theory is trusted in proportion to the honesty of its boundary claims. This design is the wrong choice in at least seven situations, and the phase diagram makes the first of them quantitative.

**Beyond the frontier.** Flat access distributions combined with strict-serializability demands everywhere. Theorem 4.2 says partiality buys nothing there, and a conventional fully materialized engine with a simpler operational story wins. This is not a hypothetical corner: the CacheLib measurements report production workloads at α ≈ 0.55–0.7, and a bank whose derived-read access resembles those rather than Twitter's caches should expect to sit near or beyond the frontier for its strict views.

**Where history is a liability.** Domains with legal erasure obligations over primary facts fight a never-partial base. Crypto-shredding is the only offered mitigation, and the most authoritative regulatory treatment of it says it moves "closer to the effects of data erasure" — deliberately declining to say it satisfies the right. Where a regulator rejects that, so must this design. The honest position is that immutability and erasure are in genuine tension and this architecture chooses immutability.

**Scan-dominant analytics.** Ad hoc, scan-heavy analysis over cold history is served better by a columnar warehouse. Nilestream's cold tier can feed one, and its REVs can be column-shaped, but pretending to be a warehouse would betray the design centre.

**Ultra-low-latency writes.** The epoch quantum places a floor on commit visibility latency. Tick-level trading systems will refuse it, and should.

**Adversarial or multi-operator trust.** The hash chain provides evidence to a party who retained a digest and checks it. Where participants do not trust the operator, the correct answer is a Byzantine-tolerant design, which this thesis scopes out.

**Small or invariant-free applications.** Where there is no money, no audit obligation and no read-model explosion, the payoff from invariant typing is small and the cost of a new language is not. SQL on a conventional engine remains the rational default, and the SQL surface here softens but does not eliminate the adoption cost.

**High-contention thresholds at extreme scale.** Proposition 3.2 is a constraint, not a design flaw to be engineered away: authorization against a floor cannot be made coordination-free. Where the authorization rate on a single account exceeds what a coordinated path can serve even with splitting, the answer is a business-level change — pre-funded sub-accounts, netting windows — not a systems trick, and the thesis says so rather than implying otherwise.

## 11.2 Risks and Mitigations

**Theory-model gap.** The proofs idealize the engine. Mitigated by the differential oracle, fault campaigns, and the assumption ledger of Section 3.15; residual risk acknowledged, with mechanization named as the remedy this thesis does not deliver.

**Single-project engineering scale.** A from-scratch DBMS, language and compiler is a very large surface. The position has improved and the risk has not gone away: a stage-0 compiler, a typed IR with a verifier, a REV runtime and a durable concurrent write path are built and tested, and the loop from source text to measured result is closed — but distribution, consensus, the wire protocols and the self-hosted back end are untouched, and Appendix E.0 now says so in the appendix that describes them. Mitigated by the spine discipline, by kill-criteria phasing that halts cheaply if premises fail, by aggressive reuse at commodity boundaries, and by scoping the self-hosted back-end as a bounded artifact with a published gap rather than an attempt to compete with a mature optimizer. The most likely failure mode of this project is not a wrong theorem but an unfinished instrument, and the phase structure exists to make that failure informative rather than total.

**Benchmark realism.** Synthetic workloads with published assumptions and swept parameters; industrial-trace partnership named as the remedy in Chapter 12. This is the threat a committee should press hardest, and the pre-registration mechanism is the only real defence.

**Premise risk on skew.** The literature disagrees about how skewed production access is, and the flat end of that range is unfavourable. Treated as a first-class design input: the optimizer exists precisely because the right materialization decision cannot be assumed, and S1's protocol reports the regimes separately rather than averaging them.

**Cryptographic and standards agility.** Commitment schemes, hash functions and currency data all age. Mitigated by algorithm identifiers in segment headers, rotation as an audited ledger event, and per-currency scale carried in the type rather than assumed.

**Misplaced emphasis between the two artifacts.** E14 establishes that the language case is
stronger than the engine case, and this document is weighted the other way: Nilestream
occupies substantially more of it than Niles does. That is a defect in the thesis rather
than in the work, and the honest mitigation is not to argue the engine up but to rebalance
the writing — §12 records it. A committee is entitled to ask why an argument whose evidence
favours the compiler spends most of its pages on the database, and the answer is historical
(the engine was built first) rather than principled.

**Seductive generality.** The temptation to grow Niles into an application language. Mitigated by holding the scope tiers of Section 6.12 as normative and by treating any proposal to add ambient I/O, wall-clock reads or unguarded recursion as a change to the thesis's claims rather than a feature.

**Optimizer opacity.** An adaptive component that changes behaviour under load is operationally frightening if it cannot be interrogated. Mitigated by Theorem 4.5(a) — mode changes cannot change answers — and by logging every transition with the estimates that caused it.

## 11.3 What Would Change My Mind — and What Already Did

Stated explicitly, because a thesis that cannot name its own refutation is not falsifiable in practice however carefully it words its hypotheses. This section is now in two halves, because three of these conditions have been tested and two of them fired.

**Already changed (measured, Chapter 9).**

* *"Partiality's advantage grows with skew."* **Refuted.** Full materialization's own footprint shrinks with skew faster than partial's does, so the memory ratio moves against partiality as skew rises (measured 12:1 at *s* = 0.5 → 2.7:1 at *s* = 1.3). The claim is replaced by a memory-price condition with an interior optimum (§9.3.4, Appendix J.9).
* *"Per-key anchor indices make reconstruction cost history-independent."* **Refuted twice** — once with a fixed key space and once with the key space grown in proportion — and then **restored** by a mechanism the design did not have: per-key checkpointing, which held cost flat at ≈ 8.5 base rows across a 64× increase in history (§9.4.1). This produced SC7 and is the clearest case in the thesis of an experiment changing the theory.
* *"The prototype can speak to hot-account contention."* **Refuted as a matter of instrument**: single-threaded execution cannot exhibit contention, so the flat hot-share result is a non-result and is reported as one (§9.4.4).
* *"A single-sealer write path is the throughput ceiling."* **Refuted, and in the useful direction.** The measured cost of durability *falls* as concurrency rises — 5.7× at one thread, 4.8× at sixteen — because an `fsync` costs the same whether it commits one transaction or five hundred, and the sealer batches. Transactions per fsync rise 1.0 → 8.8 (§9.13.3). This was a design worry, not a stated claim, which is why it appears here rather than in Chapter 4; it is recorded because it was the objection the design was most likely to fail on.
* *"Reconstructible epoch-anchored views need a new engine."* **Refuted.** The whole
  mechanism — partial materialization, honest absence, anchored reconstruction, per-key
  checkpoints, delta-proportional maintenance — runs in stock PostgreSQL 16 with zero
  divergences under eviction and the right asymptotics (§6.10.3, E14). This is the finding
  that most damages this thesis's own emphasis, and it stands: the engine contribution is
  cumulative rather than enabling, and Nilestream is justified as the instrument that makes
  the theory testable rather than as a capability nothing else could provide.
* *"The currency-row solver was sound."* **Refuted, twice, by reading it against the
  literature.** It had no control-flow join, so it analysed a program in which both arms of
  every branch run, and produced four false errors on an eleven-line correct transaction.
  The first fix over-corrected and downgraded a real forty-dollar hole to a warning. Both
  are recorded in §4.5.1, because a checker that accuses correct programs is worse than no
  checker.
* *"Move guarantees conservation of value."* **Refuted by Move's own paper**, which states
  that its type system "will not ensure that the total value of all Coins in existence is
  preserved". The correction strengthens rather than weakens this thesis, which is why it
  was easy to miss.
* *"The worked example demonstrates the calculus."* **Refuted in the most useful way available.** It violated the calculus. `available_balance` promised `ledger_consistent` over a `read_your_writes` input, which is the two-views-disagreeing failure Chapter 1 opens with, present in the author's own example until the compiler was run over it (§9.13.4). Nothing in this thesis argues better for a compiler than that.

**Still standing, with the result that would overturn each.**

* A conservation violation traceable to the *model* rather than to an implementation defect would falsify SC1 and halt the programme (K2). Five seeds × 10,000 transfers under continuous eviction have not produced one (§9.2.1); that is corroboration, not proof, and a single counterexample would still end it.
* A measured crossover far outside Theorem 4.2's band, in either direction, would mean the cost model omits a first-order term. The prototype located a crossover between memory prices 0.0005 and 0.002 in counted-work units (§9.3.3); a durable, concurrent implementation landing somewhere else would be informative.
* A fixed policy beating the adaptive optimizer across the workload suite would reduce SC5 to a planning-time heuristic. The measured margin of cost-aware eviction over LRU is real but uneven — 31% on reconstruction work, 4% on aggregate delay (§9.4.2) — so this one is closer to the edge than the others.
* A banking product from Section 6.22 that cannot be expressed without a kernel change would falsify the "banking as a library" claim and, with it, part of the generality thesis.
* A demonstration that the SQL-fragment translation is not semantics-preserving on some construct would require narrowing the fragment publicly rather than quietly.
* Strict serializability failing under an Elle-style cycle check would matter *even if the conservation suite still passed*, because a published analysis shows exactly that combination is possible (§9.2.2).

## 11.4 Project Identity

What this project is: **a theory thesis with a systems instrument.** The ordering matters operationally, and it has already decided several design questions — anchors are mandatory even where they cost; the IR verifier stays in the trusted base so the compiler need not; determinism gates are non-negotiable; the optimizer is forbidden from being a correctness dependency; and the reference oracle was built first, because a definition of correctness that cannot be executed is a definition nobody checks.

What it is not: a product, a blockchain, a fork of an existing partial-state system, or an SQL dialect. Its success criterion is the one stated in Chapter 1: the six results standing, their predictions met or their misses understood, and the instrument reproducible by strangers.
