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
        idem: IdemKey window 1_000_000.epochs,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_postings on postings (acct) anchor;
}
";

use crate::pg_wire::{self, Frontend};
use crate::session::Session;
use crate::tls;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

/// Accept connections until the listener is dropped, serving each on its own thread.
///
/// One thread per connection, which is the right shape at this scale and is stated rather
/// than defended: the write path is serialised through a single sealer anyway, so
/// connection-level concurrency is about *reads*, and reads over an immutable base need no
/// coordination at all.
pub fn accept_loop(
    listener: TcpListener,
    schema: String,
    engine: Arc<dyn crate::session::Serving + Send + Sync>,
) {
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
    engine: Arc<dyn crate::session::Serving + Send + Sync>,
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
        // **No lock here at all, and that is the second half of the repair.**
        //
        // This used to take one mutex around the whole of `Session::handle` — parse, plan,
        // execute, frame — and, before T-05, around the `fsync` inside it. The mutex is
        // gone: the engine takes the lock it needs, where it needs it, for as long as the
        // thing it protects is being touched. A fold takes the base **shared**, so folds run
        // concurrently with each other; an append takes it exclusively for the apply alone;
        // a keyed read takes the view's own lock for microseconds. `select nilestream_lock`
        // reports the base lock, which is now the only one a connection can queue on.
        //
        // The barrier is still waited on out here, before the reply is written, exactly as
        // T-05 left it: the acknowledgement follows the fsync, and nothing about releasing
        // the read side changes that.
        // **The outer boundary, measured rather than inferred.** From here to the reply
        // reaching the socket: `Session::handle`, the durability barrier, and the framing.
        // Subtracting `STATEMENT` from it gives the barrier wait and the wire — the term the
        // slow-read table's "unaccounted" column claimed to bound at 0–2 µs while measuring
        // nothing outside `answer_from_view` (A9-F07).
        let wire_began = std::time::Instant::now();
        let replies = session.handle(msg, &*engine);
        let pending = crate::session::Serving::take_pending(&*engine);

        // **A failed barrier must not be acknowledged as a commit.**
        //
        // The reply was framed while the epoch was merely applied. If its record never
        // reached stable storage, the client is told so instead — the rows are in the base
        // but the visible frontier has not moved past them, so no read can observe them and
        // recovery will not carry them.
        let mut replies = replies;
        for p in pending {
            if let Err(why) = p.wait() {
                eprintln!("nilestreamd: {peer} append not durable: {why}");
                replies = vec![
                    pg_wire::Backend::ErrorResponse {
                        severity: "ERROR".into(),
                        // `58030 io_error`: the transaction was well formed and the storage
                        // did not take it.
                        code: "58030".into(),
                        message: "the transaction was not made durable".into(),
                        detail: Some(why),
                    },
                    pg_wire::Backend::ReadyForQuery(b'I'),
                ];
                break;
            }
        }
        if replies.is_empty() {
            break;
        }
        pg_wire::write_all(&mut w, &replies)?;
        crate::lockstats::WIRE.record(0, wire_began.elapsed().as_nanos() as u64);
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

#[cfg(test)]
mod ordering_guard {
    //! **The acknowledgement must follow the barrier, and that is a source-level property.**
    //!
    //! `serve` frames a reply with the engine locked, releases the lock, waits on every
    //! outstanding barrier, and only then writes. Remove the wait and nothing observable
    //! changes on a machine where the storage never fails: the reply is still correct, the
    //! rows are still applied, and every existing test still passes — while the client is now
    //! told "committed" before the commit is on stable storage. That is the same shape as the
    //! nested-`cargo` defect in `counterproposal.rs`: invisible to behaviour, which is exactly
    //! why it needs a guard that reads the source.
    //!
    //! What the *behavioural* tests cover is the other half — that an applied epoch is not
    //! visible until its barrier returns — in `rev_engine::visibility_tests`. Together: the
    //! frontier cannot publish early, and the reply cannot be written early.

    #[test]
    fn the_reply_is_written_only_after_every_outstanding_barrier_is_awaited() {
        let src = include_str!("daemon.rs");
        let serve = src
            .split("pub fn serve(")
            .nth(1)
            .expect("`serve` is where the ordering lives");

        // Search the *message loop*, not the whole function: the startup handshake writes a
        // reply of its own before any transaction exists, and matching that write would make
        // the guard pass for the wrong reason.
        let loop_start = serve
            .find("while let Ok(msg) = pg_wire::read_message")
            .expect("`serve` must have a message loop");
        let body = &serve[loop_start..];

        let takes = body.find("take_pending").expect(
            "`serve` must take the outstanding barriers from the engine; without that the \
             tokens are dropped with the guard and the reply is written before the fsync",
        );
        let waits = body
            .find(".wait()")
            .expect("`serve` must wait on the barriers it took");
        let writes = body
            .find("pg_wire::write_all")
            .expect("`serve` must write the reply");

        assert!(
            takes < waits && waits < writes,
            "the order in `serve` must be take ({takes}), wait ({waits}), write ({writes}). \
             Writing before waiting acknowledges a transaction that is not yet durable."
        );
    }

    /// The barrier must not be awaited under any lock — and after T-06 `serve` holds none.
    ///
    /// The T-05 form of this guard asserted that `.wait()` came *after* the locked block
    /// closed. There is no locked block in `serve` any more: the engine takes the lock it
    /// needs where it needs it, so the property to hold is the stronger one — this function
    /// acquires nothing, and therefore cannot be holding anything when it waits.
    #[test]
    fn serve_holds_no_lock_of_its_own() {
        let src = include_str!("daemon.rs");
        let serve = src.split("pub fn serve(").nth(1).expect("`serve`");
        let loop_start = serve
            .find("while let Ok(msg) = pg_wire::read_message")
            .expect("`serve` must have a message loop");
        let body = &serve[loop_start..];
        for taken in [
            "Timed::acquire",
            "TimedWrite::acquire",
            "engine.lock(",
            "engine.write(",
        ] {
            assert!(
                !body.contains(taken),
                "`serve` acquires `{taken}`. The message loop must hold nothing: a lock taken \
                 here spans parse, plan, execute and frame for every connection, which is the \
                 defect T-05 and T-06 removed in two halves. The engine locks what it \
                 touches, for as long as it touches it."
            );
        }
    }
}

#[cfg(test)]
mod read_concurrency_guard {
    //! **A read of the base must take it shared, and that is a source-level property.**
    //!
    //! Two folds over an append-only prefix have nothing to say to each other, and until
    //! T-06 they ran one at a time: the read path took `&mut` on the engine because
    //! `Base::reconstruct` took `&mut self` to increment a counter. Turning the read path
    //! shared changes no answer — the prefix is frozen, the filters are the same, the
    //! reference evaluator is the same — so **nothing in the behavioural suite moves if this
    //! is reverted**, and every fold serialises again. That is what a source-level guard is
    //! for.
    //!
    //! The measurement is in `results/E19-scaling/`: the `fold` workload, which is an
    //! unkeyed `group by acct` the maintained view refuses, at 1/2/4/8 connections.

    #[test]
    fn the_read_path_takes_the_base_shared_and_the_append_takes_it_exclusively() {
        let src = include_str!("rev_engine.rs");

        let base = src
            .split("fn base(&self)")
            .nth(1)
            .expect("the read path's borrow of the base is `RevEngine::base`");
        let body = &base[..base.find("\n    }").expect("the accessor closes")];
        assert!(
            body.contains("TimedRead::acquire"),
            "`RevEngine::base` must take the base **shared**. An exclusive borrow here puts \
             every fold back in a queue behind every other fold, which no behavioural test \
             can see because every answer stays correct."
        );

        let append = src
            .split("fn append(&self, rows: Vec<Row>")
            .nth(1)
            .expect("`append` is where the exclusive borrow belongs");
        assert!(
            append[..append.find("\n    }").unwrap_or(append.len())]
                .contains("TimedWrite::acquire"),
            "`append` must take the base exclusively: `submit` chains a hash and mutates five \
             indexes, and the write guard is also what makes the epoch ids it assigns \
             contiguous."
        );
    }

    /// The counter that used to force the exclusive borrow must stay shared-writable.
    #[test]
    fn reconstruction_counts_through_a_shared_borrow() {
        let ledger = include_str!("../../proto-engine/src/ledger.rs");
        assert!(
            ledger.contains("fn reconstruct(&self, key: &nilestream_core::rev::Key"),
            "`Base::reconstruct` must take `&self`. It took `&mut self` for one counter, and \
             a `&mut self` taken for a counter is a `&mut self` taken over the whole base."
        );
        assert!(
            ledger.contains("fn count_rows(&self, n: u64)"),
            "the row counter must be writable through a shared borrow, or `reconstruct` \
             cannot be."
        );
    }
}
