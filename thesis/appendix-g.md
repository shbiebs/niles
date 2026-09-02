# Appendix G. Build and Implementation Kit

## G.1 Software Requirements

A pinned host toolchain recorded per release; a build driver; a scripting environment for analysis with pinned library versions; a Wasm runtime for the UDF tier and the self-hosted optimizer; optionally a container image for hermetic builds; optionally a proof assistant for the mechanization named in Chapter 12. Development on Linux or macOS; Windows through the WSL profile.

## G.2 External Dependencies

**Policy:** minimal, audited, pinned by lockfile, vendorable. Cryptographic primitives are never hand-rolled. Serialization libraries are used for *framing only* — canonical base serialization is hand-written and versioned, because the hash chain's meaning depends on byte-level stability that a third-party library is free to change. Concurrency primitives, property-testing, model-checking and benchmarking libraries are used freely, since they are outside the trusted base.

**Forbidden by policy:** any dependency that owns base bytes or commit ordering. This is why a general-purpose storage engine is not used for the base even though one is appropriate for cold view state — its compaction would rewrite data the design declares immutable (Proposition 3.4).

## G.3 Baseline Systems

Two mainstream RDBMSs with configurations published alongside the results; a total-materialization IVM engine of the Materialize/Feldera class with version pinned at run time; the in-tree no-anchor ablation, whose exact configuration diff is published so readers can see what the ablation removed; and a purpose-built ledger for the write-path floor. Each baseline is tuned per its own vendor's guidance, and each carries a fairness note stating what it is and is not being asked to do.

**Excluded by policy:** vendor performance claims lacking methodology, hardware specification or workload definition. Where a vendor's documentation and marketing disagree on a basic parameter, that disagreement is reported rather than resolved in the favourable direction.

## G.4 Minimum Hardware

Development: a modern multi-core workstation. Reference single-node evaluation: a fixed instance class with core count, memory, storage class and network recorded, and with **verified fsync semantics** — a durability benchmark on a device that lies about fsync measures nothing. The long-horizon run additionally requires a cold tier sized for the target event volume. Distributed evaluation: five reference nodes. Every figure records its hardware identifier.

## G.5 Cost Model and Materialization Heuristics

Per view and key range, the optimizer tracks: arrival rate λ_r; reuse-distance estimate; reconstruction cost ĉ_u (EWMA of measured upquery path evaluations); **reconstruction latency L̂ and the derived Z = L̂ · λ_r**; update rate; residency bytes; and the contract multiplier Φ(ℓ) from Section 3.14.

**Mode selection (rent-or-buy).** Accumulate reconstruction spend for a range; when cumulative spend reaches the cost of maintaining it materialized, switch to `full` — the classical break-even rule, 2-competitive deterministically, with a randomized variant approaching e/(e−1). Downgrade uses the mirrored rule with hysteresis.

**Eviction (cost-and-size aware, delay-weighted).** Rank candidates by expected aggregate delay rather than reuse probability alone: a credit proportional to ĉ_u · (1 + Z) · Φ(ℓ) per unit of residency, decremented as budget pressure rises — a Landlord-style discipline extended by the delayed-hit weighting. Plain LRU is implemented too, as the H-S7 comparison point.

**Spill and tier.** When a range's reconstruction cost is high but its residency cost is dominated by size rather than by update rate, `spilled` or `tiered` dominates both `demand` and `full`; the decision compares I/O cost against reconstruction cost directly.

**Hysteresis.** Mode changes require a sustained signal over a window, with a minimum dwell time per mode, because thrashing between modes is worse than either mode.

**Pinning.** Ranges pinned by contract are excluded from the decision entirely.

Appendix I gives these as algorithms with their estimator update rules; this section is the model they implement.

## G.6 Instruction Selection and Machine-Code Encoding

Selection tables live per target as pattern-to-template mappings with costs; maximal munch with an explicit tie-break order (fewer operations, then shorter encoding). Encoding is table-generated from the target model, with an exhaustive encoder test against a reference disassembler corpus per release.

## G.7 Linear-Scan Register Allocation

Live intervals from SSA liveness; intervals sorted by start; an active set ordered by end; on pressure, split at the next use and spill the interval with the furthest next use, weighted by loop depth; reloads placed at split points; callee-saved registers allocated last. The allocator's invariant checklist — no overlapping assignment, every use reachable from a definition or a reload — is checked in a debug mode and covered by golden tests per target.

## G.8 Consensus Wiring and Cross-Shard Commit

**What exists is a single-process, deterministic simulator**: `nilestream-consensus`, 23 tests, no sockets, no clock, with message loss, reordering, partition and restart as scheduling decisions the simulator makes. The cross-shard commit protocol has never been run over a network, and no sentence here should be read as saying otherwise.

In the design, each ledger group is a consensus group; sequencing follows leadership; sealing requires quorum-durable segments. Cross-shard transactions use the epoch-aligned protocol of Section 8.6, with reservation and outcome records written as ordinary base rows — so the protocol's own history is audited by the same chain it commits to, which is a small but pleasing consequence of making the log the product. In the simulator, the recovery cases (coordinator loss, participant loss, partition during prepare) each have a test; a *tabulated* recovery matrix with a test per cell is not built.

Two of those tests earned their place this cycle by failing a mutation they should have failed all along, and the reason is worth recording because it generalizes. Every scenario used three or five nodes — and for an odd cluster the strict-majority rule `replicas * 2 > total` and the off-by-one `>=` accept exactly the same sets, so weakening the quorum rule passed the entire suite. Separately, the previous-term commit test let its entry commit *before* advancing the term, so deleting Raft's current-term restriction changed nothing it observed. An even-sized cluster and an uncommitted entry are what make those two rules testable, and both cases are now present.

## G.9 Benchmark Harness and Seeds

The harness drives everything from scenario files fixing generator parameters (account population, α, mix weights, τ, memory budget, ad hoc query rate, lineage mode, per-view contracts), seeds (derived from a base seed and run index; five runs by default), warm-up and measurement windows. Output is a raw archive plus derived figures. Every figure identifier maps to exactly one scenario file, and the figure generator refuses to emit a plot for which no pre-registered template exists (Section 5.9).

Generators cover the write and read mixes of Section 9.1, including value dates distinct from booking epochs and backdated corrections, hold lifecycles with partial capture and expiry, and the mutant corpora (ill-typed programs, corrupted segments) for the negative tests.

## G.10 Build Gate and Reproduce

`gate` = hermetic build + unit and property suites + conservation suite + determinism gates (Appendix C.5 and E.18) + lints + `unsafe` inventory check. Continuous integration refuses merges on any red.

`reproduce FIGURE=<id>` = clean state → scenario run → analysis → figure bytes. `reproduce-all` regenerates every number in Chapter 9, with its wall-clock budget documented.

The artifact's front page is the two-command story: run the gate, then reproduce. If both work on the reference hardware, the empirical chapter is the reader's to re-derive — which is the only form of trust this thesis asks for.
