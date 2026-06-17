# Complete Command Reference

This is the comprehensive reference for all `cargo-cicd` commands. Commands are organized by noun, with all available verbs and options for each.

**Version:** 26.6.2

## Command Syntax

```
cargo cicd <noun> [verb] [options]
```

- **Noun:** The main subject (e.g., `status`, `target`, `test`)
- **Verb:** The action to perform (e.g., `show`, `prune`, `changed`)
- **Options:** Command-specific flags and parameters

## Global Options

These options work with all commands:

```
--help, -h          Print help for the command
--version           Print cargo-cicd version
```

## Nouns and Verbs

### evidence

Manage and adjudicate process evidence via the wasm4pm oracle.

**About:** Adjudicate runtime process evidence via wasm4pm receipt doctor.

#### evidence doctor

Run the wasm4pm receipt doctor on the latest process receipt.

```sh
cargo cicd evidence doctor
cargo cicd evidence             # Shorthand (doctor is default)
```

**Description:** Invokes `wpm receipt doctor --format json --strict` on the latest process receipt. Returns the oracle's JSON verdict (Accepted/Refused). Emits an `evidence:doctor` event with the adjudication result.

**Output:** JSON receipt from wpm oracle with verdict

**Exit code:** 0 if oracle accepts, non-zero if oracle refuses or unavailable

**Notes:**
- Requires wasm4pm (`wpm` binary) to be installed or `WPM_PATH` env var to be set
- Safe to run at any time; read-only operation
- Useful for manual evidence auditing before publish

**Examples:**

```sh
# Run receipt doctor on latest evidence
cargo cicd evidence doctor

# Check receipt validity in CI/CD pipeline
if cargo cicd evidence doctor > /dev/null; then
  echo "Evidence accepted by oracle"
fi
```

#### evidence audit

Audit process evidence receipts (alias for doctor).

```sh
cargo cicd evidence audit
```

**Description:** Canonical alias for `evidence doctor`. Runs the same wasm4pm receipt doctor operation.

**Notes:** Preferred public-facing verb for evidence adjudication.

---

### git

Manage git repository state and enforce phase closure.

**About:** Git phase management.

#### git status

Show the current git repository state.

```sh
cargo cicd git status
```

**Description:** Displays structured summary of git state:
- Current branch name
- Files staged for commit
- Modified (dirty) files
- Untracked files
- Commits ahead/behind origin
- Recommendation for next action

**Output:**
```
git status
==========
branch:       main
staged:       0
dirty:        0
untracked:    0
ahead:        0
behind:       0

next: tree is clean — ready to push
```

**Exit code:** 0 always (read-only, non-destructive)

**Notes:**
- Safe to run at any time
- Shows "PASS" if tree is clean, "WARN" if dirty
- Does not modify the repository

**Examples:**

```sh
# Check if repo is clean
cargo cicd git status

# Use in a script to gate deployment
if cargo cicd git status | grep -q "tree is clean"; then
  git push origin main
fi
```

#### git close

Enforce git phase closure: verify tree is clean, then refuse if dirty.

```sh
cargo cicd git close
```

**Description:** Ensures the git working tree is clean (no modified or untracked files). If any dirty files exist, displays them and refuses to close the phase.

**Behavior:**
- If tree is clean: prints "phase already closed", emits PASS event, exits 0
- If tree is dirty: prints dirty files, refuses closure, exits non-zero

**Output (clean):**
```
git phase closure
=================
tree is clean — phase already closed
```

**Output (dirty):**
```
git phase closure
=================
dirty files:   3
untracked:     1

phase closure requires a clean tree.
stage and commit your changes before closing the phase.
```

**Exit code:** 0 if clean, 1 if dirty

**Notes:**
- Does NOT automatically commit or stage files
- Requires manual `git add` and `git commit` before retry
- Design: refuses to hide unrelated dirty files in a batch commit
- Emits `git:close` event to evidence log

**Examples:**

