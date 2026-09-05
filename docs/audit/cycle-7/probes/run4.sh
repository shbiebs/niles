#!/usr/bin/env bash
# Cycle 7 — Host C run 4. The measurement cycle 6 could not take: T-06 on ten cores.
#
#   bash ~/Documents/niles-hostc/run4.sh
#
# Expected runtime: about 30 minutes. Non-interactive. Writes only under
# ~/Documents/niles-hostc/ and a scratch build directory there. It adds git WORKTREES
# (not checkouts) to ~/Documents/niles so your working copy, its branch, and its four
# untracked files are not touched. Nothing needs PostgreSQL: every arm is Nilestream-only.
#
# What it measures, in order:
#   1. the barrier ceiling on this disk (F_FULLFSYNC), printed by bench itself
#   2. three arms of E19 at 1/2/4/8/12/16 connections, 3 runs each, fold + point + durable:
#        A  after   c6/audit-cycle-7 (d9c8699 engine)   -- T-06 + T-06a
#        B  before  c6/05-unlock     (15425b5 engine)   -- one mutex, before T-06
#        C  t06     c6/06-rwlock-read (d681f0d engine)  -- T-06 without the lock-order fix
#   3. the mixed read/write probe on arm A: 4r/2w and 8r/4w, readers alone / writers alone / both
#   4. RSS of the bench process sampled through arm A
#   5. the T-03 regression: verdict tests with CARGO_TARGET_DIR set, default parallelism
set -u
HOME_DIR="$HOME"
NILES="$HOME_DIR/Documents/niles"
GBS="$HOME_DIR/Documents/GBS"
HC="$HOME_DIR/Documents/niles-hostc"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT="$HC/results4-$STAMP"
mkdir -p "$OUT"
LOG="$OUT/run4.log"
exec > >(tee -a "$LOG") 2>&1
say() { printf '\n### %s\n' "$*"; }
export PATH="$HOME_DIR/.cargo/bin:$PATH"
unset CARGO_TARGET_DIR

say "host"
uname -a
sysctl -n machdep.cpu.brand_string 2>/dev/null
echo "cores: $(sysctl -n hw.ncpu) (perf $(sysctl -n hw.perflevel0.logicalcpu 2>/dev/null), eff $(sysctl -n hw.perflevel1.logicalcpu 2>/dev/null))"
echo "mem:   $(( $(sysctl -n hw.memsize) / 1073741824 )) GiB"
sw_vers 2>/dev/null | tr '\n' ' '; echo
rustc --version; cargo --version
df -h "$NILES" | tail -1
cd "$NILES" && echo "niles HEAD: $(git rev-parse --abbrev-ref HEAD) $(git rev-parse --short HEAD)"
git -C "$NILES" status --porcelain | sed 's/^/  untracked-or-dirty: /'
for f in .DS_Store AGENTS.md thesis/.DS_Store thesis/Niles-Thesis.pdf; do
  printf '  %-26s %s bytes\n' "$f" "$(stat -f %z "$NILES/$f" 2>/dev/null || echo missing)"
done

say "worktrees (added, never checked out in your tree)"
cd "$NILES"
git worktree prune
for pair in "wt-after:c6/audit-cycle-7" "wt-before:c6/05-unlock" "wt-t06:c6/06-rwlock-read"; do
  d="${pair%%:*}"; b="${pair##*:}"
  if [ ! -d "$HC/$d" ]; then git worktree add -q "$HC/$d" "$b" && echo "added $HC/$d at $b"; else echo "reusing $HC/$d"; fi
  echo "  $d = $(git -C "$HC/$d" rev-parse --short HEAD)"
done

say "arm B needs the fold workload, which 15425b5's harness does not have — patching its bench.rs (scratch worktree only)"
python3 - "$HC/wt-after/crates/bank-bench/src/bin/bench.rs" "$HC/wt-before/crates/bank-bench/src/bin/bench.rs" <<'PY'
import sys
new=open(sys.argv[1]).read(); p=sys.argv[2]; old=open(p).read()
if 'workload: "fold"' in old:
    print("already patched"); sys.exit(0)
old=old.replace('for w in ["point", "durable"] {','for w in ["point", "fold", "durable"] {')
blocks=[]; i=0
while True:
    j=new.find('            // **The scan-shaped read', i)
    if j<0: break
    k=new.find('            report_scaling(&s);\n            out.push(s);\n', j)
    k=new.index('\n', k+len('            report_scaling(&s);\n            out.push(s);'))+1
    blocks.append(new[j:k]); i=k
