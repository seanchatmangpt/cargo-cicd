# MCP Server Integration Strategy for cargo-cicd

**Document Version:** 1.0  
**Date:** 2026-06-14  
**Target Audience:** Architecture leads, integration engineers, plugin developers

---

## Executive Summary

cargo-cicd v26.6.2 is a Level 5 process-data engine exposed as a boring Rust CI/CD helper. This document designs a comprehensive MCP (Model Context Protocol) server integration strategy that:

1. **Identifies high-value external MCP servers** (GitHub, workspace introspection, environment inspection)
2. **Defines custom MCP servers** that extend cargo-cicd capabilities (wasm4pm oracle, Rust docs lookup, policy suggestion)
3. **Proposes a plugin architecture** for MCP-driven behavior extension
4. **Specifies a declarative configuration format** in `cicd.toml` for MCP dependencies

The strategy maintains cargo-cicd's core invariants:
- Adapter-based external integration (no business logic in IO)
- EngineState as the single source of truth
- Feature-flag-based capability gating
- Autonomic policies in suggest-mode-only by default

---

## Design Principles

### 1. Adapter-First Architecture
All MCP servers integrate through the existing adapter layer (`src/adapters/`). Each MCP server becomes a **new adapter** that translates between MCP capabilities and `EngineState` dimensions.

**Rationale:** cargo-cicd's strength is separating external representations from business logic. MCP servers are external; adapters own the translation.

### 2. Feature-Gated Integration
MCP servers are gated by feature flags corresponding to their domain. New feature flags follow the pattern:
- `mcp-github` — GitHub API integration
- `mcp-workspace` — Workspace introspection
- `mcp-rustdoc` — Rust docs lookup
- `mcp-wasm4pm` — wasm4pm oracle server
- `mcp-policy` — Policy suggestion engine

**Rationale:** Users opt into integration footprint. Minimal-install cargo-cicd remains a lightweight CLI with no external MCP dependency.

### 3. Evidence-Driven Decisions
When MCP servers provide data, that data flows through `ProcessEventState` as evidence. Policy decisions are always auditable in `cicd.toml [[events]]`.

**Rationale:** The autonomic layer must account for external MCP recommendations. Evidence leaves a trail.

### 4. Graceful Degradation
If an MCP server is unavailable, cargo-cicd continues with reduced capability. Adapters return `WpmVerdict::Partial` (or equivalent) rather than fail hard.

**Rationale:** cargo-cicd must work offline. MCP servers enhance local-first behavior; they do not replace it.

---

## Part 1: External MCP Servers to Integrate

### 1.1 GitHub API Server

**Purpose:** Query GitHub API for branch status, PR metadata, releases, and code ownership.

**Domain Gaps Filled:**
- Current: `GitStatusAdapter` reads local git repo state only
- Gap: Cannot check if a branch is protected, if CI checks pass, or if a PR is mergeable
- Solution: MCP GitHub server provides CI status, merge queue state, and code review verdict

**Capabilities to Expose:**
```
github:read-branch-status
  Input: repo owner/name, branch ref
  Output: protection rules, CI checks, required reviews

github:get-pr-metadata
  Input: repo, PR number
  Output: merge status, checks, reviewers, draft status

github:list-recent-releases
  Input: repo
  Output: tag, asset URLs, creation timestamp

github:code-ownership
  Input: repo, file path(s)
  Output: CODEOWNERS entries, reviewers for path
```

**Integration Points:**
- **Adapter:** `GitHubMcpAdapter` in `src/adapters/github_mcp.rs`
- **State Dimension:** `GitPhaseState` (extend with `merge_ready`, `ci_checks`)
- **Feature Flag:** `mcp-github`
- **Config:** `cicd.toml [mcp.github]`

**Example Configuration:**
```toml
[mcp.github]
enabled = true
url = "http://localhost:3000"  # MCP server endpoint
repo = "seanchatmangpt/cargo-cicd"
verify_checks = ["cargo-test", "cargo-clippy"]
require_approval = true
```

**Rationale:**
- GitHub is the canonical source of CI/CD truth for most Rust projects
- Prevents force-push of branches with pending CI checks
- Integrates cargo-cicd into larger SDLC workflows (code review gates)

---

### 1.2 Workspace Introspection Server

**Purpose:** Provide rich workspace metadata beyond what `cargo metadata` offers: dependency graphs, feature-enabled crates, build time estimates, and security advisory data.

**Domain Gaps Filled:**
- Current: `CargoMetadataAdapter` reads manifest metadata only
- Gap: Cannot determine which crates will be affected by a dependency upgrade, or which crates have known CVEs
- Solution: MCP workspace server provides transitive analysis and advisory integration

**Capabilities to Expose:**
```
workspace:dependency-graph
  Input: workspace root
  Output: DAG of all dependencies (direct/transitive), features enabled per edge

workspace:affected-crates
  Input: crate name, changed files or features
  Output: list of crates affected by change to that crate

workspace:build-graph
  Input: none
  Output: compilation order, estimated build time, parallelism opportunity

workspace:security-advisories
  Input: none or crate filter
  Output: list of CVEs/yanked versions in dependency tree

workspace:feature-combination-matrix
  Input: none
  Output: cartesian product of feature combinations to test
```

