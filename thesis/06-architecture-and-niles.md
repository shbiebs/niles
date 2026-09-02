# 6. System Architecture and the Niles Language

This chapter describes the instrument: Nilestream's architecture and the design of Niles. Normative references live in the appendices (B: syntax; C: compilation; D: engine and IR; E: self-hosting compiler; H: SQL-completeness dossier; I: optimizer algorithms). Implementation status is Chapter 7; nothing here should be read as a report of a completed system.

## 6.1 The Build Spine

The system is one spine with four segments, each consuming the previous segment's output and nothing else.

1. **The ledger write path** — admission, validation against the commit rule, idempotency, authorization, sequencing, epoch sealing, hash chaining, durability.
2. **The typed intermediate representation** — a DBSP-style circuit language with anchors, effects, contracts and provenance in its types, and the *stable contract between the language and the engine*.
3. **The read-model runtime (Nilestream-Core)** — REVs executing IR circuits with partial state, eviction, anchored upqueries, the materialization optimizer, and lineage.
4. **The serving edge** — the native Niles protocol plus MySQL and PostgreSQL wire compatibility.

The spine discipline is architectural law: the read side may consume only sealed epochs — never the open epoch, never another view's internals except through declared IR edges. This is what makes Chapter 4's proofs apply to the implementation as built rather than as idealized.

The IR's role deserves emphasis because it is a design commitment rather than a convenience. Putting a typed IR between surface and engine follows the progressive-lowering philosophy of modern compiler infrastructure, where high-level semantics are retained until an optimization no longer needs them and then lowered one level at a time [Lattner et al., CGO 2021]. Here the semantics that must survive lowering are exactly the ones the theorems quantify over: anchors, contracts, effects and provenance. An IR that erased them would leave the theorems talking about a different object than the one that runs.

## 6.2 Ledger Write Path and Read-Model Engine

**Write path.** Clients submit transactions natively or through the wire protocols. Admission checks the idempotency key against the declared deduplication window (Section 3.20), returning the recorded outcome on a matching repeat and a conflict error on a fingerprint mismatch. Validation type-checks against the schema and evaluates the commit rule — for a ledger, per-transaction per-currency balance in exact integer minor units, plus authorization capabilities for flagged effects. Sequencing folds validated transactions into the open epoch in one serial order. Sealing closes the epoch on the τ boundary or size bound, computes its hash, makes it durable, and publishes visibility. Everything after the seal is immutable.

**Read-model engine.** Each REV holds its resident map (keys at points of the absence lattice), a per-key anchor index, its compiled circuit with provenance-derived upquery paths, and its contract. Application advances resident keys incrementally through delta circuits. Reads follow the protocol of Section 3.13. Eviction and mode selection are the optimizer's (Section 6.16). Lineage is emitted at the view's declared mode.

**Why the two halves can be maximally different.** The write path is where coordination lives and is deliberately conservative; the read path is where economics live and is deliberately adaptive. This asymmetry is not a compromise between them but a consequence of Proposition 3.2: the invariants that require coordination are enforced at admission, and everything downstream inherits their truth without paying for it again.

## 6.3 Reasons to Build a Custom Runtime, and Where to Reuse

**Build:** the epoch and anchor machinery (exists nowhere with these semantics); partial state with versioned holes (existing partial state is unversioned and eventually consistent); the ledger commit path (databases hide their logs, whereas here the log is the product); the IR (the proofs need its semantics pinned); the materialization optimizer (no existing optimizer decides under a consistency contract).

**Reuse:** established consensus implementations for the replicated profile rather than a bespoke protocol; standard cryptographic primitives, never hand-rolled; wire-protocol framing; an LSM library for cold view state (though *not* for the ledger itself, whose compaction would rewrite data the design declares immutable); and DBSP's operator algebra as the formal and, where license-compatible, practical basis of the IR executor.

The rule: reuse where semantics are commodity, build where the thesis's claims live.

## 6.4 Durability and the Migration Boundary

An epoch is durable when its segment and hash are synchronized to the primary medium and, in replicated profiles, acknowledged by a quorum; visibility never exceeds durability. Storage is tiered — hot segments on fast local media, sealed history migrating to cold object storage — and the **migration boundary is semantic, not merely operational**: bytes may move media, but identity is the hash chain, so a migrated prefix re-verifies on read and the audit guarantee survives storage evolution. This matters commercially as well as formally: immutable ledgers grow without bound, and published industrial experience is that ledger growth drives cold-storage offload as a primary cost concern.

Recovery replays from the last durable epoch: frontiers are re-derived, resident maps restart empty (checkpoints are pure optimization, never trusted), and Theorem 4.1 guarantees rebuilt views are exact. **Recovery is a large eviction** — which is why the same property tests cover both.

## 6.5 The General Core of Niles, and Coverage of Workload Classes

Niles is a general-purpose data language whose core is typed relations, expressions, functions and views over the relational algebra, with Rust-derived syntax for items and expressions and SQL vocabulary for query stages where Rust has no equivalent. A schema declares three kinds of object:

```niles
schema bank {
    table customers { id: Id<Customer>, name: Text, opened: Date }

    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        value_date: Date, idem: IdemKey,
        conserve per (txn, cur);
        retain forever;
    }

    view balances = postings
        .group_by(|p| (p.acct, p.cur))
        .sum(|p| p.amt)
        serve { consistency: bounded(2.epochs, 5.s), materialize: demand, retain: evictable };
}
```

`table` is ordinary relational data; `ledger` is the authoritative append-only base with a commit rule; `view` is a REV with a contract. Everything else — modules, generics, pattern matching, `let`, `fn` — is Rust-shaped.

**Workload classes on one core.** The generality claim (H-S8, C6) is discharged by construction, and the constructions are these:

