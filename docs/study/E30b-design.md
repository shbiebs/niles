# E30b′ — does the language need its own grammar? Design, pre-registered

*Cycle 15, round 2, card C15-05 (decision DA-1). This document is committed on its own, before
any program of the study is written and before any checker or executor is changed. It fixes the
corpora, the SQL money representations, the mutation operators and site patterns, the classes,
the three SQL+C+L additions and the two niles-interp additions, and the rule. A change to any of
them after this commit is a deviation, listed in `results/E30b-syntax.md` with its reason, and
never made after seeing a number it would move.*

## 1. The question and the rule

E30 (`docs/study/E30-syntax-design.md`, `results/E30-syntax.md`) asked which surface should be
principal. The cycle-15 audit (DA-1) found that most of Niles's advantage on transactions was a
single check (29 of its 30 static currency-swap refusals are NL0310, the declared-effect-row
check), that E30 under-counted SQL twice (43 swaps refused by PostgreSQL's analyser before any row
ran; 55 silent swaps that came from the one-column-per-currency representation), and that
SQL+C+L refused 5 of 6 adversarial spellings of a hold defect. So this study asks a narrower
question: **which defects need the Niles grammar to be refused at check time?**

The rule, verbatim from DA-1:

> The Niles grammar is kept for exactly the defect classes that the upgraded SQL+C+L cannot
> refuse at check time on the adversarial corpus. If it refuses all of them, annotated SQL is
> the language, and Niles is its specification and reference checker. Check time per KLOC above
> 2× is reported as a cost, not a veto.

**How a class is counted (fixed here).** For each defective adversarial program, each checker
gets one of three outcomes: *refused* (at check time, with its codes), *accepted*, or *not
expressible* (the language has no way to write the defect). A defect class **counts for the Niles
grammar** when SQL+C+L accepts at least one defective program of the class, and for that same
case Niles either refuses it or cannot express it **while it can express the case's correct
twin**. A case where Niles can express neither twin is **outside Niles** and counts for neither
language; it is reported as a Niles limit. A correct twin that a checker refuses is a **false
refusal**. False refusals are reported per checker. They do not change the rule, but a class
that SQL+C+L "refuses" only by refusing the correct twin as well is marked as such.

## 2. The checkers and executors under test

| id | surface | checker | executor |
|---|---|---|---|
| **SQL-R1** | PostgreSQL 16 SQL and PL/pgSQL, E30's schema and representation (§3), E30's programs plus effect annotations (§5.1) | `nilescheck_sql::check_all`, upgraded by §5 | PostgreSQL 16, each program in `begin … rollback`, as E30 |
| **SQL-R2** | the same, over the single-amount-column representation (§3) | as SQL-R1 | as SQL-R1 |
| **NL** | Niles's SQL-shaped query surface, E30's programs unchanged | `niles-lang` at `c15/01-repairs` (`f51fa41`), which includes NL0257; **not changed by this card** | `niles_ir::eval` for queries, `niles-interp` for transactions, upgraded by §6 |
| **RS** | Niles's pipeline surface, E30's programs unchanged | as NL | as NL |

PRQL and Datalog are not re-run. Their E30 rows stand, and they are outside this question: neither
has a checker for transactions.

**The Niles grammar does not grow** (forbidden by the work order). The Niles checker is the one at
`f51fa41`. The only Niles changes are the two executor additions of §6, which change what can be
*run*, never what is *accepted*.

## 3. The SQL money representation: both, reported side by side

The audit's finding is that E30's representation manufactured silent currency swaps: typed money
as one column per currency (`amt_usd usd, amt_eur eur, amt_jpy jpy`) next to a `cur text`. A read
that swaps `amt_usd` for `amt_eur` is a legal query that answers nulls. Niles has one `amt` column,
so it offers far fewer such sites (38 against 124). Neither representation is obviously the one a
bank would choose, so **both are reported**. Neither is chosen after the numbers.

* **R1 — E30's representation, unchanged:** `cur text`, and one typed amount column per currency.
  The E30 SQL programs are reused as they are, plus their effect annotation (§5.1).
* **R2 — one amount column:** `postings(txn, acct, cur text, amt bigint, epoch, value_date)`,
  with the amount in minor units. Currency types stay in function signatures (`m usd`) and in
  composites built in bodies. The ledger comment names its columns: `comment on table postings is
  'ledger cur=cur amount=amt'`, and `holds` likewise has `cur text, amount bigint`. The
  conservation trigger groups by `(txn, cur)` over `amt`. Each E30 SQL program is rewritten for
  R2 by the smallest edit that preserves its meaning (`amt_usd` → `amt` with `cur = 'usd'`, and
  `row(-(m).minor)::usd` → `-(m).minor`), and is checked against the oracle like any E30
  program.

