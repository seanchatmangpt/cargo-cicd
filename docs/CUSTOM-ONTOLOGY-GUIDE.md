# Custom Ontology Guide

**How to Extend cargo-cicd with Organization-Specific Capabilities**

**Document Type:** Tutorial  
**Audience:** Platform engineers, DevOps teams extending cargo-cicd  
**Prerequisites:** Basic Rust knowledge; no prior RDF/SPARQL experience required  
**Companion ADR:** `docs/adr/ADR-018-ontology-driven-manufacturing.md`

---

## Table of Contents

- [Part 1: Understanding the Ontology](#part-1-understanding-the-ontology)
- [Part 2: Writing Your First Custom Capability](#part-2-writing-your-first-custom-capability)
- [Part 3: Capability Types](#part-3-capability-types)
- [Part 4: Registering in the Ecosystem](#part-4-registering-in-the-ecosystem)
- [Part 5: Advanced — Process Model DSL](#part-5-advanced--process-model-dsl)
- [Part 6: Troubleshooting](#part-6-troubleshooting)

---

## Part 1: Understanding the Ontology

### 1.1 What Is an Ontology?

An ontology is a formal description of concepts and their relationships in a domain. cargo-cicd's ontology describes:

- **Nouns**: The things you can operate on (`status`, `target`, `test`, etc.)
- **Verbs**: The operations you can perform on each noun (`show`, `run`, `prune`, etc.)
- **Capabilities**: The pairing of a noun and verb that produces CLI command (`status show`)
- **Properties**: Attributes of capabilities (is it read-only? does it emit evidence? what's its description?)

cargo-cicd uses the **RDF** (Resource Description Framework) data model expressed in **Turtle** (`.ttl`) syntax. Don't be intimidated — Turtle is just a human-readable way to write facts about things.

### 1.2 RDF/Turtle Basics

RDF consists of **triples**: Subject → Predicate → Object.

In Turtle syntax:
```turtle
# A triple: cargo-cicd's status noun IS A skos:Concept
cc:status a skos:Concept .
#  ↑         ↑            ↑
# Subject   Predicate   Object

# Multiple predicates about the same subject:
cc:status a skos:Concept ;        # semicolon: same subject, new predicate
    cc:isNoun true ;
    skos:prefLabel "status" ;
    dcterms:description "Workspace health snapshot" .   # period: ends the statement
```

**Namespaces** are prefixes that expand to full URIs:
```turtle
@prefix cc:      <https://cargo-cicd.rs/ontology/capabilities#> .
@prefix skos:    <http://www.w3.org/2004/02/skos/core#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
```

This means `cc:status` is shorthand for the full URI `https://cargo-cicd.rs/ontology/capabilities#status`.

### 1.3 SKOS Vocabulary

cargo-cicd uses the **SKOS** (Simple Knowledge Organization System) vocabulary because it provides standard concepts for organizing knowledge:

| SKOS Term | Meaning | cargo-cicd Use |
|-----------|---------|---------------|
| `skos:Concept` | A unit of thought | Every noun and verb is a Concept |
| `skos:prefLabel` | The preferred human label | The noun/verb name in CLI grammar |
| `skos:altLabel` | An alternative label | Aliases (e.g., `workspace` → `ws`) |
| `skos:broader` | More general concept | Verb is broader than... (not used yet) |
| `skos:narrower` | More specific concept | Capability narrows to verb |

### 1.4 The cargo-cicd Namespace

The `cc:` namespace contains cargo-cicd-specific properties:

```turtle
@prefix cc: <https://cargo-cicd.rs/ontology/capabilities#> .

# Properties used in the ontology:
cc:isNoun         # true if this Concept is a noun (top-level CLI command)
cc:isVerb         # true if this Concept is a verb (sub-command)
cc:noun           # Links a verb to its parent noun
cc:defaultVerb    # The verb to use when just the noun is typed
cc:isReadOnly     # true if this verb doesn't modify state
cc:emitsEvidence  # true if this verb emits XES evidence
cc:exitOnDirty    # true if this verb should exit non-zero on dirty git state
cc:requiresConfirm # true if this verb requires --confirm flag for destructive action
```

### 1.5 Existing Capability Examples

Open `ontology/cargo-cicd-capabilities.ttl` and look at a few examples:

**A simple read-only noun with one verb**:
```turtle
cc:status a skos:Concept ;
    cc:isNoun true ;
    skos:prefLabel "status" ;
    dcterms:description "Workspace health snapshot showing git state, toolchain, and test readiness" ;
    cc:defaultVerb cc:status-show .

cc:status-show a skos:Concept ;
    cc:isVerb true ;
    cc:noun cc:status ;
    skos:prefLabel "show" ;
    dcterms:description "Display the workspace health snapshot in the terminal" ;
    cc:isReadOnly true ;
    cc:emitsEvidence true ;
    cc:exitOnDirty false .
```

**A destructive verb requiring confirmation**:
```turtle
cc:target-prune a skos:Concept ;
    cc:isVerb true ;
    cc:noun cc:target ;
    skos:prefLabel "prune" ;
    dcterms:description "Remove stale build artifacts from the target directory" ;
    cc:isReadOnly false ;
    cc:requiresConfirm true ;
    cc:emitsEvidence true .
```

---

## Part 2: Writing Your First Custom Capability

### Worked Example: "Ensure All Tests Pass Before Publication"

Let's add a new noun `gate` with a verb `check` that enforces a quality gate before publishing. This is a common organizational requirement.

**Goal**: `cargo cicd gate check` — verifies all required conditions are met for publishing.

### Step 1: Define Your Namespace

Create a custom namespace for your organization:

```turtle
# In your custom ontology file: ontology/custom/my-org-capabilities.ttl
@prefix myorg: <https://engineering.myorg.com/cargo-cicd-extensions#> .
@prefix cc:    <https://cargo-cicd.rs/ontology/capabilities#> .
@prefix skos:  <http://www.w3.org/2004/02/skos/core#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
```

### Step 2: Define the Noun

```turtle
# The "gate" noun
myorg:gate a skos:Concept ;
    cc:isNoun true ;
    skos:prefLabel "gate" ;
    dcterms:description "Quality gate enforcement before publication" ;
    cc:defaultVerb myorg:gate-check .
```

### Step 3: Define the Verb

```turtle
# The "check" verb within "gate"
myorg:gate-check a skos:Concept ;
    cc:isVerb true ;
    cc:noun myorg:gate ;
    skos:prefLabel "check" ;
    dcterms:description "Verify all quality gate conditions are met. Exits non-zero if any condition fails." ;
    cc:isReadOnly true ;
    cc:emitsEvidence true ;
    cc:exitOnDirty true .  # Fail if git has uncommitted changes
```

### Step 4: Configure ggen to Include Your Ontology

Add your ontology file to `ggen.toml`:

```toml
# ggen.toml (in your workspace root)

[ontology]
path = "ontology/cargo-cicd-capabilities.ttl"
namespace = "https://cargo-cicd.rs/ontology/capabilities#"

# Additional ontology files to include
[[ontology.imports]]
path = "ontology/custom/my-org-capabilities.ttl"
namespace = "https://engineering.myorg.com/cargo-cicd-extensions#"
prefix = "myorg"
```

### Step 5: Add a SPARQL Inference Rule (if needed)

If your capability requires reasoning (e.g., "a ComplianceCapability implies emitting evidence"), add a SPARQL inference rule:

```sparql
# In queries/myorg-inferences.sparql

# If a capability is a ComplianceCapability, it automatically emits evidence
INSERT {
    ?cap cc:emitsEvidence true .
}
WHERE {
    ?cap a myorg:ComplianceCapability .
    FILTER NOT EXISTS { ?cap cc:emitsEvidence ?existing . }
}
```

Add the query to `ggen.toml`:
```toml
[[inferences]]
sparql = "queries/myorg-inferences.sparql"
```

### Step 6: Run ggen

```sh
ggen
```

ggen will read your ontology, apply SPARQL inferences, and generate:

- `src/nouns/gate.rs` — Rust noun module scaffold
- `tests/cli/test_gate.rs` — CLI test scaffold
- `docs/reference/commands/gate.md` — Reference documentation

### Step 7: Implement the Verb Logic

Open the generated scaffold `src/nouns/gate.rs`:

```rust
//! # Gate Noun
//!
//! Quality gate enforcement before publication.
//!
//! ## Verbs
//! - `check` — Verify all quality gate conditions are met.

use clap_noun_verb::{NounCommand, VerbCommand};
use crate::engine::EngineState;

pub struct GateNoun;

impl NounCommand for GateNoun {
    fn name() -> &'static str { "gate" }
    fn description() -> &'static str { "Quality gate enforcement before publication" }
}

pub struct CheckVerb;

impl VerbCommand for CheckVerb {
    fn name() -> &'static str { "check" }
    fn description() -> &'static str {
        "Verify all quality gate conditions are met. Exits non-zero if any condition fails."
    }

    fn run() -> anyhow::Result<()> {
        let state = EngineState::from_workspace();
        // TODO: Implement gate check logic
        println!("gate check: not yet implemented");
        Ok(())
    }
}
```

**Implement the logic** (replacing the TODO):

```rust
impl VerbCommand for CheckVerb {
    fn run() -> anyhow::Result<()> {
        let state = EngineState::from_workspace();
        let mut failures = Vec::new();

        // Condition 1: Tests must have passed recently
        #[cfg(feature = "process-data")]
        {
            let last_test = state.process_events.last_event_for("test changed");
            match last_test {
                None => failures.push("No recent test run found. Run `cargo cicd test changed` first."),
                Some(event) if event.verdict_claimed == "FAIL" => {
                    failures.push("Last test run failed. Fix tests before publishing.");
                },
                _ => {
                    println!("  ✓ Tests passed");
                }
            }
        }

        // Condition 2: Git must be clean
        #[cfg(feature = "process-data")]
        {
            if !state.git_phase.dirty_files.is_empty() {
                failures.push("Uncommitted changes detected. Commit or stash before publishing.");
            } else {
                println!("  ✓ Git is clean");
            }
        }

        // Condition 3: Cargo.toml must have required publish metadata
        let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
        if !cargo_toml.contains("license") {
            failures.push("Cargo.toml missing `license` field. Required for crates.io publication.");
        } else {
            println!("  ✓ License field present");
        }

        // Report results
        if failures.is_empty() {
            println!("\n[PASS] All quality gate conditions met.");

            // Emit evidence
            #[cfg(feature = "process-data")]
            {
                let event = crate::evidence::ProcessEvent::new("gate check", "PASS");
                crate::evidence::emit_evidence(&event)?;
            }

            Ok(())
        } else {
            eprintln!("\n[FAIL] Quality gate failed:");
            for failure in &failures {
                eprintln!("  ✗ {}", failure);
            }

            // Emit failure evidence
            #[cfg(feature = "process-data")]
            {
                let mut event = crate::evidence::ProcessEvent::new("gate check", "FAIL");
                event.detail = failures.join("; ");
                crate::evidence::emit_evidence(&event)?;
            }

            std::process::exit(1);
        }
    }
}
```

### Step 8: Verify Tests Pass

The test scaffold in `tests/cli/test_gate.rs` verifies basic CLI contracts:

```sh
cargo test --test cli test_gate
```

---

## Part 3: Capability Types

cargo-cicd's ontology supports four primary capability types. Declaring a type enables specific behaviors and ggen template selection.

### 3.1 ComplianceCapability

Maps to a regulatory standard. Enables compliance-specific evidence attributes.

```turtle
myorg:gate-check a skos:Concept, myorg:ComplianceCapability ;
    myorg:mapsToStandard "SLSA-L3" ;
    myorg:mapsToStandard "NIST-SP-800-218-PW.1.2" ;
    cc:noun myorg:gate ;
    skos:prefLabel "check" ;
    dcterms:description "Quality gate for SLSA L3 compliance" .
```

When `mapsToStandard` is present, ggen generates XES attributes for the standard in the evidence output:
```xml
<string key="cargoCI:compliance_standard" value="SLSA-L3"/>
<string key="cargoCI:standard_control" value="NIST-SP-800-218-PW.1.2"/>
```

### 3.2 SecurityCapability

Maps to vulnerability scanning or security assessment. Enables security-specific evidence.

```turtle
myorg:audit-security a skos:Concept, myorg:SecurityCapability ;
    myorg:securityTool "cargo-audit" ;
    myorg:cveDatabase "rustsec" ;
    cc:noun myorg:audit ;
    skos:prefLabel "security" ;
    dcterms:description "Scan workspace for known vulnerabilities via cargo-audit" .
```

### 3.3 QualityCapability

Maps to code quality metrics (coverage, complexity, linting).

```turtle
myorg:quality-check a skos:Concept, myorg:QualityCapability ;
    myorg:minCoverage "80" ;          # Minimum line coverage %
    myorg:maxComplexity "15" ;         # Maximum cyclomatic complexity
    cc:noun myorg:quality ;
    skos:prefLabel "check" ;
    dcterms:description "Verify code quality metrics meet organizational thresholds" .
```

### 3.4 Custom Capabilities

For domain-specific needs, define your own capability type:

```turtle
# Define a new capability type
myorg:PerformanceCapability a rdfs:Class ;
    rdfs:subClassOf cc:Capability ;
    rdfs:comment "A capability that validates performance characteristics" .

# Use it
myorg:perf-benchmark a skos:Concept, myorg:PerformanceCapability ;
    myorg:latencyBudgetMs "100" ;
    myorg:throughputMinRps "1000" ;
    cc:noun myorg:perf ;
    skos:prefLabel "benchmark" ;
    dcterms:description "Run performance benchmarks and compare against SLO thresholds" .
```

---

## Part 4: Registering in the Ecosystem

### 4.1 Publishing to the Ontology Registry

The ontology registry is a Git repository where organizations publish their capability ontologies. Others can import them into their `ggen.toml`.

**To publish your ontology**:

1. Fork the ontology registry: `https://github.com/cargo-cicd-rs/ontology-registry`
2. Add your ontology in `registry/<your-org>/<capability-name>/v1/capabilities.ttl`
3. Add a registry entry in `registry/<your-org>/index.json`:

```json
{
  "org": "myorg",
  "capabilities": [
    {
      "name": "gate",
      "version": "1.0.0",
      "path": "registry/myorg/gate/v1/capabilities.ttl",
      "namespace": "https://engineering.myorg.com/cargo-cicd-extensions#",
      "description": "Quality gate enforcement capabilities",
      "standards": ["SLSA-L3", "NIST-SP-800-218"],
      "license": "Apache-2.0"
    }
  ]
}
```

4. Open a pull request. Registry maintainers review for namespace conflicts and basic schema validity.

### 4.2 Versioning Your Ontology

Ontology versions follow semantic versioning:

- **Patch** (1.0.0 → 1.0.1): Add optional properties; fix descriptions.
- **Minor** (1.0.0 → 1.1.0): Add new verbs to existing nouns; add new optional nouns.
- **Major** (1.0.0 → 2.0.0): Remove or rename nouns/verbs; change required properties; break backward compatibility.

**Version declaration in Turtle**:
```turtle
myorg:GateCapabilityOntology a owl:Ontology ;
    owl:versionIRI <https://engineering.myorg.com/cargo-cicd-extensions/v1.0.0> ;
    owl:versionInfo "1.0.0" ;
    dcterms:modified "2026-06-17" .
```

### 4.3 Importing Other Teams' Ontologies

To use capabilities published by another team:

```toml
# In your ggen.toml
[[ontology.imports]]
registry = "cargo-cicd-rs/ontology-registry"
org = "security-team"
name = "vulnerability-gate"
version = "^1.0"
namespace = "https://security.myorg.com/cargo-cicd-extensions#"
prefix = "secgate"
```

ggen fetches the ontology from the registry during `ggen` runs (with local caching).

---

## Part 5: Advanced — Process Model DSL

### 5.1 Declaring Process Constraints

A process model defines the required order and timing of capability executions. You can declare that your `gate check` must happen before `publish run`:

```turtle
# Process constraint: gate check must precede publish run
myorg:GateBeforePublish a pm:OrderingConstraint ;
    pm:model myorg:MyOrgReleaseProcess ;
    pm:before myorg:gate-check ;
    pm:after cc:publish-run ;
    pm:type pm:StrictlyBefore ;
    dcterms:description "Quality gate must be cleared before publishing" .
```

### 5.2 Temporal Constraints

Declare that tests must have run within 24 hours of publication:

```turtle
myorg:TestRecencyForPublish a pm:TemporalConstraint ;
    pm:model myorg:MyOrgReleaseProcess ;
    pm:activity cc:test-changed ;
    pm:precedingActivity cc:publish-run ;
    pm:maxIntervalHours 24 ;
    dcterms:description "Tests must have run within 24 hours of publishing" .
```

### 5.3 Defining Your Full Process Model

Combine constraints into a complete process model:

```turtle
# The full process model
myorg:MyOrgReleaseProcess a pm:ProcessModel ;
    pm:version "1.0" ;
    dcterms:description "MyOrg's release process for Rust crates" ;
    pm:requiredActivities (
        cc:status-show
        cc:test-changed
        myorg:gate-check
        cc:publish-run
    ) ;
    pm:constraints (
        myorg:GateBeforePublish
        myorg:TestRecencyForPublish
    ) .
```

### 5.4 Conformance Checking Against Your Process

Once a process model is defined, enable conformance checking:

```toml
# cicd.toml
[process_model]
active = "myorg:MyOrgReleaseProcess"

[[process_model.sources]]
name = "myorg-release-process"
path = "ontology/custom/my-org-release-process.ttl"
```

Run conformance check:
```sh
cargo cicd evidence audit --process-model myorg:MyOrgReleaseProcess
```

Output:
```
Process Conformance Check: myorg:MyOrgReleaseProcess
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fitness:    0.85 (85% of recent traces are conformant)
Violations: 15 non-conformant traces in past 7 days

Top violations:
  1. MissingRequiredActivity: gate-check (12 traces)
     → Run `cargo cicd gate check` before `cargo cicd publish run`
  2. TemporalViolation: test-changed > 24h before publish (3 traces)
     → Tests are more than 24h old. Re-run `cargo cicd test changed`.
```

---

## Part 6: Troubleshooting

### Issue: ggen Fails with "Namespace collision"

**Symptom**:
```
ggen: Error: Namespace collision between 'cc:status-show' and 'myorg:status-show'
Two concepts share the same skos:prefLabel "show" under noun "status"
```

**Cause**: Your custom ontology defines a verb with the same noun/verb name as an existing capability.

**Fix**: Either use a different verb name, or explicitly extend the existing noun:

```turtle
# Instead of redefining status-show, add a new verb to status:
myorg:status-compliance a skos:Concept ;
    cc:isVerb true ;
    cc:noun cc:status ;           # Use the existing cc: noun
    skos:prefLabel "compliance" ; # Different verb name
    dcterms:description "Show compliance status alongside workspace health" .
```

### Issue: ggen Fails with "SPARQL parse error"

**Symptom**:
```
ggen: Error in queries/myorg-inferences.sparql line 3:
  unexpected token: 'FROM' (expected WHERE)
```

**Fix**: SPARQL is case-sensitive and has strict syntax. Common mistakes:

```sparql
# WRONG: missing graph pattern braces
SELECT ?cap WHERE ?cap a myorg:ComplianceCapability .

# CORRECT:
SELECT ?cap WHERE { ?cap a myorg:ComplianceCapability . }
```

**Debugging tip**: Test your SPARQL query against the ontology:
```sh
# Using Apache Jena (if installed)
sparql --data ontology/cargo-cicd-capabilities.ttl \
       --query queries/myorg-inferences.sparql

# Using online SPARQL endpoint
# Load your .ttl file into https://yasgui.triply.cc/
```

### Issue: Generated Rust Code Doesn't Compile

**Symptom**:
```
error[E0412]: cannot find type `GateNoun` in this scope
```

**Cause**: The generated `src/nouns/gate.rs` was generated but not registered in `src/nouns/mod.rs`.

**Fix**: ggen should have updated `mod.rs`, but if it didn't:

```rust
// In src/nouns/mod.rs
pub mod gate;    // Add this line

// In src/main.rs or wherever nouns are registered:
use nouns::gate::{GateNoun, CheckVerb};
// Register in clap-noun-verb builder...
```

The ggen template for `mod.rs` registration is in `templates/noun_mod_registration.rs.tera`. If it's missing, add it:

```toml
# In ggen.toml
[[outputs]]
template = "templates/noun_mod_registration.rs.tera"
output = "src/nouns/mod.rs"
per = "all"
```

### Issue: Tests Fail with "forbidden term in help output"

**Symptom**:
```
test invariant_public_boundary_no_forbidden_terms_in_all_help FAILED
  "ALIVE" found in 'cargo cicd gate check --help' output
```

**Cause**: Your verb description or output contains a forbidden term. See `CLAUDE.md` for the complete list.

**Fix**: Remove the forbidden term from your ontology descriptions and verb implementation:

```turtle
# WRONG:
myorg:gate-check a skos:Concept ;
    dcterms:description "Check ALIVE status of quality gates" .  # ALIVE is forbidden

# CORRECT:
myorg:gate-check a skos:Concept ;
    dcterms:description "Check the current status of quality gates" .
```

Then re-run ggen:
```sh
ggen
cargo test --test invariants
```

### Issue: SPARQL Query Returns No Results

**Symptom**: ggen runs without errors, but no new noun is generated.

**Cause**: The SPARQL capability projection query doesn't match your ontology triples.

**Debugging**: Run the capability projection query manually:
```sh
# Print all concepts that the projection query would select
sparql --data ontology/cargo-cicd-capabilities.ttl \
       --data ontology/custom/my-org-capabilities.ttl \
       --query queries/capability_projection.sparql
```

**Common cause**: Missing prefix declaration at top of your `.ttl` file. The projection query uses `cc:isNoun` — if your concepts don't have `cc:isNoun true`, they won't be selected.

```turtle
# Check your concept has this triple:
myorg:gate cc:isNoun true .
```

### Issue: ggen Output Is Not Idempotent

**Symptom**:
```
$ ggen
$ git diff
  src/nouns/gate.rs (modified)
$ ggen again
$ git diff
  src/nouns/gate.rs (modified, different content)
```

**Cause**: Non-deterministic SPARQL query ordering, or template generates content that depends on runtime state (timestamps, random UUIDs).

**Fix**: 
1. Add `ORDER BY` to all SPARQL queries: `ORDER BY ?noun_name ?verb_name`
2. Remove any runtime-state dependencies from templates:
```tera
{# WRONG — timestamp in generated code #}
//! Generated at {{ now() }}

{# CORRECT — no runtime state #}
//! Generated by ggen from ontology
```

### Advanced Debugging: Inspect ggen's Intermediate Representation

To see the data model that ggen builds before template rendering:

```sh
ggen --dump-model
```

This outputs a JSON representation of all concepts and their properties, which is what the templates receive as context.

---

## Quick Reference Card

### Turtle Syntax Cheat Sheet

```turtle
# Prefix declarations (at top of file)
@prefix myorg: <https://myorg.com/cargo-cicd#> .
@prefix cc:    <https://cargo-cicd.rs/ontology/capabilities#> .

# Define a noun
myorg:noun-name a skos:Concept ;
    cc:isNoun true ;
    skos:prefLabel "noun-name" ;
    dcterms:description "Human-readable description" ;
    cc:defaultVerb myorg:noun-name-default-verb .

# Define a verb
myorg:noun-name-verb-name a skos:Concept ;
    cc:isVerb true ;
    cc:noun myorg:noun-name ;
    skos:prefLabel "verb-name" ;
    dcterms:description "Human-readable description" ;
    cc:isReadOnly true ;         # or false
    cc:emitsEvidence true ;      # or false
    cc:requiresConfirm false .   # or true for destructive
```

### ggen.toml Quick Reference

```toml
[ontology]
path = "ontology/cargo-cicd-capabilities.ttl"
namespace = "https://cargo-cicd.rs/ontology/capabilities#"

[[ontology.imports]]
path = "ontology/custom/my-capabilities.ttl"
namespace = "https://myorg.com/cargo-cicd#"
prefix = "myorg"
```

### Common Commands

```sh
# Generate code from ontology
ggen

# Verify generation is idempotent
ggen && git diff --stat

# Test generated noun CLI contracts
cargo test --test cli

# Test forbidden term invariants
cargo test --test invariants

# Check your SPARQL query
sparql --data ontology/*.ttl --query queries/your-query.sparql
```

---

*Guide version 1.0 — 2026-06-17*  
*See also: `docs/adr/ADR-018-ontology-driven-manufacturing.md`, `docs/adr/ADR-020-phase2-pluggable-process-models.md`*
