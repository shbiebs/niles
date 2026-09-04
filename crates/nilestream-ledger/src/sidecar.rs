//! **The detachable store: where an erasable byte lives, and how it stops existing.**
//!
//! # The rule this file exists to make true
//!
//! *The chained, scanned record contains no erasable byte.* A ledger is append-only and
//! hash-chained because that is what makes it evidence; a data subject has a right to have
//! their personal data destroyed. Those two facts are usually presented as a contradiction, and
//! the usual resolutions are both bad: re-chain the history (which makes a legitimate change
//! indistinguishable from tampering, and costs O(history)), or decline to erase.
//!
//! There is a third arrangement and it is the one here. A value that may ever have to be erased
//! never goes on the chain at all. What goes on the chain is a **commitment** — `H(salt ‖
//! value)` with a fresh 32-byte salt — and one state byte. The value itself, and its salt, live
//! in this file, encrypted. Erasure then destroys the bytes and the key and leaves the chain
//! untouched, because the chain never held them: `h_e = H(h_{e-1} ‖ structural bytes ‖
//! commitments)` has the same inputs before and after, so the head hash is the same byte string
//! and every verification still passes. The commitment that remains is, without the salt and
//! without the value, useless — which is the reasoning the EDPB's guidance on pseudonymisation
//! sets out at para. 53.
//!
//! # Why zero *and* compact, and why the key shred is separate
//!
//! Zeroing the entry in place and `fsync`ing makes the bytes gone from the file as the operating
//! system presents it. That is necessary and it is not sufficient: the file still has a hole the
//! size of the value, which leaks its length, and on a copy-on-write filesystem or a
//! log-structured flash controller the old block may still be readable by someone who can get
//! underneath the filesystem. So [`Sidecar::compact`] rewrites the file without the tombstoned
//! entries, which removes the hole and the length with it.
//!
//! Even that does not reach every physical cell, and this module does not claim it does. The
//! claim is narrower and it is the honest one: **what makes a copy we do not own useless is the
//! key shred**, which is a separate step in a separate store. Zero-and-compact makes the bytes
//! gone from the files we own; shredding the key makes any stale copy ciphertext with no key.
//! Neither alone is a purge on modern media. Together they are what NIST SP 800-88 calls one.
//!
//! # The layout
//!
//! ```text
//! file    := header | entry*
//! header  := magic "NLSIDE01" (8 bytes)
//! entry   := u8 state | u32 len | commitment [u8;32] | key_id [u8;16]
//!          | nonce [u8;24] | ciphertext [u8; len] | tag [u8;16]
//! state   := 1 present | 0 tombstoned
//! ```
//!
//! `len` is the length of the ciphertext, which is the length of the plaintext: a stream cipher
//! does not pad. That is a leak of the value's length and it is stated rather than hidden — a
//! narrative of four characters is distinguishable from one of four hundred while the entry is
//! present. It stops being distinguishable after compaction, because the entry is gone.
//!
//! The salt is **inside** the ciphertext, not beside it. A salt in the clear beside a commitment
//! would let anyone holding the file confirm a guessed value by recomputing `H(salt ‖ guess)`,
//! which is precisely the property the commitment is supposed not to have once the value is
//! erased.
//!
//! # What the chain must not do
//!
//! Nothing in this file is ever hashed into the chain, and `a_sidecar_byte_flip_does_not_break_
//! the_chain` is the test that says so. If it were, erasure would break verification and the
//! whole arrangement would collapse back into the contradiction it exists to resolve.

use crate::aead;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Identifies the file's format. A version byte in the magic rather than a separate field, so a
/// file from another format is rejected at the first read rather than misparsed.
const MAGIC: &[u8; 8] = b"NLSIDE01";

/// A 32-byte commitment, `H(salt ‖ value)`. What the chain carries in place of a value.
pub type Commitment = [u8; 32];

/// Which key sealed an entry. Opaque here; the key store gives it meaning.
pub type KeyId = [u8; 16];

const STATE_PRESENT: u8 = 1;
const STATE_TOMBSTONED: u8 = 0;
const SALT_LEN: usize = 32;

