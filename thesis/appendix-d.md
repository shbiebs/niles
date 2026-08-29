# Appendix D. Nilestream Engine and Intermediate Representation Reference

## D.1 Component Map

| Crate | Role |
|---|---|
| `nilestream-ledger` | Epoch segments, sequencer, hash chain, durability, admission and commit rules |
| `nilestream-core` | REV runtime: resident maps, anchor indices, apply loop, upqueries, contracts |
| `nilestream-optimizer` | Adaptive materialization: estimators, mode selection, eviction, hysteresis |
| `nilestream-lineage` | Provenance annotation, explain, reproduce, impact |
| `niles-ir` | Typed IR: circuit types, verifier, interpreter, upquery paths |
| `niles-lang` | Stage-0 compiler: lexer, parser, type/effect checker, lowering; SQL surface |
| `niles-stdlib` | `std::bank`, `std::temporal`, money, builtins |
| `nilestream-storage` | Tiering, migration, checkpoints, cold backend |
| `nilestream-server` | Daemon: sessions, native protocol, MySQL/PostgreSQL wire, observability, audit |
| `conservation-suite` | Reference oracle, property tests, fault campaigns |
| `bank-bench` | NilesBank generator, harness, analysis |

## D.2 Ledger Write-Path API

```rust
pub struct Ledger { /* segments, chain head, open epoch, idempotency window */ }

impl Ledger {
    /// Admission: idempotency check, commit rule, authorization. Idempotent by key
    /// within the declared window; a fingerprint mismatch is a conflict, not a replay.
    pub fn submit(&self, txn: ValidatedTxn) -> Result<Pending, AdmitError>;

    /// Seals on the tau boundary or size bound: hash, fsync (and quorum), publish vis.
    pub fn seal_boundary(&self) -> SealedEpoch;

    /// An immutable, verify-on-read reader over the prefix at `upto`.
    pub fn prefix(&self, upto: Epoch) -> PrefixReader;

    pub fn frontiers(&self) -> Frontiers;                  // seal / dur / vis
    pub fn verify_chain(&self, from: Epoch, to: Epoch) -> Result<(), ChainBreak>;
}
```

Validation stages, in order: schema typing; the commit rule (for ledgers, per-currency zero-sum in exact integer minor units); authorization capabilities for flagged effects; idempotency-window check. `SealedEpoch` carries `(id, parent_hash, hash, count, bytes)`, and publication order is fsync → chain append → visibility advance, never reordered. **There is no update or delete API**, by design: corrections are appends and hold resolutions are appends.

## D.3 Read-Model Runtime API

```rust
pub enum Slot<V> { Bottom, Hole(Epoch), Pending(Epoch, Waiters), Present(V, Epoch) }

pub struct Rev { /* circuit, resident map, anchor index, contract, mode */ }

impl Rev {
    pub fn read(&self, key: Key, sess: &mut Session) -> ReadOutcome; // Hit | Wait | Upquery
    pub fn apply(&mut self, epoch: &SealedEpoch);          // advance anchors monotonically
    pub fn evict(&mut self, key: Key);                     // Present -> Hole(anchor)
    pub fn upquery(&self, key: Key, at: Epoch) -> Anchored<Val>;   // pull over frozen prefix
    pub fn frontier(&self) -> Epoch;                       // applied_V
    pub fn explain(&self, key: Key) -> Lineage;            // at the view's lineage mode
    pub fn set_mode(&mut self, range: KeyRange, mode: Mode); // optimizer-driven; see Thm 4.5(a)
}
```

`Slot` is the absence lattice of Section 3.4 verbatim — the type mirrors the lattice deliberately, so that a state the theory does not contemplate cannot be represented.

**Anchor indices.** Per key, an epoch-skip structure over that key's delta positions in the segment stream, giving amortized O(1) location of the deltas in an interval. This is the structure that discharges Theorem 4.3′'s access assumption in practice; without it, locating a key's history would scan and would reintroduce a dependence on base length.

## D.4 The Typed Intermediate Representation

The IR is a circuit language: nodes are operators, edges carry `Stream<Z<Row>>` types annotated with anchors, effects, contracts and provenance.

**Operator set.** `source(table|base|ledger) map filter project join(inner|left|semi|anti) group aggregate(Δ-form) distinct union negate integrate differentiate delay window fixpoint(guarded) json_path graph_step index_terms sink(view{contract})`.

**Per-node metadata.** Row type; key type; delta form; **upquery path** (the reverse-mode slice specification — which input keys suffice to recompute one output key, derived at plan time as the support of the provenance polynomial); effect row; and lineage mode.

