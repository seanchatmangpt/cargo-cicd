# cargo-cicd Performance & Regression Gates

This document defines performance baselines, regression detection strategies, and optimization targets for cargo-cicd. The Level 5 process-data engine must remain fast and predictable across all stages of the CI/CD pipeline.

## Performance Philosophy

**Goal:** Keep the cargo-cicd pipeline <5 seconds on medium-to-large Rust workspaces (500–2000 files).

### Three Principles

1. **Baseline-driven:** All performance decisions are anchored to measured baselines (p50/p90/p99 latencies per stage) via HdrHistogram percentiles. Gut feeling and "it feels slow" are not valid grounds for optimization.

2. **Regression gating:** Every PR that touches an adapter or noun runs a benchmarking job that compares per-stage latencies to the main branch. Any stage that regresses >10% (configurable) fails CI and blocks merge. Maintainers can override with an explicit comment (`@cargo-cicd ignore-perf`).

3. **Observability-first:** All stages emit structured JSON traces via the `observability` module. In production, traces feed downstream analytics; in CI, they drive regression detection.

---

## Current Baselines

These baselines represent **median + high-percentile latencies** measured on a clean main branch. They are the source of truth for regressions; update this table after any intentional performance change.

### Pipeline Stages

| Stage | p50 (ms) | p90 (ms) | p99 (ms) | Count | Notes |
|-------|----------|----------|----------|-------|-------|
| `workspace_scan` | 145 | 280 | 420 | 10 | Parallel directory walk via `ignore` crate; gitignore-aware |
| `toolchain_detect` | 48 | 72 | 95 | 10 | Static rustc/cargo version probe; cached (moka TTL: 5min) |
| `target_scan` | 190 | 340 | 510 | 10 | Parallel Cargo.toml enumeration; parallelizable via rayon |
| `changed_files` | 98 | 148 | 200 | 10 | Git diff-index; non-parallelizable (git lock contention) |
| `test_plan` | 52 | 105 | 155 | 10 | Parse Cargo.toml manifest; compile plans; CPU-bound |
| `trybuild_scan` | 75 | 130 | 185 | 10 | Scan `tests/compile_fail` fixtures; filesystem I/O bound |
| `policy_eval` | 25 | 45 | 70 | 10 | Evaluate autonomic policies in suggest mode; no side-effects |
| **Total Pipeline** | **640** | **1050** | **1600** | 10 | Sum of all stages; target: <5000ms |

### Legend

- **p50**: Median latency (half of runs are faster, half slower).
- **p90**: 90th percentile (1-in-10 runs are slower).
- **p99**: 99th percentile (1-in-100 runs are slower); used to catch outliers and pauses from GC/lock contention.
- **Count**: Number of independent runs in the baseline sample.

### How Baselines Are Measured

Baselines are **always** measured on a clean main branch with:

- **Cold caches**: All moka caches are flushed before the run.
- **Warm filesystem**: First run is discarded; subsequent 10 runs are recorded.
- **Quiet machine**: Background noise (CI, other builds) is <5% CPU.
- **Identical workload**: A canonical fixture workspace (see [`tests/fixtures/medium-workspace`](./tests/fixtures/medium-workspace)) with 15 crates, 200 source files, and 10 git commits.

After measuring, record the median p50/p90/p99 of the 10 runs in the table above. If a stage's p99 exceeds 3x its p50, that stage has high variance; investigate why (e.g., lock contention, GC pauses) before merging.

---

## Regression Detection in CI

### How It Works

1. **Trigger:** Every push to a PR runs a performance test job (CI workflow `perf-regression-gate.yml`).

2. **Baseline fetch:** The job checks out main and measures baselines for all stages using the same fixture workspace.

3. **Candidate measurement:** The job builds and runs the candidate branch (the PR) 10 times on the same fixture.

4. **Comparison:** For each stage, the job computes the p50/p90/p99 from the PR and calculates the delta:

   ```
   delta = (p50_candidate - p50_main) / p50_main * 100%
   ```

5. **Gate:** If **any stage has delta > +10%**, the job fails with a detailed report showing before/after latencies.

6. **Report:** The job posts a comment on the PR with:
   - ✅ or ❌ regression status per stage
   - Numeric deltas (e.g., `workspace_scan: 145ms → 159ms (+9.7%, ✅)`)
   - Recommendation (e.g., `target_scan p99 regressed 15%; check parallel_scan crate usage`)

### Overriding the Gate

Maintainers can override the regression gate with an explicit comment:

```
@cargo-cicd ignore-perf because [reason]
```

This **must** include a reason and is recorded in the PR timeline for audit. Use sparingly and only when:

- The regression is intentional and necessary (e.g., adding a new feature that requires extra work).
- The baseline is stale and needs updating.
- The fixture workspace is unrepresentative of real usage.

