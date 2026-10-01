# 7. Implementation Design and Current Status

This chapter records what is specified, what is built, and what is not. That boundary moved substantially during the work, and this chapter is where the current position is stated once and plainly, so that no reader has to infer it from an appendix's tense.

**Status.** The artifact accompanying this thesis contains a **working stage-0 compiler** for a large subset of Niles, a **typed IR with a verifier**, a **REV runtime** that executes compiled circuits, a **durable, concurrent ledger write path**, and a **research prototype** with an experiment harness. The loop from Niles source text to a measured result is closed end to end and is reproducible by a stranger with `cargo`. What remains unbuilt is stated in §7.5 with equal precision: there is no distributed execution over a network (consensus and cross-shard commit exist only in a deterministic simulator, and are stopped), no MySQL listener (a codec, and nothing calls it), no back end and so no self-hosted one, and the executable IR fragment is narrower than the IR the compiler emits. The PostgreSQL wire protocol is built; an earlier version of this sentence said there was no wire protocol, while §7.5 described the one there is.

**Where each component stands**, rendered from `thesis/scope.toml` (§1.9):

<!-- BEGIN:scope-table.7 thesis/scope.toml#scopetable -->

*Generated from `thesis/scope.toml`. Do not edit by hand.*

| Component | Status | Decision | What exists | Described in |
|---|---|---|---|---|
| The engine: PostgreSQL with the append-only ledger, the REV mechanism and the checks | **product** | DB-1 (cycle 16): PostgreSQL is the product engine, whatever the engine comparison (H-E1) finds; DB-11: extended in SQL and PL/pgSQL first, with a compiled extension only where a measurement shows SQL is the bottleneck | E14 rebuilt the REV mechanism in stock PostgreSQL 16 (§6.10.3), and it runs as E27's P+ arm (crates/comparator); crates/rev-sidecar is the hybrid H3 (a PostgreSQL ledger with a REV sidecar). No packaged PostgreSQL ledger schema ships yet | §6.10.3, §9.14, results/E27-comparator.md |
| The language: PostgreSQL's SQL and PL/pgSQL with a checked dialect | **product** | DB-8 (cycle 16, confirmed by the author 2026-10-01), on the measurements H-L1 (E14 counterproposal) and H-L2 (E30b') | crates/nilescheck-sql: a catalog checker reading annotations from PostgreSQL comments, with linearity, conservation through niles-lang's solver, capabilities, declared effects and body typing. Enforcement at DDL time is not built (cycle 16, C16-10) | §6.10.3, §9.11, results/E14-minimal-counterproposal.md, results/E30b-syntax.md |
| Niles: the full grammar, type system and consistency-effect calculus | **reference** | H-L1 retired the grammar to a specification; DB-8 made Niles the specification of the dialect's semantics and the reference checker it is tested against. It does not replace SQL | crates/niles-lang (the stage-0 front end, the checker and lowering to the IR), crates/nilesc, crates/niles-interp | §6, Appendix B, §4.5 |
| Nilestream: the ledger write path, the REV runtime and the reference oracle | **instrument** | A-3 kept it (2026-09-21); DB-1 made it the theory's instrument and reference REV implementation, not the product engine | crates/nilestream-ledger, crates/nilestream-core, crates/conservation-suite, crates/proto-engine | §7.1, §7.4, Appendix D, Appendix K |
| Nilestream's PostgreSQL wire surface | **instrument** | A-3 kept it (2026-09-21) | crates/nilestream-server: PostgreSQL protocol v3, simple and extended query paths, driven by psql in conformance tests; no binary result format and no SCRAM | §7.3, §7.5, Appendix D.4 |
| GBS: a core-banking system in Rust, whose product goal is to replace Loan IQ and Calypso | **product** | DB-4 (GBS's backend is PostgreSQL), DB-5 (replacing Loan IQ and Calypso is the product goal; the thesis evaluates only what is built), DB-10 (pricing and risk through ORE as a separate process) | a separate, private repository: the kernel, the mechanisms and the products, with a Nilestream adapter held by its G3 correctness suite. Its PostgreSQL backend and the G3p suite are cycle 16's C16-05, not yet run | §11.4 |
| The MySQL wire listener ("the MySQL replacement") | **specification and future work** | A-3 (2026-09-21) stopped the listener | a packet codec only (crates/nilestream-server/src/mysql_wire.rs, with tests); nothing calls it and no socket accepts MySQL clients | §7.3, §11.5.4, Appendix D.4 |
| The native Nilestream frame protocol | **specification and future work** | never scheduled; Appendix D.4 has described it as specification since cycle 9 | a one-line module stub, crates/nilestream-server/src/native_proto.rs, and nothing else | Appendix D.4 |
| Distributed execution, consensus and the cross-shard commit protocol | **specification and future work** | A-3 (2026-09-21) stopped consensus and distribution | a single-process deterministic simulator: crates/nilestream-consensus (a replicated ledger group and two-phase commit over groups, cross_shard.rs) and crates/nilestream-core/src/distributed.rs. None of it has run over a network | §8.5, §8.6, §8.7, Appendix G.8 |
| The self-hosting compiler | **specification and future work** | A-3 (2026-09-21) stopped the bootstrap | stage 0 and the first front-end stage only: crates/niles-interp (an interpreter for the imperative subset), bootstrap/lexer.niles and bootstrap/parser.niles. Nothing from Appendix E.6 onward | Appendix E |
| Compilation targets: native Linux and macOS on ARM64, Windows on x86-64, WSL and WASM | **specification and future work** | A-3 (2026-09-21), with cycle 14's D-3 keeping the chapter as specification | nothing: nilesc lowers to the typed IR and has no back end | Appendix C, Appendix G.6, Appendix G.7 |
| The online adaptive materialization optimizer (C5) | **specification and future work** | A-3 (2026-09-21) shrank the optimizer to the planner and stopped the online optimizer | the planner kept by A-3 (crates/nilestream-optimizer: unnesting, join ordering, three eviction rules); no estimator, no hysteresis schedule, no rent-or-buy break-even | §4.6, Appendix I |
| Committed (homomorphic) commitments for confidential columns | **specification and future work** | A-3 (2026-09-21): confidentiality labels stay and are checked; the commitments stop | the labels and their flow check in crates/niles-lang; no commitment scheme | §6.20, §6.21, §6.22, Appendix B.13 |

<!-- END:scope-table.7 -->

Everything described below that is *not* marked as built is written as an implementation plan with its design decisions and their justifications. No claim of completed performance work is made anywhere in this thesis beyond the measurements Chapter 9 reports and labels.

## 7.1 Nilestream-Core and the Ledger

**Ledger crate.** Implements Definition 3.1 directly: a segment format whose record header carries a length and its own check word, a batch sequence number and the parent hash, with the body's checksum and the chain link over it, a sequencer folding validated writes into the open epoch, hash chaining, a seal-then-publish ordering of fsync, chain append and visibility advance, and verify-on-read for migrated prefixes. The admission pipeline enforces idempotency against the declared window, the commit rule (for ledgers, per-currency zero-sum in exact integer minor units — floating point never touches amounts), and authorization capabilities.

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

**PostgreSQL wire compatibility** exists for one reason: unmodified clients, drivers and reporting tools connect to Nilestream and work, so the instrument and the product engine, which is PostgreSQL, can be driven by the same clients and compared over the same protocol. The adapter translates a documented dialect subset into IR. **MySQL wire compatibility was designed for the same reason and is specification and future work:** decision A-3 stopped the listener, and only a packet codec exists (§7.5, `thesis/scope.toml`). An earlier version of this paragraph described both adapters as existing for incremental adoption by an institution replacing its core banking estate; the product now meets that need by being PostgreSQL.

**Compatibility policy.** The supported subset is documented and tested against a recorded-traffic corpus, and **everything outside it fails loudly with a named unsupported-feature error**. Silent divergence is treated as a correctness bug, not a compatibility gap. This is the operational counterpart of Theorem 4.6(c)'s fragment honesty: a translation theorem about a stated fragment is only meaningful if constructs outside the fragment are refused rather than approximated.

**Anchors across the wire.** Because every answer carries its epoch, the adapters expose the anchor through protocol-appropriate channels (a session variable and an optional message attribute), so that an application which cares about "as of when" can obtain it without abandoning its existing driver. An application that ignores it gets ordinary SQL behaviour at the view's declared contract.

**Observability and audit endpoints** export the frontier gauges, hit/miss/upquery counters, reconstruction-latency distributions, mode transitions with their causes, and the audit operations of Section 6.15 — chain verification, reproduction of a published answer, and impact analysis. These are the instruments Chapter 9's protocols read; building them alongside the features rather than afterwards is what makes the evaluation possible at all.

## 7.4 What Exists Today

Every row marked **built** is compiled and tested by `cargo test --workspace`, which currently runs **401 tests**. Every figure in this table is produced by the build rather than typed by hand.

| Component | Status |
|---|---|
| **Keyword registry** (`niles-lang::keywords`) | **Built** — 176 keywords on four axes; the single source of truth from which the lexer, the reserved list and Appendix B.19 are all generated |
| **Normative grammar** (`grammar/niles.ebnf`) | **Built** — 206 rules, 497 productions, with a drift test checking it against the registry and the compiler in both directions |
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
| **Durable write path** (`nilestream-ledger`) | **Built** — hash-chained segments whose eight-byte record header checks itself (`len_check = !len ^ 0xA5A5_A5A5`) and whose CRC covers the body; recovery that separates a torn tail from damage and truncates only the former; tamper and splice detection. The self-check is cycle 9's: the checksum does not cover the length, so a flipped bit in a length prefix made a record look longer than the file, recovery called that a torn tail, and a ten-record segment reopened empty with a success code. The corpus at `crates/nilestream-ledger/tests/torn_corpus.rs` is exhaustive over every bit of a ten-record fixture; the one class it cannot decide — damage to the *last* record, which no file can tell from a crash during its write — is `LC-36` / `BLOCKED-recovery-tip` |
| **Concurrent sequencer** | **Built** — single sealer with group commit; publishes the frontier only after fsync; idempotency across and within a batch |
| **End-to-end runner** (`nilestream`) | **Built** — compiles a `.niles` file, verifies the circuit, installs it, runs a workload, reports counted work |
| Reference oracle (Appendix F) | **Built and tested** — 17 tests: balance folding, bitemporal queries, per-currency conservation, idempotency, FX atomicity, chain verification, tamper detection |
| Research prototype and experiment harness | **Built and run** — E1–E10 in `results/`; E11–E13 through the compiled path |
| Optimizer cost rules, IR contract types | **Built and tested** |
| Hash chaining | Built with a **placeholder hasher** (ADR 0002) — the API is drop-in for a cryptographic one |
| **Materialization planner** (`nilestream-optimizer::offline`) | **Built** — filters by contract, then prices; every plan explains itself and names what it ruled out |
| **PostgreSQL wire protocol** (`nilestream-server`) | **Built** — v3 startup and simple query, over a real socket; `nilestreamd` answers `psql` |
| **Replicated ledger groups** (`nilestream-consensus`) | **Built** — elections, hash-linked replication, quorum commit, deterministic simulator |
| **Cost-based join ordering** (`nilestream-optimizer::join_order`) | **Built** — `DPccp` up to twelve relations, greedy above; a three-term cost model adding resident state and reconstruction depth to the classical flow term, with reconstructibility as a *legality* constraint rather than a cost |
| **Extended query protocol** (`nilestream-server::extended`) | **Built** — parse/bind/describe/execute, with the epoch-keyed plan cache that answers the design question §7.5 previously recorded as open; the binary format is still refused by name |
| **MySQL wire** (`nilestream-server::mysql_wire`) | **A codec, and no listener.** Packet framing with sequence numbers, length-encoded integers, handshake, `OK`/`ERR`, column definitions and text rows are built and tested, and money is `NEWDECIMAL`, never `DOUBLE`. **Nothing calls them**: no socket is bound, so no MySQL client has ever connected, and H-S5's client-compatibility half has no pass rate for that reason. The crate's own module table says the same thing, and this row used to say "Built" |
| **TLS negotiation and policy** (`nilestream-server::tls`) | **Built** — both negotiation state machines, the four-way policy, `libpq`'s six `sslmode` values, and the MySQL downgrade defence. The cryptography is **delegated** behind a provider trait and is not implemented here; a `Require` policy with no provider **fails at startup rather than serving cleartext** |
| **Distributed read path** (`nilestream-core::distributed`) | **Built** — a versioned shard map, the strict frontier as the *minimum* across shards, one round trip per shard, and a `(key, epoch)` cache that needs no invalidation because a frozen prefix cannot change |
| **Cross-shard commit** (`nilestream-consensus::cross_shard`) | **Built** — two-phase commit whose coordinator *is* a ledger group; per-currency conservation checked across fragments; the decision refused before it is durable; a commit epoch strictly above every prepared epoch, so visibility at an anchor is identical on every shard |
| **Stage-0 execution and bootstrap gates** (`niles-interp`, `bootstrap/lexer.niles`) | **Built** — a tree-walking interpreter for the imperative subset that refuses the relational tier by name, a lexer written in Niles, and four gates: stage-1 run, stage-1 equivalence against the reference lexer, stage-2 self-application, stage-3 fixpoint |
| Binary result format, SCRAM authentication | **Not built**; each refusal names its reason |
| Membership change, log compaction | Specified in §8.6; **not built** |
| Self-hosted **compiler** (Appendix E, stages beyond the lexer) | A parser, type-checker and lowering pass in Niles remain **unwritten**; Appendix E.0 and E.19.1 say precisely how far the bootstrap reaches |

## 7.5 What Is Deliberately Not Built, and What That Costs the Claims

Four gaps, each with the claim it withholds.

<!-- BEGIN:scope-distributed.7 thesis/scope.toml#scopeline -->

*Generated from `thesis/scope.toml`. Do not edit by hand.*

> **Scope: specification and future work.** Distributed execution, consensus and the cross-shard commit protocol. *Decision:* A-3 (2026-09-21) stopped consensus and distribution. *What exists:* a single-process deterministic simulator: crates/nilestream-consensus (a replicated ledger group and two-phase commit over groups, cross_shard.rs) and crates/nilestream-core/src/distributed.rs. None of it has run over a network. Nothing in this chapter is evaluated; it is a design.

<!-- END:scope-distributed.7 -->

**The distributed path is built as a protocol and unbuilt as a deployment, and the distinction is the whole of what §7.5 can honestly claim.** `nilestream-consensus` implements a replicated ledger group; `cross_shard` implements two-phase commit over several such groups; `nilestream-core::distributed` implements the sharded read path. Each is tested in a deterministic simulator — 25 seeds at 20% loss and 30% reordering for consensus, and property tests over the shard map and the commit protocol — and **not one of them has run over a network.** A protocol that has only ever been exercised in-process has not met partial failure, clock skew, or an operator, and the standard result is that the first contact with a real network finds something. Every performance figure in Chapter 9 remains single-node, and Theorem 4.2's frontier is located in counted work rather than in nodes.

Two design results from that work are worth stating independently of their deployment status, because they are arguments rather than code. First, **the classical objection to two-phase commit dissolves when the coordinator is itself a ledger group**: 2PC blocks when a single coordinator fails between prepare and decide, and a replicated coordinator that persists its decision through quorum before sending it has no such window — the successor reads the decision rather than re-deciding it. Second, **a cross-shard upquery needs no coordination at all**, because it reads a frozen prefix: the prefix cannot change, so the read cannot be made stale by a concurrent write, and its result can be cached without an invalidation protocol. That is the anchoring argument of Theorem 4.1 paying a distributed dividend, and it is the clearest case in this thesis of an immutability commitment buying something a mutable design would have had to coordinate for.

**The PostgreSQL wire protocol is built, and it is a surface rather than semantics.** `nilestreamd` speaks PostgreSQL wire protocol v3 — the simple query path over a real socket, and now the extended path. The MySQL packet layer is implemented alongside it as a codec that nothing calls; its listener is specification and future work (decision A-3). An earlier version of this paragraph began "Both wire protocols are built". There is deliberately **no compatibility layer with its own execution path**: every query on every surface is parsed as Niles, lowered to the same IR, verified by the same verifier and served from the same REV runtime, because two ways to compute an answer is two answers that can disagree.

The extended protocol's open design question is now **answered**. A prepared statement must be cached against the epoch it was planned at, since a plan valid at one visibility frontier need not be valid at another; the resolution is an epoch-keyed plan cache in which a plan is valid iff no schema epoch occurred after the one it compiled at — one integer comparison — and a stale entry returns a request to recompile rather than an error, so a schema change costs a round trip and never a failed statement.

What remains unbuilt on this path is narrower and is named: the binary result format, and SCRAM authentication. **TLS is a special case and is described as one.** The negotiation, the policy layer, `libpq`'s six `sslmode` values with a predicate recording which of them actually authenticate the server (only `verify-ca` and `verify-full` do), and MySQL's server-side downgrade check are implemented and tested; the *cryptography* sits behind a provider trait and is delegated to a vetted implementation. That is not a shortcut and the thesis does not present it as one: a record layer written from scratch fails silently against an adaptive attacker, and a thesis about static checkability of financial invariants has nothing to gain from shipping its own. The design commitment that matters is that a `Require` policy with no provider **fails at startup rather than serving cleartext** — the same principle as `Hole` versus zero in the absence lattice, applied to a socket. The transcript is in `results/wire-protocol-session.md`, and it is compared byte for byte on every host that runs the reproduction gate: two of them, running `psql` 16.13 on Ubuntu and `psql` 18.6 on macOS, produced identical transcripts and differed only in the client's version banner, which is why that banner now lives beside the transcript in `results/wire-protocol-client.md` rather than inside it. Agreement across two major versions of an independent client is a stronger statement about the framing than any single run could be.

**The executable IR fragment is narrower than the emitted IR.** The compiler lowers and the verifier accepts joins, fixpoints, set operations and ordering stages; the runtime executes source → optional filter/map → one keyed aggregate. The runtime **rejects** anything else rather than mis-executing it, which is the right failure, but it means the measurements are about the balance-shaped view and not about arbitrary queries. Chapter 9 says so at every table.

**Partial self-hosting: one front-end stage, not a compiler.** Appendix E's bootstrap now has stage-0 *execution* — a tree-walking interpreter for the imperative subset — and stage 1 has an input: a lexer written in Niles, which agrees with the Rust lexer token for token over a 29-case corpus, lexes its own source identically, and is byte-identical run to run. That gate rejected four real defects on its first run, including a case-sensitivity error that arises precisely because Niles's two ancestries disagree (SQL's keywords are case-insensitive, Rust's are not).

What is still absent is the rest of the compiler: no parser, type-checker or lowering pass exists in Niles, so E.1's stage 2 — "stage 1 recompiling the same sources" — has no meaning yet, and the expressiveness claim that full self-hosting would test remains untested. The bootstrap has moved from *no input* to *one verified front-end stage*, which is a first rung; E.19.1 states the distance to the next one.

The phased program of Chapter 8 states, for each subsequent phase, the kill criteria that would end it.
