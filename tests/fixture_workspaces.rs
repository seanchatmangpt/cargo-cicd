/// Integration tests that use the FixtureWorkspace helpers directly.
///
/// Each test constructs a fixture, verifies the on-disk state is correct,
/// and confirms the fixture can be used as a `current_dir` for assert_cmd.
mod fixtures;

use fixtures::FixtureWorkspace;

#[test]
fn fixture_clean_has_cargo_toml() {
    let ws = FixtureWorkspace::clean();
    assert!(
        ws.root.join("Cargo.toml").exists(),
        "clean fixture must have Cargo.toml"
    );
    assert!(
        !ws.root.join("target").exists(),
        "clean fixture must not have target/"
    );
    assert!(
        !ws.root.join("cicd.toml").exists(),
        "clean fixture must not have cicd.toml"
    );
}

#[test]
fn fixture_missing_manifest_has_no_cargo_toml() {
    let ws = FixtureWorkspace::missing_manifest();
    assert!(
        !ws.root.join("Cargo.toml").exists(),
        "missing_manifest fixture must not have Cargo.toml"
    );
}

#[test]
fn fixture_dirty_has_untracked_file() {
    let ws = FixtureWorkspace::dirty();
    assert!(
        ws.root.join("Cargo.toml").exists(),
        "dirty fixture must have Cargo.toml"
    );
    assert!(
        ws.root.join("untracked.txt").exists(),
        "dirty fixture must have untracked file"
    );
}

#[test]
fn fixture_toolchain_mismatch_has_toolchain_file() {
    let ws = FixtureWorkspace::with_toolchain_mismatch();
    assert!(
        ws.root.join("rust-toolchain.toml").exists(),
        "toolchain mismatch fixture must have rust-toolchain.toml"
    );
    let content = std::fs::read_to_string(ws.root.join("rust-toolchain.toml")).unwrap();
    assert!(
        content.contains("1.50.0"),
        "toolchain file must declare old channel"
    );
}
