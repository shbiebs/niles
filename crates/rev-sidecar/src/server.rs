//! The sidecar's PostgreSQL-wire listener: REV reads answered here, the rest relayed.

use crate::stream::Shared;
use bank_bench::wire::{Client, WireError};
use nilestream_core::rev::{Base, ReadOutcome};
use nilestream_server::pg_wire::{self, Backend, Field, Frontend};
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

/// How long a write waits for the stream to deliver its epoch before it is reported failed.
const WRITE_VISIBLE_TIMEOUT: Duration = Duration::from_secs(30);

pub fn serve(listener: TcpListener, shared: Arc<Shared>) {
    for (n, conn) in listener.incoming().enumerate() {
        let Ok(stream) = conn else { continue };
        let shared = Arc::clone(&shared);
        std::thread::spawn(move || {
            let _ = session(stream, shared, n as u32 + 1);
        });
    }
}

fn session(stream: TcpStream, shared: Arc<Shared>, pid: u32) -> std::io::Result<()> {
    stream.set_nodelay(true)?;
    let mut w = stream.try_clone()?;
    let mut r = BufReader::new(stream);
    loop {
        match pg_wire::read_startup(&mut r)? {
            Frontend::SslRequest => {
                w.write_all(b"N")?;
                w.flush()?;
            }
            Frontend::Startup { .. } => break,
            _ => return Ok(()),
        }
    }
    pg_wire::write_all(&mut w, &pg_wire::startup_reply(pid, 0))?;
    let mut upstream: Option<Client> = None;
    loop {
        let reply = match pg_wire::read_message(&mut r)? {
            Frontend::Query(sql) => answer(&shared, &mut upstream, &sql),
            Frontend::Terminate => return Ok(()),
            _ => vec![pg_wire::unsupported(
                "this message",
                "the REV sidecar serves the simple query protocol only",
            )],
        };
        let mut out = reply;
        out.push(Backend::ReadyForQuery(b'I'));
        pg_wire::write_all(&mut w, &out)?;
    }
}

fn error(code: &str, message: String) -> Backend {
    Backend::ErrorResponse {
        severity: "ERROR".into(),
        code: code.into(),
        message,
        detail: None,
    }
}

/// `rev_read(A, C)` or `rev_read(A, C, E)`: the integers inside the call.
fn rev_read_args(sql: &str) -> Option<Vec<i64>> {
    let s = sql.trim().trim_end_matches(';').to_ascii_lowercase();
    let rest = s.strip_prefix("select value from rev_read(")?;
    let inner = rest.strip_suffix(')')?;
    inner.split(',').map(|x| x.trim().parse().ok()).collect()
}

fn answer(shared: &Shared, upstream: &mut Option<Client>, sql: &str) -> Vec<Backend> {
    if let Some(args) = rev_read_args(sql) {
        return match args.as_slice() {
            [a, c] => read(shared, *a, *c, None),
            [a, c, e] if *e >= 0 => read(shared, *a, *c, Some(*e as u64)),
            _ => vec![error(
                "42601",
                format!("rev_read takes (acct, cur[, epoch]), not {args:?}"),
            )],
        };
    }
    if sql.trim().eq_ignore_ascii_case("select rev_stats") {
        return stats(shared);
    }
    relay(shared, upstream, sql)
}

fn read(shared: &Shared, a: i64, c: i64, at: Option<u64>) -> Vec<Backend> {
    let base = shared.base.as_ref();
    let frontier = base.frontier();
    let anchor = at.unwrap_or(frontier);
    if anchor > frontier {
        return vec![error(
            "22023",
            format!("epoch {anchor} is beyond the view's frontier {frontier}"),
        )];
    }
    let key = vec![a, c];
    // The two-phase read of `nilestream-core`: decide under the view lock, fold without it,
    // install under it again. A reader that finds a flight at its own anchor waits for it.
    let value = loop {
        let outcome = {
            let mut rt = shared.rt.lock().unwrap();
            let view = rt.view_mut(&shared.view).expect("the view is installed");
            view.begin_read(&key, anchor)
        };
        match outcome {
            ReadOutcome::Hit(ans) => break Some(ans.value),
            ReadOutcome::Fold(t) => {
                let (v, rows) = base.reconstruct(t.key(), t.anchor());
                let ans = {
                    let mut rt = shared.rt.lock().unwrap();
                    let view = rt.view_mut(&shared.view).expect("the view is installed");
                    view.finish_fold(t, v, rows)
                };
                // A key with no base rows through the anchor is absent, not zero.
                break (rows > 0).then_some(ans.value);
            }
            ReadOutcome::Join(w) => {
                if let Some(j) = w.wait() {
                    break (j.base_rows > 0).then_some(j.answer.value);
                }
            }
        }
    };
    vec![
        Backend::RowDescription(vec![Field::numeric("value"), Field::int8("anchor")]),
        Backend::DataRow(vec![value.map(|v| v.to_string()), Some(anchor.to_string())]),
        Backend::CommandComplete("SELECT 1".into()),
    ]
}

