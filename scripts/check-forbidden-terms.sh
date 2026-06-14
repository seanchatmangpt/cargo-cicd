#!/bin/bash

# check-forbidden-terms.sh — Detect forbidden terminology in cargo-cicd files
# Used by: git pre-commit hook and pre-commit framework
# Part of the cargo-cicd governance policy

# Forbidden terms (from CLAUDE.md)
FORBIDDEN_TERMS=(
    "ALIVE"
    "Inspection Gate"
    "wall"
    "Nehemiah"
    "Field8"
    "Instinct8"
    "Cargo Court"
    "AGI"
    "Truex"
    "CONSTRUCT8"
)

ERROR=0

# Process files passed as arguments
for FILE in "$@"; do
    # Skip if file doesn't exist
    [ ! -f "$FILE" ] && continue

    # Skip binary files and generated code
    case "$FILE" in
        */target/*|*/.cargo/*|*/generated/*|ggen.toml) continue ;;
    esac

    for TERM in "${FORBIDDEN_TERMS[@]}"; do
        # Case-insensitive search but context-aware
        # (avoid false positives in comments explaining why term is forbidden)
        if grep -iq "$TERM" "$FILE" 2>/dev/null; then
            # Double-check it's not in a comment explaining the restriction
            if ! grep -iE "^\s*#.*[Ff]orbidden.*$TERM" "$FILE" > /dev/null 2>&1; then
                echo "✗ $FILE: Found forbidden term '$TERM'"
                ERROR=1
            fi
        fi
    done
done

exit $ERROR
