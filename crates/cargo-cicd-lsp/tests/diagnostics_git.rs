//! diagnostics_git — fixture-based tests for GitPhaseAnalyzer.

use cargo_cicd_core::diagnostics::code::CicdCode;
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;
use cargo_cicd_lsp::analyzers::git_phase::GitPhaseAnalyzer;
use cargo_cicd_lsp::analyzers::CicdAnalyzer;

/// A dirty synthetic snapshot must produce CICD-GIT-001.
#[test]
fn dirty_tree_raises_git_001() {
    let snapshot = WorkspaceSnapshot::synthetic(/*dirty=*/ true);
    let findings = GitPhaseAnalyzer.analyze(&snapshot);

    let codes: Vec<CicdCode> = findings.iter().map(|f| f.code).collect();
    assert!(
        codes.contains(&CicdCode::GitDirtyTreeBlocksClose),
        "expected CICD-GIT-001 in findings, got: {:?}",
        codes
    );
}

/// A clean synthetic snapshot must not produce CICD-GIT-001.
#[test]
fn clean_tree_no_git_001() {
    let snapshot = WorkspaceSnapshot::synthetic(/*dirty=*/ false);
    let findings = GitPhaseAnalyzer.analyze(&snapshot);

    let codes: Vec<CicdCode> = findings.iter().map(|f| f.code).collect();
    assert!(
        !codes.contains(&CicdCode::GitDirtyTreeBlocksClose),
        "CICD-GIT-001 must not appear for a clean tree, got: {:?}",
        codes
    );
}
