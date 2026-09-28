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

/// Where PostgreSQL 16's binaries are: `PG_BIN`, else Debian/Ubuntu's location. On a Mac,
/// `PG_BIN=/opt/homebrew/opt/postgresql@16/bin` (Homebrew) or Postgres.app's `bin`.
pub fn bin() -> String {
    std::env::var("PG_BIN").unwrap_or_else(|_| "/usr/lib/postgresql/16/bin".into())
}

/// Whether this process is root — the cloud container, where clusters are administered as
/// the `postgres` OS user over peer authentication. Elsewhere (Host C) the current user owns
/// the clusters and is their superuser, as with a Homebrew or Postgres.app installation.
pub fn is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false)
}

/// The socket directory the clusters listen on, and where `tools/pg-provision.sh`'s `psql`
/// looks by default: Debian's under root, `/tmp` (Homebrew's and Postgres.app's) elsewhere.
pub fn socket_dir() -> &'static str {
    if is_root() {
        "/var/run/postgresql"
    } else {
        "/tmp"
    }
}

/// Where the per-arm data directories live: `E27_PGDATA`, else a fixed place per host kind.
pub fn base_dir() -> PathBuf {
    if let Ok(d) = std::env::var("E27_PGDATA") {
        return PathBuf::from(d);
    }
    if is_root() {
        PathBuf::from("/var/lib/postgresql/e27")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".e27-pg")
    }
}

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
        "'/var/run/postgresql' (root) or '/tmp' (elsewhere)",
        "where tools/pg-provision.sh's psql looks by default",
    ),
];

pub struct Cluster {
    pub name: String,
    pub port: u16,
    pub dir: PathBuf,
}

/// Run an administrative shell command: as the `postgres` OS user under root, as the current
/// user elsewhere.
fn as_postgres(cmd: &str) -> Result<String, String> {
    let out = if is_root() {
        Command::new("su").args(["postgres", "-c", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    }
    .map_err(|e| format!("administrative shell: {e}"))?;
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
            dir: base_dir().join(name),
        }
    }

    /// Initialise if absent, start if stopped, provision the harness role. Idempotent.
    pub fn up(&self, repo_root: &Path) -> Result<(), String> {
        let dir = self.dir.display().to_string();
        let bin = bin();
        if !self.dir.join("PG_VERSION").exists() {
            as_postgres(&format!("mkdir -p {dir}"))?;
            // Under root the superuser is `postgres`; elsewhere it is the current user, which
            // is who tools/pg-provision.sh connects as.
            let superuser = if is_root() { "-U postgres" } else { "" };
            as_postgres(&format!(
                "{bin}/initdb -D {dir} --auth-local=peer --auth-host=scram-sha-256 {superuser} -E UTF8 >/dev/null"
            ))?;
        }
        let running = self.dir.join("postmaster.pid").exists()
            && Command::new(format!("{bin}/pg_isready"))
                .args(["-q", "-h", "127.0.0.1", "-p", &self.port.to_string()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        if !running {
            let _ = std::fs::remove_file(self.dir.join("postmaster.pid"));
            let opts: Vec<String> = SETTINGS
                .iter()
                .map(|(k, v, _)| {
                    if *k == "unix_socket_directories" {
                        format!("-c {k}='{}'", socket_dir())
                    } else {
                        format!("-c {k}={v}")
                    }
                })
                .collect();
            as_postgres(&format!(
                "{bin}/pg_ctl -D {dir} -w -l {dir}/log.txt -o \"-p {} {}\" start >/dev/null",
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
            "{}/pg_ctl -D {} -m fast stop >/dev/null",
            bin(),
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
