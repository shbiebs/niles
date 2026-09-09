//! **One machine-readable line per reported event, so a reader never greps prose.**
//!
//! Every refusal this harness makes used to be a `grep` for an English sentence in a
//! benchmark transcript: `"E19 mixed"`, `"flights at .*: n/a"`, `"  merge at "`,
//! `"(^|[^0-9.])0 writes/s"`. Three separate defects came out of that one habit in cycle 10
//! alone — a substring match that read `900 writes/s` as `0 writes/s` (fixed by a character
//! class, which is a patch and not a fix), a check that lived under one mode and so watched
//! nothing in the others (F-11-20), and a renamed server column that answered
//! `unwrap_or(0)` for four cycles. The common cause is that the *presentation* was the
//! interface. Rewording a sentence for a human reader could change a verdict, and the only
//! thing standing between a reworded line and a wrong pass was that nobody had reworded one
//! yet.
//!
//! So every event the harness scores now prints, beside its human line, exactly one line of
//! this form:
//!
//! ```text
//! NLSBENCH/1 kind=mixed target=nilestream run=1 readers=6 writers=3 reads=41231 ...
//! ```
//!
//! and the harness parses *that*. The human line above it is then free to say anything at
//! all; a test asserts that rewording it changes no verdict.
//!
//! # The four outcomes, kept four
//!
//! The distinction this module exists to preserve is between four things a field can be, and
//! the reason each is separate is a real wrong number in this repository's history:
//!
//! | outcome | on the wire | means |
//! |---|---|---|
//! | **missing** | the key is not in the line | this build does not report the field at all. A reader asking for it is asking a question this binary cannot answer, and the answer is not `0`. |
//! | **not measured** | `key=n/a` | the build reports the field and could not obtain it this run — the server refused the connection, the counter was not exposed. Distinct from `0`, which is a measurement. |
//! | **zero** | `key=0` | measured, and the measurement is zero. `deferred_merges=0` is the finding, not the absence of one. |
//! | **malformed** | `key=` or `key=abc` where a number belongs | the writer is broken. Never silently coerced; it refuses. |
//!
//! A field is **never omitted to mean absent-this-run** — a kind's schema is fixed, every one
//! of its fields is always printed, and `n/a` carries "not obtained". That is what makes
//! *missing* mean "different build" and nothing else.
//!
//! # Why the field list is the CSV header
//!
//! `mixed` lines take their field names and their order from [`MIXED_CSV_HEADER`] and their
//! values from the same `Vec<String>` that writes the CSV row. The line and the row cannot
//! drift, because there is one list. `results_headers.rs` already refuses a committed CSV
//! whose header the writer no longer writes; that test now also protects the summary line.
//!
//! [`MIXED_CSV_HEADER`]: crate::workloads::MIXED_CSV_HEADER

use std::collections::BTreeMap;

/// The version-stamped prefix. A reader that does not recognise the version refuses rather
/// than parsing a schema it was not written against.
pub const SUMMARY_PREFIX: &str = "NLSBENCH/1";

/// The literal a field carries when the writer reports it and could not obtain it.
pub const NOT_MEASURED: &str = "n/a";

/// Escape one value so a line is unambiguously splittable on spaces and `=`.
///
/// Only three characters are touched, and the encoding is reversible, so a `not_run` reason
/// or a target name with a space in it survives the round trip instead of being silently
/// truncated at the space — which is how a two-word reason would otherwise become a stray
/// unparsable token and take the whole line down with it.
pub fn escape(v: &str) -> String {
    let mut s = String::with_capacity(v.len());
    for c in v.chars() {
        match c {
            '%' => s.push_str("%25"),
            ' ' => s.push_str("%20"),
            '=' => s.push_str("%3D"),
            '\n' | '\r' | '\t' => s.push_str("%20"),
            _ => s.push(c),
        }
    }
    if s.is_empty() {
        // An empty value would render as a bare `key=`, which this module's own parser calls
        // malformed. An empty string is a legitimate value (`not_run` when the level ran), so
        // it gets a token of its own.
        return "-".into();
    }
    s
}

