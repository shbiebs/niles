//! The PostgreSQL wire protocol, version 3.
//!
//! # Why a wire protocol is a thesis-relevant artifact and not plumbing
//!
//! The adoption argument in §6.9 is that a new database is unadoptable if it also requires
//! new drivers, new tooling and a rewrite of everything that talks to the old one. Speaking
//! an existing wire protocol makes adoption incremental: an institution can point one
//! reporting job at Nilestream, compare its answers against the system of record, and
//! decide. That argument was, until this module, an argument. It is now a thing `psql` can
//! connect to.
//!
//! The design point worth stating: **the protocol is a surface, not a semantics.** A
//! PostgreSQL client sends a string; the server parses it as Niles (the SQL surface of
//! Appendix B.17), lowers it to the same IR as any other query, and serves it from the same
//! REV runtime under the same contract. There is no PostgreSQL compatibility *layer* with
//! its own execution path — which is exactly the seam this thesis argues against elsewhere,
//! and it would be inconsistent to build one here.
//!
//! # What is implemented, and what is deliberately not
//!
//! Implemented: the startup exchange, trust authentication, the **simple query** protocol
//! (`Query` → `RowDescription` → `DataRow`* → `CommandComplete` → `ReadyForQuery`), error
//! and notice responses, parameter status, backend key data, and termination.
//!
//! Not implemented, and each for a stated reason rather than for lack of time:
//!
//! * **The extended query protocol** (`Parse`/`Bind`/`Execute`). It is the right thing for a
//!   production server and it needs a prepared-statement cache keyed by an epoch, because a
//!   plan prepared at one visibility frontier may not be valid at another. That is a real
//!   design question this thesis has not answered, and shipping a version that ignored it
//!   would be worse than not shipping one.
//! * **Real authentication.** Trust only. A ledger that authenticated over a protocol this
//!   session cannot make confidential would be security theatre.
//! * **TLS.** Same reason. The `SSLRequest` handshake is answered with a refusal, correctly,
//!   rather than ignored.
//! * **The binary format.** Text only. Binary is an optimisation, and an unmeasured one here.
//!
//! Anything a client asks for that is not implemented gets an `ErrorResponse` naming what is
//! missing. A server that silently ignored a `Bind` would return results for a query the
//! client did not ask.

use std::io::{Read, Write};

/// A message the frontend can send.
#[derive(Debug, Clone, PartialEq)]
pub enum Frontend {
    /// The first message, which has no type byte — the protocol's one irregularity.
    Startup {
        version: u32,
        params: Vec<(String, String)>,
    },
    /// A request to negotiate TLS, also without a type byte.
    SslRequest,
    /// `Q` — the simple query protocol.
    Query(String),
    /// `X` — goodbye.
    Terminate,
    /// `P`, `B`, `E`, `D`, `S`, `C`, `H`, `F` — the extended query protocol.
    ///
    /// The body travels with the tag. It used to be discarded at the decoder, which is
    /// why `extended.rs` — a plan cache with eleven tests and a written answer to the
    /// epoch-invalidation question — was reachable from nothing: the session could not
    /// have served a `Parse` even had it wanted to, because the statement text never
    /// arrived.
    Extended(u8, Vec<u8>),
    /// `p` — a password message.
    Password(Vec<u8>),
    Unknown(u8),
}

/// A column in a result.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    /// PostgreSQL type OID. Only three are used, and each is chosen deliberately.
    pub type_oid: u32,
    pub type_size: i16,
}

impl Field {
    /// `int8`. Epochs, counts and identifiers.
    pub fn int8(name: &str) -> Field {
        Field {
            name: name.into(),
            type_oid: 20,
            type_size: 8,
        }
    }
    /// `text`.
    pub fn text(name: &str) -> Field {
        Field {
            name: name.into(),
            type_oid: 25,
            type_size: -1,
        }
    }
    /// `numeric`, **not** `float8`.
    ///
    /// This is the one type choice in the module with a thesis behind it. A money column
    /// sent as a double is a money column a client will round, and the whole point of
    /// carrying exact minor units at a declared scale through the type system is lost at
    /// the last hop if the wire format throws it away. `numeric` is arbitrary-precision
    /// decimal in every PostgreSQL client, so the exactness survives the boundary.
    pub fn numeric(name: &str) -> Field {
        Field {
            name: name.into(),
            type_oid: 1700,
            type_size: -1,
        }
    }
}

