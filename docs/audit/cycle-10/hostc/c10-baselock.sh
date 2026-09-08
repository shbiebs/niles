#!/usr/bin/env bash
# c10-baselock.sh — the base-lock measurement, and the two-arm harness every cycle-10 pass
# line is scored against.
#
# WHAT THIS MEASURES
#   The mixed workload at 6r/3w, 9r/5w and 12r/6w on one or two builds, at TWO working
#   points: the historical one (2,000 accounts, 2,500 budget — a budget above the distinct
#   key count, so the view is effectively full) and a genuinely partial one (the same
#   accounts at a budget well below them, so eviction and reconstruction actually happen).
#   Cycle 9 measured only the first and wrote about partial materialisation.
#
#   Per level and per point: the throughput row, the slowest-16 keyed reads broken into all
#   six phases of `answer_from_view` with the outcome that says which of them a row even ran,
#   the flight counters including the ones that are expected to be zero, the anchor gap at
#   begin and at finish, and the full `nilestream_lockstats` table with the base lock's
#   shared and exclusive modes reported separately.
#
#   Run it FIRST with `--baseline-only` on the current build: that transcript is the baseline
#   every C10 pass line is written against. Cycle 9 wrote a pass line against a document and
#   the document was wrong by 7.8x; this script exists so that cannot happen again.
#
# THE NEUTRAL CONTROL
#   `--baseline-only` also runs an A=A arm: the same SHA built twice, in two worktrees,
#   interleaved exactly as a real two-arm run would be. Whatever difference that produces is
#   the harness's own noise floor, and no later result smaller than it means anything. It is
#   the falsification control for the harness itself, and it runs BEFORE any performance task
#   is scored. `--no-neutral` skips it and says so in the status section.
#
# WHAT IT REFUSES TO DO
#   * It never writes a tracked artefact. `--publish` is refused, not ignored.
#   * It never installs anything and never reaches the network.
#   * It refuses a dirty repository, a worktree left at the wrong commit, an occupied port,
#     a warm-up that did not complete, a level with no writer progress, and a bench log
#     missing a column this script reports. Each refusal names what it saw.
#   * It prints a status for every section. A section that did not run is named, and the exit
#     code is non-zero.
#
# USAGE
#   bash c10-baselock.sh --baseline-only                    current build + A=A (est. 25-40 min)
#   bash c10-baselock.sh --self-test                        the fault injections only (< 1 min)
#   bash c10-baselock.sh --candidate c10/04-deferred-merge  two arms, interleaved
#   bash c10-baselock.sh --baseline <ref> --candidate <ref> --out ~/c10-out
#
#   Both refs are BRANCH NAMES or SHAs that exist in the repo; the resolved SHA of each is
#   printed in the preflight and is what the transcript is about.
#
# The author's zsh has `interactive_comments` off. Run this with `bash`, as written above.

set -uo pipefail

REPO="${C10_REPO:-$HOME/Documents/niles}"
OUT="${C10_OUT:-$HOME/c10-baselock-out}"

# **No default baseline ref.** It used to default to `c7/01-durable-rows`, a cycle-7 branch,
# so a run with no flags measured a build three cycles old and labelled it "baseline". The
# baseline is the build under test, which is whatever the repository has checked out; that is
# resolved in the preflight and printed, and an explicit `--baseline <ref>` overrides it.
BASELINE_REF=""
CANDIDATE_REF=""
BASELINE_LABEL="baseline"
CANDIDATE_LABEL="candidate"

WARMUPS=2
MEASURED=5
SECONDS_PER_LEVEL=30
LEVELS="6,9,12"
ACCOUNTS=2000
ROUNDS=8
BUDGET_FULL=2500
BUDGET_PARTIAL=400
BASELINE_ONLY=0
NEUTRAL=1
SELF_TEST=0
PORT_BASE=6543
REPLICATE_TIMEOUT=900

FAILED=0
SKIPPED=""

