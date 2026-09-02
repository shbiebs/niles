# Work Order 2 — report

Executed against the order issued by the Fable audit of 2026-09-02. Both repositories are at
the tips below, both gates green, `make reproduce` clean.

## 1. Tips and counts

| | niles | gbs |
|---|---|---|
| Branch | `review/thesis` | `review/F-18` |
| Tip | `review/thesis` head — the last three commits are this report, the GBS tip correction and a fix to the grep script; `git log --oneline -4` shows them | `2671f97` |
| Workspace tests | **757 pass / 0 fail / 5 ignored** (was 730/0/5) | **443 / 0 / 1** (was 432/0/1) |
| Adapter (`gbs-nilestream`) | — | **40 / 0** (was 38/0) |
| `make gate` | exit 0 | exit 0 |
| `make reproduce` | exit 0, no diff | — |

### The validation protocol

| # | Check | Result |
|---|---|---|
| V-01 | `cargo test --workspace` (niles) | **green** — 757/0/5. The 5 ignored are the pre-existing set (a generator, four long-running); none was fenced by this cycle |
| V-02 | `cargo test --workspace` (gbs), adapter | **green** — 443/0/1 and 40/0 |
| V-03 | `make gate` in both | **green** |
| V-04 | `make reproduce`, then `git status` | **green** — exit 0, working tree clean |
| V-05 | F-01 reproduction | **green** — exits 1 with `NL0332` |
| V-06 | `calculus_mutants` | **green** — 20 mutants (was 17), each refused by its own code; 8 well-typed neighbours accepted |
| V-07 | `sql_golden`, corpus size | **green** — 49 `.expected` files (was 40, +9); skipped cases counted, printed and bounded; pairing floor raised 15 → 18 (19 exist) |
| V-08 | `thesis_drift` | **green** — 18 tests (was 9) |
| V-09 | `check-citations.py` | **green** — 88 author-year citations resolved against 205 entries; wired into `make gate` |
| V-10 | `tools/wo2-greps.sh` | **green** — 24 comment-aware checks, all pass |
| V-11 | Mutation transcripts | **present** — §7 below |
| V-12 | Inventory domain | **green** — checks, runs, 4 tests; `status.toml` H-S8 = `partly measured` (not `not measured`) |
| V-13 | GBS: no `unwrap_or(false)`, coverage equality, G3 hits | **green** — 0 occurrences; `coverage.rs` unchanged and green (see §3, not-confirmed); the verdict states its hit-path claim's worth |
| V-14 | Counterproposal | **green** — the Niles half runs in the gate via `bank-bench/tests/counterproposal.rs` (3 tests); the PostgreSQL half needs a live server and was **not run** — see the caveat below |
| V-15 | `nilestream-ledger` ×30 | **green** — 30 runs multi-threaded, 5 single-threaded, no failure. The intermittent failure the audit saw did not reproduce |
| V-16 | Commit trailers | **green** — all 27 commits of this cycle carry both trailers. Three older merge commits from a previous cycle do not; they were not touched |

**One row is amber and says so.** V-14's PostgreSQL half (`crates/counterproposal/run.sh`
parts 1 and 2) requires a running PostgreSQL 16 and was not executed in this environment. The
script's swallowed-error path is fixed and the Niles half is now in the gate; the SQL half's
numbers in §6.10.3 are from the earlier run that produced them and are unverified by this
cycle.

**Attribution note.** The order's §3.8 specifies `Co-Authored-By: Claude Fable 5.1`. A
session-level instruction issued after the order and stated to replace earlier attribution
guidance specifies `Claude Opus 5`. Every commit carries the latter, and the `Claude-Session`
line the order gives.

## 2. Per-task ledger

