//! **An S-expression rendering of the surface tree.**
//!
//! This exists for one reason: the stage-1 equivalence gate for the parser. The Rust
//! parser and `bootstrap/parser.niles` share no types — one builds `ast::Expr`, the other
//! builds a Niles `enum E` inside the stage-0 interpreter — so the two cannot be compared
//! structurally without first writing an adapter, and an adapter is exactly the thing a
//! gate must not be allowed to be. They can be compared on a *rendering*, provided both
//! sides render independently and the format is tight enough that two different trees
//! cannot produce the same text.
//!
//! Three rules make the format unambiguous, and both implementations obey them:
//!
//! 1. **Every node is a parenthesised list whose head is a tag.** No bare atoms except in
//!    the argument positions the tag fixes.
//! 2. **Optional slots are never elided.** An absent `else`, an absent return type, an
//!    absent initialiser all render as `(none)`. Eliding them would let `(let p A)` mean
//!    either "no type" or "no initialiser", and the two parsers could then disagree while
//!    producing the same string.
//! 3. **Variable-length runs are wrapped.** `(params …)`, `(args …)`, `(arms …)`,
//!    `(fields …)`, `(stmts …)` — so a list never runs into a following fixed slot.
//!
//! Spans are deliberately *not* rendered. Span agreement is the *lexer's* gate, which
//! already compares every token's offsets; repeating it here would make a parser
//! disagreement look like a lexer disagreement, and would couple the parser gate to
//! trivia handling it has no opinion about.
//!
//! # Totality, and what `(unsupported …)` is for
//!
//! `bootstrap/parser.niles` implements the imperative subset the stage-0 interpreter can
//! execute. It does not implement `txn`, `fx`, `hold`, the SQL surface, schemas or views.
//! This renderer is nevertheless *total* over the AST: an out-of-scope form renders as
//! `(unsupported <form>)`. That is what lets the gate detect an out-of-scope input and
//! exclude it **visibly**, by the same discipline `in_scope` uses for the lexer, rather
//! than either crashing or quietly passing. [`covers`] is the predicate.

use crate::ast::*;
use crate::lexer::TimeUnit;

/// Render a whole program.
pub fn program(p: &Program) -> String {
    let mut s = String::from("(program");
    for i in &p.items {
        s.push(' ');
        s.push_str(&item(i));
    }
    s.push(')');
    s
}

/// Whether a rendering is entirely inside the subset `bootstrap/parser.niles` implements.
///
/// A gate that compared renderings containing `(unsupported …)` would be comparing two
/// admissions of defeat and calling them equal.
pub fn covers(rendered: &str) -> bool {
    !rendered.contains("(unsupported ")
}

fn unsupported(what: &str) -> String {
    format!("(unsupported {what})")
}

// ── names, paths, atoms ──────────────────────────────────────────────────────────────

fn name(n: &Name) -> String {
    n.text.clone()
}

fn path(p: &Path) -> String {
    p.segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("::")
}

/// The six escapes the lexer decodes, run backwards. Every other byte goes through as it
/// is, so the rendering is not a valid Niles string literal in general — it does not need
/// to be, it needs to be injective, which these five rules make it.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn opt<T>(v: Option<&T>, f: impl Fn(&T) -> String) -> String {
    match v {
        Some(x) => f(x),
        None => "(none)".into(),
    }
}

// ── items ────────────────────────────────────────────────────────────────────────────

fn vis(public: bool) -> &'static str {
    if public {
        "pub"
    } else {
        "priv"
    }
}

