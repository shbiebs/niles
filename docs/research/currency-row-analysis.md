# The Currency-Row Solver: Placement, Properties, and Corrections

*A literature-grounded critique. Section 8 lists verification status for every claim I could not confirm from a primary source.*

---

## 0. Executive summary of the corrections

Before the detail, the seven things in your framing that are wrong or imprecise, ranked by how much damage they would do in a thesis defence:

1. **Move does *not* guarantee conservation of value.** The Move paper says so explicitly. This is your headline comparison and it is currently backwards.
2. **`Violates` is not a sound "must" verdict.** As specified it is a *may*-violation, and I give counterexamples below.
3. **"Row polymorphism" is a misuse.** What you have is a *graded* module and ordinary first-order unification on a phantom type parameter. There is a nearby design where the term becomes correct.
4. **Your incompleteness diagnosis is misattributed.** `x - x` failing is a *Herbrand/value-numbering* incompleteness, not an arithmetic one. The arithmetic incompletenesses are different and worse.
5. **"Decision procedure" is an overclaim.** You have a normalization plus equality test in a free abelian group — complete for the *word problem*, not a decision procedure for linear integer arithmetic.
6. **"Interprocedural only through inlining" contradicts "calls contribute a fresh opaque symbol."** These are two different treatments; you need to say which applies when.
7. **Nomos also does not guarantee conservation** — it guarantees linearity *modulo minting and burning*, plus gas bounds.

Two of these (1, 7) are good news: the gap you think you are filling is larger than you think.

---

## 1. What this is, in the standard taxonomy

### 1.1 The carrier

Per currency, your amount is an element of the **free ℤ-module on the set of opaque symbols, extended by a constant**: ℤ^X ⊕ ℤ, i.e. an *affine* form (not linear — you have a constant term; call it affine and be consistent). The full transaction state is the direct sum ⨁_{c ∈ Currency} (ℤ^X ⊕ ℤ) — a finite map from currency to that module. This carrier is exactly Ellerman's algebraic reconstruction of double-entry bookkeeping [36]: the *Pacioli group* (a group of differences on T-account pairs), which he generalizes to *vectors* indexed by property type. Your "row" is his multi-dimensional generalization, with currency as the index set. Cite him; it gives your carrier five hundred years of pedigree and a 2014 formalization.

### 1.2 Is it an abstract interpretation?

**Conditionally yes, and the condition is the join.** To be an abstract interpretation in the Cousot–Cousot sense [2] you must exhibit a concretization γ from your abstract elements to sets of concrete rows, prove each transfer function sound w.r.t. γ, and define a join for control-flow merges. If your implementation walks the `txn` body syntactically and accumulates, with no defined merge operation at `if`/`match`/loop back-edges, then what you have is **symbolic execution** or a **type-and-effect system** [40], not an abstract interpretation — and it is either path-explosive or unsound at merges. Decide which and say so. This is the single most important structural question about your artifact.

If you *do* have a join: note that your domain element is a **single affine form per currency**, not a set of affine constraints. The only sound join on a single-expression representation is: *if the two forms are not equal after normalization, go to ⊤ (Undecided)*. That is a **flat lattice of symbolic constants** — the linear-expression analogue of constant propagation. It is strictly weaker than Karr's domain, whose join is affine hull and can retain `net = x - y` when the branches disagree on `x` and `y` individually.

### 1.3 Relation to Karr

Karr [1] infers, at each program point, the set of *all* affine relations holding among the program's variables; the domain elements are affine subspaces of ℚ^k, represented by bases or kernels; join is affine hull; the algorithm is exact for *affine programs* (assignments are affine or nondeterministic, guards ignored) [6]. Müller-Olm and Seidl sharpened the complexity to **O(n·k³)** with number lengths bounded by O(n·k²) [6].

The precise relationship to your analysis is best stated as an **encoding**, not a specialization:

> For each currency c, introduce a ghost variable `net_c`, initialized to 0. Model `debit(a, m)` as `net_c := net_c − m` and `credit(a, m)` as `net_c := net_c + m`. Model each opaque money source as a nondeterministic assignment `x := ?`. Then "the transaction conserves c" is exactly the Karr query *"does the affine relation `net_c = 0` hold at the `txn` exit?"*

So: **your verdict is a Karr-expressible query, computed by a weaker, non-relational symbolic domain.** Not a degenerate case of Karr — a *different, cheaper implementation of one Karr query*. That is a defensible and honest framing, and it lets you inherit Karr's theory for free: in particular the exactness result [6] tells you what your domain gives up.

There is a second axis where you are **incomparable** to Karr rather than weaker: Karr's variables are program locations, whereas your `xᵢ` are *named atoms* — two reads of the same pure call are (or should be) the same symbol. That is a **Herbrand equality** / global-value-numbering component [10][11][12]. So the honest full name for your domain is:

> **A combination domain of affine expressions over Herbrand atoms**, restricted to a single accumulator per currency.

That combination has real literature: Gulwani and Necula's polynomial-time global value numbering [10]; Müller-Olm, Rüthing and Seidl on checking Herbrand equalities [11]; the interprocedural version [12]; and Petter/Seidl's "smooth combination of linear and Herbrand equalities" for polynomial-time must-alias analysis. Cite these; they are exactly your combination and nobody in banking has connected them to conservation.

---

## 2. Related domains, and whether you have a decision procedure

