# cargo-cicd QUICKSTART (agent-optimized)

Source of truth: `CLAUDE.md`. Architecture diagrams: `.claude/ARCHITECTURE.md`. Pattern reference: `.claude/PATTERNS.md`.

---

## Prerequisites

| Tool | Install | Required? |
|------|---------|----------|
| Rust stable | `rustup update stable` | Always |
| Rust nightly | `rustup install nightly` | trybuild tests only |
| cargo-make | `cargo install cargo-make` | Canonical commands |
| wpm oracle | build from wasm4pm source repo | Evidence gate / release only |
| ggen | `cargo install ggen` | Ontology changes only |

---

## Bootstrap Sequence

Run in order. Stop on first failure.

```bash
cargo make build
./target/debug/cargo-cicd --version   # expect: cargo-cicd 26.6.2
cargo make check
./target/debug/cargo-cicd status
cargo test --test invariants
```

All 7 invariants must pass on clean checkout. Any failure = upstream breakage.

---

## Architecture

### Manufacturing Pipeline

```
ontology/cargo-cicd-capabilities.ttl  →  ggen  →  src/nouns/*.rs
                                                →  tests/cli/
                                                →  README.md
                                                →  docs/reference/commands/
```

DO NOT hand-edit generated files. DO NOT add a noun without updating the ontology first.

### Default Verb Injection (`src/main.rs::inject_default_verbs()`)

| Bare noun | Resolves to |
|-----------|-------------|
| `status` | `status show` |
| `workspace` | `workspace doctor` |
| `evidence` | `evidence doctor` |
| `publish` | `publish run` |

### Noun Registry

| Noun | Default verb | Category |
|------|-------------|----------|
| `status` | `show` | read-only |
| `git` | — | execution |
| `test` | — | execution |
| `trybuild` | — | execution |
| `target` | — | dry-run / execution |
| `workspace` | `doctor` | read-only |
| `publish` | `run` | execution |
| `evidence` | `doctor` | adjudication |
| `pipeline` | — | execution |
| `lsp` | — | read-only |
| `analyze` | — | read-only |
| `autoarch` | — | read-only |
| `certification` | `show` | read-only |
| `sbom` | — | execution |
| `ui` | — | read-only |

Verb categories: read-only (`show`, `status`, `explain`, `doctor`) · dry-run (`prune --dry-run`) · execution (`run`, `close`) · adjudication (`audit`).

### EngineState Fields and Adapters

| Field | Adapter | External source |
|-------|---------|----------------|
| `workspace` | `CargoMetadataAdapter` | `Cargo.toml` line scan |
| `toolchain` | `ToolchainDetector` | `rustc --version` |
| `target` | `TargetScannerAdapter` | recursive walkdir |
| `changed_files` | `ChangedFileDetector` | `git diff origin/main --name-only` |
| `git_phase` | `GitStatusAdapter` | `git status --porcelain` |
| `trybuild` | `TrybuildDetector` | `tests/ui/` scan |
| `test_plan` | derived | `changed_files` |
| `process_events` | runtime | verbs populate |
| `artifacts` | runtime | verbs populate |
| `policies` | runtime | policy runner |
| `projection` | compile | feature flags |

Adapter contract: never panic, never call other adapters, silently return defaults on failure.

`EngineState::from_workspace()` calls all adapters in sequence; swallows all errors.

### Evidence Emission (every verb that performs work)

```rust
// Canonical pattern — use wasm4pm-compat, never hand-roll OCEL structs
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence};
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;

// 1. Build OCEL
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Adjudicate (shell-out only)
// wpm audit <file.ocel.json>  →  Accept | Refuse | Blocked
```

**Cargo.toml dependency:**
```toml
wasm4pm-compat = { path = "/Users/sac/wasm4pm-compat", features = ["formats", "strict"] }
```

FORBIDDEN: hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs.
FORBIDDEN: calling `wpm` on `.xes` files in new code.
FORBIDDEN: adjudicating inside cargo-cicd (invariant E1).
DELETE: `src/ocel.rs` — replace all imports with `wasm4pm_compat::ocel::*`.
LEGACY: `evidence_xes_v2.rs` — do not extend; OCEL is the only format.

