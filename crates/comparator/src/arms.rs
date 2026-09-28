//! The arms: how each one is started, loaded, written to and asked the six questions.
//!
//! Every arm answers the **same** question for the same key at the same epoch, in its own
//! dialect, over the PostgreSQL wire through the same client (`bank_bench::wire`). The texts
//! are here side by side (§5.5), and [`sql_table`] prints them into E27's header so a reader
//! can check that the arms were asked the same thing.

use crate::oracle::{Key, Oracle};
use crate::pgcluster::{pss_of, Cluster};
use crate::universe::{Leg, Txn, Universe};
use bank_bench::wire::{Client, Rows};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The six questions of cycle 13's Part 4 (§5.5). Keys and epochs are the oracle's.
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// q1: one key's balance at the head.
    Point(Key),
    /// q2: one key's balance at a historical epoch index.
    Anchored(Key, u64),
    /// q3: one key's statement — legs and their sum over a value-day window.
    Statement(Key, i32, i32),
    /// q4: desk exposure by currency.
    Desk(u64),
    /// q5: the ten largest balances in a currency.
    Top10(u32),
    /// q6: the regulatory extract — every key's balance.
    Extract,
}

impl Query {
    pub fn id(&self) -> &'static str {
        match self {
            Query::Point(_) => "q1",
            Query::Anchored(..) => "q2",
            Query::Statement(..) => "q3",
            Query::Desk(_) => "q4",
            Query::Top10(_) => "q5",
            Query::Extract => "q6",
        }
    }
    pub const IDS: [&'static str; 6] = ["q1", "q2", "q3", "q4", "q5", "q6"];
    pub fn describe(id: &str) -> &'static str {
        match id {
            "q1" => "point balance at head",
            "q2" => "point balance at a historical anchor",
            "q3" => "one account's month statement",
            "q4" => "desk exposure by currency",
            "q5" => "top-10 by balance",
            "q6" => "regulatory extract (every balance)",
            _ => "?",
        }
    }
}

/// An answer, in the one form every arm and the oracle are compared in.
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Balance(Option<i128>),
    Statement(u64, Option<i128>),
    PerCurrency(BTreeMap<u32, i128>),
    /// The ten largest balances, in the order returned (the oracle's is descending; ties
    /// make the accounts ambiguous, the sequence of balances is not).
    Top(Vec<i128>),
    Report(BTreeMap<Key, i128>),
}

/// What the oracle says the answer is.
pub fn expected(o: &Oracle, q: &Query) -> Answer {
    match q {
        Query::Point(k) => Answer::Balance(o.balance(*k, o.head())),
        Query::Anchored(k, e) => Answer::Balance(o.balance(*k, *e)),
        Query::Statement(k, a, b) => {
            let (n, s) = o.statement(*k, *a, *b);
            Answer::Statement(n, s)
        }
        Query::Desk(d) => Answer::PerCurrency(o.desk(*d)),
        Query::Top10(c) => Answer::Top(o.top10(*c)),
        Query::Extract => Answer::Report(o.report()),
    }
}

/// Why an arm does not run a query: printed as `NOT RUN (<reason>)` in its cells.
pub type NotRun = &'static str;

/// What loading an arm did, for the header.
#[derive(Debug, Clone, Default)]
pub struct LoadReport {
    pub seconds: f64,
    pub statements: u64,
    pub notes: Vec<String>,
}

