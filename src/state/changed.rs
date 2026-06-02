use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFileState {
    pub path: std::path::PathBuf,
    pub kind: ChangeKind,
    pub affected_tests: Vec<String>,
    pub affected_fixtures: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeKind {
    Source,
    Test,
    Macro,
    BuildScript,
    Manifest,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedTestState {
    pub test_name: String,
    pub source_file: std::path::PathBuf,
    pub fixture_paths: Vec<std::path::PathBuf>,
}
