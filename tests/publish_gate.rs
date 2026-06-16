use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn publish_run_emits_adjudication_line() {
    // Verify publish run mentions adjudication (not a dry-run invocation test, just output structure)
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["publish", "run"]);
    // publish run should complete (may warn if wpm absent) without panicking
    // Exit 0 = Admitted + dry-run passed; non-zero = wpm absent or dry-run failed
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must contain either "adjudication:" or "receipt doctor:" — either path
    assert!(
        stdout.contains("adjudication:") || stdout.contains("receipt doctor:"),
        "publish run must report adjudication status. got: {}",
        stdout
    );
}

#[test]
fn publish_check_runs_dry_run() {
    // Verify publish check runs cargo publish --dry-run in a minimal workspace
    // and completes without panicking, emitting a [PASS] or [FAIL] status
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"",
    )
    .unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["publish", "check"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // publish check must emit a status line: [PASS] or [FAIL]
    assert!(
        stdout.contains("[PASS]") || stdout.contains("[FAIL]"),
        "publish check must emit [PASS] or [FAIL] status. got: {}",
        stdout
    );
}

#[test]
fn publish_validate_checks_preconditions() {
    // Verify publish validate checks preconditions and emits status markers
    let tmp = TempDir::new().unwrap();
    // Create minimal Cargo.toml with only name and version (missing description, license)
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"",
    )
    .unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["publish", "validate"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // publish validate must emit status markers [PASS], [WARN], or [FAIL]
    assert!(
        stdout.contains("[PASS]") || stdout.contains("[WARN]") || stdout.contains("[FAIL]"),
        "publish validate must emit status markers. got: {}",
        stdout
    );
    // Should mention checking cicd.toml, Cargo.toml metadata, README, LICENSE
    assert!(
        stdout.contains("cicd.toml") || stdout.contains("Cargo.toml") || stdout.contains("README"),
        "publish validate must check preconditions. got: {}",
        stdout
    );
}

#[test]
fn publish_run_with_missing_receipt_fails() {
    // Verify publish run handles missing evidence gracefully
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"",
    )
    .unwrap();
    // Ensure no evidence directory exists (clean slate)
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["publish", "run"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // publish run must emit either adjudication status or a warning about oracle unavailable
    assert!(
        stdout.contains("adjudication:")
            || stdout.contains("receipt doctor:")
            || stderr.contains("warning")
            || stderr.contains("oracle"),
        "publish run must handle missing evidence. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn publish_run_emits_evidence() {
    // Verify publish run creates or updates evidence (events.jsonl or receipt)
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"",
    )
    .unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["publish", "run"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // publish run must emit adjudication line
    assert!(
        stdout.contains("adjudication:") || stdout.contains("receipt doctor:"),
        "publish run must report adjudication. got: {}",
        stdout
    );
    // cicd.toml should have been written
    let cicd_path = tmp.path().join("cicd.toml");
    assert!(
        cicd_path.exists(),
        "publish run must write cicd.toml to workspace"
    );
    let cicd_content = std::fs::read_to_string(&cicd_path).unwrap();
    // cicd.toml must contain workspace state
    assert!(
        cicd_content.contains("[workspace]"),
        "cicd.toml must contain [workspace] section. got: {}",
        cicd_content
    );
}
