use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPhaseState {
    pub branch: String,
    pub dirty_files: Vec<std::path::PathBuf>,
    pub staged_files: Vec<std::path::PathBuf>,
    pub untracked: Vec<std::path::PathBuf>,
    pub ahead: usize,
    pub behind: usize,
    pub recommended_action: String,
}
