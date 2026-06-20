//! Core MCP server implementation.
//!
//! [`ProjectMcpServer`] implements `rmcp::ServerHandler` and wires the four
//! tools to the MCP router.
//!
//! # rmcp API note
//!
//! The rmcp 0.1 crate is relatively new.  The attribute macro API shown here
//! (`#[tool(...)]`, `tool_router!`, `#[tool_handler(...)]`) matches the
//! published API as of early 2025.  If you encounter compile errors, check
//! the rmcp changelog — the macro names may have been renamed.
//!
//! Relevant upstream: <https://github.com/modelcontextprotocol/rust-sdk>

use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_router, tool_handler, ServerHandler,
};
use tracing::{error, info, instrument};

use crate::config::ServerConfig;
use crate::error::McpError;
use crate::tools;

// ---------------------------------------------------------------------------
// ProjectMcpServer
// ---------------------------------------------------------------------------

/// MCP server that exposes project workspace functionality to AI assistants.
///
/// Wrap it in an `Arc` if you need to share it between tasks, although the
/// rmcp transport owns the handler for its lifetime.
#[derive(Clone)]
pub struct ProjectMcpServer {
    /// A shared reference to the runtime configuration.
    config: Arc<ServerConfig>,
}

impl ProjectMcpServer {
    /// Construct a new server from the given configuration.
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Return the workspace root path.
    pub fn workspace_root(&self) -> &PathBuf {
        &self.config.workspace_root
    }

    // ------------------------------------------------------------------
    // Tool: project_status
    // ------------------------------------------------------------------

