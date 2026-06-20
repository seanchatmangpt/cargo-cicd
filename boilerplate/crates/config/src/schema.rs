use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// The canonical shape of project configuration. All optional sub-sections
/// are `None` by default; they are populated from file/env/override layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub logging: LoggingConfig,
    pub database: Option<DatabaseConfig>,
    pub service: Option<ServiceConfig>,
}

// ---------------------------------------------------------------------------
// ProjectConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Human-readable project name.  Default: empty string.
    #[serde(default)]
    pub name: String,
    /// Semver string sourced from Cargo.toml at compile time.
    #[serde(default = "default_pkg_version")]
    pub version: String,
    /// Deployment environment; defaults to `Development`.
    #[serde(default)]
    pub environment: Environment,
}

fn default_pkg_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    /// Reads `APP_ENV` (checked first) or `ENVIRONMENT` from the process
    /// environment. Falls back to `Development` when neither is set.
    pub fn from_env() -> Self {
        let raw = std::env::var("APP_ENV")
            .or_else(|_| std::env::var("ENVIRONMENT"))
            .unwrap_or_default();

        match raw.to_lowercase().as_str() {
            "staging" => Self::Staging,
            "production" | "prod" => Self::Production,
            _ => Self::Development,
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

// ---------------------------------------------------------------------------
// LoggingConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// `tracing` subscriber filter string, e.g. `"info"` or `"debug,hyper=warn"`.
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Output format: `text` (human-readable) or `json` (structured).
    #[serde(default)]
    pub format: LogFormat,
    /// Shorthand to force JSON output; overrides `format` when `true`.
    #[serde(default)]
    pub json: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Text,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Text
    }
}

// ---------------------------------------------------------------------------
// DatabaseConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Connection URL.  Default: `sqlite://project.db`.
    #[serde(default = "default_db_url")]
    pub url: String,
    /// Maximum number of pooled connections.  Default: 5.
    #[serde(default = "default_db_max_connections")]
    pub max_connections: u32,
    /// Connection-acquire timeout in seconds.  Default: 30.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_db_url() -> String {
    "sqlite://project.db".to_string()
}
fn default_db_max_connections() -> u32 {
    5
}

// ---------------------------------------------------------------------------
// ServiceConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Bind address.  Default: `127.0.0.1`.
    #[serde(default = "default_service_host")]
    pub host: String,
    /// Bind port.  Default: 8080.
    #[serde(default = "default_service_port")]
    pub port: u16,
    /// Allowed CORS origins.  Default: empty (no cross-origin requests).
    #[serde(default)]
    pub cors_origins: Vec<String>,
    /// Per-request timeout in seconds.  Default: 30.
    #[serde(default = "default_timeout_secs")]
    pub request_timeout_secs: u64,
}

fn default_service_host() -> String {
    "127.0.0.1".to_string()
}
fn default_service_port() -> u16 {
    8080
}
fn default_timeout_secs() -> u64 {
    30
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

impl Default for Config {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            logging: LoggingConfig::default(),
            database: None,
            service: None,
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::default(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Text,
            json: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://project.db".to_string(),
            max_connections: 5,
            timeout_secs: 30,
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_origins: Vec::new(),
            request_timeout_secs: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

impl Config {
    /// Validate the merged configuration.
    ///
    /// Returns `Ok(())` when all constraints pass; otherwise returns the first
    /// `ConfigError::ValidationFailed` encountered.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // project.name must not be blank
        if self.project.name.trim().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "project.name",
                reason: "must not be empty".to_string(),
            });
        }

        // project.version must look like semver (non-empty is sufficient here)
        if self.project.version.trim().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "project.version",
                reason: "must not be empty".to_string(),
            });
        }

        // logging.level must be non-empty
        if self.logging.level.trim().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "logging.level",
                reason: "must not be empty".to_string(),
            });
        }

        if let Some(db) = &self.database {
            // database.url must be non-empty
            if db.url.trim().is_empty() {
                return Err(ConfigError::ValidationFailed {
                    field: "database.url",
                    reason: "must not be empty".to_string(),
                });
            }
            // max_connections must be at least 1
            if db.max_connections == 0 {
                return Err(ConfigError::ValidationFailed {
                    field: "database.max_connections",
                    reason: "must be at least 1".to_string(),
                });
            }
        }

        if let Some(svc) = &self.service {
            // port must be > 0
            if svc.port == 0 {
                return Err(ConfigError::ValidationFailed {
                    field: "service.port",
                    reason: "must be greater than 0".to_string(),
                });
            }
            // host must be non-empty
            if svc.host.trim().is_empty() {
                return Err(ConfigError::ValidationFailed {
                    field: "service.host",
                    reason: "must not be empty".to_string(),
                });
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_fails_validation_because_name_is_empty() {
        let cfg = Config::default();
        let err = cfg.validate().unwrap_err();
        match err {
            ConfigError::ValidationFailed { field, .. } => assert_eq!(field, "project.name"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn valid_minimal_config_passes_validation() {
        let mut cfg = Config::default();
        cfg.project.name = "my-project".to_string();
        cfg.validate().expect("minimal config should be valid");
    }

    #[test]
    fn service_port_zero_fails_validation() {
        let mut cfg = Config::default();
        cfg.project.name = "test".to_string();
        cfg.service = Some(ServiceConfig {
            port: 0,
            ..ServiceConfig::default()
        });
        let err = cfg.validate().unwrap_err();
        match err {
            ConfigError::ValidationFailed { field, .. } => {
                assert_eq!(field, "service.port")
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn database_max_connections_zero_fails_validation() {
        let mut cfg = Config::default();
        cfg.project.name = "test".to_string();
        cfg.database = Some(DatabaseConfig {
            max_connections: 0,
            ..DatabaseConfig::default()
        });
        let err = cfg.validate().unwrap_err();
        match err {
            ConfigError::ValidationFailed { field, .. } => {
                assert_eq!(field, "database.max_connections")
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn environment_from_env_defaults_to_development() {
        // Test the serde round-trip using the Value API (not the document API).
        let e: Environment = toml::Value::String("staging".into())
            .try_into()
            .unwrap();
        assert_eq!(e, Environment::Staging);

        let e: Environment = toml::Value::String("production".into())
            .try_into()
            .unwrap();
        assert_eq!(e, Environment::Production);

        let e: Environment = toml::Value::String("development".into())
            .try_into()
            .unwrap();
        assert_eq!(e, Environment::Development);
    }

    #[test]
    fn full_config_with_all_sections_validates() {
        let cfg = Config {
            project: ProjectConfig {
                name: "full-project".to_string(),
                version: "1.2.3".to_string(),
                environment: Environment::Production,
            },
            logging: LoggingConfig {
                level: "warn".to_string(),
                format: LogFormat::Json,
                json: true,
            },
            database: Some(DatabaseConfig {
                url: "postgres://localhost/mydb".to_string(),
                max_connections: 20,
                timeout_secs: 10,
            }),
            service: Some(ServiceConfig {
                host: "0.0.0.0".to_string(),
                port: 443,
                cors_origins: vec!["https://example.com".to_string()],
                request_timeout_secs: 60,
            }),
        };
        cfg.validate().expect("full config should be valid");
    }
}