```sh
# Verify phase can be closed
cargo cicd git close

# In a release pipeline: stage evidence, then close phase
git add cicd.toml evidence/
git commit -m "emit release evidence"
cargo cicd git close
```

---

### pipeline

Execute the full declared manufacturing pipeline.

**About:** Execute the full declared manufacturing pipeline.

#### pipeline run

Run all pipeline stages in sequence.

```sh
cargo cicd pipeline run
```

**Description:** Executes the complete pipeline as a single operation:
1. `status show` — Display workspace status
2. `target show` — Check target directory size
3. `test changed` — Run tests for changed files
4. `trybuild changed` — Run trybuild for changed fixtures
5. `workspace doctor` — Diagnose workspace health
6. `publish run` — Publish cicd.toml with current state
7. `status audit` — Run wasm4pm evidence audit (if available)

Each stage runs as a subprocess. Output from each stage is streamed. If any stage fails, the pipeline continues but overall exit is non-zero.

**Output (example):**
```
cargo-cicd manufacturing pipeline
==================================
  status:show ... PASS (152ms)
  target:show ... PASS (28ms)
  test:changed ... PASS (2340ms)
  trybuild:changed ... PASS (1850ms)
  workspace:doctor ... PASS (89ms)
  publish:run ... PASS (45ms)
  status:audit ... ACCEPT (320ms)

Pipeline completed in 4824ms
```

**Exit code:** 0 if all stages pass, 1 if any stage fails

**Notes:**
- Fresh session: clears previous evidence logs and creates new session ID
- Emits XES trace for process mining (TRUTHFUL fitness ≥0.95)
- Oracle audit skips gracefully if wpm not found
- Useful for nightly CI/CD runs or pre-release validation

**Examples:**

```sh
# Run full pipeline
cargo cicd pipeline run

# Run pipeline and track exit code
cargo cicd pipeline run && echo "Pipeline passed" || echo "Pipeline failed"

# In CI: run full pipeline on every commit
cargo cicd pipeline run > /tmp/pipeline.log 2>&1
```

---

### publish

Publish workspace state to cicd.toml.

**About:** Publish cicd.toml with current workspace state.

#### publish run

Emit cicd.toml with current workspace state.

```sh
cargo cicd publish run
cargo cicd publish           # Shorthand (run is default)
```

**Description:** Captures current workspace state and writes to `cicd.toml`:
- Workspace name, root directory, toolchain
- Target directory size in GB
- Git state (dirty flag, staged/unstaged/untracked counts)
- Changed file counts since base ref (default: origin/main)
- Changed test and trybuild fixture counts
- Adjudication status from wasm4pm receipt doctor (if available)

Runs receipt doctor to verify publish readiness. If oracle is available and refuses the receipt, publish fails. If oracle is unavailable, proceeds with warning.

**Output:**
```
  adjudication: RECEIPT_DOCTOR:accepted
published cicd.toml
  workspace:    my-project
  toolchain:    stable-2026-06-14
  target:       8.42 GB
  dirty:        false
  changed:      5
```

**cicd.toml example:**
```toml
[workspace]
name = "my-project"
toolchain = "stable-2026-06-14"
target_dir = "target"

[state]
target_size_gb = 8.42
dirty = false
changed_files = 5
changed_tests = 2
changed_trybuild_fixtures = 0

[[events]]
timestamp = "2026-06-14T12:34:56.789Z"
activity = "publish:run"
verdict = "PASS"
```

**Exit code:** 0 on success, 1 if oracle refuses or operation fails

**Notes:**
- Safe to run repeatedly (overwrites previous cicd.toml)
- Useful after every test pass or before deployment
- Receipt doctor adjudication is required for release gates
- Emits `publish:run` event

**Examples:**

```sh
# Publish current state after successful tests
cargo test && cargo cicd publish run

# Publish and verify receipt doctor accepts
if cargo cicd publish run 2>&1 | grep -q "accepted"; then
  echo "Ready for release"
  git push origin main
fi

# In CI: publish on every commit
cargo cicd publish run || exit 1
```

