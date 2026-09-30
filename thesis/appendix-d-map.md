*Generated from the workspace by `thesis/gen-appendix-d.py`. Do not edit by hand.*

| Crate | Role | public items |
|---|---|--:|
| `bank-bench` | NilesBank generator, wall-clock harness, thesis drift tests | 114 |
| `comparator` | E27: Nilestream against PostgreSQL, pg_ivm, ReadySet, a REV sidecar and TigerBeetle under eviction, across bank sizes; the five-anomalies probe | 92 |
| `conservation-suite` | Reference oracle and the conservation property tests | 23 |
| `experiments` | The E-series measurement harness | 0 |
| `niles-interp` | The imperative-subset interpreter `nilesc run` drives, and the ledger it posts to | 18 |
| `niles-ir` | Typed IR: circuit types, verifier, reference interpreter, upquery paths | 67 |
| `niles-lang` | Stage-0 compiler: lexer, parser, type/effect checker, lowering; SQL surface | 139 |
| `nilesc` | The compiler driver: `check`, `verify`, `run` | 0 |
| `nilescheck-sql` | Hand-written PostgreSQL 16 SQL and PL/pgSQL parser; the catalog checker (E14 PostgreSQL + checker) and SQL+C+L: linearity, conservation through niles-lang's solver, capabilities (E14 columns); since cycle 15 (E30b′) effect annotations (NL0310), body typing (NL0250/NL0255/NL0332) and NSQ002 for dynamic SQL in a ledger writer | 97 |
| `nilestream` | The engine binary: sweep and serve | 0 |
| `nilestream-consensus` | A single-process, deterministic simulator for replication and cross-shard commit. No sockets, no clock | 21 |
| `nilestream-core` | REV runtime: resident maps, anchor indices, apply loop, upqueries, contracts | 28 |
| `nilestream-ledger` | Epoch segments, sequencer, hash chain, durability, admission and commit rules | 32 |
| `nilestream-optimizer` | Plan-time mode selection and the eviction policies (the adaptive optimizer of §4.6 is specified and not built) | 37 |
| `nilestream-server` | Daemon: sessions, PostgreSQL wire surface, conformance | 87 |
| `proto-engine` | The research prototype the counted-work experiments run on | 24 |
| `rev-sidecar` | E27 arms H3 and T: a nilestream-core REV fed by PostgreSQL logical replication (pgoutput) or TigerBeetle CDC, served over the PostgreSQL wire | 12 |
| `syntax-study` | E30: the syntax study — one corpus in five surfaces, an oracle, the executors and the mutation classification (design docs/study/E30-syntax-design.md); and E30b′, the adversarial study of whether Niles needs its own grammar (docs/study/E30b-design.md) | 101 |
