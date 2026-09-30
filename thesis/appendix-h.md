# Appendix H. Generality and SQL-Completeness Dossier

This appendix discharges Theorem 4.6 in detail: what the generality claim means, what is proved, what is constructed, and what is deliberately not claimed.

## H.1 Why the Naive Claim Is Not Provable

"Niles is SQL-complete" is not a well-formed statement. SQL is defined by a multi-part international standard with optional features, implementation-defined behaviour, and no single formal semantics against which completeness could be established. Any thesis asserting completeness with respect to SQL is asserting completeness with respect to an object that does not exist in the required form.

Three well-formed statements are available instead, and this appendix proves the first two and constructs the third.

## H.2 Claim 1 — Relational Completeness

**Statement.** Niles-Core expresses every query expressible in the safe (domain-independent) relational calculus.

**Construction.** Each primitive relational operator is exhibited as a Niles pipeline stage:

| Operator | Niles |
|---|---|
| Selection σ | `.where(pred)` |
| Projection π | `.map(f)` |
| Cartesian product × | `.join(other, \|_,_\| true)` |
| Union ∪ | `.union(other)` |
| Difference − | `.except(other)` |
| Rename ρ | `.map` with field construction |

Closure under composition holds because every stage returns a relation-typed value. Codd's reduction of calculus expressions to algebra expressions then gives the result, and the modern development of algebra-equals-safe-calculus supplies the converse direction. ∎

**What this does and does not buy.** It is table stakes: it says nothing about recursion, aggregation, nulls or bag semantics, all of which SQL has and calculus does not. Any language claiming to replace SQL must clear this bar and then keep going.

## H.3 Claim 2 — Fixpoint Completeness over the Epoch-Ordered Base

**Statement.** Niles-Core with guarded recursion expresses exactly the queries computable in polynomial time over the base.

**Proof.** Niles-Core includes relational calculus (H.2) and a least-fixpoint construct (`.fixpoint(step) guard measure(m)`). Immerman's theorem states that relational calculus plus least fixpoint, *over structures containing a total ordering relation*, expresses exactly the PTIME queries; the result was obtained independently by Vardi.

The ordering hypothesis is the usual obstacle to applying this theorem to relational data, since a relation is an unordered set and the theorem fails without order. **Here the hypothesis is satisfied by the data model itself.** The base is an epoch-ordered sequence of appended rows; epochs are totally ordered by construction (Definition 3.1), and rows within an epoch are canonically ordered by the serialization that the hash chain commits to. The total order is therefore not an assumption added for the proof but a property the system must have anyway for its hash chain and its determinism obligation.

Guardedness restricts recursion to well-founded measures, which is what keeps evaluation in PTIME rather than merely computable, and is checked by the compiler (Section 9.12). ∎

**Why this is the right headline claim.** It is a theorem, it is checkable, it is strictly stronger than relational completeness, and it captures a real capability SQL only acquired by extension (recursive queries). It also has an agreeable structural moral: the design decision taken for auditability — total order over an immutable base — is the same decision that buys the expressiveness result.

## H.4 Claim 3 — The SQL-Core Translation

**Statement.** There is a total compilation of SQL-Core into the Niles IR that preserves *denotation*: for every SQL-Core query and every finite instance, the Z-set the compiled circuit evaluates to is the Z-set the query denotes — and, where a query has a Niles pipeline spelling, the two spellings evaluate to the same Z-set.

Two words of an earlier statement are gone and their loss is the point. **α-equivalence of circuits** is not claimed: the corpus compares *answers*, not circuit shapes, and two lowerings that denote the same relation may legitimately differ in structure. And *semantics-preserving* is narrowed to denotational agreement on finite instances, which is what a golden corpus can witness.

