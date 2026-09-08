//! **The server's front door refuses what it cannot honour — F-70, LC-16, LC-35.**
//!
//! Every numeric flag used `parse().unwrap_or(default)`. `--budget 2,500` — a comma, the
//! ordinary way a person writes that number — ran the phase diagram's lever at its default
//! and said nothing; a mistyped `--port` served somewhere else and the operator's client
//! could not connect to a server that was running perfectly well. The engine's own standing
//! rule is honest refusal over silent fallback, and its front door was the one place that
//! broke it.
//!
//! Two contracts are decided here as well. **LC-16:** durability is not a default — a server
//! that serves volatile because nobody said otherwise has a guarantee that depends on what an
//! operator forgot to type, and a client cannot tell from outside. **LC-35:** neither is an
//! unbounded idempotency window — it costs memory that grows with history, and a bound
//! nobody can see is a bound nobody can act on.
//!
//! Driven as a process, because that is what an operator drives. A test that called a parsing
//! function would be testing a function; what refuses is a command.

use std::process::Command;

/// Run the daemon with these arguments and return `(exit code, stderr)`.
///
/// Every case here must fail *before* binding a socket, so nothing needs a port to be free
/// and nothing is left running.
fn refused(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_nilestreamd"))
        .args(args)
        .output()
        .expect("the daemon binary runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn a_numeric_flag_that_does_not_parse_is_refused_by_name() {
    for (flag, bad) in [
        ("--port", "54,33"),
        ("--accounts", "10_000"),
        ("--rounds", "two"),
        ("--budget", "2,500"),
    ] {
        let (code, err) = refused(&["--volatile", "--idem-window", "unbounded", flag, bad]);
        assert_ne!(
            code, 0,
            "`{flag} {bad}` started the server. A flag that falls back to its default runs \
             the server on a configuration nobody asked for and reports nothing: {err}"
        );
        assert!(
            err.contains(flag) && err.contains(bad),
            "the refusal must name the flag and the text that did not parse; got: {err}"
        );
    }
}

#[test]
fn an_unknown_flag_is_refused_rather_than_ignored() {
    let (code, err) = refused(&["--volatile", "--idem-window", "unbounded", "--budgt", "10"]);
    assert_ne!(
        code, 0,
        "an ignored flag is a configuration the operator believes is in effect"
    );
    assert!(err.contains("--budgt"), "{err}");
}

#[test]
fn a_mode_that_is_neither_full_nor_demand_is_refused() {
    let (code, err) = refused(&[
        "--volatile",
        "--idem-window",
        "unbounded",
        "--mode",
        "partial",
    ]);
    assert_ne!(code, 0, "`--mode partial` used to mean `demand`, silently");
    assert!(err.contains("--mode"), "{err}");
}

/// **LC-16, decided: durability is stated or refused.**
#[test]
fn a_server_with_neither_durable_nor_volatile_refuses_to_start() {
    let (code, err) = refused(&["--idem-window", "unbounded"]);
    assert_ne!(
        code, 0,
        "the server started without a durable sink and without being told to: its \
         acknowledgement then means nothing a client can rely on"
    );
    assert!(
        err.contains("--durable") && err.contains("--volatile"),
        "the refusal must name both ways out: {err}"
    );
}

#[test]
fn durable_and_volatile_together_are_contradictory() {
    let (code, _) = refused(&[
        "--durable",
        "/nonexistent/never-created.seg",
        "--volatile",
        "--idem-window",
        "unbounded",
    ]);
    assert_ne!(code, 0);
}

/// **LC-35, decided: an unbounded idempotency window is stated or refused.**
///
/// The shipped schema declares a window, so this needs one that does not.
#[test]
fn a_schema_with_no_declared_window_is_refused_unless_the_flag_says_unbounded() {
    let dir = std::env::temp_dir().join(format!("nls-flags-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let schema = dir.join("no-window.niles");
    std::fs::write(
        &schema,
        "schema s {\n    currency usd { scale: 2 }\n    ledger postings {\n        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,\n        conserve per (txn, cur);\n        retain forever;\n    }\n}\n",
    )
    .expect("write the schema");
    let path = schema.to_str().expect("utf-8");

    let (code, err) = refused(&["--volatile", "--schema", path]);
    assert_ne!(
        code, 0,
        "a schema with no declared window keeps every identity ever committed, forever, in \
         both indexes. Choosing that silently is what LC-35 refuses: {err}"
    );
    assert!(
        err.contains("--idem-window"),
        "the refusal must name the way to say you meant it: {err}"
    );
    let _ = std::fs::remove_file(&schema);
}

/// And the flag may not contradict a schema that does declare one.
#[test]
fn a_flag_that_disagrees_with_the_schemas_declared_window_is_refused() {
    let (code, err) = refused(&["--volatile", "--idem-window", "42"]);
    assert_ne!(
        code, 0,
        "the shipped schema declares 1,000,000 epochs; a flag saying 42 is a contradiction, \
         and silently preferring either one is a window nobody declared: {err}"
    );
    assert!(
        err.contains("1000000") || err.contains("1_000_000"),
        "{err}"
    );
}
