# 5. The counterexamples: what the measurements refused

This chapter is about the results that went against the thesis's first plan, and the
decisions they forced. Each result is presented as a **decision card**. A card states four
things:

- the **alternatives** that were on the table;
- the **criterion**, fixed before the numbers existed;
- the **evidence**, with the results file that holds it;
- the **measurement that would reverse it**.

A card is not a summary of the results file. The file is the evidence. The card says what the
evidence was asked, and what answer would have changed the decision.

The cards are in the order the measurements were taken. The last two are the product
decisions taken on their evidence: DB-1 (which engine) and DB-11 (how to extend it).
Section 5.9 regenerates every report cited here from its committed rows. A reader can check
that what a card quotes is what the rows say.

## 5.1 E14: can a REV be built without a new engine or a new language?

**Alternatives.**
- (a) The reconstructible epoch-anchored view (REV) needs Nilestream and Niles.
- (b) The REV mechanism runs in stock PostgreSQL 16, and only the static checks need a new
  language.
- (c) Neither needs anything new.

**Criterion.** Part 1: whether every property of the mechanism holds in stock PostgreSQL:
- partial materialisation;
- honest absence;
- anchored reconstruction;
- checkpoints;
- delta-proportional maintenance.

Part 2: thirteen defect classes, each written in Niles and against a good-faith PostgreSQL
schema, scored by the stage at which each is caught: compile time, runtime, or never.

**Evidence** (`results/E14-minimal-counterproposal.md`).
- The mechanism runs in PostgreSQL, and reconstruction equivalence holds exactly. Anyone can
  have a REV today without adopting anything from this thesis.
- Part 2 first favoured the language: against SQL alone, seven of ten paired classes were
  undetectable at any stage.
- Cycle 14 revised that half (R2-04, R2-05). With the properties written as comments and a
  checker reading them (SQL+C+L, `crates/nilescheck-sql`), SQL reaches every class.
- **E14 contains no Nilestream measurement.** It cannot be cited about the engine's
  performance.

**What would reverse it.** A REV property that PostgreSQL cannot hold, shown by a failing
reconstruction-equivalence check.

## 5.2 E27: Nilestream against its alternatives