**SQL-Core, stated precisely — and the fragment is now the parser's.** The definition below was produced by putting each construct through `nilesc` and recording what came back, not by listing what a translation ought to cover. The fragment covers: `SELECT` with projection, expressions and aliases; `FROM` with base tables and comma-separated relations; `JOIN` — **inner, left, right and full** — with `ON`; `CROSS JOIN`, the product (refused as NL0516 until cycle 15, when it was lowered to the empty-key join the comma form already used, C15-05b); `WHERE`; `GROUP BY` with `HAVING`; the aggregates `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`; `ORDER BY`, `LIMIT`, `OFFSET` with literal bounds; the set operations `UNION`, `UNION ALL`, `EXCEPT`, `INTERSECT`; correlated `EXISTS` and `NOT IN`; three-valued logic with `NULL`, `IS NULL`, `IS NOT NULL` and null-propagating comparisons; **bag semantics** including duplicate preservation and `DISTINCT`; `AS OF` on the epoch axis; `WITH`, whose entries are inlined, with an optional column list; and `WITH RECURSIVE … AS (base UNION step)` lowered to the guarded fixpoint, for the recursive terms SQL admits; window functions — `row_number`, `rank`, `dense_rank` and the running `sum`, `count`, `min`, `max` and `avg` — `OVER (PARTITION BY … ORDER BY …)` with SQL's default frame, as projection items of a query that does not aggregate; and `recorded_at`, a base's or ledger's system-time column, the epoch that recorded the row, readable by name and left out of `*` (these three since cycle 15, C15-05b, the author's decision 4; the recursion because the `with` list had been parsed and discarded, so a plain `WITH` was refused as NL0500 and `WITH RECURSIVE` was listed here without lowering; windows and the epoch column because E30's Q07 could not be written without them, finding F8). A `JOIN … ON` is keyed by the cross-side `column = column` conjuncts of its `ON` and filtered by the rest; until C15-05b it was keyed on the two sides' anchor indices with `ON` only a residual, so a join whose condition was not the anchor answered a subset of its rows, silently (golden case 75).

**Refused, and therefore outside SQL-Core.** Each is refused with a named code and each has a case in the golden corpus, so the boundary is in the build rather than in a reader's assumptions:

