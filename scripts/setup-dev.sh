#!/usr/bin/env bash
# scripts/setup-dev.sh
# One-command developer setup for cargo-cicd.
# Installs required tooling, validates the Rust toolchain, creates the
# evidence directory, and prints a development environment summary.
# Run from the repository root: bash scripts/setup-dev.sh

set -euo pipefail

# ── ANSI colour codes ────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

PASS="${GREEN}✓${RESET}"
FAIL="${RED}✗${RESET}"
WARN="${YELLOW}⚠${RESET}"
INFO="${CYAN}→${RESET}"

section() { echo -e "\n${BOLD}${CYAN}$*${RESET}"; }
ok()      { echo -e "  ${PASS} $*"; }
fail()    { echo -e "  ${FAIL} $*"; }
warn()    { echo -e "  ${WARN} $*"; }
info()    { echo -e "  ${INFO} $*"; }

FAILURES=0

# ── Banner ────────────────────────────────────────────────────────────────────
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BOLD}  cargo-cicd  |  Developer setup${RESET}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

# ── 1. Rust toolchain check ──────────────────────────────────────────────────
section "1/4  Rust toolchain (MSRV 1.85)"

MSRV_MINOR=85

if ! command -v rustc &>/dev/null; then
  fail "rustc not found — install Rust from https://rustup.rs"
  FAILURES=$((FAILURES + 1))
else
  RUSTC_VERSION="$(rustc --version 2>&1)"
  ok "rustc: ${RUSTC_VERSION}"

  RUSTC_SEMVER="$(echo "${RUSTC_VERSION}" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')"
  RUSTC_MINOR="$(echo "${RUSTC_SEMVER}" | cut -d. -f2)"
  RUSTC_MAJOR="$(echo "${RUSTC_SEMVER}" | cut -d. -f1)"

  if [[ "${RUSTC_MAJOR}" -gt 1 ]] || \
     [[ "${RUSTC_MAJOR}" -eq 1 && "${RUSTC_MINOR}" -ge "${MSRV_MINOR}" ]]; then
    ok "MSRV 1.${MSRV_MINOR} satisfied (${RUSTC_SEMVER})"
  else
    fail "Rust ${RUSTC_SEMVER} is below MSRV 1.${MSRV_MINOR}"
    info "Run: rustup update stable"
    FAILURES=$((FAILURES + 1))
  fi
fi

if command -v rustup &>/dev/null; then
  ok "rustup available: $(rustup --version 2>&1 | head -1)"
  # Ensure rustfmt and clippy are present
  if rustup component list --installed 2>/dev/null | grep -q rustfmt; then
    ok "rustfmt component installed"
  else
    info "Installing rustfmt..."
    rustup component add rustfmt
    ok "rustfmt installed"
  fi
  if rustup component list --installed 2>/dev/null | grep -q clippy; then
    ok "clippy component installed"
  else
    info "Installing clippy..."
    rustup component add clippy
    ok "clippy installed"
  fi
else
  warn "rustup not found — component management unavailable"
fi

# ── 2. cargo-make ─────────────────────────────────────────────────────────────
section "2/4  cargo-make"

if cargo make --version &>/dev/null 2>&1; then
  ok "cargo-make already installed: $(cargo make --version 2>&1 | head -1)"
else
  info "cargo-make not found — installing (this may take a minute)..."
  if cargo install cargo-make; then
    ok "cargo-make installed successfully"
  else
    fail "cargo-make installation failed"
    warn "Try manually: cargo install cargo-make"
    FAILURES=$((FAILURES + 1))
  fi
fi

# ── 3. Evidence directory ────────────────────────────────────────────────────
section "3/4  Evidence directory"

EVIDENCE_DIR="target/cargo-cicd/evidence"
if mkdir -p "${EVIDENCE_DIR}"; then
  ok "Evidence directory ready: ${EVIDENCE_DIR}"
else
  fail "Failed to create: ${EVIDENCE_DIR}"
  FAILURES=$((FAILURES + 1))
fi

# ── 4. wasm4pm instructions ──────────────────────────────────────────────────
section "4/4  wasm4pm oracle"

DEFAULT_WPM="/Users/sac/wasm4pm/target/release/wpm"

if [[ -n "${WPM_PATH:-}" ]] && [[ -x "${WPM_PATH}" ]]; then
  ok "wpm found via WPM_PATH: ${WPM_PATH}"
elif [[ -x "${DEFAULT_WPM}" ]]; then
  ok "wpm found at default path: ${DEFAULT_WPM}"
  export WPM_PATH="${DEFAULT_WPM}"
  ok "Exported WPM_PATH=${WPM_PATH}"
elif command -v wpm &>/dev/null; then
  WPM_BIN="$(command -v wpm)"
  ok "wpm found on PATH: ${WPM_BIN}"
  export WPM_PATH="${WPM_BIN}"
else
  warn "wpm binary not found"
  echo ""
  echo -e "  ${BOLD}wasm4pm setup instructions:${RESET}"
  echo -e "  ${CYAN}1.${RESET} Clone and build wasm4pm:"
  echo -e "       git clone https://github.com/your-org/wasm4pm \$HOME/wasm4pm"
  echo -e "       cd \$HOME/wasm4pm && cargo build --release"
  echo -e "  ${CYAN}2.${RESET} Set WPM_PATH in your shell profile:"
  echo -e "       export WPM_PATH=\"\$HOME/wasm4pm/target/release/wpm\""
  echo -e "  ${CYAN}3.${RESET} Or symlink to a directory on PATH:"
  echo -e "       ln -sf \$HOME/wasm4pm/target/release/wpm /usr/local/bin/wpm"
  echo ""
  echo -e "  Evidence-gate tests (wasm4pm_evidence_gate.rs) require wpm at runtime."
  echo -e "  Internal smoke tests will pass without it; release closure requires wpm."
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BOLD}  Development environment summary${RESET}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

echo ""
echo -e "  ${BOLD}Key commands:${RESET}"
echo -e "    ${CYAN}cargo make build${RESET}              Build the workspace"
echo -e "    ${CYAN}cargo make check${RESET}              Lint + type-check (no binary)"
echo -e "    ${CYAN}cargo make test${RESET}               Run all tests"
echo -e "    ${CYAN}cargo test --test invariants${RESET}  Public boundary invariants"
echo -e "    ${CYAN}cargo test --features process-data${RESET}  Level 5 engine tests"
echo -e "    ${CYAN}cargo test --features autonomic${RESET}     Policy tests"
echo ""
echo -e "  ${BOLD}Evidence gate (wasm4pm required):${RESET}"
echo -e "    ${CYAN}cargo test --test wasm4pm_evidence_gate --features wasm4pm${RESET}"
echo ""
echo -e "  ${BOLD}Claude Code hooks:${RESET}"
echo -e "    .claude/hooks/session-start.sh    — runs on session open"
echo -e "    .claude/hooks/pre-commit-check.sh — runs before commit"
echo ""

if [[ "${FAILURES}" -eq 0 ]]; then
  echo -e "  ${PASS} ${BOLD}${GREEN}Setup complete — workspace ready for development.${RESET}"
else
  echo -e "  ${WARN} ${BOLD}${YELLOW}Setup completed with ${FAILURES} issue(s) — see above.${RESET}"
fi

echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo ""

exit 0
