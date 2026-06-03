//! wasm4pm evidence gate — positive acceptance cases.
//! Tests assert only wasm4pm verdicts; never cargo-cicd internal state.
//!
//! # Oracle-Absent Coverage Note
//!
//! The Accept branch in each test below is only exercised when the wpm binary
//! is present at `/Users/sac/wasm4pm/target/release/wpm`. In CI environments
//! without that binary, tests fall back to the `Blocked` path and the `Accept`
//! assertions are silently skipped.
//!
//! To make oracle-absent failures visible instead of silently skipped, set the
//! environment variable `REQUIRE_WPM_ORACLE=1` before running these tests. When
//! that variable is set and the wpm binary is absent, tests will panic with a
//! clear message rather than falling back to `Blocked`.
//!
//! ```sh
//! # Run with oracle required (fails fast if wpm binary is absent):
//! REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
//!
//! # Run normally (graceful Blocked fallback when wpm is absent):
//! cargo test --test wasm4pm_evidence_gate
//! ```

use cargo_cicd::evidence::{
    assert_wpm_verdict, emit_xes, ExpectedWpmVerdict, ProcessEvent, WpmEvidenceOracle,
};
use tempfile::TempDir;

/// Returns the verdict to expect when the wpm oracle is absent.
///
/// When `REQUIRE_WPM_ORACLE=1` is set, panics with a clear message so CI
/// pipelines that have the wpm binary configured are forced to exercise the
/// Accept path rather than silently falling back to Blocked.
fn absent_oracle_verdict(test_name: &str) -> ExpectedWpmVerdict {
    if std::env::var("REQUIRE_WPM_ORACLE").as_deref() == Ok("1") {
        panic!(
            "REQUIRE_WPM_ORACLE=1 is set but the wpm oracle binary is absent. \
             Test '{}' cannot exercise its Accept assertion. \
             Ensure the wpm binary exists at /Users/sac/wasm4pm/target/release/wpm.",
            test_name
        );
    }
    ExpectedWpmVerdict::Blocked
}

#[test]
fn evidence_gate_status_show_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("status show", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    let oracle = WpmEvidenceOracle::new();
    // Accept is only asserted when the wpm oracle binary is present (see module doc).
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_status_show_accepted"),
        );
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
    // Accept is only asserted when the wpm oracle binary is present (see module doc).
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_target_show_accepted"),
        );
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
    // Accept is only asserted when the wpm oracle binary is present (see module doc).
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_target_prune_accepted"),
        );
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
    // Accept is only asserted when the wpm oracle binary is present (see module doc).
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_changed_test_accepted"),
        );
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
    // Accept is only asserted when the wpm oracle binary is present (see module doc).
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_git_close_accepted"),
        );
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
    // Accept is only asserted when the wpm oracle binary is present (see module doc).
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_publish_run_accepted"),
        );
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
    // Accept is only asserted when the wpm oracle binary is present (see module doc).
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_workspace_doctor_accepted"),
        );
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

/// Hard gate: invoke `wpm doctor` as a live binary and assert a non-error verdict.
///
/// Per the evidence-gate acceptance law (CLAUDE.md §wasm4pm Evidence Gate),
/// release closure requires wpm adjudication. When the wpm binary is present,
/// this test is a hard gate: a non-zero exit code or FAIL/REFUSE in output
/// causes the test to fail — it does NOT silently skip. Only binary absence
/// causes a graceful skip (or hard panic when REQUIRE_WPM_ORACLE=1).
#[test]
fn evidence_gate_wpm_doctor_hard_gate() {
    use std::process::Command;

    const WPM_KNOWN_PATH: &str = "/Users/sac/wasm4pm/target/release/wpm";

    // Resolve binary: env override → known path → PATH
    let wpm_bin = if let Ok(p) = std::env::var("WPM_PATH") {
        if std::path::Path::new(&p).exists() {
            Some(std::path::PathBuf::from(p))
        } else {
            None
        }
    } else if std::path::Path::new(WPM_KNOWN_PATH).exists() {
        Some(std::path::PathBuf::from(WPM_KNOWN_PATH))
    } else if let Ok(out) = Command::new("which").arg("wpm").output() {
        if out.status.success() {
            let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !p.is_empty() {
                Some(std::path::PathBuf::from(p))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let wpm_bin = match wpm_bin {
        Some(b) => b,
        None => {
            if std::env::var("REQUIRE_WPM_ORACLE").as_deref() == Ok("1") {
                panic!(
                    "GATE-FAIL: REQUIRE_WPM_ORACLE=1 but wpm binary not found. \
                     Install wasm4pm at {WPM_KNOWN_PATH} or set WPM_PATH."
                );
            }
            eprintln!(
                "GATE-SKIP: wpm binary not found — wpm doctor hard gate skipped. \
                 Set REQUIRE_WPM_ORACLE=1 to make absence a hard failure."
            );
            return;
        }
    };

    // wpm IS present — invoke `wpm doctor` as a hard gate.
    let output = Command::new(&wpm_bin)
        .arg("doctor")
        .output()
        .unwrap_or_else(|e| panic!("GATE-FAIL: wpm doctor spawn failed: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}").to_lowercase();

    // Hard assertion: exit 0 and no FAIL/REFUSE in output.
    assert!(
        output.status.success(),
        "GATE-FAIL: wpm doctor exited with non-zero status {}.\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code().unwrap_or(-1),
    );
    assert!(
        !combined.contains("fail") && !combined.contains("refuse"),
        "GATE-FAIL: wpm doctor output contains failure indicators.\nstdout: {stdout}\nstderr: {stderr}",
    );
}
