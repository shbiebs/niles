#!/usr/bin/env bash
# c11-baselock.sh — the base-lock measurement, and the two-arm harness every cycle-11 pass
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
#   bash c11-baselock.sh --baseline-only                    current build + A=A (est. 25-40 min)
#   bash c11-baselock.sh --self-test                        the fault injections only (< 1 min)
#   bash c11-baselock.sh --candidate c10/04-deferred-merge  two arms, interleaved
#   bash c11-baselock.sh --baseline <ref> --candidate <ref> --out ~/c10-out
#
#   Both refs are BRANCH NAMES or SHAs that exist in the repo; the resolved SHA of each is
#   printed in the preflight and is what the transcript is about.
#
# The author's zsh has `interactive_comments` off. Run this with `bash`, as written above.

set -uo pipefail

REPO="${C11_REPO:-$HOME/Documents/niles}"
OUT="${C11_OUT:-$HOME/c11-baselock-out}"

# **What this run was asked to do, kept verbatim for the manifest.** A transcript that records
# its own arguments is the difference between "the pinned arm was slower" and "the pinned arm,
# as this invocation defined `pinned`, was slower". Captured before parsing, because parsing
# is where a mistyped flag stops being visible.
INVOCATION="$0 $*"

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
MERGE_ARMS=0
CANDIDATE_CAPS=default
BASELINE_CAPS=default
BASELINE_TOOLCHAIN=""
CANDIDATE_TOOLCHAIN=""
NEUTRAL=1
SELF_TEST=0
PORT_BASE=6543
REPLICATE_TIMEOUT=900

FAILED=0
SKIPPED=""

# **The toolchain: the tree's pin when it resolves, `stable` only when it does not.**
#
# Both of this cycle's scripts exported `RUSTUP_TOOLCHAIN=stable` unconditionally. That was
# written for the cloud container, where the pinned 1.95.0 cannot be resolved (no egress to
# static.rust-lang.org) and `stable` *is* 1.95.0 — so the override was invisible there. On
# Host C the pin resolves and `stable` is 1.97.1, so the override silently swapped the
# compiler: every gate and the whole cycle-11 baseline were built with a toolchain the tree
# does not specify, and nothing said so.
#
# The rule now: use the pin if it resolves, fall back to `stable` if it does not, and print
# which happened. A measurement built with a compiler other than the one the tree pins is not
# wrong, but it is a different measurement and it must be labelled.
# The toolchain probe is shared: see toolchain.sh beside this script (F-11-01).
#
# `HERE` is resolved once, before anything can change directory: `$0` is relative when the
# script is invoked by a relative path, and `dirname "$0"` then stops naming this directory
# the moment any code runs from somewhere else. The gates script's self-test caught exactly
# that, and this is the same line.
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/toolchain.sh"

