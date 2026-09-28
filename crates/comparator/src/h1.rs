//! **H1 — ReadySet in front of PostgreSQL** (cycle 14, R2-03; decision D-6).
//!
//! ReadySet is the maintained descendant of Noria, the partially-stateful dataflow system the
//! thesis builds on: its *deep* caches are dataflow views whose state is filled on demand by
//! upqueries and kept current from PostgreSQL's logical replication stream. It is used here
//! under its licence, **BSL 1.1**, as non-production benchmark use (release `stable-260924`,
//! 1.31.0; binary unpacked from the `.deb` the author approved, SHA-256 `db221fd4…5696`).
//!
//! # What is cached, and what is proxied
//!
//! A deep cache is created for every question whose shape ReadySet accepts (measured
//! 2026-09-28 on this build): **q1, q4, q5, q6**. q2 (`epoch <= $3`) and q3 (`value_day
//! between $3 and $4`) are refused by `CREATE DEEP CACHE` with "Query contains placeholders in
//! unsupported positions", so ReadySet proxies them to PostgreSQL, which answers them exactly
//! at its head. The load verifies the routing of each question with `EXPLAIN LAST STATEMENT`
//! and records it; a question that is proxied is labelled so in E27, because its latency is
//! PostgreSQL's plus a hop, not the cache's.
//!
//! # Exactness: every answer is tagged with the replica's position
//!
//! ReadySet applies the replication stream asynchronously, so an answer can be *stale* —
//! right for an earlier point of PostgreSQL's history — without being *wrong*. The spec asks
//! that each answer be compared against PostgreSQL at the position it was computed at. The
//! position is not on the reply, so it is bracketed: ReadySet's minimum applied replication
//! offset is read just before the read and its maximum just after (`SHOW READYSET STATUS`,
//! outside the timed region), and each measured write records PostgreSQL's WAL position just
//! before and just after it (`pg_current_wal_insert_lsn()` on a direct connection). The oracle
//! epochs the replica **must** have applied are those whose after-position is at or below the
//! first offset; those it **may** have applied are those whose before-position is below the
//! second. An answer is then:
//!
//! * **exact and fresh** — equal to the oracle at the head;
//! * **exact and stale** — equal to the oracle at some epoch in the bracket, not at the head;
//!   counted as `stale_reads`, with the lag in epochs, and *not* a divergence;
//! * **inexact** — equal to the oracle at no epoch in the bracket: a wrong answer. Counted as
//!   `inexact_reads` and as an oracle divergence, which refuses H1's comparisons at that point.
//!
//! H1's inexactness rate is `inexact_reads` over reads.
//!
//! # Credentials
//!
//! ReadySet takes the upstream as a URL that includes the password, and uses the same user
//! and password to authenticate its own clients (SCRAM-SHA-256, its default). The URL is
//! built in this process from the harness's password (`bank_bench::scram::password`, the
//! untracked file `tools/pg-provision.sh` wrote) and handed to ReadySet **in its environment**
//! (`UPSTREAM_DB_URL`). It is never on a command line, in a log, or in a file, and the load
//! checks that ReadySet's log does not contain it. ReadySet needs the harness role to hold
//! `REPLICATION` on H1's cluster, and a publication it cannot create itself (the role is not a
//! superuser); both are granted on H1's cluster only, by the cluster superuser over the local
//! socket.

use crate::arms::{
    load_base, Arm, LoadReport, NotRun, Query, COMMON_SQL, IMMUTABLE_SQL, POST_TXN_SQL,
};
use crate::pgcluster::{pss_of, Cluster};
use crate::universe::{Leg, Txn, Universe};
use bank_bench::wire::{Client, Rows};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Where the unpacked ReadySet binary is: `READYSET_BIN`, else `/opt/arms/readyset`.
pub fn binary() -> PathBuf {
    std::env::var("READYSET_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/arms/readyset/usr/bin/readyset"))
}

/// The deep caches H1 creates, by question. Their texts are the ones [`H1Arm::sql`] sends,
/// with literals in place of the placeholders; ReadySet matches a literal query to the cache
/// whose parameterised form it has.
pub const CACHES: &[(&str, &str)] = &[
    (
        "q1",
        "select sum(amt) from arm.postings where acct = $1 and cur = $2",
    ),
    (
        "q4",
        "select cur, sum(amt) from arm.postings where desk = $1 group by cur",
    ),
    (
        "q5",
        "select acct, sum(amt) as b from arm.postings where cur = $1 group by acct order by b desc limit 10",
    ),
    (
        "q6",
        "select acct, cur, sum(amt) from arm.postings group by acct, cur",
    ),
];

