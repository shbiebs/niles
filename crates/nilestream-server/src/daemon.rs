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

/// **Wait for every receipt this message's statements produced, in submission order.**
///
/// The receipts belong to one connection's `Session`, so this waits for that connection's own
/// barriers and for nobody else's (A9-F01). Extracted from `serve` so the failure path can be
/// tested directly: a barrier that does not return is not something a test can arrange by
/// asking the storage nicely.
///
/// Stops at the first failure. A message whose first statement did not reach stable storage is
/// answered with that failure whatever its later statements did, because the reply is one
/// reply and a partial acknowledgement is the thing this whole path exists to prevent.
fn await_all(receipts: Vec<crate::rev_engine::Pending>) -> Result<(), String> {
    for p in receipts {
        p.wait()?;
    }
    Ok(())
}

/// The reply for a transaction the storage did not take.
///
/// `58030 io_error`: the transaction was well formed and the storage did not take it. It
/// replaces the framed reply entirely — the rows are in the base but the visible frontier has
/// not moved past them, so no read can observe them and recovery will not carry them.
fn not_durable(why: &str) -> Vec<pg_wire::Backend> {
    vec![
        pg_wire::Backend::ErrorResponse {
            severity: "ERROR".into(),
            code: "58030".into(),
            message: "the transaction was not made durable".into(),
            detail: Some(why.to_string()),
        },
        pg_wire::Backend::ReadyForQuery(b'I'),
    ]
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
        // **This connection's own receipts.** The engine used to own one list of outstanding
        // barriers and this loop drained *the list*, so a concurrent connection could take
        // this one's token and this one would then acknowledge with nothing to wait on
        // (A9-F01). `Session` belongs to this connection and to no other.
        let pending = session.take_receipts();

        // **A failed barrier must not be acknowledged as a commit.**
        //
        // The reply was framed while the epoch was merely applied. If its record never
        // reached stable storage, the client is told so instead — the rows are in the base
        // but the visible frontier has not moved past them, so no read can observe them and
        // recovery will not carry them.
        let mut replies = replies;
        if let Err(why) = await_all(pending) {
            eprintln!("nilestreamd: {peer} append not durable: {why}");
            replies = not_durable(&why);
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

    use super::{await_all, not_durable};
    use crate::pg_wire;

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

        let takes = body.find("take_receipts").expect(
            "`serve` must take the outstanding barriers from its own session; without that \
             the tokens are dropped with the guard and the reply is written before the fsync",
        );
        assert!(
            !body.contains("take_pending"),
            "`serve` must not drain a list the engine owns: another connection's receipt \
             can be in it, and taking it acknowledges this connection's write against \
             somebody else's barrier (A9-F01)"
        );
        let waits = body
            .find("await_all(")
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

    /// **A failed barrier answers the statement that caused it — C9-02.2.**
    ///
    /// The reply was framed while the epoch was merely applied. If its record never reached
    /// stable storage the client is told so instead, and the tag it would have got is
    /// discarded rather than sent.
    #[test]
    fn a_barrier_that_never_returns_replaces_the_reply_with_58030() {
        let framed = vec![
            pg_wire::Backend::CommandComplete("INSERT 0 2".into()),
            pg_wire::Backend::ReadyForQuery(b'I'),
        ];
        let receipts = vec![crate::rev_engine::a_barrier_that_will_never_return(7)];
        let why = await_all(receipts).expect_err("the barrier cannot return");
        let replies = not_durable(&why);
        assert!(
            !replies.iter().any(|m| matches!(
                m,
                pg_wire::Backend::CommandComplete(t) if t.starts_with("INSERT")
            )),
            "a commit tag must not survive a failed barrier: {framed:?}"
        );
        match replies.first() {
            Some(pg_wire::Backend::ErrorResponse { code, detail, .. }) => {
                assert_eq!(code, "58030", "io_error is the code a client acts on");
                assert!(detail.is_some(), "the client is told why");
            }
            other => panic!("expected an ErrorResponse, got {other:?}"),
        }
    }

    /// **A message with no appends waits for nothing — C9-02.1's other half.**
    ///
    /// A reader's reply must not be delayed by, or rewritten because of, somebody else's
    /// barrier. With receipts owned by the session there is nothing for a read to wait on:
    /// the list it drains is its own and it is empty.
    #[test]
    fn a_message_that_appended_nothing_has_no_barrier_to_wait_for() {
        assert!(
            await_all(Vec::new()).is_ok(),
            "a read must not be able to fail on durability"
        );
        // And a receipt that did return leaves the reply alone.
        assert!(await_all(vec![crate::rev_engine::a_barrier_that_has_returned(3)]).is_ok());
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

        // **The marker is the signature `append` actually has.**
        //
        // This arm split on `"fn append(&self, rows: Vec<Row>"`, which is not how the
        // function is written — `rustfmt` broke that signature across four lines long ago.
        // The only place that string existed in `rev_engine.rs` was inside the *lock-order
        // test's* own list of markers, so the split landed in a test module and the
        // `contains("TimedWrite::acquire")` below was satisfied by that module's prose. The
        // arm has been asserting nothing about `append` since the signature was reformatted,
        // and it went on passing because a guard that reads the source can be satisfied by
        // the source of another guard. It surfaced when cycle 10 replaced that marker list
        // with a derived scan and the string disappeared.
        //
        // Two consequences kept here deliberately: the marker is the multi-line form, and
        // the body is required to be found rather than defaulted to the rest of the file by
        // `unwrap_or(len)` — a default that turns "the function was not located" into "the
        // whole file, which certainly contains the string I am looking for".
        const APPEND_SIG: &str = "\n    fn append(\n        &self,\n        rows: Vec<Row>,";
        let append = src
            .split(APPEND_SIG)
            .nth(1)
            .expect("`append` is where the exclusive borrow belongs, at its real signature");
        let append = &append[..append
            .find("\n    }")
            .expect("the body of `append` closes at its own indentation")];
        assert!(
            append.contains("TimedWrite::acquire"),
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