| Construct | Code | Why |
|---|---|---|
| `USING (k)` | NL0001 | Not parsed. `ON` is the only join condition in the fragment. |
| `COUNT(DISTINCT x)` | NL0002, NL0001 | `distinct` is a reserved word and the aggregate-argument position does not accept it. |
| `CASE WHEN` | NL0508 | The projection list lowers scalars the IR has an operator for; a conditional is not among them. |
| a `WITH` column list naming a different number of columns than its body produces | NL0523 | Cycle 15 (C15-05b). The list renames the body's columns one for one. |
| `WITH RECURSIVE … AS (base UNION ALL step)` | NL0524 | Cycle 15 (C15-05b). `union all` asks for the bag PostgreSQL's working-table iteration produces; the guarded fixpoint computes a set, so the text would be answered as a different query, and on a cyclic graph the bag never stops growing. `union` lowers. |
| a recursive step outside the terms SQL admits: an aggregate, `order by`/`limit`, a further set operation, the CTE read twice, in a subquery or on the null-padded side of an outer join; a base that reads the CTE | NL0525 | Cycle 15 (C15-05b). These are PostgreSQL's own restrictions on a recursive term, and they are what make the step monotone, which is what gives the least fixpoint SQL's answer. |
| `LIMIT ⟨non-literal⟩` | NL0504 | A bound the lowering cannot read became "every row". |
| a scalar subquery in the projection list | NL0508 | |
| a set operation between different arities | NL0512 | |
| an expression over an aggregate in the projection list (`sum(v) * 2`) | NL0517 | The projection loop matched only *bare* aggregate calls and ignored the rest, so this lowered to an aggregate node with no aggregates and served the grouping column alone. Refusing costs an expressiveness the fragment never had; the alternative was reporting `sum(v)` under the name `sum(v) * 2`. |
| a projected column outside the `group by` | NL0517 | PostgreSQL's `must appear in the GROUP BY clause`; here the column was silently dropped from the result. |
| a `group by` column the projection does not name | NL0517 | The aggregate operator emits every grouping column, so the result would carry a column the query never asked for. |
| a grouping column projected after an aggregate | NL0517 | The operator emits keys before aggregates, so the column cannot be placed where it was written; reordering it silently would be a wrong answer that looks right. |
| a `having` that tests an aggregate the projection does not compute | NL0518 | Cycle 14 (R2-06). An aggregate *call* in `having` — `having sum(amt) < 0`, as SQL writes it — lowered to an argument-less call and dropped every group, so the query answered no rows with no diagnostic. It now names the output column holding that aggregate; an aggregate the query does not compute has no column to name and is refused. |
| a keyed join (or `intersect`) whose two sides are keyed on different numbers of columns | NL0519 | Cycle 14 (R2-06). Every join the IR executes matches the left key against the right key column by column, so `postings` anchored on `(acct, cur)` joined to `accounts` anchored on `(id)` could not produce a row — on either surface — and answered empty with no diagnostic. Since cycle 15 (C15-05b) the SQL surface's join is keyed by its `ON`, so this holds for the pipeline's `.join(u)` and for `intersect` only. |
| an `order_by` key that is not a column, a tuple of keys, or `asc(..)`/`desc(..)` around one (`.order_by(\|r\| (-r.sum, r.acct))`) | NL0509 | Cycle 14 (R2-06). The key walker skipped what it did not recognise, so the negation meant as a descending order vanished, the view was ordered by `acct` alone and `.limit(5)` returned the five lowest ids. Descending is written `desc(r.sum)`. |
| a column name that more than one side of a join carries, with no qualifier naming the side and no join key making them one value | NL0520 | Cycle 14 (R2-06). Names resolved first-match, qualifiers were read as nothing, and a pipeline join's residual was lowered against its left input and dropped when it did not lower — so the self-join `on q.cur = p.cur and q.txn < p.txn`, on either surface, compared each row with itself and answered empty. A from-list alias and a two-parameter join closure's parameters now name their side's columns; `q.x` for a side with no `x`, and an unqualified name on two sides the key does not equate, are refused. `group by` and an aggregating projection read the qualifier too; in `order by` a qualified name is still read by its name alone, so a duplicated one there is refused rather than resolved. |
| a string that names no declared currency, against a currency column (`cur = "xyz"`); a string against a column that is neither text nor currency (`k = "one"`) | NL0521 | Cycle 14 (R2-06), narrowed in cycle 15 (C15-05b, decision 6). Until cycle 14 every string evaluated as `0`, the first declared currency's code (E30's F11). R2-06 then refused every string that was not a currency's name. Since C15-05b the IR has text values: a string against a `Text` column is text, with equality, ordering by content and `like`, so `name = "alice"` and `like "ac%"` lower; a declared currency's name against a currency column is its code. What is still refused is a string no column could hold. |
| a money literal against an amount whose currency the lowering cannot find (a `Money` column with no currency column, no filter pinning one, and no `Money<c>` type) | NL0522 | Cycle 15 (C15-05b, decision 8). A money literal met an amount as a number, so `having sum(amt) < 0.00 eur` over usd rows answered (E30's F13). The amount is now lowered with its currency (`Scalar::InCurrency`: the declared `Money<c>`, a conjunct such as `cur = "usd"`, the row's own currency column, or a filter upstream), and the evaluator refuses a comparison or a sum across two currencies. Where none of those gives the currency, the comparison is refused here. An aggregate's amount carries its currency too, where the group key or a filter does not already fix it (the author's decision R2-d): `select acct, sum(amt) from postings group by acct` over usd and eur postings is refused by the reference evaluator as the server refused it, where it had answered their integer sum (finding F-05b-6). |
| a window function outside the projection (`where`, `having`, `group by`, `order by`), inside an expression, or over a query that aggregates | NL0526 | Cycle 15 (C15-05b). SQL computes windows after `where`, `group by` and `having`, so none of them can read one; the IR's window is over rows, not groups. Aggregate in a subquery and window its result. |
| `OVER` after a function that is not a window function (`lag`, `rank(x)`), or a running aggregate whose amount's currency cannot be found | NL0527, NL0522 | Cycle 15 (C15-05b). A running sum's amount carries its currency, so a partition holding two currencies is refused at run time rather than summed. |
| a window frame clause (`ROWS BETWEEN …`, `RANGE BETWEEN …`) | NL0001 | Not parsed: the default frame — the partition up to the current row and its peers, or the whole partition without an order — is the only one the IR has. |
| `SELECT` with no `FROM` | NL0511 | |
| DDL, DML, TCL, DCL | — | Surface syntax with no circuit; see below. |

**DDL, DML, TCL and DCL are not part of this claim.** An earlier statement folded them into the translation, which cannot be right: a `CREATE TABLE` denotes no Z-set, so "lowers to an α-equivalent circuit" is not a statement about it. They are accepted as surface syntax where the language has them (`insert`/`update`/`delete` against a `table` only, W4), refused where it does not (`alter`, `drop`), and the theorem quantifies over queries.

**Explicitly outside the fragment**, and refused with a named error rather than approximated: implementation-defined collation and locale behaviour; procedural extensions; cursors with update semantics; triggers (whose effects are expressed as views or transaction-tier code instead); user-defined types outside the declared type system; and any construct whose standard behaviour is implementation-defined in a way that would make the translation's semantics-preservation claim vacuous.

