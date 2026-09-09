//! **The torn-segment corpus: every mutation is refused, or recovers exactly what was
//! acknowledged.**
//!
//! Cycle 8's T-14.3 tried one short envelope at one offset and found a silent drop. The path
//! refused that case afterwards; nothing showed it refused any *other* case. Cycle 9's audit
//! built this as a throwaway probe and found that 1,120 of 8,840 single-bit flips recovered a
//! prefix and reported success — 216 of them in a non-final record's length prefix, the four
//! bytes the checksum does not cover (F-61), and the rest in the final record, which the
//! format cannot distinguish from a crash mid-write (LC-36).
//!
//! The property, one sentence: **refuse, or recover exactly what was acknowledged; never a
//! prefix with a success code.** Truncation is the one mutation allowed to shorten the answer,
//! because a torn tail is what a crash leaves and trimming it is the whole point of recovery.
//!
//! Exhaustive and deterministic: every byte offset, every bit, plus the structural mutations
//! (a duplicated record, a swapped pair, a junk tail) and a second family in which the
//! storage layer is made to *pass* — the CRC recomputed — so that only the envelope inside is
//! malformed. That second family is the one that reaches `decode_envelope` at all; without it
//! every case stops at the checksum and the envelope's own strictness is never exercised.

use nilestream_ledger::frontiers::Frontier;
use nilestream_ledger::segment::{Record, Segment, SyncPolicy};
use nilestream_ledger::sequencer::{Sequencer, Txn};
use std::collections::BTreeMap;

/// **A fixture path this test owns, and can prove it owns.**
///
/// # What was here, and what it did on the Mac
///
/// ```text
/// temp_dir()/niles-torn-{pid}-{tag}-{wall_clock_nanos}
/// let _ = std::fs::remove_file(&p);          // unconditional, on a path it had not created
/// ```
///
/// Three things at once. The name's only unique component is a *clock reading*, and a clock
/// reading is not an identity: two threads that call this within the same tick get the same
/// path. Two test functions share the tag `base`, so a collision does not even need bad luck
/// across tags. And the helper then **deletes** whatever is at that path — a path it did not
/// create and has no claim to — so the loser of the race has its fixture removed from under
/// it and fails at `read`, which is the `exit 101` this file showed on 1 of 3 eight-thread
/// runs on Host C.
///
/// # What this is instead
///
/// A **directory**, created with `create_dir`, which is atomic: exactly one caller can create
/// a given name and every other gets `AlreadyExists`. The directory is the ownership token —
/// having created it is the proof — and the fixture file lives inside it, so no caller can
/// name another's file at all. The suffix is a process-local counter rather than a clock,
/// which is monotone by construction instead of by hope, and a collision (another process,
/// the same pid after a wrap) is *refused and retried*, never taken.
///
/// Cleanup removes only this directory, on drop, after the handles inside it are closed.
/// Nothing here ever deletes a path it did not create.
struct Owned {
    dir: std::path::PathBuf,
}

impl Owned {
    fn new(tag: &str) -> Owned {
        // **Per tag, not global.** A single counter shared across tags makes one tag's
        // sequence depend on how many allocations other tests happened to make first, which
        // is exactly the kind of cross-test coupling this whole repair is about — and it made
        // `a_taken_name_is_stepped_over_and_its_contents_are_untouched` pass vacuously,
        // because the name it squatted was no longer the name the allocator would try.
        use std::collections::HashMap;
        use std::sync::{Mutex, OnceLock};
        static SEQ: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
        let seq = SEQ.get_or_init(|| Mutex::new(HashMap::new()));
        let root = std::env::temp_dir();
        for _ in 0..1_000 {
            let n = {
                let mut g = seq.lock().expect("the fixture counter is not poisoned");
                let e = g.entry(tag.to_string()).or_insert(0);
                let n = *e;
                *e += 1;
                n
            };
            let dir = root.join(format!("niles-torn-{}-{tag}-{n}", std::process::id()));
            match std::fs::create_dir(&dir) {
                Ok(()) => return Owned { dir },
                // Someone else has this name. Refused, not taken: the whole defect was a
                // helper that resolved a collision by deleting the other party's file.
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("cannot create a fixture directory in {root:?}: {e}"),
            }
        }
        panic!("a thousand consecutive fixture names were taken in {root:?}");
    }