note()  { printf '%s\n' "$*"; }
head2() { printf '\n=== %s ===\n' "$*"; }
skip()  { SKIPPED="${SKIPPED}
  - $*"; FAILED=1; note "NOT RUN: $*"; }

# ---------------------------------------------------------------------------
# Checks, written as functions so `--self-test` can inject a fault into each one
# and require the refusal. A check that is only ever exercised by the happy path
# is a check nobody has seen work.
# ---------------------------------------------------------------------------

# A repository with uncommitted changes measures something that is in no commit, so the SHA
# in the transcript names a build that does not exist.
check_clean_repo() {
  repo="$1"
  dirty="$(git -C "$repo" status --porcelain 2>/dev/null)"
  if [ -n "$dirty" ]; then
    note "REFUSED: $repo has uncommitted changes, so the SHA below would name a build that"
    note "         is not the one measured:"
    printf '%s\n' "$dirty" | head -20 | sed 's/^/           /'
    return 1
  fi
  return 0
}

# A worktree left over from an earlier run sits at whatever commit that run built. Reusing it
# measures the old build under the new build's name, which is the single most expensive way
# for this harness to lie.
check_worktree_at() {
  wt="$1"; want="$2"
  [ -d "$wt" ] || return 0
  have="$(git -C "$wt" rev-parse HEAD 2>/dev/null)"
  if [ "$have" != "$want" ]; then
    note "REFUSED: the worktree $wt is at ${have:-<unreadable>} and this run is about $want."
    note "         It is a leftover from an earlier run. Remove it and re-run:"
    note "           git -C $REPO worktree remove --force $wt"
    return 1
  fi
  return 0
}

# A port with something already on it is a replicate that either fails to bind or, worse,
# measures whatever is listening there.
check_port_free() {
  port="$1"
  if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
    exec 3>&- 2>/dev/null
    note "REFUSED: something is already listening on 127.0.0.1:$port."
    return 1
  fi
  return 0
}

# Every column this script reports must exist in the log it reports it from. The benchmark
# renders a name the server does not have as `n/a` and warns; a harness that reads past that
# warning publishes a table of `n/a` as if it were a measurement.
check_no_missing_fields() {
  log="$1"
  if grep -q "WARNING: the server's slow-read table has no" "$log" 2>/dev/null; then
    note "REFUSED: the server in this run does not report a column this transcript is about:"
    grep "WARNING: the server's slow-read table has no" "$log" | head -4 | sed 's/^/           /'
    return 1
  fi
  if grep -q "flights at .*: n/a" "$log" 2>/dev/null; then
    note "REFUSED: the flight counters came back n/a, so this build does not report them and"
    note "         every flight number in this transcript would be absent rather than zero."
    return 1
  fi
  return 0
}

# A mixed level with no writer progress is a read-only level wearing a mixed level's name,
# and the anchor-mismatch fallback this whole harness exists to observe cannot occur in it.
check_writer_progress() {
  log="$1"
  # **A zero, not a zero digit.** This matched the substring `0 writes/s`, which is inside
  # `900 writes/s`, so a healthy level was refused as having made no writer progress and the
  # self-test caught it on its own clean control. The number must be a whole number zero.
  bad="$(grep "E19 mixed" "$log" 2>/dev/null | grep -E "(^|[^0-9.])0 writes/s" || true)"
  if [ -n "$bad" ]; then
    note "REFUSED: a mixed level made no writer progress, so its reads never saw a moving"
    note "         frontier and its fallback rate is about nothing:"
    printf '%s\n' "$bad" | sed 's/^/           /'
    return 1
  fi
  if ! grep -q "E19 mixed" "$log" 2>/dev/null; then
    note "REFUSED: the run produced no mixed level at all."
    return 1
  fi
  return 0
}

# Whether the build under test has the checkpoint flag. Probed WITHOUT binding a port: the old
# probe ran the daemon with `--port 1`, which cannot bind without privileges, so it exited
# non-zero whether or not it knew the flag and the answer was "refuses" on every build
# including one where C10-04 had landed. Argument parsing happens before anything is opened,
# and an unknown flag has its own message, so this reads the message.
checkpoint_probe() {
  wt="$1"
  out="$("$wt/target/release/nilestreamd" --checkpoint-interval 0 --schema /nonexistent/c10 2>&1)"
  if printf '%s' "$out" | grep -q "is not a flag this server knows"; then
    echo "refuses"
  elif printf '%s' "$out" | grep -q "cannot read /nonexistent/c10"; then
    echo "accepts"
  else
    echo "indeterminate"
  fi
}

# Run a command with a deadline, without depending on `timeout(1)` being installed.
run_with_deadline() {
  secs="$1"; shift
  "$@" &
  pid=$!
  waited=0
  while kill -0 "$pid" 2>/dev/null; do
    if [ "$waited" -ge "$secs" ]; then
      kill -TERM "$pid" 2>/dev/null
      sleep 2
      kill -KILL "$pid" 2>/dev/null
      wait "$pid" 2>/dev/null
      return 124
    fi
    sleep 1
    waited=$(( waited + 1 ))
  done
  wait "$pid"
  return $?
}

# ---------------------------------------------------------------------------

while [ $# -gt 0 ]; do
  case "$1" in
    --baseline-only) BASELINE_ONLY=1 ;;
    --self-test)     SELF_TEST=1 ;;
    --no-neutral)    NEUTRAL=0 ;;
    --baseline)  BASELINE_REF="${2:-}"; BASELINE_LABEL="baseline ${2:-}"; shift ;;
    --candidate) CANDIDATE_REF="${2:-}"; CANDIDATE_LABEL="candidate ${2:-}"; shift ;;
    --repo)   REPO="${2:-}"; shift ;;
    --out)    OUT="${2:-}"; shift ;;
    --warmups)  WARMUPS="${2:-}"; shift ;;
    --measured) MEASURED="${2:-}"; shift ;;
    --seconds)  SECONDS_PER_LEVEL="${2:-}"; shift ;;
    --publish)
      note "REFUSED: this script never writes a tracked artefact. Read the numbers, then"
      note "         land them yourself. (--publish exists on c10-publish.sh, not here.)"
      exit 2 ;;
    *) note "REFUSED: unknown flag \`$1\`. A mistyped flag that is ignored is a run that"
       note "         measured something other than what was asked for."
       exit 2 ;;
  esac
  shift
