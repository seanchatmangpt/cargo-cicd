use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Size information for a Cargo target directory.
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// Absolute path to the target directory.
    pub path: PathBuf,
    /// Total size in bytes.
    pub size_bytes: u64,
    /// Total size in gigabytes.
    pub size_gb: f64,
}

/// Scan the target directory and return size information.
pub fn scan_target(path: &Path) -> Result<TargetInfo> {
    let size_bytes: u64 = if path.exists() {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
    } else {
        0
    };
    let size_gb = size_bytes as f64 / 1_073_741_824.0;
    Ok(TargetInfo { path: path.to_path_buf(), size_bytes, size_gb })
}

/// Identify candidate subdirectories for pruning: `incremental/` and `deps/` subtrees.
///
/// Returns paths that are well-known cache directories inside the target tree.
pub fn suggest_prune_candidates(path: &Path) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();
    if !path.exists() {
        return Ok(candidates);
    }
    // Walk one level of profile dirs (debug, release, etc.) and collect known prune targets.
    for profile_entry in std::fs::read_dir(path)? {
        let profile_entry = match profile_entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let profile_path = profile_entry.path();
        if !profile_path.is_dir() {
            continue;
        }
        for subdir in &["incremental", ".fingerprint"] {
            let candidate = profile_path.join(subdir);
            if candidate.is_dir() {
                candidates.push(candidate);
            }
        }
    }
    Ok(candidates)
}