**What R2 needs from the checker that is not one of the three additions:** conservation must be
able to read a leg whose currency is a `cur` value and whose amount is a plain integer column.
`conserve.rs` gains exactly that. When the ledger comment names `cur=` and `amount=`, a leg's
currency is the row's `cur` string literal, or undecided when that is not a literal, and its
amount is the `amount` expression, read as the solver reads minor units today. This adapts the
same judgement to a second shape. It adds no rule, and it is listed as a representation adapter,
not as an addition.

## 4. The corpora

### 4.1 E30's 30 tasks

The same tasks, dataset (seed 1), oracle and answer canonicalisation as E30 (`crates/syntax-study`).
Programs:

* SQL-R1: E30's programs, and for every function-defining task (T01–T10) the effect annotation of
  §5.1 appended.
* SQL-R2: the R2 rewrite of every E30 SQL program, written after this commit and before any
  checker change. Each is compared with the oracle before the mutation stage, and a disagreement
  is a builder error, fixed then and counted (E30 §5).
* NL, RS: E30's programs, unchanged. Three consequences of §6 are stated now:
  - T02 and T06 become executable (money arithmetic). If their answers disagree with the oracle,
    that is a builder error, and it is fixed before mutation and listed.
  - T04 NL/RS become executable, but under the author's decision of 2026-09-30 (§6.2) a capture
    posts nothing. So T04's Niles answer cannot equal the oracle's transfer from A to B. It is
    recorded **not expressible as the task states: a capture names no payee**.
  - T05 NL/RS become executable. A void posts nothing, and the oracle expects nothing.

  T08 and T10 (relational reads inside a function) and T09 (a capability argument) stay
  **unexecuted**, as in E30.

### 4.2 The adversarial corpus

Written after this commit and **committed before any checker or executor change**, so that no
addition of §5 is tuned to it. Each case is a **defective** program and its **correct twin**, in
SQL (R1, and R2 where the case is about representation) and in Niles where Niles can express it.
The cases are fixed here by what they test. That makes 14 cases and at least 28 SQL programs, so
the corpus meets the work order's floor of 20:

| case | class (work order) | defective | correct twin |
|---|---|---|---|
| A01 | schema-level omission | a hold is placed and never resolved, under a schema variant whose `hold_ref` domain carries no `linear` comment | the hold is resolved |
| A02 | schema-level omission | an unbalanced transfer into `postings`, under a schema variant whose `postings` carries no `ledger` comment | a balanced transfer, same variant |
| A03 | dynamic `EXECUTE` | an unbalanced pair inserted through `execute format('insert into postings …')` | the balanced pair, through the same `execute` |
| A04 | dynamic `EXECUTE` | as A03, with the table name assembled by concatenation (`'post' \|\| 'ings'`) | the balanced pair, same spelling |
| A05 | functions calling functions | a helper inserts one leg; the task calls it for the debit only | the task calls it for both legs |
| A06 | functions calling functions | a two-level chain whose inner function posts in eur, under an outer function declared usd only | the chain posts in usd |
| A07 | triggers | an `after insert` trigger on the ledger inserts an unbalanced fee leg | the trigger inserts a balanced fee pair |
| A08 | triggers | a trigger function that `update`s the ledger | a trigger that appends a compensating pair |
| A09 | declared-effect mismatch | declared `debits usd; credits usd`, and the body posts a balanced pair in eur | the body posts in usd |
| A10 | declared-effect mismatch | declared `reads@snapshot`, and the body reads a view served `bounded` | the view is served `snapshot` |
| A11 | currency swap, typed position | a balanced pair in eur built from the minor units of a usd parameter (`row(-(m).minor)::eur`, `m usd`) | built from an eur parameter |
| A12 | currency swap, untyped position | R1: a usd amount in `amt_usd` labelled `cur = 'eur'`; R2: `cur = 'eur'` with `-(m).minor`, `m usd` | labelled `'usd'` |
| A13 | back-valued correction | a correction written as `update postings set value_date = …` | compensating postings with the earlier value date |
| A14 | back-valued correction | a correction whose two legs carry different value dates | both legs carry the corrected value date |

The Niles twins are the natural Niles spelling of the same defect. Where Niles has no spelling
for the defect (A03, A04, A07, A08, A12, and A13/A14 because Niles has no value-date form, E30's
F9), the case is recorded *not expressible* for Niles, with the reason. Schema variants (A01, A02)
are separate schema files beside the corpus, each a copy of the E30 schema with the one comment
removed, and the diff is shown in the results.

