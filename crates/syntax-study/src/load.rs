//! **The load check** (design §4): each surface loads the same rows, and the loaded row count
//! and SHA-256 of the canonical rows are asserted equal per surface. Each surface's rows are
//! read back *from its executor* — PostgreSQL's tables, Soufflé's input relations, the
//! sources the Niles evaluator is handed — and compared with the dataset's own.
//!
//! Canonical rows: `parties` id|parent, `accounts` id|owner|desk, `postings`
//! txn|acct|cur|amt|value_date (ISO), `holds` id|acct|cur|amount|open (0/1), `null` for a
//! missing value; and, where a surface stores a posting's epoch as data, `epochs`
//! txn|acct|epoch. Rows are sorted before hashing.

use crate::data::Dataset;
use crate::kinds::{day_iso, iso};
use std::collections::BTreeMap;
use std::path::Path;

pub type Rows = BTreeMap<String, Vec<String>>;

pub fn expected(d: &Dataset, with_epochs: bool) -> Rows {
    let mut r = Rows::new();
    r.insert(
        "parties".into(),
        d.parties
            .iter()
            .map(|p| {
                format!(
                    "{}|{}",
                    p.id,
                    p.parent.map(|x| x.to_string()).unwrap_or("null".into())
                )
            })
            .collect(),
    );
    r.insert(
        "accounts".into(),
        d.accounts
            .iter()
            .map(|a| format!("{}|{}|{}", a.id, a.owner, a.desk))
            .collect(),
    );
    r.insert(
        "postings".into(),
        d.postings
            .iter()
            .map(|p| {
                format!(
                    "{}|{}|{}|{}|{}",
                    p.txn,
                    p.acct,
                    p.cur,
                    p.amt,
                    day_iso(p.value_date)
                )
            })
            .collect(),
    );
    r.insert(
        "holds".into(),
        d.holds
            .iter()
            .map(|h| {
                format!(
                    "{}|{}|{}|{}|{}",
                    h.id, h.acct, h.cur, h.amount, h.open as i32
                )
            })
            .collect(),
    );
    if with_epochs {
        r.insert(
            "epochs".into(),
            d.postings
                .iter()
                .map(|p| format!("{}|{}|{}", p.txn, p.acct, p.epoch))
                .collect(),
        );
    }
    r
}

/// Row count and SHA-256 (hex) of a relation's sorted rows.
pub fn digest(rows: &[String]) -> (usize, String) {
    let mut v = rows.to_vec();
    v.sort();
    let h = nilestream_ledger::chain::sha256(v.join("\n").as_bytes());
    (v.len(), h.iter().map(|b| format!("{b:02x}")).collect())
}

fn bool01(s: &str) -> String {
    match s {
        "t" | "true" => "1".into(),
        "f" | "false" => "0".into(),
        o => o.into(),
    }
}

/// Read back from PostgreSQL after loading `schema` and `data` (inside `begin … rollback`).
pub fn from_pg(c: &mut bank_bench::wire::Client, typed: bool, data: &str) -> Result<Rows, String> {
    let schema = if typed {
        crate::pg::SCHEMA_SQL
    } else {
        crate::pg::SCHEMA_PLAIN
    };
    let (money, hold_money) = if typed {
        (
            "coalesce((amt_usd).minor, (amt_eur).minor, (amt_jpy).minor)",
            "coalesce((amt_usd).minor, (amt_eur).minor)",
        )
    } else {
        ("amt", "amount")
    };
    let steps: Vec<(&'static str, String)> = vec![
        (
            "read",
            "select id, coalesce(parent::text, 'null') from parties".into(),
        ),
        ("read", "select id, owner, desk from accounts".into()),
        (
            "read",
            format!("select txn, acct, cur, {money}, value_date from postings"),
        ),
        (
            "read",
            format!("select id, acct, cur, {hold_money}, open from holds"),
        ),
        ("read", "select txn, acct, epoch from postings".into()),
    ];
    let reads = crate::pg::run(c, schema, data, "", &steps).map_err(|e| format!("{e:?}"))?;
    let names = ["parties", "accounts", "postings", "holds", "epochs"];
    let mut r = Rows::new();
    for (n, rows) in names.iter().zip(reads) {
        r.insert(
            n.to_string(),
            rows.into_iter()
                .map(|row| row.iter().map(|x| bool01(x)).collect::<Vec<_>>().join("|"))
                .collect(),
        );
    }
    Ok(r)
}

/// Read back Soufflé's input relations: a program that copies each one to an output.
pub fn from_dl(bin: &Path, facts: &Path, work: &Path) -> Result<Rows, String> {
    let mut r = Rows::new();
    let copies: [(&str, &str, &str); 5] = [
        ("parties", "answer(x) :- parties(x).", "x: number"),
        (
            "parent",
            "answer(c, p) :- parent(c, p).",
            "c: number, p: number",
        ),
        (
            "accounts",
            "answer(i, o, k) :- accounts(i, o, k).",
            "i: number, o: number, k: number",
        ),
        (
            "postings",
            "answer(t, a, c, m, e, v) :- postings(t, a, c, m, e, v).",
            "t: number, a: number, c: symbol, m: number, e: number, v: number",
        ),
        (
            "holds",
            "answer(i, a, c, m, o) :- holds(i, a, c, m, o).",
            "i: number, a: number, c: symbol, m: number, o: number",
        ),
    ];
    let mut raw: BTreeMap<&str, Vec<Vec<String>>> = BTreeMap::new();
    for (rel, rule, decl) in copies {
        let prog = format!(".decl answer({decl})\n{rule}\n.output answer\n");
        let rows = crate::ext::souffle_run(bin, facts, &work.join(format!("load-{rel}")), &prog)
            .map_err(|e| format!("{rel}: {e:?}"))?;
        raw.insert(rel, rows);
    }
    let parent: BTreeMap<String, String> = raw["parent"]
        .iter()
        .map(|r| (r[0].clone(), r[1].clone()))
        .collect();
    r.insert(
        "parties".into(),
        raw["parties"]
            .iter()
            .map(|x| {
                format!(
                    "{}|{}",
                    x[0],
                    parent.get(&x[0]).cloned().unwrap_or("null".into())
                )
            })
            .collect(),
    );
    r.insert(
        "accounts".into(),
        raw["accounts"].iter().map(|x| x.join("|")).collect(),
    );
    let day = |s: &str| s.parse::<i64>().map(iso).unwrap_or_else(|_| s.to_string());
    r.insert(
        "postings".into(),
        raw["postings"]
            .iter()
            .map(|x| format!("{}|{}|{}|{}|{}", x[0], x[1], x[2], x[3], day(&x[5])))
            .collect(),
    );
    r.insert(
        "epochs".into(),
        raw["postings"]
            .iter()
            .map(|x| format!("{}|{}|{}", x[0], x[1], x[4]))
            .collect(),
    );
    r.insert(
        "holds".into(),
        raw["holds"].iter().map(|x| x.join("|")).collect(),
    );
    Ok(r)
}
