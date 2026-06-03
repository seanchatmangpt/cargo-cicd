use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ArtifactState {
    pub cicd_toml_path: Option<String>,
    pub last_published: Option<String>,
}
