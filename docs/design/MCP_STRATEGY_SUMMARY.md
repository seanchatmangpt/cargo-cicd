# MCP Integration Strategy — Executive Summary

**Version:** 1.0  
**Date:** 2026-06-14  
**Status:** Ready for Review and Implementation  
**Target Release:** v26.6.3 (Phase 1: Foundation)

---

## What This Is

A comprehensive design for integrating Model Context Protocol (MCP) servers into cargo-cicd, transforming it from a local-only Rust CI/CD tool into a networked orchestrator that can coordinate with GitHub, Rust ecosystem tools, and custom enterprise systems.

---

## Three Documents You Need to Read

### 1. **MCP_INTEGRATION_STRATEGY.md** (40 KB)
**Read this first.** Covers the big picture:
- **Part 1:** Which external MCP servers to integrate (GitHub API, workspace introspection, environment inspection, wasm4pm oracle)
- **Part 2:** Custom MCP servers designed for cargo-cicd (wasm4pm oracle, Rust docs lookup, policy suggestion engine)
- **Part 3:** Plugin architecture for third-party MCP extensions
- **Part 4:** Configuration format (`[mcp.*]` sections in cicd.toml) with examples
- **Part 5-8:** Implementation roadmap, design rationale, security considerations, testing strategy
- **Part 9:** End-to-end example showing information flow

**Key Takeaway:** MCP servers integrate via adapters (a pattern cargo-cicd already uses). Each MCP server is optional, feature-gated, and degradable if unavailable.

### 2. **MCP_SCHEMA_REFERENCE.md** (29 KB)
**Read this while implementing.** Contains:
- Full cicd.toml schema with all `[mcp.*]` sections
- JSON schemas for all MCP tool requests/responses (GitHub, workspace, environment, wasm4pm, rustdoc, policy)
- Rust data structures (config, events, adapters)
- Test fixtures (mock servers, configuration examples)
- Validation rules

**Key Takeaway:** This is the specification. Use it to validate configurations and ensure adapters/servers conform.

### 3. **MCP_IMPLEMENTATION_GUIDE.md** (29 KB)
**Read this when you code.** Step-by-step instructions:
- How to write an MCP adapter (example: GitHub MCP adapter)
- How to write an MCP server (example: GitHub MCP server in Rust with axum)
- Testing patterns (unit tests, integration tests with mock servers)
- Deployment patterns (local, container, Kubernetes, CI/CD)
- Troubleshooting guide

**Key Takeaway:** Copy the templates, customize for your domain, run the tests.

---

## Quick Architecture Overview

```
cargo-cicd v26.6.3+
├── Existing Adapters (local-only)
│   ├── GitStatusAdapter → git status --porcelain
│   ├── CargoMetadataAdapter → cargo metadata --format json
│   ├── TargetScannerAdapter → du -sh target/
│   └── ToolchainDetector → rust-toolchain.toml
│
└── NEW: MCP Adapters (networked, optional)
    ├── GitHubMcpAdapter (feature: mcp-github)
    │   └── → MCP Server → GitHub API
    │
    ├── WorkspaceMcpAdapter (feature: mcp-workspace)
    │   └── → MCP Server → dependency graph, advisories
    │
    ├── EnvironmentMcpAdapter (feature: mcp-environment)
    │   └── → MCP Server → system resources, toolchains
    │
    ├── Wasm4pmMcpAdapter (feature: mcp-wasm4pm)
    │   └── → MCP Server → wpm oracle
    │
    ├── RustdocMcpAdapter (feature: mcp-rustdoc)
    │   └── → MCP Server → breaking change detection
    │
    └── PolicySuggestionAdapter (feature: mcp-policy)
        └── → MCP Server → LLM policy explanations

All adapters → EngineState (single source of truth)
EngineState → Policies → User output & cicd.toml events
```

---

## Key Design Decisions (and Why)

### Decision 1: Why MCP Servers (Not Native Rust Libraries)?
- **Isolation:** Crash in one server doesn't crash cargo-cicd
- **Language Agnostic:** Users can write servers in Python, Go, JavaScript
- **Versioning:** Update servers without rebuilding cargo-cicd
- **Graceful Degradation:** If server is down, cargo-cicd continues with reduced capability

### Decision 2: Why Feature Flags?
- **Binary Size:** Minimal cargo-cicd stays < 10 MB
- **Supply Chain:** Users who don't need GitHub integration don't carry that code
- **Discoverability:** Clear which integrations are available

### Decision 3: Why Extend cicd.toml (Not a Separate File)?
- **Single Source of Truth:** All cargo-cicd config in one place
- **Gitignore:** Already excluded from VCS
- **Consistency:** Reuses validation and documentation

### Decision 4: Why Graceful Degradation (Not Hard Fail)?
- **Local-First Philosophy:** cargo-cicd works offline
- **CI Resilience:** External services fail; cargo-cicd shouldn't
- **User Experience:** Run on planes, in VPNs, when servers are down

---

## Concrete Example: Adding GitHub Integration

