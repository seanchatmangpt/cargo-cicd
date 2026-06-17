//! Tutorial anchor for `docs/tutorials/03-full-pipeline.md`.
//! For explanation of *why* all capabilities compose without conflict, see
//! `docs/explanation/combinatorial-maximalism.md`.
//!
//! Run:
//!   cargo run --example 03_max_pipeline \
//!       --features process-data,autonomic,advanced
//!
//! What you will see: all ten advanced modules exercised in sequence, a full
//! OCEL evidence record written, and per-stage latency percentiles printed.

use cargo_cicd::evidence::{emit_ocel, evidence_dir, ProcessEvent};
use cargo_cicd::EngineState;

#[cfg(feature = "advanced")]
use cargo_cicd::advanced::{
    cache::EngineCache,
    dep_graph::WorkspaceGraph,
    diagnostics::{render, severity_of, toolchain_mismatch_at},
    fingerprint::{hash_bytes, workspace_digest},
    histogram::StageLatencies,
    observability::{init_tracing, record_event, PipelineStage},
    parallel_scan,
    pattern::MultiPatternScanner,
    snapshot::{decode, encode, EngineSnapshot, StageRecord},
    timeline::ProcessTimeline,
};

fn main() {
    #[cfg(not(feature = "advanced"))]
    {
        eprintln!("Run with --features process-data,autonomic,advanced");
        std::process::exit(1);
    }

    #[cfg(feature = "advanced")]
    run_maximalist_pipeline();
}

#[cfg(feature = "advanced")]
fn run_maximalist_pipeline() {
    use std::time::Duration;

    // ── 1. Observability ────────────────────────────────────────────────────
    init_tracing();
    let _root_stage = PipelineStage::enter("maximalist_pipeline");

    let mut timeline = ProcessTimeline::new();
    timeline.record("pipeline:start");

    // ── 2. Engine state ─────────────────────────────────────────────────────
    let state = EngineState::from_workspace();
    println!("[1/10] workspace: {}", state.workspace.name);
    record_event("engine_state", true);

    // ── 3. Parallel scan ────────────────────────────────────────────────────
    let scan = parallel_scan::scan(std::path::Path::new(".")).unwrap_or_default();
    println!(
        "[2/10] parallel_scan: {} files, {} reclaimable bytes",
        scan.total_files,
        scan.reclaimable_bytes()
    );
    timeline.record("scan:complete");

    // ── 4. Fingerprint (BLAKE3) ──────────────────────────────────────────────
    let digest = workspace_digest(
        &scan
            .per_extension
            .iter()
            .map(|(ext, _)| {
                let bytes = ext.as_bytes();
                (std::path::PathBuf::from(ext), hash_bytes(bytes))
            })
            .collect::<Vec<_>>(),
    );
    println!("[3/10] fingerprint: {digest}");

    // ── 5. Cache ─────────────────────────────────────────────────────────────
    let cache = EngineCache::new(256, Duration::from_secs(300));
    cache.put_labeled("workspace_digest", digest.to_string().into_bytes(), "fingerprint");
    let hit = cache.get("workspace_digest").map(|e| e.len()).unwrap_or(0);
    println!("[4/10] cache: {hit} bytes cached");

    // ── 6. Dependency graph ──────────────────────────────────────────────────
    let mut graph = WorkspaceGraph::new();
    for member in &state.workspace.members {
        graph.add_crate(member);
    }
    let build_order = graph.build_order().unwrap_or_default();
    println!(
        "[5/10] dep_graph: {} members, build order: {:?}",
        state.workspace.members.len(),
        build_order
    );

    // ── 7. Timeline ───────────────────────────────────────────────────────────
    timeline.record("pipeline:midpoint");
    println!("[6/10] timeline: {} events recorded", timeline.len());

    // ── 8. Pattern scanner ────────────────────────────────────────────────────
    let scanner = MultiPatternScanner::new(&["TODO", "FIXME", "HACK", "XXX"])
        .expect("valid governance patterns");
    let matches = scanner.scan(concat!(
        "TODO: update this\n",
        "FIXME: broken\n",
        "OK line",
    ));
    println!("[7/10] pattern: {} governance matches", matches.len());

    // ── 9. Diagnostics (miette) ───────────────────────────────────────────────
    let source = format!("[state]\ntoolchain = \"{}\"", state.workspace.toolchain);
    let diag = toolchain_mismatch_at(
        "cicd.toml",
        &source,
        &state.workspace.toolchain,
        &state.workspace.toolchain,
        &state.toolchain.rust_version,
    );
    let rendered = render(&diag);
    let severity = severity_of(&diag);
    println!("[8/10] diagnostics: {severity:?} — {} chars", rendered.len());

    // ── 10. Histogram ─────────────────────────────────────────────────────────
    let mut latencies = StageLatencies::new("workspace_scan");
    latencies.record(45_000); // 45 ms in microseconds
    latencies.record(12_000); // 12 ms
    latencies.record(1_000);  // 1 ms
    latencies.record(8_000);  // 8 ms
    let p99_us = latencies.p99();
    println!("[9/10] histogram: workspace_scan p99 = {}µs", p99_us);

    // ── 11. Snapshot (bitcode) ────────────────────────────────────────────────
    timeline.record("pipeline:end");
    let snapshot = EngineSnapshot {
        workspace_root: state.workspace.root_path.clone(),
        toolchain: state.toolchain.rust_version.clone(),
        changed_files: state.changed_files.changed_rs_files.clone(),
        target_bytes: state.target.total_size_bytes,
        git_phase: state.git_phase.branch.clone(),
        schema_version: EngineSnapshot::current_schema_version(),
        stages: vec![
            StageRecord { name: "scan".into(), ok: true, elapsed_ms: 45 },
            StageRecord { name: "fingerprint".into(), ok: true, elapsed_ms: 12 },
            StageRecord { name: "dep_graph".into(), ok: true, elapsed_ms: 8 },
        ],
    };
    let encoded = encode(&snapshot).expect("snapshot encodes");
    let _restored = decode(&encoded).expect("snapshot round-trips");
    println!("[10/10] snapshot: {} bytes (bitcode)", encoded.len());

    // ── 12. OCEL evidence ─────────────────────────────────────────────────────
    let mut events = vec![
        ProcessEvent::new("status show", "PASS"),
        ProcessEvent::new("target show", "PASS"),
        ProcessEvent::new("workspace doctor", "PASS"),
        ProcessEvent::new("evidence audit", "PASS"),
    ];
    for e in &mut events {
        e.case_id = Some("maximalist_pipeline".to_string());
    }

    let dir = evidence_dir();
    std::fs::create_dir_all(&dir).expect("creates evidence dir");
    let out = dir.join("max_pipeline.ocel.json");
    emit_ocel(&events, &out).expect("emit OCEL evidence");

    // ── Summary ───────────────────────────────────────────────────────────────
    let total = timeline.total_span().map(|s| format!("{s}")).unwrap_or_else(|| "n/a".into());
    println!();
    println!("pipeline complete — all 10 advanced modules exercised");
    println!("  total span : {total}");
    println!("  evidence   : {}", out.display());
    println!("  verdict    : Blocked (wpm not required — oracle is optional)");
}
