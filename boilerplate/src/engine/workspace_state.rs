//! Workspace identity state dimension.

/// Workspace identity — name, root path, and crate membership.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceState {
    /// The `[package] name` or `[workspace]` display name.
    pub name: String,
    /// Absolute path to the workspace root (directory containing the root `Cargo.toml`).
    pub root_path: String,
    /// Relative paths of all workspace member crates.
    pub members: Vec<String>,
    /// Rust edition declared in the root manifest.
    pub edition: String,
    /// Minimum Supported Rust Version, if declared.
    pub rust_version: Option<String>,
}
