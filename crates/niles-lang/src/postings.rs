//! **The posting shape of a function**: which legs it emits, against which accounts, in which
//! direction, for how much.
//!
//! # What this is for
//!
//! GBS has two implementations of every product. `gbs-products/src/lending.rs::drawdown`
//! builds a posting set in Rust; `gbs/niles/gbs.niles::syndicated_drawdown` declares the same
//! transaction in Niles, where the compiler proves its conservation statically. Nothing
//! checked that the two agree.
//!
//! That is a seam, and thesis §6.9 argues against exactly this shape — *three surfaces, one
//! intermediate representation*. A conformance gap here means the schema's "five conservation
//! obligations proved statically" is a proof about a program that is not the one running.
//!
//! # Why a *shape* rather than an evaluation
//!
//! The obvious design is to run the Niles function and compare its output. `niles-interp`
//! exists, but it deliberately refuses the relational tier — `post`, `debit`, `credit`, `txn`
//! are `Error::NotInSubset` — because a stage-0 interpreter that quietly grew a banking
//! runtime would be a second implementation of the thing the compiler is meant to check.
//! Building one to serve a conformance test would create the seam the test exists to detect.
//!
//! So this extracts what the function *declares*: the sequence of legs, each with a direction,
//! an account expression and an amount expression. That is enough to catch the failures a
//! conformance test is for — a leg missing, a sign flipped, an amount that differs, an account
//! bound to the wrong parameter — and it is honest about what it is not.
//!
//! # What it does not check
//!
//! Control flow. A function whose legs sit inside an `if` contributes both branches' legs to
//! the shape, so two implementations that agree on every leg and disagree on *when* they are
//! emitted will pass. That limitation is stated in the extracted output itself
//! ([`Shape::branched`]), so a fixture cannot silently rely on it.

use crate::ast::{Block, Expr, Item, Program, Stmt};

/// Which way a leg moves value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Debit,
    Credit,
}

impl Direction {
    pub fn name(self) -> &'static str {
        match self {
            Direction::Debit => "debit",
            Direction::Credit => "credit",
        }
    }
}

/// One leg, as the source declares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    pub direction: Direction,
    /// The account expression, rendered. A parameter name, or a path.
    pub account: String,
    /// The amount, rendered: `100.00 usd` for a literal, or a parameter name.
    pub amount: String,
}

/// The legs a function declares, in source order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shape {
    pub function: String,
    pub legs: Vec<Leg>,
    /// Whether any leg sits inside a conditional.
    ///
    /// Reported rather than hidden: a shape extracted from a branched body is the union of the
    /// branches, so two implementations that agree on every leg and disagree on *when* each
    /// fires would compare equal. A fixture that relies on that should have to see the word.
    pub branched: bool,
    /// Whether any leg sits inside an `fx` form, whose legs conserve independently.
    pub multi_leg_fx: bool,
}

impl Shape {
    /// The canonical rendering, one leg per line.
    ///
    /// Text rather than a struct because the consumer is another repository: GBS compares this
    /// against a fixture its own Rust products produced, and a byte comparison of a documented
    /// format is a comparison two codebases can make without sharing a type.
    pub fn render(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("function {}\n", self.function));
        if self.branched {
            s.push_str("branched true\n");
        }
        if self.multi_leg_fx {
            s.push_str("fx true\n");
        }
        for l in &self.legs {
            s.push_str(&format!(
                "{} {} {}\n",
                l.direction.name(),
                l.account,
                l.amount
            ));
        }
        s
    }
}

/// Extract the posting shape of one function.
///
/// `None` if the program declares no such function — distinguished from a function with no
/// legs, which yields an empty `Shape`. A conformance test that could not tell those apart
/// would pass against a typo in the function name.
pub fn shape_of(prog: &Program, function: &str) -> Option<Shape> {
    // Functions live at the top level. A `schema` block holds relations, views and indices —
    // not functions — so there is one place to look, and looking in two would have been the
    // extractor guessing at a grammar it can read.
    for item in &prog.items {
        if let Item::Fn(f) = item {
            if f.name.text == function {
                let mut shape = Shape {
                    function: function.to_string(),
                    ..Shape::default()
                };
                // A function with no body is a declaration: it contributes no legs, which is
                // different from not existing, and the caller can tell.
                if let Some(body) = &f.body {
                    walk_block(body, &mut shape, false);
                }
                return Some(shape);
            }
        }
    }
    None
}

