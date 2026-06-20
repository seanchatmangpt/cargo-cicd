# 0002 — Error Handling Strategy (thiserror + anyhow)

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-06-20 |
| **Supersedes** | — |
| **Superseded by** | — |

## Context

Rust's error handling ecosystem offers two main library-level approaches for producing and consuming
errors, plus the standard library's own `std::error::Error` trait. The choice affects:

- **Callsite ergonomics**: how much boilerplate is needed to propagate an error up the call stack.
- **Matchability**: whether callers can pattern-match on specific error variants to recover or
  branch.
- **Diagnostic quality**: how much context appears in logs and user-facing messages.
- **Crate boundary behaviour**: library crates should not force their error type on every consumer;
  application crates care less about this.

Two dominant crates exist in this space: `thiserror` (structured, matchable enums) and `anyhow`
(type-erased, context-chain). They are not mutually exclusive and are frequently combined.

A secondary concern is how errors cross the HTTP boundary: the Axum service must convert domain
errors into `(StatusCode, Json<ErrorResponse>)` responses, which requires the errors to be
structured enough to map to HTTP semantics.

## Decision

We will use a split strategy:

- **Library crates** (`project-core`, `project-domain`, `project-sqlite`, `project-config`): define
  errors as `thiserror` enums. Each variant is named, carries relevant fields, and implements
  `std::error::Error`. Callers can `match` on variants to recover or produce different HTTP
  responses.
- **Application boundary** (`src/main.rs`, service handlers, CLI entry points): use `anyhow` for
  rapid error propagation. Add `.context("human-readable description")` at call sites that cross a
  meaningful boundary (e.g. "loading config", "starting HTTP listener"). This context chain appears
  in logs and `--help` error output without polluting library types.
- **HTTP boundary** (`crates/service/src/error.rs`): implement `IntoResponse` for each domain error
  enum. The mapping from variant to `StatusCode` lives here and nowhere else. This is the only
  place that ties domain errors to HTTP semantics.

## Alternatives Considered

**Option A — thiserror everywhere**

Define `thiserror` enums at every layer, including `main.rs` and handlers.

- Pro: every error is matchable and structured throughout.
- Pro: no type erasure; compiler verifies exhaustiveness where `match` is used.
- Con: `main.rs` and handler code accumulates large `From` impl chains to convert between error
  types at every layer boundary.
- Con: adding context (e.g. "while doing X") requires a separate `#[error("context: {source}")]`
  wrapper variant per call site.
- Con: the library/application distinction is eroded — `main.rs`-level enum variants leak into
  library crates via re-exports.

**Option B — anyhow everywhere**

Use `anyhow::Result` and `anyhow::Error` throughout the entire codebase.

- Pro: minimum boilerplate; `?` just works everywhere.
- Pro: `.context()` adds richness at no extra type cost.
- Con: library consumers cannot `match` on specific error conditions without downcasting, which is
  fragile and not discoverable from the type signature.
- Con: the HTTP boundary cannot map error kinds to status codes without downcasting.
- Con: widely considered an anti-pattern for published library crates (the `anyhow` docs themselves
  recommend against it for libraries).

**Option C — thiserror in libraries, anyhow in application (chosen)**

Described in the Decision section.

- Pro: library crates expose structured, matchable errors that callers can handle precisely.
- Pro: application code is terse; `anyhow` context chains produce rich diagnostic messages with no
  boilerplate.
- Pro: the HTTP boundary can match on `thiserror` variants to produce correct status codes.
- Con: requires contributors to know which layer they are writing for to choose the right crate.
- Con: there is a small impedance mismatch at the boundary: `anyhow::Error` cannot be `match`ed,
  so once a domain error is wrapped in `anyhow` context it must be downcast to recover. The rule
  is: do not wrap domain errors in `anyhow` before the service layer has finished mapping them.

**Option D — Custom error trait with Box<dyn Error>**

Define a project-specific error trait and box it everywhere.

- Pro: no external dependencies.
- Con: reinvents what `thiserror` does better, with more boilerplate.
- Con: box erases the type; same matchability problem as `anyhow`.

## Consequences

### Positive

- Domain errors are typed and exhaustively matchable. Adding a new variant triggers a compile error
  at every `match` site that does not handle it, preventing silent regressions.
- `anyhow` context chains in logs make it immediately clear which operation failed and why,
  without needing to trace through a chain of `From` impls.
- The HTTP error mapping is localised to one file; changing an HTTP status code for a domain
  condition requires editing exactly one `match` arm.

### Negative / Trade-offs

- Contributors must consciously switch between `thiserror` and `anyhow` depending on which layer
  they are working in. A wrong choice is caught by code review but not by the compiler.
- `thiserror` enums can become verbose when there are many variants with subtly different context
  fields. The temptation to add a catch-all `Other(anyhow::Error)` variant should be resisted
  in library crates.

### Neutral

- Both crates are zero-cost abstractions; there is no runtime overhead compared to hand-written
  `Display` impls.
- `thiserror` and `anyhow` are maintained by the same author (David Tolnay) and are designed to
  interoperate.

## Implementation Notes

- `project-core` defines `CoreError` in `crates/core/src/error.rs`. All other library error enums
  have a `Core(#[from] CoreError)` variant.
- The HTTP error mapping is in `crates/service/src/error.rs`.
- `main.rs` uses `anyhow::Result<()>` as its return type so startup failures print a full context
  chain.
- Clippy lint `clippy::wildcard_enum_match_arm` is enabled to discourage `_ =>` catch-alls in
  domain error matches.
