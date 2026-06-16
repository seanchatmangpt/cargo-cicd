# Release Checklist for cargo-cicd

**Version**: 26.6.3  
**Release Lead**: [Assignee]  
**Target Date**: [Date]  
**Go/No-Go Decision**: [ ] GO | [ ] NO-GO  

---

## 1. Pre-Release Validation Gates

### 1.1 Core Test Suite (All Tests Must Pass)

- [ ] **All tests pass**: `cargo test --all-features`
  - Runtime: ~3-5 minutes
  - Acceptable failure: None
  - Sign-off: CI/CD pipeline

- [ ] **Invariants pass**: `cargo test --test invariants`
  - 7 public boundary invariants enforced
  - Runtime: ~30 seconds
  - Acceptable failure: None (gates release)
  - Sign-off: Code Reviewer

- [ ] **Feature matrix tested**:
  - [ ] `cargo test --features process-data`
  - [ ] `cargo test --features autonomic`
  - [ ] `cargo test --features advanced`
  - [ ] `cargo test --features wasm4pm`
  - [ ] `cargo test --features "advanced,wasm4pm"`
  - Acceptable failure: None
  - Sign-off: Test Engineer

### 1.2 Integration Test Suite

- [ ] **CLI command projection**: `cargo test --test cli`
  - Validates all noun-verb grammar combinations
  - Runtime: ~1 minute
  - Sign-off: Code Reviewer

- [ ] **cicd.toml schema truth**: `cargo test --test cicd_toml_truth`
  - Validates carrier file serialization/deserialization
  - Runtime: ~30 seconds
  - Sign-off: Test Engineer

- [ ] **Autonomic policies**: `cargo test --test autonomic_policies`
  - Validates suggest-mode (non-destructive) behavior
  - Runtime: ~45 seconds
  - Sign-off: Code Reviewer

- [ ] **Changed test detection**: `cargo test --test changed_tests`
  - Validates crate change detection logic
  - Runtime: ~30 seconds
  - Sign-off: Test Engineer

- [ ] **Git phase closure**: `cargo test --test git_phase_closure`
  - Validates lawful branch-close sequence
  - Runtime: ~45 seconds
  - Sign-off: Code Reviewer

- [ ] **Feature projection**: `cargo test --test feature_projection`
  - Validates feature flag surface contract
  - Runtime: ~30 seconds
  - Sign-off: Test Engineer

### 1.3 wasm4pm Evidence Gates (Release-Critical)

**NOTE**: No release may claim success solely from cargo-cicd internal tests. Evidence-gate tests are mandatory for v26.6.3+.

- [ ] **Evidence gate passes**: `cargo test --test wasm4pm_evidence_gate --features wasm4pm`
  - Emits process evidence as XES (XML Event Stream)
  - Invokes wpm oracle: `wpm audit <file.xes>`
  - Invokes receipt doctor: `wpm receipt doctor --format json --strict <receipt.json>`
  - Asserts `Accept` verdict from both oracle and receipt doctor
  - Runtime: ~2 minutes
  - Acceptable failure: None (blocking release)
  - Sign-off: Evidence Gate Reviewer
  - Evidence location: `target/cargo-cicd/evidence/`

- [ ] **Harness validates**: `cargo test --test wasm4pm_harness --features wasm4pm`
  - Validates evidence collection instrumentation
  - Runtime: ~1 minute
  - Sign-off: Test Engineer

- [ ] **Mutation testing**: `cargo test --test wasm4pm_evidence_mutation --features wasm4pm`
  - Validates evidence robustness under runtime variations
  - Runtime: ~90 seconds
  - Sign-off: Test Engineer

- [ ] **Refusal cases**: `cargo test --test wasm4pm_refusal_cases --features wasm4pm`
  - Validates graceful handling of evidence-gate rejections
  - Runtime: ~45 seconds
  - Sign-off: Test Engineer

### 1.4 Code Quality Gates

- [ ] **No clippy warnings**: `cargo clippy --all-features -- -D warnings`
  - Runtime: ~2 minutes
  - Acceptable failure: None
  - Sign-off: Code Reviewer

- [ ] **Format clean**: `cargo fmt --all -- --check`
  - Runtime: ~30 seconds
  - Acceptable failure: None
  - Sign-off: Code Reviewer (auto-run `cargo fmt --all` before commit if needed)

