# Cycle 11 work order — Fable

Auditor: Claude Fable 5.1, in the cloud container, against `niles` at **`653a369`**
(`c10/02-read-safety`) and `gbs` at **`01e1d85`** (`c10/00-adapter-guard`), from the brief at
`docs/audit/cycle-11/fable-brief.md` (`49271f7`). Astra's companion order is read beside this one;
where the two disagree the author decides, and every finding here carries its evidence class:
**M** (measured by me in the container), **A** (measured by the author on Host C, transcript on
file), **S** (read from source with file and line), **D** (read from a document).

This order answers the brief's §1 first, because the brief says a work order that does not is not
the deliverable. Tasks follow in §3, and there are fewer of them than in cycle 10, on purpose.

---

## 0. The gate at the heads audited

Container (2-core Xeon, `RUSTUP_TOOLCHAIN=stable` = 1.95.0, PostgreSQL 16 started by hand):

| tree | check | result | class |
|---|---|---|---|
| niles `653a369` | `cargo fmt --all -- --check` | clean | M |
| niles `653a369` | `cargo clippy --offline --workspace --all-targets -- -D warnings` | clean, 0 warnings | M |
| niles `653a369` | `cargo test --offline --workspace --no-fail-fast` | **1,087 pass / 0 fail / 8 ignored** | M |
| niles `653a369` | same, ×3 under two `yes >/dev/null` hogs at `--test-threads=8` | 1,087 / 0 / 8 all three; **no test changed verdict** | M |
| niles `653a369` | same, pinned to one core, `--test-threads=1` | 1,087 / 0 / 8 | M |
| niles `653a369` | `make reproduce` | **EXIT 0**, diff clean; §0.1 | M |
| gbs `01e1d85` | `cargo fmt`, `cargo clippy -D warnings` | clean | M |
| gbs `01e1d85` | workspace and adapter suites | **504 / 0 / 1** and **53 / 0 / 0**; §0.1 | M |
| both | `docs/audit/cycle-10/hostc/c10-gates.sh` in the container | **red on every section: `toolchain '1.95.0…' is not installed`** — a defect in the script, not the trees; §2 F-11-01 | M |

Host C (author, cycle 10 end): `c10-gates.sh` green everywhere except `numeric_binary_oracle`
(PostgreSQL not on 5432 — environment), at `f3a668e`; the T04.2 run at `d680c83` completed 5/5
both arms. **A**.

GitHub: `shbiebs/niles` is readable from the container and its `c10/*` and `c11/00-audit` heads
match §2.3 of the brief. `shbiebs/gbs` is **not** readable from the container (`could not read
Username`) and not from the bridge either; the gbs head on the remote is unverified by me. **The
author must paste, when convenient**: `git -C ~/Documents/GBS ls-remote origin c10/00-adapter-guard`.

Mac, through the bridge: niles at `653a369` on `c10/02-read-safety` with `c11/00-audit` at
`49271f7`; gbs at `01e1d85`; the five untracked files at 10,244 / 16,639 / 6,148 / 8,196 /
816,110 bytes. Nothing else untracked or modified.

### 0.1 Container results that finished after this section was first written