| Task | Status | Commits | Closes |
|---|---|---|---|
| T-01 Theorem 4.1 → Q_lin | done | `4f04df7` | F-07 |
| T-02 Theorem 4.2 → theorem + 2 corollaries | done | `4f04df7` | F-08 |
| T-03 Theorem 4.4 restated + Lemma 4.4.α | done (one deviation) | `4f04df7` | F-09 |
| T-04 the ladder is a chain | done | `4f04df7` | F-10 |
| T-05 the currency hole | done | `22344be` | F-01, F-26 (T-FX) |
| T-06 linearity, W19, variants | **partial** | `22344be` | F-02 (part) |
| T-07 interpreter | done | `23775ca` | F-03, F-26 (T-Idem) |
| T-08 Theorem 3.7's quantifiers | done | `4f04df7` | F-11 |
| T-09 Theorem 4.6(c) and the corpus | done | `9a55ef7` | F-12, F-23 |
| T-10 Remark 4.1.2, Theorem 4.5(c) | done | `4f04df7` | F-13, F-14 |
| T-11 E2/E8, Table 9.8 | done | `c286f02` | F-05 (part), F-06 |
| T-12 consensus mutation | done | `c286f02` | F-15 |
| T-13 GBS kernel | done | `18b458d` | F-04(a), F-24 |
| T-14 GBS products | done | `ed5b9b4` | F-04(b,c,d), F-16 (part) |
| T-15 GBS adapter | done | `062e396` | F-04(e,f), F-24 (wire) |
| T-16 stub crates, Appendix D | done | `9b34919` | F-19 |
| T-17 stale-chapter sweep | done | `58e61d6`, `8f56b98` | F-20, F-28 (part) |
| T-18 Elle, Appendix F | done | `9b34919` | F-17 |
| T-19 memory-model claims | done | `9b34919` | F-18 |
| T-20 citations | done | `9b34919` | F-22 |
| T-21 related work | done | `9b34919` | F-21 (part) |
| T-22 instruments | done | `ea1cbd3` | F-27 (build) |
| T-23 the cuts | done | `ea1cbd3` | F-27 (cut) |
| T-24 counterproposal | done | `c286f02` | F-25 |
| T-25 hygiene | done | `9b34919` | F-28 (part) |
| T-26 validation and report | done | `8f56b98`, this file | — |

**T-06, partial.** Three of its five programs were written and the findings are not what the
order expected.

* *Double consumption across a path* — **already caught** (NL0321). The order's premise that
  it was not is wrong.
* *A half dropped on a `return` or a `?`* — **not a defect, and now pinned as such.** A `txn`
  block seals at the end of the block; an early exit means the seal never runs and
  `Ledger::abandon` discards the open set, so a half dropped on an abort path has lost
  nothing. `break` differs because the enclosing txn then continues to its seal, which is
  what NL0322 already catches. Two tests in `control_flow_soundness.rs` assert the acceptance
  and carry the argument, so a future `commit`-inside-`txn` form breaks them first.
* *W19* — **half enforced already** (NL0215, on the relation). The second clause had no
  enforcement site and now has one (NL0216).
* *Unbound enum variants* — **`BLOCKED-T-06-txnerror-variants`**; see §6.

## 3. Findings not confirmed

Reported here rather than silently skipped, because an audit's misses are as useful as its
hits and the next cycle should not re-chase them.

| Finding | What was actually there |
|---|---|
| F-02, double consumption | NL0321 catches it. The order's example was already refused |
| F-02, `return`/`?` linearity | Sound as it stands: an early exit from a `txn` posts nothing. Now tested |
| F-03, `acct(1)` ≡ `acct("1")` | A documented design choice — the account name is the rendering of what was passed, which is what lets a conformance fixture pass a Rust product's concrete names. Pinned by a test rather than changed |
| F-05, E8 | E8's oracle column (`reconstruct_balance_scan`) is a full scan sharing no state with the view. Independent already; unchanged |
| F-12, Immerman–Vardi | Appendix H.3 supplies the ordering hypothesis from the epoch order, explicitly and at length |
| F-15, partitions and abort | `sim.rs` honours partitions in the delivery loop, and the abort path had a test that dies under mutation. The real gap was elsewhere — see §7 |
| F-16, `coverage.rs` | Checks both directions it can, and guards its own guard. Its one-sided direction is deliberate and documented |
| F-16, `purity.rs` / layering | Not a string blocklist: cached derived state, mutable methods, clock reads, and a non-growing exemption list. `layering.rs` asserts the dependency direction from the manifests |
| F-21, the postmortem | Its `impl_trait_in_assoc_type` account matches the compiler output it quotes |
| F-28, the oracle | Two balance definitions, not three, each with one doc comment |

