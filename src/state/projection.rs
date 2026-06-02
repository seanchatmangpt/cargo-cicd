use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionProfile {
    pub level: u8,
    pub public_surface: Vec<String>,
}