- [ ] **Audit clean**: `cargo audit`
  - Runtime: ~1 minute
  - Acceptable failure: None (known CVEs must be documented in SECURITY.md)
  - Sign-off: Security Reviewer

- [ ] **Deny check** (if deny.toml configured): `cargo deny check`
  - Runtime: ~1 minute
  - Sign-off: Dependency Reviewer

### 1.5 Performance & Regression Testing

- [ ] **Benchmark comparison** (if benchmarks exist):
  - Baseline: Last stable release (26.6.2)
  - Target: Current branch
  - Acceptable regression: <= 10% p99 latency increase (documented)
  - Command: `cargo bench --all-features 2>&1 | tee bench-report-26.6.3.txt`
  - Runtime: ~5-10 minutes
  - Sign-off: Performance Reviewer

- [ ] **Binary size check**:
  - Release build: `cargo build --release`
  - Current size: _____ MB
  - Previous size (26.6.2): _____ MB
  - Acceptable growth: <= 15% or document justification
  - Sign-off: Code Reviewer

- [ ] **No deprecations introduced**:
  - [ ] Public API stable (no `#[deprecated]` added without migration period)
  - [ ] CLI output format unchanged (or versioned)
  - [ ] cicd.toml schema backwards-compatible
  - Sign-off: Code Reviewer

### 1.6 Forbidden Term Audit

cargo-cicd uses a ggen-manufactured ontology. Forbidden terms must not appear in public-facing surfaces (docs, CLI help, README, Cargo.toml description).

- [ ] **Scan code for forbidden terms**: `grep -r "ALIVE\|Inspection Gate\|Nehemiah\|Field8\|Instinct8\|Cargo Court\|AGI\|Truex\|CONSTRUCT8\|wall" src/ crates/ --include="*.rs" || true`
  - Acceptable: Matches only in comments with rationale, or in non-public modules
  - Sign-off: Code Reviewer

- [ ] **Scan docs for forbidden terms**: `grep -r "ALIVE\|Inspection Gate\|Nehemiah\|Field8\|Instinct8\|Cargo Court\|AGI\|Truex\|CONSTRUCT8" docs/ *.md --include="*.md" 2>/dev/null || true`
  - Acceptable: None in public documentation
  - Sign-off: Documentation Reviewer

- [ ] **Scan Cargo metadata**:
  - `Cargo.toml` description: Does not contain forbidden terms ✓
  - `Cargo.toml` keywords: Do not contain forbidden terms ✓
  - Sign-off: Code Reviewer

---

## 2. Documentation Readiness

### 2.1 Core Documentation

- [ ] **README.md updated**:
  - [ ] Version number updated (26.6.2 → 26.6.3)
  - [ ] New features described (if any)
  - [ ] Installation instructions valid (`cargo install cargo-cicd`)
  - [ ] Example commands working and up-to-date
  - [ ] Links and URLs valid
  - Sign-off: Documentation Reviewer

- [ ] **CHANGELOG.md created/updated**:
  - [ ] Entry for v26.6.3 with date
  - [ ] Sections: Features, Bugfixes, Breaking Changes, Deprecations, Internal
  - [ ] Links to related issues/PRs (if applicable)
  - [ ] All contributor names spelled correctly
  - [ ] Follows Keepachangelog.com format (recommended)
  - Sign-off: Release Lead

Example template:
```markdown
## [26.6.3] — 2026-06-14

### Added
- New feature description
- Another feature

### Fixed
- Bugfix description

### Changed
- Breaking change (if any)

### Internal
- Evidence gate refactored for wasm4pm v3.1

[26.6.3]: https://github.com/seanchatmangpt/cargo-cicd/compare/v26.6.2...v26.6.3
```

### 2.2 API & Code Documentation

- [ ] **Doc comments complete**:
  - [ ] All public modules have `//!` module-level docs
  - [ ] All public functions have doc comments with examples (if applicable)
  - [ ] All public structs have field documentation
  - Command: `cargo doc --no-deps --all-features --document-private-items 2>&1 | grep -i "missing\|error" || echo "Docs clean"`
  - Runtime: ~2 minutes
  - Sign-off: Documentation Reviewer

- [ ] **Examples updated**:
  - [ ] All example files in `examples/` directory run without error
  - [ ] Examples reflect current API
  - Command: `cargo test --example '*' 2>&1` (if examples test harness exists)
  - Sign-off: Code Reviewer

### 2.3 Migration & Known Issues

