//! **`?` is applied only to what can be a `Result` or an `Option`** (cycle 15, C15-01;
//! E30 finding F12, author decision 7 of 2026-09-30).
//!
//! Until this card the typechecker's `Expr::Try` arm returned its operand's shape and never
//! looked at it, so `credit(b, m)?` — `credit` returns the leg itself, not a `Result` —
//! type-checked, and the interpreter raised `TypeMismatch { want: "Result or Option" }` at run
//! time. E30 found it when a sign-flip mutant turned `debit(..)?` into `credit(..)?`.
//!
//! `refused` fails against `f59cb16`, where no NL0257 exists; `accepted` is its control,
//! so the fix cannot be "refuse every `?`". GBS's `transfer(..)?`, whose operand the
//! checker's call rule shapes as `Money` because an *argument* is money, is in `accepted`:
//! the first version of this check refused it.

use niles_lang::{parser, resolve, typecheck};

const SCHEMA: &str = include_str!("mutants/schema.niles");

fn codes(body: &str) -> (Vec<String>, String) {
    let src = format!("{SCHEMA}\n{body}\n");
    let (prog, mut d) = parser::parse_program(&src);
    let (cat, rd) = resolve::resolve_program(&prog, 0);
    d.extend(rd);
    let (_r, td) = typecheck::check_program(&prog, &cat);
    d.extend(td);
    (
        d.items.iter().map(|x| x.code.to_string()).collect(),
        d.render(&src, "try.niles"),
    )
}

const SIG: &str = "(from: Id<Account>, to: Id<Account>, amount: Money<usd>)
    -> Result<TxnId, TxnError> ! { append, debit<usd>, credit<usd> }";

#[test]
fn refused() {
    let cases = [
        (
            "a credit half",
            format!("fn f{SIG} {{ txn idem(\"f\") {{ post(debit(from, amount)?, credit(to, amount)?) }} }}"),
        ),
        (
            "a money literal",
            format!("fn f{SIG} {{ let m = 10.00 usd?; txn idem(\"f\") {{ post(debit(from, m)?, credit(to, m)) }} }}"),
        ),
        (
            "an integer literal",
            format!("fn f{SIG} {{ let n = 4?; txn idem(\"f\") {{ post(debit(from, amount)?, credit(to, amount)) }} }}"),
        ),
        (
            "a tuple",
            format!("fn f{SIG} {{ let p = (1, 2)?; txn idem(\"f\") {{ post(debit(from, amount)?, credit(to, amount)) }} }}"),
        ),
        (
            "money arithmetic",
            format!("fn f{SIG} {{ let m = (amount + amount)?; txn idem(\"f\") {{ post(debit(from, amount)?, credit(to, amount)) }} }}"),
        ),
        (
            "a call declared to return money",
            format!("fn fee() -> Money<usd> ! {{ }} {{ 1.50 usd }}\nfn f{SIG} {{ let m = fee()?; txn idem(\"f\") {{ post(debit(from, m)?, credit(to, m)) }} }}"),
        ),
        (
            "a call declared to return an integer",
            format!("fn n() -> i64 ! {{ }} {{ 4 }}\nfn f{SIG} {{ let k = n()?; txn idem(\"f\") {{ post(debit(from, amount)?, credit(to, amount)) }} }}"),
        ),
    ];
    for (what, src) in cases {
        let (c, rendered) = codes(&src);
        assert!(
            c.iter().any(|x| x == "NL0257"),
            "`?` on {what} must be refused with NL0257; got {c:?}\n{rendered}"
        );
        assert!(
            !c.iter().any(|x| x.starts_with("NL00")),
            "{what}: the case does not parse, so it tests nothing: {c:?}\n{rendered}"
        );
    }
}

#[test]
fn accepted() {
    let cases = [
        (
            "a debit half",
            format!("fn f{SIG} {{ txn idem(\"f\") {{ post(debit(from, amount)?, credit(to, amount)) }} }}"),
        ),
        (
            "a call declared to return a Result, with a money argument",
            format!(
                "fn t{SIG} {{ txn idem(\"t\") {{ post(debit(from, amount)?, credit(to, amount)) }} }}\n\
                 fn g(a: Id<Account>, b: Id<Account>) -> Result<TxnId, TxnError> ! {{ append, debit<usd>, credit<usd> }} {{ let _x = t(a, b, 5.00 usd)?; t(a, b, 5.00 usd) }}"
            ),
        ),
        (
            "a variable bound to such a call",
            format!(
                "fn t{SIG} {{ txn idem(\"t\") {{ post(debit(from, amount)?, credit(to, amount)) }} }}\n\
                 fn g(a: Id<Account>, b: Id<Account>) -> Result<TxnId, TxnError> ! {{ append, debit<usd>, credit<usd> }} {{ let r = t(a, b, 5.00 usd); let _x = r?; t(a, b, 5.00 usd) }}"
            ),
        ),
    ];
    for (what, src) in cases {
        let (c, rendered) = codes(&src);
        assert!(
            !c.iter().any(|x| x == "NL0257"),
            "`?` on {what} is well typed and must be accepted; got {c:?}\n{rendered}"
        );
        assert!(
            !c.iter().any(|x| x.starts_with("NL00")),
            "{what}: the case does not parse: {c:?}\n{rendered}"
        );
    }
}
