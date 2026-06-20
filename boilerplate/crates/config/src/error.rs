use std::path::PathBuf;
use thiserror::Error;

/// All errors that can arise when loading or validating configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The specified config file does not exist on disk.
    #[error("config file not found: {0}")]
    FileNotFound(PathBuf),

    /// The config file exists but could not be parsed as valid TOML.
    #[error("failed to parse config file {path}: {source}")]
    ParseError {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// A required field failed its validation constraint.
    #[error("validation failed for field `{field}`: {reason}")]
    ValidationFailed {
        field: &'static str,
        reason: String,
    },

    /// An environment variable was present but its value could not be used.
    #[error("invalid value for env var `{var}`: {reason}")]
    EnvVarInvalid { var: String, reason: String },

    /// The TOML merge or deserialization step failed.
    #[error("config merge error: {0}")]
    MergeError(String),

    /// I/O error while reading a config file.
    #[error("I/O error reading config: {0}")]
    Io(#[from] std::io::Error),
}