/// Every function that declares at least one leg, in source order.
pub fn all_shapes(prog: &Program) -> Vec<Shape> {
    prog.items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(f) => Some(f.name.text.clone()),
            _ => None,
        })
        .filter_map(|n| shape_of(prog, &n))
        .filter(|s| !s.legs.is_empty())
        .collect()
}

fn walk_block(b: &Block, shape: &mut Shape, in_branch: bool) {
    for st in &b.stmts {
        match st {
            Stmt::Let { init: Some(e), .. } => walk_expr(e, shape, in_branch),
            Stmt::Expr(e) | Stmt::Semi(e) => walk_expr(e, shape, in_branch),
            _ => {}
        }
    }
    if let Some(tail) = &b.tail {
        walk_expr(tail, shape, in_branch);
    }
}

fn walk_expr(e: &Expr, shape: &mut Shape, in_branch: bool) {
    match e {
        Expr::Call { callee, args, .. } => {
            let name = callee_name(callee);
            match name.as_deref() {
                Some("debit") | Some("credit") => {
                    let direction = if name.as_deref() == Some("debit") {
                        Direction::Debit
                    } else {
                        Direction::Credit
                    };
                    if in_branch {
                        shape.branched = true;
                    }
                    // `debit(account, amount)` — the first two positional arguments. A call
                    // with fewer is a program that does not compile, and this runs after the
                    // front end, so it cannot be reached with one.
                    let account = args.first().map(|a| render(&a.value)).unwrap_or_default();
                    let amount = args.get(1).map(|a| render(&a.value)).unwrap_or_default();
                    shape.legs.push(Leg {
                        direction,
                        account,
                        amount,
                    });
                }
                _ => {}
            }
            for a in args {
                walk_expr(&a.value, shape, in_branch);
            }
        }
        Expr::Try { expr, .. } => walk_expr(expr, shape, in_branch),
        Expr::Block(b) => walk_block(b, shape, in_branch),
        Expr::If {
            cond, then, els, ..
        } => {
            walk_expr(cond, shape, in_branch);
            // Everything below an `if` is branched, whatever the enclosing context was.
            walk_block(then, shape, true);
            if let Some(e) = els {
                walk_expr(e, shape, true);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            walk_expr(lhs, shape, in_branch);
            walk_expr(rhs, shape, in_branch);
        }
        Expr::Unary { operand, .. } => walk_expr(operand, shape, in_branch),
        Expr::Assign { target, value, .. } => {
            walk_expr(target, shape, in_branch);
            walk_expr(value, shape, in_branch);
        }
        Expr::Tuple { elems, .. } | Expr::Array { elems, .. } => {
            for el in elems {
                walk_expr(el, shape, in_branch);
            }
        }
        Expr::Field { base, .. } => walk_expr(base, shape, in_branch),
        Expr::Index { base, index, .. } => {
            walk_expr(base, shape, in_branch);
            walk_expr(index, shape, in_branch);
        }
        Expr::Cast { expr, .. } => walk_expr(expr, shape, in_branch),
        Expr::Stage { recv, args, .. } => {
            walk_expr(recv, shape, in_branch);
            for a in args {
                walk_expr(&a.value, shape, in_branch);
            }
        }
        Expr::Closure { body, .. } => walk_expr(body, shape, in_branch),

        // ── the novel forms ─────────────────────────────────────────────────────────
        //
        // Matched by name rather than reached through a generic walk. An extractor that
        // guessed at these would be an extractor that silently found no legs the day one of
        // them changed shape — and "no legs" is the answer that makes a conformance test pass.
        Expr::Txn { body, .. } => walk_block(body, shape, in_branch),
        Expr::Fx { legs, rate, .. } => {
            // Each leg conserves independently, which is what makes cross-currency movement
            // atomic without a currency both sides share. The flag is recorded so a fixture
            // reader knows the legs below belong to more than one conservation obligation.
            shape.multi_leg_fx = true;
            for (_, e) in legs {
                walk_expr(e, shape, in_branch);
            }
            if let Some(r) = rate {
                walk_expr(r, shape, in_branch);
            }
        }
        Expr::Resolve { hold, .. } => walk_expr(hold, shape, in_branch),
        Expr::Hold { args, .. } | Expr::Authorize { args, .. } | Expr::Declassify { args, .. } => {
            for a in args {
                walk_expr(&a.value, shape, in_branch);
            }
        }

        // ── control flow ────────────────────────────────────────────────────────────
        Expr::Match {
            scrutinee, arms, ..
        } => {
            walk_expr(scrutinee, shape, in_branch);
            for arm in arms {
                walk_expr(&arm.body, shape, true);
            }
        }
        Expr::Case { arms, els, .. } => {
            for (c, v) in arms {
                walk_expr(c, shape, in_branch);
                walk_expr(v, shape, true);
            }
            if let Some(e) = els {
                walk_expr(e, shape, true);
            }
        }
        Expr::While { cond, body, .. } => {
            walk_expr(cond, shape, in_branch);
            // A leg inside a loop fires an unknown number of times, which is a stronger
            // caveat than `branched` — but it is the same *kind* of caveat, so it is recorded
            // the same way rather than passing silently.
            walk_block(body, shape, true);
        }
        Expr::Loop { body, .. } => walk_block(body, shape, true),
        Expr::For { iter, body, .. } => {
            walk_expr(iter, shape, in_branch);
            walk_block(body, shape, true);
        }
        Expr::Return { value: Some(v), .. } => walk_expr(v, shape, in_branch),
        Expr::Sql { inner, .. } => walk_expr(inner, shape, in_branch),
        Expr::Explain { target, .. }
        | Expr::Reproduce { target, .. }
        | Expr::Impact { target, .. } => walk_expr(target, shape, in_branch),

        // Leaves and the forms that cannot contain a posting.
        _ => {}
    }
}

fn callee_name(e: &Expr) -> Option<String> {
    match e {
        Expr::Path(p) => p.segments.last().map(|s| s.text.clone()),
        _ => None,
    }
}

/// Render an expression the way a fixture spells it.
///
/// Deliberately narrow: a parameter is its name, a money literal is `100.00 usd`, an integer
/// is its digits. Anything else renders as `<expr>` — which makes a fixture that depended on
/// it obviously unusable rather than subtly wrong.
fn render(e: &Expr) -> String {
    match e {
        Expr::Path(p) => p
            .segments
            .iter()
            .map(|s| s.text.clone())
            .collect::<Vec<_>>()
            .join("::"),
        Expr::Money {
            minor,
            scale,
            currency,
            ..
        } => {
            let d = 10i128.pow(*scale);
            let sign = if *minor < 0 { "-" } else { "" };
            let a = minor.abs();
            if *scale == 0 {
                format!("{sign}{a} {}", currency.text)
            } else {
                format!(
                    "{sign}{}.{:0width$} {}",
                    a / d,
                    a % d,
                    currency.text,
                    width = *scale as usize
                )
            }
        }
        Expr::Int(v, _) => v.to_string(),
        Expr::Str(s, _) => format!("{s:?}"),
        Expr::Field { base, name, .. } => format!("{}.{}", render(base), name.text),
        _ => "<expr>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn parse(src: &str) -> Program {
        let (p, d) = parser::parse_program(src);
        assert!(!d.has_errors(), "{}", d.render(src, "<test>"));
        p
    }

    const TRANSFER: &str = r#"
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
"#;

    #[test]
    fn a_two_leg_transfer_yields_two_legs_in_source_order() {
        let s = shape_of(&parse(TRANSFER), "transfer").expect("found");
        assert_eq!(s.legs.len(), 2);
        assert_eq!(
            s.legs[0],
            Leg {
                direction: Direction::Debit,
                account: "from".into(),
                amount: "amount".into(),
            }
        );
        assert_eq!(s.legs[1].direction, Direction::Credit);
        assert_eq!(s.legs[1].account, "to");
        assert!(!s.branched);
    }

    #[test]
    fn a_missing_function_is_none_and_a_legless_one_is_empty() {
        // A conformance test that could not tell these apart would pass against a typo in the
        // function name, which is the one failure that makes the whole apparatus decorative.
        let prog = parse(TRANSFER);
        assert!(shape_of(&prog, "no_such_function").is_none());

        let legless = parse("fn nothing() -> Int { 1 }\n");
        let s = shape_of(&legless, "nothing").expect("found");
        assert!(s.legs.is_empty());
    }

    #[test]
    fn money_literals_render_at_their_own_scale() {
        let src = r#"
fn fees(a: Id<Account>, b: Id<Account>, c: Id<Account>) -> Result<TxnId, TxnError> ! { append } {
    txn idem("f") {
        post(
            debit(a, 100.00 usd)?,
            credit(b, 99.75 usd),
            credit(c, 0.25 usd),
        )
    }
}
"#;
        let s = shape_of(&parse(src), "fees").expect("found");
        let rendered: Vec<String> = s.legs.iter().map(|l| l.amount.clone()).collect();
        assert_eq!(rendered, vec!["100.00 usd", "99.75 usd", "0.25 usd"]);

        // And the whole shape renders to something another repository can compare.
        let text = s.render();
        assert!(text.starts_with("function fees\n"), "{text}");
        assert!(text.contains("debit a 100.00 usd\n"), "{text}");
        assert!(text.contains("credit c 0.25 usd\n"), "{text}");
    }

    #[test]
    fn a_scale_zero_currency_renders_without_a_decimal_point() {
        // JPY at scale 0. A renderer that assumed two decimals would spell 15000 as 150.00,
        // and a fixture built from it would be wrong by a hundred while looking plausible.
        let src = r#"
fn jpy(a: Id<Account>, b: Id<Account>) -> Result<TxnId, TxnError> ! { append } {
    txn idem("j") { post(debit(a, 15000 jpy)?, credit(b, 15000 jpy)) }
}
"#;
        let s = shape_of(&parse(src), "jpy").expect("found");
        assert_eq!(s.legs[0].amount, "15000 jpy");
    }

    #[test]
    fn legs_inside_a_conditional_are_marked_branched() {
        // The limitation, surfaced. A shape extracted from a branched body is the union of the
        // branches, so two implementations agreeing on every leg and disagreeing on *when*
        // each fires compare equal. A fixture relying on that has to see the word.
        let src = r#"
fn guarded(a: Id<Account>, b: Id<Account>, amount: Money<usd>, ok: Bool)
    -> Result<TxnId, TxnError> ! { append }
{
    txn idem("g") {
        if ok {
            post(debit(a, amount)?, credit(b, amount))
        } else {
            abort(TxnError::Insufficient)
        }
    }
}
"#;
        let s = shape_of(&parse(src), "guarded").expect("found");
        assert!(s.branched, "the legs are inside an `if`");
        assert!(s.render().contains("branched true"), "{}", s.render());
        assert_eq!(s.legs.len(), 2);
    }

    #[test]
    fn a_many_legged_allocation_keeps_every_leg_and_its_order() {
        let src = r#"
fn draw(b: Id<Account>, l1: Id<Account>, l2: Id<Account>, l3: Id<Account>)
    -> Result<TxnId, TxnError> ! { append }
{
    txn idem("d") {
        post(
            debit(l1, 100.00 usd)?,
            debit(l2, 100.00 usd)?,
            debit(l3, 100.00 usd)?,
            credit(b, 300.00 usd),
        )
    }
}
"#;
        let s = shape_of(&parse(src), "draw").expect("found");
        assert_eq!(s.legs.len(), 4);
        assert_eq!(
            s.legs
                .iter()
                .filter(|l| l.direction == Direction::Debit)
                .count(),
            3
        );
        assert_eq!(s.legs[3].account, "b");
        assert_eq!(s.legs[3].amount, "300.00 usd");

        // Order is part of the shape: a set that reordered would compare equal to one that
        // did not, and entry order is part of a posting set's identity.
        let accounts: Vec<&str> = s.legs.iter().map(|l| l.account.as_str()).collect();
        assert_eq!(accounts, vec!["l1", "l2", "l3", "b"]);
    }

    #[test]
    fn every_function_with_legs_is_found_by_the_bulk_extractor() {
        let src = format!("{TRANSFER}\nfn nothing() -> Int {{ 1 }}\n");
        let shapes = all_shapes(&parse(&src));
        assert_eq!(
            shapes.len(),
            1,
            "the legless function is not a posting shape"
        );
        assert_eq!(shapes[0].function, "transfer");
    }
}
