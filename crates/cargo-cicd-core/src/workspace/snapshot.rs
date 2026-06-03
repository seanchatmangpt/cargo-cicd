//! Workspace state snapshot.

use std::path::PathBuf;

use crate::evidence::freshness::EvidenceState;
use crate::git::status::GitStatusSummary;

/// Snapshot of the workspace state at a point in time.
pub struct WorkspaceSnapshot {
    /// Workspace root directory.
    pub root: PathBuf,
    /// Evidence state (exists, freshness).
    pub evidence_state: EvidenceState,
    /// Git working-tree status.
    pub git_status: GitStatusSummary,
    /// Whether a `receipts/` directory exists at the workspace root.
    pub has_receipts_dir: bool,
}

impl WorkspaceSnapshot {
    /// Construct a snapshot by inspecting the workspace at `root` (path version).
    pub fn from_path(root: &std::path::Path) -> Self {
        Self::from_root(root.to_path_buf())
    }

    /// Construct a minimal synthetic snapshot for use in tests.
    ///
    /// When `dirty` is `true` the snapshot reports a dirty working tree.
    /// No filesystem is probed.
    pub fn synthetic(dirty: bool) -> Self {
        Self {
            root: PathBuf::from("/synthetic"),
            evidence_state: EvidenceState::default(),
            git_status: GitStatusSummary {
                dirty,
                untracked_count: 0,
            },
            has_receipts_dir: false,
        }
    }

    /// Construct a snapshot by inspecting the workspace at `root`.
    pub fn from_root(root: PathBuf) -> Self {
        let evidence_dir = root.join("target").join("cargo-cicd").join("evidence");
        let evidence_state = EvidenceState::from_dir(&evidence_dir, &root);
        let git_status = GitStatusSummary::detect(&root);
        let has_receipts_dir = root.join("receipts").is_dir();
        Self {
            root,
            evidence_state,
            git_status,
            has_receipts_dir,
        }
    }
}
