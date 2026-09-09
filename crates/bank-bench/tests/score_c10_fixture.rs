//! **The scorer reproduces cycle 10's Host C table, from cycle 10's own CSVs, to the integer.**
//!
//! `tests/fixtures/c10-hostc-t04.2/` is the twenty per-replicate `mixed.csv` files the T04.2
//! comparison was measured into on Host C (`~/c10-baselock-out`, five measured replicates ×
//! two arms × two working points), copied unaltered. Nothing here is generated and nothing is
//! rounded on the way in: the header is the one this build writes, and every cell is the cell
//! the daemon wrote in October.
//!
//! # What it guards, and why a fixture rather than an assertion about arithmetic
//!
//! The pooled MAD is `sqrt((a² + b²)/2)`. Cycle 10's report asked its reader to compute it,
//! and two auditors then did — one of them getting six wrong numbers by taking the *mean* of
//! the two MADs instead of their RMS (F-11-02, and A10-09 before it, which is the same
//! mistake made once already). A unit test on `pooled_mad(6, 8)` catches that particular
//! slip. It does not catch the scorer being wired to the wrong column, grouping two working
//! points together, silently dropping a replicate whose CSV would not parse, or picking the
//! other arm as the reference and inverting every sign — and each of those would produce a
//! plausible table.
//!
//! So the guard is end to end and against numbers that were published before the scorer
//! existed: read the directory, group it, score it, and compare against the six rows in the
//! cycle-11 audit's §0.3 table, which were themselves recomputed from these same raw logs.
//!
//! # The writer rows are here too, and they say something the read rows do not
//!
//! A11-07 reported the merge arm costing ~30% of writer throughput and 53–80% of write p99.
//! Scoring the same CSVs shows that regression is **entirely at the partial working point**:
//! at `full`, writer throughput and write p99 are within the gate on every level. That is a
//! narrower and more useful claim than the one the audit made, and it is here as a row rather
//! than a sentence so that it cannot quietly stop being true.

use bank_bench::score::{self, Verdict};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/c10-hostc-t04.2")
}

fn row<'a>(
    scored: &'a [score::Scored],
    point: &str,
    level: &str,
    metric: &str,
) -> &'a score::Scored {
    scored
        .iter()
        .find(|r| r.point == point && r.level == level && r.metric == metric)
        .unwrap_or_else(|| panic!("no scored row for {point} {level} {metric}"))
}

/// Every row of the published table, as `(point, level, median_merge, median_pinned, rel %,
/// pooled MAD, MADs apart, fires)`.
///
/// The medians and pooled MADs are stated to the integer, the percentages to two places, and
/// the MADs to one — the precision each was published at. A tolerance of half an integer unit
/// is applied to the medians and pooled MADs (inclusive, since a value ending in .5 rounds
/// to the integer above) so this table can be read against the audit
/// document without transcribing more digits than the document carries.
type PublishedRow = (&'static str, &'static str, f64, f64, f64, f64, f64, bool);

const T04_2_READS: &[PublishedRow] = &[
    (
        "full", "6r/3w", 132_430.0, 145_466.0, -8.96, 1_311.0, 9.9, false,
    ),
    (
        "full", "9r/5w", 135_738.0, 153_087.0, -11.33, 1_908.0, 9.1, true,
    ),
    (
        "full", "12r/6w", 143_117.0, 155_821.0, -8.15, 1_363.0, 9.3, false,
    ),
    (
        "partial", "6r/3w", 123_859.0, 133_341.0, -7.11, 1_024.0, 9.3, false,
    ),
    (
        "partial", "9r/5w", 118_986.0, 136_482.0, -12.82, 1_047.0, 16.7, true,
    ),
    (
        "partial", "12r/6w", 118_949.0, 136_161.0, -12.64, 1_197.0, 14.4, true,
    ),
];

