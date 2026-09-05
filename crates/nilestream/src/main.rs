//! `nilestream` — the end-to-end runner. **This binary is the closed loop.**
//!
//! ```text
//!   Niles source text
//!        │  niles-lang: lex, parse, resolve, typecheck (currency rows, effects, linearity)
//!        ▼
//!   typed IR circuit                    niles-ir: keys, anchors, contracts, provenance
//!        │  niles-ir::verify            the trusted verifier; a bad circuit stops here
//!        ▼
//!   REV runtime                         nilestream-core: absence lattice, anchored upqueries
//!        │  over an immutable, epoch-ordered, hash-chained ledger
//!        ▼
//!   counted work                        base rows read, deltas applied, resident entry-epochs
//! ```
//!
//! Every stage is a real artifact in this repository, and nothing between them is
//! hand-built. That is the difference between a thesis whose system is specified and one
//! whose system is an instrument: the numbers this prints are produced by executing the
//! program that the language's own compiler emitted.
//!
//! Usage:
//! ```text
//!   nilestream run FILE VIEW [--budget N] [--epochs N] [--reads N] [--skew S]
//!                            [--policy lru|random|cost] [--checkpoint C] [--seed N]
//!   nilestream sweep FILE VIEW           the phase diagram: budget x memory price
//! ```

use nilestream_core::rev::{Anchored, Base, Key, Policy, Runtime, Stats};
use proto_engine::{Ledger, Posting, Row, Zipf};
use std::process::ExitCode;

// The bridge — an immutable, epoch-ordered, hash-chained ledger presented to the runtime
// through the three reads it needs — now lives in `proto-engine` beside the ledger itself,
// as `impl rev::Base for Ledger`. It was here, and the daemon had no copy at all, which is
// how the wire path came to serve every read by materialising the base instead of asking the
// runtime. One implementation cannot be used by one binary and forgotten by another.

#[derive(Clone, Copy)]
struct Config {
    budget: Option<u64>,
    epochs: u64,
    reads: u64,
    skew: f64,
    accounts: usize,
    policy: Policy,
    checkpoint: usize,
    seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            budget: Some(2_000),
            epochs: 20_000,
            reads: 40_000,
            skew: 1.1,
            accounts: 20_000,
            policy: Policy::Lru,
            checkpoint: 64,
            seed: 1,
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let (cmd, path, view) = (args[1].as_str(), args[2].as_str(), args[3].as_str());

    let mut cfg = Config::default();
    let mut i = 4;
    while i + 1 < args.len() {
        let v = &args[i + 1];
        match args[i].as_str() {
            "--budget" => cfg.budget = if v == "none" { None } else { v.parse().ok() },
            "--epochs" => cfg.epochs = v.parse().unwrap_or(cfg.epochs),
            "--reads" => cfg.reads = v.parse().unwrap_or(cfg.reads),
            "--skew" => cfg.skew = v.parse().unwrap_or(cfg.skew),
            "--accounts" => cfg.accounts = v.parse().unwrap_or(cfg.accounts),
            "--checkpoint" => cfg.checkpoint = v.parse().unwrap_or(cfg.checkpoint),
            "--seed" => cfg.seed = v.parse().unwrap_or(cfg.seed),
            "--policy" => {
                cfg.policy = match v.as_str() {
                    "random" => Policy::Random,
                    "cost" => Policy::CostAware,
                    _ => Policy::Lru,
                }
            }
            _ => {}
        }
        i += 2;
    }

