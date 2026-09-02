//! Durable segments: the write-ahead log, and what "committed" actually means.
//!
//! # Why durability is not an afterthought here
//!
//! The whole thesis rests on the base being authoritative, immutable and fully retained.
//! An in-memory base satisfies none of those across a restart, so every claim about
//! reconstruction is conditional on a durable one existing. This module is that condition
//! discharged: epochs are appended to length-prefixed, checksummed segment files, and an
//! epoch is **committed** exactly when its record is on stable storage, not when it is in
//! a buffer.
//!
//! # The commit rule, stated precisely
//!
//! A commit is: write the record, `fsync` the segment, *then* advance the visibility
//! frontier. The order is the entire content of the guarantee. Advancing the frontier
//! first would make a reader able to observe an epoch that a crash then erases — and in a
//! ledger, an observation that is later erased is not a stale read. It is a transaction
//! that a customer saw succeed and that no longer exists.
//!
//! [`SyncPolicy`] exists because that guarantee has a price, and the price should be
//! measurable rather than assumed. `Always` is the ledger-grade setting. `Never` is for
//! measurement only — it makes the *cost* of durability visible by removing it, and a
//! segment written under it is explicitly not a ledger.
//!
//! # Recovery
//!
//! A crash can leave a partial record at the tail: the process died between the write and
//! the fsync, or mid-write. Recovery reads forward, validating each record's checksum and
//! its chain link, and **truncates at the first record that fails**. That is the only
//! correct behaviour: a record whose checksum fails may be half-written, and a record whose
//! chain link fails is either corruption or an attempt to splice history. Neither may be
//! read past, because everything after it depends on it.

use crate::chain::Hasher256;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// When to force data to stable storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPolicy {
    /// fsync every epoch before the frontier advances. The only ledger-grade setting.
    Always,
    /// fsync every N epochs. A bounded window of committed-then-lost epochs: honest only
    /// where the caller can tolerate losing N, which a ledger cannot.
    Every(u32),
    /// Never fsync. **For measurement only.** A segment written under this policy is not
    /// a ledger; it is a benchmark of everything except the guarantee.
    Never,
}

/// One record on disk.
///
/// Layout, little-endian: `len: u32 | epoch: u64 | parent: [u8;32] | hash: [u8;32] |
/// payload: [u8; len - 72] | crc: u32`. The length prefix comes first so a truncated tail
/// is detectable without parsing.
///
/// The checksum covers the **body** — everything after the length prefix — and not the
/// prefix itself. That is deliberate and it is the honest statement: the prefix is
/// protected instead by the fact that a wrong length makes the record either overrun the
/// file (caught as `ShortTail`) or misalign the body (caught as a checksum failure). The
/// doc comment previously said "covers everything", which was simply false, and on an
/// audit artefact the difference between "everything" and "everything after the first
/// four bytes" is the sort of thing a reader is entitled to have stated correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub epoch: u64,
    pub parent: [u8; 32],
    pub hash: [u8; 32],
    pub payload: Vec<u8>,
}

const HEADER: usize = 8 + 32 + 32;

impl Record {
    /// The chain link: `h_e = H(h_{e-1} || canon(payload))`.
    ///
    /// Computed here rather than taken on trust, so that a caller cannot append a record
    /// whose hash does not follow from its parent. That is what makes the chain evidence
    /// rather than decoration.
    pub fn seal(epoch: u64, parent: [u8; 32], payload: Vec<u8>) -> Record {
        let mut h = Hasher256::new();
        h.update(&parent);
        h.update(&epoch.to_le_bytes());
        h.update(&payload);
        Record {
            epoch,
            parent,
            hash: h.finalize(),
            payload,
        }
    }

    fn encode(&self) -> Vec<u8> {
        let body_len = HEADER + self.payload.len();
        let mut out = Vec::with_capacity(4 + body_len + 4);
        out.extend_from_slice(&(body_len as u32).to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&self.parent);
        out.extend_from_slice(&self.hash);
        out.extend_from_slice(&self.payload);
        out.extend_from_slice(&crc32(&out[4..]).to_le_bytes());
        out
    }
}