---

### status

Show workspace CI/CD status.

**About:** Show workspace CI/CD status.

#### status show

Display current workspace status.

```sh
cargo cicd status show
cargo cicd status           # Shorthand (show is default)
```

**Description:** Displays a summary of workspace CI/CD readiness:
- Active toolchain (from rustup or rust-toolchain file)
- Target directory size and verdict (pass/warn based on 20 GB threshold)
- Current git branch
- Count of dirty and untracked files
- Overall git state (clean/dirty)

**Output:**
```
cargo-cicd workspace status
===========================
toolchain:    stable-2026-06-14
target:       8.42 GB [pass]
branch:       main
dirty files:  0
untracked:    0
git:          clean
```

**Exit code:** 0 always (read-only)

**Notes:**
- Safe to run at any time
- Non-destructive; does not modify anything
- Useful as a quick health check before committing

**Examples:**

```sh
# Quick status check
cargo cicd status

# Monitor status in a loop
while true; do cargo cicd status; sleep 30; done

# Status in a pre-commit hook
cargo cicd status | grep -q "clean" || exit 1
```

#### status audit

Adjudicate current evidence via the wasm4pm oracle.

```sh
cargo cicd status audit
```

**Description:** Runs wasm4pm audit on the current evidence XES file. Emits adjudication events:
- `status:audit` — the audit activity itself
- `evidence:audit` — the oracle's verdict (ACCEPT/REFUSE)
- `receipt:write` — only if oracle accepts

**Exit code:** 0 if oracle accepts, 1 if refuses or XES not found

**Notes:**
- Requires XES evidence file at `target/cargo-cicd/evidence/events.xes`
- Requires wasm4pm (`wpm audit` command)
- Useful for manual evidence validation before release

**Examples:**

```sh
# Audit current evidence
cargo cicd status audit

# Audit and fail if not accepted
cargo cicd status audit || echo "Evidence not accepted"
```

---

### target

Manage the Cargo target directory.

**About:** Manage target directory.

#### target show

Display target directory size and state.

```sh
cargo cicd target show
```

**Description:** Reports on the local target directory:
- Total size in GB
- Comparison against 20 GB default max
- Verdict: pass (under limit), warn (approaching limit), or fail (over limit)
- Recommendation if over limit

**Output:**
```
target directory: target
total size:       8.42 GB
max configured:   20.0 GB
verdict:          pass
```

**Output (over limit):**
```
target directory: target
total size:       25.50 GB
max configured:   20.0 GB
verdict:          fail
recommendation:   run 'cargo cicd target prune' to free space
```

**Exit code:** 0 always (read-only)

**Notes:**
- Non-destructive; does not modify the directory
- Threshold is 20 GB by default
- Useful for monitoring disk usage

**Examples:**

```sh
# Check target size
cargo cicd target show

# Alert if target is too large
if cargo cicd target show | grep -q "fail"; then
  echo "WARNING: target directory too large"
  cargo cicd target prune --apply
fi
```

#### target prune

Plan and optionally execute target directory cleanup.

```sh
cargo cicd target prune              # Dry-run (default)
cargo cicd target prune --apply      # Actually delete artifacts
```

**Description:** Identifies and optionally removes old build artifacts:
- Default: shows what WOULD be deleted (dry-run)
- With `--apply`: actually deletes the artifacts

**Safe deletion candidates (in order):**
1. `target/debug/incremental` — incremental build cache
2. `target/debug/.fingerprint` — build fingerprints
3. `target/debug/deps` — dependency artifacts

Release artifacts (`target/release/*`) are NEVER deleted automatically.

**Output (dry-run):**
```
target prune plan
=================
current size: 25.50 GB
mode:         suggest (use --apply to execute)

suggested candidates:
  target/debug/incremental (12.34 GB)
  target/debug/.fingerprint (3.42 GB)
  target/debug/deps (5.23 GB)

to execute: cargo cicd target prune --apply
note: release artifacts are never deleted automatically
```

