//! What the storage can actually do — the only honest denominator for a durability claim.
//!
//! # Why an absolute throughput figure is the wrong calibration
//!
//! The first version of this harness calibrated against a published number: PostgreSQL 18.1
//! at 333 transactions per second per core, from
//! `docs/research/performance-baselines.md`. The gate fired immediately — the measured rate
//! was 16× that — and the investigation is worth recording, because the gate was right and
//! the calibration was wrong.
//!
//! A durable commit is an `fsync`, and an `fsync` costs whatever the device charges. The same
//! literature records the spread: **1.6–12.4µs** on a device with power-loss protection and
//! **891–2974µs** without. That is a factor of a thousand, and it sits directly under the
//! transaction rate. A machine whose `fsync` costs 3ms cannot exceed ~333 durable commits per
//! second per connection no matter how good the database is; a machine whose `fsync` costs
//! 100µs cannot be *held* to 333 without the harness being wrong about what it measured.
//!
//! So 333 txn/s is not a property of PostgreSQL. It is a property of PostgreSQL **on a
//! ~3ms-fsync device**, and calibrating against it on different storage tests the storage.
//!
//! # What this module calibrates against instead
//!
//! The device's own `fsync` ceiling, measured here, on the same filesystem, moments before
//! the benchmark runs. A single-connection durable workload must land **below** that ceiling
//! (it cannot commit faster than it can sync) and **not far** below it (or the harness is not
//! committing per transaction). Both bounds are machine-independent, which is the property
//! the published figure did not have.
//!
//! The two failure modes this actually catches:
//!
//! * **Far below the ceiling** — the loop is not doing the work, or every statement is paying
//!   for something the benchmark did not mean to measure.
//! * **Above the ceiling** — the commits are not reaching storage. With one connection there
//!   is no group commit to explain it, so this is a `synchronous_commit = off` that somebody
//!   forgot to mention, and it is the single most common way a durability number is inflated.

use std::io::Write;
use std::time::Instant;

/// What one device charges for a durable write.
#[derive(Debug, Clone, Copy)]
pub struct FsyncCost {
    pub per_call_us: f64,
    pub calls: u32,
}

impl FsyncCost {
    /// The most durable commits per second one connection could possibly achieve.
    pub fn ceiling_per_second(&self) -> f64 {
        if self.per_call_us <= 0.0 {
            return f64::INFINITY;
        }
        1_000_000.0 / self.per_call_us
    }

    /// Where this device sits in the range the literature reports.
    ///
    /// Named rather than numeric, because the useful question about a benchmark machine is
    /// which *class* of storage it has: a result from a power-loss-protected NVMe and one
    /// from a consumer SSD are not comparable, and a reader deserves to be told which they
    /// are looking at.
    pub fn device_class(&self) -> &'static str {
        match self.per_call_us {
            c if c < 50.0 => "power-loss-protected or write-cached (fsync under 50µs)",
            c if c < 500.0 => "fast NVMe or virtualised block device (fsync 50–500µs)",
            c if c < 5_000.0 => "consumer SSD or networked storage (fsync 0.5–5ms)",
            _ => "slow or heavily contended storage (fsync above 5ms)",
        }
    }
}

