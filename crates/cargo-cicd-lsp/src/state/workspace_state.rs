//! WorkspaceState — workspace root tracking and snapshot refresh.

use std::path::PathBuf;

/// Tracks the current workspace root path.
#[derive(Debug, Default)]
pub struct WorkspaceState {
    pub root: Option<PathBuf>,
}

impl WorkspaceState {
    /// Create a new, empty WorkspaceState.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the workspace root.
    pub fn set_root(&mut self, path: PathBuf) {
        self.root = Some(path);
    }

    /// Return the current root path, if set.
    pub fn root(&self) -> Option<&PathBuf> {
        self.root.as_ref()
    }

    /// Clear the workspace root.
    pub fn clear(&mut self) {
        self.root = None;
    }
}
