use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub name: String,
    pub root: std::path::PathBuf,
    pub toolchain: String,
    pub target_dir: std::path::PathBuf,
    pub dirty: bool,
    pub target_size_gb: f64,
    pub changed_files: usize,
    pub changed_tests: usize,
    pub changed_trybuild_fixtures: usize,
}
