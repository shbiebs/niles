# E30 — the syntax study (cycle 14, R2-06; decision D-3)

*Rendered by `cargo run -p syntax-study --bin e30 -- report` from the rows in `results/E30-syntax/`, every one of which `make e30` regenerated at commit `09f60281641a`. The design, pre-registered before any program was written, is `docs/study/E30-syntax-design.md` (`5fbc173` the pre-registration, `563c3a6` amendment A1, `104ea46` amendment A2); every departure from it is listed under **Deviations** with its reason. Reproducing needs PostgreSQL 16 as the gate's `bench` role and the approved `prqlc` and `souffle` under `/opt/arms`.*

## The rule, evaluated

> The principal surface is the one that (i) expresses the most corpus tasks, then (ii) has the highest static-rejection rate under the mutation set, then (iii) the lowest check cost; ties are reported as ties. A surface that cannot express a task class is not principal for that class; new syntax is justified only for classes no surface expresses.

Applied per class, over the surfaces in scope for the class, and overall over the three surfaces in scope for all thirty tasks (PRQL and DL are in scope for the ten queries only, design §2). *Expresses* counts a program its checker accepts whose answer equals the oracle's, and — for NL and RS — a program its checker accepts that no Niles executor can run (A1's *unexecuted*), whose correctness is therefore not established; the verified count is shown beside it. Check cost ranks only the in-process checkers (SQL, NL, RS); PRQL's and DL's are external binaries, reported apart and never ranked against them (design §6). A checker's cost is its time per KLOC over its whole corpus (the class files alone are too few to time apart), and NL and RS share every non-query file.

| class | surface | expresses (verified) | static-rejection rate | check cost, µs per KLOC | position |
|---|---|--:|--:|--:|---|
| Q — queries | SQL | 10 of 10 (10) | 9.1% | 6650 |  |
| Q — queries | NL | 8 of 10 (8) | 0.0% | 2444 | cannot express the whole class: not principal for it |
| Q — queries | RS | 7 of 10 (7) | 0.0% | 1946 | cannot express the whole class: not principal for it |
| Q — queries | PRQL | 10 of 10 (10) | 0.0% | not ranked (external) |  |
| Q — queries | DL | 9 of 10 (9) | 77.4% | not ranked (external) | cannot express the whole class: not principal for it |
| T — transactions | SQL | 10 of 10 (10) | 2.3% | 6650 |  |
| T — transactions | NL | 10 of 10 (3) | 77.8% | 2444 |  |
| T — transactions | RS | 10 of 10 (3) | 77.8% | 1946 |  |
| V — view contracts | SQL | 5 of 5 (5) | 0.0% | 6650 |  |
| V — view contracts | NL | 5 of 5 (5) | 0.0% | 2444 |  |
| V — view contracts | RS | 5 of 5 (5) | 0.0% | 1946 |  |
| B — temporal reads | SQL | 5 of 5 (5) | 0.0% | 6650 |  |
| B — temporal reads | NL | 4 of 5 (4) | 0.0% | 2444 | cannot express the whole class: not principal for it |
| B — temporal reads | RS | 4 of 5 (4) | 0.0% | 1946 | cannot express the whole class: not principal for it |
| all 30 | SQL | 30 of 30 (30) | 3.0% | 6650 |  |
| all 30 | NL | 27 of 30 (20) | 56.5% | 2444 |  |
| all 30 | RS | 26 of 30 (19) | 60.3% | 1946 |  |

* **Q — queries:** **SQL**, by (ii) — (i) tied SQL, PRQL.
* **T — transactions:** **NL and RS, tied** — (i) and (ii) tied NL, RS; the two share this class's programs, so (iii) cannot separate them.
* **V — view contracts:** **NL and RS, tied** — (i) and (ii) tied SQL, NL, RS; the two share this class's programs, so (iii) cannot separate them.
* **B — temporal reads:** **SQL**, by (i).
* **all 30:** **SQL**, by (i).
* **New syntax:** not justified — every class has a surface that expresses all of its tasks.

**How much weight this bears.** Read the static-rejection rates with the mutant counts below them: a class with few mutants, or with many excluded as unexecuted, supports a ranking less than its percentage suggests, and the NL/RS transaction rate is measured on the three transactions a Niles executor could run (T01, T03, T07) plus every mutant the checker refused. Its range, were every unexecuted mutant caught or every one missed, is given in the safety table. The syntax rule in §6.25, B.1 and J.1 is restated from this result in R2-11 or round 3, never here (design §8).

