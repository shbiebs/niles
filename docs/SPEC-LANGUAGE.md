# SPEC-LANGUAGE — the ideal query language

**Normative specification.** Twenty-four requirements, each with a rationale, its status in
prior art, the mechanism that satisfies it, and **an acceptance test**. A requirement with
no acceptance test is an aspiration, and this document contains none.

Numbering matches `REQUIREMENTS.md` §2. Status values:

| | Meaning |
|---|---|
| **Built** | Implemented in this repository, with the named test passing |
| **Partial** | Implemented for a stated fragment; the gap is named |
| **Specified** | Designed here, not built |
| **Adopt** | Solved in prior art; the design decision is to take it rather than invent |

Conformance language is RFC 2119: **MUST**, **MUST NOT**, **SHOULD**, **MAY**.

---

## Part I — The execution model

### L-1 Native speed

**The language MUST compile to native machine code with no runtime dispatch on the hot path,
and MUST NOT require a language runtime to be present at execution.**

*Rationale.* "Speed of C" is not a slogan here but a constraint on the type system: it is
achievable only if every call site's target is known statically, which means no dynamic
dispatch on arbitrary runtime types. That is the requirement that Ruby-dynamism would have
made unsatisfiable, and its removal is what makes this specification coherent
(`REQUIREMENTS.md` §1).

*Prior art.* Rust, Zig and C demonstrate it. Julia does not, and its failure is instructive:
`juliac --trim` obtains AOT compilation by prohibiting dynamic dispatch.

*Mechanism.* Monomorphisation at `comptime` (L-20); no vtable in the query path; the
imperative tier lowers to the same IR as the relational tier (L-14).

*Acceptance test.* A Niles program with no `dyn` in its signature MUST produce a binary
containing no indirect call in the query loop, verified by disassembly. **Status: Specified.**
The stage-0 compiler lowers to the IR and the runtime executes it; native emission is
Appendix E.6+ and unbuilt.

---

### L-2 Mathematically functional

**Expressions in query position MUST be referentially transparent. A function whose effect
row is empty MUST be safe to reorder, duplicate, elide, or evaluate in parallel.**

*Rationale.* This is the requirement everything else rests on. An expression with no side
effects can be transformed by the optimizer without a proof obligation per transformation,
which is the mechanism behind L-8. It is also what makes L-19 (deductive) and L-10
(set-at-a-time) the same language rather than two glued together.

*Mechanism.* Effect rows (`! { append, debit<usd> }`) make effects part of the type. Purity
is the empty row, and the compiler infers it rather than trusting an annotation.

*Acceptance test.* `crates/niles-lang/src/effects.rs` — a function declaring an empty row
whose body appends MUST fail with `NL0310`. Verified in
GBS's `tests/niles_schema.rs::an_understated_effect_row_is_rejected_with_both_spans`.
**Status: Built.**

---

### L-6 Compiled, no *tracing* GC, memory-safe

**The language MUST be memory-safe without a tracing garbage collector. Reclamation of
partially-materialised state MUST be deterministic and epoch-scoped.**

*Rationale and restatement.* Partial state with eviction has extents decided at runtime by
the read workload; there is no lexical scope to bind them to, and every engine that does
this uses epoch-based reclamation — which is a collector. The requirement is achievable when
restated as *no tracing GC*, and the restatement is free here because the epoch-ordered
ledger already defines the reclamation domain.

*The bonus result.* **The visibility timeline and the memory-reclamation domain are the same
object.** An epoch that no reader is anchored below is an epoch whose superseded REV state
is unreachable, by the same argument that makes it invisible. Reclamation therefore needs no
separate mechanism, no reference counts on view state, and no stop-the-world pause — it is
the frontier computation the engine already performs.

*Acceptance test.* A workload evicting and reconstructing 10⁶ keys MUST show bounded resident
memory with no unbounded growth, and MUST NOT exhibit a pause attributable to reclamation.
**Status: Partial** — `nilestream-core` evicts and reconstructs with counted work; the
epoch-scoped reclamation proof is specified and unwritten.

---

### L-16 / L-22 No interpretive overhead in steady state

**The engine MUST NOT impose interpretive overhead on a repeated query. It MUST NOT compile
unconditionally.**

*The contradiction, and its resolution.* Requiring compilation with no interpreter
contradicts being a PostgreSQL replacement, and the contradiction is measured rather than
argued:

| | Cost |
|---|---|
| LLVM code generation | **40–90 ms per query** (Neumann, PVLDB 2021) |
| Direct x86 emission | 1–2 ms |
| Umbra "Flying Start" bytecode tier | **108× faster to compile, 1.2× slower to execute** |

