# MCP Server Integration Strategy for cargo-cicd

**Version:** 1.0  
**Date:** 2026-06-14  
**Status:** Architecture Design  

---

## Executive Summary

This document outlines a comprehensive strategy for integrating Model Context Protocol (MCP) servers with cargo-cicd. The strategy balances extending cargo-cicd's capabilities through external services (GitHub, workspace inspection, environment analysis) while defining a plugin architecture for custom domain-specific servers (wasm4pm oracle, Rust documentation).

**Key Principles:**
- **Separation of Concerns:** MCP servers handle external APIs and system inspection; adapters translate into EngineState
- **Layered Dependency Management:** Declare MCP dependencies in `cicd.toml` with capability discovery and fallback behavior
- **Feature-Flag Gating:** MCP integrations align with existing `process-data`, `autonomic`, and `contrib` feature flags
- **Evidence Preservation:** All MCP interactions are logged as process events in XES format for wasm4pm adjudication
- **Composability:** Policies and nouns remain agnostic to MCP backend; adapters abstract the integration layer

---

## Part 1: External MCP Servers Integration

### 1.1 Recommended External MCP Servers

#### A. GitHub MCP Server (Primary Integration)

**Purpose:** Unify git state inspection, pull request history, commit metadata, and release management into a single server.

**Why not inline `cargo-cicd git` command:**
- PR history and CI status require API calls; shelling to `git` and `gh` is incomplete
- wasm4pm process evidence requires commit graph lineage
- Autonomic policies need to correlate git state with remote history

**Capabilities:**
```
- List commits with metadata (author, timestamp, message, tree hash)
- Query current branch, remote tracking, ahead/behind
- Fetch open PRs and review status
- Query release tags and changelog artifacts
- List workflow runs and check statuses
- Detect protected branches and merge strategies
```

**Integration Pattern:**

```rust
// src/adapters/mcp/github_mcp.rs
pub struct GitHubMcpAdapter {
    mcp_server: McpClient,
    repo_owner: String,
    repo_name: String,
}

impl GitHubMcpAdapter {
    pub fn new(repo_owner: String, repo_name: String) -> Self { /* */ }
    
    /// Fetch commits since base_ref for changed test detection
    pub fn commits_since(&self, base_ref: &str) -> Result<Vec<CommitMetadata>> { /* */ }
    
    /// Query PR metadata for current branch
    pub fn get_pr_for_branch(&self, branch: &str) -> Result<Option<PullRequestMetadata>> { /* */ }
    
    /// Fetch latest release tag
    pub fn get_latest_release(&self) -> Result<ReleaseMetadata> { /* */ }
}

pub struct CommitMetadata {
    pub hash: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub files_changed: Vec<String>,
}

pub struct PullRequestMetadata {
    pub number: u32,
    pub title: String,
    pub state: String, // "open" | "closed" | "merged"
    pub reviews: Vec<ReviewMetadata>,
    pub checks: Vec<CheckRunMetadata>,
}
```

**Why:** Current git state detection is local-only; GitHub MCP bridges to remote repository state needed for:
- Autonomic `changed_tests` comparison against remote base
- Publish gate checking for approved PRs before release
- Evidence correlation with CI workflows

**Rationale:** GitHub-hosted Rust projects are the primary use case; centralized API access via MCP eliminates subprocess shell fragility.

---

#### B. Workspace/File System MCP Server

**Purpose:** Provide capability-aware file system inspection beyond `walkdir`, with caching and change detection.

**Capabilities:**
```
- List files matching patterns with size/mtime metadata
- Query directory tree structure
- Detect workspace member boundaries
- Calculate hash digests for change tracking
- Filter by file extension/glob patterns efficiently
```

**Integration Pattern:**

```rust
// src/adapters/mcp/fs_mcp.rs
pub struct FsMcpAdapter {
    mcp_server: McpClient,
}

impl FsMcpAdapter {
    /// List all test files in workspace
    pub fn find_tests(&self, workspace_root: &Path) -> Result<Vec<TestFileInfo>> { /* */ }
    
    /// Get workspace members (Cargo.toml locations)
    pub fn workspace_members(&self) -> Result<Vec<WorkspaceMember>> { /* */ }
    
    /// Change detection: hash files modified since last run
    pub fn changed_files_hash(&self, since: DateTime<Utc>) -> Result<Vec<FileChangeRecord>> { /* */ }
}

pub struct TestFileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: DateTime<Utc>,
    pub test_functions: Vec<String>,
}

pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub manifest: CargoTomlMetadata,
}
```

**Why:** `tests/changed_tests.rs` currently walks the filesystem; MCP server can cache and use file system watchers for faster incremental updates.

**Rationale:** Enables future Claude Code integration where the IDE's file watcher feeds change events directly to cargo-cicd.

---

#### C. Environment Inspection MCP Server

**Purpose:** Safely query system environment, toolchain versions, and capability availability without spawning subprocesses.

**Capabilities:**
```
- Query rustup toolchain list with versions
- Check for installed tools (cargo-make, cargo-nextest, clippy, miri)
- Get environment variable values (filtered for security)
- Query available Rust targets
- Detect system-level dependencies (protoc, llvm-tools)
```

**Integration Pattern:**

```rust
// src/adapters/mcp/env_mcp.rs
pub struct EnvironmentMcpAdapter {
    mcp_server: McpClient,
}

impl EnvironmentMcpAdapter {
    /// Get rustup active toolchain and available options
    pub fn toolchain_info(&self) -> Result<ToolchainInfo> { /* */ }
    
    /// Check if a cargo subcommand is available
    pub fn has_cargo_tool(&self, tool: &str) -> Result<bool> { /* */ }
    
    /// List available Rust targets
    pub fn rust_targets(&self) -> Result<Vec<String>> { /* */ }
}

pub struct ToolchainInfo {
    pub active: String,
    pub available: Vec<String>,
    pub rustc_version: String,
    pub cargo_version: String,
    pub has_make: bool,
    pub has_nextest: bool,
    pub has_clippy: bool,
    pub has_miri: bool,
}
```