done

# ---------------------------------------------------------------------------
# --self-test: each refusal above, exercised against a fault it is supposed to catch.
# No build, no daemon, no network. Under a minute.
# ---------------------------------------------------------------------------
if [ "$SELF_TEST" -eq 1 ]; then
  head2 "self-test: every refusal, against the fault it is for"
  st_fail=0
  expect_refusal() {
    what="$1"; shift
    if "$@" >/dev/null 2>&1; then
      note "  NOT REFUSED: $what — this check does not catch the fault it is for"
      st_fail=1
    else
      note "  refused    : $what"
    fi
  }
  expect_accept() {
    what="$1"; shift
    if "$@" >/dev/null 2>&1; then
      note "  accepted   : $what"
    else
      note "  WRONGLY REFUSED: $what — a clean input must pass, or the check refuses"
      note "                   everything and its refusals mean nothing"
      st_fail=1
    fi
  }

  TD="$(mktemp -d "${TMPDIR:-/tmp}/c10-selftest.XXXXXX")"
  trap 'rm -rf "$TD"' EXIT

  # dirty worktree
  git -C "$TD" init -q rep 2>/dev/null
  ( cd "$TD/rep" && git config user.email c10@example.invalid && git config user.name c10 \
    && echo one > a.txt && git add a.txt && git commit -qm one ) >/dev/null 2>&1
  expect_accept "a clean repository"                     check_clean_repo "$TD/rep"
  echo two >> "$TD/rep/a.txt"
  expect_refusal "a dirty repository"                    check_clean_repo "$TD/rep"
  ( cd "$TD/rep" && git checkout -q -- . )

  # stale worktree, and a matching one
  SHA="$(git -C "$TD/rep" rev-parse HEAD)"
  git -C "$TD/rep" worktree add --detach -q "$TD/wt" "$SHA" >/dev/null 2>&1
  expect_accept "a worktree at the commit under test"    check_worktree_at "$TD/wt" "$SHA"
  expect_refusal "a worktree at another commit"          check_worktree_at "$TD/wt" "0000000000000000000000000000000000000000"
  expect_accept "a worktree that does not exist yet"     check_worktree_at "$TD/absent" "$SHA"

  # port collision, against a port this shell is holding open
  PORT=$(( PORT_BASE + 137 ))
  expect_accept "a free port"                            check_port_free "$PORT"

  # **A listener this test starts itself**, so the occupied case is exercised rather than
  # hoped for. No installs: whichever of perl or python3 is already on the machine opens a
  # socket, listens, and is killed on the way out. If neither exists the case is reported as
  # not run and the self-test fails — an untested refusal is not a refusal.
  LISTENER_PID=""
  if command -v perl >/dev/null 2>&1; then
    perl -e 'use IO::Socket::INET; my $s = IO::Socket::INET->new(LocalAddr=>"127.0.0.1", LocalPort=>$ARGV[0], Listen=>1, ReuseAddr=>1) or exit 1; sleep 60;' "$PORT" &
    LISTENER_PID=$!
  elif command -v python3 >/dev/null 2>&1; then
    python3 -c 'import socket,sys,time
