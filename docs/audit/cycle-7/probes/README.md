# Fable's cycle-7 probes — scratch, not repository code

Standalone sources the audit ran **outside** the trees; kept here so the executor can rebuild them
as `[workspace]`-less crates with path dependencies on `bank-bench`, `nilestream-server` and
`proto-engine`, and so a reader of the work order can see exactly what produced each number.

| file | what it measured | finding |
|---|---|---|
| `mixed.rs` | readers alone / writers alone / both, over the wire against the hosted durable engine; lock and read-model counters after | F-27, F-34 |
| `crash.rs` | concurrent writers against an external `nilestreamd --durable`; the shell around it SIGKILLs and restarts | **F-26** |
| `mem.rs` | RSS around `RevEngine::seeded` at 20k / 200k / 2M base rows | F-31 |
| `run4.sh` | Host C: three arms of E19 at 1–16 connections, the mixed probe, RSS, T-03 | F-28, F-30, F-34, F-36 |
| `run5.sh` | Host C: T-03 against the right GBS commit; clippy under 1.97.1 | F-33, F-37 |
| `preflight-host-A.txt` | the container's preflight, verbatim | §0 |

The exact fallback count in F-27 came from two `AtomicU64`s added to `answer_from_view` in a
**throwaway worktree**, never committed; T-02 builds the real counter.
