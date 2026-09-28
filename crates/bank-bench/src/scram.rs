//! SCRAM-SHA-256 client authentication (RFC 5802, RFC 7677), in `std` plus the ledger's own
//! SHA-256.
//!
//! # Why this exists
//!
//! Until cycle 14 the harness spoke only `trust` authentication and refused everything else.
//! That was a deliberate scope limit, and it had a cost nobody wrote down: a stock PostgreSQL
//! 16 authenticates TCP connections from 127.0.0.1 with `scram-sha-256`, so every fresh
//! machine had to have its `pg_hba.conf` weakened before `make gate` could pass — a
//! precondition that lived in the heads of whoever had provisioned the last container and in
//! no file in either repository. Cycle 14's audit found the gate failing on a fresh
//! container for exactly that reason (decision D-1). The remedy is not to weaken the server;
//! it is to let the client authenticate properly.
//!
//! # What is and is not here
//!
//! HMAC-SHA-256 (RFC 2104), PBKDF2-HMAC-SHA-256 (RFC 8018 §5.2), the base64 alphabet of RFC
//! 4648 §4, and the SCRAM exchange without channel binding (`n,,` — the harness has no TLS,
//! so there is no channel to bind). The server's final signature is **verified**: a client
//! that accepted any `v=` would authenticate to an impostor that simply said "ok".
//!
//! SHA-256 is the ledger's implementation (`nilestream_ledger::chain`, ADR 0003), which is
//! already held to the NIST vectors; this module adds no second hash.

use nilestream_ledger::chain::{sha256, Hasher256};

const BLOCK: usize = 64;

