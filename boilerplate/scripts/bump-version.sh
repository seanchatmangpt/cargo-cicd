#!/usr/bin/env bash
# bump-version.sh — Update version in all workspace Cargo.toml files and README.md
#
# Usage:
#   ./scripts/bump-version.sh <new-version>
#   ./scripts/bump-version.sh 1.2.3
#
# Updates:
#   - Root Cargo.toml
#   - All crate Cargo.toml files discovered under the workspace
#   - README.md version references (badge URLs, code snippets)
#
# Idempotent: running twice with the same version is a no-op.

set -euo pipefail

# ---------------------------------------------------------------------------
# Color helpers (TTY-aware)
# ---------------------------------------------------------------------------
if [[ -t 1 ]]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  CYAN='\033[0;36m'
  BOLD='\033[1m'
  RESET='\033[0m'
else
  RED='' GREEN='' YELLOW='' CYAN='' BOLD='' RESET=''
fi

info()    { echo -e "${CYAN}[bump-version]${RESET} $*"; }
success() { echo -e "${GREEN}[bump-version]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[bump-version] WARN:${RESET} $*" >&2; }
error()   { echo -e "${RED}[bump-version] ERROR:${RESET} $*" >&2; }
die()     { error "$*"; exit 1; }

# ---------------------------------------------------------------------------
# Usage
# ---------------------------------------------------------------------------
usage() {
  cat << EOF
${BOLD}Usage:${RESET}
  $(basename "$0") <new-version>

${BOLD}Arguments:${RESET}
  <new-version>   Target semver version, e.g. 1.2.3 (without leading 'v')

${BOLD}Examples:${RESET}
  $(basename "$0") 1.2.3
  $(basename "$0") 0.5.0-beta.1

${BOLD}What it updates:${RESET}
  - Root Cargo.toml (workspace version)
  - All member crate Cargo.toml files
  - README.md version references (badges, code examples)
EOF
}

if [[ $# -eq 0 ]] || [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
  usage
  exit 0
fi

NEW_VERSION="$1"

# Validate semver format (allow pre-release labels)
if [[ ! "${NEW_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9._-]+)?(\+[a-zA-Z0-9._-]+)?$ ]]; then
  die "Invalid version '${NEW_VERSION}'. Expected semver format: X.Y.Z or X.Y.Z-label"
fi

# ---------------------------------------------------------------------------
# Locate repo root
# ---------------------------------------------------------------------------
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "${REPO_ROOT}"

# ---------------------------------------------------------------------------
# Platform-compatible sed in-place
# ---------------------------------------------------------------------------
# GNU sed: sed -i 's/.../.../g' file
# BSD/macOS sed: sed -i '' 's/.../.../g' file
sed_inplace() {
  local pattern="$1"
  local file="$2"
  if sed --version 2>&1 | grep -q GNU; then
    sed -i "${pattern}" "${file}"
  else
    sed -i '' "${pattern}" "${file}"
  fi
}

# ---------------------------------------------------------------------------
# Detect current version from root Cargo.toml
# ---------------------------------------------------------------------------
ROOT_CARGO="${REPO_ROOT}/Cargo.toml"
[[ -f "${ROOT_CARGO}" ]] || die "No Cargo.toml found at ${ROOT_CARGO}"

CURRENT_VERSION=$(grep -m1 '^version\s*=' "${ROOT_CARGO}" \
  | sed 's/.*=\s*"\(.*\)"/\1/' | tr -d '[:space:]')

if [[ -z "${CURRENT_VERSION}" ]]; then
  die "Could not determine current version from ${ROOT_CARGO}"
fi

if [[ "${CURRENT_VERSION}" == "${NEW_VERSION}" ]]; then
  info "Version is already ${NEW_VERSION} — nothing to do."
  exit 0
fi

info "Bumping version: ${BOLD}${CURRENT_VERSION}${RESET} -> ${BOLD}${NEW_VERSION}${RESET}"

# ---------------------------------------------------------------------------
# Collect all Cargo.toml files in the workspace
# ---------------------------------------------------------------------------
mapfile -t CARGO_FILES < <(find "${REPO_ROOT}" \
  -name 'Cargo.toml' \
  -not -path '*/target/*' \
  -not -path '*/.git/*' \
  | sort)

info "Found ${#CARGO_FILES[@]} Cargo.toml file(s):"
for f in "${CARGO_FILES[@]}"; do
  echo "    ${f#"${REPO_ROOT}/"}"
done

# ---------------------------------------------------------------------------
# Update version in each Cargo.toml
# ---------------------------------------------------------------------------
UPDATED_COUNT=0

for cargo_file in "${CARGO_FILES[@]}"; do
  # Only update the version = "X.Y.Z" line directly under [package] or
  # [workspace.package]. We use a targeted pattern that matches the exact
  # current version string to avoid touching dependency version pins.
  if grep -q "^version\s*=\s*\"${CURRENT_VERSION}\"" "${cargo_file}"; then
    sed_inplace "s/^version\s*=\s*\"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" \
      "${cargo_file}"
    success "  Updated: ${cargo_file#"${REPO_ROOT}/"}"
    (( UPDATED_COUNT++ )) || true
  else
    warn "  Skipped (version not found or already updated): ${cargo_file#"${REPO_ROOT}/"}"
  fi
done

# ---------------------------------------------------------------------------
# Update README.md version references
# ---------------------------------------------------------------------------
README="${REPO_ROOT}/README.md"

if [[ -f "${README}" ]]; then
  README_CHANGED=0

  # Pattern 1: cargo add crate@X.Y.Z
  if grep -q "@${CURRENT_VERSION}" "${README}"; then
    sed_inplace "s/@${CURRENT_VERSION}/@${NEW_VERSION}/g" "${README}"
    README_CHANGED=1
  fi

  # Pattern 2: version = "X.Y.Z" in Cargo.toml snippets
  if grep -q "\"${CURRENT_VERSION}\"" "${README}"; then
    sed_inplace "s/\"${CURRENT_VERSION}\"/\"${NEW_VERSION}\"/g" "${README}"
    README_CHANGED=1
  fi

  # Pattern 3: badge URLs containing /v/X.Y.Z or /badge/X.Y.Z or similar
  if grep -qE "v${CURRENT_VERSION//./\\.}" "${README}"; then
    sed_inplace "s/v${CURRENT_VERSION//./\\.}/v${NEW_VERSION}/g" "${README}"
    README_CHANGED=1
  fi

  if [[ "${README_CHANGED}" -eq 1 ]]; then
    success "  Updated: README.md"
  else
    info "  README.md: no version references matching ${CURRENT_VERSION} found, skipped."
  fi
else
  warn "README.md not found at ${README}, skipped."
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
success "Done. Updated ${UPDATED_COUNT} Cargo.toml file(s) from ${CURRENT_VERSION} to ${NEW_VERSION}."

# Remind the caller to run cargo check so the lockfile refreshes
info "Tip: run 'cargo check' to refresh Cargo.lock with the new version."
