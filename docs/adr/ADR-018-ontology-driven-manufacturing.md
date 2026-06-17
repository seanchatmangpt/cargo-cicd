# ADR-018: Ontology-Driven CLI Grammar Manufacturing via ggen

**Status:** Accepted (Current Implementation)  
**Date:** 2026-06-17  
**Deciders:** cargo-cicd core team  
**Tags:** ggen, ontology, manufacturing, rdf, sparql, tera, code-generation

---

## Context

cargo-cicd is a CLI tool where the command grammar (nouns and verbs) needs to evolve as new CI/CD capabilities are added. A naive approach handwrites each noun module in `src/nouns/` along with its clap parsing code, help text, test scaffolding, and documentation.

### The Handwriting Problem

With 10 existing nouns (status, target, test, trybuild, git, publish, workspace, evidence, pipeline, lsp) and planned expansion to 20+ nouns under Vision 2030, handwriting creates:

1. **Inconsistency**: Help text, error messages, and command output format drift between nouns as different developers implement them.
2. **Documentation lag**: Reference documentation (`docs/reference/commands/*.md`) falls out of sync with the implementation.
3. **Test scaffolding duplication**: Each noun needs similar test scaffolding (argument parsing, help text verification, forbidden term checking). Writing this from scratch for each noun is error-prone.
4. **Ontology drift**: The intended capabilities (defined conceptually) diverge from the implemented capabilities (defined in code) over time.
5. **Regeneration cost**: When the noun-verb pattern changes (e.g., a new standard verb structure), all noun modules must be updated manually.

### The ggen Manufacturing Pipeline

ggen (grammar generator) is a code generation tool that manufactures CLI grammar from a formal ontology. The pipeline is:

```
ontology/cargo-cicd-capabilities.ttl     (RDF/Turtle — source of truth)
        ↓
queries/*.sparql                           (SPARQL inference rules)
        ↓ [ggen SPARQL reasoning]
        ↓
templates/*.tera                           (Tera template files)
        ↓ [ggen template rendering]
        ↓
src/nouns/<noun>.rs                        (Rust noun modules)
tests/cli/test_<noun>.rs                  (CLI test scaffolding)
docs/reference/commands/<noun>.md         (Reference documentation)
README.md sections                        (Public documentation)
```

The ontology is the single source of truth. ggen ensures that code, tests, and documentation are always synchronized.

---

## Decision

**The CLI grammar is manufactured from an RDF/Turtle ontology via ggen. All new nouns and verbs must be defined in the ontology first, then manufactured by ggen.**

### Ontology Structure

The ontology uses the `cc:` namespace (cargo-cicd capabilities):

```turtle
# In ontology/cargo-cicd-capabilities.ttl
@prefix cc: <https://cargo-cicd.rs/ontology/capabilities#> .
@prefix skos: <http://www.w3.org/2004/02/skos/core#> .
@prefix dcterms: <http://purl.org/dc/terms/> .

# Define a noun
cc:status a skos:Concept ;
    cc:isNoun true ;
    skos:prefLabel "status" ;
    dcterms:description "Workspace health snapshot showing git state, toolchain, and test readiness" ;
    cc:defaultVerb cc:status-show .

# Define a verb within the noun
cc:status-show a skos:Concept ;
    cc:isVerb true ;
    cc:noun cc:status ;
    skos:prefLabel "show" ;
    dcterms:description "Display the workspace health snapshot in the terminal" ;
    cc:isReadOnly true ;
    cc:emitsEvidence true ;
    cc:exitOnDirty false .
```

### SPARQL Inference Rules

SPARQL queries infer derived properties from the ontology:

```sparql
# queries/capability_projection.sparql
# Selects all nouns and their verbs for code generation

SELECT ?noun_name ?verb_name ?description ?is_read_only ?emits_evidence ?default_verb
WHERE {
    ?noun cc:isNoun true ;
          skos:prefLabel ?noun_name ;
          dcterms:description ?description .
    OPTIONAL { ?noun cc:defaultVerb ?default_verb_concept .
               ?default_verb_concept skos:prefLabel ?default_verb . }
    OPTIONAL {
        ?verb cc:isVerb true ;
              cc:noun ?noun ;
              skos:prefLabel ?verb_name .
        OPTIONAL { ?verb cc:isReadOnly ?is_read_only . }
        OPTIONAL { ?verb cc:emitsEvidence ?emits_evidence . }
    }
}
ORDER BY ?noun_name ?verb_name
```

### Tera Templates

Templates generate Rust code, tests, and docs:

```
# templates/noun_module.rs.tera
//! # {{ noun.name | capitalize }} Noun
//!
//! {{ noun.description }}
//!
//! ## Verbs
{% for verb in noun.verbs %}
//! - `{{ verb.name }}` — {{ verb.description }}
{% endfor %}

use clap_noun_verb::{NounCommand, VerbCommand};
use crate::engine::EngineState;

pub struct {{ noun.name | pascal_case }}Noun;

impl NounCommand for {{ noun.name | pascal_case }}Noun {
    fn name() -> &'static str { "{{ noun.name }}" }
    fn description() -> &'static str { "{{ noun.description }}" }
}

{% for verb in noun.verbs %}
pub struct {{ verb.name | pascal_case }}Verb;

impl VerbCommand for {{ verb.name | pascal_case }}Verb {
    fn name() -> &'static str { "{{ verb.name }}" }
    fn description() -> &'static str { "{{ verb.description }}" }

    fn run() -> anyhow::Result<()> {
        let state = EngineState::from_workspace();
        // TODO: Implement {{ noun.name }} {{ verb.name }} logic
        println!("{{ noun.name }} {{ verb.name }}: not yet implemented");
        Ok(())
    }
}
{% endfor %}
```

### ggen.toml Configuration

```toml
# ggen.toml — manufacturing pipeline configuration
[ontology]
path = "ontology/cargo-cicd-capabilities.ttl"
namespace = "https://cargo-cicd.rs/ontology/capabilities#"

[queries]
capability_projection = "queries/capability_projection.sparql"
default_verb_projection = "queries/default_verb_projection.sparql"

[[outputs]]
template = "templates/noun_module.rs.tera"
output = "src/nouns/{noun.name}.rs"
per = "noun"

[[outputs]]
template = "templates/cli_test.rs.tera"
output = "tests/cli/test_{noun.name}.rs"
per = "noun"

[[outputs]]
template = "templates/reference_doc.md.tera"
output = "docs/reference/commands/{noun.name}.md"
per = "noun"

[[outputs]]
template = "templates/README_commands.md.tera"
output = "README.md"
section = "## Commands"
per = "all"
```

### Adding a New Noun via ggen

To add a new noun (e.g., `cargo cicd audit`):

1. **Define in ontology**:
   ```turtle
   cc:audit a skos:Concept ;
       cc:isNoun true ;
       skos:prefLabel "audit" ;
       dcterms:description "Security and compliance audit of workspace dependencies" ;
       cc:defaultVerb cc:audit-run .

   cc:audit-run a skos:Concept ;
       cc:isVerb true ;
       cc:noun cc:audit ;
       skos:prefLabel "run" ;
       dcterms:description "Run the full audit suite" ;
       cc:isReadOnly false ;
       cc:emitsEvidence true .
   ```

2. **Run ggen**:
   ```sh
   ggen
   ```

3. **Implement the logic** in the scaffolded `src/nouns/audit.rs`.

4. **Tests are already scaffolded** in `tests/cli/test_audit.rs`.

5. **Documentation is already generated** in `docs/reference/commands/audit.md`.

### Manufacturing Invariant

The `tests/ggen_customization_guard.rs` test verifies that ggen regeneration is idempotent:

