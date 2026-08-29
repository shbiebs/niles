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

**Statement.** There is a total, semantics-preserving compilation of SQL-Core into the Niles IR, such that an SQL-Core query and its Niles counterpart lower to α-equivalent circuits.

**SQL-Core, stated precisely.** The fragment covers: `SELECT` with projection, expressions and aliases; `FROM` with base tables, subqueries and derived tables; `JOIN` (inner, left, right, full, cross) with `ON` and `USING`; `WHERE`; `GROUP BY` with `HAVING`; aggregate functions (`COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, `COUNT DISTINCT`); `ORDER BY`, `LIMIT`, `OFFSET`; set operations (`UNION`, `UNION ALL`, `EXCEPT`, `INTERSECT`); scalar and correlated subqueries in the positions where they are safe; `CASE`; three-valued logic with `NULL`, `IS NULL`, and null-propagating comparisons; **bag semantics** including duplicate preservation and `DISTINCT`; `WITH` and `WITH RECURSIVE` (the latter mapping to guarded fixpoint); DDL (`CREATE TABLE`, `CREATE VIEW`, `CREATE INDEX`, constraints); DML (`INSERT`, `UPDATE`, `DELETE`) restricted to `table` objects; TCL (`BEGIN`, `COMMIT`, `ROLLBACK`); DCL (`GRANT`, `REVOKE`); and the temporal forms of the standard's system-versioned tables (`AS OF`, period predicates), which map onto the epoch and valid-time axes.

**Explicitly outside the fragment**, and refused with a named error rather than approximated: implementation-defined collation and locale behaviour; procedural extensions; cursors with update semantics; triggers (whose effects are expressed as views or transaction-tier code instead); user-defined types outside the declared type system; and any construct whose standard behaviour is implementation-defined in a way that would make the translation's semantics-preservation claim vacuous.

**Proof method.** Structural induction on the fragment's grammar. For each production, a lowering rule into IR is given, and the induction hypothesis is that the rule preserves the bag-semantics denotation with null handling. The interesting cases are: `GROUP BY` with `HAVING` over bags, where the delta form must handle group emptiness; `LEFT JOIN` under incremental maintenance, where null-extension must be retracted correctly when a match later appears; `DISTINCT`, whose delta form is the one DBSP treats explicitly; three-valued logic, which is lowered to explicit option handling rather than left implicit; and `WITH RECURSIVE`, which lowers to the guarded fixpoint and therefore inherits the guardedness requirement — a recursive SQL query without a well-founded measure is rejected, and this is a *deliberate incompatibility* with SQL's permissiveness, stated as such.

**Testing.** Golden-file α-equivalence over a corpus: each SQL-Core query and its hand-written Niles counterpart must lower to circuits equal up to renaming. Failures are specification bugs, not test flakes. The corpus is part of the build gate.

## H.5 The Outer Bound: Computable Queries

The strongest available notion of completeness for a query language is Chandra and Harel's *computable queries* — generic, isomorphism-preserving, partial recursive functions on databases. Niles's UDF tier reaches this bar in the trivial sense that arbitrary computation is expressible there.

**The thesis does not claim it for the declarative tier, and the refusal is deliberate.** A declarative tier that can express arbitrary computation cannot be reliably planned, incrementalized, guarded, or reasoned about for cost — and Theorem 4.3's cost law, Theorem 4.6(b)'s PTIME characterization, and the optimizer's ability to choose materialization modes all depend on staying inside a tractable class. The tiering of Section 6.12 is thus an expressiveness decision, not merely an engineering one: computational generality is available, in a sandbox, outside the class where the guarantees live.

## H.6 Comparative Position

A survey of the SQL-alternative landscape yields a finding worth recording. Among Datalog and its differential variants, PRQL, Malloy, EdgeQL, Morel, Logica, LINQ, KQL and Rel, **none claims formal SQL-superset expressive power**. Several (PRQL, Malloy, Logica) compile to SQL and are therefore bounded above by it. One (KQL) is read-only by design. The one closest in ambition (Rel) argues for a full language rather than a sublanguage from design philosophy — a small core plus libraries — rather than from a theorem.

A proved expressiveness result is therefore a genuinely novel artifact in this space, and it is a better claim than "replaces SQL" precisely because a reader can check it.

## H.7 Workload-Class Coverage

Generality of the *engine*, as distinct from the language, is discharged constructively (S8). Each class is a REV over the same base with the same contract vocabulary; what differs is the circuit and the resident representation the optimizer chooses.

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
| SQL-Core translation, semantics-preserving | Constructed by structural induction, tested by golden-file equivalence (H.4) |
| Computable-query completeness of the declarative tier | **Not claimed**, deliberately (H.5) |
| Full SQL-standard compatibility including implementation-defined behaviour | **Not claimed**; out-of-fragment constructs fail loudly (H.4) |
| Workload-class coverage | Constructed and audited, not proved (H.7) |
