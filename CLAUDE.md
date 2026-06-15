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
Primary oracle command: wpm receipt doctor --format json --strict <receipt.json>
Secondary XES health check: wpm audit <file.xes>
Evidence format: XES (XML Event Stream), not JSONL
Evidence dir: target/cargo-cicd/evidence/

### Test Hierarchy

1. Unit/smoke/projection tests (non-closing):
   - May use assert_cmd/tempfile
   - May test CLI parsing, public boundaries, schemas
   - Files: tests/invariants.rs, tests/cli/, tests/feature_projection.rs, etc.

2. wasm4pm evidence-gate tests (closing — release gate):
   - Must emit process evidence as XES
   - Must invoke wpm oracle: `wpm audit <file.xes>`
   - Must invoke receipt doctor: `wpm receipt doctor --format json --strict` on emitted receipts
   - Must assert wasm4pm Accept/Refuse verdict from both oracle and receipt doctor
   - Files: tests/wasm4pm_evidence_gate.rs, tests/wasm4pm_evidence_mutation.rs, tests/wasm4pm_refusal_cases.rs

No release may claim ALIVE solely from cargo-cicd internal tests.

### Policies (`src/policies/`)
Autonomic policies run in `suggest` mode by default (configured in `cicd.toml [autonomic]`). They read `PolicyState` and emit recommendations, never take destructive action.

### Tests
Integration tests in `tests/` use `assert_cmd` + `tempfile` + fixture workspaces under `tests/fixtures/`. The `invariants` test enforces the 7 non-negotiable public boundary invariants. `feature_projection` verifies the feature flag surface contract.

---

## Claude Code Ecosystem

The `.claude/` directory wires up slash commands, subagents, skills, hooks, and settings so Claude Code can assist with cargo-cicd development tasks out of the box. A companion plugin bundle lives under `plugins/cargo-cicd-kit/`.

### Slash Commands (`.claude/commands/`)

| Command | Purpose |
|---|---|
| `release` | Run the full release checklist: bump version, run `cargo make check`, run all test suites (including wasm4pm evidence-gate), tag, and summarize what's left. |
| `audit-evidence` | Invoke `wpm audit` and `wpm receipt doctor --format json --strict` against every XES file in `target/cargo-cicd/evidence/`, then report Accept/Refuse verdicts. |
| `check-invariants` | Run `cargo test --test invariants` and surface any failing public-boundary contracts with file and line context. |
| `new-noun` | Scaffold a new noun module in `src/nouns/` following the clap-noun-verb grammar: creates the module file, registers it in `src/nouns/mod.rs`, and adds a default-verb entry in `main.rs::inject_default_verbs()`. |
| `ui-demo` | Run `cargo cicd ui demo` and capture/display terminal output so UI component changes can be previewed quickly. |
| `clean-target` | Remove stale build artifacts and prune `target/cargo-cicd/evidence/` of old XES files while preserving the latest receipt per command. |
| `phase-close` | Invoke `cargo cicd git close` to advance the git phase, then confirm cicd.toml `[state]` reflects the new phase. |

### Subagents (`.claude/agents/`)

| Agent | Purpose |
|---|---|
| `rust-reviewer` | Reviews Rust source changes in `src/` for correctness, safety, and adherence to the adapter/engine/noun architecture boundaries. Triggered when reviewing PRs or asking "is this safe?" |
| `invariant-guardian` | Specialises in the 7 public-boundary invariants. Runs `cargo test --test invariants` and maps failures back to the specific noun/verb output contract that broke. |
| `test-author` | Writes new integration tests under `tests/` using `assert_cmd` + `tempfile` patterns already established in the repo. |
| `ggen-regenerator` | Runs `ggen` after ontology or template changes and verifies the regenerated noun modules compile and pass `cargo make check`. |
| `evidence-gate-runner` | Orchestrates the wasm4pm evidence-gate: runs `cargo test --features wasm4pm`, collects XES from `target/cargo-cicd/evidence/`, invokes `wpm audit` and `wpm receipt doctor`, and reports verdicts. |
| `ui-polisher` | Reviews and improves terminal UI output in `src/ui/` — checks colour/glyph contracts, plain-mode cleanliness, and consistency with the design system. |
| `release-captain` | Drives the end-to-end release flow: version bump, full test run, evidence-gate closure, changelog entry, tag. Refuses to proceed if any wasm4pm verdict is Refuse. |

### Skills (`.claude/skills/`)

