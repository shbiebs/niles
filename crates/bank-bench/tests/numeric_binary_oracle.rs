//! **The `numeric` binary encoding, checked against the implementation it has to match.**
//!
//! `nilestream_server::pg_wire::binary::numeric` writes PostgreSQL's own decimal wire format
//! from exact minor units and a declared scale. A test that decoded our bytes with our own
//! decoder would prove the two halves agree with each other and nothing about whether either
//! is right; the format is defined by one implementation, so that implementation is the
//! oracle. This asks a running PostgreSQL to send the same values in binary and compares the
//! bytes.
//!
//! It is the money boundary. A rounding, a lost digit or a mis-signed zero here is a
//! rounding, a lost digit or a mis-signed zero in a balance, at the last hop where every
//! layer above has been careful.
//!
//! Skipped — loudly — when no server is reachable, because a test that passes because it did
//! nothing is worse than one that fails.

use bank_bench::wire::Client;

/// Values chosen for where a decimal encoder goes wrong: zero and its sign, a value whose
/// fraction fills exactly one base-10000 group and one that fills part of one, a value with
/// an all-zero interior group, negatives of each, and magnitudes past `i64` so the `i128`
/// path is real.
const CASES: &[(i128, u32)] = &[
    (0, 2),
    (0, 0),
    (1, 2),
    (-1, 2),
    (100, 2),
    (-100, 2),
    (150, 2),
    (12_345, 2),
    (-12_345, 2),
    (1_000_000, 2),
    (100_000_000, 2),
    (100_000_001, 2),
    // An interior group of zeros: 1.0000_0001 at scale 8.
    (100_000_001, 8),
    (999_999_999_999_999_999, 2),
    (-999_999_999_999_999_999, 2),
    // Past i64, so the i128 accumulation is exercised rather than assumed.
    (170_141_183_460_469_231_731_687_303_715_884_105, 2),
    (-170_141_183_460_469_231_731_687_303_715_884_105, 2),
    (7, 0),
    (-7, 0),
    (1, 9),
    (-1, 9),
];

fn port() -> u16 {
    std::env::var("PGPORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5432)
}

/// The decimal text of `minor` at `scale`, which is what PostgreSQL is asked to parse.
fn as_text(minor: i128, scale: u32) -> String {
    if scale == 0 {
        return minor.to_string();
    }
    let neg = minor < 0;
    let digits = minor.unsigned_abs().to_string();
    let digits = if digits.len() <= scale as usize {
        format!(
            "{}{}",
            "0".repeat(scale as usize + 1 - digits.len()),
            digits
        )
    } else {
        digits
    };
    let split = digits.len() - scale as usize;
    format!(
        "{}{}.{}",
        if neg { "-" } else { "" },
        &digits[..split],
        &digits[split..]
    )
}

#[test]
fn our_numeric_bytes_are_postgresqls_numeric_bytes() {
    let Ok(mut c) = Client::connect("127.0.0.1", port(), "bench", "postgres") else {
        panic!(
            "no PostgreSQL on 127.0.0.1:{} — this test is the only check that the money \
             encoding is right, and skipping it silently would leave that unchecked. Start a \
             server, or set PGPORT.",
            port()
        );
    };

    for &(minor, scale) in CASES {
        let text = as_text(minor, scale);
        // `numeric(p, s)` so the server's `dscale` is the declared one rather than whatever
        // the literal happened to carry — the same rule the engine follows, which is what
        // keeps `1.50` from being sent as `1.5`.
        let sql = format!("select {text}::numeric(40, {scale})");
        let stmt = c.prepare(&sql, &[]).expect("prepares");
        let rows = c.execute_binary(&stmt).expect("executes");
        let theirs = rows[0][0].as_ref().expect("a value").clone();

        let mut ours = Vec::new();
        nilestream_server::pg_wire::binary::numeric(minor, scale, &mut ours);

        assert_eq!(
            ours, theirs,
            "minor {minor} at scale {scale} ({text}): our bytes {ours:02x?} against \
             PostgreSQL's {theirs:02x?}"
        );
    }
}
