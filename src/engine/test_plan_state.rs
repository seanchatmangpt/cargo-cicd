use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TestPlanState {
    pub selected_tests: Vec<String>,
    pub conservative_mode: bool,
    pub reason: Option<String>,
    pub estimated_count: usize,
}
