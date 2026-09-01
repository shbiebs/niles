# ADR 0003: SHA-256 for the hash chain

**Supersedes ADR 0002 (Placeholder hash in the scaffold).**

## Status

Accepted, and built: `nilestream_ledger::chain`.

## Context

ADR 0002 recorded a deliberate stand-in: a 256-bit FNV-1a variant, honest in its own doc
comment about not being collision resistant, so the workspace could build with zero external
dependencies. It was to be swapped for `blake3` at Phase 0.

The architecture review found that the swap had not happened while three documents had come
to depend on it:

* `SPEC-LANGUAGE.md` **L-13** resolves the zero-copy tension by asserting that *the chain is
  the validation* — the trust boundary is exactly where the chain ends.
* `REQUIREMENTS.md` repeats that resolution.
* The thesis's auditability claim rests on a reader being able to detect a spliced, truncated
  or edited history.

**A chain built on a non-collision-resistant hash validates against accident, not against an
adversary.** That distinction is the whole content of "tamper-evident": a ledger whose chain
anyone able to compute a collision can forge is a ledger with a checksum, and it should say
checksum. The placeholder was not wrong to exist; leaving it under those three claims was.

## Decision

Implement **SHA-256 (FIPS 180-4)** in-tree, replacing `Hasher256` behind the same
`new`/`update`/`finalize` API, so nothing downstream needed changing.

**No dependency**, and not out of habit. The chain hash is the *audit artefact*: a dependency
is a thing that can change what a hash is without changing this repository, and every such
change silently invalidates every chain ever computed and every attestation made about one.
Two hundred lines that cannot move are worth more than a crate that is faster. This also keeps
the workspace's zero-dependency property, which is load-bearing for the reproducibility claim
rather than an aesthetic.

The reference implementation: no SIMD, no unrolling, no hand-tuned compression. Constant-time
execution is not required — the inputs are public — and collision resistance comes from the
algorithm rather than from the coding.

## Why SHA-256 rather than BLAKE3, as ADR 0002 intended

BLAKE3 is faster and its tree structure would suit a segment store. SHA-256 was chosen anyway:

* **It is the one an auditor already has.** A published chain digest is meaningful to somebody
  who has never seen this code, and `sha256sum` is on every machine. An attestation whose
  verification requires installing a library is an attestation with a barrier in front of it.
* **The vectors are normative and public.** FIPS 180-4 and NIST CAVP give a test suite that
  makes "this is SHA-256" checkable rather than asserted, which a from-scratch implementation
  needs far more than a vendored one does.
* **Speed is not the constraint here.** The chain hashes one canonical body per epoch, behind
  an `fsync` that costs 93µs on the benchmark machine. SHA-256 at a few hundred MB/s is not
  what the write path is waiting for, and `results/E13-durability.md` records the measured
  share.

If the write path ever becomes hash-bound the decision is worth revisiting, and this ADR is
where the revisit starts.

## Consequences

* **Every chain digest changed.** Values recorded before this ADR do not verify against builds
  after it, which is correct: they were computed with a different function. Fixtures were
  regenerated; `results/E13-durability.md` notes the change and the commit.
* L-13 and `REQUIREMENTS.md` now state what the validation *is* rather than that one exists.
* `chain::hex` renders a digest the way every other tool does, so a value in a log can be
  compared with `sha256sum` output by eye.
* The conformance obligation is a test rather than a claim: `cargo test -p nilestream-ledger`
  runs the NIST vectors including the one-million-character long message, a chunking-invariance
  test (the segment writer hashes a parent and a body in two `update` calls), and a check that
  the placeholder — reproduced in the test itself, so the comparison is derived rather than
  remembered — is not what the build computes.
