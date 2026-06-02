use crate::state::toolchain::ToolchainState;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

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
    if s.is_empty() { None } else { Some(s) }
}
