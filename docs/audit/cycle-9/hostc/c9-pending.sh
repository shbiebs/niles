#!/usr/bin/env bash
# c9-pending.sh — the corrected section D for C9-06.2.
#
# WHAT THIS MEASURES
#   The two-phase keyed read (C9-06) against its immediate predecessor, at 6r/3w, 9r/5w and
#   12r/6w, with the arms interleaved inside one session so a thermal or scheduler drift
#   moves both together instead of moving one.
#
#   The cycle-8 curve (94,038 / 48,310 / 20,507 reads/s; view-wait max 22,160 us) is
#   INFERRED FROM A DOCUMENT and is not the baseline. The baseline is measured here, in this
#   session, from the predecessor SHA below.
#
# WHAT IT REFUSES TO DO
#   * It never writes a tracked artefact. `--publish` is refused, not ignored: results go to
#     a scratch directory under $OUT and the author lands them deliberately or not at all.
#   * It never installs anything and never reaches the network.
#   * It prints a status for every section. A section that did not run is named, and the exit
#     code is non-zero.
#
# USAGE
#   bash c9-pending.sh                    full run, both arms   (est. 25-35 min, cap 45)
#   bash c9-pending.sh --baseline-only    the baseline arm only (est. 12 min)
#   bash c9-pending.sh --repo ~/Documents/niles --out ~/c9-pending-out
#
# The author's zsh has `interactive_comments` off. Run this with `bash`, as written above.

set -uo pipefail

BASELINE_SHA="ebe7f8f"
# The branch tip, not a SHA: a script that names its own commit cannot name it, because
# writing the SHA in changes it. The resolved SHA is printed in the preflight, which is
# what the transcript needs.
CANDIDATE_REF="c9/07-pending"
BASELINE_LABEL="C9-05 (predecessor)"
CANDIDATE_LABEL="C9-06 (two-phase read)"

REPO="$HOME/Documents/niles"
OUT="$HOME/c9-pending-out"
WARMUPS=2
MEASURED=5
SECONDS_PER_LEVEL=30
LEVELS="6,9,12"
ACCOUNTS=2000
ROUNDS=8
BUDGET=2500
BASELINE_ONLY=0
PORT_BASE=6543

FAILED=0
SKIPPED=""

note()  { printf '%s\n' "$*"; }
head2() { printf '\n=== %s ===\n' "$*"; }
skip()  { SKIPPED="${SKIPPED}
  - $*"; FAILED=1; note "NOT RUN: $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --baseline-only) BASELINE_ONLY=1 ;;
    --repo)   REPO="$2"; shift ;;
    --out)    OUT="$2"; shift ;;
    --warmups)  WARMUPS="$2"; shift ;;
    --measured) MEASURED="$2"; shift ;;
    --seconds)  SECONDS_PER_LEVEL="$2"; shift ;;
    --publish)
      note "REFUSED: this script never writes a tracked artefact. Read the numbers, then"
      note "         land them yourself. (--publish exists on c9-publish.sh, not here.)"
      exit 2 ;;
    *) note "REFUSED: unknown flag \`$1\`. A mistyped flag that is ignored is a run that"
       note "         measured something other than what was asked for."
       exit 2 ;;
  esac
  shift
done

head2 "0. preflight"
note "date            : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
note "uname           : $(uname -a)"
note "repo            : $REPO"
note "out             : $OUT"
note "baseline        : $BASELINE_SHA  $BASELINE_LABEL"
note "toolchain       : RUSTUP_TOOLCHAIN=stable, RUSTUP_AUTO_INSTALL=0 (never let rustup download)"
note "candidate       : $CANDIDATE_REF -> $(git -C "$REPO" rev-parse --short "$CANDIDATE_REF" 2>/dev/null || echo 'UNRESOLVED')  $CANDIDATE_LABEL"
note "levels          : $LEVELS readers (writers = ceil(readers/2): 6r3w, 9r5w, 12r6w)"
note "per level       : ${SECONDS_PER_LEVEL}s mixed, $WARMUPS warm-up + $MEASURED measured per arm"
note "budget/accounts : $BUDGET / $ACCOUNTS   rounds $ROUNDS"

