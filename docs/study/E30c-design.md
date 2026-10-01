# E30c — can the dialect refuse schema-level omission? Design, pre-registered

*Cycle 16, round 2, card C16-10. This document and the corpus in
`crates/syntax-study/corpus/e30c/` are committed together, before any part of the rule in §3 is
built: no event trigger, no catalog pass, no change to `nilescheck-sql`. A change to the rule, the
corpus or the counting after this commit is a deviation, listed in `results/E30c-dialect.md`
with its reason, and never made after seeing an outcome it would move. Nobody but the builder of
the checker has written or reviewed the corpus; that is a stated limit (§8).*

## 1. The question, and the rule that answers it

E30b′ (`results/E30b-syntax.md`, H-L2) left exactly one defect class for the Niles grammar:
**schema-level omission**. In A01 a hold domain was never declared linear; in A02 a ledger table
was never declared a ledger. SQL+C+L's rules are keyed on annotations made in `COMMENT ON`
statements, so a schema that omits the comment turns the rule off and the checker fails open.
Niles cannot omit them: a hold is linear and a `post` conserves, whatever the schema says.

The cycle-16 research (R-C §3) adds the constraint that shapes the design. A PostgreSQL event
trigger can refuse DDL (a failing `ddl_command_end` trigger rolls the statement back), but there
is no commit-time event, so a trigger at `CREATE TABLE` cannot see a `COMMENT ON` that follows
it. A declaration that is to be enforced when the DDL runs must therefore be **in the DDL
itself**, and what the trigger cannot see must be caught by a **catalog pass** before deploy.

**The rule, from the work order, fixed here:**

