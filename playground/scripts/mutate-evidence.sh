#!/usr/bin/env bash
set -euo pipefail

PASS=0; FAIL=0; BLOCKED=0

if ! command -v wpm &>/dev/null; then
  echo "  [BLOCKED] wpm not found in PATH — install wasm4pm to run refusal gate"
  BLOCKED=$((BLOCKED+1))
  echo ""
  echo "PASS=$PASS FAIL=$FAIL BLOCKED=$BLOCKED"
  exit 0
fi

EVIDENCE_DIR="$(mktemp -d)"
trap "rm -rf $EVIDENCE_DIR" EXIT

check_refused() {
  local name="$1" file="$2"
  if wpm audit "$file" 2>&1; then
    echo "  [FAIL] expected REFUSE for $name but got ACCEPT"
    FAIL=$((FAIL+1))
  else
    echo "  [PASS] wpm refused: $name"
    PASS=$((PASS+1))
  fi
}

# empty file
printf '' > "$EVIDENCE_DIR/empty.jsonl"
check_refused "empty file" "$EVIDENCE_DIR/empty.jsonl"

# binary garbage
printf '\x00\x01\xff\xfe NOT JSON' > "$EVIDENCE_DIR/binary-garbage.jsonl"
check_refused "binary garbage" "$EVIDENCE_DIR/binary-garbage.jsonl"

# truncated JSON
printf '{"event_id":"' > "$EVIDENCE_DIR/truncated.jsonl"
check_refused "truncated json" "$EVIDENCE_DIR/truncated.jsonl"

# missing required fields
printf '{"timestamp":"2026-01-01T00:00:00Z"}\n' > "$EVIDENCE_DIR/missing-fields.jsonl"
check_refused "missing required fields" "$EVIDENCE_DIR/missing-fields.jsonl"

echo ""
echo "PASS=$PASS FAIL=$FAIL BLOCKED=$BLOCKED"
[ $FAIL -eq 0 ]