note()  { printf '%s\n' "$*"; }
head2() { printf '\n=== %s ===\n' "$*"; }
skip()  { SKIPPED="${SKIPPED}
  - $*"; FAILED=1; note "NOT RUN: $*"; }

# **A refusal that the run cannot continue past.**
#
# `skip` records and returns, which is right for a section whose absence leaves the rest of
# the transcript meaningful — a checkpoint probe that could not run does not invalidate the
# replicates below it. It is wrong for anything the measurement is *made of*, and the
# difference cost a 40-minute run: a leftover worktree at an old commit was refused by
# `build_arm`, and the script then ran every warm-up and all five replicates against the
# binary already sitting in that directory. Twenty refusals followed, every one of them
# naming the wrong cause, and the right one had scrolled past.
#
# The shape is the paired adapter gate's, which printed `SKIPPED` and returned `ok` while the
# pair was broken for a whole cycle. A refusal that does not stop the thing it refuses is a
# note, and notes do not gate.
# The status section and the exit, in one place because `fatal` needs them too: a run that
# stops early must still say what it did and did not do, in the same words as one that
# finishes, or a reader has to learn two shapes of transcript.
summary_and_exit() {
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
}

fatal() {
  skip "$*"
  note ""
  note "STOPPING. This is not a section that can be missing from a transcript: everything"
  note "below it would be measured against something this script has just refused, and would"
  note "be reported under a label the refusal says is wrong."
  summary_and_exit
}

# ---------------------------------------------------------------------------
# Checks, written as functions so `--self-test` can inject a fault into each one
# and require the refusal. A check that is only ever exercised by the happy path
# is a check nobody has seen work.
# ---------------------------------------------------------------------------

# A repository with uncommitted changes measures something that is in no commit, so the SHA
# in the transcript names a build that does not exist.
check_clean_repo() {
  repo="$1"
  # **Tracked files only, and the reason is not leniency.** Each arm is built in a detached
  # worktree at a resolved SHA, so nothing in the author's working tree reaches the binary
  # either way; what the check is protecting is the transcript's *label*, which says "the
  # checked-out build". A modified tracked file makes that label name a build that exists
  # nowhere. An untracked file cannot: it is in no commit and in no worktree.
  #
  # This matters concretely. Host C's `niles` checkout permanently carries five untracked
  # files — `.DS_Store`, `AGENTS.md`, `niles/.DS_Store`, `thesis/.DS_Store` and
  # `thesis/Niles-Thesis.pdf` — which the audit protocol says are never to be touched. A
  # check that refused on those would refuse every run on the machine the run is for, and the
  # obvious way out would have been to delete them.
  #
  # **One further exemption, and it is derived rather than listed.** `make reproduce`
  # regenerates a few files under `results/` whose bytes are a property of the host or the
  # toolchain rather than of this code — E18's allocation *byte* columns, the `psql` version
  # that drove the wire transcript. On Host C running the reproduction gate therefore leaves
  # them modified, permanently, and a check that refused on them would refuse every run on
  # the machine the run is for: the same failure mode the untracked exemption above exists to
  # avoid, and with the same obvious wrong way out. They are exempt for the same reason too —
  # each arm is built in a detached worktree at a resolved SHA, so a modified working-tree
  # copy reaches no binary.
  #
  # The exempt set is read out of `results/MANIFEST.csv`, never written here. A second list
  # of the same files in a second file is how the reproduction diff came to disagree with the
  # class it claimed to follow; repeating that mistake in the script that gates the
  # measurement would be worse, because this one refuses rather than reports. A repository
  # with no manifest exempts nothing, which is the right answer for `gbs`.
  manifest="$repo/results/MANIFEST.csv"
  exempt=""
  if [ -f "$manifest" ]; then
    exempt="$(awk -F, '$2=="toolchain-scoped"||$2=="host-scoped" {print "results/" $1}' \
      "$manifest")"
  fi

  dirty="$(git -C "$repo" status --porcelain --untracked-files=no 2>/dev/null)"
  kept=""
  waived=""
  # `status --porcelain` prints two status columns, a space, then the path.
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    path="${line#???}"
    if [ -n "$exempt" ] && printf '%s\n' "$exempt" | grep -qxF "$path"; then
      waived="${waived}${path}
"
    else
      kept="${kept}${line}
"
    fi
  done <<EOF
$dirty
EOF

  # A waived file is named, not hidden. The point of the exemption is that these differ; a
  # reader of the transcript should see which ones did.
  if [ -n "$waived" ]; then
    note "waived          : $(printf '%s' "$waived" | grep -c .) modified results file(s) whose"
    note "                  class in results/MANIFEST.csv says their bytes are not comparable"
    note "                  across hosts, so the reproduction gate is expected to change them:"
    printf '%s' "$waived" | sed 's/^/                    /'
  fi

  if [ -n "$kept" ]; then
    note "REFUSED: $repo has modified tracked files, so the SHA below would name a build that"
    note "         is not the one measured:"
    printf '%s' "$kept" | head -20 | sed 's/^/           /'
    return 1
  fi
  return 0
}

# Untracked files are not a refusal, but they are not invisible either: they are counted and
# named in the preflight so a reader can see what is in the tree that is in no commit.
report_untracked() {
  repo="$1"
  u="$(git -C "$repo" ls-files --others --exclude-standard 2>/dev/null)"
  if [ -z "$u" ]; then
    note "untracked       : none"
    return 0
  fi
  n="$(printf '%s\n' "$u" | wc -l | tr -d ' ')"
  note "untracked       : $n file(s), in no commit and in no built worktree:"
  printf '%s\n' "$u" | head -12 | sed 's/^/                  /'
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
# ---------------------------------------------------------------------------------------------
# THE RUN MANIFEST, AND THE RULE THAT A REFUSED STAGE IS NEVER LAUNCHED (C11-06.3)
# ---------------------------------------------------------------------------------------------
#
# A 40-minute cycle-10 run was lost to this exact shape: `build_arm` refused a worktree that
# was at the wrong commit, the refusal was recorded with `skip`, and every warm-up and all five
# replicates then ran against the binary already sitting in that directory. Twenty refusals
# followed, each naming the wrong cause. `fatal` fixed the *recording*; it did not make the
# rule structural, because the next `|| skip` somebody writes puts it back.
#
# The rule here is about the executable rather than about the control flow. Each arm's binary
# is hashed when it is built, into `$OUT/exe-<arm>.sha`. Nothing launches a binary whose hash
# is not the hash this run recorded — an arm whose build never ran has no recorded hash and is
# therefore unlaunchable, whatever is on disk from a previous run. A stale executable cannot be
# reached by a stage whose prerequisite failed, and the self-test proves it with a sentinel:
# the stale binary would create a file if launched, and the arm asserts the file is absent.

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" 2>/dev/null | cut -d' ' -f1
  elif command -v shasum   >/dev/null 2>&1; then shasum -a 256 "$1" 2>/dev/null | cut -d' ' -f1
  else echo "no-sha256-tool"; fi
}

# One row per stage, appended as it happens: a stage that never appears did not run, and a
# stage that ran and failed says so in the same file. `not run` is a value, not an absence.
stage() {
  [ -d "$OUT" ] || return 0
  printf '%s,%s,%s\n' "$1" "$2" "$(printf '%s' "${3:-}" | tr ',\n' ';;')" >> "$OUT/manifest.csv"
}

record_exe() {
  arm="$1"; wt="$2"
  for b in bench nilestreamd; do
    sha256_of "$wt/target/release/$b" > "$OUT/exe-$arm-$b.sha"
  done
}

# **The launch guard.** Returns non-zero — and says which of the two reasons it is — when the
# binary about to be launched is not the one this run built.
may_launch() {
  arm="$1"; wt="$2"; bin="${3:-bench}"
  rec="$OUT/exe-$arm-$bin.sha"
  if [ ! -s "$rec" ]; then
    note "  REFUSED TO LAUNCH: no hash was recorded for the $arm arm's $bin, so its build stage"
    note "                     did not complete. Whatever is in $wt/target/release is a"
    note "                     previous run's binary and this run knows nothing about it."
    return 1
  fi
  now="$(sha256_of "$wt/target/release/$bin")"
  if [ "$now" != "$(cat "$rec")" ]; then
    note "  REFUSED TO LAUNCH: the $arm arm's $bin on disk is not the one this run built"
    note "                     (recorded $(cut -c1-12 < "$rec"), found $(printf '%s' "$now" | cut -c1-12))."
    return 1
  fi
  return 0
}

# **Read one field out of the benchmark's machine-readable lines.**
#
# Every check below used to grep an English sentence out of the transcript, which put this
# harness's interface in the benchmark's *presentation*: rewording a line for a human reader
# could move a verdict, and three cycle-10 defects came out of that one habit — a substring
# match that read `900 writes/s` as `0 writes/s`, a check that lived under one mode and
# watched nothing in the others (F-11-20), and a server column renamed under a reader that
# answered `unwrap_or(0)` for four cycles.
#
# `bench` now prints one `NLSBENCH/1 kind=<kind> k=v ...` line per reported event. This prints
# the raw token of one field, one line of output per matching line in the log, and it keeps
# the four outcomes four:
#
#   (nothing)       no line of that kind at all — this build does not print them
#   __MISSING__     the line is there and does not carry the field: a different build
#   n/a             reported, and not obtained this run
#   0               a measurement, and it is zero
#
# A caller that wants "absent or zero" has to say which, at its own call site.
summary_fields() {
  awk -v kind="$2" -v want="$3" '
    /^NLSBENCH\/1 / {
      k = ""; v = "__MISSING__"
      for (i = 2; i <= NF; i++) {
        eq = index($i, "=")
        if (eq == 0) continue
        key = substr($i, 1, eq - 1)
        val = substr($i, eq + 1)
        if (key == "kind") k = val
        else if (key == want) v = val
      }
      if (k == kind) print v
    }
  ' "$1" 2>/dev/null
}

check_no_missing_fields() {
  log="$1"
  if grep -q "WARNING: the server's slow-read table has no" "$log" 2>/dev/null; then
    note "REFUSED: the server in this run does not report a column this transcript is about:"
    grep "WARNING: the server's slow-read table has no" "$log" | head -4 | sed 's/^/           /'
    return 1
  fi
  # **The structured path.** A build that prints `flights` lines is checked field by field,
  # and `n/a` (asked, not obtained) is told from `0` (measured) and from a field this build
  # does not carry at all.
  vals="$(summary_fields "$log" flights pending_joins)"
  if [ -n "$vals" ]; then
    if printf '%s\n' "$vals" | grep -qx '__MISSING__'; then
      note "REFUSED: a flights line in this transcript does not carry \`pending_joins\`, so the"
      note "         build that wrote it is not the build this harness is scoring."
      return 1
    fi
    if printf '%s\n' "$vals" | grep -qx 'n/a'; then
      note "REFUSED: the flight counters came back n/a, so this build does not report them and"
      note "         every flight number in this transcript would be absent rather than zero."
      return 1
    fi
    return 0
  fi
  # **The legacy path, and it says it is one.** Pair 2 of the cycle-11 campaign compares two
  # commits that both predate the machine-readable line; refusing them would be refusing the
  # measurement the campaign exists for. So the prose grep still runs — and the run's manifest
  # records `summary_lines=absent` for this arm, so a reader knows which check was applied.
  if grep -q "flights at .*: n/a" "$log" 2>/dev/null; then
    note "REFUSED: the flight counters came back n/a, so this build does not report them and"
    note "         every flight number in this transcript would be absent rather than zero."
    return 1
  fi
  return 0
}

# **In --merge-arms, a transcript must name the arm that produced it.**
#
# The merging build and the pinned control are one binary with two values of
# NILESTREAM_MERGE_CAPS, which is what makes the comparison a comparison of one change. The
# price of that is that nothing about the binary distinguishes the arms, so the *only* thing
# that can is the caps the daemon reports back. A run whose log does not carry the merge line
# is a run whose arm rests on the invocation having been typed correctly, and the whole point
# of an ablation is not to have to trust that.
check_merge_line() {
  log="$1"; want="$2"
  # **The structured path.** Every `flights` line carries the caps the daemon ran with, so an
  # arm is checked once per level rather than once per log: a run whose first level agreed and
  # whose later ones did not would have passed the old single-line check.
  ce="$(summary_fields "$log" flights caps_epochs)"
  cr="$(summary_fields "$log" flights caps_rows)"
  if [ -n "$ce" ]; then
    if printf '%s\n' "$ce$cr" | grep -qx '__MISSING__'; then
      note "REFUSED: a flights line carries no caps, so its arm rests on the flag it was given."
      return 1
    fi
    n=0
    while [ "$n" -lt "$(printf '%s\n' "$ce" | wc -l | tr -d ' ')" ]; do
      n=$(( n + 1 ))
      e="$(printf '%s\n' "$ce" | sed -n "${n}p")"
      r="$(printf '%s\n' "$cr" | sed -n "${n}p")"
      if [ "$e:$r" != "$want" ]; then
        note "REFUSED: level $n of this replicate was launched as caps \`$want\` and the daemon"
        note "         reported \`$e:$r\`. One of the two is wrong and neither may be scored."
        return 1
      fi
    done
    return 0
  fi
  line="$(grep -m1 "  merge at " "$log" 2>/dev/null || true)"
  if [ -z "$line" ]; then
    note "REFUSED: this build does not report the merge counters, so its arm cannot be told"
    note "         from the other one. Both arms would be labelled by the flag they were"
    note "         given rather than by what the daemon did."
    return 1
  fi
  caps="$(printf '%s\n' "$line" | sed -n 's/.*caps \([0-9]*\) epochs \/ \([0-9]*\) rows.*/\1:\2/p')"
  if [ "$caps" != "$want" ]; then
    note "REFUSED: this replicate was launched as caps \`$want\` and the daemon reported"
    note "         \`$caps\`. One of the two is wrong and neither may be scored:"
    printf '%s\n' "$line" | sed 's/^/           /'
    return 1
  fi
  return 0
}

# **The daemon, not the flag, says which caps an arm ran with — in every mode.** The first
# two-arm `--candidate-caps off` run of this script labelled its candidate by the flag it was
# given and checked nothing, because the merge-line check lived only under `--merge-arms`
# (F-11-20). A build that prints the merge line must agree with the caps it was launched with;
# a build that prints none is accepted only at `default` outside `--merge-arms`, which is what a
# commit predating the counter can honestly be — and is refused when the arm asked for anything
# else, because a flag the daemon cannot have read is a label and not a setting.
check_arm_caps() {
  log="$1"; caps="$2"; merge_arms="${3:-0}"
  want="$caps"
  [ "$want" = "default" ] && want="32:4096"
  [ "$want" = "off" ] && want="0:0"
  # **A build that says which caps it ran with is always checked against them.** The earlier
  # version reached the check only under `--merge-arms` or a non-default request, so a
  # structured transcript reporting caps 0/0 for an arm launched at `default` passed — the
  # self-test arm for it is `a structured pinned arm launched at default`.
  if [ -n "$(summary_fields "$log" flights caps_epochs)" ] || grep -q "  merge at " "$log" 2>/dev/null; then
    check_merge_line "$log" "$want"
    return $?
  fi
  # No line of either kind: this build reports no merge counters at all. That is honest for a
  # commit predating them, and only at `default`, where nothing was asked of it.
  if [ "$caps" != "default" ] || [ "$merge_arms" -eq 1 ]; then
    check_merge_line "$log" "$want"
    return $?
  fi
  return 0
}

# A mixed level with no writer progress is a read-only level wearing a mixed level's name,
# and the anchor-mismatch fallback this whole harness exists to observe cannot occur in it.
check_writer_progress() {
  log="$1"
  # **The structured path**, which is where the substring bug could not have happened: the
  # writer rate is a field, not a token inside a sentence, so `900` cannot contain `0`.
  w="$(summary_fields "$log" mixed writes_per_second)"
  if [ -n "$w" ]; then
    if printf '%s\n' "$w" | grep -qx '__MISSING__\|n/a'; then
      note "REFUSED: a mixed level did not report a writer rate at all, so whether its readers"
      note "         saw a moving frontier is unknown rather than answered."
      return 1
    fi
    if printf '%s\n' "$w" | grep -qx '0\|0\.0'; then
      note "REFUSED: a mixed level made no writer progress, so its reads never saw a moving"
      note "         frontier and its fallback rate is about nothing."
      return 1
    fi
    return 0
  fi
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
    --merge-arms)    MERGE_ARMS=1; BASELINE_ONLY=1; NEUTRAL=0 ;;
    --candidate-caps)     CANDIDATE_CAPS="${2:-}"; shift ;;
    --baseline-caps)      BASELINE_CAPS="${2:-}"; shift ;;
    --baseline-toolchain) BASELINE_TOOLCHAIN="${2:-}"; shift ;;
    --candidate-toolchain) CANDIDATE_TOOLCHAIN="${2:-}"; shift ;;
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
      note "         land them yourself. (--publish exists on c11-publish.sh, not here.)"
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
  expect_refusal "a repository with a modified tracked file" check_clean_repo "$TD/rep"
  ( cd "$TD/rep" && git checkout -q -- . )
  # An untracked file must NOT be a refusal: Host C's checkout permanently carries five, and
  # a check that refused on those would refuse every run on the machine it was written for.
  echo scratch > "$TD/rep/untracked-thing.txt"
  expect_accept "a repository with only untracked files"  check_clean_repo "$TD/rep"
  rm -f "$TD/rep/untracked-thing.txt"

  # The manifest-derived waiver, both ways round. A fixture repository gets a two-row
  # manifest and the two files it classes, and the check must waive the incomparable one and
  # still refuse the byte-deterministic one — otherwise the waiver is a hole rather than an
  # exemption, and it would swallow exactly the diffs the gate exists to catch.
  mkdir -p "$TD/rep/results"
  printf 'path,class,regenerated_by\nE18-memory.csv,toolchain-scoped,cmd\nE18-counts.csv,byte-deterministic,cmd\n' \
    > "$TD/rep/results/MANIFEST.csv"
  echo counts > "$TD/rep/results/E18-counts.csv"
  echo bytes  > "$TD/rep/results/E18-memory.csv"
  ( cd "$TD/rep" && git add results && git commit -qm results ) >/dev/null 2>&1
  echo "bytes changed" > "$TD/rep/results/E18-memory.csv"
  expect_accept  "a modified results file the manifest classes toolchain-scoped" \
    check_clean_repo "$TD/rep"
  echo "counts changed" > "$TD/rep/results/E18-counts.csv"
  expect_refusal "a modified results file the manifest classes byte-deterministic" \
    check_clean_repo "$TD/rep"
  ( cd "$TD/rep" && git checkout -q -- . )

  # **The arm label, against the fault it is for.** In --merge-arms the two arms are one
  # binary, so nothing but this line can tell them apart; a check that accepted a log without
  # it would let both arms be labelled by the flag they were given.
  printf '  merge at 4r2w: 10 merged of 12 late landing(s) (83.3%%); rows visited 40, epochs merged 22; refused 1 over-epochs, 1 over-rows, 0 unavailable; caps 32 epochs / 4096 rows\n' > "$TD/merge-on.log"
  printf '  merge at 4r2w: 0 merged of 12 late landing(s); rows visited 0, epochs merged 0; refused 0 over-epochs, 0 over-rows, 0 unavailable; caps 0 epochs / 0 rows  [PINNED CONTROL: merging off]\n' > "$TD/merge-off.log"
  printf '  flights at 4r2w: pending_joins 0\n' > "$TD/merge-absent.log"
  expect_accept  "a log whose merge line names the caps it was launched with" \
    check_merge_line "$TD/merge-on.log" "32:4096"
  expect_accept  "the pinned control's log, naming merging off" \
    check_merge_line "$TD/merge-off.log" "0:0"
  expect_refusal "a log whose merge caps are not the ones the arm asked for" \
    check_merge_line "$TD/merge-on.log" "0:0"
  expect_refusal "a log from a build that does not report the merge at all" \
    check_merge_line "$TD/merge-absent.log" "32:4096"
  # **In every mode (F-11-20).** The candidate of a two-commit run launched at `off` must show
  # it; a baseline that predates the counter is accepted only when nothing was asked of it.
  expect_accept  "a pre-merge build launched at default, outside --merge-arms" \
    check_arm_caps "$TD/merge-absent.log" default 0
  expect_refusal "a pre-merge build launched at off, outside --merge-arms" \
    check_arm_caps "$TD/merge-absent.log" off 0
  expect_refusal "a build reporting merging on, launched at off, outside --merge-arms" \
    check_arm_caps "$TD/merge-on.log" off 0
  expect_accept  "a build reporting merging off, launched at off, outside --merge-arms" \
    check_arm_caps "$TD/merge-off.log" off 0
  expect_refusal "a pre-merge build under --merge-arms" \
    check_arm_caps "$TD/merge-absent.log" default 1

  # ---------------------------------------------------------------------------------------
  # **The machine-readable line, and the four things a field can be (C11-06.2).**
  #
  # Four fixture logs, one per outcome, and the arms that must tell them apart. The reason
  # each outcome is separate is a wrong number this project printed: a renamed column read as
  # `unwrap_or(0)` (missing read as zero), a counter reading that failed and rendered as
  # zeroes (`n/a` read as zero), `deferred_merges = 0` which is the finding rather than the
  # absence of one (zero read as `n/a`), and a log cut off mid-write (malformed read as a
  # value).
  MIXHDR='target,run,readers,writers,reads,writes,wall_ms,reads_per_second,writes_per_second,read_p50_us,read_p99_us,write_p50_us,write_p99_us,fallback_rate,max_batch,lock_wait_p99_us,base_epochs,duplicates,errors,not_run'
  mixline() {
    printf 'NLSBENCH/1 kind=mixed target=nilestream run=1 readers=6 writers=3 reads=100 writes=%s wall_ms=1000.000 reads_per_second=100.0 writes_per_second=%s read_p50_us=1.0 read_p99_us=2.0 write_p50_us=3.0 write_p99_us=4.0 fallback_rate=0.0000 max_batch=1 lock_wait_p99_us=0 base_epochs=10 duplicates=0 errors=0 not_run=-\n' "$1" "$2"
  }
  flightline() {
    printf 'NLSBENCH/1 kind=flights shape=6r/3w pending_joins=%s uninstalled_folds=%s pinned_installs=0 flights_refused=0 deferred_merges=0 waiters_refused=0 joins_answered=0 joins_retried=0 gap_at_begin_total=0 gap_at_finish_total=0 gap_at_finish_max_level=0 flights_behind_at_begin=0 flights_that_fell_behind=0 gap_begin_samples=0 gap_finish_samples=0 merge_rows_visited=0 merge_epochs_merged=0 merges_refused_epochs=0 merges_refused_rows=0 merges_refused_unavailable=0 caps_epochs=%s caps_rows=%s late_landings=0 joins_ended=0 folds_from_counters=0\n' "$1" "$1" "$2" "$3"
  }

  # (1) zero: a measured zero writer rate, and a measured zero counter.
  { mixline 0 0.0; flightline 0 0 0; } > "$TD/sum-zero.log"
  # (2) n/a: the counters were asked for and not obtained.
  { mixline 900 900.0; flightline 'n/a' 0 0; } > "$TD/sum-na.log"
  # (3) missing: a build whose flights line does not carry the field at all.
  { mixline 900 900.0
    flightline 0 0 0 | sed 's/ pending_joins=0//'
  } > "$TD/sum-missing.log"
  # (4) malformed: a log cut off mid-token.
  { mixline 900 900.0 | sed 's/writes_per_second=900.0/writes_per_second=/'
    flightline 0 0 0
  } > "$TD/sum-malformed.log"
  # (0) the clean control every refusal above must not fire on.
  { mixline 900 900.0; flightline 0 0 0; } > "$TD/sum-good.log"

  expect_accept  "a structured transcript whose writers moved" \
    check_writer_progress "$TD/sum-good.log"
  expect_refusal "a structured transcript whose writer rate is a measured zero" \
    check_writer_progress "$TD/sum-zero.log"
  expect_refusal "a structured transcript whose writer rate is a bare key" \
    check_writer_progress "$TD/sum-malformed.log"
  # **The substring bug, as a control.** `900` contains `0`; the field-based reader cannot see
  # that and this arm is what says so, rather than a comment claiming it.
  expect_accept  "a writer rate of 900, which the old grep read as a zero" \
    check_writer_progress "$TD/sum-good.log"
  expect_accept  "a structured transcript whose counters are a measured zero" \
    check_no_missing_fields "$TD/sum-zero.log"
  expect_refusal "a structured transcript whose counters came back n/a" \
    check_no_missing_fields "$TD/sum-na.log"
  expect_refusal "a structured transcript missing a counter this harness scores" \
    check_no_missing_fields "$TD/sum-missing.log"
  expect_accept  "a structured pinned arm whose every level reports caps 0/0" \
    check_arm_caps "$TD/sum-good.log" off 0
  expect_refusal "a structured pinned arm launched at default" \
    check_arm_caps "$TD/sum-good.log" default 0
  # **Every level, not the first one.** A run whose first level agreed and whose second did
  # not passed the old single-line check; two levels in one log is the arm that catches it.
  { flightline 0 0 0; flightline 0 32 4096; } > "$TD/sum-caps-drift.log"
  expect_refusal "an arm whose caps change between levels of one replicate" \
    check_arm_caps "$TD/sum-caps-drift.log" off 0

  # ---------------------------------------------------------------------------------------
  # **A stage whose prerequisite failed never launches its executable (C11-06.3).**
  #
  # The behavioural arm, not the structural one: a stale binary is put where a previous run
  # would have left it, and it is written to create a sentinel file if it is ever executed.
  # No hash is recorded for its arm, because its build never ran. The assertion is on the
  # sentinel's absence — a check that only inspected the return code would pass against a
  # guard that refused *after* launching.
  OUT_SAVED="$OUT"
  OUT="$TD/guard-out"; mkdir -p "$OUT/wt-stale/target/release"
  printf '#!/bin/sh\ntouch "%s/LAUNCHED"\n' "$TD" > "$OUT/wt-stale/target/release/bench"
  chmod +x "$OUT/wt-stale/target/release/bench"
  rm -f "$TD/LAUNCHED"
  # The launch is written the way `replicate` writes it, so that removing the guard from
  # `may_launch` really does run the binary here and the sentinel really does appear. An arm
  # that only inspected the return code would pass against a guard that refused *after*
  # launching, and against no guard at all if the caller happened not to launch.
  if may_launch stale "$OUT/wt-stale" bench >/dev/null 2>&1; then
    "$OUT/wt-stale/target/release/bench" >/dev/null 2>&1 || true
  fi
  if [ -e "$TD/LAUNCHED" ]; then
    note "  NOT REFUSED: the stale executable was launched — the sentinel $TD/LAUNCHED exists"
    st_fail=1
  else
    note "  refused    : a stale executable whose arm never built, and it was not launched"
  fi
  # The clean control: an arm whose build recorded this very binary's hash may launch it.
  record_exe stale "$OUT/wt-stale"
  expect_accept "an executable whose hash this run recorded" \
    may_launch stale "$OUT/wt-stale" bench
  # And the tampered case: the same arm, a different binary on disk.
  printf '#!/bin/sh\nexit 0\n' > "$OUT/wt-stale/target/release/bench"
  expect_refusal "an executable replaced after its hash was recorded" \
    may_launch stale "$OUT/wt-stale" bench
  OUT="$OUT_SAVED"

  # **A build that was refused must stop the run, not annotate it.**
  #
  # Both halves, because either alone is weak. The behavioural half: `fatal` really exits, and
  # says so in words a reader will see above twenty misleading refusals. The structural half:
  # no build failure anywhere in this script is still wired to `skip`, which is the form the
  # defect took — the failure was recorded and the sweep ran anyway, against whatever binary
  # was already in the refused worktree. A test of the behaviour would pass on the next
  # `|| skip` somebody adds.
  out="$( ( fatal "a test arm" ) 2>&1 )"; rc=$?
  if [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q "STOPPING"; then
    note "  refused    : a fatal section stops the run rather than annotating it"
  else
    note "  NOT REFUSED: \`fatal\` returned $rc without stopping — a refused build would be"
    note "               followed by a full sweep against the binary it refused"
    st_fail=1
  fi
  if grep -n "build_arm .* || *skip" "$0" >/dev/null 2>&1; then
    note "  NOT REFUSED: a build failure is still wired to \`skip\`, which records and returns:"
    grep -n "build_arm .* || *skip" "$0" | sed 's/^/               /'
    st_fail=1
  else
    note "  refused    : every build failure in this script is fatal, not recorded"
  fi

  # **One toolchain probe.** Three scripts each carried a copy in cycle 10 and one was repaired
  # twice while the other two kept the defect (F-11-01). Any script beside this one that defines
  # its own `pick_toolchain` is the defect coming back.
  dup="$(grep -l '^pick_toolchain()' "$HERE"/*.sh 2>/dev/null | grep -v '/toolchain.sh$' || true)"
  if [ -z "$dup" ]; then
    note "  refused    : no cycle-11 script carries its own toolchain probe"
  else
    note "  NOT REFUSED: a script defines its own toolchain probe: $dup"
    st_fail=1
  fi

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
    perl -e 'use IO::Socket::INET; my $s = IO::Socket::INET->new(LocalAddr=>"127.0.0.1", LocalPort=>$ARGV[0], Listen=>16, ReuseAddr=>1) or exit 1; sleep 60;' "$PORT" &
    LISTENER_PID=$!
  elif command -v python3 >/dev/null 2>&1; then
    python3 -c 'import socket,sys,time
s=socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(sys.argv[1]))); s.listen(16); time.sleep(60)' "$PORT" &
    LISTENER_PID=$!
  fi
  if [ -n "$LISTENER_PID" ]; then
    sleep 1
    # **Exactly one connection is made, and it is the one under test.**
    #
    # This first probed the port itself and *then* called `check_port_free`, which is two
    # connections. The listener never calls `accept`, so the first one sits in the backlog —
    # and on Darwin a connect to a socket whose backlog is full is refused, so the second
    # connect failed and `check_port_free` reported the port free. Host C's self-test caught
    # it as `NOT REFUSED: an occupied port` while the Linux container passed, which is the
    # portability difference this whole self-test exists to surface. The backlog is 16 now,
    # and the readiness check and the assertion are the same single call.
    if check_port_free "$PORT" >/dev/null 2>&1; then
      note "  not run    : an occupied port (the listener did not come up on $PORT, so the"
      note "               check was never given the fault it is for)"
      st_fail=1
    else
      note "  refused    : an occupied port"
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

  head2 "self-test: the build phase reads nothing the run phase sets (F-11-13)"
  # Static: every `$ARM_…` reference must come after the line that first assigns ARM_B_WT.
  # The first container run of this script died in build_arm on `ARM_B_TC: unbound variable`
  # — a per-arm toolchain read before the run phase had named the arms. `set -u` makes such
  # a slip fatal at the first real build, which is one full build too late; this arm makes it
  # fatal here.
  build_line="$(grep -n '^build_arm() {' "$0" | head -1 | cut -d: -f1)"
  first_arm_line="$(grep -n '^# RUN PHASE: the arms are named from here on' "$0" | head -1 | cut -d: -f1)"
  if [ -z "$build_line" ] || [ -z "$first_arm_line" ] || [ "$build_line" -ge "$first_arm_line" ]; then
    note "  NOT REFUSED: could not locate build_arm and the run-phase marker in order"
    st_fail=1
  else
    early_refs="$(sed -n "${build_line},$((first_arm_line - 1))p" "$0" | grep -v '^ *#' | grep -c '\$ARM_[A-Z_]*' || true)"
    if [ "${early_refs:-0}" -ne 0 ]; then
      note "  NOT REFUSED: $early_refs \$ARM_* reference(s) between build_arm (line $build_line) and the run phase (line $first_arm_line)"
      st_fail=1
    else
      note "  accepted   : no \$ARM_* reference between build_arm (line $build_line) and the run phase (line $first_arm_line)"
    fi
  fi

  head2 "self-test: c11-pairs.sh holds one variable per pair (F-11-20)"
  pairs="$(dirname "$0")/c11-pairs.sh"
  if [ ! -f "$pairs" ]; then
    note "  NOT REFUSED: c11-pairs.sh is missing beside this script"; st_fail=1
  else
    outs="$(grep -o -- '--out "\$OUT_ROOT/pair[0-9]"' "$pairs" | sort -u | wc -l | tr -d ' ')"
    p1="$(sed -n '/PAIR 1/,/rc1=/p' "$pairs")"
    if [ "$outs" -ne 2 ]; then
      note "  NOT REFUSED: c11-pairs.sh does not give its two pairs two output directories"; st_fail=1
    else
      note "  accepted   : the two pairs write to two directories"
    fi
    if printf '%s' "$p1" | grep -q -- '--baseline-caps off' && printf '%s' "$p1" | grep -q -- '--candidate-caps off'; then
      note "  accepted   : pair 1 launches BOTH arms with merging off"
    else
      note "  NOT REFUSED: pair 1 does not launch both arms at merging off"; st_fail=1
    fi
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