- [ ] **Migration guide (if breaking changes)**:
  - [ ] Document old API → new API mappings
  - [ ] Deprecation timeline clarified (how long old API supported)
  - [ ] Examples of migration patterns
  - File: `docs/MIGRATION-26.6.3.md`
  - Sign-off: Documentation Reviewer

- [ ] **Known issues documented**:
  - [ ] Open bugs tracked in GitHub Issues (not in code)
  - [ ] Workarounds documented in Issues
  - [ ] File: `docs/KNOWN_ISSUES.md` (if issues > 0)
  - Sign-off: Release Lead

### 2.4 CLAUDE.md Alignment

- [ ] **CLAUDE.md remains authoritative**:
  - [ ] Architecture section up-to-date
  - [ ] Build & test commands match actual commands
  - [ ] Feature flags section current
  - [ ] Test hierarchy documented accurately
  - [ ] No stale references to internal tools/paths
  - Sign-off: Code Reviewer

---

## 3. Code Quality Gate

### 3.1 Test Coverage

- [ ] **Test coverage maintained or improved**:
  - Baseline (26.6.2): _____ %
  - Current (26.6.3): _____ %
  - Minimum acceptable: 80% (new code only)
  - Tool: `cargo tarpaulin --all-features --out Html` (if tarpaulin installed)
  - Runtime: ~5-10 minutes
  - Sign-off: Test Engineer

- [ ] **All integration test fixtures valid**:
  - Location: `tests/fixtures/`
  - [ ] Fixture workspaces can be created and torn down
  - [ ] No hardcoded paths (use tempfile)
  - Command: `cargo test --test fixture_workspaces 2>&1 | grep -i "pass\|fail"`
  - Sign-off: Test Engineer

### 3.2 Code Hygiene

- [ ] **No new TODOs/FIXMEs without tracking**:
  - Command: `grep -r "TODO\|FIXME" src/ crates/ --include="*.rs" | grep -v "// TODO: tracked in #[0-9]\+" || true`
  - Acceptable: TODOs must reference GitHub issue number (e.g., `// TODO: tracked in #123`)
  - Sign-off: Code Reviewer

- [ ] **No debug prints/eprintln**:
  - Command: `grep -r "println!\|eprintln!\|dbg!" src/ crates/ --include="*.rs" | grep -v "^[[:space:]]*//\|test\|example" || true`
  - Acceptable: None in release builds (only in tests/examples)
  - Sign-off: Code Reviewer

- [ ] **No unwrap() without safety comment**:
  - Command: `grep -r "\.unwrap()" src/ crates/ --include="*.rs" | wc -l`
  - For each unwrap, verify: either in test code, or has preceding `// SAFETY: ...` comment
  - Sign-off: Code Reviewer

### 3.3 Security Audit

- [ ] **No hardcoded secrets**:
  - Command: `grep -r "password\|secret\|token\|key\|credential" src/ crates/ --include="*.rs" -i | grep -v "// \|test" || true`
  - Acceptable: None in source code (use env vars or config files)
  - Sign-off: Security Reviewer

- [ ] **No unsafe code without documentation**:
  - Command: `grep -r "unsafe" src/ crates/ --include="*.rs" | wc -l`
  - For each unsafe block: must have `// SAFETY: ...` comment explaining invariant
  - Sign-off: Security Reviewer

- [ ] **Safe dependency versions**:
  - Run: `cargo update --dry-run && cargo audit --deny warnings`
  - Acceptable: No advisories, or documented exemptions in Cargo.toml
  - Sign-off: Dependency Reviewer

### 3.4 Manufacturing Pipeline Integrity

- [ ] **ggen customization guard passes**: `cargo test --test ggen_customization_guard`
  - Validates that manual edits to ggen output don't break regeneration
  - Runtime: ~1 minute
  - Sign-off: Code Reviewer

- [ ] **Refusal calibration valid**: `cargo test --test refusal_calibration`
  - Validates autonomic policy refusal thresholds
  - Runtime: ~1 minute
  - Sign-off: Code Reviewer

- [ ] **LSP explain functional**: `cargo test --test lsp_explain --features "process-data"`
  - Language Server Protocol integration working
  - Runtime: ~30 seconds
  - Sign-off: Code Reviewer

---

## 4. Dependency Review

### 4.1 Dependency Audit

