//! GitPhaseAnalyzer — raises CICD-GIT-001 and CICD-GIT-002.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Analyzes git working-tree state and raises findings for dirty or untracked conditions.
pub struct GitPhaseAnalyzer;

impl CicdAnalyzer for GitPhaseAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        if snapshot.git_status.dirty {
            findings.push(CicdFinding::new(
                CicdCode::GitDirtyTreeBlocksClose,
                ".git/index",
                "src/",
                vec![
                    "cargo cicd git status".to_string(),
                    "cargo cicd git close".to_string(),
                ],
                "Working tree has uncommitted changes. Stage and commit before closing.",
            ));
        }

        if snapshot.git_status.untracked_count > 0 {
            findings.push(CicdFinding::new(
                CicdCode::GitUntrackedArtifacts,
                ".git/index",
                "src/",
                vec![
                    "cargo cicd git status".to_string(),
                    "cargo cicd git close".to_string(),
                ],
                format!(
                    "{} untracked file(s) present. Add to .gitignore or stage intentionally.",
                    snapshot.git_status.untracked_count
                ),
            ));
        }

        findings
    }

    fn name(&self) -> &'static str {
        "GitPhaseAnalyzer"
    }
}