if ! pick_toolchain "$REPO"; then
  note "$TOOLCHAIN_HOW"
  note ""
  note 'Nothing is built or measured without a named compiler. `stable` is not assumed here:'
  note 'on Host C `stable` is 1.97.1 and the tree pins 1.95.0, and a run that silently took'
  note "the first of those is MF-7 — the confound that cost cycle 10 the comparability of its"
  note "whole baseline."
  exit 3
fi

head2 "0. preflight"
note "date            : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
note "uname           : $(uname -a)"
note "repo            : $REPO"
note "out             : $OUT"

if ! git -C "$REPO" rev-parse --git-dir >/dev/null 2>&1; then
  note "FATAL: $REPO is not a git checkout (a linked worktree's .git is a file, so -d is the wrong test; F-11-12)."; exit 3
fi
if ! command -v cargo >/dev/null 2>&1; then
  note "FATAL: no cargo on PATH."; exit 3
fi

check_clean_repo "$REPO" || exit 3
report_untracked "$REPO"

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
note "rustc           : $TOOLCHAIN_USED"
note "toolchain       : $TOOLCHAIN_HOW"
note "pin declared    : $TOOLCHAIN_PIN  (rust-toolchain.toml)"
note "cargo           : $(cargo --version 2>&1)"
note "governor        : $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'n/a')"
note "nproc           : $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo '?')"
note "levels          : $LEVELS readers (writers = ceil(readers/2): 6r3w, 9r5w, 12r6w)"
note "working points  : full  = $ACCOUNTS accounts / $BUDGET_FULL budget (budget above the key count)"
note "                : part  = $ACCOUNTS accounts / $BUDGET_PARTIAL budget (genuinely partial)"
note "per level       : ${SECONDS_PER_LEVEL}s mixed, $WARMUPS warm-up + $MEASURED measured per arm"
ARMS=1
if [ "$BASELINE_ONLY" -eq 0 ] || [ "$NEUTRAL" -eq 1 ] || [ "$MERGE_ARMS" -eq 1 ]; then ARMS=2; fi
REPLICATES_TOTAL=$(( (WARMUPS + MEASURED) * ARMS * 2 ))
note "replicate cap   : ${REPLICATE_TIMEOUT}s, after which the replicate is killed and counted as not run"
note "replicates      : $REPLICATES_TOTAL total = ($WARMUPS warm-up + $MEASURED measured) x $ARMS arm(s) x 2 working points"
note "                : each is 3 levels of ${SECONDS_PER_LEVEL}s mixed plus this machine's fixed"
note "                : point/fold/durable phases. The first \`replicate elapsed\` line below"
note "                : times one; multiply by $REPLICATES_TOTAL for the run. Two working"
note "                : points is what doubles it, and it is the half cycle 9 never measured."
if [ "$MERGE_ARMS" -eq 1 ]; then
  note "arms            : one build, two settings — merge (caps 32 epochs / 4096 rows) against"
  note "                : the pinned control (merging off). A second worktree would add a"
  note "                : confound and buy nothing: the change under test is a value the"
  note "                : daemon reads at start-up, not a difference in the source."
  note "noise floor     : the A=A control from the --baseline-only run on this host. It is not"
  note "                : re-measured here; the gate below is scored against it."
