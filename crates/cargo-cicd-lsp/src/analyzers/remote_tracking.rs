//! RemoteTrackingAnalyzer — raises CICD-GIT-003 when local branch is behind its upstream.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Checks whether the local branch is behind its remote tracking branch.
pub struct RemoteTrackingAnalyzer;

impl CicdAnalyzer for RemoteTrackingAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        // Run `git rev-list --count HEAD..@{u}` to count commits behind remote.
        let output = std::process::Command::new("git")
            .args(["rev-list", "--count", "HEAD..@{u}"])
            .current_dir(&snapshot.root)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let count_str = String::from_utf8_lossy(&out.stdout);
                let count: u32 = count_str.trim().parse().unwrap_or(0);
                if count > 0 {
                    findings.push(CicdFinding::new(
                        CicdCode::BranchBehindRemote,
                        ".git/HEAD",
                        "git remote",
                        vec!["git pull --rebase".to_string()],
                        format!(
                            "Local branch is {} commit(s) behind remote; run `git pull --rebase`.",
                            count
                        ),
                    ));
                }
            }
            _ => {
                // No upstream configured or git unavailable — not an error.
            }
        }

        findings
    }

    fn name(&self) -> &'static str {
        "RemoteTrackingAnalyzer"
    }
}
