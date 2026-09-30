//! **What each output column is**, so a value every surface encodes differently — a currency
//! as a name, a text, or its declaration index; a date as ISO text or days since 1970 — is
//! rendered once, the same way, before answers are compared.

use niles_ir::value::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Int,
    Cur,
    Date,
    Text,
}

/// `YYYY-MM-DD` of a day count since 1970-01-01 (Howard Hinnant's civil-from-days).
pub fn iso(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// A dataset day (days since 2026-01-01) as ISO text.
pub fn day_iso(day: i64) -> String {
    iso(niles_ir::value::days_since_epoch("2026-01-01").expect("a date") + day)
}

/// One IR value, rendered for an answer.
pub fn render(v: Value, k: Kind, codes: &BTreeMap<i128, String>, _day0: i64) -> String {
    match (v, k) {
        (Value::Null, _) => "NULL".into(),
        (Value::Int(i), Kind::Cur) => codes.get(&i).cloned().unwrap_or_else(|| format!("?{i}")),
        (Value::Int(i), Kind::Date) => iso(i as i64),
        (Value::Int(i), _) => i.to_string(),
        // Since cycle 15 (C15-05b): money renders as its minor units, as it did when it was an
        // `Int`, and text as itself.
        (Value::Money { minor, .. }, _) => minor.to_string(),
        (v @ Value::Text(_), _) => v.to_string(),
    }
}

/// The column kinds of each task's answer.
pub fn of(task: &str) -> &'static [Kind] {
    use Kind::*;
    match task {
        "Q01" | "Q10" | "V01" | "V02" | "V03" | "B01" | "B02" | "B04" => &[Int, Cur, Int],
        "Q02" | "Q05" | "Q06" | "V04" => &[Int, Int],
        "Q03" => &[Int, Cur, Int],
        "Q04" => &[Int],
        "Q07" => &[Int, Cur, Int, Int],
        "Q08" => &[Int, Int, Int],
        "Q09" => &[Int, Int],
        "V05" => &[Cur, Int],
        "B03" => &[Text, Int, Cur, Int],
        "B05" => &[Int, Date, Cur, Int],
        "T09" => &[Int, Cur, Int],
        t if t.starts_with('T') => &[Int, Cur, Int],
        other => panic!("no task {other}"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn iso_round_trips() {
        for t in [
            "1970-01-01",
            "2026-01-01",
            "2026-02-09",
            "2024-02-29",
            "2000-03-01",
        ] {
            let d = niles_ir::value::days_since_epoch(t).unwrap();
            assert_eq!(super::iso(d), t);
        }
    }
}
