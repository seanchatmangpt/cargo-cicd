#!/bin/bash

# setup-hooks.sh — Install and configure git pre-commit hooks for cargo-cicd
# Usage: ./setup-hooks.sh [--uninstall]

set -e

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Helper functions
header() { echo -e "${BLUE}▶ $1${NC}"; }
success() { echo -e "${GREEN}✓ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; }
info() { echo "  $1"; }

# ============================================================================
# Uninstall mode
# ============================================================================
if [ "$1" == "--uninstall" ]; then
    header "Uninstalling hooks"

    if [ -f "$HOOKS_DIR/pre-commit" ]; then
        rm "$HOOKS_DIR/pre-commit"
        success "Removed .git/hooks/pre-commit"
    fi

    if [ -f "$REPO_ROOT/.pre-commit-config.yaml" ]; then
        echo -n "Remove .pre-commit-config.yaml? (y/n): "
        read -r REMOVE_CONFIG
        if [ "$REMOVE_CONFIG" = "y" ]; then
            rm "$REPO_ROOT/.pre-commit-config.yaml"
            success "Removed .pre-commit-config.yaml"
        fi
    fi

    if command -v pre-commit &> /dev/null; then
        echo -n "Uninstall pre-commit framework? (y/n): "
        read -r UNINSTALL_FRAMEWORK
        if [ "$UNINSTALL_FRAMEWORK" = "y" ]; then
            pre-commit uninstall
            success "Pre-commit framework uninstalled"
        fi
    fi

    echo ""
    success "Hooks uninstalled. You can commit without restrictions."
    exit 0
fi

# ============================================================================
# Install mode
# ============================================================================
header "Installing cargo-cicd hooks"
echo ""

# Check if we're in a git repo
if [ ! -d "$REPO_ROOT/.git" ]; then
    error "Not a git repository. Run this script from a git root or checkout."
    exit 1
fi

success "Git repository detected at $REPO_ROOT"

# ============================================================================
# 1. Install the main pre-commit hook
# ============================================================================
header "Installing .git/hooks/pre-commit"

if [ ! -d "$HOOKS_DIR" ]; then
    mkdir -p "$HOOKS_DIR"
    info "Created hooks directory"
fi

if [ -f "$SCRIPT_DIR/.git/hooks/pre-commit" ]; then
    cp "$SCRIPT_DIR/.git/hooks/pre-commit" "$HOOKS_DIR/pre-commit"
    chmod +x "$HOOKS_DIR/pre-commit"
    success "Installed .git/hooks/pre-commit (executable)"
else
    error "Hook file not found at $SCRIPT_DIR/.git/hooks/pre-commit"
    info "Make sure you're running this from the repository root."
    exit 1
fi

# ============================================================================
# 2. Install the pre-commit framework config (optional)
# ============================================================================
header "Setting up pre-commit framework"

if [ ! -f "$REPO_ROOT/.pre-commit-config.yaml" ]; then
    if [ -f "$SCRIPT_DIR/.pre-commit-config.yaml" ]; then
        cp "$SCRIPT_DIR/.pre-commit-config.yaml" "$REPO_ROOT/.pre-commit-config.yaml"
        success "Installed .pre-commit-config.yaml"
    else
        warn "Could not find .pre-commit-config.yaml template"
    fi
else
    info ".pre-commit-config.yaml already exists (keeping existing)"
fi

if command -v pre-commit &> /dev/null; then
    info "pre-commit framework detected"
    echo -n "Install pre-commit hook environments? (y/n): "
    read -r INSTALL_ENVS
    if [ "$INSTALL_ENVS" = "y" ]; then
        pre-commit install
        success "Pre-commit environments installed"
        pre-commit run --all-files || warn "Some pre-commit checks failed on existing files (OK for first run)"
    fi
else
    warn "pre-commit framework not installed"
    info "Optional: Install with: pip install pre-commit"
    info "Then run:  pre-commit install"
fi

# ============================================================================
# 3. Create forbidden-terms checker script (referenced by pre-commit-config.yaml)
# ============================================================================
header "Setting up forbidden-terms checker"

SCRIPTS_DIR="$REPO_ROOT/scripts"
if [ ! -d "$SCRIPTS_DIR" ]; then
    mkdir -p "$SCRIPTS_DIR"
    info "Created scripts directory"
fi

cat > "$SCRIPTS_DIR/check-forbidden-terms.sh" << 'EOF'
#!/bin/bash
# Check for forbidden terms in staged files
# Part of cargo-cicd pre-commit framework

FORBIDDEN_TERMS=(
    "ALIVE"
    "Inspection Gate"
    "wall"
    "Nehemiah"
    "Field8"
    "Instinct8"
    "Cargo Court"
    "AGI"
    "Truex"
    "CONSTRUCT8"
)

ERROR=0
for FILE in "$@"; do
    [ ! -f "$FILE" ] && continue
    for TERM in "${FORBIDDEN_TERMS[@]}"; do
        if grep -iq "$TERM" "$FILE" 2>/dev/null; then
            echo "✗ $FILE: Found forbidden term '$TERM'"
            ERROR=1
        fi
    done
done

exit $ERROR
EOF

chmod +x "$SCRIPTS_DIR/check-forbidden-terms.sh"
success "Created scripts/check-forbidden-terms.sh"

# ============================================================================
# 4. Test the hooks
# ============================================================================
header "Testing hooks"

echo ""
info "Running a dry-run of the pre-commit hook..."

if bash "$HOOKS_DIR/pre-commit" /dev/null 2>&1 | head -10; then
    success "Hook executable and functional"
else
    warn "Hook test produced output (this may be normal)"
fi

# ============================================================================
# 5. Summary
# ============================================================================
echo ""
header "Hook installation complete!"
echo ""
echo "Installed hooks:"
info "✓ .git/hooks/pre-commit — Runs before every commit"
info "  Checks: formatting, compilation, tests, forbidden terms, commit format"
echo ""
echo "Optional additions:"
info "• .pre-commit-config.yaml — Framework config (pre-commit run --all-files)"
info "• scripts/check-forbidden-terms.sh — Forbidden term checker"
echo ""
echo "Next steps:"
info "1. Make a test commit: git add . && git commit -m 'test(core): validate hooks'"
info "2. Verify checks run and pass"
info "3. (Optional) Install pre-commit framework: pip install pre-commit && pre-commit install"
echo ""
echo "Disable hooks temporarily:"
info "  SKIP=pre-commit git commit ..."
echo ""
echo "Uninstall all hooks:"
info "  $0 --uninstall"
echo ""
success "Ready to enforce code quality!"
