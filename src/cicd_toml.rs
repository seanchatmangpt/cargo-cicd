use serde::{Deserialize, Serialize};

/// The root configuration structure for cicd.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CicdConfig {
    #[serde(default)]
    pub workspace: WorkspaceConfig,

    #[serde(default)]
    pub state: StateConfig,

    #[serde(default)]
    pub target: TargetConfig,

    #[serde(default)]
    pub test: TestConfig,

    #[serde(default)]
    pub trybuild: TrybuildConfig,

    #[serde(default)]
    pub git: GitConfig,

    #[serde(default)]
    pub autonomic: AutonomicConfig,

    #[serde(default)]
    pub events: Vec<EventConfig>,
}

/// [workspace] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub toolchain: String,

    #[serde(default)]
    pub target_dir: String,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            toolchain: String::new(),
            target_dir: String::new(),
        }
    }
}

/// [state] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    #[serde(default)]
    pub dirty: bool,

    #[serde(default)]
    pub target_size_gb: f64,

    #[serde(default)]
    pub changed_files: usize,

    #[serde(default)]
    pub changed_tests: usize,

    #[serde(default)]
    pub changed_trybuild_fixtures: usize,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            dirty: false,
            target_size_gb: 0.0,
            changed_files: 0,
            changed_tests: 0,
            changed_trybuild_fixtures: 0,
        }
    }
}

/// [target] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    #[serde(default = "default_max_size")]
    pub max_size_gb: f64,

    #[serde(default = "default_prune_days")]
    pub prune_after_days: usize,
}

fn default_max_size() -> f64 {
    20.0
}

fn default_prune_days() -> usize {
    14
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            max_size_gb: default_max_size(),
            prune_after_days: default_prune_days(),
        }
    }
}

/// [test.changed] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestChangedConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub base: String,
}

impl Default for TestChangedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base: "origin/main".to_string(),
        }
    }
}

/// [test] section (contains subsections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    #[serde(default)]
    pub changed: TestChangedConfig,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            changed: TestChangedConfig::default(),
        }
    }
}

/// [trybuild.changed] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrybuildChangedConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub snapshot_mode: String,
}

impl Default for TrybuildChangedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            snapshot_mode: "changed-only".to_string(),
        }
    }
}

/// [trybuild] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrybuildConfig {
    #[serde(default)]
    pub changed: TrybuildChangedConfig,
}

impl Default for TrybuildConfig {
    fn default() -> Self {
        Self {
            changed: TrybuildChangedConfig::default(),
        }
    }
}

/// [git.phase] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPhaseConfig {
    #[serde(default)]
    pub require_clean_tree: bool,

    #[serde(default)]
    pub commit_after_phase: bool,
}

impl Default for GitPhaseConfig {
    fn default() -> Self {
        Self {
            require_clean_tree: true,
            commit_after_phase: false,
        }
    }
}

/// [git] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default)]
    pub phase: GitPhaseConfig,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            phase: GitPhaseConfig::default(),
        }
    }
}

/// [autonomic] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub mode: String,
}

impl Default for AutonomicConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "suggest".to_string(),
        }
    }
}

/// [[events]] section (array of events)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    #[serde(default)]
    pub kind: String,

    #[serde(default)]
    pub verdict: String,
}

impl CicdConfig {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to a TOML file.
    pub fn to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(&self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CicdConfig::default();
        assert_eq!(config.target.max_size_gb, 20.0);
        assert_eq!(config.target.prune_after_days, 14);
        assert!(config.autonomic.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = CicdConfig {
            workspace: WorkspaceConfig {
                name: "my-project".to_string(),
                toolchain: "stable".to_string(),
                target_dir: "./target".to_string(),
            },
            state: StateConfig {
                dirty: false,
                target_size_gb: 2.5,
                changed_files: 3,
                changed_tests: 2,
                changed_trybuild_fixtures: 1,
            },
            ..Default::default()
        };

        let toml_str = toml::to_string(&config).expect("serialization failed");
        assert!(toml_str.contains("my-project"));
        assert!(toml_str.contains("stable"));
    }

    #[test]
    fn test_test_changed_defaults() {
        let test_config = TestConfig::default();
        assert!(test_config.changed.enabled);
        assert_eq!(test_config.changed.base, "origin/main");
    }

    #[test]
    fn test_git_phase_defaults() {
        let git_config = GitConfig::default();
        assert!(git_config.phase.require_clean_tree);
        assert!(!git_config.phase.commit_after_phase);
    }
}

impl Default for CicdConfig {
    fn default() -> Self {
        Self {
            workspace: WorkspaceConfig::default(),
            state: StateConfig::default(),
            target: TargetConfig::default(),
            test: TestConfig::default(),
            trybuild: TrybuildConfig::default(),
            git: GitConfig::default(),
            autonomic: AutonomicConfig::default(),
            events: Vec::new(),
        }
    }
}