/// The fixed overhead of one entry: state, length, commitment, key id, nonce, tag.
const ENTRY_OVERHEAD: usize = 1 + 4 + 32 + 16 + aead::NONCE_LEN + aead::TAG_LEN;

/// What went wrong. No variant means "carried on anyway".
#[derive(Debug)]
pub enum SidecarError {
    Io(std::io::Error),
    Random(crate::random::RandomError),
    /// The file does not begin with this format's magic.
    NotASidecar,
    /// A record's length prefix runs past the end of the file. Truncation or corruption.
    Truncated {
        at: u64,
    },
    /// Asked to zero a commitment the file does not hold.
    NoSuchCommitment(Commitment),
    /// The entry is present but its tag did not verify under the key supplied.
    ///
    /// Deliberately not merged with "no such entry": a caller that cannot tell "you gave me the
    /// wrong key" from "there is nothing here" cannot tell a key-store bug from an erasure.
    NotAuthentic(Commitment),
}

impl std::fmt::Display for SidecarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SidecarError::Io(e) => write!(f, "{e}"),
            SidecarError::Random(e) => write!(f, "{e}"),
            SidecarError::NotASidecar => write!(f, "not a sidecar file (bad magic)"),
            SidecarError::Truncated { at } => {
                write!(f, "an entry at offset {at} runs past the end of the file")
            }
            SidecarError::NoSuchCommitment(c) => {
                write!(f, "no entry for commitment {}", hex(c))
            }
            SidecarError::NotAuthentic(c) => write!(
                f,
                "the entry for commitment {} did not authenticate under the key supplied",
                hex(c)
            ),
        }
    }
}

impl std::error::Error for SidecarError {}

impl From<std::io::Error> for SidecarError {
    fn from(e: std::io::Error) -> Self {
        SidecarError::Io(e)
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The commitment of a value under a salt: `SHA-256(salt ‖ value)`.
///
/// The salt is what makes the commitment hiding. Without it a commitment over a low-entropy
/// value — an account name, a short narrative, a yes/no — is a dictionary lookup, and an erased
/// field whose value can be recovered by guessing was not erased.
pub fn commit(salt: &[u8; SALT_LEN], value: &[u8]) -> Commitment {
    let mut h = crate::chain::Hasher256::new();
    h.update(salt);
    h.update(value);
    h.finalize()
}

/// One entry as it sits in the file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    state: u8,
    commitment: Commitment,
    key_id: KeyId,
    nonce: [u8; aead::NONCE_LEN],
    /// Ciphertext of `salt ‖ value`, followed by the tag.
    sealed: Vec<u8>,
}

impl Entry {
    fn encoded_len(&self) -> usize {
        // `sealed` already carries the tag, so the overhead here excludes it.
        ENTRY_OVERHEAD - aead::TAG_LEN + self.sealed.len()
    }

    fn write_to(&self, out: &mut Vec<u8>) {
        out.push(self.state);
        // The length is of the sealed bytes including the tag, so a reader needs no separate
        // knowledge of the tag length to skip an entry it does not care about.
        out.extend_from_slice(&(self.sealed.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.commitment);
        out.extend_from_slice(&self.key_id);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.sealed);
    }
}

/// A per-segment detachable store.
///
/// Per segment rather than per subject, and the reason is operational rather than
/// architectural: one file per party is millions of inodes and a directory scan per erasure,
/// where one file per segment is a bounded number of files and a linear rewrite of one of them.
pub struct Sidecar {
    path: PathBuf,
}

impl Sidecar {
    /// The sidecar beside a segment: `<segment>.side`.
    pub fn beside(segment: &Path) -> Sidecar {
        let mut p = segment.as_os_str().to_owned();
        p.push(".side");
        Sidecar {
            path: PathBuf::from(p),
        }
    }

