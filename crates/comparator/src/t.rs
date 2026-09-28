//! **T — TigerBeetle as the ledger floor, with CDC through RabbitMQ into the REV sidecar**
//! (cycle 14, R2-03; decision D-6).
//!
//! | part | what runs |
//! |---|---|
//! | ledger | TigerBeetle 0.17.9, one replica (`--replica-count=1`), Direct I/O, `--cache-grid=256MiB` |
//! | client | `tools/arms/tigerbeetle/driver.py` with the official Python client, outside both workspaces |
//! | change stream | `tigerbeetle amqp` publishing to exchange `tb-cdc` on RabbitMQ 3.12.1 (loopback), consumed by the driver |
//! | read side | `rev-sidecar --ledger driver:…`: the same nilestream-core REV as H3, budget and LRU as N |
//!
//! **What T is for.** §7: "T informs the write floor only". Its write is acknowledged when
//! TigerBeetle has committed it (fsync, one replica), which is the ledger-floor number the rule
//! compares with N's and P+'s commits/s. Its reads (q1, q2) are served by the REV from the
//! change stream, so they may trail the head: each is bracketed by the sidecar's frontier just
//! before and just after it and held to the oracle at an epoch in that bracket, as H1's are
//! (`h1.rs`). q3–q6 are NOT RUN: TigerBeetle has no query language, and a report would be a
//! client-side fold over lookups — a measurement of the driver, not of the ledger.
//!
//! **Credentials.** The broker user `cdc` gets a password generated here into an untracked
//! file (mode 600) under the scratch directory; `rabbitmqctl` reads it on standard input; the
//! driver reads the file. `tigerbeetle amqp` accepts the password only as a command-line flag,
//! so it is visible in that process's command line on this single-user host (stated to the
//! author before the download was approved); it is in no log, and the load checks that.
//!
//! **Deviation, stated before any T measurement:** `--cache-grid=256MiB` rather than
//! TigerBeetle's 1 GiB default, so that all eight arms fit the 8 GB host at once; TigerBeetle
//! still allocates ~2.3 GiB at start (measured 2026-09-28), which is in T's memory figure.

use crate::arms::{Arm, LoadReport, NotRun, Query};
use crate::pgcluster::pss_of;
use crate::universe::{Leg, Txn, Universe};
use bank_bench::wire::Client;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const T_NOT_RUN: NotRun =
    "T is the ledger floor (§7): TigerBeetle has no query language, so a statement, a desk \
     exposure, a top-10 or an extract would be a client-side fold over lookups in the driver \
     — a measurement of the driver, not of the ledger";

fn env_path(var: &str, default: &str) -> PathBuf {
    std::env::var(var)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default))
}

pub struct TArm {
    pub dir: PathBuf,
    pub repo: PathBuf,
    pub tb_port: u16,
    pub driver_port: u16,
    pub port: u16,
    pub amqp_port: u16,
    tb: Option<Child>,
    driver: Option<Child>,
    cdc: Option<Child>,
    sidecar: Option<Child>,
    broker_up: bool,
    status: Mutex<Option<Client>>,
}

impl TArm {
    pub fn new(dir: PathBuf, repo: &Path) -> TArm {
        TArm {
            dir,
            repo: repo.to_path_buf(),
            tb_port: 3101,
            driver_port: 5464,
            port: 5465,
            amqp_port: 5672,
            tb: None,
            driver: None,
            cdc: None,
            sidecar: None,
            broker_up: false,
            status: Mutex::new(None),
        }
    }

    fn tigerbeetle() -> PathBuf {
        env_path("TIGERBEETLE_BIN", "/opt/arms/tigerbeetle/tigerbeetle")
    }
    fn python() -> PathBuf {
        env_path("TB_PYTHON", "/opt/arms/tbvenv/bin/python")
    }
    fn password_file(&self) -> PathBuf {
        self.dir.join("amqp-password")
    }

    fn broker_env(&self, cmd: &mut Command) {
        let base = self.dir.join("rabbitmq");
        cmd.env("RABBITMQ_NODENAME", "e27t@localhost")
            .env("RABBITMQ_NODE_IP_ADDRESS", "127.0.0.1")
            .env("RABBITMQ_NODE_PORT", self.amqp_port.to_string())
            .env("RABBITMQ_DIST_PORT", "25673")
            .env("ERL_EPMD_ADDRESS", "127.0.0.1")
            .env("RABBITMQ_MNESIA_BASE", base.join("mnesia"))
            .env("RABBITMQ_LOG_BASE", base.join("log"))
            .env("RABBITMQ_PID_FILE", base.join("pid"))
            .env("RABBITMQ_FEATURE_FLAGS_FILE", base.join("feature_flags"));
    }

