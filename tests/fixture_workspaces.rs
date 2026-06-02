/// Integration tests that use the FixtureWorkspace helpers directly.
///
/// Each test constructs a fixture, verifies the on-disk state is correct,
/// and confirms the fixture can be used as a `current_dir` for assert_cmd.
mod fixtures;

use fixtures::FixtureWorkspace;

#[test]
fn fixture_clean_has_cargo_toml() {
    let ws = FixtureWorkspace::clean();
    assert!(ws.root.join("Cargo.toml").exists(), "clean fixture must have Cargo.toml");
    assert!(!ws.root.join("target").exists(), "clean fixture must not have target/");
    assert!(!ws.root.join("cicd.toml").exists(), "clean fixture must not have cicd.toml");
}

#[test]
fn fixture_missing_manifest_has_no_cargo_toml() {
    let ws = FixtureWorkspace::missing_manifest();
    assert!(!ws.root.join("Cargo.toml").exists(), "missing_manifest fixture must not have Cargo.toml");
}

#[test]
fn fixture_dirty_has_untracked_file() {
    let ws = FixtureWorkspace::dirty();
    assert!(ws.root.join("Cargo.toml").exists(), "dirty fixture must have Cargo.toml");
    assert!(ws.root.join("untracked.txt").exists(), "dirty fixture must have untracked file");
}

#[test]
fn fixture_toolchain_mismatch_has_toolchain_file() {
    let ws = FixtureWorkspace::with_toolchain_mismatch();
    assert!(ws.root.join("rust-toolchain.toml").exists(), "toolchain mismatch fixture must have rust-toolchain.toml");
    let content = std::fs::read_to_string(ws.root.join("rust-toolchain.toml")).unwrap();
    assert!(content.contains("1.50.0"), "toolchain file must declare old channel");
}

#[test]
fn fixture_target_over_limit_has_target_dir() {
    let ws = FixtureWorkspace::with_target_over_limit();
    let placeholder = ws.root.join("target").join("debug").join("placeholder.bin");
    assert!(placeholder.exists(), "over-limit fixture must have target/debug/placeholder.bin");
    let meta = std::fs::metadata(&placeholder).unwrap();
    assert_eq!(meta.len(), 1_048_576, "placeholder must be exactly 1 MB");
}

#[test]
fn fixture_corrupted_cicd_toml_is_invalid_toml() {
    let ws = FixtureWorkspace::with_corrupted_cicd_toml();
    let content = std::fs::read_to_string(ws.root.join("cicd.toml")).unwrap();
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(parsed.is_err(), "corrupted_cicd_toml fixture must fail TOML parse");
}

#[test]
fn fixture_stale_cicd_toml_claims_clean_but_is_dirty() {
    let ws = FixtureWorkspace::with_stale_cicd_toml();
    let content = std::fs::read_to_string(ws.root.join("cicd.toml")).unwrap();
    assert!(content.contains("dirty = false"), "stale cicd.toml must claim dirty=false");
    assert!(ws.root.join("untracked.txt").exists(), "stale fixture workspace must actually be dirty");
}

#[test]
fn fixture_changed_trybuild_has_ui_dir() {
    let ws = FixtureWorkspace::with_changed_trybuild_fixture();
    let ui_dir = ws.root.join("tests").join("ui");
    assert!(ui_dir.exists(), "trybuild fixture must have tests/ui/");
    let changed = ui_dir.join("changed_fixture.rs");
    assert!(changed.exists(), "trybuild fixture must have changed_fixture.rs");
    // Ten pre-existing fixtures.
    for i in 0..10 {
        let existing = ui_dir.join(format!("existing_{i:02}.rs"));
        assert!(existing.exists(), "trybuild fixture must have existing_{i:02}.rs");
    }
}
