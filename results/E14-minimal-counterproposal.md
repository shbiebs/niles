# E14 — The Minimal Counterproposal

*Can reconstructible epoch-anchored views be built with current Rust and SQL? And if they
can, what is left for a new language and a new engine to do?*

Reproduce with `./crates/counterproposal/run.sh` against PostgreSQL 16.13 and a built
`nilesc`. The schema is `crates/counterproposal/sql/schema.sql`; the Niles corpus is
`crates/counterproposal/niles/`.

## Method

A differential defect corpus. Twelve defect classes, each written twice — once against a
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

`compile` = rejected before the program can run. `runtime` = raises when executed.
`never` = accepted and executed, with the stated consequence.

| # | Defect | PostgreSQL | Niles | Niles code |
|---|---|---|---|---|
| D1 | Transaction that does not balance | **runtime** | **compile** | `NL0300` |
| D2 | Adding USD to EUR in a query | **never** | **compile** | `NL0250` |
| D3 | `100.50 jpy` — half a yen, where JPY has scale 0 | **never** — persisted | **compile** | `NL0240` |
| D4 | Mixed-currency transaction: 100 USD → 100 EUR | **runtime** — rolled back | **compile** | `NL0300` |
| D5 | Strict view derived from a bounded-stale view | **never** | **compile** | `NL0311` |
| D6 | View predicate reading the wall clock | **never** | **compile** | `IR013` |
| D7 | Materializing that view | **never** | **compile** | `IR013` |
| D8 | Filtering on a column that should be encrypted | **never** | **compile** | `NL0260` |
| D9 | Overdraft with no authorization | **never** — persisted | **compile** | `NL0312` |
| D10 | `drop trigger` removes conservation | **never** — money destroyed | **not expressible** | — |
| D11 | `update` against the ledger | **runtime** | **compile** | `NL0230` |
| D12 | `ledger_consistent` + `spilled` contract | **not expressible** | **compile** | `NL0220`, `IR010` |
| D13 | Missing anchor index on a demand view | **never** (silent slowdown) | **compile warning** | `NL0223` |

**Totals.** PostgreSQL: 3 caught at runtime, 0 at compile time, 9 never. Niles: 11 at
compile time, 1 as a compile-time warning, 1 with no spelling in the language.

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

**Q: Do we need an SQL replacement, or only a MySQL/PostgreSQL replacement with REVs?**
Neither framing survives. The evidence says the *opposite* of the thesis's own emphasis:

* The **engine** case is the weaker one. The mechanism runs in PostgreSQL with the right
  asymptotics. What a custom engine buys is that the REV is a first-class object the
  database maintains and verifies, rather than application code each team rewrites — a real
  benefit, but an engineering one, not a capability one.
* The **language** case is the stronger one. Nine of twelve defect classes are undetectable
  in SQL *at any stage*, because SQL has nothing to state the property against: no
  per-currency scale in the type, no per-view rung, no linearity, no effect row, no
  reproducibility obligation on a predicate. These are not gaps PostgreSQL could close with
  more triggers; a trigger is a runtime check, and D10 shows what a runtime check is worth.

**Q: What would it take to prove Niles should exist?**
This table is the shape of the argument but not yet a proof, and three things are missing.

1. **A frequency premise.** The table shows these defects are *undetectable*, not that they
   are *common*. A defect class nobody writes costs nothing to miss. Establishing frequency
   needs a corpus of real banking code or an incident study, and this thesis has neither.
2. **A cost side.** Against 9 avoided defect classes stands the cost of a new language:
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
* **Twelve defect classes are not a corpus.** They were chosen because the thesis claims to
  catch them, which biases toward Niles. A corpus drawn independently — from CVEs, from
  bank incident reports, from `git log` on an open-source ledger — would be much stronger,
  and its absence is the largest single weakness of this experiment.
* **Extensions were not tried.** `pg_ivm` provides incremental view maintenance, and
  TimescaleDB's continuous aggregates provide something close to partial materialization.
  Neither was tested, and either could shift Part 1's conclusion further toward PostgreSQL.
  Neither would touch Part 2, which is about static checking.
* **The Niles side is checked by the compiler this thesis wrote.** A compiler that agrees
  with its author's expectations is weak evidence. The twelve programs are in the
  repository so that a reader can disagree with them.
