# MCP Integration Implementation Guide

**Document Version:** 1.0  
**Date:** 2026-06-14  
**Target Audience:** Developers implementing MCP adapters, cargo-cicd maintainers

---

## Overview

This guide provides step-by-step instructions for implementing MCP adapters and MCP servers for cargo-cicd. It covers:

1. **Adapter Implementation** — How to write MCP adapters that integrate with `EngineState`
2. **MCP Server Implementation** — How to write MCP servers that cargo-cicd adapters consume
3. **Testing Strategy** — How to test adapters and servers
4. **Distribution** — How to publish adapters and servers

---

## Part 1: Implementing an MCP Adapter

### 1.1 Anatomy of an MCP Adapter

An MCP adapter is a Rust struct that:
1. Holds a reference to MCP server configuration
2. Implements the `McpAdapter` trait
3. Makes HTTP calls to the MCP server
4. Translates responses into `EngineState` fields

**File Structure:**
```
src/adapters/
├── mod.rs                    # Adapter registry
├── github_mcp.rs             # GitHub MCP adapter (new)
├── workspace_mcp.rs          # Workspace MCP adapter (new)
├── environment_mcp.rs        # Environment MCP adapter (new)
└── ... (existing adapters)
```

### 1.2 Example: GitHub MCP Adapter

Create `src/adapters/github_mcp.rs`:

```rust
use anyhow::Result;
use crate::cicd_toml::McpGitHubConfig;
use crate::engine::EngineState;
use crate::integrations::WpmVerdict;
use serde::{Deserialize, Serialize};

/// GitHub MCP adapter: fetches branch status, PR metadata, etc. from MCP server
pub struct GitHubMcpAdapter {
    config: McpGitHubConfig,
}

#[derive(Debug, Serialize)]
struct McpRequest {
    method: String,
    params: McpRequestParams,
}

#[derive(Debug, Serialize)]
struct McpRequestParams {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct McpResponse {
    content: Vec<McpContent>,
}

#[derive(Debug, Deserialize)]
struct McpContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

impl GitHubMcpAdapter {
    pub fn new(config: McpGitHubConfig) -> Self {
        Self { config }
    }
    
    /// Get branch status from GitHub MCP server
    async fn get_branch_status(&self, branch: &str) -> Result<BranchStatus> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("GitHub MCP is disabled"));
        }
        
        let client = reqwest::Client::new();
        let request = McpRequest {
            method: "tools/call".to_string(),
            params: McpRequestParams {
                name: "get_branch_status".to_string(),
                arguments: serde_json::json!({
                    "branch": branch,
                    "repo": self.config.repo,
                    "check_enforcement": true
                }),
            },
        };
        
        let response = client
            .post(&self.config.url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(self.config.cache_ttl_secs))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub MCP server returned status {}",
                response.status()
            ));
        }
        
        let mcp_response: McpResponse = response.json().await?;
        
        // Parse the response content (second text item in the response array)
        if let Some(content) = mcp_response.content.iter().find(|c| c.content_type == "text") {
            if let Ok(status) = serde_json::from_str::<BranchStatus>(&content.text) {
                return Ok(status);
            }
        }
        
        Err(anyhow::anyhow!("Failed to parse GitHub MCP response"))
    }
    
    /// Get PR metadata from GitHub MCP server
    async fn get_pr_metadata(&self, pr_number: u32) -> Result<PrMetadata> {
        let client = reqwest::Client::new();
        let request = McpRequest {
            method: "tools/call".to_string(),
            params: McpRequestParams {
                name: "get_pr_metadata".to_string(),
                arguments: serde_json::json!({
                    "pr_number": pr_number,
                    "repo": self.config.repo
                }),
            },
        };
        
        let response = client
            .post(&self.config.url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;
        
        let mcp_response: McpResponse = response.json().await?;
        
        if let Some(content) = mcp_response.content.iter().find(|c| c.content_type == "text") {
            if let Ok(metadata) = serde_json::from_str::<PrMetadata>(&content.text) {
                return Ok(metadata);
            }
        }
        
        Err(anyhow::anyhow!("Failed to parse PR metadata"))
    }
}

/// Implement the McpAdapter trait
impl crate::adapters::McpAdapter for GitHubMcpAdapter {
    fn populate_engine_state(&self, state: &mut EngineState) -> Result<WpmVerdict> {
        if !self.is_enabled() {
            return Ok(WpmVerdict::Partial);
        }
        
        // Use tokio runtime to execute async operations
        let rt = tokio::runtime::Runtime::new()?;
        
        // Attempt to get branch status
        match rt.block_on(self.get_branch_status("main")) {
            Ok(status) => {
                state.git_phase.ci_checks_passing = status.all_checks_passing;
                state.git_phase.branch_protected = status.protected;
                
                // Log the successful call
                eprintln!(
                    "[MCP] GitHub: branch status retrieved (CI: {}, protected: {})",
                    status.all_checks_passing, status.protected
                );
                
                Ok(WpmVerdict::Pass)
            }
            Err(e) => {
                // Graceful degradation: MCP unavailable
                eprintln!("[MCP] GitHub: {} — continuing without GitHub data", e);
                Ok(WpmVerdict::Partial)
            }
        }
    }
    
    fn name(&self) -> &'static str {
        "github-mcp"
    }
    
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

// Data structures for GitHub responses

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BranchStatus {
    pub branch: String,
    pub protected: bool,
    pub requires_approving_reviews: u32,
    pub all_checks_passing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrMetadata {
    pub number: u32,
    pub title: String,
    pub draft: bool,
    pub mergeable: bool,
    pub state: String,
    pub approved_reviews: u32,
}
```

