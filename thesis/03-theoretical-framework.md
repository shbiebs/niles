# 3. Theoretical Framework

This chapter fixes the formal universe in which the contributions of Chapter 4 are stated and proved. Notation: Z[A] denotes Z-sets over tuples of type A; streams are epoch-indexed families; ⊞ and ⊟ are Z-set addition and negation; I and D are DBSP integration and differentiation.

## 3.1 The General Core: Bases, Committed Writes, and Derived State

**Definition 3.1 (Authoritative base).** An *authoritative base* is a triple B = (Δ, h, ⊑) where Δ = ⟨Δ₀, Δ₁, Δ₂, …⟩ is an unbounded sequence of *epoch deltas*, each Δₑ ∈ Z[Row] a finite Z-set of committed rows with positive weights only (append-only: no retraction at the base); h is the hash chain h₋₁ = genesis, hₑ = H(hₑ₋₁ ∥ canon(Δₑ)) for a collision-resistant H and a canonical serialization; and ⊑ is the prefix order on epochs. The *state at epoch e* is the integral S(e) = Σ_{i≤e} Δᵢ = I(Δ)(e).

Three clauses carry the guarantee set. *Never partial:* every committed epoch is fully durable and fully retained; there are no holes at the base, ever. *Strictly serializable:* the writes folded into Δₑ result from a commit protocol producing one serial order consistent with real time; the epoch sequence is that order, coarsened. *Tamper-evident:* hₑ commits to the entire prefix, so a holder of a later digest can detect revision of any earlier part — the guarantee, per Section 1.9, is detection relative to a retained digest, not physical immutability.

**Definition 3.2 (Commit rule).** A base carries a *commit rule* C: a decidable predicate on candidate write sets. Admission computes C and appends only if it holds. The general core requires only that C be decidable and deterministic given the prefix; specific bases instantiate it — a uniqueness constraint, a referential rule, or (Section 3.3) a per-group sum-zero rule.

**Definition 3.3 (Derived state, versioned).** A *view definition* is a DBSP circuit Q from base streams to a keyed output Z[K × V]. The *ideal view at epoch e* is V\*(e) = Q(S(e)) — total, a mathematical object rather than a data structure. A *versioned partial state* for Q is a partial function M : K ⇀ (V × Epoch), where M(k) = (v, e) asserts v = V\*(e)[k].

The ledger of Section 3.2 is Definition 3.1 with a particular commit rule and particular column types. Every result in this chapter and the next is stated at the level of Definitions 3.1–3.3 and instantiated afterwards. This is the formal content of "banking is a domain layer on a general relational core."

## 3.2 The Ledger as a Domain Instance

A **ledger** is a base whose rows are postings p = (txn, acct, cur, amt, valid, idem, …), whose commit rule is per-transaction, per-currency balance (Definition 3.5), and whose columns carry: an idempotency key with a defined admission semantics (Section 3.20); two time axes (Section 3.9); and an amount typed as `Money⟨cur⟩` in exact minor units at the currency's own scale.

The scale is per-currency and is not two. ISO 4217 assigns minor-unit exponents that include 0 (JPY, CLP, ISK), 2 (USD, EUR), and 3 (KWD, BHD, TND, OMR, JOD), and some entries have no minor unit at all. A `Money` type with a fixed scale of two is therefore *provably wrong* for a general system, which is why the scale is part of the type (Appendix B.4) and why the industrial "store everything in cents" convention is treated in this thesis as a special case rather than a rule.

## 3.3 The Double-Entry Invariant and Its Coordination Cost

**Definition 3.4 (Balanced delta).** Write w(p) for a posting's Z-set weight. A delta Δ is *balanced* iff for every transaction t and currency c: Σ { amt(p)·w(p) : p ∈ Δ, txn(p) = t, cur(p) = c } = 0.

**Definition 3.5 (Conservation).** A ledger *conserves value* iff every Δₑ is balanced. Then for all e, the signed sum of all balances per currency is invariant, changing only through postings against explicitly modeled issuance accounts.

**Lemma 3.1 (Compositionality).** Balanced deltas are closed under ⊞ and under integer weighting; hence balance of every transaction implies balance of every epoch, every prefix, and the whole ledger. *Proof:* linearity of the sum in Definition 3.4 over Z-set addition. ∎

**Proposition 3.2 (The coordination boundary).** Conservation is a sum-equality invariant and is I-confluent; the non-negativity invariant "balance ≥ 0" is not. *Justification:* Bailis et al. establish I-confluence as necessary and sufficient for invariant-preserving coordination-free execution, and classify a `>` threshold under decrement as not I-confluent while attribute equality and materialized-view update are I-confluent [PVLDB 2015]. Two independent lines confirm the same boundary from the systems side: Doppel's splittability conditions admit self-commuting operations that return nothing — blind `Add` qualifies, a threshold check does not, and a *read* of a split record forces rejoining [Narula et al., OSDI '14]; and sharded counters scale writes linearly while providing no mechanism for an aggregate threshold. ∎

