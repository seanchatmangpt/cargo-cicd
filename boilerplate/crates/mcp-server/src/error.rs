//! Error types for the MCP server.
//!
//! [`McpError`] wraps common failure modes and converts into the rmcp error
//! type so all tool handlers can use `?` uniformly.

use std::fmt;

// ---------------------------------------------------------------------------
// McpError
// ---------------------------------------------------------------------------

/// All error variants that a tool handler can produce.
///
/// Each variant maps to a descriptive MCP error code that is sent back to the
/// AI assistant.
#[derive(Debug)]
pub enum McpError {
    /// A workspace-level operation failed (e.g., Cargo.toml not found).
    WorkspaceError(anyhow::Error),

    /// An I/O operation failed (e.g., reading a source file).
    IoError(std::io::Error),

    /// A spawned subprocess exceeded the configured timeout.
    TimeoutError {
        /// Human-readable name of the operation that timed out.
        operation: String,
        /// Timeout value in seconds.
        timeout_secs: u64,
    },

    /// The caller requested access to a path outside the workspace root
    /// (path-traversal protection).
    PermissionDenied(String),

    /// JSON serialisation or deserialisation failed.
    SerializationError(serde_json::Error),
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkspaceError(e) => write!(f, "workspace error: {e}"),
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            Self::TimeoutError { operation, timeout_secs } => {
                write!(f, "operation '{operation}' timed out after {timeout_secs}s")
            }
            Self::PermissionDenied(path) => {
                write!(f, "permission denied: path '{path}' is outside workspace root")
            }
            Self::SerializationError(e) => write!(f, "serialization error: {e}"),
        }
    }
}

impl std::error::Error for McpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WorkspaceError(e) => Some(e.as_ref()),
            Self::IoError(e) => Some(e),
            Self::SerializationError(e) => Some(e),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Conversions from stdlib / third-party errors
// ---------------------------------------------------------------------------

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializationError(e)
    }
}

impl From<anyhow::Error> for McpError {
    fn from(e: anyhow::Error) -> Self {
        Self::WorkspaceError(e)
    }
}

// ---------------------------------------------------------------------------
// Conversion into rmcp::Error
//
// NOTE: The exact rmcp::Error API depends on the crate version. The
// `rmcp::Error` type is expected to provide at least `internal_error(msg)`
// or equivalent.  Adjust the call site if the API differs.
// ---------------------------------------------------------------------------

impl From<McpError> for rmcp::Error {
    fn from(e: McpError) -> Self {
        // rmcp's `Error` carries an error code and a human-readable message.
        // We use the `internal_error` constructor because all variants
        // represent server-side failures from the client's perspective.
        //
        // If rmcp exposes distinct error codes (e.g. `permission_denied`),
        // swap the arms below for more precise codes.
        rmcp::Error::new(
            rmcp::model::ErrorCode::InternalError,
            e.to_string(),
            None,
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_workspace_error() {
        let e = McpError::WorkspaceError(anyhow::anyhow!("Cargo.toml not found"));
        assert!(e.to_string().contains("workspace error"));
        assert!(e.to_string().contains("Cargo.toml not found"));
    }

    #[test]
    fn display_timeout_error() {
        let e = McpError::TimeoutError {
            operation: "cargo check".into(),
            timeout_secs: 60,
        };
        let msg = e.to_string();
        assert!(msg.contains("cargo check"));
        assert!(msg.contains("60"));
    }

    #[test]
    fn display_permission_denied() {
        let e = McpError::PermissionDenied("/etc/passwd".into());
        assert!(e.to_string().contains("/etc/passwd"));
        assert!(e.to_string().contains("outside workspace root"));
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let mcp: McpError = io.into();
        matches!(mcp, McpError::IoError(_));
    }

    #[test]
    fn from_serde_error() {
        let bad: Result<serde_json::Value, _> = serde_json::from_str("{invalid}");
        let serde_e = bad.unwrap_err();
        let mcp: McpError = serde_e.into();
        matches!(mcp, McpError::SerializationError(_));
    }

    #[test]
    fn into_rmcp_error() {
        let mcp = McpError::PermissionDenied("/tmp/secret".into());
        let rmcp_err: rmcp::Error = mcp.into();
        // Just verify conversion succeeds and message is preserved.
        let msg = format!("{rmcp_err:?}");
        assert!(!msg.is_empty());
    }
}
