#!/bin/bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
BINARY="${BINARY:-$REPO_ROOT/target/debug/cargo-cicd}"
EVIDENCE_DIR="$REPO_ROOT/playground/evidence"
mkdir -p "$EVIDENCE_DIR"

echo "=== cargo-cicd playground ==="
echo "binary: $BINARY"
echo ""

PASS=0; FAIL=0

run_command() {
    local name="$1"; shift
    echo "▶ $name"
    if "$@" > "$EVIDENCE_DIR/${name}.log" 2>&1; then
        echo "  ✓ PASS"
        PASS=$((PASS+1))
    else
        echo "  ✗ FAIL (exit $?)"
        cat "$EVIDENCE_DIR/${name}.log"
        FAIL=$((FAIL+1))
    fi
}

run_command "status"              "$BINARY" status
run_command "target-show"         "$BINARY" target show
run_command "target-prune"        "$BINARY" target prune
run_command "target-prune-dry"    "$BINARY" target prune --dry-run
run_command "test-changed"        "$BINARY" test changed
run_command "trybuild-changed"    "$BINARY" trybuild changed
run_command "git-status"          "$BINARY" git status
run_command "publish"             "$BINARY" publish
run_command "workspace-doctor"    "$BINARY" workspace doctor

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] || exit 1
