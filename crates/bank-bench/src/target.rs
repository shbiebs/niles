//! What a benchmark target is, and the two that exist.
//!
//! # The rule this trait exists to enforce
//!
//! Both targets are driven **over the PostgreSQL wire protocol, through the same client**.
//! A comparison where one side is called in-process and the other over TCP is not a
//! comparison: it omits, from one side only, the syscalls, the copies and the round trip
//! that the other pays on every query. That omission is invisible in the result and worth
//! roughly the whole margin the specification claims, so the shape of this module is the
//! shape of the claim's honesty.
//!
//! # What a target must supply
//!
//! Enough to prepare a workload, run it, and be described in the results — including the
//! configuration that would change the numbers, because a durable-commit figure with
//! `synchronous_commit = off` is a different measurement wearing the same name.

use crate::wire::{Client, Rows, WireError};

/// One system under test.
pub trait Target {
    /// A short name for the CSV's `target` column.
    fn name(&self) -> &str;

    /// The configuration that would change the numbers, as `key=value` pairs.
    ///
    /// Recorded in `docs/BENCHMARK.md` rather than left to a reader to assume. A run that
    /// does not say what `synchronous_commit` was is a run whose durable-commit figure means
    /// nothing.
    fn configuration(&mut self) -> Vec<(String, String)>;

    /// Create the schema and load the data a workload needs.
    fn prepare(&mut self, accounts: i64) -> Result<(), WireError>;

    /// Run one statement, returning what came back.
    fn run(&mut self, sql: &str) -> Result<Rows, WireError>;

    /// Prepare a statement for repeated execution.
    fn prepare_statement(&mut self, sql: &str) -> Result<String, WireError>;

    /// Execute a prepared statement.
    fn exec(&mut self, name: &str, params: &[&str]) -> Result<Rows, WireError>;

    /// Remove whatever `prepare` created, so a re-run starts from the same place.
    fn teardown(&mut self) -> Result<(), WireError>;

    /// Whether this target's commit is durable — `fsync` on the write path.
    ///
    /// Reported rather than assumed, because the durable-commit row of the performance
    /// contract is meaningless without it, and a target that is not durable must not be
    /// compared against one that is.
    fn is_durable(&mut self) -> bool;

    /// What this target cannot do, as a reason string.
    ///
    /// A workload a target cannot run is reported `NOT RUN` **with this reason**, never
    /// silently omitted and never approximated with something else. An engine that cannot yet
    /// serve a workload is a finding; a benchmark that quietly measured a substitute would be
    /// a fabrication.
    fn unsupported(&self, workload: &str) -> Option<String>;
}

/// PostgreSQL, over its own wire protocol.
pub struct PgTarget {
    client: Client,
    host: String,
    port: u16,
    database: String,
}

impl PgTarget {
    pub fn connect(host: &str, port: u16, user: &str, database: &str) -> Result<Self, WireError> {
        Ok(PgTarget {
            client: Client::connect(host, port, user, database)?,
            host: host.to_string(),
            port,
            database: database.to_string(),
        })
    }

    /// A second connection to the same server — for a workload that needs one.
    pub fn reconnect(&self) -> Result<Client, WireError> {
        Client::connect(&self.host, self.port, "bench", &self.database)
    }
}

impl Target for PgTarget {
    fn name(&self) -> &str {
        "postgres"
    }

    fn configuration(&mut self) -> Vec<(String, String)> {
        // Read from the server rather than from a config file: what matters is what the
        // running process believes, and the two can differ.
        let mut out = Vec::new();
        for setting in [
            "server_version",
            "synchronous_commit",
            "fsync",
            "shared_buffers",
            "work_mem",
            "max_wal_size",
            "wal_level",
            "full_page_writes",
        ] {
            if let Ok(r) = self.client.simple(&format!("show {setting}")) {
                if let Some(Some(v)) = r.rows.first().and_then(|row| row.first()) {
                    out.push((setting.to_string(), v.clone()));
                }
            }
        }
        out
    }

    fn prepare(&mut self, accounts: i64) -> Result<(), WireError> {
        self.teardown()?;
        // The schema mirrors what the engine serves: an append-only postings table, and a
        // balance is a fold over it. Deliberately *not* a balances table with an UPDATE —
        // that would be a different system, and comparing against it would answer a question
        // nobody asked.
        self.client.simple(
            "create table postings (
                 id     bigserial primary key,
                 txn    text      not null,
                 acct   bigint    not null,
                 cur    text      not null,
                 amt    bigint    not null,
                 epoch  bigint    not null
             )",
        )?;
        self.client.simple("create index ix_postings_acct on postings (acct, epoch)")?;
        // Seed: one opening posting per account, and its contra against a house account, so
        // the table conserves exactly as the ledger does.
        self.client.simple(&format!(
            "insert into postings (txn, acct, cur, amt, epoch)
             select 'seed-' || g, g, 'USD', g * 100, g from generate_series(1, {accounts}) g"
        ))?;
        self.client.simple(&format!(
            "insert into postings (txn, acct, cur, amt, epoch)
             select 'seed-' || g, 0, 'USD', -g * 100, g from generate_series(1, {accounts}) g"
        ))?;
        self.client.simple("analyze postings")?;
        Ok(())
    }

    fn run(&mut self, sql: &str) -> Result<Rows, WireError> {
        self.client.simple(sql)
    }

    fn prepare_statement(&mut self, sql: &str) -> Result<String, WireError> {
        self.client.prepare(sql, &[])
    }

    fn exec(&mut self, name: &str, params: &[&str]) -> Result<Rows, WireError> {
        self.client.execute(name, params)
    }

    fn teardown(&mut self) -> Result<(), WireError> {
        self.client.simple("drop table if exists postings")?;
        Ok(())
    }

    fn is_durable(&mut self) -> bool {
        // Durable only if *both* are on. `fsync = off` with `synchronous_commit = on` is a
        // configuration that reports commits it can lose, and it is a common benchmark
        // shortcut, so it is checked rather than assumed.
        let sync = self
            .client
            .simple("show synchronous_commit")
            .ok()
            .and_then(|r| r.rows.first()?.first()?.clone())
            .unwrap_or_default();
        let fsync = self
            .client
            .simple("show fsync")
            .ok()
            .and_then(|r| r.rows.first()?.first()?.clone())
            .unwrap_or_default();
        sync == "on" && fsync == "on"
    }

    fn unsupported(&self, _workload: &str) -> Option<String> {
        None
    }
}

