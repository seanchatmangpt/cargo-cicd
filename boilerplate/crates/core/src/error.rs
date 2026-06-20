//! Typed error taxonomy for the workspace.
//!
//! All domain-level errors are defined here as variants of [`CoreError`].
//! Callers that need ergonomic propagation should wrap these with `anyhow`.

use thiserror::Error;

/// The canonical `Result` alias for this crate — errors are [`CoreError`].
pub type Result<T, E = CoreError> = std::result::Result<T, E>;

/// Domain-level error taxonomy.
///
/// Add a new variant for each distinct failure mode that a *caller* may need
/// to handle programmatically.  Failures that are always fatal and never
/// matched on should be propagated via `anyhow::Error` instead.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoreError {
    /// The workspace root could not be located or is not a Cargo workspace.
    #[error("workspace not found: {reason}")]
    WorkspaceNotFound {
        /// Human-readable explanation of why the workspace was not found.
        reason: String,
    },

    /// A required configuration field is missing or has an invalid value.
    #[error("configuration error in `{field}`: {reason}")]
    ConfigInvalid {
        /// The name of the invalid configuration field.
        field: &'static str,
        /// Human-readable explanation of the validation failure.
        reason: String,
    },

    /// An I/O operation failed.  Wraps `std::io::Error`.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization or deserialization step failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// An external process (git, rustc, cargo) returned a non-zero exit code.
    #[error("external process `{command}` failed with exit code {code}")]
    ProcessFailed {
        /// The command that was invoked.
        command: String,
        /// The non-zero exit code returned by the process.
        code: i32,
    },

    /// An external process was not found on `PATH`.
    #[error("required program `{program}` not found on PATH")]
    ProgramNotFound {
        /// The name of the program that could not be found.
        program: String,
    },

    /// A verdict that was expected has not been received.
    #[error("verdict required but oracle is unavailable")]
    OracleUnavailable,

    /// A public-boundary invariant was violated.
    #[error("invariant `{name}` violated: {details}")]
    InvariantViolated {
        /// The name of the invariant that was violated.
        name: &'static str,
        /// A detailed description of the violation.
        details: String,
    },
}

impl CoreError {
    /// Convenience constructor for [`CoreError::WorkspaceNotFound`].
    pub fn workspace_not_found(reason: impl Into<String>) -> Self {
        Self::WorkspaceNotFound { reason: reason.into() }
    }

    /// Convenience constructor for [`CoreError::ConfigInvalid`].
    pub fn config_invalid(field: &'static str, reason: impl Into<String>) -> Self {
        Self::ConfigInvalid { field, reason: reason.into() }
    }

    /// Convenience constructor for [`CoreError::ProcessFailed`].
    pub fn process_failed(command: impl Into<String>, code: i32) -> Self {
        Self::ProcessFailed { command: command.into(), code }
    }

    /// Convenience constructor for [`CoreError::ProgramNotFound`].
    pub fn program_not_found(program: impl Into<String>) -> Self {
        Self::ProgramNotFound { program: program.into() }
    }

    /// Convenience constructor for [`CoreError::InvariantViolated`].
    pub fn invariant_violated(name: &'static str, details: impl Into<String>) -> Self {
        Self::InvariantViolated { name, details: details.into() }
    }
}