/// Inverse of [`escape`].
pub fn unescape(v: &str) -> String {
    if v == "-" {
        return String::new();
    }
    let mut out = String::with_capacity(v.len());
    let b: Vec<char> = v.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '%' && i + 2 < b.len() {
            let hex: String = b[i + 1..i + 3].iter().collect();
            if let Ok(n) = u8::from_str_radix(&hex, 16) {
                out.push(n as char);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// Render one line from a comma-separated header and the values that go with it.
///
/// # Panics
///
/// Never. A length mismatch is rendered as a `schema_error` field rather than panicking in
/// the middle of a measurement run: losing the run to a formatting bug would cost more than
/// the malformed line the reader will refuse anyway.
pub fn line(kind: &str, header: &str, values: &[String]) -> String {
    let names: Vec<&str> = header.split(',').collect();
    let mut s = format!("{SUMMARY_PREFIX} kind={}", escape(kind));
    if names.len() != values.len() {
        s.push_str(&format!(
            " schema_error=header%20{}%20values%20{}",
            names.len(),
            values.len()
        ));
        return s;
    }
    for (n, v) in names.iter().zip(values) {
        s.push(' ');
        s.push_str(n.trim());
        s.push('=');
        s.push_str(&escape(v));
    }
    s
}

/// What a reader gets when it asks a parsed line for a numeric field.
///
/// The four variants are the four outcomes of this module's opening table, and no method on
/// them collapses one into another. In particular there is no `unwrap_or(0)`.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    /// The key is not in the line: this build does not report it.
    Missing,
    /// `key=n/a`: reported, not obtained this run.
    NotMeasured,
    /// A number.
    Num(f64),
    /// Present and not a number where one belongs. Carries the raw token.
    Malformed(String),
}

impl Field {
    /// The number, or `None` — for a caller that has *already decided* what the other three
    /// outcomes mean and wants to say so at its own call site. Deliberately not `unwrap_or`.
    pub fn number(&self) -> Option<f64> {
        match self {
            Field::Num(n) => Some(*n),
            _ => None,
        }
    }
    /// A short word naming the outcome, for a refusal message.
    pub fn why(&self) -> String {
        match self {
            Field::Missing => "missing (this build does not report it)".into(),
            Field::NotMeasured => "n/a (reported, not obtained this run)".into(),
            Field::Num(n) => format!("{n}"),
            Field::Malformed(t) => format!("malformed ({t:?})"),
        }
    }
}

/// One parsed summary line.
#[derive(Debug, Clone)]
pub struct Line {
    pub kind: String,
    fields: BTreeMap<String, String>,
}

impl Line {
    /// Parse one line. `None` when the text is not a summary line of a version this reader
    /// knows — never a partially parsed line, and never a line with a duplicated key, which
    /// would make "the value" depend on which occurrence a reader happened to take.
    pub fn parse(text: &str) -> Option<Line> {
        let t = text.trim();
        let rest = t.strip_prefix(SUMMARY_PREFIX)?;
        let mut fields = BTreeMap::new();
        let mut kind = None;
        for tok in rest.split_whitespace() {
            let (k, v) = tok.split_once('=')?;
            if k.is_empty() {
                return None;
            }
            if k == "kind" {
                if kind.is_some() {
                    return None;
                }
                kind = Some(unescape(v));
                continue;
            }
            if fields.insert(k.to_string(), v.to_string()).is_some() {
                return None;
            }
        }
        Some(Line {
            kind: kind?,
            fields,
        })
    }

    /// Every field name this line carries, sorted. `kind` is not among them.
    pub fn names(&self) -> Vec<&str> {
        self.fields.keys().map(|s| s.as_str()).collect()
    }

    /// The raw (unescaped) text of a field, or `None` when the field is missing.
    pub fn text(&self, name: &str) -> Option<String> {
        self.fields.get(name).map(|v| unescape(v))
    }

    /// A numeric field, resolved into exactly one of the four outcomes.
    pub fn num(&self, name: &str) -> Field {
        let Some(raw) = self.fields.get(name) else {
            return Field::Missing;
        };
        if raw == NOT_MEASURED {
            return Field::NotMeasured;
        }
        let plain = unescape(raw);
        match plain.trim().parse::<f64>() {
            Ok(n) if n.is_finite() => Field::Num(n),
            _ => Field::Malformed(plain),
        }
    }

    /// Every summary line in a transcript, in order.
    pub fn all_in(text: &str) -> Vec<Line> {
        text.lines().filter_map(Line::parse).collect()
    }

    /// Every summary line of one kind, in order.
    pub fn all_of(text: &str, kind: &str) -> Vec<Line> {
        Line::all_in(text)
            .into_iter()
            .filter(|l| l.kind == kind)
            .collect()
    }
}

/// The field names each kind is required to carry, in the order it prints them.
///
/// This table is what a schema test asserts against, and what makes a *missing* field
/// detectable: a reader that knows the kind knows every name it must see, so a build that
/// does not print one is distinguishable from a build that printed `n/a`.
pub fn schema(kind: &str) -> Option<Vec<&'static str>> {
    let h = match kind {
        "mixed" => crate::workloads::MIXED_CSV_HEADER,
        "flights" => FLIGHTS_HEADER,
        "score" => SCORE_HEADER,
        _ => return None,
    };
    Some(h.split(',').map(|s| s.trim()).collect())
}

/// Every kind this crate emits, so a test can enumerate them.
pub const KINDS: &[&str] = &["mixed", "flights", "score"];

/// The flight-counter line: the level's shape, every counter the server is asked for, and
/// the three derived quantities the transcript's prose used to compute inline.
///
/// The counters are the *deltas over this level* except the three named `_level`, which are
/// running maxima or configured caps and are not differences of anything.
pub const FLIGHTS_HEADER: &str = "shape,pending_joins,uninstalled_folds,pinned_installs,\
    flights_refused,deferred_merges,waiters_refused,joins_answered,joins_retried,\
    gap_at_begin_total,gap_at_finish_total,gap_at_finish_max_level,flights_behind_at_begin,\
    flights_that_fell_behind,gap_begin_samples,gap_finish_samples,merge_rows_visited,\
    merge_epochs_merged,merges_refused_epochs,merges_refused_rows,merges_refused_unavailable,\
    caps_epochs,caps_rows,late_landings,joins_ended,folds_from_counters";

/// One scored comparison: an arm pair at one working point and one level.
///
/// `verdict` is one of `fires`, `noise-limited`, `refused`; `direction` one of `faster`,
/// `slower`, `none`. `gate_rel` and `gate_mads` are the two thresholds this row was scored
/// against, printed so a row carries the gate it passed rather than the gate the reader
/// assumes.
pub const SCORE_HEADER: &str = "point,level,readers,writers,metric,arm_a,arm_b,n_a,n_b,\
    median_a,median_b,mad_a,mad_b,min_a,max_a,min_b,max_b,pooled_mad,rel_change,mads_apart,\
    gate_rel,gate_mads,verdict,direction";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_with_spaces_and_equals_survives_the_round_trip() {
        for v in [
            "plain",
            "two words",
            "a=b",
            "100% of it",
            "",
            "line\nbreak",
            "%20 literal",
        ] {
            assert_eq!(unescape(&escape(v)), v.replace(['\n', '\r', '\t'], " "));
        }
    }

    #[test]
    fn the_four_outcomes_are_four() {
        let l = Line::parse("NLSBENCH/1 kind=t zero=0 na=n/a bad=abc").expect("parses");
        assert_eq!(l.num("zero"), Field::Num(0.0));
        assert_eq!(l.num("na"), Field::NotMeasured);
        assert_eq!(l.num("bad"), Field::Malformed("abc".into()));
        assert_eq!(l.num("nowhere"), Field::Missing);
    }

    #[test]
    fn a_duplicated_key_is_not_a_line() {
        assert!(Line::parse("NLSBENCH/1 kind=t a=1 a=2").is_none());
        assert!(Line::parse("NLSBENCH/1 kind=t kind=u").is_none());
    }

    #[test]
    fn a_line_without_the_version_prefix_is_not_a_line() {
        assert!(Line::parse("NLSBENCH/2 kind=t a=1").is_none());
        assert!(Line::parse("  E19 mixed nilestream 6r/3w run 1 — 41231 reads/s").is_none());
    }

    #[test]
    fn a_bare_key_is_malformed_and_not_missing() {
        // `escape("")` is `-`, so a writer never emits `key=`; a hand-written or truncated
        // line that does must be told apart from a key that is not there.
        let l = Line::parse("NLSBENCH/1 kind=t a=").expect("parses");
        assert_eq!(l.num("a"), Field::Malformed(String::new()));
    }

    #[test]
    fn a_length_mismatch_renders_a_schema_error_rather_than_panicking() {
        let s = line("mixed", "a,b,c", &["1".into()]);
        let l = Line::parse(&s).expect("parses");
        assert!(l.text("schema_error").is_some(), "{s}");
    }

    #[test]
    fn every_kind_has_a_schema_and_no_schema_repeats_a_name() {
        for k in KINDS {
            let s = schema(k).unwrap_or_else(|| panic!("{k} has no schema"));
            let mut sorted = s.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), s.len(), "{k} repeats a field name");
            assert!(
                !s.contains(&"kind"),
                "{k} may not carry a field named `kind`"
            );
        }
    }
}
