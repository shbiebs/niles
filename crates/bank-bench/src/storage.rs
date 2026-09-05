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

/// **The device's ceiling, probed repeatedly, with its own spread.**
///
/// One probe is not a ceiling on shared or virtualised storage. Measured here, repeated
/// probes seconds apart on this container's device ranged 7,794 to 12,761 durable commits
/// per second — a factor of 1.6 — while nothing about the database changed. A `durable` row
/// published as an absolute number against a device that moves like that is a number about
/// the device, and reading a 10% movement in it as an engine regression is reading noise.
///
/// So the harness probes several times and reports the median, the spread, and — the figure
/// that actually means something — each target's rate as a **fraction of what the device
/// could do**. That last one is machine-independent: "we get 55% of the fsyncs this storage
/// can deliver" is a statement about the engine, and it survives being run somewhere else.
/// **Which durability barrier `File::sync_data` actually issues here.**
///
/// A ceiling is meaningless without it, and the difference is not a detail: the same probe
/// measured **5,300–6,000/s on Linux ext4**, **500–960/s on ext4 inside a VM**, and **255/s on
/// APFS** — and about a million per second on an overlay mounted `fsync=volatile`, which is
/// not storage evidence at all. Two of those numbers differ by 20× because the *disks*
/// differ; one differs by four orders of magnitude because the barrier was not a barrier.
///
/// The macOS case is the one that has bitten this project. `fsync(2)` on APFS does not flush
/// the drive's write cache; `fcntl(F_FULLFSYNC)` does, and Rust's `sync_data` issues the
/// latter. PostgreSQL's *default* `wal_sync_method` on macOS is the former — so on the
/// author's own machine PostgreSQL committed 13,458 durable transactions per second against
/// a 324/s barrier while reporting `fsync=on`, and the two systems were durable against
/// different failures. See [`crate::storage::BARRIER`] and the `wal_sync_method`
/// pre-registration in `bench`.
pub const BARRIER: &str = if cfg!(target_os = "linux") {
    // `sync_data` is `fdatasync(2)`: the data and any metadata needed to read it back.
    "fdatasync"
} else if cfg!(target_vendor = "apple") {
    // `sync_data` falls back to `sync_all`, which is `fcntl(F_FULLFSYNC)` — the real barrier.
    "F_FULLFSYNC"
} else if cfg!(target_os = "windows") {
    "FlushFileBuffers"
} else {
    "fsync"
};

#[derive(Debug, Clone, Copy)]
pub struct Ceiling {
    /// Median durable commits per second the device can sustain, per connection.
    pub median: f64,
    /// Median absolute deviation of the probes, in the same unit.
    pub mad: f64,
    pub lowest: f64,
    pub highest: f64,
    pub probes: u32,
    /// The barrier the probe issued. Carried on the value rather than looked up at print
    /// time so a ceiling cannot be reported beside the wrong one.
    pub barrier: &'static str,
}

impl Ceiling {
    /// The spread as a multiple: highest over lowest.
    ///
    /// The number that says whether an absolute durable figure means anything. Near 1.0 the
    /// device is steady and a rate can be quoted; at 1.6 it cannot, and only the fraction of
    /// the ceiling can.
    pub fn spread(&self) -> f64 {
        if self.lowest <= 0.0 {
            return f64::INFINITY;
        }
        self.highest / self.lowest
    }

    /// Whether the device held still enough for an absolute rate to be worth quoting.
    ///
    /// A tenth of a spread is generous; the point is to catch the case where it is not
    /// close, and to say so in the results rather than leave a reader to wonder.
    pub fn steady(&self) -> bool {
        self.spread() <= 1.1
    }

    /// What fraction of the device's ceiling a measured rate achieved.
    pub fn efficiency(&self, measured_tps: f64) -> f64 {
        if self.median <= 0.0 {
            return 0.0;
        }
        measured_tps / self.median
    }
}

