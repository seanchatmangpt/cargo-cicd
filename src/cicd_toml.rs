use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// v26.6.2 cicd.toml schema — the carrier contract
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CicdToml {
    pub workspace: WorkspaceSection,
    pub state: StateSection,
    pub target: TargetSection,
    #[serde(rename = "test")]
    pub test: TestSection,
    #[serde(rename = "trybuild")]
    pub trybuild: TrybuildSection,
    #[serde(rename = "git")]
    pub git: GitSection,
    pub autonomic: AutonomicSection,
    #[serde(default)]
    pub events: Vec<EventRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceSection {
    pub name: String,
    pub toolchain: String,
    pub target_dir: String,
}

impl Default for WorkspaceSection {
    fn default() -> Self {
        Self {
            name: detect_workspace_name(),
            toolchain: detect_toolchain(),
            target_dir: "target".into(),
        }
    }
}

fn detect_workspace_name() -> String {
    // Read workspace name from Cargo.toml if available
    std::fs::read_to_string("Cargo.toml")
        .ok()
        .and_then(|s| {
            s.lines().find(|l| l.starts_with("name = ")).map(|l| {
                l.trim_start_matches("name = ")
                    .trim_matches('"')
                    .to_string()
            })
        })
        .unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "workspace".into())
        })
}

fn detect_toolchain() -> String {
    // Check rust-toolchain.toml or rust-toolchain
    if let Ok(content) = std::fs::read_to_string("rust-toolchain.toml") {
        if let Some(line) = content.lines().find(|l| l.contains("channel")) {
            if let Some(ch) = line.split('"').nth(1) {
                return ch.to_string();
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string("rust-toolchain") {
        return content.trim().to_string();
    }
    "stable".into()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct StateSection {
    pub dirty: bool,
    pub target_size_gb: f64,
    pub changed_files: usize,
    pub changed_tests: usize,
    pub changed_trybuild_fixtures: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetSection {
    pub max_size_gb: u32,
    pub prune_after_days: u32,
}

impl Default for TargetSection {
    fn default() -> Self {
        Self {
            max_size_gb: 20,
            prune_after_days: 14,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TestSection {
    pub changed: TestChangedSection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestChangedSection {
    pub enabled: bool,
    pub base: String,
}

impl Default for TestChangedSection {
    fn default() -> Self {
        Self {
            enabled: true,
            base: "origin/main".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TrybuildSection {
    pub changed: TrybuildChangedSection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrybuildChangedSection {
    pub enabled: bool,
    pub snapshot_mode: String,
}

impl Default for TrybuildChangedSection {
    fn default() -> Self {
        Self {
            enabled: true,
            snapshot_mode: "changed-only".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GitSection {
    pub phase: GitPhaseSection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitPhaseSection {
    pub require_clean_tree: bool,
    pub commit_after_phase: bool,
}

impl Default for GitPhaseSection {
    fn default() -> Self {
        Self {
            require_clean_tree: true,
            commit_after_phase: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutonomicSection {
    pub enabled: bool,
    pub mode: String,
}

impl Default for AutonomicSection {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "suggest".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventRecord {
    pub kind: String,
    pub verdict: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl EventRecord {
    pub fn status_pass() -> Self {
        Self {
            kind: "status".into(),
            verdict: "pass".into(),
            details: None,
            timestamp: None,
        }
    }
}

impl CicdToml {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Write configuration to a TOML file (compat alias).
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        self.write(path)
    }

    /// Build from current workspace state
    pub fn from_current_workspace() -> Self {
        let mut cicd = Self::default();
        cicd.events.push(EventRecord::status_pass());
        cicd
    }

    /// Write to the given path
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize cicd.toml: {}", e))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Write to ./cicd.toml
    pub fn write_default(&self) -> Result<()> {
        self.write("cicd.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cicd_toml() {
        let config = CicdToml::default();
        assert_eq!(config.target.max_size_gb, 20);
        assert_eq!(config.target.prune_after_days, 14);
        assert!(config.autonomic.enabled);
        assert_eq!(config.autonomic.mode, "suggest");
        assert!(config.git.phase.require_clean_tree);
        assert!(!config.git.phase.commit_after_phase);
        assert!(config.test.changed.enabled);
        assert_eq!(config.test.changed.base, "origin/main");
        assert!(config.trybuild.changed.enabled);
        assert_eq!(config.trybuild.changed.snapshot_mode, "changed-only");
    }

    #[test]
    fn test_write_and_read_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cicd.toml");
        let config = CicdToml::default();
        config.write(&path).expect("write failed");
        let loaded = CicdToml::from_file(&path).expect("read failed");
        assert_eq!(loaded.target.max_size_gb, config.target.max_size_gb);
        assert_eq!(loaded.autonomic.mode, config.autonomic.mode);
    }

    #[test]
    fn test_from_current_workspace_has_status_pass_event() {
        let cicd = CicdToml::from_current_workspace();
        assert_eq!(cicd.events.len(), 1);
        assert_eq!(cicd.events[0].kind, "status");
        assert_eq!(cicd.events[0].verdict, "pass");
        assert!(cicd.events[0].details.is_none());
        assert!(cicd.events[0].timestamp.is_none());
    }

    #[test]
    fn test_event_record_optional_fields_skip_serialization() {
        let record = EventRecord::status_pass();
        let serialized = toml::to_string_pretty(&record).unwrap();
        assert!(!serialized.contains("details"));
        assert!(!serialized.contains("timestamp"));
    }
}