| Class | Construction |
|---|---|
| **OLTP** | Committed writes to the base plus point-lookup REVs at ℓ₂–ℓ₅. |
| **OLAP / HTAP** | REVs whose circuits aggregate; the optimizer chooses column-shaped resident representations and `full` or `tiered` modes for wide scans. The row/column duality that HTAP systems obtain by dual formats over a mutable table is obtained here by multiple REVs over one immutable base — the same idea with a stronger correctness story. |
| **Time-series** | A REV over a base partitioned on the valid-time axis, with compressed resident representations; the published delta-of-delta and XOR schemes are the reference design for that compression. |
| **Document** | A typed semi-structured column with path-indexed resident state, following the decomposed-binary and tree-of-paths designs that production document stores use. |
| **Graph** | Guarded recursion over an edge base — the same fixpoint construct that gives Theorem 4.6(b) — with pattern syntax aligned to the standardized property-graph vocabulary. |
| **Search** | An inverted-index REV whose resident state is immutable segments with tombstones and background merging; that this architecture serves search is not a hypothesis but two decades of production evidence. |

The point is not that one storage format serves everything — it does not — but that one *semantics* does. Format, layout and residency are the optimizer's choices under Theorem 4.5(a), which proves them semantics-preserving.

## 6.6 How Banking Was Made First-Class Without Making It Special

Banking is a library over the general core, not a fork of it. A standard module `std::bank` defines accounts, postings, transactions, `Money⟨cur, scale⟩`, holds, authorization capabilities and the double-entry discipline, all expressed with type-system features available to any user library: currency-indexed types, linear halves, effect rows, and commit rules.

The design intent is that the compiler knows nothing about money as such; that it knows about indexed monoid rows, linear consumption and declared commit rules, and that `std::bank` instantiates them.

**That is not what is implemented, and the sentence that used to stand here said it was.** Seven banking forms are **keywords in the compiler's own registry** — `txn`, `hold`, `resolve`, `post`, `fx`, `conserve`, `idem` — each with a variant in `Expr` and a case in the parser, the lowering and the effect calculus (`niles-lang/src/keywords.rs`, `ast.rs`). `Money` is a type constructor the typechecker knows by name, and the currency-row solver is a pass, not a library. A user library could not add any of them. What is general is the *machinery* underneath — indexed rows, linear types, effect rows, commit rules — and what is banking-specific is the surface syntax over it, which is a weaker and quite different claim.

**And the falsifier has not been run.** Section 9.11's non-financial conservation domain — inventory with serial-number conservation — does not exist: `grep -ri inventory` over the crates, the schemas and the examples returns nothing. Until it does, the generality of the core is a design argument rather than a tested one, and H-S8's status line says so. See §11.3 for the falsifier as it stands.

## 6.7 Balance as a Function of Time

There is no stored balance anywhere. `balance(acct, cur)` is a temporal signal — a function from epoch to `Money⟨cur⟩` — defined as the running sum of the postings signal; available balance subtracts unresolved holds. Reads of "the balance" are reads of this signal at an anchor; statements are integrals over valid-time windows; interest accrual is an operator on the signal.

This one decision eliminates the classic dual-write bug — a stored balance drifting from its postings — by making it *inexpressible*, and it is what lets Corollary 4.1.1 apply to the thing institutions actually serve. It is also what makes the APSN hazard analyzable: the available and ledger balances are two REVs over one base, differing only in which rows their circuits admit, so their divergence is a property of contracts and anchors rather than an emergent mystery.

## 6.8 The Temporal Signal Library

`std::temporal` generalizes the above: signals (epoch-indexed values), windows, `as_of(epoch)`, valid-time selection and intervals, `latest`, accrual and decay combinators, and bitemporal join forms. All are **epoch-deterministic**: no wall-clock read occurs inside query evaluation, and "now" enters only as the anchor chosen by the contract. That restriction is what keeps reconstruction pure and cross-target execution deterministic, and it is enforced by the effect checker rather than by convention.

## 6.9 Three Surfaces, One Intermediate Representation

Niles native, the SQL surface, and the imperative Rust-shaped sublanguage all lower into one typed IR. The equivalence obligation is stated as a theorem and tested as a build gate: an SQL query in the stated fragment and its Niles counterpart *denote the same Z-set on every finite instance* (Theorem 4.6(c), Appendix H). Circuit-level α-equivalence is not claimed — the corpus compares answers, and one query may have two honest lowerings. The IR, not any surface, is what the theorems and the runtime interpret.

Embedded APIs — language bindings that construct IR directly — are a fourth surface with the same status: they are checked by the IR verifier on submission, so an embedding cannot smuggle in a circuit the type system would reject.

## 6.10 The Language-Creation Gate

Creating a language is expensive, and the thesis states the gate it had to pass. A new language is justified only if: **(G1)** the required semantics cannot be expressed as a library in an existing language; **(G2)** they cannot be added to SQL without semantic dishonesty; and **(G3)** the type-level guarantees are load-bearing for the scientific claims.

**G1.** Linear posting halves, effect rows for consistency, and compile-time currency rows require machinery a general-purpose host language's trait system can partially emulate but not enforce completely or ergonomically — affine approximation instead of linearity, no effect system, and diagnostics that surface as solver artifacts rather than domain errors.