**The verifier checks:** type and effect coherence; contract compatibility at sinks (a sink may not demand a rung its sources cannot supply); guardedness of every fixpoint; absence of nondeterministic operators; well-formedness of upquery paths (every output key must have a finite recomputation witness — a circuit without one cannot be partially materialized and is rejected for `demand` mode at compile time rather than failing at runtime); and confidentiality flow.

**Identity.** IR is serialized content-addressed, so **view identity is the IR hash**. This is what makes "the same view" well-defined across upgrades, what lets the optimizer's statistics survive a redeploy that did not change semantics, and what makes a semantic change to a view a visible, auditable event.

## D.5 Wire Protocols

**Native.** Length-prefixed versioned frames carrying typed results *with anchors*; sessions hold the ladder watermarks implementing ℓ₁ and ℓ₂.

**MySQL adapter.** Handshake, standard authentication plugins, query and prepared-statement lifecycles, text and binary result sets; the supported statement subset is documented and corresponds to the fragment of Appendix H. The anchor is exposed through a session variable and an optional result attribute.

**PostgreSQL adapter.** Startup and authentication, simple and extended query protocols, text and binary formats, cursors; the anchor is exposed through a notice field and a settable/gettable parameter.

**Policy.** Anything outside the documented subset returns a named unsupported-feature error. Silent divergence is a correctness bug, not a compatibility gap — the operational counterpart of Theorem 4.6(c)'s fragment honesty.

## D.6 Storage, Durability, and Memory

**Segment format.** Header (magic, version, algorithm identifiers, epoch id, parent hash), body (canonically serialized rows, compressed), footer (hash, checksum). Algorithm identifiers in the header are what make cryptographic agility possible without breaking historical verification.

**Durability.** Group commit within τ; fsync then chain-append then publish; quorum acknowledgement in replicated profiles.

**Tiering.** Hot segments on fast local media; cold segments in object storage with verify-on-read. Checkpoints of resident maps are pure caches — never trusted, always re-derivable.

**No base compaction.** Proposition 3.4 makes full retention necessary for reconstructibility, so segments may be migrated and re-encoded but never merged away. This is the one place where a conventional storage engine's most useful feature is deliberately forbidden.

**Memory.** Arena-per-open-epoch on the write path; slab allocation for resident maps under the optimizer's budget. The global memory budget is the `m` of every theorem in Chapter 4.

## D.7 Configuration Reference

**Per view:** `consistency` (six rungs), `freshness` (K, T), `checkpoint` (per-key interval C; Theorem 3.7), `materialize` (`absent | demand | full | spilled | tiered | auto`), `budget_share`, `retain` (`evictable | pinned | forever`), `lineage` (`off | key | full`), `backfill`, `upquery_parallelism`, `checkpoint`.

**Global:** epoch period τ, durability mode, memory budget, tiering thresholds, optimizer aggressiveness and hysteresis, wire-protocol toggles, audit endpoints, UDF fuel limits.

Every knob is pinned and recorded in benchmark configurations (Section 9.2), because an unrecorded knob is an unreproducible result.

## D.8 Observability, Audit, and Operations

**Metrics.** Frontier gauges (seal, dur, vis, applied per view); hit, miss and upquery counters; **reconstruction-latency distributions and the derived Z per key range** (required by Section 5.5, since a phase-diagram point without Z is uninterpretable); eviction pressure; mode transitions *with the estimator values that caused them*; admission rejections by cause; chain-verification status.

**Audit endpoints.** Chain verification over a range; reproduction of a published `(answer, epoch)` by recomputation and byte comparison; bitemporal query by (system epoch, valid time); and impact analysis from a base row to affected derived entries.

**Operations.** Drain-and-seal; checkpoint; migrate; rotate keys (recorded as a ledger event); deploy and retire views by IR hash. Logs are structured and epoch-stamped.

## D.9 Deployment Topology

**Single node.** One process, all crates in-process.

**Replicated.** A ledger group under established consensus, with sequencing following leadership; read models on followers or dedicated read nodes subscribing to sealed epochs.

**Sharded.** Multiple ledger groups partitioned by account space (or, for currency-partitioned deployments, by ledger), with cross-shard transactions via the epoch-aligned protocol of Section 8.6 and read models joining per-group frontiers into vector anchors. At this tier ℓ₃ requires a consistent cut across groups rather than a scalar epoch — the one place the formal machinery changes shape.

**Edge.** Read-only replicas serving ℓ₀–ℓ₃ from shipped segments; strict rungs always route to the group, because ℓ₅ is provably unavailable at the edge.
