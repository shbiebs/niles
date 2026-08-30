# Niles / Nilestream

A theory of **reconstructible epoch-anchored views (REVs)** — versioned, partially
materialized derived state over an immutable, epoch-ordered, hash-chained base — and the
instruments that test it:

- **Niles** — a general-purpose, state-aware replacement for SQL (SQL-first syntax, Rust
  fallback, novel-last) that makes double-entry invariants, money with per-currency scale,
  bitemporality and value dates, idempotency windows, per-view consistency,
  materialization mode, lineage, immutability and retention statically checkable.
- **Nilestream** — a from-scratch streaming DBMS pairing a ledger-grade, strictly
  serializable write path with a partial-state read-model runtime over a typed IR that is
  the stable contract between language and engine; MySQL/PostgreSQL wire compatible.

The general core is domain-free: an authoritative base relation with committed writes.
The double-entry ledger, monetary types and bitemporal semantics are a **domain layer**
over a fully general relational core.

**The thesis lives in [`thesis/`](thesis/)** — chapters 00–13, appendices A–K (including J, the adjudication of competing design positions, and K, the prototype and reproduction guide), and IEEE
references, one Markdown file per chapter (also delivered as `thesis/Niles-Thesis.docx`).

## Status — read this first

This is a **theory project with a working instrument**. The loop from Niles source text to a
measured result is closed end to end, and `cargo test --workspace` runs **201 tests**.

```
  Niles source text
       │  niles-lang     lex, parse, resolve, typecheck (currency rows, effects, linearity)
       ▼
  typed IR circuit       niles-ir: keys, anchors, contracts, provenance
       │  niles-ir::verify        the trusted verifier; a bad circuit stops here
       ▼
  REV runtime            nilestream-core: absence lattice, anchored upqueries, eviction
       │  over a durable, hash-chained, epoch-ordered ledger
       ▼
  counted work           base rows read, deltas applied, resident entry-epochs
```

| Component | Status |
|---|---|
| **Keyword registry** — 174 keywords, four axes, single source of truth | **Built**, 6 tests |
| **Normative grammar** — 205 rules, 496 productions, drift-tested against the compiler | **Built**, 7 tests |
| **Generated keyword reference** (`docs/keywords.md`) | **Built**, blessing-tested |
| **Stage-0 compiler** — lexer, parser, resolver, typechecker, lowering | **Built**, 48 + 21 tests |
| **Currency-row solver** — conservation and currency safety, statically | **Built**, 11 tests |
| **Consistency-effect calculus** — rung monotonicity, capabilities, linearity | **Built**, 12 tests |
| **Typed IR + verifier + upquery paths** (`niles-ir`) | **Built**, 39 tests |
| **REV runtime** (`nilestream-core`) — absence lattice, anchored upqueries | **Built**, 21 tests |
| **Durable, concurrent write path** (`nilestream-ledger`) — WAL, recovery, sequencer | **Built**, 21 tests |
| **End-to-end runner** (`nilestream`) — source → IR → engine → numbers | **Built**, measured |
| Reference oracle (Appendix F) — the program that *defines* correctness | **Built**, 17 tests |
| Optimizer cost rules (Appendix I), IR contract types | **Built**, 14 tests |
| Research prototype + experiment harness (E1–E10) | **Built and run**; `results/` |
| Hash chaining | Built with a **placeholder hasher** (ADR 0002); API is drop-in |
| Query planner beyond lowering, wire protocols, server daemon | **Not built** |
| Distributed execution, consensus, cross-shard commit | **Not built** |
| Self-hosted compiler (Appendix E stages 1–3) | **Not built**; Appendix E.0 says so |

**Measurements were taken, and three of them refuted claims the thesis had made.** Chapter 9
§§9.1–9.4 report real results from the prototype in machine-independent counted-work units
over five fixed seeds. §§9.5–9.12 remain pre-registered design for everything the prototype
cannot do (durability, concurrency, distribution, the compiler); no number there is invented.

Reproduce everything:

```sh
cargo test --workspace                      # 201 tests

# the compiler, on the thesis's own worked example
cargo run -p nilesc -- check   examples/demo_bank.niles
cargo run -p nilesc -- effects examples/demo_bank.niles
cargo run -p nilesc -- explain examples/demo_bank.niles
cargo run -p nilesc -- verify  examples/demo_bank.niles

# the closed loop: compile, verify, install, run, measure
cargo build --release -p nilestream
./target/release/nilestream run   examples/demo_bank.niles ledger_balance --checkpoint 16
./target/release/nilestream sweep examples/demo_bank.niles ledger_balance

# durability and group commit
./target/release/durability-bench

# the original hand-written harness
cargo build --release -p experiments
./target/release/experiments all             # ~10 min; artifacts land in results/
```

