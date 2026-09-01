# GBS — Global Banking System

A core banking platform built on the Niles/Nilestream thesis results.

**Read first:** [`docs/PLAN.md`](docs/PLAN.md) answers the two questions this project
started from — does the language exist, does the engine exist — and sets the roadmap.
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) is the design and the falsifiable claim.

## The claim

The thesis asserts that banking is a *library over a fully general relational core*: no
banking product requires a change to the kernel. GBS is the test of that claim, and the
test is mechanical rather than editorial.

```
gbs-products     compositions only — MUST NOT depend on anything below gbs-mechanisms
      ▼
gbs-mechanisms   M2..M7, as patterns of M1
      ▼
gbs-kernel       M1: balanced posting sets, per-currency conservation, the chart
      ▼
nilestream-*     ledger, REV runtime, consensus  ·  niles-lang, niles-ir
```

`crates/gbs-products/tests/layering.rs` reads the Cargo manifests and fails the build if a
product reaches past the mechanisms layer. A product that needs a kernel change **cannot be
written** without breaking a named test whose failure message says which claim it falsifies.

## The seven mechanisms

Twenty-nine product lines, seven mechanisms. If an eighth is needed, that is a design change
to record — `docs/ARCHITECTURE.md` §4 has the coverage matrix, and
`crates/gbs-products/tests/coverage.rs` checks it against the code.

| | Mechanism | The property that makes it worth having |
|---|---|---|
| M1 | Balanced posting set | Conservation is checked **per currency**; a `Sealed` set cannot be forged |
| M2 | Contingent schedule | The **generator** is checked at declaration, so every occurrence it will ever produce is balanced |
| M3 | Hold / commitment | Resolved **exactly once**; an expiry is a resolution; liveness never reads a clock |
| M4 | Fractional participation | Exact rationals, largest-remainder with quota, a **designated residual holder** — no rounding account |
| M5 | Capability-gated lifecycle | No status field. `state_at(epoch)` answers what a dispute asks |
| M6 | Position signal | `Present` / `Absent` / `Unavailable` — `Err(_) => 0` does not typecheck |
| M7 | Rate-indexed valuation | Deterministic, integer-exact, records its inputs so a valuation reproduces |

## Two halves, two kinds of guarantee

`crates/` is Rust: the kernel and mechanisms enforce conservation, linearity and honest
absence **at run time, on values**.

`niles/gbs.niles` is the schema: the same guarantees declared where the compiler checks them
**before the program runs**. It compiles to 6 relations, 7 views, 6 functions, with **5
conservation obligations proved statically and 0 discharged to the runtime**.

## Build

```sh
cargo test --workspace          # 635 tests; 216 are GBS's
cargo run -p nilesc -- check gbs/niles/gbs.niles
cargo run -p nilesc -- verify gbs/niles/gbs.niles
```

## What is not built

18 of 29 product lines; `gbs-api`; and the matching engine, which is deliberately a
**separate tier** — an exchange matching engine's latency budget is four orders of magnitude
below a durable ledger's, and one system claiming both would be lying about one of them.
`docs/PLAN.md` has the three-tier separation and the reason the epoch model is the right
interface between them.