**Polyhedra** [3]: conjunctions of affine *inequalities*, convex-hull join, exponential in the worst case. Strictly more expressive than you need for conservation (which is an equality) but necessary the moment you have `min`, `max`, `clamp`, overdraft floors, or guarded transfers.

**Octagon** [4]: constraints of the form ±x ± y ≤ c; O(k²) memory, O(k³) closure. Note precisely: octagon *can* express a two-legged balance (`x + y ≤ 0 ∧ x + y ≥ 0` ⟹ `x + y = 0`), but **cannot express a three-or-more-legged balance** (`x + y + z = 0`). For a system whose core claim is double-entry with splits, octagon is structurally inadequate — say this, it is a clean argument.

**Congruences** [5]: `x ≡ a (mod m)`. Orthogonal to conservation but *directly relevant to your minor-unit discipline* — "all amounts are integral cents", "all lot sizes are multiples of 100". If you claim rounding-safety anywhere, this is the domain you want, and you should say so.

**Machine arithmetic.** If your amounts are `i64`/`i128` cents rather than mathematical ℤ, an affine domain over ℚ or ℤ is **unsound with respect to wraparound**. The correct move is either (a) discharge an overflow-freedom side condition separately, or (b) work in the affine domain over ℤ/2^w — see Elder et al., *Abstract Domains of Affine Relations* [13], which studies exactly this and shows the Müller-Olm/Seidl and King/Søndergaard modular domains are incomparable. Ignoring this is a live soundness hole in a *banking* thesis.

### 2.1 Is the zero-check a decision procedure?

**No, and you should stop calling it one.** What you have is:

- a **canonical form** (sorted, coefficient-normalized affine combination), and
- an **equality test against 0** on that canonical form.

The precise theoretical statement is: *this is a complete decision procedure for the word problem in a finitely generated free abelian group*, decidable in time linear in the term size. It is **not** a decision procedure for Presburger arithmetic [15], nor for quantifier-free linear integer arithmetic, and it is **not** the Omega test [16] — the Omega test decides satisfiability of systems of linear integer *inequalities* with quantifier elimination, with worst-case exponential (and for full Presburger, at least doubly-exponential [15]) behaviour. Your fragment is deliberately far below that.

Also drop "syntactically zero". After normalization it is *canonically* zero. "Syntactically" invites the reader to think you compare unnormalized ASTs.

### 2.2 Your completeness characterization: partly right, misattributed

You wrote: *"complete for the fragment it represents but incomplete for integer arithmetic generally, because it cannot see `x - x` is zero if the two `x` came from two different calls to the same pure function."*

The first clause is right (see the word-problem statement above). The causal clause is **wrong**: that is not an arithmetic incompleteness. Separate three sources:

**(i) Atom-identification incompleteness (Herbrand layer).** Failing to see `f(a) − f(a) = 0` is a failure of *value numbering*, i.e. of your symbol-allocation policy, not of the linear algebra. It is fixable by congruence closure over pure calls [10]. Saying "my linear domain is incomplete because of this" attributes a defect to the wrong component and hides the fact that it is *fixable*.

**(ii) Arithmetic-fragment incompleteness (the real one).** You cannot handle: products of two symbolic values; **division and rounding** — which is where actual banking conservation bugs live (`fee = amount * 3 / 10000`, then split with a remainder leg); guards (`if x > 0`); `min`/`max`/`clamp`; and modular wraparound.

**(iii) Genuine undecidability.** This is the result you should be leaning on. Müller-Olm and Seidl prove, by reduction from Post's Correspondence Problem, that **in affine programs extended with positive affine equality guards, it is undecidable whether a given affine relation holds at a program point** [6]. So "Undecided" is not an engineering embarrassment you might one day eliminate — it is *forced*. That reframes your three-valued verdict from "my tool is weak" to "the problem is undecidable and my domain is a principled decidable island." Use it.

**(iv) A soundness obligation you have not stated.** Cancelling `+m − m` is only valid if the two occurrences of `m` denote the same runtime value. If `m` is a mutable field read, or a call to a non-pure function, or aliased state that a nested call mutated, the cancellation is **unsound in the `Conserves` direction**. You need an explicit purity/immutability side condition on what may become an atom. Given that immutability is already a pillar of the Niles design, state this as a *dependency between two of your contributions*, not as a footnote.

---

## 3. The three-valued verdict

### 3.1 Terminology

- **"Sound but incomplete"** — correct and standard; keep it, but say sound *for what claim*.
- **"May/must"** — correct and standard, from the dataflow tradition [8]. Your `Conserves` is a **must** verdict (a proof); `Undecided` is the absence of one.
- **"Three-valued"** — avoid. In this literature "three-valued analysis" names Sagiv–Reps–Wilhelm's Kleene-logic shape analysis [39], which is a different thing. Say **"trichotomy: proof / alarm / unknown"**, the standard verifier vocabulary.
- **"Gradual verification"** — **wrong term** as used. Bader, Aldrich and Tanter's gradual verification [38] is about *imprecise formulas* (`?`) and static/dynamic hybrid checking, not about a verifier that answers "don't know." *However*: if `Undecided` causes you to **emit a residual runtime balance assertion at the txn boundary**, then gradual verification genuinely applies, and so do hybrid type checking [41] and soft typing. That is a much stronger design and a legitimate use of the term. Recommend it.
- **"Complete"** — loaded. In abstract interpretation, *completeness* is a technical property of a domain w.r.t. a concrete operation (Giacobazzi et al. [42]). Say "complete for the equational theory of free abelian groups" and nothing looser.

