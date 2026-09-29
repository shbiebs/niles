#!/bin/sh
# E27 at the big-lender size (10⁶ accounts) on Host C, the author's Mac — arms N and P+ only,
# counted work only.
#
# Why here: the cloud container has 2 cores and 8 GB; 10⁶ accounts is 2 × 10⁷ legs per arm.
# Why only N and P+: the author's decision of 2026-09-28. P and M refresh a full materialised
# view on every write, which at 10⁶ is minutes per write; they are NOT RUN at 10⁶ and E27 says
# why.
#
# Why counted work only (the author's decision of 2026-09-28): the Mac has 16 GB and, scaled
# tenfold from the container's 10⁵ measurements, N alone needs about 9 GB single-currency and
# 27 GB multi-currency, so this run swaps. Its latencies and memory figures then measure the
# swap device; they stay in the point files, are labelled swap-affected, and are never
# rendered or compared. What is published is the work the engines count (keys resident,
# reads, hits, misses, base rows touched, relation sizes), rendered beside the container's
# 10³–10⁵ by `comparator render --shape counted` into results/E27-hostc.md. The targeted p99
# shape is not run here: it measures latency only.
#
# Order: single-currency first (hours); then multi-currency, which may take days, because
# every multi-currency read on N folds the ledger. Set E27_SERIES=single to stop after the
# first. The run is resumable per point (--skip-existing): a point interrupted part-way is
# measured again from its start. Keep the Mac awake (`caffeinate -i sh tools/e27-hostc.sh`)
# and leave free disk for swap (the script warns below 60 GB free).
#
# What it needs: PostgreSQL 16 binaries (Homebrew `postgresql@16` or Postgres.app), the Rust
# toolchain pinned in rust-toolchain.toml, and this checkout at the R2-03 head. It creates
# per-arm clusters under ~/.e27-pg (port 5452 for P+), owned by you, with peer auth on the
# socket and SCRAM on TCP; it never edits pg_hba.conf.
#
# Usage, from the repository root:  caffeinate -i sh tools/e27-hostc.sh
# Output: results/E27-hostc/ (one .tsv per point), and a copy under
# ~/Documents/niles-sync/cycle-14/e27-hostc/main/ for the session to read.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ -z "${PG_BIN:-}" ]; then
    for d in /opt/homebrew/opt/postgresql@16/bin /usr/local/opt/postgresql@16/bin \
             /Applications/Postgres.app/Contents/Versions/16/bin; do
        if [ -x "$d/initdb" ]; then PG_BIN=$d; break; fi
    done
fi
if [ -z "${PG_BIN:-}" ] || [ ! -x "$PG_BIN/initdb" ]; then
    echo "e27-hostc: PostgreSQL 16 not found. Install it (brew install postgresql@16) or set PG_BIN to its bin directory." >&2
    exit 2
fi
case "$("$PG_BIN/postgres" --version)" in
    *" 16."*) ;;
    *) echo "e27-hostc: $PG_BIN is not PostgreSQL 16: $("$PG_BIN/postgres" --version)" >&2; exit 2 ;;
esac
export PG_BIN
export PATH="$PG_BIN:$PATH"
export E27_PGDATA="$HOME/.e27-pg"

echo "e27-hostc: $(uname -srm), $(sysctl -n hw.ncpu) CPUs, $(( $(sysctl -n hw.memsize) / 1048576 )) MiB"
# Memory: this run is expected to swap (see the header); it is not refused for it. Swap on
# macOS grows on the boot volume, so warn when free space is short.
mem_gib=$(( $(sysctl -n hw.memsize) / 1073741824 ))
free_gib=$(df -g "$HOME" | awk 'NR==2 {print $4}')
echo "e27-hostc: ${mem_gib} GiB of memory, ${free_gib} GiB free on the home volume; latency and memory figures from this run are swap-affected and are not published"
if [ "${free_gib:-0}" -lt 60 ]; then
    echo "e27-hostc: warning: under 60 GiB free; the multi-currency series may exhaust swap space" >&2
fi
echo "e27-hostc: $(cargo --version), $("$PG_BIN/postgres" --version)"
echo "e27-hostc: commit $(git rev-parse HEAD)"

cargo build --release -p nilestream-server --bin nilestreamd
cargo build --release -p comparator

started=$(date +%s)
for series in ${E27_SERIES:-single multi}; do
    echo "e27-hostc: $(date -u +%FT%TZ) series $series"
    ./target/release/comparator run --series "$series" --sizes 1000000 \
        --seeds 1,7,42,100,2024 --arms N,P+ --dir results/E27-hostc --skip-existing \
        --scratch "$HOME/.e27-scratch"
    out="$HOME/Documents/niles-sync/cycle-14/e27-hostc/main"
    mkdir -p "$out"
    cp results/E27-hostc/*.tsv "$out/"
    echo "e27-hostc: $(date -u +%FT%TZ) series $series done; copied to $out"
done
echo "e27-hostc: measured in $(( $(date +%s) - started )) s"
ls -l "$HOME/Documents/niles-sync/cycle-14/e27-hostc/main"
echo "e27-hostc: done"
