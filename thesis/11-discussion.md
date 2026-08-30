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
* *"Noria cannot be used in production for core banking."* **Refuted as unsupported by its own evidence.** The attempt that motivated this thesis never built, started, or connected to Noria; zero SQL statements were executed against it, so no semantics of Noria were ever exercised, and the failure chain is entirely one of artefact availability. A reviewer's line — *you did not fail to build a banking system on Noria; you failed to build Noria* — is available and correct. §1.1.1 and §11.5.5 now say so. The Noria limits this thesis relies on are the ones its authors state in print, cited as literature rather than as findings.
* *"The bootstrap's stage 1 has no input, because stage 0 cannot compile the whole language."* **Refuted by re-reading the requirement.** A bootstrap needs stage 0 to *evaluate* Niles, not to emit machine code; "stage-0 compiler" had been read as "native compiler", which was this document's mistake rather than the design's. A tree-walking interpreter plus a Niles-written lexer produced four working gates in a day, and the gates immediately rejected four real defects — two keyword-table drifts, a case-sensitivity error arising from SQL and Rust disagreeing, and a flaw in the gate itself (Appendix E.19.1).
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

## 11.5 The Existence Questions, Answered Against Evidence

Six questions decide whether this thesis has a subject. They are not rhetorical, and three of them are answered against the thesis's own preferred emphasis. They are set out here rather than in Chapter 1 because each can now be answered with a measurement or a reading rather than an intention.

The general form of the burden is worth stating once. A new artefact — a language, an engine — earns its existence by exhibiting something that **cannot be obtained by composing what already exists**, and "cannot" has to mean cannot, not "would be inconvenient to". Three grades of answer are available, and the thesis is careful about which it claims:

| Grade | Meaning | What discharges it |
|---|---|---|
| **Enabling** | Nothing existing can do this at all | An impossibility argument, or an exhaustive negative survey |
| **Cumulative** | Existing systems can, at a cost the new artefact removes | A measured differential on a like-for-like task |
| **Editorial** | A matter of taste, ergonomics, or preference | Nothing; it is not a research claim |

An artefact justified only at the editorial grade should not be in a thesis. An artefact justified at the cumulative grade belongs in one, provided the differential is measured and the word "enabling" is never used for it.

### 11.5.1 What would have to be proved for Niles to exist?

The burden has three parts, and they are conjunctive.

**(i) A property must be statically checkable in Niles that is not statically checkable in SQL plus a host language.** Not *harder*: not checkable. The property this thesis nominates is **conservation of money across a transaction boundary under control flow** — that every path through a `txn` block posts a set of entries summing to zero per currency, decided before the program runs.

**(ii) The property must matter.** A checkable property nobody's defect corpus contains is a solution without a problem.

**(iii) The check must be sound, and its incompleteness must be characterised rather than merely admitted.** A checker that accuses correct programs is worse than no checker, and this thesis has direct evidence for that claim: an earlier version of the currency-row solver produced four false errors on an eleven-line correct transaction (§4.5.1).

**Status: (i) discharged, (ii) discharged, (iii) partially discharged.**

On (i): the differential-defect experiment (§6.10.3, E14) writes twelve defect classes twice — once against a good-faith PostgreSQL 16 schema using the strongest tool PostgreSQL offers for each job, once in Niles — and records the *stage* at which each is caught. PostgreSQL catches three at run time, none at compile time, and never catches nine. Niles catches eleven at compile time and warns on the twelfth. The nine PostgreSQL never catches are not oversights in the schema; they are properties a schema cannot express, because a `CHECK` constraint sees one row and a trigger sees one transaction, while conservation under branching is a property of a *program*.

On (ii): D10 of that corpus is the specific evidence. Dropping one trigger and inserting one row leaves the ledger total at `−500.0000` with nothing in the system able to say so. The APSN pattern of §1.1 is the same failure at institutional scale, and it is named in supervisory guidance rather than inferred.

On (iii): the solver is sound in the direction that matters — it never reports `Conserves` for a program that can lose money — and Contribution 4's soundness theorem is stated over its formal core. Its incompleteness is *characterised*: `Undecided` is forced by a reduction from PCP (Müller-Olm and Seidl), so no complete alternative exists; the domain is affine forms over opaque atoms per currency, which is a Karr-expressible query answered in a weaker non-relational domain, and the loop rule is exact rather than a widening because a row is a homomorphism into a free abelian group. What is **not** discharged is a mechanised proof of the soundness theorem; it is on paper, and §3.11 records its verification status honestly.