/// Why recovery stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TruncationCause {
    /// Clean end of file.
    CleanEnd,
    /// The tail is shorter than its own length prefix says: a crash mid-write.
    ShortTail { at_offset: u64 },
    /// The checksum failed: the record is damaged or half-written.
    BadChecksum { epoch: u64, at_offset: u64 },
    /// The chain link does not follow from the previous record. Corruption, or an attempt
    /// to splice history. Either way, nothing after it may be read.
    BrokenChain { epoch: u64, at_offset: u64 },
    /// Epochs are out of order.
    OutOfOrder { expected: u64, found: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recovery {
    /// Records that validated, in order.
    pub records: Vec<Record>,
    /// Where and why reading stopped.
    pub cause: TruncationCause,
    /// Bytes discarded from the tail.
    pub truncated_bytes: u64,
}

impl Recovery {
    pub fn head(&self) -> Option<u64> {
        self.records.last().map(|r| r.epoch)
    }
    pub fn head_hash(&self) -> [u8; 32] {
        self.records.last().map(|r| r.hash).unwrap_or([0u8; 32])
    }
    /// Whether recovery ended at a clean end of file rather than at damage.
    pub fn was_clean(&self) -> bool {
        self.cause == TruncationCause::CleanEnd
    }
}

/// An append-only segment file.
pub struct Segment {
    path: PathBuf,
    file: File,
    policy: SyncPolicy,
    since_sync: u32,
    head_hash: [u8; 32],
    next_epoch: u64,
    /// How many fsyncs this segment has performed. The cost of the guarantee, counted.
    pub fsyncs: u64,
    pub bytes_written: u64,
}

impl Segment {
    /// Open a segment, recovering any existing content first.
    ///
    /// Recovery is not optional and cannot be skipped: a segment whose tail has not been
    /// validated is a segment whose head hash is unknown, and appending to it would extend
    /// a chain from a link that may not exist.
    pub fn open(
        path: impl AsRef<Path>,
        policy: SyncPolicy,
    ) -> std::io::Result<(Segment, Recovery)> {
        let path = path.as_ref().to_path_buf();
        let recovery = if path.exists() {
            recover(&path)?
        } else {
            Recovery {
                records: Vec::new(),
                cause: TruncationCause::CleanEnd,
                truncated_bytes: 0,
            }
        };

        // Damage in the *middle* of a segment is not a tail to be trimmed. Truncating to
        // the last valid record would discard every committed, fsynced, acknowledged
        // epoch after the damage — one flipped byte in the first record of a three-record
        // file emptied it entirely and restarted the chain from genesis, after which the
        // file validated cleanly and nothing could tell that history had been lost.
        //
        // The tail is the last record only. If the bytes after the damage could still
        // hold a record, this is mid-file corruption: refuse, and let an operator decide.
        let valid_len: u64 = recovery
            .records
            .iter()
            .map(|r| (4 + HEADER + r.payload.len() + 4) as u64)
            .sum();
        if !matches!(recovery.cause, TruncationCause::CleanEnd) {
            let file_len = std::fs::metadata(&path)?.len();
            let damaged_from = valid_len;
            // How long the damaged record *claims* to be, read from its own length
            // prefix. Bytes beyond that are a further record, so the damage is not at
            // the tail and truncating to `valid_len` would discard committed epochs.
            let declared = declared_record_len(&path, damaged_from)?;
            let is_tail = match declared {
                // The length prefix itself is unreadable: nothing can follow.
                None => true,
                Some(n) => file_len <= damaged_from.saturating_add(n),
            };
            if !is_tail {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "segment {} is damaged at offset {damaged_from} ({:?}) and {} \
                         bytes of further records follow it; refusing to open, and \
                         truncating nothing. A ledger does not repair itself by \
                         forgetting.",
                        path.display(),
                        recovery.cause,
                        file_len - damaged_from - declared.unwrap_or(0)
                    ),
                ));
            }
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            // Explicit, and load-bearing: this is a ledger segment. `false` is already
            // the default, but a segment opened with truncation would silently discard
            // every committed epoch, so the intent is stated rather than inherited.
            .truncate(false)
            .open(&path)?;
        if file.metadata()?.len() != valid_len {
            file.set_len(valid_len)?;
            file.sync_all()?;
        }
        let mut file = file;
        file.seek(SeekFrom::End(0))?;

        // The file's *directory entry* must be durable too. `fdatasync` on the file makes
        // its contents survive; it says nothing about whether the name exists after a
        // crash, so a segment created and fully synced can still be absent on restart.
        if let Some(dir) = path.parent() {
            if let Ok(d) = File::open(if dir.as_os_str().is_empty() {
                Path::new(".")
            } else {
                dir
            }) {
                // Best effort: some filesystems refuse to sync a directory handle, and
                // failing to open a segment because of that would be worse than the risk.
                let _ = d.sync_all();
            }
        }

        let head_hash = recovery.head_hash();
        let next_epoch = recovery.head().map_or(0, |e| e + 1);
        Ok((
            Segment {
                path,
                file,
                policy,
                since_sync: 0,
                head_hash,
                next_epoch,
                fsyncs: 0,
                bytes_written: valid_len,
            },
            recovery,
        ))
    }

    pub fn head_hash(&self) -> [u8; 32] {
        self.head_hash
    }
    pub fn next_epoch(&self) -> u64 {
        self.next_epoch
    }
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one epoch and, if the policy says so, force it to stable storage.
    ///
    /// Returns the sealed record. **The caller must not advance the visibility frontier
    /// until this returns**, which is why it returns the record rather than taking a
    /// callback: the ordering is the guarantee, and making it the caller's obligation in
    /// code is more honest than burying it in a comment.
    pub fn append(&mut self, payload: Vec<u8>) -> std::io::Result<Record> {
        let rec = Record::seal(self.next_epoch, self.head_hash, payload);
        let bytes = rec.encode();
        // A partial write must not be left in place. `write_all` can fail after writing
        // some bytes — ENOSPC is the ordinary way — and the previous behaviour was to
        // return the error and carry on appending, so every later epoch (fsynced,
        // published, acknowledged to a client) sat behind a torn record and was discarded
        // at the next recovery. Roll back to the last complete record instead.
        if let Err(e) = self.file.write_all(&bytes) {
            let _ = self.file.set_len(self.bytes_written);
            let _ = self.file.seek(SeekFrom::End(0));
            let _ = self.file.sync_data();
            return Err(e);
        }
        self.bytes_written += bytes.len() as u64;
        self.since_sync += 1;
        let should_sync = match self.policy {
            SyncPolicy::Always => true,
            SyncPolicy::Every(n) => self.since_sync >= n.max(1),
            SyncPolicy::Never => false,
        };
        if should_sync {
            self.file.sync_data()?;
            self.fsyncs += 1;
            self.since_sync = 0;
        }
        self.head_hash = rec.hash;
        self.next_epoch += 1;
        Ok(rec)
    }

    /// Force everything written so far to stable storage.
    pub fn sync(&mut self) -> std::io::Result<()> {
        self.file.sync_data()?;
        self.fsyncs += 1;
        self.since_sync = 0;
        Ok(())
    }
}

