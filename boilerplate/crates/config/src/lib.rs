//! # project-config
//!
//! Layered configuration for the project workspace.
//!
//! ## Priority order (highest wins)
//!
//! 1. Programmatic overrides — [`ConfigLoader::with_overrides`]
//! 2. Environment variables  — [`ConfigLoader::with_env`]
//! 3. Config file            — [`ConfigLoader::with_file`] / [`ConfigLoader::with_file_optional`]
//! 4. Compiled-in defaults   — always present as the base
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use project_config::{Config, ConfigLoader, ConfigError};
//!
//! // Standard resolution: defaults → project.toml (if present) → APP_* env vars
//! let config: Config = project_config::load_config()?;
//!
//! println!("project: {}", config.project.name);
//! println!("log level: {}", config.logging.level);
//! # Ok::<(), ConfigError>(())
//! ```
//!
//! ## Custom stack
//!
//! ```rust,no_run
//! use project_config::{Config, ConfigLoader, ConfigError};
//!
//! let config = ConfigLoader::new()
//!     .with_file_optional("/etc/myapp/config.toml")
//!     .with_file_optional("project.toml")
//!     .with_env("MYAPP")
//!     .load()?;
//! # Ok::<(), ConfigError>(())
//! ```

pub mod error;
pub mod layer;
pub mod loader;
pub mod schema;
pub mod watcher;

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use error::ConfigError;
pub use loader::{load_config, ConfigLoader};
pub use schema::{
    Config, DatabaseConfig, Environment, LogFormat, LoggingConfig, ProjectConfig, ServiceConfig,
};
