# E17 — Subquery unnesting

*Does turning a correlated subquery into a set-at-a-time join preserve the answer, and
what does it cost?*

Reproduce with `cargo test -p nilestream-optimizer --test unnest_corpus`, and the table
below with `cargo test -p nilestream-optimizer --test unnest_corpus -- --ignored`, which
writes `results/E17-unnesting-table.md`. The rewrite is
`crates/nilestream-optimizer/src/unnest.rs`; the corpus is
`crates/nilestream-optimizer/tests/unnest_corpus.rs`.

## What had to exist first

Three things the IR did not have.

**A nested form.** A correlated subquery had no representation at all: the surface AST had
no `exists`, the IR had no dependent join, and "unnesting" was therefore a rewrite with no
input. `Op::Apply { kind, correlation }` is that input — inputs `[outer, inner]`, and for
each outer row the matching set of the inner relation, with [`ApplyKind`] deciding what is
done with it. It is deliberately **not incremental**: one new outer row re-scans the inner
relation, so a dependent join has no delta rule, and `verify` refuses one on a path to a
served output (IR018). That reframes the whole exercise. Unnesting is not an optimisation
that a busy planner may skip; it is the step that makes a correlated query expressible as
a view at all, and the speedup is a consequence.

**A null.** The IR's value model was `i128`. `not in` cannot be stated without a null, and
the roadmap makes one wrong `not in` case a kill criterion, so `crates/niles-ir/src/value.rs`
adds `Value::{Null, Int}` and Kleene three-valued logic. The module keeps the thesis's
three absences apart by name — `null` (unknown in the data), `Option::None` (absent in a
program) and `Slot::Absent` (evicted, and reconstructible) — because collapsing any pair
is a defect with a name, and the third of those collapses is the `Err(_) => 0` of §1.1.1.

**One semantics, not two.** The reference evaluator lived in a `#[cfg(test)]` block inside
`schedule.rs`. It moved to `crates/niles-ir/src/eval.rs` and is now public, because two
copies of a semantics is two semantics: a rewrite could be "correct" under the copy its
own author wrote. The schedule catalogue's 1,000-trial denotation tests and the unnesting
corpus now share it.

## Method

Twenty-four correlated queries, each built twice — nested and unnested — and checked three
ways.

1. **Denotationally.** Evaluate both circuits on the same dataset; the Z-sets must be
   equal. Deliberately not "the unnested plan contains a semi-join": a plan can contain a
   semi-join and compute the wrong thing, and a structural assertion passes it.
2. **Against an oracle**, for the eight `not in` cases. Denotational equivalence says the
   rewrite is faithful to the `Apply`; it says nothing about whether the `Apply` is
   faithful to SQL. The oracle is written straight from the three-valued truth table, so a
   shared misunderstanding cannot cancel between the two circuits.
3. **Structurally**, as a cross-check: no `Apply` survives, `verify` passes, and the
   report names which rewrite fired.

The dataset carries the four things that break rewrites — duplicates, retractions (negative
weights), nulls in the probed column on each side, and empty correlation groups — and is
scale-parameterised, for the reason in the results below.

## The five rewrites

| Nested | Unnested |
|---|---|
| `exists (…)` | semi-join on the correlation |
| `not exists (…)` | anti-join on the correlation |
| `x in (…)` | semi-join, with the probe folded into the key |
| `x not in (…)` | filter, anti-join, **and a null witness** |
| scalar subquery | left outer join against a grouped aggregate |

Four are one node. The fifth is the one that is usually wrong.

## `not in`, and why an anti-join is not enough

Writing `T`, `F`, `U` for true, false and unknown, and remembering that `where` keeps a row
only when the predicate is `T`:

* the probe is null → `U` → dropped, **even against an empty subquery**;
* some in-scope value equals the probe → `F` → dropped;
* else some in-scope value is null → `U` → **dropped, although nothing matched**;
* else → `T` → kept.

The third line is the one that surprises, and it is why a plain anti-join is wrong: an
anti-join keeps exactly the rows that did not match, unknown ones included. A single null
in the subquery's column makes `not in` return **no rows at all** for every non-matching
probe. The first line is the same mistake in the other direction: an anti-join *keeps* a
null probe, because it matched nothing.

The rewrite is therefore

```text
Filter(probe is not null)                        -- the first line
  |> AntiJoin(inner,  on correlation ∪ {probe = inner})   -- the second
  |> AntiJoin(witness, on correlation)                    -- the third
```

with `witness = Distinct(Map(correlation)(Filter(inner is null)(R)))`: one row per
correlation group containing a null. The uncorrelated case needs no special handling — with
an empty correlation the second anti-join has an empty key, every pair matches, and the
result is "keep the rows iff the witness is empty", which is exactly the rule.