pub trait Arm: Sync {
    fn name(&self) -> &'static str;
    /// One line for the header: what this arm is.
    fn describe(&self) -> String;
    fn connect(&self) -> Result<Client, String>;
    fn supports(&self, q: &Query) -> Result<(), NotRun>;
    /// The statement this arm is asked for `q`, with the oracle's epoch indexes mapped to the
    /// arm's own numbering.
    fn sql(&self, q: &Query) -> String;
    /// The statements of one write (one round trip each), sealing oracle epoch index `idx`.
    fn write_sql(&self, idx: u64, t: &Txn) -> Vec<String>;
    fn load(&mut self, u: &Universe, budget: usize) -> Result<LoadReport, String>;
    /// Every leg the arm holds, for the checksum.
    fn legs(&self, c: &mut Client) -> Result<Vec<Leg>, String>;
    /// Process memory, bytes (PSS).
    fn pss(&self) -> Option<u64>;
    /// Arm-specific state figures for the memory table: (name, value) pairs.
    fn state(&self, c: &mut Client) -> Vec<(String, i128)>;
    /// Whether the arm's last sealed epoch is the oracle's head — checked after every run,
    /// so a write that sealed zero or two epochs is caught where it happened.
    fn at_head(&self, c: &mut Client, head: u64) -> Result<bool, String>;
    /// The oracle's epoch index for an anchor this arm stamped on a reply.
    fn oracle_index(&self, arm_epoch: u64) -> u64 {
        arm_epoch
    }
    fn stop(&mut self);
}

fn cell(rows: &Rows, r: usize, c: usize) -> Result<Option<i128>, String> {
    let Some(row) = rows.rows.get(r) else {
        return Err(format!("no row {r}"));
    };
    match row.get(c) {
        Some(Some(t)) => t
            .trim()
            .parse::<i128>()
            .map(Some)
            .or_else(|_| {
                // numeric renders with a scale when it has one ("12.00"); an integral numeric
                // is the only kind these arms produce, so a fractional part is an error.
                let (w, f) = t.trim().split_once('.').ok_or(())?;
                if f.chars().all(|c| c == '0') {
                    w.parse::<i128>().map(Some).map_err(|_| ())
                } else {
                    Err(())
                }
            })
            .map_err(|_| format!("cell {r},{c} = {t:?} is not an integer")),
        Some(None) => Ok(None),
        None => Err(format!("no column {c}")),
    }
}

/// Parse a reply into an [`Answer`] by the shape the query asks for. Column positions are
/// fixed by the texts in this file: balance last; statement (count, sum); desk (cur, sum);
/// top (…, sum) descending; report (acct, cur, sum).
pub fn parse(q: &Query, rows: &Rows) -> Result<Answer, String> {
    // The value is the last column that is not Nilestream's `anchor` stamp, which the served
    // engine appends to every reply.
    let last = rows
        .columns
        .iter()
        .rposition(|c| c != "anchor")
        .ok_or("no value column")?;
    Ok(match q {
        Query::Point(_) | Query::Anchored(..) => {
            if rows.rows.is_empty() {
                Answer::Balance(None)
            } else if rows.rows.len() == 1 {
                Answer::Balance(cell(rows, 0, last)?)
            } else {
                return Err(format!("{} rows for one key", rows.rows.len()));
            }
        }
        Query::Statement(..) => {
            let n = cell(rows, 0, 0)?.unwrap_or(0) as u64;
            Answer::Statement(n, cell(rows, 0, 1)?)
        }
        Query::Desk(_) => {
            let mut m = BTreeMap::new();
            for r in 0..rows.rows.len() {
                let c = cell(rows, r, 0)?.ok_or("null currency")? as u32;
                m.insert(c, cell(rows, r, 1)?.ok_or("null sum")?);
            }
            Answer::PerCurrency(m)
        }
        Query::Top10(_) => {
            // In the order the arm returned them: `order by … desc` is part of the question,
            // and an arm that returned the right ten in the wrong order answered wrongly.
            let mut v = Vec::new();
            for r in 0..rows.rows.len() {
                v.push(cell(rows, r, last)?.ok_or("null balance")?);
            }
            Answer::Top(v)
        }
        Query::Extract => {
            let mut m = BTreeMap::new();
            for r in 0..rows.rows.len() {
                let a = cell(rows, r, 0)?.ok_or("null acct")? as u64;
                let c = cell(rows, r, 1)?.ok_or("null cur")? as u32;
                m.insert((a, c), cell(rows, r, 2)?.ok_or("null sum")?);
            }
            Answer::Report(m)
        }
    })
}

