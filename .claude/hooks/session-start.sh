#!/usr/bin/env bash
# .claude/hooks/session-start.sh
# SessionStart hook for cargo-cicd — runs at the start of every Claude Code session.
# Checks toolchain, tooling, and build health; prints a status summary.
# Always exits 0 so the session is never blocked.

set -uo pipefail

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

# ── Helpers ──────────────────────────────────────────────────────────────────
section() { echo -e "\n${BOLD}${CYAN}$*${RESET}"; }
ok()      { echo -e "  ${PASS} $*"; }
fail()    { echo -e "  ${FAIL} $*"; }
warn()    { echo -e "  ${WARN} $*"; }

# Accumulate failures (non-blocking — we always exit 0)
FAILURES=0

# ── Banner ────────────────────────────────────────────────────────────────────
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BOLD}  cargo-cicd  |  SessionStart environment check${RESET}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

# ── 1. Rust toolchain ─────────────────────────────────────────────────────────
section "1/4  Rust toolchain"

RUSTC_VERSION=""
if command -v rustc &>/dev/null; then
  RUSTC_VERSION="$(rustc --version 2>&1)"
  ok "rustc found: ${RUSTC_VERSION}"
else
  fail "rustc not found — install via https://rustup.rs"
  FAILURES=$((FAILURES + 1))
fi

# MSRV check: require >= 1.85
MSRV="1.85"
if [[ -n "${RUSTC_VERSION}" ]]; then
  # Extract "1.NN.N" from "rustc 1.NN.N (hash date)"
  RUSTC_SEMVER="$(echo "${RUSTC_VERSION}" | grep -oP '\d+\.\d+\.\d+')"
  RUSTC_MAJOR="$(echo "${RUSTC_SEMVER}" | cut -d. -f1)"
  RUSTC_MINOR="$(echo "${RUSTC_SEMVER}" | cut -d. -f2)"

  MSRV_MINOR="$(echo "${MSRV}" | cut -d. -f2)"

  if [[ "${RUSTC_MAJOR}" -gt 1 ]] || \
     [[ "${RUSTC_MAJOR}" -eq 1 && "${RUSTC_MINOR}" -ge "${MSRV_MINOR}" ]]; then
    ok "MSRV ${MSRV} satisfied (found ${RUSTC_SEMVER})"
  else
    fail "MSRV ${MSRV} NOT satisfied (found ${RUSTC_SEMVER}) — run: rustup update"
    FAILURES=$((FAILURES + 1))
  fi
fi

# ── 2. cargo-make ─────────────────────────────────────────────────────────────
section "2/4  cargo-make"

if cargo make --version &>/dev/null 2>&1; then
  MAKE_VERSION="$(cargo make --version 2>&1 | head -1)"
  ok "cargo-make available: ${MAKE_VERSION}"
else
  fail "cargo-make not found — install with: cargo install cargo-make"
  warn "Fallback build: cargo build / cargo check"
  FAILURES=$((FAILURES + 1))
fi

# ── 3. wasm4pm binary ─────────────────────────────────────────────────────────
section "3/4  wasm4pm oracle"

DEFAULT_WPM="/Users/sac/wasm4pm/target/release/wpm"
WPM_FOUND=""

# Honour WPM_PATH if already exported by the caller; otherwise try the default.
if [[ -n "${WPM_PATH:-}" ]]; then
  if [[ -x "${WPM_PATH}" ]]; then
    WPM_FOUND="${WPM_PATH}"
    ok "wpm found via WPM_PATH: ${WPM_PATH}"
  else
    warn "WPM_PATH set to '${WPM_PATH}' but binary not executable there"
  fi
fi

if [[ -z "${WPM_FOUND}" ]]; then
  if [[ -x "${DEFAULT_WPM}" ]]; then
    WPM_FOUND="${DEFAULT_WPM}"
    export WPM_PATH="${DEFAULT_WPM}"
    ok "wpm found at default path: ${DEFAULT_WPM}"
    ok "Exported WPM_PATH=${WPM_PATH}"
  elif command -v wpm &>/dev/null; then
    WPM_FOUND="$(command -v wpm)"
    export WPM_PATH="${WPM_FOUND}"
    ok "wpm found on PATH: ${WPM_FOUND}"
    ok "Exported WPM_PATH=${WPM_PATH}"
  else
    fail "wpm binary not found"
    warn "  Expected at: ${DEFAULT_WPM}"
    warn "  Or set WPM_PATH env var, or add wpm to PATH"
    warn "  Evidence-gate tests require wpm at runtime"
    FAILURES=$((FAILURES + 1))
  fi
fi

# ── 4. Cargo check ────────────────────────────────────────────────────────────
section "4/4  Build validation (cargo check)"

if cargo check --quiet 2>/dev/null; then
  ok "cargo check passed — workspace compiles cleanly"
else
  CHECK_EXIT=$?
  fail "cargo check failed (exit ${CHECK_EXIT})"
  warn "Run 'cargo check' manually for full diagnostics"
  FAILURES=$((FAILURES + 1))
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

if [[ "${FAILURES}" -eq 0 ]]; then
  echo -e "  ${PASS} ${BOLD}${GREEN}All checks passed — workspace ready.${RESET}"
else
  echo -e "  ${WARN} ${BOLD}${YELLOW}${FAILURES} check(s) need attention (see above).${RESET}"
  echo -e "     Session continues — checks are advisory only."
fi

echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo ""

# Always exit 0 — hooks must not block the session.
exit 0
