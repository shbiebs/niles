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

**SQL-Core, stated precisely — and the fragment is now the parser's.** The definition below was produced by putting each construct through `nilesc` and recording what came back, not by listing what a translation ought to cover. The fragment covers: `SELECT` with projection, expressions and aliases; `FROM` with base tables and comma-separated relations; `JOIN` — **inner, left, right and full** — with `ON`; `WHERE`; `GROUP BY` with `HAVING`; the aggregates `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`; `ORDER BY`, `LIMIT`, `OFFSET` with literal bounds; the set operations `UNION`, `UNION ALL`, `EXCEPT`, `INTERSECT`; correlated `EXISTS` and `NOT IN`; three-valued logic with `NULL`, `IS NULL`, `IS NOT NULL` and null-propagating comparisons; **bag semantics** including duplicate preservation and `DISTINCT`; `AS OF` on the epoch axis; and `WITH RECURSIVE` mapping to the guarded fixpoint.

**Refused, and therefore outside SQL-Core.** Each is refused with a named code and each has a case in the golden corpus, so the boundary is in the build rather than in a reader's assumptions:

| Construct | Code | Why |
|---|---|---|
| `CROSS JOIN` | NL0516 | Every join operator in the IR joins on a key and there is no product operator. This one is instructive: until this cycle a cross join *lowered to the keyed inner join and answered a different query* — four rows where twelve were asked for — and no case in the corpus covered it. |
| `USING (k)` | NL0001 | Not parsed. `ON` is the only join condition in the fragment. |
| `COUNT(DISTINCT x)` | NL0002, NL0001 | `distinct` is a reserved word and the aggregate-argument position does not accept it. |
| `CASE WHEN` | NL0508 | The projection list lowers scalars the IR has an operator for; a conditional is not among them. |
| `WITH` (non-recursive) | NL0500 | Not lowered — while `WITH RECURSIVE` *is*, through the guarded fixpoint. The easier form being the missing one is exactly the kind of gap a fragment stated from the parser exposes and a fragment stated from ambition hides. |
| `LIMIT ⟨non-literal⟩` | NL0504 | A bound the lowering cannot read became "every row". |
| a scalar subquery in the projection list | NL0508 | |
| a set operation between different arities | NL0512 | |
| `SELECT` with no `FROM` | NL0511 | |
| DDL, DML, TCL, DCL | — | Surface syntax with no circuit; see below. |

**DDL, DML, TCL and DCL are not part of this claim.** An earlier statement folded them into the translation, which cannot be right: a `CREATE TABLE` denotes no Z-set, so "lowers to an α-equivalent circuit" is not a statement about it. They are accepted as surface syntax where the language has them (`insert`/`update`/`delete` against a `table` only, W4), refused where it does not (`alter`, `drop`), and the theorem quantifies over queries.

**Explicitly outside the fragment**, and refused with a named error rather than approximated: implementation-defined collation and locale behaviour; procedural extensions; cursors with update semantics; triggers (whose effects are expressed as views or transaction-tier code instead); user-defined types outside the declared type system; and any construct whose standard behaviour is implementation-defined in a way that would make the translation's semantics-preservation claim vacuous.

**Proof method.** Structural induction on the fragment's grammar. For each production, a lowering rule into IR is given, and the induction hypothesis is that the rule preserves the bag-semantics denotation with null handling. The interesting cases are: `GROUP BY` with `HAVING` over bags, where the delta form must handle group emptiness; `LEFT JOIN` under incremental maintenance, where null-extension must be retracted correctly when a match later appears; `DISTINCT`, whose delta form is the one DBSP treats explicitly; three-valued logic, which is lowered to explicit option handling rather than left implicit; and `WITH RECURSIVE`, which lowers to the guarded fixpoint and therefore inherits the guardedness requirement — a recursive SQL query without a well-founded measure is rejected, and this is a *deliberate incompatibility* with SQL's permissiveness, stated as such.

**Testing.** A golden corpus of 42 cases in `crates/niles-lang/tests/golden`, each carrying the answer it must produce on a fixed four-row dataset written out in `DATA.md`, so an expected file can be read without running anything. Nineteen cases carry both spellings and the test asserts the two denote the same Z-set; the rest are refusals, or SQL forms with no pipeline spelling, and the test now *counts and prints* what it skips so that a case cannot quietly opt out of the comparison. Failures are specification bugs, not test flakes, and the corpus is part of the build gate.

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
```
| SQL-Core translation, semantics-preserving | Constructed by structural induction, tested by golden-file equivalence (H.4) |
| Computable-query completeness of the declarative tier | **Not claimed**, deliberately (H.5) |
| Full SQL-standard compatibility including implementation-defined behaviour | **Not claimed**; out-of-fragment constructs fail loudly (H.4) |
| Workload-class coverage | Constructed and audited, not proved (H.7) |
