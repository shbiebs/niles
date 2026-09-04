//! XChaCha20-Poly1305 AEAD, for the end-to-end encrypted payloads the ledger carries
//! (thesis Ch. 6.5).
//!
//! # Why this is written out here
//!
//! The workspace takes no dependencies, and in the environment this was built in crates.io is
//! unreachable, so an audited AEAD crate is not an option that exists. That is the whole reason
//! this file is hand-written. It is not a preference.
//!
//! It should be said plainly that this is a worse position to be in than [`crate::chain`] is.
//! A hand-rolled hash over public inputs fails loudly: it either matches the NIST vectors or it
//! does not, and nothing about the failure is subtle. A hand-rolled AEAD holds a secret key, and
//! the ways it can be wrong include ways that pass every vector — a tag compared with an early
//! return leaks its own answer through timing, a nonce reused across two messages destroys the
//! confidentiality of both, a partial plaintext returned before the tag is checked hands an
//! attacker a decryption oracle. Vectors do not catch any of those. They are structural.
//!
//! So the mitigation here is two things and no more than two things. The first is the published
//! vectors in the test module below: RFC 8439 §2.3.2, §2.4.2, §2.5.2 and §2.8.2, and
//! draft-irtf-cfrg-xchacha §2.2.1 and A.3.1. Passing them means this computes the same function
//! the rest of the world computes, which is the necessary condition and not the sufficient one.
//! The second is that the tag comparison is constant-time by construction, that [`open`] checks
//! the tag before it decrypts anything, and that neither function is given a nonce to choose.
//!
//! What this file is not is audited, side-channel-hardened against anything beyond the tag
//! compare, or fast. The reference implementation is what it is: no SIMD, no unrolling, and the
//! Poly1305 accumulator carries through five 26-bit limbs because that is the arrangement whose
//! bounds can be argued in a comment.
//!
//! # The swap boundary
//!
//! A deployment that has a supply chain should replace this. The surface a caller sees is
//! [`seal`] and [`open`] and the three length constants, deliberately, so that replacing the
//! body of this module with a call into an audited implementation is a change to one file and
//! to nothing that uses it. Keeping that boundary narrow is the reason there is no streaming
//! API, no in-place variant, and no exposed key schedule: every one of those would be a second
//! thing the replacement has to match.
//!
//! # What the algorithms are
//!
//! ChaCha20 is RFC 8439 §2.3–2.4. Poly1305 is RFC 8439 §2.5, used as a one-time authenticator
//! whose key comes from ChaCha20 block zero, per §2.6. The AEAD composition — the MAC covering
//! `aad ‖ pad16(aad) ‖ ct ‖ pad16(ct) ‖ le64(|aad|) ‖ le64(|ct|)` — is §2.8. HChaCha20 and the
//! 24-byte nonce are draft-irtf-cfrg-xchacha §2.2 and §3. The extended nonce is the reason to
//! prefer XChaCha20 over plain ChaCha20-Poly1305 here: 192 bits is enough that a random nonce
//! per message is safe without a counter that survives a crash, and a ledger writer that has to
//! remember a 96-bit counter across a restart is a ledger writer with a way to reuse one.

/// A 256-bit key. Anything shorter is not this construction.
pub const KEY_LEN: usize = 32;

/// A 192-bit nonce. The width is the point: it is large enough to draw at random per message.
pub const NONCE_LEN: usize = 24;

/// The Poly1305 tag, appended to the ciphertext by [`seal`] and consumed by [`open`].
pub const TAG_LEN: usize = 16;

// ── ChaCha20 ────────────────────────────────────────────────────────────────────────────────

/// `"expand 32-byte k"`, as four little-endian words. RFC 8439 §2.3.
///
/// The constant is in the state so that the first row is not attacker-influenced; it is not a
/// magic number that could be any other value.
const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

/// The quarter-round of RFC 8439 §2.1, operating on four indices of the state.
///
/// The additions are wrapping because the algorithm is defined mod 2^32; a checked add here
/// would panic on perfectly valid input, which is why it is spelled out rather than left to the
/// operator.
fn quarter_round(s: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(16);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(12);
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(8);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(7);
}

