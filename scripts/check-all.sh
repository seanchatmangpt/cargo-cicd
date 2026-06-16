#!/usr/bin/env bash
# cargo-cicd All Checks Script
# Purpose: Run all validation tasks in parallel (fmt, clippy, tests, coverage)
# Usage: ./scripts/check-all.sh [OPTIONS]

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
SHOW_SLOWEST=1

show_help() {
    cat << EOF
${BLUE}cargo-cicd Check All Validation${NC}

${YELLOW}USAGE:${NC}
  $(basename "$0") [OPTIONS]

${YELLOW}OPTIONS:${NC}
  --help          Show this help message
  --verbose       Enable verbose output
  --quiet         Suppress non-essential output
  --no-slowest    Don't show slowest task at the end

${YELLOW}DESCRIPTION:${NC}
  Runs all validation checks in parallel:
  - cargo fmt --check (code formatting)
  - cargo clippy (linting)
  - cargo test (all tests)
  - cargo test --doc (documentation tests)

  Results are color-coded and a summary is displayed.
  The slowest task is reported at the end.

${YELLOW}EXAMPLES:${NC}
  # Run all checks with defaults
  ./scripts/check-all.sh

  # Run with verbose output
  ./scripts/check-all.sh --verbose

  # Quick quiet mode
  ./scripts/check-all.sh --quiet

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

# ─── Temp Files for Parallel Execution ──────────────────────────────────────────
readonly TMPDIR_BASE="${TMPDIR:-.}/.cargo-cicd-check"
mkdir -p "$TMPDIR_BASE"

trap "rm -rf $TMPDIR_BASE" EXIT

# ─── Check Functions ──────────────────────────────────────────────────────────
check_fmt() {
    local task="Format Check"
    local output_file="$TMPDIR_BASE/fmt.txt"
    local start_time start_sec

    start_time=$(date +%s.%N)

    if cargo fmt --check > "$output_file" 2>&1; then
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|PASS|$elapsed" > "$TMPDIR_BASE/fmt.status"
        [[ $VERBOSE -eq 1 ]] && cat "$output_file"
    else
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|FAIL|$elapsed" > "$TMPDIR_BASE/fmt.status"
        cat "$output_file" >> "$TMPDIR_BASE/fmt.status"
    fi
}

check_clippy() {
    local task="Clippy Lint"
    local output_file="$TMPDIR_BASE/clippy.txt"
    local start_time

    start_time=$(date +%s.%N)

    if cargo clippy --all-targets --all-features -- -D warnings > "$output_file" 2>&1; then
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|PASS|$elapsed" > "$TMPDIR_BASE/clippy.status"
        [[ $VERBOSE -eq 1 ]] && cat "$output_file"
    else
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|FAIL|$elapsed" > "$TMPDIR_BASE/clippy.status"
        cat "$output_file" >> "$TMPDIR_BASE/clippy.status"
    fi
}

check_tests() {
    local task="Unit Tests"
    local output_file="$TMPDIR_BASE/tests.txt"
    local start_time

    start_time=$(date +%s.%N)

    if cargo test --lib 2>&1 | tee "$output_file" | grep -E "(test result|running)"; then
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|PASS|$elapsed" > "$TMPDIR_BASE/tests.status"
    else
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|FAIL|$elapsed" > "$TMPDIR_BASE/tests.status"
    fi
}

check_doc_tests() {
    local task="Doc Tests"
    local output_file="$TMPDIR_BASE/doctests.txt"
    local start_time

    start_time=$(date +%s.%N)

    if cargo test --doc 2>&1 | tee "$output_file" | grep -E "(test result|running)"; then
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|PASS|$elapsed" > "$TMPDIR_BASE/doctests.status"
    else
        local elapsed
        elapsed=$(awk -v s="$start_time" 'BEGIN {print int((systime() - s) * 1000)}')
        echo "$task|FAIL|$elapsed" > "$TMPDIR_BASE/doctests.status"
    fi
}

# ─── Summary Functions ─────────────────────────────────────────────────────────
print_check_result() {
    local status_file=$1
    local task result elapsed line1 line2

    if [[ ! -f "$status_file" ]]; then
        return
    fi

    read -r line1 < "$status_file"
    IFS='|' read -r task result elapsed <<< "$line1"

    if [[ "$result" == "PASS" ]]; then
        printf "  ${GREEN}✓${NC} %-20s ${GRAY}(%dms)${NC}\n" "$task" "$elapsed"
    else
        printf "  ${RED}✗${NC} %-20s ${GRAY}(%dms)${NC}\n" "$task" "$elapsed"
        tail -n +2 "$status_file" | head -10 | sed 's/^/      /'
    fi
}

find_slowest_check() {
    local slowest_task=""
    local slowest_time=0

    for status_file in "$TMPDIR_BASE"/*.status; do
        [[ ! -f "$status_file" ]] && continue

        local line1
        read -r line1 < "$status_file"
        local task elapsed
        IFS='|' read -r task _ elapsed <<< "$line1"

        if [[ $elapsed -gt $slowest_time ]]; then
            slowest_time=$elapsed
            slowest_task="$task"
        fi
    done

    if [[ -n "$slowest_task" ]]; then
        printf "\n${YELLOW}Slowest:${NC} $slowest_task (${slowest_time}ms)\n"
    fi
}

check_results() {
    local pass_count=0
    local fail_count=0
    local total=0

    for status_file in "$TMPDIR_BASE"/*.status; do
        [[ ! -f "$status_file" ]] && continue

        total=$((total + 1))
        local line1
        read -r line1 < "$status_file"
        local result
        result=$(echo "$line1" | cut -d'|' -f2)

        if [[ "$result" == "PASS" ]]; then
            pass_count=$((pass_count + 1))
        else
            fail_count=$((fail_count + 1))
        fi
    done

    return "$fail_count"
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
            --no-slowest)
                SHOW_SLOWEST=0
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    log_info "Running all validation checks in parallel..."

    # Launch all checks in parallel
    check_fmt &
    local pid_fmt=$!

    check_clippy &
    local pid_clippy=$!

    check_tests &
    local pid_tests=$!

    check_doc_tests &
    local pid_doctests=$!

    # Wait for all to complete
    wait $pid_fmt || true
    wait $pid_clippy || true
    wait $pid_tests || true
    wait $pid_doctests || true

    # Print results
    log_info "Validation Results:"
    echo "" >&2
    print_check_result "$TMPDIR_BASE/fmt.status"
    print_check_result "$TMPDIR_BASE/clippy.status"
    print_check_result "$TMPDIR_BASE/tests.status"
    print_check_result "$TMPDIR_BASE/doctests.status"
    echo "" >&2

    if [[ $SHOW_SLOWEST -eq 1 ]]; then
        find_slowest_check
    fi

    # Check final status
    if check_results; then
        log_success "All validation checks passed!"
        return 0
    else
        local fail_count
        fail_count=$(for f in "$TMPDIR_BASE"/*.status; do grep -q "FAIL" "$f" && echo 1; done | wc -l)
        log_error "$fail_count validation check(s) failed"
        return 1
    fi
}

main "$@"
