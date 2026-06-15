#!/usr/bin/env bash
set -euo pipefail

# SessionStart hook for cargo-cicd
# Prints a concise project-readiness summary — no builds, always exits 0.

cat <<'BANNER'
┌─────────────────────────────────────────────────────┐
│           cargo-cicd — project ready                │
└─────────────────────────────────────────────────────┘
BANNER

echo "Project : cargo-cicd (Rust CI/CD workspace helper)"
echo ""

# Toolchain detection — best-effort, never fatal
if rustc_ver=$(rustc --version 2>/dev/null); then
    echo "Toolchain: ${rustc_ver}"
else
    echo "Toolchain: rustc not found on PATH (install via rustup)"
fi

echo ""
echo "Key commands:"
echo "  cargo make build       — build the workspace"
echo "  cargo make check       — lint + type-check (no build artefacts)"
echo "  cargo test             — run all integration tests"
echo "  cargo cicd status      — show workspace status"
echo "  cargo cicd ui demo     — launch the terminal UI demo"
echo ""
echo "Nouns (cargo cicd <noun> <verb>):"
echo "  status      show | audit"
echo "  target      show | prune"
echo "  test        changed"
echo "  trybuild    changed"
echo "  git         status | close"
echo "  publish     run"
echo "  workspace   doctor"
echo "  evidence    doctor | audit"
echo "  ui          demo | dashboard"
echo "  lsp"
echo "  pipeline"
echo ""
echo "Commit format: feat(core|cli|target|test|git|autonomic|docs|receipts): <description>"
echo ""

exit 0
