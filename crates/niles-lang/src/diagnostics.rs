//! Diagnostics as a first-class subsystem.
//!
//! rustc's error quality is not a matter of wording; it is architectural. Errors are
//! structured values with a code, a primary span, labelled secondary spans, notes, and
//! *machine-applicable* suggestions, and `--explain` prints a long-form explanation keyed
//! by the code. Niles copies that shape, for a reason specific to this thesis: the
//! interesting errors here are not syntax errors but conservation, currency, linearity,
//! effect and contract errors, and each of those needs to point at two places at once —
//! the money that was created and the rule that forbids it, the hold and its second
//! resolution, the view's declared rung and the effect that exceeds it.
//!
//! A one-span error cannot say that.

use crate::lexer::Span;
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

/// A span with a message attached. The primary label carries the diagnostic's main claim;
/// secondary labels carry the context that makes it a claim rather than an assertion.
#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub msg: String,
    pub primary: bool,
}

/// A fix the tool can apply without asking. Kept separate from a note because the
/// distinction — "here is a thing you could read" versus "here is an edit I can make" —
/// is what lets a formatter, an IDE and a migration tool share one mechanism.
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub span: Span,
    pub replacement: String,
    pub msg: String,
    pub applicability: Applicability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    /// Safe to apply automatically.
    MachineApplicable,
    /// Probably right; needs a human.
    MaybeIncorrect,
    /// Contains a placeholder the human must fill in.
    HasPlaceholders,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    /// A stable code, `NLnnnn`. Stability is the point: a code is what a user searches for
    /// and what `nilesc --explain` looks up, so codes are never reused for a new meaning.
    pub code: &'static str,
    pub msg: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    pub fn error(code: &'static str, msg: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            code,
            msg: msg.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    pub fn warning(code: &'static str, msg: impl Into<String>) -> Self {
        Diagnostic { severity: Severity::Warning, ..Self::error(code, msg) }
    }
    pub fn primary(mut self, span: Span, msg: impl Into<String>) -> Self {
        self.labels.push(Label { span, msg: msg.into(), primary: true });
        self
    }
    pub fn secondary(mut self, span: Span, msg: impl Into<String>) -> Self {
        self.labels.push(Label { span, msg: msg.into(), primary: false });
        self
    }
    pub fn note(mut self, n: impl Into<String>) -> Self {
        self.notes.push(n.into());
        self
    }
    pub fn suggest(
        mut self,
        span: Span,
        replacement: impl Into<String>,
        msg: impl Into<String>,
        a: Applicability,
    ) -> Self {
        self.suggestions.push(Suggestion {
            span,
            replacement: replacement.into(),
            msg: msg.into(),
            applicability: a,
        });
        self
    }
    pub fn span(&self) -> Span {
        self.labels
            .iter()
            .find(|l| l.primary)
            .or_else(|| self.labels.first())
            .map(|l| l.span)
            .unwrap_or_default()
    }
}

/// The sink. Collects, sorts by position, and renders.
#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, d: Diagnostic) {
        self.items.push(d);
    }
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }
    pub fn error_count(&self) -> usize {
        self.items.iter().filter(|d| d.severity == Severity::Error).count()
    }
    pub fn extend(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }
    pub fn sorted(&self) -> Vec<&Diagnostic> {
        let mut v: Vec<&Diagnostic> = self.items.iter().collect();
        v.sort_by_key(|d| (d.span().start, d.severity));
        v
    }

    /// Render every diagnostic against the source, rustc-style: a header line, a gutter
    /// with line numbers, carets under the primary label, and notes below.
    pub fn render(&self, src: &str, filename: &str) -> String {
        let lines: Vec<&str> = src.split('\n').collect();
        let mut out = String::new();
        for d in self.sorted() {
            let _ = writeln!(out, "{}[{}]: {}", d.severity.as_str(), d.code, d.msg);
            for label in &d.labels {
                let (line, col) = line_col(src, label.span.start as usize);
                let text = lines.get(line - 1).copied().unwrap_or("");
                let gutter = format!("{line}");
                let pad = " ".repeat(gutter.len());
                let _ = writeln!(out, "{pad}--> {filename}:{line}:{col}");
                let _ = writeln!(out, "{pad} |");
                let _ = writeln!(out, "{gutter} | {text}");
                let caret = if label.primary { '^' } else { '-' };
                let width = label.span.len().max(1).min(text.len().saturating_sub(col - 1).max(1));
                let _ = writeln!(
                    out,
                    "{pad} | {}{} {}",
                    " ".repeat(col.saturating_sub(1)),
                    caret.to_string().repeat(width),
                    label.msg
                );
            }
            for n in &d.notes {
                let _ = writeln!(out, "  = note: {n}");
            }
            for s in &d.suggestions {
                let _ = writeln!(out, "  = help: {} (`{}`)", s.msg, s.replacement);
            }
            out.push('\n');
        }
        out
    }
}

/// 1-based line and column for a byte offset.
pub fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(src.len());
    let before = &src[..offset];
    let line = before.matches('\n').count() + 1;
    let col = before.rfind('\n').map_or(offset, |i| offset - i - 1) + 1;
    (line, col)
}

/// Levenshtein distance, capped, for "did you mean" suggestions. Cheap and adequate:
/// the candidate sets here are keyword lists and stage lists, in the low hundreds.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// The closest candidate within a sane distance, or nothing. Suggesting a wildly different
/// word is worse than suggesting none: it sends the reader looking for a relationship that
/// is not there.
pub fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (word.len() / 3).max(1) + 1;
    candidates
        .map(|c| (edit_distance(word, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_is_one_based() {
        let src = "abc\ndef\n";
        assert_eq!(line_col(src, 0), (1, 1));
        assert_eq!(line_col(src, 4), (2, 1));
        assert_eq!(line_col(src, 6), (2, 3));
    }

    #[test]
    fn suggestions_are_near_misses_only() {
        let cands = ["where", "map", "group_by"];
        assert_eq!(closest("wher", cands.into_iter()), Some("where"));
        assert_eq!(closest("grup_by", cands.into_iter()), Some("group_by"));
        assert_eq!(closest("quantum", cands.into_iter()), None);
    }

    #[test]
    fn a_diagnostic_can_point_at_two_places() {
        // The property the money errors need: the violation and the rule that forbids it.
        let d = Diagnostic::error("NL0300", "transaction does not conserve `usd`")
            .primary(Span::new(10, 20), "sums to 5.00 usd, not zero")
            .secondary(Span::new(0, 5), "`conserve per (txn, cur)` declared here")
            .note("double-entry requires every currency in a transaction to sum to zero");
        assert_eq!(d.labels.len(), 2);
        assert!(d.labels[0].primary && !d.labels[1].primary);
    }

    #[test]
    fn rendering_points_at_the_right_line() {
        let src = "schema s {\n  bad here\n}\n";
        let mut ds = Diagnostics::new();
        ds.push(Diagnostic::error("NL0001", "unexpected token").primary(Span::new(13, 16), "here"));
        let out = ds.render(src, "t.niles");
        assert!(out.contains("t.niles:2:3"), "{out}");
        assert!(out.contains("^^^"), "{out}");
    }
}
