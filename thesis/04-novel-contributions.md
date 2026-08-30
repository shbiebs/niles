# 4. Novel Scientific Contributions

This chapter states and proves the seven named results. Six were derived; the seventh (SC7) was produced by an experiment that refuted a claim the others depended on. Each is presented as: the object, the theorem, the proof (with stated model restrictions), what it buys the system, and — because several sit close to existing work — an explicit statement of what is new relative to the nearest prior art.

## 4.1 The Reconstructible Epoch-Anchored View (REV)

**Definition 4.1 (REV).** A *reconstructible epoch-anchored view* is a tuple R = (Q, M, κ) where:

- Q is a DBSP circuit over an authoritative base (Definition 3.1) with keyed output in Z[K × V], annotated at plan time with *upquery paths* — for each output key, the provenance support sufficient to recompute it (Section 3.10);
- M : K → 𝔸 assigns each key a point of the absence lattice, subject to the **certification invariant** M(k) = Present(v, e) ⟹ v = V\*(e)[k], and to monotone anchoring and honest absence;
- κ is the *serve contract*: consistency rung, freshness bound (K, T), materialization mode (Section 4.6), memory budget share, retention, lineage mode, and confidentiality.

The REV supports six operations: **read**(k, session) per κ; **apply**(Δₑ) folding a sealed epoch into resident entries; **evict**(k) mapping Present(v, e) ↦ Hole(e); **upquery**(k, a) mapping Hole/⊥ ↦ Present(Q(S(a))[k], a); **version**(k) returning the anchor without the value; and **explain**(k) returning the lineage of the current entry at its lineage mode.

The REV is the thesis's central object because it is the smallest structure carrying simultaneously (i) incremental maintenance, via Q's DBSP incrementalization; (ii) cache economics, via M's partiality; (iii) provable anchoring, via epochs and the certification invariant; and (iv) declared behaviour, via κ, so that the runtime's freedom to choose mechanisms is bounded by a statement the compiler has checked. Everything a bank calls a balance, an available balance, a statement, a position, a risk rollup or a fraud feature is a REV; so is every index, and so is every OLAP aggregate.

## 4.2 Contribution 1 — The Versioned Partial-State Algebra and the Reconstruction Theorem

**The algebra.** Let 𝒱 = K ⇀ (V × E) be versioned partial maps. Define: restriction M|_κ; join M₁ ⊔ M₂ (defined when overlapping keys agree at equal anchors, otherwise taking the later certified entry); anchor-advance α_e applying deltas in (e_old, e] per resident key; eviction ε_κ; and reconstruction ρ_{κ,a} filling κ from Q(S(a)).

The algebra is DBSP-compatible: α_e is implemented by the incrementalized circuit Q^Δ restricted to resident keys, which is well-defined because incrementalization is compositional (the chain rule) and linear operators commute with restriction; for non-linear operators the circuit's upquery paths supply the finite input slices sufficient to recompute an output key, and these exist for every Niles-expressible query by structural induction on the circuit — the same construction the provenance literature gives as the support of the how-provenance polynomial.

**Theorem 4.1 (Epoch-Anchored Reconstruction).** For every REV with circuit Q over base B, every key k, and every epoch a:

ρ_{k,a}(ε_k(M))(k) = (Q(S(a))[k], a) = the entry a never-evicting execution anchored at a holds for k.

Hence evict-then-reconstruct is observationally identical to never-evict, at every anchor, for every key, under every interleaving of apply, evict and upquery that respects monotone anchoring and honest absence.

*Proof.* By P1–P3 of the proof ladder. (1) S(a) is a pure value: the base is append-only and sealed epochs are immutable, so S(a) = Σ_{i≤a} Δᵢ is determined by a alone, and no interleaving affects it. (2) ρ_{k,a} evaluates the upquery paths against S(a); Q is a function and S(a) is fixed, so the result is determined by (Q, a, k) — reconstruction is pure. (3) A never-evicting execution's entry at anchor a is Q(S(a))[k] by the certification invariant, maintained inductively by apply: the base case is trivial, and the step follows because α advances a certified entry by exactly D(Q(S))(e′) for each e′ in the interval, with I ∘ D the identity (F2). (4) The two sides are therefore the same value with the same anchor. Interleaving-independence: evict and upquery touch only M, never B; apply is per-key monotone; honest absence preserves the anchor across eviction so ρ's target is well-defined. ∎

**The anomalies, discharged by construction.** Partial-state dataflow over a *mutable* base must exclude five anomalies by protocol — double application, skipped deltas, upquery races, lost deltas, and upquery deadlock. Each is a consequence of reconstructing against a moving target, and each is dissolved here rather than defended against:

| Anomaly | Why it cannot arise |
|---|---|
| Double application | An entry's value is certified *at an anchor*; applying a delta already included in S(a) would move the anchor backwards, which monotone anchoring forbids. Idempotence is a consequence of anchoring, not of message deduplication. |
| Skipped deltas | Reconstruction is anchored: ρ_{k,a} recomputes from the prefix at a, so any delta ≤ a is included by definition, whether or not it was ever seen by the resident state. |
| Upquery races | Upqueries read a *frozen* prefix. Two concurrent reconstructions at anchors a₁ ≤ a₂ produce mutually consistent certified entries, and the higher anchor wins by monotone anchoring. |
| Lost deltas | A delta addressed to an evicted key may be discarded with no downstream notification, because the key's future value will be reconstructed from the base rather than from the delta stream. This is what removes the need for eviction notices to propagate. |
| Upquery deadlock | Reconstruction is a pull-mode evaluation over an immutable prefix; it never waits on the update path, so there is no cycle to deadlock on. |