pub fn item(i: &Item) -> String {
    match i {
        Item::Fn(f) if !f.attrs.is_empty() => unsupported("attrs"),
        Item::Fn(f) => fn_decl(f),
        Item::Struct(d) if !d.attrs.is_empty() => unsupported("attrs"),
        Item::Struct(d) => format!(
            "(struct {} {} (generics{}) (fields{}))",
            vis(d.public),
            name(&d.name),
            d.generics.iter().map(|g| format!(" {}", name(g))).collect::<String>(),
            d.fields.iter().map(|f| format!(" {}", field_decl(f))).collect::<String>(),
        ),
        Item::Enum(d) => format!(
            "(enum {} {} (generics{}) (variants{}))",
            vis(d.public),
            name(&d.name),
            d.generics.iter().map(|g| format!(" {}", name(g))).collect::<String>(),
            d.variants
                .iter()
                .map(|(n, tys)| format!(
                    " (variant {}{})",
                    name(n),
                    tys.iter().map(|t| format!(" {}", ty(t))).collect::<String>()
                ))
                .collect::<String>(),
        ),
        Item::Use { path: p, .. } => format!("(use {})", path(p)),
        Item::Error(_) => "(ierr)".into(),
        Item::Schema(_) => unsupported("schema"),
        Item::Trait(_) => unsupported("trait"),
        Item::Impl(_) => unsupported("impl"),
        Item::Mod { .. } => unsupported("mod"),
        Item::Const { .. } => unsupported("const"),
        Item::TypeAlias { .. } => unsupported("type-alias"),
        Item::Capability { .. } => unsupported("capability"),
        Item::View(_) => unsupported("view"),
    }
}

fn fn_decl(f: &FnDecl) -> String {
    format!(
        "(fn {} {} (generics{}) (params{}) {} {} {})",
        vis(f.public),
        name(&f.name),
        f.generics.iter().map(|g| format!(" {}", name(g))).collect::<String>(),
        f.params
            .iter()
            .map(|p| format!(" (param {} {})", pat(&p.pat), ty(&p.ty)))
            .collect::<String>(),
        opt(f.ret.as_ref(), ty),
        opt(f.effects.as_ref(), effect_row),
        opt(f.body.as_ref(), block),
    )
}

/// A struct field is a name and a type. The other things a [`FieldDecl`] can carry —
/// `primary key`, `unique`, `default`, `check`, attributes — belong to relation fields,
/// which are not in the subset; rendering them as an ordinary field would let a
/// difference the Niles parser cannot see pass as agreement.
fn field_decl(f: &FieldDecl) -> String {
    if f.primary_key || f.unique || f.default.is_some() || f.check.is_some() || !f.attrs.is_empty() {
        return unsupported("field-modifier");
    }
    format!("(field {} {})", name(&f.name), ty(&f.ty))
}

fn effect_row(r: &EffectRow) -> String {
    format!(
        "(effects{})",
        r.effects
            .iter()
            .map(|e| format!(
                " (eff {} {} (args{}))",
                name(&e.name),
                opt(e.at.as_ref(), name),
                e.args.iter().map(|a| format!(" {}", name(a))).collect::<String>()
            ))
            .collect::<String>()
    )
}

// ── types ────────────────────────────────────────────────────────────────────────────

pub fn ty(t: &Ty) -> String {
    match t {
        Ty::Path { path: p, args, .. } => format!(
            "(tp {}{})",
            path(p),
            args.iter().map(|a| format!(" {}", ty(a))).collect::<String>()
        ),
        Ty::Ref { inner, mutable, .. } => {
            format!("(tref {} {})", if *mutable { "mut" } else { "imm" }, ty(inner))
        }
        Ty::Tuple { elems, .. } => format!(
            "(ttuple{})",
            elems.iter().map(|e| format!(" {}", ty(e))).collect::<String>()
        ),
        Ty::Slice { elem, .. } => format!("(tslice {})", ty(elem)),
        Ty::Array { elem, len, .. } => format!("(tarray {} {})", ty(elem), expr(len)),
        Ty::Fn { params, ret, effects, .. } => format!(
            "(tfn (params{}) {} {})",
            params.iter().map(|p| format!(" {}", ty(p))).collect::<String>(),
            ty(ret),
            effect_row(effects)
        ),
        Ty::Dyn { path: p, .. } => format!("(tdyn {})", path(p)),
        Ty::Unit(_) => "(tunit)".into(),
        Ty::Infer(_) => "(tinfer)".into(),
        Ty::Error(_) => "(terr)".into(),
    }
}

// ── blocks, statements, patterns ─────────────────────────────────────────────────────