assert len(blocks)==2, len(blocks)
for b, anchor in ((blocks[0], '            let s = workloads::concurrent(\n                workloads::Level {\n                    workload: "durable",\n                    target: "postgres",'),
                  (blocks[1], '            let s = workloads::concurrent(\n                workloads::Level {\n                    workload: "durable",\n                    target: "nilestream",')):
    assert old.count(anchor)==1, anchor
    old=old.replace(anchor, b+anchor,1)
open(p,'w').write(old); print("patched")
PY

say "builds (release, offline; three target dirs so nothing contends)"
for d in wt-after wt-before wt-t06; do
  ( cd "$HC/$d" && CARGO_TARGET_DIR="$HC/target-$d" cargo build --release --offline -p bank-bench --bin bench 2>&1 | tail -1 )
done

say "mixed-workload probe (built against arm A)"
mkdir -p "$HC/probes/mixed/src"
cat > "$HC/probes/mixed/Cargo.toml" <<TOML
[package]
name = "mixed"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
bank-bench = { path = "$HC/wt-after/crates/bank-bench" }
nilestream-server = { path = "$HC/wt-after/crates/nilestream-server" }
proto-engine = { path = "$HC/wt-after/crates/proto-engine" }
TOML
cat > "$HC/probes/mixed/src/main.rs" <<'RS'
//! **The mixed workload, which nothing in the repository measures.**
//!
//! Hosts the durable engine exactly as `bench --host-nls` does, drives it over the wire from
//! reader and writer threads, and reports three phases: readers alone, writers alone, both.
//! Two things the single-workload rows cannot show:
//!
//! * **The anchor-mismatch fallback.** A keyed read whose view entry is stamped later than the
//!   session's anchor falls back to the fold, ~100x the cost. There is no counter for it
//!   (work order 6 asked twice). The proxy here is the read latency distribution: reads that
//!   take more than `FOLD_FACTOR` times the readers-alone p50 are counted as fold-shaped.
//! * **Writer starvation (LC-18).** `std::sync::RwLock` is not writer-preferring everywhere; a
//!   sustained read stream can hold appends off. Append p99/max in the mixed phase against the
//!   writers-alone phase is the number.
//!
//! Usage: mixed <readers> <writers> <accounts> <budget> <seconds> [port]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

const FOLD_FACTOR: f64 = 20.0;

fn pct(v: &mut [u64], p: f64) -> u64 {
    if v.is_empty() {
        return 0;
    }
    v.sort_unstable();
    let rank = ((p / 100.0) * v.len() as f64).ceil() as usize;
    v[rank.clamp(1, v.len()) - 1]
}

struct Phase {
    name: &'static str,
    read_lat: Vec<u64>,
    write_lat: Vec<u64>,
    read_err: u64,
    write_err: u64,
    dup: u64,
    secs: f64,
}