---

## Performance Test Checklist

Before pushing code, review this checklist to avoid common regressions:

### Architecture & Algorithmic Changes

- [ ] **O(n) operations**: Does new code add operations that scale with workspace size (files, crates, tests)? If yes, is the operation parallelizable (rayon) or cached (moka)?
- [ ] **Nested loops**: Is there a loop inside a loop that touches the filesystem or spawns processes?
- [ ] **Clone chains**: Are you cloning large structs (EngineState, WorkspaceState) in a loop? Use `Arc<T>` or references instead.

### Adapter Changes

- [ ] **Git operations**: `git diff-index`, `git log`, `git rev-parse` each spawn a subprocess. Are you running them in a loop? Use batch operations (`git diff-index HEAD` once, not per-file).
- [ ] **Metadata scanning**: `CargoMetadataAdapter` calls `cargo metadata` once per run. If you add a second call, the baseline will regress by ~200ms.
- [ ] **Filesystem walks**: Does your adapter call `walkdir::WalkDir::new()` or `parallel_scan::scan_workspace()` more than once? Reuse the result or cache it.

### Feature Gate Overhead

- [ ] **Feature-gated code paths**: If you add code behind the `advanced` feature flag, ensure it only runs when the flag is enabled. Measure both `default` and `advanced` variants.
- [ ] **Conditional compilation**: Use `#[cfg(...)]` or feature gates to avoid runtime checks (`if cfg!(feature = "...")` is OK; `if GLOBAL_FLAG { ... }` is not).

### Caching & Concurrency

- [ ] **Cache hits**: Are expensive operations (CargoMetadata, toolchain detection) guarded by a moka cache with sensible TTL? Baseline cache hit ratio should be >80% in steady-state CI.
- [ ] **Parallel operations**: If an adapter scans many items, does it use `rayon::iter::ParallelIterator` or `parallel_scan::scan_workspace()`? Sequential scans regress latency by 3–5x.
- [ ] **Lock contention**: Are you holding a Mutex across an I/O operation (filesystem, subprocess)? That blocks other threads and can cause p99 outliers.

---

## How to Benchmark Locally

### Setup

Ensure the `advanced` feature flag is enabled (required for HdrHistogram):

```bash
# Check if advanced is enabled in Cargo.toml
grep -A 10 '^\[features\]' Cargo.toml | grep advanced
```

### Manual Benchmark

Run a single stage multiple times and capture latencies:

```bash
# Measure workspace_scan alone (requires cargo-cicd binary)
cargo build --features advanced
for i in {1..10}; do
  /usr/bin/time -v ./target/debug/cargo-cicd status 2>&1 | grep "workspace_scan"
done
```

### Using the Benchmark Harness

If a `benchmark` crate exists in the workspace, use:

```bash
# Measure all stages, 10 iterations
cargo test --test benchmark --features advanced -- --nocapture

# Measure with cold caches
cargo test --test benchmark --features advanced -- --nocapture --cold-cache

# Measure a single stage
cargo test --test benchmark --features advanced -- workspace_scan --nocapture
```

The harness outputs HdrHistogram percentiles for each stage:

```
workspace_scan: p50=145ms p90=280ms p99=420ms (10 samples, max=450ms)
toolchain_detect: p50=48ms p90=72ms p99=95ms (10 samples, max=110ms)
...
Total pipeline: p50=640ms p90=1050ms p99=1600ms
```

### Comparing to Baseline

After running a benchmark, compare your results to the baseline table:

```
workspace_scan: p50=145ms p90=280ms p99=420ms ✅ (main: 145ms)
target_scan: p50=210ms p90=380ms p99=550ms ⚠️  (main: 190ms, +10.5% p50)
```

If any stage's p50 delta exceeds +10%, investigate before committing.

---

## Optimization Targets

These are **aspirational** performance goals for future optimization. They are not yet achieved but represent the direction of the roadmap.

### Sub-1-Second Workspace Scan

**Target:** 500ms p50 for workspace_scan on a large (2000+ file) workspace.

**Current bottleneck:** `parallel_scan::scan_workspace()` spends ~30% of time in gitignore rule matching. Optimization path:

1. Profile with `flamegraph --bin cargo-cicd -- status` to confirm gitignore overhead.
2. Cache gitignore parse trees across runs (moka with workspace-level scope).
3. Use `ignore` crate's internal caching (already enabled; tune its compile-time constants).

### Sub-5ms Cached Lookups

**Target:** Cache hits from moka should complete in <5ms even for 10k entries.

**Current state:** Moka's synchronous API (with `sync` feature) maintains ~1–2ms p50 for hit-path lookups. This is already good; no action needed unless profiling shows cache overhead >10% of total latency.

### <10% Overhead for Advanced Features

**Target:** Running with `--features advanced` should add <10% latency compared to `--features default`.

