# wasm4pm Excluded Surfaces — cargo-cicd v26.6.2

Surfaces classified DO_NOT_USE from the wasm4pm Full Capability Map.

## Hard Exclusions

| Surface | Reason |
|---------|--------|
| `wpm doctor` as CI gate | Exits 0 even when checks fail — false-positive CI passes |
| `wpm lean` as validation gate | Hardcoded / not machine-parseable enough for gate use |
| `wpm agent` lifecycle for CI | Not stable for CI integration |
| `wpm wizard` | Interactive only |
| `wpm config show` | Config inspection, not evidence adjudication |
| `wpm mining conformance` | Confirmed stub — exits 0 regardless of input |
| `wpm oracle check` | Confirmed stub — exits 0 regardless of input |
| `wpm oracle watch` | Confirmed stub — exits 0 regardless of input |

## Integration Guard

`tests/ggen_customization_guard.rs::no_forbidden_terms_in_public_docs` enforces that
forbidden wasm4pm surfaces do not appear in public-facing documentation or CLI help text.

## Rationale

Using `wpm doctor` as a CI gate would produce false-positive passes. Using `wpm lean` 
or stubs would certify processes that were never actually adjudicated. The evidence gate 
law requires an external oracle that can genuinely refuse — only `wpm receipt doctor` 
and `wpm audit` meet this standard.
