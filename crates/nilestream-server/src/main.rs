// See lib.rs: the wire surface is incomplete by construction today (no write path, the
// extended query protocol unwired), so several items have no caller yet. Kept rather than
// deleted so the gap stays visible.
#![allow(dead_code)]
// `Backend::BackendKeyData` is the PostgreSQL message name.
#![allow(clippy::enum_variant_names)]

//! `nilestreamd` — the Nilestream daemon.
//!
//! Listens on TCP, speaks the PostgreSQL wire protocol, compiles each query as Niles, and
//! serves it from the REV runtime over a durable ledger. One thread per connection, which
//! is the right shape at this scale and is stated rather than defended: the write path is
//! serialised through a single sealer anyway, so connection-level concurrency is about
//! *reads*, and reads over an immutable base need no coordination at all.
//!
//! ```text
//! nilestreamd [--port N] [--schema FILE] [--accounts N] [--rounds N]
//!             [--budget N] [--mode demand|full]
//! ```
//!
//! # What serves a read
//!
//! Reads are answered by a **partial view over an immutable, hash-chained ledger**
//! (`rev_engine`): partial materialisation, the absence lattice, an anchored upquery on a
//! miss. The daemon used to answer from a `HashMap` and say so in this banner, which was
//! honest and made it unmeasurable — the E16 wall-clock harness could compare it to
//! PostgreSQL but the resulting `PARITY` measured the protocol path rather than the engine.
//!
//! `--budget` and `--mode` are the levers the phase diagram is swept with: a parity result at
//! a 0% miss rate and one at a 40% miss rate are different findings, and a server that could
//! only be run warm would only ever produce the flattering half.
//!
//! Point `psql -h 127.0.0.1 -p 5433 -U anyone bank` at it.

#[path = "daemon.rs"]
mod daemon;
// The engine-lock instrument. `daemon` times every acquisition through it; the binary
// itself never calls it, which is why it needs the allow.
#[allow(dead_code)]
#[path = "lockstats.rs"]
mod lockstats;
// The extended query protocol's plan cache. Reachable from the session as of this
// change; before it, the module existed and no listener referred to it.
mod extended;
#[path = "pg_wire.rs"]
mod pg_wire;
#[path = "rev_engine.rs"]
mod rev_engine;
// The keyed-aggregate fast path `rev_engine` takes before materialising anything.
#[path = "scan_fold.rs"]
mod scan_fold;
#[path = "session.rs"]
mod session;
// The daemon uses one policy (`insecure`) and one negotiation, so most of `tls.rs` is dead
// code *in this binary* while being live in the library and exercised by its 17 tests. The
// alternative — trimming the module to what the binary happens to call — would delete the
// policy layer that exists so a deployment can turn TLS on with one line.
#[allow(dead_code)]
#[path = "tls.rs"]
mod tls;

use rev_engine::RevEngine;
use session::Serving;
use std::net::TcpListener;
use std::sync::Arc;

