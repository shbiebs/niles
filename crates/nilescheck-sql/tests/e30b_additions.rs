//! **The three SQL+C+L additions of E30b′, each held in both directions** (cycle 15, C15-05;
//! `docs/study/E30b-design.md` §5). Every rule refuses its defect and accepts the neighbour
//! beside it, so a rule that refused everything would pass only half of its tests.
//!
//! The programs here are written for these tests. None is from the study's corpus
//! (`crates/syntax-study/corpus/e30b/`), which the additions were built without reading.

fn schema(which: &str) -> String {
    let rel = match which {
        "r1" => "../syntax-study/corpus/schema/schema.sql",
        "r2" => "../syntax-study/corpus/e30b/schema/r2.sql",
        other => panic!("no schema {other}"),
    };
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).unwrap()
}

/// Every error code the full checker reports for the program, after the schema.
fn errors(which: &str, program: &str) -> Vec<&'static str> {
    all(which, program, true)
}

fn all(which: &str, program: &str, errors_only: bool) -> Vec<&'static str> {
    let src = format!("{}\n{program}", schema(which));
    let (stmts, _) =
        nilescheck_sql::parse(&src).unwrap_or_else(|e| panic!("{}: {program}", e.msg));
    let mut v: Vec<&'static str> = nilescheck_sql::check_all(&stmts)
        .into_iter()
        .filter(|d| d.error || !errors_only)
        .map(|d| d.code)
        .collect();
    v.sort();
    v.dedup();
    v
}

const HEAD: &str = "declare\n    t bigint := (select coalesce(max(txn), 0) + 1 from postings);\n    e bigint := (select coalesce(max(epoch), 0) + 1 from postings);\nbegin\n";

fn transfer_r1(cur: &str, ann: &str) -> String {
    format!(
        "create function pay(a bigint, b bigint, m {cur}) returns void language plpgsql as $$\n{HEAD}    insert into postings (txn, acct, cur, amt_{cur}, epoch, value_date) values\n        (t, a, '{cur}', row(-(m).minor)::{cur}, e, current_date),\n        (t, b, '{cur}', m, e, current_date);\nend $$;\ncomment on function pay(bigint, bigint, {cur}) is 'effects: {ann}';\n"
    )
}

// ---------------------------------------------------------------- 5.1 effect annotations

