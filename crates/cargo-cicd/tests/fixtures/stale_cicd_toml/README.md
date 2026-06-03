# Fixture: stale_cicd_toml

Valid but stale cicd.toml: state.dirty=false when workspace IS dirty. Tests that
cargo-cicd detects stale state and does not trust cached values.