/// A PostgreSQL WAL position, `X/Y` hexadecimal, as one number.
pub fn parse_lsn(s: &str) -> Option<u64> {
    let (hi, lo) = s.trim().split_once('/')?;
    Some((u64::from_str_radix(hi, 16).ok()? << 32) | u64::from_str_radix(lo, 16).ok()?)
}

/// The applied position in a ReadySet offset pair `(0/443C0F8, 0/443C128)`: the pair is the
/// last applied transaction's (commit LSN, end LSN) — measured 2026-09-28: after one
/// `post_txn`, `pg_current_wal_insert_lsn()` was `0/443C128`, ReadySet's pair
/// `(0/443C0F8, 0/443C128)`, 48 bytes apart, one commit record. The end LSN is the one
/// comparable with a WAL position taken after a commit.
fn applied_of_pair(s: &str) -> Option<u64> {
    parse_lsn(s.trim().trim_end_matches(')').split(',').nth(1)?)
}

pub struct H1Arm {
    pub cluster: Cluster,
    pub port: u16,
    pub dir: PathBuf,
    pub repo: PathBuf,
    /// ReadySet's `--memory-limit`, bytes; `None` runs it without one.
    pub memory_limit: Option<u64>,
    /// No append-only trigger on the base: `H1M`, the mutable twin used only by the anomaly
    /// probe's mutating mode (as M+ is to P+).
    pub mutable: bool,
    child: Option<Child>,
    /// A direct connection to H1's PostgreSQL, for WAL positions.
    upstream: Mutex<Option<Client>>,
    /// A connection to ReadySet for `SHOW READYSET STATUS`.
    status: Mutex<Option<Client>>,
    /// Oracle epoch index → (WAL position just before its write, just after).
    positions: Mutex<BTreeMap<u64, (u64, u64)>>,
    /// The oracle head when the load finished; every epoch through it is applied (the load
    /// waits for that).
    loaded_head: u64,
    /// Which questions ReadySet served from a cache after the load, and which it proxied.
    pub routing: BTreeMap<String, String>,
}

impl H1Arm {
    pub fn new(pg_port: u16, port: u16, dir: PathBuf, repo: &Path) -> H1Arm {
        H1Arm {
            cluster: Cluster::new("h1", pg_port),
            port,
            dir,
            repo: repo.to_path_buf(),
            memory_limit: None,
            mutable: false,
            child: None,
            upstream: Mutex::new(None),
            status: Mutex::new(None),
            positions: Mutex::new(BTreeMap::new()),
            loaded_head: 0,
            routing: BTreeMap::new(),
        }
    }

    fn upstream_client(&self) -> Result<Client, String> {
        Client::connect("127.0.0.1", self.cluster.port, "bench", "bank").map_err(|e| e.to_string())
    }

    fn wal_position(&self) -> Result<u64, String> {
        let mut g = self.upstream.lock().unwrap();
        if g.is_none() {
            *g = Some(self.upstream_client()?);
        }
        let rows = g
            .as_mut()
            .unwrap()
            .simple("select pg_current_wal_insert_lsn()::text")
            .map_err(|e| e.to_string())?;
        rows.rows
            .first()
            .and_then(|r| r.first().cloned().flatten())
            .and_then(|s| parse_lsn(&s))
            .ok_or_else(|| "pg_current_wal_lsn() unreadable".to_string())
    }

    /// (minimum, maximum) replication offset ReadySet reports as applied.
    fn offsets(&self) -> Result<(u64, u64), String> {
        let mut g = self.status.lock().unwrap();
        if g.is_none() {
            *g = Some(self.connect()?);
        }
        let rows = g
            .as_mut()
            .unwrap()
            .simple("show readyset status")
            .map_err(|e| e.to_string())?;
        let get = |name: &str| {
            rows.rows
                .iter()
                .find(|r| r.first().cloned().flatten().as_deref() == Some(name))
                .and_then(|r| r.get(1).cloned().flatten())
                .and_then(|v| applied_of_pair(&v))
        };
        match (
            get("Minimum Replication Offset"),
            get("Maximum Replication Offset"),
        ) {
            (Some(a), Some(b)) => Ok((a, b)),
            _ => Err("SHOW READYSET STATUS has no replication offsets".into()),
        }
    }