**Integration Points:**
- **Adapter:** `WorkspaceMcpAdapter` in `src/adapters/workspace_mcp.rs`
- **State Dimensions:** `TestPlanState`, `ChangedFileState`, `TargetState`
- **Feature Flag:** `mcp-workspace`
- **Config:** `cicd.toml [mcp.workspace]`

**Example Configuration:**
```toml
[mcp.workspace]
enabled = true
url = "http://localhost:3001"
include_advisories = true
estimate_build_time = true
feature_matrix_limit = 32  # Don't test more than 32 combinations
```

**Rationale:**
- Enables intelligent changed-test selection (run only tests for affected crates)
- Prevents build failures from supply-chain compromises
- Provides data for target directory pressure warnings (build artifact prediction)

---

### 1.3 Environment Inspection Server

**Purpose:** Query system state: installed tools, Rust toolchain variants, system limits (disk, CPU), and developer environment health.

**Domain Gaps Filled:**
- Current: `ToolchainDetector` reads only `rust-toolchain.toml`
- Gap: Cannot detect mismatched nightly features, or predict if MSRV compilation will succeed
- Solution: MCP environment server provides multi-version Rust detection and system resource queries

**Capabilities to Expose:**
```
env:installed-rustc-versions
  Input: none
  Output: list of installed toolchains, their channels, targets

env:system-resources
  Input: none
  Output: available disk, CPU cores, RAM, temp dir capacity

env:development-tools
  Input: tool names (e.g., "llvm-tools", "miri", "clippy")
  Output: installed versions, paths, health check results

env:target-dir-distribution
  Input: workspace root
  Output: breakdown by target/debug/, target/release/, target/*/
           with size, age, and reclamation opportunity

env:ci-environment-type
  Input: none
  Output: "local" | "github-actions" | "gitlab-ci" | "circleci", etc.
```

**Integration Points:**
- **Adapter:** `EnvironmentMcpAdapter` in `src/adapters/env_mcp.rs`
- **State Dimensions:** `ToolchainState`, `TargetState`, `WorkspaceState`
- **Feature Flag:** `mcp-environment`
- **Config:** `cicd.toml [mcp.environment]`

**Example Configuration:**
```toml
[mcp.environment]
enabled = true
url = "http://localhost:3002"
check_msrv_toolchain = true
predict_build_time = true
alert_on_low_disk = true
low_disk_threshold_gb = 5.0
```

**Rationale:**
- Prevents MSRV regressions by testing against declared MSRV before pushing
- Alerts before target/ fills the disk (critical for CI systems with limited ephemeral storage)
- Enables hermetic, reproducible builds (detects if environment drifted)

---

### 1.4 Process Mining Server (wasm4pm Oracle)

**Purpose:** Submit evidence (XES format) to the wasm4pm oracle for process compliance audit.

**Domain Gaps Filled:**
- Current: `Wasm4pmShell` shell-outs to `wpm` binary
- Gap: No structured MCP interface; complex manual CLI parsing
- Solution: MCP wasm4pm server provides rich process verdict types, compliance scoring, and drill-down diagnostics

**Capabilities to Expose:**
```
wasm4pm:submit-evidence
  Input: XES file path, submission metadata
  Output: verdict (Accept/Warn/Refuse), compliance score, non-conformance details

wasm4pm:explain-verdict
  Input: verdict UUID from previous submission
  Output: human-readable explanation of why verdict was issued

wasm4pm:conformance-signature
  Input: evidence file(s)
  Output: SHA256 fingerprint, enablement flags, oracle version
```

**Integration Points:**
- **Adapter:** `Wasm4pmMcpAdapter` in `src/integrations/wasm4pm_mcp.rs`
- **State Dimension:** `ProcessEventState` (extend verdict types)
- **Feature Flag:** `mcp-wasm4pm` (implies `wasm4pm`)
- **Config:** `cicd.toml [mcp.wasm4pm]`

**Example Configuration:**
```toml
[mcp.wasm4pm]
enabled = true
url = "http://localhost:3003"
oracle_version = "26.6.2"
require_accept = true
fallback_to_shell = true  # If MCP unavailable, use wpm binary
```

**Rationale:**
- Structured MCP interface is more maintainable than CLI parsing
- Enables programmatic policy decisions based on oracle verdict
- Decouples cargo-cicd release cycle from wasm4pm updates

---

## Part 2: Custom MCP Servers Designed for cargo-cicd

### 2.1 wasm4pm Oracle MCP Server

**Name:** `cargo-cicd-wasm4pm-oracle`  
**Language:** Rust  
**Role:** Replace shell-out adapter with structured MCP interface to wasm4pm.

**MCP Resources:**
```
resource: wasm4pm://oracle/status
  Description: Current oracle status and version
  Content: JSON { version, uptime, enabled_capabilities }

resource: wasm4pm://verdict/{submission_uuid}
  Description: Detailed verdict for a submission
  Content: JSON { verdict, score, details, timestamp }
```

