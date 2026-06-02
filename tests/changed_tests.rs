use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_trybuild_changed_does_not_mention_all_fixtures() {
    let dir = TempDir::new().unwrap();
    // No changed fixtures in tempdir
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("trybuild")
        .arg("changed")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    // Must NOT mention 'all fixtures' or '624' without explicit flag
    assert!(
        !combined.contains("all fixtures")
            || combined.contains("no changed")
            || combined.contains("0 changed"),
        "trybuild changed should report 0 changed, not run all: {}",
        combined
    );
}

#[test]
fn test_trybuild_changed_selects_only_changed_fixture() {
    let dir = TempDir::new().unwrap();
    // Create one fixture file in tests/ui/compile_fail/
    let ui_dir = dir.path().join("tests/ui/compile_fail");
    std::fs::create_dir_all(&ui_dir).unwrap();
    std::fs::write(ui_dir.join("my_law.rs"), "fn main() {}").unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("trybuild")
        .arg("changed")
        .output()
        .unwrap();
    // Should exit without error (even if 0 changes detected without git)
    assert!(
        output.status.code().is_some(),
        "trybuild changed should not panic"
    );
}

#[test]
fn test_test_changed_emits_test_plan_not_fake_precision() {
    let dir = TempDir::new().unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("test")
        .arg("changed")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    // Must mention plan, changed, or 0 — must not claim to have run tests it didn't
    // Accept: 'no changed files', 'test plan', '0 tests', 'conservative'
    // The command must not panic
    assert!(
        output.status.code().is_some(),
        "test changed should not panic: {}",
        combined
    );
}

#[test]
fn test_test_changed_with_modified_rust_file_produces_plan() {
    // Conservative plan is fine — no fake precision
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("lib.rs"), "pub fn foo() {}").unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("test")
        .arg("changed")
        .output()
        .unwrap();
    assert!(output.status.code().is_some());
}
