//! **SQL+C+L's three rule families, held in both directions** (cycle 14, R2-05): each rule
//! refuses its defect, and accepts the correct program beside it — a checker that refused
//! everything would pass the first half alone.

fn preamble() -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../counterproposal/sql-checked/_preamble.sql"),
    )
    .unwrap()
}

fn codes(extra: &str) -> Vec<&'static str> {
    let src = format!("{}\n{extra}", preamble());
    let (stmts, _) = nilescheck_sql::parse(&src).unwrap_or_else(|e| panic!("{}: {extra}", e.msg));
    let mut v: Vec<&'static str> = nilescheck_sql::check_all(&stmts)
        .into_iter()
        .map(|d| d.code)
        .collect();
    v.sort();
    v
}

fn pl(name: &str, params: &str, decls: &str, body: &str) -> String {
    format!(
        "create function {name}({params}) returns void language plpgsql as $$\ndeclare\n{decls}\nbegin\n{body}\nend $$;\n"
    )
}

// ---------------- linearity ----------------

#[test]
fn a_hold_resolved_on_every_arm_of_an_if_is_accepted() {
    let c = codes(&pl(
        "f",
        "a bigint, ok boolean",
        "h hold_ref;",
        "h := hold(a, row(1)::usd);\nif ok then perform resolve_post(h, row(1)::usd); else perform resolve_void(h); end if;",
    ));
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn a_hold_resolved_on_one_arm_only_is_refused() {
    let c = codes(&pl(
        "f",
        "a bigint, ok boolean",
        "h hold_ref;",
        "h := hold(a, row(1)::usd);\nif ok then perform resolve_void(h); end if;",
    ));
    assert_eq!(c, vec!["NL0320"]);
}

#[test]
fn a_hold_resolved_twice_is_refused() {
    let c = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref;",
        "h := hold(a, row(1)::usd);\nperform resolve_post(h, row(1)::usd);\nperform resolve_void(h);",
    ));
    assert_eq!(c, vec!["NL0321"]);
}

#[test]
fn a_path_that_raises_owes_nothing() {
    // The transaction rolls back, and the hold with it.
    let c = codes(&pl(
        "f",
        "a bigint, ok boolean",
        "h hold_ref;",
        "h := hold(a, row(1)::usd);\nif not ok then raise exception 'refused'; end if;\nperform resolve_void(h);",
    ));
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn a_return_before_the_hold_is_resolved_is_refused_and_after_is_accepted() {
    let early = codes(&pl(
        "f",
        "a bigint, ok boolean",
        "h hold_ref;",
        "h := hold(a, row(1)::usd);\nif ok then return; end if;\nperform resolve_void(h);",
    ));
    assert_eq!(
        early,
        vec!["NL0320"],
        "the early return leaves the hold open"
    );
    let late = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref;",
        "h := hold(a, row(1)::usd);\nperform resolve_void(h);\nreturn;",
    ));
    assert!(late.is_empty(), "{late:?}");
}

#[test]
fn a_hold_made_and_resolved_in_each_iteration_is_accepted_and_one_left_is_refused() {
    let ok = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref;",
        "for i in 1..3 loop\n  h := hold(a, row(1)::usd);\n  perform resolve_void(h);\nend loop;",
    ));
    assert!(ok.is_empty(), "{ok:?}");
    let left = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref;",
        "for i in 1..3 loop\n  h := hold(a, row(1)::usd);\nend loop;",
    ));
    assert!(left.contains(&"NL0322"), "{left:?}");
}

#[test]
fn a_hold_bound_before_a_loop_and_consumed_in_it_is_refused() {
    let c = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref;",
        "h := hold(a, row(1)::usd);\nfor i in 1..3 loop\n  perform resolve_void(h);\nend loop;",
    ));
    assert_eq!(c, vec!["NL0321"]);
}

#[test]
fn a_hold_moved_into_a_function_that_resolves_it_is_accepted() {
    let c = codes(&format!(
        "{}{}",
        pl("settle", "h hold_ref", "", "perform resolve_void(h);"),
        pl(
            "f",
            "a bigint",
            "h hold_ref;",
            "h := hold(a, row(1)::usd);\nperform settle(h);"
        )
    ));
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn a_hold_bound_to_a_variable_of_another_type_is_refused() {
    let c = codes(&pl(
        "f",
        "a bigint",
        "x bigint;",
        "x := hold(a, row(1)::usd);",
    ));
    assert_eq!(c, vec!["NL0320"]);
}

#[test]
fn a_hold_moved_between_variables_is_owed_once() {
    let ok = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref; g hold_ref;",
        "h := hold(a, row(1)::usd);\ng := h;\nperform resolve_void(g);",
    ));
    assert!(ok.is_empty(), "{ok:?}");
    let both = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref; g hold_ref;",
        "h := hold(a, row(1)::usd);\ng := h;\nperform resolve_void(h);",
    ));
    assert!(
        both.contains(&"NL0321") || both.contains(&"NL0320"),
        "{both:?}"
    );
}

#[test]
fn select_into_binds_and_a_select_with_no_destination_drops() {
    let ok = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref;",
        "select hold(a, row(1)::usd) into h;\nperform resolve_void(h);",
    ));
    assert!(ok.is_empty(), "{ok:?}");
    let dropped = codes(&pl(
        "f",
        "a bigint",
        "",
        "perform 1 + 1;\nperform hold(a, row(1)::usd) is not null;",
    ));
    assert_eq!(dropped, vec!["NL0320"]);
}

