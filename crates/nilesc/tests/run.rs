//! **`nilesc run`, driven as a process.**
//!
//! The consumer of this subcommand is a test in another repository that shells out to the
//! binary, reads one line of hex from stdout and compares it with bytes a different encoder
//! produced. So the tests here drive the binary the same way: arguments, files, stdout,
//! stderr, exit code. A test that called `run_function` in-process would be testing a
//! function; what the conformance suite depends on is a *command*.
//!
//! Each exit code is exercised, because the caller's next move depends on which it got:
//!
//! * `0` — the set is on stdout.
//! * `1` — the function ran and the ledger refused it, or the file does not check.
//! * `2` — the run could not happen: no such function, a bad fixture, or a construct outside
//!   the interpretable subset. `NotInSubset` is the one that becomes a recorded language gap
//!   rather than a defect, and it is the one that must be distinguishable.

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// A scratch file that removes itself.
struct Temp(PathBuf);

impl Temp {
    fn new(name: &str, contents: &str) -> Temp {
        let mut p = std::env::temp_dir();
        let salt = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("nilesc-run-{name}-{salt}-{}", std::process::id()));
        let mut f = std::fs::File::create(&p).expect("create the scratch file");
        f.write_all(contents.as_bytes()).expect("write it");
        Temp(p)
    }
    fn path(&self) -> &str {
        self.0.to_str().expect("a utf-8 path")
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn nilesc(args: &[&str]) -> Out {
    let exe = env!("CARGO_BIN_EXE_nilesc");
    let o = Command::new(exe)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("cannot run {exe}: {e}"));
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

const TRANSFER: &str = r#"
schema bank {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }
}

fn transfer(from: Id<Account>, to: Id<Account>, amount: Money<usd>)
    -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("transfer", window: 30.days) {
        let d = debit(from, amount)?;
        let c = credit(to, amount);
        post(d, c)
    }
}

// Two *independent* amounts, so the currency row cannot decide the question statically: the
// calculus proves conservation when one binding is used twice and reports "undecided" when two
// unrelated ones are, which is exactly the case a runtime seal exists to catch. Writing this
// with literals instead — `debit 10.00, credit 9.00` — makes NL0300 fire at compile time and
// the file never runs at all, which is the checker being right and is a different test.
fn skew(from: Id<Account>, to: Id<Account>, out: Money<usd>, back: Money<usd>)
    -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("skew") {
        let d = debit(from, out)?;
        let c = credit(to, back);
        post(d, c)
    }
}

fn authorises(acct: Id<Account>, amount: Money<usd>)
    -> Result<TxnId, TxnError>
    ! { append, hold<usd> }
{
    let h = hold(acct, amount, expires: 7.days)?;
    resolve h post 43.17 usd
}
"#;

fn args_file(contents: &str) -> Temp {
    Temp::new("args", contents)
}

#[test]
fn a_function_runs_and_prints_the_canonical_encoding_of_what_it_posted() {
    let src = Temp::new("src.niles", TRANSFER);
    let args = args_file("acct a.usd\nacct b.usd\nmoney 25000 usd 2\n");
    let out = nilesc(&["run", src.path(), "transfer", "--args", args.path()]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    let hex = out.stdout.trim();
    assert!(!hex.is_empty(), "nothing on stdout");
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "stdout must be one line of hex: {hex:?}"
    );

    // The layout, read back from the hex rather than asserted as an opaque blob — otherwise
    // this test pins a string and says nothing about what the string means.
    let bytes = hex_bytes(hex);
    assert_eq!(&bytes[0..4], &8u32.to_be_bytes(), "txn_len");
    assert_eq!(&bytes[4..12], b"transfer", "the idem key is the identity");
    assert_eq!(&bytes[12..16], &2u32.to_be_bytes(), "two legs");
}

/// The same function, twice, byte for byte. A conformance fixture is worth nothing if the
/// thing producing it can vary between runs.
#[test]
fn two_runs_of_one_function_produce_the_same_bytes() {
    let src = Temp::new("src.niles", TRANSFER);
    let args = args_file("acct a.usd\nacct b.usd\nmoney 25000 usd 2\n");
    let a = nilesc(&["run", src.path(), "transfer", "--args", args.path()]);
    let b = nilesc(&["run", src.path(), "transfer", "--args", args.path()]);
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(a.code, 0);
}

/// The arguments reach the postings: a different amount is different bytes.
///
/// Without this, `--args` could be ignored entirely and every test above would still pass.
#[test]
fn the_arguments_reach_the_postings() {
    let src = Temp::new("src.niles", TRANSFER);
    let one = args_file("acct a.usd\nacct b.usd\nmoney 25000 usd 2\n");
    let two = args_file("acct a.usd\nacct b.usd\nmoney 25001 usd 2\n");
    let a = nilesc(&["run", src.path(), "transfer", "--args", one.path()]);
    let b = nilesc(&["run", src.path(), "transfer", "--args", two.path()]);
    assert_eq!((a.code, b.code), (0, 0));
    assert_ne!(a.stdout, b.stdout, "one cent must change the bytes");

    let names = args_file("acct x.usd\nacct b.usd\nmoney 25000 usd 2\n");
    let c = nilesc(&["run", src.path(), "transfer", "--args", names.path()]);
    assert_ne!(
        a.stdout, c.stdout,
        "a different account must change the bytes"
    );
}

