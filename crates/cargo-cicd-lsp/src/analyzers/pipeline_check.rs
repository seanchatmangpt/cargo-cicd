//! PipelineCheckAnalyzer — raises CICD-PIPELINE-002 when cicd.toml is absent.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Checks that cicd.toml is present at the workspace root.
pub struct PipelineCheckAnalyzer;

impl CicdAnalyzer for PipelineCheckAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        // CICD-PIPELINE-002: cicd.toml not found at workspace root.
        let cicd_toml = snapshot.root.join("cicd.toml");
        if !cicd_toml.exists() {
            findings.push(CicdFinding::new(
                CicdCode::NoCicdTomlFound,
                "workspace root",
                "cicd.toml",
                vec!["cargo cicd publish run".to_string()],
                "cicd.toml not found at workspace root; run `cargo cicd publish run` to generate it.",
            ));
        }

        findings
    }

    fn name(&self) -> &'static str {
        "PipelineCheckAnalyzer"
    }
}
