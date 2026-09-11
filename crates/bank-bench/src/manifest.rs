//! **The run manifest's schema, owned by the program that owns the measurement.**
//!
//! `c11-baselock.sh` writes one row per stage into `<out>/manifest.csv` as the run happens —
//! deliberately, and deliberately in shell. The manifest exists so that a run which dies
//! halfway still leaves a file saying what it was and how far it got (C11-06.3), and the first
//! stages it records are the *builds*. A manifest written by `bench` could not record that
//! `bench` failed to build, which is the stage most worth recording, so the writer stays where
//! it is.
//!
//! What moves here is the **schema**: the header, the three outcome words, and what each one
//! means. Those were a shell string contract, and a shell string contract is the thing this
//! project has repeatedly had to repair — a renamed column read with `unwrap_or(0)`, a prose
//! grep standing in for a field, two readers disagreeing about one file. `bench
//! --check-manifest <dir>` reads a finished run's manifest and refuses one that does not match,
//! and the test below is the schema's own assertion rather than a sentence about it.
//!
//! # What a manifest can and cannot say
//!
//! A stage that never appears **did not run**. That is the absence the file is designed to
//! make legible, and it is why `NotRun` is a written value rather than a missing row: "the
//! launch guard refused" and "nobody got here" are different facts and the second is not
//! recorded by silence.

/// The header every manifest carries, byte for byte.
pub const HEADER: &str = "stage,outcome,detail";

/// What a stage did. Three words, and the set is closed: an outcome this enum does not name
/// is a row no reader can interpret, which is a refusal rather than a row to skip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The stage ran and did what it was for.
    Ok,
    /// The stage ran and did not. `detail` names where the evidence is.
    Failed,
    /// The stage did not run, and this row says so rather than the row being absent. Its
    /// `detail` is the reason — a refused launch guard, a missing prerequisite.
    NotRun,
}

impl Outcome {
    /// The word as it appears in the file. `not run` has a space; it is written by shell and
    /// quoted there, and changing it to `not_run` would be a schema change, not a tidy-up.
    pub fn word(&self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::Failed => "failed",
            Outcome::NotRun => "not run",
        }
    }

    pub fn parse(s: &str) -> Option<Outcome> {
        match s {
            "ok" => Some(Outcome::Ok),
            "failed" => Some(Outcome::Failed),
            "not run" => Some(Outcome::NotRun),
            _ => None,
        }
    }

    /// Every outcome, so a caller can enumerate rather than remember.
    pub const ALL: [Outcome; 3] = [Outcome::Ok, Outcome::Failed, Outcome::NotRun];
}

/// One row of a run manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub stage: String,
    pub outcome: Outcome,
    pub detail: String,
}

/// What a manifest could not be read as. Each names the line, because a reader who has to
/// find the bad row by eye is a reader who will take the first plausible one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    Empty,
    BadHeader { found: String },
    BadOutcome { line: usize, found: String },
    TooFewFields { line: usize, found: usize },
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::Empty => write!(
                f,
                "the manifest is empty. It is written before anything is built, so an empty \
                 one means the run did not reach the line that writes the header"
            ),
            Problem::BadHeader { found } => write!(
                f,
                "the header is `{found}` and this build writes `{HEADER}`. A reader that \
                 accepted both would be two readers disagreeing about one file"
            ),
            Problem::BadOutcome { line, found } => write!(
                f,
                "line {line} has outcome `{found}`, which is not one of {:?}. An outcome no \
                 reader can interpret is a refusal, not a row to skip",
                Outcome::ALL.map(|o| o.word())
            ),
            Problem::TooFewFields { line, found } => write!(
                f,
                "line {line} has {found} field(s) and the schema is three: {HEADER}"
            ),
        }
    }
}