### 3.2 Is your soundness claim actually true? Counterexamples

Your `Conserves` direction is fine **provided** the join is ⊤-on-disagreement and the purity condition of §2.2(iv) holds.

Your **`Violates` direction is not sound as a must-verdict.** Three counterexamples:

**(a) Infeasible path.**
```
txn {
  debit(a, 5 USD);
  credit(b, 5 USD);
  if impossible_guard { credit(c, 1 USD) }   // guard never true
}
```
The merge sees a branch contributing `+1 USD`; with no path feasibility the analysis reports a nonzero constant. The program *always* conserves. **False `Violates`.**

**(b) Abort / rollback.**
```
txn {
  debit(a, 5 USD);
  if !authorized { abort }     // partial row discarded by the runtime
  credit(b, 5 USD);
}
```
The abort path has row `−5 USD`. At the *ledger* level the transaction is atomic and conserving. If the analysis treats `abort` as a normal exit, **false `Violates`**. Your transfer function must map abort-exits to ⊥, not to the accumulated row. This one is a real bug class, not a hypothetical.

**(c) Constant-folded imbalance that is genuinely reachable but currency-mismatched.** If `credit(b, 5 EUR)` was intended and the row is keyed by currency, you get `USD: −5, EUR: +5`. Both entries are nonzero constants, so you emit *two* `Violates`. Fine — but note the verdict is per-currency, so a single logical bug produces N alarms. A user-facing "unbalanced transaction" report should aggregate.

**Recommendation:** rename to **`Conserves` (proved) / `MayViolate` (alarm) / `Unknown`**, and reserve a genuine `Violates` for the restricted case where the `txn` body is straight-line, abort-free, and all atoms eliminated — there the negative verdict *is* a must, and you can even produce a witness. That restricted case is worth stating as a small theorem: *on straight-line abort-free bodies, the analysis is sound and complete in both directions.*

---

## 4. Prior art on conservation and resource accounting

### 4.1 The central distinction you need

**Linearity conserves *identity*; your row solver conserves *magnitude*. Neither implies the other.** Make this the organizing sentence of your related-work section, because essentially every system below sits on the identity side.

A linear type on `Money` [17][18][19] guarantees the *token* is neither duplicated nor dropped. It says nothing about whether `split(m, k) : (Money, Money)` returns parts summing to `m`. Conversely your row solver would happily accept `credit(b, m); credit(c, m)` from a single externally-sourced `m` — coefficient `2m`, verdict `Undecided`, not `Violates`. The two disciplines are complementary and Niles should carry both.

### 4.2 Move — your framing is backwards

You wrote that "Move explicitly guarantees resources cannot be created or destroyed." Move's own paper contradicts this. It guarantees, statically via the bytecode verifier:

- no copying of a resource value,
- no implicit discarding (resources "must be moved exactly once"),
- no reuse after move.

And then states the limitation directly: *"the Move type system cannot catch all implementation mistakes inside the module. For example, the type system will not ensure that the total value of all Coins in existence is preserved"* [20].

That is: Move enforces **resource safety** (no duplication or loss of *values of resource type*), not **conservation of the numeric field inside them**. Inside `Coin { value: u64 }`'s defining module, `value` is an ordinary `u64` and can be set to anything. Conservation in Move is a *specification obligation* discharged by the **Move Prover** [21] with hand-written `spec` invariants and an SMT backend — not by the type system, and not automatically. Sui inherits the same split.

**This is the best thing you found.** Your solver targets precisely the property Move's type system declines to check, and it does so automatically rather than via user-written specs and SMT. Rewrite this comparison; it strengthens your contribution rather than weakening it.

### 4.3 Nomos — also weaker than you assumed

Nomos [22] combines shared binary session types, a linear type system for assets, and AARA-derived gas bounds. The paper's own phrasing is that linearity holds **"modulo minting and burning"** — a `mint` process creates coins from nothing and `burn` destroys them, by design. So Nomos gives protocol conformance + no *accidental* duplication + resource bounds, **not** conservation of funds. Same correction as Move.

### 4.4 AARA — the strongest structural analogue

Automatic Amortized Resource Analysis [26][27][28] annotates types with **potential** expressed as linear (later multivariate polynomial) combinations of size parameters, generates linear constraints from the typing rules, and discharges them with an **LP solver**. The resemblance to your solver is real and worth a subsection:

| | AARA | Currency-row solver |
|---|---|---|
| Annotation | linear combination of size indices | affine form over opaque atoms |
| Inference | constraint generation + LP | normalization + zero test |
| Claim | *upper bound* on a monotone resource | *equality to zero* of a signed quantity |
| Compositional? | **yes** — potential is in the function type | not yet (see §5) |

The differences that matter: AARA's soundness is via a potential-function argument (physicist's method) and it bounds a *consumed* resource; you assert an *equality* on a *signed* quantity in a group rather than a monoid. Being in a group is actually an advantage — no widening needed for cancellation. Say so.

### 4.5 Separation logic

Permission accounting [29][31] and concurrent separation logic [30] are the closest *logical* analogue: fractional permissions must sum back to 1, and the logic enforces that sum as a conservation law over ℚ ∩ [0,1]. If you want a program-logic framing of conservation rather than an abstract-domain framing, this is where to look.

### 4.6 Smart-contract and blockchain languages

