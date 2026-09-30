# E30b′ — does the language need its own grammar? (cycle 15, C15-05; decision DA-1)

*Rendered by `cargo run -p syntax-study --bin e30b -- report` from the rows in `results/E30b-syntax/`, which `e30b measure`, `e30b mutate`, `e30b adversarial` and the memory probe's `e30bcost` wrote at commit `ada7f0b8ec51`. The design, pre-registered before any program or checker change, is `docs/study/E30b-design.md` (commit `163456d`); the corpus was committed before any addition was built (`7b5425c`). The SQL+C+L additions are `634f289` (effect annotations), `cfcfbff` (body typing) and `93c529a` (NSQ002); the niles-interp additions are `25e6df7`. E30's results (`results/E30-syntax.md`) are not overwritten.*

## The rule, evaluated

> The Niles grammar is kept for exactly the defect classes that the upgraded SQL+C+L cannot refuse at check time on the adversarial corpus. If it refuses all of them, annotated SQL is the language, and Niles is its specification and reference checker. Check time per KLOC above 2× is reported as a cost, not a veto.

A class **counts for the Niles grammar** when SQL+C+L accepts a defective program of it that Niles refuses, or cannot express while it can express the correct twin (design §1). A case whose correct twin Niles cannot write is *outside Niles*, and counts for neither language.

| case | class | SQL+C+L on the defective program | Niles on it | Niles writes the correct twin | counts for the Niles grammar |
|---|---|---|---|---|---|
| A01 | schema-level omission | SQL-R1: accepted | refused (NL0320) | yes | **yes** |
| A02 | schema-level omission | SQL-R1: accepted | refused (NL0300) | yes | **yes** |
| A03 | dynamic execute | SQL-R1: refused (NSQ002) | not expressible | no | no — SQL+C+L refuses it |
| A04 | dynamic execute | SQL-R1: accepted | not expressible | no | no — outside Niles (neither twin expressible) |
| A05 | functions calling functions | SQL-R1: accepted | accepted | yes | no — Niles accepts it too |
| A06 | functions calling functions | SQL-R1: refused (NL0310) | refused (NL0310) | yes | no — SQL+C+L refuses it |
| A07 | triggers | SQL-R1: accepted | not expressible | no | no — outside Niles (neither twin expressible) |
| A08 | triggers | SQL-R1: refused (NL0230) | not expressible | no | no — SQL+C+L refuses it |
| A09 | declared-effect mismatch | SQL-R1: refused (NL0310) | refused (NL0310) | yes | no — SQL+C+L refuses it |
| A10 | declared-effect mismatch | SQL-R1: refused (NL0310) | refused (NL0310) | yes | no — SQL+C+L refuses it |
| A11 | currency swap, typed position | SQL-R1: refused (NL0255); SQL-R2: refused (NL0255) | not expressible | yes | no — SQL+C+L refuses it |
| A12 | currency swap, untyped position | SQL-R1: refused (NL0255); SQL-R2: refused (NL0255) | not expressible | yes | no — SQL+C+L refuses it |
| A13 | back-valued correction | SQL-R1: refused (NL0230) | not expressible | no | no — SQL+C+L refuses it |
| A14 | back-valued correction | SQL-R1: accepted | not expressible | no | no — outside Niles (neither twin expressible) |

**Verdict: 1 of the 8 classes in `cases.tsv` counts for the Niles grammar:** *schema-level omission* (A01, A02). In every other class the upgraded SQL+C+L refuses what Niles refuses, or neither refuses it, or the case lies outside Niles. By the rule, the Niles grammar is kept for exactly the class named, and for no other.

* **Accepted by both checkers:** A05.
* **Accepted by SQL+C+L and outside Niles** (Niles writes neither twin): A04, A07, A14.
* **False refusals** (a correct twin refused): SQL+C+L A03; Niles none.

## The adversarial corpus, every program

