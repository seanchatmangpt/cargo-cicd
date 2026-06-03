use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyState {
    pub name: String,
    pub mode: PolicyMode,
    pub signals: Vec<String>,
    pub recommendation: String,
    pub verdict: PolicyVerdict,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyMode {
    Suggest,
    Apply,
    Disabled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyVerdict {
    Pass,
    Warn,
    Fail,
}
