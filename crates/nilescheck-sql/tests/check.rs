//! The checker against cycle 13's Appendix A shapes: each diagnostic fires on the view built
//! to draw it and on no other, and the clean views draw nothing.

use nilescheck_sql::check::Diag;
use nilescheck_sql::check_all as check;

fn diags() -> Vec<Diag> {
    let src = include_str!("fixtures/contracts.sql");
    let (stmts, _) = nilescheck_sql::parse(src).expect("the fixture parses");
    check(&stmts)
}

fn has(d: &[Diag], code: &str, needle: &str) -> bool {
    d.iter().any(|x| x.code == code && x.msg.contains(needle))
}

#[test]
fn each_rule_fires_where_it_should_and_nowhere_else() {
    let d = diags();
    let show = || {
        d.iter()
            .map(|x| format!("{} {}", x.code, x.msg))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(has(&d, "NL0220", "available_balance"), "{}", show());
    assert!(has(&d, "NL0222", "`top_accounts`"), "{}", show());
    assert!(!has(&d, "NL0222", "top_accounts_snapshot"), "{}", show());
    assert!(has(&d, "NL0223", "daily_totals"), "{}", show());
    assert!(has(&d, "NL0311", "derived_strict"), "{}", show());
    assert!(has(&d, "IR013", "mtd_now"), "{}", show());
    assert_eq!(
        d.iter()
            .filter(|x| x.code == "NL0260" && x.msg.contains("postings.owner"))
            .count(),
        1,
        "{}",
        show()
    );
    assert_eq!(
        d.iter().filter(|x| x.code == "NL0230").count(),
        2,
        "{}",
        show()
    );
    for clean in ["ledger_balance", "clean_balance"] {
        assert!(
            !d.iter()
                .any(|x| x.msg.starts_with(&format!("view `{clean}`"))),
            "{clean} is clean:\n{}",
            show()
        );
    }
    assert_eq!(d.len(), 8, "exactly the eight expected:\n{}", show());
}
