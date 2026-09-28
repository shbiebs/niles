//! One size point, one seed: load every arm from one universe, prove they hold the same data,
//! then run the same script on each, interleaved, checking every answer against the oracle.
//!
//! # The shape of a run (§5.6), fixed before the first measured run
//!
//! * **Mixed phase, one client:** `ops` operations, every tenth a two-leg transfer (90% reads,
//!   10% writes), the reads drawn from the query mix [`MIX`]. The writes are the same
//!   transactions, in the same positions, on every arm; the oracle applies each once.
//! * **Concurrency phases, 2 and 4 clients:** point reads only (q1 and q2), `conc_reads` per
//!   client, all clients started together, over the state the mixed phase left. Point reads
//!   are the queries with enough samples per run for a p99; q3–q6 appear a handful of times
//!   per run and are measured at one client only.
//! * **Warm-up:** `warmups` runs of the same shape before the first measured run, discarded
//!   as measurements and used for two things only: the resolution floor, and the refusal
//!   condition "warm-up MAD above 15% of the median".
//! * **Interleaving:** run r visits the arms in an order rotated by r, so no arm always goes
//!   first after a pause.
//! * **Every read is checked** against the oracle outside the timed region. A divergence is
//!   counted and the first examples kept; any divergence refuses every comparison involving
//!   that arm at that size and seed.

use crate::arms::{expected, parse, Answer, Arm, LoadReport, Query};
use crate::oracle::{Key, Oracle};
use crate::universe::{checksum_of, Params, Rng, Universe};
use bank_bench::wire::Client;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

/// The read mix of the mixed phase: (query id, weight). Declared in E27's header.
pub const MIX: &[(&str, f64)] = &[
    ("q1", 0.60),
    ("q2", 0.25),
    ("q3", 0.05),
    ("q4", 0.03),
    ("q5", 0.05),
    ("q6", 0.02),
];

#[derive(Debug, Clone)]
pub struct Config {
    pub ops: usize,
    pub runs: usize,
    pub warmups: usize,
    pub conc_levels: Vec<usize>,
    pub conc_reads: usize,
    pub budget_share: f64,
    pub checkpoint: usize,
    pub alpha: f64,
    pub multi_currency: bool,
}