- **Obsidian** [23]: typestate + `asset` annotations; the compiler errors when an asset is dropped. Identity, not magnitude.
- **Flint** [24]: `Asset` trait with atomic transfer operations and safe arithmetic. Identity, plus overflow safety.
- **Scilla** [25]: communicating-automata IR with a Coq embedding for verification; conservation is a property you *prove*, not one the type system gives.
- **SolType** [32]: refinement types for Solidity with a `sum` abstraction over mappings, able to express and check `sum(balances) == totalSupply`. **This is the closest existing work to statically checking a ledger balance invariant** and you must cite it. Differences: it is refinement types + SMT (so heavier and less predictable than your normalization), it targets overflow safety with the sum invariant as a means, and it is single-asset — no currency dimension.

### 4.7 Double-entry as a static analysis: the gap

I searched for prior work treating double-entry balance as a type system or static analysis and **found none** in mainstream indexing. What exists is (i) Ellerman's algebra of the *carrier* [36], (ii) financial-contract combinator DSLs (Peyton Jones, Eber, Seward [37]) which type *contract structure* but do not check conservation, and (iii) DAML, whose ledger model enforces authorization and atomicity but does not statically type balance.

**Do not claim this gap absolutely.** Say: *"An extensive search of the programming-languages and financial-software literature did not surface prior work checking double-entry balance as a static type or abstract-interpretation property; the closest are SolType's sum-refinements and the Move Prover's user-written conservation specs."* That is defensible; "no such work exists" is not.

---

## 5. The `txn` boundary and interprocedural treatment

### 5.1 Your description is self-contradictory

"Interprocedural only through inlining" and "calls that produce money contribute a fresh opaque symbol" are **two different analyses**. Presumably: inline when the callee body is available and non-recursive; havoc to a fresh atom otherwise. State the policy as a rule with side conditions. In standard vocabulary:

- Inlining = **cloning-based / full call-string context sensitivity** [8]; unsound-by-omission for recursion, exponential in call depth.
- Fresh atom per opaque call = **havoc abstraction**, which is sound (it can only produce `Undecided`) but silently defeats cancellation across the boundary.

Note also that **IFDS** [9] does *not* apply here: IFDS requires a finite domain with distributive transfer functions, and your domain is infinite.

### 5.2 The summary-based version exists, with theory

Yes — and the reference is exact. Müller-Olm and Seidl, *Precise Interprocedural Analysis through Linear Algebra*, POPL 2004 [7], computes **all** valid affine relations context-sensitively, representing each procedure not by a single summary but by a **finite-dimensional vector space of weakest-precondition transformers**, spanned by at most (k+1)² matrices, in **O(n·k⁸)** time — linear in program size. This is Sharir and Pnueli's *functional* approach [8] instantiated for exactly your domain. It is the direct, citable answer to "could I compute a conservation summary per function and compose them?"

### 5.3 What your summary should look like

Because `debit`/`credit` are *pure additive effects into a commutative group*, the net effect of a function is a **monoid homomorphism into the free abelian group**: sequential composition is `+`, branching is join, identity is the zero row. This is the algebraic reason a summary-based version is easy — much easier than the general Karr case. Concretely:

```
fn transfer<C>(from, to, m: Money<C>) -> ()  net { C: 0 }
fn fee<C>(m: Money<C>) -> Money<C>           net { C: -1·m + 1·result }
```

That is a **type-and-effect system** [40] whose effect algebra is a Currency-indexed free ℤ-module. Two consequences worth stating as results:

1. **Modularity theorem.** If every callee's `net` annotation is verified, the caller's row is computable without inlining, and verification is compositional and separately checkable. This is what AARA does with potential, and it is what you should do with rows.
2. **Loop theorem.** Since `net` is a homomorphism, *if each loop iteration's net is zero, the loop's net is zero for any trip count* — no widening, no trip-count reasoning, no symbolic multiplication. Conversely a loop whose body nets `+m` contributes `n·m` with symbolic `n`, which is **outside your fragment** (product of two symbolics) and must go to `Undecided`. This asymmetry is a genuine, defensible small theorem and directly justifies the "balanced-per-iteration" idiom in Niles.

---

## 6. The currency dimension

### 6.1 You are right that it is not Kennedy's setting — for the right reason

Kennedy's units of measure [33][34] make units a **free abelian group** under multiplication, generated by base units and unit variables with integer exponents, with equality decided by **AG-unification** (equational unification for abelian groups) admitting most general unifiers; the payoff is a parametricity/scaling-invariance theorem (`f : ∀α. float<α> → float<α²>` implies `f(kx) = k²f(x)`).

Currencies are not this, because `USD · EUR` is not a meaningful type. Your value space is the **direct sum** ⨁_{c} A — equivalently, `Money` is a **Currency-graded abelian group**, and conservation is the statement that a well-typed `txn` is *homogeneously zero in every grade*. The right vocabulary is **grading** / **indexed direct sum**, not "group of units" and not "disjoint sum" (a disjoint sum has no addition; you need the direct sum, where per-index addition is defined and cross-index addition is not).

### 6.2 But FX pulls you straight back into Kennedy

The moment you type exchange rates, `Rate<C₁,C₂>` naturally has dimension `C₂ · C₁⁻¹`, and `convert : Money<C₁> → Rate<C₁,C₂> → Money<C₂>` is dimensionally exactly Kennedy's multiplication. So the honest statement is:

