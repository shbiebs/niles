//! A minimal PostgreSQL wire-protocol client, in `std` only.
//!
//! # Why the harness has its own client
//!
//! Two reasons, and the second is the one that matters.
//!
//! **The workspace has no external dependencies.** Not one crate in Nilestream pulls
//! anything from outside this repository, and a benchmark is a bad place to break that: the
//! client is on the measured path, so its version becomes part of every number reported.
//!
//! **Both targets must pay the same client cost.** `SPEC-ENGINE.md` Part 0 compares
//! Nilestream to PostgreSQL, and a comparison where one side is driven in-process and the
//! other over TCP is not a comparison — it silently omits the protocol cost that only one
//! side pays. One client, one code path, two servers. It also means this client exercises
//! Nilestream's own `pg_wire` implementation from the other side, which is a second thing
//! worth having.
//!
//! # Scope, stated plainly
//!
//! Simple query protocol, extended query protocol (parse/bind/describe/execute/sync), trust
//! authentication, no TLS, text result format. That is what the benchmark sends and it is
//! deliberately not more: every message type implemented here is one that has to be correct,
//! and an unused one is a liability rather than an asset.
//!
//! MD5 and SCRAM authentication are **refused with a named error** rather than half-supported
//! — a client that silently failed to authenticate would look like a slow server.

use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// What can go wrong between here and a server.
#[derive(Debug)]
pub enum WireError {
    Io(std::io::Error),
    /// The server demanded an authentication method this client does not implement.
    ///
    /// Named rather than generic: `password` in `pg_hba.conf` where the harness expects
    /// `trust` is the single most likely reason a benchmark run fails to start, and
    /// "authentication failed" would send an operator to the wrong file.
    Auth(String),
    /// The server returned an `ErrorResponse`. Carries the SQLSTATE and the message.
    Server { sqlstate: String, message: String },
    /// A message this client does not understand, or one that arrived out of order.
    Protocol(String),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::Io(e) => write!(f, "{e}"),
            WireError::Auth(m) => write!(
                f,
                "the server requires {m}, which this client does not implement. The harness \
                 expects `trust` authentication; check pg_hba.conf"
            ),
            WireError::Server { sqlstate, message } => write!(f, "[{sqlstate}] {message}"),
            WireError::Protocol(m) => write!(f, "protocol: {m}"),
        }
    }
}

impl From<std::io::Error> for WireError {
    fn from(e: std::io::Error) -> Self {
        WireError::Io(e)
    }
}

/// One result set: column names and rows of nullable text.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Rows {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
    /// The `CommandComplete` tag, e.g. `SELECT 3` or `INSERT 0 1`.
    pub tag: String,
}

impl Rows {
    /// Column `col` of the first row, parsed as an `i128`.
    ///
    /// `None` rather than zero for a missing row **and** for a SQL NULL: an account with no
    /// postings and an account with a zero balance are different facts, and this is the layer
    /// where the difference would be quietest to lose. A client that flattened NULL to zero
    /// would make every honest absence in the engine invisible from the outside — which is the
    /// §1.1.1 defect, one layer further out than usual.
    pub fn nth(&self, col: usize) -> Option<i128> {
        self.rows.first()?.get(col)?.as_ref()?.trim().parse().ok()
    }

    /// The first column of the first row.
    ///
    /// Note that a `select acct, sum(amt) ... group by acct` returns the *key* first, so this
    /// is the account and not the balance. Use [`nth`](Self::nth) or [`by_name`](Self::by_name)
    /// when the value is wanted; the name is kept because a single-column query is common.
    pub fn scalar(&self) -> Option<i128> {
        self.nth(0)
    }

    /// The named column of the first row.
    ///
    /// The two servers name the same quantity differently — PostgreSQL calls it `sum` and
    /// Nilestream calls it `value` — so a test that asserts on a balance should say which
    /// column it means rather than counting positions and being right by accident.
    pub fn by_name(&self, column: &str) -> Option<i128> {
        let i = self.columns.iter().position(|c| c == column)?;
        self.nth(i)
    }
}

/// A connection to a PostgreSQL-wire server.
pub struct Client {
    r: BufReader<TcpStream>,
    w: TcpStream,
    /// The backend's process id, from `BackendKeyData`. Useful in a server log when a run
    /// misbehaves.
    pub backend_pid: i32,
    prepared: u32,
}

