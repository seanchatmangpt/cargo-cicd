//! Trace class distinguishes pipeline runs from ambient accumulated command history.
//!
//! A pipeline trace (all declared activities in order) may target TRUTHFUL fitness.
//! An ambient trace (accumulated multi-command workspace history) honestly reports VARIANCE.
//!
//! Conflating these two trace classes produces misleading conformance readings.

/// The execution context of a set of process events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceClass {
    /// A deliberate full-pipeline execution (status → test → publish → audit).
    /// May target TRUTHFUL fitness.
    PipelineRun,
    /// Accumulated ambient command history from normal workspace activity.
    /// Honestly reports VARIANCE — not a failure.
    LiveWorkspaceTrace,
    /// Unknown or unclassified trace.
    #[default]
    Unknown,
}

impl TraceClass {
    /// Returns true if this trace class may be held to TRUTHFUL fitness standards.
    pub fn may_target_truthful(&self) -> bool {
        matches!(self, Self::PipelineRun)
    }

    /// Returns the string label written to XES / verdict JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PipelineRun => "pipeline_run",
            Self::LiveWorkspaceTrace => "live_workspace_trace",
            Self::Unknown => "unknown",
        }
    }
}