- [ ] **All new dependencies justified**:
  - [ ] No unnecessary deps added
  - [ ] Each new dep: rationale documented in commit message or CHANGELOG
  - [ ] Version constraints are reasonable (not `0.0.*` unless pinned)
  - Sign-off: Dependency Reviewer

- [ ] **Transitive dependencies clean**:
  - Run: `cargo tree --all-features --duplicates`
  - Acceptable: No unresolved duplicates, or duplicates documented
  - Runtime: ~1 minute
  - Sign-off: Dependency Reviewer

- [ ] **Feature flag dependencies validated**:
  - [ ] `process-data` features (core engine)
  - [ ] `autonomic` features (implies `process-data`)
  - [ ] `advanced` features (optional high-performance deps)
  - [ ] `wasm4pm` features (implies `process-data`)
  - [ ] `contrib` features (implies `process-data`)
  - Command: `cargo build --no-default-features --features advanced && cargo build --no-default-features --features wasm4pm`
  - Runtime: ~2 minutes
  - Sign-off: Test Engineer

### 4.2 Version Pinning

- [ ] **Dependencies appropriately pinned**:
  - Critical runtime deps (clap, serde): pinned to major version (e.g., `"4"`)
  - Development/optional deps: may use `~` or `^` as appropriate
  - Internal deps: pinned to exact version (e.g., `clap-noun-verb = "26.6.2"`)
  - Command: `cargo update --dry-run && cargo tree`
  - Sign-off: Dependency Reviewer

---

## 5. Sign-Off Matrix

| Role | Requirement | Sign-Off |
|------|-------------|----------|
| **Code Reviewer** | >= 1 approval on main PR | [ ] Name: ____________ Date: _______ |
| **Test Engineer** | >= 80% coverage (new code) | [ ] Name: ____________ Date: _______ |
| **Performance Reviewer** | <= 10% regression (or documented) | [ ] Name: ____________ Date: _______ |
| **Security Reviewer** | No CVEs/hardcoded secrets/unsafe code | [ ] Name: ____________ Date: _______ |
| **Evidence Gate Reviewer** | wasm4pm gates pass + receipts valid | [ ] Name: ____________ Date: _______ |
| **Documentation Reviewer** | README/CHANGELOG/docs complete | [ ] Name: ____________ Date: _______ |
| **Dependency Reviewer** | Audit clean, new deps justified | [ ] Name: ____________ Date: _______ |
| **Release Lead** | All gates passed, ready to publish | [ ] Name: ____________ Date: _______ |

**Gate Summary**:
- All mandatory gates signed: [ ]
- All optional gates reviewed: [ ]
- No blockers remaining: [ ]

---

## 6. Release Process Steps

### Phase 1: Final Validation (Day of Release)

1. **Create release branch**:
   ```bash
   git checkout -b release/26.6.3
   ```

2. **Bump version in Cargo.toml**:
   ```bash
   # Edit Cargo.toml: 26.6.2 → 26.6.3
   # Edit crates/*/Cargo.toml if applicable
   cargo check  # Verify dependencies resolve
   ```

3. **Run full test suite**:
   ```bash
   cargo test --all-features
   cargo test --test invariants
   cargo test --test wasm4pm_evidence_gate --features wasm4pm
   ```

4. **Run dry-publish**:
   ```bash
   cargo publish --dry-run
   ```
   - Acceptable error: None
   - Review package contents: `tar tzf target/package/cargo-cicd-26.6.3.crate | head -20`

5. **Commit version bump and CHANGELOG**:
   ```bash
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "chore(release): bump to 26.6.3"
   ```

6. **Create PR for release branch**:
   ```bash
   git push origin release/26.6.3
   gh pr create --title "Release 26.6.3" --body "$(cat CHANGELOG.md | head -20)..."
   ```

7. **Merge to main after approvals**:
   ```bash
   # After >= 1 code review + evidence gate pass
   gh pr merge --squash release/26.6.3
   ```

### Phase 2: Tag & Publish (After Merge to main)

8. **Create annotated git tag**:
   ```bash
   git checkout main && git pull
   git tag -a v26.6.3 -m "Release 26.6.3: wasm4pm evidence gate stabilized"
   git push origin v26.6.3
   ```

9. **Create GitHub Release** (auto-generated from tag or manual):
   ```bash
   gh release create v26.6.3 \
     --title "cargo-cicd 26.6.3" \
     --notes "$(cat CHANGELOG.md | sed -n '/^## \[26.6.3\]/,/^## \[/p' | head -20)"
   ```

