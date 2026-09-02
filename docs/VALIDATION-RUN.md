# The Opus handoff blueprint — validation protocol, run

*Every item of `HANDOFF-OPUS.md` Part 3, run in order, with what it actually reported.*

Two items are **not met**, and both are recorded here as gaps rather than reworded into
passes. Three are met by a check under a different name from the one the protocol
predicted, which is noted per row so a reader can find it.

Reproduce: `cd niles && cargo test --workspace`; `cd gbs && NILES_ROOT=../niles cargo test
--workspace`. Individual commands are in the rows.

## Result

| ID | Proves | Pass criterion | Measured | |
|---|---|---|---|---|
| V-0 | GC-2 baseline | niles ≥ 430, gbs ≥ 258, 0 fail | **600** and **415**, 0 fail | ✅ |
| V-0b | GC-3 | layering passes, file unchanged | 7 tests, file untouched | ✅ |
| V-1a | F3 gone | no cached-quantity fields | no matches | ✅ |
| V-1b | F10 gone | `&mut self` only where declared | 9 files, all in `purity.rs`'s ALLOWED with a reason | ✅ |
| V-1c | F9 seen | `seen` is a lookup | passes | ✅ |
| V-1d | F9 signal | a balance touches only its account's entries | passes | ✅ |
| V-1e | F13 | a corner-unbalanced generator is refused | passes, as `a_generator_that_only_fails_at_a_boundary_is_refused_by_the_declared_space` | ✅ |
| V-1f | crash | `replay_` and `crash_` all pass | 8 and 7 | ✅ |
| V-2a | F2 | reopen, duplicate-after-reopen, truncation, chain re-walk | 15 tests | ✅ |
| V-2b | F2 honesty | `NotDurable` only in the sync-failure branch | 3 sites: one doc line, two sync branches | ✅ |
| V-2c | fsync real | ≥ 1 fsync per committed set | `exactly one fsync per committed set, not two`, counted over 1,000 | ✅ |
| V-3a | F1 calibration | within ±30% of 333 txn/s/core | **not met — the figure was wrong.** See below | ⚠️ |
| V-3b | F1 table | no `NOT RUN` rows | **not met — three rows remain.** See below | ❌ |
| V-3c | F14 | `phase_diagram.csv` present in `E16-wallclock/` | **not produced.** See below | ❌ |
| V-3d | GC-1 | every ratio has a CSV name or the word *predicted* nearby | 26 ratios, all sourced | ✅ |
| V-4a | unnesting fires | every corpus query's rewrite matches expected | 24 cases, 16 tests | ✅ |
| V-4b | not-in NULL | 5+ NULL cases give the SQL result | **8**, each also against a hand-written oracle | ✅ |
| V-4c | rungs preserved | rung monotonicity on rewritten circuits | contract carried node for node; `verify` passes on every unnested plan | ✅ |
| V-5a | F5 fragment | `NotInCatalogue` present; Phase 5 kill criterion removed | 7 sites; ROADMAP Phase 5 is a checker | ✅ |
| V-5b | denotation | 1,000 random Z-set trials per rule | passes, now over the shared evaluator | ✅ |
| V-5c | negative | pushing a select across both sides is refused | passes | ✅ |
| V-6a | F7 | ≥ 40 functions, rates present, defective all caught | 40 correct + 5 defective; 0 undecided | ✅ |
| V-6b | no weakening | currency-row tests unchanged and passing | passes | ✅ |
| V-7a | F6 | NIST CAVP vectors | passes | ✅ |
| V-7b | stale build | the placeholder is gone | passes | ✅ |
| V-7c | no dep | no external crate added | 1 `[dependencies]`, empty | ✅ |
| V-8a | F8 | lexer gates (14) + parser gates (≥ 6) | **14 + 16** | ✅ |
| V-8b | fixpoint | serialised AST identical across stages | passes, including the front end over its own two files | ✅ |
| V-9a | F4 | conformance byte-identical | **9 of 9 shapes**, not 29. See below | ⚠️ |
| V-9b | F12 | every claimed mechanism is called | 11 tests | ✅ |
| V-10 | F9 matching | counted ops linear; no `remove(0)` | passes; the two `remove(0)` matches are comments explaining why it is not used | ✅ |
| V-11a | F11 | the pending-cross-target sentence is present | present in C.5 | ✅ |
| V-11b | docx | builds, word count logged | 93,129 words, 302 KB | ✅ |

## The three that are not clean

**V-3a — the calibration target was wrong, and the gate is what found it.** The protocol
asked for a figure within ±30% of 333 txn/s/core. The harness's plausibility gate fired at
16× over its ceiling, and the gate was right: 333 txn/s/core had been carried from a device
with power-loss protection, where an `fsync` costs 1.6–12.4 µs, to one without, where it
costs 891–2,974 µs. The number is not a property of the software. Calibration is now
measured per device at run time, and `docs/BENCHMARK.md` records the method rather than the
figure. Marked ⚠️ rather than ✅ because the criterion as written cannot be met, and
rewriting it to match what was measured would be the thing this file exists to avoid.

**V-3b — three `NOT RUN` rows remain, for an architectural reason.** `nilestreamd` exposes
no write surface over the wire, so `oltp` and `durable` have nothing to drive; its read
surface serves per-key balances, so `analytical` has no scan to run. Filling those rows
needs a write path and a scan surface on the server, which is engineering that was not in
the blueprint. The rows are labelled `NOT RUN` with the reason in the results file and in
thesis §9.14.1, which is the honest state: a prediction printed beside a measurement in the
same typography is how a specification becomes marketing.

**V-3c — no phase-diagram overlay.** The item depends on a wall-clock phase sweep, which
depends on V-3b's missing surfaces. The counted-work phase diagram in §9.3.3 and the
compiled-path one in §9.13.2 stand; the wall-clock overlay does not exist and is not
claimed anywhere.

**V-9a — nine shapes, not twenty-nine, and on purpose.** All 29 matrix rows are
implemented, and the conformance surface covers 9 *distinct posting shapes*. Five advisory
rows share one composition and therefore one shape; writing five identical schema functions
to make the count reach 29 would widen the number without widening what is checked. A test
enumerates the product modules and requires each one that posts to have a case, so a module
added later without a shape fails there. `analytics.rs` is exempt because it posts nothing,
and the exemption is checked in its source rather than taken on trust.

## The findings the protocol produced

Running the protocol is not the same as running the tests, and three of these came from the
protocol rather than from the suite:

* the calibration figure (V-3a), which no test could have questioned because it was a
  constant nobody had measured on this device;
* the `NOT RUN` rows (V-3b), which a suite passes over in silence because a test that is
  not written does not fail;
* the conformance count (V-9a), where the discrepancy between 9 and 29 turned out to be a
  fact about the products rather than a gap in the coverage.
