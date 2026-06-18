# ADR-020: Phase 2 Pluggable Process Model Architecture

**Status:** Proposed (Phase 2)  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team, Vision 2030 architecture committee  
**Tags:** process-models, plugins, dsl, rdf, conformance, phase-2, extensibility

---

## Context

cargo-cicd's current process model is **fixed**: every workspace follows the same sequence of CI/CD activities (status → test → publish), and conformance is checked against cargo-cicd's built-in process definition. This works for the default use case but fails for:

1. **Regulated industries**: A medical device software team (FDA 21 CFR Part 11) has different required process steps than a consumer game library. Hardcoded process models cannot satisfy both.

2. **Organizational compliance**: Many enterprises have internal process requirements (change management, mandatory code review gates, deployment approval workflows) that do not map to cargo-cicd's default model.

3. **Standards certification**: Different compliance frameworks (SLSA, DO-178C, ISO 26262, IEC 62443) have different required activities and their ordering. A fixed model cannot satisfy all of them simultaneously.

4. **Domain-specific workflows**: A cryptography library has different quality gates than a web framework (timing attack testing, FIPS compliance). Domain expertise must be expressible as process constraints.

5. **Team-specific policies**: Even within a single organization, different teams may have different release process requirements. A monorepo with multiple crates may need per-crate process model customization.

### What is a Process Model?

A process model defines:
1. **Activities**: The named steps in the process (e.g., `code-review`, `test-run`, `security-scan`, `publish`).
2. **Ordering constraints**: Which activities must happen before others (e.g., `test-run` must precede `publish`).
3. **Temporal constraints**: How quickly activities must complete, and how recently they must have occurred.
4. **Conditional paths**: Different paths through the process for different conditions (e.g., breaking changes vs. patch releases).
5. **Conformance rules**: What constitutes a conformant trace (e.g., every trace must contain exactly one `publish` and at least one `test-run`).

Currently, cargo-cicd's process model is implicit — embedded in the order of verb calls and the assertions in test code. A pluggable process model makes this explicit and externally customizable.

### Vision 2030 Requirement

Phase 2 must support pluggable process models so that:

1. Organizations can define their own compliance process models in a standard format.
2. cargo-cicd can check conformance against the declared process model automatically.
3. Multiple process models can coexist in a workspace (different models per crate or per environment).
4. The oracle ecosystem can use process models for adjudication (an oracle checks evidence against a declared model).

---

## Decision

**Phase 2 introduces a pluggable process model architecture. Process models are expressed as RDF/Turtle documents (PNML or custom ontology extensions) that define activity sequences, ordering constraints, and conformance rules. Models are loaded at compile-time (embedded in ggen) or at runtime (loaded from disk or URL).**

This is a Phase 2 decision — implementation is planned for Phase 2. Phase 1 uses the current hardcoded model.

### Process Model DSL

Process models are expressed as extensions to the cargo-cicd ontology:

```turtle
# A simple process model: test must happen before publish
@prefix cc: <https://cargo-cicd.rs/ontology/capabilities#> .
@prefix pm: <https://cargo-cicd.rs/ontology/process-model#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Define the process model identity
cc:BasicReleaseModel a pm:ProcessModel ;
    pm:version "1.0" ;
    pm:description "Basic Rust crate release process" .

# Define activities in the model
cc:CodeReviewActivity a pm:Activity ;
    pm:model cc:BasicReleaseModel ;
    pm:activityName "code-review" ;
    pm:required true .

cc:TestRunActivity a pm:Activity ;
    pm:model cc:BasicReleaseModel ;
    pm:activityName "test-run" ;
    pm:required true ;
    pm:minimumCount 1 .

cc:PublishActivity a pm:Activity ;
    pm:model cc:BasicReleaseModel ;
    pm:activityName "publish" ;
    pm:required true ;
    pm:maximumCount 1 .

# Ordering constraints
cc:ReviewBeforeTest a pm:OrderingConstraint ;
    pm:model cc:BasicReleaseModel ;
    pm:before cc:CodeReviewActivity ;
    pm:after cc:TestRunActivity ;
    pm:type pm:StrictlyBefore .

cc:TestBeforePublish a pm:OrderingConstraint ;
    pm:model cc:BasicReleaseModel ;
    pm:before cc:TestRunActivity ;
    pm:after cc:PublishActivity ;
    pm:type pm:StrictlyBefore .

# Temporal constraints
cc:TestRecencyConstraint a pm:TemporalConstraint ;
    pm:model cc:BasicReleaseModel ;
    pm:activity cc:TestRunActivity ;
    pm:maxAgeHours 24 .  # Tests must have run within 24 hours of publish
```

