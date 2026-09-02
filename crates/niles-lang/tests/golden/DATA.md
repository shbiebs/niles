# The fixed dataset every golden case is evaluated on

Written here so the `.expected` files can be read without running anything, and so a
change to the data is a change to a file somebody has to review rather than a constant
buried in a test.

`t(k, v, n)` — `n` is nullable, and two rows share `k = 1` so `distinct` and the
aggregates have something to do:

| k | v | n |
|---|---|---|
| 1 | 10 | 100 |
| 1 | 10 | null |
| 2 | 20 | 200 |
| 3 | 30 | null |

`u(k, w)` — `k = 3` is missing, so the outer joins differ from the inner one, and `k = 4`
has no match on the left:

| k | w |
|---|---|
| 1 | 1 |
| 2 | 2 |
| 4 | 4 |

`edges(src, dst)` — a six-node graph with a cycle (5 → 6 → 5), so a transitive closure that
did not clamp weights would never converge:

| src | dst |
|---|---|
| 1 | 2 |
| 2 | 3 |
| 3 | 4 |
| 5 | 6 |
| 6 | 5 |
