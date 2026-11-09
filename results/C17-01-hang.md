# C17-01: the `concurrent_differential` hang, before and after the fixture fix

*Cycle 17, round 2. Host: Linux 6.18.44 x86_64, 2 cores, 8 GB; `cargo 1.97.0 (c980f4866 2026-06-30)`.*

**The checkpoint interval is not applicable:** the test runs the REV runtime over an in-memory fake base, with no ledger and no checkpoints.

The test is `rev::concurrent_differential::every_answer_matches_an_independent_fold_at_its_own_anchor`
(`crates/nilestream-core/src/rev.rs`). Each run is the debug test binary under `timeout 20`, with
`--exact`; exit 124 is a hang.

| build | runs | passed | hung | failed | wall time |
|---|---|---|---|---|---|
| `c5bba2d` (before; cycle 17 round 1, 1.95.0 build) | 40 | 37 | 3 | 0 | 71 s |
| C17-01 (after; this branch, 1.97.0) | 200 | 200 | 0 | 0 | 6 s |

**The cause** (round 1, thread stacks of one hung run, in `cycle-17-evidence.bundle`):
- the advancer was inside `Runtime::advance` → `apply_epoch` → the fake base's `deltas_at`;
- `deltas_at` scanned every row sealed so far, so catching up was quadratic;
- the writer was unbounded and was blocked on the rows lock;
- one reader was waiting on the runtime mutex, which the advancer kept re-taking.

That is a harness livelock, not an engine deadlock.

**The fix is in the test fixture only:**
- rows are indexed by epoch, so `deltas_at` is O(deltas), held by a count-probe test;
- the writer stops sealing at four times the readers' target and then idles.

The test's two preconditions are unchanged: the frontier must move 120 epochs under the
readers, and 1,500 comparisons must be made.

**Engine note, not fixed (DB-6):** `Runtime::advance` holds the runtime lock for a whole catch-up
(`rev.rs`, `pub fn advance`), so a long catch-up delays every reader behind it.

Command:

```sh
B=$(ls -t target/debug/deps/nilestream_core-* | grep -v '\.d$' | head -1)
for i in $(seq 1 200); do timeout 20 "$B" --exact \
  rev::concurrent_differential::every_answer_matches_an_independent_fold_at_its_own_anchor; done
```
