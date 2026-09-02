# Appendix C. Compilation Model and Target Binaries

## C.1 Compilation Pipeline

Source (Niles native, SQL surface, or embedded API) → lex/parse → AST → name resolution → type-and-effect checking (currency rows, linearity, consistency effects, confidentiality flows, idempotency windows) → IR lowering → **IR verification** → optimization → back-end (native code generation or wasm32) → link.

Declarative-tier items pass additionally through the planner — circuit construction, join ordering, delta-form selection, provenance-derived upquery-path derivation, and initial materialization-mode defaults — before code generation. Transaction-tier and UDF code follow the classical path.

Every stage emits a content-addressed artifact, so a build is a Merkle tree of stage outputs. That is what the determinism gate (C.5) compares, and it is also what makes a compiler bug bisectable: a divergence identifies the stage that introduced it.

**The verification step is not an optimization.** The IR verifier is an independent checker that re-validates all compiler output, and it exists so the kernel trusts the verifier rather than the compiler (Section 6.14). It deliberately shares no code with the lowerer; otherwise it would confirm the lowerer's assumptions instead of the IR's rules.

## C.2 Target Triples and the Target Model as Data

Supported v1 targets: `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, a WSL profile (Linux with the static-linking profile of Appendix E.16), and `wasm32-wasip1` for UDFs and the self-hosted optimizer.

Each target is a **declarative record** (Appendix E.2): word size, endianness, alignment, calling convention (parameter and return registers, caller/callee-saved sets, stack alignment), relocation kinds, and object-format parameters. All back-ends are generic over the record and encoder tables are generated from it, so adding a target is a data change plus tables rather than a compiler rewrite. The WSL profile is the existence proof: a target record plus linker flags, with no compiler-code change.

## C.3 Back-Ends

Two. The **self-hosted back-end** (Appendix E.14–E.16) is the default and the scientific artifact. An **optional established back-end** provides release-grade optimization for users who want it and, more importantly, provides the measuring stick for the self-hosted one.

Policy: correctness claims and all evaluation numbers come from the self-hosted back-end unless a figure explicitly says otherwise. The optional back-end exists to *measure* the remaining optimization gap, which is published per release rather than hidden (E.13).

## C.4 The UDF WebAssembly ABI

UDFs compile to wasm32 against a frozen ABI. Imports: none beyond a capability table passed at instantiation — the capability-based model in which a module "starts with no ambient authority and can only do what the host explicitly grants" is exactly the property the kernel boundary needs. Exports: a single call entry over a canonical, versioned serialization of `Anchored` values and domain types. Memory: one guest-owned linear memory with host copy-in/copy-out and no sharing. Metering: fuel injected at load. ABI version negotiation is recorded per view, so a UDF upgrade is an epoch-visible event rather than a silent behaviour change.

## C.5 The Cross-Target Determinism Obligation — and Why Wasm Alone Does Not Provide It

**Obligation.** For every (IR circuit, base prefix, anchor), the emitted answer bytes are identical on every supported target and back-end. Theorem 4.1's exactness, hash-chain reproducibility, and the audit `reproduce` operation all depend on this.

**The correction.** It would be convenient to assume the Wasm sandbox delivers determinism. It does not. The specification names three sources of implementation-dependent behaviour: **NaN payloads** (canonical NaNs carry a non-deterministic sign bit, and non-canonical inputs produce non-deterministic outputs), **resource exhaustion** (an engine may fail to grow linear memory, and available resources differ across devices), and **host functions** [Haas et al., PLDI '17]. A UDF that produces a NaN, or that exhausts memory at a device-dependent point, would break reconstruction-equivalence over a hash-chained base. Stated as "Wasm is deterministic," the obligation would be false.

**The obligation, stated correctly, is therefore three mitigations plus three engine rules.**

*UDF tier:* (1) reject NaN-producing floating-point operations in UDF signatures reachable from view computation, or canonicalize NaN at the kernel boundary before any value crosses into a circuit; (2) make resource exhaustion a **deterministic** abort at a fixed fuel bound rather than a device-dependent failure, so the same input aborts at the same point everywhere; (3) permit no host imports outside a deterministic allowlist.

*Engine:* (4) canonical ordering before every emission — no hash-order iteration ever escapes into output; (5) money as exact integers at the currency's scale, never binary floating point; (6) no ambient wall-clock reads in query context, so "now" enters only as a contract-chosen anchor.

**The gate.** Continuous integration executes the conservation suite and a query corpus on every supported target and back-end, at every optimization level, hashing every output stream; any divergence fails the build. The same gate covers the compiler itself (E.18). A test suite deliberately attempts each of the three Wasm non-determinism sources and must observe the mitigation firing.

**Status, stated rather than implied.** What runs today is the *run-to-run* half of that gate, on one target: `niles_interp::determinism_gate` executes a Niles program five times over freshly loaded copies and requires byte-identical output, and the bootstrap gates of E.19 apply it to the Niles-written lexer and parser, including to each parsing its own source. That establishes freedom from the accidental non-determinism this obligation is most often violated by — hash-map iteration order, address-dependent behaviour, a seeded structure escaping into output — and it establishes nothing about a *second* target, because it runs in one process on one machine.

**Cross-target verification is pending, and pending on two artefacts that do not exist**: a WASM build and an ARM64 build to compare against x86-64. Until both run and their output streams hash equal, the obligation of this section is unmet. The return type of the gate says so — it reports the number of runs and whether they agreed, and has no field in which to record a target — so the code cannot be read as claiming more than it checks.

## C.6 Build and Deployment Model

**Build.** One workspace for the engine and stage-0 compiler, plus the self-hosting stages driven by a bootstrap target; hermetic builds with pinned toolchains (Appendix G.1). Outputs: the server daemon, the CLI (compile, check, REPL), the bare compiler, and the wasm optimizer module.

**Deployment.** A single static binary per target. Configuration is declarative (contract defaults, storage tiers, τ, memory budgets — Appendix D.7). State directories separate by durability class: the base segments (the only truth), derived view state (disposable), and keys (deployment-profile).

**Upgrades.** Drain-and-seal, swap binary, recover. Recovery correctness *is* the upgrade correctness argument, since recovery is a large eviction (Theorem 4.1). Downgrade is supported to any binary whose IR version can verify the stored circuits, which is why the IR carries an explicit version and the verifier rejects rather than guesses.
