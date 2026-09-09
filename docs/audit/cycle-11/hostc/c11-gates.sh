#!/usr/bin/env bash
# c11-gates.sh — the correctness gates, on Host C, in the pairing the audit requires.
#
# WHAT THIS IS FOR
#   `c11-baselock.sh` measures. This one decides whether anything may be measured at all: the
#   two workspaces' test suites, the paired adapter gate that neither workspace can run
#   alone, the generated-document gates, and the memory budgets.
#
#   Three cycle-11 findings are about gates that were green while checking nothing:
#     * the paired adapter check printed SKIPPED and reported `ok` when no GBS checkout was
#       found, so the pair stayed broken for a whole cycle with both sides green;
#     * Appendix D's generator truncated each file at the first *occurrence of the characters*
#       `#[cfg(test)]` — including inside a doc comment — and understated the public API by
#       32 items, silently, in the direction that looks correct;
#     * the GBS tamper guard flipped `buf.len()/2` and called it a payload; it was a header.
#   So this script's own contract is that a section which did not run says so and the exit
#   code is non-zero.
#
# USAGE
#   bash c11-gates.sh                      both trees, at their defaults
#   bash c11-gates.sh --niles ~/n --gbs ~/g
#
#   NILES_NO_GBS is NOT honoured here and the script says so if it is set: it is an opt-out
#   for standalone development, and a paired audit gate is exactly what it opts out of.
#
# The author's zsh has `interactive_comments` off. Run this with `bash`, as written above.
#
# Expect 10-20 minutes, most of it two `cargo test --workspace` runs.

set -uo pipefail

NILES="${C11_NILES:-$HOME/Documents/niles}"
GBS="${C11_GBS:-$HOME/Documents/GBS}"

# **The commit each tree is expected to be at.** A gate run is evidence about a commit pair;
# without this the pair is whatever happened to be checked out, and a reader learns it from a
# line in the transcript that nothing checked. Empty means "do not check", which is the old
# behaviour and is reported as such.
NILES_SHA="${C11_NILES_SHA:-}"
GBS_SHA="${C11_GBS_SHA:-}"

# **The newer compiler, run separately.** A11-06's lint rows are red on 1.97.1 and green on
# the pin. Running both in one pass would make the gate's verdict depend on which compiler
# happened to be first; running the newer one as its own section, allowed to be red, keeps the
# pin's verdict clean and still surfaces the rows. Empty means the section is not run and says
# so.
LINT_TOOLCHAIN="${C11_LINT_TOOLCHAIN:-}"

SELF_TEST=0

FAILED=0
NOTRUN=""

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
# **Resolved once, at the top, before anything can change directory.** `$0` is relative when
# the script is invoked by a relative path, so `dirname "$0"` stops naming this directory the
# moment any code runs from somewhere else — which the self-test arm `run from a foreign
# working directory` demonstrated by failing.
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/toolchain.sh"

note()  { printf '%s\n' "$*"; }
hdr()   { printf '\n=== %s ===\n' "$*"; }
notrun(){ NOTRUN="${NOTRUN}
  - $*"; FAILED=1; note "NOT RUN: $*"; }
bad()   { FAILED=1; note "FAILED: $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --niles) NILES="${2:-}"; shift ;;
    --gbs)   GBS="${2:-}"; shift ;;
    --niles-sha) NILES_SHA="${2:-}"; shift ;;
    --gbs-sha)   GBS_SHA="${2:-}"; shift ;;
    --lint-toolchain) LINT_TOOLCHAIN="${2:-}"; shift ;;
    --self-test) SELF_TEST=1 ;;
    *) note "REFUSED: unknown flag \`$1\`. A mistyped flag that is ignored is a run that"
       note "         checked something other than what was asked for."
       exit 2 ;;
  esac
  shift
done

export CARGO_NET_OFFLINE=true
export GIT_TERMINAL_PROMPT=0