fn stats(shared: &Shared) -> Vec<Backend> {
    let rt = shared.rt.lock().unwrap();
    let view = rt.view(&shared.view).expect("the view is installed");
    let s = view.stats;
    let cols: Vec<(&str, i128)> = vec![
        ("resident", view.resident_count() as i128),
        ("reads", s.reads as i128),
        ("hits", s.hits as i128),
        ("misses", s.misses as i128),
        ("upqueries", s.upqueries as i128),
        ("rows_touched", s.base_rows_read as i128),
        ("evictions", s.evictions as i128),
        ("deltas_applied", s.deltas_applied as i128),
        ("applied_through", view.applied_through() as i128),
        ("view_metadata_keys", view.metadata_len() as i128),
        ("view_slots", view.slots_len() as i128),
    ];
    vec![
        Backend::RowDescription(cols.iter().map(|(n, _)| Field::int8(n)).collect()),
        Backend::DataRow(cols.iter().map(|(_, v)| Some(v.to_string())).collect()),
        Backend::CommandComplete("SELECT 1".into()),
    ]
}

fn relay(shared: &Shared, upstream: &mut Option<Client>, sql: &str) -> Vec<Backend> {
    if upstream.is_none() {
        match Client::connect("127.0.0.1", shared.base.port, "bench", "bank") {
            Ok(c) => *upstream = Some(c),
            Err(e) => return vec![error("08006", format!("the ledger is unreachable: {e}"))],
        }
    }
    let c = upstream.as_mut().unwrap();
    let rows = match c.simple(sql) {
        Ok(rows) => rows,
        Err(WireError::Server { sqlstate, message }) => return vec![error(&sqlstate, message)],
        Err(e) => {
            *upstream = None;
            return vec![error("08006", format!("relay: {e}"))];
        }
    };
    // A write is acknowledged when its epoch is in the view, not merely in the ledger.
    if sql
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("select arm.post_txn(")
    {
        let e = match c.simple("select max(id) from arm.epochs") {
            Ok(r) => r.nth(0).unwrap_or(0) as u64,
            Err(e) => return vec![error("08006", format!("reading the new epoch: {e}"))],
        };
        let f = shared.base.wait_for(e, WRITE_VISIBLE_TIMEOUT);
        if f < e {
            return vec![error(
                "57014",
                format!(
                    "epoch {e} committed in the ledger but the stream had not delivered it to \
                     the view after {WRITE_VISIBLE_TIMEOUT:?} (frontier {f})"
                ),
            )];
        }
    }
    let mut out = Vec::with_capacity(rows.rows.len() + 2);
    if !rows.columns.is_empty() {
        out.push(Backend::RowDescription(
            rows.columns.iter().map(|n| Field::text(n)).collect(),
        ));
        for row in rows.rows {
            out.push(Backend::DataRow(row));
        }
    }
    out.push(Backend::CommandComplete(if rows.tag.is_empty() {
        "OK".into()
    } else {
        rows.tag
    }));
    out
}

#[cfg(test)]
mod tests {
    use super::rev_read_args;

    #[test]
    fn rev_read_is_recognised_and_nothing_else_is() {
        assert_eq!(
            rev_read_args("select value from rev_read(7, 0)"),
            Some(vec![7, 0])
        );
        assert_eq!(
            rev_read_args("SELECT value FROM rev_read(7, 1, 120);"),
            Some(vec![7, 1, 120])
        );
        assert_eq!(rev_read_args("select value from rev_read(7, x)"), None);
        assert_eq!(rev_read_args("select sum(amt) from arm.postings"), None);
    }
}
