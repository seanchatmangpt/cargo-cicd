use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PolicyState {
    pub policies: Vec<PolicyEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyEntry {
    pub name: String,
    pub enabled: bool,
    pub mode: String,
    pub verdict: Option<String>,
    pub recommendation: Option<String>,
}