**G2.** SQL has no effects, no linearity and no anchors; dialect pragmas would be unchecked comments. This is where the strongest opposing position must be met rather than avoided. "One SQL to Rule Them All" argues that streaming needs only three additions to standard SQL — time-varying relations as the foundational abstraction, event-time semantics with watermarks, and a small set of keywords controlling materialization of results [Begoli et al., SIGMOD '19]. The argument is good, and this thesis concedes the part that is right: for *streaming semantics alone*, SQL plus those three extensions is sufficient, and Niles's query core is deliberately isomorphic to that position. The disagreement is about the other axis. Conservation, currency safety, idempotency windows, retention, confidentiality and per-view consistency are *type* obligations, not query semantics, and there is no way to add a type-and-effect discipline to SQL's grammar without changing what a well-formed program is — at which point one has a new language and should say so. The honest formulation of the gate is therefore: SQL can be extended to say *what to compute over streams*; it cannot be extended to *refuse programs that would lose money*.

**G3.** Theorem 4.4 quantifies over well-typed programs. The type system is the theorem's premise, not its packaging.

### 6.10.1 The Gate, Re-Assessed Against a Built Compiler

The three conditions above were argued before any of the language existed. A gate assessed only in prose is a gate the author sets the height of, so this section re-assesses each one against the stage-0 compiler in `crates/niles-lang`, and records where the evidence is weaker than the argument.

**G1 — could this be a library?** The argument was that a host language's trait system can *approximate* linear posting halves and currency rows but cannot enforce them ergonomically. The compiler makes the comparison concrete, and the honest answer is split.

The parts that a library **could** carry, in Rust specifically: currency-parameterised money types, an affine approximation of linearity through move semantics and a `#[must_use]` drop guard, and a per-currency scale in a const generic. A determined library author gets most of the way there.

The parts a library **cannot** carry, and this is where the gate holds:

* **Rung monotonicity.** The judgement that rejected `available_balance` (§9.13.4) is a property of a *view definition's whole dependency graph*, not of any value in it. There is no type a library can attach to a value that says "the freshest guarantee obtainable from every transitive input of the expression this value came from". It needs the compiler to have the view graph, which means it needs views to be a language construct.
* **The `undecided` verdict.** The currency-row solver distinguishes "provably conserving", "provably not", and "the checker cannot see the amount". The third is what makes the first two trustworthy, and it requires whole-program symbolic accounting across a `txn` boundary. A trait bound has no way to be *partially* satisfied and report the residue.
* **The two-span diagnostic.** Every interesting error in §9.13.4 points at two places: the money that was created and the rule that forbids it; the view's promise and the read that breaks it. In a library encoding these surface as trait-solver artifacts naming synthetic types, which is precisely the ergonomic failure the original argument predicted, and which the built diagnostics avoid.

**G1 holds, but on narrower ground than originally claimed.** The load-bearing part is not money typing — a library can nearly do that — it is the *whole-program, graph-level* judgements: rung monotonicity, conservation across a transaction, and reconstructibility. Those need a compiler that owns the view graph.

**G2 — could this be SQL?** The concession stands and is now sharper. The Begoli *et al.* position — that streaming needs only time-varying relations, event-time semantics and a small materialization vocabulary added to standard SQL — is right about *query semantics*, and Niles's query core is deliberately isomorphic to it. Building the SQL surface in `sql_surface.rs` and testing that both surfaces lower to the same circuit (§9.13, `the_two_surfaces_lower_to_the_same_circuit`) is that concession made mechanical rather than rhetorical: for the stated fragment, the SQL spelling and the pipeline spelling are the same program.

What no SQL extension reaches is the four judgements the compiler actually performs. `conserve per (txn, cur)` is not a constraint on a row, it is a typing rule over a transaction. `! { append, debit<usd> }` is not a comment, it is a subsumption check. Linearity for holds is not a trigger, it is a use-count. Rung monotonicity is not a hint, it is a graph property. Adding any one of them to SQL changes what a well-formed program *is*, and at that point one has a new language and should say so.

**The formulation the built artifact supports:** SQL can be extended to say what to compute over streams. It cannot be extended to *refuse programs that would lose money*, because refusing is a type-system act and SQL has no type system in the sense required.

**G3 — is the type system load-bearing for the science?** This is the condition with the strongest new evidence and it is not the evidence expected. Theorem 4.4 quantifies over well-typed programs, so the type system is formally the theorem's premise; that was always true and was always a slightly circular defence. What the built compiler adds is empirical: **the type system caught four defects in this thesis's own worked program**, one of which — the rung-monotonicity violation in `available_balance` — is the exact failure mode Chapter 1 motivates the whole thesis with, written into the example by the person who formulated the rule.

That is a fact about the instrument's value that no amount of prose could have established, and it cuts both ways. It is strong evidence that the checks are load-bearing. It is also evidence that the *author* of a consistency calculus will violate it in an eighty-line example, which is the strongest argument available that a bank's engineers will violate it in a hundred-thousand-line codebase, and that a compiler rather than a review process is what should catch them.

**Verdict.** All three conditions pass, with G1 narrowed and G3 strengthened. The narrowing matters and is stated because a gate that always passes is not a gate: the honest position is that a substantial part of what Niles offers — money types, scale safety, an affine approximation of linearity — *could* be a Rust library, and that the case for a language rests specifically on the whole-program graph judgements and on the diagnostic quality that follows from owning the syntax.

**Why not fork an existing compiler.** A fork inherits the full maintenance surface of a general-purpose toolchain while fighting it on linearity-versus-affinity and on effects, and the declarative query core is un-Rust-like at the surface in any case. Niles instead *borrows Rust's syntax shape* — familiarity without the fork — with a fresh, small, self-hosting implementation (Appendix E).

**Precedent for the decision.** In 2024 ISO published GQL as a standalone database language rather than as another SQL part, developed by the same working group that maintains SQL, while *also* standardizing SQL/PGQ as a read-only embedded sublanguage over the same graph patterns. The standards community thus treats "embedded sublanguage" and "standalone language" as genuinely different artifacts and has recently chosen the latter when the model differs enough. That is not proof that Niles is justified, but it is evidence that the question this gate asks is the right one.


### 6.10.2 Diagnostics as a Design Obligation, and the Evidence for It

The gate argument above claims that a library encoding degrades the *diagnostics*, and that
this is part of why the language is warranted. That claim needs its own evidence, because it
is the kind that is easy to assert and rarely checked.

**The theoretical case is strong and it is not about spans.** Haack and Wells's position on
type errors is that the location of an error is not a point but "a set of program points (a
slice) all of which are necessary for the type error", and that algorithms which "identify
one node of the program tree which participates in the type error … will often be the wrong
node to blame". Zhang and Myers reach the same place through constraint analysis; Chen and
Erwig note that committing to a single location fails "because in some cases the program
text does not contain enough information to confidently make the right decision".

