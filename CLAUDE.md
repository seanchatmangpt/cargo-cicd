# CLAUDE.md

cargo-cicd keeps Rust workspaces clean, fast, and push-ready.  
Private: Level 5 process-data engine; manufactures noun-verb CLI from an ontology; emits XES evidence for wasm4pm adjudication.

Pipeline: `ontology/cargo-cicd-capabilities.ttl` → `ggen` → `src/nouns/*.rs` + `tests/cli/*` + `docs/`  
State carrier: `cicd.toml` | Oracle: `wpm` binary | Version: 26.6.19

---

## FORBIDDEN Terms (public output, help text, docs)

| Term | Term | Term |
|------|------|------|
| `ALIVE` | `Inspection Gate` | `wall` |
| `Nehemiah` | `Field8` | `Instinct8` |
| `Cargo Court` | `AGI` | `Truex` |
| `CONSTRUCT8` | | |

Enforced by: `cargo test --test invariants` (`invariant_public_boundary_no_forbidden_terms_in_all_help`).

> **Note:** In `.claude/hooks/public-boundary-guard.sh`, `wall` is matched as a whole-word pattern (`\bwall\b`) rather than a substring, to avoid false positives on common English words. All other terms are matched as plain substrings. The invariant test does not make this distinction.

---

## Build & Test

```sh
cargo make build          # build
cargo make check          # lint + type-check
cargo make test           # all tests

cargo test --test invariants
cargo test --test cli
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases

cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm
```

## Commit Format

`feat|fix|docs|test|chore(core|cli|target|test|git|autonomic|docs|receipts): description`

---

## Architecture

### Manufacturing Pipeline
Edit TTL → run `ggen` → implement handlers in `src/nouns/`. Never hand-edit generated files.

Default verb injection (`src/main.rs::inject_default_verbs()`):
- `status` → `status show` | `publish` → `publish run`
- `workspace` → `workspace doctor` | `evidence` → `evidence doctor`

### Nouns (`src/nouns/`)
`evidence` (doctor, audit) · `pipeline` (run) · `status` (show) · `target` (show, prune) · `test` (changed) · `trybuild` (changed, full) · `git` (status, close, phase) · `publish` (run) · `workspace` (doctor) · `lsp` (explain) · `affidavit` (seal, verify) [feature-gated] · `analyze` (dep-order) [feature=lsp or anti-llm-cheat] · `autoarch` (tune) [feature=autoarch] · `certification` (show) · `sbom` (generate, show) · `ui` (demo, dashboard)

Verb categories: read-only (show, status, explain, doctor) · dry-run (prune --dry-run) · execution (run, close) · adjudication (audit)

### EngineState (`src/engine/`)
Aggregate root. Nouns **read** it; adapters **populate** it. `EngineState::from_workspace()` calls all adapters in sequence, silently handling failures.

Fields: `workspace` · `toolchain` · `target` · `changed_files` · `test_plan` · `trybuild` · `git_phase` · `process_events` · `artifacts` · `policies` · `projection`

### Adapters (`src/adapters/`)
Stateless, pure translators. Never call each other. Silently fail (return defaults). Key adapters:
- `CargoMetadataAdapter` — workspace name/members (fast, no cargo invocation)
- `GitStatusAdapter` — `git status --porcelain`, ahead/behind
- `TargetScannerAdapter` — recursive walkdir (slow; cache result in cicd.toml)
- `ChangedFileDetector` — `git diff origin/main --name-only`
- `CicdTomlWriter` — serializes EngineState → cicd.toml

### Evidence Emission (`src/evidence.rs`)
Pattern: `start` event → [work] → `complete` event → optional wpm adjudication.

Invariants:
- **E1** cargo-cicd never adjudicates itself; only wpm issues verdicts
- **E2** XES file must exist before `audit_xes()` is called
- **E3** Oracle unavailable + non-Blocked expectation = panic
- **E4** Tests assert wpm verdict only, never internal state
- **E5** XES groups by `case_id` into `<trace>` elements
- **E6** JSONL mirrors XES
- **E7** `Blocked` is a first-class expectation, not an error

