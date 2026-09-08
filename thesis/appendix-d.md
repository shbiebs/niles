# Appendix D. Nilestream Engine and Intermediate Representation Reference

## D.1 Component Map

**This appendix is generated, and it did not use to be.** Every type and method D.2 and D.3
listed — an `append_batch` on a ledger handle, a `serve` on a read model, a storage tier,
an `explain` returning a lineage — was absent from `crates/`, in an appendix a reader consults precisely
to find out what exists. Three of the eleven crates in the map below it were stubs with no
caller; two of those have since been deleted from the workspace and the third with them
(`docs/ROADMAP.md` records all three as planned). What follows is read off the crates by
`thesis/gen-appendix-d.py`, so a component that does not exist cannot appear here and a
method that does not exist cannot be listed. The *roles* are hand-written, because what a
component is for is not derivable from its source.

<!-- BEGIN:appendix-d-map thesis/appendix-d-map.md#verbatim -->

*Generated from `thesis/appendix-d-map.md`. Do not edit by hand.*

*Generated from the workspace by `thesis/gen-appendix-d.py`. Do not edit by hand.*

| Crate | Role | public items |
|---|---|--:|
| `bank-bench` | NilesBank generator, wall-clock harness, thesis drift tests | 70 |
| `conservation-suite` | Reference oracle and the conservation property tests | 23 |
| `experiments` | The E-series measurement harness | 0 |
| `niles-interp` | The imperative-subset interpreter `nilesc run` drives, and the ledger it posts to | 16 |
| `niles-ir` | Typed IR: circuit types, verifier, reference interpreter, upquery paths | 58 |
| `niles-lang` | Stage-0 compiler: lexer, parser, type/effect checker, lowering; SQL surface | 134 |
| `nilesc` | The compiler driver: `check`, `verify`, `run` | 0 |
| `nilestream` | The engine binary: sweep and serve | 0 |
| `nilestream-consensus` | A single-process, deterministic simulator for replication and cross-shard commit. No sockets, no clock | 21 |
| `nilestream-core` | REV runtime: resident maps, anchor indices, apply loop, upqueries, contracts | 27 |
| `nilestream-ledger` | Epoch segments, sequencer, hash chain, durability, admission and commit rules | 32 |
| `nilestream-optimizer` | Plan-time mode selection and the eviction policies (the adaptive optimizer of §4.6 is specified and not built) | 37 |
| `nilestream-server` | Daemon: sessions, PostgreSQL wire surface, conformance | 87 |
| `proto-engine` | The research prototype the counted-work experiments run on | 24 |

<!-- END:appendix-d-map -->

## D.2 The Public Surface of the Write Path, the Read Model and the IR

Generated from the crates. The prose after it says what the shapes mean; the shapes
themselves are whatever the source has.

<!-- BEGIN:appendix-d-api thesis/appendix-d-api.md#verbatim -->

*Generated from `thesis/appendix-d-api.md`. Do not edit by hand.*

*Generated from the workspace by `thesis/gen-appendix-d.py`. Do not edit by hand.*

**`nilestream-ledger`**

```rust
pub const KEY_LEN: usize
pub const NONCE_LEN: usize
pub const TAG_LEN: usize
pub fn hchacha20(key: &[u8
pub fn seal(key: &[u8
pub fn open(
pub struct Hasher256
pub fn sha256(bytes: &[u8]) -> [u8
pub fn chain_hash(parent: &[u8
pub fn hex(digest: &[u8
pub struct Frontier
pub struct Snapshot
pub type Minor
pub struct Epoch(pub u64)
pub enum RandomError
pub fn fill(out: &mut [u8]) -> Result<(), RandomError>
pub fn bytes<const N: usize>() -> Result<[u8
pub enum SyncPolicy
pub struct Record
pub enum TruncationCause
pub struct Recovery
pub struct Segment
pub fn recover(path: impl AsRef<Path>) -> std::io::Result<Recovery>
pub struct Txn
pub enum Rejected
pub struct SequencerStats
pub struct Sequencer
pub type Commitment
pub type KeyId
pub enum SidecarError
pub fn commit(salt: &[u8
pub struct Sidecar
```

**`nilestream-core`**

```rust
pub type Epoch
pub enum Slot<V>
pub type ShardId
pub struct ShardMap
pub enum ReadError
pub struct Shard
pub struct Cluster
pub fn maintenance_stride(c: Consistency) -> u64
pub fn is_highly_available(c: Consistency) -> bool
pub fn permits_reading(outer: Consistency, inner: Consistency) -> bool
pub type Key
pub type Value
pub struct Anchored
pub enum ReadOutcome
pub struct Joined
pub struct Completion
pub struct FoldTicket
pub struct WaitTicket
pub enum ReadMode
pub trait Base
pub struct MergeCaps
pub enum MergeRefusal
pub enum Policy
pub struct Stats
pub struct Rev
pub enum Unsupported
pub struct Runtime
```

