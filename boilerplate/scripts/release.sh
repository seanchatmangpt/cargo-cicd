#!/usr/bin/env bash
# release.sh — Local pre-release gate for cargo-project
#
# Usage:
#   ./scripts/release.sh [--dry-run] [--yes]
#
# Options:
#   --dry-run    Run every validation step; skip the final tag + push (step 9).
#   --yes        Skip the interactive confirmation prompt in step 9.
#   -h, --help   Show this help.
#
# What it does:
#   Step 1  — Check git working tree is clean
#   Step 2  — Check current branch is main (or master)
#   Step 3  — Read VERSION from src/Cargo.toml
#   Step 4  — Validate VERSION looks like semver (X.Y.Z)
#   Step 5  — Confirm CHANGELOG.md has a ## [VERSION] entry
#   Step 6  — Run full CI gate: fmt --check, clippy, tests
#   Step 7  — Verify the tag vVERSION does not already exist
#   Step 8  — Print release summary
#   Step 9  — Tag and push (skipped in --dry-run)

set -euo pipefail

# ---------------------------------------------------------------------------
# Color output (TTY-aware)
# ---------------------------------------------------------------------------
if [[ -t 1 ]]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  CYAN='\033[0;36m'
  BOLD='\033[1m'
  DIM='\033[2m'
  RESET='\033[0m'
else
  RED='' GREEN='' YELLOW='' CYAN='' BOLD='' DIM='' RESET=''
fi

# Symbols (ASCII-safe so this works on every terminal)
PASS_MARK="${GREEN}[PASS]${RESET}"
FAIL_MARK="${RED}[FAIL]${RESET}"
SKIP_MARK="${DIM}[SKIP]${RESET}"
INFO_MARK="${CYAN}[INFO]${RESET}"

pass()  { echo -e "${PASS_MARK} $*"; }
fail()  { echo -e "${FAIL_MARK} $*" >&2; }
skip()  { echo -e "${SKIP_MARK} $*"; }
info()  { echo -e "${INFO_MARK} $*"; }
die()   { fail "$*"; exit 1; }
sep()   { echo -e "\n${BOLD}--- $* ---${RESET}"; }

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
DRY_RUN=false
YES=false

usage() {
  cat <<EOF
${BOLD}Usage:${RESET}
  $(basename "$0") [--dry-run] [--yes]

${BOLD}Options:${RESET}
  --dry-run    Validate everything; skip git tag and push (step 9).
  --yes        Skip interactive confirmation prompt.
  -h, --help   Show this help.

${BOLD}Steps:${RESET}
  1  Check git working tree is clean
  2  Check current branch is main or master
  3  Read VERSION from src/Cargo.toml
  4  Validate VERSION is semver (X.Y.Z)
  5  Confirm CHANGELOG.md has a ## [VERSION] entry
  6  Run full CI gate: fmt --check, clippy -D warnings, tests
  7  Verify tag v\$VERSION does not already exist
  8  Print release summary
  9  Create annotated tag and push (skipped with --dry-run)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=true; shift ;;
    --yes|-y)  YES=true;     shift ;;
    -h|--help) usage; exit 0 ;;
    *) die "Unknown option: $1 — run with --help for usage." ;;
  esac
done

# ---------------------------------------------------------------------------
# Locate workspace root
# ---------------------------------------------------------------------------
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" \
  || die "Not inside a git repository."
cd "${REPO_ROOT}"

if ${DRY_RUN}; then
  echo -e "\n${DIM}Dry-run mode — step 9 (tag + push) will be skipped.${RESET}\n"
fi

# ---------------------------------------------------------------------------
# Step 1: Working tree must be clean
# ---------------------------------------------------------------------------
sep "Step 1 — Working tree"

DIRTY=$(git status --porcelain 2>/dev/null)
if [[ -n "${DIRTY}" ]]; then
  fail "Working tree is dirty. Commit or stash your changes first."
  echo ""
  git status --short
  echo ""
  die "Refusing to release from a dirty working tree."
fi
pass "Working tree is clean."

# ---------------------------------------------------------------------------
# Step 2: Must be on main (or master)
# ---------------------------------------------------------------------------
sep "Step 2 — Branch check"

BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
if [[ "${BRANCH}" != "main" ]] && [[ "${BRANCH}" != "master" ]]; then
  die "Current branch is '${BRANCH}'. Releases must be tagged from main or master."
fi
pass "On branch '${BRANCH}'."

# ---------------------------------------------------------------------------
# Step 3: Read VERSION from src/Cargo.toml
# ---------------------------------------------------------------------------
sep "Step 3 — Read version"

CARGO_TOML="${REPO_ROOT}/src/Cargo.toml"
[[ -f "${CARGO_TOML}" ]] || die "src/Cargo.toml not found at ${CARGO_TOML}"

VERSION=$(grep -m1 '^version' "${CARGO_TOML}" | cut -d'"' -f2)
[[ -n "${VERSION}" ]] || die "Could not parse version from ${CARGO_TOML}"
info "VERSION = ${BOLD}${VERSION}${RESET}"