    pub fn at(path: impl Into<PathBuf>) -> Sidecar {
        Sidecar { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether the file exists. A sidecar that was never written and one that was deleted are
    /// the same thing to a reader, which is what makes recovery-without-the-sidecar work.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Seal a value into the store and return the commitment the chain will carry.
    ///
    /// The salt is fresh per value and is sealed *with* the value, so the file holds nothing
    /// that helps anyone confirm a guess. The associated data binds the ciphertext to the
    /// commitment: an entry moved to another commitment's slot fails to authenticate rather
    /// than decrypting to the wrong field's value.
    pub fn put(
        &self,
        key: &[u8; aead::KEY_LEN],
        key_id: KeyId,
        value: &[u8],
    ) -> Result<Commitment, SidecarError> {
        let salt: [u8; SALT_LEN] = crate::random::bytes().map_err(SidecarError::Random)?;
        let nonce: [u8; aead::NONCE_LEN] = crate::random::bytes().map_err(SidecarError::Random)?;
        let commitment = commit(&salt, value);

        let mut plain = Vec::with_capacity(SALT_LEN + value.len());
        plain.extend_from_slice(&salt);
        plain.extend_from_slice(value);
        let sealed = aead::seal(key, &nonce, &commitment, &plain);

        let entry = Entry {
            state: STATE_PRESENT,
            commitment,
            key_id,
            nonce,
            sealed,
        };
        let mut buf = Vec::with_capacity(entry.encoded_len());
        entry.write_to(&mut buf);

        let mut f = if self.path.exists() {
            std::fs::OpenOptions::new().append(true).open(&self.path)?
        } else {
            let mut f = std::fs::File::create(&self.path)?;
            f.write_all(MAGIC)?;
            f
        };
        f.write_all(&buf)?;
        // `fsync` here for the same reason the segment does it: an entry the ledger believes it
        // wrote and the disk does not have is a field that reads `Present` and cannot be opened.
        f.sync_all()?;
        Ok(commitment)
    }

    /// Read a value back. `Ok(None)` when there is no present entry for the commitment.
    ///
    /// `Ok(None)` rather than an error for a missing entry, because after an erasure that is the
    /// expected state and not a fault. A *wrong key* is an error, because it is.
    pub fn get(
        &self,
        key: &[u8; aead::KEY_LEN],
        commitment: &Commitment,
    ) -> Result<Option<Vec<u8>>, SidecarError> {
        let entries = self.read_all()?;
        let Some(e) = entries
            .iter()
            .find(|e| e.state == STATE_PRESENT && &e.commitment == commitment)
        else {
            return Ok(None);
        };
        let plain = aead::open(key, &e.nonce, &e.commitment, &e.sealed)
            .ok_or(SidecarError::NotAuthentic(*commitment))?;
        if plain.len() < SALT_LEN {
            return Err(SidecarError::NotAuthentic(*commitment));
        }
        Ok(Some(plain[SALT_LEN..].to_vec()))
    }

    /// Overwrite an entry's value region with zeroes in place, tombstone it, and `fsync`.
    ///
    /// The first of the two destructive steps. After this the value's bytes and its salt are
    /// gone from the file's contents; what remains is a hole of the value's length, which is why
    /// [`compact`](Self::compact) is the second step and not an optimisation.
    pub fn zero(&self, commitment: &Commitment) -> Result<(), SidecarError> {
        use std::io::{Seek, SeekFrom};
        let mut found = false;
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)?;
        for (offset, e) in self.scan()? {
            if &e.commitment != commitment {
                continue;
            }
            found = true;
            // The state byte first, so a crash between the two writes leaves an entry that is
            // tombstoned and still readable rather than one that is present and unreadable. A
            // reader must never see `Present` over zeroes: that is a field claiming to hold a
            // value it cannot produce.
            f.seek(SeekFrom::Start(offset))?;
            f.write_all(&[STATE_TOMBSTONED])?;
            f.sync_all()?;
            // Then the nonce and the sealed bytes. The commitment and key id stay: they name
            // what was destroyed, and `verify-erasure` reads them.
            let value_at = offset + 1 + 4 + 32 + 16;
            let value_len = aead::NONCE_LEN + e.sealed.len();
            f.seek(SeekFrom::Start(value_at))?;
            f.write_all(&vec![0u8; value_len])?;
            f.sync_all()?;
        }
        if !found {
            return Err(SidecarError::NoSuchCommitment(*commitment));
        }
        Ok(())
    }

    /// Rewrite the file without tombstoned entries, and return the new file's hash.
    ///
    /// The step that makes the bytes leave the file rather than merely become zeroes inside it.
    /// Written to a temporary beside the target and renamed, so a crash leaves either the old
    /// file or the new one and never a half-rewritten store.
    ///
    /// The returned hash is what `ErasureComplete` carries onto the chain: it is what lets a
    /// verifier holding only the chain and the file check that the file it has is the file the
    /// erasure produced.
    pub fn compact(&self) -> Result<[u8; 32], SidecarError> {
        let entries = self.read_all()?;
        let mut out = Vec::with_capacity(MAGIC.len() + entries.len() * 128);
        out.extend_from_slice(MAGIC);
        for e in entries.iter().filter(|e| e.state == STATE_PRESENT) {
            e.write_to(&mut out);
        }

        let tmp = self.path.with_extension("side.compacting");
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(&out)?;
            f.sync_all()?;
        }
        std::fs::rename(&tmp, &self.path)?;
        // The directory entry too: without this the rename can be lost while the file's own
        // contents are safe, which is the failure that leaves the pre-erasure file in place.
        if let Some(dir) = self.path.parent() {
            if let Ok(d) = std::fs::File::open(dir) {
                let _ = d.sync_all();
            }
        }

        let mut h = crate::chain::Hasher256::new();
        h.update(&out);
        Ok(h.finalize())
    }