`crates/syntax-study/corpus/e30b/adversarial/`, with `cases.tsv`. Every correct SQL twin was also run on PostgreSQL 16 against the dataset, in `begin … rollback`, with the deferred conservation check forced; every correct Niles twin was run by `niles-interp` where it can be.

| case | surface | twin | verdict | codes | run |
|---|---|---|---|---|---|
| A01 | SQL-R1 | defective | accepted |  |  |
| A01 | SQL-R1 | correct | accepted |  | runs; 200 postings after |
| A01 | NL | defective | refused | NL0320 |  |
| A01 | NL | correct | accepted |  | runs; 0 legs |
| A02 | SQL-R1 | defective | accepted |  |  |
| A02 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A02 | NL | defective | refused | NL0300 |  |
| A02 | NL | correct | accepted |  | runs; 2 legs |
| A03 | SQL-R1 | defective | refused | NSQ002 |  |
| A03 | SQL-R1 | correct | refused | NSQ002 | runs; 202 postings after |
| A03 | NL | defective | not expressible |  | Niles has no dynamic SQL |
| A03 | NL | correct | not expressible |  | Niles has no dynamic SQL |
| A04 | SQL-R1 | defective | accepted |  |  |
| A04 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A04 | NL | defective | not expressible |  | Niles has no dynamic SQL |
| A04 | NL | correct | not expressible |  | Niles has no dynamic SQL |
| A05 | SQL-R1 | defective | accepted |  |  |
| A05 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A05 | NL | defective | accepted |  |  |
| A05 | NL | correct | accepted |  | runs; 2 legs |
| A06 | SQL-R1 | defective | refused | NL0310 |  |
| A06 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A06 | NL | defective | refused | NL0310 |  |
| A06 | NL | correct | accepted |  | runs; 2 legs |
| A07 | SQL-R1 | defective | accepted |  |  |
| A07 | SQL-R1 | correct | accepted |  | runs; 204 postings after |
| A07 | NL | defective | not expressible |  | Niles has no triggers |
| A07 | NL | correct | not expressible |  | Niles has no triggers |
| A08 | SQL-R1 | defective | refused | NL0230 |  |
| A08 | SQL-R1 | correct | accepted |  | runs; 204 postings after |
| A08 | NL | defective | not expressible |  | Niles has no triggers |
| A08 | NL | correct | not expressible |  | Niles has no triggers |
| A09 | SQL-R1 | defective | refused | NL0310 |  |
| A09 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A09 | NL | defective | refused | NL0310 |  |
| A09 | NL | correct | accepted |  | runs; 2 legs |
| A10 | SQL-R1 | defective | refused | NL0310 |  |
| A10 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A10 | NL | defective | refused | NL0310 |  |
| A10 | NL | correct | accepted |  | unexecuted: niles-interp refuses `a read of `bal` (the interpreter has no relational tier)` |
| A11 | SQL-R1 | defective | refused | NL0255 |  |
| A11 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A11 | SQL-R2 | defective | refused | NL0255 |  |
| A11 | SQL-R2 | correct | accepted |  | runs; 202 postings after |
| A11 | NL | defective | not expressible |  | a Money value's minor units cannot be taken out and re-labelled |
| A11 | NL | correct | accepted |  | runs; 2 legs |
| A12 | SQL-R1 | defective | refused | NL0255 |  |
| A12 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A12 | SQL-R2 | defective | refused | NL0255 |  |
| A12 | SQL-R2 | correct | accepted |  | runs; 202 postings after |
| A12 | NL | defective | not expressible |  | a posting's currency is its Money's; there is no separate label |
| A12 | NL | correct | accepted |  | runs; 2 legs |
| A13 | SQL-R1 | defective | refused | NL0230 |  |
| A13 | SQL-R1 | correct | accepted |  | runs; 204 postings after |
| A13 | NL | defective | not expressible |  | no Niles form sets a posting's value date (E30 F9) |
| A13 | NL | correct | not expressible |  | no Niles form sets a posting's value date (E30 F9) |
| A14 | SQL-R1 | defective | accepted |  |  |
| A14 | SQL-R1 | correct | accepted |  | runs; 202 postings after |
| A14 | NL | defective | not expressible |  | no Niles form sets a posting's value date (E30 F9) |
| A14 | NL | correct | not expressible |  | no Niles form sets a posting's value date (E30 F9) |

