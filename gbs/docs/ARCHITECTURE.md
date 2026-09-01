# GBS — Global Banking System

**A core banking platform built on the Niles/Nilestream thesis results.**

This document is the design. It decides the crate layering, the mechanism set, and — most
importantly — the falsifiable claim the whole build is a test of.

---

## 0. Sources, and what was actually retrievable

Two videos were named as functional reference. Their transcripts could not be retrieved
(YouTube returned HTTP 429 across every attempt; mirrors and transcript sites were blocked
or 404). What *was* established, from YouTube's oembed endpoint and one third-party
write-up:

| | Established | Not established |
|---|---|---|
| `FLS7NP8YMUQ` | *"Anyone can build a bank: creating a new banking backend"*, Vlad Yatsenko (Revolut CTO), Devoxx. Secondhand summary from a blog write-up of the talk | The talk's own words; the blog is a third party's account |
| `VJ-nxtPlGCQ` | *"How does a typical Core banking Solution architecture looks like?"*, The Fintech Techie. A 5–10 minute explainer | **Nothing about its content.** No description, transcript, or chapters |

The feature surface video 1 implies — multi-currency including crypto, FX, card issuing and
authorisation, domestic and cross-border transfers, P2P payments, investments, spending
analytics, real-time fraud scoring, open banking — is consistent with, and a strict subset
of, the scope the user supplied directly. **The user's own scope list is therefore the
requirement**, and the videos are corroborating context rather than a source. Where this
document needed a fact the videos would have supplied, it says so rather than inventing one.

Two architectural points from video 1's write-up *are* load-bearing here, because they
describe the failure this system is designed to avoid:

* The backend "evolved to microservices by adding an event store for persistent messaging"
  — an event log **retrofitted** onto a transactional core, which is precisely the seam
  §1.1 of the thesis is about: two systems of record glued by a pipeline.
* Java was chosen for "transactional behaviour and guaranteed consistency of information."
  The instinct is right and the mechanism is a library convention. GBS's position is that
  the guarantee should be in the type system and the ledger, not in the discipline of 220
  people.

---

## 1. The claim this build tests

The thesis (§6.6, §11.5.1) asserts that **banking is a library over a fully general
relational core** — that no banking product requires a change to the kernel. That is stated
there and not tested, and §11.3 lists its refutation condition explicitly:

> A banking product from §6.24 that cannot be expressed without a kernel change would
> falsify the "banking as a library" claim and, with it, part of the generality thesis.

**GBS is that test.** Concretely:

1. The kernel (`gbs-kernel`) implements *one* thing: balanced posting sets over an
   immutable epoch-ordered ledger, with per-currency conservation.
2. Seven mechanisms (`gbs-mechanisms`) are built on the kernel and add no new notion of
   value movement — each is a *pattern* of posting sets.
3. Every product line (`gbs-products`) is a composition of mechanisms. **No product may
   touch the kernel, the ledger, or the runtime directly.**

Point 3 is enforced by the build, not by review: `tests/layering.rs` reads the manifests
and fails if `gbs-products` acquires a dependency on anything below `gbs-mechanisms`. A
product that needs a kernel change cannot be written without breaking that test, which
makes the falsification condition mechanical.

The coverage matrix in §4 is therefore the deliverable, and each empty cell is a claim
still owed.

---

## 2. Which thesis findings this uses, and how

Ordered by the grading of §11.5 — enabling first, then cumulative. A design that used the
cumulative results and ignored the enabling ones would have no reason to exist.

### Enabling

**E1 — Anchored reconstruction (Thm 4.1).** Every read model is a REV over a frozen prefix.
The consequence for a bank is specific and is the whole reason for the architecture: the
*available balance* and the *ledger balance* are two views of one ledger at one anchor, so
they cannot disagree about which pending facts they include. That disagreement is the APSN
pattern named in FDIC FIL-19-2023 and CFPB Circular 2022-06. GBS does not "keep them in
sync"; there is nothing to sync, because both are functions of the same prefix.

**E2 — Coordination-free cross-shard reads.** A position read spanning currency shards or
entity shards reads a frozen prefix, which cannot change. No distributed read protocol, no
invalidation, and a result that caches indefinitely. This is what makes a global position
across 29 currencies a cheap read rather than a coordinated one.