else
  note "neutral A=A     : $( [ "$NEUTRAL" -eq 1 ] && echo 'yes, before anything is scored' || echo 'SKIPPED by --no-neutral' )"
fi

mkdir -p "$OUT" || { note "FATAL: cannot create $OUT"; exit 3; }

# ---------------------------------------------------------------------------------------------
# The run manifest (C11-06.3). Written before anything is built, so that a run which dies
# halfway still leaves a file saying what it was and how far it got.
# ---------------------------------------------------------------------------------------------
: > "$OUT/manifest.csv"
printf 'stage,outcome,detail\n' >> "$OUT/manifest.csv"
{
  printf 'run                 : %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'invocation          : %s\n' "$INVOCATION"
  printf 'host                : %s\n' "$(uname -a)"
  printf 'repo                : %s\n' "$REPO"
  printf 'repo HEAD           : %s\n' "$(git -C "$REPO" rev-parse HEAD 2>/dev/null)"
  printf 'repo branch         : %s\n' "$(git -C "$REPO" rev-parse --abbrev-ref HEAD 2>/dev/null)"
  # **The diff, not just the SHA.** A dirty tree measured at a SHA is not that SHA, and a
  # hash of the diff is what lets a later reader ask whether two runs had the same
  # uncommitted change without needing the change itself.
  printf 'tracked-diff sha256 : %s\n' "$(git -C "$REPO" diff HEAD 2>/dev/null | sha256_of /dev/stdin)"
  printf 'baseline            : %s -> %s\n' "$BASELINE_REF" "$BASELINE_SHA"
  [ "$BASELINE_ONLY" -eq 0 ] && printf 'candidate           : %s -> %s\n' "$CANDIDATE_REF" "$CANDIDATE_SHA"
  printf 'toolchain pin       : %s\n' "$TOOLCHAIN_PIN"
  printf 'toolchain selector  : %s\n' "$TOOLCHAIN_SELECTOR"
  printf 'toolchain used      : %s\n' "$TOOLCHAIN_USED"
  printf 'toolchain how       : %s\n' "$TOOLCHAIN_HOW"
  printf 'RUSTUP_TOOLCHAIN in : %s\n' "$TOOLCHAIN_INHERITED"
  printf 'C11_TOOLCHAIN       : %s\n' "${C11_TOOLCHAIN:--}"
  printf 'CARGO_NET_OFFLINE   : %s\n' "${CARGO_NET_OFFLINE:--}"
  printf 'arm caps            : baseline=%s candidate=%s merge-arms=%s\n' "$BASELINE_CAPS" "$CANDIDATE_CAPS" "$MERGE_ARMS"
  printf 'arm toolchains      : baseline=%s candidate=%s\n' "${BASELINE_TOOLCHAIN:--}" "${CANDIDATE_TOOLCHAIN:--}"
  printf 'limits              : warmups=%s measured=%s levels=%s seconds=%s accounts=%s rounds=%s\n' \
    "$WARMUPS" "$MEASURED" "$LEVELS" "$SECONDS_PER_LEVEL" "$ACCOUNTS" "$ROUNDS"
  printf 'budgets             : full=%s partial=%s\n' "$BUDGET_FULL" "$BUDGET_PARTIAL"
  printf 'replicate cap       : %ss\n' "$REPLICATE_TIMEOUT"
  printf 'port base           : %s\n' "$PORT_BASE"
} > "$OUT/manifest.txt"
note "manifest        : $OUT/manifest.txt (and manifest.csv, one row per stage)"