# ---------------------------------------------------------------------------------------------
# **One pin per repository, resolved in that repository (C11-06.4).**
#
# The first version of this script called `pick_toolchain` with no argument. The function read
# an ambient `REPO` that this script never sets, so under `set -u` both reads died and the
# preflight printed
#
#     rustc       :
#                 : RUSTUP_TOOLCHAIN=stable, because the pin (none declared) does not resolve here
#
# for a tree whose `rust-toolchain.toml` says 1.95.0 on its first line. The gate then built
# everything under whatever `stable` is on that machine and reported that the tree had asked
# for it — MF-7 again, one cycle after MF-7, inside the script written to prevent MF-7.
#
# Two repositories can pin two compilers, so each is resolved against its own tree and each
# section is launched with the selector its repository produced. Nothing inherits: `run_pinned`
# assigns `RUSTUP_TOOLCHAIN` in the child, so an operator's own default — Host C's is 1.97.1 —
# cannot reach a section that resolved 1.95.0.
resolve_repo() {
  _r="$1"; _tag="$2"
  if ! pick_toolchain "$_r"; then
    note "FATAL: $TOOLCHAIN_HOW"
    note ""
    note "This is a refusal of everything downstream, not a fallback. A gate that quietly"
    note "substituted a compiler would produce green rows about a build the tree never asked"
    note "for, which is the confound that cost cycle 10 its whole baseline."
    exit 3
  fi
  eval "${_tag}_PIN=\$TOOLCHAIN_PIN"
  eval "${_tag}_SEL=\$TOOLCHAIN_SELECTOR"
  eval "${_tag}_VER=\$TOOLCHAIN_USED"
  eval "${_tag}_HOW=\$TOOLCHAIN_HOW"
}

# Run a cargo (or make) command inside a repository under that repository's selector.
in_repo() {
  _r="$1"; _sel="$2"; shift 2
  ( cd "$_r" && RUSTUP_TOOLCHAIN="$_sel" "$@" )
}

# **The exact pair, when one was named.** A prefix is accepted because that is how a SHA is
# written in a work order; the comparison is on the prefix of the full SHA and never the other
# way round, so a four-character argument cannot match by accident of formatting.
check_sha() {
  _name="$1"; _repo="$2"; _want="$3"
  [ -z "$_want" ] && { note "$_name expected : (none given — this run is not pinned to a commit)"; return 0; }
  _have="$(git -C "$_repo" rev-parse HEAD)"
  case "$_have" in
    "$_want"*) note "$_name expected : $_want, and HEAD matches" ;;
    *) note "FATAL: $_name is at $_have and this run was told to expect $_want."
       note "       Every gate below would be evidence about a different commit."
       exit 3 ;;
  esac
}

