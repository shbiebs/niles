//! **The five anomalies (§5.7)**: can partial maintenance give a wrong answer under
//! concurrency, and on which arm?
//!
//! The thesis says Nilestream dissolves five anomalies of partially-stateful dataflow by
//! construction: double application, skipped delta, lost delta, upquery race and upquery
//! deadlock. "By construction" is unfalsifiable unless the same driver can exhibit at least
//! one of them somewhere, so the probe runs one driver against every arm and reports, per arm,
//! what it saw. Pre-registered (§5.7): at least one exhibited on M or H1, none on N.
//!
//! # The driver
//!
//! A small universe (10³ accounts), a residency budget of 5 keys against 20 hot keys, so
//! nearly every read reconstructs. Then, concurrently: **R ≥ 2 readers** reading the hot keys
//! at the head and recording `(key, anchor, value)` from each reply; **one appender** posting
//! transfers between hot accounts; and, in the *mutating* mode, **one mutator** that rewrites a
//! historical leg of a hot key (+D on one leg, −D on its twin, D = 10⁹ minor units, in one
//! transaction, so conservation still holds and the change is recognisable).
//!
//! # The five, as predicates over what was observed
//!
//! With `truth(k, e)` the balance of k through epoch e — the oracle's, plus, in the mutating
//! mode, every mutation of k's legs sealed at or before e that had landed when the read ran
//! (a mutation that may have landed *during* the read is allowed either way: see
//! `truth_range`) — and
//! `d = value − truth` for each read:
//! 1. **double application** — `d` equals the amount of one of k's own legs at or before e;
//! 2. **skipped delta** — `d` equals minus such an amount, or (mutating) a nonzero multiple
//!    of D;
//! 3. **lost delta** — after the driver stops, a resident value that differs from the truth at
//!    the view's frontier, or two quiescent head reads that are both wrong;
//! 4. **upquery race** — two reads of the same `(k, e)`, with the same mutations landed and
//!    none in flight, returned different values (such an answer is a function of the anchor);
//! 5. **upquery deadlock** — any SQLSTATE 40P01, or a read that did not return.
//!
//! A nonzero `d` that matches neither 1 nor 2 is reported as *other divergence*, never folded
//! into a category it does not fit.
//!
//! On an arm that answers from a lagging replica (H1) a read carries no anchor; it is
//! bracketed instead by the replica's applied position just before and after it, which gives
//! a range of epochs it can be for, and it is right if it matches the truth at any of them
//! (`h1.rs`). Mutations are placed on the same clock: the replica's log position on H1,
//! nanoseconds since the probe began elsewhere.

use crate::arms::{Arm, Query};
use crate::oracle::{Key, Oracle};
use crate::universe::{Params, Rng, Universe};
use bank_bench::wire::{Client, WireError};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub const D: i64 = 1_000_000_000;

#[derive(Debug, Clone, Default)]
pub struct ProbeReport {
    pub arm: String,
    pub mode: String,
    pub readers: usize,
    pub reads: u64,
    pub appends: u64,
    pub mutations_attempted: u64,
    pub mutations_applied: u64,
    pub mutation_refusal: Option<String>,
    pub double_application: u64,
    pub skipped_delta: u64,
    pub lost_delta: u64,
    pub upquery_race: u64,
    pub upquery_deadlock: u64,
    pub other_divergence: u64,
    pub read_errors: u64,
    pub appends_failed: u64,
    pub examples: Vec<String>,
}

impl ProbeReport {
    pub fn exhibited(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        for (n, name) in [
            (self.double_application, "double application"),
            (self.skipped_delta, "skipped delta"),
            (self.lost_delta, "lost delta"),
            (self.upquery_race, "upquery race"),
            (self.upquery_deadlock, "upquery deadlock"),
        ] {
            if n > 0 {
                v.push(name);
            }
        }
        v
    }
}

/// The head read this probe issues, returning (value, anchor stamped by the arm).
fn head_sql(arm: &dyn Arm, (a, c): Key) -> String {
    match arm.name() {
        "N" => arm.sql(&Query::Point((a, c))),
        _ => format!(
            "select value, anchor from arm.rev_read({a}, {c}::smallint, (select applied_through from arm.rev_meta))"
        ),
    }
}