/// Ten double-rounds — four column quarter-rounds then four diagonal ones — which is what
/// "20 rounds" means. Shared by ChaCha20 and HChaCha20, which differ only in what they do with
/// the state afterwards.
fn twenty_rounds(s: &mut [u32; 16]) {
    for _ in 0..10 {
        quarter_round(s, 0, 4, 8, 12);
        quarter_round(s, 1, 5, 9, 13);
        quarter_round(s, 2, 6, 10, 14);
        quarter_round(s, 3, 7, 11, 15);
        quarter_round(s, 0, 5, 10, 15);
        quarter_round(s, 1, 6, 11, 12);
        quarter_round(s, 2, 7, 8, 13);
        quarter_round(s, 3, 4, 9, 14);
    }
}

/// Reads a little-endian `u32` out of a four-byte window.
///
/// Every multi-byte quantity in ChaCha20 and Poly1305 is little-endian; a big-endian reader here
/// produces an implementation that is self-consistent, round-trips its own output, and agrees
/// with nobody. The vectors are what catch it.
fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Lays out the sixteen-word state: constants, key, counter, nonce. RFC 8439 §2.3.
fn state(key: &[u8; KEY_LEN], counter: u32, nonce: &[u8; 12]) -> [u32; 16] {
    let mut s = [0u32; 16];
    s[..4].copy_from_slice(&CONSTANTS);
    for (i, chunk) in key.chunks_exact(4).enumerate() {
        s[4 + i] = le32(chunk);
    }
    s[12] = counter;
    for (i, chunk) in nonce.chunks_exact(4).enumerate() {
        s[13 + i] = le32(chunk);
    }
    s
}

/// One 64-byte keystream block. RFC 8439 §2.3.
///
/// The final addition of the original state is what makes the permutation one-way in the sense
/// the cipher needs; without it this is HChaCha20, which is a different function used for a
/// different purpose and is deliberately a separate item below.
fn block(key: &[u8; KEY_LEN], counter: u32, nonce: &[u8; 12]) -> [u8; 64] {
    let initial = state(key, counter, nonce);
    let mut working = initial;
    twenty_rounds(&mut working);

    let mut out = [0u8; 64];
    for (i, (w, init)) in working.iter().zip(initial.iter()).enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&w.wrapping_add(*init).to_le_bytes());
    }
    out
}

/// XORs `data` with the keystream starting at `counter`. Encryption and decryption are the same
/// operation, which is why there is one function and not two.
///
/// The counter increment wraps rather than panicking. A message long enough to wrap is 256 GiB,
/// far past anything this ledger writes as a single payload, but a panic in a cryptographic
/// routine is a denial of service reachable from a length, so it is not left to chance.
fn keystream_xor(key: &[u8; KEY_LEN], counter: u32, nonce: &[u8; 12], data: &mut [u8]) {
    for (i, chunk) in data.chunks_mut(64).enumerate() {
        let ks = block(key, counter.wrapping_add(i as u32), nonce);
        for (byte, k) in chunk.iter_mut().zip(ks.iter()) {
            *byte ^= k;
        }
    }
}

