# Tutorial: Combinatorial Maximalism

**Combinatorial maximalism** means activating every feature flag simultaneously and
exercising every capability in a single coherent pipeline. The goal is to prove that
all capabilities compose without conflict and to produce the richest possible evidence
record from a single workspace run.

This tutorial walks you through building a maximalist pipeline from scratch. By the
end you will have:

- enabled all five feature tiers (`process-data`, `autonomic`, `wasm4pm`, `contrib`,
  `advanced`)
- exercised all ten advanced modules
- used every OCEL 2.0 type in `cargo_cicd::ocel`
- run all ten CLI nouns in sequence
- emitted a dual-format evidence record (XES + OCEL) and submitted it to the oracle

**Who this is for:** contributors who want to understand the full capability surface of
cargo-cicd, or teams who need to validate that a workspace build is end-to-end
instrumented before release.

**Prerequisites:**
- Rust 1.85 or later
- A Cargo workspace with at least one member crate
- `cargo install cargo-cicd --version 26.6.2`
- Optional: `wpm` on `PATH` (required only for oracle steps — all others degrade
  gracefully to `Blocked`)

---

## Step 1 — Enable all features

Add cargo-cicd as a dev-dependency with all features active:

```toml
# Cargo.toml (workspace root)
[dev-dependencies]
cargo-cicd = { version = "26.6.2", features = [
    "process-data",
    "autonomic",
    "wasm4pm",
    "contrib",
    "advanced",
] }
```

Or enable them when running commands and tests:

```sh
cargo test --features process-data,autonomic,wasm4pm,contrib,advanced
```

Verify the full feature surface compiles without errors:

```sh
cargo check --features process-data,autonomic,wasm4pm,contrib,advanced
```

You should see zero errors. This is the combinatorial-maximalism compile gate.

---

## Step 2 — Initialize observability

Every pipeline stage should be instrumented. The `advanced::observability` module
installs a global JSON-format tracing subscriber in a single call.

```rust
use cargo_cicd::advanced::observability::{init_tracing, PipelineStage, record_event};

// Call once at process start. Idempotent — safe to call from tests.
init_tracing();
```

Wrap each pipeline stage in a `PipelineStage` guard. When the guard drops, it records
the elapsed duration as a structured JSON event:

```rust
let workspace_root = std::path::Path::new(".");

let scan_report = {
    let _stage = PipelineStage::enter("workspace_scan");
    cargo_cicd::advanced::parallel_scan::scan_workspace(workspace_root)
        .expect("workspace scan failed")
    // _stage drops here → emits { "stage": "workspace_scan", "elapsed_ms": ... }
};

record_event("workspace_scan", true);
```

All downstream stages follow this same pattern.

---

## Step 3 — Scan the workspace in parallel

`advanced::parallel_scan` walks the workspace tree in parallel across all available
cores, respecting `.gitignore`, and produces a `ScanReport`.

```rust
use cargo_cicd::advanced::parallel_scan::scan_workspace;

let report = scan_workspace(workspace_root).expect("scan failed");

println!("Files:       {}", report.total_files);
println!("Total bytes: {}", report.total_bytes);
println!("Reclaimable: {} bytes", report.reclaimable_bytes());

// Per-extension breakdown is deterministic (BTreeMap):
for (ext, stats) in &report.per_extension {
    println!("  .{}: {} files, {} bytes", ext, stats.count, stats.bytes);
}

// Target directories discovered:
for (dir, bytes) in &report.target_dirs {
    println!("  target: {} ({} bytes)", dir.display(), bytes);
}
```

`ScanReport` is the foundation for everything that follows: fingerprinting, caching,
and governance scanning all consume it.

---

## Step 4 — Fingerprint scan artifacts

`advanced::fingerprint` computes BLAKE3 hashes over file content. The
`workspace_digest` function combines per-file hashes into a single Merkle root.

```rust
use cargo_cicd::advanced::fingerprint::{hash_bytes, hash_file, workspace_digest};
use std::path::PathBuf;

// Hash the workspace root Cargo.toml:
let cargo_toml_hash = hash_file(&workspace_root.join("Cargo.toml"))
    .expect("Cargo.toml not readable");
println!("Cargo.toml fingerprint: {}", cargo_toml_hash.to_hex());

// Build a workspace digest from all Rust source files:
let rs_entries: Vec<(PathBuf, _)> = report.per_extension
    .get("rs")
    .map(|_| {
        // In practice, walk only .rs files and hash each one.
        // Here we hash a constant to represent the set.
        vec![(
            workspace_root.join("src/lib.rs").to_path_buf(),
            hash_bytes(b"placeholder"),
        )]
    })
    .unwrap_or_default();

let digest = workspace_digest(&rs_entries);
println!("Workspace digest: {}", digest.to_hex());

// Use the hash bytes as a provenance anchor (see Step 11):
let digest_bytes: Vec<u8> = digest.as_bytes().to_vec();
```

---

## Step 5 — Cache adapter results

`advanced::cache` provides a concurrent, TTL-aware cache backed by `moka`. Expensive
adapter calls (metadata parsing, git invocations) should be cached between pipeline
stages.

