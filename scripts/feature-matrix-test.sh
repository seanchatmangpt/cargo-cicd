#!/usr/bin/env bash
# cargo-cicd Feature Matrix Test Script
# Purpose: Test all feature combinations and identify incompatibilities
# Usage: ./scripts/feature-matrix-test.sh [OPTIONS]

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
SHOW_TABLE=1

show_help() {
    cat << EOF
${BLUE}cargo-cicd Feature Matrix Test${NC}

${YELLOW}USAGE:${NC}
  $(basename "$0") [OPTIONS]

${YELLOW}OPTIONS:${NC}
  --help        Show this help message
  --verbose     Enable verbose output
  --quiet       Suppress non-essential output
  --no-table    Don't show summary table at end

${YELLOW}DESCRIPTION:${NC}
  Tests all feature combinations:
  - No features (default)
  - process-data
  - autonomic (includes process-data)
  - contrib (includes process-data)
  - wasm4pm (includes process-data)
  - advanced (includes process-data + many deps)
  - Combined feature sets (where compatible)

  For each combination:
  - Compiles the code
  - Runs tests
  - Reports pass/fail

  Identifies which feature combinations are incompatible.

${YELLOW}EXAMPLES:${NC}
  # Test all feature combinations
  ./scripts/feature-matrix-test.sh

  # With verbose output to debug failures
  ./scripts/feature-matrix-test.sh --verbose

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

# ─── Temp Directory Setup ───────────────────────────────────────────────────────
readonly TMPDIR_BASE="${TMPDIR:-.}/.cargo-cicd-matrix"
mkdir -p "$TMPDIR_BASE"
trap "rm -rf $TMPDIR_BASE" EXIT

# ─── Feature Combinations ───────────────────────────────────────────────────────
declare -a FEATURE_SETS=(
    ""                                    # default, no features
    "process-data"
    "autonomic"
    "contrib"
    "wasm4pm"
    "advanced"
    "process-data,autonomic"
    "process-data,contrib"
    "process-data,wasm4pm"
    "autonomic,contrib"
    "autonomic,wasm4pm"
    "contrib,wasm4pm"
)

# ─── Test Functions ────────────────────────────────────────────────────────────
test_feature_combo() {
    local features=$1
    local output_file=$2
    local status_file=$3

    local feature_label="${features:-(none)}"
    log_verbose "Testing feature combination: $feature_label"

    # Test compilation
    local compile_output
    if [[ -z "$features" ]]; then
        compile_output=$(cargo build 2>&1) || {
            echo "COMPILE_FAIL" > "$status_file"
            echo "$compile_output" >> "$status_file"
            return 1
        }
    else
        compile_output=$(cargo build --features "$features" 2>&1) || {
            echo "COMPILE_FAIL" > "$status_file"
            echo "$compile_output" >> "$status_file"
            return 1
        }
    fi

    # Test compilation
    local test_output
    if [[ -z "$features" ]]; then
        test_output=$(cargo test --lib 2>&1) || {
            echo "TEST_FAIL" > "$status_file"
            echo "$test_output" >> "$status_file"
            return 1
        }
    else
        test_output=$(cargo test --lib --features "$features" 2>&1) || {
            echo "TEST_FAIL" > "$status_file"
            echo "$test_output" >> "$status_file"
            return 1
        }
    fi

    echo "PASS" > "$status_file"
    return 0
}

run_feature_tests() {
    local index=0
    local total=${#FEATURE_SETS[@]}

    log_info "Testing $total feature combinations..."

    for features in "${FEATURE_SETS[@]}"; do
        index=$((index + 1))
        local pct=$((index * 100 / total))

        local feature_label="${features:-(default)}"
        printf "[%3d%%] Testing: %-35s " "$pct" "$feature_label" >&2

        local status_file="$TMPDIR_BASE/result_$index.status"
        local output_file="$TMPDIR_BASE/output_$index.log"

        if test_feature_combo "$features" "$output_file" "$status_file"; then
            log_success "PASS"
        else
            local status
            status=$(head -1 "$status_file")
            log_error "$status"
        fi
    done

    echo "" >&2
}

# ─── Results Summary ───────────────────────────────────────────────────────────
count_results() {
    local status=$1
    local count=0

    for status_file in "$TMPDIR_BASE"/result_*.status; do
        [[ ! -f "$status_file" ]] && continue
        local result
        result=$(head -1 "$status_file")
        [[ "$result" == "$status" ]] && count=$((count + 1))
    done

    echo "$count"
}

print_summary_table() {
    local index=0
    local pass_count
    pass_count=$(count_results "PASS")
    local total=${#FEATURE_SETS[@]}
    local fail_count=$((total - pass_count))

    log_info "Feature Matrix Test Results:"
    echo "" >&2

    cat >&2 << EOF
${BLUE}┌─ Feature Combination ─────────────────────────┬────────┐${NC}
${BLUE}│ Feature Set                                    │ Result │${NC}
${BLUE}├────────────────────────────────────────────────┼────────┤${NC}
EOF

    for features in "${FEATURE_SETS[@]}"; do
        index=$((index + 1))
        local status_file="$TMPDIR_BASE/result_$index.status"
        local result
        result=$(head -1 "$status_file")

        local feature_label="${features:-(default)}"
        feature_label=$(printf "%-44s" "$feature_label")

        if [[ "$result" == "PASS" ]]; then
            printf "${BLUE}│${NC} %s ${GREEN}✓ PASS${NC}\n" "$feature_label"
        else
            printf "${BLUE}│${NC} %s ${RED}✗ FAIL${NC}\n" "$feature_label"
        fi
    done

    cat >&2 << EOF
${BLUE}└────────────────────────────────────────────────┴────────┘${NC}

${BLUE}Summary:${NC}
  Passed: ${GREEN}$pass_count${NC}/$total
  Failed: $([ $fail_count -gt 0 ] && echo -e "${RED}$fail_count${NC}" || echo "0")

EOF

    if [[ $fail_count -gt 0 ]]; then
        log_warn "Failed feature combinations:"
        index=0
        for features in "${FEATURE_SETS[@]}"; do
            index=$((index + 1))
            local status_file="$TMPDIR_BASE/result_$index.status"
            local result
            result=$(head -1 "$status_file")
            if [[ "$result" != "PASS" ]]; then
                local feature_label="${features:-(default)}"
                printf "  - %s\n" "$feature_label" >&2
                if [[ $VERBOSE -eq 1 ]]; then
                    tail -n +2 "$status_file" | head -5 | sed 's/^/      /' >&2
                fi
            fi
        done
    fi
}

identify_incompatibilities() {
    log_info "Analyzing incompatibilities..."

    local autonomic_pass=0
    local process_data_pass=0

    # Check individual features
    for i in "${!FEATURE_SETS[@]}"; do
        local status_file="$TMPDIR_BASE/result_$((i + 1)).status"
        local result
        result=$(head -1 "$status_file")

        if [[ "$result" == "PASS" ]]; then
            case "${FEATURE_SETS[$i]}" in
                "autonomic")
                    autonomic_pass=1
                    ;;
                "process-data")
                    process_data_pass=1
                    ;;
            esac
        fi
    done

    echo ""
    if [[ $autonomic_pass -eq 1 ]]; then
        log_success "autonomic feature is compatible"
    fi
    if [[ $process_data_pass -eq 1 ]]; then
        log_success "process-data feature is compatible"
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
            --no-table)
                SHOW_TABLE=0
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    log_info "Starting feature matrix test..."
    run_feature_tests

    if [[ $SHOW_TABLE -eq 1 ]]; then
        print_summary_table
    fi

    identify_incompatibilities

    # Determine exit status
    local fail_count
    fail_count=$(count_results "FAIL")
    if [[ $fail_count -eq 0 ]]; then
        log_success "All feature combinations passed!"
        return 0
    else
        log_error "$fail_count feature combination(s) failed"
        return 1
    fi
}

main "$@"
