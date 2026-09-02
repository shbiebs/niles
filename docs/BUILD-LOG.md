# Build log — from specification to instrument

Running record of the work that turns the thesis's *specified* language and engine into a
*built* one, with the reasoning behind each decision. Written as it happens, so that a
session that ends mid-way leaves something a later one can pick up. Newest section last.

## Why this file exists

The Turn-5 audit found the thesis and the repository contradicting each other. Appendix
B.14 said the full grammar "is maintained in the artifact at
`crates/niles-lang/grammar/niles.ebnf`"; the file had five lines. Appendix B.19 said the
keyword reference "is generated from the compiler's keyword registry"; there was no
registry and `docs/keywords.md` was a placeholder. Appendix E described the bootstrap in
the present tense. Chapter 7 correctly said "stub crates only", so the document
contradicted itself rather than merely overclaiming.

Two ways to fix that: weaken the claims, or build the artifacts. This log records the
second.

---

## Stage 0 — research before design

`docs/research/compiler-lessons.md` (12,000 words, primary sources, IEEE references)
collects transferable lessons from rustc and from SQL compilers. Five findings changed the
design that follows; two corrected assumptions this project held.

**Changed the design.**

1. **Split parsing from analysis on an epoch boundary.** PostgreSQL separates raw parsing
   from semantic analysis because "system catalog lookups can only be done within a
   transaction, and we do not wish to start a transaction immediately upon receiving a
   query string." Nilestream has the strictly stronger version of that constraint: a name
   resolves only relative to a *visibility frontier*, so a Niles parse must be epoch-free
   and total, and analysis epoch-anchored. This is not an imported style — it falls out of
   the consistency model, and it is now an argument in the thesis rather than an
   implementation note.
2. **The keyword registry is a data table with four axes, and everything generates from
   it.** PostgreSQL's `kwlist.h` is one table with no logic, consumed by whoever defines
   the macro, and it generates the perfect-hash lookup, the parser's token declarations
   *and* the documentation. Niles copies the pattern and adds two axes the thesis needs:
   `origin` (SQL / Rust / novel), which generates Appendix B.3's three tables
   mechanically, and `since`, which makes the edition mechanism enforceable.
3. **Reserved-word count is a function of parser technology, not vocabulary size.** SQL
   reserves heavily partly because LALR(1) cannot resolve context-dependent
   identifier/keyword ambiguity. Hand-written recursive descent with unbounded lookahead
   keeps `epoch`, `ledger`, `serve` and `budget` usable as column names — which is what
   makes Niles adoptable against a bank's existing schema. Measured outcome: 0 of 66
   novel keywords are reserved.
4. **Four IR levels, split by what becomes checkable at each.** Following RFC 1211's
   account of why MIR exists: surface CST, desugared, fully typed (where the
   consistency-effect calculus is stated), and a flat epoch-explicit operator DAG (where
   the partial-state algebra lives and materialization is decided).
5. **GoogleSQL's accessed-field discipline for the IR.** A mechanism that turns "the
   back-end silently ignored a semantic field" into a hard error is worth a great deal in
   a system whose central claim is that money cannot be created or destroyed.

**Corrected assumptions.** Chalk did not ship — what shipped is `rustc_next_trait_solver`,
influenced by it. The bootstrap gate is stage2-vs-stage3, not stage2 against itself, and
modern stage0 is a released beta, not a snapshot. Noria has no documented IR, so
"Nilestream has a typed IR and Noria did not" is a positioning asset, not a comparison.
And no authoritative production count exists for the SQL-92 or SQL:2016 BNF — so this
thesis cites none, and counts its own grammar instead.

---

## Stage 1 — the keyword registry (`crates/niles-lang/src/keywords.rs`, 470 lines)

174 keywords in one table, in ASCII order within four blocks. Each row carries spelling,
token, category, label status, origin, edition, a one-line description and a minimal
example. Six tests, including two that enforce policy rather than mechanics: the registry
must stay sorted (so a diff and a perfect-hash generator agree), and **no novel keyword may
be reserved without a written justification in the file**.

Measured: 174 keywords — 70 SQL-derived, 31 Rust-derived, 66 novel, 7 reserved-for-future.
95 unreserved, 66 reserved, the rest in the two intermediate classes. Every one of the 66
novel words is unreserved.

## Stage 2 — the lexer (`src/lexer.rs`, 640 lines, 10 tests)

Two layers, following rustc's split of a dependency-free `rustc_lexer` from a cooking
`StringReader`. The raw layer knows characters, not keywords. **The stream is lossless** —
every byte of input belongs to exactly one token or one trivium, checked by a test — which
is what a formatter, an IDE and a pretty-print round-trip need.

Four literal forms exist in neither parent language, and each needed a disambiguation
decision:

* **Money** `10.00 usd`. The literal's *own* scale is counted from the digits and kept
  separate from the currency, so `10.001 usd` becomes a scale error naming both numbers
  rather than a silent rounding. A number followed by an ordinary word is not money: only
  a three-letter lowercase non-keyword closes the literal.
* **Epoch** `#4200`, disambiguated from the attribute prefix `#[` by one character.
* **Two time axes** `@2026-03-01` and `v@2026-03-01`. Two prefixes, because bitemporality
  has two axes and one spelling would make the axis invisible at the point of use. An
  ISO-8601 body always begins with a digit, which is how `@confidential` and
  `read@snapshot` share the sigil without ambiguity.
* **Duration** `7.days`, checked before the fraction rule.

## Stage 3 — AST and diagnostics (`src/ast.rs` 720 lines, `src/diagnostics.rs` 250 lines)

The AST is the *raw parse* level: a `Name` is a string and a span, never a resolved id.
`StageKind::Unknown` exists so an unrecognised pipeline stage parses and is diagnosed with
a suggestion, and `Expr::Error` / `Item::Error` are ordinary variants.

Diagnostics are structured values with a stable `NLnnnn` code, a primary span, labelled
secondary spans, notes and machine-applicable suggestions. The reason is specific to this
thesis: the interesting errors are conservation, currency, linearity, effect and contract
errors, and every one of them must point at two places at once — the money that was
created *and* the rule that forbids it; the hold *and* its second resolution; the view's
declared rung *and* the effect that exceeds it. A one-span error cannot say that.

## Stage 4 — the parser (`src/parser.rs`, 1,300 lines, 15 tests)

Hand-written recursive descent with a Pratt loop. Every `parse_*` returns a node, error
nodes included; every recovery point names a synchronising set and always consumes at
least one token. Two properties have tests of their own: **a bad declaration between two
good ones must not destroy either**, and **parsing is total** — twenty adversarial inputs
including empty, unbalanced and non-UTF-8-shaped text must all terminate with a tree.

One design point worth recording: after `.` or `|>`, and in struct-literal field position,
**every** keyword is legal, because those positions have exactly one reading. That is why
`p.where(..)`, `p.select`, `p.order` and `p.check` all work without reserving anything.

The worked program of thesis Appendix B.20 is now `examples/demo_bank.niles` and parses
clean, under test. Where the thesis's printed example used forms the language does not
have — an `idem"..."` adjacent-string literal, `acct!(1001)` macros — the example was
corrected to the real syntax rather than the syntax being bent to the prose.

## Stage 5 — the normative grammar (`grammar/niles.ebnf`, 600 lines)

205 named rules; 496 productions in BNF-normal form. **Both figures are computed by
`tests/grammar_drift.rs` and asserted against the header**, so neither can drift. The file
carries fifteen sections, ending with twenty numbered well-formedness rules (W1–W20) that
are deliberately *not* syntax — a grammar that tried to encode "a hold is consumed exactly
once" would be neither readable nor decidable — each of which is a claim that some later
phase enforces it.