/// The total on-disk size the record at `offset` claims for itself, from its own length
/// prefix. `None` if the prefix cannot be read, which means nothing can follow it.
///
/// This is what distinguishes a torn tail from mid-file corruption. A crash mid-write
/// leaves a record shorter than it says it is, with nothing after it. Damage with a
/// complete further record behind it is a different event, and the two must not be
/// repaired the same way.
fn declared_record_len(path: &Path, offset: u64) -> std::io::Result<Option<u64>> {
    let mut f = File::open(path)?;
    if f.seek(SeekFrom::Start(offset)).is_err() {
        return Ok(None);
    }
    let mut len_buf = [0u8; 4];
    if f.read_exact(&mut len_buf).is_err() {
        return Ok(None);
    }
    let body = u32::from_le_bytes(len_buf) as u64;
    Ok(Some(4 + body + 4))
}

/// Read a segment forward, validating as it goes, and stop at the first failure.
pub fn recover(path: impl AsRef<Path>) -> std::io::Result<Recovery> {
    let file = File::open(&path)?;
    let total = file.metadata()?.len();
    let mut r = BufReader::new(file);
    let mut records: Vec<Record> = Vec::new();
    let mut offset = 0u64;
    let mut parent = [0u8; 32];
    let mut expected_epoch = 0u64;

    let cause = loop {
        let mut len_buf = [0u8; 4];
        match r.read_exact(&mut len_buf) {
            Ok(()) => {}
            // `UnexpectedEof` at a record boundary with nothing read is the clean end.
            // *Anything else* is not: a one-, two- or three-byte tail is a crash caught
            // mid-length-prefix, and a real I/O error is a failing disk. Mapping both to
            // "clean" reported `was_clean = true` on a torn file and made a device error
            // indistinguishable from a tidy shutdown.
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof && offset == total => {
                break TruncationCause::CleanEnd
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break TruncationCause::ShortTail { at_offset: offset }
            }
            Err(e) => return Err(e),
        };
        let body_len = u32::from_le_bytes(len_buf) as usize;
        if body_len < HEADER || (offset + 4 + body_len as u64 + 4) > total {
            break TruncationCause::ShortTail { at_offset: offset };
        }
        let mut body = vec![0u8; body_len];
        if r.read_exact(&mut body).is_err() {
            break TruncationCause::ShortTail { at_offset: offset };
        }
        let mut crc_buf = [0u8; 4];
        if r.read_exact(&mut crc_buf).is_err() {
            break TruncationCause::ShortTail { at_offset: offset };
        }
        let epoch = u64::from_le_bytes(body[0..8].try_into().unwrap());
        let mut with_len = Vec::with_capacity(body.len());
        with_len.extend_from_slice(&body);
        if crc32(&with_len) != u32::from_le_bytes(crc_buf) {
            break TruncationCause::BadChecksum {
                epoch,
                at_offset: offset,
            };
        }
        let mut rec_parent = [0u8; 32];
        rec_parent.copy_from_slice(&body[8..40]);
        let mut rec_hash = [0u8; 32];
        rec_hash.copy_from_slice(&body[40..72]);
        let payload = body[72..].to_vec();

        if epoch != expected_epoch {
            break TruncationCause::OutOfOrder {
                expected: expected_epoch,
                found: epoch,
            };
        }
        // Recompute the link. A record whose stated hash does not follow from its parent is
        // either damaged or spliced, and in a ledger those are the same problem.
        let recomputed = Record::seal(epoch, parent, payload.clone());
        if rec_parent != parent || recomputed.hash != rec_hash {
            break TruncationCause::BrokenChain {
                epoch,
                at_offset: offset,
            };
        }

        parent = rec_hash;
        expected_epoch = epoch + 1;
        offset += 4 + body_len as u64 + 4;
        records.push(Record {
            epoch,
            parent: rec_parent,
            hash: rec_hash,
            payload,
        });
    };

    Ok(Recovery {
        records,
        cause,
        truncated_bytes: total - offset,
    })
}