head2 "0b. the bounded deadlock witness"
note "A10-01: a stats snapshot that held the view while it waited for the base, against an"
note "append that holds the base while it waits for the view. The mixed sweep below queries"
note "that table between levels, so this is run FIRST: a baseline that can hang is not a"
note "baseline, and a hang here means the run is blocked, not slow."
DW_LOG="$OUT/deadlock-witness.log"
( cd "$REPO" && CARGO_NET_OFFLINE=true \
    cargo test --offline -p nilestream-server --lib -- \
      a_stats_snapshot_and_a_concurrent_append_both_finish \
      the_base_is_acquired_before_the_view_on_every_path_that_takes_both \
      the_read_path_takes_the_base_shared_and_the_append_takes_it_exclusively ) \
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
    # **Prune first.** A previous run's output directory deleted by hand leaves the worktree
    # still registered in the repository, and `git worktree add` then refuses a path it
    # considers taken — which this reported as "could not create the worktree", a message
    # that names the symptom and hides the cause. Pruning removes exactly the registrations
    # whose directories are gone and touches nothing that exists.
    git -C "$REPO" worktree prune >/dev/null 2>&1
    if ! add_err="$(git -C "$REPO" worktree add --detach "$wt" "$sha" 2>&1)"; then
      note "  could not create the worktree for $arm at $sha; git said:"
      printf '%s\n' "$add_err" | sed 's/^/    /'
      return 1
    fi
  fi
  tc=""
  # The flag variables, not ARM_B_TC/ARM_C_TC: those are set after the build phase, and the
  # first container run of this script died here with `ARM_B_TC: unbound variable` (F-11-13).
  case "$arm" in baseline) tc="$BASELINE_TOOLCHAIN" ;; candidate) tc="$CANDIDATE_TOOLCHAIN" ;; esac
  # **The toolchain that built this arm, recorded from inside its worktree** (MF-7, MF-14): an
  # override applies to this arm only, and the line below is what the transcript reports.
  ( cd "$wt" && if [ -n "$tc" ]; then export RUSTUP_TOOLCHAIN="$tc"; fi && rustc -vV 2>&1 | sed -n 's/^release: //p' ) > "$OUT/toolchain-$arm.txt" 2>&1
  note "  $arm toolchain: $(cat "$OUT/toolchain-$arm.txt")$( [ -n "$tc" ] && printf ' (override %s)' "$tc" )"
  ( cd "$wt" && if [ -n "$tc" ]; then export RUSTUP_TOOLCHAIN="$tc"; fi && CARGO_NET_OFFLINE=true \
      cargo build --offline --release -p nilestream-server -p bank-bench >"$OUT/build-$arm.log" 2>&1 ) || {
    note "  build failed for $arm; see $OUT/build-$arm.log"
    stage "build-$arm" failed "see $OUT/build-$arm.log"
    return 1; }
  built="$(git -C "$wt" rev-parse HEAD)"
  if [ "$built" != "$sha" ]; then
    note "  the $arm worktree is at $built and this run is about $sha"
    stage "build-$arm" failed "worktree at $built, expected $sha"
    return 1
  fi
  # **The hash is recorded only here**, after the build succeeded and the commit was verified.
  # Every launch is checked against it, so an arm that never reached this line cannot be run.
  record_exe "$arm" "$wt"
  note "  $arm built at $built"
  note "  $arm bench sha256: $(cut -c1-16 < "$OUT/exe-$arm-bench.sha")"
  stage "build-$arm" ok "$built $(cut -c1-16 < "$OUT/exe-$arm-bench.sha") $(cat "$OUT/toolchain-$arm.txt")"
  return 0
}