## The E30 corpus: expressiveness

Each program checked by its surface's checker and run against the oracle. *correct* is the checker's acceptance plus an answer equal to the oracle's; *unexecuted* is accepted with no executor that can run it.

| class | surface | correct | unexecuted | not expressible | tokens (sum) |
|---|---|--:|--:|--:|--:|
| Q — queries | SQL-R1 | 10 | 0 | 0 | 456 |
| Q — queries | SQL-R2 | 10 | 0 | 0 | 335 |
| Q — queries | NL | 8 | 0 | 2 | 329 |
| Q — queries | RS | 7 | 0 | 3 | 380 |
| T — transactions | SQL-R1 | 10 | 0 | 0 | 1406 |
| T — transactions | SQL-R2 | 10 | 0 | 0 | 1286 |
| T — transactions | NL | 6 | 3 | 1 | 895 |
| T — transactions | RS | 6 | 3 | 1 | 895 |
| V — view contracts | SQL-R1 | 5 | 0 | 0 | 352 |
| V — view contracts | SQL-R2 | 5 | 0 | 0 | 245 |
| V — view contracts | NL | 5 | 0 | 0 | 315 |
| V — view contracts | RS | 5 | 0 | 0 | 315 |
| B — temporal reads | SQL-R1 | 5 | 0 | 0 | 392 |
| B — temporal reads | SQL-R2 | 5 | 0 | 0 | 247 |
| B — temporal reads | NL | 4 | 0 | 1 | 182 |
| B — temporal reads | RS | 4 | 0 | 1 | 182 |
| all 30 | SQL-R1 | 30 | 0 | 0 | 2606 |
| all 30 | SQL-R2 | 30 | 0 | 0 | 2113 |
| all 30 | NL | 23 | 3 | 4 | 1721 |
| all 30 | RS | 22 | 3 | 5 | 1772 |

Programs wrong, refused or raising: none. Not expressible, with the reason each program's row gives: Q07 NL, Q07 RS, Q08 RS, Q09 NL, Q09 RS, T04 NL, T04 RS, B03 NL, B03 RS.

## The E30 corpus: mutation

E30's operators M1–M6 and site patterns, unchanged, with R2's one M2 pattern (design §7). Static-rejection rate = static ÷ (static + runtime + silent); equivalent and unexecuted mutants are reported and excluded. *Without annotation sites* removes the mutants whose edit falls inside an effect annotation (SQL's `'effects: …'` string, Niles's `! { … }` row), since those sites exist only because the annotation does.

