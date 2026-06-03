#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
EVIDENCE_XES="$REPO_ROOT/target/cargo-cicd/evidence/events.xes"

# Discover wpm in priority order — checks known local build before PATH
WPM=""
for candidate in \
    "${WPM_BIN:-}" \
    "$REPO_ROOT/../wasm4pm/target/release/wpm" \
    "$HOME/wasm4pm/target/release/wpm" \
    "$HOME/wasm4pm/target/debug/wpm" \
    "$(command -v wpm 2>/dev/null || true)"; do
    if [ -n "$candidate" ] && [ -x "$candidate" ]; then
        WPM="$candidate"
        break
    fi
done

if [ -z "${WPM:-}" ]; then
    echo "BLOCKED: wasm4pm oracle unavailable"
    echo "  Searched: \$WPM_BIN, ../wasm4pm/target/release/wpm, ~/wasm4pm/, PATH"
    exit 1
fi

echo "=== wasm4pm evidence validation ==="
echo "oracle: $WPM"
"$WPM" --version
echo ""

echo "--- wpm doctor ---"
"$WPM" doctor || true
echo ""

if [ ! -f "$EVIDENCE_XES" ]; then
    echo "No evidence XES at $EVIDENCE_XES — running playground first"
    bash "$REPO_ROOT/playground/scripts/run-playground.sh"
fi

echo "--- wpm audit: positive evidence ---"
"$WPM" audit "$EVIDENCE_XES"
echo "verdict: ACCEPT"
