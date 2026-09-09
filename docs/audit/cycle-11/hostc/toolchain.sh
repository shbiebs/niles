# Sourced, not run. The one toolchain probe every cycle-11 script uses.
#
# Cycle 10 had three copies of this function and repaired one of them twice (MF-11, MF-14). The
# other two still unset the override that makes a container build possible and then report that
# the tree's pin resolves. One copy, sourced, is the repair; a self-test arm in c11-baselock.sh
# asserts that no script under this directory defines its own.
#
# ---------------------------------------------------------------------------------------------
# WHAT CHANGED IN C11-06.4, AND THE DEFECT THAT CAUSED IT
#
# The first version took its repository from an ambient `REPO` variable. `c11-gates.sh` calls
# this function and never sets one, so under `set -u` both reads died, `TOOLCHAIN_PIN` came back
# empty, and the preflight printed:
#
#     rustc       :
#                 : RUSTUP_TOOLCHAIN=stable, because the pin (none declared) does not resolve here
#
# — a sentence about a pin the function had never looked for, in a repository that declares
# 1.95.0 in `rust-toolchain.toml`. The gate then built everything under whatever `stable` is on
# that machine and said the tree had asked for it. That is MF-7 again, one cycle after MF-7, in
# the script written to prevent MF-7.
#
# Three things follow from it and are the contract of this file now.
#
#   1. **The repository is an argument.** `pick_toolchain <repo>`. A caller that supplies none
#      gets a refusal with a non-zero return, not a default and not an empty string.
#   2. **`stable` is never a silent fallback.** If the pin does not resolve, this function
#      refuses unless the caller has explicitly named a substitute in `C11_TOOLCHAIN`. The
#      container has no egress to `static.rust-lang.org` and cannot resolve `1.95.0` by name,
#      so it runs with `C11_TOOLCHAIN=stable` — an override that is now *in the transcript*
#      rather than in the script.
#   3. **A child is launched with the selected compiler, not with the ambient one.**
#      `TOOLCHAIN_SELECTOR` is the name to launch with, and `run_pinned` sets it in the child's
#      environment. An inherited `RUSTUP_TOOLCHAIN` from the caller's shell — Host C's is
#      1.97.1 — cannot reach the child, because the child's value is assigned rather than
#      inherited.
#
# Sets: TOOLCHAIN_PIN (what the tree asks for), TOOLCHAIN_SELECTOR (what a child is launched
# with), TOOLCHAIN_USED (`rustc --version` under it), TOOLCHAIN_HOW (one sentence for the
# transcript), TOOLCHAIN_INHERITED (what the caller's environment had, or `-`).
# Returns: 0 selected, 2 called wrong, 3 the pin does not resolve and no substitute was named.

# **What the operator's own shell had, captured when this file is sourced.**
#
# It must be read before any `pick_toolchain` call, because that call exports its selector: a
# script that read it afterwards would report its own choice back to itself as the value it
# inherited. The gates script printed exactly that — `inherited: RUSTUP_TOOLCHAIN=stable` in a
# shell that had none — until this moved out of the function.
TOOLCHAIN_INHERITED="${RUSTUP_TOOLCHAIN:--}"
export TOOLCHAIN_INHERITED