This table is, in practice, the strongest engineering argument in the thesis: an immutable, totally ordered base does not merely make the correctness proof possible, it removes an entire protocol.

**Corollary 4.1.1 (Conservation across eviction).** Let Q_bal be the per-(account, currency) balance view. For all epochs a and all evict/reconstruct schedules σ, the sum over accounts of reconstructed balances at anchor a equals the sum over accounts of never-evicted balances at anchor a, and both are zero per currency when issuance accounts are included. *Eviction followed by reconstruction can never create or destroy money.* *Proof:* Theorem 4.1 gives pointwise equality with V\*(a); conservation holds of V\*(a) by Lemma 3.1; pointwise-equal families have equal sums. ∎

**Corollary 4.1.2 (Lineage preservation).** Under lineage mode `full`, the reconstructed entry carries the same provenance polynomial as the never-evicted entry. *Proof:* both are the image under the same semiring homomorphism of the same polynomial over the same prefix, by the factorization theorem [Green et al., PODS '07]. ∎

**What is new.** The mechanism (materialize on read, evict, upquery) is Noria's, and is credited as such. The novelty is the *anchoring*: entries carry versions, upqueries name an epoch, and the base is immutable — which converts an eventual-consistency system with five protocol hazards into a system whose exactness conjunct is a theorem at every consistency level. Corollary 4.1.1 is the machine-relevant safety property that partial-state dataflow could not previously state, let alone prove. Corollary 4.1.2's route through semiring factorization is deliberately cheap: it inherits a 2007 result rather than re-proving one.

## 4.3 Contribution 2 — The Eviction–Consistency Frontier Theorem

**Setting.** Cost model of Section 3.14. A workload W has popularity π (Zipf α), read fraction, write rate λ, and delayed-hit ratio Z. Full materialization of a view with key domain K costs memory |K| plus application on every written key; a partial policy with budget m < |K| has stationary miss rate miss(π, m).

**Theorem 4.2 (Eviction–Consistency Frontier).**

*(i) Strictness forces synchrony.* Any implementation serving reads at ℓ₅ must, on a read of key k whose slot is Hole or ⊥, block until an upquery anchored at vis(t_r) completes, where vis(t_r) includes every write that precedes the read in real time. No anchor below vis(t_r) may be served.

*(ii) The frontier.* The expected per-operation cost of ℓ₅ partial serving is

C_part = C_hot + miss(π, m) · c_u · w · (1 + Θ(Z))

against full materialization's C_full = C_apply(λ, |K|) + C_mem(|K|). For every (c_u, w, λ, Z) there is a threshold miss\* such that miss(π, m) ≥ miss\* implies C_part ≥ C_full.

*(iii) Impossibility form.* For flat popularity (α → 0) and budget m = o(|K|), the miss rate tends to 1 − m/|K| and C_part exceeds C_full for all budgets below the threshold: under strict serializability with flat skew, partial materialization is provably no cheaper than full materialization, and no policy escapes, because the requirement is information-theoretic — the reader must observe writes for which it holds no resident trace, and the only sources are the base (pay c_u·w·(1+Θ(Z))) or residency (pay C_full).

*Proof.* (i) From the ℓ₅ predicate of Section 3.8: RT forces e_r = vis(t_r), and EXACT forces agreement with V\*(e_r). A Hole holds no value at any anchor — honest absence retains only a version — so any served value must be computed against S(vis(t_r)), which requires that prefix sealed and durable. Serving an anchor below vis(t_r) admits a real-time-ordering witness (a write committing before the read in real time and not observed), violating RT. (ii) Linearity of expectation over the read stream under a stationary policy, with the delayed-hit term accounting for requests that arrive while a reconstruction is in flight. (iii) Substituting the uniform miss rate and taking budgets below threshold gives strict inequality. The no-policy-exit clause is a competitive-analysis argument: any online policy is a function of observed history, and under uniform popularity every non-resident key is equally likely next, so no predictor beats residency mass — the standard paging lower bound against an offline adversary [Sleator & Tarjan 1985; Fiat et al. 1991], transported to this cost model. ∎

**Corroboration from the delayed-hits literature, correctly attributed.** Atre et al. establish that when a fetch takes Z inter-arrival times, hit rate is the wrong objective and even Belady's offline-optimal policy is not latency-optimal; they also report a competitive lower bound of Ω(kZ) for deterministic online algorithms, attributing it to parallel work. That bound has exactly the shape of clause (iii) — as reconstruction latency grows relative to arrival rate, the advantage of holding less state collapses — and it is cited here as independent corroboration of the frontier's existence, not as a result of this thesis.

**Positioning against the nearest prior break-even claim.** ORCHESTRA reported that provenance-guided incremental deletion beats full recomputation "even when deleting up to approximately 80% of the instance" [Green et al., VLDB '07]. That is a break-even of the same *kind* and must be engaged. Three differences make Theorem 4.2 a distinct result rather than a restatement: (a) ORCHESTRA's crossover concerns *update volume* under total materialization, whereas the frontier here concerns *residency budget and consistency level* under partial materialization; (b) ORCHESTRA has no consistency dimension — nothing in it changes when the demanded rung rises, whereas Φ(ℓ) is the frontier's most important axis; and (c) the frontier includes reconstruction *latency* through Z, which a batch update-exchange setting does not face. The honest summary: ORCHESTRA established that provenance-guided incremental maintenance has a measurable break-even against recomputation; this thesis establishes where *partial* materialization has a break-even against *full* materialization as a function of consistency, and proves an impossibility region.

**How the theorem is used.** Below the frontier (skew, tolerant contracts, modest write fan-in) partiality wins; at it, the regimes price equally; beyond it (flat skew, ℓ₅ everywhere, high Z) the honest design is full materialization — and Nilestream's optimizer implements exactly this decision per view and key range (Contribution 5), which makes the theorem an operating policy rather than only a boundary marker. The PACELC analogy is deliberate: as PACELC says "else, latency versus consistency," this theorem says "under eviction, consistency versus memory — choose which you pay."

## 4.4 Contribution 3 — A Complexity Theory of Consistency under Partial Materialization

**Theorem 4.3 (Rung pricing; upper bounds constructive).** In the cost model of Section 3.14:

- **ℓ₀ BS(K,T):** O(1) anchor check per read plus miss(π,m)·c_u·w; maintenance may batch up to K epochs, amortizing application by a factor of K. No term in n.
- **ℓ₁/ℓ₂ (session rungs):** add O(1) per-session watermark state and a comparison per read; upquery cost unchanged. No term in n.
- **ℓ₃ SNAP:** add snapshot pinning — upqueries anchored at the read's single epoch, and resident versions retained across the pin window p, giving memory O(m · p̂) for mean pinned span p̂. No term in n.
- **ℓ₄ SER:** add validation of read-set order, O(contended keys) per transaction under optimistic validation over anchors. No term in n.
- **ℓ₅ Ledger-consistent:** add the freshness term of Theorem 4.2 — every miss waits for seal and application to its slice, adding expected τ/2 plus application backlog, with the delayed-hit multiplier. No term in n.

**Theorem 4.3′ (History-independence, with matching lower bounds in the restricted model).** Assume upquery paths of bounded width w and anchor-indexed access to per-key delta positions. Then (a) every rung's marginal per-operation cost is bounded by a function of (π, read/write mix, λ, m, τ, w, Z) alone — *the price of consistency is workload-shaped, not history-shaped*; and (b) any implementation must pay Ω(1) anchor bookkeeping at ℓ₁ and above, Ω(miss · w) base touches at every rung, and Ω(freshness) at ℓ₅.

*Proof.* Upper bounds are exhibited by the algorithms of Appendix D: per-key anchor indices (epoch-skip structures over the key's delta positions) give amortized O(1) locate cost, and each rung's mechanism adds exactly the structure listed; inspection shows no dependence on n, since history enters only through per-key update counts, a workload property. Lower bounds: anchor bookkeeping by a fooling-set argument over traces that violate MONO if fewer than one bit of session watermark is kept; base touches by the adversary argument of Theorem 4.2(iii); freshness from clause (i) of the same theorem. *Restriction, stated honestly:* without anchor-indexed access, locating a key's deltas could require scanning history and would reintroduce n. The assumption is implemented, not idealized — but the theorem is about implementations that make it. ∎

**Consequence.** The institutional question "which rung can we afford for this view" is answered by measuring the workload, not by sizing the history: ten years of ledger cost the same per read as ten days, at every rung. This is the theory behind S3, and it is the claim most directly falsifiable by a single regression.

## 4.5 Contribution 4 — The Consistency-Effect Calculus and Niles Soundness

**The calculus λ_niles.** A core calculus with: base types including `Money⟨c, s⟩` (currency index c at scale s), `Account`, `Posting`, bitemporal types; *linear* resource types for posting halves; an effect lattice over {read@ℓ, append, debit⟨c⟩, credit⟨c⟩, overdraw, fx⟨a,b⟩, declassify}; and judgment Γ ⊢ e : T ! ε. Key rules, abbreviated:

- **(T-Money-Add)** both operands `Money⟨c,s⟩` with the *same* c and s. No rule joins distinct currency indices; currency mismatch is untypeable, in the manner Kennedy's units-of-measure system makes dimensional mismatch untypeable.
- **(T-Posting)** posting construction consumes linear debit/credit halves; a transaction term `txn{p₁…pₙ}` types only if, per currency, the multiset of halves cancels — the balance obligation is a type-level sum over an indexed monoid, discharged by the currency-row solver.
- **(T-Overdraw)** a posting whose static balance bound may cross zero requires the ambient capability `Auth⟨overdraw⟩`, which has no introduction rule except an authorization term.
- **(T-FX)** `fx⟨a,b⟩{legA, legB, rate}` types only if legA balances in a and legB balances in b (Definition 3.7 internalized).
- **(T-Serve)** a read term is typed at its view's declared rung; a function demanding ℓ₃ composes only with contexts serving ℓ₃ or above — consistency is an effect, so under-consistent composition is a type error.
- **(T-Idem)** submission is typed as an idempotent effect keyed by an idempotency term, with the declared deduplication window as part of the type (Section 3.20).

**Theorem 4.4 (Niles Soundness).** If ⊢ P : T ! ε, then for every trace of P's execution under the LTS of Section 3.11 — including crash/recover, evict, upquery and concurrency — (1) every committed transaction is balanced per currency, hence by Lemma 3.1 and Corollary 4.1.1 value is conserved at every epoch and across all eviction schedules; (2) no value of type `Money⟨a,·⟩` flows into a `Money⟨b,·⟩` context for a ≠ b; (3) no committed posting overdraws an account unless an `Auth⟨overdraw⟩` capability was in scope at its typing; (4) every read observed by P satisfies its declared rung's predicate.

*Proof.* Progress and preservation for λ_niles by structural induction. The novel cases are the currency-row solver — preservation of the per-currency sum under reduction — and linear consumption of posting halves, which is exactly what makes balance structural rather than checked. Clause (1): a typeable committed transaction has cancelling currency rows, giving Definition 3.4 and hence ledger-level conservation. Clause (2): no typing rule joins distinct indices, and preservation carries indices through reduction. Clause (3): capabilities are unforgeable (no introduction rule but authorization), so by preservation every overdraw redex carries one. Clause (4): the runtime serves per contract by P6, and T-Serve makes the demanded rung no stronger than the served rung. ∎

**What is inherited and what is new.** Inherited: dimensional safety from units-of-measure typing [Kennedy]; exactly-once resource consumption from linear types [Wadler]; effect rows from the algebraic-effects tradition [Leijen; Plotkin & Pretnar]; the pattern of making a security or audit property a typing property [Miller et al., POPL '14]. New: the *composition* of currency indices with linear posting halves so that double-entry balance is a typing judgement rather than a runtime constraint; and the treatment of *consistency as an effect*, so that a function's type records the freshness it requires and mis-composition is a compile error.

**What it does not give.** Clause (3) is enforcement of authorization, not of solvency: Proposition 3.2 proves non-negativity is not coordination-free, so the type system can require that the capability be present and the runtime must still coordinate at the reservation. Two further limits are declared rather than glossed. Refinement-style predicates (e.g. richer overdraft conditions) are decidable only relative to a chosen qualifier set, which is the standard price of liquid typing [Rondon et al., PLDI '08]. And confidentiality typing (Section 6.18) is a non-interference-style discipline whose *static-label* fragment is well understood, while dynamic labels remain an open problem in the information-flow literature — Sabelfeld and Myers note that no non-interference results exist for the dynamic-label fragment they describe. Niles therefore restricts confidentiality annotations to static labels, and says so.


### 4.5.1 The Currency-Row Solver: What the Analysis Actually Is

Contribution 4's conservation clause rests on a static analysis, and the thesis previously
asserted its properties in a sentence. This section places it in the program-analysis
literature, states its soundness precisely, and records where the original framing was
wrong — in two cases badly enough that the implementation was wrong with it.

**The carrier.** A transaction's net effect is not a number but a *row*: a finite map from
currency to a signed amount. Amounts are affine forms over opaque atoms, `c₀ + Σ kᵢ·xᵢ`
with integer coefficients, closed under addition, negation and integer scaling but not
under multiplication of two symbolic values or division. The whole state is the direct sum
`⨁_c (ℤ^X ⊕ ℤ)` over currencies.

That carrier is not new, and its pedigree is worth claiming rather than reinventing:
Ellerman's algebraic reconstruction of double-entry bookkeeping gives exactly this structure
as the *Pacioli group*, a group of differences on T-account pairs, generalized to vectors
indexed by property type. The row is his multi-dimensional generalization with currency as
the index.

**What it is, in the standard taxonomy.** The honest name is *a combination domain of affine
expressions over Herbrand atoms, restricted to a single accumulator per currency*. Two
components, and separating them matters because they fail differently.

The relationship to Karr's algorithm is best stated as an **encoding rather than a
specialization**. Introduce a ghost variable `net_c` per currency, model `debit` as
`net_c := net_c − m` and `credit` as `net_c := net_c + m`, and model each opaque money
source as a nondeterministic assignment. Then "this transaction conserves *c*" is exactly
the Karr query *does the affine relation `net_c = 0` hold at the transaction's exit?* The
solver is therefore a **cheap, non-relational implementation of one Karr query**, not a
degenerate case of Karr's domain — and it is strictly weaker at joins, where Karr's affine
hull can retain `net = x − y` across branches that disagree on `x` and `y` separately.

It is also *incomparable* to Karr on a second axis: Karr's variables are program locations,
whereas the atoms here are named values, so two reads of the same pure call should be the
same atom. That is a Herbrand-equality component, and the literature for the combination
exists — Gulwani and Necula's polynomial-time global value numbering, Müller-Olm, Rüthing
and Seidl on checking Herbrand equalities, and the interprocedural version.

**It is not a decision procedure.** The thesis previously used that phrase and it is an
overclaim. What the solver has is a canonical form and an equality test against zero, which
is *complete for the word problem in a finitely generated free abelian group* and decidable
in time linear in term size. It is not a decision procedure for Presburger arithmetic and
not the Omega test; the fragment sits far below both, deliberately.

**Where the incompleteness actually lives.** The thesis previously attributed the analysis's
limits to its inability to see that `x − x` is zero when the two `x` come from separate
calls. That is a *value-numbering* failure, not an arithmetic one, and it is fixable by
congruence closure. The arithmetic limits are different and worse, and they are where real
banking bugs live: **division and rounding** (`fee = amount * 3 / 10000`, then split with a
remainder leg), products of two symbolic values, guards, `min`/`max`, and — since amounts
are machine integers rather than mathematical ones — modular wraparound. An affine domain
over ℚ or ℤ is unsound with respect to overflow, so overflow-freedom is a side condition
this thesis discharges by construction (i128 minor units against realistic magnitudes) and
does not prove.

**The third verdict is forced, not conceded.** Müller-Olm and Seidl show by reduction from
Post's Correspondence Problem that in affine programs *with affine equality guards*, whether
a given affine relation holds at a program point is **undecidable**. `Undecided` is therefore
not an engineering weakness that more effort would remove; it is the shape of the problem,
and a decidable island inside it is the most any analysis of this kind can offer.

**Soundness, restated after two corrections.** The verdicts are `Conserves` (a proof),
`Violates` (an accusation) and `Undecided` (an obligation discharged to the runtime). The
standard vocabulary is *sound but incomplete*, and *must* versus *may*; "three-valued
analysis" is avoided because it names Sagiv–Reps–Wilhelm's shape analysis, which is a
different thing.

Getting `Violates` sound took two attempts, and both failures are instructive.

*The first* was that the analysis had **no control-flow join at all**. It walked both arms
of a branch into one accumulator, which is not an abstract interpretation of the program but
an analysis of a different program — one in which both branches run. On an eleven-line
transaction whose two arms each conserve, it produced four false errors: a phantom
imbalance, a spurious "never consumed", and two spurious "consumed twice". A checker that
accuses correct programs is worse than no checker, because it teaches its users to disable
it. The fix is a **top-preserving join**: where the arms agree the entry survives, where they
disagree it is poisoned to `Undecided`. On a single-expression-per-currency representation
that is the only sound join available, and the weakening is always toward silence.

*The second* was an over-correction. Faced with the observation that an aborting path leaves
a partial row, the analysis was made to weaken its verdict whenever a `?` appeared — and
promptly downgraded an unambiguous forty-dollar hole to a warning. The right treatment is
not to weaken but to **drop**: a `txn` is sealed atomically, so a path that leaves early
commits nothing and is not a path whose conservation is a question. This is a case of a
*runtime* guarantee buying *static* precision, and it is worth stating as such, because the
usual direction of that trade is the reverse.

With a top-preserving join and atomic transactions, a decided non-zero entry surviving a
merge was agreed by every arm — so it is a must-violation, and no provenance-based weakening
is needed. The machinery introduced to defend against merges turned out to be made
unnecessary by doing the merges correctly.

**The loop rule, which is a small result.** Because a row is a monoid homomorphism from
statement sequences into a free abelian group, *if each iteration's net is zero, the loop's
net is zero for any trip count* — with no widening, no trip-count reasoning and no fixpoint.
Conversely a body netting `m` contributes `n·m` for symbolic `n`, a product of two symbolic
values, hence `Undecided`. The asymmetry is not a limitation to apologise for: it is exactly
the discipline a batch-posting loop should follow, and it makes "balance every iteration" a
checkable style rule rather than advice.

**What conservation is not.** Linearity conserves *identity*; the row solver conserves
*magnitude*; **neither implies the other**. A linear type on `Money` guarantees the token is
not duplicated or dropped and says nothing about whether `split(m, k)` returns parts summing
to `m`. Conversely the row solver would accept `credit(b, m); credit(c, m)` from a single
external `m` — coefficient `2m`, verdict `Undecided`. Niles carries both disciplines, and
the thesis previously blurred them.

**The comparison to Move, corrected.** Earlier drafts stated that Move guarantees resources
cannot be created or destroyed. Move's own paper says the opposite: it guarantees no copying,
no implicit discarding and no reuse after move — *resource safety* — and then states
explicitly that "the Move type system cannot catch all implementation mistakes inside the
module. For example, the type system will not ensure that the total value of all Coins in
existence is preserved." Conservation in Move is a specification obligation discharged by
the Move Prover with hand-written invariants and an SMT backend. The same correction applies
to Nomos, whose linearity holds *modulo minting and burning* by design.

This correction **strengthens** the contribution rather than weakening it: the property
Move's type system explicitly declines to check is the one this solver checks
automatically, without user-written specs and without an SMT call.

**The closest prior art** is SolType, which gives Solidity refinement types with a `sum`
abstraction over mappings and can express `sum(balances) == totalSupply`. It is heavier
(refinement types plus SMT, so less predictable than normalization), it targets overflow
safety with the sum invariant as a means rather than an end, and it is single-asset — there
is no currency dimension. An extensive search did not surface prior work checking
double-entry balance as a static type or abstract-interpretation property; that is stated as
the outcome of a search rather than as an absolute, because the accounting-information-
systems literature is poorly indexed by computer-science search.

**The currency dimension is grading, not units and not rows.** Two terminological corrections.
It is *not* Kennedy's units-of-measure setting: currencies do not form a free abelian group
under multiplication, because `USD · EUR` names nothing. `Money` is a **Currency-graded
abelian group**, and conservation says a well-typed transaction is homogeneously zero in
every grade. Nor is the currency-variable machinery *row polymorphism* in Wand's or Leijen's
sense; it is ordinary first-order unification on a phantom type parameter, and calling it
row polymorphism was a misuse. There is a nearby design in which the term would be earned —
effect rows of the form `net ⟨usd: 0, eur: 0 | ρ⟩`, where a row variable and scoped-label
constraints would supply the *absence* facts ("this transaction has no JPY leg") that
unification alone cannot — and §12 records it as future work.

**FX pulls Kennedy straight back in, and it is where the solver stops.** A typed rate
`Rate<C₁,C₂>` has dimension `C₂·C₁⁻¹`, and `convert` multiplies a symbolic amount by a
symbolic rate: a product of two symbolic values, outside the fragment. The thesis therefore
does *not* claim static conservation across an FX conversion. What it claims, and what the
`fx` form implements, is the construction real systems use: the trade becomes **two balanced
single-currency legs against a position account**, each of which the solver handles
unchanged. Stating that converts a hole into a design decision, which is what it should have
been from the start.

**Interprocedurally, the analysis is currently weak, and the strong version has a citation.**
Today it inlines where a callee body is available and havocs to a fresh atom otherwise —
cloning-based context sensitivity plus havoc abstraction, sound but defeating cancellation
across the boundary. The compositional version is not speculative: Müller-Olm and Seidl's
POPL 2004 result computes all valid affine relations context-sensitively, representing each
procedure as a finite-dimensional vector space of weakest-precondition transformers, in time
linear in program size. For this domain it would be easier still, because the net effect of a
function is a homomorphism into a commutative group, so a summary is one row:

```niles
fn transfer<C>(from, to, m: Money<C>) -> ()      net { C: 0 }
fn fee<C>(m: Money<C>) -> Money<C>               net { C: -1*m + 1*result }
```

That is a type-and-effect system whose effect algebra is a currency-indexed free ℤ-module,
and it would make conservation **separately checkable per function** rather than only within
a transaction — the same move AARA makes with potential annotations. It is the single
highest-value extension to this contribution and it is not built.

## 4.6 Contribution 5 (SC5) — The Adaptive Materialization Calculus and Optimizer

The engineering requirement is an algorithm that decides what to materialize fully, what to keep on demand, and what to evict. The scientific contribution is to give that decision a formal object, a cost order, and provable guarantees — and to be explicit about which classical guarantees survive the setting and which do not.

**Definition 4.2 (Materialization mode lattice).** For each (view, key range), the mode μ ∈ 𝕄 = {absent, demand, full, spilled, tiered}, where: *absent* keeps nothing and reconstructs on every read; *demand* materializes on read and evicts under pressure (partial state); *full* materializes the whole range and maintains it eagerly; *spilled* keeps the range materialized but resident on secondary storage, paying I/O instead of reconstruction; *tiered* keeps a hot prefix resident and the remainder spilled, with a promotion rule. 𝕄 is ordered by resident cost and inversely by per-read reconstruction cost, and the optimizer's job is to choose a point per range subject to the view's contract.

**Definition 4.3 (Mode assignment problem).** Given views V with contracts κ, key ranges partitioned per view, a memory budget m, and a workload model, choose μ : ranges → 𝕄 minimizing expected total cost (application + residency + reconstruction + I/O + SLO-violation penalty) subject to: (a) every read satisfies its contract's rung and freshness bound; (b) Σ residency ≤ m; (c) pinned and `forever` ranges are assigned *full* or *tiered*.

**Theorem 4.5 (Optimizer properties).**

*(a) Safety.* For every assignment μ satisfying (a)–(c), the observable behaviour of the system is identical to that under any other satisfying assignment. Mode is cost, not semantics. *Proof:* by Theorem 4.1 every mode serves the certified value at the required anchor; modes differ only in where the value comes from. ∎ This clause is what allows the optimizer to change its mind at runtime without a correctness argument per change.

*(b) Hardness.* The offline mode-assignment problem is NP-hard. *Proof:* restrict to two modes (absent, full), unit residency cost, and a linear per-read cost model; the problem becomes view selection under a space budget, NP-complete by reduction from set cover [Harinarayan et al., SIGMOD '96]. ∎

*(c) The greedy guarantee does not transfer unmodified.* The classical (1 − 1/e) bound for view selection holds under a cost model in which the cost of answering a query is the size of the view used. Partial materialization violates that model: cost depends on miss rate and reconstruction depth, so benefit is not submodular in general. Two recoveries are given: (i) under *fixed* miss-rate estimates per range, benefit is again submodular and greedy recovers (1 − 1/e) — this is the mode used for planning-time defaults; (ii) in the general case, the guarantee is abandoned and replaced by the online analysis of (d), which is the mode used at runtime. *This is a negative result and is reported as one*: the most-cited guarantee in the view-selection literature does not survive the move to partial state.

*(d) Online guarantees per range.* For a single key range with reconstruction cost c_u and maintenance cost c_a per epoch, the decision "keep on demand or materialize fully" is a rent-or-buy problem: the break-even rule (materialize once cumulative reconstruction spend reaches the maintenance cost of materializing) is 2-competitive, and a randomized rule achieves a ratio approaching e/(e−1) ≈ 1.58, which is optimal for this class [Karlin et al., Algorithmica 1994]. For eviction *within* the demand mode, entries have heterogeneous size and heterogeneous reconstruction cost, which is the file-caching setting: a Landlord-style policy attains the k/(k−h+1) resource-augmented bound [Young, Algorithmica 2002], and this is the policy Nilestream implements rather than plain LRU.

*(e) The delayed-hit caveat.* Under reconstruction latency, the objective is aggregate delay rather than miss count, and the literature establishes that policies optimal for hit rate — including Belady's offline optimum — are not optimal for latency, with a reported deterministic competitive lower bound of Ω(kZ) attributed to parallel work [Atre et al., SIGCOMM '20]. The optimizer therefore ranks eviction candidates by *expected aggregate delay* — reconstruction cost weighted by the number of requests expected to arrive during reconstruction — rather than by reuse probability alone, and the thesis claims no optimality in this regime, only that ignoring Z is provably wrong.

**The algorithm.** Per range, maintain EWMA estimates of arrival rate, reuse distance, reconstruction cost and reconstruction latency; compute the rent-or-buy break-even for the mode choice and the Landlord-style credit for eviction, both adjusted by the delayed-hit weighting and by the contract multiplier Φ(ℓ) of Section 3.14; re-evaluate on a hysteresis schedule to avoid thrash. Ranges pinned by contract are excluded. Appendix I gives the algorithm in full, with its estimators, its hysteresis rule, and the dynamic program used offline to compute the optimum a measured run is compared against.

**Relationship to the self-tuning literature.** Choosing physical structures in response to observed demand is the agenda of adaptive indexing — "each query is interpreted not only as a request for a particular result set, but also as an advice to crack the physical database store" [Idreos et al., CIDR '07] — and of automated physical design [Chaudhuri & Narasayya, VLDB '97] and self-driving databases [Pavlo et al., CIDR '17]. What is new here is that the decision is made *under a declared consistency contract*, which none of that literature has: the same range may be cheap to keep on demand at ℓ₀ and unaffordable at ℓ₅, and Φ(ℓ) is the term that says so. The adaptive-indexing literature also supplies a cautionary datum the optimizer takes seriously: the choice of *how* to fill on demand changes convergence by two orders of magnitude — adaptive merging converges in roughly forty queries where cracking needs thousands [Graefe & Kuno, EDBT '10] — so reconstruction granularity is an optimizer parameter, not a constant.

## 4.7 Contribution 6 (SC6) — Generality: What "Replacing SQL" Can Mean and Be Proved

The claim that a new language replaces SQL is not, as stated, provable: SQL has no single formal semantics to be complete with respect to. This contribution replaces the slogan with three statements that can be proved, and proves them.

**Theorem 4.6 (Generality).** Let Niles-Core be the declarative tier of Niles (Appendix B) and IR its intermediate representation.

*(a) Relational completeness.* Niles-Core expresses every query expressible in the safe relational calculus. *Proof:* by exhibiting the relational operators as Niles pipeline stages and appealing to Codd's reduction of calculus to algebra [Codd, RJ987, 1972], with the algebra-equals-safe-calculus equivalence in the standard development. ∎

*(b) Fixpoint completeness over the epoch-ordered base.* Niles-Core with guarded recursion expresses exactly the queries computable in polynomial time over the base. *Proof:* Niles-Core includes relational calculus (a) plus a least-fixpoint construct, and the base carries a total order on epochs and a derived total order on rows. Immerman's theorem states that relational calculus plus least fixpoint, over structures with a total ordering relation, expresses exactly the PTIME queries [Immerman 1982/1986; independently Vardi 1982]. The ordering hypothesis — the usual obstacle to applying this theorem to unordered relational data — is satisfied *by the data model itself*, since the ledger totally orders its rows. Guardedness restricts recursion to well-founded measures, which is what keeps evaluation in PTIME rather than merely computable. ∎

*(c) SQL-fragment translation.* There is a total, semantics-preserving compilation of SQL-Core — the fragment stated in Appendix H, covering SELECT-FROM-WHERE with bag semantics, joins, grouping and aggregation, set operations, nulls with three-valued logic, ordering and limits, DDL, DML and TCL — into IR, such that the SQL query and its Niles counterpart lower to α-equivalent circuits. *Proof:* by structural induction on the SQL fragment's grammar, with the induction given in Appendix H and tested by golden-file equivalence over a corpus. ∎

**Remarks.** (i) The strongest available notion of a *complete* query language is Chandra and Harel's computable queries [JCSS 1980]; Niles's UDF tier reaches it in the trivial sense that arbitrary computation is expressible there, and the thesis does *not* claim it for the declarative tier, whose whole point is to stay in PTIME so that plans are optimizable and guarded. (ii) Fragment honesty: (c) is a claim about a stated fragment, and every construct outside it must fail loudly rather than silently diverge — the compatibility policy of Section 7.3. (iii) Comparative novelty: a survey of nine SQL alternatives finds that not one claims formal SQL-superset expressive power, and several are bounded above by SQL because they compile to it. A proved expressiveness theorem is therefore a genuinely novel artifact in this space, and it is a stronger claim than "replaces SQL" because it can be checked.

**Generality of the engine, not just the language.** The workload-class claim (S8) is discharged constructively rather than by theorem, and Section 6.5 gives the construction: OLTP is the write path plus point-lookup REVs; analytical rollups are REVs whose circuits aggregate, with column-shaped resident representations chosen by the optimizer; time-series is a REV over a base partitioned by the valid-time axis, with compression of the resident representation; document access is a REV over a typed semi-structured column with path-indexed resident state; graph traversal is guarded recursion over an edge base, in the sense standardized by SQL/PGQ and GQL; and search is an inverted-index REV whose resident state is exactly Lucene's immutable-segments-plus-tombstones shape — which is itself an existence proof that this architecture serves that workload. In every case the object is a REV over the same base with the same contract vocabulary, which is what "one engine" means here: not that one storage format serves everything, but that one *semantics* does, and format is the optimizer's choice.

## 4.8 Contribution 7 — The Bounded Reconstruction Theorem (SC7)

This contribution exists because an experiment refuted a claim the thesis had asserted in every prior draft, and the refutation identified the mechanism that makes the claim true. It is presented in that order, because the order is the evidence.

**The claim as it stood.** SC3 holds that the price of consistency under partial materialization is workload-shaped rather than history-shaped, and the earlier drafts attributed that property to per-key *anchor indices*: since a reconstruction folds only the postings on its own key, and per-key update counts are a workload property, cost should not depend on the length of the base.

**The refutation (§9.4.1).** With the key space held fixed, measured reconstruction cost grew 64× as history grew 64×. Anchor indices delivered an 80–112× reduction in the constant and no change whatever in the growth. The obvious defence — that a real institution opens accounts as it grows, so per-key counts stay bounded — was then tested by growing the key space in proportion, and also failed: cost still grew roughly 16× over the same range. The reason is intrinsic to skew rather than to the design. Under a Zipf distribution a hot key retains a roughly constant *share* of traffic, so its absolute update count grows with total traffic no matter how many cold keys are added; and reconstruction requests, drawn from the same distribution, land preferentially on exactly those keys. Growing the key space dilutes the tail, not the head.

**The mechanism.** A per-key **checkpoint**: every C postings on a key, record (epoch, value), and reconstruct by folding only the suffix since the newest checkpoint at or before the requested anchor. Formally this is Definitions 3.9–3.10 and **Theorem 3.7**: checkpointed reconstruction returns the same value as unbounded reconstruction, and reads at most C/2 + 1 base rows in expectation, independent of base length.

**The confirmation (§9.4.1, Table 9.7).** At C = 16, measured cost was flat at ≈ 8.5 base rows across a 64× increase in history, against a predicted C/2 + 1 = 9. At C = 64 it was 32.0 against a predicted 33. At C = 256 it was still rising slowly over the range tested, as the bound predicts for an interval comparable to the per-key update counts involved.

**Why this is a contribution and not an implementation note.** Three reasons.

*It changes the status of SC3.* The cost law is no longer an emergent property of the data structure but a **design obligation** with a stated constant. C is a term in the theory: it appears in the bound, it is declarable per view (§6.18), and choosing it is a decision the theory prices rather than a knob an operator guesses.

*It explains prior art that was otherwise unexplained.* Production ledgers maintain running balances rather than folding journals on demand, and at least one purpose-built ledger offers a per-account flag to retain balance history at each transfer. Those are the same bound bought under a different name. The thesis previously treated such materialized balances as an optimization that its own design made unnecessary; the measurement shows they are the mechanism that makes the design's cost claim true.

*It sharpens the immutability argument rather than weakening it.* A checkpoint is derived state — recomputable from the base by construction — so it is evictable, rebuildable, and outside the retention guarantee. The base still never forgets; the checkpoint merely spares a reader from re-reading all of it. That is precisely the base/derived split of F4 applied to itself, one level down, and it is a pleasing sign that the architecture's central distinction is doing work at more than one scale.

**What it does not claim.** The bound is on *reconstruction* work for a keyed fold. It says nothing about views whose recomputation is not a per-key fold — joins in particular, where the upquery path touches slices of several inputs and where checkpointing a single key is not obviously sufficient. Extending the bound to that class is stated as open in Chapter 12, and the prototype does not exercise it (Appendix K.7).

## 4.9 What Is New Beyond Noria, Materialize, ORCHESTRA, and the Log-Backed Map

Because several systems are close on one axis each, the differences are stated per system.

**Beyond Noria.** (1) *Base:* Noria's base tables are mutable; the REV's base is immutable, hash-chained and strictly serializable — which makes reconstruction pure and, per §4.2, dissolves the five anomalies rather than defending against them. (2) *Anchors:* Noria's entries are unversioned; REV entries carry epochs, making "as of" a first-class answer. (3) *Consistency:* Noria is explicitly eventually consistent with randomized eviction and lists stronger consistency as future work; this thesis serves declared rungs to ℓ₅ and proves where that is affordable. (4) *Invariants:* nothing in Noria can state, let alone prove, Corollary 4.1.1. (5) *Policy:* Noria's eviction is randomized; C5 replaces it with a cost model carrying competitive guarantees. (6) *Language and audit:* Noria consumes SQL and has no retention, audit or bitemporal story. What is kept, and credited, is the economic insight and the mechanism.

**Beyond Materialize (and the IVM engines).** Materialize already ships bounded staleness, serializable and strict serializable isolation, and documents precisely the anomaly that strict serializability removes. The differentiator is therefore not the ladder but *partiality*: those guarantees are over totally maintained state, so the memory cost of a view is the size of its output, not the size of its working set. This thesis's ladder is defined over *partial* state, which is what makes Theorem 4.2 necessary — Materialize never has to ask when eviction stops paying, because it never evicts.

**Beyond ORCHESTRA.** Semiring provenance for incremental maintenance of derived state, with a measured break-even against recomputation, is 2007 prior art. §4.3 states the three differences (residency and consistency versus update volume; the Φ(ℓ) axis; reconstruction latency) and this thesis adopts ORCHESTRA's formalism where it applies rather than reinventing it — Corollary 4.1.2 is an inherited result, not a claimed one.

**Beyond the log-backed map.** Trillian's verifiable log-backed map is structurally "authoritative log plus derived state applied only from the log," and Lucene has run immutable segments with tombstones and merging for two decades. The architecture is therefore not claimed as novel. What is claimed is the layer above it: partial materialization with versioned absence, anchored reconstruction, per-view consistency contracts, an optimizer with competitive guarantees, and a language whose types make the invariants checkable.

## 4.10 Summary: A Formal Theory and the Artifacts That Test It

The seven contributions interlock. C1 makes partiality *safe* — exactness at any anchor, conservation across eviction, lineage preserved. C3 *prices* every rung above that floor and shows the price is workload-shaped. C2 *bounds* the whole approach, saying where it stops paying and proving that no policy escapes. C5 *decides*, turning the bound into a runtime policy with competitive guarantees and an explicit account of which classical guarantees survive. C4 *transfers* all of it to programs, making conservation, currency safety and authorization typing judgements. C6 *generalizes*, replacing an unprovable slogan about SQL with three theorems and a construction covering the workload classes a general-purpose DBMS must serve. And SC7 *bounds the bound*: it supplies the checkpointing mechanism without which C3's cost law is false for skewed workloads, and it is the one contribution here that an experiment produced rather than confirmed.

Niles and Nilestream exist to make each claim falsifiable: the reconstruction theorem is attacked by the conservation suite under fault injection (S4), the cost theorems by the phase-diagram and history-scaling experiments (S2, S3), the optimizer by comparison against fixed policies and an offline optimum (S7), the generality claim by a corpus and an audited change log (S8), and everything by differential testing against the executable oracle. The theory is the contribution; Chapters 5–9 build and specify the test.
