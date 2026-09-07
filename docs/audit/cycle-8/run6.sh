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
  if [ -n "$(git -C "$wt" status --porcelain)" ]; then
    echo "REFUSING: $wt is dirty:"
    git -C "$wt" status --short | sed 's/^/  /'
    echo "  A worktree must be at a known commit for its measurement to name one. If these are"
    echo "  a previous run's outputs under results/, they are recoverable:"
    echo "    git -C $wt checkout -- results/ && git -C $wt clean -fd results/"
    exit 3
  fi
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

# --- a daemon for the sections that need one -----------------------------------------------
# `bench` connects to an ALREADY RUNNING nilestreamd on --nls-port; it does not start one.
# Run 4's mixed figures came from a probe crate that spawned its own daemon, and this script
# inherited the invocation without the daemon: every Nilestream level reported "Connection
# refused (os error 61)" and the section looked empty. Started here, killed on exit, with its
# segment in the scratch directory so nothing durable lands in the worktree.
NLSD_PID=""
start_nlsd() {
  local wt="$1" port="${2:-5434}"
  stop_nlsd
  mkdir -p "$OUT/seg"
  "$wt/target/release/nilestreamd" --port "$port" --accounts 10000 --rounds 2 --budget 2500 \
      --durable "$OUT/seg/run6-$port.seg" > "$OUT/nlsd-$port.log" 2>&1 &
  NLSD_PID=$!
  for _ in $(seq 1 60); do
    if nc -z 127.0.0.1 "$port" 2>/dev/null; then
      echo "nilestreamd up on $port (pid $NLSD_PID)"
      return 0
    fi
    sleep 0.5
  done
  echo "REFUSING: nilestreamd did not come up on 127.0.0.1:$port — see $OUT/nlsd-$port.log"
  tail -5 "$OUT/nlsd-$port.log" | sed 's/^/  | /'
  return 1
}
stop_nlsd() {
  [ -n "$NLSD_PID" ] || return 0
  kill "$NLSD_PID" 2>/dev/null
  wait "$NLSD_PID" 2>/dev/null
  NLSD_PID=""
}
trap 'stop_nlsd' EXIT

# --- build both arms once ------------------------------------------------------------------------
build() { ( cd "$1" && cargo build -q --release --offline -p bank-bench -p nilestream-server --bin bench --bin nilestreamd 2>&1 | tail -3 ); }
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
  if ! grep -q -- "--oltp-connections" "$HC/wt-a/crates/bank-bench/src/bin/bench.rs"; then
    echo "REFUSING section B: T-15 (--oltp-connections) is not in arm A yet."
  elif ! pg_isready -h 127.0.0.1 -p 5433 >/dev/null 2>&1; then
    # The contract table IS the comparison, so this section needs the baseline it compares
    # against. It refuses once, here, instead of three times inside the harness.
    echo "REFUSING section B: no PostgreSQL on 127.0.0.1:5433. The contract table is a"
    echo "  comparison and this harness will not substitute anything for a real one. Start it:"
    echo "    initdb -D <pgdata> -U bench --auth=trust"
    echo "    pg_ctl -D <pgdata> -o '-p 5433 -c listen_addresses=127.0.0.1' start"
    echo "    createdb -h 127.0.0.1 -p 5433 -O bench bench"
  else
    if start_nlsd "$HC/wt-b" 5434; then
      ( cd "$HC/wt-b" && ./target/release/bench --run --out "$OUT/e16-arm-b" 2>&1 | tee "$OUT/e16-b.txt" | tail -20 )
    fi
    if start_nlsd "$HC/wt-a" 5434; then
      ( cd "$HC/wt-a" && ./target/release/bench --run --baseline "$(git -C "$HC/wt-b" rev-parse --short HEAD)" \
          --oltp-connections 1,4,16 --publish 2>&1 | tee "$OUT/e16-a.txt" | tail -50 )
    fi
    stop_nlsd
  fi
fi

# --- C. E19 scaling under the T-02 header, --publish (T-11.2) -------------------------------------
if want C; then
  echo; echo "### C. E19 scaling 1/2/4/8/12/16, 3 runs, --publish"
  if ! pg_isready -h 127.0.0.1 -p 5433 >/dev/null 2>&1; then
    echo "REFUSING section C: no PostgreSQL on 127.0.0.1:5433 (see section B for how to start one)."
  elif start_nlsd "$HC/wt-a" 5434; then
    ( cd "$HC/wt-a" && ./target/release/bench --run --scaling-only --connections 1,2,4,8,12,16 \
        --mixed-seconds 20 --publish 2>&1 | tee "$OUT/e19.txt" | tail -30 )
    head -1 "$HC/wt-a/results/E19-scaling/point.csv"
    stop_nlsd
  fi
fi

# --- D. the tail: three mixed shapes with the per-read breakdown (T-12.3) --------------------------
#
# Needs neither PostgreSQL nor `--publish`: the mixed workload measures ONE engine, compares
# nothing, and writes nothing a repository tracks. It does need a running `nilestreamd` —
# `bench` connects to one and never starts one, which is what the first version of this
# section got wrong: every level reported "Connection refused" and the section read as empty.
if want D; then
  echo; echo "### D. mixed 4r2w / 8r1w / 8r4w with the slowest-16 breakdown"
  if ! grep -q -- "report_slow_reads" "$HC/wt-a/crates/bank-bench/src/bin/bench.rs"; then
    echo "REFUSING section D: T-12 (the slowest-16 breakdown) is not in arm A yet."
  elif start_nlsd "$HC/wt-a" 5434; then
    for shape in 4:2 8:1 8:4; do
      r=${shape%:*}; w=${shape#*:}
      # `--out` outside the worktree. A run that does not publish must not write a committed
      # CSV, and until T-15's follow-up a default `--out` did exactly that: it is what left
      # `wt-a` dirty and made the next run of this script refuse, correctly.
      ( cd "$HC/wt-a" && ./target/release/bench --run --scaling-only --nls-only \
          --out "$OUT/e19-D" --connections $((r+w)) --mixed-seconds 30 2>&1 \
          | tee "$OUT/mixed-${r}r${w}w.txt" \
          | grep -E "^\| (readers|writers|mixed)|slowest|rank \||^ +[0-9]+ \||view_(wait|hold)_(p99|max)|lock_wait_max|fallbacks" )
      # A grep that matches nothing must not look like a section that ran. Show the tail.
      grep -qE "^\| mixed" "$OUT/mixed-${r}r${w}w.txt" || { echo "  (no mixed row — the run did not get that far; last lines:)"; tail -6 "$OUT/mixed-${r}r${w}w.txt" | sed 's/^/  | /'; }
    done
    stop_nlsd
  fi
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