    match cmd {
        "run" => match run(path, view, cfg) {
            Ok(s) => {
                report(view, &cfg, &s);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("nilestream: {e}");
                ExitCode::from(1)
            }
        },
        "sweep" => match sweep(path, view, cfg) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("nilestream: {e}");
                ExitCode::from(1)
            }
        },
        other => {
            eprintln!("nilestream: unknown command `{other}`\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

const USAGE: &str = "\
nilestream — compile a Niles program and run it on the partial-state engine

USAGE:
    nilestream run   FILE VIEW [options]   run a workload, print counted work
    nilestream sweep FILE VIEW [options]   sweep the budget, print the phase data as CSV

OPTIONS:
    --budget N|none   resident-entry ceiling (default 2000; `none` = full materialization)
    --epochs N        epochs to seal (default 20000)
    --reads N         reads to issue (default 40000)
    --skew S          Zipf rank exponent; higher is more skewed (default 1.1)
    --accounts N      key-space size (default 20000)
    --policy P        lru | random | cost (default lru)
    --checkpoint C    per-key checkpoint interval, 0 to disable (default 64)
    --seed N          PRNG seed (default 1)
";

/// **The whole loop, in one function.**
fn run(path: &str, view: &str, cfg: Config) -> Result<Stats, String> {
    // ---- 1. source text -> typed IR --------------------------------------------------
    let src = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let (prog, mut diags) = niles_lang::parser::parse_program(&src);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    diags.extend(rd);
    let (_report, td) = niles_lang::typecheck::check_program(&prog, &cat);
    diags.extend(td);
    if diags.has_errors() {
        return Err(format!(
            "the program does not compile:\n{}",
            diags.render(&src, path)
        ));
    }
    let (lowered, ld) = niles_lang::lower::lower_program(&prog, &cat);
    if ld.has_errors() {
        return Err(format!("lowering failed:\n{}", ld.render(&src, path)));
    }

    // ---- 2. the verifier stands between the compiler and the engine -------------------
    // The compiler is not in the trusted base; this is. A compiler bug produces a rejected
    // circuit here rather than a wrong answer downstream.
    let vr = niles_ir::verify::verify(&lowered.circuit);
    if !vr.is_ok() {
        return Err(format!("the circuit did not verify:\n{}", vr.render()));
    }

    // Keep only the requested view, so the run measures one REV rather than all of them.
    let mut circuit = lowered.circuit;
    let Some(id) = circuit.outputs.get(view).copied() else {
        let mut names: Vec<&String> = circuit.outputs.keys().collect();
        names.sort();
        return Err(format!("no view `{view}` in {path}; available: {names:?}"));
    };
    circuit.outputs.clear();
    circuit.set_output(view, id);
    circuit.reset_access();

    // ---- 3. install on the REV runtime ------------------------------------------------
    let mut rt = Runtime::install(circuit, cfg.budget, cfg.policy).map_err(|e| {
        format!(
            "this view cannot run on the current engine fragment: {}",
            e.explain()
        )
    })?;

    // The accessed-field audit: if the runtime planned without reading a semantic
    // annotation, that is a hard error here, not a silently wrong answer later.
    let audit = rt.circuit.audit_access();
    let relevant: Vec<_> = audit
        .unread
        .into_iter()
        .filter(|(n, _)| rt.views.iter().any(|v| v.node == *n))
        .collect();
    if !relevant.is_empty() {
        return Err(format!(
            "the runtime ignored semantic IR fields: {relevant:?}"
        ));
    }

    // ---- 4. a ledger, and a workload over it -------------------------------------------
    let mut base = if cfg.checkpoint > 0 {
        Ledger::with_checkpoints(cfg.checkpoint)
    } else {
        Ledger::new()
    };
    let mut writes = Zipf::new(cfg.accounts, cfg.skew, cfg.seed);
    let mut reads = Zipf::new(cfg.accounts, cfg.skew, cfg.seed ^ 0x5eed);

    // Interleave writes and reads so the run exercises maintenance against reconstruction,
    // which is the trade the phase diagram is about.
    let read_every = (cfg.epochs / cfg.reads.max(1)).max(1);
    for e in 1..=cfg.epochs {
        // One balanced transfer per epoch: two postings that sum to zero, so the ledger's
        // own commit rule accepts them and conservation holds by construction.
        let from = writes.sample() as u64;
        let to = (writes.sample() as u64 + 1) % cfg.accounts as u64;
        let amt = 100 + (e as i128 % 900);
        let rows = vec![
            Row::Post(Posting {
                txn: e,
                acct: from,
                cur: 0,
                amt: -amt,
                valid: e as i64,
            }),
            Row::Post(Posting {
                txn: e,
                acct: to,
                cur: 0,
                amt,
                valid: e as i64,
            }),
        ];
        // Use the epoch the ledger actually sealed rather than the loop counter. They are
        // not the same: the ledger numbers epochs from zero, and an off-by-one here does
        // not produce a wrong answer — it produces a *silent null*, a run in which no
        // delta is ever found and the maintenance counters read zero. The first version of
        // this runner had exactly that bug, and it is the reason the loop below asserts
        // that maintenance actually happened.
        let sealed = match base.submit(&format!("t{e}"), rows) {
            Ok(id) => id,
            Err(_) => continue,
        };

        rt.advance(&base, sealed);

        if e % read_every == 0 {
            let k: Key = vec![reads.sample() as i64, 0];
            let anchor = base.frontier();
            let v = rt.view_mut(view).expect("installed above");
            let a: Anchored = v.read(&base, &k, anchor);
            std::hint::black_box(a);
        }
    }

    let s = rt.stats();
    // A run that applied and skipped no deltas measured nothing about maintenance, whatever
    // its other counters say. Failing loudly beats reporting a null as a result.
    if s.deltas_applied + s.deltas_skipped == 0 {
        return Err(
            "the run observed no deltas at all: the epoch stream and the ledger disagree, \
             and any maintenance figure from this run would be a silent null"
                .to_string(),
        );
    }
    Ok(s)
}

fn report(view: &str, cfg: &Config, s: &Stats) {
    println!("view                  {view}");
    println!(
        "budget                {}",
        cfg.budget
            .map_or("none (full materialization)".into(), |b| b.to_string())
    );
    println!(
        "policy                {:?}   checkpoint interval {}",
        cfg.policy, cfg.checkpoint
    );
    println!(
        "skew (Zipf s)         {}   key space {}",
        cfg.skew, cfg.accounts
    );
    println!();
    println!("reads                 {}", s.reads);
    println!(
        "hits / misses         {} / {}   (hit ratio {:.3})",
        s.hits,
        s.misses,
        s.hit_ratio()
    );
    println!("upqueries             {}", s.upqueries);
    println!(
        "base rows read        {}   ({:.1} per upquery)",
        s.base_rows_read,
        if s.upqueries == 0 {
            0.0
        } else {
            s.base_rows_read as f64 / s.upqueries as f64
        }
    );
    println!("deltas applied        {}", s.deltas_applied);
    println!(
        "deltas skipped        {}   (the saving partiality buys)",
        s.deltas_skipped
    );
    println!("evictions             {}", s.evictions);
    println!("peak resident         {}", s.peak_resident);
    println!(
        "resident entry-epochs {}   (the memory term: the integral, not the peak)",
        s.resident_entry_epochs
    );
}

/// The phase diagram: sweep the budget, price each run under a range of memory prices, and
/// print the CSV. Pricing after the fact is what lets one measured run be re-scored under
/// many cost models without re-running anything.
fn sweep(path: &str, view: &str, base_cfg: Config) -> Result<(), String> {
    println!(
        "budget,memory_price,resident_entry_epochs,deltas_applied,base_rows_read,cost,hit_ratio,z"
    );
    let budgets: Vec<Option<u64>> = vec![
        Some(250),
        Some(500),
        Some(1_000),
        Some(2_000),
        Some(4_000),
        Some(8_000),
        None,
    ];
    for b in budgets {
        let mut cfg = base_cfg;
        cfg.budget = b;
        let s = run(path, view, cfg)?;
        for price in [0.0001f64, 0.0005, 0.002, 0.01, 0.05] {
            let cost = s.cost(price, 1.0, 1.0);
            // Z, the delayed-hit factor of Theorem 4.2(ii), in counted work: base rows
            // per reconstruction against deltas per read. Defined identically in E4's
            // writer, so the two phase diagrams' columns mean the same thing — which is the
            // reason to define it in units of counted work rather than borrowing a time.
            let z = if s.misses == 0 {
                0.0
            } else {
                let rows_per_reconstruction = s.base_rows_read as f64 / s.misses as f64;
                let deltas_per_read = s.deltas_applied as f64 / s.reads.max(1) as f64;
                rows_per_reconstruction / deltas_per_read.max(1e-9)
            };
            println!(
                "{},{},{},{},{},{:.3},{:.4},{z:.4}",
                b.map_or("full".into(), |x| x.to_string()),
                price,
                s.resident_entry_epochs,
                s.deltas_applied,
                s.base_rows_read,
                cost,
                s.hit_ratio()
            );
        }
    }
    Ok(())
}
