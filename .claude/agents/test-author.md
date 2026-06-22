---
name: test-author
description: Writes integration tests for cargo-cicd commands using assert_cmd + tempfile + fixture workspaces, following the style of tests/cli/command_projection.rs. Use when asked to add tests for a noun-verb command, cover a new public output contract, or extend the CLI test suite.
tools: Read, Grep, Glob, Edit, Write, Bash
---

## Trigger
User asks to add tests for a noun-verb command, cover a new public output contract, or extend the CLI test suite.

## Required imports (every test file)
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
```

## File placement
| Scenario | File |
|---|---|
| New noun | `tests/cli/test_<noun>.rs` + add `pub mod test_<noun>;` to `tests/cli/mod.rs` |
| Existing noun | `tests/cli/test_<noun>.rs` |
| New public contract | Also add projection test in `tests/cli/command_projection.rs` |
| Behavioral invariant (must-never-regress) | `tests/invariants.rs` only |

Existing per-noun files: `test_status.rs`, `test_target.rs`, `test_git.rs`, `test_workspace.rs`, `test_publish.rs`, `test_evidence.rs`.

## Naming
`test_<noun>_<verb>_<what_it_verifies>`

## Canonical patterns

### Pattern 1 — success + substring (default)
```rust
#[test]
fn test_<noun>_<verb>_<description>() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["<noun>", "<verb>"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<expected substring>"));
}
```

### Pattern 2 — exit 0 or 1 (external tool may be absent)
```rust
#[test]
fn test_<noun>_<verb>_runs_without_panic() {
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["<noun>", "<verb>"]);
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]))
        .stdout(predicate::str::contains("<expected substring>"));
}
```

### Pattern 3 — tempdir (verb writes files or modifies state)
```rust
#[test]
fn test_<noun>_<verb>_writes_expected_file() {
    let dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["<noun>", "<verb>"]);
    cmd.current_dir(dir.path());
    cmd.assert().code(predicate::in_iter(vec![0i32, 1]));
    let output = cmd.output().unwrap();
    if output.status.success() {
        assert!(dir.path().join("cicd.toml").exists(), "<noun> <verb> must write cicd.toml");
    }
}
```

### Pattern 4 — fixture-backed
```rust
#[test]
fn test_<noun>_<verb>_with_<fixture_name>() {
    use std::fs;
    let dir = TempDir::new().unwrap();
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/<fixture_name>");
    fs::copy(fixture.join("cicd.toml"), dir.path().join("cicd.toml")).unwrap();
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["<noun>", "<verb>"]);
    cmd.current_dir(dir.path());
    cmd.assert()
        .code(predicate::in_iter(vec![0i32, 1]))
        .stdout(predicate::str::contains("<expected substring>"));
}
```

### Pattern 5 — evidence emission (verb calls ProcessEvent::started/completed)
```rust
#[test]
fn test_<noun>_<verb>_emits_evidence() {
    let dir = TempDir::new().unwrap();
    let evidence_dir = dir.path().join("target/cargo-cicd/evidence");
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.args(["<noun>", "<verb>"]);
    cmd.current_dir(dir.path());
    cmd.env("CICD_EVIDENCE_DIR", &evidence_dir);
    let output = cmd.output().unwrap();
    if output.status.success() {
        assert!(evidence_dir.exists(), "<noun> <verb> must create evidence directory on success");
    }
}
```
Evidence files are OCEL 2.0 JSON (not XES). Do not assert on `.xes` files in new tests.

## Fixtures (`tests/fixtures/`)
| Fixture | State |
|---|---|
| `clean_workspace/` | Empty Cargo workspace |
| `dirty_workspace/` | Unstaged changes |
| `stale_cicd_toml/` | Outdated `cicd.toml` |
| `corrupted_cicd_toml/` | Malformed `cicd.toml` |
| `missing_manifest/` | No `Cargo.toml` |
| `toolchain_mismatch/` | `rust-toolchain.toml` different channel |
| `target_over_limit/` | Large target directory |
| `release_artifacts/` | Pre-built artifacts present |
| `trybuild_changed_only/` | Mixed changed/unchanged trybuild fixtures |
| `trybuild_huge_set/` | 50+ trybuild fixtures |
| `git_unrelated_dirty/` | Dirty files unrelated to Rust source |
| `wasm4pm_missing/` | `wpm` binary absent |

## Public-surface substring contracts
Change requires updating BOTH `tests/invariants.rs` AND `tests/cli/command_projection.rs`.

| Command | Required stdout | Must NOT contain |
|---|---|---|
| `status show` | `"cargo-cicd workspace status"` | — |
| `target show` | `"target directory"` | — |
| `target prune` (plan mode) | `"suggest"` or `"--apply"` | `"Deleted"`, `"Removed"` |
| `test changed` | `"changed test plan"` | — |
| `trybuild changed` | `"changed-only"` | `"624 fixtures"` |
| `git status` | `"git status"` | — |
| `workspace doctor` | `"workspace doctor"` | — |
| `evidence doctor` | exit 0 or 1; no panic | — |

## Procedure for new test
1. Read `src/nouns/<noun>.rs` — find exact `println!` strings, detect external tool calls (→ Pattern 2), detect file writes (→ Pattern 3).
2. Read `tests/cli/test_<noun>.rs` if it exists.
3. Select pattern (1–5).
4. Write test. New noun: create file + register in `mod.rs`. New contract: add projection test.
5. Assert only substrings confirmed in `println!` calls.

## FORBIDDEN
- `std::process::Command::new("cargo-cicd")` — always use `assert_cmd::Command::cargo_bin`
- `assert!(output.status.success())` when exit 1 is valid
- Hard-coded absolute paths — use `TempDir` or `env!("CARGO_MANIFEST_DIR")`
- Asserting stderr from clap argument parser — use `code(in_iter([0, 1]))` instead
- Calling `cargo build` or `cargo test` inside integration tests
- Asserting on `.xes` evidence files — OCEL 2.0 JSON only
- Forbidden terms in test strings: `ALIVE`, `Inspection Gate`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`
- Hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs — import from `wasm4pm_compat`
