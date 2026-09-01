//! **GBS products** — the library. Compositions of mechanisms, and nothing else.
//!
//! Every product here is built from `gbs-mechanisms`, which is built from `gbs-kernel`.
//! None of them touches the ledger, the REV runtime or the consensus layer, and
//! `gbs/tests/layering.rs` fails the build if that ever stops being true.
//!
//! That test is the point. `ARCHITECTURE.md` §1 makes a falsifiable claim — that banking is
//! a library over a general core — and thesis §11.3 states its refutation condition: a
//! product that cannot be expressed without a kernel change. Enforcing the layering
//! mechanically means a product needing such a change *cannot be written* without breaking
//! a named test, which turns the refutation condition from a promise into a build failure.

pub mod fx;
pub mod lending;
pub mod derivatives;
pub mod liquidity;
pub mod tradefinance;
