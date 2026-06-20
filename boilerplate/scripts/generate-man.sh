#!/usr/bin/env bash
# generate-man.sh — Generate or refresh the cargo-project man page.
#
# Strategy (tried in order):
#   1. help2man  — if installed, produces a well-structured roff man page
#                  directly from --help output.
#   2. Fallback  — copies the hand-maintained docs/man/cargo-project.1 as-is.
#
# Usage:
#   ./scripts/generate-man.sh [--output PATH]
#
# Options:
#   --output PATH    Write man page to PATH instead of docs/man/cargo-project.1
#
# After generation you can view it with:
#   man ./docs/man/cargo-project.1

set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Defaults
# ─────────────────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT="${REPO_ROOT}/docs/man/cargo-project.1"
VERSION="0.1.0"

# ─────────────────────────────────────────────────────────────────────────────
# Argument parsing
# ─────────────────────────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --output)
            OUTPUT="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [--output PATH] [--version X.Y.Z]"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

log()  { printf '\033[1;32m=>\033[0m %s\n' "$*"; }
info() { printf '   %s\n' "$*"; }
die()  { printf '\033[1;31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

# ─────────────────────────────────────────────────────────────────────────────
# Resolve binary
# ─────────────────────────────────────────────────────────────────────────────

if command -v cargo-project >/dev/null 2>&1; then
    BIN="cargo-project"
elif [[ -x "${REPO_ROOT}/target/release/cargo-project" ]]; then
    BIN="${REPO_ROOT}/target/release/cargo-project"
elif [[ -x "${REPO_ROOT}/target/debug/cargo-project" ]]; then
    BIN="${REPO_ROOT}/target/debug/cargo-project"
else
    die "cargo-project binary not found. Build with 'cargo build' first."
fi

info "Binary: $BIN"
info "Output: $OUTPUT"

mkdir -p "$(dirname "$OUTPUT")"

# ─────────────────────────────────────────────────────────────────────────────
# Strategy 1 — help2man
# ─────────────────────────────────────────────────────────────────────────────

if command -v help2man >/dev/null 2>&1; then
    log "Generating man page with help2man"

    # Build include file with DESCRIPTION and EXAMPLES sections that help2man
    # cannot derive from --help alone.
    INCLUDE_FILE="$(mktemp /tmp/cargo-project-man-include.XXXXXX)"
    trap 'rm -f "$INCLUDE_FILE"' EXIT

    cat > "$INCLUDE_FILE" <<'INCLUDE'
[DESCRIPTION]
.B cargo\-project
keeps your Rust workspace clean, fast, and push\-ready.

It inspects your workspace via a set of adapters (git, cargo, rustc) and
reports the overall health through a noun\-verb command grammar:

.RS 4
.nf
cargo project status show
cargo project workspace doctor
cargo project completions \-\-shell bash
.fi
.RE

The tool is safe by default: all read\-only commands exit 0; destructive
commands require an explicit \fB\-\-confirm\fR flag.

[EXAMPLES]
.TP
\fBcargo project status\fR
Show the workspace health snapshot (equivalent to \fBstatus show\fR).

.TP
\fBcargo project status show \-\-json\fR
Emit the snapshot as machine\-readable JSON.

.TP
\fBcargo project workspace doctor\fR
Run all workspace diagnostics.

.TP
\fBcargo project completions \-\-shell bash > ~/.bash_completion.d/cargo\-project\fR
Install bash tab\-completions for the current user.

.TP
\fBcargo project completions \-\-shell fish > ~/.config/fish/completions/cargo\-project.fish\fR
Install fish tab\-completions.
INCLUDE

    help2man \
        --name "keeps your Rust workspace clean, fast, and push-ready" \
        --section 1 \
        --include "$INCLUDE_FILE" \
        --no-info \
        --version-string "$VERSION" \
        "$BIN" > "$OUTPUT"

    log "Written (via help2man): $OUTPUT"
    exit 0
fi

# ─────────────────────────────────────────────────────────────────────────────
# Strategy 2 — fallback: use the hand-maintained man page
# ─────────────────────────────────────────────────────────────────────────────

log "help2man not found — using hand-maintained man page"
STATIC_SOURCE="${REPO_ROOT}/docs/man/cargo-project.1"

if [[ -f "$STATIC_SOURCE" && "$STATIC_SOURCE" != "$OUTPUT" ]]; then
    cp "$STATIC_SOURCE" "$OUTPUT"
    log "Copied: $STATIC_SOURCE → $OUTPUT"
elif [[ -f "$STATIC_SOURCE" ]]; then
    log "Already at target location: $OUTPUT"
else
    die "Static man page not found at $STATIC_SOURCE and help2man is unavailable."
fi

# ─────────────────────────────────────────────────────────────────────────────
# Validate
# ─────────────────────────────────────────────────────────────────────────────

if command -v groff >/dev/null 2>&1; then
    log "Validating man page with groff"
    groff -man -Tutf8 "$OUTPUT" > /dev/null && info "groff validation passed."
elif command -v mandoc >/dev/null 2>&1; then
    log "Validating man page with mandoc"
    mandoc -T lint "$OUTPUT" && info "mandoc lint passed."
else
    info "Neither groff nor mandoc found — skipping validation."
fi

log "Done. View with:  man $OUTPUT"
