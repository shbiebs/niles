#!/usr/bin/env bash
# Cycle 7 audit preflight — run this FIRST, in whatever container you are auditing from,
# and paste its entire output at the top of your work order.
#
#   bash docs/audit/cycle-7/preflight.sh            # from a clone of `niles`
#   bash preflight.sh /path/to/niles /path/to/gbs   # or name both trees
#
# It writes nothing outside its own scratch directory and touches no tracked file. It
# answers one question: **what may a number measured in this container be used to claim?**
# Cycle 6 lost a whole finding to that question going unasked — an audit container on an
# overlay mounted `fsync=volatile` reported a ~1,000,000/s "fsync" ceiling, which is a mount
# option and not storage, and every durability conclusion drawn there was void.

set -u
NILES="${1:-$(pwd)}"
GBS="${2:-}"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
say() { printf '%s\n' "$*"; }
hdr() { printf '\n=== %s\n' "$*"; }

say "cycle-7 audit preflight — $(date -u '+%Y-%m-%dT%H:%M:%SZ')"

# ---------------------------------------------------------------- A. the machine
hdr "A. host"
say "uname            : $(uname -srmo 2>/dev/null || uname -a)"
say "cores (nproc)    : $(nproc 2>/dev/null || echo '?')"
if [ -r /proc/cpuinfo ]; then
  say "cpu model        : $(grep -m1 -E 'model name|Model' /proc/cpuinfo | cut -d: -f2- | sed 's/^ *//')"
fi
say "memtotal         : $(awk '/MemTotal/{printf "%.1f GiB", $2/1048576}' /proc/meminfo 2>/dev/null || echo '?')"
say "cgroup cpu.max   : $(cat /sys/fs/cgroup/cpu.max 2>/dev/null || echo 'n/a')"
say "cgroup mem.max   : $(cat /sys/fs/cgroup/memory.max 2>/dev/null || echo 'n/a')"
say "NOTE: nproc can exceed what the cgroup actually grants. If cpu.max is not 'max',"
say "      divide the quota by the period — that, not nproc, is your core count."

# ---------------------------------------------------------- B. storage, and the trap
hdr "B. filesystem under the tree — THE decisive section"
TARGET_FS_DIR="$NILES"
say "path             : $TARGET_FS_DIR"
if command -v findmnt >/dev/null 2>&1; then
  findmnt -no SOURCE,FSTYPE,OPTIONS --target "$TARGET_FS_DIR" 2>/dev/null | sed 's/^/mount            : /'
else
  df -PT "$TARGET_FS_DIR" 2>/dev/null | tail -1 | sed 's/^/df               : /'
  mount 2>/dev/null | grep -E ' on / | overlay|fuse' | head -3 | sed 's/^/mount            : /'
fi
say "free space       : $(df -Ph "$TARGET_FS_DIR" 2>/dev/null | tail -1 | awk '{print $4" avail of "$2}')"
say ""
say "-- the barrier probe: 4 KiB write + fdatasync, 7 times, median reported"
if command -v python3 >/dev/null 2>&1; then
  python3 - "$SCRATCH" <<'PY'
import os, statistics, sys, time
d = sys.argv[1]
p = os.path.join(d, "barrier.probe")
fd = os.open(p, os.O_CREAT | os.O_WRONLY, 0o600)
buf = b"x" * 4096
us = []
for i in range(7):
    os.pwrite(fd, buf, 0)
    t = time.perf_counter()
    try:
        os.fdatasync(fd)
        name = "fdatasync"
    except AttributeError:
        os.fsync(fd)
        name = "fsync"
    us.append((time.perf_counter() - t) * 1e6)
os.close(fd); os.unlink(p)
m = statistics.median(us)
rate = 1e6 / m if m > 0 else float("inf")
print(f"barrier          : {name}")
print(f"median           : {m:.1f} us  ->  {rate:,.0f} barriers/s")
print(f"spread           : {min(us):.1f}-{max(us):.1f} us")
if rate > 100000:
    print("VERDICT          : *** NOT STORAGE EVIDENCE ***")
    print("                   A barrier this fast is a mount option, not a device. This")
    print("                   container CANNOT produce any durability number. Every")
    print("                   `durable` row measured here is void; say so in your work")
    print("                   order and route those measurements to Host C.")