This proposition governs the architecture. Conservation — the invariant Contribution 1's corollary protects — is cheap and survives concurrency and partition; no-overdraft — the invariant Contribution 4 enforces — necessarily costs coordination. The design therefore makes the coordination explicit and confines it: the authorization path runs at the top rung of the ladder (Section 3.7) while every other view is free to run cheaply, and the reservation mechanism of Section 3.19 is the classical escrow answer to exactly this constraint.

## 3.4 The Lattice of Absence

Partial state needs a semantics for *not being there*. For each key k, a materialization sits at exactly one point of the *absence lattice* 𝔸, ordered by information content:

⊥ (never-computed) ⊏ Hole(e′) ⊏ Pending(e′) ⊏ Present(v, e′)

⊥: the runtime asserts nothing about k. Hole(e′): the key was evicted; the runtime retains the *version* it held, not the value. Pending(e′): an upquery anchored at e′ is in flight; readers wait or degrade per contract. Present(v, e′): v = V\*(e′)[k], certified.

Two invariants are imposed and later discharged. **Monotone anchoring:** maintenance moves a Present entry's anchor only forward. **Honest absence:** eviction maps Present(v, e) to Hole(e), never to ⊥ — absence of *value* never masquerades as absence of *history*. Honest absence is what lets the consistency predicates of Section 3.8 be evaluated without the value being resident, and it is the structural reason the runtime never has to guess whether a hole is stale.

The lattice is also the formal answer to a question Noria answers operationally: what does an evicted entry *mean*? In Noria an evicted entry is ⊥ with an eviction notice propagated forward so that downstream operators drop updates addressed to it. Here, Hole(e) is a *positive assertion about a version*, and because the base is immutable, the pair (Hole(e), requested epoch e″) determines exactly what must be recomputed. Section 4.2 shows this turns the five partial-state anomalies into non-events.

Pointwise join and meet over keys give partial materializations a lattice structure used in the frontier constructions of Chapter 4.

## 3.5 Epochs

The epoch is the unit of visibility, versioning, hashing, and consistency. The commit protocol admits writes, validates them against the commit rule, sequences them into the open epoch, and *seals* on a boundary (time quantum τ or size bound); the sealed epoch obtains its hash, becomes durable, and only then becomes visible. There is no observable state "inside" an epoch: sealing is the atom of the global timeline.

τ trades commit latency against per-epoch amortization. It is a measured knob, never a semantic one — every theorem quantifies over arbitrary τ > 0. Epochs also discretize DBSP time, so "the view at e" is simultaneously well-defined for *every* view, which is what makes cross-view reads coherent (predicate X-CONSIST, Section 3.8).

The design lineage here is deterministic execution: Calvin and Aria establish that pre-agreeing a total order over transaction inputs makes execution a deterministic function of that order, removing the need for a distributed commit vote [Thomson et al., SIGMOD '12; Lu et al., PVLDB '20]. The epoch sequence is that agreed order; the read side inherits its determinism, which is precisely what Theorem 4.1 needs.

## 3.6 Visibility Frontiers

**Definition 3.6 (Frontiers).** At wall-clock instant t: seal(t) is the greatest sealed epoch; dur(t) ≤ seal(t) the greatest durable epoch; vis(t) ≤ dur(t) the greatest *visible* epoch; and for each view V, applied_V(t) ≤ vis(t) the greatest epoch fully applied to V's resident state. The default profile maintains dur = vis (visibility waits for durability), and each view publishes applied_V as an ordinary queryable value.

Frontiers turn staleness from folklore into arithmetic: the staleness of Present(v, e) at time t is vis(t) − e epochs, or the τ-scaled duration. Every read is answered *with its anchor*: the API returns (result, e), never a bare result.

The distinction from progress tracking is important and is a boundary of novelty rather than a claim of it. Naiad's frontier is a lower bound on what may still arrive at a location, computed from could-result-in relations over pointstamps [Murray et al., SOSP '13]. A visibility frontier over a sealed, totally ordered base is not an estimate: nothing can arrive before a sealed epoch. The machinery is simpler *because* the base is authoritative, which is one of the concrete dividends of putting the ledger inside the system rather than upstream of it.

## 3.7 The Consistency Ladder

Six rungs, each declarable per view in the `serve` clause:

- **ℓ₀ Bounded staleness(K, T):** answers anchored at e ≥ vis(t) − K epochs, and no older than T wall-clock, whichever binds first. The dual parameterization follows shipping practice [Azure Cosmos DB].
- **ℓ₁ Monotonic reads:** per session, anchors never decrease.
- **ℓ₂ Read-your-writes:** a session's reads are anchored at or after the epoch of its own last committed write.
- **ℓ₃ Epoch-consistent (snapshot):** everything ℓ₂ gives, and in addition each read — including multi-key and multi-view read sets — is anchored at a *single* epoch e ≤ vis(t).
- **ℓ₄ Serializable:** everything ℓ₃ gives, and in addition reads *and writes* are placeable in one serial order consistent with commit order. For a read-only transaction this is implied by ℓ₃ (Section 3.8), so ℓ₄ is a rung about read-write transactions.
- **ℓ₅ Ledger-consistent (strict serializable):** anchored at vis(t) exactly, reflecting every committed write that precedes the read in real time; where the read also reserves (Section 3.19), the reservation is atomic with the observation.