That argument transfers to money safety exactly. A conservation violation is constituted by
the postings that fail to net *and* the rule that says they must. A double resolution is
constituted by the binding and both consumptions. A rung violation is constituted by the
contract, the read, and the view in between. Reporting one of those is reporting an
arbitrary member of a set, and the choice is a heuristic rather than a fact about the
program. This is a soundness-of-blame argument, and it needs no human-subjects result.

**The empirical case is weaker than this thesis previously implied, and the correction is
worth making explicitly.** No controlled study compares multi-span against single-span
diagnostics, for any error class, in any language. The relevant results are these:

* Barik et al.'s eye-tracking study (56 participants, defects derived from an analysis of 26
  million builds) found that developers *do* read error messages — 13–25% of task time — and
  that reading them is as effortful per fixation as reading source code (419 ms against
  394 ms, versus ~275 ms for silent English reading). It also found, in one task, that two
  identical messages for two subclasses led 55 of 56 participants to the wrong fix when the
  correct change was in the parent. That is a misattribution-of-blame result, which is
  suggestive for this design and is not evidence about span count.
* Barik, Ford, Murphy-Hill and Parnin's later work models a message as a Toulmin argument —
  claim, grounds, **warrant**, backing — and finds, with 68 professional developers, that
  developers prefer proper argument structure *when neither message offers a resolution*,
  **but will accept a deficient structure if it provides a resolution**. The `conserve per
  (txn, cur)` rule is the warrant, and pointing at its declaration is the backing; but the
  second half of that sentence is a constraint on the design, not a footnote.
* Denny, Luxton-Reilly and Carpenter found **no effect** from enhanced messages on any of
  three measures (83 CS1 students, randomised). The result stands for what it tested —
  syntax errors, novices, submission-count proxies — and the authors note a confound running
  against the enhancement: their enhanced condition displayed *one* error where the control
  displayed two. It should be cited, not dismissed.
* And the compiler most often held up as exemplary for humane errors, Elm, is largely
  **single-region**, with the counterparty expressed in prose.

**So the honest claim is convergence on dual *reference*, not on dual *spans*.** rustc's
`note: required by this bound in …` and `note: the lint level is defined here`, GCC's
labelled ranges — introduced to make mismatches clear "without requiring users to
cross-reference distant code locations" — and the Language Server Protocol's first-class
`relatedInformation` field all point at the other place; whether that is a span, a note or
prose is an open rendering question.

**Two design rules follow, and both are concessions rather than wins.**

*The primary label must stand alone.* rustc's own guidance requires that a primary label
make sense "if it were the only thing being displayed", because in an IDE it often is. If a
Niles money-safety diagnostic is unintelligible without its second span, the primary label is
underspecified — so `net movement on every path through this transaction is −40.00, which
must be zero` carries the whole claim, and the rule reference is additive.

*A fix outranks a warrant.* Following Barik et al.'s second clause, the rule reference is
attached only when the diagnostic has no machine-applicable suggestion, and is elided when it
has one, with the rule's identity remaining in the message text either way. This is
implemented — `Diagnostic::warrant` — rather than described, and the number of elisions is
observable so that the rule cannot silently stop firing.

**The precedent for the specific error class is rustc's borrow checker**, which is the
industrial system closest to this one. A use-after-move (E0382) is reported with *seven*
labelled locations across three windows, whose core is precisely the shape Niles uses for a
doubly-resolved hold: *bound here*, *first consumed here*, *consumed again here*, with the
caret on the second use rather than the first. Conflicting-borrow errors (E0499, E0502) use
three. This thesis follows that layout because it is the only place the pattern has been
exercised at scale, not because a study has validated it.

**What is not claimed.** That developers attend to the second span; no eye-tracking study has
looked inside a diagnostic. That message enhancement improves outcomes; the record there is
mixed to null. That there is literature on diagnostics for linear or effect type systems;
there is none, and that gap is a research opportunity rather than a citation. The experiment
that would settle it is small and well-scoped — the same violation rendered three ways,
measured on blame-attribution accuracy and time-to-correct-fix — and §12 records it as future
work rather than pretending it has been run.

### 6.10.3 The Counterproposal, and Where It Leaves Both Gates

The gates above ask whether Niles *could* be a library or an SQL extension. A sharper
question is the one a committee will actually ask: **why not simply add reconstructible
epoch-anchored views to PostgreSQL, and keep SQL?** That is the minimal counterproposal, and
because it is the strongest objection available it was built and measured rather than
argued. Experiment E14 is the result; the harness is `crates/counterproposal/run.sh` and it
runs against PostgreSQL 16.13.

**The mechanism survives the counterproposal intact, and this thesis reports that first
because it goes against its own emphasis.** A good-faith PostgreSQL schema — money as
`numeric`, immutability by trigger, idempotency by unique index, conservation as a deferred
constraint trigger, reconstruction as a `STABLE` function folding the suffix after the newest
checkpoint, and the REV itself as a table with a `state ∈ {present, hole, pending}` column —
reproduces every mechanical property: zero divergences across fifty keys under continuous
eviction, honest absence with fifty versions kept and zero values kept, checkpoint-bounded
reconstruction at five buffer hits, and maintenance proportional to deltas rather than to
view size. **Reconstructible epoch-anchored views can be built in PostgreSQL today.** Anyone
who wants the mechanism can have it without adopting anything from this thesis, and the
engine contribution must be argued on other grounds.

Two of those grounds survive. PostgreSQL's own `REFRESH MATERIALIZED VIEW` is wholesale and
takes no key, so a REV in PostgreSQL is *application code the database does not verify*, and
every team writes it again. And PostgreSQL isolation is a property of a transaction rather
than of a view, so "this balance may be four epochs stale and that one may not" has no SQL
spelling at all — which is why the consistency ladder cannot be expressed, let alone checked.

**The defect corpus is where the case inverts.** Twelve defect classes, written twice, scored
by the stage at which each is caught:

