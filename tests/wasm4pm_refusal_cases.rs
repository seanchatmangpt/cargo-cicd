//! wasm4pm evidence gate — dedicated refusal ledger.
//! E5: Every positive path must have a mutated negative that wasm4pm refuses.
//! Invariant tests E1-E3 prove structural properties of the evidence API.

use cargo_cicd::evidence::{assert_wpm_verdict, ExpectedWpmVerdict, WpmEvidenceOracle};
use tempfile::TempDir;

#[test]
fn refusal_corrupted_xml_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("corrupted.ocel.json");
    std::fs::write(&xes_path, "THIS IS NOT XML").unwrap();
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn refusal_empty_xes_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("empty.ocel.json");
    std::fs::write(&xes_path, b"").unwrap();
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn refusal_missing_file_returns_refuse() {
    let oracle = WpmEvidenceOracle::new();
    // Missing file: audit_ocel will get an invocation error → Refuse (or Blocked if unavailable)
    let result = oracle.audit_ocel(std::path::Path::new("/nonexistent/path/missing.ocel.json"));
    // If oracle is unavailable, result is Blocked — that is acceptable
    // If oracle is available, result should be Refuse (file not found triggers error path)
    match result {
        ExpectedWpmVerdict::Blocked => {} // oracle not available — acceptable
        ExpectedWpmVerdict::Refuse => {}  // oracle refused missing file — correct
        ExpectedWpmVerdict::Accept => {
            panic!("oracle must not accept a missing XES file")
        }
    }
}
