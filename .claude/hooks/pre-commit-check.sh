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
STEP=0
TOTAL=4

echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BOLD}  cargo-cicd  |  Pre-commit checks${RESET}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

# ── 1. Forbidden-terms scan on staged files ───────────────────────────────────
STEP=$((STEP + 1))
section "${STEP}/${TOTAL}  Forbidden-terms scan (staged files)"

FORBIDDEN_PATTERN='ALIVE|Nehemiah|Field8|Instinct8|Cargo Court|AGI|Truex|CONSTRUCT8|Inspection Gate'
STAGED_FILES="$(git diff --cached --name-only 2>/dev/null)"

if [[ -n "${STAGED_FILES}" ]]; then
  FORBIDDEN_HITS="$(echo "${STAGED_FILES}" | xargs grep -lE "${FORBIDDEN_PATTERN}" 2>/dev/null || true)"
  if [[ -n "${FORBIDDEN_HITS}" ]]; then
    fail "Forbidden terms found in staged files:"
    echo "${FORBIDDEN_HITS}" | while read -r f; do
      echo -e "     ${YELLOW}${f}${RESET}"
      grep -nE "${FORBIDDEN_PATTERN}" "${f}" 2>/dev/null | head -5 | while read -r line; do
        echo -e "       ${RED}${line}${RESET}"
      done
    done
    FAILURES=$((FAILURES + 1))
  else
    ok "No forbidden terms in staged files"
  fi
else
  ok "No staged files to scan"
fi

# ── 2. cargo fmt --check ──────────────────────────────────────────────────────
STEP=$((STEP + 1))
section "${STEP}/${TOTAL}  Formatting (cargo fmt --check)"

if cargo fmt --check 2>/dev/null; then
  ok "Code is formatted correctly"
else
  fail "Formatting issues found"
  echo -e "     Fix with: ${YELLOW}cargo fmt${RESET}"
  FAILURES=$((FAILURES + 1))
fi

# ── 3. cargo clippy ───────────────────────────────────────────────────────────
STEP=$((STEP + 1))
section "${STEP}/${TOTAL}  Lints (cargo clippy --all-features -- -D warnings)"

CLIPPY_OUTPUT="$(cargo clippy --all-features -- -D warnings 2>&1)"; CLIPPY_EXIT=$?

if [[ $CLIPPY_EXIT -eq 0 ]]; then
  ok "clippy: no warnings or errors"
else
  fail "clippy reported warnings / errors (exit ${CLIPPY_EXIT})"
  echo ""
  echo "${CLIPPY_OUTPUT}" | tail -30
  FAILURES=$((FAILURES + 1))
fi

# ── 4a. invariants test suite — default features ──────────────────────────────
STEP=$((STEP + 1))
section "${STEP}a/${TOTAL}  Public-boundary invariants — default features (cargo test --test invariants)"

INVARIANTS_OUTPUT="$(cargo test --test invariants 2>&1)"; INVARIANTS_EXIT=$?

if [[ $INVARIANTS_EXIT -eq 0 ]]; then
  # Count tests run
  PASSED="$(echo "${INVARIANTS_OUTPUT}" | grep -c 'test .* ok' 2>/dev/null || echo '?')"
  ok "All invariant tests passed — default features (${PASSED} tests)"
else
  fail "Invariant tests FAILED — default features (exit ${INVARIANTS_EXIT})"
  echo ""
  # Show last 40 lines of output for context
  echo "${INVARIANTS_OUTPUT}" | tail -40
  FAILURES=$((FAILURES + 1))
fi

section "${STEP}b/${TOTAL}  Public-boundary invariants — feature-gated help text (cargo test --test invariants --features autonomic,wasm4pm)"

INVARIANTS_FEAT_OUTPUT="$(cargo test --test invariants --features autonomic,wasm4pm 2>&1)"; INVARIANTS_FEAT_EXIT=$?

if [[ $INVARIANTS_FEAT_EXIT -eq 0 ]]; then
  PASSED_FEAT="$(echo "${INVARIANTS_FEAT_OUTPUT}" | grep -c 'test .* ok' 2>/dev/null || echo '?')"
  ok "All invariant tests passed — autonomic,wasm4pm features (${PASSED_FEAT} tests)"
else
  fail "Invariant tests FAILED — autonomic,wasm4pm features (exit ${INVARIANTS_FEAT_EXIT})"
  echo ""
  echo "${INVARIANTS_FEAT_OUTPUT}" | tail -40
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