/// Read a manifest's text into rows, or say why it could not be read.
///
/// `detail` may contain anything the writer put there except a comma or a newline, both of
/// which it replaces with `;` before writing — so this splits on the first two commas only and
/// takes the rest as the detail.
pub fn parse(text: &str) -> Result<Vec<Row>, Problem> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Err(Problem::Empty);
    };
    if header.trim_end() != HEADER {
        return Err(Problem::BadHeader {
            found: header.to_string(),
        });
    }
    let mut out = Vec::new();
    for (i, l) in lines.enumerate() {
        let n = i + 2; // 1-based, and the header is line 1
        if l.trim().is_empty() {
            continue;
        }
        let mut parts = l.splitn(3, ',');
        let (Some(stage), Some(outcome)) = (parts.next(), parts.next()) else {
            return Err(Problem::TooFewFields {
                line: n,
                found: l.split(',').count(),
            });
        };
        let Some(outcome) = Outcome::parse(outcome) else {
            return Err(Problem::BadOutcome {
                line: n,
                found: outcome.to_string(),
            });
        };
        out.push(Row {
            stage: stage.to_string(),
            outcome,
            detail: parts.next().unwrap_or("").to_string(),
        });
    }
    Ok(out)
}

/// Every stage the manifest holds that did not do what it was for. The caller decides what
/// that means — a `NotRun` score stage is normal on a host with no scorer; a `Failed` build
/// is not — so this reports rather than judging.
pub fn not_ok(rows: &[Row]) -> Vec<&Row> {
    rows.iter().filter(|r| r.outcome != Outcome::Ok).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_is_the_one_the_harness_writes() {
        // `c11-baselock.sh` writes this literal. If either side changes it, the other must,
        // and this is the assertion that makes that true rather than hoped.
        assert_eq!(HEADER, "stage,outcome,detail");
    }

    #[test]
    fn every_outcome_round_trips_through_its_written_word() {
        for o in Outcome::ALL {
            assert_eq!(
                Outcome::parse(o.word()),
                Some(o),
                "`{}` did not round-trip",
                o.word()
            );
        }
    }

    #[test]
    fn not_run_keeps_its_space_because_the_harness_writes_it_that_way() {
        // The shell writes `stage "replicate-$tag" "not run" "..."`. An underscore here would
        // be a schema change that looks like a tidy-up, and every archived manifest would stop
        // parsing.
        assert_eq!(Outcome::NotRun.word(), "not run");
    }

    #[test]
    fn a_real_manifest_parses_and_keeps_its_detail_whole() {
        let text = "stage,outcome,detail\n\
                    build-baseline,ok,/tmp/wt a1b2c3 1.95.0\n\
                    replicate-full-1,not run,the launch guard refused\n\
                    score,failed,/tmp/out/score.txt\n";
        let rows = parse(text).expect("a well-formed manifest");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].outcome, Outcome::Ok);
        assert_eq!(rows[1].outcome, Outcome::NotRun);
        assert_eq!(rows[1].detail, "the launch guard refused");
        assert_eq!(rows[2].outcome, Outcome::Failed);
        assert_eq!(not_ok(&rows).len(), 2);
    }

    #[test]
    fn a_detail_containing_a_comma_is_still_one_field() {
        // The writer replaces commas, but a reader that split on every comma would mangle a
        // row the day the writer stopped — and would do it silently, which is the whole
        // reason this schema is here rather than in a `cut -d,` somewhere.
        let rows = parse("stage,outcome,detail\nx,ok,a,b,c\n").expect("parses");
        assert_eq!(rows[0].detail, "a,b,c");
    }

    #[test]
    fn an_unknown_outcome_is_refused_and_the_line_is_named() {
        let e = parse("stage,outcome,detail\nx,skipped,why\n").expect_err("must refuse");
        assert_eq!(
            e,
            Problem::BadOutcome {
                line: 2,
                found: "skipped".into()
            }
        );
        assert!(format!("{e}").contains("line 2"));
    }

    #[test]
    fn a_stale_header_is_refused_rather_than_read_as_this_one() {
        let e = parse("stage,result,detail\nx,ok,y\n").expect_err("must refuse");
        assert!(matches!(e, Problem::BadHeader { .. }));
    }

    #[test]
    fn an_empty_manifest_is_a_refusal_not_an_empty_run() {
        assert_eq!(parse(""), Err(Problem::Empty));
    }
}