```rust
#[test]
fn invariant_ggen_output_is_idempotent() {
    // Run ggen
    let output = Command::new("ggen").output().unwrap();
    assert!(output.status.success(), "ggen failed");
    
    // Verify no files changed
    let git_diff = Command::new("git")
        .args(["diff", "--name-only"])
        .output()
        .unwrap();
    let changed = String::from_utf8_lossy(&git_diff.stdout);
    assert!(changed.is_empty(),
        "ggen produced non-idempotent output. Changed files:\n{}", changed);
}
```

This test fails if the ontology and generated code are out of sync, ensuring that developers don't hand-edit generated files.

---

## Consequences

### Positive

1. **Single source of truth**: The ontology (`ontology/cargo-cicd-capabilities.ttl`) is the authoritative definition of cargo-cicd's capabilities. Code, tests, and documentation are all derived from it.

2. **Consistency enforcement**: All nouns follow identical patterns because they're generated from the same templates. Help text, error messages, and output format are uniform.

3. **Documentation as a byproduct**: Adding a new noun automatically generates reference documentation. Documentation is never out of date with the implementation (assuming ggen is run after ontology changes).

4. **Test scaffolding**: New nouns come with test stubs that verify help text, forbidden term absence, and basic CLI parsing. These tests would otherwise be forgotten or delayed.

5. **Formal process specification**: The ontology is a formal RDF/Turtle document. It can be reasoned over with SPARQL, imported into ontology visualization tools, and extended by third parties (see `docs/CUSTOM-ONTOLOGY-GUIDE.md`).

6. **Vision 2030 extensibility**: The ontology-driven approach enables the custom ontology ecosystem (Phase 2) where organizations define their own capabilities and have them manufactured by ggen.

### Negative

1. **ggen dependency**: Developers must have ggen installed to work on the ontology. If ggen has bugs or changes its behavior, all generated output is affected. Mitigation: ggen is a stable tool; the idempotency test catches regressions.

2. **Customization constraints**: Developers cannot add complex logic to generated files — any custom code is overwritten by the next `ggen` run. Mitigation: Generated files are scaffolds; logic is in separate impl files that ggen does not touch.

3. **Template learning curve**: Understanding Tera templates and SPARQL requires additional skills. Mitigation: The `docs/CUSTOM-ONTOLOGY-GUIDE.md` provides comprehensive tutorials.

4. **Template brittleness**: Changes to ggen templates affect all generated output. A template bug can break the entire CLI grammar. Mitigation: The idempotency test and CI gate catch template regressions immediately.

5. **SPARQL complexity**: Complex inference rules can be difficult to debug. Mitigation: SPARQL queries are kept simple; complex logic is in the Tera templates, not the queries.

---

## Manufacturing vs. Handwriting Comparison

| Concern | Handwriting | ggen Manufacturing |
|---------|------------|-------------------|
| Consistency | Drift over time | Enforced by templates |
| Documentation | Manual, often stale | Auto-generated, always current |
| Test scaffolding | Written once, may be skipped | Auto-generated for every noun |
| New noun cost | High (write module + tests + docs) | Low (define in ontology, run ggen) |
| Ontology/code sync | Not enforced | Enforced by idempotency test |
| Customization | Unconstrained | Constrained to template patterns |
| Learning curve | Rust only | Rust + SPARQL + Tera |

---

## References

- cargo-cicd ontology: `ontology/cargo-cicd-capabilities.ttl`
- SPARQL queries: `queries/`
- Tera templates: `templates/`
- ggen configuration: `ggen.toml`
- Idempotency test: `tests/ggen_customization_guard.rs`
- Custom ontology guide: `docs/CUSTOM-ONTOLOGY-GUIDE.md`
- RDF/Turtle specification: https://www.w3.org/TR/turtle/
- SKOS vocabulary: https://www.w3.org/TR/skos-reference/
- SPARQL 1.1: https://www.w3.org/TR/sparql11-query/
- Tera templates: https://keats.github.io/tera/

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-17 | Vision 2030 Architecture Committee | Documented as ADR for Phase 1 Weeks 9-12 |
