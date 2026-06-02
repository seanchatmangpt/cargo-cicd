use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ChangedFileState {
    pub base_ref: String,
    pub changed_rs_files: Vec<String>,
    pub changed_test_files: Vec<String>,
    pub changed_trybuild_fixtures: Vec<String>,
    pub total_changed: usize,
}
