#!/bin/bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
EVIDENCE_DIR="$REPO_ROOT/playground/evidence"

echo "=== Cleaning playground evidence ==="

if [ -d "$EVIDENCE_DIR" ]; then
    find "$EVIDENCE_DIR" -type f ! -name '.gitkeep' -delete
    echo "Cleared: $EVIDENCE_DIR"
fi

echo "Done."
