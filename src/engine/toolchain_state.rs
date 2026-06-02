use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ToolchainState {
    pub active: String,
    pub rust_version: String,
    pub is_nightly: bool,
    pub mismatch_detected: bool,
    pub required: Option<String>,
}