### 1.3 Registering the Adapter

Update `src/adapters/mod.rs`:

```rust
pub mod github_mcp;
pub mod workspace_mcp;
pub mod environment_mcp;

pub use github_mcp::GitHubMcpAdapter;
pub use workspace_mcp::WorkspaceMcpAdapter;
pub use environment_mcp::EnvironmentMcpAdapter;

/// Trait that all MCP adapters must implement
pub trait McpAdapter {
    fn populate_engine_state(&self, state: &mut EngineState) -> Result<WpmVerdict>;
    fn name(&self) -> &'static str;
    fn is_enabled(&self) -> bool;
}
```

### 1.4 Integrating into Main Startup

Update `src/main.rs` to load adapters:

```rust
fn populate_engine_state_from_mcp(
    engine: &mut EngineState,
    config: &CicdToml,
) -> Result<()> {
    // Load GitHub adapter if enabled
    if config.mcp.enabled {
        if let Some(github_cfg) = &config.mcp_github {
            let adapter = GitHubMcpAdapter::new(github_cfg.clone());
            let verdict = adapter.populate_engine_state(engine)?;
            if verdict == WpmVerdict::Partial {
                eprintln!("GitHub MCP unavailable, continuing");
            }
        }
        
        // Load Workspace adapter if enabled
        if let Some(workspace_cfg) = &config.mcp_workspace {
            let adapter = WorkspaceMcpAdapter::new(workspace_cfg.clone());
            let verdict = adapter.populate_engine_state(engine)?;
            if verdict == WpmVerdict::Partial {
                eprintln!("Workspace MCP unavailable, continuing");
            }
        }
        
        // Load Environment adapter if enabled
        if let Some(env_cfg) = &config.mcp_environment {
            let adapter = EnvironmentMcpAdapter::new(env_cfg.clone());
            let verdict = adapter.populate_engine_state(engine)?;
            if verdict == WpmVerdict::Partial {
                eprintln!("Environment MCP unavailable, continuing");
            }
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let config = CicdToml::load("cicd.toml")?;
    config.validate_mcp_config()?;
    
    let mut engine = EngineState::default();
    
    // Load built-in adapters (git, cargo metadata, etc.)
    load_builtin_adapters(&mut engine)?;
    
    // Load MCP adapters
    populate_engine_state_from_mcp(&mut engine, &config)?;
    
    // Continue with existing logic...
    Ok(())
}
```

