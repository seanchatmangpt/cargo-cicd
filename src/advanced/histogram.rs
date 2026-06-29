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
#[derive(Clone, Debug)]
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
