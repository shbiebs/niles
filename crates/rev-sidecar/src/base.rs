//! `nilestream-core`'s `Base`, over a ledger and its change stream: PostgreSQL and its
//! logical replication (H3), or TigerBeetle and its CDC through the line protocol of
//! `tools/arms/tigerbeetle/driver.py` (T).

use bank_bench::wire::Client;
use nilestream_core::rev::{Base, BasePlan, Key, Value};
use nilestream_core::Epoch;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// How many applied epochs' deltas are kept after they are applied. Deltas are needed after
/// application only by a deferred merge, which is off by default (LC-38); the few kept make
/// `deltas_available_from` true rather than zero.
const RETAIN_EPOCHS: u64 = 64;

/// One connection to arm T's driver.
type DriverConn = (BufReader<TcpStream>, TcpStream);

pub struct PgBase {
    plan: BasePlan,
    pub port: u16,
    /// `Some(host:port)` when the ledger is TigerBeetle behind arm T's driver.
    pub driver: Option<String>,
    pool: Mutex<Vec<Client>>,
    dpool: Mutex<Vec<DriverConn>>,
    /// Per epoch, the per-key deltas the stream delivered.
    log: Mutex<BTreeMap<Epoch, Vec<(Key, Value)>>>,
    /// The first epoch whose deltas this base holds: the one after the loaded head, since the
    /// stream starts after the load.
    from: AtomicU64,
    frontier: AtomicU64,
    moved: Condvar,
    moved_lock: Mutex<()>,
    /// Upqueries sent to PostgreSQL, and the base rows they reported.
    pub upqueries: AtomicU64,
    pub rows_read: AtomicU64,
}

impl PgBase {
    pub fn new(plan: BasePlan, port: u16, loaded_head: Epoch) -> PgBase {
        PgBase {
            plan,
            port,
            driver: None,
            pool: Mutex::new(Vec::new()),
            dpool: Mutex::new(Vec::new()),
            log: Mutex::new(BTreeMap::new()),
            from: AtomicU64::new(loaded_head + 1),
            frontier: AtomicU64::new(loaded_head),
            moved: Condvar::new(),
            moved_lock: Mutex::new(()),
            upqueries: AtomicU64::new(0),
            rows_read: AtomicU64::new(0),
        }
    }

    /// Record one committed epoch's deltas (before the view applies them).
    pub fn record(&self, e: Epoch, deltas: Vec<(Key, Value)>) {
        let mut log = self.log.lock().unwrap();
        log.insert(e, deltas);
        let keep_from = e.saturating_sub(RETAIN_EPOCHS);
        while let Some((&first, _)) = log.first_key_value() {
            if first >= keep_from {
                break;
            }
            log.remove(&first);
            self.from.store(first + 1, Ordering::Release);
        }
    }

    /// Publish `e` as the frontier, after the view has applied it, and wake waiting writers.
    pub fn publish(&self, e: Epoch) {
        let _g = self.moved_lock.lock().unwrap();
        self.frontier.store(e, Ordering::Release);
        self.moved.notify_all();
    }

    /// Wait until the frontier is at least `e`, or the timeout passes. Returns the frontier.
    pub fn wait_for(&self, e: Epoch, timeout: Duration) -> Epoch {
        let deadline = std::time::Instant::now() + timeout;
        let mut g = self.moved_lock.lock().unwrap();
        loop {
            let f = self.frontier.load(Ordering::Acquire);
            let now = std::time::Instant::now();
            if f >= e || now >= deadline {
                return f;
            }
            g = self.moved.wait_timeout(g, deadline - now).unwrap().0;
        }
    }

    /// A base whose ledger is arm T's driver.
    pub fn over_driver(plan: BasePlan, addr: &str, loaded_head: Epoch) -> PgBase {
        let mut b = PgBase::new(plan, 0, loaded_head);
        b.driver = Some(addr.to_string());
        b
    }