/// HChaCha20: key plus a 16-byte nonce to a 32-byte subkey. draft-irtf-cfrg-xchacha §2.2.
///
/// Two details separate it from [`block`] and both matter. There is no final addition of the
/// initial state, and the output is words 0..4 and 12..16 rather than all sixteen — the rows
/// that were the constants and the nonce, not the rows that were the key. Getting either wrong
/// yields a subkey that is a deterministic function of the key and nonce, so everything still
/// round-trips locally and nothing interoperates.
pub fn hchacha20(key: &[u8; KEY_LEN], nonce: &[u8; 16]) -> [u8; 32] {
    let mut s = [0u32; 16];
    s[..4].copy_from_slice(&CONSTANTS);
    for (i, chunk) in key.chunks_exact(4).enumerate() {
        s[4 + i] = le32(chunk);
    }
    for (i, chunk) in nonce.chunks_exact(4).enumerate() {
        s[12 + i] = le32(chunk);
    }
    twenty_rounds(&mut s);

    let mut out = [0u8; 32];
    for (i, w) in s[..4].iter().chain(s[12..].iter()).enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

// ── Poly1305 ────────────────────────────────────────────────────────────────────────────────

/// The 26-bit limb mask. Five of these hold the 130-bit accumulator.
const LIMB: u64 = (1 << 26) - 1;

/// Poly1305 over the prime 2^130 − 5. RFC 8439 §2.5.
///
/// # Why 26-bit limbs
///
/// The accumulator is 130 bits wide and every block multiplies it by `r`, so the intermediate
/// product is 260 bits and does not fit any primitive. Five limbs of 26 bits is the arrangement
/// whose bounds are small enough to state: each limb stays below 2^27 between blocks, the
/// pre-multiplied `r` limbs stay below 5·2^26 < 2^29, and each of the five column sums is
/// therefore below 5·2^27·2^29 = 5·2^56 < 2^59. That fits a `u128` with room to spare, so no
/// step of the reduction can silently overflow. The alternative — a general bignum — would be
/// correct too, and its bounds would be an argument about code rather than about arithmetic.
///
/// This is a *one-time* authenticator. The key comes from ChaCha20 block zero of the message's
/// own nonce and is used for exactly one message; reusing it across two messages reveals `r` and
/// with it the ability to forge. Nothing in this module lets a caller supply one.
struct Poly1305 {
    /// The clamped multiplier, in five 26-bit limbs.
    r: [u64; 5],
    /// The addend applied once at the end, kept whole because it is only ever used mod 2^128.
    s: u128,
    /// The accumulator, in five 26-bit limbs.
    h: [u64; 5],
    /// Bytes not yet consumed by a full 16-byte block.
    buffer: [u8; 16],
    buffered: usize,
}

impl Poly1305 {
    /// Splits and clamps the one-time key. RFC 8439 §2.5.
    ///
    /// The clamp is not decoration: it forces the top four bits of each of `r`'s four 32-bit
    /// words and the bottom two bits of three of them to zero, which is exactly the condition
    /// that keeps the limb bounds above true and the multiplication carry-free enough to reduce
    /// in one pass.
    fn new(key: &[u8; 32]) -> Self {
        let mut low = [0u8; 16];
        low.copy_from_slice(&key[..16]);
        let mut high = [0u8; 16];
        high.copy_from_slice(&key[16..]);

        let t = u128::from_le_bytes(low) & 0x0fff_fffc_0fff_fffc_0fff_fffc_0fff_ffff;
        let limb = LIMB as u128;
        Poly1305 {
            r: [
                (t & limb) as u64,
                ((t >> 26) & limb) as u64,
                ((t >> 52) & limb) as u64,
                ((t >> 78) & limb) as u64,
                ((t >> 104) & limb) as u64,
            ],
            s: u128::from_le_bytes(high),
            h: [0; 5],
            buffer: [0; 16],
            buffered: 0,
        }
    }

    /// Absorbs one 16-byte block: `h = (h + n) · r mod 2^130 − 5`.
    ///
    /// `high` carries the appended 1 bit that makes each block distinct from the same bytes
    /// followed by zeros — 2^128 for a full block, which lands at 2^24 in the fifth limb, and 0
    /// for a final short block, which encodes its own 1 byte inline instead. Dropping that bit
    /// is the classic length-extension hole in a Poly1305 clone.
    fn absorb(&mut self, data: &[u8; 16], high: u64) {
        let n = u128::from_le_bytes(*data);
        let limb = LIMB as u128;
        let h0 = self.h[0] + (n & limb) as u64;
        let h1 = self.h[1] + ((n >> 26) & limb) as u64;
        let h2 = self.h[2] + ((n >> 52) & limb) as u64;
        let h3 = self.h[3] + ((n >> 78) & limb) as u64;
        let h4 = self.h[4] + (n >> 104) as u64 + high;

        let [r0, r1, r2, r3, r4] = self.r;
        // Folding 2^130 back in costs a factor of 5, so the wrapped-around terms are `r · 5`.
        let (s1, s2, s3, s4) = (r1 * 5, r2 * 5, r3 * 5, r4 * 5);
        let mul = |a: u64, b: u64| (a as u128) * (b as u128);

        let d0 = mul(h0, r0) + mul(h1, s4) + mul(h2, s3) + mul(h3, s2) + mul(h4, s1);
        let d1 = mul(h0, r1) + mul(h1, r0) + mul(h2, s4) + mul(h3, s3) + mul(h4, s2);
        let d2 = mul(h0, r2) + mul(h1, r1) + mul(h2, r0) + mul(h3, s4) + mul(h4, s3);
        let d3 = mul(h0, r3) + mul(h1, r2) + mul(h2, r1) + mul(h3, r0) + mul(h4, s4);
        let d4 = mul(h0, r4) + mul(h1, r3) + mul(h2, r2) + mul(h3, r1) + mul(h4, r0);

        let mut carry = d0 >> 26;
        let n0 = (d0 as u64) & LIMB;
        let d1 = d1 + carry;
        carry = d1 >> 26;
        let n1 = (d1 as u64) & LIMB;
        let d2 = d2 + carry;
        carry = d2 >> 26;
        let n2 = (d2 as u64) & LIMB;
        let d3 = d3 + carry;
        carry = d3 >> 26;
        let n3 = (d3 as u64) & LIMB;
        let d4 = d4 + carry;
        carry = d4 >> 26;
        let n4 = (d4 as u64) & LIMB;

        // The carry out of the top limb wraps to the bottom multiplied by 5, which is the
        // definition of reducing mod 2^130 − 5.
        let mut m0 = n0 + (carry as u64) * 5;
        let m1 = n1 + (m0 >> 26);
        m0 &= LIMB;
        self.h = [m0, m1, n2, n3, n4];
    }

    /// Feeds arbitrary-length input, buffering across calls so that the AEAD can hand over the
    /// associated data, the padding, the ciphertext and the length trailer as separate slices
    /// without materialising the concatenation.
    fn update(&mut self, mut data: &[u8]) {
        const FULL_BLOCK_BIT: u64 = 1 << 24;

        if self.buffered > 0 {
            let take = (16 - self.buffered).min(data.len());
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(&data[..take]);
            self.buffered += take;
            data = &data[take..];
            if self.buffered == 16 {
                let full = self.buffer;
                self.absorb(&full, FULL_BLOCK_BIT);
                self.buffered = 0;
            }
        }
        while data.len() >= 16 {
            let mut full = [0u8; 16];
            full.copy_from_slice(&data[..16]);
            self.absorb(&full, FULL_BLOCK_BIT);
            data = &data[16..];
        }
        if !data.is_empty() {
            self.buffer[..data.len()].copy_from_slice(data);
            self.buffered = data.len();
        }
    }

    /// Finishes the last short block, reduces fully into the canonical range, and adds `s`.
    ///
    /// The final conditional subtraction of the prime is done with a mask rather than a branch.
    /// The tag itself is about to be published, so the secrecy argument is thin, but the
    /// accumulator on the way there is not public and a branch on it is a branch on key
    /// material.
    fn finalize(mut self) -> [u8; TAG_LEN] {
        if self.buffered > 0 {
            let used = self.buffered;
            let mut last = self.buffer;
            last[used] = 1;
            for byte in last[used + 1..].iter_mut() {
                // The tail may hold bytes from an earlier, longer partial block.
                *byte = 0;
            }
            self.absorb(&last, 0);
            self.buffered = 0;
        }

        let [mut h0, mut h1, mut h2, mut h3, mut h4] = self.h;
        let mut carry = h1 >> 26;
        h1 &= LIMB;
        h2 += carry;
        carry = h2 >> 26;
        h2 &= LIMB;
        h3 += carry;
        carry = h3 >> 26;
        h3 &= LIMB;
        h4 += carry;
        carry = h4 >> 26;
        h4 &= LIMB;
        h0 += carry * 5;
        carry = h0 >> 26;
        h0 &= LIMB;
        h1 += carry;

        // g = h + 5. If that does not borrow out of the 2^130 place then h ≥ p and g is h − p.
        let mut g0 = h0 + 5;
        carry = g0 >> 26;
        g0 &= LIMB;
        let mut g1 = h1 + carry;
        carry = g1 >> 26;
        g1 &= LIMB;
        let mut g2 = h2 + carry;
        carry = g2 >> 26;
        g2 &= LIMB;
        let mut g3 = h3 + carry;
        carry = g3 >> 26;
        g3 &= LIMB;
        let g4 = (h4 + carry).wrapping_sub(1 << 26);

        // Borrow sets the sign bit, so `mask` is all zeros when h < p and all ones when h ≥ p.
        let mask = (g4 >> 63).wrapping_sub(1);
        let keep = !mask;
        h0 = (h0 & keep) | (g0 & mask);
        h1 = (h1 & keep) | (g1 & mask);
        h2 = (h2 & keep) | (g2 & mask);
        h3 = (h3 & keep) | (g3 & mask);
        h4 = (h4 & keep) | ((g4 & LIMB) & mask);

        // Only the low 128 bits survive the addition of s, so the fifth limb contributes its
        // low 24 bits and the rest is dropped here rather than by a shift that discards it.
        let acc = (h0 as u128)
            | ((h1 as u128) << 26)
            | ((h2 as u128) << 52)
            | ((h3 as u128) << 78)
            | (((h4 & 0x00ff_ffff) as u128) << 104);
        acc.wrapping_add(self.s).to_le_bytes()
    }
}

// ── The AEAD ────────────────────────────────────────────────────────────────────────────────

/// Sixteen zeros, the most padding §2.8 can ever call for.
const PADDING: [u8; 16] = [0u8; 16];

/// How many zero bytes bring `len` up to a 16-byte boundary.
fn pad16(len: usize) -> usize {
    (16 - (len % 16)) % 16
}

/// Derives the per-message ChaCha20 key and 96-bit nonce from the 192-bit one.
/// draft-irtf-cfrg-xchacha §3.
///
/// The first sixteen nonce bytes go through HChaCha20 into a subkey; the last eight become the
/// low half of a ChaCha20 nonce whose first four bytes are zero. The zeros are not padding for
/// convenience — they are what keeps block counter and nonce in the layout RFC 8439 defines, so
/// that the inner construction is unmodified ChaCha20-Poly1305.
fn derive(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN]) -> ([u8; 32], [u8; 12]) {
    let mut hnonce = [0u8; 16];
    hnonce.copy_from_slice(&nonce[..16]);
    let subkey = hchacha20(key, &hnonce);

    let mut inner = [0u8; 12];
    inner[4..].copy_from_slice(&nonce[16..]);
    (subkey, inner)
}

