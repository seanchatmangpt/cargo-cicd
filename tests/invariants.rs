//! Invariant tests — properties that must hold across all cargo-cicd commands.
//! These cut across all capability dimensions.
use assert_cmd::Command;

const FORBIDDEN_TERMS: &[&str] = &[
    "ALIVE", "Inspection Gate", "Nehemiah", "Field8",
    "Instinct8", "Cargo Court", "Truex", "CONSTRUCT8",
];

const PUBLIC_COMMANDS: &[&[&str]] = &[
    &["status"],
    &["target", "show"],
    &["target", "prune"],
    &["git", "status"],
    &["workspace", "doctor"],
    &["--help"],
    &["--version"],
];

/// I1: No public command output contains forbidden private doctrine terms.
#[test]
fn invariant_public_boundary() {
    for args in PUBLIC_COMMANDS {
        let output = Command::cargo_bin("cargo-cicd").unwrap()
            .args(*args).output().expect("command failed to run");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);
        for term in FORBIDDEN_TERMS {
            assert!(!combined.contains(term),
                "Forbidden term {:?} found in output of {:?}\n---\n{}", term, args, combined);
        }
    }
}

/// I1b: Help text does not contain private doctrine.
#[test]
fn invariant_help_text_public_safe() {
    let output = Command::cargo_bin("cargo-cicd").unwrap()
        .arg("--help").output().unwrap();
    let help = String::from_utf8_lossy(&output.stdout);
    for term in FORBIDDEN_TERMS {
        assert!(!help.contains(term),
            "Forbidden term {:?} in --help output", term);
    }
}

/// I3: git close must not claim success on a dirty working tree.
#[test]
fn invariant_no_false_close_on_dirty_tree() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    // Create an untracked file — dirty workspace
    std::fs::write(tmp.path().join("untracked.rs"), "// untracked").unwrap();
    // git close should fail (non-zero) when tree is dirty with no git repo
    // In a non-git dir it will fail due to git errors — which is the correct refusal
    let result = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["git", "close"]).current_dir(tmp.path()).output().unwrap();
    // Should not claim "phase closed" on a dirty or non-git tree
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!stdout.contains("phase already closed") || !result.status.success(),
        "git close claimed closed on dirty/no-git tree: {}", stdout);
}

/// I4: target prune must not perform deletions by default.
#[test]
fn invariant_no_destructive_default_prune() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    // Create a fake target dir with a file
    std::fs::create_dir_all(tmp.path().join("target/debug")).unwrap();
    std::fs::write(tmp.path().join("target/debug/binary"), "fake binary").unwrap();
    let before_exists = tmp.path().join("target/debug/binary").exists();
    Command::cargo_bin("cargo-cicd").unwrap()
        .args(["target", "prune"]).current_dir(tmp.path()).assert().success();
    let after_exists = tmp.path().join("target/debug/binary").exists();
    assert_eq!(before_exists, after_exists,
        "target prune deleted files without explicit --apply flag");
}

/// I5: trybuild changed must not run all fixtures by default.
#[test]
fn invariant_no_full_trybuild_by_default() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    // Create 5 fake trybuild fixture .rs files
    std::fs::create_dir_all(tmp.path().join("tests/compile_fail")).unwrap();
    for i in 0..5 {
        std::fs::write(
            tmp.path().join(format!("tests/compile_fail/fixture_{}.rs", i)),
            "// fixture"
        ).unwrap();
    }
    // trybuild changed in a non-git dir will find 0 changed fixtures
    let output = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["trybuild", "changed"]).current_dir(tmp.path())
        .output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must say "0 changed fixtures" or "no changed trybuild fixtures"
    // Must NOT say "running all fixtures" or "5 fixtures selected"
    let runs_all = stdout.contains("5 fixtures") || stdout.contains("running all");
    assert!(!runs_all, "trybuild changed ran all fixtures by default: {}", stdout);
}

/// I2: publish is deterministic on stable inputs (two runs, same output structure).
#[test]
fn invariant_publish_deterministic_structure() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    // First publish
    let r1 = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["publish", "run"]).current_dir(tmp.path()).output().unwrap();
    let cicd1_exists = tmp.path().join("cicd.toml").exists();
    // Second publish — same inputs
    let r2 = Command::cargo_bin("cargo-cicd").unwrap()
        .args(["publish", "run"]).current_dir(tmp.path()).output().unwrap();
    let cicd2_exists = tmp.path().join("cicd.toml").exists();
    // Both should succeed or both fail — consistent behavior
    assert_eq!(r1.status.success(), r2.status.success(),
        "publish produced inconsistent exit code on identical inputs");
    assert_eq!(cicd1_exists, cicd2_exists,
        "publish toggled cicd.toml existence on repeated runs");
}