s=socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(sys.argv[1]))); s.listen(1); time.sleep(60)' "$PORT" &
    LISTENER_PID=$!
  fi
  if [ -n "$LISTENER_PID" ]; then
    sleep 1
    if (exec 3<>"/dev/tcp/127.0.0.1/$PORT") 2>/dev/null; then
      exec 3>&- 2>/dev/null
      expect_refusal "an occupied port"                  check_port_free "$PORT"
    else
      note "  not run    : an occupied port (the listener did not come up on $PORT)"
      st_fail=1
    fi
    kill "$LISTENER_PID" 2>/dev/null
    wait "$LISTENER_PID" 2>/dev/null
  else
    note "  not run    : an occupied port (neither perl nor python3 is on this machine to"
    note "               open a socket with, and this script installs nothing)"
    st_fail=1
  fi

  # missing field, zero writer progress, and their clean controls
  cat > "$TD/good.log" <<'LOG'
  E19 mixed nilestream 6r/3w run 1 — 8000 reads/s (p50 210µs), 900 writes/s (p99 900µs), fallback 0.10%
  flights at 6r/3w: pending_joins 3, uninstalled_folds 0, pinned_installs 1, flights_refused 0, deferred_merges 0, waiters_refused 0
LOG
  cat > "$TD/missing.log" <<'LOG'
  WARNING: the server's slow-read table has no `fold_us` column; this build's table is reading a server that predates it
  E19 mixed nilestream 6r/3w run 1 — 8000 reads/s (p50 210µs), 900 writes/s (p99 900µs), fallback 0.10%
LOG
  cat > "$TD/na.log" <<'LOG'
  E19 mixed nilestream 6r/3w run 1 — 8000 reads/s (p50 210µs), 900 writes/s (p99 900µs), fallback 0.10%
  flights at 6r/3w: n/a (the server does not report the flight counters)
LOG
  cat > "$TD/zero.log" <<'LOG'
  E19 mixed nilestream 6r/3w run 1 — 8000 reads/s (p50 210µs), 0 writes/s (p99 0µs), fallback 0.00%
LOG
  cat > "$TD/nolevel.log" <<'LOG'
  E19 point nilestream @6 conn run 1 — 8000 ops/s, p99 300µs, fallback n/a
LOG
  expect_accept "a complete log"                         check_no_missing_fields "$TD/good.log"
  expect_refusal "a log missing a reported column"       check_no_missing_fields "$TD/missing.log"
  expect_refusal "a log whose flight counters are n/a"   check_no_missing_fields "$TD/na.log"
  expect_accept "a log with writer progress"             check_writer_progress "$TD/good.log"
  expect_refusal "a level with no writer progress"       check_writer_progress "$TD/zero.log"
  expect_refusal "a run with no mixed level at all"      check_writer_progress "$TD/nolevel.log"

  # the deadline
  expect_accept "a command inside its deadline"          run_with_deadline 5 sleep 1
  if run_with_deadline 2 sleep 30; then
    note "  NOT REFUSED: a command past its deadline"
    st_fail=1
  else
    note "  refused    : a command past its deadline (exit 124)"
  fi

  head2 "self-test: the flags"
  for bad in --nonsense --publish; do
    if bash "$0" "$bad" >/dev/null 2>&1; then
      note "  NOT REFUSED: \`$bad\`"
      st_fail=1
    else
      note "  refused    : \`$bad\`"
    fi
  done
  if bash "$0" >/dev/null 2>&1; then
    note "  NOT REFUSED: no arms named at all"
    st_fail=1
  else
    note "  refused    : no arms named at all"
  fi

  head2 "self-test: status"
  if [ "$st_fail" -eq 0 ]; then
    note "every injected fault was refused, and every clean control was accepted"
    exit 0
  fi
  note "AT LEAST ONE CHECK DID NOT DO ITS JOB — the harness is not admissible; see above"
  exit 1
fi

if [ "$BASELINE_ONLY" -eq 0 ] && [ -z "$CANDIDATE_REF" ]; then
  note "REFUSED: name a candidate with --candidate <ref>, or pass --baseline-only, or"
  note "         --self-test to exercise this script's own refusals."
  note "         A two-arm run with no second arm is a one-arm run that says otherwise."
  exit 2
