# 7. Implementation Design and Current Status

This chapter records what is specified, what is built, and what is not. That boundary moved substantially during the work, and this chapter is where the current position is stated once and plainly, so that no reader has to infer it from an appendix's tense.

**Status.** The artifact accompanying this thesis contains a **working stage-0 compiler** for a large subset of Niles, a **typed IR with a verifier**, a **REV runtime** that executes compiled circuits, a **durable, concurrent ledger write path**, and a **research prototype** with an experiment harness. The loop from Niles source text to a measured result is closed end to end and is reproducible by a stranger with `cargo`. What remains unbuilt is stated in §7.5 with equal precision: there is no distributed execution, no consensus, no wire protocol, no self-hosted back end, and the executable IR fragment is narrower than the IR the compiler emits.

Everything described below that is *not* marked as built is written as an implementation plan with its design decisions and their justifications. No claim of completed performance work is made anywhere in this thesis beyond the measurements Chapter 9 reports and labels.

## 7.1 Nilestream-Core and the Ledger

**Ledger crate.** Implements Definition 3.1 directly: an epoch segment format (header with epoch identifier, parent hash, count and checksum; body of canonically serialized rows; footer with the epoch hash), a sequencer folding validated writes into the open epoch, hash chaining, a seal-then-publish ordering of fsync, chain append and visibility advance, and verify-on-read for migrated prefixes. The admission pipeline enforces idempotency against the declared window, the commit rule (for ledgers, per-currency zero-sum in exact integer minor units — floating point never touches amounts), and authorization capabilities.

Two design decisions are worth recording because they follow from the theory rather than from convenience. First, **no compaction of the base**: Proposition 3.4 makes full retention necessary for reconstructibility, so the storage engine may migrate and re-encode segments but may never merge away rows. This is why a general-purpose LSM engine is not used for the base even though one is appropriate for cold *view* state. Second, **resolution by append**: holds are resolved by appending a resolution row rather than by updating the pending row (Section 3.19), which keeps the base free of any update path at all — the same choice the strictest production ledgers make.

**Nilestream-Core.** Implements the REV runtime: resident maps holding absence-lattice slots; per-key anchor indices (epoch-skip structures) that discharge Theorem 4.3′'s access assumption in practice rather than in idealization; the apply loop consuming sealed epochs through compiled delta circuits; the upquery engine evaluating circuits in pull mode along provenance-derived paths against a frozen prefix; the materialization optimizer of Contribution 5; and the lineage emitter.

The divergences from prior partial-state dataflow are exactly the ones the theory requires: entries are versioned; upqueries name an epoch; deletions are negative-weight deltas rather than tombstone special cases; eviction requires no downstream notification; and eviction policy is cost-model-driven rather than randomized. The expected engineering dividend — an entire class of protocol machinery that does not need to exist (§4.2) — is a *prediction* of the design, and Phase 2 (Section 8.3) is where it is confirmed or refuted.

**Recovery.** On restart, the durable prefix is scanned to the last sealed segment, the chain is verified forward from the last trusted digest, frontiers are re-derived, and views restart cold, with checkpoints as optional warm-start optimization that correctness never depends on.

## 7.2 The Niles Compiler

A classical pipeline — lexer, recursive-descent parser with error recovery, name resolution, type-and-effect checking, IR lowering — with three checkers doing the thesis-critical work.

**The currency-row solver** implements T-Posting: transaction bodies are elaborated to per-currency sums over linear posting halves, and each row must cancel syntactically or via amounts proved equal by an exact integer constant evaluator. Failures name the unbalanced currency and the residual amount. This is expected to be the most-used diagnostic in practice, and its message quality is treated as a first-class design concern rather than an afterthought — a compile error that a developer cannot act on is a runtime error with extra steps.

**The effect checker** implements the rows of Section 4.5: consistency demands propagate up call graphs, `serve` contracts are checked against demands at view boundaries, idempotency windows are checked at submission sites, and confidentiality flows are checked against static labels.

**The IR verifier** is a small, independent type checker for circuits that re-checks all compiler output at load time, keeping the compiler outside the trusted base (Section 6.14). It is the one component where duplication of logic is deliberate: the verifier must not share code with the lowerer, or it would validate the lowerer's assumptions rather than the IR's rules.

**Surfaces.** The SQL frontend parses the stated fragment and lowers to the same IR, with golden-file α-equivalence tests enforcing Theorem 4.6(c) as a build gate. Embedded APIs construct IR directly and are verified identically.

## 7.3 The Server and Wire Compatibility

The server assembles the spine into a deployable daemon: connection handling, authentication, session state (the watermarks implementing ℓ₁ and ℓ₂), the native protocol, and the two compatibility surfaces.

**MySQL and PostgreSQL wire compatibility** exist for one reason: incremental adoption. An institution cannot replace a core banking estate atomically, so the design's viability depends on unmodified clients, drivers and reporting tools connecting to Nilestream and working. Both adapters translate a documented dialect subset into IR.

**Compatibility policy.** The supported subset is documented and tested against a recorded-traffic corpus, and **everything outside it fails loudly with a named unsupported-feature error**. Silent divergence is treated as a correctness bug, not a compatibility gap. This is the operational counterpart of Theorem 4.6(c)'s fragment honesty: a translation theorem about a stated fragment is only meaningful if constructs outside the fragment are refused rather than approximated.

