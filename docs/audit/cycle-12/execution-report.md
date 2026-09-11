# Cycle 12 — the builder's execution report

*Written against `docs/audit/cycle-11/fable-foundations-work-order.md` (GBS
`c11/fable-foundations-work-order`, `e477216`), the Fable foundations audit. It reports the
eight cards the author cut from §4.1: **F-INT, F-X1, F-H1, F-H2, F-H3, F-H5, F-H7, F-M1**.
The remaining five prerequisite cards — F-I1, F-E1, F-L1, F-S1, F-D1 — are `not done: not
reached` and §6 says so one by one.*

*This report lives in niles by LC-50: execution reports beside the code they cite, work orders
and audit briefs in private GBS.*

---

## 0. The decisions the author made, and where each took effect

| decision | answer | where it landed |
|---|---|---|
| session scope | F-INT + the six small cards + F-M1 | eight cards, ten commits, two repositories |
| **the `answers()` allocation** (new, §2 F-INT.1) | change the trait to return a borrow | `Base::answers(&self) -> &BasePlan`, niles `cc8cc66` / gbs `f964918` |
| **F-H3's manifest half** (new, §2 F-H3) | a Rust-owned schema + validator; the shell keeps writing | `bank_bench::manifest`, `bench --check-manifest`; recorded as `BLOCKED-branch-F-H3` |
| **the marker leak under F-M1** (new, §2 F-M1) | ship the metadata half; record the slot leak as its own card | `Rev::slots_len`, an asserted finding, and §7's first cycle-13 card |

Nothing in the work order's §7.1 was answered, and §7 below lists what each unanswered one
still blocks.

---

## 1. What ran, and on what

Every command in this report ran in the cloud container. **Nothing here is a Host C
measurement and no row below is a performance claim.** The allocation counts are deterministic
and are the only numbers this report asserts.

| | |
|---|---|
| host | Linux 6.18.44-fc-v24, x86_64, 2 cores, 7 GiB, ext4 |
| compiler | `rustc 1.95.0 (59807616e 2026-04-14)`, selected as `RUSTUP_TOOLCHAIN=stable` |
| admissibility | the named toolchain `1.95.0` is not installed and cannot be (no egress); `stable` *is* 1.95.0 here. Every row is a **diagnostic run under an alias**, not exact-pin acceptance — the audit's own distinction, and the reason F-X1 below cannot be witnessed here |
| 1.97.1 | **not installed.** F-X1's guard is Host C's own lint row |
| PostgreSQL | 16.13 on 127.0.0.1:5432, `fsync=on`, `synchronous_commit=on`, `wal_sync_method=fdatasync` |
| offline | `CARGO_NET_OFFLINE=true`, `RUSTUP_AUTO_INSTALL=0` throughout; nothing downloaded, nothing installed |

**The five protected files** were never touched: nothing in this cycle reads, stages or writes
on Host C, and every git command there is in the author's own Run-now block (§5).

---

## 2. Every target sentence, verbatim, with its verdict

### F-INT — integrate the tranche, then re-gate

> *The cycle-11 head reaches every tranche-1 commit, `downstream_adapter` is green against the
> paired GBS head on Host C under the exact pin, and the README's test count equals the
> recount.*

**done in the container; the Host C exact-pin half is the author's run (§5).** niles
`c11/11-cycle-head` (`49b820f`) reaches all eight cycle-11 commits including the two the
previous head omitted; GBS `c11/05-cycle-head` (`4938430`) carries both trait methods on one
`JournalBase`. `downstream_adapter` is green against that pair here. The README holds the
recount.

*What the merge actually was.* One content conflict, and it is the two cards meeting:
C11-05(c) changed `merge_suffix` to return `Option<Merged>`; C11-01 added `require_base(base)`
as its first statement. **The resolution is their conjunction** — taking either side would have
silently dropped a card — with a comment saying so at the site.

*Two required trait methods landed on different branches, so every test fixture was missing
one.* The library built; four test binaries did not. `RawBase` gained `delta_rows_at` counting
*source* rows (not `deltas_at(e).len()`, which is both the defect C11-05(a) removed and the
wrong number, since `deltas_at` returns one row per group); `Wide`, `Stops` and `Rows` gained
`answers()`. **`tools/memprobe` needed the same repair three times** — the S-2 shape again.

