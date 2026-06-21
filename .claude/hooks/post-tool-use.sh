#!/usr/bin/env bash
# After cargo build/make/test, remind about evidence emission if relevant.
set -euo pipefail

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('command',''))" 2>/dev/null || true)

if [[ -n "$COMMAND" ]] && echo "$COMMAND" | grep -qE 'cargo (build|make|test)'; then
  EVIDENCE_DIR="target/cargo-cicd/evidence"
  if [[ -d "$EVIDENCE_DIR" ]]; then
    COUNT=$(find "$EVIDENCE_DIR" -name '*.xes' -o -name '*.jsonl' 2>/dev/null | wc -l | tr -d ' ')
    if [[ "$COUNT" -gt 0 ]]; then
      echo "INFO: $COUNT evidence file(s) in $EVIDENCE_DIR — run 'cargo cicd evidence doctor' if the evidence gate is relevant." >&2
    fi
  fi
fi

exit 0
