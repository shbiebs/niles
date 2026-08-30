# 8. The Phased Research Program

The program is staged so that every phase has entry conditions, deliverables, and **pre-registered kill criteria** — conditions under which the engineering bet is declared lost and the theory revised. Phases may overlap in engineering; their evaluations are sequential. Kill criteria are stated as *shapes* rather than as invented thresholds wherever no defensible reference figure exists, because a fabricated threshold is worse than an honest one.

## 8.1 Phase 0 — Ledger Floor, Read-Model Break-Even, Kill Criteria

Two questions must survive contact with hardware before any language exists.

**(a) The ledger floor.** Can an append-only, hash-chained, durable, strictly serializable single-node ledger sustain throughput competitive with the conventional shape it replaces? Deliverable: the ledger crate plus a microbenchmark measuring sustained posting throughput and commit latency against batch size and epoch period τ.

*Reference points, correctly scoped.* The comparison baseline is a measured PostgreSQL/MySQL configuration on the same hardware, not a vendor figure. Published marketing claims for purpose-built ledgers are explicitly excluded as baselines, because they arrive without methodology, hardware specification or workload definition; where a vendor's own documentation and homepage disagree with each other on a basic parameter, that is itself a reason to treat such figures as unusable.

*Kill criterion K0a:* if the ledger write path cannot match a tuned conventional RDBMS configuration on the same hardware for the same durability guarantee, the "ledger-grade write path at no cost to the floor" premise fails, and the thesis must either narrow its claims to read-side economics or change the write-path design.

**(b) The break-even sketch.** Does a prototype partial view over the ledger beat full materialization anywhere real? Deliverable: one hand-built REV against a fully materialized twin, swept across α — including the flat end of the empirically attested range.

*Kill criterion K0b:* if no (α, budget) point shows a material resident-state advantage at acceptable latency, the partial-state premise fails *before* the theory's constants are tuned to fit. This criterion is deliberately placed first, because it is the cheapest possible refutation of the thesis's economic claim, and running it late would be a methodological error.

## 8.2 Phase 1 — Formal Model and Language Design

**Deliverables.** Chapter 3's framework complete; Theorems 4.1–4.6 proved in their stated models; λ_niles specified; the Niles surface specified (Appendix B); the IR specified (Appendix D); the executable oracle running (Appendix F — *this deliverable is complete*).

**Gate.** The proof-ladder audit of Section 3.15: every premise either proved or explicitly registered as an assumption with its threat. No performance claims exit this phase.

## 8.3 Phase 2 — Single-Node Vertical Slice

**Deliverable.** The full spine on one node — ledger, IR, Nilestream-Core, native protocol — running the conservation suite and a reduced benchmark, with the stage-0 compiler covering the declarative and transaction tiers.

**Evaluation.** Differential testing against the oracle; the first round of the S4 fault campaign; the floor re-measured through the real pipeline; and the first empirical check of the §4.2 claim that anchoring removes the need for eviction-notice propagation and anomaly-avoidance protocol.

*Kill criterion K2:* any conservation violation not attributable to a fixable implementation defect — that is, a model-level counterexample — halts the program until the theory is revised. This is the criterion the thesis most wants to be tested, because it is the one that would matter most if it fired.

## 8.4 Phase 3 — Adaptive Materialization, Optimizer, and Language Evaluation

**Deliverables.** The optimizer of Contribution 5 live, with its estimators and the offline dynamic program that grades it; the full consistency ladder served; the S1, S2, S3 and S7 experiment suites; the SQL surface and both wire protocols with the compatibility corpus; the lineage subsystem at all three modes; and the language-scope corpus — the banking products of Section 6.21 plus the non-financial domain library and the ported SQL workloads.

**Evaluation.** The phase diagram (Section 9.6) drawn from real runs, with Z reported at every point; the optimizer measured against every fixed policy and against the offline optimum; S6's enforcement comparison; the compatibility corpus green or its gaps enumerated.

This phase produces the thesis's core empirical chapters, and its templates are frozen before it begins (Section 5.9).

## 8.5 Phase 4 — Distributed Execution and Recovery

Scale-out: ledger groups partitioning the account space, each running the Phase-2 spine under established replication; read models subscribing to multiple groups with per-group frontiers joined into vector anchors; recovery drills — node loss, media loss, partition — with frontier reconstruction re-verified in the distributed setting.

The consistency ladder's predicates generalize by replacing the scalar visibility frontier with a vector one; ℓ₃ then requires a *consistent cut* across groups rather than a single scalar epoch, which is the only place the formal machinery genuinely changes shape.

