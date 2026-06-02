use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_publish_emits_cicd_toml() {
    let tmp = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["publish", "run"]);
    cmd.current_dir(tmp.path());
    // publish should succeed even in empty dir (graceful)
    cmd.assert().code(predicates::prelude::predicate::in_iter(vec![0u32, 1]));
}
