# Developer Experience Guide

cargo-cicd keeps Rust workspaces clean, fast, and push-ready. This guide
collects everything a contributor needs to move efficiently from clone to
first commit.

---

## Quick Reference Card

### Daily Workflow

```sh
# Health check before starting work
cargo cicd status

# After editing source files
cargo make check          # type-check + lint without a full build
cargo make test           # full test suite

# Before committing
cargo cicd git status     # confirm phase, dirty files, ahead/behind
cargo make invariants     # verify no forbidden terms leaked into help text
cargo make fmt            # format code
```

### Testing

```sh
cargo make test                      # all test suites
cargo make test-verbose              # all tests, show stdout
cargo make test-features             # with process-data and autonomic flags
cargo make invariants                # 7 public boundary invariants only

# Run a single integration test file
cargo test --test cli
cargo test --test cicd_toml_truth

# Run a single test function
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help

# Run with a feature flag
cargo test --features wasm4pm --test wasm4pm_evidence_gate
```

### Evidence Gate (Release)

```sh
cargo make gate                       # evidence gate — requires wpm binary
wpm audit target/cargo-cicd/evidence/evt-*.xes
wpm receipt doctor --format json --strict receipts/*.json
```

### Release Steps

```sh
cargo make release-check              # fmt-check + lint + test + invariants + feature tests
# bump version in Cargo.toml and src/main.rs
git add -A
git commit -m "chore(release): vX.Y.Z ready for release"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main --tags
```

---

## Shell Aliases (Recommended)

Add these to your `~/.bashrc` or `~/.zshrc`:

```bash
# cargo-cicd shortcuts
alias cicd-status='cargo cicd status'
alias cicd-doctor='cargo cicd workspace doctor'
alias cicd-gate='cargo cicd evidence doctor'
alias cicd-git='cargo cicd git status'
alias cicd-test='cargo cicd test changed'

# cargo-make shortcuts
alias cm='cargo make'
alias cm-test='cargo make test'
alias cm-check='cargo make check'
alias cm-lint='cargo make lint'
alias cm-ci='cargo make ci'
```

After adding, reload your shell:

```bash
source ~/.bashrc   # or source ~/.zshrc
```

---

## Cargo Make Tasks Reference

All tasks are defined in `Makefile.toml` at the workspace root.

| Task | Description | When to Use |
|------|-------------|-------------|
| `build` | Build the debug binary | Local development |
| `build-release` | Release binary with `autonomic,wasm4pm` features | Pre-release verification |
| `check` | `cargo check --all-targets --all-features` | Fast feedback without compiling |
| `test` | `cargo test --workspace` | Standard test run |
| `test-verbose` | Tests with `--nocapture` | Debugging failing tests |
| `test-features` | Tests with `process-data,autonomic` | Feature-gated code paths |
| `lint` | `clippy --all-targets --all-features -D warnings` | Catch style and correctness issues |
| `fmt` | `cargo fmt --all` | Format code in place |
| `fmt-check` | `cargo fmt --all --check` | CI formatting gate |
| `invariants` | `cargo test --test invariants` | Public boundary verification |
| `gate` | Evidence gate with wpm oracle | Release verification |
| `ci` | `fmt-check` + `lint` + `test` + `invariants` | Full local CI pass |
| `release-check` | `ci` + `test-features` | Final pre-release gate |

Run any task with:

```sh
cargo make <task>
```

---

## VS Code / IDE Setup

### rust-analyzer Settings

Add to `.vscode/settings.json` (create if absent):

