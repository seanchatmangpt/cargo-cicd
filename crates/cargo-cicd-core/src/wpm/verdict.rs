//! Verdict schema contract for wpm/wasm4pm court output.
//!
//! The authoritative fields are `overall_fitness` and `verdict`.
//! `precision` is present but may be null (explicitly unsupported).
//! Consumers MUST read `overall_fitness`, never `fitness`.

use crate::evidence::trace_class::TraceClass;

/// The structured verdict emitted by the wpm court.
///
/// Consumers must read `overall_fitness`, not `fitness`.
/// If `overall_fitness` is absent, treat as BLOCKED, not 0.0.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WpmVerdict {
    /// Authoritative fitness score. Range [0.0, 1.0].
    /// MUST be read by consumers instead of any other fitness key.
    pub overall_fitness: Option<f64>,

    /// Precision score. Null when not computed by this court implementation.
    /// Explicitly null ≠ 0.0.
    pub precision: Option<f64>,

    /// Human-readable conformance verdict (TRUTHFUL, VARIANCE, DECEPTIVE).
    pub verdict: String,

    /// Token deviation summary (e.g. "M:0 R:0").
    pub token_deviation: Option<String>,

    /// Execution context of the audited trace.
    pub trace_class: Option<TraceClass>,

    /// Source of the discovered model.
    pub model_source: Option<String>,

    /// Reference to the receipt that was adjudicated.
    pub receipt_ref: Option<String>,
}

impl WpmVerdict {
    /// Returns the authoritative fitness score, or None if not present.
    /// Never falls back to a different key.
    pub fn authoritative_fitness(&self) -> Option<f64> {
        self.overall_fitness
    }

    /// Returns true if the verdict represents conformant execution.
    pub fn is_conformant(&self) -> bool {
        matches!(
            self.verdict.to_uppercase().as_str(),
            "TRUTHFUL" | "VARIANCE"
        )
    }

    /// Returns true if precision was explicitly computed (not just absent or null).
    pub fn has_precision(&self) -> bool {
        self.precision.is_some()
    }
}
