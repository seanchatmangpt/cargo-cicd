use std::process::Command;

pub struct ToolchainDetector;

impl ToolchainDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn active_toolchain() -> String {
        let output = Command::new("rustup")
            .args(["show", "active-toolchain"])
            .output();
        if let Ok(out) = output {
            return String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .next()
                .unwrap_or("stable")
                .to_string();
        }
        "stable".into()
    }

    pub fn rust_version() -> String {
        let output = Command::new("rustc").args(["--version"]).output();
        if let Ok(out) = output {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
        "unknown".into()
    }
}

impl Default for ToolchainDetector {
    fn default() -> Self {
        Self::new()
    }
}
