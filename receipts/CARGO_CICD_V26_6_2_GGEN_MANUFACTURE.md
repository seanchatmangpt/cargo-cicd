---
receipt: CARGO_CICD_V26_6_2_GGEN_MANUFACTURE
date: 2026-06-02
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# ggen Manufacturing Receipt

## Source Law Surfaces Created

### Ontology
| File | Path | Content |
|---|---|---|
| cargo-cicd.ttl | `ontology/cargo-cicd.ttl` | 7 nouns, 9 verbs, 4 policies declared in Turtle RDF |

### SPARQL Queries
| File | Path | Purpose |
|---|---|---|
| commands.rq | `queries/commands.rq` | SELECT query projecting noun/verb pairs for code generation |

### Tera Templates
| File | Path | Output target |
|---|---|---|
| noun.rs.tera | `templates/noun.rs.tera` | `src/nouns/{noun_name}.rs` — noun module |
| command_doc.md.tera | `templates/command_doc.md.tera` | `docs/commands/{noun_name}.md` — command docs |
| cli_test.rs.tera | `templates/cli_test.rs.tera` | `tests/cli/test_{noun_name}.rs` — integration tests |
| cicd_toml_schema.rs.tera | `templates/cicd_toml_schema.rs.tera` | cicd.toml schema Rust types |

### Control File
| File | Path | Role |
|---|---|---|
| ggen.toml | `/Users/sac/cargo-cicd/ggen.toml` | Manufacturing control: project name, ontology source, generation rules |

## ggen.toml Rules Declared

```toml
[[rules]]
name = "noun-modules"
query = "queries/commands.rq"
template = "templates/noun.rs.tera"
output = "src/nouns/{{noun_name}}.rs"

[[rules]]
name = "command-docs"
query = "queries/commands.rq"
template = "templates/command_doc.md.tera"
output = "docs/commands/{{noun_name}}.md"

[[rules]]
name = "cli-tests"
query = "queries/commands.rq"
template = "templates/cli_test.rs.tera"
output = "tests/cli/test_{{noun_name}}.rs"
```

## Manufacturing Status

| Stage | Status | Notes |
|---|---|---|
| Source law (ontology) | CREATED | `ontology/cargo-cicd.ttl` present |
| SPARQL queries | CREATED | `queries/commands.rq` present |
| Tera templates | CREATED | 4 templates present |
| `ggen.toml` control | CREATED | Present at repo root |
| ggen CLI sync | NOT RUN | Requires ggen CLI with SPARQL backend activated |
| Manufactured outputs | HAND-INITIALIZED | `src/nouns/*.rs` bootstrapped manually; manufacture-ready for ggen sync |

## Verdict: PARTIAL
Source law is complete and manufacture-ready. ggen sync has not been run — requires ggen CLI invocation against the SPARQL backend. Manufactured outputs exist as hand-initialized bootstrap. Full manufacture receipt requires `ggen sync` execution producing artifact hashes.