elif rate > 20000:
    print("VERDICT          : suspicious — verify the mount is not volatile before")
    print("                   publishing any durability figure.")
else:
    print("VERDICT          : plausible as a real barrier for THIS container. Still a")
    print("                   host-shaped number: usable for within-host ratios only.")
PY
else
  say "python3 absent — cannot probe the barrier. Treat every durability number from this"
  say "container as unusable until the probe runs."
fi
say ""
say "reminder: \`make fsync-proof\` checks the syscall REACHES THE KERNEL. It passes on a"
say "volatile overlay too. The two checks are necessary together; neither is sufficient."

# ------------------------------------------------------------------ C. toolchain
hdr "C. toolchain"
say "rustc            : $(rustc --version 2>/dev/null || echo 'ABSENT — you cannot build or test')"
say "cargo            : $(cargo --version 2>/dev/null || echo ABSENT)"
say "host triple      : $(rustc -vV 2>/dev/null | awk '/^host:/{print $2}')"
say "rustfmt          : $(cargo fmt --version 2>/dev/null || echo ABSENT)"
say "clippy           : $(cargo clippy --version 2>/dev/null || echo ABSENT)"
say "valgrind         : $(valgrind --version 2>/dev/null || echo 'ABSENT — no callgrind/dhat/massif attribution')"
say "python3          : $(python3 --version 2>&1 || echo ABSENT)"
say "git              : $(git --version 2>/dev/null || echo ABSENT)"
say "strace           : $(command -v strace >/dev/null 2>&1 && echo present || echo 'ABSENT — make fsync-proof cannot run')"
say "CARGO_TARGET_DIR : ${CARGO_TARGET_DIR:-<unset, which is correct>}"
if [ -n "${CARGO_TARGET_DIR:-}" ]; then
  say "WARNING          : set. In cycle 5 this made a nested \`cargo run\` contend with the"
  say "                   outer \`cargo test\` on one build-directory lock: six spurious"
  say "                   failures and a wrong verdict in a thesis table. T-03 removed the"
  say "                   nested calls; verifying that WITH this set is a task, not an"
  say "                   accident. Unset it for ordinary runs."
fi
RSV="$(rustc --version 2>/dev/null | awk '{print $2}')"
say ""
say "the tree pins \`channel = \"stable\"\` with \`rust-version = 1.95.0\`, because the machine"
say "it was built on had no egress to static.rust-lang.org and could not name a version."
say "Your \`stable\` is ${RSV:-unknown}. If that is NEWER than 1.95.0, a new lint can turn"
say "\`clippy -- -D warnings\` red for reasons unrelated to this code. **A red gate from a"
say "newer toolchain is a finding about the pin, not about the code** — report it that way."

# ------------------------------------------------------------------- D. services
hdr "D. PostgreSQL"
say "psql             : $(psql --version 2>/dev/null || echo ABSENT)"
say "postgres         : $(postgres --version 2>/dev/null || pg_config --version 2>/dev/null || echo ABSENT)"
if command -v pg_isready >/dev/null 2>&1; then
  say "pg_isready       : $(pg_isready 2>&1 | tail -1)"
else
  say "pg_isready       : ABSENT"
fi
say ""
say "\`numeric_binary_oracle\` needs a running server with a \`bench\` role and PGPORT set."
say "Without it the tree reads RED for a missing service, not a defect — do not report that"
say "as a failing test. Start one with \`pg_ctlcluster 16 main start\` or a private cluster:"
say "  initdb -D <pgdata outside the repo> -U bench --auth=trust"
say "  pg_ctl -D <pgdata> -o '-p 5433 -c listen_addresses=127.0.0.1' start"
say "  createdb -h 127.0.0.1 -p 5433 -O bench bench"
say "Record fsync / synchronous_commit / full_page_writes / wal_level / shared_buffers /"
say "work_mem / max_wal_size AND **wal_sync_method** beside any comparison."

