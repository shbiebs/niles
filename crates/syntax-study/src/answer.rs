//! **Answers, canonically** (design §5). A query's answer is a multiset of rows, compared
//! sorted, unless the task says order is part of it (Q05); a transaction's is the multiset of
//! legs it appends. Every value is its text: integers in decimal, a currency by its name, a
//! null as `NULL`. Column names are not part of an answer — they differ by surface and mean
//! nothing to the comparison.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub rows: Vec<Vec<String>>,
    pub ordered: bool,
}

impl Answer {
    pub fn set(mut rows: Vec<Vec<String>>) -> Answer {
        rows.sort();
        Answer {
            rows,
            ordered: false,
        }
    }
    pub fn ordered(rows: Vec<Vec<String>>) -> Answer {
        Answer {
            rows,
            ordered: true,
        }
    }
    /// The same answer, read under `self`'s ordering rule: an executor does not know whether
    /// order matters, the task does.
    pub fn like(&self, rows: Vec<Vec<String>>) -> Answer {
        if self.ordered {
            Answer::ordered(rows)
        } else {
            Answer::set(rows)
        }
    }
    /// A short description of the first difference, for a report.
    pub fn diff(&self, other: &Answer) -> Option<String> {
        if self == other {
            return None;
        }
        if self.rows.len() != other.rows.len() {
            return Some(format!(
                "{} rows expected, {} returned",
                self.rows.len(),
                other.rows.len()
            ));
        }
        for (i, (a, b)) in self.rows.iter().zip(&other.rows).enumerate() {
            if a != b {
                return Some(format!("row {i}: expected {a:?}, got {b:?}"));
            }
        }
        Some("differs".into())
    }
}
