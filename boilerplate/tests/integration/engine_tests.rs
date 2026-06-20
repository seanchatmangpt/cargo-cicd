//! Integration tests for `EngineState`.
//!
//! These tests are feature-gated behind `process-data` and require a real
//! filesystem — they create temporary directories to simulate workspace roots.
//!
//! Run with:
//!   cargo test --test engine_tests --features process-data

#![cfg(feature = "process-data")]

use project_core::engine::EngineState;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Workspace fixture helpers
// ---------------------------------------------------------------------------

/// Minimal `Cargo.toml` for a single-crate workspace named `test-ws`.
const MINIMAL_CARGO_TOML: &str = r#"[package]
name = "test-ws"
version = "0.1.0"
edition = "2021"
"#;

/// `Cargo.toml` for a workspace with two members.
const WORKSPACE_CARGO_TOML: &str = r#"[workspace]
members = ["crate-a", "crate-b"]
resolver = "2"
"#;

const MEMBER_CARGO_TOML: &str = r#"[package]
name = "{NAME}"
version = "0.1.0"
edition = "2021"
"#;

/// Write a minimal single-crate workspace into `dir`.
fn setup_workspace(dir: &TempDir) {
    fs::write(dir.path().join("Cargo.toml"), MINIMAL_CARGO_TOML)
        .expect("failed to write Cargo.toml");
    // Cargo also expects a src/lib.rs (optional but realistic).
    fs::create_dir_all(dir.path().join("src")).expect("failed to create src/");
    fs::write(dir.path().join("src").join("lib.rs"), "// test workspace\n")
        .expect("failed to write src/lib.rs");
}

/// Write a multi-member workspace into `dir`.
fn setup_multi_member_workspace(dir: &TempDir) {
    fs::write(dir.path().join("Cargo.toml"), WORKSPACE_CARGO_TOML)
        .expect("failed to write workspace Cargo.toml");

    for name in &["crate-a", "crate-b"] {
        let crate_dir = dir.path().join(name);
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).expect("failed to create crate src dir");
        let manifest = MEMBER_CARGO_TOML.replace("{NAME}", name);
        fs::write(crate_dir.join("Cargo.toml"), &manifest)
            .expect("failed to write member Cargo.toml");
        fs::write(src_dir.join("lib.rs"), "// placeholder\n")
            .expect("failed to write member lib.rs");
    }
}

/// Initialise a git repository in `path` with a single empty commit.
/// Returns `false` and skips git operations if git is not available.
fn git_init_with_commit(path: &Path) -> bool {
    let git_available = Command::new("git")
        .args(["--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !git_available {
        return false;
    }

    let run = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(path)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .output()
            .expect("git command failed")
    };

    run(&["init"]);
    run(&["config", "user.email", "test@example.com"]);
    run(&["config", "user.name", "Test"]);

    // Stage whatever is already in the directory.
    run(&["add", "."]);
    run(&["commit", "--allow-empty", "-m", "init"]);

    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// `EngineState::from_workspace()` populates `workspace.name` from `Cargo.toml`.
///
/// The name adapter does a line-by-line scan of Cargo.toml; it must extract
/// "test-ws" correctly.
#[test]
fn engine_state_populates_workspace_name() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    let state = EngineState::from_workspace_at(dir.path())
        .expect("EngineState::from_workspace_at must not fail");

    assert_eq!(
        state.workspace.name.as_str(),
        "test-ws",
        "workspace.name should match the package name in Cargo.toml"
    );
}

/// `workspace.root_path` is set to the directory that was probed.
#[test]
fn engine_state_sets_root_path() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    let state = EngineState::from_workspace_at(dir.path())
        .expect("from_workspace_at must not fail");

    let root = state.workspace.root_path.as_str();
    // The path should contain the temp dir's path (canonical forms may differ).
    assert!(
        root.contains(dir.path().to_str().unwrap())
            || dir.path().to_str().unwrap().contains(root),
        "root_path {:?} should match temp dir {:?}",
        root,
        dir.path()
    );
}

/// Multi-member workspace: `workspace.members` lists both crates.
#[test]
fn engine_state_lists_workspace_members() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_multi_member_workspace(&dir);

    let state = EngineState::from_workspace_at(dir.path())
        .expect("from_workspace_at must not fail");

    let members = &state.workspace.members;
    assert!(
        members.len() >= 2,
        "expected at least 2 workspace members, got {}: {:?}",
        members.len(),
        members
    );

    let has_crate_a = members.iter().any(|m| m.contains("crate-a"));
    let has_crate_b = members.iter().any(|m| m.contains("crate-b"));
    assert!(has_crate_a, "crate-a should be in members: {:?}", members);
    assert!(has_crate_b, "crate-b should be in members: {:?}", members);
}

