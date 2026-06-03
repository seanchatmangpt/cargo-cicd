//! CloseReadinessAnalyzer — raises CICD-CLOSE-001 when blocking findings are present.
//!
//! Runs last. Uses a simple heuristic over the workspace snapshot:
//! dirty tree OR missing evidence OR missing publish receipt → CICD-CLOSE-001.

use cargo_cicd_core::diagnostics::route::RepairRoute;
use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::evidence::freshness::FreshnessVerdict;
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

pub struct CloseReadinessAnalyzer;

impl CicdAnalyzer for CloseReadinessAnalyzer {
    fn name(&self) -> &'static str {
        "close_readiness"
    }

    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        // Check conditions that prevent responsible phase close.
        let dirty = snapshot.git_status.dirty;
        let no_evidence = snapshot.evidence_state.freshness != FreshnessVerdict::Fresh;
        let no_receipt = !snapshot.has_receipts_dir;

        if dirty || no_evidence || no_receipt {
            vec![CicdFinding::new(
                CicdCode::FalseCloseRisk,
                "workspace snapshot",
                "cli",
                vec!["cargo cicd workspace doctor".to_string()],
                format!(
                    "Phase close risk: workspace state not fully reconciled. \
                         dirty={dirty}, no_evidence={no_evidence}, no_receipt={no_receipt}"
                ),
            )
            .with_route(RepairRoute {
                command: "cargo cicd workspace doctor".into(),
                explanation: "Review all workspace readiness conditions".into(),
            })]
        } else {
            vec![]
        }
    }
}