---

## Part 2: Implementing an MCP Server

### 2.1 MCP Server Template (Rust)

An MCP server exposes tools and resources over JSON-RPC. Here's a minimal example in Rust:

Create `mcp-github-server/src/main.rs`:

```rust
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

type SharedState = Arc<RwLock<ServerState>>;

struct ServerState {
    cache: std::collections::HashMap<String, String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;
    
    let state = Arc::new(RwLock::new(ServerState {
        cache: std::collections::HashMap::new(),
    }));
    
    let app = axum::Router::new()
        .route("/", axum::routing::post(handle_request))
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    println!("GitHub MCP server listening on 0.0.0.0:{}", port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn handle_request(
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::Json(req): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(serde_json::json!({}));
    
    match method {
        "tools/list" => {
            return axum::Json(json!({
                "tools": [
                    {
                        "name": "get_branch_status",
                        "description": "Get branch protection and CI status",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "branch": { "type": "string" },
                                "repo": { "type": "string" },
                                "check_enforcement": { "type": "boolean" }
                            },
                            "required": ["branch", "repo"]
                        }
                    },
                    {
                        "name": "get_pr_metadata",
                        "description": "Get PR status and check results",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "pr_number": { "type": "integer" },
                                "repo": { "type": "string" }
                            },
                            "required": ["pr_number", "repo"]
                        }
                    }
                ]
            }));
        }
        
        "tools/call" => {
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
            
            match tool_name {
                "get_branch_status" => {
                    let branch = args.get("branch").and_then(|v| v.as_str()).unwrap_or("main");
                    let repo = args.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                    
                    // Fetch from GitHub API
                    let branch_status = fetch_github_branch_status(repo, branch)
                        .await
                        .unwrap_or_else(|_| serde_json::json!({
                            "branch": branch,
                            "protected": false,
                            "all_checks_passing": false
                        }));
                    
                    return axum::Json(json!({
                        "type": "tool_result",
                        "content": [
                            {
                                "type": "text",
                                "text": "Success"
                            },
                            {
                                "type": "text",
                                "text": branch_status.to_string()
                            }
                        ]
                    }));
                }
                
                "get_pr_metadata" => {
                    let pr_number = args.get("pr_number").and_then(|v| v.as_u64()).unwrap_or(0);
                    let repo = args.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let pr_metadata = fetch_github_pr_metadata(repo, pr_number as u32)
                        .await
                        .unwrap_or_else(|_| serde_json::json!({
                            "number": pr_number,
                            "state": "closed"
                        }));
                    
                    return axum::Json(json!({
                        "type": "tool_result",
                        "content": [
                            {
                                "type": "text",
                                "text": "Success"
                            },
                            {
                                "type": "text",
                                "text": pr_metadata.to_string()
                            }
                        ]
                    }));
                }
                
                _ => {
                    return axum::Json(json!({
                        "type": "error",
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Unknown tool: {}", tool_name)
                            }
                        ]
                    }));
                }
            }
        }
        
        _ => {
            return axum::Json(json!({
                "type": "error",
                "content": [
                    {
                        "type": "text",
                        "text": format!("Unknown method: {}", method)
                    }
                ]
            }));
        }
    }
}

async fn fetch_github_branch_status(
    repo: &str,
    branch: &str,
) -> anyhow::Result<serde_json::Value> {
    // Call GitHub API v3
    let token = std::env::var("GITHUB_TOKEN")?;
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://api.github.com/repos/{}/branches/{}",
        repo, branch
    );
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;
    
    let data: serde_json::Value = response.json().await?;
    
    Ok(json!({
        "branch": branch,
        "protected": data.get("protected").and_then(|v| v.as_bool()).unwrap_or(false),
        "all_checks_passing": true  // Simplified; would need to check commit status
    }))
}

async fn fetch_github_pr_metadata(
    repo: &str,
    pr_number: u32,
) -> anyhow::Result<serde_json::Value> {
    // Similar to branch status, fetch PR data from GitHub API
    let token = std::env::var("GITHUB_TOKEN")?;
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://api.github.com/repos/{}/pulls/{}",
        repo, pr_number
    );
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    let pr: serde_json::Value = response.json().await?;
    
    Ok(json!({
        "number": pr_number,
        "title": pr.get("title"),
        "state": pr.get("state"),
        "mergeable": pr.get("mergeable")
    }))
}
```

