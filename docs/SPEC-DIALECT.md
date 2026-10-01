# The dialect: PostgreSQL's SQL and PL/pgSQL, checked

**Status: normative for the checker as built; §5's verdict is E30c's result**
(cycle 16, C16-10). The product language of this project is PostgreSQL 16's SQL and PL/pgSQL
with the conventions below (decision DB-8). Niles — the full grammar, type system and
consistency-effect calculus — is their specification and the reference checker
`nilescheck-sql` is tested against; it does not replace SQL.

Every convention is stock PostgreSQL: a comment, a table row, or where an object is created.
PostgreSQL accepts every checked script unchanged, and a script that ignores the conventions is
still SQL. The checker is `crates/nilescheck-sql`:

```text
nilescheck-sql [--catalog-only | --dialect] FILE…
```

`--catalog-only` runs the catalog rules alone (E14's "PostgreSQL + checker" column); with no flag,
SQL+C+L (the catalog rules, linearity, conservation, capabilities, declared effects and body
typing); `--dialect` adds the schema rule of §4.

## 1. Money

A currency is a composite type `(minor bigint)` whose comment declares its scale:

```sql
create type currency.usd as (minor bigint);
comment on type currency.usd is 'currency scale=2';
```

PostgreSQL then refuses `usd + eur` in a view at `CREATE` time (there is no such operator), and
the checker refuses it where PostgreSQL does not look until run time: inside PL/pgSQL bodies, in
arguments, in assignments, in inserts. A literal with more fractional digits than the scale is
refused rather than rounded into the integer.

## 2. The annotations

| annotation | on | means | checked by |
|---|---|---|---|
| `currency scale=N` | a composite type | a currency with N fractional digits | typing (NL0250, NL0255, NL0332) |
| `ledger` | a table | append-only, conserving per (transaction, currency); its conservation is a deferred constraint trigger | catalog rules, conservation through `niles-lang`'s solver |
| `anchor` | an index | the index anchored reads use | catalog rules |
| `confidential:e2ee` | a column | an end-to-end-encrypted column, readable only where the label flows | catalog rules |
| a row of `serve_contract(view_name, consistency, materialize, retain, budget, lineage)` | a view | the view's serve contract: its rung, its materialisation, its retention, its lineage mode | catalog rules (NL0311 and its family) |
| `linear <kind>` | a domain | every value is consumed exactly once on every path that commits | linearity (NL0320) |
| `produces <kind>` / `consumes <kind>` | a function | it makes, or consumes, a linear value | linearity |
| `capability <effect>` | a domain | an unforgeable capability | capabilities |
| `requires <effect>` / `grants <effect>` | a function | a caller must hold the capability; or the function may build one | capabilities |
| `effects: append; debits <cur>; credits <cur>; holds <cur>; authorizes <cur>; reads@<rung>` | a function | the function's declared effect row; its body's inferred effects must be permitted by it | effects (NL0310) |

**Dynamic SQL in a function that writes a ledger is an error** (NSQ002): the statement it runs
exists only at run time, and conservation, typing and effects cannot be checked on it. Elsewhere
it is a warning (NSQ001).

## 3. What the checker refuses, by code

The codes are `niles-lang`'s wherever the rule is the same rule, so the two checkers' outputs can
be compared line by line.

| code | refuses |
|---|---|
| NL0250 | arithmetic or a comparison between two currencies |
| NL0240 | a currency value built from a literal with more fractional digits than its scale |
| NL0255 | a value of one currency where another is required |
| NL0332 | an assignment, a default or a `return` of the wrong currency |
| NL0300 / NL0301 | a transaction that cannot balance per currency, or may not across a branch (conservation, through `niles-lang`'s solver) |
| NL0211 | conservation removed from a ledger |
| NL0230 | a ledger mutated: `update`, `delete`, `truncate`, `merge` or a rule |
| NL0310 | an inferred effect the declared row does not permit; also a materialisation of a definition that cannot be reproduced |
| NL0320 | a linear value not consumed exactly once |
| NL0312 / NL0330 | a capability required and not held, or built rather than received |
| NL0260 | a predicate on a confidential column |
| NL0311, NL0220–NL0223, NL0321, NL0322, NL0100, IR013 | the serve-contract family and its neighbours (`check.rs`) |
| NSQ001 / NSQ002 | dynamic SQL: a warning, or an error in a function that writes a ledger |
| NSQ003 | a warning: an item of an `effects:` annotation the checker cannot read |
| NSQ110–NSQ115 | the schema rule (§4), with `--dialect` |

## 4. The schema rule: declarations made in the DDL itself

E30b′ found one class the annotations above could not close: **schema-level omission**. A rule
keyed on a comment is turned off by omitting the comment, so a ledger never commented `ledger`
was not checked as one. A PostgreSQL event trigger can refuse DDL, but it fires before a later
`COMMENT ON` exists. So the dialect declares what an object is **by where it is created**
(`docs/study/E30c-design.md`, which gives the alternatives considered and why they were not
taken):

| reserved schema | holds | guaranteed |
|---|---|---|
| `currency` | the currency types | each must carry its scale comment (NSQ115) |
| `ledger` | ledger tables | money-bearing, with a `txn` column (NSQ111); conservation and append-only triggers installed by the dialect, never dropped or disabled (NSQ112) |
| `state` | tables of money not yet posted | keyed by a linear or capability handle (NSQ113) |
| `linear` | linear domains | the domain's name is its kind; no comment needed |
| `capability` | capability domains | the domain's name is its effect; no comment needed |

Money stored anywhere else — a table, a copy made with `LIKE`, `CREATE TABLE AS` or `SELECT
INTO`, a partition, a column added later, a table moved — is refused (NSQ110). A function role
naming a kind or effect that no domain declares is refused (NSQ114).

The rule is enforced twice. **At DDL time**, inside PostgreSQL, by the event triggers of
`crates/syntax-study/corpus/e30c/schema/dialect.sql`, installed by the cluster's superuser
(`tools/e30c-install.sh`). **At check time**, by `nilescheck-sql --dialect` in the gate, because a
superuser, `event_triggers = off` or single-user mode bypasses the triggers.

**What the rule does not claim.** An amount written as a bare `bigint`, outside every currency
type, is not money to it; E30c's case S05 measures what that costs. A superuser who disables the
triggers and runs no gate is outside it.

## 5. The verdict on the rule

From `results/E30c-dialect.md` (E30c, cycle 16, C16-10; pre-registered at `a038646`, run after
it). **On E30c's fourteen omission cases every defective script is refused**: eleven at DDL time by
the event triggers (clauses D1–D3), all fourteen at check time by `nilescheck-sql --dialect`
(NSQ110–NSQ115), and **no correct twin is refused** by either. The baseline checker (SQL+C+L)
and stock PostgreSQL accept every defective script; stock PostgreSQL's two refusals (S09, S10)
are only that the triggers it is asked to drop or disable do not exist there. The four cases the
triggers cannot see (S02–S04: a linear handle, a capability, a currency scale) are about
annotations a later statement supplies, which is why the catalog pass exists. By the design's
rule, Niles's schema surface becomes specification only, **on this corpus**: fourteen cases
written by the rule's builder show that a class can slip past a mechanism, not how often one does
in a bank's schema, and the triggers' cost on DDL is not measured.

## 6. What C4's soundness can and cannot claim over annotated PL/pgSQL

C4, the consistency-effect calculus and its soundness theorem, is **partly proved** for
λ_niles: clauses (1)–(3) hold for the calculus, the transfer to system traces is a lemma marked
as a sketch, and clause (4) is conditional on an obligation that is specified and not proved
(`thesis/status.toml`). Over the dialect it claims less, and the difference is stated rather than
left to be inferred:

* **No theorem transfers.** There is no translation from annotated PL/pgSQL into λ_niles, so no
  clause of C4 is a theorem about a PL/pgSQL program. What holds of the dialect is a property of
  the checker, tested on corpora — E14's thirteen defect classes and five spellings of a forgotten
  hold, E30's thirty tasks, E30b′'s fourteen adversarial cases, E30c's fourteen omission cases —
  not proved.
* **The checker fails open where it cannot infer.** Body typing leaves alone anything whose type
  it cannot infer; a currency that is not a literal is undecided; dynamic SQL outside a ledger
  writer is a warning. Each is a place where a program the calculus would refuse can pass.
* **What the checker does close, on those corpora:** cross-currency arithmetic and argument
  passing in bodies; declared effects against inferred ones; linearity of values whose domain is
  declared linear; capabilities; conservation of statically visible ledger writes through the
  same solver Niles uses; and, with `--dialect`, the omission class of §4.
* **What the runtime still enforces whatever the checker says:** the deferred conservation
  trigger and the append-only triggers on a ledger. A program the checker wrongly accepts cannot
  commit a non-conserving transaction or edit history; it can only be refused later, at commit,
  than it would have been at check time.

## 7. Not covered

Triggers written by the author on ledger tables are checked as functions, not as the effects of
the statements that fire them; back-valued corrections have no Niles form to compare against
(E30b′, A13–A14); and a console session typed by hand is checked by nothing.
