# Performance Profiler Subagent Definition v2.0.0

## Overview

A comprehensive subagent definition for cargo-cicd that specializes in identifying performance bottlenecks, analyzing pipeline stage latencies, and recommending concrete optimizations. This definition integrates deeply with cargo-cicd's process-data engine architecture.

## Key Enhancements (v1.0 → v2.0)

### 1. **Expanded Specializations** (7 → 8 core specialties)
- Added explicit **XES Evidence Stream Correlation** for process mining and wasm4pm compliance
- Enhanced each specialization with:
  - Detailed capability descriptions
  - Specific module locations in codebase
  - Tool/integration references
  - Applicable use cases

### 2. **Comprehensive Tools & Integrations Section** (5 → 11 detailed integrations)

#### Metrics & Analytics
- **metrics_collector** (src/integrations/metrics_collector.rs)
  - HdrHistogram-backed stage latency aggregation
  - p50/p90/p99/max percentile extraction
  - Parallel worker result merging

- **histogram_analysis** (src/advanced/histogram.rs::StageLatencies)
  - 1µs to 60s bounded tracking
  - 3 significant figures precision
  - Clamp-safe recording; auto-merge support

- **timeline_correlation** (src/advanced/timeline.rs::ProcessTimeline)
  - Nanosecond-precision jiff timestamps
  - Span measurement between arbitrary events
  - ISO-8601 serialization for portability

#### Instrumentation & Observability
- **observability_instrumentation** (src/advanced/observability.rs)
  - RAII PipelineStage guard for automatic timing
  - JSON-formatted tracing output
  - Idempotent global subscriber initialization

- **process_event_evidence** (src/evidence.rs)
  - Lifecycle-based event recording (start/complete pairs)
  - XES (XML Event Stream) and JSONL serialization
  - wasm4pm format compliance

#### Performance Optimization Tools
- **engine_cache_metrics** (src/advanced/cache.rs)
  - Moka concurrent TTL-aware caching
  - Hit/miss frequency estimation
  - Capacity and TTL tuning recommendations

- **parallel_scan_report** (src/advanced/parallel_scan.rs)
  - ignore::WalkBuilder + rayon integration
  - Gitignore-aware multi-threaded scanning
  - Per-extension file type breakdown

#### Adapter Profiling
- **adapter_profiling** (src/adapters/)
  - Seven external-source adapters with typical latencies
  - I/O intensity and subprocess overhead analysis
  - Bottleneck identification methodology

- **feature_flag_matrix** (tests/feature_projection.rs)
  - 6 compilation variants for overhead analysis
  - Expected overhead ranges per feature
  - Build-time vs runtime cost breakdown

### 3. **Enhanced Data Sources** (6 → 7 documented sources)
- ProcessEvent lifecycle pairs (primary)
- EngineState runtime dimensions (secondary)
- cicd.toml [state] snapshots (baseline/historical)
- ProcessTimeline events (high-precision timing)
- Feature flag builds (overhead analysis matrix)
- Adapter traces (observability instrumentation)
- Cache metrics (moka effectiveness)

### 4. **Expanded Constraints** (7 → 9 explicit constraints)
- Added **no_artifact_modification** (cannot corrupt evidence files)
- Added **xes_format_validation** (must validate before analysis)
- Added **minimum_sample_size** (< 5 samples = low confidence warning)
- Structured each constraint with:
  - Clear boolean requirement
  - Detailed rationale
  - Enforcement methodology

### 5. **Rich Guidance Section** (5 → 6 guidance topics with detailed methodologies)

#### Percentile Calculation
- Standard 6-step methodology
- Outlier detection and flagging
- Statistical confidence reporting

#### Feature Flag Profiling
- 6-variant compilation matrix
- 5+ iteration per-variant testing
- Overhead categorization (0-5% negligible to >20% critical)
- Feature gating recommendation threshold (>15%)

#### Adapter Analysis
- Correlate command to execution sequence
- CPU-bound vs I/O-bound classification
- Caching eligibility assessment
- Parallelization opportunity identification

#### Regression Detection
- Baseline loading from cicd.toml [state]
- Delta percentage calculation
- 4-tier severity classification (OK/WARN/significant/severe)
- Code change correlation

#### Optimization Priority
- 6-level strategy ranked by ROI
- P99 latency focus (tail latency elimination)
- Sequential bottleneck addressing (parallelization)
- Profile-before-optimize requirement

#### Cache Analysis
- Occupancy tracking over time
- Hit/miss ratio estimation (>70% good, <30% problematic)
- Capacity tuning (when to increase/decrease)
- TTL tuning (10-30m typical range)
- Cacheable adapter identification

