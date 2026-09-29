# E30 — the syntax study: design, pre-registered

*Cycle 14, round 2, card R2-06 (decision D-3). This document is committed on its own, before
any program of the study is written, and is the pre-registration: the corpus, the surfaces, the
metrics, the mutation operators, the classification of a mutant and the decision rule are fixed
here. A change to any of them after this commit is a deviation, listed as one in
`results/E30-syntax.md` with its reason, and never made after seeing a number it would move.*

## 1. The question and the rule

The author asked (D-3) which syntax should be principal — SQL, Rust, another language, or all
new — judged on **expressiveness, safety and speed**. The rule, written in the round-2 work
order §7 before round 2 measured anything:

> The principal surface is the one that (i) expresses the most corpus tasks, then (ii) has the
> highest static-rejection rate under the mutation set, then (iii) the lowest check cost; ties
> are reported as ties. A surface that cannot express a task class is not principal for that
> class; new syntax is justified only for classes no surface expresses.

It is applied **per task class** (§3) as well as over the whole corpus, because the second
sentence makes the class the unit at which a surface can be excluded.

**Context the study inherits.** R2-05 closed D-2 as **(d)**: every defect class and every d14
spelling the Niles checker refuses, SQL with annotations and a checker (SQL+C+L) also refuses
(`results/E14-minimal-counterproposal.md` Part 2b, claim H-L1). That verdict was about what a
checker can refuse; this study asks which *surface* to write in, and does not assume its answer.

## 2. The surfaces

| id | surface | tasks | checker | executor |
|---|---|---|---|---|
| **SQL** | PostgreSQL 16 SQL and PL/pgSQL with R2-05's annotations (SQL+C+L): currencies as composite types, a ledger and its linear and capability domains by `comment on`, contracts as rows of `serve_contract` | all 30 | `nilescheck_sql::check_all` | PostgreSQL 16 (the gate's cluster), each program in `begin … rollback` |
| **NL** | Niles as it is, with its SQL-shaped query surface (`select … from …`, Appendix B.17) | all 30 | `nilesc check` (parse, resolve, typecheck, lower) | queries: `niles_ir::eval` over the lowered circuit; transactions: `nilesc run` (the posting set) |
| **RS** | Niles's pipeline surface, `postings.group_by(\|p\| p.acct).sum(\|p\| p.amt)` — the Rust-shaped representative until L-4 (a Rust DSL) exists | all 30 | as NL | as NL |
| **PRQL** | PRQL 0.13.14 (`prqlc`, Apache-2.0; approved 2026-09-29) | the 10 query tasks | `prqlc compile --target sql.postgres` | the SQL it emits, on PostgreSQL |
| **DL** | Soufflé 2.5 Datalog (UPL-1.0; approved 2026-09-29), interpreter mode | the 10 query tasks | Soufflé's own front end: a run over empty fact files, which reports parse and semantic errors before evaluating anything | `souffle -F <facts> -D -` |
| **NEW** | new syntax | only a task no surface above expresses | — | — |

**NL and RS share the non-query tasks.** Niles has one spelling for a transaction, a serve
contract and a temporal read: the imperative sublanguage and `serve { … }` are Rust-shaped in
both. For the 20 non-query tasks the NL and RS programs are **the same file**, the study says so
in every table, and the pair can only tie there. They differ on the 10 query tasks, which is
where the SQL-shaped and pipeline surfaces of one language diverge.

**PRQL and DL on the query tasks only**, as the work order says. A temporal read or a
transaction written in them would be a query over columns this study's schema happens to have,
not the language's own form of the construct, so their absence from those classes is recorded as
*not in scope*, not as *not expressible*.

## 3. The corpus — 30 tasks in four classes

Every task runs against one fixed dataset (§4) and has one expected answer computed by the
**oracle** (§5), which is written in Rust independently of every surface. A program is correct
when its answer equals the oracle's.

### Q — queries (10), from the SQL fragment of §4.7 / Appendix H and beyond it

| id | task | constructs it needs |
|---|---|---|
| Q01 | balance per (account, currency) | group, sum |
| Q02 | accounts whose usd balance is negative | group, having |
| Q03 | total per (desk, currency), the desk from `accounts` | join, group |
| Q04 | accounts with no postings at all | anti-join |
| Q05 | the five largest usd balances, ties broken by account id | order descending, limit |
| Q06 | number of distinct currencies each account has posted in | distinct count |
| Q07 | running balance per account in epoch order | window (running sum) |
| Q08 | rank of each account's usd balance within its desk | window (rank) |
| Q09 | every ancestor party of each party (`parties.parent`) | recursion |
| Q10 | per account and currency, posted balance plus open holds | union all, group |

### T — banking transactions (10), each correct (it conserves, and consumes what it makes)

| id | task |
|---|---|
| T01 | transfer a parameter amount between two accounts |
| T02 | transfer with a fee: the payee receives the amount less the fee, a fee account the fee |
| T03 | FX settlement: a usd pair and an eur pair, atomic |
| T04 | place a hold and capture part of it |
| T05 | place a hold and void it |
| T06 | syndicated drawdown: the borrower debited, three lenders credited by fixed shares that sum to the total |
| T07 | daily interest accrual: receivable debited, income credited |
| T08 | a repayment capped at the outstanding balance, the excess to an unapplied account |
| T09 | an overdraft grant, which requires the authority to grant it |
| T10 | reversal of a prior transaction by compensating entries (no update) |

