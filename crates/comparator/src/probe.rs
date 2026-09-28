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
//! With `truth(k, e)` the balance of k through epoch e — the oracle's in the append-only mode;
//! the final base's in the mutating mode, since a mutable base has no other — and
//! `d = value − truth` for each read:
//! 1. **double application** — `d` equals the amount of one of k's own legs at or before e;
//! 2. **skipped delta** — `d` equals minus such an amount, or (mutating) a nonzero multiple
//!    of D;
//! 3. **lost delta** — after the driver stops, a resident value that differs from the truth at
//!    the view's frontier, or two quiescent head reads that are both wrong;
//! 4. **upquery race** — two reads of the same `(k, e)` returned different values (an answer
//!    at a fixed anchor is a function of the anchor);
//! 5. **upquery deadlock** — any SQLSTATE 40P01, or a read that did not return.
//!
//! A nonzero `d` that matches neither 1 nor 2 is reported as *other divergence*, never folded
//! into a category it does not fit.

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
    let observed: Mutex<Vec<(Key, u64, i128)>> = Mutex::new(Vec::new());
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
                    match c.simple(&head_sql(arm, k)) {
                        Ok(rows) => match value_and_anchor(&rows) {
                            Some((v, a)) => local.push((k, a, v)),
                            None => {
                                read_errors.fetch_add(1, Ordering::Relaxed);
                            }
                        },
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
            let (stop, errors, deadlocks, hot) = (&stop, &errors, &deadlocks, &hot);
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
                    match c.simple(&sql) {
                        Ok(_) => applied += 1,
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
    let mut cache: BTreeMap<(Key, u64), i128> = BTreeMap::new();
    let mut by_anchor: BTreeMap<(Key, u64), BTreeSet<i128>> = BTreeMap::new();
    for &(k, a, v) in &obs {
        by_anchor.entry((k, a)).or_default().insert(v);
        let idx = arm.oracle_index(a);
        let truth = if mutate && arm.name() != "N" {
            match cache.get(&(k, a)) {
                Some(t) => *t,
                None => {
                    let t = base_truth(&mut c, k, a).unwrap_or(i128::MIN);
                    cache.insert((k, a), t);
                    t
                }
            }
        } else {
            oracle.balance(k, idx).unwrap_or(0)
        };
        let d = v - truth;
        if d == 0 {
            continue;
        }
        let legs = oracle.legs_through(k, idx);
        let kind = if legs.iter().any(|&x| x as i128 == d) {
            rep.double_application += 1;
            "double application"
        } else if legs.iter().any(|&x| -(x as i128) == d) || (mutate && d % D as i128 == 0) {
            rep.skipped_delta += 1;
            "skipped delta"
        } else {
            rep.other_divergence += 1;
            "other divergence"
        };
        if rep.examples.len() < 12 {
            rep.examples.push(format!(
                "{kind}: key {k:?} at #{a}: read {v}, truth {truth}, d = {d}"
            ));
        }
    }
    for ((k, a), vs) in &by_anchor {
        if vs.len() > 1 {
            rep.upquery_race += 1;
            if rep.examples.len() < 16 {
                rep.examples
                    .push(format!("upquery race: key {k:?} at #{a}: values {vs:?}"));
            }
        }
    }
    // Quiescent: is what the view holds now right?
    if arm.name() == "N" {
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