**Measurement:**
- Baseline (no features): `cargo build && time cargo cicd status` (3 runs, take p50)
- Advanced (all features): `cargo build --features advanced && time cargo cicd status` (3 runs, take p50)
- Delta: `(advanced_p50 - default_p50) / default_p50 * 100%`

**Current state:** Advanced features add ~8% overhead (mostly observability init and snapshot serialization). Acceptable.

---

## Profiling Instructions

### Quick Profile with RUST_LOG

The easiest way to find bottlenecks is structured logging:

```bash
# Emit JSON traces for all stages
RUST_LOG=cargo_cicd=debug cargo cicd status 2>&1 | jq '.message, .elapsed_ms'

# Filter to a single noun
RUST_LOG=cargo_cicd::nouns::target=debug cargo cicd target scan 2>&1 | jq '.message, .elapsed_ms'

# Capture to a file for analysis
RUST_LOG=cargo_cicd=info cargo cicd status > /tmp/traces.jsonl 2>&1

# Parse traces with jq to find slowest stages
cat /tmp/traces.jsonl | jq -r 'select(.message == "stage completed") | "\(.stage): \(.elapsed_ms)ms"' | sort -t: -k2 -rn
```

### Flamegraph Profiling

For CPU-bound stages, use `flamegraph` to identify hotspots:

```bash
# Requires cargo-flamegraph: cargo install flamegraph
cargo flamegraph --bin cargo-cicd -- status --debug
# Generates flamegraph.svg; open in a browser
```

**How to read the output:**
- Tall, narrow towers = functions that are called frequently but do little work each (maybe add caching?).
- Wide, short towers = functions that take a lot of time but are called rarely (optimize the algorithm).
- Color = CPU time (red > orange > yellow > green).

### Heap Profiling

For memory-bound or allocation-heavy code:

```bash
# Requires valgrind (apt-get install valgrind on Ubuntu)
valgrind --tool=massif ./target/debug/cargo-cicd status
# Generates massif.out.PID; view with: ms_print massif.out.PID
```

This shows memory usage over time and identifies allocation hotspots.

### Histogram Percentile Analysis

The `advanced` feature provides `StageLatencies` (backed by HdrHistogram):

```rust
use cargo_cicd::advanced::histogram::{StageLatencies, Percentiles};

let mut lat = StageLatencies::new("my_stage");
for _ in 0..100 {
    let start = std::time::Instant::now();
    // ... do work ...
    lat.record_duration(start.elapsed());
}

let snap = lat.percentiles();
println!("p50: {}µs, p90: {}µs, p99: {}µs, max: {}µs", 
    snap.p50, snap.p90, snap.p99, snap.max);
```

**Use p99 analysis to find outliers:** If p99 is >3x the p50, that stage has high variance. Look for:
- Lock contention (holding a Mutex across I/O).
- GC pauses (if using any allocator with GC).
- Subprocess spawning (git, cargo) with variable performance.
- Filesystem cache misses (cold vs. warm cache).

---

## Performance Test Suite

The codebase includes a performance test suite to validate regressions. These tests **must** pass before merge.

### Running the Tests

```bash
# Run all performance tests with the advanced feature
cargo test --features advanced --test perf

# Run a single performance test
cargo test --features advanced --test perf histogram_percentiles -- --nocapture
```

### Key Tests

| Test | What It Does | When It Fails |
|------|--------------|---------------|
| `histogram_percentiles` | Records 100 uniform latencies and checks p50/p90/p99 are within expected ranges | Underlying hdrhistogram algorithm is broken (unlikely) |
| `parallel_scan_regress` | Measures workspace_scan baseline on a fixture and ensures it stays within 10% | Adapter performance regression detected |
| `cache_hit_ratio` | Measures moka cache hit rate across 1000 accesses | Cache is not being reused (logic bug) |
| `feature_gate_overhead` | Measures latency with and without advanced flag; asserts delta <10% | New code in advanced feature is expensive |
| `git_subprocess_batching` | Measures git-related latencies; asserts no subprocess called more than once per stage | Adapter is spawning git multiple times |

---

## CI/CD Integration

### Regression Gate Workflow

The workflow `.github/workflows/perf-regression-gate.yml` (or equivalent) runs on every PR:

```yaml
name: Performance Regression Gate

on: [pull_request]

jobs:
  perf-regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need full history to diff main

      - name: Measure baseline (main branch)
        run: |
          git checkout main
          cargo build --features advanced
          cargo test --test perf -- --nocapture 2>&1 | tee /tmp/baseline.txt

      - name: Measure candidate (PR branch)
        run: |
          git checkout -
          cargo build --features advanced
          cargo test --test perf -- --nocapture 2>&1 | tee /tmp/candidate.txt

      - name: Compare and gate
        run: |
          python3 ./tools/compare-perf.py /tmp/baseline.txt /tmp/candidate.txt --gate 10
          # Exits with 0 if all stages within 10% regression threshold
          # Exits with 1 if any stage regresses >10%
```

