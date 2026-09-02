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
