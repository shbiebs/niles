#!/bin/sh
# Every precondition of `make gate`, checked at once and named (cycle 14, decision D-1).
# The gate had preconditions that lived in no file: a pinned toolchain, a running PostgreSQL,
# and a role that a fresh server does not have. A fresh container failed the gate on the
# third, one test at a time. This names everything missing in one message.
set -u
fail=0
v=$(cargo --version 2>/dev/null || true)
case "$v" in
    "cargo 1.95.0 "*) echo "preflight: toolchain $v" ;;
    *) echo "preflight: toolchain is '$v', not the 1.95.0 pin (rust-toolchain.toml; RUSTUP_TOOLCHAIN=stable is allowed only if it reports 1.95.0)" >&2; fail=1 ;;
esac
port=${PGPORT:-5432}
if pg_isready -q -h 127.0.0.1 -p "$port"; then
    echo "preflight: PostgreSQL accepting on 127.0.0.1:$port"
else
    echo "preflight: no PostgreSQL on 127.0.0.1:$port (start it: pg_ctlcluster 16 main start, or your platform's equivalent)" >&2
    fail=1
fi
root=$(cd "$(dirname "$0")/.." && pwd)
if [ -z "${PGPASSWORD:-}" ] && [ ! -s "${NILES_PGPASSFILE:-$root/.pg-bench-password}" ]; then
    echo "preflight: no password for role bench (run tools/pg-provision.sh)" >&2
    fail=1
fi
[ "$fail" = 0 ] || exit 1
cargo run -q -p bank-bench --bin preflight
