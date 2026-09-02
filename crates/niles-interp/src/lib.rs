//! **Stage 0 execution**: a tree-walking interpreter for Niles's imperative subset
//! (thesis Appendix E.1, E.3).
//!
//! # Why an interpreter, and why it is the missing piece
//!
//! Appendix E specifies a three-stage bootstrap: stage 0 is a compiler written in the
//! host language that accepts the whole of Niles; stage 1 is the Niles-written compiler
//! *compiled by* stage 0; stage 2 is stage 1 recompiling the same sources; stage 3 is the
//! fixpoint check. E.0 recorded, honestly, that stage 1 had no input, because the stage-0
//! compiler had no code generation for the imperative tier: it could lower a *query* to
//! the IR, and it could type-check a function, but it could not **run** one. A bootstrap
//! needs execution, not compilation. Nothing in the definition of stage 0 requires it to
//! emit machine code; it requires it to *evaluate* Niles programs. That is what this
//! crate provides, and it is what turns E.1 from a plan into a thing with an exit status.
//!
//! This is not a workaround. It is how bootstraps are normally started — a first-stage
//! interpreter, then a compiler written in the language and run under it, then the
//! compiler compiling itself. Treating "stage 0 must be a native compiler" as a
//! requirement would have been an error of the thesis's own making.
//!
//! # What it accepts, precisely
//!
//! The **imperative subset**: functions, `let` with patterns, `if`/`else`, `while`,
//! `loop`, `for`, `match`, `break`/`continue`/`return`, blocks with tail values, integer
//! and string and boolean arithmetic, comparison, arrays with index and mutation, structs
//! and their fields, tuple-variant enums, closures, and calls.
//!
//! It does **not** accept the relational tier — `view`, pipelines, `txn`, `hold`, `fx`,
//! `fixpoint` — and does not pretend to. Those forms are rejected by name with
//! [`Error::NotInSubset`], because a bootstrap that silently evaluated `txn` as a plain
//! block would be a bootstrap that had quietly abandoned the conservation guarantee the
//! rest of the thesis is about. The refusal is the honest behaviour and the tests pin it.
//!
//! # Determinism is a property this crate must have, not merely hope for
//!
//! Appendix C.4 imposes a cross-target determinism obligation: the same input must
//! produce byte-identical output on every target. An interpreter is where that obligation
//! is easiest to break — hash-map iteration order, address-derived identity, float
//! formatting, wall-clock reads. Three decisions here address it directly. Environments
//! are `BTreeMap`, so scope iteration is by name and not by hash seed. There is no
//! floating-point arithmetic in the subset at all. And there is no clock, no
//! randomness, and no I/O beyond an explicit output buffer the caller owns, so a program
//! run twice cannot differ. [`determinism_gate`] is the runnable form of that claim.

pub mod ledger;

use ledger::{Ledger, Leg};
use niles_lang::ast::*;
use niles_lang::lexer::Span;
use std::collections::BTreeMap;
use std::rc::Rc;

/// A runtime value.
///
/// Deliberately small. Money, epochs and instants are *carried* but not operated on:
/// arithmetic on them belongs to the checked tier, and an interpreter that added two
/// `Money` values without consulting the currency row would be re-implementing the very
/// thing Contribution 4 makes static. They are here so a program can pass them through.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i128),
    Bool(bool),
    Str(Rc<String>),
    Bytes(Rc<Vec<u8>>),
    Array(Rc<Vec<Value>>),
    Tuple(Rc<Vec<Value>>),
    /// A struct or an enum's struct-variant: a name and ordered fields.
    Record {
        name: String,
        fields: Rc<BTreeMap<String, Value>>,
    },
    /// `Some(x)`, `Tok::Ident(s)` — a path and positional payload.
    Variant {
        path: String,
        payload: Rc<Vec<Value>>,
    },
    Closure(Rc<ClosureVal>),
    /// Carried, not computed on. See the type docs.
    Money {
        minor: i128,
        scale: u32,
        currency: String,
    },
    Epoch(u64),
}

#[derive(Debug, PartialEq)]
pub struct ClosureVal {
    pub params: Vec<Pat>,
    pub body: Expr,
    /// Captured by value at creation. The subset has no interior mutability, so a
    /// snapshot is sound and avoids the reference cycles a shared environment would need.
    pub captured: BTreeMap<String, Value>,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "unit",
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            Value::Str(_) => "str",
            Value::Bytes(_) => "bytes",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Record { .. } => "struct",
            Value::Variant { .. } => "variant",
            Value::Closure(_) => "closure",
            Value::Money { .. } => "money",
            Value::Epoch(_) => "epoch",
        }
    }

    /// A total, deterministic rendering. Used by `print` and by the determinism gate, so
    /// it must never depend on iteration order or on an address.
    pub fn render(&self) -> String {
        match self {
            Value::Unit => "()".into(),
            Value::Int(i) => i.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => (**s).clone(),
            Value::Bytes(b) => format!("b\"{}\"", String::from_utf8_lossy(b)),
            Value::Array(xs) => {
                format!(
                    "[{}]",
                    xs.iter().map(|v| v.render()).collect::<Vec<_>>().join(", ")
                )
            }
            Value::Tuple(xs) => {
                format!(
                    "({})",
                    xs.iter().map(|v| v.render()).collect::<Vec<_>>().join(", ")
                )
            }
            // BTreeMap iteration is by key, so this is stable across runs and targets.
            Value::Record { name, fields } => format!(
                "{name} {{ {} }}",
                fields
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.render()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Variant { path, payload } if payload.is_empty() => path.clone(),
            Value::Variant { path, payload } => format!(
                "{path}({})",
                payload
                    .iter()
                    .map(|v| v.render())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Closure(_) => "<closure>".into(),
            Value::Money {
                minor,
                scale,
                currency,
            } => {
                let d = 10i128.pow(*scale);
                format!(
                    "{}.{:0width$} {currency}",
                    minor / d,
                    (minor % d).abs(),
                    width = *scale as usize
                )
            }
            Value::Epoch(e) => format!("#{e}"),
        }
    }

    fn truthy(&self, at: Span) -> Result<bool, Error> {
        match self {
            Value::Bool(b) => Ok(*b),
            // No coercion. A language whose conditions accept integers is a language in
            // which `if balance` means something, and it should not.
            other => Err(Error::TypeMismatch {
                want: "bool".into(),
                got: other.type_name().into(),
                at,
            }),
        }
    }
}

/// What can go wrong. Every variant carries a span, because a stage-0 failure that cannot
/// be located is a stage-0 failure nobody can fix.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A construct outside the imperative subset. Named, never silently approximated.
    NotInSubset {
        form: &'static str,
        at: Span,
    },
    Unbound {
        name: String,
        at: Span,
    },
    TypeMismatch {
        want: String,
        got: String,
        at: Span,
    },
    NoField {
        name: String,
        at: Span,
    },
    IndexOutOfBounds {
        index: i128,
        len: usize,
        at: Span,
    },
    DivideByZero {
        at: Span,
    },
    /// Integer overflow. Checked rather than wrapping: a ledger interpreter that wrapped
    /// silently would be the exact defect Contribution 4 exists to exclude.
    Overflow {
        op: &'static str,
        at: Span,
    },
    WrongArity {
        want: usize,
        got: usize,
        at: Span,
    },
    NoMatchingArm {
        at: Span,
    },
    /// The fuel ran out. See [`Interp::with_fuel`].
    OutOfFuel,
    NotCallable {
        got: String,
        at: Span,
    },
    /// Call nesting exceeded [`Interp::max_depth`].
    ///
    /// This exists because the alternative is a stack overflow, and a stack overflow in
    /// Rust aborts the *process* — a compiler that dies without a diagnostic when handed a
    /// deeply nested input is worse than one that stops and says why. The limit is a
    /// property of the host, not of Niles, and [`Interp::with_max_depth`] is how a caller
    /// that has arranged a larger stack raises it.
    TooDeep {
        limit: usize,
        at: Span,
    },
    /// The ledger refused a posting set. Not a bug in the program's *evaluation* — the
    /// program ran — but a refusal by the one rule this interpreter enforces, and it carries
    /// the refusal verbatim so a caller does not have to guess which currency was short.
    Refused {
        why: ledger::Refusal,
        at: Span,
    },
}

impl Error {
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::NotInSubset { at, .. }
            | Error::Unbound { at, .. }
            | Error::TypeMismatch { at, .. }
            | Error::NoField { at, .. }
            | Error::IndexOutOfBounds { at, .. }
            | Error::DivideByZero { at }
            | Error::Overflow { at, .. }
            | Error::WrongArity { at, .. }
            | Error::NoMatchingArm { at }
            | Error::NotCallable { at, .. }
            | Error::TooDeep { at, .. }
            | Error::Refused { at, .. } => Some(*at),
            Error::OutOfFuel => None,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Error::NotInSubset { form, .. } => format!(
                "`{form}` is outside the stage-0 imperative subset; the relational tier is \
                 compiled, not interpreted"
            ),
            Error::Unbound { name, .. } => format!("`{name}` is not bound here"),
            Error::TypeMismatch { want, got, .. } => format!("expected {want}, found {got}"),
            Error::NoField { name, .. } => format!("no field `{name}`"),
            Error::IndexOutOfBounds { index, len, .. } => {
                format!("index {index} is out of bounds for a length of {len}")
            }
            Error::DivideByZero { .. } => "division by zero".into(),
            Error::Overflow { op, .. } => format!("integer overflow in `{op}`"),
            Error::WrongArity { want, got, .. } => {
                format!("expected {want} argument(s), found {got}")
            }
            Error::NoMatchingArm { .. } => "no arm matched".into(),
            Error::OutOfFuel => "evaluation exceeded its fuel budget".into(),
            Error::NotCallable { got, .. } => format!("`{got}` is not callable"),
            Error::TooDeep { limit, .. } => {
                format!("call nesting exceeded {limit}; the host stack, not Niles, is the limit")
            }
            Error::Refused { why, .. } => format!("the ledger refused the set: {why}"),
        }
    }
}

/// Non-local control flow, threaded through evaluation as an error-like signal.
enum Flow {
    Err(Error),
    Return(Value),
    Break,
    Continue,
}

impl From<Error> for Flow {
    fn from(e: Error) -> Flow {
        Flow::Err(e)
    }
}

type Eval<T> = Result<T, Flow>;

/// A lexical scope chain. `BTreeMap` rather than `HashMap` for the reason in the module
/// docs: iteration order must not depend on a hash seed.
#[derive(Debug, Default, Clone)]
struct Env {
    frames: Vec<BTreeMap<String, Value>>,
}

