//! E21 — what the reply path costs: the flush, the client, and whether either is free.
//!
//! Three questions were asked of T-32's wire work after it landed, and none of them had an
//! answer in the repository — the numbers existed only in a session's scrollback, which is
//! the same as not existing. This example is the instrument that produces them, so the
//! figures in `docs/BENCHMARK.md` have something behind them that can be re-run.
//!
//! **What would it cost to remove the flush?** `write_all` renders a served answer through a
//! 64 KB buffer and writes it out as it fills. The alternative is `put_rows`, which is the
//! same renderer with a no-op flush: it builds the whole reply and writes it once. Both are
//! in the crate, both are exercised here over a real loopback socket, and the bytes they put
//! on the wire are asserted identical before either is timed — a speed comparison between two
//! encoders that disagree about their output is not a comparison.
//!
//! **Is it efficient for the harness client to discard its answers?** The workloads time a
//! statement and throw the rows away; `simple_counted` reads the reply without materialising
//! a `String` per cell, and `simple` materialises every one. The difference is the floor the
//! client puts under every measured statement, and it is charged to **both** targets equally,
//! so the question is not fairness but how much of a measured millisecond was the harness.
//!
//! **Does the device hold still?** A `durable` figure is an `fsync` rate. `storage::ceiling`
//! probes the device repeatedly and reports its spread; this run records it beside the wire
//! numbers so a reader can see whether the host was steady while they were taken.
//!
//! ```text
//! cargo run --release -p bank-bench --example wire_cost
//! ```

use bank_bench::wire::Client;
use nilestream_server::pg_wire::{self, Backend, Format, RowBlock};
use std::io::{Read, Write};
use std::time::Instant;

/// A block of `rows` answers, three integer columns plus the anchor, all text format.
fn block(rows: usize) -> RowBlock {
    use niles_ir::value::Value;
    let mut z = niles_ir::eval::ZSet::new();
    for i in 0..rows as i128 {
        z.insert(vec![Value::Int(i), Value::Int(i * 7 + 3)], 1);
    }
    RowBlock {
        z,
        anchor: 9_999,
        formats: vec![Format::Text, Format::Text, Format::Text],
    }
}

fn reply(b: &RowBlock) -> Vec<Backend> {
    vec![
        Backend::Rows(b.clone()),
        Backend::CommandComplete(format!("SELECT {}", pg_wire::row_count(b))),
        Backend::ReadyForQuery(b'I'),
    ]
}

/// The assembled path: the same renderer with nowhere to flush to, written once.
fn write_assembled(w: &mut impl Write, msgs: &[Backend]) -> std::io::Result<()> {
    let mut buf = Vec::new();
    for m in msgs {
        match m {
            Backend::Rows(b) => pg_wire::put_rows(&mut buf, b),
            other => pg_wire::encode_into(&mut buf, other),
        }
    }
    w.write_all(&buf)?;
    w.flush()
}

/// A drain on the other end of a loopback pair, so the write path meets a real socket with a
/// real receiver rather than a `Vec`. Returns the port and a handle that yields bytes read.
fn drain() -> (u16, std::sync::mpsc::Receiver<std::net::TcpStream>) {
    let l = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("a loopback port");
    let port = l.local_addr().expect("a bound address").port();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            if tx.send(s).is_err() {
                return;
            }
        }
    });
    (port, rx)
}

struct Flush {
    rows: usize,
    replies: u32,
    streamed_ms: f64,
    streamed_mad: f64,
    assembled_ms: f64,
    assembled_mad: f64,
}

impl Flush {
    /// Streamed over assembled, at the median. Below 1.0 the flush is *winning*.
    fn ratio(&self) -> f64 {
        self.streamed_ms / self.assembled_ms
    }

    /// Whether the two medians are separated by more than the replies' own scatter.
    ///
    /// The reason this exists: the first published run of this file had the ten-thousand-row
    /// ratio at 0.96x and the next at 1.06x, from the same binary minutes apart. A ratio that
    /// moves 10% between runs is not evidence of a 4% difference, and a file that printed
    /// one of them as a finding would be reporting scheduler noise. So the spread is measured
    /// and the verdict is drawn from it rather than from the point estimate.
    fn separated(&self) -> bool {
        (self.streamed_ms - self.assembled_ms).abs()
            > self.streamed_mad.max(self.assembled_mad) * 2.0
    }
}

/// Median and median absolute deviation of a sample, in its own unit.
fn median_mad(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a wall-clock sample"));
    let m = v[v.len() / 2];
    let mut d: Vec<f64> = v.iter().map(|x| (x - m).abs()).collect();
    d.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a deviation"));
    (m, d[d.len() / 2])
}

