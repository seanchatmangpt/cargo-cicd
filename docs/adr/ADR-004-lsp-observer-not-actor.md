# ADR-004: LSP Observer, Not Actor

**Status:** Accepted
**Date:** 2026-06-03

## Context

The LSP integration provides workspace symbol information that can inform `EngineState` — knowing which symbols exist, where they are defined, and how they are used. There is a temptation to use this same channel to perform workspace mutations: renaming symbols, applying refactors, or running code actions during CI/CD pipeline execution.

## Decision

The LSP integration is an observer only. It reads workspace state to populate `EngineState`. It never:

- Mutates files
- Runs code actions or refactors
- Invokes LSP workspace/applyEdit
- Spawns processes based on LSP findings

Domain functions that need to act on LSP-derived information receive it as data via `EngineState` and make their own decisions. The LSP adapter's contract is: observe and report. Never act.

## Rationale

Mixing observation and action in the same adapter creates non-deterministic pipeline behavior. An LSP server's available actions depend on server state, file system state, and timing — none of which can be controlled in a reproducible CI/CD pipeline. Observer-only adapters are stateless, testable with mock data, and produce no side effects that would corrupt evidence logs.

## Consequences

- `LspAdapter` exposes only read methods: `workspace_symbols()`, `document_symbols()`, `go_to_definition()`, `find_references()`.
- Actions derived from LSP data are expressed as domain function outputs, not adapter side effects.
- Tests can inject mock LSP data without spinning up a language server.
- Pipeline replay is deterministic: the same input state always produces the same `EngineState`.

## Violation

If the LSP adapter performs mutations, pipeline execution becomes non-idempotent. Evidence logs would contain events that depend on LSP server timing and availability, making conformance checking non-reproducible. The declared process model cannot be validated against such logs.
