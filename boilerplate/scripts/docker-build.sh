#!/usr/bin/env bash
# =============================================================================
# scripts/docker-build.sh
# Build the cargo-project-serve Docker image with version tagging.
#
# Usage:
#   ./scripts/docker-build.sh [OPTIONS]
#
# Options:
#   --push          Push the built image to the registry
#   --registry REG  Override the image registry (default: none / local)
#   --no-cache      Build without Docker layer cache
#   --platform P    Target platform (e.g. linux/amd64,linux/arm64)
#   -h, --help      Show this help message
# =============================================================================
set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { printf '\033[0;32m[build]\033[0m %s\n' "$*"; }
warn() { printf '\033[0;33m[warn]\033[0m  %s\n' "$*" >&2; }
die()  { printf '\033[0;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
  grep '^# ' "$0" | cut -c3-
  exit 0
}

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
PUSH=false
REGISTRY=""
NO_CACHE=""
PLATFORM="linux/amd64"
DOCKERFILE="Dockerfile"

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --push)       PUSH=true; shift ;;
    --registry)   REGISTRY="${2:?--registry requires a value}"; shift 2 ;;
    --no-cache)   NO_CACHE="--no-cache"; shift ;;
    --platform)   PLATFORM="${2:?--platform requires a value}"; shift 2 ;;
    -h|--help)    usage ;;
    *)            die "Unknown argument: $1" ;;
  esac
done

# ---------------------------------------------------------------------------
# Derive version from Cargo.toml (workspace root)
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

VERSION="$(grep -m1 '^version' Cargo.toml | sed 's/.*= *"\(.*\)"/\1/')"
if [[ -z "${VERSION}" ]]; then
  warn "Could not read version from Cargo.toml; falling back to 'dev'"
  VERSION="dev"
fi

GIT_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
BUILD_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

log "Version   : ${VERSION}"
log "Git commit: ${GIT_COMMIT}"
log "Build date: ${BUILD_DATE}"
log "Platform  : ${PLATFORM}"

# ---------------------------------------------------------------------------
# Construct image name(s)
# ---------------------------------------------------------------------------
BASE_NAME="cargo-project-serve"
if [[ -n "${REGISTRY}" ]]; then
  IMAGE_BASE="${REGISTRY}/${BASE_NAME}"
else
  IMAGE_BASE="${BASE_NAME}"
fi

TAG_VERSION="${IMAGE_BASE}:${VERSION}"
TAG_LATEST="${IMAGE_BASE}:latest"
TAG_COMMIT="${IMAGE_BASE}:${GIT_COMMIT}"

log "Image tags:"
log "  ${TAG_VERSION}"
log "  ${TAG_LATEST}"
log "  ${TAG_COMMIT}"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
log "Building image…"

docker build \
  ${NO_CACHE} \
  --platform "${PLATFORM}" \
  --file "${DOCKERFILE}" \
  --build-arg BUILD_DATE="${BUILD_DATE}" \
  --build-arg GIT_COMMIT="${GIT_COMMIT}" \
  --build-arg VERSION="${VERSION}" \
  --tag "${TAG_VERSION}" \
  --tag "${TAG_LATEST}" \
  --tag "${TAG_COMMIT}" \
  "${REPO_ROOT}"

# ---------------------------------------------------------------------------
# Report image size
# ---------------------------------------------------------------------------
log "Build complete."
IMAGE_SIZE="$(docker image inspect "${TAG_VERSION}" --format '{{.Size}}' | awk '{printf "%.1f MB", $1/1024/1024}')"
log "Image size: ${IMAGE_SIZE}"

# ---------------------------------------------------------------------------
# Optional push
# ---------------------------------------------------------------------------
if [[ "${PUSH}" == "true" ]]; then
  if [[ -z "${REGISTRY}" ]]; then
    die "--push requires --registry to be set (won't push to unqualified names)"
  fi
  log "Pushing ${TAG_VERSION}…"
  docker push "${TAG_VERSION}"
  log "Pushing ${TAG_LATEST}…"
  docker push "${TAG_LATEST}"
  log "Pushing ${TAG_COMMIT}…"
  docker push "${TAG_COMMIT}"
  log "Push complete."
else
  log "Skipping push (pass --push --registry <reg> to push)."
fi

log "Done."
