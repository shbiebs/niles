### Per case, at k = 16

| # | case | regime | rewrite | nested | unnested | ratio |
|---|---|---|---|---|---|---|
| 1 | exists, correlated | correlated | `exists-to-semi-join` | 16528 | 704 | 23.48x |
| 2 | exists, no matching group | empty inner | `exists-to-semi-join` | 288 | 288 | 1.00x |
| 3 | exists, with an inner filter | correlated | `exists-to-semi-join` | 9728 | 672 | 14.48x |
| 4 | exists, uncorrelated | uncorrelated | `exists-to-semi-join` | 16528 | 16640 | 0.99x |
| 5 | not exists, correlated | correlated | `not-exists-to-anti-join` | 16528 | 704 | 23.48x |
| 6 | not exists, empty inner | empty inner | `not-exists-to-anti-join` | 288 | 288 | 1.00x |
| 7 | not exists, with an inner filter | correlated | `not-exists-to-anti-join` | 9728 | 672 | 14.48x |
| 8 | not exists, uncorrelated | empty inner | `not-exists-to-anti-join` | 288 | 288 | 1.00x |
| 9 | in, correlated | correlated | `in-to-semi-join` | 16528 | 592 | 27.92x |
| 10 | in, against a duplicated inner | uncorrelated | `in-to-semi-join` | 16528 | 2304 | 7.17x |
| 11 | in, inner column all null | correlated | `in-to-semi-join` | 7248 | 384 | 18.88x |
| 12 | in, empty inner | empty inner | `in-to-semi-join` | 288 | 288 | 1.00x |
| 13 | not in, correlated, nulls on the right | correlated | `not-in-to-anti-join-with-null-witness` | 16528 | 1056 | 15.65x |
| 14 | not in, correlated, no nulls anywhere | correlated | `not-in-to-anti-join-with-null-witness` | 7248 | 736 | 9.85x |
| 15 | not in, uncorrelated, one null poisons everything | uncorrelated | `not-in-to-anti-join-with-null-witness` | 16528 | 2770 | 5.97x |
| 16 | not in, uncorrelated, no nulls | uncorrelated | `not-in-to-anti-join-with-null-witness` | 7248 | 1696 | 4.27x |
| 17 | not in, inner column entirely null | correlated | `not-in-to-anti-join-with-null-witness` | 7248 | 960 | 7.55x |
| 18 | not in, empty inner is vacuously true | empty inner | `not-in-to-anti-join-with-null-witness` | 288 | 544 | 0.53x |
| 19 | not in, inner filtered to nothing | empty inner | `not-in-to-anti-join-with-null-witness` | 512 | 992 | 0.52x |
| 20 | not in, a group that both matches and has a null | correlated | `not-in-to-anti-join-with-null-witness` | 16208 | 1274 | 12.72x |
| 21 | scalar sum, correlated | correlated | `scalar-to-outer-join-with-aggregate` | 16528 | 944 | 17.51x |
| 22 | scalar count over an empty group | empty inner | `scalar-to-outer-join-with-aggregate` | 288 | 432 | 0.67x |
| 23 | scalar min, with nulls in the group | correlated | `scalar-to-outer-join-with-aggregate` | 16528 | 944 | 17.51x |
| 24 | scalar max over a filtered inner | correlated | `scalar-to-outer-join-with-aggregate` | 9728 | 944 | 10.31x |

**Corpus total at k = 16:** 225376 → 37116 row-operations, 6.07x.

### The curve

Whole corpus, and the correlated regime alone. The corpus total grows sub-linearly because the uncorrelated cases stay quadratic in both plans and come to dominate a sum that mixes regimes; the correlated column is the one the claim is about.

| k | outer rows | inner rows | corpus nested | corpus unnested | corpus ratio | correlated ratio |
|---|---|---|---|---|---|---|
| 1 | 9 | 7 | 1396 | 1101 | 1.27x | **1.44x** |
| 4 | 36 | 28 | 15736 | 5388 | 2.92x | **4.29x** |
| 16 | 144 | 112 | 225376 | 37116 | 6.07x | **15.71x** |
| 64 | 576 | 448 | 3500416 | 397308 | 8.81x | **61.39x** |

### Crossover, per case

The smallest power-of-two scale at which the unnested plan does less work.

| case | crossover k |
|---|---|
| exists, correlated | 1 |
| exists, with an inner filter | 1 |
| not exists, correlated | 1 |
| not exists, with an inner filter | 1 |
| in, correlated | 1 |
| in, inner column all null | 1 |
| not in, correlated, nulls on the right | 1 |
| not in, correlated, no nulls anywhere | 1 |
| not in, inner column entirely null | 2 |
| not in, a group that both matches and has a null | 2 |
| scalar sum, correlated | 1 |
| scalar min, with nulls in the group | 1 |
| scalar max over a filtered inner | 1 |
