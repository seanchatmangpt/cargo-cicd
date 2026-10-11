//! CLI integration tests for `cargo-cicd hooks` (install/uninstall/pre-tool-use).
//!
//! Chicago discipline: every assertion drives the REAL built binary via
//! CARGO_BIN_EXE_<CARGO_PKG_NAME> (same runtime-resolution style as
//! tests/version_parity_proof.rs) and asserts on real filesystem state and
//! real subprocess output. No mocks.

use std::process::Command;
use tempfile::TempDir;

fn bin() -> Command {
    let bin_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
    let bin = std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
        panic!("CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?")
    });
    Command::new(bin)
}

fn hooks_json(repo: &TempDir) -> String {
    let path = repo.path().join(".agents").join("hooks.json");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("hooks.json should exist at {:?}: {}", path, e))
}

#[test]
fn install_writes_hooks_json_and_reports() {
    let repo = TempDir::new().expect("tempdir");
    let out = bin()
        .args([
            "hooks",
            "install",
            "--repo",
            repo.path().to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("binary should run");

    assert!(
        out.status.success(),
        "hooks install exited non-zero: {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );

    let content = hooks_json(&repo);
    assert!(
        content.contains("cargo-cicd.execute"),
        "hook must use the cargo-cicd.execute key: {content}"
    );
    assert!(
        content.contains("hooks pre-tool-use --repo $WORKSPACE_DIR"),
        "hook must invoke the pre-tool-use verb: {content}"
    );
    assert!(
        !content.contains("cargo run"),
        "hook must not shell out through `cargo run`: {content}"
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(".agents/hooks.json"),
        "install output should name the hook path: {stdout}"
    );
}

#[test]
fn install_unsupported_provider_fails_nonzero_with_json() {
    let repo = TempDir::new().expect("tempdir");
    let out = bin()
        .args([
            "hooks",
            "install",
            "--repo",
            repo.path().to_str().expect("utf-8 path"),
            "--provider",
            "github",
        ])
        .output()
        .expect("binary should run");

    assert!(
        !out.status.success(),
        "unsupported provider must exit non-zero via the normal error path"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("unsupported_provider"),
        "stdout must carry the unsupported_provider error JSON: {stdout}"
    );
    assert!(
        stdout.contains("\"provider\":\"github\"")
            || stdout.contains("\"provider\": \"github\""),
        "error JSON must name the rejected provider: {stdout}"
    );
    assert!(
        !repo.path().join(".agents").exists(),
        "failed install must not create the .agents directory"
    );
}

#[test]
fn install_json_flag_emits_receipt_schema() {
    let repo = TempDir::new().expect("tempdir");
    let out = bin()
        .args([
            "hooks",
            "install",
            "--repo",
            repo.path().to_str().expect("utf-8 path"),
            "--json",
        ])
        .output()
        .expect("binary should run");
    assert!(out.status.success(), "install --json should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cargo-cicd.hooks.install.v1"),
        "--json receipt must carry the install schema: {stdout}"
    );
}

#[test]
fn uninstall_honors_repo_flag_and_is_idempotent() {
    let repo_a = TempDir::new().expect("tempdir a");
    let repo_b = TempDir::new().expect("tempdir b");
    let repo_arg = |r: &TempDir| r.path().to_str().expect("utf-8 path").to_string();

    for repo in [&repo_a, &repo_b] {
        let out = bin()
            .args(["hooks", "install", "--repo", &repo_arg(repo)])
            .output()
            .expect("binary should run");
        assert!(out.status.success(), "setup install should succeed");
    }
    assert!(repo_a.path().join(".agents/hooks.json").exists());
    assert!(repo_b.path().join(".agents/hooks.json").exists());

    let out = bin()
        .args(["hooks", "uninstall", "--repo", &repo_arg(&repo_a)])
        .output()
        .expect("binary should run");
    assert!(
        out.status.success(),
        "uninstall --repo should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !repo_a.path().join(".agents/hooks.json").exists(),
        "uninstall --repo must remove the target repo's hooks.json"
    );
    assert!(
        repo_b.path().join(".agents/hooks.json").exists(),
        "uninstall --repo must not touch other repos (repo flag was previously ignored)"
    );

    // Idempotent: second uninstall of an absent hook still exits 0.
    let out = bin()
        .args(["hooks", "uninstall", "--repo", &repo_arg(&repo_a)])
        .output()
        .expect("binary should run");
    assert!(
        out.status.success(),
        "uninstall must be idempotent, got {:?}",
        out.status
    );
}

#[test]
fn uninstall_without_flag_operates_on_cwd() {
    let repo = TempDir::new().expect("tempdir");
    let out = bin()
        .args(["hooks", "install"])
        .current_dir(repo.path())
        .output()
        .expect("binary should run");
    assert!(out.status.success(), "cwd install should succeed");
    assert!(
        repo.path().join(".agents/hooks.json").exists(),
        "cwd install should write ./.agents/hooks.json"
    );

    let out = bin()
        .args(["hooks", "uninstall"])
        .current_dir(repo.path())
        .output()
        .expect("binary should run");
    assert!(out.status.success(), "cwd uninstall should succeed");
    assert!(
        !repo.path().join(".agents/hooks.json").exists(),
        "flagless uninstall must remove ./.agents/hooks.json"
    );
}

#[test]
fn pre_tool_use_malformed_stdin_fails_open_with_allow() {
    // Fail-open is a documented deliberate default: deny-by-uncertainty would
    // brick user tooling when the hook envelope is unexpected.
    let repo = TempDir::new().expect("tempdir");
    use std::io::Write;
    let mut child = bin()
        .args([
            "hooks",
            "pre-tool-use",
            "--repo",
            repo.path().to_str().expect("utf-8 path"),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("binary should spawn");
    child
        .stdin
        .take()
        .expect("stdin piped")
        .write_all(b"this is not json {{{")
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");

    assert!(out.status.success(), "malformed stdin must exit 0 (fail-open)");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(r#""allow_tool": true"#) || stdout.contains(r#""allow_tool":true"#),
        "malformed stdin must yield an allow decision: {stdout}"
    );
}

#[test]
fn pre_tool_use_blocked_command_denies() {
    let repo = TempDir::new().expect("tempdir");
    let payload = r#"{"toolCall":{"args":{"CommandLine":"make release"}}}"#;
    use std::io::Write;
    let mut child = bin()
        .args([
            "hooks",
            "pre-tool-use",
            "--repo",
            repo.path().to_str().expect("utf-8 path"),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("binary should spawn");
    child
        .stdin
        .take()
        .expect("stdin piped")
        .write_all(payload.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait");

    assert!(
        !out.status.success(),
        "blocked command must exit non-zero"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(r#""allow_tool": false"#) || stdout.contains(r#""allow_tool":false"#),
        "blocked command must yield deny JSON: {stdout}"
    );
}
