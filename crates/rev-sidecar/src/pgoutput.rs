//! A decoder for PostgreSQL's logical replication stream: the `XLogData` / keepalive framing
//! of the streaming replication protocol (PostgreSQL 16 documentation §55.4, "Streaming
//! Replication Protocol"), and the `pgoutput` messages inside it (§55.9, "Logical Replication
//! Message Formats"), protocol version 1.
//!
//! Only what an append-only ledger produces is decoded into values: `Begin`, `Relation`,
//! `Insert` and `Commit`. `Update`, `Delete` and `Truncate` are recognised and reported as
//! such, because on H3's base they are impossible (the append-only trigger refuses them) and
//! a sidecar that silently skipped one would be serving a view of a history that no longer
//! exists. `Type` and `Origin` carry nothing a sum over integers needs and are skipped by
//! name.

/// One message of the replication stream's outer framing.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    /// `w`: WAL data starting at `start`, with the server's WAL end, and one pgoutput message.
    XLogData {
        start: u64,
        wal_end: u64,
        message: Vec<u8>,
    },
    /// `k`: the server's WAL end, and whether it wants a status update now.
    Keepalive { wal_end: u64, reply_requested: bool },
}

/// One pgoutput message.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Begin {
        final_lsn: u64,
        xid: u32,
    },
    Commit {
        commit_lsn: u64,
        end_lsn: u64,
    },
    Relation {
        oid: u32,
        namespace: String,
        name: String,
        columns: Vec<String>,
    },
    /// A new row, in text format: one entry per column, `None` for NULL.
    Insert {
        relation: u32,
        values: Vec<Option<String>>,
    },
    /// A change an append-only base cannot have: `U`, `D` or `T`, by its tag.
    Forbidden(char),
    /// `Y` (type) or `O` (origin): nothing a sum over integers needs.
    Skipped(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodeError(pub String);

struct Cursor<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn new(b: &'a [u8]) -> Self {
        Cursor { b, at: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.at + n > self.b.len() {
            return Err(DecodeError(format!(
                "message truncated: wanted {n} bytes at {} of {}",
                self.at,
                self.b.len()
            )));
        }
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }
    fn i16(&mut self) -> Result<i16, DecodeError> {
        Ok(i16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, DecodeError> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn cstr(&mut self) -> Result<String, DecodeError> {
        let rest = &self.b[self.at..];
        let end = rest
            .iter()
            .position(|&c| c == 0)
            .ok_or_else(|| DecodeError("unterminated string".into()))?;
        let s = String::from_utf8_lossy(&rest[..end]).into_owned();
        self.at += end + 1;
        Ok(s)
    }
    fn rest(&self) -> &'a [u8] {
        &self.b[self.at..]
    }
}

/// Decode one `CopyData` payload of a replication stream.
pub fn frame(data: &[u8]) -> Result<Frame, DecodeError> {
    let mut c = Cursor::new(data);
    match c.u8()? {
        b'w' => {
            let start = c.u64()?;
            let wal_end = c.u64()?;
            let _send_time = c.u64()?;
            Ok(Frame::XLogData {
                start,
                wal_end,
                message: c.rest().to_vec(),
            })
        }
        b'k' => {
            let wal_end = c.u64()?;
            let _send_time = c.u64()?;
            let reply_requested = c.u8()? != 0;
            Ok(Frame::Keepalive {
                wal_end,
                reply_requested,
            })
        }
        other => Err(DecodeError(format!(
            "unknown replication frame `{}`",
            other as char
        ))),
    }
}

/// Decode one pgoutput message (protocol version 1).
pub fn message(data: &[u8]) -> Result<Message, DecodeError> {
    let mut c = Cursor::new(data);
    let tag = c.u8()?;
    Ok(match tag {
        b'B' => {
            let final_lsn = c.u64()?;
            let _ts = c.u64()?;
            let xid = c.u32()?;
            Message::Begin { final_lsn, xid }
        }
        b'C' => {
            let _flags = c.u8()?;
            let commit_lsn = c.u64()?;
            let end_lsn = c.u64()?;
            let _ts = c.u64()?;
            Message::Commit {
                commit_lsn,
                end_lsn,
            }
        }
        b'R' => {
            let oid = c.u32()?;
            let namespace = c.cstr()?;
            let name = c.cstr()?;
            let _replica_identity = c.u8()?;
            let n = c.i16()?;
            let mut columns = Vec::with_capacity(n.max(0) as usize);
            for _ in 0..n {
                let _flags = c.u8()?;
                columns.push(c.cstr()?);
                let _type_oid = c.u32()?;
                let _typmod = c.i32()?;
            }
            Message::Relation {
                oid,
                namespace,
                name,
                columns,
            }
        }
        b'I' => {
            let relation = c.u32()?;
            let kind = c.u8()?;
            if kind != b'N' {
                return Err(DecodeError(format!(
                    "insert without a new tuple (`{}`)",
                    kind as char
                )));
            }
            Message::Insert {
                relation,
                values: tuple(&mut c)?,
            }
        }
        b'U' | b'D' | b'T' => Message::Forbidden(tag as char),
        b'Y' | b'O' => Message::Skipped(tag as char),
        other => {
            return Err(DecodeError(format!(
                "unknown pgoutput message `{}`",
                other as char
            )))
        }
    })
}

fn tuple(c: &mut Cursor) -> Result<Vec<Option<String>>, DecodeError> {
    let n = c.i16()?;
    let mut out = Vec::with_capacity(n.max(0) as usize);
    for _ in 0..n {
        match c.u8()? {
            b'n' => out.push(None),
            b't' => {
                let len = c.i32()?;
                if len < 0 {
                    return Err(DecodeError("negative column length".into()));
                }
                out.push(Some(
                    String::from_utf8_lossy(c.take(len as usize)?).into_owned(),
                ));
            }
            // `u`: an unchanged TOASTed value, which an insert cannot carry.
            other => {
                return Err(DecodeError(format!(
                    "column kind `{}` in an insert",
                    other as char
                )))
            }
        }
    }
    Ok(out)
}

/// A standby status update (`r`): the positions this consumer has written, flushed and
/// applied, so the server may release WAL behind them.
pub fn status_update(applied: u64, now_micros: u64, reply: bool) -> Vec<u8> {
    let mut b = vec![b'r'];
    for _ in 0..3 {
        b.extend_from_slice(&applied.to_be_bytes());
    }
    b.extend_from_slice(&now_micros.to_be_bytes());
    b.push(reply as u8);
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cstr(b: &mut Vec<u8>, s: &str) {
        b.extend_from_slice(s.as_bytes());
        b.push(0);
    }

    #[test]
    fn a_transaction_decodes_in_order() {
        // Relation arm.postings (id, epoch, acct, amt).
        let mut r = vec![b'R'];
        r.extend_from_slice(&16_384u32.to_be_bytes());
        cstr(&mut r, "arm");
        cstr(&mut r, "postings");
        r.push(b'd');
        r.extend_from_slice(&4i16.to_be_bytes());
        for name in ["id", "epoch", "acct", "amt"] {
            r.push(0);
            cstr(&mut r, name);
            r.extend_from_slice(&20u32.to_be_bytes());
            r.extend_from_slice(&(-1i32).to_be_bytes());
        }
        assert_eq!(
            message(&r).unwrap(),
            Message::Relation {
                oid: 16_384,
                namespace: "arm".into(),
                name: "postings".into(),
                columns: vec!["id".into(), "epoch".into(), "acct".into(), "amt".into()],
            }
        );
        // Insert (7, 51, 3, -250) with one NULL-free tuple.
        let mut i = vec![b'I'];
        i.extend_from_slice(&16_384u32.to_be_bytes());
        i.push(b'N');
        i.extend_from_slice(&4i16.to_be_bytes());
        for v in ["7", "51", "3", "-250"] {
            i.push(b't');
            i.extend_from_slice(&(v.len() as i32).to_be_bytes());
            i.extend_from_slice(v.as_bytes());
        }
        assert_eq!(
            message(&i).unwrap(),
            Message::Insert {
                relation: 16_384,
                values: ["7", "51", "3", "-250"]
                    .map(|s| Some(s.to_string()))
                    .to_vec(),
            }
        );
        // Commit at (0/443C0F8, 0/443C128).
        let mut cm = vec![b'C', 0];
        cm.extend_from_slice(&0x443C0F8u64.to_be_bytes());
        cm.extend_from_slice(&0x443C128u64.to_be_bytes());
        cm.extend_from_slice(&0u64.to_be_bytes());
        assert_eq!(
            message(&cm).unwrap(),
            Message::Commit {
                commit_lsn: 0x443C0F8,
                end_lsn: 0x443C128
            }
        );
    }

    #[test]
    fn a_change_an_append_only_base_cannot_have_is_named_not_skipped() {
        assert_eq!(message(b"U\0\0\0\0").unwrap(), Message::Forbidden('U'));
        assert_eq!(message(b"D\0\0\0\0").unwrap(), Message::Forbidden('D'));
        assert_eq!(message(b"T\0\0\0\0").unwrap(), Message::Forbidden('T'));
    }

    #[test]
    fn truncated_messages_are_errors_not_panics() {
        assert!(message(b"C\0\0").is_err());
        assert!(message(b"I\0\0\0\x01N\0\x01t\0\0\0\x09ab").is_err());
        assert!(frame(b"w\0\0").is_err());
        assert!(frame(b"z").is_err());
    }

    #[test]
    fn keepalives_and_status_updates_frame_as_the_protocol_says() {
        let mut k = vec![b'k'];
        k.extend_from_slice(&42u64.to_be_bytes());
        k.extend_from_slice(&0u64.to_be_bytes());
        k.push(1);
        assert_eq!(
            frame(&k).unwrap(),
            Frame::Keepalive {
                wal_end: 42,
                reply_requested: true
            }
        );
        let s = status_update(0x10, 5, false);
        assert_eq!(s.len(), 1 + 8 * 4 + 1);
        assert_eq!(s[0], b'r');
        assert_eq!(&s[1..9], &0x10u64.to_be_bytes());
    }
}
