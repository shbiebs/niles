# 5. Research Design and Methodology

## 5.1 General Research Type

The research is **applied, quantitative, and experimental with a formal-deductive core**. It is applied because the object of study is motivated by, and validated against, a concrete class of production workloads; quantitative because every system hypothesis is operationalized as measurable variables with declared units; and formal-deductive because the six contributions are theorems whose proofs precede and predict the measurements.

The design is *theory-instrument*: the theory makes point and shape predictions — a crossover band, a zero regression slope, a zero violation count, a competitive ratio within a bound — and the instrument exists to expose those predictions to refutation. Each hypothesis in Section 1.6 names its falsifier, which is the Popperian structure this thesis holds itself to. The instrument is a means; a negative result that is *legible* is a successful outcome of this design, and Chapter 9 pre-registers the templates that make it so.

## 5.2 Research Analysis Method

**Hypothetico-deductive with differential baselines.**

For formal claims: deductive proof within the stated models — the LTS of Section 3.11, the cost model of Section 3.14, and the calculus λ_niles — with model-fidelity gaps tracked explicitly in Section 3.15 and attacked empirically rather than argued away.

For empirical claims: controlled experiments sweeping the independent variables of Section 1.6, with control variables pinned and reported. Analysis uses descriptive statistics (median, p95/p99, interquartile range across at least five seeded runs), regression where the theory predicts a functional form (log–log slope for H-S3/H-F1), agreement bands where the theory predicts a region rather than a point (H-S2), and ratio-to-offline-optimum where an optimum is computable (H-S7).

For correctness claims: differential comparison against the executable reference oracle 𝒪 (Section 3.11, Appendix F), which converts "is it right" into byte-for-byte comparison of every observable — balances at every rung's anchor, bitemporal answers, chain digests, admission decisions, and lineage.

Three analysis rules are fixed in advance to protect against researcher degrees of freedom. (i) Templates — axes, units, baselines, and the predicted direction with its theoretical source — are frozen before runs; deviations are reported as findings, not smoothed. (ii) Any figure quoted from the literature carries its original context, and vendor performance claims without published methodology are named as such and never used as baselines. (iii) Negative space is reported: where partiality loses, where the optimizer is beaten by a fixed policy, and where the theory's constants are loose.

## 5.3 Methodology Type

**Design science**, iterated through the phased program of Chapter 8: each phase builds an artifact increment, evaluates it against pre-registered kill criteria, and feeds the result back into the theory — a failed prediction narrows a theorem's model or kills a design branch. The methodology is also **constructive**: for upper bounds, language claims and generality results, the proof is an exhibited algorithm, typing derivation or translation, not an existence argument.

## 5.4 Analysis Model

Three layers, matched to the three kinds of claim, and every result in Chapter 9 is tagged with its layer.

**Formal layer.** The LTS, the cost model, λ_niles, the mode lattice. Claims are theorems; the threat is model realism, addressed by the verification-status table (Section 3.15) and by mechanization as declared future work.

**Systems layer.** The running engine under the benchmark harness. Claims are performance and resource curves; the threats are measurement and configuration, addressed by pinned configurations, seeded runs, and reproducibility tooling.

**Program layer.** The corpus of Niles programs — the banking library, benchmark transactions, non-financial domain libraries, ported SQL, and deliberately ill-typed mutants. Claims are compile-time acceptance or rejection and runtime invariant preservation; the threat is corpus representativeness, addressed by including third-party-derived workloads and by publishing the corpus.

## 5.5 Technique and Instrument

**Techniques.** Microbenchmarking (the ledger floor); macrobenchmarking (the core-banking workload); property-based testing of algebra laws; fault injection (crash, eviction storm, duplicate and reordered delivery, recovery mid-upquery); differential testing against 𝒪; exhaustive-interleaving model checking of the two lock disciplines identified in Section 3.17; static analysis of the corpus by the Niles compiler itself; and offline optimality computation by dynamic programming over recorded traces for H-S7.

**Instruments.** The benchmark harness with seeded generators (Appendix G); operating-system and process counters (resident set size, CPU time) alongside engine-internal counters (upquery rate, hit rate, applied-frontier lag, mode transitions, reconstruction latency distribution) exported through the observability surface (Appendix D); monotonic timers with reported resolution; the conservation checker of Appendix F as the invariant instrument; and the lineage subsystem as the traceability instrument. All instruments, configurations and seeds ship with the artifact.

**Instrumentation of the delayed-hit parameter.** Because Z — reconstruction time divided by mean inter-arrival time for a key range — is the decisive parameter in Theorem 4.2 and in the optimizer's eviction rule, it is measured directly rather than inferred: the runtime records reconstruction service times per range and arrival intervals per range, and Z is reported alongside every phase-diagram point. An experiment that reports a crossover without reporting Z is not interpretable.

## 5.6 Sample Description

Three populations.

**Workloads.** Synthetic core-banking histories with controlled parameters: account populations spanning several orders of magnitude; Zipf α swept across the empirically attested range — deliberately including the flat end (α ≈ 0.55–0.7, as reported for Meta's SocialGraph and CDN workloads) and not only the skewed end (α ≈ 1–2.5, as reported for Twitter's in-memory caches) — read/write mixes including write-heavy configurations, since more than a third of the Twitter clusters studied were write-heavy; and event volumes spanning several orders of magnitude for the history-scaling experiment. Product-mix scenarios cover the portfolio of Section 6.21.

