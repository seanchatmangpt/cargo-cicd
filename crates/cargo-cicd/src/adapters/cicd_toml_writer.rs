use crate::cicd_toml::CicdToml;
use anyhow::Result;
use std::path::Path;

pub struct CicdTomlWriter;

impl CicdTomlWriter {
    pub fn new() -> Self {
        Self
    }

    /// Snapshot the current workspace state into `path` and return the written config.
    pub fn write_current_state(path: &str) -> Result<CicdToml> {
        let cicd = CicdToml::from_current_workspace();
        cicd.write(path)?;
        Ok(cicd)
    }
}

impl Default for CicdTomlWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialize `cicd` to TOML and write it to `path`.
pub fn write_cicd_toml(cicd: &CicdToml, path: &Path) -> Result<()> {
    cicd.write(path)
}
