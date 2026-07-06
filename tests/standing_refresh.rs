//! Integration test: `cargo cicd standing refresh` against a temp workspace
//! with a minimal `[standing]` section, asserting
//! `target/praxis-standing/standing.json` is written and parses.

use assert_cmd::Command;
use cargo_cicd::cicd_toml::CicdToml;
use cargo_cicd_core::standing::StandingDocument;
use tempfile::TempDir;

#[test]
fn standing_refresh_writes_parseable_standing_json() {
    let dir = TempDir::new().expect("tempdir");

    // A minimal cicd.toml with a [standing] section pointing at a fixture
    // doctor command; every other ingestor is left unconfigured so it falls
    // back to its tolerant UNSEEN entry.
    let mut cfg = CicdToml::default();
    cfg.standing.doctor_command =
        Some("echo '{\"build\": true, \"frontier\": {\"pass_rate\": 1.0}}'".to_string());
    cfg.write(dir.path().join("cicd.toml"))
        .expect("write cicd.toml");

    let output = Command::cargo_bin("cargo-cicd")
        .expect("binary exists")
        .args(["standing", "refresh", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("command ran");

    assert!(
        output.status.success(),
        "standing refresh failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let standing_json_path = dir
        .path()
        .join("target")
        .join("praxis-standing")
        .join("standing.json");
    assert!(
        standing_json_path.exists(),
        "expected {} to exist",
        standing_json_path.display()
    );

    let content = std::fs::read_to_string(&standing_json_path).expect("read standing.json");
    let doc: StandingDocument =
        serde_json::from_str(&content).expect("standing.json must parse as StandingDocument");

    assert_eq!(doc.standing_version, "1");
    assert!(
        !doc.artifacts.is_empty(),
        "expected at least the doctor-report artifact"
    );
    let doctor = doc
        .artifacts
        .iter()
        .find(|a| a.id == "doctor-report")
        .expect("doctor-report artifact present");
    assert!(doctor
        .standing
        .contains(&cargo_cicd_core::standing::StandingStatus::Builds));

    // The other five focused sub-slices and the TTL/OCEL side effects should
    // also exist.
    for name in [
        "standing.ttl",
        "benchmark-summary.json",
        "receipt-summary.json",
        "client-surface-summary.json",
        "claim-index.json",
        "LSP_DIAGNOSTICS.json",
    ] {
        let p = dir.path().join("target").join("praxis-standing").join(name);
        assert!(p.exists(), "expected {} to exist", p.display());
    }
}

#[test]
fn standing_report_reads_back_refreshed_document() {
    let dir = TempDir::new().expect("tempdir");
    let cfg = CicdToml::default();
    cfg.write(dir.path().join("cicd.toml"))
        .expect("write cicd.toml");

    Command::cargo_bin("cargo-cicd")
        .expect("binary exists")
        .args(["standing", "refresh"])
        .current_dir(dir.path())
        .output()
        .expect("refresh ran");

    let report = Command::cargo_bin("cargo-cicd")
        .expect("binary exists")
        .args(["standing", "report", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("report ran");

    assert!(report.status.success(), "standing report failed");
    let stdout = String::from_utf8_lossy(&report.stdout).to_string();
    // clap_noun_verb's CLI runner appends a trailing `null` line (the JSON
    // serialization of the verb's `Ok(())` return value) after whatever the
    // verb itself printed — a framework-wide quirk, not specific to
    // `standing report` (e.g. `receipt verify --json` does the same). Parse
    // only the first line, which is the verb's own JSON output.
    let first_line = stdout.lines().next().unwrap_or_default();
    let doc: StandingDocument = serde_json::from_str(first_line)
        .expect("report --json must be a parseable StandingDocument");
    assert_eq!(doc.standing_version, "1");
}
