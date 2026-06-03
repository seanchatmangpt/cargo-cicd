//! Publish readiness check.

use std::path::Path;

/// Publish readiness assessment for a workspace.
pub struct PublishReadiness {
    /// True when `cicd.toml` exists at the workspace root.
    pub cicd_toml_exists: bool,
    /// Path to the dry-run marker file if present.
    pub dry_run_marker: Option<String>,
    /// True when a dry-run marker is present AND at least one receipt exists.
    pub dry_run_has_receipt: bool,
}

impl PublishReadiness {
    /// Assess publish readiness for the workspace at `root`.
    pub fn from_workspace(root: &Path) -> Self {
        let cicd_toml_exists = root.join("cicd.toml").exists();

        // Look for a dry-run marker (.dry-run or dry-run.stamp).
        let dry_run_marker = ["dry-run.stamp", ".dry-run"]
            .iter()
            .find(|name| root.join(name).exists())
            .map(|name| name.to_string());

        // Check whether any receipt file exists.
        let receipts_dir = root.join("receipts");
        let has_receipts = receipts_dir.is_dir()
            && std::fs::read_dir(&receipts_dir)
                .map(|mut rd| rd.next().is_some())
                .unwrap_or(false);

        let dry_run_has_receipt = dry_run_marker.is_some() && has_receipts;

        Self {
            cicd_toml_exists,
            dry_run_marker,
            dry_run_has_receipt,
        }
    }
}