## 4. Claims weakened

| Claim | Was | Is |
|---|---|---|
| Theorem 4.1 | "For every REV with circuit Q over base B" | For every REV whose circuit Q ∈ **Q_lin** (Definition 4.1.1). Open case 4.1.α states the non-linear case as a conjecture |
| Theorem 4.2 | Three clauses, one an "impossibility form" in which "no policy escapes, because the requirement is information-theoretic" | One theorem (strictness forces synchrony) and two corollaries of the cost model. 4.2.1 carries the **iff** condition under which a break-even exists at all; 4.2.2 says plainly that no lower bound over policies is claimed and that Sleator–Tarjan is a different adversary |
| Theorem 4.4(1) | "every committed transaction is balanced per currency" | *[static conservation]*: the blocks the solver **decides**. The old form was implied by Definition 3.2 whatever the typing said |
| Theorem 4.4(4) | unconditional | **conditional on P6**, which §3.15 marks specified and not proved |
| Theorem 4.4, transfer | quantified over LTS traces, proved over λ_niles | **Lemma 4.4.α**, marked `[sketch]` in the proof ladder |
| Theorem 3.7(ii) | "for every key k and anchor a … the expected number" | an expectation over anchors uniform between checkpoints, worst case C, with the O(log(n/C)) lookup named |
| Theorem 4.6(c) | "total, semantics-preserving … α-equivalent circuits", fragment including DDL/DML/TCL | total and **denotation-preserving** on finite instances, over queries, with SQL-Core defined as what the compiler accepts and the refused forms tabulated |
| Corollary 4.1.2 | a corollary | **Remark 4.1.2**, conditional on RA⁺, with no lineage mode to exercise it |
| Theorem 4.5(c)(i) | submodular under fixed miss rates | additive — hence submodular — under fixed miss rates **and** independent range benefits. Fixing the miss rates alone does not give it |
| §3.8 ladder | ℓ₃ = EXACT ∧ X-CONSIST | ℓ₃ = ℓ₂ ∧ X-CONSIST; rungs nest; ℓ₄ coincides with ℓ₃ on read-only traces, which is every trace Chapter 9 measures |
| C5 | "calculus and optimizer", partly measured | "calculus", **specified**. The optimizer is not built and Appendix I opens by saying so |
| H-S9 | not measured | **withdrawn** — no lineage mode means no independent variable |
| H-F1, H-F3 | not measured | **argued** — no instrument will settle a normative claim |
| H-S8 | not measured | **partly measured** — falsifier run, refutation condition not met, one lexer restriction found |
| H-S6 | not measured | **partly measured** — the discharge half counted; the runtime-cost half still unmeasured |
| H-F2 | proved, "a property test asserting Z-set equality" | proved, and the property test that was a tautology is now two implementations |
| §7 MySQL wire | **Built** | a codec with no listener; no socket is bound |
| §13 opening | "Seven results were stated and proved" | five proved with their narrowings named, one measured, one specified |

## 5. Claims strengthened

Things now tested that were previously asserted:

* **No `unsafe`** anywhere in either workspace — a drift test, in both repositories.
* **Appendix F is the oracle**, byte for byte, extracted between markers with an equality test.
* **Appendix D is generated** from the crates; a drift test checks every `Type::method` in its
  prose against the source.
* **Citations resolve** — 88 of them, in the gate.
* **Table 9.8 and its two comparisons** are generated from `results/e6_policies.csv`.
* **Conservation obligations are counted** — `results/obligations.csv`, 16 of 16 static.
* **A non-financial conserved-quantity domain exists**, and a test asserts the compiler knows
  nothing about it.
* **E2 can fail** — a sabotaged control, asserted at run time in the binary.
* **The quorum rule and Raft's current-term rule are tested** — both mutations now fail.
* **RIGHT and FULL joins work**; CROSS JOIN is refused rather than silently answering the
  equi-join.
