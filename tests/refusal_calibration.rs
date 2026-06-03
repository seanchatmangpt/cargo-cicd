//! Refusal calibration tests — document actual wpm oracle behavior.
//!
//! These tests call `wpm audit <file>` directly and record the observed exit
//! codes.  They serve as a calibration ledger: if wpm behavior changes, at
//! least one of these tests will fail, making the change visible.
//!
//! All three tests are skip-safe when wpm is unavailable (they print
//! "SKIPPED" and return without failing).

use std::path::PathBuf;

/// Locate the wpm binary.  Search order:
/// 1. `$WPM_PATH` env var
/// 2. Known release path `/Users/sac/wasm4pm/target/release/wpm`
/// 3. `PATH` via `which wpm`
fn find_wpm() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("WPM_PATH") {
        let pb = PathBuf::from(&p);
        if pb.exists() {
            return Some(pb);
        }
    }
    let known = PathBuf::from("/Users/sac/wasm4pm/target/release/wpm");
    if known.exists() {
        return Some(known);
    }
    if let Ok(o) = std::process::Command::new("which").arg("wpm").output() {
        if o.status.success() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(PathBuf::from(s));
            }
        }
    }
    None
}

/// A minimal, conformant XES log.  wpm must exit 0 (it may still emit a
/// DECEPTIVE / WARN verdict in its output — that is a quality verdict, not a
/// parse failure).
#[test]
fn test_wpm_accepts_minimal_valid_xes() {
    let Some(wpm) = find_wpm() else {
        eprintln!("SKIPPED: wpm not available");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let xes = dir.path().join("valid.xes");
    std::fs::write(
        &xes,
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<log xes.version=\"1.0\" xmlns=\"http://www.xes-standard.org/\">\n",
            "  <trace><event>",
            "<string key=\"concept:name\" value=\"test:command\"/>",
            "<string key=\"time:timestamp\" value=\"2026-06-02T12:00:00.000Z\"/>",
            "</event></trace>\n",
            "</log>\n",
        ),
    )
    .unwrap();
    let out = std::process::Command::new(&wpm)
        .args(["audit", xes.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    eprintln!(
        "valid xes exit={} stdout={} stderr={}",
        out.status, stdout, stderr
    );
    // wpm must produce an exit code (not killed by signal).
    assert!(
        out.status.code().is_some(),
        "wpm must exit with a code, not a signal"
    );
    // Observed behavior: exit 0 — wpm parses the XES and emits a quality
    // verdict (possibly DECEPTIVE for empty-command traces, but NOT an error).
    assert_eq!(
        out.status.code().unwrap(),
        0,
        "wpm must exit 0 for a valid, parseable XES (got non-zero; \
         stdout={stdout} stderr={stderr})"
    );
}

/// Binary garbage that is not valid UTF-8 must be refused (exit non-zero).
/// Observed: wpm exits 1 with "stream did not contain valid UTF-8".
#[test]
fn test_wpm_refuses_binary_garbage() {
    let Some(wpm) = find_wpm() else {
        eprintln!("SKIPPED: wpm not available");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let garbage = dir.path().join("garbage.xes");
    std::fs::write(&garbage, b"\x00\x01\x02\xFF binary garbage not xml at all").unwrap();
    let out = std::process::Command::new(&wpm)
        .args(["audit", garbage.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    eprintln!(
        "garbage exit={} stdout={} stderr={}",
        out.status, stdout, stderr
    );
    // wpm must refuse binary garbage — exit non-zero.
    assert!(
        !out.status.success(),
        "wpm must refuse binary garbage — exit was 0 (unexpected ACCEPT); \
         stdout={stdout} stderr={stderr}"
    );
}

/// Empty file behavior — document and assert that wpm exits cleanly.
/// Observed: wpm exits 1 with a load-error (empty file cannot be parsed).
#[test]
fn test_wpm_behavior_on_empty_file() {
    let Some(wpm) = find_wpm() else {
        eprintln!("SKIPPED: wpm not available");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let empty = dir.path().join("empty.xes");
    std::fs::write(&empty, "").unwrap();
    let out = std::process::Command::new(&wpm)
        .args(["audit", empty.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    eprintln!(
        "empty exit={} stdout={} stderr={}",
        out.status, stdout, stderr
    );
    // wpm must produce an exit code (not killed by signal).
    assert!(
        out.status.code().is_some(),
        "wpm must exit cleanly on empty file (got signal); \
         stdout={stdout} stderr={stderr}"
    );
    // Observed behavior: empty file → exit 1 (parse/load error).
    assert!(
        !out.status.success(),
        "wpm must refuse an empty XES file (exit was 0); \
         stdout={stdout} stderr={stderr}"
    );
}
