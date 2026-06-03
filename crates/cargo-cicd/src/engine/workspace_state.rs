use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct WorkspaceState {
    pub name: String,
    pub root_path: String,
    pub members: Vec<String>,
    pub toolchain: String,
    pub rust_edition: String,
}
