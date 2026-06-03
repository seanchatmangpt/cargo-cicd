use assert_cmd::Command;

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
