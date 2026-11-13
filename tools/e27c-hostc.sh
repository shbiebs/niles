#!/bin/sh
# E27c on Host C, the author's Mac (cycle 17, C17-12 run 3). The design is
# docs/study/E27c-design.md; its §7.1, committed before this script, is this run's
# pre-registration, and nothing here may contradict it.
#
# What it runs (§7.1 items 1–4): arms N, N'-lazy, P+, H3 and P-inc; single-currency at 10⁵ and
# 10⁶ and multi-currency at 10⁵ (multi-currency at 10⁶ is not run: on 16 GiB it would swap);
# α ∈ {0.6, 1.1}; seeds 1, 7, 42, 100, 2024; for each (α, seed) the main point (E27b's shape,
# with the counted pass), then the p99 point (E27b's targeted shape; q1 only on multi). The
# order is single 10⁵, multi 10⁵, single 10⁶. 60 main points and 60 p99 points in all.
#
# How long: see the estimate the session gave with this script. It is long, so it runs in
# stages: E27C_STAGES picks them (default "single-100000 multi-100000 single-1000000"), and a
# re-run resumes, because a point already written is skipped. Running one stage per night is
# fine; every stage is copied out when it finishes.
#
# The quiet-machine protocol (work order §6.3) is yours to keep: quit Claude and its VM, keep
# the Mac on mains power with other applications closed and no Time Machine backup, and run
# this under caffeinate. The script builds first, then waits up to ten minutes for the
# one-minute load average to fall to 1.5 or below and refuses to start if it does not. Before
# and after every point it records the load averages and the swap in use (`vm.swapusage`) in
# points.tsv; a point during which swap in use grew is marked swap-affected there, and its
# latency and memory figures will not be compared (§7.1 item 7).
#
# What it needs: PostgreSQL 16 binaries (Homebrew `postgresql@16` or Postgres.app; or set
# PG_BIN), the Rust toolchain rust-toolchain.toml pins, and a checkout at this branch's head
# with no local changes (every point records whether the worktree is clean). It creates
# per-arm clusters under ~/.e27-pg (ports 5452, 5455 and 5461), owned by you, with peer auth
# on the socket and SCRAM on TCP; it never edits pg_hba.conf. The daemons use ports 5450,
# 5462 and 5463 and data directories under ~/.e27-scratch.
#
# Usage, from the repository root:   caffeinate -dimsu sh tools/e27c-hostc.sh
# Output: results/E27c-hostc/ (main/a*/, p99/a*/, host.txt, points.tsv), and a copy of all of
# it, with SHA256SUMS, under ~/Documents/niles-sync/cycle-17/runs/e27c-hostc/ for the session
# to read. It renders nothing: E27c's renderer is written after the points (§7.1 item 8).
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ "$(uname -s)" != Darwin ]; then
    echo "e27c-hostc: this is Host C's script and expects macOS; uname says $(uname -s)." >&2
    exit 2
fi

if [ -z "${PG_BIN:-}" ]; then
    for d in /opt/homebrew/opt/postgresql@16/bin /usr/local/opt/postgresql@16/bin \
             /Applications/Postgres.app/Contents/Versions/16/bin; do
        if [ -x "$d/initdb" ]; then PG_BIN=$d; break; fi
    done
fi
if [ -z "${PG_BIN:-}" ] || [ ! -x "$PG_BIN/initdb" ]; then
    echo "e27c-hostc: PostgreSQL 16 not found. Install it (brew install postgresql@16) or set PG_BIN to its bin directory." >&2
    exit 2
fi
case "$("$PG_BIN/postgres" --version)" in
    *" 16."*) ;;
    *) echo "e27c-hostc: $PG_BIN is not PostgreSQL 16: $("$PG_BIN/postgres" --version)" >&2; exit 2 ;;
esac
export PG_BIN
export PATH="$PG_BIN:$PATH"
export E27_PGDATA="$HOME/.e27-pg"

if ! git diff --quiet HEAD -- crates tools/memprobe Cargo.toml Cargo.lock; then
    echo "e27c-hostc: the checkout has local changes under crates/, tools/memprobe or Cargo.*; every point would record MODIFIED. Run it from a clean checkout of this branch." >&2
    exit 2
fi

stages=${E27C_STAGES:-single-100000 multi-100000 single-1000000}
for st in $stages; do
    case "$st" in
        single-100000|multi-100000|single-1000000) ;;
        *) echo "e27c-hostc: unknown stage $st (single-100000, multi-100000 or single-1000000; §7.1 item 1)" >&2; exit 2 ;;
    esac
done

free_gib=$(df -g "$HOME" | awk 'NR==2 {print $4}')
echo "e27c-hostc: ${free_gib} GiB free on the home volume"
case " $stages " in
    *" single-1000000 "*)
        if [ "${free_gib:-0}" -lt 40 ]; then
            echo "e27c-hostc: under 40 GiB free; the 10⁶ stage writes about 2 × 10⁷ legs into three clusters and two daemon segments. Free some space first, or set E27C_STAGES to the 10⁵ stages." >&2
            exit 2
        fi ;;
esac

# Built before any point, so no build runs beside one. The two metered daemons are E27b's
# (tools/memprobe); N and N'-lazy are the same binary.
cargo build --release -p comparator
cargo build --release --manifest-path tools/memprobe/Cargo.toml \
    --bin nilestreamd-metered --bin rev-sidecar-metered

