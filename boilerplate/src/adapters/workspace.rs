//! Workspace adapter — reads workspace identity from `Cargo.toml`.
//!
//! Deliberately avoids shelling out to `cargo metadata` (which is slow and
//! requires a full workspace resolution).  Instead, we parse `Cargo.toml`
//! line-by-line for the fields we need.

#![cfg(feature = "process-data")]

use crate::engine::WorkspaceState;

/// Populates [`WorkspaceState`] from the nearest `Cargo.toml`.
pub struct WorkspaceAdapter;

impl WorkspaceAdapter {
    /// Build a [`WorkspaceState`] from the current directory's `Cargo.toml`.
    ///
    /// Silently returns the `Default` state on any failure.
    pub fn populate() -> WorkspaceState {
        match Self::try_populate() {
            Ok(state) => state,
            Err(err) => {
                tracing::warn!("WorkspaceAdapter failed, using defaults: {err}");
                WorkspaceState::default()
            }
        }
    }

    fn try_populate() -> anyhow::Result<WorkspaceState> {
        let manifest_path = Self::find_workspace_root()?;
        let content = std::fs::read_to_string(&manifest_path)?;

        let root_path = manifest_path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut state = WorkspaceState {
            root_path,
            ..Default::default()
        };

        // Parse the TOML line-by-line for the fields we care about.
        // A proper TOML parse is used here because the line-by-line approach
        // is fragile for multi-line values.
        let manifest: toml::Value = toml::from_str(&content)?;

        // Try [package] first (single-crate workspace), then [workspace].
        if let Some(pkg) = manifest.get("package") {
            state.name = pkg
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned();
            state.edition = pkg
                .get("edition")
                .and_then(|v| v.as_str())
                .unwrap_or("2021")
                .to_owned();
            state.rust_version = pkg
                .get("rust-version")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
        } else if let Some(ws) = manifest.get("workspace") {
            // Multi-crate workspace — use the directory name as display name.
            state.name = std::path::Path::new(&state.root_path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "workspace".to_owned());

            if let Some(members) = ws.get("members").and_then(|v| v.as_array()) {
                state.members = members
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(str::to_owned)
                    .collect();
            }

            if let Some(pkg) = ws.get("package") {
                state.edition = pkg
                    .get("edition")
                    .and_then(|v| v.as_str())
                    .unwrap_or("2021")
                    .to_owned();
                state.rust_version = pkg
                    .get("rust-version")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned);
            }
        }

        Ok(state)
    }

    /// Walk up the directory tree to find the workspace root `Cargo.toml`.
    fn find_workspace_root() -> anyhow::Result<std::path::PathBuf> {
        let mut dir = std::env::current_dir()?;
        loop {
            let candidate = dir.join("Cargo.toml");
            if candidate.exists() {
                return Ok(candidate);
            }
            if !dir.pop() {
                anyhow::bail!("no Cargo.toml found in any ancestor directory");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_does_not_panic() {
        // Even in an arbitrary directory, populate() must return a valid state.
        let _state = WorkspaceAdapter::populate();
    }

    #[test]
    fn state_default_is_valid() {
        let state = WorkspaceState::default();
        assert!(state.name.is_empty());
        assert!(state.members.is_empty());
    }
}
