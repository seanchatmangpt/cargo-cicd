use assert_cmd::Command;

#[test]
fn test_target_show() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["target", "show"]);
    cmd.assert().success();
}

#[test]
fn test_target_prune_plan() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["target", "prune"]);
    // prune in plan mode should always succeed
    cmd.assert().success();
}
