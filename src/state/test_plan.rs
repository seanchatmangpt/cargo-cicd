use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlan {
    pub entries: Vec<TestPlanEntry>,
    pub total: usize,
    pub verdict: TestPlanVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlanEntry {
    pub test_name: String,
    pub reason: String,
    pub fixture_paths: Vec<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestPlanVerdict {
    Pass,
    Warn,
    Fail,
    Empty,
}