fn legs_from(rows: &Rows) -> Result<Vec<Leg>, String> {
    let mut out = Vec::with_capacity(rows.rows.len());
    for r in 0..rows.rows.len() {
        out.push(Leg {
            acct: cell(rows, r, 0)?.ok_or("null acct")? as u64,
            cur: cell(rows, r, 1)?.ok_or("null cur")? as u32,
            amt: cell(rows, r, 2)?.ok_or("null amt")? as i64,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// N — the Nilestream daemon.
// ---------------------------------------------------------------------------------------------

/// Nilestream's served relation, and the reason q3 and q4 cannot be asked of it.
pub const N_NOT_RUN: NotRun =
    "Nilestream's served relation is fixed at (txn, acct, cur, amt, idem) — \
     crates/nilestream-server/src/session.rs parse_insert takes four columns — so it holds no \
     value day and no desk";

pub struct NArm {
    pub port: u16,
    pub dir: PathBuf,
    pub binary: PathBuf,
    pub multi_currency: bool,
    pub checkpoint: usize,
    child: Option<Child>,
    /// Oracle epoch index i is N's epoch `f_start + i`; established by probing during the
    /// load (it is -1 on the unseeded daemon, whose first sealed epoch is #0).
    pub f_start: i64,
}

impl NArm {
    pub fn new(
        port: u16,
        dir: PathBuf,
        binary: PathBuf,
        multi_currency: bool,
        checkpoint: usize,
    ) -> NArm {
        NArm {
            port,
            dir,
            binary,
            multi_currency,
            checkpoint,
            child: None,
            f_start: 0,
        }
    }

    pub fn schema(&self) -> String {
        format!(
            "schema bank {{\n    currency usd {{ scale: 2 }}\n{}    ledger postings {{\n        \
             txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,\n        \
             idem: IdemKey window 1_000_000.epochs,\n        conserve per (txn, cur);\n        \
             retain forever;\n    }}\n    index ix_postings on postings (acct) anchor;\n}}\n",
            if self.multi_currency {
                "    currency eur { scale: 2 }\n"
            } else {
                ""
            }
        )
    }

    fn epoch(&self, idx: u64) -> u64 {
        (self.f_start + idx as i64) as u64
    }

    /// Probe the frontier exactly: a read `as of` e succeeds iff e ≤ frontier (the served
    /// engine refuses a later anchor with 22023, `tests/as_of.rs`).
    pub fn frontier_is(&self, c: &mut Client, f: u64) -> Result<bool, String> {
        let at = |e: u64| {
            format!("select acct, sum(amt) from postings as of system time {e} where acct = 1 and cur = 0 group by acct")
        };
        let ok_at = c.simple(&at(f)).is_ok();
        let beyond = match c.simple(&at(f + 1)) {
            Ok(_) => false,
            Err(bank_bench::wire::WireError::Server { sqlstate, .. }) => sqlstate == "22023",
            Err(e) => return Err(e.to_string()),
        };
        Ok(ok_at && beyond)
    }
}

impl Arm for NArm {
    fn name(&self) -> &'static str {
        "N"
    }
    fn describe(&self) -> String {
        format!(
            "Nilestream daemon (`nilestreamd`, release build) on 127.0.0.1:{}, --durable \
             (SyncPolicy::Always), --mode demand, LRU, --checkpoint {}, budget 5% of keys",
            self.port, self.checkpoint
        )
    }
    fn connect(&self) -> Result<Client, String> {
        Client::connect("127.0.0.1", self.port, "bench", "bank").map_err(|e| e.to_string())
    }
    fn supports(&self, q: &Query) -> Result<(), NotRun> {
        match q {
            Query::Statement(..) | Query::Desk(_) => Err(N_NOT_RUN),
            _ => Ok(()),
        }
    }
    fn sql(&self, q: &Query) -> String {
        let point = |(a, c): Key, at: Option<u64>| {
            let asof = at
                .map(|e| format!(" as of system time {e}"))
                .unwrap_or_default();
            if self.multi_currency {
                format!("select acct, cur, sum(amt) from postings{asof} where acct = {a} and cur = {c} group by acct, cur")
            } else {
                format!("select acct, sum(amt) from postings{asof} where acct = {a} group by acct")
            }
        };
        match q {
            Query::Point(k) => point(*k, None),
            Query::Anchored(k, e) => point(*k, Some(self.epoch(*e))),
            Query::Top10(c) => format!(
                "select acct, sum(amt) from postings where cur = {c} group by acct order by sum(amt) desc limit 10"
            ),
            Query::Extract => "select acct, cur, sum(amt) from postings group by acct, cur".into(),
            Query::Statement(..) | Query::Desk(_) => String::new(),
        }
    }
    fn write_sql(&self, _idx: u64, t: &Txn) -> Vec<String> {
        let [a, b] = t.legs;
        vec![format!(
            "insert into postings values ({}, {}, {}, {}), ({}, {}, {}, {})",
            t.id, a.acct, a.cur, a.amt, t.id, b.acct, b.cur, b.amt
        )]
    }
    fn load(&mut self, u: &Universe, budget: usize) -> Result<LoadReport, String> {
        let t0 = Instant::now();
        let _ = std::fs::remove_dir_all(&self.dir);
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let schema = self.dir.join("schema.niles");
        std::fs::write(&schema, self.schema()).map_err(|e| e.to_string())?;
        let log = std::fs::File::create(self.dir.join("daemon.log")).map_err(|e| e.to_string())?;
        let child = Command::new(&self.binary)
            .args([
                "--port",
                &self.port.to_string(),
                "--schema",
                &schema.display().to_string(),
                "--durable",
                &self.dir.join("segment").display().to_string(),
                "--accounts",
                "0",
                "--rounds",
                "0",
                "--budget",
                &budget.to_string(),
                "--checkpoint",
                &self.checkpoint.to_string(),
                "--mode",
                "demand",
            ])
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .map_err(|e| format!("spawn {}: {e}", self.binary.display()))?;
        self.child = Some(child);
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut c = loop {
            match self.connect() {
                Ok(c) => break c,
                Err(e) if Instant::now() > deadline => {
                    return Err(format!("nilestreamd did not come up: {e}"))
                }
                Err(_) => std::thread::sleep(Duration::from_millis(50)),
            }
        };
        let mut statements = 0;
        let mut batch: Vec<String> = Vec::new();
        let mut current = 1;
        let flush = |c: &mut Client, batch: &mut Vec<String>| -> Result<(), String> {
            if batch.is_empty() {
                return Ok(());
            }
            let sql = format!("insert into postings values {}", batch.join(", "));
            batch.clear();
            c.simple(&sql).map(|_| ()).map_err(|e| e.to_string())
        };
        for t in &u.txns {
            if t.batch != current {
                flush(&mut c, &mut batch)?;
                statements += 1;
                if current == 1 {
                    // The epoch the first batch sealed as, found by probing rather than
                    // assumed: the unseeded daemon reports frontier #0 both before and after
                    // its first append (measured 2026-09-28), so oracle index i is N's
                    // epoch i - 1 — and this probe is what says so on every load.
                    let first = (0..4)
                        .find(|&f| self.frontier_is(&mut c, f).unwrap_or(false))
                        .ok_or("could not establish the epoch of the first batch")?;
                    self.f_start = first as i64 - 1;
                }
                current = t.batch;
            }
            for l in t.legs {
                batch.push(format!("({}, {}, {}, {})", t.id, l.acct, l.cur, l.amt));
            }
        }
        flush(&mut c, &mut batch)?;
        statements += 1;
        let f = self.epoch(u.batches());
        if !self.frontier_is(&mut c, f)? {
            return Err(format!(
                "after {} batches the frontier is not #{f}: one insert statement did not seal exactly one epoch",
                u.batches()
            ));
        }
        Ok(LoadReport {
            seconds: t0.elapsed().as_secs_f64(),
            statements,
            notes: vec![format!(
                "one insert statement per batch, one epoch each; N epoch = oracle index {:+}; frontier #{f} after the load, asserted by an `as of system time` probe",
                self.f_start
            )],
        })
    }
    fn legs(&self, c: &mut Client) -> Result<Vec<Leg>, String> {
        let rows = c
            .simple("select acct, cur, amt from postings")
            .map_err(|e| e.to_string())?;
        legs_from(&rows)
    }
    fn pss(&self) -> Option<u64> {
        pss_of(self.child.as_ref()?.id())
    }
    fn state(&self, c: &mut Client) -> Vec<(String, i128)> {
        let Ok(rows) = c.simple("select nilestream_stats") else {
            return vec![];
        };
        [
            "resident",
            "reads",
            "hits",
            "misses",
            "rows_touched",
            "view_answers",
            "fallbacks",
            "view_metadata_keys",
        ]
        .iter()
        .filter_map(|n| rows.by_name(n).map(|v| (n.to_string(), v)))
        .collect()
    }
    fn at_head(&self, c: &mut Client, head: u64) -> Result<bool, String> {
        self.frontier_is(c, self.epoch(head))
    }
    fn oracle_index(&self, arm_epoch: u64) -> u64 {
        (arm_epoch as i64 - self.f_start) as u64
    }
    fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for NArm {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------------------------
// P, P+, M — PostgreSQL 16, one cluster each.
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgKind {
    /// Ordinary materialised view, refreshed CONCURRENTLY after every write; append-only base.
    P,
    /// The E14 mechanism (repaired, see `sql/pplus.sql`); append-only base.
    PPlus,
    /// P without the append-only trigger — the mutable arm of the anomaly probe.
    M,
    /// P+ without the append-only trigger — the probe's second mutable arm (see run.rs).
    MPlus,
    /// pg_ivm 1.15: the balances view maintained incrementally inside each writing
    /// transaction, fully materialised (R2-03).
    H2,
}

pub struct PgArm {
    pub kind: PgKind,
    pub cluster: Cluster,
    pub repo: PathBuf,
}

const COMMON: &str = include_str!("../sql/common.sql");
const IMMUTABLE: &str = include_str!("../sql/immutable.sql");
const P_SQL: &str = include_str!("../sql/p.sql");
const PPLUS_SQL: &str = include_str!("../sql/pplus.sql");
const H2_SQL: &str = include_str!("../sql/h2.sql");

impl PgArm {
    pub fn new(kind: PgKind, port: u16, repo: &Path) -> PgArm {
        let name = match kind {
            PgKind::P => "p",
            PgKind::PPlus => "pplus",
            PgKind::M => "m",
            PgKind::MPlus => "mplus",
            PgKind::H2 => "h2",
        };
        let mut arm = PgArm {
            kind,
            cluster: Cluster::new(name, port),
            repo: repo.to_path_buf(),
        };
        let label = arm.name();
        arm.cluster.extra = crate::pgcluster::ARM_SETTINGS
            .iter()
            .filter(|(a, ..)| *a == label)
            .map(|(_, k, v, why)| (*k, *v, *why))
            .collect();
        arm
    }
    fn mechanism(&self) -> bool {
        matches!(self.kind, PgKind::PPlus | PgKind::MPlus)
    }
}

impl Arm for PgArm {
    fn name(&self) -> &'static str {
        match self.kind {
            PgKind::P => "P",
            PgKind::PPlus => "P+",
            PgKind::M => "M",
            PgKind::MPlus => "M+",
            PgKind::H2 => "H2",
        }
    }
    fn describe(&self) -> String {
        let what = match self.kind {
            PgKind::P => "ordinary materialised view `balances`, REFRESH MATERIALIZED VIEW CONCURRENTLY after every write; append-only trigger",
            PgKind::PPlus => "the E14 mechanism (partial slots, LRU, C = 16 checkpoints, anchored reconstruction) with the two cycle-14 repairs of sql/pplus.sql; append-only trigger",
            PgKind::M => "P without the append-only trigger (mutable base)",
            PgKind::MPlus => "P+ without the append-only trigger (mutable base)",
            PgKind::H2 => "pg_ivm 1.15 IMMV `balances` (sum per acct, cur), maintained inside each writing transaction, fully materialised, no eviction; append-only trigger; `shared_preload_libraries = pg_ivm`",
        };
        format!(
            "PostgreSQL 16, own cluster on 127.0.0.1:{}, SCRAM-SHA-256 as role bench: {what}",
            self.cluster.port
        )
    }
    fn connect(&self) -> Result<Client, String> {
        Client::connect("127.0.0.1", self.cluster.port, "bench", "bank").map_err(|e| e.to_string())
    }
    fn supports(&self, _q: &Query) -> Result<(), NotRun> {
        Ok(())
    }
    fn sql(&self, q: &Query) -> String {
        let m = self.mechanism();
        match q {
            Query::Point((a, c)) if m => format!(
                "select value from arm.rev_read({a}, {c}::smallint, (select applied_through from arm.rev_meta))"
            ),
            Query::Point((a, c)) => {
                format!("select balance from arm.balances where acct = {a} and cur = {c}")
            }
            Query::Anchored((a, c), e) if m => {
                format!("select value from arm.rev_read({a}, {c}::smallint, {e})")
            }
            Query::Anchored((a, c), e) => format!(
                "select sum(amt) from arm.postings where acct = {a} and cur = {c} and epoch <= {e}"
            ),
            Query::Statement((a, c), d0, d1) => format!(
                "select count(*), sum(amt) from arm.postings where acct = {a} and cur = {c} and value_day between {d0} and {d1}"
            ),
            Query::Desk(d) if m => format!(
                "select cur, sum(amt) from arm.postings where desk = {d} group by cur"
            ),
            Query::Desk(d) => format!(
                "select cur, sum(balance) from arm.balances where acct % 16 = {d} group by cur"
            ),
            Query::Top10(c) if m => format!(
                "select acct, sum(amt) from arm.postings where cur = {c} group by acct order by 2 desc limit 10"
            ),
            Query::Top10(c) => format!(
                "select acct, balance from arm.balances where cur = {c} order by balance desc limit 10"
            ),
            Query::Extract if m => {
                "select acct, cur, sum(amt) from arm.postings group by acct, cur".into()
            }
            Query::Extract => "select acct, cur, balance from arm.balances".into(),
        }
    }
    fn write_sql(&self, idx: u64, t: &Txn) -> Vec<String> {
        let [a, b] = t.legs;
        let call = format!(
            "select arm.post_txn({idx}, {}, {}, {}, {}::smallint, {}, {})",
            t.id, a.acct, b.acct, a.cur, b.amt, t.value_day
        );
        if self.mechanism() || self.kind == PgKind::H2 {
            vec![call]
        } else {
            vec![
                call,
                "refresh materialized view concurrently arm.balances".into(),
            ]
        }
    }
    fn load(&mut self, u: &Universe, budget: usize) -> Result<LoadReport, String> {
        let t0 = Instant::now();
        self.cluster.up(&self.repo)?;
        let mut c = self.connect()?;
        if c.auth != bank_bench::wire::AuthMethod::ScramSha256 {
            return Err("the cluster did not authenticate with SCRAM-SHA-256".into());
        }
        let run = |c: &mut Client, sql: &str| c.simple(sql).map(|_| ()).map_err(|e| e.to_string());
        if self.kind == PgKind::H2 {
            // `create extension pg_ivm` needs a superuser (the extension is not marked
            // trusted); run it over the local socket as the cluster's superuser, then let the
            // harness role call pg_ivm's functions. The harness role stays unprivileged.
            self.cluster.admin_sql(
                "bank",
                "create extension if not exists pg_ivm; grant usage on schema pgivm to bench",
            )?;
        }
        run(&mut c, COMMON)?;
        if matches!(self.kind, PgKind::P | PgKind::PPlus | PgKind::H2) {
            run(&mut c, IMMUTABLE)?;
        }
        run(
            &mut c,
            match self.kind {
                _ if self.mechanism() => PPLUS_SQL,
                PgKind::H2 => H2_SQL,
                _ => P_SQL,
            },
        )?;
        let mut statements = 3;
        let mut rows: Vec<String> = Vec::new();
        let mut current = 1;
        let flush = |c: &mut Client, e: u64, rows: &mut Vec<String>| -> Result<(), String> {
            if rows.is_empty() {
                return Ok(());
            }
            let sql = format!(
                "insert into arm.epochs values ({e}, '\\x00', '\\x00'); \
                 insert into arm.postings (epoch, txn, acct, cur, amt, value_day, desk) values {}",
                rows.join(", ")
            );
            rows.clear();
            c.simple(&sql).map(|_| ()).map_err(|e| e.to_string())
        };
        for t in &u.txns {
            if t.batch != current {
                flush(&mut c, current, &mut rows)?;
                statements += 1;
                current = t.batch;
            }
            for l in t.legs {
                rows.push(format!(
                    "({}, {}, {}, {}, {}, {}, {})",
                    t.batch,
                    t.id,
                    l.acct,
                    l.cur,
                    l.amt,
                    t.value_day,
                    crate::universe::desk_of(l.acct)
                ));
            }
        }
        flush(&mut c, current, &mut rows)?;
        statements += 1;
        run(
            &mut c,
            &format!(
                "do $$ begin for e in 1..{} loop perform arm.seal(e); end loop; end $$",
                u.batches()
            ),
        )?;
        if self.mechanism() {
            run(&mut c, &format!("select arm.after_load({budget})"))?;
        } else if self.kind == PgKind::H2 {
            run(&mut c, "select arm.after_load_h2()")?;
        } else {
            run(&mut c, "refresh materialized view arm.balances")?;
        }
        run(&mut c, "vacuum analyze")?;
        statements += 3;
        Ok(LoadReport {
            seconds: t0.elapsed().as_secs_f64(),
            statements,
            notes: vec![format!(
                "one multi-row insert per batch with explicit epoch ids 1..{}; epochs sealed in order by arm.seal (sha256 chain); {}; vacuum analyze",
                u.batches(),
                if self.mechanism() {
                    "arm.after_load: checkpoints every 16 legs per key, key counts, budget"
                } else if self.kind == PgKind::H2 {
                    "IMMV created and populated by pgivm.create_immv after the load"
                } else {
                    "materialised view refreshed"
                }
            )],
        })
    }
    fn legs(&self, c: &mut Client) -> Result<Vec<Leg>, String> {
        let rows = c
            .simple("select acct, cur, amt from arm.postings")
            .map_err(|e| e.to_string())?;
        legs_from(&rows)
    }
    fn pss(&self) -> Option<u64> {
        self.cluster.pss_bytes()
    }
    fn state(&self, c: &mut Client) -> Vec<(String, i128)> {
        let mut out = Vec::new();
        let rels: &[&str] = if self.mechanism() {
            &[
                "arm.rev",
                "arm.checkpoints",
                "arm.key_counts",
                "arm.postings",
            ]
        } else {
            &["arm.balances", "arm.postings"]
        };
        for r in rels {
            if let Ok(rows) = c.simple(&format!("select pg_total_relation_size('{r}')")) {
                if let Some(v) = rows.nth(0) {
                    out.push((format!("bytes {r}"), v));
                }
            }
        }
        if self.mechanism() {
            if let Ok(rows) = c.simple("select present from arm.rev_meta") {
                if let Some(v) = rows.nth(0) {
                    out.push(("resident".into(), v));
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
        self.cluster.down();
    }
}

/// The side-by-side texts for the header (§5.5), with placeholder values.
pub fn sql_table(arms: &[&dyn Arm]) -> String {
    let qs = [
        Query::Point((7, 0)),
        Query::Anchored((7, 0), 100),
        Query::Statement((7, 0), 10, 31),
        Query::Desk(3),
        Query::Top10(0),
        Query::Extract,
    ];
    let mut s = String::from("| query | ");
    s += &arms
        .iter()
        .map(|a| a.name())
        .collect::<Vec<_>>()
        .join(" | ");
    s += " |\n|---|";
    s += &"---|".repeat(arms.len());
    s += "\n";
    for q in &qs {
        s += &format!("| {} {} | ", q.id(), Query::describe(q.id()));
        let cells: Vec<String> = arms
            .iter()
            .map(|a| match a.supports(q) {
                Ok(()) => format!("`{}`", a.sql(q)),
                Err(_) => "NOT RUN (see note)".into(),
            })
            .collect();
        s += &cells.join(" | ");
        s += " |\n";
    }
    s
}
