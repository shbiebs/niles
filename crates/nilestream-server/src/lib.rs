// A few accessors exist for introspection and for the benchmark harness rather than for
// the server's own path. The blanket allow this replaces was justified by "there is no
// write path and the extended query protocol is not wired into the connection loop" —
// both of which are now false, so the justification had to go with them.
#![allow(dead_code)]
// `Backend::BackendKeyData` is the PostgreSQL message name; renaming it would make the
// protocol harder to read against the specification.
#![allow(clippy::enum_variant_names)]

//! **Nilestream server** — the daemon and its wire protocols (thesis §6.9, §7.3).
//!
//! The adoption argument of §6.9 is that a database nobody can connect to with the tools
//! they already have is a database nobody adopts. This crate is that argument's artifact.
//!
//! | Module | Status |
//! |---|---|
//! | [`pg_wire`] | **Built** — PostgreSQL wire protocol v3, simple query path |
//! | [`session`] | **Built** — query → Niles → IR → verifier → REV runtime, one path |
//! | [`rev_engine`] | **Built** — the compiled circuit is evaluated over the base; the partial view is the point path |
//! | [`extended`] | **Built** — `Parse`/`Bind`/`Describe`/`Execute`/`Sync`, with plans keyed by schema epoch |
//! | `mysql_wire` | **A codec, with no listener.** Frames encode and decode and nothing calls them; §7.3 specifies the protocol and this crate does not serve it |
//! | `native_proto` | Specified; not built |
//! | `audit`, `observability` | Specified in Appendix D.8; not built |
//!
//! The design commitment worth restating: a wire protocol here is a **surface, not a
//! semantics**. There is no compatibility layer with its own execution path, because two
//! ways to compute an answer is two answers that can disagree — which is the seam this
//! whole thesis argues against.
//!
//! That commitment was, until this revision, false of this crate. The session compiled the
//! client's SQL, verified the circuit, and then **discarded it**: the answer came from a
//! hard-coded fold of `sum(amt)` over an account scraped out of the query text with a digit
//! scanner. Two different questions about one account returned the same number. The
//! compiler was a decoration on a fixed answer, which is the compatibility-layer failure in
//! its purest form — one path that parses and another that answers.

pub mod audit;
pub mod daemon;
pub mod extended;
pub mod mysql_wire;
pub mod native_proto;
pub mod observability;
pub mod pg_wire;
pub mod rev_engine;
pub mod session;
pub mod tls;