    fn wait_until(
        &self,
        what: &str,
        secs: u64,
        mut ok: impl FnMut() -> bool,
    ) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(secs);
        while !ok() {
            if Instant::now() > deadline {
                return Err(format!(
                    "ReadySet: timed out after {secs}s waiting for {what}"
                ));
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        Ok(())
    }

    fn kill(&mut self) {
        *self.status.lock().unwrap() = None;
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

fn first_cell(rows: &Rows) -> Option<String> {
    rows.rows.first().and_then(|r| r.first().cloned().flatten())
}

impl Arm for H1Arm {
    fn name(&self) -> &'static str {
        if self.mutable {
            "H1M"
        } else {
            "H1"
        }
    }
    fn describe(&self) -> String {
        format!(
            "ReadySet stable-260924 (1.31.0; BSL 1.1, non-production benchmark use) on 127.0.0.1:{} \
             in front of PostgreSQL 16 (own cluster on 127.0.0.1:{}, SCRAM-SHA-256 as role bench, \
             {}); deep caches for q1, q4, q5, q6, q2 and q3 proxied to \
             PostgreSQL; {}; every answer checked against the oracle at the replica's bracketed \
             position",
            self.port,
            self.cluster.port,
            if self.mutable {
                "no append-only trigger: the probe's mutable twin"
            } else {
                "append-only trigger"
            },
            match self.memory_limit {
                Some(m) => format!("--memory-limit {m} bytes"),
                None => "no --memory-limit (ReadySet evicts by process memory, not by key count)".into(),
            }
        )
    }
    fn connect(&self) -> Result<Client, String> {
        Client::connect("127.0.0.1", self.port, "bench", "bank").map_err(|e| e.to_string())
    }
    fn supports(&self, _q: &Query) -> Result<(), NotRun> {
        Ok(())
    }
    fn sql(&self, q: &Query) -> String {
        match q {
            Query::Point((a, c)) => {
                format!("select sum(amt) from arm.postings where acct = {a} and cur = {c}")
            }
            Query::Anchored((a, c), e) => format!(
                "select sum(amt) from arm.postings where acct = {a} and cur = {c} and epoch <= {e}"
            ),
            Query::Statement((a, c), d0, d1) => format!(
                "select count(*), sum(amt) from arm.postings where acct = {a} and cur = {c} and value_day between {d0} and {d1}"
            ),
            Query::Desk(d) => {
                format!("select cur, sum(amt) from arm.postings where desk = {d} group by cur")
            }
            Query::Top10(c) => format!(
                "select acct, sum(amt) as b from arm.postings where cur = {c} group by acct order by b desc limit 10"
            ),
            Query::Extract => "select acct, cur, sum(amt) from arm.postings group by acct, cur".into(),
        }
    }
    fn write_sql(&self, idx: u64, t: &Txn) -> Vec<String> {
        let [a, b] = t.legs;
        vec![format!(
            "select arm.post_txn({idx}, {}, {}, {}, {}::smallint, {}, {})",
            t.id, a.acct, b.acct, a.cur, b.amt, t.value_day
        )]
    }
    fn before_write(&self, idx: u64) -> Result<(), String> {
        let p = self.wal_position()?;
        self.positions.lock().unwrap().insert(idx, (p, u64::MAX));
        Ok(())
    }
    fn after_write(&self, idx: u64) -> Result<(), String> {
        let p = self.wal_position()?;
        if let Some(e) = self.positions.lock().unwrap().get_mut(&idx) {
            e.1 = p;
        }
        Ok(())
    }
    fn replica_offsets(&self) -> Option<Result<(u64, u64), String>> {
        Some(self.offsets())
    }
    fn log_position(&self) -> Option<u64> {
        self.wal_position().ok()
    }
    fn epochs_between(&self, lo: u64, hi: u64) -> (u64, u64) {
        let pos = self.positions.lock().unwrap();
        let must = pos
            .iter()
            .filter(|(_, &(_, after))| after <= lo)
            .map(|(i, _)| *i)
            .max()
            .unwrap_or(self.loaded_head)
            .max(self.loaded_head);
        let may = pos
            .iter()
            .filter(|(_, &(before, _))| before < hi)
            .map(|(i, _)| *i)
            .max()
            .unwrap_or(self.loaded_head)
            .max(must);
        (must, may)
    }
    fn load(&mut self, u: &Universe, _budget: usize) -> Result<LoadReport, String> {
        let t0 = Instant::now();
        self.kill();
        *self.upstream.lock().unwrap() = None;
        self.positions.lock().unwrap().clear();
        self.routing.clear();
        self.cluster.up(&self.repo)?;
        // A previous point's replication slot and publication belong to a schema that is
        // about to be dropped: remove them first, as the superuser.
        self.cluster.admin_sql(
            "bank",
            "select pg_drop_replication_slot(slot_name) from pg_replication_slots where database = 'bank'; \
             drop publication if exists readyset",
        )?;
        let mut c = self.upstream_client()?;
        if c.auth != bank_bench::wire::AuthMethod::ScramSha256 {
            return Err("the cluster did not authenticate with SCRAM-SHA-256".into());
        }
        let run = |c: &mut Client, sql: &str| c.simple(sql).map(|_| ()).map_err(|e| e.to_string());
        run(&mut c, COMMON_SQL)?;
        if !self.mutable {
            run(&mut c, IMMUTABLE_SQL)?;
        }
        run(&mut c, POST_TXN_SQL)?;
        let mut statements = 3 + load_base(&mut c, u)?;
        run(&mut c, "vacuum analyze")?;
        statements += 1;
        self.cluster.admin_sql(
            "bank",
            "alter role bench replication; create publication readyset for tables in schema arm",
        )?;

        let _ = std::fs::remove_dir_all(&self.dir);
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let password = bank_bench::scram::password().map_err(|e| format!("{e:?}"))?;
        let url = format!(
            "postgresql://bench:{password}@127.0.0.1:{}/bank",
            self.cluster.port
        );
        let log = self.dir.join("readyset.log");
        let mut cmd = Command::new(binary());
        cmd.env("UPSTREAM_DB_URL", &url)
            .args([
                "--address",
                &format!("127.0.0.1:{}", self.port),
                "--metrics-address",
                &format!("127.0.0.1:{}", self.port + 1000),
                "--storage-dir",
                &self.dir.display().to_string(),
                "--log-file",
                &log.display().to_string(),
                "--replication-tables",
                "arm.*",
                "--cache-mode",
                "deep",
                "--disable-create-publication",
                "--disable-setup-ddl-replication",
                "--disable-telemetry",
                "--no-color",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(m) = self.memory_limit {
            cmd.args(["--memory-limit", &m.to_string()]);
        }
        drop(url);
        self.child = Some(
            cmd.spawn()
                .map_err(|e| format!("spawn {}: {e}", binary().display()))?,
        );

        // Up, snapshotted, replicating.
        self.wait_until("the listener", 60, || self.connect().is_ok())?;
        let mut rs = self.connect()?;
        self.wait_until("the snapshot of arm.*", 1800, || {
            rs.simple("show readyset tables")
                .map(|r| {
                    !r.rows.is_empty()
                        && r.rows
                            .iter()
                            .all(|row| row.get(1).cloned().flatten().as_deref() == Some("Online"))
                })
                .unwrap_or(false)
        })?;
        for (_, text) in CACHES {
            rs.simple(&format!("create deep cache from {text}"))
                .map_err(|e| format!("create deep cache from {text}: {e}"))?;
        }
        // Every loaded leg is in the snapshot, which ReadySet took after the load committed.
        // Checked rather than assumed: the cached q6 must equal the universe's balances. (A
        // WAL-position barrier does not work here: ReadySet's applied offset advances only on
        // commits to the published tables, and the load is followed by WAL — the publication,
        // vacuum — that is not.)
        let mut want: BTreeMap<(u64, u32), i128> = BTreeMap::new();
        for t in &u.txns {
            for l in t.legs {
                *want.entry((l.acct, l.cur)).or_default() += l.amt as i128;
            }
        }
        let want = crate::arms::Answer::Report(want);
        self.wait_until("the cached q6 to equal the loaded balances", 600, || {
            rs.simple(&self.sql(&Query::Extract))
                .ok()
                .and_then(|r| crate::arms::parse(&Query::Extract, &r).ok())
                .is_some_and(|a| a == want)
        })?;
        self.loaded_head = u.batches();

        // Which questions are served from a cache: asked once each, then EXPLAIN LAST STATEMENT.
        let probe_key = u
            .txns
            .first()
            .map(|t| (t.legs[0].acct, t.legs[0].cur))
            .unwrap_or((1, 0));
        for q in [
            Query::Point(probe_key),
            Query::Anchored(probe_key, 1),
            Query::Statement(probe_key, 0, 10),
            Query::Desk(0),
            Query::Top10(0),
            Query::Extract,
        ] {
            rs.simple(&self.sql(&q)).map_err(|e| e.to_string())?;
            let dest = rs
                .simple("explain last statement")
                .ok()
                .and_then(|r| first_cell(&r))
                .unwrap_or_else(|| "unknown".into());
            self.routing.insert(q.id().into(), dest);
        }
        // The password must not have reached ReadySet's log.
        if let Ok(text) = std::fs::read_to_string(&log) {
            if text.contains(&password) {
                return Err("ReadySet wrote the upstream password to its log — refused".into());
            }
        }
        Ok(LoadReport {
            seconds: t0.elapsed().as_secs_f64(),
            statements,
            notes: vec![
                "PostgreSQL loaded as on P (one multi-row insert per batch, epochs sealed in order); \
                 publication `readyset` for schema arm; ReadySet snapshotted arm.*, and its cached \
                 q6 was checked equal to the loaded balances before the first read"
                    .into(),
                format!(
                    "routing after the load (EXPLAIN LAST STATEMENT): {}",
                    self.routing
                        .iter()
                        .map(|(q, d)| format!("{q} → {d}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ],
        })
    }
    fn legs(&self, c: &mut Client) -> Result<Vec<Leg>, String> {
        let rows = c
            .simple("select acct, cur, amt from arm.postings")
            .map_err(|e| e.to_string())?;
        crate::arms::legs_from(&rows)
    }
    fn pss(&self) -> Option<u64> {
        let rs = pss_of(self.child.as_ref()?.id())?;
        Some(rs + self.cluster.pss_bytes().unwrap_or(0))
    }
    fn state(&self, c: &mut Client) -> Vec<(String, i128)> {
        let mut out = Vec::new();
        if let Some(p) = self.child.as_ref().and_then(|ch| pss_of(ch.id())) {
            out.push(("readyset process PSS".into(), p as i128));
        }
        if let Some(p) = self.cluster.pss_bytes() {
            out.push(("postgres cluster PSS".into(), p as i128));
        }
        if let Ok(rows) = c.simple("select pg_total_relation_size('arm.postings')") {
            if let Some(v) = rows.nth(0) {
                out.push(("bytes arm.postings".into(), v));
            }
        }
        out
    }
    fn at_head(&self, c: &mut Client, head: u64) -> Result<bool, String> {
        let rows = c
            .simple("select max(id) from arm.epochs")
            .map_err(|e| e.to_string())?;
        Ok(rows.nth(0) == Some(head as i128))
    }
    fn stop(&mut self) {
        self.kill();
        *self.upstream.lock().unwrap() = None;
        self.cluster.down();
    }
}

impl Drop for H1Arm {
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wal_positions_parse_and_order() {
        assert_eq!(parse_lsn("0/192D068"), Some(0x192D068));
        assert_eq!(parse_lsn("1/0"), Some(1 << 32));
        assert!(parse_lsn("1/0").unwrap() > parse_lsn("0/FFFFFFFF").unwrap());
        assert_eq!(applied_of_pair("(0/443C0F8, 0/443C128)"), Some(0x443C128));
        assert_eq!(parse_lsn("garbage"), None);
    }

    #[test]
    fn the_bracket_is_the_epochs_that_must_and_may_have_been_applied() {
        let arm = H1Arm::new(1, 2, PathBuf::from("/nonexistent"), Path::new("."));
        let mut a = arm;
        a.loaded_head = 50;
        {
            let mut p = a.positions.lock().unwrap();
            p.insert(51, (100, 200));
            p.insert(52, (300, 400));
            p.insert(53, (500, 600));
        }
        // Replica at 250..250: 51 is surely applied (after 200 ≤ 250); 52 began at 300 > 250.
        assert_eq!(a.epochs_between(250, 250), (51, 51));
        // Replica between 150 and 450: 51 only may be applied at the low end; up to 52 at the
        // high end (52 began at 300 < 450; 53 began at 500).
        assert_eq!(a.epochs_between(150, 450), (50, 52));
        // Nothing past the load applied yet.
        assert_eq!(a.epochs_between(50, 90), (50, 50));
    }
}