/// A message the backend sends.
#[derive(Debug, Clone, PartialEq)]
pub enum Backend {
    AuthenticationOk,
    ParameterStatus(String, String),
    BackendKeyData {
        pid: u32,
        secret: u32,
    },
    /// `Z` — with the transaction status: `I` idle, `T` in a transaction, `E` failed.
    ReadyForQuery(u8),
    RowDescription(Vec<Field>),
    /// A row. `None` is SQL NULL, which is distinct from an empty string and from an
    /// evicted hole — three different absences, and conflating any two is a bug.
    DataRow(Vec<Option<String>>),
    CommandComplete(String),
    EmptyQueryResponse,
    ErrorResponse {
        severity: String,
        code: String,
        message: String,
        detail: Option<String>,
    },
    NoticeResponse {
        message: String,
    },
    // --- the extended protocol's acknowledgements ---
    //
    // Empty-bodied, one byte of tag each, and all four were missing: the module that
    // implements the extended protocol could not have replied without them.
    /// `1` — the statement parsed and is cached.
    ParseComplete,
    /// `2` — the portal is bound.
    BindComplete,
    /// `3` — the statement or portal is closed.
    CloseComplete,
    /// `n` — the statement returns no rows, the answer to a `Describe` of one that does not.
    NoData,
    /// **A run of already-encoded backend messages.**
    ///
    /// The served rows of a query arrive here, framed once into one buffer, rather than as
    /// ten thousand `DataRow(Vec<Option<String>>)` values. The reason is the cost: every one
    /// of those was a `Vec` for the row, a `String` per cell, a body `Vec` inside `encode`
    /// and an output `Vec` around it — six allocations per row, for a reply whose content is
    /// a handful of integers.
    ///
    /// Nothing about the bytes changes. `RowDescription` and `CommandComplete` stay ordinary
    /// messages either side of it, which is what keeps `Execute`'s "do not re-send the
    /// description" filter working and keeps a reply readable in a packet capture.
    /// [`decoded_rows`] parses rows back out of one, so a test can assert on what a client
    /// would see rather than on which representation produced it.
    Raw(Vec<u8>),
}

/// Format an `i128` into `out` without allocating a `String` for it.
///
/// `i128::to_string` allocates, and a served reply of ten thousand rows with three integer
/// cells each allocated thirty thousand short strings whose only purpose was to be copied
/// into a buffer and dropped. The digits are produced into a stack array and copied once.
fn put_i128(out: &mut Vec<u8>, mut v: i128) {
    if v == 0 {
        out.push(b'0');
        return;
    }
    // 39 digits is the widest `i128`, plus a sign.
    let mut buf = [0u8; 40];
    let mut i = buf.len();
    let neg = v < 0;
    // Accumulate negatively so `i128::MIN` does not overflow on negation.
    if !neg {
        v = -v;
    }
    while v != 0 {
        i -= 1;
        buf[i] = b'0' + (-(v % 10)) as u8;
        v /= 10;
    }
    if neg {
        i -= 1;
        buf[i] = b'-';
    }
    out.extend_from_slice(&buf[i..]);
}

/// **Frame one `DataRow` straight into a reply buffer.**
///
/// `cells` is the row; `null` cells are written as the -1 length that means SQL NULL, which
/// is not an empty string and not an evicted hole. The message's length prefix is patched in
/// after the body is written, so nothing is measured twice and nothing is allocated per row.
pub fn put_data_row(out: &mut Vec<u8>, cells: impl IntoIterator<Item = Option<i128>>) {
    out.push(b'D');
    let len_at = out.len();
    out.extend_from_slice(&0i32.to_be_bytes());
    let count_at = out.len();
    out.extend_from_slice(&0i16.to_be_bytes());
    let mut n: i16 = 0;
    for c in cells {
        n += 1;
        match c {
            None => out.extend_from_slice(&(-1i32).to_be_bytes()),
            Some(v) => {
                let cell_len_at = out.len();
                out.extend_from_slice(&0i32.to_be_bytes());
                let from = out.len();
                put_i128(out, v);
                let w = (out.len() - from) as i32;
                out[cell_len_at..cell_len_at + 4].copy_from_slice(&w.to_be_bytes());
            }
        }
    }
    out[count_at..count_at + 2].copy_from_slice(&n.to_be_bytes());
    let body = (out.len() - len_at) as i32;
    out[len_at..len_at + 4].copy_from_slice(&body.to_be_bytes());
}