**Evidence invariants:**
- E1: cargo-cicd never adjudicates itself; only `wpm` issues verdicts
- E2: XES file must exist before `audit_xes()` is called
- E3: oracle unavailable + non-Blocked expectation = panic
- E4: tests assert wpm verdict only, never internal state
- E5: XES groups by `case_id` into `<trace>` elements
- E6: JSONL mirrors XES
- E7: `Blocked` is a first-class expectation, not an error

Object types in cargo-cicd domain: `Workspace`, `Crate`, `TestRun`, `GitCommit`, `Release`, `Receipt`, `EvidenceFile`, `Policy`, `Toolchain`.

**OCEL 2.0 JSON shape:**
```json
{ "eventTypes": [...], "objectTypes": [...], "events": [...], "objects": [...] }
```
`OCELEvent.relationships`: `Vec<OCELRelationship { objectId, qualifier }>`

---

## Common Tasks

### Add a verb to an existing noun

```bash
# 1. Edit ontology
vim ontology/cargo-cicd-capabilities.ttl
# 2. Regenerate
ggen
# 3. Implement handler in src/nouns/<noun>.rs
# 4. Emit evidence (mandatory for all mutation verbs)
# 5. Write tests (smoke + --confirm guard)
cargo test --test invariants   # must still pass
```

Verb handler skeleton:
```rust
pub struct RepairVerb;
impl VerbCommand for RepairVerb {
    fn run() -> Result<()> {
        let state = EngineState::from_workspace();
        // ... work ...
        Ok(())
    }
}
```

### Fix a failing test

```bash
cargo test --test invariants -- --nocapture
cargo test --test invariants <test_fn_name> -- --nocapture
RUST_LOG=debug cargo run -- status show 2>&1 | head -40   # adapter debug
```

Failure → fix → regression test → `cargo make test`.

### Run the evidence gate

```bash
# Offline (wpm absent → Blocked verdict, not failure)
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases

# Online (wpm on PATH)
which wpm && wpm --version
cargo test --test wasm4pm_evidence_gate -- --nocapture
wpm audit target/cargo-cicd/evidence/evt-*.xes
```

Oracle returns `Refuse`: check for missing `case_id`, missing `complete` lifecycle event, or empty `verdict_claimed`.

### Debug a failing adapter

```bash
RUST_LOG=debug cargo run -- status show 2>&1 | grep -i "adapter\|error\|failed"
# Test underlying commands directly:
git status --porcelain
git diff origin/main --name-only
rustc --version
```

Adapter → state dimension mapping: `git_status.rs`→`GitPhaseState`, `cargo_metadata.rs`→`WorkspaceState`, `toolchain_detector.rs`→`ToolchainState`, `target_scanner.rs`→`TargetState`, `changed_file_detector.rs`→`ChangedFileState`.

### Add an autonomic policy

```rust
// src/policies/<name>.rs
#[cfg(feature = "autonomic")]
pub fn eval(state: &EngineState) -> PolicyEntry { /* suggest only, never mutate */ }
```

```rust
// src/policies/mod.rs  — add: pub mod <name>;
// src/autonomic/policies.rs — add: crate::policies::<name>::eval(state),
```

```bash
cargo build --features autonomic
cargo test --features autonomic --test autonomic_policies
```

### Write an integration test

```rust
use tempfile::TempDir;
use assert_cmd::Command;

#[test]
fn test_<noun>_<verb>() {
    let dir = TempDir::new().unwrap();
    // write minimal Cargo.toml + src/lib.rs to dir
    let output = Command::cargo_bin("cargo-cicd").unwrap()
        .current_dir(dir.path())
        .args(["<noun>", "<verb>"])
        .output().unwrap();
    assert!(output.status.success());
    // assert on stdout substrings or wpm verdict — never internal state
}
```

---

## Forbidden Terms

Banned from ALL public output (help text, status messages, errors). One occurrence = release block.

