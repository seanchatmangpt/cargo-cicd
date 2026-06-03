use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TrybuildState {
    pub all_fixtures: Vec<String>,
    pub changed_fixtures: Vec<String>,
    pub snapshot_mode: String,
    pub run_all_by_default: bool,
}
