//! The harness reaches PostgreSQL through SCRAM-SHA-256, not through a weakened
//! `pg_hba.conf` (cycle 14, decision D-1).
//!
//! Until cycle 14 the harness spoke only `trust`, so every machine that ran the gate had had
//! its server's authentication weakened by hand, and nothing in the repository said so. This
//! test holds the replacement: against the provisioned role (`tools/pg-provision.sh`) the
//! connection must complete a verified SCRAM exchange, and a wrong password must be refused
//! by the server rather than slip through.
//!
//! A machine whose server is deliberately `trust` — some desktop PostgreSQL distributions
//! ship that way for local sockets — sets `NILES_PG_TRUST_OK=1`, and the test says what it
//! skipped. The escape is named so that using it is a visible decision, not a default.

use bank_bench::wire::{AuthMethod, Client, WireError};

fn port() -> u16 {
    std::env::var("PGPORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5432)
}

#[test]
fn the_harness_reaches_postgresql_through_scram_and_a_wrong_password_is_refused() {
    let c = match Client::connect("127.0.0.1", port(), "bench", "bank") {
        Ok(c) => c,
        Err(e) => panic!(
            "no PostgreSQL reachable as `bench` on 127.0.0.1:{}: {e}. Run tools/pg-provision.sh \
             (make preflight names what is missing).",
            port()
        ),
    };
    if c.auth == AuthMethod::Trust {
        assert!(
            std::env::var("NILES_PG_TRUST_OK").as_deref() == Ok("1"),
            "PostgreSQL accepted `bench` without authentication. The gate is meant to pass \
             against a server with its stock scram-sha-256 rule; set NILES_PG_TRUST_OK=1 only \
             on a machine whose server is trust by design, and say so in the run's log"
        );
        eprintln!("NILES_PG_TRUST_OK=1: the server is trust; the SCRAM half was not exercised");
        return;
    }
    assert_eq!(c.auth, AuthMethod::ScramSha256);
    drop(c);

    // The same exchange with a wrong password. `PGPASSWORD` wins over the file, so setting it
    // here is enough; it is restored afterwards. This is the only test in this binary, so no
    // other test reads the environment concurrently.
    let saved = std::env::var("PGPASSWORD").ok();
    std::env::set_var("PGPASSWORD", "definitely-not-the-provisioned-password");
    let refused = Client::connect("127.0.0.1", port(), "bench", "bank");
    match saved {
        Some(p) => std::env::set_var("PGPASSWORD", p),
        None => std::env::remove_var("PGPASSWORD"),
    }
    match refused {
        Err(WireError::Server { sqlstate, .. }) => assert_eq!(
            sqlstate, "28P01",
            "a wrong password is `invalid_password`, not some other failure"
        ),
        Err(other) => panic!("a wrong password failed for another reason: {other}"),
        Ok(_) => panic!("a wrong password was accepted"),
    }
}
