# cargo-cicd Quick Start

**cargo-cicd keeps Rust workspaces clean, fast, and push-ready.** It provides a noun-verb CLI for workspace health, test selection, git phase tracking, artifact publishing, and process evidence emission — all from a single binary.

---

## Prerequisites

- Rust stable (`rustup update stable`)
- `cargo-make` (`cargo install cargo-make`)
- `git` 2.30+
- Optional: `wpm` binary (wasm4pm oracle, required only for evidence gate in release)

---

## 4-Step Setup

```bash
# 1. Verify the workspace builds and passes lint
cargo make check

# 2. Run all tests
cargo make test

# 3. Workspace snapshot (should exit 0)
cargo cicd status

# 4. Full health check (autonomic policies)
cargo cicd workspace doctor
```

---

## Common Daily Tasks

| Task | Command |
|------|---------|
| Workspace health | `cargo cicd workspace doctor` |
| Git state snapshot | `cargo cicd git status` |
| Run only changed tests | `cargo cicd test changed` |
| Target dir size (dry-run clean) | `cargo cicd target prune --dry-run` |
| Publish readiness check | `cargo cicd publish run` |
| Evidence gate | `cargo cicd evidence doctor` |
| List workspace members | `cargo cicd workspace list` |
| Validate Cargo.toml | `cargo cicd workspace validate` |
| Compiler error snapshots (changed) | `cargo cicd trybuild changed` |

---

## Key Concepts

### Noun-Verb CLI Grammar
Every command is `cargo cicd <noun> <verb>`. Bare nouns inject default verbs automatically:
- `cargo cicd status` → `cargo cicd status show`
- `cargo cicd workspace` → `cargo cicd workspace doctor`
- `cargo cicd evidence` → `cargo cicd evidence doctor`
- `cargo cicd publish` → `cargo cicd publish run`

Available nouns: `status`, `git`, `test`, `trybuild`, `target`, `publish`, `workspace`, `evidence`, `lsp`, `pipeline`

### cicd.toml — The State Carrier
`cicd.toml` at workspace root is auto-generated after each major operation. It records workspace dimensions, git state, and emitted process events. **Do not hand-edit it** — the next verb run overwrites it.

### Evidence Emission
Every verb emits a `ProcessEvent` pair (start + complete) in XES format to `target/cargo-cicd/evidence/`. The `wpm` oracle adjudicates these events and returns `Accept`, `Refuse`, or `Blocked` (offline). Verdicts are stored in `receipts/`.

### Autonomic Policies
Seven workspace policies run in **suggest mode** (read-only). They detect: dirty git state, target pressure, toolchain mismatch, changed trybuild fixtures, branch drift, stale evidence, un-adjudicated publish. All recommendations require user action — no auto-remediation.

### Feature Flags
| Flag | Purpose |
|------|---------|
| `process-data` | Enable EngineState + adapters |
| `autonomic` | Enable policy suggestions (implies `process-data`) |
| `wasm4pm` | Enable oracle adjudication (implies `process-data`) |

Default build has no feature flags enabled. Use `--features autonomic` for policy output.

---

## Quick Troubleshooting

| Symptom | Fix |
|---------|-----|
| `No Cargo.toml found` | Run from workspace root where `Cargo.toml` lives |
| `Oracle unavailable` | `wpm` not on PATH; evidence gate needs `wpm` binary |
| Tests run full suite | Use `cargo cicd test changed` to scope to changed files |
| `cicd.toml` is stale | Run `cargo cicd workspace doctor` to refresh |
| Status exits non-zero | Run `RUST_LOG=debug cargo cicd status show` to diagnose |

---

## Next Steps

- Architecture patterns: `.claude/ARCHITECTURE.md`
- Full design patterns: `.claude/PATTERNS.md`
- Testing strategy: `.claude/TESTING.md`
- Complete reference: `CLAUDE.md`
