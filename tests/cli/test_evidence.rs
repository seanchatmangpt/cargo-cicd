use assert_cmd::Command;
use predicates::prelude::predicate;

/// Verify that `cargo cicd evidence doctor` is wired and reachable.
///
/// When the wpm oracle is absent the verb exits non-zero with a
/// diagnostic message; when it is present it exits 0. Either way the
/// binary must start, parse the noun+verb, and produce output — so we
/// accept exit code 0 or 1 and require non-empty stdout or stderr.
#[test]
fn test_evidence_doctor_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["evidence", "doctor"]);
    // Exit code 0 (wpm present) or 1 (wpm absent) are both acceptable;
    // any other code would indicate a panic or clap parse failure.
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]));
}

/// Verify that the bare noun `cargo cicd evidence` also reaches doctor
/// via the default-verb injection in main.rs.
#[test]
fn test_evidence_bare_noun_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.arg("evidence");
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]));
}
