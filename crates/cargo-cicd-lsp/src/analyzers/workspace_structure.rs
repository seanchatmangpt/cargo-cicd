//! WorkspaceStructureAnalyzer — raises CICD-WORKSPACE-001 when Cargo.toml is absent or unreadable.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Validates basic workspace Cargo.toml structure.
pub struct WorkspaceStructureAnalyzer;

impl CicdAnalyzer for WorkspaceStructureAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        let cargo_toml = snapshot.root.join("Cargo.toml");

        if !cargo_toml.exists() {
            findings.push(CicdFinding::new(
                CicdCode::WorkspaceStructureInvalid,
                "workspace root",
                "Cargo.toml",
                vec!["cargo cicd workspace validate".to_string()],
                "Cargo.toml not found at workspace root.",
            ));
            return findings;
        }

        // Check that Cargo.toml is readable.
        if std::fs::read_to_string(&cargo_toml).is_err() {
            findings.push(CicdFinding::new(
                CicdCode::WorkspaceStructureInvalid,
                cargo_toml.to_string_lossy().as_ref(),
                "Cargo.toml",
                vec!["cargo cicd workspace validate".to_string()],
                "Cargo.toml exists but cannot be read.",
            ));
        }

        findings
    }

    fn name(&self) -> &'static str {
        "WorkspaceStructureAnalyzer"
    }
}