### Step 1: User Configuration (cicd.toml)
```toml
[mcp]
enabled = true

[mcp.github]
enabled = true
url = "http://localhost:3000"
repo = "seanchatmangpt/cargo-cicd"
verify_checks = ["cargo-test", "cargo-clippy"]
require_approval = true
```

### Step 2: Adapter Code (src/adapters/github_mcp.rs)
```rust
pub struct GitHubMcpAdapter { config: McpGitHubConfig }

impl McpAdapter for GitHubMcpAdapter {
    fn populate_engine_state(&self, state: &mut EngineState) -> Result<WpmVerdict> {
        // HTTP POST to MCP server
        // Parse response
        // Update state.git_phase.ci_checks_passing, etc.
        // Return WpmVerdict::Pass or WpmVerdict::Partial (if server unavailable)
    }
}
```

### Step 3: MCP Server (mcp-github-server/src/main.rs)
```rust
// Expose tools: get_branch_status, get_pr_metadata
// Call GitHub API v3
// Return JSON responses
```

### Step 4: Startup Integration (src/main.rs)
```rust
if config.mcp.enabled {
    if let Some(github_cfg) = &config.mcp_github {
        let adapter = GitHubMcpAdapter::new(github_cfg.clone());
        adapter.populate_engine_state(&mut engine)?;
    }
}
```

### Step 5: User Experience
```bash
$ cargo cicd status
✓ git.phase: clean (from git status --porcelain)
✓ github.checks: passing (from GitHub MCP server)
✓ github.approval: approved (from GitHub MCP server)
✅ Safe to push!
```

---

## Implementation Phases

### Phase 1: Foundation (v26.6.3) — 2 weeks
**Goal:** Prove the pattern works; GitHub integration only

- [ ] Add `[mcp]` section to cicd.toml schema
- [ ] Implement `GitHubMcpAdapter`
- [ ] Create GitHub MCP server reference implementation
- [ ] Add `mcp-github` feature flag
- [ ] Write comprehensive tests

**Deliverables:**
- cargo-cicd compiles with/without `mcp-github` feature
- Tests pass with MCP server running and unavailable
- Documentation: MCP_INTEGRATION_STRATEGY.md (already done)

### Phase 2: Ecosystem (v26.6.4) — 2 weeks
**Goal:** Complete the triumvirate (workspace, environment)

- [ ] Implement `WorkspaceMcpAdapter`
- [ ] Implement `EnvironmentMcpAdapter`
- [ ] Create workspace & environment MCP server reference implementations
- [ ] Update test suite for multi-server scenarios

**Deliverables:**
- 3 MCP adapters working together
- Example docker-compose.yml with all three servers
- Integration tests showing dependency impact analysis

### Phase 3: Advanced (v26.6.5) — 2 weeks
**Goal:** Support sophisticated use cases (breaking changes, oracle integration)

- [ ] Implement `RustdocMcpAdapter`
- [ ] Implement `Wasm4pmMcpAdapter` (replace shell-out)
- [ ] Update wasm4pm tests to use MCP

**Deliverables:**
- Breaking change detection in transitive deps
- Structured wasm4pm oracle access

### Phase 4: LLM & Plugins (v26.6.6) — 3 weeks
**Goal:** Make cargo-cicd user-friendly (explanations, extensions)

- [ ] Implement `PolicySuggestionAdapter`
- [ ] Implement plugin loader
- [ ] Create plugin template

**Deliverables:**
- Policy verdicts include "why" (LLM explanation)
- Third-party developers can add custom adapters/policies

### Phase 5: Hardening (v26.6.7) — 2 weeks
**Goal:** Production readiness

- [ ] Security audit (MCP server trust model)
- [ ] Performance tuning (caching, connection pooling)
- [ ] Comprehensive documentation
- [ ] CLI command: `cargo cicd mcp status`

**Deliverables:**
- Production-grade documentation
- CLI dashboard for MCP server health

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| MCP server crashes | Process isolation; graceful degradation (WpmVerdict::Partial) |
| Network latency | Configurable timeouts; caching in cicd.toml |
| MCP spec changes | Version pinning in feature flags; adapter trait is stable |
| User confusion | Comprehensive docs; `cargo cicd mcp status` CLI command |
| Supply chain | Feature flags allow minimal-install (no MCP code if not needed) |

---

## Success Criteria

### Functional
- [ ] cargo-cicd works identically with MCP disabled (local-only mode)
- [ ] MCP servers can be swapped without recompiling cargo-cicd
- [ ] Graceful degradation: server unavailable = reduced capability, not failure
- [ ] Plugin system supports third-party adapters

### Non-Functional
- [ ] Minimal install (no features) < 10 MB binary
- [ ] Full install (all features) < 50 MB binary
- [ ] MCP call latency < 5 seconds (user-facing)
- [ ] Evidence audit trail in cicd.toml [[events]]

### Testing
- [ ] 90%+ test coverage for adapters
- [ ] Integration tests with mock MCP servers
- [ ] Feature flag tests ensure no cross-contamination
- [ ] Graceful degradation tests for each adapter

---

## File Structure After Implementation

