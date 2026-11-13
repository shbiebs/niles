#!/bin/sh
# E27c's history sweep, in the cloud container (cycle 17, C17-12 run 3). The design is
# docs/study/E27c-design.md; its §4 and its §7.1, committed before this script, are this
# run's pre-registration, and nothing here may contradict them.
#
# What it runs (§7.1 items 1–4): arms N, N'-lazy, P+, H3 and P-inc; single-currency at 10⁴
# accounts with 1×, 4×, 16× and 64× E27b's history (10, 40, 160 and 640 transactions per
# account; 1× is the reference); α ∈ {0.6, 1.1}; seeds 1, 7, 42, 100, 2024; for each
# (history, α, seed) the main point (with the counted pass), then the p99 point. Histories
# in increasing order, so the largest, which may not fit, comes last. 80 main and 80 p99
# points in all.
#
# What carries the predictions here is counted: view-state and checkpoint bytes, and base
# rows per reconstruction from the counted pass (§5 predictions 1 and 3). Latency cells are
# subject to this container's noise (results/M-NOISE.md on branch c17/04-m-noise) and are
# reported with that beside them (§7.1 item 1).
#
# E27C_HISTORIES picks histories (default "1 4 16 64"); a re-run resumes, because a point
# already written is skipped. Before and after every point it records the load averages and
# the memory available (`free -m`) in points.tsv.
#
# Usage, from the repository root, as root in the container:
#   RUSTUP_TOOLCHAIN=stable sh tools/e27c-sweep.sh
# Output: results/E27c-sweep/ (main/a*/h*/, p99/a*/h*/, host.txt, points.tsv). It renders
# nothing: E27c's renderer is written after the points (§7.1 item 8).
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ "$(uname -s)" != Linux ]; then
    echo "e27c-sweep: this is the container's script; on Host C use tools/e27c-hostc.sh." >&2
    exit 2
fi
if ! git diff --quiet HEAD -- crates tools/memprobe Cargo.toml Cargo.lock; then
    echo "e27c-sweep: the checkout has local changes under crates/, tools/memprobe or Cargo.*; every point would record MODIFIED." >&2
    exit 2
fi
histories=${E27C_HISTORIES:-1 4 16 64}
for h in $histories; do
    case "$h" in 1|4|16|64) ;; *) echo "e27c-sweep: history $h is not one of 1, 4, 16, 64 (§4)" >&2; exit 2 ;; esac
done

cargo build --release -p comparator
cargo build --release --manifest-path tools/memprobe/Cargo.toml \
    --bin nilestreamd-metered --bin rev-sidecar-metered

load1() { cut -d' ' -f1 /proc/loadavg; }
avail() { free -m | awk '/^Mem:/ {print $7}'; }

out=results/E27c-sweep
mkdir -p "$out"
if [ ! -f "$out/host.txt" ]; then
    {
        echo "commit: $(git rev-parse HEAD)"
        echo "toolchain: $(cargo --version)"
        echo "postgres: $(${PG_BIN:-/usr/lib/postgresql/16/bin}/postgres --version)"
    } > "$out/host.txt"
fi
{
    echo "start (histories $histories): $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "  host: $(uname -srm), $(nproc) CPUs, $(free -m | awk '/^Mem:/ {print $2}') MiB"
    echo "  load averages: $(cut -d' ' -f1-3 /proc/loadavg)"
    echo "  free on /: $(df -BG --output=avail / | tail -1 | tr -d ' ')"
} >> "$out/host.txt"
if [ ! -f "$out/points.tsv" ]; then
    printf 'shape\talpha\thistory_factor\tseries\tsize\tseed\tstarted_utc\tseconds\tload1_before\tload1_after\tavail_mib_before\tavail_mib_after\n' > "$out/points.tsv"
fi
cat "$out/host.txt"

arms="N,N'-lazy,P+,H3,P-inc"
point() {
    shape=$1 alpha=$2 h=$3 seed=$4
    dir="$out/$shape/a$alpha/h$h"
    file="$dir/single-10000-$seed.tsv"
    if [ -f "$file" ]; then
        echo "e27c-sweep: skip $file"
        return 0
    fi
    extra=""
    if [ "$shape" = p99 ]; then extra="--shape p99-engine"; fi
    l0=$(load1) m0=$(avail) t0=$(date +%s) d0=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    # shellcheck disable=SC2086 # $extra is a list of flags
    ./target/release/comparator run --study e27c --series single --sizes 10000 \
        --seeds "$seed" --alpha "$alpha" --history-factor "$h" --arms "$arms" $extra \
        --dir "$dir" --skip-existing --scratch /var/tmp/e27c-sweep
    printf '%s\t%s\t%s\tsingle\t10000\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$shape" "$alpha" "$h" \
        "$seed" "$d0" "$(( $(date +%s) - t0 ))" "$l0" "$(load1)" "$m0" "$(avail)" >> "$out/points.tsv"
}

for h in $histories; do
    echo "e27c-sweep: $(date -u +%FT%TZ) history ${h}×"
    for alpha in 0.6 1.1; do
        for seed in 1 7 42 100 2024; do
            point main "$alpha" "$h" "$seed"
            point p99 "$alpha" "$h" "$seed"
        done
    done
    echo "history ${h}× done: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$out/host.txt"
done
echo "e27c-sweep: points with a failure line: $(grep -l '^failure' "$out"/*/a*/h*/*.tsv 2>/dev/null | wc -l)"
echo "e27c-sweep: done"