head2 "1. build"
BASELINE_WT="$OUT/wt-baseline"
build_arm baseline "$BASELINE_SHA" || fatal "the baseline arm did not build"
if [ "$BASELINE_ONLY" -eq 0 ]; then
  CANDIDATE_WT="$OUT/wt-candidate"
  build_arm candidate "$CANDIDATE_SHA" || fatal "the candidate arm did not build"
elif [ "$NEUTRAL" -eq 1 ]; then
  # The neutral arm is the same commit in a second worktree, built separately. Same source,
  # separate build directory, separate binary: if those alone move the number, the harness
  # cannot resolve anything smaller.
  CONTROL_WT="$OUT/wt-control"
  build_arm control "$BASELINE_SHA" || fatal "the A=A control arm did not build"
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
  arm="$1"; wt="$2"; rep="$3"; tag="$4"; point="$5"; budget="$6"; caps="${7:-default}"
  port=$(( PORT_BASE + 2 * (rep_seq % 40) ))
  outdir="$OUT/results-$arm-$point-$rep"
  rm -rf "$outdir"; mkdir -p "$outdir"
  log="$OUT/bench-$arm-$point-$rep.log"

  may_launch "$arm" "$wt" bench || { note "  [$tag] not launched"; stage "replicate-$tag" "not run" "the launch guard refused"; return 1; }
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
    # **The arm, and the only thing that differs between the two of them.** One binary, two
    # values: a difference between the arms therefore cannot be a difference between two
    # compilations, which is the confound a second worktree would reintroduce.
    CARGO_NET_OFFLINE=true NILESTREAM_MERGE_CAPS="$caps" \
      "$wt/target/release/bench" \
        --run --nls-only --scaling-only --host-nls --nls-port "$port" \
        --connections "$LEVELS" --mixed-seconds "$SECONDS_PER_LEVEL" \
        --accounts "$ACCOUNTS" --rounds "$ROUNDS" --nls-budget "$budget" \
        --runs 1 --out "$outdir" >"$log" 2>&1
  }
  rep_began=$(date +%s)
  run_with_deadline "$REPLICATE_TIMEOUT" run_one
  rc=$?
  rep_took=$(( $(date +%s) - rep_began ))

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
  check_arm_caps "$log" "$caps" "$MERGE_ARMS" || { note "  [$tag] see $log"; return 1; }

  # **How long this replicate took**, so the reader can multiply rather than guess. There are
  # REPLICATES_TOTAL of them and the preflight cannot know the fixed per-replicate cost of
  # this machine; the first line of this kind tells you what the whole run will cost.
  note "  [$tag] replicate elapsed: ${rep_took}s"
  # The whole mixed section, not a grep of it: the slowest-16 table and the lock histogram
  # are rows of numbers with no keyword in them, and a filter that drops them keeps only the
  # sentence saying a table exists.
  sed -n '/E19 mixed/,$p' "$log" | grep -v "connected as\|disconnected after" | sed 's/^/    /'
  return 0
}