Headline measured findings:

* **Reconstruction equivalence holds** — ~13,600 reconstructions under continuous eviction,
  zero divergences from an independent ledger fold; conservation exact; rebuild-from-base exact.
* **A real phase boundary exists**, located between memory prices 0.0005 and 0.002 in
  counted-work units, with a strictly **interior** optimal budget.
* **"Partiality pays on skew" is false** — full materialization's footprint shrinks with skew
  faster than partial's, so the memory ratio moves *against* partiality (12:1 → 2.7:1).
* **Anchor indices do not give history-independence** (80–112× constant factor, zero
  asymptotic change); **per-key checkpoints do** — flat at ≈8.5 base rows across a 64×
  history increase, against a predicted C/2+1 = 9. This produced contribution SC7.
* **A consistency rung's cost falls on maintenance, not reads** — 66× between the loosest
  and strictest rung, scaling ≈1/k, with read-side metrics indistinguishable.
* **The cost of durability falls as concurrency rises** — 5.7× at one thread, 4.8× at
  sixteen, as group commit amortises the fsync (transactions per fsync 1.0 → 8.8).
* **The compiler found four defects in the thesis's own worked program**, including a
  rung-monotonicity violation that is exactly the failure Chapter 1 motivates the thesis
  with. See §9.13.4 — it is the strongest evidence here that the checks are load-bearing.

## Layout

| Path | Contents |
|---|---|
| `thesis/` | The thesis (Markdown chapters + docx) |
| `crates/nilestream-ledger` | Append-only, hash-chained, epoch-ordered write path |
| `crates/nilestream-core` | REV runtime: partial state, upqueries, eviction, ladder |
| `crates/nilestream-optimizer` | Adaptive materialization: modes, estimators, eviction |
| `crates/nilestream-lineage` | Provenance: explain, reproduce, impact |
| `crates/nilestream-storage` | Tiering, durability, migration boundary |
| `crates/nilestream-server` | Daemon, native + MySQL/PostgreSQL wire protocols |
| `crates/niles-ir` | Typed IR: circuits, contracts, verifier |
| `crates/niles-lang` | Stage-0 Niles compiler + SQL surface |
| `crates/niles-stdlib` | `std::bank`, `std::temporal`, money, builtins |
| `crates/conservation-suite` | **Runnable reference oracle** + suites |
| `crates/bank-bench` | NilesBank benchmark generator/harness |
| `crates/proto-engine` | **The research prototype measured in Chapter 9** |
| `crates/experiments` | **The experiment harness** (E1–E10) |
| `results/` | **CSV artifacts + console transcripts of every measured number** |
| `examples/` | Niles programs |
| `docs/`, `targets/`, `bench/` | ADRs, target models, scenarios |

## Quick start

```sh
cargo check --workspace   # everything compiles
cargo test  --workspace   # 33 tests pass today
```

The oracle in `crates/conservation-suite/src/oracle.rs` is the ~350-line program that
defines correctness for the whole system: Nilestream is correct iff it is indistinguishable
from this deliberately unoptimized fold (thesis 3.11, 9.3, Appendix F). It covers
conservation per currency, FX atomicity, idempotency, bitemporal answers, holds with
partial capture / void / expiry, the available-balance identity, and tamper detection.

## The seven results

1. **Versioned partial-state algebra + reconstruction theorem** — eviction and
   reconstruction are exact at every anchor; corollary: they can never create or destroy
   money. Discharges by construction the five anomalies partial-state dataflow otherwise
   excludes by protocol.
2. **Eviction–Consistency Frontier Theorem** — where strict-serializable partial
   materialization stops being cheaper than full materialization.
3. **Complexity theory of consistency under partial materialization** — the price of each
   consistency rung is workload-shaped, not history-shaped.
4. **Consistency-effect calculus + Niles soundness** — well-typed programs conserve value,
   cannot mismatch currencies, cannot overdraw without authorization.
5. **Adaptive materialization calculus + optimizer** — per view and key range across
   {absent, demand, full, spilled, tiered}, with competitive guarantees.
6. **Generality result** — relational completeness, fixpoint completeness over the
   epoch-ordered base, and a proved SQL-fragment translation.
7. **Bounded Reconstruction Theorem (SC7)** — reconstruction cost is bounded by the per-key
   checkpoint interval, not by history length. *Produced by an experiment that refuted the
   claim the other contributions rested on.*

## Roadmap

Phased program (thesis Ch. 8) with pre-registered kill criteria: Phase 0 ledger floor and
break-even → Phase 1 formal model and language → Phase 2 single-node vertical slice →
Phase 3 adaptive materialization, optimizer and language evaluation → Phase 4 distributed
execution → Phase 5 hardening.
