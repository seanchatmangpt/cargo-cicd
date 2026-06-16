# /test — Run cargo-cicd Test Suites

Run tests for the cargo-cicd Rust workspace. Understands the stratified test hierarchy, feature flags, and offline oracle behaviour.

---

## Test Hierarchy

cargo-cicd tests are stratified by gate type:

### Tier 1 — Unit & Smoke Tests (non-closing)

These run fast, require no external binaries, and validate public boundaries and internal logic.

| Suite | File | What it validates |
|-------|------|-------------------|
| `invariants` | `tests/invariants.rs` | 7 non-negotiable public boundary rules (no forbidden terms, binary name, exit codes, safety warnings) |
| `cli` | `tests/cli/` | Noun/verb CLI parsing, dispatch, and output format for all nouns |
| `feature_projection` | `tests/feature_projection.rs` | Feature flag surface contract — flags compile and gate the right code |
| `autonomic_policies` | `tests/autonomic_policies.rs` | Policy evaluation logic (target pressure, git dirty, toolchain mismatch, etc.) |
| `git_phase_closure` | `tests/git_phase_closure.rs` | Git state detection accuracy (branch, dirty files, ahead/behind) |
| `changed_tests` | `tests/changed_tests.rs` | Changed-file classification accuracy for `.rs` files and trybuild fixtures |
| `cicd_toml_truth` | `tests/cicd_toml_truth.rs` | `cicd.toml` serialization/deserialization round-trip |

`invariants` is always the **first gate** — if it fails, stop and fix before running anything else. A forbidden term in help output or a wrong exit code invalidates the release boundary.

### Tier 2 — Evidence Gate Tests (closing — release gate)

These require the `wpm` oracle binary. They assert on the wasm4pm verdict, never on cargo-cicd internal state.

| Suite | File | What it validates |
|-------|------|-------------------|
| `wasm4pm_evidence_gate` | `tests/wasm4pm_evidence_gate.rs` | Happy-path evidence emission → oracle adjudication (`Accept`) |
| `wasm4pm_evidence_mutation` | `tests/wasm4pm_evidence_mutation.rs` | Corrupt or mutated evidence is rejected by oracle (`Refuse`) |
| `wasm4pm_refusal_cases` | `tests/wasm4pm_refusal_cases.rs` | Edge cases: oracle unavailable, malformed XES, missing fields |

**No release without Tier 2 passing with a live oracle.**

---

## Usage

### Run everything (default)

```bash
cargo make test
```

Runs all Tier 1 and Tier 2 suites in the correct order. Requires `cargo-make` installed (`cargo install cargo-make`).

Fallback without cargo-make:

```bash
cargo test
```

### Run a single suite by name

```bash
cargo test --test invariants
cargo test --test cli
cargo test --test feature_projection
cargo test --test autonomic_policies
cargo test --test git_phase_closure
cargo test --test changed_tests
cargo test --test cicd_toml_truth
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```

### Run a specific test function within a suite

```bash
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
cargo test --test cli test_status_show_exits_zero
```

### Run with feature flags

Some test suites only compile or exercise meaningful paths under specific features:

```bash
# Enable Level 5 engine internals (EngineState, adapters, cicd.toml)
cargo test --features process-data

# Enable autonomic policy suggestions (implies process-data)
cargo test --features autonomic

# Enable wasm4pm oracle integration (implies process-data)
cargo test --features wasm4pm

# Combine features
cargo test --features autonomic,wasm4pm
```

---

## Recommended execution order

1. **Always start with invariants** — catches forbidden terms and broken CLI boundaries immediately:
   ```bash
   cargo test --test invariants
   ```

2. **Run Tier 1 suites** — fast, no external dependencies:
   ```bash
   cargo test --test cli
   cargo test --test feature_projection
   cargo test --test autonomic_policies
   cargo test --test changed_tests
   cargo test --test git_phase_closure
   cargo test --test cicd_toml_truth
   ```

3. **Run Tier 2 evidence gate** — requires `wpm` on PATH:
   ```bash
   cargo test --test wasm4pm_evidence_gate -- --nocapture
   cargo test --test wasm4pm_evidence_mutation
   cargo test --test wasm4pm_refusal_cases
   ```

---

## Offline environments — ExpectedWpmVerdict::Blocked

If `wpm` is not installed, Tier 2 tests must declare:

```rust
ExpectedWpmVerdict::Blocked
```

This is a **first-class expectation**, not an error. It means: "the oracle was unavailable; verdict adjudication was skipped." Tests that expect `Blocked` pass in CI without wpm, but cannot close the release gate.

For a release, `Blocked` must be replaced with a live `Accept` from the oracle. If you see `Blocked` in a release gate run:

1. Check `wpm` is installed: `which wpm && wpm --version`
2. Add the wpm binary directory to `PATH`
3. Re-run the Tier 2 suite

---

## What each Tier 1 invariant enforces

The `tests/invariants.rs` suite enforces 7 non-negotiable rules:

1. **No forbidden terms in help output** — scans every noun/verb `--help` for banned internal terms (`ALIVE`, `Inspection Gate`, `wall`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`)
2. **No destructive action without `--confirm`** — prune and close verbs must require explicit confirmation
3. **No full trybuild run by default** — `trybuild changed` only runs changed fixtures; conservative mode must be active
4. **Noun names are lowercase ASCII** — enforces consistent CLI grammar
5. **Binary name is `cargo-cicd`** — the installed binary must match
6. **Status command exits 0** — `cargo cicd status show` is the baseline health check; it must always succeed
7. **Git close has safety warnings** — `git close` must print a warning before any destructive phase transition

---

## Reporting

After running, check:

- Exit code 0 = all tests passed
- Exit code non-zero = at least one test failed; read the `FAILED` lines in output

For verbose output:

```bash
cargo test --test invariants -- --nocapture
cargo test -- --nocapture 2>&1 | tee test-output.log
```

To count failures:

```bash
cargo test 2>&1 | grep -E "^(test .* FAILED|FAILED|error)"
```
