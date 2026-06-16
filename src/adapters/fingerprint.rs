//! Workspace artifact fingerprinting adapter.
//!
//! The `FingerprintAdapter` computes deterministic content hashes of all
//! artifact files in the workspace to detect changes. When the `advanced`
//! feature is enabled, it uses BLAKE3-based Merkle digests; otherwise it
//! returns `None`.
//!
//! # Example
//!
//! Use within the engine to detect artifact changes:
//!
//! ```ignore
//! use cargo_cicd::adapters::FingerprintAdapter;
//! use std::path::Path;
//!
//! if let Some(fingerprint) = FingerprintAdapter::digest_artifacts(workspace_root) {
//!     let cache_key = FingerprintAdapter::cache_key();
//!     // Store or compare against previous: fingerprint.to_hex()
//! }
//! ```

use std::path::Path;

/// Adapter for computing workspace artifact content hashes.
pub struct FingerprintAdapter;

impl FingerprintAdapter {
    /// Compute a deterministic hash of all artifact files in the workspace.
    ///
    /// When the `advanced` feature is enabled, this uses `advanced::fingerprint::workspace_digest`
    /// to compute a BLAKE3-based Merkle root over all artifacts in the workspace (e.g., target/,
    /// build outputs). The result is order-independent but sensitive to any change in artifact
    /// content or path.
    ///
    /// When the `advanced` feature is disabled, this returns `None`.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root path.
    ///
    /// # Returns
    ///
    /// `Some(Fingerprint)` if the advanced feature is on and artifacts can be scanned;
    /// `None` otherwise.
    #[cfg(feature = "advanced")]
    pub fn digest_artifacts(root: &Path) -> Option<crate::advanced::fingerprint::Fingerprint> {
        use crate::advanced::fingerprint::{hash_file, workspace_digest};
        use std::collections::BTreeMap;
        use walkdir::WalkDir;

        let target_dir = root.join("target");
        if !target_dir.exists() {
            // No artifacts yet; return empty digest.
            let empty: Vec<(
                std::path::PathBuf,
                crate::advanced::fingerprint::Fingerprint,
            )> = Vec::new();
            return Some(workspace_digest(&empty));
        }

        let mut entries = BTreeMap::new();

        for entry in WalkDir::new(&target_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(fp) = hash_file(entry.path()) {
                if let Ok(rel_path) = entry.path().strip_prefix(root) {
                    entries.insert(rel_path.to_path_buf(), fp);
                }
            }
        }

        let entries_vec: Vec<_> = entries.into_iter().collect();
        Some(workspace_digest(&entries_vec))
    }

    /// Compute a deterministic hash of all artifact files in the workspace (disabled variant).
    ///
    /// When the `advanced` feature is disabled, this always returns `None`.
    #[cfg(not(feature = "advanced"))]
    pub fn digest_artifacts(_root: &Path) -> Option<()> {
        None
    }

    /// Return the cache key string for storing workspace artifact fingerprints.
    ///
    /// Use this key when caching or comparing fingerprints across runs.
    pub fn cache_key() -> &'static str {
        "workspace_artifacts_fingerprint"
    }
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::TempDir;

    #[test]
    fn digest_artifacts_is_deterministic() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path();

        // Create a mock artifact structure.
        let target_dir = root.join("target");
        std::fs::create_dir_all(&target_dir).expect("create target dir");

        let artifact = target_dir.join("debug/app");
        std::fs::create_dir_all(artifact.parent().unwrap()).expect("create artifact parent");
        let mut file = std::fs::File::create(&artifact).expect("create artifact file");
        file.write_all(b"mock artifact content")
            .expect("write artifact");
        drop(file);

        // Compute digest twice and assert determinism.
        let digest1 =
            FingerprintAdapter::digest_artifacts(root).expect("first digest should succeed");
        let digest2 =
            FingerprintAdapter::digest_artifacts(root).expect("second digest should succeed");

        assert_eq!(
            digest1, digest2,
            "identical artifacts must produce identical fingerprints"
        );
    }
}
