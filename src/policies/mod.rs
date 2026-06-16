pub mod branch_behind;
pub mod diagnostics_bridge;
pub mod evidence_stale;
pub mod git_phase_dirty;
pub mod publish_not_adjudicated;
pub mod target_pressure;
pub mod toolchain_mismatch;
pub mod trybuild_changed;

pub use branch_behind::BranchBehindPolicy;
pub use diagnostics_bridge::render_policy_diagnostic;
pub use evidence_stale::EvidenceStalePoliciy;
pub use git_phase_dirty::GitPhaseDirtyPolicy;
pub use publish_not_adjudicated::PublishNotAdjudicatedPolicy;
pub use target_pressure::TargetPressurePolicy;
pub use toolchain_mismatch::ToolchainMismatchPolicy;
pub use trybuild_changed::TrybuildChangedPolicy;

/// Policy mode — only suggest is enabled by default
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyMode {
    Suggest,
    Apply,
}

/// Policy verdict
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyVerdict {
    Pass,
    Warn,
    Alert,
}

/// Common policy interface
pub trait CicdPolicy {
    fn name(&self) -> &'static str;
    fn enabled(&self) -> bool;
    fn mode(&self) -> PolicyMode;
    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult;
}

#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub name: String,
    pub enabled: bool,
    pub mode: String,
    pub verdict: String,
    pub recommendation: Option<String>,
    pub event_kind: String,
}
