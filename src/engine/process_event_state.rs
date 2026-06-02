use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProcessEventState {
    pub events: Vec<ProcessEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessEvent {
    pub kind: String,
    pub verdict: String,
    pub timestamp: String,
    pub details: Option<String>,
}