**MCP Tools:**
```
tool: submit_evidence
  Args: evidence_file, metadata
  Returns: { verdict, score, submission_uuid, evidence_hash }

tool: explain_verdict
  Args: submission_uuid
  Returns: { explanation, non_conformances, recommendations }

tool: audit_xes
  Args: file_path
  Returns: { valid, errors, warnings, conformance_percentage }
```

**Implementation Pattern:**
```rust
// src/adapters/wasm4pm_mcp.rs
pub struct Wasm4pmMcpAdapter {
    client: MCP_CLIENT,  // Handle to MCP server connection
    config: Wasm4pmMcpConfig,
}

impl Wasm4pmMcpAdapter {
    pub async fn submit_evidence(&self, evidence_path: &Path) -> Result<WpmVerdict> {
        // Call MCP tool: submit_evidence
        // Parse response
        // Return as WpmVerdict enum
    }
}
```

**Rationale:**
- Eliminates brittle CLI parsing (WpmResult string parsing)
- Enables bidirectional communication (request/response with streaming)
- Provides explicit API contract via MCP schema

---

### 2.2 Rust Documentation Lookup Server

**Name:** `cargo-cicd-rustdoc`  
**Language:** Rust  
**Role:** Query local rustdoc index to resolve crate/item documentation, detect breaking changes, and cross-reference MSRV requirements.

**MCP Resources:**
```
resource: rustdoc://crate/{crate_name}/{version}
  Description: Documentation for a crate version
  Content: HTML, item index, feature flags

resource: rustdoc://item/{crate}/{path::to::Item}
  Description: Specific item documentation
  Content: JSON { signature, docs, examples, stability }

resource: rustdoc://breaking-changes/{crate}/{from}..{to}
  Description: Breaking changes in version range
  Content: JSON array of breaking changes with explanations
```

**MCP Tools:**
```
tool: lookup_item
  Args: crate_name, item_path, version_spec
  Returns: { signature, docs, examples, stability_tier }

tool: check_feature_gate
  Args: item_path, required_feature
  Returns: { feature_name, required_version, docs_link }

tool: find_breaking_changes
  Args: crate_name, from_version, to_version
  Returns: array of { item, change_type, migration_guide }
```

**Integration with cargo-cicd:**
- **Adapter:** `RustdocMcpAdapter` in `src/adapters/rustdoc_mcp.rs`
- **State Dimension:** `TestPlanState` (what tests to run given a version bump)
- **Feature Flag:** `mcp-rustdoc`
- **Config:** `cicd.toml [mcp.rustdoc]`

**Example Configuration:**
```toml
[mcp.rustdoc]
enabled = true
url = "http://localhost:3004"
check_msrv_items = true  # Warn if MSRV crate uses unstable item
check_deprecations = true
```

**Use Case:**
```
When upgrading a transitive dependency from v1.0 → v2.0:
1. cargo-cicd detects the change via workspace MCP server
2. rustdoc MCP server identifies breaking changes
3. Test plan is auto-adjusted to run full suite (not just changed tests)
4. Output highlights items that need code updates
```

**Rationale:**
- Prevents silent failures from breaking changes in deps
- Enables MSRV-aware dependency upgrades
- Surfaces items no longer available on the declared MSRV

---

### 2.3 Policy Suggestion Engine

**Name:** `cargo-cicd-policy-engine`  
**Language:** Rust or Python  
**Role:** AI-powered autonomic policy suggestions (explain "why" a policy fired).

**MCP Resources:**
```
resource: policy://suggestions
  Description: Current policy suggestions in suggest-mode
  Content: JSON array of { policy_name, verdict, reason, confidence }

resource: policy://history/{policy_name}
  Description: Historical verdicts for a policy
  Content: JSON array of { timestamp, verdict, context }
```

**MCP Tools:**
```
tool: explain_policy_verdict
  Args: policy_name, engine_state (JSON)
  Returns: { verdict, reasoning, context_summary, confidence }

tool: suggest_policy_override
  Args: policy_name, desired_verdict
  Returns: { feasibility, side_effects, prerequisites }

tool: simulate_policy_change
  Args: policy_config (TOML), engine_state
  Returns: { verdict, affected_runs, downstream_effects }
```

**Integration with cargo-cicd:**
- **Adapter:** `PolicySuggestionAdapter` in `src/adapters/policy_suggestion_mcp.rs`
- **State Dimension:** `PolicyState` (extend with explanation and confidence)
- **Feature Flag:** `mcp-policy` (implies `autonomic`)
- **Config:** `cicd.toml [mcp.policy]`

**Example Configuration:**
```toml
[mcp.policy]
enabled = true
url = "http://localhost:3005"
engine = "claude-3-5-sonnet"  # Which LLM backend
confidence_threshold = 0.75
explain_all_verdicts = true
```

**Example Interaction:**
```
$ cargo cicd status
Policies:
  • target_pressure: WARN
    size 14.2 GB exceeds limit 20 GB by 71%
    [Explanation from MCP policy engine]:
    "Target directory grew 2GB since last run. Three debug builds are
    present for old branches. Run `cargo clean --release` to reclaim 1.8GB."
  
  • trybuild_changed: PASS
    Only changed fixtures recompiled (12/847)
```