/// The rows a client would decode from a message list, whichever representation carried them.
///
/// For tests. A `DataRow` contributes itself; a [`Backend::Raw`] is parsed for the `D` frames
/// inside it. The point is that a test asserts on what reaches the client, so moving rows
/// from one representation to the other cannot quietly change what is asserted.
pub fn decoded_rows(msgs: &[Backend]) -> Vec<Vec<Option<String>>> {
    let mut out = Vec::new();
    for m in msgs {
        match m {
            Backend::DataRow(r) => out.push(r.clone()),
            Backend::Raw(b) => {
                let mut at = 0usize;
                while at + 5 <= b.len() {
                    let tag = b[at];
                    let len =
                        i32::from_be_bytes([b[at + 1], b[at + 2], b[at + 3], b[at + 4]]) as usize;
                    let body = &b[at + 5..at + 1 + len];
                    at += 1 + len;
                    if tag != b'D' {
                        continue;
                    }
                    let n = i16::from_be_bytes([body[0], body[1]]) as usize;
                    let mut p = 2usize;
                    let mut row = Vec::with_capacity(n);
                    for _ in 0..n {
                        let w =
                            i32::from_be_bytes([body[p], body[p + 1], body[p + 2], body[p + 3]]);
                        p += 4;
                        if w < 0 {
                            row.push(None);
                        } else {
                            let w = w as usize;
                            row.push(Some(String::from_utf8_lossy(&body[p..p + w]).into_owned()));
                            p += w;
                        }
                    }
                    out.push(row);
                }
            }
            _ => {}
        }
    }
    out
}

/// Encode one message **into** a caller's buffer.
///
/// The buffered form is what a reply of many messages uses: `encode` allocates a body vector
/// and an output vector per message, which is two allocations for four bytes of
/// `CommandComplete` and twenty thousand for a ten-thousand-row answer.
pub fn encode_into(out: &mut Vec<u8>, msg: &Backend) {
    if let Backend::Raw(b) = msg {
        out.extend_from_slice(b);
        return;
    }
    out.extend_from_slice(&encode(msg));
}

pub fn encode(msg: &Backend) -> Vec<u8> {
    let (tag, body) = match msg {
        // Already framed, length prefixes and all: it is a run of messages rather than one,
        // so there is no tag to add and no length to compute.
        Backend::Raw(b) => return b.clone(),
        Backend::AuthenticationOk => (b'R', 0i32.to_be_bytes().to_vec()),
        Backend::ParseComplete => (b'1', Vec::new()),
        Backend::BindComplete => (b'2', Vec::new()),
        Backend::CloseComplete => (b'3', Vec::new()),
        Backend::NoData => (b'n', Vec::new()),
        Backend::ParameterStatus(k, v) => {
            let mut b = Vec::new();
            put_cstr(&mut b, k);
            put_cstr(&mut b, v);
            (b'S', b)
        }
        Backend::BackendKeyData { pid, secret } => {
            let mut b = Vec::new();
            b.extend_from_slice(&pid.to_be_bytes());
            b.extend_from_slice(&secret.to_be_bytes());
            (b'K', b)
        }
        Backend::ReadyForQuery(status) => (b'Z', vec![*status]),
        Backend::RowDescription(fields) => {
            let mut b = Vec::new();
            b.extend_from_slice(&(fields.len() as i16).to_be_bytes());
            for f in fields {
                put_cstr(&mut b, &f.name);
                b.extend_from_slice(&0u32.to_be_bytes()); // table oid: not from a table
                b.extend_from_slice(&0i16.to_be_bytes()); // column attnum
                b.extend_from_slice(&f.type_oid.to_be_bytes());
                b.extend_from_slice(&f.type_size.to_be_bytes());
                b.extend_from_slice(&(-1i32).to_be_bytes()); // type modifier
                b.extend_from_slice(&0i16.to_be_bytes()); // text format
            }
            (b'T', b)
        }
        Backend::DataRow(cols) => {
            let mut b = Vec::new();
            b.extend_from_slice(&(cols.len() as i16).to_be_bytes());
            for c in cols {
                match c {
                    // -1 length is NULL. Not an empty string: a client that could not tell
                    // those apart could not tell "no balance" from "zero balance".
                    None => b.extend_from_slice(&(-1i32).to_be_bytes()),
                    Some(s) => {
                        b.extend_from_slice(&(s.len() as i32).to_be_bytes());
                        b.extend_from_slice(s.as_bytes());
                    }
                }
            }
            (b'D', b)
        }
        Backend::CommandComplete(tag) => {
            let mut b = Vec::new();
            put_cstr(&mut b, tag);
            (b'C', b)
        }
        Backend::EmptyQueryResponse => (b'I', Vec::new()),
        Backend::ErrorResponse {
            severity,
            code,
            message,
            detail,
        } => {
            let mut b = Vec::new();
            b.push(b'S');
            put_cstr(&mut b, severity);
            b.push(b'C');
            put_cstr(&mut b, code);
            b.push(b'M');
            put_cstr(&mut b, message);
            if let Some(d) = detail {
                b.push(b'D');
                put_cstr(&mut b, d);
            }
            b.push(0);
            (b'E', b)
        }
        Backend::NoticeResponse { message } => {
            let mut b = Vec::new();
            b.push(b'S');
            put_cstr(&mut b, "NOTICE");
            b.push(b'M');
            put_cstr(&mut b, message);
            b.push(0);
            (b'N', b)
        }
    };
    let mut out = Vec::with_capacity(body.len() + 5);
    out.push(tag);
    out.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
    out.extend_from_slice(&body);
    out
}

