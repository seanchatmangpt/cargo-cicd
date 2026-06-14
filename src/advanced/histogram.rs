//! Latency percentile tracking for engine pipeline stages.
//!
//! This module provides [`StageLatencies`], a thin wrapper around an
//! [`hdrhistogram::Histogram`] tuned for instrumenting the duration of a single
//! pipeline stage in the cargo-cicd process-data engine. Durations are recorded
//! in **microseconds** so that sub-millisecond stages (cache lookups, fingerprint
//! comparisons) and multi-second stages (full workspace scans) share one model.
//!
//! Each stage owns its own histogram. Recorded samples can be summarized into a
//! [`Percentiles`] snapshot, and histograms from independent workers can be
//! [`merge`](StageLatencies::merge)d to aggregate latency across a parallel run.

use std::time::Duration;

use hdrhistogram::Histogram;

/// Lowest recordable value in microseconds (1 microsecond).
const LOW_MICROS: u64 = 1;
/// Highest recordable value in microseconds (60 seconds).
const HIGH_MICROS: u64 = 60_000_000;
/// Significant figures of precision retained by the underlying histogram.
const SIGFIGS: u8 = 3;

/// Records and summarizes the latency distribution of a single pipeline stage.
///
/// Durations are stored in microseconds in an HdrHistogram bounded to
/// `1us..=60s` at three significant figures of precision, which keeps the
/// memory footprint small while preserving accurate high-percentile values.
#[derive(Clone)]
pub struct StageLatencies {
    stage: String,
    hist: Histogram<u64>,
}

impl StageLatencies {
    /// Creates an empty latency tracker for the named pipeline `stage`.
    ///
    /// The histogram is bounded to `1us..=60s`. Values outside that range are
    /// clamped on record so instrumentation never panics on an outlier.
    pub fn new(stage: &str) -> Self {
        // new_with_bounds only fails for nonsensical bound/sigfig combinations,
        // which are fixed constants here, so a failure is a programming error.
        let hist = Histogram::<u64>::new_with_bounds(LOW_MICROS, HIGH_MICROS, SIGFIGS)
            .expect("valid histogram bounds for stage latency tracking");
        Self {
            stage: stage.to_string(),
            hist,
        }
    }

    /// Returns the name of the stage this tracker instruments.
    pub fn stage(&self) -> &str {
        &self.stage
    }

    /// Records a single observed duration in microseconds.
    ///
    /// The value is clamped into the recordable range, so callers do not need
    /// to pre-validate outliers.
    pub fn record(&mut self, micros: u64) {
        let clamped = micros.clamp(LOW_MICROS, HIGH_MICROS);
        // saturating_record can only error on a value outside auto-resize
        // bounds; clamping above guarantees an in-range value.
        self.hist.saturating_record(clamped);
    }

    /// Records an observed [`Duration`], converting it to microseconds.
    pub fn record_duration(&mut self, d: Duration) {
        let micros = d.as_micros().min(u64::MAX as u128) as u64;
        self.record(micros);
    }

    /// Returns the median (50th percentile) latency in microseconds.
    pub fn p50(&self) -> u64 {
        self.hist.value_at_quantile(0.50)
    }

    /// Returns the 90th percentile latency in microseconds.
    pub fn p90(&self) -> u64 {
        self.hist.value_at_quantile(0.90)
    }

    /// Returns the 99th percentile latency in microseconds.
    pub fn p99(&self) -> u64 {
        self.hist.value_at_quantile(0.99)
    }

    /// Returns the maximum recorded latency in microseconds.
    pub fn max(&self) -> u64 {
        self.hist.max()
    }

    /// Returns the arithmetic mean of recorded latencies in microseconds.
    pub fn mean(&self) -> f64 {
        self.hist.mean()
    }

    /// Returns the number of samples recorded.
    pub fn count(&self) -> u64 {
        self.hist.len()
    }

    /// Merges another stage's samples into this one.
    ///
    /// Useful for aggregating per-worker histograms after a parallel run. The
    /// stage name of `self` is preserved.
    pub fn merge(&mut self, other: &StageLatencies) {
        // add only fails when the source contains values outside this
        // histogram's range; both use identical fixed bounds.
        self.hist
            .add(&other.hist)
            .expect("merging histograms with identical bounds");
    }