The ladder is deliberately not a free menu. The HAT results place ℓ₀–ℓ₂ in the highly-available or sticky-available classes and ℓ₃–ℓ₅ in the unavailable class [Bailis et al., PVLDB 2013]; a per-view contract is therefore a statement about *what a view gives up*, and the language makes that statement explicit rather than implicit. The worked assignments from Chapter 1's motivation: a dashboard declares ℓ₀; a statement as of a value date declares ℓ₃ anchored at a value-date-derived epoch; a balance-and-hold authorization check declares ℓ₅ with reservation.

## 3.8 Consistency Levels as Formal Predicates

Following the decomposition style of the consistency-model literature [Viotti & Vukolić, ACM CSUR 2016] and the predicate form of the session guarantees [Terry et al., PDIS 1994], each rung is a conjunction of independently varying predicates over a system trace.

A read event is r = (session s, keys 𝒦, **write set 𝒲**, views 𝒱, result ρ, anchor e_r, wall time t_r). The write set is what makes ℓ₄ say anything: a predicate over read-only events cannot distinguish serializability from snapshot isolation, because a single-anchor exact read is trivially placeable in a serial order after its own anchor. Read-only events have 𝒲 = ∅ and the ladder collapses at ℓ₄ for them, which is stated below rather than hidden.

- **EXACT(r):** ∀k ∈ 𝒦: ρ[k] = V\*(e_r)[k]. *(Exactness at the anchor.)*
- **BS(K,T)(r):** e_r ≥ vis(t_r) − K and time(vis(t_r)) − time(e_r) ≤ T.
- **MONO(r₁,r₂):** r₁ <_s r₂ ⟹ e_{r₁} ≤ e_{r₂}.
- **RYW(w,r):** w <_s r ⟹ epoch(w) ≤ e_r.
- **X-CONSIST(r):** a single e_r for the whole read set, across views.
- **SER:** ∃ a total order O on transactions and reads, consistent with commit order, such that every event's read set returns V\* at its predecessor prefix in O *and* every event's writes 𝒲 are ordered after that prefix.
- **RT:** O is consistent with real time; equivalently ∀r: e_r = vis(t_r).

Then: ℓ₀ = EXACT ∧ BS; ℓ₁ = ℓ₀ ∧ MONO; ℓ₂ = ℓ₁ ∧ RYW; ℓ₃ = ℓ₂ ∧ X-CONSIST; ℓ₄ = ℓ₃ ∧ SER; ℓ₅ = ℓ₄ ∧ RT.

**Each rung is the previous one conjoined with a predicate, so the rungs are nested as sets of traces: ℓ₀ ⊇ ℓ₁ ⊇ ℓ₂ ⊇ ℓ₃ ⊇ ℓ₄ ⊇ ℓ₅.** This is worth stating because it was not true of an earlier draft, which defined ℓ₃ as EXACT ∧ X-CONSIST — dropping BS, MONO and RYW — so that a "stronger" rung permitted a session whose anchors went backwards, and the ladder priced in Contribution 3 was not a ladder.

**Where ℓ₄ bites, and where it does not.** For a read-only event (𝒲 = ∅), SER is implied by EXACT ∧ X-CONSIST: place the event immediately after its own anchor e_r in the commit order and the definition is satisfied, because the event's read set is exactly V\*(e_r) by EXACT and shares one anchor by X-CONSIST. **ℓ₄ and ℓ₃ therefore coincide on read-only workloads**, and Theorem 4.3's O(contended keys) price for ℓ₄ is paid only by events with 𝒲 ≠ ∅. Chapter 9 measures ℓ₀ against ℓ₅ over read-only workloads and so measures nothing about that price; §9.12 records it as not measured rather than implying otherwise.

Two observations do a great deal of work later. First, **EXACT is a conjunct of every rung and is identical at every rung**: it says the value agrees with the ideal view *at whatever anchor was served*. It is discharged once, by the Reconstruction Theorem, independently of eviction — which is why eviction cannot damage correctness at any level. Second, everything that *differs* between rungs is anchor policy. The ladder therefore prices only anchor policy, which is what makes Contribution 3's history-independence result possible.

A note on attribution, since the seam matters: Papadimitriou's strict serializability (SSR) is defined over the history's own operation order, not external real time [JACM 1979]; the real-time formulation is the Herlihy–Wing composition [TOPLAS 1990]. The RT predicate above is the latter, stated over the visibility timeline, and the thesis uses "ledger-consistent" as its operational name to avoid conflating the two.

**A locality caveat that the design depends on.** Herlihy and Wing prove that linearizability is a *local* property — a system is linearizable if each of its objects is — and, in the same breath, that "neither serializability nor strict serializability is a local property." This matters here more than it does in most systems, because this thesis serves *per-view* consistency contracts and must not be read as claiming that composing individually strictly-serializable views yields a strictly-serializable whole. It does not, in general.

What rescues the design is not the effect system but the **single visibility timeline**. Every view is a function of the same totally ordered epoch sequence, so a multi-view read taken at one common anchor is a read of one global state — which is exactly what X-CONSIST demands, and why it is stated as a predicate over a shared anchor rather than as a per-view property. The compositionality theorem of Section 3.4.2 should therefore be read for what it proves — that *effect demands* propagate soundly through a dataflow, so a view cannot silently observe a parent at a weaker contract — and not as a global-consistency composition result. The global property comes from the shared timeline; the type system's job is to stop a program from asking for less than it needs.