## Expressiveness

| surface | Q | T | V | B | verified correct | unexecuted (A1) | not expressible |
|---|--:|--:|--:|--:|--:|--:|---|
| SQL | 10/10 | 10/10 | 5/5 | 5/5 | 30 | 0 | — |
| NL | 8/10 | 10/10 | 5/5 | 4/5 | 20 | 7 | Q07, Q09, B03 |
| RS | 7/10 | 10/10 | 5/5 | 4/5 | 19 | 7 | Q07, Q08, Q09, B03 |
| PRQL | 10/10 | not in scope | not in scope | not in scope | 10 | 0 | — |
| DL | 9/10 | not in scope | not in scope | not in scope | 9 | 0 | Q05 |

**Why each task is not expressible**, in the program's own `.none` file:

* **B03 NL.** A back-valued correction cannot be written: a `txn` block, `post`, `debit` and `credit` have no argument or clause for a posting's value date (the grammar's `Txn` is an idempotency key and a body), so the correction's legs can only take the default value date. The two reads — `.valid_at(v@2026-01-21)` and `.as_of(#5)` — are expressible (B02, B01); the write the task is about is not. SPEC-LANGUAGE §bitemporality records the axis as "Partial".
* **B03 RS.** A back-valued correction cannot be written: a `txn` block, `post`, `debit` and `credit` have no argument or clause for a posting's value date (the grammar's `Txn` is an idempotency key and a body), so the correction's legs can only take the default value date. The two reads — `.valid_at(v@2026-01-21)` and `.as_of(#5)` — are expressible (B02, B01); the write the task is about is not. SPEC-LANGUAGE §bitemporality records the axis as "Partial".
* **Q05 DL.** A Datalog relation is a set: the task's answer is ordered (largest first, ties by account), and Soufflé has no order or limit to state it with. The five-row set alone is expressible by counting, but it is not the task.
* **Q07 NL.** Niles has no window function on its SQL surface (`over (partition by …)` is a parse error, NL0001/NL0009), and a posting's epoch — the task's order — is not a column a query can read (finding F8). The SQL-92 form, a self-join summing the earlier postings of the same account and currency, resolves its aliases since Niles fix 5/5, but it can only order by `txn`, which is a different query that coincides with this one only where epochs and txn ids happen to agree.
* **Q07 RS.** The pipeline has no window stage, and a posting's epoch — the task's order — is not a column (finding F8). A self-join `.join(postings, |l, r| r.cur == l.cur && r.txn <= l.txn)` resolves each parameter to its own side since Niles fix 5/5, but it orders by `txn`, which is a different query that coincides with this one only where epochs and txn ids happen to agree.
* **Q08 RS.** No window or rank stage in the pipeline. The SQL-92 form needs every account compared with every other account of its desk: `cross_join` is refused (NL0516, "no product operator exists in the IR"), and a keyed join of the balance view with itself matches on its key, (acct, desk), so each account meets only itself. The SQL surface's comma from-list lowers to a join with empty keys, which the evaluator runs as a product (finding F10); the pipeline has no spelling for it.
* **Q09 NL.** `with recursive` does not parse on the SQL surface (NL0001, NL0002, NL0004, NL0511); the mapping table lists it as specified, not implemented.
* **Q09 RS.** `.fixpoint(|acc| …) guard measure(…)` is specified but has no lowering (NL0507), as the mapping table says.

**Why each unexecuted program is unexecuted:**

* T08 (NL and RS, one file): niles-interp refuses `a read of `loan_balance` (the interpreter has no relational tier)`.
* T10 (NL and RS, one file): niles-interp refuses `a read of `postings` (the interpreter has no relational tier)`.
* T04, T05 (NL and RS, one file): niles-interp refuses `hold`.
* T02, T06 (NL and RS, one file): niles-interp refuses `money arithmetic (checked tier only)`.
* T09 (NL and RS, one file): the interpreter cannot be handed a capability.

