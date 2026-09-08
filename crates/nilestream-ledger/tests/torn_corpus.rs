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

fn tmp(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "niles-torn-{}-{tag}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_file(&p);
    p
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
    let p = tmp(tag);
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
    let p = tmp("base");
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
    let _ = std::fs::remove_file(&p);
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

    let p = tmp("open-refuses");
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
