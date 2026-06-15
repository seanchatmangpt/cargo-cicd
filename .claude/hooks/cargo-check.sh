#!/usr/bin/env bash
set -euo pipefail

# Manual convenience script — NOT auto-wired as a hook.
# Run directly: bash .claude/hooks/cargo-check.sh
#
# Checks formatting then performs a type-check pass (no artefacts emitted).

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

echo "==> cargo fmt --all -- --check"
cargo fmt --all -- --check

echo ""
echo "==> cargo check"
cargo check

echo ""
echo "cargo-check: all checks passed."
