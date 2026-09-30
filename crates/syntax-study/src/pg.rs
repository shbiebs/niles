//! **The PostgreSQL executor**, for SQL and for the SQL PRQL emits (design §2): each run is
//! `begin`, a scratch schema, the surface's schema, the dataset, the program, the harness's
//! reads, `set constraints all immediate` where a transaction ran (A1), then `rollback` —
//! nothing a run does survives it.

use crate::data::Dataset;
use crate::kinds::day_iso;
use bank_bench::wire::{Client, WireError};
use std::fmt::Write as _;

pub const SCHEMA_SQL: &str = include_str!("../corpus/schema/schema.sql");
pub const SCHEMA_PLAIN: &str = include_str!("../corpus/schema/schema_plain.sql");

/// Why a run produced no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlErr {
    /// PostgreSQL raised; `at` says which step (the program's definition, or its execution).
    Server {
        at: &'static str,
        sqlstate: String,
        message: String,
    },
    /// The harness could not run at all (no server, no role): a blocked run, never a verdict.
    Blocked(String),
}

pub fn connect() -> Result<Client, SqlErr> {
    let port = std::env::var("PGPORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5432);
    Client::connect("127.0.0.1", port, "bench", "bank").map_err(|e| {
        SqlErr::Blocked(format!(
            "no PostgreSQL as `bench` on 127.0.0.1:{port}: {e}; run tools/pg-provision.sh"
        ))
    })
}

fn server(at: &'static str, e: WireError) -> SqlErr {
    match e {
        WireError::Server { sqlstate, message } => SqlErr::Server {
            at,
            sqlstate,
            message,
        },
        other => SqlErr::Blocked(other.to_string()),
    }
}

/// The dataset as `insert` statements, for the typed schema (`typed`) or the plain one.
pub fn data_sql(d: &Dataset, typed: bool) -> String {
    let mut s = String::new();
    s += "insert into parties values ";
    s += &d
        .parties
        .iter()
        .map(|p| {
            format!(
                "({}, {})",
                p.id,
                p.parent
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "null".into())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    s += ";\ninsert into accounts values ";
    s += &d
        .accounts
        .iter()
        .map(|a| format!("({}, {}, {})", a.id, a.owner, a.desk))
        .collect::<Vec<_>>()
        .join(", ");
    s += ";\ninsert into postings values ";
    s += &d
        .postings
        .iter()
        .map(|p| {
            let money = if typed {
                let cols: Vec<String> = ["usd", "eur", "jpy"]
                    .iter()
                    .map(|c| {
                        if *c == p.cur {
                            format!("row({})::{c}", p.amt)
                        } else {
                            format!("null::{c}")
                        }
                    })
                    .collect();
                cols.join(", ")
            } else {
                p.amt.to_string()
            };
            format!(
                "({}, {}, '{}', {money}, {}, date '{}')",
                p.txn,
                p.acct,
                p.cur,
                p.epoch,
                day_iso(p.value_date)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n  ");
    s += ";\ninsert into holds values ";
    s += &d
        .holds
        .iter()
        .map(|h| {
            if typed {
                let (u, e) = if h.cur == "usd" {
                    (format!("row({})::usd", h.amount), "null::eur".to_string())
                } else {
                    ("null::usd".to_string(), format!("row({})::eur", h.amount))
                };
                format!("({}, {}, '{}', {u}, {e}, {})", h.id, h.acct, h.cur, h.open)
            } else {
                format!(
                    "({}, {}, '{}', {}, {})",
                    h.id, h.acct, h.cur, h.amount, h.open
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    s += ";\n";
    let _ = writeln!(s);
    s
}

/// One run: set up, then each `(label, sql)` step in order; the rows of the steps whose label
/// starts with `read` are returned, in order. Always rolled back.
pub fn run(
    c: &mut Client,
    schema: &str,
    data: &str,
    program: &str,
    steps: &[(&'static str, String)],
) -> Result<Vec<Vec<Vec<String>>>, SqlErr> {
    c.simple("begin").map_err(|e| server("setup", e))?;
    let out = (|| {
        c.simple("create schema e30_run")
            .map_err(|e| server("setup", e))?;
        c.simple("set local search_path = e30_run")
            .map_err(|e| server("setup", e))?;
        c.simple(schema).map_err(|e| server("setup", e))?;
        c.simple(data).map_err(|e| server("setup", e))?;
        c.simple("set constraints all immediate")
            .map_err(|e| server("setup", e))?;
        c.simple("set constraints all deferred")
            .map_err(|e| server("setup", e))?;
        if !program.trim().is_empty() {
            c.simple(program).map_err(|e| server("definition", e))?;
        }
        let mut reads = Vec::new();
        for (label, sql) in steps {
            let r = c.simple(sql).map_err(|e| server("execution", e))?;
            if label.starts_with("read") {
                reads.push(
                    r.rows
                        .into_iter()
                        .map(|row| {
                            row.into_iter()
                                .map(|v| v.unwrap_or_else(|| "NULL".into()))
                                .collect()
                        })
                        .collect(),
                );
            }
        }
        Ok(reads)
    })();
    let _ = c.simple("rollback");
    out
}