*The README count is recomputed, not merged.* Both siblings raised it from a common 939, to
1003 and 943; either side taken whole is a conflict resolved into a false statement. The merged
tree holds 1,014, and 1,036 by the end of the cycle.

### F-INT.1 — a base lends its plan (new; the author's decision)

Not a card in the order. **Integrating C11-01 put three E18 budgets over at once**, and the
reason it had never been seen is the shape cycle 11 named twice: `fbae068` made `answers()`
required and did not add it to `tools/memprobe`, which is `exclude`d from the workspace — so
C11-01's own branch could not build memprobe, `make memory` and `make reproduce` could not run
there, and the per-read cost was never measured on the branch that introduced it.

`require_base` runs on every miss, every `apply_epoch` and every `merge_suffix`, and
`BasePlan::sum` does `relation.to_string()` plus a `Vec`:

| scenario | before C11-01 | with C11-01, owned | with the borrow |
|---|--:|--:|--:|
| `ledger_seeded` | 3.8 | **4.8** | 3.8 |
| `append_in_memory` | 7.0 | **9.0** | 7.0 |
| `rev_metadata_2x_budget` (budget 7.5) | 6.8 | **8.8** | 6.8 |

The budgets were not raised. `Base::answers` returns `&BasePlan`; ten implementors store it
once — a `OnceLock` where the plan is constant, `&self.plan` for `fragment_matrix`'s `RawBase`,
whose plan varies per case and is the reason the signature is a borrow rather than a
`&'static`. **Guard:** restore the per-call construction and memprobe reports the same three
rows at 4.8 / 9.0 / 8.8, `3 scenario(s) are over budget`, exit 1.

### F-X1 — the one 1.97.1 lint

> *niles clippy on 1.97.1 is clean on Host C.*

**not done here: it cannot be. The repair is made; its guard is Host C's.** `block.z` is
`BTreeMap<Row, i128>` and the loop bound the weight only to discard it; it iterates `keys()`
now. **The container cannot witness it, and that is checked rather than assumed:**
`clippy::for_kv_map` *exists* in 1.95.0 — an unknown name produces `warning[E0602]: unknown
lint` and this one does not — but denying it against the unfixed code on 1.95.0 exits 0. The
lint is present and does not fire; 1.97.1 sees through the type alias where 1.95.0 does not.
So this is the reverse of A11-06: genuinely toolchain-dependent.

### F-H1 — `make reproduce` builds what it runs

> *`make reproduce` exits 0 from a fresh worktree on Host C, and the E18 byte rows it
> regenerates carry the Host C banner.*

**done in the container from a fresh worktree pair; the Host C half is the author's run.**
Three defects, in the order a fresh tree meets them:

1. **the sweep's binary was never built by the target that runs it** — the prerequisite lived
   in a comment. Every green container `reproduce` row in every cycle's record ran in a warm
   tree;
2. **the path was hardcoded.** `./target/release/nilestream` is wrong whenever
   `CARGO_TARGET_DIR` is set, which is how a fresh worktree is built — so adding a build line
   alone still failed at 127. It runs through `cargo run` now, which builds what it runs and
   knows where it is;
3. **`results/obligations.csv` is not byte-deterministic; it is pair-dependent, and it failed
   silently.** Its first row is `../gbs/niles/gbs.niles`, carrying **11 of the corpus's 16
   obligations**. `corpus()` walked three roots and `continue`d past one it could not read, so
   a checkout with no sibling GBS wrote `TOTAL,,5,0` where the committed file says
   `TOTAL,,16,0`, `MANIFEST.csv` still called it `byte-deterministic`, and the diff failed with
   nothing saying why. **That is the Appendix D failure mode in a second place: a missing input
   that produces a smaller answer looks exactly like a correct one.** The file's own doc
   comment reads *"the corpus is deliberately all of them and not a selection: choosing which
   programs to count is choosing the answer"* — and the walk was choosing.

The pair is required now, with the opt-out `downstream_adapter` already uses; `NILES_NO_GBS`
suppresses the *write* rather than shrinking the number.

**Guards:** the `cargo run` line reverted → `Error 127`, Host C's failure reproduced; a fresh
tree with no sibling gbs → `corpus_obligations` refuses naming the path and "Eleven of sixteen
obligations live there", exit 101; the same with `NILES_NO_GBS=1` → measures, prints that the
count is over a subset, writes nothing, exit 0. **Acceptance: `make reproduce` exits 0 from a
fresh `<root>/niles` + `<root>/gbs` pair with an empty target dir** — the first time that has
been true.