fn sqlstate(e: &WireError) -> Option<String> {
    match e {
        WireError::Server { sqlstate, .. } => Some(sqlstate.clone()),
        _ => None,
    }
}

/// One head read of `k`: `(first, last, value)`, where `first..=last` are the oracle epoch
/// indexes the answer can be for. On an arm that stamps an anchor (N, P+, M+) that is one
/// epoch. On an arm that answers from a lagging replica (H1) it is the bracket of epochs the
/// replica must and may have applied around the read (see `h1.rs`). `Ok(None)` is a reply
/// that could not be read as a value.
fn read_head(
    arm: &dyn Arm,
    c: &mut Client,
    k: Key,
    clock: &dyn Fn() -> u64,
) -> Result<Option<Observed>, WireError> {
    if let Some(before) = arm.replica_offsets() {
        let Ok((lo, _)) = before else {
            return Ok(None);
        };
        let rows = c.simple(&arm.sql(&Query::Point(k)))?;
        let Some(Ok((_, hi))) = arm.replica_offsets() else {
            return Ok(None);
        };
        let (must, may) = arm.epochs_between(lo, hi);
        let v = rows
            .rows
            .first()
            .and_then(|r| r.last().cloned().flatten())
            .and_then(|t| {
                let t = t.trim().to_string();
                t.parse::<i128>()
                    .ok()
                    .or_else(|| t.split_once('.').and_then(|(w, _)| w.parse().ok()))
            });
        return Ok(v.map(|v| Observed {
            key: k,
            first: must,
            last: may,
            value: v,
            window: (lo, hi),
        }));
    }
    let w0 = clock();
    let rows = c.simple(&head_sql(arm, k))?;
    let w1 = clock();
    Ok(value_and_anchor(&rows).map(|(v, a)| {
        let i = arm.oracle_index(a);
        Observed {
            key: k,
            first: i,
            last: i,
            value: v,
            window: (w0, w1),
        }
    }))
}

/// One read the probe observed. `first..=last` are the oracle epochs it can be for;
/// `window` is where it stood on the arm's clock — for a replica, the applied log positions
/// just before and after it; otherwise nanoseconds since the probe began, just before the
/// request and just after the reply.
#[derive(Debug, Clone, Copy)]
struct Observed {
    key: Key,
    first: u64,
    last: u64,
    value: i128,
    window: (u64, u64),
}

/// One applied mutation: +D on the first leg of `target`, −D on that leg's twin in the same
/// transaction (on `twin`), both sealed at epoch `epoch`; `at` is where it stood on the arm's
/// clock just before it was sent and just after it committed.
#[derive(Debug, Clone, Copy)]
struct Mutation {
    target: Key,
    twin: Key,
    epoch: u64,
    at: (u64, u64),
}

/// The values a read of `k` through epoch `e` may truthfully return in a window `w`, given the
/// mutations applied so far: the oracle's balance, plus every mutation that had certainly
/// landed before the window opened, plus any subset of those that may have landed during it.
/// Returned as (the value with only the certain ones, the lowest and highest extra multiple of
/// D the uncertain ones allow).
fn truth_range(o: &Oracle, ms: &[Mutation], k: Key, e: u64, w: (u64, u64)) -> (i128, i128, i128) {
    let mut base = o.balance(k, e).unwrap_or(0);
    let (mut up, mut down) = (0i128, 0i128);
    for m in ms {
        if m.epoch > e {
            continue;
        }
        let effect = if m.target == k {
            D as i128
        } else if m.twin == k {
            -(D as i128)
        } else {
            continue;
        };
        if m.at.1 <= w.0 {
            base += effect;
        } else if m.at.0 < w.1 {
            if effect > 0 {
                up += 1;
            } else {
                down += 1;
            }
        }
    }
    (base, -down, up)
}