At 100 MB an interpreter finishes *every* query before a compiler has finished compiling. A
system that compiles unconditionally cannot serve OLTP, and OLTP is most of what it is
replacing.

*Mechanism — adaptive tiering, three tiers, promotion by measured cost:*

1. **Bytecode**, entered unconditionally. Latency to first tuple ≈ 1 ms.
2. **Direct machine-code emission** on promotion. 1–2 ms, no LLVM.
3. **Optimising back end** only where the query's measured runtime exceeds the compile cost
   — the crossover is ~100 ms.

*Acceptance test.* A point-lookup query MUST return in under 1 ms end to end, and a
long-running analytical query MUST reach tier 3. The tier a query ran at MUST be reportable,
so a regression is attributable. **Status: Specified.** `nilestream-core` executes the IR
directly; tiering is unbuilt.

---

### L-17 Vectorized execution

**Bulk operators SHOULD process cache-resident batches with SIMD where the operation is
data-parallel.**

*Status: Adopt — and calibrate the claim.* Solved in prior art (DuckDB, Velox, Photon).
Kersten et al. (VLDB 2018) measured the real gain: **1.4× on TPC-H Q6, ~1.1× on Q3/Q9** —
against 8.4× on a microbenchmark selection. And **compiled versus vectorized is a wash**: on
Q6 the two tie at 15 ms, and the paper's whole spread is 0.66×–1.93×, described by its
authors as "not large differences."

This is therefore a **parity requirement, not a differentiator**. A specification that
promised an order of magnitude from SIMD would be citing a microbenchmark.

*Acceptance test.* Batch size MUST be tunable and MUST default to a value fitting L1. A
scan-heavy query MUST show measurable SIMD utilisation. The reported speedup MUST be
measured on a real query, never on a microbenchmark.

---

### L-13 Zero-copy, bounded by trust

**Reading from the ledger MUST NOT copy. Reading from an untrusted buffer MUST validate.**

*The tension, precisely.* Of {zero-copy, memory-safe, untrusted input} any two are
obtainable. Zero-copy within owned memory is solved and safe. Zero-copy from external bytes
asserts unchecked type invariants; validation is O(n).

*The resolution, specific to this design.* **A hash-chained, append-only ledger written only
by the engine is a trusted writer, and the chain is the validation** — amortised per epoch
rather than per read. So the trust boundary is exactly where the hash chain ends: zero-copy
from the ledger is sound, zero-copy from a client's wire buffer is not, and the two are
distinguishable in the type system rather than by convention.

*What the validation is, since the argument depends on it.* **SHA-256 (FIPS 180-4)**, over the
canonical body, chained parent-first so the digest at any epoch commits to every epoch before
it (`nilestream_ledger::chain`, ADR 0003). This sentence used to be missing, and the review was
right that its absence mattered: the chain was a 256-bit FNV variant whose own doc comment said
it was not collision resistant, and a chain that is not collision resistant validates against
accident rather than against an adversary. The resolution above is only as strong as the
function underneath it, so the function is now named and its conformance is a test — the NIST
vectors, including the long message, plus a chunking-invariance check the segment writer
relies on.

*Acceptance test.* A read of *n* ledger rows MUST perform O(1) allocations, not O(n). A
decode from a client buffer MUST be rejected if the chain does not verify.
**Status: Specified.**

---

## Part II — Query semantics

### L-5 / L-23 Supersedes SQL

**Every SQL construct in the supported fragment MUST lower to the same IR as its Niles
equivalent, and MUST produce identical results.**

*The honest form of the claim.* SQL cannot be shown inadequate; it is relationally complete.
What can be shown is that a coherent type system spanning money, effects, consistency,
confidentiality and retention has not been built, is not obtainable by composing five
independent extensions because the analyses are mutually constraining, and that Niles is an
existence proof. **Replacement happens by supersession**, which is also the migration path:
an unported program keeps working and gains the new checks incrementally.

*The commitment that makes this safe.* A wire protocol is a **surface, not a semantics**.
There is no compatibility layer with its own execution path, because two ways to compute an
answer is two answers that can disagree.

*Acceptance test.* `crates/niles-lang/tests/end_to_end.rs` — a SQL surface query and its
pipeline equivalent MUST lower to structurally identical circuits. **Status: Built** for the
declared fragment; the fragment MUST be published rather than implied, and narrowing it is a
public act.

---

### L-8 / L-9 Algorithm and schedule, separated — with verified schedules

**A query MUST be expressible as an algorithm with no physical commitments. A schedule MAY
be attached, and when attached MUST be verified to preserve the algorithm's denotation.**

*The tension.* "Let the engine do more" and "more explicit control" are genuinely opposed
within one syntactic layer. That is why PostgreSQL has refused hints for twenty-five years,
and its six stated objections all reduce to *hints are unverified*.