/// Probe the device `probes` times, pausing between, and report the distribution.
///
/// Spaced rather than back to back: a burst measures one moment of the device's mood, and the
/// question is whether its mood changes over the minutes a benchmark takes.
pub fn ceiling(dir: &str, probes: u32, calls: u32) -> std::io::Result<Ceiling> {
    let mut v = Vec::with_capacity(probes as usize);
    for i in 0..probes.max(1) {
        v.push(fsync_cost(dir, calls)?.ceiling_per_second());
        if i + 1 < probes {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = v[v.len() / 2];
    let mut d: Vec<f64> = v.iter().map(|x| (x - median).abs()).collect();
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Ok(Ceiling {
        median,
        mad: d[d.len() / 2],
        lowest: v[0],
        highest: v[v.len() - 1],
        probes: v.len() as u32,
        barrier: BARRIER,
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

    #[test]
    fn a_ceiling_reports_the_devices_spread_and_refuses_to_call_a_moving_device_steady() {
        let steady = Ceiling {
            median: 10_000.0,
            mad: 100.0,
            lowest: 9_800.0,
            highest: 10_200.0,
            probes: 7,
            barrier: BARRIER,
        };
        assert!((steady.spread() - 1.0408).abs() < 0.001);
        assert!(steady.steady(), "a 4% spread is a device holding still");

        let moving = Ceiling {
            median: 10_243.0,
            mad: 804.0,
            lowest: 7_794.0,
            highest: 12_761.0,
            probes: 7,
            barrier: BARRIER,
        };
        assert!((moving.spread() - 1.637).abs() < 0.01);
        assert!(
            !moving.steady(),
            "the container's own storage moved 1.6x between probes seconds apart; a harness \
             that called that steady would be publishing the device's mood as an engine result"
        );
    }

    #[test]
    fn the_fraction_of_the_ceiling_is_what_survives_a_change_of_machine() {
        // The same engine, the same efficiency, on two devices a factor of two apart. The
        // absolute rate says the engine changed; the fraction says it did not. That is the
        // whole reason the `durable` row is published as a fraction.
        let fast = Ceiling {
            median: 20_000.0,
            mad: 0.0,
            lowest: 20_000.0,
            highest: 20_000.0,
            probes: 3,
            barrier: BARRIER,
        };
        let slow = Ceiling {
            median: 10_000.0,
            mad: 0.0,
            lowest: 10_000.0,
            highest: 10_000.0,
            probes: 3,
            barrier: BARRIER,
        };
        assert!((fast.efficiency(9_400.0) - slow.efficiency(4_700.0)).abs() < 1e-9);
        assert_eq!(
            slow.efficiency(0.0),
            0.0,
            "and a target that committed nothing has no share of the device"
        );
    }

    #[test]
    fn probing_the_real_device_returns_a_distribution_rather_than_a_point() {
        // Its own directory: the probe file has a fixed name, and the test above asserts the
        // shared temp dir holds none of them before it starts.
        let dir = std::env::temp_dir().join("bench-ceiling-probe-test");
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        let path = dir
            .to_str()
            .expect("a temp dir with a printable path")
            .to_string();
        let c = ceiling(&path, 3, 20).expect("the device is probeable");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(c.probes, 3);
        assert!(c.median > 0.0, "some rate was measured");
        assert!(
            c.lowest <= c.median && c.median <= c.highest,
            "median {} outside [{}, {}]",
            c.median,
            c.lowest,
            c.highest
        );
        assert!(
            c.spread() >= 1.0,
            "the spread is a multiple, never below one"
        );
    }
}

#[cfg(test)]
mod barrier_tests {
    //! **A ceiling without its barrier is not a number.**
    //!
    //! The same probe measures 4,961–5,825/s on this container's ext4, 500–959/s on ext4
    //! inside a VM, 255/s on APFS through `F_FULLFSYNC`, and about a million per second on
    //! an overlay mounted `fsync=volatile`. Three of those are storage; the fourth is a
    //! mount option. Carrying the barrier on the `Ceiling` value rather than looking it up
    //! when printing is what stops a ceiling being reported beside the wrong one.

    use super::*;

    #[test]
    fn a_ceiling_carries_the_barrier_the_probe_actually_issued() {
        let dir = std::env::temp_dir().join("niles-barrier-test");
        let _ = std::fs::create_dir_all(&dir);
        let c = ceiling(dir.to_str().expect("utf-8"), 2, 8).expect("probe");
        assert_eq!(
            c.barrier, BARRIER,
            "the ceiling must name the barrier this platform's `sync_data` issues"
        );
        assert!(!c.barrier.is_empty());
    }

    /// The constant must match the platform, because it is the label the contract is read
    /// against. `make fsync-proof` checks the other half — that the named call reaches the
    /// kernel — and the two together are what make a durable figure interpretable.
    #[test]
    fn the_barrier_name_matches_the_platform() {
        if cfg!(target_os = "linux") {
            assert_eq!(BARRIER, "fdatasync", "`sync_data` is fdatasync(2) on Linux");
        } else if cfg!(target_vendor = "apple") {
            assert_eq!(
                BARRIER, "F_FULLFSYNC",
                "`sync_data` falls back to `sync_all`, which is fcntl(F_FULLFSYNC), on Apple \
                 platforms — plain fsync(2) does not flush the drive cache on APFS"
            );
        }
    }
}
