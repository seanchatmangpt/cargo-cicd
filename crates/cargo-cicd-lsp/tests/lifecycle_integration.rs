//! lifecycle_integration — end-to-end wire test for raise/clear_by_code through DiagnosticStore.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_lsp::lifecycle::{clear_by_code, raise};
use cargo_cicd_lsp::state::DiagnosticStore;

const TEST_URI: &str = "file:///test.rs";

/// raise() inserts a finding; clear_by_code() removes it by code string.
#[test]
fn lifecycle_raise_and_clear_work_with_store() {
    let mut store = DiagnosticStore::new();

    let finding = CicdFinding::minimal(CicdCode::GitDirtyTreeBlocksClose, "test");
    raise(&mut store, TEST_URI.to_string(), finding);

    assert_eq!(
        store.get_all(TEST_URI).len(),
        1,
        "raise must insert one finding"
    );

    clear_by_code(
        &mut store,
        TEST_URI,
        CicdCode::GitDirtyTreeBlocksClose.as_str(),
    );

    assert_eq!(
        store.get_all(TEST_URI).len(),
        0,
        "clear_by_code must remove the finding"
    );
}

/// raise() followed by clear_by_code() on a different code leaves the original intact.
#[test]
fn clear_by_code_is_code_scoped() {
    let mut store = DiagnosticStore::new();

    raise(
        &mut store,
        TEST_URI.to_string(),
        CicdFinding::minimal(CicdCode::GitDirtyTreeBlocksClose, "git finding"),
    );
    raise(
        &mut store,
        TEST_URI.to_string(),
        CicdFinding::minimal(CicdCode::EvidenceMissing, "evidence finding"),
    );

    assert_eq!(
        store.get_all(TEST_URI).len(),
        2,
        "both findings must be present"
    );

    // Clear only CICD-GIT-001 — EvidenceMissing must survive.
    clear_by_code(
        &mut store,
        TEST_URI,
        CicdCode::GitDirtyTreeBlocksClose.as_str(),
    );

    let remaining: Vec<CicdCode> = store.get_all(TEST_URI).iter().map(|f| f.code).collect();
    assert_eq!(
        remaining.len(),
        1,
        "one finding must remain after targeted clear"
    );
    assert!(
        remaining.contains(&CicdCode::EvidenceMissing),
        "EvidenceMissing must survive clear_by_code(GIT-001), got: {:?}",
        remaining
    );
}