#[test]
fn a_body_within_its_declared_row_is_accepted() {
    let c = errors(
        "r1",
        &transfer_r1("usd", "append; debits usd; credits usd; reads@ledger_consistent"),
    );
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn a_body_posting_a_currency_its_row_does_not_name_is_nl0310() {
    let c = errors(
        "r1",
        &transfer_r1("eur", "append; debits usd; credits usd; reads@ledger_consistent"),
    );
    assert_eq!(c, ["NL0310"]);
}

#[test]
fn a_wildcard_currency_permits_every_currency() {
    let c = errors(
        "r1",
        &transfer_r1("eur", "append; debits *; credits *; reads@ledger_consistent"),
    );
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn an_undeclared_read_is_nl0310_and_reads_match_exactly() {
    // The body reads the ledger to allocate its ids: `reads@ledger_consistent`. Declaring a
    // different rung does not cover it, in either direction (Niles's rule).
    let c = errors(
        "r1",
        &transfer_r1("usd", "append; debits usd; credits usd; reads@snapshot"),
    );
    assert_eq!(c, ["NL0310"]);
}

#[test]
fn a_read_of_a_contracted_view_is_its_rung() {
    let prog = |rung: &str| {
        format!(
            "create view v as select acct, (sum(amt_usd)).minor as b from postings group by acct;\n\
             insert into serve_contract values ('v', '{rung}', 'auto', 'evictable', null, 'off');\n\
             create function peek(a bigint) returns bigint language sql as $$ select b from v where acct = a $$;\n\
             comment on function peek(bigint) is 'effects: reads@snapshot';\n"
        )
    };
    assert!(errors("r1", &prog("snapshot")).is_empty());
    assert_eq!(errors("r1", &prog("bounded")), ["NL0310"]);
}

#[test]
fn effects_follow_calls_through_an_unannotated_function() {
    // The inner function posts eur; the middle one has no annotation, so its inferred
    // effects are the inner one's; the outer one is declared usd.
    let prog = |decl: &str| {
        format!(
            "create function inner_leg(t bigint, a bigint, x eur, e bigint) returns void language sql as $$\n\
               insert into postings (txn, acct, cur, amt_eur, epoch, value_date) values (t, a, 'eur', x, e, current_date) $$;\n\
             create function middle(t bigint, a bigint, x eur, e bigint) returns void language plpgsql as $$\n\
             begin perform inner_leg(t, a, x, e); end $$;\n\
             create function outer_fn(a bigint, x eur) returns void language plpgsql as $$\n\
             begin perform middle(1, a, x, 1); end $$;\n\
             comment on function outer_fn(bigint, eur) is 'effects: append; credits {decl}';\n"
        )
    };
    assert!(!errors("r1", &prog("eur")).contains(&"NL0310"));
    assert!(errors("r1", &prog("usd")).contains(&"NL0310"));
}

#[test]
fn a_hold_and_a_grant_are_effects_of_the_functions_that_make_them() {
    let hold = |decl: &str| {
        format!(
            "create function h(a bigint, m usd) returns void language plpgsql as $$\n\
             declare x hold_ref; begin x := hold(a, m); perform resolve_void(x); end $$;\n\
             comment on function h(bigint, usd) is 'effects: {decl}';\n"
        )
    };
    assert!(errors("r1", &hold("holds usd")).is_empty());
    assert_eq!(errors("r1", &hold("holds eur")), ["NL0310"]);
    let grant = |decl: &str| {
        format!(
            "create function g(a bigint, m usd, cap auth_overdraft) returns void language plpgsql as $$\n\
             begin perform authorize_overdraft(a, m); end $$;\n\
             comment on function g(bigint, usd, auth_overdraft) is 'effects: {decl}';\n"
        )
    };
    assert!(errors("r1", &grant("authorizes usd")).is_empty());
    assert_eq!(errors("r1", &grant("append")), ["NL0310"]);
}

#[test]
fn an_unannotated_function_is_not_held_to_anything() {
    let prog = transfer_r1("eur", "x").replace("comment on function pay(bigint, bigint, eur) is 'effects: x';\n", "");
    assert!(errors("r1", &prog).is_empty());
}

#[test]
fn r2_legs_take_their_currency_from_the_cur_literal() {
    let prog = |cur: &str| {
        format!(
            "create function pay(a bigint, b bigint, m {cur}) returns void language plpgsql as $$\n{HEAD}    insert into postings (txn, acct, cur, amt, epoch, value_date) values\n        (t, a, '{cur}', -(m).minor, e, current_date),\n        (t, b, '{cur}', (m).minor, e, current_date);\nend $$;\ncomment on function pay(bigint, bigint, {cur}) is 'effects: append; debits usd; credits usd; reads@ledger_consistent';\n"
        )
    };
    assert!(errors("r2", &prog("usd")).is_empty(), "{:?}", errors("r2", &prog("usd")));
    assert_eq!(errors("r2", &prog("eur")), ["NL0310"]);
}

#[test]
fn an_unreadable_annotation_item_is_a_warning_not_a_pass() {
    let c = all(
        "r1",
        &transfer_r1("usd", "append; debits usd; credits usd; reads@ledger_consistent; moves money"),
        false,
    );
    assert!(c.contains(&"NSQ003"), "{c:?}");
}

// ---------------------------------------------------------------- 5.2 typing of bodies

fn body(params: &str, decls: &str, stmts: &str) -> String {
    format!(
        "create function f({params}) returns void language plpgsql as $$\ndeclare\n    t bigint := 1;\n    e bigint := 1;\n{decls}\nbegin\n{stmts}\nend $$;\n"
    )
}

#[test]
fn minor_units_of_two_currencies_do_not_add() {
    let prog = |n: &str| body(&format!("m usd, n {n}"), "    x bigint;", "    x := (m).minor + (n).minor;");
    assert!(errors("r1", &prog("usd")).is_empty());
    assert_eq!(errors("r1", &prog("eur")), ["NL0250"]);
}

#[test]
fn a_currency_built_from_another_currencys_minor_units_is_nl0255() {
    let prog = |src: &str| {
        body(
            &format!("a bigint, b bigint, m {src}"),
            "",
            "    insert into postings (txn, acct, cur, amt_eur, epoch, value_date) values\n        (t, a, 'eur', row(-(m).minor)::eur, e, current_date),\n        (t, b, 'eur', row((m).minor)::eur, e, current_date);",
        )
    };
    assert!(errors("r1", &prog("eur")).is_empty());
    assert_eq!(errors("r1", &prog("usd")), ["NL0255"]);
}

#[test]
fn an_argument_of_the_wrong_currency_is_nl0255() {
    let prog = |n: &str| {
        format!(
            "create function g(a bigint, m usd) returns void language plpgsql as $$ begin perform 1; end $$;\n{}",
            body(&format!("a bigint, n {n}"), "", "    perform g(a, n);")
        )
    };
    assert!(errors("r1", &prog("usd")).is_empty());
    assert_eq!(errors("r1", &prog("eur")), ["NL0255"]);
}

#[test]
fn a_ledger_row_labelled_with_another_currency_is_nl0255_in_r1() {
    let prog = |label: &str| {
        body(
            "a bigint, b bigint, m usd",
            "",
            &format!("    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values\n        (t, a, '{label}', row(-(m).minor)::usd, e, current_date),\n        (t, b, '{label}', m, e, current_date);"),
        )
    };
    assert!(errors("r1", &prog("usd")).is_empty());
    assert_eq!(errors("r1", &prog("eur")), ["NL0255"]);
}

#[test]
fn a_ledger_row_labelled_with_another_currency_is_nl0255_in_r2() {
    let prog = |label: &str| {
        body(
            "a bigint, b bigint, m usd",
            "",
            &format!("    insert into postings (txn, acct, cur, amt, epoch, value_date) values\n        (t, a, '{label}', -(m).minor, e, current_date),\n        (t, b, '{label}', (m).minor, e, current_date);"),
        )
    };
    assert!(errors("r2", &prog("usd")).is_empty(), "{:?}", errors("r2", &prog("usd")));
    assert_eq!(errors("r2", &prog("eur")), ["NL0255"]);
}

#[test]
fn a_declaration_or_a_return_of_the_wrong_currency_is_nl0332() {
    let decl = |n: &str| body(&format!("n {n}"), "    x usd := n;", "    perform 1;");
    assert!(errors("r1", &decl("usd")).is_empty());
    assert_eq!(errors("r1", &decl("eur")), ["NL0332"]);
    let ret = |n: &str| {
        format!("create function r(n {n}) returns usd language plpgsql as $$ begin return n; end $$;\n")
    };
    assert!(errors("r1", &ret("usd")).is_empty());
    assert_eq!(errors("r1", &ret("eur")), ["NL0332"]);
}

#[test]
fn what_the_typing_cannot_see_it_leaves_alone() {
    // A plain integer relabelled as usd: no currency to compare, so nothing is refused.
    let prog = body(
        "a bigint, b bigint, k bigint",
        "",
        "    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values\n        (t, a, 'usd', row(-k)::usd, e, current_date),\n        (t, b, 'usd', row(k)::usd, e, current_date);",
    );
    assert!(errors("r1", &prog).is_empty(), "{:?}", errors("r1", &prog));
}

// ---------------------------------------------------------------- the adapter's reading of `(m).minor`

#[test]
fn a_parenthesised_minor_is_its_parameter_so_the_legs_cancel() {
    // `(m).minor` parses as the field of a one-element row. conserve.rs read that as a fresh
    // amount, so `row(-(m).minor)::usd` never cancelled `m`, contrary to its own
    // documentation, and a leg one minor unit off was undecided instead of refused.
    let prog = |credit: &str| {
        format!(
            "create function pay(a bigint, b bigint, m usd) returns void language plpgsql as $$\n{HEAD}    insert into postings (txn, acct, cur, amt_usd, epoch, value_date) values\n        (t, a, 'usd', row(-(m).minor)::usd, e, current_date),\n        (t, b, 'usd', {credit}, e, current_date);\nend $$;\n"
        )
    };
    assert!(errors("r1", &prog("m")).is_empty());
    assert_eq!(errors("r1", &prog("row((m).minor + 1)::usd")), ["NL0300"]);
}