    /// The fixture file's path. Inside the owned directory, so it is unreachable by name from
    /// any other allocation.
    fn path(&self) -> std::path::PathBuf {
        self.dir.join("segment")
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // Only this directory, and only what is in it. A failure here is not a test failure:
        // the temp directory is the operating system's to reclaim, and panicking in a
        // destructor during unwinding aborts the process.
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Verdict {
    /// The segment would not open, or the replay refused it. The bytes are untouched.
    Refused,
    /// Every acknowledged transaction came back, in order.
    Exact,
    /// A strict prefix of the acknowledged transactions came back, and the caller was told
    /// nothing was wrong. **This is the defect.**
    PrefixWithSuccess,
    /// Something else came back: not the history, not a prefix of it.
    Other,
}

/// Open the mutated bytes and try to recover, reporting what happened and whether the input
/// file was modified in the attempt.
fn classify(bytes: &[u8], sent: &[(String, Vec<u8>)], tag: &str) -> (Verdict, String, bool) {
    let owned = Owned::new(tag);
    let p = owned.path();
    std::fs::write(&p, bytes).expect("write the fixture");
    let before = std::fs::read(&p).expect("read it back");
    let outcome = match Sequencer::open_bounded(&p, SyncPolicy::Always, None) {
        Err(e) => (Verdict::Refused, format!("open: {e}")),
        Ok((s, recovery)) => {
            let r = match Sequencer::recover_txns_checked(&recovery) {
                Err(e) => (Verdict::Refused, format!("replay: {e}")),
                Ok(back) if back == sent => (Verdict::Exact, String::new()),
                Ok(back) if back.len() < sent.len() && back[..] == sent[..back.len()] => (
                    Verdict::PrefixWithSuccess,
                    format!(
                        "{} of {} transactions, cause {:?}, {} bytes truncated",
                        back.len(),
                        sent.len(),
                        recovery.cause,
                        recovery.truncated_bytes
                    ),
                ),
                Ok(back) => (
                    Verdict::Other,
                    format!("{} transactions, not a prefix", back.len()),
                ),
            };
            s.shutdown();
            r
        }
    };
    let after = std::fs::read(&p).expect("read after");
    let _ = std::fs::remove_file(&p);
    (outcome.0, outcome.1, before != after)
}

/// A small, fully acknowledged fixture: ten transactions, **one per record**.
///
/// Deterministic on purpose. An earlier version queued half of them with `submit_pending` so
/// that one record would carry a multi-transaction envelope, and the sealer's drain then
/// decided how many records there were — so the corpus's size, and the offsets it reports,
/// changed with the machine's load. A corpus whose case count is scheduler-shaped cannot be
/// compared between runs, which is the whole reason for having one.
///
/// The multi-transaction decode path is still exercised, by the envelope family below: a
/// record whose count is raised to two makes the decoder walk a second `(key, payload)` pair
/// that is not there, which is exactly the framing a short batch envelope produces.
fn fixture() -> (Vec<u8>, Vec<(String, Vec<u8>)>) {
    let owned = Owned::new("base");
    let p = owned.path();
    let sent: Vec<(String, Vec<u8>)> = (0..10u8)
        .map(|i| (format!("txn-{i:02}"), vec![i; 4 + i as usize]))
        .collect();
    {
        let (segment, recovery) = Segment::open(&p, SyncPolicy::Always).unwrap();
        let seen = Sequencer::recover_seen_checked(&recovery).expect("a fresh segment decodes");
        let s = Sequencer::start_with_window(segment, Frontier::new(), SyncPolicy::Always, seen);
        // `submit` waits for each barrier, so the sealer's drain finds one transaction every
        // time and the file's shape does not depend on scheduling.
        for (k, payload) in &sent {
            s.submit(Txn {
                idem_key: k.clone(),
                payload: payload.clone(),
            })
            .expect("new");
        }
        s.shutdown();
    }
    let bytes = std::fs::read(&p).expect("the fixture");
    // No `remove_file` here: `owned` removes its own directory when it drops at the end of
    // this function, after every handle in it is closed. The old helper removed the path
    // eagerly and, worse, removed it again on the *next* allocation of the same name.
    (bytes, sent)
}

/// The offsets at which each record starts, walked with the format's own framing.
fn record_starts(bytes: &[u8]) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut o = 0usize;
    while o + 8 <= bytes.len() {
        starts.push(o);
        let len = u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap()) as usize;
        o += 8 + len + 4;
    }
    starts
}