**Why:** Avoid subprocess spawning overhead; centralized capability detection enables offline policy decisions.

**Rationale:** Supports autonomic policies that conditionally recommend tool installation or fallback strategies.

---

### 1.2 MCP Server Discovery & Lifecycle Management

**Problem:** How does cargo-cicd know which MCP servers are available, and what should it do if they're unavailable?

**Solution: Capability Discovery via `MCPRegistry`**

```rust
// src/integrations/mcp_registry.rs
pub struct MCPRegistry {
    /// Map of server_id -> ServerCapability
    servers: HashMap<String, ServerCapability>,
    /// Map of feature_flag -> required_servers
    feature_requirements: HashMap<String, Vec<String>>,
}

pub struct ServerCapability {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    /// null = optional, true = required by a feature flag
    pub required: Option<String>,
    /// timeout in ms before fallback
    pub timeout_ms: u32,
    pub capabilities: Vec<String>, // ["list_commits", "get_pr", ...]
    pub fallback: FallbackStrategy,
}

pub enum FallbackStrategy {
    /// Use subprocess fallback (git, gh commands)
    Subprocess,
    /// Disable the feature gracefully
    Disable,
    /// Fail hard (for critical features like wasm4pm)
    Fail,
}

impl MCPRegistry {
    pub fn new() -> Self { /* */ }
    
    /// Load server capabilities from cicd.toml [mcp] section
    pub fn from_cicd_toml(config: &CicdToml) -> Result<Self> { /* */ }
    
    /// Probe all servers; update availability state
    pub fn probe(&mut self) -> HashMap<String, bool> { /* */ }
    
    /// Get a server handle; returns None if unavailable and fallback=Disable
    pub fn get_server(&self, id: &str) -> Result<Option<McpClient>> { /* */ }
    
    /// Validate that all required servers for current feature flags are available
    pub fn validate_feature_requirements(&self) -> Result<()> { /* */ }
}
```

**Initialization Flow:**

```rust
// In main.rs or session initialization
fn initialize_mcp_registry(cicd_config: &CicdToml) -> Result<MCPRegistry> {
    let mut registry = MCPRegistry::from_cicd_toml(cicd_config)?;
    let availability = registry.probe();
    
    // Log availability
    for (server_id, available) in availability {
        if available {
            eprintln!("✓ MCP server {} is available", server_id);
        } else {
            eprintln!("⚠ MCP server {} is unavailable", server_id);
        }
    }
    
    // Validate feature requirements (fail fast if autonomic or wasm4pm require unavailable servers)
    registry.validate_feature_requirements()?;
    
    Ok(registry)
}
```

---

### 1.3 Feature Flag Integration

**Mapping MCP dependencies to feature flags:**

| Feature Flag | Required MCP Servers | Fallback |
|--------------|---------------------|----------|
| `default` | None | Subprocess commands |
| `autonomic` | `github` (for remote base detection) | Subprocess `gh`, `git` |
| `process-data` | `environment` (for capability detection) | None — assume `cargo-make` available |
| `wasm4pm` | `environment`, `github` | Fail — evidence-gate requires remote history |
| `contrib` | All servers | Subprocess fallbacks, reduced capabilities |

**Feature-gating in code:**

```rust
#[cfg(feature = "autonomic")]
pub async fn run_changed_tests(&self) -> Result<TestPlanState> {
    let github = self.mcp_registry.get_server("github")?
        .ok_or_else(|| anyhow::anyhow!("GitHub MCP required for autonomic mode"))?;
    
    let remote_commits = github.commits_since(&self.base_ref).await?;
    // ... compare against local commits
}

#[cfg(not(feature = "autonomic"))]
pub async fn run_changed_tests(&self) -> Result<TestPlanState> {
    // Fallback: use subprocess git commands
    let local_commits = self.git_subprocess_commits().await?;
    // ... limited change detection
}
```

---

## Part 2: Custom Domain-Specific MCP Servers

### 2.1 wasm4pm Oracle MCP Server

**Purpose:** Provide a local MCP interface to the wasm4pm evidence adjudication oracle, enabling tight integration with the Level 5 process-data engine.

**Why separate from embedded integration:**
- wasm4pm binary is external system dependency
- Process evidence format (XES) is standardized; benefits from versioning MCP contract
- Policies and verbs should not import wasm4pm internals
- Evidence-gate testing needs pluggable oracle for mutation testing

**Capabilities:**

```
- validate_xes(path) → ComplianceResult
- process_event_list(events) → XesDocument
- audit(xes_path) → {verdict, reasoning, violations}
- doctor_receipt(receipt_json) → {status, errors}
- suggest_repair(receipt_json, violation) → SuggestedRepair
```

**Server Definition:**