fn put_cstr(b: &mut Vec<u8>, s: &str) {
    b.extend_from_slice(s.as_bytes());
    b.push(0);
}

pub fn get_cstr(b: &[u8], at: &mut usize) -> String {
    let start = *at;
    while *at < b.len() && b[*at] != 0 {
        *at += 1;
    }
    let s = String::from_utf8_lossy(&b[start..*at]).into_owned();
    *at += 1;
    s
}

/// Read the startup message, which has no type byte.
pub fn read_startup(r: &mut impl Read) -> std::io::Result<Frontend> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = i32::from_be_bytes(len_buf) as usize;
    if !(8..=10_000).contains(&len) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "implausible startup length",
        ));
    }
    let mut body = vec![0u8; len - 4];
    r.read_exact(&mut body)?;
    let version = u32::from_be_bytes(body[0..4].try_into().unwrap());
    // The magic number for `SSLRequest`: 1234 << 16 | 5679.
    if version == 80877103 {
        return Ok(Frontend::SslRequest);
    }
    let mut at = 4;
    let mut params = Vec::new();
    while at < body.len() && body[at] != 0 {
        let k = get_cstr(&body, &mut at);
        let v = get_cstr(&body, &mut at);
        params.push((k, v));
    }
    Ok(Frontend::Startup { version, params })
}

/// Read one ordinary message.
pub fn read_message(r: &mut impl Read) -> std::io::Result<Frontend> {
    let mut tag = [0u8; 1];
    r.read_exact(&mut tag)?;
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = i32::from_be_bytes(len_buf) as usize;
    if !(4..=64 * 1024 * 1024).contains(&len) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "implausible message length",
        ));
    }
    let mut body = vec![0u8; len - 4];
    r.read_exact(&mut body)?;
    Ok(match tag[0] {
        b'Q' => {
            let mut at = 0;
            Frontend::Query(get_cstr(&body, &mut at))
        }
        b'X' => Frontend::Terminate,
        b'p' => Frontend::Password(body),
        t @ (b'P' | b'B' | b'E' | b'D' | b'S' | b'C' | b'H' | b'F') => Frontend::Extended(t, body),
        other => Frontend::Unknown(other),
    })
}

/// The reply to a startup message. Trust authentication, then the parameters a client
/// needs before it will proceed.
pub fn startup_reply(pid: u32, secret: u32) -> Vec<Backend> {
    vec![
        Backend::AuthenticationOk,
        // `server_version` is what a client uses to decide which features to attempt. It is
        // reported honestly: this is Nilestream, and a client that assumes PostgreSQL
        // internals from a version string will be disappointed either way.
        Backend::ParameterStatus("server_version".into(), "16.0 (Nilestream 0.1)".into()),
        Backend::ParameterStatus("server_encoding".into(), "UTF8".into()),
        Backend::ParameterStatus("client_encoding".into(), "UTF8".into()),
        // Money is exact minor units at a declared scale, and `DateStyle` and
        // `integer_datetimes` are what stop a client from reinterpreting them.
        Backend::ParameterStatus("DateStyle".into(), "ISO, MDY".into()),
        Backend::ParameterStatus("integer_datetimes".into(), "on".into()),
        Backend::BackendKeyData { pid, secret },
        Backend::ReadyForQuery(b'I'),
    ]
}

/// The refusal for a protocol feature that is specified but not built. A named refusal is
/// the only honest answer: silently ignoring a `Bind` would return results for a query the
/// client did not ask for.
pub fn unsupported(feature: &str, why: &str) -> Backend {
    Backend::ErrorResponse {
        severity: "ERROR".into(),
        code: "0A000".into(), // feature_not_supported
        message: format!("{feature} is not supported by this server"),
        detail: Some(why.into()),
    }
}

