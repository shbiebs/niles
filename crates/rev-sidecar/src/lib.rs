//! **The REV sidecar** — arm H3 of E27 (cycle 14, R2-03; decision D-6).
//!
//! The hybrid the engine rule of the round-2 order asks about: PostgreSQL stays the ledger —
//! append-only, strictly serialisable, durable — and the derived read side is a
//! reconstructible epoch-anchored view (REV) kept by `nilestream-core` in a separate process
//! that reads PostgreSQL's logical replication stream. If this arm matches Nilestream within
//! the gate on memory and p99 under eviction, the recommended product is this hybrid and
//! Nilestream is the instrument that established the mechanism (§7).
//!
//! # The mapping, stated once
//!
//! | REV concept | here |
//! |---|---|
//! | the base | `arm.postings` in PostgreSQL, append-only by trigger |
//! | an epoch | the ledger's own epoch id (`arm.postings.epoch`), one per transaction; its **commit LSN** is where it becomes visible — the sidecar's frontier advances when the stream delivers that commit |
//! | a delta | the `pgoutput` inserts of one committed transaction, summed per `(acct, cur)` |
//! | an upquery at `e` | `select sum(amt), count(*) from arm.postings where acct = $1 and cur = $2 and epoch <= e` on a pooled SCRAM connection — the base's own index `postings_anchor (acct, cur, epoch)` |
//! | the view | `nilestream-core`'s `Rev`, installed from the same Niles text as Nilestream's balance view, demand mode, LRU, the same budget |
//!
//! `nilestream-core` is reused, not forked: [`base::PgBase`] implements its `Base` trait, and
//! reads go through `Rev`'s two-phase `begin_read` / `finish_fold` exactly as the daemon's do,
//! so concurrent readers of one key share one upquery and no view lock is held across one.
//!
//! # What the wire serves
//!
//! `select value from rev_read(A, C)` (head) and `select value from rev_read(A, C, E)` (as of
//! epoch E) are answered from the REV. Everything else is forwarded verbatim to PostgreSQL on
//! the session's own upstream connection and relayed. A write (`select arm.post_txn(…)`) is
//! acknowledged only after the stream has delivered and applied its epoch, so a read that
//! follows a write on any session sees it: the frontier at the time of a head read is the
//! newest epoch any client has been told is committed.
//!
//! # Scope
//!
//! The listener is loopback-only and asks its clients for no password, exactly as
//! `nilestreamd` does; its own connections to PostgreSQL authenticate with SCRAM-SHA-256 as
//! the provisioned role. It is an experimental arm, not a product: it serves one view.

pub mod base;
pub mod pgoutput;
pub mod server;
pub mod stream;