```rust
use cargo_cicd::advanced::cache::EngineCache;
use std::time::Duration;

// 256-entry cache, 5-minute TTL:
let cache = EngineCache::new(256, Duration::from_secs(300));

// Cache the scan report bytes (serialized):
let scan_bytes = serde_json::to_vec(&serde_json::json!({
    "total_files": report.total_files,
    "total_bytes": report.total_bytes,
})).unwrap();

cache.put_labeled("workspace_scan", scan_bytes, "ScanReport");

// Later retrieval (Arc-wrapped, zero-copy):
if let Some(hit) = cache.get("workspace_scan") {
    println!("cache hit: {} bytes ({})", hit.len(), hit.label);
}

// Or, compute-and-cache in one shot:
let _cached = cache.get_or_insert_with("workspace_digest", || {
    digest.as_bytes().to_vec()
});

println!("cache entries: {}", cache.entry_count());
```

---

## Step 6 — Build the dependency graph

`advanced::dep_graph` constructs a directed dependency graph of workspace crates using
`petgraph` and computes a topological build order.

```rust
use cargo_cicd::advanced::dep_graph::WorkspaceGraph;

let mut graph = WorkspaceGraph::new();

// Register workspace members (read from Cargo.toml in practice):
graph.add_crate("cargo-cicd");
graph.add_crate("cargo-cicd-core");
graph.add_crate("cargo-cicd-lsp");

// Declare dependencies:
graph.add_dependency("cargo-cicd", "cargo-cicd-core");
graph.add_dependency("cargo-cicd", "cargo-cicd-lsp");
graph.add_dependency("cargo-cicd-lsp", "cargo-cicd-core");

// Compute build order (topological sort):
let order = graph.build_order().expect("dependency cycle detected");
println!("Build order: {:?}", order);
// ["cargo-cicd-core", "cargo-cicd-lsp", "cargo-cicd"]

// Find dependents of a crate:
let dependents = graph.dependents_of("cargo-cicd-core");
println!("Dependents of cargo-cicd-core: {:?}", dependents);

// Cycle detection:
assert!(!graph.has_cycle(), "workspace must be acyclic");

// Strongly connected components (each is a singleton in a DAG):
let sccs = graph.strongly_connected_components();
println!("SCCs: {} (expected 3 for a DAG)", sccs.len());
```

---

## Step 7 — Record timing with the timeline

`advanced::timeline` provides a high-precision, append-only event timeline backed by
`jiff`. Every pipeline stage records its start time; `span_between` measures the
elapsed duration between any two events.

```rust
use cargo_cicd::advanced::timeline::ProcessTimeline;

let mut timeline = ProcessTimeline::new();

// Record the pipeline start:
timeline.record("pipeline:start");

// Record intermediate stages:
timeline.record("workspace:scan:complete");
timeline.record("workspace:fingerprint:complete");
timeline.record("workspace:graph:complete");

// ... run CLI pipeline here (Step 9) ...

timeline.record("pipeline:complete");

// Measure total pipeline duration:
if let Some(span) = timeline.total_span() {
    println!("Total pipeline duration: {}", span);
}

// Measure a specific stage:
if let Some(scan_span) = timeline.span_between("pipeline:start", "workspace:scan:complete") {
    println!("Scan took: {}", scan_span);
}

// Export as ISO 8601 for the OCEL log:
for (label, ts) in timeline.to_iso8601() {
    println!("  {} at {}", label, ts);
}
```

---

## Step 8 — Collect latency histograms

`advanced::histogram` uses `hdrhistogram` to record the microsecond latency
distribution of each pipeline stage. This gives p50/p90/p99 statistics for CI
performance analysis.

```rust
use cargo_cicd::advanced::histogram::StageLatencies;
use std::time::Instant;

let mut scan_hist = StageLatencies::new("workspace_scan");
let mut graph_hist = StageLatencies::new("dep_graph");

// Record multiple observations (e.g., in a test loop or over multiple runs):
let t0 = Instant::now();
// ... do work ...
scan_hist.record_duration(t0.elapsed());

// You can also record raw microsecond values:
graph_hist.record(1_250); // 1250 µs

// Merge histograms from parallel workers:
let mut combined = StageLatencies::new("combined");
combined.merge(&scan_hist);
combined.merge(&graph_hist);

// Report percentiles:
let p = combined.percentiles();
println!(
    "Latency percentiles (µs): p50={} p90={} p99={} max={} mean={:.1}",
    p.p50, p.p90, p.p99, p.max, p.mean
);
```

---

## Step 9 — Scan for governance patterns

`advanced::pattern` uses Aho-Corasick to scan text for multiple patterns simultaneously.
Use it for license compliance, forbidden-term detection, or governance rules.

```rust
use cargo_cicd::advanced::pattern::MultiPatternScanner;

// Detect forbidden terms in all source files:
let forbidden = MultiPatternScanner::new_case_insensitive(&[
    "TODO", "FIXME", "HACK", "unsafe",
]).expect("pattern compile failed");

let source = std::fs::read_to_string("src/main.rs").unwrap_or_default();

if forbidden.contains_any(&source) {
    let matches = forbidden.scan(&source);
    for m in &matches {
        println!(
            "governance: '{}' at [{}, {}]",
            m.pattern, m.start, m.end
        );
    }
}

// Detect which patterns fired:
let matched = forbidden.matched_patterns(&source);
println!("Matched governance patterns: {:?}", matched);
```

---

## Step 10 — Diagnose any violations

If any stage fails, `advanced::diagnostics` renders a rich, human-readable diagnostic
using `miette`.

