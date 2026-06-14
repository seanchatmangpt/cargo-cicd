# MCP Server Integration Design Documents

This directory contains the complete design for integrating Model Context Protocol (MCP) servers into cargo-cicd.

## Documents

### 1. **MCP_STRATEGY_SUMMARY.md** ⭐ START HERE
- **Length:** ~387 lines (5-10 minute read)
- **Purpose:** Executive summary, roadmap, quick reference
- **Read if:** You want to understand what we're building, when, and why
- **Contains:**
  - Architecture overview (diagram)
  - Key design decisions with rationale
  - Implementation phases (Phase 1-5)
  - Success criteria
  - Quick reference for finding information in other docs

### 2. **MCP_INTEGRATION_STRATEGY.md** 
- **Length:** ~1,179 lines (30-45 minute read)
- **Purpose:** Complete strategic and design documentation
- **Read if:** You're reviewing the design, making architectural decisions, or need the full rationale
- **Contains:**
  - Part 1: External MCP servers to integrate (GitHub, workspace, environment)
  - Part 2: Custom MCP servers (wasm4pm oracle, Rust docs, policy engine)
  - Part 3: Plugin architecture
  - Part 4: Configuration format (cicd.toml)
  - Part 5: Implementation roadmap
  - Part 6: Design rationale (why each decision)
  - Part 7: Security considerations
  - Part 8: Testing strategy
  - Part 9: End-to-end example
  - Part 10: Backward compatibility

### 3. **MCP_SCHEMA_REFERENCE.md**
- **Length:** ~1,327 lines (reference document)
- **Purpose:** Technical specification and schemas
- **Read if:** You're implementing adapters/servers or validating configurations
- **Contains:**
  - MCP configuration schema (full cicd.toml structure)
  - MCP server tool schemas (request/response formats)
  - MCP server resource schemas
  - cicd.toml examples (minimal, full, GitHub-only, offline)
  - Adapter Rust types (data structures)
  - Test fixtures (mock servers, config examples)
  - Configuration validation code
  - Quick reference checklist

### 4. **MCP_IMPLEMENTATION_GUIDE.md**
- **Length:** ~1,070 lines (hands-on guide)
- **Purpose:** Step-by-step implementation instructions with code examples
- **Read if:** You're implementing adapters or MCP servers
- **Contains:**
  - Part 1: Anatomy of an MCP adapter (with GitHub example)
  - Part 2: Anatomy of an MCP server (with GitHub example in Rust/axum)
  - Part 3: Testing strategies (unit, integration, configuration)
  - Part 4: Distribution & publishing (crates.io, GitHub releases)
  - Part 5: Deployment patterns (local, Docker, Kubernetes, CI/CD)
  - Part 6: Troubleshooting guide
  - Implementation checklists

## Reading Guide by Role

### Architect / Lead Designer
1. **MCP_STRATEGY_SUMMARY.md** (15 min) — Get the big picture
2. **MCP_INTEGRATION_STRATEGY.md** Part 1-2 (20 min) — Understand scope
3. **MCP_INTEGRATION_STRATEGY.md** Part 6 (10 min) — Review design decisions

**Time: 45 minutes**

### Project Manager / Tech Lead
1. **MCP_STRATEGY_SUMMARY.md** (10 min)
2. **MCP_STRATEGY_SUMMARY.md** Implementation Phases section (5 min)
3. **MCP_INTEGRATION_STRATEGY.md** Part 5 (10 min)

**Time: 25 minutes**

### Backend Developer (Implementing Phase 1)
1. **MCP_STRATEGY_SUMMARY.md** (10 min)
2. **MCP_IMPLEMENTATION_GUIDE.md** Part 1 (20 min) — Adapter template
3. **MCP_IMPLEMENTATION_GUIDE.md** Part 2 (20 min) — Server template
4. **MCP_IMPLEMENTATION_GUIDE.md** Part 3 (15 min) — Tests
5. **MCP_SCHEMA_REFERENCE.md** (Reference) — Schemas as needed

