//! The MySQL client/server protocol.
//!
//! Two wire protocols rather than one, for the reason §6.9 gives: a database nobody can
//! connect to with the tools they already have is a database nobody adopts, and the tools a
//! bank already has are split roughly evenly between the two dialect families. Supporting
//! only PostgreSQL would make the adoption argument true for half the audience.
//!
//! # What is genuinely different from PostgreSQL's protocol
//!
//! Not much, and the differences are mostly clerical — which is itself the point, because it
//! means the *semantics* live in one place and only the framing differs.
//!
//! * **Framing.** MySQL packets are a 3-byte little-endian length plus a 1-byte sequence
//!   number, and the sequence number must increment per packet within a command or the
//!   client desynchronises. PostgreSQL has a type byte and a big-endian length and no
//!   sequence. The sequence number is the single most common source of a hung MySQL
//!   connection, so it is a field of the codec rather than a parameter.
//! * **Length-encoded integers.** MySQL's `lenenc` encoding is used for column counts and
//!   string lengths: one byte below 251, then a marker byte and 2, 3 or 8 bytes.
//! * **Everything is a string on the wire** in the text protocol, including integers. This
//!   turns out to *help* the one thing this thesis cares about: there is no float
//!   representation to fall into, so a money column crosses as decimal text and stays exact.
//! * **The handshake runs the other way.** MySQL's server speaks first, with a greeting
//!   carrying the auth plugin and a nonce; PostgreSQL's client speaks first.
//!
//! # The design commitment is unchanged
//!
//! A wire protocol is a **surface, not a semantics**. A MySQL client's SQL is parsed as
//! Niles's SQL surface, lowered to the same IR, verified by the same verifier and served
//! from the same REV runtime as a PostgreSQL client's — and as a pipeline-surface query.
//! There is no MySQL execution path. Two ways to compute an answer is two answers that can
//! disagree, and a compatibility layer is exactly the seam this thesis argues against.
//!
//! # What is not built
//!
//! `mysql_native_password` is answered with an "auth switch not required" path and no
//! credential is checked, matching the PostgreSQL side's trust-only posture; the prepared
//! statement protocol (`COM_STMT_PREPARE` and friends) is refused by name; and compression
//! and the binary result-set format are absent. Each refusal names its reason, because a
//! server that silently ignores a command returns results for something the client did not
//! ask.

/// Capability flags, only the ones this server asserts or checks.
pub mod caps {
    pub const LONG_PASSWORD: u32 = 0x0000_0001;
    pub const CLIENT_PROTOCOL_41: u32 = 0x0000_0200;
    pub const SECURE_CONNECTION: u32 = 0x0000_8000;
    pub const PLUGIN_AUTH: u32 = 0x0008_0000;
    pub const DEPRECATE_EOF: u32 = 0x0100_0000;
    /// Asserted by a client that wants TLS. See `crate::tls`.
    pub const CLIENT_SSL: u32 = 0x0000_0800;
}

/// MySQL column types, restricted to the three this server emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColType {
    /// `MYSQL_TYPE_LONGLONG` — epochs, counts, identifiers.
    LongLong = 0x08,
    /// `MYSQL_TYPE_NEWDECIMAL` — **money**. Never `MYSQL_TYPE_DOUBLE` (0x05), for the same
    /// reason PostgreSQL money is `numeric` and never `float8`: exactness that survived the
    /// type system has to survive the last hop.
    NewDecimal = 0xf6,
    /// `MYSQL_TYPE_VAR_STRING`.
    VarString = 0xfd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: ColType,
}

impl Column {
    pub fn int(name: &str) -> Column {
        Column {
            name: name.into(),
            ty: ColType::LongLong,
        }
    }
    pub fn money(name: &str) -> Column {
        Column {
            name: name.into(),
            ty: ColType::NewDecimal,
        }
    }
    pub fn text(name: &str) -> Column {
        Column {
            name: name.into(),
            ty: ColType::VarString,
        }
    }
}

/// Commands this server recognises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `COM_QUERY` (0x03).
    Query(String),
    /// `COM_QUIT` (0x01).
    Quit,
    /// `COM_PING` (0x0e).
    Ping,
    /// `COM_INIT_DB` (0x02).
    InitDb(String),
    /// `COM_STMT_*` — the prepared statement protocol, refused by name.
    Stmt(u8),
    Unknown(u8),
}

