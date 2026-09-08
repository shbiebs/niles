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
