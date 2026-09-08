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

# **A flag this script does not know is a refusal.** It took none and ignored everything,
# so `bash c10-rwlock.sh --release` or a mistyped path ran the default probe and reported it
# as though it had answered the question that was asked.
DEADLINE=180
while [ $# -gt 0 ]; do
  case "$1" in
    --deadline) DEADLINE="${2:-}"; shift ;;
    *) printf '%s\n' "REFUSED: unknown argument \`$1\`. This probe takes only --deadline <s>."
       printf '%s\n' "         An ignored argument is a run that measured something other than"
       printf '%s\n' "         what was asked for."
       exit 2 ;;
  esac
  shift
done

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
    // **Nearest-rank, stated.** The quantiles were read at `waits[100]` and `waits[180]` of
    // 200 sorted samples and labelled p50 and p90; nearest-rank puts them at index
    // `ceil(q*n) - 1`, which is 99 and 179. One index on a 200-sample distribution is not a
    // large error, and a number published under a name it does not have is still wrong.
    let q = |p: f64| -> u64 { waits[((p * waits.len() as f64).ceil() as usize).max(1) - 1] };
    println!("platform                                     : {} / std::sync::RwLock", std::env::consts::OS);
    // **Arrival is fixed by construction; admission is what varies.** The writer always
    // *arrives* first — it spins 50 us after the barrier and the second reader spins 150 —
    // so a label about arrival order would be reporting the schedule this program wrote.
    // What is measured is which of the two was *admitted* first while the first reader held
    // the lock shared, and that is a property of the implementation.
    println!("arrival order (fixed by this probe)           : writer at ~50us, second reader at ~150us, both while reader 1 holds the lock shared until ~300us");
    println!("second reader ADMITTED after the queued writer : {writer_first}/200  (writer-preferring)");
    println!("second reader ADMITTED before the queued writer: {reader_first}/200  (reader-preferring)");
    println!(
        "second reader's wait, us                      : p50 {} p90 {} max {} (nearest-rank, n={})",
        q(0.50),
        q(0.90),
        waits[waits.len() - 1],
        waits.len()
    );
    println!();
    println!("READ IT AS: a majority AFTER means a queued writer blocks later readers here, so a");
    println!("reader holding the base across a fold delays every reader that arrives behind the");
    println!("appender. A majority BEFORE means readers stream past a queued writer and the base");
    println!("wait in the slowest-16 tables needs another explanation.");
}
EOF
echo "=== c10-rwlock: $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(uname -srm) ==="
# **Both toolchains, labelled.** The header printed one line and called it "toolchain",
# which named the override this script uses and not what the tree asks for — so a reader
# could not tell whether the two agree, and the audit that ran this on two Mac toolchains
# had no line to distinguish them by.
echo "toolchain (tree pin)  : $(rustc --version 2>&1 | head -1)"
echo "toolchain (used here) : $(RUSTUP_TOOLCHAIN=stable rustc --version 2>&1 | head -1)"
echo "                        The probe is built and run with the second. If they differ,"
echo "                        this result is about the second and must be labelled with it."

# **A deadline, because a probe about lock scheduling can be the thing that hangs.** 200
# trials of three threads should take a second or two; without a bound, a scheduling
# pathology here stops the audit instead of reporting one.
( cd "$SCRATCH" && RUSTUP_TOOLCHAIN=stable RUSTUP_AUTO_INSTALL=0 cargo run -q --release --offline 2>&1 ) &
probe=$!
waited=0
while kill -0 "$probe" 2>/dev/null; do
  if [ "$waited" -ge "$DEADLINE" ]; then
    kill -TERM "$probe" 2>/dev/null; sleep 2; kill -KILL "$probe" 2>/dev/null
    wait "$probe" 2>/dev/null
    echo "NOT RUN: the probe passed its ${DEADLINE}s deadline and was killed. A probe that has"
    echo "         to be killed is not a slow measurement, it is no measurement."
    exit 1
  fi
  sleep 1
  waited=$(( waited + 1 ))
done
wait "$probe"
rc=$?
if [ $rc -ne 0 ]; then echo "NOT RUN: the probe did not build or run (exit $rc)"; exit 1; fi
echo "=== done ==="
