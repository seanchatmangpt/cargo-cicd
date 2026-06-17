# Release Process

This guide covers versioning, changelog format, wasm4pm validation gates, and the release checklist.

## Versioning

cargo-cicd uses **semantic versioning**: `MAJOR.MINOR.PATCH`

- **MAJOR** — breaking changes to public API (noun names, verb names, cicd.toml schema, exit codes)
- **MINOR** — new nouns/verbs, new state dimensions, new adapters (backward compatible)
- **PATCH** — bug fixes, internal refactoring, documentation (backward compatible)

Current version: **26.6.2** (see `Cargo.toml`)

### Version Bumping

Only maintainers bump the version in `Cargo.toml`. When ready for a release, update:

```toml
[package]
version = "26.6.3"  # or "26.7.0" or "27.0.0"
```

## Changelog Format

Maintain `CHANGELOG.md` in the repo root. Use this format:

```markdown
# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [26.7.0] — 2026-07-15

### Added
- New `workspace list-members` verb to enumerate workspace crates
- EngineState support for dependency graph (behind `process-data` feature)
- Evidence emission now includes git commit hash

### Changed
- `status show` output now includes target/ pressure indicator
- cicd.toml `[state]` section schema expanded with new fields

### Fixed
- Git dirty detection now correctly ignores .gitignore'd files
- Target pruning no longer removes symlinks

### Deprecated
- `target prune --dry-run` flag (use `target show` instead)

## [26.6.2] — 2026-06-14

### Added
- Initial release: `status`, `target`, `test`, `git`, `publish` nouns
- cicd.toml state carrier file
- wasm4pm evidence gate integration (feature-gated)
- Autonomic policies (feature-gated)
```

### Section Guidelines

- **Added** — new features, new nouns/verbs, new state dimensions
- **Changed** — behavior changes, schema changes (still backward compatible)
- **Fixed** — bug fixes
- **Deprecated** — APIs being removed in a future version (warn users now)
- **Removed** — previously deprecated APIs that are now gone
- **Security** — security fixes (use for critical issues)

## wasm4pm Validation Gates

**For v26.6.2 and later**, releases are gated by wasm4pm evidence validation. No release is complete without wasm4pm sign-off.

### Prerequisites

- `wpm` binary available at `/Users/sac/wasm4pm/target/release/wpm`
- Evidence directory: `target/cargo-cicd/evidence/`
- Receipts generated during test runs

### Release Validation Steps

1. **Run all tests to generate evidence:**

```bash
cargo test --all-features
```

This will emit XES files to `target/cargo-cicd/evidence/`.

2. **Run the evidence-gate tests:**

```bash
cargo test --test wasm4pm_evidence_gate --features wasm4pm
```

These tests:
- Invoke `wpm audit <file.xes>` on each evidence file
- Invoke `wpm receipt doctor --format json --strict` on generated receipts
- Assert that wasm4pm returns **Accept** verdict (not Refuse or Undecidable)

3. **If tests fail:**

- Check `wpm audit` output for evidence format issues
- Check `wpm receipt doctor` output for receipt issues
- Investigate and fix the root cause
- Regenerate evidence by re-running tests
- Re-run evidence-gate tests

4. **Once tests pass:**

- Prepare release notes (update CHANGELOG.md)
- Tag the commit: `git tag -a v26.6.3 -m "Release 26.6.3: ..."`
- Push: `git push origin v26.6.3`
- Publish to crates.io: `cargo publish` (maintainer only)

### Example Evidence-Gate Test

```rust
#[test]
fn test_evidence_gate_accepts_clean_run() {
    // Run command to generate evidence
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .arg("status")
        .arg("show")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // Validate with wasm4pm
    let audit_result = Command::new("wpm")
        .args(&["audit", "target/cargo-cicd/evidence/session-001.xes"])
        .output()
        .unwrap();
    
    let audit_output = String::from_utf8_lossy(&audit_result.stdout);
    assert!(audit_output.contains("Accept"), "wasm4pm should accept evidence");
}
```

## Release Checklist

Before creating a release tag:

- [ ] All tests pass: `cargo test --all-features`
- [ ] Invariants pass: `cargo test --test invariants`
- [ ] Evidence-gate tests pass: `cargo test --test wasm4pm_evidence_gate --features wasm4pm`
- [ ] No forbidden terms in help text: checked by invariants test
- [ ] Feature flags work correctly: `cargo test --features process-data`, `cargo test --features autonomic`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Lints pass: `cargo clippy -- -D warnings`
- [ ] CHANGELOG.md updated with all changes
- [ ] Version bumped in `Cargo.toml`
- [ ] README.md updated if user-facing behavior changed
- [ ] CLAUDE.md updated if architectural changes
- [ ] All PRs merged and commits squashed
- [ ] Branch is clean: `git status` shows nothing

