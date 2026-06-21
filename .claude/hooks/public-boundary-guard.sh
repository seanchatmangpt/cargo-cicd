#!/usr/bin/env bash
set -euo pipefail

# PostToolUse hook — public-boundary term guard (warn-only, always exits 0).
# Matcher: Edit|Write|MultiEdit
#
# Reads the tool JSON from stdin, extracts the edited file path, and checks
# public-surface files (src/, README.md, docs/) for forbidden terms.

# Forbidden terms that must never appear in public-surface files.
FORBIDDEN_TERMS=(
    "ALIVE"
    "Inspection Gate"
    "Nehemiah"
    "Field8"
    "Instinct8"
    "Cargo Court"
    "AGI"
    "Truex"
    "CONSTRUCT8"
)
# "wall" is a common English word — guard only the capitalised sentinel form
# used as a project code-word, not the generic noun.
# Per spec it is listed; include it as a whole-word match below.

# Extract the file path from stdin (tool JSON). Tolerant of empty/missing input.
FILE_PATH=""
if stdin_json=$(python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    ti = data.get('tool_input', data)
    print(ti.get('file_path', ti.get('path', '')))
except Exception:
    print('')
" 2>/dev/null); then
    FILE_PATH="${stdin_json}"
fi

# Nothing to check if we could not determine a path.
if [[ -z "${FILE_PATH}" ]]; then
    exit 0
fi

# Resolve to absolute path if relative.
if [[ "${FILE_PATH}" != /* ]]; then
    FILE_PATH="$(pwd)/${FILE_PATH}"
fi

# Skip if the file does not exist.
if [[ ! -f "${FILE_PATH}" ]]; then
    exit 0
fi

# Determine whether this is a public-surface file.
# Public surface: src/**, README.md, docs/**
# Excluded from checking: CLAUDE.md, anything under .claude/
is_public_surface=0

# Strip repo root prefix for pattern matching (works even without $REPO_ROOT).
REPO_ROOT="${CLAUDE_PROJECT_DIR:-$(git -C "$(dirname "$FILE_PATH")" rev-parse --show-toplevel 2>/dev/null || dirname "$(dirname "$FILE_PATH")")}"
rel="${FILE_PATH#${REPO_ROOT}/}"

case "${rel}" in
    CLAUDE.md)             is_public_surface=0 ;;
    .claude/*)             is_public_surface=0 ;;
    README.md)             is_public_surface=1 ;;
    src/*)                 is_public_surface=1 ;;
    docs/*)                is_public_surface=1 ;;
    *)                     is_public_surface=0 ;;
esac

if [[ "${is_public_surface}" -eq 0 ]]; then
    exit 0
fi

# Scan the file for each forbidden term.
found_any=0
for term in "${FORBIDDEN_TERMS[@]}"; do
    if grep -qF "${term}" "${FILE_PATH}" 2>/dev/null; then
        echo "WARNING: forbidden public-boundary term \"${term}\" found in ${FILE_PATH}" >&2
        found_any=1
    fi
done

# Check for the uppercase sentinel "WALL" only (whole-word). Lowercase "wall" is
# too common in English prose (firewall, brick wall, wall clock) to match broadly.
if grep -qw "WALL" "${FILE_PATH}" 2>/dev/null; then
    echo "WARNING: forbidden public-boundary term \"WALL\" found in ${FILE_PATH}" >&2
    found_any=1
fi

if [[ "${found_any}" -eq 1 ]]; then
    echo "WARNING: review ${FILE_PATH} and remove all forbidden terms before committing." >&2
fi

# Warn-only — always exit 0 so Claude is never blocked.
exit 0