### F-H2 — the arrival-gap probe makes a claim

> *At gap ≥ 1 the merge arm's hit rate exceeds the pinned arm's by more than 50 points … it
> cannot die silently again.*

**done.** The probe armed `MergeCaps::default()`, which C11-05(b) set to `OFF`; from that
commit both arms were the pinned arm, the second was labelled "merge", and five lines of
identical numbers printed. `#[ignore]`d so no gate ran it, assertion-free so nothing noticed.
It is armed with `MergeCaps::ON` by name, asserts the separation (97.8 vs 0.0 at gap 1, so the
threshold is nowhere near the measurement), and `make gate` runs it in a new `measurements`
target. The numbers are unchanged from `653a369`, so this repairs the instrument and moves no
result. **Guard:** restore `default()` → *"a separation of 0.0 points … the two arms are not
being armed differently"*, exit 101.

### F-H3 — the scorer's exit code says what its summary says

> *A scoring run with any refused row exits non-zero and the harness's Block B/D pass line
> reads that exit.*

**done, with the manifest half deviating and recorded.** `bench --score` printed its refusal
count and returned 0 regardless. The audit's injection is now a committed test: one replicate's
`reads_per_second` renamed, the arm correctly excluded, four replicates where five are
required, **18 of 36 rows refused, exit 3**; the clean fixture is the control at 0 refused,
exit 0. "Noise-limited" is not a refusal — it is a scored row whose separation did not clear
the gate. The decision lives in `score::exit_code`, so the *interface* has a test.