**Output (with --apply):**
```
target prune plan
=================
current size: 25.50 GB
mode:         apply (deleting incremental artifacts)

  removed target/debug/incremental
  removed target/debug/.fingerprint
  removed target/debug/deps

freed: 20.99 GB
note: release artifacts are never deleted automatically
```

**Exit code:** 0 on success

**Notes:**
- Default is dry-run; must use `--apply` to actually delete
- Release artifacts always protected
- Safe to run repeatedly
- Next build will re-create deleted artifacts automatically

**Examples:**

```sh
# Preview what would be deleted
cargo cicd target prune

# Delete if over limit
cargo cicd target show | grep -q "fail" && cargo cicd target prune --apply

# Aggressive workspace cleanup
cargo cicd target prune --apply && cargo clean && cargo build
```

---

### test

Run tests for changed files.

**About:** Run changed tests.

#### test changed

Run tests only for changed source files.

```sh
cargo cicd test changed
```

**Description:** Identifies changed Rust source files since `origin/main` and determines affected test files. Conservative by design: if detection fails, suggests running full `cargo test`.

**Output:**
```
changed test plan
=================
base ref:         origin/main
changed .rs:      3
affected tests:   2
  tests/integration_tests.rs
  src/lib.rs

run: cargo test tests/integration_tests.rs src/lib.rs

note: exact affected-test selection is conservative by design
```

**Output (no changes):**
```
changed test plan
=================
base ref:         origin/main
changed .rs:      0
affected tests:   0
no changed test files detected — conservative mode
recommendation: run 'cargo test' to be safe
```

