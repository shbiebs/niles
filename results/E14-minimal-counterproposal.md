# E14 — The Minimal Counterproposal

*Can reconstructible epoch-anchored views be built with current Rust and SQL? And if they
can, what is left for a new language and a new engine to do?*

Reproduce with `./crates/counterproposal/run.sh` against PostgreSQL 16.13 and a built
`nilesc`. The schema is `crates/counterproposal/sql/schema.sql`; the Niles corpus is
`crates/counterproposal/niles/`.

## Method

A differential defect corpus. Thirteen defect classes in Niles, **ten of them written twice** — the paired copy against a
**good-faith** PostgreSQL 16 schema and once in Niles — recording for each the *stage* at
which it is caught: compile time, runtime, or never.

Good faith matters, so the PostgreSQL side is built with the strongest tool the system
offers for each job, and where PostgreSQL has a better idiom than Nilestream's, it is used:

* Money is `numeric(38,4)`, not `float8`. This is **at least as good** as i128 minor units
  and arguably better, because the scale is visible in the column type.
* Immutability is a `BEFORE UPDATE OR DELETE` trigger raising an exception.
* Idempotency is a unique index — **better than Nilestream's**, which keeps its window in
  memory and loses it on restart.
* Conservation is a `DEFERRABLE INITIALLY DEFERRED` constraint trigger, so a transaction
  may be temporarily unbalanced mid-flight, which is the correct semantics.
* Reconstruction is a `STABLE` plpgsql function folding the suffix after the newest
  checkpoint — SC7 expressed as a query.
* The REV is a table used as a cache, with a `state ∈ {present, hole, pending}` column.
  **SQL's own three-valued logic turns out to be an asset here**: a NULL value beside a
  non-null version *is* honest absence, expressed natively.

## Part 1 — the mechanism works

| Property | Result in PostgreSQL 16 |
|---|---|
| Reconstruction equivalence under eviction | **0 divergences** across 50 keys |
| Honest absence (value dropped, version kept) | 50 holes: **50 versions kept, 0 values kept** |
| Checkpoint-bounded reconstruction | **5 shared buffer hits** per reconstruction at C = 16 |
| Maintenance proportional to deltas, not view size | 100 deltas applied, 880 skipped, over 500 epochs |
| Conservation violation | caught (deferred trigger, at COMMIT) |
| Mutation of the ledger | caught (trigger) |

**The answer to the first question is yes.** Reconstructible epoch-anchored views —
partial materialization, honest absence, anchored reconstruction, per-key checkpoints,
delta-proportional maintenance — are implementable in stock PostgreSQL today, and they
work. Nothing in the mechanism requires a new engine.

This is the finding that most weakens this thesis's own emphasis, and it is reported first
for that reason.

## Part 2 — the defect corpus

`compile` / `check` = rejected before the program can run (`check` for a checker over SQL).
`runtime` = raises when executed. `never` = accepted and executed, with the stated
consequence. Five columns since cycle 14, R2-05: **PostgreSQL** alone; **PG + catalog checker**
(R2-04, `nilescheck-sql --catalog-only`); **SQL+C+L** (R2-05: the catalog checker plus
linearity, conservation through `niles-lang`'s solver, and capabilities, over the same
annotated SQL); **Niles**; and **Rust** (d14 only — round 1's class-B probe, re-run by the
gate; the thirteen classes have no Rust arm).

