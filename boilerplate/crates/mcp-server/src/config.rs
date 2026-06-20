//! Server configuration for the MCP server.
//!
//! [`ServerConfig`] holds all runtime knobs.  It can be constructed
//! programmatically, defaulted, or populated from environment variables via
//! [`ServerConfig::from_env`].

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// TransportKind
// ---------------------------------------------------------------------------

/// Which transport the MCP server listens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportKind {
    /// Standard I/O (stdin / stdout) — the default and most compatible mode.
    ///
    /// Claude Desktop, Cursor, and most MCP hosts support stdio transport.
    Stdio,

    /// HTTP + SSE (Server-Sent Events) transport.
    ///
    /// Useful for web-based or multi-client scenarios.  The bind address
    /// should be in `host:port` format, e.g. `"0.0.0.0:8080"`.
    Http {
        /// TCP address the HTTP server binds to (e.g. `"127.0.0.1:8080"`).
        bind_addr: String,
    },
}

impl Default for TransportKind {
    fn default() -> Self {
        Self::Stdio
    }
}

// ---------------------------------------------------------------------------
// ServerConfig
// ---------------------------------------------------------------------------

/// Runtime configuration for [`crate::ProjectMcpServer`].
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Which transport to use.
    pub transport: TransportKind,

    /// Absolute path to the workspace root.
    ///
    /// All file-access tools validate that the requested path is under this
    /// root (path-traversal protection).
    pub workspace_root: PathBuf,

    /// Maximum number of seconds a spawned subprocess may run before being
    /// killed with a [`crate::error::McpError::TimeoutError`].
    pub timeout_secs: u64,

    /// Maximum file size (in bytes) that the `read_file` tool will return.
    ///
    /// Requests for files larger than this limit are rejected with a
    /// descriptive error.  Default: 1 MiB.
    pub max_file_size_bytes: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: TransportKind::Stdio,
            workspace_root: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from(".")),
            timeout_secs: 60,
            max_file_size_bytes: 1024 * 1024, // 1 MiB
        }
    }
}

impl ServerConfig {
    /// Construct a config from environment variables, falling back to
    /// [`Default`] values for anything not set.
    ///
    /// | Variable              | Effect                                          |
    /// |-----------------------|-------------------------------------------------|
    /// | `MCP_BIND_ADDR`       | If set, use HTTP transport on this address.     |
    /// | `MCP_WORKSPACE_ROOT`  | Override the workspace root path.              |
    /// | `MCP_TIMEOUT_SECS`    | Override the subprocess timeout (default 60).  |
    /// | `MCP_MAX_FILE_BYTES`  | Override the max-file-size limit (default 1 MiB). |
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        // Transport: if MCP_BIND_ADDR is set, switch to HTTP.
        if let Ok(addr) = std::env::var("MCP_BIND_ADDR") {
            cfg.transport = TransportKind::Http { bind_addr: addr };
        }

        // Workspace root override.
        if let Ok(root) = std::env::var("MCP_WORKSPACE_ROOT") {
            cfg.workspace_root = PathBuf::from(root);
        }

        // Timeout override.
        if let Ok(secs) = std::env::var("MCP_TIMEOUT_SECS") {
            if let Ok(n) = secs.parse::<u64>() {
                cfg.timeout_secs = n;
            }
        }

        // Max file size override.
        if let Ok(bytes) = std::env::var("MCP_MAX_FILE_BYTES") {
            if let Ok(n) = bytes.parse::<usize>() {
                cfg.max_file_size_bytes = n;
            }
        }

        cfg
    }

    /// Return `true` if the server is configured for stdio transport.
    pub fn is_stdio(&self) -> bool {
        self.transport == TransportKind::Stdio
    }

    /// Return the bind address for HTTP transport, or `None` for stdio.
    pub fn http_bind_addr(&self) -> Option<&str> {
        match &self.transport {
            TransportKind::Http { bind_addr } => Some(bind_addr.as_str()),
            TransportKind::Stdio => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_stdio() {
        let cfg = ServerConfig::default();
        assert!(cfg.is_stdio());
        assert_eq!(cfg.timeout_secs, 60);
        assert_eq!(cfg.max_file_size_bytes, 1024 * 1024);
    }

    #[test]
    fn from_env_http_transport() {
        // Temporarily set environment variables.
        std::env::set_var("MCP_BIND_ADDR", "127.0.0.1:9999");
        let cfg = ServerConfig::from_env();
        std::env::remove_var("MCP_BIND_ADDR");

        assert!(!cfg.is_stdio());
        assert_eq!(cfg.http_bind_addr(), Some("127.0.0.1:9999"));
    }

    #[test]
    fn from_env_workspace_root() {
        std::env::set_var("MCP_WORKSPACE_ROOT", "/tmp/my-project");
        let cfg = ServerConfig::from_env();
        std::env::remove_var("MCP_WORKSPACE_ROOT");

        assert_eq!(cfg.workspace_root, PathBuf::from("/tmp/my-project"));
    }

    #[test]
    fn from_env_timeout_override() {
        std::env::set_var("MCP_TIMEOUT_SECS", "120");
        let cfg = ServerConfig::from_env();
        std::env::remove_var("MCP_TIMEOUT_SECS");

        assert_eq!(cfg.timeout_secs, 120);
    }

    #[test]
    fn from_env_invalid_timeout_uses_default() {
        std::env::set_var("MCP_TIMEOUT_SECS", "not-a-number");
        let cfg = ServerConfig::from_env();
        std::env::remove_var("MCP_TIMEOUT_SECS");

        // Falls back to the default value.
        assert_eq!(cfg.timeout_secs, 60);
    }

    #[test]
    fn http_bind_addr_none_for_stdio() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.http_bind_addr(), None);
    }
}