| Skill | Purpose |
|---|---|
| `release-checklist` | Step-by-step release procedure specific to cargo-cicd: version bump in `Cargo.toml`, `cargo make check`, invariants, evidence-gate, tag format `v<semver>`. |
| `evidence-audit` | How to collect XES evidence, run `wpm audit <file.xes>`, run `wpm receipt doctor --format json --strict <receipt.json>`, and interpret Accept/Refuse. |
| `noun-scaffold` | Concrete steps to add a new noun: create `src/nouns/<noun>.rs`, implement `NounCommand` + at least one `VerbCommand`, register in `src/nouns/mod.rs`, wire default verb in `main.rs`. |
| `ui-component` | How to add a new component to `src/ui/`: implement in a dedicated module, re-export from `src/ui/mod.rs`, ensure `Style::paint` for colour and `symbols::*` for glyphs, add a demo entry to `cargo cicd ui demo`. |
| `invariant-audit` | How to read `tests/invariants.rs`, map each invariant to its noun/verb output contract, and fix a failing assertion without breaking the public boundary. |

### Hooks (`.claude/hooks/`)

Hooks fire on Claude Code lifecycle events. The project registers hooks in `.claude/settings.json` (see below). A `SessionStart` hook checks that `cargo make check` is available and warns if `wpm` is not on PATH (evidence-gate tests will fail without it). A `PostToolUse` hook on `Write`/`Edit` targeting `src/**/*.rs` reminds the agent to run `cargo make check` before committing.

### Settings (`.claude/settings.json`)

Stores project-scoped permissions, environment variables, and hook registrations. Key allowed tools include `Bash(cargo make *)`, `Bash(cargo test *)`, `Bash(wpm *)`, and `Read`/`Grep`/`Glob` across the whole tree. The `CICD_EVIDENCE_DIR` env var is set to `target/cargo-cicd/evidence/` so scripts and agents resolve the evidence directory consistently.

### Plugin Bundle (`plugins/cargo-cicd-kit/`)

`plugins/cargo-cicd-kit/` is a self-contained Claude Code plugin that bundles the commands, agents, skills, and hooks above into a distributable unit. `plugins/cargo-cicd-kit/.claude-plugin/plugin.json` declares the bundle metadata. A root `.claude-plugin/marketplace.json` points to it. Installing the plugin in another workspace gives that workspace the full cargo-cicd assistant toolkit.

---

## Terminal UI Design System (`src/ui/`)

`src/ui/` is a zero-dependency terminal UI toolkit used by all noun output. It has no external crate dependencies — only `std`. Modules:

| Module | Role |
|---|---|
| `caps` | Detect terminal capabilities: colour support, Unicode support, TTY vs pipe. |
| `style` | `Style::paint(text, style)` — the single entry point for all coloured output. Auto-disables when stdout is not a TTY, so piped output is always plain. |
| `symbols` | Named glyph constants (`CHECK`, `CROSS`, `ARROW`, `BULLET`, etc.) with ASCII fallbacks selected at runtime via `caps`. All glyphs in noun output must come from this module. |
| `text` | String helpers including `display_width` (Unicode-aware column width for table alignment) and `truncate`. |
| `table` | Columnar table renderer. Column widths use `text::display_width`. |
| `panel` | Bordered content panel with an optional title. |
| `badge` | Inline status badge (e.g. `[PASS]`, `[FAIL]`, `[SKIP]`). |
| `progress` | Single-line progress indicator / spinner for long-running noun verbs. |
| `chart` | Horizontal bar chart for target-size or test-count summaries. |
| `tree` | Hierarchical tree renderer for workspace/package structures. |
| `theme` | Named colour palette (`Theme::default()`, `Theme::plain()`). Plain theme is selected automatically off-TTY. |
| `layout` | Composes panels/tables/trees into a full-screen layout for dashboard use. |
| `diagnostics` | Structured diagnostic message renderer (error/warn/info/hint with source location). |
| `dashboard` | Full-workspace status dashboard combining multiple components; invoked by `cargo cicd ui dashboard`. |

**Rules that must never be broken:**

1. All colour goes through `Style::paint` — never write raw ANSI escape codes directly in noun modules.
2. All glyphs go through `symbols::*` — never hard-code Unicode characters or box-drawing glyphs inline.
3. Column widths for tables/panels must use `text::display_width`, not `.len()`, to handle multi-byte characters correctly.
4. When stdout is not a TTY, output must be plain text with no escape codes and ASCII-only glyphs — this is enforced by `caps` and verified by tests that capture non-TTY output.

Run `cargo cicd ui demo` to render every component in isolation. Run `cargo cicd ui dashboard` for the full workspace overview. Both commands exercise the complete design-system surface.