**Alternatives.** Measured per size range, and decided by the engine rule:
- N (Nilestream);
- P+ (PostgreSQL with E14's mechanism, repaired);
- the hybrid H3 (PostgreSQL ledger plus a REV sidecar);
- P and M (full materialisation in PostgreSQL);
- H1 (ReadySet), H2 (pg_ivm) and T (TigerBeetle).

**Criterion** (cycle-14 round-2 work order, §7). N is justified as a product iff it beats P+
**and** H3, by the joint gate and above the floor, on memory per key read **and** on p99 read
latency under eviction.

**Evidence** (`results/E27-comparator.md`, `results/E27-p99-engine.md`).
- N was justified at no size from 10³ to 10⁵.
- N won the tails of head reads.
- P+ won memory almost everywhere.
- The P+ branch of the rule fired at single-currency 10⁵ only.
- The cycle-15 audit then found that E27 had not measured the engine as designed:
  - multi-currency reads never used the view;
  - the memory clause set a process that holds its base in memory against a database whose
    base is on disk;
  - q3 and q4 could not run on N.
- So E27 stands as a result about the engine as it was, and E27b was designed to measure it
  fairly.

**What would reverse it.** E27b, below.

## 5.3 E27b: the fair re-run

**Alternatives.** Those of E27, with H1, H2 and T not re-run (`docs/study/E27b-design.md` §3).

**Criterion.** E27's rule, unchanged. Memory is now **view-state bytes per key read**,
counted per arm. It was fixed in the design commit `77997dd` before any engine change.

**Evidence** (`results/E27b-comparator.md` and `results/E27b-p99-engine.md`, cycle 16).
- **Memory:**
  - H3 beats N at every size where the clause was evaluated (single 10³–10⁵, multi 10³ and
    10⁴). So N is not justified at any of them, whatever p99 shows.
  - Against P+, N is better at 10³ and 10⁴ and no different at single 10⁵.
- **Why N loses:** its figure is mostly the ledger's per-key checkpoint state. At single 10⁵
  the median checkpoint state is 15,901,376 B, more than the whole balance table of full
  materialisation (11,624,448 B).
- **Multi-currency 10⁵ memory is refused.** The memory cgroup killed N at every seed there.
- **p99** (the targeted re-run, on E27b's build, by the author's decision of 2026-10-10). The
  result is a trade, as in E27:
  - N's head reads have the shorter tails in the single-currency series at 10³ and 10⁴.
  - P+'s and H3's anchored multi-currency reads have the shorter tails.
  - Most 10⁵ cells are refused by the warm-up MAD rule.
- **The rule's verdict:** no clause holds at any series and size. H-E1 is refuted in the
  measured range.
- **The first attempt stopped on a wrong answer from N** (F-02-3: an empty scalar aggregate
  answered no row). It was fixed before the re-run, and its points are kept apart in
  `results/E27b-comparator/attempt-1/`.

**What it changed.** The theory, not just the engine. A per-key running aggregate is full
state for that aggregate, so with checkpointing on, partial state is never smaller than full
state in order. Card C16-03 restated the cost model (thesis §3.14, Theorem 3.7(iii),
Proposition 3.7.2).

**What would reverse it.** Under DB-3, E27c (`docs/study/E27c-design.md`, future work) would
measure an engine whose checkpoints are lazy, windowed or off the heap. That N′ beating P+
and H3 on both clauses at some size would reopen DB-1 for that size range.

## 5.4 E30: which surface for which task

**Alternatives.** Five surfaces:
- SQL (PostgreSQL with its checker);
- NL (Niles);
- RS (Rust);
- PRQL;
- Datalog (DL).

**Criterion** (pre-registered). Per task class, the principal surface is the one that, in
order: (i) expresses the most tasks; (ii) has the highest static-rejection rate under
mutation; (iii) has the lowest check cost. New syntax is justified only for a class no surface
expresses.

**Evidence** (`results/E30-syntax.md`).
- Over all thirty tasks the principal surface is **SQL, by (i)**: it expresses 30 of 30;
  Niles 27, of which 20 are verified.
- NL and RS tie on transactions and view contracts.
- New syntax is not justified: every class has a surface that expresses all of its tasks.

**What would reverse it.** A task class that SQL cannot express and Niles can.

## 5.5 E30b′: the adversarial corpus

**Alternatives.** The Niles grammar is kept for a defect class, or retired for it.

**Criterion** (decision DA-1). The grammar is kept for exactly the classes that the upgraded
SQL+C+L cannot refuse at check time on a corpus written before the upgrade.

**Evidence** (`results/E30b-syntax.md`).
- One class of eight counts for the grammar: **schema-level omission**. A hold domain was
  never declared linear; a ledger was never declared a ledger.
- SQL+C+L's rules are keyed on annotations, so they fail open when an annotation is
  missing.
- In every other class the upgraded checker refuses what Niles refuses.

**What it changed.** E30c was designed against exactly that class.

## 5.6 E30c: the schema rule

**Alternatives.**
- Keep Niles's schema surface for the omission class.
- Declare what an object is by where it is created: event triggers at DDL time, plus a
  catalog pass in the gate (`docs/SPEC-DIALECT.md` §4).

**Criterion** (`docs/study/E30c-design.md` §1, pre-registered at `a038646`). If every
defective script of the fourteen omission cases is refused, at DDL time or at check time,
Niles's schema surface becomes specification only. A correct twin that is refused is reported
as a false refusal.

**Evidence** (`results/E30c-dialect.md`).
- The triggers refuse 11 of 14 defective scripts at DDL time.
- The catalog pass (`nilescheck-sql --dialect`) refuses all 14.
- Neither refuses a correct twin.
- The baseline checker and stock PostgreSQL accept every defective script.

**What would reverse it.** An omission case outside these fourteen that passes both
mechanisms. Fourteen cases written by the builder of the rule show that a class can slip
past a mechanism, not how often one does.

## 5.7 DB-1: the product engine is PostgreSQL

**Alternatives** (`docs/audit/cycle-16`, gbs).
- (a) PostgreSQL is the product engine, and annotated PostgreSQL SQL is the product language.
  Nilestream becomes the thesis's instrument.
- (b) Nilestream stays a product candidate and gains a PostgreSQL-dialect front end.
- (c) Two languages, stated openly.

**Criterion.** One language on one engine, as far as the evidence above points.

**Evidence.**
- E27 justified N at no size; so did E27b, by its memory clause.
- H-L1 and H-L2: checked SQL refuses what Niles refuses, except schema-level omission, which
  E30c closes.
- The checked language runs on PostgreSQL, not on Nilestream.

**Taken:** (a).

**What would reverse it** (written when it was taken). N, with its memory model restated
(DB-3), beating P+ **and** H3 on both clauses at 10⁵ or above. That would reopen (b) for that
size range.

## 5.8 DB-11: PostgreSQL is extended in SQL and PL/pgSQL first

**Alternatives.**
- SQL and PL/pgSQL with the existing static checker, and no new dependency.
- A compiled extension for the REV or the checks, now.
- A fork of PostgreSQL.

**Criterion.** No compiled extension and no new dependency without a measurement showing SQL
is the bottleneck, and then a new decision.

**Evidence.**
- E14 showed the REV mechanism runs in PL/pgSQL with the right asymptotics.
- GBS's ledger runs on it: G3p, 29 product lines, 0 failures (`results/G3p-verdict.md` in
  gbs).
- No measurement yet shows SQL to be the bottleneck.

**What would reverse it.** A measurement in which the PL/pgSQL mechanism is the bottleneck, at
a size the product needs.

## 5.9 Regenerating what these cards quote

Every report cited above, except E14's (which needs its own run, `crates/counterproposal/
run.sh`), is rendered from rows committed beside it. Rendering them again must change nothing.

```sh
cargo run -q --release -p comparator -- render > /dev/null 2>&1
cargo run -q --release -p comparator -- render --study e27b > /dev/null 2>&1
cargo run -q --release -p comparator -- render --study e27b --shape p99-engine > /dev/null 2>&1
cargo run -q -p syntax-study --bin e30 -- report > /dev/null
cargo run -q -p syntax-study --bin e30b -- report > /dev/null
cargo run -q -p syntax-study --bin e30c -- report > /dev/null
git diff --exit-code -- results/ > /dev/null && echo "every report is what its rows say"
```

```expected
every report is what its rows say
```

## 5.10 What this chapter does not cover

- The measurements that confirmed the plan (E8, E10, E18 and the rest). They are in the
  thesis's Chapter 9.
- GBS's decisions other than its backend.
