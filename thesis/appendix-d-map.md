*Generated from the workspace by `thesis/gen-appendix-d.py`. Do not edit by hand.*

| Crate | Role | public items |
|---|---|--:|
| `bank-bench` | NilesBank generator, wall-clock harness, thesis drift tests | 60 |
| `conservation-suite` | Reference oracle and the conservation property tests | 23 |
| `experiments` | The E-series measurement harness | 0 |
| `niles-interp` | The imperative-subset interpreter `nilesc run` drives, and the ledger it posts to | 16 |
| `niles-ir` | Typed IR: circuit types, verifier, reference interpreter, upquery paths | 40 |
| `niles-lang` | Stage-0 compiler: lexer, parser, type/effect checker, lowering; SQL surface | 133 |
| `nilesc` | The compiler driver: `check`, `verify`, `run` | 0 |
| `nilestream` | The engine binary: sweep and serve | 0 |
| `nilestream-consensus` | A single-process, deterministic simulator for replication and cross-shard commit. No sockets, no clock | 21 |
| `nilestream-core` | REV runtime: resident maps, anchor indices, apply loop, upqueries, contracts | 19 |
| `nilestream-ledger` | Epoch segments, sequencer, hash chain, durability, admission and commit rules | 32 |
| `nilestream-optimizer` | Plan-time mode selection and the eviction policies (the adaptive optimizer of §4.6 is specified and not built) | 37 |
| `nilestream-server` | Daemon: sessions, PostgreSQL wire surface, conformance | 81 |
| `proto-engine` | The research prototype the counted-work experiments run on | 20 |