**Time: 65 minutes + reference lookups**

### DevOps / Operations
1. **MCP_STRATEGY_SUMMARY.md** Architecture overview (5 min)
2. **MCP_IMPLEMENTATION_GUIDE.md** Part 5 (20 min) — Deployment patterns
3. **MCP_SCHEMA_REFERENCE.md** Configuration examples (10 min)

**Time: 35 minutes**

### Code Reviewer
1. **MCP_SCHEMA_REFERENCE.md** (10 min) — Expected schemas
2. **MCP_IMPLEMENTATION_GUIDE.md** Part 3 (10 min) — Test patterns
3. Review code against templates in MCP_IMPLEMENTATION_GUIDE.md

**Time: 20 minutes + code review**

### MCP Server Author (Third-Party)
1. **MCP_INTEGRATION_STRATEGY.md** Part 2 (10 min)
2. **MCP_SCHEMA_REFERENCE.md** Tool/resource schemas (15 min)
3. **MCP_IMPLEMENTATION_GUIDE.md** Part 2 (20 min) — Server template
4. **MCP_IMPLEMENTATION_GUIDE.md** Part 4 (10 min) — Publishing

**Time: 55 minutes**

## Quick Reference

### "How do I...?"

| Question | Document | Section |
|----------|----------|---------|
| Understand what we're building? | MCP_STRATEGY_SUMMARY.md | Overview & Phases |
| See the architecture? | MCP_STRATEGY_SUMMARY.md | Architecture Overview (diagram) |
| Write an MCP adapter? | MCP_IMPLEMENTATION_GUIDE.md | Part 1 (with GitHub example) |
| Write an MCP server? | MCP_IMPLEMENTATION_GUIDE.md | Part 2 (with GitHub example) |
| Know what cicd.toml looks like? | MCP_SCHEMA_REFERENCE.md | MCP Configuration Schema |
| Configure GitHub integration? | MCP_SCHEMA_REFERENCE.md | Examples: GitHub-Only |
| Test an adapter? | MCP_IMPLEMENTATION_GUIDE.md | Part 3 (Unit/Integration) |
| Deploy MCP servers? | MCP_IMPLEMENTATION_GUIDE.md | Part 5 (Deployment Patterns) |
| Know the design rationale? | MCP_INTEGRATION_STRATEGY.md | Part 6 (Design Decisions) |
| See error troubleshooting? | MCP_IMPLEMENTATION_GUIDE.md | Part 6 (Troubleshooting) |

### "Why was X designed this way?"

| Topic | Document | Section |
|-------|----------|---------|
| Why MCP servers (not native libraries)? | MCP_INTEGRATION_STRATEGY.md | Part 6: Decision 1 |
| Why feature flags? | MCP_INTEGRATION_STRATEGY.md | Part 6: Decision 2 |
| Why extend cicd.toml? | MCP_INTEGRATION_STRATEGY.md | Part 6: Decision 3 |
| Why graceful degradation? | MCP_INTEGRATION_STRATEGY.md | Part 6: Decision 4 |
| Why separate policy engine? | MCP_INTEGRATION_STRATEGY.md | Part 6: Decision 5 |
| Security model? | MCP_INTEGRATION_STRATEGY.md | Part 7: Security |

## Document Statistics

| Document | Lines | Read Time | Type |
|----------|-------|-----------|------|
| MCP_STRATEGY_SUMMARY.md | 387 | 10 min | Executive summary |
| MCP_INTEGRATION_STRATEGY.md | 1,179 | 40 min | Strategy & design |
| MCP_SCHEMA_REFERENCE.md | 1,327 | Reference | Specifications |
| MCP_IMPLEMENTATION_GUIDE.md | 1,070 | 30 min | How-to guide |
| **Total** | **3,963** | **80 min** | Complete package |

## Key Concepts