# -------------------------------------------------------------------- E. network
hdr "E. network egress"
for host in https://github.com https://crates.io https://static.rust-lang.org; do
  code="$(curl -sS -m 8 -o /dev/null -w '%{http_code}' "$host" 2>/dev/null || echo 'FAIL')"
  say "$(printf '%-28s' "$host"): $code"
done
say ""
say "Egress here is YOURS, not the executing agent's. **No task you write may require"
say "network access at execution time**: Opus's container has no egress to crates.io or"
say "static.rust-lang.org, and no task may install anything. Both trees have **zero external"
say "dependencies**, so a bare toolchain builds them offline — verify that with"
say "\`cargo test --offline --workspace\` rather than assuming it."

# ----------------------------------------------------------------- F. tree state
hdr "F. trees"
for t in "$NILES" "$GBS"; do
  [ -z "$t" ] && continue
  [ -d "$t/.git" ] || { say "$t: not a git tree"; continue; }
  say "$t"
  say "  HEAD           : $(git -C "$t" log --oneline -1 2>/dev/null)"
  say "  branch         : $(git -C "$t" rev-parse --abbrev-ref HEAD 2>/dev/null)"
  say "  dirty          : $(git -C "$t" status --porcelain 2>/dev/null | wc -l | tr -d ' ') entries"
  git -C "$t" status --porcelain 2>/dev/null | head -6 | sed 's/^/    /'
done
say ""
say "EXPECTED at the start of cycle 7:"
say "  niles  d9c8699  c6/06a-lock-order   (or 474f153 c6/audit-cycle-7, which adds only docs)"
say "  gbs    e803b7d  c6/07-hold-index"
say "If you are on the author's Mac, the FOUR untracked files in \`niles\` are his and must"
say "never be edited, staged, deleted, moved or bundled: .DS_Store, AGENTS.md,"
say "thesis/.DS_Store, thesis/Niles-Thesis.pdf. Anything else untracked is yours to explain."

# ------------------------------------------------------------------- G. the gate
hdr "G. gate — run these yourself and record the result"
say "  cargo fmt --all -- --check"
say "  cargo clippy --offline --all-targets -- -D warnings"
say "  cargo test --offline --workspace          # niles: 801 test fns; gbs: 506 declared"
say "  make reproduce                            # must exit 0 with a clean diff"
say "  make fsync-proof                          # needs strace"
say "A red row here is a RESULT. Record it; never fix it by deletion."

# -------------------------------------------------------------- H. admissibility
hdr "H. admissibility — fill this in and put it in your work order"
say "  cores actually granted        : ____   (cgroup quota, not nproc)"
say "  barrier + median + rate       : ____"
say "  storage evidence?             : yes / NO (volatile mount)"
say "  may publish durability rows?  : ____"
say "  may publish >3-core curves?   : ____   (needs the cgroup answer above)"
say "  toolchain newer than 1.95.0?  : ____"
say "  valgrind attribution possible : ____"
say "  PostgreSQL comparison possible: ____   (and under which wal_sync_method)"
say ""
say "Rules this table decides:"
say "  * No absolute wall-clock, throughput or fsync figure beside another host's without a"
say "    host column."
say "  * Deterministic counters (instructions, allocations, visited entries, max_batch,"
say "    txns/fsync, lock buckets, compiler verdicts) gate first and need ONE run."
say "  * Wall clock needs 2 warm-ups and >=5 measured runs, arms interleaved, median/MAD/"
say "    range; a gate fires only on >=10% AND >=3x the pooled MAD, else 'noise-limited'."
say "  * 'unsupported', 'blocked', 'not run', 'noise-limited' are RESULTS. Omission is not."
say ""
say "preflight complete."
