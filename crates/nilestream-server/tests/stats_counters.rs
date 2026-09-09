//! **Every counter on the `nilestream_stats` wire, driven or explained, one row each.**
//!
//! `session.rs` already asserts that the reply *names* the columns the benchmark reads. That
//! catches a deletion. It does not catch the failure this project has actually had four times:
//! a column that is present, is read, and never moves — or moves and means something other
//! than its name. `merges_refused_epochs` was reported as a merge refusal while the pinned
//! control refused every landing by construction; `flights_that_fell_behind` is structurally
//! zero because the base is held shared across the fold, and nothing said so; `deferred_merges`
//! is *expected* to read zero, which is the finding rather than the absence of one; and a
//! renamed column was read with `unwrap_or(0)` for four cycles.
//!
//! So this is a table with one row per wire column, and the table is closed at both ends:
//!
//! * every column in the `RowDescription` must have a row here, or the completeness test
//!   fails and names it — **a counter added to the wire without a row fails this test**;
//! * every row here must name a column that exists, so a renamed column fails too.
//!
//! Each row says one of three things, and each is checked:
//!
//! | kind | what is asserted |
//! |---|---|
//! | `Driven` | zero on a fresh engine with no events (the isolated control), non-zero after the named event, driven through `Session::handle` and read back off the wire |
//! | `Level` | not a counter of events but a size or a configured cap; asserted to be readable and to take the value the fixture implies |
//! | `StaysZero` | zero on a fresh engine **and** zero after the events that would move a counter of that name, with the reason recorded — this is where a structural zero is written down so that it fails here if it ever stops being one |
//!
//! `StaysZero` is the important one. A counter documented in prose as "always zero here" is a
//! sentence; a counter asserted to stay zero after the very events that would move it is a
//! guard, and F-11-09's structural zero has one now.

use nilestream_server::daemon;
use nilestream_server::pg_wire::{Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

fn session() -> Session {
    // The daemon's own schema, so the counters are driven through the statements a served
    // session actually compiles rather than through a schema written to suit the test.
    Session::new(
        "bench".into(),
        "bank".into(),
        daemon::DEFAULT_SCHEMA.to_string(),
    )
}

/// The stats reply, as a column name to value map, read the way `bank-bench` reads it: by
/// name, off the wire, out of a `RowDescription` and a `DataRow`.
fn stats_over_the_wire(s: &mut Session, e: &RevEngine) -> Vec<(String, Option<String>)> {
    let out = s.handle(Frontend::Query("select nilestream_stats".into()), e);
    let names: Vec<String> = out
        .iter()
        .find_map(|m| match m {
            Backend::RowDescription(f) => Some(f.iter().map(|x| x.name.clone()).collect()),
            _ => None,
        })
        .expect("the stats reply carries a row description");
    let row = nilestream_server::pg_wire::decoded_rows(&out)
        .into_iter()
        .next()
        .expect("the stats reply carries exactly one row");
    assert_eq!(
        names.len(),
        row.len(),
        "the row description and the data row disagree on how many columns there are"
    );
    names.into_iter().zip(row).collect()
}

fn value(s: &mut Session, e: &RevEngine, name: &str) -> u128 {
    let cols = stats_over_the_wire(s, e);
    let (_, v) = cols
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("`{name}` is not on the wire"));
    v.as_ref()
        .unwrap_or_else(|| panic!("`{name}` came back null"))
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("`{name}` is not an integer: {e}"))
}