impl Config {
    pub fn declared(multi_currency: bool) -> Config {
        Config {
            ops: 200,
            runs: 10,
            warmups: 3,
            conc_levels: vec![2, 4],
            conc_reads: 60,
            budget_share: 0.05,
            checkpoint: 16,
            alpha: 0.6,
            multi_currency,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Read(Query, Answer),
    Write(u64, crate::universe::Txn),
}

/// One run's script: the mixed phase, then one list of reads per client per level.
#[derive(Debug, Clone)]
pub struct Script {
    pub mixed: Vec<Op>,
    pub conc: Vec<(usize, Vec<Vec<Op>>)>,
    pub keys_read: BTreeSet<Key>,
}

fn draw_key(u: &Universe, o: &Oracle, r: &mut Rng) -> Key {
    for _ in 0..100 {
        let acct = u.sampler.draw(r);
        let ks = o.keys_of(acct);
        if !ks.is_empty() {
            return ks[r.below(ks.len() as u64) as usize];
        }
    }
    let ks = o.keys();
    ks[r.below(ks.len() as u64) as usize]
}

fn draw_read(id: &str, u: &Universe, o: &Oracle, r: &mut Rng) -> Query {
    match id {
        "q1" => Query::Point(draw_key(u, o, r)),
        "q2" => {
            let k = draw_key(u, o, r);
            let first = o.first_epoch(k).expect("drawn keys exist");
            Query::Anchored(k, first + r.below(o.head() - first + 1))
        }
        "q3" => {
            let k = draw_key(u, o, r);
            let day = (o.head() / 50) as i32;
            Query::Statement(k, day - 21, day)
        }
        "q4" => Query::Desk(r.below(16)),
        "q5" => Query::Top10(if u.params.multi_currency && r.unit() < 0.5 {
            1
        } else {
            0
        }),
        _ => Query::Extract,
    }
}

fn pick(r: &mut Rng) -> &'static str {
    let x = r.unit();
    let mut acc = 0.0;
    for (id, w) in MIX {
        acc += w;
        if x < acc {
            return id;
        }
    }
    MIX[MIX.len() - 1].0
}

/// Build one run's script and advance the oracle through its writes.
pub fn script(
    u: &Universe,
    o: &mut Oracle,
    cfg: &Config,
    r: &mut Rng,
    next_txn: &mut u64,
) -> Script {
    let mut mixed = Vec::with_capacity(cfg.ops);
    let mut keys_read = BTreeSet::new();
    let note = |q: &Query, ks: &mut BTreeSet<Key>| match q {
        Query::Point(k) | Query::Anchored(k, _) | Query::Statement(k, _, _) => {
            ks.insert(*k);
        }
        _ => {}
    };
    for i in 0..cfg.ops {
        if i % 10 == 9 {
            let idx = o.head() + 1;
            let t = Universe::one(&u.params, &u.sampler, r, *next_txn, idx);
            *next_txn += 1;
            o.apply(idx, t.clone());
            mixed.push(Op::Write(idx, t));
        } else {
            let q = draw_read(pick(r), u, o, r);
            note(&q, &mut keys_read);
            let a = expected(o, &q);
            mixed.push(Op::Read(q, a));
        }
    }
    let mut conc = Vec::new();
    for &level in &cfg.conc_levels {
        let mut clients = Vec::new();
        for _ in 0..level {
            let mut ops = Vec::with_capacity(cfg.conc_reads);
            for _ in 0..cfg.conc_reads {
                let id = if r.unit() < 0.7 { "q1" } else { "q2" };
                let q = draw_read(id, u, o, r);
                note(&q, &mut keys_read);
                let a = expected(o, &q);
                ops.push(Op::Read(q, a));
            }
            clients.push(ops);
        }
        conc.push((level, clients));
    }
    Script {
        mixed,
        conc,
        keys_read,
    }
}

/// Nearest-rank percentile of microsecond samples.
pub fn pct(xs: &[f64], p: f64) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rank = ((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize;
    v[rank.min(v.len()) - 1]
}

/// Fewer samples than this in one run and a p99 for that run is not computed: a "p99" of
/// nine samples is their maximum.
pub const MIN_FOR_P99: usize = 50;

/// Latency samples, microseconds, by query id.
type Lat = BTreeMap<&'static str, Vec<f64>>;
/// A (legs, SHA-256) checksum.
pub type Checksum = (u64, String);

#[derive(Debug, Clone, Default)]
pub struct Divergence {
    pub count: u64,
    /// Reads chosen as a deadlock victim (SQLSTATE 40P01) and retried. Not a divergence: the
    /// retry's answer is checked like any other, and its latency includes the failed attempt,
    /// because that is what the client waited for.
    pub deadlock_retries: u64,
    pub examples: Vec<String>,
}

/// What one arm did in one run.
#[derive(Debug, Clone, Default)]
pub struct RunOut {
    pub values: BTreeMap<String, f64>,
    /// Raw latency samples by metric family (e.g. "q1", "c4_q2", "write"), for pooled p999.
    pub samples: BTreeMap<String, Vec<f64>>,
    pub divergence: Divergence,
    pub errors: Vec<String>,
}

fn exec_read(
    arm: &dyn Arm,
    c: &mut Client,
    q: &Query,
    want: &Answer,
    out: &mut Vec<f64>,
    d: &mut Divergence,
    errs: &mut Vec<String>,
) {
    let sql = arm.sql(q);
    let t0 = Instant::now();
    let mut res = c.simple(&sql);
    let mut tries = 1;
    while let Err(bank_bench::wire::WireError::Server { sqlstate, .. }) = &res {
        if sqlstate != "40P01" || tries >= 20 {
            break;
        }
        d.deadlock_retries += 1;
        tries += 1;
        res = c.simple(&sql);
    }
    let us = t0.elapsed().as_secs_f64() * 1e6;
    match res {
        Ok(rows) => {
            out.push(us);
            match parse(q, &rows) {
                Ok(got) if &got == want => {}
                Ok(got) => {
                    d.count += 1;
                    if d.examples.len() < 5 {
                        let show = |a: &Answer| {
                            let s = format!("{a:?}");
                            if s.len() > 160 {
                                format!("{}…", &s[..160])
                            } else {
                                s
                            }
                        };
                        d.examples
                            .push(format!("{q:?}: got {} want {}", show(&got), show(want)));
                    }
                }
                Err(e) => {
                    d.count += 1;
                    if d.examples.len() < 5 {
                        d.examples.push(format!("{q:?}: unparseable reply: {e}"));
                    }
                }
            }
        }
        Err(e) => {
            d.count += 1;
            if d.examples.len() < 5 {
                d.examples.push(format!("{q:?}: error {e}"));
            }
            if errs.len() < 5 {
                errs.push(format!("{q:?} `{sql}`: {e}"));
            }
        }
    }
}

/// Run one script on one arm.
pub fn run_arm(arm: &dyn Arm, s: &Script) -> Result<RunOut, String> {
    let mut out = RunOut::default();
    let mut c = arm.connect()?;
    let mut lat: BTreeMap<&'static str, Vec<f64>> = BTreeMap::new();
    let mut writes = Vec::new();
    for op in &s.mixed {
        match op {
            Op::Read(q, want) => {
                if arm.supports(q).is_err() {
                    continue;
                }
                let v = lat.entry(q.id()).or_default();
                exec_read(
                    arm,
                    &mut c,
                    q,
                    want,
                    v,
                    &mut out.divergence,
                    &mut out.errors,
                );
            }
            Op::Write(idx, t) => {
                let stmts = arm.write_sql(*idx, t);
                let t0 = Instant::now();
                for st in &stmts {
                    c.simple(st)
                        .map_err(|e| format!("{} write #{idx} `{st}`: {e}", arm.name()))?;
                }
                writes.push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
    }
    for (id, v) in &lat {
        out.values.insert(format!("{id}_p50_us"), pct(v, 50.0));
        if v.len() >= MIN_FOR_P99 {
            out.values.insert(format!("{id}_p99_us"), pct(v, 99.0));
        }
        out.samples.insert((*id).to_string(), v.clone());
    }
    if !writes.is_empty() {
        out.values.insert("write_p50_us".into(), pct(&writes, 50.0));
        out.values.insert("write_p99_us".into(), pct(&writes, 99.0));
        out.values.insert(
            "commits_per_s".into(),
            writes.len() as f64 / (writes.iter().sum::<f64>() / 1e6),
        );
        out.samples.insert("write".into(), writes);
    }
    // Concurrency: every client connects first, then all start together.
    for (level, clients) in &s.conc {
        let mut conns = Vec::new();
        for _ in 0..*level {
            conns.push(arm.connect()?);
        }
        let barrier = std::sync::Barrier::new(*level);
        let results: Vec<(Lat, Divergence, Vec<String>)> = std::thread::scope(|sc| {
            let hs: Vec<_> = conns
                .into_iter()
                .zip(clients.iter())
                .map(|(mut c, ops)| {
                    let b = &barrier;
                    sc.spawn(move || {
                        let mut lat: BTreeMap<&'static str, Vec<f64>> = BTreeMap::new();
                        let mut d = Divergence::default();
                        let mut errs = Vec::new();
                        b.wait();
                        for op in ops {
                            if let Op::Read(q, want) = op {
                                let v = lat.entry(q.id()).or_default();
                                exec_read(arm, &mut c, q, want, v, &mut d, &mut errs);
                            }
                        }
                        (lat, d, errs)
                    })
                })
                .collect();
            hs.into_iter()
                .map(|h| h.join().expect("client thread"))
                .collect()
        });
        let mut merged: BTreeMap<&'static str, Vec<f64>> = BTreeMap::new();
        for (lat, d, errs) in results {
            for (k, v) in lat {
                merged.entry(k).or_default().extend(v);
            }
            out.divergence.count += d.count;
            out.divergence.deadlock_retries += d.deadlock_retries;
            for e in d.examples {
                if out.divergence.examples.len() < 5 {
                    out.divergence.examples.push(e);
                }
            }
            out.errors.extend(errs);
        }
        for (id, v) in merged {
            out.values
                .insert(format!("c{level}_{id}_p50_us"), pct(&v, 50.0));
            if v.len() >= MIN_FOR_P99 {
                out.values
                    .insert(format!("c{level}_{id}_p99_us"), pct(&v, 99.0));
            }
            out.samples.insert(format!("c{level}_{id}"), v);
        }
    }
    out.values.insert(
        "read_deadlock_retries".into(),
        out.divergence.deadlock_retries as f64,
    );
    Ok(out)
}

/// Everything one (size, seed) produced.
#[derive(Debug, Clone, Default)]
pub struct PointResult {
    pub accounts: u64,
    pub seed: u64,
    pub keys: usize,
    pub budget: usize,
    pub history_txns: u64,
    pub batches: u64,
    pub universe_checksum: (u64, String),
    pub end_checksum: (u64, String),
    /// Per arm: load report, loaded checksum, end checksum.
    pub loads: BTreeMap<String, (LoadReport, Checksum, Checksum)>,
    pub pss_loaded: BTreeMap<String, u64>,
    pub state_end: BTreeMap<String, Vec<(String, i128)>>,
    /// (arm, run index; warm-ups negative) -> the run's values.
    pub runs: Vec<(String, i64, RunOut)>,
    pub not_run: BTreeMap<String, Vec<(String, String)>>,
    pub failures: Vec<String>,
}

fn add_memory(arm: &dyn Arm, loaded: u64, keys_read: usize, out: &mut RunOut) {
    if let Some(p) = arm.pss() {
        out.values
            .insert("pss_mib".into(), p as f64 / (1024.0 * 1024.0));
        if keys_read > 0 {
            out.values.insert(
                "pss_growth_bytes_per_key_read".into(),
                (p as f64 - loaded as f64) / keys_read as f64,
            );
        }
    }
}

/// Load, verify, warm up, measure. Arms are loaded fresh for every (size, seed).
pub fn run_point(
    arms: &mut [Box<dyn Arm>],
    cfg: &Config,
    accounts: u64,
    seed: u64,
    log: &mut dyn FnMut(&str),
) -> PointResult {
    let params = Params::declared(accounts, seed, cfg.alpha, cfg.multi_currency);
    let u = Universe::generate(&params);
    let mut o = Oracle::of(&u);
    assert!(o.conserves(), "the generated history conserves");
    let keys = o.key_count();
    let budget = ((keys as f64 * cfg.budget_share).round() as usize).max(1);
    let mut res = PointResult {
        accounts,
        seed,
        keys,
        budget,
        history_txns: params.history_txns,
        batches: u.batches(),
        universe_checksum: u.checksum(),
        ..Default::default()
    };
    log(&format!(
        "size {accounts} seed {seed}: {} txns in {} batches, {keys} keys, budget {budget}",
        params.history_txns,
        u.batches()
    ));

    // Load every arm and prove they hold the same legs.
    for arm in arms.iter_mut() {
        arm.stop();
        match arm.load(&u, budget) {
            Ok(rep) => {
                let got = arm
                    .connect()
                    .and_then(|mut c| arm.legs(&mut c))
                    .map(|mut l| checksum_of(&mut l));
                match got {
                    Ok(ck) => {
                        log(&format!(
                            "  {} loaded in {:.1}s, {} legs, checksum {}",
                            arm.name(),
                            rep.seconds,
                            ck.0,
                            &ck.1[..16]
                        ));
                        if ck != res.universe_checksum {
                            res.failures.push(format!(
                                "{}: loaded checksum {:?} differs from the universe's {:?} — refused",
                                arm.name(),
                                ck,
                                res.universe_checksum
                            ));
                        }
                        res.loads
                            .insert(arm.name().into(), (rep, ck, Default::default()));
                    }
                    Err(e) => res
                        .failures
                        .push(format!("{}: reading legs: {e}", arm.name())),
                }
            }
            Err(e) => res
                .failures
                .push(format!("{}: load failed: {e}", arm.name())),
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        if let Some(p) = arm.pss() {
            res.pss_loaded.insert(arm.name().into(), p);
        }
    }
    if !res.failures.is_empty() {
        return res;
    }

    // NOT RUN cells, stated once per arm.
    for arm in arms.iter() {
        for id in Query::IDS {
            let probe = match id {
                "q3" => Query::Statement((1, 0), 0, 0),
                "q4" => Query::Desk(0),
                _ => continue,
            };
            if let Err(why) = arm.supports(&probe) {
                res.not_run
                    .entry(arm.name().into())
                    .or_default()
                    .push((id.into(), why.into()));
            }
        }
    }

    let mut r = Rng::new(seed ^ accounts.rotate_left(17) ^ 0xE27);
    let mut next_txn = params.history_txns + 1;
    let mut keys_read: BTreeSet<Key> = BTreeSet::new();
    let n_arms = arms.len();
    for run in 0..(cfg.warmups + cfg.runs) {
        let s = script(&u, &mut o, cfg, &mut r, &mut next_txn);
        keys_read.extend(s.keys_read.iter().copied());
        let idx = run as i64 - cfg.warmups as i64;
        for k in 0..n_arms {
            let arm = &arms[(k + run) % n_arms];
            match run_arm(arm.as_ref(), &s) {
                Ok(mut out) => {
                    match arm
                        .connect()
                        .and_then(|mut c| arm.at_head(&mut c, o.head()))
                    {
                        Ok(true) => {}
                        Ok(false) => {
                            res.failures.push(format!(
                                "{} run {idx}: the arm's last epoch is not the oracle's head #{}",
                                arm.name(),
                                o.head()
                            ));
                            return res;
                        }
                        Err(e) => out.errors.push(format!("head check: {e}")),
                    }
                    let loaded = res.pss_loaded.get(arm.name()).copied().unwrap_or(0);
                    add_memory(arm.as_ref(), loaded, keys_read.len(), &mut out);
                    if out.divergence.count > 0 {
                        log(&format!(
                            "  {} run {idx}: {} oracle divergences, e.g. {:?}",
                            arm.name(),
                            out.divergence.count,
                            out.divergence.examples.first()
                        ));
                    }
                    res.runs.push((arm.name().into(), idx, out));
                }
                Err(e) => {
                    res.failures.push(format!("{} run {idx}: {e}", arm.name()));
                    return res;
                }
            }
        }
        log(&format!("  run {idx} done"));
    }

    res.end_checksum = o.checksum();
    for arm in arms.iter() {
        if let Ok(mut c) = arm.connect() {
            if let Ok(mut l) = arm.legs(&mut c) {
                let ck = checksum_of(&mut l);
                if ck != res.end_checksum {
                    res.failures.push(format!(
                        "{}: end-of-run checksum {:?} differs from the oracle's {:?}",
                        arm.name(),
                        ck,
                        res.end_checksum
                    ));
                }
                if let Some(e) = res.loads.get_mut(arm.name()) {
                    e.2 = ck;
                }
            }
            res.state_end.insert(arm.name().into(), arm.state(&mut c));
        }
    }
    res
}