```json
{
  "rust-analyzer.cargo.features": ["process-data"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": [
    "--all-targets",
    "--all-features",
    "--",
    "-D",
    "warnings"
  ],
  "rust-analyzer.cargo.allFeatures": false,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### Recommended Extensions

| Extension | ID | Purpose |
|-----------|----|---------|
| rust-analyzer | `rust-lang.rust-analyzer` | Rust language support |
| Even Better TOML | `tamasfe.even-better-toml` | Syntax highlighting for `Cargo.toml`, `cicd.toml`, `Makefile.toml` |
| Error Lens | `usernamehw.errorlens` | Inline diagnostic messages |
| GitLens | `eamodio.gitlens` | Git phase visibility |

### Task Runner Integration

VS Code can run cargo-make tasks directly. Add to `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo-make: ci",
      "type": "shell",
      "command": "cargo make ci",
      "group": { "kind": "build", "isDefault": true },
      "presentation": { "reveal": "always", "panel": "shared" },
      "problemMatcher": "$rustc"
    },
    {
      "label": "cargo-make: test-verbose",
      "type": "shell",
      "command": "cargo make test-verbose",
      "group": "test",
      "presentation": { "reveal": "always", "panel": "dedicated" },
      "problemMatcher": "$rustc"
    },
    {
      "label": "cargo-make: invariants",
      "type": "shell",
      "command": "cargo make invariants",
      "group": "test",
      "presentation": { "reveal": "always", "panel": "shared" },
      "problemMatcher": "$rustc"
    }
  ]
}
```

Press `Ctrl+Shift+B` (or `Cmd+Shift+B`) to run the default `ci` task.

---

## Environment Variables

Set these in your shell profile for consistent development behaviour:

| Variable | Recommended Value | Purpose |
|----------|-------------------|---------|
| `RUST_BACKTRACE` | `1` | Full backtraces on panics |
| `CARGO_INCREMENTAL` | `0` | Reproducible builds; required for stable evidence emission |
| `CARGO_TERM_COLOR` | `always` | Coloured output even when piped |
| `WPM_PATH` | `/path/to/wpm` | Override wasm4pm binary location |
| `RUST_LOG` | `debug` | Verbose adapter logging (development only) |

Example `.env`-style block for your shell profile:

```bash
export RUST_BACKTRACE=1
export CARGO_INCREMENTAL=0
export CARGO_TERM_COLOR=always
# export WPM_PATH="/usr/local/bin/wpm"   # uncomment if wpm is not on PATH
```

`CARGO_INCREMENTAL=0` is also enforced in `.cargo/config.toml` for the
workspace, but setting it in the environment makes it visible to all tools
(e.g., IDE background compilation).

---

## Debugging Tips

### Get test output

```sh
cargo make test-verbose
# or for a single test:
cargo test --test cli -- --nocapture
```

### Check workspace state before committing

```sh
cargo cicd git status        # dirty files, ahead/behind, phase
cargo cicd workspace doctor  # full workspace diagnostics
```

### Diagnose a failing status command

```sh
RUST_LOG=debug cargo run -- status show 2>&1 | head -40
git status --porcelain
git diff origin/main --name-only
```

### Inspect emitted evidence

```sh
ls -la target/cargo-cicd/evidence/
cat target/cargo-cicd/evidence/evt-*.jsonl | python3 -m json.tool
wpm audit target/cargo-cicd/evidence/evt-*.xes
```

### Forbidden term leak

If `invariant_public_boundary_no_forbidden_terms_in_all_help` fails:

```sh
# The invariant test is the source of truth for the reserved-term list.
# Run it to see exactly which --help output trips the gate:
cargo test --test invariants \
  invariant_public_boundary_no_forbidden_terms_in_all_help -- --nocapture
```

Then search the source for the specific term the test reported:

```sh
rg "<term-reported-by-the-test>" src/
```

### Feature flag not compiling

```sh
cargo build --features autonomic 2>&1 | grep error
cargo build --features wasm4pm  2>&1 | grep error
```

### cicd.toml not written after a command

Read-only verbs (`show`, `status`, `explain`, `doctor`) may not write
`cicd.toml`. Execution verbs should. Force a write via:

```sh
cargo cicd workspace doctor
ls -la cicd.toml
```

---

## Cargo Aliases (`.cargo/config.toml`)

The workspace ships pre-configured aliases. Use them anywhere inside the repo:

```sh
cargo t        # cargo test --workspace
cargo b        # cargo build
cargo c        # cargo check --all-targets
cargo l        # cargo clippy --all-targets --all-features -- -D warnings
cargo f        # cargo fmt --all
```

cargo-cicd subcommand aliases (prefix the installed binary name):

```sh
cargo cicd-status   # cargo cicd status
cargo cicd-doctor   # cargo cicd workspace doctor
cargo cicd-git      # cargo cicd git status
cargo cicd-test     # cargo cicd test changed
cargo cicd-gate     # cargo cicd evidence doctor
```