/// **Parse a flag's value, or refuse by name.**
///
/// Every numeric flag used `parse().unwrap_or(default)`. `--budget 2,500` — a comma, the
/// ordinary way a person writes that number — ran the phase diagram's lever at its default and
/// said nothing; a mistyped `--port` served somewhere else. The engine's own rule is honest
/// refusal over silent fallback, and its front door was the one place that broke it (F-70).
fn flag<T: std::str::FromStr>(name: &str, raw: &str) -> T {
    raw.parse().unwrap_or_else(|_| {
        eprintln!(
            "nilestreamd: {name} expects {}, and `{raw}` is not one. Refusing to start: a \
             flag that fell back to its default would run this server on a configuration \
             nobody asked for and report nothing.",
            std::any::type_name::<T>()
        );
        std::process::exit(2);
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 5433u16;
    let mut schema_path: Option<String> = None;
    let mut accounts = 1000i64;
    let mut rounds = 3u32;
    let mut budget = 100_000usize;
    let mut full = false;
    let mut durable: Option<String> = None;
    // **Durability is a choice that has to be made out loud (LC-16).** Neither flag is a
    // default: a server that serves volatile because nobody said otherwise is a server whose
    // guarantee depends on what an operator forgot to type.
    let mut volatile = false;
    // **And so is an unbounded idempotency window (LC-35).** A schema whose wire relation
    // declares no `idem` column keeps every identity ever committed, forever; that is a
    // legitimate diagnostic configuration and an illegitimate silent one.
    let mut idem_window_flag: Option<Option<u64>> = None;
    let mut i = 1;
    while i < args.len() {
        // The flags that take no value are matched first, so `while i + 1 < args.len()`
        // cannot swallow them — it used to, and a trailing `--volatile` would have been
        // ignored entirely.
        if args[i] == "--volatile" {
            volatile = true;
            i += 1;
            continue;
        }
        if i + 1 >= args.len() {
            eprintln!("nilestreamd: {} expects a value", args[i]);
            std::process::exit(2);
        }
        match args[i].as_str() {
            "--port" => port = flag("--port", &args[i + 1]),
            "--schema" => schema_path = Some(args[i + 1].clone()),
            "--accounts" => accounts = flag("--accounts", &args[i + 1]),
            "--rounds" => rounds = flag("--rounds", &args[i + 1]),
            "--budget" => budget = flag("--budget", &args[i + 1]),
            "--mode" => match args[i + 1].as_str() {
                "full" => full = true,
                "demand" => full = false,
                other => {
                    eprintln!("nilestreamd: --mode expects `full` or `demand`, not `{other}`");
                    std::process::exit(2);
                }
            },
            "--durable" => durable = Some(args[i + 1].clone()),
            "--idem-window" => {
                idem_window_flag = Some(match args[i + 1].as_str() {
                    "unbounded" => None,
                    n => Some(flag::<u64>("--idem-window", n)),
                })
            }
            other => {
                eprintln!(
                    "nilestreamd: `{other}` is not a flag this server knows. Refusing to \
                     start rather than ignoring it: an ignored flag is a configuration the \
                     operator believes is in effect."
                );
                std::process::exit(2);
            }
        }
        i += 2;
    }

    if durable.is_none() && !volatile {
        eprintln!(
            "nilestreamd: refusing to start without `--durable <segment>`.\n\
             \n\
             This server's contract is that an acknowledged transaction is on stable storage. \
             Without a segment there is nothing to acknowledge against, and a client cannot \
             tell the difference from the outside — which is why the choice is a flag and not \
             a default (LC-16). Pass `--volatile` to run without durability on purpose; the \
             banner will say so, and no figure from such a run is a contract result."
        );
        std::process::exit(2);
    }
    if durable.is_some() && volatile {
        eprintln!("nilestreamd: `--durable` and `--volatile` are contradictory");
        std::process::exit(2);
    }

    let schema = match &schema_path {
        Some(p) => std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("nilestreamd: cannot read {p}: {e}");
            std::process::exit(2);
        }),
        None => daemon::DEFAULT_SCHEMA.to_string(),
    };

    // Refuse to start on a schema that does not compile. A server that accepted a broken
    // schema would fail on the first query instead, at which point the operator is
    // debugging a connection rather than a file.
    let (prog, mut d) = niles_lang::parser::parse_program(&schema);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    d.extend(rd);
    if d.has_errors() {
        eprint!(
            "{}",
            d.render(&schema, schema_path.as_deref().unwrap_or("<default>"))
        );
        eprintln!("nilestreamd: the schema does not compile; refusing to start");
        std::process::exit(1);
    }

    // The read side: a partial view over an immutable ledger, seeded with `rounds` balanced
    // transfers per account. Seeded rather than empty because a benchmark against an empty
    // server reports excellent latencies for queries that return nothing, and because a view
    // over a base with no history never exercises the miss path — which is the interesting
    // one.
    let mode = if full {
        proto_engine::ViewMode::Full
    } else {
        proto_engine::ViewMode::Demand
    };
    // **`--durable <segment>` makes the shipped binary what the benchmark has been hosting.**
    //
    // Until this flag existed, `nilestreamd` could not attach a durable sink at all — the
    // only durable engine in the project was the one `bank-bench` builds in-process, and it
    // refuses to run without a PostgreSQL to compare against. So the shipped server's
    // durable throughput was undefined, and "how does Nilestream scale with connections"
    // could not be asked on a machine without PostgreSQL 16. Both of those are instrument
    // gaps rather than engine defects, and this is half of closing them.
    //
    // `SyncPolicy::Always` is the only policy `DurableSink::open` accepts, so there is no
    // flag here that quietly buys throughput by weakening the guarantee.
    // **The declared idempotency window, in epochs, from the schema this daemon compiled.**
    //
    // `idem: IdemKey window N.epochs` reached nothing below the compiler until T-05 (cycle
    // 8): both idempotency indexes kept every identity ever committed, at a measured 68.8 B
    // and 99.6 B each (E18), so the write path's memory was a function of history. A schema
    // that declares no window still keeps everything — the banner says which, because a
    // memory bound nobody can see is a memory bound nobody can act on.
    // The relation the wire writes to is `postings`; its `idem` column's window is the one
    // that governs. Reading "whichever relation the hash map yields first" made the daemon's
    // window differ between two starts of the same binary on the same schema (F-71).
    let idem_window = match crate::session::declared_idem_window(&schema) {
        Ok(w) => w,
        Err(why) => {
            eprintln!("nilestreamd: {why}");
            std::process::exit(2);
        }
    };
    // **LC-35: an unbounded window is a choice, not a default.** A schema whose wire relation
    // declares no `idem` column keeps every identity ever committed — a legitimate
    // configuration for a diagnostic run and an illegitimate silent one, because the memory
    // it costs grows with history and nothing says so.
    let idem_window = match (idem_window, idem_window_flag) {
        (Some(w), None) => {
            eprintln!("  idempotency window: {w} epochs, from the schema");
            Some(w)
        }
        (Some(w), Some(f)) => {
            eprintln!(
                "nilestreamd: the schema declares an idempotency window of {w} epochs and \
                 `--idem-window` says {}. The declaration is the contract; remove the flag, \
                 or change the schema.",
                f.map(|n| n.to_string())
                    .unwrap_or_else(|| "unbounded".into())
            );
            std::process::exit(2);
        }
        (None, Some(Some(w))) => {
            eprintln!("  idempotency window: {w} epochs, from --idem-window");
            Some(w)
        }
        (None, Some(None)) => {
            eprintln!(
                "  idempotency window: UNBOUNDED (explicit) — every identity ever committed \
                 is kept, in both indexes, and the write path's memory grows with history"
            );
            None
        }
        (None, None) => {
            eprintln!(
                "nilestreamd: this schema's wire relation declares no `idem` column, so \
                 every identity ever committed would be kept forever in both idempotency \
                 indexes and the write path's memory would grow with history.\n\
                 \n\
                 Refusing to start rather than choosing that silently (LC-35). Declare \
                 `idem: IdemKey window N.epochs` on the relation, or pass `--idem-window N` \
                 — or `--idem-window unbounded` to say you meant it."
            );
            std::process::exit(2);
        }
    };
    let base = RevEngine::seeded(
        accounts,
        rounds,
        budget,
        mode,
        proto_engine::EvictionPolicy::Lru,
    )
    .with_idem_window(idem_window);
    let base = match &durable {
        None => {
            eprintln!(
                "  durability: VOLATILE (--volatile) — nothing is written to stable storage \
                 and no figure from this run is a contract result"
            );
            base
        }
        Some(path) => match base.with_durable_bounded(path, idem_window) {
            Ok(e) => {
                eprintln!("  durable sink at {path} — SyncPolicy::Always, fsync before publish");
                e
            }
            // Refuse rather than fall back to a volatile engine. A server that was asked for
            // durability and silently served without it is the single most direct way to
            // fabricate a durability number.
            Err(e) => {
                eprintln!("nilestreamd: cannot open a durable sink at {path}: {e}");
                std::process::exit(2);
            }
        },
    };
    let engine = Arc::new(base);

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("nilestreamd: cannot bind 127.0.0.1:{port}: {e}");
            std::process::exit(2);
        }
    };
    eprintln!("nilestreamd 0.1 — PostgreSQL wire protocol on 127.0.0.1:{port}");
    eprintln!(
        "  schema: {} ({} view(s), {} relation(s))",
        schema_path.as_deref().unwrap_or("<default>"),
        cat.views.len(),
        cat.relations.len()
    );
    {
        eprintln!(
            "  read path: partial view ({:?}, budget {budget}) over a hash-chained ledger, \
             frontier #{}",
            mode,
            engine.frontier()
        );
    }
    if durable.is_none() {
        eprintln!("  NOTE: nothing here is durable and there is no consensus: an epoch is");
        eprintln!("        acknowledged from memory, and a restart loses every row. Reads do");
        eprintln!("        run concurrently over the base. It serves the real REV mechanism");
        eprintln!("        -- partial state, honest absence, anchored reconstruction -- and it");
        eprintln!("        is not a production database.");
    } else {
        // **Two claims that were false when they were printed.** "Durable" meant the epoch
        // number reached stable storage, not the rows, so a restart recovered nothing (T-01);
        // and the engine mutex had been replaced by a reader-writer split two commits before
        // (T-06). A banner is the first thing an operator reads and the last thing anyone
        // re-checks.
        eprintln!("  NOTE: appends are durable -- rows are recorded and replayed on reopen --");
        eprintln!("        and reads run concurrently over the base. There is no consensus.");
        eprintln!("        `select nilestream_sealer` reports batching and lock contention.");
    }
    eprintln!("  try:  psql -h 127.0.0.1 -p {port} -U anyone bank");

    daemon::accept_loop(listener, schema, engine);
}