# Both working points, one after the other, for one arm and one replicate index.
replicate_both_points() {
  arm="$1"; wt="$2"; rep="$3"; label="$4"; caps="${5:-default}"
  ok=0
  note "  [$label] point: full ($ACCOUNTS accounts / $BUDGET_FULL budget)"
  rep_seq=$(( rep_seq + 1 ))
  replicate "$arm" "$wt" "$rep" "$label full" full "$BUDGET_FULL" "$caps" && ok=$(( ok + 1 ))
  note "  [$label] point: partial ($ACCOUNTS accounts / $BUDGET_PARTIAL budget)"
  rep_seq=$(( rep_seq + 1 ))
  replicate "$arm" "$wt" "$rep" "$label partial" partial "$BUDGET_PARTIAL" "$caps" && ok=$(( ok + 1 ))
  [ "$ok" -eq 2 ]
}

# RUN PHASE: the arms are named from here on (the self-test arm for F-11-13 keys on this line)
if [ "$BASELINE_ONLY" -eq 0 ]; then
  ARM_B_WT="$BASELINE_WT"; ARM_C_WT="${CANDIDATE_WT:-}"
  ARM_B="baseline";        ARM_C="candidate"
  ARM_B_LABEL="$BASELINE_LABEL"; ARM_C_LABEL="$CANDIDATE_LABEL"