### V — view contracts (5)

| id | view and contract |
|---|---|
| V01 | ledger balance — `read_your_writes`, materialised on demand, evictable |
| V02 | available balance (ledger balance minus open holds) — `ledger_consistent`, reading only inputs served at that rung |
| V03 | month-to-date statement — bounded staleness, on demand, reconstructible through an anchor index |
| V04 | posting velocity per account — materialised in full, pinned |
| V05 | trial balance per currency — `snapshot`, materialised in full |

### B — temporal and bitemporal reads (5)

| id | task |
|---|---|
| B01 | balance per account as of epoch *e* (system time) |
| B02 | balance per account valid at value date *d* (world time) |
| B03 | a back-valued correction: post a leg with a value date before *d*; B02 at *d* changes, B01 at an epoch before the correction does not |
| B04 | balance valid at *d* as known at epoch *e* (both axes) |
| B05 | one account's statement for a value-date range, as of epoch *e* |

## 4. The dataset

Generated deterministically by `crates/syntax-study` from seed 1 and committed as fixture files
beside the corpus: 20 accounts in 4 desks under 12 parties (a parent chain three deep, one
party with no parent); currencies usd, eur (scale 2) and jpy (scale 0); 200 postings in balanced
transactions over 10 epochs with value dates spread over 40 days, 3% back-valued; 6 holds, 3
open. Two accounts have no postings (Q04), two usd balances are negative (Q02), and two usd
balances tie (Q05). Each surface loads the same rows — PostgreSQL by `copy`, Soufflé by fact
files, Niles by its fixture — and the loaded row count and SHA-256 of the canonical rows are
asserted equal per surface.

## 5. The oracle and semantic equality

The oracle is a Rust function per task over the fixture, sharing no code with any checker or
executor. Answers are canonical: a query's answer is a multiset of rows sorted by every column
(order is part of the answer only where the task says so — Q05, Q07); a transaction's is the
posting set it appends (`txn`-relative, sorted); a contract's is (accepted by the checker, the
view's rows at the head).

**A program that disagrees with the oracle is a builder error**, fixed before the mutation stage;
the number of such fixes per surface is reported and no fix is made after mutation begins.

## 6. The metrics

### Expressiveness

* **Expressible**: yes, or no with the reason (the construct missing, and the diagnostic when a
  checker refuses a correct program).
* **Tokens**: the task's own text only (the per-surface schema or preamble is counted once and
  reported separately), tokenised by one surface-neutral tokenizer: identifiers and keywords,
  numbers, strings, and each operator or punctuation mark are one token each; comments and
  whitespace are none. SQL+C+L's annotations are `comment on … is '…'` *statements* and count.
* **Distinct constructs**: the distinct tokens of a program that appear in its surface's own
  keyword list (shipped with the study as data, one list per surface, from each language's
  documentation) plus distinct operator tokens.

### Safety — mutation

Six operators, applied mechanically at **every** site where each applies in every expressible
program; the site patterns are per surface and are data in the study, fixed with this design.

| op | mutation |
|---|---|
| M1 | swap a currency: one occurrence of usd ↔ eur (jpy ↔ usd where it is the only one) |
| M2 | swap debit and credit: negate one leg's amount, or swap a debit form for a credit form |
| M3 | drop a guard or clause: delete one conjunct of a filter, `having`, join condition or `if` guard |
| M4 | shift an epoch by one: an anchor or `as of` literal *e* → *e* + 1 |
| M5 | remove a consumption or the conservation statement: delete one `resolve`/consumer call, or the `conserve` clause, trigger or annotation |
| M6 | weaken a contract: a consistency rung one step lower on the ladder |

Each mutant is classified, in this order:

1. **static** — the surface's checker refuses it;
2. **runtime** — it is accepted and raises when executed;
3. **silent** — it runs and its answer differs from the unmutated program's;
4. **equivalent** — it runs and its answer equals the unmutated program's.

**Static-rejection rate** = static ÷ (static + runtime + silent). Equivalent mutants are
reported and excluded, because no checker should refuse a program that means the same thing.

### Speed

* **Check cost**: parse and check time and peak heap per KLOC. In-process for SQL
  (`nilescheck-sql`) and NL/RS (`niles-lang`), by the memory probe's allocator as in
  `results/E14-checkcost.md`. PRQL and DL are external binaries: wall time per invocation over
  the corpus minus the same binary's time on an empty program, **reported separately and never
  ranked against the in-process figures**, and no heap figure.
* **Run-time speed is not measured.** NL and RS lower to the same IR (the SQL surface's golden
  equality, `crates/niles-lang/tests/sql_golden.rs`), so their run time is identical by
  construction; PRQL runs as the SQL it emits. The study says so rather than time noise.

## 7. Not measured, and said so

Readability and learnability (no user study). The programs are written by the builder of two of
the checkers — mitigated by this pre-registration, the mechanical mutation, and the oracle, not
removed. Programs are written for correctness, not brevity, and no program is shortened after
its token count is known.

## 8. Output

`results/E30-syntax.md` — per surface and per class: expressible count with reasons, tokens,
constructs, mutants by class and the static-rejection rate, check cost; the rule evaluated per
class and overall, with this document's commit cited. `results/E30-syntax/` holds the per-program
and per-mutant rows. The syntax rule in §6.25, B.1 and J.1 is restated from the result in R2-11
or round 3, never before.