Four drift tests run in both directions: every registry keyword must appear in some
production, every alphabetic grammar terminal must be a registry keyword or a known
library name, the stage vocabulary must match `StageKind::all_names()`, and the W-rules
must be contiguous.

## Stage 6 — the generated keyword reference (`docs/keywords.md`)

`cargo run -p niles-lang --bin gen-keyword-ref` renders the registry into the normative
per-keyword reference of Appendix B.19. `tests/keyword_ref.rs` is a blessing test in
`compiletest`'s style: it fails if the checked-in file differs from what the generator
would produce now. That is what turns "generated from the compiler's keyword registry"
from a claim about how the file was once produced into one that holds continuously.

**Status at this point: 45 tests passing in `niles-lang`. The three thesis/repo
contradictions identified in the Turn-5 audit are closed by construction rather than by
weakening the prose.**

---

## Stage 7 — the semantic phases (`resolve`, `typecheck`, `currency_rows`, `effects`)

Four modules, and the split between them is the parse/analyse boundary taken seriously.

`resolve.rs` builds the catalog **at an epoch**. That is not ceremony: in this system a
name resolves only relative to a visibility frontier, because a migration is itself a
ledger fact and `postings` at epoch 4,200 may not have the columns it has at 9,000. So
`Catalog::at(epoch)` carries the epoch, and the same code serves the offline compiler and
the server path. PostgreSQL splits parse from analyse because catalog lookups need a
transaction; this is the stronger version of the same constraint, and it is an argument
the thesis can make rather than a style it imported.

`currency_rows.rs` (11 tests) is the machinery behind the first half of Contribution 4. A
transaction's net effect is not a number, it is a *row*: a map from currency to signed
amount. `conserve per (txn, cur)` says every entry of that row is zero — not that the
total is zero, which would let 10 USD cancel 10 EUR. Amounts are linear combinations of
opaque symbols plus a constant, so `debit(a, x); credit(b, x)` is provably conserving for
an unknown `x`, and a currency variable with union-find makes `fn move<C>(..)` conserving
for every `C` without monomorphising by currency.

The design decision worth recording is the **third verdict**. A row can be *conserving*,
*violating*, or **undecided** — the checker cannot see through the amount. Undecided is
not an error; it is an obligation handed to the runtime and counted. A checker that
accused every program it could not follow would be unusable, and one that assumed the good
case would make the soundness theorem vacuous. The third verdict is what keeps the first
two honest.

`effects.rs` (12 tests) is the second half. The novel judgement is **rung monotonicity**: a
view served at rung ℓ may not read a view served below ℓ. Everything else in the calculus
is a currency-flavoured restatement of a known effect system; this one is not, and it is
the one that earned its keep (see Stage 9).

## Stage 8 — the IR (`niles-ir`, 39 tests)

Four levels, split by what becomes checkable at each — the reason rustc has MIR, applied
here. The circuit level is where the partial-state algebra lives, and it carries two things
a conventional relational IR does not.

**The accessed-field discipline**, borrowed from GoogleSQL's resolved AST. Every
semantically load-bearing field on a node sits behind an accessor that records the read,
and `assert_all_accessed` fails if any went unread. The failure this prevents is not a
crash. It is an engine that does not notice a `ledger_consistent` annotation, serves from
stale state, and returns a number that looks exactly right. No answer-level test catches
that; only a test of whether the annotation was *read* does.

**Conservation transparency**, composed along paths. A `filter` drops rows, so money can
leave a filtered view without leaving the ledger. That is a legitimate thing to want and
it is not a control total, and the IR is where the difference becomes checkable.

`upquery_path.rs` derives reconstruction routes and is where anchoring stops being a slogan.
A path is a pure function of `(circuit, node, key, epoch)` — no locks, no sequence numbers,
no protocol state — because the prefix it reads is not moving. The five partial-state
anomalies are not prevented here; they have no state in which they could occur.

`verify.rs` is in the trusted base and the compiler is not. That is deliberate: the
compiler is large and will be extended by people who did not write it; the verifier is
small and can be read in an afternoon. A compiler bug therefore produces a *rejected
circuit* rather than a wrong answer, and a wrong answer here means money.

## Stage 9 — closing the loop, and what it found

`crates/nilestream` compiles a `.niles` file, verifies the circuit, installs it on the REV
runtime, and runs a workload against a real ledger. Two Chapter 9 findings reproduce
through that path rather than around it: SC7 (flat at 8.4 base rows across a 16× history
increase, predicted 9) and the phase diagram (strictly interior optimum, boundary between
memory prices 0.0005 and 0.002).

**Then the compiler was run over the thesis's own worked program, and rejected it.** Four
errors, of which one matters most: `available_balance` promised `ledger_consistent` while
reading a `read_your_writes` view. That is the two-derived-views-disagreeing failure
Chapter 1 opens with, written into the example by the person who formulated the rule that
forbids it. Nothing else in this project argues as well for having a compiler.

The other three: `holds` declared as a `ledger` when it does not conserve; `settle_fx`
declaring an effect row describing a transaction that could not exist; and a view predicate
calling `month_start()`, which reads the wall clock — the IR verifier rejected it because a
reconstruction could then legitimately differ from the value it replaced, which would make
reconstruction-equivalence *false* rather than unproven.

## Stage 10 — durability and concurrency (`nilestream-ledger`, 21 tests)

Length-prefixed, CRC-checked records; the hash chain recomputed on read rather than taken
on trust; recovery that truncates at the first bad record and reports how much it dropped;
a single-sealer sequencer with group commit that publishes the frontier only after fsync.

The ordering rule is the entire guarantee: **durable, then visible**. Publishing first
would let a reader observe an epoch a crash then erases, and in a ledger that is not a
stale read — it is a transaction the customer watched succeed and that no longer exists.

Three defects found by the tests, each a money bug rather than a crash:

1. **Two copies of one idempotency key in the same batch both committed.** The dedup
   consulted committed history but not the batch being assembled. A client retrying fast —
   or a client and a proxy retrying together — lands both in one drain. The result is a
   duplicated payment, not an error.
2. **A duplicate was accounted for after its reply was sent**, so a caller could observe an
   outcome before the state that produced it. The same ordering rule as durable-before-
   visible, one level up. It failed about one run in three, which is exactly the frequency
   at which a race gets written off as flakiness.
3. **`Always` synced twice per epoch.** The benchmark surfaced it as 0.5 transactions per
   fsync — a figure with no sensible interpretation, which was the clue. Fixing it raised
   single-threaded throughput 3,003 → 4,450 tx/s.

E13 then measured what Chapter 9 had marked *to be measured*: durability costs 5.7× at one
thread and 4.8× at sixteen, because group commit amortises the fsync. Transactions per
fsync rise 1.0 → 8.8. **A single sealer is a batching opportunity, not the ceiling it looks
like** — which was the design's most likely objection, and is now answered with a number.

## Where this leaves the thesis

The three contradictions from the Turn-5 audit are closed by building the artifacts rather
than by weakening the claims. Chapter 7 now states what is built and what is not with equal
precision, §7.5 lists each gap with the claim it withholds, and Appendix E.0 labels its own
tense. §6.10.1 re-assesses the language-creation gate against the built compiler and
**narrows G1**, because a gate that always passes is not a gate.

Still unbuilt, and stated as such: distribution, consensus, wire protocols, a cost-based
planner, and the self-hosted bootstrap. The executable IR fragment is narrower than the IR
the compiler emits, and the runtime rejects what it cannot run rather than mis-executing it.

---

