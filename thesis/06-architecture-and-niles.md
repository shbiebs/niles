# 6. System Architecture and the Niles Language

This chapter describes the instrument: Nilestream's architecture and the design of Niles. Normative references live in the appendices (B: syntax; C: compilation; D: engine and IR; E: self-hosting compiler; H: SQL-completeness dossier; I: optimizer algorithms). Implementation status is Chapter 7; nothing here should be read as a report of a completed system.

## 6.1 The Build Spine

The system is one spine with four segments, each consuming the previous segment's output and nothing else.

1. **The ledger write path** — admission, validation against the commit rule, idempotency, authorization, sequencing, epoch sealing, hash chaining, durability.
2. **The typed intermediate representation** — a DBSP-style circuit language with anchors, effects, contracts and provenance in its types, and the *stable contract between the language and the engine*.
3. **The read-model runtime (Nilestream-Core)** — REVs executing IR circuits with partial state, eviction, anchored upqueries, the materialization optimizer, and lineage.
4. **The serving edge** — the native Niles protocol plus MySQL and PostgreSQL wire compatibility.

The spine discipline is architectural law: the read side may consume only sealed epochs — never the open epoch, never another view's internals except through declared IR edges. This is what makes Chapter 4's proofs apply to the implementation as built rather than as idealized.

The IR's role deserves emphasis because it is a design commitment rather than a convenience. Putting a typed IR between surface and engine follows the progressive-lowering philosophy of modern compiler infrastructure, where high-level semantics are retained until an optimization no longer needs them and then lowered one level at a time [Lattner et al., CGO 2021]. Here the semantics that must survive lowering are exactly the ones the theorems quantify over: anchors, contracts, effects and provenance. An IR that erased them would leave the theorems talking about a different object than the one that runs.

## 6.2 Ledger Write Path and Read-Model Engine

**Write path.** Clients submit transactions natively or through the wire protocols. Admission checks the idempotency key against the declared deduplication window (Section 3.20), returning the recorded outcome on a matching repeat and a conflict error on a fingerprint mismatch. Validation type-checks against the schema and evaluates the commit rule — for a ledger, per-transaction per-currency balance in exact integer minor units, plus authorization capabilities for flagged effects. Sequencing folds validated transactions into the open epoch in one serial order. Sealing closes the epoch on the τ boundary or size bound, computes its hash, makes it durable, and publishes visibility. Everything after the seal is immutable.

**Read-model engine.** Each REV holds its resident map (keys at points of the absence lattice), a per-key anchor index, its compiled circuit with provenance-derived upquery paths, and its contract. Application advances resident keys incrementally through delta circuits. Reads follow the protocol of Section 3.13. Eviction and mode selection are the optimizer's (Section 6.16). Lineage is emitted at the view's declared mode.

**Why the two halves can be maximally different.** The write path is where coordination lives and is deliberately conservative; the read path is where economics live and is deliberately adaptive. This asymmetry is not a compromise between them but a consequence of Proposition 3.2: the invariants that require coordination are enforced at admission, and everything downstream inherits their truth without paying for it again.

## 6.3 Reasons to Build a Custom Runtime, and Where to Reuse

**Build:** the epoch and anchor machinery (exists nowhere with these semantics); partial state with versioned holes (existing partial state is unversioned and eventually consistent); the ledger commit path (databases hide their logs, whereas here the log is the product); the IR (the proofs need its semantics pinned); the materialization optimizer (no existing optimizer decides under a consistency contract).

**Reuse:** established consensus implementations for the replicated profile rather than a bespoke protocol; standard cryptographic primitives, never hand-rolled; wire-protocol framing; an LSM library for cold view state (though *not* for the ledger itself, whose compaction would rewrite data the design declares immutable); and DBSP's operator algebra as the formal and, where license-compatible, practical basis of the IR executor.

The rule: reuse where semantics are commodity, build where the thesis's claims live.

## 6.4 Durability and the Migration Boundary

An epoch is durable when its segment and hash are synchronized to the primary medium and, in replicated profiles, acknowledged by a quorum; visibility never exceeds durability. Storage is tiered — hot segments on fast local media, sealed history migrating to cold object storage — and the **migration boundary is semantic, not merely operational**: bytes may move media, but identity is the hash chain, so a migrated prefix re-verifies on read and the audit guarantee survives storage evolution. This matters commercially as well as formally: immutable ledgers grow without bound, and published industrial experience is that ledger growth drives cold-storage offload as a primary cost concern.

