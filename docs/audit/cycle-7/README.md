# Cycle 7 audit briefs

Two auditors read the same two repositories **independently**, from different briefs, and each
produces a work order. The author reconciles the two afterwards; that reconciliation is what cycle 6
called the consolidated work order.

| brief | auditor | territory |
|---|---|---|
| [`fable.md`](fable.md) | Fable | the empirical half — concurrency, memory, locks, storage, allocation, latency, and the harnesses that produce them |
| [`gpt-astra.md`](gpt-astra.md) | GPT 6 Astra | the structural half — claims versus code, the type system, the thesis text, GBS layering, the bootstrap |

The split is deliberate and the overlap is too. §8 of each brief names four things **both** must
check independently: cycle 6's trust in F-12, F-13 and F-14 came from two audits finding them
separately, and its sharpest single insight came from the two *disagreeing* about a ratio and each
stating its evidence class.

**Both auditors run [`preflight.sh`](preflight.sh) first**, in whatever container they audit from,
and paste its output at the top of their work order. It is the same script for both, so the two
environment manifests can be laid side by side when the work orders are reconciled — which matters,
because cycle 6's sharpest disagreement (a group-commit ratio of 4.1× against 8.2×) resolved only
once each auditor's mount options were on the table. It reports the host and the *cgroup-granted*
core count, the filesystem under the tree, a barrier probe with a verdict on whether the container
can produce durability evidence at all, the toolchain, PostgreSQL, egress, both tree hashes, and an
admissibility table to fill in.

```
bash docs/audit/cycle-7/preflight.sh /path/to/niles /path/to/gbs
```

Neither auditor writes production code. The deliverable is instructions for Claude Opus to execute.

The state both briefs are written against is Niles `d9c8699` (`c6/06a-lock-order`) and GBS
`e803b7d` (`c6/07-hold-index`).