> Per-currency balances live in a Currency-graded module, not a group of units; but a typed FX layer re-introduces the free abelian group on currencies, and there Kennedy's system applies directly.

**And this is where your solver breaks.** `convert` multiplies a symbolic amount by a symbolic rate — a product of two symbolics, outside your fragment. Since your thesis promises "atomic cross-currency conservation," you need to say what the conservation predicate even *is* across an FX leg (conservation relative to a rate, or conservation of a numéraire-valued total, or per-leg conservation with the FX desk as a counterparty account). The last of these is what real systems do and it is the one your existing solver can handle unchanged — the FX trade becomes two balanced single-currency legs against a position account. Say that explicitly; it converts a hole into a design decision.

### 6.3 "Row polymorphism" — currently a misuse

Wand, Rémy and Leijen's rows [35] are **type-level** finite maps from labels to *types*, with **row variables** and a `lacks`/scoped-label discipline enabling extensible records and principal types. Your map is a value-/analysis-level map from currency labels to *group elements*, and your "currency variables resolved by union-find" is plain first-order syntactic unification of a phantom type parameter — standard Hindley–Milner, not row unification.

Two things follow. First, correct the term. Second, note that there *is* a design where it becomes accurate and would be a genuine contribution: make the net effect a **row type** in the Leijen sense,

```
fn settle(...) -> () net ⟨usd: 0, eur: 0 | ρ⟩
```

with a row variable `ρ` for currencies the function does not touch, and scoped-label/`lacks` constraints supplying exactly the *absence* facts ("this transaction has no JPY leg") that union-find alone cannot give you. That would be row polymorphism, correctly deployed, on the effect side. It is a strictly better design than what you described and it makes the citation earned.

---

## 7. Consolidated correction list

| # | Claim as stated | Correction |
|---|---|---|
| 1 | "Move guarantees resources cannot be created or destroyed" | Move guarantees no copy / no implicit discard / no reuse of resource **values**, statically. Its paper explicitly says the type system does **not** ensure total Coin value is preserved [20]. Conservation is a Move Prover spec obligation [21]. |
| 2 | "Nomos … resource accounting" implies conservation | Nomos enforces linearity **modulo minting and burning**, plus AARA gas bounds. Not conservation [22]. |
| 3 | "Row polymorphism" | Misuse. You have a Currency-**graded** module + HM unification on a phantom parameter. Rows [35] are type-level label→type maps with row variables. Fix the term, or adopt the effect-row design that earns it. |
| 4 | "syntactically zero" | Say **canonically** zero — you normalize first. |
| 5 | "decision procedure" | It is normalization + equality test; complete for the **word problem in a free abelian group**, not a decision procedure for LIA or Presburger [15][16]. |
| 6 | "incomplete … because two `x` from two calls" | That is a **Herbrand/value-numbering** incompleteness [10][11], and it is *fixable* by congruence closure. The arithmetic incompletenesses are division/rounding, symbolic products, guards, min/max, and modular wraparound. |
| 7 | Implicitly, that full precision is a matter of effort | With affine equality **guards**, deciding whether an affine relation holds is **undecidable** (PCP reduction) [6]. `Undecided` is forced, not a weakness. |
| 8 | "degenerate case of Karr" | It is a **Karr-expressible query** (`net_c = 0` with a ghost accumulator and `x := ?` for money sources) computed in a *weaker, non-relational* domain. Weaker at joins; incomparable due to atom naming. |
| 9 | "`Violates` if all kᵢ=0, c₀≠0" | Not a sound **must**-violation: infeasible paths and abort/rollback paths give false alarms. Rename to `MayViolate`; reserve a true `Violates` for straight-line abort-free bodies, where you can also emit a witness. |
| 10 | "abstract interpretation" | Only if you define γ, prove transfer soundness, **and** define a join. Without a join it is symbolic execution or a type-and-effect system [40]. |
| 11 | "commutative group" for amounts | Fine mathematically, but if amounts are `i64`/`i128` minor units, an affine domain over ℚ/ℤ is **unsound under wraparound**. Use ℤ/2^w [13] or discharge overflow-freedom separately. |
| 12 | Cancellation `+m − m = 0` | Requires a **purity/immutability side condition** on atoms. Two reads of a mutable field are not the same value. |
| 13 | "interprocedural only through inlining" | Contradicts "calls contribute a fresh opaque symbol." Name both: cloning-based context sensitivity vs. havoc abstraction. IFDS [9] does not apply (infinite domain). |
| 14 | "gradual verification" as a name for the trichotomy | Wrong [38]. Use "sound but incomplete", "may/must" [8], "proof/alarm/unknown". Gradual verification *does* apply if `Undecided` emits a residual runtime assertion [38][41]. |
| 15 | Conservation subsumes asset safety | It does not. **Linearity conserves identity; the row solver conserves magnitude.** `credit(b,m); credit(c,m)` gives `2m` → `Undecided`, not `Violates`. Ship both disciplines. |
| 16 | Cross-currency conservation is in scope | `convert` multiplies two symbolics — outside the fragment. Either define conservation relative to a rate, or model FX as two balanced legs against a position account (recommended). |
| 17 | Novelty of double-entry-as-static-analysis | Likely genuine, but state it as "an extensive search did not surface prior work; the closest are SolType [32] and Move Prover specs [21]" — not as an absolute. |

---

## 8. Verification status