### 6. **Realistic Example Prompts** (5 → 6 scenarios)
- Per-stage latency extraction and slowest adapter identification
- Feature flag overhead comparison with threshold-based gating
- Bottleneck profiling (WalkDir vs file filtering)
- Baseline snapshot generation with regression alerting
- Noun bottleneck identification with optimization prioritization
- Cache metrics analysis with tuning recommendations

### 7. **Approval Gates & Quality Standards**
Five explicit approval gates ensure high-quality analysis:
1. Must cite evidence files by path (target/cargo-cicd/evidence/)
2. Must validate XES structure before percentile extraction
3. Must separate performance analysis from optimization implementation
4. Must establish baseline (cicd.toml [state] or prior runs)
5. Must report confidence level (flag if sample size < 5)

## Architecture Integration Points

### ProcessEvent Lifecycle Model
```
Start Event                    Complete Event
 |                              |
 |--duration_ms: None          |--duration_ms: Some(u64)
 |--command: "status:show"     |--command: "status:show"
 |--case_id: "run-123"         |--case_id: "run-123"
 |--timestamp_iso: ISO8601     |--timestamp_iso: ISO8601
 |--verdict_claimed: PASS      |--verdict_claimed: PASS
 |                             |
 +-------- Span Duration -------+
```

### Seven External-Source Adapters (Typical Latency Ranges)

| Adapter | Operation | Latency | Bottleneck Type |
|---------|-----------|---------|-----------------|
| CargoMetadataAdapter | cargo metadata | 50-200ms | Subprocess + JSON parse |
| TargetScannerAdapter | WalkDir + parallel_scan | 100ms-5s | Filesystem I/O |
| GitStatusAdapter | git status + diff | 50-500ms | Subprocess latency |
| ChangedFileDetector | Regex classify | 10-100ms | Pattern matching |
| TrybuildDetector | Manifest scan | 20-200ms | Glob matching |
| FingerprintAdapter | BLAKE3 hashing | 50-500ms | Crypto throughput |
| ToolchainDetector | Toolchain detect | 10-100ms | File I/O + subprocess |

### Feature Flag Overhead Matrix

| Variant | Flags | Expected p95 Overhead | Recommendation |
|---------|-------|----------------------|-----------------|
| baseline | none | 0% | reference point |
| process-data | +process-data | +5-10% | acceptable |
| autonomic | +process-data+autonomic | +8-15% | monitor; gate if >15% |
| wasm4pm | +process-data+wasm4pm | +3-8% | acceptable |
| advanced | +advanced | -5-10% | beneficial (parallelization) |
| full | all | -2-5% | overall negative (advanced wins) |

### Regression Detection Thresholds

| Delta Range | Status | Action |
|------------|--------|--------|
| < 10% | OK | Monitor |
| 10-50% | WARN | Investigate |
| 50-150% | SIGNIFICANT | Review changes |
| > 150% | SEVERE | Rollback candidate |

## Evidence Format (XES)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.creator="cargo-cicd">
  <trace>
    <string key="case_id" value="run-2026-06-14-22-51-00"/>
    <event>
      <string key="concept:name" value="status:show"/>
      <string key="lifecycle:transition" value="start"/>
      <date key="timestamp_iso" value="2026-06-14T22:51:00Z"/>
    </event>
    <event>
      <string key="concept:name" value="status:show"/>
      <string key="lifecycle:transition" value="complete"/>
      <date key="timestamp_iso" value="2026-06-14T22:51:00.250Z"/>
      <int key="duration_ms" value="250"/>
      <string key="verdict_claimed" value="PASS"/>
    </event>
  </trace>