`ALIVE` · `Inspection Gate` · `wall` · `Nehemiah` · `Field8` · `Instinct8` · `Cargo Court` · `AGI` · `Truex` · `CONSTRUCT8`

```bash
# Diagnose leak
for noun in status git test trybuild target workspace publish evidence pipeline lsp analyze autoarch certification sbom ui; do
  cargo run -- $noun --help 2>&1 | grep -E "ALIVE|Inspection Gate|Nehemiah|Field8|Instinct8|Cargo Court|AGI|Truex|CONSTRUCT8"
done
rg "ALIVE|Nehemiah|Field8" src/
# Fix → re-run:
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
```

---

## Commit Format

`<type>(<scope>): <description>`

Types: `feat` `fix` `test` `docs` `chore` `refactor`

| Scope | Files |
|-------|-------|
| `core` | `src/engine/`, `src/evidence.rs`, `src/session.rs` |
| `cli` | `src/nouns/`, `src/main.rs` |
| `target` | target adapters / noun |
| `test` | test files |
| `git` | git adapter / noun |
| `autonomic` | `src/autonomic/`, `src/policies/` |
| `docs` | `CLAUDE.md`, `.claude/`, `docs/` |
| `receipts` | `receipts/`, oracle integration |

Scope is mandatory.

---

## Key Files

### Entry Points

| File | Role |
|------|------|
| `src/main.rs` | Binary entry; `inject_default_verbs()`; noun dispatch |
| `src/lib.rs` | Public API re-exports |
| `Cargo.toml` | Feature flags, workspace members, `[[bin]]` |
| `Makefile.toml` | cargo-make task definitions |

### Noun Modules (`src/nouns/`)

One file per noun. `src/nouns/mod.rs` = noun registry (clap wiring).

### Engine State (`src/engine/`)

`mod.rs` (struct + constructor) · `workspace_state.rs` · `toolchain_state.rs` · `target_state.rs` · `changed_file_state.rs` · `git_phase_state.rs` · `process_event_state.rs` · `policy_state.rs` · `trybuild_state.rs` · `test_plan_state.rs` · `projection_profile.rs`

### Adapters (`src/adapters/`)

`cargo_metadata.rs` · `manifest_parser.rs` · `git_status.rs` · `toolchain_detector.rs` · `target_scanner.rs` · `changed_file_detector.rs` · `trybuild_detector.rs` · `cicd_toml_writer.rs`

### Evidence & Oracle

| File | Role |
|------|------|
| `src/evidence.rs` | `ProcessEvent`, XES serialization, invariants E1–E7 |
| `src/integrations/wasm4pm_shell.rs` | Shell-out to `wpm audit` and `wpm receipt doctor` |
| `src/integrations/wasm4pm_current.rs` | Oracle state, XES format |
| `src/session.rs` | `read_or_create_session_id()` |

### Policies (`src/policies/`)

`git_phase_dirty` · `target_pressure` · `toolchain_mismatch` · `trybuild_changed` · `branch_behind` · `evidence_stale` · `publish_not_adjudicated` · `mod.rs` (registry)

### Tests

| File | Validates |
|------|-----------|
| `tests/invariants.rs` | 7 public boundary invariants |
| `tests/cli/` | Noun/verb CLI parsing and output |
| `tests/feature_projection.rs` | Feature flag surface contract |
| `tests/cicd_toml_truth.rs` | cicd.toml round-trip |
| `tests/autonomic_policies.rs` | Policy evaluation |
| `tests/wasm4pm_evidence_gate.rs` | Happy-path → `Accept` |
| `tests/wasm4pm_evidence_mutation.rs` | Corrupt evidence → `Refuse` |
| `tests/wasm4pm_refusal_cases.rs` | Oracle unavailable, malformed OCEL |

### Ontology & Generation

| File | Role |
|------|------|
| `ontology/cargo-cicd-capabilities.ttl` | Ground truth for noun/verb grammar |
| `ggen.toml` | Code gen config |
| `queries/*.sparql` | SPARQL inference rules |
| `templates/` | Tera templates for README and docs |