`a_plain_anti_join_would_have_been_wrong_which_is_why_the_witness_exists` builds the naive
rewrite by hand and asserts the two really do differ on this dataset, so the witness is
load-bearing rather than decorative.

## Results — correctness

| Check | Result |
|---|---|
| All 24 cases unnest, and fire exactly the expected rewrite | **pass** |
| Nested and unnested denote the same Z-set | **pass**, all 24 |
| The 8 `not in` cases match a hand-written three-valued oracle — nested *and* unnested | **pass** |
| No `Apply` survives; the unnested plan verifies | **pass** |
| The nested plan is *rejected* by the verifier (IR018) | **pass**, all 24 |
| A widened semi-join is rejected as duplicate inflation (IR017) | **pass** |
| Unnesting through `limit`/`order_by` is refused, with a reason | **pass** |
| Unnesting through an uncertified UDF is refused; certifying it lifts the refusal | **pass** |

No `not in` case is wrong, so the roadmap's kill criterion is not triggered.

## Results — cost

Counted work is row-operations in the reference evaluator: source rows read, join build and
probe steps, pairs produced, groups touched.

| k | outer rows | inner rows | corpus ratio | **correlated-regime ratio** |
|---|---|---|---|---|
| 1 | 9 | 7 | 1.27x | **1.44x** |
| 4 | 36 | 28 | 2.92x | **4.29x** |
| 16 | 144 | 112 | 6.07x | **15.71x** |
| 64 | 576 | 448 | 8.81x | **61.39x** |

The correlated column quadruples as `k` quadruples: 1.44 → 4.29 → 15.71 → 61.39. That is
the signature of removing a quadratic, not of shaving a constant, and it is the claim the
gate asserts (`r16 > 3·r4`, `r64 > 3·r16`).

The corpus column grows sub-linearly, and the reason is reported rather than smoothed: the
corpus contains three regimes and only one of them has an asymptotic win to give.

| Regime | Cases | Why |
|---|---|---|
| **Correlated** | 13 | `|outer| × |inner|` becomes `|outer| + |inner|`. The measured claim. |
| **Uncorrelated** | 4 | With no correlation every outer row matches every inner row; both plans are quadratic. |
| **Empty inner** | 7 | The subquery evaluates to nothing, so the nested loop's inner iteration never runs. There is no quadratic to remove; these cases are in the corpus for their *answers*, not their cost. |

