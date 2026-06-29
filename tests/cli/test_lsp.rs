use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn lsp_doctor_runs() {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "doctor"])
        .assert()
        .code(predicate::in_iter(vec![0i32, 1]));
}

// Strict success path requires the (feature-gated) `lsp` noun to be registered.
#[cfg(feature = "lsp")]
#[test]
fn lsp_explain_known_code() {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "explain", "CICD-GIT-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dirty_tree_blocks_close"));
}

#[test]
fn lsp_explain_unknown_code_fails() {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "explain", "CICD-NOTEXIST-999"])
        .assert()
        .failure();
}

#[cfg(feature = "anti-llm-cheat")]
#[test]
fn lsp_check_runs_and_emits_verdict() {
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "check"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    let has_verdict = text.contains("PASS")
        || text.contains("WARN")
        || text.contains("FAIL")
        || text.contains("no admissibility violations");
    assert!(has_verdict, "lsp check missing verdict in output: {}", text);
    for term in &[
        "ALIVE",
        "Nehemiah",
        "CONSTRUCT8",
        "Instinct8",
        "Cargo Court",
        "AGI",
    ] {
        assert!(
            !text.contains(term),
            "forbidden term '{}' in lsp check output",
            term
        );
    }
}

#[cfg(not(feature = "anti-llm-cheat"))]
#[test]
fn lsp_check_unavailable_without_feature() {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "check"])
        .assert()
        .failure();
}