impl Env {
    fn new() -> Self {
        Env {
            frames: vec![BTreeMap::new()],
        }
    }
    fn push(&mut self) {
        self.frames.push(BTreeMap::new());
    }
    fn pop(&mut self) {
        self.frames.pop();
    }
    fn define(&mut self, name: &str, v: Value) {
        self.frames
            .last_mut()
            .expect("at least one frame")
            .insert(name.into(), v);
    }
    fn get(&self, name: &str) -> Option<&Value> {
        self.frames.iter().rev().find_map(|f| f.get(name))
    }
    fn set(&mut self, name: &str, v: Value) -> bool {
        for f in self.frames.iter_mut().rev() {
            if let Some(slot) = f.get_mut(name) {
                *slot = v;
                return true;
            }
        }
        false
    }
    /// Everything visible, innermost shadowing outermost. The capture set of a closure.
    fn flatten(&self) -> BTreeMap<String, Value> {
        let mut out = BTreeMap::new();
        for f in &self.frames {
            for (k, v) in f {
                out.insert(k.clone(), v.clone());
            }
        }
        out
    }
}

/// The interpreter.
pub struct Interp {
    fns: BTreeMap<String, Rc<FnDecl>>,
    /// Enum variants, so `Tok::Ident` resolves as a constructor rather than a variable.
    variants: BTreeMap<String, usize>,
    /// Struct field lists, for positional construction and for arity checking.
    structs: BTreeMap<String, Vec<String>>,
    /// Everything the program printed. The only output channel; there is no `stdout`
    /// here, because a determinism gate that compares terminal output compares the
    /// terminal too.
    pub output: Vec<String>,
    /// A step budget. Guarded recursion (§6.7) is a *language* guarantee for the
    /// relational tier; the imperative tier has ordinary `while`, so an interpreter
    /// running a compiler needs a way to stop rather than hang a test suite.
    fuel: u64,
    /// Current call nesting, and the ceiling it may not cross.
    ///
    /// A tree-walking interpreter consumes host stack in proportion to the *Niles* call
    /// depth, and a stack overflow in Rust aborts the process rather than unwinding. So
    /// the depth is counted, and crossing the ceiling is an ordinary [`Error`] with a
    /// span. The default is chosen against the *debug* build, which is where the tests
    /// run and where each frame is widest; [`Interp::with_max_depth`] raises it for a
    /// caller that has arranged the stack to match (see `run_with_stack`).
    depth: usize,
    max_depth: usize,
    /// `idem` windows evaluated past. See [`Interp::txn`]; `nilesc run` reports the count.
    ignored_windows: u32,
    /// The ledger `txn`, `debit`, `credit` and `post` operate on.
    ///
    /// Public because a caller running a function for its *postings* — `nilesc run` — wants
    /// the sealed set afterwards, and because a caller seeding opening balances wants to say
    /// so before the run. Empty and inert unless a program uses the ledger forms.
    pub ledger: Ledger,
}

impl Default for Interp {
    fn default() -> Self {
        Interp::new()
    }
}

impl Interp {
    pub fn new() -> Self {
        Interp {
            fns: BTreeMap::new(),
            variants: BTreeMap::new(),
            structs: BTreeMap::new(),
            output: Vec::new(),
            fuel: 50_000_000,
            depth: 0,
            max_depth: DEFAULT_MAX_DEPTH,
            ignored_windows: 0,
            ledger: Ledger::new(),
        }
    }

    /// How many `idem` windows were evaluated past. See [`Interp::txn`].
    pub fn ignored_windows(&self) -> u32 {
        self.ignored_windows
    }

    pub fn with_fuel(mut self, fuel: u64) -> Self {
        self.fuel = fuel;
        self
    }

    /// Raise the call-depth ceiling. Only meaningful when the caller has also arranged a
    /// stack large enough to hold that many frames — [`run_with_stack`] is the pairing.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Load a parsed program's items. Relational items are *skipped*, not rejected: a
    /// source file may legitimately declare a schema and a helper function, and the
    /// interpreter's business is the function.
    pub fn load(&mut self, prog: &Program) {
        for item in &prog.items {
            self.load_item(item);
        }
    }

