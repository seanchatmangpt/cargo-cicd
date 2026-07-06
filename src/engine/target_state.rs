use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TargetState {
    pub path: String,
    pub total_size_bytes: u64,
    pub max_size_bytes: u64,
    pub prune_after_days: u32,
    pub stale_profiles: Vec<String>,
    pub verdict: TargetVerdict,
    /// Number of entries under `target/` that could not be read while
    /// scanning (permission errors, races, etc.). Non-zero means the
    /// reported size may be an undercount.
    pub scan_errors: usize,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetVerdict {
    #[default]
    Pass,
    Warn,
    Fail,
}

impl std::fmt::Display for TargetVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "pass"),
            Self::Warn => write!(f, "warn"),
            Self::Fail => write!(f, "fail"),
        }
    }
}