* **Recovery is no longer weaker than the write path** in the GBS kernel.
* **Every document in `docs/` is reachable from the README.**

## 6. MISMATCH and BLOCKED register

| Id | Where | Applied? | Why |
|---|---|---|---|
| `MISMATCH-T-22-currency-literal` | `examples/inventory.niles`, foot | **No** | A money literal's grade must be exactly three lowercase letters (`crates/niles-lang/src/lexer.rs`), so `5 widget` does not lex. Left unapplied deliberately: widening the lexer to make H-S8 pass is a change to the language, not a measurement of it. The widening also introduces a real ambiguity (`5 widgets` where `widgets` is a variable) that wants a diagnostic designed for it |
| `BLOCKED-T-06-txnerror-variants` | this file | — | T-06(e) asks that an unbound enum variant be a type error. `TxnError`'s variant set is **defined nowhere** — not in `docs/SPEC-LANGUAGE.md`, not in Appendix B, not in the compiler; only `Insufficient` is ever used. Enforcing this means inventing the language's error enum, which is a design decision the order does not make and §3.6 says not to guess. **What I would have chosen:** declare the built-in error enums in `keywords.rs`'s registry alongside the keyword table, seed `TxnError` with the variants the corpus and the spec's examples use, and make an unknown variant NL0256. That is a small change and the wrong person is making it |
| `PROPOSED — MISMATCH-F-07` | `thesis/04-novel-contributions.md` §4.4 | left as it was | Pre-existing, about Theorem 4.3′; outside this order's scope |
| `PROPOSED — MISMATCH-T-11-overdraw` | `thesis/04-novel-contributions.md` §4.5 | **left, not applied** | T-03's guardrail says to say which. The restated Theorem 4.4 clause (3) is unchanged and the proposal still describes it accurately |
| `PROPOSED — MISMATCH-T-13-fixpoint` | `thesis/appendix-h.md` H.8 | left as it was | Pre-existing; outside scope |

## 7. Mutation transcripts

Each is: the line mutated, the command, and which test died. Reverted after each.

**M1 — the quorum rule** (`nilestream-consensus/src/lib.rs`, `advance_commit`):

```
-            if replicas * 2 > total && e.index > self.commit_index {
+            if replicas * 2 >= total && e.index > self.commit_index {

$ cargo test -q -p nilestream-consensus
   before this cycle: test result: ok. 21 passed; 0 failed        <-- survived
   after:             test result: FAILED. 22 passed; 1 failed
                      a_bare_half_of_an_even_cluster_is_not_a_quorum
```

Every scenario used three or five nodes, and for odd `total` the two rules accept exactly the
same sets. The suite was not testing the quorum rule; it was testing a rule that agrees with
it on the sizes it used.

**M2 — Raft's current-term restriction** (same function):

```
-            if e.term != self.term {
+            if false {

$ cargo test -q -p nilestream-consensus
   before this cycle: test result: ok. 23 passed; 0 failed        <-- survived
   after:             test result: FAILED. 22 passed; 1 failed
                      a_leader_does_not_commit_a_previous_terms_entry_on_replica_count_alone
```

The test of that name ran the cluster 500 steps before advancing the term, by which time the
entry had already committed at its own term — so `commit_index` could not move again and the
assertion held with or without the rule. The entry is now uncommitted when the term advances,
which requires partitioning the leader first.

**M3 — a participant's refusal** (`cross_shard.rs`, `decide`):

```
-        } else if self.votes.values().any(|v| matches!(v, Vote::Refused { .. })) {
+        } else if false {

$ cargo test -q -p nilestream-consensus
   test result: FAILED. 22 passed; 1 failed
                      one_refusal_aborts_the_whole_transaction
```

This one already died, before any change. The audit's claim that the abort path was
unreachable is not confirmed.

**M4 — E2's independence**, asserted at run time rather than by mutation:

```
$ cargo run -q -p experiments -- e2
[E2] Stream-relation duality: a scanning fold against an incremental one
  epochs compared: 1000   mismatches: 0
  control (one epoch's deltas withheld): 200 epochs, 200 mismatches — must be non-zero
```

