# Sourced, not run. The one toolchain probe every cycle-11 script uses.
#
# Cycle 10 had three copies of this function and repaired one of them twice (MF-11, MF-14). The
# other two still unset the override that makes a container build possible and then report that
# the tree's pin resolves. One copy, sourced, is the repair; a self-test arm in c11-baselock.sh
# asserts that no script under this directory defines its own.
#
# Sets: TOOLCHAIN_USED, TOOLCHAIN_HOW, TOOLCHAIN_PIN. Takes: REPO (the repository the pin lives
# in). An explicit override in ARM_TOOLCHAIN wins and is reported as such.
pick_toolchain() {
  export RUSTUP_AUTO_INSTALL=0
  TOOLCHAIN_PIN="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$REPO/rust-toolchain.toml" 2>/dev/null)"
  TOOLCHAIN_PIN="${TOOLCHAIN_PIN:-none declared}"
  if [ -n "${ARM_TOOLCHAIN:-}" ]; then
    export RUSTUP_TOOLCHAIN="$ARM_TOOLCHAIN"
    TOOLCHAIN_USED="$( cd "$REPO" 2>/dev/null && rustc --version 2>&1 )"
    TOOLCHAIN_HOW="RUSTUP_TOOLCHAIN=$ARM_TOOLCHAIN, an explicit arm override (the pin is $TOOLCHAIN_PIN)"
    return 0
  fi
  unset RUSTUP_TOOLCHAIN
  if ( cd "$REPO" 2>/dev/null && rustc --version ) >/dev/null 2>&1; then
    TOOLCHAIN_USED="$( cd "$REPO" && rustc --version 2>&1 )"
    TOOLCHAIN_HOW="the tree's pin ($TOOLCHAIN_PIN), resolved in $REPO"
  else
    export RUSTUP_TOOLCHAIN=stable
    TOOLCHAIN_USED="$( cd "$REPO" 2>/dev/null && rustc --version 2>&1 )"
    TOOLCHAIN_HOW="RUSTUP_TOOLCHAIN=stable, because the pin ($TOOLCHAIN_PIN) does not resolve here"
  fi
}