Recovery replays from the last durable epoch: frontiers are re-derived, resident maps restart empty (checkpoints are pure optimization, never trusted), and Theorem 4.1 guarantees rebuilt views are exact. **Recovery is a large eviction** — which is why the same property tests cover both.

## 6.5 The General Core of Niles, and Coverage of Workload Classes

Niles is a general-purpose data language whose core is typed relations, expressions, functions and views over the relational algebra, with Rust-derived syntax for items and expressions and SQL vocabulary for query stages where Rust has no equivalent. A schema declares three kinds of object:

```niles
schema bank {
    table customers { id: Id<Customer>, name: Text, opened: Date }

    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        value_date: Date, idem: IdemKey,
        conserve per (txn, cur);
        retain forever;
    }

    view balances = postings
        .group_by(|p| (p.acct, p.cur))
        .sum(|p| p.amt)
        serve { consistency: bounded(2.epochs, 5.s), materialize: demand, retain: evictable };
}
```

`table` is ordinary relational data; `ledger` is the authoritative append-only base with a commit rule; `view` is a REV with a contract. Everything else — modules, generics, pattern matching, `let`, `fn` — is Rust-shaped.

**Workload classes on one core.** The generality claim (S8, C6) is discharged by construction, and the constructions are these:

| Class | Construction |
|---|---|
| **OLTP** | Committed writes to the base plus point-lookup REVs at ℓ₂–ℓ₅. |
| **OLAP / HTAP** | REVs whose circuits aggregate; the optimizer chooses column-shaped resident representations and `full` or `tiered` modes for wide scans. The row/column duality that HTAP systems obtain by dual formats over a mutable table is obtained here by multiple REVs over one immutable base — the same idea with a stronger correctness story. |
| **Time-series** | A REV over a base partitioned on the valid-time axis, with compressed resident representations; the published delta-of-delta and XOR schemes are the reference design for that compression. |
| **Document** | A typed semi-structured column with path-indexed resident state, following the decomposed-binary and tree-of-paths designs that production document stores use. |
| **Graph** | Guarded recursion over an edge base — the same fixpoint construct that gives Theorem 4.6(b) — with pattern syntax aligned to the standardized property-graph vocabulary. |
| **Search** | An inverted-index REV whose resident state is immutable segments with tombstones and background merging; that this architecture serves search is not a hypothesis but two decades of production evidence. |

The point is not that one storage format serves everything — it does not — but that one *semantics* does. Format, layout and residency are the optimizer's choices under Theorem 4.5(a), which proves them semantics-preserving.

## 6.6 How Banking Was Made First-Class Without Making It Special

Banking is a library over the general core, not a fork of it. A standard module `std::bank` defines accounts, postings, transactions, `Money⟨cur, scale⟩`, holds, authorization capabilities and the double-entry discipline, all expressed with type-system features available to any user library: currency-indexed types, linear halves, effect rows, and commit rules.

The compiler knows nothing about money as such; it knows about indexed monoid rows, linear consumption and declared commit rules, and `std::bank` instantiates them. This is the falsifiable sense of "general core": Section 9.11 builds a non-financial conservation domain — inventory with serial-number conservation — using the same features and no kernel changes, with the change log audited as the falsifier.

## 6.7 Balance as a Function of Time

There is no stored balance anywhere. `balance(acct, cur)` is a temporal signal — a function from epoch to `Money⟨cur⟩` — defined as the running sum of the postings signal; available balance subtracts unresolved holds. Reads of "the balance" are reads of this signal at an anchor; statements are integrals over valid-time windows; interest accrual is an operator on the signal.

This one decision eliminates the classic dual-write bug — a stored balance drifting from its postings — by making it *inexpressible*, and it is what lets Corollary 4.1.1 apply to the thing institutions actually serve. It is also what makes the APSN hazard analyzable: the available and ledger balances are two REVs over one base, differing only in which rows their circuits admit, so their divergence is a property of contracts and anchors rather than an emergent mystery.

## 6.8 The Temporal Signal Library

`std::temporal` generalizes the above: signals (epoch-indexed values), windows, `as_of(epoch)`, valid-time selection and intervals, `latest`, accrual and decay combinators, and bitemporal join forms. All are **epoch-deterministic**: no wall-clock read occurs inside query evaluation, and "now" enters only as the anchor chosen by the contract. That restriction is what keeps reconstruction pure and cross-target execution deterministic, and it is enforced by the effect checker rather than by convention.

