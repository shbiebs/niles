# 7. Implementation Design and Current Status

This chapter records what is specified, what is built, and what is not. **Status, stated once and plainly: the artifact accompanying this thesis is a workspace scaffold with the executable reference oracle (Appendix F) implemented and passing its tests. The engine, compiler and server described below are designs with stub crates, not measured systems.** Everything in this chapter is therefore written as an implementation *plan* with its design decisions and their justifications; nothing here reports a measurement, and no claim of completed performance work is made anywhere in this thesis.

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

| Component | Status |
|---|---|
| Reference oracle (Appendix F) | **Implemented and tested** — balance folding, bitemporal queries, per-currency conservation, idempotency rejection, FX-atomicity, chain verification, tamper detection |
| Workspace, crate structure, IR type skeletons, contract types | Scaffolded, compiles |
| Ledger write path, Nilestream-Core, compiler, server | Specified in this thesis and Appendices B–E; stub crates only |
| Benchmark harness, phase-diagram experiments | Specified in Chapter 9 and Appendix G; not run |

The phased program of Chapter 8 states, for each subsequent phase, the kill criteria that would end it.
