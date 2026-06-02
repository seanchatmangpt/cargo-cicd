//! cargo-cicd — Level 5 State Model for Rust CI/CD Pipelines
//!
//! This crate provides a comprehensive internal state machine for managing Rust workspace
//! compilation, testing, target directory lifecycle, and git phases.
//!
//! ## Core Modules
//!
//! - [`state`] — Level 5 state model types
//! - [`cicd_toml`] — Configuration contract (cicd.toml deserializer/serializer)

pub mod state;
pub mod cicd_toml;

pub use state::{
    WorkspaceState, TargetState, ChangedFileState, TestPlanState, TrybuildFixtureState,
    GitPhaseState, ProcessEventState, ArtifactState, PolicyState, ProjectionProfile,
    StateVerdict, ChangeKind, SnapshotStatus, EventKind, PolicyMode,
};

pub use cicd_toml::{CicdConfig, WorkspaceConfig, StateConfig, TargetConfig, TestConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that key types are re-exported
        let _: StateVerdict = StateVerdict::Pass;
        let _: ChangeKind = ChangeKind::Source;
        let _: SnapshotStatus = SnapshotStatus::Synced;
    }
}