**Exit code:** 0 always (planning only, doesn't run tests)

**Notes:**
- Detects changes relative to `origin/main` by default (customizable in cicd.toml)
- Conservative: favors running more tests over missing coverage
- Useful for fast feedback during development
- Does NOT actually run cargo test; just suggests which tests to run

**Examples:**

```sh
# See which tests would run
cargo cicd test changed

# Run the suggested tests
TEST_FILES=$(cargo cicd test changed | grep "run:" | cut -d: -f2)
eval "cargo test $TEST_FILES"

# Or use in shell pipeline
cargo cicd test changed | grep "^run:" | xargs cargo test
```

---

### trybuild

Manage trybuild compile-fail and compile-pass fixtures.

**About:** Manage trybuild fixtures.

#### trybuild changed

Run trybuild only for changed fixtures.

```sh
cargo cicd trybuild changed
```

**Description:** Identifies changed trybuild fixture files (`tests/ui/*.rs`) and displays the scope of the trybuild run. Conservative: if no changed fixtures detected, recommends full `cargo test` instead.

**Output:**
```
trybuild changed plan
====================
base ref:             origin/main
changed fixtures:     2
mode:                 changed-only (all-fixture run is opt-in)
snapshot mode:        changed-only

selected fixtures:
  tests/ui/compile_fail_01.rs
  tests/ui/compile_pass_02.rs

to update snapshots: TRYBUILD=overwrite cargo test
```

**Output (no changes):**
```
trybuild changed plan
====================
base ref:             origin/main
changed fixtures:     0
mode:                 changed-only (all-fixture run is opt-in)
snapshot mode:        changed-only

no changed trybuild fixtures detected
skipping trybuild run — use 'cargo test' for full run
```

**Exit code:** 0 always (planning only)

**Notes:**
- Detects changes to files under `tests/ui/`
- Conservative: recommends full run if detection uncertain
- Snapshot updates use `TRYBUILD=overwrite` environment variable
- Does NOT actually run cargo test; just suggests scope

**Examples:**

```sh
# See which fixtures would run
cargo cicd trybuild changed

# Update snapshots for changed fixtures
TRYBUILD=overwrite cargo test

# Run just the changed fixtures
TRYBUILD=overwrite cargo test tests/ui/
```

---

### workspace

Workspace diagnostics and health checks.

**About:** Workspace diagnostics.

#### workspace doctor

Diagnose workspace health and run autonomic policies.

```sh
cargo cicd workspace doctor
cargo cicd workspace         # Shorthand (doctor is default)
```

**Description:** Comprehensive workspace health check:
- Verifies Cargo.toml exists
- Checks active toolchain
- Checks for rust-toolchain file
- Verifies git repository exists
- Checks cicd.toml state
- Runs autonomic policy checks (version skew, toolchain mismatch, etc.)

**Output (healthy):**
```
workspace doctor
================
[OK] Cargo.toml
[OK] toolchain: stable-2026-06-14
[OK] rust-toolchain file
[OK] git repository
[OK] cicd.toml (run 'cargo cicd publish' to generate)

autonomic policy results
------------------------
[PASS] pinned-toolchain: matches active toolchain
[PASS] target-size: 8.42 GB < 20 GB limit

workspace is healthy
```

**Output (unhealthy):**
```
workspace doctor
================
[FAIL] Cargo.toml
[OK] toolchain: stable-2026-06-14
[WARN] rust-toolchain file
[OK] git repository
[WARN] cicd.toml (run 'cargo cicd publish' to generate)

FAIL: workspace has critical issues
```

**Exit code:** 0 if healthy, 1 if critical issues detected

**Notes:**
- Critical issues: missing Cargo.toml or .git directory
- Warnings do not block operations but should be addressed
- Autonomic policies provide recommendations
- Safe to run at any time

**Examples:**

```sh
# Full workspace diagnosis
cargo cicd workspace

# Use in CI: fail if critical issues
cargo cicd workspace || exit 1

# Check health before publishing
cargo cicd workspace && cargo cicd publish
```

---

## Special Commands

### cargo cicd --version

```sh
cargo cicd --version
```

Displays the installed version (26.6.2).

### cargo cicd --help

```sh
cargo cicd --help
```

Displays brief help for all commands.

### cargo cicd <noun> --help

```sh
cargo cicd status --help
cargo cicd target --help
cargo cicd git --help
```

Displays help specific to a noun and its verbs.

---

## Exit Codes

All commands follow these exit code conventions:

| Code | Meaning |
|------|---------|
| 0 | Success (command completed as expected) |
| 1 | Failure (command encountered an error or refused) |
| 2 | Invalid workspace (Cargo.toml not found) |
| 3 | Readiness check failed (not ready for operation) |

---

## Environment Variables

### WPM_PATH

Set the path to the wasm4pm binary if not on PATH:

```sh
export WPM_PATH=/path/to/wpm
cargo cicd evidence doctor
```

---

## File Paths

### cicd.toml

Default location: workspace root (`./cicd.toml`)

Contains workspace state and CI/CD configuration.

### Evidence Directory

Default location: `target/cargo-cicd/evidence/`

Contains:
- `events.jsonl` — Event log (JSONL format)
- `events.xes` — XES trace (process mining format)
- `audit-events.xes` — Canonical audit trace
- `receipts/latest.json` — Latest process receipt
- `.session` — Session ID

---

## Tips and Tricks

### Scripting with cargo-cicd

```sh
# Pre-commit hook: verify workspace before committing
#!/bin/bash
cargo cicd workspace || exit 1
cargo cicd test changed || exit 1
cargo cicd target show || exit 1
exit 0
```

### CI/CD Integration

```sh
# GitHub Actions: run full pipeline
- name: cargo-cicd pipeline
  run: cargo cicd pipeline run

# GitLab CI: check workspace health
script:
  - cargo cicd workspace
  - cargo cicd publish run
```

### Automating Cleanup

```sh
# Cleanup script for development
#!/bin/bash
echo "Cleaning workspace..."
cargo cicd target prune --apply
cargo clean
cargo build
cargo cicd publish
```

### Monitoring Workspace Size

```sh
# Monitor target size
while true; do
  SIZE=$(cargo cicd target show | grep "total size" | awk '{print $3}')
  echo "Target size: $SIZE"
  sleep 60
done
```
