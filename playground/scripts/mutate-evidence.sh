#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
PASS=0; FAIL=0; BLOCKED=0

# Discover wpm — same priority order as validate-with-wasm4pm.sh
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
    echo "BLOCKED: wasm4pm oracle unavailable — refusal gate skipped"
    echo "PASS=$PASS FAIL=$FAIL BLOCKED=1"
    exit 1
fi

echo "=== wasm4pm refusal gate ==="
echo "oracle: $WPM"

EVIDENCE_DIR="$(mktemp -d)"
trap "rm -rf $EVIDENCE_DIR" EXIT

check_refused() {
    local name="$1" file="$2"
    if "$WPM" audit "$file" >/dev/null 2>&1; then
        echo "  ✗ FAIL: expected REFUSE for $name but got ACCEPT"
        FAIL=$((FAIL+1))
    else
        echo "  ✓ REFUSED: $name"
        PASS=$((PASS+1))
    fi
}

# 1. Empty file
printf '' > "$EVIDENCE_DIR/empty.xes"
check_refused "empty file" "$EVIDENCE_DIR/empty.xes"

# 2. Binary garbage
printf '\x00\x01\xff\xfe NOT XML' > "$EVIDENCE_DIR/binary-garbage.xes"
check_refused "binary garbage" "$EVIDENCE_DIR/binary-garbage.xes"

# 3. Truncated XES
printf '<?xml version="1.0"' > "$EVIDENCE_DIR/truncated.xes"
check_refused "truncated xml" "$EVIDENCE_DIR/truncated.xes"

# 4. Mismatched tags (proven to cause wpm exit 1)
cat > "$EVIDENCE_DIR/mismatched-tags.xes" <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="">
  <trace>
    <event>
      <string key="concept:name" value="status show"/>
    </wrong_close>
  </trace>
</log>
XML
check_refused "mismatched tags" "$EVIDENCE_DIR/mismatched-tags.xes"

# 5. Corrupt XML (plain text)
echo "NOT VALID XML AT ALL" > "$EVIDENCE_DIR/corrupt.xes"
check_refused "corrupt xml" "$EVIDENCE_DIR/corrupt.xes"

echo ""
echo "Refusal gate: $PASS refused, $FAIL unexpected-accepts, $BLOCKED blocked"
[ "$FAIL" -eq 0 ] || exit 1