/// Render a compiler diagnostic as a wire error, preserving its code.
///
/// The `NLnnnn` code travels to the client rather than being flattened into PostgreSQL's
/// generic syntax-error code, because the interesting errors here — conservation, currency,
/// rung — have no PostgreSQL equivalent, and mapping them onto `42601 syntax_error` would
/// tell the user the one thing that is certainly false.
pub fn diagnostic_error(code: &str, message: &str, detail: Option<&str>) -> Backend {
    // **`42P01` is `undefined_table`, and it was the answer to everything.** A client's
    // retry logic reads the SQLSTATE, so answering one code to a syntax error, a type
    // error and a rung violation alike told every client the same untrue thing. The Niles
    // code still travels in the message, because it is what a person searches for; the
    // SQLSTATE is what a driver acts on, so it is now derived from the code's family.
    Backend::ErrorResponse {
        severity: "ERROR".into(),
        code: sqlstate_for(code).into(),
        message: format!("[{code}] {message}"),
        detail: detail.map(|d| d.to_string()),
    }
}

/// An `ErrorResponse` carrying a SQLSTATE chosen by the caller.
pub fn sqlstate_error(sqlstate: &str, message: &str, detail: Option<&str>) -> Backend {
    Backend::ErrorResponse {
        severity: "ERROR".into(),
        code: sqlstate.into(),
        message: message.to_string(),
        detail: detail.map(|d| d.to_string()),
    }
}

/// The SQLSTATE a Niles diagnostic code maps to.
///
/// The families are the compiler's own, so this is a translation rather than a judgement:
/// `NL00xx` is the parser, `NL01xx`/`NL02xx` are declarations, `NL03xx` is the body-level
/// calculus, `NL04xx` is guarded recursion, `NL05xx` is lowering, and `IR0xx` is the
/// verifier. Each maps to the closest standard class, and anything unrecognised is
/// `XX000` — *internal error* — which is the honest code for "this server produced a
/// diagnostic nobody has classified".
pub fn sqlstate_for(code: &str) -> &'static str {
    if code.starts_with("IR") {
        return "58000"; // system_error: the circuit did not verify
    }
    let Some(n) = code.strip_prefix("NL").and_then(|d| d.parse::<u32>().ok()) else {
        return "XX000";
    };
    match n {
        // The parser.
        0..=99 => "42601", // syntax_error
        // Declarations: a relation, a view or a contract that does not hold together.
        100..=239 => "42P01", // undefined_table / invalid declaration
        // Money and typing: a scale, a currency, an operator, a call argument.
        240..=259 => "42804", // datatype_mismatch
        // Confidentiality.
        260..=299 => "42501", // insufficient_privilege
        // Conservation and the effect calculus. A capability failure is a privilege
        // failure to a client, and a conservation failure is an integrity one — a driver
        // treats those differently, and before this they were the same code.
        312 | 330 | 331 => "42501", // insufficient_privilege: no capability
        300..=399 => "23000",       // integrity_constraint_violation
        // Guarded recursion.
        400..=499 => "42P20", // invalid_recursion
        // Lowering: the form exists in the surface and not in the circuit.
        500..=599 => "0A000", // feature_not_supported
        _ => "XX000",
    }
}