impl Client {
    /// Connect, start up, and wait for `ReadyForQuery`.
    pub fn connect(host: &str, port: u16, user: &str, database: &str) -> Result<Client, WireError> {
        let stream = TcpStream::connect((host, port))?;
        // Nagle would batch a small query with the next write and add a round trip's worth
        // of latency to every point lookup — which is the workload most sensitive to it, and
        // the one whose number would then be measuring the algorithm least.
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(60)))?;
        let mut c = Client {
            r: BufReader::new(stream.try_clone()?),
            w: stream,
            backend_pid: 0,
            prepared: 0,
        };
        c.startup(user, database)?;
        Ok(c)
    }

    fn startup(&mut self, user: &str, database: &str) -> Result<(), WireError> {
        let mut body = Vec::new();
        body.extend_from_slice(&196_608i32.to_be_bytes()); // protocol 3.0
        for (k, v) in [("user", user), ("database", database), ("client_encoding", "UTF8")] {
            body.extend_from_slice(k.as_bytes());
            body.push(0);
            body.extend_from_slice(v.as_bytes());
            body.push(0);
        }
        body.push(0);
        let mut msg = ((body.len() + 4) as i32).to_be_bytes().to_vec();
        msg.extend_from_slice(&body);
        self.w.write_all(&msg)?;
        self.w.flush()?;

        loop {
            let (tag, body) = self.read_message()?;
            match tag {
                b'R' => {
                    let code = i32::from_be_bytes(body[0..4].try_into().unwrap());
                    match code {
                        0 => {}  // AuthenticationOk
                        5 => return Err(WireError::Auth("MD5 password authentication".into())),
                        10 => return Err(WireError::Auth("SCRAM-SHA-256 authentication".into())),
                        3 => return Err(WireError::Auth("cleartext password authentication".into())),
                        other => {
                            return Err(WireError::Auth(format!("authentication method {other}")))
                        }
                    }
                }
                b'K' => {
                    self.backend_pid = i32::from_be_bytes(body[0..4].try_into().unwrap());
                }
                b'S' => {}                  // ParameterStatus
                b'Z' => return Ok(()),      // ReadyForQuery
                b'E' => return Err(error_from(&body)),
                other => {
                    return Err(WireError::Protocol(format!(
                        "unexpected message `{}` during startup",
                        other as char
                    )))
                }
            }
        }
    }

    /// Run one statement with the **simple** query protocol.
    pub fn simple(&mut self, sql: &str) -> Result<Rows, WireError> {
        let mut body = sql.as_bytes().to_vec();
        body.push(0);
        self.send(b'Q', &body)?;
        self.collect_until_ready()
    }

    /// Prepare a statement once, returning a name to execute repeatedly.
    ///
    /// The OLTP workload runs the same statement thousands of times, and re-parsing it every
    /// time would measure the parser rather than the engine. Preparing once is what a real
    /// application does, so it is what the benchmark does.
    pub fn prepare(&mut self, sql: &str, param_types: &[i32]) -> Result<String, WireError> {
        self.prepared += 1;
        let name = format!("bench{}", self.prepared);
        let mut body = Vec::new();
        put_cstr(&mut body, &name);
        put_cstr(&mut body, sql);
        body.extend_from_slice(&(param_types.len() as i16).to_be_bytes());
        for t in param_types {
            body.extend_from_slice(&t.to_be_bytes());
        }
        self.send(b'P', &body)?;
        self.send(b'S', &[])?; // Sync
        self.collect_until_ready()?;
        Ok(name)
    }

    /// Bind and execute a prepared statement with text parameters.
    pub fn execute(&mut self, name: &str, params: &[&str]) -> Result<Rows, WireError> {
        let mut bind = Vec::new();
        put_cstr(&mut bind, ""); // unnamed portal
        put_cstr(&mut bind, name);
        bind.extend_from_slice(&0i16.to_be_bytes()); // all parameters as text
        bind.extend_from_slice(&(params.len() as i16).to_be_bytes());
        for p in params {
            bind.extend_from_slice(&(p.len() as i32).to_be_bytes());
            bind.extend_from_slice(p.as_bytes());
        }
        bind.extend_from_slice(&0i16.to_be_bytes()); // all results as text
        self.send(b'B', &bind)?;

        let mut exec = Vec::new();
        put_cstr(&mut exec, "");
        exec.extend_from_slice(&0i32.to_be_bytes()); // no row limit
        self.send(b'E', &exec)?;
        self.send(b'S', &[])?;
        self.collect_until_ready()
    }

    fn send(&mut self, tag: u8, body: &[u8]) -> Result<(), WireError> {
        let mut msg = Vec::with_capacity(body.len() + 5);
        msg.push(tag);
        msg.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
        msg.extend_from_slice(body);
        self.w.write_all(&msg)?;
        self.w.flush()?;
        Ok(())
    }

    fn read_message(&mut self) -> Result<(u8, Vec<u8>), WireError> {
        let mut head = [0u8; 5];
        self.r.read_exact(&mut head)?;
        let len = i32::from_be_bytes(head[1..5].try_into().unwrap());
        if !(4..=64 * 1024 * 1024).contains(&len) {
            return Err(WireError::Protocol(format!("implausible message length {len}")));
        }
        let mut body = vec![0u8; (len - 4) as usize];
        self.r.read_exact(&mut body)?;
        Ok((head[0], body))
    }

    /// Read until `ReadyForQuery`, assembling whatever result set arrived.
    ///
    /// An `ErrorResponse` is remembered and returned **after** the ready message, not thrown
    /// immediately: leaving unread messages in the socket desynchronises the connection, and
    /// the next query would fail for a reason that has nothing to do with itself.
    fn collect_until_ready(&mut self) -> Result<Rows, WireError> {
        let mut out = Rows::default();
        let mut failure: Option<WireError> = None;
        loop {
            let (tag, body) = self.read_message()?;
            match tag {
                b'T' => out.columns = parse_row_description(&body),
                b'D' => out.rows.push(parse_data_row(&body)),
                b'C' => out.tag = cstr_at(&body, 0).0,
                b'E' => failure = Some(error_from(&body)),
                b'Z' => break,
                // NoticeResponse, ParameterStatus, EmptyQueryResponse, ParseComplete,
                // BindComplete, NoData, PortalSuspended — all expected and all uninteresting
                // to a benchmark.
                b'N' | b'S' | b'I' | b'1' | b'2' | b'3' | b'n' | b's' | b't' => {}
                other => {
                    return Err(WireError::Protocol(format!(
                        "unexpected message `{}`",
                        other as char
                    )))
                }
            }
        }
        match failure {
            Some(e) => Err(e),
            None => Ok(out),
        }
    }
}