    fn ctl(&self, args: &[&str], stdin: Option<&str>) -> Result<(), String> {
        let mut cmd = Command::new("rabbitmqctl");
        self.broker_env(&mut cmd);
        cmd.args(["-q", "-n", "e27t@localhost"])
            .args(args)
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| format!("rabbitmqctl: {e}"))?;
        if let (Some(input), Some(mut sin)) = (stdin, child.stdin.take()) {
            sin.write_all(input.as_bytes())
                .map_err(|e| format!("rabbitmqctl stdin: {e}"))?;
        }
        let out = child
            .wait_with_output()
            .map_err(|e| format!("rabbitmqctl: {e}"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(format!(
                "rabbitmqctl {}: {}",
                args.first().unwrap_or(&""),
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }

    /// Start the broker once per process, and (re)create the `cdc` user with a password
    /// generated on this machine into an untracked, mode-600 file.
    fn broker(&mut self) -> Result<(), String> {
        if self.broker_up {
            return Ok(());
        }
        for d in ["rabbitmq/mnesia", "rabbitmq/log"] {
            std::fs::create_dir_all(self.dir.join(d)).map_err(|e| e.to_string())?;
        }
        // Debian's `rabbitmq-server` drops to the `rabbitmq` user when started as root, so its
        // directories must be that user's.
        if crate::pgcluster::is_root() {
            let _ = Command::new("chown")
                .args(["-R", "rabbitmq:rabbitmq"])
                .arg(self.dir.join("rabbitmq"))
                .status();
        }
        let log =
            std::fs::File::create(self.dir.join("rabbitmq.out")).map_err(|e| e.to_string())?;
        let mut cmd = Command::new("rabbitmq-server");
        self.broker_env(&mut cmd);
        cmd.stdin(Stdio::null()).stdout(log).stderr(Stdio::null());
        cmd.spawn().map_err(|e| format!("rabbitmq-server: {e}"))?;
        let deadline = Instant::now() + Duration::from_secs(90);
        while self.ctl(&["await_startup"], None).is_err() {
            if Instant::now() > deadline {
                return Err("RabbitMQ did not start within 90 s".into());
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        let mut raw = [0u8; 24];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut raw))
            .map_err(|e| format!("/dev/urandom: {e}"))?;
        let password: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(self.password_file())
                .map_err(|e| format!("password file: {e}"))?;
            f.write_all(password.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        let _ = self.ctl(&["delete_user", "cdc"], None);
        self.ctl(&["add_user", "cdc"], Some(&password))?;
        self.ctl(
            &["set_permissions", "-p", "/", "cdc", ".*", ".*", ".*"],
            None,
        )?;
        self.broker_up = true;
        Ok(())
    }

    fn broker_pid(&self) -> Option<u32> {
        std::fs::read_to_string(self.dir.join("rabbitmq/pid"))
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    fn kill_children(&mut self) {
        *self.status.lock().unwrap() = None;
        for c in [
            self.sidecar.take(),
            self.cdc.take(),
            self.driver.take(),
            self.tb.take(),
        ]
        .into_iter()
        .flatten()
        {
            let mut c = c;
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    fn wait_port(port: u16, what: &str, secs: u64) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(secs);
        while TcpStream::connect(("127.0.0.1", port)).is_err() {
            if Instant::now() > deadline {
                return Err(format!("{what} did not listen on {port} within {secs} s"));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    fn spawn(&self, cmd: &mut Command, log: &str) -> Result<Child, String> {
        let f = std::fs::File::create(self.dir.join(log)).map_err(|e| e.to_string())?;
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(f)
            .spawn()
            .map_err(|e| format!("spawn {cmd:?}: {e}"))
    }
}

/// The driver's `LOAD` lines for one transaction: the paying leg is the debit.
fn load_row(t: &Txn) -> String {
    let [a, b] = t.legs;
    let (from, to) = if a.amt < 0 { (a, b) } else { (b, a) };
    format!(
        "{} {} {} {} {} {}\n",
        t.id, from.acct, to.acct, from.cur, to.amt, t.value_day
    )
}

impl Arm for TArm {
    fn name(&self) -> &'static str {
        "T"
    }
    fn describe(&self) -> String {
        format!(
            "TigerBeetle 0.17.9 (Apache-2.0), one replica on 127.0.0.1:{}, Direct I/O, \
             --cache-grid=256MiB; writes acknowledged at TigerBeetle's commit; CDC by \
             `tigerbeetle amqp` (--idle-interval-ms=1) through RabbitMQ 3.12.1 into \
             `rev-sidecar --ledger driver` on 127.0.0.1:{} (REV, LRU, budget 5% of keys; upquery \
             = the key's balance at the epoch's timestamp from TigerBeetle's account history); \
             q1, q2 checked at the sidecar's bracketed frontier; q3–q6 NOT RUN",
            self.tb_port, self.port
        )
    }
    fn connect(&self) -> Result<Client, String> {
        Client::connect("127.0.0.1", self.port, "bench", "bank").map_err(|e| e.to_string())
    }
    fn supports(&self, q: &Query) -> Result<(), NotRun> {
        match q {
            Query::Point(_) | Query::Anchored(..) => Ok(()),
            _ => Err(T_NOT_RUN),
        }
    }
    fn sql(&self, q: &Query) -> String {
        match q {
            Query::Point((a, c)) => format!("select value from rev_read({a}, {c})"),
            Query::Anchored((a, c), e) => format!("select value from rev_read({a}, {c}, {e})"),
            _ => String::new(),
        }
    }
    fn write_sql(&self, idx: u64, t: &Txn) -> Vec<String> {
        let [a, b] = t.legs;
        vec![format!(
            "select arm.post_txn({idx}, {}, {}, {}, {}::smallint, {}, {})",
            t.id, a.acct, b.acct, a.cur, b.amt, t.value_day
        )]
    }
    fn replica_offsets(&self) -> Option<Result<(u64, u64), String>> {
        let mut g = self.status.lock().unwrap();
        if g.is_none() {
            match self.connect() {
                Ok(c) => *g = Some(c),
                Err(e) => return Some(Err(e)),
            }
        }
        Some(
            g.as_mut()
                .unwrap()
                .simple("select rev_frontier")
                .map_err(|e| e.to_string())
                .and_then(|r| r.nth(0).ok_or_else(|| "no frontier".to_string()))
                .map(|f| (f as u64, f as u64)),
        )
    }
    /// The sidecar's frontier is already an epoch: a read between two frontier readings is
    /// at an epoch between them.
    fn epochs_between(&self, lo: u64, hi: u64) -> (u64, u64) {
        (lo, hi)
    }
    fn load(&mut self, u: &Universe, budget: usize) -> Result<LoadReport, String> {
        let t0 = Instant::now();
        self.kill_children();
        let sidecar_bin = self.repo.join("target/release/rev-sidecar");
        if !sidecar_bin.exists() {
            return Err(format!(
                "{} is missing; run `cargo build --release -p rev-sidecar` first",
                sidecar_bin.display()
            ));
        }
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        self.broker()?;

        // A fresh ledger per point.
        let data = self.dir.join("0_0.tigerbeetle");
        let _ = std::fs::remove_file(&data);
        let st = Command::new(Self::tigerbeetle())
            .args([
                "format",
                "--cluster=0",
                "--replica=0",
                "--replica-count=1",
                &data.display().to_string(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("tigerbeetle format: {e}"))?;
        if !st.success() {
            return Err("tigerbeetle format failed".into());
        }
        self.tb = Some(self.spawn(
            Command::new(Self::tigerbeetle()).args([
                "start",
                &format!("--addresses=127.0.0.1:{}", self.tb_port),
                "--cache-grid=256MiB",
                &data.display().to_string(),
            ]),
            "tigerbeetle.log",
        )?);
        // TigerBeetle allocates its ~2.3 GiB and opens its 1 GiB journal before it listens;
        // with the other seven arms loaded at 10⁵ that took more than a minute.
        Self::wait_port(self.tb_port, "TigerBeetle", 600)?;
        self.driver = Some(
            self.spawn(
                Command::new(Self::python()).args([
                    self.repo
                        .join("tools/arms/tigerbeetle/driver.py")
                        .display()
                        .to_string(),
                    "--tb".into(),
                    format!("127.0.0.1:{}", self.tb_port),
                    "--listen".into(),
                    format!("127.0.0.1:{}", self.driver_port),
                    "--amqp".into(),
                    format!("127.0.0.1:{}", self.amqp_port),
                    "--amqp-password-file".into(),
                    self.password_file().display().to_string(),
                ]),
                "driver.log",
            )?,
        );
        Self::wait_port(self.driver_port, "the driver", 60)?;

        // The load, one request per batch (one epoch each).
        let s = TcpStream::connect(("127.0.0.1", self.driver_port)).map_err(|e| e.to_string())?;
        let mut w = s.try_clone().map_err(|e| e.to_string())?;
        let mut r = BufReader::new(s);
        let mut reply = |w: &mut TcpStream, req: String| -> Result<String, String> {
            w.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
            let mut line = String::new();
            r.read_line(&mut line).map_err(|e| e.to_string())?;
            let line = line.trim().to_string();
            if line.starts_with("OK") {
                Ok(line)
            } else {
                Err(format!("driver: {line}"))
            }
        };
        let mut statements = 0;
        let mut i = 0;
        while i < u.txns.len() {
            let e = u.txns[i].batch;
            let mut j = i;
            let mut body = String::new();
            while j < u.txns.len() && u.txns[j].batch == e {
                body += &load_row(&u.txns[j]);
                j += 1;
            }
            reply(&mut w, format!("LOAD {e} {}\n{body}", j - i))?;
            statements += 1;
            i = j;
        }
        let ts_last = reply(&mut w, "CDC\n".into())?
            .split_whitespace()
            .nth(1)
            .unwrap_or("0")
            .to_string();
        drop(w);

        let password = std::fs::read_to_string(self.password_file())
            .map_err(|e| format!("password file: {e}"))?;
        self.cdc = Some(self.spawn(
            Command::new(Self::tigerbeetle()).args([
                "amqp".to_string(),
                format!("--addresses=127.0.0.1:{}", self.tb_port),
                "--cluster=0".into(),
                format!("--host=127.0.0.1:{}", self.amqp_port),
                "--vhost=/".into(),
                "--user=cdc".into(),
                format!("--password={}", password.trim()),
                "--publish-exchange=tb-cdc".into(),
                "--idle-interval-ms=1".into(),
                format!("--timestamp-last={ts_last}"),
            ]),
            "amqp.log",
        )?);
        self.sidecar = Some(self.spawn(
            Command::new(&sidecar_bin).args([
                "--listen".to_string(),
                format!("127.0.0.1:{}", self.port),
                "--ledger".into(),
                format!("driver:127.0.0.1:{}", self.driver_port),
                "--budget".into(),
                budget.to_string(),
                "--loaded-head".into(),
                u.batches().to_string(),
            ]),
            "sidecar.log",
        )?);
        Self::wait_port(self.port, "rev-sidecar", 60)?;
        // No log may carry the broker password.
        for log in [
            "tigerbeetle.log",
            "driver.log",
            "amqp.log",
            "sidecar.log",
            "rabbitmq.out",
        ] {
            if let Ok(text) = std::fs::read_to_string(self.dir.join(log)) {
                if text.contains(password.trim()) {
                    return Err(format!("{log} contains the broker password — refused"));
                }
            }
        }
        Ok(LoadReport {
            seconds: t0.elapsed().as_secs_f64(),
            statements,
            notes: vec![format!(
                "one create_transfers per batch through the driver (one epoch each; accounts \
                 created on first use with flag `history`); CDC started after the load at \
                 --timestamp-last={ts_last}; the REV certified through #{} with nothing resident",
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
        let mut total = 0;
        for c in [&self.tb, &self.driver, &self.cdc, &self.sidecar]
            .into_iter()
            .flatten()
        {
            total += pss_of(c.id())?;
        }
        total += self.broker_pid().and_then(pss_of).unwrap_or(0);
        Some(total)
    }
    fn state(&self, c: &mut Client) -> Vec<(String, i128)> {
        let mut out = Vec::new();
        for (n, ch) in [
            ("tigerbeetle PSS", &self.tb),
            ("driver PSS", &self.driver),
            ("tigerbeetle amqp PSS", &self.cdc),
            ("sidecar PSS", &self.sidecar),
        ] {
            if let Some(p) = ch.as_ref().and_then(|x| pss_of(x.id())) {
                out.push((n.into(), p as i128));
            }
        }
        if let Some(p) = self.broker_pid().and_then(pss_of) {
            out.push(("rabbitmq PSS".into(), p as i128));
        }
        if let Ok(rows) = c.simple("select rev_stats") {
            for n in [
                "resident",
                "reads",
                "hits",
                "misses",
                "rows_touched",
                "evictions",
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
        self.kill_children();
        if self.broker_up {
            let _ = self.ctl(&["stop"], None);
            self.broker_up = false;
        }
    }
}

impl Drop for TArm {
    fn drop(&mut self) {
        self.stop();
    }
}
