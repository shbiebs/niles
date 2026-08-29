//! Hash chaining: h_e = H(h_{e-1} || canon(delta_e)) (thesis Def. 3.1).
//!
//! PLACEHOLDER: `Hasher256` is a deterministic, NON-cryptographic stand-in
//! mirroring blake3's API so the workspace builds offline (see ADR 0002).
//! Phase 0 swaps this for the `blake3` crate; the API is drop-in.

/// 256-bit FNV-1a-style placeholder hasher. NOT collision resistant.
pub struct Hasher256 {
    state: [u64; 4],
}

impl Default for Hasher256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher256 {
    pub fn new() -> Self {
        Self { state: [0xcbf29ce484222325, 0x9e3779b97f4a7c15, 0x517cc1b727220a95, 0x2545f4914f6cdd1d] }
    }

    pub fn update(&mut self, bytes: &[u8]) -> &mut Self {
        for (i, b) in bytes.iter().enumerate() {
            let lane = i & 3;
            self.state[lane] ^= *b as u64;
            self.state[lane] = self.state[lane].wrapping_mul(0x100000001b3);
            self.state[(lane + 1) & 3] = self.state[(lane + 1) & 3].rotate_left(13) ^ self.state[lane];
        }
        self
    }

    pub fn finalize(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, s) in self.state.iter().enumerate() {
            // one final avalanche round per lane
            let mut x = *s;
            x ^= x >> 33;
            x = x.wrapping_mul(0xff51afd7ed558ccd);
            x ^= x >> 33;
            out[i * 8..(i + 1) * 8].copy_from_slice(&x.to_le_bytes());
        }
        out
    }
}

/// Chain one canonicalized epoch body onto a parent hash.
pub fn chain_hash(parent: &[u8; 32], canonical_body: &[u8]) -> [u8; 32] {
    let mut h = Hasher256::new();
    h.update(parent);
    h.update(canonical_body);
    h.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