| class | surface | mutants | static | runtime | silent | equivalent | unexecuted | static-rejection rate | without annotation sites |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|
| Q — queries | SQL-R1 | 33 | 3 | 0 | 30 | 0 | 0 | 9.1% | 9.1% |
| Q — queries | SQL-R2 | 15 | 3 | 0 | 12 | 0 | 0 | 20.0% | 20.0% |
| Q — queries | NL | 14 | 0 | 0 | 11 | 3 | 0 | 0.0% | 0.0% |
| Q — queries | RS | 8 | 0 | 0 | 7 | 1 | 0 | 0.0% | 0.0% |
| T — transactions | SQL-R1 | 125 | 82 | 32 | 7 | 4 | 0 | 67.8% | 61.8% |
| T — transactions | SQL-R2 | 90 | 53 | 32 | 5 | 0 | 0 | 58.9% | 47.9% |
| T — transactions | NL | 59 | 43 | 9 | 0 | 0 | 7 | 82.7% | 73.5% |
| T — transactions | RS | 59 | 43 | 9 | 0 | 0 | 7 | 82.7% | 73.5% |
| V — view contracts | SQL-R1 | 20 | 0 | 0 | 16 | 4 | 0 | 0.0% | 0.0% |
| V — view contracts | SQL-R2 | 6 | 0 | 0 | 2 | 4 | 0 | 0.0% | 0.0% |
| V — view contracts | NL | 6 | 0 | 0 | 2 | 4 | 0 | 0.0% | 0.0% |
| V — view contracts | RS | 6 | 0 | 0 | 2 | 4 | 0 | 0.0% | 0.0% |
| B — temporal reads | SQL-R1 | 38 | 5 | 2 | 27 | 4 | 0 | 14.7% | 14.7% |
| B — temporal reads | SQL-R2 | 18 | 3 | 2 | 11 | 2 | 0 | 18.8% | 18.8% |
| B — temporal reads | NL | 6 | 0 | 0 | 4 | 2 | 0 | 0.0% | 0.0% |
| B — temporal reads | RS | 6 | 0 | 0 | 4 | 2 | 0 | 0.0% | 0.0% |
| all 30 | SQL-R1 | 216 | 90 | 34 | 80 | 12 | 0 | 44.1% | 38.4% |
| all 30 | SQL-R2 | 129 | 59 | 34 | 30 | 6 | 0 | 48.0% | 38.5% |
| all 30 | NL | 85 | 43 | 9 | 17 | 9 | 7 | 62.3% | 49.0% |
| all 30 | RS | 79 | 43 | 9 | 13 | 7 | 7 | 66.2% | 53.2% |

### Candidates from the E30 corpus (design §7)

Cells where NL's or RS's static-rejection rate is strictly higher than SQL-R1's or SQL-R2's, both rates defined. They are candidates only: the adversarial corpus decides the rule.

| operator | class | SQL-R1 | SQL-R2 | NL | RS | Niles's static refusals, NL and RS together |
|---|---|--:|--:|--:|--:|---|
| M1 | T — transactions | 82.5% (97) | 77.3% (66) | 93.3% (30) | 93.3% (30) | NL0250 NL0251 NL0310 ×4, NL0250 NL0310 ×10, NL0310 ×38, NL0310 NL0312 ×2, NL0312 ×2 |
| M2 | T — transactions | 0.0% (19) | 0.0% (19) | 61.1% (18) | 61.1% (18) | NL0257 ×22 |

## Check cost

In-process, release build, by E30's protocol (design §8).

* commit: ada7f0b8ec51
* worktree: clean
* host: Linux 6.18.44-fc-v50 x86_64 — 2 CPUs
* toolchain: rustc 1.95.0 (59807616e 2026-04-14)
* protocol: 20 warm-up passes, then 200 timed passes; each pass checks every file once with each checker, alternating; in-process, release build; peak heap from E18's counting allocator

| corpus | surface | files | lines | µs per KLOC | MAD of a pass, µs | peak heap per KLOC, max bytes |
|---|---|--:|--:|--:|--:|--:|
| E30 corpus | SQL-R1 | 30 | 3791 | 13745 | 1977 | 1248774 |
| E30 corpus | SQL-R2 | 30 | 3606 | 13573 | 1924 | 1209032 |
| E30 corpus | NL | 26 | 1320 | 3729 | 318 | 1247596 |
| E30 corpus | RS | 25 | 1287 | 2552 | 153 | 1288353 |
| adversarial | SQL-R1 | 28 | 3696 | 13504 | 1813 | 1142971 |
| adversarial | SQL-R2 | 4 | 493 | 13348 | 203 | 1134097 |
| adversarial | NL | 14 | 774 | 3965 | 179 | 1276678 |

