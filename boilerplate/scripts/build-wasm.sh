#!/usr/bin/env bash
# build-wasm.sh — Build the project-wasm crate with wasm-pack and optionally
# run wasm-opt for an additional size reduction pass.
#
# Usage:
#   bash scripts/build-wasm.sh [--target <target>] [--no-opt] [--release|--dev]
#
# Options:
#   --target <target>   wasm-pack target: web (default), bundler, nodejs, no-modules
#   --no-opt            Skip wasm-opt even if it is installed
#   --dev               Build in debug mode (faster compile, larger .wasm)
#   --release           Build in release mode (default)
#
# Environment variables:
#   WASM_CRATE_DIR      Path to the wasm crate (default: crates/wasm)
#   WASM_OUT_DIR        Output directory for pkg/ (default: <WASM_CRATE_DIR>/pkg)

set -euo pipefail

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
WASM_CRATE_DIR="${WASM_CRATE_DIR:-crates/wasm}"
WASM_OUT_DIR="${WASM_OUT_DIR:-${WASM_CRATE_DIR}/pkg}"
BUILD_TARGET="web"
BUILD_MODE="--release"
RUN_WASM_OPT=true

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      BUILD_TARGET="$2"
      shift 2
      ;;
    --no-opt)
      RUN_WASM_OPT=false
      shift
      ;;
    --dev)
      BUILD_MODE="--dev"
      shift
      ;;
    --release)
      BUILD_MODE="--release"
      shift
      ;;
    -h|--help)
      sed -n '2,25p' "$0"   # print the usage comment at the top of this file
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { echo "[build-wasm] $*"; }
warn() { echo "[build-wasm] WARN: $*" >&2; }
die()  { echo "[build-wasm] ERROR: $*" >&2; exit 1; }

require_cmd() {
  command -v "$1" &>/dev/null || die "$1 is not installed. $2"
}

human_size() {
  # Print a file size in human-readable form (K/M/G) without coreutils -h
  local bytes="$1"
  if   (( bytes >= 1073741824 )); then printf "%.1f GiB" "$(echo "scale=1; $bytes/1073741824" | bc)"
  elif (( bytes >= 1048576 ));    then printf "%.1f MiB" "$(echo "scale=1; $bytes/1048576"    | bc)"
  elif (( bytes >= 1024 ));       then printf "%.1f KiB" "$(echo "scale=1; $bytes/1024"       | bc)"
  else                                 printf "%d B"      "$bytes"
  fi
}

# ---------------------------------------------------------------------------
# Pre-flight checks
# ---------------------------------------------------------------------------
require_cmd wasm-pack \
  "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"

[[ -f "${WASM_CRATE_DIR}/Cargo.toml" ]] \
  || die "Cargo.toml not found at '${WASM_CRATE_DIR}'. Run this script from the workspace root."

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
log "Building ${WASM_CRATE_DIR} → ${WASM_OUT_DIR}"
log "  target: ${BUILD_TARGET}"
log "  mode:   ${BUILD_MODE}"

wasm-pack build \
  "${BUILD_MODE}" \
  --target "${BUILD_TARGET}" \
  --out-dir "${WASM_OUT_DIR}" \
  "${WASM_CRATE_DIR}"

WASM_FILE="${WASM_OUT_DIR}/$(ls "${WASM_OUT_DIR}"/*.wasm 2>/dev/null | head -1 | xargs basename 2>/dev/null || true)"

if [[ -z "${WASM_FILE}" || ! -f "${WASM_FILE}" ]]; then
  die "No .wasm file found in ${WASM_OUT_DIR} after build."
fi

PRE_SIZE=$(wc -c < "${WASM_FILE}")
log "Build complete. .wasm size before wasm-opt: $(human_size "${PRE_SIZE}")"

# ---------------------------------------------------------------------------
# wasm-opt (optional size reduction)
# ---------------------------------------------------------------------------
if [[ "${RUN_WASM_OPT}" == "true" ]]; then
  if command -v wasm-opt &>/dev/null; then
    log "Running wasm-opt -Oz …"
    OPTIMISED_FILE="${WASM_FILE%.wasm}.opt.wasm"
    wasm-opt -Oz "${WASM_FILE}" -o "${OPTIMISED_FILE}"

    POST_SIZE=$(wc -c < "${OPTIMISED_FILE}")
    SAVED=$(( PRE_SIZE - POST_SIZE ))
    log "wasm-opt complete."
    log "  Before : $(human_size "${PRE_SIZE}")"
    log "  After  : $(human_size "${POST_SIZE}")"
    log "  Saved  : $(human_size "${SAVED}")"

    # Replace the original .wasm with the optimised one so the pkg/ is self-contained
    mv "${OPTIMISED_FILE}" "${WASM_FILE}"
    log "Replaced ${WASM_FILE} with optimised binary."
  else
    warn "wasm-opt not found; skipping optimisation pass."
    warn "Install binaryen: brew install binaryen  or  apt install binaryen"
  fi
else
  log "Skipping wasm-opt (--no-opt specified)."
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
log ""
log "Output artefacts in ${WASM_OUT_DIR}:"
ls -lh "${WASM_OUT_DIR}" | grep -v '^total' | awk '{print "  " $0}'

FINAL_SIZE=$(wc -c < "${WASM_FILE}")
log ""
log "Final .wasm size: $(human_size "${FINAL_SIZE}")"
log "Done."