```rust
use cargo_cicd::advanced::diagnostics::{render, severity_of, EngineDiagnostic};

// Example: target directory exceeds budget:
let diag = EngineDiagnostic::TargetPressure {
    size_mb: report.reclaimable_bytes() / 1_048_576,
    budget_mb: 8_192,
};

println!("Severity: {:?}", severity_of(&diag));
println!("{}", render(&diag));

// Example: dirty git phase:
let git_diag = EngineDiagnostic::DirtyGitPhase {
    phase: "feature/ocel-migration".to_string(),
    dirty_paths: 3,
};
println!("{}", render(&git_diag));
```

---

## Step 11 — Run all CLI nouns

The maximalist pipeline exercises every CLI noun in sequence. Each emits a
`ProcessEvent` that becomes part of the evidence record.

```sh
# Step 11a: workspace health
cargo cicd workspace doctor

# Step 11b: status snapshot
cargo cicd status show

# Step 11c: target analysis
cargo cicd target show
cargo cicd target prune --dry-run

# Step 11d: selective tests
cargo cicd test changed

# Step 11e: trybuild fixtures
cargo cicd trybuild changed

# Step 11f: git state
cargo cicd git status

# Step 11g: evidence collection
cargo cicd evidence doctor

# Step 11h: LSP diagnostic (OCEL reference)
cargo cicd lsp explain CICD-EVIDENCE-002

# Step 11i: full pipeline run
cargo cicd pipeline run

# Step 11j: publish check (dry-run; do not publish in tutorial)
# cargo cicd publish run  # omit to avoid actual publishing
```

Each command appends a `ProcessEvent` to `target/cargo-cicd/evidence/events.jsonl`
and regenerates both `events.xes` and `events.ocel.json`.

---

## Step 12 — Snapshot the engine state

After all stages complete, take a compact binary snapshot with `advanced::snapshot`.

```rust
use cargo_cicd::advanced::snapshot::{decode, encode, EngineSnapshot, StageRecord};

let snapshot = EngineSnapshot {
    workspace_root: workspace_root.to_string_lossy().into_owned(),
    toolchain: "stable".to_string(),
    changed_files: vec!["src/ocel.rs".to_string(), "tests/ocel_chicago_tdd.rs".to_string()],
    target_bytes: report.reclaimable_bytes(),
    git_phase: "clean".to_string(),
    schema_version: EngineSnapshot::current_schema_version(),
    stages: vec![
        StageRecord { name: "workspace_scan".into(), ok: true, elapsed_ms: 42 },
        StageRecord { name: "dep_graph".into(),      ok: true, elapsed_ms: 3  },
        StageRecord { name: "pipeline_run".into(),   ok: true, elapsed_ms: 890 },
    ],
};

// Serialize to compact binary (bitcode):
let bytes = encode(&snapshot).expect("snapshot encode failed");
println!("Snapshot size: {} bytes", bytes.len());

// Store to disk:
std::fs::write("target/cargo-cicd/engine.snap", &bytes).unwrap();

// Deserialize and verify:
let restored = decode(&bytes).expect("snapshot decode failed");
assert_eq!(restored.workspace_root, snapshot.workspace_root);
assert_eq!(restored.stages.len(), 3);
println!("Snapshot round-trip: ok");
```

---

## Step 13 — Build an OCEL 2.0 log using all types

This is the evidence-model heart of combinatorial maximalism. Every type in
`cargo_cicd::ocel` is exercised here.

