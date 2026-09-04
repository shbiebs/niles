//! **Randomness that is either cryptographic or an error — never a fallback.**
//!
//! [`crate::aead`] takes a nonce and does not choose one, deliberately: the caller has to. This
//! module is what a caller uses, and the whole of its design is one refusal.
//!
//! A nonce repeated under one key is the failure most likely to actually happen and the worst
//! one when it does. Under ChaCha20-Poly1305 a repeat does not merely leak the relationship
//! between two messages: it exposes the Poly1305 one-time key, and an attacker who has that can
//! forge tags for messages that were never written. So the only two acceptable outcomes of
//! asking for a nonce are a nonce from the operating system's CSPRNG and an error. There is no
//! third arm. In particular there is no arm that mixes a clock, a process id and an address
//! into something that looks random — that pattern is how systems end up with 2⁴⁰ of entropy
//! in a 192-bit field, and it never announces itself.
//!
//! # Why `/dev/urandom` rather than `getrandom(2)`
//!
//! The syscall would be better: it cannot fail on a missing device node and it cannot be
//! defeated by a chroot without `/dev`. Reaching it needs `unsafe`, and this repository has
//! none outside its measurement tool. Reading the device is what std can do, and the failure
//! modes it adds are ones this module turns into errors rather than into weak keys.
//!
//! Two properties are checked rather than assumed. The read must return **exactly** as many
//! bytes as were asked for — a short read from a character device is not supposed to happen and
//! is an error here rather than a partially-filled buffer. And the bytes must not be all zero:
//! that is not a randomness test, it is a wiring test, and it catches the one failure a stubbed
//! or mis-opened source actually produces.

use std::io::Read;

/// Where the bytes come from. Named so an error can say it.
const SOURCE: &str = "/dev/urandom";

/// Why randomness could not be obtained. There is no variant meaning "so we used something
/// else".
#[derive(Debug)]
pub enum RandomError {
    /// The source could not be opened or read.
    Unavailable(std::io::Error),
    /// The source returned fewer bytes than were asked for.
    Short { asked: usize, got: usize },
    /// The source returned all zeroes, which no working CSPRNG does at this length and every
    /// broken one does.
    AllZero(usize),
}

impl std::fmt::Display for RandomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RandomError::Unavailable(e) => {
                write!(
                    f,
                    "{SOURCE} could not be read ({e}); refusing to invent a nonce"
                )
            }
            RandomError::Short { asked, got } => write!(
                f,
                "{SOURCE} returned {got} bytes of {asked}; a short read from the system CSPRNG \
                 is a broken source, not a smaller amount of randomness"
            ),
            RandomError::AllZero(n) => write!(
                f,
                "{SOURCE} returned {n} zero bytes; the source is stubbed or mis-opened"
            ),
        }
    }
}

impl std::error::Error for RandomError {}

/// Fill `out` from the system CSPRNG, or fail.
pub fn fill(out: &mut [u8]) -> Result<(), RandomError> {
    if out.is_empty() {
        return Ok(());
    }
    let mut f = std::fs::File::open(SOURCE).map_err(RandomError::Unavailable)?;
    f.read_exact(out).map_err(|e| match e.kind() {
        std::io::ErrorKind::UnexpectedEof => RandomError::Short {
            asked: out.len(),
            got: 0,
        },
        _ => RandomError::Unavailable(e),
    })?;
    if out.iter().all(|b| *b == 0) {
        return Err(RandomError::AllZero(out.len()));
    }
    Ok(())
}

/// `N` fresh bytes, or an error.
pub fn bytes<const N: usize>() -> Result<[u8; N], RandomError> {
    let mut b = [0u8; N];
    fill(&mut b)?;
    Ok(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_returns_the_length_asked_for() {
        let a: [u8; 24] = bytes().expect("the system CSPRNG is readable");
        assert_eq!(a.len(), 24);
    }

    #[test]
    fn two_requests_do_not_return_the_same_bytes() {
        // Not a randomness test — a *wiring* test. A source that returns a constant, a
        // fixed buffer, or the same page twice is the realistic failure, and it is
        // indistinguishable from working randomness by every other check here.
        let a: [u8; 32] = bytes().expect("readable");
        let b: [u8; 32] = bytes().expect("readable");
        assert_ne!(
            a, b,
            "two draws from the CSPRNG were identical; the source is not a CSPRNG"
        );
    }

    #[test]
    fn every_byte_position_eventually_varies() {
        // A source wired through a buffer that is only partly filled looks fine at the front
        // and is constant at the back. Two hundred draws over 24 bytes make a position that
        // never changes overwhelmingly unlikely to be chance.
        let first: [u8; 24] = bytes().expect("readable");
        let mut varied = [false; 24];
        for _ in 0..200 {
            let n: [u8; 24] = bytes().expect("readable");
            for i in 0..24 {
                varied[i] |= n[i] != first[i];
            }
        }
        for (i, v) in varied.iter().enumerate() {
            assert!(*v, "byte {i} never changed across 200 draws");
        }
    }

    #[test]
    fn an_empty_request_is_not_an_error() {
        // Because a caller sealing a value with no associated data should not have to
        // special-case the length, and because an empty slice is trivially not all-zero.
        assert!(fill(&mut []).is_ok());
    }

    #[test]
    fn the_error_says_what_was_wrong_and_never_offers_a_substitute() {
        let e = RandomError::Short { asked: 24, got: 3 };
        let s = e.to_string();
        assert!(s.contains("24") && s.contains('3'), "{s}");
        // The property that matters is what the type cannot express: there is no variant
        // meaning "fell back to a weaker source", so no caller can be handed one by accident.
        let e = RandomError::AllZero(24);
        assert!(e.to_string().contains("stubbed"), "{e}");
    }
}
