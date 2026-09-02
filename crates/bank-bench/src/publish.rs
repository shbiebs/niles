//! **Where a benchmark's output is allowed to go, and whether two runs are comparable.**
//!
//! Two rules, both of them repairs of defects the harness shipped with, and both stated as
//! functions so a test can hold them rather than a comment asking a reader to be careful.
//!
//! The first: a run writes into its own output directory, and touches the **committed**
//! results artifact only when publication was asked for. `write_all` used to
//! `File::create("results/E16-wallclock.md")` unconditionally, whatever `--out` said, so
//! every exploratory run — an audit's, a bisect's, a CI job's — left the working tree dirty
//! with a document measured under whatever flags that run happened to use.
//!
//! The second: two runs of one target must start from the same base. PostgreSQL was
//! re-prepared between runs and Nilestream was not, so each Nilestream run began with the
//! previous run's appends still in its ledger and the analytical row declined monotonically
//! across five runs. The committed median was the median of a drift, which is a number with
//! no referent.

use std::path::{Path, PathBuf};

/// The committed results document, relative to the repository root.
pub const COMMITTED: &str = "results/E16-wallclock.md";

/// Every path the rendered document is written to.
///
/// The run's own directory always; the committed artifact only under `publish`. A caller
/// cannot reach the committed path without passing the flag, which is the property
/// `an_unpublished_run_writes_nothing_a_repository_tracks` holds.
pub fn destinations(out_dir: &Path, publish: bool) -> Vec<PathBuf> {
    let mut out = vec![out_dir.join("E16-wallclock.md")];
    if publish {
        out.push(PathBuf::from(COMMITTED));
    }
    out
}

/// Whether this run starts from the same base as the target's first run, and what to say if
/// it does not.
///
/// The unit is per target — rows for PostgreSQL, epochs for Nilestream — and is never
/// compared across targets. What matters is that a target answers the same question twice.
pub fn base_drift(target: &str, first: u64, now: u64, run: u32) -> Option<String> {
    if first == now {
        return None;
    }
    Some(format!(
        "{target} starts run {run} from a base of {now} where run 1 started from {first}. \
         The runs are not the same measurement, and their median would be the median of a \
         drift. Re-preparation between runs is what keeps the two targets comparable."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unpublished_run_writes_nothing_a_repository_tracks() {
        // The rule: a benchmark must not touch a committed artifact unless it was asked to
        // publish one. Held as a property of the destination list rather than as care taken
        // at the call site, because the call site is where it was got wrong.
        let dir = Path::new("/tmp/bench-out");
        let unpublished = destinations(dir, false);
        assert_eq!(unpublished.len(), 1);
        assert!(
            !unpublished.iter().any(|p| p.starts_with("results")),
            "an unpublished run reached the committed tree: {unpublished:?}"
        );
        assert!(unpublished[0].starts_with(dir));

        let published = destinations(dir, true);
        assert!(
            published.iter().any(|p| p == Path::new(COMMITTED)),
            "`--publish` must reach the committed document: {published:?}"
        );
        assert!(
            published.iter().any(|p| p.starts_with(dir)),
            "and must still write the run's own copy, so the two can be diffed"
        );
    }

    #[test]
    fn a_run_starting_from_a_different_base_is_reported_with_both_numbers() {
        assert_eq!(base_drift("nilestream", 19_999, 19_999, 2), None);
        let why = base_drift("nilestream", 19_999, 21_249, 2).expect("a drift is reported");
        // Both numbers, because "the base changed" is not actionable and "21249 where run 1
        // started from 19999" is: 1,250 legs is exactly one run of `oltp` plus `durable`.
        assert!(why.contains("19999") && why.contains("21249"), "{why}");
        assert!(why.contains("run 2"), "{why}");
    }
}
