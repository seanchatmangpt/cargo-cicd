# Evidence Format Reference

When the `process-data` feature is enabled, `cargo-cicd` emits structured
process evidence on every command run. This document describes the three files
emitted per run, the primary OCEL 2.0 schema, and the legacy XES side-channel.

## Primary format: OCEL 2.0

The primary evidence format is **OCEL 2.0** (Object-Centric Event Log), a JSON
format designed for process mining tools and external oracle adjudication. The
wpm oracle reads `events.ocel.json` directly:

```sh
wpm audit target/cargo-cicd/evidence/events.ocel.json
```

OCEL 2.0 was chosen over XES because it natively supports object-centric
relationships (workspaces, crates, pipeline runs) without forcing them into a
flat trace structure.

## Files emitted per run

Every call to `append_events()` writes three files to
`target/cargo-cicd/evidence/`:

| File | Role | Format |
|------|------|--------|
| `events.ocel.json` | **Primary** — wpm oracle audit target | OCEL 2.0 JSON |
| `events.jsonl` | Full-fidelity journal — all events, machine-readable | JSON Lines |
| `events.xes` | Legacy side-channel — kept for backwards compatibility | XES 2.0 XML |

Configure the output directory in `cicd.toml`:

```toml
[process_data]
output_dir = "target/cargo-cicd/evidence"
```

## OCEL 2.0 schema

An `OcelLog` is the root document. It wraps a map of `OcelEvent` objects, a
map of `OcelObject` objects, and type-level metadata.

### OcelLog (root document)

```json
{
  "ocel:version": "2.0",
  "ocel:events": { ... },
  "ocel:objects": { ... },
  "ocel:event-types": [],
  "ocel:object-types": []
}
```

### OcelEvent

Each event is keyed by its unique event ID:

```json
"evt-status-show-20260619134507123Z": {
  "ocel:activity": "status:show",
  "ocel:timestamp": "2026-06-19T13:45:07.123Z",
  "ocel:vmap": {
    "verdict_claimed": "PASS",
    "duration_ms": 42,
    "workspace_id": "cargo-cicd",
    "trace_class": "live_workspace"
  },
  "ocel:typedOmap": [
    { "objectId": "workspace:cargo-cicd", "qualifier": "executed-in" }
  ]
}
```

### OcelEvent fields

| Field | Type | Description |
|-------|------|-------------|
| `ocel:activity` | string | Command in `noun:verb` form (e.g. `"status:show"`) |
| `ocel:timestamp` | string | ISO-8601 UTC when the event completed |
| `ocel:vmap` | object | Attribute map — verdict, duration, workspace context |
| `ocel:typedOmap` | array | Object relationships (workspace, crate, pipeline run) |

### Standard vmap attributes

| Key | Type | Description |
|-----|------|-------------|
| `verdict_claimed` | string | `"PASS"`, `"WARN"`, or `"FAIL"` |
| `duration_ms` | number | Elapsed milliseconds (completion events) |
| `workspace_id` | string | Workspace identifier |
| `trace_class` | string | `"live_workspace"` or `"pipeline_run"` |

## Oracle call

After events are written, `status audit` reads `events.ocel.json` and passes
it to the wpm oracle:

```sh
wpm audit target/cargo-cicd/evidence/events.ocel.json
# Output: Accept / Refuse / Blocked
```

`pipeline run` writes `audit-events.ocel.json` and passes it to
`wpm receipt_verify_ocel2()`. There is no XES fallback in either path.

## Event fields by command

### StatusShowEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `dirty_files` | int | Count of uncommitted files |
| `publish_ready` | bool | Whether workspace is publish-ready |
| `branch` | string | Current git branch |

### TargetShowEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `size_bytes` | int | Total target directory size in bytes |
| `oldest_artefact_days` | int | Age of oldest artefact in days |

### TargetPruneEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `bytes_freed` | int | Bytes removed by prune |
| `artefacts_removed` | int | Count of removed artefacts |
| `threshold_days` | int | Age threshold used for pruning |

### TestChangedEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `changed_crates` | string | Comma-separated list of tested crates |
| `tests_passed` | int | Count of passing tests |
| `tests_failed` | int | Count of failing tests |
| `verdict` | string | `"pass"` or `"fail"` |

### TrybuildChangedEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `changed_crates` | string | Crates with changed fixtures |
| `fixtures_pass` | int | Passing fixture count |
| `fixtures_fail` | int | Failing fixture count |
| `verdict` | string | `"pass"` or `"fail"` |

### GitCloseEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `branch_closed` | string | Branch that was closed |
| `trunk_branch` | string | Branch merged into |
| `commit_hash` | string | Merge commit hash |

### PublishRunEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `crates_published` | string | Comma-separated crate names and versions |
| `dry_run` | bool | Whether this was a dry run |
| `verdict` | string | `"pass"` or `"fail"` |

### WorkspaceDoctorEvent

| vmap key | Type | Description |
|----------|------|-------------|
| `issues_found` | int | Count of structural issues |
| `verdict` | string | `"pass"` or `"fail"` |

## Example OCEL 2.0 document

```json
{
  "ocel:version": "2.0",
  "ocel:events": {
    "evt-status-show-20260619134507123Z": {
      "ocel:activity": "status:show",
      "ocel:timestamp": "2026-06-19T13:45:07.123Z",
      "ocel:vmap": {
        "verdict_claimed": "PASS",
        "dirty_files": 0,
        "publish_ready": true,
        "branch": "main",
        "duration_ms": 42,
        "workspace_id": "my-project",
        "trace_class": "live_workspace"
      },
      "ocel:typedOmap": [
        { "objectId": "workspace:my-project", "qualifier": "executed-in" }
      ]
    }
  },
  "ocel:objects": {
    "workspace:my-project": {
      "ocel:type": "workspace",
      "ocel:ovmap": {
        "root": "/home/user/my-project"
      }
    }
  },
  "ocel:event-types": [],
  "ocel:object-types": []
}
```

## Legacy: XES side-channel

`events.xes` is written on every run alongside `events.ocel.json`. It is kept
for backwards compatibility with process mining tools that consume XES (ProM,
Disco, Celonis) but it is **not** the oracle audit target.

See [XES format](xes-format.md) for the full XES
attribute contract. The primary oracle call uses `events.ocel.json`; do not
pass `events.xes` to `wpm audit` in new integrations.