/// Write a sequence of backend messages.
/// Write a batch of backend messages as **one** buffer.
///
/// The encoding is per message; the *write* is not, and the difference is worth a paragraph
/// because it cost a factor of five hundred.
///
/// This function used to call `write_all` once per message. A query reply is four small
/// messages — `RowDescription`, `DataRow`, `CommandComplete`, `ReadyForQuery` — so it left the
/// socket four times, each write small enough for Nagle's algorithm to hold pending the
/// previous segment's acknowledgement, and the peer's delayed-ACK timer to wait 40ms before
/// sending it. The wall-clock harness measured 23 point lookups per second against
/// PostgreSQL's 12,000, and 43ms per query is the delayed-ACK timer wearing a database's
/// clothes.
///
/// Nothing in a counted-work benchmark could have found this: the engine did the right amount
/// of work, in the right order, and then waited. It is the clearest argument in the repository
/// for measuring wall-clock against a baseline rather than counting operations.
///
/// One buffer, one write, one flush. Callers should also set `TCP_NODELAY`, which the daemon
/// now does — belt and braces, because a single reply that outgrows one segment would
/// otherwise reintroduce the same stall at a larger size.
pub fn write_all(w: &mut impl Write, msgs: &[Backend]) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(64 * msgs.len().max(1));
    for m in msgs {
        buf.extend_from_slice(&encode(m));
    }
    w.write_all(&buf)?;
    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn startup_bytes(params: &[(&str, &str)]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&196608u32.to_be_bytes()); // protocol 3.0
        for (k, v) in params {
            put_cstr(&mut body, k);
            put_cstr(&mut body, v);
        }
        body.push(0);
        let mut out = Vec::new();
        out.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
        out.extend_from_slice(&body);
        out
    }

    #[test]
    fn a_real_startup_message_parses() {
        let bytes = startup_bytes(&[("user", "ada"), ("database", "bank")]);
        let m = read_startup(&mut Cursor::new(bytes)).unwrap();
        let Frontend::Startup { version, params } = m else {
            panic!("{m:?}")
        };
        assert_eq!(version, 196608, "protocol 3.0");
        assert_eq!(
            params,
            vec![
                ("user".into(), "ada".into()),
                ("database".into(), "bank".into())
            ]
        );
    }

    #[test]
    fn an_ssl_request_is_recognised_rather_than_misparsed() {
        let mut out = Vec::new();
        out.extend_from_slice(&8i32.to_be_bytes());
        out.extend_from_slice(&80877103u32.to_be_bytes());
        assert_eq!(
            read_startup(&mut Cursor::new(out)).unwrap(),
            Frontend::SslRequest
        );
    }

    #[test]
    fn a_simple_query_round_trips() {
        let mut bytes = vec![b'Q'];
        let mut body = Vec::new();
        put_cstr(
            &mut body,
            "select acct, sum(amt) from postings group by acct",
        );
        bytes.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
        bytes.extend_from_slice(&body);
        let m = read_message(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(
            m,
            Frontend::Query("select acct, sum(amt) from postings group by acct".into())
        );
    }

    #[test]
    fn every_backend_message_is_length_prefixed_correctly() {
        // The framing property. A message whose declared length disagrees with its body
        // desynchronises the connection permanently, and the failure appears several
        // messages later as nonsense — so it gets its own test.
        let msgs = vec![
            Backend::AuthenticationOk,
            Backend::ParameterStatus("server_encoding".into(), "UTF8".into()),
            Backend::BackendKeyData {
                pid: 42,
                secret: 99,
            },
            Backend::ReadyForQuery(b'I'),
            Backend::RowDescription(vec![Field::int8("acct"), Field::numeric("balance")]),
            Backend::DataRow(vec![Some("1001".into()), Some("850.00".into())]),
            Backend::DataRow(vec![Some("1002".into()), None]),
            Backend::CommandComplete("SELECT 2".into()),
            Backend::EmptyQueryResponse,
            unsupported(
                "the extended query protocol",
                "prepared statements need an epoch-keyed plan cache",
            ),
            Backend::NoticeResponse {
                message: "hello".into(),
            },
        ];
        for m in &msgs {
            let bytes = encode(m);
            assert!(bytes.len() >= 5, "{m:?}");
            let declared = i32::from_be_bytes(bytes[1..5].try_into().unwrap()) as usize;
            assert_eq!(
                declared,
                bytes.len() - 1,
                "declared length {declared} disagrees with the frame for {m:?}"
            );
        }
    }

    #[test]
    fn money_is_sent_as_numeric_and_never_as_a_float() {
        // The one type choice with a thesis behind it. Sending money as float8 (OID 701)
        // would let the client round exactly the value the type system worked to keep exact.
        let f = Field::numeric("balance");
        assert_eq!(f.type_oid, 1700, "numeric");
        assert_ne!(f.type_oid, 701, "float8 must never carry money");
        assert_eq!(f.type_size, -1, "arbitrary precision, so variable length");
    }

    #[test]
    fn null_is_distinguishable_from_an_empty_string() {
        // Three absences — SQL null, the empty string, and an evicted hole — and a wire
        // format that conflated the first two would make "no balance" and "zero-length
        // balance" the same answer.
        let with_null = encode(&Backend::DataRow(vec![None]));
        let with_empty = encode(&Backend::DataRow(vec![Some(String::new())]));
        assert_ne!(with_null, with_empty);
        assert_eq!(
            &with_null[7..11],
            &(-1i32).to_be_bytes(),
            "null is length -1"
        );
        assert_eq!(&with_empty[7..11], &0i32.to_be_bytes(), "empty is length 0");
    }

    #[test]
    fn an_extended_message_carries_its_body_to_the_session() {
        // **Inverted.** This used to assert that `P`, `B` and `E` decoded to a bare tag —
        // which they did, and that was the defect: the body was discarded, so the statement
        // text never reached the session and `extended.rs` could not have served a `Parse`
        // even if it had been called.
        let mut bytes = vec![b'P'];
        let mut body = Vec::new();
        put_cstr(&mut body, "stmt1");
        put_cstr(&mut body, "select acct from postings");
        body.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
        bytes.extend_from_slice(&body);
        let msg = read_message(&mut Cursor::new(bytes)).unwrap();
        let Frontend::Extended(tag, got) = msg else {
            panic!("{msg:?}")
        };
        assert_eq!(tag, b'P');
        let mut at = 0;
        assert_eq!(get_cstr(&got, &mut at), "stmt1");
        assert_eq!(get_cstr(&got, &mut at), "select acct from postings");
    }

    #[test]
    fn the_four_extended_acknowledgements_encode_to_their_protocol_tags() {
        // All four were missing from `Backend`, so the module implementing the extended
        // protocol had nothing to reply with.
        for (msg, tag) in [
            (Backend::ParseComplete, b'1'),
            (Backend::BindComplete, b'2'),
            (Backend::CloseComplete, b'3'),
            (Backend::NoData, b'n'),
        ] {
            let bytes = encode(&msg);
            assert_eq!(bytes[0], tag, "{msg:?}");
            assert_eq!(
                i32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
                4,
                "an empty body is a length of four: {msg:?}"
            );
        }
    }

    #[test]
    fn a_compiler_diagnostic_keeps_its_code_on_the_wire() {
        // A conservation error mapped onto `42601 syntax_error` would tell the user the one
        // thing that is certainly false about their program.
        let e = diagnostic_error(
            "NL0300",
            "this transaction does not conserve `usd`",
            Some("net movement is -40.00"),
        );
        let Backend::ErrorResponse { message, .. } = &e else {
            panic!()
        };
        assert!(message.contains("NL0300"), "{message}");
        assert!(message.contains("conserve"), "{message}");
    }

    #[test]
    fn the_startup_reply_ends_ready_for_query() {
        // A client blocks until it sees `Z`. A reply that omits it hangs the connection
        // with no error anywhere, which is the least debuggable failure in the protocol.
        let r = startup_reply(1, 2);
        assert_eq!(r.first(), Some(&Backend::AuthenticationOk));
        assert!(matches!(r.last(), Some(Backend::ReadyForQuery(b'I'))));
        assert!(r
            .iter()
            .any(|m| matches!(m, Backend::ParameterStatus(k, _) if k == "server_version")));
    }

    #[test]
    fn implausible_lengths_are_rejected_rather_than_allocated() {
        // A hostile or broken client sending a four-gigabyte length must not cause the
        // server to try to allocate it.
        let mut bytes = vec![b'Q'];
        bytes.extend_from_slice(&i32::MAX.to_be_bytes());
        assert!(read_message(&mut Cursor::new(bytes)).is_err());
        let mut s = Vec::new();
        s.extend_from_slice(&3i32.to_be_bytes());
        assert!(read_startup(&mut Cursor::new(s)).is_err());
    }

    /// **The framed row is byte-identical to the message it replaced.**
    ///
    /// T-05 stopped building a `Backend::DataRow(Vec<Option<String>>)` per served row and
    /// started writing the frame straight into the reply buffer. That is only a performance
    /// change if the bytes are the same, and "the same" has to mean every case that differs
    /// on the wire: a null is a -1 length and not an empty string, a negative number carries
    /// its sign, zero is one digit, and `i128::MIN` is the value a naive formatter overflows
    /// on while negating.
    #[test]
    fn a_framed_row_is_the_bytes_the_message_encoded() {
        let cases: Vec<Vec<Option<i128>>> = vec![
            vec![],
            vec![None],
            vec![Some(0)],
            vec![Some(-1)],
            vec![Some(1), None, Some(-85_500)],
            vec![Some(i128::MAX), Some(i128::MIN)],
            vec![Some(9_999), Some(-9_999), None, Some(0)],
        ];
        for cells in cases {
            let mut framed = Vec::new();
            put_data_row(&mut framed, cells.iter().copied());
            let as_message = encode(&Backend::DataRow(
                cells.iter().map(|c| c.map(|v| v.to_string())).collect(),
            ));
            assert_eq!(
                framed, as_message,
                "the framed form and the message form disagree for {cells:?}"
            );
            // And it decodes back to what went in, so a test asserting on rows is asserting
            // on the same thing whichever representation carried them.
            let back = decoded_rows(&[Backend::Raw(framed)]);
            assert_eq!(
                back,
                vec![cells
                    .iter()
                    .map(|c| c.map(|v| v.to_string()))
                    .collect::<Vec<_>>()]
            );
        }
    }

    /// A `Raw` run carrying several rows decodes to all of them, in order.
    #[test]
    fn a_reply_buffer_carries_every_row_it_was_given() {
        let mut buf = Vec::new();
        for i in 0..5i128 {
            put_data_row(&mut buf, [Some(i), None, Some(-i)]);
        }
        let rows = decoded_rows(&[Backend::Raw(buf)]);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[3], vec![Some("3".into()), None, Some("-3".into())]);
    }
}

