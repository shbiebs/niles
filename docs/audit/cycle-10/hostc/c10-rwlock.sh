#!/usr/bin/env bash
# c10-rwlock.sh — is std::sync::RwLock writer-preferring on this machine?
#
# WHY
#   C9-06.2's slowest-16 tables say keyed reads wait on the BASE lock. The mechanism the
#   audit inferred is: a reader holds the base read guard across a fold; the appender queues
#   for the write guard behind it; and every reader that arrives while the appender is queued
#   waits too, because the lock is writer-preferring. The Linux std implementation measured
#   that way in the container (143/200 trials admitted the second reader AFTER the queued
#   writer). Host C is Darwin, and Rust's RwLock there is a different implementation. This
#   measures it rather than assuming it.
#
# WHAT IT DOES
#   Writes a 60-line std-only Rust program to a scratch directory OUTSIDE the tree, builds it
#   with the pinned toolchain, runs it, prints the result, and removes nothing you own. No
#   network, no dependencies, nothing installed. Runtime: under one minute.
#
#   bash c10-rwlock.sh

set -u
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/c10-rwlock.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT
mkdir -p "$SCRATCH/src"
cat > "$SCRATCH/Cargo.toml" <<'EOF'
[package]
name = "rwprobe"
version = "0.1.0"
edition = "2021"
[dependencies]
EOF
cat > "$SCRATCH/src/main.rs" <<'EOF'
use std::sync::{Arc, Barrier, RwLock};
use std::time::{Duration, Instant};
fn spin(us: u64) { let t = Instant::now(); while t.elapsed() < Duration::from_micros(us) {} }
fn main() {
    let mut writer_first = 0; let mut reader_first = 0; let mut waits = Vec::new();
    for _ in 0..200 {
        let l = Arc::new(RwLock::new(0u64));
        let b = Arc::new(Barrier::new(3));
        let order = Arc::new(std::sync::Mutex::new(Vec::<&'static str>::new()));
        let r1 = { let (l, b, o) = (l.clone(), b.clone(), order.clone()); std::thread::spawn(move || {
            let g = l.read().unwrap(); b.wait(); spin(300); o.lock().unwrap().push("r1-release"); drop(g); }) };
        let w = { let (l, b, o) = (l.clone(), b.clone(), order.clone()); std::thread::spawn(move || {
            b.wait(); spin(50); let mut g = l.write().unwrap(); o.lock().unwrap().push("w"); *g += 1; spin(50); drop(g); }) };
        b.wait(); spin(150);
        let t = Instant::now();
        let g = l.read().unwrap(); let waited = t.elapsed();
        order.lock().unwrap().push("r2");
        drop(g);
        r1.join().unwrap(); w.join().unwrap();
        let o = order.lock().unwrap().clone();
        let iw = o.iter().position(|x| *x == "w").unwrap();
        let ir2 = o.iter().position(|x| *x == "r2").unwrap();
        if iw < ir2 { writer_first += 1 } else { reader_first += 1 }
        waits.push(waited.as_micros() as u64);
    }
    waits.sort();
    println!("platform                                     : {} / std::sync::RwLock", std::env::consts::OS);
    println!("second reader admitted AFTER the queued writer : {writer_first}/200  (writer-preferring)");
    println!("second reader admitted BEFORE the queued writer: {reader_first}/200  (reader-preferring)");
    println!("second reader's wait, us                      : p50 {} p90 {} max {}", waits[100], waits[180], waits[199]);
    println!();
    println!("READ IT AS: a majority AFTER means a queued writer blocks later readers here, so a");
    println!("reader holding the base across a fold delays every reader that arrives behind the");
    println!("appender. A majority BEFORE means readers stream past a queued writer and the base");
    println!("wait in the slowest-16 tables needs another explanation.");
}
EOF
echo "=== c10-rwlock: $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(uname -srm) ==="
echo "toolchain: $(RUSTUP_TOOLCHAIN=stable rustc --version 2>&1)"
( cd "$SCRATCH" && RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0 cargo run -q --release --offline 2>&1 )
rc=$?
if [ $rc -ne 0 ]; then echo "NOT RUN: the probe did not build or run (exit $rc)"; exit 1; fi
echo "=== done ==="