10. **Publish to crates.io**:
    ```bash
    cargo publish
    ```
    - Check: https://crates.io/crates/cargo-cicd/26.6.3
    - Propagation time: ~1-2 minutes to be searchable

11. **Verify publication**:
    ```bash
    cargo install cargo-cicd --version 26.6.3
    cargo cicd status show  # Smoke test
    ```

### Phase 3: Announcements

12. **Announce release**:
    - [ ] GitHub release page (created above)
    - [ ] Email (optional): Maintainers + key users
    - [ ] Changelog summary posted to relevant channels

13. **Monitor for 24 hours**:
    - [ ] Check GitHub Issues for reported bugs
    - [ ] Monitor crates.io download stats
    - [ ] Verify docs render correctly on docs.rs

---

## 7. Post-Release Validation (48-Hour Window)

### 7.1 Immediate Checks (First 2 Hours)

- [ ] **Package found on crates.io**:
  - URL: https://crates.io/crates/cargo-cicd/26.6.3
  - Status: Yanked? [No], Indexed? [Yes]
  - Runtime: ~5 minutes
  - Sign-off: Release Lead

- [ ] **Documentation renders**:
  - URL: https://docs.rs/cargo-cicd/26.6.3
  - Check: All modules, functions documented
  - No broken links
  - Runtime: ~5 minutes
  - Sign-off: Documentation Reviewer

- [ ] **Fresh install works**:
  ```bash
  cargo install cargo-cicd --version 26.6.3 --force
  cargo cicd status show
  cargo cicd --version  # Verify version printed
  ```
  - Runtime: ~2-3 minutes
  - Sign-off: Release Lead

### 7.2 Extended Validation (First 24 Hours)

- [ ] **No critical issues reported**:
  - Monitor GitHub Issues for new reports tagged `[26.6.3]`
  - Acceptable: <= 1 non-critical bug
  - Sign-off: Release Lead

- [ ] **Download count normal**:
  - Baseline (26.6.2): _____ downloads/day
  - Current (26.6.3): _____ downloads/day
  - Acceptable variance: >= 80% of baseline (or growing if new feature)
  - URL: https://crates.io/crates/cargo-cicd/graphs/downloads
  - Runtime: Manual check
  - Sign-off: Release Lead

- [ ] **No regressions in CI/CD pipelines**:
  - Test: Run `cargo cicd` in real workspace (if applicable)
  - Acceptable: Works as 26.6.2
  - Sign-off: Release Lead

### 7.3 Post-Release (24-48 Hours)

- [ ] **Evidence gate receipts archived**:
  - Location: `target/cargo-cicd/evidence/v26.6.3/`
  - Format: XES files + receipt doctor output
  - Retention: Keep for 90 days minimum
  - Sign-off: Evidence Gate Reviewer

- [ ] **Performance metrics recorded**:
  - Benchmark results: `bench-report-26.6.3.txt`
  - Binary size: _____ MB
  - Test coverage: _____ %
  - Stored in: `docs/metrics/26.6.3/`
  - Sign-off: Performance Reviewer

- [ ] **Hotfix plan readiness** (if issues found):
  - Create branch: `hotfix/26.6.3.1` if needed
  - SLA for critical fix: < 24 hours
  - Sign-off: Release Lead

---

## 8. Rollback Plan (If Release Fails)

If post-release validation reveals critical issues:

1. **Yank from crates.io** (immediate):
   ```bash
   # On crates.io: "Yank" button in web UI
   # Or via API (if supported): cargo yank --vers 26.6.3
   ```

2. **Notify users** (within 1 hour):
   - GitHub Issue marked `[CRITICAL]`
   - Email blast (if user base > 1000)
   - Message in repo README (temporary warning)

3. **Prepare hotfix**:
   - Branch: `hotfix/26.6.3.1`
   - Accelerated testing: Minimal feature test + evidence gate only
   - Timeline: Publish within 24 hours

4. **Post-mortem**:
   - Root cause analysis
   - Prevention in future releases
   - Document in `docs/INCIDENTS.md`

---

## 9. Timeline & Scheduling

| Phase | Owner | Duration | Notes |
|-------|-------|----------|-------|
| Pre-Release Validation | Test Lead | 1-2 days | Run all test suites, evidence gates |
| Documentation Review | Doc Lead | 4-6 hours | README, CHANGELOG, API docs |
| Code Review & Sign-Off | Maintainers | 4-6 hours | >= 1 approval, all gates signed |
| Release Execution | Release Lead | 30 min | Tag, publish, verify |
| Post-Release Monitoring | Release Lead | 48 hours | Watch for issues, monitor stats |
| **Total Time to Stable** | — | **3-4 days** | From code freeze to production-ready |

