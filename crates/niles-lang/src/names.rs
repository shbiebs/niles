//! **Name resolution for the imperative sublanguage: does every name denote something?**
//!
//! This is the check the compiler did not have. `resolve.rs` resolves *declarations* —
//! relations, views, columns, currencies, contracts — and `typecheck.rs` judges the shapes
//! of expressions it can name. Between the two, nothing ever asked whether an identifier in
//! a function body referred to anything at all. `typecheck`'s `Expr::Path` arm reads
//!
//! ```text
//! sc.bindings.get(name).cloned().unwrap_or(Shape::Opaque)
//! ```
//!
//! and `Shape::Opaque` is the checker's honest "I cannot see inside this" — so a name that
//! was never bound was indistinguishable from a value of a user type. `nilesc check` on a
//! program whose body is a single undefined identifier printed `ok` and exited 0. So did a
//! call to a function that does not exist.
//!
//! # Why this is not a nicety
//!
//! Every soundness statement in this thesis is of the form *a well-typed program cannot
//! ...* — cannot create money, cannot mismatch currencies, cannot overdraw without
//! authority (Theorem 4.4). A theorem of that shape quantifies over the programs the
//! checker accepts. A checker that accepts programs containing names that denote nothing
//! is quantifying over a set strictly larger than the calculus describes, and the
//! implementation half of the theorem is correspondingly weaker than its statement. That is
//! what `status.toml`'s H-S4 records, and this file is what narrows the set.
//!
//! # What is checked, and what is deliberately not
//!
//! Checked: a **single-segment** path in value position, and the callee of a call written
//! as a single-segment path. Two diagnostics, [`UNBOUND`] and [`UNKNOWN_FN`].
//!
//! Not checked, each for a reason rather than for want of time:
//!
//! * **Multi-segment paths** (`Outcome::Post`, `bank::transfer`). Resolving these needs the
//!   module and `use` graph, which this compiler does not build. Reporting on them from the
//!   last segment alone would be a guess.
//! * **Type positions.** `Ty::Path` names live in their own namespace with its own
//!   declarations and generics; a value-namespace walk has no business judging them.
//! * **SQL `select` and `exists` subtrees.** Their binders are table aliases and column
//!   names, a different discipline, already served by NL0400–NL0403. Walking into them with
//!   this scope stack would report every column as unbound.
//! * **Bare identifiers inside a pipeline stage's arguments.** In query context a bare name
//!   is a *column*, not a binding: `lower::collect_order_keys` reads `order_by(|r| asc(x))`
//!   with `x` as the column `x`. Column names are resolved against the relation by
//!   `resolve::check_view` — that is what NL0400, NL0403 and the `*_unknown_column` goldens
//!   are — so this pass would be a second, worse answer to a question already answered.
//!   Calls inside query context *are* still checked, against the operator set below plus
//!   the program's own functions, so a misspelt predicate helper is still refused.
//! * **Field and method names.** `r.amt` is a projection, not a name lookup, and a stage's
//!   method name is already judged by NL0401 and NL0505.
//!
//! Each of these is a *stated* limit: the corpus is checked against it in
//! `tests/name_resolution.rs`, and the three control programs of Appendix C are refused.
//!
//! # The walker cannot fall behind the AST
//!
//! [`Walk::expr`] matches every `Expr` variant explicitly and has **no wildcard arm**. A new
//! variant added to the AST fails to compile here, which is the only mechanism that keeps a
//! walk of this kind honest; `typecheck::each_child` ends in `_ => {}` and has silently
//! missed variants before.

use crate::ast::*;
use crate::diagnostics::{closest, Diagnostic, Diagnostics};
use crate::resolve::Catalog;
use std::collections::BTreeSet;

/// A name in value position that denotes nothing.
pub const UNBOUND: &str = "NL0204";
/// A call whose callee names no function this program or the host provides.
pub const UNKNOWN_FN: &str = "NL0205";