    fn load_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => {
                self.fns.insert(f.name.text.clone(), Rc::new(f.clone()));
            }
            Item::Struct(s) => {
                self.structs.insert(
                    s.name.text.clone(),
                    s.fields.iter().map(|f| f.name.text.clone()).collect(),
                );
            }
            Item::Enum(e) => {
                for (v, tys) in &e.variants {
                    self.variants
                        .insert(format!("{}::{}", e.name.text, v.text), tys.len());
                    // The bare form too, so `Ident(s)` works inside the enum's own module,
                    // matching Rust's `use Tok::*`.
                    self.variants.entry(v.text.clone()).or_insert(tys.len());
                }
            }
            Item::Impl(i) => {
                for f in &i.items {
                    self.fns.insert(f.name.text.clone(), Rc::new(f.clone()));
                }
            }
            Item::Mod { items, .. } => {
                for it in items {
                    self.load_item(it);
                }
            }
            _ => {}
        }
    }

    pub fn function_names(&self) -> Vec<String> {
        self.fns.keys().cloned().collect()
    }

    /// Call a top-level function by name.
    pub fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, Error> {
        let at = Span { start: 0, end: 0 };
        let f = self.fns.get(name).cloned().ok_or_else(|| Error::Unbound {
            name: name.into(),
            at,
        })?;
        match self.call_decl(&f, args, at) {
            Ok(v) => Ok(v),
            Err(Flow::Err(e)) => Err(e),
            Err(Flow::Return(v)) => Ok(v),
            Err(_) => Err(Error::NotInSubset {
                form: "break outside a loop",
                at,
            }),
        }
    }

    fn call_decl(&mut self, f: &FnDecl, args: Vec<Value>, at: Span) -> Eval<Value> {
        if args.len() != f.params.len() {
            return Err(Error::WrongArity {
                want: f.params.len(),
                got: args.len(),
                at,
            }
            .into());
        }
        let body = match &f.body {
            Some(b) => b.clone(),
            None => {
                return Err(Error::NotInSubset {
                    form: "function without a body",
                    at,
                }
                .into())
            }
        };
        // A fresh environment, not the caller's: the subset has no dynamic scope.
        let mut env = Env::new();
        for (p, v) in f.params.iter().zip(args) {
            self.bind(&mut env, &p.pat, v)?;
        }
        self.enter(at)?;
        let r = match self.block(&mut env, &body) {
            Ok(v) => Ok(v),
            Err(Flow::Return(v)) => Ok(v),
            Err(other) => Err(other),
        };
        self.depth -= 1;
        r
    }

    /// Count one level of call nesting, refusing rather than overflowing the host stack.
    fn enter(&mut self, at: Span) -> Eval<()> {
        if self.depth >= self.max_depth {
            return Err(Error::TooDeep {
                limit: self.max_depth,
                at,
            }
            .into());
        }
        self.depth += 1;
        Ok(())
    }

    fn burn(&mut self) -> Eval<()> {
        if self.fuel == 0 {
            return Err(Error::OutOfFuel.into());
        }
        self.fuel -= 1;
        Ok(())
    }

    // ── statements and blocks ────────────────────────────────────────────────────────

    fn block(&mut self, env: &mut Env, b: &Block) -> Eval<Value> {
        env.push();
        let r = self.block_inner(env, b);
        env.pop();
        r
    }

    fn block_inner(&mut self, env: &mut Env, b: &Block) -> Eval<Value> {
        for s in &b.stmts {
            self.stmt(env, s)?;
        }
        match &b.tail {
            Some(e) => self.expr(env, e),
            None => Ok(Value::Unit),
        }
    }

    fn stmt(&mut self, env: &mut Env, s: &Stmt) -> Eval<()> {
        self.burn()?;
        match s {
            Stmt::Let {
                pat, init, span, ..
            } => {
                let v = match init {
                    Some(e) => self.expr(env, e)?,
                    None => Value::Unit,
                };
                let _ = span;
                self.bind(env, pat, v)
            }
            Stmt::Expr(e) | Stmt::Semi(e) => {
                self.expr(env, e)?;
                Ok(())
            }
            Stmt::Item(i) => {
                self.load_item(i);
                Ok(())
            }
            Stmt::Dml(d) => Err(Error::NotInSubset {
                form: match d {
                    Dml::Insert { .. } => "insert",
                    Dml::Update { .. } => "update",
                    Dml::Delete { .. } => "delete",
                    Dml::Begin(_) => "begin",
                    Dml::Commit(_) => "commit",
                    Dml::Rollback(_) => "rollback",
                    Dml::Grant { .. } => "grant",
                    Dml::Revoke { .. } => "revoke",
                    Dml::Backfill { .. } => "backfill",
                    Dml::Emit { .. } => "emit",
                },
                at: Span { start: 0, end: 0 },
            }
            .into()),
            Stmt::Error(sp) => Err(Error::NotInSubset {
                form: "a statement that did not parse",
                at: *sp,
            }
            .into()),
        }
    }

    // ── patterns ─────────────────────────────────────────────────────────────────────

    fn bind(&mut self, env: &mut Env, p: &Pat, v: Value) -> Eval<()> {
        match self.try_bind(env, p, &v) {
            Ok(true) => Ok(()),
            Ok(false) => Err(Error::NoMatchingArm { at: p.span() }.into()),
            Err(e) => Err(e),
        }
    }

    /// Attempt a match, binding on success. Returns `false` for a refutable pattern that
    /// did not match, which is how `match` arms are tried in order.
    fn try_bind(&mut self, env: &mut Env, p: &Pat, v: &Value) -> Eval<bool> {
        match p {
            Pat::Wild(_) => Ok(true),
            Pat::Bind { name, .. } => {
                env.define(&name.text, v.clone());
                Ok(true)
            }
            Pat::Tuple { elems, span } => match v {
                Value::Tuple(xs) | Value::Array(xs) if xs.len() == elems.len() => {
                    for (pe, ve) in elems.iter().zip(xs.iter()) {
                        if !self.try_bind(env, pe, ve)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => {
                    let _ = span;
                    Ok(false)
                }
            },
            Pat::TupleStruct { path, elems, .. } => {
                let want = path_text(path);
                match v {
                    Value::Variant { path: got, payload }
                        if variant_matches(&want, got) && payload.len() == elems.len() =>
                    {
                        for (pe, ve) in elems.iter().zip(payload.iter()) {
                            if !self.try_bind(env, pe, ve)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Pat::Struct { path, fields, .. } => {
                let want = path_text(path);
                match v {
                    Value::Record { name, fields: got } if variant_matches(&want, name) => {
                        for (fname, fpat) in fields {
                            match got.get(&fname.text) {
                                Some(fv) => {
                                    let fv = fv.clone();
                                    if !self.try_bind(env, fpat, &fv)? {
                                        return Ok(false);
                                    }
                                }
                                None => return Ok(false),
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Pat::Lit(e) => {
                let lv = self.expr(env, e)?;
                Ok(&lv == v)
            }
            Pat::Path(path) => {
                let want = path_text(path);
                // A bare lowercase identifier in pattern position with no known variant is
                // a binding, matching Rust. An uppercase one, or one that names a known
                // variant, is a unit-variant test.
                let known = self.variants.contains_key(&want);
                let uppercase = want.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                if known || uppercase || want.contains("::") {
                    match v {
                        Value::Variant { path: got, payload } if payload.is_empty() => {
                            Ok(variant_matches(&want, got))
                        }
                        _ => Ok(false),
                    }
                } else {
                    env.define(&want, v.clone());
                    Ok(true)
                }
            }
            Pat::Error(sp) => Err(Error::NotInSubset {
                form: "a pattern that did not parse",
                at: *sp,
            }
            .into()),
        }
    }

    // ── expressions ──────────────────────────────────────────────────────────────────

    fn expr(&mut self, env: &mut Env, e: &Expr) -> Eval<Value> {
        self.burn()?;
        match e {
            // --- leaves ---
            Expr::Int(i, _) => Ok(Value::Int(*i)),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Str(s, _) => Ok(Value::Str(Rc::new(s.clone()))),
            Expr::Bytes(b, _) => Ok(Value::Bytes(Rc::new(b.clone()))),
            Expr::Unit(_) => Ok(Value::Unit),
            Expr::Money {
                minor,
                scale,
                currency,
                ..
            } => Ok(Value::Money {
                minor: *minor,
                scale: *scale,
                currency: currency.text.clone(),
            }),
            Expr::Epoch(n, _) => Ok(Value::Epoch(*n)),
            // Floats are excluded from the subset on purpose: cross-target determinism
            // (Appendix C.4) and money never being a float are the same commitment.
            Expr::Float(_, sp) => Err(Error::NotInSubset {
                form: "floating-point literal",
                at: *sp,
            }
            .into()),
            // Temporal literals belong to the bitemporal tier, which is checked and
            // lowered rather than interpreted. Carrying them as opaque values would let a
            // Niles program compare two instants without the axis discipline of §3.8.
            Expr::Instant { span, .. } => Err(Error::NotInSubset {
                form: "temporal literal",
                at: *span,
            }
            .into()),
            Expr::Duration { span, .. } => Err(Error::NotInSubset {
                form: "duration literal",
                at: *span,
            }
            .into()),

            Expr::Path(p) => {
                let name = path_text(p);
                if let Some(v) = env.get(&name) {
                    return Ok(v.clone());
                }
                if let Some(&arity) = self.variants.get(&name) {
                    if arity == 0 {
                        return Ok(Value::Variant {
                            path: name,
                            payload: Rc::new(vec![]),
                        });
                    }
                }
                Err(Error::Unbound { name, at: p.span }.into())
            }

            // --- composition ---
            Expr::Tuple { elems, .. } => {
                let vs = self.eval_all(env, elems)?;
                Ok(Value::Tuple(Rc::new(vs)))
            }
            Expr::Array { elems, .. } => {
                let vs = self.eval_all(env, elems)?;
                Ok(Value::Array(Rc::new(vs)))
            }
            Expr::StructLit { path, fields, .. } => self.struct_lit(env, path, fields),
            Expr::Field { base, name, span } => self.field_of(env, base, name, *span),
            Expr::Index { base, index, span } => self.index_of(env, base, index, *span),

            Expr::Closure { params, body, .. } => self.make_closure(env, params, body),

            Expr::Call { callee, args, span } => self.call_expr(env, callee, args, *span),

            // `txn idem(key, window: …) { … }` — the one relational-tier form the interpreter
            // evaluates, because a posting set has to be sealed somewhere for a function to
            // have run at all. Everything else in `refused` below stays refused.
            Expr::Txn { idem, body, span } => self.txn(env, idem.as_ref(), body, *span),

            Expr::Unary { op, operand, span } => self.unary_op(env, *op, operand, *span),

            Expr::Binary { op, lhs, rhs, span } => self.binary(env, *op, lhs, rhs, *span),

            Expr::Assign {
                target,
                value,
                span,
            } => self.assign(env, target, value, *span),

            // A cast in the subset is between integers and is therefore the identity;
            // `as u8`-style truncation is not modelled, because a silent truncation in a
            // ledger interpreter is exactly the class of defect this thesis argues against.
            Expr::Cast { expr, .. } => self.expr(env, expr),

            Expr::Try { expr, span } => self.try_op(env, expr, *span),

            // --- control ---
            Expr::Block(b) => self.block(env, b),

            Expr::If {
                cond,
                then,
                els,
                span,
            } => {
                let c = self.expr(env, cond)?.truthy(*span)?;
                if c {
                    self.block(env, then)
                } else {
                    match els {
                        Some(e) => self.expr(env, e),
                        None => Ok(Value::Unit),
                    }
                }
            }

            Expr::Case { arms, els, span } => self.case_arms(env, arms, els.as_deref(), *span),
            Expr::Match {
                scrutinee,
                arms,
                span,
            } => self.match_expr(env, scrutinee, arms, *span),

            Expr::While { cond, body, span } => {
                loop {
                    self.burn()?;
                    if !self.expr(env, cond)?.truthy(*span)? {
                        break;
                    }
                    match self.block(env, body) {
                        Ok(_) | Err(Flow::Continue) => {}
                        Err(Flow::Break) => break,
                        Err(other) => return Err(other),
                    }
                }
                Ok(Value::Unit)
            }

            Expr::Loop { body, .. } => {
                loop {
                    self.burn()?;
                    match self.block(env, body) {
                        Ok(_) | Err(Flow::Continue) => {}
                        Err(Flow::Break) => break,
                        Err(other) => return Err(other),
                    }
                }
                Ok(Value::Unit)
            }

            Expr::For {
                pat,
                iter,
                body,
                span,
            } => self.for_expr(env, pat, iter, body, *span),

            Expr::Return { value, .. } => {
                let v = match value {
                    Some(e) => self.expr(env, e)?,
                    None => Value::Unit,
                };
                Err(Flow::Return(v))
            }
            Expr::Break(_) => Err(Flow::Break),
            Expr::Continue(_) => Err(Flow::Continue),

            Expr::Stage { span, .. } => self.stage_or_method(env, e, *span),

            // --- the relational tier: named, refused, never approximated ---
            _ => Err(Flow::Err(refused(e))),
        }
    }

    /// Everything outside the imperative subset, named rather than approximated.
    ///
    /// Split out of [`Interp::expr`] with the other cold arms for a reason that is a
    /// finding in its own right (§E.19): a `match` with thirty arms compiles, in a debug
    /// build, to a frame holding the union of every arm's locals. Measured at ~95 KB per
    /// Niles call frame, which put a recursive-descent parser out of reach on any ordinary
    /// stack. Moving the fat arms behind `#[inline(never)]` is what makes stage 1 able to
    /// run a parser at all.
    #[inline(never)]
    fn assign(&mut self, env: &mut Env, target: &Expr, value: &Expr, span: Span) -> Eval<Value> {
        {
            let v = self.expr(env, value)?;
            let span = &span;
            match target {
                Expr::Path(p) => {
                    let name = path_text(p);
                    if env.set(&name, v) {
                        Ok(Value::Unit)
                    } else {
                        Err(Error::Unbound { name, at: *span }.into())
                    }
                }
                // `xs[i] = v`. Arrays are `Rc<Vec<_>>`, so this reads, mutates a
                // clone, and rebinds — O(n) per write. Correct and slow, which is the
                // right trade for a stage-0 interpreter whose successor is a compiler.
                Expr::Index { base, index, .. } => {
                    let idx = match self.expr(env, index)? {
                        Value::Int(n) => n,
                        other => {
                            return Err(Error::TypeMismatch {
                                want: "int".into(),
                                got: other.type_name().into(),
                                at: *span,
                            }
                            .into())
                        }
                    };
                    let name = match &**base {
                        Expr::Path(p) => path_text(p),
                        _ => {
                            return Err(Error::NotInSubset {
                                form: "assignment to a computed place",
                                at: *span,
                            }
                            .into())
                        }
                    };
                    let cur = env.get(&name).cloned().ok_or_else(|| {
                        Flow::Err(Error::Unbound {
                            name: name.clone(),
                            at: *span,
                        })
                    })?;
                    match cur {
                        Value::Array(xs) => {
                            if idx < 0 || idx as usize >= xs.len() {
                                return Err(Error::IndexOutOfBounds {
                                    index: idx,
                                    len: xs.len(),
                                    at: *span,
                                }
                                .into());
                            }
                            let mut new = (*xs).clone();
                            new[idx as usize] = v;
                            env.set(&name, Value::Array(Rc::new(new)));
                            Ok(Value::Unit)
                        }
                        other => Err(Error::TypeMismatch {
                            want: "array".into(),
                            got: other.type_name().into(),
                            at: *span,
                        }
                        .into()),
                    }
                }
                Expr::Field { base, name, .. } => {
                    let vname = match &**base {
                        Expr::Path(p) => path_text(p),
                        _ => {
                            return Err(Error::NotInSubset {
                                form: "assignment to a computed place",
                                at: *span,
                            }
                            .into())
                        }
                    };
                    let cur = env.get(&vname).cloned().ok_or_else(|| {
                        Flow::Err(Error::Unbound {
                            name: vname.clone(),
                            at: *span,
                        })
                    })?;
                    match cur {
                        Value::Record { name: rn, fields } => {
                            let mut m = (*fields).clone();
                            m.insert(name.text.clone(), v);
                            env.set(
                                &vname,
                                Value::Record {
                                    name: rn,
                                    fields: Rc::new(m),
                                },
                            );
                            Ok(Value::Unit)
                        }
                        other => Err(Error::TypeMismatch {
                            want: "struct".into(),
                            got: other.type_name().into(),
                            at: *span,
                        }
                        .into()),
                    }
                }
                _ => Err(Error::NotInSubset {
                    form: "assignment to this place",
                    at: *span,
                }
                .into()),
            }
        }
    }

    #[inline(never)]
    fn field_of(&mut self, env: &mut Env, base: &Expr, name: &Name, span: Span) -> Eval<Value> {
        let b = self.expr(env, base)?;
        match &b {
            Value::Record { fields, .. } => fields.get(&name.text).cloned().ok_or_else(|| {
                Error::NoField {
                    name: name.text.clone(),
                    at: span,
                }
                .into()
            }),
            // Tuple field access, `t.0`.
            Value::Tuple(xs) => match name.text.parse::<usize>() {
                Ok(i) if i < xs.len() => Ok(xs[i].clone()),
                _ => Err(Error::NoField {
                    name: name.text.clone(),
                    at: span,
                }
                .into()),
            },
            other => Err(Error::TypeMismatch {
                want: "struct".into(),
                got: other.type_name().into(),
                at: span,
            }
            .into()),
        }
    }

    #[inline(never)]
    fn index_of(&mut self, env: &mut Env, base: &Expr, index: &Expr, span: Span) -> Eval<Value> {
        let b = self.expr(env, base)?;
        let idx = match self.expr(env, index)? {
            Value::Int(n) => n,
            other => {
                return Err(Error::TypeMismatch {
                    want: "int".into(),
                    got: other.type_name().into(),
                    at: span,
                }
                .into())
            }
        };
        match &b {
            Value::Array(xs) | Value::Tuple(xs) => {
                if idx < 0 || idx as usize >= xs.len() {
                    Err(Error::IndexOutOfBounds {
                        index: idx,
                        len: xs.len(),
                        at: span,
                    }
                    .into())
                } else {
                    Ok(xs[idx as usize].clone())
                }
            }
            Value::Bytes(bs) => {
                if idx < 0 || idx as usize >= bs.len() {
                    Err(Error::IndexOutOfBounds {
                        index: idx,
                        len: bs.len(),
                        at: span,
                    }
                    .into())
                } else {
                    Ok(Value::Int(bs[idx as usize] as i128))
                }
            }
            other => Err(Error::TypeMismatch {
                want: "array".into(),
                got: other.type_name().into(),
                at: span,
            }
            .into()),
        }
    }

    #[inline(never)]
    fn struct_lit(&mut self, env: &mut Env, path: &Path, fields: &[(Name, Expr)]) -> Eval<Value> {
        let mut m = BTreeMap::new();
        for (n, fe) in fields {
            let v = self.expr(env, fe)?;
            m.insert(n.text.clone(), v);
        }
        Ok(Value::Record {
            name: path_text(path),
            fields: Rc::new(m),
        })
    }

    #[inline(never)]
    fn make_closure(
        &mut self,
        env: &mut Env,
        params: &[(Pat, Option<Ty>)],
        body: &Expr,
    ) -> Eval<Value> {
        Ok(Value::Closure(Rc::new(ClosureVal {
            params: params.iter().map(|(p, _)| p.clone()).collect(),
            body: body.clone(),
            captured: env.flatten(),
        })))
    }

    #[inline(never)]
    fn unary_op(&mut self, env: &mut Env, op: UnOp, operand: &Expr, span: Span) -> Eval<Value> {
        let v = self.expr(env, operand)?;
        match (op, &v) {
            (UnOp::Neg, Value::Int(i)) => i
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| Error::Overflow { op: "-", at: span }.into()),
            (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            // References are a no-op here: the subset has no aliasing, because values are
            // cloned. §6.6's memory model is a compile-time claim, and an interpreter that
            // faked ownership would be evidence for nothing.
            (UnOp::Ref | UnOp::RefMut | UnOp::Deref, _) => Ok(v),
            _ => Err(Error::TypeMismatch {
                want: "a numeric or boolean operand".into(),
                got: v.type_name().into(),
                at: span,
            }
            .into()),
        }
    }

    #[inline(never)]
    fn try_op(&mut self, env: &mut Env, inner: &Expr, span: Span) -> Eval<Value> {
        let v = self.expr(env, inner)?;
        match &v {
            Value::Variant { path, payload } if variant_matches("Ok", path) => {
                Ok(payload.first().cloned().unwrap_or(Value::Unit))
            }
            Value::Variant { path, .. } if variant_matches("Err", path) => Err(Flow::Return(v)),
            Value::Variant { path, payload } if variant_matches("Some", path) => {
                Ok(payload.first().cloned().unwrap_or(Value::Unit))
            }
            Value::Variant { path, .. } if variant_matches("None", path) => Err(Flow::Return(v)),
            other => Err(Error::TypeMismatch {
                want: "Result or Option".into(),
                got: other.type_name().into(),
                at: span,
            }
            .into()),
        }
    }

    #[inline(never)]
    fn case_arms(
        &mut self,
        env: &mut Env,
        arms: &[(Expr, Expr)],
        els: Option<&Expr>,
        span: Span,
    ) -> Eval<Value> {
        for (pred, body) in arms {
            if self.expr(env, pred)?.truthy(span)? {
                return self.expr(env, body);
            }
        }
        match els {
            Some(e) => self.expr(env, e),
            // SQL's `case` with no `else` yields null. The subset has no null, so this is
            // unit, and the difference is documented rather than hidden.
            None => Ok(Value::Unit),
        }
    }

    #[inline(never)]
    fn match_expr(
        &mut self,
        env: &mut Env,
        scrutinee: &Expr,
        arms: &[MatchArm],
        span: Span,
    ) -> Eval<Value> {
        let v = self.expr(env, scrutinee)?;
        for arm in arms {
            env.push();
            let matched = match self.try_bind(env, &arm.pat, &v) {
                Ok(m) => m,
                Err(e) => {
                    env.pop();
                    return Err(e);
                }
            };
            if matched {
                let guard_ok = match &arm.guard {
                    Some(g) => match self.expr(env, g).and_then(|x| Ok(x.truthy(span)?)) {
                        Ok(b) => b,
                        Err(e) => {
                            env.pop();
                            return Err(e);
                        }
                    },
                    None => true,
                };
                if guard_ok {
                    let r = self.expr(env, &arm.body);
                    env.pop();
                    return r;
                }
            }
            env.pop();
        }
        Err(Error::NoMatchingArm { at: span }.into())
    }

    #[inline(never)]
    fn for_expr(
        &mut self,
        env: &mut Env,
        pat: &Pat,
        iter: &Expr,
        body: &Block,
        span: Span,
    ) -> Eval<Value> {
        let it = self.expr(env, iter)?;
        let items: Vec<Value> = match &it {
            Value::Array(xs) | Value::Tuple(xs) => (**xs).clone(),
            Value::Bytes(bs) => bs.iter().map(|b| Value::Int(*b as i128)).collect(),
            Value::Record { name, fields } if name == "Range" => {
                let lo = as_int(fields.get("start"), span)?;
                let hi = as_int(fields.get("end"), span)?;
                (lo..hi).map(Value::Int).collect()
            }
            other => {
                return Err(Error::TypeMismatch {
                    want: "an iterable".into(),
                    got: other.type_name().into(),
                    at: span,
                }
                .into())
            }
        };
        for v in items {
            self.burn()?;
            env.push();
            let bound = self.try_bind(env, pat, &v);
            let r = match bound {
                Ok(true) => self.block_inner(env, body).map(|_| ()),
                Ok(false) => Err(Flow::Err(Error::NoMatchingArm { at: span })),
                Err(e) => Err(e),
            };
            env.pop();
            match r {
                Ok(()) | Err(Flow::Continue) => {}
                Err(Flow::Break) => break,
                Err(other) => return Err(other),
            }
        }
        Ok(Value::Unit)
    }

    fn eval_all(&mut self, env: &mut Env, es: &[Expr]) -> Eval<Vec<Value>> {
        // Left to right, always. Evaluation order is observable through `print`, so it is
        // part of the determinism obligation and not an implementation detail.
        es.iter().map(|e| self.expr(env, e)).collect()
    }

    /// A `.method(..)` reaches the parser as a `Stage`. In the imperative subset those
    /// are the collection and string builtins.
    fn stage_or_method(&mut self, env: &mut Env, e: &Expr, span: Span) -> Eval<Value> {
        let (recv, name, args) = match e {
            Expr::Stage {
                recv, name, args, ..
            } => (recv, name.text.as_str(), args),
            _ => unreachable!("stage_or_method called on a non-stage"),
        };
        let r = self.expr(env, recv)?;
        let mut vs = Vec::new();
        for a in args {
            vs.push(self.expr(env, &a.value)?);
        }
        builtin_method(&r, name, &vs, span).map_err(Flow::Err)
    }

    fn call_expr(&mut self, env: &mut Env, callee: &Expr, args: &[Arg], span: Span) -> Eval<Value> {
        let name = match callee {
            Expr::Path(p) => Some(path_text(p)),
            _ => None,
        };
        let mut vs = Vec::new();
        for a in args {
            vs.push(self.expr(env, &a.value)?);
        }

        if let Some(n) = &name {
            // A variant constructor: `Tok::Ident("x")`.
            if let Some(&arity) = self.variants.get(n) {
                if arity == vs.len() {
                    return Ok(Value::Variant {
                        path: n.clone(),
                        payload: Rc::new(vs),
                    });
                }
            }
            // A struct built positionally.
            if let Some(fields) = self.structs.get(n).cloned() {
                if fields.len() == vs.len() {
                    let m: BTreeMap<String, Value> = fields.into_iter().zip(vs.clone()).collect();
                    return Ok(Value::Record {
                        name: n.clone(),
                        fields: Rc::new(m),
                    });
                }
            }
            // A user function.
            if let Some(f) = self.fns.get(n).cloned() {
                return self.call_decl(&f, vs, span);
            }
            // A ledger builtin, then a free one.
            if let Some(v) = self.builtin_ledger(n, &vs, span)? {
                return Ok(v);
            }
            if let Some(v) = self.builtin_free(n, &vs, span)? {
                return Ok(v);
            }
            // A closure held in a variable.
            if let Some(Value::Closure(c)) = env.get(n).cloned() {
                return self.call_closure(&c, vs, span);
            }
            return Err(Error::Unbound {
                name: n.clone(),
                at: span,
            }
            .into());
        }

        match self.expr(env, callee)? {
            Value::Closure(c) => self.call_closure(&c, vs, span),
            other => Err(Error::NotCallable {
                got: other.type_name().into(),
                at: span,
            }
            .into()),
        }
    }

    fn call_closure(&mut self, c: &Rc<ClosureVal>, args: Vec<Value>, span: Span) -> Eval<Value> {
        if args.len() != c.params.len() {
            return Err(Error::WrongArity {
                want: c.params.len(),
                got: args.len(),
                at: span,
            }
            .into());
        }
        let mut env = Env::new();
        for (k, v) in &c.captured {
            env.define(k, v.clone());
        }
        env.push();
        for (p, v) in c.params.iter().zip(args) {
            self.bind(&mut env, p, v)?;
        }
        self.enter(span)?;
        let r = match self.expr(&mut env, &c.body) {
            Ok(v) => Ok(v),
            Err(Flow::Return(v)) => Ok(v),
            Err(other) => Err(other),
        };
        self.depth -= 1;
        r
    }

    /// `txn idem(key, window: …) { … }` — open a transaction, run the body, seal it.
    ///
    /// The identity is the `idem` key, evaluated: it is the transaction's name in the encoded
    /// set, so a schema saying `idem("transfer", window: 30.days)` produces a set whose txn is
    /// `"transfer"` and whose bytes a Rust product's `PostingSet::new("transfer")` can be
    /// compared against. A `txn` with no `idem` clause takes the empty identity, which is what
    /// a set nobody named has and is not a name this interpreter invents.
    ///
    /// The window is **evaluated and discarded**, deliberately and not silently: the
    /// duration literal it holds is outside the subset, so evaluating it would refuse the whole
    /// function over a clause whose only effect is on an idempotency store this interpreter
    /// does not have. Ignoring the *value* while honouring the *identity* is the honest split,
    /// and a run that needed the window would be a run this interpreter should refuse rather
    /// than approximate — which is why `nilesc run` reports the window it ignored.
    ///
    /// If the body fails, the open transaction is abandoned rather than left half-built: a
    /// refused transaction that leaves legs behind would make the next `txn` seal somebody
    /// else's money.
    fn txn(
        &mut self,
        env: &mut Env,
        idem: Option<&IdemSpec>,
        body: &Block,
        span: Span,
    ) -> Eval<Value> {
        let key = match idem {
            None => String::new(),
            Some(spec) => match self.expr(env, &spec.key)? {
                Value::Str(s) => (*s).clone(),
                other => {
                    return Err(Error::TypeMismatch {
                        want: "a string idempotency key".into(),
                        got: other.type_name().into(),
                        at: spec.span,
                    }
                    .into())
                }
            },
        };
        if let Some(spec) = idem {
            if spec.window.is_some() {
                self.ignored_windows += 1;
            }
        }
        self.ledger.begin(&key);
        let out = self.block(env, body);
        if out.is_err() {
            self.ledger.abandon();
            return out;
        }
        self.ledger
            .seal()
            .map_err(|why| Flow::Err(Error::Refused { why, at: span }))?;
        out
    }

    /// The ledger builtins: `acct`, `debit`, `credit`, `post`.
    ///
    /// Separate from [`Interp::builtin_free`] so that the free-function table stays the
    /// "100-line host shim" it is documented to be, and so a reader looking for what the
    /// interpreter does to money finds it in one place.
    fn builtin_ledger(&mut self, name: &str, vs: &[Value], span: Span) -> Eval<Option<Value>> {
        let v = match (name, vs) {
            // An account identifier. Niles writes `acct(1001)` and the schema's `Id<Account>`
            // is opaque, so the interpreter's account *name* is the rendering of whatever was
            // passed — an integer renders as its digits, a string as itself. That makes the
            // account a caller's choice, which is what lets a fixture pass the concrete names
            // a Rust product posts to (`loan.fac-1.agent`) where the schema says `lender_a`.
            ("acct", [v]) => Value::Str(Rc::new(v.render())),
            ("debit", [a, m]) => self.leg(a, m, -1, span)?,
            ("credit", [a, m]) => self.leg(a, m, 1, span)?,
            ("post", legs) => {
                let mut out = Vec::with_capacity(legs.len());
                for l in legs {
                    out.push(as_leg(l, span).map_err(Flow::Err)?);
                }
                self.ledger
                    .push_legs(out)
                    .map_err(|why| Flow::Err(Error::Refused { why, at: span }))?;
                // `post` yields the transaction identity, so `txn { … post(…) }` evaluates to
                // it and a function returning `Result<TxnId, TxnError>` has something to
                // return. Wrapped in `Ok`, because that is the declared type.
                let id = self
                    .ledger
                    .open
                    .as_ref()
                    .map(|o| o.txn.clone())
                    .unwrap_or_default();
                Value::Variant {
                    path: "Ok".into(),
                    payload: Rc::new(vec![Value::Str(Rc::new(id))]),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    /// One leg. `sign` is −1 for a debit and +1 for a credit.
    ///
    /// The sign convention is the one place the two representations of a posting genuinely
    /// differ, and it is written here rather than inferred: **Niles writes the verb and the
    /// amount is positive; the kernel stores a signed amount and derives the verb.** Getting
    /// this backwards would make every conformance fixture pass while the two implementations
    /// moved money in opposite directions.
    fn leg(&mut self, account: &Value, amount: &Value, sign: i128, span: Span) -> Eval<Value> {
        let account = match account {
            Value::Str(s) => (**s).clone(),
            other => other.render(),
        };
        let Value::Money {
            minor,
            scale,
            currency,
        } = amount
        else {
            return Err(Error::TypeMismatch {
                want: "money".into(),
                got: amount.type_name().into(),
                at: span,
            }
            .into());
        };
        let signed = minor.checked_mul(sign).ok_or(Error::Overflow {
            op: if sign < 0 { "debit" } else { "credit" },
            at: span,
        })?;
        let mut fields = BTreeMap::new();
        fields.insert("account".into(), Value::Str(Rc::new(account)));
        fields.insert("minor".into(), Value::Int(signed));
        fields.insert("scale".into(), Value::Int(*scale as i128));
        // **Upper case.** The kernel's `Currency::new` documents this as "the only
        // normalisation the kernel performs", and it performs it because two spellings of one
        // currency would defeat per-currency conservation: the residuals would balance
        // separately and both would be zero. A schema writes `300.00 usd` and a product writes
        // `"USD"`, so the second implementation of a normative rule applies the same rule —
        // otherwise a conformance comparison fails on a case difference and reports it as a
        // divergence about money.
        fields.insert(
            "currency".into(),
            Value::Str(Rc::new(currency.to_ascii_uppercase())),
        );
        let leg = Value::Record {
            name: "Leg".into(),
            fields: Rc::new(fields),
        };
        // `debit(...)` is written with a `?` in every schema in this repository, so it must be
        // a `Result`; `credit(...)` is not, so it must not be. Rather than have two shapes,
        // both return the bare leg and `?` on a non-`Result` would be an error — which is
        // wrong. So a debit is wrapped and a credit is not, exactly matching how the schema
        // writes them, and `as_leg` below unwraps whichever arrives.
        Ok(if sign < 0 {
            Value::Variant {
                path: "Ok".into(),
                payload: Rc::new(vec![leg]),
            }
        } else {
            leg
        })
    }

    /// Free functions. Deliberately few: this is the "100-line host shim" of Appendix E.3,
    /// and every entry is a capability the Niles-written compiler would otherwise have to
    /// obtain from somewhere unprincipled.
    fn builtin_free(&mut self, name: &str, vs: &[Value], span: Span) -> Eval<Option<Value>> {
        let v = match (name, vs) {
            ("print", [v]) => {
                self.output.push(v.render());
                Value::Unit
            }
            ("len", [v]) => Value::Int(value_len(v, span).map_err(Flow::Err)? as i128),
            ("panic", [v]) => {
                return Err(Error::TypeMismatch {
                    want: "no panic".into(),
                    got: v.render(),
                    at: span,
                }
                .into())
            }
            ("Some", [v]) => Value::Variant {
                path: "Some".into(),
                payload: Rc::new(vec![v.clone()]),
            },
            ("Ok", [v]) => Value::Variant {
                path: "Ok".into(),
                payload: Rc::new(vec![v.clone()]),
            },
            ("Err", [v]) => Value::Variant {
                path: "Err".into(),
                payload: Rc::new(vec![v.clone()]),
            },
            ("range", [Value::Int(a), Value::Int(b)]) => {
                let mut m = BTreeMap::new();
                m.insert("start".to_string(), Value::Int(*a));
                m.insert("end".to_string(), Value::Int(*b));
                Value::Record {
                    name: "Range".into(),
                    fields: Rc::new(m),
                }
            }
            ("chr", [Value::Int(i)]) => Value::Str(Rc::new(((*i as u8) as char).to_string())),
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    fn binary(&mut self, env: &mut Env, op: BinOp, l: &Expr, r: &Expr, span: Span) -> Eval<Value> {
        // Short-circuit before evaluating the right side, or `i < len(s) && s[i] == 32`
        // reads out of bounds at the end of every string in the Niles lexer.
        if matches!(op, BinOp::And | BinOp::Or) {
            let lv = self.expr(env, l)?.truthy(span)?;
            return match (op, lv) {
                (BinOp::And, false) => Ok(Value::Bool(false)),
                (BinOp::Or, true) => Ok(Value::Bool(true)),
                _ => Ok(Value::Bool(self.expr(env, r)?.truthy(span)?)),
            };
        }
        let a = self.expr(env, l)?;
        let b = self.expr(env, r)?;
        arith(op, &a, &b, span).map_err(Flow::Err)
    }
}

fn as_int(v: Option<&Value>, at: Span) -> Eval<i128> {
    match v {
        Some(Value::Int(i)) => Ok(*i),
        Some(other) => Err(Error::TypeMismatch {
            want: "int".into(),
            got: other.type_name().into(),
            at,
        }
        .into()),
        None => Err(Error::NoField {
            name: "start/end".into(),
            at,
        }
        .into()),
    }
}

fn value_len(v: &Value, at: Span) -> Result<usize, Error> {
    match v {
        Value::Str(s) => Ok(s.len()),
        Value::Bytes(b) => Ok(b.len()),
        Value::Array(xs) | Value::Tuple(xs) => Ok(xs.len()),
        other => Err(Error::TypeMismatch {
            want: "a sized value".into(),
            got: other.type_name().into(),
            at,
        }),
    }
}

/// Whether a pattern's path names this runtime variant. `Tok::Ident` matches a value
/// tagged `Ident` or `Tok::Ident`, so a program can `use` the enum or not.
fn variant_matches(want: &str, got: &str) -> bool {
    want == got || want.rsplit("::").next() == got.rsplit("::").next()
}

fn path_text(p: &Path) -> String {
    p.segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

/// A `Leg` record back out of a value, accepting `Ok(leg)` as well as a bare `leg`.
///
/// Both shapes arrive because the schema writes `debit(..)?` and `credit(..)` — the first is a
/// `Result` and the second is not. Accepting either here is what lets `post` take them in one
/// list, which is how every function in `gbs.niles` is written.
fn as_leg(v: &Value, at: Span) -> Result<Leg, Error> {
    let v = match v {
        Value::Variant { path, payload } if path == "Ok" && payload.len() == 1 => &payload[0],
        other => other,
    };
    let Value::Record { name, fields } = v else {
        return Err(Error::TypeMismatch {
            want: "a leg from `debit` or `credit`".into(),
            got: v.type_name().into(),
            at,
        });
    };
    if name != "Leg" {
        return Err(Error::TypeMismatch {
            want: "a leg from `debit` or `credit`".into(),
            got: name.clone(),
            at,
        });
    }
    let get = |k: &str| -> Result<&Value, Error> {
        fields.get(k).ok_or(Error::NoField {
            name: k.to_string(),
            at,
        })
    };
    let (Value::Str(account), Value::Int(minor), Value::Int(scale), Value::Str(currency)) = (
        get("account")?,
        get("minor")?,
        get("scale")?,
        get("currency")?,
    ) else {
        return Err(Error::TypeMismatch {
            want: "a well-formed leg".into(),
            got: "a leg with the wrong field types".into(),
            at,
        });
    };
    Ok(Leg {
        // The two fields Niles has no form for. See `ledger`'s module docs.
        id: 0,
        narrative: String::new(),
        consumes: None,
        account: (**account).clone(),
        minor: *minor,
        scale: u32::try_from(*scale).unwrap_or(0),
        currency: (**currency).clone(),
        epoch: 0,
        value_date: 0,
    })
}

/// The relational tier and the un-parsed: named, refused, never approximated.
///
/// One function rather than a dozen match arms, so that `Interp::expr`'s frame does not
/// carry a dozen `Error` constructions it will almost never perform.
#[inline(never)]
fn refused(e: &Expr) -> Error {
    let (form, at) = match e {
        Expr::Float(_, s) => ("floating-point literal", *s),
        Expr::Instant { span, .. } => ("temporal literal", *span),
        Expr::Duration { span, .. } => ("duration literal", *span),
        Expr::Hold { span, .. } => ("hold", *span),
        Expr::Resolve { span, .. } => ("resolve", *span),
        Expr::Fx { span, .. } => ("fx", *span),
        Expr::Fixpoint { span, .. } => ("fixpoint", *span),
        Expr::Authorize { span, .. } => ("authorize", *span),
        Expr::Declassify { span, .. } => ("declassify", *span),
        Expr::Explain { span, .. } => ("explain", *span),
        Expr::Reproduce { span, .. } => ("reproduce", *span),
        Expr::Impact { span, .. } => ("impact", *span),
        Expr::Sql { span, .. } => ("sql", *span),
        Expr::Select(s) => ("select", s.span),
        Expr::Error(s) => ("an expression that did not parse", *s),
        other => ("an expression outside the imperative subset", other.span()),
    };
    Error::NotInSubset { form, at }
}

/// The default call-depth ceiling.
///
/// Calibrated against the *widest* frame — a debug build, measured at ~32 KB per Niles
/// call — on the *smallest* stack this code runs on: 1 MB. 24 frames is about 780 KB, so
/// the ceiling is reached, and reported with a span, before the stack is.
///
/// The first value tried here was 64, on the assumption that a spawned thread gets 2 MB.
/// It aborted the test process, which is how the number became measured rather than
/// assumed — and is a small demonstration of why the counter had to exist at all.
///
/// It is deliberately low. A number chosen for the comfortable case would be a number
/// that turns into a process abort on the uncomfortable one, and the whole point of the
/// counter is that the failure mode be a diagnostic. Anything that needs more says so —
/// [`Interp::with_max_depth`] — and arranges the stack to match, via [`run_with_stack`].
pub const DEFAULT_MAX_DEPTH: usize = 24;

/// Run `f` on a thread with a stack sized for `depth` Niles call frames.
///
/// **Why this exists, stated plainly.** A tree-walking interpreter spends host stack in
/// proportion to the interpreted program's call depth, and the cost per frame is a
/// property of the *host build*, not of Niles: measured on this interpreter it is roughly
/// 5 KB per Niles frame in a release build and 20× that in a debug build, because a
/// `match` over thirty expression variants compiles, unoptimised, to a frame holding the
/// union of every arm's locals. That 20× is why `Interp::expr`'s fat arms are behind
/// `#[inline(never)]`, and why this helper exists rather than a comment saying "be
/// careful": a recursive-descent parser nests six interpreter frames per level of
/// expression nesting, so `bootstrap/parser.niles` needs several hundred, and the default
/// 8 MB main-thread stack — 2 MB under `cargo test` — does not provide them.
///
/// rustc does the same thing for the same reason; this is the ordinary shape of the
/// problem, not a peculiarity of Niles.
///
/// [`Value`] is `Rc`-based and therefore not `Send`, so it cannot cross back out of the
/// worker. That is a feature rather than an inconvenience: it forces the caller to say
/// what it wants *out* of the run — a rendering, a number, a verdict — instead of moving
/// an interpreter's heap between threads.
pub fn run_with_stack<T: Send + 'static>(
    depth: usize,
    f: impl FnOnce() -> T + Send + 'static,
) -> std::thread::Result<T> {
    // 128 KB per frame: the measured debug cost with headroom, so the depth ceiling is
    // reached — and reported — before the stack is.
    let bytes = (depth.max(64) * 128 * 1024).max(8 * 1024 * 1024);
    std::thread::Builder::new()
        .stack_size(bytes)
        .spawn(f)
        .expect("a worker thread")
        .join()
}

/// Arithmetic and comparison. Every integer operation is checked.
fn arith(op: BinOp, a: &Value, b: &Value, at: Span) -> Result<Value, Error> {
    use BinOp::*;
    match (op, a, b) {
        (Add, Value::Int(x), Value::Int(y)) => x
            .checked_add(*y)
            .map(Value::Int)
            .ok_or(Error::Overflow { op: "+", at }),
        (Sub, Value::Int(x), Value::Int(y)) => x
            .checked_sub(*y)
            .map(Value::Int)
            .ok_or(Error::Overflow { op: "-", at }),
        (Mul, Value::Int(x), Value::Int(y)) => x
            .checked_mul(*y)
            .map(Value::Int)
            .ok_or(Error::Overflow { op: "*", at }),
        (Div, Value::Int(_), Value::Int(0)) => Err(Error::DivideByZero { at }),
        (Div, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
        (Rem, Value::Int(_), Value::Int(0)) => Err(Error::DivideByZero { at }),
        (Rem, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x % y)),
        (BitAnd, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x & y)),
        (BitOr, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x | y)),
        (BitXor, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x ^ y)),

        (Add, Value::Str(x), Value::Str(y)) => Ok(Value::Str(Rc::new(format!("{x}{y}")))),

        // Money arithmetic is *not* provided. Adding two `Money` values requires the
        // currency check that the type system performs; doing it here would create a
        // second, unchecked path to the same operation, which is the seam §6.9 refuses.
        (Add | Sub, Value::Money { .. }, _) | (Add | Sub, _, Value::Money { .. }) => {
            Err(Error::NotInSubset {
                form: "money arithmetic (checked tier only)",
                at,
            })
        }

        (Eq, _, _) => Ok(Value::Bool(a == b)),
        (Ne, _, _) => Ok(Value::Bool(a != b)),
        (Lt | Le | Gt | Ge, _, _) => {
            let ord = match (a, b) {
                (Value::Int(x), Value::Int(y)) => x.cmp(y),
                (Value::Str(x), Value::Str(y)) => x.cmp(y),
                _ => {
                    return Err(Error::TypeMismatch {
                        want: "two comparable values of the same type".into(),
                        got: format!("{} and {}", a.type_name(), b.type_name()),
                        at,
                    })
                }
            };
            Ok(Value::Bool(match op {
                Lt => ord.is_lt(),
                Le => ord.is_le(),
                Gt => ord.is_gt(),
                _ => ord.is_ge(),
            }))
        }

        (In, x, Value::Array(xs)) => Ok(Value::Bool(xs.contains(x))),
        (NotIn, x, Value::Array(xs)) => Ok(Value::Bool(!xs.contains(x))),

        _ => Err(Error::TypeMismatch {
            want: format!("operands for `{op:?}`"),
            got: format!("{} and {}", a.type_name(), b.type_name()),
            at,
        }),
    }
}

/// Methods on built-in values. The second half of the host shim.
fn builtin_method(recv: &Value, name: &str, args: &[Value], at: Span) -> Result<Value, Error> {
    match (recv, name, args) {
        // --- universal ---
        (v, "len", []) => Ok(Value::Int(value_len(v, at)? as i128)),
        (v, "to_string", []) => Ok(Value::Str(Rc::new(v.render()))),

        // --- strings ---
        (Value::Str(s), "byte_at", [Value::Int(i)]) => {
            let b = s.as_bytes();
            if *i < 0 || *i as usize >= b.len() {
                Err(Error::IndexOutOfBounds {
                    index: *i,
                    len: b.len(),
                    at,
                })
            } else {
                Ok(Value::Int(b[*i as usize] as i128))
            }
        }
        (Value::Str(s), "slice", [Value::Int(a), Value::Int(b)]) => {
            let bytes = s.as_bytes();
            let (lo, hi) = (
                *a.max(&0) as usize,
                (*b).clamp(0, bytes.len() as i128) as usize,
            );
            if lo > hi || hi > bytes.len() {
                return Err(Error::IndexOutOfBounds {
                    index: *b,
                    len: bytes.len(),
                    at,
                });
            }
            Ok(Value::Str(Rc::new(
                String::from_utf8_lossy(&bytes[lo..hi]).into_owned(),
            )))
        }
        (Value::Str(s), "starts_with", [Value::Str(p)]) => Ok(Value::Bool(s.starts_with(&**p))),
        (Value::Str(s), "contains", [Value::Str(p)]) => Ok(Value::Bool(s.contains(&**p))),
        // ASCII-only, deliberately: a locale-dependent case fold would make the lexer's
        // keyword lookup depend on the environment, which Appendix C.4 forbids.
        (Value::Str(s), "to_lower", []) => Ok(Value::Str(Rc::new(s.to_ascii_lowercase()))),
        (Value::Str(s), "to_upper", []) => Ok(Value::Str(Rc::new(s.to_ascii_uppercase()))),
        (Value::Str(s), "bytes", []) => Ok(Value::Bytes(Rc::new(s.as_bytes().to_vec()))),
        (Value::Str(s), "parse_int", []) => match s.trim().parse::<i128>() {
            Ok(i) => Ok(Value::Variant {
                path: "Some".into(),
                payload: Rc::new(vec![Value::Int(i)]),
            }),
            Err(_) => Ok(Value::Variant {
                path: "None".into(),
                payload: Rc::new(vec![]),
            }),
        },

        // --- arrays. Persistent: `push` returns a new array rather than mutating, which
        // is what lets the subset have no aliasing at all. ---
        (Value::Array(xs), "push", [v]) => {
            let mut new = (**xs).clone();
            new.push(v.clone());
            Ok(Value::Array(Rc::new(new)))
        }
        (Value::Array(xs), "get", [Value::Int(i)]) => {
            if *i < 0 || *i as usize >= xs.len() {
                Ok(Value::Variant {
                    path: "None".into(),
                    payload: Rc::new(vec![]),
                })
            } else {
                Ok(Value::Variant {
                    path: "Some".into(),
                    payload: Rc::new(vec![xs[*i as usize].clone()]),
                })
            }
        }
        (Value::Array(xs), "last", []) => match xs.last() {
            Some(v) => Ok(Value::Variant {
                path: "Some".into(),
                payload: Rc::new(vec![v.clone()]),
            }),
            None => Ok(Value::Variant {
                path: "None".into(),
                payload: Rc::new(vec![]),
            }),
        },

        // --- character classification. In the shim rather than in Niles because the
        // alternative is a Niles-side table, and a bootstrap should not begin by
        // transcribing ASCII. ---
        (Value::Int(c), "is_digit", []) => {
            Ok(Value::Bool((b'0' as i128..=b'9' as i128).contains(c)))
        }
        (Value::Int(c), "is_alpha", []) => Ok(Value::Bool(
            (b'a' as i128..=b'z' as i128).contains(c) || (b'A' as i128..=b'Z' as i128).contains(c),
        )),
        (Value::Int(c), "is_ident_start", []) => Ok(Value::Bool(
            (b'a' as i128..=b'z' as i128).contains(c)
                || (b'A' as i128..=b'Z' as i128).contains(c)
                || *c == b'_' as i128,
        )),
        (Value::Int(c), "is_ident_continue", []) => Ok(Value::Bool(
            (b'a' as i128..=b'z' as i128).contains(c)
                || (b'A' as i128..=b'Z' as i128).contains(c)
                || (b'0' as i128..=b'9' as i128).contains(c)
                || *c == b'_' as i128,
        )),
        (Value::Int(c), "is_space", []) => Ok(Value::Bool(matches!(
            *c as u8,
            b' ' | b'\t' | b'\n' | b'\r'
        ))),

        // --- Option/Result ---
        (Value::Variant { path, payload }, "unwrap_or", [d]) => {
            if variant_matches("Some", path) || variant_matches("Ok", path) {
                Ok(payload.first().cloned().unwrap_or_else(|| d.clone()))
            } else {
                Ok(d.clone())
            }
        }
        (Value::Variant { path, .. }, "is_some", []) => {
            Ok(Value::Bool(variant_matches("Some", path)))
        }
        (Value::Variant { path, .. }, "is_none", []) => {
            Ok(Value::Bool(variant_matches("None", path)))
        }

        (r, n, _) => Err(Error::NoField {
            name: format!("{n} on {}", r.type_name()),
            at,
        }),
    }
}

// ── the determinism gate (Appendix C.4, E.18) ────────────────────────────────────────

/// The result of running a program twice and comparing.
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismReport {
    pub runs: usize,
    pub identical: bool,
    /// The output of the first run, for the record.
    pub output: Vec<String>,
    /// The first index at which two runs disagreed, if any.
    pub first_divergence: Option<usize>,
}

/// Run `entry` `runs` times over freshly loaded copies of the same program and check that
/// every run produced byte-identical output.
///
/// This is the runnable form of Appendix C.4's obligation, scoped to what stage 0 can
/// actually check: it establishes *run-to-run* determinism on one target. It does **not**
/// establish cross-target determinism, which needs the same program run on ARM64 and
/// x86-64 and the outputs compared, and which this gate cannot perform from inside one
/// process. §E.19 should say so, and the return type does not pretend otherwise.
pub fn determinism_gate(
    prog: &Program,
    entry: &str,
    runs: usize,
) -> Result<DeterminismReport, Error> {
    determinism_gate_deep(prog, entry, runs, DEFAULT_MAX_DEPTH)
}

/// [`determinism_gate`] with a raised call-depth ceiling, for a program that needs one.
///
/// The caller is responsible for the stack: see [`run_with_stack`]. This exists because
/// the parser's self-check nests far deeper than the lexer's, and a gate that silently
/// used the default ceiling would report `TooDeep` as a determinism failure.
pub fn determinism_gate_deep(
    prog: &Program,
    entry: &str,
    runs: usize,
    max_depth: usize,
) -> Result<DeterminismReport, Error> {
    let mut first: Option<Vec<String>> = None;
    let mut divergence = None;
    for _ in 0..runs {
        let mut it = Interp::new().with_max_depth(max_depth);
        it.load(prog);
        it.call(entry, vec![])?;
        match &first {
            None => first = Some(it.output.clone()),
            Some(f) => {
                if *f != it.output {
                    divergence = Some(
                        f.iter()
                            .zip(&it.output)
                            .position(|(a, b)| a != b)
                            .unwrap_or(f.len().min(it.output.len())),
                    );
                }
            }
        }
    }
    let output = first.unwrap_or_default();
    Ok(DeterminismReport {
        runs,
        identical: divergence.is_none(),
        output,
        first_divergence: divergence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use niles_lang::parser;

    fn run(src: &str, entry: &str) -> Result<Value, Error> {
        let (prog, d) = parser::parse_program(src);
        assert!(!d.has_errors(), "parse errors: {:?}", d.items);
        let mut it = Interp::new();
        it.load(&prog);
        it.call(entry, vec![])
    }

    fn run_out(src: &str, entry: &str) -> Vec<String> {
        let (prog, d) = parser::parse_program(src);
        assert!(!d.has_errors(), "parse errors: {:?}", d.items);
        let mut it = Interp::new();
        it.load(&prog);
        it.call(entry, vec![]).expect("should run");
        it.output
    }

    // ── arithmetic and control flow ─────────────────────────────────────────────────

    #[test]
    fn arithmetic_and_let_work() {
        assert_eq!(
            run(
                "fn main() -> i64 { let x = 2; let y = 3; x * y + 1 }",
                "main"
            ),
            Ok(Value::Int(7))
        );
    }

    #[test]
    fn if_else_selects_a_branch() {
        let src = "fn f(n: i64) -> i64 { if n > 10 { 1 } else { 2 } }
                   fn main() -> i64 { f(20) + f(1) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(3)));
    }

    #[test]
    fn while_loops_and_accumulates() {
        let src = "fn main() -> i64 {
            let mut i = 0;
            let mut total = 0;
            while i < 10 { total = total + i; i = i + 1; }
            total
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(45)));
    }

    #[test]
    fn break_and_continue_do_what_they_say() {
        let src = "fn main() -> i64 {
            let mut i = 0;
            let mut n = 0;
            loop {
                i = i + 1;
                if i > 100 { break; }
                if i % 2 == 0 { continue; }
                n = n + 1;
            }
            n
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(50)));
    }

    #[test]
    fn recursion_works_which_is_what_a_parser_needs() {
        let src = "fn fact(n: i64) -> i64 { if n <= 1 { 1 } else { n * fact(n - 1) } }
                   fn main() -> i64 { fact(10) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(3_628_800)));
    }

    #[test]
    fn early_return_leaves_the_function_not_the_block() {
        let src = "fn f(n: i64) -> i64 { if n == 0 { return 42; } 7 }
                   fn main() -> i64 { f(0) * 100 + f(1) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(4207)));
    }

    // ── data ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn arrays_index_push_and_iterate() {
        let src = "fn main() -> i64 {
            let mut xs = [1, 2, 3];
            xs = xs.push(4);
            let mut total = 0;
            for x in xs { total = total + x; }
            total * 10 + xs[0]
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(101)));
    }

    #[test]
    fn structs_carry_and_update_fields() {
        let src = "struct P { a: i64, b: i64 }
                   fn main() -> i64 {
                       let mut p = P { a: 1, b: 2 };
                       p.a = 10;
                       p.a + p.b
                   }";
        assert_eq!(run(src, "main"), Ok(Value::Int(12)));
    }

    #[test]
    fn enums_construct_and_match_with_payloads() {
        let src = "enum Tok { Ident(str), Num(i64), Eof }
            fn describe(t: Tok) -> i64 {
                match t {
                    Tok::Num(n) => n,
                    Tok::Ident(s) => len(s),
                    Tok::Eof => 0,
                }
            }
            fn main() -> i64 { describe(Tok::Num(5)) * 100 + describe(Tok::Ident(\"abcd\")) * 10 + describe(Tok::Eof) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(540)));
    }

    #[test]
    fn match_guards_are_honoured_and_order_matters() {
        let src = "fn f(n: i64) -> i64 {
            match n {
                x if x < 0 => 0,
                0 => 1,
                x => x * 2,
            }
        }
        fn main() -> i64 { f(-5) * 100 + f(0) * 10 + f(3) }";
        assert_eq!(run(src, "main"), Ok(Value::Int(16)));
    }

    #[test]
    fn closures_capture_by_value_at_creation() {
        // A snapshot capture, stated in the type docs. If this ever became by-reference,
        // the interpreter would acquire aliasing the language does not have.
        let src = "fn main() -> i64 {
            let mut n = 1;
            let f = |x| x + n;
            n = 100;
            f(5)
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(6)));
    }

    // ── strings and the lexer primitives ─────────────────────────────────────────────

    #[test]
    fn string_primitives_are_enough_to_scan_text() {
        let src = "fn main() -> i64 {
            let s = \"let x = 42\";
            let mut i = 0;
            let mut digits = 0;
            while i < len(s) {
                let c = s.byte_at(i);
                if c.is_digit() { digits = digits + 1; }
                i = i + 1;
            }
            digits
        }";
        assert_eq!(run(src, "main"), Ok(Value::Int(2)));
    }

    #[test]
    fn slice_and_concatenation_build_tokens() {
        let src = "fn main() -> str { let s = \"abcdef\"; s.slice(1, 3) + \"|\" + s.slice(4, 6) }";
        assert_eq!(run(src, "main"), Ok(Value::Str(Rc::new("bc|ef".into()))));
    }

    // ── the refusals ────────────────────────────────────────────────────────────────

    /// `txn` used to be refused by name, and the reason it was is now the reason it is not.
    ///
    /// The old test read: *"If `txn` evaluated as a block, a Niles-written program could appear
    /// to conserve money while nothing checked it."* That was the right worry and refusing the
    /// form was the wrong answer to it, because a form nobody can execute is a form nobody can
    /// compare against a second implementation — which is exactly how a conformance suite ends
    /// up comparing renderings instead of documents (F-25).
    ///
    /// `txn` now evaluates, and the worry is answered by the seal: a set that does not conserve
    /// is refused, by currency, with the residual named. The test asserts the property rather
    /// than the refusal of the syntax.
    #[test]
    fn a_txn_that_does_not_conserve_is_refused_with_the_currency_and_the_residual() {
        let src = "fn main() -> i64 { txn idem(\"t\") { post(debit(acct(1), 10.00 usd)?, \
                   credit(acct(2), 9.00 usd)) }; 0 }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        match it.call("main", vec![]) {
            Err(Error::Refused {
                why: ledger::Refusal::Unbalanced { currency, residual },
                ..
            }) => {
                assert_eq!(currency, "USD", "normalised, as the kernel normalises it");
                assert_eq!(residual, -100, "one dollar short, in minor units");
            }
            other => panic!("an unbalanced set must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_txn_that_conserves_seals_and_the_legs_are_in_the_order_written() {
        let src = "fn main() -> i64 { txn idem(\"pay\") { post(debit(acct(1), 10.00 usd)?, \
                   credit(acct(2), 10.00 usd)) }; 0 }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        assert_eq!(it.call("main", vec![]), Ok(Value::Int(0)));
        assert_eq!(it.ledger.sealed.len(), 1);
        let set = &it.ledger.sealed[0];
        assert_eq!(set.txn, "pay");
        assert_eq!(set.legs.len(), 2);
        assert_eq!(set.legs[0].account, "1");
        assert_eq!(set.legs[0].minor, -1000, "a debit is negative");
        assert_eq!(set.legs[1].minor, 1000, "a credit is positive");
        assert_eq!(set.legs[0].currency, "USD");
    }

    /// Two spellings of one currency are one currency, here as in the kernel.
    ///
    /// A schema writes `10.00 usd` — the lexer's money suffix is lower case — and a Rust
    /// product writes `Currency::new("USD")`, which upper-cases. Without the same
    /// normalisation on both sides, a set posted from a `Money` argument spelled `usd` and a
    /// literal spelled `USD` would carry two currencies, each summing to zero on its own: a
    /// transaction that conserves twice and moves money once.
    #[test]
    fn two_spellings_of_one_currency_are_one_currency() {
        let src = "fn main() -> i64 { txn idem(\"mixed\") { post(debit(acct(1), amt)?, \
                   credit(acct(2), 10.00 usd)) }; 0 }"
            .replace("amt", "10.00 usd");
        let (prog, _) = parser::parse_program(&src);
        let mut it = Interp::new();
        it.load(&prog);
        assert_eq!(it.call("main", vec![]), Ok(Value::Int(0)));
        let set = &it.ledger.sealed[0];
        assert_eq!(set.legs[0].currency, "USD");
        assert_eq!(set.legs[1].currency, "USD");
    }

    /// And the same rule reached through an *argument*, which is the path a conformance
    /// fixture actually takes: `nilesc run --args` supplies `money 1000 usd 2`, and the leg it
    /// produces must carry the code the kernel would.
    #[test]
    fn a_money_argument_is_normalised_like_a_literal() {
        let src = "fn f(m: Money<usd>) -> i64 { txn idem(\"a\") { post(debit(acct(1), m)?, \
                   credit(acct(2), m)) }; 0 }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        let arg = Value::Money {
            minor: 1_000,
            scale: 2,
            currency: "usd".into(),
        };
        assert_eq!(it.call("f", vec![arg]), Ok(Value::Int(0)));
        assert_eq!(it.ledger.sealed[0].legs[0].currency, "USD");
    }

    /// The rest of the relational tier is still refused by name, and this is where that is
    /// checked — so evaluating `txn` did not quietly open the others.
    #[test]
    fn the_rest_of_the_relational_tier_is_refused_by_name_and_never_approximated() {
        for (src, want) in [
            (
                "fn main() -> i64 { let h = hold(acct(1), 20.00 usd, expires: 7.days)?; 0 }",
                "hold",
            ),
            ("fn main() -> i64 { let x = resolve h void; 0 }", "resolve"),
            (
                "fn main() -> i64 { let x = 1.5; 0 }",
                "floating-point literal",
            ),
        ] {
            let (prog, _) = parser::parse_program(src);
            let mut it = Interp::new();
            it.load(&prog);
            match it.call("main", vec![]) {
                Err(Error::NotInSubset { form, .. }) => assert_eq!(form, want, "for {src}"),
                other => panic!("`{want}` must be refused by name, got {other:?}"),
            }
        }
    }

    #[test]
    fn money_arithmetic_is_refused_because_the_check_lives_in_the_type_system() {
        let src = "fn main() -> i64 { let a = 10.00 usd; let b = 5.00 eur; a + b }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        assert!(matches!(
            it.call("main", vec![]),
            Err(Error::NotInSubset {
                form: "money arithmetic (checked tier only)",
                ..
            })
        ));
    }

    #[test]
    fn floats_have_no_spelling_in_the_subset() {
        // Cross-target determinism and money-is-not-a-float are the same commitment.
        let src = "fn main() -> i64 { let x = 1.5; 0 }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new();
        it.load(&prog);
        assert!(matches!(
            it.call("main", vec![]),
            Err(Error::NotInSubset {
                form: "floating-point literal",
                ..
            })
        ));
    }

    #[test]
    fn overflow_is_an_error_and_never_a_wrap() {
        let src = "fn main() -> i64 { let big = 170141183460469231731687303715884105727; big + 1 }";
        assert!(matches!(
            run(src, "main"),
            Err(Error::Overflow { op: "+", .. })
        ));
    }

    #[test]
    fn division_by_zero_is_an_error_with_a_span() {
        let src = "fn main() -> i64 { let z = 0; 1 / z }";
        let e = run(src, "main").unwrap_err();
        assert!(matches!(e, Error::DivideByZero { .. }));
        assert!(e.span().is_some(), "a stage-0 failure must be locatable");
    }

    #[test]
    fn an_out_of_bounds_index_names_the_index_and_the_length() {
        let src = "fn main() -> i64 { let xs = [1, 2]; xs[5] }";
        let e = run(src, "main").unwrap_err();
        assert!(matches!(
            e,
            Error::IndexOutOfBounds {
                index: 5,
                len: 2,
                ..
            }
        ));
        // And the span must point at the indexing expression, not at the function. We
        // check the text it covers rather than its offsets, so the assertion survives an
        // edit to the string above.
        let sp = e.span().expect("locatable");
        assert_eq!(&src[sp.start as usize..sp.end as usize], "xs[5]");
    }

    #[test]
    fn an_unbound_name_is_reported_rather_than_defaulted() {
        assert!(matches!(
            run("fn main() -> i64 { nope }", "main"),
            Err(Error::Unbound { .. })
        ));
    }

    #[test]
    fn a_non_boolean_condition_is_rejected_rather_than_coerced() {
        let src = "fn main() -> i64 { if 1 { 2 } else { 3 } }";
        assert!(matches!(run(src, "main"), Err(Error::TypeMismatch { .. })));
    }

    #[test]
    fn an_infinite_loop_runs_out_of_fuel_instead_of_hanging_the_suite() {
        let src = "fn main() -> i64 { loop { } }";
        let (prog, _) = parser::parse_program(src);
        let mut it = Interp::new().with_fuel(1000);
        it.load(&prog);
        assert_eq!(it.call("main", vec![]), Err(Error::OutOfFuel));
    }

    // ── determinism ──────────────────────────────────────────────────────────────────

    #[test]
    fn evaluation_order_is_left_to_right_and_observable() {
        let src = "fn a() -> i64 { print(\"a\"); 1 }
                   fn b() -> i64 { print(\"b\"); 2 }
                   fn main() -> i64 { a() + b() }";
        assert_eq!(run_out(src, "main"), vec!["a", "b"]);
    }

    #[test]
    fn the_short_circuit_really_short_circuits() {
        // Not a nicety: the Niles lexer's bounds checks depend on it.
        let src = "fn t() -> bool { print(\"evaluated\"); true }
                   fn main() -> i64 { if false && t() { 1 } else { 0 } }";
        assert_eq!(run_out(src, "main"), Vec::<String>::new());
    }

    #[test]
    fn the_determinism_gate_passes_on_a_deterministic_program() {
        let src = "fn main() -> i64 {
            let mut i = 0;
            while i < 5 { print(i.to_string()); i = i + 1; }
            0
        }";
        let (prog, _) = parser::parse_program(src);
        let r = determinism_gate(&prog, "main", 8).unwrap();
        assert!(r.identical);
        assert_eq!(r.runs, 8);
        assert_eq!(r.output, vec!["0", "1", "2", "3", "4"]);
        assert_eq!(r.first_divergence, None);
    }

    #[test]
    fn scope_iteration_is_by_name_so_a_hash_seed_cannot_leak_into_output() {
        // The concrete determinism hazard this design closes. Were `Env` a `HashMap`,
        // nothing in the language would change, and yet a struct's rendering could
        // reorder between processes.
        let src = "struct S { zeta: i64, alpha: i64, mu: i64 }
                   fn main() -> i64 { print(S { zeta: 1, alpha: 2, mu: 3 }.to_string()); 0 }";
        let out = run_out(src, "main");
        assert_eq!(out, vec!["S { alpha: 2, mu: 3, zeta: 1 }"]);
    }

    #[test]
    fn the_gate_reports_the_number_of_runs_it_actually_did() {
        let src = "fn main() -> i64 { print(\"x\"); 0 }";
        let (prog, _) = parser::parse_program(src);
        assert_eq!(determinism_gate(&prog, "main", 3).unwrap().runs, 3);
    }

    #[test]
    fn a_failure_inside_the_gate_surfaces_rather_than_being_counted_as_identical() {
        let src = "fn main() -> i64 { let z = 0; 1 / z }";
        let (prog, _) = parser::parse_program(src);
        assert!(determinism_gate(&prog, "main", 2).is_err());
    }
    // ── the call-depth ceiling ───────────────────────────────────────────────────────

    fn deep_program(depth: i64) -> Program {
        let src = format!(
            "fn down(n: i64) -> i64 {{ if n == 0 {{ 0 }} else {{ 1 + down(n - 1) }} }}\n\
             fn main() -> i64 {{ down({depth}) }}"
        );
        parser::parse_program(&src).0
    }

    #[test]
    fn deep_recursion_is_an_error_and_not_a_crash() {
        // The point of the counter. Before it, this input aborted the *process* on a
        // stack overflow — no diagnostic, no span, no exit status a test could read.
        let prog = deep_program(10_000);
        let mut it = Interp::new();
        it.load(&prog);
        match it.call("main", vec![]) {
            Err(Error::TooDeep { limit, .. }) => assert_eq!(limit, DEFAULT_MAX_DEPTH),
            other => panic!("expected a depth error, got {other:?}"),
        }
    }

    #[test]
    fn the_ceiling_is_not_so_low_that_ordinary_recursion_trips_it() {
        // A guard that fired on reasonable programs would be a worse defect than the one
        // it prevents. Half the default must be comfortably reachable.
        let prog = deep_program((DEFAULT_MAX_DEPTH as i64) / 2);
        let mut it = Interp::new();
        it.load(&prog);
        assert_eq!(
            it.call("main", vec![]),
            Ok(Value::Int((DEFAULT_MAX_DEPTH as i128) / 2))
        );
    }

    #[test]
    fn a_raised_ceiling_needs_a_stack_to_match_and_run_with_stack_provides_one() {
        // The pairing the two APIs exist to enforce: `with_max_depth` alone would move the
        // abort rather than remove it. 600 frames is ~19 MB of debug-build stack, well past
        // any default, and the run must still produce the right answer rather than dying.
        let r = run_with_stack(1200, || {
            let prog = deep_program(600);
            let mut it = Interp::new().with_max_depth(1200);
            it.load(&prog);
            // `Value` is not `Send`, so the answer crosses back as a rendering.
            it.call("main", vec![])
                .map(|v| v.render())
                .map_err(|e| e.message())
        })
        .expect("the worker must not crash");
        assert_eq!(r, Ok("600".to_string()));
    }

    #[test]
    fn the_depth_counter_unwinds_so_repeated_calls_do_not_accumulate() {
        // An `enter` without a matching decrement would make the tenth call fail where the
        // first succeeded — the classic shape of a leaked counter.
        let prog = deep_program(8);
        let mut it = Interp::new();
        it.load(&prog);
        for _ in 0..50 {
            assert_eq!(it.call("main", vec![]), Ok(Value::Int(8)));
        }
    }
}