There is a fourth part of the burden that this thesis does **not** meet and should stop implying it meets: **that a new language is required rather than a library, a linter, or an embedded DSL in Rust.** The static analysis described in §4.5 operates on an AST; nothing about it requires that AST to come from a novel surface syntax. A Rust procedural macro over a `txn!{}` block, or a static analyser over a restricted SQL dialect, could in principle host the same currency-row solver. The honest claim is therefore that **the analysis is enabling and the surface syntax is editorial**, and §6.5's argument for a new language rests on integration — one type system spanning money, effects, consistency rungs, confidentiality and retention, rather than five analyses over four surfaces — which is a cumulative argument. Chapter 12 records the experiment that would settle it: implement the currency-row solver as a Rust macro and measure what it cannot reach.

### 11.5.2 Can reconstructible epoch-anchored views be implemented with only current Rust and SQL?

**Yes. Measured, not conceded.** This is the finding that most damages this thesis's emphasis, and it is reported first because of that.

E14 builds the entire REV mechanism in stock PostgreSQL 16.13: partial materialization as a cache table with a `state ∈ {present, hole, pending}` column, honest absence as a NULL value beside a non-null version — SQL's three-valued logic turns out to be an *asset* here, expressing "miss ≠ zero" natively — reconstruction as a `STABLE` plpgsql function folding the suffix after the newest checkpoint, and per-key checkpoints as an ordinary table. The results:

| Property | PostgreSQL 16 |
|---|---|
| Reconstruction equivalence under eviction | 0 divergences across 50 keys |
| Honest absence | 50 holes, 50 versions kept, 0 values kept |
| Checkpoint-bounded reconstruction | 5 buffer hits per reconstruction at *C* = 16 |
| Maintenance proportional to deltas | 100 applied, 880 skipped, over 500 epochs |

The theory is therefore **not** engine-specific, and that is a strength of the theory rather than a weakness: a result that holds only inside its author's prototype is a property of the prototype. It does mean the sentence "REVs require a new engine" is false and must not appear.

Three structural gaps remain, and they are gaps rather than defects: PostgreSQL has no per-key `REFRESH` (materialized-view refresh is whole-view), its isolation is per-transaction rather than per-view so a consistency ladder cannot be attached to a view definition, and it offers no stable read anchor a client can pin across statements. Each is a *cumulative* deficiency — expressible with enough application code — not an enabling one.

### 11.5.3 Has it been proved that Nilestream should exist?

**No, and the thesis should say so in exactly these words.** Nilestream is justified at the **instrument** grade, not the enabling grade.

The argument that survives is: the theory makes quantitative predictions — the eviction-consistency frontier, the cost law for rungs, the phase diagram — and testing them requires an implementation whose counted work is attributable to a mechanism rather than to a query planner, a buffer manager, and thirty years of accumulated optimisation. A measurement of upquery cost inside PostgreSQL measures PostgreSQL. That is a real reason to build a system and a poor reason to claim a system contribution.

What Nilestream demonstrably provides beyond the PostgreSQL construction, on evidence in this repository, is: per-view consistency rungs with a rung-monotonicity check (a view served at ℓ may not read a view served below ℓ), reconstruction paths that are *verified* to be anchored rather than assumed to be, counted work exposed as a first-class measurement, cross-shard commit whose coordinator is itself a ledger group, and an IR verifier in the trusted base so the compiler need not be. Each is cumulative. None is enabling. The correct sentence for the abstract is that **the theory is the contribution and Nilestream is its instrument**, which is what §11.4 already says and what the rest of the document must be brought into line with.

### 11.5.4 Is an SQL replacement needed, or only a MySQL/PostgreSQL-compatible engine with REVs?

Splitting the question is what makes it answerable, because the two halves have different answers.

**The engine surface: compatibility is sufficient and replacement is unnecessary.** §11.5.2 settles this. Wire compatibility is the right delivery vehicle, and this repository takes that seriously enough to implement it — PostgreSQL wire protocol v3 including the extended query path, MySQL packet framing, and the TLS negotiation state machines for both, with the deliberate design commitment that a wire protocol is a *surface, not a semantics*: there is no compatibility layer with its own execution path, because two ways to compute an answer is two answers that can disagree.

**The language surface: a replacement is not needed either, but an extension is, and the extension is not expressible as one.** SQL's obstacle is not that it lacks features; it is that its type system has no place to put them. A consistency rung is a property of a *view definition*; SQL has no view-level modality. A conservation obligation is a property of a *transaction body*; SQL's transaction is a control construct with no type. A currency is a property of a *value*; SQL's `numeric` has a scale but no unit, and `Kennedy`-style unit checking requires the unit to be part of the type rather than a column comment. Each of these could be bolted on individually — and PostgreSQL's extension mechanism is strong enough that several have been, in isolation — but they interact: a view's rung constrains which views it may read, which constrains where money may flow, which constrains what may be declassified. The thesis's claim is that these five analyses want one type system.

