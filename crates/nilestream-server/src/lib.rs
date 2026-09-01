//! **Nilestream server** — the daemon and its wire protocols (thesis §6.9, §7.3).
//!
//! The adoption argument of §6.9 is that a database nobody can connect to with the tools
//! they already have is a database nobody adopts. This crate is that argument's artifact.
//!
//! | Module | Status |
//! |---|---|
//! | [`pg_wire`] | **Built** — PostgreSQL wire protocol v3, simple query path |
//! | [`session`] | **Built** — query → Niles → IR → verifier → REV runtime, one path |
//! | [`rev_engine`] | **Built** — reads served by a partial view over an immutable ledger |
//! | `mysql_wire` | Specified in §7.3; not built |
//! | `native_proto` | Specified; not built |
//! | `audit`, `observability` | Specified in Appendix D.8; not built |
//!
//! The design commitment worth restating: a wire protocol here is a **surface, not a
//! semantics**. There is no compatibility layer with its own execution path, because two
//! ways to compute an answer is two answers that can disagree — which is the seam this
//! whole thesis argues against.

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
