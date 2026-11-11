# 6. The decision catalogue

*The construction manual, chapter 6 (cycle 17, C17-13). Every load-bearing decision the author has
taken, as a card: the alternatives that were on the table, the criterion that chose between them,
the evidence the choice rested on, and the measurement that would reverse it. A card whose
evidence does not exist says so: a decision taken on judgment is recorded as judgment, not
dressed as a measurement. The ids are the ones the audits used (cycle 13's A-n, cycle 14's D-n
and S-1, cycle 15's DA-n and R2-x, cycle 16's DB-n, cycle 17's D17-n). The list of ids this
chapter must carry is `docs/manual/decision-ids.txt`, and a test fails if one has no card.*

*Where a later decision superseded an earlier one, both cards stay, and the earlier one says what
replaced it. The manual records how the project arrived where it is, not only where it is.*

The replay of this chapter runs the test that checks it:

```sh
cargo test -q -p bank-bench --test manual_catalogue
```

## 6.1 Cycle 13: scope and shape (2026-09-21)

### A-1 — What Niles is

- **Alternatives:** (a) the full Niles grammar as the language; (b) a Rust DSL plus a SQL catalog checker, with linearity downgraded from a compile error to a lint.
- **Criterion:** a plan one author can finish, with the thesis's checks kept where they carry the claim.
- **Evidence:** cycle 13's audit found the full-grammar plan unfinishable at the observed rate of work. No measurement of the price of (b) existed when it was taken.
- **Chosen:** (b), with the price (error → lint) stated in the thesis; Appendix E (self-hosting) moved to future work.
- **Superseded:** D-2 measured the price and found it worse than stated (a bound-then-dropped hold compiles under `-D warnings`); E30 and E30b′ then made annotated SQL the language and Niles the specification (R2-b, DB-1, DB-8).
- **What would reverse it:** already reversed by D-2's measurement.

### A-2 — The universe and the scale row

- **Alternatives:** synthetic parameters only; U4, the synthetic generator calibrated by the author's professional shapes (distributions only, never data); public data alone.
- **Criterion:** realistic shapes without any data attributable to an employer.
- **Evidence:** the author's shapes, given as distributions. The big-lender row (10⁶ accounts, 10⁷ postings a day, 50 views per account, teller p99 20 ms, regulator p99 5 s) is the author's statement, not a measurement.
- **Chosen:** U4, with the big-lender row; counted work at full scale, wall-clock on a declared sample.
- **What would reverse it:** a public or operator-day trace (DA-9) whose fitted shapes fall outside U4's ranges.

### A-3 — Scope

- **Alternatives:** the full plan; the recommended cut: stop the self-hosting bootstrap, the MySQL listener, consensus and distribution, and the online optimizer; shrink the grammar and GBS's products to a pilot slice.
- **Criterion:** what the theory needs measured, finishable by one author.
- **Evidence:** cycle 13's count of planned against executed cards.
- **Chosen:** the cut. The stopped components stay in the thesis as specification and future work (D-3's default).
- **What would reverse it:** a cycle whose executed cards keep pace with the full plan.

### A-4 — niles is public

- **Alternatives:** a private repository; public on purpose.
- **Criterion:** the thesis's artifacts are open to examiners and readers.
- **Evidence:** none needed; a choice of the author's.
- **Chosen:** public, AGPL-3.0-only (decision 9 of 2026-09-30 aligned the crates' metadata with the licence).
- **What would reverse it:** an author decision, not a measurement.

### A-5 — GBS reaches the container by bundles

- **Alternatives:** a GitHub integration; git bundles from the author's Mac.
- **Criterion:** no credential ever enters a container, a chat or a file.
- **Evidence:** none needed; a security rule.
- **Chosen:** bundles (kept by DA-10).
- **What would reverse it:** a read-only, credential-free channel the author approves.

### A-6 — The pilot slice is syndicated lending, Loan IQ-shaped

- **Alternatives:** a capital-markets slice; a retail slice; syndicated lending.
- **Criterion:** the domain the author operates daily, so its semantics can be signed by a practitioner.
- **Evidence:** the author's role (A-11).
- **Chosen:** syndicated lending.
- **What would reverse it:** an author decision; DB-5 later widened the scope to Calypso's side as well.

### A-7 — The residual rule is declared per facility

- **Alternatives:** a fixed rule; a rounding account; declared per facility.
- **Criterion:** a syndicate's shares sum exactly to every movement, with the residue placed by the agreement.
- **Evidence:** the exact-rational participation mechanism and its tests.
- **Chosen:** per facility, default largest remainder then the agent absorbs. The lending sign-off (D17-5) later made the choice per facility among largest remainder, the agent, or a pro-rata carry.
- **What would reverse it:** an agreement form the three rules cannot express.

### A-8 — `committed` means Appendix B's commitment

- **Alternatives:** the registry's earlier sentence; Appendix B's additively homomorphic commitment.
- **Criterion:** one definition across the thesis and the code.
- **Evidence:** the two texts disagreed.
- **Chosen:** Appendix B's.
- **What would reverse it:** a proof that the sum check needs a different commitment.

### A-9 — LC-20 is struck

- **Alternatives:** keep the obligation; strike it.
- **Criterion:** an obligation that blocks work must be needed by a claim.
- **Evidence:** cycle 13's audit; no claim depended on it.
- **Chosen:** struck.
- **What would reverse it:** a claim that is found to need it.

### A-10 — `master` follows the cycle head

- **Alternatives:** fast-forward every cycle; move it only after review.
- **Criterion:** `master` is where a reader starts.
- **Evidence:** none; a process choice.
- **Chosen:** fast-forward every cycle. Cycle 17 moves it at round end, after review.
- **What would reverse it:** an author decision.

### A-11 — Reviewers

- **Alternatives:** name external reviewers; use published standards as stand-ins; state the slots empty.
- **Criterion:** no claim called sound that needs a reviewer nobody is.
- **Evidence:** the author is a lending-operations practitioner; no accountant, quant or security reviewer is engaged.
- **Chosen:** the author is the lending-operations reviewer; the accounting, quant and security/operations slots are recorded as empty.
- **What would reverse it:** a named reviewer accepting a slot.

## 6.2 Cycle 14: the counterexamples and the language's price (2026-09-27 to 2026-09-28)

### D-1 — PostgreSQL login for the harness

- **Alternatives:** (a) SCRAM-SHA-256 in the harness's own wire client and a checked-in provisioning script; (b) `trust` authentication by editing `pg_hba.conf`.
- **Criterion:** the gate runs against a server nobody had to weaken.
- **Evidence:** a fresh container failed the gate on a precondition written in no file; a harness that spoke only `trust` passed only where `pg_hba.conf` had been weakened by hand.
- **Chosen:** (a). Chapter 2 carries the card in full.
- **What would reverse it:** a release in which no result depends on PostgreSQL.

### D-2 — Whether Niles must exist as a language

- **Alternatives:** keep A-1(b) and state the price; (b′) keep Niles's checker for the money kernel only; return to the full grammar; (d) a checked dialect of SQL with no new grammar.
- **Criterion:** which surface refuses the defect classes the thesis claims, measured on one corpus.
- **Evidence:** under a Rust DSL, a hold bound, read and dropped compiles under `clippy -D warnings`, where Niles's NL0320 refuses it.
- **Chosen:** measure (b) against (d): SQL+C+L beside the Niles checker on the E14 corpus plus d14's five spellings, under a pre-registered rule.
- **What would reverse it:** the measurement itself; it was taken, and R2-b records the result.

### D-3 — Syntax by measurement

- **Alternatives:** decide the principal syntax by argument; by a study over candidate surfaces (annotated SQL, Rust-shaped, Niles, PRQL, Datalog, a new syntax).
- **Criterion:** expressiveness, safety and speed on one pre-registered corpus.
- **Evidence:** E30, then E30b′.
- **Chosen:** the study. Chapters on components A-3 stopped stay in the thesis as specification and future work (a marked default).
- **What would reverse it:** a later study on a wider corpus with a different verdict.

### D-4 — Verification modes

- **Alternatives:** (a) attested checkpoints, so `chained` and `conserved` verify from the latest checkpoint; (b) scrubber only.
- **Criterion:** a read path's verification cost must be workload-shaped, not history-shaped (Contribution 3).
- **Evidence:** the audit's reading: without checkpoints, `chained` and `conserved` re-verify from genesis, the same objection that removed `indexed` from read paths.
- **Chosen:** (a), with (b) as the interim behaviour; nothing beyond `lineage` built yet.
- **What would reverse it:** a measurement showing attested checkpoints cost more per read than the scrubber's latency allows.

### D-5 — Universe calibration

- **Alternatives:** synthetic only; U4 plus public activity calibration plus a read-skew sweep.
- **Criterion:** shapes defensible from public sources, with skew a swept variable rather than an assumption.
- **Evidence:** the public sources named in the decision (DTCC, SEC ABS-EE, Freddie Mac, BIS Triennial, FFIEC, Fedwire); α = 0.6 as the declared synthetic default (revised from 0.9 by the author).
- **Chosen:** the combined calibration, with α swept over [0.6, 1.6].
- **What would reverse it:** an operator-day trace (DA-9) whose fitted α falls outside the swept range.

### D-6 — The comparison arms

- **Alternatives:** N against plain PostgreSQL only; a wider set.
- **Criterion:** every serious way to build the read side is an arm, so a verdict for N is earned against the strongest counterexample.
- **Evidence:** the hybrid designs the audit found (ReadySet, pg_ivm, a REV sidecar, TigerBeetle as a ledger floor).
- **Chosen:** N, P+, P, M, H1, H2, H3 and T. C17-03 later added P-inc.
- **What would reverse it:** an arm shown to be dominated at every size, which would leave the set.

### D-7 — Accounting and regulatory review

- **Alternatives:** wait for a reviewer; cite the governing standards and state that no accountant has reviewed.
- **Criterion:** no accounting claim stated as reviewed when it is not.
- **Evidence:** A-11's empty slots.
- **Chosen:** cite the standard per rule, make rules effective-dated, jurisdiction-keyed data, and keep a regulatory register (first: US, Mexico, EU, UK). Research, not legal advice.
- **What would reverse it:** a named reviewer's sign-off, which would replace the statement of absence.

### D-8 — GBS's order

- **Alternatives:** lending semantics first; the command envelope and business date first.
- **Criterion:** controls before products, so every product posts under them.
- **Evidence:** three live defects the audit found (non-unique derivative ids, epoch-embedding ids, single-actor four-eyes).
- **Chosen:** the envelope (principal, role, request id, reason, maker and checker, approval) and the business date, then lending.
- **What would reverse it:** none expected; it is an ordering.

### D-9 — The front end's targets

- **Alternatives:** an operator UI only; an SDK only; both, across the author's toolchains.
- **Criterion:** cover what a bank's operators and its other systems use.
- **Evidence:** the author's statement of the toolchains he uses or wants.
- **Chosen:** an operator UI on the JVM and a JVM SDK; candidates measured in a spike. D17-8 and D17-9 refined it.
- **What would reverse it:** the spike's pre-registered rule.

### D-10 — Cadence

- **Alternatives:** a full audit every round; build rounds with a light self-check.
- **Criterion:** more built per audit, without losing review.
- **Evidence:** cycle 13's planned-against-executed counts.
- **Chosen:** a build round of at most twelve cards; DA-5 replaced the card cap with author touchpoints.
- **What would reverse it:** a build round whose self-check misses defects the next audit finds.

### D-11 — "Poised to replace"

- **Alternatives:** soften the wording; build the operational layer.
- **Criterion:** a claim of replacing Loan IQ needs the layer operators use daily.
- **Evidence:** the audit's list of what Loan IQ users rely on (approvals, audit trail, business date, calendars, general ledger, settlement, reconciliation, regulatory extracts).
- **Chosen:** build the operational layer across rounds; until it exists, the thesis says it is in progress.
- **What would reverse it:** an author decision to narrow the product goal.

### S-1 — Scale

- **Alternatives:** one bank size; a sweep across orders of magnitude.
- **Criterion:** GBS must serve small banks and big ones, so every verdict is stated per size range.
- **Evidence:** the author's statement (2026-09-28).
- **Chosen:** every comparison is a size sweep, small-lender and big-lender profiles marked, crossovers reported.
- **What would reverse it:** none; it is the shape of every later design.

## 6.3 Cycle 15: the language study and the engine re-measured (2026-09-30)

### DA-1 — The language

- **Alternatives:** grow Niles now; one more study (E30b′) first, comparing an upgraded SQL+C+L with Niles on E30's corpus plus an adversarial corpus written before any program.
- **Criterion:** the Niles grammar is kept only for defect classes annotated SQL cannot refuse at check time.
- **Evidence:** E30's verdict per class.
- **Chosen:** E30b′ first; no language growth before its result.
- **What would reverse it:** E30b′ itself, whose result R2-b records.

### DA-2 — The ten decisions of 2026-09-30

- **Alternatives:** build all ten; re-sequence them behind E30b′ and E27b.
- **Criterion:** nothing that a pending measurement could invalidate is built before it.
- **Evidence:** the dependency of each decision on E30b′ or E27b.
- **Chosen:** re-sequence: interpreter gaps, value-date writes, the `?` fix, the licence and PostgreSQL 16 on the Mac kept; the rest held. R2-c later built the held IR decisions before E27b.
- **What would reverse it:** none; a sequencing decision.

### DA-3 — The engine

- **Alternatives:** accept E27's verdict; fix the engine as designed and re-measure (E27b).
- **Criterion:** the engine is judged as designed, by the unchanged engine rule.
- **Evidence:** cycle 15's audit: E27 had not measured N as designed (a multi-currency read never used the view; the memory clause set N's process PSS against a cluster whose base is on disk; q3 and q4 could not run on N).
- **Chosen:** fix and re-measure, with view-state bytes as the memory metric.
- **What would reverse it:** E27b's verdict, which DB-1 then acted on.

### DA-4 — Cycle 14's unlanded cards

- **Alternatives:** drop them; keep all six re-scoped.
- **Criterion:** each still answers a question the thesis or GBS needs.
- **Evidence:** the audit's re-reading of each card.
- **Chosen:** all six, re-scoped; the envelope became the first GBS card.
- **What would reverse it:** none; a planning decision.

### DA-5 — Round size

- **Alternatives:** a fixed card cap; rounds sized by sessions and author touchpoints.
- **Criterion:** the author's time is the scarce input.
- **Evidence:** round 1's touchpoint count.
- **Chosen:** at most three author touchpoints per session, no fixed card cap.
- **What would reverse it:** a round that stalls on touchpoints the cap did not foresee.

### DA-6 — The front end

- **Alternatives:** one stack; spikes A′ (Rust core with native shells) and B (Rust/WASM in Tauri), API first.
- **Criterion:** the pre-registered spike rule, after the client API exists.
- **Evidence:** cycle 15's research on the families' coverage and memory.
- **Chosen:** API first, then A′ and B. D17-8 dropped gRPC; D17-9 re-scoped A′.
- **What would reverse it:** the spike's rule.

### DA-7 — The manual

- **Alternatives:** write the manual at the end; a chapter every round, each replayed.
- **Criterion:** the manual builds the system, so each chapter must run.
- **Evidence:** none needed; a process choice.
- **Chosen:** a chapter every round; D17-10 made it two.
- **What would reverse it:** none.

### DA-8 — Thesis wording

- **Alternatives:** correct later; correct all four overclaims now.
- **Criterion:** no sentence claims more than is measured.
- **Evidence:** the audit's four findings ("no system combines…" without its search terms; Pareto skew stated as fact; H-E1's text; status overclaims).
- **Chosen:** all four now, with drift tests where one fits.
- **What would reverse it:** none.

### DA-9 — Read skew

- **Alternatives:** a synthetic sweep only; an operator-day replay once screens exist.
- **Criterion:** skew is measured from use, not assumed.
- **Evidence:** none yet; the replay needs GBS screens.
- **Chosen:** the synthetic sweep now; the author drives a scripted operator day later, its reads logged as the trace.
- **What would reverse it:** the trace itself, which replaces the synthetic prior.

### DA-10 — GBS's remote access

- **Alternatives:** a GitHub integration; bundles plus the author's pasted read-only output.
- **Criterion:** no credential anywhere.
- **Evidence:** A-5.
- **Chosen:** bundles and the author's read-only commands.
- **What would reverse it:** as A-5.

### DA-11 — Reproduce in the gate

- **Alternatives:** reproduce by hand; `make gate` runs `make reproduce`.
- **Criterion:** "gate exit 0" includes every byte-deterministic result regenerating identically.
- **Evidence:** E18's results had drifted without a failure.
- **Chosen:** the gate runs reproduce.
- **What would reverse it:** none.

### R2-a — How a capture executes in the interpreter

- **Alternatives:** add a payee convention during the study; execute as Appendix B writes it.
- **Criterion:** the study measures the language as specified, not as patched during it.
- **Evidence:** Appendix B names no payee for `resolve h post amt`.
- **Chosen:** as written: a capture closes the hold and posts nothing; E30's T04 is recorded as not expressible as stated.
- **What would reverse it:** a revision of Appendix B, outside a study.

### R2-b — What E30b′ makes the language

- **Alternatives:** Niles as the language; annotated SQL for everything; annotated SQL plus Niles's schema declarations.
- **Criterion:** E30b′'s per-class verdict.
- **Evidence:** E30b′ found Niles's grammar needed only for schema-level omission (ledgers, currencies, linear holds, capabilities).
- **Chosen:** annotated SQL (SQL+C+L) for queries, transactions, views and temporal reads; Niles's grammar for schema declarations only; the Niles checker as specification and reference.
- **What would reverse it:** a defect class that annotated SQL cannot refuse at check time and Niles's grammar can, outside schema declarations.

### R2-c — When to build the held IR decisions

- **Alternatives:** after E27b; before E27b's re-measurement.
- **Criterion:** E27b measures the changed engine once.
- **Evidence:** C15-02 itself re-runs the arms, so only "before C15-02" keeps "before the re-measurement".
- **Chosen:** now, as C15-05b, before C15-02.
- **What would reverse it:** none; a sequencing decision.

### R2-d — Aggregates across currencies

- **Alternatives:** tag every aggregate's currency; tag only where the key or a filter does not fix it.
- **Criterion:** no cross-currency sum, and the balance view's fast path unchanged.
- **Evidence:** F-05b-6: the reference evaluator summed amounts across currencies where the group key did not fix the currency.
- **Chosen:** tag where needed.
- **What would reverse it:** a view where the untagged path sums across currencies.

### R2-e — The recursion bound

- **Alternatives:** a fixed round bound; the bound as a `serve` setting.
- **Criterion:** `with recursive` has no measure, so the bound is its only guard and belongs to the view's contract.
- **Evidence:** none beyond the design.
- **Chosen:** a `serve` knob, default 1,000; non-convergence is still refused.
- **What would reverse it:** a termination measure for the recursion, which would make the bound unnecessary.

### R2-f — `recorded_at` in `select *`

- **Alternatives:** shown; hidden.
- **Criterion:** `*` returns the user's columns.
- **Evidence:** none; a surface choice.
- **Chosen:** hidden from `*`; read by name.
- **What would reverse it:** none expected.

## 6.4 Cycle 16: PostgreSQL as the product (2026-10-01)

### DB-1 — Which SQL, on which engine

- **Alternatives:** Nilestream as the product engine; PostgreSQL as the product engine with Nilestream as the thesis instrument.
- **Criterion:** the engine rule over E27b, per size range.
- **Evidence:** E27 found N justified at no size measured on this host; E27b's first attempt stopped at F-02-3 (DB-2), and its full re-run, after this decision, did not change the verdict (D17-1).
- **Chosen:** PostgreSQL is the product engine; annotated PostgreSQL SQL the product language; Nilestream the instrument and reference REV; Niles the specification. D17-1 scoped it to 10³–10⁵ accounts, k = 1, 2 cores.
- **What would reverse it:** a corrected N that beats P-inc and H3 on both clauses of the engine rule at 10⁶ or at k ≥ 10 on the large host (D17-1).

### DB-2 — E27b after F-02-3

- **Alternatives:** keep the first attempt's points; fix and re-run in full.
- **Criterion:** a verdict from one engine, not a mixture.
- **Evidence:** F-02-3: N answered an empty scalar aggregate with no row where SQL answers one.
- **Chosen:** fix SQL's one-row rule, then re-run E27b in full; the first attempt's 20 points kept as disclosed history.
- **What would reverse it:** none.

### DB-3 — The memory model

- **Alternatives:** keep the cost model; restate it with every term the engine holds.
- **Criterion:** the model counts what the engine holds.
- **Evidence:** E27b's view-state bytes included running aggregates, checkpoints and holes beyond the resident values.
- **Chosen:** the model gains the running-aggregate, checkpoint Θ(H/C), hole and budget terms; reconstruction time is bounded by C at memory Θ(|K| + H/C); LC-31 decided (holes compact to ⊥).
- **What would reverse it:** a measured term outside the four.

### DB-4 — GBS's place

- **Alternatives:** GBS after the engine; GBS as the main lane.
- **Criterion:** the product moves while measurements run beside it.
- **Evidence:** DB-1's answer.
- **Chosen:** the envelope and lending on PostgreSQL as the main lane.
- **What would reverse it:** none.

### DB-5 — What replacing Loan IQ and Calypso means

- **Alternatives:** lending only now; both now (against the recommendation).
- **Criterion:** the author's product goal.
- **Evidence:** none; the author's choice, with its risks recorded (no quant reviewer; the pricing route decides dependencies).
- **Chosen:** both, with a capital-markets track beside lending.
- **What would reverse it:** an author decision.

### DB-6 — The surface and IR growth of C15-05b

- **Alternatives:** revert; freeze.
- **Criterion:** the instrument's semantics stay fixed while it is measured.
- **Evidence:** DB-1 makes Nilestream the instrument.
- **Chosen:** freeze, no revert.
- **What would reverse it:** none.

### DB-7 — Cycle 15's unlanded cards

- **Alternatives:** drop; re-scope.
- **Criterion:** as DA-4.
- **Evidence:** the audit's reading.
- **Chosen:** re-scoped, card by card.
- **What would reverse it:** none.

### DB-8 — The outline's wording

- **Alternatives:** Niles replaces SQL; the author's own wording.
- **Criterion:** the outline says what the project is.
- **Evidence:** the author's statement, confirmed as drafted.
- **Chosen:** "an optimized SQL (plus a dialect), optimized and extended PostgreSQL"; Nilestream and the full Niles grammar are the thesis's instrument and specification.
- **What would reverse it:** an author decision.

### DB-9 — Accessibility in the front-end rule

- **Alternatives:** leave it out; a screen reader must work on every desktop target.
- **Criterion:** operators who use a screen reader can use GBS.
- **Evidence:** none needed; a requirement.
- **Chosen:** added to the rule.
- **What would reverse it:** none.

### DB-10 — The pricing and risk route

- **Alternatives:** build pricing in Rust; ORE/QuantLib as an external engine.
- **Criterion:** correct prices without a quant reviewer, and GBS free of external crates.
- **Evidence:** ORE and QuantLib are mature, BSD-licensed and widely used.
- **Chosen:** ORE as a separate process GBS calls over a narrow protocol; each file an approved download.
- **What would reverse it:** ORE failing to reproduce its own published examples.

### DB-11 — How PostgreSQL is extended

- **Alternatives:** a compiled extension; SQL and PL/pgSQL first.
- **Criterion:** no new dependency until a measurement shows SQL is the bottleneck.
- **Evidence:** none yet that SQL is the bottleneck.
- **Chosen:** SQL and PL/pgSQL first; an extension only after a measurement, as a new decision.
- **What would reverse it:** a profile showing PL/pgSQL as the bottleneck at a size the product targets.

## 6.5 Cycle 17 (2026-10-10)

### D17-1 — DB-1 against the restated goals

- **Alternatives:** reopen the engine question now; confirm DB-1 scoped to what was measured.
- **Criterion:** a decision is stated at the scope of its evidence.
- **Evidence:** E27b at 10³–10⁵ accounts, k = 1, on 2 cores.
- **Chosen:** confirmed, scoped.
- **What would reverse it:** N′ beating P-inc and H3 on both clauses at 10⁶ or at k ≥ 10 on the large host.

### D17-2 — E27c

- **Alternatives:** no E27c; N′-lazy only, against N, P+, H3 and P-inc, on the large host.
- **Criterion:** one change to the engine, pre-registered.
- **Evidence:** DB-3's memory terms.
- **Chosen:** N′-lazy only, after GBS's read-model card.
- **What would reverse it:** E27c's own pre-registered rule.

### D17-3 — GBS's read model

- **Alternatives:** keep P+; adopt P-inc on a probe; a pre-registered card that adopts its winner.
- **Criterion:** the card's rule, written before any run.
- **Evidence:** at the time, one unreplicated probe.
- **Chosen:** the card (G-RM). It adopted P-inc by its tie rule, with daily balance snapshots as the as-of mechanism, after re-running all four arms under one setting.
- **What would reverse it:** the same rule at 10⁶ accounts on the large host.

### D17-4 — The licence coupling

- **Alternatives:** keep GBS linked to an AGPL crate; decouple with GBS's own wire client.
- **Criterion:** GBS's product crates hold no AGPL code.
- **Evidence:** the optional adapter's dependency graph.
- **Chosen:** decouple, add `LICENSE-APACHE`, guard the closure by a test; a lawyer before any distribution.
- **What would reverse it:** a legal opinion that the coupling is harmless, which no one has given.

### D17-5 — The lending sign-off

- **Alternatives:** build on the draft; the author signs each item.
- **Criterion:** no lending rule built unsigned.
- **Evidence:** the author's sign-off of 2026-11-10, with amendments (no floors; lookback by default, with the same day's rate by exception).
- **Chosen:** item-by-item sign-off, then build; a test refuses an implemented item that is not signed.
- **What would reverse it:** an amendment by the author.