**Execution of the adversarial corpus.** The rule is about check time, so every program is
checked by every checker that applies. In addition, every SQL correct twin is run on PostgreSQL
(in `begin … rollback`, with deferred constraints forced), to show that it runs and conserves. A
correct twin that fails is a builder error, fixed before any checker verdict is read. Niles
correct twins are run by `niles-interp` where it can execute them, and are otherwise marked
unexecuted.

## 5. The three SQL+C+L additions, specified before they are built

Each gets guard tests: a program it must refuse and a neighbour it must accept. Each also gets a
sabotage run showing that the refusal disappears when the rule is disabled.

### 5.1 Effect annotations (the SQL spelling of NL0310)

* **Spelling:** `comment on function f(…) is 'effects: <e>; <e>; …'`. Each `<e>` is one of:
  - `append`;
  - `debits <cur>` or `credits <cur>`;
  - `holds <cur>`;
  - `authorizes <cur>`;
  - `reads@<rung>`.

  `<cur>` may be `*`. A function without an `effects:` comment is not checked. That matches Niles,
  where an omitted row means "infer and report".
* **Inferred effects of a body:**
  - An insert into a ledger is `append`. Each leg is `debits C` if its amount is syntactically
    negative (a unary minus at its head, inside a `row(…)` or not, or a negative literal),
    `credits C` if it is positive, and **both** if its sign is not syntactic.
  - `C` is the leg's currency: R1 by the typed column filled, R2 by the `cur` literal. It is `*`
    when neither is known.
  - A call to a function commented `produces` is `holds C`, where C is the currency of its
    money-typed argument.
  - A call to a function commented `requires` is `authorizes C`.
  - A call to a function carrying `effects:` contributes its declared row. A call to one without
    it contributes that function's inferred effects, transitively; a cycle contributes nothing
    further and is reported as a note.
  - A `select` from a relation that has a `serve_contract` row in the script is `reads@<its
    rung>`. A `select` from a ledger is `reads@ledger_consistent`.
* **The check:** every inferred effect must be permitted by the declared row. The permission rule
  is Niles's (`effects.rs`, `declared_permits`): a currency is matched exactly or by `*`, and
  reads are matched exactly. Refused as **NL0310**, with Niles's meaning.
* **The corpus's annotations** are written for the SQL-R1 and SQL-R2 T programs after this commit,
  before any checker change. Each states the effects its SQL body has, which is the Niles row of
  the same task wherever the bodies do the same thing. **An SQL body that reads the ledger to
  allocate a transaction id declares `reads@ledger_consistent`, because it does read it.**

### 5.2 Typing of PL/pgSQL and `language sql` bodies

* **Environment:**
  - parameters and declared variables, with their types;
  - table columns;
  - user functions' parameter and return types;
  - the currency composites.
* **Types inferred:**
  - a column, parameter or variable has its declared type;
  - `row(…)::C` and `x::C` are `C`;
  - `(v).minor` of a `C` value is **`minor⟨C⟩`**, an integer carrying the currency it came from;
  - `+`, `-` and unary `-` on `minor⟨C⟩` stay `minor⟨C⟩`;
  - an integer literal is `minor⟨?⟩`, compatible with any currency;
  - `sum/min/max/coalesce/least/greatest` of `T` are `T`;
  - a user function's call is its declared return type.
* **Refused at check time:**
  - **NL0250**: `+`, `-` or a comparison between two currencies, including `minor⟨C⟩` against
    `minor⟨D⟩`. This is the existing rule, now over body variables as well as columns.
  - **NL0255**: a value of currency C where currency D is required. The positions are:
    - a user function's argument;
    - an insert into a column of type D;
    - `row(e)::D` with `e: minor⟨C⟩`;
    - a ledger row whose `cur` string literal names D while its money is C. In R1, the money is
      the typed column filled, or its value. In R2, it is the `amount` expression's `minor⟨C⟩`.
  - **NL0332**: an assignment, a `declare … :=` default, or a `return` whose value is C where the
    declared type is D.
* Anything whose type the checker cannot infer is left alone. No rule guesses.

### 5.3 Dynamic `EXECUTE` in a ledger-writing function is an error

* **A unit writes a ledger** when any of these holds:
  - its static statements insert into, update, delete from or merge into a ledger;
  - it calls, directly or transitively, a function that writes a ledger;
  - it is the function of a trigger on a ledger;
  - **or** a string literal anywhere in it contains a ledger's name as a whole word.
* **The rule:**
  - An `execute`, `return query execute` or `for … in execute` in a unit that writes a ledger is
    **NSQ002, an error**: "dynamic SQL in a function that writes a ledger: the statement it runs
    exists only at run time, and conservation, typing and effects cannot be checked on it".
  - Elsewhere it stays **NSQ001, a warning**, as today.
