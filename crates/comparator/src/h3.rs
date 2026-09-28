//! **H3 — a PostgreSQL ledger with a REV sidecar** (cycle 14, R2-03; decision D-6).
//!
//! PostgreSQL holds the append-only ledger, loaded exactly as every other PostgreSQL arm's.
//! The derived side is `rev-sidecar` (`crates/rev-sidecar`): a separate process that reads the
//! ledger's logical replication stream (`pgoutput`, publication `h3` on `arm.postings`, a
//! temporary slot created after the load), keeps the balance view as a `nilestream-core` REV
//! — demand-filled, the same residency budget and LRU as N — and answers upqueries with
//! `epoch <= e` against the ledger. q1 and q2 are answered by the REV; q3–q6 and writes are
//! relayed to PostgreSQL by the sidecar, so the harness speaks to one endpoint per arm, as it
//! does to every other.
//!
//! A write is acknowledged by the sidecar only when its epoch has arrived through the stream
//! and been applied to the view, so every read is held to the oracle at the head, exactly as
//! N's and P+'s are: H3 is compared as an exact arm, and what that costs is in its write
//! latency.

use crate::arms::{
    load_base, Arm, LoadReport, NotRun, Query, COMMON_SQL, IMMUTABLE_SQL, POST_TXN_SQL,
};
use crate::pgcluster::{pss_of, Cluster};
use crate::universe::{Leg, Txn, Universe};
use bank_bench::wire::Client;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct H3Arm {
    pub cluster: Cluster,
    pub port: u16,
    pub dir: PathBuf,
    pub repo: PathBuf,
    pub binary: PathBuf,
    child: Option<Child>,
}

impl H3Arm {
    pub fn new(pg_port: u16, port: u16, dir: PathBuf, repo: &Path) -> H3Arm {
        H3Arm {
            cluster: Cluster::new("h3", pg_port),
            port,
            dir,
            repo: repo.to_path_buf(),
            binary: repo.join("target/release/rev-sidecar"),
            child: None,
        }
    }

    fn upstream(&self) -> Result<Client, String> {
        Client::connect("127.0.0.1", self.cluster.port, "bench", "bank").map_err(|e| e.to_string())
    }

    fn kill(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Arm for H3Arm {
    fn name(&self) -> &'static str {
        "H3"
    }
    fn describe(&self) -> String {
        format!(
            "PostgreSQL 16 append-only ledger (own cluster on 127.0.0.1:{}, SCRAM-SHA-256 as role \
             bench, append-only trigger) + `rev-sidecar` on 127.0.0.1:{}: logical replication \
             (pgoutput, commit order as epoch order) into a nilestream-core REV, demand mode, \
             LRU, budget 5% of keys, upquery `epoch <= e` on the ledger's index; q1 and q2 from \
             the REV, q3–q6 relayed to PostgreSQL; a write is acknowledged once its epoch is in \
             the view",
            self.cluster.port, self.port
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
            Query::Point((a, c)) => format!("select value from rev_read({a}, {c})"),
            Query::Anchored((a, c), e) => format!("select value from rev_read({a}, {c}, {e})"),
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
    fn load(&mut self, u: &Universe, budget: usize) -> Result<LoadReport, String> {
        let t0 = Instant::now();
        self.kill();
        if !self.binary.exists() {
            return Err(format!(
                "{} is missing; run `cargo build --release -p rev-sidecar` first",
                self.binary.display()
            ));
        }
        self.cluster.up(&self.repo)?;
        self.cluster
            .admin_sql("bank", "drop publication if exists h3")?;
        let mut c = self.upstream()?;
        if c.auth != bank_bench::wire::AuthMethod::ScramSha256 {
            return Err("the cluster did not authenticate with SCRAM-SHA-256".into());
        }
        let run = |c: &mut Client, sql: &str| c.simple(sql).map(|_| ()).map_err(|e| e.to_string());
        run(&mut c, COMMON_SQL)?;
        run(&mut c, IMMUTABLE_SQL)?;
        run(&mut c, POST_TXN_SQL)?;
        let mut statements = 3 + load_base(&mut c, u)?;
        run(&mut c, "vacuum analyze")?;
        statements += 1;
        self.cluster.admin_sql(
            "bank",
            "alter role bench replication; create publication h3 for table arm.postings",
        )?;
        let _ = std::fs::remove_dir_all(&self.dir);
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let log = std::fs::File::create(self.dir.join("sidecar.log")).map_err(|e| e.to_string())?;
        self.child = Some(
            Command::new(&self.binary)
                .args([
                    "--listen",
                    &format!("127.0.0.1:{}", self.port),
                    "--upstream-port",
                    &self.cluster.port.to_string(),
                    "--budget",
                    &budget.to_string(),
                    "--loaded-head",
                    &u.batches().to_string(),
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(log)
                .spawn()
                .map_err(|e| format!("spawn {}: {e}", self.binary.display()))?,
        );
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if self.connect().is_ok() {
                break;
            }
            if let Some(Ok(Some(status))) = self.child.as_mut().map(|c| c.try_wait()) {
                let msg = std::fs::read_to_string(self.dir.join("sidecar.log")).unwrap_or_default();
                return Err(format!("rev-sidecar exited ({status}): {}", msg.trim()));
            }
            if Instant::now() > deadline {
                return Err("rev-sidecar did not come up within 60 s".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(LoadReport {
            seconds: t0.elapsed().as_secs_f64(),
            statements,
            notes: vec![format!(
                "PostgreSQL loaded as on P (one multi-row insert per batch, epochs sealed in order); \
                 publication `h3` on arm.postings; rev-sidecar started after the load with a \
                 temporary logical slot, the view certified through the loaded head #{} with \
                 nothing resident, budget {budget}",
                u.batches()
            )],
        })
    }
    fn legs(&self, c: &mut Client) -> Result<Vec<Leg>, String> {
        let rows = c
            .simple("select acct, cur, amt from arm.postings")
            .map_err(|e| e.to_string())?;
        crate::arms::legs_from(&rows)
    }
    fn pss(&self) -> Option<u64> {
        let s = pss_of(self.child.as_ref()?.id())?;
        Some(s + self.cluster.pss_bytes().unwrap_or(0))
    }
    fn state(&self, c: &mut Client) -> Vec<(String, i128)> {
        let mut out = Vec::new();
        if let Some(p) = self.child.as_ref().and_then(|ch| pss_of(ch.id())) {
            out.push(("sidecar process PSS".into(), p as i128));
        }
        if let Some(p) = self.cluster.pss_bytes() {
            out.push(("postgres cluster PSS".into(), p as i128));
        }
        if let Ok(rows) = c.simple("select rev_stats") {
            for n in [
                "resident",
                "reads",
                "hits",
                "misses",
                "rows_touched",
                "evictions",
                "view_metadata_keys",
                "view_slots",
            ] {
                if let Some(v) = rows.by_name(n) {
                    out.push((n.to_string(), v));
                }
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
        self.cluster.down();
    }
}

impl Drop for H3Arm {
    fn drop(&mut self) {
        self.kill();
    }
}