### D17-6 — ORE's source

- **Alternatives:** the author fetches; the container fetches the tag, reports size and checksum first.
- **Criterion:** every download is visible before anything reads it.
- **Evidence:** none needed; a process rule.
- **Chosen:** a shallow clone of the tag in its own directory, deleted if refused.
- **What would reverse it:** none.

### D17-7 — Where large measurements run

- **Alternatives:** this container; the author's Mac.
- **Criterion:** 10⁶ and k-sweeps need more than 2 cores and 8 GB.
- **Evidence:** the container's limits; M-NOISE tests whether its noise is a length or a host effect.
- **Chosen:** the Mac, under a quiet-machine protocol, the arm64 host recorded in every header.
- **What would reverse it:** M-NOISE finding the container's refusals a run-length effect, for sizes it can hold.

### D17-8 — The transport

- **Alternatives:** WebSocket and SSE plus gRPC; WebSocket and SSE everywhere.
- **Criterion:** one transport family on every target.
- **Evidence:** none required gRPC.
- **Chosen:** WebSocket and SSE everywhere, the JVM SDK included.
- **What would reverse it:** an integrator that requires gRPC.

### D17-9 — Front-end candidates

- **Alternatives:** A′ as specified; B plus Avalonia for the desktop.
- **Criterion:** the spike's pre-registered rule, a screen reader checked on each target.
- **Evidence:** cycle 17's research on desktop coverage.
- **Chosen:** spike B and Avalonia (Compose kept for Android and the JVM SDK), after G-RM.
- **What would reverse it:** the spike's rule.

