#!/usr/bin/env bash
# Cycle 8 — Host C run 6: the same-session A/B, E19 under the T-02 header, and the tail breakdown.
#   cp ~/Documents/niles/docs/audit/cycle-8/run6.sh ~/Documents/niles-hostc/run6.sh
#   bash ~/Documents/niles-hostc/run6.sh            # everything, ~25 min on ten cores
#   bash ~/Documents/niles-hostc/run6.sh --only D   # one section (A..E)
#
# Two arms, both DETACHED worktrees at explicit SHAs, retargeted on every run and REFUSED on
# mismatch. This replaces run5.sh's `[ -d wt ] || git worktree add` pattern (work order 8, F-51):
# a worktree that already exists is never trusted to be at the commit its name suggests.
#
#   ARM_A  the code under test — default: HEAD of ~/Documents/niles (whatever c8/* is merged)
#   ARM_B  the baseline        — default: 8d229ea, run 4's arm A, the last C measurement
#
# `--publish` is used here and ONLY here: this is the reference host. The script commits the
# published results/ on a branch c8/08-hostc-run6 inside arm A's worktree; the author pushes it.
set -u
HC="$HOME/Documents/niles-hostc"
NILES="$HOME/Documents/niles"
GBS="$HOME/Documents/GBS"
ARM_A="${ARM_A:-$(git -C "$NILES" rev-parse HEAD)}"
ARM_B="${ARM_B:-8d229ea}"
ONLY="${2:-ALL}"; [ "${1:-}" = "--only" ] || ONLY=ALL
OUT="$HC/results6-$(date +%Y%m%d-%H%M%S)"; mkdir -p "$OUT"
exec > >(tee -a "$OUT/run6.log") 2>&1
export PATH="$HOME/.cargo/bin:$PATH"
unset CARGO_TARGET_DIR
export PGPORT="${PGPORT:-5432}"

want() { case "$ONLY" in ALL|"$1") return 0;; *) return 1;; esac; }

# --- the retarget-and-refuse clause -------------------------------------------------------------
# retarget <worktree> <sha>: create detached if absent, else check out <sha> detached; then verify.
retarget() {
  local wt="$1" sha="$2" full
  full=$(git -C "$NILES" rev-parse --verify "$sha^{commit}" 2>/dev/null) || { echo "REFUSING: $sha is not a commit in $NILES"; exit 3; }
  git -C "$NILES" worktree prune
  if [ -d "$wt" ]; then
    git -C "$wt" checkout -q --detach "$full" || { echo "REFUSING: cannot retarget $wt to $full"; exit 3; }
  else
    git -C "$NILES" worktree add -q --detach "$wt" "$full" || { echo "REFUSING: cannot add $wt"; exit 3; }
  fi
  local have; have=$(git -C "$wt" rev-parse HEAD)
  [ "$have" = "$full" ] || { echo "REFUSING: $wt is at $have, wanted $full"; exit 3; }
  [ -z "$(git -C "$wt" status --porcelain)" ] || { echo "REFUSING: $wt is dirty:"; git -C "$wt" status --short; exit 3; }
  echo "arm at $wt: $(git -C "$wt" rev-parse --short HEAD) $(git -C "$wt" log -1 --format=%s)"
}

echo "run6 — $(date -u +%FT%TZ) — $(uname -a)"
echo "niles working copy: $(git -C "$NILES" rev-parse --short HEAD) $(git -C "$NILES" rev-parse --abbrev-ref HEAD)"
# The four untracked files are the author's; anything else untracked in the main tree is reported.
git -C "$NILES" status --short | grep -vE '^\?\? (\.DS_Store|AGENTS\.md|thesis/\.DS_Store|thesis/Niles-Thesis\.pdf)$' && echo "(unexpected entries above — the run continues; report them)"
retarget "$HC/wt-a" "$ARM_A"
retarget "$HC/wt-b" "$ARM_B"

# --- A. preflight for C --------------------------------------------------------------------------
if want A; then
  echo; echo "### A. preflight"
  sysctl -n machdep.cpu.brand_string hw.ncpu hw.perflevel0.physicalcpu hw.perflevel1.physicalcpu hw.memsize 2>/dev/null
  echo "rustc: $(rustc --version)   cargo: $(cargo --version)"
  echo "pin in arm A: $(grep channel "$HC/wt-a/rust-toolchain.toml")"
  ( cd "$HC/wt-a" && bash docs/audit/cycle-8/preflight.sh 2>/dev/null | sed -n '/barrier/,/VERDICT/p' )
  psql -c "select version()" -tA 2>/dev/null; psql -c "show wal_sync_method" -tA 2>/dev/null
fi

# --- build both arms once ------------------------------------------------------------------------
build() { ( cd "$1" && cargo build -q --release --offline -p bank-bench -p nilestream-server 2>&1 | tail -3 ); }
if want B || want C || want D; then
  echo; echo "### build arm A"; build "$HC/wt-a"
  echo "### build arm B"; build "$HC/wt-b"
fi

