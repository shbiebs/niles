# E14 — check cost per KLOC, Niles against SQL+C+L

*Written by `cargo run --release -p nilescheck-sql --bin checkcost` (cycle 14, R2-05). Machine-dependent: every figure is a measurement on the host below, and only the ratio between the two checkers on one host means anything.*

| field | value |
|---|---|
| commit | `6215fcd7793a` |
| worktree | clean (crates/, Cargo.toml, Cargo.lock) |
| host | Linux 6.18.44-fc-v37 x86_64 — 2 CPUs |
| toolchain | `rustc 1.95.0 (59807616e 2026-04-14)` |
| checkpoint_interval | n/a (no engine runs: a checker's parse and check) |
| protocol | 20 warm-up passes, then 200 timed passes; each pass checks every file once with each checker, alternating; in-process, release build; peak heap from a counting allocator |

| checker | corpus | files | lines | median pass µs (MAD) | µs per KLOC | peak heap per check, max over files, bytes | peak heap per KLOC, max over files, bytes |
|---|---|--:|--:|--:|--:|--:|--:|
| Niles (`nilesc check` front end) | 13 classes + 5 d14 spellings | 18 | 236 | 1367 (150) | 5793 | 34755 | 2932600 |
| SQL+C+L (`nilescheck-sql`, preamble included per file) | 13 classes + 5 d14 spellings | 18 | 1890 | 7492 (643) | 3964 | 119441 | 1121651 |

**Time per KLOC, SQL+C+L over Niles: 0.68×.** The pre-registered rule reports a checker whose time per KLOC is above 2× the other's as a cost, not a veto; neither is above 2× the other's.

**What a line is here.** Each SQL file is checked as the 98-line preamble followed by the defect, so most of SQL+C+L's lines are the preamble's declarations, re-checked every time; each Niles file carries its own nine-line schema. The two corpora state the same eighteen defects, and the figure is time over the lines each checker actually reads — not over the lines a programmer would write for the defect alone.