```rust
// src/integrations/mcp/wasm4pm_server.rs
pub struct Wasm4pmMcpServer {
    binary_path: PathBuf,
    cache_dir: PathBuf,
}

impl Wasm4pmMcpServer {
    pub fn new(binary_path: PathBuf) -> Self { /* */ }
    
    /// Shell to `wpm audit <file.xes>` and parse JSON result
    pub async fn audit(&self, xes_path: &Path) -> Result<AuditResult> {
        let output = Command::new(&self.binary_path)
            .args(["audit", xes_path.to_str().unwrap()])
            .output()?;
        
        let result: AuditResult = serde_json::from_slice(&output.stdout)?;
        Ok(result)
    }
    
    /// Validate receipt against wasm4pm doctor
    pub async fn doctor_receipt(&self, receipt: &Receipt) -> Result<DoctorResult> {
        let receipt_json = serde_json::to_string(&receipt)?;
        let output = Command::new(&self.binary_path)
            .args(["receipt", "doctor", "--format", "json", "--strict", "-"])
            .stdin(Stdio::piped())
            .output()?;
        
        let result: DoctorResult = serde_json::from_slice(&output.stdout)?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditResult {
    pub verdict: String, // "Accept" | "Refuse"
    pub reasoning: String,
    pub violations: Vec<String>,
    pub process_metrics: ProcessMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorResult {
    pub status: String, // "healthy" | "violated"
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

**MCP Server Registration:**

```toml
# In cicd.toml [mcp.servers]
[[mcp.servers]]
id = "wasm4pm"
name = "wasm4pm Oracle"
type = "custom"
endpoint = "local://wasm4pm"
required_by = ["wasm4pm"] # feature flag
timeout_ms = 30000
fallback = "fail"

[mcp.servers.config]
binary_path = "/usr/local/bin/wpm"
version_requirement = ">= 5.0.0"
```

---

### 2.2 Rust Documentation & Type-Info MCP Server

**Purpose:** Enable autonomic policies and interactive help to reference Rust standard library, common crates, and type information without embedding large data.

**Rationale:**
- Policies like `ToolchainMismatchPolicy` could recommend MSRV from crate metadata
- `cargo cicd help target` could reference linker flags from `rustc --help`
- Custom trybuild errors could be cross-referenced with crate docs

**Capabilities:**

```
- lookup_crate(name, version) → CrateMetadata
- search_std_lib(query) → Vec<StdLibItem>
- get_rustc_capabilities(toolchain) → RustcInfo
- resolve_error(compiler_error) → ErrorExplanation
```

**Server Definition:**

```rust
// src/integrations/mcp/rust_docs_server.rs
pub struct RustDocsServer {
    cache_dir: PathBuf,
    docs_index: DocsIndex,
}

impl RustDocsServer {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        let docs_index = DocsIndex::load_or_fetch(&cache_dir)?;
        Ok(Self { cache_dir, docs_index })
    }
    
    /// Query crate metadata from crates.io
    pub async fn lookup_crate(&self, name: &str, version: &str) -> Result<CrateMetadata> { /* */ }
    
    /// Fuzzy search standard library
    pub async fn search_std_lib(&self, query: &str) -> Result<Vec<StdLibItem>> { /* */ }
    
    /// Get rustc capabilities for a specific toolchain
    pub async fn rustc_info(&self, toolchain: &str) -> Result<RustcInfo> { /* */ }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateMetadata {
    pub name: String,
    pub version: String,
    pub msrv: Option<String>,
    pub features: Vec<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustcInfo {
    pub version: String,
    pub commit_hash: String,
    pub commit_date: String,
    pub target_triple: String,
    pub sysroot: PathBuf,
    pub available_features: Vec<String>,
}
```

---

### 2.3 Test Coverage & Mutation Testing MCP Server

**Purpose:** Provide instrumentation and coverage analysis for `changed_tests` and autonomic recommendations.

**Capabilities:**

```
- instrument_test(test_name, source_path) → InstrumentedCode
- analyze_coverage(test_run) → CoverageReport
- suggest_missing_cases(function, coverage) → Vec<TestCase>
- estimate_test_runtime(test_suite) → RuntimeMetrics
```

**Why MCP:**
- Integration with `cargo tarpaulin`, `llvm-cov`, or `cargo-mutants`
- Decouples cargo-cicd from coverage tool ecosystem churn
- Enables policy recommendations: "changed_tests_incomplete: add X more cases"

---

## Part 3: Plugin Architecture for Extending cargo-cicd

### 3.1 Policy Plugin System

**Problem:** Current policies are hardcoded; users can't add custom autonomic rules.

**Solution: Plugin Registry with Dynamic Loading**

```rust
// src/plugins/policy_plugin.rs
pub trait PolicyPlugin: Send + Sync {
    /// Unique plugin ID (e.g., "my-org.custom-lint-policy")
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Required feature flags (e.g., ["autonomic"])
    fn requires_features(&self) -> Vec<&str>;
    
    /// Read state from EngineState and produce a PolicyResult
    fn evaluate(&self, state: &EngineState) -> anyhow::Result<PolicyResult>;
    
    /// User-facing recommendation text
    fn recommendation_template(&self) -> &str;
}

pub struct PolicyPluginRegistry {
    plugins: HashMap<String, Box<dyn PolicyPlugin>>,
}

impl PolicyPluginRegistry {
    pub fn new() -> Self { /* */ }
    
    /// Load plugins from shared library or WASM module
    pub fn load_plugin(&mut self, id: &str, path: &Path) -> Result<()> { /* */ }
    
    /// Evaluate all registered policies
    pub fn evaluate_all(&self, state: &EngineState) -> Result<Vec<PolicyResult>> { /* */ }
}
```

**Plugin Loading Strategies:**

#### A. Shared Library (.so/.dylib/.dll)

```rust
// User provides a compiled shared library
// cargo-cicd uses libloading to dynamically load it
pub fn load_shared_library_plugin(path: &Path) -> Result<Box<dyn PolicyPlugin>> {
    unsafe {
        let library = libloading::Library::new(path)?;
        let constructor: libloading::Symbol<unsafe extern "C" fn() -> *mut dyn PolicyPlugin> =
            library.get(b"create_plugin")?;
        Ok(Box::from_raw(constructor()))
    }
}

// User's plugin crate (external to cargo-cicd)
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn PolicyPlugin {
    Box::into_raw(Box::new(MyCustomPolicy))
}
```

**Plugin Crate Template:**

```toml
# external-policy/Cargo.toml
[package]
name = "my-custom-policy"
version = "0.1.0"
crate-type = ["cdylib"]

[dependencies]
cargo-cicd = "26.6.2"  # For trait definitions
serde = "1"
anyhow = "1"
```

```rust
// external-policy/src/lib.rs
use cargo_cicd::plugins::PolicyPlugin;
use cargo_cicd::state::EngineState;

pub struct MyCustomPolicy;

impl PolicyPlugin for MyCustomPolicy {
    fn id(&self) -> &str { "my-org.custom-policy" }
    fn name(&self) -> &str { "Custom Lint Policy" }
    fn requires_features(&self) -> Vec<&str> { vec!["autonomic"] }
    
    fn evaluate(&self, state: &EngineState) -> anyhow::Result<PolicyResult> {
        // Custom logic using state
        Ok(PolicyResult {
            name: "custom-lint".into(),
            verdict: "pass".into(),
            recommendation: None,
            /* ... */
        })
    }
    
    fn recommendation_template(&self) -> &str {
        "Custom linting found issues: {details}"
    }
}
```

#### B. WASM Modules

```rust
// src/plugins/wasm_plugin.rs
pub struct WasmPolicyPlugin {
    module: wasmtime::Module,
    instance: wasmtime::Instance,
}

impl WasmPolicyPlugin {
    pub fn load(path: &Path) -> Result<Self> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::from_file(&engine, path)?;
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;
        Ok(Self { module, instance })
    }
    
    /// Call the WASM plugin's evaluate() function
    pub fn evaluate(&self, state_json: &str) -> Result<PolicyResult> { /* */ }
}
```

**Why WASM:** Language-agnostic; users can write plugins in Go, C, or Rust and compile to WASM.

#### C. Configuration-Driven Policies

```toml
# cicd.toml [policies.custom]
[[policies.custom]]
id = "my-workspace.require-docs"
enabled = true
mode = "suggest"
rule = "if changed_files contains *.rs and not contains *.md, suggest doc update"
severity = "warn"
```

---

### 3.2 Adapter Plugin System

**Problem:** Not all external sources fit the built-in adapters (git, cargo, filesystem). Users might need to integrate with:
- Custom CI systems (internal Jenkins, BuildKite)
- Project metadata from custom tools
- License / compliance checkers

**Solution: Adapter Registration**

```rust
// src/adapters/plugin_adapter.rs
pub trait AdapterPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    
    /// Populate part of EngineState (e.g., CustomStateField)
    fn populate(&self, state: &mut EngineState) -> Result<()>;
    
    /// Whether this adapter is available (e.g., tool installed)
    fn is_available(&self) -> bool;
}