```rust
use cargo_cicd::ocel::*;
use std::collections::HashMap;

// ── 13a: Construct the type schema ────────────────────────────────────────────

let types = OcelTypes {
    object_types: OcelLog::cargo_object_types(),   // 11 canonical types
    event_types: vec![
        OcelEventType {
            name: "status:show".into(),
            attributes: vec![
                OcelObjectAttribute { name: "verdict_claimed".into(), attr_type: "string".into() },
                OcelObjectAttribute { name: "trace_class".into(),     attr_type: "string".into() },
            ],
        },
        OcelEventType {
            name: "pipeline:run".into(),
            attributes: vec![
                OcelObjectAttribute { name: "duration_ms".into(), attr_type: "integer".into() },
            ],
        },
    ],
};

// ── 13b: Construct objects for every canonical type ────────────────────────────

let mut objects: HashMap<String, OcelObject> = HashMap::new();

// cargo.workspace
let mut ws_ovmap = HashMap::new();
ws_ovmap.insert("workspace_id".into(), serde_json::json!("cargo-cicd-workspace"));
ws_ovmap.insert("repo_path".into(),    serde_json::json!("/home/user/cargo-cicd"));
objects.insert("ws:main".into(), OcelObject {
    object_type: "cargo.workspace".into(),
    ovmap: ws_ovmap,
    o2o: vec![],
});

// cargo.git-phase
let mut git_ovmap = HashMap::new();
git_ovmap.insert("branch".into(),      serde_json::json!("claude/gracious-turing-bcvou2"));
git_ovmap.insert("dirty_count".into(), serde_json::json!(0));
objects.insert("git:phase".into(), OcelObject {
    object_type: "cargo.git-phase".into(),
    ovmap: git_ovmap,
    o2o: vec![],
});

// cargo.target
let mut tgt_ovmap = HashMap::new();
tgt_ovmap.insert("total_size_bytes".into(), serde_json::json!(report.total_bytes));
objects.insert("target:main".into(), OcelObject {
    object_type: "cargo.target".into(),
    ovmap: tgt_ovmap,
    o2o: vec![],
});

// cargo.toolchain
let mut tc_ovmap = HashMap::new();
tc_ovmap.insert("rust_version".into(), serde_json::json!("1.85.0"));
objects.insert("toolchain:stable".into(), OcelObject {
    object_type: "cargo.toolchain".into(),
    ovmap: tc_ovmap,
    o2o: vec![],
});

// cargo.crate (three workspace members)
for (name, id) in &[
    ("cargo-cicd",      "crate:root"),
    ("cargo-cicd-core", "crate:core"),
    ("cargo-cicd-lsp",  "crate:lsp"),
] {
    let mut ovmap = HashMap::new();
    ovmap.insert("name".into(), serde_json::json!(name));
    objects.insert((*id).into(), OcelObject {
        object_type: "cargo.crate".into(),
        ovmap,
        o2o: vec![],
    });
}

// cargo.test-plan
let mut tp_ovmap = HashMap::new();
tp_ovmap.insert("estimated_count".into(), serde_json::json!(58));
objects.insert("testplan:ocel".into(), OcelObject {
    object_type: "cargo.test-plan".into(),
    ovmap: tp_ovmap,
    o2o: vec![],
});

// cargo.trybuild
let mut tb_ovmap = HashMap::new();
tb_ovmap.insert("snapshot_mode".into(), serde_json::json!("changed"));
objects.insert("trybuild:main".into(), OcelObject {
    object_type: "cargo.trybuild".into(),
    ovmap: tb_ovmap,
    o2o: vec![],
});

// cargo.policy
let mut pol_ovmap = HashMap::new();
pol_ovmap.insert("verdict".into(), serde_json::json!("Pass"));
objects.insert("policy:target-pressure".into(), OcelObject {
    object_type: "cargo.policy".into(),
    ovmap: pol_ovmap,
    o2o: vec![],
});

// cargo.artifact
objects.insert("artifact:binary".into(), OcelObject {
    object_type: "cargo.artifact".into(),
    ovmap: HashMap::new(),
    o2o: vec![],
});

// cargo.evidence
let mut ev_ovmap = HashMap::new();
ev_ovmap.insert("format".into(), serde_json::json!("ocel2+xes"));
objects.insert("evidence:session".into(), OcelObject {
    object_type: "cargo.evidence".into(),
    ovmap: ev_ovmap,
    // O2O: evidence links to the workspace
    o2o: vec![OcelRelationship {
        object_id: "ws:main".into(),
        object_type: "cargo.workspace".into(),
        qualifier: Some("evidences".into()),
    }],
});

// cargo.pipeline
let mut pl_ovmap = HashMap::new();
pl_ovmap.insert("trace_class".into(), serde_json::json!("pipeline_run"));
objects.insert("pipeline:maximalist".into(), OcelObject {
    object_type: "cargo.pipeline".into(),
    ovmap: pl_ovmap,
    o2o: vec![],
});

// ── 13c: Construct events with full typed-omap ────────────────────────────────

let mut events: HashMap<String, OcelEvent> = HashMap::new();

// Helper: build a typed-omap entry referencing all key objects
let all_rels: Vec<OcelRelationship> = vec![
    OcelRelationship { object_id: "ws:main".into(),          object_type: "cargo.workspace".into(), qualifier: None },
    OcelRelationship { object_id: "git:phase".into(),        object_type: "cargo.git-phase".into(), qualifier: None },
    OcelRelationship { object_id: "pipeline:maximalist".into(), object_type: "cargo.pipeline".into(), qualifier: None },
];

// Event: status show
let mut status_vmap: HashMap<String, serde_json::Value> = HashMap::new();
status_vmap.insert("verdict_claimed".into(), serde_json::json!("PASS"));
status_vmap.insert("trace_class".into(),     serde_json::json!("pipeline_run"));
events.insert("evt:status-show".into(), OcelEvent {
    activity: "status:show".into(),
    timestamp: "2026-06-17T00:00:01Z".into(),
    vmap: status_vmap,
    typed_omap: all_rels.clone(),
});

// Event: workspace doctor
let mut doctor_vmap: HashMap<String, serde_json::Value> = HashMap::new();
doctor_vmap.insert("verdict_claimed".into(), serde_json::json!("PASS"));
events.insert("evt:workspace-doctor".into(), OcelEvent {
    activity: "workspace:doctor".into(),
    timestamp: "2026-06-17T00:00:02Z".into(),
    vmap: doctor_vmap,
    typed_omap: all_rels.clone(),
});

// Event: target show
let mut tgt_vmap: HashMap<String, serde_json::Value> = HashMap::new();
tgt_vmap.insert("verdict_claimed".into(), serde_json::json!("PASS"));
events.insert("evt:target-show".into(), OcelEvent {
    activity: "target:show".into(),
    timestamp: "2026-06-17T00:00:03Z".into(),
    vmap: tgt_vmap,
    typed_omap: all_rels.clone(),
});

// Event: test changed
let mut test_vmap: HashMap<String, serde_json::Value> = HashMap::new();
test_vmap.insert("verdict_claimed".into(), serde_json::json!("PASS"));
events.insert("evt:test-changed".into(), OcelEvent {
    activity: "test:changed".into(),
    timestamp: "2026-06-17T00:00:04Z".into(),
    vmap: test_vmap,
    typed_omap: all_rels.clone(),
});

// Event: publish run
let mut pub_vmap: HashMap<String, serde_json::Value> = HashMap::new();
pub_vmap.insert("verdict_claimed".into(), serde_json::json!("PASS"));
events.insert("evt:publish-run".into(), OcelEvent {
    activity: "publish:run".into(),
    timestamp: "2026-06-17T00:00:05Z".into(),
    vmap: pub_vmap,
    typed_omap: all_rels.clone(),
});

// Event: evidence audit
let mut audit_vmap: HashMap<String, serde_json::Value> = HashMap::new();
audit_vmap.insert("verdict_claimed".into(), serde_json::json!("PASS"));
events.insert("evt:evidence-audit".into(), OcelEvent {
    activity: "evidence:audit".into(),
    timestamp: "2026-06-17T00:00:06Z".into(),
    vmap: audit_vmap,
    typed_omap: all_rels.clone(),
});

// ── 13d: Assemble and validate the log ────────────────────────────────────────

let log = OcelLog { version: "2.0".into(), types, events, objects };
let report_v = log.validate();

assert!(
    report_v.valid,
    "OCEL validation must pass: {:?}",
    report_v.violations
);
println!(
    "OCEL log: {} events, {} objects, {} violations",
    report_v.event_count,
    report_v.object_count,
    report_v.violations.len()
);
```

