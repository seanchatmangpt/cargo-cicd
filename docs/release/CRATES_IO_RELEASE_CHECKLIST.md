# crates.io Release Checklist

Use this checklist before each release to verify all conditions are met.

## Pre-Release Conditions

- [ ] 1. `Cargo.toml` version field is updated and matches intended release version
- [ ] 2. `README.md` exists and contains accurate install and usage instructions
- [ ] 3. `LICENSE-MIT` is present with correct copyright year and holder
- [ ] 4. `LICENSE-APACHE` is present with correct copyright year and holder
- [ ] 5. `Cargo.toml` `license` field is set to `MIT OR Apache-2.0`
- [ ] 6. `Cargo.toml` `description` field is present and crates.io-safe
- [ ] 7. `Cargo.toml` `repository` field points to the correct GitHub URL
- [ ] 8. `Cargo.toml` `documentation` field is set (or omitted to use docs.rs default)
- [ ] 9. All public API items have doc comments
- [ ] 10. `cargo doc --no-deps` completes without warnings
- [ ] 11. `cargo test` passes on a clean checkout
- [ ] 12. No forbidden internal terms appear in any public-facing file (README, CLI help, docs)
- [ ] 13. `cicd.toml` does not contain internal-only fields that would confuse external users
- [ ] 14. Feature flags are documented in `README.md` or `Cargo.toml` docs
- [ ] 15. CHANGELOG or release notes are updated for the release version
- [ ] 16. `cargo publish --dry-run` completes without error

## Post-Release Verification

- [ ] crates.io page renders README correctly
- [ ] docs.rs build succeeds
- [ ] `cargo install cargo-cicd` installs and the binary runs
