pub mod workspace_state;
pub mod toolchain_state;
pub mod target_state;
pub mod changed_file_state;
pub mod test_plan_state;
pub mod trybuild_state;
pub mod git_phase_state;
pub mod process_event_state;
pub mod artifact_state;
pub mod policy_state;
pub mod projection_profile;

pub use workspace_state::WorkspaceState;
pub use toolchain_state::ToolchainState;
pub use target_state::TargetState;
pub use changed_file_state::ChangedFileState;
pub use test_plan_state::TestPlanState;
pub use trybuild_state::TrybuildState;
pub use git_phase_state::GitPhaseState;
pub use process_event_state::ProcessEventState;
pub use artifact_state::ArtifactState;
pub use policy_state::PolicyState;
pub use projection_profile::ProjectionProfile;

/// Full Level 5 engine state — all dimensions
#[derive(Debug, Default)]
pub struct EngineState {
    pub workspace: WorkspaceState,
    pub toolchain: ToolchainState,
    pub target: TargetState,
    pub changed_files: ChangedFileState,
    pub test_plan: TestPlanState,
    pub trybuild: TrybuildState,
    pub git_phase: GitPhaseState,
    pub process_events: ProcessEventState,
    pub artifacts: ArtifactState,
    pub policies: PolicyState,
    pub projection: ProjectionProfile,
}
