#!/bin/sh
# E27 at the big-lender size (10⁶ accounts) on Host C, the author's Mac — arms N and P+ only.
#
# Why here: the cloud container has 2 cores and 8 GB; 10⁶ accounts is 2 × 10⁷ legs per arm.
# Why only N and P+: the author's decision of 2026-09-28. P and M refresh a full materialised
# view on every write, which at 10⁶ is minutes per write; they are NOT RUN at 10⁶ and E27 says
# why. Figures from this host are never compared with the container's (§5.2): the points go
# to their own directories and render into their own files.
#
# What it needs: PostgreSQL 16 binaries (Homebrew `postgresql@16` or Postgres.app), the Rust
# toolchain pinned in rust-toolchain.toml, and this checkout at the R2-02 head. It creates
# per-arm clusters under ~/.e27-pg (port 5452 for P+), owned by you, with peer auth on the
# socket and SCRAM on TCP; it never edits pg_hba.conf. Memory (PSS) is Linux-only, so this run
# reports time and throughput, not memory. On the multi-currency series the p99 re-run reads q1
# only (the author's decision of 2026-09-28): there each q2 folds the whole ledger.
#
# Usage, from the repository root:  sh tools/e27-hostc.sh
# Output: results/E27-hostc/ and results/E27-p99-hostc/ (one .tsv per point), and a copy of
# both under ~/Documents/niles-sync/cycle-14/e27-hostc/{main,p99}/ for the session to read.
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
echo "e27-hostc: $(cargo --version), $("$PG_BIN/postgres" --version)"
echo "e27-hostc: commit $(git rev-parse HEAD)"

cargo build --release -p nilestream-server --bin nilestreamd
cargo build --release -p comparator

started=$(date +%s)
for series in single multi; do
    ./target/release/comparator run --series "$series" --sizes 1000000 \
        --seeds 1,7,42,100,2024 --arms N,P+ --dir results/E27-hostc --skip-existing \
        --scratch "$HOME/.e27-scratch"
    q1only=""
    if [ "$series" = multi ]; then q1only="--q1-only"; fi
    ./target/release/comparator run --shape p99 $q1only --series "$series" --sizes 1000000 \
        --seeds 1,7,42,100,2024 --arms N,P+ --dir results/E27-p99-hostc --skip-existing \
        --scratch "$HOME/.e27-scratch"
done
echo "e27-hostc: measured in $(( $(date +%s) - started )) s"

out="$HOME/Documents/niles-sync/cycle-14/e27-hostc"
mkdir -p "$out/main" "$out/p99"
cp results/E27-hostc/*.tsv "$out/main/"
cp results/E27-p99-hostc/*.tsv "$out/p99/"
ls -l "$out/main" "$out/p99"
echo "e27-hostc: done; the points are in $out"