# ---------------------------------------------------------------------------------------------
# --self-test: the preflight's own refusals, against the faults they are for (C11-06.4/.6).
# No cargo, no network, under a minute.
# ---------------------------------------------------------------------------------------------
if [ "$SELF_TEST" -eq 1 ]; then
  hdr "self-test: the toolchain and repository checks"
  st_fail=0
  ok_()   { note "  ok      : $*"; }
  bad_()  { note "  FAILED  : $*"; st_fail=1; }

  TD="$(mktemp -d "${TMPDIR:-/tmp}/c11-gates-selftest.XXXXXX")"
  trap 'rm -rf "$TD"' EXIT

  mkrepo() {
    mkdir -p "$1" && git -C "$1" init -q . 2>/dev/null
    git -C "$1" config user.email c11@example.invalid
    git -C "$1" config user.name c11
    printf 'x\n' > "$1/f.txt"
    git -C "$1" add f.txt >/dev/null 2>&1
    git -C "$1" commit -qm one >/dev/null 2>&1
  }

  # (1) **A repository with no pin, and no substitute named, is a refusal.** The whole defect
  #     this section exists for was a silent `stable`, so the arm asserts the *absence* of a
  #     selector rather than the presence of a message.
  mkrepo "$TD/nopin"
  ( unset C11_TOOLCHAIN; pick_toolchain "$TD/nopin" ) >/dev/null 2>&1
  rc=$?
  if [ "$rc" -eq 3 ]; then ok_ "a tree with no resolvable pin, and no substitute named, refuses"
  else bad_ "a tree with no resolvable pin returned $rc instead of 3 — this is the silent \`stable\`"; fi

  # (2) **A named substitute is accepted and says so.**
  if ( C11_TOOLCHAIN=stable pick_toolchain "$TD/nopin" >/dev/null 2>&1 ); then
    ( C11_TOOLCHAIN=stable pick_toolchain "$TD/nopin" >/dev/null 2>&1
      case "$TOOLCHAIN_HOW" in *C11_TOOLCHAIN=stable*) exit 0 ;; *) exit 1 ;; esac )
    if [ $? -eq 0 ]; then ok_ "a named substitute is used and the transcript says it was one"
    else bad_ "a named substitute was used and the transcript does not say so"; fi
  else
    bad_ "a named substitute was refused"
  fi

  # (3) **Called with no repository at all.** This is the bug that produced the fabricated
  #     'the pin (none declared) does not resolve here' line, and the arm is that the function
  #     returns 2 and sets no selector rather than reporting about a tree it never opened.
  ( unset REPO; pick_toolchain ) >/dev/null 2>&1
  rc=$?
  ( unset REPO; pick_toolchain >/dev/null 2>&1; [ -z "${TOOLCHAIN_SELECTOR:-}" ] )
  empty=$?
  if [ "$rc" -eq 2 ] && [ "$empty" -eq 0 ]; then
    ok_ "pick_toolchain with no repository refuses and selects nothing"
  else
    bad_ "pick_toolchain with no repository returned $rc and left a selector — it is reporting about a tree it never read"
  fi

  # (4) **An inherited RUSTUP_TOOLCHAIN cannot reach a child.** Host C's shell default is
  #     1.97.1; a section that resolved 1.95.0 must launch 1.95.0. The child prints what it
  #     was given, so the assertion is on the child's value and not on the parent's.
  TOOLCHAIN_SELECTOR=the-selected-one
  got="$(RUSTUP_TOOLCHAIN=1.97.1; export RUSTUP_TOOLCHAIN; run_pinned sh -c 'printf %s "$RUSTUP_TOOLCHAIN"')"
  if [ "$got" = "the-selected-one" ]; then ok_ "an inherited RUSTUP_TOOLCHAIN=1.97.1 does not reach the child"
  else bad_ "the child ran with \`$got\` — the caller's environment decided the compiler"; fi
  got="$(in_repo "$TD/nopin" the-repo-selector sh -c 'printf %s "$RUSTUP_TOOLCHAIN"')"
  if [ "$got" = "the-repo-selector" ]; then ok_ "in_repo launches under the repository's own selector"
  else bad_ "in_repo launched under \`$got\`"; fi
  unset TOOLCHAIN_SELECTOR

  # (5) **A path with spaces.** Every quoting mistake in this script shows up here and nowhere
  #     else, because no path on either host has a space in it today.
  mkrepo "$TD/a dir with spaces"
  if [ "$(git -C "$TD/a dir with spaces" rev-parse --is-inside-work-tree 2>/dev/null)" = "true" ] \
     && ( C11_TOOLCHAIN=stable pick_toolchain "$TD/a dir with spaces" >/dev/null 2>&1 ); then
    ok_ "a repository path containing spaces is read, not split"
  else
    bad_ "a repository path containing spaces was not handled"
  fi

  # (6) **A linked worktree**, whose `.git` is a file and not a directory. The old test was
  #     `[ -d "$REPO/.git" ]` and refused every worktree on the machine (F-11-12).
  git -C "$TD/nopin" worktree add -q --detach "$TD/wt" HEAD >/dev/null 2>&1
  if [ -f "$TD/wt/.git" ] && [ "$(git -C "$TD/wt" rev-parse --is-inside-work-tree 2>/dev/null)" = "true" ]; then
    ok_ "a linked worktree (its .git is a file) is accepted"
  else
    bad_ "a linked worktree was rejected, or the fixture did not build one"
  fi

  # (7) **A bare repository has a git dir and no working tree**, and every gate below is about
  #     files. `--git-dir` accepted it; `--is-inside-work-tree` does not.
  git init -q --bare "$TD/bare.git" 2>/dev/null
  if [ "$(git -C "$TD/bare.git" rev-parse --is-inside-work-tree 2>/dev/null)" = "true" ]; then
    bad_ "a bare repository was accepted as a working tree"
  else
    ok_ "a bare repository is refused as a working tree"
  fi

  # (8) **A foreign cwd.** The whole script must behave the same run from anywhere; the
  #     sibling `toolchain.sh` is sourced by the script's own directory and not by `.`.
  if ( cd / && . "$HERE/toolchain.sh" && C11_TOOLCHAIN=stable pick_toolchain "$TD/nopin" >/dev/null 2>&1 ); then
    ok_ "sourced and run from a foreign working directory"
  else
    bad_ "the script depends on the directory it is run from"
  fi

  # (9) **An expected SHA that does not match stops the run.** `check_sha` exits, so the arm
  #     runs it in a subshell and asserts the exit code.
  ( check_sha "fixture" "$TD/nopin" "0000000000000000000000000000000000000000" ) >/dev/null 2>&1
  if [ $? -ne 0 ]; then ok_ "a HEAD that is not the expected SHA stops the run"
  else bad_ "a HEAD that is not the expected SHA was accepted"; fi
  ( check_sha "fixture" "$TD/nopin" "$(git -C "$TD/nopin" rev-parse HEAD)" ) >/dev/null 2>&1
  if [ $? -eq 0 ]; then ok_ "a HEAD that is the expected SHA is accepted"
  else bad_ "a matching SHA was refused"; fi

  hdr "self-test: status"
  if [ "$st_fail" -eq 0 ]; then
    note "every injected fault was refused, and every clean control was accepted"
    exit 0
  fi
  note "AT LEAST ONE CHECK DID NOT DO ITS JOB — this gate is not admissible; see above"
  exit 1