## 8.6 The Cross-Shard Commit Protocol

Transactions spanning ledger groups — an FX pair across currency-partitioned groups, a transfer across account partitions — commit via an epoch-aligned two-phase protocol. Participants reserve balanced sub-transactions in their open epochs under a shared cross-shard identifier; a consensus-backed coordinator collects reservations and issues commit or abort before the participating epochs seal; sealed epochs record the outcome. The cross-shard transaction is therefore atomic *on the visibility timeline*: either all legs are present at the agreed vector anchor or none are.

Conservation extends because each leg balances per currency locally (Definition 3.7), so no interleaving of failures can strand value, and an aborted reservation leaves no postings.

**Status: built as a protocol, tested in simulation, never run over a network** (`nilestream-consensus::cross_shard`, 12 tests). Four properties are worth stating, because each answers an objection the classical protocol attracts.

*The blocking objection dissolves.* Two-phase commit's standard indictment is that a coordinator failing between prepare and decide leaves participants holding locks with no one to ask. Here the coordinator **is a ledger group**: the decision is persisted through quorum before it is sent, and the implementation refuses to send a decision that is not yet durable. A successor coordinator therefore *reads* the decision rather than re-deciding it, and the window in which the classical protocol blocks does not exist.

*Conservation is checked where — and only where — it can be.* Each fragment balances locally per currency, but a transaction can be locally balanced on every shard and still create money globally, since a shard sees only its own legs. The coordinator holds every fragment and is thus the single place the global per-currency sum can be taken; it aborts a transaction whose fragments do not sum to zero **even when every shard has voted to prepare**. That is a check no participant could have performed.

*Visibility is identical everywhere, by construction.* The commit epoch is chosen strictly above every prepared epoch on every participant, however skewed the shards. A reader at any anchor therefore sees all legs or none — the atomicity of §3.6 holding across shards without a distributed read protocol, because it is a property of where the epoch lands rather than of what the readers agree.

*A cross-shard upquery needs no coordination at all.* It reads a frozen prefix, which cannot change; the read cannot be made stale by a concurrent write, and its result can be cached indefinitely with no invalidation protocol. The distributed read path (`nilestream-core::distributed`, 10 tests) takes the *minimum* frontier across shards as the strictly-consistent read point, batches one round trip per shard, and caches by `(key, epoch)`. This is Theorem 4.1's anchoring paying a distributed dividend, and it is the clearest instance in this thesis of an immutability commitment buying something a mutable design would have had to coordinate for.

The protocol's novelty budget is deliberately small — it is two-phase commit hardened by epoch alignment and consensus-replicated participants — because the thesis's claims live elsewhere. Where a cheaper design is available it is preferred: the deterministic-execution literature shows that pre-agreeing the input order removes the need for a commit *vote* [Thomson et al., SIGMOD '12; Lu et al., PVLDB '20], and the extent to which that technique can subsume the reservation step here is an implementation question this phase settles empirically rather than a claim made in advance.

## 8.7 Fault Model and Its Boundaries

**In scope:** crash-stop failures, media loss up to the replication factor, network partitions, message loss, duplication and reordering, and the injected schedules of the S4 campaign.

**Out of scope, declared:** Byzantine replicas — the hash chain provides tamper *evidence* relative to a retained digest, not Byzantine tolerance, and a BFT profile is future work; correlated loss of all replicas, mitigated by cold-storage migration rather than by protocol; and clock attacks beyond what epoch ordering already ignores, since epochs never trust wall clocks for ordering.

**A stated asymmetry.** Integrity degrades more gracefully than availability: several out-of-scope scenarios still leave the tamper-evidence guarantee intact, because any surviving holder of a later digest can detect revision of the prefix. This asymmetry is a design value, and it is also the precise limit of what the guarantee means — it requires that someone retained a digest and that someone checks.

## 8.8 Phase 5 — Production Hardening, General and Banking

**General.** Backpressure and admission control; quota and multi-tenancy isolation; online schema and view evolution — where new views backfill by upquery *by construction*, which is one of the design's quieter dividends; observability maturation; upgrade drills gated on determinism.

**Banking.** The confidentiality profile of Sections 6.17–6.19 enabled end to end; the product portfolio exercised under load; audit rehearsals in which arbitrary historical answers are reproduced under supervision, exercising the reproduction endpoint against the regulatory expectation that a complete, time-stamped audit trail permits recreating a prior state; and operational runbooks.

**Exit condition of the program.** Every hypothesis in Section 1.6 has met its data — supported, refuted, or partial — and every refutation has a written account of what it implies for the theory. The program is complete when the claims are settled, not when the system is finished.
