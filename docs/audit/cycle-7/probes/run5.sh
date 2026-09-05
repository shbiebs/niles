#!/usr/bin/env bash
# Cycle 7 — Host C run 5: the T-03 regression, done right. About one minute.
#   bash ~/Documents/niles-hostc/run5.sh
# run4 tested GBS's verdict suite in your WORKING COPY, which is at dm02/checkpoints — before
# T-03 removed the nested `cargo run`. This adds a GBS worktree at c6/07-hold-index (which
# includes T-03) and runs the suite there, with and without CARGO_TARGET_DIR, full output.
set -u
HC="$HOME/Documents/niles-hostc"
GBS="$HOME/Documents/GBS"
OUT="$HC/results5-$(date +%Y%m%d-%H%M%S)"; mkdir -p "$OUT"
exec > >(tee -a "$OUT/run5.log") 2>&1
export PATH="$HOME/.cargo/bin:$PATH"
unset CARGO_TARGET_DIR
cd "$GBS" && git worktree prune
[ -d "$HC/wt-gbs" ] || git worktree add -q "$HC/wt-gbs" c6/07-hold-index
echo "gbs worktree: $(git -C "$HC/wt-gbs" rev-parse --short HEAD) (expect e803b7d)"
echo "working copy : $(git -C "$GBS" rev-parse --short HEAD) $(git -C "$GBS" rev-parse --abbrev-ref HEAD)"
echo
echo "### A. c6/07-hold-index, CARGO_TARGET_DIR UNSET"
( cd "$HC/wt-gbs" && NILES_ROOT="$HC/wt-after" cargo test --release --offline -p gbs-products --test niles_schema 2>&1 | grep -E "^test |test result|BLOCKED|panicked|error\[" )
echo
echo "### B. c6/07-hold-index, CARGO_TARGET_DIR SET (the condition that failed in cycle 5)"
( cd "$HC/wt-gbs" && NILES_ROOT="$HC/wt-after" CARGO_TARGET_DIR="$HC/target-t03-gbs2" cargo test --release --offline -p gbs-products --test niles_schema 2>&1 | grep -E "^test |test result|BLOCKED|panicked|error\[" )
echo
echo "### C. working copy dm02/checkpoints (pre-T-03), CARGO_TARGET_DIR SET — the two run4 failures, in full"
( cd "$GBS" && NILES_ROOT="$HC/wt-after" CARGO_TARGET_DIR="$HC/target-t03-gbs" cargo test --release --offline -p gbs-products --test niles_schema 2>&1 | grep -E "^test |test result|panicked|assert|left:|right:|nilesc|cargo" | head -40 )
echo
echo "### D. gate on the after arm with this Mac's toolchain ($(rustc --version))"
( cd "$HC/wt-after" && cargo clippy --release --offline --all-targets -- -D warnings 2>&1 | grep -E "^(error|warning)" | sort | uniq -c | sort -rn | head -10; echo "clippy exit: ${PIPESTATUS[0]}" )
echo "done — $OUT"
