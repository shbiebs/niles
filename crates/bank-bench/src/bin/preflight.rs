//! `make preflight`'s database half: can the harness reach PostgreSQL as it will during the
//! gate? Connects as `bench` to `bank` and to `postgres` (the numeric oracle's database) and
//! prints how each authenticated. Exits non-zero with every failure named at once, so a fresh
//! machine is provisioned in one pass rather than one gate failure at a time (cycle 14, D-1).

use bank_bench::wire::Client;

fn main() {
    let port: u16 = std::env::var("PGPORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5432);
    let mut failures = Vec::new();
    for db in ["bank", "postgres"] {
        match Client::connect("127.0.0.1", port, "bench", db) {
            Ok(c) => println!("preflight: bench@127.0.0.1:{port}/{db} ok ({:?})", c.auth),
            Err(e) => failures.push(format!("bench@127.0.0.1:{port}/{db}: {e}")),
        }
    }
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("preflight: {f}");
        }
        eprintln!("preflight: run tools/pg-provision.sh (it never edits pg_hba.conf)");
        std::process::exit(1);
    }
}
