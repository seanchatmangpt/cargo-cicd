//! `project_status` tool — returns workspace metadata as JSON.
//!
//! Reads the top-level `Cargo.toml` of the workspace root and returns the
//! package/workspace name, version (if present), workspace members, and the
//! `rust-version` field (if set).

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::McpError;

// ---------------------------------------------------------------------------
// Output shape
// ---------------------------------------------------------------------------

/// JSON payload returned by the `project_status` tool.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatusResult {
    /// Workspace or package name, as found in `Cargo.toml`.
    pub name: String,

    /// Package version string (e.g. `"0.1.0"`), or `null` if this is a
    /// virtual workspace manifest with no `[package]` table.
    pub version: Option<String>,

    /// Workspace members listed in `[workspace] members = [...]`.
    ///
    /// For single-crate projects this will be an empty list.
    pub members: Vec<String>,

    /// The `rust-version` field from `[package]` or `[workspace.package]`,
    /// or `null` if not declared.
    pub rust_version: Option<String>,
}

// ---------------------------------------------------------------------------
// Minimal TOML parser (no proc-macro dependency needed at runtime)
// ---------------------------------------------------------------------------

/// Read and parse `Cargo.toml` in `workspace_root`, returning a
/// [`ProjectStatusResult`].
///
/// We parse only the fields we need using `serde_json::Value` over the
/// TOML string, converted to JSON via the `toml` crate.
/// If `toml` is not in scope we fall back to a line-by-line heuristic so
/// the crate compiles without an extra dependency.
pub fn read_project_status(workspace_root: &Path) -> Result<ProjectStatusResult, McpError> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let raw = std::fs::read_to_string(&cargo_toml_path).map_err(|e| {
        McpError::WorkspaceError(anyhow::anyhow!(
            "Cannot read Cargo.toml at {}: {e}",
            cargo_toml_path.display()
        ))
    })?;

    parse_cargo_toml(&raw)
}

/// Parse a `Cargo.toml` string and extract the fields we care about.
///
/// Uses a simple line-by-line scan so that the crate has no `toml` crate
/// dependency at the library level.  This is intentionally minimal — it
/// handles the common case and degrades gracefully for complex manifests.
fn parse_cargo_toml(raw: &str) -> Result<ProjectStatusResult, McpError> {
    // Attempt to use the `toml` crate if it happens to be compiled in.
    // Because we cannot control whether `toml` is a dependency from this
    // boilerplate, we always use the line-by-line fallback which handles the
    // 95 % case: straightforward single-value assignments, no TOML arrays
    // spread across multiple lines except the `members` array.
    let name = extract_string_field(raw, "name").unwrap_or_else(|| "unknown".into());
    let version = extract_string_field(raw, "version");
    let rust_version = extract_string_field(raw, "rust-version");
    let members = extract_members_array(raw);

    Ok(ProjectStatusResult { name, version, members, rust_version })
}

/// Extract a simple `key = "value"` assignment from a TOML string.
fn extract_string_field(raw: &str, key: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        // Match `key = "value"` or `key = 'value'`
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                // Quoted string
                if rest.starts_with('"') || rest.starts_with('\'') {
                    let quote = rest.chars().next().unwrap();
                    let inner = rest.trim_matches(quote);
                    // Remove trailing inline comment
                    let value = inner.split('#').next().unwrap_or(inner).trim().trim_matches(quote);
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Extract a TOML inline array from `members = [...]`.
///
/// Handles both inline `members = ["a", "b"]` and simple multi-line arrays
/// (one member per line between `[` and `]`).
fn extract_members_array(raw: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut in_members = false;
    let mut buffer = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if in_members {
            buffer.push_str(trimmed);
            buffer.push(' ');
            if trimmed.contains(']') {
                break;
            }
        } else if let Some(rest) = trimmed.strip_prefix("members") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                buffer.push_str(rest.trim());
                if rest.contains(']') {
                    break;
                }
                in_members = true;
            }
        }
    }

    // Parse what we collected between `[` and `]`.
    if let (Some(start), Some(end)) = (buffer.find('['), buffer.rfind(']')) {
        let inner = &buffer[start + 1..end];
        for item in inner.split(',') {
            let item = item.trim().trim_matches('"').trim_matches('\'').trim();
            if !item.is_empty() {
                members.push(item.to_string());
            }
        }
    }

    members
}

/// Convert a [`ProjectStatusResult`] to an MCP-ready JSON [`Value`].
pub fn status_to_json(result: &ProjectStatusResult) -> Result<Value, McpError> {
    serde_json::to_value(result).map_err(McpError::SerializationError)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const WORKSPACE_MANIFEST: &str = r#"
[workspace]
members = ["crates/core", "crates/mcp-server"]
resolver = "2"

[workspace.package]
rust-version = "1.75"
"#;

    const PACKAGE_MANIFEST: &str = r#"
[package]
name = "my-tool"
version = "1.2.3"
rust-version = "1.74"

[dependencies]
serde = "1"
"#;

    #[test]
    fn parses_workspace_manifest() {
        let result = parse_cargo_toml(WORKSPACE_MANIFEST).unwrap();
        assert_eq!(result.members, vec!["crates/core", "crates/mcp-server"]);
    }

    #[test]
    fn parses_package_name_and_version() {
        let result = parse_cargo_toml(PACKAGE_MANIFEST).unwrap();
        assert_eq!(result.name, "my-tool");
        assert_eq!(result.version.as_deref(), Some("1.2.3"));
        assert_eq!(result.rust_version.as_deref(), Some("1.74"));
    }

    #[test]
    fn no_members_for_single_crate() {
        let result = parse_cargo_toml(PACKAGE_MANIFEST).unwrap();
        assert!(result.members.is_empty());
    }

    #[test]
    fn status_to_json_roundtrip() {
        let s = ProjectStatusResult {
            name: "test".into(),
            version: Some("0.1.0".into()),
            members: vec!["crates/a".into()],
            rust_version: None,
        };
        let v = status_to_json(&s).unwrap();
        assert_eq!(v["name"], "test");
        assert_eq!(v["version"], "0.1.0");
        assert!(v["rust_version"].is_null());
    }

    #[test]
    fn read_project_status_missing_file() {
        let result = read_project_status(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Cannot read Cargo.toml"));
    }
}