```
cargo-cicd/
├── docs/design/
│   ├── MCP_INTEGRATION_STRATEGY.md      (This strategy document)
│   ├── MCP_SCHEMA_REFERENCE.md          (Configuration & schema specs)
│   ├── MCP_IMPLEMENTATION_GUIDE.md      (Step-by-step implementation)
│   └── MCP_STRATEGY_SUMMARY.md          (This file)
│
├── src/
│   ├── main.rs                          (Updated to load MCP adapters)
│   ├── cicd_toml.rs                     (Updated with McpConfig structs)
│   ├── adapters/
│   │   ├── mod.rs                       (Export McpAdapter trait)
│   │   ├── github_mcp.rs                (NEW)
│   │   ├── workspace_mcp.rs             (NEW)
│   │   ├── environment_mcp.rs           (NEW)
│   │   ├── rustdoc_mcp.rs               (NEW, Phase 3)
│   │   └── policy_suggestion_mcp.rs     (NEW, Phase 4)
│   ├── integrations/
│   │   └── wasm4pm_mcp.rs               (NEW, Phase 3)
│   └── plugins/                         (NEW, Phase 4)
│       └── loader.rs
│
├── tests/
│   ├── mcp_adapters.rs                  (Unit tests for adapters)
│   ├── mcp_integration.rs               (Integration tests with mock servers)
│   ├── mcp_config_validation.rs         (Configuration validation)
│   └── fixtures/
│       └── mcp_configs.rs               (Test configurations)
│
├── mcp-github-server/                   (NEW reference implementation)
├── mcp-workspace-server/                (NEW reference implementation)
├── mcp-environment-server/              (NEW reference implementation)
├── mcp-rustdoc-server/                  (NEW, Phase 3)
├── mcp-policy-engine/                   (NEW, Phase 4)
│
└── Cargo.toml                           (Updated with features)
    [features]
    mcp-github = []
    mcp-workspace = []
    mcp-environment = []
    mcp-rustdoc = []
    mcp-wasm4pm = ["process-data"]
    mcp-policy = ["autonomic"]
    mcp-all = ["mcp-github", "mcp-workspace", "mcp-environment", "mcp-rustdoc", "mcp-wasm4pm", "mcp-policy"]
```

---

## How to Use These Documents

### For Architecture Review
1. Read this summary (5 min)
2. Read MCP_INTEGRATION_STRATEGY.md Part 1-2 (15 min)
3. Review design decisions (Part 6) and rationale (Part 6)

### For Implementation Planning
1. Read Phase 1 in this summary (2 min)
2. Review MCP_SCHEMA_REFERENCE.md for cicd.toml format (5 min)
3. Read MCP_IMPLEMENTATION_GUIDE.md Part 1-2 for adapter/server templates (15 min)
4. Start coding with Part 1.2 (GitHub Adapter example) as reference

### For Testing
1. Review MCP_IMPLEMENTATION_GUIDE.md Part 3 (15 min)
2. Copy test templates from MCP_SCHEMA_REFERENCE.md (5 min)
3. Adapt for your adapter/server

### For Operations/DevOps
1. Read MCP_IMPLEMENTATION_GUIDE.md Part 5 (Deployment Patterns) (10 min)
2. Use provided docker-compose.yml and Kubernetes manifests

---

## Next Steps

1. **Code Review:** Have architecture team review MCP_INTEGRATION_STRATEGY.md
2. **Scope Approval:** Confirm Phase 1 scope with stakeholders
3. **Repository Setup:** Create branches for MCP adapter work
4. **Dependency Updates:** Add `reqwest`, `tokio`, `axum` to Cargo.toml (if not already present)
5. **Kickoff:** Schedule implementation kickoff with feature flag owners

---

## Document Locations

- **MCP_INTEGRATION_STRATEGY.md** — Strategic overview and design rationale
- **MCP_SCHEMA_REFERENCE.md** — Technical specifications and schemas
- **MCP_IMPLEMENTATION_GUIDE.md** — Hands-on implementation guide
- **MCP_STRATEGY_SUMMARY.md** — This file; executive summary and quick reference

---

## Questions?

Refer to:
- **"Why X?"** → MCP_INTEGRATION_STRATEGY.md Part 6 (Rationale)
- **"What's the schema for Y?"** → MCP_SCHEMA_REFERENCE.md
- **"How do I implement Z?"** → MCP_IMPLEMENTATION_GUIDE.md
- **"What's the roadmap?"** → This file, Implementation Phases

---

## Appendix: Glossary

- **MCP:** Model Context Protocol — standard for tool/resource sharing
- **Adapter:** Rust struct that translates external data into EngineState
- **EngineState:** Single source of truth; aggregate root of all runtime dimensions
- **cicd.toml:** Configuration and state file written to workspace root
- **WpmVerdict:** Enumeration (Pass, Warn, Fail, Partial, NotAvailable)
- **Feature Flag:** Rust cargo feature; controls what code is compiled
- **Graceful Degradation:** System continues with reduced capability if MCP unavailable
- **Plugin:** Third-party code that registers custom adapters/policies

---

**Prepared by:** Claude (claude.ai/code)  
**Review Status:** Ready for Technical Review  
**Last Updated:** 2026-06-14  
**Next Review:** After Phase 1 implementation (2026-07-14)
