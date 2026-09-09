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
MERGE_ARMS=0
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
# compiler: every gate and the whole cycle-10 baseline were built with a toolchain the tree
# does not specify, and nothing said so.
#
# The rule now: use the pin if it resolves, fall back to `stable` if it does not, and print
# which happened. A measurement built with a compiler other than the one the tree pins is not
# wrong, but it is a different measurement and it must be labelled.
pick_toolchain() {
  # **Refusal before probing.** Without this, asking `rustc --version` under an unresolvable
  # pin makes rustup try to *download* the toolchain — which is a network access at execution
  # time, and this cycle's standing rule is that no task requires one.
  export RUSTUP_AUTO_INSTALL=0
  # **Probe the pin, not whatever the caller already exported.** This asked `rustc --version`
  # with the inherited environment and then, on success, concluded that the *tree's pin*
  # resolves and unset the override. A shell that already had `RUSTUP_TOOLCHAIN=stable` set —
  # which is how the executor's container has to build at all, the pinned 1.95.0 being
  # unfetchable without egress — therefore reported "the tree's pin resolves on this host",
  # removed the very override that made it true, and every `cargo` below failed with
  # `toolchain 1.95.0 is not installed`. The probe has to measure the thing it is deciding
  # about, so the override is cleared before it rather than after.
  unset RUSTUP_TOOLCHAIN
  # **Probed from inside the repository, because that is where the pin lives.**
  #
  # `rust-toolchain.toml` is resolved relative to the working directory, and this script is
  # normally invoked by absolute path from wherever the author happens to be standing. Run
  # from `~`, the probe found no pin at all, resolved to the default channel, and reported
  # `rustc 1.97.1 ... the tree's pin, which resolves on this host` — a sentence in which
  # every clause is false. The builds below `cd` into their worktrees and so did use the
  # pin; only the line describing them was wrong, which is the worse of the two failures to
  # have, because it is the line a reader trusts.
  if ( cd "$REPO" 2>/dev/null && rustc --version ) >/dev/null 2>&1; then
    TOOLCHAIN_USED="$( cd "$REPO" && rustc --version 2>&1 )"
    TOOLCHAIN_HOW="the tree's pin, resolved in $REPO, which resolves on this host"
  else
    export RUSTUP_TOOLCHAIN=stable
    TOOLCHAIN_USED="$( cd "$REPO" 2>/dev/null && rustc --version 2>&1 )"
    TOOLCHAIN_HOW="RUSTUP_TOOLCHAIN=stable, because the tree's pin does not resolve here"
  fi
  # The pin as written, so a reader can compare it with what resolved rather than trust
  # this script's adjective.
  TOOLCHAIN_PIN="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$REPO/rust-toolchain.toml" 2>/dev/null)"
  TOOLCHAIN_PIN="${TOOLCHAIN_PIN:-none declared}"
}

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
    --merge-arms)    MERGE_ARMS=1; BASELINE_ONLY=1; NEUTRAL=0 ;;
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

pick_toolchain

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
  ( cd "$wt" && CARGO_NET_OFFLINE=true \
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
  if [ "$MERGE_ARMS" -eq 1 ]; then
    want="$caps"
    [ "$want" = "default" ] && want="32:4096"
    [ "$want" = "off" ] && want="0:0"
    check_merge_line "$log" "$want" || { note "  [$tag] see $log"; return 1; }
  fi

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

if [ "$BASELINE_ONLY" -eq 0 ]; then
  ARM_B_WT="$BASELINE_WT"; ARM_C_WT="${CANDIDATE_WT:-}"
  ARM_B="baseline";        ARM_C="candidate"
  ARM_B_LABEL="$BASELINE_LABEL"; ARM_C_LABEL="$CANDIDATE_LABEL"
else
  ARM_B_WT="$BASELINE_WT"; ARM_C_WT="${CONTROL_WT:-}"
  ARM_B="baseline";        ARM_C="control"
  ARM_B_LABEL="$BASELINE_LABEL"; ARM_C_LABEL="A=A control (same commit, second build)"
fi
ARM_B_CAPS=default
ARM_C_CAPS=default
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