**Tokens and constructs**, over the tasks every compared surface expresses in the class (so each column counts the same tasks); the schema or preamble each surface's programs are checked with is counted once, separately. Tokens are the surface-neutral tokenizer's (`crates/syntax-study/src/tokens.rs`); constructs are a program's distinct keywords (the surface's list in `crates/syntax-study/corpus/keywords/`) plus its distinct operators, summed over programs.

| class | common tasks | surface | tokens | keywords | operators |
|---|---|---|--:|--:|--:|
| Q — queries | Q01 Q02 Q03 Q04 Q06 Q10 | SQL | 250 | 38 | 10 |
| Q — queries | Q01 Q02 Q03 Q04 Q06 Q10 | NL | 180 | 51 | 8 |
| Q — queries | Q01 Q02 Q03 Q04 Q06 Q10 | RS | 319 | 30 | 21 |
| Q — queries | Q01 Q02 Q03 Q04 Q06 Q10 | PRQL | 166 | 30 | 14 |
| Q — queries | Q01 Q02 Q03 Q04 Q06 Q10 | DL | 457 | 27 | 26 |
| T — transactions | T01 T02 T03 T04 T05 T06 T07 T08 T09 T10 | SQL | 1254 | 163 | 50 |
| T — transactions | T01 T02 T03 T04 T05 T06 T07 T08 T09 T10 | NL | 965 | 65 | 81 |
| T — transactions | T01 T02 T03 T04 T05 T06 T07 T08 T09 T10 | RS | 965 | 65 | 81 |
| V — view contracts | V01 V02 V03 V04 V05 | SQL | 352 | 68 | 7 |
| V — view contracts | V01 V02 V03 V04 V05 | NL | 315 | 57 | 23 |
| V — view contracts | V01 V02 V03 V04 V05 | RS | 315 | 57 | 23 |
| B — temporal reads | B01 B02 B04 B05 | SQL | 186 | 25 | 9 |
| B — temporal reads | B01 B02 B04 B05 | NL | 182 | 19 | 22 |
| B — temporal reads | B01 B02 B04 B05 | RS | 182 | 19 | 22 |

| surface | schema file | lines | tokens |
|---|---|--:|--:|
| SQL | `schema.sql` | 108 | 993 |
| NL | `schema.niles` | 39 | 185 |
| RS | `schema.niles` | 39 | 185 |
| PRQL | `schema_plain.sql` | 10 | 98 |
| DL | `schema.dl` | 15 | 103 |

PRQL has no data-definition language; its programs run over the plain schema in SQL (`schema_plain.sql`), counted with the SQL tokenizer.

## Safety — the mutation set

399 mutants, every site of M1–M6 in every expressible program (the site patterns are in `crates/syntax-study/src/mutate.rs` and listed below). Static-rejection rate = static ÷ (static + runtime + silent); equivalent and unexecuted mutants are reported and excluded (design §6, A1).

| surface | class | mutants | static | runtime | silent | equivalent | unexecuted | rate | range, every unexecuted missed – caught |
|---|---|--:|--:|--:|--:|--:|--:|--:|---|
| SQL | Q | 33 | 3 | 0 | 30 | 0 | 0 | 9.1% | — |
| SQL | T | 92 | 2 | 76 | 8 | 6 | 0 | 2.3% | — |
| SQL | V | 20 | 0 | 0 | 16 | 4 | 0 | 0.0% | — |
| SQL | B | 38 | 0 | 7 | 27 | 4 | 0 | 0.0% | — |
| SQL | all | 183 | 5 | 83 | 81 | 14 | 0 | 3.0% | — |
| NL | Q | 14 | 0 | 0 | 11 | 3 | 0 | 0.0% | — |
| NL | T | 63 | 35 | 10 | 0 | 0 | 18 | 77.8% | 55.6% – 84.1% |
| NL | V | 6 | 0 | 0 | 2 | 4 | 0 | 0.0% | — |
| NL | B | 6 | 0 | 0 | 4 | 2 | 0 | 0.0% | — |
| NL | all | 89 | 35 | 10 | 17 | 9 | 18 | 56.5% | 43.8% – 66.2% |
| RS | Q | 8 | 0 | 0 | 7 | 1 | 0 | 0.0% | — |
| RS | T | 63 | 35 | 10 | 0 | 0 | 18 | 77.8% | 55.6% – 84.1% |
| RS | V | 6 | 0 | 0 | 2 | 4 | 0 | 0.0% | — |
| RS | B | 6 | 0 | 0 | 4 | 2 | 0 | 0.0% | — |
| RS | all | 83 | 35 | 10 | 13 | 7 | 18 | 60.3% | 46.1% – 69.7% |
| PRQL | Q | 11 | 0 | 0 | 11 | 0 | 0 | 0.0% | — |
| PRQL | all | 11 | 0 | 0 | 11 | 0 | 0 | 0.0% | — |
| DL | Q | 33 | 24 | 0 | 7 | 2 | 0 | 77.4% | — |
| DL | all | 33 | 24 | 0 | 7 | 2 | 0 | 77.4% | — |

