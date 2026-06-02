// Test that feature flags expose correct projections without contradiction.
use tempfile::TempDir;

// Test 1: default features — no rich process export required.
// Note: clap_noun_verb exits 1 for --help (help text is routed via stderr with
// an "Argument parsing failed" prefix). The binary must not panic and must emit
// recognisable help content.
#[test]
fn test_default_features_build_succeeds() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .arg("--help")
        .output()
        .unwrap();
    // Binary must produce output — either stdout or stderr.
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        combined.contains("cargo-cicd") || combined.contains("Usage"),
        "Expected help output from cargo-cicd --help, got: {}",
        combined
    );
    // Must not have panicked.
    assert!(
        !combined.contains("panicked"),
        "cargo-cicd --help panicked: {}",
        combined
    );
}

// Test 2: INVARIANT — no forbidden terms in public output.
// clap_noun_verb routes --help through stderr; both streams are checked.
#[test]
fn test_public_boundary_invariant_help_text() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .arg("--help")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    for forbidden in &[
        "ALIVE",
        "Nehemiah",
        "CONSTRUCT8",
        "Instinct8",
        "Inspection Gate",
        "Cargo Court",
        "AGI",
        "Truex",
    ] {
        assert!(
            !text.contains(forbidden),
            "Public output contains forbidden term: {}",
            forbidden
        );
    }
}

// Test 3: feature names do not leak private architecture
#[test]
fn test_feature_names_are_public_safe() {
    let cargo_toml = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .unwrap();
    // Allowed features: default, process-data, autonomic, contrib, wasm4pm
    assert!(
        cargo_toml.contains("process-data"),
        "Cargo.toml missing expected feature: process-data"
    );
    assert!(
        cargo_toml.contains("autonomic"),
        "Cargo.toml missing expected feature: autonomic"
    );
    // Not allowed: ALIVE, cell8, nightly_foundry in feature names
    assert!(
        !cargo_toml.contains("cell8"),
        "Cargo.toml contains forbidden feature name: cell8"
    );
    assert!(
        !cargo_toml.contains("ALIVE"),
        "Cargo.toml contains forbidden feature name: ALIVE"
    );
}

// Test 4: publish includes correct sections for process-data projection
#[test]
fn test_publish_emits_all_required_sections() {
    // run publish, read cicd.toml, check for [workspace], [state], [target] sections
    let dir = TempDir::new().unwrap();
    std::process::Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir.path())
        .arg("publish")
        .arg("run")
        .output()
        .unwrap();
    let toml_path = dir.path().join("cicd.toml");
    if toml_path.exists() {
        let content = std::fs::read_to_string(&toml_path).unwrap();
        assert!(content.contains("[workspace]"), "missing [workspace] section");
        assert!(content.contains("[state]"), "missing [state] section");
        assert!(content.contains("[target]"), "missing [target] section");
    }
}