fn run_phase(
    name: &'static str,
    port: u16,
    readers: u32,
    writers: u32,
    accounts: i64,
    seconds: u64,
    txn_seed: u64,
) -> Phase {
    let stop = Arc::new(AtomicBool::new(false));
    let start = Arc::new(Barrier::new((readers + writers + 1) as usize));
    let mut hs = Vec::new();
    let read_out: Arc<std::sync::Mutex<Vec<u64>>> = Default::default();
    let write_out: Arc<std::sync::Mutex<Vec<u64>>> = Default::default();
    let read_err = Arc::new(AtomicU64::new(0));
    let write_err = Arc::new(AtomicU64::new(0));
    let dup = Arc::new(AtomicU64::new(0));

    for t in 0..readers {
        let (stop, start, out, err) = (stop.clone(), start.clone(), read_out.clone(), read_err.clone());
        hs.push(std::thread::spawn(move || {
            let mut c = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank").expect("connect");
            let mut rng = bank_bench::workloads::Rng::seeded(0xA11CE ^ (t as u64) << 8);
            let mut lat = Vec::with_capacity(1 << 16);
            start.wait();
            while !stop.load(Ordering::Relaxed) {
                let k = rng.skewed_key(accounts, 0.9);
                let sql = format!("select acct, sum(amt) from postings where acct = {k} group by acct");
                let at = Instant::now();
                match c.simple_counted(&sql) {
                    Ok(_) => lat.push(at.elapsed().as_micros() as u64),
                    Err(_) => {
                        err.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            out.lock().unwrap().extend(lat);
        }));
    }
    for t in 0..writers {
        let (stop, start, out, err, dup) = (stop.clone(), start.clone(), write_out.clone(), write_err.clone(), dup.clone());
        hs.push(std::thread::spawn(move || {
            let mut c = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank").expect("connect");
            let mut rng = bank_bench::workloads::Rng::seeded(0xB0B ^ (t as u64) << 8);
            let mut lat = Vec::with_capacity(1 << 16);
            let mut i: u64 = 0;
            start.wait();
            while !stop.load(Ordering::Relaxed) {
                i += 1;
                let a = rng.skewed_key(accounts, 0.9);
                let id = 700_000_000u64 + txn_seed * 10_000_000 + (t as u64) * 1_000_000 + i;
                let sql = format!("insert into postings values ({id}, {a}, 0, 0)");
                let at = Instant::now();
                match c.simple_counted(&sql) {
                    Ok(_) => lat.push(at.elapsed().as_micros() as u64),
                    Err(e) => {
                        if format!("{e:?}").contains("already") {
                            dup.fetch_add(1, Ordering::Relaxed);
                        } else {
                            err.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
            out.lock().unwrap().extend(lat);
        }));
    }
    start.wait();
    let t0 = Instant::now();
    std::thread::sleep(Duration::from_secs(seconds));
    stop.store(true, Ordering::Relaxed);
    for h in hs {
        let _ = h.join();
    }
    let read_lat = std::mem::take(&mut *read_out.lock().unwrap());
    let write_lat = std::mem::take(&mut *write_out.lock().unwrap());
    Phase {
        name,
        read_lat,
        write_lat,
        read_err: read_err.load(Ordering::Relaxed),
        write_err: write_err.load(Ordering::Relaxed),
        dup: dup.load(Ordering::Relaxed),
        secs: t0.elapsed().as_secs_f64(),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 6 {
        eprintln!("usage: mixed <readers> <writers> <accounts> <budget> <seconds> [port]");
        std::process::exit(2);
    }
    let readers: u32 = a[1].parse().unwrap();
    let writers: u32 = a[2].parse().unwrap();
    let accounts: i64 = a[3].parse().unwrap();
    let budget: usize = a[4].parse().unwrap();
    let seconds: u64 = a[5].parse().unwrap();
    let port: u16 = a.get(6).and_then(|p| p.parse().ok()).unwrap_or(0);

    let dir = std::env::temp_dir().join(format!("mixed-probe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let seg = dir.join("mixed.seg");
    let engine = nilestream_server::rev_engine::RevEngine::seeded(
        accounts,
        1,
        budget,
        proto_engine::ViewMode::Demand,
        proto_engine::EvictionPolicy::Lru,
    )
    .with_durable(&seg)
    .expect("durable sink");
    let listener = std::net::TcpListener::bind(("127.0.0.1", port)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let engine = Arc::new(engine);
    {
        let e = engine.clone();
        let schema = nilestream_server::daemon::DEFAULT_SCHEMA.to_string();
        std::thread::spawn(move || nilestream_server::daemon::accept_loop(listener, schema, e));
    }
    std::thread::sleep(Duration::from_millis(200));

    println!("mixed-workload probe — {readers} readers, {writers} writers, {accounts} accounts, budget {budget}, {seconds}s per phase, durable");
    println!("cores: {}", std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0));

    // warm the view a little so the readers-alone phase is not all misses
    {
        let mut c = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank").unwrap();
        for k in 0..accounts.min(2000) {
            let _ = c.simple_counted(&format!("select acct, sum(amt) from postings where acct = {k} group by acct"));
        }
    }

    let phases = vec![
        run_phase("readers-alone", port, readers, 0, accounts, seconds, 1),
        run_phase("writers-alone", port, 0, writers, accounts, seconds, 2),
        run_phase("mixed", port, readers, writers, accounts, seconds, 3),
    ];

    let base_read_p50 = {
        let mut v = phases[0].read_lat.clone();
        pct(&mut v, 50.0).max(1)
    };
    let fold_threshold = (base_read_p50 as f64 * FOLD_FACTOR) as u64;

    println!();
    println!("| phase | reads/s | read p50 | read p99 | read max | fold-shaped (> {fold_threshold}us) | writes/s | write p50 | write p99 | write max | dup | err |");
    println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
    for p in &phases {
        let mut r = p.read_lat.clone();
        let mut w = p.write_lat.clone();
        let fold_shaped = r.iter().filter(|&&x| x > fold_threshold).count();
        let fold_pct = if r.is_empty() { 0.0 } else { 100.0 * fold_shaped as f64 / r.len() as f64 };
        println!(
            "| {} | {:.0} | {} | {} | {} | {} ({:.2}%) | {:.0} | {} | {} | {} | {} | {} |",
            p.name,
            r.len() as f64 / p.secs,
            pct(&mut r, 50.0),
            pct(&mut r, 99.0),
            r.iter().max().copied().unwrap_or(0),
            fold_shaped,
            fold_pct,
            w.len() as f64 / p.secs,
            pct(&mut w, 50.0),
            pct(&mut w, 99.0),
            w.iter().max().copied().unwrap_or(0),
            p.dup,
            p.read_err + p.write_err,
        );
    }

    // the lock and sealer counters, after everything
    let mut c = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank").unwrap();
    if let Ok(rows) = c.simple("select nilestream_sealer") {
        println!();
        println!("sealer/lock after all phases:");
        for (col, val) in rows.columns.iter().zip(rows.rows.first().cloned().unwrap_or_default()) {
            println!("  {col:<22} {}", val.unwrap_or_default());
        }
    }
    if let Ok(rows) = c.simple("select nilestream_stats") {
        println!("read-model stats:");
        for (col, val) in rows.columns.iter().zip(rows.rows.first().cloned().unwrap_or_default()) {
            println!("  {col:<22} {}", val.unwrap_or_default());
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
RS
( cd "$HC/probes/mixed" && CARGO_TARGET_DIR="$HC/target-probe" cargo build --release --offline 2>&1 | tail -1 )

run_arm() {
  local name="$1" dir="$2"
  say "E19 arm $name — $dir — 1/2/4/8/12/16 connections, 3 runs"
  mkdir -p "$OUT/$name"
  ( cd "$HC/$dir" && "$HC/target-$dir/release/bench" --run --scaling-only --nls-only --host-nls \
      --connections 1,2,4,8,12,16 --runs 3 --out "$OUT/$name" 2>&1 \
      | grep -vE "^nilestreamd: 127" ) &
  local bpid=$!
  if [ "$name" = "A-after" ]; then
    ( while kill -0 $bpid 2>/dev/null; do
        pgrep -f "release/bench --run" | head -1 | xargs -I{} ps -o rss= -p {} 2>/dev/null
        sleep 1
      done ) > "$OUT/$name/rss-kib.txt" 2>/dev/null &
  fi
  wait $bpid
  echo "--- $name summary:"
  grep -E "^\| (fold|point|durable) \| nilestream" "$OUT/$name/E19-scaling/E19-scaling.md" 2>/dev/null
}
run_arm A-after  wt-after
run_arm B-before wt-before
run_arm C-t06    wt-t06

say "RSS through arm A (KiB): min / p50 / max"
sort -n "$OUT/A-after/rss-kib.txt" 2>/dev/null | awk '{a[NR]=$1} END{if(NR>0) printf "%d / %d / %d over %d samples\n", a[1], a[int(NR/2)+1], a[NR], NR}'

say "mixed probe on arm A — 4 readers / 2 writers"
"$HC/target-probe/release/mixed" 4 2 10000 2500 8 2>&1 | grep -v "^nilestreamd" | tee "$OUT/mixed-4r2w.txt"
say "mixed probe on arm A — 8 readers / 4 writers"
"$HC/target-probe/release/mixed" 8 4 10000 2500 8 2>&1 | grep -v "^nilestreamd" | tee "$OUT/mixed-8r4w.txt"
say "mixed probe on arm A — 8 readers / 1 writer (starvation shape)"
"$HC/target-probe/release/mixed" 8 1 10000 2500 8 2>&1 | grep -v "^nilestreamd" | tee "$OUT/mixed-8r1w.txt"

say "T-03 regression — verdict tests WITH CARGO_TARGET_DIR set, default parallelism"
( cd "$HC/wt-after" && CARGO_TARGET_DIR="$HC/target-t03" cargo test --release --offline -p bank-bench --test counterproposal 2>&1 | grep -E "^test result|FAILED|BLOCKED" )
( cd "$GBS" && NILES_ROOT="$HC/wt-after" CARGO_TARGET_DIR="$HC/target-t03-gbs" cargo test --release --offline -p gbs-products --test niles_schema 2>&1 | grep -E "^test result|FAILED|BLOCKED" )

say "done — results under $OUT ; paste this whole log back"
echo "$OUT"
