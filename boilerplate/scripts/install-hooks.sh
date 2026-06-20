#!/usr/bin/env bash
# install-hooks.sh — Install git hooks from .git-hooks/ into the local repository
#
# Usage:
#   ./scripts/install-hooks.sh              # Install hooks
#   ./scripts/install-hooks.sh --uninstall  # Restore default hooks path
#   ./scripts/install-hooks.sh --list       # Show installed hooks and their status

set -euo pipefail

# ---------------------------------------------------------------------------
# Color helpers (TTY-aware)
# ---------------------------------------------------------------------------
if [[ -t 1 ]]; then
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  CYAN='\033[0;36m'
  RED='\033[0;31m'
  BOLD='\033[1m'
  RESET='\033[0m'
else
  GREEN='' YELLOW='' CYAN='' RED='' BOLD='' RESET=''
fi

info()    { echo -e "${CYAN}[hooks]${RESET} $*"; }
success() { echo -e "${GREEN}[hooks]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[hooks] WARN:${RESET} $*"; }
error()   { echo -e "${RED}[hooks] ERROR:${RESET} $*" >&2; }

# ---------------------------------------------------------------------------
# Locate repo root
# ---------------------------------------------------------------------------
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "Not inside a git repository." >&2
  exit 1
}

HOOKS_DIR="${REPO_ROOT}/.git-hooks"

# Known hooks managed by this script (in installation order)
MANAGED_HOOKS=(
  "commit-msg"
  "pre-commit"
  "pre-push"
  "prepare-commit-msg"
)

# ---------------------------------------------------------------------------
# Handle --uninstall
# ---------------------------------------------------------------------------
if [[ "${1:-}" == "--uninstall" ]]; then
  git -C "${REPO_ROOT}" config --unset core.hooksPath || true
  success "Restored default git hooks path (.git/hooks)."
  info "Git will now look for hooks in .git/hooks/ (the default)."
  exit 0
fi

# ---------------------------------------------------------------------------
# Handle --list
# ---------------------------------------------------------------------------
if [[ "${1:-}" == "--list" ]]; then
  CURRENT_PATH="$(git -C "${REPO_ROOT}" config core.hooksPath 2>/dev/null || echo '(default: .git/hooks)')"
  echo ""
  echo -e "${BOLD}Git hooks status${RESET}"
  echo "  hooks path: ${CURRENT_PATH}"
  echo ""

  if [[ ! -d "${HOOKS_DIR}" ]]; then
    warn ".git-hooks/ directory not found at ${HOOKS_DIR}"
    exit 0
  fi

  echo -e "  ${BOLD}Hook             Exists   Executable   Managed${RESET}"
  echo "  -------          ------   ----------   -------"
  for hook in "${MANAGED_HOOKS[@]}"; do
    HOOK_PATH="${HOOKS_DIR}/${hook}"
    if [[ -f "${HOOK_PATH}" ]]; then
      EXISTS="${GREEN}yes${RESET}"
      if [[ -x "${HOOK_PATH}" ]]; then
        EXEC="${GREEN}yes${RESET}"
      else
        EXEC="${RED}no ${RESET}"
      fi
      MANAGED="${GREEN}yes${RESET}"
    else
      EXISTS="${YELLOW}no ${RESET}"
      EXEC="${YELLOW}n/a${RESET}"
      MANAGED="${YELLOW}no ${RESET}"
    fi
    printf "  %-16s %-8b %-12b %b\n" "${hook}" "${EXISTS}" "${EXEC}" "${MANAGED}"
  done

  # Also list any extra (unmanaged) hooks in the directory
  while IFS= read -r hook_file; do
    hook_name="$(basename "${hook_file}")"
    IS_MANAGED=0
    for m in "${MANAGED_HOOKS[@]}"; do
      [[ "${m}" == "${hook_name}" ]] && IS_MANAGED=1 && break
    done
    if [[ "${IS_MANAGED}" == "0" ]]; then
      EXEC_STATUS="${YELLOW}n/a${RESET}"
      [[ -x "${hook_file}" ]] && EXEC_STATUS="${GREEN}yes${RESET}"
      printf "  %-16s %-8b %-12b %b\n" "${hook_name}" "${GREEN}yes${RESET}" "${EXEC_STATUS}" "${YELLOW}(extra)${RESET}"
    fi
  done < <(find "${HOOKS_DIR}" -maxdepth 1 -type f | sort)

  echo ""
  exit 0
fi

# ---------------------------------------------------------------------------
# Verify .git-hooks directory exists
# ---------------------------------------------------------------------------
if [[ ! -d "${HOOKS_DIR}" ]]; then
  error ".git-hooks/ directory not found at ${HOOKS_DIR}"
  warn "Nothing to install."
  exit 1
fi

# ---------------------------------------------------------------------------
# Make all managed hook scripts executable
# ---------------------------------------------------------------------------
info "Making hooks executable..."
INSTALLED=0
MISSING=()

for hook in "${MANAGED_HOOKS[@]}"; do
  HOOK_PATH="${HOOKS_DIR}/${hook}"
  if [[ -f "${HOOK_PATH}" ]]; then
    chmod +x "${HOOK_PATH}"
    info "  chmod +x .git-hooks/${hook}"
    INSTALLED=$(( INSTALLED + 1 ))
  else
    MISSING+=("${hook}")
  fi
done

# Also chmod any extra hooks that exist
while IFS= read -r hook_file; do
  hook_name="$(basename "${hook_file}")"
  IS_MANAGED=0
  for m in "${MANAGED_HOOKS[@]}"; do
    [[ "${m}" == "${hook_name}" ]] && IS_MANAGED=1 && break
  done
  if [[ "${IS_MANAGED}" == "0" ]]; then
    chmod +x "${hook_file}"
    info "  chmod +x .git-hooks/${hook_name} (extra)"
  fi
done < <(find "${HOOKS_DIR}" -maxdepth 1 -type f | sort)

if [[ "${#MISSING[@]}" -gt 0 ]]; then
  for m in "${MISSING[@]}"; do
    warn "Hook not found (skipped): .git-hooks/${m}"
  done
fi

# ---------------------------------------------------------------------------
# Configure git to use the .git-hooks directory
# ---------------------------------------------------------------------------
git -C "${REPO_ROOT}" config core.hooksPath .git-hooks
echo ""
success "Git hooks installed (${INSTALLED} hook(s))."
info "Hooks path set to: .git-hooks/"
info "To uninstall:  ./scripts/install-hooks.sh --uninstall"
info "To list hooks: ./scripts/install-hooks.sh --list"

# ---------------------------------------------------------------------------
# Verification — confirm each managed hook is executable
# ---------------------------------------------------------------------------
echo ""
echo -e "${BOLD}Verification:${RESET}"
ALL_OK=1
for hook in "${MANAGED_HOOKS[@]}"; do
  HOOK_PATH="${HOOKS_DIR}/${hook}"
  if [[ ! -f "${HOOK_PATH}" ]]; then
    printf "  %-22s %b\n" "${hook}" "${YELLOW}MISSING${RESET}"
    ALL_OK=0
  elif [[ -x "${HOOK_PATH}" ]]; then
    printf "  %-22s %b\n" "${hook}" "${GREEN}OK (executable)${RESET}"
  else
    printf "  %-22s %b\n" "${hook}" "${RED}NOT EXECUTABLE — run: chmod +x .git-hooks/${hook}${RESET}"
    ALL_OK=0
  fi
done

echo ""
if [[ "${ALL_OK}" == "1" ]]; then
  success "All hooks verified."
else
  warn "One or more hooks could not be verified. Check messages above."
  exit 1
fi