#[test]
fn every_mutation_is_refused_or_recovers_exactly_what_was_acknowledged() {
    let (bytes, sent) = fixture();
    let starts = record_starts(&bytes);
    assert_eq!(
        starts.len(),
        sent.len(),
        "PRECONDITION UNMET: the fixture must hold one record per transaction, and its shape \
         must not depend on scheduling"
    );
    let last_record = *starts.last().expect("at least one record");
    let (v, why, _) = classify(&bytes, &sent, "clean");
    assert_eq!(
        v,
        Verdict::Exact,
        "PRECONDITION UNMET: the unmutated fixture must recover exactly: {why}"
    );
    eprintln!(
        "  fixture: {} bytes, {} records, {} transactions, last record at {last_record}",
        bytes.len(),
        starts.len(),
        sent.len()
    );

    // ---- 1. truncation at every byte offset -------------------------------------------
    //
    // A truncated segment is a crash mid-write by definition: recovering a prefix is correct
    // here and is the only mutation class for which it is.
    let mut trunc: BTreeMap<Verdict, usize> = BTreeMap::new();
    for cut in 0..bytes.len() {
        let (v, why, changed) = classify(&bytes[..cut], &sent, "cut");
        assert!(
            matches!(
                v,
                Verdict::PrefixWithSuccess | Verdict::Exact | Verdict::Refused
            ),
            "truncation at {cut} produced {v:?}: {why}"
        );
        assert!(
            !changed || v != Verdict::Refused,
            "a refusal at {cut} rewrote its input"
        );
        *trunc.entry(v).or_default() += 1;
    }
    eprintln!("  truncation ({} offsets): {trunc:?}", bytes.len());

    // ---- 2. every single-bit flip ------------------------------------------------------
    let mut flips: BTreeMap<Verdict, usize> = BTreeMap::new();
    let mut leaks: Vec<String> = Vec::new();
    let mut rewrote: Vec<String> = Vec::new();
    for byte in 0..bytes.len() {
        for bit in 0..8u32 {
            let mut m = bytes.clone();
            m[byte] ^= 1 << bit;
            let (v, why, changed) = classify(&m, &sent, "flip");
            *flips.entry(v).or_default() += 1;
            if v == Verdict::Refused && changed {
                rewrote.push(format!("byte {byte} bit {bit}"));
            }
            // A prefix with a success code is the defect — **except** in the final record,
            // which a crash mid-write is indistinguishable from without a retained tip
            // (LC-36, `BLOCKED-recovery-tip`).
            if matches!(v, Verdict::PrefixWithSuccess | Verdict::Other) && byte < last_record {
                leaks.push(format!("byte {byte} bit {bit}: {v:?} — {why}"));
            }
        }
    }
    eprintln!("  bit flips ({} cases): {flips:?}", bytes.len() * 8);
    assert!(
        leaks.is_empty(),
        "{} single-bit flip(s) outside the final record recovered a prefix and reported \
         success. Each one is an acknowledged, fsynced transaction discarded by a clean \
         open. First few: {:?}",
        leaks.len(),
        &leaks[..leaks.len().min(5)]
    );
    assert!(
        rewrote.is_empty(),
        "a refusal must leave the segment's bytes untouched; these did not: {rewrote:?}"
    );

    // ---- 3. structural mutations -------------------------------------------------------
    let mut dup = bytes.clone();
    dup.extend_from_slice(&bytes[last_record..]);
    let (v, why, _) = classify(&dup, &sent, "dup");
    assert!(
        matches!(v, Verdict::Refused | Verdict::Exact),
        "a duplicated record must be refused or trimmed as a tail, not folded in: {v:?} {why}"
    );

    let (a, b, c) = (starts[1], starts[2], starts[3]);
    let mut swapped = bytes[..a].to_vec();
    swapped.extend_from_slice(&bytes[b..c]);
    swapped.extend_from_slice(&bytes[a..b]);
    swapped.extend_from_slice(&bytes[c..]);
    let (v, why, changed) = classify(&swapped, &sent, "swap");
    assert_eq!(
        v,
        Verdict::Refused,
        "a reordered pair must be refused: {why}"
    );
    assert!(!changed, "the refusal rewrote its input");

    let mut junk = bytes.clone();
    junk.extend_from_slice(&[0xFF; 30]);
    let (v, why, _) = classify(&junk, &sent, "junk");
    assert_eq!(
        v,
        Verdict::Exact,
        "thirty bytes of junk after the last record is a torn write and must be trimmed: {why}"
    );

    // ---- 4. malformed envelopes, rebuilt so the storage layer passes them --------------
    //
    // **The family that must reach `decode_envelope`, and the first attempt did not.**
    // Mutating a record's payload and repairing only its CRC leaves its chain hash stale, so
    // recovery refuses it at the storage layer and the decoder is never asked — the test then
    // passes for the wrong reason, which is the same class of defect it exists to catch. The
    // record is rebuilt properly instead: `Record::seal` over the mutated payload gives the
    // correct link, and the whole record is re-encoded. The **last** record is the one
    // rebuilt, so nothing downstream of it can break the chain, and the envelope inside it is
    // then the only thing wrong with the file.
    let rebuild_last = |payload: Vec<u8>| -> Vec<u8> {
        let last = starts[starts.len() - 1];
        let prev_hash = {
            // The parent of the last record is exactly the bytes its header already carries.
            let mut h = [0u8; 32];
            h.copy_from_slice(&bytes[last + 8 + 8..last + 8 + 8 + 32]);
            h
        };
        let batch_seq = u64::from_le_bytes(bytes[last + 8..last + 16].try_into().unwrap());
        let rec = Record::seal(batch_seq, prev_hash, payload);
        let body_len = (8 + 32 + 32 + rec.payload.len()) as u32;
        let mut out = bytes[..last].to_vec();
        out.extend_from_slice(&body_len.to_le_bytes());
        out.extend_from_slice(&(!body_len ^ 0xA5A5_A5A5).to_le_bytes());
        let body_at = out.len();
        out.extend_from_slice(&rec.batch_seq.to_le_bytes());
        out.extend_from_slice(&rec.parent);
        out.extend_from_slice(&rec.hash);
        out.extend_from_slice(&rec.payload);
        let crc = crc32_ieee(&out[body_at..]);
        out.extend_from_slice(&crc.to_le_bytes());
        out
    };

    // The last record's real envelope: count | (key_len, key, payload_len, payload)*.
    let last_payload = {
        let last = starts[starts.len() - 1];
        let len = u32::from_le_bytes(bytes[last..last + 4].try_into().unwrap()) as usize;
        bytes[last + 8 + 72..last + 8 + len].to_vec()
    };
    {
        // Sanity: rebuilding with the payload untouched must recover exactly, or every
        // assertion below would be testing the rebuild rather than the mutation.
        let (v, why, _) = classify(&rebuild_last(last_payload.clone()), &sent, "rebuild");
        assert_eq!(
            v,
            Verdict::Exact,
            "PRECONDITION UNMET: the rebuilt last record must be indistinguishable from the \
             original: {why}"
        );
    }

    let mut envelope_cases = 0usize;
    for (tag, delta) in [("count+1", 1i64), ("count-1", -1i64)] {
        let mut m = last_payload.clone();
        let count = u32::from_le_bytes(m[0..4].try_into().unwrap());
        let new = (count as i64 + delta).max(0) as u32;
        m[0..4].copy_from_slice(&new.to_le_bytes());
        let (v, why, changed) = classify(&rebuild_last(m), &sent, "env");
        envelope_cases += 1;
        assert_eq!(
            v,
            Verdict::Refused,
            "a record whose envelope declares the wrong transaction count ({tag}) passed the \
             storage layer and was accepted: {why}"
        );
        assert!(!changed, "the refusal rewrote its input");
    }

    // Trailing bytes: the declared transactions do not consume the payload.
    {
        let mut m = last_payload.clone();
        m.extend_from_slice(&[0u8; 7]);
        let (v, why, _) = classify(&rebuild_last(m), &sent, "trailing");
        envelope_cases += 1;
        assert_eq!(
            v,
            Verdict::Refused,
            "seven bytes nobody claimed followed the declared transactions and the envelope \
             was accepted anyway: {why}"
        );
    }

    // A key that is not the bytes that were written.
    {
        let mut m = last_payload.clone();
        // count(4) | key_len(4) | key…
        m[8] = 0xFF;
        let (v, why, _) = classify(&rebuild_last(m), &sent, "utf8");
        envelope_cases += 1;
        assert_eq!(
            v,
            Verdict::Refused,
            "an identity that is not the bytes that were written must be refused rather than \
             replaced with U+FFFD and carried on: {why}"
        );
    }

    // A key length that runs past the end of the envelope.
    {
        let mut m = last_payload.clone();
        m[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        let (v, why, _) = classify(&rebuild_last(m), &sent, "keylen");
        envelope_cases += 1;
        assert_eq!(
            v,
            Verdict::Refused,
            "a key length past the end of the envelope was accepted: {why}"
        );
    }

    assert_eq!(
        envelope_cases, 5,
        "PRECONDITION UNMET: the envelope family must run all five cases"
    );
}

/// **An undecodable envelope must refuse the *open*, not just the replay — F-62.**
///
/// The corpus above cannot see this on its own: it asks `open_bounded` and then
/// `recover_txns_checked`, and the second refuses whether or not the first did. What the
/// first one used to do is the defect. `open_bounded` called an unchecked wrapper —
/// `recover_seen_checked(..).unwrap_or_default()` — so on any envelope the decoder refused,
/// the idempotency window came back **empty** and a sequencer started with it. Every identity
/// committed before that restart was then new again, and the retry the window exists to
/// refuse would commit a second time. The daemon happened to survive because a later checked
/// replay returned `Err`; nothing else did.
///
/// So this asks the one question the corpus cannot: does the *sequencer* refuse to start?
#[test]
fn a_segment_whose_envelope_cannot_be_decoded_refuses_to_open_at_all() {
    let (bytes, sent) = fixture();
    let starts = record_starts(&bytes);
    let last = starts[starts.len() - 1];
    let len = u32::from_le_bytes(bytes[last..last + 4].try_into().unwrap()) as usize;
    let mut payload = bytes[last + 8 + 72..last + 8 + len].to_vec();
    // One more transaction declared than the envelope holds.
    let count = u32::from_le_bytes(payload[0..4].try_into().unwrap());
    payload[0..4].copy_from_slice(&(count + 1).to_le_bytes());

    let mut parent = [0u8; 32];
    parent.copy_from_slice(&bytes[last + 16..last + 48]);
    let batch_seq = u64::from_le_bytes(bytes[last + 8..last + 16].try_into().unwrap());
    let rec = Record::seal(batch_seq, parent, payload);
    let body_len = (72 + rec.payload.len()) as u32;
    let mut out = bytes[..last].to_vec();
    out.extend_from_slice(&body_len.to_le_bytes());
    out.extend_from_slice(&(!body_len ^ 0xA5A5_A5A5).to_le_bytes());
    let body_at = out.len();
    out.extend_from_slice(&rec.batch_seq.to_le_bytes());
    out.extend_from_slice(&rec.parent);
    out.extend_from_slice(&rec.hash);
    out.extend_from_slice(&rec.payload);
    let crc = crc32_ieee(&out[body_at..]);
    out.extend_from_slice(&crc.to_le_bytes());

    let owned = Owned::new("open-refuses");
    let p = owned.path();
    std::fs::write(&p, &out).expect("write");
    match Sequencer::open_bounded(&p, SyncPolicy::Always, None) {
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("idempotency window"),
                "the refusal must name what could not be rebuilt: {msg}"
            );
        }
        Ok((s, recovery)) => {
            let recovered = Sequencer::recover_txns_checked(&recovery);
            let window_len = recovery.records.len();
            s.shutdown();
            panic!(
                "the sequencer started over a segment whose envelope does not decode. Its \
                 window was rebuilt from {window_len} record(s) by a wrapper that answers \
                 `unwrap_or_default()`, so every one of the {} identities committed before \
                 this restart is new to it again. The replay's own verdict was {:?}, which \
                 nobody had to ask for.",
                sent.len(),
                recovered.map(|v| v.len())
            );
        }
    }
    let _ = std::fs::remove_file(&p);
}

