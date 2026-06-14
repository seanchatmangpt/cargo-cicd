//! Snapshot persistence adapter for engine state.
//!
//! [`StateSnapshotAdapter`] persists engine state snapshots to disk via the
//! `advanced::snapshot` module, enabling recovery of workspace, toolchain,
//! and changed-file state across invocations.
//!
//! # Usage Example
//!
//! ```ignore
//! use cargo_cicd::adapters::StateSnapshotAdapter;
//! use cargo_cicd::EngineState;
//! use std::path::Path;
//!
//! let engine = EngineState::default();
//! let workspace_root = Path::new("/home/user/cargo-cicd");
//!
//! // Save the current engine state
//! StateSnapshotAdapter::save(&engine, &StateSnapshotAdapter::state_cache_path(workspace_root))?;
//!
//! // Load the state snapshot back
//! match StateSnapshotAdapter::load(&StateSnapshotAdapter::state_cache_path(workspace_root))? {
//!     Some(snapshot) => println!("Loaded snapshot from {}", snapshot.workspace_root),
//!     None => println!("No snapshot found on disk"),
//! }
//! # Ok::<(), std::io::Error>(())
//! ```

use crate::engine::EngineState;
use std::path::{Path, PathBuf};

#[cfg(feature = "advanced")]
use crate::advanced::snapshot::{self, EngineSnapshot};

/// Adapter for persisting engine state snapshots to disk.
pub struct StateSnapshotAdapter;

impl StateSnapshotAdapter {
    /// Saves the engine state to disk via the snapshot module.
    ///
    /// When the `advanced` feature is enabled, serializes workspace root,
    /// toolchain, and changed_files state and writes to the given path
    /// using the compact bitcode encoding from [`advanced::snapshot`].
    ///
    /// When `advanced` is disabled, this is a no-op returning Ok(()).
    #[cfg(feature = "advanced")]
    pub fn save(engine: &EngineState, path: &Path) -> std::io::Result<()> {
        let snapshot = EngineSnapshot {
            workspace_root: engine.workspace.root_path.clone(),
            toolchain: engine.toolchain.active.clone(),
            changed_files: engine.changed_files.changed_rs_files.clone(),
            target_bytes: 0, // target bytes not available from EngineState
            git_phase: String::new(), // git phase not available from basic EngineState
            schema_version: EngineSnapshot::current_schema_version(),
            stages: Vec::new(), // stages not available from basic EngineState
        };

        let bytes = snapshot::encode(&snapshot)?;
        std::fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Saves the engine state to disk via the snapshot module.
    ///
    /// When the `advanced` feature is disabled, this is a no-op returning Ok(()).
    #[cfg(not(feature = "advanced"))]
    pub fn save(_engine: &EngineState, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }

    /// Loads an engine state snapshot from disk.
    ///
    /// When the `advanced` feature is enabled, reads and deserializes a snapshot
    /// from the given path using the bitcode decoder from [`advanced::snapshot`].
    ///
    /// Returns:
    /// - `Ok(Some(snapshot))` if the snapshot was successfully loaded
    /// - `Ok(None)` if the file doesn't exist (not an error)
    /// - `Err(e)` if the file exists but cannot be read or is malformed
    ///
    /// When `advanced` is disabled, this is a no-op returning Ok(None).
    #[cfg(feature = "advanced")]
    pub fn load(path: &Path) -> std::io::Result<Option<EngineSnapshot>> {
        if !path.exists() {
            return Ok(None);
        }

        let bytes = std::fs::read(path)?;
        let snapshot = snapshot::decode(&bytes)?;
        Ok(Some(snapshot))
    }

    /// Loads an engine state snapshot from disk.
    ///
    /// When the `advanced` feature is disabled, this is a no-op returning Ok(None).
    #[cfg(not(feature = "advanced"))]
    pub fn load(_path: &Path) -> std::io::Result<Option<()>> {
        Ok(None)
    }

    /// Returns the standard cache path for engine state snapshots.
    ///
    /// Returns `.cargo/cicd_state.snapshot` relative to the given workspace root.
    pub fn state_cache_path(workspace_root: &Path) -> PathBuf {
        workspace_root.join(".cargo/cicd_state.snapshot")
    }
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_save_load_equality() {
        let dir = tempfile::tempdir().expect("failed to create temp directory");
        let cache_path = dir.path().join(".cargo/cicd_state.snapshot");

        // Create an engine with some sample state
        let mut engine = EngineState::default();
        engine.workspace.root_path = "/home/user/cargo-cicd".to_string();
        engine.workspace.name = "cargo-cicd".to_string();
        engine.toolchain.active = "stable".to_string();
        engine.toolchain.rust_version = "1.75.0".to_string();
        engine.changed_files.changed_rs_files =
            vec!["src/lib.rs".to_string(), "src/adapters/state_snapshot.rs".to_string()];
        engine.changed_files.total_changed = 2;

        // Save to the cache path
        StateSnapshotAdapter::save(&engine, &cache_path)
            .expect("save should succeed");

        // Load it back
        let loaded = StateSnapshotAdapter::load(&cache_path)
            .expect("load should succeed")
            .expect("snapshot should exist");

        // Assert key fields match
        assert_eq!(loaded.workspace_root, engine.workspace.root_path);
        assert_eq!(loaded.toolchain, engine.toolchain.active);
        assert_eq!(loaded.changed_files, engine.changed_files.changed_rs_files);
        assert_eq!(loaded.schema_version, EngineSnapshot::current_schema_version());
    }

    #[test]
    fn load_nonexistent_file_returns_none() {
        let nonexistent = PathBuf::from("/tmp/never_created_snapshot_12345.snap");
        let result = StateSnapshotAdapter::load(&nonexistent)
            .expect("load should not error on missing file");
        assert!(result.is_none(), "missing file should return None, not error");
    }

    #[test]
    fn state_cache_path_is_relative_to_workspace() {
        let workspace_root = Path::new("/home/user/my-workspace");
        let cache_path = StateSnapshotAdapter::state_cache_path(workspace_root);

        assert_eq!(cache_path, PathBuf::from("/home/user/my-workspace/.cargo/cicd_state.snapshot"));
    }
}