### 2.2 Dockerfile for MCP Server

Create `mcp-github-server/Dockerfile`:

```dockerfile
FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/app/target/release/mcp-github-server /usr/local/bin/

ENV PORT=3000
EXPOSE 3000

CMD ["mcp-github-server"]
```

### 2.3 Docker Compose Setup

Create `docker-compose.yml` for testing:

```yaml
version: '3'
services:
  cargo-cicd:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      CARGO_CICD_MCP_GITHUB_URL: "http://github-mcp:3000"
      CARGO_CICD_MCP_WORKSPACE_URL: "http://workspace-mcp:3001"
      CARGO_CICD_MCP_ENVIRONMENT_URL: "http://environment-mcp:3002"
    ports:
      - "3000:3000"
    depends_on:
      - github-mcp
      - workspace-mcp
      - environment-mcp

  github-mcp:
    build:
      context: ./mcp-github-server
      dockerfile: Dockerfile
    environment:
      PORT: 3000
      GITHUB_TOKEN: ${GITHUB_TOKEN}
    ports:
      - "3000:3000"

  workspace-mcp:
    build:
      context: ./mcp-workspace-server
      dockerfile: Dockerfile
    environment:
      PORT: 3001
    ports:
      - "3001:3001"

  environment-mcp:
    build:
      context: ./mcp-environment-server
      dockerfile: Dockerfile
    environment:
      PORT: 3002
    ports:
      - "3002:3002"
```

---

## Part 3: Testing MCP Integrations

### 3.1 Unit Tests for Adapters

Create `tests/mcp_adapters.rs`:

```rust
#[cfg(test)]
mod tests {
    use cargo_cicd::adapters::GitHubMcpAdapter;
    use cargo_cicd::cicd_toml::McpGitHubConfig;
    use cargo_cicd::engine::EngineState;
    use cargo_cicd::integrations::WpmVerdict;

    #[test]
    fn test_github_mcp_adapter_is_enabled() {
        let config = McpGitHubConfig {
            enabled: true,
            url: "http://localhost:3000".to_string(),
            repo: "test/repo".to_string(),
            verify_checks: vec![],
            require_approval: false,
            protect_main: false,
            cache_ttl_secs: 300,
        };
        
        let adapter = GitHubMcpAdapter::new(config);
        assert!(adapter.is_enabled());
        assert_eq!(adapter.name(), "github-mcp");
    }
    
    #[test]
    fn test_github_mcp_adapter_disabled() {
        let config = McpGitHubConfig {
            enabled: false,
            ..Default::default()
        };
        
        let adapter = GitHubMcpAdapter::new(config);
        assert!(!adapter.is_enabled());
    }
    
    #[tokio::test]
    async fn test_github_mcp_adapter_unavailable_returns_partial() {
        let config = McpGitHubConfig {
            enabled: true,
            url: "http://localhost:9999".to_string(),  // Non-existent server
            repo: "test/repo".to_string(),
            verify_checks: vec![],
            require_approval: false,
            protect_main: false,
            cache_ttl_secs: 300,
        };
        
        let adapter = GitHubMcpAdapter::new(config);
        let mut engine = EngineState::default();
        
        let verdict = adapter.populate_engine_state(&mut engine).unwrap();
        assert_eq!(verdict, WpmVerdict::Partial);
    }
}
```

### 3.2 Integration Tests with Mock Server

Create `tests/mcp_integration.rs`:

```rust
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use serde_json::json;

    #[tokio::test]
    async fn test_github_mcp_integration_with_mock_server() {
        // Start mock MCP server
        let server = MockGitHubServer::new(3001);
        let _handle = server.spawn();
        
        // Create adapter
        let config = McpGitHubConfig {
            enabled: true,
            url: "http://localhost:3001".to_string(),
            repo: "test/repo".to_string(),
            verify_checks: vec![],
            require_approval: false,
            protect_main: false,
            cache_ttl_secs: 300,
        };
        
        let adapter = GitHubMcpAdapter::new(config);
        let mut engine = EngineState::default();
        
        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Populate engine state
        let verdict = adapter.populate_engine_state(&mut engine).unwrap();
        
        assert_eq!(verdict, WpmVerdict::Pass);
    }
    
    struct MockGitHubServer {
        port: u16,
    }
    
    impl MockGitHubServer {
        fn new(port: u16) -> Self {
            Self { port }
        }
        
        fn spawn(self) -> tokio::task::JoinHandle<()> {
            let port = self.port;
            tokio::spawn(async move {
                let app = axum::Router::new()
                    .route("/", axum::routing::post(handle_mock_request));
                
                let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
                    .await
                    .unwrap();
                
                axum::serve(listener, app).await.unwrap();
            })
        }
    }
    
    async fn handle_mock_request(
        axum::Json(req): axum::Json<serde_json::Value>,
    ) -> axum::Json<serde_json::Value> {
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
        
        match method {
            "tools/call" => {
                return axum::Json(json!({
                    "type": "tool_result",
                    "content": [
                        { "type": "text", "text": "Success" },
                        {
                            "type": "text",
                            "text": json!({
                                "branch": "main",
                                "protected": true,
                                "all_checks_passing": true
                            }).to_string()
                        }
                    ]
                }));
            }
            _ => {
                return axum::Json(json!({
                    "type": "error",
                    "content": []
                }));
            }
        }
    }
}
```

### 3.3 Configuration Validation Tests

Create `tests/mcp_config_validation.rs`:

```rust
#[test]
fn test_mcp_config_validation_missing_url() {
    let config = toml::from_str::<CicdToml>(r#"
[mcp]
enabled = true

[mcp.github]
enabled = true
repo = "test/repo"
"#).unwrap();
    
    let result = config.validate_mcp_config();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("url is required"));
}

#[test]
fn test_mcp_config_validation_complete() {
    let config = toml::from_str::<CicdToml>(r#"
[mcp]
enabled = true

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "test/repo"
"#).unwrap();
    
    let result = config.validate_mcp_config();
    assert!(result.is_ok());
}

#[test]
fn test_mcp_disabled_skips_validation() {
    let config = toml::from_str::<CicdToml>(r#"
[mcp]
enabled = false

[mcp.github]
enabled = true
# Missing url and repo — but should be ignored
"#).unwrap();
    
    let result = config.validate_mcp_config();
    assert!(result.is_ok());
}
```

---

## Part 4: Distribution & Publishing

### 4.1 Publishing an MCP Adapter Crate

Create `Cargo.toml` for a standalone adapter:

```toml
[package]
name = "cargo-cicd-mcp-github"
version = "0.1.0"
edition = "2021"
authors = ["Your Organization"]
description = "GitHub MCP adapter for cargo-cicd"
repository = "https://github.com/yourorg/cargo-cicd-mcp-github"
license = "MIT OR Apache-2.0"

[dependencies]
cargo-cicd = { version = "26.6.2", features = ["contrib"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"

[dev-dependencies]
assert_cmd = "2"
tempfile = "3"
```

### 4.2 Publishing to crates.io

```bash
# Test locally
cargo test

# Build documentation
cargo doc --no-deps --open

# Publish to crates.io
cargo publish
```

### 4.3 GitHub Release

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run tests
        run: cargo test --release
      
      - name: Build release binary
        run: cargo build --release
      
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: target/release/mcp-github-server
          draft: false
          prerelease: false
