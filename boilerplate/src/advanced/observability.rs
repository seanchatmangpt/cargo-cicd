//! Structured span instrumentation for pipeline stages via [`tracing`].
//!
//! Provides a thin, ergonomic layer over `tracing` that:
//! - initialises a JSON subscriber exactly once per process,
//! - wraps pipeline work in RAII span guards that record elapsed time on exit,
//! - emits structured events with `stage`, `success`, and `timestamp_ms`.
//!
//! # Example
//!
//! ```rust,ignore
//! use my_crate::advanced::observability::{init_tracing, with_stage, record_event};
//!
//! // Call once at process start.
//! init_tracing();
//!
//! // Wrap a unit of work in a named stage.
//! let result = with_stage("cargo_metadata_scan", || {
//!     // ... do work ...
//!     42_u64
//! });
//!
//! // Emit a one-shot event without a span.
//! record_event("git_phase_check", true);
//! ```

use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{info, info_span, Span};

/// Global initialisation guard — ensures the subscriber is registered at most
/// once even if `init_tracing` is called from multiple threads.
static TRACING_INIT: OnceLock<()> = OnceLock::new();

/// Initialise the global `tracing` subscriber with JSON formatting and an
/// `RUST_LOG`-controlled filter.
///
/// Idempotent: subsequent calls are no-ops.  Safe to call from tests.
///
/// The subscriber writes to `stderr` in JSON format so it does not interfere
/// with structured `stdout` output from noun verbs.
pub fn init_tracing() {
    TRACING_INIT.get_or_init(|| {
        // Use `try_init` so that tests which install their own subscriber
        // (e.g., `tracing_subscriber::fmt::try_init()`) do not cause a panic.
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_writer(std::io::stderr)
            .try_init();
    });
}

// ─── PipelineStage ───────────────────────────────────────────────────────────

/// RAII guard that creates a `tracing` span on construction and records the
/// elapsed duration (in milliseconds) as a structured field when dropped.
///
/// # Example
///
/// ```rust,ignore
/// {
///     let _stage = PipelineStage::enter("target_scan");
///     // ... do work ...
/// } // span exits here; elapsed_ms is recorded
/// ```
pub struct PipelineStage {
    name: &'static str,
    started_at: Instant,
    /// The entered span guard.  Held here so it is dropped at the same time as
    /// the `PipelineStage` itself.
    _guard: tracing::span::EnteredSpan,
}

impl PipelineStage {
    /// Enter a new span named `name`.
    ///
    /// The span is active (entered) for as long as the returned `PipelineStage`
    /// value is live.
    pub fn enter(name: &'static str) -> Self {
        let span: Span = info_span!("pipeline_stage", stage = name);
        let guard = span.entered();
        Self {
            name,
            started_at: Instant::now(),
            _guard: guard,
        }
    }

    /// Return how long this stage has been running so far.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

impl Drop for PipelineStage {
    fn drop(&mut self) {
        let elapsed_ms = self.started_at.elapsed().as_millis() as u64;
        info!(
            stage = self.name,
            elapsed_ms = elapsed_ms,
            "stage complete"
        );
    }
}

// ─── record_event ────────────────────────────────────────────────────────────

/// Emit a structured `tracing` event recording a stage outcome.
///
/// Fields emitted:
/// - `stage` — the name passed in,
/// - `success` — whether the stage succeeded,
/// - `timestamp_ms` — milliseconds since the Unix epoch.
pub fn record_event(stage: &str, success: bool) {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64;

    info!(
        stage = stage,
        success = success,
        timestamp_ms = timestamp_ms,
        "pipeline event"
    );
}

// ─── with_stage ──────────────────────────────────────────────────────────────

/// Run `f` inside a named [`PipelineStage`] span and return its result.
///
/// The span is entered before `f` is called and exits (recording `elapsed_ms`)
/// when `f` returns, regardless of whether `f` panics.
///
/// # Example
///
/// ```rust,ignore
/// let workspace_name = with_stage("workspace_scan", || {
///     CargoMetadataAdapter::workspace_name()
/// });
/// ```
pub fn with_stage<F, T>(name: &'static str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let _stage = PipelineStage::enter(name);
    f()
}

// ─── StageTimer ──────────────────────────────────────────────────────────────

/// A lightweight timer for measuring stage latency without activating a
/// `tracing` span.  Use when `tracing` is disabled or when you need to
/// capture elapsed time for metrics without emitting a log entry.
#[derive(Debug)]
pub struct StageTimer {
    label: String,
    started_at: Instant,
}

impl StageTimer {
    /// Start a new timer labelled `label`.
    pub fn start(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            started_at: Instant::now(),
        }
    }

    /// Return elapsed time since the timer was started.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Return elapsed milliseconds (truncated).
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// Return the label supplied at construction.
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // init_tracing is idempotent — call many times, no panic.
    #[test]
    fn init_tracing_is_idempotent() {
        init_tracing();
        init_tracing();
        init_tracing();
    }

    #[test]
    fn pipeline_stage_enter_and_drop_does_not_panic() {
        init_tracing();
        {
            let _stage = PipelineStage::enter("test_stage");
        }
        // If we get here without panicking, the test passes.
    }

    #[test]
    fn pipeline_stage_elapsed_is_non_negative() {
        let stage = PipelineStage::enter("elapsed_test");
        // elapsed() must return a non-negative duration.
        assert!(stage.elapsed() >= Duration::ZERO);
    }

    #[test]
    fn record_event_does_not_panic() {
        init_tracing();
        record_event("test_event_success", true);
        record_event("test_event_failure", false);
    }

    #[test]
    fn with_stage_returns_closure_result() {
        init_tracing();
        let result = with_stage("compute", || 42_u64);
        assert_eq!(result, 42);
    }

    #[test]
    fn with_stage_propagates_string_result() {
        init_tracing();
        let s = with_stage("string_stage", || "hello".to_string());
        assert_eq!(s, "hello");
    }

    #[test]
    fn with_stage_works_with_unit_closure() {
        init_tracing();
        let mut side_effect = false;
        with_stage("unit_stage", || {
            side_effect = true;
        });
        assert!(side_effect);
    }

    // ── StageTimer ──────────────────────────────────────────────────────────

    #[test]
    fn stage_timer_elapsed_ms_is_non_negative() {
        let timer = StageTimer::start("test_timer");
        assert!(timer.elapsed_ms() < 60_000, "timer should not exceed 1 minute in a test");
    }

    #[test]
    fn stage_timer_label_is_preserved() {
        let timer = StageTimer::start("my_label");
        assert_eq!(timer.label(), "my_label");
    }

    #[test]
    fn stage_timer_elapsed_increases_over_time() {
        let timer = StageTimer::start("monotonic_test");
        let t1 = timer.elapsed();
        // Spin briefly to ensure time passes (no sleep — tests must be fast).
        let _: u64 = (0..100_000u64).sum();
        let t2 = timer.elapsed();
        assert!(t2 >= t1, "elapsed time must be monotonically non-decreasing");
    }
}