pub struct AdapterPluginRegistry {
    adapters: HashMap<String, Box<dyn AdapterPlugin>>,
}

impl AdapterPluginRegistry {
    pub fn register(&mut self, adapter: Box<dyn AdapterPlugin>) {
        self.adapters.insert(adapter.id().to_string(), adapter);
    }
    
    pub fn run_all(&self, state: &mut EngineState) -> Result<()> {
        for adapter in self.adapters.values() {
            if adapter.is_available() {
                adapter.populate(state)?;
            }
        }
        Ok(())
    }
}
```

**Usage in main:**

```rust
fn main() -> Result<()> {
    let mut state = EngineState::default();
    
    // Built-in adapters
    GitStatusAdapter::populate(&mut state)?;
    CargoMetadataAdapter::populate(&mut state)?;
    
    // Plugin adapters
    let plugin_registry = AdapterPluginRegistry::from_cicd_toml(&config)?;
    plugin_registry.run_all(&mut state)?;
    
    // ... rest of CLI
}
```

---

## Part 4: Configuration Format for MCP Dependencies in cicd.toml

### 4.1 Schema Definition

```toml
# cicd.toml
[workspace]
name = "my-workspace"
toolchain = "stable"
target_dir = "target"

# ──────────────────────────────────────────────────────────────
# MCP Server Configuration
# ──────────────────────────────────────────────────────────────

[mcp]
enabled = true
# Global timeout override (ms)
default_timeout_ms = 5000
# Fail-fast on probe: if true, cargo cicd exits if any required server is unavailable
fail_fast = false

## External MCP Servers

[[mcp.servers]]
id = "github"
name = "GitHub API"
type = "external"
endpoint = "https://mcp.github.com"
# If null: server is optional. If string: name of feature flag that requires it.
required_by = "autonomic"
timeout_ms = 10000
fallback = "subprocess"  # subprocess | disable | fail

[mcp.servers.config]
# Server-specific config (transparent to cargo-cicd)
owner = "seanchatmangpt"
repo = "cargo-cicd"
token = "env:GITHUB_TOKEN"  # Reference to env var

[[mcp.servers]]
id = "workspace-fs"
name = "File System Server"
type = "external"
endpoint = "stdio://workspace-fs"  # stdio | http | unix_socket | tcp
required_by = "process-data"
timeout_ms = 3000
fallback = "disable"

[mcp.servers.config]
cache_dir = ".cargo-cicd-cache"
watch_enabled = true

[[mcp.servers]]
id = "environment"
name = "Environment Inspector"
type = "external"
endpoint = "http://localhost:3000"
required_by = null
timeout_ms = 2000
fallback = "subprocess"

[mcp.servers.config]
# Restrict which env vars are readable
env_allowlist = ["RUST_BACKTRACE", "CARGO_BUILD_JOBS", "RUSTFLAGS"]

## Custom MCP Servers

[[mcp.servers]]
id = "wasm4pm-oracle"
name = "wasm4pm Evidence Adjudication"
type = "custom"
endpoint = "subprocess://wpm"
required_by = "wasm4pm"
timeout_ms = 30000
fallback = "fail"

[mcp.servers.config]
binary_path = "env:WPM_BINARY"  # defaults to /usr/local/bin/wpm
version_requirement = ">= 5.0.0"

