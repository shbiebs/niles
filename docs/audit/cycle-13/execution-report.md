# Cycle 13 — the builder's execution report

*Eight commits in `niles`, one in `gbs`. Seven of the work order's twenty-eight cards.
`make gate` exit 0 on both repositories at every commit; `make reproduce` exit 0 on a clean
`niles` tree.*

This report is written to be argued with. Every number in it was produced by a command in
one of the two repositories, and where a card's acceptance sentence was not met, the section
says which clause and why rather than reporting the card as done.

---

## 0. What landed

| Card | Commit | Branch | What it closed |
|---|---|---|---|
| **E-3** | `e8d1597` | `c13/01-provenance` | every regenerable results file carries its configuration; §9's two stale tables generated |
| **T-1** | `cae097d` | `c13/02-status-truth` | `status.toml` tells the truth about four claims; the A-8 keyword decision |
| **L-1** | `0f76152` | `c13/07-checker-resolves` | the checker resolves its names and checks its `let` annotations |
| **E-2** | `8d33ce6`, `e3e3b63`, `ef5dbac` | `c13/02-lock-as-it-is` | E19 re-measured on the lock the daemon has; the provenance header it never had |
| **E-4** | `28c84e0` | `c13/03-checkpoints-on` | `MISMATCH-daemon-checkpoints` split into the part that was real, the part that was immaterial, and the part that was misattributed |
| **T-3** | `cdc4ec1` | `c13/05-e14-filed` | E14 filed as language evidence, and scored by the compiler that exists |
| **G-1** | `5b9c42d` (gbs) | `c13/01-nonnegative-boundary` | the signed-quantity class, closed at the boundaries and listed everywhere else |

The `niles` branches are a **linear chain** in the order above, so `c13/05-e14-filed` is the
cycle head and contains everything. The work order assigns branch names per card and two of
them share the `c13/02-` prefix (`c13/02-status-truth` is mine; the order named
`c13/02-lock-as-it-is` for E-2). No collision, and it is recorded here rather than left for
the author to notice.

**Not started, and why.** E-1 — the card the cycle is for — needs `docs/universe.md`, which
needs decision **A-2**. S-1 needs the same. P-1 needs **A-6**, **A-7** and **A-11**. L-2, L-3,
L-4 and the M, K, D and P tranches were below the line once the critical path stopped at E-4.

---

## 1. Seven things that were not true, and are now

Listed by how much they change what the thesis may claim, not by card order.

### 1.1 The checker accepted programs that are not programs

`nilesc check` printed `ok` and exited 0 on all three of these:

```niles
let z: Text = m;        // m: Money<usd>
no_such_function(1)
unbound_name
```

Theorem 4.4 opens *Let ⊢ P : T ! ε*, so every clause is hypothetical on the judgement and
the artifact's contribution to the theorem is the fidelity of `nilesc check` to ⊢. The
fidelity had a hole at the *base* of the type system rather than at its interesting edges,
and the set the artifact quantified over was correspondingly larger than the calculus
describes — larger in the least interesting way.

The middle case was the worst. `typecheck::call` looked the callee up in the program's
summaries and, not finding it, fell through: the call contributed no effects to its caller's
row and no movement to its conservation obligation. **The obligation did not move to the
commit rule under clause (1). It ceased to exist.**

Now NL0256, NL0205 and NL0204, each with a mutant and a well-typed neighbour. Four limits
are *stated* rather than closed — multi-segment paths, type positions, SQL subtrees, bare
names in query context — and each is a test in `tests/name_resolution.rs`, so narrowing one
later means deleting the test that says it was wide.

**It found two things immediately.** The solver corpus called `daily_accrual` and `abort`,
neither declared anywhere — and the case calling `daily_accrual` exists to establish that
*an opaque call is a symbol and a symbol cancels with itself*. It was calling nothing.

### 1.2 E14's verdicts came from a compiler six cycles old

