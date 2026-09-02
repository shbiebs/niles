//! Diagnostics as a first-class subsystem.
//!
//! rustc's error quality is not a matter of wording; it is architectural. Errors are
//! structured values with a code, a primary span, labelled secondary spans, notes, and
//! *machine-applicable* suggestions, and `--explain` prints a long-form explanation keyed
//! by the code. Niles copies that shape, for a reason specific to this thesis: the
//! interesting errors here are not syntax errors but conservation, currency, linearity,
//! effect and contract errors.
//!
//! # Why more than one span, and how strong that claim actually is
//!
//! The theoretical case is the strong one, and it comes from type-error *slicing*. Haack
//! and Wells's position is that the location of a type error is not a point but "a set of
//! program points (a slice) all of which are necessary for the type error", and that
//! algorithms which "identify one node of the program tree which participates in the type
//! error … will often be the wrong node to blame". Zhang and Myers and Chen and Erwig
//! reach the same conclusion from different directions, the latter noting that committing
//! to a single location fails "because in some cases the program text does not contain
//! enough information to confidently make the right decision".
//!
//! That argument transfers here exactly. A conservation violation is constituted by the
//! postings that fail to net *and* the rule that says they must; a double resolution by
//! the binding *and* both consumptions. Reporting one of those is reporting an arbitrary
//! member of a set, and the choice is a heuristic rather than a fact about the program.
//!
//! **What there is no evidence for** is that a *second span* is the right vehicle. No
//! controlled study compares multi-span against single-span diagnostics, for any error
//! class, in any language. The empirical support is indirect: Barik et al. found developers
//! prefer messages with proper argument structure (claim, grounds, warrant) — but only
//! when neither message offers a resolution. And the most-admired compiler diagnostics in
//! the field, Elm's, are largely *single*-region with the counterparty in prose. The
//! honest claim is convergence on dual *reference* — rustc's `required by this bound in
//! …`, GCC's labelled ranges, the Language Server Protocol's `relatedInformation` — with
//! span-versus-note-versus-prose an open rendering question.
//!
//! Two design rules follow, and both are concessions:
//!
//! 1. **The primary span must stand alone.** rustc's own guidance is that a primary label
//!    should make sense "if it were the only thing being displayed". If a Niles diagnostic
//!    is unintelligible without its second span, the primary label is underspecified.
//! 2. **A fix outranks a warrant.** See [`Diagnostic::warrant`].
//!
//! The full evidence review, including the case against this design, is in
//! `docs/research/diagnostics-evidence.md`.

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
    /// Warrants suppressed in favour of a machine-applicable fix. See [`Diagnostic::warrant`].
    elided_warrants: usize,
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
            elided_warrants: 0,
            severity: Severity::Error,
            code,
            msg: msg.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    pub fn warning(code: &'static str, msg: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            ..Self::error(code, msg)
        }
    }
    pub fn primary(mut self, span: Span, msg: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            msg: msg.into(),
            primary: true,
        });
        self
    }
    pub fn secondary(mut self, span: Span, msg: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            msg: msg.into(),
            primary: false,
        });
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
    /// Attach a **warrant**: the rule that makes this a violation, at the place it was
    /// declared.
    ///
    /// Deliberately not the same call as [`Diagnostic::secondary`], because it carries a
    /// design rule the evidence supports and a plain secondary span does not.
    ///
    /// Barik, Ford, Murphy-Hill and Parnin modelled an error message as a Toulmin
    /// argument — claim, grounds, warrant, backing — and found, with 68 professional
    /// developers, that developers prefer proper argument structure *when neither message
    /// offers a resolution*, **but will accept a deficient structure if it provides a
    /// resolution**. That second clause is a constraint on this design, not a footnote:
    /// where a machine-applicable fix exists, the fix is what the developer wants, and a
    /// second span pointing at a rule declaration three files away is cost without benefit.
    ///
    /// So the warrant is *conditional*. It is attached when the diagnostic has no
    /// machine-applicable suggestion, and elided when it has one — while the rule's
    /// identity stays in the message text either way, so nothing is lost, only relocated.
    ///
    /// See `docs/research/diagnostics-evidence.md` §4 for the case against, which is
    /// stronger than it first appears and which this method concedes rather than argues
    /// with.
    pub fn warrant(mut self, span: Span, rule: impl Into<String>) -> Self {
        let has_fix = self
            .suggestions
            .iter()
            .any(|s| s.applicability == Applicability::MachineApplicable);
        if !has_fix {
            self.labels.push(Label {
                span,
                msg: rule.into(),
                primary: false,
            });
        } else {
            self.elided_warrants += 1;
        }
        self
    }

    /// How many warrants were suppressed because a machine-applicable fix was available.
    /// Exposed so a test can check the rule is actually being applied rather than
    /// accidentally never triggering.
    pub fn elided_warrants(&self) -> usize {
        self.elided_warrants
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
        self.items
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
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
                let width = label
                    .span
                    .len()
                    .max(1)
                    .min(text.len().saturating_sub(col - 1).max(1));
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
    fn a_fix_outranks_a_warrant() {
        // Barik et al.'s second clause, as a rule the code obeys: given a machine-applicable
        // resolution, the developer wants the resolution, and a second span pointing at a
        // rule declaration in another file is cost without benefit.
        let with_fix = Diagnostic::error("NL0300", "does not conserve `usd`")
            .primary(Span::new(10, 20), "net movement is 5.00 usd")
            .suggest(
                Span::new(10, 20),
                "post(balancing)",
                "add the balancing posting",
                Applicability::MachineApplicable,
            )
            .warrant(Span::new(0, 5), "`conserve per (txn, cur)` declared here");
        assert_eq!(
            with_fix.labels.len(),
            1,
            "the warrant must yield to the fix"
        );
        assert_eq!(
            with_fix.elided_warrants(),
            1,
            "and the elision must be observable, not silent"
        );

        let without_fix = Diagnostic::error("NL0300", "does not conserve `usd`")
            .primary(Span::new(10, 20), "net movement is 5.00 usd")
            .warrant(Span::new(0, 5), "`conserve per (txn, cur)` declared here");
        assert_eq!(
            without_fix.labels.len(),
            2,
            "with no fix, the warrant is what the reader has"
        );
    }

    #[test]
    fn a_placeholder_suggestion_does_not_displace_the_warrant() {
        // Only a *machine-applicable* fix outranks the rule. A suggestion the developer
        // must fill in themselves is not a resolution in Barik's sense.
        let d = Diagnostic::error("NL0300", "does not conserve `usd`")
            .primary(Span::new(10, 20), "net movement is 5.00 usd")
            .suggest(
                Span::new(10, 20),
                "post(<account>, 5.00 usd)",
                "add a balancing posting",
                Applicability::HasPlaceholders,
            )
            .warrant(Span::new(0, 5), "`conserve per (txn, cur)` declared here");
        assert_eq!(d.labels.len(), 2);
        assert_eq!(d.elided_warrants(), 0);
    }

    #[test]
    fn the_primary_label_stands_alone() {
        // rustc's doctrine, as a property of ours: the primary label must make sense as
        // the only thing displayed, because in an IDE it often is.
        let d = Diagnostic::error("NL0300", "this transaction does not conserve `usd`")
            .primary(
                Span::new(10, 20),
                "net movement on every path through this transaction is -40.00, which must be zero",
            )
            .warrant(Span::new(0, 5), "`conserve per (txn, cur)` declared here");
        let primary = d.labels.iter().find(|l| l.primary).unwrap();
        assert!(
            primary.msg.len() > 40 && primary.msg.contains("-40.00"),
            "a primary label that needs the second span to be understood is underspecified: {}",
            primary.msg
        );
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