/// **PostgreSQL's binary wire encodings, for the types this server sends.**
///
/// Binary is not smaller than text for a small integer — `1` is one byte of text and eight of
/// `int8` — so bandwidth is not the reason it is here. Parsing is. A text reply makes the
/// server format every integer into digits and the client parse every one back, and that
/// round trip is the bulk of what a large answer costs at both ends.
///
/// A format is negotiated **per column**, as the protocol specifies: `Bind` carries a result
/// format code for each, and `RowDescription` reports what was chosen. `psql` and every other
/// text client are unaffected, because text stays the default and is what an unrequested
/// column gets.
pub mod binary {
    /// `int8`: eight bytes, big-endian, two's complement.
    ///
    /// The value is an `i128` here because that is what the engine carries. One that does not
    /// fit is **refused by the caller**, never truncated — see `session`'s 22003 path. A
    /// balance that silently lost its high bits would be the §1.1.1 defect at the last hop.
    pub fn int8(v: i64, out: &mut Vec<u8>) {
        out.extend_from_slice(&v.to_be_bytes());
    }

    /// `numeric`: PostgreSQL's own decimal encoding, from exact minor units and a scale.
    ///
    /// The format, from `src/backend/utils/adt/numeric.c`: `ndigits`, `weight`, `sign`,
    /// `dscale`, each an `i16`, then `ndigits` base-10000 groups as `i16`. `weight` is the
    /// base-10000 exponent of the **first** group, so a value's integer part occupies groups
    /// `weight` down to 0 and its fraction the negative ones. `dscale` is the number of
    /// decimal digits to display after the point, which is the column's declared scale and
    /// not a property of the value — that is what keeps `1.50` from being sent as `1.5`.
    ///
    /// Written out rather than reached for from a crate, because this repository's kernel and
    /// engine have no dependencies, and because the encoding is the *money* boundary: a
    /// rounding here is a rounding of a balance. It is checked against the real PostgreSQL
    /// in `crates/bank-bench/tests/numeric_binary_oracle.rs`, which asks a running server to
    /// send the same values in binary and compares the bytes — the only test of an encoding
    /// that is worth anything is one against the implementation it has to match.
    pub fn numeric(minor: i128, scale: u32, out: &mut Vec<u8>) {
        const NBASE: i128 = 10_000;
        // Sign is carried separately; PostgreSQL's zero is positive with no digits.
        let neg = minor < 0;
        // Accumulate on the negative side so `i128::MIN` does not overflow on negation.
        let mut n = if neg { minor } else { -minor };

        // Split at the decimal point. `scale` digits are fractional; the rest are integral.
        // The fraction is padded on the right to a whole number of base-10000 groups, which
        // is what makes the group boundaries line up with the point.
        let pad = (4 - (scale % 4)) % 4;
        for _ in 0..pad {
            n *= 10;
        }
        let frac_groups = (scale as usize + pad as usize) / 4;

        // Digits, least significant group first.
        let mut groups: Vec<i16> = Vec::new();
        if n == 0 {
            // Zero: no digit groups at all, weight 0, and the declared scale still shown.
            out.extend_from_slice(&0i16.to_be_bytes());
            out.extend_from_slice(&0i16.to_be_bytes());
            out.extend_from_slice(&0u16.to_be_bytes());
            out.extend_from_slice(&(scale as i16).to_be_bytes());
            return;
        }
        while n != 0 {
            groups.push((-(n % NBASE)) as i16);
            n /= NBASE;
        }
        // `weight` counts base-10000 places of the integer part, less one. The groups below
        // the point are the fractional ones.
        let mut weight = groups.len() as i32 - frac_groups as i32 - 1;

        // Most significant first, which is the order the wire wants.
        groups.reverse();
        // Leading zero groups carry no information and PostgreSQL does not send them; each
        // one dropped lowers the weight by a place.
        while groups.first() == Some(&0) {
            groups.remove(0);
            weight -= 1;
        }
        // Trailing zero groups are equally redundant: `dscale` says how much to display, so
        // the digits need not be padded out to it.
        while groups.last() == Some(&0) {
            groups.pop();
        }

        out.extend_from_slice(&(groups.len() as i16).to_be_bytes());
        out.extend_from_slice(&(weight as i16).to_be_bytes());
        out.extend_from_slice(&(if neg { 0x4000u16 } else { 0x0000u16 }).to_be_bytes());
        out.extend_from_slice(&(scale as i16).to_be_bytes());
        for g in groups {
            out.extend_from_slice(&g.to_be_bytes());
        }
    }
}
