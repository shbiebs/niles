# Appendix J. Adjudication: Competing Design Positions, Judged

This appendix exists because a thesis that silently absorbs every suggestion is not a thesis but a scrapbook. Each row below is a point on which two positions were held — an earlier draft's and a later one's, or a stated claim and what the evidence turned out to say. Each is decided, with the reason and the evidence, and the losing position is recorded rather than deleted.

Verdicts are of four kinds: **A** (the position stated in the author's own draft prevails), **B** (the alternative prevails), **Both wrong** (the evidence refutes both, and a third statement replaces them), and **Open** (genuinely undecided, with the experiment that would decide it).

---

## J.1 Syntax lineage: SQL-first or Rust-first?

**Position A (later draft).** Rule 1: reuse SQL syntax wherever SQL has a construct. Rule 2: fall back to Rust where SQL has none. Rule 3: invent only for what neither can express.

**Position B (earlier draft).** The inverse: Rust-first, SQL-fallback, novel-last.

**Verdict: A.** Three reasons, in increasing order of weight.

First, positioning. A language that claims to be *a replacement for SQL* and then greets a SQL user with unfamiliar spelling for `GROUP BY` has made an unforced adoption error. The population to be persuaded reads SQL.

Second, the translation theorem. The generality claim now rests on a semantics-preserving compilation of a stated SQL fragment (§9.7, Appendix H), proved by structural induction. That proof is materially easier to state, check and maintain when the target's vocabulary mirrors the source's, because the induction becomes near-syntactic for the reused fragment and the novel cases are confined to the constructs SQL does not have. Rust-first would have made every clause a translation case.

Third, the exception is what makes the rule honest. Position A's rule carries a stated exception — a SQL spelling is not reused where it would sacrifice determinism or efficiency the engine depends on, so unbounded recursive CTEs are admitted only behind a guard and `float` is not a legal money type. That exception is doing real work: it is where the language declines to inherit SQL's permissiveness, and it makes "SQL-first" a discipline rather than a capitulation.

What Position B was right about, and is retained: pipelined clause *order*, so a query reads in evaluation order; Rust's shape for the imperative tier, generics and attributes; and the rule that a novel construct never overloads an inherited keyword with changed meaning.

---

## J.2 How many named scientific contributions?

**Position A.** Four: the versioned partial-state algebra with the reconstruction theorem (SC1), the Eviction–Consistency Frontier Theorem (SC2), the complexity theory of consistency under partial materialization (SC3), and the consistency-effect calculus with the soundness theorem (SC4).

**Position B.** Six: the four above, plus an adaptive materialization calculus with competitive guarantees, plus a generality result replacing the unprovable "replaces SQL" with provable statements.

**Verdict: B, extended to seven by experiment.** SC1–SC4 stand exactly as stated in Position A. Two are added because they answer questions SC1–SC4 raise but do not close: SC2 says *where* partiality stops paying, but nothing says *what the runtime should therefore do* — that is SC5, the mode calculus over {absent, demand, full, spilled, tiered} with its rent-or-buy and file-caching bounds, and §9.3.4's measured interior optimum is direct evidence that a fixed policy is wrong. And the thesis's own H7 asserts SQL-completeness, which is not a well-formed claim without a definition — that is SC6.

A seventh was added by measurement rather than by argument. **SC7, the Bounded Reconstruction Theorem**, states that reconstruction cost is bounded by the checkpoint interval rather than by history length. It exists because §9.4.1 falsified SC3's history-independence claim twice, and the third experiment showed what mechanism restores it. It is the clearest case in this thesis of an experiment changing the theory rather than confirming it.

---

## J.3 How is SQL-completeness to be established?

**Position A.** By a mechanical SQL→Niles lowering *and* by passing an industry SQL-conformance suite.

**Position B.** By a translation-completeness theorem over a stated fragment, plus feature-coverage and differential-testing corroboration.

**Verdict: B, and the evidence is decisive rather than merely persuasive.** The conformance-suite half of Position A is not available. The only officially-blessed SQL conformance suite ever produced targets SQL-92, was frozen on 31 December 1996, and the programme that validated against it was terminated on 1 July 1997. Its own user guide states that it "would be incorrect for implementations to claim conformance … simply by virtue of correct performance of these tests." The modern alternative in common use is a *differential* tester that compares engines against one another rather than against the standard, and explicitly excludes transactional behaviour and concurrency from its scope. There is no authoritative, current, complete SQL:2016/2023 conformance corpus, and the standard's licensing obstructs building one.

This is a finding, not a quibble: a reviewer who knows this literature would have asked, and the thesis would have had no answer. The claim is restructured in §9.7 and Appendix H, with the translation theorem as the primary — and strictly stronger — argument, because it quantifies over all programs in the fragment rather than over a corpus.

---

## J.4 "RPO = 0"

**Position A.** The ledger is fully durable, RPO ≈ 0 in some passages and RPO = 0 in others.

**Position B.** Durability claims must be scoped to a failure domain.

**Verdict: Both insufficiently precise; a third statement replaces them.** The authoritative definition is a *recovery point*: how much data loss a process can tolerate. No standards document defines what RPO = 0 requires mechanically, so the requirement must be derived: zero tolerable loss means the acknowledged-durable point equals the committed point *across the failure domain in scope*. A concrete production instance makes the shape clear — one commercial database achieves storage-level RPO zero by requiring four of six storage nodes to acknowledge before confirming a transaction, and its own vendor writes candidly that even that is scope-bounded against sufficiently correlated disasters.

The thesis therefore states durability as a triple, and never as a bare number:

* **RPO = 0 against process and OS crash**, with fsync-before-acknowledge — and even this presumes the device honours flush, which consumer SSDs frequently do not, and presumes the system panics rather than retries on a flush error, since a retried `fsync` can report success after the data has already been lost.
* **RPO = 0 against single-node loss** only with synchronous quorum replication.
* **RPO bounded by replication lag** otherwise.

An unqualified "RPO = 0" for a single-node design would be a correctness error in the thesis, not a marketing flourish.

---

## J.5 The skew parameter

**Position A.** "α (higher = more skew)", used as the phase-diagram axis.

**Verdict: Position A is an error, and the correction matters.** In this literature α denotes at least three different quantities: a power-law density exponent, the Pareto *shape* parameter, and (loosely) a Zipf exponent. For the Zipf rank exponent, higher does mean more skew. For the Pareto shape parameter, higher means a *thinner* tail and *less* skew — the exact opposite. Since the thesis's headline empirical object is a phase diagram parameterized by skew, an ambiguous axis is one a reader can read backwards.

The thesis now uses **Zipf rank exponent *s***, defines it on first use, and labels the axis with its direction. Chapter 9's measured diagram is stated in those terms throughout.

---

## J.6 Sign, side, and the representation of negative balances

**Position A.** Flows are signed; stocks are magnitude-plus-side. A balance that would "go negative" is a positive magnitude that has changed side, i.e. migrated to the complementary account class. Non-negative money types therefore lose no expressiveness: they relocate negativity from the number to the account class.

**Verdict: A, and it is corroborated by production practice rather than merely internally coherent.** This was checked against a purpose-built ledger's documented data model. That system stores four separate unsigned 128-bit integers — pending and posted debits, pending and posted credits — and **has no signed balance field at all**. Net balance is computed by the application as debits − credits or credits − debits *according to the account's type*. Non-negativity is opt-in per account through flags whose very names encode the normal side (`debits_must_not_exceed_credits`, `credits_must_not_exceed_debits`).

That is Position A's design, arrived at independently, in production, by a system whose entire purpose is financial correctness. The sign convention lives in the account's type rather than in the sign of a number. It is unusual for a thesis design decision to have this quality of external corroboration, and the section should say so.

One caveat is added: the claim that overdraft is "a draw on a linked credit line" was *not* found in that system's documentation, so the thesis presents it as its own modelling choice rather than as attributed practice.

---

## J.7 Does the conservation-of-money suite establish strict serializability?

**Position A (implicit in both drafts).** The Jepsen-style bank test is the correctness centrepiece; passing it under concurrency, eviction and recovery is the headline correctness result.

**Verdict: Both drafts overclaimed, and there is a published counterexample.** In a Jepsen analysis of a distributed SQL database, "bank tests (both within a single table and between multiple tables) passed consistently" *while* a causal-reverse anomaly demonstrated that the system provided serializability but **not** strict serializability. Conservation of money is therefore strictly weaker than the top rung of the ladder, and a passing suite cannot be evidence for it.

Two consequences, both adopted. The correctness programme gains a second instrument: black-box anomaly inference over recorded histories, which infers a dependency graph in Adya's formalism and detects the cycles that define the anomalies, and which is *sound* — a reported anomaly is present in every interpretation of the observation. And the thesis gains an argument it did not previously make: that instrument's power depends on *traceability*, the property that a complete version history can be recovered by reading, for which append-only structures are the canonical case. **An append-only, hash-chained ledger is unusually amenable to black-box verification, and that is a positive argument for the architecture** (§9.2.2).

It is also worth recording what Jepsen itself says about its own epistemic status: "we can prove the presence of bugs, but not their absence."

---

## J.8 Do per-view consistency contracts compose?

**Position A.** Theorem 3.1: if a view is well-typed at effect *L* and served at a frontier satisfying φ_L, then every parent is observed at an effect no weaker than *L*; hence composition cannot violate a contract.

**Verdict: A holds as stated, but requires a caveat it did not carry, and the caveat is load-bearing.** Theorem 3.1 is about *effect demands* propagating through a dataflow, and in that scope it is correct. It must not be read as the stronger claim that per-view strict serializability composes into *global* strict serializability, because that stronger claim is false in general: linearizability is a local property — a system is linearizable if each object is — but **neither serializability nor strict serializability is local**. Composing two individually strictly-serializable views does not, without further argument, yield a strictly-serializable whole.

What rescues the design is not the effect system but the *single visibility timeline*: because every view is anchored to the same totally ordered epoch sequence, a multi-view read at a common anchor is a read of one global state, which is exactly the X-CONSIST predicate. The composition argument therefore runs through the shared timeline, not through the type system. Chapter 3 now says this explicitly, and Theorem 3.1's scope is stated as effect propagation rather than as global consistency composition. This is the kind of gap a viva finds.

---

## J.9 "Partiality pays on skew"

**Position A and B alike.** Demand materialization's advantage grows with skew.

**Verdict: Both wrong; refuted by measurement (§9.3.4).** As skew rises, full materialization touches fewer distinct keys, so *its* footprint shrinks too — measured resident-entry-epochs for full materialization fell from 17.6 M at *s* = 0.5 to 3.9 M at *s* = 1.3, while partial's stayed near 1.45 M. The ratio therefore moves **against** partial as skew rises (12:1 → 2.7:1). Skew simultaneously reduces partial's reconstruction penalty, so the two effects oppose one another.

Replacement claim, supported by the data: *partial materialization pays when memory is expensive relative to reconstruction; the optimum budget is interior; and skew determines the shape of the trade rather than its direction.*

This is the single most useful thing measurement did for this thesis, because the refuted claim was intuitive, was stated in both drafts, and would have been repeated in a defence.

---

## J.10 Is the cost of consistency history-shaped?

**Position A and B alike.** No — it is workload-shaped (fan-out × churn), not history-shaped.

**Verdict: Both wrong as stated; restored under a mechanism neither draft had (§9.4.1).** Reconstruction cost from the raw log grew linearly with history when the key space was fixed (124.5 → 8,008.9 base rows as writes grew 64×), and still grew when the key space was grown in proportion (210.4 → 3,303.7), because a Zipf-hot key keeps a roughly constant *share* of a growing traffic total. Per-key anchor indices — which the earlier draft credited with history-independence — deliver an 80–112× constant-factor reduction and no change in growth.

Per-key checkpointing restores the property: at interval C = 16, cost was flat at ≈ 8.5 base rows across a 64× increase in history, against a predicted C/2 + 1 = 9. The claim now reads: **reconstruction cost is bounded by the checkpoint interval, not by history length** — a *design obligation* rather than an emergent property, promoted to SC7.

---

## J.11 Hot-account contention

**Position A.** Hot accounts are the defining write-path difficulty; the ledger must sustain hot-account throughput.

**Verdict: A is right about the problem and the thesis cannot yet contribute evidence about it.** The prototype is single-threaded, so its flat hot-share column (§9.4.4) measures the absence of an experiment, not the absence of a problem. That is stated as a non-result.

The external evidence is strong and is adopted in place of a claim of the thesis's own. Two independent production write-ups, on different stacks, report that the shared settlement or omnibus account is a *structural* hot key in double-entry systems — every transaction touches it — and both chose to relax synchrony on the hot side rather than shard the account: one reports a per-account ceiling of 3–4 update operations per second raised to roughly 30 by 250 ms batching windows, and explicitly rejected sharding as complicating the single-balance concept; the other routes hot-account entries to an asynchronous path with a 60-second application bound and recommends that hot accounts receive only asynchronous entries.

This is unusually good support for the thesis's central architectural claim, and it is worth stating plainly: **real ledgers already run the hot leg at bounded staleness and the cold leg at strong consistency — they simply do it ad hoc and untyped.** A system that makes that split declarable and gives the staleness bound a formal meaning is addressing a documented, quantified production problem rather than a hypothetical one.

---

## J.12 Non-convexity of the achievable consistency region (H1)

**Position A.** The achievable-consistency region under partial state is non-convex: snapshot reads aligned to epoch boundaries are cheaper to reach than intermediate guarantees that are not.

**Verdict: Open.** §9.4.3 measured cost per rung and found it lands almost entirely on maintenance frequency, scaling as roughly 1/k in the staleness allowance — which is smooth, not obviously non-convex. But that experiment varied staleness, not the *alignment* of a read to an epoch boundary, which is what H1 is actually about. The hypothesis is therefore untested rather than unsupported.

The experiment that would decide it: hold the rung fixed and vary whether a multi-key read's demanded anchor coincides with a sealed epoch, measuring coordination work in each case. If H1 holds, the cost curve has a discontinuity at the boundary. The variable is stated in Chapter 1 accordingly.

---

## J.13 Naming

**Position A.** Kaskata / KaskataFlow, after an earlier Upbasin / UpbasinFlow.

**Position B.** Niles / Nilestream.

**Verdict: B, by the author's instruction.** The thesis, prototype and artifacts use **Niles** for the language and **Nilestream** for the system throughout. The renaming is mechanical and reversible: no claim, theorem or measurement depends on it.

---

## J.14 Smaller corrections carried forward

| Point | Correction | Evidence |
|---|---|---|
| Self-hosting bootstrap | A bit-identical stage-2/stage-3 build proves the compiler is a **fixed point**, not that it is trustworthy — a self-reproducing trojan *is* a fixed point. Detecting that needs diverse double-compiling with an independently sourced parent; reproducibility is its precondition, not its substitute. | The trusting-trust literature; the corresponding toolchain's own guide presents the extra stage as an optional breakage check and claims nothing more. |
| Confidentiality labels | Restricted to **static** labels. | No non-interference results exist for the dynamic-label fragment of the information-flow literature; a guarantee that cannot be proved is not claimed. |
| WebAssembly determinism | Not assumed — three named sources of implementation-dependent behaviour (NaN payloads, resource exhaustion, host functions) are closed off explicitly. | The WebAssembly specification's own statement. |
| Tamper-evidence | Detection **relative to a retained digest**, by a party who checks — not prevention. | Certificate Transparency states this outright; the analogous commercial ledger scopes its verification the same way. |
| Consistency-ladder advertising | A ladder can be advertised with five rungs and have four. | A published TLA+ specification of a commercial database's five consistency levels found two indistinguishable to a client, and bounded staleness unbounded under strongly-consistent writes. |
| TPC-derived results | Any TPC-style workload run here is unaudited and will be reported as "TPC-C-derived", never as `tpmC`. | TPC comparability and audit rules. |
| Vendor throughput figures | Excluded as baselines where methodology, isolation level and durability setting are undisclosed. | A widely-cited core-banking figure of 150,080 transactions per second discloses none of these. |

---

## J.15 What the adjudication changed

Of the positions examined: three were decided in favour of the later draft (J.1, J.6, and J.11's problem statement), three in favour of the earlier draft (J.2, J.3, J.5), four found **both** positions wrong and replaced them (J.4, J.7, J.9, J.10), one was sharpened with a caveat that changes what it may be used for (J.8), one remains open with its deciding experiment named (J.12), and one was settled by preference (J.13).

The four "both wrong" rows are the ones worth dwelling on. Two of them (J.9, J.10) were overturned by experiments run for this revision, and both had been asserted confidently in every prior draft. Neither would have survived a defence. That is the argument for building the prototype at all: not that it proved the thesis, but that it disproved parts of it early and cheaply, and in one case handed back the mechanism (checkpointing) that makes the claim true.
