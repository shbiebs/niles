*Generated from the workspace by `thesis/gen-appendix-d.py`. Do not edit by hand.*

**`nilestream-ledger`**

```rust
pub struct Hasher256
pub fn sha256(bytes: &[u8]) -> [u8
pub fn chain_hash(parent: &[u8
pub fn hex(digest: &[u8
pub struct Frontier
pub struct Snapshot
pub type Minor
pub struct Epoch(pub u64)
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
pub trait Base
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
pub struct Violation
pub struct VerifyReport
pub fn verify(c: &Circuit) -> VerifyReport
```
