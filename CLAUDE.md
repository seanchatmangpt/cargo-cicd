# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Claude Code Configuration — cargo-cicd

## Mission
cargo-cicd is a manufactured Level 5 process-data engine exposed as a boring Rust CI/CD helper.

## Key Facts
- Public: "cargo-cicd keeps Rust workspaces clean, fast, and push-ready."
- Private: Level 5 process-data engine with cicd.toml carrier
- Manufacturing: ggen → clap-noun-verb → cargo-cicd
- Gate: Dung Gate (output/artifact manufacture)

## FORBIDDEN in public docs/CLI/help text
ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8

## Commit Format
feat(core|cli|target|test|git|autonomic|docs|receipts): description

---

## Build & Test Commands

```sh
# Build
cargo make build          # preferred — uses cargo-make
cargo build               # fallback if cargo-make unavailable

# Check (lint + type-check without building)
cargo make check

# Run all tests
cargo make test

# Run a single integration test by name
cargo test --test invariants
cargo test --test cli
cargo test --test cicd_toml_truth
cargo test --test autonomic_policies
cargo test --test changed_tests
cargo test --test git_phase_closure
cargo test --test feature_projection

# Run a specific test function
cargo test --test invariants test_function_name

# Run with a feature flag
cargo test --features process-data
cargo test --features autonomic
```

---

## Architecture

### Noun-Verb CLI Grammar
The CLI uses `clap-noun-verb` (local crate at `/Users/sac/clap-noun-verb`). Each noun is a module in `src/nouns/` implementing `NounCommand`. Verbs within each noun implement `VerbCommand`. Default verb injection happens in `main.rs::inject_default_verbs()` so bare nouns work (e.g. `cargo cicd status` → `status show`).

**Nouns:** `status`, `target`, `test`, `trybuild`, `git`, `publish`, `workspace`

### Level 5 Engine State (`src/engine/`)
`EngineState` is the aggregate root — a struct of all runtime dimensions:
- `WorkspaceState`, `ToolchainState`, `TargetState`
- `ChangedFileState`, `TestPlanState`, `TrybuildState`
- `GitPhaseState`, `ProcessEventState`, `ArtifactState`
- `PolicyState`, `ProjectionProfile`

Nouns read from `EngineState`; adapters populate it from external sources.

### Adapters (`src/adapters/`)
Each adapter owns one external source: `GitStatusAdapter`, `TargetScannerAdapter`, `ToolchainDetector`, `CargoMetadataAdapter`, `ChangedFileDetector`, `CicdTomlWriter`, `TrybuildDetector`. Adapters translate external representations into the internal state model — no business logic.

### cicd.toml
`cicd.toml` is the carrier/state file written to the workspace root. It stores workspace config (`[workspace]`, `[state]`, `[target]`, etc.) and emitted `[[events]]`. `CicdToml` in `src/cicd_toml.rs` owns its schema; `CicdTomlWriter` in adapters owns writes.

### ggen / Ontology Pipeline
`ggen.toml` + `ontology/cargo-cicd.ttl` + SPARQL queries in `queries/` + Tera templates in `templates/` are the manufacturing pipeline for generating noun modules and CLI test scaffolding. Run `ggen` to regenerate from ontology changes.

### Feature Flags
- `process-data` — enables Level 5 engine internals
- `autonomic` — implies `process-data`; enables policy/suggest mode
- `wasm4pm` — implies `process-data`; wasm4pm integration seam (richer runtime integration, not the evidence-gate acceptance law)
- `contrib` — implies `process-data`

### wasm4pm Evidence Gate

wasm4pm is not an optional future integration for acceptance testing.

For v26.6.2, cargo-cicd must emit process evidence and the evidence-gate tests must submit that evidence to the discovered current wpm oracle.

Internal smoke tests may pass, but release closure requires:

  cargo-cicd emits → wasm4pm adjudicates → tests assert wasm4pm verdict.

The wasm4pm feature flag gates richer runtime integration, not the evidence-gate acceptance law.

wpm binary: /Users/sac/wasm4pm/target/release/wpm
Primary oracle command: wpm audit <file.xes>
Evidence format: XES (XML Event Stream), not JSONL
Evidence dir: target/cargo-cicd/evidence/

### Test Hierarchy

1. Unit/smoke/projection tests (non-closing):
   - May use assert_cmd/tempfile
   - May test CLI parsing, public boundaries, schemas
   - Files: tests/invariants.rs, tests/cli/, tests/feature_projection.rs, etc.

2. wasm4pm evidence-gate tests (closing — release gate):
   - Must emit process evidence as XES
   - Must invoke wpm oracle: wpm audit <file.xes>
   - Must assert wasm4pm Accept/Refuse verdict
   - Files: tests/wasm4pm_evidence_gate.rs, tests/wasm4pm_evidence_mutation.rs, tests/wasm4pm_refusal_cases.rs

No release may claim ALIVE solely from cargo-cicd internal tests.

### Policies (`src/policies/`)
Autonomic policies run in `suggest` mode by default (configured in `cicd.toml [autonomic]`). They read `PolicyState` and emit recommendations, never take destructive action.

### Tests
Integration tests in `tests/` use `assert_cmd` + `tempfile` + fixture workspaces under `tests/fixtures/`. The `invariants` test enforces the 7 non-negotiable public boundary invariants. `feature_projection` verifies the feature flag surface contract.
