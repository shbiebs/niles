//! `nilestreamd` — the Nilestream daemon.
//!
//! Listens on TCP, speaks the PostgreSQL wire protocol, compiles each query as Niles, and
//! serves it from the REV runtime over a durable ledger. One thread per connection, which
//! is the right shape at this scale and is stated rather than defended: the write path is
//! serialised through a single sealer anyway, so connection-level concurrency is about
//! *reads*, and reads over an immutable base need no coordination at all.
//!
//! ```text
//! nilestreamd [--port N] [--schema FILE] [--accounts N] [--seed N]
//! ```
//!
//! Point `psql -h 127.0.0.1 -p 5433 -U anyone bank` at it.

#[path = "pg_wire.rs"]
mod pg_wire;
#[path = "session.rs"]
mod session;

use pg_wire::{Backend, Frontend};
use session::{MemoryEngine, Serving, Session};
use std::collections::HashMap;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

const DEFAULT_SCHEMA: &str = "\
schema bank {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }
    index ix_postings on postings (acct) anchor;
}
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 5433u16;
    let mut schema_path: Option<String> = None;
    let mut accounts = 1000i64;
    let mut i = 1;
    while i + 1 < args.len() {
        match args[i].as_str() {
            "--port" => port = args[i + 1].parse().unwrap_or(port),
            "--schema" => schema_path = Some(args[i + 1].clone()),
            "--accounts" => accounts = args[i + 1].parse().unwrap_or(accounts),
            _ => {}
        }
        i += 2;
    }

    let schema = match &schema_path {
        Some(p) => std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("nilestreamd: cannot read {p}: {e}");
            std::process::exit(2);
        }),
        None => DEFAULT_SCHEMA.to_string(),
    };

    // Refuse to start on a schema that does not compile. A server that accepted a broken
    // schema would fail on the first query instead, at which point the operator is
    // debugging a connection rather than a file.
    let (prog, mut d) = niles_lang::parser::parse_program(&schema);
    let (cat, rd) = niles_lang::resolve::resolve_program(&prog, 0);
    d.extend(rd);
    if d.has_errors() {
        eprint!("{}", d.render(&schema, schema_path.as_deref().unwrap_or("<default>")));
        eprintln!("nilestreamd: the schema does not compile; refusing to start");
        std::process::exit(1);
    }

    // A seeded balance set, so a connecting client has something to read. This is a demo
    // engine, and the banner says so: a reader who mistook it for a durable one would
    // draw a wrong conclusion from every number it returned.
    let mut data = HashMap::new();
    for a in 1..=accounts {
        data.insert(("__wire_result".to_string(), a), (a as i128) * 100);
    }
    let engine = Arc::new(Mutex::new(MemoryEngine {
        frontier: 1,
        data,
        views: cat.views.keys().map(|k| (k.clone(), 2)).collect(),
    }));

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("nilestreamd: cannot bind 127.0.0.1:{port}: {e}");
            std::process::exit(2);
        }
    };
    eprintln!("nilestreamd 0.1 — PostgreSQL wire protocol on 127.0.0.1:{port}");
    eprintln!("  schema: {} ({} view(s), {} relation(s))", schema_path.as_deref().unwrap_or("<default>"), cat.views.len(), cat.relations.len());
    eprintln!("  NOTE: the read side is an in-memory demo engine. It is not durable, and no");
    eprintln!("        number it returns should be read as a measurement of a database.");
    eprintln!("  try:  psql -h 127.0.0.1 -p {port} -U anyone bank");

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let schema = schema.clone();
        let engine = Arc::clone(&engine);
        std::thread::spawn(move || {
            if let Err(e) = serve(stream, schema, engine) {
                eprintln!("nilestreamd: connection ended: {e}");
            }
        });
    }
}

fn serve(stream: TcpStream, schema: String, engine: Arc<Mutex<MemoryEngine>>) -> std::io::Result<()> {
    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    let mut w = stream.try_clone()?;
    let mut r = BufReader::new(stream);

    // The startup exchange. `SSLRequest` is answered with a single `N` — a refusal, not
    // silence — because a client that gets no answer hangs.
    let mut startup = pg_wire::read_startup(&mut r)?;
    if startup == Frontend::SslRequest {
        w.write_all_bytes(b"N")?;
        startup = pg_wire::read_startup(&mut r)?;
    }
    let Frontend::Startup { params, .. } = startup else {
        return Ok(());
    };
    let get = |k: &str| params.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone()).unwrap_or_default();
    let mut session = Session::new(get("user"), get("database"), schema);
    eprintln!("nilestreamd: {peer} connected as `{}` to `{}`", session.user, session.database);

    let pid = std::process::id();
    pg_wire::write_all(&mut w, &pg_wire::startup_reply(pid, 0x5eed))?;

    loop {
        let msg = match pg_wire::read_message(&mut r) {
            Ok(m) => m,
            Err(_) => break,
        };
        if msg == Frontend::Terminate {
            break;
        }
        let replies = {
            let mut e = engine.lock().unwrap();
            session.handle(msg, &mut *e)
        };
        if replies.is_empty() {
            break;
        }
        pg_wire::write_all(&mut w, &replies)?;
    }
    eprintln!("nilestreamd: {peer} disconnected after {} queries", session.queries_served);
    Ok(())
}

/// A tiny helper so the `SSLRequest` refusal reads clearly at the call site.
trait WriteBytes {
    fn write_all_bytes(&mut self, b: &[u8]) -> std::io::Result<()>;
}
impl<T: std::io::Write> WriteBytes for T {
    fn write_all_bytes(&mut self, b: &[u8]) -> std::io::Result<()> {
        self.write_all(b)?;
        self.flush()
    }
}

#[allow(dead_code)]
fn _assert_backend_is_used(_: Backend) {}
#[allow(dead_code)]
fn _assert_serving_is_used(_: &dyn Serving) {}
