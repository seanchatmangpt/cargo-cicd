#!/bin/bash
#
# validate-advanced-build.sh
# Validates advanced build feature combinations for cargo-cicd
#
# This script tests multiple feature flag combinations to ensure they:
# 1. Compile without errors (cargo check)
# 2. Pass unit tests (cargo test --lib)
# 3. Pass clippy linting (cargo clippy)
#
# Usage: validate-advanced-build.sh [--quick] [--verbose] [--fix]

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default parameters
QUICK=false
VERBOSE=false
FIX=false

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --quick)
      QUICK=true
      shift
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    --fix)
      FIX=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Feature combinations to test
declare -a COMBINATIONS=(
  "advanced"
  "advanced,autonomic"
  "advanced,wasm4pm"
  "advanced,contrib"
  "advanced,autonomic,wasm4pm,contrib"
)

declare -a COMBINATION_NAMES=(
  "advanced-only"
  "advanced-autonomic"
  "advanced-wasm4pm"
  "advanced-contrib"
  "advanced-all"
)

# Initialize counters
COMPATIBLE=0
INCOMPATIBLE=0
PARTIAL=0
TOTAL=${#COMBINATIONS[@]}

# Create report file
REPORT_DIR="target/cargo-cicd/validation-reports"
mkdir -p "$REPORT_DIR"
REPORT_FILE="$REPORT_DIR/validation-$(date +%s).json"
SUMMARY_FILE="$REPORT_DIR/validation-summary-$(date +%s).txt"

# Start timing
START_TIME=$(date +%s)

# Initialize JSON report
cat > "$REPORT_FILE" <<EOF
{
  "validation": "validate-advanced-build",
  "timestamp": "$(date -Iseconds)",
  "parameters": {
    "quick": $QUICK,
    "verbose": $VERBOSE,
    "fix": $FIX
  },
  "combinations": []
}
EOF

# Function to run cargo command
run_cargo_command() {
  local features="$1"
  local cmd="$2"
  local full_cmd="cargo $cmd --features $features"

  if [ "$VERBOSE" = true ]; then
    echo -e "${BLUE}Running: $full_cmd${NC}"
  fi

  set +e
  if [ "$VERBOSE" = true ]; then
    output=$($full_cmd 2>&1)
    exit_code=$?
  else
    output=$($full_cmd 2>&1 | tail -20)
    exit_code=$?
  fi
  set -e

  echo "$output"
  return $exit_code
}

# Function to report combination result
report_combination() {
  local name="$1"
  local features="$2"
  local check_status="$3"
  local test_status="$4"
  local clippy_status="$5"
  local overall="$6"

  if [ "$overall" = "COMPATIBLE" ]; then
    echo -e "${GREEN}✅ $name: COMPATIBLE${NC}"
    ((COMPATIBLE++))
  elif [ "$overall" = "PARTIAL" ]; then
    echo -e "${YELLOW}⚠️  $name: PARTIAL (quick mode)${NC}"
    ((PARTIAL++))
  else
    echo -e "${RED}❌ $name: INCOMPATIBLE${NC}"
    ((INCOMPATIBLE++))
  fi
}

# Main validation loop
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Validating Advanced Build Feature Combinations${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

for idx in "${!COMBINATIONS[@]}"; do
  COMBO="${COMBINATIONS[$idx]}"
  COMBO_NAME="${COMBINATION_NAMES[$idx]}"

  echo -e "${BLUE}[$(($idx + 1))/$TOTAL] Testing: $COMBO_NAME (features: $COMBO)${NC}"
  echo "---"

  # Step 1: cargo check
  echo -n "  ① cargo check: "
  if check_output=$(run_cargo_command "$COMBO" "check"); then
    echo -e "${GREEN}PASS${NC}"
    CHECK_STATUS="PASS"
  else
    echo -e "${RED}FAIL${NC}"
    CHECK_STATUS="FAIL"
    if [ "$VERBOSE" = true ]; then
      echo -e "${RED}Error output:${NC}"
      echo "$check_output" | head -30
    fi
  fi

  # Step 2: cargo test (unless --quick)
  if [ "$QUICK" = true ]; then
    echo -n "  ② cargo test: "
    echo -e "${YELLOW}SKIP${NC} (--quick mode)"
    TEST_STATUS="SKIP"
  else
    echo -n "  ② cargo test: "
    if test_output=$(run_cargo_command "$COMBO" "test --lib --no-fail-fast 2>&1"); then
      echo -e "${GREEN}PASS${NC}"
      TEST_STATUS="PASS"
    else
      echo -e "${RED}FAIL${NC}"
      TEST_STATUS="FAIL"
      if [ "$VERBOSE" = true ]; then
        echo -e "${RED}Test output:${NC}"
        echo "$test_output" | tail -40
      fi
    fi
  fi

  # Step 3: cargo clippy (unless --quick)
  if [ "$QUICK" = true ]; then
    echo -n "  ③ cargo clippy: "
    echo -e "${YELLOW}SKIP${NC} (--quick mode)"
    CLIPPY_STATUS="SKIP"
  else
    echo -n "  ③ cargo clippy: "
    # First try without --fix
    if clippy_output=$(run_cargo_command "$COMBO" "clippy --all-targets -- -D warnings 2>&1"); then
      echo -e "${GREEN}PASS${NC}"
      CLIPPY_STATUS="PASS"
    else
      # Check if --fix can resolve issues
      if [ "$FIX" = true ]; then
        echo -n " (attempting --fix) "
        if fix_output=$(run_cargo_command "$COMBO" "clippy --all-targets --fix --allow-dirty --allow-staged 2>&1"); then
          # Verify fix worked
          if run_cargo_command "$COMBO" "clippy --all-targets -- -D warnings" > /dev/null 2>&1; then
            echo -e "${GREEN}FIXED${NC}"
            CLIPPY_STATUS="FIXED"
          else
            echo -e "${RED}FAIL${NC}"
            CLIPPY_STATUS="FAIL"
          fi
        else
          echo -e "${RED}FAIL${NC}"
          CLIPPY_STATUS="FAIL"
        fi
      else
        echo -e "${RED}FAIL${NC}"
        CLIPPY_STATUS="FAIL"
      fi
    fi
  fi

  # Determine overall status
  if [ "$CHECK_STATUS" = "FAIL" ] || [ "$TEST_STATUS" = "FAIL" ] || [ "$CLIPPY_STATUS" = "FAIL" ]; then
    OVERALL="INCOMPATIBLE"
  elif [ "$TEST_STATUS" = "SKIP" ] || [ "$CLIPPY_STATUS" = "SKIP" ]; then
    OVERALL="PARTIAL"
  else
    OVERALL="COMPATIBLE"
  fi

  report_combination "$COMBO_NAME" "$COMBO" "$CHECK_STATUS" "$TEST_STATUS" "$CLIPPY_STATUS" "$OVERALL"
  echo ""
done

# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Print summary
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}SUMMARY${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo "Total combinations tested: $TOTAL"
echo -e "Fully compatible:       ${GREEN}$COMPATIBLE✅${NC}"
echo -e "Partially tested:       ${YELLOW}$PARTIAL⚠️${NC}"
echo -e "Incompatible:           ${RED}$INCOMPATIBLE❌${NC}"
echo "Duration: ${DURATION}s"
echo ""

# Recommendations
if [ $INCOMPATIBLE -eq 0 ]; then
  echo -e "${GREEN}✅ All tested combinations are compatible!${NC}"
  echo "   You can safely use any combination of: advanced, autonomic, wasm4pm, contrib"
  EXIT_CODE=0
elif [ $INCOMPATIBLE -gt 0 ] && [ $COMPATIBLE -gt 0 ]; then
  echo -e "${YELLOW}⚠️  Some combinations are incompatible.${NC}"
  echo "   Compatible combinations found - use those for production."
  echo "   Run with --verbose to see detailed error messages."
  EXIT_CODE=1
else
  echo -e "${RED}❌ No compatible combinations found!${NC}"
  echo "   There is a fundamental issue preventing builds."
  echo "   Run with --verbose to see detailed error messages."
  EXIT_CODE=2
fi

# Save summary
{
  echo "Validation Report: validate-advanced-build"
  echo "Generated: $(date)"
  echo ""
  echo "Parameters:"
  echo "  Quick mode: $QUICK"
  echo "  Verbose:    $VERBOSE"
  echo "  Auto-fix:   $FIX"
  echo ""
  echo "Results:"
  echo "  Total:       $TOTAL"
  echo "  Compatible:  $COMPATIBLE"
  echo "  Partial:     $PARTIAL"
  echo "  Incompatible: $INCOMPATIBLE"
  echo "  Duration:    ${DURATION}s"
  echo ""
  echo "Reports saved to: $REPORT_DIR/"
} | tee "$SUMMARY_FILE"

echo ""
echo "JSON report: $REPORT_FILE"
echo "Summary:     $SUMMARY_FILE"

exit $EXIT_CODE
