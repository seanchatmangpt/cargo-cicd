use crate::state::target::{TargetState, TargetVerdict};
use anyhow::Result;
use std::path::Path;

/// Read the target directory state (size, verdict).
pub fn read_target_state(target_dir: &Path, max_size_gb: f64) -> Result<TargetState> {
    let total_size_bytes = dir_size_bytes(target_dir);
    let total_size_gb = total_size_bytes as f64 / 1_073_741_824.0;

    let verdict = if total_size_gb >= max_size_gb {
        TargetVerdict::Fail
    } else if total_size_gb >= max_size_gb * 0.8 {
        TargetVerdict::Warn
    } else {
        TargetVerdict::Pass
    };

    Ok(TargetState {
        path: target_dir.to_path_buf(),
        total_size_gb,
        max_size_gb,
        verdict,
    })
}

fn dir_size_bytes(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