else
  ARM_B_WT="$BASELINE_WT"; ARM_C_WT="${CONTROL_WT:-}"
  ARM_B="baseline";        ARM_C="control"
  ARM_B_LABEL="$BASELINE_LABEL"; ARM_C_LABEL="A=A control (same commit, second build)"
fi
ARM_B_CAPS="$BASELINE_CAPS"
ARM_C_CAPS="$CANDIDATE_CAPS"
if [ "$BASELINE_CAPS" != "default" ]; then
  BASELINE_LABEL="$BASELINE_LABEL (NILESTREAM_MERGE_CAPS=$BASELINE_CAPS)"
  ARM_B_LABEL="$BASELINE_LABEL"
fi
if [ "$BASELINE_ONLY" -eq 0 ] && [ "$CANDIDATE_CAPS" != "default" ]; then
  CANDIDATE_LABEL="$CANDIDATE_LABEL (NILESTREAM_MERGE_CAPS=$CANDIDATE_CAPS)"
  ARM_C_LABEL="$CANDIDATE_LABEL"
fi
TWO_ARMS=1
if [ "$BASELINE_ONLY" -eq 1 ] && [ "$NEUTRAL" -eq 0 ]; then TWO_ARMS=0; fi

# **T04.2: the two arms are one build and two settings.**
#
# Everything above this point is about comparing two *commits*, which needs two worktrees
# because the binaries differ. The merge ablation is the other shape: the change under test is
# a value the daemon reads at start-up, so a second build would add a confound — two
# compilations of identical source that can differ by layout alone — in exchange for nothing.
# One worktree, one binary, two values of NILESTREAM_MERGE_CAPS, and every replicate's
# transcript is checked to carry the caps the daemon actually ran with.
if [ "$MERGE_ARMS" -eq 1 ]; then
  ARM_B_WT="$BASELINE_WT"; ARM_C_WT="$BASELINE_WT"
  ARM_B="merge";           ARM_C="pinned"
  ARM_B_CAPS=default;      ARM_C_CAPS=off
  ARM_B_LABEL="deferred merge on (caps 32 epochs / 4096 rows, preregistered)"
  ARM_C_LABEL="pinned control (merging off) — the policy this cycle measures against"
  TWO_ARMS=1
fi

head2 "3. warm-ups (discarded, but not ignored)"
note "A warm-up that did not complete is a cold arm measured as a warm one, so its exit is"
note "checked. The numbers are still thrown away."
i=0
warm_bad=0
while [ "$i" -lt "$WARMUPS" ]; do
  i=$(( i + 1 ))
  note "warm-up $i / $WARMUPS"
  if ! replicate_both_points "$ARM_B" "$ARM_B_WT" "w$i" "$ARM_B warm-up $i" "$ARM_B_CAPS" >/dev/null 2>&1; then
    note "  the $ARM_B warm-up $i did not complete"
    warm_bad=1
  fi
  if [ "$TWO_ARMS" -eq 1 ]; then
    if ! replicate_both_points "$ARM_C" "$ARM_C_WT" "w$i" "$ARM_C warm-up $i" "$ARM_C_CAPS" >/dev/null 2>&1; then
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
    replicate_both_points "$ARM_B" "$ARM_B_WT" "$i" "$ARM_B $i" "$ARM_B_CAPS" && ok_b=$(( ok_b + 1 ))
    if [ "$TWO_ARMS" -eq 1 ]; then
      note "--- $ARM_C_LABEL ---"
      replicate_both_points "$ARM_C" "$ARM_C_WT" "$i" "$ARM_C $i" "$ARM_C_CAPS" && ok_c=$(( ok_c + 1 ))
    fi
  else
    note "--- replicate $i / $MEASURED : order BA ---"
    if [ "$TWO_ARMS" -eq 1 ]; then
      note "--- $ARM_C_LABEL ---"
      replicate_both_points "$ARM_C" "$ARM_C_WT" "$i" "$ARM_C $i" "$ARM_C_CAPS" && ok_c=$(( ok_c + 1 ))
    fi
    note "--- $ARM_B_LABEL ---"
    replicate_both_points "$ARM_B" "$ARM_B_WT" "$i" "$ARM_B $i" "$ARM_B_CAPS" && ok_b=$(( ok_b + 1 ))
  fi
done

head2 "5. the scored table — computed here, not by the reader"
# **The arithmetic belongs to the program that owns the schema (F-11-02, C11-06.1).**
#
# The previous version of this section listed the statistics a reader was expected to compute
# from the transcript. Two auditors did exactly that, from the same cycle-10 logs, and
# disagreed: six pooled MADs were published as the mean of the two arms' MADs where the pooled
# MAD is their RMS — the same slip A10-09 had already been. `bench --score` reads the
# per-replicate CSVs this run just wrote and prints the table; this script prints its output
# and computes nothing of its own.
SCORER=""
for cand in "${BASELINE_WT:-}/target/release/bench" "${CANDIDATE_WT:-}/target/release/bench" "${CONTROL_WT:-}/target/release/bench"; do
  [ -x "$cand" ] && { SCORER="$cand"; break; }
done
if [ -z "$SCORER" ]; then
  skip "no built \`bench\` was available to score this run; the CSVs are in $OUT and \`bench --score $OUT\` will score them"
  stage score "not run" "no scorer binary"
elif "$SCORER" --score "$OUT" > "$OUT/score.txt" 2>&1; then
  cat "$OUT/score.txt"
  stage score ok "$OUT/score.txt"
else
  note "the scorer refused this run:"
  sed 's/^/  /' "$OUT/score.txt"
  skip "bench --score refused to score this run; see $OUT/score.txt"
  stage score failed "$OUT/score.txt"
fi

head2 "5b. what to paste back"
note "Paste this whole transcript. Beside the table above, the reader wants, per arm, per working point and per level:"
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
if [ "$BASELINE_ONLY" -eq 1 ] && [ "$NEUTRAL" -eq 0 ] && [ "$MERGE_ARMS" -eq 0 ]; then
  skip "the neutral A=A control was skipped by --no-neutral; nothing here establishes a noise floor"
fi
if [ "$MERGE_ARMS" -eq 1 ]; then
  note ""
  note "The noise floor this comparison is scored against was measured by the --baseline-only"
  note "run on this host and is NOT re-measured here: worst separation 0.83%, about one pooled"
  note "MAD, against a gate of >=10% relative AND >=3 pooled MADs. If the difference above does"
  note "not clear both, the merge makes no performance claim. That is a result and it is the"
  note "one to report; the correctness targets of T04.1 stand on their own and do not depend"
  note "on it."
fi

summary_and_exit