**`niles-ir`**

```rust
pub type NodeId
pub enum Anchor
pub struct Checked<T>
pub struct Node
pub struct Circuit
pub struct AccessReport
pub fn internal_contract() -> ServeContract
pub type Row
pub type ZSet
pub fn add(z: &mut ZSet, row: Row, w: i128)
pub fn zset(rows: &[(&[i128], i128)]) -> ZSet
pub fn row(vs: &[Option<i128>]) -> Row
pub fn eval_scalar(s: &Scalar, r: &[Value]) -> Value
pub fn keeps(p: &Scalar, r: &[Value]) -> bool
pub struct Eval<'a>
pub enum EvalError
pub fn implements(op: &Op) -> bool
pub fn run(c: &Circuit, output: &str, sources: &BTreeMap<String, ZSet>) -> (ZSet, u64)
pub fn run_node(c: &Circuit, id: NodeId, sources: &BTreeMap<String, ZSet>) -> (ZSet, u64)
pub fn try_run_node(
pub fn try_run_with(
pub fn try_run_node_with(
pub fn try_run(
pub fn fold(a: Agg, vals: &[(Value, i128)]) -> Value
pub enum Consistency
pub enum Materialize
pub enum Retention
pub enum Lineage
pub struct ServeContract
pub type ColIdx
pub enum Agg
pub enum JoinKind
pub enum Scalar
pub enum ScalarOp
pub enum Op
pub enum ApplyKind
pub enum Step
pub enum ScheduleError
pub struct Schedule
pub fn check(circuit: &Circuit, schedule: &Schedule) -> Result<Circuit, ScheduleError>
pub fn apply(circuit: &Circuit, step: &Step) -> Result<Circuit, ScheduleError>
pub fn columns_read(s: &Scalar, out: &mut Vec<ColIdx>)
pub fn step_from_name(name: &str, node: NodeId) -> Result<Step, ScheduleError>
pub fn catalogue() -> Vec<&'static str>
pub struct Hop
pub struct UpqueryPath
pub enum NoPath
pub fn derive(circuit: &Circuit, node: NodeId, epoch: u64) -> Result<UpqueryPath, NoPath>
pub enum Value
pub enum Tri
pub fn compare(a: Value, b: Value, f: impl Fn(i128, i128) -> bool) -> Tri
pub fn arith(a: Value, b: Value, f: impl Fn(i128, i128) -> i128) -> Value
pub fn truth(v: Value) -> Tri
pub fn days_since_epoch(text: &str) -> Option<i64>
pub struct Violation
pub struct VerifyReport
pub fn verify(c: &Circuit) -> VerifyReport
pub fn unevaluable(c: &Circuit, implemented: impl Fn(&Op) -> bool) -> Vec<Violation>
```

<!-- END:appendix-d-api -->

Validation stages, in order: schema typing; the commit rule (for ledgers, per-currency
zero-sum in exact integer minor units); authorization capabilities for flagged effects;
idempotency-window check. Publication order is fsync → chain append → visibility advance,
never reordered. **There is no update or delete API**, by design: corrections are appends and
hold resolutions are appends.

`Slot` is the absence lattice of Section 3.4 verbatim — the type mirrors the lattice
deliberately, so that a state the theory does not contemplate cannot be represented.

## D.3 The Typed Intermediate Representation

**Specification and implementation mixed.** The operator set, the verifier's checks and content-addressed identity are built (`niles-ir`, and the listing in D.2); the provenance metadata is not, since no lineage mode exists.

The IR is a circuit language: nodes are operators, edges carry `Stream<Z<Row>>` types annotated with anchors, effects, contracts and provenance.

**Operator set.** `source(table|base|ledger) map filter project join(inner|left|semi|anti) group aggregate(Δ-form) distinct union negate integrate differentiate delay window fixpoint(guarded) json_path graph_step index_terms sink(view{contract})`.

**Per-node metadata.** Row type; key type; delta form; **upquery path** (the reverse-mode slice specification — which input keys suffice to recompute one output key, derived at plan time as the support of the provenance polynomial); effect row; and lineage mode.

**The verifier checks:** type and effect coherence; contract compatibility at sinks (a sink may not demand a rung its sources cannot supply); guardedness of every fixpoint; absence of nondeterministic operators; well-formedness of upquery paths (every output key must have a finite recomputation witness — a circuit without one cannot be partially materialized and is rejected for `demand` mode at compile time rather than failing at runtime); and confidentiality flow.

**Identity.** IR is serialized content-addressed, so **view identity is the IR hash**. This is what makes "the same view" well-defined across upgrades, what lets the optimizer's statistics survive a redeploy that did not change semantics, and what makes a semantic change to a view a visible, auditable event.

## D.4 Wire Protocols