[[mcp.servers]]
id = "rust-docs"
name = "Rust Documentation"
type = "custom"
endpoint = "local://rust-docs"
required_by = null
timeout_ms = 5000
fallback = "disable"

[mcp.servers.config]
cache_dir = ".cargo-cicd-docs-cache"
update_interval_days = 7

[[mcp.servers]]
id = "coverage-analysis"
name = "Test Coverage Analyzer"
type = "custom"
endpoint = "subprocess://coverage"
required_by = "process-data"
timeout_ms = 60000
fallback = "disable"

[mcp.servers.config]
tool = "tarpaulin"  # or "llvm-cov"
instrumentation = "runtime"

# ──────────────────────────────────────────────────────────────
# Plugin Configuration
# ──────────────────────────────────────────────────────────────

[plugins]
enabled = true

# Policy plugins
[[plugins.policies]]
id = "my-org.custom-lint-policy"
path = "~/.cargo/plugins/libcustom_policy.so"
enabled = true
requires_features = ["autonomic"]

[[plugins.policies]]
id = "my-org.compliance-checker"
path = "~/.cargo/plugins/compliance.wasm"
enabled = true
requires_features = []

# Adapter plugins
[[plugins.adapters]]
id = "jenkins-ci-adapter"
path = "~/.cargo/plugins/jenkins_adapter.so"
enabled = true
requires_features = ["process-data"]

[plugins.adapters.config]
jenkins_url = "https://jenkins.internal.company.com"
job_prefix = "my-team/"

# ──────────────────────────────────────────────────────────────
# Existing sections
# ──────────────────────────────────────────────────────────────

[state]
dirty = false
target_size_gb = 2.5
changed_files = 3
changed_tests = 1
changed_trybuild_fixtures = 0

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
enabled = true
base = "origin/main"

[trybuild.changed]
enabled = true
snapshot_mode = "changed-only"

[git.phase]
require_clean_tree = true
commit_after_phase = false