/// Nilestream, over the same protocol.
///
/// # What this target can and cannot answer, stated here rather than discovered later
///
/// `nilestreamd` compiles each query as Niles and serves it from the read-model runtime. The
/// **read** path is the real mechanism: partial materialisation, the absence lattice, anchored
/// reconstruction. The **write** path over the wire is not yet exposed — the server has no
/// `insert` surface — so any workload that writes is reported `NOT RUN` with that reason.
///
/// Naming it here is the whole point. A benchmark that quietly substituted a read for a write,
/// or measured an in-process call and labelled it a commit, would produce a number that looks
/// like evidence and is not.
pub struct NilestreamTarget {
    client: Client,
    accounts: i64,
}

impl NilestreamTarget {
    pub fn connect(host: &str, port: u16) -> Result<Self, WireError> {
        Ok(NilestreamTarget {
            client: Client::connect(host, port, "bench", "bank")?,
            accounts: 0,
        })
    }
}

impl Target for NilestreamTarget {
    fn name(&self) -> &str {
        "nilestream"
    }

    fn configuration(&mut self) -> Vec<(String, String)> {
        let mut out = vec![("engine".into(), "nilestreamd".into())];
        if let Ok(r) = self.client.simple("select nilestream_frontier") {
            if let Some(Some(v)) = r.rows.first().and_then(|row| row.first()) {
                out.push(("frontier".into(), v.clone()));
            }
        }
        out
    }

    fn prepare(&mut self, accounts: i64) -> Result<(), WireError> {
        // The server is started with `--accounts N`, so preparation is a check that the data
        // is there rather than a load. Verified rather than assumed: a run against an empty
        // server would report excellent latencies for queries that returned nothing.
        self.accounts = accounts;
        let probe = self
            .client
            .simple("select acct, sum(amt) from postings where acct = 1 group by acct")?;
        if probe.rows.is_empty() {
            return Err(WireError::Protocol(
                "the server has no data for account 1; start nilestreamd with --accounts".into(),
            ));
        }
        Ok(())
    }

    fn run(&mut self, sql: &str) -> Result<Rows, WireError> {
        self.client.simple(sql)
    }

    fn prepare_statement(&mut self, sql: &str) -> Result<String, WireError> {
        self.client.prepare(sql, &[])
    }

    fn exec(&mut self, name: &str, params: &[&str]) -> Result<Rows, WireError> {
        self.client.execute(name, params)
    }

    fn teardown(&mut self) -> Result<(), WireError> {
        Ok(())
    }

    fn is_durable(&mut self) -> bool {
        // The daemon's own banner says the read side is an in-memory demo engine. Claiming
        // durability here would be the adapter defect this project spent a commit removing
        // from GBS, reintroduced in a benchmark.
        false
    }

    fn unsupported(&self, workload: &str) -> Option<String> {
        nilestream_gap(workload)
    }
}

/// What `nilestreamd` cannot serve over the wire today, by workload.
///
/// A free function so it can be tested without a server, and so the list is in one place
/// where it can be deleted a line at a time as the surface grows. Each entry is a **finding**
/// rather than a shortcoming of the benchmark: the engine's read path is real and its write
/// path is not exposed, and a results table that said otherwise would be the fabrication this
/// whole harness exists to replace.
pub fn nilestream_gap(workload: &str) -> Option<String> {
    match workload {
        "oltp" | "durable" => Some(
            "nilestreamd exposes no write surface over the wire; the write path is exercised \
             in-process by the conservation suite and is not comparable here"
                .into(),
        ),
        "analytical" => Some(
            "the server's read surface serves per-key balances; a scan-and-group-by surface \
             is not exposed"
                .into(),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_target_that_cannot_run_a_workload_says_so_with_a_reason() {
        // The discipline that keeps a `NOT RUN` from becoming a fabricated number. Every
        // refusal names what is missing, so the results table reports a gap rather than
        // filling it in.
        for w in ["oltp", "durable", "analytical"] {
            let reason = nilestream_gap(w).unwrap_or_else(|| panic!("{w} should be refused"));
            assert!(reason.len() > 40, "a reason, not a shrug: {reason}");
        }
        assert_eq!(nilestream_gap("point"), None, "the read path is real and is measured");
    }

    #[test]
    fn the_reasons_name_what_is_missing_rather_than_that_something_is() {
        let oltp = nilestream_gap("oltp").unwrap();
        assert!(oltp.contains("no write surface"), "{oltp}");
        let analytical = nilestream_gap("analytical").unwrap();
        assert!(analytical.contains("per-key balances"), "{analytical}");
    }
}