fi

hdr "0. preflight"
note "date        : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
note "uname       : $(uname -srm)"
note "niles       : $NILES"
note "gbs         : $GBS"
# **`--is-inside-work-tree`, not `--git-dir`.** A bare repository has a git directory and no
# working tree, and every command below is about files on disk; `--git-dir` accepts it. A
# linked worktree, whose `.git` is a file rather than a directory, is accepted by both and was
# rejected by the `-d "$REPO/.git"` test this replaced (F-11-12).
for pair in "niles:$NILES" "gbs:$GBS"; do
  _n="${pair%%:*}"; _p="${pair#*:}"
  if [ "$(git -C "$_p" rev-parse --is-inside-work-tree 2>/dev/null)" != "true" ]; then
    note "FATAL: $_p is not inside a git working tree, so the $_n gates below would be about"
    note "       files this script cannot name a commit for."
    exit 3
  fi
done
note "niles HEAD  : $(git -C "$NILES" rev-parse --short HEAD) on $(git -C "$NILES" rev-parse --abbrev-ref HEAD)"
note "gbs HEAD    : $(git -C "$GBS" rev-parse --short HEAD) on $(git -C "$GBS" rev-parse --abbrev-ref HEAD)"

check_sha "niles" "$NILES" "$NILES_SHA"
check_sha "gbs  " "$GBS" "$GBS_SHA"

resolve_repo "$NILES" NILES_TC
resolve_repo "$GBS" GBS_TC
note "niles rustc : $NILES_TC_VER"
note "            : $NILES_TC_HOW"
note "gbs rustc   : $GBS_TC_VER"
note "            : $GBS_TC_HOW"
note "inherited   : RUSTUP_TOOLCHAIN=$TOOLCHAIN_INHERITED in this shell; every child below is"
note "            : launched with the selector its own repository produced, so this value"
note "            : cannot reach one."
if [ -n "${NILES_NO_GBS:-}" ]; then
  note ""
  note "REFUSED: NILES_NO_GBS is set in this environment. It is an opt-out from the paired"
  note "         adapter gate, and a paired audit gate is what it opts out of. Unset it and"
  note "         re-run:  unset NILES_NO_GBS"
  exit 2
fi

# Untracked files are not a refusal here either, for the reason c11-baselock.sh gives: the
# five protected files in Host C's tree are untracked, and a check that refused on them would
# refuse every run on this machine.
for tree in "$NILES" "$GBS"; do
  dirty="$(git -C "$tree" status --porcelain --untracked-files=no 2>/dev/null)"
  if [ -n "$dirty" ]; then
    note ""
    note "NOTE: $tree has modified tracked files. The gates below still run — they are about"
    note "      the working tree, not about a commit — but the SHA above does not describe"
    note "      what was tested:"
    printf '%s\n' "$dirty" | head -10 | sed 's/^/        /'
  fi
done

