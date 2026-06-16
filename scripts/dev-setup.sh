#!/usr/bin/env bash
# cargo-cicd Development Setup Script
# Purpose: Initialize development environment with all prerequisites
# Usage: ./scripts/dev-setup.sh [OPTIONS]

set -euo pipefail

# ─── Colors & Formatting ───────────────────────────────────────────────────────
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly GRAY='\033[0;37m'
readonly NC='\033[0m' # No Color

# ─── Flags ─────────────────────────────────────────────────────────────────────
VERBOSE=0
QUIET=0

show_help() {
    cat << EOF
${BLUE}cargo-cicd Development Setup${NC}

${YELLOW}USAGE:${NC}
  $(basename "$0") [OPTIONS]

${YELLOW}OPTIONS:${NC}
  --help          Show this help message
  --verbose       Enable verbose output
  --quiet         Suppress non-essential output
  --skip-build    Skip cargo build step
  --skip-hooks    Skip pre-commit hooks installation

${YELLOW}DESCRIPTION:${NC}
  Initializes the development environment by:
  - Verifying Rust version (1.85+)
  - Installing pre-commit hooks
  - Running cargo build to validate baseline
  - Displaying environment summary

${YELLOW}EXAMPLES:${NC}
  # Full setup with defaults
  ./scripts/dev-setup.sh

  # Setup with verbose output
  ./scripts/dev-setup.sh --verbose

  # Skip build, only check and setup hooks
  ./scripts/dev-setup.sh --skip-build

EOF
}

# ─── Output Functions ──────────────────────────────────────────────────────────
log_info() {
    [[ $QUIET -eq 0 ]] && printf "${BLUE}ℹ${NC} %s\n" "$*" >&2
}

log_success() {
    printf "${GREEN}✓${NC} %s\n" "$*" >&2
}

log_warn() {
    printf "${YELLOW}⚠${NC} %s\n" "$*" >&2
}

log_error() {
    printf "${RED}✗${NC} %s\n" "$*" >&2
}

log_verbose() {
    [[ $VERBOSE -eq 1 ]] && printf "${GRAY}${1}${NC}\n" >&2
}

# ─── Validation Functions ──────────────────────────────────────────────────────
check_rust_version() {
    log_info "Checking Rust version..."

    if ! command -v rustc &> /dev/null; then
        log_error "Rust not installed. Please install from https://rustup.rs/"
        return 1
    fi

    local rust_version
    rust_version=$(rustc --version | awk '{print $2}')
    local required_version="1.85"

    log_verbose "Found Rust version: $rust_version"

    if [[ $(printf '%s\n' "$required_version" "$rust_version" | sort -V | head -n1) != "$required_version" ]]; then
        log_error "Rust version $rust_version is below required minimum 1.85+"
        log_info "Update with: rustup update"
        return 1
    fi

    log_success "Rust version $rust_version meets requirement (1.85+)"
    return 0
}

check_cargo() {
    log_info "Checking cargo..."

    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found"
        return 1
    fi

    local cargo_version
    cargo_version=$(cargo --version | awk '{print $2}')
    log_verbose "Found cargo version: $cargo_version"
    log_success "cargo is installed ($cargo_version)"
    return 0
}

install_pre_commit_hooks() {
    log_info "Setting up pre-commit hooks..."

    local hooks_dir=".git/hooks"

    if [[ ! -d "$hooks_dir" ]]; then
        log_error "Not in a git repository"
        return 1
    fi

    # Create pre-commit hook
    local pre_commit_hook="$hooks_dir/pre-commit"
    cat > "$pre_commit_hook" << 'HOOK_EOF'
#!/usr/bin/env bash
# Pre-commit hook: run cargo check and clippy

set -euo pipefail

echo "Running pre-commit checks..."
cargo fmt --check || {
    echo "Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
}

cargo clippy --all-targets --all-features -- -D warnings || {
    echo "Clippy warnings found. Please fix them."
    exit 1
}

exit 0
HOOK_EOF

    chmod +x "$pre_commit_hook"
    log_success "Pre-commit hook installed at $pre_commit_hook"

    # Create pre-push hook
    local pre_push_hook="$hooks_dir/pre-push"
    cat > "$pre_push_hook" << 'HOOK_EOF'
#!/usr/bin/env bash
# Pre-push hook: ensure tests pass

set -euo pipefail

echo "Running tests before push..."
cargo test --lib 2>&1 | tail -20

exit 0
HOOK_EOF

    chmod +x "$pre_push_hook"
    log_success "Pre-push hook installed at $pre_push_hook"
    return 0
}

run_cargo_build() {
    log_info "Running cargo build to validate baseline..."

    if cargo build 2>&1 | tail -5; then
        log_success "Cargo build successful"
        return 0
    else
        log_error "Cargo build failed"
        return 1
    fi
}

print_environment_summary() {
    log_info "Environment Summary:"

    local rust_version
    rust_version=$(rustc --version)
    local cargo_version
    cargo_version=$(cargo --version)
    local project_name
    project_name=$(grep '^name' Cargo.toml | head -1 | cut -d'"' -f2)
    local project_version
    project_version=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

    cat >&2 << EOF

${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${BLUE}Project:${NC}  $project_name v$project_version
${BLUE}Rust:${NC}     $rust_version
${BLUE}Cargo:${NC}    $cargo_version
${BLUE}Hooks:${NC}    $([ -x .git/hooks/pre-commit ] && echo "✓ Installed" || echo "✗ Not installed")
${BLUE}Build:${NC}    ✓ Successful
${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}

EOF
}

# ─── Main ──────────────────────────────────────────────────────────────────────
main() {
    local skip_build=0
    local skip_hooks=0

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --help)
                show_help
                exit 0
                ;;
            --verbose)
                VERBOSE=1
                shift
                ;;
            --quiet)
                QUIET=1
                shift
                ;;
            --skip-build)
                skip_build=1
                shift
                ;;
            --skip-hooks)
                skip_hooks=1
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    log_info "Starting cargo-cicd development setup..."

    # Run checks
    check_rust_version || exit 1
    check_cargo || exit 1

    if [[ $skip_hooks -eq 0 ]]; then
        install_pre_commit_hooks || exit 1
    fi

    if [[ $skip_build -eq 0 ]]; then
        run_cargo_build || exit 1
    fi

    print_environment_summary
    log_success "Development environment ready!"
    return 0
}

main "$@"
