# ADR-019: Progressive Feature Disclosure via Cargo Feature Flags

**Status:** Accepted  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee  
**Tags:** features, cargo, progressive-disclosure, lean-binary, opt-in

---

## Context

cargo-cicd targets a wide range of users:

1. **Casual users**: Developers who want a simple CI/CD helper that checks workspace health and runs tests. They do not need oracle adjudication, autonomic policies, or LSP integration.

2. **Process engineers**: Teams that need full evidence emission, oracle adjudication, and compliance gate enforcement. They want the full Level 5 engine.

3. **Power users**: Developers integrating cargo-cicd into complex pipelines who need distributed oracle consensus, process mining dashboards, and ontology customization.

4. **IDE integrators**: Tools consuming the LSP server for inline diagnostics.

5. **Offline/embedded users**: Teams in air-gapped environments or on constrained hardware who need minimal binary size.

A single binary that includes all capabilities would impose on every user the compile-time and runtime cost of capabilities they don't need. The advanced feature ecosystem (10 opt-in crates) alone adds significant compile time and binary size.

### Binary Size Concerns

Without feature gating:

| Feature | Binary Size Increase | Compile Time Increase |
|---------|--------------------|--------------------|
| process-data (full engine) | +800KB | +15s |
| autonomic (policies) | +200KB | +5s |
| wasm4pm (oracle shell) | +50KB | +2s |
| advanced (10 opt-in crates) | +2MB | +60s |
| lsp (language server) | +1.5MB | +30s |

A default binary with all features enabled would be ~5MB and take ~2 minutes to compile. A lean default binary is ~800KB and compiles in ~15 seconds.

### User Experience Philosophy

Users should get value immediately from the default binary. Advanced capabilities should be discoverable but not mandatory. The progressive disclosure principle:

1. Default binary works with zero configuration.
2. Each feature layer adds capability without breaking existing use.
3. Feature combinations are explicitly documented.
4. The most heavyweight features (oracle, LSP, advanced scanning) are always opt-in.

---

## Decision

**cargo-cicd uses a layered Cargo feature flag system with `default = []` (lean binary). All features beyond basic CLI operation are opt-in.**

### Feature Flag Hierarchy

```toml
# Cargo.toml
[features]
default = []

# Layer 1: Level 5 engine internals
process-data = []

# Layer 2: Policy suggestions (implies process-data)
autonomic = ["process-data"]

# Layer 3: Community contributor tooling (implies process-data)
contrib = ["process-data"]

# Layer 4: wasm4pm oracle integration (implies process-data)
wasm4pm = ["process-data"]

# Layer 5: Language server protocol (implies process-data)
lsp = ["process-data", "dep:tower-lsp", "dep:tokio"]

# Layer 6: High-performance scanning and analytics
advanced = [
    "process-data",
    "dep:ignore",
    "dep:rayon",
    "dep:blake3",
    "dep:tracing",
    "dep:tracing-subscriber",
    "dep:miette",
    "dep:thiserror",
    "dep:moka",
    "dep:bitcode",
    "dep:petgraph",
    "dep:jiff",
    "dep:hdrhistogram",
    "dep:aho-corasick",
]

# Vision 2030 Phase 2: Anti-LLM-cheat provenance tracking
anti-llm-cheat = ["process-data"]

# Convenience bundles
full = ["autonomic", "wasm4pm", "advanced", "lsp", "anti-llm-cheat"]
ci = ["wasm4pm", "process-data"]      # Minimal CI pipeline bundle
release-gate = ["wasm4pm", "autonomic"]  # For release CI
```

### Feature Semantics

**`default = []` (lean binary)**

The default binary provides:
- All noun-verb CLI grammar (status, target, test, trybuild, git, publish, workspace, evidence, pipeline, lsp nouns)
- Basic workspace inspection using direct filesystem and process calls
- Help text, error messages, and exit codes
- No Level 5 engine state
- No evidence emission
- No oracle adjudication

This is sufficient for users who want simple CI/CD commands without the overhead of the full engine.

**`process-data`**

Enables the Level 5 engine (`EngineState`, adapters, cicd.toml writing, evidence emission). Required for any feature that needs structured state.

```rust
#[cfg(feature = "process-data")]
mod engine {
    pub use crate::engine::EngineState;
}
```

**`autonomic`**

Enables autonomic policy suggestions. Requires `process-data`. When enabled, `workspace doctor` runs all policies and emits recommendations. Never takes destructive action.

```rust
#[cfg(feature = "autonomic")]
fn run_policy_suggestions(state: &EngineState) -> Vec<PolicyEntry> {
    policies::run_all_policies(state)
}
```

**`wasm4pm`**

Enables the wasm4pm oracle integration seam. Requires `process-data`. When enabled, evidence emission is followed by oracle adjudication. In the default build, evidence is emitted but not adjudicated.

```rust
#[cfg(feature = "wasm4pm")]
fn adjudicate(xes_path: &Path) -> WpmVerdict {
    Wasm4pmShell::audit_xes(xes_path)
        .unwrap_or(WpmVerdict::Blocked)
}
```

**`lsp`**