**SQL's run-time failures, split.** 43 of SQL's 83 runtime mutants were raised by PostgreSQL's analyser (SQLSTATE class 42: an undefined operator or function, a datatype mismatch) — when the view or function was created or the query was planned, before any row was read. The design's checker for SQL is SQL+C+L (`nilescheck_sql::check_all`), so these count as runtime; had PostgreSQL's analyser been counted as SQL's checker, SQL's overall rate would be 28.4%. The rest (40) were raised while running, most by the ledger's deferred conservation trigger (`P0001`).

**By operator**, all classes:

| surface | operator | mutants | static | runtime | silent | equivalent | unexecuted | rate |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| DL | M1 | 4 | 0 | 0 | 2 | 2 | 0 | 0.0% |
| DL | M3 | 29 | 24 | 0 | 5 | 0 | 0 | 82.8% |
| NL | M1 | 38 | 30 | 2 | 3 | 1 | 2 | 85.7% |
| NL | M2 | 22 | 0 | 8 | 0 | 0 | 14 | 0.0% |
| NL | M3 | 17 | 0 | 0 | 12 | 3 | 2 | 0.0% |
| NL | M4 | 3 | 0 | 0 | 2 | 1 | 0 | 0.0% |
| NL | M5 | 2 | 2 | 0 | 0 | 0 | 0 | 100.0% |
| NL | M6 | 7 | 3 | 0 | 0 | 4 | 0 | 100.0% |
| PRQL | M1 | 3 | 0 | 0 | 3 | 0 | 0 | 0.0% |
| PRQL | M3 | 8 | 0 | 0 | 8 | 0 | 0 | 0.0% |
| RS | M1 | 37 | 30 | 2 | 2 | 1 | 2 | 88.2% |
| RS | M2 | 22 | 0 | 8 | 0 | 0 | 14 | 0.0% |
| RS | M3 | 12 | 0 | 0 | 9 | 1 | 2 | 0.0% |
| RS | M4 | 3 | 0 | 0 | 2 | 1 | 0 | 0.0% |
| RS | M5 | 2 | 2 | 0 | 0 | 0 | 0 | 100.0% |
| RS | M6 | 7 | 3 | 0 | 0 | 4 | 0 | 100.0% |
| SQL | M1 | 124 | 0 | 63 | 55 | 6 | 0 | 0.0% |
| SQL | M2 | 23 | 0 | 20 | 1 | 2 | 0 | 0.0% |
| SQL | M3 | 26 | 3 | 0 | 22 | 1 | 0 | 12.0% |
| SQL | M4 | 4 | 0 | 0 | 3 | 1 | 0 | 0.0% |
| SQL | M5 | 2 | 2 | 0 | 0 | 0 | 0 | 100.0% |
| SQL | M6 | 4 | 0 | 0 | 0 | 4 | 0 | — |

**The site patterns** (`mutate.rs`):

| op | where it applies |
|---|---|
| M1 | every currency name in a word or a string, whole or `_`-separated (`amt_usd`, `Money<usd>`, `"usd"`, `'usd'`): usd → eur, eur → usd, jpy → usd |
| M2 | Niles: each `debit(` ↔ `credit(`. SQL, inside `insert into postings`: each `row(x)::c` has its sign toggled, and each bare money variable in a `values` tuple's `amt_c` position becomes `row(-(m).minor)::c` |
| M3 | each top-level conjunct of a condition, with its connective; a one-conjunct condition loses its clause. SQL `where`, `having`, a join's `on`; PL/pgSQL `if`/`elsif`; Niles `.where`/`.filter`/`.having` and a join closure's body, Niles `if`; PRQL `filter` and a join's condition; Soufflé rule bodies of two or more literals. An `if` with one conjunct is a site only when it guards a failure (the mutant makes it `false`) |
| M4 | Niles `#e`; SQL and PRQL a number compared with `epoch`: e → e + 1 |
| M5 | SQL `perform resolve_…(…);`, Niles `resolve …`; a `conserve` clause or a conservation or linearity annotation in a program |
| M6 | each consistency rung, as a word or a string's text, one step down: bounded < monotonic < read_your_writes < snapshot < serializable < ledger_consistent |