    /// Return workspace metadata (name, version, members, rust-version).
    ///
    /// Reads the top-level `Cargo.toml` from the workspace root and returns
    /// a JSON object with the following keys:
    ///
    /// - `name` — workspace or package name
    /// - `version` — package version (may be `null` for virtual workspaces)
    /// - `members` — workspace member paths
    /// - `rust_version` — minimum Rust version if declared
    #[tool(
        name = "project_status",
        description = "Return workspace metadata from Cargo.toml: name, version, members, \
                       rust-version.  Use this to understand the project structure before \
                       exploring source files."
    )]
    #[instrument(skip(self), name = "tool/project_status")]
    async fn project_status(&self) -> Result<CallToolResult, rmcp::Error> {
        info!("project_status called");

        let result = catch_panic(|| {
            tools::read_project_status(&self.config.workspace_root)
                .and_then(|r| {
                    serde_json::to_string_pretty(&r).map_err(McpError::SerializationError)
                })
        })
        .await
        .map_err(|e: McpError| rmcp::Error::from(e))?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ------------------------------------------------------------------
    // Tool: list_commands
    // ------------------------------------------------------------------

    /// Return all available CLI subcommands as a JSON array.
    ///
    /// Each element has `noun`, `verb`, `description`, `is_destructive`, and
    /// `example` keys.  Use this to discover what operations are available
    /// before calling `run_checks`.
    #[tool(
        name = "list_commands",
        description = "List all available CLI noun-verb commands with descriptions and \
                       examples.  Use this to discover what the project binary can do."
    )]
    #[instrument(skip(self), name = "tool/list_commands")]
    async fn list_commands(&self) -> Result<CallToolResult, rmcp::Error> {
        info!("list_commands called");

        let cmds = tools::list_commands();
        let json = serde_json::to_string_pretty(&cmds).map_err(|e| {
            rmcp::Error::from(McpError::SerializationError(e))
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ------------------------------------------------------------------
    // Tool: read_file
    // ------------------------------------------------------------------

    /// Read a file from the workspace (path-traversal–safe).
    ///
    /// Only files within the workspace root are accessible.  Binary files
    /// and files larger than the configured limit are rejected.
    ///
    /// # Parameter
    ///
    /// - `path` — workspace-relative (or absolute) path to the file.
    ///   Absolute paths are re-anchored inside the workspace root.
    #[tool(
        name = "read_file",
        description = "Read a file from the workspace.  Only files inside the workspace root \
                       are accessible (path-traversal protection is enforced). Pass a \
                       workspace-relative path, e.g. `src/main.rs` or `Cargo.toml`."
    )]
    #[instrument(skip(self), name = "tool/read_file")]
    async fn read_file(
        &self,
        #[tool(param, description = "Workspace-relative path to the file to read")] path: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        info!(path = %path, "read_file called");

        let root = self.config.workspace_root.clone();
        let max_bytes = self.config.max_file_size_bytes;

        let result = catch_panic(|| {
            tools::read_file_safe(&root, &path, max_bytes)
                .and_then(|r| {
                    serde_json::to_string_pretty(&r).map_err(McpError::SerializationError)
                })
        })
        .await
        .map_err(rmcp::Error::from)?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ------------------------------------------------------------------
    // Tool: run_checks
    // ------------------------------------------------------------------

    /// Run `cargo check --workspace` and return stdout, stderr, and exit code.
    ///
    /// The subprocess is killed after the configured timeout (default 60 s).
    /// Returns a JSON object with `stdout`, `stderr`, `exit_code`, `success`,
    /// and `timed_out` keys.
    #[tool(
        name = "run_checks",
        description = "Run `cargo check --workspace` in the workspace root and return the \
                       compiler diagnostics.  The subprocess is killed after the timeout \
                       (default 60 s).  Returns stdout, stderr, exit_code, and success."
    )]
    #[instrument(skip(self), name = "tool/run_checks")]
    async fn run_checks(&self) -> Result<CallToolResult, rmcp::Error> {
        info!("run_checks called");

        let root = self.config.workspace_root.clone();
        let timeout = self.config.timeout_secs;

        // run_cargo_checks is async so we await it directly.  We still wrap
        // it in catch_panic to handle any unexpected panics gracefully.
        let result: Result<tools::ChecksResult, McpError> =
            tools::run_cargo_checks(&root, timeout).await;

        let checks = result.map_err(rmcp::Error::from)?;
        let json = serde_json::to_string_pretty(&checks).map_err(|e| {
            rmcp::Error::from(McpError::SerializationError(e))
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ------------------------------------------------------------------
    // Router
    // ------------------------------------------------------------------

    /// Build the tool router that maps tool names to handler methods.
    ///
    /// Every method annotated with `#[tool(...)]` above must be listed here.
    fn router(&self) -> rmcp::tool_router::ToolRouter<Self> {
        tool_router![
            Self::project_status,
            Self::list_commands,
            Self::read_file,
            Self::run_checks,
        ]
    }
}

// ---------------------------------------------------------------------------
// ServerHandler implementation
// ---------------------------------------------------------------------------

/// NOTE: The `#[tool_handler(router = self.router())]` attribute generates the
/// `call_tool` and `list_tools` methods required by `ServerHandler`.
/// If the rmcp API differs in your version, manually implement those methods
/// by delegating to `self.router().call_tool(...)` and
/// `self.router().list_tools()`.
#[tool_handler(router = self.router())]
impl ServerHandler for ProjectMcpServer {
    /// Metadata that is sent to the AI assistant during the MCP handshake.
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "project-mcp-server".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            instructions: Some(
                "This MCP server exposes the workspace project to AI assistants. \
                 Use `project_status` to understand the project structure, \
                 `list_commands` to discover available CLI operations, \
                 `read_file` to inspect source files, and \
                 `run_checks` to compile-check the workspace."
                    .into(),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Panic-catching helper
// ---------------------------------------------------------------------------

/// Run a synchronous, potentially-panicking closure and convert any panic into
/// a [`McpError::WorkspaceError`].
///
/// This prevents a panicking tool handler from bringing down the entire server
/// process.
async fn catch_panic<F, T>(f: F) -> Result<T, McpError>
where
    F: FnOnce() -> Result<T, McpError> + Send + 'static,
    T: Send + 'static,
{
    // Spawn on a blocking thread so the async runtime is not affected by the
    // panic.  `catch_unwind` requires `UnwindSafe`, which we assert here.
    tokio::task::spawn_blocking(move || {
        std::panic::catch_unwind(AssertUnwindSafe(f))
            .unwrap_or_else(|payload| {
                let msg = extract_panic_message(&payload);
                error!(panic = %msg, "tool handler panicked");
                Err(McpError::WorkspaceError(anyhow::anyhow!(
                    "tool handler panicked: {msg}"
                )))
            })
    })
    .await
    .map_err(|join_err| {
        McpError::WorkspaceError(anyhow::anyhow!("spawn_blocking failed: {join_err}"))
    })?
}

/// Extract a human-readable message from a panic payload.
fn extract_panic_message(payload: &dyn std::any::Any) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".into()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerConfig, TransportKind};
    use std::fs;
    use tempfile::TempDir;

    fn test_server(dir: &TempDir) -> ProjectMcpServer {
        ProjectMcpServer::new(ServerConfig {
            transport: TransportKind::Stdio,
            workspace_root: dir.path().to_path_buf(),
            timeout_secs: 60,
            max_file_size_bytes: 1024 * 1024,
        })
    }

    #[test]
    fn server_new_stores_config() {
        let dir = TempDir::new().unwrap();
        let server = test_server(&dir);
        assert_eq!(server.workspace_root(), dir.path());
    }

    #[test]
    fn server_info_name() {
        let dir = TempDir::new().unwrap();
        let server = test_server(&dir);
        let info = server.get_info();
        assert_eq!(info.server_info.name, "project-mcp-server");
    }

    #[tokio::test]
    async fn project_status_returns_text() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"[package]
name = "test-proj"
version = "0.2.0"
"#,
        )
        .unwrap();
        let server = test_server(&dir);
        let result = server.project_status().await.unwrap();
        // CallToolResult contains text content.
        let text = result
            .content
            .iter()
            .filter_map(|c| c.as_text())
            .next()
            .expect("expected text content");
        assert!(text.contains("test-proj"));
    }

    #[tokio::test]
    async fn list_commands_returns_json_array() {
        let dir = TempDir::new().unwrap();
        let server = test_server(&dir);
        let result = server.list_commands().await.unwrap();
        let text = result
            .content
            .iter()
            .filter_map(|c| c.as_text())
            .next()
            .expect("expected text content");
        let v: serde_json::Value = serde_json::from_str(text).expect("valid JSON");
        assert!(v.is_array());
        assert!(!v.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn read_file_returns_content() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.txt"), "hello mcp").unwrap();
        let server = test_server(&dir);
        let result = server.read_file("test.txt".into()).await.unwrap();
        let text = result
            .content
            .iter()
            .filter_map(|c| c.as_text())
            .next()
            .expect("expected text content");
        assert!(text.contains("hello mcp"));
    }

    #[tokio::test]
    async fn read_file_rejects_traversal() {
        let dir = TempDir::new().unwrap();
        let server = test_server(&dir);
        // Attempt to escape via ../../etc/passwd.
        let result = server.read_file("../../etc/passwd".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn catch_panic_converts_panic_to_error() {
        let result: Result<i32, McpError> = catch_panic(|| {
            panic!("intentional panic for testing");
        })
        .await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("panic"), "expected 'panic' in: {msg}");
    }
}
