//! Server assembly: sessions, native protocol, MySQL/PG wire adapters,
//! observability and audit endpoints (thesis 7.3, Appendix D.6/D.9).

pub mod session;
pub mod native_proto;
pub mod mysql_wire;
pub mod pg_wire;
pub mod observability;
pub mod audit;