#[test]
fn an_exception_handler_is_an_alternative_path() {
    // The handler runs after the body's work was rolled back to the block's start: the hold
    // made in the body does not exist there, and the handler owes nothing for it.
    let c = codes(&pl(
        "f",
        "a bigint",
        "h hold_ref;",
        "begin\n  h := hold(a, row(1)::usd);\n  perform resolve_void(h);\nexception when others then\n  raise notice 'failed';\nend;",
    ));
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn without_a_linear_domain_the_rule_is_silent() {
    // The catalog checker's column must not change because this rule exists.
    let src = "create table t (a int);\ncreate function f() returns void language plpgsql as $$ begin perform 1; end $$;";
    let (stmts, _) = nilescheck_sql::parse(src).unwrap();
    assert!(nilescheck_sql::linear::check(&stmts).is_empty());
}

// ---------------- conservation ----------------

#[test]
fn a_transfer_of_a_parameter_amount_conserves() {
    let c = codes(
        "create function move_usd(a bigint, b bigint, m usd) returns void language plpgsql as $$
begin
    insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
        (1, 7, a, 'usd', row(-(m).minor)::usd, current_date),
        (1, 7, b, 'usd', m, current_date);
end $$;",
    );
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn two_transactions_are_judged_separately() {
    let c = codes(
        "create function two(a bigint, b bigint) returns void language plpgsql as $$
begin
    insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
        (1, 7, a, 'usd', row(-100)::usd, current_date),
        (1, 8, b, 'usd', row(100)::usd, current_date);
end $$;",
    );
    assert_eq!(
        c,
        vec!["NL0300", "NL0300"],
        "each of txn 7 and txn 8 is unbalanced"
    );
}

#[test]
fn an_amount_the_checker_cannot_see_through_is_undecided_not_accused() {
    let c = codes(
        "create function fee(a bigint, b bigint, m usd) returns void language plpgsql as $$
begin
    insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
        (1, 7, a, 'usd', row(-(m).minor)::usd, current_date),
        (1, 7, b, 'usd', row((m).minor / 2)::usd, current_date);
end $$;",
    );
    assert!(c.is_empty(), "undecided is not a diagnostic: {c:?}");
}

#[test]
fn the_arms_of_an_if_are_joined_not_summed() {
    // Each arm posts a balanced pair: summing both arms would double the legs, and the
    // solver's join keeps what the arms agree on.
    let c = codes(
        "create function pay(a bigint, b bigint, big boolean) returns void language plpgsql as $$
begin
    if big then
        insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
            (1, 7, a, 'usd', row(-500)::usd, current_date), (1, 7, b, 'usd', row(500)::usd, current_date);
    else
        insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
            (1, 7, a, 'usd', row(-5)::usd, current_date), (1, 7, b, 'usd', row(5)::usd, current_date);
    end if;
end $$;",
    );
    assert!(c.is_empty(), "{c:?}");
}

/// **The solver's own limit, inherited unchanged.** One path balances and the other loses
/// 5.00: the arms disagree, and `Row::join` (`niles-lang/src/currency_rows.rs`) goes to top
/// on disagreement — undecided, not an accusation. The adapter does not do better than the
/// solver it reuses; this test records the inherited limit, so a change to either is seen.
#[test]
fn a_leg_that_only_one_arm_posts_is_undecided_as_the_solver_rules() {
    let c = codes(
        "create function pay(a bigint, b bigint, big boolean) returns void language plpgsql as $$
begin
    insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
        (1, 7, a, 'usd', row(-500)::usd, current_date);
    if big then
        insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
            (1, 7, b, 'usd', row(500)::usd, current_date);
    end if;
end $$;",
    );
    assert!(c.is_empty(), "{c:?}");
}

/// Losing the same amount on **every** path is a must-violation after the join.
#[test]
fn a_loss_on_every_path_is_refused_after_the_join() {
    let c = codes(
        "create function pay(a bigint, b bigint, big boolean) returns void language plpgsql as $$
begin
    if big then
        insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
            (1, 7, a, 'usd', row(-500)::usd, current_date), (1, 7, b, 'usd', row(495)::usd, current_date);
    else
        insert into postings (epoch, txn, acct, cur, amt_usd, value_date) values
            (1, 7, a, 'usd', row(-10)::usd, current_date), (1, 7, b, 'usd', row(5)::usd, current_date);
    end if;
end $$;",
    );
    assert_eq!(c, vec!["NL0300"], "{c:?}");
}

// ---------------- capabilities ----------------

#[test]
fn a_function_holding_the_capability_may_call_what_requires_it() {
    let c = codes(&pl(
        "f",
        "a bigint, auth auth_overdraft",
        "",
        "perform authorize_overdraft(a, row(1)::usd);",
    ));
    assert!(c.is_empty(), "{c:?}");
}

#[test]
fn a_capability_cannot_be_cast_into_existence() {
    let c = codes(
        "create function g(a bigint, auth auth_overdraft) returns void language plpgsql as $$ begin perform authorize_overdraft(a, row(1)::usd); end $$;
create function f(a bigint) returns void language plpgsql as $$ begin perform g(a, 42::auth_overdraft); end $$;",
    );
    assert_eq!(c, vec!["NL0330"]);
}

#[test]
fn a_function_returning_a_capability_is_a_factory_unless_it_grants() {
    let factory =
        codes("create function mint() returns auth_overdraft language sql as $$ select 1 $$;");
    assert!(factory.contains(&"NL0330"), "{factory:?}");
    let granter = codes(
        "create function mint() returns auth_overdraft language sql as $$ select 1::auth_overdraft $$;
comment on function mint() is 'grants overdraft';",
    );
    assert!(granter.is_empty(), "{granter:?}");
}

#[test]
fn a_top_level_call_that_requires_a_capability_is_refused() {
    let c = codes("select authorize_overdraft(1, row(1)::usd);");
    assert_eq!(c, vec!["NL0312"]);
}
