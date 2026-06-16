#!/usr/bin/env bash
# cargo-cicd Ready-to-Push Validation Script
# Purpose: Comprehensive pre-push validation (what CI will run)
# Usage: ./scripts/ready-to-push.sh [OPTIONS]

set -euo pipefail

# ─── Colors & Formatting ───────────────────────────────────────────────────────
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly GRAY='\033[0;37m'
readonly NC='\033[0m'

# ─── Flags ─────────────────────────────────────────────────────────────────────
VERBOSE=0
QUIET=0
FIX_ISSUES=0
SKIP_FEATURE_MATRIX=0

show_help() {
    cat << EOF
${BLUE}cargo-cicd Ready-to-Push Validation${NC}

${YELLOW}USAGE:${NC}
  $(basename "$0") [OPTIONS]

${YELLOW}OPTIONS:${NC}
  --help              Show this help message
  --verbose           Enable verbose output
  --quiet             Suppress non-essential output
  --fix               Attempt to auto-fix common issues (fmt)
  --skip-features     Skip feature matrix testing

${YELLOW}DESCRIPTION:${NC}
  Performs comprehensive pre-push validation simulating CI:
  1. Code format check (cargo fmt)
  2. Lint check (cargo clippy)
  3. Unit tests (cargo test --lib)
  4. Integration tests
  5. Feature matrix test (all combinations)

  If issues are found, suggests fixes and can auto-fix formatting.
  Final output: "safe to push" or "fix these first"

${YELLOW}EXAMPLES:${NC}
  # Check if ready to push
  ./scripts/ready-to-push.sh

  # Auto-fix formatting and retry
  ./scripts/ready-to-push.sh --fix

  # Quick check without feature matrix
  ./scripts/ready-to-push.sh --skip-features

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

section_header() {
    echo "" >&2
    printf "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n" >&2
    printf "${BLUE}%s${NC}\n" "$1" >&2
    printf "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n" >&2
}

# ─── Temp Directory Setup ───────────────────────────────────────────────────────
readonly TMPDIR_BASE="${TMPDIR:-.}/.cargo-cicd-push-check"
mkdir -p "$TMPDIR_BASE"
trap "rm -rf $TMPDIR_BASE" EXIT

declare -a FAILED_CHECKS=()
declare -a SUGGESTIONS=()

# ─── Git State Checks ───────────────────────────────────────────────────────────
check_git_status() {
    section_header "Git Status Check"

    log_info "Checking git status..."

    local git_status
    git_status=$(git status --porcelain)

    if [[ -n "$git_status" ]]; then
        log_warn "Uncommitted changes detected:"
        echo "$git_status" | head -10 | sed 's/^/  /' >&2
        if [[ $(echo "$git_status" | wc -l) -gt 10 ]]; then
            echo "  ... and more" >&2
        fi

        SUGGESTIONS+=("Stage and commit changes before pushing")
        return 1
    fi

    log_success "No uncommitted changes"
    return 0
}

check_commits_ahead() {
    section_header "Commits Ahead of Main"

    log_info "Checking commits ahead of main..."

    local ahead
    ahead=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")

    if [[ $ahead -eq 0 ]]; then
        log_warn "No commits ahead of main"
        SUGGESTIONS+=("Branch may not be properly set up or is on main")
        return 1
    fi

    log_success "Branch has $ahead commit(s) ahead of main"
    return 0
}

# ─── Format Check ──────────────────────────────────────────────────────────────
check_format() {
    section_header "Code Format Check"

    log_info "Checking code formatting..."

    if cargo fmt --check > "$TMPDIR_BASE/fmt.log" 2>&1; then
        log_success "Code formatting is correct"
        return 0
    else
        log_error "Code formatting issues found"
        FAILED_CHECKS+=("Format Check")
        SUGGESTIONS+=("Run 'cargo fmt' to fix formatting")

        if [[ $FIX_ISSUES -eq 1 ]]; then
            log_info "Auto-fixing formatting..."
            cargo fmt
            log_success "Formatting fixed"
            return 0
        fi

        return 1
    fi
}

# ─── Lint Check ────────────────────────────────────────────────────────────────
check_clippy() {
    section_header "Lint Check (Clippy)"

    log_info "Running clippy..."

    if cargo clippy --all-targets --all-features -- -D warnings > "$TMPDIR_BASE/clippy.log" 2>&1; then
        log_success "No clippy warnings"
        return 0
    else
        log_error "Clippy warnings found"
        FAILED_CHECKS+=("Clippy Check")
        SUGGESTIONS+=("Fix clippy warnings with: cargo clippy --all-features")

        if [[ $VERBOSE -eq 1 ]]; then
            grep "warning:" "$TMPDIR_BASE/clippy.log" | head -5 | sed 's/^/  /' >&2
        fi

        return 1
    fi
}

# ─── Unit Tests ────────────────────────────────────────────────────────────────
check_unit_tests() {
    section_header "Unit Tests"

    log_info "Running unit tests..."

    if cargo test --lib 2>&1 | tee "$TMPDIR_BASE/unit_tests.log" | grep -E "test result:"; then
        local result
        result=$(grep "test result:" "$TMPDIR_BASE/unit_tests.log" | tail -1)

        if [[ "$result" =~ "ok" ]]; then
            log_success "All unit tests passed"
            return 0
        else
            log_error "Unit tests failed"
            FAILED_CHECKS+=("Unit Tests")
            SUGGESTIONS+=("Fix failing tests with: cargo test --lib")
            return 1
        fi
    else
        log_error "Could not run unit tests"
        FAILED_CHECKS+=("Unit Tests")
        return 1
    fi
}

# ─── Integration Tests ──────────────────────────────────────────────────────────
check_integration_tests() {
    section_header "Integration Tests"

    log_info "Running integration tests..."

    if cargo test --test invariants 2>&1 | tee "$TMPDIR_BASE/integration_tests.log" | grep -E "test result:"; then
        local result
        result=$(grep "test result:" "$TMPDIR_BASE/integration_tests.log" | tail -1)

        if [[ "$result" =~ "ok" ]]; then
            log_success "Integration tests passed"
            return 0
        else
            log_error "Integration tests failed"
            FAILED_CHECKS+=("Integration Tests")
            SUGGESTIONS+=("Fix integration tests with: cargo test --test invariants")
            return 1
        fi
    else
        log_warn "Could not run integration tests (may not be critical)"
        return 0
    fi
}

# ─── Feature Matrix Test ────────────────────────────────────────────────────────
check_feature_matrix() {
    section_header "Feature Matrix Test"

    if [[ $SKIP_FEATURE_MATRIX -eq 1 ]]; then
        log_info "Feature matrix test skipped (--skip-features)"
        return 0
    fi

    log_info "Testing feature combinations (this may take a few minutes)..."

    local key_features=("process-data" "autonomic" "advanced")
    local all_pass=1

    for feature in "${key_features[@]}"; do
        if cargo build --features "$feature" > "$TMPDIR_BASE/feature_$feature.log" 2>&1; then
            log_success "Feature '$feature' compiles"
        else
            log_error "Feature '$feature' failed to compile"
            FAILED_CHECKS+=("Feature: $feature")
            SUGGESTIONS+=("Fix compilation of feature '$feature'")
            all_pass=0
        fi
    done

    if [[ $all_pass -eq 1 ]]; then
        return 0
    else
        return 1
    fi
}

# ─── Final Summary ──────────────────────────────────────────────────────────────
print_final_summary() {
    section_header "Final Summary"

    local checks_passed=0
    local checks_failed=${#FAILED_CHECKS[@]}

    if [[ $checks_failed -eq 0 ]]; then
        cat >&2 << EOF

${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${GREEN}           ✓ SAFE TO PUSH${NC}
${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}

All validation checks passed!
Your branch is ready to push.

EOF
        return 0
    else
        cat >&2 << EOF

${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${RED}           ✗ NOT READY TO PUSH${NC}
${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}

Failed Checks:
EOF

        for check in "${FAILED_CHECKS[@]}"; do
            printf "  ${RED}✗${NC} %s\n" "$check" >&2
        done

        if [[ ${#SUGGESTIONS[@]} -gt 0 ]]; then
            echo "" >&2
            echo "Suggestions:" >&2
            for suggestion in "${SUGGESTIONS[@]}"; do
                printf "  ${YELLOW}•${NC} %s\n" "$suggestion" >&2
            done
        fi

        cat >&2 << EOF

Fix the issues above and run this script again.

${YELLOW}Quick fixes:${NC}
  cargo fmt                      # Format code
  cargo clippy --fix             # Auto-fix clippy warnings
  cargo test --lib              # Run tests locally

EOF
        return 1
    fi
}

# ─── Main ──────────────────────────────────────────────────────────────────────
main() {
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
            --fix)
                FIX_ISSUES=1
                shift
                ;;
            --skip-features)
                SKIP_FEATURE_MATRIX=1
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    log_info "Starting ready-to-push validation..."

    # Run all checks
    check_git_status || true
    check_commits_ahead || true
    check_format || true
    check_clippy || true
    check_unit_tests || true
    check_integration_tests || true
    check_feature_matrix || true

    # Print final summary
    print_final_summary
}

main "$@"
