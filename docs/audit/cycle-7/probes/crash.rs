//! Concurrent writers against an EXTERNAL daemon. Each thread posts amt=1 to its own account
//! and records how many inserts were acknowledged. The shell around this kills the daemon
//! with SIGKILL while this is running, restarts it on the same segment, and checks that every
//! acknowledged insert is still there: sum(amt) for the thread's account >= acked.
//! Usage: crash <port> <writers> <seconds> <run_tag>
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let port: u16 = a[1].parse().unwrap();
    let writers: u32 = a[2].parse().unwrap();
    let seconds: u64 = a[3].parse().unwrap();
    let tag: u64 = a[4].parse().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let mut hs = Vec::new();
    for t in 0..writers {
        let stop = stop.clone();
        hs.push(std::thread::spawn(move || {
            let mut c = bank_bench::wire::Client::connect("127.0.0.1", port, "bench", "bank").expect("connect");
            let acct = 5_000 + t as i64;
            let mut acked = 0u64;
            let mut i = 0u64;
            let mut last_err = String::new();
            while !stop.load(Ordering::Relaxed) {
                i += 1;
                let id = 600_000_000u64 + tag * 10_000_000 + (t as u64) * 1_000_000 + i;
                match c.simple_counted(&format!("insert into postings values ({id}, {acct}, 0, 1), ({id}, 9999, 0, -1)")) {
                    Ok(_) => acked += 1,
                    Err(e) => { last_err = format!("{e:?}"); break; }
                }
            }
            (t, acct, acked, i, last_err)
        }));
    }
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(seconds) { std::thread::sleep(Duration::from_millis(50)); }
    stop.store(true, Ordering::Relaxed);
    for h in hs {
        let (t, acct, acked, sent, err) = h.join().unwrap();
        println!("thread {t} acct {acct} acked {acked} sent {sent} err {}", if err.is_empty() { "-".into() } else { err.chars().take(60).collect::<String>() });
    }
}