/// Free functions the interpreter provides, and the forms that look like calls but are
/// built into the language.
///
/// This list is the *host shim* of Appendix E.3 plus the ledger primitives, and it is
/// asserted against `niles_interp`'s dispatch by `tests/name_resolution.rs`, so the two
/// cannot drift: a builtin added to the interpreter and not here becomes a false NL0205,
/// and one removed there and left here becomes a call that resolves to nothing at run time.
pub const BUILTIN_FNS: &[&str] = &[
    // ledger primitives (`Interp::builtin_ledger`)
    "acct", "debit", "credit", "post", // free functions (`Interp::builtin_free`)
    "print", "len", "panic", "range", "chr", // constructors the interpreter builds directly
    "Some", "Ok", "Err",
];

/// Value-position names the language provides without a declaration.
const BUILTIN_VALUES: &[&str] = &["self", "Self", "None", "true", "false", "null"];

/// Names spelled like functions that are **query forms**, legal only inside a pipeline
/// stage's arguments.
///
/// These are not functions and there is nowhere to declare them: `lower.rs` matches on the
/// name and emits an operator. `asc`/`desc` are read by `collect_order_keys` and the five
/// aggregates by `aggregate_of`, which is why the list is exactly this and not longer — an
/// aggregate this list has and `aggregate_of` does not would lower to nothing with no
/// diagnostic, and `tests/name_resolution.rs` holds the two lists together.
///
/// They are accepted **only** in query context. `sum(x)` in a function body denotes nothing
/// there, and saying so is the point of this pass.
pub const QUERY_FNS: &[&str] = &["asc", "desc", "sum", "count", "min", "max", "avg"];

/// Check every name in the program's imperative code against what is in scope.
///
/// Called from [`crate::resolve::resolve_program`] once the catalog is complete, so that
/// every caller of the compiler gets it without having to know it exists. That placement is
/// deliberate: a check wired into one entry point and not the other twelve is a check the
/// project does not have, and this repository has found that shape three times.
pub fn check_program(prog: &Program, cat: &Catalog) -> Diagnostics {
    let mut w = Walk {
        d: Diagnostics::new(),
        globals: BTreeSet::new(),
        functions: BTreeSet::new(),
        scopes: Vec::new(),
        query_depth: 0,
    };
    w.seed(prog, cat);
    for item in &prog.items {
        w.item(item);
    }
    w.d
}

struct Walk {
    d: Diagnostics,
    /// Everything nameable in value position without a local binding.
    globals: BTreeSet<String>,
    /// Everything callable.
    functions: BTreeSet<String>,
    /// Innermost last. A name is in scope if any frame holds it.
    scopes: Vec<BTreeSet<String>>,
    /// How many pipeline-stage argument lists enclose the expression being walked.
    ///
    /// Non-zero means **query context**: bare names may be columns, and the query forms of
    /// [`QUERY_FNS`] are callable. Zero means the imperative sublanguage, where neither is
    /// true. A counter rather than a flag because a stage argument may contain a closure
    /// containing another stage.
    query_depth: usize,
}

impl Walk {
    /// Collect every name the program declares, before walking any body.
    ///
    /// Items are order-independent, as they are in Rust: a function may call one declared
    /// below it. Collecting first is what makes that true rather than accidental.
    fn seed(&mut self, prog: &Program, cat: &Catalog) {
        for n in BUILTIN_VALUES {
            self.globals.insert((*n).to_string());
        }
        for n in BUILTIN_FNS {
            self.functions.insert((*n).to_string());
            // A builtin is also a value: `let f = print;` is not offered today, but naming
            // one outside call position should not be reported as denoting nothing.
            self.globals.insert((*n).to_string());
        }
        // The catalog's own namespace: what a pipeline can start from, and what a money
        // literal names.
        for n in cat.relations.keys().chain(cat.views.keys()) {
            self.globals.insert(n.clone());
        }
        for n in cat.currencies.keys() {
            self.globals.insert(n.clone());
        }
        for n in cat.capabilities.keys() {
            self.globals.insert(n.clone());
        }
        for item in &prog.items {
            self.seed_item(item);
        }
    }

