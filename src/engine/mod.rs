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
pub use policy_state::{PolicyEntry, PolicyState};
pub use process_event_state::{ProcessEvent, ProcessEventState};
pub use projection_profile::ProjectionProfile;
pub use target_state::TargetState;
pub use test_plan_state::TestPlanState;
pub use toolchain_state::ToolchainState;
pub use trybuild_state::TrybuildState;
pub use workspace_state::WorkspaceState;

/// Full Level 5 engine state — the aggregate root for all workspace dimensions.
///
/// Populated by [`EngineState::from_workspace`] via independent adapters.
/// Each adapter silently fails; partial data is better than no data.
/// Use [`Default`] in tests to get an all-zero state without filesystem access.
///
/// # Example
///
/// ```
/// use cargo_cicd::EngineState;
/// let state = EngineState::default();
/// assert!(state.workspace.name.is_empty());
/// assert!(state.git_phase.dirty_files.is_empty());
/// ```
#[derive(Debug, Default)]
pub struct Pending;

#[derive(Debug, Default)]
pub struct EngineStateInner<State> {
    /// Workspace metadata: name, root path, members, toolchain, Rust edition.
    pub workspace: WorkspaceState,
    /// Active Rust toolchain and compiler version.
    pub toolchain: ToolchainState,
    /// Target directory path and cumulative size in bytes.
    pub target: TargetState,
    /// Changed `.rs` files since `origin/main`, classified into test and trybuild sets.
    pub changed_files: ChangedFileState,
    /// Estimated test count and whether conservative mode is active.
    pub test_plan: TestPlanState,
    /// Trybuild fixture inventory and snapshot mode setting.
    pub trybuild: TrybuildState,
    /// Git branch, dirty/staged/untracked files, and ahead/behind counts.
    pub git_phase: GitPhaseState,
    /// Process events accumulated in this session (mirrors the `cicd.toml` events table).
    pub process_events: ProcessEventState,
    /// Paths to `cicd.toml` and any other emitted artifact manifests.
    pub artifacts: ArtifactState,
    /// Autonomic policy entries and their current verdicts.
    pub policies: PolicyState,
    /// Feature flag surface contract for v26.6.2.
    pub projection: ProjectionProfile,
    /// The admitted cicd.toml configuration (or default if not present/invalid).
    pub config: crate::cicd_toml::CicdToml,
    /// BLAKE3 witness hash from the star-toml admission pipeline; None if no admitted config.
    pub config_witness_hash: Option<String>,
    #[doc(hidden)]
    pub _state: std::marker::PhantomData<State>,
}

pub type EngineState = EngineStateInner<Pending>;

impl EngineState {
    /// Build a real EngineState by querying all available adapters.
    /// Failures are silenced — partial data is better than no data.
    pub fn from_workspace() -> Self {
        let mut state = Self::default();

        // Load cicd.toml through the star-toml admission pipeline
        match crate::cicd_toml::load_admitted() {
            Ok(admitted) => {
                state.config_witness_hash = Some(admitted.witness().hash().to_string());
                state.config = admitted.value().clone();
            }
            Err(_) => {
                // No cicd.toml or validation failed — fall back to load_or_default()
                eprintln!("cargo-cicd: warning: cicd.toml not admitted; using defaults");
                state.config = crate::cicd_toml::load_or_default();
                state.config_witness_hash = None;
            }
        }

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
        let (target_size, target_scan_errors) =
            crate::adapters::TargetScannerAdapter::total_size_bytes_with_errors(&target_dir);
        state.target.total_size_bytes = target_size;
        state.target.scan_errors = target_scan_errors;

        // Populate changed_files state
        let base_ref = state.config.test.changed.base.clone();
        state.changed_files.base_ref = base_ref.clone();
        let changed_rs = crate::adapters::ChangedFileDetector::changed_rs_files(&base_ref);
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
        if let Ok(cicd_toml) = crate::cicd_toml::CicdToml::from_file(std::path::Path::new(
            &format!("{}/cicd.toml", state.workspace.root_path),
        )) {
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

        // Populate projection profile with current package version
        state.projection = ProjectionProfile::v26_7_6();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // std::env::set_current_dir is process-global; serialize all cwd-mutating tests.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    fn write_cicd_toml(dir: &std::path::Path, base: &str) {
        let content = format!(
            r#"[workspace]
name = "test-workspace"
toolchain = "stable"
target_dir = "target"

[state]
dirty = false
target_size_gb = 0.0
changed_files = 0
changed_tests = 0
changed_trybuild_fixtures = 0

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
enabled = true
base = "{base}"

[trybuild.changed]
enabled = true
snapshot_mode = "changed-only"

[git.phase]
require_clean_tree = true
commit_after_phase = false

[autonomic]
enabled = true
mode = "suggest"
"#
        );
        std::fs::write(dir.join("cicd.toml"), content).unwrap();
    }

    #[test]
    fn engine_uses_cicd_toml_base_ref() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        write_cicd_toml(dir.path(), "origin/develop");
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let state = EngineState::from_workspace();
        std::env::set_current_dir(prev).unwrap();
        assert_eq!(state.changed_files.base_ref, "origin/develop");
        assert_eq!(state.config.test.changed.base, "origin/develop");
        assert!(state.config_witness_hash.is_some());
    }

    #[test]
    fn engine_defaults_to_origin_main() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let state = EngineState::from_workspace();
        std::env::set_current_dir(prev).unwrap();
        assert_eq!(state.changed_files.base_ref, "origin/main");
        assert_eq!(state.config.test.changed.base, "origin/main");
        assert!(state.config_witness_hash.is_none());
    }
}