/// A set that does not conserve is refused, with the currency and the residual named, and
/// exits 1 — a wrong answer, not a missing capability.
#[test]
fn a_set_that_does_not_conserve_is_refused_and_exits_one() {
    let src = Temp::new("src.niles", TRANSFER);
    let args = args_file("acct a.usd\nacct b.usd\nmoney 1000 usd 2\nmoney 900 usd 2\n");
    let out = nilesc(&["run", src.path(), "skew", "--args", args.path()]);
    assert_eq!(out.code, 1, "stdout: {} stderr: {}", out.stdout, out.stderr);
    assert!(
        out.stderr.contains("USD") && out.stderr.contains("100"),
        "the refusal must name the currency and the residual: {}",
        out.stderr
    );
    assert!(out.stdout.trim().is_empty(), "and print no set");
}

/// A construct outside the subset exits **2** and names the construct.
///
/// This is the distinction the whole exit-code scheme exists for. `hold` is a real Niles form
/// that this interpreter cannot evaluate; a caller seeing this must record a language gap, not
/// a conformance failure, and a single exit code for both would make the two indistinguishable
/// from a script.
#[test]
fn a_construct_outside_the_subset_exits_two_and_names_itself() {
    let src = Temp::new("src.niles", TRANSFER);
    let args = args_file("acct a.usd\nmoney 4317 usd 2\n");
    let out = nilesc(&["run", src.path(), "authorises", "--args", args.path()]);
    assert_eq!(out.code, 2, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("NotInSubset: hold"),
        "the construct must be named: {}",
        out.stderr
    );
}

/// A misspelled function exits 2 and says so, rather than succeeding with an empty set.
#[test]
fn an_unknown_function_is_refused_rather_than_answered_with_nothing() {
    let src = Temp::new("src.niles", TRANSFER);
    let out = nilesc(&["run", src.path(), "transfr"]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("declares no function `transfr`"));
}

/// The ledger fixture is read, and a malformed one is a refusal rather than a silent skip.
#[test]
fn a_ledger_fixture_is_read_and_a_bad_one_is_named() {
    let src = Temp::new("src.niles", TRANSFER);
    let args = args_file("acct a.usd\nacct b.usd\nmoney 25000 usd 2\n");

    let good = Temp::new(
        "ledger",
        "# a world\naccount a.usd USD 2\nopen a.usd USD 100000\n",
    );
    let out = nilesc(&[
        "run",
        src.path(),
        "transfer",
        "--ledger",
        good.path(),
        "--args",
        args.path(),
    ]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let bad = Temp::new("ledger-bad", "account a.usd USD two\n");
    let out = nilesc(&[
        "run",
        src.path(),
        "transfer",
        "--ledger",
        bad.path(),
        "--args",
        args.path(),
    ]);
    assert_eq!(out.code, 2, "a bad fixture must not be skipped");
    assert!(
        out.stderr.contains("line 1"),
        "and must name the line: {}",
        out.stderr
    );

    let out = nilesc(&[
        "run",
        src.path(),
        "transfer",
        "--ledger",
        "/nonexistent/ledger",
        "--args",
        args.path(),
    ]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("cannot read"));
}

/// A file that does not typecheck is not run.
///
/// Running an ill-typed program would make `run` a second, unchecked path into the language,
/// which is the seam §6.9 argues against in the wire protocols and is no more acceptable here.
#[test]
fn a_file_that_does_not_check_is_not_run() {
    let src = Temp::new("bad.niles", "fn f() -> i64 { undefined_name() }\n");
    let out = nilesc(&["run", src.path(), "f"]);
    assert_eq!(out.code, 1, "stdout: {}", out.stdout);
    assert!(out.stdout.trim().is_empty());
}

fn hex_bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}

/// A program whose functions exercise nesting, replay and account identity.
const IDENTITY: &str = r#"
schema bank {
    currency usd { scale: 2 }
    ledger postings {
        txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,
        idem: IdemKey window 30.days,
        conserve per (txn, cur);
        retain forever;
    }
}

fn once(from: Id<Account>, to: Id<Account>, amount: Money<usd>)
    -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("once", window: 30.days) {
        post(debit(from, amount)?, credit(to, amount))
    }
}

// The same key, twice. Under T-Idem this is one transaction.
fn replay(from: Id<Account>, to: Id<Account>, amount: Money<usd>)
    -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    once(from, to, amount)?;
    once(from, to, amount)
}

// Two different keys. Two transactions, and the control for the test above.
fn twice(from: Id<Account>, to: Id<Account>, amount: Money<usd>)
    -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("first", window: 30.days) { post(debit(from, amount)?, credit(to, amount)) };
    txn idem("second", window: 30.days) { post(debit(from, amount)?, credit(to, amount)) }
}

