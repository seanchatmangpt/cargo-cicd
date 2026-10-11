//! Git status model.

use std::path::Path;

/// Git status at a point in time (legacy alias — use [`GitStatusSummary`]).
pub struct GitStatus;

/// Summary of git working-tree state.
#[derive(Debug, Clone, Default)]
pub struct GitStatusSummary {
    /// True when the working tree has uncommitted changes.
    pub dirty: bool,
    /// Number of untracked files.
    pub untracked_count: usize,
}

impl GitStatusSummary {
    /// Detect git status by running `git status --porcelain` in `root`.
    pub fn detect(root: &Path) -> Self {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let dirty = stdout
                    .lines()
                    .any(|l| !l.starts_with("??") && !l.trim().is_empty());
                let untracked_count = stdout.lines().filter(|l| l.starts_with("??")).count();
                Self {
                    dirty,
                    untracked_count,
                }
            }
            _ => Self::default(),
        }
    }

    /// Fallible variant: surfaces git failure as a typed error instead of
    /// silently reading as a clean tree (the legacy `detect` keeps its
    /// tolerant default for compatibility).
    pub fn try_detect(root: &Path) -> Result<Self, GitStatusError> {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
            .map_err(|e| GitStatusError {
                root: root.display().to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(GitStatusError {
                root: root.display().to_string(),
                reason: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let dirty = stdout
            .lines()
            .any(|l| !l.starts_with("??") && !l.trim().is_empty());
        let untracked_count = stdout.lines().filter(|l| l.starts_with("??")).count();
        Ok(Self {
            dirty,
            untracked_count,
        })
    }
}

/// `git status` could not run to completion in `root`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("git status failed in {root}: {reason}")]
pub struct GitStatusError {
    /// Repository root passed in.
    pub root: String,
    /// Spawn error message or git stderr.
    pub reason: String,
}
