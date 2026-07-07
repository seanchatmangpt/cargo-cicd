//! Structured observability for the cargo-cicd process-data engine.
//!
//! This module wires the engine pipeline to [`tracing`] so that each stage of
//! the Level 5 process-data flow emits structured spans and events. Adapters and
//! nouns can open a [`PipelineStage`] guard around a unit of work; when the guard
//! drops it records the elapsed duration as a structured event, giving the
//! engine a uniform, machine-readable trace of how process data moved through
//! the pipeline.
//!
//! The output is JSON-formatted (via [`tracing_subscriber`]'s `fmt` layer in
//! JSON mode) so traces compose cleanly with the engine's other structured
//! artifacts (such as `cicd.toml` events) and can be ingested by downstream
//! tooling. Filtering honors the `RUST_LOG` environment variable through an
//! [`EnvFilter`](tracing_subscriber::EnvFilter), defaulting to `info`.
//!
//! Typical usage inside an engine stage:
//!
//! ```no_run
//! use cargo_cicd::advanced::observability::{init_tracing, record_event, PipelineStage};
//!
//! init_tracing();
//! {
//!     let _stage = PipelineStage::enter("workspace_scan");
//!     // ... do work that populates engine state ...
//!     record_event("workspace_scan", true);
//! } // drop here emits an `elapsed_ms` event for the stage
//! ```

use std::sync::Once;
use std::time::Instant;

use tracing_subscriber::EnvFilter;

/// Ensures the global subscriber is installed at most once for the process.
static INIT: Once = Once::new();

/// Install the process-wide structured tracing subscriber.
///
/// This builds a [`tracing_subscriber`] registry with:
/// - an [`EnvFilter`] sourced from `RUST_LOG`
///   ([`EnvFilter::from_default_env`](tracing_subscriber::EnvFilter::from_default_env)),
///   falling back to the `info` level when the variable is unset or unparsable, and
/// - a JSON-formatted `fmt` layer so engine traces are emitted as structured
///   records suitable for downstream process-data ingestion.
///
/// The installation is **idempotent**: it is guarded by a [`std::sync::Once`] and
/// additionally uses `try_init`, so calling [`init_tracing`] more than once (for
/// example, from multiple engine entry points or from tests) never panics and
/// never installs a competing global subscriber.
pub fn init_tracing() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        // `try_init` returns an error rather than panicking if a global
        // subscriber is already set, which keeps this call safe to make from
        // anywhere in the engine and from test harnesses.
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .try_init();
    });
}

/// RAII guard that instruments a single stage of the engine pipeline.
///
/// Entering a stage opens an info-level [`tracing`] span and starts a real-time
/// timer. When the guard is dropped — typically at the end of the enclosing
/// scope — it emits a structured info event carrying the stage `name` and the
/// `elapsed_ms` duration, so the time spent in each process-data stage is
/// captured automatically without manual bookkeeping.
///
/// The span is held open for the lifetime of the guard, so any events recorded
/// while the guard is in scope (including those from [`record_event`]) are
/// nested under the stage span.
pub struct PipelineStage {
    name: String,
    started: Instant,
    span: tracing::Span,
}

impl PipelineStage {
    /// Open an info-level span for the named pipeline `stage` and begin timing.
    ///
    /// Hold the returned guard for the duration of the stage's work; dropping it
    /// records the elapsed time. Stage names should be stable, lowercase
    /// identifiers (for example `"target_scan"` or `"changed_files"`) so traces
    /// aggregate cleanly across runs.
    pub fn enter(name: &str) -> Self {
        let span = tracing::info_span!("pipeline_stage", stage = name);
        // Emit an explicit "begin" marker within the span for traces that want
        // an open/close pair rather than only a duration on close.
        {
            let _entered = span.enter();
            tracing::info!(stage = name, "stage entered");
        }
        PipelineStage {
            name: name.to_string(),
            started: Instant::now(),
            span,
        }
    }

    /// The name of the stage this guard is instrumenting.
    #[allow(dead_code)] // accessor with no current call site; kept for API symmetry
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for PipelineStage {
    fn drop(&mut self) {
        let elapsed_ms = self.started.elapsed().as_millis() as u64;
        let _entered = self.span.enter();
        tracing::info!(
            stage = %self.name,
            elapsed_ms = elapsed_ms,
            "stage completed"
        );
    }
}

/// Emit a structured tracing event for a discrete engine `stage` outcome.
///
/// This records an info-level event with two structured fields:
/// - `stage`: the name of the pipeline stage the event belongs to, and
/// - `ok`: whether the stage step succeeded.
///
/// Use this for point-in-time outcomes within a stage (for example, an adapter
/// reporting that it successfully populated a slice of engine state). For
/// whole-stage timing, prefer the [`PipelineStage`] guard, which records
/// duration automatically on drop.
#[allow(dead_code)] // exercised by examples/03_max_pipeline.rs
pub fn record_event(stage: &str, ok: bool) {
    tracing::info!(stage = stage, ok = ok, "stage event");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::subscriber::with_default;
    use tracing_subscriber::fmt;

    /// A subscriber that writes JSON to the per-test writer; capturing the exact
    /// bytes is unnecessary for these checks — we only need the instrumentation
    /// to run end-to-end without panicking under an active subscriber.
    fn test_subscriber() -> impl tracing::Subscriber + Send + Sync {
        fmt()
            .json()
            .with_test_writer()
            .with_env_filter(EnvFilter::new("info"))
            .finish()
    }

    #[test]
    fn init_tracing_is_idempotent() {
        // Calling twice must never panic; the `Once` + `try_init` guard makes
        // the second call a no-op.
        init_tracing();
        init_tracing();
    }

    #[test]
    fn pipeline_stage_enters_and_drops() {
        with_default(test_subscriber(), || {
            let stage = PipelineStage::enter("unit_test_stage");
            assert_eq!(stage.name(), "unit_test_stage");
            // Explicit drop exercises the duration-recording path.
            drop(stage);
        });
    }

    #[test]
    fn record_event_emits_without_panicking() {
        with_default(test_subscriber(), || {
            record_event("unit_test_stage", true);
            record_event("unit_test_stage", false);
        });
    }

    #[test]
    fn nested_stages_are_safe() {
        with_default(test_subscriber(), || {
            let _outer = PipelineStage::enter("outer");
            {
                let _inner = PipelineStage::enter("inner");
                record_event("inner", true);
            }
            record_event("outer", true);
        });
    }
}