*The resolution, and the contribution.* Halide (PLDI 2013) separates *what* from *how*.
Neumann and Leis reach for the same move in SaneQL; Brandon asks for it in "Against SQL."
**Nobody has made the schedule verified.** A schedule term carrying a proof that it preserves
denotation eliminates five of PostgreSQL's six objections, and Nilestream is unusually well
placed to build it because **the IR verifier is already in the trusted base** — this needs a
new judgement, not a new subsystem.

```
view balances = postings |> group_by(acct) |> sum(amt)
    schedule { join_order: [postings, accounts], materialize: demand }
    // ^ verified: the checker proves this schedule computes the same relation
```

*Acceptance test.* A schedule that changes the result MUST be rejected by the verifier, not
merely produce a different plan. A schedule that is merely slower MUST be accepted. The
negative control — a schedule that reorders a non-commutative operation — MUST fail.
**Status: Specified.** This is the single highest-value unbuilt item in this document.

---

### L-10 Set-at-a-time

**Query operators MUST be defined over relations, not tuples. There MUST be no row cursor
in the language.**

*Acceptance test.* The grammar MUST contain no cursor construct;
`crates/niles-lang/grammar/niles.ebnf` is normative. **Status: Built.**

---

### L-15 Nested subqueries and CTEs — and the unnesting that matters

**Subqueries and CTEs MUST be expressible, and correlated subqueries MUST be unnested where
a semantics-preserving rewrite exists.**

*The finding that reorders the roadmap.* Dreseler et al. (PVLDB 2020) measured what the
optimizer's parts are actually worth:

| Optimization | Measured value |
|---|---|
| **Subquery unnesting** | **~510× geomean** |
| Join ordering | ~7% |

Every instinct says the optimizer's job is join enumeration. The measurement says it is
unnesting, by two orders of magnitude. `join_order.rs` is built and correct; **the next
optimizer investment MUST be the unnesting rules.**

*Acceptance test.* A correlated `exists` subquery MUST lower to a semi-join, not to a nested
loop with a per-tuple subplan. A benchmark comparing the two forms MUST show the ratio.
**Status: Specified** — the parser accepts subqueries; the unnesting rules are unbuilt and
are now the top optimizer priority.

---

### L-19 Deductive and recursive

**Recursion MUST be expressible with a termination witness. Unguarded recursion MUST have no
spelling.**

*Rationale.* A fixpoint that could diverge stalls an epoch, and a stalled epoch stalls the
visibility timeline for every reader. Termination is therefore not a nicety; it is a
liveness property of the whole system.

```
q.fixpoint(step) guard measure(depth)
```

*Acceptance test.* `Op::Fixpoint { measure, max_rounds }` carries the measure in the IR, not
merely at parse time, so the runtime can stop. A program omitting the guard MUST NOT parse.
**Status: Built** (IR and grammar); WCOJ execution is **Adopt** and unbuilt.

*Calibration.* WCOJ is a **parity requirement for graph-shaped queries, not a
differentiator.** Freitag's own hybrid optimizer chose WCOJ **zero times out of 923 joins**
across TPC-H and JOB. The 4-clique gain is 4.5–6.9× against EmptyHeaded.

---

### L-11 / L-18 Arrays, ML, and no impedance mismatch

**Arrays MUST be a first-class type with the same type system as relations. Moving between a
relation and an array MUST NOT require serialisation.**

*Rationale.* The impedance mismatch is not syntactic; it is that a relation and an array
live in different memory representations and different type systems. A columnar relation
*is* an array of columns, so the mismatch is an artefact of the boundary rather than of the
domains.

*Acceptance test.* A feature-extraction query producing a matrix MUST NOT copy the underlying
column buffers. **Status: Specified.**

---

## Part III — Correctness and the thesis findings

### L-12 Isolation as an effect, not a language guarantee

**The language MUST type consistency rungs and MUST reject a computation that promises a rung
above its stalest input. The language MUST NOT claim to provide strict serializability.**

*The category error, corrected.* Strict serializability is Papadimitriou's serializability
plus Herlihy–Wing real-time order — a property of a protocol over a durable store. The
mechanisms that produce it *are* a runtime. What the language provides is the **discipline**;
the engine provides the **guarantee**.

This also corrects the thesis: Contribution 4's soundness theorem should state what it
proves — that a well-typed program cannot *request* an unsound combination.

*Acceptance test.* `NL0311` MUST fire when a view promising `ledger_consistent` reads a
`bounded` one, and the diagnostic MUST explain that a computation is no fresher than its
stalest input. Verified in `niles_schema.rs::a_rung_monotonicity_violation_is_rejected`, and
the counterfactual — deriving the availability decision from the cheaper view — is *run*,
not asserted. **Status: Built.**

