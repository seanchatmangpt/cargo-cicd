//! Allowlisted command execution for cargo-cicd LSP code actions.

const PERMITTED_COMMANDS: &[&str] = &[
    "cargo cicd git status",
    "cargo cicd git close --repo . --json",
    "cargo cicd target show",
    "cargo cicd target prune --dry-run",
    "cargo cicd test changed",
    "cargo cicd workspace doctor",
    "cargo cicd evidence doctor",
];

/// Execute a command only if it appears in the allowlist.
///
/// Returns stdout on success, stderr on failure.
pub async fn execute_permitted(cmd: &str) -> Result<String, String> {
    if !PERMITTED_COMMANDS.contains(&cmd) {
        return Err(format!("Command not in allowlist: {}", cmd));
    }

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (program, args) = parts.split_first().ok_or_else(|| "Empty command".to_string())?;

    let output = tokio::process::Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn permitted_command_accepted() {
        // Should not panic — result may be Ok or Err depending on whether cargo-cicd is installed,
        // but the allowlist check must pass.
        let result = execute_permitted("cargo cicd git status").await;
        // The command may fail if cargo-cicd is not installed, but not due to allowlist rejection.
        match result {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    !e.contains("Command not in allowlist"),
                    "unexpected allowlist rejection: {e}"
                );
            }
        }
    }

    #[tokio::test]
    async fn forbidden_command_rejected() {
        let result = execute_permitted("rm -rf /").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Command not in allowlist"));
    }
}