`counterproposal.rs` took whatever `nilesc` binary it found in `target/`, preferring
`release`, and built one only if none existed. The release binary in this container was
dated **2026-09-07** against a HEAD of **2026-09-21**. A stale binary is a present binary,
and presence was the only thing checked — by a test whose own header explains at length how
it was hardened against a binary that could not *run*.

That is worse than the defect it replaced. A blocked test announces itself; a test scoring
an old compiler reports confident numbers about a language state that no longer exists, and
those numbers are §6.10.3's table and §11.5's sentence.

Scored by the current compiler: **12 refused at compile time, 1 warned, 0 accepted in
silence**, of 13. The thesis said 11 / 1 / 1. Two cells moved and neither moved because
anyone was looking at this table — `d6`'s silent acceptance fell to L-1's NL0205, and `d10`
was listed *not expressible* when a `ledger` with no `conserve` clause is refused with
NL0211 and NL0300. A rule that cannot be dropped at run time *because it cannot be omitted
at compile time* is the stronger result, and the cell was under-reporting it.

### 1.3 The daemon's lock had not been what three normative places said for seven cycles

`d681f0d` (cycle 6, **2026-09-05**) replaced `Arc<Mutex<RevEngine>>` with a `Serving` trait
object implemented by `std::sync::RwLock<RevEngine>`. E19's rows were committed by `fa012ed`
on **2026-09-04** — verified from git, not taken from the audit; `f7cdfdd` and `4a8f63d`
touched the document afterwards and neither re-measured.

Each of the three places drew a *conclusion* from the mutex: that reads serialise against
each other, that a second core cannot be used for reads over an immutable base, that a
client-side group-commit experiment would measure the lock rather than the sealer. None was
tested, because the benchmark that would have tested them drives one connection. **A
sentence no measurement depends on is a sentence no measurement can contradict.**

Re-measured, three interleaved replicates at 1/2/4/8 connections on two granted cores:

| workload | engine 2 → 4 | engine 4 → 8 | PostgreSQL control |
|---|--:|--:|---|
| `durable` | 1.72× ± 0.24 | 1.74× ± 0.19 | 1.23× ± 0.07, 1.22× ± 0.04 |
| `fold` | 1.03× ± 0.04 | 0.95× ± 0.04 | 1.13× ± 0.05, 0.99× ± 0.07 |
| `point` | 1.44× ± 0.53 | 1.55× ± 0.34 | 0.66× ± 0.28, 0.85× ± 0.24 |

The mutex-era rows had the engine **flat at 0.98×** over 2 → 4 with the verdict column
reading *the added connection buys nothing measurable*. On `fold` both arms flatten above
four connections, which on two cores is the host. On `point` the four-connection MADs are
22% and 28% of their medians, so **no ratio there is separated from noise and none is
claimed**.

**What is not claimed:** that the lock caused it. Different container instances, and this
repository has a documented case of an instance alone moving a contract row from MET to NOT
MET. `results/E19-scaling-historical.md` keeps the old rows, states that objection, and gives
the one comparison that survives it — each era's engine ratio relative to its own
within-session PostgreSQL control, 0.70 then and 1.40 now, a change in sign. The experiment
that would settle it builds both engine versions and interleaves them in one session. It has
not been run, and `Provenance`'s *none named* baseline field records that in every run.

### 1.4 The checkpoint caveat was three different claims wearing one marker

**Real.** E8 is H-S3's read-side leg and ran only at `checkpoint_interval = 0` — the
configuration §9.4.1 says the cost law excludes — while H-S3's other leg, E10, is the one
experiment that varies C. Rerun at C ∈ {0, 16, 64}:

* the paired lax:strict read ratio is **2.52× ± 0.05 at C = 0** and **1.98× ± 0.00 at
  C = 16**. The MAD is zero to two places, which two deterministic counts over one workload
  should give and which the C = 0 ratio does not;
* **everything except the read column is invariant in C** — misses, deltas applied,
  maintenance passes and hit rate are identical to the unit at all three intervals. *A
  checkpoint changes what a reconstruction costs, not whether a read misses;*
