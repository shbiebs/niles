//! `nilestream-core`'s `Base`, over PostgreSQL and the replication stream.

use bank_bench::wire::Client;
use nilestream_core::rev::{Base, BasePlan, Key, Value};
use nilestream_core::Epoch;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// How many applied epochs' deltas are kept after they are applied. Deltas are needed after
/// application only by a deferred merge, which is off by default (LC-38); the few kept make
/// `deltas_available_from` true rather than zero.
const RETAIN_EPOCHS: u64 = 64;

pub struct PgBase {
    plan: BasePlan,
    pub port: u16,
    pool: Mutex<Vec<Client>>,
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
            pool: Mutex::new(Vec::new()),
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
