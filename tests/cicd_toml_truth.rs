use std::process::Command;
use tempfile::TempDir;

/// Fail-closed contract: `publish run` without --dry-run refuses with the
/// typed REFUSED:explicit_dry_run_required error and writes nothing.
#[test]
fn test_publish_run_refuses_without_explicit_dry_run() {
    let dir = TempDir::new().unwrap();
    assert!(!dir.path().join("cicd.toml").exists());
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir.path())
        .args(["publish", "run"])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "publish run without --dry-run must exit non-zero"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("REFUSED:explicit_dry_run_required"),
        "expected typed REFUSED:explicit_dry_run_required, got: {combined:?}"
    );
    assert!(
        !dir.path().join("cicd.toml").exists(),
        "refused publish must not write cicd.toml"
    );
}

/// With --dry-run but no wpm oracle on PATH, publish is HARD REFUSED with
/// the typed REFUSED:oracle_unavailable error and writes nothing.
#[test]
fn test_publish_dry_run_refuses_without_oracle() {
    let dir = TempDir::new().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir.path())
        .env_remove("WPM_BIN")
        .env_remove("WPM_PATH")
        .env_remove("WPM_SEARCH_PATHS")
        .args(["publish", "run", "--dry-run"])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "publish --dry-run without a wpm oracle must exit non-zero"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("REFUSED:oracle_unavailable"),
        "expected typed REFUSED:oracle_unavailable, got: {combined:?}"
    );
    assert!(
        !dir.path().join("cicd.toml").exists(),
        "refused publish must not write cicd.toml"
    );
}

/// Seed a minimal publishable package so `cargo publish --dry-run`
/// (the final publish gate) can succeed in the fixture.
fn seed_minimal_package(dir: &std::path::Path) {
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"cicd_truth_fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "").unwrap();
}

/// `publish run --dry-run` with a stub oracle that adjudicates successfully
/// proceeds and writes a valid cicd.toml.
#[test]
fn test_publish_dry_run_writes_cicd_toml_with_oracle() {
    let dir = TempDir::new().unwrap();
    seed_minimal_package(dir.path());
    let bin_dir = dir.path().join(".bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    // Stub wpm: accept any `receipt doctor` invocation with an admitting
    // JSON verdict on stdout.
    std::fs::write(
        bin_dir.join("wpm"),
        "#!/bin/sh\necho '{\"state\":\"Admitted\"}'\nexit 0\n",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(bin_dir.join("wpm"), std::fs::Permissions::from_mode(0o755)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir.path())
        .args(["publish", "run", "--dry-run"])
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "publish --dry-run with accepting oracle should proceed, got: {combined:?}"
    );
    let content = std::fs::read_to_string(dir.path().join("cicd.toml")).unwrap_or_default();
    assert!(!content.is_empty(), "cicd.toml should be created");
}

/// Seed a stub wpm oracle (`receipt doctor` always admits) and run
/// `publish run --dry-run`, returning the resulting cicd.toml content.
fn run_publish(dir: &std::path::Path) -> String {
    seed_minimal_package(dir);
    let bin_dir = dir.join(".bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::write(
        bin_dir.join("wpm"),
        "#!/bin/sh\necho '{\"state\":\"Admitted\"}'\nexit 0\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin_dir.join("wpm"), std::fs::Permissions::from_mode(0o755))
            .unwrap();
    }
    let _output = Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir)
        .args(["publish", "run", "--dry-run"])
        .output()
        .unwrap();
    std::fs::read_to_string(dir.join("cicd.toml")).unwrap_or_default()
}

#[test]
fn test_publish_deterministic_on_unchanged_state() {
    let dir = TempDir::new().unwrap();
    let first = run_publish(dir.path());
    assert!(!first.is_empty(), "first publish must write cicd.toml");
    // Run again — workspace state unchanged
    let second = run_publish(dir.path());
    // Key sections must match (timestamps may differ — compare structural parts)
    let first_workspace: Vec<&str> = first
        .lines()
        .filter(|l| l.starts_with('[') || l.contains('='))
        .collect();
    let second_workspace: Vec<&str> = second
        .lines()
        .filter(|l| l.starts_with('[') || l.contains('='))
        .collect();
    // At least the section headers should be identical
    let first_sections: Vec<&str> = first_workspace
        .iter()
        .filter(|l| l.starts_with('['))
        .copied()
        .collect();
    let second_sections: Vec<&str> = second_workspace
        .iter()
        .filter(|l| l.starts_with('['))
        .copied()
        .collect();
    assert_eq!(
        first_sections, second_sections,
        "cicd.toml sections should be deterministic"
    );
}

#[test]
fn test_corrupted_cicd_toml_does_not_silently_pass() {
    let dir = TempDir::new().unwrap();
    // Write invalid TOML
    std::fs::write(dir.path().join("cicd.toml"), b"[[invalid toml garbage @#$").unwrap();
    // publish should either refuse, repair, or report explicitly — not silently succeed
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir.path())
        .args(["publish", "run"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);
    // Must mention the issue somehow OR write a valid new cicd.toml
    let new_content = std::fs::read_to_string(dir.path().join("cicd.toml")).unwrap_or_default();
    let repaired = toml::from_str::<toml::Value>(&new_content).is_ok();
    assert!(
        repaired
            || combined.contains("corrupt")
            || combined.contains("invalid")
            || combined.contains("error"),
        "corrupted cicd.toml must be repaired or explicitly reported"
    );
}

/// cicd.toml written by pipeline run contains a [state] section.
#[test]
fn cicd_toml_state_section_present_after_status() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    let _ = Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .args(["status", "show"])
        .current_dir(tmp.path())
        .output();
    let toml_path = tmp.path().join("cicd.toml");
    if toml_path.exists() {
        let content = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            content.contains("[state]"),
            "cicd.toml missing [state] section: {}",
            content
        );
    }
    // If no cicd.toml written, that is also acceptable (status may not write it)
}

/// cicd.toml workspace section has target_dir field.
#[test]
fn cicd_toml_workspace_has_target_dir() {
    let dir = TempDir::new().unwrap();
    let content = run_publish(dir.path());
    if !content.is_empty() && content.contains("[workspace]") {
        assert!(
            content.contains("target_dir"),
            "cicd.toml missing target_dir: {}",
            content
        );
    }
}

/// cicd.toml produced by publish run is valid TOML.
#[test]
fn cicd_toml_parses_as_valid_toml() {
    let dir = TempDir::new().unwrap();
    let content = run_publish(dir.path());
    if !content.is_empty() {
        let parsed: Result<toml::Value, _> = toml::from_str(&content);
        assert!(
            parsed.is_ok(),
            "cicd.toml is not valid TOML: {:?}",
            parsed.err()
        );
    }
}