/// CRC-32 (IEEE), the same polynomial the segment uses. Written out here rather than exposed
/// from the crate: a test that shares the implementation it is checking against can be wrong
/// in the same way twice.
fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc ^= *b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    crc ^ 0xFFFF_FFFF
}

// ── fixture ownership, and the failure it explains ─────────────────────────────────────────

/// **The old helper's naming scheme, reproduced so the collision can be demonstrated rather
/// than assumed.**
///
/// Identical to what `tmp` did: process id, tag, and a clock reading. The clock is supplied
/// so the case is a *controlled* one — the Mac failure appeared on 1 of 3 eight-thread runs,
/// which is scheduler luck, and a test that depends on scheduler luck to reproduce a defect
/// cannot be used to say the defect is understood.
fn old_scheme(tag: &str, clock_nanos: u128) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "niles-torn-{}-{tag}-{clock_nanos}",
        std::process::id()
    ))
}

/// **The cause, demonstrated.** Two allocations that read the clock within one tick, under
/// the same tag, produce the same path — and the old helper then deleted it.
///
/// This is what makes the Host C failure explained rather than merely consistent with the
/// evidence. `fixture()` and `classify()` both allocated under fixed tags (`base` and the
/// case's own), eight test threads ran them concurrently, and `std::fs::remove_file` on a
/// path the caller had not created is what turned a name collision into a missing file at
/// `read`. It reproduces here with no threads at all, because the clock is an input.
#[test]
fn the_old_naming_scheme_collides_within_one_clock_tick_and_deletes_the_other_fixture() {
    let frozen = 1_700_000_000_000_000_000u128;
    let a = old_scheme("base", frozen);
    let b = old_scheme("base", frozen);
    assert_eq!(a, b, "two allocations in one tick name the same file");

    // The first "test" writes its fixture.
    std::fs::write(&a, b"the first fixture").expect("write");
    // The second allocates, and the old helper's last line runs.
    let _ = std::fs::remove_file(&b);
    assert!(
        std::fs::read(&a).is_err(),
        "the second allocation deleted the first one's fixture, which is `exit 101` at the \
         next `read` — and this is why the repair is ownership and not a longer name"
    );
    let _ = std::fs::remove_file(&a);
}