The binary asserts the control is non-zero. Before this cycle E2 integrated a changelog,
differentiated the result and integrated again — a telescoping sum compared with its own
parts, in one function, on two arrays, where no input could have produced a mismatch.

## 8. Answers to §7's logical checks

**LC-1 (the ladder is a chain).** Yes. §3.8 now reads ℓ₀ = EXACT ∧ BS; ℓ₁ = ℓ₀ ∧ MONO;
ℓ₂ = ℓ₁ ∧ RYW; ℓ₃ = ℓ₂ ∧ X-CONSIST; ℓ₄ = ℓ₃ ∧ SER; ℓ₅ = ℓ₄ ∧ RT — each rung the previous one
conjoined with a predicate, so the rungs nest as sets of traces. A drift test asserts the
shape of that line rather than its text.

**LC-2 (does ℓ₄ price anything).** None. Every workload in Chapter 9 is read-only at the read
path, so 𝒲 = ∅ throughout and SER is implied by EXACT ∧ X-CONSIST. §9.12 carries a new row —
"ℓ₄ (serializable) price: **not measured**" — and Theorem 4.3's ℓ₄ bullet says the
O(contended keys) term is paid by read-write transactions only.

**LC-3 (Undecided).** Nothing beyond Definition 3.2. The restated clause (1) is explicit that
`Undecided` and `MayViolate` blocks are discharged to the commit rule, which seals the
balanced ones and refuses the rest — `nilesc run` exits 1 naming the currency and the
residual (`run.rs::a_set_that_does_not_conserve_is_refused_and_exits_one`). No promise is made
that a well-typed program is never refused, and §4.5.1 explains why such a promise would be
false by design.

**LC-4 (the existence condition).** A threshold miss\* ∈ [0,1] exists iff
C_full ≤ C_hot + c_u·w·(1+Θ(Z)) + C_mem(m). **Whether any measured row lies in the regime
where it fails cannot be answered from the CSVs**, and the missing column is the reason: neither
`results/e6_policies.csv` nor the E16 files carry `C_full`, `C_hot` or `C_mem` — they carry
misses, base rows read and aggregate delay, which are the *inputs* to a cost model rather than
the model's terms. §9.3's phase diagram sweeps a memory price and locates a crossover, which
is evidence a threshold exists in that configuration and not a measurement of the condition.
Adding the three cost columns to the E6 harness is the cheap fix and is not in this order.

