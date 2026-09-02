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

### [T-02] 2026-09-02T02:25Z LC-1 The effective anchor of a cold-but-resident key

*Question.* Where does a cold-but-resident key's effective anchor come from at a stride
boundary, and what is rewritten per epoch?

*Answer.* **From the view's `applied`, inherited on read; nothing per entry is rewritten
in an epoch that carried no delta for it.**

The view holds one `applied: Epoch`. A resident entry holds its own `stamp`, written only
when a delta actually touched it. A read computes the effective anchor as
`max(stamp, applied)` and the certification invariant is that this value is honest: every
delta in `(stamp, applied]` has either been applied to the entry or did not exist for that
key. Maintenance therefore stays O(deltas) rather than O(resident) — the alternative,
stamping every resident entry on every epoch, would measure the harness instead of the
design, which Appendix K.6 already records as an instrumentation decision.

The reason this has to be written down before the code is that the bug it prevents is
invisible until a cold key is read at a lax rung. A per-entry frontier updated only on
touch would leave a key that received no delta for a thousand epochs still claiming its
thousand-epoch-old anchor, and `BS(K,T)` would fail for exactly the keys that are cheapest
to serve. Inheritance is what makes "no delta means the value is unchanged" a property of
the view rather than a hope about each entry.

The converse obligation is the one T-02 is closing: inheritance is only sound if `applied`
never runs ahead of the deltas actually folded in. F-03 is precisely that failure — a
stride boundary that advanced `applied` past epochs whose deltas were never applied to
anyone — so `advance` must fold every epoch in `(applied, e]`, not only `e`.

### [T-02] 2026-09-02T03:05Z RESULT The three defects reproduced, and what fixing them cost the thesis

All three pointers reproduced exactly as described. Each is now a named test that fails
without its fix — verified by reverting the fix and watching `f01_`, `f02_` and the CERT
property test go red, which is the only thing that makes them regression tests rather
than assertions about current behaviour.

**F-01.** `install` wrote `Present(v, anchor)` for any anchor and `read` served
`max(stamp, applied)`, so an entry installed by a *historical* read was promoted to the
view's applied frontier and served, as a hit, for a later anchor. The fix is a `pinned`
set: an entry installed below the applied frontier keeps its own stamp, is never
promoted, and receives no delta (it is missing earlier ones, so folding a later one
compounds the gap rather than closing it).

**F-02.** `apply_epoch` folded a delta into any resident entry regardless of its stamp,
so an entry reconstructed *ahead* of the frontier received epochs it already carried.
A delta at or below an entry's own stamp is now skipped.

**F-03.** `advance` applied only the boundary epoch at a stride and then set
`applied = e`, so the `stride - 1` epochs in between were never folded into anyone while
the view was nevertheless certified through the boundary. `advance` and the E8 loop now
fold every epoch in `(applied, e]`.

**An epoch-zero hazard the fix exposed.** `applied: Epoch` starting at 0 cannot
distinguish "nothing folded yet" from "epoch 0 folded", and `proto-engine` numbers epochs
from zero. Folding "everything after `applied`" therefore skipped the ledger's first
epoch — the one that funds every account. `PartialView::apply_through` carries an
explicit `applied_any` flag, and `the_first_maintenance_pass_folds_epoch_zero` pins it.

### [T-02] 2026-09-02T03:05Z RESULT E8 re-run: the rung tax is not what was reported

`cargo run --release -p experiments -- e8`, commit on `review/F-01-F-06`, workload
parameters unchanged (10,000 accounts, 40,000 ops, budget 5%, skew 0.9, five seeds).
**Divergences: 0** across every row — the oracle column this experiment did not have.

| Rung | deltas applied | apply calls | misses | base rows read | hit rate |
|---|---|---|---|---|---|
| bounded(k=64) | 2,171 | 61 | 26,644 | 102,624 | 0.259 |
| bounded(k=8) | 2,514 | 446 | 24,700 | 73,811 | 0.315 |
| strict(k=0) | 3,621 | 4,017 | 19,658 | 40,869 | 0.456 |

Thesis Table 9.9 reports, for the same workload: deltas applied **55 / 408 / 3,621**,
misses 19,714 / 19,670 / 19,658, hit rate 0.455 / 0.456 / 0.456 — and concludes that the
rung's price is a **66x** difference in maintenance that is **invisible on the read path**.

Three claims in that sentence do not survive the correction.

1. **The 66x was the count of deltas thrown away.** Corrected, the ratio in deltas
   applied is **1.67x**, not 66x. What is still ~66x is *apply calls* — maintenance
   passes — which is a batching saving and a materially weaker claim: the same deltas,
   in fewer, larger passes.
2. **The read path is not indistinguishable across rungs; it is where the cost moved.**
   A lax rung now reads **2.5x more base rows** (102,624 vs 40,869) and misses far more
   often (26,644 vs 19,658). This is not a new cost: it is the cost that was previously
   hidden, because entries were being *falsely certified* through a frontier whose
   deltas had never been applied, so they registered as hits.
3. **The high hit rate of a lax rung was an artefact of the same defect.** 0.259 rather
   than 0.455.

The honest summary the thesis will have to carry: a bounded rung buys fewer maintenance
passes and pays for them in reconstruction, and the trade is visible on both sides of the
ledger rather than free on one. §9.4.3's "the tax for demanding freshness is not paid on
the read path at all" is refuted. Carried to T-05, which rewrites §9.4.3, §9.5.2, §9.12
and Appendix K.6 from these figures and adds Appendix J.16.

### [T-02] 2026-09-02T03:05Z TESTS niles 600/0/3 -> 613/0/3
`proto-engine` had no tests at all before this commit and now has 7; `nilestream-core`
goes 24 -> 37. Gate green.

### [T-03] 2026-09-02T03:35Z RESULT Checkpoints were not what Definition 3.9 defines

Reproduced. A checkpoint was recorded mid-epoch, at the running value after a particular
posting, while reconstruction resumes at the first row with `epoch > cp_epoch`. Every
later posting on that key *within the same epoch* was therefore skipped. Three postings of
+5 on one account in one epoch, at C = 2: indexed 10, full fold 15.

Definition 3.9 says a checkpoint is `(e, V*(e)[k])` — the value at the **end** of epoch e.
Recording now happens once per key per epoch, after every row of the epoch has been
folded, and only when the posting count crossed a multiple of the interval during it.

**Why no experiment could see it.** E10 and E11 both run one posting per key per epoch,
where mid-epoch and end-of-epoch coincide. E1, the only experiment that checks *values*,
runs with `Ledger::new()` — no checkpoints at all. So the mechanism SC7 rests on had its
cost measured and its correctness never checked. Theorem 3.7 clause (ii) was tested;
clause (i), that checkpointing changes cost and not value, was not.

Re-running E10 after the fix gives the same figures as before — 6.8 / 8.3 / 8.0 / 8.5 at
C = 16 across a 64x history increase — which is the expected result and worth stating:
**SC7's cost claim is unaffected**, because the defect was invisible to that workload. The
correction is to the mechanism's correctness, not to the measurement.

**One test of mine was wrong before the code was.** The first version of the bound test
read always at `head` and reported C = 64 missing its bound at 41 rows against 33.
Theorem 3.7(ii) is an expectation over an anchor falling *uniformly* between checkpoints;
reading at the head samples one fixed offset, which at 1,000 postings and C = 64 is 40
rather than the mean 32. The test now samples anchors across the history, which is what
the theorem says. Recorded because the failure looked exactly like a bound violation.

### [T-03] 2026-09-02T03:35Z TESTS niles 613/0/3 -> 616/0/3

### [T-04] 2026-09-02T04:15Z RESULT The oracle is now the oracle

