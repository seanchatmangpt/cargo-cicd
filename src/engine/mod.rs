pub mod artifact_state;
pub mod changed_file_state;
pub mod git_phase_state;
pub mod policy_state;
pub mod process_event_state;
pub mod projection_profile;
pub mod target_state;
pub mod test_plan_state;
pub mod toolchain_state;
pub mod trybuild_state;
pub mod workspace_state;

pub use artifact_state::ArtifactState;
pub use changed_file_state::ChangedFileState;
pub use git_phase_state::GitPhaseState;
pub use policy_state::PolicyState;
pub use process_event_state::ProcessEventState;
pub use projection_profile::ProjectionProfile;
pub use target_state::TargetState;
pub use test_plan_state::TestPlanState;
pub use toolchain_state::ToolchainState;
pub use trybuild_state::TrybuildState;
pub use workspace_state::WorkspaceState;

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

impl EngineState {
    /// Build a real EngineState by querying all available adapters.
    /// Failures are silenced — partial data is better than no data.
    pub fn from_workspace() -> Self {
        let mut state = Self::default();

        // Populate workspace state
        state.workspace.name = crate::adapters::CargoMetadataAdapter::workspace_name();
        state.workspace.root_path = crate::adapters::CargoMetadataAdapter::target_dir()
            .trim_end_matches("/target")
            .to_string();
        state.workspace.members = crate::adapters::CargoMetadataAdapter::workspace_members();

        // Populate git phase state
        if let Ok(git) = crate::adapters::GitStatusAdapter::query() {
            state.git_phase.branch = git.branch;
            state.git_phase.dirty_files = git.dirty_files;
            state.git_phase.staged_files = git.staged_files;
            state.git_phase.untracked_files = git.untracked_files;
            state.git_phase.ahead = git.ahead;
            state.git_phase.behind = git.behind;
        }

        // Populate toolchain state
        state.toolchain.active = crate::adapters::ToolchainDetector::active_toolchain();
        state.toolchain.rust_version = crate::adapters::ToolchainDetector::rust_version();

        // Populate target state
        let target_dir = crate::adapters::CargoMetadataAdapter::target_dir();
        state.target.path = target_dir.clone();
        state.target.total_size_bytes =
            crate::adapters::TargetScannerAdapter::total_size_bytes(&target_dir);

        state
    }
}
