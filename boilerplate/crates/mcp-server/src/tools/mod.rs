//! Tool modules for the MCP server.
//!
//! Each submodule implements one tool that is exposed to the AI assistant.
//! The tool logic (input parsing, safety checks, I/O) lives here; the MCP
//! plumbing (attribute macros, routing) lives in [`crate::server`].
//!
//! ## Adding a new tool
//!
//! 1. Create `src/tools/my_tool.rs`.
//! 2. Implement the core logic as a plain `fn` or `async fn` that returns
//!    `Result<SomeResult, McpError>`.
//! 3. Add `pub mod my_tool;` below.
//! 4. In [`crate::server`]:
//!    - Add an `async fn` method with `#[tool(...)]`.
//!    - Add it to the `tool_router!` macro.

pub mod checks;
pub mod commands;
pub mod fs;
pub mod status;

// Convenience re-exports so callers can write `tools::ChecksResult` instead
// of `tools::checks::ChecksResult`.
pub use checks::{run_cargo_checks, ChecksResult};
pub use commands::{list_commands, CommandEntry};
pub use fs::{read_file_safe, ReadFileResult};
pub use status::{read_project_status, ProjectStatusResult};
