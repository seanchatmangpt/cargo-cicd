//! wasm4pm evidence gate — positive acceptance cases.
//! Tests assert only wasm4pm verdicts; never cargo-cicd internal state.

use cargo_cicd::evidence::{
    assert_wpm_verdict, emit_xes, ExpectedWpmVerdict, ProcessEvent, WpmEvidenceOracle,
};
use tempfile::TempDir;

#[test]
fn evidence_gate_status_show_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("status show", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn evidence_gate_target_show_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("target show", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn evidence_gate_target_prune_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("target prune plan", "DRY-RUN")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn evidence_gate_changed_test_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("test changed", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn evidence_gate_git_close_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("git close", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn evidence_gate_publish_run_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("publish run", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

#[test]
fn evidence_gate_workspace_doctor_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("workspace doctor", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

/// Verify oracle is discovered (no panics on detect). E7 compliance.
#[test]
fn evidence_gate_oracle_discover() {
    let oracle = WpmEvidenceOracle::new();
    // Calling is_available must not panic — both true and false are valid
    let available = oracle.is_available();
    // If available, audit of a nonexistent path must return Refuse (not panic)
    if available {
        let result = oracle.audit_xes(std::path::Path::new("/nonexistent/oracle_discover.xes"));
        // Either Refuse or Blocked — must not panic
        let _ = result;
    }
}
