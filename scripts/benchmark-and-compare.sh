#!/usr/bin/env bash
# cargo-cicd Benchmark and Compare Script
# Purpose: Run benchmarks with advanced features and compare to main branch
# Usage: ./scripts/benchmark-and-compare.sh [OPTIONS]

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
SAVE_RESULTS=0
BASELINE_BRANCH="main"

show_help() {
    cat << EOF
${BLUE}cargo-cicd Benchmark and Compare${NC}

${YELLOW}USAGE:${NC}
  $(basename "$0") [OPTIONS]

${YELLOW}OPTIONS:${NC}
  --help              Show this help message
  --verbose           Enable verbose output
  --quiet             Suppress non-essential output
  --save              Save results to file (benchmark_results.json)
  --baseline BRANCH   Compare against branch (default: main)

${YELLOW}DESCRIPTION:${NC}
  Runs benchmarks with the 'advanced' feature set and compares
  against a baseline (default: main branch). Shows:
  - Current build time
  - Baseline build time
  - Regression/improvement percentage
  - Detailed breakdown if regression detected

${YELLOW}EXAMPLES:${NC}
  # Benchmark current code against main
  ./scripts/benchmark-and-compare.sh

  # Save results for CI/tracking
  ./scripts/benchmark-and-compare.sh --save

  # Compare against develop branch
  ./scripts/benchmark-and-compare.sh --baseline develop

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

# ─── Benchmark Functions ───────────────────────────────────────────────────────
measure_build_time() {
    local feature_flags=${1:-"advanced"}
    local output_file=${2:-.}

    log_verbose "Building with features: $feature_flags"

    # Clean build to get accurate measurement
    cargo clean --release 2>/dev/null || true

    local start_time
    start_time=$(date +%s%N)

    if cargo build --release --features "$feature_flags" > "$output_file" 2>&1; then
        local end_time
        end_time=$(date +%s%N)
        local duration=$((($end_time - $start_time) / 1000000)) # Convert to ms

        log_verbose "Build completed in ${duration}ms"
        echo "$duration"
        return 0
    else
        log_error "Build failed with features: $feature_flags"
        cat "$output_file" | head -20 >&2
        return 1
    fi
}

check_baseline_exists() {
    local branch=$1

    if ! git rev-parse --verify "$branch" > /dev/null 2>&1; then
        log_warn "Baseline branch '$branch' not found"
        return 1
    fi

    return 0
}

stash_current_changes() {
    log_verbose "Stashing current changes..."

    if ! git diff --quiet; then
        git stash push -m "benchmark-temp" > /dev/null 2>&1
        return 0
    fi

    return 1
}

restore_changes() {
    if [[ -n "$(git stash list | head -1)" ]]; then
        log_verbose "Restoring stashed changes..."
        git stash pop > /dev/null 2>&1 || true
    fi
}

measure_baseline_time() {
    local baseline_branch=$1
    local current_branch
    current_branch=$(git rev-parse --abbrev-ref HEAD)

    log_info "Measuring baseline build time (branch: $baseline_branch)..."

    # Stash current changes
    local had_stash=0
    stash_current_changes && had_stash=1

    # Switch to baseline
    if ! git checkout "$baseline_branch" > /dev/null 2>&1; then
        log_error "Failed to checkout baseline branch: $baseline_branch"
        [[ $had_stash -eq 1 ]] && restore_changes
        return 1
    fi

    # Measure baseline
    local baseline_time
    if baseline_time=$(measure_build_time "advanced" /tmp/baseline_build.log); then
        log_verbose "Baseline build time: ${baseline_time}ms"
    else
        log_error "Baseline build measurement failed"
        git checkout "$current_branch" > /dev/null 2>&1
        [[ $had_stash -eq 1 ]] && restore_changes
        return 1
    fi

    # Return to current branch
    git checkout "$current_branch" > /dev/null 2>&1

    # Restore changes
    [[ $had_stash -eq 1 ]] && restore_changes

    echo "$baseline_time"
}

calculate_regression() {
    local current=$1
    local baseline=$2

    if [[ $baseline -eq 0 ]]; then
        echo "0"
        return
    fi

    local diff=$((current - baseline))
    local percent=$((diff * 100 / baseline))

    echo "$percent"
}

format_duration() {
    local ms=$1
    local sec=$((ms / 1000))
    local remaining=$((ms % 1000))

    if [[ $sec -gt 0 ]]; then
        printf "%d.%03ds" "$sec" "$remaining"
    else
        printf "%dms" "$ms"
    fi
}

# ─── Results Functions ─────────────────────────────────────────────────────────
print_comparison_table() {
    local current=$1
    local baseline=$2
    local regression=$3

    cat >&2 << EOF

${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${BLUE}Build Time Comparison${NC}
${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${BLUE}Current Build:${NC}  $(format_duration "$current")
${BLUE}Baseline Build:${NC} $(format_duration "$baseline")
EOF

    if [[ $regression -lt 0 ]]; then
        local improvement=$((regression * -1))
        printf "${GREEN}Improvement:${NC}  ${improvement}%%\n" >&2
    elif [[ $regression -gt 0 ]]; then
        printf "${RED}Regression:${NC}   +${regression}%%\n" >&2
    else
        printf "${GREEN}No Change${NC}    0%%\n" >&2
    fi

    echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" >&2
}

save_benchmark_results() {
    local current=$1
    local baseline=$2
    local regression=$3
    local output_file="benchmark_results.json"

    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    local branch
    branch=$(git rev-parse --abbrev-ref HEAD)

    local commit
    commit=$(git rev-parse --short HEAD)

    cat > "$output_file" << EOF
{
  "timestamp": "$timestamp",
  "branch": "$branch",
  "commit": "$commit",
  "baseline_branch": "$BASELINE_BRANCH",
  "build_time_ms": $current,
  "baseline_time_ms": $baseline,
  "regression_percent": $regression,
  "features": "advanced"
}
EOF

    log_success "Results saved to $output_file"
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
            --save)
                SAVE_RESULTS=1
                shift
                ;;
            --baseline)
                shift
                BASELINE_BRANCH=$1
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    log_info "Starting benchmark and comparison..."

    # Check if baseline exists
    if ! check_baseline_exists "$BASELINE_BRANCH"; then
        log_warn "Skipping baseline comparison"
        log_info "Measuring current build time only..."

        local current_time
        if current_time=$(measure_build_time "advanced" /tmp/current_build.log); then
            log_success "Current build time: $(format_duration "$current_time")"
            return 0
        else
            return 1
        fi
    fi

    # Measure current and baseline
    log_info "Measuring current build time (features: advanced)..."
    local current_time
    if ! current_time=$(measure_build_time "advanced" /tmp/current_build.log); then
        return 1
    fi

    local baseline_time
    if ! baseline_time=$(measure_baseline_time "$BASELINE_BRANCH"); then
        return 1
    fi

    # Calculate regression
    local regression
    regression=$(calculate_regression "$current_time" "$baseline_time")

    # Print comparison
    print_comparison_table "$current_time" "$baseline_time" "$regression"

    # Warn if significant regression
    if [[ $regression -gt 10 ]]; then
        log_warn "Significant performance regression detected (>10%)"
        log_info "Consider profiling with: cargo flamegraph --features advanced"
    elif [[ $regression -gt 5 ]]; then
        log_warn "Minor performance regression detected (5-10%)"
    fi

    # Save results if requested
    if [[ $SAVE_RESULTS -eq 1 ]]; then
        save_benchmark_results "$current_time" "$baseline_time" "$regression"
    fi

    log_success "Benchmark complete!"
    return 0
}

main "$@"
