#!/usr/bin/env bash
# Session start hook for Claude Code (web + CLI). Always exits 0.
set -uo pipefail

WD="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$WD" 2>/dev/null || true

echo "cargo-cicd v26.6.2 — Claude Code Session Started"
echo "Workspace: $WD"
echo ""

check_tool() {
  local cmd="$1" hint="$2" req="$3"
  if command -v "$cmd" &>/dev/null; then
    echo "  [ok] $cmd $($cmd --version 2>/dev/null | head -1 | cut -d' ' -f2-)"
  elif [[ "$req" == "required" ]]; then
    echo "  [MISSING] $cmd — $hint"
  else
    echo "  [optional] $cmd not found — $hint"
  fi
}

echo "Tools:"
check_tool cargo  "install rustup from https://rustup.rs" required
check_tool git    "install git" required
check_tool makers "cargo install cargo-make" optional
check_tool wpm    "build wasm4pm, add to PATH (evidence gate only)" optional
echo ""

[[ -f "$WD/Cargo.toml" ]] && echo "  [ok] Cargo.toml found" || echo "  [warn] Cargo.toml not found"
[[ -f "$WD/cicd.toml" ]] || echo "  [info] cicd.toml absent — run: cargo cicd workspace doctor"

BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"
DIRTY="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
echo "  [git] branch=$BRANCH  dirty=$DIRTY"
echo ""

if [[ -d "$WD/target" ]]; then
  echo "  [target] $(du -sh "$WD/target" 2>/dev/null | cut -f1) used"
else
  echo "  [target] not yet built"
fi
echo ""

echo "Feature flags:  default | process-data | autonomic | wasm4pm | contrib"
echo ""
echo "Quick commands:"
echo "  cargo make build           build the binary"
echo "  cargo make test            run all tests"
echo "  cargo make check           lint + type-check"
echo "  cargo cicd status          workspace snapshot"
echo "  cargo cicd workspace doctor  full diagnostics"
echo "  cargo cicd evidence doctor   evidence gate"
echo ""

export RUST_BACKTRACE=1
export RUST_LOG=info
export CARGO_TERM_COLOR=always

exit 0
