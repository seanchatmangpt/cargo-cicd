#!/usr/bin/env bash
# release.sh — End-to-end release automation for a Rust workspace
#
# Usage:
#   ./scripts/release.sh [major|minor|patch] [OPTIONS]
#
# Options:
#   major|minor|patch    Version bump type (required unless --version is used)
#   --version <X.Y.Z>    Explicit target version (skips bump calculation)
#   --dry-run            Show what would happen; make no changes
#   --yes                Skip interactive confirmation prompts
#   --no-push            Commit and tag locally; do not push
#   --help               Show this help
#
# Workflow:
#   1. Validates working tree is clean
#   2. Runs full test suite
#   3. Bumps version in all Cargo.toml files
#   4. Updates CHANGELOG.md with conventional commits since last tag
#   5. Creates commit "chore(release): vX.Y.Z"
#   6. Creates annotated tag vX.Y.Z
#   7. Pushes commit and tag to origin

set -euo pipefail

# ---------------------------------------------------------------------------
# Color helpers (TTY-aware)
# ---------------------------------------------------------------------------
if [[ -t 1 ]]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  CYAN='\033[0;36m'
  MAGENTA='\033[0;35m'
  BOLD='\033[1m'
  DIM='\033[2m'
  RESET='\033[0m'
else
  RED='' GREEN='' YELLOW='' CYAN='' MAGENTA='' BOLD='' DIM='' RESET=''
fi

info()    { echo -e "${CYAN}[release]${RESET} $*"; }
step()    { echo -e "\n${BOLD}${MAGENTA}==>${RESET}${BOLD} $*${RESET}"; }
success() { echo -e "${GREEN}[release]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[release] WARN:${RESET} $*" >&2; }
error()   { echo -e "${RED}[release] ERROR:${RESET} $*" >&2; }
die()     { error "$*"; exit 1; }
dry()     { echo -e "${DIM}[dry-run]${RESET} $*"; }

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
BUMP_TYPE=""
EXPLICIT_VERSION=""
DRY_RUN=false
YES=false
NO_PUSH=false

usage() {
  cat << EOF
${BOLD}Usage:${RESET}
  $(basename "$0") [major|minor|patch] [OPTIONS]
  $(basename "$0") --version X.Y.Z [OPTIONS]

${BOLD}Bump types:${RESET}
  major    Bump X in X.Y.Z  (breaking changes)
  minor    Bump Y in X.Y.Z  (new features)
  patch    Bump Z in X.Y.Z  (bug fixes)

${BOLD}Options:${RESET}
  --version <X.Y.Z>   Use an explicit version instead of bumping
  --dry-run           Show planned changes without making them
  --yes               Skip all confirmation prompts
  --no-push           Tag locally but do not push to remote
  -h, --help          Show this help

${BOLD}Examples:${RESET}
  $(basename "$0") patch
  $(basename "$0") minor --dry-run
  $(basename "$0") major --yes
  $(basename "$0") --version 2.0.0-beta.1 --no-push
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    major|minor|patch)
      BUMP_TYPE="$1"
      shift
      ;;
    --version)
      [[ $# -ge 2 ]] || die "--version requires an argument"
      EXPLICIT_VERSION="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --yes|-y)
      YES=true
      shift
      ;;
    --no-push)
      NO_PUSH=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "Unknown argument: $1. Run with --help for usage."
      ;;
  esac
done

# Must have either a bump type or an explicit version
if [[ -z "${BUMP_TYPE}" ]] && [[ -z "${EXPLICIT_VERSION}" ]]; then
  usage
  echo ""
  die "Either a bump type (major|minor|patch) or --version X.Y.Z is required."
fi

if [[ -n "${BUMP_TYPE}" ]] && [[ -n "${EXPLICIT_VERSION}" ]]; then
  die "Cannot use both a bump type and --version at the same time."
fi

# ---------------------------------------------------------------------------
# Locate repo root
# ---------------------------------------------------------------------------
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" \
  || die "Not inside a git repository."
cd "${REPO_ROOT}"

SCRIPTS_DIR="${REPO_ROOT}/scripts"

# ---------------------------------------------------------------------------
# Platform-compatible sed
# ---------------------------------------------------------------------------
sed_inplace() {
  local pattern="$1"
  local file="$2"
  if sed --version 2>&1 | grep -q GNU 2>/dev/null; then
    sed -i "${pattern}" "${file}"
  else
    sed -i '' "${pattern}" "${file}"
  fi
}

# ---------------------------------------------------------------------------
# Read current version from root Cargo.toml
# ---------------------------------------------------------------------------
ROOT_CARGO="${REPO_ROOT}/Cargo.toml"
[[ -f "${ROOT_CARGO}" ]] || die "No Cargo.toml found at ${ROOT_CARGO}"

CURRENT_VERSION=$(grep -m1 '^version\s*=' "${ROOT_CARGO}" \
  | sed 's/.*=\s*"\(.*\)"/\1/' | tr -d '[:space:]')
[[ -n "${CURRENT_VERSION}" ]] || die "Could not parse current version from ${ROOT_CARGO}"

# ---------------------------------------------------------------------------
# Calculate new version
# ---------------------------------------------------------------------------
semver_bump() {
  local version="$1"
  local bump="$2"

  # Strip any pre-release label for arithmetic
  local core="${version%%-*}"
  IFS='.' read -r MAJOR MINOR PATCH <<< "${core}"

  case "${bump}" in
    major) MAJOR=$(( MAJOR + 1 )); MINOR=0; PATCH=0 ;;
    minor) MINOR=$(( MINOR + 1 )); PATCH=0 ;;
    patch) PATCH=$(( PATCH + 1 )) ;;
  esac

  echo "${MAJOR}.${MINOR}.${PATCH}"
}