Generated (DO NOT EDIT): `README.md` command reference sections · `docs/reference/commands/*.md`

### Workspace Artifacts

| File | Role |
|------|------|
| `cicd.toml` | Persistent state carrier; written by `CicdTomlWriter`; not committed |
| `target/cargo-cicd/evidence/` | OCEL `.json` and JSONL evidence files |
| `receipts/` | wasm4pm receipt artifacts |

---

## Feature Flags

| Flag | Implies | Effect |
|------|---------|--------|
| `process-data` | — | Level 5 engine, adapters, cicd.toml |
| `autonomic` | process-data | Policy suggestions (suggest mode only) |
| `wasm4pm` | process-data | wpm oracle integration |
| `affidavit` | process-data | `affi` receipt engine, `affidavit` noun |
| `advanced` | — | parallel_scan, blake3, tracing, miette, moka, bitcode, petgraph, jiff, hdrhistogram, aho-corasick |

---

## Slash Commands

| Command | Source | Action |
|---------|--------|--------|
| `/build` | `.claude/commands/build.md` | `cargo make build`, verify binary |
| `/test` | `.claude/commands/test.md` | Tier 1 then Tier 2 test suites |
| `/git` | `.claude/commands/git.md` | Git phase check and closure |
| `/release` | `.claude/commands/release.md` | 12-step release gate |
| `/check` | `.claude/commands/check.md` | `cargo make check` (clippy + types) |
| `/evidence` | `.claude/commands/evidence.md` | Evidence gate workflow |
| `/status` | `.claude/commands/status.md` | Workspace health snapshot |
| `/workspace` | `.claude/commands/workspace.md` | `workspace doctor` diagnostics |
| `/audit-evidence` | `.claude/commands/audit-evidence.md` | Evidence doctor + status audit |
| `/check-invariants` | `.claude/commands/check-invariants.md` | Public boundary invariant suite |
| `/clean-target` | `.claude/commands/clean-target.md` | `target show` + `target prune` preview |
| `/new-noun` | `.claude/commands/new-noun.md` | Scaffold new CLI noun |
| `/phase-close` | `.claude/commands/phase-close.md` | `git close` with safety check |
| `/ui-demo` | `.claude/commands/ui-demo.md` | UI demo + design system walkthrough |

### Hooks (`.claude/settings.json`)

| Hook | When | Script |
|------|------|--------|
| `SessionStart` | Session begins | `.claude/hooks/session-start.sh` |
| `PreToolUse` | Before each Bash call | `.claude/hooks/pre-tool-use.sh` |
| `PostToolUse` | After each Bash call | `.claude/hooks/post-tool-use.sh` |

---

## Common Errors

| Symptom | Cause | Fix |
|---------|-------|-----|
| Invariant fails forbidden term | Leaked internal term in help text | `rg "<term>" src/` → fix description |
| Evidence gate `Refuse` | Missing `case_id` or `complete` event | Check `src/evidence.rs` emission pattern |
| Adapter returns empty state | Underlying command unavailable | `RUST_LOG=debug cargo run -- status show` |
| `ggen` overwrites hand-edited noun | Noun not in ontology | Update TTL first, then regenerate |
| Test asserts internal state | Violates invariant E4 | Assert on `wpm_verdict` only |
| `ocel.rs` struct conflicts | Shadow of wasm4pm-compat | Delete `src/ocel.rs`, use `wasm4pm_compat::ocel::*` |

---

## Quick Reference

```bash
cargo make build
cargo make check
cargo make test
cargo test --test invariants
cargo test --test cli
cargo test --features autonomic
cargo test --features wasm4pm
cargo test --test wasm4pm_evidence_gate -- --nocapture
RUST_LOG=debug cargo run -- status show
wpm audit target/cargo-cicd/evidence/evt-*.xes
wpm receipt doctor --format json --strict receipts/*.json
ggen
cargo build --features autonomic,wasm4pm,contrib
```

---

**Version:** cargo-cicd 26.6.2  
**Last Updated:** 2026-06-21