fi

head2 "0. preflight"
note "date            : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
note "uname           : $(uname -a)"
note "repo            : $REPO"
note "out             : $OUT"

if [ ! -d "$REPO/.git" ]; then
  note "FATAL: $REPO is not a git checkout."; exit 3
fi
if ! command -v cargo >/dev/null 2>&1; then
  note "FATAL: no cargo on PATH."; exit 3
fi

check_clean_repo "$REPO" || exit 3

if [ -z "$BASELINE_REF" ]; then
  BASELINE_REF="$(git -C "$REPO" rev-parse --abbrev-ref HEAD 2>/dev/null)"
  BASELINE_LABEL="baseline $BASELINE_REF (the checked-out build)"
fi
if ! git -C "$REPO" cat-file -e "${BASELINE_REF}^{commit}" 2>/dev/null; then
  note "FATAL: $REPO does not contain the baseline commit $BASELINE_REF."; exit 3
fi
BASELINE_SHA="$(git -C "$REPO" rev-parse "${BASELINE_REF}^{commit}")"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  if ! git -C "$REPO" cat-file -e "${CANDIDATE_REF}^{commit}" 2>/dev/null; then
    note "FATAL: $REPO does not contain the candidate commit $CANDIDATE_REF."
    note "       Fetch the bundle first, then re-run."; exit 3
  fi
  CANDIDATE_SHA="$(git -C "$REPO" rev-parse "${CANDIDATE_REF}^{commit}")"
  if [ "$CANDIDATE_SHA" = "$BASELINE_SHA" ]; then
    note "REFUSED: the two arms resolve to the same commit $BASELINE_SHA. That is a neutral"
    note "         A=A control, which this script runs on purpose under --baseline-only; as"
    note "         a two-arm run it would be reported as a comparison of a change to itself."
    exit 2
  fi
fi

note "baseline        : $BASELINE_REF -> $BASELINE_SHA  $BASELINE_LABEL"
if [ "$BASELINE_ONLY" -eq 0 ]; then
note "candidate       : $CANDIDATE_REF -> $CANDIDATE_SHA  $CANDIDATE_LABEL"
fi
note "toolchain       : RUSTUP_TOOLCHAIN=stable, RUSTUP_AUTO_INSTALL=0 (never let rustup download)"
note "rustc           : $(RUSTUP_TOOLCHAIN=stable rustc --version 2>&1)"
note "cargo           : $(RUSTUP_TOOLCHAIN=stable cargo --version 2>&1)"
note "governor        : $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'n/a')"
note "nproc           : $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo '?')"
note "levels          : $LEVELS readers (writers = ceil(readers/2): 6r3w, 9r5w, 12r6w)"
note "working points  : full  = $ACCOUNTS accounts / $BUDGET_FULL budget (budget above the key count)"
note "                : part  = $ACCOUNTS accounts / $BUDGET_PARTIAL budget (genuinely partial)"
note "per level       : ${SECONDS_PER_LEVEL}s mixed, $WARMUPS warm-up + $MEASURED measured per arm"
note "replicate cap   : ${REPLICATE_TIMEOUT}s, after which the replicate is killed and counted as not run"
note "neutral A=A     : $( [ "$NEUTRAL" -eq 1 ] && echo 'yes, before anything is scored' || echo 'SKIPPED by --no-neutral' )"

mkdir -p "$OUT" || { note "FATAL: cannot create $OUT"; exit 3; }

head2 "0b. the bounded deadlock witness"
note "A10-01: a stats snapshot that held the view while it waited for the base, against an"
note "append that holds the base while it waits for the view. The mixed sweep below queries"
note "that table between levels, so this is run FIRST: a baseline that can hang is not a"
note "baseline, and a hang here means the run is blocked, not slow."
DW_LOG="$OUT/deadlock-witness.log"
( cd "$REPO" && RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true \
    cargo test --offline -p nilestream-server --lib \
      a_stats_snapshot_and_a_concurrent_append_both_finish \
      the_base_is_acquired_before_the_view_on_every_path_that_takes_both ) \
  >"$DW_LOG" 2>&1
DW_RC=$?
if [ "$DW_RC" -eq 0 ]; then
  note "witness: PASSED — both guards green; the diagnostic path cannot close the cycle."
  grep -E "^test result" "$DW_LOG" | sed 's/^/  /'