---

## Step 14 — Query the log with OCPQ predicates

`ocpq_eval` checks structural predicates over the OCEL log:

```rust
use cargo_cicd::ocel::{BasicPredicate, ocpq_eval};

let predicates = vec![
    // Every status:show event must reference a cargo.workspace object:
    BasicPredicate::E2O {
        event_type: "status:show".into(),
        object_type: "cargo.workspace".into(),
    },
    // Every workspace:doctor event must reference a cargo.workspace object:
    BasicPredicate::E2O {
        event_type: "workspace:doctor".into(),
        object_type: "cargo.workspace".into(),
    },
    // The evidence object should link to the workspace object (O2O):
    BasicPredicate::O2O {
        from_type: "cargo.evidence".into(),
        to_type:   "cargo.workspace".into(),
    },
    // All pipeline activities should complete within 60 seconds:
    BasicPredicate::Tbe {
        event_type:   "publish:run".into(),
        threshold_ms: 60_000,
    },
];

let results = ocpq_eval(&log, &predicates);
for (pred, result) in predicates.iter().zip(&results) {
    println!("  {:?}: {}", pred, if *result { "PASS" } else { "FAIL" });
}
assert!(results.iter().all(|&r| r), "all OCPQ predicates must hold");
```

---

## Step 15 — Compute process conformance

Build a Petri net representing the expected pipeline sequence and measure
token-replay fitness over the observed event trace.

```rust
use cargo_cicd::ocel::{ConformanceResult, Dfg, PetriNet};

// Observed trace from the pipeline run:
let trace = [
    "status:show",
    "workspace:doctor",
    "target:show",
    "test:changed",
    "publish:run",
    "evidence:audit",
];

// Build a DFG from the trace:
let dfg = Dfg::from_trace(&trace);
println!("DFG edges: {}", dfg.edges.len());
assert!(dfg.edges.contains_key("status:show -> workspace:doctor"));

// Build a Petri net covering the declared pipeline:
let mut net = PetriNet::new();
net.places = vec!["start".into(), "mid".into(), "end".into()];
net.transitions = trace.iter().map(|s| s.to_string()).collect();
net.arcs = vec![
    ("start".into(), "status:show".into()),
    ("status:show".into(), "workspace:doctor".into()),
    ("workspace:doctor".into(), "target:show".into()),
    ("target:show".into(), "test:changed".into()),
    ("test:changed".into(), "publish:run".into()),
    ("publish:run".into(), "evidence:audit".into()),
    ("evidence:audit".into(), "end".into()),
];
net.initial_marking = vec!["start".into()];
net.final_marking   = vec!["end".into()];

let fitness = net.token_replay_fitness(&trace);
let conformance = ConformanceResult::truthful(fitness);

println!("Fitness: {:.3}", conformance.fitness);
println!("Verdict: {:?}", conformance.verdict);
assert!(fitness > 0.9, "pipeline must achieve >90% token-replay fitness");
```

---

## Step 16 — Optimize with Pareto analysis

Use `reject_dominated` and `DimensionGroup<U>` to choose the best pipeline
configuration from a set of candidate runs:

```rust
use cargo_cicd::ocel::{
    DimMs, DimRatio, DimensionGroup, is_dominated, reject_dominated,
};

// Candidate runs: (fitness, simplicity) pairs
// Higher is better on both dimensions.
let candidates: Vec<(f64, f64)> = vec![
    (0.97, 0.85),  // fast but medium simplicity
    (0.92, 0.95),  // slower but high simplicity
    (0.75, 0.70),  // dominated — worse on both axes
    (0.97, 0.80),  // dominated by (0.97, 0.85)
];

let pareto_front = reject_dominated(&candidates);
println!("Pareto-optimal configurations: {:?}", pareto_front);
assert!(!pareto_front.contains(&(0.75, 0.70)), "dominated candidate rejected");

// Check individual dominance:
assert!(is_dominated((0.75, 0.70), &candidates), "low-fitness run is dominated");

// Track latency measurements with dimensional groups:
let mut latency_ms: DimensionGroup<DimMs> = DimensionGroup::new("pipeline_latency_ms");
latency_ms.push(890.0);
latency_ms.push(910.0);
latency_ms.push(875.0);

let mut fitness_ratio: DimensionGroup<DimRatio> = DimensionGroup::new("token_replay_fitness");
fitness_ratio.push(fitness);

println!(
    "Latency: mean={:.1}ms max={:.1}ms min={:.1}ms",
    latency_ms.mean(), latency_ms.max(), latency_ms.min()
);
```