## 3.9 Bitemporality, Value Dates, and Audit

Every base row carries two times: *valid time* (when the fact is true in the world — for payments, the value date) and *system time* (the epoch that recorded it). Bitemporal state is B(e_sys, t_valid): the view computed from rows with epoch ≤ e_sys and valid-time ≤ t_valid. Late-arriving and corrected data move only along the system axis — a backdated correction is a new epoch asserting a fact about an old valid time — so "what did the institution believe at s about time t" is a total function, queryable forever.

This is not a convenience feature. PSD2 Article 87 constrains the credit value date to be "no later than the business day on which the amount … is credited to the payee's payment service provider's account" and the debit value date "no earlier than the time at which the amount … is debited": a legal constraint *relating the two axes*. Interest accrual runs on the valid-time axis while audit runs on the system-time axis. A system with one clock cannot state either requirement, which is the argument for putting both in the type system (Appendix B.4) rather than in application convention.

The model aligns with the standardized one where possible: SQL:2011's application-time and system-versioned tables, with the closed-open period model and the rule that historical rows cannot be modified by users [Kulkarni & Michels, 2012]. The difference is that here system time is not a timestamp column but the epoch — a discrete, hash-committed, totally ordered coordinate — so "as of" is exact rather than approximate, and reproducibility is bit-for-bit.

Audit obligations reduce to two mechanically checkable properties: chain verification of the prefix, and *reproducibility* — any published (ρ, e) can be recomputed from the prefix at e and compared byte-for-byte. Retention in Niles is a type-level annotation on *views*, never on the base: derived state may forget; the base may not (H-F4).

## 3.10 Lineage: A Provenance Algebra for Derived State

Full traceability is a requirement, and it is obtained by construction rather than by logging. Annotate base rows with elements of a commutative semiring; positive relational operators map union and projection to `+` and join to `·`. The factorization theorem gives the key property: for any positive query and any commutative semiring, the semantics factors through polynomial provenance, so ℕ[X] over row identifiers is the most general annotation and every other semantics is recovered by a homomorphism [Green, Karvounarakis & Tannen, PODS '07].

Three consequences are used later. (i) Z-sets are the (ℤ, +, ·) instance, so the thesis's algebra is already a semiring image and lineage rides along the same circuits rather than requiring a parallel mechanism. (ii) **Upquery paths are provenance witnesses.** For each output key, the set of base slices sufficient to recompute it is exactly the support of its provenance polynomial; deriving upquery plans is therefore a provenance computation, which is the formal version of Noria's key-provenance tracing. (iii) Preservation of lineage under eviction-and-reconstruction is a corollary of factorization rather than a separate theorem: if the reconstructed value is the homomorphic image of the same polynomial, its explanation is the same explanation.

Three lineage modes are offered per view, and the choice is a contract term: `off` (anchors only), `key` (which base keys contributed), and `full` (the how-provenance polynomial, retained). Chapter 9's H-S9 measures the cost of each; ORCHESTRA is the prior art that established provenance-guided incremental maintenance and must be positioned against, not rediscovered [Green et al., VLDB '07].

## 3.11 Transition Semantics and the Reference Oracle

The system is specified as a labeled transition system 𝒮 over configurations C = (B, {M_V}, sessions) with labels: submit(w), seal(e), apply(V, e), evict(V, k), upquery(V, k, e), read(…), reserve(…), crash, recover.

The *reference oracle* 𝒪 is the same LTS implemented as simply as possible: a single-threaded interpreter that holds the entire base in memory, materializes nothing, and answers every read by folding a prefix (it is `crates/conservation-suite`, and Appendix F reproduces its semantic core, extracted from that source rather than transcribed). 𝒪 *defines* correctness: Nilestream is correct iff every observable trace of Nilestream is an observable trace of 𝒪 under the declared per-view contracts. All testing in Chapter 9 is differential testing against 𝒪; all proofs below are simulation arguments toward it.

The methodological point is worth stating plainly: a small, obviously-correct program that anyone can read is a better definition of correctness than a specification nobody checks, and it can be *executed against* the real system on every commit.

**A second instrument, and why this architecture would suit it — and it is not built.** Differential testing against 𝒪 checks *values*; it does not check *isolation*. The distinction is not academic: a published analysis of a distributed SQL database found that its conservation-of-money tests passed consistently while a causal-reverse anomaly showed the system was serializable but not strictly serializable. **The instrument that would close that gap is black-box anomaly inference in the style of Elle** — inferring an Adya-style dependency graph from client-observed histories and detecting the cycles that define G0, G1a–c, G-single and G2, including real-time and per-process edges, and *sound* in the sense that an anomaly it reports is present in every interpretation of the observation. No such checker exists in this repository: `grep -ri "adya\|g-single"` over the crates returns nothing, and §9.12 accordingly records strict serializability as **not tested** rather than as tested by conservation. Section 12 lists it as the first correctness instrument worth building, and this paragraph is a statement of what is missing rather than of what the programme does.