fn put_cstr(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(s.as_bytes());
    out.push(0);
}

fn cstr_at(b: &[u8], from: usize) -> (String, usize) {
    let end = b[from..].iter().position(|&c| c == 0).map_or(b.len(), |i| from + i);
    (String::from_utf8_lossy(&b[from..end]).into_owned(), end + 1)
}

fn parse_row_description(b: &[u8]) -> Vec<String> {
    if b.len() < 2 {
        return Vec::new();
    }
    let n = i16::from_be_bytes(b[0..2].try_into().unwrap()) as usize;
    let mut out = Vec::with_capacity(n);
    let mut i = 2;
    for _ in 0..n {
        let (name, next) = cstr_at(b, i);
        out.push(name);
        i = next + 18; // table oid, column no, type oid, size, modifier, format
        if i > b.len() {
            break;
        }
    }
    out
}

fn parse_data_row(b: &[u8]) -> Vec<Option<String>> {
    if b.len() < 2 {
        return Vec::new();
    }
    let n = i16::from_be_bytes(b[0..2].try_into().unwrap()) as usize;
    let mut out = Vec::with_capacity(n);
    let mut i = 2;
    for _ in 0..n {
        if i + 4 > b.len() {
            break;
        }
        let len = i32::from_be_bytes(b[i..i + 4].try_into().unwrap());
        i += 4;
        if len < 0 {
            out.push(None); // SQL NULL, which is not an empty string
            continue;
        }
        let end = (i + len as usize).min(b.len());
        out.push(Some(String::from_utf8_lossy(&b[i..end]).into_owned()));
        i = end;
    }
    out
}

