use crate::state::toolchain::ToolchainState;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Workspace-level metadata derived from `cargo metadata --format-version 1 --no-deps`.
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// The workspace (package) name from Cargo.toml.
    pub name: String,
    /// Absolute path to the workspace root.
    pub root: PathBuf,
    /// Absolute path to the Cargo target directory.
    pub target_dir: PathBuf,
}

/// Read workspace metadata by running `cargo metadata --format-version 1 --no-deps`.
pub fn read_workspace() -> Result<WorkspaceInfo> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let json: serde_json::Value = serde_json::from_slice(&out.stdout)?;

    let root = json
        .get("workspace_root")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let target_dir = json
        .get("target_directory")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));

    // Derive name: prefer first workspace member package name, then directory name.
    let name = json
        .get("packages")
        .and_then(|pkgs| pkgs.as_array())
        .and_then(|arr| arr.first())
        .and_then(|pkg| pkg.get("name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            root.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "workspace".into())
        });

    Ok(WorkspaceInfo {
        name,
        root,
        target_dir,
    })
}

/// Read toolchain state from the workspace root.
pub fn read_toolchain(root: &Path) -> Result<ToolchainState> {
    let channel = detect_toolchain_channel(root);
    let version = detect_toolchain_version(root);
    let pinned = root.join("rust-toolchain.toml").exists() || root.join("rust-toolchain").exists();

    Ok(ToolchainState {
        channel,
        version,
        pinned,
        matches_cicd_toml: true, // resolved at policy layer
    })
}

fn detect_toolchain_channel(root: &Path) -> String {
    let toml_path = root.join("rust-toolchain.toml");
    if toml_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&toml_path) {
            if contents.contains("nightly") {
                return "nightly".to_string();
            }
            if contents.contains("beta") {
                return "beta".to_string();
            }
            return "stable".to_string();
        }
    }
    "stable".to_string()
}

fn detect_toolchain_version(root: &Path) -> Option<String> {
    let out = Command::new("rustc")
        .args(["--version"])
        .current_dir(root)
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
