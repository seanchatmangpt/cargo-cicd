use assert_cmd::Command;

#[test]
fn test_git_status() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["git", "status"]);
    // may fail if no git repo but binary must run
    cmd.assert()
        .code(predicates::prelude::predicate::in_iter(vec![0u32, 1]));
}