| Stage | PostgreSQL | Niles |
|---|---:|---:|
| Compile time | 0 | 11 (+1 warning) |
| Runtime | 3 | — |
| Never caught | 9 | — |
| Not expressible | 1 | 1 |

PostgreSQL wins one comparison outright: a mixed-currency transaction moving 100 USD to 100
EUR is caught at COMMIT, because the deferred trigger groups by `(txn, cur)` and both groups
are non-zero. That is a complete and correct detection.

The nine it never catches are the ones without an SQL spelling to check against: adding USD
to EUR in a query; `100.50 jpy` where JPY has scale zero, which PostgreSQL stores as
`-100.5000` without complaint; an authorization derived from a bounded-stale view; a
materialized view whose predicate reads `now()`; a filter on a column that should be
encrypted; an unauthorized overdraft; a missing anchor index. These are not gaps PostgreSQL
could close with more triggers, because a trigger is a runtime object.

**Which the tenth defect demonstrates.** One statement —

```sql
drop trigger postings_conserve on postings;
```

— followed by a single one-legged insert, and the ledger's control total reads −500.00 with
nothing left in the database that will ever say so. This is not a criticism of PostgreSQL; a
trigger is supposed to be droppable, and a migration, a `pg_restore`, or a replication tool
that omits triggers will drop it without anyone deciding to. It is an observation about
**where the invariant lives**. In the SQL version conservation is a runtime object that can
be removed from a running system; in Niles it is a property of the program text, and a
program without a balancing posting has no executable form from which to remove the check.

**What E14 settles, and what it does not.** It settles that the engine case is the weaker of
the two and the language case the stronger — the opposite of where this thesis spends its
pages, and §12 records the rebalancing as work the document still needs. It does not settle
that Niles should exist, and three premises are missing before it could:

1. **Frequency.** The table shows these defects are undetectable, not that they are common.
   A defect class nobody writes costs nothing to miss. Establishing frequency needs a corpus
   of real banking code or an incident study, and this thesis has neither.
2. **Cost.** Against nine avoided defect classes stands training, tooling, hiring, the
   reserved-word collisions of §9.13.5, and the risk that the compiler is itself wrong. E14
   measures only the benefit column.
3. **Human effect.** Whether a compile-time rejection prevents an incident that a runtime
   exception would also have prevented is a question about developers, not compilers.

The verdict this thesis therefore reaches is **argued, with the argument's missing premises
named** — which is a weaker claim than "proved", and the right one.

### 6.10.4 The Second Gate: Should Nilestream Exist?

The language-creation gate has an engine counterpart, and this thesis previously did not
state it. Applying the same three conditions:

**(E1) Can the required semantics be obtained from an existing engine?** Partly, and more
than expected. E14 shows the REV mechanism running in PostgreSQL with the right asymptotics.
What cannot be obtained is per-view consistency — isolation is per-transaction — and a
stable, user-visible read anchor, since neither `xmin` nor an LSN is one. E1 **fails as
stated** and survives only in the narrower form: *some* required semantics are unavailable,
not all.

**(E2) Can they be added without semantic dishonesty?** Per-view consistency cannot be added
to an engine whose isolation is a transaction property without changing what a transaction
means. A read anchor could be exposed. Partial materialization could be added to materialized
views. So E2 is **partly satisfiable**: a determined PostgreSQL fork could reach much of this,
and the honest position is that Nilestream's engine contribution is *cumulative* rather than
*enabling*.

**(E3) Is the engine load-bearing for the scientific claims?** This is where it holds, and
for one reason: the theorems quantify over a **typed IR with anchors, contracts and
provenance on every node**, and the accessed-field discipline that makes an engine fail
loudly when it ignores one of them. A conventional engine has no such object, so the proofs
would be about something other than what runs. The instrument is load-bearing even where the
mechanism is not.

**Verdict on Nilestream: weaker than the verdict on Niles, and the thesis should say so.**
The engine is justified as the *instrument that makes the theory testable* and as an
integration of mechanisms that are individually available elsewhere — not as a set of
capabilities no existing system could reach. Chapter 11's risk register gains this as a
named risk, and Chapter 12 records rebalancing the document's emphasis as outstanding work.

## 6.11 Join Ordering Under Partial State

Join ordering is the oldest solved problem in query optimisation and it is not the problem this system has. Selinger's dynamic program, and its modern form as Moerkotte and Neumann's `DPccp`, minimises one quantity: the number of intermediate tuples a plan produces. That is the right objective for a batch engine, where a join is a *transient* — it consumes its inputs, emits its output and holds nothing afterwards.

In a dataflow engine a join is not transient. It is a standing operator with two indexes, and those indexes are resident for as long as the view exists. The number that matters is therefore not how many tuples pass through but how many are still in memory when nothing is passing through at all, and a plan that produces fewer intermediate tuples while keeping a larger index is worse. Partial state adds a third quantity neither of the first two captures: when a REV evicts a key and later reads it, reconstruction walks *up* the operator graph to the nearest reconstructible ancestor, and the shape of the join tree is the shape of that walk. A deep left-deep tree makes reconstruction deep; a bushy tree makes it shallow but multiplies the internal indexes. The objective is thus three-term:

$$\mathrm{cost}(P) \;=\; w_{\text{flow}} \sum_{v \in P} |v| \;+\; w_{\text{state}} \sum_{v \in P} \mathrm{resident}(v) \;+\; w_{\text{recon}} \cdot \Pr[\text{miss}] \cdot \mathbb{E}[\text{upquery work}]$$

The three weights are not universal constants; they are read off the view's serve contract. A view at `ledger_consistent` under a tight freshness bound pays for state to avoid reconstruction latency; a cold analytical view at `bounded` pays for reconstruction to avoid state. That is Contribution 3's claim — *the price is workload-shaped, not history-shaped* — appearing as a planner input rather than as a theorem, and it is why the phase diagram of §9.13 has regions at all: were one plan optimal everywhere in the weight space, there would be nothing to characterise.