**E3 — Static conservation under control flow (Contribution 4).** Every product's posting
logic is written in Niles and checked before it runs. The differential corpus (E14) puts
this at 11 of 12 defect classes caught at compile time against PostgreSQL's zero. For a
syndicated loan splitting a repayment eleven ways under four branches, this is the
difference between a proof and a code review.

**E4 — Honest absence.** A missing position is `Hole`, never `0`. The GBS/Noria postmortem
(§1.1.1) recorded a developer writing `Err(_) => 0, // Account not found or zero balance`
— a database error rendered as a zero balance. In GBS that line does not typecheck.

**E5 — 2PC whose coordinator is a ledger group.** A cross-entity or cross-shard settlement
commits without the classical blocking window.

### Cumulative

**C1 — Per-view consistency rungs with monotonicity.** An authorisation decision reads at
`ledger_consistent`; an end-of-day risk roll-up reads at `bounded(5m)`. The compiler
rejects a view served at a strict rung that reads one served below it, so "the risk number
is stale but the limit check used it" is a compile error.

**C2 — Bitemporality as a first-class axis.** Value date versus booking date is not a pair
of columns; it is the two axes of §3.8. A back-valued correction is a new fact at an
earlier valid time, never a mutation.

**C3 — Partial materialisation with per-key checkpoints (SC7).** The read-model explosion
of §1.1 is the bank's actual shape: thousands of derived views, Pareto-skewed access. SC7
bounds reconstruction at *C/2+1*.

**C4 — Capability-gated effects.** Effect rows carry the capability a transition requires,
so an LC amendment without the issuing-bank capability does not compile.

**C5 — Lineage and audit.** `explain`, `reproduce e at #4200`, `impact` — an auditor's
questions as language constructs.

---

## 3. The seven mechanisms

The design bet, stated so it can be wrong: **twenty-odd product lines need seven
mechanisms.** If an eighth is required, this section is where it will show up, and adding
one is a design change to be recorded rather than a routine extension.

### M1 — Balanced posting set
The atom. A `txn` seals a set of entries whose amounts sum to exactly zero **per currency**.
Not "per transaction" — per currency, because a transaction touching three currencies has
three independent obligations and a single scalar sum would hide a mismatch. Everything
below is a pattern of M1; none of them is a new kind of value movement.

*Used by:* everything.

### M2 — Contingent schedule
A bitemporally-dated set of *future* posting sets, declared once, each becoming effective
at a valid-time instant subject to a predicate. The property that makes this worth having
as a mechanism rather than a cron job: **a schedule is generated by a rule, and the rule is
proved balanced by construction**, so no scheduled posting can be unbalanced whenever it
fires and whatever the calendar does.

*Used by:* forwards and swaps (contingent legs), accruals, coupon and amortisation
schedules, repayment waterfalls, funds sweep, cash pooling, zero-balance structures,
fee schedules, dividend and distribution runs.

### M3 — Hold / commitment
An affine reservation against an account that must be resolved **exactly once** — post,
void, or expire. Linearity is checked statically: a hold resolved twice, or never, is a
compile error. An expiry is a resolution, not an absence of one.

*Used by:* card authorisation, loan commitments and drawdown, letter-of-credit issuance,
margin and collateral, settlement obligations, subscription and redemption windows.

### M4 — Fractional participation
A posting split across a participant set by shares that must sum to exactly one, in exact
rational arithmetic, with a **designated residual holder** so that rounding cannot create
or destroy value. The residual is not a rounding account that absorbs error silently; it is
a named participant who receives the remainder, and the sum is still exactly the original.

*Used by:* syndicated lending (participation shares), fund and ETF units, creation and
redemption baskets, supply-chain finance (receivable assignment across
assignor/assignee), trust and philanthropy allocations, waterfall tranches, fee sharing.

### M5 — Capability-gated state machine
A lifecycle whose transitions **are ledger events** — not metadata alongside them — each
requiring a capability held by the actor. There is no state field to drift from the
postings, because the state *is* a fold over them.

*Used by:* letters of credit, trade loans, M&A deal stages, corporate actions, KYC and
onboarding, dispute and chargeback lifecycles, distressed-asset workout stages.

### M6 — Position signal
A REV that is a function of time: balance, exposure, or holding, per currency or per
instrument, at any anchor on either temporal axis. E2 is what makes the cross-shard
version cheap.

*Used by:* FX positions, portfolio and multi-asset positions, risk analytics, limit and
exposure checks, NAV inputs, regulatory roll-ups under BCBS 239.