/// Computes the tag over the §2.8 MAC input, given an already-derived inner key and nonce.
///
/// The length trailer is the part that is easy to leave out and impossible to notice without it:
/// without `le64(|aad|) ‖ le64(|ct|)` an attacker can move bytes across the boundary between
/// associated data and ciphertext and produce the same tag.
fn tag(key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], ciphertext: &[u8]) -> [u8; TAG_LEN] {
    let first = block(key, 0, nonce);
    let mut one_time = [0u8; 32];
    one_time.copy_from_slice(&first[..32]);

    let mut mac = Poly1305::new(&one_time);
    mac.update(aad);
    mac.update(&PADDING[..pad16(aad.len())]);
    mac.update(ciphertext);
    mac.update(&PADDING[..pad16(ciphertext.len())]);
    mac.update(&(aad.len() as u64).to_le_bytes());
    mac.update(&(ciphertext.len() as u64).to_le_bytes());
    mac.finalize()
}

/// Compares two tags in time that does not depend on where they first differ.
///
/// # Why this is not `==`
///
/// The obvious comparison returns as soon as it finds a mismatched byte. An attacker who can
/// time [`open`] can then submit forged tags and learn how many leading bytes were right, one
/// byte at a time — 16 · 256 queries instead of 2^128, which is the difference between
/// infeasible and an afternoon. So every byte is examined, the differences are folded into a
/// single accumulator, and exactly one comparison happens at the end. There is no early return
/// in this function and there must not be one added.
fn tags_equal(a: &[u8; TAG_LEN], b: &[u8; TAG_LEN]) -> bool {
    let mut difference = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        difference |= x ^ y;
    }
    difference == 0
}