That instrument's power depends on *traceability* — the property that a complete version history can be recovered by reading, for which append-only structures are the canonical example. An append-only, hash-chained ledger is exactly such a structure. This is a genuine and previously unstated argument for the architecture: **the design is unusually amenable to black-box verification, precisely because it never overwrites.** A mutable-state store must be coaxed into revealing its version history; this one is its version history.

## 3.12 The Proof Ladder

The results are arranged so that each discharges the premises of the next.

- **P1** Hash-chain integrity and append-only base ⇒ prefixes are identified by their digests and revisions are detectable.
- **P2** ⇒ S(e) is well-defined and immutable per e.
- **P3** ⇒ *Upquery purity*: reconstruction is a pure function of (Q, prefix ≤ e, k).
- **P4** ⇒ **Reconstruction Theorem** (Thm 4.1).
- **P5** ⇒ the EXACT conjunct of every rung's predicate.
- **P6** Per-rung anchor-policy implementations are correct (the consistency-and-invariant protocol, Section 3.13).
- **P7** ⇒ end-to-end per-view contracts.
- **P8** ⇒ **Niles soundness** (Thm 4.4) transfers the guarantees to programs.
- **P9** The optimizer's mode choices never weaken P7 — materialization mode is *cost*, not *semantics* (Thm 4.5's safety clause).

## 3.13 The Consistency-and-Invariant Protocol

The protocol is the operational content of P6, and is stated here because Chapter 4's theorems assume it.

**Write path.** admit → validate(C, idempotency, authorization) → sequence into open epoch → seal(τ or size) → hash → fsync (and quorum, in replicated profiles) → publish vis. Reservations (Section 3.19) are ordinary appended rows admitted under the same rule.

**Maintenance.** For each view, apply sealed epochs in order through the incrementalized circuit, touching only resident keys, advancing anchors monotonically. Because the base is immutable, application is a pure function of (resident state, Δₑ) and may proceed concurrently with reads at older anchors.

**Read path.** Given (view, key, contract, session): compute the *required anchor* a from the contract and session watermarks (ℓ₀: any e ≥ vis − K; ℓ₂: ≥ session's last write epoch; ℓ₃: the read set's chosen e; ℓ₅: vis(t)). If the slot is Present(v, e) with e ≥ a, serve (v, e). If Present with e < a, either advance (apply pending epochs for that key) or upquery at a, per the optimizer's latency policy. If Hole or ⊥, upquery at a. If Pending(a′) with a′ ≥ a, join the wait.

**Upquery.** Evaluate the circuit in pull mode along the provenance-derived paths against the *frozen prefix at a*. Fill the slot as Present(value, a). Because the prefix is immutable, no update can interleave with the reconstruction in a way that changes its result; concurrent epoch application to the same key resolves by anchor comparison, and the higher anchor wins.

**Eviction.** Present(v, e) ↦ Hole(e), at any time, for any key not pinned by contract or by an in-flight snapshot. No propagation of eviction notices downstream is required, because a downstream hole is filled by upquery at *its* required anchor, not by a message from upstream.

That last sentence is the protocol's most consequential simplification, and Section 4.2 proves it sound.

## 3.14 The Cost Law for Consistency Rungs