> If every defective program of the corpus in §4 is refused at DDL time (by the event trigger,
> inside PostgreSQL) or at check time (by `nilescheck-sql`'s catalog pass), Niles's schema
> surface becomes specification only, and the last class of R2-b closes. Otherwise each class
> with a defective program accepted by both is named as the residual, and the Niles grammar is
> kept for it.

**How a case is counted (fixed here).** Each defective program `d.sql` gets one outcome:
*refused at DDL* (PostgreSQL raised the dialect's error while running it), *refused at check*
(accepted by PostgreSQL, refused by the catalog pass with a code), or *accepted*. Each correct
twin `c.sql` must be *accepted* by both; a correct twin that is refused is a **false refusal**,
reported per mechanism. A class whose defective programs are refused only by also refusing
their correct twins is marked as such, as in E30b′.

## 2. The declarations, made in the DDL

The dialect declares what a table or domain *is* by **where it is created**. Placement is part of
the `CREATE` statement, so the event trigger sees it, and omitting it cannot leave a money-bearing
object in a place the rules do not cover: every place is covered, and the default place refuses.

| reserved schema | holds | what the dialect guarantees there |
|---|---|---|
| `currency` | composite types `(minor bigint)`, one per currency | a currency; its scale is declared by `comment on type … is 'currency scale=N'` (§3, rule K4) |
| `ledger` | append-only, conserving tables | conservation per `(txn, currency)` at commit, and no `UPDATE`, `DELETE` or `TRUNCATE`, installed by the dialect, not by the author |
| `state` | mutable tables that hold money which is not yet posted (holds, limits, pending items) | every row is reached through a linear or capability handle (§3, rule K2) |
| `linear` | domains whose values are linear: consumed exactly once | SQL+C+L's linearity, without a `linear <kind>` comment; the kind is the domain's name |
| `capability` | domains whose values are capabilities | SQL+C+L's capability rule, without a `capability <effect>` comment; the effect is the domain's name |

**Why placement, and not the alternatives** (the work order asks for the choice with reasons):

* **A reserved domain on a column** (for example `txn ledger_txn`) puts the declaration in the
  DDL too, but it marks one column of a table that may be copied, partitioned or altered column
  by column, and a table built with `LIKE` or `CREATE TABLE AS` carries the domain into places
  that are not ledgers. A schema is one fact per object and survives none of those silently.
* **`SECURITY LABEL`** is in the DDL's neighbourhood but needs a label provider written in C,
  which decision DB-11 rules out without a measurement showing SQL is the bottleneck.
* **A naming convention** is a heuristic, and R-C §3's second caveat applies: it moves the
  fail-open from "forgot to annotate" to "forgot the convention". Placement has the same
  weakness for *money-bearing*, which is why §3 defines money-bearing by type and not by name.

What is still declared by comment, and why it is enough: a currency's **scale** and a function's
**`produces` / `consumes` / `requires` / `grants`** roles. Both are checked by the catalog pass,
which runs over the whole catalog, after every comment exists.

## 3. The rule's clauses

**Money-bearing.** A column is money-bearing when its type is a composite type in schema
`currency`, an array of one, or a domain over one. A table is money-bearing when it has a
money-bearing column. Nothing else is money-bearing: a `bigint` named `amount` is not, and a
case in §4 holds what that costs (S05).

**Refused at DDL** (the event trigger, `ddl_command_end`, raising and so rolling back):

* **D1.** A money-bearing table, or a partition of one, created or moved into any schema other than
  `ledger` or `state`. This covers `CREATE TABLE`, `CREATE TABLE … (LIKE …)`, `CREATE TABLE AS`,
  `SELECT INTO`, `CREATE TABLE … PARTITION OF`, `ALTER TABLE … ADD COLUMN` of a money-bearing
  type, and `ALTER TABLE … SET SCHEMA`.
* **D2.** A table in `ledger` that is not money-bearing, or that has no column named `txn`.
* **D3.** A ledger table left without the dialect's conservation and append-only triggers, or
  with either disabled or dropped: `DROP TRIGGER`, `ALTER TABLE … DISABLE TRIGGER` on a ledger
  table are refused. The triggers are installed by the dialect when the table is created, named
  `dialect_conserve` and `dialect_append_only`; the author writes neither.

**Refused at check** (the catalog pass in `nilescheck-sql`, run over the schema and the program
together, in the gate):

* **K1.** Everything D1–D3 refuse, again, from the script: the event trigger can be bypassed by a
  superuser, by `event_triggers = off`, or in single-user mode (R-C §3), and the gate must not
  depend on the deploy having had it on.
* **K2.** A money-bearing table in `state` whose primary key is not of a domain in `linear` or
  `capability`. A row of unposted money is a resource, and a resource is handed out linearly.
* **K3.** A function commented `produces <kind>` or `consumes <kind>` for which no domain
  `linear.<kind>` exists, or `requires <effect>` / `grants <effect>` for which no domain
  `capability.<effect>` exists. A role naming a kind nobody declared is an omission spelled as a
  typo.
* **K4.** A composite type in `currency` with no `currency scale=N` comment.

**What the dialect does not claim.** A money amount written as a bare `bigint`, outside every
currency type, is not money to these rules (S05 measures it rather than assuming it away). A
superuser who disables the event trigger and runs no gate is outside the rule; the gate is what
K1 is for. PL/pgSQL bodies are checked by SQL+C+L's existing rules, unchanged by this card.

## 4. The corpus: fourteen cases, written before the rule

Each case is a directory `Snn/` with `d.sql` (the defective script) and `c.sql` (its correct
twin), run in one transaction after `schema/base.sql`, which creates the reserved schemas, three
currency types with their scales, and E30's non-money tables. `cases.tsv` gives each case's class
and the clause expected to refuse it. **The expectation column is a prediction, not a result**:
a case whose expectation turns out wrong is reported as wrong, not edited.

| case | omission | defective script does | expected to refuse |
|---|---|---|---|
| S01 | ledger not declared (A02, restated) | creates `public.postings` with currency columns | D1 |
| S02 | linear handle not declared (A01, restated) | a `state.holds` keyed by a plain `bigint`, its domain in `public` | K2 |
| S03 | capability not declared | a function `requires overdraft` with no `capability.overdraft` | K3 |
| S04 | currency scale not declared | `currency.gbp` with no scale comment | K4 |
| S05 | money as a bare integer | `ledger.postings` whose only amount column is `bigint` | D2 (not money-bearing) |
| S06 | ledger copied with `LIKE` | `public.p2 (like ledger.postings including all)` | D1 |
| S07 | ledger copied with `CREATE TABLE AS` | `public.snapshot as select * from ledger.postings` | D1 |
| S08 | ledger copied with `SELECT INTO` | `select * into public.copy from ledger.postings` | D1 |
| S09 | conservation removed | `drop trigger` on the ledger's conservation trigger | D3 |
| S10 | append-only switched off | `alter table … disable trigger` on the ledger's append-only trigger | D3 |
| S11 | money added to an undeclared table | `alter table public.accounts add column limit_amt currency.usd` | D1 |
| S12 | ledger moved out of its schema | `alter table ledger.postings set schema public` | D1 |
| S13 | a partition outside the ledger | `create table public.postings_2026 partition of ledger.postings_p …` | D1 |
| S14 | money state outside `state` | `public.holds` with a currency column | D1 |

Every correct twin does the same thing in the declared place: `ledger.postings`,
`linear.hold_ref`, `capability.overdraft`, a scale comment, `currency.usd` for the amount, a copy
into `state` keyed by a linear handle, and so on. S09 and S10's correct twins create and drop an
unrelated trigger on a `state` table, so a rule that refused every `DROP TRIGGER` would be caught
as a false refusal.

**The baseline, measured beside it.** The same fourteen pairs are also run against today's
SQL+C+L (`nilescheck-sql` at this commit, before the catalog pass) and against stock PostgreSQL
with no event trigger. The prediction is that both accept every defective script, because
neither has a rule keyed on placement. That is the gap the card closes, measured rather than
asserted.

## 5. Building it

1. `schema/dialect.sql` in the corpus: the reserved schemas, the event-trigger function and the
   event trigger, and the conservation and append-only trigger functions the dialect installs.
   The event trigger is created by the cluster's superuser (event triggers are superuser-only,
   R-C §3); the scripts under test run as the provisioned non-superuser role.
2. The catalog pass in `nilescheck-sql`: rules K1–K4 over the parsed script, with new `NSQ`
   codes, each with a test that fails when the rule is removed.
3. `crates/syntax-study`: a runner for the corpus, writing `results/E30c-dialect.md` and its
   manifest row in the same commit.

**What may change in building without being a deviation:** the mechanism by which D3's triggers
are installed (the event trigger issuing `CREATE TRIGGER`, or refusing a ledger table until a
dialect function has installed them), provided D1–D3 refuse exactly what §3 says. **What is a
deviation:** any change to §3's clauses, to the corpus, or to the counting in §1.

## 6. Output

`results/E30c-dialect.md`: one row per case and mechanism (event trigger, catalog pass, baseline
SQL+C+L, stock PostgreSQL), the outcome and the code or error; the false refusals; the verdict
under §1's rule; and, if any class is residual, its name and what Niles does with it. The design's
predictions in §4 are printed beside the outcomes.

## 7. What follows from each outcome

* **Every defective script refused, no false refusal.** The schema surface of Niles becomes
  specification only; `docs/SPEC-DIALECT.md` states the dialect, and H-L2's instrument text
  records that the last class closed, on this corpus.
* **A residual class.** It is named in H-L2 and in `docs/SPEC-DIALECT.md` as what the dialect
  cannot refuse, with the Niles construct that does.
* **A false refusal.** Reported as a cost of the rule, as E30b′ reported A03's.

## 8. Not measured, and said so

* **Who wrote the corpus.** The builder of the checker, who also wrote the rule. Fourteen cases
  show that a class *can* slip past a mechanism, not how often one does in a bank's schema.
* **Bypass.** A superuser can turn the event trigger off; the gate (K1) is the answer, and the
  gate is only as good as the discipline that runs it.
* **Runtime cost** of the event trigger on DDL. DDL is rare; it is not measured here.
* **C4's soundness over annotated PL/pgSQL.** That is `docs/SPEC-DIALECT.md`'s statement to make,
  after this study, and this design does not anticipate it.