| # | Defect | PostgreSQL | PG + catalog checker | SQL+C+L | Niles | Rust | SQL+C+L code | Niles code |
|---|---|---|---|---|---|---|---|---|
| D1 | Transaction that does not balance | **runtime** | **not caught** | **check** | **compile** | — | `NL0300` | `NL0300` |
| D2 | Adding USD to EUR in a query | **never** | **check** | **check** | **compile** | — | `NL0250` | `NL0250` |
| D3 | `100.50 jpy` — half a yen, where JPY has scale 0 | **never** — persisted | **check** | **check** | **compile** | — | `NL0240` | `NL0240` |
| D4 | Mixed-currency transaction: 100 USD → 100 EUR | **runtime** — rolled back | **not caught** | **check** | **compile** | — | `NL0300` | `NL0300` |
| D5 | Strict view derived from a bounded-stale view | **never** | **check** | **check** | **compile** | — | `NL0311` | `NL0311` |
| D6 | View predicate reading the wall clock | **never** | **check** | **check** | **compile** | — | `IR013` | `NL0205`, `IR013` |
| D7 | Materializing that view | **never** | **check** | **check** | **compile** | — | `NL0310` | `NL0310` |
| D8 | Filtering on a column that should be encrypted | **never** | **check** | **check** | **compile** | — | `NL0260` | `NL0260` |
| D9 | Overdraft with no authorization | **never** — persisted | **not caught** | **check** | **compile** | — | `NL0312` | `NL0312` |
| D10 | `drop trigger` removes conservation | **never** — money destroyed | **check** | **check** | **compile** | — | `NL0211` | `NL0211`, `NL0300` |
| D11 | `update` against the ledger | **runtime** | **check** | **check** | **compile** | — | `NL0230` | `NL0230` |
| D12 | `ledger_consistent` + `spilled` contract | **not expressible** | **check** | **check** | **compile** | — | `NL0220` | `NL0220`, `IR010` |
| D13 | Missing anchor index on a demand view | **never** (silent slowdown) | **check warning** | **check warning** | **compile warning** | — | `NL0223` | `NL0223` |
| D14.1 | A hold made and never bound (bare call) | **never** — hold left open | **not caught** | **check** | **compile** | **compile** (`unused_must_use`) | `NL0320` | `NL0320` |
| D14.2 | A hold bound to a discard (`let _ =` / `_ :=`) | **never** — hold left open | **not caught** | **check** | **compile** | **accepted** | `NL0320` | `NL0320` |
| D14.3 | A hold bound, read, and dropped | **never** — hold left open | **not caught** | **check** | **compile** | **accepted** | `NL0320` | `NL0320` |
| D14.4 | A hold explicitly dropped or ignored | **never** — hold left open | **not caught** | **check** | **compile** | **accepted** | `NL0320` | `NL0320` |
| D14.5 | A hold forgotten (`mem::forget` or its nearest spelling) | **never** — hold left open | **not caught** | **check** | **compile** | **clippy** (`forget_non_drop`; accepted once `Hold` has a destructor) | `NL0320` | `NL0320` |