### Recommended Schedule (For releases on Friday)

- **Monday-Wednesday**: Code development, testing, PR reviews
- **Thursday**: Pre-release validation, finalize CHANGELOG, sign-offs
- **Friday 9 AM**: Release execution (tag, publish)
- **Friday 10 AM - Monday 9 AM**: Post-release monitoring

---

## 10. Checklist Summary

| Gate | Status | Owner | Notes |
|------|--------|-------|-------|
| **Tests** | [ ] ✓ | Test Lead | All suites + evidence gates |
| **Docs** | [ ] ✓ | Doc Lead | README, CHANGELOG, API docs |
| **Code Quality** | [ ] ✓ | Code Reviewer | Clippy, fmt, audit, forbidden terms |
| **Deps** | [ ] ✓ | Dep Reviewer | Audit clean, new deps justified |
| **Security** | [ ] ✓ | Security Reviewer | No CVEs, no hardcoded secrets |
| **Performance** | [ ] ✓ | Perf Reviewer | <= 10% regression (or documented) |
| **Evidence** | [ ] ✓ | Evidence Reviewer | wasm4pm gates pass + receipts |
| **Sign-Off** | [ ] ✓ | Release Lead | All reviewers signed off |
| **Published** | [ ] ✓ | Release Lead | On crates.io, verified working |

**Overall Release Status**: [ ] READY TO SHIP

---

## Appendix A: Command Reference

### Comprehensive Test Run
```bash
#!/bin/bash
set -e

echo "=== Test Suite: All Features ==="
cargo test --all-features

echo "=== Test: Invariants ==="
cargo test --test invariants

echo "=== Test: Feature Matrix ==="
cargo test --features process-data
cargo test --features autonomic
cargo test --features advanced
cargo test --features wasm4pm

echo "=== Test: Evidence Gate (wasm4pm) ==="
cargo test --test wasm4pm_evidence_gate --features wasm4pm

echo "=== Code Quality ==="
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
cargo audit

echo "=== All gates passed! ==="
```

### Publish Checklist
```bash
#!/bin/bash
set -e

VERSION="26.6.3"

# 1. Dry run
echo "Publishing (dry-run)..."
cargo publish --dry-run

# 2. Tag
git tag -a "v${VERSION}" -m "Release ${VERSION}"
git push origin "v${VERSION}"

# 3. Publish
echo "Publishing to crates.io..."
cargo publish

# 4. Verify
sleep 5
cargo install cargo-cicd --version "${VERSION}" --force
cargo cicd --version

echo "Release ${VERSION} published successfully!"
```

---

## Appendix B: Known Issues & Workarounds

### v26.6.3 Known Limitations

1. **wasm4pm evidence gate requires wpm binary**:
   - The release includes XES evidence generation, but wpm binary must be installed separately
   - URL: https://github.com/seanchatmangpt/wasm4pm
   - Workaround: Set `WPM_SKIP=1` for testing without oracle

2. **CI/CD timeout on large workspaces**:
   - Known issue on workspaces with > 50 crates
   - Workaround: Run `cargo cicd target prune` before large test batches

3. **ggen regeneration requires manual review**:
   - If `ontology/cargo-cicd.ttl` changes, regeneration may have conflicts
   - Workaround: Review `git diff` after `ggen` and resolve manually

### Regression from 26.6.2

None documented as of release.

---

## Appendix C: Contact & Escalation

| Role | Owner | Contact | Escalation |
|------|-------|---------|------------|
| Release Lead | [Name] | [Email] | GitHub Issue #[number] |
| Test Lead | [Name] | [Email] | GitHub Issue #[number] |
| Code Lead | [Name] | [Email] | GitHub Issue #[number] |
| Infra Lead | [Name] | [Email] | GitHub Issue #[number] |

**Critical Issue (Blocking Release)**: Open GitHub Issue with label `[CRITICAL]` + `[v26.6.3]`

**Post-Release Issue**: Open GitHub Issue with label `[REGRESSION]` + `[v26.6.3]`

---

**Document Version**: 1.0  
**Last Updated**: 2026-06-14  
**Next Review**: After v26.6.3 release  