#[test]
fn the_scorer_reproduces_the_six_t04_2_read_rows_to_the_integer() {
    let (rows, problems) = score::read_rows(&fixture());
    assert!(
        problems.is_empty(),
        "the fixture did not read cleanly: {problems:?}"
    );
    assert_eq!(rows.len(), 60, "20 replicate files × 3 levels");
    let scored = score::score(&rows);

    for (point, level, m_merge, m_pinned, rel, pmad, mads, fires) in T04_2_READS {
        let r = row(&scored, point, level, "reads_per_second");
        // Arm A is the reference and the scorer prefers `pinned` over `merge` by name, which
        // is what makes the published sign (merge slower than pinned) come out negative.
        assert_eq!(
            r.arm_a, "pinned",
            "{point} {level}: the reference arm moved"
        );
        assert_eq!(r.arm_b, "merge");
        assert_eq!((r.n_a, r.n_b), (5, 5), "{point} {level}");
        assert!(
            (r.median_a - m_pinned).abs() <= 0.5,
            "{point} {level}: pinned median {}, published {m_pinned}",
            r.median_a
        );
        assert!(
            (r.median_b - m_merge).abs() <= 0.5,
            "{point} {level}: merge median {}, published {m_merge}",
            r.median_b
        );
        assert!(
            (r.rel_change * 100.0 - rel).abs() < 0.005,
            "{point} {level}: rel {:.4}%, published {rel}%",
            r.rel_change * 100.0
        );
        assert!(
            (r.pooled_mad - pmad).abs() < 1.0,
            "{point} {level}: pooled MAD {:.1}, published {pmad}. The mean of the two MADs \
             would be a different number and this is the assertion that says which one this \
             scorer computes.",
            r.pooled_mad
        );
        assert!(
            (r.mads_apart - mads).abs() < 0.05,
            "{point} {level}: {:.2} MADs apart, published {mads}",
            r.mads_apart
        );
        let expect = if *fires {
            Verdict::Fires
        } else {
            Verdict::NoiseLimited
        };
        assert_eq!(
            r.verdict,
            expect,
            "{point} {level}: rel {:.2}% and {:.1} MADs against a gate of {:.0}% and {:.0} MADs",
            r.rel_change * 100.0,
            r.mads_apart,
            score::GATE_REL * 100.0,
            score::GATE_MADS
        );
        assert_eq!(
            r.direction, "worse",
            "{point} {level}: the merge arm reads slower"
        );
    }

    // Three of six clear both halves; all six exceed three MADs. The cycle-10 report said
    // exactly this and it is the sentence the fixture confirms.
    let fired = T04_2_READS.iter().filter(|r| r.7).count();
    assert_eq!(fired, 3);
    assert!(T04_2_READS.iter().all(|r| r.6 >= 3.0));
}

#[test]
fn the_fixture_can_tell_the_rms_from_the_mean_and_says_on_how_many_rows() {
    // **Why the wrong pooled MAD survived two reviews.** The RMS and the mean of two numbers
    // agree exactly when the two are equal and diverge as they separate. On this fixture's
    // `full 6r/3w` row the two arms' MADs are 1357.3 and 1264.3, so the mean is 1310.8 and the
    // RMS 1311.6 — a difference of 0.8 in a number published as `1,311`. A reader checking
    // that row could not have told which formula produced it, and neither could a test with a
    // one-unit tolerance.
    //
    // So this asserts the discriminating power directly rather than assuming it: at least
    // four of the six rows must separate the two formulae by more than the tolerance the
    // table above allows. If a future fixture were to lose that, the guard on
    // `pooled_mad` would be passing by luck and this test says so instead.
    let (rows, _) = score::read_rows(&fixture());
    let scored = score::score(&rows);
    let mut discriminating = 0;
    let mut report = Vec::new();
    for (point, level, _, _, _, published, _, _) in T04_2_READS {
        let r = row(&scored, point, level, "reads_per_second");
        let mean = (r.mad_a + r.mad_b) / 2.0;
        let rms = r.pooled_mad;
        if (mean - published).abs() >= 1.0 {
            discriminating += 1;
        }
        report.push(format!(
            "{point} {level}: mads {:.1}/{:.1}, rms {rms:.1}, mean {mean:.1}, published {published}",
            r.mad_a, r.mad_b
        ));
    }
    assert!(
        discriminating >= 4,
        "only {discriminating} of 6 rows separate the RMS from the mean by more than this \
         table's tolerance, so the pooled-MAD guard is largely passing by coincidence:\n  {}",
        report.join("\n  ")
    );
}

