use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// WorkspaceState captures the overall state of a Rust workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub name: String,
    pub toolchain: String,
    pub target_dir: PathBuf,
    pub dirty: bool,
    pub size: u64,
    pub changed_files: usize,
    pub changed_tests: usize,
}

/// TargetState represents the compiled target directory state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetState {
    pub path: PathBuf,
    pub total_size: u64,
    pub profile_sizes: HashMap<String, u64>, // e.g., "debug" -> bytes, "release" -> bytes
    pub stale_candidates: Vec<PathBuf>,
    pub max_size: u64,
    pub verdict: StateVerdict,
}

/// ChangedFileState tracks a file that has changed and its impact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFileState {
    pub path: PathBuf,
    pub kind: ChangeKind,
    pub affected_tests: Vec<String>,
    pub affected_fixtures: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeKind {
    Source,
    Test,
    Macro,
    BuildScript,
    Manifest,
    Unknown,
}

/// TestPlanState represents the test plan derived from changed files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlanState {
    pub selected_tests: Vec<String>,
    pub reason: String,
    pub count: usize,
}

/// TrybuildFixtureState tracks the state of trybuild snapshot fixtures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrybuildFixtureState {
    pub fixture_path: PathBuf,
    pub changed: bool,
    pub snapshot_status: SnapshotStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnapshotStatus {
    Synced,
    StaleSnapshot,
    MissingFixture,
    NewFixture,
}

/// GitPhaseState represents the state of the git tree and commits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPhaseState {
    pub branch: String,
    pub dirty_files: Vec<PathBuf>,
    pub staged: Vec<PathBuf>,
    pub untracked: Vec<PathBuf>,
    pub ahead_behind: (usize, usize), // (ahead, behind)
    pub verdict: StateVerdict,
}

/// ProcessEventState represents a single event in the CI/CD process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEventState {
    pub kind: EventKind,
    pub verdict: StateVerdict,
    pub timestamp: SystemTime,
    pub payload: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventKind {
    StatusCheck,
    TargetClean,
    TestRun,
    FixtureSync,
    GitPhaseCheck,
    Autonomic,
}

/// ArtifactState represents an output artifact (binary, docs, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactState {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub created_at: SystemTime,
}

/// PolicyState represents a configured policy and its evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyState {
    pub name: String,
    pub mode: PolicyMode,
    pub signals: Vec<String>,
    pub recommendation: String,
    pub verdict: StateVerdict,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyMode {
    Enforce,
    Suggest,
    Disabled,
}

/// ProjectionProfile describes what level of detail is exposed vs hidden.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionProfile {
    pub level: u8,
    pub public_surface: Vec<String>,
    pub hidden_internals: Vec<String>,
}

/// StateVerdict is the overall judgment of a state component.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StateVerdict {
    Pass,
    Warn,
    Fail,
    Pending,
}

impl std::fmt::Display for StateVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateVerdict::Pass => write!(f, "PASS"),
            StateVerdict::Warn => write!(f, "WARN"),
            StateVerdict::Fail => write!(f, "FAIL"),
            StateVerdict::Pending => write!(f, "PENDING"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_verdict_display() {
        assert_eq!(StateVerdict::Pass.to_string(), "PASS");
        assert_eq!(StateVerdict::Warn.to_string(), "WARN");
        assert_eq!(StateVerdict::Fail.to_string(), "FAIL");
        assert_eq!(StateVerdict::Pending.to_string(), "PENDING");
    }

    #[test]
    fn test_workspace_state_serialization() {
        let state = WorkspaceState {
            name: "test".to_string(),
            toolchain: "stable".to_string(),
            target_dir: PathBuf::from("/target"),
            dirty: false,
            size: 1000,
            changed_files: 5,
            changed_tests: 3,
        };

        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: WorkspaceState =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.dirty, false);
    }
}