**Proof method.** Structural induction on the fragment's grammar. For each production, a lowering rule into IR is given, and the induction hypothesis is that the rule preserves the bag-semantics denotation with null handling. The interesting cases are: `GROUP BY` with `HAVING` over bags, where the delta form must handle group emptiness; `LEFT JOIN` under incremental maintenance, where null-extension must be retracted correctly when a match later appears; `DISTINCT`, whose delta form is the one DBSP treats explicitly; three-valued logic, which is lowered to explicit option handling rather than left implicit; and `WITH RECURSIVE`, which lowers to the fixpoint operator. **SQL writes no measure, and the lowering does not invent one**: a recursive CTE carries the round bound alone — 1,000 rounds unless the view's contract raises it with `max_rounds` (the author's decision R2-e; Appendix D.6) — and a recursion that has not converged by then is refused at run time (`NonTerminating`) rather than answered. That is weaker than the pipeline's `guard measure(m)`, which states a termination argument, and two consequences are stated rather than hidden. A recursion that converges after more rounds than its bound — a transitive closure over a longer chain — is refused although it has an answer, unless the view raises the bound; that is incompleteness, not a wrong answer. And the guardedness Section H.3 relies on for the declarative tier is, for the SQL spelling, the bound rather than a well-founded measure: each round is polynomial and the number of rounds is constant, so evaluation stays polynomial, but no measure is checked. The earlier text of this paragraph said a recursive SQL query without a measure is rejected; no code ever did that, because until cycle 15 no recursive SQL query lowered at all.

**Testing.** A golden corpus of 75 cases in `crates/niles-lang/tests/golden` (cycle 15; this sentence said 42 and nineteen for several cycles after the corpus had grown), each carrying the answer it must produce on a small fixed dataset written out in `DATA.md`, so an expected file can be read without running anything. Twenty-three cases carry both spellings and the test asserts the two denote the same Z-set; the rest are refusals, or SQL forms with no pipeline spelling, and the test now *counts and prints* what it skips so that a case cannot quietly opt out of the comparison. Failures are specification bugs, not test flakes, and the corpus is part of the build gate.

The corpus is also what caught the `CROSS JOIN`, `RIGHT JOIN` and `FULL JOIN` defects: right and full joins parsed, lowered, verified — and evaluated to *nothing at all*, because the reference evaluator emitted matched pairs only for the inner and left kinds. The lesson is not that the constructs were broken; it is that a fragment claimed in an appendix and not written down as cases is a fragment nobody is testing.

## H.5 The Outer Bound: Computable Queries

The strongest available notion of completeness for a query language is Chandra and Harel's *computable queries* — generic, isomorphism-preserving, partial recursive functions on databases. Niles's UDF tier reaches this bar in the trivial sense that arbitrary computation is expressible there.

**The thesis does not claim it for the declarative tier, and the refusal is deliberate.** A declarative tier that can express arbitrary computation cannot be reliably planned, incrementalized, guarded, or reasoned about for cost — and Theorem 4.3's cost law, Theorem 4.6(b)'s PTIME characterization, and the optimizer's ability to choose materialization modes all depend on staying inside a tractable class. The tiering of Section 6.12 is thus an expressiveness decision, not merely an engineering one: computational generality is available, in a sandbox, outside the class where the guarantees live.

## H.6 Comparative Position

A survey of the SQL-alternative landscape yields a finding worth recording. Among Datalog and its differential variants, PRQL, Malloy, EdgeQL, Morel, Logica, LINQ, KQL and Rel, **none claims formal SQL-superset expressive power**. Several (PRQL, Malloy, Logica) compile to SQL and are therefore bounded above by it. One (KQL) is read-only by design. The one closest in ambition (Rel) argues for a full language rather than a sublanguage from design philosophy — a small core plus libraries — rather than from a theorem.

A proved expressiveness result is therefore a genuinely novel artifact in this space, and it is a better claim than "replaces SQL" precisely because a reader can check it.

## H.7 Workload-Class Coverage

Generality of the *engine*, as distinct from the language, is discharged constructively (H-S8). Each class is a REV over the same base with the same contract vocabulary; what differs is the circuit and the resident representation the optimizer chooses.

| Class | Circuit | Resident representation | Precedent adopted |
|---|---|---|---|
| OLTP | Point lookups over keyed views | Row-shaped hash or tree index | Conventional |
| OLAP / HTAP | Aggregating circuits | Column-shaped, `full` or `tiered` | Dual-format HTAP designs, obtained here as separate REVs over one immutable base |
| Time-series | Windowed aggregation over the valid-time axis | Compressed, time-partitioned | Published delta-of-delta and XOR compression schemes |
| Document | Path extraction over a semi-structured column | Path-indexed | Decomposed binary representation; schema-agnostic tree-of-paths indexing |
| Graph | Guarded fixpoint over an edge base | Adjacency-shaped | The standardized property-graph pattern vocabulary |
| Search | Term-indexing circuit | Immutable segments with tombstones and background merging | The two-decade production existence proof that this architecture serves search |

The claim being made is narrow and defensible: not that one storage format serves all workloads, but that one *semantics* does, and that format is the optimizer's choice under Theorem 4.5(a)'s safety clause. The falsifier is a workload class requiring a kernel change, which Section 9.11 audits.

## H.8 Summary of What Is Claimed

| Claim | Status |
|---|---|
| Relational completeness | Proved (H.2) |
| Fixpoint completeness over the epoch-ordered base = PTIME | Proved by reduction to Immerman–Vardi, with the ordering hypothesis supplied by the data model (H.3) |

```text
PROPOSED — MISMATCH-T-13-fixpoint. Not applied to the table row above.

The row is a claim about the *language*, and the language cannot express a transitive
closure. The theorem is untouched — the reduction to Immerman–Vardi stands, and it is about
the fixpoint operator over an ordered base — but the row as it reads invites a reader to
believe that a PTIME query can be written, and one cannot.

WHAT WORKS

  The fixpoint operator evaluates. `Op::Fixpoint` takes the seed and the step's output, with
  the step reading the accumulator through the `Delay` that closes the cycle; it runs to a
  least fixpoint and reports non-convergence as `EvalError::NonTerminating { rounds, tail }`
  rather than returning what it had reached. Case 31 of the golden corpus reaches closure;
  `a_non_terminating_fixpoint_is_refused_rather_than_answered` shows a growing step refused
  with the round count and the accumulator's last sizes. Before this the step was *discarded
  at lowering* — the node held a termination guard and no body, so the operator was
  verifiable and not evaluable, and C6(b) had no runnable witness at all.

WHAT DOES NOT

  Two pieces of surface syntax are missing, and a closure needs both:

    1. A join cannot state its key. `.join(u)` takes the two sides' anchor keys, so a step
       can only join the accumulator's `src` to `edges`' `src`; a closure needs `acc.dst` to
       `edges.src`.
    2. A projection cannot name a duplicated column. After a join the schema is
       [src, dst, src, dst] and `col_index` returns the first match, so the pair a closure
       step must produce — the outer `src` with the inner `dst` — has no spelling.

  So the step cannot produce the shape it consumes, and NL0514 refuses it: the right refusal
  for the wrong reason, an arity check doing the work a missing feature should be reported by.
  Case 36 of the golden corpus records it.

THE REPLACEMENT WORDING

  The table row:
    "Fixpoint completeness over the epoch-ordered base = PTIME | Proved by reduction to
     Immerman–Vardi, with the ordering hypothesis supplied by the data model (H.3).
     **Not exercised: the stage-0 surface cannot express a transitive closure** — a `join`
     stage takes no explicit key and a projection cannot name a duplicated column — so no
     PTIME query has been written in Niles. The operator itself evaluates to a least fixpoint
     and refuses non-convergence (golden cases 31 and 36)."

  §4.7's C6(b), correspondingly: the completeness claim is a claim about the calculus, and
  the surface's shortfall is named beside it rather than left to a reader to discover by
  trying.

STATUS, cycle 15 (C15-05b): the premise now holds for one surface only. The SQL surface
  writes the closure — `with recursive r(a, b) as (select src, dst from edges union select
  r.a, e.dst from r join edges e on r.b = e.src) select * from r`, golden case 71, ten pairs
  over a graph with a cycle — because a SQL join names its key in `on` and a qualified
  column names its side. The pipeline surface still cannot (case 36). A PTIME query has now
  been written; the fixpoint that answers it is guarded by its round bound, not a measure
  (H.4's proof-method paragraph).
```
| SQL-Core translation, semantics-preserving | Constructed by structural induction, tested by golden-file equivalence (H.4) |
| Computable-query completeness of the declarative tier | **Not claimed**, deliberately (H.5) |
| Full SQL-standard compatibility including implementation-defined behaviour | **Not claimed**; out-of-fragment constructs fail loudly (H.4) |
| Workload-class coverage | Constructed and audited, not proved (H.7) |