    /// Send one request line to the driver and read `until` returns true for a line; the
    /// lines read are returned. A connection is taken from the pool and returned to it.
    pub fn driver_lines(
        &self,
        request: &str,
        mut until: impl FnMut(&str) -> bool,
    ) -> Result<Vec<String>, String> {
        let addr = self.driver.as_deref().ok_or("no driver configured")?;
        let pooled = self.dpool.lock().unwrap().pop();
        let (mut r, mut w) = match pooled {
            Some(c) => c,
            None => {
                let s = TcpStream::connect(addr).map_err(|e| format!("driver {addr}: {e}"))?;
                s.set_nodelay(true).map_err(|e| e.to_string())?;
                (BufReader::new(s.try_clone().map_err(|e| e.to_string())?), s)
            }
        };
        w.write_all(format!("{request}\n").as_bytes())
            .map_err(|e| format!("driver: {e}"))?;
        let mut out = Vec::new();
        loop {
            let mut line = String::new();
            if r.read_line(&mut line).map_err(|e| format!("driver: {e}"))? == 0 {
                return Err("the driver closed the connection".into());
            }
            let line = line.trim_end().to_string();
            let done = until(&line);
            out.push(line);
            if done {
                break;
            }
        }
        self.dpool.lock().unwrap().push((r, w));
        Ok(out)
    }

    /// One request, one `OK …` reply; the words after `OK`.
    pub fn driver_request(&self, request: &str) -> Result<Vec<String>, String> {
        let line = self
            .driver_lines(request, |_| true)?
            .pop()
            .unwrap_or_default();
        match line.strip_prefix("OK") {
            Some(rest) => Ok(rest.split_whitespace().map(String::from).collect()),
            None => Err(format!("driver: {line}")),
        }
    }

    fn client(&self) -> Client {
        if let Some(c) = self.pool.lock().unwrap().pop() {
            return c;
        }
        // A base that cannot reach its ledger cannot answer, and `Base::reconstruct` has no
        // error channel: the sidecar stops, and the harness records the arm as failed rather
        // than measuring an answer that was never computed.
        Client::connect("127.0.0.1", self.port, "bench", "bank").unwrap_or_else(|e| {
            panic!(
                "rev-sidecar: cannot reach the ledger on port {}: {e}",
                self.port
            )
        })
    }
}

impl Base for PgBase {
    fn answers(&self) -> &BasePlan {
        &self.plan
    }
    fn frontier(&self) -> Epoch {
        self.frontier.load(Ordering::Acquire)
    }
    /// The upquery. No checkpoints: an upquery at `e` is `epoch <= e` over the key's own
    /// index, as the round-2 order specifies for H3, so its cost is the key's history.
    fn reconstruct(&self, key: &Key, anchor: Epoch) -> (Value, u64) {
        let (a, c) = (key[0], key[1]);
        if self.driver.is_some() {
            // T: the key's balance at the epoch's timestamp, from TigerBeetle's account
            // history — an index lookup that reports one row, not a fold.
            let words = self
                .driver_request(&format!("UP {a} {c} {anchor}"))
                .unwrap_or_else(|e| panic!("rev-sidecar: upquery for {key:?} at {anchor}: {e}"));
            let num = |i: usize| -> i128 {
                words
                    .get(i)
                    .and_then(|w| w.parse().ok())
                    .unwrap_or_else(|| panic!("rev-sidecar: driver replied {words:?}"))
            };
            let (value, n) = (num(0), num(1) as u64);
            self.upqueries.fetch_add(1, Ordering::Relaxed);
            self.rows_read.fetch_add(n, Ordering::Relaxed);
            return (value, n);
        }
        let mut client = self.client();
        let rows = client
            .simple(&format!(
                "select coalesce(sum(amt), 0)::text, count(*)::text from arm.postings \
                 where acct = {a} and cur = {c} and epoch <= {anchor}"
            ))
            .unwrap_or_else(|e| panic!("rev-sidecar: upquery for {key:?} at {anchor}: {e}"));
        self.pool.lock().unwrap().push(client);
        let row = rows.rows.first().expect("an aggregate returns one row");
        let cell = |i: usize| -> i128 {
            row.get(i)
                .cloned()
                .flatten()
                .and_then(|s| s.trim().parse().ok())
                .expect("integer cells")
        };
        let (value, n) = (cell(0), cell(1) as u64);
        self.upqueries.fetch_add(1, Ordering::Relaxed);
        self.rows_read.fetch_add(n, Ordering::Relaxed);
        (value, n)
    }
    fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)> {
        self.log
            .lock()
            .unwrap()
            .get(&e)
            .cloned()
            .unwrap_or_default()
    }
    fn delta_rows_at(&self, e: Epoch) -> u64 {
        self.log
            .lock()
            .unwrap()
            .get(&e)
            .map_or(0, |v| v.len() as u64)
    }
    fn deltas_available_from(&self) -> Epoch {
        self.from.load(Ordering::Acquire)
    }
}