### M7 — Rate-indexed valuation
An external observation (a rate fixing, a market price) feeds a **deterministic** pricing
UDF whose output is settled as balanced postings. The determinism obligation is not
optional: a valuation that cannot be reproduced at an epoch cannot be audited, and
`reproduce e at #4200` is the test.

*Used by:* caps and floors, OTC derivatives, mark-to-market, NAV strikes, collateral
revaluation, impairment on distressed portfolios, real-estate revaluation.

---

## 4. Product coverage matrix

The falsifiable artifact. Each row is a product line from the scope; each cell is the
mechanisms it needs. **A row that cannot be filled without an eighth mechanism, or without
a kernel change, falsifies the claim of §1.**

| Product line | M1 | M2 | M3 | M4 | M5 | M6 | M7 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| FX and multi-currency | ● | | | | | ● | ● |
| Forwards and swaps | ● | ● | ● | | | ● | ● |
| OTC, caps and floors | ● | ● | ● | | | ● | ● |
| Lending — revolving | ● | ● | ● | | | ● | ● |
| Lending — term | ● | ● | ● | | | ● | ● |
| Lending — syndicated | ● | ● | ● | ● | | ● | ● |
| Letters of credit | ● | ● | ● | | ● | ● | |
| Trade loans | ● | ● | ● | | ● | ● | |
| Supply-chain finance | ● | ● | ● | ● | ● | ● | |
| Funds sweep | ● | ● | | | | ● | |
| Cash pooling | ● | ● | | | | ● | |
| Zero-balance structures | ● | ● | | | | ● | |
| Securities — equities | ● | ● | ● | | ● | ● | ● |
| Securities — fixed income | ● | ● | ● | | ● | ● | ● |
| Cash management | ● | ● | | | | ● | |
| Multi-asset and alternatives | ● | ● | ● | ● | ● | ● | ● |
| ETF platform | ● | ● | ● | ● | ● | ● | ● |
| Risk analytics | | | | | | ● | ● |
| Portfolio management | ● | ● | | ● | | ● | ● |
| Trading and operations | ● | ● | ● | | ● | ● | ● |
| M&A and capital raising | ● | ● | ● | ● | ● | ● | |
| Illiquid and distressed portfolios | ● | ● | | ● | ● | ● | ● |
| Real estate | ● | ● | ● | ● | ● | ● | ● |
| Clearing and prime brokerage | ● | ● | ● | ● | ● | ● | ● |
| Wealth planning | ● | ● | | ● | ● | ● | ● |
| Trust services | ● | ● | | ● | ● | ● | |
| Philanthropy | ● | ● | | ● | ● | ● | |
| Specialised financing | ● | ● | ● | ● | ● | ● | ● |
| Advisor-guided and self-directed | ● | ● | ● | ● | ● | ● | ● |

Two observations worth stating now, before the code either confirms or embarrasses them.

**Interest accrual is M7, and the first draft of this matrix forgot it.** Revolving and term
lending were marked as needing no rate-indexed valuation, which is wrong — an accrual is
exactly a rate applied to a balance over a period, and it is the same `accrue` a swap's
fixed leg uses. The coverage test in `crates/gbs-products/tests/coverage.rs` caught it by
comparing the row against what `lending.rs` actually imports, which is the reason that test
exists.

**Risk analytics is the only row with no M1.** It moves no money — it is read-side only,
M6 and M7. That is a real structural fact and a good sign: the decomposition distinguishes
things that post from things that observe.

**The rows crowd toward the right as products get more complex.** The exotic end (prime
brokerage, specialised financing, real estate) needs all seven, and the simple end needs
two or three. That is the shape a correct decomposition should have. If instead every row
needed all seven, the mechanisms would not be independent and the set would be wrong.

---

## 5. Crate layering

```
gbs-products     the product library — compositions only
      │          MUST NOT depend on anything below gbs-mechanisms
      ▼
gbs-mechanisms   M2..M7, as patterns of M1
      │
      ▼
gbs-kernel       M1: accounts, entries, posting sets, per-currency conservation
      │
      ▼
nilestream-*     ledger, REV runtime, consensus  ·  niles-lang, niles-ir
```

`tests/layering.rs` reads the Cargo manifests and asserts this. The test is the whole
point: without it, "banking as a library" is a claim about taste.

