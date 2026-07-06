# XES 2.0 Specification for Process Evidence in Rust CI/CD

**Format:** IEEE XES (Extensible Event Stream)  
**Standard:** ISO/IEC 20880:2013  
**Version:** 2.0  
**Companion format:** JSONL (JSON Lines)

---

> **Note (2026-06-19):** XES is now a legacy side-channel. cargo-cicd emits
> `events.ocel.json` (OCEL 2.0) as the primary audit format. `events.xes`
> continues to be written on every run for backwards compatibility with process
> mining tools (ProM, Disco, Celonis), but it is not the oracle audit target.
> New integrations should use `wpm audit events.ocel.json`, not `events.xes`.

## 1. Overview

XES (eXtensible Event Stream) is an international standard for process event log interchange. It enables process mining tools such as ProM, Disco, and Celonis to discover, conformance-check, and enhance process models from real execution traces.

This document defines the XES 2.0 attribute contract for process evidence emitted by Rust CI/CD pipelines. It is **vendor-neutral**: any Rust CI/CD tool that follows this contract will produce event logs compatible with the above mining tools.

---

## 2. XES 2.0 Format (ISO/IEC 20880:2013)

### 2.1 Root Element

The log root element must carry the XES version and namespace:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="2.0" xmlns:xes="http://www.xes-standard.org/">
```

**Required attributes on `<log>`:**

| Attribute | Required value |
|---|---|
| `xes.version` | `"2.0"` |
| `xmlns:xes` | `"http://www.xes-standard.org/"` |

### 2.2 Extensions

Declare the following extensions within the `<log>` element. They define the standard key namespaces used in trace and event attributes:

```xml
<extension name="Concept"       prefix="concept"   uri="http://www.xes-standard.org/concept.xesext"/>
<extension name="Time"          prefix="time"       uri="http://www.xes-standard.org/time.xesext"/>
<extension name="Lifecycle"     prefix="lifecycle"  uri="http://www.xes-standard.org/lifecycle.xesext"/>
<extension name="Organizational" prefix="org"       uri="http://www.xes-standard.org/org.xesext"/>
```

### 2.3 Trace Element

Each `<trace>` groups events that belong to the same case (process instance):

```xml
<trace>
  <string key="concept:name" value="{case_id}"/>
  <!-- Workspace context attributes -->
  ...
  <event>...</event>
  <event>...</event>
</trace>
```

**Required `<trace>` attributes:**

| Key | Type | Description |
|---|---|---|
| `concept:name` | string | Case identifier (same as `case_id`) |
| `cargo_cicd:workspace_id` | string | Unique workspace identifier (e.g. repo name) |
| `cargo_cicd:workspace_root` | string | Absolute path to the workspace root |
| `cargo_cicd:git_branch` | string | Current git branch (`HEAD-detached` if detached) |
| `cargo_cicd:git_commit_sha` | string | Short git commit SHA (`HEAD-not-resolved` if unavailable) |
| `cargo_cicd:toolchain_version` | string | Rustc version string (e.g. `"rustc 1.86.0"`) |
| `cargo_cicd:cargo_version` | string | Cargo version string (e.g. `"cargo 1.86.0"`) |
| `cargo_cicd:os_version` | string | Operating system version (e.g. `"Ubuntu 22.04"`) |
| `cargo_cicd:session_id` | string | Unique session identifier for this invocation |

### 2.4 Event Element

Each `<event>` represents a single process occurrence within a trace:

```xml
<event>
  <string key="cargo_cicd:event_id" value="{event_id}"/>
  <string key="concept:name" value="{noun}:{verb}"/>
  <date   key="time:timestamp" value="{ISO-8601 UTC}"/>
  <string key="lifecycle:transition" value="start|complete"/>
  <string key="cargo_cicd:verdict_claimed" value="PASS|WARN|FAIL"/>
  <string key="cargo_cicd:workspace_id" value="{workspace_id}"/>
  <string key="cargo_cicd:trace_class" value="live_workspace|pipeline_run"/>
  <!-- Completion-only attributes below -->
  <int    key="cargo_cicd:duration_ms" value="{ms}"/>
