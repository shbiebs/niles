//! One PostgreSQL cluster per PostgreSQL arm.
//!
//! **Why separate clusters.** The arms are compared on memory as well as time, and one
//! cluster serving three arms has one set of shared buffers, one WAL and one set of
//! background processes: its memory cannot be attributed to an arm, and one arm's writes
//! share the WAL device's queue with another's. A cluster per arm, configured identically
//! (§5.3), makes each arm's process memory its own and its fsyncs its own.
//!
//! **Authentication is not weakened.** Clusters are initialised with `peer` on the local
//! socket (administration, as the `postgres` OS user) and `scram-sha-256` on TCP — the stock
//! PostgreSQL 16 posture — and the harness's role is created by `tools/pg-provision.sh`.

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = "/usr/lib/postgresql/16/bin";

/// §5.3's configuration, every setting with the reason it is set.
pub const SETTINGS: &[(&str, &str, &str)] = &[
    (
        "shared_buffers",
        "1GB",
        "cycle-13 Part 4, identical per arm",
    ),
    (
        "synchronous_commit",
        "on",
        "a commit is on stable storage when acknowledged, as on N",
    ),
    (
        "fsync",
        "on",
        "the same durability contract as N's SyncPolicy::Always",
    ),
    (
        "wal_level",
        "logical",
        "H3 reads logical replication; set on every arm so they stay alike",
    ),
    (
        "max_connections",
        "64",
        "room for 4 concurrent clients per arm with margin",
    ),
    (
        "random_page_cost",
        "1.1",
        "the data directory is on solid-state storage",
    ),
    (
        "jit",
        "off",
        "so plan-time variance is not attributed to an arm",
    ),
    ("listen_addresses", "'127.0.0.1'", "local only"),
    (
        "unix_socket_directories",
        "'/var/run/postgresql'",
        "where tools/pg-provision.sh looks",
    ),
];

pub struct Cluster {
    pub name: String,
    pub port: u16,
    pub dir: PathBuf,
}

fn as_postgres(cmd: &str) -> Result<String, String> {
    let out = Command::new("su")
        .args(["postgres", "-c", cmd])
        .output()
        .map_err(|e| format!("su postgres: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "`{cmd}` failed: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

impl Cluster {
    pub fn new(name: &str, port: u16) -> Cluster {
        Cluster {
            name: name.into(),
            port,
            dir: PathBuf::from(format!("/var/lib/postgresql/e27/{name}")),
        }
    }

    /// Initialise if absent, start if stopped, provision the harness role. Idempotent.
    pub fn up(&self, repo_root: &Path) -> Result<(), String> {
        let dir = self.dir.display().to_string();
        if !self.dir.join("PG_VERSION").exists() {
            as_postgres(&format!("mkdir -p {dir}"))?;
            as_postgres(&format!(
                "{BIN}/initdb -D {dir} --auth-local=peer --auth-host=scram-sha-256 -U postgres -E UTF8 >/dev/null"
            ))?;
        }
        let running = self.dir.join("postmaster.pid").exists()
            && Command::new("pg_isready")
                .args(["-q", "-h", "127.0.0.1", "-p", &self.port.to_string()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        if !running {
            let _ = std::fs::remove_file(self.dir.join("postmaster.pid"));
            let opts: Vec<String> = SETTINGS
                .iter()
                .map(|(k, v, _)| format!("-c {k}={v}"))
                .collect();
            as_postgres(&format!(
                "{BIN}/pg_ctl -D {dir} -w -l {dir}/log.txt -o \"-p {} {}\" start >/dev/null",
                self.port,
                opts.join(" ")
            ))?;
        }
        let script = repo_root.join("tools/pg-provision.sh");
        let out = Command::new("sh")
            .arg(&script)
            .env("PGPORT", self.port.to_string())
            .output()
            .map_err(|e| format!("pg-provision: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "pg-provision on port {}: {}",
                self.port,
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        Ok(())
    }

    pub fn down(&self) {
        let _ = as_postgres(&format!(
            "{BIN}/pg_ctl -D {} -m fast stop >/dev/null",
            self.dir.display()
        ));
    }

    /// Proportional set size of the cluster: the postmaster and every process it started.
    /// PSS, not RSS, because backends map the same shared buffers and RSS would count them
    /// once per process.
    pub fn pss_bytes(&self) -> Option<u64> {
        let pid: u32 = std::fs::read_to_string(self.dir.join("postmaster.pid"))
            .ok()?
            .lines()
            .next()?
            .trim()
            .parse()
            .ok()?;
        let mut total = pss_of(pid)?;
        for entry in std::fs::read_dir("/proc").ok()?.flatten() {
            let Ok(child) = entry.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            if parent_of(child) == Some(pid) {
                total += pss_of(child).unwrap_or(0);
            }
        }
        Some(total)
    }
}

pub fn parent_of(pid: u32) -> Option<u32> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    s.lines()
        .find_map(|l| l.strip_prefix("PPid:"))
        .and_then(|v| v.trim().parse().ok())
}

/// PSS of one process in bytes, from `/proc/<pid>/smaps_rollup`.
pub fn pss_of(pid: u32) -> Option<u64> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok()?;
    s.lines()
        .find_map(|l| l.strip_prefix("Pss:"))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
        .map(|kb| kb * 1024)
}