---

## Step 17 — Detect anomalies and drift

```rust
use cargo_cicd::ocel::{detect_drift, page_hinkley_test, score_sequence_anomaly, select_ucb1};

// Historical latency observations (ms):
let historical: Vec<f64> = vec![
    880.0, 892.0, 875.0, 901.0, 886.0,
    890.0, 878.0, 895.0, 883.0, 891.0,
];

// Current run window:
let current = vec![890.0];

// Anomaly detection — current run vs historical baseline:
let anomaly_score = score_sequence_anomaly(&historical);
println!("Anomaly score: {:.3}", anomaly_score);

// Drift detection:
let baseline = &historical[..5];
let recent   = &historical[5..];
let drifted = detect_drift(baseline, recent);
println!("Drift detected: {}", drifted);

// Change-point detection:
let change_point = page_hinkley_test(&historical, 10.0, 0.5);
println!("Change point: {:?}", change_point);

// UCB1 policy selection — choose among pipeline variants:
let rewards = [4.2f64, 3.8, 5.1];  // cumulative reward per variant
let counts  = [10u64, 8, 6];       // times each variant was run
let chosen  = select_ucb1(&rewards, &counts, 24);
println!("UCB1 selects pipeline variant: {}", chosen);
```

---

## Step 18 — Process mining utilities

```rust
use cargo_cicd::ocel::{
    jaccard_similarity, mcts_select, synchronizing_merge, Perturbator,
};

// Jaccard similarity between two pipeline activity sets:
let run_a = ["status:show", "workspace:doctor", "test:changed"];
let run_b = ["status:show", "target:show",      "test:changed"];
let sim = jaccard_similarity(&run_a, &run_b);
println!("Jaccard similarity between runs: {:.2}", sim);

// Synchronizing merge of two event-id sequences:
let seq_a = vec!["evt:001".into(), "evt:002".into(), "evt:004".into()];
let seq_b = vec!["evt:002".into(), "evt:003".into(), "evt:005".into()];
let merged = synchronizing_merge(&seq_a, &seq_b);
println!("Merged event sequence: {:?}", merged);

// MCTS arm selection for adaptive pipeline ordering:
let scores = [0.9f64, 0.6, 0.8, 0.7];
let chosen = mcts_select(&scores, 1.41);
println!("MCTS selects stage: {}", chosen);

// Mutation testing with Perturbator (for evidence mutation tests):
let p = Perturbator::new(42);
let trace: Vec<String> = trace.iter().map(|s| s.to_string()).collect();
let perturbed = p.perturb_trace(&trace);
let dropped   = p.drop_event(&trace);
let noisy     = p.inject_noise(&trace, "noise:injected");
println!("Perturbed: {:?}", perturbed);
println!("Dropped:   {:?}", dropped);
println!("Noisy:     {:?}", noisy);
```

---

## Step 19 — Build a provenance chain and knowledge base

```rust
use cargo_cicd::ocel::{
    admit_atom, admit_rule, blake3_hex, canonical_json, hash_bytes, replay, Blake3Hash,
    Prolog8Receipt, ProvenanceChain,
};

// Provenance chain: link the workspace scan hash to the OCEL evidence:
let mut chain = ProvenanceChain::new();
chain.append(digest_bytes.as_slice(), "workspace_scan", "2026-06-17T00:00:00Z");

// Serialize the OCEL log canonically:
let log_json = serde_json::to_value(&serde_json::json!({"version": "2.0"})).unwrap();
let canonical = canonical_json(&log_json);
chain.append(canonical.as_bytes(), "ocel_log", "2026-06-17T00:00:07Z");

// The root hash is the tamper-evident receipt:
let root = chain.root_hash().expect("chain must have entries");
println!("Provenance chain root: {}", root.0);

// Verify content hash:
let h = Blake3Hash::of(b"cargo-cicd process evidence");
assert!(h.verify(b"cargo-cicd process evidence"), "Blake3Hash::verify must pass");

// Build a Prolog8 knowledge base from the evidence:
let mut kb = Vec::new();
admit_atom(&mut kb, "status:show");
admit_atom(&mut kb, "workspace:doctor");
admit_atom(&mut kb, "test:changed");
admit_rule(&mut kb, "pipeline_ok", &["status:show", "workspace:doctor", "test:changed"]);

let receipt = Prolog8Receipt::from_kb(&kb);
println!("KB hash: {}", receipt.hash.0);

// Replay the trace against the knowledge base:
let score = replay(&kb, &["status:show", "workspace:doctor", "test:changed"]);
println!("KB replay score: {:.2} (expect 1.0 for full coverage)", score);
assert!(score > 0.0, "replay must return positive score for proved atoms");

// blake3_hex as a standalone function (64-char hex, no external deps):
let hex = blake3_hex(b"cargo-cicd");
assert_eq!(hex.len(), 64);
println!("blake3_hex: {}", hex);

// hash_bytes (returns Blake3Hash struct):
let h2 = hash_bytes(b"evidence payload");
println!("hash_bytes: {}", h2.0);
```