## 6.9 Three Surfaces, One Intermediate Representation

Niles native, the SQL surface, and the imperative Rust-shaped sublanguage all lower into one typed IR. The equivalence obligation is stated as a theorem and tested as a build gate: an SQL query in the stated fragment and its Niles counterpart lower to α-equivalent circuits (Theorem 4.6(c), Appendix H). The IR, not any surface, is what the theorems and the runtime interpret.

Embedded APIs — language bindings that construct IR directly — are a fourth surface with the same status: they are checked by the IR verifier on submission, so an embedding cannot smuggle in a circuit the type system would reject.

## 6.10 The Language-Creation Gate

Creating a language is expensive, and the thesis states the gate it had to pass. A new language is justified only if: **(G1)** the required semantics cannot be expressed as a library in an existing language; **(G2)** they cannot be added to SQL without semantic dishonesty; and **(G3)** the type-level guarantees are load-bearing for the scientific claims.

**G1.** Linear posting halves, effect rows for consistency, and compile-time currency rows require machinery a general-purpose host language's trait system can partially emulate but not enforce completely or ergonomically — affine approximation instead of linearity, no effect system, and diagnostics that surface as solver artifacts rather than domain errors.

**G2.** SQL has no effects, no linearity and no anchors; dialect pragmas would be unchecked comments. This is where the strongest opposing position must be met rather than avoided. "One SQL to Rule Them All" argues that streaming needs only three additions to standard SQL — time-varying relations as the foundational abstraction, event-time semantics with watermarks, and a small set of keywords controlling materialization of results [Begoli et al., SIGMOD '19]. The argument is good, and this thesis concedes the part that is right: for *streaming semantics alone*, SQL plus those three extensions is sufficient, and Niles's query core is deliberately isomorphic to that position. The disagreement is about the other axis. Conservation, currency safety, idempotency windows, retention, confidentiality and per-view consistency are *type* obligations, not query semantics, and there is no way to add a type-and-effect discipline to SQL's grammar without changing what a well-formed program is — at which point one has a new language and should say so. The honest formulation of the gate is therefore: SQL can be extended to say *what to compute over streams*; it cannot be extended to *refuse programs that would lose money*.

**G3.** Theorem 4.4 quantifies over well-typed programs. The type system is the theorem's premise, not its packaging.

**Why not fork an existing compiler.** A fork inherits the full maintenance surface of a general-purpose toolchain while fighting it on linearity-versus-affinity and on effects, and the declarative query core is un-Rust-like at the surface in any case. Niles instead *borrows Rust's syntax shape* — familiarity without the fork — with a fresh, small, self-hosting implementation (Appendix E).

**Precedent for the decision.** In 2024 ISO published GQL as a standalone database language rather than as another SQL part, developed by the same working group that maintains SQL, while *also* standardizing SQL/PGQ as a read-only embedded sublanguage over the same graph patterns. The standards community thus treats "embedded sublanguage" and "standalone language" as genuinely different artifacts and has recently chosen the latter when the model differs enough. That is not proof that Niles is justified, but it is evidence that the question this gate asks is the right one.

## 6.11 Memory Model, Allocation, Immutability, and Memory Safety

The runtime's memory model mirrors the formal one. Sealed data is immutable: epoch segments are frozen buffers, and resident view entries are immutable values *replaced* rather than mutated on anchor advance, with structural sharing. Allocation is arena-per-epoch on the write path — an epoch's allocations free as a unit after sealing and migration — and slab-based for resident maps under the optimizer's budget.

Ownership types are the enforcement mechanism: no mutable reference to sealed data exists in the type surface, so the two mutable structures (admission queue, resident maps) are the entire audited concurrency surface. At the Niles level, `ledger` values are immutable by type, bindings are immutable by default with explicit opt-in, and user functions receive borrowed immutable views. Memory safety for user code comes from the sandbox of the UDF tier, not from trusting user code.

## 6.12 Language Scope

Niles is a data language with an imperative sublanguage, not a systems language. Three tiers:

- **Declarative tier** — schemas, bases, views, contracts. Total, optimizable, PTIME by construction (guarded recursion only).
- **Transaction tier** — imperative Rust-shaped logic with effects, where the commit rules and the banking discipline bind.
- **UDF tier** — Rust-shaped functions compiled to WebAssembly for portability and sandboxing.

Explicitly out of scope: raw pointers, unrestricted I/O in query context, ambient wall-clock reads, and unbounded recursion in views. Application code, servers and user interfaces are out of scope by design; Niles connects to host languages through the wire protocols and embedded APIs. Section 11.2 lists "seductive generality" — the temptation to grow Niles into an application language — as a named risk, and these tiers as the mitigation.

## 6.13 The Computation Tier and the Determinism Obligation

The computation tier executes IR circuits: a scheduler assigns operator work per sealed epoch, hot circuits run fused and compiled, cold circuits interpret, and upqueries execute the same circuits in pull mode along provenance-derived paths. The compilation model follows the data-centric, push-based approach in which operator boundaries disappear in generated code and tuples stay in registers between pipeline breakers [Neumann, PVLDB 2011].

**Determinism rule:** given (prefix, circuit, anchor), output bytes are identical across runs and targets. This is not a nicety — Theorem 4.1's exactness and the hash-chain's reproducibility both depend on it.

Achieving it requires closing sources of non-determinism explicitly, and one of them is a correction to naive assumptions about the UDF sandbox. WebAssembly is *not* fully deterministic: its specification names three sources of implementation-dependent behaviour — NaN payloads (canonical NaNs carry a non-deterministic sign bit, and non-canonical inputs yield non-deterministic outputs), resource exhaustion (engines may fail to grow linear memory), and host functions [Haas et al., PLDI '17]. A UDF that produces a NaN, or that exhausts memory at a device-dependent point, would break reconstruction-equivalence over a hash-chained base. The obligation is therefore stated as three concrete mitigations rather than as an assumption: ban NaN-producing floating-point operations in UDFs or canonicalize at the kernel boundary; make resource exhaustion a *deterministic* abort at a fixed fuel bound rather than a device-dependent one; and permit no host functions outside a deterministic allowlist. Within the engine itself, determinism is enforced by canonical ordering before emission, integer-exact money, and prohibition of hash-order iteration.

## 6.14 The Kernel Boundary

The trusted kernel is small: admission, validation and sealing; hash chaining; storage; frontier bookkeeping; and the IR executor's core operators. Outside it: UDFs (sandboxed, resource-metered, with no ambient authority — capabilities are passed explicitly, following the capability-based model of the Wasm system interface); surface compilers, whose output is re-checked by the IR verifier, so the kernel trusts the *verifier* rather than the compiler; the materialization optimizer, whose decisions are semantics-preserving by Theorem 4.5(a) and therefore cannot be a correctness dependency; and wire adapters, which translate and never touch state directly.

The soundness theorem's trusted computing base is thus the kernel plus the IR verifier — stated explicitly so that the S4 campaign attacks the right boundary, and so that adding a domain library or an optimizer heuristic provably does not enlarge it.

## 6.15 Lineage and Traceability

Full traceability of operations, changes and states is a system requirement (Section 1.3(4)) and is implemented as an ordinary consequence of the algebra rather than as a logging subsystem.

Three modes are declarable per view: `off` (anchors only — every answer still carries the epoch it is correct as of); `key` (which base keys contributed); and `full` (the how-provenance polynomial retained). Provenance rides the same circuits as values, because Z-sets are a semiring instance and the factorization theorem guarantees that computing the most general annotation once permits every other semantics to be recovered by homomorphism [Green et al., PODS '07].

Three operational capabilities follow. **Explain**: for any served answer, return the base rows and the derivation that produced it. **Reproduce**: recompute any published (answer, epoch) from the prefix and compare byte-for-byte — the mechanical form of an audit. **Impact**: given a base row, identify the derived entries whose provenance includes it, which is what makes correction workflows (a reversing entry) analyzable rather than hopeful. Costs are measured under S9, with the published ~30% figure for interactive dataflow lineage as the order-of-magnitude reference point rather than a target.

## 6.16 Checkpoints as a Declared View Property

Theorem 3.7 makes the per-key checkpoint interval C the constant in the reconstruction bound, and §9.4.1 measures it: at C = 16 reconstruction cost was flat at ≈ 8.5 base rows across a 64× increase in history, against a predicted C/2 + 1 = 9. C is therefore a *contract term*, declared per view alongside consistency and materialization mode, and not a hidden engine default — because choosing it is choosing a point on the reconstruction-cost/checkpoint-storage trade, and the theory prices that choice.

Checkpoints are derived state: each is recomputable from the base, so they are evictable and rebuildable and stand outside the retention guarantee, exactly as views do. This is the base/derived split of F4 applied one level down, to the reconstruction path itself.

## 6.17 The Materialization Optimizer in the Runtime

Contribution 5's algorithm lives here. Per view and key range, the optimizer maintains estimates of arrival rate, reuse distance, reconstruction cost, reconstruction *latency* and update rate; computes the rent-or-buy break-even for mode selection and a cost-and-size-aware credit for eviction; weights both by the contract multiplier Φ(ℓ) and by the delayed-hit factor; and moves ranges between modes on a hysteresis schedule.

Three properties make this safe to run continuously. It cannot change what an answer *is* (Theorem 4.5(a)). It cannot violate a contract, because contract satisfaction is a constraint of the assignment problem rather than an objective term. And it is *observable*: every mode transition is logged with the estimates that caused it, so a surprising decision is diagnosable rather than mysterious. Appendix I gives the algorithm, the estimators, and the offline dynamic program used to compute the optimum against which S7 measures it.

## 6.18 End-to-End Encryption: Threat Model and Requirements

**Requirement.** Fields marked confidential must be protected such that a compromised operator — an honest-but-curious server, stolen media, subpoenaed backups — cannot read them, while the base's ordering, hashing and conservation guarantees still hold.

**Threat model.** The adversary controls storage and can read server memory, except for designated key-holding clients or hardware modules; the adversary cannot break standard cryptography. Availability attacks and traffic analysis are out of scope, and leakage is bounded rather than eliminated (Section 6.19).

## 6.19 E2EE Versus Computation Over Data

The honest tension is that a server cannot compute over what it cannot read. Fully homomorphic encryption is rejected for the serving path on cost. The design instead *partitions fields by computability need*, and forces the partition to be declared:

- **Identity and narrative fields** (names, memos, counterparty details): true end-to-end encryption, opaque to the server, never usable in server-side predicates.
- **Structural fields** (account identifiers, currencies, epochs): pseudonymized but server-visible, because ordering and routing need them.
- **Amounts**: a deployment choice between plaintext-to-server (full functionality) and *committed* amounts using an additively homomorphic commitment scheme, which preserves the ability to verify that postings sum to zero while hiding values — since the sum of commitments opens to the sum of values [Pedersen, CRYPTO '91].

Confidentiality is a static type annotation checked like everything else: end-to-end-encrypted fields cannot appear in server-side predicates, joins or aggregates (a compile error names the leak); committed fields admit only the operations their scheme supports; and declassification requires an explicit capability that appears in the audit trail. The discipline is information-flow typing narrowed to the two lattice points the system actually offers — and narrowed deliberately: labels are **static**, because non-interference results for dynamic labels remain an open problem in that literature, and a guarantee this thesis cannot prove is one it does not claim.

## 6.20 Defence in Depth, Leakage, Limits, and Correctness Obligations

**Layers.** Transport encryption; storage encryption at rest with per-tenant keys; field-level encryption per Section 6.18 with client- or module-held keys; hash-chain integrity computed over ciphertext so tamper-evidence survives confidentiality; key rotation recorded *in the ledger itself*, so rotation is an audited event; and capability-scoped decryption in clients.

**Declared leakage.** Access patterns (which keys are read when), field sizes and cardinalities, structural fields by design, and timing. Mitigations for access-pattern leakage are out of scope.

**Correctness obligations added by encryption.** Chain verification must be ciphertext-stable across re-encryption boundaries, solved by chaining over ciphertext plus explicit rotation records. Conservation over committed amounts holds via the commitment homomorphism and is tested as an extension of the conservation suite. Recovery must not require decryption — and does not, since replay operates over ciphertext.

**Crypto-shredding, stated accurately.** Where erasure obligations conflict with retention, destroying a key renders the retained ciphertext unreadable. The most authoritative published treatment of this technique in a regulatory context states that it can "make the data practically inaccessible, and therefore move closer to the effects of data erasure" — and deliberately stops short of saying it satisfies the erasure right [CNIL, 2018]. This thesis preserves that hedge, and Section 11.1 accordingly lists legal erasure over base facts among the cases where this architecture is the wrong choice.

## 6.21 A Phased Build-and-Evaluation Guide

Each subsystem carries its falsifier from birth: the ledger ships with the floor benchmark before any read model exists; the IR ships with golden-equivalence tests before optimization; partial state ships with the conservation suite wired in; the optimizer ships with the offline dynamic program that grades it; wire protocols ship with the compatibility corpus. *No subsystem without its falsifier* is the methodology of Chapter 5 made concrete, and the ordering follows Chapter 8.

## 6.22 Product Coverage

The banking layer's target portfolio, all expressible as `std::bank` and `std::temporal` constructs over the general core:

- **FX and multi-currency** — FX-atomic transactions (Definition 3.7) with positions as per-currency signals; ledger partitioning by currency with linked cross-currency transfers is the industrial precedent.
- **Derivatives** — forwards and swaps as bitemporal schedules of contingent postings; caps and floors as rate-indexed contingent legs priced by UDFs and settled as balanced postings.
- **Lending** — revolving and term facilities (commitment, drawdown, accrual signals, repayment waterfalls); syndicated facilities with participation shares as fractional postings conserving per participant set.
- **Trade finance** — letters of credit as capability-gated state machines whose transitions are ledger events; trade loans; supply-chain finance with conservation across assignor and assignee.
- **Liquidity** — sweeps, cash pooling and zero-balance structures as scheduled inter-account postings generated by temporal rules and balanced by construction.
- **Authorization and holds** — the reservation model of Section 3.19, with available balance as a REV and the authorization decision at ℓ₅.

Each is a worked example in the artifact; none requires a kernel change, which is the running scope test of Section 9.11.

## 6.23 Syntax Lineage and the SQL-First, Rust-Fallback Rule

**The normative rule** for every surface decision: (1) if SQL already expresses the construct, use SQL's keywords and shape — the sole permitted deviation being pipelined clause *order*, which reads in evaluation order while reusing every SQL keyword verbatim; (2) otherwise, if Rust has an equivalent, use Rust's spelling exactly; (3) otherwise, and only then, invent — and inventions introduce new keywords, types and literals rather than overloading inherited ones.

The rule carries one exception, and the exception is what makes it a discipline rather than a capitulation: **a SQL spelling is not reused where it would sacrifice determinism or efficiency the engine depends on.** An unbounded recursive CTE is admitted only behind the guard of Section 6.12; `float` is not a legal money type; and Rust spellings implying ambient mutation, raw pointers, threads or I/O are not surfaced at all (Section 6.14). The language declines to inherit SQL's permissiveness precisely where permissiveness would cost a guarantee.

This ordering is the reverse of the one an earlier draft adopted, and the reasons for reversing it are given in Appendix J.1: the population to be persuaded reads SQL; the translation theorem of Section 4.7 becomes near-syntactic for the reused fragment when the vocabulary matches, and would otherwise make every clause a translation case; and a language claiming to replace SQL that respells `GROUP BY` has made an unforced adoption error.

**Lineage.** Query vocabulary — filtering, joining, grouping, having, set operations, DDL, DML, TCL and DCL statements — comes from SQL, in pipelined order rather than SQL's inverted clause order. Items, expressions, patterns, generics, traits, attribute annotations and the imperative tier come from Rust — chosen as the fallback because the engine itself is written in Rust and the UDF tier compiles through the same toolchain, so one syntactic idiom spans language and engine. Novel forms — base and ledger declarations, `serve` contracts, anchors, `Money`, value dates and bitemporal literals, idempotency keys, effect annotations, materialization modes, lineage modes and confidentiality annotations — are new, with no false cognates: a novel construct never reuses an inherited keyword with changed meaning.

The rule is testable: any construct violating the precedence order is a specification bug, and Appendix B applies it item by item.

## 6.24 Build Stack and Compilation Targets

**The four-technology stack:** Rust (engine, kernel, stage-0 compiler host); the Niles IR (the semantic centre); WebAssembly (the UDF ABI and the self-hosted optimizer's own execution target); and an optional established back-end for release-grade native code generation, against which the self-hosted back-end's remaining gap is measured and published rather than hidden.

**Targets:** native Linux on ARM64 and x86-64, macOS on ARM64, Windows on x86-64, a WSL profile (Linux with a static-linking profile), and wasm32. The target model is data (Appendix E.2): each target is a declarative record consumed by all back-ends, so adding a target is a data change plus encoder tables. The cross-target determinism obligation, with the Wasm mitigations of Section 6.13, is Appendix C.5.