fn by_int(amount: Money<usd>) -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("ident", window: 30.days) {
        post(debit(acct(1), amount)?, credit(acct(2), amount))
    }
}

fn by_str(amount: Money<usd>) -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("ident", window: 30.days) {
        post(debit(acct("1"), amount)?, credit(acct("2"), amount))
    }
}

fn nested(from: Id<Account>, to: Id<Account>, amount: Money<usd>)
    -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
{
    txn idem("outer", window: 30.days) {
        post(debit(from, amount)?, credit(to, amount));
        txn idem("inner", window: 30.days) {
            post(debit(from, amount)?, credit(to, amount))
        }
    }
}
"#;

/// **A replay under one idempotency key posts one set.**
///
/// `idem(key)` was parsed, carried, and used as the transaction's *name* — and never
/// compared against anything. A function calling a transfer twice under one key posted
/// twice and moved the balance twice, which is the failure an idempotency key exists to
/// prevent and the one T-Idem says the calculus rules out. The window is still evaluated
/// past (this interpreter has no clock, and `nilesc run` reports the count), but the
/// identity half is now honoured for the length of a run.
#[test]
fn a_replay_under_one_idempotency_key_posts_one_set() {
    let src = Temp::new("identity.niles", IDENTITY);
    let args = args_file("acct a.usd\nacct b.usd\nmoney 25000 usd 2\n");
    let one = nilesc(&["run", src.path(), "once", "--args", args.path()]);
    let replayed = nilesc(&["run", src.path(), "replay", "--args", args.path()]);
    assert_eq!(one.code, 0, "stderr: {}", one.stderr);
    assert_eq!(
        replayed.code, 0,
        "a replay must seal one set, and `run` refuses a function that sealed two: {}",
        replayed.stderr
    );
    assert_eq!(
        one.stdout, replayed.stdout,
        "calling the same keyed transaction twice must post what calling it once posts"
    );

    // **The control, and it is the stronger half.** Two *different* keys are two
    // transactions — and `run` refuses a function that sealed two sets, by design, because
    // it compares one. So this exits 2 with the count in the message, which is exactly what
    // `replay` did before the key was compared against anything. Memoizing every `txn`
    // unconditionally would turn this green and is what the assertion rules out.
    let two = nilesc(&["run", src.path(), "twice", "--args", args.path()]);
    assert_eq!(two.code, 2, "stdout: {}", two.stdout);
    assert!(
        two.stderr.contains("sealed 2 posting sets"),
        "two distinct keys must still be two transactions: {}",
        two.stderr
    );
}

/// **A `txn` inside a `txn` is refused and named.**
///
/// `Ledger::begin` replaces the open set rather than extending it, so the inner block
/// discarded every leg the outer one had accumulated and then sealed only its own: a set
/// that balances by itself while the outer half of the transaction is silently gone. The
/// calculus of §4.5 gives `txn` no nesting rule, so this exits 2 as a language gap rather
/// than being given a meaning here — the same treatment `hold` gets.
#[test]
fn a_nested_transaction_is_refused_rather_than_flattened() {
    let src = Temp::new("identity.niles", IDENTITY);
    let args = args_file("acct a.usd\nacct b.usd\nmoney 25000 usd 2\n");
    let out = nilesc(&["run", src.path(), "nested", "--args", args.path()]);
    assert_eq!(out.code, 2, "stdout: {} stderr: {}", out.stdout, out.stderr);
    assert!(
        out.stderr.contains("NotInSubset: nested txn"),
        "the construct must be named: {}",
        out.stderr
    );
    assert!(out.stdout.trim().is_empty(), "and print no set");
}

/// **`acct(1)` and `acct("1")` are the same account, deliberately.**
///
/// This is not a defect and the test exists to say so. The interpreter's account name is
/// the *rendering* of whatever was passed, which is what lets a conformance fixture pass
/// the concrete names a Rust product posts to (`loan.fac-1.agent`) where the schema says
/// `lender_a`. Making the integer form distinct — or refusing it — would break every
/// fixture in the GBS conformance suite for no gain in safety: `Id<Account>` is opaque, so
/// there is no type-level difference for the aliasing to violate.
///
/// What the test pins is that the aliasing is *total*: both spellings produce identical
/// bytes, so no fixture can depend on the difference.
#[test]
fn an_account_is_named_by_the_rendering_of_what_was_passed() {
    let src = Temp::new("identity.niles", IDENTITY);
    let args = args_file("money 25000 usd 2\n");
    let a = nilesc(&["run", src.path(), "by_int", "--args", args.path()]);
    let b = nilesc(&["run", src.path(), "by_str", "--args", args.path()]);
    assert_eq!(
        (a.code, b.code),
        (0, 0),
        "stderr: {} {}",
        a.stderr,
        b.stderr
    );
    assert_eq!(
        a.stdout, b.stdout,
        "an account identifier is its rendering; the two spellings are one account"
    );
}
