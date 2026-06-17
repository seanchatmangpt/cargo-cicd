# Process Mining Dashboard Architecture

**Document Type:** Technical Architecture  
**Status:** Proposed (Phase 2)  
**Date:** 2026-06-17  
**Audience:** cargo-cicd core engineers, DevOps platform teams, process mining researchers  
**Companion ADR:** `docs/adr/ADR-011-xes-v2-format.md`

---

## Table of Contents

1. [Data Flow Architecture](#1-data-flow-architecture)
2. [Dashboard Components](#2-dashboard-components)
3. [Process Mining Queries](#3-process-mining-queries)
4. [ProM and Disco Compatibility](#4-prom-and-disco-compatibility)
5. [Phase 2 Implementation Plan](#5-phase-2-implementation-plan)

---

## 1. Data Flow Architecture

### 1.1 End-to-End Data Flow

The process mining pipeline transforms command executions into dashboard visualizations through a multi-stage data flow:

```
┌─────────────────────────────────────────────────────────────────┐
│                         Developer Workstation                    │
│                                                                  │
│  cargo cicd <noun> <verb>                                        │
│         │                                                        │
│         ▼                                                        │
│  target/cargo-cicd/evidence/                                     │
│  ├── evt-status-show-20260617T140000Z.xes    ◄── Canonical      │
│  └── evt-status-show-20260617T140000Z.jsonl  ◄── Streaming      │
│                                                                  │
└──────────────────────────────┬──────────────────────────────────┘
                               │ JSONL stream (file watch / push)
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│                    Evidence Collector Service                     │
│                    (Rust, event-driven)                          │
│                                                                  │
│  - Watches target/cargo-cicd/evidence/ for new *.jsonl files    │
│  - De-duplicates events (by event_id)                           │
│  - Validates schema (required fields present)                    │
│  - Applies workspace-level tags                                  │
│  - Routes to ingestion queue                                     │
└──────────────────────────────┬───────────────────────────────────┘
                               │ Event stream
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Ingestion Queue                              │
│                 (Local: in-memory ring buffer)                   │
│                 (Hosted: Apache Kafka / NATS)                    │
│                                                                  │
│  - Buffers events during collector → store handoff              │
│  - Provides backpressure when store is slow                     │
│  - Enables fan-out (store + real-time WebSocket simultaneously) │
└───────────────┬──────────────────────────────┬───────────────────┘
                │                              │
                ▼                              ▼
┌───────────────────────────┐   ┌──────────────────────────────────┐
│   Process Mining Store    │   │    Real-time WebSocket Feed      │
│  (SQLite local /          │   │  (WebSocket server for live       │
│   PostgreSQL hosted)      │   │   dashboard updates)             │
│                           │   │                                  │
│  Tables:                  │   │  Clients subscribe to:           │
│  - events                 │   │  - workspace events feed         │
│  - traces                 │   │  - verdict stream                │
│  - workspaces             │   │  - policy alert stream           │
│  - verdicts               │   └──────────────────────────────────┘
│  - conformance_results    │
│  - policy_entries         │
└──────────────┬────────────┘
               │
               ▼
┌──────────────────────────────────────────────────────────────────┐
│                      Dashboard API                               │
│                   (REST, read-only, JSON)                        │
│                                                                  │
│  GET /api/v1/workspaces                                         │
│  GET /api/v1/traces?workspace=<id>&from=<iso>&to=<iso>          │
│  GET /api/v1/verdicts/distribution?workspace=<id>               │
│  GET /api/v1/bottlenecks?workspace=<id>                         │
│  GET /api/v1/conformance?workspace=<id>&model=<model_id>        │
│  GET /api/v1/policies/violations?workspace=<id>                 │
│  GET /api/v1/export/xes?workspace=<id>&from=<iso>&to=<iso>      │
└──────────────────────────────┬───────────────────────────────────┘
                               │ HTTP/JSON
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Dashboard Frontend                           │
│               (React + Chart.js / Recharts)                     │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │ Trace Timeline   │  │ Verdict Dist.    │  │ Bottlenecks   │ │
│  │ (Gantt-style)    │  │ (Pie chart)      │  │ (Histogram)   │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │ Policy Heatmap   │  │ Certification    │  │ Conformance   │ │
│  │                  │  │ Status           │  │ Fitness Score │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

### 1.2 Evidence Collector Service

The evidence collector is a lightweight Rust binary (`cargo-cicd-collector`) that runs as a background service:

```rust
// crates/cargo-cicd-collector/src/main.rs

use notify::{Watcher, RecursiveMode};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let evidence_dir = Path::new("target/cargo-cicd/evidence");
    let store = Arc::new(ProcessMiningStore::open("target/cargo-cicd/pm-store.db")?);
    let queue = Arc::new(InMemoryQueue::new(10_000));   // 10K event buffer

    // Watch for new JSONL files
    let mut watcher = notify::recommended_watcher(move |event| {
        if let Ok(notify::Event { kind: EventKind::Create(_), paths, .. }) = event {
            for path in paths {
                if path.extension() == Some(OsStr::new("jsonl")) {
                    tokio::spawn(ingest_jsonl_file(path, queue.clone(), store.clone()));
                }
            }
        }
    })?;

    watcher.watch(evidence_dir, RecursiveMode::NonRecursive)?;

    // Also process existing files on startup
    for entry in std::fs::read_dir(evidence_dir)?.flatten() {
        if entry.path().extension() == Some(OsStr::new("jsonl")) {
            ingest_jsonl_file(entry.path(), queue.clone(), store.clone()).await?;
        }
    }

    // Start REST API server
    let api = DashboardApi::new(store.clone());
    api.serve("127.0.0.1:7878").await?;

    Ok(())
}

async fn ingest_jsonl_file(
    path: PathBuf,
    queue: Arc<InMemoryQueue>,
    store: Arc<ProcessMiningStore>,
) -> anyhow::Result<()> {
    let file = tokio::fs::File::open(&path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let event: ProcessEventJson = serde_json::from_str(&line)?;

        // De-duplicate by event_id
        if !store.event_exists(&event.event_id).await? {
            queue.push(event.clone())?;
            store.insert_event(event).await?;
        }
    }
    Ok(())
}
```

### 1.3 Process Mining Store Schema

The store uses SQLite for local deployments and PostgreSQL for hosted/multi-tenant:

```sql
-- Workspaces table: one row per unique workspace
CREATE TABLE workspaces (
    workspace_id TEXT PRIMARY KEY,  -- "cargo-cicd@/home/user/cargo-cicd"
    display_name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    first_seen_at TIMESTAMP NOT NULL,
    last_seen_at TIMESTAMP NOT NULL,
    event_count INTEGER DEFAULT 0
);

-- Events table: raw process events
CREATE TABLE events (
    event_id TEXT PRIMARY KEY,      -- "evt-status-show-20260617T140000Z-start"
    workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
    case_id TEXT NOT NULL,          -- Groups events into a trace
    command TEXT NOT NULL,          -- "status show"
    lifecycle_transition TEXT NOT NULL,  -- "start" | "complete"
    timestamp_iso TIMESTAMP NOT NULL,
    verdict_claimed TEXT,           -- "PASS" | "WARN" | "FAIL"
    duration_ms INTEGER,            -- Only on "complete" events
    trace_class TEXT,               -- "live_workspace" | "pipeline_run"
    oracle_key_fingerprint TEXT,    -- "pending" | "SHA256:..."
    raw_jsonl TEXT NOT NULL         -- Original JSONL line (for reprocessing)
);

-- Traces table: aggregated case instances
CREATE TABLE traces (
    trace_id TEXT PRIMARY KEY,      -- case_id
    workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
    command TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    duration_ms INTEGER,
    final_verdict TEXT,             -- From "complete" event
    oracle_verdict TEXT,            -- From oracle adjudication
    trace_class TEXT,
    event_count INTEGER DEFAULT 0
);

-- Verdicts table: oracle adjudication results
CREATE TABLE verdicts (
    verdict_id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL REFERENCES traces(trace_id),
    workspace_id TEXT NOT NULL,
    oracle_id TEXT,
    oracle_key_fingerprint TEXT,
    verdict TEXT NOT NULL,          -- "Accept" | "Refuse" | "Blocked"
    adjudicated_at TIMESTAMP NOT NULL,
    receipt_path TEXT
);

-- Conformance results table
CREATE TABLE conformance_results (
    result_id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL REFERENCES traces(trace_id),
    workspace_id TEXT NOT NULL,
    process_model_id TEXT NOT NULL,
    fitness REAL NOT NULL,
    precision REAL NOT NULL,
    violations_json TEXT,           -- JSON array of ConformanceViolation
    checked_at TIMESTAMP NOT NULL
);

-- Policy entries table
CREATE TABLE policy_entries (
    entry_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
    policy_name TEXT NOT NULL,
    verdict TEXT NOT NULL,          -- "Pass" | "Warn" | "Skip"
    recommendation TEXT,
    emitted_at TIMESTAMP NOT NULL
);

-- Indexes for common query patterns
CREATE INDEX events_workspace_timestamp ON events(workspace_id, timestamp_iso);
CREATE INDEX events_case_id ON events(case_id);
CREATE INDEX traces_workspace_command ON traces(workspace_id, command);
CREATE INDEX traces_workspace_started ON traces(workspace_id, started_at);
CREATE INDEX verdicts_workspace_adjudicated ON verdicts(workspace_id, adjudicated_at);
```

---

## 2. Dashboard Components

### 2.1 Workspace Trace Timeline (Gantt-Style)

The trace timeline shows command executions as horizontal bars across time, colored by verdict:

```
Timeline: cargo-cicd workspace (past 24 hours)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Time:  09:00    10:00    11:00    12:00    13:00    14:00    15:00

status show  ██▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒██
             PASS                                        PASS

test changed ████████████████                    ████████████████
             WARN:no-changes                    PASS (12 tests)

publish run                          ████████████████████████████
                                     PASS → Accept (oracle)

pipeline run ████████████████████████████████████████████████████
             PASS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Legend: ██ PASS  ██ WARN  ██ FAIL  ██ Blocked
```

**API**: `GET /api/v1/traces/timeline?workspace=<id>&from=<iso>&to=<iso>`

```json
{
  "workspace_id": "cargo-cicd@/home/user/cargo-cicd",
  "from": "2026-06-17T09:00:00Z",
  "to": "2026-06-17T15:00:00Z",
  "traces": [
    {
      "trace_id": "status_show_phase",
      "command": "status show",
      "started_at": "2026-06-17T09:00:05Z",
      "completed_at": "2026-06-17T09:00:06Z",
      "duration_ms": 1234,
      "final_verdict": "PASS",
      "oracle_verdict": "Accept"
    }
  ]
}
```

**Frontend implementation**: Chart.js Gantt plugin or D3.js timeline. Each bar is clickable to drill into individual events.

### 2.2 Verdict Distribution (Pie Chart)

Shows the distribution of verdicts (claimed and oracle-adjudicated) over a time window:

```
Verdict Distribution (past 7 days)
                                           Claimed    Oracle
  ┌─────────────────────────────────────┐
  │                 ██                  │  PASS       78%       Accept    75%
  │             ████████                │  WARN       18%       Refuse     2%
  │           ██████████████            │  FAIL        4%       Blocked   23%
  │         ████████████████████        │
  │           PASS 78%                  │
  └─────────────────────────────────────┘
```

**API**: `GET /api/v1/verdicts/distribution?workspace=<id>&from=<iso>&to=<iso>`

```json
{
  "workspace_id": "...",
  "period": "7d",
  "claimed": {
    "PASS": 156,
    "WARN": 36,
    "FAIL": 8,
    "total": 200
  },
  "oracle": {
    "Accept": 150,
    "Refuse": 4,
    "Blocked": 46,
    "total": 200
  }
}
```

### 2.3 Bottleneck Detection (Slowest Steps Histogram)

Shows which commands take the longest to execute, helping identify CI/CD bottlenecks:

```
Command Duration Distribution (past 30 days, 95th percentile)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
target prune    ████████████████████████████████████  45.2s
pipeline run    ██████████████████████████████        38.7s
test changed    ████████████████████                  25.3s
trybuild full   ████████████████████                  24.1s
publish run     ██████████████                        18.2s
workspace doc   ████████                              10.4s
git close       ████                                   4.8s
status show     █                                      1.2s
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Note: target prune exceeds 30s SLO in 8% of runs (large target dir)
```

**API**: `GET /api/v1/bottlenecks?workspace=<id>&percentile=95`

```json
{
  "workspace_id": "...",
  "percentile": 95,
  "commands": [
    {
      "command": "target prune",
      "p50_ms": 12000,
      "p95_ms": 45200,
      "p99_ms": 89000,
      "sample_count": 45,
      "slo_ms": 30000,
      "slo_breach_rate": 0.08
    }
  ]
}
```

**Algorithm**: Work-in-progress (WIP) counting using duration histogram. The `hdrhistogram` crate (`src/advanced/histogram.rs`) provides accurate percentile calculations.

### 2.4 Policy Violation Heatmap

Shows which policies fire most frequently, by day of week:

```
Policy Violations Heatmap (past 4 weeks)
                Mon   Tue   Wed   Thu   Fri   Sat   Sun
git_dirty       ████  ███   █     ████  █████  ░     ░
target_pressure ██    █     ███   ██    █      ░     ░
branch_behind   ░     ████  ░     ░     ██     ░     ░
test_stale      ░     ░     ░     ██    ███    ░     ░
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
████ 10+  ███ 5-9  ██ 2-4  █ 1  ░ 0
```

**API**: `GET /api/v1/policies/heatmap?workspace=<id>&weeks=4`

Insights:
- `git_dirty` fires most on Fridays → developers leaving uncommitted work over weekends.
- `branch_behind` fires on Tuesdays → Monday merges from main are not pulled.
- `target_pressure` fires Wednesdays after large compiles.

### 2.5 Certification Status

Shows the current certification status per crate/standard:

```
Certification Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Crate                    SLSA-L3   NIST-218   Oracle
─────────────────────────────────────────────────────
cargo-cicd              ✓ Accept  ✓ Accept   wasm4pm
cargo-cicd-core         ✓ Accept  ✓ Accept   wasm4pm
cargo-cicd-lsp          ⚠ Pending  -          -
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Last checked: 2026-06-17T14:00:00Z
```

**API**: `GET /api/v1/certification/status?workspace=<id>`

---

## 3. Process Mining Queries

### 3.1 Trace Conformance Checking (BESeP Algorithm)

BESeP (Behavior-based conformance checking with State Event log Petri net) checks whether observed traces conform to a declared process model.

The algorithm works by replaying observed traces against a Petri net model:

```rust
// src/conformance/beseep.rs

pub struct BeSePChecker {
    petri_net: PetriNet,  // Compiled from process model
}

pub struct ReplayResult {
    pub trace_id: String,
    pub fitness: f32,        // 0.0 = no fit, 1.0 = perfect fit
    pub missing_tokens: u32, // Tokens that needed to be artificially added
    pub remaining_tokens: u32, // Tokens left over after replay
    pub consumed_tokens: u32,  // Tokens consumed during replay
    pub produced_tokens: u32,  // Tokens produced during replay
}

impl BeSePChecker {
    pub fn replay_trace(&self, trace: &XesTrace) -> ReplayResult {
        let mut state = self.petri_net.initial_marking();
        let mut missing = 0u32;
        let mut consumed = 0u32;
        let mut produced = 0u32;

        for event in &trace.events {
            if event.lifecycle_transition != "complete" { continue; }

            let transition = self.petri_net.find_transition(&event.command);

            match transition {
                None => {
                    // Unseen activity — log as precision issue
                    missing += 1;
                },
                Some(t) => {
                    // Check if transition is enabled
                    while !state.is_enabled(t) {
                        // Need to fire additional transitions to enable t
                        if let Some(recovery) = state.find_shortest_path_to(t) {
                            for rt in recovery {
                                state = state.fire(rt);
                                consumed += rt.input_weight();
                                produced += rt.output_weight();
                                missing += rt.input_weight();  // Artificially enabled
                            }
                        } else {
                            missing += 1;
                            break;
                        }
                    }
                    state = state.fire(t);
                    consumed += t.input_weight();
                    produced += t.output_weight();
                }
            }
        }

        let remaining = state.token_count();

        // Fitness formula (token-based replay)
        let fitness = 0.5 * (1.0 - (missing as f32 / consumed as f32))
                    + 0.5 * (1.0 - (remaining as f32 / produced as f32));

        ReplayResult {
            trace_id: trace.case_id.clone(),
            fitness: fitness.clamp(0.0, 1.0),
            missing_tokens: missing,
            remaining_tokens: remaining,
            consumed_tokens: consumed,
            produced_tokens: produced,
        }
    }
}
```

### 3.2 Bottleneck Detection (Work-in-Progress Counting)

WIP counting measures how many traces are simultaneously in-progress at any time:

```rust
pub struct WipAnalyzer;

impl WipAnalyzer {
    pub fn compute_wip_over_time(
        traces: &[Trace],
        resolution: Duration,  // Time bucket size
    ) -> Vec<(DateTime<Utc>, usize)> {
        let (earliest, latest) = traces.iter()
            .flat_map(|t| [t.started_at, t.completed_at.unwrap_or(Utc::now())])
            .fold((Utc::now(), DateTime::<Utc>::MIN_UTC), |(min, max), ts| {
                (min.min(ts), max.max(ts))
            });

        let mut time = earliest;
        let mut wip_over_time = Vec::new();

        while time <= latest {
            let wip = traces.iter()
                .filter(|t| {
                    t.started_at <= time &&
                    t.completed_at.map_or(true, |c| c >= time)
                })
                .count();
            wip_over_time.push((time, wip));
            time += resolution;
        }

        wip_over_time
    }
}
```

High WIP indicates bottlenecks: many traces are started but few are completing quickly.

### 3.3 Case Variant Analysis

Identifies distinct execution paths (variants) across all traces:

```rust
pub struct VariantAnalyzer;

impl VariantAnalyzer {
    /// Returns all distinct execution paths and their frequencies
    pub fn compute_variants(traces: &[Trace]) -> Vec<Variant> {
        let mut variant_counts: HashMap<Vec<String>, usize> = HashMap::new();

        for trace in traces {
            // Extract the activity sequence (ordered by timestamp)
            let sequence: Vec<String> = trace.events.iter()
                .filter(|e| e.lifecycle_transition == "complete")
                .map(|e| e.command.clone())
                .collect();

            *variant_counts.entry(sequence).or_insert(0) += 1;
        }

        let total = traces.len();
        let mut variants: Vec<Variant> = variant_counts.into_iter()
            .map(|(sequence, count)| Variant {
                sequence,
                count,
                frequency: count as f32 / total as f32,
            })
            .collect();

        variants.sort_by(|a, b| b.count.cmp(&a.count));
        variants
    }
}
```

**Example output**:

| Rank | Variant | Count | Frequency |
|------|---------|-------|-----------|
| 1 | status show → test changed → git close | 45 | 38% |
| 2 | status show → test changed | 31 | 26% |
| 3 | status show | 25 | 21% |
| 4 | pipeline run | 12 | 10% |
| 5 | status show → publish run | 6 | 5% |

The most common variant (38%) shows the "happy path" — status, test, git close.

### 3.4 Fitness Calculation

Overall log fitness measures what fraction of traces conform to the reference process model:

```rust
pub fn calculate_log_fitness(
    conformance_results: &[ReplayResult],
) -> LogFitness {
    let total = conformance_results.len() as f32;
    if total == 0.0 {
        return LogFitness { fitness: 0.0, conformant_traces: 0, total_traces: 0 };
    }

    let conformant = conformance_results.iter()
        .filter(|r| r.fitness >= 0.95)   // Fitness threshold for "conformant"
        .count();

    let avg_fitness: f32 = conformance_results.iter()
        .map(|r| r.fitness)
        .sum::<f32>() / total;

    LogFitness {
        fitness: avg_fitness,
        conformant_traces: conformant,
        total_traces: conformance_results.len(),
    }
}
```

**API**: `GET /api/v1/conformance?workspace=<id>&model=basic-release&from=<iso>`

```json
{
  "workspace_id": "...",
  "process_model": "basic-release/v1.0",
  "period": "7d",
  "fitness": 0.87,
  "conformant_traces": 174,
  "total_traces": 200,
  "top_violations": [
    { "violation": "MissingRequiredActivity:code-review", "count": 18 },
    { "violation": "TemporalViolation:test-run:age_hours=26.5", "count": 8 }
  ]
}
```

---

## 4. ProM and Disco Compatibility

### 4.1 XES Export for ProM Import

ProM (University of Eindhoven) is the leading academic process mining framework. cargo-cicd evidence XES files are directly importable into ProM without conversion.

**Export endpoint**:

```
GET /api/v1/export/xes?workspace=<id>&from=<iso>&to=<iso>
Content-Type: application/xml
Content-Disposition: attachment; filename="cargo-cicd-export.xes"
```

The exported XES file combines multiple traces from the time window into a single XES log:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="2.0" xmlns="http://www.xes-standard.org/">
  <extension name="Lifecycle" prefix="lifecycle"
             uri="http://www.xes-standard.org/lifecycle.xesext"/>
  <extension name="CargoCI" prefix="cargoCI"
             uri="https://cargo-cicd.rs/xes-extensions/v2/cargoCI.xesext"/>

  <string key="cargoCI:workspace_id" value="cargo-cicd@/home/user/cargo-cicd"/>
  <string key="cargoCI:export_from" value="2026-06-10T00:00:00Z"/>
  <string key="cargoCI:export_to" value="2026-06-17T23:59:59Z"/>

  <!-- One trace per command invocation -->
  <trace>
    <string key="concept:name" value="status_show_phase_001"/>
    <!-- ... events ... -->
  </trace>
  <trace>
    <string key="concept:name" value="test_changed_phase_002"/>
    <!-- ... events ... -->
  </trace>
</log>
```

### 4.2 ProM Plugin Integration Plan

ProM has a Java plugin architecture. A cargo-cicd plugin for ProM will:

1. **Read from REST API**: The plugin reads from the cargo-cicd dashboard API rather than requiring local XES files.
2. **Auto-refresh**: ProM's workspace auto-refreshes as new events arrive via the real-time feed.
3. **Custom visualizations**: The plugin provides cargo-cicd-specific views (verdict distribution, oracle status, policy violations) using ProM's visualization framework.

**Plugin architecture**:
```java
// ProM plugin (Java)
@Plugin(name = "cargo-cicd Evidence Log", parameterLabels = {"API URL"}, returnLabels = {"XLog"})
public class CargoCI2ProM {
    public XLog importFromCargoCicdApi(PluginContext context, String apiUrl) {
        var traces = fetchFromApi(apiUrl + "/api/v1/traces?from=7d");
        return convertToXLog(traces);
    }
}
```

Phase 2 milestone: ProM plugin published to ProM Package Manager (PPM).

### 4.3 Disco Compatibility

Disco (Fluxicon) is a commercial process mining tool with excellent XES support. cargo-cicd evidence is compatible with Disco via:

1. **Direct XES import**: Disco's "Import Event Log" dialog accepts cargo-cicd XES files directly.
2. **HTTPS import**: Disco 3.0+ supports importing from URL. The export endpoint serves XES directly.
3. **Recommended settings**:
   - **Case ID attribute**: `concept:name` (the trace name)
   - **Activity attribute**: `concept:name` (the event name, maps to `command`)
   - **Timestamp attribute**: `time:timestamp`
   - **Lifecycle filter**: Enable; use `lifecycle:transition` for start/complete filtering

**Disco import instructions** (to be published in docs/tutorials):
```
1. Open Disco
2. File > Import > Event Log (XES)
3. Select: target/cargo-cicd/evidence/*.xes
   OR enter URL: http://localhost:7878/api/v1/export/xes?workspace=<id>
4. Case ID: concept:name
5. Activity: concept:name
6. Click Import
```

### 4.4 Celonis Compatibility

Celonis (commercial process mining SaaS) can ingest via:

1. **JSONL stream**: Celonis EMS can ingest from a Kafka topic. The cargo-cicd JSONL companion files can be streamed via Kafka.
2. **PostgreSQL direct**: If using the PostgreSQL backend, Celonis can query the `events` and `traces` tables directly.
3. **CSV export**: The dashboard API provides CSV export for Celonis batch import.

**API**: `GET /api/v1/export/csv?workspace=<id>&from=<iso>&to=<iso>`

---

## 5. Phase 2 Implementation Plan

### 5.1 Weeks 1-4: Event Ingestion Service

**Goal**: Build the evidence collector service and process mining store.

**Deliverables**:

- [ ] `crates/cargo-cicd-collector/` — standalone Rust binary.
- [ ] SQLite backend (`cargo-cicd-collector --store sqlite`).
- [ ] JSONL file watcher using `notify` crate.
- [ ] Schema migration using `rusqlite_migration`.
- [ ] `cargo cicd evidence doctor --dashboard` to start collector.
- [ ] De-duplication logic (idempotent ingest).

**Performance targets**:
- Ingest latency: < 100ms per event (file-watch to store insert).
- De-duplication overhead: < 10ms per event (SQLite indexed lookup).
- Concurrent writers: 1 (single writer, multiple readers).

**Test scenarios**:
1. Ingest 1000 JSONL events; verify all stored with correct schema.
2. Re-ingest same 1000 events; verify no duplicates.
3. File watcher detects new file within 500ms.
4. Store survives crash mid-ingest (SQLite WAL mode).
5. Schema migration from v0 (empty) to v1.

### 5.2 Weeks 5-8: Dashboard API

**Goal**: Build the REST API serving dashboard data.

**Endpoints**:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/health` | Health check |
| GET | `/api/v1/workspaces` | List all workspaces |
| GET | `/api/v1/traces` | List traces (paginated, filterable) |
| GET | `/api/v1/traces/{id}` | Single trace with events |
| GET | `/api/v1/traces/timeline` | Gantt-style timeline data |
| GET | `/api/v1/verdicts/distribution` | Verdict pie chart data |
| GET | `/api/v1/bottlenecks` | Duration histogram |
| GET | `/api/v1/conformance` | Conformance fitness |
| GET | `/api/v1/policies/heatmap` | Policy heatmap data |
| GET | `/api/v1/certification/status` | Certification per standard |
| GET | `/api/v1/export/xes` | XES export |
| GET | `/api/v1/export/csv` | CSV export |

**Technology**: Axum (Rust) for the REST server. Read-only API — no writes via REST.

**Performance targets**:
- p95 query latency: < 500ms for any single API call.
- 1000 traces/day per workspace, 90 days retention.
- Max workspaces per instance: 100 (local SQLite) / unlimited (PostgreSQL).

### 5.3 Weeks 9-12: Dashboard Frontend

**Goal**: Build the web frontend for the process mining dashboard.

**Technology**: React + Recharts (or Chart.js). Single-page application served by the collector service at `/`.

**Components to build**:

1. **Trace timeline** (Gantt): `react-gantt-chart` or custom D3.js.
2. **Verdict distribution** (Pie): Recharts `PieChart`.
3. **Bottleneck histogram**: Recharts `BarChart` with percentile annotations.
4. **Policy heatmap**: Custom SVG grid (like GitHub contribution graph).
5. **Certification status**: Table with colored status badges.
6. **Conformance fitness score**: Recharts `RadialBarChart`.

**Database choice summary**:

| Property | SQLite (local) | PostgreSQL (hosted) |
|----------|---------------|---------------------|
| Setup | Zero-config | Requires PostgreSQL server |
| Scale | 1 workspace, 90 days | Unlimited |
| Multi-tenant | No | Yes |
| Performance | 500ms p95 (1000 traces/day) | 100ms p95 (1M traces/day) |
| Use case | Local development | Team/organization dashboard |

**Frontend performance targets**:
- Initial page load: < 2s (webpack bundle < 500KB gzipped).
- Timeline render (1000 traces): < 500ms.
- Real-time update latency (WebSocket): < 1s after event emitted.

### 5.4 Success Metrics

| Metric | Target |
|--------|--------|
| Ingestion throughput | 1000 events/day (local), 100K/day (hosted) |
| API p95 latency | < 500ms |
| Dashboard page load | < 2s |
| Real-time update lag | < 1s |
| ProM/Disco compatibility | 100% XES import success rate |
| Test coverage | > 80% line coverage |
| Binary size (collector) | < 15MB |
| Memory usage (collector) | < 50MB RSS |

---

*Document version 1.0 — 2026-06-17*  
*See also: `docs/adr/ADR-011-xes-v2-format.md`, `docs/PHASE-2-DESIGN.md`*
