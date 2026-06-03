# Evidence Format Reference

When the `process-data` feature is enabled, `cargo-cicd` emits structured
events in XES (eXtensible Event Stream) format. This document describes the
event schema and fields.

## XES overview

XES is an IEEE-standard format for process event logs. Each event belongs to
a trace (a sequence of events for one logical entity) and carries a set of
typed attributes.

`cargo-cicd` uses XES 2.0. Events are written as newline-delimited XML to
the configured output path.

## File location

Default: `.cicd/events.xes` relative to the workspace root.

Configure in `cicd.toml`:

```toml
[process_data]
output_path = ".cicd/events.xes"
```

## Trace structure

Each workspace session is one trace. The trace ID is derived from the
workspace root path and the session start time.

```xml
<trace>
  <string key="concept:name" value="cargo-cicd:my-project:2026-06-02T12:00:00Z"/>
  <string key="workspace:name" value="my-project"/>
  <string key="workspace:root" value="/home/user/my-project"/>
  <!-- events -->
</trace>
```

## Event fields

Every event has these mandatory fields:

| Key | Type | Description |
|-----|------|-------------|
| `concept:name` | string | Event name (e.g., `StatusShowEvent`) |
| `time:timestamp` | date | ISO 8601 timestamp when the event completed |
| `lifecycle:transition` | string | Always `"complete"` for cargo-cicd events |
| `org:resource` | string | The command that produced the event |

### StatusShowEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:dirty_files` | int | Count of uncommitted files |
| `cicd:publish_ready` | bool | Whether workspace is publish-ready |
| `cicd:branch` | string | Current git branch |

### TargetShowEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:size_bytes` | int | Total target directory size in bytes |
| `cicd:oldest_artefact_days` | int | Age of oldest artefact in days |

### TargetPruneEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:bytes_freed` | int | Bytes removed by prune |
| `cicd:artefacts_removed` | int | Count of removed artefacts |
| `cicd:threshold_days` | int | Age threshold used for pruning |

### TestChangedEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:changed_crates` | string | Comma-separated list of tested crates |
| `cicd:tests_passed` | int | Count of passing tests |
| `cicd:tests_failed` | int | Count of failing tests |
| `cicd:verdict` | string | `"pass"` or `"fail"` |

### TrybuildChangedEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:changed_crates` | string | Crates with changed fixtures |
| `cicd:fixtures_pass` | int | Passing fixture count |
| `cicd:fixtures_fail` | int | Failing fixture count |
| `cicd:verdict` | string | `"pass"` or `"fail"` |

### GitCloseEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:branch_closed` | string | Branch that was closed |
| `cicd:trunk_branch` | string | Branch merged into |
| `cicd:commit_hash` | string | Merge commit hash |

### PublishRunEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:crates_published` | string | Comma-separated crate names and versions |
| `cicd:dry_run` | bool | Whether this was a dry run |
| `cicd:verdict` | string | `"pass"` or `"fail"` |

### WorkspaceDoctorEvent

| Key | Type | Description |
|-----|------|-------------|
| `cicd:issues_found` | int | Count of structural issues |
| `cicd:verdict` | string | `"pass"` or `"fail"` |

## Example XES document

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="2.0" xmlns="http://www.xes-standard.org/">
  <trace>
    <string key="concept:name" value="cargo-cicd:my-project:2026-06-02T12:00:00Z"/>
    <event>
      <string key="concept:name" value="StatusShowEvent"/>
      <date key="time:timestamp" value="2026-06-02T12:00:01Z"/>
      <string key="lifecycle:transition" value="complete"/>
      <string key="org:resource" value="cargo cicd status show"/>
      <int key="cicd:dirty_files" value="0"/>
      <boolean key="cicd:publish_ready" value="true"/>
      <string key="cicd:branch" value="main"/>
    </event>
  </trace>
</log>
```
