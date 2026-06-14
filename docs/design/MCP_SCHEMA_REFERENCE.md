# MCP Schema Reference for cargo-cicd

**Document Version:** 1.0  
**Date:** 2026-06-14  
**Target Audience:** Integration engineers, config authors

---

## Table of Contents

1. [MCP Configuration Schema](#mcp-configuration-schema)
2. [MCP Server Tool Schemas](#mcp-server-tool-schemas)
3. [MCP Resource Schemas](#mcp-resource-schemas)
4. [cicd.toml Examples](#cicdtoml-examples)
5. [Adapter Rust Types](#adapter-rust-types)
6. [Test Fixtures](#test-fixtures)

---

## MCP Configuration Schema

### Root cicd.toml Structure

```toml
# ── Core Configuration ────────────────────────────────────────────────────────

[workspace]
name = "cargo-cicd"
toolchain = "stable-aarch64-apple-darwin"
target_dir = "target"

[state]
dirty = false
target_size_gb = 2.5
changed_files = 0
changed_tests = 0
changed_trybuild_fixtures = 0

[autonomic]
enabled = true
mode = "suggest"

# ── NEW: MCP Global Settings ──────────────────────────────────────────────────

[mcp]
enabled = true                          # Master kill-switch
log_level = "info"                      # "debug" | "info" | "warn" | "error"
timeout_secs = 30                       # Default timeout per MCP call
connection_pool_size = 5                # Max concurrent connections
retry_attempts = 3                      # Retry failed calls
retry_backoff_ms = 100                  # Exponential backoff

# ── GitHub MCP Server ─────────────────────────────────────────────────────────

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "seanchatmangpt/cargo-cicd"
verify_checks = [
  "continuous-integration/travis-ci",
  "continuous-integration/appveyor"
]
require_approval = true
protect_main = true
cache_ttl_secs = 300

# ── Workspace MCP Server ──────────────────────────────────────────────────────

[mcp.workspace]
enabled = true
url = "http://localhost:3001"
include_advisories = true
estimate_build_time = true
feature_matrix_limit = 32
dependency_graph_depth = "transitive"
cache_ttl_secs = 600

# ── Environment MCP Server ────────────────────────────────────────────────────

[mcp.environment]
enabled = true
url = "http://localhost:3002"
check_msrv_toolchain = true
predict_build_time = true
alert_on_low_disk = true
low_disk_threshold_gb = 5.0
cache_ttl_secs = 300

# ── wasm4pm Oracle MCP Server ─────────────────────────────────────────────────

[mcp.wasm4pm]
enabled = true
url = "http://localhost:3003"
oracle_version = "26.6.2"
require_accept = true
fallback_to_shell = true
evidence_dir = "target/cargo-cicd/evidence"
cache_verdicts = false

# ── Rust Documentation MCP Server ─────────────────────────────────────────────

[mcp.rustdoc]
enabled = false
url = "http://localhost:3004"
check_msrv_items = true
check_deprecations = true
build_local_docs = true
doc_cache_dir = "target/doc"
cache_ttl_secs = 3600

# ── Policy Suggestion MCP Server ──────────────────────────────────────────────

[mcp.policy]
enabled = false
url = "http://localhost:3005"
engine = "claude-3-5-sonnet"
confidence_threshold = 0.75
explain_all_verdicts = true
context_window_tokens = 4096
cache_explanations = true
cache_ttl_secs = 3600

# ── Plugin System ─────────────────────────────────────────────────────────────

[plugins]
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

# ── Events Log (Appended by cargo-cicd) ───────────────────────────────────────

[[events]]
kind = "status"
verdict = "pass"
timestamp = "2026-06-14T10:30:00Z"
mcp_servers = ["github", "workspace"]

[[events]]
kind = "mcp_call"
server = "github"
tool = "get_pr_metadata"
timestamp = "2026-06-14T10:30:01Z"
result = "success"
cached = false
duration_ms = 145
```

---

## MCP Server Tool Schemas

### GitHub MCP Server Tools

#### Tool: `get_branch_status`

**Purpose:** Check branch protection rules, CI status, and required reviews.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_branch_status",
    "arguments": {
      "branch": "main",
      "repo": "owner/name",
      "check_enforcement": true
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "Success"
    },
    {
      "type": "text",
      "text": "{\"branch\": \"main\", \"protected\": true, \"requires_approving_reviews\": 1, \"dismiss_stale_reviews\": true, \"require_code_owner_reviews\": true, \"required_status_checks\": [\"continuous-integration/travis-ci\", \"codecov/patch\"], \"all_checks_passing\": true}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct BranchStatus {
    pub branch: String,
    pub protected: bool,
    pub requires_approving_reviews: u32,
    pub dismiss_stale_reviews: bool,
    pub require_code_owner_reviews: bool,
    pub required_status_checks: Vec<String>,
    pub all_checks_passing: bool,
    pub last_updated_at: String,
}
```

#### Tool: `get_pr_metadata`

**Purpose:** Get PR status, checks, and review information.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_pr_metadata",
    "arguments": {
      "pr_number": 42,
      "repo": "owner/name"
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"number\": 42, \"title\": \"Add MCP integration\", \"draft\": false, \"mergeable\": true, \"merged\": false, \"state\": \"open\", \"head_commit\": \"abc123def456\", \"approved_reviews\": 1, \"requested_changes\": 0, \"checks\": [{\"name\": \"cargo-test\", \"status\": \"success\", \"conclusion\": \"success\"}]}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PrMetadata {
    pub number: u32,
    pub title: String,
    pub draft: bool,
    pub mergeable: bool,
    pub merged: bool,
    pub state: String,  // "open" | "closed"
    pub head_commit: String,
    pub approved_reviews: u32,
    pub requested_changes: u32,
    pub checks: Vec<CheckRun>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckRun {
    pub name: String,
    pub status: String,     // "queued" | "in_progress" | "completed"
    pub conclusion: String, // "success" | "failure" | "neutral" | "cancelled" | "skipped"
}
```

#### Tool: `list_recent_releases`

**Purpose:** Get recent releases and asset information.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_recent_releases",
    "arguments": {
      "repo": "owner/name",
      "limit": 10
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "[{\"tag_name\": \"v26.6.2\", \"created_at\": \"2026-06-14T10:00:00Z\", \"assets\": [{\"name\": \"cargo-cicd-x86_64-unknown-linux-gnu\", \"download_count\": 234}]}]"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub created_at: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub size: u64,
    pub download_count: u32,
    pub browser_download_url: String,
}
```

#### Tool: `get_code_ownership`

**Purpose:** Determine CODEOWNERS for given paths.

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_code_ownership",
    "arguments": {
      "repo": "owner/name",
      "paths": ["src/main.rs", "Cargo.toml"]
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"src/main.rs\": [\"@alice\", \"@bob\"], \"Cargo.toml\": [\"@carol\"]}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeOwnershipMap {
    #[serde(flatten)]
    pub owners_by_path: std::collections::HashMap<String, Vec<String>>,
}
```

---

### Workspace MCP Server Tools

#### Tool: `get_dependency_graph`

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_dependency_graph",
    "arguments": {
      "depth": "transitive",
      "include_dev_deps": true
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"nodes\": [{\"id\": \"cargo-cicd\", \"version\": \"26.6.2\"}, {\"id\": \"serde\", \"version\": \"1.0.210\"}], \"edges\": [{\"from\": \"cargo-cicd\", \"to\": \"serde\", \"version_req\": \"=1.0\", \"features\": [\"derive\"]}]}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub version: String,
    pub is_workspace_member: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub version_req: String,
    pub features: Vec<String>,
    pub optional: bool,
}
```

#### Tool: `get_affected_crates`

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_affected_crates",
    "arguments": {
      "changed_crate": "cargo-cicd-core",
      "include_transitive_dependents": true
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"affected\": [\"cargo-cicd\", \"cargo-cicd-lsp\"], \"count\": 2}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AffectedCratesResult {
    pub affected: Vec<String>,
    pub count: usize,
}
```

#### Tool: `check_security_advisories`

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "check_security_advisories",
    "arguments": {
      "advisory_filter": "any"
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"advisories\": [{\"id\": \"GHSA-xxxx-yyyy-zzzz\", \"crate\": \"tokio\", \"version\": \"1.0.0\", \"advisory\": \"Use-after-free in tokio::spawn\", \"severity\": \"critical\"}], \"count\": 1}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub id: String,
    pub crate_name: String,
    pub version: String,
    pub advisory: String,
    pub severity: String,  // "low" | "medium" | "high" | "critical"
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityAuditResult {
    pub advisories: Vec<SecurityAdvisory>,
    pub count: usize,
}
```

---

### Environment MCP Server Tools

#### Tool: `get_installed_rustc_versions`

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"toolchains\": [{\"name\": \"stable-aarch64-apple-darwin\", \"version\": \"1.85.0\", \"default\": true}, {\"name\": \"nightly-aarch64-apple-darwin\", \"version\": \"1.86.0-nightly\"}]}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledToolchain {
    pub name: String,
    pub version: String,
    pub default: bool,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledRustcVersions {
    pub toolchains: Vec<InstalledToolchain>,
}
```

#### Tool: `get_system_resources`

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"cpu_cores\": 8, \"cpu_count_physical\": 4, \"memory_gb\": 16, \"disk_available_gb\": 256, \"temp_dir\": \"/tmp\", \"temp_available_gb\": 100}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_cores: usize,
    pub cpu_count_physical: usize,
    pub memory_gb: u64,
    pub disk_available_gb: u64,
    pub temp_dir: String,
    pub temp_available_gb: u64,
}
```

#### Tool: `check_development_tools`

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "check_development_tools",
    "arguments": {
      "tools": ["llvm-tools", "miri", "cargo-fmt", "cargo-clippy"]
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"tools\": [{\"name\": \"llvm-tools\", \"installed\": true, \"version\": \"1.85.0\"}, {\"name\": \"miri\", \"installed\": false}]}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevelopmentToolsStatus {
    pub tools: Vec<ToolStatus>,
}
```

---

### wasm4pm MCP Server Tools

#### Tool: `submit_evidence`

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "submit_evidence",
    "arguments": {
      "evidence_path": "target/cargo-cicd/evidence/events.xes",
      "metadata": {
        "session_id": "abc123",
        "workflow": "cargo-cicd-status"
      }
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"verdict\": \"Accept\", \"score\": 0.98, \"submission_uuid\": \"550e8400-e29b-41d4-a716-446655440000\", \"evidence_hash\": \"sha256:abc123...\"}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitEvidenceResult {
    pub verdict: String,  // "Accept" | "Warn" | "Refuse"
    pub score: f32,
    pub submission_uuid: String,
    pub evidence_hash: String,
}
```

#### Tool: `explain_verdict`

**Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "explain_verdict",
    "arguments": {
      "submission_uuid": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**Response:**
```json
{
  "type": "tool_result",
  "content": [
    {
      "type": "text",
      "text": "{\"verdict\": \"Accept\", \"explanation\": \"Process execution conforms to requirements.\", \"non_conformances\": [], \"recommendations\": []}"
    }
  ]
}
```

**Rust Type:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ExplainVerdictResult {
    pub verdict: String,
    pub explanation: String,
    pub non_conformances: Vec<String>,
    pub recommendations: Vec<String>,
}
```

---

## MCP Resource Schemas

### GitHub MCP Resources

```
resource: github://repository
  Type: application/json
  Content:
  {
    "name": "cargo-cicd",
    "owner": "seanchatmangpt",
    "url": "https://github.com/seanchatmangpt/cargo-cicd",
    "description": "Level 5 process-data engine",
    "default_branch": "main",
    "is_fork": false
  }

resource: github://branch/{branch}
  Type: application/json
  Content:
  {
    "name": "main",
    "protected": true,
    "protection_rules": {
      "require_code_owner_reviews": true,
      "required_status_checks": ["continuous-integration"]
    },
    "last_commit": "abc123def456"
  }

resource: github://pull_requests
  Type: application/json
  Content:
  [
    {
      "number": 42,
      "title": "Add MCP integration",
      "state": "open",
      "author": "alice"
    }
  ]
```

### Workspace Resources

```
resource: workspace://dependency-graph
  Type: application/json
  Content: { "nodes": [...], "edges": [...] }

resource: workspace://advisories
  Type: application/json
  Content:
  {
    "count": 1,
    "advisories": [
      {
        "id": "GHSA-xxxx-yyyy-zzzz",
        "severity": "critical"
      }
    ]
  }
```

### Environment Resources

```
resource: env://system-status
  Type: application/json
  Content:
  {
    "cpu_cores": 8,
    "memory_gb": 16,
    "disk_available_gb": 256
  }

resource: env://toolchain-status
  Type: application/json
  Content:
  {
    "default": "stable-aarch64-apple-darwin",
    "installed": ["stable", "nightly"]
  }
```

---

## cicd.toml Examples

### Example 1: Minimal MCP Setup

```toml
[workspace]
name = "my-project"
toolchain = "stable"
target_dir = "target"

[mcp]
enabled = true
timeout_secs = 30

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "org/my-project"
```

### Example 2: Full MCP Stack

```toml
[workspace]
name = "my-monorepo"
toolchain = "stable-x86_64-unknown-linux-gnu"
target_dir = "target"

[mcp]
enabled = true
log_level = "info"
timeout_secs = 30
connection_pool_size = 5

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "company/monorepo"
verify_checks = ["cargo-test", "cargo-clippy", "cargo-fmt"]
require_approval = true
protect_main = true
cache_ttl_secs = 300

[mcp.workspace]
enabled = true
url = "http://localhost:3001"
include_advisories = true
estimate_build_time = true
feature_matrix_limit = 16
cache_ttl_secs = 600

[mcp.environment]
enabled = true
url = "http://localhost:3002"
check_msrv_toolchain = true
predict_build_time = true
alert_on_low_disk = true
low_disk_threshold_gb = 10.0

[mcp.wasm4pm]
enabled = true
url = "http://localhost:3003"
oracle_version = "26.6.2"
require_accept = true
cache_verdicts = false

[mcp.policy]
enabled = true
url = "http://localhost:3005"
engine = "claude-3-5-sonnet"
confidence_threshold = 0.8
explain_all_verdicts = true
cache_explanations = true

[autonomic]
enabled = true
mode = "suggest"

[[events]]
kind = "status"
verdict = "pass"
mcp_servers = ["github", "workspace", "environment", "wasm4pm"]
```

### Example 3: GitHub-Only Integration

```toml
[mcp]
enabled = true
timeout_secs = 15

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "owner/repo"
verify_checks = ["ci/github"]
require_approval = false
protect_main = false
```

### Example 4: Offline (All Disabled)

```toml
[mcp]
enabled = false

# All MCP sections are ignored
[mcp.github]
enabled = false

[mcp.workspace]
enabled = false
```

---

## Adapter Rust Types

### MCP Configuration Structs

```rust
// src/cicd_toml.rs — Configuration structures

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CicdToml {
    pub workspace: WorkspaceSection,
    pub state: StateSection,
    pub target: TargetSection,
    pub test: TestSection,
    pub trybuild: TrybuildSection,
    pub git: GitSection,
    pub autonomic: AutonomicSection,
    #[serde(default)]
    pub mcp: McpGlobalConfig,
    #[serde(default)]
    pub mcp_github: Option<McpGitHubConfig>,
    #[serde(default)]
    pub mcp_workspace: Option<McpWorkspaceConfig>,
    #[serde(default)]
    pub mcp_environment: Option<McpEnvironmentConfig>,
    #[serde(default)]
    pub mcp_wasm4pm: Option<McpWasm4pmConfig>,
    #[serde(default)]
    pub mcp_rustdoc: Option<McpRustdocConfig>,
    #[serde(default)]
    pub mcp_policy: Option<McpPolicyConfig>,
    #[serde(default)]
    pub plugins: std::collections::HashMap<String, PluginConfig>,
    #[serde(default)]
    pub events: Vec<EventRecord>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpGlobalConfig {
    pub enabled: bool,
    pub log_level: String,
    pub timeout_secs: u64,
    pub connection_pool_size: usize,
    pub retry_attempts: u32,
    pub retry_backoff_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpGitHubConfig {
    pub enabled: bool,
    pub url: String,
    pub repo: String,
    #[serde(default)]
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpEnvironmentConfig {
    pub enabled: bool,
    pub url: String,
    pub check_msrv_toolchain: bool,
    pub predict_build_time: bool,
    pub alert_on_low_disk: bool,
    pub low_disk_threshold_gb: f64,
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpWasm4pmConfig {
    pub enabled: bool,
    pub url: String,
    pub oracle_version: String,
    pub require_accept: bool,
    pub fallback_to_shell: bool,
    pub evidence_dir: String,
    pub cache_verdicts: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpRustdocConfig {
    pub enabled: bool,
    pub url: String,
    pub check_msrv_items: bool,
    pub check_deprecations: bool,
    pub build_local_docs: bool,
    pub doc_cache_dir: String,
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct McpPolicyConfig {
    pub enabled: bool,
    pub url: String,
    pub engine: String,
    pub confidence_threshold: f32,
    pub explain_all_verdicts: bool,
    pub context_window_tokens: usize,
    pub cache_explanations: bool,
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub url: String,
    pub enabled: bool,
    #[serde(default)]
    pub features: Vec<String>,
}
```

### Event Recording Structs

```rust
// src/cicd_toml.rs — Event recording

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventRecord {
    pub kind: String,      // "status", "mcp_call", "policy_eval", etc.
    pub verdict: String,   // "pass", "warn", "fail"
    pub timestamp: String, // ISO8601
    #[serde(flatten)]
    pub details: serde_json::Value, // Additional event-specific data
}

// Example event records:
// {
//   "kind": "mcp_call",
//   "verdict": "success",
//   "timestamp": "2026-06-14T10:30:01Z",
//   "server": "github",
//   "tool": "get_pr_metadata",
//   "duration_ms": 145,
//   "cached": false
// }
//
// {
//   "kind": "policy_eval",
//   "verdict": "warn",
//   "timestamp": "2026-06-14T10:30:02Z",
//   "policy": "target_pressure",
//   "recommendation": "Clean target directory"
// }
```

### MCP Adapter Trait

```rust
// src/adapters/mod.rs

pub trait McpAdapter {
    /// Populate EngineState from MCP server(s).
    /// Returns WpmVerdict::Partial if MCP is unavailable.
    fn populate_engine_state(&self, state: &mut EngineState) -> Result<WpmVerdict>;
    
    /// Get the name of this adapter (for logging).
    fn name(&self) -> &'static str;
    
    /// Check if this adapter is enabled in cicd.toml.
    fn is_enabled(&self) -> bool;
}

// Example implementation:
pub struct GitHubMcpAdapter {
    config: McpGitHubConfig,
    client: HttpClient,
}

impl McpAdapter for GitHubMcpAdapter {
    fn populate_engine_state(&self, state: &mut EngineState) -> Result<WpmVerdict> {
        if !self.is_enabled() {
            return Ok(WpmVerdict::Partial);
        }
        
        match self.get_branch_status(&self.config.repo) {
            Ok(status) => {
                state.git_phase.ci_checks_passing = status.all_checks_passing;
                state.git_phase.approvals = status.approved_count;
                Ok(WpmVerdict::Pass)
            }
            Err(_) => {
                // Graceful degradation: continue without GitHub data
                Ok(WpmVerdict::Partial)
            }
        }
    }
    
    fn name(&self) -> &'static str { "github-mcp" }
    fn is_enabled(&self) -> bool { self.config.enabled }
}
```

---

## Test Fixtures

### Mock MCP Servers for Testing

```rust
// tests/mcp_servers/mock_github.rs

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MockGitHubServer {
    port: u16,
    responses: Arc<RwLock<HashMap<String, String>>>,
}

impl MockGitHubServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn with_branch_status(
        self,
        branch: &str,
        status: BranchStatus,
    ) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut responses = self.responses.write().await;
            responses.insert(
                format!("get_branch_status/{}", branch),
                serde_json::to_string(&status).unwrap(),
            );
        });
        self
    }
    
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        let responses = self.responses.clone();
        let port = self.port;
        
        tokio::spawn(async move {
            // Start simple HTTP server on localhost:port
            // Respond to MCP calls with pre-configured responses
        })
    }
}

#[test]
fn test_github_mcp_adapter_with_successful_check() {
    let server = MockGitHubServer::new(3000)
        .with_branch_status("main", BranchStatus {
            branch: "main".into(),
            protected: true,
            all_checks_passing: true,
            ..Default::default()
        });
    
    let _handle = server.spawn();
    
    let adapter = GitHubMcpAdapter::new("http://localhost:3000");
    let mut engine_state = EngineState::default();
    
    let verdict = adapter.populate_engine_state(&mut engine_state).unwrap();
    assert_eq!(verdict, WpmVerdict::Pass);
    assert!(engine_state.git_phase.ci_checks_passing);
}

#[test]
fn test_github_mcp_adapter_graceful_degradation() {
    // Don't start the server
    let adapter = GitHubMcpAdapter::new("http://localhost:9999");
    let mut engine_state = EngineState::default();
    
    let verdict = adapter.populate_engine_state(&mut engine_state).unwrap();
    assert_eq!(verdict, WpmVerdict::Partial);
    // Engine state is unchanged
}
```

### Configuration Fixtures

```rust
// tests/fixtures/mcp_configs.rs

pub fn minimal_mcp_config() -> CicdToml {
    toml::from_str(r#"
[mcp]
enabled = true

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "test/repo"
"#).unwrap()
}

pub fn full_mcp_config() -> CicdToml {
    toml::from_str(r#"
[mcp]
enabled = true
timeout_secs = 30

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "test/repo"

[mcp.workspace]
enabled = true
url = "http://localhost:3001"

[mcp.wasm4pm]
enabled = true
url = "http://localhost:3003"
require_accept = true
"#).unwrap()
}

pub fn offline_config() -> CicdToml {
    toml::from_str(r#"
[mcp]
enabled = false
"#).unwrap()
}
```

---

## Configuration Validation

```rust
// src/cicd_toml.rs — Validation

impl CicdToml {
    /// Validate all MCP configurations.
    pub fn validate_mcp_config(&self) -> Result<()> {
        if !self.mcp.enabled {
            return Ok(());
        }
        
        // Validate GitHub config
        if let Some(cfg) = &self.mcp_github {
            if cfg.enabled {
                if cfg.url.is_empty() {
                    anyhow::bail!("mcp.github.url is required when enabled");
                }
                if cfg.repo.is_empty() {
                    anyhow::bail!("mcp.github.repo is required when enabled");
                }
            }
        }
        
        // Validate workspace config
        if let Some(cfg) = &self.mcp_workspace {
            if cfg.enabled && cfg.url.is_empty() {
                anyhow::bail!("mcp.workspace.url is required when enabled");
            }
        }
        
        // ... similar for other MCP configs ...
        
        Ok(())
    }
}
```

---

## Quick Reference: Configuration Checklist

### To enable GitHub MCP:
```toml
[mcp]
enabled = true

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "owner/repo"
```

### To enable all MCP servers:
```toml
[mcp]
enabled = true
timeout_secs = 30

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "owner/repo"

[mcp.workspace]
enabled = true
url = "http://localhost:3001"

[mcp.environment]
enabled = true
url = "http://localhost:3002"

[mcp.wasm4pm]
enabled = true
url = "http://localhost:3003"

[mcp.policy]
enabled = true
url = "http://localhost:3005"
```

### To disable all MCP:
```toml
[mcp]
enabled = false
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-14
