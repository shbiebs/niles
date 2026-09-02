//! **The daemon, as a library.** A listener, a connection loop, and the startup exchange.
//!
//! Extracted from `main.rs` so that something other than the binary can host it — specifically
//! the wall-clock harness, which runs the server on a thread of its own process and drives it
//! over TCP from the same client it uses for PostgreSQL.
//!
//! That arrangement is worth being precise about, because it looks like a shortcut and is the
//! opposite of one. The fairness rule the benchmark rests on is that **both targets pay the
//! same client cost**: a comparison where one side is called in-process and the other over TCP
//! omits, from one side only, the syscalls and the round trip the other pays. Hosting the
//! listener on a thread changes *where the server runs*, not *how the client talks to it* —
//! the socket, the protocol, the framing and the round trip are all still there. Calling
//! `RevEngine::read` directly from the harness would have been the shortcut, and it is exactly
//! what this module exists to avoid.

/// The schema the daemon serves when none is supplied.
///
/// Public so the wall-clock harness hosts *this* schema rather than a copy of it. Two
/// spellings of one schema is two things that can drift, and a benchmark measuring a schema
/// the server does not serve would be measuring nothing in particular.
pub const DEFAULT_SCHEMA: &str = "\
schema bank {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_postings on postings (acct) anchor;
}
";

use crate::pg_wire::{self, Frontend};
use crate::rev_engine::RevEngine;
use crate::session::Session;
use crate::tls;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

/// Accept connections until the listener is dropped, serving each on its own thread.
///
/// One thread per connection, which is the right shape at this scale and is stated rather
/// than defended: the write path is serialised through a single sealer anyway, so
/// connection-level concurrency is about *reads*, and reads over an immutable base need no
/// coordination at all.
pub fn accept_loop(listener: TcpListener, schema: String, engine: Arc<Mutex<RevEngine>>) {
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let schema = schema.clone();
        let engine = Arc::clone(&engine);
        std::thread::spawn(move || {
            let _ = serve(stream, schema, engine);
        });
    }
}

pub fn serve(
    stream: TcpStream,
    schema: String,
    engine: Arc<Mutex<RevEngine>>,
) -> std::io::Result<()> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_default();
    // **Disable Nagle.** A request/response protocol with small replies is the exact shape
    // Nagle's algorithm penalises: the reply is held pending an acknowledgement the peer's
    // delayed-ACK timer will not send for 40ms. The wall-clock harness measured 23 point
    // lookups per second before this line existed, against PostgreSQL's 12,000 — a factor of
    // five hundred, none of which was the engine doing anything wrong.
    stream.set_nodelay(true)?;
    let mut w = stream.try_clone()?;
    let mut r = BufReader::new(stream);

    // The startup exchange, routed through the TLS policy of §6.12 rather than through a
    // hardcoded refusal. This build ships no TLS provider — the cryptography is delegated,
    // see `tls.rs` — so the policy here is `Disabled` and the negotiation answers `N`,
    // which is the protocol-correct refusal: it lets the client's own `sslmode` decide,
    // where an `ErrorResponse` would break a `prefer` client that would have connected.
    //
    // The point of going through the policy rather than around it is that changing this
    // one line to `Policy::Require` with a provider is the whole of what enabling TLS
    // costs, and that a `Require` policy with no provider refuses the connection here
    // instead of silently serving it in the clear.
    let tls_config = tls::TlsConfig::insecure();
    let mut startup = pg_wire::read_startup(&mut r)?;
    if startup == Frontend::SslRequest {
        match tls::PgNegotiation::new(&tls_config).decide(true) {
            tls::PgStep::Cleartext(reply) | tls::PgStep::Upgrade(reply) => {
                w.write_all_bytes(&[reply.byte()])?;
            }
            tls::PgStep::Refuse(why) => {
                eprintln!("nilestreamd: {peer} refused: {}", why.message());
                return Ok(());
            }
        }
        startup = pg_wire::read_startup(&mut r)?;
    } else if let tls::PgStep::Refuse(why) = tls::PgNegotiation::new(&tls_config).decide(false) {
        eprintln!("nilestreamd: {peer} refused: {}", why.message());
        return Ok(());
    }
    let Frontend::Startup { params, .. } = startup else {
        return Ok(());
    };
    let get = |k: &str| {
        params
            .iter()
            .find(|(a, _)| a == k)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    let mut session = Session::new(get("user"), get("database"), schema);
    eprintln!(
        "nilestreamd: {peer} connected as `{}` to `{}`",
        session.user, session.database
    );

    let pid = std::process::id();
    pg_wire::write_all(&mut w, &pg_wire::startup_reply(pid, 0x5eed))?;

    while let Ok(msg) = pg_wire::read_message(&mut r) {
        if msg == Frontend::Terminate {
            break;
        }
        let replies = {
            let mut e = engine.lock().unwrap();
            session.handle(msg, &mut *e)
        };
        if replies.is_empty() {
            break;
        }
        pg_wire::write_all(&mut w, &replies)?;
    }
    eprintln!(
        "nilestreamd: {peer} disconnected after {} queries",
        session.queries_served
    );
    Ok(())
}

/// A tiny helper so the `SSLRequest` refusal reads clearly at the call site.
trait WriteBytes {
    fn write_all_bytes(&mut self, b: &[u8]) -> std::io::Result<()>;
}
impl<T: std::io::Write> WriteBytes for T {
    fn write_all_bytes(&mut self, b: &[u8]) -> std::io::Result<()> {
        self.write_all(b)?;
        self.flush()
    }
}