Verdicts: `PASS` · `WARN` · `FAIL` · `WARN:dry_run` · `WARN:oracle_unavailable`  
wpm: `wpm audit <file.xes>` → Accept/Refuse/Blocked · `wpm receipt doctor --format json --strict <receipt.json>`

### Feature Flags

| Flag | Implies | Effect |
|------|---------|--------|
| `process-data` | — | Level 5 engine, adapters, cicd.toml |
| `autonomic` | process-data | Policy suggestions (suggest mode only, never destructive) |
| `wasm4pm` | process-data | wpm oracle integration |
| `affidavit` | process-data | `affi` receipt engine, `affidavit` noun |
| `autoarch` | autonomic | Autonomous architecture enforcement layer |
| `contrib` | process-data | Contributor workflow extensions |
| `lsp` | — | LSP integration for explain verb |
| `anti-llm-cheat` | dep:lsp-max-anti-cheat | Anti-cheat enforcement via lsp-max-anti-cheat |
| `advanced` | process-data | parallel_scan, blake3, tracing, miette, moka, bitcode, petgraph, jiff, hdrhistogram, aho-corasick |

### Autonomic Policies (`src/autonomic/`, `src/policies/`)
All policies are suggest-mode (read-only). Verdicts: Pass · Warn · Skip.  
Policies: `target_pressure` · `toolchain_mismatch` · `trybuild_changed` · `branch_behind` · `evidence_stale` · `publish_not_adjudicated` · `git_phase_dirty`

Add a policy: create `src/policies/<name>.rs` with `fn eval(&EngineState) -> PolicyEntry`, register in `policies::run_all_policies()`, test in `tests/autonomic_policies.rs`.

### Terminal UI (`src/ui/`)
Zero-dependency (std only). Rules: all colour via `Style::paint`, all glyphs via `symbols::*`, widths via `text::display_width`, plain output when not TTY.  
`cargo cicd ui demo` · `cargo cicd ui dashboard`

---

## Test Tiers

**Tier 1 — Unit/Smoke** (`assert_cmd` + `tempfile`): invariants · cli · feature_projection · cicd_toml_truth · autonomic_policies · changed_tests · git_phase_closure

7 invariants: no forbidden terms · no destructive action without --confirm · no full trybuild by default · lowercase noun names · binary is `cargo-cicd` · status exits 0 · git close has safety warnings

**Tier 2 — Evidence Gate** (requires `wpm`): wasm4pm_evidence_gate · wasm4pm_evidence_mutation · wasm4pm_refusal_cases  
Assert on wpm verdict (`Accept`/`Refuse`/`Blocked`), never on internal state. Use `ExpectedWpmVerdict::Blocked` when wpm unavailable.

---

## Release Checklist

```sh
cargo make test
cargo test --test invariants
cargo build --features autonomic,wasm4pm
cargo test --test wasm4pm_evidence_gate
wpm receipt doctor --format json --strict receipts/*.json
# ensure ggen run, CHANGELOG updated, version bumped, git clean
git tag -a v<VERSION> -m "Release v<VERSION>"
git push origin main --tags
```

---

## Claude Code Ecosystem (`.claude/`)

Skills: `release-checklist` · `evidence-audit` · `noun-scaffold` · `ui-component` · `invariant-audit`  
Agents: `rust-reviewer` · `invariant-guardian` · `test-author` · `ggen-regenerator` · `evidence-gate-runner`  
Commands: `release` · `audit-evidence` · `check-invariants` · `new-noun` · `ui-demo` · `clean-target` · `phase-close`  
Plugin bundle: `plugins/cargo-cicd-kit/` — distributable toolkit for other workspaces.

**Last Updated:** 2026-06-21