if [[ -n "${EXPLICIT_VERSION}" ]]; then
  NEW_VERSION="${EXPLICIT_VERSION}"
else
  NEW_VERSION=$(semver_bump "${CURRENT_VERSION}" "${BUMP_TYPE}")
fi

# Validate result
if [[ ! "${NEW_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9._-]+)?$ ]]; then
  die "Computed version '${NEW_VERSION}' is not valid semver."
fi

echo ""
echo -e "${BOLD}Release Plan${RESET}"
echo "  Current version : ${CURRENT_VERSION}"
echo "  New version     : ${BOLD}${NEW_VERSION}${RESET}"
echo "  Tag             : v${NEW_VERSION}"
echo "  Dry run         : ${DRY_RUN}"
echo "  Push            : $(if ${NO_PUSH}; then echo "no (--no-push)"; else echo "yes"; fi)"
echo ""

# ---------------------------------------------------------------------------
# STEP 1: Validate working tree is clean
# ---------------------------------------------------------------------------
step "1/7 — Checking working tree"

DIRTY=$(git status --porcelain 2>/dev/null)
if [[ -n "${DIRTY}" ]]; then
  error "Working tree is dirty. Commit or stash your changes first."
  echo ""
  git status --short
  echo ""
  die "Refusing to release from a dirty working tree."
fi
success "Working tree is clean."

# ---------------------------------------------------------------------------
# STEP 2: Run full test suite
# ---------------------------------------------------------------------------
step "2/7 — Running test suite"

if ${DRY_RUN}; then
  dry "Would run: cargo test --all-features --workspace"
else
  info "Running: cargo test --all-features --workspace"
  cargo test --all-features --workspace || die "Tests failed. Refusing to release."
  success "All tests passed."
fi

# ---------------------------------------------------------------------------
# STEP 3: Bump version
# ---------------------------------------------------------------------------
step "3/7 — Bumping version in Cargo.toml files"

if ${DRY_RUN}; then
  dry "Would run: ${SCRIPTS_DIR}/bump-version.sh ${NEW_VERSION}"
  dry "  Root Cargo.toml: ${CURRENT_VERSION} -> ${NEW_VERSION}"
  # Show what would be touched
  while IFS= read -r f; do
    dry "  ${f#"${REPO_ROOT}/"}"
  done < <(find "${REPO_ROOT}" -name 'Cargo.toml' \
    -not -path '*/target/*' -not -path '*/.git/*' | sort)
else
  [[ -x "${SCRIPTS_DIR}/bump-version.sh" ]] \
    || chmod +x "${SCRIPTS_DIR}/bump-version.sh"
  "${SCRIPTS_DIR}/bump-version.sh" "${NEW_VERSION}"

  # Refresh Cargo.lock
  info "Refreshing Cargo.lock..."
  cargo check --quiet 2>/dev/null || true
  success "Version bumped to ${NEW_VERSION}."
fi

# ---------------------------------------------------------------------------
# STEP 4: Update CHANGELOG.md
# ---------------------------------------------------------------------------
step "4/7 — Updating CHANGELOG.md"

CHANGELOG="${REPO_ROOT}/CHANGELOG.md"
TODAY=$(date +%Y-%m-%d)

# Get the last tag to know the range for new commits
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

build_changelog_section() {
  local version="$1"
  local date="$2"
  local last_tag="$3"

  local log_range=""
  if [[ -n "${last_tag}" ]]; then
    log_range="${last_tag}..HEAD"
  else
    log_range="HEAD"
  fi

  echo "## [${version}] - ${date}"

  # Categorise by conventional commit type
  local -A CATEGORIES=(
    [feat]="Added"
    [fix]="Fixed"
    [perf]="Changed"
    [refactor]="Changed"
    [docs]="Changed"
    [style]="Changed"
    [test]="Changed"
    [build]="Changed"
    [ci]="Changed"
    [chore]="Changed"
    [revert]="Removed"
    [security]="Security"
    [deps]="Changed"
  )

  declare -A SECTION_LINES

  # Store pattern in a variable — bash [[ =~ ]] requires the pattern not to be
  # a bare literal when it contains parentheses, to avoid parser ambiguity.
  local CONV_PAT
  CONV_PAT='^(feat|fix|perf|refactor|docs|style|test|build|ci|chore|revert|security|deps)(\([^)]+\))?(!)?: '

  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    # Parse: <type>(<scope>): description  or  <type>: description
    if [[ "${line}" =~ ${CONV_PAT} ]]; then
      local type="${BASH_REMATCH[1]}"
      local scope="${BASH_REMATCH[2]}"
      local breaking="${BASH_REMATCH[3]}"
      # Description is everything after the ": "
      local desc="${line#*: }"
      local section="${CATEGORIES[${type}]:-Changed}"
      if [[ -n "${breaking}" ]]; then
        section="Changed"
        desc="**BREAKING** ${desc}"
      fi
      if [[ -n "${scope}" ]]; then
        SECTION_LINES[${section}]+="- ${scope#(}: ${desc}"$'\n'
      else
        SECTION_LINES[${section}]+="- ${desc}"$'\n'
      fi
    fi
  done < <(git log --pretty=format:"%s" ${log_range} 2>/dev/null || true)

  for section in "Added" "Changed" "Deprecated" "Removed" "Fixed" "Security"; do
    if [[ -n "${SECTION_LINES[${section}]:-}" ]]; then
      echo "### ${section}"
      echo -n "${SECTION_LINES[${section}]}"
      echo ""
    fi
  done
}

if ${DRY_RUN}; then
  dry "Would prepend to CHANGELOG.md:"
  dry "  ## [${NEW_VERSION}] - ${TODAY}"
  if [[ -n "${LAST_TAG}" ]]; then
    dry "  (commits since ${LAST_TAG})"
  else
    dry "  (all commits — no previous tag found)"
  fi
else
  NEW_SECTION=$(build_changelog_section "${NEW_VERSION}" "${TODAY}" "${LAST_TAG}")

  if [[ -f "${CHANGELOG}" ]]; then
    # Insert after the "## [Unreleased]" block (or after the header if none)
    # Strategy: find the first "## [" line that is not "[Unreleased]" and insert before it
    TMP_CHANGELOG=$(mktemp)

    awk -v new_section="${NEW_SECTION}" '
      /^## \[Unreleased\]/ {
        print                        # print [Unreleased] header
        in_unreleased = 1
        next
      }
      in_unreleased && /^## \[/ {
        # We hit the next version block; emit our new section first
        print new_section
        in_unreleased = 0
      }
      { print }
      END {
        # If file had only [Unreleased] and nothing else, append at end
        if (in_unreleased) {
          print ""
          print new_section
        }
      }
    ' "${CHANGELOG}" > "${TMP_CHANGELOG}"

    # If the file has no [Unreleased] section, just prepend after the title
    if ! grep -q '## \[Unreleased\]' "${TMP_CHANGELOG}" 2>/dev/null; then
      {
        head -n 5 "${CHANGELOG}"
        echo ""
        echo "${NEW_SECTION}"
        tail -n +6 "${CHANGELOG}"
      } > "${TMP_CHANGELOG}"
    fi

    mv "${TMP_CHANGELOG}" "${CHANGELOG}"
  else
    # Create a minimal CHANGELOG if none exists
    cat > "${CHANGELOG}" << EOF
