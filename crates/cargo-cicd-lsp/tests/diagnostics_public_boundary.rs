//! diagnostics_public_boundary — fixture-based tests for PublicBoundaryAnalyzer.

use std::path::Path;

use cargo_cicd_core::diagnostics::code::CicdCode;
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;
use cargo_cicd_lsp::analyzers::public_boundary::PublicBoundaryAnalyzer;
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

/// private-term-leak README.md contains "ALIVE" → CICD-PUBLIC-001.
#[test]
fn private_term_leak_raises_public_001() {
    let snapshot = snapshot_for_fixture("private-term-leak");
    let findings = PublicBoundaryAnalyzer.analyze(&snapshot);

    let codes: Vec<CicdCode> = findings.iter().map(|f| f.code).collect();
    assert!(
        codes.contains(&CicdCode::PublicPrivateTermLeak),
        "expected CICD-PUBLIC-001 in findings, got: {:?}",
        codes
    );
}

/// A workspace without forbidden terms must not produce CICD-PUBLIC-001.
#[test]
fn clean_workspace_no_public_001() {
    let snapshot = WorkspaceSnapshot::synthetic(false);
    let findings = PublicBoundaryAnalyzer.analyze(&snapshot);

    let codes: Vec<CicdCode> = findings.iter().map(|f| f.code).collect();
    assert!(
        !codes.contains(&CicdCode::PublicPrivateTermLeak),
        "CICD-PUBLIC-001 must not appear for synthetic (no README) snapshot, got: {:?}",
        codes
    );
}