**Verified against a primary source I fetched:** Karr's exactness and the O(n·k³) complexity and the guard-undecidability result [6]; Müller-Olm/Seidl POPL'04 complexity O(n·k⁸), backward WP formulation, and summaries-as-vector-spaces [7]; Move's explicit disclaimer about total Coin value [20]; Nomos's "linear modulo minting and burning" and its author list of four (Das, Balzer, Hoffmann, Pfenning) [22]; Kennedy's free-abelian-group units, AG-unification, MGU theorem, and scaling-invariance [33]; Ellerman's Pacioli group and vector generalization [36]; Elder et al. on affine relations mod 2^w [13].

**Not verified — treat as needing a library check before you cite:**
- The **DOI strings** for [7], [9], [16], [24], [25], [26], [27], [34], [39], [40], [41], [42]. I have the correct titles, authors, venues and years, but did not open a page confirming the DOI digits. Verify each in ACM DL / DBLP before submission.
- **Nomos venue and author count.** The CMU preprint I fetched carries a placeholder header ("Proc. ACM Program. Lang., Vol. 1, No. CONF"). Jan Hoffmann's publication page lists it as **CSF 2021**. Some listings add a fifth author (Santurkar); I could not confirm. Check DBLP.
- **Granger 1989** ("Static analysis of arithmetical congruences", *Int. J. Computer Mathematics*) — I verified the 1991 TAPSOFT companion's DOI but not the 1989 journal article's pagination.
- **SolType author list and exact venue** (POPL 2022 vs. PLDI 2022). The PDF is at the UT Austin URL below; confirm before citing.
- I did **not** find any paper treating double-entry balance as a static analysis or type system. Absence of evidence here is weak evidence of absence — a targeted search of the accounting-information-systems literature (which is poorly indexed by CS search) is worth doing before you claim the gap.

---

## References

[1] M. Karr, "Affine relationships among variables of a program," *Acta Informatica*, vol. 6, no. 2, pp. 133–151, 1976. doi: 10.1007/BF00268497. https://doi.org/10.1007/BF00268497 · PDF: https://www.cs.utexas.edu/~tdillig/cs395/karr.pdf

[2] P. Cousot and R. Cousot, "Abstract interpretation: a unified lattice model for static analysis of programs by construction or approximation of fixpoints," in *Proc. 4th ACM SIGACT-SIGPLAN Symp. Principles of Programming Languages (POPL)*, 1977, pp. 238–252. doi: 10.1145/512950.512973. https://dl.acm.org/doi/10.1145/512950.512973

[3] P. Cousot and N. Halbwachs, "Automatic discovery of linear restraints among variables of a program," in *Proc. 5th ACM SIGACT-SIGPLAN Symp. Principles of Programming Languages (POPL)*, 1978, pp. 84–96. doi: 10.1145/512760.512770. https://dl.acm.org/doi/10.1145/512760.512770

[4] A. Miné, "The octagon abstract domain," *Higher-Order and Symbolic Computation*, vol. 19, no. 1, pp. 31–100, 2006. doi: 10.1007/s10990-006-8609-1. https://doi.org/10.1007/s10990-006-8609-1 · https://inria.hal.science/hal-00136639

[5] P. Granger, "Static analysis of linear congruence equalities among variables of a program," in *TAPSOFT '91*, LNCS 493, pp. 169–192. doi: 10.1007/3-540-53982-4_10. https://link.springer.com/chapter/10.1007/3-540-53982-4_10

[6] M. Müller-Olm and H. Seidl, "A note on Karr's algorithm," in *ICALP 2004*, LNCS 3142, pp. 1016–1027. doi: 10.1007/978-3-540-27836-8_85. https://www.uni-muenster.de/imperia/md/content/informatik/agmueller-olm/publications/icalp04.pdf

[7] M. Müller-Olm and H. Seidl, "Precise interprocedural analysis through linear algebra," in *Proc. 31st ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, Venice, Italy, 2004, pp. 330–341. http://www2.in.tum.de/bib/files/Muller-Olm04Precise.pdf *(DOI unverified)*

[8] M. Sharir and A. Pnueli, "Two approaches to interprocedural data flow analysis," in *Program Flow Analysis: Theory and Applications*, S. Muchnick and N. Jones, Eds. Prentice-Hall, 1981, pp. 189–233. https://archive.org/details/twoapproachestoi00shar

[9] T. Reps, S. Horwitz, and M. Sagiv, "Precise interprocedural dataflow analysis via graph reachability," in *Proc. 22nd ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 1995, pp. 49–61. *(DOI unverified)*

[10] S. Gulwani and G. C. Necula, "A polynomial-time algorithm for global value numbering," *Science of Computer Programming*, vol. 64, no. 1, pp. 97–114, 2007. https://www.sciencedirect.com/science/article/pii/S0167642306001596 · https://people.eecs.berkeley.edu/~necula/Papers/gvndet-journal06.pdf

[11] M. Müller-Olm, O. Rüthing, and H. Seidl, "Checking Herbrand equalities and beyond," in *VMCAI 2005*, LNCS 3385, pp. 79–96. doi: 10.1007/978-3-540-30579-8_6. https://link.springer.com/chapter/10.1007/978-3-540-30579-8_6

[12] M. Müller-Olm, H. Seidl, and B. Steffen, "Interprocedural Herbrand equalities," in *ESOP 2005*, LNCS 3444, pp. 31–45. doi: 10.1007/978-3-540-31987-0_4. https://link.springer.com/chapter/10.1007/978-3-540-31987-0_4

