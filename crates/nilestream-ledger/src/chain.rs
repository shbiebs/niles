//! Hash chaining: `h_e = H(h_{e-1} ‖ canon(δ_e))` (thesis Def. 3.1).
//!
//! # Why this is SHA-256 and not a stand-in
//!
//! Until the architecture review it was a 256-bit FNV-1a variant, labelled a placeholder and
//! honest about it: *"NOT collision resistant"*. The review's objection was that three
//! documents leaned on the chain while that was true.
//!
//! `SPEC-LANGUAGE.md` L-13 states the zero-copy rule as *bounded by trust — the hash chain is
//! the validation*. `REQUIREMENTS.md` repeats it. The thesis's auditability claim rests on a
//! reader being able to detect a spliced or edited history. **A chain built on a
//! non-collision-resistant hash validates against accident and not against an adversary**,
//! and the distinction is the entire content of "tamper-evident". A ledger whose chain can be
//! forged by anyone who can compute a collision is a ledger with a checksum, and it should say
//! checksum.
//!
//! So this is FIPS 180-4 SHA-256, written out, with the NIST vectors as tests.
//!
//! # Why it takes no dependency
//!
//! `nilestream-ledger` depends on nothing, and neither does any crate in this workspace. That
//! is worth keeping precisely here: the chain hash is the **audit artefact**. A dependency is a
//! thing that can change what a hash is without changing this repository, and every such change
//! silently invalidates every chain ever computed and every attestation ever made about one.
//! Two hundred lines that cannot move are worth more than a crate that is faster.
//!
//! The implementation is the reference one — no SIMD, no unrolling, no hand-tuned compression
//! round. Constant-time execution is not required (the inputs are public); **collision
//! resistance** is, and that comes from the algorithm rather than from the coding.
//!
//! # The canonical body
//!
//! `chain_hash` hashes `parent ‖ body`. What a "body" is, byte for byte, is the caller's
//! contract — `gbs-kernel::encode` is one such encoding, and it documents its layout as
//! normative for the same reason this module documents its algorithm. A chain over a
//! non-canonical encoding is a chain that two honest implementations disagree about, and the
//! disagreement looks exactly like tampering.

/// SHA-256 (FIPS 180-4).
///
/// The API is the one the placeholder had — `new`, `update`, `finalize` — so nothing that used
/// it needed changing. What changed is what the output means.
#[derive(Clone)]
pub struct Hasher256 {
    /// The eight working variables, H(0)..H(7).
    state: [u32; 8],
    /// Bytes not yet consumed by a full 64-byte block.
    buffer: [u8; 64],
    buffered: usize,
    /// Total message length in **bits**, which is what the padding encodes.
    bits: u64,
}

impl Default for Hasher256 {
    fn default() -> Self {
        Self::new()
    }
}

/// The first thirty-two bits of the fractional parts of the cube roots of the first
/// sixty-four primes. FIPS 180-4 §4.2.2.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The first thirty-two bits of the fractional parts of the square roots of the first eight
/// primes. FIPS 180-4 §5.3.3.
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

impl Hasher256 {
    pub fn new() -> Self {
        Hasher256 {
            state: H0,
            buffer: [0u8; 64],
            buffered: 0,
            bits: 0,
        }
    }

    pub fn update(&mut self, bytes: &[u8]) -> &mut Self {
        self.bits = self.bits.wrapping_add((bytes.len() as u64).wrapping_mul(8));
        let mut rest = bytes;

        // Fill a partial block first, so `update` may be called with any chunking and produce
        // the same digest — which the segment writer relies on when it hashes a parent and a
        // body in two calls.
        if self.buffered > 0 {
            let want = 64 - self.buffered;
            let take = want.min(rest.len());
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(&rest[..take]);
            self.buffered += take;
            rest = &rest[take..];
            if self.buffered == 64 {
                let block = self.buffer;
                self.compress(&block);
                self.buffered = 0;
            }
        }
        while rest.len() >= 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&rest[..64]);
            self.compress(&block);
            rest = &rest[64..];
        }
        if !rest.is_empty() {
            self.buffer[..rest.len()].copy_from_slice(rest);
            self.buffered = rest.len();
        }
        self
    }

    /// The digest. Takes `&self` — the placeholder's signature — so a caller may finalize and
    /// keep hashing; the padding is applied to a copy.
    pub fn finalize(&self) -> [u8; 32] {
        let mut h = self.clone();
        // FIPS 180-4 §5.1.1: append a 1 bit, then zeros, then the 64-bit big-endian length,
        // so the padded message is a whole number of 512-bit blocks.
        h.update_no_len(&[0x80]);
        while h.buffered != 56 {
            h.update_no_len(&[0x00]);
        }
        let bits = self.bits.to_be_bytes();
        h.update_no_len(&bits);

        let mut out = [0u8; 32];
        for (i, w) in h.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
        }
        out
    }

    /// `update` without counting the bytes into the length — for the padding itself, which is
    /// not part of the message.
    fn update_no_len(&mut self, bytes: &[u8]) {
        let saved = self.bits;
        self.update(bytes);
        self.bits = saved;
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (s, v) in self.state.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *s = s.wrapping_add(v);
        }
    }
}