`conservation-suite` was a 543-line fold with seventeen unit tests, declared as a
dependency of `bank-bench` and referenced by nothing; `faults.rs` and `properties.rs` were
one-line stubs. Thesis §3.11 ("all testing in Chapter 9 is differential testing against
𝒪"), §5.2 and Appendix F ("the one component of this project that already runs") were
therefore describing a plan.

`properties.rs` is now the harness — a `Schedule` of the transitions §3.11 names, an
`Observable` trait one anchored read wide, and a `differential` runner — and `faults.rs`
is the campaign builder: crash, eviction storm, duplicate delivery, read reordering. Both
`proto-engine` and `nilestream-core` have differential suites over them, each with its own
negative control that answers zero for a miss and must be caught.

**A defect in the harness, worth recording because it looked exactly like an engine bug.**
The first `Observable` returned a bare value, and the runner compared it against the
oracle at the *requested* anchor. That reported 2,009 units of divergence on the first
run. The engine was right: `read(key, anchor)` means "at least as fresh as `anchor`", so a
fresher entry may legitimately answer and says so in its returned anchor. The trait now
returns `Answer { value, anchor }` and the runner checks two separate obligations — the
answer is not older than the anchor asked for, and its value is exact at the anchor it
carries. Conflating them tests something no engine promises.

**The oracle could not answer its own headline question.** `rows_upto` did
`anchor as usize + 1`, so `u64::MAX` — "everything retained" — panicked. Now saturating:
the definition of correctness does not get to abort.

### [T-04] 2026-09-02T04:15Z RESULT E1 re-run against a real oracle

`cargo run --release -p experiments -- e1`. Two changes to what it measures:

* the expected value comes from `conservation-suite`, not from `Ledger::reconstruct_balance`
  compared with itself;
* anchors are drawn from the whole retained history rather than fixed at the head.

| seed | upqueries | evictions | divergences | rebuild mismatches | hit-path | miss-path | historical-anchor reads |
|---|---|---|---|---|---|---|---|
| 1 | 3,018 | 2,712 | 0 | 0 | 357 | 2,977 | 3,332 |
| 7 | 3,023 | 2,698 | 0 | 0 | 352 | 2,982 | 3,333 |
| 42 | 3,007 | 2,686 | 0 | 0 | 368 | 2,966 | 3,328 |
| 100 | 2,990 | 2,662 | 0 | 0 | 385 | 2,949 | 3,331 |
| 2024 | 3,017 | 2,703 | 0 | 0 | 358 | 2,976 | 3,332 |

Still zero divergences, and now the claim is worth more: Table 9.1 previously reported
~13,600 reconstructions agreeing with an oracle that was the same function, at a single
anchor. The corrected run compares against an independent fold at ~3,330 *historical*
anchors per seed. The rebuild-from-base check likewise now compares against the oracle
rather than against the engine's own reconstruction.

The hit-path column is small (~360 of ~3,330) and honestly so: with eight slots for forty
accounts and anchors spread over the history, a read at a historical anchor pins its entry
and the next read at a different anchor misses. Reported rather than tuned away.

### [T-04] 2026-09-02T04:15Z TESTS niles 616/0/3 -> 638/0/3

### [T-05] 2026-09-02T04:45Z RESULT Chapter 9 rewritten from the corrected instrument

Tables 9.1 and 9.9 are now generated blocks, filled by `thesis/include-results.py` from
`results/E1-correctness.md` and `results/E8-rungs.md`, both written by the harness itself
rather than by a script run beside it. `thesis_drift.rs` gains two tests: the blocks must
stay generated, and Appendix J must keep the refuted figures.

**§9.4.3 is rewritten and its headline claim withdrawn.** "The tax for demanding freshness
is not paid on the read path at all" is refuted. The corrected measurement:

| Rung | deltas applied | maintenance passes | misses | base rows read | divergences |
|---|---|---|---|---|---|
| bounded(k=64) | 2,171 | 61 | 26,644 | 102,624 | 0 |
| bounded(k=8) | 2,514 | 446 | 24,700 | 73,811 | 0 |
| strict(k=0) | 3,621 | 4,017 | 19,658 | 40,869 | 0 |

Replacement claim, written into §9.4.3, §9.5.2 and §9.12: a bounded rung buys fewer
maintenance *passes* — the same deltas, folded less often — and pays for them in
reconstruction, at 2.5x the base rows read.

**Appendix J.16** retains the refuted table (55 / 408 / 3,621 deltas; misses varying by
0.3%) with the mechanism that produced it, and J.15's count of "both wrong" rows goes from
four to five. J.16 is the first row in that appendix refuted by *reading* an experiment
rather than running one, and the appendix now says so: an experiment that reports counted
work and never checks a value can be precise, reproducible across five seeds, and
measuring its own defect.

**Appendix K.6's "E8 null" narrative is corrected.** It presented the sequence
null → diagnosis → re-instrumentation as evidence of care. The diagnosis was wrong: the
null was real, and the re-instrumentation measured the defect more sharply. What closed it
was an oracle column, not another counter.

**§9.2.1 and K.3** now state what the E1 oracle is. The prose no longer claims an
independent fold where there was a self-comparison, and the anchor discipline is named:
about 3,330 of roughly 3,340 reads per seed are at a historical anchor.

**Old names.** `results/e4.log`, `e56.log`, `e78.log`, `e9.log`, `e10.log` were stale
captured transcripts carrying a "Kaskata research prototype" banner; the program has said
"Niles" for some time. Regenerated from real runs. `grep -rli "kaskata\|upbasin"` over
`results/`, `crates/` and `docs/` is now empty.

### [T-05] 2026-09-02T04:45Z TESTS niles 638/0/3 -> 640/0/3

### [T-06] 2026-09-02T05:20Z RESULT Six durability holes, and the one that erased history

All of F-28 and F-29 reproduced. The worst is not the one that looks worst.

**Mid-file damage erased the ledger.** `open` truncated to the last valid record whatever
the damage was. One flipped payload byte in the *first* record of a three-record segment
therefore emptied the file — 266 bytes to 0 — and restarted the chain from genesis, after
which the file validated cleanly and nothing could tell that two committed, fsynced,
acknowledged epochs had ever existed. For a base whose defining property is that it is
never partial and never forgets, self-repair by forgetting is the wrong default. `open`
now refuses, names the offset and the cause, and truncates nothing.

The discriminator took two attempts and the first was wrong in an instructive way. "A
torn tail is smaller than the smallest complete record" fails on a large record: chopping
9 bytes off a 120-byte record leaves 111 bytes, which is bigger than a minimal record and
is still plainly a torn tail. The right question is not how many bytes remain but whether
a *further* record follows, so the damaged record's own length prefix is read and the tail
is the tail iff nothing lies beyond what it claims for itself.

**A torn tail of one to three bytes was reported as a clean end**, because every
`read_exact` error on the length prefix mapped to `CleanEnd` — which also made a real I/O
error indistinguishable from a tidy shutdown. Now: `UnexpectedEof` exactly at the end of
file is clean, `UnexpectedEof` anywhere else is a torn tail, and any other error is
returned.

**A partial write stayed on disk.** `write_all` can fail after writing some bytes;
`append` returned the error and the sealer carried on, so every later epoch sat behind a
torn record and was discarded at the next recovery. The write is now rolled back to the
last complete record.

**`Never` published before any fsync** while `submit`'s contract promises durability;
**`Every(n)` synced twice** per n-th epoch. The sequencer now refuses `Never` outright —
the policy stays on `Segment` for the durability benchmark, which exists to price the
guarantee by removing it — and the double sync is gone.

**No directory fsync.** A segment created and fully `fdatasync`ed can still be absent
after a crash, because the *name* was never made durable. Now synced, best-effort.

**The idempotency window did not survive a restart** (F-29): `seen` was an in-memory map,
so the retry a restart provokes — the retry a client is most likely to send — committed
twice. The batch framing now carries each transaction's key, `Sequencer::recover_seen`
rebuilds the window from the segment, and `Sequencer::open` uses it. The framing change is
inside the record payload, which is hashed; no committed fixture depended on it.

**One doc comment was false**: the checksum "covers everything" — it covers the body, not
the length prefix. On an audit artefact that distinction is worth stating correctly, and
the comment now explains what protects the prefix instead.

### [T-06] 2026-09-02T05:20Z TESTS nilestream-ledger 33 -> 40 tests; niles workspace 600/0/3 -> 607/0/3 on this branch
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

### [T-14] 2026-09-02T13:10Z RESULT F-16 confirmed — the compiler was decoration on a hard-coded answer

`session.rs` parsed the client's SQL, resolved it, typechecked it, lowered it, **verified the
circuit**, and then discarded it. `pick_view` returned the constant `"__wire_result"`;
`rev_engine`'s `read` ignored the view name, took `key[0]`, and folded `sum(amt)` for
currency 0; `extract_keys` collected digit runs out of the query *text* after the first
`" where "`.

So two different questions about one account returned the same number. The shape of the
`Serving` trait is what made this invisible — `read(&mut self, view: &str, key: &[i64],
anchor: u64) -> Option<i128>` cannot accept a circuit, so nothing in the type said the answer
was unrelated to the query. `Serving::query` now takes the circuit, and
`two_queries_over_one_key_return_different_answers` is the test that would have said so.

The engine materialises the base as a Z-set at the anchor and evaluates the circuit with the
same `niles_ir::eval` the golden corpus uses. That is a full fold and it is the honest cost of
an arbitrary query against a partial-state engine; the partial view remains as `read_point`,
which is the mechanism the phase diagram measures.

### [T-14] 2026-09-02T13:15Z RESULT F-17 — the write path, the extended protocol, and a real client

**The wire surface was read-only.** An `INSERT` was wrapped in a view and rejected, and
`begin`/`commit` flipped a flag and emitted a notice apologising that a sealed epoch cannot be
rolled back. Writes are now buffered inside a transaction and sealed as **one epoch** at
`COMMIT`, so there is no moment in which half of it has happened, and `ROLLBACK` discards the
buffer — there is nothing to compensate because nothing was sealed. `UPDATE`/`DELETE` against
a ledger are refused with `0A000` and the compensating-entry note.

**A simple query string may hold several statements.** It was treated as one, so
`psql -c "begin; insert …; commit"` — the ordinary way anyone scripts a transaction — came
back as a syntax error on the word `begin`. The transaction machinery existed and no real
client could reach it.

**`extended.rs` is wired.** It is a complete plan cache with eleven tests and a written
answer to the epoch-invalidation question (a plan is valid exactly while no schema epoch has
occurred after the one it was compiled at — a comparison of two integers), and it was
reachable from nothing: the decoder discarded the extended messages' *bodies*, so the
statement text never arrived. `Frontend::Extended` now carries the body, `Backend` gained the
four acknowledgements it had no way to send (`ParseComplete`, `BindComplete`,
`CloseComplete`, `NoData`), and the session serves `P`/`B`/`D`/`E`/`S`/`C`. Two tests that
asserted the *refusal* are inverted.

**Every diagnostic reached the client as `42P01` — undefined_table.** A driver's retry logic
reads the SQLSTATE, so a syntax error, a currency mismatch, a rung violation and a missing
capability were all reported as "no such table". `pg_wire::sqlstate_for` now maps the Niles
code's own family to a standard class, and a missing capability (`42501`) is distinguished
from a conservation failure (`23000`) and a repeated identity (`23505`) — a driver retries
those differently.

### [T-14] 2026-09-02T13:20Z RESULT the conformance suite uses `psql`, and three tests were inverted

Every wire test until now used the in-repo client, which is a client written against the same
understanding of the protocol as the server: two halves of one misunderstanding agree
perfectly. `results/wire-protocol-session.md` was a transcript of that conversation, cited as
evidence that a PostgreSQL client can connect.

`tests/psql_conformance.rs` drives `psql (PostgreSQL) 16.13` — real libpq — through the SSL
negotiation, the startup exchange, a point lookup, a scan, an insert, a transaction, an
unsupported construct, and a catalog query. **9 tests, all passing.** The transcript is
regenerated from that run.

Three assertions were inverted, and each inversion is the finding:

* `an_unkeyed_query_is_refused_with_an_explanation` → `…_is_answered_by_the_scan_surface`.
  The server refused `group by` over the base because "scanning defeats the mechanism being
  measured". That is a benchmark's reason, not a database's, and it is why the analytical row
  of Part 0 had nothing to run against.
* `a_missing_key_is_null_and_not_zero` asserted that a query for an account with no postings
  returns **one row whose value is NULL**, and `e16_nilestream.rs` argued this is "stronger
  than what PostgreSQL's `group by` does". It is not stronger — it was a *fabricated row*,
  manufactured from the key the digit scanner found in the query text. A grouped aggregate
  over an empty group produces no group. The absence distinction is real and is enforced by
  `read_point`'s `Option`, which is where it belongs.
* `the_extended_protocol_is_refused_with_the_open_design_question_named` → the protocol is
  served.

### [T-14] 2026-09-02T13:25Z RESULT durability, and a budget that binds

`DurableSink` wraps the sequencer at `SyncPolicy::Always`, which is the only policy it
accepts. `a_durable_append_returns_only_after_the_sync` is a **counter-test**, because the
property is an ordering and an ordering cannot be seen in a successful run: it reads the file
from a separate handle after each acknowledged epoch, and asserts the bytes are there *now*.

`bench.rs` set the Nilestream budget to 100,000 against 10,000 accounts, so after warm-up
nothing was ever evicted: the point workload measured the hit path exclusively while being
presented as a measurement of partial materialisation. The default is now 2,500 — a quarter
of the key space — and a budget that does not bind prints a warning saying so.

Every CSV row now carries `protocol_path` and `miss_rate`. The protocol is one constant so
both targets cannot differ: libpq's simple and extended paths differ by a round trip and by
whether the server re-plans, so a comparison in which each side used one would be measuring
the protocol.

### [T-14] 2026-09-02T13:28Z TESTS niles 615/0/3 -> 658/0/4

```
which psql                                              -> /usr/bin/psql (16.13)
cargo test -p nilestream-server --test psql_conformance  -> 9 passed
cargo test -p nilestream-server                          -> 152 passed
grep -rn "fn extract_keys\|fn pick_view" src              -> no match
grep -rn "extended::" src/session.rs                      -> 3 matches (the module is wired)
```

### [T-15] 2026-09-02T14:10Z RESULT F-14, F-15 — the calibration probed the wrong device and three rows were refused

**The `fsync` probe measured `temp_dir()`.** On this container that is a different filesystem
from `$PGDATA`: a tmpfs reports about a microsecond per call and a ceiling near a million
commits per second, against which every real durable rate looks implausibly low and the
plausibility gate is measuring the wrong device. The server is now asked
`show data_directory` and the harness refuses to guess when it will not say.

Measured on `/var/lib/postgresql/16/main` (`/dev/vda`, ext4): **77.9 µs per fsync**, ceiling
**12 845** durable commits/s per connection, PostgreSQL at **5 152 txn/s** — 40% of it, which
is the gate's basis for passing.

**Three of the four Part 0 rows were refused for reasons that had stopped being true.**
`nilestream_gap` returned "nilestreamd exposes no write surface over the wire" for `oltp` and
`durable`, and "a scan-and-group-by surface is not exposed" for `analytical`. T-14 made all
three false, so three quarters of the table reported a gap in the engine that was a gap in
the harness's beliefs about it.

**The `durable` row was measured against a non-durable append.** `NilestreamTarget::is_durable`
returned a hard-coded `false` with a comment about the read side being an in-memory demo, so
when the server acquired a durable path nothing would have noticed. The daemon is hosted with
a `DurableSink` at `SyncPolicy::Always` and refuses to start without one; the target asks the
server (`select nilestream_durability`) instead of asserting.

### [T-15] 2026-09-02T14:15Z RESULT the measured Part 0 table, and two NOT METs

| row | contract | PostgreSQL 16.13 | Nilestream | ratio | verdict |
|---|---|---|---|---|---|
| oltp | 5–10× | 4 518 ops/s | 4 189 ops/s | 0.93× | **NOT MET** |
| analytical | 10–12× | 330.0 ops/s | 44.4 ops/s | 0.13× | **NOT MET** |
| point | parity | 122.1 µs p99 | 131.1 µs p99 | 0.93× | PARITY |
| durable | parity | 4 654 ops/s | 3 820 ops/s | 0.82× | PARITY |

Five runs, 500 operations each, 10 000 accounts, both targets over the **simple** protocol,
both durable rows at `synchronous_commit = on` / `SyncPolicy::Always`, on two cores.

Both NOT METs are attributed in `BENCHMARK.md`'s new **Known limitations of the Nilestream
path** list — item 3 (per-query compilation: the simple path parses, resolves, typechecks,
lowers and verifies every statement) and item 6 (two cores against a contract written for a
48-core baseline) for `oltp`; item 4 (a full fold per unkeyed query) for `analytical`.
Neither is attributed to the engine's correctness. What the table supports is **a measured
baseline and a characterized gap**, which is what §9.14.1 and §11.3 must say (T-17).

### [T-15] 2026-09-02T14:20Z RESULT the `point` row's parity is a stronger claim than it was

`miss_rate` is now a column and it reads **1.0000**. The wire path evaluates the compiled
circuit over a source scan through the anchor index and does **not** consult the partially
materialised view, so every point read is an anchored reconstruction. Parity with an indexed
aggregate while reconstructing every read is a different claim from parity with a warm cache,
and the old harness could not have told them apart: the budget was 100 000 against 10 000
accounts, so nothing was ever evicted, and the column read `n/a` because nothing could ask.

That the view is unread by the wire path is itself a gap, and it is item 5 on the limitations
list rather than a silence.

### [T-15] 2026-09-02T14:25Z RESULT Z is recorded at every phase-diagram point

`results/e4_phase.csv` and `results/e12_phase_compiled.csv` gained a `z` column, defined
identically in both writers as base rows per reconstruction divided by deltas per read — the
counted-work analogue of Theorem 4.2(ii)'s delayed-hit factor, stated rather than borrowed
from a clock, because the whole point of counted work is that it transports.

The column's absence is why `results/E16-band.md` can pre-register only one side of the
crossover: every other constant in clause (ii) was measurable and this one was recorded
nowhere. E4 now reports Z from 5.3 to 307 across the grid.

### [T-15] 2026-09-02T14:28Z TESTS niles 658/0/4 -> 662/0/4

```
bench -- --calibrate --run --render --pg-port 5432 --host-nls  -> exit 0
grep -c "NOT RUN" results/E16-wallclock.md                     -> 0
grep -c "8-14" crates/bank-bench/src/bin/bench.rs              -> 0
python3 thesis/include-results.py --check                      -> exit 0
awk -F, 'NR>1 && $11==""' results/e4_phase.csv | wc -l         -> 0   (z is column 11)
awk -F, 'NR>1 && $8==""'  results/e12_phase_compiled.csv | wc -l -> 0 (z is column 8)
git log --format=%H -1 -- results/E16-band.md                  -> 504c131, an ancestor of the CSV commit
```

### [T-19] 2026-09-02T14:40Z RESULT F-48 — `make reproduce` regenerates and diffs

Three results files were produced by `#[ignore]`d tests with no guard at all — E15's bootstrap
gates, E17's unnesting corpus, E18's solver verdicts — so each could drift from the code that
made it and no build would notice. A results file nobody can regenerate is a results file
nobody can check.

The target now runs every generator and diffs the tree:

```
cargo run -q -p niles-lang --bin gen-sql-surface
cargo test -p niles-lang --test solver_verdicts -- --ignored
cargo test -p nilestream-optimizer --test unnest_corpus -- --ignored
cargo test -p nilestream-server --test psql_conformance -- --ignored transcript
cargo run --release -p bank-bench --bin bench -- --render
cargo run --release -p experiments -- e1 e4 e8
./target/release/nilestream sweep examples/demo_bank.niles ledger_balance > results/e12_phase_compiled.csv
python3 thesis/include-results.py
git diff --exit-code -- results/ thesis/ docs/SPEC-LANGUAGE.md
```

The diff covers `thesis/` and `docs/SPEC-LANGUAGE.md` as well as `results/`, because the
generated blocks in the thesis and the SQL-surface status sentence are derived from the same
sources and can drift the same way.

**It does not re-run `bench --run`.** That measurement is wall-clock, needs a live PostgreSQL,
and is machine-dependent: a diff against a committed CSV would fail on any machine but the one
that produced it. `--render` re-derives the table *from* the committed CSVs, which is the part
that must not drift, and the durable run's own reproduction recipe is in `BENCHMARK.md`.

`cd niles && make reproduce` → **exit 0** on a clean tree.

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

### [T-17] 2026-09-02T15:10Z MISMATCH-F-07 Theorem 4.3′ rests on a premise its own §4.8 refutes

**The thesis says** (§4.4 proof, verbatim): *"inspection shows no dependence on n, since
history enters only through per-key update counts, a workload property."* §3.14 says the same.

**The thesis also says** (H-S3 status, §4.8, §3.20): without checkpointing, cost grew **64×**
as history grew 64×, because under skewed access a hot key retains a roughly constant share of
a growing traffic total — so its update count is *not* a workload property bounded independent
of n. The mechanism that rescues the claim is per-key checkpointing at interval C, promoted to
SC7, and it appears nowhere in Theorem 4.3′'s parameter list.

The full proposal — revised statement with C as a **hypothesis of the theorem** rather than an
ambient fact, where C enters the upper bound, the revised lower bound
Ω(min(C, deltas-since-anchor)), and the matching edits to §3.14's Φ, §3.15's table, §1.6's H-S3
and §4.8 — is written verbatim as a fenced `PROPOSED` block in
`thesis/04-novel-contributions.md`, immediately after the theorem, and is **not applied** to
the running text.

### [T-17] 2026-09-02T15:15Z MISMATCH-F-08 C2 is an existence claim, not a predictive one

**The thesis says** (§1.6 H-S2): the crossover falls *"within the band predicted by the
Eviction–Consistency Frontier Theorem"*, and (§9.12 Finding 1) *"that is the frontier the
thesis predicts, located empirically for the first time."*

**Theorem 4.2(ii) predicts no band.** Its cost expression carries `(1 + Θ(Z))` and states no
constant for Θ, so the theorem asserts that a threshold *exists* for every (c_u, w, λ, Z).
An existence claim cannot be agreed or disagreed with by a located crossover.

C2 drops to an existence claim, refuted by finding no crossover anywhere in a swept price
range and not by finding one at an unexpected price. `results/E16-band.md` — committed at
`504c131`, before the durable run — registers the one side that *is* derivable, with Θ(Z)
set to zero, and names the missing constant.

**§5.5's rule is now satisfied.** It says a crossover without Z is not interpretable, and
neither phase CSV had a Z column. Both do (T-15), defined identically in the two writers;
E4 reports Z from 5.3 to 307 across the grid.

Written as a fenced `PROPOSED` block in `thesis/09-evaluation.md` at §9.12 Finding 1.

### [T-17] 2026-09-02T15:20Z MISMATCH-F-09 the restated H-S1 excludes its own counterexample

**The thesis says** (§9.13.2): *"the optimum is strictly interior wherever it is not at the
swept boundary."* An optimum at an extreme **is** at the swept boundary by definition, so no
measurement can refute the sentence, and the E12 table shows extremal optima at both price
ends.

The proposal gives H-S1 an explicit memory-price interval read from E12 —
**[0.0005, 0.002]** in units of one base-row read per resident entry per epoch — and the
falsifier that can fire: *a monotone cost-versus-budget curve at a price inside that
interval*. An extremal optimum outside it refutes nothing, and at a memory price of zero
full materialization is optimal by construction.

Written as a fenced `PROPOSED` block in `thesis/09-evaluation.md` at §9.13.2.

### [T-17] 2026-09-02T15:25Z RESULT §9.14.1 and §11.3 rewritten from the measured verdicts

§9.14.1's claim that three rows are `NOT RUN` because "nilestreamd exposes no write surface
over the wire" is gone: all four rows are measured on both targets, and **two say NOT MET**
(oltp 0.93× against 5–10×, analytical 0.13× against 10–12×). Each is attributed to a named
item on `BENCHMARK.md`'s limitations list and not to the engine's correctness, and the
section now states what the table supports — *an engine with a measured baseline and a
characterized gap*.

The paragraph claiming "the measured runs sit at 8–14% misses" is replaced. They did not: the
budget was 100,000 against 10,000 accounts, so nothing was ever evicted, the true rate was
zero, and the figure was typed into the renderer's prose with no column carrying it. The
measured rate is **1.00** — the wire path does not consult the partial view at all — which
makes the parity result a claim about reconstruction rather than about a warm cache.

§11.3 gains four refutation rows from this run: the two unmet Part 0 rows; that the served
answer was not the query the client sent; that the type checker did not implement λ_niles;
and that the SQL surface did not lower ten of the forms its table called `Lowered`. The
C2 falsifier row is rewritten as an existence claim per MISMATCH-F-08.

**The absence pattern is at fourteen instances, not three.** §11.5's paragraph naming
`Err(_) => 0`, `sum` over an empty group and `unwrap_or(LitBool(true))` as "three costumes"
now records the eleven more that a review looking for the shape found — every one written
after that paragraph — and states the rule that covers them: a wildcard arm in a soundness
predicate and an `unwrap_or` at a lowering site are the same defect.

§11.5.7's row claiming the `LitBool(true)` site is pinned is corrected: only the SQL surface
was; the pipeline surface kept it.

### [T-17] 2026-09-02T15:30Z RESULT F-10 — four status statements that contradicted each other

| where | said | now says |
|---|---|---|
| §1.6 | "**No measurements have been taken yet**" | measurements have been taken and some refuted the claim they tested |
| §3.15 | "Empirical validation: **None yet.**" | partial, with the sections that are measurement and the sections that are protocol named |
| Front matter | "no comparison against another database system is claimed" | E14 and E16 are claimed and reported, and two E16 rows are unmet |
| §1.9 | the same sentence | the same correction, with E16's durable rows named |

`bash thesis/build.sh` → exit 0. `Niles-Thesis.docx` rebuilt; **95,081 words** across the
thesis sources.

### [T-17] 2026-09-02T15:32Z BLOCKED-T-17-remainder the rest of T-17 is not done, and this says what

T-17 has fourteen numbered outputs. Five are done above (the three theory MISMATCHes, the
measured rewrites of §9.14.1/§11.3/§11.5.7, and F-10's status statements). The following are
**not done**, and no part of the thesis claims they are:

1. The single-sourced status statement from `thesis/status.toml` (output 1).
2. Identifier normalisation — `F1…F4`, `S1…S10`, `H0`, `SC1…SC6` to the `H-F`/`H-S`/`C`
   namespace — and the `no_bare_hypothesis_identifiers` drift test (output 2).
3. Appendix B's keyword lists generated from `keywords.rs` (output 3).
4. Appendix E's figures produced by a test (output 4).
5. `SPEC-LANGUAGE.md`'s per-L-id restructuring and generated Part V (output 5).
6. `ROADMAP.md`'s reorder rule and phase statuses (output 7).
7. §11.5.3/§11.5.4's rewording for the extended protocol and MySQL — **now materially
   wrong**, because T-14 wired the extended protocol and the MySQL row's "codec with no
   listener" is stated in `mysql_wire.rs` but not in the thesis (output 8, part).
8. `ARCHITECTURE.md` §7's generated counts and `README.md`'s stale figures (output 9).
9. F-51's "not measured" status lines with the missing instrument named (output 11).

Item 7 is the one that matters most, because it is a claim that has become *less* true
during this run rather than merely staying stale.

### [T-17] 2026-09-02T17:10Z RESULT the status of a claim was written in five places, and they disagreed

`thesis/status.toml` is now the single source: 21 claims — 14 hypotheses and 7 contributions —
each with a status, where it is reported, and, for anything not measured, **the instrument that
does not exist**. `include-results.py` renders it into §1.9.1's table, the Abstract's status
paragraph, §3.15's aggregate row and Appendix K.3, and `status_statement_is_single_sourced`
fails the build if any of the four stops being generated or if a hypothesis §1.6 declares has
no entry.

The parser refuses a claim that is `not measured` and names no missing instrument. That rule is
the point of the file rather than a nicety: *a hypothesis with no runner is a hypothesis with no
status*, and "to be measured" without naming what is missing is a promise wearing a result's
clothes.

**Seven hypotheses have no runner at all** — H-F1, H-F3, H-S6, H-S7, H-S8, H-S9, H-S10 — and
each row now says what would have to exist. Four more are partly measured with the unmeasured
half named. That is the honest shape of this thesis's evidence and it is now stated in one
place rather than reconstructed by a reader from five.

### [T-17] 2026-09-02T17:15Z RESULT identifier normalisation: `H0/H-S1`, `C3/H2`, and an `H1` nobody defined

Every claim identifier is now in one namespace: `H-F1…H-F4`, `H-S1…H-S10`, `C1…C6`, `SC7`.
Before, the same claim appeared as `H0` and `S1` in one sentence, as `SC3` and `H2` in another,
and §1.7.1's `C6` was called `H7` in Appendix J.

The worst of them was Appendix J.12's `H1`, which appeared three times **and was defined
nowhere in the document** — a section turning on what an identifier means, with no way for a
reader to find out. It is now `H-conv`, defined where it is raised, and explicitly not a
numbered hypothesis of §1.6: it is a conjecture this appendix leaves open, and the name says
which kind of object it is.

`no_bare_hypothesis_identifiers` greps every thesis file for the bare forms. Its own negative
control is `the_bare_identifier_check_has_teeth`, which checks that `H-S3` passes, `S3` fails,
and `CS1` — a citation to a study of 83 computer-science students — is not read as `S1`.

### [T-17] 2026-09-02T17:20Z RESULT four claims about the wire surface, three of them false

§11.5.4 listed "PostgreSQL wire protocol v3 including the extended query path, MySQL packet
framing, and the TLS negotiation state machines for both" as things this repository implements.
One of the three is true. The section now separates them:

* **PostgreSQL v3 — served.** Simple and extended paths, both wired into the connection loop,
  both driven by `psql`.
* **MySQL — a codec with no listener.** Frames encode and decode; nothing calls them.
* **TLS — a state machine with no provider.** `NoProvider` refuses every accept and the daemon
  runs `TlsConfig::insecure()`.

§11.5.3's list of what Nilestream "demonstrably provides beyond the PostgreSQL construction"
loses *"cross-shard commit whose coordinator is itself a ledger group"*: `persist_decision`
sets a boolean over an in-memory participant set, and the ledger group is a design stated in
the module's own documentation. A list of what a repository demonstrably provides may not carry
a design.

§11.5.7's five rows are corrected in place — each now says what the module *is* rather than
that it has tests — and §9.14.1 gains the limitation that explains one of its NOT MET verdicts:
**the daemon serves every query under one mutex**, so a 5–10× multiple against a 48-core
baseline is not reachable through a global lock. §9.13.3 gains the note that its group-commit
scaling is measured on the ledger crate and would not appear through the server.

### [T-17] 2026-09-02T17:25Z RESULT §4.6(d) claimed a policy no crate implements

*"a Landlord-style policy attains the k/(k−h+1) resource-augmented bound, and this is the
policy Nilestream implements rather than plain LRU."*

Three artefacts carry an eviction rule and none is Landlord. `nilestream-optimizer/eviction.rs`
is 52 lines of unused scaffolding with no Theorem 4.2 term. `nilestream-core`'s `CostAware`
approximates reconstruction cost by 1, making it an LFU. `proto-engine`'s `CostAware` weights
by cost *and* delayed-hit factor — **a different policy from the one sharing its name in the
runtime**, so the two engines Chapter 9 compares are not evicting alike. The claim is withdrawn
and §4.6's algorithm paragraph is marked *specified, not built*, with Φ(ℓ) labelled an assumed
constant.

### [T-17] 2026-09-02T17:30Z RESULT §6.6's "the compiler knows nothing about money" was false

Seven banking forms are keywords in the compiler's own registry — `txn`, `hold`, `resolve`,
`post`, `fx`, `conserve`, `idem` — each with an `Expr` variant and a case in the parser, the
lowering and the effect calculus. `Money` is a type constructor the typechecker knows by name.
A user library could not add any of them. What is general is the machinery underneath; what is
banking-specific is the surface over it, which is a weaker and different claim.

And §9.11's falsifier — a non-financial conservation domain built with no kernel changes — does
not exist. `grep -ri inventory` over the crates, the schemas and the examples returns nothing,
while §1.4 said Chapter 9 builds one. Both sentences are corrected and H-S8's status line says
the same thing.

### [T-17] 2026-09-02T17:35Z RESULT Appendix E quoted three figures and two were wrong

`parser.niles` was "~1,050 lines" against a file of 1,647; the self-application corpus was
"1,200 lines" against 2,068. The third — 127,165 bytes of tree — was correct, and I found that
out by *inventing a replacement for it* and having the new test reject the invention. That is
the whole argument for computing a figure rather than typing it, demonstrated on the person
writing the fix.

`stage_2_the_niles_front_end_parses_its_own_two_source_files` now measures all three and asserts
the appendix carries them.

### [T-17] 2026-09-02T17:40Z RESULT `SPEC-LANGUAGE.md` Part V counted four requirements that had no section

F-35, confirmed on all counts. L-3, L-4, L-7 and L-14 were counted **Built** in Part V and had
no section anywhere in the document. The prose said "Ten of twenty-four built" while its own
rows summed to eleven. L-8/L-9 said `Specified` beside a section describing the catalogue
checker that `ROADMAP.md` Phase 5 marks BUILT. Part IV item 5 said "It does not claim the
schedule verifier exists" — of a verifier that exists.

The four missing sections are written, each with its acceptance test and an honest status, and
**Part V is now generated from the sections** by `gen-spec-conformance`, checked by
`spec_conformance.rs`, and wired into `make generated` and `make reproduce`. A section with no
`Status:` line is an error rather than a default, with its own negative control — because
defaulting a missing status is exactly how L-8/L-9 came to read `Specified`.

L-24's table loses three **Built** marks: bitemporality is Partial (`recorded_at` and
`bitemporal` set a flag nothing reads), lineage is Specified (`explain`, `reproduce` and
`impact` parse and none lowers), and determinism is Partial with its citation corrected —
`IR013` is the reconstruction-path anchoring rule and says nothing about determinism. A rule
cited for a property it does not check is worse than no citation.

### [T-17] 2026-09-02T17:45Z RESULT §5.9 claimed two gates that do not exist

*"the build gate refuses to produce results from a dirty tree"* and *"the figure generator
refuses to emit a plot for which no template exists"*. Neither exists (F-42). §5.9 now lists
what **is** enforced — generated blocks, `make reproduce`'s diff, the single-sourced status,
the keyword and Appendix E figures — and says plainly that pre-registration here is a
discipline visible in `git log` rather than a gate. `results/E16-band.md` was committed before
the durable run and that ordering is the evidence; calling it a gate made an auditable claim
out of an intention.

### [T-17] 2026-09-02T17:50Z RESULT `ROADMAP.md`'s ordering rule was three rules read as one

The rule reads "sequenced by measured value", and flat it contradicts its own table: unnesting
is worth 510× and sits at phase 2, behind a phase-1 item whose cited result is a *reliability*
figure. The three findings are in three regimes — expressibility, tail reliability, throughput
— and are not commensurable.

**Decision: the rule is restated per regime and the phase numbers are left alone.** They are
cited by number in the thesis, in `SPEC-ENGINE.md` and in this document's own gates, and
renumbering to satisfy an ordering argument would break every citation to fix a table of
contents. The order actually taken already follows the corrected rule — phase 2 is built and
phase 1 is not — which is the clearest evidence that the flat reading was never the one in use.

Phase 1 is marked **NOT RUNNABLE** (its gate names the Join Order Benchmark, and neither JOB
nor a general scan-and-join surface exists), and phases 3, 4, 6 and 7 are marked **OUT OF
SCOPE** for this increment with a reason each. A status vocabulary is added so that BUILT, *not
runnable* and *out of scope* are three different statements rather than three shades of silence.

### [T-17] 2026-09-02T17:55Z TESTS niles 702/0/4 -> 729/0/5; thesis 101,216 words

`python3 thesis/include-results.py --check` → 0.
`cargo test -p bank-bench --test thesis_drift` → 8 passed, including `no_bare_hypothesis_identifiers`,
`appendix_b_keywords_match_registry` and `status_statement_is_single_sourced`.
`cargo test -p niles-lang --test spec_conformance` → 4 passed.
`grep -nE "\b(F[1-4]|S([1-9]|10)|H[0-9])\b" thesis/*.md | grep -v "H-F\|H-S\|SC7\|C[1-6]\|H-conv"` → no matches.
`grep -c "MISMATCH-F-07\|MISMATCH-F-08\|MISMATCH-F-09" docs/BUILD-LOG.md` → 3.
`bash thesis/build.sh` → exit 0, `Niles-Thesis.docx` rebuilt, **101,216 words**.
`make gate` → exit 0 in both repositories.

### [T-17] 2026-09-02T17:56Z DONE the nine outputs of `BLOCKED-T-17-remainder` are closed

### [T-17] 2026-09-02T18:20Z RESULT §6's validation protocol, run in full — and two things it found

Every V item run. Two of them found something, and both were real.

**V-02c (GC-12, no `HashMap` in the partial-state path).** `nilestream-core/src/rev.rs`'s test
module used `HashMap` for the reference base's checkpoints and read counts. They are never
iterated, so the answers were unaffected — and the reference base a determinism test is checked
*against* must not be the one thing in the experiment whose order depends on a hash seed. A rule
with an exception for the parts a reader is least likely to check is not a rule. `BTreeMap` now,
with the reasoning at the type.

**V-15d (the benchmark recipe).** `docs/BENCHMARK.md`'s one command read
`--operations 2000 --runs 10`, and the committed `results/E16-wallclock.md` was produced with
**500 and 5**. The single command a reader would type was not the command that produced the
numbers underneath it, which is the whole of what a reproduction recipe is for. The recipe is
corrected and `the_benchmark_recipe_reproduces_the_committed_numbers` reads the parameters back
out of the results file, so the two cannot drift again.

**Three §6 greps are stale rather than red**, and each is recorded here rather than worked
around:

* **V-09a / V-11b.** `grep -n "unwrap_or(0)"`, `grep -n "unwrap_or(Scalar::"` and
  `grep -n "_ => true"` all match — **inside comments explaining that the defect was removed**.
  The code is clean; the greps are not comment-aware. Re-run against non-comment lines they are
  empty, and that is the check that was meant.
* **V-11c.** The command names `mutants/auth_forged.niles`; the corpus file is
  `auth_forged_from_a_literal.niles` (there is a `_from_a_string` sibling, which is the point of
  the pair). `nilesc check` on the real path exits **1**.
* **V-15c.** The command reads column 10 of the E16 CSVs as the protocol path. T-15 added
  `miss_rate` and `protocol_path` after §6 was written, so column 10 is now `not_run` and the
  protocol path is column 11. Read correctly: one value, `simple`, across all four workloads,
  with `not_run` empty everywhere.

**One acceptance criterion is not met as literally written, by choice.** V-17e asks for 24
`### L-` headings, "combined sections split". There are **20 sections covering 24 requirement
identifiers**: `L-5 / L-23`, `L-8 / L-9`, `L-11 / L-18` and `L-16 / L-22` are each one section
because each pair is one design, and splitting them would produce four pairs of near-duplicate
prose to make a count come out. The property the criterion is after is enforced instead, by
`every_requirement_the_summary_counts_has_a_section`: each of L-1…L-24 appears in exactly one
heading, and the generated Part V states both numbers.

### [T-17] 2026-09-02T18:25Z RESULT the V table, in full

| ID | Result |
|---|---|
| V-0 | niles 729/0/5 · gbs 432/0/1 · adapter 38/0/0 — all ≥ floors, 0 failed |
| V-0b | pass; `layering.rs` diff empty |
| V-0c | `make gate` exit 0 in both |
| V-01 | both `channel = "1.95.0"` |
| V-02a/b/d | pass |
| V-02c | pass after the `BTreeMap` change above |
| V-03 | pass |
| V-04a/b/c | pass; `hit_path_comparisons` present |
| V-05a/b | pass; no stale banners |
| V-06a/b/c | pass |
| V-07a/b/c | pass; no `pub fn record_entries`; `fdatasync` observed |
| V-08 | pass |
| V-09a/b | pass (see the comment-grep note) |
| V-10 | 10 layering tests, 11 negative controls |
| V-11a/b/c | pass (see the corpus-name note) |
| V-12 | 15 pass |
| V-13a/b | pass |
| V-14a/b/c/d | pass; `extended::` referenced from `session.rs`; MySQL "codec, with no listener" in both places |
| V-15a | `fsync off` present in the OLTP baseline cell |
| V-15b | `E16-wallclock.md` reproduces; 0 `NOT RUN` |
| V-15c | one protocol path (`simple`) per run pair, at column 11 |
| V-15d | hardware, `pg_settings`, one command, runs, medians, limitations, NOT MET — all present |
| V-15e | `z` / `miss_rate` present in all three |
| V-15f | `E16-band.md`'s commit is an ancestor of the durable CSV's |
| V-16a | `make g3` → 29 rows, 0 failures |
| V-16b | pass; §7 reads "23 product-evidenced + 6 generic-path-only of 29" |
| V-17a/b/c/d | pass; 5 `MISMATCH-F-0[789]` mentions |
| V-17e | 20 sections / 24 ids — deviation recorded above |
| V-17f | `thesis/build.sh` exit 0; **101,216 words** |
| V-18 | **7 of 9 conform by execution**; the two divergences named with their fields |
| V-19 | `make reproduce` exit 0 |
| V-Σ | counts above, in both build logs |

### [T-17] 2026-09-02T18:35Z LC-4 Does the served answer depend on anything but the circuit, the parameters and the anchor?

*Question.* Does the served answer depend on anything other than the verified circuit, the
parameters and the anchor?

**Answer: No; `pick_view` and `extract_keys` no longer exist.**

Both are gone from `crates/nilestream-server/src/`, and the only occurrences of either name in
the repository are two comments recording what they did — `pick_view` returned the constant
`"__wire_result"` and `extract_keys` scraped digit runs out of the query *text*, so two
different questions about one account returned the same number and the compiler was a
decoration on a fixed answer.

`Session::query` now takes `(&Circuit, output_name, anchor)` and nothing else, and
`RevEngine::query` evaluates that circuit over the base at that anchor. The remaining
question-shaped dependence is the anchor-index **predicate pushdown**, which narrows the source
scan when it can read an `acct = k` restriction out of the circuit — and it is a narrowing of
the *input*, not a choice of answer, which is why it carries two obligations:
`the_pushdown_and_the_full_scan_agree` and
`a_predicate_the_pushdown_does_not_understand_abandons_the_restriction`. A pushdown that
guessed would be `pick_view` in a better disguise.

### [T-17] 2026-09-02T18:40Z LC-5 Every Part 0 row: module, column, protocol path, attribution

*Question.* For each Part 0 row, which workload module and which CSV column produced the
Measured value, over which protocol path, and to which limitations-list item is any NOT MET
attributed?

| Part 0 row | Workload module | CSV column | Protocol path | Verdict → attribution |
|---|---|---|---|---|
| **OLTP, durable, strictly serializable** (5–10×) | `bank-bench::workloads::oltp` → `results/E16-wallclock/oltp.csv` | `ops_per_sec`, median of 5 runs | `simple` (column `protocol_path`, one value per run pair) | **NOT MET** at 0.93× → `BENCHMARK.md` limitation **3** (the simple path compiles every statement afresh) and **6** (two cores against a 48-core baseline figure), plus the global lock recorded as limitation **1** and now stated in thesis §9.14.1 |
| **Scan-heavy analytical** (10–12×) | `bank-bench::workloads::analytical` → `analytical.csv` | `ops_per_sec`, median of 5 | `simple` | **NOT MET** at 0.13× → limitation **4** (an unkeyed `group by` materialises the whole base per query) and the three named constructs outside the lowered fragment, which make the row five PostgreSQL statements against three |
| **Point lookup by primary key** (parity) | `bank-bench::workloads::point` → `point.csv` | `p99_us`, median of 5; `miss_rate` alongside | `simple` | **PARITY** at 0.93×, with `miss_rate = 1.00` — every read an anchored reconstruction, which is a stronger result than parity with a warm cache and is stated as such |
| **Durable single-commit latency** (parity) | `bank-bench::workloads::durable` → `durable.csv` | `ops_per_sec`, median of 5; `durable` column asserted true by asking the server `select nilestream_durability` | `simple` | **PARITY** at 0.82×, `synchronous_commit = on` against `SyncPolicy::Always` on the same device |

No blank cell. The four Part 0 rows this table covers are the four the harness runs; the other
five rows of `SPEC-ENGINE.md`'s Part 0 table are marked *not available* there and no measured
value is claimed for any of them.

---

# §8 — Final report

Written at the end of the run, over both repositories. Every hash below is on the branch named
beside it; `review/thesis` is the tip that contains all of them.

## 8.1 Findings closed

Forty-eight findings are numbered in the work order (F-01…F-48 and F-51; there is no F-45,
F-49 or F-50). All forty-eight are closed. Commit and branch, by the task that closed each:

| F | Closed by | Commit | Branch | Repository |
|---|---|---|---|---|
| F-44 | T-01 | `8e0e051`, `4db597e`, `dd77bed` | `master` | both |
| F-01, F-02, F-03 (code), F-30, F-31, F-43 | T-02 | `bb71832`, `e6655f9` | `review/F-01-F-06` | niles |
| F-04 | T-03 | `3b0774e` | `review/F-01-F-06` | niles |
| F-05, F-06 | T-04 | `07383a6` · `3583347` | `review/F-01-F-06` | niles · gbs |
| F-03 (thesis), F-41 | T-05 | `e0d3fdd` | `review/F-01-F-06` | niles |
| F-28, F-29 | T-06 | `bb4da35` | `review/F-28-F-29` | niles |
| F-19, F-21, F-27 | T-07 | `3de8137` | `review/F-19-F-27` | gbs |
| F-20 | T-08 | `55b22d0` | `review/F-19-F-27` | gbs |
| F-22, F-26 | T-09 | `8904a91` | `review/F-19-F-27` | gbs |
| F-23 | T-10 | `1523d10`, `176bd4b` | `review/F-23` | gbs |
| F-11, F-37, F-46, F-47 | T-11 | `c666a8a`, `4ce84c4` · `d1e8627` | `review/F-11-F-13` | niles · gbs |
| F-24 | T-12 | `8216abf` · `c1dfe68` | `review/F-11-F-13` | niles · gbs |
| F-12, F-13 | T-13 | `9de19c9` | `review/F-11-F-13` | niles |
| F-16, F-17 | T-14 | `cf76b95` | `review/F-14-F-17` | niles |
| F-14, F-15, F-08 (Z half) | T-15 | `504c131`, `19984ea`, `2e35096` | `review/F-14-F-17` | niles |
| F-18 | T-16 | `913bf55`, `d000b9a`, `1cfc572` · `02e0725` | `review/F-18` | gbs · niles |
| F-07, F-09, F-10, F-25 (counts), F-32–F-36, F-38–F-40, F-42, F-51, F-08 (band half) | T-17 | `ff0e36e`, `701aa8e`, `6db10c0`, `ee5746b` · `1e8a152` | `review/thesis` · `review/F-18` | niles · gbs |
| F-25 (agreement half) | T-18 | `0626547`, `430176c` · `7f080c0` | `review/F-11-F-13`, `review/F-18` | niles · gbs |
| F-48 | T-19 | `3476018` | `review/F-14-F-17` | niles |

## 8.2 Findings not closed, and why

**None is left open.** Three are closed in a form weaker than a naive reading of the finding
would suggest, and each is recorded as such rather than counted quietly:

* **F-07** is closed as a **MISMATCH proposal, not an applied edit.** GC-02 forbids editing a
  theorem to match an implementation, so Theorem 4.3′ carries a fenced `PROPOSED` block naming
  the checkpoint interval C as a hypothesis, and the running text is unchanged. The thesis says
  what it always said; the proposal says what it should say and why.
* **F-08's band half** is closed by **stating that no band is derivable.** Θ(Z) has no constant
  anywhere in the thesis, so a two-sided band cannot be computed without inventing one.
  `results/E16-band.md` pre-registers the one-sided prediction that *is* derivable, and H-S2's
  status line says the theorem's band was never derived and is not tested.
* **F-51** is closed by **naming the missing instrument for every hypothesis without a runner**,
  which is a documentation outcome and not a measurement. Seven hypotheses still have no runner.

## 8.3 Every BLOCKED, MISMATCH and STALE raised, verbatim

Nine, across both build logs. Each is quoted where it was raised; the identifiers are:

* `BLOCKED-T-01-toolchain` (niles/gbs, T-01) — the pinned toolchain could not be installed in
  this environment; the pin is in `rust-toolchain.toml` and the build runs on what is present,
  with the deviation recorded.
* `BLOCKED-T-09-lifecycle` (gbs, T-09) — lifecycle transitions are not ledger events, so a
  transition's history is not reconstructible from the segment alone. Named, not worked around.
* `BLOCKED-T-17-remainder` (niles, T-17 partial) — the nine outputs of T-17 left undone at that
  commit. **Now closed**: all nine are done and the entry at 17:56Z says so.
* `BLOCKED-T-18-close_offering` (niles, T-18) — *conditional*: it is what
  `MISMATCH-T-18-close_offering` becomes if the schema is corrected, because the correction needs
  `resolve`, which the interpreter refuses by name.
* `MISMATCH-F-07` (Theorem 4.3′ with C as a hypothesis), `MISMATCH-F-08` (C2 reduced to
  existence, not prediction), `MISMATCH-F-09` (H-S1 with an explicit memory-price interval) —
  the three theory proposals, each written both to this log and as a fenced `PROPOSED` block at
  the claim it affects.
* `MISMATCH-T-11-overdraw` (λ_niles's T-Overdraw rule against what the checker realises).
* `MISMATCH-T-13-fixpoint` (Appendix H's completeness table against the lowered fixpoint).
* `MISMATCH-T-16-<row>` — **never raised.** No matrix row needed a `gbs-kernel` change (LC-3).
* `MISMATCH-T-18-normalised-fields` (two normalised fields specified, three needed).
* `MISMATCH-T-18-close_offering`, `MISMATCH-T-18-position_account` — the two conformance
  divergences, pinned in `KNOWN_DIVERGENCES` with their exact fields.
* `STALE-F-11g` (niles, T-11) — a finding pointer that no longer matched the code when it was
  reached.

## 8.4 Test counts, before and after

| Workspace | Before (T-01 floor) | After | Δ |
|---|---|---|---|
| `niles` | 600 / 0 / 3 | **729 / 0 / 5** | +129 |
| `gbs` (workspace) | 415 / 0 / 1 | **432 / 0 / 1** | +17 |
| `gbs` adapter (`gbs-nilestream`) | 15 / 0 / 0 | **38 / 0 / 0** | +23 |

Passed / failed / ignored. The ignored count rose by two in `niles` and both are generators run
by `make reproduce` (`gen-sql-surface`'s corpus regenerator and `psql_conformance`'s transcript),
which GC-13 permits because they are run by a committed command rather than skipped.

## 8.5 Branches and merge order

```
master               T-01              (both repositories)
review/F-01-F-06     T-02 → T-03 → T-04 → T-05        niles (+ the gbs oracle test)
review/F-28-F-29     T-06                             niles
review/F-19-F-27     T-07 → T-08 → T-09               gbs
review/F-23          T-10                             gbs   (stacked on review/F-19-F-27)
review/F-11-F-13     T-11 → T-12 → T-13 → T-18        niles (+ gbs schema commits)
review/F-14-F-17     T-14 → T-15 → T-19               niles (merged review/F-28-F-29)
review/F-18          T-16, and T-18's GBS side        both
review/thesis        T-17, §6, §7, §8                 niles
```

**Merge order:** `master` → `review/F-01-F-06` → `review/F-28-F-29` → `review/F-19-F-27` →
`review/F-23` → `review/F-11-F-13` → `review/F-14-F-17` → `review/F-18` → `review/thesis`.

**Three deviations from §5.0, each recorded where it was taken.** `review/F-23` stacked on
`review/F-19-F-27` rather than standing alone (T-10). `review/F-18` in GBS additionally stacked
on `review/F-11-F-13`, because the Niles checkout it builds against carries T-11's checker and
the pre-T-12 schema does not pass it — a schema checked by an older checker than the one it
ships with is not checked (T-16). T-18's GBS side landed on `review/F-18` rather than
`review/F-11-F-13`, because the rewritten `conformance.rs` uses `Session::over`, which is
T-16's, and does not compile on the earlier branch (T-18). The intended *ordering* is preserved
in every case.

## 8.6 Thesis claims to be weakened, downgraded or dropped — with the replacement wording

Every entry below is either already applied at this commit (marked **applied**) or is a
proposal that GC-02 forbids applying (marked **proposed**, and carried as a fenced `PROPOSED`
block at the claim).

**C2 — existence, not prediction.** *(proposed; `MISMATCH-F-08`)*
> Replace: "the measured cost of partial materialization crosses that of full materialization
> **within the band predicted by** the Eviction–Consistency Frontier Theorem."
> With: "a crossover in the price of memory exists and is located by E4 and E12. **The
> theorem's band is not derived and is not tested**: `(1 + Θ(Z))` carries no stated constant
> anywhere in this thesis, and a band computed with the constant at 1 and one computed with it
> at 10 differ by an order of magnitude in exactly the region the experiment measures.
> `results/E16-band.md` pre-registers the one-sided prediction that is derivable and states the
> falsifier for it."

**C3 — Theorem 4.3′ with C as a hypothesis.** *(proposed; `MISMATCH-F-07`)*
> Add to the theorem's parameter list the checkpoint interval **C**, and to its hypotheses:
> "the implementation maintains per-key checkpoints at interval C." The upper bound becomes
> bounded by C and per-key update density rather than by base length n; the lower bound becomes
> Ω(min(C, deltas since anchor)) base touches per miss in the restricted model, with checkpoints
> as a stated hypothesis. §3.14's Φ, §3.15's verification table, §1.6's H-S3 and §4.8 take the
> matching edits.

**C4 — the implementation clause, as T-11 realised it.** *(applied)*
> The soundness theorem is proved for λ_niles. What the *checker* now realises is stated
> separately and not conflated with it: seventeen mutants are refused, each by its own
> diagnostic code, each with an accepted well-typed neighbour; effect rows are transitive
> through calls with a fixpoint over mutually recursive functions; a read at a rung a
> declaration does not name is refused in both directions. H-S4's status is **partly measured**,
> and what is missing is named: no corpus of well-typed programs is executed on Nilestream under
> crash-recovery, eviction and adversarial schedules with an oracle counting violations. The
> claim quantifies over executions; the evidence is about the checker.

**C6(c) — the narrowed fragment.** *(applied)*
> The stated SQL fragment has 34 forms. Eight are proved *equivalent* — the SQL and pipeline
> spellings denote the same Z-set on the golden corpus — thirteen are lowered with a golden case
> in one surface, **four are refused each with the diagnostic code that refuses it**, two are
> lowered with nothing checking what they compute and say so, two are specified, and five are
> deliberate exclusions carrying their reasons. A fragment with four named refusals in it is not
> a supersession, and `SPEC-LANGUAGE.md` L-5/L-23 now reads **Partial**.

**E2 — what is wired.** *(applied)*
> "MySQL and PostgreSQL wire compatibility" becomes: **PostgreSQL v3 is served** — simple and
> extended paths, both in the connection loop, both driven by `psql`. **MySQL packet framing is
> a codec with no listener.** **TLS is a negotiation state machine with no provider**:
> `NoProvider` refuses every accept and the daemon runs `TlsConfig::insecure()`. `distributed.rs`
> and `cross_shard.rs` are models nothing calls; the claim that the cross-shard coordinator "is
> itself a ledger group" is withdrawn — `persist_decision` sets a boolean.

**E3 — the oracle now in use.** *(applied)*
> `conservation-suite::oracle` is the correctness oracle for `proto-engine`, `nilestream-core`
> and, through the adapter, GBS's kernel, with differential tests at anchors drawn uniformly
> from `[0, head]` rather than at the head alone, and a fault campaign (crash, truncate,
> evict-storm, duplicate delivery, reorder) that is tests rather than stubs. E1's "independent
> oracle" comment is true now; it named a self-comparison when it was written.

**G2 — measured baseline, characterized gap.** *(applied)*
> "§7's performance contract states four targets relative to PostgreSQL" becomes: all four rows
> are measured on both engines over the same protocol path. **Two say NOT MET** — 0.93× on
> durable OLTP against 5–10×, and 0.13× on analytical against 10–12× — each attributed to a
> numbered item of `BENCHMARK.md`'s limitations list and none to the engine's correctness. Two
> say PARITY, and the `point` row is at a **measured miss rate of 1.00**: every read an anchored
> reconstruction, which is a stronger result than parity with a warm cache. What the table
> supports is an engine with a measured baseline and a characterized gap, not a performance
> claim.

**G3 — the Phase 8 verdict.** *(applied)*
> "Run the eleven implemented product lines against Nilestream" becomes **"23 product-evidenced
> + 6 generic-path-only of 29"**, with the six named — *Clearing and prime brokerage*, *M&A and
> capital raising*, *Philanthropy*, *Specialised financing*, *Trust services*, *Wealth
> planning* — and with `wire` reported per row: two shapes cross the PostgreSQL wire and
> twenty-seven are in-process. No matrix row needed a `gbs-kernel` change.

**F-51's hypotheses — "not measured; instrument absent: …".** *(applied, from
`thesis/status.toml`)*
> * **H-F1** — not measured; instrument absent: no long-horizon experiment, no live-generating
>   source, no finite-first comparison stack, no regression of per-answer cost on accumulated
>   input.
> * **H-F3** — not measured; instrument absent: no comparative audit. §9.7 fixes the counting
>   rules; nothing counts.
> * **H-S4** — partly measured; instrument absent: no execution campaign over well-typed
>   programs with an oracle counting violations.
> * **H-S5** — partly measured; instrument absent: no client-compatibility pass rate, and no
>   MySQL listener at all.
> * **H-S6** — not measured; instrument absent: no ported corpus and no trigger-based SQL
>   baseline, so neither side of the comparison has been run; the runtime-overhead half has no
>   measurement of any kind.
> * **H-S7** — not measured; instrument absent: the adaptive optimizer is not built, and
>   `offline.rs` is a planner rather than a dynamic program, so there is no offline optimum to
>   measure a competitive ratio against.
> * **H-S8** — not measured; instrument absent: the non-financial conservation domain does not
>   exist, and seven banking forms are keywords in the compiler's registry rather than a library
>   over it.
> * **H-S9** — not measured; instrument absent: `nilestream-lineage` is stubs, so there is no
>   lineage mode, no audit corpus, no completeness figure and no overhead figure.
> * **H-S10** — not measured; instrument absent: every instrument in this repository is serial
>   where the experiment needs concurrency.

**Two further withdrawals not on §8's minimum list, because the run found them.** *(applied)*
> **§4.6(d)** — "a Landlord-style policy … is the policy Nilestream implements rather than plain
> LRU" is withdrawn. None of the three eviction rules in the repository is Landlord, two share a
> name and are different policies, and Φ(ℓ) is an assumed constant.
> **§6.6** — "the compiler knows nothing about money as such" is withdrawn. Seven banking forms
> are keywords in the compiler's own registry with cases in the parser, the lowering and the
> effect calculus, and no user library could add one.

## 8.7 The recurring defect, at fifteen

`Err(_) => 0`, a `sum` over an empty group, `unwrap_or(LitBool(true))`, and eleven more found by
looking for the shape rather than waiting for it — all the same defect: **an absence given a
reasonable default that is a wrong answer wearing a plausible shape.**

The fifteenth arrived during T-16 and wears a new disguise. `Rev::read(k, a)` treats `a` as a
floor on freshness, so an entry certified at epoch 21 is a *hit* for a question about epoch 8,
and an audit query came back thirteen transfers out of date. Not a zero standing in for an
unknown: a **fresher** answer standing in for the one asked about, every digit of it true.

The rule the repositories now follow is unchanged and now has a second half: an absence gets a
named representation or a diagnostic and never a default — **and an answer carries the anchor it
is true at, which a caller must read rather than assume.**

---

# Cycle: Work Order 2 (the Fable audit)

`review/thesis` @ `27eb990`, `review/F-18` @ `9f451ba`. niles 757/0/5 (from 730/0/5); gbs
443/0/1 and adapter 40/0 (from 432/0/1 and 38/0). Both gates green, `make reproduce` clean.
The full account is `docs/WORK-ORDER-2-REPORT.md`; this entry is the short version and the
things worth remembering.

**The audit's headline was right and its details were often wrong, in both directions.** Of
28 findings, 10 were not confirmed on inspection — double consumption *is* caught, `sim.rs`
*does* honour partitions, `coverage.rs` and `purity.rs` and `layering.rs` are stronger than
reported, the oracle has two balance definitions and not three. Two were larger than
reported: the consensus suite survived a weakened quorum rule *for a different reason* than
the audit gave, and the adapter's scale blindness turned out to mean the instrument rows had
been running at the wrong scale entirely.

**What the fixes found that nobody had reported.** Writing the SQL corpus's missing cases
found that RIGHT and FULL joins evaluated to nothing and CROSS JOIN answered the equi-join —
parsed, lowered, and passed the IR verifier. Running H-S8's falsifier found a three-letter
restriction on grades in the lexer. Running the citation checker found that a drift test had
rewritten a paper's title: reference [90] read "H-F1 Lightning: HTAP as a service" because
`no_bare_hypothesis_identifiers` flagged the bare `F1` and someone obliged.

**The theorems now say what their proofs prove.** Theorem 4.1 for Q_lin with the general case
open; Theorem 4.2 as one theorem and two corollaries, with the condition under which a
break-even exists and no claim over policies; Theorem 4.4's clause (1) about the blocks the
solver decides, clause (4) conditional on P6, and the transfer to LTS traces named as a
sketched lemma; Theorem 3.7 as an expectation; Theorem 4.6(c) as denotation-preservation over
the fragment the compiler accepts. C5 is `specified`; H-S9 is withdrawn; H-F1 and H-F3 are
`argued`; H-S8 and H-S6 moved from unmeasured to partly measured because instruments were
built for them.

**Two new status values exist because "not measured" was promising instruments that were
never coming.** `specified` for a design nothing implements; `argued` for a claim no
instrument will settle.

**The thing to carry forward.** Three separate defects this cycle had the same shape: two
sides of a comparison sharing a derivation or a constant. E2 compared a telescoping sum with
its own parts; the novation check compared `n` with `-n`; both arms of G3 folded at a
hard-coded scale. Each was green, each was documented as catching what it could not catch. A
systematic pass asking *does one side of this equality derive from the other?* is the highest-
value instrument this repository does not have.
