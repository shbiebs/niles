//! **The schema of the machine-readable line, asserted where it is written.**
//!
//! A harness that greps prose has its interface in its output formatting, where nothing
//! checks it. These tests move that interface into a schema and then pin it, so that the
//! human-readable sentence above each summary line is free to be reworded — which it will be,
//! because it is written for people — without any risk of moving a verdict.
//!
//! The four outcomes a field can have (**missing**, **`n/a`**, **zero**, **malformed**) are
//! asserted here on four fixture transcripts. Each of the four corresponds to a wrong number
//! this repository actually printed: a renamed server column read as `unwrap_or(0)` for four
//! cycles (missing read as zero), `deferred_merges` expected to be zero and therefore easy to
//! mistake for unreported (zero read as `n/a`), a flight-counter reading that failed and
//! rendered as zeroes (`n/a` read as zero), and a truncated log whose last line was half
//! written (malformed read as a value).

use bank_bench::summary::{self, Field, Line};
use bank_bench::workloads::{self, MIXED_CSV_HEADER};
use std::time::Duration;

fn sample(not_run: Option<&str>) -> workloads::MixedSample {
    workloads::MixedSample {
        target: "nilestream".into(),
        run: 1,
        readers: 6,
        writers: 3,
        reads: 3_848_491,
        writes: 15_028,
        wall: Duration::from_millis(30_015),
        read_p50: Duration::from_micros(40),
        read_p99: Duration::from_micros(113),
        write_p50: Duration::from_micros(5_989),
        write_p99: Duration::from_micros(8_264),
        fallback_rate: Some(0.0),
        max_batch: Some(5),
        lock_wait_p99_us: Some(3),
        base_epochs: Some(34_027),
        duplicates: 0,
        errors: 0,
        not_run: not_run.map(|s| s.to_string()),
    }
}

#[test]
fn a_mixed_line_carries_exactly_the_csv_header_fields_and_no_others() {
    let l = Line::parse(&sample(None).summary_line()).expect("a summary line");
    assert_eq!(l.kind, "mixed");
    let mut want: Vec<&str> = MIXED_CSV_HEADER.split(',').collect();
    want.sort_unstable();
    assert_eq!(l.names(), want);
    assert_eq!(
        summary::schema("mixed").expect("mixed has a schema").len(),
        MIXED_CSV_HEADER.split(',').count(),
        "the schema table and the CSV header are the same list, or one of them is wrong"
    );
}

#[test]
fn the_csv_row_and_the_summary_line_are_two_renderings_of_one_field_list() {
    let s = sample(None);
    let cells: Vec<String> = s.to_csv().split(',').map(|c| c.to_string()).collect();
    let l = Line::parse(&s.summary_line()).expect("a summary line");
    for (name, cell) in MIXED_CSV_HEADER.split(',').zip(&cells) {
        assert_eq!(
            l.text(name).as_deref(),
            Some(cell.as_str()),
            "field `{name}` differs between the CSV row and the summary line"
        );
    }
}

#[test]
fn a_level_that_did_not_run_still_prints_every_field() {
    // The refusal path is the one a harness most needs to parse and the one least likely to
    // be exercised by hand. A `not_run` level that printed a shorter line would be a level a
    // reader could only learn about from prose.
    let l = Line::parse(&sample(Some("port 7431 was occupied")).summary_line()).expect("a line");
    let mut want: Vec<&str> = MIXED_CSV_HEADER.split(',').collect();
    want.sort_unstable();
    assert_eq!(l.names(), want);
    assert_eq!(
        l.text("not_run").as_deref(),
        Some("port 7431 was occupied"),
        "a reason with spaces in it survives the line, rather than being cut at the first one"
    );
    assert_eq!(l.num("reads"), Field::Num(3_848_491.0));
}

#[test]
fn an_absent_optional_is_n_a_and_never_zero() {
    let mut s = sample(None);
    s.fallback_rate = None;
    s.base_epochs = None;
    let l = Line::parse(&s.summary_line()).expect("a line");
    assert_eq!(l.num("fallback_rate"), Field::NotMeasured);
    assert_eq!(l.num("base_epochs"), Field::NotMeasured);
    // And the measured zero next to them stays a zero.
    assert_eq!(l.num("duplicates"), Field::Num(0.0));
}