/// Measure the cost of an `fsync` on the filesystem holding `dir`.
///
/// Writes and syncs a single page repeatedly. Deliberately the same page rather than an
/// append: a growing file also pays for metadata and block allocation, and the number wanted
/// here is the cost of the *durability barrier* alone, which is what a commit pays.
///
/// The file is removed afterwards, and the measurement takes about a fifth of a second.
pub fn fsync_cost(dir: &str, calls: u32) -> std::io::Result<FsyncCost> {
    let path = std::path::Path::new(dir).join(format!(
        "bench-fsync-probe-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    let page = [0u8; 4096];

    // One untimed pass, so the first call's file creation and any lazy allocation are not
    // charged to the measurement.
    f.write_all(&page)?;
    f.sync_data()?;

    let started = Instant::now();
    for _ in 0..calls {
        use std::io::Seek;
        f.seek(std::io::SeekFrom::Start(0))?;
        f.write_all(&page)?;
        f.sync_data()?;
    }
    let elapsed = started.elapsed();
    drop(f);
    let _ = std::fs::remove_file(&path);

    Ok(FsyncCost {
        per_call_us: elapsed.as_nanos() as f64 / 1000.0 / calls.max(1) as f64,
        calls,
    })
}

/// Whether a measured durable-commit rate is consistent with the device it ran on.
///
/// Returns `Ok(())` when the rate sits in the plausible band, and an explanation when it does
/// not. The bounds are wide because they are meant to catch a broken harness rather than to
/// grade a database.
pub fn plausible(measured_tps: f64, cost: FsyncCost) -> Result<(), String> {
    let ceiling = cost.ceiling_per_second();
    if measured_tps > ceiling * 1.1 {
        return Err(format!(
            "{measured_tps:.0} durable commits per second exceeds this device's fsync ceiling \
             of {ceiling:.0}/s ({:.0}µs per call). One connection has no group commit to \
             explain that, so the commits are not reaching storage — check \
             `synchronous_commit` and `fsync`",
            cost.per_call_us
        ));
    }
    if measured_tps < ceiling * 0.05 {
        return Err(format!(
            "{measured_tps:.0} durable commits per second is under 5% of this device's fsync \
             ceiling of {ceiling:.0}/s. The harness is paying for something other than the \
             commit, or the loop is not doing the work it claims"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_probe_measures_something_and_cleans_up_after_itself() {
        let dir = std::env::temp_dir();
        let before: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("bench-fsync-probe")
            })
            .collect();
        assert!(before.is_empty(), "a previous run left a probe file behind");

        let cost = fsync_cost(dir.to_str().unwrap(), 20).expect("probe");
        assert!(cost.per_call_us > 0.0, "an fsync takes non-zero time");
        assert!(cost.ceiling_per_second().is_finite());

        let after: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("bench-fsync-probe")
            })
            .collect();
        assert!(after.is_empty(), "the probe file was removed");
    }

    #[test]
    fn a_rate_above_the_devices_ceiling_is_refused_and_says_why() {
        // The most common way a durability number is inflated: `synchronous_commit = off`
        // that nobody mentioned. With one connection there is no group commit to explain a
        // rate above the ceiling, so this is a hard refusal rather than a note.
        let cost = FsyncCost {
            per_call_us: 1_000.0,
            calls: 100,
        }; // 1000/s ceiling
        let e = plausible(5_000.0, cost).unwrap_err();
        assert!(e.contains("not reaching storage"), "{e}");
        assert!(e.contains("synchronous_commit"), "{e}");
    }

    #[test]
    fn a_rate_far_below_the_ceiling_is_refused_as_a_broken_harness() {
        let cost = FsyncCost {
            per_call_us: 100.0,
            calls: 100,
        }; // 10,000/s ceiling
        let e = plausible(50.0, cost).unwrap_err();
        assert!(e.contains("not doing the work"), "{e}");
    }

    #[test]
    fn a_plausible_rate_passes_on_both_fast_and_slow_storage() {
        // The property the published-figure calibration did not have: the same rule works on
        // a 3ms device and a 100µs one, because the denominator is the device.
        let slow = FsyncCost {
            per_call_us: 3_000.0,
            calls: 100,
        }; // ~333/s
        assert!(
            plausible(333.0, slow).is_ok(),
            "the published figure, on its own storage"
        );
        assert!(plausible(300.0, slow).is_ok());

        let fast = FsyncCost {
            per_call_us: 109.0,
            calls: 100,
        }; // ~9,170/s
        assert!(
            plausible(5_325.0, fast).is_ok(),
            "and this machine, on its storage"
        );
        assert!(
            plausible(333.0, fast).is_err(),
            "while 333/s on fast storage is the harness being wrong, which is the point"
        );
    }

    #[test]
    fn the_device_class_names_what_a_reader_needs_to_compare_two_results() {
        assert!(FsyncCost {
            per_call_us: 5.0,
            calls: 1
        }
        .device_class()
        .contains("power-loss"));
        assert!(FsyncCost {
            per_call_us: 109.0,
            calls: 1
        }
        .device_class()
        .contains("NVMe"));
        assert!(FsyncCost {
            per_call_us: 2_000.0,
            calls: 1
        }
        .device_class()
        .contains("consumer"));
        assert!(FsyncCost {
            per_call_us: 9_000.0,
            calls: 1
        }
        .device_class()
        .contains("slow"));
    }
}