**What the classes contain**, read from the rows in `results/E30-syntax/mutants.tsv`:

* **A leg's sign (M2).** Neither checker refuses a flipped leg in these programs: both conservation solvers (Niles's, and SQL+C+L's, which is the same solver) read a parameter as a symbol that may be zero, so `2m ≠ 0` is *undecided* and discharged to the run-time seal, where both ledgers refuse the posting set: runtime, wherever the flip unbalances the set. SQL's T10 is the exception — its reversal is an `insert … select`, and a flipped sign copies the reversed legs unchanged, which conserves and is silent. On Niles, turning a `debit(..)?` into a `credit(..)?` is runtime for a different reason: the typechecker accepts `?` on a value that is not a `Result`, and the interpreter raises (F12).
* **A consumption removed (M5).** Refused statically on both SQL+C+L (NL0320) and Niles (NL0320), on T04 and T05 — the one operator every checker catches everywhere it applies.
* **A weakened contract (M6).** No checker refuses a view served one rung lower when nothing else reads it, and no single-session executor can tell the difference, so M6 is *equivalent* on V01, V02, V04 and V05 on every surface. Niles refuses it only where a function declares the rung it reads at (T08, T10: NL0310).
* **Datalog's grounding check (M3).** Soufflé refuses most dropped body literals because a variable is left ungrounded; that is a static check of Datalog's own, and it is why DL's query rate is high. PRQL has no checker beyond compilation, and every one of its mutants ran.
* **Equivalent mutants that depend on the data.** B04's value-date conjunct and its epoch + 1, T10's eur and jpy branches, and several currency swaps change nothing on this dataset (no posting of epoch ≤ 5 is valued after 2026-01-21, and none of epoch 6 on or before it; the reversed transaction is usd only). They are equivalent here, not in general; the design fixed one dataset.

## Speed — check cost

In-process, by `tools/memprobe`'s `e30cost` (E14's protocol). Host: Linux 6.18.44-fc-v50 x86_64 — 2 CPUs; toolchain `rustc 1.95.0 (59807616e 2026-04-14)`; measured at commit `09f60281641a` (clean worktree). 20 warm-up passes, then 200 timed passes; each pass checks every file once with each checker, alternating; in-process, release build; peak heap from E18's counting allocator.

| checker | files | lines read | median pass µs (MAD) | µs per KLOC | peak heap per check, max, bytes | peak heap per KLOC, max, bytes |
|---|--:|--:|--:|--:|--:|--:|
| SQL | 30 | 3781 | 25143 (536) | 6650 | 155407 | 1175374 |
| NL | 27 | 1371 | 3351 (110) | 2444 | 70407 | 1247596 |
| RS | 26 | 1338 | 2603 (96) | 1946 | 70407 | 1288353 |

A line is a line the checker reads: each SQL file is checked as the 108-line schema followed by the program, and each Niles file as its 39-line schema followed by the program, so most of SQL's lines are its schema's, re-checked every time — the E14 convention.

External, wall time per invocation (5 warm-up passes, 30 timed, the median per file), **not ranked against the in-process figures**:

| surface | binary | files | program lines | median ms per invocation | empty program ms | net ms per KLOC of program |
|---|---|--:|--:|--:|--:|--:|
| PRQL | prqlc 0.13.14 | 10 | 51 | 13.80 | 6.77 | 1380.0 |
| DL | Version: 2.5 (2.5) | 9 | 41 | 9.60 | 7.75 | 405.7 |

**Run-time speed is not measured** (design §6): NL and RS lower to the same IR, so their run time is identical by construction, and PRQL runs as the SQL it emits.

## The dataset, loaded

One dataset from seed 1 (`e30 gen`; fixtures in `crates/syntax-study/corpus/data/`). Each surface's rows were read back from its own executor and compared with the dataset's (design §4):