/// Length-encoded integer, MySQL's variable-width unsigned encoding.
pub fn put_lenenc(out: &mut Vec<u8>, v: u64) {
    match v {
        0..=250 => out.push(v as u8),
        251..=0xffff => {
            out.push(0xfc);
            out.extend_from_slice(&(v as u16).to_le_bytes());
        }
        0x1_0000..=0xff_ffff => {
            out.push(0xfd);
            out.extend_from_slice(&(v as u32).to_le_bytes()[..3]);
        }
        _ => {
            out.push(0xfe);
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
}

pub fn get_lenenc(b: &[u8], at: &mut usize) -> Option<u64> {
    let first = *b.get(*at)?;
    *at += 1;
    Some(match first {
        0xfc => {
            let v = u16::from_le_bytes(b.get(*at..*at + 2)?.try_into().ok()?) as u64;
            *at += 2;
            v
        }
        0xfd => {
            let s = b.get(*at..*at + 3)?;
            *at += 3;
            u32::from_le_bytes([s[0], s[1], s[2], 0]) as u64
        }
        0xfe => {
            let v = u64::from_le_bytes(b.get(*at..*at + 8)?.try_into().ok()?);
            *at += 8;
            v
        }
        // 0xfb is NULL in a result row, and 0xff is an error packet marker; neither is a
        // length, and returning a length for them is how a decoder walks off the end.
        0xfb | 0xff => return None,
        n => n as u64,
    })
}

fn put_lenenc_str(out: &mut Vec<u8>, s: &str) {
    put_lenenc(out, s.len() as u64);
    out.extend_from_slice(s.as_bytes());
}

/// Frame a payload as a MySQL packet.
///
/// The sequence number is the argument most often got wrong and the one whose error is
/// least debuggable: a client that receives an out-of-order sequence stops reading and the
/// connection hangs with nothing in any log. It is therefore threaded explicitly rather than
/// held in a mutable field somewhere.
pub fn packet(seq: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 4);
    let len = payload.len() as u32;
    out.extend_from_slice(&len.to_le_bytes()[..3]);
    out.push(seq);
    out.extend_from_slice(payload);
    out
}

/// The server's opening greeting (protocol 10).
pub fn handshake(connection_id: u32, nonce: &[u8; 20]) -> Vec<u8> {
    let mut p = Vec::new();
    p.push(10); // protocol version
    p.extend_from_slice(b"8.0.0-Nilestream-0.1\0");
    p.extend_from_slice(&connection_id.to_le_bytes());
    p.extend_from_slice(&nonce[..8]);
    p.push(0); // filler
    let flags = caps::LONG_PASSWORD
        | caps::CLIENT_PROTOCOL_41
        | caps::SECURE_CONNECTION
        | caps::PLUGIN_AUTH
        | caps::DEPRECATE_EOF;
    p.extend_from_slice(&(flags as u16).to_le_bytes()); // lower capability flags
    p.push(0x21); // charset: utf8_general_ci
    p.extend_from_slice(&0u16.to_le_bytes()); // status
    p.extend_from_slice(&((flags >> 16) as u16).to_le_bytes()); // upper capability flags
    p.push(21); // auth plugin data length
    p.extend_from_slice(&[0u8; 10]); // reserved
    p.extend_from_slice(&nonce[8..]);
    p.push(0);
    p.extend_from_slice(b"mysql_native_password\0");
    p
}

/// `OK_Packet`.
pub fn ok(affected: u64, last_insert_id: u64, info: &str) -> Vec<u8> {
    let mut p = vec![0x00];
    put_lenenc(&mut p, affected);
    put_lenenc(&mut p, last_insert_id);
    p.extend_from_slice(&0x0002u16.to_le_bytes()); // SERVER_STATUS_AUTOCOMMIT
    p.extend_from_slice(&0u16.to_le_bytes()); // warnings
    p.extend_from_slice(info.as_bytes());
    p
}

/// `ERR_Packet`.
///
/// The SQLSTATE is carried through rather than flattened, and the Niles diagnostic code
/// stays in the message for the same reason it does on the PostgreSQL side: mapping a
/// conservation error onto a generic syntax error would tell the user the one thing that is
/// certainly false about their program.
pub fn err(code: u16, sqlstate: &str, message: &str) -> Vec<u8> {
    let mut p = vec![0xff];
    p.extend_from_slice(&code.to_le_bytes());
    p.push(b'#');
    p.extend_from_slice(sqlstate.as_bytes());
    p.extend_from_slice(message.as_bytes());
    p
}

/// A column definition packet (protocol 41).
pub fn column_def(c: &Column) -> Vec<u8> {
    let mut p = Vec::new();
    put_lenenc_str(&mut p, "def"); // catalog
    put_lenenc_str(&mut p, ""); // schema
    put_lenenc_str(&mut p, ""); // table
    put_lenenc_str(&mut p, ""); // org_table
    put_lenenc_str(&mut p, &c.name);
    put_lenenc_str(&mut p, &c.name); // org_name
    put_lenenc(&mut p, 0x0c); // length of the fixed-length fields
    p.extend_from_slice(&0x003fu16.to_le_bytes()); // charset: binary
    p.extend_from_slice(&65535u32.to_le_bytes()); // column length
    p.push(c.ty as u8);
    p.extend_from_slice(&0u16.to_le_bytes()); // flags
    p.push(if c.ty == ColType::NewDecimal { 4 } else { 0 }); // decimals
    p.extend_from_slice(&0u16.to_le_bytes()); // filler
    p
}

/// A text-protocol result row.
///
/// `NULL` is `0xfb`, distinct from an empty string, which is a zero-length lenenc string.
/// The absence lattice reaches a MySQL client intact for the same reason it reaches a
/// PostgreSQL one: a view with no entry for a key is not a view whose balance is zero.
pub fn text_row(values: &[Option<String>]) -> Vec<u8> {
    let mut p = Vec::new();
    for v in values {
        match v {
            None => p.push(0xfb),
            Some(s) => put_lenenc_str(&mut p, s),
        }
    }
    p
}

/// Decode a command packet payload.
pub fn parse_command(payload: &[u8]) -> Command {
    let Some((&tag, rest)) = payload.split_first() else {
        return Command::Unknown(0);
    };
    match tag {
        0x01 => Command::Quit,
        0x02 => Command::InitDb(String::from_utf8_lossy(rest).into_owned()),
        0x03 => Command::Query(String::from_utf8_lossy(rest).into_owned()),
        0x0e => Command::Ping,
        0x16..=0x1c => Command::Stmt(tag),
        other => Command::Unknown(other),
    }
}

/// The refusal for the prepared statement protocol.
///
/// The same open design question as on the PostgreSQL side: a plan must be cached against
/// the epoch it was compiled at. That question now *has* an answer (`crate::extended`), so
/// this refusal is a statement about implementation effort rather than about an unsolved
/// problem — and saying which of the two it is matters.
pub fn stmt_unsupported(tag: u8) -> Vec<u8> {
    err(
        1295,
        "HY000",
        &format!(
            "COM_STMT (0x{tag:02x}) is not implemented by this server. The epoch-keyed plan \
             cache it needs exists (see the PostgreSQL extended-query path); only the MySQL \
             framing for it is missing."
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packets_carry_a_three_byte_length_and_a_sequence() {
        let p = packet(7, b"hello");
        assert_eq!(&p[0..3], &[5, 0, 0], "little-endian 3-byte length");
        assert_eq!(p[3], 7, "sequence number");
        assert_eq!(&p[4..], b"hello");
    }

    #[test]
    fn the_sequence_number_is_threaded_rather_than_hidden() {
        // A client receiving an out-of-order sequence stops reading, and the connection
        // hangs with nothing in any log. Making it a parameter is a defence against the
        // least debuggable failure in this protocol.
        let a = packet(0, b"x");
        let b = packet(1, b"x");
        assert_ne!(a[3], b[3]);
    }

    #[test]
    fn length_encoded_integers_round_trip_at_every_width() {
        for v in [
            0u64,
            1,
            250,
            251,
            0xffff,
            0x1_0000,
            0xff_ffff,
            0x100_0000,
            u64::MAX,
        ] {
            let mut buf = Vec::new();
            put_lenenc(&mut buf, v);
            let mut at = 0;
            assert_eq!(get_lenenc(&buf, &mut at), Some(v), "failed at {v}");
            assert_eq!(
                at,
                buf.len(),
                "decoder consumed the wrong number of bytes for {v}"
            );
        }
    }

    #[test]
    fn the_null_and_error_markers_are_not_decoded_as_lengths() {
        // 0xfb is NULL and 0xff is an error marker. Returning a length for either is how a
        // decoder walks off the end of a buffer.
        let mut at = 0;
        assert_eq!(get_lenenc(&[0xfb], &mut at), None);
        let mut at = 0;
        assert_eq!(get_lenenc(&[0xff], &mut at), None);
    }

    #[test]
    fn money_is_new_decimal_and_never_double() {
        // The same commitment as the PostgreSQL side. MYSQL_TYPE_DOUBLE is 0x05 and money
        // must never carry it.
        let c = Column::money("balance");
        assert_eq!(c.ty as u8, 0xf6);
        assert_ne!(c.ty as u8, 0x05);
        let def = column_def(&c);
        assert!(def.contains(&0xf6));
        assert!(
            *def.last().unwrap() == 0 && def[def.len() - 3] == 4,
            "decimals field set for money"
        );
    }

    #[test]
    fn null_is_distinguishable_from_an_empty_string_on_this_wire_too() {
        let with_null = text_row(&[None]);
        let with_empty = text_row(&[Some(String::new())]);
        assert_eq!(with_null, vec![0xfb]);
        assert_eq!(with_empty, vec![0x00]);
        assert_ne!(with_null, with_empty);
    }

    #[test]
    fn the_handshake_advertises_protocol_41_and_a_full_nonce() {
        let nonce = [7u8; 20];
        let h = handshake(42, &nonce);
        assert_eq!(h[0], 10, "protocol version 10");
        assert!(h
            .windows(11)
            .any(|w| w == b"Nilestream\0" || w.starts_with(b"Nilestream")));
        assert!(h.ends_with(b"mysql_native_password\0"));
        // The nonce is split across the packet in two pieces; both must be present.
        assert!(h.windows(8).any(|w| w == &nonce[..8]));
        assert!(h.windows(12).any(|w| w == &nonce[8..]));
    }

    #[test]
    fn commands_decode_to_the_right_variants() {
        assert_eq!(
            parse_command(b"\x03select 1"),
            Command::Query("select 1".into())
        );
        assert_eq!(parse_command(b"\x01"), Command::Quit);
        assert_eq!(parse_command(b"\x0e"), Command::Ping);
        assert_eq!(parse_command(b"\x02bank"), Command::InitDb("bank".into()));
        assert_eq!(parse_command(b"\x16select 1"), Command::Stmt(0x16));
        assert_eq!(parse_command(b""), Command::Unknown(0));
    }

    #[test]
    fn an_error_packet_keeps_the_sqlstate_and_the_niles_code() {
        // Flattening a conservation error into a generic syntax error would tell the user
        // the one thing that is certainly false about their program.
        let e = err(
            1064,
            "42000",
            "[NL0300] this transaction does not conserve `usd`",
        );
        assert_eq!(e[0], 0xff);
        assert_eq!(u16::from_le_bytes([e[1], e[2]]), 1064);
        assert_eq!(e[3], b'#');
        assert_eq!(&e[4..9], b"42000");
        assert!(String::from_utf8_lossy(&e).contains("NL0300"));
    }

    #[test]
    fn the_prepared_statement_refusal_distinguishes_effort_from_an_open_problem() {
        // The question that blocked it has been answered; only the framing is missing, and
        // saying which of the two it is matters to anyone deciding whether to depend on it.
        let p = stmt_unsupported(0x16);
        let s = String::from_utf8_lossy(&p);
        assert!(s.contains("epoch-keyed plan cache it needs exists"));
        assert!(s.contains("only the MySQL framing"));
    }

    #[test]
    fn an_ok_packet_reports_affected_rows() {
        let p = ok(3, 0, "SELECT 3");
        assert_eq!(p[0], 0x00);
        let mut at = 1;
        assert_eq!(get_lenenc(&p, &mut at), Some(3));
    }
}