[13] M. Elder, J. Lim, T. Sharma, T. Andersen, and T. Reps, "Abstract domains of affine relations," *ACM Trans. Program. Lang. Syst.*, vol. 36, no. 4, 2014. https://research.cs.wisc.edu/wpis/papers/TOPLAS-00017-2013-Revision2.pdf · https://research.cs.wisc.edu/wpis/abstracts/tr1792.abs.html

[14] A. Miné, "Tutorial on static inference of numeric invariants by abstract interpretation," *Foundations and Trends in Programming Languages*, vol. 4, no. 3–4, pp. 120–372, 2017. https://perso.lip6.fr/Antoine.Mine/publi/article-mine-FTiPL17.pdf

[15] M. J. Fischer and M. O. Rabin, "Super-exponential complexity of Presburger arithmetic," MIT LCS TM-043, 1974. https://en.wikisource.org/wiki/Page:Super-exponential_Complexity_of_Presburger_Arithmetic_by_Fischer_and_Rabin_(1974)_-_MIT-LCS-TM-043.pdf/1

[16] W. Pugh, "The Omega test: a fast and practical integer programming algorithm for dependence analysis," in *Proc. Supercomputing '91*, 1991, pp. 4–13. https://www.researchgate.net/publication/2772646 *(DOI unverified; a CACM 1992 version also exists)*

[17] J.-Y. Girard, "Linear logic," *Theoretical Computer Science*, vol. 50, no. 1, pp. 1–101, 1987. doi: 10.1016/0304-3975(87)90045-4

[18] P. Wadler, "Linear types can change the world!," in *Programming Concepts and Methods*, M. Broy and C. Jones, Eds. North-Holland, 1990. https://homepages.inf.ed.ac.uk/wadler/papers/linear/linear.ps

[19] D. Walker, "Substructural type systems," in *Advanced Topics in Types and Programming Languages*, B. C. Pierce, Ed. MIT Press, 2005, ch. 1. https://www.cs.princeton.edu/~dpw/cv.html

[20] S. Blackshear, E. Cheng, D. L. Dill, V. Gao, B. Maurer, T. Nowacki, A. Pott, S. Qadeer, Rain, D. Russi, S. Sezer, T. Zakian, and R. Zhou, "Move: A language with programmable resources," Diem Association, rev. May 2020. https://diem-developers-components.netlify.app/papers/diem-move-a-language-with-programmable-resources/2020-05-26.pdf

[21] J. E. Zhong, K. Cheang, S. Qadeer, W. Grieskamp, S. Blackshear, J. Park, Y. Zohar, C. Barrett, and D. L. Dill, "The Move Prover," in *CAV 2020*, LNCS 12224, pp. 137–150. doi: 10.1007/978-3-030-53288-8_7. https://link.springer.com/chapter/10.1007/978-3-030-53288-8_7

[22] A. Das, S. Balzer, J. Hoffmann, and F. Pfenning, "Resource-aware session types for digital contracts," in *Proc. 2021 IEEE 34th Computer Security Foundations Symposium (CSF)*, 2021. arXiv:1902.06056. https://arxiv.org/abs/1902.06056 · https://www.cs.cmu.edu/~balzers/publications/digital_contracts_as_session_types.pdf *(venue and possible fifth author unverified)*

[23] M. Coblenz, R. Oei, T. Etzel, P. Koronkevich, M. Baker, Y. Bloem, B. A. Myers, J. Sunshine, and J. Aldrich, "Obsidian: Typestate and assets for safer blockchain programming," *ACM Trans. Program. Lang. Syst.*, vol. 42, no. 3, 2020. doi: 10.1145/3417516. https://dl.acm.org/doi/10.1145/3417516

[24] F. Schrans, S. Eisenbach, and S. Drossopoulou, "Writing safe smart contracts in Flint," in *Conf. Companion of the 2nd Int. Conf. on Art, Science, and Engineering of Programming*, 2018. *(DOI unverified)*

[25] I. Sergey, V. Nagaraj, J. Johannsen, A. Kumar, A. Trunov, and K. C. G. Hao, "Safer smart contract programming with Scilla," *Proc. ACM Program. Lang.*, vol. 3, no. OOPSLA, 2019. *(DOI unverified)*

[26] M. Hofmann and S. Jost, "Static prediction of heap space usage for first-order functional programs," in *Proc. 30th ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 2003, pp. 185–197. https://homepages.inf.ed.ac.uk/stg/grants/REQUEST/references/p185-hofmann.pdf *(DOI unverified)*

[27] J. Hoffmann, K. Aehlig, and M. Hofmann, "Multivariate amortized resource analysis," *ACM Trans. Program. Lang. Syst.*, vol. 34, no. 3, 2012. https://www.cs.yale.edu/homes/hoffmann/papers/HAH12Toplas.pdf *(DOI unverified)*

[28] J. Hoffmann and S. Jost, "Two decades of automatic amortized resource analysis," *Mathematical Structures in Computer Science*, vol. 32, no. 6, pp. 729–759, 2022. https://www.cs.cmu.edu/~janh/assets/pdf/HoffmannJ21.pdf

[29] R. Bornat, C. Calcagno, P. O'Hearn, and M. Parkinson, "Permission accounting in separation logic," in *Proc. 32nd ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 2005, pp. 259–270. https://dl.acm.org/doi/10.1145/1047659.1040327