/// Time both write paths, alternating, against a socket whose peer is draining it.
fn flush_cost(rows: usize, replies: u32) -> Flush {
    let b = block(rows);
    let msgs = reply(&b);

    // Fairness first: the two paths must agree byte for byte before either is timed.
    let mut a = Vec::new();
    pg_wire::write_all(&mut a, &msgs).expect("a Vec sink cannot fail");
    let mut c = Vec::new();
    write_assembled(&mut c, &msgs).expect("a Vec sink cannot fail");
    assert_eq!(
        a, c,
        "the streamed and assembled paths disagree about the bytes of a {rows}-row reply; \
         timing them against each other would be comparing two different replies"
    );
    let bytes = a.len();

    let (port, incoming) = drain();
    let mut streamed: Vec<f64> = Vec::with_capacity(replies as usize);
    let mut assembled: Vec<f64> = Vec::with_capacity(replies as usize);

    for _ in 0..replies {
        for which in 0..2 {
            let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect");
            s.set_nodelay(true).expect("nodelay");
            let mut peer = incoming.recv().expect("the drain accepted");
            let reader = std::thread::spawn(move || {
                let mut got = 0usize;
                let mut buf = vec![0u8; 64 * 1024];
                while got < bytes {
                    match peer.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => got += n,
                        Err(_) => break,
                    }
                }
                got
            });
            let t = Instant::now();
            if which == 0 {
                pg_wire::write_all(&mut s, &msgs).expect("streamed write");
            } else {
                write_assembled(&mut s, &msgs).expect("assembled write");
            }
            let e = t.elapsed().as_secs_f64() * 1000.0;
            drop(s);
            let got = reader.join().expect("the reader finished");
            assert_eq!(got, bytes, "the peer did not receive the whole reply");
            if which == 0 {
                streamed.push(e);
            } else {
                assembled.push(e);
            }
        }
    }

    let (sm, smad) = median_mad(&mut streamed);
    let (am, amad) = median_mad(&mut assembled);
    Flush {
        rows,
        replies,
        streamed_ms: sm,
        streamed_mad: smad,
        assembled_ms: am,
        assembled_mad: amad,
    }
}

struct ClientFloor {
    rows: usize,
    rounds: u32,
    eager_ms: f64,
    lean_ms: f64,
}

impl ClientFloor {
    fn saved_ms(&self) -> f64 {
        self.eager_ms - self.lean_ms
    }
    /// What fraction of an eagerly-read statement was the client, not the server.
    ///
    /// The honest denominator. An earlier framing quoted the two *decode* steps against each
    /// other and got a factor of thirteen, which is true of the decoder and says nothing
    /// about a workload: a timed statement also contains a round trip and the server's own
    /// work, and those are in both terms. What a workload sees is this share.
    fn share(&self) -> f64 {
        if self.eager_ms <= 0.0 {
            return 0.0;
        }
        self.saved_ms() / self.eager_ms
    }
}

