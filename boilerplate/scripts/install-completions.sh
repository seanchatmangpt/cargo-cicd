#!/usr/bin/env bash
# install-completions.sh — Install shell completions for cargo-project.
#
# Usage:
#   ./scripts/install-completions.sh [bash|zsh|fish]
#
# When the shell argument is omitted the script auto-detects the current shell
# from $SHELL and installs for that shell.
#
# Supported shells and install locations:
#   bash  → ~/.bash_completion.d/cargo-project
#   zsh   → ~/.zsh/completions/_cargo-project
#   fish  → ~/.config/fish/completions/cargo-project.fish
#
# PowerShell and Elvish users should follow the manual steps printed at the end
# of this script.

set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

log()  { printf '\033[1;32m=>\033[0m %s\n' "$*"; }
info() { printf '   %s\n' "$*"; }
warn() { printf '\033[1;33m!! %s\033[0m\n' "$*"; }
die()  { printf '\033[1;31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

require_binary() {
    command -v "$1" >/dev/null 2>&1 || die "Required binary not found: $1"
}

# ─────────────────────────────────────────────────────────────────────────────
# Resolve the cargo-project binary
# ─────────────────────────────────────────────────────────────────────────────

# Prefer an already-installed binary; fall back to the debug build in ./target.
if command -v cargo-project >/dev/null 2>&1; then
    BIN="cargo-project"
elif [[ -x "./target/debug/cargo-project" ]]; then
    BIN="./target/debug/cargo-project"
elif [[ -x "./target/release/cargo-project" ]]; then
    BIN="./target/release/cargo-project"
else
    die "cargo-project binary not found. Run 'cargo build' first, or install via 'cargo install'."
fi

info "Using binary: $BIN"

# ─────────────────────────────────────────────────────────────────────────────
# Detect shell
# ─────────────────────────────────────────────────────────────────────────────

REQUESTED_SHELL="${1:-}"

if [[ -z "$REQUESTED_SHELL" ]]; then
    # Auto-detect from $SHELL environment variable.
    SHELL_NAME="$(basename "${SHELL:-}")"
    case "$SHELL_NAME" in
        bash)  REQUESTED_SHELL="bash" ;;
        zsh)   REQUESTED_SHELL="zsh"  ;;
        fish)  REQUESTED_SHELL="fish" ;;
        *)
            warn "Could not auto-detect shell from \$SHELL='${SHELL:-}'."
            warn "Specify explicitly: $0 [bash|zsh|fish]"
            exit 1
            ;;
    esac
    info "Auto-detected shell: $REQUESTED_SHELL"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Install per-shell
# ─────────────────────────────────────────────────────────────────────────────

case "$REQUESTED_SHELL" in

# ── bash ─────────────────────────────────────────────────────────────────────
bash)
    DEST_DIR="${HOME}/.bash_completion.d"
    DEST_FILE="${DEST_DIR}/cargo-project"

    log "Installing bash completions"
    mkdir -p "$DEST_DIR"
    "$BIN" completions --shell bash > "$DEST_FILE"
    log "Written: $DEST_FILE"

    # Check whether ~/.bash_completion.d is sourced.
    RC_FILES=("${HOME}/.bashrc" "${HOME}/.bash_profile" "${HOME}/.profile")
    SOURCE_LINE="source \"${HOME}/.bash_completion.d/cargo-project\""
    ALREADY_SOURCED=false

    for rc in "${RC_FILES[@]}"; do
        if [[ -f "$rc" ]] && grep -qF "bash_completion.d" "$rc" 2>/dev/null; then
            ALREADY_SOURCED=true
            break
        fi
    done

    if [[ "$ALREADY_SOURCED" == false ]]; then
        warn "~/.bash_completion.d is not yet sourced by your shell startup files."
        info "Add one of the following to ~/.bashrc (or ~/.bash_profile):"
        info ""
        info "  Option A — source only cargo-project:"
        info "    ${SOURCE_LINE}"
        info ""
        info "  Option B — source the entire directory (if you have multiple tools):"
        info "    for f in ~/.bash_completion.d/*; do source \"\$f\"; done"
        info ""
    fi

    info "Reload your shell or run:  source ${DEST_FILE}"
    ;;

# ── zsh ──────────────────────────────────────────────────────────────────────
zsh)
    DEST_DIR="${HOME}/.zsh/completions"
    DEST_FILE="${DEST_DIR}/_cargo-project"

    log "Installing zsh completions"
    mkdir -p "$DEST_DIR"
    "$BIN" completions --shell zsh > "$DEST_FILE"
    log "Written: $DEST_FILE"

    # Check whether DEST_DIR is in $fpath.
    FPATH_CHECK="${HOME}/.zshrc"
    if ! grep -qF ".zsh/completions" "$FPATH_CHECK" 2>/dev/null; then
        warn "~/.zsh/completions may not be in your \$fpath."
        info "Add to ~/.zshrc BEFORE the 'compinit' call:"
        info ""
        info "  fpath=(~/.zsh/completions \$fpath)"
        info "  autoload -Uz compinit && compinit"
        info ""
    fi

    info "Reload completions:  autoload -Uz compinit && compinit"
    ;;

# ── fish ─────────────────────────────────────────────────────────────────────
fish)
    # fish reads all *.fish files in completions directories automatically.
    DEST_DIR="${HOME}/.config/fish/completions"
    DEST_FILE="${DEST_DIR}/cargo-project.fish"

    log "Installing fish completions"
    mkdir -p "$DEST_DIR"
    "$BIN" completions --shell fish > "$DEST_FILE"
    log "Written: $DEST_FILE"
    info "fish picks up completions automatically — no further action needed."
    ;;

# ── unsupported ──────────────────────────────────────────────────────────────
powershell|pwsh)
    log "PowerShell — manual steps required"
    info ""
    info "  1. Generate the completion script:"
    info "     cargo project completions --shell powershell | Out-File -Append \$PROFILE"
    info ""
    info "  2. Reload your profile:"
    info "     . \$PROFILE"
    info ""
    info "Note: \$PROFILE is typically:"
    info "  ~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1  (pwsh 7+)"
    info "  ~/Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1  (Windows PS 5)"
    exit 0
    ;;

elvish)
    log "Elvish — manual steps required"
    DEST_DIR="${HOME}/.config/elvish/completions"
    info ""
    info "  1. Create the completions directory (if absent):"
    info "     mkdir -p ${DEST_DIR}"
    info ""
    info "  2. Generate the completion script:"
    info "     cargo project completions --shell elvish \\"
    info "         > ${DEST_DIR}/cargo-project.elv"
    info ""
    info "  3. Add to ~/.config/elvish/rc.elv:"
    info "     use ${DEST_DIR}/cargo-project"
    exit 0
    ;;

*)
    die "Unknown shell: '$REQUESTED_SHELL'. Supported: bash, zsh, fish, powershell, elvish"
    ;;
esac

log "Done! Tab-completions for cargo-project are now installed for $REQUESTED_SHELL."
