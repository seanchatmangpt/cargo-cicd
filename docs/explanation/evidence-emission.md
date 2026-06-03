<!-- BEGIN custom:full-doc -->
# Evidence Emission

This document explains what the `process-data` feature does, what events
cargo-cicd emits, and why structured event records are the foundation of
local CI/CD trust.

## What process-data is

The `process-data` feature is the event-recording layer of cargo-cicd. Every
command that changes workspace state emits a structured event record to
`cicd.toml`. These records are the evidence that a lawful sequence of
operations occurred.

This is not logging for debugging. It is process evidence: a durable,
machine-readable record of what happened, when, and with what outcome. The
distinction matters because evidence can be audited, replayed, and compared
against a declared process model. A debug log cannot.

## The event model

Each event has:

| Field | Type | Meaning |
|-------|------|---------|
| `event_type` | string | The name of the event (e.g., `WorkspaceDoctorEvent`) |
| `timestamp` | RFC 3339 | When the event was emitted |
| `outcome` | `pass` \| `fail` \| `skip` | Whether the operation succeeded |
| `payload` | object | Command-specific structured data |

Events are appended to `cicd.toml` under the `[[events]]` array. They are
never overwritten or deleted by cargo-cicd.

## Emitted events by command

| Command | Event emitted | Key payload fields |
|---------|--------------|-------------------|
| `workspace doctor` | `WorkspaceDoctorEvent` | `crate_count`, `warnings`, `errors` |
| `status show` | `StatusShowEvent` | `dirty_files`, `pending_tests`, `publish_ready` |
| `target show` | _(none — read-only)_ | — |
| `target prune` | `TargetPruneEvent` | `bytes_freed`, `artefacts_removed`, `threshold_days` |
| `test changed` | `TestChangedEvent` | `affected_crates`, `pass_count`, `fail_count` |
| `trybuild changed` | `TrybuildChangedEvent` | `affected_crates`, `compile_fail_pass`, `compile_pass_pass` |
| `git status` | _(none — read-only)_ | — |
| `git close` | `GitCloseEvent` | `branch`, `trunk`, `merged_at`, `commits_merged` |
| `publish run` | `PublishRunEvent` | `crates_published`, `registry`, `versions` |

Read-only commands (`target show`, `git status`) do not emit events because
they do not change workspace state. Emitting an event for a read would produce
evidence of nothing.

## Why evidence matters

A command that reports success is not proof that the process was lawful. The
command might have skipped a step, encountered a recoverable error it suppressed,
or run in a context that made its output meaningless. Evidence emission changes
the accountability surface: the record in `cicd.toml` can be independently
verified against the declared process model.

This is the same principle that underlies object-centric process mining: trust
what the event log can prove, not what the code claims.

## The cicd.toml event log

`cicd.toml` is append-only from cargo-cicd's perspective. It records the
accumulation of workspace lifecycle events. A workspace that has been through
doctor, test, and publish will have a `cicd.toml` that tells that story in
order.

Example fragment:

```toml
[[events]]
event_type = "WorkspaceDoctorEvent"
timestamp  = "2026-06-02T09:14:00Z"
outcome    = "pass"

  [events.payload]
  crate_count = 3
  warnings    = 0
  errors      = 0

[[events]]
event_type = "PublishRunEvent"
timestamp  = "2026-06-02T09:22:00Z"
outcome    = "pass"

  [events.payload]
  crates_published = ["my-api"]
  registry         = "crates.io"
  versions         = ["26.6.2"]
```

## What evidence emission does not do

- It does not transmit data to any remote service. All evidence stays in
  `cicd.toml` on disk.
- It does not replace tests. A `TestChangedEvent` with `outcome = "pass"`
  means the tests passed; it does not mean the tests are sufficient.
- It does not perform conformance checking. Comparing the event log against
  a declared process model is a separate operation outside cargo-cicd's scope.
<!-- END custom:full-doc -->