</log>
```

## Output Format Examples

### JSON Metrics Report
```json
{
  "timestamp": "2026-06-14T22:51:00Z",
  "analysis_type": "latency_percentiles",
  "stages": [
    {
      "name": "target_scan",
      "p50_ms": 150, "p90_ms": 280, "p95_ms": 320, "p99_ms": 450,
      "max_ms": 520, "min_ms": 120, "mean_ms": 180.5, "sample_count": 47
    }
  ],
  "comparison": {
    "baseline_p95_ms": 200,
    "current_p95_ms": 320,
    "pct_change": "+60%",
    "regression_status": "WARN"
  }
}
```

### Markdown Performance Report

Includes:
- Bottleneck summary (worst 3 stages by p99)
- Feature flag overhead table
- Per-adapter latency breakdown
- Cache effectiveness analysis
- Optimization recommendations (ranked by impact)

### CSV Baseline Comparison

```
stage,baseline_p50,baseline_p95,current_p50,current_p95,delta_pct,recommendation
status_show,50,90,55,95,+5.6%,monitor
target_scan,150,250,180,320,+28%,investigate
```

## Key Files Referenced

| File | Purpose |
|------|---------|
| /CLAUDE.md | Architecture reference; noun/verb/adapter definitions; feature flags |
| src/evidence.rs | ProcessEvent lifecycle and XES serialization |
| src/advanced/histogram.rs | HdrHistogram-backed StageLatencies |
| src/advanced/timeline.rs | jiff-backed ProcessTimeline |
| src/advanced/cache.rs | Moka EngineCache for metrics |
| src/advanced/parallel_scan.rs | ignore + rayon scanning |
| src/advanced/observability.rs | PipelineStage instrumentation |
| src/integrations/metrics_collector.rs | MetricsCollector aggregation |
| src/adapters/ | Seven external-source adapters |
| tests/feature_projection.rs | Feature flag surface contract |

## YAML Definition Location

**File:** `/home/user/cargo-cicd/.claude/subagents/performance-profiler.yaml`

**Version:** 2.0.0 (543 lines)

**Status:** Ready for deployment

## Usage Scenarios

### Scenario 1: Bottleneck Identification
**Prompt:** "Why is `cargo cicd status show` slow? Extract p99 latency for each stage."

**Agent Tasks:**
1. Read target/cargo-cicd/evidence/events.xes
2. Filter events for "status:show" command
3. Extract duration_ms from complete events
4. Compute p50/p95/p99 percentiles (StageLatencies)
5. Identify slowest adapter (correlate to sub-stages)
6. Output JSON metrics + markdown report

### Scenario 2: Feature Flag Overhead Profiling
**Prompt:** "Compare performance with/without the `autonomic` feature flag."

**Agent Tasks:**
1. Build 2 variants: baseline, baseline+autonomic
2. Run 5+ iterations each under identical load
3. Extract p95 latency per variant
4. Compute overhead: (autonomic_p95 - baseline_p95) / baseline_p95 * 100
5. Flag if overhead > 15% (problematic threshold)
6. Output comparison table + recommendation

### Scenario 3: Regression Detection
**Prompt:** "Generate a baseline performance snapshot. Alert if any stage exceeds 3x historical p95."

**Agent Tasks:**
1. Load historical events from cicd.toml [state]
2. Compute baseline_p95 for each stage (minimum 5 samples)
3. On new run, extract current_p95 for each stage
4. Compute delta_pct = (current - baseline) / baseline * 100
5. Flag p95 > 3x baseline (150%+ delta) as SEVERE
6. Output CSV with baseline, current, delta, status

### Scenario 4: Cache Analysis
**Prompt:** "Analyze cache hit ratios. Should we increase TTL or capacity?"

**Agent Tasks:**
1. Read observability traces for EngineCache operations
2. Count put() operations (inserts)
3. Count get() calls and hits
4. Estimate hit ratio: hits / (hits + misses)
5. If hit ratio > 70%: capacity is good, consider TTL tuning
6. If hit ratio < 30%: cache is ineffective; review key design
7. Output capacity/TTL tuning recommendations

### Scenario 5: Adapter Efficiency
**Prompt:** "Which adapter is the bottleneck? Recommend caching or parallelization strategy."

**Agent Tasks:**
1. Correlate ProcessEvent.command to adapter execution sequence
2. For each adapter, extract start/complete latency
3. Rank adapters by p99 latency
4. Classify bottleneck: CPU-bound (fingerprint), I/O-bound (TargetScanner), subprocess (git)
5. Recommend:
   - If stateless + repeated: caching (moka EngineCache)
   - If I/O-bound: parallelization (rayon, ignore::WalkBuilder)
   - If subprocess: consider in-process replacement
6. Output analysis with specific module recommendations

## Constraints Enforced

### Read-Only Guarantee
- No code modifications, test modifications, or artifact mutations
- Recommendations are advisory only
- Implementation is caller's responsibility

### Evidence Integrity
- Must validate XES structure before analysis
- Fail fast on malformed events (missing duration_ms, invalid timestamps)
- Report sample size < 5 as "low confidence"

### Statistical Rigor
- Percentile extraction requires minimum 5 samples per stage
- Baseline comparison requires separate builds per feature variant
- Overhead classification is deterministic and reproducible

### Operational Safety
- No live instrumentation (no code injection)
- No modification of ProcessEvent emission logic
- No interference with test fixtures or workspace state
- Post-hoc analysis only; all timing already captured

## Version History

- **v1.0.0** (original definition)
  - 7 specializations
  - 5 tools/integrations
  - 5 example prompts
  - Basic constraints

- **v2.0.0** (current enhancement)
  - 8 specializations (added XES Evidence Stream Correlation)
  - 11 detailed tools/integrations (comprehensive descriptions)
  - 6+ example prompts (realistic scenarios)
  - 9 explicit constraints (with rationales)
  - 6 guidance topics (detailed methodologies)
  - 5 approval gates (quality standards)
  - Session metadata for regeneration