/// The digest of a byte string, in one call.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = Hasher256::new();
    h.update(bytes);
    h.finalize()
}

/// Chain one canonicalised epoch body onto a parent hash.
///
/// `H(parent ‖ body)`. The parent goes first so that a chain is a *prefix* commitment: the
/// digest at epoch `e` depends on every epoch before it, which is what makes an edit anywhere
/// in the history detectable at the head rather than only where it happened.
pub fn chain_hash(parent: &[u8; 32], canonical_body: &[u8]) -> [u8; 32] {
    let mut h = Hasher256::new();
    h.update(parent);
    h.update(canonical_body);
    h.finalize()
}

/// Render a digest as lowercase hex, for a log or an attestation.
pub fn hex(digest: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── the NIST vectors ────────────────────────────────────────────────────────────
    //
    // FIPS 180-4 and the NIST CAVP short/long message sets. These are what make the claim
    // "this is SHA-256" checkable rather than asserted: an implementation that passes them is
    // computing the function the rest of the world computes, and an attestation about a chain
    // digest means something to somebody who has never seen this code.

    #[test]
    fn the_empty_string() {
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn abc_the_first_fips_example() {
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn the_fifty_six_byte_message_which_needs_a_second_padding_block() {
        // 56 bytes is the sharp edge of the padding rule: the 0x80 byte fits, the 64-bit
        // length does not, so a second block is required. An implementation that got the
        // boundary wrong passes "abc" and fails here.
        assert_eq!(
            hex(&sha256(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn the_one_hundred_and_twelve_byte_message() {
        assert_eq!(
            hex(&sha256(
                b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn\
                  hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
            )),
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"
        );
    }

    #[test]
    fn one_million_letter_a() {
        // The long-message vector. Slow enough to be worth running once, and it exercises the
        // block loop across tens of thousands of compressions — where an off-by-one in the
        // buffering shows up and nowhere else.
        let mut h = Hasher256::new();
        let chunk = vec![b'a'; 1_000];
        for _ in 0..1_000 {
            h.update(&chunk);
        }
        assert_eq!(
            hex(&h.finalize()),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn the_digest_does_not_depend_on_how_the_input_was_chunked() {
        // The property the segment writer relies on: it hashes a parent and a body in two
        // `update` calls, and a hasher that buffered wrongly would produce a different digest
        // from the same bytes delivered differently — a chain that fails to verify against
        // itself depending on how it was written.
        let message: Vec<u8> = (0..1_000u32).map(|i| (i % 251) as u8).collect();
        let whole = sha256(&message);
        for chunk in [1usize, 7, 63, 64, 65, 127, 128, 333] {
            let mut h = Hasher256::new();
            for part in message.chunks(chunk) {
                h.update(part);
            }
            assert_eq!(h.finalize(), whole, "chunked by {chunk}");
        }
    }

    #[test]
    fn finalize_does_not_consume_the_hasher() {
        // `finalize` takes `&self`, as the placeholder's did. Padding a copy means a caller can
        // finalize and keep hashing, and — more importantly — that finalizing twice gives the
        // same answer, which a padding applied in place would not.
        let mut h = Hasher256::new();
        h.update(b"abc");
        let first = h.finalize();
        assert_eq!(first, h.finalize(), "finalizing twice");
        h.update(b"def");
        assert_eq!(
            h.finalize(),
            sha256(b"abcdef"),
            "and hashing continues from there"
        );
    }

    // ── the placeholder is gone ─────────────────────────────────────────────────────

    /// The hasher this module used to be, reproduced so the test below compares against a
    /// value it *derives* rather than one somebody remembered.
    ///
    /// A hard-coded "the old digest was..." constant would be a number with no provenance, and
    /// a test that asserts against one is checking a memory rather than a fact.
    fn the_old_placeholder(bytes: &[u8]) -> [u8; 32] {
        let mut state: [u64; 4] = [
            0xcbf29ce484222325,
            0x9e3779b97f4a7c15,
            0x517cc1b727220a95,
            0x2545f4914f6cdd1d,
        ];
        for (i, b) in bytes.iter().enumerate() {
            let lane = i & 3;
            state[lane] ^= *b as u64;
            state[lane] = state[lane].wrapping_mul(0x100000001b3);
            state[(lane + 1) & 3] = state[(lane + 1) & 3].rotate_left(13) ^ state[lane];
        }
        let mut out = [0u8; 32];
        for (i, s) in state.iter().enumerate() {
            let mut x = *s;
            x ^= x >> 33;
            x = x.wrapping_mul(0xff51afd7ed558ccd);
            x ^= x >> 33;
            out[i * 8..(i + 1) * 8].copy_from_slice(&x.to_le_bytes());
        }
        out
    }

    #[test]
    fn the_placeholder_hash_is_not_what_this_computes() {
        // Guarding against a stale build or a partial revert.
        for message in [&b""[..], b"abc", b"the quick brown fox", &[0u8; 200][..]] {
            assert_ne!(
                sha256(message),
                the_old_placeholder(message),
                "the placeholder is still in the build"
            );
        }
    }

    #[test]
    fn the_placeholder_collides_where_sha256_does_not() {
        // Why the swap was necessary rather than tidy. The old hasher processed bytes into
        // four lanes by position mod 4, so a message whose bytes are permuted *within* their
        // lanes can be made to collide with a short search. Finding a SHA-256 collision is a
        // research result; finding one of these is a loop.
        //
        // The test asserts the weaker, checkable thing: over a small space of messages the
        // placeholder's outputs are far less well separated than SHA-256's. A chain built on it
        // validates against accident, not against an adversary — which is the whole difference
        // between a checksum and tamper-evidence.
        use std::collections::HashSet;
        let messages: Vec<Vec<u8>> = (0..4_096u32)
            .map(|i| vec![(i & 0xff) as u8, ((i >> 8) & 0xff) as u8])
            .collect();

        // Compare the top 16 bits of each digest: a well-mixed function spreads 4,096 inputs
        // across 65,536 buckets with few coincidences, and a poorly-mixed one clusters.
        let bucket = |d: [u8; 32]| u16::from_be_bytes([d[0], d[1]]);
        let sha_buckets: HashSet<u16> = messages.iter().map(|m| bucket(sha256(m))).collect();
        let old_buckets: HashSet<u16> = messages
            .iter()
            .map(|m| bucket(the_old_placeholder(m)))
            .collect();

        assert!(
            sha_buckets.len() > old_buckets.len(),
            "SHA-256 spread {} inputs over {} buckets; the placeholder managed {}",
            messages.len(),
            sha_buckets.len(),
            old_buckets.len()
        );
        assert!(
            sha_buckets.len() > 3_900,
            "SHA-256 should show near-birthday-bound spreading, not {}",
            sha_buckets.len()
        );
    }

    #[test]
    fn a_digest_is_not_a_checksum_and_a_one_bit_change_says_so() {
        // Avalanche. Not a proof of collision resistance — nothing short of the literature is —
        // but the property whose absence would mean the swap had not really happened.
        let a = sha256(b"the quick brown fox");
        let b = sha256(b"the quick brown fox!");
        let differing_bits: u32 = a.iter().zip(&b).map(|(x, y)| (x ^ y).count_ones()).sum();
        assert!(
            (96..=160).contains(&differing_bits),
            "about half of 256 bits should differ; {differing_bits} did"
        );
    }

    // ── the chain ───────────────────────────────────────────────────────────────────

    #[test]
    fn chain_is_deterministic_and_parent_sensitive() {
        let a = chain_hash(&[0; 32], b"epoch-0");
        let b = chain_hash(&[0; 32], b"epoch-0");
        let c = chain_hash(&a, b"epoch-0");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn body_sensitive() {
        assert_ne!(chain_hash(&[0; 32], b"x"), chain_hash(&[0; 32], b"y"));
    }

    #[test]
    fn the_chain_commits_to_every_epoch_before_it() {
        // What makes an edit detectable at the *head* rather than only where it happened, and
        // therefore what makes a single published digest an attestation about a whole history.
        let bodies: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d", b"e"];
        let walk = |bodies: &[&[u8]]| {
            let mut h = [0u8; 32];
            for b in bodies {
                h = chain_hash(&h, b);
            }
            h
        };
        let honest = walk(&bodies);

        // Edit epoch 1 — the second of five — and the head differs.
        let mut tampered = bodies.clone();
        tampered[1] = b"B";
        assert_ne!(
            walk(&tampered),
            honest,
            "an edit in the middle changed the head"
        );

        // Drop an epoch: also detectable, which is what makes truncation evidence rather than
        // an ordinary short read.
        let truncated: Vec<&[u8]> = bodies[..4].to_vec();
        assert_ne!(walk(&truncated), honest);

        // Reorder two epochs, keeping the same set of bodies.
        let mut swapped = bodies.clone();
        swapped.swap(2, 3);
        assert_ne!(walk(&swapped), honest, "reordering is detectable too");
    }

    #[test]
    fn hex_renders_a_digest_the_way_every_other_tool_does() {
        assert_eq!(hex(&[0u8; 32]), "0".repeat(64));
        assert_eq!(hex(&[0xffu8; 32]), "f".repeat(64));
        assert_eq!(hex(&sha256(b"abc")).len(), 64);
    }
}
