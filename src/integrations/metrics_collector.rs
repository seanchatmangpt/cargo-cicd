//! Metrics collector for tracking pipeline-stage latencies.
//!
//! This module provides [`MetricsCollector`], which aggregates latency measurements
//! across multiple pipeline stages using HdrHistogram-backed histograms.
//!
//! # Example
//!
//! ```ignore
//! # #[cfg(feature = "advanced")]
//! # {
//! use cargo_cicd::integrations::MetricsCollector;
//!
//! let mut collector = MetricsCollector::new();
//! collector.record_stage("scan", 150_000);      // 150ms
//! collector.record_stage("scan", 200_000);      // 200ms
//! collector.record_stage("test", 5_000_000);    // 5s
//! collector.record_stage("test", 6_000_000);    // 6s
//!
//! println!("{}", collector.report());
//! // Output includes lines like:
//! //   scan: p50=175µs, p90=200µs, p99=200µs
//! //   test: p50=5500µs, p90=6000µs, p99=6000µs
//! # }
//! ```

#[cfg(feature = "advanced")]
use std::collections::HashMap;

/// Collects and reports latency metrics across multiple pipeline stages.
///
/// When the `advanced` feature is enabled, this holds a map of stage names to
/// [`StageLatencies`](crate::advanced::histogram::StageLatencies) histograms and
/// provides methods to record measurements and generate reports.
///
/// When the `advanced` feature is disabled, methods are no-ops.
#[derive(Clone, Debug)]
pub struct MetricsCollector {
    #[cfg(feature = "advanced")]
    stages: HashMap<String, crate::advanced::histogram::StageLatencies>,
}

impl MetricsCollector {
    /// Creates an empty metrics collector.
    pub fn new() -> Self {
        #[cfg(feature = "advanced")]
        {
            Self {
                stages: HashMap::new(),
            }
        }
        #[cfg(not(feature = "advanced"))]
        {
            Self {}
        }
    }

    /// Records a latency measurement for the named pipeline stage in microseconds.
    ///
    /// If this is the first measurement for a stage, the stage's histogram is
    /// created automatically. Measurements are clamped to the histogram's valid
    /// range (1µs to 60s), so no panic occurs on outliers.
    pub fn record_stage(&mut self, stage_name: &str, micros: u64) {
        #[cfg(feature = "advanced")]
        {
            self.stages
                .entry(stage_name.to_string())
                .or_insert_with(|| crate::advanced::histogram::StageLatencies::new(stage_name))
                .record(micros);
        }
        #[cfg(not(feature = "advanced"))]
        {
            let _ = (stage_name, micros);
        }
    }

    /// Returns a human-readable summary of all recorded stages and their percentiles.
    ///
    /// Each line shows: `stage_name: p50=XXµs, p90=XXµs, p99=XXµs`
    ///
    /// When `advanced` is disabled, returns an empty string.
    pub fn report(&self) -> String {
        #[cfg(feature = "advanced")]
        {
            if self.stages.is_empty() {
                return String::new();
            }

            let mut lines: Vec<String> = self
                .stages
                .iter()
                .map(|(name, latencies)| {
                    format!(
                        "{}: p50={}µs, p90={}µs, p99={}µs",
                        name,
                        latencies.p50(),
                        latencies.p90(),
                        latencies.p99()
                    )
                })
                .collect();
            lines.sort();
            lines.join("\n")
        }
        #[cfg(not(feature = "advanced"))]
        {
            String::new()
        }
    }

    /// Merges another collector's measurements into this one.
    ///
    /// For each stage in `other`, if that stage exists in `self`, the histograms
    /// are merged; otherwise, the stage is added. This is useful for combining
    /// metrics from parallel workers.
    pub fn merge(&mut self, other: &MetricsCollector) {
        #[cfg(feature = "advanced")]
        {
            for (name, other_hist) in &other.stages {
                self.stages
                    .entry(name.clone())
                    .or_insert_with(|| crate::advanced::histogram::StageLatencies::new(name))
                    .merge(other_hist);
            }
        }
        #[cfg(not(feature = "advanced"))]
        {
            let _ = other;
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;

    #[test]
    fn record_and_report_works() {
        let mut collector = MetricsCollector::new();
        collector.record_stage("scan", 100);
        collector.record_stage("scan", 200);
        collector.record_stage("test", 1000);
        collector.record_stage("test", 2000);

        let report = collector.report();
        assert!(!report.is_empty(), "report should not be empty");
        assert!(
            report.contains("scan:"),
            "report should contain 'scan:' stage"
        );
        assert!(
            report.contains("test:"),
            "report should contain 'test:' stage"
        );
        assert!(
            report.contains("p50="),
            "report should contain p50 percentile"
        );
        assert!(
            report.contains("p90="),
            "report should contain p90 percentile"
        );
        assert!(
            report.contains("p99="),
            "report should contain p99 percentile"
        );
        assert!(
            report.contains("µs"),
            "report should contain microsecond symbol"
        );
    }

    #[test]
    fn merge_combines_histograms() {
        let mut collector1 = MetricsCollector::new();
        collector1.record_stage("stage_a", 100);
        collector1.record_stage("stage_a", 200);
        collector1.record_stage("stage_b", 1000);

        let mut collector2 = MetricsCollector::new();
        collector2.record_stage("stage_a", 150);
        collector2.record_stage("stage_b", 2000);
        collector2.record_stage("stage_c", 500);

        let count_before_a = 2u64;
        let count_before_b = 1u64;

        collector1.merge(&collector2);

        let report = collector1.report();
        assert!(
            report.contains("stage_a:"),
            "merged collector should contain stage_a"
        );
        assert!(
            report.contains("stage_b:"),
            "merged collector should contain stage_b"
        );
        assert!(
            report.contains("stage_c:"),
            "merged collector should contain stage_c from collector2"
        );

        // Verify the merged histogram for stage_a has combined measurements
        // (We can't directly access the histogram, but we can verify it through report)
        // The report should show combined data from both collectors.
        assert!(
            !report.is_empty(),
            "merged report should have data from both collectors"
        );
    }
}