/// HMAC-SHA-256 of `msg` under `key` (RFC 2104).
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; BLOCK];
    if key.len() > BLOCK {
        k[..32].copy_from_slice(&sha256(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Hasher256::new();
    inner.update(&ipad).update(msg);
    let inner = inner.finalize();
    let mut outer = Hasher256::new();
    outer.update(&opad).update(&inner);
    outer.finalize()
}

/// PBKDF2-HMAC-SHA-256 (RFC 8018 §5.2), writing `out.len()` bytes of derived key.
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32, out: &mut [u8]) {
    assert!(iterations >= 1, "PBKDF2 needs at least one iteration");
    for (block_index, chunk) in out.chunks_mut(32).enumerate() {
        let mut first = Vec::with_capacity(salt.len() + 4);
        first.extend_from_slice(salt);
        first.extend_from_slice(&((block_index as u32) + 1).to_be_bytes());
        let mut u = hmac_sha256(password, &first);
        let mut t = u;
        for _ in 1..iterations {
            u = hmac_sha256(password, &u);
            for (a, b) in t.iter_mut().zip(u.iter()) {
                *a ^= b;
            }
        }
        chunk.copy_from_slice(&t[..chunk.len()]);
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding (RFC 4648 §4).
pub fn b64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = match chunk.len() {
            3 => (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8 | chunk[2] as u32,
            2 => (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8,
            _ => (chunk[0] as u32) << 16,
        };
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Standard base64 decoding; `None` on any character outside the alphabet or a bad length.
pub fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.as_bytes();
    if !s.len().is_multiple_of(4) {
        return None;
    }
    let val = |c: u8| -> Option<u32> { B64.iter().position(|&b| b == c).map(|p| p as u32) };
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    for quad in s.chunks(4) {
        let pad = quad.iter().rev().take_while(|&&c| c == b'=').count();
        if pad > 2 || quad[..4 - pad].contains(&b'=') {
            return None;
        }
        let mut n = 0u32;
        for (i, &c) in quad.iter().enumerate() {
            let v = if i >= 4 - pad { 0 } else { val(c)? };
            n |= v << (18 - 6 * i);
        }
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    Some(out)
}

/// Why a SCRAM exchange was refused. Each variant is a different operator action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScramError {
    /// No password was available — `PGPASSWORD` unset and no password file.
    NoPassword(String),
    /// The server's first message was not what RFC 5802 §7 says it is.
    Malformed(String),
    /// The server's nonce does not begin with ours: a replayed or forged exchange.
    NonceMismatch,
    /// The server's final signature is wrong: it does not know the password's verifier.
    ServerSignature,
}

/// One client-side SCRAM-SHA-256 exchange. Three steps, each consuming the previous.
pub struct Scram {
    password: Vec<u8>,
    client_nonce: String,
    client_first_bare: String,
}

/// The state after the client's final message, holding what verifies the server.
pub struct AwaitingServerFinal {
    expected_server_signature: [u8; 32],
}

impl Scram {
    /// Start an exchange. The user name is sent empty: PostgreSQL authenticates the user named
    /// in the startup packet and ignores the SCRAM `n=` attribute (protocol docs, §55.3.1).
    pub fn new(password: &str, client_nonce: String) -> Scram {
        Scram {
            password: password.as_bytes().to_vec(),
            client_first_bare: format!("n=,r={client_nonce}"),
            client_nonce,
        }
    }

    /// For the RFC 7677 vector, whose client-first message names its user.
    #[cfg(test)]
    fn with_user(user: &str, password: &str, client_nonce: &str) -> Scram {
        Scram {
            password: password.as_bytes().to_vec(),
            client_first_bare: format!("n={user},r={client_nonce}"),
            client_nonce: client_nonce.to_string(),
        }
    }

    /// `client-first-message`: the GS2 header (`n,,` — no channel binding) and the bare part.
    pub fn client_first(&self) -> String {
        format!("n,,{}", self.client_first_bare)
    }

    /// Consume `server-first-message`, produce `client-final-message`.
    pub fn client_final(
        self,
        server_first: &str,
    ) -> Result<(String, AwaitingServerFinal), ScramError> {
        let mut nonce = None;
        let mut salt = None;
        let mut iterations = None;
        for attr in server_first.split(',') {
            match attr.split_once('=') {
                Some(("r", v)) => nonce = Some(v),
                Some(("s", v)) => salt = Some(v),
                Some(("i", v)) => iterations = Some(v),
                _ => {}
            }
        }
        let (Some(nonce), Some(salt), Some(iterations)) = (nonce, salt, iterations) else {
            return Err(ScramError::Malformed(server_first.to_string()));
        };
        if !nonce.starts_with(&self.client_nonce) || nonce.len() == self.client_nonce.len() {
            return Err(ScramError::NonceMismatch);
        }
        let salt = b64_decode(salt).ok_or_else(|| ScramError::Malformed("salt".into()))?;
        let iterations: u32 = iterations
            .parse()
            .ok()
            .filter(|&i| i >= 1)
            .ok_or_else(|| ScramError::Malformed("iteration count".into()))?;

        let mut salted = [0u8; 32];
        pbkdf2_hmac_sha256(&self.password, &salt, iterations, &mut salted);
        let client_key = hmac_sha256(&salted, b"Client Key");
        let stored_key = sha256(&client_key);
        let server_key = hmac_sha256(&salted, b"Server Key");

        let without_proof = format!("c=biws,r={nonce}");
        let auth_message = format!("{},{server_first},{without_proof}", self.client_first_bare);
        let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());
        let mut proof = client_key;
        for (p, s) in proof.iter_mut().zip(client_signature.iter()) {
            *p ^= s;
        }
        let final_msg = format!("{without_proof},p={}", b64_encode(&proof));
        let expected = hmac_sha256(&server_key, auth_message.as_bytes());
        Ok((
            final_msg,
            AwaitingServerFinal {
                expected_server_signature: expected,
            },
        ))
    }
}

impl AwaitingServerFinal {
    /// Verify `server-final-message`. An `e=` error from the server is reported as malformed
    /// with its text, since it names the reason (`invalid-proof`, …).
    pub fn verify(self, server_final: &str) -> Result<(), ScramError> {
        if let Some(e) = server_final.strip_prefix("e=") {
            return Err(ScramError::Malformed(format!("server error: {e}")));
        }
        let v = server_final
            .strip_prefix("v=")
            .ok_or_else(|| ScramError::Malformed(server_final.to_string()))?;
        let got = b64_decode(v).ok_or_else(|| ScramError::Malformed("signature".into()))?;
        if got.as_slice() == self.expected_server_signature {
            Ok(())
        } else {
            Err(ScramError::ServerSignature)
        }
    }
}

/// A client nonce: 18 bytes from the operating system's generator, base64-encoded (24
/// printable characters, none of them `,`). Reading `/dev/urandom` keeps the workspace free
/// of dependencies; a platform without it gets a named error rather than a weak nonce.
pub fn fresh_nonce() -> Result<String, ScramError> {
    use std::io::Read;
    let mut bytes = [0u8; 18];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| ScramError::Malformed(format!("no random source for the nonce: {e}")))?;
    Ok(b64_encode(&bytes))
}

/// Where the harness finds the password for its PostgreSQL role, in order:
/// `PGPASSWORD`; the file named by `NILES_PGPASSFILE`; `.pg-bench-password` at the workspace
/// root, which `tools/pg-provision.sh` writes and `.gitignore` keeps out of every commit.
/// The password is never logged — not here, not in an error.
pub fn password() -> Result<String, ScramError> {
    if let Ok(p) = std::env::var("PGPASSWORD") {
        if !p.is_empty() {
            return Ok(p);
        }
    }
    let path = std::env::var("NILES_PGPASSFILE").unwrap_or_else(|_| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../.pg-bench-password").to_string()
    });
    match std::fs::read_to_string(&path) {
        Ok(s) if !s.trim().is_empty() => Ok(s.trim().to_string()),
        _ => Err(ScramError::NoPassword(format!(
            "the server asked for a password and none was found: set PGPASSWORD, or run \
             tools/pg-provision.sh, which writes {path}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    // RFC 4231 §4.2 and §4.3 — test cases 1 and 2. §4.7 (case 6) exercises a key longer than
    // the block, which is the branch that hashes the key first.
    #[test]
    fn hmac_sha256_matches_rfc_4231() {
        assert_eq!(
            hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        assert_eq!(
            hex(&hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    // RFC 7914 §11 gives PBKDF2-HMAC-SHA-256 vectors; the first is cheap enough to run always.
    #[test]
    fn pbkdf2_hmac_sha256_matches_rfc_7914() {
        let mut out = [0u8; 64];
        pbkdf2_hmac_sha256(b"passwd", b"salt", 1, &mut out);
        assert_eq!(
            hex(&out),
            "55ac046e56e3089fec1691c22544b605f94185216dde0465e68b9d57c20dacbc\
             49ca9cccf179b645991664b39d77ef317c71b845b1e30bd509112041d3a19783"
        );
    }

    #[test]
    fn pbkdf2_hmac_sha256_matches_rfc_7914_at_80000_iterations() {
        let mut out = [0u8; 64];
        pbkdf2_hmac_sha256(b"Password", b"NaCl", 80_000, &mut out);
        assert_eq!(
            hex(&out),
            "4ddcd8f60b98be21830cee5ef22701f9641a4418d04c0414aeff08876b34ab56\
             a1d425a1225833549adb841b51c9b3176a272bdebba1d078478f62b397f33c8d"
        );
    }

    #[test]
    fn base64_round_trips_and_refuses_what_is_not_base64() {
        for input in [&b""[..], b"f", b"fo", b"foo", b"foob", b"fooba", b"foobar"] {
            let e = b64_encode(input);
            assert_eq!(b64_decode(&e).as_deref(), Some(input), "{e}");
        }
        assert_eq!(b64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(b64_encode(b"fo"), "Zm8=");
        assert_eq!(b64_decode("Zm9v!mFy"), None);
        assert_eq!(b64_decode("Zm9"), None);
        assert_eq!(b64_decode("Z=9v"), None);
    }

    // RFC 7677 §3: the whole SCRAM-SHA-256 exchange for user "user", password "pencil".
    #[test]
    fn the_exchange_reproduces_rfc_7677() {
        let s = Scram::with_user("user", "pencil", "rOprNGfwEbeRWgbNEkqO");
        assert_eq!(s.client_first(), "n,,n=user,r=rOprNGfwEbeRWgbNEkqO");
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
                            s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let (client_final, awaiting) = s.client_final(server_first).expect("server-first");
        assert_eq!(
            client_final,
            "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
             p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
        );
        awaiting
            .verify("v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=")
            .expect("the RFC's server signature verifies");
    }

    #[test]
    fn a_wrong_server_signature_is_refused() {
        let s = Scram::with_user("user", "pencil", "rOprNGfwEbeRWgbNEkqO");
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
                            s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let (_, awaiting) = s.client_final(server_first).unwrap();
        assert_eq!(
            awaiting.verify("v=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
            Err(ScramError::ServerSignature)
        );
    }

    #[test]
    fn a_server_nonce_that_does_not_extend_ours_is_refused() {
        let s = Scram::new("pencil", "abc".into());
        assert_eq!(
            s.client_final("r=xyzabc,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
                .err(),
            Some(ScramError::NonceMismatch)
        );
        let s = Scram::new("pencil", "abc".into());
        assert_eq!(
            s.client_final("r=abc,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
                .err(),
            Some(ScramError::NonceMismatch),
            "a server that adds nothing to the nonce contributed no freshness"
        );
    }

    #[test]
    fn a_fresh_nonce_is_printable_and_never_repeats() {
        let a = fresh_nonce().unwrap();
        let b = fresh_nonce().unwrap();
        assert_eq!(a.len(), 24);
        assert!(!a.contains(','));
        assert_ne!(a, b);
    }
}