### Process Model Loading

Models are loaded via the `ggen.toml` configuration (compile-time) or `cicd.toml` (runtime):

**Compile-time (via ggen)**:

```toml
# ggen.toml
[[process_models]]
name = "basic-release"
path = "ontology/models/basic-release-model.ttl"
namespace = "https://cargo-cicd.rs/ontology/process-model#"

[[process_models]]
name = "fda-part11"
url = "https://models.cargo-cicd.rs/regulatory/fda-part-11-v1.ttl"
namespace = "https://cargo-cicd.rs/ontology/regulatory#"
```

**Runtime (via cicd.toml)**:

```toml
# cicd.toml
[process_model]
active = "basic-release"

[[process_model.sources]]
name = "basic-release"
path = "ontology/models/basic-release-model.ttl"

[[process_model.sources]]
name = "fda-part11"
url = "https://models.cargo-cicd.rs/regulatory/fda-part-11-v1.ttl"
cache_path = "target/cargo-cicd/models/fda-part11-v1.ttl"
cache_expiry_hours = 168  # Re-fetch weekly
```

### Conformance Checking Engine

The conformance checking engine compares recorded XES traces against the declared process model:

```rust
// src/conformance/mod.rs

pub struct ConformanceChecker {
    model: ProcessModel,
}

pub struct ConformanceResult {
    pub fitness: f32,          // 0.0-1.0: what fraction of activities were present
    pub precision: f32,        // 0.0-1.0: what fraction of recorded activities were expected
    pub violations: Vec<ConformanceViolation>,
    pub verdict: ConformanceVerdict,
}

pub enum ConformanceViolation {
    MissingRequiredActivity { activity: String },
    OrderingViolation { before: String, after: String, actual_order: ActivityOrder },
    TemporalViolation { activity: String, age_hours: f32, max_age_hours: f32 },
    ExceededMaximumCount { activity: String, count: usize, max: usize },
}

pub enum ConformanceVerdict {
    Conformant,
    NonConformant,
    PartiallyConformant { fitness: f32 },
}

impl ConformanceChecker {
    pub fn new(model: ProcessModel) -> Self { Self { model } }

    pub fn check_trace(&self, trace: &XesTrace) -> ConformanceResult {
        let mut violations = Vec::new();

        // Check required activities
        for required in self.model.required_activities() {
            if !trace.contains_activity(&required.name) {
                violations.push(ConformanceViolation::MissingRequiredActivity {
                    activity: required.name.clone(),
                });
            }
        }

        // Check ordering constraints
        for constraint in self.model.ordering_constraints() {
            if let (Some(before_pos), Some(after_pos)) = (
                trace.last_position_of(&constraint.before),
                trace.first_position_of(&constraint.after),
            ) {
                if before_pos > after_pos {
                    violations.push(ConformanceViolation::OrderingViolation {
                        before: constraint.before.clone(),
                        after: constraint.after.clone(),
                        actual_order: ActivityOrder::Reversed,
                    });
                }
            }
        }

        // Check temporal constraints
        for temporal in self.model.temporal_constraints() {
            if let Some(last_occurrence) = trace.last_occurrence_of(&temporal.activity) {
                let age_hours = (Utc::now() - last_occurrence).num_hours() as f32;
                if age_hours > temporal.max_age_hours {
                    violations.push(ConformanceViolation::TemporalViolation {
                        activity: temporal.activity.clone(),
                        age_hours,
                        max_age_hours: temporal.max_age_hours,
                    });
                }
            }
        }

        let fitness = self.calculate_fitness(trace, &violations);
        let verdict = if violations.is_empty() {
            ConformanceVerdict::Conformant
        } else {
            ConformanceVerdict::NonConformant
        };

        ConformanceResult { fitness, precision: 1.0, violations, verdict }
    }
}
```

### Integration with Evidence Gate

The conformance checker integrates with the oracle evidence gate:

1. Before oracle adjudication, the conformance checker runs against the XES evidence.
2. A `ConformanceViolation` list is embedded in the XES trace.
3. The oracle can use the conformance result as an input to its adjudication.

