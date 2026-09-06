# Handoff to the executor — cycle 7

**Read `work-order-7-consolidated.md` and nothing else first.** It is the only document the executor
needs; the two audit work orders and the briefs are its provenance.

**State at handoff.** Niles `c6/audit-cycle-7` at the commit carrying this file (engine identical to
`d9c8699`); GBS `e803b7d` on `c6/07-hold-index`. Both trees clean of tracked changes. GBS `make gate`
is **red** against current Niles (F-47) — that is T-00 and it is first. The four pre-existing
untracked files in the author's Mac checkout are his and are never touched.

**Decisions already made.** LC-21 (fail-stop on barrier failure): decided, fail-stop. LC-17: closed.
LC-14, LC-18: closed. Everything else in §8 of the order is carried, not gated.

**What the executor does in order.** T-00 → T-01 → T-01b → T-02 → T-03 → T-04 → T-04a, then the cut
line. Every task's guard is proved failing on the reverted change in a disposable worktree, with the
transcript in the report. The report carries §5 of the order verbatim with `done` / `not done: <why>`
beside every line.

**What needs the author.** After T-01 and T-02 land: `bash ~/Documents/niles-hostc/run4.sh` on the
Mac, and `channel = "1.95.0"` in `rust-toolchain.toml` (needs network once).

**Attribution.** From the executing session's own instructions; record "none required" if there are
none. Never bake in a trailer.