* E30 corpus: SQL-R1 takes **3.69×** NL's time per KLOC — above 2×, so reported as a cost (the rule: not a veto).
* E30 corpus: SQL-R2 takes **3.64×** NL's time per KLOC — above 2×, so reported as a cost (the rule: not a veto).
* adversarial: SQL-R1 takes **3.41×** NL's time per KLOC — above 2×, so reported as a cost (the rule: not a veto).
* adversarial: SQL-R2 takes **3.37×** NL's time per KLOC — above 2×, so reported as a cost (the rule: not a veto).

## Deviations from the design

* **D1 — a defect in the existing conservation adapter was fixed** (`3ee2e76`). Building the typing rules found that `(m).minor` parses as the field of a one-element row, which `conserve.rs` read as a fresh amount, contrary to its own documentation ("`row(-(m).minor)::usd` and `m` cancel"). The adapter now reads the parenthesised form as `m`. It is not one of the three additions; it was made before any adversarial verdict was read, and it has a guard test that fails against the old reading.
* **D2 — `hold`'s named arguments are not evaluated** by niles-interp (`25e6df7`). The design said they are evaluated and not otherwise interpreted. A duration literal (`expires: 7.days`) is outside the interpreter's subset, so evaluating one would have refused every hold, T05 included; the arguments have no effect under design §6.2 either way.
* **D3 — no program was edited after its mutants were classified,** and no builder fix was needed: every SQL-R1 and SQL-R2 program, and T02, T05 and T06 in NL and RS, equalled the oracle on their first run (`e30b run`, before `e30b mutate`).

## What the numbers contain

* **Niles's static refusals of a flipped leg (M2 on T) are all NL0257.** Turning `debit(a, m)?` into `credit(a, m)?` leaves a `?` on a value that is not a `Result`, which C15-01 made a check-time error; E30 classed the same mutants *runtime* (its finding F12). The refusal comes from the shape of the two functions' return types, not from the conservation solver: the flip in the other direction (`credit` to `debit`, which leaves no `?` behind) is refused at run time by both languages' ledgers. SQL+C+L refuses no M2 mutant statically, because its solver reads a parameter as a symbol that may be zero, so `2m ≠ 0` is undecided and discharged to the conservation trigger, exactly as Niles's solver does.
* **Every run-time currency swap in SQL's transactions is on the annotation's signature line.** Each M1 mutant of a T program classed *runtime*, in both representations, is an edit to the parameter types written in `comment on function task(bigint, bigint, usd)`: the comment then names a function that does not exist, and PostgreSQL raises when the script defines it. The checker keys annotations by function name, so it does not see this; neither would a reviewer reading the effect list, which is unchanged.
* **M6 finds no site inside SQL's annotation strings.** E30's M6 pattern matches a rung that is a whole word or a whole string; `reads@ledger_consistent` inside `'effects: …'` is neither. So M6 mutates Niles's `read@…` rows (three T sites, all refused as NL0310) and nothing in SQL's T programs. This is E30's site pattern, kept unchanged by the design, and it is stated here because it makes the M6 row incomparable.
* **SQL+C+L fails open on a missing annotation.** A01 (a hold domain never declared linear) and A02 (a ledger never declared a ledger) are accepted because the checker's rules are keyed on the annotations the schema omitted; Niles's holds are linear and its conservation applies to every `post`, whatever the schema says. This is the class the verdict names.
* **Niles accepts A05.** A helper posting one half of a transfer, called inside the caller's transaction, leaves the caller's row unbalanced by the symbolic amount `m`, which may be zero, so the solver discharges it to the run-time seal rather than refusing it. Neither checker refuses A05; it counts for neither language.
* **NSQ002 refuses A03's correct twin as well as its defect,** by design: dynamic SQL in a ledger writer cannot be checked, so the rule refuses it whatever it builds. A04's ledger name, assembled by concatenation, escapes the string-literal clause of §5.3, as the design said it would; A04 is outside Niles either way.

