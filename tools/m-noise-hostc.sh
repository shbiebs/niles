#!/bin/sh
# M-NOISE on Host C, the author's Mac (cycle 17, C17-12 run 1). The design is
# docs/study/M-NOISE-design.md; its §9, committed before this script, is this run's rule.
#
# The same ten points as C17-04 (results/MANIFEST.csv rows for M-NOISE/L1 and L2): single
# currency, 10⁴ accounts, arms P+ and P-inc, seeds 1, 7, 42, 100, 2024; for each seed the L1
# point, then the L2 point (--length-factor 2). About an hour.
#
# The quiet-machine protocol (work order §6.3) is yours to keep: mains power, other
# applications closed, no Time Machine backup during the run, and the Mac kept awake by
# running this script under caffeinate. The script records the machine (sysctl, sw_vers) and
# the load averages at the start and the end, and refuses to start when the one-minute load
# average is above 1.5.
#
# What it needs: PostgreSQL 16 binaries (Homebrew `postgresql@16` or Postgres.app; or set
# PG_BIN), the Rust toolchain rust-toolchain.toml pins, and a checkout at this branch's head
# with no local changes (every point records whether the worktree is clean). It creates
# per-arm clusters under ~/.e27-pg (ports 5452 and 5455), owned by you, with peer auth on the
# socket and SCRAM on TCP; it never edits pg_hba.conf.
#
# Usage, from the repository root:   caffeinate -dimsu sh tools/m-noise-hostc.sh
# Output: results/M-NOISE-hostc/ (L1/, L2/, host.txt) and results/M-NOISE-hostc.md, and a copy
# of all of it under ~/Documents/niles-sync/cycle-17/runs/m-noise-hostc/ for the session to read.
# Re-running resumes: a point already written is skipped (--skip-existing).
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ "$(uname -s)" != Darwin ]; then
    echo "m-noise-hostc: this is Host C's script and expects macOS; uname says $(uname -s)." >&2
    exit 2
fi

if [ -z "${PG_BIN:-}" ]; then
    for d in /opt/homebrew/opt/postgresql@16/bin /usr/local/opt/postgresql@16/bin \
             /Applications/Postgres.app/Contents/Versions/16/bin; do
        if [ -x "$d/initdb" ]; then PG_BIN=$d; break; fi
    done
fi
if [ -z "${PG_BIN:-}" ] || [ ! -x "$PG_BIN/initdb" ]; then
    echo "m-noise-hostc: PostgreSQL 16 not found. Install it (brew install postgresql@16) or set PG_BIN to its bin directory." >&2
    exit 2
fi
case "$("$PG_BIN/postgres" --version)" in
    *" 16."*) ;;
    *) echo "m-noise-hostc: $PG_BIN is not PostgreSQL 16: $("$PG_BIN/postgres" --version)" >&2; exit 2 ;;
esac
export PG_BIN
export PATH="$PG_BIN:$PATH"
export E27_PGDATA="$HOME/.e27-pg"

# The one-minute load average, from `sysctl -n vm.loadavg` ("{ 1.23 1.10 0.98 }").
load1=$(sysctl -n vm.loadavg | awk '{print $2}')
if awk -v l="$load1" 'BEGIN { exit !(l > 1.5) }'; then
    echo "m-noise-hostc: refused to start: the one-minute load average is $load1, above 1.5 (work order §6.3)." >&2
    echo "m-noise-hostc: close other applications, wait a few minutes, and run it again." >&2
    exit 3
fi

if ! git diff --quiet HEAD -- crates Cargo.toml Cargo.lock; then
    echo "m-noise-hostc: the checkout has local changes under crates/ or Cargo.*; every point would record MODIFIED. Run it from a clean checkout of this branch." >&2
    exit 2
fi

out=results/M-NOISE-hostc
mkdir -p "$out/L1" "$out/L2"
record() {
    {
        echo "$1: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "  machine: $(sysctl -n hw.machine), $(sysctl -n hw.ncpu) CPUs, $(( $(sysctl -n hw.memsize) / 1048576 )) MiB"
        echo "  macOS: $(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
        echo "  load averages: $(sysctl -n vm.loadavg)"
    } >> "$out/host.txt"
}
if [ ! -f "$out/host.txt" ]; then
    {
        echo "commit: $(git rev-parse HEAD)"
        echo "toolchain: $(cargo --version)"
        echo "postgres: $("$PG_BIN/postgres" --version)"
    } > "$out/host.txt"
fi
record start
cat "$out/host.txt"

# Built before the first point, so no build runs beside one (design §7).
cargo build --release -p comparator

started=$(date +%s)
for seed in 1 7 42 100 2024; do
    ./target/release/comparator run --study e27b --series single --sizes 10000 --seeds "$seed" \
        --arms P+,P-inc --dir "$out/L1" --skip-existing --scratch "$HOME/.e27-scratch"
    ./target/release/comparator run --study e27b --series single --sizes 10000 --seeds "$seed" \
        --arms P+,P-inc --length-factor 2 --dir "$out/L2" --skip-existing --scratch "$HOME/.e27-scratch"
done
record end
echo "m-noise-hostc: measured in $(( $(date +%s) - started )) s"

./target/release/comparator m-noise --host hostc

copy="$HOME/Documents/niles-sync/cycle-17/runs/m-noise-hostc"
mkdir -p "$copy/L1" "$copy/L2"
cp "$out"/L1/*.tsv "$copy/L1/"
cp "$out"/L2/*.tsv "$copy/L2/"
cp "$out/host.txt" results/M-NOISE-hostc.md "$copy/"
( cd "$copy" && shasum -a 256 host.txt M-NOISE-hostc.md L1/*.tsv L2/*.tsv > SHA256SUMS )
echo "m-noise-hostc: copied to $copy"
sed -n '/^## Verdict/,/^## Refusals/p' results/M-NOISE-hostc.md
echo "m-noise-hostc: done"