That claim is **cumulative, not enabling**, and this section is where the thesis stops overstating it. The strongest honest version: *a coherent type system spanning money, effects, consistency, confidentiality and retention has not been built, is not obtainable by composing five independent extensions because the analyses are mutually constraining, and Niles is an existence proof that it can be built.* An existence proof is a real contribution. It is not an impossibility result, and the thesis previously read in places as though it were.

### 11.5.5 Has it been proved that Noria cannot be used in production?

**No — and the attempt that motivated this thesis does not prove it either.** This is a correction to the thesis's own motivating narrative, and it is made here because a reviewer would make it otherwise.

The author's earlier attempt to build a core-banking system (GBS) on Noria is documented in a 4,565-line transcript, analysed line by line in `docs/research/gbs-noria-postmortem.md`. Its central finding constrains what may be argued from it:

> **Noria was never successfully built, started, or connected to. Zero SQL statements were ever executed against it. Every Noria failure in that transcript is a build, packaging, or distribution failure — not a semantic one.**

The failure chain is entirely one of artefact availability: the `noria` crate requires a 2021-era nightly Rust; it uses `impl_trait_in_assoc_type` in a form modern nightly rejects; its transitive `librocksdb-sys v6.20.3` panics under modern LLVM's bindgen; no published container image exists; and a source build pinned to `nightly-2021-01-01` still fails because *unpinned transitive dependencies* have moved on. The attempt then pivoted to Materialize, which produced the transcript's only genuine engine refusal (`CREATE TABLE with a primary key or unique constraint is not supported`), and ended without a confirmed end-to-end run on any engine.

Two further honesty obligations follow from that document, and both cut against a convenient narrative:

* **The build failure is partly self-inflicted.** The source build ran a bare `cargo build --release` with no `--locked` and no pinned upstream commit; unpinned resolution is the textbook cause of exactly the failure observed. The cheapest available fix was never attempted, which weakens even the availability claim.
* **The largest single category of friction was neither Noria nor semantics.** A chat interface silently stripped `<` and `>` from code blocks across six exchanges, breaking every Rust generic and every Yew `html!` macro, and producing more compile errors than Noria and Materialize combined. Docker Desktop conflicts, Homebrew collisions, CORS and a `trunk` build failure account for most of the rest.
* **Money was never a float.** The attempt used `BIGINT` minor units from the first schema onward, and said why. If any part of this thesis lists float-money as a failure mode this attempt hit, it is wrong and must be cut.

What the transcript *does* establish is two things, and they are worth having.

**First, research-artefact sustainability.** A system distributed only as a source tree pinned to a five-year-old nightly, with unlocked transitive dependencies and a C++ dependency broken by later toolchains, is undeployable regardless of its design merit. This is a claim about *artefacts*, not about *ideas*, and it argues for the engineering practice this repository adopts — pinned toolchain, locked dependencies, determinism gates, a reproducible build — rather than against Noria's design, which this thesis builds directly upon and credits accordingly.

**Second, and more on-thesis: guarantees absent from the engine reappear as unverified application code.** The transcript shows this happening in real time. Uniqueness, once the engine refused primary keys, became `transaction_id + 1` with a comment saying it keeps the key unique. Atomicity became a comment reading *"in a real banking app, you would wrap this in a SQL transaction."* A failed balance read became `Err(_) => 0, // Account not found or zero balance` — a database error silently rendered as a zero balance, which is the "miss ≠ zero" defect this thesis's absence lattice exists to make impossible, appearing spontaneously in code written to work around an engine that could not express it. No currency column, no overdraft check and no audit metadata appear in any schema in the transcript.

That second finding is an *illustration* of a failure mode, from n = 1, in an AI-assisted session by one developer with no controlled comparison. It is not a measurement, and Chapter 9 does not treat it as one. Its correct home is §1.1, as the observation that motivated the work, clearly labelled as an anecdote — which is what motivations are — with the differential defect corpus of §6.10.3 doing the evidentiary work.

**What Noria's own authors claim.** The limits this thesis actually relies on are stated in the Noria papers, not discovered in the transcript: eventual consistency only, randomized eviction, unsupported SQL keywords, and transactions and stronger consistency listed as future work [Gjengset et al., OSDI '18; Gjengset, PhD thesis]. Those are literature citations and are cited as such. Presenting them as findings of the GBS attempt would be a category error, and the earlier draft came closer to doing so than it should have.