    /// The hash of the file as it stands. What a verifier compares against `ErasureComplete`.
    pub fn hash(&self) -> Result<[u8; 32], SidecarError> {
        let bytes = match std::fs::read(&self.path) {
            Ok(b) => b,
            // A file that does not exist hashes as the empty file rather than erroring: a
            // segment with no confidential values has no sidecar, and that is not a fault.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(SidecarError::Io(e)),
        };
        let mut h = crate::chain::Hasher256::new();
        h.update(&bytes);
        Ok(h.finalize())
    }

    /// Every commitment the file still holds a present entry for.
    ///
    /// What `verify-erasure` asks: after an erasure, none of the listed commitments may appear.
    pub fn present(&self) -> Result<Vec<Commitment>, SidecarError> {
        Ok(self
            .read_all()?
            .into_iter()
            .filter(|e| e.state == STATE_PRESENT)
            .map(|e| e.commitment)
            .collect())
    }

    fn read_all(&self) -> Result<Vec<Entry>, SidecarError> {
        Ok(self.scan()?.into_iter().map(|(_, e)| e).collect())
    }

    /// Every entry with the offset it starts at.
    fn scan(&self) -> Result<Vec<(u64, Entry)>, SidecarError> {
        let mut bytes = Vec::new();
        match std::fs::File::open(&self.path) {
            Ok(mut f) => {
                f.read_to_end(&mut bytes)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(SidecarError::Io(e)),
        }
        if bytes.len() < MAGIC.len() || &bytes[..MAGIC.len()] != MAGIC {
            if bytes.is_empty() {
                return Ok(Vec::new());
            }
            return Err(SidecarError::NotASidecar);
        }

        let mut at = MAGIC.len();
        let mut out = Vec::new();
        while at < bytes.len() {
            let start = at as u64;
            let need = 1 + 4 + 32 + 16 + aead::NONCE_LEN;
            if at + need > bytes.len() {
                return Err(SidecarError::Truncated { at: start });
            }
            let state = bytes[at];
            at += 1;
            let len =
                u32::from_le_bytes(bytes[at..at + 4].try_into().expect("four bytes")) as usize;
            at += 4;
            let commitment: Commitment = bytes[at..at + 32].try_into().expect("thirty-two bytes");
            at += 32;
            let key_id: KeyId = bytes[at..at + 16].try_into().expect("sixteen bytes");
            at += 16;
            let nonce: [u8; aead::NONCE_LEN] =
                bytes[at..at + aead::NONCE_LEN].try_into().expect("a nonce");
            at += aead::NONCE_LEN;
            if at + len > bytes.len() {
                return Err(SidecarError::Truncated { at: start });
            }
            let sealed = bytes[at..at + len].to_vec();
            at += len;
            out.push((
                start,
                Entry {
                    state,
                    commitment,
                    key_id,
                    nonce,
                    sealed,
                },
            ));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory that removes itself, so a byte-scan test cannot be reading a
    /// previous run's file.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let p = std::env::temp_dir().join(format!("nls-sidecar-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).expect("a scratch directory");
            Scratch(p)
        }
        fn file(&self, n: &str) -> PathBuf {
            self.0.join(n)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn key() -> [u8; aead::KEY_LEN] {
        // A fixed key in tests: what is under test is the file, not the key store, and a
        // random key would make a failure unreproducible.
        let mut k = [0u8; aead::KEY_LEN];
        for (i, b) in k.iter_mut().enumerate() {
            *b = i as u8 ^ 0x5a;
        }
        k
    }

    const KID: KeyId = [7u8; 16];

    /// **The acceptance criterion the whole design exists for: the value is not in the file.**
    ///
    /// Not "is not in the record" — the sidecar is where the value went, and if the value were
    /// readable there the arrangement would have moved a plaintext from one file to another and
    /// called it encryption. This scans for the bytes.
    #[test]
    fn a_sealed_value_appears_nowhere_in_the_file_it_is_stored_in() {
        let s = Scratch::new("plaintext");
        let side = Sidecar::at(s.file("seg.side"));
        let secret = b"Ada Lovelace, 12 Marylebone Road, account closed at customer request";
        side.put(&key(), KID, secret).expect("sealed");

        let bytes = std::fs::read(side.path()).expect("the file");
        assert!(
            !contains(&bytes, secret),
            "the sealed value is present verbatim in the sidecar"
        );
        // And no substantial run of it either: a cipher that leaked a prefix would pass a
        // whole-value scan.
        for w in [8usize, 12, 16] {
            for chunk in secret.windows(w) {
                assert!(
                    !contains(&bytes, chunk),
                    "a {w}-byte run of the value survives in the file: {:?}",
                    String::from_utf8_lossy(chunk)
                );
            }
        }
    }

    /// **After zero and compact, the bytes are gone — checked against the bytes themselves.**
    ///
    /// The ciphertext is captured before the erasure, so this is not "we cannot find the
    /// plaintext" (which encryption alone gives) but "the exact bytes that were in the file are
    /// no longer in the file". That is the difference between a value that is unreadable and a
    /// value that is destroyed, and only the second one is erasure.
    #[test]
    fn zero_then_compact_removes_the_exact_bytes_that_were_there() {
        let s = Scratch::new("erase");
        let side = Sidecar::at(s.file("seg.side"));
        let a = side.put(&key(), KID, b"the value to erase").expect("a");
        let b = side.put(&key(), KID, b"a value that stays").expect("b");

        let before = std::fs::read(side.path()).expect("the file");
        let entries = side.scan().expect("scan");
        let victim = entries
            .iter()
            .find(|(_, e)| e.commitment == a)
            .map(|(_, e)| e.clone())
            .expect("the entry is there");
        assert!(contains(&before, &victim.sealed), "the fixture is wrong");

        side.zero(&a).expect("zeroed");
        let after_zero = std::fs::read(side.path()).expect("the file");
        assert!(
            !contains(&after_zero, &victim.sealed),
            "the ciphertext survived the zeroing"
        );
        assert!(
            !contains(&after_zero, &victim.nonce),
            "the nonce survived the zeroing"
        );

        let len_before = after_zero.len();
        side.compact().expect("compacted");
        let after = std::fs::read(side.path()).expect("the file");
        assert!(
            after.len() < len_before,
            "compaction did not shorten the file: {} then {}",
            len_before,
            after.len()
        );
        assert!(!contains(&after, &victim.sealed));

        // And the survivor is still readable, which is the half of erasure that is easy to
        // break: a compaction that lost the wrong entry would pass every scan above.
        assert_eq!(
            side.get(&key(), &b).expect("readable"),
            Some(b"a value that stays".to_vec()),
            "compaction destroyed an entry that was not erased"
        );
        assert_eq!(side.present().expect("present"), vec![b]);
    }

    /// **Zeroing without compacting leaves the file the right length and the bytes gone.**
    ///
    /// Stated separately because the two steps make different claims and a reader should be
    /// able to see which one does what. This is also the negative control for the test above:
    /// if `compact` were a no-op, that test's length assertion is what would catch it.
    #[test]
    fn zeroing_alone_leaves_a_hole_and_compaction_is_what_closes_it() {
        let s = Scratch::new("hole");
        let side = Sidecar::at(s.file("seg.side"));
        let c = side
            .put(&key(), KID, b"x".repeat(200).as_slice())
            .expect("put");
        let before = std::fs::metadata(side.path()).expect("stat").len();
        side.zero(&c).expect("zeroed");
        assert_eq!(
            std::fs::metadata(side.path()).expect("stat").len(),
            before,
            "zeroing changed the file's length; it is supposed to overwrite in place"
        );
        side.compact().expect("compacted");
        assert!(
            std::fs::metadata(side.path()).expect("stat").len() < before,
            "compaction is what removes the hole, and it did not"
        );
    }

    /// **An erased entry reads as absent, not as an error and not as a wrong value.**
    #[test]
    fn a_zeroed_entry_reads_as_absent() {
        let s = Scratch::new("absent");
        let side = Sidecar::at(s.file("seg.side"));
        let c = side.put(&key(), KID, b"gone soon").expect("put");
        assert!(side.get(&key(), &c).expect("readable").is_some());
        side.zero(&c).expect("zeroed");
        assert_eq!(
            side.get(&key(), &c).expect("no error"),
            None,
            "an erased field must read as absent rather than as a fault: a fault is a thing an \
             operator investigates, and there is nothing here to investigate"
        );
    }

    /// **A wrong key is distinguishable from an erasure.**
    ///
    /// If it were not, a key-store bug would present as "the data was erased" and an erasure
    /// would present as "something is broken". Both readings are wrong and both are actionable
    /// in different directions.
    #[test]
    fn a_wrong_key_is_an_error_and_not_an_absence() {
        let s = Scratch::new("wrongkey");
        let side = Sidecar::at(s.file("seg.side"));
        let c = side.put(&key(), KID, b"secret").expect("put");
        let mut other = key();
        other[0] ^= 0xff;
        match side.get(&other, &c) {
            Err(SidecarError::NotAuthentic(x)) => assert_eq!(x, c),
            other => panic!("a wrong key gave {other:?} rather than NotAuthentic"),
        }
    }

    /// **The commitment binds the ciphertext to its slot.**
    ///
    /// An entry moved to another commitment's place must fail to authenticate rather than
    /// decrypt to some other field's value. Without the associated data binding, swapping two
    /// entries in the file would swap two customers' narratives and nothing would notice.
    #[test]
    fn an_entry_moved_to_another_commitment_does_not_open() {
        let s = Scratch::new("bind");
        let side = Sidecar::at(s.file("seg.side"));
        let a = side.put(&key(), KID, b"alice's narrative").expect("a");
        let b = side.put(&key(), KID, b"bob's narrative").expect("b");
        let entries = side.scan().expect("scan");
        let ea = entries
            .iter()
            .find(|(_, e)| e.commitment == a)
            .unwrap()
            .1
            .clone();

        // Open a's ciphertext under b's commitment as associated data.
        assert!(
            aead::open(&key(), &ea.nonce, &b, &ea.sealed).is_none(),
            "a sealed value opened under a commitment that is not its own"
        );
    }

    /// **The salt is not in the clear, so a guess cannot be confirmed.**
    ///
    /// The property that makes the surviving commitment useless after erasure. If the salt sat
    /// beside the commitment, anyone with the file could compute `H(salt ‖ guess)` and check —
    /// and an erased value that can be recovered by guessing was not erased.
    #[test]
    fn the_salt_never_appears_outside_the_ciphertext() {
        let s = Scratch::new("salt");
        let side = Sidecar::at(s.file("seg.side"));
        // Recover the salt the only legitimate way and then look for it in the file.
        let value = b"yes";
        let c = side.put(&key(), KID, value).expect("put");
        let entries = side.scan().expect("scan");
        let e = &entries[0].1;
        let plain = aead::open(&key(), &e.nonce, &e.commitment, &e.sealed).expect("opens");
        let salt: [u8; SALT_LEN] = plain[..SALT_LEN].try_into().expect("a salt");
        assert_eq!(
            commit(&salt, value),
            c,
            "the commitment is over salt ‖ value"
        );

        let bytes = std::fs::read(side.path()).expect("the file");
        assert!(
            !contains(&bytes, &salt),
            "the salt is in the file in the clear; the commitment is then a dictionary lookup"
        );
    }

    /// **A missing sidecar is not an error.**
    ///
    /// Recovery must not need this file. A segment whose sidecar was deleted — by an operator,
    /// by a restore, by an erasure that removed the last entry — still recovers, and the fields
    /// read as unreadable rather than the ledger failing to start.
    #[test]
    fn a_sidecar_that_does_not_exist_reads_as_empty() {
        let s = Scratch::new("missing");
        let side = Sidecar::at(s.file("never-written.side"));
        assert!(!side.exists());
        assert_eq!(side.present().expect("no error"), Vec::<Commitment>::new());
        assert_eq!(side.get(&key(), &[0u8; 32]).expect("no error"), None);
        // And it hashes, so an `ErasureComplete` over a segment with no confidential values
        // carries a definite value rather than a special case.
        assert!(side.hash().is_ok());
    }

    /// **Compaction's hash is a function of what survived, and nothing else.**
    #[test]
    fn the_compaction_hash_is_over_the_surviving_entries() {
        let s = Scratch::new("hash");
        let a = Sidecar::at(s.file("a.side"));
        let b = Sidecar::at(s.file("b.side"));
        // Two files that end up holding the same one entry by different routes. The hash must
        // agree, or `ErasureComplete` cannot be checked by a verifier who did not watch the
        // erasure happen.
        let ka = a.put(&key(), KID, b"keep").expect("a");
        let doomed = a.put(&key(), KID, b"erase me").expect("doomed");
        a.zero(&doomed).expect("zeroed");
        let ha = a.compact().expect("compacted");

        // b holds only the survivor — but its salt and nonce are fresh, so the bytes differ and
        // so must the hash. That is the honest property: the hash is over the file, and two
        // files holding the same *values* are not the same file.
        b.put(&key(), KID, b"keep").expect("b");
        let hb = b.compact().expect("compacted");
        assert_ne!(ha, hb, "two files with different bytes hashed the same");
        assert_eq!(
            ha,
            a.hash().expect("hash"),
            "compaction returned a hash that is not the file's"
        );
        assert_eq!(a.present().expect("present"), vec![ka]);
    }

    /// **Zeroing something that is not there is an error, not a silent success.**
    ///
    /// An `erase` that reports success for a commitment it never found would produce an
    /// `ErasureComplete` event about data that is still somewhere else.
    #[test]
    fn zeroing_an_unknown_commitment_is_refused() {
        let s = Scratch::new("unknown");
        let side = Sidecar::at(s.file("seg.side"));
        side.put(&key(), KID, b"something").expect("put");
        match side.zero(&[9u8; 32]) {
            Err(SidecarError::NoSuchCommitment(_)) => {}
            other => panic!("erasing an absent commitment gave {other:?}"),
        }
    }

    /// **A file that is not a sidecar is refused rather than parsed.**
    #[test]
    fn a_file_with_the_wrong_magic_is_refused() {
        let s = Scratch::new("magic");
        let p = s.file("not.side");
        std::fs::write(&p, b"this is not a sidecar file at all").expect("written");
        match Sidecar::at(p).present() {
            Err(SidecarError::NotASidecar) => {}
            other => panic!("a foreign file gave {other:?}"),
        }
    }

    /// **A truncated file is refused rather than read as far as it goes.**
    ///
    /// Reading a partial store as though it were complete is how an erasure verification
    /// passes against a file that lost its tail.
    #[test]
    fn a_truncated_file_is_refused() {
        let s = Scratch::new("trunc");
        let p = s.file("seg.side");
        let side = Sidecar::at(p.clone());
        side.put(&key(), KID, b"a value long enough to lose the end of")
            .expect("put");
        let bytes = std::fs::read(&p).expect("the file");
        std::fs::write(&p, &bytes[..bytes.len() - 10]).expect("truncated");
        match side.present() {
            Err(SidecarError::Truncated { .. }) => {}
            other => panic!("a truncated file gave {other:?}"),
        }
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        if needle.is_empty() || needle.len() > haystack.len() {
            return false;
        }
        haystack.windows(needle.len()).any(|w| w == needle)
    }
}
