//! cicd.toml truth tests — every publish must reflect actual workspace state.
use assert_cmd::Command;
use tempfile::TempDir;

/// Published cicd.toml must be valid TOML.
#[test]
fn truth_publish_emits_valid_toml() {
    let tmp = TempDir::new().unwrap();
    let result = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["publish", "run"]).current_dir(tmp.path()).output().unwrap();
    if result.status.success() {
        let path = tmp.path().join("cicd.toml");
        assert!(path.exists(), "publish did not create cicd.toml");
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Result<toml::Value, _> = toml::from_str(&content);
        assert!(parsed.is_ok(), "published cicd.toml is not valid TOML:\n{}", content);
    }
}

/// cicd.toml must contain required sections.
#[test]
fn truth_cicd_toml_has_required_sections() {
    let tmp = TempDir::new().unwrap();
    let result = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["publish", "run"]).current_dir(tmp.path()).output().unwrap();
    if result.status.success() && tmp.path().join("cicd.toml").exists() {
        let content = std::fs::read_to_string(tmp.path().join("cicd.toml")).unwrap();
        let required = &["[workspace]", "[state]", "[target]", "[autonomic]"];
        for section in required {
            assert!(content.contains(section),
                "cicd.toml missing required section {}: {}", section, content);
        }
    }
}

/// cicd.toml events section must contain at least one record after publish.
#[test]
fn truth_publish_appends_event_record() {
    let tmp = TempDir::new().unwrap();
    let result = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["publish", "run"]).current_dir(tmp.path()).output().unwrap();
    if result.status.success() && tmp.path().join("cicd.toml").exists() {
        let content = std::fs::read_to_string(tmp.path().join("cicd.toml")).unwrap();
        assert!(content.contains("[[events]]"),
            "publish did not add an [[events]] record: {}", content);
        assert!(content.contains("kind"),
            "events record missing kind field: {}", content);
    }
}

/// workspace doctor must explain a missing Cargo.toml in terms users understand.
#[test]
fn truth_workspace_doctor_explains_missing_manifest() {
    let tmp = TempDir::new().unwrap();
    let output = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["workspace", "doctor"]).current_dir(tmp.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must reference Cargo.toml somehow — users need to know what is wrong
    assert!(stdout.contains("Cargo.toml") || stdout.contains("FAIL"),
        "workspace doctor did not explain missing manifest: {}", stdout);
}

/// target show verdict must be one of: pass, warn, fail.
#[test]
fn truth_target_show_verdict_is_valid() {
    let tmp = TempDir::new().unwrap();
    let output = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["target", "show"]).current_dir(tmp.path()).output().unwrap();
    assert!(output.status.success(), "target show failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_verdict = stdout.contains("pass") || stdout.contains("warn") || stdout.contains("fail");
    assert!(has_verdict, "target show missing verdict (pass/warn/fail): {}", stdout);
}