</event>
```

**Required `<event>` attributes (all events):**

| Key | Type | Description |
|---|---|---|
| `cargo_cicd:event_id` | string | Globally unique event identifier |
| `concept:name` | string | Activity name in `{noun}:{verb}` form |
| `time:timestamp` | date | ISO-8601 UTC with milliseconds (e.g. `2026-06-17T12:00:00.000Z`) |
| `lifecycle:transition` | string | `"start"` or `"complete"` |
| `cargo_cicd:verdict_claimed` | string | `"PASS"`, `"WARN"`, or `"FAIL"` |
| `cargo_cicd:workspace_id` | string | Workspace identifier (repeated from trace level) |
| `cargo_cicd:trace_class` | string | `"live_workspace"` or `"pipeline_run"` |

**Additional `<event>` attributes (completion events only):**

| Key | Type | Required | Description |
|---|---|---|---|
| `cargo_cicd:duration_ms` | int | No | Wall-clock elapsed milliseconds |
| `wasm4pm:verdict_adjudicated` | string | No | External oracle verdict (`"Accept"`, `"Refuse"`, `"Blocked"`) |
| `wasm4pm:adjudicated_at` | string | No | ISO-8601 timestamp when the oracle responded |
| `wasm4pm:oracle_command` | string | No | Path to the oracle binary that produced the verdict |

---

## 3. Event Lifecycle Transitions

| Transition | When emitted | `verdict_claimed` |
|---|---|---|
| `start` | Immediately before the command begins executing | Empty string |
| `complete` | After the command finishes, before exit | `"PASS"`, `"WARN"`, or `"FAIL"` |

Every command should emit a `start` event followed by a `complete` event. Process mining fitness degrades if only one lifecycle side is present.

**Verdict semantics:**

| Value | Meaning |
|---|---|
| `PASS` | Command completed successfully; all checks satisfied |
| `WARN` | Command completed with warnings; execution continues |
| `FAIL` | Command encountered a blocking error; execution halts |

Special values for dry-run or oracle states:
- `WARN:dry_run` — planning phase, no destructive action taken
- `WARN:oracle_unavailable` — oracle binary not found at adjudication time

---

## 4. Trace Class Values

The `cargo_cicd:trace_class` attribute separates two distinct process streams:

| Value | When used | Token-replay role |
|---|---|---|
| `live_workspace` | Ambient per-command invocations | Accumulated history; VARIANCE expected |
| `pipeline_run` | Full sequential execution via `pipeline run` | Complete declared-process execution |

Token-replay fitness should be evaluated on `pipeline_run` traces only. The `live_workspace` stream is for diagnostic and discovery use.

---

## 5. Case ID Naming Convention

Case IDs group events that belong to the same process instance. Use the following pattern:

```
{workspace_id}_{noun}_{verb}_{ISO8601_date}
```

**Examples:**

| Case ID | Describes |
|---|---|
| `cargo-cicd_status_show_20260617` | Status show command on 2026-06-17 |
| `cargo-cicd_pipeline_run_20260617` | Full pipeline run on 2026-06-17 |
| `my-app_publish_run_20260617` | Publish run for `my-app` workspace |

The `{ISO8601_date}` component should use compact format `YYYYMMDD` without separators.

---

## 6. JSONL Companion Format

Every XES file must have a JSONL companion with the same events in the same order. JSONL (JSON Lines) uses one JSON object per line, enabling streaming parsers.

### 6.1 Required Fields

Each JSONL line must include:

| Field | Type | Description |
|---|---|---|
| `event_id` | string | Same as `cargo_cicd:event_id` in XES |
| `timestamp_iso` | string | ISO-8601 UTC timestamp |
| `lifecycle_transition` | string | `"start"` or `"complete"` |
| `command` | string | Raw command string (e.g. `"status show"`) |
| `verdict_claimed` | string | `"PASS"`, `"WARN"`, or `"FAIL"` |
| `workspace_id` | string | Workspace identifier |
| `repo_path` | string | Path to the repository |
| `trace_class` | string | `"live_workspace"` or `"pipeline_run"` |

### 6.2 Optional Fields

Omit these fields entirely (not `null`) when absent:

| Field | Type | Present when |
|---|---|---|
| `case_id` | string | Event belongs to a named case |
| `duration_ms` | number | Event is a completion event |
| `verdict_adjudicated` | string | Oracle has responded |
| `adjudicated_at` | string | Oracle has responded |
| `oracle_command` | string | Oracle binary was invoked |

### 6.3 File Naming

JSONL companion files follow the same naming convention as XES files:

```
evt-{case_id}-{timestamp}.jsonl
```

---

## 7. Full XML Example (Annotated)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!-- Root element MUST carry xes.version and xmlns:xes -->
<log xes.version="2.0" xmlns:xes="http://www.xes-standard.org/">

  <!-- Declare the four standard extensions -->
  <extension name="Concept"        prefix="concept"   uri="http://www.xes-standard.org/concept.xesext"/>
  <extension name="Time"           prefix="time"       uri="http://www.xes-standard.org/time.xesext"/>
  <extension name="Lifecycle"      prefix="lifecycle"  uri="http://www.xes-standard.org/lifecycle.xesext"/>
  <extension name="Organizational" prefix="org"        uri="http://www.xes-standard.org/org.xesext"/>

  <!-- One <trace> per case (process instance) -->
  <trace>
    <!-- concept:name is the case identifier — REQUIRED by all process mining tools -->
    <string key="concept:name"               value="cargo-cicd_status_show_20260617"/>

    <!-- Workspace context — required for XES 2.0 conformance -->
    <string key="cargo_cicd:workspace_id"    value="cargo-cicd"/>
    <string key="cargo_cicd:workspace_root"  value="/home/user/cargo-cicd"/>
    <string key="cargo_cicd:git_branch"      value="main"/>
    <string key="cargo_cicd:git_commit_sha"  value="abc1234"/>
    <string key="cargo_cicd:toolchain_version" value="rustc 1.86.0 (05f9846f8 2025-01-01)"/>
    <string key="cargo_cicd:cargo_version"   value="cargo 1.86.0 (3f1d47a 2025-01-01)"/>
    <string key="cargo_cicd:os_version"      value="Ubuntu 22.04.4 LTS"/>
    <string key="cargo_cicd:session_id"      value="session-20260617120000000Z"/>

    <!-- Start event: emitted before execution begins -->
    <event>
      <string key="cargo_cicd:event_id"       value="evt-status-show-20260617120000000Z"/>
      <string key="concept:name"              value="status:show"/>
      <date   key="time:timestamp"            value="2026-06-17T12:00:00.000Z"/>
      <string key="lifecycle:transition"      value="start"/>
      <string key="cargo_cicd:verdict_claimed" value=""/>
      <string key="cargo_cicd:workspace_id"  value="cargo-cicd"/>
      <string key="cargo_cicd:trace_class"   value="live_workspace"/>
    </event>

    <!-- Complete event: emitted after execution finishes -->
    <event>
      <string key="cargo_cicd:event_id"       value="evt-status-show-20260617120000042Z"/>
      <string key="concept:name"              value="status:show"/>
      <date   key="time:timestamp"            value="2026-06-17T12:00:00.042Z"/>
      <string key="lifecycle:transition"      value="complete"/>
      <string key="cargo_cicd:verdict_claimed" value="PASS"/>
      <string key="cargo_cicd:workspace_id"  value="cargo-cicd"/>
      <string key="cargo_cicd:trace_class"   value="live_workspace"/>
      <!-- Completion-only: elapsed time -->
      <int    key="cargo_cicd:duration_ms"   value="42"/>
      <!-- Completion-only: oracle adjudication (present only after wasm4pm round-trip) -->
      <!-- <string key="wasm4pm:verdict_adjudicated" value="Accept"/> -->
      <!-- <string key="wasm4pm:adjudicated_at"      value="2026-06-17T12:00:00.100Z"/> -->
      <!-- <string key="wasm4pm:oracle_command"      value="/usr/local/bin/wpm"/> -->
    </event>
  </trace>
</log>
```