if ! command -v cargo >/dev/null 2>&1; then
  note "FATAL: no cargo on PATH."; exit 3
fi
note "rustc           : $(rustc --version 2>&1)"
note "cargo           : $(cargo --version 2>&1)"
note "governor        : $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'n/a')"
note "nproc           : $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo '?')"

if [ ! -d "$REPO/.git" ]; then
  note "FATAL: $REPO is not a git checkout."; exit 3
fi
if ! git -C "$REPO" cat-file -e "${BASELINE_SHA}^{commit}" 2>/dev/null; then
  note "FATAL: $REPO does not contain the baseline commit $BASELINE_SHA."; exit 3
fi
if [ "$BASELINE_ONLY" -eq 0 ] && ! git -C "$REPO" cat-file -e "${CANDIDATE_REF}^{commit}" 2>/dev/null; then
  note "FATAL: $REPO does not contain the candidate commit $CANDIDATE_REF."
  note "       Fetch the cycle-9 bundle first, then re-run."; exit 3
fi

mkdir -p "$OUT" || { note "FATAL: cannot create $OUT"; exit 3; }

# ---------------------------------------------------------------------------
# Build each arm in its own detached worktree, so neither can be measured with
# the other's binary and neither touches the author's working tree.
# ---------------------------------------------------------------------------
build_arm() {
  arm="$1"; sha="$2"
  wt="$OUT/wt-$arm"
  if [ ! -d "$wt" ]; then
    git -C "$REPO" worktree add --detach "$wt" "$sha" >/dev/null 2>&1 || {
      note "  could not create the worktree for $arm at $sha"; return 1; }
  fi
  ( cd "$wt" && RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0 \
      cargo build --offline --release -p nilestream-server -p bank-bench >"$OUT/build-$arm.log" 2>&1 ) || {
    note "  build failed for $arm; see $OUT/build-$arm.log"; return 1; }
  note "  $arm built: $(git -C "$wt" rev-parse --short HEAD)"
  return 0
}

# Both arms must agree about checkpoints, and at these SHAs neither has the flag: the
# checkpoint work is C9-07 and must not be entangled with this measurement. Probing for it
# is cheaper than asserting it, and an arm that accepted the flag would make the two runs
# incomparable without saying so.
checkpoint_probe() {
  wt="$1"
  if "$wt/target/release/nilestreamd" --checkpoint-interval 0 --port 1 >/dev/null 2>&1; then
    echo "accepts"
  else
    echo "refuses"
  fi
}

head2 "1. build"
build_arm baseline "$BASELINE_SHA" || { skip "the baseline arm did not build"; }
BASELINE_WT="$OUT/wt-baseline"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  build_arm candidate "$CANDIDATE_REF" || { skip "the candidate arm did not build"; }
  CANDIDATE_WT="$OUT/wt-candidate"
fi

head2 "2. the checkpoint premise"
note "baseline  --checkpoint-interval : $(checkpoint_probe "$BASELINE_WT" 2>/dev/null || echo 'not probed')"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  note "candidate --checkpoint-interval : $(checkpoint_probe "$CANDIDATE_WT" 2>/dev/null || echo 'not probed')"
  note "Both arms must read the same. 'refuses' on both is the expected reading at these"
  note "SHAs: the daemon has no checkpoint flag until C9-07, so checkpoints are 0 on both"
  note "arms by construction and Pending is measured on its own."
fi

