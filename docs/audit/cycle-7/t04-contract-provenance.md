# T-04 — the contract is a same-session A/B, and C is the reference — closes F-29

## The finding was not a wrong number

E16's `report` row read **2.68× MET** on one instance of host class A and **2.17× NOT MET** on
another. That looks like a regression. It is not one:

| row | committed (cycle 5, an A instance) | before T-05 (`05eeef7`) | after T-05 (`15425b5`) | after T-06a |
|---|--:|--:|--:|--:|
| oltp | 1.07× NOT MET | 1.02× | 1.15× | 1.10× |
| analytical | 1.93× NOT MET | 2.04× | 2.13× | 2.04× |
| **report** | **2.68× MET** | **2.30× NOT MET** | 2.12× | **2.17× NOT MET** |

The three engine versions measured *on the second instance* are within 8% of each other, MADs ≤4%.
PostgreSQL itself runs 35% slower there — oltp 4,519 → 2,706 ops/s, report 149 → 98 — so both arms
of the ratio moved, and the ratio moved with the machine rather than with the code. **A 2.6×
threshold sitting inside the cross-instance variance of one host class is not a contract; it is a
coin.** Nothing in the published document let a reader see which coin had been tossed.

## What was built

**Every generated E16 document carries a header naming its provenance**: host, instance, session,
barrier rate, and the commit that produced the binary. Each field answers a question a reader needs
before comparing two tables:

* **instance**, not host class — the two A instances answer to the same hostname, so the header
  takes the boot id. This is the field F-29 turns on.
* **session** — both arms of every ratio come from one invocation, and the header says so.
* **barrier** — the durable row is an fsync rate, and the same probe has measured 255/s to 5,825/s
  across this project's hosts.
* **engine commit**, stamped at **build** time and not read at run time. `git rev-parse` in the
  running process reports whatever the tree is checked out at when the binary is invoked, so a
  benchmark built at one commit and run after a checkout would label itself with code it does not
  contain — the same class of error as the stale worktree that made a whole `run4.sh` re-run
  measure the wrong commit. A dirty tree is flagged, because then the hash identifies nothing.
* **baseline commit**, from `--baseline <commit>`. A run without one says in its header that it is
  an absolute measurement and not an A/B, which is exactly what E16 was quoted as for two cycles.

`--baseline` records, and does not execute. Measuring the other arm means checking that commit out
and running again in the same session on the same instance — a script's job. A flag that silently
rebuilt the tree would be worse than no flag.

**Host C is named the reference** in `SPEC-ENGINE.md` Part 0, `docs/BENCHMARK.md` and thesis §9.14.1:
Apple M4, 10 cores, APFS on NVMe, `F_FULLFSYNC` at ~255 barriers/s, script `run4.sh`. Not because it
is fast — it is the slowest storage here — but because it is the only machine in this project that
gives the same answer twice.

## Guards

* `the_contract_header_names_its_host_session_and_commit` — the block names all five fields and
  carries the build-time commit. Reverted by deleting the commit row: fails.
* `the_e16_document_renders_the_provenance_block_under_its_title` — source-level, because the
  builder lives in the binary. It is exactly as strong here: there is no behaviour to observe if
  the call is absent, which is the failure being guarded. Reverted by deleting the call: fails.
* `a_run_with_no_baseline_says_it_is_not_an_ab` — the negative case, which matters more than the
  positive one.
* `no_verdict_without_a_baseline_arm` — `judge` returns `NOT RUN` rather than a verdict computed
  against a baseline that is not there. This is the structural half of the same-session rule: both
  arms of every ratio are drawn from one invocation's samples, so a verdict cannot be formed across
  sessions even by mistake.

## `MISMATCH-contract-instance` — resolved

Thesis 09:686–702 printed absolute E16 figures from an unnamed instance as though they were the
contract. The generated table is unchanged — it is what the harness measured — but the prose around
it now states the rule, records the three-arm demonstration above, and says what survives across
instances: the ordering of the rows and the sign of each gap. Not the second decimal place, and not
a verdict that turns on it.

## What this gives up

The ability to quote "2.68× MET" without a host and a session beside it, which was never
legitimate. Every contract figure in this project is now a claim about one machine at one moment,
and says so.
