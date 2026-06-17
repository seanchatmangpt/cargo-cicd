#!/bin/sh
# install-hooks.sh — Install cargo-cicd git hooks into .git/hooks/
# Usage: sh scripts/install-hooks.sh [--force]
#
# By default the installer skips hooks that already exist (unless --force is
# passed).  Run from the repository root.

set -e

# ---------------------------------------------------------------------------
# Color helpers
# ---------------------------------------------------------------------------
if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  CYAN='\033[0;36m'
  BOLD='\033[1m'
  RESET='\033[0m'
else
  RED=''
  GREEN=''
  YELLOW=''
  CYAN=''
  BOLD=''
  RESET=''
fi

FORCE=0
for arg in "$@"; do
  case "$arg" in
    --force|-f) FORCE=1 ;;
    --help|-h)
      printf "Usage: sh scripts/install-hooks.sh [--force]\n"
      printf "  --force  Overwrite existing hooks\n"
      exit 0
      ;;
  esac
done

# ---------------------------------------------------------------------------
# Locate repository root and hooks directory
# ---------------------------------------------------------------------------
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf "%b  Not inside a git repository. Run from the cargo-cicd root.%b\n" "$RED" "$RESET"
  exit 1
}
HOOKS_SRC="${REPO_ROOT}/scripts/hooks"
HOOKS_DST="${REPO_ROOT}/.git/hooks"

if [ ! -d "$HOOKS_SRC" ]; then
  printf "%b  Source directory not found: %s%b\n" "$RED" "$HOOKS_SRC" "$RESET"
  printf "  Run this script from the repository root.\n"
  exit 1
fi

if [ ! -d "$HOOKS_DST" ]; then
  printf "%b  .git/hooks directory not found: %s%b\n" "$RED" "$HOOKS_DST" "$RESET"
  exit 1
fi

printf "\n%b--- cargo-cicd hook installer ---%b\n\n" "$BOLD" "$RESET"
printf "  Source : %s\n" "$HOOKS_SRC"
printf "  Target : %s\n\n" "$HOOKS_DST"

INSTALLED=0
SKIPPED=0
ERRORS=0

# ---------------------------------------------------------------------------
# Install each hook
# ---------------------------------------------------------------------------
for HOOK_FILE in "${HOOKS_SRC}"/*; do
  [ -f "$HOOK_FILE" ] || continue
  HOOK_NAME=$(basename "$HOOK_FILE")
  DEST="${HOOKS_DST}/${HOOK_NAME}"

  if [ -e "$DEST" ] && [ "$FORCE" -eq 0 ]; then
    printf "  %b~%b  %-20s %b(already exists — use --force to overwrite)%b\n" \
      "$YELLOW" "$RESET" "$HOOK_NAME" "$CYAN" "$RESET"
    SKIPPED=$((SKIPPED + 1))
    continue
  fi

  if cp "$HOOK_FILE" "$DEST" && chmod +x "$DEST"; then
    printf "  %b✓%b  %-20s -> %s\n" "$GREEN" "$RESET" "$HOOK_NAME" "$DEST"
    INSTALLED=$((INSTALLED + 1))
  else
    printf "  %b✗%b  %-20s  %bfailed to install%b\n" "$RED" "$RESET" "$HOOK_NAME" "$RED" "$RESET"
    ERRORS=$((ERRORS + 1))
  fi
done

printf "\n"

if [ "$ERRORS" -gt 0 ]; then
  printf "%b  Installation completed with errors (%s installed, %s skipped, %s errors).%b\n\n" \
    "$RED" "$INSTALLED" "$SKIPPED" "$ERRORS" "$RESET"
  exit 1
else
  printf "%b  Installation complete: %s installed, %s skipped.%b\n" \
    "$GREEN" "$INSTALLED" "$SKIPPED" "$RESET"
  if [ "$SKIPPED" -gt 0 ]; then
    printf "  %b  Re-run with --force to overwrite existing hooks.%b\n" "$YELLOW" "$RESET"
  fi
  printf "\n  Hooks installed:\n"
  printf "    pre-commit  — format, clippy, invariants, forbidden-term scan\n"
  printf "    pre-push    — all pre-commit + full suite + hygiene checks\n"
  printf "    commit-msg  — conventional-commit format enforcement\n"
  printf "\n  See docs/dod/GIT_HOOKS.md for full documentation.\n\n"
  exit 0
fi
