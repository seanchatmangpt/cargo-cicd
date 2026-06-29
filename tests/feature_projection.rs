// Test that feature flags expose correct projections without contradiction.

// Test: feature names do not leak private architecture
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