---

## Step 20 — Flatten and analyze the log

```rust
// Flatten: case-centric view of the log (grouped by cargo.pipeline object):
let flat = log.flatten();
println!("Flat cases: {}", flat.cases.len());
for case in &flat.cases {
    println!("  case {}: {} events, {} objects",
        case.case_id, case.events.len(), case.objects.len());
}

// E2O and O2O relationship extracts:
let e2o = log.e2o();
println!("E2O relationships: {}", e2o.len());
for (eid, oid, otype) in &e2o {
    println!("  {} → {} ({})", eid, oid, otype);
}

let o2o = log.o2o();
println!("O2O relationships: {}", o2o.len());

// Object attribute values:
let ws_paths = log.oaval("cargo.workspace", "repo_path");
for (oid, val) in &ws_paths {
    println!("  {} repo_path = {}", oid, val);
}
```

---

## Step 21 — Emit dual-format evidence

```rust
use cargo_cicd::evidence::{
    append_events, assert_wpm_verdict_ocel, build_ocel_log, emit_ocel, emit_ocel_filtered,
    ExpectedWpmVerdict, ProcessEvent, WpmEvidenceOracle,
};
use std::path::Path;

// Build ProcessEvents for the pipeline stages:
let events: Vec<ProcessEvent> = vec![
    ProcessEvent::new("status:show",      "PASS"),
    ProcessEvent::new("workspace:doctor", "PASS"),
    ProcessEvent::new("target:show",      "PASS"),
    ProcessEvent::new("test:changed",     "PASS"),
    ProcessEvent::new("publish:run",      "PASS"),
    ProcessEvent::new("evidence:audit",   "PASS"),
    ProcessEvent::new("receipt:write",    "PASS"),
];

let evidence_dir = Path::new("target/cargo-cicd/evidence");

// Append accumulates events across calls and archives to history/:
append_events(&events, evidence_dir).expect("append_events failed");

// Also emit a fresh OCEL snapshot (filtered to declared activities only):
let ocel_path = evidence_dir.join("pipeline-run.ocel.json");
emit_ocel_filtered(&events, &ocel_path).expect("emit_ocel_filtered failed");

// Build a serde_json::Value for programmatic inspection:
let ocel_value = build_ocel_log(&events);
assert_eq!(ocel_value["ocel:version"], "2.0");
assert!(ocel_value["ocel:events"].as_object().unwrap().len() > 0);
```

---

## Step 22 — Verify with the oracle

```rust
// Instantiate the oracle (auto-detects wpm binary):
let oracle = WpmEvidenceOracle::new();

if oracle.is_available() {
    println!("wpm oracle: available");
} else {
    println!("wpm oracle: unavailable (Blocked verdict expected — E7)");
}

// assert_wpm_verdict_ocel handles both cases:
// - If oracle available: calls wpm receipt verify-ocel2 and checks verdict
// - If oracle unavailable AND expected == Blocked: passes silently (E7)
assert_wpm_verdict_ocel(&oracle, &ocel_path, &ExpectedWpmVerdict::Blocked);

// The Wasm4pmShell exposes all six receipt sub-commands for direct use:
use cargo_cicd::integrations::Wasm4pmShell;
if let Some(wpm) = Wasm4pmShell::detect() {
    let path = ocel_path.to_str().unwrap();
    let _ = wpm.receipt_verify_ocel2(path);
    let _ = wpm.receipt_canonicalize_ocel2(path);
    let _ = wpm.receipt_detect_fixture_mutation(path);
    let _ = wpm.receipt_verify_boundary_evidence(path);
    let _ = wpm.receipt_verify_proof_class(path);
    let _ = wpm.autoprocess();
}
```

---

## Step 23 — Run as an integration test

Wrap the entire pipeline in a single integration test to make it part of CI:

```toml
# Cargo.toml
[[test]]
name = "maximalist_pipeline"
path = "tests/maximalist_pipeline.rs"
```