* The string-literal clause is written here, before A04 exists. A04, whose ledger name is
  assembled by concatenation, is therefore **not** covered by any clause above unless the
  function also writes the ledger statically. What the rule does with A04 is measured, not
  arranged.

## 6. The two niles-interp additions

### 6.1 Money arithmetic outside the checked tier

`+` and `-` of two `Money` values, unary `-` of a `Money`, and `<`, `<=`, `>`, `>=` between two
`Money` values:

* They are evaluated when both operands have the same currency (compared as the kernel compares,
  upper-cased) and the same scale.
* A mismatch raises a run-time error naming both currencies.
* Overflow raises, as integer arithmetic does.

The checker already refuses mixed-currency arithmetic statically. This path exists so that a
well-typed program runs, and it re-checks what the type system proved, so that it cannot become a
second, unchecked way to the same operation (the reason `lib.rs` gave for refusing it).

### 6.2 `hold` and `resolve`, as the specification writes them

By the author's decision of 2026-09-30 (option "as written"):

* **`hold(acct, amount, …)`** appends an open hold (account, amount, currency) to the
  interpreter's ledger and returns a linear `Hold` value carrying its id. Named arguments other
  than the account and the amount (`expires:`, an authority) are evaluated and not otherwise
  interpreted.
* **`resolve h void`** and **`resolve h expire`** close the hold and post nothing.
* **`resolve h post amt`** closes the hold and records the captured amount. It posts nothing,
  because Appendix B names no counter-party. `amt` must be of the hold's currency and not exceed
  it, or it raises.
* Resolving a hold twice, or resolving one that does not exist, raises. The checker refuses the
  first statically (NL0320/NL0321), and this is its run-time counterpart.

Capability arguments (`Auth`, T09) are **not** part of this card, and no choice about them arises
here.

## 7. Mutation, classes and the E30-corpus reading

* **Operators and site patterns: E30's M1–M6, unchanged** (`crates/syntax-study/src/mutate.rs`).
  No new operator is added.
* **R2 sites:** R2 programs get the same patterns. M2's SQL pattern is extended to R2's shape, so
  that the sign of the `amt` expression in a ledger insert is toggled (`-(m).minor` ↔ `(m).minor`),
  exactly as R1 toggles the sign inside `row(…)::C`. **This is the only pattern change, and it
  is fixed here.**
* **Annotation sites:** the effect annotations are strings, so M1 and M6 find sites inside them, as
  they do in Niles's `! { … }` rows. Every mutant row records whether its site is in an annotation.
  The results report rates with and without those sites.
* **Classes: E30's five**, in E30's order: static, runtime, silent, equivalent, unexecuted.
  Static-rejection rate = static ÷ (static + runtime + silent).
* **The E30-corpus reading** is supporting evidence, and **the adversarial corpus decides the
  rule**. For each (operator, task class) cell, and separately for R1 and R2, the results list
  the cells where Niles's static-rejection rate (NL, and RS) is strictly higher than
  SQL+C+L's, with the mutants that make the difference. These are *candidates*. A candidate
  becomes a counted class only if an adversarial case of the same defect kind confirms it.

## 8. Speed

Check time and peak heap per KLOC, in-process, by the memory probe's counting allocator, as E30
did:

* for SQL-R1 and SQL-R2 with the upgraded checker;
* for NL and RS;
* over each surface's E30 programs and adversarial programs, measured separately.

The upgraded SQL+C+L's time per KLOC is divided by Niles's (NL). **A ratio above 2 is reported as
a cost, and does not veto.**

## 9. Output

* `results/E30b-syntax.md`, rendered from `results/E30b-syntax/`, which holds the rows: programs,
  mutants, adversarial verdicts and cost. It contains:
  - the rule evaluated;
  - the adversarial table, per case and class, per checker, with false refusals;
  - the list of classes that count for the Niles grammar, or the statement that there are none;
  - the E30-corpus candidates;
  - check cost;
  - the deviations;
  - this document's commit.
* E30's files are not overwritten.
* A new claim id in `thesis/status.toml` records the verdict and references H-L1.
* After the result, the author is asked what it implies for the held decisions: 1, 4 (recursion
  and windows), 5, 6 and 8. Nothing that the answer changes is built before he answers.

## 10. Not measured, and said so

* Readability and learnability.
* The programs are written by the builder of both checkers. This is mitigated, not removed, by:
  - this pre-registration;
  - the adversarial corpus being committed before any addition is built;
  - the additions being specified here;
  - mechanical mutation.
* The adversarial corpus has one or two cases per class. It shows whether a class *can* slip past
  a checker, not how often it does in real code.