/// The same shape against the new allocator: same tag, no clock, and the two are different.
#[test]
fn two_allocations_under_one_tag_are_two_directories() {
    let a = Owned::new("base");
    let b = Owned::new("base");
    assert_ne!(a.dir, b.dir);
    std::fs::write(a.path(), b"a").expect("write a");
    std::fs::write(b.path(), b"b").expect("write b");
    // Neither can name the other's file, and neither deletes it.
    assert_eq!(std::fs::read(a.path()).expect("a survives"), b"a");
    assert_eq!(std::fs::read(b.path()).expect("b survives"), b"b");
}

/// **A forced name collision is refused, not resolved by deletion.**
///
/// The allocator's directory is created in the way that makes ownership provable, so the test
/// takes the name it is about to ask for and checks that the allocator steps past it with the
/// squatter's contents intact.
#[test]
fn a_taken_name_is_stepped_over_and_its_contents_are_untouched() {
    // Allocate once to learn the shape of the next name, then squat it.
    // The counter is per tag, so `squat`'s sequence belongs to this test alone and the name
    // below really is the next one the allocator will try. With one global counter it was
    // not, and this test passed against a deliberately broken allocator.
    let probe = Owned::new("squat");
    let n: u64 = probe
        .dir
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.rsplit('-').next())
        .and_then(|s| s.parse().ok())
        .expect("the suffix is the counter");
    let next =
        std::env::temp_dir().join(format!("niles-torn-{}-squat-{}", std::process::id(), n + 1));
    std::fs::create_dir(&next).expect("squat the next name");
    std::fs::write(next.join("segment"), b"not yours").expect("write the squatter's file");

    let taken = Owned::new("squat");
    assert_ne!(
        taken.dir, next,
        "the allocator must not take a name it did not create"
    );
    assert_eq!(
        std::fs::read(next.join("segment")).expect("the squatter's file is still there"),
        b"not yours",
        "a collision was resolved by deleting somebody else's file, which is the defect"
    );
    let _ = std::fs::remove_dir_all(&next);
}

/// Concurrent allocation under one tag, with no reliance on scheduling to expose a defect:
/// every thread writes a value only it knows and reads it back.
#[test]
fn concurrent_allocations_under_one_tag_do_not_disturb_each_other() {
    let handles: Vec<_> = (0..16u8)
        .map(|i| {
            std::thread::spawn(move || {
                let owned = Owned::new("base");
                let mine = vec![i; 32];
                std::fs::write(owned.path(), &mine).expect("write");
                // Long enough that every thread is between its write and its read at once,
                // which is the window the old helper's `remove_file` landed in.
                std::thread::sleep(std::time::Duration::from_millis(20));
                let back = std::fs::read(owned.path()).expect("my own fixture is still mine");
                assert_eq!(back, mine, "thread {i} read somebody else's fixture");
            })
        })
        .collect();
    for h in handles {
        h.join().expect("no thread lost its fixture");
    }
}
