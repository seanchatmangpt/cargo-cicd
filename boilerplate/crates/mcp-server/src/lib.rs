//! `project-mcp-server` — MCP server exposing workspace functionality to AI assistants.
//!
//! # Quick start
//!
//! ```no_run
//! use project_mcp_server::{ProjectMcpServer, ServerConfig, TransportKind};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ServerConfig::from_env();
//!     let server = ProjectMcpServer::new(config);
//!     // See main.rs for transport wiring.
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! main.rs
//!   └─ ServerConfig::from_env()
//!        └─ ProjectMcpServer::new(config)
//!             └─ rmcp transport (stdio or HTTP/SSE)
//!                  └─ ServerHandler::call_tool()
//!                       ├─ tools::status   → read_project_status()
//!                       ├─ tools::commands → list_commands()
//!                       ├─ tools::fs       → read_file_safe()
//!                       └─ tools::checks   → run_cargo_checks()
//! ```
//!
//! # Security
//!
//! The `read_file` tool enforces path-traversal protection: only files under
//! the workspace root (resolved to their canonical path) are accessible.
//! See [`tools::fs`] for details.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod error;
pub mod server;
pub mod tools;

// ---------------------------------------------------------------------------
// Public re-exports
// ---------------------------------------------------------------------------

/// The main MCP server struct.  Start here.
pub use server::ProjectMcpServer;

/// Runtime configuration: transport, workspace root, timeouts.
pub use config::{ServerConfig, TransportKind};

/// All error variants produced by tool handlers.
pub use error::McpError;
