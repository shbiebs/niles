#!/usr/bin/env bash
#
# T11-01 — the two interleaved pairs that separate the compiler from the code (F-11-04).
#
#   Pair 1  same commit (653a369, merging OFF), 1.95.0 against 1.97.1     the toolchain effect
#   Pair 2  same toolchain (the pin, 1.95.0), e29a025 against 653a369-OFF  the code effect
#
# Each pair is c11-baselock.sh with two arms, warm-ups discarded, five measured replicates,
# AB/BA balanced, one daemon per replicate; every replicate's transcript records the rustc that
# built its arm from inside that arm's worktree. Runtime: roughly 40 minutes per pair at the
# defaults — run them one after the other, not together. Estimate 80-90 minutes.
#
# Why two pairs and not one: the cycle-10 T00 baseline (e29a025) was built with 1.97.1 (MF-7) and
# the T04.2 control (d680c83) with 1.95.0, in different sessions; their 5-9% difference is
# attributable to neither until one of the two variables is held still. Pair 1 holds the code;
# pair 2 holds the compiler. Both must be five replicates; a pair that completes fewer is not a
# result and the script says so.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
REPO="${C11_NILES:-$HOME/Documents/niles}"
if ! ( cd "$REPO" && rustup run 1.97.1 rustc --version ) >/dev/null 2>&1; then
  echo "REFUSED: rustup cannot run 1.97.1 on this host, so pair 1 cannot be built. Pair 2 alone"
  echo "         cannot separate the compiler from the code; do not run it and report it as a"
  echo "         partial. (Host C had 1.97.1 installed through cycle 10.)"
  exit 3
fi
# Two pairs, two output directories: baselock refuses a worktree that is not at the commit the
# run is about, so a second pair reusing the first pair's directory would be refused at its
# first build (the MF-13 rule, applied to this script). And pair 1's BASELINE arm is 653a369 too,
# so it must be launched at `off` as well — the first draft of this script left it at default,
# which would have compared merging-on-1.95.0 against merging-off-1.97.1: two variables, no
# answer (F-11-20). Both arms of pair 1 now carry --*-caps off and the daemon's own merge line
# is checked in both.
OUT_ROOT="${C11_OUT:-$HOME/c11-pairs-out}"
echo "=== PAIR 1: same commit 653a369 (merging off in BOTH arms), 1.95.0 vs 1.97.1 ==="
bash "$here/c11-baselock.sh" --repo "$REPO" --no-neutral --out "$OUT_ROOT/pair1" \
  --baseline 653a369 --candidate 653a369 \
  --baseline-caps off --candidate-caps off \
  --baseline-toolchain 1.95.0 --candidate-toolchain 1.97.1 "$@"
rc1=$?
echo
echo "=== PAIR 2: same toolchain (the pin), e29a025 vs 653a369 (merging off) ==="
bash "$here/c11-baselock.sh" --repo "$REPO" --no-neutral --out "$OUT_ROOT/pair2" \
  --baseline e29a025 --candidate 653a369 --candidate-caps off "$@"
rc2=$?
echo
echo "pair 1 exit $rc1, pair 2 exit $rc2. Raw output under $OUT_ROOT/pair1 and $OUT_ROOT/pair2. Paste both transcripts whole."
[ "$rc1" -eq 0 ] && [ "$rc2" -eq 0 ]