/// CRC-32 (IEEE), computed with a small table built on first use. Written out rather than
/// taken from a crate because this workspace builds without network access, and a
/// checksum is short enough that a dependency would be the larger risk.
fn crc32(data: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, e) in t.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            *e = c;
        }
        t
    });
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc = table[((crc ^ *b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("niles-seg-{name}-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn an_appended_epoch_survives_a_reopen() {
        let p = tmp("reopen");
        {
            let (mut s, rec) = Segment::open(&p, SyncPolicy::Always).unwrap();
            assert!(rec.records.is_empty());
            for i in 0..5u8 {
                s.append(vec![i, i, i]).unwrap();
            }
            assert_eq!(s.fsyncs, 5, "Always must fsync every epoch");
        }
        let (s, rec) = Segment::open(&p, SyncPolicy::Always).unwrap();
        assert_eq!(rec.records.len(), 5);
        assert!(rec.was_clean());
        assert_eq!(rec.head(), Some(4));
        assert_eq!(
            s.next_epoch(),
            5,
            "reopening continues the chain, it does not restart it"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn the_chain_links_each_epoch_to_the_one_before() {
        let p = tmp("chain");
        {
            let (mut s, _) = Segment::open(&p, SyncPolicy::Always).unwrap();
            for i in 0..4u8 {
                s.append(vec![i]).unwrap();
            }
        }
        let rec = recover(&p).unwrap();
        for w in rec.records.windows(2) {
            assert_eq!(
                w[1].parent, w[0].hash,
                "epoch {} does not link to {}",
                w[1].epoch, w[0].epoch
            );
        }
        assert_eq!(
            rec.records[0].parent, [0u8; 32],
            "the first epoch's parent is the zero hash"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_torn_tail_is_truncated_and_everything_before_it_survives() {
        // The crash case: the process died mid-write. The committed prefix must come back
        // intact and the partial record must go, because a half-written record is not a
        // fact about the world.
        let p = tmp("torn");
        {
            let (mut s, _) = Segment::open(&p, SyncPolicy::Always).unwrap();
            for i in 0..6u8 {
                s.append(vec![i; 40]).unwrap();
            }
        }
        // Simulate the tear: chop the last few bytes.
        let len = std::fs::metadata(&p).unwrap().len();
        let f = OpenOptions::new().write(true).open(&p).unwrap();
        f.set_len(len - 9).unwrap();
        drop(f);

        let rec = recover(&p).unwrap();
        assert_eq!(rec.records.len(), 5, "the five whole epochs must survive");
        assert!(
            matches!(rec.cause, TruncationCause::ShortTail { .. }),
            "{:?}",
            rec.cause
        );
        assert!(rec.truncated_bytes > 0);

        // Reopening truncates the damage and continues cleanly from the surviving head.
        let (mut s, rec2) = Segment::open(&p, SyncPolicy::Always).unwrap();
        assert_eq!(rec2.records.len(), 5);
        assert_eq!(s.next_epoch(), 5);
        s.append(vec![99]).unwrap();
        let rec3 = recover(&p).unwrap();
        assert_eq!(rec3.records.len(), 6);
        assert!(
            rec3.was_clean(),
            "the segment must be usable again after recovery: {:?}",
            rec3.cause
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn tampering_with_a_committed_record_is_detected_and_stops_the_read() {
        // The property the hash chain exists for. A reader who kept a digest can tell that
        // history was altered; a reader who did not can at least tell that it no longer
        // validates, and must not read past the break.
        let p = tmp("tamper");
        {
            let (mut s, _) = Segment::open(&p, SyncPolicy::Always).unwrap();
            for i in 0..5u8 {
                s.append(vec![i; 16]).unwrap();
            }
        }
        let mut bytes = std::fs::read(&p).unwrap();
        // Flip a payload byte deep inside the file, in epoch 2's record.
        let idx = 4 + HEADER + 4 + (4 + HEADER + 16 + 4) * 2;
        bytes[idx] ^= 0xFF;
        std::fs::write(&p, &bytes).unwrap();

        let rec = recover(&p).unwrap();
        assert!(
            rec.records.len() < 5,
            "the read must stop at the tampered record"
        );
        assert!(
            matches!(
                rec.cause,
                TruncationCause::BadChecksum { .. } | TruncationCause::BrokenChain { .. }
            ),
            "{:?}",
            rec.cause
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_spliced_record_breaks_the_chain_even_with_a_valid_checksum() {
        // The stronger case: an attacker who rewrites a record *and* fixes its checksum.
        // The chain still catches it, because the following record's parent no longer
        // matches. This is why the checksum and the chain are both needed: one catches
        // damage, the other catches intent.
        let p = tmp("splice");
        {
            let (mut s, _) = Segment::open(&p, SyncPolicy::Always).unwrap();
            for i in 0..4u8 {
                s.append(vec![i; 8]).unwrap();
            }
        }
        let mut bytes = std::fs::read(&p).unwrap();
        let rec_size = 4 + HEADER + 8 + 4;
        // Rewrite epoch 1's payload and repair its CRC, leaving its stated hash alone.
        let start = rec_size;
        bytes[start + 4 + HEADER] = 0xAA;
        let body: Vec<u8> = bytes[start + 4..start + 4 + HEADER + 8].to_vec();
        let crc = crc32(&body);
        bytes[start + 4 + HEADER + 8..start + rec_size].copy_from_slice(&crc.to_le_bytes());
        std::fs::write(&p, &bytes).unwrap();

        let rec = recover(&p).unwrap();
        assert!(
            matches!(rec.cause, TruncationCause::BrokenChain { epoch: 1, .. }),
            "{:?}",
            rec.cause
        );
        assert_eq!(
            rec.records.len(),
            1,
            "only the epoch before the splice may be read"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn the_sync_policy_changes_how_often_the_disk_is_forced() {
        // Durability has a price, and the price should be countable rather than assumed.
        let p = tmp("policy");
        let (mut s, _) = Segment::open(&p, SyncPolicy::Every(4)).unwrap();
        for i in 0..12u8 {
            s.append(vec![i]).unwrap();
        }
        assert_eq!(s.fsyncs, 3, "Every(4) over 12 epochs is 3 fsyncs, not 12");
        let _ = std::fs::remove_file(&p);

        let p2 = tmp("policy-never");
        let (mut s2, _) = Segment::open(&p2, SyncPolicy::Never).unwrap();
        for i in 0..12u8 {
            s2.append(vec![i]).unwrap();
        }
        assert_eq!(
            s2.fsyncs, 0,
            "Never is for measurement only, and must really never sync"
        );
        let _ = std::fs::remove_file(&p2);
    }

    #[test]
    fn recovery_reports_what_it_discarded_rather_than_discarding_it_quietly() {
        let p = tmp("report");
        {
            let (mut s, _) = Segment::open(&p, SyncPolicy::Always).unwrap();
            for i in 0..3u8 {
                s.append(vec![i; 20]).unwrap();
            }
        }
        let mut bytes = std::fs::read(&p).unwrap();
        bytes.extend_from_slice(&[0xFF; 30]); // junk tail
        std::fs::write(&p, &bytes).unwrap();
        let rec = recover(&p).unwrap();
        assert_eq!(rec.records.len(), 3);
        assert!(
            rec.truncated_bytes >= 30,
            "the operator must be told how much was dropped"
        );
        assert!(!rec.was_clean());
        let _ = std::fs::remove_file(&p);
    }
}

#[cfg(test)]
mod durability_tests {
    //! What "committed" survives, and what a damaged segment is allowed to do about it.

    use super::*;

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "niles-dur-{name}-{}-{}.log",
            std::process::id(),
            name.len()
        ))
    }

    fn three_records(p: &PathBuf) {
        let _ = std::fs::remove_file(p);
        let (mut s, _) = Segment::open(p, SyncPolicy::Always).unwrap();
        for i in 0..3u8 {
            s.append(vec![i; 40]).unwrap();
        }
    }

    #[test]
    fn torn_middle_refuses_to_open_and_truncates_nothing() {
        // One flipped byte in the *first* record of three. Previously `open` truncated to
        // the last valid record — zero of them — emptying a 266-byte file and restarting
        // the chain from genesis, after which it validated cleanly and nothing could tell
        // that two committed, fsynced epochs had been lost.
        let p = tmp("tornmid");
        three_records(&p);
        let before = std::fs::metadata(&p).unwrap().len();

        let mut bytes = std::fs::read(&p).unwrap();
        let payload_byte = 4 + HEADER + 1;
        bytes[payload_byte] ^= 0xFF;
        std::fs::write(&p, &bytes).unwrap();

        let err = match Segment::open(&p, SyncPolicy::Always) {
            Ok(_) => panic!("a segment damaged in the middle must not open"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("refusing to open"),
            "{}",
            err.to_string()
        );
        assert_eq!(
            std::fs::metadata(&p).unwrap().len(),
            before,
            "a refusal must not shorten the file by a single byte"
        );
    }

    #[test]
    fn short_tail_of_one_byte_is_torn() {
        // A tail of one, two or three bytes is a crash caught mid-length-prefix. Mapping
        // the read error to `CleanEnd` reported `was_clean = true` on a torn file.
        for chop in [1u64, 2, 3] {
            let p = tmp(&format!("shorttail{chop}"));
            three_records(&p);
            let mut bytes = std::fs::read(&p).unwrap();
            bytes.truncate(bytes.len());
            // Append a partial length prefix: a write that started and did not finish.
            bytes.extend(std::iter::repeat_n(0xABu8, chop as usize));
            std::fs::write(&p, &bytes).unwrap();

            let rec = recover(&p).unwrap();
            assert!(
                !rec.was_clean(),
                "a {chop}-byte tail must not be reported as a clean end: {:?}",
                rec.cause
            );
            assert!(matches!(rec.cause, TruncationCause::ShortTail { .. }));
            assert_eq!(rec.records.len(), 3, "the whole records must survive");
            // And it opens: a torn tail is the one damage a ledger may repair itself from.
            let (_s, r2) = Segment::open(&p, SyncPolicy::Always).unwrap();
            assert_eq!(r2.records.len(), 3);
        }
    }

    #[test]
    fn write_failure_mid_batch_leaves_no_orphaned_epochs() {
        // ENOSPC in the middle of a record. The previous behaviour returned the error and
        // carried on appending, so every later epoch — fsynced, published, acknowledged —
        // sat behind a torn record and was discarded at the next recovery.
        let p = tmp("enospc");
        three_records(&p);
        let len_before = std::fs::metadata(&p).unwrap().len();

        // Simulate the partial write directly: bytes on the end that are not a record.
        let mut bytes = std::fs::read(&p).unwrap();
        bytes.extend(std::iter::repeat_n(0u8, 7));
        std::fs::write(&p, &bytes).unwrap();

        // Reopening rolls the tail back, and the next append lands on a clean boundary.
        let (mut s, rec) = Segment::open(&p, SyncPolicy::Always).unwrap();
        assert_eq!(rec.records.len(), 3);
        assert_eq!(std::fs::metadata(&p).unwrap().len(), len_before);
        s.append(vec![9u8; 40]).unwrap();
        drop(s);

        let after = recover(&p).unwrap();
        assert!(after.was_clean(), "{:?}", after.cause);
        assert_eq!(
            after.records.len(),
            4,
            "the new epoch follows the old three"
        );
        for (i, r) in after.records.iter().enumerate() {
            assert_eq!(r.epoch, i as u64, "epochs must be contiguous");
        }
    }

    #[test]
    fn a_segment_survives_reopening_many_times() {
        // The directory entry must be durable too, not only the bytes inside the file.
        let p = tmp("reopen");
        let _ = std::fs::remove_file(&p);
        for i in 0..5u8 {
            let (mut s, r) = Segment::open(&p, SyncPolicy::Always).unwrap();
            assert_eq!(r.records.len(), i as usize);
            s.append(vec![i; 16]).unwrap();
        }
        let rec = recover(&p).unwrap();
        assert_eq!(rec.records.len(), 5);
        assert!(rec.was_clean());
    }
}