### D17-10 — The manual's scope

- **Alternatives:** one chapter a round; two.
- **Criterion:** the manual keeps pace with the build.
- **Evidence:** none; a process choice.
- **Chosen:** two chapters a round, coverage and this catalogue first.
- **What would reverse it:** none.

### D17-11 — The outline

- **Alternatives:** keep the outline in the decision record; paste it into the project description.
- **Criterion:** one place every session reads.
- **Evidence:** none.
- **Chosen:** pasted by the author.
- **What would reverse it:** none.

### D17-12 — The SIMM licence

- **Alternatives:** obtain a licence; none, SIMM not run.
- **Criterion:** ISDA SIMM's methodology is copyrighted and patented, and ISDA requires a licence of every user.
- **Evidence:** ISDA's public SIMM text and licensing FAQ.
- **Chosen:** none held; SIMM is not run.
- **What would reverse it:** a licence held.

### D17-13 — A SIMM engine of GBS's own

- **Alternatives:** none; build the licence-free part now and the SIMM-specific part only under a licence.
- **Criterion:** nothing that needs ISDA's licence is built, run or published without it.
- **Evidence:** as D17-12.
- **Chosen:** in two parts, in the private GBS repository: a licence-free sensitivity and aggregation engine, checked against the public FRTB standardised method; the SIMM-specific part only under a signed vendor licence. No SIMM parameter, formula or test enters this repository.
- **What would reverse it:** none in this repository; the second part depends on a licence, not a measurement.