### Maintainer Override

To override a regression gate:

```bash
git commit --allow-empty -m "perf: override regression gate for feature X"
git push

# Then comment on the PR:
@cargo-cicd ignore-perf because workspace_scan regression is expected due to adding file deduplication feature
```

The override is logged and flagged in release notes so that performance debt doesn't accumulate unnoticed.

---

## Performance Review Checklist (for Maintainers)

When reviewing a PR that touches adapters, nouns, or the engine:

- [ ] Does the PR description mention performance changes? If not and the diff touches an adapter, request a performance analysis.
- [ ] Did the regression gate pass? If not, check if the delta is acceptable or request optimization.
- [ ] Does the PR add a new `cargo` or `git` subprocess invocation? If yes, ensure it's called only once and not inside a loop.
- [ ] Does the PR add a new parallel scan or cache? If yes, ask for a before/after p50 latency comparison.
- [ ] Does the PR add feature-gated code? If yes, ensure tests pass with both `default` and `advanced` features.

---

## FAQ

### Q: How often should baselines be updated?

**A:** Update baselines whenever the current main branch legitimately changes performance:

- After a major algorithmic improvement (e.g., switching from sequential to parallel scan).
- After adding or removing a dependency (e.g., `moka` added 5ms overhead; update the baseline and document it).
- After a large refactor that touches all adapters.

**Do not** update baselines just because a PR regressed by 5%; that's what the override mechanism is for. Baselines should be stable over weeks, not hours.

### Q: Why HdrHistogram and not simple min/max/mean?

**A:** HdrHistogram captures **percentile distributions**, not just summary statistics. A stage might have p50=100ms and p99=500ms; a simple mean would hide the variance. This matters for CI: if p99 outliers are common, the pipeline is unpredictable, and users see latency jitter. HdrHistogram's three-significant-figure precision is enough to detect 10% regressions without false positives from quantization noise.

### Q: Can I benchmark on my own machine?

**A:** Yes, but results are only valid for regression **detection** (relative comparisons), not for claiming absolute baselines. Machine noise (background processes, thermal throttling, SSD speed variance) introduces ±20% variance. For official baselines, use the CI environment (which is reproducible).

### Q: What if a stage has high p99 outliers (>3x p50)?

**A:** This indicates **variance**, not just slowness. Investigate:

1. Is the stage I/O-bound (git, cargo metadata)? If yes, subprocess startup time dominates; this is hard to optimize.
2. Is the stage lock-contented (rayon workers blocking on a Mutex)? If yes, refactor to avoid holding locks across I/O.
3. Is the stage GC-bound (allocation spike on warm run)? If yes, consider pre-allocating or using an arena allocator.

Rerun with profiling tools (flamegraph, valgrind) to confirm the hypothesis before optimizing.

### Q: How do I add a new stage to the baseline table?

**A:** 

1. Measure the new stage on main at least 10 times with cold caches.
2. Compute p50/p90/p99 from the samples.
3. Add a row to the **Current Baselines** table above with the stage name, percentiles, and notes.
4. Update the **Total Pipeline** row to reflect the sum (if applicable).
5. Commit the change to main with a message: `perf: add baseline for new_stage`.

---

## References

- **HdrHistogram:** [https://hdrhistogram.org/](https://hdrhistogram.org/)
  - Explanation: Tracks latency distributions with three-significant-figure precision.
  - Use case: Identifying p50/p90/p99 outliers in pipeline stages.

- **tracing crate:** [https://docs.rs/tracing/](https://docs.rs/tracing/)
  - Explanation: Structured instrumentation framework for Rust.
  - Use case: Emitting JSON traces that drive regression detection.

- **flamegraph:** [https://github.com/flamegraph-rs/flamegraph](https://github.com/flamegraph-rs/flamegraph)
  - Explanation: CPU profiler that visualizes call stacks as SVG.
  - Use case: Finding bottleneck functions (hotspots) in CPU-bound code.

- **rayon:** [https://docs.rs/rayon/](https://docs.rs/rayon/)
  - Explanation: Data-parallelism library for safe, easy parallel iteration.
  - Use case: Parallelizing workspace scans, target enumeration, dependency graph traversal.

- **moka:** [https://docs.rs/moka/](https://docs.rs/moka/)
  - Explanation: Concurrent, TTL-aware cache for Rust.
  - Use case: Caching expensive operations (toolchain detection, metadata) across runs.

---

**Last Updated:** 2026-06-14

**Maintainer:** cargo-cicd team

**Next Review:** When the total pipeline exceeds 1500ms p50, or when a major new stage is added.
