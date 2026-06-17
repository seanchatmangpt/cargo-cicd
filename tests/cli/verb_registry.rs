use assert_cmd::Command;

#[test]
fn lsp_verbs_are_registered() {
    // doctor and explain are always registered
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "doctor", "--help"])
        .assert()
        .success();
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "explain", "--help"])
        .assert()
        .success();
}