**Rationale:**
- Users understand *why* policies fire, not just *that* they fired
- Confidence scores help distinguish high-signal from noise verdicts
- Enables future A/B testing of policy thresholds

---

## Part 3: Plugin Architecture for MCP-Driven Extensions

### 3.1 Core Plugin Model

cargo-cicd's plugin architecture is **adapter-based**, not callback-based. New behaviors are added as:

1. **New Adapter** implementing a single external source interface
2. **State Dimension** extension (optional new field in `EngineState`)
3. **Policy** consuming the new state dimension
4. **Feature Flag** controlling visibility

### 3.2 Example: Custom Integration Plugin

**Scenario:** A company wants to integrate cargo-cicd with their internal build cache server (Meson or Bazel).

**Plugin Structure:**
```
cargo-cicd-build-cache-plugin/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Plugin entry point
│   ├── adapter.rs                # BuildCacheMcpAdapter
│   ├── state.rs                  # BuildCacheState extension
│   └── policy.rs                 # CacheHealthPolicy
├── mcp-server/
│   ├── main.rs                   # Bundled MCP server binary
│   └── handler.rs                # Tool/resource implementations
└── README.md
```

**Cargo.toml:**
```toml
[package]
name = "cargo-cicd-build-cache-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
cargo-cicd = { version = "26.6.2", features = ["contrib"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"

[features]
default = ["mcp-server"]
mcp-server = []  # Include bundled MCP server

# Enable in workspace alongside cargo-cicd
[workspace]
members = [".."]
```

**Plugin Code (src/lib.rs):**
```rust
use anyhow::Result;
use cargo_cicd::{adapters, engine::EngineState};

/// Plugin registration hook (called by cargo-cicd startup)
pub fn register(engine: &mut EngineState) -> Result<()> {
    // Populate BuildCacheState from MCP server
    let adapter = BuildCacheMcpAdapter::new();
    adapter.populate_engine_state(engine)?;
    Ok(())
}

/// Policy implementation
pub struct CacheHealthPolicy;
impl cargo_cicd::policies::CicdPolicy for CacheHealthPolicy {
    fn name(&self) -> &'static str { "build_cache_health" }
    fn enabled(&self) -> bool { true }
    fn mode(&self) -> PolicyMode { PolicyMode::Suggest }
    fn evaluate(&self) -> PolicyResult { /* ... */ }
}
```

**cicd.toml Integration:**
```toml
[plugins]
build_cache = { url = "file:///path/to/plugin", enabled = true }

[mcp.build_cache]
url = "http://localhost:3006"
cache_dir = "/var/cache/build"
ttl_days = 30
```

### 3.3 Plugin Loading and Lifecycle

**Design:** Two-phase startup
1. **Parse Phase** — Read `cicd.toml [plugins]` and detect available plugins
2. **Initialize Phase** — Load plugins, call `register()`, populate `EngineState`

**Pseudocode:**
```rust
// In main.rs
fn main() -> Result<()> {
    let config = CicdToml::load("cicd.toml")?;
    let mut engine = EngineState::default();
    
    // Phase 1: Built-in adapters
    load_builtin_adapters(&mut engine)?;
    
    // Phase 2: Plugins
    for (plugin_name, plugin_cfg) in &config.plugins {
        let plugin_lib = load_plugin_library(plugin_cfg.url)?;
        plugin_lib.register(&mut engine)?;
        eprintln!("Loaded plugin: {}", plugin_name);
    }
    
    // Phase 3: Run nouns (as before)
    run_noun(&engine)?;
}
```

### 3.4 Plugin Discovery and Distribution

**Registry Approach:**
- Plugins are published to GitHub Releases or crates.io
- `cicd.toml` specifies plugin URL (git, crate, or local path)
- cargo-cicd fetches and links at startup (vendored into binary or loaded as shared object)

**Example Plugin Declarations:**
```toml
# From crates.io
[plugins.my-plugin]
url = "crate:cargo-cicd-my-plugin@0.1.0"

# From GitHub
[plugins.custom-cache]
url = "github:company/cargo-cicd-plugins/build-cache@v0.2.0"

# Local development
[plugins.dev]
url = "file:///home/dev/cargo-cicd-plugins/my-plugin"
```

---

## Part 4: Configuration Format for MCP Dependencies

### 4.1 cicd.toml MCP Section Schema

