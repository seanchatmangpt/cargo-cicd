# 0001 — Hexagonal Architecture for Domain Isolation

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-06-20 |
| **Supersedes** | — |
| **Superseded by** | — |

## Context

When we set up this workspace we had to choose how to organise the relationship between business
logic and infrastructure (HTTP, databases, configuration). Several pressures shaped the decision:

- **Testability**: domain logic must be unit-testable without spinning up a database or HTTP server.
- **Replaceability**: we may want to swap SQLite for Postgres, or Axum for a gRPC server, without
  rewriting business rules.
- **Compile-time separation**: if the database crate and the HTTP crate are independent of each
  other, they can be compiled in parallel and evolved by different contributors without merge
  conflicts.
- **Clear ownership**: contributors should be able to answer "where does this code belong?" without
  guessing.

The standard Rust response to these pressures is some form of the ports-and-adapters (hexagonal)
pattern, but there are several ways to organise it in a Cargo workspace.

## Decision

We will organise the workspace as a hexagonal architecture with the following layer rules:

1. `project-core` — shared primitives (error types, `Result` alias, newtypes). No domain logic.
2. `project-domain` — all business entities, value objects, and **port traits** (async-trait
   interfaces for repositories and external services). Depends only on `project-core`.
3. Adapter crates (`project-sqlite`, `project-mcp-server`, …) — implement the port traits defined
   in `project-domain`. They depend on `project-domain` but are unknown to it.
4. `project-service` — Axum HTTP layer. Depends on `project-domain` and `project-config`. Receives
   adapter implementations via dependency injection at startup; does not `use project-sqlite`
   directly.
5. `src/` (binary) — wires the adapters into the service. The only crate that imports everything.

Dependency direction: `src` → `project-service` → `project-domain` → `project-core`.
Adapters plug in at the `src` level; they are never imported by service or domain.

## Alternatives Considered

**Option A — Single flat crate**

Keep everything in `src/` with module-level separation (`src/domain/`, `src/db/`, `src/http/`).

- Pro: simpler initial setup; no workspace overhead.
- Pro: no cross-crate `pub` leakage concerns.
- Con: Rust's module system does not enforce dependency direction — a function in `src/db/` can
  call `src/http/` and the compiler will not object.
- Con: as the codebase grows, untangling a flat crate is significantly harder than merging two
  clean crates.
- Con: the entire project recompiles on any change, slowing iteration.

**Option B — Domain crate + all-in-one service crate**

Two crates: `project-domain` and everything else in one `project-app` crate.

- Pro: fewer crates to manage.
- Con: HTTP and database code are compiled together; parallel compilation gains are lost.
- Con: swapping the database layer requires editing the same crate that contains HTTP handlers.

**Option C — Hexagonal with separate adapter crates (chosen)**

Described above.

- Pro: compiler enforces dependency direction at the crate boundary.
- Pro: adapter crates compile independently; changing SQLite code does not invalidate the service
  crate's build cache.
- Pro: domain crate can be tested with `mockall` or in-memory fakes without any real adapter.
- Con: more `Cargo.toml` files to maintain.
- Con: `pub` visibility must be managed carefully to avoid leaking internals across crates.

## Consequences

### Positive

- The domain layer has no I/O dependencies; its tests run in milliseconds.
- Adding a new database backend (e.g. Postgres) means writing a new adapter crate; the domain and
  service crates are untouched.
- Parallel workspace compilation is effective because the four library crates have no fan-in
  dependency on a shared mutable crate (other than `project-core`, which changes rarely).

### Negative / Trade-offs

- Contributors must understand which crate a new type belongs to before writing code.
- Dependency injection in Rust without a DI framework requires passing `Arc<dyn PortTrait>` through
  constructors, which is more boilerplate than in garbage-collected languages.
- `async-trait` (or RPITIT on Rust ≥ 1.75) adds syntactic overhead to port definitions.

### Neutral

- `cargo deny` bans are used to enforce the dependency rules at CI time. If a PR accidentally
  imports a database crate from the domain layer, CI fails.

## Implementation Notes

- Port traits live in `crates/domain/src/ports.rs`.
- The SQLite adapter implementations live in `crates/sqlite/src/`.
- Dependency injection wiring is in `src/main.rs` (`fn build_router`).
- See the crate topology diagram in [ARCHITECTURE.md](../../ARCHITECTURE.md#crate-topology).
