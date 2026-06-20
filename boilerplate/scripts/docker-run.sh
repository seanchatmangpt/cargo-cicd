#!/usr/bin/env bash
# =============================================================================
# scripts/docker-run.sh
# Run cargo-project-serve locally via Docker.
#
# Usage:
#   ./scripts/docker-run.sh [OPTIONS]
#
# Options:
#   --version TAG   Image version/tag to run (default: latest)
#   --port PORT     Host port to bind (default: 8080)
#   --shell         Open an interactive shell instead of starting the service
#                   (uses a debug image with a shell; distroless has none)
#   --logs          Tail logs of a running container and exit
#   --stop          Stop and remove the running container
#   --env FILE      Load extra environment variables from FILE (default: .env)
#   -h, --help      Show this help message
#
# Examples:
#   ./scripts/docker-run.sh                  # start service on :8080
#   ./scripts/docker-run.sh --port 9090      # start on :9090
#   ./scripts/docker-run.sh --shell          # open debug shell
#   ./scripts/docker-run.sh --logs           # tail running container logs
#   ./scripts/docker-run.sh --stop           # stop the container
# =============================================================================
set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { printf '\033[0;32m[run]\033[0m   %s\n' "$*"; }
warn() { printf '\033[0;33m[warn]\033[0m  %s\n' "$*" >&2; }
die()  { printf '\033[0;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
  grep '^# ' "$0" | cut -c3-
  exit 0
}

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
VERSION="latest"
HOST_PORT="8080"
CONTAINER_NAME="cargo-project-serve"
DATA_VOLUME="cargo-project-data"
ENV_FILE=".env"
MODE="run"   # run | shell | logs | stop

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) VERSION="${2:?--version requires a value}"; shift 2 ;;
    --port)    HOST_PORT="${2:?--port requires a value}"; shift 2 ;;
    --shell)   MODE="shell"; shift ;;
    --logs)    MODE="logs"; shift ;;
    --stop)    MODE="stop"; shift ;;
    --env)     ENV_FILE="${2:?--env requires a value}"; shift 2 ;;
    -h|--help) usage ;;
    *)         die "Unknown argument: $1" ;;
  esac
done

IMAGE="cargo-project-serve:${VERSION}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ---------------------------------------------------------------------------
# Handle --logs and --stop modes (operate on existing container)
# ---------------------------------------------------------------------------
if [[ "${MODE}" == "logs" ]]; then
  log "Tailing logs for container '${CONTAINER_NAME}'…"
  docker logs --follow --tail 100 "${CONTAINER_NAME}"
  exit 0
fi

if [[ "${MODE}" == "stop" ]]; then
  log "Stopping container '${CONTAINER_NAME}'…"
  docker stop "${CONTAINER_NAME}" 2>/dev/null || warn "Container was not running."
  docker rm   "${CONTAINER_NAME}" 2>/dev/null || warn "Container did not exist."
  log "Done."
  exit 0
fi

# ---------------------------------------------------------------------------
# Ensure the named data volume exists
# ---------------------------------------------------------------------------
if ! docker volume inspect "${DATA_VOLUME}" &>/dev/null; then
  log "Creating data volume '${DATA_VOLUME}'…"
  docker volume create "${DATA_VOLUME}"
fi

# ---------------------------------------------------------------------------
# Remove any existing container with the same name (idempotent start)
# ---------------------------------------------------------------------------
if docker inspect "${CONTAINER_NAME}" &>/dev/null; then
  log "Removing existing container '${CONTAINER_NAME}'…"
  docker rm -f "${CONTAINER_NAME}"
fi

# ---------------------------------------------------------------------------
# Build the common docker run flags
# ---------------------------------------------------------------------------
DOCKER_ARGS=(
  --name "${CONTAINER_NAME}"
  --volume "${DATA_VOLUME}:/data"
  --publish "${HOST_PORT}:8080"
  --env RUST_LOG="${RUST_LOG:-info}"
  --env APP_ENV="${APP_ENV:-production}"
  --env DATABASE_URL="${DATABASE_URL:-sqlite:///data/app.db}"
)

# Load extra env vars from file if it exists
if [[ -f "${REPO_ROOT}/${ENV_FILE}" ]]; then
  log "Loading environment from ${ENV_FILE}"
  DOCKER_ARGS+=(--env-file "${REPO_ROOT}/${ENV_FILE}")
fi

# ---------------------------------------------------------------------------
# --shell mode: use a debug-friendly image (rust:1.86-slim) that has /bin/bash
# ---------------------------------------------------------------------------
if [[ "${MODE}" == "shell" ]]; then
  log "Opening shell in debug container (image: rust:1.86-slim-bookworm)…"
  log "Note: using the slim Rust image, not the production distroless image."
  docker run --rm -it \
    "${DOCKER_ARGS[@]}" \
    --entrypoint /bin/bash \
    "rust:1.86-slim-bookworm"
  exit 0
fi

# ---------------------------------------------------------------------------
# Normal run mode
# ---------------------------------------------------------------------------
log "Starting ${IMAGE} → http://localhost:${HOST_PORT}"

docker run \
  --detach \
  --restart unless-stopped \
  "${DOCKER_ARGS[@]}" \
  "${IMAGE}"

log "Container '${CONTAINER_NAME}' started."
log "  Logs : ./scripts/docker-run.sh --logs"
log "  Stop : ./scripts/docker-run.sh --stop"
log "  Health: http://localhost:${HOST_PORT}/health"
