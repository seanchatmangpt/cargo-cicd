# cargo-cicd Quick Start

cargo-cicd keeps Rust workspaces clean, fast, and push-ready. It provides a noun-verb CLI for common CI/CD tasks: checking workspace health, running changed tests, managing git phase state, and emitting process evidence for external adjudication.

---

## Prerequisites

- Rust stable toolchain (`rustup show`)
- `cargo-make` (`cargo install cargo-make`)
- `git` (workspace must be a git repo)
- Optional: `wpm` binary on PATH for evidence gate tests

---

## 4-Step Setup

```bash
# 1. Build and check for errors
cargo make check

# 2. Run all tests
cargo make test

# 3. Snapshot workspace health
cargo cicd status

# 4. Full diagnostic
cargo cicd workspace doctor
```

All four should exit 0 before you start working.

---

## Common Tasks

| Task | Command |
|------|---------|
| Workspace health | `cargo cicd workspace doctor` |
| Git state | `cargo cicd git status` |
| Run changed tests | `cargo cicd test changed` |
| Dry-run target cleanup | `cargo cicd target prune --dry-run` |
| Publish readiness | `cargo cicd publish run` |
| Evidence check | `cargo cicd evidence doctor` |
| Commit staged files | `cargo cicd git commit` |
| Validate Cargo.toml | `cargo cicd workspace validate` |
| List workspace members | `cargo cicd workspace list` |

---

## Key Concepts

### Noun-Verb CLI
Every command is structured as `cargo cicd <noun> <verb>`. A few nouns have default verbs so the bare noun works too:
- `cargo cicd status` → `cargo cicd status show`
- `cargo cicd workspace` → `cargo cicd workspace doctor`
- `cargo cicd evidence` → `cargo cicd evidence doctor`
- `cargo cicd publish` → `cargo cicd publish run`

### cicd.toml
`cicd.toml` at the workspace root is the state carrier. It is **auto-generated** by cargo-cicd — do not edit manually, as it will be overwritten. It persists workspace metadata, git state, and emitted process events.

### Evidence Emission
Every verb that does work emits a pair of process events:
1. `ProcessEvent::started("noun:verb")` — at the start
2. `ProcessEvent::completed("noun:verb", elapsed, verdict)` — on completion

Both events share a `case_id` (session identifier) and are written to `target/cargo-cicd/evidence/` as XES and JSONL files. The `cargo cicd evidence doctor` verb submits this evidence to the wasm4pm oracle for external adjudication.

### Autonomic Policies
`cargo cicd workspace doctor` runs 7 read-only workspace checks (policies). Each policy produces a `[PASS]`, `[WARN]`, or `[SUGGEST]` verdict with a human-readable recommendation. Policies **never** take destructive action — they only suggest.

### Feature Flags
```bash
# Default build (minimal)
cargo build

# Enable autonomic policies
cargo build --features autonomic

# Enable wasm4pm evidence gate
cargo build --features wasm4pm

# Full release build
cargo build --release --features autonomic,wasm4pm
```

---

## Quick Troubleshooting

| Symptom | Fix |
|---------|-----|
| `No Cargo.toml found` | Navigate to workspace root: `cd /path/to/workspace` |
| `Oracle unavailable` | `wpm` not on PATH; add wasm4pm binary or declare `Blocked` in tests |
| Tests run full suite unexpectedly | Use `cargo cicd test changed` to run only affected tests |
| `cicd.toml` stale | Run `cargo cicd workspace doctor` to regenerate |
| `git close` refuses | Commit or stash dirty files before closing phase |

---

## Next Steps

- **Architecture**: `.claude/ARCHITECTURE.md` — system diagrams and data flows
- **Patterns**: `.claude/PATTERNS.md` — coding conventions and required patterns
- **Testing**: `.claude/TESTING.md` — test hierarchy and how to write tests
- **Full Reference**: `CLAUDE.md` — complete project guide (single source of truth)
- **Contributing**: `CONTRIBUTING.md` — how to add verbs, policies, and tests
- **Definition of Done**: `docs/DEFINITION_OF_DONE.md` — DoD for every work type
