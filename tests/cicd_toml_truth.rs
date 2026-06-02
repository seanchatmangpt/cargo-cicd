use tempfile::TempDir;
use std::process::Command;

fn run_publish(dir: &std::path::Path) -> String {
    Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir).args(["publish", "run"]).output().unwrap();
    std::fs::read_to_string(dir.join("cicd.toml")).unwrap_or_default()
}

#[test]
fn test_publish_creates_cicd_toml_when_absent() {
    let dir = TempDir::new().unwrap();
    assert!(!dir.path().join("cicd.toml").exists());
    let content = run_publish(dir.path());
    assert!(!content.is_empty(), "cicd.toml should be created");
}

#[test]
fn test_publish_deterministic_on_unchanged_state() {
    let dir = TempDir::new().unwrap();
    let first = run_publish(dir.path());
    // Run again — workspace state unchanged
    let second = run_publish(dir.path());
    // Key sections must match (timestamps may differ — compare structural parts)
    let first_workspace: Vec<&str> = first.lines().filter(|l| l.starts_with('[') || l.contains('=')).collect();
    let second_workspace: Vec<&str> = second.lines().filter(|l| l.starts_with('[') || l.contains('=')).collect();
    // At least the section headers should be identical
    let first_sections: Vec<&str> = first_workspace.iter().filter(|l| l.starts_with('[')).copied().collect();
    let second_sections: Vec<&str> = second_workspace.iter().filter(|l| l.starts_with('[')).copied().collect();
    assert_eq!(first_sections, second_sections, "cicd.toml sections should be deterministic");
}

#[test]
fn test_corrupted_cicd_toml_does_not_silently_pass() {
    let dir = TempDir::new().unwrap();
    // Write invalid TOML
    std::fs::write(dir.path().join("cicd.toml"), b"[[invalid toml garbage @#$").unwrap();
    // publish should either refuse, repair, or report explicitly — not silently succeed
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir.path()).args(["publish", "run"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);
    // Must mention the issue somehow OR write a valid new cicd.toml
    let new_content = std::fs::read_to_string(dir.path().join("cicd.toml")).unwrap_or_default();
    let repaired = toml::from_str::<toml::Value>(&new_content).is_ok();
    assert!(
        repaired || combined.contains("corrupt") || combined.contains("invalid") || combined.contains("error"),
        "corrupted cicd.toml must be repaired or explicitly reported"
    );
}