**Full Schema:**
```toml
# ── MCP Server Configuration ──────────────────────────────────────────────────

[mcp]
# Global MCP settings
enabled = true              # Master kill-switch for all MCP integrations
log_level = "info"         # "debug" | "info" | "warn" | "error"
timeout_secs = 30          # Default timeout for MCP calls
connection_pool_size = 5   # Max concurrent connections per server

# ── GitHub Integration ────────────────────────────────────────────────────────

[mcp.github]
enabled = true
url = "http://localhost:3000"         # MCP server endpoint
repo = "owner/repo"                   # GitHub repo (owner/name)
verify_checks = [
  "cargo-test",
  "cargo-clippy",
  "cargo-fmt"
]
require_approval = true               # Block push if not approved
protect_main = true                   # Extra checks for main branch
cache_ttl_secs = 300                  # Cache PR/branch status

# ── Workspace Introspection ───────────────────────────────────────────────────

[mcp.workspace]
enabled = true
url = "http://localhost:3001"
include_advisories = true             # Check for security advisories
estimate_build_time = true            # Predict compilation time
feature_matrix_limit = 32             # Max combinations to test
dependency_graph_depth = "transitive" # "direct" | "transitive"
cache_ttl_secs = 600                  # Cache dependency graph

# ── Environment Inspection ────────────────────────────────────────────────────

[mcp.environment]
enabled = true
url = "http://localhost:3002"
check_msrv_toolchain = true           # Verify MSRV toolchain installed
predict_build_time = true             # Estimate link time
alert_on_low_disk = true
low_disk_threshold_gb = 5.0
check_ci_environment = true           # Detect CI vs. local
cache_ttl_secs = 300

# ── wasm4pm Oracle ────────────────────────────────────────────────────────────

[mcp.wasm4pm]
enabled = true
url = "http://localhost:3003"
oracle_version = "26.6.2"
require_accept = true                 # Fail if verdict != Accept
fallback_to_shell = true              # Fall back to `wpm` binary if MCP unavailable
evidence_dir = "target/cargo-cicd/evidence"
cache_verdicts = false                # Don't cache oracle decisions (always re-audit)

# ── Rust Documentation Server ─────────────────────────────────────────────────

[mcp.rustdoc]
enabled = false
url = "http://localhost:3004"
check_msrv_items = true               # Warn if item unavailable on MSRV
check_deprecations = true             # Alert on deprecated items
build_local_docs = true               # Generate docs if missing
doc_cache_dir = "target/doc"
cache_ttl_secs = 3600

# ── Policy Suggestion Engine ──────────────────────────────────────────────────

[mcp.policy]
enabled = false
url = "http://localhost:3005"
engine = "claude-3-5-sonnet"          # LLM backend: "claude-*" | "gpt-*" | custom
confidence_threshold = 0.75           # Only suggest if confidence >= threshold
explain_all_verdicts = true           # Include explanations in all policy outputs
context_window_tokens = 4096          # Limit context sent to LLM
cache_explanations = true
cache_ttl_secs = 3600

# ── Plugin System ─────────────────────────────────────────────────────────────

[plugins]
# Plugin URL format: "file://", "crate:", "github:", "http://"
build_cache = {
  url = "crate:cargo-cicd-build-cache@0.1.0",
  enabled = true,
  features = ["mcp-server"]
}

[mcp.build_cache]
enabled = true
url = "http://localhost:3006"
cache_dir = "/var/cache/build"
ttl_days = 30
max_cache_size_gb = 100
```

### 4.2 Schema Validation

**Rust Structure:**
```rust
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpConfig {
    pub enabled: bool,
    pub log_level: String,
    pub timeout_secs: u64,
    pub connection_pool_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpGitHubConfig {
    pub enabled: bool,
    pub url: String,
    pub repo: String,
    pub verify_checks: Vec<String>,
    pub require_approval: bool,
    pub protect_main: bool,
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpWorkspaceConfig {
    pub enabled: bool,
    pub url: String,
    pub include_advisories: bool,
    pub estimate_build_time: bool,
    pub feature_matrix_limit: usize,
    pub dependency_graph_depth: String,
    pub cache_ttl_secs: u64,
}

// ... similar for other configs ...

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CicdToml {
    // ... existing fields ...
    pub mcp: McpConfig,
    pub mcp_github: Option<McpGitHubConfig>,
    pub mcp_workspace: Option<McpWorkspaceConfig>,
    // ... etc ...
    pub plugins: std::collections::HashMap<String, PluginConfig>,
}
```

### 4.3 Environment Variable Overrides

Users can override MCP settings via environment variables following the pattern:

```bash
# Override GitHub MCP server URL
export CARGO_CICD_MCP_GITHUB_URL="http://custom-server:3000"

# Override wasm4pm oracle requirement
export CARGO_CICD_MCP_WASM4PM_REQUIRE_ACCEPT="false"

# Disable all MCP integrations
export CARGO_CICD_MCP_ENABLED="false"

# Set log level globally
export CARGO_CICD_MCP_LOG_LEVEL="debug"
```

**Implementation:**
```rust
fn load_mcp_config() -> Result<McpConfig> {
    let mut cfg = CicdToml::load("cicd.toml")?.mcp;
    
    // Override from environment
    if let Ok(val) = std::env::var("CARGO_CICD_MCP_ENABLED") {
        cfg.enabled = val.parse()?;
    }
    if let Ok(val) = std::env::var("CARGO_CICD_MCP_LOG_LEVEL") {
        cfg.log_level = val;
    }
    // ... etc ...
    
    Ok(cfg)
}
```

---

## Part 5: Implementation Roadmap