## Stage 11 — the planner (`nilestream-optimizer::offline`, 18 tests)

Contribution 5's calculus, as a decision the engine takes. It sits between the compiler
(which fixes *what* a view computes and *what it promises*) and the runtime (which keeps
some of it resident).

The design is one ordering: **filter by contract, then price.** Infeasible modes are
removed before any cost is computed, never after. Pricing first and filtering after would
be the same code with the same output in the common case and a contract violation in the
uncommon one — the shape of bug that survives testing. That ordering is what makes a bad
estimate cost compute rather than breach a rung.

Wired into `nilesc plan`, which prints the decision *and* what it ruled out and why.
§11.2 lists optimizer opacity as a named risk: an adaptive component that changes behaviour
under load and cannot be interrogated is one an operator will not trust at the moment it
matters most.

## Stage 12 — the wire protocol (`nilestream-server`, 21 tests)

PostgreSQL wire protocol v3, simple query path, over a real socket. `nilestreamd` answers
`psql`. The transcript is in `results/wire-protocol-session.md`.

The commitment: **a wire protocol is a surface, not a semantics.** A client's SQL is parsed
as Niles's SQL surface, lowered to the same IR, verified by the same verifier, served from
the same runtime. No compatibility layer with its own execution path — two ways to compute
an answer is two answers that can disagree, which is the seam this thesis argues against
everywhere else, and building one here would have been incoherent.

Three decisions the transcript shows:

* **The anchor is a column on every row.** Not a footnote. "The same question, asked twice,
  answered consistently" is checkable only if the client can see which moment each answer
  belongs to.
* **A missing key is NULL, not zero.** The absence lattice reaches the client intact. There
  are three absences here — SQL null, the empty string, and an evicted hole — and the codec
  keeps all three apart.
* **Money is `numeric`, never `float8`.** Exactness that survived the type system has to
  survive the last hop, or the whole apparatus of per-currency scales ends at the socket.

Everything unimplemented is refused by name with its reason. The extended query protocol's
refusal states the open design question: a prepared statement must be cached against the
epoch it was planned at, because a plan valid at one visibility frontier need not be valid
at another. Shipping a version that ignored that would be worse than not shipping one.

## Stage 13 — consensus (`nilestream-consensus`, 9 tests)

**The ledger is a log**, so the Raft mapping is an identity rather than an analogy: epoch =
entry, sealer = leader, visibility frontier = commit index. Distributing the single-node
write path changes who decides the order, and what "durable enough to publish" means — one
disk becomes a quorum — and nothing else. That is the retrospective argument for having
built §7.1's write path the way it is.

**What the hash chain adds over Raft.** Raft's log matching property is maintained by
protocol: a follower accepts an `AppendEntries` if the previous index and term match. It is
sound, and it rests entirely on nodes reporting their own state honestly — a node that lied
about its previous term, through a bug or a corrupted disk, would be believed. Here the
previous entry is identified by its **hash**, and the follower recomputes the link before
accepting. A forged entry is rejected even from a current leader.

This is not Byzantine tolerance: a lying leader can still refuse to make progress, and
§11.1 scopes Byzantine settings out. It converts a class of *silent divergence* into a
detected one, and it costs nothing, because the chain is computed anyway for audit.

**Every test is deterministic** — seeded PRNG, logical clock, no sleeps. A consensus test
that sleeps and hopes is worse than no consensus test: it trains its reader to re-run it.
25 seeds at 20% message loss and 30% reordering, with election safety, log matching, state
machine safety and chain integrity asserted after **every single message delivery**, not at
the end. An invariant checked only at the end tells you a system was broken without telling
you when.

Unbuilt and named as data in `NOT_BUILT`, so the list cannot quietly shrink in the prose
while the code stays the same: membership changes, log compaction, pre-vote, leadership
transfer, and the cross-shard commit protocol of §8.6.

## Final position

264 tests. The three Turn-5 contradictions are closed by building the artifacts. The loop
runs from Niles source text to a measured result, and two of Chapter 9's headline findings
reproduce through it rather than around it.

Still unbuilt, and stated in §7.5 with the claim each withholds: a distributed read path,
cross-shard commit, the extended query protocol, TLS, the MySQL wire protocol, cost-based
join ordering, and the self-hosted bootstrap of Appendix E stages 1–3. The executable IR
fragment remains narrower than the IR the compiler emits, and the runtime rejects what it
cannot run rather than mis-executing it.

## Session 8 — the distributed path, the wire surfaces, and the bootstrap

**Built.** `nilestream-core::distributed` (sharded read path, 10 tests);
`nilestream-consensus::cross_shard` (2PC with a ledger-group coordinator, 12);
`nilestream-server::extended` (epoch-keyed plan cache, 11); `::mysql_wire` (11);
`::tls` (negotiation and policy, cryptography delegated, 17);
`nilestream-optimizer::join_order` (`DPccp` + a three-term cost model, 16);
`niles-interp` (stage-0 execution, 28) and `bootstrap/lexer.niles` with 14 gates.
418 workspace tests, 0 failures, 2 benign warnings.

**Three design results worth keeping separate from their code.**

1. *2PC's blocking objection dissolves when the coordinator is a ledger group.* The
   decision is persisted through quorum before it is sent; a successor reads it rather
   than re-deciding. The implementation refuses to send a decision that is not durable.
2. *A cross-shard upquery needs no coordination at all*, because it reads a frozen prefix.
   The read cannot be made stale, and its result caches forever with no invalidation
   protocol. This is Thm 4.1's anchoring paying a distributed dividend.
3. *Join ordering in a partial-state engine is not the classical problem.* A join is a
   standing operator with two resident indexes, and reconstruction walks the tree, so the
   objective has three terms rather than one — and reconstructibility is a **legality**
   constraint pruned before costing, so a bad estimate can make a plan slow but not wrong.

**The bootstrap unblocked itself by re-reading its own requirement.** Appendix E.0 said
stage 1 had no input because stage 0 could not compile the whole language. But a bootstrap
needs stage 0 to *evaluate* Niles, not to emit machine code; "stage-0 compiler" had been
read as "native compiler". A tree-walking interpreter plus a Niles-written lexer produced
four working gates the same day.

**The gate found four defects on its first run**, which is the evidence that it has
discriminating power: two keyword-table drifts (`evict`/`conserves` do not exist;
`from`/`post` were missing), a case-sensitivity error (Niles inherits case-insensitive
keywords from SQL while Rust is case-sensitive — the two lineages disagree and only the
registry settles it), and a flaw in the gate itself, which excluded every input the
reference lexer errored on and thereby silently dropped the two hardest cases. The
keyword table is now **generated** from `keywords.rs`, with a test that fails on
divergence: the single source of truth crosses the bootstrap boundary.

**Two claims retracted, both consequential, neither fixable by a test.**

* *"Noria cannot be used in production for core banking."* The motivating attempt never
  built, started or connected to Noria; zero SQL ran against it. Every failure in that
  4,565-line transcript is a build or packaging failure. §1.1.1 and §11.5.5 now say so,
  and the Noria limits the thesis relies on are cited from its authors' own papers.
* *"REVs need a new engine."* Already refuted by E14 and now restated in §11.5.2 with the
  enabling/cumulative/editorial grading applied throughout: Niles's *analysis* is
  enabling, its *surface syntax* is editorial, and Nilestream is an instrument.

---

## Session 9 — the parser in Niles, subquery unnesting, and four defaults that were wrong answers

Two blueprint tasks closed (8 and 4, the latter in two rounds), and the thesis reconciled
to what the repositories now do. Word count 92,773 across 26 files; `Niles-Thesis.docx`
rebuilt.