    fn seed_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => {
                self.functions.insert(f.name.text.clone());
                self.globals.insert(f.name.text.clone());
            }
            Item::Mod { items, .. } => items.iter().for_each(|i| self.seed_item(i)),
            // An `impl` block's methods are callable as free functions in this surface —
            // `typecheck::collect_fns` already treats them that way when building the call
            // graph, so the two views of "what is callable" agree.
            Item::Impl(i) => {
                for f in &i.items {
                    self.functions.insert(f.name.text.clone());
                    self.globals.insert(f.name.text.clone());
                }
            }
            Item::Trait(t) => {
                for f in &t.items {
                    self.functions.insert(f.name.text.clone());
                    self.globals.insert(f.name.text.clone());
                }
            }
            Item::Const { name, .. } => {
                self.globals.insert(name.text.clone());
            }
            Item::Capability { name, .. } => {
                self.globals.insert(name.text.clone());
            }
            // A struct or enum is a type, and its constructors are names in value position:
            // `Outcome::Post` is a path this file does not judge, but a tuple struct used
            // bare is, so the type's own name is a value.
            Item::Struct(s) => {
                self.globals.insert(s.name.text.clone());
                self.functions.insert(s.name.text.clone());
            }
            Item::Enum(e) => {
                self.globals.insert(e.name.text.clone());
                for (v, _) in &e.variants {
                    self.globals.insert(v.text.clone());
                    self.functions.insert(v.text.clone());
                }
            }
            Item::TypeAlias { name, .. } => {
                self.globals.insert(name.text.clone());
            }
            Item::Use { path, .. } => {
                // A `use` brings its last segment into scope. The module graph is not built,
                // so nothing is checked about where it came from — but a name imported and
                // then used must not be reported as unbound.
                self.globals.insert(path.last().text.clone());
                self.functions.insert(path.last().text.clone());
            }
            Item::Schema(_) | Item::View(_) | Item::Error(_) => {}
        }
    }

    // ---- scopes ----

    fn push(&mut self) {
        self.scopes.push(BTreeSet::new());
    }
    fn pop(&mut self) {
        self.scopes.pop();
    }
    fn bind(&mut self, n: &str) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(n.to_string());
        } else {
            self.globals.insert(n.to_string());
        }
    }
    fn bind_pat(&mut self, p: &Pat) {
        for n in p.bindings() {
            self.bind(&n.text);
        }
    }
    fn in_scope(&self, n: &str) -> bool {
        self.globals.contains(n) || self.scopes.iter().any(|s| s.contains(n))
    }

    // ---- items ----

    fn item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.function(f),
            Item::Mod { items, .. } => items.iter().for_each(|i| self.item(i)),
            Item::Impl(i) => i.items.iter().for_each(|f| self.function(f)),
            Item::Trait(t) => t.items.iter().for_each(|f| self.function(f)),
            Item::Const { value, .. } => {
                self.push();
                self.expr(value);
                self.pop();
            }
            // A view body is a query, and a query's names are relations, views and the
            // closure parameters of its stages — all of which this walk handles.
            Item::View(v) => {
                self.push();
                self.expr(&v.body);
                self.pop();
            }
            Item::Schema(s) => {
                for si in &s.items {
                    if let SchemaItem::View(v) = si {
                        self.push();
                        self.expr(&v.body);
                        self.pop();
                    }
                }
            }
            Item::Struct(_)
            | Item::Enum(_)
            | Item::Use { .. }
            | Item::TypeAlias { .. }
            | Item::Capability { .. }
            | Item::Error(_) => {}
        }
    }

    fn function(&mut self, f: &FnDecl) {
        self.push();
        for g in &f.generics {
            self.bind(&g.text);
        }
        for p in &f.params {
            self.bind_pat(&p.pat);
        }
        if let Some(b) = &f.body {
            self.block(b);
        }
        self.pop();
    }

    fn block(&mut self, b: &Block) {
        self.push();
        // A block's own items are visible throughout it, as at the top level.
        for s in &b.stmts {
            if let Stmt::Item(i) = s {
                self.seed_item(i);
            }
        }
        for s in &b.stmts {
            self.stmt(s);
        }
        if let Some(t) = &b.tail {
            self.expr(t);
        }
        self.pop();
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            // The initializer is walked **before** the pattern binds, so `let x = x;`
            // refers to the outer `x` — which is what the language means and what the
            // interpreter does.
            Stmt::Let { pat, init, .. } => {
                if let Some(e) = init {
                    self.expr(e);
                }
                self.bind_pat(pat);
            }
            Stmt::Expr(e) | Stmt::Semi(e) => self.expr(e),
            Stmt::Item(i) => self.item(i),
            // DML names relations and columns, which `resolve.rs` and `typecheck.rs`
            // already judge against the catalog. Walking its expressions here would need
            // the column scope this pass deliberately does not build.
            Stmt::Dml(_) => {}
            Stmt::Error(_) => {}
        }
    }

    // ---- expressions ----

    /// **No wildcard arm.** See this module's header: a new `Expr` variant must be handled
    /// here or the crate does not build.
    fn expr(&mut self, e: &Expr) {
        match e {
            // ---- leaves that bind or name nothing ----
            Expr::Int(..)
            | Expr::Float(..)
            | Expr::Bool(..)
            | Expr::Str(..)
            | Expr::Bytes(..)
            | Expr::Unit(..)
            | Expr::Null(..)
            | Expr::Epoch(..)
            | Expr::Instant { .. }
            | Expr::Duration { .. }
            | Expr::Break(..)
            | Expr::Continue(..)
            | Expr::Error(..) => {}

            // A money literal's currency is checked against the catalog by NL0241, with a
            // better diagnostic than this pass could give.
            Expr::Money { .. } => {}

            Expr::Path(p) => self.path_in_value_position(p),

            Expr::Call { callee, args, span } => {
                match &**callee {
                    Expr::Path(p) if p.segments.len() == 1 => {
                        let name = &p.last().text;
                        let query_form = self.query_depth > 0 && QUERY_FNS.contains(&name.as_str());
                        if !query_form
                            && !self.functions.contains(name.as_str())
                            && !self.in_scope(name)
                        {
                            let cand = closest(name, self.functions.iter().map(|s| s.as_str()))
                                .map(|s| s.to_string());
                            let mut d = Diagnostic::error(
                                UNKNOWN_FN,
                                format!("cannot find function `{name}` in this scope"),
                            )
                            .primary(p.span, "not a function this program declares")
                            .note("a call to a name that denotes nothing used to type-check: the callee was not looked up, so the call contributed no effects and no conservation movement to its caller — the obligation did not move to the runtime, it ceased to exist");
                            if let Some(c) = cand {
                                d = d.note(format!("a function named `{c}` is declared"));
                            }
                            self.d.push(d);
                        }
                    }
                    // A multi-segment callee, or a call of a computed value. Neither is
                    // judged here; see the header.
                    other => self.expr(other),
                }
                let _ = span;
                args.iter().for_each(|a| self.expr(&a.value));
            }

            // Built-in forms spelled like calls. Their names are keywords, not bindings.
            Expr::Hold { args, .. }
            | Expr::Authorize { args, .. }
            | Expr::Declassify { args, .. } => args.iter().for_each(|a| self.expr(&a.value)),

            // The receiver is walked outside query context — `balances` in
            // `balances.as_of(#4200)` must denote a relation or a view — and the arguments
            // inside it, because that is where columns and query forms live.
            Expr::Stage { recv, args, .. } => {
                self.expr(recv);
                self.query_depth += 1;
                args.iter().for_each(|a| self.expr(&a.value));
                self.query_depth -= 1;
            }

            Expr::Closure { params, body, .. } => {
                self.push();
                for (p, _) in params {
                    self.bind_pat(p);
                }
                self.expr(body);
                self.pop();
            }

            Expr::Txn { idem, body, .. } => {
                if let Some(spec) = idem {
                    self.expr(&spec.key);
                    if let Some(w) = &spec.window {
                        self.expr(w);
                    }
                }
                self.block(body);
            }

            Expr::Fx { legs, rate, .. } => {
                legs.iter().for_each(|(_, l)| self.expr(l));
                if let Some(r) = rate {
                    self.expr(r);
                }
            }

            Expr::Resolve { hold, outcome, .. } => {
                self.expr(hold);
                if let ResolveOutcome::Post(a) = outcome {
                    self.expr(a);
                }
            }

            Expr::Fixpoint {
                recv,
                step,
                measure,
                ..
            } => {
                self.expr(recv);
                self.expr(step);
                self.expr(measure);
            }

            Expr::Tuple { elems, .. } | Expr::Array { elems, .. } => {
                elems.iter().for_each(|x| self.expr(x))
            }
            // The path is a type name; the field values are expressions.
            Expr::StructLit { fields, .. } => fields.iter().for_each(|(_, v)| self.expr(v)),
            Expr::Field { base, .. } => self.expr(base),
            Expr::Index { base, index, .. } => {
                self.expr(base);
                self.expr(index);
            }
            Expr::Unary { operand, .. } => self.expr(operand),
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Assign { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            Expr::Cast { expr, .. } | Expr::Try { expr, .. } => self.expr(expr),

            Expr::Block(b) => self.block(b),
            Expr::If {
                cond, then, els, ..
            } => {
                self.expr(cond);
                self.block(then);
                if let Some(e) = els {
                    self.expr(e);
                }
            }
            Expr::Case { arms, els, .. } => {
                for (c, v) in arms {
                    self.expr(c);
                    self.expr(v);
                }
                if let Some(e) = els {
                    self.expr(e);
                }
            }
            Expr::Match {
                scrutinee, arms, ..
            } => {
                self.expr(scrutinee);
                for a in arms {
                    self.push();
                    self.bind_pat(&a.pat);
                    if let Some(g) = &a.guard {
                        self.expr(g);
                    }
                    self.expr(&a.body);
                    self.pop();
                }
            }
            Expr::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Expr::Loop { body, .. } => self.block(body),
            Expr::For {
                pat, iter, body, ..
            } => {
                self.expr(iter);
                self.push();
                self.bind_pat(pat);
                self.block(body);
                self.pop();
            }
            Expr::Return { value, .. } => {
                if let Some(v) = value {
                    self.expr(v);
                }
            }

            Expr::Explain { target, .. } | Expr::Impact { target, .. } => self.expr(target),
            Expr::Reproduce { target, at, .. } => {
                self.expr(target);
                if let Some(a) = at {
                    self.expr(a);
                }
            }

            // The SQL surface: a different binder discipline, stated in the header.
            Expr::Sql { .. } | Expr::Exists { .. } | Expr::Select(_) => {}
        }
    }

    fn path_in_value_position(&mut self, p: &Path) {
        if p.segments.len() != 1 {
            return;
        }
        // In query context a bare name is a column; see the header.
        if self.query_depth > 0 {
            return;
        }
        let name = &p.last().text;
        if self.in_scope(name) {
            return;
        }
        let cand = self
            .scopes
            .iter()
            .flatten()
            .map(|s| s.as_str())
            .chain(self.globals.iter().map(|s| s.as_str()))
            .collect::<Vec<_>>();
        let suggestion = closest(name, cand.into_iter()).map(|s| s.to_string());
        let mut d = Diagnostic::error(UNBOUND, format!("cannot find `{name}` in this scope"))
            .primary(p.span, "not a binding, a relation, a view or a declaration")
            .note("an unbound name used to reach the type checker as `Opaque` — the same classification a value of a user type gets — so it was indistinguishable from a value the checker simply could not see inside");
        if let Some(c) = suggestion {
            d = d.note(format!("`{c}` is in scope"));
        }
        self.d.push(d);
    }
}