[30] P. W. O'Hearn, "Resources, concurrency and local reasoning," *Theoretical Computer Science*, vol. 375, no. 1–3, pp. 271–307, 2007. http://www0.cs.ucl.ac.uk/staff/p.ohearn/papers/concur04.pdf

[31] J. Boyland, "Checking interference with fractional permissions," in *SAS 2003*, LNCS 2694, pp. 55–72. *(DOI unverified)*

[32] B. Tan, B. Mariano, S. K. Lahiri, I. Dillig, and Y. Feng, "SolType: Refinement types for arithmetic overflow in Solidity," *Proc. ACM Program. Lang.*, vol. 6, no. POPL, 2022. arXiv:2110.00677. https://arxiv.org/pdf/2110.00677 · https://www.cs.utexas.edu/~isil/soltype.pdf *(author list / venue partially unverified)*

[33] A. Kennedy, "Types for units-of-measure: Theory and practice," in *Central European Functional Programming School (CEFP 2009)*, LNCS 6299, pp. 268–305. http://typesatwork.imm.dtu.dk/material/TaW_Paper_TypesAtWork_Kennedy.pdf

[34] A. Kennedy, "Relational parametricity and units of measure," in *Proc. 24th ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 1997, pp. 442–455. *(DOI unverified)* · See also A. Kennedy, "Programming languages and dimensions," PhD thesis, Univ. of Cambridge, 1996.

[35] D. Leijen, "Extensible records with scoped labels," in *Trends in Functional Programming (TFP 2005)*, pp. 179–194. https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/scopedlabels.pdf · See also M. Wand, "Complete type inference for simple objects," *LICS 1987*; D. Rémy, "Type inference for records in a natural extension of ML," 1993.

[36] D. Ellerman, "On double-entry bookkeeping: The mathematical treatment," *Accounting Education*, vol. 23, no. 5, pp. 483–501, 2014. doi: 10.1080/09639284.2014.949803. arXiv:1407.1898. https://arxiv.org/abs/1407.1898

[37] S. Peyton Jones, J.-M. Eber, and J. Seward, "Composing contracts: an adventure in financial engineering (functional pearl)," in *Proc. 5th ACM SIGPLAN Int. Conf. on Functional Programming (ICFP)*, 2000, pp. 280–292. doi: 10.1145/357766.351267. https://dl.acm.org/doi/10.1145/357766.351267

[38] J. Bader, J. Aldrich, and É. Tanter, "Gradual program verification," in *VMCAI 2018*, LNCS 10747, pp. 25–46. doi: 10.1007/978-3-319-73721-8_2. http://www.cs.cmu.edu/~aldrich/papers/vmcai2018-gradual-verification.pdf

[39] M. Sagiv, T. Reps, and R. Wilhelm, "Parametric shape analysis via 3-valued logic," *ACM Trans. Program. Lang. Syst.*, vol. 24, no. 3, pp. 217–298, 2002. *(DOI unverified)*

[40] J. M. Lucassen and D. K. Gifford, "Polymorphic effect systems," in *Proc. 15th ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 1988, pp. 47–57. *(DOI unverified)*

[41] C. Flanagan, "Hybrid type checking," in *Proc. 33rd ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 2006, pp. 245–256. *(DOI unverified)*

[42] R. Giacobazzi, F. Ranzato, and F. Scozzari, "Making abstract interpretations complete," *Journal of the ACM*, vol. 47, no. 2, pp. 361–416, 2000. *(DOI unverified)*

[43] MystenLabs Team, "The Sui smart contracts platform," 2022. https://docs.sui.io/paper/sui.pdf

---

**Sources consulted (search results):** [Karr — Springer](https://link.springer.com/article/10.1007/BF00268497) · [A Note on Karr's Algorithm](https://www.uni-muenster.de/imperia/md/content/informatik/agmueller-olm/publications/icalp04.pdf) · [Precise Interprocedural Analysis through Linear Algebra](http://www2.in.tum.de/bib/files/Muller-Olm04Precise.pdf) · [Move whitepaper](https://diem-developers-components.netlify.app/papers/diem-move-a-language-with-programmable-resources/2020-05-26.pdf) · [The Move Prover](https://link.springer.com/chapter/10.1007/978-3-030-53288-8_7) · [Nomos preprint](https://www.cs.cmu.edu/~balzers/publications/digital_contracts_as_session_types.pdf) · [Jan Hoffmann publications](https://www.cs.cmu.edu/~janh/publications/) · [Kennedy, Types for Units-of-Measure](http://typesatwork.imm.dtu.dk/material/TaW_Paper_TypesAtWork_Kennedy.pdf) · [Leijen, Scoped Labels](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/scopedlabels.pdf) · [Ellerman, arXiv:1407.1898](https://arxiv.org/abs/1407.1898) · [Elder et al., Abstract Domains of Affine Relations](https://research.cs.wisc.edu/wpis/abstracts/tr1792.abs.html) · [Miné, Numeric Invariants Tutorial](https://perso.lip6.fr/Antoine.Mine/publi/article-mine-FTiPL17.pdf) · [Obsidian, TOPLAS](https://dl.acm.org/doi/abs/10.1145/3417516) · [SolType](https://www.cs.utexas.edu/~isil/soltype.pdf) · [Gradual Program Verification](http://www.cs.cmu.edu/~aldrich/papers/vmcai2018-gradual-verification.pdf) · [Composing Contracts](https://dl.acm.org/doi/10.1145/357766.351267)
