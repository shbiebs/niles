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
. "$(cd "$(dirname "$0")" && pwd)/toolchain.sh"

note()  { printf '%s\n' "$*"; }
hdr()   { printf '\n=== %s ===\n' "$*"; }
notrun(){ NOTRUN="${NOTRUN}
  - $*"; FAILED=1; note "NOT RUN: $*"; }
bad()   { FAILED=1; note "FAILED: $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --niles) NILES="${2:-}"; shift ;;
    --gbs)   GBS="${2:-}"; shift ;;
    *) note "REFUSED: unknown flag \`$1\`. A mistyped flag that is ignored is a run that"
       note "         checked something other than what was asked for."
       exit 2 ;;
  esac
  shift
done

pick_toolchain
export CARGO_NET_OFFLINE=true
export GIT_TERMINAL_PROMPT=0

hdr "0. preflight"
note "date        : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
note "uname       : $(uname -srm)"
note "niles       : $NILES"
note "gbs         : $GBS"
git -C "$NILES" rev-parse --git-dir >/dev/null 2>&1 || { note "FATAL: $NILES is not a git checkout."; exit 3; }
git -C "$GBS" rev-parse --git-dir >/dev/null 2>&1   || { note "FATAL: $GBS is not a git checkout."; exit 3; }
note "niles HEAD  : $(git -C "$NILES" rev-parse --short HEAD) on $(git -C "$NILES" rev-parse --abbrev-ref HEAD)"
note "gbs HEAD    : $(git -C "$GBS" rev-parse --short HEAD) on $(git -C "$GBS" rev-parse --abbrev-ref HEAD)"
note "rustc       : $TOOLCHAIN_USED"
note "            : $TOOLCHAIN_HOW"
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
if ( cd "$NILES" && cargo test --offline --workspace ) >/tmp/c11-gates-niles.log 2>&1; then
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
if ( cd "$NILES" && GBS_ROOT="$GBS" cargo test --offline -p nilestream-core --test downstream_adapter -- --nocapture ) \
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
if ( cd "$GBS" && cargo test --offline --workspace ) >/tmp/c11-gates-gbs.log 2>&1; then
  grep -E "^test result" /tmp/c11-gates-gbs.log \
    | awk -F'[ ;]+' '{p+=$4; f+=$6; i+=$8} END {printf "  workspace: pass %d  fail %d  ignored %d\n", p, f, i}'
else
  bad "the gbs workspace suite"
  grep -E "^test .* FAILED|^error|panicked at" /tmp/c11-gates-gbs.log | head -20 | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-gbs.log"
fi
# `gbs-nilestream` is deliberately excluded from the default members — it needs this niles
# checkout beside it — so `--workspace` above does not build it and it is run by manifest path.
if ( cd "$GBS" && cargo test --offline --manifest-path crates/gbs-nilestream/Cargo.toml ) \
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
if ( cd "$NILES" && make generated ) >/tmp/c11-gates-generated.log 2>&1; then
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
if ( cd "$NILES" && make reproduce ) >/tmp/c11-gates-reproduce.log 2>&1; then
  note "  green"
  grep -E "OVER BUDGET|within budget|allocations" /tmp/c11-gates-reproduce.log | head -20 | sed 's/^/    /'
else
  bad "make reproduce"
  grep -E "OVER BUDGET|differs|error" /tmp/c11-gates-reproduce.log | head -20 | sed 's/^/    /'
  note "  full log: /tmp/c11-gates-reproduce.log"
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