pick_toolchain() {
  export RUSTUP_AUTO_INSTALL=0
  _repo="${1:-${REPO:-}}"
  if [ -z "$_repo" ]; then
    TOOLCHAIN_PIN="unknown"
    TOOLCHAIN_SELECTOR=""
    TOOLCHAIN_USED=""
    TOOLCHAIN_HOW="REFUSED: pick_toolchain was called without a repository, so no pin was read"
    return 2
  fi
  if [ ! -d "$_repo" ]; then
    TOOLCHAIN_PIN="unknown"
    TOOLCHAIN_SELECTOR=""
    TOOLCHAIN_USED=""
    TOOLCHAIN_HOW="REFUSED: $_repo is not a directory, so no pin was read"
    return 2
  fi
  TOOLCHAIN_PIN="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$_repo/rust-toolchain.toml" 2>/dev/null)"
  TOOLCHAIN_PIN="${TOOLCHAIN_PIN:-none declared}"

  # An explicit arm override wins over everything and is reported as an override. This is the
  # A/B harness asking for one arm on 1.95.0 and the other on 1.97.1 on purpose.
  if [ -n "${ARM_TOOLCHAIN:-}" ]; then
    TOOLCHAIN_SELECTOR="$ARM_TOOLCHAIN"
    TOOLCHAIN_USED="$(RUSTUP_TOOLCHAIN="$TOOLCHAIN_SELECTOR" sh -c "cd '$_repo' && rustc --version" 2>&1)"
    TOOLCHAIN_HOW="RUSTUP_TOOLCHAIN=$ARM_TOOLCHAIN, an explicit arm override (the tree pins $TOOLCHAIN_PIN)"
    export RUSTUP_TOOLCHAIN="$TOOLCHAIN_SELECTOR"
    return 0
  fi

  # The tree's own pin, resolved with nothing inherited in the way.
  if [ "$TOOLCHAIN_PIN" != "none declared" ] \
     && ( unset RUSTUP_TOOLCHAIN; cd "$_repo" && rustc --version ) >/dev/null 2>&1; then
    TOOLCHAIN_SELECTOR="$TOOLCHAIN_PIN"
    TOOLCHAIN_USED="$(RUSTUP_TOOLCHAIN="$TOOLCHAIN_SELECTOR" sh -c "cd '$_repo' && rustc --version" 2>&1)"
    TOOLCHAIN_HOW="the tree's pin ($TOOLCHAIN_PIN), resolved in $_repo and named to every child"
    export RUSTUP_TOOLCHAIN="$TOOLCHAIN_SELECTOR"
    return 0
  fi

  # The pin does not resolve. This is a refusal unless a substitute was named out loud.
  if [ -z "${C11_TOOLCHAIN:-}" ]; then
    TOOLCHAIN_SELECTOR=""
    TOOLCHAIN_USED=""
    TOOLCHAIN_HOW="REFUSED: the pin ($TOOLCHAIN_PIN) declared by $_repo does not resolve here \
and no substitute was named. Set C11_TOOLCHAIN to the toolchain to use instead — in the cloud \
container that is \`stable\`, where \`stable\` is 1.95.0 — so that the substitution is in the \
transcript rather than in this script."
    return 3
  fi
  TOOLCHAIN_SELECTOR="$C11_TOOLCHAIN"
  TOOLCHAIN_USED="$(RUSTUP_TOOLCHAIN="$TOOLCHAIN_SELECTOR" sh -c "cd '$_repo' && rustc --version" 2>&1)"
  TOOLCHAIN_HOW="C11_TOOLCHAIN=$C11_TOOLCHAIN, named by the caller because the tree's pin \
($TOOLCHAIN_PIN) does not resolve here. Every row this run produces was built with \
$TOOLCHAIN_SELECTOR and not with $TOOLCHAIN_PIN."
  export RUSTUP_TOOLCHAIN="$TOOLCHAIN_SELECTOR"
  return 0
}

# Run a command with an explicitly selected compiler, whatever the caller's environment holds.
#
# `RUSTUP_TOOLCHAIN` is *assigned* in the child rather than left to be inherited, which is what
# makes an operator's own 1.97.1 (Host C's default) unable to reach a gate that resolved 1.95.0.
# `rustup run <tc> <cmd>` would do the same thing and is not used only because it requires
# `rustup` on PATH, which a plain cargo installation need not have.
run_pinned() {
  if [ -z "${TOOLCHAIN_SELECTOR:-}" ]; then
    echo "run_pinned: no toolchain was selected, so nothing may be launched" >&2
    return 3
  fi
  RUSTUP_TOOLCHAIN="$TOOLCHAIN_SELECTOR" "$@"
}
