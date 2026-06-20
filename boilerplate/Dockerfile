# =============================================================================
# Dockerfile — multi-stage production build for cargo-project-serve
# Rust 1.86 / Edition 2021
# =============================================================================

# ---------------------------------------------------------------------------
# Stage 1: chef — base layer with cargo-chef installed
# ---------------------------------------------------------------------------
FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef
WORKDIR /app

# ---------------------------------------------------------------------------
# Stage 2: planner — compute the dependency recipe
# ---------------------------------------------------------------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------------------------------------------------------------------------
# Stage 3: builder — compile dependencies (cached), then compile the app
# ---------------------------------------------------------------------------
FROM chef AS builder

# Bring in the pre-computed recipe so Docker can cache the dependency layer
COPY --from=planner /app/recipe.json recipe.json

# Cook (compile) only dependencies — this layer is invalidated only when
# Cargo.toml / Cargo.lock change, not when src/ changes.
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy the full source and build the application binary
COPY . .
RUN cargo build --release --bin cargo-project-serve

# ---------------------------------------------------------------------------
# Stage 4: runtime — minimal distroless image (no shell, no package manager)
# Distroless cc-debian12 ships glibc + libgcc — sufficient for a Rust binary.
# The image runs as uid=65532 (nobody) by default; no USER directive needed.
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

# Copy only the compiled binary
COPY --from=builder /app/target/release/cargo-project-serve /app/cargo-project-serve

# OCI / Docker image labels
LABEL org.opencontainers.image.title="cargo-project-serve" \
      org.opencontainers.image.description="HTTP service layer for cargo-project workspace" \
      org.opencontainers.image.source="https://github.com/your-org/cargo-project" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0" \
      org.opencontainers.image.base.name="gcr.io/distroless/cc-debian12"

# Runtime environment
ENV RUST_LOG=info \
    APP_ENV=production

EXPOSE 8080

# Health check — calls the binary's built-in health endpoint.
# Distroless has no curl/wget, so we use the binary itself.
# Interval: 30s, timeout: 5s, start-period: 10s, retries: 3
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/app/cargo-project-serve", "--health-check"]

ENTRYPOINT ["/app/cargo-project-serve"]
