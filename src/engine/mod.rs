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
pub use policy_state::{PolicyState, PolicyEntry};
pub use process_event_state::{ProcessEventState, ProcessEvent};
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
        let target_dir = crate::adapters::CargoMetadataAdapter::target_dir();
        state.workspace.root_path = target_dir
            .trim_end_matches("/target")
            .trim_end_matches("\\target")
            .to_string();
        state.workspace.members = crate::adapters::CargoMetadataAdapter::workspace_members();
        // Populate toolchain and rust_edition from detection
        state.workspace.toolchain = detect_toolchain();
        state.workspace.rust_edition = detect_rust_edition();

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
        state.target.path = target_dir.clone();
        state.target.total_size_bytes =
            crate::adapters::TargetScannerAdapter::total_size_bytes(&target_dir);

        // Populate changed_files state
        let base_ref = "origin/main";
        state.changed_files.base_ref = base_ref.to_string();
        let changed_rs = crate::adapters::ChangedFileDetector::changed_rs_files(base_ref);
        let changed_test_files: Vec<String> = changed_rs
            .iter()
            .filter(|f| crate::adapters::ChangedFileDetector::is_test_file(f))
            .cloned()
            .collect();
        let changed_trybuild: Vec<String> = changed_rs
            .iter()
            .filter(|f| crate::adapters::ChangedFileDetector::is_trybuild_fixture(f))
            .cloned()
            .collect();
        state.changed_files.total_changed = changed_rs.len();
        state.changed_files.changed_rs_files = changed_rs;
        state.changed_files.changed_test_files = changed_test_files;
        state.changed_files.changed_trybuild_fixtures = changed_trybuild;

        // Populate test_plan state with conservative mode if changes exist
        state.test_plan.estimated_count = state.changed_files.total_changed;
        if state.changed_files.total_changed > 0 {
            state.test_plan.conservative_mode = true;
            state.test_plan.reason = Some("Changed files detected".into());
        }

        // Populate trybuild state
        let root_path = state.workspace.root_path.clone();
        state.trybuild.all_fixtures = crate::adapters::TrybuildDetector::all_fixtures(&root_path);
        state.trybuild.changed_fixtures = state.changed_files.changed_trybuild_fixtures.clone();
        state.trybuild.snapshot_mode = "changed-only".to_string();
        state.trybuild.run_all_by_default = false;

        // Populate process_events state from cicd.toml events if it exists
        if let Ok(cicd_toml) = crate::cicd_toml::CicdToml::from_file(
            std::path::Path::new(&format!("{}/cicd.toml", state.workspace.root_path)),
        ) {
            state.process_events.events = cicd_toml
                .events
                .iter()
                .map(|e| ProcessEvent {
                    kind: e.kind.clone(),
                    verdict: e.verdict.clone(),
                    timestamp: e.timestamp.clone().unwrap_or_default(),
                    details: e.details.clone(),
                })
                .collect();
        }

        // Populate artifacts state with cicd.toml path
        let cicd_toml_path = format!("{}/cicd.toml", state.workspace.root_path);
        if std::path::Path::new(&cicd_toml_path).exists() {
            state.artifacts.cicd_toml_path = Some(cicd_toml_path);
        }

        // Populate policies state with default autonomic policy entry
        state.policies.policies = vec![PolicyEntry {
            name: "autonomic".to_string(),
            enabled: true,
            mode: "suggest".to_string(),
            verdict: None,
            recommendation: None,
        }];

        // Populate projection profile with v26.6.2
        state.projection = ProjectionProfile::v26_6_2();

        state
    }
}

/// Detect toolchain from rust-toolchain.toml or rust-toolchain
fn detect_toolchain() -> String {
    if let Ok(content) = std::fs::read_to_string("rust-toolchain.toml") {
        if let Some(line) = content.lines().find(|l| l.contains("channel")) {
            if let Some(ch) = line.split('"').nth(1) {
                return ch.to_string();
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string("rust-toolchain") {
        return content.trim().to_string();
    }
    "stable".into()
}

/// Detect rust edition from Cargo.toml
fn detect_rust_edition() -> String {
    if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
        for line in content.lines() {
            if line.trim().starts_with("edition") {
                if let Some(edition) = line.split('"').nth(1) {
                    return edition.to_string();
                }
            }
        }
    }
    "2021".into()
}