# Changelog
All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

${NEW_SECTION}
EOF
  fi

  success "CHANGELOG.md updated for v${NEW_VERSION}."
fi

# ---------------------------------------------------------------------------
# STEP 5: Confirm before tagging
# ---------------------------------------------------------------------------
step "5/7 — Confirmation"

if ${DRY_RUN}; then
  dry "Would create commit: chore(release): v${NEW_VERSION}"
  dry "Would create annotated tag: v${NEW_VERSION}"
  if ! ${NO_PUSH}; then
    dry "Would push to origin: commit + tag v${NEW_VERSION}"
  fi
  echo ""
  success "Dry run complete. No changes were made."
  exit 0
fi

if ! ${YES}; then
  echo ""
  echo -e "${BOLD}Ready to release v${NEW_VERSION}.${RESET}"
  echo "  This will:"
  echo "    - Commit: chore(release): v${NEW_VERSION}"
  echo "    - Tag:    v${NEW_VERSION}"
  if ! ${NO_PUSH}; then
    echo "    - Push commit and tag to origin"
  fi
  echo ""
  read -r -p "Continue? [y/N] " CONFIRM
  case "${CONFIRM}" in
    y|Y|yes|YES) ;;
    *) warn "Aborted by user."; exit 0 ;;
  esac
