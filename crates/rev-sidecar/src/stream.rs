//! The replication consumer: `pgoutput` in, epochs applied to the REV, frontier published.

use crate::base::PgBase;
use crate::pgoutput::{self, Frame, Message};
use bank_bench::wire::Client;
use nilestream_core::rev::{Key, Runtime, Value};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// What the stream thread and every session share.
pub struct Shared {
    pub base: Arc<PgBase>,
    pub rt: Mutex<Runtime>,
    pub view: String,
}

/// Where `arm.postings`' columns are in the stream's tuples, learned from its `Relation`.
#[derive(Debug, Clone, Copy)]
struct Columns {
    epoch: usize,
    acct: usize,
    cur: usize,
    amt: usize,
}

fn micros_now() -> u64 {
    // The protocol's clock is microseconds since 2000-01-01; the server uses it only for
    // lag reporting, so the Unix epoch offset is applied for honesty and nothing depends on it.
    const PG_EPOCH_OFFSET_S: u64 = 946_684_800;
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
        .saturating_sub(PG_EPOCH_OFFSET_S * 1_000_000)
}

/// Open the replication connection and a temporary slot (created *after* the load, so the
/// stream carries exactly the writes that follow it), start streaming, and report readiness.
pub fn open(port: u16, slot: &str, publication: &str) -> Result<Client, String> {
    let mut c = Client::connect_replication("127.0.0.1", port, "bench", "bank")
        .map_err(|e| format!("replication connection: {e}"))?;
    c.simple(&format!(
        "CREATE_REPLICATION_SLOT {slot} TEMPORARY LOGICAL pgoutput"
    ))
    .map_err(|e| format!("CREATE_REPLICATION_SLOT: {e}"))?;
    c.start_copy_both(&format!(
        "START_REPLICATION SLOT {slot} LOGICAL 0/0 (proto_version '1', publication_names '{publication}')"
    ))
    .map_err(|e| format!("START_REPLICATION: {e}"))?;
    Ok(c)
}

/// Consume the stream until it ends or fails. Every committed transaction's postings become
/// one epoch's deltas, recorded in the base, applied to the view, and published as the new
/// frontier — in that order, so a reader that sees the frontier sees a view that has it.
pub fn run(mut c: Client, shared: Arc<Shared>) -> Result<(), String> {
    let mut relations: BTreeMap<u32, Option<Columns>> = BTreeMap::new();
    let mut pending: Vec<(i64, i64, i64, i128)> = Vec::new();
    let mut applied_lsn = 0u64;
    loop {
        let Some(data) = c.copy_recv().map_err(|e| format!("stream: {e}"))? else {
            return Ok(());
        };
        match pgoutput::frame(&data).map_err(|e| e.0)? {
            Frame::Keepalive {
                reply_requested, ..
            } => {
                if reply_requested {
                    c.copy_send(&pgoutput::status_update(applied_lsn, micros_now(), false))
                        .map_err(|e| format!("status update: {e}"))?;
                }
            }
            Frame::XLogData { message, .. } => {
                match pgoutput::message(&message).map_err(|e| e.0)? {
                    Message::Relation {
                        oid,
                        namespace,
                        name,
                        columns,
                    } => {
                        let at = |n: &str| columns.iter().position(|c| c == n);
                        let cols = if namespace == "arm" && name == "postings" {
                            match (at("epoch"), at("acct"), at("cur"), at("amt")) {
                                (Some(epoch), Some(acct), Some(cur), Some(amt)) => Some(Columns {
                                    epoch,
                                    acct,
                                    cur,
                                    amt,
                                }),
                                _ => {
                                    return Err(format!("arm.postings lacks a column: {columns:?}"))
                                }
                            }
                        } else {
                            None
                        };
                        relations.insert(oid, cols);
                    }
                    Message::Begin { .. } => pending.clear(),
                    Message::Insert { relation, values } => {
                        let Some(cols) = relations.get(&relation).copied().flatten() else {
                            continue;
                        };
                        let int = |i: usize| -> Result<i128, String> {
                            values
                                .get(i)
                                .cloned()
                                .flatten()
                                .and_then(|s| s.parse::<i128>().ok())
                                .ok_or_else(|| format!("arm.postings column {i} is not an integer"))
                        };
                        pending.push((
                            int(cols.epoch)? as i64,
                            int(cols.acct)? as i64,
                            int(cols.cur)? as i64,
                            int(cols.amt)?,
                        ));
                    }
                    Message::Commit { end_lsn, .. } => {
                        applied_lsn = end_lsn;
                        if pending.is_empty() {
                            continue;
                        }
                        let e = pending[0].0;
                        if pending.iter().any(|p| p.0 != e) {
                            return Err(format!(
                                "one transaction carried postings of several epochs: {:?}",
                                pending.iter().map(|p| p.0).collect::<Vec<_>>()
                            ));
                        }
                        let e = e as u64;
                        let f = shared.base.frontier();
                        if e <= f {
                            return Err(format!(
                                "epoch {e} arrived at frontier {f}: epochs must commit in order"
                            ));
                        }
                        let mut per_key: BTreeMap<Key, Value> = BTreeMap::new();
                        for (_, a, cur, amt) in pending.drain(..) {
                            *per_key.entry(vec![a, cur]).or_default() += amt;
                        }
                        commit(&shared, e, per_key);
                        c.copy_send(&pgoutput::status_update(applied_lsn, micros_now(), false))
                            .map_err(|e| format!("status update: {e}"))?;
                    }
                    Message::Forbidden(tag) => {
                        return Err(format!(
                            "the stream carried a `{tag}` message: the base is append-only, so a \
                         view of it cannot follow an update, delete or truncate"
                        ))
                    }
                    Message::Skipped(_) => {}
                }
            }
        }
    }
}

