# Architecture

> This document describes the high-level structure of PROJECT.
> It is intended for contributors who want to understand how the codebase is organised
> before making changes. Update it when you make significant structural changes.
>
> Inspired by the approach from [rust-analyzer](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md).

## Bird's Eye View

[2-3 sentence summary of what the project does and its core design philosophy.]

The codebase is organised as a Cargo workspace with a thin binary entry-point (`src/`) and a set of
focused library crates under `crates/`. All business logic is pushed into `project-domain`;
`project-service` and the binary depend on that domain, never the other way around.

## Crate Topology

```
src/                   ← binary entry-point (main.rs) + optional lib.rs public API
crates/
├── core/              ← shared types, Result alias, root error enum
├── domain/            ← business entities, value objects, port traits (async-trait)
├── service/           ← Axum HTTP service, handlers, middleware, router
├── config/            ← layered configuration (env vars / config file / CLI flags)
├── sqlite/            ← SQLite repository implementations of port traits
└── mcp-server/        ← Model Context Protocol server (optional; drop if unneeded)
```

**Dependency rules** (enforced by `cargo deny` bans and `[workspace.lints]`):

- `project-core` must not depend on any other workspace crate.
- `project-domain` may depend only on `project-core`.
- `project-sqlite` depends on `project-domain` (implements port traits defined there).
- `project-service` depends on `project-domain` and `project-config`; never on `project-sqlite`
  directly — it receives repository implementations via dependency injection.
- `src/` (the binary) wires everything together: it instantiates the SQLite repositories and passes
  them into the service.
- No circular dependencies. Run `cargo deny check` to verify.

## Key Data Flows

### HTTP Request Handling

```
HTTP request
  → tower middleware stack (TraceLayer, TimeoutLayer, CorsLayer, RequestIdLayer)
  → Axum router (crates/service/src/router.rs)
  → handler function (crates/service/src/handlers/)
  → domain service method (crates/domain/src/)
  → port trait call (async-trait, resolved at runtime to a concrete adapter)
  → SQLite repository (crates/sqlite/src/)
  → HTTP response (Json / StatusCode)
```

Errors propagate upward as typed `thiserror` enums from the domain layer; the service boundary
converts them to `(StatusCode, Json<ErrorResponse>)` via an `IntoResponse` impl.

### Configuration Loading

```
CLI flags (clap, highest priority)
  → environment variables (SCREAMING_SNAKE_CASE prefix)
  → config file (TOML, path from --config or PROJECT_CONFIG_FILE)
  → compiled-in defaults (lowest priority)
```

All layers are merged by `crates/config/src/loader.rs` into a single `Config` struct. Later
layers in the list above win when keys conflict. `Config` is constructed once at startup and
passed as shared state into the service.

### Error Propagation

```
Domain operation fails
  → thiserror enum variant (structured, matchable)
  → service handler catches and maps to HTTP status + error body
  → anyhow::Context wraps at application boundary for rich diagnostics in logs
```

## Important Modules

| Module | Location | Purpose |
|--------|----------|---------|
| `CoreError` | `crates/core/src/error.rs` | Root error type; all crates' errors convert to this |
| Port traits | `crates/domain/src/ports.rs` | `async-trait` interfaces; adapters implement these |
| Entities | `crates/domain/src/entities.rs` | Aggregate roots and value objects |
| `Router` | `crates/service/src/router.rs` | All HTTP routes registered in one place |
| `Config` schema | `crates/config/src/schema.rs` | Deserialisation target for all configuration layers |
| `Loader` | `crates/config/src/loader.rs` | Merges CLI / env / file / defaults |
| `inject_default_verbs` | `src/main.rs` | Maps bare nouns to their default verb (CLI shortcut) |
| Noun registry | `src/nouns/mod.rs` | Registers every `NounCommand` with the clap builder |

## Cross-Cutting Concerns

### Error Handling Strategy

- **Library crates** (`core`, `domain`, `sqlite`): `thiserror` enums. Errors are structured and
  matchable by callers. Each variant carries enough context to produce a useful message without
  `anyhow`.
- **Application / binary** (`src/main.rs`, service handlers): `anyhow` for context chains and rapid
  error propagation. `.context("doing X")` is preferred over bare `?` at every call site that adds
  useful information.
- **HTTP boundary**: service handlers implement `IntoResponse` for domain error enums, converting
  them to `(StatusCode, Json<ErrorResponse>)`. The mapping lives in
  `crates/service/src/error.rs`.

See [ADR 0002](docs/decisions/0002-error-handling-strategy.md) for the full rationale.

### Observability

- `tracing` spans wrap every request handler (via `tower-http`'s `TraceLayer`) and every
  significant background task.
- In development (`RUST_LOG=debug`), logs are pretty-printed with `tracing-subscriber`'s
  `fmt` layer. In production, `EnvFilter` + JSON output is enabled.
- Span IDs are propagated through `tower-http`'s `RequestIdLayer` so a single request can be
  correlated across log lines.
- Add `#[tracing::instrument]` to any function where latency or argument capture is useful.
  Avoid it on hot inner loops.

### Testing Layers

1. **Unit tests** — `#[cfg(test)]` blocks inside each module. Pure logic, no I/O. Run with
   `cargo test --workspace`.
2. **Integration tests** — `tests/` directory. Use `assert_cmd` for CLI tests and `tempfile`
   for isolated filesystem workspaces. Each test is self-contained.
3. **Property tests** — `tests/property/` with `proptest`. State invariants verified over
   randomly generated inputs. Run with `cargo test --test property`.
4. **Snapshot tests** — `tests/snapshot/` and `<test>.snap` files with `cargo-insta`. Pins CLI
   output contracts. Accept or reject snapshots with `cargo insta review`.
5. **Benchmarks** — `tests/benches/` with `criterion`. Guard against performance regressions.
   Run with `cargo bench`.

Feature-flag tests:

```sh
cargo test --features process-data
cargo test --features autonomic
cargo test --features advanced
cargo test --features advanced,autonomic
```

## When to Split vs. Merge Crates

**Split a crate when:**

- The code has a distinct public API boundary that external consumers might use independently.
- Two subsystems have non-overlapping compile-time dependencies, so keeping them together forces
  unnecessary recompilation.
- Different teams or ownership areas own different pieces.

**Merge crates when:**

- The crate is small (< 500 LOC) and only ever used by one other crate internally.
- Splitting created more indirection than clarity (excessive `pub use` re-exports, thin wrappers).
- Workspace build time is dominated by crate-graph overhead rather than code complexity.

**Rule of thumb**: Prefer fewer, cohesive crates over many tiny ones. Tokio consolidated their
crate count significantly and improved the contributor experience as a result.

## Decision Records

See [docs/decisions/](docs/decisions/) for Architecture Decision Records (ADRs).
Each significant architectural choice is recorded with context, options considered, and rationale.
New architectural changes should be accompanied by an ADR before the PR is merged.
