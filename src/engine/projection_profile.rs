use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProjectionProfile {
    pub version: String,
    pub public_level: u8,
    pub suppress_private_fields: bool,
}

impl ProjectionProfile {
    pub fn v26_7_6() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").into(),
            public_level: 2,
            suppress_private_fields: true,
        }
    }
}
