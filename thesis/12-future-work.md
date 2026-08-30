# 12. Future Work

**Mechanized proofs.** Formalize the partial-state algebra, Theorem 4.1 and λ_niles soundness in a proof assistant, closing the largest declared gap in Section 3.15. The cost is lower than it would otherwise be because the substrate has already been mechanized: DBSP's mathematics has a Lean formalization, so the integration and differentiation laws this thesis builds on need not be re-established from scratch. The LTS and calculus were deliberately given small-step, syntax-directed presentations to keep this tractable.

**Sharpening the frontier.** Theorem 4.2's constants are loose outside special cases. A tight competitive analysis of online eviction under anchored strictness — properly accounting for reconstruction latency, where the literature's Ω(kZ)-style lower bounds suggest the achievable region is narrow — would turn the frontier band into a frontier line. Two extensions matter practically: correlated and non-stationary access processes, since real workloads have phases; and *learned* eviction with certified regret bounds against the theorem's optimum, which is the natural meeting point between this thesis and the learned-caching literature.

**Optimizer theory.** Contribution 5 abandons the classical greedy guarantee because partial materialization violates the cost model that guarantee assumes. Recovering an approximation guarantee for the general case — perhaps by identifying a restricted but realistic class where benefit remains submodular, perhaps by a different algorithmic route entirely — is an open problem this thesis states cleanly rather than solves.

**Byzantine and multi-operator profiles.** The hash chain already gives evidence; adding Byzantine-tolerant consensus profiles and cross-institution shared bases — interbank settlement as mutually audited REV subscriptions — would extend the theory to adversarial trust models and meet the blockchain literature on this thesis's semantic terms.

**Richer confidentiality.** Beyond commitments: zero-knowledge range proofs for authorization without disclosure (proving no overdraft without revealing a balance); secure multi-party computation for aggregate views over committed amounts; and a quantitative leakage budget for access patterns. The information-flow side has a specific open problem attached: Niles restricts confidentiality to static labels because non-interference results for dynamic labels do not exist in the literature, and extending the guarantee to dynamic labels is research rather than engineering.

**Language growth within the gate.** Guarded-recursive temporal logic for compliance rules — expressing obligations such as "no account below zero for more than three business days" as typed views; gradual adoption of the effect system by programs written against the SQL surface; and a verified-optimizer path in which compiler passes carry translation-validation certificates.

**Bootstrap trust, done properly.** Appendix E's determinism gates prove the compiler is a *fixed point*, which detects accidental non-determinism and some miscompilation classes. It does not detect a trusting-trust attack — a self-reproducing trojan is a fixed point by construction, so a bit-identical self-build is exactly what a successful attack looks like. Detecting that requires diverse double-compiling with an independently sourced parent compiler, of which reproducibility is a precondition rather than a substitute. Executing a genuine diverse-double-compilation of the Niles bootstrap is future work, and stating that plainly is more useful than implying the gates already provide it.

**Industrial validation.** Replace synthetic mixes with anonymized industrial traces under partnership; run the audit-rehearsal protocol with a real supervisory team against the audit-trail alternative that regulation now permits; and port one production product to `std::bank`, measuring the architectural-audit deltas in the field rather than in the laboratory.

**Beyond banking.** The conservation pattern generalizes to any conserved-quantity domain — inventory, emissions accounting, energy markets, in-game economies, clinical supply chains. A systematic treatment of the pattern family — which indexed monoids, which capability structures, which commit rules — would test whether Contribution 4 is a *schema* for domain calculi rather than one instance, which is the most interesting way this thesis could turn out to be more general than it claims.

## 12.x Work Named by the Counterproposal and the Solver Analysis

Six items, each arising from a specific gap identified in Chapters 4 and 6 rather than from
a general wish for more.

**A defect corpus drawn independently.** E14's twelve defect classes were chosen because
this thesis claims to catch them, which biases the result toward Niles. A corpus drawn from
CVEs, from published bank incident reports, or from the commit history of an open-source
ledger would be far stronger, and its absence is E14's largest single weakness. This is the
missing *frequency* premise of §6.10.3, and without it the argument for Niles is incomplete
in a way no amount of further engineering repairs.

**A cost study.** Against nine avoided defect classes stands the cost of adopting a new
language. Measuring it needs at minimum a migration of a real schema, timed, with the
reserved-word collisions of §9.13.5 counted rather than anticipated.

**A diagnostics trial.** The same money-safety violation rendered three ways — single span;
span plus prose warrant; span plus a second span at the rule — measured on blame-attribution
accuracy and time-to-correct-fix. No study has compared multi-span against single-span
diagnostics for any error class in any language, so this would be the first, it is small, and
§6.10.2's design rests on its outcome.

**A compositional conservation summary.** The solver is interprocedural only by inlining.
Müller-Olm and Seidl's context-sensitive affine analysis shows the compositional version is
tractable, and for this domain it is easier still, because a function's net effect is a
homomorphism into a commutative group and so a summary is a single row: `fn fee<C>(m:
Money<C>) -> Money<C> net { C: -1*m + 1*result }`. That would make conservation separately
checkable per function rather than only within a transaction — the single highest-value
extension to Contribution 4.

**Effect rows that earn the name.** §4.5.1 records that the currency-variable machinery is
first-order unification on a phantom parameter, not row polymorphism. The design in which the
term would be earned — `net ⟨usd: 0, eur: 0 | ρ⟩`, with a row variable and scoped-label
constraints supplying the *absence* facts that unification cannot — is strictly better than
what is built, and would let a function state that it has no JPY leg.

**Congruence closure over pure calls.** The solver's inability to see that `f(a) − f(a)` is
zero is a value-numbering failure, not an arithmetic one, and Gulwani and Necula's
polynomial-time global value numbering fixes it. This is the cheapest precision improvement
available and it was misattributed as a fundamental limit in earlier drafts.