**Reconstructibility is a constraint, not a cost, and this has no analogue in a classical optimiser.** Some orderings are not expensive but *illegal*, because they place a non-reconstructible operator on the upquery path of a partial node — a source that is a mutable table rather than an immutable base, or a windowing node whose input has been discarded. Reconstruction from a frozen prefix is what makes anchored upqueries sound (Theorem 4.1); an operator that cannot be replayed breaks the chain. Such candidates are pruned *before* costing, so a bad cardinality estimate can make a plan slow but cannot make it wrong. The same discipline governs the materialization optimizer, for the same reason: the optimizer is forbidden from being a correctness dependency.

The implementation (`nilestream-optimizer::join_order`, 16 tests) runs `DPccp` for at most twelve relations and a greedy minimum-selectivity fallback above that, recording in the plan *which* ran, so that a measured regression can be attributed. Cardinality estimation uses the standard containment and independence assumptions and is wrong in the standard ways — correlated banking columns such as `account` and `currency` violate independence systematically — and a missing statistic falls back visibly rather than to a selectivity of one, because a cross product that looks free is how an estimator error hides. Cross products are enumerated only when the query graph is genuinely disconnected: a cross product in a standing dataflow operator is a resident quadratic index, which is a different order of mistake from a transient one.

## 6.12 Transport Security, and Where Cryptography Stops Being This Thesis's Problem

Two negotiation protocols are implemented, together with the policy layer that decides whether a cleartext connection may proceed. The cryptography is **not** implemented and is delegated behind a provider trait.

That division is a design position rather than an omission, and the thesis states it as one. Writing a TLS stack is the canonical example of work that must not be done from scratch: the failure mode is silent, the attacker is adaptive, and the defects that matter — padding oracles, timing leaks in MAC verification, state-machine confusion permitting a handshake step to be skipped — are precisely the ones a functional test suite passes. A thesis whose contribution is the static checkability of financial invariants has nothing to gain and a great deal of credibility to lose by shipping its own record layer.

The commitment that *is* load-bearing is what happens when the provider is absent. The obvious behaviour — accept cleartext, log a warning — is how a database comes to serve a ledger over a plaintext socket because a certificate expired overnight and something helpfully degraded. Here a `Require` policy with no provider **fails at startup**, before the listener binds, and refuses every connection independently at connection time in case the provider stops working later. This is the absence-lattice principle applied to a socket: an absence is reported honestly rather than papered over.

**The two protocols are not equally well designed, and the asymmetry belongs in the text rather than in a footnote.** PostgreSQL's client sends an eight-byte `SSLRequest` *before* the startup packet and the server answers with a single byte; nothing has been exchanged, so a refusal leaks nothing, and the user name and database travel inside the tunnel. MySQL's server sends its greeting in cleartext first, and the client signals upgrade by setting a capability flag in a *truncated* handshake response. Two consequences follow. The banner and auth-plugin name are on the wire in the clear regardless — nothing a server can do; it is in the protocol. And the upgrade is **client-asserted**, so a server that merely honours the flag lets any client opt out of encryption by not setting it. Enforcement must be a server-side check after reading the truncated response, and this is a real historical vulnerability class rather than a hypothetical.

One further honesty obligation is discharged in the types. `libpq` has six `sslmode` values and only two of them authenticate the server: `require` encrypts to a peer it has not identified, which defeats passive interception and nothing else. The implementation carries a predicate recording which modes actually authenticate, and the audit line reads *"encrypted, server identity unverified by the client"* rather than "TLS enabled" — deliberately awkward, because it is the true statement and the comfortable phrasing is not.

## 6.13 Memory Model, Allocation, Immutability, and Memory Safety

The runtime's memory model mirrors the formal one. Sealed data is immutable: epoch segments are frozen buffers, and resident view entries are immutable values *replaced* rather than mutated on anchor advance, with structural sharing. Allocation is arena-per-epoch on the write path — an epoch's allocations free as a unit after sealing and migration — and slab-based for resident maps under the optimizer's budget.

Ownership types are the enforcement mechanism: no mutable reference to sealed data exists in the type surface, so the two mutable structures (admission queue, resident maps) are the entire audited concurrency surface. At the Niles level, `ledger` values are immutable by type, bindings are immutable by default with explicit opt-in, and user functions receive borrowed immutable views. Memory safety for user code comes from the sandbox of the UDF tier, not from trusting user code.

## 6.14 Language Scope

Niles is a data language with an imperative sublanguage, not a systems language. Three tiers:

- **Declarative tier** — schemas, bases, views, contracts. Total, optimizable, PTIME by construction (guarded recursion only).
- **Transaction tier** — imperative Rust-shaped logic with effects, where the commit rules and the banking discipline bind.
- **UDF tier** — Rust-shaped functions compiled to WebAssembly for portability and sandboxing.

Explicitly out of scope: raw pointers, unrestricted I/O in query context, ambient wall-clock reads, and unbounded recursion in views. Application code, servers and user interfaces are out of scope by design; Niles connects to host languages through the wire protocols and embedded APIs. Section 11.2 lists "seductive generality" — the temptation to grow Niles into an application language — as a named risk, and these tiers as the mitigation.

## 6.15 The Computation Tier and the Determinism Obligation

The computation tier executes IR circuits: a scheduler assigns operator work per sealed epoch, hot circuits run fused and compiled, cold circuits interpret, and upqueries execute the same circuits in pull mode along provenance-derived paths. The compilation model follows the data-centric, push-based approach in which operator boundaries disappear in generated code and tuples stay in registers between pipeline breakers [Neumann, PVLDB 2011].

**Determinism rule:** given (prefix, circuit, anchor), output bytes are identical across runs and targets. This is not a nicety — Theorem 4.1's exactness and the hash-chain's reproducibility both depend on it.

