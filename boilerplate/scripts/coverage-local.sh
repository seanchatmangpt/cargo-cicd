#!/usr/bin/env bash
# coverage-local.sh — Run tarpaulin coverage locally, open HTML report, and
# optionally print an LCOV summary if lcov is installed.
set -euo pipefail

FEATURES="process-data autonomic"
OUTPUT_DIR="target/coverage"

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────
log()  { printf '\033[1;34m[coverage]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[coverage]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[coverage]\033[0m ERROR: %s\n' "$*" >&2; exit 1; }

# Detect OS for the "open browser" step
open_browser() {
    local path="$1"
    case "$(uname -s)" in
        Darwin)  open "$path" ;;
        Linux)
            if command -v xdg-open &>/dev/null; then
                xdg-open "$path"
            else
                warn "xdg-open not found; cannot auto-open the report."
                log  "Report is at: $path"
            fi
            ;;
        *)
            warn "Unknown OS; cannot auto-open the report."
            log  "Report is at: $path"
            ;;
    esac
}

# ─────────────────────────────────────────────────────────────────────────────
# Install cargo-tarpaulin if not present
# ─────────────────────────────────────────────────────────────────────────────
if ! command -v cargo-tarpaulin &>/dev/null; then
    log "cargo-tarpaulin not found — installing (this may take a minute)..."
    cargo install cargo-tarpaulin --locked
else
    log "cargo-tarpaulin already installed: $(cargo tarpaulin --version 2>&1 | head -1)"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Run tarpaulin
# ─────────────────────────────────────────────────────────────────────────────
log "Running tarpaulin with features: ${FEATURES}"
log "Output directory: ${OUTPUT_DIR}"

mkdir -p "${OUTPUT_DIR}"

cargo tarpaulin \
    --workspace \
    --features "${FEATURES}" \
    --timeout 180 \
    --out Html Lcov \
    --output-dir "${OUTPUT_DIR}" \
    --exclude-files "*/main.rs" \
    --exclude-files "tests/*" \
    --exclude-files "benches/*" \
    --exclude-files "build.rs" \
    -- --test-threads 1

# ─────────────────────────────────────────────────────────────────────────────
# LCOV summary
# ─────────────────────────────────────────────────────────────────────────────
LCOV_FILE="${OUTPUT_DIR}/lcov.info"
if [[ -f "${LCOV_FILE}" ]]; then
    if command -v lcov &>/dev/null; then
        log "LCOV summary:"
        lcov --summary "${LCOV_FILE}" 2>&1 || warn "lcov --summary failed; check ${LCOV_FILE} manually."
    else
        warn "lcov not installed — skipping summary. Install with: sudo apt install lcov (Debian/Ubuntu) or brew install lcov (macOS)."
        log  "LCOV data written to: ${LCOV_FILE}"
    fi
else
    warn "lcov.info not found at ${LCOV_FILE}; tarpaulin may have failed."
fi

# ─────────────────────────────────────────────────────────────────────────────
# Open HTML report
# ─────────────────────────────────────────────────────────────────────────────
HTML_REPORT="${OUTPUT_DIR}/tarpaulin-report.html"
if [[ -f "${HTML_REPORT}" ]]; then
    log "Opening HTML report: ${HTML_REPORT}"
    open_browser "$(realpath "${HTML_REPORT}")"
else
    warn "HTML report not found at ${HTML_REPORT}."
fi

log "Done."
