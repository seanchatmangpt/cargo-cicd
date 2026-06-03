//! EvidenceAnalyzer — raises CICD-EVIDENCE-001 through CICD-EVIDENCE-004.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::evidence::freshness::FreshnessVerdict;
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Analyzes evidence state and events.jsonl content for known defects.
pub struct EvidenceAnalyzer;

impl CicdAnalyzer for EvidenceAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        if !snapshot.evidence_state.exists {
            findings.push(CicdFinding::new(
                CicdCode::EvidenceMissing,
                "target/cargo-cicd/evidence/",
                "target/cargo-cicd/evidence/",
                vec![
                    "cargo cicd test changed".to_string(),
                    "cargo cicd workspace doctor".to_string(),
                ],
                "No evidence file found. Run the manufacturing pipeline to emit process evidence.",
            ));
            return findings;
        }

        if snapshot.evidence_state.freshness == FreshnessVerdict::Stale {
            findings.push(CicdFinding::new(
                CicdCode::EvidenceStale,
                "target/cargo-cicd/evidence/",
                "target/cargo-cicd/evidence/",
                vec![
                    "cargo cicd test changed".to_string(),
                    "cargo cicd workspace doctor".to_string(),
                ],
                "Evidence file is stale relative to source files. Re-run the pipeline.",
            ));
        }

        // Scan events.jsonl if present
        let evidence_dir = snapshot
            .root
            .join("target")
            .join("cargo-cicd")
            .join("evidence");
        let jsonl_path = evidence_dir.join("events.jsonl");

        if jsonl_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&jsonl_path) {
                let jsonl_str = jsonl_path.to_string_lossy().into_owned();

                // CICD-EVIDENCE-003: hardcoded zero timestamp
                let hardcoded_ts_count = content
                    .lines()
                    .filter(|l| l.contains("T00:00:00.000Z"))
                    .count();
                if hardcoded_ts_count > 0 {
                    findings.push(CicdFinding::new(
                        CicdCode::EvidenceHardcodedTimestamp,
                        jsonl_str.clone(),
                        "source event emitter",
                        vec![
                            "cargo cicd test changed".to_string(),
                            "cargo cicd workspace doctor".to_string(),
                        ],
                        format!(
                            "{} event(s) contain hardcoded 'T00:00:00.000Z' timestamp. \
                             Timestamps must be derived from actual event times.",
                            hardcoded_ts_count
                        ),
                    ));
                }

                // CICD-EVIDENCE-004: null case_id
                let null_case_count = content
                    .lines()
                    .filter(|l| l.contains("\"case_id\":null") || l.contains("\"case_id\": null"))
                    .count();
                if null_case_count > 0 {
                    findings.push(CicdFinding::new(
                        CicdCode::EvidenceMissingCaseId,
                        jsonl_str,
                        "source event emitter",
                        vec![
                            "cargo cicd test changed".to_string(),
                            "cargo cicd workspace doctor".to_string(),
                        ],
                        format!(
                            "{} event(s) have null case_id. Every event must be traceable to an \
                             object lifecycle.",
                            null_case_count
                        ),
                    ));
                }
            }
        }

        findings
    }

    fn name(&self) -> &'static str {
        "EvidenceAnalyzer"
    }
}