# --- B. E16 same-session A/B, --publish on arm A (T-15.3) -------------------------------------------
# Today's harness (docs/BENCHMARK.md "Running the A/B"): the two arms run back to back in one
# session, arm B absolute into --out, arm A naming its baseline. Once T-15 lands, bench takes
# `--baseline-bin <path>` and interleaves the arms itself; the script then passes wt-b's daemon.
# THE AUTHOR MUST RUN THIS after c8/07-ab-script is merged (for --oltp-connections); before that
# the flag is unknown and the run refuses rather than measuring one connection again.
if want B; then
  echo; echo "### B. E16 same-session A/B  (arm B absolute, then arm A against it; --publish on A)"
  ( cd "$HC/wt-b" && ./target/release/bench --run --out "$OUT/e16-arm-b" 2>&1 | tee "$OUT/e16-b.txt" | tail -20 )
  if grep -q -- "--oltp-connections" "$HC/wt-a/crates/bank-bench/src/bin/bench.rs"; then
    ( cd "$HC/wt-a" && ./target/release/bench --run --baseline "$(git -C "$HC/wt-b" rev-parse --short HEAD)" \
        --baseline-bin "$HC/wt-b/target/release/nilestreamd" --oltp-connections 1,4,16 --publish 2>&1 | tee "$OUT/e16-a.txt" | tail -40 )
  else
    echo "REFUSING section B's arm A: T-15 (--oltp-connections / --baseline-bin) is not in arm A yet; arm B's absolute run is kept in $OUT/e16-arm-b"
  fi
fi

# --- C. E19 scaling under the T-02 header, --publish (T-11.2) -------------------------------------
if want C; then
  echo; echo "### C. E19 scaling 1/2/4/8/12/16, 3 runs, --publish"
  ( cd "$HC/wt-a" && ./target/release/bench --run --scaling-only --connections 1,2,4,8,12,16 \
      --mixed-seconds 20 --publish 2>&1 | tee "$OUT/e19.txt" | tail -30 )
  head -1 "$HC/wt-a/results/E19-scaling/point.csv"
fi

# --- D. the tail: three mixed shapes with the per-read breakdown (T-12.3) --------------------------
if want D; then
  echo; echo "### D. mixed 4r2w / 8r1w / 8r4w with the slowest-16 breakdown"
  # `--nls-only`: the mixed workload measures ONE engine and compares nothing to
  # PostgreSQL, so it needs no PostgreSQL — and without this flag the harness refuses,
  # correctly, three times in a row. The first version of this script omitted it and
  # produced an empty section on Host C.
  grep -q -- "report_slow_reads" "$HC/wt-a/crates/bank-bench/src/bin/bench.rs" || { echo "REFUSING section D: T-12 (the slowest-16 breakdown) is not in arm A yet"; ONLY=NONE; }
  [ "$ONLY" != NONE ] && for shape in 4:2 8:1 8:4; do
    r=${shape%:*}; w=${shape#*:}
    ( cd "$HC/wt-a" && ./target/release/bench --run --scaling-only --nls-only \
        --connections $((r+w)) --mixed-seconds 30 2>&1 | tee "$OUT/mixed-${r}r${w}w.txt" \
        | grep -E "^\| (readers|writers|mixed)|slowest|rank \||^ +[0-9]+ \||view_(wait|hold)_(p99|max)|lock_wait_max|fallbacks" )
    # A grep that matches nothing must not look like a section that ran. Show the tail.
    grep -qE "^\| mixed" "$OUT/mixed-${r}r${w}w.txt" || { echo "  (no mixed row — the run did not get that far; last lines:)"; tail -6 "$OUT/mixed-${r}r${w}w.txt" | sed 's/^/  | /'; }
  done
fi

# --- E. the 1.97.1 gate in both trees ---------------------------------------------------------------
if want E; then
  echo; echo "### E. clippy at $(rustc +1.97.1 --version 2>/dev/null || echo 'no 1.97.1 toolchain')"
  for t in "$HC/wt-a" "$GBS"; do
    ( cd "$t" && cargo +1.97.1 clippy --offline --all-targets -- -D warnings 2>&1 | grep -E "^(error|warning)" | sort | uniq -c | sort -rn | head; echo "$t clippy exit: ${PIPESTATUS[0]}" )
  done
fi

# --- commit what --publish wrote, on a branch the author pushes -------------------------------------
if want B || want C; then
  cd "$HC/wt-a"
  if [ -n "$(git status --porcelain -- results/)" ]; then
    git checkout -q -b "c8/08-hostc-run6-$(date +%Y%m%d)" 2>/dev/null || git checkout -q "c8/08-hostc-run6-$(date +%Y%m%d)"
    git add results/ && git commit -q -m "Host C run 6: E16 same-session A/B and E19 under the T-02 header (--publish on the reference host)

Arm A $(git rev-parse --short "$ARM_A"), arm B $(git rev-parse --short "$ARM_B"). Log: $OUT/run6.log"
    echo "committed $(git rev-parse --short HEAD) on $(git rev-parse --abbrev-ref HEAD) — push it:"
    echo "  git -C $HC/wt-a push origin $(git rev-parse --abbrev-ref HEAD)"
  else
    echo "nothing published (no --publish section ran, or results/ unchanged)"
  fi
fi
echo; echo "done — $OUT  (paste the tail of run6.log into the session)"
