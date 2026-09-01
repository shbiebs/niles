# ADR 0002: Placeholder hash in the scaffold

**SUPERSEDED by ADR 0003 (SHA-256 for the hash chain).**

Kept because a superseded decision that is deleted looks like a decision that was never made,
and the reasoning below is the reasoning somebody would repeat.

---

# ADR 0002: Placeholder hash in the scaffold

The scaffold ships an in-tree, NON-cryptographic 256-bit hasher
(`nilestream_ledger::chain::Hasher256`) so the workspace builds with zero
external dependencies. It mirrors blake3's update/finalize API; Phase 0
replaces it with the real `blake3` crate (uncomment in Cargo.tomls).
Nothing downstream may assume collision resistance until that swap.

---

## Why it was superseded

The swap did not happen at Phase 0, and in the meantime `SPEC-LANGUAGE.md` L-13,
`REQUIREMENTS.md` and the thesis's auditability claim all came to rest on the chain being
*validation*. A chain built on a non-collision-resistant hash validates against accident and
not against an adversary. The placeholder was not wrong to exist; leaving it under those
claims was. See ADR 0003.
