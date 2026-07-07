pub mod branch_behind;
pub mod evidence_stale;
pub mod git_phase_dirty;
pub mod publish_not_adjudicated;
pub mod target_pressure;
pub mod toolchain_mismatch;
pub mod trybuild_changed;

pub use branch_behind::BranchBehindPolicy;
pub use evidence_stale::EvidenceStalePolicy;
pub use git_phase_dirty::GitPhaseDirtyPolicy;
pub use publish_not_adjudicated::PublishNotAdjudicatedPolicy;
pub use target_pressure::TargetPressurePolicy;
pub use toolchain_mismatch::ToolchainMismatchPolicy;
pub use trybuild_changed::TrybuildChangedPolicy;

/// Common policy interface
pub trait CicdPolicy {
    fn name(&self) -> &'static str;
    fn enabled(&self) -> bool;
    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult;
}

#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub verdict: String,
    pub recommendation: Option<String>,
}