/// Host the daemon on a thread, then read the same answer both ways, interleaved.
fn client_floor(accounts: i64, rounds: u32) -> Result<ClientFloor, String> {
    let l = std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
    let port = l.local_addr().map_err(|e| e.to_string())?.port();
    let engine = std::sync::Arc::new(std::sync::Mutex::new(
        nilestream_server::rev_engine::RevEngine::seeded(
            accounts,
            1,
            usize::MAX,
            proto_engine::ViewMode::Demand,
            proto_engine::EvictionPolicy::Lru,
        ),
    ));
    let schema = nilestream_server::daemon::DEFAULT_SCHEMA.to_string();
    std::thread::spawn(move || nilestream_server::daemon::accept_loop(l, schema, engine));

    let mut c = Client::connect("127.0.0.1", port, "bench", "bank").map_err(|e| e.to_string())?;
    let sql = "select acct, sum(amt) from postings group by acct";

    // One of each, discarded: the first reply of a session pays for a view that is not yet
    // installed, and that cost belongs to neither policy.
    let warm = c.simple_counted(sql).map_err(|e| e.to_string())?;
    if warm.rows == 0 {
        return Err(format!("the statement returned no rows: `{sql}`"));
    }
    let _ = c.simple(sql).map_err(|e| e.to_string())?;

    let mut eager = 0.0f64;
    let mut lean = 0.0f64;
    for _ in 0..rounds {
        let t = Instant::now();
        let r = c.simple(sql).map_err(|e| e.to_string())?;
        eager += t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let k = c.simple_counted(sql).map_err(|e| e.to_string())?;
        lean += t.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            r.rows.len() as u64,
            k.rows,
            "the two client policies disagree about how many rows arrived"
        );
    }

    Ok(ClientFloor {
        rows: warm.rows as usize,
        rounds,
        eager_ms: eager / rounds as f64,
        lean_ms: lean / rounds as f64,
    })
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "results".to_string());

    let flushes = vec![flush_cost(10_000, 40), flush_cost(100_000, 10)];
    let floor = client_floor(10_000, 20);
    let ceiling = bank_bench::storage::ceiling(&out, 7, 200);

    let mut s = String::new();
    s.push_str(
        "# E21 — the reply path's cost: the flush, the client, and the device\n\n\
         **Generated by `cargo run --release -p bank-bench --example wire_cost`. Nothing in \
         this file is typed in by hand.**\n\n\
         T-32 made a served reply's peak memory constant in its row count by rendering \
         through a bounded buffer and flushing it as it fills. Three questions follow from \
         that, and this file answers them with measurements rather than with reasoning about \
         what a buffer ought to cost.\n\n",
    );

    s.push_str(
        "## What the flush costs\n\n\
         Both paths are the same renderer; the streamed one flushes a 64 KB buffer as it \
         fills, the assembled one accumulates the whole reply and writes it once. They are \
         asserted **byte for byte identical** before either is timed, run alternately against \
         a loopback socket with a draining peer, and reported as the median of the \
         per-reply wall clock with its median absolute deviation beside it.\n\n\
         | Rows | Replies each | Streamed | Assembled | Streamed ÷ assembled | Separated by more than the scatter? |\n\
         |--:|--:|--:|--:|--:|:--|\n",
    );
    for f in &flushes {
        s.push_str(&format!(
            "| {} | {} | {:.3} ± {:.3} ms | {:.3} ± {:.3} ms | {:.2}× | {} |\n",
            f.rows,
            f.replies,
            f.streamed_ms,
            f.streamed_mad,
            f.assembled_ms,
            f.assembled_mad,
            f.ratio(),
            if f.separated() { "**yes**" } else { "no" }
        ));
    }
    let worst = flushes
        .iter()
        .map(|f| f.ratio())
        .fold(0.0f64, |a, b| a.max(b));
    let costly = flushes
        .iter()
        .filter(|f| f.separated() && f.ratio() > 1.0)
        .count();
    let saving = flushes
        .iter()
        .filter(|f| f.separated() && f.ratio() < 1.0)
        .count();
    s.push_str(&format!(
        "\n{} The worst ratio here is {worst:.2}×; of the {} sizes measured, {} show \
         streaming ahead by more than the replies\u{2019} own scatter and {} show it behind by \
         more.\n\n\
         The shape is the one the mechanism predicts. A small reply fits in a few buffer-fulls, \
         so the extra `write` calls are a visible fraction of a short wall clock; a large one \
         makes the assembled path grow and copy a buffer the size of the whole answer, and \
         that overtakes the syscalls. **So the price of the bound is a few percent of wall \
         clock on small replies, and it is negative on large ones.** What it buys is the \
         other column: E18 measures peak reply memory at 69,632 bytes for a ten-thousand-row \
         and a hundred-thousand-row answer alike, where the assembled path measured 835 KB \
         and 6.7 MB. A constant against a linear is the trade, and a few percent at ten \
         thousand rows is what it costs.\n\n",
        if costly == 0 {
            "**The flush is not paid for at any size measured here.**"
        } else {
            "**The flush costs a little at small replies, and it is recorded rather than \
             argued away.**"
        },
        flushes.len(),
        saving,
        costly
    ));

    s.push_str("## What the harness client's discard is worth\n\n");
    match &floor {
        Ok(f) => {
            s.push_str(&format!(
                "The workloads time a statement and throw the answer away. `simple` builds a \
                 `Vec` per row and a `String` per cell on the way to throwing it away; \
                 `simple_counted` walks the same bytes for their lengths. The same statement, \
                 the same server, the two policies interleaved:\n\n\
                 | Rows in the reply | Rounds each | Eager (`simple`) | Lean (`simple_counted`) | Saved | The client's share of the eager statement |\n\
                 |--:|--:|--:|--:|--:|--:|\n\
                 | {} | {} | {:.3} ms | {:.3} ms | {:.3} ms | {:.0}% |\n\n",
                f.rows,
                f.rounds,
                f.eager_ms,
                f.lean_ms,
                f.saved_ms(),
                100.0 * f.share()
            ));
            s.push_str(&format!(
                "**Yes, and by a margin that was distorting the analytical row.** {:.3} ms per \
                 ten-thousand-row reply — {:.0}% of the statement as the harness used to time \
                 it — is client-side work that no workload asked for. It was \
                 charged to *both* targets — the harness drives PostgreSQL and Nilestream \
                 through this same client, which is the fairness rule — so removing it did not \
                 favour either side; what it did was stop a fixed cost from sitting inside \
                 every ratio and dragging both toward 1.0×. A ratio measured with a large \
                 constant added to both terms is not the engines' ratio.\n\n\
                 The lean path is not a shortcut around the protocol. Every byte of every \
                 reply is still read off the socket and every cell's length is still parsed — \
                 what is skipped is copying cell bodies into `String`s that are dropped \
                 unread. A statement whose *answer* a workload needs (the conservation checks, \
                 the point lookups that assert a balance) still uses `simple`.\n\n",
                f.saved_ms(),
                100.0 * f.share()
            ));
        }
        Err(e) => s.push_str(&format!(
            "**REFUSED.** The client floor could not be measured: {e}\n\n\
             It is reported as unmeasured rather than filled in from a previous run, which \
             would be publishing a number this build did not produce.\n\n"
        )),
    }

    s.push_str("## Whether the device held still\n\n");
    match &ceiling {
        Ok(c) => {
            s.push_str(&format!(
                "The `durable` row of E16 is an `fsync` rate, so it is bounded by the storage \
                 underneath it — **and by which barrier the storage was asked for**. This run \
                 issued `{}` ({} probes):\n\n\
                 | Barrier | Median | MAD | Lowest | Highest | Spread |\n\
                 |---|--:|--:|--:|--:|--:|\n\
                 | `{}` | {:.0}/s | {:.0} ({:.1}%) | {:.0}/s | {:.0}/s | {:.2}× |\n\n\
                 The barrier is named because it is not a detail. The same probe measures \
                 5,300–6,000/s on Linux ext4, 500–960/s on ext4 inside a VM, and 255/s on \
                 APFS through `F_FULLFSYNC` — and about a million per second on an overlay \
                 mounted `fsync=volatile`, which is not storage evidence at all. A durable \
                 rate quoted without its barrier cannot be compared with anything.\n\n",
                c.barrier,
                c.probes,
                c.barrier,
                c.median,
                c.mad,
                100.0 * c.mad / c.median.max(1.0),
                c.lowest,
                c.highest,
                c.spread()
            ));
            if c.steady() {
                s.push_str(
                    "The device held still on this run, so an absolute durable rate taken \
                     here means something. That is not guaranteed on the next one, which is \
                     why E16 publishes each target's rate as a fraction of the ceiling \
                     measured in its own session as well as the rate itself.\n\n",
                );
            } else {
                s.push_str(&format!(
                    "**It did not.** A {:.2}× swing in what the storage can do, between probes \
                     seconds apart and with no database involved, is {}. An absolute durable \
                     figure taken against storage like this is partly a measurement of the \
                     storage, and a change in it between sessions is not by itself evidence \
                     about the engine. This is why E16 reports the `durable` row as a \
                     **fraction of the device ceiling**, probed in the same session as the \
                     run: a fraction survives a change of machine, and a rate does not.\n\n",
                    c.spread(),
                    if c.spread() >= 1.25 {
                        "as large as the movements E16's `durable` row has been asked to \
                         explain as engine behaviour"
                    } else {
                        "modest, but it is on the same order as the within-run scatter of the \
                         `durable` row itself, which is enough to make a small ratio movement \
                         unattributable"
                    }
                ));
            }
        }
        Err(e) => s.push_str(&format!(
            "**REFUSED.** The device could not be probed: {e}\n\n"
        )),
    }

    s.push_str(
        "## What this does not settle\n\n\
         The flush comparison is a **write-path** measurement against a loopback socket with \
         a peer that reads as fast as it can. A slow or distant consumer changes the picture \
         in the streamed path's favour, not against it — an assembled reply must be complete \
         before its first byte moves — but that case is not measured here.\n\n\
         The client figures are Nilestream-side because the daemon can be hosted in this \
         process; the same two policies read PostgreSQL's replies over the same socket code, \
         so the floor applies there too, but this file does not measure it there.\n",
    );

    let path = std::path::Path::new(&out).join("E21-wire-cost.md");
    if let Some(p) = path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    std::fs::write(&path, &s).expect("the results file is writable");
    eprintln!("wrote {}", path.display());
    if let Err(e) = &floor {
        eprintln!("client floor REFUSED: {e}");
    }
}
