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

/// The committed scaling document (E19), and the default output directories.
pub const COMMITTED_SCALING: &str = "results/E19-scaling.md";
pub const DEFAULT_OUT: &str = "results/E16-wallclock";
pub const DEFAULT_SCALING_OUT: &str = "results/E19-scaling";

/// The committed E23 document, and its default CSV directory. Same rule as E19's.
pub const COMMITTED_E23: &str = "results/E23-scaling.md";
pub const DEFAULT_E23_OUT: &str = "results/E23-scaling";

/// Where an E23 sweep's CSV goes, given `--out` and whether this run publishes. Same gate as
/// [`scaling_dir`], and for the same reason.
pub fn e23_dir(out: &Path, publish: bool) -> PathBuf {
    if publish && out == Path::new(DEFAULT_OUT) {
        PathBuf::from(DEFAULT_E23_OUT)
    } else {
        out.join("E23-scaling")
    }
}

/// Every path the E23 document is written to. Same rule as [`destinations`].
pub fn e23_destinations(dir: &Path, publish: bool) -> Vec<PathBuf> {
    let mut out = vec![dir.join("E23-scaling.md")];
    if publish {
        out.push(PathBuf::from(COMMITTED_E23));
    }
    out
}

/// Where a scaling run's CSVs go, given `--out` and whether this run publishes.
///
/// The committed layout — E19 beside E16 in `results/` — is reached **only when
/// publishing**, which is the same gate the documents have and the rule this project states:
/// *benchmarks touch committed artefacts only with `--publish`*. It was not true of the
/// CSVs. `scaling_dir` mapped the default `--out` to the committed directory unconditionally,
/// so an exploratory run with no `--publish` at all overwrote `results/E19-scaling/*.csv` and
/// left the tree dirty. On Host C that is exactly what happened: a diagnostic run rewrote
/// four committed files and added a fifth, and the next run's retarget-and-refuse clause
/// stopped the whole script — correctly, and for a reason nobody had put there on purpose.
///
/// Any other `--out` keeps the scaling run inside it either way, so a baseline capture, a
/// bisect or a CI job still cannot reach the committed tree by forgetting that E19 has a
/// directory of its own.
pub fn scaling_dir(out: &Path, publish: bool) -> PathBuf {
    if publish && out == Path::new(DEFAULT_OUT) {
        PathBuf::from(DEFAULT_SCALING_OUT)
    } else {
        out.join("E19-scaling")
    }
}

/// Every path the scaling document is written to. Same rule as [`destinations`].
pub fn scaling_destinations(dir: &Path, publish: bool) -> Vec<PathBuf> {
    let mut out = vec![dir.join("E19-scaling.md")];
    if publish {
        out.push(PathBuf::from(COMMITTED_SCALING));
    }
    out
}

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
    fn a_scaling_run_under_a_custom_out_stays_out_of_the_committed_tree() {
        // The same rule as the contract document, and it needs its own test because E19 has
        // its own default directory: a caller who passed `--out /tmp/...` and nothing else
        // would otherwise have written `results/E19-scaling/*.csv` from an exploratory run.
        let scratch = Path::new("/tmp/wo4/t02-before");
        let dir = scaling_dir(scratch, false);
        assert!(
            dir.starts_with(scratch),
            "a custom --out must contain its own scaling directory, got {dir:?}"
        );
        assert!(
            !scaling_destinations(&dir, false)
                .iter()
                .any(|p| p.starts_with("results")),
            "an unpublished scaling run reached the committed tree"
        );
        assert!(scaling_destinations(&dir, true)
            .iter()
            .any(|p| p == Path::new(COMMITTED_SCALING)));

        // And the default `--out` puts E19 beside E16 rather than inside it — **when
        // publishing**: the two are separate experiments and their CSVs must not share a
        // directory, and the committed directory is reached only by a run that says it is
        // publishing.
        assert_eq!(
            scaling_dir(Path::new(DEFAULT_OUT), true),
            Path::new(DEFAULT_SCALING_OUT)
        );
    }

    /// **A run that does not publish may not write a committed CSV — the rule, not nearly
    /// the rule.**
    ///
    /// `scaling_dir` mapped the *default* `--out` to `results/E19-scaling` whatever the run
    /// was doing, so a diagnostic run with no `--publish` overwrote four committed files and
    /// added a fifth. It happened on Host C, and the next run's retarget-and-refuse clause
    /// stopped the script — which is the only reason anybody saw it.
    #[test]
    fn a_default_out_without_publish_stays_out_of_the_committed_tree() {
        for (dir, what) in [
            (scaling_dir(Path::new(DEFAULT_OUT), false), "E19"),
            (e23_dir(Path::new(DEFAULT_OUT), false), "E23"),
        ] {
            assert_ne!(
                dir,
                Path::new(DEFAULT_SCALING_OUT),
                "{what} reached the committed directory without --publish"
            );
            assert!(
                dir.starts_with(DEFAULT_OUT),
                "{what} without --publish must stay under --out, got {dir:?}"
            );
        }
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

    #[test]
    fn an_unpublished_e23_sweep_writes_nothing_a_repository_tracks() {
        let scratch = Path::new("/tmp/e23-scratch");
        assert!(!e23_destinations(&e23_dir(scratch, false), false)
            .iter()
            .any(|p| p == Path::new(COMMITTED_E23)));
        assert!(e23_destinations(&e23_dir(scratch, true), true)
            .iter()
            .any(|p| p == Path::new(COMMITTED_E23)));
        // The default `--out` puts E23 beside E16 and E19 rather than inside E16's
        // directory, which is where it landed the first time and where its CSV would have
        // been mistaken for one of E16's.
        assert_eq!(
            e23_dir(Path::new(DEFAULT_OUT), true),
            Path::new(DEFAULT_E23_OUT)
        );
        assert_eq!(e23_dir(scratch, false), scratch.join("E23-scaling"));
        // Three experiments, three directories, none of them a prefix of another's file.
        assert_ne!(
            e23_dir(Path::new(DEFAULT_OUT), true),
            Path::new(DEFAULT_OUT)
        );
        assert_ne!(
            e23_dir(Path::new(DEFAULT_OUT), true),
            scaling_dir(Path::new(DEFAULT_OUT), true)
        );
    }
}