### Phase 1: Foundation (v26.6.3)
- [ ] Add `[mcp]` section to cicd.toml schema
- [ ] Create `McpConfig` structs and validation
- [ ] Implement MCP server detection/connection logic
- [ ] Add `mcp-github` feature flag
- [ ] Implement `GitHubMcpAdapter` (basic: branch status, CI checks)
- [ ] Update existing adapters to handle MCP unavailability gracefully

**Acceptance Criteria:**
- `cargo test` passes with all MCP servers unavailable
- With MCP server running, GitHub branch status is read and integrated into `GitPhaseState`
- Feature flag gates prevent compile-time bloat when not needed

### Phase 2: Workspace & Environment (v26.6.4)
- [ ] Implement `WorkspaceMcpAdapter` (dependency graph, advisories)
- [ ] Implement `EnvironmentMcpAdapter` (toolchain, resources)
- [ ] Extend `TestPlanState` to account for affected crates
- [ ] Add `mcp-workspace` and `mcp-environment` feature flags

**Acceptance Criteria:**
- Affected-crate analysis reduces test time by 30%+ on large monorepos
- Security advisory detection blocks known-vulnerable transitive deps
- MSRV toolchain detection prevents compilation against wrong version

### Phase 3: Advanced Integrations (v26.6.5)
- [ ] Implement `Wasm4pmMcpAdapter` (replaces shell-out logic)
- [ ] Implement `RustdocMcpAdapter` (breaking change detection)
- [ ] Add `mcp-wasm4pm` and `mcp-rustdoc` feature flags
- [ ] Create test suite for breaking change scenarios

**Acceptance Criteria:**
- wasm4pm verdict is obtained via MCP instead of shell-out
- Breaking changes in transitive deps are surfaced before build failure
- MSRV conformance is checked against actual Rust docs

### Phase 4: Plugin System & Policy Engine (v26.6.6)
- [ ] Implement plugin loader in main.rs
- [ ] Add plugin section to cicd.toml schema
- [ ] Implement `PolicySuggestionAdapter` (LLM-powered explanations)
- [ ] Create plugin template and documentation
- [ ] Add `mcp-policy` feature flag

**Acceptance Criteria:**
- Third-party plugins can register custom adapters and policies
- Policy verdicts include confidence scores and explanations
- Plugin lifecycle tests pass (load, initialize, cleanup)

### Phase 5: Documentation & Ecosystem (v26.6.7)
- [ ] Write MCP server implementation guide
- [ ] Publish reference implementations (GitHub, workspace, env, rustdoc)
- [ ] Create plugin development tutorial
- [ ] Add MCP server health check command (`cargo cicd mcp status`)

---

## Part 6: Rationale for Key Design Decisions

### Decision 1: Why Adapters, Not Native Rust Libraries?

**Alternative:** Link to Rust libraries directly (e.g., `octocat` for GitHub API).

**Decision:** Use MCP servers instead.

**Rationale:**
1. **Language Agnostic:** MCP servers can be written in Python, JavaScript, or Go. Cargo-cicd users may want to write policies in languages other than Rust.
2. **Process Isolation:** Each MCP server runs in its own process. A crash in the GitHub server doesn't crash cargo-cicd.
3. **Versioning Independence:** Users can update the GitHub MCP server without rebuilding cargo-cicd.
4. **Graceful Degradation:** If the server is down, cargo-cicd continues with reduced capability (MCP unavailable pattern).
5. **Feature Compliance:** MCP is explicitly designed for tool use and resource sharing — the correct abstraction for external integrations.

**Trade-off:** Network overhead. MCP is JSON-RPC over HTTP; it's slower than in-process calls. Acceptable because:
- MCP calls are infrequent (CI/CD workflow, not hot path)
- Results are cached in `cicd.toml` and EngineState
- Timeout is configurable (users can set `timeout_secs = 5` for fast-fail)

### Decision 2: Why Feature Flags for MCP Servers?

**Alternative:** Always compile MCP adapters; users just leave them disabled in config.

**Decision:** Gate them behind feature flags.

**Rationale:**
1. **Binary Size:** Minimal cargo-cicd (default features) should be < 10 MB. Each MCP adapter adds dependencies (serde, http, JSON libs). Feature flags allow minimal installs.
2. **Supply Chain:** Users who don't need GitHub integration should not have the GitHub MCP adapter code in their binary (reduced attack surface).
3. **Compile Time:** Feature flags mean the adapter code isn't compiled unless needed.
4. **Documentation:** Feature flags are discoverable in `cargo cicd --help` and `Cargo.toml`.

**Trade-off:** Users must explicitly enable features. This is a one-time cost when installing cargo-cicd.

### Decision 3: Why Store MCP Config in cicd.toml (Not a Separate File)?

**Alternative:** Create `cicd-mcp.toml` or `.cargo-cicd-mcp.json`.

**Decision:** Extend the existing `cicd.toml` file.