pub fn block(b: &Block) -> String {
    format!(
        "(block (stmts{}) {})",
        b.stmts.iter().map(|s| format!(" {}", stmt(s))).collect::<String>(),
        opt(b.tail.as_deref(), expr),
    )
}

fn stmt(s: &Stmt) -> String {
    match s {
        Stmt::Let { pat: p, ty: t, init, .. } => {
            format!("(let {} {} {})", pat(p), opt(t.as_ref(), ty), opt(init.as_ref(), expr))
        }
        Stmt::Expr(e) => format!("(expr {})", expr(e)),
        Stmt::Semi(e) => format!("(semi {})", expr(e)),
        Stmt::Item(i) => format!("(item {})", item(i)),
        Stmt::Dml(_) => unsupported("dml"),
        Stmt::Error(_) => "(serr)".into(),
    }
}

pub fn pat(p: &Pat) -> String {
    match p {
        Pat::Wild(_) => "(pwild)".into(),
        Pat::Bind { name: n, mutable, by_ref, .. } => format!(
            "(pbind {} {} {})",
            name(n),
            if *mutable { "mut" } else { "imm" },
            if *by_ref { "ref" } else { "val" }
        ),
        Pat::Tuple { elems, .. } => format!(
            "(ptuple{})",
            elems.iter().map(|e| format!(" {}", pat(e))).collect::<String>()
        ),
        Pat::TupleStruct { path: pa, elems, .. } => format!(
            "(pctor {}{})",
            path(pa),
            elems.iter().map(|e| format!(" {}", pat(e))).collect::<String>()
        ),
        Pat::Struct { path: pa, fields, rest, .. } => format!(
            "(pstruct {} (fields{}) {})",
            path(pa),
            fields
                .iter()
                .map(|(n, sub)| format!(" (pf {} {})", name(n), pat(sub)))
                .collect::<String>(),
            if *rest { "rest" } else { "norest" }
        ),
        Pat::Lit(e) => format!("(plit {})", expr(e)),
        Pat::Path(pa) => format!("(ppath {})", path(pa)),
        Pat::Error(_) => "(perr)".into(),
    }
}

// ── expressions ──────────────────────────────────────────────────────────────────────

fn arg(a: &Arg) -> String {
    format!("(arg {} {})", opt(a.name.as_ref(), |n| format!("(n {})", name(n))), expr(&a.value))
}

fn args(v: &[Arg]) -> String {
    format!("(args{})", v.iter().map(|a| format!(" {}", arg(a))).collect::<String>())
}

fn unop(o: UnOp) -> &'static str {
    match o {
        UnOp::Neg => "neg",
        UnOp::Not => "not",
        UnOp::Deref => "deref",
        UnOp::Ref => "ref",
        UnOp::RefMut => "refmut",
    }
}

/// The operator names the two implementations share. Deliberately *not* the source
/// spelling: `and` and `&&` are the same operator, and a rendering that kept them apart
/// would make the two parsers disagree about a program they both parsed correctly.
pub fn binop(o: BinOp) -> &'static str {
    use BinOp::*;
    match o {
        Add => "add",
        Sub => "sub",
        Mul => "mul",
        Div => "div",
        Rem => "rem",
        Eq => "eq",
        Ne => "ne",
        Lt => "lt",
        Le => "le",
        Gt => "gt",
        Ge => "ge",
        And => "and",
        Or => "or",
        BitAnd => "bitand",
        BitOr => "bitor",
        BitXor => "bitxor",
        Is => "is",
        IsNot => "isnot",
        In => "in",
        NotIn => "notin",
        Like => "like",
        Between => "between",
    }
}

pub fn stage_name(k: StageKind) -> &'static str {
    use StageKind::*;
    match k {
        Where => "where",
        Map => "map",
        Select => "select",
        GroupBy => "group_by",
        Having => "having",
        Join => "join",
        LeftJoin => "left_join",
        RightJoin => "right_join",
        FullOuterJoin => "full_outer_join",
        CrossJoin => "cross_join",
        Union => "union",
        UnionAll => "union_all",
        Except => "except",
        Intersect => "intersect",
        Distinct => "distinct",
        DistinctBy => "distinct_by",
        OrderBy => "order_by",
        Limit => "limit",
        Offset => "offset",
        Sum => "sum",
        Count => "count",
        Min => "min",
        Max => "max",
        Avg => "avg",
        Fold => "fold",
        Fixpoint => "fixpoint",
        AsOf => "as_of",
        ValidAt => "valid_at",
        Get => "get",
        Range => "range",
        Unknown => "unknown",
    }
}

