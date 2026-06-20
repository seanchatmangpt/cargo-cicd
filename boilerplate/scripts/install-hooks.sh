#!/usr/bin/env bash
# install-hooks.sh — Install git hooks from .git-hooks/ into the local repository
#
# Usage:
#   ./scripts/install-hooks.sh
#   ./scripts/install-hooks.sh --uninstall    # Restore default hooks path

set -euo pipefail

# ---------------------------------------------------------------------------
# Color helpers (TTY-aware)
# ---------------------------------------------------------------------------
if [[ -t 1 ]]; then
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  CYAN='\033[0;36m'
  BOLD='\033[1m'
  RESET='\033[0m'
else
  GREEN='' YELLOW='' CYAN='' BOLD='' RESET=''
fi

info()    { echo -e "${CYAN}[hooks]${RESET} $*"; }
success() { echo -e "${GREEN}[hooks]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[hooks] WARN:${RESET} $*"; }

# ---------------------------------------------------------------------------
# Locate repo root
# ---------------------------------------------------------------------------
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "Not inside a git repository." >&2
  exit 1
}

HOOKS_DIR="${REPO_ROOT}/.git-hooks"

# ---------------------------------------------------------------------------
# Handle --uninstall
# ---------------------------------------------------------------------------
if [[ "${1:-}" == "--uninstall" ]]; then
  git -C "${REPO_ROOT}" config --unset core.hooksPath || true
  success "Restored default git hooks path (.git/hooks)."
  exit 0
fi

# ---------------------------------------------------------------------------
# Verify .git-hooks directory exists
# ---------------------------------------------------------------------------
if [[ ! -d "${HOOKS_DIR}" ]]; then
  warn ".git-hooks/ directory not found at ${HOOKS_DIR}"
  warn "Nothing to install."
  exit 1
fi

# ---------------------------------------------------------------------------
# Make all hook scripts executable
# ---------------------------------------------------------------------------
info "Making hooks executable..."
chmod +x "${HOOKS_DIR}"/*

HOOK_COUNT=$(find "${HOOKS_DIR}" -maxdepth 1 -type f | wc -l | tr -d ' ')
info "Found ${HOOK_COUNT} hook(s) in ${HOOKS_DIR}/"

for hook in "${HOOKS_DIR}"/*; do
  [[ -f "${hook}" ]] || continue
  echo "    $(basename "${hook}")"
done

# ---------------------------------------------------------------------------
# Configure git to use the .git-hooks directory
# ---------------------------------------------------------------------------
git -C "${REPO_ROOT}" config core.hooksPath .git-hooks

echo ""
success "Git hooks installed."
info "Hooks path set to: .git-hooks/"
info "To uninstall: ./scripts/install-hooks.sh --uninstall"
echo ""
echo -e "${BOLD}Active hooks:${RESET}"
for hook in "${HOOKS_DIR}"/*; do
  [[ -f "${hook}" ]] || continue
  echo "  - $(basename "${hook}")"
done