Achieving it requires closing sources of non-determinism explicitly, and one of them is a correction to naive assumptions about the UDF sandbox. WebAssembly is *not* fully deterministic: its specification names three sources of implementation-dependent behaviour — NaN payloads (canonical NaNs carry a non-deterministic sign bit, and non-canonical inputs yield non-deterministic outputs), resource exhaustion (engines may fail to grow linear memory), and host functions [Haas et al., PLDI '17]. A UDF that produces a NaN, or that exhausts memory at a device-dependent point, would break reconstruction-equivalence over a hash-chained base. The obligation is therefore stated as three concrete mitigations rather than as an assumption: ban NaN-producing floating-point operations in UDFs or canonicalize at the kernel boundary; make resource exhaustion a *deterministic* abort at a fixed fuel bound rather than a device-dependent one; and permit no host functions outside a deterministic allowlist. Within the engine itself, determinism is enforced by canonical ordering before emission, integer-exact money, and prohibition of hash-order iteration.

## 6.16 The Kernel Boundary

The trusted kernel is small: admission, validation and sealing; hash chaining; storage; frontier bookkeeping; and the IR executor's core operators. Outside it: UDFs (sandboxed, resource-metered, with no ambient authority — capabilities are passed explicitly, following the capability-based model of the Wasm system interface); surface compilers, whose output is re-checked by the IR verifier, so the kernel trusts the *verifier* rather than the compiler; the materialization optimizer, whose decisions are semantics-preserving by Theorem 4.5(a) and therefore cannot be a correctness dependency; and wire adapters, which translate and never touch state directly.

The soundness theorem's trusted computing base is thus the kernel plus the IR verifier — stated explicitly so that the H-S4 campaign attacks the right boundary, and so that adding a domain library or an optimizer heuristic provably does not enlarge it.

## 6.17 Lineage and Traceability

Full traceability of operations, changes and states is a system requirement (Section 1.3(4)) and is implemented as an ordinary consequence of the algebra rather than as a logging subsystem.

Three modes are declarable per view: `off` (anchors only — every answer still carries the epoch it is correct as of); `key` (which base keys contributed); and `full` (the how-provenance polynomial retained). Provenance rides the same circuits as values, because Z-sets are a semiring instance and the factorization theorem guarantees that computing the most general annotation once permits every other semantics to be recovered by homomorphism [Green et al., PODS '07].

Three operational capabilities follow. **Explain**: for any served answer, return the base rows and the derivation that produced it. **Reproduce**: recompute any published (answer, epoch) from the prefix and compare byte-for-byte — the mechanical form of an audit. **Impact**: given a base row, identify the derived entries whose provenance includes it, which is what makes correction workflows (a reversing entry) analyzable rather than hopeful. Costs are measured under H-S9, with the published ~30% figure for interactive dataflow lineage as the order-of-magnitude reference point rather than a target.

## 6.18 Checkpoints as a Declared View Property

Theorem 3.7 makes the per-key checkpoint interval C the constant in the reconstruction bound, and §9.4.1 measures it: at C = 16 reconstruction cost was flat at ≈ 8.5 base rows across a 64× increase in history, against a predicted C/2 + 1 = 9. C is therefore a *contract term*, declared per view alongside consistency and materialization mode, and not a hidden engine default — because choosing it is choosing a point on the reconstruction-cost/checkpoint-storage trade, and the theory prices that choice.

Checkpoints are derived state: each is recomputable from the base, so they are evictable and rebuildable and stand outside the retention guarantee, exactly as views do. This is the base/derived split of H-F4 applied one level down, to the reconstruction path itself.

## 6.19 The Materialization Optimizer in the Runtime

Contribution 5's algorithm lives here. Per view and key range, the optimizer maintains estimates of arrival rate, reuse distance, reconstruction cost, reconstruction *latency* and update rate; computes the rent-or-buy break-even for mode selection and a cost-and-size-aware credit for eviction; weights both by the contract multiplier Φ(ℓ) and by the delayed-hit factor; and moves ranges between modes on a hysteresis schedule.

Three properties make this safe to run continuously. It cannot change what an answer *is* (Theorem 4.5(a)). It cannot violate a contract, because contract satisfaction is a constraint of the assignment problem rather than an objective term. And it is *observable*: every mode transition is logged with the estimates that caused it, so a surprising decision is diagnosable rather than mysterious. Appendix I gives the algorithm, the estimators, and the offline dynamic program used to compute the optimum against which H-S7 measures it.

## 6.20 End-to-End Encryption: Threat Model and Requirements

**Requirement.** Fields marked confidential must be protected such that a compromised operator — an honest-but-curious server, stolen media, subpoenaed backups — cannot read them, while the base's ordering, hashing and conservation guarantees still hold.

**Threat model.** The adversary controls storage and can read server memory, except for designated key-holding clients or hardware modules; the adversary cannot break standard cryptography. Availability attacks and traffic analysis are out of scope, and leakage is bounded rather than eliminated (Section 6.19).

## 6.21 E2EE Versus Computation Over Data

The honest tension is that a server cannot compute over what it cannot read. Fully homomorphic encryption is rejected for the serving path on cost. The design instead *partitions fields by computability need*, and forces the partition to be declared:

- **Identity and narrative fields** (names, memos, counterparty details): true end-to-end encryption, opaque to the server, never usable in server-side predicates.
- **Structural fields** (account identifiers, currencies, epochs): pseudonymized but server-visible, because ordering and routing need them.
- **Amounts**: a deployment choice between plaintext-to-server (full functionality) and *committed* amounts using an additively homomorphic commitment scheme, which preserves the ability to verify that postings sum to zero while hiding values — since the sum of commitments opens to the sum of values [Pedersen, CRYPTO '91].

Confidentiality is a static type annotation checked like everything else: end-to-end-encrypted fields cannot appear in server-side predicates, joins or aggregates (a compile error names the leak); committed fields admit only the operations their scheme supports; and declassification requires an explicit capability that appears in the audit trail. The discipline is information-flow typing narrowed to the two lattice points the system actually offers — and narrowed deliberately: labels are **static**, because non-interference results for dynamic labels remain an open problem in that literature, and a guarantee this thesis cannot prove is one it does not claim.

## 6.22 Defence in Depth, Leakage, Limits, and Correctness Obligations

**Layers.** Transport encryption; storage encryption at rest with per-tenant keys; field-level encryption per Section 6.18 with client- or module-held keys; hash-chain integrity computed over ciphertext so tamper-evidence survives confidentiality; key rotation recorded *in the ledger itself*, so rotation is an audited event; and capability-scoped decryption in clients.

**Declared leakage.** Access patterns (which keys are read when), field sizes and cardinalities, structural fields by design, and timing. Mitigations for access-pattern leakage are out of scope.

**Correctness obligations added by encryption.** Chain verification must be ciphertext-stable across re-encryption boundaries, solved by chaining over ciphertext plus explicit rotation records. Conservation over committed amounts holds via the commitment homomorphism and is tested as an extension of the conservation suite. Recovery must not require decryption — and does not, since replay operates over ciphertext.

**Crypto-shredding, stated accurately.** Where erasure obligations conflict with retention, destroying a key renders the retained ciphertext unreadable. The most authoritative published treatment of this technique in a regulatory context states that it can "make the data practically inaccessible, and therefore move closer to the effects of data erasure" — and deliberately stops short of saying it satisfies the erasure right [CNIL, 2018]. This thesis preserves that hedge, and Section 11.1 accordingly lists legal erasure over base facts among the cases where this architecture is the wrong choice.

## 6.23 A Phased Build-and-Evaluation Guide

Each subsystem carries its falsifier from birth: the ledger ships with the floor benchmark before any read model exists; the IR ships with golden-equivalence tests before optimization; partial state ships with the conservation suite wired in; the optimizer ships with the offline dynamic program that grades it; wire protocols ship with the compatibility corpus. *No subsystem without its falsifier* is the methodology of Chapter 5 made concrete, and the ordering follows Chapter 8.

## 6.24 Product Coverage

The banking layer's target portfolio, all expressible as `std::bank` and `std::temporal` constructs over the general core:

- **FX and multi-currency** — FX-atomic transactions (Definition 3.7) with positions as per-currency signals; ledger partitioning by currency with linked cross-currency transfers is the industrial precedent.
- **Derivatives** — forwards and swaps as bitemporal schedules of contingent postings; caps and floors as rate-indexed contingent legs priced by UDFs and settled as balanced postings.
- **Lending** — revolving and term facilities (commitment, drawdown, accrual signals, repayment waterfalls); syndicated facilities with participation shares as fractional postings conserving per participant set.
- **Trade finance** — letters of credit as capability-gated state machines whose transitions are ledger events; trade loans; supply-chain finance with conservation across assignor and assignee.
- **Liquidity** — sweeps, cash pooling and zero-balance structures as scheduled inter-account postings generated by temporal rules and balanced by construction.
- **Authorization and holds** — the reservation model of Section 3.19, with available balance as a REV and the authorization decision at ℓ₅.

Each is a worked example in the artifact; none requires a kernel change, which is the running scope test of Section 9.11.

## 6.25 Syntax Lineage and the SQL-First, Rust-Fallback Rule

**The normative rule** for every surface decision: (1) if SQL already expresses the construct, use SQL's keywords and shape — the sole permitted deviation being pipelined clause *order*, which reads in evaluation order while reusing every SQL keyword verbatim; (2) otherwise, if Rust has an equivalent, use Rust's spelling exactly; (3) otherwise, and only then, invent — and inventions introduce new keywords, types and literals rather than overloading inherited ones.

The rule carries one exception, and the exception is what makes it a discipline rather than a capitulation: **a SQL spelling is not reused where it would sacrifice determinism or efficiency the engine depends on.** An unbounded recursive CTE is admitted only behind the guard of Section 6.12; `float` is not a legal money type; and Rust spellings implying ambient mutation, raw pointers, threads or I/O are not surfaced at all (Section 6.14). The language declines to inherit SQL's permissiveness precisely where permissiveness would cost a guarantee.

This ordering is the reverse of the one an earlier draft adopted, and the reasons for reversing it are given in Appendix J.1: the population to be persuaded reads SQL; the translation theorem of Section 4.7 becomes near-syntactic for the reused fragment when the vocabulary matches, and would otherwise make every clause a translation case; and a language claiming to replace SQL that respells `GROUP BY` has made an unforced adoption error.

**Lineage.** Query vocabulary — filtering, joining, grouping, having, set operations, DDL, DML, TCL and DCL statements — comes from SQL, in pipelined order rather than SQL's inverted clause order. Items, expressions, patterns, generics, traits, attribute annotations and the imperative tier come from Rust — chosen as the fallback because the engine itself is written in Rust and the UDF tier compiles through the same toolchain, so one syntactic idiom spans language and engine. Novel forms — base and ledger declarations, `serve` contracts, anchors, `Money`, value dates and bitemporal literals, idempotency keys, effect annotations, materialization modes, lineage modes and confidentiality annotations — are new, with no false cognates: a novel construct never reuses an inherited keyword with changed meaning.

The rule is testable: any construct violating the precedence order is a specification bug, and Appendix B applies it item by item.

## 6.26 Build Stack and Compilation Targets

**The four-technology stack:** Rust (engine, kernel, stage-0 compiler host); the Niles IR (the semantic centre); WebAssembly (the UDF ABI and the self-hosted optimizer's own execution target); and an optional established back-end for release-grade native code generation, against which the self-hosted back-end's remaining gap is measured and published rather than hidden.

**Targets:** native Linux on ARM64 and x86-64, macOS on ARM64, Windows on x86-64, a WSL profile (Linux with a static-linking profile), and wasm32. The target model is data (Appendix E.2): each target is a declarative record consumed by all back-ends, so adding a target is a data change plus encoder tables. The cross-target determinism obligation, with the Wasm mitigations of Section 6.13, is Appendix C.5.