```

---

## Part 5: Deployment Patterns

### 5.1 Local Development

```bash
# Terminal 1: Start all MCP servers
docker-compose up

# Terminal 2: Run cargo-cicd
cargo cicd status
```

### 5.2 Container Deployment

Deploy MCP servers as sidecar containers:

```yaml
# kubernetes deployment
apiVersion: v1
kind: Pod
metadata:
  name: cargo-cicd-with-mcp
spec:
  containers:
    - name: cargo-cicd
      image: seanchatmangpt/cargo-cicd:26.6.2
      env:
        - name: CARGO_CICD_MCP_GITHUB_URL
          value: "http://localhost:3000"
        - name: CARGO_CICD_MCP_WORKSPACE_URL
          value: "http://localhost:3001"
    
    - name: github-mcp
      image: seanchatmangpt/mcp-github:0.1.0
      env:
        - name: PORT
          value: "3000"
        - name: GITHUB_TOKEN
          valueFrom:
            secretKeyRef:
              name: github-credentials
              key: token
      ports:
        - containerPort: 3000
    
    - name: workspace-mcp
      image: seanchatmangpt/mcp-workspace:0.1.0
      env:
        - name: PORT
          value: "3001"
      ports:
        - containerPort: 3001
```

### 5.3 CI/CD Integration

```yaml
# GitHub Actions workflow
name: cargo-cicd Status

on: [push, pull_request]

jobs:
  status:
    runs-on: ubuntu-latest
    services:
      github-mcp:
        image: seanchatmangpt/mcp-github:latest
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-cicd
        run: cargo install cargo-cicd --features mcp-github
      
      - name: Check status
        env:
          CARGO_CICD_MCP_GITHUB_URL: "http://localhost:3000"
        run: cargo cicd status --check-push
```

---

## Part 6: Troubleshooting

### 6.1 MCP Server Connection Errors

**Symptom:** `Error: connection refused`

**Debugging:**
```bash
# Check if MCP server is running
curl http://localhost:3000/

# Check logs
docker logs <container_name>

# Test with longer timeout
export CARGO_CICD_MCP_TIMEOUT_SECS=60
cargo cicd status
```

### 6.2 Invalid Response Format

**Symptom:** `Failed to parse GitHub MCP response`

**Debugging:**
```bash
# Capture raw response
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{"method": "tools/call", "params": {"name": "get_branch_status", "arguments": {"branch": "main", "repo": "test/repo"}}}'

# Check MCP server implementation matches spec
```

### 6.3 Caching Issues

**Symptom:** Old data despite server update

**Solution:**
```toml
# Disable caching in cicd.toml
[mcp.github]
cache_ttl_secs = 0  # Cache disabled

# Or clear cache manually
rm -f .cargo-cicd-cache
```

---

## Checklist for Implementing New MCP Adapter

- [ ] Create adapter file in `src/adapters/`
- [ ] Implement `McpAdapter` trait
- [ ] Add configuration struct to `CicdToml`
- [ ] Add feature flag to `Cargo.toml`
- [ ] Update `src/adapters/mod.rs` with export
- [ ] Update `src/main.rs` to load adapter
- [ ] Write unit tests
- [ ] Write integration tests with mock server
- [ ] Update cicd.toml schema documentation
- [ ] Test graceful degradation (server unavailable)
- [ ] Test configuration validation
- [ ] Add to CHANGELOG.md

---

## Checklist for Implementing New MCP Server

- [ ] Define MCP tools and resources
- [ ] Implement HTTP server (Rust/Python/Node.js)
- [ ] Write tool implementations
- [ ] Handle authentication (GitHub token, etc.)
- [ ] Implement error handling and logging
- [ ] Create Dockerfile
- [ ] Write integration tests
- [ ] Document API in README.md
- [ ] Create example docker-compose.yml
- [ ] Publish to container registry
- [ ] Publish to GitHub Releases

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-14
