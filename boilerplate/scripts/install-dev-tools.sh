#!/usr/bin/env bash
# install-dev-tools.sh — install recommended cargo-make dev toolchain
#
# Installs tools used by Makefile.toml tasks. Skips any tool already on PATH.
# Usage: ./scripts/install-dev-tools.sh

set -euo pipefail

# ──────────────────────────────────────────────────────────────────────────────
# Helpers
# ──────────────────────────────────────────────────────────────────────────────

INSTALLED=()
SKIPPED=()
FAILED=()

green()  { printf '\033[0;32m%s\033[0m\n' "$*"; }
yellow() { printf '\033[0;33m%s\033[0m\n' "$*"; }
red()    { printf '\033[0;31m%s\033[0m\n' "$*"; }
bold()   { printf '\033[1m%s\033[0m\n' "$*"; }

install_tool() {
  local name="$1"
  local bin="$2"
  shift 2
  local install_args=("$@")

  if command -v "$bin" >/dev/null 2>&1; then
    yellow "  [skip]    $name — already installed ($(command -v "$bin"))"
    SKIPPED+=("$name")
    return 0
  fi

  printf '  [install] %s ...\n' "$name"
  if cargo install "${install_args[@]}" 2>&1; then
    green "  [ok]      $name installed"
    INSTALLED+=("$name")
  else
    red "  [fail]    $name — installation failed (see output above)"
    FAILED+=("$name")
  fi
}

# ──────────────────────────────────────────────────────────────────────────────
# Detect toolchain
# ──────────────────────────────────────────────────────────────────────────────

bold ""
bold "==> cargo dev-tools installer"
printf "    Rust: %s\n" "$(rustc --version 2>/dev/null || echo 'not found')"
printf "    cargo: %s\n" "$(cargo --version 2>/dev/null || echo 'not found')"
printf "    date: %s\n" "$(date -u '+%Y-%m-%d %H:%M UTC')"
bold ""

IS_NIGHTLY=false
if rustc --version 2>/dev/null | grep -q nightly; then
  IS_NIGHTLY=true
fi

# ──────────────────────────────────────────────────────────────────────────────
# Stable tools (always install)
# ──────────────────────────────────────────────────────────────────────────────

bold "==> Stable tools"

install_tool "cargo-nextest" "cargo-nextest" \
  "cargo-nextest" "--locked"

install_tool "cargo-hack" "cargo-hack" \
  "cargo-hack" "--locked"

install_tool "cargo-audit" "cargo-audit" \
  "cargo-audit" "--locked"

install_tool "cargo-deny" "cargo-deny" \
  "cargo-deny" "--locked"

install_tool "cargo-bloat" "cargo-bloat" \
  "cargo-bloat"

install_tool "cargo-expand" "cargo-expand" \
  "cargo-expand" "--locked"

install_tool "cargo-watch" "cargo-watch" \
  "cargo-watch" "--locked"

install_tool "cargo-semver-checks" "cargo-semver-checks" \
  "cargo-semver-checks" "--locked"

# ──────────────────────────────────────────────────────────────────────────────
# Nightly-only tools
# ──────────────────────────────────────────────────────────────────────────────

bold ""
bold "==> Nightly-only tools"

if [ "$IS_NIGHTLY" = true ]; then
  install_tool "cargo-udeps" "cargo-udeps" \
    "cargo-udeps" "--locked"
else
  yellow "  [skip]    cargo-udeps — requires a nightly toolchain (current: stable)"
  yellow "            Switch with: rustup override set nightly"
  SKIPPED+=("cargo-udeps")
fi

# ──────────────────────────────────────────────────────────────────────────────
# Summary
# ──────────────────────────────────────────────────────────────────────────────

bold ""
bold "==> Summary"

if [ ${#INSTALLED[@]} -gt 0 ]; then
  green "  Installed (${#INSTALLED[@]}): ${INSTALLED[*]}"
fi

if [ ${#SKIPPED[@]} -gt 0 ]; then
  yellow "  Skipped   (${#SKIPPED[@]}): ${SKIPPED[*]}"
fi

if [ ${#FAILED[@]} -gt 0 ]; then
  red "  Failed    (${#FAILED[@]}): ${FAILED[*]}"
  bold ""
  red "  One or more tools failed to install. Check output above for details."
  exit 1
fi

bold ""
green "  All tools ready. Run 'cargo make --list-all-steps' to see available tasks."
