use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetState {
    pub path: std::path::PathBuf,
    pub total_size_gb: f64,
    pub max_size_gb: f64,
    pub verdict: TargetVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetVerdict {
    Pass,
    Warn,
    Fail,
}