* C = 64 behaves like C = 0, and a new `checkpointed_keys` column says why: C = 16 covers 54
  keys of 10,000, C = 64 covers 11. **Removing a quarter of a lax rung's read cost took
  bounding 0.54% of the keys** — §1.2's Pareto argument turning up somewhere nobody put it.

**Immaterial.** At E16's own seeding (`--rounds 1`) every account key holds one posting, so
no interval above 1 records anything for any of them. At C = 16 the mechanism covers **one
key of 10,001** — the house, which `point` does not read. Coverage of the accounts begins
exactly at `rounds = C`. The caveat has been read for four cycles as *these figures would be
different*; it should be read as *the configuration was not stated, and at this seeding the
mechanism does not engage.*

**Misattributed.** E23's `serve_path` column reads `fold` and `report-from-view`; the path a
checkpoint bounds is `IndexFold`, which appears in neither series. Struck, with the struck
text kept.

### 1.5 The register's own text was altered on its way to the reader

`status.toml` is TOML, so a quotation mark inside a value is spelled `\"`.
`include-results.py` stripped the surrounding quotes and never looked inside, so the first
instrument text to quote a thesis sentence rendered into §1.6's table as a literal backslash
before every quote. **Nothing failed:** the generator wrote it, and `--check` compared it
against itself and agreed.

That is this project's characteristic failure one level down — not two copies that disagree,
but one copy silently transformed in transit. A register whose text is altered on the way to
the reader is not a single source.

### 1.6 A header that lied about a dirty tree, in both directions

E19 carried no provenance header at all. Adding one found two defects in the mechanism it
reuses:

* the build-time `NILES_DIRTY` stamp goes stale in the most ordinary state there is. A build
  script emitting any `cargo:rerun-if-changed` loses cargo's default "re-run when a file in
  this package changes"; this one names `.git/HEAD` and `.git/index`, and **neither moves
  when a tracked file is edited and not staged.** Observed, not reasoned about: the E19
  document produced two commits earlier printed a commit hash with no warning beside it from
  a tree with four modified tracked files;
* the run-time check that replaced it was correct and **taken too late**. This harness writes
  *tracked* files under `results/`, so a check after the first CSV lands reports the run's own
  writes — it would have said "dirty" on every invocation from every tree, ever. A flag that
  is always set carries exactly as much information as one that is never set.

Both fields are read once now, by `Provenance::init()` as the first statement of `main`. The
second field needed it for its own reason: `scaling_document` calls `gather` twice, the
session id is the clock in seconds, and the block's own sentence reads *both arms of every
ratio below come from this one invocation*. E19's last two runs produced `6ab14040` and
`6ab1413b` from a single invocation.

### 1.7 A conserving ledger cannot see a sign error

Cycle 11's S-4 recorded one bug in `lending::drawdown`. It is a class, and larger than the
audit's eight: **21 more** public amount-taking functions were found by the structural test
G-1 added.

A negative drawdown is two legs summing to zero. So is a redemption of −5 units, a
distribution that collects from its beneficiaries, and a participant set `[3/2, −1/2]` that
allocates one and a half times the whole to the first party and takes half from the second.
**They conserve.** Conservation is the wrong instrument for a sign error, which is why nine
cycles of a conservation-checked ledger did not notice.

Seven boundaries take a `NonNegative` now; three of those (`repay`, `reimburse`,
`subscribe`) had **no bound at all in either direction** and gain their missing upper bound
as well. Three more are guarded in place. Of the 21 the test found, seven are `Signed` by
design and eleven are `Deferred` with what each waits on — several need an accounting
judgement about which direction a quantity may run, which is **A-11**, and a builder
guessing at it would be putting a guess in a type. The deferred count is asserted, so the set
can shrink without a decision and cannot grow without one.

`securities::subscribe` is the audit's ninth site, found while converting `redeem`.

---

## 2. Two process findings