#[cfg(test)]
mod banner_tests {
    /// **The banner is the first thing an operator reads and the last thing anyone
    /// re-checks.**
    ///
    /// Two of its claims were false when they were printed: "durable" meant the epoch number
    /// reached stable storage and not the rows, so a restart recovered nothing; and it
    /// described an engine mutex that had been replaced by a reader-writer split two commits
    /// earlier. Both were fixed in cycle 7 and neither was guarded, so nothing stops the next
    /// one.
    ///
    /// Source-level, and deliberately so: the alternative is spawning the daemon to read its
    /// stderr, which is a slower test of a weaker property. What it asserts is the shape of
    /// the claim, not its wording — the banner may not name a mutex the engine does not have,
    /// and may not call the read side single-threaded when readers run concurrently.
    #[test]
    fn the_banner_describes_the_engine_this_binary_has() {
        let src = include_str!("main.rs");
        // Only `main`'s own body: this module's prose and its own needles are in this file
        // too, and a test that matched them would fail on itself.
        let body = &src[src.find("fn main()").expect("main")
            ..src.find("mod banner_tests").expect("this module")];
        let banner: String = body
            .lines()
            .filter(|l| l.trim_start().starts_with("eprintln!") || l.trim_start().starts_with('"'))
            .collect::<Vec<_>>()
            .join("\n");
        for (needle, why) in [
            (
                "mutex",
                "the engine has been behind an `RwLock` since cycle 6; a banner naming a mutex \
                 describes a serialisation this binary does not do",
            ),
            (
                "single-threaded",
                "readers run concurrently over the base, durable or not",
            ),
        ] {
            assert!(
                !banner.to_lowercase().contains(needle),
                "the banner says `{needle}`: {why}"
            );
        }
        // And the durability claim is only made when a sink is attached.
        let durable_claim = src
            .lines()
            .find(|l| l.contains("appends are durable"))
            .expect("the durable branch's NOTE");
        assert!(
            src[..src.find(durable_claim).expect("found above")].contains("if durable.is_none()"),
            "the `appends are durable` line must sit in the branch that has a durable sink: \
             printed unconditionally it is the claim T-01 found false for a whole cycle"
        );
    }
}