**Rationale:**
1. **Single Source of Truth:** All cargo-cicd config (workspace, target, test, git, autonomic, **mcp**, plugins) is in one place.
2. **Gitignore:** `cicd.toml` is already in `.gitignore` (it's machine-written state). MCP secrets (if any) are naturally excluded.
3. **Portability:** Users moving projects don't need to track multiple config files.
4. **Schema Consistency:** cicd.toml is already validated and documented. Adding MCP sections reuses that validation.

**Trade-off:** cicd.toml becomes larger. Mitigated by good documentation and optional sections.

### Decision 4: Why Graceful Degradation (Not Hard Fail)?

**Alternative:** If an MCP server is unavailable, cargo-cicd fails immediately.

**Decision:** Return `WpmVerdict::Partial` (or equivalent) and continue.

**Rationale:**
1. **Local-First Philosophy:** cargo-cicd's core selling point is "works without network." Hard fails on missing MCP violate that.
2. **CI Resilience:** In CI, external services occasionally fail. cargo-cicd should be resilient.
3. **User Experience:** Users should be able to run cargo-cicd on planes, in offline environments, or when their MCP server is being updated.

**Trade-off:** Some features are silent no-ops when MCP is unavailable. Mitigated by:
- Log messages indicating which MCP servers were contacted and their status
- `cargo cicd mcp status` command shows health of all configured MCP servers
- Policy verdicts include "partial" status when MCP data was missing

### Decision 5: Why Policy Engine as Separate MCP Server (Not Built-In)?

**Alternative:** Add `explain_policy_verdict()` method directly to policies in cargo-cicd.

**Decision:** Move policy explanation to a separate MCP server.

**Rationale:**
1. **LLM Dependency:** Policy explanation requires an LLM (Claude, GPT, etc.). Coupling that to cargo-cicd adds heavyweight dependencies and increases binary size.
2. **Cost & Privacy:** LLM API calls cost money. Users should opt in explicitly and control which backend is used.
3. **Modularity:** Policy explanation is optional. Users can run cargo-cicd with or without explanations.
4. **Future-Proof:** New explanation backends (different LLMs, rule-based engines) can be swapped without recompiling cargo-cicd.

**Trade-off:** Users must run an extra MCP server for explanations. This is acceptable because:
- Policy explanations are nice-to-have, not essential
- The MCP server can be a lightweight Python script or Bash wrapper
- Explanations are cached in `cicd.toml`, reducing repeated calls

---

## Part 7: Security Considerations

### 7.1 MCP Server Trust Model

**Assumption:** All MCP servers are trusted (run on localhost or internal network).

**Rationale:** 
- MCP servers execute tools on your machine (shell commands, file access).
- Untrusted MCP servers are equivalent to code execution vulnerabilities.
- cargo-cicd assumes you trust your local tooling environment.

**Mitigation:**
1. MCP servers must be explicitly enabled and configured in `cicd.toml`.
2. Only connect to MCP servers on localhost or internal IPs (enforce in config schema).
3. Document that MCP servers should be run in isolated containers if sourced from untrusted origins.

### 7.2 Credential Handling

**Pattern:** MCP servers handle credentials; cargo-cicd never stores them.

**Example (GitHub):**
```toml
[mcp.github]
url = "http://localhost:3000"
# No credentials in cicd.toml
```

The GitHub MCP server handles auth (reads `~/.ssh/`, `GITHUB_TOKEN` env var, etc.).

**Rationale:** 
- Credentials are tool-specific; cargo-cicd shouldn't manage them.
- Users are comfortable with MCP servers having credentials (they already do for their IDEs, git clients, etc.).

### 7.3 Evidence Auditing

**Pattern:** All MCP calls are logged in `cicd.toml [[events]]`.

```toml
[[events]]
kind = "mcp_call"
server = "github"
tool = "get_pr_metadata"
timestamp = "2026-06-14T10:30:00Z"
result = "success"
cached = false
```

**Rationale:** 
- Users can audit which MCP servers were consulted.
- Supports debugging and compliance investigations.
- Evidence persists even if MCP server is later taken offline.

---

## Part 8: Testing Strategy for MCP Integrations

### 8.1 Mock MCP Servers for Testing

**Pattern:** Fixture MCP servers in `tests/mcp_servers/`.

```rust
// tests/mcp_servers/mock_github.rs
pub struct MockGitHubServer {
    port: u16,
    responses: HashMap<String, String>,
}

impl MockGitHubServer {
    pub fn new(port: u16) -> Self { /* ... */ }
    pub fn with_pr_status(mut self, pr: u32, status: String) -> Self { /* ... */ }
    pub fn spawn(self) -> tokio::task::JoinHandle<()> { /* spawn HTTP server */ }
}

#[test]
fn test_github_mcp_adapter_with_passing_checks() {
    let server = MockGitHubServer::new(3000)
        .with_pr_status(123, "success");
    let _handle = server.spawn();
    
    let adapter = GitHubMcpAdapter::new("http://localhost:3000");
    let verdict = adapter.check_branch_status("main").unwrap();
    assert_eq!(verdict, GitPhaseState { ci_checks: true, .. });
}
```

### 8.2 Graceful Degradation Tests

```rust
#[test]
fn test_mcp_unavailable_returns_partial() {
    // Don't start MCP server
    let adapter = GitHubMcpAdapter::new("http://localhost:3000");
    let result = adapter.check_branch_status("main");
    
    assert!(result.is_ok()); // Doesn't panic
    let verdict = result.unwrap();
    assert_eq!(verdict.status, WpmVerdict::Partial);
}
```

### 8.3 Feature Flag Tests

```rust
#[test]
#[cfg(feature = "mcp-github")]
fn test_github_mcp_adapter_compiles() {
    let _adapter = GitHubMcpAdapter::new("http://localhost:3000");
}

#[test]
#[cfg(not(feature = "mcp-github"))]
fn test_github_mcp_adapter_not_compiled_without_feature() {
    // Verify the adapter is not in the binary
    // (This is compile-time verification via compiler)
}
```

---

## Part 9: Backward Compatibility & Migration

### 9.1 Existing cicd.toml Files

**Current v26.6.2 format** has no MCP section. **New v26.6.3** parses MCP section as optional.

**Migration:**
```rust
impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // MCP is opt-in
            // ... all disabled by default ...
        }
    }
}

// When parsing cicd.toml, if [mcp] is absent, use defaults
let mcp_cfg = toml_value.get("mcp")
    .map(|v| serde_json::from_value(v))
    .transpose()?
    .unwrap_or_default();
```

**Result:** Existing `cicd.toml` files work unchanged; MCP is disabled.

### 9.2 Adapter API Stability

Adapters implement a stable trait:
```rust
pub trait ExternalAdapter {
    fn populate_engine_state(&self, state: &mut EngineState) -> Result<()>;
}
```

This trait is guaranteed stable across cargo-cicd versions (under semver). MCP adapters implement it alongside existing adapters.

---

## Part 10: Example: End-to-End Flow

**Scenario:** User pushes a commit; cargo-cicd checks if it's safe.

```
1. User: $ cargo cicd status --check-push

2. cargo-cicd startup:
   - Load cicd.toml, parse [mcp.*] sections
   - Detect enabled MCP servers, create adapters
   - EngineState initialized (empty)

3. Populate EngineState:
   - GitStatusAdapter: reads git status (local)
   - GitHubMcpAdapter: checks GitHub PR status (MCP → GitHub server)
   - EnvironmentMcpAdapter: checks system resources (MCP → env server)
   - WorkspaceMcpAdapter: checks for known CVEs (MCP → workspace server)
   - Wasm4pmMcpAdapter: submits XES evidence (MCP → wasm4pm server)

4. Policy evaluation:
   - GitPhaseDirtyPolicy: checks EngineState.git_phase.dirty
     Verdict: PASS (working tree clean)
   
   - TargetPressurePolicy: checks EngineState.target.size_gb
     Verdict: PASS (2.5 GB < 20 GB limit)
   
   - CicdPolicyEngine (MCP): calls policy suggestion server
     Verdict: PASS with explanation
     "No issues detected. All checks passing. Safe to push."

5. Output to user:
   ✓ git.phase: PASS — clean tree
   ✓ target.pressure: PASS — 2.5 GB (under limit)
   ✓ github.checks: PASS — CI passing (from GitHub MCP)
   ✓ security.advisories: PASS — no known CVEs (from workspace MCP)
   ✓ autonomic.policy: PASS — all policies green
   
   ✅ Safe to push!

6. Log to cicd.toml:
   [[events]]
   kind = "status"
   verdict = "pass"
   timestamp = "2026-06-14T10:30:00Z"
   mcp_servers_contacted = ["github", "environment", "workspace", "wasm4pm", "policy"]
   
```

---

## Conclusion

This MCP integration strategy transforms cargo-cicd from a local-only tool into a networked orchestrator while maintaining:

- **Local-first operation** (MCP is gracefully optional)
- **Adapter-based architecture** (no business logic in IO)
- **Feature-gated capability** (users control integration footprint)
- **Evidence-driven decisions** (all external input is auditable)

The four integration points (external MCP servers, custom MCP servers, plugin system, cicd.toml config) form a complete ecosystem for extending cargo-cicd to meet diverse CI/CD needs across organizations.

---

## Appendix A: MCP Server Reference Implementations

### GitHub MCP Server
**Repository:** `https://github.com/seanchatmangpt/mcp-github`  
**Language:** Rust  
**Dependencies:** `octocat` crate for GitHub API  
**Entry Point:** `./target/release/mcp-github --port 3000`

### Workspace MCP Server
**Repository:** `https://github.com/seanchatmangpt/mcp-workspace`  
**Language:** Rust  
**Dependencies:** `cargo_metadata`, `advisory-db`  
**Entry Point:** `./target/release/mcp-workspace --port 3001`

### Environment MCP Server
**Repository:** `https://github.com/seanchatmangpt/mcp-environment`  
**Language:** Rust or Bash  
**Dependencies:** `sysinfo` crate, system utilities  
**Entry Point:** `./target/release/mcp-environment --port 3002`

### wasm4pm MCP Server
**Repository:** `https://github.com/seanchatmangpt/mcp-wasm4pm`  
**Language:** Rust  
**Dependencies:** `wasm4pm` binary on PATH  
**Entry Point:** `./target/release/mcp-wasm4pm-oracle --port 3003`

---

**Document Author:** Claude (claude.ai/code)  
**Last Updated:** 2026-06-14  
**Next Review:** 2026-08-14
