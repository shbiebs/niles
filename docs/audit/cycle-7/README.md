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

Neither auditor writes production code. The deliverable is instructions for Claude Opus to execute.

The state both briefs are written against is Niles `d9c8699` (`c6/06a-lock-order`) and GBS
`e803b7d` (`c6/07-hold-index`).
