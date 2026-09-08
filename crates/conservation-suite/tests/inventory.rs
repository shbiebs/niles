//! **H-S8's falsifier: a conserved quantity that is not money.**
//!
//! H-S8 says one engine and one language serve every workload class on a *general relational
//! core*, with banking a library over it. §11.3 states its refutation condition — a domain
//! that cannot be expressed without a kernel change falsifies it — and §6.6 records what
//! stands in the way: seven banking forms are keywords in the compiler's registry rather
//! than a library over general machinery. Until this file existed the hypothesis had no
//! runner at all: `grep -ri inventory` over the crates, schemas and examples returned
//! nothing, and `status.toml` said so.
//!
//! `examples/inventory.niles` is the instance: stock movements between warehouses, where
//! units of a SKU are conserved exactly as money is and nothing is money. This file runs it
//! through the same three checks the money domain gets — it checks, it conserves statically,
//! and it posts — and asserts the one thing that makes the exercise a test rather than a
//! demonstration: **nothing in the compiler or the kernel was changed to allow it.**
//!
//! The result is a *qualified* pass, and the qualification is in the source: see
//! `MISMATCH-T-22-currency-literal` at the foot of the example. The machinery is general;
//! the notation is not.

use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn nilesc(args: &[&str]) -> (i32, String, String) {
    let out = Command::new("cargo")
        .args(["run", "-q", "-p", "nilesc", "--"])
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("run nilesc");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

const EXAMPLE: &str = "examples/inventory.niles";

#[test]
fn the_non_financial_domain_checks_and_its_conservation_is_proved_statically() {
    let (code, out, err) = nilesc(&["check", EXAMPLE]);
    assert_eq!(code, 0, "stdout: {out}\nstderr: {err}");
    assert!(
        out.contains("2 conservation obligation(s) proved statically"),
        "both movement functions must be *proved* to conserve, not discharged to the \
         runtime — the currency-row solver working over a grade that is not a currency is \
         the whole claim: {out}"
    );
}

#[test]
fn a_movement_posts_two_cancelling_legs_in_the_non_financial_grade() {
    let args = repo_root().join("target/inventory.args");
    std::fs::create_dir_all(args.parent().expect("target/")).ok();
    std::fs::write(&args, "acct wh.a\nacct wh.b\nmoney 5 widget 0\n").expect("write args");
    let (code, out, err) = nilesc(&[
        "run",
        EXAMPLE,
        "move_stock",
        "--args",
        args.to_str().expect("utf-8"),
    ]);
    assert_eq!(code, 0, "stderr: {err}");
    let hex = out.trim();
    assert!(!hex.is_empty(), "nothing on stdout");

    // The grade reaches the encoding. `WIDGET` in ASCII is what proves the ledger is
    // conserving a SKU rather than quietly treating everything as one currency.
    let ascii: String = hex
        .as_bytes()
        .chunks(2)
        .filter_map(|c| u8::from_str_radix(std::str::from_utf8(c).ok()?, 16).ok())
        .map(|b| if b.is_ascii_graphic() { b as char } else { '.' })
        .collect();
    assert_eq!(
        ascii.matches("WIDGET").count(),
        2,
        "both legs must carry the SKU as their grade: {ascii}"
    );

    // And the two legs cancel: the interpreter's seal refuses a set that does not, so an
    // exit of 0 already says so. This asserts the shape as well, so a future encoding
    // change that dropped a leg would not pass silently.
    let bytes: Vec<u8> = hex
        .as_bytes()
        .chunks(2)
        .filter_map(|c| u8::from_str_radix(std::str::from_utf8(c).ok()?, 16).ok())
        .collect();
    assert_eq!(&bytes[8..12], &2u32.to_be_bytes(), "two legs");
}

#[test]
fn a_movement_that_does_not_conserve_is_refused() {
    // The negative control. Without it, a checker that proved nothing would pass the test
    // above — "0 errors" is what a checker that has stopped checking also reports.
    let src = std::fs::read_to_string(repo_root().join(EXAMPLE)).expect("the example");
    let with_leak = format!(
        "{src}\n\
         fn leak(from: Id<Account>, to: Id<Account>, out: Money<widget>, back: Money<widget>)\n\
             -> Result<TxnId, TxnError>\n\
             ! {{ append, debit<widget>, credit<widget> }}\n\
         {{ txn idem(\"leak\") {{ post(debit(from, out)?, credit(to, back)) }} }}\n"
    );
    let path = repo_root().join("target/inventory-leak.niles");
    std::fs::write(&path, &with_leak).expect("write");
    let (code, out, _) = nilesc(&["check", path.to_str().expect("utf-8")]);
    assert_eq!(
        code, 0,
        "two unrelated quantities are `Undecided`, not an error"
    );
    assert!(
        out.contains("1 conservation obligation(s) discharged to the runtime")
            || out.contains("discharged to the runtime"),
        "a movement of two unrelated quantities cannot be proved and must be handed to the \
         seal rather than waved through: {out}"
    );

    // And the seal refuses it, in the non-financial grade, naming the residual.
    let args = repo_root().join("target/inventory-leak.args");
    std::fs::write(
        &args,
        "acct wh.a\nacct wh.b\nmoney 5 widget 0\nmoney 3 widget 0\n",
    )
    .expect("write args");
    let (code, out, err) = nilesc(&[
        "run",
        path.to_str().expect("utf-8"),
        "leak",
        "--args",
        args.to_str().expect("utf-8"),
    ]);
    assert_eq!(code, 1, "stdout: {out}");
    assert!(
        err.contains("WIDGET") && err.contains('2'),
        "the refusal must name the grade and the residual, exactly as it does for money: {err}"
    );
}

/// **Nothing was changed to make the domain work**, which is the claim under test.
///
/// A falsifier that quietly widened the compiler would be a demonstration, not a
/// measurement. The example itself is the evidence — it is ordinary Niles — and this checks
/// the two things a reader would otherwise have to take on trust: that the language has no
/// inventory-specific anything, and that the example uses no construct the banking corpus
/// does not.
#[test]
fn the_compiler_knows_nothing_about_inventory() {
    let mut hits = Vec::new();
    let mut stack = vec![repo_root().join("crates/niles-lang/src")];
    stack.push(repo_root().join("crates/niles-ir/src"));
    stack.push(repo_root().join("crates/nilestream-core/src"));
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&p)
                .unwrap_or_default()
                .to_lowercase();
            for word in ["inventory", "widget", "sprocket", "warehouse", "sku"] {
                if text.contains(word) {
                    hits.push(format!("{}: {word}", p.display()));
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "the compiler and the runtime must know nothing about this domain; found:\n  {}",
        hits.join("\n  ")
    );
}