else
  note "witness: FAILED (exit $DW_RC). The baseline is BLOCKED, not slow. Do not score any"
  note "         performance task against this build; land the correctness repair first."
  sed -n '/panicked at/,+6p' "$DW_LOG" | head -20 | sed 's/^/  /'
  skip "the bounded deadlock witness failed; the baseline is blocked"
  head2 "status"
  note "SECTIONS THAT DID NOT RUN OR DID NOT COMPLETE:$SKIPPED"
  exit 1
fi

# ---------------------------------------------------------------------------
# Build each arm in its own detached worktree, so neither can be measured with
# the other's binary and neither touches the author's working tree.
# ---------------------------------------------------------------------------
build_arm() {
  arm="$1"; sha="$2"
  wt="$OUT/wt-$arm"
  check_worktree_at "$wt" "$sha" || return 1
  if [ ! -d "$wt" ]; then
    git -C "$REPO" worktree add --detach "$wt" "$sha" >/dev/null 2>&1 || {
      note "  could not create the worktree for $arm at $sha"; return 1; }
  fi
  ( cd "$wt" && RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true \
      cargo build --offline --release -p nilestream-server -p bank-bench >"$OUT/build-$arm.log" 2>&1 ) || {
    note "  build failed for $arm; see $OUT/build-$arm.log"; return 1; }
  built="$(git -C "$wt" rev-parse HEAD)"
  if [ "$built" != "$sha" ]; then
    note "  the $arm worktree is at $built and this run is about $sha"; return 1
  fi
  note "  $arm built at $built"
  return 0
}

head2 "1. build"
BASELINE_WT="$OUT/wt-baseline"
build_arm baseline "$BASELINE_SHA" || skip "the baseline arm did not build"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  CANDIDATE_WT="$OUT/wt-candidate"
  build_arm candidate "$CANDIDATE_SHA" || skip "the candidate arm did not build"
elif [ "$NEUTRAL" -eq 1 ]; then
  # The neutral arm is the same commit in a second worktree, built separately. Same source,
  # separate build directory, separate binary: if those alone move the number, the harness
  # cannot resolve anything smaller.
  CONTROL_WT="$OUT/wt-control"
  build_arm control "$BASELINE_SHA" || skip "the A=A control arm did not build"
fi

head2 "2. the checkpoint premise"
note "baseline  --checkpoint-interval : $(checkpoint_probe "$BASELINE_WT" 2>/dev/null || echo 'not probed')"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  note "candidate --checkpoint-interval : $(checkpoint_probe "$CANDIDATE_WT" 2>/dev/null || echo 'not probed')"
  note "'refuses' means the build has no checkpoint flag and the daemon ran at C = 0."
  note "'accepts' means C10-04 has landed on that arm. The two arms MAY differ when C10-04 is"
  note "the candidate — that is the change under test — and MUST NOT differ otherwise."
fi
note "Probed without binding a port: the previous probe ran the daemon on port 1, which"
note "cannot bind, so it exited non-zero on every build and always answered 'refuses'."

# ---------------------------------------------------------------------------
# One replicate: its own daemon, its own port, its own storage, torn down after.
# ---------------------------------------------------------------------------
rep_seq=0

