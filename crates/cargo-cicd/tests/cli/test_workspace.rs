use assert_cmd::Command;

#[test]
fn test_workspace_doctor() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["workspace", "doctor"]);
    cmd.assert().success();
}