fn error_from(b: &[u8]) -> WireError {
    let mut sqlstate = String::new();
    let mut message = String::new();
    let mut i = 0;
    while i < b.len() && b[i] != 0 {
        let field = b[i];
        let (value, next) = cstr_at(b, i + 1);
        match field {
            b'C' => sqlstate = value,
            b'M' => message = value,
            _ => {}
        }
        i = next;
    }
    WireError::Server { sqlstate, message }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_data_row_distinguishes_null_from_an_empty_string() {
        // The distinction this harness exists to respect. A NULL balance and a zero balance
        // are different facts, and a client that flattened them would make every honest
        // absence in the engine invisible from the outside.
        let mut b = Vec::new();
        b.extend_from_slice(&2i16.to_be_bytes());
        b.extend_from_slice(&(-1i32).to_be_bytes()); // NULL
        b.extend_from_slice(&0i32.to_be_bytes()); // empty string
        assert_eq!(parse_data_row(&b), vec![None, Some(String::new())]);
    }

    #[test]
    fn a_row_description_yields_the_column_names_in_order() {
        let mut b = Vec::new();
        b.extend_from_slice(&2i16.to_be_bytes());
        for name in ["acct", "balance"] {
            put_cstr(&mut b, name);
            b.extend_from_slice(&[0u8; 18]);
        }
        assert_eq!(parse_row_description(&b), vec!["acct", "balance"]);
    }

    #[test]
    fn an_error_response_carries_the_sqlstate_and_the_message() {
        let mut b = Vec::new();
        b.push(b'S');
        put_cstr(&mut b, "ERROR");
        b.push(b'C');
        put_cstr(&mut b, "42P01");
        b.push(b'M');
        put_cstr(&mut b, "relation \"nope\" does not exist");
        b.push(0);
        match error_from(&b) {
            WireError::Server { sqlstate, message } => {
                assert_eq!(sqlstate, "42P01");
                assert!(message.contains("does not exist"));
            }
            other => panic!("{other}"),
        }
    }

    #[test]
    fn a_reading_reports_absence_as_absence_whether_it_is_a_missing_row_or_a_null() {
        let present = Rows {
            columns: vec!["sum".into()],
            rows: vec![vec![Some("-4200".into())]],
            tag: "SELECT 1".into(),
        };
        assert_eq!(present.scalar(), Some(-4_200));

        let no_rows = Rows { columns: vec!["sum".into()], rows: vec![], tag: "SELECT 0".into() };
        assert_eq!(no_rows.scalar(), None, "no row is not zero");

        let null = Rows {
            columns: vec!["sum".into()],
            rows: vec![vec![None]],
            tag: "SELECT 1".into(),
        };
        assert_eq!(null.scalar(), None, "and neither is NULL");
    }

    #[test]
    fn a_balance_is_read_by_name_because_the_two_servers_name_it_differently() {
        // `select acct, sum(amt) ... group by acct` puts the *key* first on both servers, so
        // `scalar()` is the account number. A test that asserted on position would be right by
        // accident on one server and wrong on the other.
        let pg = Rows {
            columns: vec!["acct".into(), "sum".into()],
            rows: vec![vec![Some("42".into()), Some("-4200".into())]],
            tag: "SELECT 1".into(),
        };
        assert_eq!(pg.scalar(), Some(42), "the key, not the balance");
        assert_eq!(pg.nth(1), Some(-4_200));
        assert_eq!(pg.by_name("sum"), Some(-4_200));

        let nls = Rows {
            columns: vec!["key".into(), "value".into(), "anchor".into()],
            rows: vec![vec![Some("42".into()), Some("-4200".into()), Some("7".into())]],
            tag: "SELECT 1".into(),
        };
        assert_eq!(nls.by_name("value"), Some(-4_200), "the same quantity, a different name");
        assert_eq!(nls.by_name("sum"), None, "and asking for the wrong name yields nothing");
    }

    #[test]
    fn an_unsupported_authentication_method_says_which_file_to_look_in() {
        // The single most likely reason a benchmark run fails to start, and a generic
        // "authentication failed" would send an operator to the wrong place.
        let e = WireError::Auth("SCRAM-SHA-256 authentication".into());
        let msg = e.to_string();
        assert!(msg.contains("SCRAM"), "{msg}");
        assert!(msg.contains("pg_hba.conf"), "{msg}");
    }

    #[test]
    fn a_truncated_data_row_yields_what_it_can_rather_than_panicking() {
        // A benchmark that panicked on a malformed row would report a crash where a server
        // returned something unexpected, and the operator would debug the wrong process.
        let mut b = Vec::new();
        b.extend_from_slice(&3i16.to_be_bytes());
        b.extend_from_slice(&4i32.to_be_bytes());
        b.extend_from_slice(b"ab"); // claims four bytes, holds two
        let row = parse_data_row(&b);
        assert_eq!(row.len(), 1);
        assert_eq!(row[0].as_deref(), Some("ab"));
    }
}
