use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProjectionProfile {
    pub version: String,
    pub public_level: u8,
    pub suppress_private_fields: bool,
}

impl ProjectionProfile {
    pub fn v26_6_2() -> Self {
        Self { version: "26.6.2".into(), public_level: 2, suppress_private_fields: true }
    }
}