| surface | relation | rows | SHA-256 of the canonical rows | equal |
|---|---|--:|---|---|
| SQL | accounts | 20 | `2b11b272941df47e3145007b9dbb493107e8a8ca3de9083eb11da947a6d0f8f8` | yes |
| SQL | epochs | 200 | `a88b2e2101f4ef9a80e79e1fd43fe96284d3d2ccf8c0b2565e3ba27d26e9fdff` | yes |
| SQL | holds | 6 | `6cdbf91bdaae2b76c68cd00285147ad741a2b76391fcacc8285e0d32fcd681cd` | yes |
| SQL | parties | 12 | `24845c1c0591f8de0be58e2e7e67e217495308087477ae17b0f63e2a9c45bfaa` | yes |
| SQL | postings | 200 | `496da2ebd1f326a6ddb0adf760b3279e2a87886daeb3a907b366df13a82244db` | yes |
| PRQL | accounts | 20 | `2b11b272941df47e3145007b9dbb493107e8a8ca3de9083eb11da947a6d0f8f8` | yes |
| PRQL | epochs | 200 | `a88b2e2101f4ef9a80e79e1fd43fe96284d3d2ccf8c0b2565e3ba27d26e9fdff` | yes |
| PRQL | holds | 6 | `6cdbf91bdaae2b76c68cd00285147ad741a2b76391fcacc8285e0d32fcd681cd` | yes |
| PRQL | parties | 12 | `24845c1c0591f8de0be58e2e7e67e217495308087477ae17b0f63e2a9c45bfaa` | yes |
| PRQL | postings | 200 | `496da2ebd1f326a6ddb0adf760b3279e2a87886daeb3a907b366df13a82244db` | yes |
| DL | accounts | 20 | `2b11b272941df47e3145007b9dbb493107e8a8ca3de9083eb11da947a6d0f8f8` | yes |
| DL | epochs | 200 | `a88b2e2101f4ef9a80e79e1fd43fe96284d3d2ccf8c0b2565e3ba27d26e9fdff` | yes |
| DL | holds | 6 | `6cdbf91bdaae2b76c68cd00285147ad741a2b76391fcacc8285e0d32fcd681cd` | yes |
| DL | parties | 12 | `24845c1c0591f8de0be58e2e7e67e217495308087477ae17b0f63e2a9c45bfaa` | yes |
| DL | postings | 200 | `496da2ebd1f326a6ddb0adf760b3279e2a87886daeb3a907b366df13a82244db` | yes |
| NL, RS | accounts | 20 | `2b11b272941df47e3145007b9dbb493107e8a8ca3de9083eb11da947a6d0f8f8` | yes |
| NL, RS | holds | 6 | `6cdbf91bdaae2b76c68cd00285147ad741a2b76391fcacc8285e0d32fcd681cd` | yes |
| NL, RS | parties | 12 | `24845c1c0591f8de0be58e2e7e67e217495308087477ae17b0f63e2a9c45bfaa` | yes |
| NL, RS | postings | 200 | `496da2ebd1f326a6ddb0adf760b3279e2a87886daeb3a907b366df13a82244db` | yes |
| NL, RS | epochs |  | — | n/a: system time, not a column (A1) |

## Deviations from the design