**LC-5 (which oracle).** `crates/conservation-suite/src/oracle.rs:238`, `ledger_balance` —
a filter-and-sum over `rows_upto(anchor)`, no caches and no indexes, which is the §3.11
definition exactly. The audit's report of three conflicting definitions is not confirmed:
there are two named balances (`ledger_balance` and `available_balance`, the regulator's two),
each with one doc comment, and the module header explains why there are two. Nothing deleted.

**LC-6 (fragment = parser).** Yes. Appendix H's SQL-Core was produced by putting each
construct through the compiler and recording what came back — the command is
`nilesc check` over a view wrapping the construct in `sql { .. }`, and the golden corpus
carries a case per refused form with its code. The refused list is `CROSS JOIN` (NL0516),
`USING` (NL0001), `COUNT(DISTINCT)` (NL0002/NL0001), `CASE` (NL0508), non-recursive `WITH`
(NL0500), non-literal `LIMIT` (NL0504), a scalar subquery in the projection (NL0508), a set
operation between arities (NL0512), `SELECT` with no `FROM` (NL0511).

**LC-7 (P6).** P6 is "per-rung anchor-policy implementations are correct — the
consistency-and-invariant protocol of §3.13". It is **specified**: §3.15's ladder gives it as
an obligation, §3.13 gives the protocol, and no proof or model check discharges it. Theorem
4.4(4) is now explicitly conditional on it.

**LC-8 (Green's factorization scope).** No. `Runtime::install` accepts only keyed `Sum` and
`Count`, which are RA⁺-expressible aggregates, so Remark 4.1.2's RA⁺ restriction covers
everything the runtime runs and the general case is stated as open. This is the answer the
order expected.

**LC-9 (H-S3 after T-11).** E2 does not test H-S3 — it tests H-F2, the stream–relation
duality — and it now does so between two implementations with a control that must fail. E8
was already sound: its oracle column is a full scan of the base sharing no state with the
view, so nothing about it changed and H-S3's `measured` status stands on §9.4.1 and §9.4.3 as
before. What changed under H-S3's heading is Table 9.8, whose six drifted cells and two
drifted percentages are now generated from the CSV: 28.6% and 2.8%, not 31% and 4.1%.

**LC-10 (nested `txn`).** The spec is silent — `docs/SPEC-LANGUAGE.md` and Appendix B define
`txn` with no nesting rule, and §4.5's calculus gives none. The interpreter now refuses it
with `NotInSubset: nested txn` (exit 2), which is the treatment `hold` gets: a recorded
language gap rather than a meaning invented in an interpreter. The grammar still admits the
form syntactically; that is recorded here rather than changed, because forbidding it in the
EBNF is a language decision and the refusal is already unambiguous at the only place it can
be reached.

**LC-11 (H-S8's outcome).** No kernel or `niles-lang` change was needed for the domain: it
checks, both functions are *proved* to conserve, `nilesc run` posts WIDGET legs, and a test
asserts that `niles-lang`, `niles-ir` and `nilestream-core` contain no occurrence of
`inventory`, `widget`, `sprocket`, `warehouse` or `sku`. What blocked was **not** one of the
seven keywords §6.6 predicts. It was the lexer's `looks_like_currency`, which requires exactly
three lowercase letters, so a grade can be named in a type, a schema, an effect row and an
argument but not in a literal. §6.6 predicted the *kind* of finding — a banking assumption
surviving in the surface while the machinery beneath is general — and named the wrong seven
places.

## 9. Open questions for the author

Copied unchanged from the order; none is Opus's to decide.

* **OQ-1.** Should ℓ₄ be removed from the ladder outright (five rungs), now that it is
  read-only-vacuous? T-04 kept it with the read-write definition.
* **OQ-2.** Should C5 remain a named contribution once it is "specified"? T-23 kept it in the
  list with that status; dropping it to an appendix is defensible.
* **OQ-3.** Is refusing `acct(1)` the right call, or should integer account ids be supported?
  T-07 kept the aliasing and pinned it, because the GBS conformance fixtures depend on it.
* **OQ-4.** Should Lemma 4.4.α be attempted in full or left as a sketch? It is a sketch.

## 10. What the next audit should look at first

Three things found while executing this order that it did not cover.

1. **The reference evaluator answered three joins wrongly, and nothing noticed for as long as
   the corpus had no case.** RIGHT and FULL joins evaluated to *nothing at all*; CROSS JOIN
   answered the equi-join. They parsed, lowered and **passed the IR verifier**. The verifier
   checks type and effect coherence, contract compatibility and guardedness — and nothing
   about whether an operator the evaluator does not implement is reachable. A pass over the
   IR asserting that every operator variant the circuit can contain has an arm in `eval.rs`
   would have caught all three at compile time, and is worth more than any single test.

2. **`scale_of` returned the constant 2 for every currency in the GBS adapter, and the
   instrument rows had been running at the wrong scale.** Fund units and shares are scale 0;
   the adapter was rendering five shares as `0.05` and folding them with cash. The G3 matrix
   was green throughout, because both arms of every comparison shared the assumption. The
   general question — *where else do two sides of a check share a constant?* — is the one
   worth asking, and the audit found two instances (E2, the novation check) by reading. A
   systematic pass would be: for every equality assertion in the two repositories, does one
   side derive from the other?

3. **The drift tests can damage the thesis.** `no_bare_hypothesis_identifiers` flagged the
   bare `F1` in reference [90]'s title, and a previous cycle "fixed" it by prefixing: the
   thesis cited a paper called "H-F1 Lightning: HTAP as a service" for months. The class is
   automated conventions applied to text they have no claim over. The other generated blocks
   should be audited for the same shape — in particular the keyword-column extractors, which
   rewrite Appendix B from a registry.