**One of the three is built.** The PostgreSQL surface is implemented and driven by `psql` in conformance tests. The native frame protocol and the MySQL adapter are **specification**: there is no MySQL listener, which is why H-S5's client-compatibility half is unmeasured.

**Native.** Length-prefixed versioned frames carrying typed results *with anchors*; sessions hold the ladder watermarks implementing ℓ₁ and ℓ₂.

**MySQL adapter.** Handshake, standard authentication plugins, query and prepared-statement lifecycles, text and binary result sets; the supported statement subset is documented and corresponds to the fragment of Appendix H. The anchor is exposed through a session variable and an optional result attribute.

**PostgreSQL adapter.** Startup and authentication, simple and extended query protocols, text and binary formats, cursors; the anchor is exposed through a notice field and a settable/gettable parameter.

**Policy.** Anything outside the documented subset returns a named unsupported-feature error. Silent divergence is a correctness bug, not a compatibility gap — the operational counterpart of Theorem 4.6(c)'s fragment honesty.

## D.5 Storage, Durability, and Memory

**Durability and the segment format are built** (`nilestream-ledger`, measured at parity with PostgreSQL's `fsync` path in §9.14.1). **Tiering, cold storage and the memory arenas are specification** — the crate that would have held them was a stub and has been deleted; `docs/ROADMAP.md` records it.

**Segment format.** Header (magic, version, algorithm identifiers, epoch id, parent hash), body (canonically serialized rows, compressed), footer (hash, checksum). Algorithm identifiers in the header are what make cryptographic agility possible without breaking historical verification.

**Durability.** Group commit within τ; fsync then chain-append then publish; quorum acknowledgement in replicated profiles.

**Tiering.** Hot segments on fast local media; cold segments in object storage with verify-on-read. Checkpoints of resident maps are pure caches — never trusted, always re-derivable.

**No base compaction.** Proposition 3.4 makes full retention necessary for reconstructibility, so segments may be migrated and re-encoded but never merged away. This is the one place where a conventional storage engine's most useful feature is deliberately forbidden.

**Memory.** Arena-per-open-epoch on the write path; slab allocation for resident maps under the optimizer's budget. The global memory budget is the `m` of every theorem in Chapter 4.

## D.6 Configuration Reference

**The per-view knobs are built** — `serve { consistency, materialize, .. }` is checked by the compiler and read by the runtime. The global table is **specification**; the optimizer knobs in it belong to an optimizer that is not built (§4.6).

**Per view:** `consistency` (six rungs), `freshness` (K, T), `checkpoint` (per-key interval C; Theorem 3.7), `materialize` (`absent | demand | full | spilled | tiered | auto`), `budget_share`, `retain` (`evictable | pinned | forever`), `lineage` (`off | key | full`), `backfill`, `upquery_parallelism`, `checkpoint`.

**Global:** epoch period τ, durability mode, memory budget, tiering thresholds, optimizer aggressiveness and hysteresis, wire-protocol toggles, audit endpoints, UDF fuel limits.

Every knob is pinned and recorded in benchmark configurations (Section 9.2), because an unrecorded knob is an unreproducible result.

## D.7 Observability, Audit, and Operations

**Specification**, except the counters `nilestream-core` already keeps (reads, hits, misses, upqueries, base rows, evictions) and chain verification, which `nilestream-ledger` implements and the recovery path uses.

**Metrics.** Frontier gauges (seal, dur, vis, applied per view); hit, miss and upquery counters; **reconstruction-latency distributions and the derived Z per key range** (required by Section 5.5, since a phase-diagram point without Z is uninterpretable); eviction pressure; mode transitions *with the estimator values that caused them*; admission rejections by cause; chain-verification status.

**Audit endpoints.** Chain verification over a range; reproduction of a published `(answer, epoch)` by recomputation and byte comparison; bitemporal query by (system epoch, valid time); and impact analysis from a base row to affected derived entries.

**Operations.** Drain-and-seal; checkpoint; migrate; rotate keys (recorded as a ledger event); deploy and retire views by IR hash. Logs are structured and epoch-stamped.

## D.8 Deployment Topology

**Single node is built. Everything below it is specification**, and the replicated and sharded rows are exercised only in the single-process simulator of `nilestream-consensus` — never over a network.

**Single node.** One process, all crates in-process.

**Replicated.** A ledger group under established consensus, with sequencing following leadership; read models on followers or dedicated read nodes subscribing to sealed epochs.

**Sharded.** Multiple ledger groups partitioned by account space (or, for currency-partitioned deployments, by ledger), with cross-shard transactions via the epoch-aligned protocol of Section 8.6 and read models joining per-group frontiers into vector anchors. At this tier ℓ₃ requires a consistent cut across groups rather than a scalar epoch — the one place the formal machinery changes shape.

**Edge.** Read-only replicas serving ℓ₀–ℓ₃ from shipped segments; strict rungs always route to the group, because ℓ₅ is provably unavailable at the edge.