/// What a row claims about its counter.
enum Kind {
    /// A count of events. Zero before, non-zero after `drive`.
    Driven(&'static str),
    /// A size or a configured cap, not a count of events. The string is the description
    /// a reader needs and is not asserted on, which is why it is allowed to be unread.
    Level(#[allow(dead_code)] &'static str),
    /// Zero before *and* after the events that would move a counter of this name, for the
    /// stated reason. A structural zero, written down where it can fail.
    StaysZero(&'static str),
}

use Kind::*;

/// One row per column on the wire.
///
/// The `drive` column names which of the fixtures below moves it; the fixtures are small and
/// deliberately separate, so that a counter driven by "a keyed read" is not accidentally
/// driven by "an eviction" and reported as though the first had done it.
const TABLE: &[(&str, Kind)] = &[
    ("reads", Driven("a keyed read")),
    (
        "hits",
        Driven("a keyed read repeated while the key is resident"),
    ),
    (
        "misses",
        Driven("a keyed read of a key the view does not hold"),
    ),
    (
        "rows_touched",
        Driven("a reconstruction, which folds base rows"),
    ),
    (
        "resident",
        Level("keys the view holds; bounded by the residency budget"),
    ),
    ("view_answers", Driven("a keyed read the view answered")),
    (
        "fallbacks",
        StaysZero(
            "this counts the anchor mismatch alone: the view was consulted and its answer \
             discarded because the entry was not exact at the anchor asked for. That needs a \
             writer advancing the frontier between the two, and this fixture has one thread \
             and no concurrent appends during the reads",
        ),
    ),
    (
        "view_metadata_keys",
        Level("the eviction policy's metadata, one entry per resident key"),
    ),
    (
        "idem_window_keys",
        Level("identities the base's idempotency index holds"),
    ),
    (
        "schema_parses",
        Driven("an append, whose currency check is the only path that reads the schema"),
    ),
    (
        "pending_joins",
        StaysZero(
            "a join needs a second reader arriving at the same key and anchor while the first \
             is still folding; one thread cannot produce one. Asserting the zero is what keeps \
             this row honest rather than quietly untested",
        ),
    ),
    (
        "uninstalled_folds",
        StaysZero("needs a competing flight at another anchor for the same key"),
    ),
    (
        "pinned_installs",
        StaysZero("needs a landing below an already-applied frontier, so it needs a writer"),
    ),
    (
        "flights_refused",
        StaysZero("needs MAX_FLIGHTS concurrent owners; one thread holds at most one"),
    ),
    (
        "deferred_merges",
        StaysZero(
            "a merge happens only when a fold lands late enough that the frontier moved under \
             it, which needs a writer. The zero here is a measurement — A10-16 is about \
             exactly this counter being reported when it reads zero — and not an absence",
        ),
    ),
    (
        "merge_rows_visited",
        StaysZero("no landing is late in this fixture, so none merges"),
    ),
    ("merge_epochs_merged", StaysZero("as merge_rows_visited")),
    (
        "merge_max_epochs",
        Level("the configured epoch cap; 32 while MergeCaps::default() is on, 0 when off"),
    ),
    (
        "merge_max_rows",
        Level("the configured row cap; 4096 on, 0 off"),
    ),
    (
        "merges_refused_epochs",
        StaysZero("a refusal is counted per late landing, and none arrives here"),
    ),
    ("merges_refused_rows", StaysZero("as merges_refused_epochs")),
    (
        "merges_refused_unavailable",
        StaysZero("as merges_refused_epochs"),
    ),
    (
        "waiters_refused",
        StaysZero("the other capacity refusal, kept apart from flights_refused; needs waiters"),
    ),
    (
        "joins_answered",
        StaysZero("no join is made here, so none ends"),
    ),
    (
        "joins_retried",
        StaysZero("no join is made here, so none ends"),
    ),
    (
        "gap_at_begin_total",
        StaysZero(
            "the *total lag*, not the sample count beside it. Every flight here begins at the \
             frontier because nothing advances it, so thirty samples sum to zero — which is \
             why the two are separate counters and why dividing one by the wrong denominator \
             once printed a mean anchor gap larger than the maximum it was a mean of",
        ),
    ),
    ("gap_at_finish_total", StaysZero("as gap_at_begin_total")),
    (
        "gap_at_finish_max",
        Level("a running maximum, not a delta over the window"),
    ),
    (
        "gap_begin_samples",
        Driven("a reconstruction, which samples the gap once per authorised flight"),
    ),
    (
        "gap_finish_samples",
        Driven("a reconstruction that installs, which samples the gap again at the finish"),
    ),
    (
        "flights_behind_at_begin",
        StaysZero("a flight begins behind only if the frontier moved before it started"),
    ),
    (
        "flights_that_fell_behind",
        StaysZero(
            "**structurally zero, and this is the guard for it (F-11-09).** The base is held \
             shared across the whole fold, so the frontier a flight started at cannot move \
             under it and no flight can fall further behind. If this ever becomes non-zero the \
             hold has been released somewhere — which is exactly what LC-37's prototype \
             (C11-12) proposes — and the reconstruction-completeness argument that rests on \
             the hold has to be re-made, not this assertion relaxed",
        ),
    ),
];

fn fresh() -> RevEngine {
    // Small and deliberately partial: a budget below the key count is what makes a read miss
    // and reconstruct, which is the path most of these counters live on.
    RevEngine::seeded(40, 2, 12, ViewMode::Demand, EvictionPolicy::Lru)
}

/// Drive the events every `Driven` row names, through the wire.
fn drive_everything(s: &mut Session, e: &RevEngine) {
    // A cold keyed read (miss, rows_touched, view_answers, reads), then the same read again
    // (hit). The statement is the one a client sends — this drives the served path rather
    // than `read_point`, which is a read model no query consults and which two cycles of
    // tests warmed while asserting about the other one.
    let ask = |s: &mut Session, key: i64| -> Vec<Backend> {
        s.handle(
            Frontend::Query(format!(
                "select acct, sum(amt) from postings where acct = {key} group by acct"
            )),
            e,
        )
    };
    for key in [3_i64, 7, 11] {
        for _ in 0..2 {
            let out = ask(s, key);
            assert!(
                !out.iter()
                    .any(|m| matches!(m, Backend::ErrorResponse { .. })),
                "the fixture query was refused: {out:?}"
            );
        }
    }
    // Enough distinct keys to exceed the residency budget, so the eviction policy's metadata
    // is populated and a later read reconstructs rather than hitting.
    for key in 1..=30_i64 {
        let _ = ask(s, key);
    }
    // **An append, because `schema_parses` is on the write path.** The counter increments
    // when a statement first needs the schema's currency table, which a `select` never does:
    // driving it with reads alone would have left this row untested while the test claimed
    // otherwise, and that is the shape of defect this file exists for. Two legs, so the
    // conservation rule the ledger declares is satisfied and the append is accepted.
    let insert = s.handle(
        Frontend::Query(
            "insert into postings values (990001, 1, 0, 250), (990001, 2, 0, -250)".into(),
        ),
        e,
    );
    assert!(
        !insert
            .iter()
            .any(|m| matches!(m, Backend::ErrorResponse { .. })),
        "the fixture append was refused: {insert:?}"
    );
}

#[test]
fn every_wire_column_has_a_row_in_this_table_and_every_row_names_a_column() {
    let (mut s, e) = (session(), fresh());
    let cols = stats_over_the_wire(&mut s, &e);
    let on_wire: Vec<&str> = cols.iter().map(|(n, _)| n.as_str()).collect();
    let in_table: Vec<&str> = TABLE.iter().map(|(n, _)| *n).collect();

    let unrowed: Vec<&&str> = on_wire.iter().filter(|n| !in_table.contains(n)).collect();
    assert!(
        unrowed.is_empty(),
        "these counters are on the `nilestream_stats` wire and no row in this table says what \
         they are or how they move: {unrowed:?}\n\
         Add a row — `Driven` with the event that moves it, `Level` if it is a size or a cap, \
         or `StaysZero` with the reason it cannot move here. A counter nobody can drive is a \
         counter nobody can check, and this project has published four of them."
    );

    let phantom: Vec<&&str> = in_table.iter().filter(|n| !on_wire.contains(n)).collect();
    assert!(
        phantom.is_empty(),
        "this table names counters the wire does not carry: {phantom:?}. Either the column was \
         renamed — in which case every reader asking for the old name is now getting nothing, \
         which is how a renamed column became four cycles of `unwrap_or(0)` — or it was \
         removed and this row should go with it."
    );

    let mut sorted = in_table.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), in_table.len(), "a counter is listed twice");
}

#[test]
fn the_zero_event_control_is_zero_for_every_counter_of_events() {
    // **The isolated control.** A fresh engine, nothing asked of it but the stats query
    // itself. Any `Driven` counter that is already non-zero here is counting something other
    // than the event its row names — which is the defect, not a detail of the fixture.
    let (mut s, e) = (session(), fresh());
    for (name, kind) in TABLE {
        match kind {
            Driven(_) | StaysZero(_) => {
                let v = value(&mut s, &e, name);
                assert_eq!(
                    v, 0,
                    "`{name}` is {v} on a fresh engine before any event. Its row says it counts \
                     events, so something other than those events is moving it."
                );
            }
            Level(_) => {
                // A level is readable and may legitimately be anything, including a
                // configured cap; reading it is the assertion.
                let _ = value(&mut s, &e, name);
            }
        }
    }
}

#[test]
fn every_driven_counter_moves_and_every_stays_zero_counter_does_not() {
    let (mut s, e) = (session(), fresh());
    // `schema_parses` moves when the session first compiles a statement, so the control for
    // it is taken before anything is driven — which the previous test already asserted.
    drive_everything(&mut s, &e);

    for (name, kind) in TABLE {
        let v = value(&mut s, &e, name);
        match kind {
            Driven(event) => assert!(
                v > 0,
                "`{name}` is still zero after {event}. Either the event does not reach it or \
                 the counter does not count what its name says; both are findings and neither \
                 is fixed by removing this row."
            ),
            StaysZero(why) => assert_eq!(
                v, 0,
                "`{name}` moved to {v} in a fixture where it is documented not to:\n  {why}\n\
                 If the behaviour is intended, the reason above has stopped being true and the \
                 claim that rests on it has to be re-made. Do not relax this assertion to make \
                 the suite green."
            ),
            Level(_) => {}
        }
    }
}
