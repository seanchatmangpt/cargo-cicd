<!-- BEGIN custom:introduction -->
# cargo-cicd

**cargo-cicd keeps Rust workspaces clean, fast, and push-ready.**

[![Crates.io](https://img.shields.io/crates/v/cargo-cicd.svg)](https://crates.io/crates/cargo-cicd)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

`cargo-cicd` is a Cargo subcommand that brings CI/CD discipline to your local
workflow. Run it before you push — catch dirty trees, bloated target
directories, broken trybuild fixtures, and workspace health issues before they
reach remote pipelines.
<!-- END custom:introduction -->

---

## Install

```sh
cargo install cargo-cicd
```

Verify:

```sh
cargo cicd --version
# cargo-cicd 26.6.19
```

If the binary is not found, ensure `~/.cargo/bin` is on your `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

---

## Quick Start

Five commands to get going in any Cargo workspace:

```sh
# 1. Check workspace health
cargo cicd workspace doctor

# 2. See overall workspace status
cargo cicd status

# 3. Run only the tests affected by your changes
cargo cicd test changed

# 4. Check the target directory size and age
cargo cicd target show

# 5. Review git state before pushing
cargo cicd git status
```

That's the daily loop. When you're ready to publish:

```sh
cargo cicd publish run
```

---

## Commands

cargo-cicd uses a `<noun> <verb>` grammar. Every noun has a default verb, so
bare nouns work too (`cargo cicd status` = `cargo cicd status show`).

<!-- BEGIN ggen:commands -->
<!-- Rendered from ontology/cargo-cicd.ttl. Do not edit by hand. -->

| Command | Description |
|---------|-------------|
| `cargo cicd status show` | Displays the current workspace status: dirty files, pending tests, last-known trybuild result, and publish readiness. Read-only; emits a StatusShowEvent. |
| `cargo cicd target show` | Reports the size and age profile of the local Cargo target directory without modifying it. |
| `cargo cicd target prune` | Removes stale build artefacts from the Cargo target directory according to configurable age/size policy. Emits a TargetPruneEvent recording bytes freed. |
| `cargo cicd test changed` | Runs cargo test restricted to crates whose source files have changed since the last green commit. Emits a TestChangedEvent with pass/fail counts and affected crate list. |
| `cargo cicd trybuild changed` | Runs trybuild type-law fixtures for changed crates, verifying that compile-fail fixtures fail for the correct named law and compile-pass fixtures succeed. Emits a TrybuildChangedEvent. |
| `cargo cicd git status` | Surfaces a structured summary of the git working-tree state: branch, ahead/behind counts, staged/unstaged/untracked file counts, and last-commit metadata. |
| `cargo cicd git close` | Performs the lawful branch-close sequence: ensures tests pass, commits any staged evidence, merges to the trunk branch, and emits a GitCloseEvent as a receipt. |
| `cargo cicd publish run` | Publishes eligible workspace crates to crates.io after verifying all release readiness conditions are met. Emits a PublishRunEvent that the wasm4pm oracle may audit post-release. |
| `cargo cicd workspace doctor` | Diagnoses the Cargo workspace for structural health: duplicate dependencies, missing feature declarations, version skew, and toolchain mismatch. Emits a WorkspaceDoctorEvent. |
| `cargo cicd certification show` | Prints an IEC 61508 / ISO 26262 / SOC2 / TOGAF compliance summary against registered certification bodies. |
| `cargo cicd sbom generate` | Generates a CycloneDX SBOM (`sbom.json`) from the workspace via `cargo-cyclonedx`. |
| `cargo cicd sbom show` | Displays the previously generated SBOM. |

<!-- END ggen:commands -->

### Compliance & Supply Chain

```sh
# Print IEC 61508 / ISO 26262 / SOC2 / TOGAF compliance summary
cargo cicd certification show

# Generate a CycloneDX SBOM (requires cargo-cyclonedx)
cargo cicd sbom generate

# Display the previously generated SBOM
cargo cicd sbom show
```

`cargo cicd sbom generate` degrades gracefully when `cargo-cyclonedx` is not
installed and tells you how to add it.

### Global Flags

| Flag | Description |
|------|-------------|
| `--help` | Print help for a command or noun |
| `--version` | Print the cargo-cicd version |
| `--cicd-toml <path>` | Use a different `cicd.toml` path (default: workspace root) |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Command failed (check stderr) |
| 2 | Workspace not found or invalid |
| 3 | Readiness check failed |

---

## Feature Flags

cargo-cicd ships a lean default binary. Optional capabilities are gated behind
Cargo feature flags.

| Feature | Default | Description |
|---------|---------|-------------|
| `process-data` | no | Emits structured XES event logs for each command |
| `autonomic` | no | Enables policy suggestions after each run (implies `process-data`) |
| `wasm4pm` | no | Integrates with the wasm4pm oracle for external adjudication (implies `process-data`) |
| `contrib` | no | Contributor-only diagnostics and internal tooling (implies `process-data`) |

Install with a feature:

```sh
cargo install cargo-cicd --features autonomic
```

Use as a dependency:

```toml
[dependencies]
cargo-cicd = { version = "26.6.19", features = ["process-data"] }
```

See [docs/reference/feature-flags.md](docs/reference/feature-flags.md) for full
details.

---

## Configuration

<!-- BEGIN custom:cicd-toml -->
cargo-cicd reads and writes `cicd.toml` in your workspace root. Add it to
`.gitignore` — it is a local state file, not a project artifact.

A minimal `cicd.toml`:

```toml
[target]
max_size_gb = 10.0
prune_after_days = 14

[test.changed]
base = "origin/main"

[autonomic]
enabled = false
mode = "suggest"
```

Full schema reference: [docs/reference/cicd-toml.md](docs/reference/cicd-toml.md)
<!-- END custom:cicd-toml -->

---

## CI/CD Integration

### GitHub Actions

```yaml
name: CI

on: [push, pull_request]

jobs:
  cicd:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-cicd
        run: cargo install cargo-cicd

      - name: Workspace doctor
        run: cargo cicd workspace doctor

      - name: Status check
        run: cargo cicd status

      - name: Run changed tests
        run: cargo cicd test changed

      - name: Check target directory
        run: cargo cicd target show
```

For release pipelines, add the wasm4pm oracle and enforce evidence gates:

```yaml
      - name: Release gate
        env:
          REQUIRE_WPM_ORACLE: "1"
        run: |
          cargo test --test wasm4pm_evidence_gate --features wasm4pm
          cargo test --test wasm4pm_evidence_mutation --features wasm4pm
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/INDEX.md](docs/INDEX.md) | Master documentation index — find anything |
| [docs/dx/ONBOARDING.md](docs/dx/ONBOARDING.md) | 30-minute onboarding guide for new contributors |
| [docs/dx/CHEATSHEET.md](docs/dx/CHEATSHEET.md) | Developer cheat sheet: commands, flags, env vars |
| [docs/dx/ECOSYSTEM_MAP.md](docs/dx/ECOSYSTEM_MAP.md) | Visual map of all modules and their interactions |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Deep-dive: EngineState, adapters, noun-verb grammar |
| [TESTING_GUIDE.md](TESTING_GUIDE.md) | Testing strategy, fixtures, evidence-gate patterns |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Debugging adapter failures, test issues, policy verdicts |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution workflow, commit format, PR process |
| [SKILLS_CATALOG.md](SKILLS_CATALOG.md) | Claude Code skills for automation and release |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow. The short version:

```sh
git clone https://github.com/seanchatmangpt/cargo-cicd
cd cargo-cicd
cargo build
cargo test
git checkout -b feat/your-feature
# make changes
git commit -m "feat(core): describe the change"
```

Commit format: `feat(core|cli|target|test|git|autonomic|docs|receipts): description`

---

## License

Licensed under either of:

- [MIT](LICENSE-MIT)
- [Apache-2.0](LICENSE-APACHE)

at your option.