**Anchors across the wire.** Because every answer carries its epoch, the adapters expose the anchor through protocol-appropriate channels (a session variable and an optional message attribute), so that an application which cares about "as of when" can obtain it without abandoning its existing driver. An application that ignores it gets ordinary SQL behaviour at the view's declared contract.

**Observability and audit endpoints** export the frontier gauges, hit/miss/upquery counters, reconstruction-latency distributions, mode transitions with their causes, and the audit operations of Section 6.15 — chain verification, reproduction of a published answer, and impact analysis. These are the instruments Chapter 9's protocols read; building them alongside the features rather than afterwards is what makes the evaluation possible at all.

## 7.4 What Exists Today

Every row marked **built** is compiled and tested by `cargo test --workspace`, which currently runs **201 tests**. Every figure in this table is produced by the build rather than typed by hand.

| Component | Status |
|---|---|
| **Keyword registry** (`niles-lang::keywords`) | **Built** — 174 keywords on four axes; the single source of truth from which the lexer, the reserved list and Appendix B.19 are all generated |
| **Normative grammar** (`grammar/niles.ebnf`) | **Built** — 205 rules, 496 productions, with a drift test checking it against the registry and the compiler in both directions |
| **Generated keyword reference** (`docs/keywords.md`) | **Built** — regenerated from the registry, with a blessing test that fails if the two disagree |
| **Lexer** | **Built** — two-layer, lossless, covering money with per-currency scale, both temporal axes, epochs and durations |
| **Parser** | **Built** — hand-written recursive descent with Pratt expressions; resilient, and total on arbitrary input |
| **Resolver and catalog** | **Built** — epoch-anchored; discharges the declaration-level well-formedness rules W1–W3, W5, W9–W12, W19 |
| **Currency-row solver** | **Built** — conservation and currency safety decided statically, with an honest *undecided* verdict where an amount is opaque |
| **Consistency-effect calculus** | **Built** — effect rows, rung monotonicity, capability requirements, linearity for holds and posting halves |
| **Lowering to the typed IR** | **Built** — the pipeline surface and the SQL surface lower to the same circuit, under test |
| **Typed IR, verifier, upquery-path derivation** (`niles-ir`) | **Built** — the operator set, the accessed-field discipline, and a verifier in the trusted base |
| **`nilesc` driver** | **Built** — `check`, `parse`, `explain`, `verify`, `upquery`, `effects` |
| **REV runtime** (`nilestream-core`) | **Built** — the absence lattice, anchored upqueries, eviction, counted work; executes the key-aggregate fragment |
| **Durable write path** (`nilestream-ledger`) | **Built** — CRC-checked, hash-chained, length-prefixed segments; crash recovery that truncates at the first bad record; tamper and splice detection |
| **Concurrent sequencer** | **Built** — single sealer with group commit; publishes the frontier only after fsync; idempotency across and within a batch |
| **End-to-end runner** (`nilestream`) | **Built** — compiles a `.niles` file, verifies the circuit, installs it, runs a workload, reports counted work |
| Reference oracle (Appendix F) | **Built and tested** — 17 tests: balance folding, bitemporal queries, per-currency conservation, idempotency, FX atomicity, chain verification, tamper detection |
| Research prototype and experiment harness | **Built and run** — E1–E10 in `results/`; E11–E13 through the compiled path |
| Optimizer cost rules, IR contract types | **Built and tested** |
| Hash chaining | Built with a **placeholder hasher** (ADR 0002) — the API is drop-in for a cryptographic one |
| Query planner beyond lowering | Partial: lowering fixes the circuit; no cost-based join ordering |
| Wire protocols, server daemon | Specified in §7.3; **not built** |
| Distributed execution, consensus, cross-shard commit | Specified in §8.5–8.6; **not built** |
| Self-hosted compiler (Appendix E, stages 1–3) | Specified; **not built**, and Appendix E.0 says so |

## 7.5 What Is Deliberately Not Built, and What That Costs the Claims

Four gaps, each with the claim it withholds.

**No distribution and no consensus.** Everything measured is single-node. The cross-shard commit protocol of §8.6 is a design; nothing in this thesis measures it. This withholds every claim about scale-out, and it is why Theorem 4.2's frontier is located here in counted work rather than in nodes.

**No wire protocol.** MySQL and PostgreSQL compatibility is the adoption argument of §6.9, and it is unbuilt. This costs the thesis nothing *theoretical* — the IR is the contract, not the protocol — but it means the incremental-adoption story is an argument rather than a demonstration.

**The executable IR fragment is narrower than the emitted IR.** The compiler lowers and the verifier accepts joins, fixpoints, set operations and ordering stages; the runtime executes source → optional filter/map → one keyed aggregate. The runtime **rejects** anything else rather than mis-executing it, which is the right failure, but it means the measurements are about the balance-shaped view and not about arbitrary queries. Chapter 9 says so at every table.

**No self-hosting.** Appendix E's three-stage bootstrap needs a stage-0 compiler that accepts the *whole* language; the one that exists accepts a large proper subset, with no trait solver, no monomorphisation and no native code generation. Stage 1 therefore has no input, and no line of the Niles-written compiler has been written. The expressiveness claim that self-hosting would test is consequently untested.

The phased program of Chapter 8 states, for each subsequent phase, the kill criteria that would end it.
