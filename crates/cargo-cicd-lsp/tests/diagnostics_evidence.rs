//! diagnostics_evidence — fixture-based tests for EvidenceAnalyzer.

use std::path::Path;

use cargo_cicd_core::diagnostics::code::CicdCode;
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;
use cargo_cicd_lsp::analyzers::evidence::EvidenceAnalyzer;
use cargo_cicd_lsp::analyzers::CicdAnalyzer;

fn fixture_root(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("workspaces")
        .join(name)
}

fn snapshot_for_fixture(name: &str) -> WorkspaceSnapshot {
    WorkspaceSnapshot::from_path(&fixture_root(name))
}

/// hardcoded-timestamp fixture has a T00:00:00.000Z timestamp → CICD-EVIDENCE-003.
#[test]
fn hardcoded_timestamp_raises_evidence_003() {
    let snapshot = snapshot_for_fixture("hardcoded-timestamp");
    let findings = EvidenceAnalyzer.analyze(&snapshot);

    let codes: Vec<CicdCode> = findings.iter().map(|f| f.code).collect();
    assert!(
        codes.contains(&CicdCode::EvidenceHardcodedTimestamp),
        "expected CICD-EVIDENCE-003 in findings, got: {:?}",
        codes
    );
}

/// missing-case-id fixture has null case_id → CICD-EVIDENCE-004.
#[test]
fn missing_case_id_raises_evidence_004() {
    let snapshot = snapshot_for_fixture("missing-case-id");
    let findings = EvidenceAnalyzer.analyze(&snapshot);

    let codes: Vec<CicdCode> = findings.iter().map(|f| f.code).collect();
    assert!(
        codes.contains(&CicdCode::EvidenceMissingCaseId),
        "expected CICD-EVIDENCE-004 in findings, got: {:?}",
        codes
    );
}

/// stale-evidence fixture (no XES, only JSONL) is detected as stale → CICD-EVIDENCE-002.
#[test]
fn stale_evidence_raises_evidence_002() {
    let snapshot = snapshot_for_fixture("stale-evidence");
    let findings = EvidenceAnalyzer.analyze(&snapshot);

    let codes: Vec<CicdCode> = findings.iter().map(|f| f.code).collect();
    assert!(
        codes.contains(&CicdCode::EvidenceStale),
        "expected CICD-EVIDENCE-002 in findings, got: {:?}",
        codes
    );
}