---

### L-24 The thesis findings, as language obligations

The organising constraint. Each finding becomes a requirement the language MUST meet.

| Finding | Grade | Language obligation | Status |
|---|---|---|---|
| Anchored reconstruction (Thm 4.1) | Enabling | Every view MUST declare an anchor; a non-reconstructible input MUST be a compile error, not a cost | **Built** — `join_order.rs` prunes it as *legality* before costing |
| Honest absence | Enabling | A miss MUST NOT be expressible as zero. `Reading` has no numeric arm | **Built** — GBS's `gbs-mechanisms::signal` |
| Static conservation under control flow | Enabling | A `txn` MUST balance per currency on every path, decided before running | **Built** — 11 of 12 defect classes at compile time (E14); **0% undecided on a 40-function corpus** (E18) |
| Coordination-free cross-shard reads | Enabling | A read of a frozen prefix MUST require no coordination and MUST be cacheable without invalidation | **Built** — `nilestream-core::distributed` |
| Per-view rungs with monotonicity | Cumulative | L-12 | **Built** |
| Bitemporality | Cumulative | Two axes MUST be distinct types; a back-valued correction MUST be an append | **Built** |
| Capability-gated effects | Cumulative | A transition MUST name its capability in the effect row | **Built** |
| Lineage and audit | Cumulative | `explain`, `reproduce e at #4200`, `impact` MUST be language constructs | **Built** |
| Determinism (Appendix C.4) | Cumulative | No wall-clock read in a view predicate; no float in the money path; stable iteration order | **Built** — `IR013` |

---

### L-20 `comptime` — where the dynamism belongs

**Metaprogramming MUST occur at compile time and MUST produce statically-typed code.**

*Rationale, and the resolution of the original tension.* The previous requirement list asked
for runtime dynamism *and* static financial invariants. Those are opposed. The Terra/Zig
`comptime` model resolves it: put the dynamism in a compile-time meta-stage — which is where
"specialised code per query shape" (L-20's own demand) belongs anyway — and keep the
execution layer static and reclamation-free.

*Acceptance test.* A `comptime` block MUST NOT be able to observe runtime data, and MUST
produce code that type-checks under the ordinary rules. **Status: Specified.**

---

### L-21 The cost model as a testable discipline

**The engine MUST NOT do asymptotically more work than the query requires, and the counted
work MUST be observable.**

*Rationale.* "Respect the fundamental complexity" is untestable as prose and testable as a
counter. Nilestream already exposes counted work as a first-class measurement, which is what
makes a claim about complexity checkable rather than asserted.

*Acceptance test.* For a query over *n* rows with selectivity *s*, counted work MUST be
O(n) for a scan and O(s·n) for an indexed access, verified across three input sizes.
**Status: Built** — counted work is exposed; the asymptotic assertions are specified.

---

## Part IV — What this specification does not claim

Stated because a specification that named no limits would be marketing.

1. **It does not claim SQL is inadequate.** SQL is relationally complete. The claim is
   supersession by a type system nobody has built, not inadequacy.
2. **It does not claim an order of magnitude from SIMD or from compilation.** Both are
   parity requirements. The measured figures are 1.4× and a wash.
3. **It does not claim WCOJ helps relational workloads.** Zero of 923 joins.
4. **It does not claim to provide strict serializability.** That is the engine's, and
   §L-12 says so.
5. **It does not claim the schedule verifier exists.** It is specified, it is the highest-
   value unbuilt item here, and it is the one genuinely novel contribution in this document.
6. **It does not claim the bootstrap is a compiler.** Two front-end stages — a lexer and a
   parser written in Niles, each verified against the reference implementation, the parser
   node for node over its own 1,200-line source — are two rungs, not a ladder. The
   type-checker and the lowering pass are still Rust.

## Part V — Conformance summary

| Part | Requirements | Built | Partial | Specified | Adopt |
|---|---|---|---|---|---|
| Execution model | L-1, L-2, L-6, L-16, L-17, L-13, L-22 | 1 | 1 | 4 | 1 |
| Query semantics | L-5, L-8, L-9, L-10, L-11, L-15, L-18, L-19, L-23 | 3 | 1 | 4 | 1 |
| Correctness | L-3, L-4, L-7, L-12, L-14, L-20, L-21, L-24 | 6 | 1 | 1 | 0 |

**Ten of twenty-four built with passing tests. Three partial. Nine specified. Two adopt.**
The gap is the roadmap, and `ROADMAP.md` sequences it by measured value: unnesting first
(510×), then adaptive tiering, then the schedule verifier.