    /// Captures a [`Percentiles`] snapshot of the current distribution.
    pub fn percentiles(&self) -> Percentiles {
        Percentiles {
            count: self.count(),
            p50: self.p50(),
            p90: self.p90(),
            p99: self.p99(),
            max: self.max(),
            mean: self.mean(),
        }
    }
}

/// An immutable snapshot of a stage's latency distribution.
///
/// All latency fields are in microseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentiles {
    /// Number of samples recorded.
    pub count: u64,
    /// Median latency.
    pub p50: u64,
    /// 90th percentile latency.
    pub p90: u64,
    /// 99th percentile latency.
    pub p99: u64,
    /// Maximum recorded latency.
    pub max: u64,
    /// Mean latency.
    pub mean: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records the uniform distribution 1..=1000 microseconds.
    fn uniform_1_to_1000(stage: &str) -> StageLatencies {
        let mut lat = StageLatencies::new(stage);
        for micros in 1..=1000u64 {
            lat.record(micros);
        }
        lat
    }

    #[test]
    fn percentiles_track_known_distribution() {
        let lat = uniform_1_to_1000("scan");
        // For a uniform 1..=1000 distribution the p50 sits near 500 and the
        // p99 near 990. HdrHistogram quantization means we assert ranges, not
        // exact values, to avoid flakiness.
        assert!(
            (495..=505).contains(&lat.p50()),
            "p50 should be near 500, got {}",
            lat.p50()
        );
        assert!(
            (985..=995).contains(&lat.p99()),
            "p99 should be near 990, got {}",
            lat.p99()
        );
        assert!(
            (895..=905).contains(&lat.p90()),
            "p90 should be near 900, got {}",
            lat.p90()
        );
    }

    #[test]
    fn max_and_count_are_exact() {
        let lat = uniform_1_to_1000("scan");
        assert_eq!(lat.count(), 1000);
        // Max is 1000 within histogram tolerance at this scale.
        assert!(
            (999..=1001).contains(&lat.max()),
            "max should be ~1000, got {}",
            lat.max()
        );
        // Mean of 1..=1000 is 500.5.
        assert!(
            (498.0..=503.0).contains(&lat.mean()),
            "mean should be near 500.5, got {}",
            lat.mean()
        );
    }

    #[test]
    fn merge_combines_count_and_takes_larger_max() {
        let mut small = StageLatencies::new("a");
        for micros in 1..=500u64 {
            small.record(micros);
        }
        let mut large = StageLatencies::new("b");
        for micros in 1..=2000u64 {
            large.record(micros);
        }
        let small_count = small.count();
        let large_max = large.max();

        small.merge(&large);

        assert_eq!(small.count(), small_count + 2000);
        // Merged max equals the larger histogram's max within tolerance.
        assert!(
            (1990..=2010).contains(&small.max()),
            "merged max should match larger histogram (~2000), got {}",
            small.max()
        );
        assert!(small.max() >= large_max - 10);
        // Stage name of the receiver is preserved.
        assert_eq!(small.stage(), "a");
    }

    #[test]
    fn percentiles_snapshot_matches_queries() {
        let lat = uniform_1_to_1000("snapshot-stage");
        let snap = lat.percentiles();
        assert_eq!(snap.count, lat.count());
        assert_eq!(snap.p50, lat.p50());
        assert_eq!(snap.p90, lat.p90());
        assert_eq!(snap.p99, lat.p99());
        assert_eq!(snap.max, lat.max());
        assert_eq!(snap.mean, lat.mean());
    }

    #[test]
    fn empty_tracker_has_zero_count() {
        let lat = StageLatencies::new("idle");
        assert_eq!(lat.count(), 0);
    }

    #[test]
    fn record_duration_converts_to_micros() {
        let mut lat = StageLatencies::new("timed");
        lat.record_duration(Duration::from_millis(2)); // 2000 micros
        assert!(
            (1990..=2010).contains(&lat.max()),
            "2ms should record as ~2000 micros, got {}",
            lat.max()
        );
        assert_eq!(lat.count(), 1);
    }
}