* **D1 — a relational read inside a Niles function is unexecuted.** `niles-interp` has no relational tier, so a function reading a view or a relation (T08's `loan_balance.get`, T10's `postings.where`) reaches it as an unbound name; the harness classes this as unexecuted rather than as the program's failure. A1 named `hold`, `resolve`, `fx` and `authorize` only.
* **D2 — money arithmetic outside the checked tier is unexecuted.** `niles-interp` also refuses money arithmetic (T02's `m - fee`, T06's `m1 + m2 + m3`) outside its checked tier. As found; A1 did not list it. T03 was predicted unexecuted by A1 (`fx`) and is not: its program states the settlement as four legs, and runs.
* **D3 — B03 is not expressible in NL/RS, not unexecuted.** A2 expected B03's Niles program to be checked and unexecuted. Writing it found that no Niles form sets a posting's value date (F9), so there is no program to check.
* **D4 — the mutation site patterns were written after the corpus.** The design fixes the operators and says their per-surface site patterns are data "fixed with this design"; they were written after every program existed and before any mutant was counted, and are listed above. One trial run on Q02 was made to test the classifier; it found F11, its output was discarded, and the full run was made from the start after the fix.
* **D5 — the Niles checker was fixed before the mutation stage.** Writing the corpus found five silent wrong answers in Niles's lowering (F1, F4–F7), and the mutation trial a sixth (F11). The author decided (2026-09-30) to fix them before the mutation stage: six commits, "Niles fix 1/5" … "6/6", each with a guard test that fails against the old lowering. The corpus was re-run after the fixes; three programs were restored to the spelling first written and Q08 NL became expressible (`crates/syntax-study/corpus/builder-fixes.tsv`). The checkers were frozen from the full mutation run on.
* **D6 — the Niles harness gained a typed call boundary after the first full mutation run.** The first full run classed four NL/RS mutants of T03 as equivalent: M1 changed a parameter's declared currency, and the interpreter's `call` did not compare the argument's currency with the declaration, so the program ran on the harness's `usd` value regardless. PostgreSQL refuses the same call at run time ("function … does not exist"). The harness now refuses a call whose argument currency differs from the parameter's, as a typed caller would, and the four mutants are runtime. This changed Niles's numbers against Niles (equivalent is excluded from the rate; runtime counts against it), and it is the only change made after a full mutation run.

## Findings

Found while writing and running the corpus, in order; each names the program that found it.

| id | finding | status |
|---|---|---|
| F1 | Niles SQL surface: an aggregate call in `having` (`having sum(amt) < 0.00 usd`) lowered to an argument-less call and dropped every group (Q02 NL: 0 rows for 2) | fixed, Niles fix 1/5, NL0518 |
| F2 | Niles has no currency literal for a predicate: a bare `usd` is read as a column (NL0501), a string must be used, and an undeclared currency string was accepted; the SQL surface lexes only double-quoted strings | the undeclared string is refused since fix 6/6 (NL0521); the rest stands |
| F3 | `niles_ir::eval` passes `AsOf`/`ValidAt` through; `niles-interp` refuses `hold`, `resolve`, `fx`, `authorize` | A1; the harness applies the pins |
| F4 | a keyed join whose two sides have keys of different length never matches, on either surface (Q03: 0 rows for 12) | fixed, Niles fix 2/5, NL0519 |
| F5 | pipeline `.order_by(\|r\| (-r.sum, r.acct))` lowered to an ascending order on the first column (Q05 RS) | fixed, Niles fix 3/5, NL0509; descending is `desc(..)` |
| F6 | a join to a grouped count matched on the aggregate's *input* key, so account ids were matched against counts (Q04 RS: 10 rows for 2) | fixed, Niles fix 4/5 |
| F7 | a self-join's qualifiers were ignored, so both sides resolved to the left (Q07/Q08) | fixed, Niles fix 5/5, NL0520 |
| F8 | no window functions on either Niles surface, and a posting's epoch is not a column | stands; Q07 and Q08 RS are not expressible |
| F9 | no Niles form sets a posting's value date, so a back-valued correction cannot be written | stands; B03 is not expressible |
| F10 | the SQL surface refuses `cross join` (NL0516, "no product operator") but lowers a comma from-list to a join with empty keys, which the evaluator runs as a product; the pipeline has no product spelling | recorded for the author, not fixed |
| F11 | the one scalar evaluator (reference evaluator and server scan fold) read every string literal as `0`, so `where cur = "eur"` answered the usd rows and `like` compared with 0 (found by the Q02 mutation trial) | fixed, Niles fix 6/6, NL0521 |
| F12 | the typechecker accepts `?` on a value that is not a `Result` (`credit(a, m)?`); the interpreter raises at run time | recorded, not fixed (the checker is frozen for the study) |
| F13 | a money literal's currency is dropped when evaluated, and a `Money` column of undeclared currency compares with any currency's literal: Q02's `having sum(amt) < 0.00 eur` over usd rows is accepted and answers as `< 0` | recorded, not fixed |

## Not measured, and said so

Readability and learnability (no user study). The programs were written by the builder of two of the checkers — mitigated by the pre-registration, the mechanical mutation and the independent oracle, not removed. Programs were written for correctness, not brevity, and none was shortened after its token count was known. The study has one dataset; equivalence is equivalence on it.
