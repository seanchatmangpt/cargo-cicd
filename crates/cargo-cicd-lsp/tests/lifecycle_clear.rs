//! lifecycle_clear — residual-preserved law for DiagnosticStore.

use cargo_cicd_core::diagnostics::code::CicdCode;
use cargo_cicd_core::diagnostics::finding::CicdFinding;
use cargo_cicd_lsp::state::diagnostic_store::DiagnosticStore;

const TEST_URI: &str = "file:///workspace/src/lib.rs";

fn make_finding(code: CicdCode) -> CicdFinding {
    CicdFinding::minimal(code, code.title())
}

/// After inserting CICD-GIT-001 and CICD-EVIDENCE-002 for the same URI,
/// removing CICD-GIT-001 must leave CICD-EVIDENCE-002 intact.
#[test]
fn remove_code_preserves_residual() {
    let mut store = DiagnosticStore::new();

    store.insert(
        TEST_URI.to_string(),
        make_finding(CicdCode::GitDirtyTreeBlocksClose),
    );
    store.insert(TEST_URI.to_string(), make_finding(CicdCode::EvidenceStale));

    // Sanity: both present before removal
    let before: Vec<CicdCode> = store.get_all(TEST_URI).iter().map(|f| f.code).collect();
    assert!(
        before.contains(&CicdCode::GitDirtyTreeBlocksClose),
        "GIT-001 should be present"
    );
    assert!(
        before.contains(&CicdCode::EvidenceStale),
        "EVIDENCE-002 should be present"
    );

    // Remove only CICD-GIT-001
    store.remove_code(TEST_URI, CicdCode::GitDirtyTreeBlocksClose.as_str());

    let after: Vec<CicdCode> = store.get_all(TEST_URI).iter().map(|f| f.code).collect();

    assert!(
        !after.contains(&CicdCode::GitDirtyTreeBlocksClose),
        "CICD-GIT-001 must be cleared, got: {:?}",
        after
    );
    assert!(
        after.contains(&CicdCode::EvidenceStale),
        "CICD-EVIDENCE-002 must be preserved after removing CICD-GIT-001, got: {:?}",
        after
    );
}

/// clear_uri removes all findings for the URI.
#[test]
fn clear_uri_removes_all() {
    let mut store = DiagnosticStore::new();

    store.insert(
        TEST_URI.to_string(),
        make_finding(CicdCode::GitDirtyTreeBlocksClose),
    );
    store.insert(TEST_URI.to_string(), make_finding(CicdCode::EvidenceStale));

    store.clear_uri(TEST_URI);

    let after = store.get_all(TEST_URI);
    assert!(
        after.is_empty(),
        "clear_uri must remove all findings, got: {:?}",
        after.len()
    );
}
