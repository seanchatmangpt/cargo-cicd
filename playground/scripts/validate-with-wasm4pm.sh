#!/usr/bin/env bash
set -euo pipefail

VERDICT="UNKNOWN"

if ! command -v wpm &>/dev/null; then
  echo "  [BLOCKED] wpm not found in PATH — install wasm4pm to enable validation"
  VERDICT="BLOCKED"
  exit 0
fi

echo "  wpm found: $(command -v wpm)"

if wpm doctor 2>&1; then
  VERDICT="PASS"
  echo "  [PASS] wpm doctor returned 0"
else
  VERDICT="FAIL"
  echo "  [FAIL] wpm doctor returned non-zero"
fi

echo ""
echo "VERDICT=$VERDICT"
[ "$VERDICT" != "FAIL" ]