hdr "1. niles: the workspace suite"
if in_repo "$NILES" "$NILES_TC_SEL" cargo test --offline --workspace >/tmp/c11-gates-niles.log 2>&1; then
  grep -E "^test result" /tmp/c11-gates-niles.log \
    | awk -F'[ ;]+' '{p+=$4; f+=$6; i+=$8} END {printf "  pass %d  fail %d  ignored %d\n", p, f, i}'
  note "  green"
else
  bad "the niles workspace suite"
  grep -E "^test .* FAILED|^error|panicked at" /tmp/c11-gates-niles.log | head -20 | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-niles.log"
fi
note '  NOTE: numeric_binary_oracle needs a PostgreSQL on 5432. Its absence is a host fact'
note "        and a red test either way — if it failed above for that reason, say so; do not"
note "        report it as a defect in this tree."

hdr "2. the paired adapter gate — the one neither tree can run alone"
note "This is the check that was green for a whole cycle while the pair was broken: this"
note "workspace never builds the adapter, and GBS's own gate was not being run."
if GBS_ROOT="$GBS" in_repo "$NILES" "$NILES_TC_SEL" cargo test --offline -p nilestream-core --test downstream_adapter -- --nocapture \
     >/tmp/c11-gates-adapter.log 2>&1; then
  if grep -q "PAIRED ADAPTER GATE: NOT RUN" /tmp/c11-gates-adapter.log; then
    notrun "the paired adapter gate opted out (NILES_NO_GBS). A pass produced by an opt-out is not a pass."
    grep "PAIRED ADAPTER GATE" /tmp/c11-gates-adapter.log | sed 's/^/    /'
  else
    note "  green: the adapter compiles against this tree's public traits"
  fi
else
  bad "the paired adapter gate"
  grep -E "PAIRED ADAPTER GATE|error\[E|panicked at" /tmp/c11-gates-adapter.log | head -15 | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-adapter.log"
fi

hdr "3. gbs: the workspace suite, and the adapter crate that is not a default member"
if in_repo "$GBS" "$GBS_TC_SEL" cargo test --offline --workspace >/tmp/c11-gates-gbs.log 2>&1; then
  grep -E "^test result" /tmp/c11-gates-gbs.log \
    | awk -F'[ ;]+' '{p+=$4; f+=$6; i+=$8} END {printf "  workspace: pass %d  fail %d  ignored %d\n", p, f, i}'
else
  bad "the gbs workspace suite"
  grep -E "^test .* FAILED|^error|panicked at" /tmp/c11-gates-gbs.log | head -20 | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-gbs.log"
fi
# `gbs-nilestream` is deliberately excluded from the default members — it needs this niles
# checkout beside it — so `--workspace` above does not build it and it is run by manifest path.
if in_repo "$GBS" "$GBS_TC_SEL" cargo test --offline --manifest-path crates/gbs-nilestream/Cargo.toml \
     >/tmp/c11-gates-gbs-adapter.log 2>&1; then
  grep -E "^test result" /tmp/c11-gates-gbs-adapter.log \
    | awk -F'[ ;]+' '{p+=$4; f+=$6; i+=$8} END {printf "  adapter  : pass %d  fail %d  ignored %d\n", p, f, i}'
else
  bad "the gbs adapter crate (gbs-nilestream)"
  grep -E "^test .* FAILED|panicked at" /tmp/c11-gates-gbs-adapter.log | head -20 | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-gbs-adapter.log"
fi

hdr "4. the generated documents"
note "Each is regenerated from the workspace and diffed against the committed copy, so a"
note "document that drifts from the artifact fails a build rather than being noticed in"
note "review. The API extractor's own self-test runs first: its failure mode is a *smaller*"
note "appendix, which looks exactly like a correct one."
if in_repo "$NILES" "$NILES_TC_SEL" make generated >/tmp/c11-gates-generated.log 2>&1; then
  note "  green"
  grep -E "^  ok: " /tmp/c11-gates-generated.log | sed 's/^/  /'
else
  bad "the generated-document gate"
  head -40 /tmp/c11-gates-generated.log | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-generated.log"
fi

hdr "5. the memory budgets"
note "Deterministic counters, so one run decides. A budget is never raised to make a row"
note "pass: an over-budget row is a finding, and A10-19 was one."
if in_repo "$NILES" "$NILES_TC_SEL" make reproduce >/tmp/c11-gates-reproduce.log 2>&1; then
  note "  green"
  grep -E "OVER BUDGET|within budget|allocations" /tmp/c11-gates-reproduce.log | head -20 | sed 's/^/    /'
