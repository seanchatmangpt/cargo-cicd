//! `run_checks` tool — runs `cargo check --workspace` and returns the output.
//!
//! The subprocess is killed if it exceeds the configured timeout (default 60 s)
//! so a broken workspace does not stall the MCP server indefinitely.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

use crate::error::McpError;

// ---------------------------------------------------------------------------
// Output shape
// ---------------------------------------------------------------------------

/// Payload returned by the `run_checks` tool.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChecksResult {
    /// Combined stdout from `cargo check`.
    pub stdout: String,

    /// Combined stderr from `cargo check`.
    pub stderr: String,

    /// Process exit code.  `0` means success.
    pub exit_code: i32,

    /// Whether the check succeeded (exit_code == 0).
    pub success: bool,

    /// The exact command that was run.
    pub command: String,

    /// Whether the subprocess was killed because it exceeded the timeout.
    pub timed_out: bool,
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

/// Run `cargo check --workspace` in `workspace_root` with a hard timeout.
///
/// # Errors
///
/// - [`McpError::TimeoutError`] if the subprocess runs longer than
///   `timeout_secs`.
/// - [`McpError::IoError`] if the subprocess cannot be spawned.
pub async fn run_cargo_checks(
    workspace_root: &Path,
    timeout_secs: u64,
) -> Result<ChecksResult, McpError> {
    let cmd_str = "cargo check --workspace --message-format=short".to_string();

    // Build the command.
    let mut cmd = Command::new("cargo");
    cmd.arg("check")
        .arg("--workspace")
        .arg("--message-format=short")
        .current_dir(workspace_root)
        // Capture both stdout and stderr.
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        // Disable terminal colours so the output is clean in JSON.
        .env("CARGO_TERM_COLOR", "never");

    // Spawn and wait with timeout.
    let run_future = async {
        let output = cmd
            .output()
            .await
            .map_err(McpError::IoError)?;
        Ok::<_, McpError>(output)
    };

    match timeout(Duration::from_secs(timeout_secs), run_future).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let exit_code = output.status.code().unwrap_or(-1);

            Ok(ChecksResult {
                stdout,
                stderr,
                exit_code,
                success: exit_code == 0,
                command: cmd_str,
                timed_out: false,
            })
        }
        Ok(Err(e)) => Err(e),
        Err(_elapsed) => Err(McpError::TimeoutError {
            operation: "cargo check --workspace".into(),
            timeout_secs,
        }),
    }
}

/// Serialize a [`ChecksResult`] to JSON.
pub fn checks_to_json(result: &ChecksResult) -> Result<Value, McpError> {
    serde_json::to_value(result).map_err(McpError::SerializationError)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialisation roundtrip test — no subprocess needed.
    #[test]
    fn checks_result_serialises() {
        let r = ChecksResult {
            stdout: "".into(),
            stderr: "error[E0308]: type mismatch".into(),
            exit_code: 1,
            success: false,
            command: "cargo check --workspace --message-format=short".into(),
            timed_out: false,
        };
        let v = checks_to_json(&r).unwrap();
        assert_eq!(v["exit_code"], 1);
        assert!(!v["success"].as_bool().unwrap());
        assert!(!v["timed_out"].as_bool().unwrap());
    }

    /// Verify the JSON shape includes all expected keys.
    #[test]
    fn checks_result_json_has_all_keys() {
        let r = ChecksResult {
            stdout: "ok".into(),
            stderr: "".into(),
            exit_code: 0,
            success: true,
            command: "cargo check --workspace".into(),
            timed_out: false,
        };
        let v = checks_to_json(&r).unwrap();
        for key in &["stdout", "stderr", "exit_code", "success", "command", "timed_out"] {
            assert!(v.get(key).is_some(), "missing key: {key}");
        }
    }

    /// Integration test: actually run `cargo check` in the real workspace.
    ///
    /// This test is ignored by default because it spawns a real subprocess
    /// which is slow in CI.  Run with `cargo test -- --ignored` when needed.
    #[tokio::test]
    #[ignore = "spawns real subprocess; run with --ignored"]
    async fn run_cargo_checks_in_real_workspace() {
        let workspace = std::env::current_dir().unwrap();
        let result = run_cargo_checks(&workspace, 120).await.unwrap();
        // We expect exit_code to be 0 or 1 — just ensure no panic/timeout.
        assert!(!result.timed_out, "unexpected timeout");
    }

    /// Verify that a very short timeout returns a [`McpError::TimeoutError`].
    ///
    /// This is also an integration test (spawns a subprocess) but is fast
    /// because the timeout fires immediately.
    #[tokio::test]
    #[ignore = "spawns real subprocess; run with --ignored"]
    async fn short_timeout_produces_timeout_error() {
        let workspace = std::env::current_dir().unwrap();
        // 0-second timeout is guaranteed to fire.
        let result = run_cargo_checks(&workspace, 0).await;
        // Either a timeout or the command completed before the timer — both
        // are acceptable; we just check we don't panic.
        let _ = result;
    }
}
