#![cfg(feature = "anti-llm-cheat")]

use assert_cmd::Command;
use predicates::prelude::predicate;
use std::fs;
use tempfile::TempDir;

fn run_lsp_check_in(dir: &std::path::Path) -> std::process::Output {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir)
        .args(["lsp", "check"])
        .output()
        .unwrap()
}

#[test]
fn lsp_check_on_empty_dir_exits_cleanly() {
    let dir = TempDir::new().unwrap();
    // Write minimal Cargo.toml so workspace detection works
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let out = run_lsp_check_in(dir.path());
    let code = out.status.code().unwrap_or(-1);
    assert!(code == 0 || code == 1, "unexpected exit code {}", code);
}

#[test]
fn lsp_check_output_contains_verdict_word() {
    let out = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "check"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string()
        + &String::from_utf8_lossy(&out.stderr);
    let has_verdict = text.contains("PASS")
        || text.contains("WARN")
        || text.contains("FAIL")
        || text.contains("no admissibility violations");
    assert!(has_verdict, "no verdict word in output: {}", text);
}

#[test]
fn lsp_check_help_mentions_admissibility() {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("admissib").or(predicate::str::contains("scan")));
}
