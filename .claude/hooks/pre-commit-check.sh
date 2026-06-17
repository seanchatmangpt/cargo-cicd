#!/usr/bin/env bash
# .claude/hooks/pre-commit-check.sh
# PreToolUse hook — runs before a git commit via Claude Code.
# Enforces: formatting, clippy lints, and public-boundary invariants.
# Exits non-zero on failure so the commit is blocked.

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

section() { echo -e "\n${BOLD}${CYAN}$*${RESET}"; }
ok()      { echo -e "  ${PASS} $*"; }
fail()    { echo -e "  ${FAIL} $*"; }

FAILURES=0

echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BOLD}  cargo-cicd  |  Pre-commit checks${RESET}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

# ── 1. cargo fmt --check ──────────────────────────────────────────────────────
section "1/3  Formatting (cargo fmt --check)"

if cargo fmt --check 2>/dev/null; then
  ok "Code is formatted correctly"
else
  fail "Formatting issues found"
  echo -e "     Fix with: ${YELLOW}cargo fmt${RESET}"
  FAILURES=$((FAILURES + 1))
fi

# ── 2. cargo clippy ───────────────────────────────────────────────────────────
section "2/3  Lints (cargo clippy -- -D warnings)"

CLIPPY_OUTPUT="$(cargo clippy -- -D warnings 2>&1)" || CLIPPY_EXIT=$?

if [[ "${CLIPPY_EXIT:-0}" -eq 0 ]]; then
  ok "clippy: no warnings or errors"
else
  fail "clippy reported warnings / errors (exit ${CLIPPY_EXIT:-?})"
  echo ""
  echo "${CLIPPY_OUTPUT}" | tail -30
  FAILURES=$((FAILURES + 1))
fi

# ── 3. invariants test suite ──────────────────────────────────────────────────
section "3/3  Public-boundary invariants (cargo test --test invariants)"

INVARIANTS_OUTPUT="$(cargo test --test invariants 2>&1)" || INVARIANTS_EXIT=$?

if [[ "${INVARIANTS_EXIT:-0}" -eq 0 ]]; then
  # Count tests run
  PASSED="$(echo "${INVARIANTS_OUTPUT}" | grep -c 'test .* ok' 2>/dev/null || echo '?')"
  ok "All invariant tests passed (${PASSED} tests)"
else
  fail "Invariant tests FAILED (exit ${INVARIANTS_EXIT:-?})"
  echo ""
  # Show last 40 lines of output for context
  echo "${INVARIANTS_OUTPUT}" | tail -40
  FAILURES=$((FAILURES + 1))
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

if [[ "${FAILURES}" -eq 0 ]]; then
  echo -e "  ${PASS} ${BOLD}${GREEN}All pre-commit checks passed — commit allowed.${RESET}"
  echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  exit 0
else
  echo -e "  ${FAIL} ${BOLD}${RED}${FAILURES} check(s) failed — commit blocked.${RESET}"
  echo -e "     Fix the issues above and try again."
  echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  exit 1
fi