**`BLOCKED-branch-F-H3`:** the card says move `manifest.csv`'s writing into `bench
--manifest`. The writer stays in the shell, because the manifest is written before anything is
built and the first stages it records are the *builds* — a manifest written by `bench` could
not record that `bench` failed to build, which inverts the property C11-06.3 added it for. What
moved is the **schema**: `bank_bench::manifest` owns the header, the three outcome words and
their meanings, with eight tests — including that `not run` keeps its space, because an
underscore there is a schema change that looks like a tidy-up and would stop every archived
manifest parsing. `bench --check-manifest` refuses a stale header, an unknown outcome or a
short row, naming the line.

**Also fixed here because this card's own test caught it:** F-H1 put its comment block at
column 0 *inside* the `reproduce` recipe, and `results_manifest.rs` parses that recipe by
taking tab-prefixed lines — so the comment truncated the recipe and the test reported "no `git
diff --exit-code` in the `reproduce` recipe". A gate that reads a Makefile by shape is a gate a
comment can blind, and it said so.

### F-H5 — the lock-order string names the locks that exist

> *A letter with no lock, or a lock with no letter, fails and names it.*

**done.** `O < B < P < V < C` appeared in five source sites and twenty-odd audit documents.
**P did not exist** — the `Mutex<Vec<Pending>>` it named was removed, and `daemon.rs:265`
*asserts* the removal, so the lock's absence was guarded while the sentence describing it was
not. The engine's lock-bearing fields are `ledger`, `runtime`, `currencies`. The five source
sites read `O < B < V < C`; `docs/audit/**` is evidence and is not rewritten.

**The test failed twice on itself before it worked, and both are the local defect class.** Its
first version matched its own doc comment quoting the old string as history; its second matched
its own filter line, which necessarily contains what it searches for. The needle is built with
`concat!` now — the split `thesis_drift` already uses for `concat!("#[", "test]")`. **Guards:**
restore `P` in `rev.rs` alone → the test names the file and quotes the line; drop `C`'s row from
the table → `left: ["B","V","F"], right: ["B","V","C","F"]`. Both exit 101.

### F-H7 — the marker inventory is generated and checked

> *The committed inventory at the cycle head lists every marker an unrestricted grep finds.*

**done, at the baseline it names rather than at the head.** The cycle-11 file lists **34**
markers where a grep at the same commit finds **50**. Three faults, each of the kind that makes
an inventory quietly smaller: it excluded `docs/audit/**`, truncated each name at the first
`_`, and walked the working tree so an untracked file counted. `docs/audit/tools/markers.py`
reads **tracked files only**, at a named commit, over the whole tree, taking whole names — and
**does not decide whether a marker is live**, because cycle 11 recorded a disagreement about
`MISMATCH-A9-F05` and a tool that guessed would have hidden it.

The corrected inventory is `docs/audit/cycle-12/markers-653a369.txt` with a `SUPERSEDES` note;
the cycle-11 file stays where it is. Three tests: the committed file equals the generator's
output; the generator's self-test passes **and still contains its three cases**, since a
self-test whose cases were deleted exits 0; and the inventory stays larger than the one it
supersedes and ≥ 50. The artifact names the baseline rather than the head because an inventory
*of* its own head would contain its own hash. **Guards:** delete a marker line → the comparison
fails naming the regeneration command; restore the `docs/audit/**` exclusion → both the
self-test case and the comparison fail. Both exit 101.

### F-M1 — per-key metadata bounded on every read path

> *After 10⁶ distinct keys read once each through every read path with budget 10³,
> `metadata_len ≤ 10³` and E18's live bytes for the scenario are flat; the wire's
> `view_metadata_keys` is bounded by the budget.*

**the metadata clause is done and guarded; the live-bytes clause is `not done`, and §3 S3 is
why.**

The repair: the entry is created in `install_at`, where residency begins and before
`enforce_budget` ranks candidates; a read bumps an entry that exists rather than creating one.
Neither policy could ever read a non-resident key's entry — both filter to `resident()` first —
so it was pure growth. All four non-installing paths now leave zero, with two controls: one
entry per resident key exactly, and **both policies still evict correctly**, because a repair
that bounded the map by removing what the policy reads would pass every other assertion and
break eviction.

**An unasked-for improvement, measured.** `entry(key.clone())` allocated a `Vec` on every read
including a hit: `rev_read_hit` falls from **1.0 to 0.0 alloc/op**, `rev_metadata_per_key` from
2.2 to 1.0. The card's budget was "`rev_read_hit` stays 1.0"; it is below it.

**Guard:** restore the creation site and three tests fail with the audit's numbers — 10,000
against 0 for abandoned reads, 10,256 against 256 for refused admissions, 1,000 against 0 for
uninstalled folds. Exit 101.

---

## 3. Every material surprise

Six. Four are findings about the code, one is a defect I introduced and the gate caught, one is
a test that failed on itself twice.

**S1 — C11-01 shipped a per-read allocation that its own branch could not measure** (EV 20).
§2 F-INT.1. The mechanism is the workspace-exclusion shape cycle 11 named twice (S-1, S-2), and
this is its third instance: a required trait method added without the excluded crate, so the
crate that measures memory could not build on the branch that changed the cost of every read.
The general rule the audit proposed — *every crate excluded from a workspace is listed with the
command that covers it, and the gate runs that command* — would have caught it, and is still
not implemented.

**S2 — `make reproduce`'s third defect is a corpus that shrinks in silence** (EV 18). §2 F-H1.
`obligations.csv` is classed `byte-deterministic` in `MANIFEST.csv` and depends on a sibling
checkout for 11 of its 16 obligations. Two of the three faults in that target were about
*where things are*; this one is about *what counts*, and it is the failure mode the Appendix D
generator already had.

**S3 — the leak under F-M1 is larger than F-M1's** (EV 18). With the metadata bounded, 10,000
abandoned reads leave `slots = 10_000`, `meta = 0`, `in_flight = 16`, `resident = 0`. An
abandoned read of a cold key leaves a `Slot::Pending` that nothing removes: `reap_cancelled`
runs only when that key is read *again*, and for a cold key it has no `prior` to restore
anyway; `is_resident` is false for a marker so eviction never selects one. Sound while the
flight is live, **unbounded once it is not** — about 125 bytes a key, and `rev_metadata_churn`
measures `live 12,544,928` for 100,000 keys.

Not repaired here, by the author's decision: the fix is eviction seeing markers whose flight is
gone, which changes the two-phase read's state machine (LC-30). `Rev::slots_len` is public so
the next card measures rather than infers, and the defect's **present size is asserted** — if
it falls the leak was repaired and that test becomes the assertion that it stays repaired; if
it rises, something else is retaining markers. *This is the answer to the audit's §1.1 Q3: the
bounded-total-memory half is still contradicted, and now by a named, measured mechanism rather
than by the one the audit found.*

**S4 — the arrival-gap probe had been dead since `14fd117`** (EV 12). §2 F-H2. Confirmed by
running it at two refs. Nothing else in the tree measures the merge against the pinned arm, so
between C11-05(b) and this commit the project had no working comparison of the two — during the
cycle that changed the merge.

**S5 — two harness gates blinded each other** (EV 10). F-H1's comment block truncated the
recipe that `results_manifest.rs` parses, and F-H3's test found it. Neither is wrong: a Makefile
comment at column 0 is legal, and parsing a recipe by tab-prefix is reasonable. The pair is the
finding, and the cheap general rule is that a recipe's prose belongs above its target.

**S6 — my own test matched itself twice** (EV 8). §2 F-H5. First its doc comment, then its
filter line. Both were caught by running it; neither would have been caught by reading it. The
project already has the fix (`concat!`) and already had the lesson; I reproduced the defect
anyway, which is the argument for running a guard rather than trusting one.

---

## 4. What this cycle changes about the audit's findings

| audit finding | what it said | what this cycle establishes |
|---|---|---|
| **H6 / S4** | the tranche-1 head omits C11-01 and C11-07 | confirmed and repaired; the merge was substantive, not mechanical (§2 F-INT) |
| **H1 / S6** | `make reproduce` needs a binary it does not build | confirmed, **and two further defects in the same target** (§2 F-H1) |
| **H2** | the arrival-gap probe is dead | confirmed and repaired; it now asserts its own shape |
| **H3** | the scorer exits 0 with rows refused | confirmed and repaired; 18 of 36 is now a test |
| **H5 / S3** | the documented lock order names a `P` that does not exist | confirmed and repaired, with a test that makes the order a claim about code |
| **H7** | the committed inventory undercounts, 34 vs 50 | confirmed exactly; regenerated, and the generator's own scope is tested |
| **S2 (F-11-18)** | metadata grows with distinct keys read | confirmed and repaired — **and it was the smaller of two leaks on that path** (§3 S3) |
| **§1.1 Q3** | bounded cache yes, bounded total no | unchanged in conclusion, sharpened in mechanism: history is O(N) by policy and *markers* are O(distinct cold keys read) by defect, with a number |
| **A11-06 / F-X1** | the 1.97.1 lint | the cast was toolchain-independent; **this one is toolchain-dependent**, and that is checked by denying the lint on 1.95.0 against the unfixed code |

---

## 5. Every SHA, and what the author runs

**niles**, all reachable from `c11/11-cycle-head` upward; bundle
`niles-c12-tranche.bundle`, sha256 `936edc9012262b1b2dfb3571dfa04d244f95f0760b708231fc15d2db20bb2b7f`:

| branch | head | card |
|---|---|---|
| `c11/11-cycle-head` | `71dd553` | F-INT (merges `e70be56`, `49b820f`) **and F-X1** (`71dd553`) |
| `c12/01-plan-borrow` | `cc8cc66` | F-INT.1 |
| `c12/02-reproduce-builds` | `1317700` | F-H1 |
| `c12/03-probe-alive` | `fc9d9c1` | F-H2 |
| `c12/04-scorer-exit` | `4e962f9` | F-H3 |
| `c12/05-lock-order` | `5cf6520` | F-H5 |
| `c12/06-markers` | `47f7f46` | F-H7 |
| `c12/07-metadata-bounded` | `921b4b7` | F-M1 (+ `F-M1.1`, Appendix D) |

**`BLOCKED-branch-F-X1`.** F-X1's one-line commit is on `c11/11-cycle-head` rather than a
branch of its own: I committed it before cutting `c12/01-plan-borrow` and noticed when the
bundle's head did not match this table. It is recorded rather than rewritten — the branch is
not yet anywhere but here, so rewriting was available and the deviation is the more useful
artifact. The effect is that the cycle head carries a test-only lint fix; nothing else moves.

**GBS**; bundle `gbs-c12-tranche.bundle`, sha256
`6801c1aa2baf1c799629fae191424b2d8db711c47f95e2e92f7186c2060f1e3a`:

| branch | head | card |
|---|---|---|
| `c11/05-cycle-head` | `4938430` | F-INT (merge of `2e0e4b4` into `c653fdf`) |
| `c12/01-plan-borrow` | `f964918` | F-INT.1 |

The niles branches are a linear chain, so each can be merged in order and any one can be the
stopping point. **Merge GBS first**: `c11/11-cycle-head` needs `answers()` on the GBS side, and
`c12/01-plan-borrow` needs the borrowed signature there.

---

## 6. Container validation, per landing

`cargo test --offline --workspace --no-fail-fast`, `cargo fmt --all -- --check`,
`cargo clippy --offline --workspace --all-targets -- -D warnings`, all at
`RUSTUP_TOOLCHAIN=stable` with PostgreSQL 16.13 up:

| pair | niles suite | gbs | adapter | fmt/clippy |
|---|---|---|---|---|
| `49b820f` / `4938430` | 76 suites, **1,162 / 0 / 8** | 504 / 0 / 1 | 53 / 0 | clean |
| `921b4b7` / `f964918` | 78 suites, **1,185 / 0 / 8** | 504 / 0 / 1 | 53 / 0 | clean |

`make gate` **exit 0** at the final head, including the new `measurements` target.
`make reproduce` **exit 0** from a fresh `<root>/niles` + `<root>/gbs` worktree pair with an
empty `CARGO_TARGET_DIR`. E18 at the final head: `ledger_seeded` 3.8, `append_in_memory` 7.0,
`rev_metadata_2x_budget` 6.8, `rev_read_hit` **0.0**, `rev_metadata_per_key` 1.0,
`rev_metadata_churn` 7.4 — no scenario over budget.

**MF-12 as practice.** Every source edit was an exact-string single-occurrence Python
replacement asserting its match count before writing. Three failed their assertion and were
rewritten rather than forced: the Makefile comment move (two occurrences of the anchor), the
`reproduce` block extraction (the slice found a different `reproduce:`), and the metadata probe
removal. `bash -n` and `cargo build` caught the rest.

---

## 7. What did not run, and why

Every card below is `not done: not reached` unless the reason is more specific.

| card | status |
|---|---|
| **F-I1** the anchor/stamp/delta instrument | **not done: not reached.** It is the next card and the one the audit says gates everything downstream — C11-05(d), LC-23/24, LC-39, LC-43. |
| **F-E1** the engine-value comparator | not done: not reached. Needs LC-49/51 or the synthetic label before its decision workload means anything. |
| **F-L1** the language-value experiment | not done: not reached. |
| **F-S1** the skew protocol | not done: not reached. |
| **F-D1** the decision catalogue | **not done: not reached, and blocked besides.** LC-20's policy answer is still owed (§8). |
| **F-M2, F-K1, F-T1, F-P1, F-P2** | below the prerequisites line by the audit's own sequencing. |
| the marker/slot leak repair (S3) | **new, and the first card cycle 13 should take** — the measurement is committed and the before-value is asserted. |

**Nothing was compressed to fit.** Where a target sentence could not be met, §2 says which
clause and why: F-X1's compiler, F-H1's and F-INT's Host C halves, F-H3's manifest deviation,
F-M1's live-bytes clause.

---

## 8. Decisions still owed

| decision | blocks |
|---|---|
| **LC-20** — is the cycle-7 text the definition, and then (a) thesis sentence / (b) a session can ask / (c) strike? | F-D1's marker strike; one line of the decision catalogue |
| **LC-49 / LC-51** — the trace universe and the four scale numbers | F-E1's decision workload, K0a/K0b, the phase diagram's header |
| **the exposure-monotonicity rule** (S-4 from cycle 11, LC-41) | needs an accounting reviewer, not a builder |
| **LC-48 per component** | after F-E1 and F-L1 |
| **LC-52** — `master` is 166 and 50 commits behind, both fast-forwardable | nothing technical; every reader who lands on `master` |
| **E18 on Host C** — commit Darwin byte rows with the banner, or restore | F-H1's Host C acceptance |
| **LC-42** — does the order gain an `H` | nothing now, but the string it would be added to is tested as of F-H5 |

**Two questions of mine.**

1. **The slot leak (S3) versus LC-30.** The repair is eviction selecting a `Pending` marker
   whose flight is gone. The predicate has to be exactly "no live flight for this key", and
   getting it wrong breaks generation-owned installs. Do you want that as cycle 13's first
   card, or do you want the LC-30 protocol sentence confirmed first?
2. **The workspace-exclusion rule.** Three instances in two cycles (S-1, S-2, and S1 here). The
   audit proposed: *every crate excluded from a workspace is listed with the command that
   covers it, and the gate runs that command; an exclusion with no covering command is a red
   row.* That is a small card. Should it go in cycle 13 ahead of F-I1?
