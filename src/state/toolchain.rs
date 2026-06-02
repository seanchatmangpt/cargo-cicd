use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainState {
    pub channel: String,
    pub version: Option<String>,
    pub pinned: bool,
    pub matches_cicd_toml: bool,
}
