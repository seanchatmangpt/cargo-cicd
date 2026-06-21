#!/usr/bin/env bash
# Reads CLAUDE_TOOL_INPUT from env (JSON); warns on destructive patterns.
set -euo pipefail

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',d).get('command',''))" 2>/dev/null || true)

if [[ -z "$COMMAND" ]]; then
  exit 0
fi

# Use `if ... grep ...; then` — the `if` construct suppresses set -e for the
# condition expression, so a non-matching grep (exit 1) does not abort the script.
if echo "$COMMAND" | grep -qE 'git (reset --hard|push --force|push -f|clean -f|checkout -- |branch -D)'; then
  echo "WARNING: DESTRUCTIVE GIT OPERATION DETECTED." >&2
  echo "This command may permanently destroy work. Consider: git stash, git branch backup/<name>, or a dry-run alternative." >&2
fi

if echo "$COMMAND" | grep -q 'rm -rf'; then
  echo "WARNING: Destructive rm -rf detected. Verify path does not include source files, evidence, or receipts." >&2
fi

if echo "$COMMAND" | grep -q 'cargo publish'; then
  echo "WARNING: cargo publish will upload to crates.io. Ensure evidence gate passed (wpm audit) and invariants are green." >&2
fi

exit 0