/// (value, anchor) from a head read: the value is the last non-anchor column, the anchor the
/// column named `anchor`.
fn value_and_anchor(rows: &bank_bench::wire::Rows) -> Option<(i128, u64)> {
    let ai = rows.columns.iter().position(|c| c == "anchor")?;
    let vi = rows.columns.iter().rposition(|c| c != "anchor")?;
    let row = rows.rows.first()?;
    let v = row.get(vi)?.as_ref()?.trim();
    let v: i128 = v
        .parse()
        .ok()
        .or_else(|| v.split_once('.').and_then(|(w, _)| w.parse().ok()))?;
    let a: u64 = row.get(ai)?.as_ref()?.trim().parse().ok()?;
    Some((v, a))
}

pub struct ProbeConfig {
    pub readers: usize,
    pub appends: u64,
    pub mutations: u64,
    pub hot: usize,
    pub budget: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        ProbeConfig {
            readers: 4,
            appends: 300,
            mutations: 100,
            hot: 20,
            budget: 5,
        }
    }
}

/// Run the probe on one arm. `mutate` selects the mutating mode.
pub fn probe(arm: &mut dyn Arm, cfg: &ProbeConfig, mutate: bool) -> Result<ProbeReport, String> {
    let u = Universe::generate(&Params::declared(1000, 1, 0.6, false));
    let mut oracle = Oracle::of(&u);
    arm.stop();
    arm.load(&u, cfg.budget)?;
    let arm: &dyn Arm = arm;
    let hot: Vec<Key> = (1..=cfg.hot)
        .map(|r| (u.sampler.account_at(r), 0))
        .collect();
    let mut rep = ProbeReport {
        arm: arm.name().into(),
        mode: if mutate {
            "append + mutate"
        } else {
            "append only"
        }
        .into(),
        readers: cfg.readers,
        ..Default::default()
    };

    // The appender's transactions, drawn now so the oracle knows them in order.
    let mut r = Rng::new(0x5EED);
    let mut writes = Vec::new();
    for next_id in (u.params.history_txns + 1..).take(cfg.appends as usize) {
        let idx = oracle.head() + 1;
        let from = hot[r.below(hot.len() as u64) as usize].0;
        let mut to = hot[r.below(hot.len() as u64) as usize].0;
        if to == from {
            to = hot[(hot.iter().position(|k| k.0 == from).unwrap() + 1) % hot.len()].0;
        }
        let amt = 1000 + r.below(100_000) as i64;
        let t = crate::universe::Txn {
            id: next_id,
            batch: idx,
            legs: [
                crate::universe::Leg {
                    acct: from,
                    cur: 0,
                    amt: -amt,
                },
                crate::universe::Leg {
                    acct: to,
                    cur: 0,
                    amt,
                },
            ],
            value_day: (idx / 50) as i32,
        };
        oracle.apply(idx, t.clone());
        writes.push((idx, t));
    }

    let stop = AtomicBool::new(false);
    let observed: Mutex<Vec<Observed>> = Mutex::new(Vec::new());
    let mutations: Mutex<Vec<Mutation>> = Mutex::new(Vec::new());
    // The arm's clock: its log position if it answers from a replica, else nanoseconds.
    let t_start = std::time::Instant::now();
    let clock_fn = || {
        arm.log_position()
            .unwrap_or_else(|| t_start.elapsed().as_nanos() as u64)
    };
    let clock: &(dyn Fn() -> u64 + Sync) = &clock_fn;
    // Each hot key's first leg (the row the mutator rewrites: the lowest id, i.e. the first
    // in load order) and the twin leg of the same transaction.
    let mut first_leg: BTreeMap<Key, (Key, u64)> = BTreeMap::new();
    for t in &u.txns {
        for (i, l) in t.legs.iter().enumerate() {
            let other = t.legs[1 - i];
            first_leg
                .entry((l.acct, l.cur))
                .or_insert(((other.acct, other.cur), t.batch));
        }
    }
    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let deadlocks = std::sync::atomic::AtomicU64::new(0);
    let read_errors = std::sync::atomic::AtomicU64::new(0);
    let appends_failed = std::sync::atomic::AtomicU64::new(0);
    let mut readers: Vec<Client> = Vec::new();
    for _ in 0..cfg.readers {
        readers.push(arm.connect()?);
    }
    let mut appender = arm.connect()?;
    let mut mutator = Some(arm.connect()?);
    let (mut applied, mut attempted, mut refusal) = (0u64, 0u64, None::<String>);

    std::thread::scope(|sc| {
        for (i, mut c) in readers.into_iter().enumerate() {
            let (stop, observed, errors, deadlocks, read_errors, hot) =
                (&stop, &observed, &errors, &deadlocks, &read_errors, &hot);
            sc.spawn(move || {
                let mut r = Rng::new(0xBEEF + i as u64);
                let mut local = Vec::new();
                while !stop.load(Ordering::Relaxed) {
                    let k = hot[r.below(hot.len() as u64) as usize];
                    match read_head(arm, &mut c, k, clock) {
                        Ok(Some(o)) => local.push(o),
                        Ok(None) => {
                            read_errors.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => {
                            if sqlstate(&e).as_deref() == Some("40P01") {
                                deadlocks.fetch_add(1, Ordering::Relaxed);
                            } else {
                                read_errors.fetch_add(1, Ordering::Relaxed);
                            }
                            let mut er = errors.lock().unwrap();
                            if er.len() < 5 {
                                er.push(format!("reader: {e}"));
                            }
                        }
                    }
                }
                observed.lock().unwrap().extend(local);
            });
        }
        // The mutator, in its own thread, runs while the appender does.
        let mutator_handle = if mutate {
            let (stop, errors, deadlocks, hot, mutations, first_leg) =
                (&stop, &errors, &deadlocks, &hot, &mutations, &first_leg);
            let mut c = mutator.take().expect("one mutator");
            let n = cfg.mutations;
            Some(sc.spawn(move || {
                let (mut applied, mut attempted, mut refusal) = (0u64, 0u64, None::<String>);
                let mut r = Rng::new(0xD00D);
                while attempted < n && !stop.load(Ordering::Relaxed) {
                    attempted += 1;
                    let (a, cur) = hot[r.below(hot.len() as u64) as usize];
                    let sql = if arm.name() == "N" {
                        format!("update postings set amt = amt + {D} where acct = {a} and cur = {cur}")
                    } else {
                        format!(
                            "begin; \
                             update arm.postings set amt = amt + {D} where id = (select min(id) from arm.postings where acct = {a} and cur = {cur}); \
                             update arm.postings set amt = amt - {D} where txn = (select txn from arm.postings where id = (select min(id) from arm.postings where acct = {a} and cur = {cur})) \
                               and id <> (select min(id) from arm.postings where acct = {a} and cur = {cur}); \
                             commit"
                        )
                    };
                    let t0 = clock();
                    let res = c.simple(&sql);
                    let t1 = clock();
                    match res {
                        Ok(_) => {
                            applied += 1;
                            if let Some(&(twin, epoch)) = first_leg.get(&(a, cur)) {
                                mutations.lock().unwrap().push(Mutation {
                                    target: (a, cur),
                                    twin,
                                    epoch,
                                    at: (t0, t1),
                                });
                            }
                        }
                        Err(e) => {
                            if sqlstate(&e).as_deref() == Some("40P01") {
                                deadlocks.fetch_add(1, Ordering::Relaxed);
                            }
                            if refusal.is_none() {
                                refusal = Some(e.to_string());
                            }
                            let _ = c.simple("rollback");
                            let mut er = errors.lock().unwrap();
                            if er.len() < 5 {
                                er.push(format!("mutator: {e}"));
                            }
                        }
                    }
                }
                (applied, attempted, refusal)
            }))
        } else {
            None
        };
        // A write chosen as a deadlock victim is retried, as any client would: the probe's
        // truth is the oracle's, which has every write, so a write that was rolled back and
        // not retried would make every later read of its keys look like a skipped delta.
        // The deadlock itself is still counted.
        for (idx, t) in &writes {
            if let Err(e) = arm.before_write(*idx) {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("appender #{idx}: before_write: {e}"));
            }
            for st in arm.write_sql(*idx, t) {
                let mut tries = 0;
                loop {
                    match appender.simple(&st) {
                        Ok(_) => break,
                        Err(e) => {
                            let dl = sqlstate(&e).as_deref() == Some("40P01");
                            if dl {
                                deadlocks.fetch_add(1, Ordering::Relaxed);
                            }
                            let mut er = errors.lock().unwrap();
                            if er.len() < 5 {
                                er.push(format!("appender #{idx}: {e}"));
                            }
                            drop(er);
                            tries += 1;
                            if !dl || tries >= 50 {
                                appends_failed.fetch_add(1, Ordering::Relaxed);
                                break;
                            }
                        }
                    }
                }
            }
            if let Err(e) = arm.after_write(*idx) {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("appender #{idx}: after_write: {e}"));
            }
            rep.appends += 1;
        }
        if let Some(h) = mutator_handle {
            (applied, attempted, refusal) = h.join().expect("mutator");
        }
        stop.store(true, Ordering::Relaxed);
    });
    rep.mutations_applied = applied;
    rep.mutations_attempted = attempted;
    rep.mutation_refusal = refusal;
    rep.upquery_deadlock = deadlocks.load(Ordering::Relaxed);
    rep.read_errors = read_errors.load(Ordering::Relaxed);
    rep.appends_failed = appends_failed.load(Ordering::Relaxed);
    if rep.appends_failed > 0 {
        return Err(format!(
            "{} appends failed after retries; the oracle no longer describes the base, so no read can be classified",
            rep.appends_failed
        ));
    }
    rep.examples.extend(errors.into_inner().unwrap());

    // The truth each read is held to.
    let obs = observed.into_inner().unwrap();
    rep.reads = obs.len() as u64;
    let mut c = arm.connect()?;
    let base_truth = |c: &mut Client, (a, cur): Key, e: u64| -> Option<i128> {
        c.simple(&format!(
            "select coalesce(sum(amt), 0) from arm.postings where acct = {a} and cur = {cur} and epoch <= {e}"
        ))
        .ok()?
        .nth(0)
    };
    // Each read is held to what was true at an epoch it can be for, given the mutations that
    // had or may have landed while it ran. (Before cycle 14's R2-03 the mutating mode held
    // every read to the *final* base, so a read taken before a mutation landed counted as a
    // skipped delta; see E27's deviations.)
    let muts = mutations.into_inner().unwrap();
    let mut by_anchor: BTreeMap<(Key, u64, i128), BTreeSet<i128>> = BTreeMap::new();
    for o in &obs {
        let (k, v) = (o.key, o.value);
        // The upquery-race test compares reads at one exact epoch with no mutation of the key
        // in flight: such an answer is a function of (key, epoch, mutations landed).
        if o.first == o.last {
            let (base, lo, hi) = truth_range(&oracle, &muts, k, o.first, o.window);
            if lo == 0 && hi == 0 {
                by_anchor.entry((k, o.first, base)).or_default().insert(v);
            }
        }
        let fits = (o.first..=o.last).any(|e| {
            let (base, lo, hi) = truth_range(&oracle, &muts, k, e, o.window);
            let d = v - base;
            d % D as i128 == 0 && (lo..=hi).contains(&(d / D as i128))
        });
        if fits {
            continue;
        }
        let mut kind = "other divergence";
        let mut shown = (o.last, truth_range(&oracle, &muts, k, o.last, o.window).0);
        for e in (o.first..=o.last).rev() {
            let truth = truth_range(&oracle, &muts, k, e, o.window).0;
            let d = v - truth;
            let legs = oracle.legs_through(k, e);
            if legs.iter().any(|&x| x as i128 == d) {
                kind = "double application";
            } else if legs.iter().any(|&x| -(x as i128) == d) || (mutate && d % D as i128 == 0) {
                kind = "skipped delta";
            } else {
                continue;
            }
            shown = (e, truth);
            break;
        }
        match kind {
            "double application" => rep.double_application += 1,
            "skipped delta" => rep.skipped_delta += 1,
            _ => rep.other_divergence += 1,
        }
        if rep.examples.len() < 12 {
            let (e, truth) = shown;
            let at = if o.first == o.last {
                format!("#{e}")
            } else {
                format!("#{e} (bracket #{}..#{})", o.first, o.last)
            };
            rep.examples.push(format!(
                "{kind}: key {k:?} at {at}: read {v}, truth {truth}, d = {}",
                v - truth
            ));
        }
    }
    for ((k, a, _), vs) in &by_anchor {
        if vs.len() > 1 {
            rep.upquery_race += 1;
            if rep.examples.len() < 16 {
                rep.examples
                    .push(format!("upquery race: key {k:?} at #{a}: values {vs:?}"));
            }
        }
    }
    // Quiescent: is what the view holds now right?
    if arm.replica_offsets().is_some() {
        // A replica: wait until it must have applied every append, then two head reads per
        // hot key; both wrong is a lost delta. A replica that never reaches the head within
        // a minute has lost a delta too, and is reported as such.
        let head = oracle.head();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let caught_up = loop {
            if let Some(Ok((lo, _))) = arm.replica_offsets() {
                if arm.epochs_between(lo, lo).0 >= head {
                    break true;
                }
            }
            if std::time::Instant::now() > deadline {
                break false;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        };
        if !caught_up {
            rep.lost_delta += 1;
            rep.examples.push(format!(
                "lost delta: the replica had not applied through #{head} a minute after the last append"
            ));
        } else {
            for &k in &hot {
                let truth = if mutate {
                    base_truth(&mut c, k, head).unwrap_or(i128::MIN)
                } else {
                    oracle.balance(k, head).unwrap_or(0)
                };
                let mut wrong = 0;
                for _ in 0..2 {
                    if let Ok(rows) = c.simple(&arm.sql(&Query::Point(k))) {
                        let v = crate::arms::parse(&Query::Point(k), &rows)
                            .ok()
                            .and_then(|a| match a {
                                crate::arms::Answer::Balance(b) => Some(b.unwrap_or(0)),
                                _ => None,
                            });
                        if v != Some(truth) {
                            wrong += 1;
                        }
                    }
                }
                if wrong == 2 {
                    rep.lost_delta += 1;
                    rep.examples
                        .push(format!("lost delta: key {k:?} wrong on two quiescent reads at #{head}"));
                }
            }
        }
    } else if arm.name() == "N" {
        for &k in &hot {
            let mut wrong = 0;
            for _ in 0..2 {
                if let Ok(rows) = c.simple(&head_sql(arm, k)) {
                    if let Some((v, a)) = value_and_anchor(&rows) {
                        if Some(v) != oracle.balance(k, arm.oracle_index(a)) {
                            wrong += 1;
                        }
                    }
                }
            }
            if wrong == 2 {
                rep.lost_delta += 1;
                rep.examples.push(format!("lost delta: key {k:?} wrong on two quiescent reads"));
            }
        }
    } else if let Ok(rows) = c.simple(
        "select r.acct, r.cur, r.value::bigint, m.applied_through from arm.rev r, arm.rev_meta m where r.state = 'present'",
    ) {
        for row in &rows.rows {
            let f = |i: usize| row[i].as_deref().unwrap_or("0").parse::<i128>().unwrap_or(0);
            let (a, cur, v, at) = (f(0) as u64, f(1) as u32, f(2), f(3) as u64);
            let truth = if mutate {
                base_truth(&mut c, (a, cur), at).unwrap_or(i128::MIN)
            } else {
                oracle.balance((a, cur), at).unwrap_or(0)
            };
            if v != truth {
                rep.lost_delta += 1;
                if rep.examples.len() < 20 {
                    rep.examples.push(format!(
                        "lost delta: resident ({a}, {cur}) = {v}, truth at #{at} = {truth}"
                    ));
                }
            }
        }
    }
    Ok(rep)
}

/// One line per report, for `results/E27-comparator/probe.tsv`.
pub fn to_tsv(rep: usize, r: &ProbeReport) -> String {
    let clean = |s: &str| s.replace(['\t', '\n'], " ");
    format!(
        "probe\t{rep}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        r.arm,
        r.mode,
        r.readers,
        r.reads,
        r.appends,
        r.mutations_attempted,
        r.mutations_applied,
        clean(r.mutation_refusal.as_deref().unwrap_or("")),
        r.double_application,
        r.skipped_delta,
        r.lost_delta,
        r.upquery_race,
        r.upquery_deadlock,
        r.other_divergence,
        r.read_errors,
        clean(&r.examples.join(" || "))
    )
}