---

## 8. Full JSONL Example (Annotated)

```jsonl
{"event_id":"evt-status-show-20260617120000000Z","timestamp_iso":"2026-06-17T12:00:00.000Z","lifecycle_transition":"start","command":"status show","verdict_claimed":"","workspace_id":"cargo-cicd","repo_path":"/home/user/cargo-cicd","trace_class":"live_workspace","case_id":"cargo-cicd_status_show_20260617"}
{"event_id":"evt-status-show-20260617120000042Z","timestamp_iso":"2026-06-17T12:00:00.042Z","lifecycle_transition":"complete","command":"status show","verdict_claimed":"PASS","workspace_id":"cargo-cicd","repo_path":"/home/user/cargo-cicd","trace_class":"live_workspace","case_id":"cargo-cicd_status_show_20260617","duration_ms":42}
```

Notes:
- Each line is a single JSON object terminated by `\n`.
- Optional fields (`case_id`, `duration_ms`, `verdict_adjudicated`, etc.) are omitted when `None` — not serialised as `null`.
- Field ordering within each object is not guaranteed; parsers must use key lookup, not positional access.

---

## 9. Conformance Requirements

### 9.1 For ProM (Process Mining Framework)

- `<log xes.version="2.0">` attribute is required for the XES 2.0 importer plugin.
- `concept:name` must be present on both `<trace>` and `<event>` elements.
- `time:timestamp` must use ISO-8601 UTC format.
- `lifecycle:transition` must use standard values (`start`, `complete`, `suspend`, `resume`, `abort`, `schedule`, `assign`, `reassign`, `withdraw`). For CI/CD use, only `start` and `complete` are required.

### 9.2 For Disco (Fluxicon)

- Requires `concept:name` on events for activity labelling.
- `time:timestamp` drives temporal analysis; missing timestamps exclude events from timeline views.
- Lifecycle transitions are optional in Disco but recommended for start-to-completion analysis.

### 9.3 For Celonis (EMS)

- Requires `time:timestamp` and `concept:name` on every event.
- Case attributes (trace-level strings) map to case table columns in Celonis data models.
- The `cargo_cicd:*` namespace attributes appear as custom columns in the case and event tables.

### 9.4 Token-Replay Fitness

For token-replay conformance checking against a declared process model:

1. **Include only `complete` lifecycle events** — start events duplicate activity names in DFG-derived Petri nets and inflate token counts.
2. **Include only declared-model activities** — noise events (e.g. `git:status`) introduce unmodelled transitions that reduce fitness scores.
3. **Sort events by `time:timestamp` ascending** within each trace — replay assumes temporal ordering.

---

## 10. Version History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-01-01 | Initial XES emission (baseline, minimal attributes) |
| 2.0 | 2026-06-17 | Full XES 2.0 compliance: `xes.version="2.0"`, `xmlns:xes`, workspace metadata in trace, event_name normalisation to `{noun}:{verb}`, JSONL companion format, oracle adjudication fields |
