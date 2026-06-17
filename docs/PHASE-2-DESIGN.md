# Phase 2 Detailed Technical Design

**Vision 2030 — Phase 2: Ecosystem Platform**  
**Document Type:** Technical Design  
**Status:** Proposed  
**Date:** 2026-06-17  
**Audience:** cargo-cicd core engineers, ecosystem partners, platform architects  

---

## Overview

Phase 1 (Weeks 1-12) establishes cargo-cicd as the preeminent Rust workspace lifecycle tool with solid evidence emission and single-oracle adjudication. Phase 2 evolves it into a **platform**: distributed oracles, process mining dashboards, ecosystem-wide analytics, and deep integration with Cargo itself.

**Phase 2 Target Completion:** 12 months post-Phase 1 release  
**Key Success Metrics:**  
- 1,000+ active workspaces using process evidence daily  
- 2-of-3 distributed oracle consensus available as GA feature  
- Process mining dashboard in beta with 50+ active dashboard users  
- cargo integration (VERIFIED badge proposal) submitted to Cargo WG  
- Ontology registry: 10+ published third-party capability ontologies  

---

## Table of Contents

1. [Distributed Oracle Architecture](#1-distributed-oracle-architecture)
2. [Process Mining Dashboards](#2-process-mining-dashboards)
3. [Ecosystem Analytics Service](#3-ecosystem-analytics-service)
4. [cargo Integration (VERIFIED Badge)](#4-cargo-integration-verified-badge)
5. [cargo audit Receipt Integration](#5-cargo-audit-receipt-integration)
6. [Ontology Registry Service](#6-ontology-registry-service)
7. [Pluggable Process Model Runtime](#7-pluggable-process-model-runtime)
8. [Phase 2 Success Metrics and SLOs](#8-phase-2-success-metrics-and-slos)

---

## 1. Distributed Oracle Architecture

**Full design:** See `docs/distributed-oracle-design.md`

### Summary

Phase 2 introduces M-of-N threshold oracle consensus using FROST-Ed25519 threshold signatures (RFC 9591). The default configuration is 2-of-3: two of three independent oracles must independently adjudicate evidence and agree on a verdict.

**Key components**:
- `crates/frost-aggregator/`: FROST RFC 9591 threshold signature library
- `src/integrations/threshold_oracle.rs`: `ThresholdOracle` struct
- Oracle key registry at `https://registry.cargo-cicd.rs/oracles/v1.json`
- Aggregate receipt format (`threshold-receipt/v2`)

**Configuration**:
```toml
# cicd.toml
[evidence.threshold_oracle]
required = 2
total = 3
group_id = "standard-2of3-2026"
```

**Benefits**: Eliminates single-oracle SPOF, satisfies regulatory independence requirements, enables vendor diversity.

**Timeline**: Weeks 1-12 of Phase 2 (see `docs/distributed-oracle-design.md` §6).

---

## 2. Process Mining Dashboards

**Full design:** See `docs/process-mining-architecture.md`

### Summary

Phase 2 ships a process mining dashboard consisting of:
1. `cargo-cicd-collector` — event ingestion service (Rust binary)
2. Dashboard REST API — read-only JSON API served by the collector
3. Dashboard frontend — React + Recharts web application
4. ProM/Disco XES export endpoint

**Dashboard components**:
- Workspace trace timeline (Gantt-style)
- Verdict distribution (pie chart)
- Bottleneck detection (95th percentile histogram)
- Policy violation heatmap
- Certification status per crate/standard
- Conformance fitness score

**Quick start**:
```sh
# Install collector
cargo install cargo-cicd-collector

# Start (auto-watches target/cargo-cicd/evidence/)
cargo cicd evidence doctor --dashboard

# Open browser
open http://localhost:7878
```

**Timeline**: Weeks 1-12 of Phase 2 (see `docs/process-mining-architecture.md` §5).

---

## 3. Ecosystem Analytics Service

The ecosystem analytics service aggregates anonymized evidence metadata across all opted-in workspaces to provide ecosystem-wide intelligence.

### 3.1 Architecture

```
opted-in workspace
  → cargo-cicd --features analytics
    → anonymized metrics batch (no PII, no code)
      → analytics.cargo-cicd.rs/ingest
        → ClickHouse time-series store
          → analytics.cargo-cicd.rs/api
            → Public dashboard: https://analytics.cargo-cicd.rs
```

**Privacy**:
- Opt-in only (never on by default)
- No code, filenames, or workspace paths transmitted
- Only: command name, verdict, duration_ms, feature flags, cargo-cicd version, OS

### 3.2 Analytics API — 10 Endpoints

**Base URL**: `https://analytics.cargo-cicd.rs/api/v1`  
**Authentication**: None (public read endpoints)  
**Rate limiting**: 100 requests/hour per IP  

---

#### Endpoint 1: Ecosystem Verdict Summary

```
GET /verdicts/summary

Response:
{
  "period": "7d",
  "total_executions": 1250000,
  "verdicts": {
    "PASS": { "count": 1100000, "pct": 88.0 },
    "WARN": { "count": 125000, "pct": 10.0 },
    "FAIL": { "count": 25000, "pct": 2.0 }
  },
  "oracle_verdicts": {
    "Accept": { "count": 980000, "pct": 78.4 },
    "Refuse": { "count": 15000, "pct": 1.2 },
    "Blocked": { "count": 255000, "pct": 20.4 }
  }
}
```

---

#### Endpoint 2: Command Popularity

```
GET /commands/popularity?period=30d

Response:
{
  "period": "30d",
  "commands": [
    { "command": "status show", "executions": 850000, "rank": 1 },
    { "command": "test changed", "executions": 620000, "rank": 2 },
    { "command": "git status", "executions": 480000, "rank": 3 },
    { "command": "workspace doctor", "executions": 290000, "rank": 4 },
    { "command": "publish run", "executions": 180000, "rank": 5 }
  ]
}
```

---

#### Endpoint 3: Command Duration Percentiles

```
GET /commands/latency?command=status+show&period=7d

Response:
{
  "command": "status show",
  "period": "7d",
  "sample_count": 850000,
  "latency_ms": {
    "p50": 245,
    "p75": 512,
    "p90": 1100,
    "p95": 1850,
    "p99": 4200
  }
}
```

---

#### Endpoint 4: Feature Flag Adoption

```
GET /features/adoption

Response:
{
  "period": "30d",
  "total_workspaces": 12500,
  "features": [
    { "feature": "default", "workspaces": 12500, "pct": 100.0 },
    { "feature": "process-data", "workspaces": 8200, "pct": 65.6 },
    { "feature": "autonomic", "workspaces": 4100, "pct": 32.8 },
    { "feature": "wasm4pm", "workspaces": 2800, "pct": 22.4 },
    { "feature": "advanced", "workspaces": 950, "pct": 7.6 }
  ]
}
```

---

#### Endpoint 5: Oracle Adoption Rate

```
GET /oracle/adoption?period=30d

Response:
{
  "period": "30d",
  "total_publish_runs": 85000,
  "with_oracle_adjudication": 62000,
  "adoption_rate": 0.729,
  "oracle_distribution": {
    "wasm4pm/0.9.x": { "count": 52000, "pct": 83.9 },
    "wasm4pm/0.8.x": { "count": 8000, "pct": 12.9 },
    "other": { "count": 2000, "pct": 3.2 }
  },
  "threshold_oracle_adoption": {
    "2of3": 1200,
    "3of5": 250,
    "none": 60550
  }
}
```

---

#### Endpoint 6: Conformance Fitness Distribution

```
GET /conformance/fitness-distribution?model=basic-release&period=30d

Response:
{
  "process_model": "basic-release/v1.0",
  "period": "30d",
  "sample_count": 28000,
  "fitness_distribution": {
    "0.95-1.00": { "count": 21000, "pct": 75.0 },
    "0.80-0.95": { "count": 4200, "pct": 15.0 },
    "0.60-0.80": { "count": 1960, "pct": 7.0 },
    "0.00-0.60": { "count": 840, "pct": 3.0 }
  },
  "mean_fitness": 0.91
}
```

---

#### Endpoint 7: Policy Violation Frequency

```
GET /policies/violations?period=7d

Response:
{
  "period": "7d",
  "total_policy_checks": 520000,
  "policy_violations": [
    { "policy": "git_phase_dirty", "violations": 45000, "rate": 0.0865 },
    { "policy": "target_pressure", "violations": 28000, "rate": 0.0538 },
    { "policy": "branch_behind", "violations": 18000, "rate": 0.0346 },
    { "policy": "evidence_stale", "violations": 12000, "rate": 0.0231 }
  ]
}
```

---

#### Endpoint 8: Cargo-cicd Version Distribution

```
GET /versions/distribution?period=30d

Response:
{
  "period": "30d",
  "versions": [
    { "version": "26.6.2", "workspaces": 7800, "pct": 62.4 },
    { "version": "26.5.x", "workspaces": 3200, "pct": 25.6 },
    { "version": "26.4.x", "workspaces": 1500, "pct": 12.0 }
  ],
  "latest_version": "26.6.2"
}
```

---

#### Endpoint 9: Provenance Classification (Phase 2)

```
GET /provenance/distribution?period=30d

Response:
{
  "period": "30d",
  "total_files_scanned": 2850000,
  "classification": {
    "Human": { "count": 1995000, "pct": 70.0 },
    "AI-Assisted": { "count": 627000, "pct": 22.0 },
    "AI-Generated": { "count": 171000, "pct": 6.0 },
    "Unknown": { "count": 57000, "pct": 2.0 }
  },
  "trend": {
    "AI-Generated_7d_delta": "+0.8%"
  }
}
```

---

#### Endpoint 10: Ecosystem Health Score

```
GET /health-score

Response:
{
  "computed_at": "2026-06-17T14:00:00Z",
  "ecosystem_health_score": 87.3,
  "components": {
    "verdict_pass_rate": { "score": 88.0, "weight": 0.3 },
    "oracle_adoption": { "score": 72.9, "weight": 0.2 },
    "conformance_fitness": { "score": 91.0, "weight": 0.2 },
    "evidence_freshness": { "score": 84.5, "weight": 0.15 },
    "policy_compliance": { "score": 91.3, "weight": 0.15 }
  },
  "trend": "+2.1 points vs 30 days ago"
}
```

---

## 4. cargo Integration (VERIFIED Badge)

### 4.1 Concept

The VERIFIED badge would appear on crates.io and in `cargo tree` output for crates with adjudicated process evidence:

```
$ cargo tree
my-crate v1.0.0 [VERIFIED ✓]
├── cargo-cicd v26.6.2 [VERIFIED ✓]
│   └── ...
└── tokio v1.38.0
    └── ...
```

`[VERIFIED ✓]` means: this crate has an oracle-adjudicated `Accept` receipt for its published version.

### 4.2 How It Would Work in cargo Internals

This requires a Cargo RFC (see ADR-014). The proposed mechanism:

1. **Publishing**: When `cargo publish` runs, it reads `[package.metadata.evidence]` from `Cargo.toml`.
2. **crates.io storage**: crates.io stores the `oracle_verdict` and `receipt_path` metadata for each published version.
3. **Badge display**: crates.io renders a "Process Verified" badge when `oracle_verdict = "Accept"`.
4. **cargo tree**: When displaying the dependency tree, cargo fetches evidence metadata from crates.io's API and renders the badge.

**Cargo modification points** (Cargo RFC scope):

```rust
// In src/ops/cargo_publish.rs (conceptual)
fn publish_crate(ws: &Workspace, opts: &PublishOpts) -> CargoResult<()> {
    // ... existing publish logic ...
    
    // NEW: Read evidence metadata
    let evidence = read_evidence_metadata(ws.root_manifest())?;
    if let Some(ev) = evidence {
        info!("Crate has oracle evidence: {:?}", ev.last_verdict);
        // Include evidence in the publish payload to crates.io
        payload.evidence_metadata = Some(ev);
    }
    
    // ... existing upload logic ...
}
```

### 4.3 Verification During `cargo add`

When a user runs `cargo add my-crate`, Cargo could display evidence status:

```
$ cargo add suspicious-crypto-lib
    Updating crates.io index
      Adding suspicious-crypto-lib v2.1.0 to dependencies
⚠ Note: this crate has no oracle-adjudicated process evidence.
  For supply chain transparency, consider crates with VERIFIED status.
  Learn more: https://cargo-cicd.rs/docs/verified-badge
```

This is advisory (not blocking) by default. A Cargo config option could make it blocking:

```toml
# ~/.cargo/config.toml
[cargo-cicd]
require_verified = true    # Refuse to add crates without VERIFIED status
```

### 4.4 RFC Submission Plan

| Step | Timeline | Owner |
|------|----------|-------|
| Write Cargo RFC draft | Phase 2 Week 3 | cargo-cicd team |
| Community review period | Phase 2 Weeks 4-8 | Cargo WG |
| Cargo WG decision | Phase 2 Week 9 | Cargo WG |
| Implementation (if accepted) | Phase 2 Weeks 10-12 | cargo-cicd + Cargo |

---

## 5. cargo audit Receipt Integration

### 5.1 Concept

`cargo-audit` is the standard Rust tool for checking crate dependencies against known security advisories. Phase 2 proposes extending `cargo-audit` with receipt verification:

```sh
$ cargo audit
    Fetching advisory database from `https://github.com/RustSec/advisory-db`
      Scanning Cargo.lock for vulnerabilities (1234 crate versions)
    Checking process evidence receipts...
      ✓ cargo-cicd v26.6.2: Accept (wasm4pm/0.9.2, 2026-06-15)
      ✓ serde v1.0.203: Accept (wasm4pm/0.9.2, 2026-06-10)
      ⚠ some-crate v1.2.3: No receipt found
    
    Vulnerabilities found: 0
    Receipt warnings: 1
```

### 5.2 How cargo audit Calls wpm verify

The integration flow:

1. `cargo audit` reads `[package.metadata.evidence]` from each crate's `Cargo.toml` in the dependency tree.
2. For crates with a `receipt_path`, `cargo audit` downloads the receipt from the crate's published artifacts.
3. `cargo audit` invokes `wpm verify <receipt.json>` for each receipt.
4. wpm verifies the oracle signature against the oracle key registry.
5. `cargo audit` reports the aggregate receipt verification status.

**Implementation in `cargo-audit`** (proposed PR to `rustsec/rustsec`):

```rust
// In src/commands/audit.rs (cargo-audit)
fn check_receipt(crate_meta: &CrateMetadata) -> ReceiptStatus {
    let evidence = crate_meta.evidence_metadata();
    let Some(evidence) = evidence else {
        return ReceiptStatus::NotPresent;
    };
    
    let receipt_bytes = download_receipt(&evidence.receipt_path)?;
    let temp_path = write_temp_receipt(&receipt_bytes)?;
    
    let output = Command::new("wpm")
        .args(["receipt", "doctor", "--format", "json", "--strict", &temp_path])
        .output()?;
    
    match output.status.code() {
        Some(0) => ReceiptStatus::Valid { verdict: "Accept".into() },
        Some(1) => ReceiptStatus::Refused { reason: parse_reason(&output) },
        _ => ReceiptStatus::OracleUnavailable,
    }
}
```

### 5.3 opt-in Configuration

Receipt verification in `cargo-audit` is opt-in:

```toml
# .cargo/audit.toml
[receipts]
check = true                    # Enable receipt checking
require_for_all = false         # Only warn, don't fail, for missing receipts
require_oracle = "wasm4pm"      # Accept only this oracle's receipts
min_oracle_version = "0.9"      # Minimum oracle version
```

---

## 6. Ontology Registry Service

The ontology registry enables teams to publish, discover, and import custom capability ontologies.

### 6.1 Registry Architecture

```
GitHub repository: cargo-cicd-rs/ontology-registry
  ↓ (CI validates ontology syntax + namespace uniqueness)
  ↓
registry.cargo-cicd.rs (static site from GitHub Pages)
  ↓
GraphQL API: registry.cargo-cicd.rs/graphql
SPARQL endpoint: registry.cargo-cicd.rs/sparql
```

The registry is a Git-backed static site. Pull requests add new ontologies. CI validates syntax and namespace uniqueness. GitHub Pages serves the static content. A small GraphQL layer wraps the static data for programmatic access.

### 6.2 GraphQL API

**Schema**:

```graphql
type Query {
  # List all published ontologies
  ontologies(
    org: String,
    capability: String,
    standard: String,
    first: Int = 20,
    after: String
  ): OntologyConnection!

  # Get a specific ontology
  ontology(id: ID!): Ontology

  # Search by SPARQL query
  sparqlQuery(query: String!): SparqlResult!
}

type Ontology {
  id: ID!
  org: String!
  name: String!
  version: String!
  namespace: String!
  description: String!
  standards: [String!]!
  license: String!
  publishedAt: DateTime!
  downloadUrl: String!
  turtleContent: String!
  capabilities: [Capability!]!
}

type Capability {
  name: String!
  kind: CapabilityKind!
  description: String!
  nouns: [String!]!
  verbs: [Verb!]!
  standards: [String!]!
}

enum CapabilityKind {
  COMPLIANCE
  SECURITY
  QUALITY
  CUSTOM
}
```

**Example query**:
```graphql
{
  ontologies(standard: "SLSA-L3") {
    nodes {
      id
      org
      name
      version
      description
      capabilities {
        name
        description
      }
    }
  }
}
```

### 6.3 SPARQL Endpoint

The registry exposes a SPARQL 1.1 endpoint for querying across all registered ontologies:

```
POST https://registry.cargo-cicd.rs/sparql
Content-Type: application/sparql-query

SELECT ?org ?name ?noun ?verb ?description
WHERE {
  ?cap a cc:Capability ;
       cc:isNoun true ;
       skos:prefLabel ?noun ;
       dcterms:description ?description .
  OPTIONAL { ?cap cc:mapsToStandard "SLSA-L3" . }
}
```

The SPARQL endpoint enables advanced ontology reasoning and capability discovery that the GraphQL API cannot support.

### 6.4 ggen Integration

```toml
# ggen.toml
[[ontology.imports]]
registry = "cargo-cicd-rs/ontology-registry"
org = "security-community"
name = "supply-chain-gates"
version = "^1.0"
```

When ggen runs, it:
1. Queries the GraphQL API for the matching ontology version.
2. Downloads the Turtle content.
3. Caches at `target/cargo-cicd/ontology-cache/<org>/<name>/<version>.ttl`.
4. Merges with the local ontology before SPARQL reasoning.

Cache expiry: 24 hours. Force refresh: `ggen --refresh-registry`.

---

## 7. Pluggable Process Model Runtime

**Full design:** See `docs/adr/ADR-020-phase2-pluggable-process-models.md`

### Summary

Phase 2 introduces a process model DSL (RDF/Turtle extension) and a runtime conformance checker. Teams define their own process models as Turtle documents. cargo-cicd checks evidence traces against declared models.

**Key addition**: WASM plugin support via `wasmtime`. Process model plugins can be distributed as signed WASM modules, enabling sandboxed execution of third-party conformance logic:

```rust
// crates/cargo-cicd-pm-runtime/src/wasm_plugin.rs

pub struct WasmProcessModelPlugin {
    store: wasmtime::Store<()>,
    instance: wasmtime::Instance,
}

impl WasmProcessModelPlugin {
    pub fn load(wasm_path: &Path) -> Result<Self> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::from_file(&engine, wasm_path)?;
        let linker = wasmtime::Linker::new(&engine);
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module)?;
        Ok(Self { store, instance })
    }

    pub fn check_conformance(&mut self, trace_json: &str) -> Result<ConformanceResult> {
        let func = self.instance.get_typed_func::<(i32, i32), (i32, i32)>(
            &mut self.store,
            "check_conformance"
        )?;
        
        // Write trace JSON to WASM memory
        let (ptr, len) = self.write_to_wasm_memory(trace_json.as_bytes())?;
        
        // Call conformance check function
        let (result_ptr, result_len) = func.call(&mut self.store, (ptr, len))?;
        
        // Read result from WASM memory
        let result_json = self.read_from_wasm_memory(result_ptr, result_len)?;
        Ok(serde_json::from_str(&result_json)?)
    }
}
```

WASM plugins are sandboxed: no filesystem access, no network access, no process spawning. They receive only the trace JSON and return a conformance result.

**Plugin distribution**: WASM plugins are distributed as `.wasm` files alongside ontology `.ttl` files in the ontology registry. ggen validates plugin signatures before use.

---

## 8. Phase 2 Success Metrics and SLOs

### 8.1 Adoption Targets

| Metric | Phase 2 Target | Measurement |
|--------|---------------|-------------|
| Active workspaces | 1,000+ | Analytics API (opt-in) |
| Daily evidence events | 100,000+ | Analytics API |
| Oracle adjudication rate | >70% of publish runs | Analytics API |
| Process mining dashboard MAU | 50+ | Dashboard login analytics |
| Third-party ontologies | 10+ | Registry PR count |
| Threshold oracle adoption | >10% of oracle users | Analytics API |

### 8.2 Latency SLOs

| Component | SLO | Measurement |
|-----------|-----|-------------|
| Evidence emission (per event) | < 50ms | p95 per-event timing |
| Single oracle adjudication | < 500ms | p95 wpm call duration |
| 2-of-3 threshold adjudication | < 1000ms | p95 total duration |
| Dashboard API | < 500ms | p95 per-request |
| Dashboard page load | < 2s | Lighthouse CI |
| Registry GraphQL query | < 200ms | p95 per-query |
| Collector event ingestion | < 100ms | p95 file-to-store latency |

### 8.3 Reliability SLOs

| Component | Availability SLO |
|-----------|----------------|
| Single oracle (wasm4pm) | 99.9% monthly |
| 2-of-3 threshold oracle | 99.999% monthly |
| Analytics API | 99.5% monthly |
| Dashboard | 99.0% monthly |
| Registry GraphQL | 99.5% monthly |

### 8.4 Test Coverage Requirements

| Test Suite | Coverage Target |
|------------|----------------|
| `tests/threshold_oracle/` | 25 scenarios, 100% pass |
| `tests/conformance/` | 15 scenarios, 100% pass |
| `tests/pm_dashboard/` | 20 scenarios, 100% pass |
| `tests/analytics_api/` | 10 endpoints, 100% pass |
| `tests/ontology_registry/` | 8 scenarios, 100% pass |
| Unit test coverage | > 80% line coverage |

### 8.5 Phase 2 Milestone Schedule

| Milestone | Target Week | Deliverable |
|-----------|------------|-------------|
| M1: Threshold oracle alpha | Week 4 | FROST library + basic 2-of-3 |
| M2: Process mining store | Week 4 | SQLite collector + schema |
| M3: Threshold oracle beta | Week 8 | Full aggregation + receipts |
| M4: Dashboard API | Week 8 | All 12 REST endpoints |
| M5: Dashboard frontend | Week 12 | React app, all 6 components |
| M6: Analytics service | Week 12 | 10 analytics endpoints |
| M7: cargo RFC submission | Week 9 | RFC filed to cargo WG |
| M8: cargo-audit RFC | Week 10 | RFC filed to rustsec |
| M9: Ontology registry | Week 12 | Registry live with 3 orgs |
| M10: WASM process models | Week 12 | Runtime + 2 example plugins |

---

*Document version 1.0 — 2026-06-17*  
*See also: `docs/distributed-oracle-design.md`, `docs/process-mining-architecture.md`, `docs/PHASE-3-DESIGN.md`*