### 11.5.6 Have the symbolic accounting boundary and the two-span diagnostics been analysed and argued?

**Both, yes — and one of them was substantially rewritten as a result.**

**Symbolic accounting across a transaction boundary** is analysed in §4.5.1 (1,857 words) with a 43-reference review in `docs/research/currency-row-analysis.md`. The analysis produced four corrections that a reviewer would otherwise have produced: the technique is a *Karr-expressible query answered in a weaker non-relational domain*, not row polymorphism (the mechanism is a Currency-graded module with Hindley–Milner unification); it is a **static analysis, not a decision procedure**; its incompleteness is **Herbrand, not arithmetic**; and `Undecided` is *forced* by a reduction from Post's Correspondence Problem rather than chosen. It also corrected a claim about Move, whose own paper states that its type system "will not ensure that the total value of all Coins in existence is preserved" — a correction that strengthens this thesis and was therefore easy to miss.

The engineering was rewritten in consequence. The solver had **no control-flow join**: it walked both arms of an `if` into one accumulator, and on an eleven-line correct transaction produced a phantom \$10 imbalance, a spurious "never consumed" on a shadowed binding, and two spurious "consumed twice". The fix is a top-preserving `Row::join` — entries both arms agree on survive, disagreements poison to `Undecided` — with branch-local scopes and linear uses taking the *maximum* across arms rather than the sum. Two over-corrections followed and are recorded: marking `?` as `MayAbort` downgraded a real \$40 hole to a warning (fixed by *dropping* abort paths, since a `txn` is sealed atomically and an abort path commits nothing — a runtime guarantee buying static precision), and gating `Violates` on straight-line provenance turned two arms that both lose \$10 into a warning (fixed by removing the gate, since a top-preserving join already guarantees a surviving decided entry was agreed by every arm). Fourteen tests in `control_flow_soundness.rs` pin the result.

**Two-span diagnostics** are analysed in §6.10.4 with a 34-reference review in `docs/research/diagnostics-evidence.md`, structured on Toulmin's claim/grounds/warrant/backing and on Haack and Wells's type-error slicing, whose central point is that an error is a *set* of program points rather than a location. The honest finding recorded there, and repeated because it constrains what may be claimed: **no controlled study compares multi-span with single-span diagnostics**, and Elm — routinely cited as the exemplar — is single-region. The thesis therefore argues the design from first principles and from the structure of the analysis (a conservation violation genuinely *has* two locations: where the money entered and where the path left it unbalanced), and does not claim empirical support that does not exist. The warrant is elided when a machine-applicable suggestion is present, on the grounds that a resolution outranks an explanation.

### 11.5.7 Have the engine defects been corrected in the repository and the thesis?

Yes, and the table below is the audit trail. "Corrected" means the defect is fixed in code with a regression test, and the thesis text that overstated the position has been rewritten rather than merely qualified.

| Defect | Repository | Thesis |
|---|---|---|
| Currency-row solver had no control-flow join (4 false errors) | `currency_rows.rs` rewritten; `control_flow_soundness.rs`, 14 tests | §4.5.1, §11.3 |
| `?` weakening downgraded a real violation | abort paths dropped, not weakened | §4.5.1 |
| `Violates` gated on straight-line provenance | gate removed | §4.5.1 |
| `from` reserved, so `fn transfer(from: …)` did not compile | seven clause-position words demoted to `ColName` | Appendix B |
| `lineage: full` did not parse | contract keys accept any keyword | Appendix B |
| `as` in a SQL projection parsed as a cast | `no_alias_depth` in the parser | Appendix B |
| Extended query protocol refused with an open design question | `extended.rs`; epoch-keyed plan cache, 11 tests | §7.4 |
| No distributed read path | `distributed.rs`, 10 tests | §7.4 |
| No cross-shard commit | `cross_shard.rs`, 12 tests | §8.6 |
| No MySQL wire | `mysql_wire.rs`, 11 tests | §7.4 |
| No TLS boundary | `tls.rs`, 17 tests | §6.12 |
| No cost-based join ordering | `join_order.rs`, 16 tests | §6.11 |
| Appendix E stage 1 had no input | `bootstrap/lexer.niles` + 14 gates | Appendix E.0, E.19 |
| Noria "unusable in production" overstated | postmortem written | **this section**, §1.1 |
| "REVs need a new engine" | — | §11.3, §11.5.2 |

The two entries with no repository column are corrections to *claims*, and they are the two most consequential, because a defect in an argument survives every test suite.