# ---------------------------------------------------------------------------
# One replicate: its own daemon, its own port, its own storage, torn down after.
# ---------------------------------------------------------------------------
replicate() {
  arm="$1"; wt="$2"; rep="$3"; tag="$4"
  port=$(( PORT_BASE + 2 * (rep_seq % 40) ))
  outdir="$OUT/results-$arm-$rep"
  rm -rf "$outdir"; mkdir -p "$outdir"

  # **One daemon per replicate, and the bench owns it.** `--host-nls` binds the port itself
  # and refuses to run if something is already there, so no replicate can silently measure
  # the previous one's warm server — which is the failure mode a separately spawned daemon
  # and a `sleep` invite. Two ports are used (the point daemon and the fully materialised
  # report daemon one past it), hence the stride of two.
  ( cd "$wt" && RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0 \
      ./target/release/bench \
        --run --nls-only --scaling-only --host-nls --nls-port "$port" \
        --connections "$LEVELS" --mixed-seconds "$SECONDS_PER_LEVEL" \
        --accounts "$ACCOUNTS" --rounds "$ROUNDS" --nls-budget "$BUDGET" \
        --runs 1 --out "$outdir" ) >"$OUT/bench-$arm-$rep.log" 2>&1
  rc=$?

  if [ $rc -ne 0 ]; then
    note "  [$tag] bench exited $rc; see $OUT/bench-$arm-$rep.log"
    return 1
  fi
  if ! grep -q "E19 mixed" "$OUT/bench-$arm-$rep.log"; then
    note "  [$tag] the run produced no mixed level; see $OUT/bench-$arm-$rep.log"
    return 1
  fi
  # The whole mixed section, not a grep of it: the slowest-16 table is rows of numbers with
  # no keyword in them, and a filter that drops the table keeps only the sentence saying a
  # table exists.
  sed -n '/E19 mixed/,$p' "$OUT/bench-$arm-$rep.log" | sed 's/^/    /'
  return 0
}

rep_seq=0

head2 "3. warm-ups (discarded)"
i=0
while [ "$i" -lt "$WARMUPS" ]; do
  i=$(( i + 1 ))
  note "warm-up $i / $WARMUPS"
  rep_seq=$(( rep_seq + 1 ))
  replicate baseline "$BASELINE_WT" "w$i" "baseline warm-up $i" >/dev/null 2>&1
  if [ "$BASELINE_ONLY" -eq 0 ]; then
    rep_seq=$(( rep_seq + 1 ))
    replicate candidate "$CANDIDATE_WT" "w$i" "candidate warm-up $i" >/dev/null 2>&1
  fi
done

head2 "4. measured replicates, interleaved"
note "Interleaved on purpose: a drift in the machine between the arms is the one confound"
note "a back-to-back A-then-B run cannot separate from the change."
ok_b=0; ok_c=0
i=0
while [ "$i" -lt "$MEASURED" ]; do
  i=$(( i + 1 ))
  note ""
  note "--- replicate $i / $MEASURED : $BASELINE_LABEL ---"
  rep_seq=$(( rep_seq + 1 ))
  if replicate baseline "$BASELINE_WT" "$i" "baseline $i"; then ok_b=$(( ok_b + 1 )); fi
  if [ "$BASELINE_ONLY" -eq 0 ]; then
    note "--- replicate $i / $MEASURED : $CANDIDATE_LABEL ---"
    rep_seq=$(( rep_seq + 1 ))
    if replicate candidate "$CANDIDATE_WT" "$i" "candidate $i"; then ok_c=$(( ok_c + 1 )); fi
  fi
done

head2 "5. what to paste back"
note "Paste this whole transcript. The reader wants, per arm and per level:"
note "  * median reads/s over the $MEASURED measured replicates, and the pooled MAD;"
note "  * the slowest-16 table's view_wait_us maximum;"
note "  * the nilestream_stats columns pending_joins / uninstalled_folds / pinned_installs /"
note "    flights_refused at 12r6w (C9-06.3 needs pending_joins > 0 over the wire);"
note "  * any level whose durability or contract counters differ between the arms."
note ""
note "The pass line for C9-06.2 is: candidate median >= 1.25x baseline median at 12r6w,"
note "under the >= 10% AND >= 3 pooled-MAD gate, with no contract or durability regression."
note "The view_wait_us maximum is REPORTED beside it, not required."
note ""
note "baseline replicates that completed : $ok_b / $MEASURED"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  note "candidate replicates that completed: $ok_c / $MEASURED"
fi
note "raw logs and CSVs                  : $OUT"

if [ "$ok_b" -lt "$MEASURED" ]; then skip "the baseline arm completed $ok_b of $MEASURED replicates"; fi
if [ "$BASELINE_ONLY" -eq 0 ] && [ "$ok_c" -lt "$MEASURED" ]; then
  skip "the candidate arm completed $ok_c of $MEASURED replicates"
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
fi

exit $FAILED