**`make gate` does not run `make reproduce`.** E18's `ledger_seeded` allocation count was
stale at HEAD by 40 — which is L-1's own name-resolution pass, 40 allocations per process
over 40,000 postings, per-op unchanged at 3.8 against a 4.2 budget. `make reproduce` catches
it; the gate does not run `make reproduce`; so it survived the gate on L-1's own commit and
was found two cards later. A byte-deterministic artefact whose regeneration is not in the
gate is guarded only by whoever remembers to run it.

**Two readers still read by position.** `render_medians` took column *indices*, so adding
`checkpoint_interval` as E8's third column shifted every index after it and the renderer
published `42 misses` for all three rungs — 42 is a seed. Nothing failed: the table was
well-formed, plausible, and about different quantities than its headings said. It resolves
names against the header now. The repository already has
`a_stale_header_is_refused_rather_than_read_by_position` for its CSV readers; this markdown
renderer was the one that still counted. `results_manifest.rs`'s reader and
`counterproposal.rs`'s `find("pub fn {name}")` prefix match were the same shape and are
fixed.

---

## 3. Deviations from the work order, each with its reason

1. **L-1's diagnostics are NL0204/NL0205, not NL01xx.** The card said "new code, NL01xx";
   NL01xx is the *lexer's* range (NL0100 is `unexpected character`). NL02xx is resolution's,
   and NL0204/NL0205 were free.
2. **L-1's pass lives in `names.rs` and is called from `resolve_program`,** rather than being
   written into `resolve.rs`. Same entry point and same guarantee — there are thirteen call
   sites of the compiler in these two repositories and a check wired into one of them is a
   check the project does not have — but the code is its own module because it needs an
   exhaustive `Expr` walk with no wildcard arm, and `resolve.rs` is about declarations.
3. **E-2 used three replicates, not two.** The card said "twice, interleaved"; the committed
   E19 protocol is three and reports medians, and two replicates make a median the mean of
   two.
4. **E-4 did not re-publish E16 at C = 16.** The card asked for E16-point at C ∈ {0, 16}.
   Re-publishing E16 moves the *contract* table on wall-clock noise, and the measurement that
   decides the question is cheaper and exact: at E16's seeding no interval above 1 covers an
   account, held by a test. The flag exists and the next recipe that deepens the accounts
   gets a failing test rather than a silent caveat.
5. **E-4's checkpoint statement lives in `MANIFEST.csv`, not in the eighteen artefacts.**
   The guard accepts either. Writing `C = 0` into files measured before the flag existed
   would be indistinguishable from a measured column, and this project does not edit a header
   onto data taken without it.
6. **T-3 corrected its own first correction.** Reading `defects.sql` showed ten executed
   cases and I wrote that only ten classes are scored; E14's own table scores all thirteen on
   both sides, three of them without an executed case. PostgreSQL's totals are unchanged at
   3 runtime / 0 compile / 9 never / 1 not expressible. The intermediate wrong version is
   recorded here because it was committed to nothing but was believed for twenty minutes.
7. **G-1 converted seven boundaries and deferred eleven**, against a card that named eight
   sites. All eight are closed; the eleven are beyond the card and several are A-11's.

---

## 4. What the author is owed, and what is owed to the author

**Blocking the critical path:**

| decision | blocks |
|---|---|
| **A-2** — the universe, the row, the sampling rate | `docs/universe.md`, and with it **E-1** and **S-1** |
| **A-6, A-7, A-11** | **P-1** |
| **A-11** alone (cycle 11's LC-41) | `check_limit`'s soundness; eleven deferred GBS boundaries |

**Manual steps.** None were required this cycle: PostgreSQL 16 is running in the container,
the toolchain resolves as `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` under `RUSTUP_TOOLCHAIN=stable`,
and every experiment run here is one the container can run. The author's fetch/push blocks
are in §5.

**A question of mine.** E-2's cross-era comparison is the strongest thing the two sets of
E19 rows support and it is not controlled. The experiment that would control it — build the
cycle-6 engine and the current one, interleave them in one session on one instance — is
perhaps a day, and it would convert *the engine scales better than it did* from an
indication into a result. It is not in the work order. Should it be a cycle-14 card?