**The bootstrap has a front end rather than a lexer.** `bootstrap/parser.niles` is a
recursive-descent parser written in Niles, ~1,050 lines, loaded with `lexer.niles` as one
program. It builds a tree — `enum Node { Atom(str), List([Node]) }` — and renders it
afterwards, rather than emitting text as it goes: the claim under test is that Niles can
*hold* a syntax tree, not that it can concatenate strings in the right order. Sixteen
gates, including the front end parsing both of its own source files, node for node
identically to the reference, across 127,165 bytes of tree.

Comparison is on an S-expression rendering produced independently by both sides, never on
structures: the two parsers share no types, and a structural comparison would need an
adapter, which is the one thing a gate must not be, since a defect in it cancels a defect
in either side.

**The parser round found a defect in the *reference*.** Assignment was left-associative:
`expr_bp` recursed for the right-hand side at binding power 1, which put assignment outside
its own `min_bp == 0` guard, so `a = b = c` parsed as `(a = b) = c` — the opposite of
Rust's rule and of the comment directly above the code. It had survived every test in the
workspace, because associativity is invisible in a token stream and the lexer round could
not have found it. It surfaced within minutes of a second implementation existing. That is
the argument for stage-1 equivalence stated as a result instead of a hope.

It also found that Appendix B had no operator precedence table, so §6.25's "the appendix
wins" had nothing to win with. B.10.1 now states precedence and associativity normatively,
and `crates/niles-lang/tests/precedence.rs` reads the table out of the markdown and checks
every level against `BinOp::precedence`.

And it found that the stage-0 interpreter spent **~95 KB of host stack per interpreted call
frame** in a debug build against 4.8 KB in release — a `match` over thirty expression
variants compiles, unoptimised, to a frame holding the union of every arm's locals. A
recursive-descent parser was unrunnable, and the failure mode was a process abort with no
diagnostic, because a stack overflow in Rust does not unwind. Cold arms moved behind
`#[inline(never)]` (debug 32 KB), and a call-depth counter turns the remaining limit into
an `Error::TooDeep` carrying a span. The default ceiling is the depth that fits a 1 MB
stack in the widest build; the first value tried assumed a 2 MB thread stack and aborted
the test process, which is how the number became measured rather than assumed.

