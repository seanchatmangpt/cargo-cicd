//! Direct Cargo.toml parsing without cargo metadata overhead.
//!
//! This module provides lightweight parsing of Cargo.toml manifests
//! using the `toml` crate, avoiding the overhead of `cargo metadata`.

use std::path::Path;
use toml::Value;

/// Parse workspace member names directly from Cargo.toml.
///
/// # Errors
///
/// Returns an error if the file cannot be read, TOML cannot be parsed,
/// or if the [workspace] members array is not present.
pub fn parse_workspace_members(manifest_path: &Path) -> anyhow::Result<Vec<String>> {
    let content = std::fs::read_to_string(manifest_path)?;
    let table: Value = toml::from_str(&content)?;

    let members = table
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .ok_or_else(|| anyhow::anyhow!("no [workspace] members array"))?;

    Ok(members
        .iter()
        .filter_map(|m| m.as_str().map(|s| s.to_string()))
        .collect())
}

/// Parse package name from Cargo.toml.
///
/// # Errors
///
/// Returns an error if the file cannot be read, TOML cannot be parsed,
/// or if the [package] name field is not present.
pub fn parse_package_name(manifest_path: &Path) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(manifest_path)?;
    let table: Value = toml::from_str(&content)?;

    table
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("no [package] name field"))
        .map(|s| s.to_string())
}

/// Parse [workspace.package] metadata if present.
///
/// Returns `Ok(None)` if the workspace.package section does not exist.
///
/// # Errors
///
/// Returns an error if the file cannot be read or TOML cannot be parsed.
pub fn parse_workspace_package_metadata(manifest_path: &Path) -> anyhow::Result<Option<Value>> {
    let content = std::fs::read_to_string(manifest_path)?;
    let table: Value = toml::from_str(&content)?;
    Ok(table
        .get("workspace")
        .and_then(|w| w.get("package"))
        .cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_members_from_root_cargo_toml() {
        let members = parse_workspace_members(Path::new("Cargo.toml")).unwrap();
        assert!(!members.is_empty());
        assert!(members.iter().any(|m| m == "."));
    }

    #[test]
    fn parse_package_name_from_root_cargo_toml() {
        let name = parse_package_name(Path::new("Cargo.toml")).unwrap();
        assert_eq!(name, "cargo-cicd");
    }

    #[test]
    fn parse_workspace_package_metadata_returns_option() {
        let metadata = parse_workspace_package_metadata(Path::new("Cargo.toml")).unwrap();
        // The root Cargo.toml may or may not have [workspace.package],
        // but the function should not error either way
        let _ = metadata;
    }
}