`gbs-api` sits beside `gbs-products` as the service surface, and `gbs/niles/` holds the
schema and view definitions that `nilesc` checks.

---

## 6. What is deliberately not in the kernel

Recorded here because the temptation to add each one is real, and each would quietly turn
the general core into a banking core:

* **No account "type" with behaviour.** An account is an identity and a currency. Whether
  it is an asset or a liability is a property of the chart of accounts, which is data.
* **No balance field.** A balance is a fold over entries at an anchor. A stored balance is
  a second source of truth, and the whole thesis is about not having two.
* **No status column anywhere.** Status is M5's fold. A `status` field is a value that can
  disagree with the events that produced it.
* **No rounding account in the kernel.** Rounding is M4's designated residual holder, which
  is a named participant rather than a place error goes to die.
* **No currency conversion in the kernel.** FX is two conserved legs sealed in one epoch
  (`fx { leg a: …, leg b: …, rate: r }`), never an amount multiplied by a rate in a
  posting. A rate applied inside a posting is how a cross-currency imbalance becomes
  invisible.

---

## 7. Status

`cargo test --workspace` runs **624 tests**, of which 205 are GBS's.

| Layer | Status |
|---|---|
| `gbs-kernel` | **Built**, 32 tests — posting sets, per-currency conservation, the chart, bitemporal stamps |
| M1 balanced posting set | **Built** — `Sealed` is unforgeable; only `PostingSet::seal` produces one |
| M2 contingent schedule | **Built** — generators checked at declaration; roll conventions; observations passed in |
| M3 hold / commitment | **Built** — resolved exactly once; expiry is a resolution; liveness never reads a clock |
| M4 fractional participation | **Built** — exact rationals in lowest terms, largest-remainder with quota, designated residual holder |
| M5 capability-gated lifecycle | **Built** — no status field; `state_at(epoch)`; undeclared-sink detection |
| M6 position signal | **Built** — `Present`/`Absent`/`Unavailable`; available and ledger balance from one anchor |
| M7 rate-indexed valuation | **Built** — five rounding modes, exact integer arithmetic, recorded inputs, `reproduces_under` |
| FX and multi-currency | **Built** — two conserved legs, one epoch |
| Lending: revolving, term, syndicated | **Built** — 13 tests including a thousand draw/repay cycles with no drift |
| Trade finance: LC, SCF | **Built** — 15 tests including state-at-presentation and three-party assignment |
| Derivatives: forwards, swaps, caps, floors | **Built** — 14 tests including collar parity |
| Liquidity: sweep, pooling, ZBA | **Built** — 11 tests including conservation under arbitrary balances |
| `tests/layering.rs` | **Built**, 6 tests — the falsification check, with a negative control |
| `tests/coverage.rs` | **Built**, 9 tests — the matrix checked against the code, with a negative control |
| The other 18 product lines | **Not built.** Their matrix rows are predictions |
| Niles schema and views | **Not built** |
| `gbs-api` | **Not built** |
| Matching engine (tier 1) | **Not built.** See `PLAN.md` — a different latency regime, deliberately separated |

**11 of 29 product lines implemented.** `the_honest_ratio_of_built_to_claimed_is_reported`
prints the figure from the build, so it cannot drift from this table.

### Four defects the tests found, recorded because they are the evidence this works

1. **`Share` overflowed on an ordinary syndicate.** Shares were kept unreduced and summed by
   `a/b + c/d = (ad+cb)/bd`, on the reasoning that contract denominators stay small. Eleven
   lenders at 10 000ths reaches 10⁴⁴ and overflows `i128`. Found by the *lending product*,
   not by M4's own tests — a mechanism's tests use the sizes its author imagined, and a
   product uses the sizes the business has. Fixed by reducing to lowest terms and summing
   over the LCM.
2. **`HalfUp` rounded the wrong way on negatives.** −7/2 gave −3 instead of −4 — exactly the
   asymmetry that makes a refund round differently from the payment it reverses.
3. **The LC rule set forbade partial drawings.** UCP 600 permits them; `present` needed to be
   a self-loop. Found by a product exercising a mechanism against a real domain rule.
4. **This matrix forgot that interest accrual is M7.** Revolving and term lending were marked
   as needing no rate-indexed valuation. Found by `coverage.rs` comparing the row against
   what `lending.rs` imports, which is why that test exists.

Two of the four were found by the layer *above* the defect, which is the argument for the
layering being real rather than decorative.