[autonomic]
enabled = true
mode = "suggest"
```

### 4.2 Config Deserialization & Validation

```rust
// src/cicd_toml.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CicdToml {
    // ... existing sections
    #[serde(default)]
    pub mcp: McpSection,
    #[serde(default)]
    pub plugins: PluginsSection,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpSection {
    pub enabled: bool,
    pub default_timeout_ms: Option<u32>,
    pub fail_fast: bool,
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: String, // "external" | "custom"
    pub endpoint: String,
    pub required_by: Option<String>,
    pub timeout_ms: u32,
    pub fallback: String, // "subprocess" | "disable" | "fail"
    #[serde(default)]
    pub config: toml::map::Map<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PluginsSection {
    pub enabled: bool,
    #[serde(default)]
    pub policies: Vec<PluginConfig>,
    #[serde(default)]
    pub adapters: Vec<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub id: String,
    pub path: String,
    pub enabled: bool,
    #[serde(default)]
    pub requires_features: Vec<String>,
    #[serde(default)]
    pub config: toml::map::Map<String, toml::Value>,
}

impl McpSection {
    /// Validate that all required servers for active feature flags are present
    pub fn validate_feature_requirements(&self, active_features: &[&str]) -> Result<()> {
        for server in &self.servers {
            if let Some(required_by) = &server.required_by {
                if active_features.contains(&required_by.as_str()) {
                    if !self.servers.iter().any(|s| &s.id == server.id) {
                        return Err(anyhow::anyhow!(
                            "Feature {} requires MCP server {}, but it is not configured",
                            required_by,
                            server.id
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
```

---

### 4.3 Environment Variable Interpolation

**Pattern:** Values prefixed with `env:` are substituted from environment.

```rust
// src/config/env_expansion.rs
pub fn expand_env_values(config: &mut toml::map::Map<String, toml::Value>) -> Result<()> {
    for (_, value) in config.iter_mut() {
        if let toml::Value::String(s) = value {
            if s.starts_with("env:") {
                let env_var = &s[4..];
                let expanded = std::env::var(env_var)
                    .map_err(|_| anyhow::anyhow!("Environment variable {} not set", env_var))?;
                *value = toml::Value::String(expanded);
            }
        }
    }
    Ok(())
}
```

**Usage:**

```toml
[mcp.servers.config]
token = "env:GITHUB_TOKEN"
binary_path = "env:WPM_BINARY"
```

---

## Part 5: Integration with Evidence & Process Events

### 5.1 MCP Interaction Logging

All MCP server calls must be logged as process events for wasm4pm adjudication.

```rust
// src/evidence.rs
pub struct ProcessEvent {
    pub timestamp: DateTime<Utc>,
    pub case_id: String,
    pub phase: String,      // "mcp:github:list_commits"
    pub actor: String,      // "cargo-cicd"
    pub activity: String,   // "invoke"
    pub resource: String,   // server name
    pub result: String,     // "success" | "timeout" | "error"
    pub details: Option<String>,
}

impl ProcessEvent {
    /// Log an MCP server invocation
    pub fn mcp_call(server_id: &str, capability: &str, result: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            phase: format!("mcp:{}:{}", server_id, capability),
            actor: "cargo-cicd".into(),
            activity: "invoke".into(),
            resource: server_id.into(),
            result: result.into(),
            details: None,
        }
    }
}
```

**Example XES Output:**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xmlns="http://www.xes-standard.org/">
  <trace>
    <string key="concept:name" value="cargo-cicd-run-26d3a9"/>
    <event>
      <string key="concept:name" value="start"/>
      <string key="lifecycle:transition" value="start"/>
    </event>
    <event>
      <string key="concept:name" value="mcp:github:list_commits"/>
      <string key="org:resource" value="github"/>
      <string key="org:result" value="success"/>
      <date key="time:timestamp" value="2026-06-14T10:30:45Z"/>
    </event>
    <event>
      <string key="concept:name" value="mcp:workspace-fs:find_tests"/>
      <string key="org:resource" value="workspace-fs"/>
      <string key="org:result" value="success"/>
      <date key="time:timestamp" value="2026-06-14T10:30:46Z"/>
    </event>
  </trace>
</log>
```

---

## Part 6: Adapter-to-EngineState Mapping

### 6.1 MCP Adapter Integration Points

```rust
// src/adapters/mcp_adapters.rs

/// Populate EngineState from external MCP servers
pub struct McpAdaptersRunner {
    registry: MCPRegistry,
}

impl McpAdaptersRunner {
    pub async fn run(&self, state: &mut EngineState) -> Result<()> {
        self.populate_git_phase(state).await?;
        self.populate_changed_files(state).await?;
        self.populate_test_plan(state).await?;
        Ok(())
    }
    
    /// Use GitHub MCP to populate git_phase state
    async fn populate_git_phase(&self, state: &mut EngineState) -> Result<()> {
        if let Ok(Some(github)) = self.registry.get_server("github") {
            let commits = github.commits_since("origin/main").await?;
            state.git_phase.remote_commits = commits.len();
            state.git_phase.latest_remote_hash = commits.first().map(|c| c.hash.clone());
        }
        Ok(())
    }
    
    /// Use GitHub MCP to detect changed test files
    async fn populate_changed_files(&self, state: &mut EngineState) -> Result<()> {
        if let Ok(Some(fs)) = self.registry.get_server("workspace-fs") {
            let test_files = fs.find_tests(&std::env::current_dir()?).await?;
            state.changed_files.test_files_changed = test_files.len();
        }
        Ok(())
    }
    
    /// Use GitHub + filesystem MCP for test plan
    async fn populate_test_plan(&self, state: &mut EngineState) -> Result<()> {
        let mut test_plan = TestPlanState::default();
        
        // Use GitHub to find commits since base
        if let Ok(Some(github)) = self.registry.get_server("github") {
            let commits = github.commits_since("origin/main").await?;
            test_plan.base_commits = commits;
        }
        
        // Use filesystem to find test files
        if let Ok(Some(fs)) = self.registry.get_server("workspace-fs") {
            let tests = fs.find_tests(&std::env::current_dir()?).await?;
            test_plan.discovered_tests = tests;
        }
        
        state.test_plan = test_plan;
        Ok(())
    }
}
```

---

## Part 7: Fallback & Degradation Strategies

### 7.1 Subprocess Fallback Layer

When an MCP server is unavailable and `fallback = "subprocess"`, cargo-cicd reverts to shell command invocation.

```rust
// src/adapters/fallback.rs
pub struct FallbackGitHub;

impl FallbackGitHub {
    pub fn commits_since(base: &str) -> Result<Vec<CommitMetadata>> {
        // Fallback: use `git log`, `gh api` subprocess commands
        let output = Command::new("git")
            .args(["log", "--format=%H:%an:%aI:%s", &format!("{}..HEAD", base)])
            .output()?;
        
        let mut commits = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                commits.push(CommitMetadata {
                    hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    timestamp: DateTime::parse_from_rfc3339(parts[2])?
                        .with_timezone(&Utc),
                    message: parts[3..].join(":"),
                    files_changed: Vec::new(), // Limited fallback capability
                });
            }
        }
        Ok(commits)
    }
}
```

**Fallback Decision Tree:**

```
if mcp_server_available {
    use mcp_server
} else if fallback == "subprocess" {
    use subprocess fallback (limited capabilities)
} else if fallback == "disable" {
    log warning; skip feature
} else if fallback == "fail" {
    error: MCP server required, cannot proceed
}
```

---

## Part 8: Testing & Validation

### 8.1 Test Categories

#### A. MCP Server Availability Tests

```rust
// tests/mcp_server_availability.rs
#[test]
fn test_github_mcp_discovery() {
    let config = CicdToml::from_path("cicd.toml").unwrap();
    let mut registry = MCPRegistry::from_cicd_toml(&config).unwrap();
    let availability = registry.probe();
    
    // GitHub server should be discoverable
    assert!(availability.contains_key("github"));
}

#[test]
fn test_mcp_fallback_subprocess_when_unavailable() {
    let mut registry = MCPRegistry::new();
    // Simulate unavailable MCP server
    registry.servers.insert("github".to_string(), ServerCapability {
        id: "github".to_string(),
        fallback: FallbackStrategy::Subprocess,
        ..Default::default()
    });
    
    // Fallback should work
    let result = registry.get_server("github");
    assert!(result.is_ok()); // Either MCP or subprocess fallback succeeds
}
```

#### B. Feature Flag Validation Tests

```rust
// tests/mcp_feature_requirements.rs
#[test]
fn test_autonomic_requires_github_mcp() {
    let config = CicdToml::default();
    let registry = MCPRegistry::from_cicd_toml(&config).unwrap();
    
    let result = registry.validate_feature_requirements(&["autonomic"]);
    // Should error if GitHub MCP is not configured
}

#[test]
#[cfg(feature = "wasm4pm")]
fn test_wasm4pm_requires_oracle_server() {
    let config = CicdToml::default();
    let registry = MCPRegistry::from_cicd_toml(&config).unwrap();
    
    let result = registry.validate_feature_requirements(&["wasm4pm"]);
    // Should error if wasm4pm oracle is not configured
}
```

#### C. Plugin Loading Tests

```rust
// tests/plugin_loading.rs
#[test]
fn test_load_policy_plugin_from_so() {
    // Compile a test plugin to .so
    let plugin = PolicyPluginRegistry::load_plugin("test-policy", Path::new("target/test_policy.so"))
        .unwrap();
    
    assert_eq!(plugin.id(), "test-policy");
    assert!(plugin.requires_features().is_empty());
}

#[test]
fn test_policy_plugin_evaluation() {
    let mut registry = PolicyPluginRegistry::new();
    registry.load_plugin("test", Path::new("target/test_policy.so")).unwrap();
    
    let state = EngineState::default();
    let results = registry.evaluate_all(&state).unwrap();
    
    assert!(!results.is_empty());
}
```

#### D. Evidence Logging Tests

```rust
// tests/mcp_evidence_logging.rs
#[test]
fn test_mcp_call_logged_as_event() {
    let event = ProcessEvent::mcp_call("github", "list_commits", "success");
    
    assert_eq!(event.phase, "mcp:github:list_commits");
    assert_eq!(event.resource, "github");
    assert_eq!(event.result, "success");
}

#[test]
fn test_xes_export_includes_mcp_events() {
    let events = vec![
        ProcessEvent::mcp_call("github", "list_commits", "success"),
        ProcessEvent::mcp_call("workspace-fs", "find_tests", "success"),
    ];
    
    let xes = XesDocument::from_events(&events);
    // Validate XES schema
    assert!(xes.validate().is_ok());
}
```

---

## Part 9: Implementation Roadmap

### Phase 1: Foundation (v26.6.3)
- [ ] Define `MCPRegistry` and `ServerCapability` structs
- [ ] Implement `cicd.toml` extensions for `[mcp]` and `[plugins]` sections
- [ ] Add GitHub MCP server integration (read-only capabilities)
- [ ] Implement subprocess fallback layer
- [ ] Add MCP event logging to evidence subsystem

### Phase 2: Custom Servers (v26.7.0)
- [ ] Implement wasm4pm oracle MCP server wrapper
- [ ] Create Rust documentation lookup server
- [ ] Add policy plugin registry and dynamic loading
- [ ] Support both .so and WASM plugin formats

### Phase 3: Plugin Ecosystem (v26.7.x)
- [ ] Implement adapter plugin system
- [ ] Add plugin template crate generator: `cargo cicd plugin new`
- [ ] Create example plugins for common use cases
- [ ] Document plugin authoring guide

### Phase 4: Advanced Features (v26.8+)
- [ ] File system watcher MCP server for incremental change detection
- [ ] Test coverage analysis MCP server
- [ ] IDE integration hooks (Claude Code LSP)
- [ ] Remote MCP server discovery and marketplace

---

## Part 10: Rationale & Design Decisions

### Why MCP for External APIs?

**Problem:** cargo-cicd currently shells out to `git`, `gh`, `cargo` directly. This is:
1. Subprocess overhead
2. Incompatible with WASM/constrained environments
3. Hard to stub/test
4. Splits logic across multiple commands

**Solution:** MCP abstracts external dependencies behind a versioned, testable interface.

**Benefits:**
- **Composability:** IDE (Claude Code), CLI (cargo-cicd), and other tools can share MCP servers
- **Testability:** Mock MCP servers for unit tests
- **Extensibility:** Users can plug in custom MCP servers for proprietary tools
- **Future-proofing:** Switch backends (e.g., GitHub CLI → GitHub API → Octokit) without changing cargo-cicd

### Why Separate Plugin Architecture?

**Problem:** Not all extensions fit the adapter pattern. Autonomic policies need customization, and different organizations have different rules.

**Solution:** Plugin traits for policies and adapters enable third-party extension without forking.

**Trade-offs:**
- **Pro:** Decouples policy domain knowledge from core cargo-cicd
- **Con:** Requires dynamic loading, ABI stability, and plugin distribution
- **Mitigation:** Provide plugin templates, host example plugins, support WASM as ABI-stable format

### Why Both Subprocess Fallback and MCP?

**Problem:** Making MCP mandatory breaks offline workflows and adds deployment complexity.

**Solution:** MCP is opt-in; subprocess fallbacks ensure cargo-cicd works in minimal environments.

**Benefit:** Users can adopt MCP gradually. Early adopters get speed and testability; conservative teams stay on subprocess.

---

## Part 11: Examples & Use Cases

### Example 1: Autonomic Policy Using GitHub MCP

```rust
// User adds custom autonomic policy via plugin
#[derive(Debug)]
pub struct RequireApprovedPullRequest;

impl PolicyPlugin for RequireApprovedPullRequest {
    fn id(&self) -> &str { "my-org.require-approved-pr" }
    fn name(&self) -> &str { "Require Approved Pull Request" }
    fn requires_features(&self) -> Vec<&str> { vec!["autonomic"] }
    
    fn evaluate(&self, state: &EngineState) -> Result<PolicyResult> {
        // This requires GitHub MCP integration
        let github = state.mcp_registry.get_server("github")?
            .ok_or_else(|| anyhow!("GitHub MCP required"))?;
        
        let pr = github.get_pr_for_branch(&state.git_phase.current_branch)?;
        
        match pr {
            Some(pr) if pr.reviews.iter().any(|r| r.state == "APPROVED") => {
                Ok(PolicyResult {
                    verdict: "pass".into(),
                    recommendation: None,
                    ..Default::default()
                })
            }
            Some(pr) => {
                Ok(PolicyResult {
                    verdict: "warn".into(),
                    recommendation: Some(format!(
                        "PR #{} requires approval. Current reviews: {:?}",
                        pr.number, pr.reviews
                    )),
                    ..Default::default()
                })
            }
            None => {
                Ok(PolicyResult {
                    verdict: "alert".into(),
                    recommendation: Some("Current branch is not associated with a pull request".into()),
                    ..Default::default()
                })
            }
        }
    }
    
    fn recommendation_template(&self) -> &str {
        "PR Review Status: {details}"
    }
}
```

**Configuration:**

```toml
[[plugins.policies]]
id = "my-org.require-approved-pr"
path = "~/.cargo/plugins/libpr_approval.so"
enabled = true
requires_features = ["autonomic"]
```

### Example 2: Custom Coverage Analysis Adapter

```rust
// User provides custom coverage adapter via MCP
pub struct CoverageAnalysisServer;

impl AdapterPlugin for CoverageAnalysisServer {
    fn id(&self) -> &str { "my-org.coverage-adapter" }
    fn name(&self) -> &str { "Coverage Analysis" }
    
    fn populate(&self, state: &mut EngineState) -> Result<()> {
        // Shell to coverage tool
        let output = Command::new("cargo")
            .args(["tarpaulin", "--format", "json"])
            .output()?;
        
        let coverage: CoverageReport = serde_json::from_slice(&output.stdout)?;
        
        // Populate state
        state.artifacts.coverage_report = Some(coverage);
        
        Ok(())
    }
    
    fn is_available(&self) -> bool {
        Command::new("cargo")
            .arg("tarpaulin")
            .arg("--version")
            .status()
            .is_ok()
    }
}
```

### Example 3: cicd.toml with All MCP & Plugin Features

```toml
[workspace]
name = "my-workspace"
toolchain = "stable"

[mcp]
enabled = true
default_timeout_ms = 5000
fail_fast = false

# GitHub integration for remote history
[[mcp.servers]]
id = "github"
name = "GitHub API"
type = "external"
endpoint = "https://mcp.github.com"
required_by = "autonomic"
timeout_ms = 10000
fallback = "subprocess"
[mcp.servers.config]
owner = "my-org"
repo = "my-repo"
token = "env:GITHUB_TOKEN"

# Workspace file system
[[mcp.servers]]
id = "workspace-fs"
name = "File System Inspector"
type = "external"
endpoint = "stdio://workspace-fs"
required_by = "process-data"
timeout_ms = 3000
fallback = "disable"

# wasm4pm oracle for evidence adjudication
[[mcp.servers]]
id = "wasm4pm-oracle"
name = "Process Evidence Validator"
type = "custom"
endpoint = "subprocess://wpm"
required_by = "wasm4pm"
timeout_ms = 30000
fallback = "fail"
[mcp.servers.config]
binary_path = "env:WPM_BINARY"
version_requirement = ">= 5.0.0"

# Custom coverage analysis
[[mcp.servers]]
id = "coverage"
name = "Coverage Analysis"
type = "custom"
endpoint = "subprocess://coverage"
required_by = null
timeout_ms = 60000
fallback = "disable"
[mcp.servers.config]
tool = "tarpaulin"

[plugins]
enabled = true

# Custom policies
[[plugins.policies]]
id = "my-org.require-approved-pr"
path = "~/.cargo/plugins/libpr_approval.so"
enabled = true
requires_features = ["autonomic"]

[[plugins.policies]]
id = "my-org.security-scan"
path = "~/.cargo/plugins/security.wasm"
enabled = true
requires_features = []

# Custom adapters
[[plugins.adapters]]
id = "my-org.jenkins-adapter"
path = "~/.cargo/plugins/jenkins.so"
enabled = true
requires_features = ["process-data"]
[plugins.adapters.config]
jenkins_url = "https://jenkins.internal"

# Rest of config
[state]
dirty = false
target_size_gb = 2.5

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
enabled = true
base = "origin/main"

[autonomic]
enabled = true
mode = "suggest"
```

---

## Part 12: Security & Isolation Considerations

### 12.1 Plugin Sandboxing

**Risk:** Plugins are arbitrary code; they could:
- Read sensitive files
- Exfiltrate environment variables
- Modify EngineState in unsafe ways

**Mitigation:**

1. **WASM Plugins:** Run in wasmtime sandbox with restricted imports
   ```rust
   let mut linker = wasmtime::Linker::new(&engine);
   // Only export safe APIs; restrict file system and env access
   linker.func_wrap("env", "get_state", |state: &EngineState| { /* */ })?;
   ```

2. **Shared Library Plugins:** Require explicit permission in `cicd.toml`
   ```toml
   [[plugins.policies]]
   id = "my-org.policy"
   path = "~/.cargo/plugins/policy.so"
   enabled = true
   # Explicit permission required for .so loading
   permission = "plugin:load-shared-library"
   ```

3. **Environment Variable Filtering:** Allowlist safe env vars
   ```toml
   [mcp.servers.config]
   env_allowlist = ["RUST_BACKTRACE", "CARGO_BUILD_JOBS"]
   # GITHUB_TOKEN, AWS_KEY, etc. are blocked by default
   ```

### 12.2 MCP Server Authentication

**Risk:** MCP servers may need credentials (GitHub token, API keys).

**Solution:**

```toml
[mcp.servers.config]
# Credentials are read from environment; never stored in cicd.toml
token = "env:GITHUB_TOKEN"
api_key = "env:CUSTOM_API_KEY"

# Alternative: credential file with restricted permissions
token_file = "~/.config/cargo-cicd/github.token"  # chmod 600
```

---

## Conclusion

This MCP integration strategy provides cargo-cicd with:

1. **Flexibility:** Extend behavior via custom policies, adapters, and MCP servers
2. **Composability:** Share MCP servers with IDE and other CLI tools
3. **Testability:** Mock MCP servers for deterministic tests
4. **Gradual Adoption:** MCP is optional; subprocess fallbacks ensure backward compatibility
5. **Evidence Preservation:** All integrations log events for wasm4pm adjudication

The design maintains cargo-cicd's core principles:
- **Level 5 process data engine:** MCP interactions are logged as events
- **Adapter-based state population:** MCP adapters populate EngineState like existing adapters
- **Autonomic policies:** User-defined and customizable via plugins
- **Local-first orientation:** Optional remote integrations, works offline

Implementation proceeds in phases, starting with GitHub MCP (v26.6.3) and expanding to custom servers and plugins (v26.7+).
