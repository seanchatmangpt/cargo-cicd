use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub kind: String,
    pub verdict: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}