**Subquery unnesting needed three things the IR did not have.** A *nested form*
(`Op::Apply`, a dependent join — deliberately not incremental, so the verifier refuses one
on a served path, which reframes unnesting as what makes a correlated query expressible as
a view at all rather than as an optimisation). A *null* (`niles-ir::value`, Kleene
three-valued logic; the IR's value model was `i128`, so `not in` was unstatable). And *one
semantics* — the reference evaluator moved out of a `#[cfg(test)]` block and became public,
shared by the schedule catalogue and the unnesting corpus, because two copies of a
semantics is two semantics.

The corpus is 24 cases checked **denotationally** rather than structurally, plus a
hand-written three-valued oracle for the eight `not in` cases so a shared misunderstanding
cannot cancel between the two circuits. Correlated-regime counted work: 1.44× at k=1 rising
to 61.39× at k=64, quadrupling as k quadruples — the signature of removing a quadratic.

**Two corrections the measurement forced.** The evaluator ran every equi-join as a nested
loop, so counted work could not distinguish the two plans and reported that unnesting saved
nothing; the *instrument* was wrong, not the rewrite. And at the original nine-by-ten
dataset the `not in` cases did more work after unnesting, because the unnested form is six
operators; loosening the assertion would have discarded the actual result, which is a
crossover. The corpus is now scale-parameterised and the crossover is measured per case.

**The surface round found the session's worst defect, and it had nothing to do with
subqueries.** `select k from t where t.z = 1` returned every row. Two faults compounded:
`=` in a SQL `where` clause parsed as an *assignment*, because Niles's two ancestries
disagree about that character and the parser took the Rust reading everywhere; and all
three predicate sites in `lower.rs` read `.unwrap_or(Scalar::LitBool(true))`, so a
predicate with no lowering became the constant `true`. The query looked correct, the plan
verified, and no answer-level test could catch it, because every row it returned was a real
row. A second defect in the same area: a correlation whose two columns shared a name —
`where u.k = t.k`, the commonest correlated predicate there is — was left behind as the
tautology `k = k`, making `exists` a no-op.

**The pattern, now named.** `Err(_) => 0` in the kernel, `sum` over an empty group, and
`unwrap_or(LitBool(true))` in a `where` clause are the same defect in three costumes: an
absence given a *reasonable default* that is a wrong answer wearing a plausible shape. The
failure mode is never a crash and never an obviously wrong number — it is a well-formed
answer no answer-level test can distinguish from the right one. §3.3's lattice of absence
exists because absences are not interchangeable and not substitutable by a value; these
three are what happens when that discipline is not carried into the implementation. The
rule the repositories now follow is that an absence gets a *named* representation or a
diagnostic, never a default, and each site is pinned by a test asserting the default is
gone. §11.5.7 records it.

**No measurement is typed into the thesis by hand any more.** `thesis/include-results.py`
copies generated tables into the chapters between markers, `build.sh` runs it before
pandoc, and `crates/bank-bench/tests/thesis_drift.rs` fails the build if a block is stale —
so a table that stopped describing the run it names is caught by the test suite rather than
by a reader.

---

## Session 10 — the review work order

Nineteen tasks (T-01 … T-19) closing findings F-01 … F-51. Entry format, one per event:

```
### [T-nn] <ISO-8601 UTC> <KIND> <one-line title>
KIND ∈ {START, DECISION, STALE-F-nn, BLOCKED-T-nn, MISMATCH-<id>, RESULT, TESTS, LC-n, DONE}
```

### [T-01] 2026-09-02T01:05Z START Convention baseline, and the branch stacks

Floors measured before any change, on `master` at `d96e6c1`:

| Workspace | passed | failed | ignored |
|---|---|---|---|
| `niles` (`cargo test --workspace`) | 600 | 0 | 3 |
| `gbs` (`cargo test --workspace`) | 415 | 0 | 1 |
| adapter (`cargo test --manifest-path crates/gbs-nilestream/Cargo.toml`) | 15 | 0 | 0 |

**Branch stacks and merge order.** `master` carries T-01 only (the pins and the two
mechanical commits); everything after it lands on a review branch rebased onto the
reformat, so that every later diff is semantic rather than whitespace.

```
master:                 T-01 (mechanical + pins), then nothing else
review/F-01-F-06:       T-02 -> T-03 -> T-04 -> T-05                (niles)
review/F-28-F-29:       T-06                                        (niles)
review/F-19-F-27:       T-07 -> T-08 -> T-09  stacked on review/F-28-F-29  (gbs; adapter after T-06)
review/F-23:            T-10                                        (gbs, independent)
review/F-11-F-13:       T-11 -> T-12 -> T-13 -> T-18                (niles; T-12 touches gbs)
review/F-14-F-17:       T-14 -> T-15 -> T-19  stacked on review/F-28-F-29 and review/F-11-F-13
review/F-18:            T-16  stacked on review/F-01-F-06, review/F-19-F-27, review/F-14-F-17
review/thesis:          T-17  stacked on everything
```

A task in one repository that needs a change in the other lands that change as its own
commit on the same-named branch there, and the entry names both hashes.

**Toolchain.** Pinned to **1.95.0** (`rustc 1.95.0 (59807616e 2026-04-14)`, the version
that builds both workspaces today) in both `rust-toolchain.toml` files, with `rustfmt` and
`clippy` components, and `rust-version = "1.95.0"` under `[workspace.package]` in both
workspace manifests. The channel was `stable`, unpinned; Appendix C.6 already claimed
"pinned toolchains", so this makes an existing claim true as well as making the lint gate
stable.

**Format baseline.** `cargo fmt --all -- --check` reported 1,274 diffs in `niles` and 612
in `gbs` before this session. The reformat is a separate, purely mechanical commit with
identical test counts on both sides of it, and both reformat commits are listed in
`.git-blame-ignore-revs`.

### [T-01] 2026-09-02T01:20Z BLOCKED-T-01-toolchain The version pin cannot be installed here

**Question for the author.** `rust-toolchain.toml` should read `channel = "1.95.0"`, and
does not. Which do you want: the version pin, which is correct for a stranger and makes
Appendix C.6's "pinned toolchains" true but leaves both repositories unbuildable in the
environment this increment was executed in; or `channel = "stable"` with the version
recorded, which builds here and does not satisfy the pin?

**Both sides, verbatim.** The work order's resolved value is `channel = "1.95.0"` in both
`rust-toolchain.toml` files. The environment has no egress to `static.rust-lang.org`
(`connect_rejected`, organization policy), so `rustup` cannot install a channel named by
version:

```
error: could not download file from
'https://static.rust-lang.org/dist/channel-rust-1.95.0.toml.sha256'
```

The obvious workaround — link the already-installed toolchain under the pinned name —
is refused by rustup itself, because a custom toolchain may not be named like a dist
version:

```
error: invalid value '1.95.0' for '<TOOLCHAIN>': invalid custom toolchain name '1.95.0'
```

The installed `stable` **is** the intended version: `rustc 1.95.0 (59807616e 2026-04-14)`,
`rustfmt 1.9.0-stable (59807616e1 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`.

**What was done, and why it is not a silent workaround.** `channel = "stable"` is kept so
that the remaining eighteen tasks can run at all — T-01 is the prerequisite for every one
of them, and blocking it blocks the increment. The intended value, the reason it is not
set, and a pointer to this entry are written into the file itself as a comment, so the
deviation is visible at the place where it matters rather than only here. Setting the pin
is a one-line change for anyone with network access.

**Consequence for T-17.** Appendix C.6's "pinned toolchains" claim is *not* made true by
this commit and must not be reported as such. T-17 states it as "the toolchain file
carries the intended pin as a comment; the channel is `stable` pending an environment
that can install a versioned channel", or the pin is set first and the claim then stands.

### [T-01] 2026-09-02T02:10Z DECISION The crate-level lint allow list (niles)

Every exception is at crate level with a justification, so the complete set is auditable
here rather than scattered over call sites. Four call-site `#[allow]`s that existed before
this session were removed or moved.

| Crate | Lint | Why |
|---|---|---|
| `niles-ir` | `should_implement_trait` | `Tri::not` is Kleene three-valued negation, named after the logic. Implementing `std::ops::Not` would give `!` a meaning on a value whose third answer is `Unknown` rather than a flipped bit. |
| `nilestream-consensus` | `should_implement_trait` | `Sim::next` steps the deterministic message pump one delivery; a step can inject a fault as well as deliver, so it is not an iterator and must not become one. |
| `nilestream-server` (lib and bin) | `dead_code`, `enum_variant_names` | The wire surface is incomplete by construction: no write path, extended protocol unwired. `ReadStats`, `MemoryEngine` and several accessors have no caller *yet*. Deleting them would hide the gap `results/E16-wallclock.md` reports; T-14 either wires or removes them. `BackendKeyData` is the PostgreSQL message name. |
| `experiments` | `dead_code` | `RunResult` carries every counter the shared driver collects, not only the ones a given experiment prints. Narrowing it to today's questions is how a harness stops being able to answer the next one. |
| `nilestream-optimizer` (test `unnest_corpus`) | `too_many_arguments` | The corpus builder takes one argument per dimension of a case; a struct moves the same nine values one level down. |

### [T-01] 2026-09-02T02:10Z RESULT Two defects the lint gate found, neither stylistic

**A logic bug clippy denies by default.** `crates/niles-lang/src/parser.rs` parsed the
`ORDER BY` direction as `self.eat_kw(Kw::Asc) || true`. The behaviour is correct — `asc`
is the default, so the direction is ascending whether or not the keyword is present, and
the call is there to *consume* the token — but written that way it reads as a bug and
`clippy::overly_complex_bool_expr` is deny-by-default, so it was an error rather than a
warning. Rewritten as a consume followed by the constant, with the reason in a comment.

**A ledger segment opened without an explicit truncation flag.**
`crates/nilestream-ledger/src/segment.rs` opened the segment with `.create(true)` and no
`.truncate(..)`. `false` is already the default so nothing was wrong, but on the one file
in this system whose accidental truncation would discard every committed epoch, the
intent should be stated rather than inherited from a default. Now explicit.

### [T-01] 2026-09-02T02:10Z TESTS Gate green in both workspaces

`niles` 600/0/3 → 600/0/3. Format: 1,274 diffs → 0. Clippy: ~55 warnings + 1 deny-level
error → 0.

### [T-11] 2026-09-02T09:05Z RESULT F-11 confirmed in every part, by running the mutants before writing the fixes

Each pointer reproduced. `nilesc check` on a file containing only the offending line:

| F-11 | mutant | before |
|---|---|---|
| (a) no general typing | `let x: Money<usd> = "hello" + true;` | `ok:` — 0 errors, 0 obligations |
| (b) `Auth` is forgeable | `let auth: Auth<authorize<usd>> = 42;` | `ok:` |
| (d) callee rows never reach the caller | a `ledger_consistent` view whose body calls a helper that reads a `bounded` view | `ok:` |
| (e) interprocedural money vanishes | `txn { leak_half(a, m) }`, callee posts one half | `ok:` — **0 proved and 0 discharged** |

(e) is the one worth stating twice. The obligation did not become a runtime check and did
not become a warning. It was not counted: `report.inferred_effects` was written and never
read, so a call contributed nothing to its caller in either dimension.

(g) is **STALE-F-11g**. `supports_must_violation` is indeed unconditionally `true`, but its
doc comment now argues for exactly that rather than "saying otherwise". The defect is a
different one: `check_conservation` branched on a predicate that is a constant, which is
`_ => true` in a soundness decision with a name on it. Both the predicate and the branch are
gone; the reasoning that makes the unconditional answer correct is kept as a comment on
`Provenance`.

### [T-11] 2026-09-02T09:10Z MISMATCH-T-11-overdraw Theorem 4.4 clause (3) as written is not implementable on this AST

**The thesis says** (§4.5, T-Overdraw; Theorem 4.4 clause 3), verbatim in substance: a
well-typed program *cannot overdraw without authorization* — read as a static guarantee that
no path reduces a balance below zero unless authorised.

**The code says** nothing of the kind, and cannot. There is no balance-bound analysis in
`niles-lang`, no abstract domain over account balances, and no representation of a balance
at a program point: `Amount` is a linear form over opaque symbols, so "is this balance
negative after this posting" is not a question the domain can express. Implementing it would
need an interval or affine-inequality domain over per-account state and a way to relate a
posting to the account it lands in — a different analysis, not a missing case in this one.

**What is implemented instead, and is enforced from this commit:** the capability discipline.
`Auth<E>` has exactly two introduction forms (a parameter, a `grant`) and no expression
produces one; a `let` annotated `Auth<E>` over any other expression is NL0330; an `Auth<E>`
argument position accepts only an `Auth` of the same effect (NL0331); and the `authorize<c>`
effect now propagates through calls, so a caller two hops from the `authorize` still needs the
capability (NL0312). The mutant `overdraw_without_authorize.niles` has *two* functions and
both are refused — before, a one-line wrapper laundered an unauthorised overdraft, because
the wrapper's row did not mention the effect.

**Proposed replacement wording for T-17**, §4.5 clause (3) and Theorem 4.4 clause (3):

> no overdraw redex is typed without a capability introduced by `authorize`

and a sentence after it stating what that does and does not buy: it guarantees that a path
which *can* reduce a balance below zero holds authority; it does not bound the balance, and
Proposition 3.2 already says the floor is not coordination-free. The stronger reading should
not appear anywhere in the thesis.

### [T-11] 2026-09-02T09:20Z DECISION Read effects match the declaration exactly, in both directions

`effects.rs::declared_permits` permitted an inferred `Read(r)` under any declared `Read(r')`
with `r <= r'`, and a unit test asserted it: *"declaring `read@snapshot` and doing
`read@bounded` is safe, because the declaration is a promise about the strongest guarantee a
caller may rely on."*

The direction is backwards, and the consequence is the exact failure rung monotonicity
exists to prevent. A `read@ℓ` states the **freshness of what the function returns** — a
computation is no fresher than its stalest input — not a permission the function asked for.
A function declaring `read@ledger_consistent` while reading a `bounded` view returns a
bounded-stale answer and tells every caller it is fresh; a `ledger_consistent` view calling
it passed rung monotonicity, because the check consulted the declaration.

Reads now match exactly. The unit test is inverted, and the inversion is the finding. Two
sources had to be corrected, both genuinely wrong:

* `examples/demo_bank.niles` — `main` declared `read@ledger_consistent` and also calls
  `explain`, which reads at `snapshot`.
* `gbs/niles/gbs.niles` — the same `main`, plus the two money effects it inherits from
  `transfer` now that rows are transitive. Committed on `review/F-11-F-13` in gbs.

### [T-11] 2026-09-02T09:30Z RESULT The lowering defaults, and a defect the first honest refusal found

Nine `unwrap_or` sites in `lower.rs` became diagnostics (F-37, F-47). The first run of the
new `where` rule failed `the_worked_example_compiles_clean_and_verifies`, and the reason is a
defect rather than a strictness problem: `where(|p| p.value_date >= v@2026-08-01)` had no
`Scalar` form, so on the pipeline surface it became `Filter{LitBool(true)}`.
`examples/available_balance.niles`'s `balance_as_of_2026_q1` returned **every posting in the
ledger** and presented it as a balance as of a date. `niles_ir::value::days_since_epoch` now
gives a date literal one canonical reading, used by both the scalar form and `valid_at`, so
the two surfaces cannot disagree about what a date is.

`StageKind::ValidAt` was `Op::ValidAt { instant: None }` unconditionally — the valid-time
axis, half of the bitemporality claim, discarded at the door whatever date was written.

### [T-11] 2026-09-02T09:40Z LC-6 Is there any expression the checker accepts whose static currency row is unknown yet counted as proved?

**No; every unknown movement is `Undecided` and counted as discharged to the runtime.**

The three places an unknown could enter, and what each does:

1. An opaque call returning money — a fresh symbol, so the entry is undecided unless the
   same symbol cancels. `Amount::is_decided` is false whenever any coefficient survives.
2. A call to a function in the same program — the callee's net row, instantiated with the
   caller's amounts. Symbols the caller cannot supply become *fresh caller* symbols, so two
   unrelated calls never look like the same amount.
3. A callee whose summary could not be computed (recursion, or the fixpoint's round budget)
   — marked `havoc`, and a havoc summary adds a fresh symbol to every entry it contributes,
   so it cannot produce a decided one however the arithmetic comes out.

`Report::conservation_proved` counts only `Verdict::Conserves`, which requires every entry
`is_zero`. `runtime_obligations` counts `Undecided` and `MayViolate`. The E18 interprocedural
group measures both cases 2 and 3 explicitly: `recursive_amortisation` is `Undecided`.

### [T-11] 2026-09-02T09:45Z TESTS niles 600/0/3 -> 610/0/3

New: `tests/calculus_mutants.rs` (5 tests over **17 mutants**, each asserting its own
diagnostic code), `the_interprocedural_group_reaches_the_caller`, and three tests on the
now-exhaustive IR predicates.

Acceptance checks from the work order:

```
cargo test -p niles-lang --test calculus_mutants   -> 5 passed, 17 mutants refused
cargo test -p niles-lang -p niles-ir               -> 0 failed
grep -n "unwrap_or(Scalar::" crates/niles-lang/src/lower.rs   -> prose only
grep -n "_ => true" crates/niles-ir/src/operator.rs           -> prose only
nilesc check .../mutants/auth_forged_from_a_literal.niles; echo $?   -> 1
cd gbs && make schema                              -> ok (after the gbs.niles fix above)
```

`results/E18-solver-verdicts.md` regenerated: the single-function distribution is unchanged
(0% of correct functions undecided, all five defects caught), and the new interprocedural
group reports **6 of 8** conserving cases proved across the call boundary, 2 undecided, and
both deliberate defects refuted. GC-01: the numbers are whatever the run produced.

### [T-12] 2026-09-02T10:20Z RESULT F-24 confirmed, and it was worse than one view

`gbs/niles/gbs.niles` had **three** views computing the same thing under different names:
`ledger_balance`, `available_balance` and `party_position` were all
`postings.group_by(|p| (p.acct, p.cur)).sum(|p| p.amt)`, differing only in their contracts.
`trial_balance` is a fourth, and is legitimately that.

The consequences, in order of how much they mattered:

* `available_balance` read `encumbrances` nowhere, while the comment above it and the
  comment above `encumbrances` both said it did. An availability decision that does not
  net out live holds is a ledger balance under another name.
* **No view in the schema read another view**, so NL0311 was vacuous in the one file it most
  needed to hold in. The check could have been deleted and every test in
  `tests/niles_schema.rs` would still have passed — the counterfactual there runs on a
  nine-line hand-written mini-schema.
* `party_position` grouped by `(acct, cur)`, so it was not a position per party at all.

### [T-12] 2026-09-02T10:25Z DECISION `party_position` is `full` and `pinned`, and the reason is a result

Keyed by `owner`, the view must join `accounts` — `owner` is not a column of `postings` —
and `accounts` is a `table`. A table's history is not retained, so state evicted from a view
rooted in one cannot be reconstructed: there is nothing left to fold. The IR verifier says
so (IR013 on three nodes) and it is right.

So the choice is real: either the chart becomes a `base`, and every reclassification becomes
an append somebody has to interpret, or a party-keyed view is fully materialised. The schema
takes the second, and says so in a comment. The Pareto-skew argument the old comment made
for `demand` is a good argument that applies to views keyed by a column the *ledger* has.

Two supporting changes in `niles`, both defects of the same family as F-37:

* `Op::derive_key` ended in `_ => None`, so `Negate`, `OrderBy` and `Limit` lost the key.
  None of the three touches a column — negation flips Z-set weights, ordering permutes rows,
  a limit drops whole rows — so all three preserve it. The loss was not cosmetic: a node
  with no derivable key is IR013, so `except` between two keyed aggregates could not be
  written, which is exactly the shape `balance minus encumbrances` takes. Now exhaustive:
  `Map` and `Fixpoint` are the two honest `None`s and are enumerated as such.
* NL0223 ("no anchor index covers this key") fired on views that are never evicted. The
  suggested remedy for `party_position` is an anchor index on a column the ledger does not
  have, so it was a permanent, unfixable line in the gate's output — and a warning nobody
  can act on is one everybody learns to scroll past. Now scoped to evictable views.

### [T-12] 2026-09-02T10:30Z RESULT `nilesc effects` names a view's sources

The acceptance check is "shows a read of `encumbrances`", and the old output could not:
`view available_balance reads no stricter than ledger_consistent` is equally true of a view
reading the ledger directly and of one reading another view at the same rung. The rung alone
could never have caught F-24. `Report::view_sources` is collected from the syntax — including
through the SQL surface's `from`, which is a `TableRef` and not an expression, so
`sql_positions` had been reporting "reads nothing" while its effect row said
`ledger_consistent`.

```
view available_balance      reads encumbrances, postings — no stricter than ledger_consistent
view encumbrances           reads holds — no stricter than ledger_consistent
view ledger_balance         reads postings — no stricter than ledger_consistent
view party_position         reads accounts, postings — no stricter than ledger_consistent
view sql_positions          reads postings — no stricter than ledger_consistent
view statement_mtd          reads postings — no stricter than ledger_consistent
view trial_balance          reads postings — no stricter than ledger_consistent
```

### [T-12] 2026-09-02T10:35Z RESULT `make schema`, recorded

```
nilesc check   ../gbs/niles/gbs.niles -> ok: 6 relation(s), 7 view(s), 12 function(s);
                                         11 conservation obligation(s) proved statically,
                                         0 discharged to the runtime
nilesc verify  ../gbs/niles/gbs.niles -> verified: 22 nodes, no violations
```

22 nodes, up from 16: the `except` adds a negate and a union, and the party join adds a join
and a second source. No warnings.

### [T-12] 2026-09-02T10:38Z TESTS gbs 415/0/0 -> 417/0/0 with NILES_ROOT set; niles 610/0/3 unchanged

`cd gbs && NILES_ROOT=../niles cargo test -p gbs-products --test niles_schema` → **15 tests**,
0 failed. `NILES_ROOT=/nonexistent` → fails, naming the path and both places it looked.
A missing checkout with no `GBS_SKIP_NILES=1` is now a failure rather than a printed skip:
these checks used to report `ok` for work they had not done.

### [T-13] 2026-09-02T11:30Z RESULT F-12 confirmed in every part, and the corpus found four more

`grep -ri golden crates` → nothing, so thesis §4.7(c) and §9.7's "golden-file α-equivalence
tests" described a file that did not exist. Every pointer in F-12 reproduced, and writing the
corpus turned up four the review had not named:

| form | what it did |
|---|---|
| `select acct from postings` | emitted no `Map`; the view returned every column |
| `select distinct k from t` | `distinct` parsed and never read; duplicates returned |
| `a UNION b` | `set_op` parsed and never read; the second query vanished |
| `from t, u` | `from.first()` was the whole list; answered from `t` alone |
| `join u on t.k = u.k` | `ON` resolved against the *left* schema only, so `u.k` was not found, `and_then` turned that into "no residual", and the join ran unconstrained — a cross product, silently |
| `where n is null` | **did not parse**: `null` is a reserved keyword with `is null` as its registry example and had no expression form at all |
| `x is not null` | the `not` was consumed and dropped, so it produced the same tree, and the same answer, as `is null` |
| `select sum(v) from t` | a *global* aggregate took the projection path, where `sum` lowered to an uncertified UDF: the query answered `0` once per row |
| `t EXCEPT u` | Z-set subtraction, so rows present only on the *right* came back with weight −1 — which SQL never produces |
| `t INTERSECT u` | a semi-join, which preserves the left's multiplicities: that is `INTERSECT ALL` |

### [T-13] 2026-09-02T11:35Z DECISION the corpus states denotations, not circuit shapes

The thesis says α-equivalence of circuits. The corpus is stronger and the reason is not
pedantry: two circuits can be structurally different and denote the same Z-set, and
structurally identical while both being wrong — which is exactly the state the SQL surface
was in, since the pipeline surface it was compared against had the same defects. So each of
the **41 cases** carries a `.expected` file holding the Z-set the query denotes on a fixed
dataset (`tests/golden/DATA.md`), and both spellings are evaluated by `niles_ir::eval` and
compared against it. 18 cases are written in both surfaces and must agree.

### [T-13] 2026-09-02T11:40Z RESULT the reference evaluator gained three operators

`Fixpoint` panicked ("the reference evaluator does not cover fixpoint"), so C6(b)'s
completeness claim had no runnable witness at all; `OrderBy` and `Limit` panicked too.

* **`Fixpoint`** now runs to a least fixpoint and `Op::Fixpoint` takes two inputs — the seed
  and the step's output — with the step reading the accumulator through the `Delay` that
  closes the cycle. The step used to be *discarded at lowering*: the node held a termination
  guard and no body, which is verifiable and not evaluable. Non-convergence inside the round
  budget is `EvalError::NonTerminating { rounds, tail }`, reported rather than answered.
* **`OrderBy`** is the identity, because a Z-set has no order to change. Stated rather than
  omitted.
* **`Limit`** forces a choice: "the first n rows" of an unordered collection is not a
  denotation, and SQL's `LIMIT` without `ORDER BY` genuinely has no defined answer. The
  reference semantics picks — nearest upstream `order by` keys, then lexicographic — and
  writes the choice down, so an engine that answers differently is disagreeing with
  something a test can check.

### [T-13] 2026-09-02T11:45Z MISMATCH-T-13-fixpoint C6(b) "fixpoint completeness": the machinery converges, the syntax cannot express a closure

**The thesis says** (§4.7, C6(b)) that the fragment is complete for fixpoint queries, with
`WITH RECURSIVE` mapped to `.fixpoint(step) guard measure(m)`.

**The code says** the fixpoint *runs*: case `31_fixpoint_identity_converges` reaches closure,
and `a_non_terminating_fixpoint_is_refused_rather_than_answered` shows a growing step being
refused with the round count and the accumulator's last sizes. What cannot be written is a
transitive closure, and case `36_fixpoint_transitive_closure` records the refusal (NL0514).

Two pieces of surface syntax are missing, and both are needed:

1. **A join cannot state its key.** `.join(u)` takes the two sides' anchor keys, so the step
   can only join the accumulator's `src` to `edges`' `src`; a closure needs `acc.dst` to
   `edges.src`.
2. **A projection cannot name a duplicated column.** After a join the schema is
   `[src, dst, src, dst]` and `col_index` returns the first match, so the outer `src` with
   the inner `dst` — the pair a closure step must produce — has no spelling.

So the step cannot produce the shape it consumes and NL0514 refuses it, which is the right
refusal for the wrong reason: the arity check is doing the work that a missing feature should
be reported by.

**Proposed for T-17:** `WITH RECURSIVE` stays `Status::Specified` (it already is), and
Appendix H's completeness statement for C6(b) is narrowed to *"the fixpoint operator
evaluates to a least fixpoint and refuses non-convergence; the surface syntax needed to
express a transitive closure — an explicit join key and qualified column references — is not
in stage 0"*, with `36_fixpoint_transitive_closure.expected` cited.

### [T-13] 2026-09-02T11:50Z RESULT the status table is generated, and every row is backed

`Status` gained two variants that did not exist because nothing could have filled them:
`Refused(code)` — a form the compiler refuses, named with the code that refuses it, which is
what narrowing the fragment looks like from inside the code — and `Untested(why)` for the DML
and TCL rows, which lower and have nothing checking what they compute. Calling those
`Lowered` beside forms with a corpus case behind them made the word mean two things.

`the_status_of_every_form_is_backed_by_a_corpus_case` reads `sql_surface.rs` and fails if a
non-excluded row claims a status with no case named on its line, or names a case that does
not exist. `cargo run -q -p niles-lang --bin gen-sql-surface -- --check` regenerates the
SPEC-LANGUAGE L-5/L-23 status sentence from `MAPPING` and fails if the committed copy
differs; it is in `make gate` as the new `generated` target.

**The fragment, as generated:** 34 forms — 8 equivalent, 13 lowered, 4 refused, 2 untested,
2 specified, 5 excluded. The previous sentence read "**Status: Built** for the declared
fragment" and had said so since before the fragment was checkable.

### [T-13] 2026-09-02T11:55Z TESTS niles 610/0/3 -> 615/0/3; golden corpus 41 cases

```
cargo test -p niles-lang --test sql_golden              -> 5 passed, 41 cases
nilesc explain <select acct from postings>              -> shows a `map` node
cargo run -q -p niles-lang --bin gen-sql-surface -- --check -> exit 0
cd gbs && NILES_ROOT=../niles make schema               -> ok, 25 nodes verified
```

`make gate` green in both repositories.

### [T-18] 2026-09-02T14:30Z DECISION `txn` is evaluated; `hold`, `resolve` and `fx` are not

F-25's agreement half is that conformance compared *renderings*. Fixing it needs a Niles
function to be executable, and executing one needs a dynamic semantics for `txn`, `debit`,
`credit` and `post`.

They are **builtins**, not language features: entries in the interpreter's free-function table
alongside `print` and `len`, with no change to the grammar and none to `niles-lang`. The forms
were already Niles forms, checked statically by the effect calculus; what was missing was an
evaluator, and an evaluator is what an interpreter is. Adding syntax here would have meant the
language the interpreter runs is not the language the compiler checks.

`hold`, `resolve`, `fx` and `fixpoint` stay refused by name, and a test asserts it, so that
opening `txn` did not quietly open the others.

### [T-18] 2026-09-02T14:35Z DECISION two tests inverted, and both inversions are the finding

`the_relational_tier_is_refused_by_name_and_never_approximated` asserted that `txn` was
refused, for the stated reason that *"a program could appear to conserve money while nothing
checked it"*. That was the right worry and refusing the form was the wrong answer to it: a form
nobody can execute is a form nobody can compare against a second implementation, which is how a
conformance suite ends up comparing renderings. The worry is now answered by the seal — an
unbalanced set is refused by currency with its residual named — and the test asserts that
property instead of the refusal of the syntax.

`bootstrap_stages.rs`'s version now checks the second half by **running** the bootstrap lexer
and asserting it sealed nothing. The grep it replaced would have called the sample program
inside a string literal in `bootstrap/lexer.niles` a breach.

### [T-18] 2026-09-02T14:40Z MISMATCH-T-18-normalised-fields the contract said two fields, the languages need three

*Work order §5, T-18, interface contract, verbatim:*

> `// conformance.rs: NORMALISED_FIELDS: &[&str] = &["stamp.system_epoch", "entry_id"]  — the
> only fields removed before comparison; any other difference is a failure.`

*What the code needs:*

> `pub const NORMALISED_FIELDS: &[&str] = &["entry.id", "entry.stamp.system_epoch",
> "entry.narrative"];`

*Proposed replacement, and why.* `entry.narrative` is a per-leg free-text annotation.
`gbs-products` sets one on most legs — `"drawdown under fac-1"`, `"novation of t1 to the
clearing house"` — and **the schema has no syntax for one at all.** With two normalised fields
the comparison fails on every narrated product for a reason that is not about money.

The alternative was to invent a narrative form in Niles so that a test would pass, which is a
language change made to satisfy an assertion and is the class of move this review exists to
catch. It is normalised instead, listed in the constant, written into the fixture files so a
fixture generated under a different normalisation is refused, and stated in `ARCHITECTURE.md`
§7. It remains a gap: two implementations that agree on every posting and disagree about what
the posting *says* are not the same document, and a hash chain covering the narrative would
diverge.

### [T-18] 2026-09-02T15:10Z RESULT Every transaction in `gbs.niles` had one identity

The finding, and the reason for the whole task. Seven of the nine functions hard-coded a
constant idempotency key:

```
txn idem("draw-gbs-1", window: 30.days)      // syndicated_drawdown
txn idem("close-gbs-1", window: 30.days)     // close_offering
txn idem("novate-gbs-1", window: 30.days)    // novate
```

The Rust products derive theirs from the business event: `draw-fac-1-req-1` names the facility
and the request, `close-ipo-1` names the offering, `novate-t1` names the trade.

**Idempotency is the ledger's, and a constant key destroys it.** A second drawdown under any
facility carries the identity of the first and is refused as a duplicate — the transaction is
correct, conserves, passes every static obligation, and cannot be committed. The failure mode
is worse than losing money: it is silently refusing a legitimate one and telling the caller it
has already happened.

Nothing had noticed because nothing could look. The shape comparison this replaces reported
legs — direction, account, amount — and the transaction identity is not a leg. The legs matched
**byte for byte in all seven**; the whole divergence was in the field the old comparison had no
way to read.

The seven now take the identity as a parameter (`request: str`), which `nilesc check` accepts
and proves conservation over unchanged: 12 functions, 11 obligations proved statically, 0
discharged to the runtime.

### [T-18] 2026-09-02T15:20Z RESULT 7 of 9 conform by execution; the two that do not

`the_two_implementations_seal_the_same_document` runs each transaction on both sides and
compares the canonical encoding. Seven agree byte for byte. Two diverge, each on one named
field, and both are gaps in what the schema can say rather than defects in the products:

* **`MISMATCH-T-18-close_offering` — `entry 0.consumes`.** Rust:
  `Some("hold:sub-ipo-1-anchor")`. Niles: `None`. `Offering::close` consumes three subscription
  holds, which is what makes a closing atomic against the encumbrances it releases; the
  schema's `close_offering` moves the money and resolves nothing, so it declares a closing
  after which three subscribers are still encumbered. **Proposed replacement:** the schema's
  `close_offering` resolves each subscription hold inside the `txn`. That needs `resolve`,
  which the interpreter refuses by name, so applying the fix converts this entry into
  `BLOCKED-T-18-close_offering` naming `resolve` rather than into agreement. Not applied here:
  it is a change to the schema's semantics, and T-18's scope is the comparison.
* **`MISMATCH-T-18-position_account` — `entry 0.value_date`.** Rust: `35`. Niles: `0`. The
  cash-management run stamps a value date; `liquidity.rs`'s own comment says *when the balance
  was read* is the entire difference between cash management and a sweep. **Niles has no form
  for a per-leg value date inside a `txn`**, so the one thing distinguishing this product line
  is the one thing the schema cannot declare. **Proposed replacement:** a value-date form on a
  leg — `credit(funding, amount) valid @day` or similar — is a language addition and belongs in
  Appendix B, not in a conformance fix.

Both are pinned in `KNOWN_DIVERGENCES` with their exact field. That list is not a suppression
list and a test enforces it: an unexpected divergence fails, a recorded one on a different
field fails, and a recorded one that *disappears* fails — so the good news cannot land
silently either.

### [T-18] 2026-09-02T15:25Z TESTS niles 615/0/4 -> 634/0/4; `-p nilesc --test run` 8 new

`cargo test -p nilesc --test run` → 8 passed, one per exit path.
`cargo test -p niles-interp` → 71 passed (41 lib + 16 + 14).
`cd gbs && NILES_ROOT=../niles cargo test -p gbs-products --test conformance` → 7 passed,
printing `7 of 9 conform by execution; 2 diverge as recorded`.

### [T-18] 2026-09-02T15:26Z DONE F-25 (agreement half) closed — the commit below, and the GBS side on `review/F-11-F-13` there
