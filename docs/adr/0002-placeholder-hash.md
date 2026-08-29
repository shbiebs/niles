# ADR 0002: Placeholder hash in the scaffold

The scaffold ships an in-tree, NON-cryptographic 256-bit hasher
(`nilestream_ledger::chain::Hasher256`) so the workspace builds with zero
external dependencies. It mirrors blake3's update/finalize API; Phase 0
replaces it with the real `blake3` crate (uncomment in Cargo.tomls).
Nothing downstream may assume collision resistance until that swap.