# The one-minute load average, from `sysctl -n vm.loadavg` ("{ 1.23 1.10 0.98 }").
load1() { sysctl -n vm.loadavg | awk '{print $2}'; }
# Swap in use, MiB, from `sysctl -n vm.swapusage` ("total = 2048.00M  used = 0.00M  free = …").
swap_used() { sysctl -n vm.swapusage | sed -n 's/.*used = \([0-9.]*\)M.*/\1/p'; }

waited=0
while awk -v l="$(load1)" 'BEGIN { exit !(l > 1.5) }'; do
    if [ "$waited" -ge 600 ]; then
        echo "e27c-hostc: refused to start: the one-minute load average is $(load1), above 1.5, ten minutes after the build (work order §6.3)." >&2
        echo "e27c-hostc: quit Claude and other applications, wait a few minutes, and run it again (the build is cached)." >&2
        exit 3
    fi
    sleep 30
    waited=$((waited + 30))
done

out=results/E27c-hostc
mkdir -p "$out"
record() {
    {
        echo "$1: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "  machine: $(sysctl -n hw.machine), $(sysctl -n hw.ncpu) CPUs, $(( $(sysctl -n hw.memsize) / 1048576 )) MiB"
        echo "  macOS: $(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
        echo "  load averages: $(sysctl -n vm.loadavg)"
        echo "  swap: $(sysctl -n vm.swapusage)"
        echo "  free on the home volume: $(df -g "$HOME" | awk 'NR==2 {print $4}') GiB"
    } >> "$out/host.txt"
}
if [ ! -f "$out/host.txt" ]; then
    {
        echo "commit: $(git rev-parse HEAD)"
        echo "toolchain: $(cargo --version)"
        echo "postgres: $("$PG_BIN/postgres" --version)"
    } > "$out/host.txt"
fi
if [ ! -f "$out/points.tsv" ]; then
    printf 'shape\talpha\tseries\tsize\tseed\tstarted_utc\tseconds\tload1_before\tload1_after\tswap_used_mib_before\tswap_used_mib_after\tswap_affected\n' > "$out/points.tsv"
fi
record "start ($stages)"
cat "$out/host.txt"

arms="N,N'-lazy,P+,H3,P-inc"
scratch="$HOME/.e27-scratch"

# One point: skipped if written; otherwise run, then one row of points.tsv.
point() {
    shape=$1 alpha=$2 series=$3 size=$4 seed=$5
    dir="$out/$shape/a$alpha"
    file="$dir/$series-$size-$seed.tsv"
    if [ -f "$file" ]; then
        echo "e27c-hostc: skip $file"
        return 0
    fi
    extra=""
    if [ "$shape" = p99 ]; then
        extra="--shape p99-engine"
        if [ "$series" = multi ]; then extra="$extra --q1-only"; fi
    fi
    l0=$(load1) s0=$(swap_used) t0=$(date +%s) d0=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    # shellcheck disable=SC2086 # $extra is a list of flags
    ./target/release/comparator run --study e27c --series "$series" --sizes "$size" \
        --seeds "$seed" --alpha "$alpha" --arms "$arms" $extra --dir "$dir" \
        --skip-existing --scratch "$scratch"
    l1=$(load1) s1=$(swap_used) t1=$(date +%s)
    affected=no
    if awk -v a="$s0" -v b="$s1" 'BEGIN { exit !(b > a) }'; then affected=yes; fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$shape" "$alpha" "$series" \
        "$size" "$seed" "$d0" "$((t1 - t0))" "$l0" "$l1" "$s0" "$s1" "$affected" >> "$out/points.tsv"
    if [ "$affected" = yes ]; then
        echo "e27c-hostc: $file is swap-affected (swap in use $s0 → $s1 MiB); it is kept and will be labelled, not compared"
    fi
}

copy="$HOME/Documents/niles-sync/cycle-17/runs/e27c-hostc"
copy_out() {
    mkdir -p "$copy"
    ( cd "$out" && find . -type f \( -name '*.tsv' -o -name host.txt \) | sed 's|^\./||' | sort |
        while read -r f; do mkdir -p "$copy/$(dirname "$f")"; cp -p "$f" "$copy/$f"; done )
    ( cd "$copy" && find . -type f ! -name SHA256SUMS | sed 's|^\./||' | sort | xargs shasum -a 256 > SHA256SUMS )
    echo "e27c-hostc: copied to $copy ($(wc -l < "$copy/SHA256SUMS") files, SHA256SUMS written)"
}

started=$(date +%s)
for st in $stages; do
    series=${st%-*}
    size=${st#*-}
    echo "e27c-hostc: $(date -u +%FT%TZ) stage $st"
    for alpha in 0.6 1.1; do
        for seed in 1 7 42 100 2024; do
            point main "$alpha" "$series" "$size" "$seed"
            point p99 "$alpha" "$series" "$size" "$seed"
        done
    done
    record "stage $st done"
    copy_out
done
record end
echo "e27c-hostc: measured in $(( $(date +%s) - started )) s"
echo "e27c-hostc: points marked swap-affected: $(awk -F'\t' 'NR > 1 && $12 == "yes"' "$out/points.tsv" | wc -l | tr -d ' ')"
echo "e27c-hostc: points with a failure line: $(grep -l '^failure' "$out"/*/a*/*.tsv 2>/dev/null | wc -l | tr -d ' ')"
echo "e27c-hostc: done"