Synthetic-with-published-shape is a deliberate choice with a declared cost: real banking data is unobtainable for legal reasons, so the generator's realism assumptions are a threat to validity (Section 9.8), mitigated by sweeping the parameters rather than fixing them, by publishing the generator, and by sensitivity analysis on mix weights. An industrial-trace partnership is named in Chapter 12 as the remedy this design cannot supply.

**Programs.** The Niles banking library; the benchmark's transaction set; deliberately ill-typed mutants for the negative half of H-S4 and H-S6; non-financial domain libraries for H-S8; and SQL programs ported to both surfaces for the translation tests of C6.

**Baselines.** PostgreSQL and MySQL with balances maintained by triggers plus materialized tables (the industry-default shape); an IVM engine of the Materialize/Feldera class consuming the same event stream with *total* materialization, which isolates partiality as the variable; Nilestream-NoAnchor, an in-tree ablation with versioning and anchoring disabled, which prices this thesis's own machinery; and a purpose-built ledger for the write-path floor. Selection rationale, fairness notes, and the exact question each baseline answers are in Section 9.1.

## 5.7 Method for the Foundational Hypotheses

H-F1–H-F4 are not benchmarked into truth; each has the method declared in Section 1.6.1, executed as follows.

**H-F2 and H-F4 are discharged in the formal layer** and mirrored executably. The duality theorem and the information-asymmetry lemma (Section 3.16) are proved within DBSP's algebra; the reconstruction-equivalence theorem is proved in Section 4.2. Both are mirrored as property-based tests — I∘D round-trips over generated histories, and evict/reconstruct round-trips over random schedules compared as canonical Z-sets at every epoch — so that the implementation is checked against the same laws the proofs use. A red property test reopens the hypothesis.

**H-F1 and H-F3 are argumentative with empirical companions.** H-F1's companion is the long-horizon run, shared with H-S3: both paradigms run against a live-generating source with the query set, hardware and skew held fixed, and per-answer compute cost and resident memory are regressed on accumulated input. The prediction is a non-zero slope for the finite-first stack and a slope indistinguishable from zero for the stream-first one; the falsifier is a zero slope for the finite-first stack. H-F3's companion is the architectural audit of Section 9.7, whose counting rules — what constitutes a component, what counts as a line of consistency glue, and what taxonomy defines an anomaly class — are fixed and published before any measurement.

**Honesty about what these experiments can establish.** H-F1 and H-F3 are positions about design, and no experiment proves a position. What the companions can do is show that the position's *measurable consequences* hold, and expose it if they do not. The thesis claims exactly that, and Section 9.9 states the verdict protocol accordingly.

## 5.8 Method for the Memory-Model and Immutability Findings

The claims of Sections 3.17 and 6.11 — that sealed data is race-free by construction and that the residual concurrency surface is two structures — are validated in three layers.

**By construction.** The implementation encodes immutability in types: sealed epochs are immutable values reachable only through shared references, and no mutable reference to prefix data exists in the type surface, so the compiler discharges the bulk of the claim. The formal warrant for taking that seriously is RustBelt's machine-checked result that well-typed programs in the λRust model exhibit no undefined behaviour, data races or memory-safety violations, with the central invariant that aliasing and mutation cannot occur simultaneously on a location — including for library types that use `unsafe` internally, provided they satisfy verifiable conditions [Jung et al., POPL '18]. Every `unsafe` block in Nilestream is therefore inventoried with the obligation it must discharge, and the inventory is published.

**By model checking.** The admission-queue and resident-map disciplines are extracted into models checked exhaustively over interleavings within the tool's bounds, with state-space coverage reported.

**By dynamic verification.** Sanitizer and interpreter runs over the concurrency suite, plus crash-recovery fault injection verifying that recovery re-derives frontiers exactly and that rebuilt views match the oracle byte-for-byte — recovery being, by Theorem 4.1, simply a large eviction.

**Acceptance criterion.** Zero races reported at all three layers, and every `unsafe` obligation either discharged by a written argument or eliminated. A partial result is reported as partial.

## 5.9 Reproducibility and Pre-Registration

Every figure in Chapter 9 is generated from the artifact: harness, seeds, configurations and analysis scripts are versioned together, and `make reproduce` re-runs every generator and then `git diff --exit-code`s the results, the thesis and `SPEC-LANGUAGE.md`, so a figure that no longer follows from the code fails the build. Five seeded runs minimum per point; medians reported, ranges given.

**What is enforced mechanically, exactly.** This section used to claim a pre-registration gate — "the figure generator refuses to emit a plot for which no template exists", and "the build gate refuses to produce results from a dirty tree". Neither exists. There is no template gate and no dirty-tree gate, and until T-19 the experiment harness swallowed results-file write errors with `.ok()`, so a run that could not write its own output reported success.

What *is* enforced, by `crates/bank-bench/tests/thesis_drift.rs` and `thesis/include-results.py`:

* every generated block in the thesis matches the file it names, and a stale one fails the test suite;
* `make reproduce` regenerates every results file and diffs, so a number in the thesis that the code no longer produces fails;
* the status of every claim is rendered from `thesis/status.toml` into all five places that state one, so they cannot disagree;
* Appendix B's keyword lists and Appendix E's figures are checked against the registry and the run that produces them.

**Pre-registration is a discipline here and not a gate**, and where it was exercised it is visible in the commit history rather than in a mechanism: `results/E16-band.md` states its predictions and was committed *before* the durable run, and `git log` is the evidence that the ordering held. That is weaker than a gate, and calling it a gate — which the paragraph this replaces did — makes an auditable claim out of an intention.