replicate() {
  arm="$1"; wt="$2"; rep="$3"; tag="$4"; point="$5"; budget="$6"
  port=$(( PORT_BASE + 2 * (rep_seq % 40) ))
  outdir="$OUT/results-$arm-$point-$rep"
  rm -rf "$outdir"; mkdir -p "$outdir"
  log="$OUT/bench-$arm-$point-$rep.log"

  check_port_free "$port"       || { note "  [$tag] port $port is occupied"; return 1; }
  check_port_free $(( port + 1 )) || { note "  [$tag] port $(( port + 1 )) is occupied"; return 1; }

  # **One daemon per replicate, and the bench owns it.** `--host-nls` binds the port itself
  # and refuses to run if something is already there, so no replicate can silently measure
  # the previous one's warm server. Two ports are used (the point daemon and the fully
  # materialised report daemon one past it), hence the stride of two.
  # `env -C` is GNU; BSD `env` on Darwin has no `-C`, so the working directory is set by a
  # subshell instead. Host C is a Mac and this script is written for it.
  run_one() {
    cd "$wt" || return 3
    RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true \
      "$wt/target/release/bench" \
        --run --nls-only --scaling-only --host-nls --nls-port "$port" \
        --connections "$LEVELS" --mixed-seconds "$SECONDS_PER_LEVEL" \
        --accounts "$ACCOUNTS" --rounds "$ROUNDS" --nls-budget "$budget" \
        --runs 1 --out "$outdir" >"$log" 2>&1
  }
  run_with_deadline "$REPLICATE_TIMEOUT" run_one
  rc=$?

  if [ $rc -eq 124 ]; then
    note "  [$tag] killed at the ${REPLICATE_TIMEOUT}s cap. A replicate that has to be killed"
    note "         is not a slow measurement, it is no measurement; see $log"
    return 1
  fi
  if [ $rc -ne 0 ]; then
    note "  [$tag] bench exited $rc; see $log"
    return 1
  fi
  check_writer_progress "$log"   || { note "  [$tag] see $log"; return 1; }
  check_no_missing_fields "$log" || { note "  [$tag] see $log"; return 1; }

  # The whole mixed section, not a grep of it: the slowest-16 table and the lock histogram
  # are rows of numbers with no keyword in them, and a filter that drops them keeps only the
  # sentence saying a table exists.
  sed -n '/E19 mixed/,$p' "$log" | grep -v "connected as\|disconnected after" | sed 's/^/    /'
  return 0
}

# Both working points, one after the other, for one arm and one replicate index.
replicate_both_points() {
  arm="$1"; wt="$2"; rep="$3"; label="$4"
  ok=0
  note "  [$label] point: full ($ACCOUNTS accounts / $BUDGET_FULL budget)"
  rep_seq=$(( rep_seq + 1 ))
  replicate "$arm" "$wt" "$rep" "$label full" full "$BUDGET_FULL" && ok=$(( ok + 1 ))
  note "  [$label] point: partial ($ACCOUNTS accounts / $BUDGET_PARTIAL budget)"
  rep_seq=$(( rep_seq + 1 ))
  replicate "$arm" "$wt" "$rep" "$label partial" partial "$BUDGET_PARTIAL" && ok=$(( ok + 1 ))
  [ "$ok" -eq 2 ]
}

if [ "$BASELINE_ONLY" -eq 0 ]; then
  ARM_B_WT="$BASELINE_WT"; ARM_C_WT="${CANDIDATE_WT:-}"
  ARM_B="baseline";        ARM_C="candidate"
  ARM_B_LABEL="$BASELINE_LABEL"; ARM_C_LABEL="$CANDIDATE_LABEL"
else
  ARM_B_WT="$BASELINE_WT"; ARM_C_WT="${CONTROL_WT:-}"
  ARM_B="baseline";        ARM_C="control"
  ARM_B_LABEL="$BASELINE_LABEL"; ARM_C_LABEL="A=A control (same commit, second build)"
fi
TWO_ARMS=1
if [ "$BASELINE_ONLY" -eq 1 ] && [ "$NEUTRAL" -eq 0 ]; then TWO_ARMS=0; fi

head2 "3. warm-ups (discarded, but not ignored)"
note "A warm-up that did not complete is a cold arm measured as a warm one, so its exit is"
note "checked. The numbers are still thrown away."
i=0
warm_bad=0
while [ "$i" -lt "$WARMUPS" ]; do
  i=$(( i + 1 ))
  note "warm-up $i / $WARMUPS"
  if ! replicate_both_points "$ARM_B" "$ARM_B_WT" "w$i" "$ARM_B warm-up $i" >/dev/null 2>&1; then
    note "  the $ARM_B warm-up $i did not complete"
    warm_bad=1
  fi
  if [ "$TWO_ARMS" -eq 1 ]; then
    if ! replicate_both_points "$ARM_C" "$ARM_C_WT" "w$i" "$ARM_C warm-up $i" >/dev/null 2>&1; then
      note "  the $ARM_C warm-up $i did not complete"
      warm_bad=1
    fi
  fi
done
if [ "$warm_bad" -ne 0 ]; then
  skip "at least one warm-up did not complete; the measured arms below started from an unknown state"
fi