/// Record an epoch's deltas, apply them to the view, then publish it as the frontier — in
/// that order, so a reader that sees the frontier sees a view that has it.
fn commit(shared: &Shared, e: u64, per_key: BTreeMap<Key, Value>) {
    shared.base.record(e, per_key.into_iter().collect());
    shared.rt.lock().unwrap().advance(shared.base.as_ref(), e);
    shared.base.publish(e);
}

/// Arm T: consume TigerBeetle's change stream as the driver forwards it from RabbitMQ —
/// `E <epoch>`, then `D <acct> <cur> <delta>` lines, then `.` — one epoch per transfer.
pub fn run_driver(addr: &str, shared: Arc<Shared>) -> Result<(), String> {
    use std::io::{BufRead, BufReader, Write};
    let mut s = std::net::TcpStream::connect(addr).map_err(|e| format!("driver {addr}: {e}"))?;
    s.write_all(b"SUBSCRIBE\n").map_err(|e| e.to_string())?;
    let r = BufReader::new(s);
    let mut epoch: Option<u64> = None;
    let mut per_key: BTreeMap<Key, Value> = BTreeMap::new();
    for line in r.lines() {
        let line = line.map_err(|e| format!("driver stream: {e}"))?;
        let w: Vec<&str> = line.split_whitespace().collect();
        match w.as_slice() {
            ["E", e] => {
                epoch = Some(e.parse().map_err(|_| format!("bad epoch line {line:?}"))?);
                per_key.clear();
            }
            ["D", a, c, d] => {
                let num = |x: &str| {
                    x.parse::<i64>()
                        .map_err(|_| format!("bad delta line {line:?}"))
                };
                *per_key.entry(vec![num(a)?, num(c)?]).or_default() += num(d)? as i128;
            }
            ["."] => {
                let e = epoch.take().ok_or("a delta block without an epoch")?;
                let f = shared.base.frontier();
                if e <= f {
                    return Err(format!("epoch {e} arrived at frontier {f}: out of order"));
                }
                commit(&shared, e, std::mem::take(&mut per_key));
            }
            _ => return Err(format!("unexpected line from the driver: {line:?}")),
        }
    }
    Ok(())
}

use nilestream_core::rev::Base as _;
