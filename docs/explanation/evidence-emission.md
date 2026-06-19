<!-- BEGIN custom:full-doc -->
# Evidence Emission

This document explains what the `process-data` feature does, what events
cargo-cicd emits, and why structured event records are the foundation of
local CI/CD trust.

## What process-data is

The `process-data` feature is the event-recording layer of cargo-cicd. Every
command that changes workspace state emits a structured event record to the
evidence directory. These records are the evidence that a lawful sequence of
operations occurred.

This is not logging for debugging. It is process evidence: a durable,
machine-readable record of what happened, when, and with what outcome. The
distinction matters because evidence can be audited, replayed, and compared
against a declared process model. A debug log cannot.

## Dual-write architecture

On every `append_events()` call, cargo-cicd writes three files to
`target/cargo-cicd/evidence/`:

| File | Role |
|------|------|
| `events.ocel.json` | **Primary audit target** — OCEL 2.0 JSON read by the wpm oracle |
| `events.jsonl` | Full-fidelity journal in JSON Lines format |
| `events.xes` | Legacy XES 2.0 XML — dual-write side-channel kept for backwards compatibility |

Audit now targets OCEL 2.0. The oracle call is:

```sh
wpm audit target/cargo-cicd/evidence/events.ocel.json
```

XES is written on every run so that existing process mining integrations
(ProM, Disco, Celonis) continue to function without change, but it is not
passed to `wpm` and is not the conformance target for release gates.

## The event model

Each OCEL event has:

| Field | Type | Meaning |
|-------|------|---------|
| `ocel:activity` | string | The command in `noun:verb` form (e.g. `"status:show"`) |
| `ocel:timestamp` | RFC 3339 | When the event completed |
| `ocel:vmap` | object | Verdict, duration, and command-specific attributes |
| `ocel:typedOmap` | array | Object relationships (workspace, crate, pipeline run) |

The `verdict_claimed` field in `ocel:vmap` records whether the command
succeeded (`PASS`), warned (`WARN`), or failed (`FAIL`). This is the
field the oracle evaluates.

## Emitted events by command

| Command | Event activity | Key vmap fields |
|---------|---------------|-----------------|
| `workspace doctor` | `workspace:doctor` | `crate_count`, `warnings`, `errors` |
| `status show` | `status:show` | `dirty_files`, `pending_tests`, `publish_ready` |
| `target show` | _(none — read-only)_ | — |
| `target prune` | `target:prune` | `bytes_freed`, `artefacts_removed`, `threshold_days` |
| `test changed` | `test:changed` | `affected_crates`, `pass_count`, `fail_count` |
| `trybuild changed` | `trybuild:changed` | `affected_crates`, `compile_fail_pass`, `compile_pass_pass` |
| `git status` | _(none — read-only)_ | — |
| `git close` | `git:close` | `branch`, `trunk`, `merged_at`, `commits_merged` |
| `publish run` | `publish:run` | `crates_published`, `registry`, `versions` |

Read-only commands (`target show`, `git status`) do not emit events because
they do not change workspace state. Emitting an event for a read would produce
evidence of nothing.

## Why evidence matters

A command that reports success is not proof that the process was lawful. The
command might have skipped a step, encountered a recoverable error it suppressed,
or run in a context that made its output meaningless. Evidence emission changes
the accountability surface: the record in `target/cargo-cicd/evidence/` can be
independently verified against the declared process model.

This is the same principle that underlies object-centric process mining: trust
what the event log can prove, not what the code claims.

## The evidence directory

`target/cargo-cicd/evidence/` accumulates evidence across runs. A workspace
that has been through doctor, test, and publish will have an `events.ocel.json`
that tells that story in order. The OCEL format groups events by object
(workspace, crate) rather than by flat trace, enabling object-centric
conformance checking.

## What evidence emission does not do

- It does not transmit data to any remote service. All evidence stays on disk.
- It does not replace tests. A `test:changed` event with `verdict_claimed = "PASS"`
  means the tests passed; it does not mean the tests are sufficient.
- It does not perform conformance checking. Comparing the event log against
  a declared process model is a separate operation performed by the wpm oracle.
<!-- END custom:full-doc -->