head2 "4. measured replicates, interleaved and order-balanced"
note "Interleaved on purpose: a drift in the machine between the arms is the one confound"
note "a back-to-back A-then-B run cannot separate from the change. Order-balanced on purpose"
note "too: with A always first, whatever the first arm of a pair pays — a cold page cache, a"
note "thermal step — is charged to A in every pair. Odd replicates run A then B, even ones B"
note "then A, so that cost lands on both arms equally."
ok_b=0; ok_c=0
i=0
while [ "$i" -lt "$MEASURED" ]; do
  i=$(( i + 1 ))
  note ""
  if [ $(( i % 2 )) -eq 1 ]; then
    note "--- replicate $i / $MEASURED : order AB ---"
    note "--- $ARM_B_LABEL ---"
    replicate_both_points "$ARM_B" "$ARM_B_WT" "$i" "$ARM_B $i" && ok_b=$(( ok_b + 1 ))
    if [ "$TWO_ARMS" -eq 1 ]; then
      note "--- $ARM_C_LABEL ---"
      replicate_both_points "$ARM_C" "$ARM_C_WT" "$i" "$ARM_C $i" && ok_c=$(( ok_c + 1 ))
    fi
  else
    note "--- replicate $i / $MEASURED : order BA ---"
    if [ "$TWO_ARMS" -eq 1 ]; then
      note "--- $ARM_C_LABEL ---"
      replicate_both_points "$ARM_C" "$ARM_C_WT" "$i" "$ARM_C $i" && ok_c=$(( ok_c + 1 ))
    fi
    note "--- $ARM_B_LABEL ---"
    replicate_both_points "$ARM_B" "$ARM_B_WT" "$i" "$ARM_B $i" && ok_b=$(( ok_b + 1 ))
  fi
done

head2 "5. what to paste back"
note "Paste this whole transcript. The reader wants, per arm, per working point and per level:"
note "  * median reads/s over the $MEASURED measured replicates, and the pooled MAD"
note "    (pooled MAD = sqrt((dA^2 + dB^2)/2) — the RMS, not the mean of the two MADs);"
note "  * the slowest-16 table with its outcome column, so a tail can be attributed to the"
note "    phase that produced it rather than averaged across rows that never ran it;"
note "  * base / base_read / base_write from the lock histogram, remembering that summed"
note "    shared holds overlap and are not occupancy;"
note "  * pending_joins, joins answered vs retried, uninstalled_folds, pinned_installs,"
note "    flights_refused, waiters_refused and deferred_merges — the last INCLUDING when it"
note "    is zero, which is the point;"
note "  * the anchor gap at begin and at finish, and how many flights fell further behind."
note ""
if [ "$BASELINE_ONLY" -eq 1 ] && [ "$NEUTRAL" -eq 1 ]; then
note "THE A=A ARM IS THE FALSIFICATION CONTROL. Its two arms are the same commit. Whatever"
note "difference appears between them is this harness's noise floor on this machine today,"
note "and no later result smaller than it is a result. THIS STAGE HAS NO PASS LINE."
else
note "Every C10 pass line is scored against THIS transcript's baseline arm, never against a"
note "document. A gate fires only on >= 10% AND >= 3 pooled MADs; else the row is"
note "noise-limited, which is a result."
fi
note ""
note "$ARM_B replicates that completed : $ok_b / $MEASURED (both working points each)"
if [ "$TWO_ARMS" -eq 1 ]; then
  note "$ARM_C replicates that completed: $ok_c / $MEASURED"
fi
note "raw logs and CSVs                  : $OUT"

if [ "$ok_b" -lt "$MEASURED" ]; then skip "the $ARM_B arm completed $ok_b of $MEASURED replicates"; fi
if [ "$TWO_ARMS" -eq 1 ] && [ "$ok_c" -lt "$MEASURED" ]; then
  skip "the $ARM_C arm completed $ok_c of $MEASURED replicates"
fi
if [ "$BASELINE_ONLY" -eq 1 ] && [ "$NEUTRAL" -eq 0 ]; then
  skip "the neutral A=A control was skipped by --no-neutral; nothing here establishes a noise floor"
fi

head2 "6. status"
if [ -n "$SKIPPED" ]; then
  note "SECTIONS THAT DID NOT RUN OR DID NOT COMPLETE:$SKIPPED"
  note ""
  note "A partial run is not a result. Report it as \`not done\` with this list."
else
  note "every section ran"
fi

note ""
note "worktrees left in place for re-runs; remove with:"
note "  git -C $REPO worktree remove --force $OUT/wt-baseline"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  note "  git -C $REPO worktree remove --force $OUT/wt-candidate"
elif [ "$NEUTRAL" -eq 1 ]; then
  note "  git -C $REPO worktree remove --force $OUT/wt-control"
fi

exit $FAILED