/// Encrypts and authenticates, returning `ciphertext ‖ tag`.
///
/// # The nonce is the caller's obligation
///
/// A nonce repeated under one key with two different plaintexts loses the confidentiality of
/// both — the keystreams are identical, so their XOR is the XOR of the plaintexts — and, worse,
/// leaks the Poly1305 key and with it the ability to forge. 192 bits is wide enough that drawing
/// one at random per message is safe, and that is the intended use. Deriving one from a counter
/// that a crash could rewind is not.
///
/// This deliberately does not encrypt in place. The ledger's payloads are moved between owners
/// rather than mutated, and an in-place variant would be a second entry point for a replacement
/// implementation to match.
pub fn seal(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let (subkey, inner) = derive(key, nonce);

    let mut out = Vec::with_capacity(plaintext.len() + TAG_LEN);
    out.extend_from_slice(plaintext);
    // Counter 1: block zero was spent on the Poly1305 key and must never also produce keystream.
    keystream_xor(&subkey, 1, &inner, &mut out);

    let mac = tag(&subkey, &inner, aad, &out);
    out.extend_from_slice(&mac);
    out
}

/// Verifies and decrypts `ciphertext ‖ tag`, returning `None` if anything at all fails.
///
/// # Why the tag is checked before a single byte is decrypted
///
/// Returning plaintext that has not been authenticated — even partially, even on the way to
/// reporting failure — turns this function into a decryption oracle, which is the standard way
/// an AEAD is broken in practice rather than in theory. So the tag is recomputed over the
/// ciphertext as received, compared in constant time, and only then is the keystream applied.
/// On failure the caller learns exactly one bit: it did not verify. Not which byte, not how far
/// in, and not whether the length, the tag or the associated data was the problem.
pub fn open(
    key: &[u8; KEY_LEN],
    nonce: &[u8; NONCE_LEN],
    aad: &[u8],
    sealed: &[u8],
) -> Option<Vec<u8>> {
    if sealed.len() < TAG_LEN {
        return None;
    }
    let (ciphertext, received) = sealed.split_at(sealed.len() - TAG_LEN);
    let mut received_tag = [0u8; TAG_LEN];
    received_tag.copy_from_slice(received);

    let (subkey, inner) = derive(key, nonce);
    let expected = tag(&subkey, &inner, aad, ciphertext);
    if !tags_equal(&expected, &received_tag) {
        return None;
    }

    let mut plaintext = ciphertext.to_vec();
    keystream_xor(&subkey, 1, &inner, &mut plaintext);
    Some(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lowercase hex, so that a failure prints something that can be pasted next to the RFC.
    fn hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            s.push(char::from_digit((*byte >> 4) as u32, 16).unwrap());
            s.push(char::from_digit((*byte & 0x0f) as u32, 16).unwrap());
        }
        s
    }

    /// Parses the hex the RFCs print, ignoring the whitespace they wrap it in.
    fn unhex(text: &str) -> Vec<u8> {
        let digits: Vec<u32> = text
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_digit(16).expect("hex digit"))
            .collect();
        assert_eq!(
            digits.len() % 2,
            0,
            "hex string has an odd number of digits"
        );
        digits
            .chunks_exact(2)
            .map(|p| (p[0] * 16 + p[1]) as u8)
            .collect()
    }

    /// The 32-byte key `00 01 .. 1f`, which RFC 8439 uses for §2.3.2 and §2.4.2.
    fn counting_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for (i, b) in k.iter_mut().enumerate() {
            *b = i as u8;
        }
        k
    }

    /// RFC 8439 §2.4.2 and §2.8.2 and draft-irtf-cfrg-xchacha A.3.1 all encrypt this.
    const SUNSCREEN: &[u8] = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";

    // ── the published vectors ───────────────────────────────────────────────────────────────
    //
    // These are what make "this is XChaCha20-Poly1305" a checkable claim rather than an
    // assertion. An implementation that passes them computes the same function as every other
    // implementation, which is the property a ciphertext written to the ledger today and read
    // by different code in ten years actually depends on.

    #[test]
    fn the_rfc_8439_block_function_vector_matches() {
        // §2.3.2. Counter 1, so this is the block that would encrypt the first 64 bytes.
        let nonce: [u8; 12] = unhex("000000090000004a00000000").try_into().unwrap();
        let out = block(&counting_key(), 1, &nonce);
        assert_eq!(
            hex(&out),
            "10f1e7e4d13b5915500fdd1fa32071c4\
             c7d1f4c733c068030422aa9ac3d46c4e\
             d2826446079faa0914c2d705d98b02a2\
             b5129cd1de164eb9cbd083e8a2503c4e"
        );
    }

    #[test]
    fn the_rfc_8439_encryption_vector_matches() {
        // §2.4.2. Note the nonce differs from §2.3.2 in its fourth byte.
        let nonce: [u8; 12] = unhex("000000000000004a00000000").try_into().unwrap();
        let mut data = SUNSCREEN.to_vec();
        keystream_xor(&counting_key(), 1, &nonce, &mut data);
        assert_eq!(
            hex(&data),
            "6e2e359a2568f98041ba0728dd0d6981\
             e97e7aec1d4360c20a27afccfd9fae0b\
             f91b65c5524733ab8f593dabcd62b357\
             1639d624e65152ab8f530c359f0861d8\
             07ca0dbf500d6a6156a38e088a22b65e\
             52bc514d16ccf806818ce91ab7793736\
             5af90bbf74a35be6b40b8eedf2785e42\
             874d"
        );
    }

    #[test]
    fn the_rfc_8439_poly1305_vector_matches() {
        // §2.5.2.
        let key = unhex(
            "85d6be7857556d337f4452fe42d506a8\
             0103808afb0db2fd4abff6af4149f51b",
        );
        let mut mac = Poly1305::new(&key.try_into().unwrap());
        mac.update(b"Cryptographic Forum Research Group");
        assert_eq!(hex(&mac.finalize()), "a8061dc1305136c6c22b8baf0c0127a9");
    }

    #[test]
    fn the_rfc_8439_aead_vector_matches() {
        // §2.8.2. This exercises the inner ChaCha20-Poly1305 directly, with a 96-bit nonce,
        // because that is the layer the vector is stated at.
        let key = unhex(
            "808182838485868788898a8b8c8d8e8f\
             909192939495969798999a9b9c9d9e9f",
        );
        let key: [u8; 32] = key.try_into().unwrap();
        let nonce: [u8; 12] = unhex("070000004041424344454647").try_into().unwrap();
        let aad = unhex("50515253c0c1c2c3c4c5c6c7");

        let mut ciphertext = SUNSCREEN.to_vec();
        keystream_xor(&key, 1, &nonce, &mut ciphertext);
        assert_eq!(
            hex(&ciphertext),
            "d31a8d34648e60db7b86afbc53ef7ec2\
             a4aded51296e08fea9e2b5a736ee62d6\
             3dbea45e8ca9671282fafb69da92728b\
             1a71de0a9e060b2905d6a5b67ecd3b36\
             92ddbd7f2d778b8c9803aee328091b58\
             fab324e4fad675945585808b4831d7bc\
             3ff4def08e4b7a9de576d26586cec64b\
             6116"
        );

        let mac = tag(&key, &nonce, &aad, &ciphertext);
        assert_eq!(hex(&mac), "1ae10b594f09e26a7e902ecbd0600691");
    }

    #[test]
    fn the_xchacha_hchacha20_subkey_vector_matches() {
        // draft-irtf-cfrg-xchacha §2.2.1.
        let nonce: [u8; 16] = unhex("000000090000004a0000000031415927")
            .try_into()
            .unwrap();
        let subkey = hchacha20(&counting_key(), &nonce);
        assert_eq!(
            hex(&subkey),
            "82413b4227b27bfed30e42508a877d73a0f9e4d58a74a853c12ec41326d3ecdc"
        );
    }

    #[test]
    fn the_xchacha_aead_vector_matches() {
        // draft-irtf-cfrg-xchacha A.3.1, the end-to-end vector for the construction this module
        // actually exports. It is the one vector here that is a draft rather than a published
        // RFC, so it is worth naming the source precisely: draft-irtf-cfrg-xchacha-03,
        // Appendix A.3.1, "Example and Test Vector for AEAD_XCHACHA20_POLY1305".
        let key = unhex(
            "808182838485868788898a8b8c8d8e8f\
             909192939495969798999a9b9c9d9e9f",
        );
        let key: [u8; 32] = key.try_into().unwrap();
        let nonce: [u8; 24] = unhex("404142434445464748494a4b4c4d4e4f5051525354555657")
            .try_into()
            .unwrap();
        let aad = unhex("50515253c0c1c2c3c4c5c6c7");

        let sealed = seal(&key, &nonce, &aad, SUNSCREEN);
        let (ciphertext, mac) = sealed.split_at(sealed.len() - TAG_LEN);
        assert_eq!(
            hex(ciphertext),
            "bd6d179d3e83d43b9576579493c0e939\
             572a1700252bfaccbed2902c21396cbb\
             731c7f1b0b4aa6440bf3a82f4eda7e39\
             ae64c6708c54c216cb96b72e1213b452\
             2f8c9ba40db5d945b11b69b982c1bb9e\
             3f3fac2bc369488f76b2383565d3fff9\
             21f9664c97637da9768812f615c68b13\
             b52e"
        );
        assert_eq!(hex(mac), "c0875924c1c7987947deafd8780acf49");

        // And the same vector must come back out, which is the half a vector cannot fake.
        assert_eq!(open(&key, &nonce, &aad, &sealed).unwrap(), SUNSCREEN);
    }

    // ── properties ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn a_sealed_message_opens_back_to_itself() {
        let key = [0x2au8; KEY_LEN];
        let nonce = [0x7fu8; NONCE_LEN];

        // The empty cases are where the padding arithmetic goes wrong: pad16(0) must be 0 and
        // not 16, or the MAC covers a block the other side does not.
        for (aad, plaintext) in [
            (&b""[..], &b""[..]),
            (&b""[..], &b"one byte short of nothing"[..]),
            (&b"header"[..], &b""[..]),
            (&b"header"[..], &b"exactly sixteen!"[..]),
        ] {
            let sealed = seal(&key, &nonce, aad, plaintext);
            assert_eq!(sealed.len(), plaintext.len() + TAG_LEN);
            assert_eq!(open(&key, &nonce, aad, &sealed).as_deref(), Some(plaintext));
        }

        // Several keystream blocks, so a counter that fails to advance shows up.
        let long: Vec<u8> = (0..1_000u32).map(|i| (i % 251) as u8).collect();
        let aad: Vec<u8> = (0..97u8).collect();
        let sealed = seal(&key, &nonce, &aad, &long);
        assert_eq!(open(&key, &nonce, &aad, &sealed).unwrap(), long);
    }

    #[test]
    fn a_flipped_bit_anywhere_makes_open_return_none() {
        // Every bit of a short message rather than one arbitrary bit, because a MAC that misses
        // a region misses all of it and a single spot-check is as likely to land outside the
        // hole as in it.
        let key = [0x11u8; KEY_LEN];
        let nonce = [0x22u8; NONCE_LEN];
        let aad = b"account:0001";
        let plaintext = b"debit 250";

        let sealed = seal(&key, &nonce, aad, plaintext);

        // Ciphertext bits and tag bits alike: `sealed` is both, and neither may be malleable.
        for byte in 0..sealed.len() {
            for bit in 0..8 {
                let mut tampered = sealed.clone();
                tampered[byte] ^= 1 << bit;
                assert!(
                    open(&key, &nonce, aad, &tampered).is_none(),
                    "flipping bit {bit} of sealed byte {byte} was accepted"
                );
            }
        }

        // The associated data is authenticated but not encrypted, which is exactly the case
        // where an implementation can look right and cover nothing.
        for byte in 0..aad.len() {
            for bit in 0..8 {
                let mut tampered = aad.to_vec();
                tampered[byte] ^= 1 << bit;
                assert!(
                    open(&key, &nonce, &tampered, &sealed).is_none(),
                    "flipping bit {bit} of aad byte {byte} was accepted"
                );
            }
        }
    }

    #[test]
    fn a_wrong_key_or_nonce_makes_open_return_none() {
        let key = [0x11u8; KEY_LEN];
        let nonce = [0x22u8; NONCE_LEN];
        let sealed = seal(&key, &nonce, b"aad", b"payload");

        let mut other_key = key;
        other_key[31] ^= 1;
        assert!(open(&other_key, &nonce, b"aad", &sealed).is_none());

        // Both halves of the nonce matter: the first sixteen bytes only through HChaCha20, the
        // last eight only through the inner nonce.
        let mut early = nonce;
        early[0] ^= 1;
        assert!(open(&key, &early, b"aad", &sealed).is_none());
        let mut late = nonce;
        late[23] ^= 1;
        assert!(open(&key, &late, b"aad", &sealed).is_none());
    }

    #[test]
    fn a_message_shorter_than_a_tag_makes_open_return_none() {
        // The length check has to come first; without it the split panics, and a panic reachable
        // from a wire-supplied length is a denial of service.
        let key = [0u8; KEY_LEN];
        let nonce = [0u8; NONCE_LEN];
        for len in 0..TAG_LEN {
            assert!(open(&key, &nonce, b"", &vec![0u8; len]).is_none());
        }
    }

    #[test]
    fn tag_comparison_reports_equality_without_an_early_return() {
        // The constant-time property itself is not testable from here; what is testable is that
        // the fold produces the right answer, including when only the last byte differs, which
        // is the case an early return would get right and a broken fold would not.
        let a = [0u8; TAG_LEN];
        assert!(tags_equal(&a, &a));

        for i in 0..TAG_LEN {
            let mut b = a;
            b[i] = 1;
            assert!(!tags_equal(&a, &b));
        }
    }
}
