//! Toolchain adapter — shells out to `rustc --version`.

#![cfg(feature = "process-data")]

use crate::engine::ToolchainState;
use std::process::Command;

/// Populates [`ToolchainState`] by invoking `rustc --version`.
pub struct ToolchainAdapter;

impl ToolchainAdapter {
    /// Build a [`ToolchainState`] from the currently active `rustc`.
    ///
    /// Returns `Default` state silently if `rustc` is not on `PATH`.
    pub fn populate() -> ToolchainState {
        match Self::try_populate() {
            Ok(state) => state,
            Err(err) => {
                tracing::warn!("ToolchainAdapter failed, using defaults: {err}");
                ToolchainState::default()
            }
        }
    }

    fn try_populate() -> anyhow::Result<ToolchainState> {
        let output = Command::new("rustc")
            .arg("--version")
            .arg("--verbose")
            .output()?;

        if !output.status.success() {
            anyhow::bail!("rustc --version exited with status {}", output.status);
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut state = ToolchainState::default();

        for line in text.lines() {
            if line.starts_with("rustc ") {
                state.rust_version = line.trim().to_owned();
            } else if let Some(val) = line.strip_prefix("release: ") {
                // Derive channel from the release string.
                state.channel = if val.contains("nightly") {
                    "nightly"
                } else if val.contains("beta") {
                    "beta"
                } else {
                    "stable"
                }
                .to_owned();
            } else if let Some(val) = line.strip_prefix("host: ") {
                state.host = val.trim().to_owned();
            }
        }

        // If verbose output wasn't available, fall back to the simple version line.
        if state.rust_version.is_empty() {
            state.rust_version = text.lines().next().unwrap_or("").trim().to_owned();
        }
        if state.channel.is_empty() {
            state.channel = "stable".to_owned();
        }

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_does_not_panic() {
        let _state = ToolchainAdapter::populate();
    }
}
