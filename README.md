# cargo-cicd

`cargo-cicd` checks your Rust workspace's health, runs only the tests affected by what you changed, and gates publishing behind a passing evidence trail — a local pre-flight checklist that runs before you push or publish.

[![Crates.io](https://img.shields.io/crates/v/cargo-cicd.svg)](https://crates.io/crates/cargo-cicd)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-orange.svg)](https://www.rust-lang.org)

Under the hood it reads your workspace's `Cargo.toml` and git state, records what it finds in a `cicd.toml` snapshot, and — with optional features enabled — emits structured process evidence (XES/OCEL) that an external oracle can adjudicate before a release is allowed to publish.

## Install

```sh
cargo install cargo-cicd
```

Verify:

```sh
cargo cicd --version
# cargo-cicd 26.6.30
```

If the binary is not found, ensure `~/.cargo/bin` is on your `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

## Quickstart

```sh
# 1. Diagnose the workspace: Cargo.toml, toolchain, git repo, cicd.toml
cargo cicd workspace doctor

# 2. See a snapshot of dirty files, target size, and branch state
cargo cicd status show

# 3. Run only the tests for crates whose source changed since the last green commit
cargo cicd test changed

# 4. Check how much space the target/ directory is using
cargo cicd target show

# 5. Stage, commit, and close out the current git phase
cargo cicd git close
```

Each of these was run against this repository while writing this README and produces real output — for example, `cargo cicd status show` prints a toolchain line, a target-usage bar, branch name, dirty/untracked file counts, and a `git [PASS/FAIL]` verdict, followed by autonomic policy suggestions such as `run 'cargo cicd evidence doctor' before publish`.

## Command overview

`cargo-cicd` uses a noun-verb CLI (`cargo cicd <noun> <verb>`). Every noun accepts `--help` for full detail. This table reflects the commands available in a default build (`cargo build`, no extra features):

| Noun | Verbs | What it does |
|------|-------|---------------|
| `status` | `show`, `audit` | Displays workspace status: dirty files, pending tests, trybuild result, and publish readiness. |
| `workspace` | `doctor`, `validate`, `list`, `sync` | Diagnoses the Cargo workspace for structural health: duplicate dependencies, missing feature declarations, version skew, toolchain mismatch. |
| `target` | `show`, `prune` | Reports on and prunes stale build artefacts in the Cargo target directory. |
| `test` | `changed`, `run`, `bench` | Runs `cargo test` restricted to crates whose source files changed since the last green commit. |
| `trybuild` | `changed`, `full` | Runs trybuild compile-fail/compile-pass fixtures for crates changed since the last green commit. |
| `git` | `status`, `fetch`, `stage`, `commit`, `diff`, `close` | Surfaces git working-tree status and performs the lawful branch-close sequence. |
| `publish` | `run`, `check`, `validate` | Publishes eligible workspace crates to crates.io after verifying release readiness. |
| `evidence` | `doctor`, `list`, `audit`, `show`, `reset` | Adjudicates and inspects recorded process evidence (XES/OCEL logs). |
| `doctor` | — | Diagnoses repository health against a recorded baseline and reports evidence drift. |
| `pipeline` | — | Runs, checks the status of, and validates the workspace's CI/CD pipeline definition. |
| `sbom` | `generate`, `show` | Generates and shows a Software Bill of Materials (SBOM) via CycloneDX. |
| `hooks` | — | Installs git hooks that integrate cargo-cicd with an external CI provider. |
| `verify` | — | Verifies a repository against configured checks, including semver compatibility. |
| `certification` | `show` | IEC 61508 / ISO 26262 compliance summary for cargo-cicd certification. |

Some additional nouns (`ocel`, `receipt`, `trace`, `standing`, `gate`, `release_gate`, `claude_context`) ship in this build for evidence replay, receipt auditing, and standing-document generation; run `cargo cicd --help` for the full, current list — the surface grows as the ontology-driven pipeline generates new commands.

## Documentation

- **[Tutorials](docs/tutorials/)** — learn by doing, start with [`quick-start.md`](docs/tutorials/quick-start.md)
- **[How-to guides](docs/how-to/)** — recipes for specific tasks (CI/CD pipelines, git hooks, IDE integration, custom ontologies)
- **[Reference](docs/reference/)** — command reference, `cicd.toml` schema, feature flags, evidence/XES format
- **[Explanation](docs/explanation/)** — the reasoning behind local-first CI/CD, changed-test planning, evidence emission, and autonomic policies
- **[Full index](docs/INDEX.md)** — a Diátaxis-organized map of everything above

## Feature flags

The default build (no extra features) provides the full public CLI surface shown above, with no process evidence emission. Optional features add capability:

| Feature | Implies | Effect |
|---------|---------|--------|
| `process-data` | — | Enables the Level 5 process-data engine: adapters, `cicd.toml` state, XES/OCEL evidence emission |
| `autonomic` | `process-data` | Adds autonomic policy suggestions (suggest-mode only, never destructive) |
| `wasm4pm` | `process-data` | Integrates the external `wpm` oracle to adjudicate recorded evidence |
| `affidavit` | `process-data` | Adds cryptographic provenance receipts via the `affi` CLI and the `affidavit` noun |
| `autoarch` | `autonomic` | Autonomous architecture-enforcement layer |
| `contrib` | `process-data` | Contributor workflow extensions |
| `lsp` | — | LSP integration for the `explain` verb |
| `anti-llm-cheat` | — | Anti-cheat enforcement via `lsp-max-anti-cheat` |
| `advanced` | `process-data` | Parallel scanning, blake3, tracing, miette, moka, bitcode, petgraph, jiff, hdrhistogram, aho-corasick |

See [`docs/reference/feature-flags.md`](docs/reference/feature-flags.md) for full detail on each flag.

## Roadmap and research

`docs/vision/` holds forward-looking, aspirational, and research material — RFCs, phase plans, and a 2030 roadmap. None of it describes currently shipped behavior. Read it if you're curious where the project is headed; skip it if you just want to use the tool today.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