**Cost model.** Charge 1 unit per resident entry-epoch (memory), c_u per upquery path evaluation of width w (base slices touched), c_a per resident-key delta application, and — following the delayed-hits formulation — account for upquery *latency* through Z, the ratio of upquery service time to mean inter-arrival time for the key range [Atre et al., SIGCOMM '20].

**Cost law (stated here, proved as Contribution 3).** For a workload W with popularity distribution π (Zipf α), working set, read/write mix, and memory budget m:

C(ℓ, W, m) = C_base(W, m) + Φ(ℓ) · U(W, m, Z)

where U is the reconstruction/coordination term determined by miss rate under π and m and by Z, and Φ is a rung multiplier: Φ(ℓ₀…ℓ₂) = O(1) (anchor bookkeeping only); Φ(ℓ₃) = O(1) plus snapshot pinning memory; Φ(ℓ₄) = O(contention) *for read-write transactions, and O(1) for read-only ones, where ℓ₄ coincides with ℓ₃*; Φ(ℓ₅) = Θ(freshness), since every read must observe vis(t). **No term depends on the base length n** — history enters only through per-key update counts, which is a workload property. That is the claim H-S3 operationalizes.

## 3.15 The Eviction–Consistency Frontier (Statement) and Verification Status

Stated here in framework vocabulary and proved as Contribution 2: *for any partially stateful implementation serving reads at ℓ₅, a read touching a Hole must complete an upquery anchored at vis(t_r), after the seal and application of all real-time-preceding writes to its slice; consequently, beyond a threshold on miss rate determined by write rate, slice width and Z, the expected cost of ℓ₅ serving under partiality is bounded below by the cost of full materialization.*

**Verification status — what is proved, tested, or assumed.** Honesty here is a deliverable.

| Result | Status |
|---|---|
| P1–P3 | Proved from the definitions; hash-chain security reduces to collision resistance of H (assumed), and the guarantee is detection relative to a retained digest. |
| Thm 4.1 (reconstruction) + conservation corollary | Proved on paper **for Q_lin**, the linear keyed fragment the runtime executes (Definition 4.1.1); mirrored as executable property tests against 𝒪. The non-linear case is Open case 4.1.α and is a conjecture, not a theorem. |
| Thm 4.2 (frontier) | The theorem — strictness forces a miss to wait for the base — is proved from the ℓ₅ predicate. Its two corollaries are arithmetic consequences of the cost model of §3.14, including the condition under which a break-even exists at all; **no lower bound over eviction policies is claimed**, and the "impossibility" wording of an earlier draft is withdrawn. Explicit constants only in special cases. The Ω(kZ) competitive lower bound for delayed hits is *attributed* to work cited in Atre et al. and is used as corroboration, not as this thesis's result. |
| Thm 4.3 (rung pricing) | Upper bounds constructive; lower bounds proved in the restricted cost model stated in §4.4, which assumes anchor-indexed access to per-key deltas. |
| Thm 4.4 (Niles soundness) | Clauses (1)–(3) proved for λ_niles by progress and preservation. Transfer to the LTS is **Lemma 4.4.α, `[sketch]`** — the crash/recover and eviction transitions are argued, not proved. Clause (4) is **conditional on P6**, which is specified and not proved. λ_niles idealizes the implemented language; the gap is a declared threat and is attacked by the H-S4 campaign. |
| Thm 4.5 (optimizer) | Safety and hardness proved. The greedy (1 − 1/e) recovery needs *two* hypotheses — fixed per-range miss rates **and** independent range benefits, which together make the objective additive — and is stated with both. The optimizer itself is **specified and not built** (C5, H-S7), so no competitive ratio is measured. |
| Thm 4.6 (generality) | Relational completeness and the SQL-fragment translation are constructive; fixpoint completeness is by reduction to Immerman–Vardi, whose ordering hypothesis the epoch order supplies. |
| Thm 3.7 (bounded reconstruction) | (i) proved for every key and anchor; (ii) is an expectation over anchors uniform between checkpoints, **not** a bound at every anchor — the worst case is C. Checkpoint lookup, O(log(n/C)), is outside the counted-work unit and inside the wall-clock figures. |
| Mechanization | Not done. A Lean or Coq development of P4 and Thm 4.4 is future work (Chapter 12), scoped but not claimed. Note that DBSP's own mathematics has been mechanized in Lean, which lowers the cost of that step. |
| Empirical validation | **Partial.** §§9.1–9.4 and §9.13–§9.14 report measurements taken; §§9.5–9.12 are protocol and prediction, and each cell says which. The row this replaces read "None yet" and contradicted the chapter it pointed at. |

The last row of that table is the aggregate, and it is generated from the same file §1.9.1's
table is, so the two cannot disagree:

<!-- BEGIN:status-row thesis/status.toml#statusrow -->

*Generated from `thesis/status.toml`. Do not edit by hand.*

| **All claims** | 6 proved, 8 measured or partly measured, 1 refuted, 2 not measured | `thesis/status.toml`, rendered into §1.9 |

<!-- END:status-row -->

## 3.16 Establishment and Formalization of the Foundational Hypotheses

**H-F1 (unboundedness).** Formally: the source is a stream Δ : ℕ → Z[Row] with no computable bound on Σ|Δᵢ|. The design consequence — no component may be admitted whose correctness or resident-state requirement depends on total input — is imposed on every part of Nilestream and audited. The *mismatch catalog* is the argument's substance: (i) a finite-first engine that destroys the changelog by in-place update must reconstruct it downstream by CDC, paying twice for information it had; (ii) audit that retention gives exactly must be approximated by triggers and shadow tables; (iii) absent anchors, applications implement version fencing by hand, and each hand-rolled fence is an anomaly class; (iv) unbounded state in stream operators must be bounded by windows chosen for engineering rather than semantic reasons, which converts a correctness question into a configuration question. Each entry names the aligned primitive that dissolves it.

**H-F2 (duality).** On epoch-indexed streams, D and I are mutually inverse [Budiu et al., Thm 2.20]. The thesis's contribution is the *asymmetry* that follows:

**Lemma 3.3 (Information asymmetry).** Let 𝔖 be the set of finite histories and let ι : 𝔖 → State send a history to its integral at its final epoch. Then ι is not injective, whereas the map sending a retained history to the family ⟨S(e)⟩_{e≤n} together with the order and per-row provenance is injective. *Proof:* two distinct histories (a single posting of +5, versus +7 followed by −2) share an integral, so ι is not injective; conversely a retained history determines every prefix integral by Definition 3.1 and determines the order by indexing, so the map is injective by construction. ∎

Hence "the table is merely the integral of the stream": the integral is a lossy summary of the stream, and it is the stream that is primitive. CQL is cited for the older architectural correspondence — relations as time-varying mappings, with three operator classes bridging streams and relations — with the explicit caveat that CQL states a reduction rather than a duality theorem; Sax et al. are cited for the systems-level duality [BIRTE '18].

**H-F3 (alignment).** Established argumentatively via the catalog above, with a comparative audit (Section 9.7) whose counting rules are fixed in advance, and with the generality half discharged by proof: Theorem 4.6 shows the stream-first design retains full relational expressive power, so alignment costs nothing in what can be asked. The strongest opposing position — that SQL needs only time-varying relations, event-time semantics and a few materialization keywords [Begoli et al., SIGMOD '19] — is engaged directly in Section 6.10 rather than ignored.

**H-F4 (the aligned shape).** The reconstruction-equivalence theorem (Thm 4.1) shows that with a fully retained base, the pair (immutable base, *partial* derived state) is observationally equivalent to (immutable base, *total* derived state) under all declared contracts. The converse direction is what makes retention *necessary* rather than merely sufficient:

**Proposition 3.4 (Necessity of retention).** If the base is truncated below epoch e₀, then for any view whose provenance support includes rows before e₀, there exists an eviction schedule after which no procedure can restore the pre-eviction value. *Proof:* by P3 reconstruction is a function of the retained prefix; a truncated prefix is a different function's domain; choose a key whose polynomial support includes a truncated row and evict it. ∎ Compaction and snapshotting are therefore *optimizations that must preserve reconstructibility*, not licence to forget.

The architectural precedents — event sourcing and CQRS, the Lambda/Kappa debate, Datomic's immutable model, Lucene's write-once segments with tombstones and background merging, Trillian's log-backed map — are treated as evidence of practicality, not of correctness; correctness is the theorem's job.

## 3.17 Immutability and Data-Race Freedom

Immutability is also a concurrency theorem. Sealed epochs and their integrals are immutable values, so every read of prefix data is race-free without synchronization, and the only contended objects in the entire system are (i) the open epoch's admission queue and (ii) each view's resident-state map.

Formally, the happens-before order generated by seal events makes every access to sealed data a read of an immutable location; hence no data race can involve the base, and the residual proof obligations are two lock disciplines — small enough for a bounded model check, **which has not been done**: there is no `loom` or `shuttle` dependency in either workspace and no sanitizer job in the build gate. What *is* discharged, and tested, is the type-level half: the workspace contains **no `unsafe` block at all** (`unsafe` is a reserved word in Niles and appears in the Rust sources only in that registry and in the tests that check for its absence), and sealed data is reachable only through shared references, so the absence of `&mut` to prefix data is a compile-time property rather than a convention. Rust's guarantee here is the affine-ownership one formalized by RustBelt, whose central invariant is that "aliasing and mutation cannot occur simultaneously on any given location" [Jung et al., POPL '18]. Section 5.8 gives the verification plan; Section 9.10 the acceptance criteria.

## 3.18 Multi-Currency, FX, and Atomic Cross-Currency Conservation

Conservation is per-currency (Definition 3.4 quantifies over c), so currencies are independent conserved quantities. An FX conversion is therefore *two* balanced legs joined atomically.

**Definition 3.7 (FX-atomic).** A transaction touching currencies A ≠ B is well-formed iff each currency's postings balance *separately* within it.

**Lemma 3.5.** Under Definition 3.7, no interleaving, eviction, reconstruction or crash-recovery can observe a state in which value has crossed currencies; cross-currency exposure exists only as explicitly modeled positions in FX-position accounts. *Proof sketch:* per-currency balance is compositional (Lemma 3.1), epochs are atomic visibility units (Section 3.5), and Corollary 4.1.1 extends conservation across eviction. ∎

The industrial precedent is exact: TigerBeetle partitions accounts by ledger, typically one per currency, permits direct transfers only within a ledger, and performs FX through atomically *linked* transfers. The type-level enforcement — `Money⟨A⟩ + Money⟨B⟩` is not typeable, and conversion requires an `fx` term carrying both legs — is Contribution 4. The formal warrant that a type system can carry this weight is Kennedy's parametricity result for units of measure, whose striking corollary is that dimension-polymorphic typing is strong enough to *rule out entire functions* — a fully polymorphic square root over dimensioned quantities cannot return a non-zero result for any argument [Kennedy, POPL '97]. Currency indices are the same machinery pointed at money.

## 3.19 Holds, Available Balance, and Escrow as Ledger Facts

The available-balance requirement is where the theory meets the regulator, and it is modeled without a single mutable cell.

**Definition 3.8 (Reservation).** A *hold* is an ordinary appended base row h = (txn, acct, cur, amount, expiry, …) marked pending. It is resolved by *appending* a resolution row: post (fully or partially, with the remainder released), void, or expire. Nothing is ever updated.

Two views follow immediately, and they are the two the regulator distinguishes: the **ledger balance** is the sum of settled postings; the **available balance** is the ledger balance minus unresolved holds. Both are REVs over the same base, differing only in which rows their circuits admit — which is exactly the FDIC's distinction between a balance computed "based only on transactions settled during the relevant period" and one that accounts for "authorized (but not settled) transactions the financial institution is obligated to pay." The APSN hazard is then diagnosable in the model rather than emergent: it is what happens when the authorization decision reads one view and the settlement decision effectively reads another at a different anchor. In Niles, the authorization path declares ℓ₅ *on the available-balance view*, so the discrepancy has a name, a contract, and a test.

**Proposition 3.6 (Holds are escrow).** The pending/resolved structure of Definition 3.8 implements the escrow method: the pair (posted, pending) per account maintains exactly the bounds O'Neil's INF/VAL/SUP triple maintains — the worst-case value of the balance given that some in-flight reservations post and others void — so an authorization may be admitted without a global lock precisely when it cannot violate the non-negativity condition regardless of how outstanding reservations resolve [O'Neil, TODS 1986]. *Consequence:* the coordination that Proposition 3.2 proves unavoidable for non-negativity is confined to the reservation step, and is paid per authorization rather than per read. TigerBeetle's `debits_pending`/`credits_pending` fields are this construction in production.

**Hot accounts.** Because account popularity is Pareto-distributed, a few accounts absorb a disproportionate share of authorizations. Three mitigations are available and all are bounded by Proposition 3.2. (i) *Conservation-only fast path:* postings that merely move value and carry no threshold are I-confluent and may be admitted with maximum concurrency. (ii) *Splitting:* per-shard slices of a hot account's reservation capacity, in the demarcation-protocol style — each shard may authorize within its granted range without messages, and boundary changes require bilateral exchange [Barbará & Garcia-Molina, VLDB J. 1994]. (iii) *Rejoin on read:* a read of a split quantity forces reconciliation, exactly as Doppel requires, so the split is invisible to correctness and visible only in latency. Section 9.13 measures which of the three the workload actually needs; the theory's contribution is to say in advance which are sound.

## 3.20 Checkpoints and the Bounded Reconstruction Theorem

Sections 3.10 and 3.14 price reconstruction as proportional to a key's own update count, which is a workload property. Experiment §9.4.1 shows that this is not by itself enough to make reconstruction cost independent of history, and the reason is a property of skew rather than of the design: under a Zipf access distribution, a hot key retains a roughly constant *share* of traffic, so its absolute update count grows with total traffic even when new keys are added at a proportional rate. Folding a hot key from genesis therefore costs more as the log ages, whatever happens to the key space.

The mechanism that closes the gap is a per-key checkpoint.

**Definition 3.9 (Checkpoint).** For a view V and key k, a *checkpoint* is a pair (e, v) with v = V\*(e)[k], derived and stored every C postings on k. Checkpoints are **derived state**: each is recomputable from the base by definition, so their existence does not weaken immutability, retention or auditability, and their loss costs performance rather than correctness.

**Definition 3.10 (Checkpointed reconstruction).** ρ\*_{k,a} evaluates k at anchor a by taking the newest checkpoint (e_c, v_c) with e_c ≤ a and folding only the postings on k in (e_c, a].

**Theorem 3.7 (Bounded Reconstruction).** Under Definitions 3.9–3.10: (i) for every key k and every anchor a, ρ\*_{k,a} = ρ_{k,a}, so checkpointing changes cost and not value; and (ii) for a key k and an anchor a drawn uniformly from the C epochs following k's last checkpoint, the *expected* number of base rows read is at most C/2 + 1, and the worst case is C. Neither depends on the length of the base.

The quantifiers in (ii) are not the quantifiers in (i) and the difference matters: C/2 + 1 is an average over anchors, not a bound holding at every anchor, and an anchor immediately before the next checkpoint reads C rows. Two costs the statement does not charge are named here rather than absorbed: locating the checkpoint is O(log(n/C)) index steps, which the counted-work unit of Section 3.14 does not count because it counts base rows, and the wall-clock figures of §9.13.1 include it.

*Proof.* (i) The checkpoint is by definition the value of the ideal view at e_c, and the postings in (e_c, a] are exactly the deltas separating e_c from a; summing them reproduces V\*(a)[k], which by Theorem 4.1 is what unbounded reconstruction returns. (ii) Between consecutive checkpoints a key accumulates exactly C postings, so an anchor falling uniformly in that interval has expectation C/2 postings behind the newest checkpoint, plus one row to read the checkpoint itself. Neither quantity mentions the base length. ∎

**Corollary 3.7.1.** Every result that assumed reconstruction cost was a workload property — the cost law of Section 3.14, and the rung pricing of Section 4.4 — holds as stated *when checkpointing is in force*, and does not hold without it for skewed workloads. The interval C is therefore not a tuning knob but a term in the theory: it is the constant in the bound, and Section 6.16 makes it a declarable property of a view rather than an engine default.

The empirical confirmation is §9.4.1: at C = 16 the measured cost was flat at ≈ 8.5 base rows across a 64× increase in history, against a predicted 9.

## 3.21 Idempotency as an Admission-Time Invariant

An idempotency key is a first-class column with a defined semantics rather than an application convention. Admission is a function of (key, canonical request fingerprint): a first submission is admitted and its outcome recorded; a repeat with the same fingerprint returns the recorded outcome without a new append; a repeat with a *different* fingerprint is rejected as a conflict. Stripe's published behaviour is the reference for this design — results are saved for the first request "regardless of whether it succeeds or fails," parameters are compared and mismatches error, and results are saved "only after the execution of an endpoint begins."

One difference is deliberate and is worth stating because it is a real tension. Stripe prunes keys after at least 24 hours, and reuse after pruning generates a new request. A ledger's retention is unbounded, so this thesis makes the *deduplication window* an explicit, typed retention parameter of the base rather than an operational default: the guarantee is "no double-apply within the declared window," and the window is part of the contract a caller can read. Claiming unbounded idempotency for free would be false; leaving it undeclared is what makes it dangerous.
