use assert_cmd::Command;

#[test]
fn test_status_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.arg("status");
    // status may fail if no git repo but binary must run
    cmd.assert()
        .code(predicates::prelude::predicate::in_iter(vec![0i32, 1]));
}

#[test]
fn test_status_show_runs() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["status", "show"]);
    cmd.assert()
        .code(predicates::prelude::predicate::in_iter(vec![0i32, 1]));
}