#[test]
fn the_writer_rows_show_the_regression_is_at_the_partial_point_only() {
    let (rows, _) = score::read_rows(&fixture());
    let scored = score::score(&rows);

    // A11-07's headline: ~30% of writer throughput, 53–80% of write p99. Both hold, and both
    // hold at `partial`.
    for (level, rel_lo, rel_hi) in [("6r/3w", -31.0, -29.0), ("9r/5w", -32.5, -30.5)] {
        let r = row(&scored, "partial", level, "writes_per_second");
        let pct = r.rel_change * 100.0;
        assert!(
            pct > rel_lo && pct < rel_hi,
            "partial {level} writer throughput {pct:.2}%, expected between {rel_lo} and {rel_hi}"
        );
        assert_eq!(
            r.verdict,
            Verdict::Fires,
            "partial {level} writer throughput"
        );
        assert_eq!(r.direction, "worse");
    }
    for (level, lo, hi) in [
        ("6r/3w", 52.0, 54.0),
        ("9r/5w", 79.0, 80.0),
        ("12r/6w", 73.0, 74.0),
    ] {
        let r = row(&scored, "partial", level, "write_p99_us");
        let pct = r.rel_change * 100.0;
        assert!(pct > lo && pct < hi, "partial {level} write p99 {pct:.2}%");
        assert_eq!(r.verdict, Verdict::Fires, "partial {level} write p99");
        assert_eq!(r.direction, "worse", "a longer write tail is worse");
    }

    // And the correction the audit did not make: at `full`, not one writer row clears the
    // gate. `full 12r/6w` moves 18.9% — over the relative half — at 1.4 pooled MADs, which is
    // the shape a reader quoting only the percentage would have called a regression.
    for level in ["6r/3w", "9r/5w", "12r/6w"] {
        for metric in ["writes_per_second", "write_p99_us"] {
            let r = row(&scored, "full", level, metric);
            assert_eq!(
                r.verdict,
                Verdict::NoiseLimited,
                "full {level} {metric}: {:.2}% at {:.2} MADs — if this now fires, the claim \
                 'the writer regression is a partial-point effect' has stopped being true and \
                 the audit's §2.3 synthesis needs rewriting, not this test",
                r.rel_change * 100.0,
                r.mads_apart
            );
        }
    }
}

#[test]
fn an_arm_short_of_five_replicates_is_refused_and_the_row_still_appears() {
    // The scorer's floor is five. Read the fixture, drop one `merge partial` replicate, and
    // the three partial rows must come back refused — not omitted, which would leave a table
    // whose length depends on its own failures.
    let (rows, _) = score::read_rows(&fixture());
    let kept: Vec<score::Row> = rows
        .into_iter()
        .filter(|r| !(r.arm == "merge" && r.point == "partial" && r.rep == "5"))
        .collect();
    let scored = score::score(&kept);
    let r = row(&scored, "partial", "6r/3w", "reads_per_second");
    match &r.verdict {
        Verdict::Refused(why) => assert!(why.contains("4 of 4"), "{why}"),
        v => panic!("expected a refusal, got {v:?}"),
    }
    // The `full` point is untouched and still scores.
    assert_eq!(
        row(&scored, "full", "9r/5w", "reads_per_second").verdict,
        Verdict::Fires
    );
}

#[test]
fn the_fixture_carries_the_header_this_build_writes() {
    // If `MIXED_CSV_HEADER` changes, this fixture stops parsing and says so here, rather
    // than being read column-by-position into a table of the wrong quantities. The fixture is
    // a measurement from a host that no longer exists in that state; it can never be
    // regenerated, so a schema change means the *comparison* must be marked historical, never
    // that the file may be hand-edited to the new header.
    let (rows, problems) = score::read_rows(&fixture());
    assert!(
        problems.is_empty(),
        "the cycle-10 fixture no longer parses against this build's schema: {problems:?}"
    );
    assert!(rows.iter().all(|r| r.target == "nilestream"));
    assert!(rows.iter().all(|r| r.not_run.is_none()));
}