### MCP Adapter
A Rust struct in `src/adapters/` that:
- Holds configuration from `cicd.toml`
- Makes HTTP calls to an MCP server
- Translates responses into `EngineState` fields
- Implements the `McpAdapter` trait

### EngineState
Cargo-cicd's single source of truth containing all runtime dimensions:
- `WorkspaceState`, `ToolchainState`, `TargetState`
- `GitPhaseState`, `TestPlanState`, etc.
- All adapters (including MCP) populate this

### Feature Flags
Cargo features that control which MCP adapters are compiled:
- `mcp-github` — GitHub API integration
- `mcp-workspace` — Workspace introspection
- `mcp-environment` — Environment inspection
- `mcp-rustdoc` — Rust documentation
- `mcp-wasm4pm` — wasm4pm oracle
- `mcp-policy` — Policy suggestion engine

### Graceful Degradation
When an MCP server is unavailable, adapters return `WpmVerdict::Partial` and cargo-cicd continues with reduced capability (not failure).

### Plugin System
Third-party developers can register custom adapters/policies by:
1. Creating a plugin crate
2. Implementing `register()` hook
3. Declaring in `cicd.toml [plugins]`

## Implementation Timeline

| Phase | Release | Focus | Duration |
|-------|---------|-------|----------|
| 1 | v26.6.3 | Foundation (GitHub) | 2 weeks |
| 2 | v26.6.4 | Ecosystem (workspace, env) | 2 weeks |
| 3 | v26.6.5 | Advanced (rustdoc, wasm4pm) | 2 weeks |
| 4 | v26.6.6 | LLM & plugins | 3 weeks |
| 5 | v26.6.7 | Hardening & docs | 2 weeks |

See MCP_STRATEGY_SUMMARY.md for details on each phase.

## Success Criteria

- ✓ cargo-cicd works offline (MCP disabled)
- ✓ MCP servers are optional (feature-gated)
- ✓ Graceful degradation (server unavailable = reduced capability, not failure)
- ✓ Plugin system (third-party can extend)
- ✓ Comprehensive testing (90%+ adapter coverage)
- ✓ Full documentation (these 4 documents)

## File Locations in Codebase

After implementation, you'll find:

```
cargo-cicd/
├── src/
│   ├── adapters/
│   │   ├── github_mcp.rs (Phase 1)
│   │   ├── workspace_mcp.rs (Phase 2)
│   │   ├── environment_mcp.rs (Phase 2)
│   │   ├── rustdoc_mcp.rs (Phase 3)
│   │   └── policy_suggestion_mcp.rs (Phase 4)
│   ├── integrations/
│   │   └── wasm4pm_mcp.rs (Phase 3)
│   └── plugins/ (Phase 4)
│
├── tests/
│   ├── mcp_adapters.rs
│   ├── mcp_integration.rs
│   └── mcp_config_validation.rs
│
├── mcp-github-server/ (reference implementation)
├── mcp-workspace-server/
├── mcp-environment-server/
├── mcp-rustdoc-server/
└── mcp-policy-engine/
```

## Contributing New MCP Servers

See **MCP_IMPLEMENTATION_GUIDE.md Part 2** for the template.

Key points:
1. Implement tools and resources per **MCP_SCHEMA_REFERENCE.md**
2. Create adapter in `src/adapters/` or as plugin
3. Add feature flag to `Cargo.toml`
4. Write tests (unit + integration)
5. Document in README/comments

## Questions or Feedback?

These documents are comprehensive but not perfect. If you find gaps:

1. **Clarification needed?** Check the Quick Reference table above
2. **Schema question?** MCP_SCHEMA_REFERENCE.md has the answer
3. **Implementation stuck?** MCP_IMPLEMENTATION_GUIDE.md examples
4. **Design question?** MCP_INTEGRATION_STRATEGY.md rationale (Part 6)

---

**Version:** 1.0  
**Date:** 2026-06-14  
**Status:** Ready for Review and Implementation  
**Total Lines of Documentation:** 3,963  
**Total Read Time (all documents):** ~80 minutes