```rust
// In src/nouns/evidence.rs AuditVerb

pub fn run(model_name: Option<&str>) -> Result<()> {
    let evidence_dir = Path::new("target/cargo-cicd/evidence");
    let xes_files: Vec<_> = glob::glob(evidence_dir.join("*.xes"))?.flatten().collect();

    // Load process model if specified
    let checker = model_name.map(|name| {
        let model = ProcessModelLoader::load(name)?;
        Ok::<_, anyhow::Error>(ConformanceChecker::new(model))
    }).transpose()?;

    for xes_path in &xes_files {
        let trace = XesParser::parse(xes_path)?;

        // Run conformance check if model loaded
        if let Some(ref checker) = checker {
            let result = checker.check_trace(&trace);
            if !result.violations.is_empty() {
                println!("⚠ Conformance violations in {}:", xes_path.display());
                for v in &result.violations {
                    println!("  - {}", v);
                }
            }
        }

        // Oracle adjudication (existing flow)
        #[cfg(feature = "wasm4pm")]
        let verdict = Wasm4pmShell::audit_xes(xes_path)?;
    }
    Ok(())
}
```

### Plugin Loading Mechanism

For runtime plugin loading (Phase 2+), process models can be distributed as WASM modules:

```rust
// Process model plugins as WASM
// Each plugin implements the ProcessModelPlugin trait

pub trait ProcessModelPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn activities(&self) -> Vec<ActivitySpec>;
    fn constraints(&self) -> Vec<ConstraintSpec>;
    fn check_trace(&self, trace: &XesTrace) -> ConformanceResult;
}

// WASM runtime (via wasmtime)
pub struct WasmProcessModelPlugin {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
    instance: wasmtime::Instance,
}
```

WASM plugins allow third parties to distribute custom process models without depending on cargo-cicd's internal APIs.

---

## Consequences

### Positive

1. **Regulatory flexibility**: Different regulatory frameworks can be expressed as distinct process models. A crate targeting DO-178C DAL B uses a stricter model than one targeting SLSA Level 1.

2. **Organizational customization**: Enterprises can define and enforce their own process models without forking cargo-cicd.

3. **Process model marketplace**: Models can be published and shared (similar to the ontology registry). Organizations can adopt community-developed models for their industry.

4. **Formal conformance checking**: Instead of implicit "did the tests pass?", conformance checking is explicit, auditable, and reproducible.

5. **Oracle alignment**: The oracle can adjudicate against a declared process model rather than cargo-cicd's defaults. Different oracles can specialize in different frameworks.

6. **Research compatibility**: Process mining research can be applied directly to cargo-cicd evidence logs using standard algorithms (token-based replay, alignment-based conformance).

### Negative

1. **Complexity for simple cases**: Teams that just want "did the tests pass?" don't need process models. The additional abstraction layer is overhead for them. Mitigation: The default model covers the simple case; custom models are explicitly opt-in.

2. **Model governance**: Who defines the authoritative model for "SLSA Level 3 compliance"? Organizational models may conflict with community models. Mitigation: The ontology registry (see Phase 2 design) governs model namespaces.

3. **SPARQL/RDF learning curve**: Defining process models in Turtle requires RDF knowledge that most Rust developers lack. Mitigation: The `docs/CUSTOM-ONTOLOGY-GUIDE.md` provides templates and worked examples.

4. **Model version management**: As models evolve, evidence produced under model v1.0 may not conform to model v2.0. Mitigation: Evidence records the model version used for conformance checking; historical evidence is always checked against the model version in effect at the time.

5. **WASM plugin security**: Untrusted WASM plugins can be malicious. Mitigation: WASM sandbox provides isolation; plugins are signed; cargo-cicd does not execute plugin code with filesystem or network access.

---

## Phase 2 Implementation Milestones

| Milestone | Target | Deliverable |
|-----------|--------|------------|
| Process model DSL design | Phase 2 Week 1-2 | Turtle schema for pm: namespace |
| Basic conformance checker | Phase 2 Week 3-4 | `ConformanceChecker` with ordering constraints |
| Model loader (file-based) | Phase 2 Week 5-6 | Load `.ttl` from disk or URL |
| oracle integration | Phase 2 Week 7-8 | Pass conformance result to wpm |
| WASM plugin support | Phase 2 Week 9-10 | `WasmProcessModelPlugin` via wasmtime |
| Model registry | Phase 2 Week 11-12 | Publish/discover models via GraphQL API |

---

## References

- PNML (Petri Net Markup Language): ISO/IEC 15909-2:2011
- ProM conformance checking algorithms: https://promtools.org/
- WASM component model: https://github.com/WebAssembly/component-model
- wasmtime Rust crate: https://crates.io/crates/wasmtime
- ADR-018: Ontology-Driven Manufacturing (ggen pipeline)
- `docs/CUSTOM-ONTOLOGY-GUIDE.md`: Process model DSL tutorial
- `docs/PHASE-2-DESIGN.md`: Phase 2 implementation plan

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Initial Phase 2 proposal |