# ---------------------------------------------------------------------------
# Step 4: Validate semver (X.Y.Z — no pre-release labels)
# ---------------------------------------------------------------------------
sep "Step 4 — Validate semver"

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  die "Version '${VERSION}' is not strict semver (X.Y.Z). Update src/Cargo.toml."
fi
pass "Version '${VERSION}' is valid semver."

# ---------------------------------------------------------------------------
# Step 5: CHANGELOG.md must have a ## [VERSION] entry
# ---------------------------------------------------------------------------
sep "Step 5 — CHANGELOG entry"

CHANGELOG="${REPO_ROOT}/CHANGELOG.md"
if [[ ! -f "${CHANGELOG}" ]]; then
  die "CHANGELOG.md not found at ${CHANGELOG}. Add a changelog entry first."
fi

if ! grep -qE "^## \[?${VERSION//./\\.}\]?" "${CHANGELOG}"; then
  fail "No '## [${VERSION}]' heading found in CHANGELOG.md."
  die "Add a changelog entry for v${VERSION} before releasing."
fi
pass "CHANGELOG.md has an entry for '${VERSION}'."

# ---------------------------------------------------------------------------
# Step 6: Full CI gate
# ---------------------------------------------------------------------------
sep "Step 6 — CI gate"

info "Running: cargo fmt --check"
if ! cargo fmt --check 2>&1; then
  fail "cargo fmt --check failed. Run 'cargo fmt' to fix formatting."
  die "Formatting check failed."
fi
pass "Formatting check passed."

info "Running: cargo clippy --workspace --all-features -- -D warnings"
if ! cargo clippy --workspace --all-features -- -D warnings 2>&1; then
  fail "cargo clippy failed. Fix all warnings before releasing."
  die "Clippy check failed."
fi
pass "Clippy check passed."

info "Running: cargo test --workspace --all-features"
if ! cargo test --workspace --all-features 2>&1; then
  fail "cargo test failed. All tests must pass before releasing."
  die "Test suite failed."
fi
pass "All tests passed."

# ---------------------------------------------------------------------------
# Step 7: Tag must not already exist
# ---------------------------------------------------------------------------
sep "Step 7 — Tag uniqueness"

TAG="v${VERSION}"
if git rev-parse "${TAG}" >/dev/null 2>&1; then
  die "Tag '${TAG}' already exists. Bump the version in src/Cargo.toml first."
fi
pass "Tag '${TAG}' is available."

# ---------------------------------------------------------------------------
# Step 8: Summary
# ---------------------------------------------------------------------------
sep "Step 8 — Release summary"

echo ""
echo -e "  ${BOLD}Binary:${RESET}   cargo-project"
echo -e "  ${BOLD}Version:${RESET}  ${VERSION}"
echo -e "  ${BOLD}Tag:${RESET}      ${TAG}"
echo -e "  ${BOLD}Branch:${RESET}   ${BRANCH}"
echo -e "  ${BOLD}Commit:${RESET}   $(git rev-parse --short HEAD)"
if ${DRY_RUN}; then
  echo -e "  ${BOLD}Dry run:${RESET}  YES — step 9 will be skipped"
fi
echo ""

# ---------------------------------------------------------------------------
# Step 9: Tag and push (skipped in --dry-run)
# ---------------------------------------------------------------------------
sep "Step 9 — Tag and push"

if ${DRY_RUN}; then
  skip "Dry run — skipping tag creation and push."
  skip "Would run: git tag -a \"${TAG}\" -m \"Release ${TAG}\""
  skip "Would run: git push origin \"${TAG}\""
  echo ""
  echo -e "${GREEN}${BOLD}Dry run complete.${RESET} All checks passed; nothing was pushed."
  exit 0
fi

# Interactive confirmation (skip with --yes)
if ! ${YES}; then
  echo -e "${BOLD}Ready to tag and push.${RESET}"
  echo -e "  git tag -a \"${TAG}\" -m \"Release ${TAG}\""
  echo -e "  git push origin \"${TAG}\""
  echo ""
  read -r -p "Continue? [y/N] " CONFIRM
  case "${CONFIRM}" in
    y|Y|yes|YES) ;;
    *) echo "Aborted."; exit 0 ;;
  esac
fi

git tag -a "${TAG}" -m "Release ${TAG}"
pass "Annotated tag '${TAG}' created."

git push origin "${TAG}"
pass "Tag '${TAG}' pushed to origin."

echo ""
echo -e "${GREEN}${BOLD}Release ${TAG} pushed!${RESET}"
echo ""
echo "Next steps:"
echo "  - Watch the release workflow: https://github.com/$(git remote get-url origin 2>/dev/null | sed 's/.*github.com[:/]\(.*\)\.git/\1/' || echo 'your-org/your-repo')/actions"
echo "  - Verify binaries are attached to the GitHub Release once the workflow completes."