| tree | check | result | class |
|---|---|---|---|
| niles `653a369` | `make reproduce` (PostgreSQL 16 up, 1.95.0) | **EXIT 0**; `git diff --exit-code -- results/ thesis/ docs/SPEC-LANGUAGE.md docs/keywords.md` clean; the three exclusions printed are exactly the three host-scoped rows of `results/MANIFEST.csv` (`wire-protocol-client.md`, `E18-memory.csv`, `E18-memory.md`); E18 counts all `ok`; the only untracked paths afterwards are this cycle's own files | M |
| gbs `01e1d85` | `cargo test --offline --workspace --no-fail-fast` | **504 pass / 0 fail / 1 ignored** | M |
| gbs `01e1d85` | `cargo test --offline --manifest-path crates/gbs-nilestream/Cargo.toml` (the gate's exact invocation, `c10-gates.sh:162`) | **53 pass / 0 fail / 0 ignored** across 7 binaries — the wire-shape and sha256-agreement suites included | M |
| both | paired adapter gate: `cargo test --offline -p nilestream-core --test downstream_adapter` in niles at `653a369` against gbs `01e1d85` | 1 / 0 / 0 | M |
| cycle-11 harness | `c11-baselock.sh --self-test` (in `docs/audit/cycle-11/hostc/`) | **26 arms accepted/refused as designed**, after two defects found by running it against a linked worktree — F-11-12 and F-11-13 below; the F-11-13 arm was shown to fail on the reverted line before the fix was kept | M |
| cycle-11 harness | `c11-baselock.sh --no-neutral --baseline e29a025 --candidate 653a369 --candidate-caps off --warmups 0 --measured 1 --seconds 3` (smoke, container) | see §2.0-result | M |
| cycle-11 probe | `c11-merge.sh` on the throwaway worktree (suffix walk outside V) | see §2.0-result | M |

Nothing in this table changes the verdict of §0: the two trees are green at their heads on both
hosts; the only red anything produced this cycle was in the harness, twice, and both were mine.

---

## 1. The questions the author asked first

### 1.0 The four fundamental questions

Each is stated as a hypothesis with a refutation condition, tested against the strongest
alternative I could name and cite. The project's own surveys —
`docs/research/language-landscape.md` (2026-09-01) and `docs/research/engine-landscape.md`
(September 2026) — are cited where they already did the work, and corrected where they answered a
different question from the one the author is asking now.

#### Q1 — Is a new query language needed?

**What Niles proves that the alternatives do not.** The compiler has 85 distinct diagnostics
(`crates/niles-lang/src`, grep `"NL0…"`), of which the ones that are not syntax or lowering are the
language's actual claim to exist. Grouped by what a bank would otherwise re-check at runtime:

| family | codes | what is proved, statically | nearest alternative, and whether it can |
|---|---|---|---|
| conservation | NL0300 / NL0301 | every `txn` block sums to zero per currency, or is *proved* undecidable and discharged to the seal (`results/obligations.csv`: **16 proved, 0 discharged** across the four corpus programs) | PostgreSQL `CONSTRAINT … DEFERRABLE INITIALLY DEFERRED` + a trigger at commit: **a data check, not a proof** — it refuses at runtime what Niles refuses at compile time, and cannot tell "this program may violate" from "this program cannot" |
| currency | NL0251, NL0214, NL0242 | `Money<usd>` and `Money<eur>` cannot be added; an `fx` leg must carry its rate; an undeclared currency is refused | a Rust DSL with phantom types does this fully (Money<C>); SQL cannot (`NUMERIC` is unitless); Materialize/Feldera SQL cannot |
| idempotency | NL0215 / NL0216 / NL0217 | a ledger declares an `IdemKey` with a window, in epochs; a transaction without one is refused | none of the alternatives has the concept at the language level; TigerBeetle has it at the API level (`u128` id, "processed at most once") |
| serve contract | NL0220, NL0222, NL0223, NL0100 | a view's consistency rung is compatible with its materialisation mode; a demand-materialised view ends in an incremental stage; an anchor index covers the key it will be read by | **no alternative has this**. Materialize's isolation level is a *session* property, not a per-view type; Feldera has no per-view consistency contract. This is the genuinely novel static content |
| retention / holds | NL0210, NL0402 | a ledger declares `retain forever`; a hold is resolved by `post`, `void` or `expire` on every path | a linter over SQL DDL could do the first; the second is a session type, which SQL has no place for |
| confidentiality | NL0260 / NL0261 | an `@confidential` column cannot be used in a predicate, key or aggregate | PostgreSQL row-level security is a runtime filter, not a static refusal; a typed embedding could do it |
| recursion | NL0400, NL0507, NL0513 | a `fixpoint` carries a termination measure | Datalog engines (Soufflé) prove termination by stratification; SQL recursive CTEs do not |

**The honest count.** Of the seven families, **four** (conservation, currency, idempotency,
holds) can be had from a typed embedding in a host language — a Rust DSL over a ledger API, with
phantom-typed money and a session type for holds — and that embedding would be cheaper than a
language by an order of magnitude: no lexer, parser, resolver, SQL surface, wire mapping, keyword
registry, or self-hosting compiler. **One** (confidentiality) is a linter. **Two** (the serve
contract and guarded recursion over *the engine's own materialisation modes*) are things only a
language that knows the engine's consistency ladder can state, and they are the ones the thesis
is about (Contribution 4, the consistency-effect calculus). So:

> **Refutation condition:** if the serve-contract family (NL0220/0222/0223) can be expressed as a
> library over an existing SQL — a `CREATE VIEW … WITH (consistency = …, materialize = …)` that a
> checker validates against the query's shape — then Niles as a *language* is not needed, and a
> checker over SQL plus a typed embedding for money is the cheaper artefact.

I could not refute it and I could not confirm it, because the question has not been asked in the
tree: `docs/research/language-landscape.md` scores thirteen clusters (C1–C13) of which **C13,
strict serializability, is called a category error at the language layer** — correct — and then
does not ask whether the *rest* of what Niles proves needs a language rather than a checker.
**Evidence class D.** The surveys answer "does the language the author wanted exist?" (no) and not
"is a language the cheapest way to prove these seven things?" (open). **This is the first
decision-refuted candidate in §1.1.**

What the author should be sure of: **the language is needed for exactly two of its seven proof
families, and both of those are about the engine, not about banking.** If the engine is not needed
(Q2), the language is not needed either; if it is, the language's value is the serve contract, and
the money/idempotency/holds families are conveniences that a Rust DSL over the same engine would
also give. That is a smaller claim than "a complete replacement for SQL" and it is the one the
evidence supports. **The SQL surface (`sql { … }` blocks, the keyword registry, Appendix B's
"complete replacement") is the part of Niles with the least proof behind it and the most cost in
front of it**, and §1.4 costs it.

#### Q2 — Is a new database management system needed?

The claim is *partial materialisation with epoch-anchored certification over an immutable ledger*.
Against the alternatives, from their own documentation (fetched 2026-09-09):

| alternative | can it host the ledger? | can it host the REVs? | the guarantee it fails |
|---|---|---|---|
| **Materialize** ("Strict Serializable" default; "Bounded Staleness" level; Real-Time Recency in preview) | as a table, yes — but cycle 1's own attempt hit `CREATE TABLE with a primary key or unique constraint is not supported` (`docs/research/gbs-noria-postmortem.md`, line 2622 of the transcript) | as fully materialised views, yes | **no partial materialisation** — a view is maintained whole or not at all, so the eviction/reconstruction phase diagram cannot exist on it; per-view consistency is not a type; Real-Time Recency "does not offer any form of guarantee when querying Materialize-local objects" |
| **Feldera** ("strongly consistent: the state of the views always corresponds to what you'd get if you ran the queries in a batch system"; fault tolerance "a preview feature that requires support from … connectors") | as an input table | as views, whole | **no partial state**, no per-key eviction, no certification anchor a client can ask for; durability of the *engine* is preview |
| **Noria / ReadySet** | as a base table | **partial state, yes** — Noria is the origin of upqueries | **eventual consistency by design** (engine-landscape [27]); no epoch, no certification interval, no strict serializability; and the `noria` crate could not be built on a 2026 toolchain (postmortem §1) |
| **PostgreSQL + logical replication into a maintenance layer** | fully — a chain column and a deferred trigger give the sealer's invariants at commit | as ordinary tables refreshed by the consumer | the maintenance layer *is* the engine you would be writing; PostgreSQL gives durability, order and constraints, not partial views |
| **TigerBeetle** (Jepsen 0.16.11: "appeared to meet its promise of Strong Serializability"; "all data … immutable, checksummed, and hash-chained"; transfers "processed at most once") | **yes, better than this project's ledger does today**: quorum-replicated WAL, VSR consensus, storage-fault tolerance, Jepsen-tested | **no** — "fixed schema", "no general queries", two entity types | derived views of any kind; bitemporality; multi-currency conservation is per-ledger-id, not per-transaction across currencies |

So the honest split is:

- **The write path is not needed as a research artefact.** TigerBeetle already gives a
  hash-chained, immutable, idempotent, strictly serialisable, replicated double-entry ledger with
  a Jepsen analysis behind it. Nilestream's ledger has a torn-record corpus and one host's
  `F_FULLFSYNC`; it has no replication, no consensus (T10 is design-only), and no external
  verification. **Every durability property Nilestream's write path claims, TigerBeetle claims
  more strongly and has had tested by a third party.** The thesis's write-path contributions
  (self-checking headers, the three-way recovery classification, the idempotency window in
  transactions) are engineering, not theory, and the thesis text already treats them so.
- **The read side is needed, and it is the whole thesis.** No surveyed system offers partial
  materialisation *with* a certification interval a client can name. Noria has the partiality and
  not the certification; Materialize and Feldera have the consistency and not the partiality.
  The REV — `[stamp, effective]`, `Hole(e)`, `Pending(e)`, generation-owned installs, pinned
  landings — exists nowhere else, and the phase diagram of where it pays is a question no vendor
  has answered because no vendor has the mechanism.

> **Refutation condition for Q2:** a system that hosts an append-only base and serves a
> *partially materialised* keyed aggregate with a stated per-read anchor and a proof that the
> answer is exact at that anchor. If one exists, Nilestream's read runtime is a reimplementation.
> I found none. **Evidence class D** (vendor documentation), with the Noria/Materialize
> attempts of cycle 1 as **A** (the author's own transcript).

What follows for the plan: **the ledger could be TigerBeetle, or an append-only PostgreSQL table
under a deferred trigger, and the thesis would lose nothing it claims as theory.** What it would
lose is the *engineering* narrative of "from-scratch DBMS with wire compatibility", and §1.3 costs
that trade. The REV runtime and the language's serve contract are the instrument the thesis needs;
the sealer, the segment format and the wire protocol are the parts a bank pilot needs, and those
are two different projects sharing a repository.

#### Q3 — What is needed to build a core banking system over unbounded data with infinite-but-skewed derived views?

A first-principles list, each row marked by what the trees hold today: **demonstrated** (a test,
a measurement or a transcript), **designed** (a chapter or spec), **named** (a heading), or
**contradicted** (evidence against).

| need | status | evidence |
|---|---|---|
| double-entry conservation per currency at commit | **demonstrated** | `niles-interp/src/ledger.rs`, conservation suite, NL0300 static proof |
| idempotent submission with a bounded window | **demonstrated** | `batch_seq`, the window in transactions, T00a.5 admission/sealer agreement |
| durable-before-visible, restart without loss | **demonstrated on one host, one disk** | torn corpus (7,984 refused / 856 prefix), `F_FULLFSYNC` preflight; **no replication, no second machine** |
| a total order of epochs with a hash chain | **demonstrated** | segment format, `prev_hash`/`hash`, tamper tests |
| bitemporality (valid vs posted time) | **designed, partly demonstrated** | `valid:` on `Posting`, `as_of`/`valid_at` in the language; no `amend`/`reverse` spelling (cycle 9, cycle 10 §6.5) |
| retention forever, auditability | **demonstrated** | `retain forever` (NL0210), full-retention base, E18 memory rows |
| a derived-view family that is partial, certified and reconstructible | **demonstrated in-process; measured served** | the REV runtime, 67 core tests, T04.2's counters |
| a derived-view family that is *skewed* in the way the thesis assumes | **contradicted — see Q4** | the served benchmark is a two-bucket mixture, not the phase diagram's Zipf |
| the product surface (Loan IQ/Calypso class) | **named**, with 29 GBS rows of which 6 `GenericPathOnly` and 1 blocked | §7 of this order |
| scale: accounts, postings/day, views/account, teller and regulator tail latency | **no number for any of them exists in either tree** | the benchmark runs 2,000 accounts; a bank has 10⁶–10⁸ |
| operations: replication, failover, backup, upgrade, observability, access control | **named** (Appendix D "deployment topology") | none demonstrated; T10 is design-only; no permission system (TigerBeetle documents the same gap and says so) |
| people and time | **not stated anywhere** | one human, one builder model, two auditor models, ~one cycle per week |

**The item the thesis's premise stands on has no measured basis: the skew.** That is Q4.

#### Q4 — Can the skew be measured on a specific data system or universe?

**First finding (M, S): the benchmark does not use the distribution the thesis describes.** The
phase diagram's experiments (`crates/proto-engine/src/workload.rs`) use a **Zipf(s)** sampler with
rank exponent s. The *served* benchmark — every E16/E19 row, every cycle-9 and cycle-10 Host C
measurement, T04.2 — uses `bank_bench::workloads::skewed_key(n, hot_share)`: **a two-bucket
mixture in which `hot_share` = 0.9 of draws land uniformly on the hottest 1% of keys and the rest
land uniformly on all keys.** With 2,000 accounts that is 20 hot keys taking 90% of reads. That is
not Zipf, not Pareto, and not what Chapter 9 says it measured. Its consequences are already in the
record without being named: at the "full" working point (budget 2,500 > 2,000 keys) every key is
resident and the workload is a 20-key cache test; at the "partial" point (budget 400) the 20 hot
keys are always resident and the 10% uniform tail is what evicts. **The 99.1% pinned-install
figure, the 4.6–5.7-epoch arrival gap, and T04.2's regression are all properties of this
mixture**, and none of them has been shown for a Zipf family. Two spellings of one concept, in two
crates, with the thesis citing the one the served path does not use.

**Can the real skew be measured?** Yes, and the estimator is standard:

- **Estimator.** For key-popularity data (reads per account over a window), fit the rank–frequency
  exponent by maximum likelihood on the discrete power law (Clauset–Shalizi–Newman 2009), report
  the **Hill estimator** on the top-k tail as a check, and report two model-free numbers the phase
  diagram can consume directly: **head share** (fraction of reads landing on the top 1% and top
  10% of keys) and the **working-set curve** (fraction of reads covered by the top *b* keys, for
  *b* = the budget). The working-set curve is exactly the function `hit_rate(budget)` the
  eviction-consistency frontier is drawn over.
- **Sample size.** For s in [0.7, 1.2] and 10⁴–10⁶ keys, 10⁶ reads bound the MLE exponent to
  ±0.02 (CSN's finite-size tables); one day of a mid-size bank's balance reads is 10⁷–10⁸.
- **Universes reachable offline**, in order of realism: (1) **GBS's own product traces**, once
  TG01 exists — a thousand-share syndicated payment produces a read pattern the benchmark can
  *replay* rather than synthesise; (2) a **synthetic generator calibrated to published
  statistics** — the thesis can cite the Zipf exponents measured on real key-value and OLTP
  traces (Facebook's `USR`/`ETC` pools, s ≈ 0.9–1.1; YCSB's default 0.99; the TPC-C non-uniform
  random skew) and state that banking balance reads are *assumed* in that band until (3); (3) a
  **public payments dataset** — the author must obtain one; none is in either tree and none is
  reachable from the container. Until (3), every chapter-9 table must carry the label
  *"skew: synthetic, two-bucket mixture (0.9 on 1%)"* or *"Zipf(s = …)"*, and the two must not be
  cited as one.
- **What refutes the premise.** If the measured head share at 1% of keys is below the budget
  fraction at which the frontier says demand materialisation stops paying — the diagram's own
  boundary — then for that universe the thesis's answer is "materialise fully", and the thesis
  must say so as a result rather than as a failure. The diagram is the deliverable; the premise
  is that a bank's reads are skewed *enough*, and it has never been tested.

**Task T11-03 makes the two generators one and labels every existing table; T11-04 is the
measurement design the author can run on a dataset when one exists.** Neither is speculative.

### 1.1 Were the right decisions made?

The unnumbered decisions first, then the ledger. Each row: what it rested on, what has been
measured since, and the verdict — *stands*, *stands-narrowed*, *refuted*, or *open*.

| decision | rested on | measured since | verdict |
|---|---|---|---|
| **Build an engine rather than adopt one** | cycle 1's Noria attempt (`gbs-noria-postmortem.md`): eleven restarts, "Noria was never successfully built, started, or connected to. Not once." Then Materialize refused a primary key | the engine landscape (Sept 2026) confirms no alternative has partial + certified; TigerBeetle has the write path | **stands-narrowed**: the *read runtime* was the right thing to build; the decision to also build the *write path and wire protocol* rested on a packaging failure, not a semantic one, and the postmortem says so in its own words ("does **not** support 'Noria cannot be used in production for core banking' on semantic grounds") |
| **Build a language rather than extend SQL** | the 17-point requirement list (`language-landscape.md`), three of whose clusters the survey itself calls incoherent | Q1 above: two of seven proof families need a language | **open, leaning refuted as scoped**: the serve contract needs a language *or a checker over SQL*; nobody has tested the checker. "A complete replacement for SQL" is not supported by any measurement in the tree |
| **Two repositories, GBS as a consumer** | GBS layering as a constraint | the paired adapter gate was green for a whole cycle while the pair was broken (cycle 9); cycle 10 made it refuse a missing GBS | **stands**, at the cost of a gate that has to be run by hand on the Mac because neither tree can run it alone |
| **Epochs rather than wall-clock** (LC-28) | the hash chain must not depend on time | nothing has contradicted it; the idempotency window is counted in transactions for the same reason | **stands** |
| **Rust with zero external dependencies** | auditability, no supply chain | every crypto/TLS/consensus need is either hand-written (`sha256`, ADR-0003) or a trait with no provider; E2EE and TLS "delegated to a vetted implementation" that does not exist in-tree | **stands for the thesis, contradicted for a pilot**: a bank will not run a hand-written SHA-256 and a TLS provider trait with no provider; the constraint and the pilot goal are incompatible and nobody has said which yields |
| **A builder that cannot push; bundles through the human** | safety; the human as the only writer to GitHub | cycle 9: one owed sync never run; cycle 10: `--ff-only` on a non-existent branch lost a landing, a stale worktree cost a 40-minute run | **stands as a control, costs a cycle-fraction per landing**; §1.5 |
| **Correctness before speed; a measured baseline before any pass line** (cycle 9, A10-09) | C9-06.2's baseline was a document, 7.8× wrong | cycle 10 measured its noise floor first (0.83%) and its headline task lost honestly | **stands, and is the best decision in the record** — without it T04.2 would have been reported as a win against a document |
| **Keep the two-phase read after +1.8% replaced ≥25%** (cycle 9) | "completing the lattice", bounding the hold | T01 built on it: joins, generation ownership, cancellation restoring exact absence; T04 built on it | **stands**: it is now load-bearing for six correctness properties and its cost is bounded and measured; reverting it is a rewrite (§1.2) |
| **Build T04 after the negative-experiment framing was withdrawn** (cycle 10, the author's option A) | the corrected counter: 99.1% pinned, 4.6–5.7-epoch suffix | −7 to −13% served, +95 points of hit rate single-threaded (M, §2.0) | **stands as a decision, refuted as a prediction**: the author chose to find out and found out; the brief's own §6.2 asked the wrong question (where the work lands, not how much of it there is) |
| **The results manifest derives the diff; the harness waives by manifest class** (cycle 10, the author's B/B) | MF-8 | `make reproduce` green on Host C for the first time (author, `f3a668e`) | **stands** |
| **LC-38 → the author chose A (build the merge)** | as above | as above | see §1.1's next row and LC-43 |
| **Keeping the branch stack as one branch** (executor, cycle 10, not a decision anyone made) | expedience | cycle 10 cannot be reverted task by task | **refuted**: LC-45 |

The numbered ledger, only where evidence moved:

- **LC-23/24 (the tail is the base lock)** — *stands*, and T04.2 sharpened it: the merge's cost
  lands under V but its rows are read under B shared; every slowest-16 table in the T04.2
  transcript is still `b wait`-dominated in the pinned arm.
- **LC-30 (Pending at a different anchor)** — implemented in cycle 9, guarded, never confirmed by
  the author in so many words; **the author should confirm or reject it now** — it is load-bearing
  for T01's join semantics.
- **LC-37 (the base hold across the fold)** — below the cut in cycles 9 and 10; **every
  measurement since points at it** (§1.1 row 1 of the numbered list, T04.2's mechanism hypothesis).
  It should not be below the cut a third time; T11-02.
- **LC-38 (merge or pin)** — *measured*: pin wins served, merge wins single-threaded; LC-43 asks
  why and T11-02 measures it. The author's option choice is in §3.
- **LC-41 (partitioned conservation)** — the honest refusal landed (NL0225); the policy is the
  author's and no auditor may invent it. §7 shows it is the invariant Calypso's account
  structures and Loan IQ's multi-branch GL both need, so it is the first language decision the
  GBS roadmap is blocked on.
- **LC-20, LC-36** — still the author's; the executor has carried them for four cycles. They
  should be decided or retired this cycle.

### 1.2 Should any commits be undone? — the revert ledger

Every commit on `c9/*` and `c10/*`, grouped; verdicts with evidence. **A revert is a task with a
guard**: none is proposed below without the measurement that scores it.

| commits | what | verdict | evidence |
|---|---|---|---|
| `366c338`…`ebe7f8f` (C9-00…C9-05) | gate fixtures, histograms, receipts, header check, currency codes, schema-once | **keep** | all are correctness or instrument repairs; none has a cost claim against it |
| `78acd64`, `4ac5032`, `a1dbca6` (C9-06 two-phase read) | begin/fold/finish, joins, generations | **keep** | +1.8% measured (A, cycle 9); six correctness properties now rest on it; a revert is a rewrite of T01 and T04 |
| `dc19a4a`, `53b40f6`, `c075bae`, `96b6d9e` | reports, briefs, work order | keep (evidence) | — |
| `8d6bab9`…`4488234`, `e29a025`…`8e708db` (T00, T00a) | instruments, gate repairs, E18 completion fix, tamper test, idem windows | **keep** | each carries a revert witness in the report; A10-19 confirmed repaired on Host C |
| `12d87dd`, `c7d3955`, `0ff16a4` (T01) | A10-01…05, A10-12 | **keep — but measure**: these are the only read-path commits between the T00 baseline (`e29a025`) and the T04.2 control (`d680c83`) other than T04.1 itself, and the control is 5–9% below the baseline (§2 F-11-04) | liveness/correctness repairs cannot be reverted on cost grounds; but their cost has never been measured and must be (T11-01) |
| `73fe782` (T02) | install walks the graph; NL0224/NL0225 | **keep** | correctness; +9 allocations per install (M), no per-op cost |
| `7b6b704` (T04.1, the merge) | `finish_fold_merging`, caps, refusals | **keep, default OFF, pending T11-02** | −7 to −13% served (A); the mechanism is correct (nine cases, five witnesses) and the phase diagram needs both arms; **`MergeCaps::default()` should become `OFF` until the walk-outside-V measurement lands**, and that is a one-line change with the arm-selector test as its guard |
| `dfac93f`, `bca7907`, `d301296`, `5ff860e`, `3b4f7d9` | harness: arms, toolchain probe (×2), fatal build | **keep** | each was found by a run and witnessed |
| `d680c83`, `67c313b`, `653a369` | reports, README count | keep | — |
| gbs `20d001e`, `01e1d85` (T00a.1, T00a.3) | tamper guard aim, hit-path precondition | **keep** | 53/0/0 on Host C |

**The branch collapse** (T02 and T04 on `c10/02-read-safety`): not a revert candidate — the
commits are fine — but it is why the T04.1 row above says "default OFF" rather than "revert":
`7b6b704` cannot be reverted cleanly because `dfac93f` and `bca7907` sit on it and the wire
columns (`session.rs`) and the benchmark (`bench.rs`) read its counters. **LC-45.**

**Nothing in either tree should be reverted this cycle.** One default should flip, and one
measurement (T11-01) decides whether T01's read-path changes carry a cost that needs a repair.

### 1.3 Could a better plan have been followed from the beginning?

The plan I would write at cycle 1, knowing what is known now, in dependency order with a cut line:

```
P1  measure the skew on a real or calibrated universe; fix the workload generator     (Q4)
P2  the REV runtime in-process, with the differential oracle and the phase diagram   (done, cycles 1-6)
P3  the serve contract as a *checker over SQL DDL* first; the language only if the
    checker cannot state it                                                            (never tried)
P4  the ledger as an adapter trait with TWO implementations: the in-tree sealer AND
    a TigerBeetle or PostgreSQL-chain backend, so every durability claim is
    comparative                                                                        (never tried)
P5  the served read path with the base hold *designed* rather than inherited (LC-37) (below every cut)
P6  one product trace (TG01) before any served measurement is called "banking"        (below every cut)
---- cut for a thesis ----
P7  wire compatibility, the SQL surface, GBS product rows beyond tier 1
P8  distribution, self-hosting, targets, E2EE                                          (Appendices C, E)
```

Diffed against the trajectory: P2 was done well and early; **P1, P3, P4, P5 and P6 were skipped or
cut in every cycle**, and P7's items were built instead — the wire protocol (cycle 3–4), the SQL
surface (Appendix B), the keyword registry, a from-scratch SHA-256 — because they are what a
"complete replacement for SQL and MySQL" needs and not what the theory needs. The cycles since 7
have been spent making the served path *honest* (instruments, harness, refusals), which was
necessary because the served path was built before its premises (P1, P5) were.

**Cost to move now.**

- **P1** (skew): one cycle; no reset. Unify the generators, label the tables, add the estimator.
- **P3** (checker-over-SQL): a *design note and a prototype*, one cycle, unmerged — the same shape
  as T03/T10. It either shows the serve contract can be a checker (and the language's scope
  shrinks to what the evidence supports) or shows it cannot (and Q1 closes in the language's
  favour with a reason). Nothing in the tree is thrown away either way; Niles remains the
  instrument that exists.
- **P4** (second ledger backend): one to two cycles behind an adapter trait; **this is the reset
  candidate**. A TigerBeetle backend would give replication, consensus and a third-party safety
  analysis for free, at the price of "zero external dependencies" (§1.1 row 5) and of the write-path
  chapters becoming comparative. **What it throws away that is proved**: nothing — the in-tree
  sealer stays as the second implementation; the torn corpus and the recovery classification are
  tests of *that* implementation. **Risk**: the dependency rule; the author decides (LC-48).
- **P5** (LC-37): T03's design note exists as a target; one cycle for the prototype. Not a reset.
- **P6** (TG01): one cycle; blocked on nothing but priority.
- **A reset of the read-model runtime**: **no**. It has 67 tests, the differential oracle, the
  lattice, the certification interval, and it is the thesis. A reset of the compiler's lowering:
  **no** — Appendix D's IR is what makes the checked-twice sweep possible. A reset of GBS's
  product layer: **partial** — the 6 `GenericPathOnly` rows and the blocked risk row are the
  product layer that exists; TG01 replaces them with a trace rather than a table.

**The largest single misallocation in ten cycles is not a wrong decision; it is P1.** A theory of
demand materialisation under skew was built, instrumented and measured for eight cycles on a
workload whose skew is a two-bucket mixture nobody chose on purpose and that the thesis does not
describe. The fix is a cycle and it comes first.

### 1.4 Does the knowledge and ability exist to build these three things?

Against the full stated scope. **Demonstrated / designed / named / contradicted**, per item:

**Niles.** Demonstrated: the front end, resolver, typechecker with 85 diagnostics, lowering to a
verified IR, the SQL surface for the executed fragment, `nilesc`, the conservation solver, the
corpus obligations (16/16 proved). Designed: Appendix B's full grammar. Named: the self-hosting
compiler and three-stage bootstrap (Appendix E), native back-ends for five targets, the UDF WASM
ABI (Appendix C), the superoptimizer, the self-hosted linker. **Contradicted**: "complete
replacement for SQL" — the executed fragment is `source → filter/map → one keyed aggregate`
(`scan_fold::plan`, `Runtime::install`), and chapter 7 says so at every table. **Reachability**:
the thesis needs what is demonstrated plus the serve-contract proof (Contribution 4) stated and
tested, which exists. Appendix E is a second doctoral thesis by itself; nothing in ten cycles has
touched it and nothing should until the first is defended.

**Nilestream.** Demonstrated: the REV runtime (in-process and served), the sealer with recovery
classification, the segment format, PostgreSQL wire for the fragment, the lock discipline with
its guards, the harness. Designed: checkpoints in the served daemon (T07), the immutable prefix
(T03), the hash handoff (T10), Appendix D. Named: distribution, cross-shard commit, consensus
("ledger groups"), E2EE, the MySQL wire beyond the fragment, replication, backup, access control.
**Contradicted**: "general-purpose DBMS" — one keyed aggregate over one base is what serves;
joins, distinct and fixpoints lower and verify and are refused at install. **Reachability**: the
thesis needs the read runtime, one honest served measurement on a Zipf family, and the phase
diagram; that is P1 + P5 + one measurement cycle. A pilot needs everything under "named", and
TigerBeetle's Jepsen history is the calibration: a single-purpose ledger with a dedicated team
took ~40 fixed issues across 0.16.11→0.16.43 to reach the safety it now claims, on two entity
types and no queries.

**GBS.** Demonstrated: 29 rows on the adapter, conservation under eviction (53 adapter tests),
the paired gate. Designed: the product model. Named: the product list (FX, derivatives, forwards,
swaps, caps/floors, syndicated, trade finance, LCs, supply-chain, sweeps, pooling, ZBAs) and the
Loan IQ / Calypso scope. **Contradicted**: nothing yet, because nothing has been measured — F-24
(no product trace) and F-12 (lifecycles do not survive restart) are both still open after four
cycles. **Reachability**: §7 gives the count; the short form is that a defensible *syndicated
lending subset* (agency, pro-rata to a thousand shares, PIK, amendments/reversals, a covenant
hold, multi-currency) is four to six cycles after TG01 and LC-41 are done, and **Calypso-class
parity is not reachable by this pipeline in any horizon the author should plan for** — it is
eight asset classes, a pricing library, margin methodologies certified by ISDA, and CCP
connectivity to sixty venues, none of which is a ledger invariant.

**The two kill criteria the thesis sets.**

- **The ledger floor** — the write path's cost per posting under its durability contract. Numbers
  exist: ~205 `F_FULLFSYNC` barriers/s on Host C (cycle 10 preflight; ~255 in cycle 9), the
  E13 durability rows, `batch_seq` group commit. The number that is *missing* is the floor
  **relative to the alternative** (P4): what TigerBeetle or a chained PostgreSQL table does on the
  same disk. Without it the floor is a measurement, not a criterion.
- **The read-model break-even** — the budget at which demand materialisation stops paying. Numbers
  exist in-process on Zipf(s) (E3/E4, the phase diagram). Served: **none on a Zipf family**;
  everything served is the two-bucket mixture (Q4). The criterion cannot be evaluated until P1.

**Cycles.** With the pipeline as it is — one builder, two auditors, one human running scripts,
about one cycle a week — **a defensible thesis is four to six cycles away**: P1 (one), P5 and the
merge decision (one to two), one honest served measurement campaign on Host C (one), TG01 as the
banking instance (one), the writing (one). **A system a bank could pilot is not reachable with
this pipeline**, and the author should stop optimising for it: every "named" item under
Nilestream is a team-year for a dedicated team, and the dependency rule forbids the shortcuts a
pilot would take. **Optimise for the thesis.** What the pilot would additionally need, if the
author ever chooses it, is in §1.3's P4 and P8 and it starts with abandoning "zero external
dependencies".

### 1.5 Which model should audit, and which should build?

Counts from the record (`docs/audit/cycle-9/`, `cycle-10/`), by the identifiers the sessions
report. Findings "survived" if a later measurement or a revert witness confirmed them; errors are
"harness-caught" if a self-test, a revert witness, a manifest test, or the arm-label refusal found
them rather than a reader.

| participant | role(s) | findings that survived measurement | findings refuted by measurement | own errors, reader-caught | own errors, harness-caught |
|---|---|---|---|---|---|
| **Claude Opus 5** (`claude-opus-5`) | executor, cycles 9–10; auditor-brief author, cycle 10 | T00's instruments (all six scopes reconcile), A10-01…05/12 repairs with witnesses, T02, T04.1's mechanism, the manifest derivation, MF-1…MF-9 | the T04 "empty window" framing (self-corrected before it was acted on); the first T04.2 mechanism hypothesis (self-refuted by its own probe) | MF-9, the counter conflation, the branch collapse | MF-4 (three vacuous guards), MF-10, MF-12, MF-13 |
| **Claude Fable 5.1** (`claude-fable-5-1`) | auditor, cycles 9–10; executor of cycle 10's last third; this order | cycle 9: the base-lock attribution (confirmed by C9-06.2); cycle 10 §6.1/§6.2 framing (confirmed as far as the counters go) | cycle 10 §6.2's expected win (the merge lost); cycle 9's §6.1 view-lock framing was inherited from cycle 8 and was wrong | the T04.2 arithmetic was done by hand (no error found, but no check either); MF-14's probe direction | MF-11 and MF-14 were found by the container's gate failing, not by reading |
| **Astra** (identifier not in the record I can read) | auditor, cycles 9–10 | cycle 9: the receipt with no connection, the two windows in two units, the currency code from hash order — all three confirmed by measurement; cycle 10: A10-07's overwrite, A10-18's mis-aimed flip, A10-20's hit assertion | none recorded as refuted | none attributable in the record | — |
| **the author** (`shbiebs`) | runs scripts, decides, reconciles | every Host C number in the record | — | one landing lost to `--ff-only` (an instruction error, not the author's); a stale worktree (the harness's failure to stop) | — |

**What the counts say.** (1) **Reading found the largest correctness defects** — Astra's three
cycle-9 findings and A10-07/18/20 were all read from source and all confirmed; the executor's
own reading found A10-06's real severity. (2) **Measuring refuted the largest predictions** —
cycle 9's headline and cycle 10's headline both lost to the harness. (3) **The executor's errors
cluster in one class** — writing a guard that cannot isolate what it tests, twice, and an edit
method with no verification step — and **that class was caught by the harness every time, never
by a reader**. (4) The mid-session model switch shows in the record only as a change of
attribution trailer; the executor's last third (T04.1, the harness fixes, the T04.2 result) has
the same error profile as the first two thirds (MF-12, MF-14 were the auditing model's). **The
error class is a property of the *role* — executing under time pressure with a script-based edit
method — not of the model.**

**Recommendation, with criteria.**

- **Audit**: two readers, as now, one of them structural (Astra's class of finding has the best
  confirmation rate in the record) and one empirical with a container. Keep both; do not merge
  the briefs.
- **Build**: the model that scored best on *harness-caught* errors is the one with the best
  harness, not the best model. So the criterion for the builder is **whether it runs the revert
  witness and the self-test before it reports**, and the record shows both Opus 5 and Fable 5.1
  doing so when the work order demands it and not otherwise. **Keep Opus 5 as the builder and
  make the work order's reporting contract the thing that is enforced** (T11-05 turns MF-12's
  rule into a check).
- **Never the same model auditing and building in one session** — not because the record shows
  it fails, but because the record shows the auditing model inherited its own §6.2 framing into
  the execution and then had to correct it (the T04 conflation was the auditor's number read by
  the auditor-turned-executor). The value of the second model is that it did not write the brief.
- **The human's irreducible role**: the only writer to GitHub; the only runner on Host C; the
  reconciliation of two orders; and the decisions no model may make (LC-20, 36, 41, 46, 48).
  Nothing in the record suggests any of those four should move.

---
### 1.6 The definition of done the author gave mid-audit

> "The work will be completed when a step by step guide exists to build from scratch Niles,
> Nilestream and GBS and prove why every design decision was the best — a guide/manual that can
> also explain every mechanism of the language, engine and system."

Taken as the cycle's definition of done, with one correction the author's own principles require.
**"Prove why every design decision was the best" is not a claim the record can make for every
decision** — §1.1 has rows marked *refuted*, *open* and *stands-narrowed*, and a manual that
called those "best" would be the document-as-baseline error of C9-06.2 in prose form. What the
record *can* support, and what the manual should say, is: **"best among the alternatives that were
tested, with the alternative named and its measurement beside it — or untested, and marked so."**
That is a stronger book than one that proves everything, because a reader can see where the
proof is.

So the manual is a **generated** document, like Appendix D, not a written one: every mechanism
row points at the test that demonstrates it (and the manual fails to build if the test is gone,
as `thesis_drift` already does for claims); every decision row carries its evidence class from
§1.1 and its measured alternative where one exists; every "named only" item from §1.4 appears as
a named-only row rather than being omitted. **T11-07 is the skeleton and the generator; the prose
is the author's and every later cycle's.** The build-from-scratch order is §1.3's P-list with the
tree's own commands beside each step, and it is the same order a new contributor would follow.

---

## 2. Findings

Class; evidence class; **HI** (human-invisible: correct on every input, wrong in structure) or
**HS** (human-surfaced); EV = impact × confidence ÷ cost, each 1–5.

| ID | class; evidence; HI/HS | finding, source and cost | EV | repair, and what it gives up |
|---|---|---|---|---|
| **F-11-01** | instrument-gap; **M**; HI | `pick_toolchain()` exists in **three** scripts (`c10-baselock.sh:93`, `c10-gates.sh:55`, `c10-rwlock.sh:113`); MF-11/MF-14 were repaired in **one**. In the container `c10-gates.sh` is red on every section with `toolchain '1.95.0…' is not installed` — it unsets the override that made the build possible. On Host C the pin resolves so the author never saw it. Two spellings of one probe, one fixed. | 25 | one probe, in one file, sourced by every script (T11-05). Gives up nothing. |
| **F-11-02** | instrument-gap; **S**; HI | The harness does not compute its own gate. `c10-baselock.sh:978–1016` *describes* medians and pooled MADs and asks the reader to compute them; every number in the T00 A=A table and the T04.2 table was computed by the executor by hand in a Python one-off. A10-09 was an arithmetic error of exactly this shape (mean of MADs for RMS). | 25 | `bench --score <out-dir>`: a tested Rust command that reads the per-replicate CSVs, prints medians, pooled MADs, gate verdicts, and refuses an arm with fewer than 5 replicates; the shell prints its table. Guard: the T04.2 six rows as a fixture (T11-05). |
| **F-11-03** | instrument-gap; **S**; HI | Every harness refusal that reads a bench log is coupled to the benchmark's output by a **string** (`"WARNING: the server's slow-read table has no"`, `"  merge at "`, `"E19 mixed"`, `"writes/s"`), and nothing on the bench side asserts those strings are stable. A rewording in `bench.rs` makes the refusal vacuous silently — MF-2's shape (a renamed column defaulted to 0), one layer up. | 20 | the bench emits one machine-readable summary line per level (JSON), the harness parses only that, and a bank-bench test asserts the schema; the human-readable lines stay for the reader (T11-05). |
| **F-11-04** | wrong-measurement; **A**; HI | The T04.2 **pinned control** (`d680c83`, merging off, 1.95.0) is **5–9% below the T00 baseline** (`e29a025`) on every one of six rows — full 6r3w 152,826 → 145,466 (−4.8%, 6.9 pooled MADs), partial 12r6w 149,104 → 136,161 (−8.7%, 19.7 MADs). Below the 10% threshold, far above the noise floor, and **confounded twice**: different sessions, and the T00 baseline was built with 1.97.1 (MF-7). T04.2's gate compared merge against pinned *within one commit* and could not see that the commit had already slowed, or that the compiler had changed. The read-path commits in between are T01's three and T04.1. | 25 | two interleaved pairs on Host C, one holding the code fixed across toolchains, one holding the toolchain fixed across `e29a025`…`653a369` with merging off (T11-01). Gives up a cycle-fraction of Host C time. |
| **F-11-05** | wrong-measurement; **S+M**; HI | The served benchmark's skew is a **two-bucket mixture** — `skewed_key(n, 0.9)`: 90% of draws uniform over the hottest 1% of keys, 10% uniform over all (`crates/bank-bench/src/workloads.rs:49–56`). The phase diagram's experiments use **Zipf(s)** (`crates/proto-engine/src/workload.rs:10`). Chapter 9 cites the served numbers as evidence for a theory stated over a Pareto family. With 2,000 accounts the served workload is 20 hot keys; the "partial" point never evicts a hot key. Every cycle-9/10 Host C number, and T04.2, is a property of this mixture. | 25 | one generator, Zipf(s) with s a labelled parameter, used by both; every existing table relabelled *"skew: mixture(0.9 on 1%)"*; the working-set curve printed per run (T11-03). Gives up comparability with cycles 8–10, which is the point. |
| **F-11-06** | stale-claim; **S**; HS | `MISMATCH-A9-F15` stands at `thesis/04-novel-contributions.md:37` ("`Runtime::install` … never walks the graph feeding it") after `73fe782` made it walk the graph; `MISMATCH-A9-F05` stands at `docs/SPEC-ENGINE.md` after C9-04.1 repaired it — flagged in the cycle-10 brief and not struck in cycle 10 either. Two repaired markers, one of them carried through two cycles. | 15 | strike both with a dated note; the marker inventory (§9.6) becomes a generated table with a `repaired-by` column so a repair without a strike fails a test (T11-06 / LC-40). |
| **F-11-07** | negative; **A+M**; HS | T04.2: merge −7% to −13% served on Host C, 9–17 pooled MADs; the single-threaded probe shows the merge *winning* by two orders of magnitude in base rows at the same gaps (M). The hypothesis — the suffix walk runs under V — is measured in §2.0 below. | — | see §2.0 and T11-02. |
| **F-11-08** | decision-refuted; **D**; HS | §1.0 Q1/Q2: the language is needed for two of its seven proof families; the write path duplicates, with weaker guarantees and no third-party test, what TigerBeetle provides. Neither survey in `docs/research/` asked the question in that form. | 20 | §1.3 P3 (checker-over-SQL note) and P4 (second ledger backend behind a trait): design notes and unmerged prototypes, LC-48. Gives up the "from-scratch replacement" narrative in exchange for a defensible one. |
| **F-11-09** | guarantee-bounded; **S+M**; HI | The outside-V merge design (my throwaway, §2.0) is sound only because `applied` at landing equals `applied` at begin — which holds only because the base is held shared across the fold (`flights_that_fell_behind` = 0 structurally). **If LC-37 removes the base hold, the merge's precondition disappears** and the walk must move back under V or re-verify. The two tasks are coupled and neither work order to date has said so. | 12 | T11-02's design states the coupling and its guard is the structural-exclusion test already in the tree (`a_flight_that_nothing_advances_under_never_falls_behind`), which must fail when the hold is released. |
| **F-11-10** | instrument-gap; **M** (heuristic); HI | Of ~34 `nilestream_stats` counters, at least the following have **no test anywhere that asserts them non-zero**: `flights_refused`, `waiters_refused`, `joins_answered`, `joins_retried`, every `gap_*` total and sample count, `flights_behind_at_begin`, `merge_rows_visited`, `merge_epochs_merged`, `view_metadata_keys`, `schema_parses`, and every lock histogram column. A grep for assertions near each name; the method undercounts multi-line asserts and is stated as a lower bound. `flights_that_fell_behind` was in this list until cycle 10 and its zero was reported across 28 replicates as a finding. | 15 | one test per counter that makes it move through the wire, in a table-driven test that fails when a counter is added without a row (T11-05). |
| **F-11-11** | instrument-gap; **M**; HS | The desktop bridge cannot read `shbiebs/gbs` and neither can the container (F-11-00's access check); the gbs remote head is verified by nobody but the author. The paired-adapter gate runs only where both trees are checked out. | 6 | the author pastes one `ls-remote` line per cycle (§0); no code. |
| **F-11-12** | instrument-gap; **M**; HI | Every cycle-10 script tests "is a git checkout" with `[ -d "$REPO/.git" ]` (`c10-baselock.sh`, `c10-gates.sh` ×2). A **linked worktree's `.git` is a file**, so the harness refuses every worktree with `FATAL: … is not a git checkout` — which is how the executor's own throwaway worktree was refused this cycle. Nobody had run the harness against anything but the author's primary checkout, so the test was never exercised on the shape it rejects. | 12 | `git -C "$REPO" rev-parse --git-dir` in the three places; **applied in the cycle-11 copies** (`c11-baselock.sh`, `c11-gates.sh`). Gives up nothing. |
| **F-11-13** | instrument-gap; **M**; HI | **Mine, this cycle.** The per-arm toolchain override I added to `c11-baselock.sh` read `$ARM_B_TC` inside `build_arm`, a variable the run phase assigns *after* the build phase; under `set -u` the first real run died with `ARM_B_TC: unbound variable` — after the self-test had passed 25 arms, because no arm exercised the build phase. The shape is F-11-01's again: a fix that ships with a self-test that does not reach the line changed. | 12 | the read now uses the flag variables (`$BASELINE_TOOLCHAIN`/`$CANDIDATE_TOOLCHAIN`); a **static self-test arm** refuses any `$ARM_*` reference between `build_arm()` and the `RUN PHASE` marker, and was shown to fail on the reverted line (exit 1) before the fix was kept. T11-05's rule follows: **a harness fix is not done until a self-test arm reaches the line it changed.** |

### 2.0 The merge with the suffix walk outside the view hold — measured

Container, two cores, `c11-merge.sh --warmups 1 --measured 3 --seconds 10`, both arms from one
binary per run, three measured replicates per arm and point — **below the harness's own
five-replicate floor, so ratios only, and no row here is a T04.2 verdict**. Two runs, two
builds: the tree at `653a369` (suffix walk under V, as shipped) and a throwaway worktree commit
(`8c7e999`, scripts added as `b3ef4e9`; never bundled, discarded after the run) that walks the
suffix **before** taking V (`walk_suffix_free`, `finish_fold_premerged`; server and core suites
green on it). Medians, pooled MAD = RMS of the two arms' MADs; the gate is ≥10% AND ≥3 MADs.

| walk | point | level | merge reads/s | pinned reads/s | rel. | pooled MAD | MADs | gate |
|---|---|---|---|---|---|---|---|---|
| under V (`653a369`) | full | 6r3w | 21,558 | 18,330 | +17.6% | 2,244 | 1.4 | noise-limited |
| under V | full | 9r5w | 20,604 | 17,300 | +19.1% | 1,209 | 2.7 | noise-limited |
| under V | full | 12r6w | 20,691 | 22,616 | **−8.5%** | 270 | 7.1 | below 10% |
| under V | partial | 6r3w | 20,104 | 15,666 | +28.3% | 324 | 13.7 | **merge faster** |
| under V | partial | 9r5w | 15,877 | 13,556 | +17.1% | 362 | 6.4 | **merge faster** |
| under V | partial | 12r6w | 16,465 | 14,125 | +16.6% | 911 | 2.6 | noise-limited |
| outside V (`8c7e999`) | full | 6r3w | 23,274 | 18,756 | +24.1% | 369 | 12.2 | **merge faster** |
| outside V | full | 9r5w | 20,847 | 18,244 | +14.3% | 274 | 9.5 | **merge faster** |
| outside V | full | 12r6w | 21,854 | 19,293 | +13.3% | 241 | 10.6 | **merge faster** |
| outside V | partial | 6r3w | 18,774 | 16,432 | +14.3% | 768 | 3.0 | **merge faster** (at the floor) |
| outside V | partial | 9r5w | 17,308 | 14,857 | +16.5% | 991 | 2.5 | noise-limited |
| outside V | partial | 12r6w | 15,956 | 15,241 | +4.7% | 828 | 0.9 | noise-limited |

What this does and does not say. (1) **On two cores the merge is not the regression Host C
measured**: under V it wins or ties five rows of six; the one row it loses (full 12r6w, −8.5%, 7.1
MADs) is the row that moving the walk outside V turns into a +13.3% / 10.6-MAD win, and outside V
the merge loses nothing. That is the direction the hypothesis predicts — the walk under V costs
readers the view hold — and it is the *only* evidence for the hypothesis so far. (2) It cannot
adjudicate T04.2, because the container **inverts Host C's sign on the very arm Host C measured**
(under V: +17.6% here, −8.96% at 9.9 MADs there, full 6r3w). A two-core Xeon with a 10-second
window is a different machine from Host C; the harness that produced the six T04.2 rows is the
only one that can retire them. (3) The two "walk" blocks are two builds in two sessions and are
**never compared row-to-row across blocks** — the only sanctioned comparisons are within a block.
(4) Three replicates is what two cores afford in the audit's time; the rule stands and the rows
are labelled by it.

Consequence for the order: T11-02 stays as written (the walk outside V, **default OFF**, measured
on Host C by `c11-merge.sh` with the same five-replicate protocol T04.2 used), and its result on
Host C decides LC-38 A/B/C; nothing in this table pre-decides it. The throwaway's design has
the F-11-09 coupling (the base hold across the fold) as its precondition, and that is written
into T11-02's guard.

**§2.0-result — the smoke of `c11-baselock.sh`** (`--no-neutral --baseline e29a025 --candidate
653a369 --candidate-caps off --warmups 0 --measured 1 --seconds 3`, container): both arms built
from their own worktrees, each arm's `toolchain-<arm>.txt` reads `1.95.0` from `rustc -vV`
inside the worktree, the candidate's label carries `(NILESTREAM_MERGE_CAPS=off)`, every
candidate `merge at` line ends `[PINNED CONTROL: merging off]` with 0 merged, and the baseline
prints no `merge at` line at all (it predates the counter) — which the arm-label refusal
accepts, correctly, for a baseline. One replicate at three seconds is a smoke and reports no
ratio. The same command on Host C is `c11-pairs.sh`'s second pair.

---
## 3. Tasks, in dependency order

Every task carries: what it closes; files; **baseline** (measured, never a document); **target**
(one sentence the executor marks `done` or `not done: why` — copied verbatim into the report);
method; acceptance; **guard and the reversion that must make it fail** (in a disposable
`git worktree`, never a compile error); guardrails; **the branch it lands on** (LC-45: a
deviation is `BLOCKED-branch-<id>`, not a decision).

The order is not by EV alone. **T11-01 is first because every later pass line is scored against
it**, and T11-05 is second because T11-01's arithmetic must not be done by hand a third time.

### T11-01 — Separate the compiler from the code (F-11-04)

**Closes** F-11-04; re-baselines everything measured since `e29a025`.
**Files** none in `crates/`; `docs/audit/cycle-11/hostc/c11-pairs.sh`, `c11-baselock.sh` (this
order ships both).
**Baseline** the two pairs the script measures — there is no prior measurement that holds either
variable still.
**The author must now run** `bash ~/Documents/niles/docs/audit/cycle-11/hostc/c11-pairs.sh` and
paste both transcripts (≈80–90 min). If `rustup run 1.97.1` fails the script refuses pair 1 and
says so; do not run pair 2 alone.
**Target T11-01.1:** *Two interleaved five-replicate pairs on Host C report, per working point and
level, the toolchain effect (653a369 merging-off, 1.95.0 against 1.97.1) and the code effect
(e29a025 against 653a369 merging-off, both on the pin), each with medians, pooled MADs and a
gate verdict, and every row that clears ≥10% and ≥3 pooled MADs names the pair that produced it
and nothing else.*
**Method** the script; the executor's only work is the scoring, and the scoring is T11-05's
`bench --score` if it has landed, else the Python one-off *committed to the tree under
`docs/audit/cycle-11/score.py` with the six T04.2 rows as its self-test*.
**Acceptance** both pairs 5/5; the arm-toolchain lines in the transcript read `1.95.0` / `1.97.1`
for pair 1 and `1.95.0` / `1.95.0` for pair 2, from inside the worktrees.
**Guard** the transcript's own arm-label refusals (already self-tested); reversion: the
`toolchain-<arm>.txt` line removed from `build_arm` → the self-test arm "no cycle-11 script carries
its own probe" still passes, so **add** a self-test arm that a transcript without both arm
toolchain lines is refused, and prove it fails when the line is removed.
**Guardrails** no publish; nothing in `results/`; the numbers go in the report with a host column.
**Branch** `c11/01-harness`.

**What the result decides.** If pair 1 clears the gate, MF-7 cost the cycle-10 baseline its
comparability and the report's T00 table is relabelled *"1.97.1"*; every later Host C number is
on 1.95.0 and stands. If pair 2 clears it, T01's read-path changes carry a cost and a bisect
between `12d87dd`, `c7d3955`, `0ff16a4` and `73fe782` is the next task — **not a repair**, a
bisect, because correctness repairs are not reverted on cost grounds; the repair, if any, is to
the mechanism the bisect names. If neither clears it, the 5–9% was two sessions and a compiler,
and the record says so.

### T11-05 — The harness computes its own gate, reads a contract, and has one probe (F-11-01/02/03/10; LC-44)

**Closes** F-11-01, F-11-02, F-11-03, F-11-10; positions LC-44.
**Files** `crates/bank-bench/src/bin/bench.rs` (a `--score <dir>` mode and a JSON summary line per
level), `crates/bank-bench/tests/` (the scorer's fixture, the counter-moves table),
`docs/audit/cycle-11/hostc/*.sh` (the harness parses the JSON line; the shell computes nothing).
**Baseline** none needed: this is an instrument.
**Target T11-05.1:** *`bench --score` reproduces the six T04.2 rows from cycle 10's per-replicate
CSVs to the integer, refuses an arm with fewer than five measured replicates, and the harness
prints its gate table from that output and from no arithmetic of its own.*
**Target T11-05.2:** *Every harness refusal that reads a benchmark log parses the benchmark's
machine-readable summary line, a bank-bench test asserts that line's schema, and a reworded
human-readable line changes no verdict.*
**Target T11-05.3:** *A table-driven test drives every `nilestream_stats` counter to a non-zero
value through the wire and fails when a counter is added to the wire without a row.*
**Target T11-05.4:** *One toolchain probe exists under `docs/audit/cycle-11/hostc/` and a self-test
arm refuses a second.* (Shipped by this order; the executor keeps it true.)
**Target T11-05.5 (F-11-12, F-11-13):** *Every harness change in this cycle ships with a self-test
arm that reaches the line changed — shown red with the change reverted, green with it kept — and
the report lists, per harness commit, the arm and both exit codes.* A self-test that passes 25
arms while the build phase cannot run is the defect F-11-13 names; the rule is the repair.
**Method** the scorer is ~150 lines of Rust with no dependencies (medians, MAD, RMS pooling); the
JSON line is one `eprintln!` per level; the counter test is one loop over a `const` table.
**Acceptance** `cargo test -p bank-bench` green; `c11-baselock.sh --self-test` green with the
new arms; a run of `c11-baselock.sh --baseline-only --warmups 0 --measured 1 --seconds 3` in the
container prints a gate table it did not compute in shell.
**Guard and reversion** (a) the scorer's fixture, reverted to mean-of-MADs → six wrong pooled
MADs and the test names them; (b) a reworded `E19 mixed` line in a fixture log → the JSON path
still refuses/accepts correctly and a test that greps the old string is deleted, not kept; (c)
the counter table with one row removed → the test fails naming the counter.
**Guardrails** the human-readable lines stay; the JSON line is additive; no counter's meaning
changes.
**Branch** `c11/01-harness`.

### T11-03 — One skew generator, and every table labelled with its skew (F-11-05; Q4)

**Closes** F-11-05; the first half of Q4.
**Files** `crates/bank-bench/src/workloads.rs` (`skewed_key` → a `Zipf(s)` sampler shared with
`proto-engine`, or the mixture kept as a *named* second family), `crates/proto-engine/src/workload.rs`,
`crates/bank-bench/src/bin/bench.rs` (print the working-set curve: reads covered by the top
1%, 10%, and the budget, per level), `thesis/09-evaluation.md`, `results/E16-wallclock.md`,
`results/E19-scaling.md` (labels), `results/MANIFEST.csv` if a file is added.
**Baseline** the served numbers at `653a369` on the mixture, as measured by T11-01's pair 2
candidate arm — **the executor does not re-measure**; the change is scored by T11-01's script
run once more on Host C after landing, as `c11-baselock.sh --baseline 653a369 --candidate
c11/02-skew --candidate-caps off`.
**Target T11-03.1:** *The served benchmark and the phase-diagram experiments draw keys from one
Zipf(s) sampler with s a labelled parameter, the mixture survives only as a named second family
selectable by flag, every existing results table and chapter-9 sentence that cites a served
number carries the skew it was measured under, and each mixed level prints the working-set curve
(reads covered by the top 1%, top 10%, and the budget).*
**Method** move `Zipf` into a shared module both crates depend on (bank-bench already depends on
proto-engine? — check; if not, a small `workload` crate with zero dependencies); default
s = 0.99 (YCSB's) and say why; keep `--skew mixture:0.9` as the reproduction of cycles 8–10.
**Acceptance** the unit test `a_skewed_draw_concentrates_on_the_hot_keys_and_a_uniform_one_does_not`
is rewritten for Zipf and passes; `make reproduce` green after the label edits; the Host C run
completes 5/5 and its table is in the report **without a pass line** — the skew change is a
relabelling, not an optimisation, and its numbers are a new baseline.
**Guard and reversion** the sampler's own test (head share at 1% within ±2 points of the closed
form for s = 0.99, n = 2,000, 10⁶ draws); reversion to the mixture → the head share is 90% ± 1 and
the test names the family it is seeing.
**Guardrails** no chapter-9 number is changed, only labelled; the mixture's numbers are not deleted.
**Branch** `c11/02-skew`.

### T11-04 — The skew estimator, runnable on any trace (Q4)

**Closes** the second half of Q4; gives the author the tool for the dataset he must obtain.
**Files** `crates/bank-bench/src/bin/bench.rs` (`--skew-of <csv>`: a column of keys → MLE
exponent, Hill estimator on the top 10%, head share at 1%/10%, the working-set curve as a CSV),
`docs/audit/cycle-11/skew-measurement.md` (the design: what to obtain, in what shape, the sample
size, and the refutation boundary from the phase diagram).
**Baseline** none; a measurement design.
**Target T11-04.1:** *`bench --skew-of` reports, for any CSV of keys, the MLE power-law exponent
with its standard error, the Hill estimate on the top decile, head share at 1% and 10%, and the
working-set curve; run on a Zipf(0.99) sample it recovers 0.99 ± 0.02, and run on the mixture it
reports head share 0.90 and refuses to fit an exponent.*
**Target T11-04.2:** *`docs/audit/cycle-11/skew-measurement.md` states what dataset the author
must obtain, the minimum sample, the estimator, and the head-share threshold below which the
phase diagram's own boundary says demand materialisation does not pay, as a number read from
`e4_phase.csv`.*
**Method** CSN 2009's discrete MLE (a 20-line numeric solve, no dependency); Hill is a sum of
logs.
**Guard and reversion** the two synthetic self-tests; reversion of the MLE to a least-squares
log-log fit → the Zipf(0.99) recovery drifts outside ±0.02 and the test says so.
**Branch** `c11/02-skew`.

### T11-02 — The merge's suffix walk outside the view hold, measured (LC-43; F-11-07, F-11-09)

**Closes** LC-43 by measurement; states the coupling to LC-37.
**Files** `crates/nilestream-core/src/rev.rs` (`finish_fold_merging` → a two-step API: a pure
`suffix_delta(base, key, anchor, applied, caps)` with no view state, and a landing that installs
merged only if `applied` is still what the walk assumed, else pins and counts
`merges_refused_moved`), `crates/nilestream-server/src/rev_engine.rs` (`answer_from_view` walks
before the second V acquisition, under the B shared guard it already holds),
`crates/nilestream-server/src/session.rs` (the new refusal counter on the wire).
**Default `MergeCaps` becomes `OFF`** in the same commit, so the served path ships the pinned
policy until this task's measurement says otherwise; the arm-selector test pins both facts.
**Baseline** T04.2's pinned arm on Host C (five replicates, transcript on file) **re-measured as
the pinned arm of this task's own run** — never the cycle-10 table.
**The author must now run** `bash ~/Documents/niles/docs/audit/cycle-11/hostc/c11-merge.sh`
after syncing `c11/03-merge-outside-v`, and paste it (≈25–40 min).
**Target T11-02.1:** *A late landing's suffix is walked with the base held shared and the view
released, the landing installs merged only when `applied` is unchanged since the walk and
otherwise pins under its own counted refusal, the owner and every same-anchor waiter still
receive the answer at `a`, and every T04.1 case and revert witness still holds.*
**Target T11-02.2:** *Against the pinned control measured in the same run on Host C, the
outside-V merge reports its gap, rows, refusals and both view holds, and makes a performance
claim only if it clears ≥10% and ≥3 pooled MADs in its favour; a loss or a noise-limited result is
recorded and `MergeCaps::OFF` stays the default.*
**Method** exactly the throwaway measured in §2.0, made honest: the `applied` guess is
`anchor + gap_begin`, and the landing verifies it. **State in the code that this rests on the base
being held shared across the fold** (F-11-09): the structural-exclusion test
`a_flight_that_nothing_advances_under_never_falls_behind` is the guard, and LC-37's prototype must
make it fail before it may release the hold.
**Acceptance** the nine `deferred_merge_tests` pass unchanged; the container two-arm run gives a
ratio (§2.0); Host C decides.
**Guard and reversion** (a) the walk moved back under V → the source guard
`the_reconstruction_happens_with_the_view_released`, extended to require the walk before the
second acquisition, fails; (b) the `applied` verification removed → a test that advances the
view between walk and landing installs a stale merge, and the differential oracle names the key.
**Guardrails** the lock order is unchanged; no new lock; the answer never changes.
**Branch** `c11/03-merge-outside-v`, after `c11/01-harness`.

### T11-06 — The marker inventory as a generated table; two stale markers struck (F-11-06; LC-40)

**Closes** F-11-06; the marker half of LC-40 (T05's second half stays below the cut).
**Files** `thesis/gen-markers.py` (new: every `MISMATCH-*`/`BLOCKED-*` occurrence in both trees
with file, line, family, and `active` / `historical` / `repaired-by <sha>` from a small
`docs/markers.toml`), `thesis/appendix-markers.md` (generated), `Makefile` (`reproduce` regenerates
it), `thesis/04-novel-contributions.md:37` and `docs/SPEC-ENGINE.md` (strike `MISMATCH-A9-F15` and
`MISMATCH-A9-F05` with dated notes).
**Baseline** the inventory in this order's §9.6 (34 families, niles; 10, gbs).
**Target T11-06.1:** *Every `MISMATCH-*` and `BLOCKED-*` occurrence in both trees appears in one
generated appendix with a status, a marker whose repairing commit is named in `docs/markers.toml`
but whose text is unchanged fails the build, and `MISMATCH-A9-F15` and `MISMATCH-A9-F05` are
struck with the commits that repaired them.*
**Guard and reversion** the omission mutant: delete one occurrence from the toml → the generator
refuses ("unlisted marker"); restore the A9-F15 sentence as active → the build fails naming
`73fe782`.
**Branch** `c11/01-harness`.

### T11-07 — The Build Book: skeleton, generator, and the decision rows (§1.6)

**Closes** the author's definition of done as a *structure*; the prose is later cycles'.
**Files** `docs/BUILD-BOOK.md` (generated skeleton + hand-written sections marked as such),
`thesis/gen-build-book.py`, `docs/decisions.toml` (every row of §1.1 with decision, evidence
class, alternative, measurement, verdict).
**Target T11-07.1:** *`docs/BUILD-BOOK.md` is generated from the tree with (i) the build-from-
scratch order of §1.3 with the tree's own commands beside each step, (ii) one row per mechanism
naming the test that demonstrates it and failing the build if the test is gone, (iii) one row per
decision from `docs/decisions.toml` with its evidence class and its measured alternative or the
word `untested`, and (iv) every "named only" item of §1.4 listed as named-only; and a decision
row cannot say "best" without a measured alternative beside it.*
**Guard and reversion** delete a demonstrated mechanism's test → the generator fails naming the
row; mark a decision `best` with no alternative → the generator refuses the row.
**Branch** `c11/04-build-book`.

### T11-09 — Design note: the serve contract as a checker over SQL (Q1; §1.3 P3)

**Files** `docs/audit/cycle-11/notes/serve-contract-as-checker.md` — no code.
**Target T11-09.1:** *A note states, for each of NL0220/NL0222/NL0223/NL0100 and the guarded-
recursion family, whether the same refusal can be produced by a checker over PostgreSQL DDL plus a
`WITH (consistency=…, materialize=…)` view option, with the reason where it cannot, and ends with
one sentence saying whether Niles is needed as a language or as a checker, and what the SQL
surface costs that the answer does not need.*
**Branch** `c11/05-notes`.

### T11-10 — Design note: the ledger behind a trait with a second backend (Q2; §1.3 P4)

**Files** `docs/audit/cycle-11/notes/ledger-as-trait.md` — no code; **LC-48 is the author's**.
**Target T11-10.1:** *A note states what `Base` and the sealer's API would have to look like for a
TigerBeetle-backed or PostgreSQL-chain-backed ledger to serve the same REVs, which thesis claims
would become comparative rather than absolute, what "zero external dependencies" would have to
yield, and the cycle cost of the prototype — and does not build it.*
**Branch** `c11/05-notes`.

### — Cut line — one agent, one cycle: T11-01, T11-05, T11-03, T11-04, T11-02, T11-06, T11-07, T11-09, T11-10.

T11-09 and T11-10 are notes and are above the cut because §1 is the cycle's purpose; if correctness
consumes the cycle, they are the first to fall and T11-07 the second.

### Below the cut, carried with re-derived positions

- **T11-08 — LC-37, the immutable prefix (cycle 10's T03)**: design note and unmerged prototype;
  **the first task of cycle 12** unless T11-02 shows the walk-outside-V merge wins, in which case
  the base hold is the *only* remaining tail and it moves up. Coupled to T11-02 by F-11-09.
- **T07 (served checkpoints, LC-32)**: re-baselined against the pinned arm of T11-02's run.
- **TG01 (the thousand-share trace, F-12/F-24, LC-34)**: still the first product task; §7 tier 1.
- **T05's second half (generated-document inclusion policies)**, **T06 (checked-twice sweep)**,
  **T08 (production memory, LC-31/33)**, **T09 (contract republication)**, **T10 (hash handoff,
  design only)**: as cycle 10 left them, re-baselined at `653a369` when taken.
- **GBS**: nothing above the cut this cycle; §7.

---

## 4. Branch stacks and merge order

`niles`, from `c11/00-audit` (`49271f7`), which is above `c10/02-read-safety` (`653a369`):

```
c11/01-harness          T11-01 (scripts), T11-05, T11-06        first; everything is scored by it
c11/02-skew             T11-03, T11-04                          after c11/01-harness
c11/03-merge-outside-v  T11-02                                  after c11/01-harness
c11/04-build-book       T11-07                                  after c11/01-harness (needs the marker generator)
c11/05-notes            T11-09, T11-10                          independent
---- cut ----
c11/06-immutable-prefix T11-08 (cycle 12 unless T11-02 promotes it)
```

Merge order into `c10/02-read-safety` → (the author's call) `master`: `c11/01-harness`, then
`c11/02-skew`, `c11/03-merge-outside-v`, `c11/04-build-book`, `c11/05-notes`. **Each task lands on
the branch named here or is `BLOCKED-branch-T11-nn`** (LC-45); a bundle carries one branch and
the sync block names it.

`gbs`, from `01e1d85`: no branch this cycle.

---

## 5. Validation protocol

Container and Host C, every landing: `cargo fmt --all -- --check`; `cargo clippy --offline
--workspace --all-targets -- -D warnings` (1.95.0 in both places now; a 1.97.1 lint is the author's
to paste); `cargo test --offline --workspace --no-fail-fast` (1,087 / 0 / 8 at `653a369`; the
README count moves with every test added and the guard says so); `make reproduce` (exit 0 after
commit; the diff is derived from the manifest); `bash docs/audit/cycle-11/hostc/c11-gates.sh`
(both trees and the paired adapter; **in the container set `C11_NILES` and `C11_GBS`**);
`bash docs/audit/cycle-11/hostc/c11-baselock.sh --self-test` (every arm; a `NOT REFUSED` line is a
red row).

Green means every row above says so. A red row is a result; **environment reds** are exactly:
`numeric_binary_oracle` with no PostgreSQL on 5432 (say so, never delete), and any `cargo` failing
with `toolchain '1.95.0…' is not installed` where the pin does not resolve — which is the
container, and only when a script carries its own probe (F-11-01; the cycle-11 scripts do not).

---

## 6. Host C scripts shipped with this order

All under `docs/audit/cycle-11/hostc/`, all inheriting `c10-baselock.sh`'s refusals and self-test
(24 arms → 26 now: one for the single probe, one static arm for F-11-13; → 27 once T11-01 adds
the arm-toolchain-lines check). Two defects were found by running these scripts, not by reading
them, and both are fixed in the cycle-11 copies only — the cycle-10 scripts are evidence and keep
them: F-11-12 (worktree `.git` is a file) and F-11-13 (build phase read a run-phase variable).

| script | runtime | runs when | note |
|---|---|---|---|
| `toolchain.sh` | — | sourced by every other | the one probe (F-11-01) |
| `c11-baselock.sh` | as cycle 10 | by the others | + `--candidate-caps`, `--baseline-toolchain`, `--candidate-toolchain`; records each arm's `rustc -vV` from inside its worktree |
| `c11-pairs.sh` | ≈80–90 min | **T11-01, now** | refuses if 1.97.1 cannot be run |
| `c11-merge.sh` | ≈25–40 min | after `c11/03-merge-outside-v` syncs | two settings of one build |
| `c11-gates.sh` | ≈10 min | every landing | as cycle 10, shared probe |
| `c11-rwlock.sh` | ≈1 min | on request | shared probe |

Self-tested in the container before this order was committed: `c11-baselock.sh --self-test` —
every injected fault refused, every clean control accepted, 26 arms; the F-11-13 arm shown red
(exit 1) with the offending line put back, green with it fixed. `c11-pairs.sh` refuses in the
container (no 1.97.1), which is correct. A two-arm `--candidate-caps off` smoke of
`c11-baselock.sh` at `--measured 1 --seconds 3` is recorded in §0.1.

---
## 7. GBS: Loan IQ and Calypso, tiered by invariant, with the parity cost

Researched again 2026-09-09 (sources in §10). Finastra's own page: Loan IQ serves "syndicated
lending, bilateral loans, and complex loan servicing", "specialty loans like CRE, SBA, and export
finance", "PIK, club deal, non-pro rata and unitranche", a "Payment-in-Kind (PIK) module",
multi-currency, GL integration, "secondary trading", an "ESG Service", "Loan IQ Nexus" as the
integration layer, and claims "21 of the top 25 syndicated lenders", "70% of the world's
syndicated loans", "$3.8T in loans syndicated … in 2024". Nasdaq Calypso (Quinnox guide, Nasdaq
product pages): "Interest Rate Derivatives, Foreign Exchange, Credit Derivatives, Equity
Derivatives, Fixed Income, Repo & Securities Financing, Commodities, Treasury Instruments"; "a
single trade record"; "market risk, counterparty credit exposure, liquidity positions, and trading
limits"; "margin calculations, collateral allocation, optimization strategies, and regulatory
methodologies such as ISDA SIMM"; "confirmation, clearing, settlement, accounting, corporate
actions, and lifecycle event management"; reporting under "Dodd-Frank, EMIR, MiFID II, FRTB, and
SA-CCR"; and, in 2026, Canton-network collateral connectivity (Nasdaq/QCP press release) — the
"tokenized collateral" item from cycle 10's brief has become a shipped integration.

**The reconciliation the brief asked for (LC-46).** The author's goal is parity; the admissibility
rule is invariants. They are reconciled by *tiering*: a capability is built when the invariant it
needs exists, and the invariant is built when a capability needs it. Parity is the union of the
tiers; the thesis is tier 1 and the language decisions in tier 2. Nothing is silently dropped —
tier 3 is the honest name for "a view over the ledger, free once tier 2 exists", and tier 4 is
the honest name for "not a ledger invariant, out of the thesis".

| capability (source) | invariant it stresses / needs | can Niles state it today? | GBS row | tier | what makes the row `ProductSpecific` |
|---|---|---|---|---|---|
| pro-rata to "over a thousand lenders" (Loan IQ) | conservation with a rounding remainder in one epoch; the batch envelope | yes (`conserve per (txn,cur)`; `ParticipantSumMismatch` exists) | TG01 | **1** | one payment → 1,000 postings → one sealed epoch → durable restart → eviction → replay, one trace |
| lifecycle that survives restart (F-12) | durable-before-visible for product state, not only postings | partially — product state is in-memory (`BLOCKED-T-09-lifecycle`) | TG01 | **1** | the same trace replayed after a kill |
| adjustments, amendments, reversals (Loan IQ) | bitemporal audit: `valid` vs posted; a reversal is a new posting that references the old | **no spelling** for `reverse(txn)` / `amend(txn, valid:)` (cycles 9, 10) | new | **2** | a language task first; then a row |
| multi-branch / multi-business-line GL mapping (Loan IQ); ISA/OSA/NOSA/GOSA account structures and "exposure netting across business lines" (Calypso) | **partitioned conservation with a cross-partition prohibition** (LC-41) | **no** — `conserve per (txn,cur)` is the only enforced grouping; NL0225 refuses anything else honestly | new | **2** | the author chooses the partition policy; then a row |
| PIK capitalisation (Loan IQ) | the ledger writing to itself on a schedule | no `at epoch`/`every` form | blocked | **2** (behind F-12) | — |
| collateral: cross-collateralisation, covenants, "violation / warnings / control" (Loan IQ) | a hold whose amount is a constraint over other accounts' holds | no — holds are per account | new | **2** | after LC-41 |
| variation margin, IM, SIMM, AANA, thresholds (Calypso) | a REV over a **non-ledger base** (prices), with a threshold | the IR lowers a join; `install` refuses it (A10-06, correctly) | Risk row, `Blocked — posts nothing` | **2** | the first non-linear REV, which is Open case 4.1.α — a thesis question, not a product one |
| secondary trading (Loan IQ), repo / securities finance (Calypso) | a transfer of a share of a position: two-sided conservation across a position ledger | yes for cash legs | Trading row (`GenericPathOnly`) | **3** | after TG01 gives the position shape |
| letters of credit, trade loans, supply-chain finance | a hold with expiry (G3) | yes | existing rows | **3** | G3 evidence exists; `ProductSpecific` when the amendment spelling exists |
| sweeps, cash pooling, ZBAs | scheduled transfers between accounts | no schedule form (as PIK) | existing rows | **3** | after the schedule form |
| single trade record / unified record (Calypso) | the ledger *is* it | yes by construction | — | **3** | every downstream artefact provably a view — the thesis's own claim |
| regulatory reporting (EMIR, SFTR, SA-CCR, FRTB) | scheduled views | no schedule form | — | **3** | as above |
| pricing library, XVA, curves, corporate actions, CCP connectivity (60 venues), SPAN2/PRISMA/IRM2 methodologies, OCR, "Academy.AI", Nexus | **none** — not ledger invariants | — | — | **4** | out of the thesis; a product company's decade |

**Parity cost, honestly.** Counting the table: **two tier-1 items** (one task, TG01), **six
tier-2 items** of which **three are one language decision** (LC-41) and **two are one language
form** (a schedule), **five tier-3 items** that are rows once tier 2 exists, and a tier 4 that is
not reachable and should not be planned. With this pipeline: **a defensible syndicated-lending
subset** (TG01 + amendments/reversals + LC-41 + PIK-with-a-schedule + a covenant hold + FX) is
**four to six cycles after TG01**, i.e. cycles 12–17. **The whole of tier 3** is two to three
cycles more. **Calypso-class parity is tier 4 and is not on any horizon** — margin methodologies
certified by ISDA and connectivity to sixty CCPs are not built by a thesis pipeline, and the
honest sentence for chapter 11 is that GBS demonstrates the *ledger and view invariants* those
systems rest on, not the systems.

**No GBS task is above the cut this cycle** — F-11-05 means the served numbers GBS would be
shaped by are not yet the thesis's own workload, and TG01 should be measured on the corrected one.
TG01 is the first task of the next cycle that takes a product.

---

## 8. Open-questions ledger

Settled, not reopened: LC-01, 02, 04, 08–12, 14, 15, 17, 18, 21, 28; LC-16, LC-35 (cycle 9);
the manifest-derived diff and the baselock waiver (cycle 10).

| LC | position after this audit |
|---|---|
| 03, 05, 06, 07, 13, 19, 22, 25, 26, 27, 29 | carried; none moved |
| **20** | `BLOCKED-LC20-definition` — four cycles; **decide or retire this cycle** |
| **23 / 24** | the base lock; stands; T11-02 and T11-08 are its two halves |
| **30** | implemented and guarded since cycle 9; **the author confirms or rejects the protocol this cycle** |
| 31, 33 | T08, below the cut |
| **32** | T07, re-baselined against T11-02's pinned arm |
| 34 | TG01; §7 tier 1; first product task, cycle 12 |
| **36** | `BLOCKED-recovery-tip`; the author's contract; **decide or retire this cycle** |
| **37** | T11-08; **must not be below the cut a third time** unless T11-02 wins outright |
| **38** | measured: pin wins served, merge wins single-threaded; **the author's A/B/C is now: A = T11-02, with OFF as the default meanwhile** |
| 39 | `pinned_installs / reads` as a diagram input: yes, *with its cost* — after T11-03 puts the diagram and the served path on one skew |
| 40 | T11-06 (markers); T05's second half below the cut |
| **41** | honest refusal landed; **the partition policy is the author's and is now the gate on three tier-2 rows of §7** |
| 42 | T10, design only, below the cut |
| **43** (new) | the merge's cost model — T11-02 decides by measurement; the container inverts the sign (§2.0) and may not conclude |
| **44** (new) | the harness as a crate — **T11-05 takes the first step** (the scorer and the JSON contract in Rust); the shell stays the driver this cycle |
| **45** (new) | the branch plan as a constraint — adopted in §3/§4; a deviation is `BLOCKED-branch-<id>` |
| **46** (new) | parity vs invariants — reconciled by tiering (§7); the author confirms the tiers |
| **47** (new) | model assignment — §1.5's recommendation; the author decides |
| **48** (new) | reset or continue — §1.3: no reset of the runtime or compiler; **P4 (a second ledger backend) is the one reset-shaped decision and T11-10 is its note** |
| **49** (new) | the skew universe — what dataset the author will obtain (T11-04.2 names the shape) |

---

## 9. Reporting requirements for the executing agent

Deliver `docs/audit/cycle-11/execution-report.md` with:

1. **The checklist**: every target line of §3 **verbatim**, one row each, `done` or `not done:
   why`, with the evidence link, the exact command and exit code, and host / SHA / toolchain
   **as recorded from inside the worktree that ran it**.
2. **Guard transcripts**: for every guard, the disposable worktree path, the reversion applied,
   the assertion that fired and its exit code. A compile error is never the witness.
3. **Timing tables** with a host column; the container's rows labelled ratio-only.
4. **The MISMATCH/BLOCKED inventory** as T11-06 generates it, or — if T11-06 did not land — the
   table from this order's §9.6 with any change marked.
5. **Worktree status** separating the five untracked files (with sizes: 10,244 / 16,639 /
   6,148 / 8,196 / 816,110) from task changes.
6. **Every SHA and every bundle's sha256 and `list-heads`.**
7. **Exactly three material facts** found while executing that this order did not cover — and
   if the executor finds more, the three with the highest EV and a one-line list of the rest.
8. **A sync section** listing every bundle and the author's commands, **repeated at every
   landing until confirmed**, each block naming the ref the bundle carries and using `git branch
   <name> FETCH_HEAD` for a branch that does not exist locally.
9. **The MF-12 rule as practice**: every multi-edit Python script either writes each edit as it
   makes it or ends with a verification pass that greps for every edit, and the report says which.

### 9.6 The marker inventory at `653a369` / `01e1d85`

niles — 34 `MISMATCH` families, 18 `BLOCKED` families (occurrences in parentheses; **S** for a
read of each family's active/historical phrasing; two are stale):

`MISMATCH-`: pending-unreachable (3; 1 historical) · T-11-overdraw (6) · F-07 (6) · durability-
restart (1) · daemon-checkpoints (3) · T-22-currency-literal (8) · e16-header (1) · F-08 (5) ·
**A9-F05 (1, `SPEC-ENGINE.md` — repaired by C9-04.1, not struck, second cycle)** · hole-version-
unused (2) · **A9-F15 (1, `thesis/04:37` — repaired by `73fe782`, not struck)** · idem-fingerprint
(1) · contract-instance · T-13-fixpoint (4) · F-09 (3) · A-01 (4) · lattice-interval · lock-count ·
T-18-normalised-fields (3) · T-18-close (3) · A9-F12 (1) · placeholder-hasher · idem-window ·
currency-at-wire · T-18-position (1) · M-01 (2) · A9-F01 · two-cores · serving-impl · T-16 · F-0.

`BLOCKED-`: recovery-tip (3) · nilesc (5) · LC20-definition · fallback-rate (5) · T-09-lifecycle
(1) · adapter (2) · T-17-remainder (3) · T-01-toolchain (2) · sync-divergence · idem-clock ·
T-18-close (2) · T-06-txnerror-variants (2) · envelope-version · branch-collision · T-nn · T-18 ·
PG16 · LC-02.

gbs — `MISMATCH-`: T-18-close (4) · T-18-position (3) · T-18-encoder-divergence (2) ·
T-18-normalised-fields (1) · T-18 (1). `BLOCKED-`: nilesc (4) · T-09-lifecycle (4) · E-01 (3) ·
T-nn · T-18-close · T-18 · LC-02.

The full per-occurrence list with file and line is in
`docs/audit/cycle-11/markers-653a369.txt`, shipped with this order.

---

## 10. Sources read for §1.0 and §7 (2026-09-09)

Materialize, *Isolation levels* (`materialize.com/docs/get-started/isolation-level/`) — the four
levels, the linearizability caveat, Real-Time Recency's scope. Feldera, *What is Feldera?*
(`docs.feldera.com/`) — "strongly consistent", fault tolerance "a preview feature". TigerBeetle,
*Safety* (`docs.tigerbeetle.com/concepts/safety/`) — quorum WAL, VSR, "immutable, checksummed,
and hash-chained", at-most-once transfers; Jepsen, *TigerBeetle 0.16.11*
(`jepsen.io/analyses/tigerbeetle-0.16.11`) — strong serializability held, the bug table, "fixed
schema", "no general queries". Finastra, *Loan IQ* (`finastra.com/lending/solutions/loan-iq`).
Quinnox, *Nasdaq Calypso: a complete guide* (`quinnox.com/blogs/nasdaq-calypso-guide/`); Nasdaq,
*Calypso treasury*, *Calypso clearing* product pages; Nasdaq/QCP Canton press release
(`nasdaq.com/press-release/…`). Clauset, Shalizi, Newman, *Power-law distributions in empirical
data* (SIAM Review 2009) for the estimator. The project's own `docs/research/engine-landscape.md`
and `language-landscape.md` for the alternatives they cite.

---