else
  bad "make reproduce"
  grep -E "OVER BUDGET|differs|error" /tmp/c11-gates-reproduce.log | head -20 | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-reproduce.log"
fi

hdr "5b. fmt and lint, on the pin and separately on the newer compiler"
note "Two runs, never one. A11-06's lint rows are red on 1.97.1 and green on the pin, so a"
note "single pass would make this gate's verdict depend on which compiler happened to be"
note "selected. The pin decides the gate; the newer compiler is reported beside it and its"
note "rows are allowed to be red until the repair lands."
for pair in "niles:$NILES:$NILES_TC_SEL" "gbs:$GBS:$GBS_TC_SEL"; do
  _n="$(printf '%s' "$pair" | cut -d: -f1)"
  _p="$(printf '%s' "$pair" | cut -d: -f2)"
  _s="$(printf '%s' "$pair" | cut -d: -f3)"
  if in_repo "$_p" "$_s" cargo fmt --all -- --check >"/tmp/c11-gates-fmt-$_n.log" 2>&1; then
    note "  $_n fmt   ($_s): clean"
  else
    bad "$_n fmt on $_s"
    head -20 "/tmp/c11-gates-fmt-$_n.log" | sed 's/^/    /'
  fi
  if in_repo "$_p" "$_s" cargo clippy --offline --workspace --all-targets -- -D warnings \
       >"/tmp/c11-gates-clippy-$_n.log" 2>&1; then
    note "  $_n clippy($_s): clean"
  else
    bad "$_n clippy on $_s"
    grep -E "^error" "/tmp/c11-gates-clippy-$_n.log" | head -10 | sed 's/^/    /'
    note "    full log: /tmp/c11-gates-clippy-$_n.log"
  fi
done
if [ "$LINT_TOOLCHAIN" = "none" ]; then
  # **A host with one toolchain says so, and the transcript carries the claim.** The cloud
  # container has no second compiler and cannot install one; without this the section would be
  # a permanent red row there, and a gate that is always red is a gate nobody reads. What
  # separates this from defaulting to silence is that `none` is an assertion by the caller, in
  # the transcript, that this host has only one toolchain — the same rule as `C11_TOOLCHAIN`.
  note "  the newer compiler was declared absent on this host (--lint-toolchain none)."
  note "  A11-06's rows are therefore NOT covered by this run, and no conclusion about the"
  note "  newer lint may be drawn from its exit code."
elif [ -z "$LINT_TOOLCHAIN" ]; then
  note "  the newer compiler was NOT run: pass --lint-toolchain <name> (Host C: 1.97.1), or"
  note "  --lint-toolchain none to assert that this host has only one."
  note "  This is not a green row. It is a row that did not run, and A11-06 is about exactly"
  note "  the rows it would have produced."
  NOTRUN="${NOTRUN}
  - the newer-compiler lint section (no --lint-toolchain given)"
  FAILED=1
else
  for pair in "niles:$NILES" "gbs:$GBS"; do
    _n="${pair%%:*}"; _p="${pair#*:}"
    if in_repo "$_p" "$LINT_TOOLCHAIN" cargo clippy --offline --workspace --all-targets -- -D warnings \
         >"/tmp/c11-gates-clippy-new-$_n.log" 2>&1; then
      note "  $_n clippy($LINT_TOOLCHAIN): clean"
    else
      note "  $_n clippy($LINT_TOOLCHAIN): RED — reported, and not counted against this gate"
      grep -E "^error" "/tmp/c11-gates-clippy-new-$_n.log" | head -10 | sed 's/^/      /'
      note "      full log: /tmp/c11-gates-clippy-new-$_n.log"
    fi
  done
fi

hdr "6. status"
if [ -n "$NOTRUN" ]; then
  note "SECTIONS THAT DID NOT RUN:$NOTRUN"
fi
if [ "$FAILED" -eq 0 ]; then
  note "every gate ran and every gate is green"
else
  note ""
  note "At least one gate failed or did not run. That is a result — report it as one, with"
  note "the log path beside it. Do not raise a budget, delete a test, or set NILES_NO_GBS to"
  note "make this exit zero."
fi
exit $FAILED
