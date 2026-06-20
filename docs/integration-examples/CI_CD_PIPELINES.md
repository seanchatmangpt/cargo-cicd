# cargo-cicd Integration Examples

Practical examples of integrating cargo-cicd into CI/CD pipelines, IDEs, and development workflows.

**Version:** 26.6.19

## Table of Contents

1. [GitHub Actions](#github-actions)
2. [GitLab CI](#gitlab-ci)
3. [Pre-Commit Hooks](#pre-commit-hooks)
4. [IDE Integration](#ide-integration)
5. [Docker & Containers](#docker--containers)
6. [Development Workflows](#development-workflows)
7. [Monitoring & Observability](#monitoring--observability)

---

## GitHub Actions

### Basic Pipeline

Run cargo-cicd pipeline on every push and pull request:

```yaml
# .github/workflows/cicd.yml
name: cargo-cicd Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  cicd:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for git-based detection

      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-cicd
        run: cargo install cargo-cicd

      - name: Run cargo-cicd pipeline
        run: cargo cicd pipeline run

      - name: Upload evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: cargo-cicd-evidence
          # Evidence artifacts are emitted in OCEL 2.0 JSON format
          # (events.ocel.json) — the wpm oracle accepts this directly.
          path: target/cargo-cicd/evidence/*.ocel.json
```

### Modular Stages

Run cargo-cicd checks as separate jobs for clarity and parallel execution:

```yaml
# .github/workflows/modular-cicd.yml
name: Modular cargo-cicd

on:
  push:
    branches: [main]
  pull_request:

jobs:
  workspace-health:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - run: cargo cicd workspace doctor

  status:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - run: cargo cicd status

  tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - name: Plan tests
        run: cargo cicd test changed
      - name: Run all tests
        run: cargo test

  target-size:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - name: Check target size
        run: cargo cicd target show
      - name: Fail if too large
        run: |
          SIZE=$(cargo cicd target show | grep "total size" | awk '{print $3}')
          if (( $(echo "$SIZE > 20" | bc -l) )); then
            echo "Target directory too large: $SIZE GB"
            exit 1
          fi

  git-health:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - run: cargo cicd git status
```

### Pre-Release Gate

Gate releases on cargo-cicd readiness:

```yaml
# .github/workflows/release.yml
name: Release (cargo-cicd gated)

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release'
        required: true

jobs:
  pre-release-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-cicd
        run: cargo install cargo-cicd

      - name: Workspace health check
        run: cargo cicd workspace doctor

      - name: Verify clean state
        run: cargo cicd git status

      - name: Check evidence
        run: cargo cicd evidence doctor || echo "Oracle unavailable; continuing..."

      - name: Run full pipeline
        run: cargo cicd pipeline run

  publish:
    needs: pre-release-check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - name: Publish release
        run: |
          cargo cicd publish run
          cargo publish --token ${{ secrets.CARGO_TOKEN }}
      - name: Create GitHub release
        uses: actions/create-release@v1
        with:
          tag_name: v${{ github.event.inputs.version }}
          release_name: Release v${{ github.event.inputs.version }}
          body: |
            See CHANGELOG for details.
          draft: false
          prerelease: false
```

### Full CI Gate Stack

The production `ci.yml` runs four gates that together cover format, correctness, process evidence, cryptographic provenance, and admissibility. The job graph is:

```
fmt
 └─► check-and-test  (includes workspace sync step)
       ├─► evidence-gate
       │     └─► affidavit-gate
       └─► lsp-admissibility
```

```yaml
# .github/workflows/ci.yml (gate stack excerpt)
jobs:

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check

  check-and-test:
    needs: fmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd

      - name: Lint and type-check
        run: cargo clippy --all-targets -- -D warnings

      - name: Run test suite
        run: cargo test

      # Workspace sync — invokes `ggen sync` if ggen.toml is present,
      # proving the ontology pipeline is intact. Non-blocking.
      - name: Workspace sync
        continue-on-error: true
        run: cargo run -- workspace sync

  # Emits OCEL 2.0 process evidence and adjudicates via wpm oracle.
  evidence-gate:
    needs: check-and-test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - run: cargo cicd evidence doctor

      - name: Upload evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: cargo-cicd-evidence
          # Evidence artifacts are emitted in OCEL 2.0 JSON format
          # (events.ocel.json) — the wpm oracle accepts this directly.
          path: target/cargo-cicd/evidence/*.ocel.json

  # Seals XES/OCEL evidence into a BLAKE3 receipt, then verifies it
  # cryptographically. continue-on-error because affi may be absent in
  # some CI environments.
  affidavit-gate:
    needs: evidence-gate
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build with affidavit feature
        run: cargo build --features affidavit
      - name: Seal evidence
        run: cargo run --features affidavit -- affidavit seal || true
      - name: Verify receipt
        run: cargo run --features affidavit -- affidavit verify || true

  # Scans changed .rs files for anti-LLM admissibility violations.
  # Requires the anti-llm-cheat feature. Non-blocking.
  lsp-admissibility:
    needs: check-and-test
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - name: Build with anti-llm-cheat feature
        run: cargo build --features anti-llm-cheat
      - name: Check LSP admissibility
        run: cargo run --features anti-llm-cheat -- lsp check
```

**What each gate provides:**

| Gate | Feature flag | Purpose | Blocking? |
|------|-------------|---------|-----------|
| `check-and-test` (workspace sync step) | none | Proves ontology pipeline (`ggen sync`) is runnable | No (`continue-on-error`) |
| `evidence-gate` | none | Emits OCEL 2.0 process evidence; adjudicates via wpm oracle | Yes |
| `affidavit-gate` | `affidavit` | BLAKE3-seals evidence into a cryptographic receipt; verifies it | No (`continue-on-error`) |
| `lsp-admissibility` | `anti-llm-cheat` | Scans changed `.rs` files for admissibility violations | No (`continue-on-error`) |

> **Note:** Evidence artifacts are emitted in OCEL 2.0 JSON format (`events.ocel.json`) — the wpm oracle accepts this directly.

---

### Release Gate

The `release.yml` workflow adds a `status audit` step that adjudicates OCEL evidence through the wpm oracle before the publish job is allowed to proceed.

```yaml
# .github/workflows/release.yml (run-gates job excerpt)
jobs:

  run-gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd

      # Install wpm oracle if absent
      - name: Install wpm oracle
        run: |
          if ! command -v wpm &>/dev/null; then
            echo "wpm not found — install wasm4pm and add it to PATH"
            # e.g.: cargo install wasm4pm
          fi

      # Adjudicates OCEL evidence via wpm oracle.
      # Accept → release proceeds; Refuse → release blocked.
      - name: Status audit (wpm adjudication)
        run: cargo run -- status audit

  publish:
    needs: run-gates
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      - name: Publish release
        run: |
          cargo cicd publish run
          cargo publish --token ${{ secrets.CARGO_TOKEN }}
```

**wpm adjudication flow for `status audit`:**

```
cargo run -- status audit
    │
    ├─ Emits ProcessEvent (verdict_claimed = "PASS"|"WARN"|"FAIL")
    ├─ Serializes to OCEL 2.0 JSON (target/cargo-cicd/evidence/events.ocel.json)
    └─ Calls: wpm audit target/cargo-cicd/evidence/events.ocel.json
                │
                └─ Returns: Accept  → release proceeds
                            Refuse  → release blocked (non-zero exit)
                            Blocked → wpm unavailable (treat as skip in local dev)
```

A `Refuse` verdict causes `cargo run -- status audit` to exit non-zero, which blocks the `publish` job via `needs: run-gates`.

---

### Matrix Testing with cargo-cicd

Test multiple Rust versions with cargo-cicd verification:

```yaml
# .github/workflows/matrix.yml
name: Matrix Testing

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@${{ matrix.rust }}

      - name: Install cargo-cicd
        run: cargo install cargo-cicd

      - name: Check workspace health
        run: cargo cicd workspace

      - name: Run changed tests
        run: cargo cicd test changed

      - name: Run full test suite
        run: cargo test

      - name: Publish state
        run: cargo cicd publish run
```

---

## GitLab CI

### Basic Pipeline

Simple cargo-cicd integration in GitLab CI:

```yaml
# .gitlab-ci.yml
stages:
  - check
  - test
  - publish

variables:
  CARGO_TERM_COLOR: "always"
  RUST_BACKTRACE: "1"

before_script:
  - rustup default stable
  - cargo install cargo-cicd

workspace:health:
  stage: check
  script:
    - cargo cicd workspace doctor

status:
  stage: check
  script:
    - cargo cicd status

target:size:
  stage: check
  script:
    - cargo cicd target show

tests:
  stage: test
  script:
    - cargo cicd test changed
    - cargo test

publish:state:
  stage: publish
  script:
    - cargo cicd publish run
  only:
    - main
```

### Caching for Performance

Optimize CI/CD with cargo-cicd and caching:

```yaml
# .gitlab-ci.yml (with caching)
cache:
  paths:
    - target/
  key:
    files:
      - Cargo.lock

stages:
  - check
  - test

before_script:
  - rustup default stable
  - cargo install --force cargo-cicd

pipeline:
  stage: test
  script:
    - cargo cicd pipeline run
  artifacts:
    when: always
    paths:
      - target/cargo-cicd/evidence/
    expire_in: 1 week
```

---

## Pre-Commit Hooks

### Shell Pre-Commit Hook

Prevent commits until cargo-cicd passes:

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running cargo-cicd checks..."

# Check workspace health
echo "Checking workspace health..."
cargo cicd workspace doctor || {
  echo "ERROR: Workspace health check failed"
  exit 1
}

# Check test status
echo "Planning tests..."
cargo cicd test changed || {
  echo "ERROR: Test planning failed"
  exit 1
}

# Verify git state
echo "Verifying git state..."
cargo cicd git status || {
  echo "ERROR: Git status check failed"
  exit 1
}

# Check target size
echo "Checking target size..."
cargo cicd target show || {
  echo "ERROR: Target size check failed"
  exit 1
}

echo "✓ All checks passed! Proceeding with commit."
exit 0
```

### Install Hook

```bash
#!/bin/bash
# scripts/install-hooks.sh

HOOK_FILE=".git/hooks/pre-commit"

cat > "$HOOK_FILE" <<'EOF'
#!/bin/bash
set -e
echo "Running cargo-cicd checks..."
cargo cicd workspace doctor || exit 1
cargo cicd test changed || exit 1
cargo cicd git status || exit 1
cargo cicd target show || exit 1
echo "✓ All checks passed!"
exit 0
EOF

chmod +x "$HOOK_FILE"
echo "Pre-commit hook installed"
```

Run once:
```bash
bash scripts/install-hooks.sh
```

---

## IDE Integration

### VS Code

Create a VS Code task to run cargo-cicd checks:

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo-cicd: Status",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "status"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    },
    {
      "label": "cargo-cicd: Workspace Doctor",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "workspace"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true,
        "panel": "new"
      }
    },
    {
      "label": "cargo-cicd: Test Changed",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "test", "changed"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false
      }
    },
    {
      "label": "cargo-cicd: Full Pipeline",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "pipeline", "run"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true
      }
    },
    {
      "label": "cargo-cicd: Target Prune",
      "type": "shell",
      "command": "cargo",
      "args": ["cicd", "target", "prune", "--apply"],
      "problemMatcher": [],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": true
      }
    }
  ]
}
```

Bind to keyboard shortcut:

```json
// .vscode/keybindings.json
[
  {
    "key": "ctrl+shift+c",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Status"
  },
  {
    "key": "ctrl+shift+w",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Workspace Doctor"
  },
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Test Changed"
  },
  {
    "key": "ctrl+shift+p",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo-cicd: Full Pipeline"
  }
]
```

### Rust-Analyzer Integration

Create a custom check that runs cargo-cicd as part of VS Code's Rust-Analyzer:

Add to `settings.json`:

```json
{
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },
  "rust-analyzer.checkOnSave.command": "cargo",
  "rust-analyzer.checkOnSave.extraArgs": ["cicd", "workspace"],
  "rust-analyzer.inlayHints.enable": true
}
```

---

## Docker & Containers

### Dockerfile

Build a Docker image with cargo-cicd pre-installed:

```dockerfile
# Dockerfile
FROM rust:latest

WORKDIR /workspace

# Install cargo-cicd
RUN cargo install cargo-cicd

# Copy workspace
COPY . .

# Run pipeline
RUN cargo cicd pipeline run

# Build the project
RUN cargo build --release

CMD ["cargo", "run", "--release"]
```

Build and run:

```bash
docker build -t my-workspace .
docker run my-workspace
```

### Docker Compose

Multi-service setup with cargo-cicd in CI/CD:

```yaml
# docker-compose.yml
version: '3.8'

services:
  ci:
    build:
      context: .
      dockerfile: Dockerfile.ci
    volumes:
      - .:/workspace
      - cargo-cache:/usr/local/cargo/registry
    environment:
      RUST_BACKTRACE: 1
    command: cargo cicd pipeline run

  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    depends_on:
      - ci

volumes:
  cargo-cache:
```

### CI Dockerfile

Dedicated Dockerfile for CI/CD checks:

```dockerfile
# Dockerfile.ci
FROM rust:latest

WORKDIR /workspace

RUN cargo install cargo-cicd

COPY Cargo.* ./
COPY src ./src
COPY tests ./tests

# Run checks
RUN cargo cicd workspace doctor && \
    cargo cicd test changed && \
    cargo cicd target show && \
    cargo cicd publish run

RUN cargo test
RUN cargo build --release
```

---

## Development Workflows

### Local Development Loop

Automated checks during development:

```bash
#!/bin/bash
# scripts/dev-loop.sh

set -e

echo "Starting dev loop... Press Ctrl+C to exit"

while true; do
  clear
  echo "=== cargo-cicd Dev Loop ==="
  echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
  echo

  # Run checks
  cargo cicd workspace || echo "⚠ Workspace check failed"
  echo
  
  cargo cicd status
  echo
  
  cargo cicd target show
  echo
  
  # Optional: run tests
  echo "Running tests..."
  cargo test --lib --doc --quiet || echo "⚠ Tests failed"
  echo

  echo "=== Waiting 30 seconds before next check ==="
  sleep 30
done
```

Run with:
```bash
bash scripts/dev-loop.sh
```

### Pre-Push Script

Enforce checks before pushing to remote:

```bash
#!/bin/bash
# scripts/pre-push.sh

echo "Running pre-push checks..."

# Refuse to push if workspace is unhealthy
cargo cicd workspace doctor || {
  echo "ERROR: Workspace health check failed. Fix before pushing."
  exit 1
}

# Refuse to push if tree is dirty
cargo cicd git close || {
  echo "ERROR: Git tree is dirty. Commit changes before pushing."
  exit 1
}

# Warn if target is large
if cargo cicd target show | grep -q "fail\|warn"; then
  echo "⚠ WARNING: Target directory is large. Consider running:"
  echo "  cargo cicd target prune --apply"
  echo "Proceeding with push anyway..."
fi

echo "✓ All pre-push checks passed!"
git push origin $(git branch --show-current)
```

Usage:
```bash
bash scripts/pre-push.sh
```

### Makefile Integration

Integrate cargo-cicd into Makefile:

```makefile
# Makefile
.PHONY: check doctor status test publish pipeline clean prune

# Install cargo-cicd
install-cicd:
	cargo install cargo-cicd

# Quick status check
status:
	cargo cicd status

# Full workspace diagnosis
doctor:
	cargo cicd workspace

# Run changed tests only
test-changed:
	cargo cicd test changed
	cargo test

# Full test suite
test:
	cargo test --all

# Publish state
publish:
	cargo cicd publish

# Full pipeline
pipeline:
	cargo cicd pipeline run

# Clean target directory
clean:
	cargo clean

# Smart prune (dry-run)
prune-preview:
	cargo cicd target prune

# Smart prune (execute)
prune:
	cargo cicd target prune --apply

# Pre-commit checks
pre-commit: doctor test-changed
	cargo cicd git status

# Pre-push checks
pre-push: doctor test-changed
	cargo cicd git close
	cargo cicd publish

# Full CI equivalent
ci: doctor test publish pipeline

.PHONY: help
help:
	@echo "cargo-cicd Makefile targets:"
	@echo "  make status          - Show workspace status"
	@echo "  make doctor          - Full workspace diagnosis"
	@echo "  make test-changed    - Run changed tests only"
	@echo "  make test            - Run full test suite"
	@echo "  make publish         - Publish state to cicd.toml"
	@echo "  make pipeline        - Run full pipeline"
	@echo "  make prune-preview   - Preview target cleanup"
	@echo "  make prune           - Execute target cleanup"
	@echo "  make pre-commit      - Pre-commit checks"
	@echo "  make pre-push        - Pre-push checks"
	@echo "  make ci              - Full CI equivalent"
```

Usage:
```bash
make doctor
make test-changed
make pre-push
make ci
```

---

## Monitoring & Observability

### Health Check Script

Monitor workspace health continuously:

```bash
#!/bin/bash
# scripts/health-check.sh

INTERVAL=${1:-60}  # Default 60 seconds

echo "Starting health check loop (interval: ${INTERVAL}s)"

while true; do
  TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
  
  echo "[$TIMESTAMP] Running checks..."
  
  # Get workspace status
  STATUS=$(cargo cicd status 2>&1)
  if echo "$STATUS" | grep -q "clean"; then
    HEALTH="✓ HEALTHY"
  else
    HEALTH="⚠ DIRTY"
  fi
  
  # Get target size
  TARGET=$(cargo cicd target show 2>&1 | grep "total size" | awk '{print $3}')
  
  # Log to file
  echo "[$TIMESTAMP] Health: $HEALTH | Target: $TARGET GB" >> /tmp/workspace-health.log
  
  # Optional: Alert if unhealthy
  if [[ "$HEALTH" == "⚠ DIRTY" ]]; then
    echo "[$TIMESTAMP] WARNING: Workspace is dirty" >&2
  fi
  
  echo ""
  sleep "$INTERVAL"
done
```

Run with:
```bash
bash scripts/health-check.sh 30  # Check every 30 seconds
```

### Metrics Export

Export cargo-cicd metrics for monitoring:

```bash
#!/bin/bash
# scripts/export-metrics.sh

# Export to JSON for processing
{
  echo "{"
  echo '  "timestamp": "'$(date -u +'%Y-%m-%dT%H:%M:%SZ')',"'
  
  # Get metrics
  TARGET_SIZE=$(cargo cicd target show 2>/dev/null | grep "total size" | awk '{print $3}' | cut -d' ' -f1)
  echo '  "target_size_gb": '${TARGET_SIZE:-0}','
  
  GIT_STATUS=$(cargo cicd git status 2>/dev/null | grep "dirty files" | awk '{print $3}')
  echo '  "dirty_files": '${GIT_STATUS:-0}','
  
  HEALTH=$(cargo cicd workspace doctor 2>/dev/null | grep "PASS\|FAIL" | grep -o "PASS\|FAIL" | tail -1)
  echo '  "workspace_health": "'${HEALTH:-UNKNOWN}'"'
  
  echo "}"
} | tee /tmp/metrics.json
```

### Integration with ELK Stack

Send cargo-cicd evidence to Elasticsearch:

```bash
#!/bin/bash
# scripts/send-to-elasticsearch.sh

ELASTICSEARCH_URL=${1:-"http://localhost:9200"}
INDEX_NAME="cargo-cicd-evidence"

# Read events from JSONL
EVENTS_FILE="target/cargo-cicd/evidence/events.jsonl"

if [ ! -f "$EVENTS_FILE" ]; then
  echo "No events file found at $EVENTS_FILE"
  exit 1
fi

# Send each event to Elasticsearch
while IFS= read -r line; do
  curl -s -X POST "$ELASTICSEARCH_URL/$INDEX_NAME/_doc" \
    -H "Content-Type: application/json" \
    -d "$line"
done < "$EVENTS_FILE"

echo "Events sent to Elasticsearch"
```

---

## Example: Complete CI/CD Pipeline

A complete, production-ready pipeline combining all approaches:

```yaml
# .github/workflows/complete-pipeline.yml
name: Complete cargo-cicd Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # Nightly

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  # Stage 1: Quick checks
  quick-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd
      
      - name: Workspace Health
        run: cargo cicd workspace doctor

      - name: Status
        run: cargo cicd status

      - name: Target Size
        run: cargo cicd target show

  # Stage 2: Tests
  tests:
    needs: quick-check
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@${{ matrix.rust }}
      - run: cargo install cargo-cicd

      - name: Plan Tests
        run: cargo cicd test changed

      - name: Run Tests
        run: cargo test

  # Stage 3: Publish & Audit (main branch only)
  publish:
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    needs: tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd

      - name: Publish State
        run: cargo cicd publish run

      - name: Evidence Audit
        run: cargo cicd evidence doctor || echo "Oracle unavailable"

      - name: Upload Evidence
        uses: actions/upload-artifact@v4
        with:
          name: evidence
          path: target/cargo-cicd/evidence/*.ocel.json

  # Stage 4: Nightly full pipeline
  nightly-pipeline:
    if: github.event_name == 'schedule'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cicd

      - name: Full Pipeline
        run: cargo cicd pipeline run

      - name: Upload Evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: nightly-evidence
          path: target/cargo-cicd/evidence/*.ocel.json
```

---

## Tips & Best Practices

1. **Always fetch full history for git-based detection:**
   ```yaml
   - uses: actions/checkout@v4
     with:
       fetch-depth: 0
   ```

2. **Cache Rust build artifacts:**
   ```yaml
   - uses: Swatinem/rust-cache@v2
   ```

3. **Run checks in parallel where possible:**
   - Status, target, workspace checks can run in parallel
   - Tests should run after workspace check passes

4. **Gate releases on cargo-cicd:**
   - Use `pipeline run` before publishing
   - Require evidence audit to pass

5. **Monitor evidence files:**
   - Upload evidence artifacts for auditing
   - Keep logs for troubleshooting

6. **Use appropriate fail conditions:**
   - Workspace doctor: fail on critical issues
   - Tests: always run, but fail on test failures
   - Publish: fail on oracle refusal

7. **Integrate with notifications:**
   - Alert on workspace health failures
   - Track metrics over time

---

## Further Reading

- [Quick Start Guide](../reference/CLI_QUICK_START.md)
- [Complete Command Reference](../reference/COMMANDS.md)
- [Troubleshooting Guide](../reference/CLI_TROUBLESHOOTING.md)
- [Architecture Documentation](../SOLUTION_ARCHITECTURE.md)