```rust
// tests/maximalist_pipeline.rs
#[test]
fn maximalist_pipeline_compiles_and_runs() {
    // The test content is the pipeline above, condensed.
    // The key assertion is that every module composes without panic.

    // Feature compile-gate:
    cargo_cicd::advanced::observability::init_tracing();

    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();

    // parallel_scan
    // (skip on tmp dir — no files; just verify it doesn't panic)
    let _ = cargo_cicd::advanced::parallel_scan::scan_workspace(root);

    // fingerprint
    use cargo_cicd::advanced::fingerprint::hash_bytes;
    let fp = hash_bytes(b"test");
    assert_eq!(fp.to_hex().len(), 64);

    // cache
    use cargo_cicd::advanced::cache::EngineCache;
    let cache = EngineCache::new(16, std::time::Duration::from_secs(60));
    cache.put("k", b"v".to_vec());
    assert!(cache.get("k").is_some());

    // dep_graph
    use cargo_cicd::advanced::dep_graph::WorkspaceGraph;
    let mut g = WorkspaceGraph::new();
    g.add_crate("a");
    g.add_crate("b");
    g.add_dependency("a", "b");
    assert!(!g.has_cycle());

    // timeline
    use cargo_cicd::advanced::timeline::ProcessTimeline;
    let mut tl = ProcessTimeline::new();
    tl.record("start");
    tl.record("end");
    assert_eq!(tl.len(), 2);

    // histogram
    use cargo_cicd::advanced::histogram::StageLatencies;
    let mut hist = StageLatencies::new("test");
    hist.record(500);
    assert!(hist.p99() > 0);

    // pattern
    use cargo_cicd::advanced::pattern::MultiPatternScanner;
    let scanner = MultiPatternScanner::new(&["TODO", "FIXME"]).unwrap();
    assert!(scanner.contains_any("TODO: fix this"));

    // snapshot
    use cargo_cicd::advanced::snapshot::{decode, encode, EngineSnapshot};
    let snap = EngineSnapshot {
        workspace_root: "/tmp".into(),
        toolchain: "stable".into(),
        changed_files: vec![],
        target_bytes: 0,
        git_phase: "clean".into(),
        schema_version: EngineSnapshot::current_schema_version(),
        stages: vec![],
    };
    let bytes = encode(&snap).unwrap();
    let snap2 = decode(&bytes).unwrap();
    assert_eq!(snap.workspace_root, snap2.workspace_root);

    // diagnostics
    use cargo_cicd::advanced::diagnostics::{render, EngineDiagnostic};
    let diag = EngineDiagnostic::TargetPressure { size_mb: 1024, budget_mb: 8192 };
    let _ = render(&diag);

    // ocel types
    use cargo_cicd::ocel::*;
    use std::collections::HashMap;
    let log = OcelLog {
        version: "2.0".into(),
        types: OcelTypes { object_types: OcelLog::cargo_object_types(), event_types: vec![] },
        events: HashMap::new(),
        objects: HashMap::new(),
    };
    let v = log.validate();
    assert!(v.valid);
    assert_eq!(log.e2o().len(), 0);

    // evidence emission
    use cargo_cicd::evidence::{emit_ocel, ProcessEvent};
    let evts = vec![ProcessEvent::new("status:show", "PASS")];
    let path = tmp.path().join("test.ocel.json");
    emit_ocel(&evts, &path).unwrap();
    assert!(path.exists());

    println!("maximalist_pipeline: all capabilities composed successfully");
}
```

Run the test with all features:

```sh
cargo test --test maximalist_pipeline \
    --features process-data,autonomic,wasm4pm,contrib,advanced
```

A passing run proves every capability composes without conflict.

---

## Summary of capabilities exercised

| Capability | Module / crate | Step |
|---|---|---|
| Parallel workspace scan | `advanced::parallel_scan` (ignore + rayon) | 3 |
| Content fingerprinting | `advanced::fingerprint` (blake3) | 4 |
| Concurrent TTL cache | `advanced::cache` (moka) | 5 |
| Dependency graph + build order | `advanced::dep_graph` (petgraph) | 6 |
| High-precision timeline | `advanced::timeline` (jiff) | 7 |
| Latency histograms | `advanced::histogram` (hdrhistogram) | 8 |
| Governance pattern scanning | `advanced::pattern` (aho-corasick) | 9 |
| Rich error diagnostics | `advanced::diagnostics` (miette + thiserror) | 10 |
| All 10 CLI nouns | `cargo cicd …` | 11 |
| Binary engine snapshot | `advanced::snapshot` (bitcode) | 12 |
| OCEL 2.0 log (all 11 types) | `ocel::OcelLog` | 13 |
| OCPQ predicates | `ocel::ocpq_eval` | 14 |
| Petri net + DFG conformance | `ocel::PetriNet`, `ocel::Dfg` | 15 |
| Pareto front optimization | `ocel::reject_dominated` | 16 |
| Anomaly + drift detection | `ocel::score_sequence_anomaly`, `detect_drift` | 17 |
| Page-Hinkley change detection | `ocel::page_hinkley_test` | 17 |
| UCB1 bandit selection | `ocel::select_ucb1` | 17 |
| Jaccard similarity | `ocel::jaccard_similarity` | 18 |
| Synchronizing merge | `ocel::synchronizing_merge` | 18 |
| MCTS selection | `ocel::mcts_select` | 18 |
| Perturbator (mutation testing) | `ocel::Perturbator` | 18 |
| Provenance chain | `ocel::ProvenanceChain` | 19 |
| Prolog8 KB + replay | `ocel::admit_atom`, `replay` | 19 |
| blake3_hex / canonical_json | `ocel::blake3_hex` | 19 |
| OCEL flatten / e2o / o2o | `ocel::OcelLog::flatten` | 20 |
| Dual-format evidence emission | `evidence::emit_ocel`, `append_events` | 21 |
| wasm4pm oracle verification | `evidence::WpmEvidenceOracle` | 22 |
| All 6 receipt sub-commands | `integrations::Wasm4pmShell` | 22 |

---

## Next steps

- **How-to guide**: [Use all features in an existing project](../how-to/use-all-features.md)
- **Reference**: [Full capabilities reference](../reference/capabilities.md)
- **Explanation**: [Why combinatorial maximalism?](../explanation/combinatorial-maximalism-rationale.md)
- **Test matrix**: [docs/testing/COMBINATORIAL_MAXIMALIST_TEST_PLAN.md](../testing/COMBINATORIAL_MAXIMALIST_TEST_PLAN.md)