A sum over a mixture of asymptotics is a number without a meaning, so the regime is
*computed* per case (by evaluating the apply's inner input) rather than declared, and
`the_three_regimes_are_all_represented_and_none_is_empty` guards the partition so that a
case which failed to speed up cannot be quietly reclassified.

## Two things the measurement corrected

**The evaluator was the wrong instrument first.** The initial reference evaluator executed
every equi-join as a nested loop, at `|L| × |R|` — the same cost as a dependent join — and
therefore reported that unnesting saved nothing. The evaluator was wrong, not the rewrite:
no engine executes an equi-join that way, and a cost model that says otherwise cannot
distinguish the two plans this gate exists to compare. It now builds an index on the right
and probes it, at `|R| + |L| + pairs`. An `Apply` keeps its nested loop, because that is
what a dependent join *is*.

**There is a crossover, and it is reported.** At the original single-instance dataset — nine
outer rows, ten inner — the `not in` cases did *more* work after unnesting, because the
unnested form is six operators and at that size the constant beats the asymptote. Loosening
the assertion would have discarded the actual result. The corpus is now scale-parameterised
and the crossover is measured per case: eleven of the thirteen correlated cases are already
cheaper at k = 1, and the two `not in` cases carrying a null witness cross at k = 2. A test
pins the worst crossover at ≤ 8, so a future change that adds a node to a rewrite shows up
as a moved number rather than as a quietly worse plan at small scale.

Three cases have a ratio **below 1** at every scale, and they are the empty-inner ones:
`not in, empty inner` (0.53x), `not in, inner filtered to nothing` (0.52x) and
`scalar count over an empty group` (0.67x). This is the honest shape of the rewrite — when
the subquery is empty there is nothing to save and the extra nodes are pure cost — and it
is a fact a planner could act on, which is why it is in the table rather than in a footnote.

## What this does and does not establish

**Establishes.** That the five rewrites preserve the answer on a corpus built to break
them; that `not in` is implemented to SQL's three-valued rule, checked against an
independent oracle rather than against itself; that the unnested form removes a quadratic,
measured, with the constant-factor regime where it does not; and that the verifier now
refuses both a dependent join on a served path and a semi-join wide enough to duplicate.

**Establishes, as of the surface round below.** That a query a user can write reaches the
rewrite: `select k from t where exists (select 1 from u where u.k = t.k)` lowers to an
`Apply` with the correlation extracted, and `unnest` turns it into a semi-join.

**Does not establish.** That a *scalar* subquery is reachable from the surface. The rewrite
is built and in the corpus; the projection path in `lower.rs` does not yet emit an `Apply`
for one, so that form is verified but unreachable. Nor are subqueries in `having`, nor
`in` over a row constructor, which is refused by name (NL0502) rather than approximated.

**Does not establish.** Anything about wall-clock. Counted work is the right unit for a
*rewrite* gate — a rewrite either does less work or it does not, and that is a property of
the plan rather than of the machine — but it is not a substitute for E16's wall-clock
measurement, and the two answer different questions.


---

# Round 2 — the surface, and two defects found on the way to it

The rewrite above was measured against circuits built by hand, because nothing a user
could write reached it: the AST had no `exists`, and `lower.rs` produced no `Apply`. Round
2 closes that path — and found, in the first five minutes, a defect that had nothing to do
with subqueries and was considerably worse than the feature being added.

## The path

`Expr::Exists { query }` in the AST, parsed by `exists (select …)`. `not exists` needs no
variant: it parses as `Unary { Not, Exists }`, because `not` is an ordinary prefix operator
and a fused node would put two spellings of one thing in the tree. `x in (select …)` and
`x not in (select …)` already parsed, as `Binary { In | NotIn, rhs: Expr::Select }`.

Lowering splits a `where` clause into subquery predicates and a residual. Only **top-level
conjuncts** are lifted: a subquery under an `or` stays where it is and is then refused,
because an `Apply` is a pipeline node and cannot be one arm of a disjunction without first
becoming a semi-join and a union — a different rewrite, not implemented. Refusing loudly
beats lowering something that is not the query that was written.

The correlation is read out of the subquery's own `where`: an equality whose two sides
belong to different relations is a correlation pair, and everything else stays as a filter
on the inner side.

## Defect 3 — the qualifier has to decide

The first version of that split asked "which schema does this name resolve in". For the
commonest correlated predicate there is — `where u.k = t.k` — the answer is *both*, since
both relations have a column called `k`. The rule gave up and left the equality as a filter
on the inner side, where it lowered to `k = k`: always true, so the subquery matched every
row and the `exists` became a no-op that returned the whole outer relation.

The fix reads the qualifier and uses the relation it names, falling back to resolution only
when there is no qualifier to read. `the_qualifier_decides_which_side_a_correlation_column_is_on`
pins both halves: the correlation is found, *and* no leftover filter is left behind
restating it as a tautology.

## Defect 4 — `=` in a `where` clause, and the silent `true`

`select k from t where t.z = 1` lowered to `Filter { predicate: LitBool(true) }`. The view
returned every row. Two independent faults compounded:

1. **The parse.** Niles's two ancestries disagree about one character. In Rust `a = b`
   assigns; in SQL's `where` clause it compares, and SQL has no assignment expression at
   all. The parser took the Rust reading everywhere, so the predicate became an `Assign`
   node — a form no `where` clause can contain.
2. **The lowering.** All three predicate sites read
   `self.scalar(..).unwrap_or(Scalar::LitBool(true))`. A predicate with no lowering became
   the constant `true`, so the clause was discarded silently.

Either alone is a bug; together they are the `Err(_) => 0` defect of §1.1.1 at plan level.
The query looked correct, the plan verified, and no answer-level test could catch it,
because every row it returned was a real row. Only reading the circuit found it.

The parser now carries a `sql_depth` counter — inside a SQL statement `=` is equality —
and `Lx::predicate` replaces the fallback with a diagnostic (NL0501). There is deliberately
no safe default: `true` returns rows that should have been filtered out and `false` hides
rows that exist, so the only honest behaviour is to refuse the view and name the predicate.
`an_unlowerable_predicate_is_an_error_and_not_a_default` pins that, including that the
specific message *replaces* the generic "this view has no lowering" rather than joining it.

This is the third time in this programme that a "reasonable default" for an absence has
turned out to be a wrong answer wearing a plausible shape — after `Err(_) => 0` in the
kernel and `sum` of an empty group. The pattern is worth naming in §11.5.

## Results — round 2

| Check | Result |
|---|---|
| `exists`, `not exists`, `in (select)`, `not in (select)` parse | **pass** |
| Each lowers to an `Apply` with the correlation extracted | **pass**, 4 forms |
| A local subquery predicate stays a filter on the inner side | **pass** |
| A conjunct beside a subquery keeps its column index across the apply | **pass** |
| A subquery under an `or` is refused (NL0501) | **pass** |
| A multi-column `in` is refused by name (NL0502) | **pass** |
| `=` in a `where` clause is a comparison; assignment survives outside SQL | **pass** |
| A `where` clause reaches the circuit as a predicate, never as `true` | **pass** |
| All four surface forms reach their rewrite end to end | **pass** |

`crates/niles-lang/tests/subqueries.rs`, 15 tests.