/// The four fixture transcripts. Each is a log as a harness would read it.
fn fixtures() -> Vec<(&'static str, String)> {
    let zero = {
        let mut s = sample(None);
        s.fallback_rate = Some(0.0);
        format!(
            "  E19 mixed nilestream 6r/3w run 1 — 128218 reads/s, fallback 0.00%\n{}\n",
            s.summary_line()
        )
    };
    let na = {
        let mut s = sample(None);
        s.fallback_rate = None;
        format!(
            "  E19 mixed nilestream 6r/3w run 1 — 128218 reads/s, fallback REFUSED\n{}\n",
            s.summary_line()
        )
    };
    // A build that predates the field: the key is simply not there.
    let missing = {
        let hdr: Vec<&str> = MIXED_CSV_HEADER
            .split(',')
            .filter(|n| *n != "fallback_rate")
            .collect();
        let vals: Vec<String> = MIXED_CSV_HEADER
            .split(',')
            .zip(sample(None).csv_fields())
            .filter(|(n, _)| *n != "fallback_rate")
            .map(|(_, v)| v)
            .collect();
        format!("{}\n", summary::line("mixed", &hdr.join(","), &vals))
    };
    // A log cut off mid-write: the last token has a key and no value.
    let malformed = {
        let l = sample(None).summary_line();
        let cut = l.replace("fallback_rate=0.0000", "fallback_rate=");
        format!("{cut}\n")
    };
    vec![
        ("zero", zero),
        ("n/a", na),
        ("missing", missing),
        ("malformed", malformed),
    ]
}

#[test]
fn the_four_outcomes_are_distinguished_by_four_fixture_logs() {
    let mut seen = Vec::new();
    for (name, text) in fixtures() {
        let l = Line::all_of(&text, "mixed")
            .pop()
            .unwrap_or_else(|| panic!("{name}: no mixed line"));
        let f = l.num("fallback_rate");
        seen.push(f.clone());
        match (name, &f) {
            ("zero", Field::Num(n)) => assert_eq!(*n, 0.0),
            ("n/a", Field::NotMeasured) => {}
            ("missing", Field::Missing) => {}
            ("malformed", Field::Malformed(t)) => assert!(t.is_empty(), "{t:?}"),
            _ => panic!("{name} resolved to {f:?}, which is one of the other three"),
        }
    }
    // Four fixtures, four distinct outcomes: no two collapse into each other.
    for i in 0..seen.len() {
        for j in (i + 1)..seen.len() {
            assert_ne!(seen[i], seen[j], "outcomes {i} and {j} are the same value");
        }
    }
}

#[test]
fn rewording_the_human_line_changes_nothing_a_reader_parses() {
    let s = sample(None);
    let machine = s.summary_line();
    let a = format!("  E19 mixed nilestream 6r/3w run 1 — 128218 reads/s\n{machine}\n");
    let b = format!(
        "  [level 6r/3w] nilestream answered 128,218 reads a second; writers made progress\n\
         {machine}\n"
    );
    let pa = Line::all_of(&a, "mixed").pop().expect("a");
    let pb = Line::all_of(&b, "mixed").pop().expect("b");
    for name in MIXED_CSV_HEADER.split(',') {
        assert_eq!(pa.text(name), pb.text(name), "field `{name}`");
    }
}

#[test]
fn every_kind_this_crate_emits_has_a_schema_and_a_writer() {
    // `flights` and `score` are written by the binary rather than by the library, so this
    // test asserts the schema exists and is well formed; the binary's own output is asserted
    // by `score_c10_fixture.rs` and by the harness self-test arms.
    for k in summary::KINDS {
        let s = summary::schema(k).unwrap_or_else(|| panic!("{k} has no schema"));
        assert!(!s.is_empty(), "{k}");
        assert!(
            s.iter().all(|n| !n.contains(' ') && !n.contains('=')),
            "{k}: a field name with a space or an `=` in it would not survive the line format"
        );
    }
    assert_eq!(summary::schema("no-such-kind"), None);
}