### Step-by-Step Release

1. **Create a release branch:**

```bash
git checkout -b release/26.7.0
```

2. **Update version in Cargo.toml:**

```toml
[package]
version = "26.7.0"
```

3. **Update CHANGELOG.md:**

```bash
# Add new [26.7.0] section at the top
# List all changes since last release
# See "Changelog Format" above
```

4. **Commit the release prep:**

```bash
git commit -m "chore(release): prepare 26.7.0

https://claude.ai/code/session_XX"
```

5. **Run final test suite:**

```bash
cargo clean
cargo test --all-features
cargo test --test invariants
cargo test --test wasm4pm_evidence_gate --features wasm4pm
```

6. **Tag the release:**

```bash
git tag -a v26.7.0 -m "Release 26.7.0: [summary of changes]"
```

7. **Push to GitHub:**

```bash
git push origin release/26.7.0
git push origin v26.7.0
```

8. **Publish to crates.io** (maintainer only):

```bash
cargo publish --allow-dirty
```

(Use `--allow-dirty` because CLAUDE.md is excluded from the package.)

9. **Create GitHub Release:**

- Go to GitHub Releases
- Click "Draft a new release"
- Select the tag `v26.7.0`
- Title: "v26.7.0: [summary]"
- Description: copy from CHANGELOG.md
- Attach any binary artifacts if applicable
- Publish

10. **Merge release branch to main:**

```bash
git checkout main
git pull origin main
git merge release/26.7.0
git push origin main
```

11. **Close the release branch:**

```bash
git branch -d release/26.7.0
git push origin :release/26.7.0  # Delete remote
```

## Hotfix Releases

If a critical bug is found after release:

1. **Create a hotfix branch from the tag:**

```bash
git checkout -b hotfix/26.6.3-critical v26.6.2
```

2. **Fix the bug and test:**

```bash
# Make changes
cargo test
cargo test --test wasm4pm_evidence_gate --features wasm4pm
```

3. **Bump patch version in Cargo.toml:**

```toml
version = "26.6.3"
```

4. **Update CHANGELOG.md with the fix**

5. **Tag and release:**

```bash
git tag -a v26.6.3 -m "Hotfix 26.6.3: [bug description]"
git push origin hotfix/26.6.3-critical
git push origin v26.6.3
cargo publish --allow-dirty
```

6. **Merge back to main:**

```bash
git checkout main
git pull origin main
git merge hotfix/26.6.3-critical --no-ff
git push origin main
```

## Publishing to crates.io

Only maintainers can publish. Require:

- crates.io account with publish permission
- GitHub SSH key configured
- Cargo.toml version already bumped
- All tests passing

```bash
# Dry run (checks what would be published)
cargo publish --dry-run --allow-dirty

# If successful, publish for real
cargo publish --allow-dirty
```

The `--allow-dirty` flag is needed because `CLAUDE.md`, `ggen.toml`, and other development files are excluded from the package (see `Cargo.toml` `exclude` list).

## Monitoring Releases

After publishing:

1. **Verify on crates.io:**
   - Visit https://crates.io/crates/cargo-cicd/26.7.0
   - Check that the README, docs link, and version are correct

2. **Test installation:**

```bash
cargo install cargo-cicd --version 26.7.0
cargo cicd --version
```

3. **Update internal docs** if needed (e.g., ggen.toml version reference)

## Deprecation Policy

When removing functionality:

1. **In version N:** Mark as deprecated in docs and with `#[deprecated]` attribute
2. **In changelog:** Document in "Deprecated" section
3. **In version N+1 or N+2:** Remove the functionality (after at least one release)
4. **In CHANGELOG:** Document removal in "Removed" section

Example:

```rust
#[deprecated(
    since = "26.7.0",
    note = "use `status show` instead of `status audit`"
)]
pub fn audit() -> Result<()> {
    // ... old implementation ...
}
```

Then in CHANGELOG:

```markdown
### Deprecated
- `status audit` verb (use `status show` instead; will be removed in 26.8.0)

## [26.8.0]

### Removed
- `status audit` verb (use `status show` instead)
```
