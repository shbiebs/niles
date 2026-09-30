//! **E27b's memory metric, guarded where it can be taken** (cycle 15, C15-02, E2).
//!
//! `select nilestream_stats` reports `view_state_bytes` (the balance view's derived state) and
//! `checkpoint_bytes` (the ledger's per-key checkpoint state), counted by cloning under a
//! meter. The workspace cannot take either figure — it has no metering allocator, by §9.10 —
//! and `nilestream-server/tests/stats_counters.rs` asserts that it reports them as NULL. This
//! test binary installs `memprobe::alloc::Metered`, exactly as `nilestreamd-metered` does, and
//! asserts how the figures move.
//!
//! The pre-registered guard (`docs/study/E27b-design.md` §2, E2) was: *zero before any read,
//! grows with installs, stays bounded by the budget under eviction*. The first two hold. **The
//! third does not, and this test says so rather than asserting it**: an evicted key leaves a
//! `Hole` carrying its version, so the view's slots — and its bytes — grow with every key ever
//! read, while its resident values and policy metadata stay at the budget. Whether a demand
//! view keeps holes at all is LC-31, which is open; E27b measures the engine as it is, so the
//! guard asserts the present behaviour and fails if it changes.

use memprobe::alloc::Metered;
use nilestream_server::daemon;
use nilestream_server::pg_wire::{Backend, Frontend};
use nilestream_server::rev_engine::RevEngine;
use nilestream_server::session::Session;
use proto_engine::{EvictionPolicy, ViewMode};

#[global_allocator]
static METERED: Metered = Metered;

const BUDGET: usize = 20;

fn stat(s: &mut Session, e: &RevEngine, name: &str) -> Option<u64> {
    let out = s.handle(Frontend::Query("select nilestream_stats".into()), e);
    let names: Vec<String> = out
        .iter()
        .find_map(|m| match m {
            Backend::RowDescription(f) => Some(f.iter().map(|x| x.name.clone()).collect()),
            _ => None,
        })
        .expect("a row description");
    let row = nilestream_server::pg_wire::decoded_rows(&out)
        .into_iter()
        .next()
        .expect("one row");
    let i = names
        .iter()
        .position(|n| n == name)
        .unwrap_or_else(|| panic!("`{name}` is not on the wire"));
    row[i]
        .as_ref()
        .map(|v| v.trim().parse().expect("an integer"))
}

fn read(s: &mut Session, e: &RevEngine, key: i64) {
    let out = s.handle(
        Frontend::Query(format!(
            "select acct, sum(amt) from postings where acct = {key} group by acct"
        )),
        e,
    );
    assert!(
        !out.iter()
            .any(|m| matches!(m, Backend::ErrorResponse { .. })),
        "the read was refused: {out:?}"
    );
}

#[test]
fn view_state_bytes_are_zero_before_a_read_grow_with_installs_and_keep_holes_under_eviction() {
    assert!(
        memprobe::alloc::metered_installed(),
        "this binary's allocator is not `Metered`, so every figure below would read zero"
    );
    assert!(nilestream_core::meter::install(memprobe::alloc::held));

    let e = RevEngine::seeded_with_checkpoints(
        400,
        3,
        BUDGET,
        ViewMode::Demand,
        EvictionPolicy::Lru,
        2,
    );
    let mut s = Session::new("bench".into(), "bank".into(), daemon::DEFAULT_SCHEMA.into());

    // Zero before any read: nothing is resident, no slot is marked, no flight is open.
    let v = |s: &mut Session| stat(s, &e, "view_state_bytes").expect("metered, so not NULL");
    assert_eq!(
        v(&mut s),
        0,
        "the view holds bytes before anything was read"
    );
    // The checkpoint state is maintained on write, so it is non-zero from the load.
    let cp = stat(&mut s, &e, "checkpoint_bytes").expect("metered");
    assert!(
        cp > 0,
        "400 accounts × 3 rounds at C = 2 checkpointed nothing"
    );

    // Grows with installs, strictly, while the budget has room.
    let mut last = 0;
    for k in 1..=BUDGET as i64 {
        read(&mut s, &e, k);
        let now = v(&mut s);
        assert!(
            now > last,
            "installing key {k} did not grow the view ({last} → {now})"
        );
        last = now;
    }
    let at_budget = last;
    let per_resident = at_budget / BUDGET as u64;
    assert_eq!(stat(&mut s, &e, "resident"), Some(BUDGET as u64));

    // Under eviction: resident values and metadata stay at the budget …
    for k in (BUDGET as i64 + 1)..=100 {
        read(&mut s, &e, k);
    }
    let at_100 = v(&mut s);
    for k in 101..=200 {
        read(&mut s, &e, k);
    }
    let at_200 = v(&mut s);
    assert_eq!(stat(&mut s, &e, "resident"), Some(BUDGET as u64));
    assert!(stat(&mut s, &e, "view_metadata_keys").unwrap() <= BUDGET as u64);

    // … and the bytes still grow, by one hole per key read beyond the budget (LC-31, open).
    // The figure is the hole's cost: less than a resident entry's, and not zero. If holes
    // are ever compacted this fails, and E27b's "bounded by the budget" clause becomes true.
    let per_hole = (at_200 - at_100) / 100;
    assert!(
        per_hole > 0,
        "the view stopped growing under eviction ({at_100} → {at_200} over 100 new keys): \
         holes are no longer retained, so the guard's third clause now holds — assert it"
    );
    assert!(
        per_hole < per_resident,
        "an evicted key costs {per_hole} B and a resident one {per_resident} B: a hole is \
         supposed to be the cheaper of the two"
    );
    // The checkpoint state did not move: nothing was written.
    assert_eq!(stat(&mut s, &e, "checkpoint_bytes"), Some(cp));
}