/// In a clean git repository (no uncommitted changes), `git.dirty_files` is 0.
#[test]
fn engine_state_clean_git_repo_has_no_dirty_files() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    if !git_init_with_commit(dir.path()) {
        // git not available — skip.
        return;
    }

    let state = EngineState::from_workspace_at(dir.path())
        .expect("from_workspace_at must not fail");

    assert_eq!(
        state.git.dirty_files.len(),
        0,
        "clean repo should have no dirty files; got: {:?}",
        state.git.dirty_files
    );
}

/// A git repo with an untracked file should report at least one dirty/untracked
/// entry.
///
/// We cannot always distinguish between "dirty" and "untracked" at the state
/// level, so we test that the combined count is > 0.
#[test]
fn engine_state_with_dirty_files_sets_dirty_count() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    if !git_init_with_commit(dir.path()) {
        return;
    }

    // Create an untracked file after the initial commit.
    fs::write(dir.path().join("untracked.txt"), b"untracked content")
        .expect("failed to write untracked file");

    let state = EngineState::from_workspace_at(dir.path())
        .expect("from_workspace_at must not fail");

    let dirty_count = state.git.dirty_files.len()
        + state.git.untracked_files.len();

    assert!(
        dirty_count > 0,
        "expected at least one dirty or untracked file after writing untracked.txt; got 0"
    );
}

/// After `git_init_with_commit` the branch name should be non-empty.
///
/// The default branch name varies by git version ("master" vs "main") so we
/// only assert non-emptiness.
#[test]
fn engine_state_with_git_repo_detects_branch() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    if !git_init_with_commit(dir.path()) {
        return;
    }

    let state = EngineState::from_workspace_at(dir.path())
        .expect("from_workspace_at must not fail");

    assert!(
        !state.git.branch.is_empty(),
        "expected a non-empty branch name after git init; got empty string"
    );
}

/// `EngineState::from_workspace_at` in a directory with no `Cargo.toml`
/// should return a default/partial state rather than panic.
///
/// Adapters are designed to silently fail; this test enforces that invariant.
#[test]
fn engine_state_in_non_workspace_dir_does_not_panic() {
    let dir = TempDir::new().expect("failed to create temp dir");
    // Deliberately do NOT write a Cargo.toml.

    // Must not panic.
    let state = EngineState::from_workspace_at(dir.path());

    // The result may be Ok(default) or Err — both are acceptable.
    // What is NOT acceptable is a panic (which would abort the test process).
    match state {
        Ok(s) => {
            // Partial state: workspace name is empty or a sentinel.
            let name = s.workspace.name.as_str();
            // No assertion on content — just confirm it didn't explode.
            let _ = name;
        }
        Err(_) => {
            // An error result is also acceptable.
        }
    }
}

/// Toolchain state is populated: `toolchain.rust_version` is non-empty when
/// `rustc` is available on PATH.
#[test]
fn engine_state_toolchain_version_nonempty_when_rustc_available() {
    let rustc_available = Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !rustc_available {
        return; // Skip if rustc not on PATH.
    }

    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    let state = EngineState::from_workspace_at(dir.path())
        .expect("from_workspace_at must not fail");

    assert!(
        !state.toolchain.rust_version.is_empty(),
        "toolchain.rust_version should not be empty when rustc is available"
    );
}

/// `ProcessEventState` starts empty for a fresh `from_workspace_at` call
/// (no events are emitted by construction alone).
#[test]
fn engine_state_process_events_initially_empty() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    let state = EngineState::from_workspace_at(dir.path())
        .expect("from_workspace_at must not fail");

    assert!(
        state.process_events.events.is_empty(),
        "process_events should be empty on a freshly constructed EngineState; \
         got {} events",
        state.process_events.events.len()
    );
}

/// Two consecutive calls to `from_workspace_at` on the same directory return
/// states that agree on `workspace.name`.
///
/// This verifies determinism of the adapter layer.
#[test]
fn engine_state_deterministic_across_two_calls() {
    let dir = TempDir::new().expect("failed to create temp dir");
    setup_workspace(&dir);

    let state_a = EngineState::from_workspace_at(dir.path())
        .expect("first call must succeed");
    let state_b = EngineState::from_workspace_at(dir.path())
        .expect("second call must succeed");

    assert_eq!(
        state_a.workspace.name,
        state_b.workspace.name,
        "workspace.name must be deterministic across two calls"
    );
}