**Totals.** PostgreSQL: 3 caught at runtime, 0 at compile time, 9 never, 1 not expressible.
PostgreSQL + checker: 10 of 13 caught at check time, 3 not caught (d1 and d4 need the conservation solver, d9 a capability; SQL+C+L adds both, R2-05).
PostgreSQL + checker, d14: 0 of 5 spellings caught (the catalog checker has no rule over a function's paths).
SQL+C+L: 13 of 13 classes caught at check time (12 refused, 1 as a warning) and 5 of 5 d14 spellings refused.
Niles, d14: 5 of 5 spellings refused at compile time (NL0320). Before R2-05's fix of NL0320, on the compiler of `2699969`, Niles refused 3 of 5: the bare call and `let _ = hold(..)?` were accepted with no diagnostic.
Rust, d14: rustc with `deny(unused_must_use)` refuses 1 of 5; `clippy -D warnings` refuses 2 of 5, and the `mem::forget` spelling only while `Hold` has no destructor.
PostgreSQL, d14: 0 of 5 spellings refused at definition time or at run time; each runs and leaves the hold open.

Each of those lines is required verbatim by the test that counts it: `e14_column.rs` and
`e14_table.rs` (the SQL columns), `bank-bench/tests/counterproposal.rs` (Niles),
`e14_rust.rs` (Rust, by running `rustc` and `clippy-driver`), and `postgres_agrees.rs`
(PostgreSQL, by creating and running each spelling). The cells of the table are the same runs'
per-file verdicts. The d14 corpus is `crates/counterproposal/d14/` — five spellings in each
language plus a control every checker must accept, committed in `2699969` before any SQL+C+L
rule existed.

**The "PostgreSQL + checker" column (cycle 14, R2-04)** is `crates/nilescheck-sql` — a
hand-written PostgreSQL 16 SQL and PL/pgSQL parser (no external dependency, the author's
decision of 2026-09-28) and the rules over it — run on `crates/counterproposal/sql-checked/`:
the same thirteen classes written in stock PostgreSQL 16 DDL with the conventions the checker
reads (money as a composite type per currency, a ledger by comment, contracts as rows of
`serve_contract`). Counted by `crates/nilescheck-sql/tests/e14_column.rs`, which requires the
line above verbatim. PostgreSQL alone accepts every one of those thirteen texts at definition
time (`tests/postgres_agrees.rs` holds that too): with composite money types it refuses
`usd + eur` in a *view* at `CREATE`, but the same sum inside PL/pgSQL is created and fails
only when run, and `row(100.5)::jpy` is rounded into the bigint without a word — the three
cells round 1 measured. The checker reaches both at check time. Two cells differ from Niles
by stage, not by outcome: D10 is caught when the `drop trigger` is in the checked script,
which is where a migration puts it; a drop typed at a live console is not in any script and
PostgreSQL's column still describes it. What the catalog checker cannot reach is exactly what
E14's Part 3 predicted: conservation of a transaction's postings and an overdraft authority,
which is a capability. R2-04 wrote that SQL "cannot type" a capability; R2-05 showed that was
an assumption, not a measurement (next section).

## Part 2b — SQL+C+L, and the pre-registered language rule (cycle 14, R2-05)

**The question** (decision D-2, 2026-09-27): does Niles need to exist as a language, or is it a
checked, annotated dialect of SQL? The rule was written before round 2 measured anything:
*if SQL+C+L refuses, at check time, every d14 spelling and every E14 class the Niles checker
refuses, the language question closes as (d) — Niles is a checked, annotated dialect of SQL,
and its grammar is retired to a specification in round 3; otherwise it closes as (b).*

**The arm.** SQL+C+L is `nilescheck-sql`'s default (`check_all`): R2-04's catalog checker plus
three rule families over the PL/pgSQL and `language sql` bodies it parses. Every annotation is
a stock PostgreSQL comment, so PostgreSQL accepts every annotated script unchanged
(`postgres_agrees.rs`):

* **Linearity** (`src/linear.rs`): a domain commented `linear hold` is consumed exactly once on
  every committing path, by a function commented `consumes hold` — NL0320 when made and not
  bound, overwritten while live, live at a `return` or at `end`, or live on one arm of a branch
  only; NL0321 when consumed twice or inside a loop; NL0322 when an iteration leaves one live.
  A path that raises commits nothing and owes nothing.
* **Conservation** (`src/conserve.rs`): the legs a body inserts into a ledger, grouped by
  `txn`, are handed to `niles-lang`'s currency-row solver unchanged — the same rows, joins,
  loop rule and verdicts, so the same NL0300, and the same honest *undecided* where the solver
  cannot see through an amount or the arms of a branch disagree (`tests/r205.rs` records that
  inherited limit).
* **Capabilities** (`src/capability.rs`): calling a function commented `requires overdraft`
  without a parameter of a domain commented `capability overdraft` is NL0312; casting,
  assigning or returning such a value outside a function commented `grants overdraft` is
  NL0330. That is the whole of Niles's "unforgeable" too.

`d9`'s checked copy used to debit a million directly, which no arm can tell from an ordinary
large debit; it now grants an overdraft without authority, as Niles's `d9` has always written
it. R2-04's column does not move (its checker reads none of the new comments).

**Niles, measured before and after.** On the compiler of `2699969` (the corpus commit, before
any change to `niles-lang`), Niles refused three d14 spellings and accepted two — `hold(..)?;`
and `let _ = hold(..)?;` — with no diagnostic: NL0320 tracked named bindings only. The author
chose (2026-09-29) to record that and fix it in this card; `275c310` makes both NL0320, and the
rule is evaluated against the fixed checker, which is the higher bar for SQL.

**Outcome of the rule.** The Niles checker refuses D1–D12 and all five d14 spellings, and warns
on D13. SQL+C+L refuses D1–D12 and all five d14 spellings, and warns on D13. **Every class and
spelling Niles refuses, SQL+C+L refuses at check time: the rule selects (d)** — Niles is a
checked, annotated dialect of SQL, and its grammar is retired to a specification in round 3.
Claim H-L1 in `thesis/status.toml` carries the verdict.

**Check cost** (the rule's second clause: above 2× the other checker's time per KLOC is
reported as a cost, not a veto): `results/E14-checkcost.md`, measured in-process on this
container, gives the ratio and states which, if either, is above 2×.

**What (d) does not say, stated with it.**

* **It is a rule over this corpus.** Eighteen defect programs and two controls, written by the
  builder of both checkers. The d14 corpus was committed before any SQL+C+L rule existed and
  the SQL spellings are each language's nearest equivalent of the same five Rust forms — but
  the checker was then built knowing the corpus, which is the strongest threat to this result.
  A corpus written by someone else, against the published conventions, is the test that
  would move it.
* **The annotations are the language.** SQL+C+L catches D9 and D14 because the script says,
  in comments, which domain is linear and which function consumes it; a script that omits the
  comments is ordinary SQL and is checked as such. Niles puts the same facts in its grammar,
  where they cannot be omitted. The rule does not price that difference; it was written not to.
* **Reach ends where the script ends.** Both checkers check programs. A console session can
  call `authorize_overdraft` with a cast capability, as a Niles user can run a different
  program; neither checker is a runtime guard.

Niles: **12 at compile time, 1 as a compile-time warning, 0 accepted in silence, 0 with no
spelling in the language.**

**Two of those cells moved in cycle 13 and neither moved because anyone was looking at this
table.**

* `D6` was the one case Niles accepted with no diagnostic at all: a view predicate calling
  `month_start()`, a helper that does not exist. The language has no `now()`, so the defect
  has no direct spelling — but an unknown function in a view predicate was simply not
  checked, which is a different thing, and this document counted it as a gap in the checker
  rather than a win. L-1 taught `resolve` to report an unbound name and an undeclared
  function, and `month_start` became `NL0205`. The instance fell when the hole did.
* `D10` was listed as **not expressible** in Niles, on the reasoning that a language with no
  run-time rule-dropping has no way to write "drop the conservation rule". The Niles file
  writes the nearer thing — a `ledger` declared with no `conserve` clause — and that is
  refused with `NL0211` and `NL0300`. A rule that cannot be dropped at run time because it
  cannot be *omitted at compile time* is the stronger result, and the cell was
  under-reporting it.

**And the numbers above were being produced by the wrong compiler.**
`crates/bank-bench/tests/counterproposal.rs` took whatever `nilesc` binary it found in
`target/`, preferring `release`, and built one only if none existed. In the working
container that binary was dated two weeks and six cycles before HEAD, so every verdict this
table carried for six cycles came from a cycle-8 compiler. A stale binary is a present
binary, and presence was the only thing checked. The test builds the compiler in its own
profile first now.

PostgreSQL wins one comparison outright: **D4 is caught**, because the deferred trigger
groups by `(txn, cur)` and both groups are non-zero. That is a correct and complete
detection of a cross-currency imbalance, at COMMIT.

### D10 is the one to look at

```sql
drop trigger postings_conserve on postings;
insert into postings (...) values (..., -500, ...);   -- one leg, no counterpart
```

```
 ledger_total_should_be_zero
-----------------------------
                   -500.0000
```

One DDL statement, and five hundred dollars ceases to exist. The ledger's control total is
now non-zero and nothing in the database will ever say so again.

This is not a criticism of PostgreSQL; a trigger is *supposed* to be droppable. It is an
observation about **where the invariant lives**. In the SQL version conservation is a
*runtime object* that a migration, a `pg_restore`, a replication tool that omits triggers,
or an operator under pressure can remove. In Niles it is a property of the *program text*:
`conserve per (txn, cur)` is checked when the program is compiled, and a program without a
balancing posting has no executable form to remove the check from.

## Part 3 — what PostgreSQL structurally cannot do

Three gaps that are not a matter of effort.

**Per-key refresh.** `REFRESH MATERIALIZED VIEW` is wholesale; there is no key argument.
PostgreSQL's own materialized views therefore cannot be REVs, which is why the schema above
hand-rolls one as a table. The hand-rolled version has the right complexity — but it is
application code, unverified by the database, and every team writes it again.

**Per-view consistency.** PostgreSQL isolation is a property of a *transaction*, not of a
*view*. There is no way to say "this balance may be four epochs stale and that one may not",
so a query touching both reads both at one snapshot regardless of intent. The consistency
ladder has no SQL spelling, which is why D5 is undetectable: there is nothing to detect
against.

**No stable read anchor.** `xmin` and `pg_current_wal_lsn()` exist but neither is a
user-visible, stable epoch an answer can be stamped with and re-read at. The `version`
column in the schema above is one that had to be invented.

## Part 4 — throughput

| | PostgreSQL REV | Nilestream REV |
|---|---|---|
| Reads/sec | 4,749 | in-process, not comparable |
| Maintenance | 100 applied / 880 skipped over 500 epochs | 1,338 applied / 2,662 skipped |
| Complexity | O(deltas) | O(deltas) |

The throughput figures are **not** a fair comparison and are not offered as one: the
PostgreSQL number includes SQL parsing, plpgsql interpretation and MVCC on every read,
while Nilestream's runtime is a function call. The figure worth reading is the last row:
**both have the right complexity.** A performance argument for a custom engine is not
available from this experiment, and the thesis should not make one.

## What this experiment settles

**Q: Can REVs be implemented only with current Rust and SQL?**
Yes, for the mechanism. Partial materialization, honest absence, anchored reconstruction,
checkpoints and delta-proportional maintenance all work in PostgreSQL 16, and the
reconstruction-equivalence property holds exactly. Anyone who wants REVs can have them
today without adopting anything from this thesis.

**This experiment contains no Nilestream measurement.** Part 1 builds the REV mechanism in
stock PostgreSQL and measures it against itself; Part 2 scores a defect corpus by what each
side's *static* tooling catches. Neither half runs the engine. A sentence citing E14 as
evidence about Nilestream's performance, or as a comparison between the two engines, is
citing an experiment that does not contain one.

**Q: Do we need an SQL replacement, or only a MySQL/PostgreSQL replacement with REVs?**
Neither framing survives. The evidence says the *opposite* of the thesis's own emphasis:

* The **engine** case is the weaker one. The mechanism runs in PostgreSQL with the right
  asymptotics. What a custom engine buys is that the REV is a first-class object the
  database maintains and verifies, rather than application code each team rewrites — a real
  benefit, but an engineering one, not a capability one.
* The **language** case is the stronger one. Seven of the ten paired defect classes are
  undetectable in SQL *at any stage*, because SQL has nothing to state the property
  against: no per-currency scale in the type, no per-view rung, no linearity, no effect
  row, no reproducibility obligation on a predicate. These are not gaps PostgreSQL could
  close with more triggers; a trigger is a runtime check, and D10 shows what a runtime
  check is worth. Three further classes are not paired at all, and that is the sharper
  version of the same point: `d12` (an infeasible serve contract) and `d13` (a missing
  anchor index) are properties SQL cannot state, so there is no second copy to write.
  **Revised by cycle 14 (R2-04, R2-05).** That paragraph compared the language with SQL
  *alone*. With the properties stated as comments and a checker reading them, SQL reaches
  every class — D12 and D13 as rows of `serve_contract`, linearity and capabilities as
  commented domains, conservation through the same solver — and the pre-registered rule of
  Part 2b selects (d). What remains of the language case is the difference the rule does not
  price: in Niles the facts are grammar and cannot be left out; in SQL+C+L they are comments
  and can.

**Q: What would it take to prove Niles should exist?**
This table is the shape of the argument but not yet a proof, and three things are missing.

1. **A frequency premise.** The table shows these defects are *undetectable*, not that they
   are *common*. A defect class nobody writes costs nothing to miss. Establishing frequency
   needs a corpus of real banking code or an incident study, and this thesis has neither.
2. **A cost side.** Against 12 avoided defect classes stands the cost of a new language:
   training, tooling, hiring, the reserved-word collisions of §9.13.5, and the risk that the
   compiler itself is wrong. This experiment measures only the benefit column.
3. **A human trial.** Whether a compile-time rejection actually prevents the production
   incident that a runtime exception would also have prevented is an empirical question
   about developers, not about compilers, and it is unanswered here.

The honest verdict is therefore **argued, with the argument's own missing premises named** —
which is the position §6.10.1 now takes, and E14 is its evidence.

## Threats to validity

* **The PostgreSQL schema is mine.** A PostgreSQL expert might catch more; I have tried to
  use the strongest available tool for each job and to say where PostgreSQL's idiom is
  better than Nilestream's, but this is a single-author artifact and adversarial review by
  a PostgreSQL specialist is the obvious next step.
* **Thirteen defect classes are not a corpus.** They were chosen because the thesis claims to
  catch them, which biases toward Niles. A corpus drawn independently — from CVEs, from
  bank incident reports, from `git log` on an open-source ledger — would be much stronger,
  and its absence is the largest single weakness of this experiment.
* **Extensions were not tried.** `pg_ivm` provides incremental view maintenance, and
  TimescaleDB's continuous aggregates provide something close to partial materialization.
  Neither was tested, and either could shift Part 1's conclusion further toward PostgreSQL.
  Neither would touch Part 2, which is about static checking.
* **The Niles side is checked by the compiler this thesis wrote.** A compiler that agrees
  with its author's expectations is weak evidence. The thirteen programs are in the
  repository so that a reader can disagree with them.