fn unit_name(u: TimeUnit) -> String {
    format!("{u:?}").to_lowercase()
}

pub fn expr(e: &Expr) -> String {
    use Expr::*;
    match e {
        Int(v, _) => format!("(int {v})"),
        Bool(v, _) => format!("(bool {v})"),
        Str(s, _) => format!("(str {})", quote(s)),
        Unit(_) => "(unit)".into(),
        Money { minor, scale, currency, .. } => {
            format!("(money {minor} {scale} {})", name(currency))
        }
        Epoch(v, _) => format!("(epoch {v})"),
        Duration { value, unit, .. } => format!("(dur {value} {})", unit_name(*unit)),
        Path(p) => format!("(path {})", path(p)),

        Tuple { elems, .. } => format!(
            "(tuple{})",
            elems.iter().map(|x| format!(" {}", expr(x))).collect::<String>()
        ),
        Array { elems, .. } => format!(
            "(array{})",
            elems.iter().map(|x| format!(" {}", expr(x))).collect::<String>()
        ),
        StructLit { path: p, fields, .. } => format!(
            "(structlit {} (fields{}))",
            path(p),
            fields
                .iter()
                .map(|(n, v)| format!(" (sf {} {})", name(n), expr(v)))
                .collect::<String>()
        ),
        Field { base, name: n, .. } => format!("(field {} {})", expr(base), name(n)),
        Index { base, index, .. } => format!("(index {} {})", expr(base), expr(index)),
        Call { callee, args: a, .. } => format!("(call {} {})", expr(callee), args(a)),
        Stage { recv, kind, name: n, args: a, .. } => {
            format!("(stage {} {} {} {})", expr(recv), stage_name(*kind), name(n), args(a))
        }
        Closure { params, body, is_move, .. } => format!(
            "(closure {} (params{}) {})",
            if *is_move { "move" } else { "nomove" },
            params
                .iter()
                .map(|(p, t)| format!(" (cp {} {})", pat(p), opt(t.as_ref(), ty)))
                .collect::<String>(),
            expr(body)
        ),
        Unary { op, operand, .. } => format!("(unary {} {})", unop(*op), expr(operand)),
        Binary { op, lhs, rhs, .. } => {
            format!("(binary {} {} {})", binop(*op), expr(lhs), expr(rhs))
        }
        Assign { target, value, .. } => format!("(assign {} {})", expr(target), expr(value)),
        Cast { expr: inner, ty: t, .. } => format!("(cast {} {})", expr(inner), ty(t)),
        Try { expr: inner, .. } => format!("(try {})", expr(inner)),

        Block(b) => block(b),
        If { cond, then, els, .. } => {
            format!("(if {} {} {})", expr(cond), block(then), opt(els.as_deref(), expr))
        }
        Match { scrutinee, arms, .. } => format!(
            "(match {} (arms{}))",
            expr(scrutinee),
            arms.iter()
                .map(|a| format!(
                    " (arm {} {} {})",
                    pat(&a.pat),
                    opt(a.guard.as_ref(), expr),
                    expr(&a.body)
                ))
                .collect::<String>()
        ),
        While { cond, body, .. } => format!("(while {} {})", expr(cond), block(body)),
        Loop { body, .. } => format!("(loop {})", block(body)),
        For { pat: p, iter, body, .. } => {
            format!("(for {} {} {})", pat(p), expr(iter), block(body))
        }
        Return { value, .. } => format!("(return {})", opt(value.as_deref(), expr)),
        Break(_) => "(break)".into(),
        Continue(_) => "(continue)".into(),
        Error(_) => "(eerr)".into(),

        // Out of the subset. Total, and visibly so.
        Float(..) => unsupported("float"),
        Bytes(..) => unsupported("bytes"),
        Instant { .. } => unsupported("instant"),
        Case { .. } => unsupported("case"),
        Txn { .. } => unsupported("txn"),
        Hold { .. } => unsupported("hold"),
        Resolve { .. } => unsupported("resolve"),
        Fx { .. } => unsupported("fx"),
        Fixpoint { .. } => unsupported("fixpoint"),
        Authorize { .. } => unsupported("authorize"),
        Declassify { .. } => unsupported("declassify"),
        Explain { .. } => unsupported("explain"),
        Reproduce { .. } => unsupported("reproduce"),
        Impact { .. } => unsupported("impact"),
        Sql { .. } => unsupported("sql"),
        Select(_) => unsupported("select"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn r(src: &str) -> String {
        let (p, _) = parser::parse_program(src);
        program(&p)
    }

    #[test]
    fn optional_slots_are_never_elided() {
        // The rule that keeps `(let p A)` from being ambiguous between "no type" and
        // "no initialiser". Both spellings must be three-slot.
        let no_ty = r("fn f() { let x = 1; }");
        let no_init = r("fn f() { let x: i64; }");
        assert!(no_ty.contains("(let (pbind x imm val) (none) (int 1))"), "{no_ty}");
        assert!(no_init.contains("(let (pbind x imm val) (tp i64) (none))"), "{no_init}");
    }

    #[test]
    fn precedence_is_visible_in_the_shape() {
        // The whole point of comparing trees rather than token streams: `a - b - c` must
        // associate left, and the rendering must show it.
        let s = r("fn f() { a - b - c }");
        assert!(
            s.contains("(binary sub (binary sub (path a) (path b)) (path c))"),
            "{s}"
        );
        let m = r("fn f() { a + b * c }");
        assert!(m.contains("(binary add (path a) (binary mul (path b) (path c)))"), "{m}");
    }

    #[test]
    fn the_two_spellings_of_one_operator_render_identically() {
        // `and` and `&&` are the same operator. A rendering that distinguished them would
        // manufacture a disagreement between two parsers that both got it right.
        assert_eq!(r("fn f() { a && b }"), r("fn f() { a and b }"));
        assert_eq!(r("fn f() { a || b }"), r("fn f() { a or b }"));
    }

    #[test]
    fn strings_round_trip_through_the_escape_rules() {
        let s = r("fn f() { \"a\\\"b\\\\c\\nd\" }");
        assert!(s.contains(r#"(str "a\"b\\c\nd")"#), "{s}");
    }

    #[test]
    fn covers_detects_an_out_of_scope_form() {
        // The exclusion predicate has to actually exclude, or the gate proves nothing.
        assert!(covers(&r("fn f() -> i64 { 1 + 2 }")));
        assert!(!covers(&r("fn f() { txn { post(a, b) } }")));
        assert!(!covers(&r("fn f() { 1.5 }")));
        assert!(!covers(&r("schema s { }")));
    }

    #[test]
    fn every_expression_variant_renders_to_something() {
        // Totality. A renderer that panicked on an unexpected node would turn an
        // out-of-scope input into a test failure instead of an exclusion.
        for src in [
            "fn f() { let (a, b) = t; }",
            "fn f() { match x { Some(y) if y > 1 => y, _ => 0 } }",
            "fn f() { xs[0].len() }",
            "fn f() { |x: i64| x + 1 }",
            "fn f() { for x in xs { g(x); } }",
            "fn f() { loop { break; } }",
            "fn f() { return; }",
            "fn f() { x as i64 }",
            "fn f() { g()? }",
            "fn f() { S { a: 1, b } }",
            "fn f() { #4200 }",
            "fn f() { 10.00 usd }",
            "fn f() { q |> where(p) }",
            "struct S<T> { a: i64, b: [T] }",
            "enum E { A(i64, str), B }",
            "pub fn g(x: &mut i64) -> Result<i64, E> ! { append, debit<usd> } { 0 }",
        ] {
            let out = r(src);
            assert!(out.starts_with("(program "), "{src} -> {out}");
            assert!(out.ends_with(')'));
        }
    }
}