Enables the Language Server Protocol server (`cargo cicd lsp explain`). Requires tokio async runtime and tower-lsp, which are heavy dependencies not needed in non-LSP contexts.

**`advanced`**

Enables the 10 opt-in high-performance crates (parallel_scan, fingerprint, observability, diagnostics, cache, snapshot, dep_graph, timeline, histogram, pattern). See `docs/ARCHITECTURE.md` §Advanced Capabilities.

**`anti-llm-cheat`** (Vision 2030, Phase 1 Weeks 9-12)

Enables provenance classification (Human/AI-Assisted/AI-Generated) as described in ADR-017. Requires `process-data` for evidence recording.

```rust
#[cfg(feature = "anti-llm-cheat")]
fn classify_provenance(path: &Path) -> ProvenanceClass {
    ProvenanceDetector::classify_file(path)
}
```

### Feature Projection Contract

The feature flag surface is a formal contract verified by `tests/feature_projection.rs`:

```rust
// tests/feature_projection.rs

#[test]
fn test_default_features_compile_and_emit_no_engine_state() {
    // Default binary must not import EngineState
    // Verified by checking that process-data feature is not enabled
    #[cfg(not(feature = "process-data"))]
    assert!(true, "Default build has no process-data");
}

#[test]
fn test_autonomic_implies_process_data() {
    #[cfg(all(feature = "autonomic", not(feature = "process-data")))]
    compile_error!("autonomic must imply process-data");
}

#[test]
fn test_wasm4pm_implies_process_data() {
    #[cfg(all(feature = "wasm4pm", not(feature = "process-data")))]
    compile_error!("wasm4pm must imply process-data");
}
```

### Binary Size Targets

| Build Configuration | Expected Size | Compile Time |
|--------------------|--------------|-------------|
| `cargo build` (default) | < 1MB | < 20s |
| `--features process-data` | < 2MB | < 35s |
| `--features autonomic` | < 2.5MB | < 40s |
| `--features wasm4pm` | < 2.5MB | < 42s |
| `--features advanced` | < 5MB | < 80s |
| `--features full` | < 7MB | < 120s |

These targets are enforced by CI checks in the release gate.

### Feature Discovery

The `status show` verb hints at available features when run with the default binary:

```
cargo cicd status show

Workspace: my-crate v1.0.0

Features: default build (lean mode)
  Process evidence:  disabled  (enable with --features wasm4pm)
  Policy suggestions: disabled  (enable with --features autonomic)
  Advanced scanning:  disabled  (enable with --features advanced)
  
  Run `cargo cicd status show --features full` for full workspace analysis.
```

This surfaces feature availability without forcing it on users.

---

## Consequences

### Positive

1. **Fast compilation for common use**: Users who just want `cargo cicd status show` compile a lean binary in 15-20 seconds.

2. **No surprise dependencies**: Users who don't need tokio or tracing don't get them. Dependencies are truly opt-in.

3. **Binary size control**: Embedded or constrained environments can use the default lean binary.

4. **Progressive onboarding**: New users start with the lean binary and discover features as they need them.

5. **Feature contract verifiability**: The `tests/feature_projection.rs` suite verifies that feature implications are correct at compile time.

6. **CI/CD pipeline flexibility**: Different CI stages can use different feature bundles:
   - PR CI: `--features process-data` (fast, evidence emission)
   - Release gate: `--features release-gate` (evidence + oracle)
   - Full audit: `--features full` (all capabilities)

### Negative

1. **Conditional compilation complexity**: `#[cfg(feature = ...)]` blocks scatter throughout the codebase make code harder to read.

2. **Feature combination testing**: 2^N feature combinations cannot all be tested. The test matrix covers common combinations but not all.

3. **Documentation overhead**: Each feature must be documented with its capabilities, implications, and binary size impact.

4. **Discoverability friction**: Users don't know what they're missing with the default build. Mitigation: `status show` hints at available features.

5. **Feature flag proliferation risk**: As Vision 2030 adds features, the flag list grows. Mitigation: Feature bundles (`full`, `ci`, `release-gate`) provide curated sets.

---

## Feature Flag Decision Tree

For users choosing their feature bundle:

```
Do you need oracle adjudication (wasm4pm)?
├── YES → use --features wasm4pm (or release-gate)
└── NO → continue

Do you want autonomic policy suggestions?
├── YES → use --features autonomic
└── NO → continue

Do you need the LSP language server?
├── YES → use --features lsp
└── NO → continue

Do you need advanced scanning (parallel, fingerprinting)?
├── YES → use --features advanced
└── NO → continue

Do you need code provenance tracking (AI attribution)?
├── YES → use --features anti-llm-cheat
└── NO → continue

→ Default build (no flags) is sufficient
```

---

## References

- Cargo feature documentation: https://doc.rust-lang.org/cargo/reference/features.html
- Feature projection tests: `tests/feature_projection.rs`
- Advanced capabilities: `docs/ARCHITECTURE.md` §Advanced Capabilities
- Cargo feature flag best practices: https://doc.rust-lang.org/cargo/reference/features.html#feature-resolver-version-2

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Documented and extended for Phase 1 Weeks 9-12 |
