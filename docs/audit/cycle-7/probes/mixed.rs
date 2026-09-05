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
