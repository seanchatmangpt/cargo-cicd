use assert_cmd::Command;

// INVARIANT 1: Public Boundary — no forbidden terms in any public output
#[test]
fn invariant_public_boundary_no_forbidden_terms_in_all_help() {
    let forbidden = [
        "ALIVE",
        "Nehemiah",
        "CONSTRUCT8",
        "Instinct8",
        "Inspection Gate",
        "Cargo Court",
        "AGI",
        "Truex",
        "Field8",
        "wall",
    ];
    let noun_verbs = [
        vec!["--help"],
        vec!["status", "--help"],
        vec!["target", "--help"],
        vec!["target", "show", "--help"],
        vec!["test", "--help"],
        vec!["trybuild", "--help"],
        vec!["git", "--help"],
        vec!["publish", "--help"],
        vec!["workspace", "--help"],
    ];
    for args in &noun_verbs {
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .args(args.iter())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout).to_string()
            + &String::from_utf8_lossy(&output.stderr);
        for term in &forbidden {
            assert!(
                !text.contains(term),
                "Forbidden term '{}' found in output of: cargo cicd {}",
                term,
                args.join(" ")
            );
        }
    }
}

// INVARIANT 4: No Destructive Default
#[test]
fn invariant_no_destructive_default_target_prune_is_safe() {
    use tempfile::TempDir;
    let dir = TempDir::new().unwrap();
    // Create a fake target dir with files
    let fake_target = dir.path().join("target/debug");
    std::fs::create_dir_all(&fake_target).unwrap();
    std::fs::write(fake_target.join("binary"), b"ELF fake binary").unwrap();
    // Run prune WITHOUT --confirm
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("target")
        .arg("prune")
        .output()
        .unwrap();
    // INVARIANT: binary must still exist after prune without confirmation
    assert!(
        fake_target.join("binary").exists(),
        "target prune without --confirm must not delete files"
    );
    let _ = output;
}

// INVARIANT 5: No Full Trybuild By Default
#[test]
fn invariant_no_full_trybuild_by_default() {
    use tempfile::TempDir;
    let dir = TempDir::new().unwrap();
    // Create a large fixture set (100 files)
    let ui_dir = dir.path().join("tests/ui/compile_fail");
    std::fs::create_dir_all(&ui_dir).unwrap();
    for i in 0..100 {
        std::fs::write(ui_dir.join(format!("fixture_{}.rs", i)), b"fn main() {}").unwrap();
    }
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("trybuild")
        .arg("changed")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    // Must NOT mention running 100 fixtures or 'all'
    // It should either report 0 changed (no git) or report changed subset
    assert!(
        !combined.contains("100 fixtures") && !combined.contains("all 100"),
        "trybuild changed must not run all 100 fixtures: {}",
        &combined[..combined.len().min(200)]
    );
}

// INVARIANT: Status command exits 0 (baseline health check)
#[test]
fn invariant_status_exits_zero() {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["status", "show"])
        .assert()
        .success();
}