fi

# ---------------------------------------------------------------------------
# STEP 6: Git commit
# ---------------------------------------------------------------------------
step "6/7 — Creating release commit"

git add Cargo.toml Cargo.lock "${CHANGELOG}"
# Also stage any member crate Cargo.toml files
while IFS= read -r f; do
  git add "${f}" 2>/dev/null || true
done < <(find "${REPO_ROOT}" -name 'Cargo.toml' \
  -not -path '*/target/*' -not -path '*/.git/*')

# Also stage README.md if it was updated
[[ -f "${REPO_ROOT}/README.md" ]] && git add "${REPO_ROOT}/README.md" 2>/dev/null || true

git commit -m "chore(release): v${NEW_VERSION}

Bump version from ${CURRENT_VERSION} to ${NEW_VERSION}.
Update CHANGELOG.md with conventional commits since ${LAST_TAG:-initial commit}."

success "Release commit created."

# ---------------------------------------------------------------------------
# STEP 7: Annotated tag
# ---------------------------------------------------------------------------
step "7/7 — Creating annotated tag"

TAG_BODY="Release v${NEW_VERSION}"

# Include a brief one-liner from the changelog if available
if [[ -f "${CHANGELOG}" ]]; then
  FIRST_ITEM=$(awk "/^## \[${NEW_VERSION}\]/{found=1; next} found && /^### /{next} found && /^- /{print; exit} found && /^## \[/{exit}" "${CHANGELOG}" || true)
  if [[ -n "${FIRST_ITEM}" ]]; then
    TAG_BODY="${TAG_BODY}

${FIRST_ITEM}"
  fi
fi

git tag -a "v${NEW_VERSION}" -m "${TAG_BODY}"
success "Tag v${NEW_VERSION} created."

# ---------------------------------------------------------------------------
# Push
# ---------------------------------------------------------------------------
if ! ${NO_PUSH}; then
  info "Pushing commit and tag to origin..."
  git push origin HEAD
  git push origin "v${NEW_VERSION}"
  success "Pushed to origin."
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
echo ""
echo -e "${BOLD}${GREEN}Release v${NEW_VERSION} complete!${RESET}"
echo ""
echo "  Tag:    v${NEW_VERSION}"
echo "  Commit: $(git rev-parse --short HEAD)"
if ! ${NO_PUSH}; then
  REMOTE_URL=$(git remote get-url origin 2>/dev/null || echo "unknown")
  echo "  Remote: ${REMOTE_URL}"
fi
echo ""
echo "Next steps:"
echo "  - Watch the release workflow in GitHub Actions"
echo "  - Verify binaries are attached to the GitHub Release"
echo "  - Announce the release"
