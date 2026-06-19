<!-- BEGIN ggen:command-reference -->
<!-- Rendered from ontology/cargo-cicd-capabilities.ttl. Do not edit by hand. -->

# `cargo cicd sbom generate` / `cargo cicd sbom show`

Generates or displays a CycloneDX JSON Software Bill of Materials (SBOM) for the workspace. `generate` shells to `cargo cyclonedx` and writes `sbom.json`; `show` reads that file and displays file size + first 20 lines.

**Noun:** `sbom` &nbsp;&nbsp; **Verbs:** `generate`, `show`

<!-- END ggen:command-reference -->

<!-- BEGIN custom:synopsis -->
## Synopsis

```sh
cargo cicd sbom generate
cargo cicd sbom show
```
<!-- END custom:synopsis -->

<!-- BEGIN custom:description -->
## Description

### `sbom generate`

Shells to `cargo cyclonedx --format json --output-cdx sbom.json` and writes the resulting CycloneDX JSON SBOM to `sbom.json` at the workspace root. If `cargo-cyclonedx` is not installed the command degrades gracefully: it prints a warning, claims verdict `WARN`, and exits 0 so that CI pipelines are not broken.

### `sbom show`

Reads `sbom.json` from the workspace root and displays:
- The file size in bytes
- The first 20 lines of the JSON content

If `sbom.json` does not exist the command prints a `WARN` and exits 0.

### Prerequisites

`sbom generate` requires `cargo-cyclonedx` to be installed:

```sh
cargo install cargo-cyclonedx
```

Verify it is available before running:

```sh
cargo cyclonedx --version
```

### Graceful Degradation

| Condition | Behaviour | Verdict |
|---|---|---|
| `cargo-cyclonedx` installed | Writes `sbom.json` | `PASS` |
| `cargo-cyclonedx` not found | Prints install hint; does not write file | `WARN` |
| `sbom.json` exists (`show`) | Displays size and first 20 lines | `PASS` |
| `sbom.json` missing (`show`) | Prints missing-file warning | `WARN` |

A `WARN` verdict means the command completed without performing its primary action. The exit code is still 0 so downstream CI steps are not gated.
<!-- END custom:description -->

<!-- BEGIN custom:evidence -->
## Evidence Emission

Both verbs emit ProcessEvents to `target/cargo-cicd/evidence/`.

### `sbom generate`

Event lifecycle:
1. `start` transition at entry
2. `cargo cyclonedx` invoked (or skipped with `WARN` if absent)
3. `complete` transition with `verdict_claimed` of `PASS` or `WARN`

XES `case_id`: `sbom_generate_phase`

### `sbom show`

Event lifecycle:
1. `start` transition at entry
2. `sbom.json` read (or skipped with `WARN` if absent)
3. `complete` transition with `verdict_claimed` of `PASS` or `WARN`

XES `case_id`: `sbom_show_phase`

```
target/cargo-cicd/evidence/
├── evt-sbom-generate-<timestamp>Z.xes
├── evt-sbom-generate-<timestamp>Z.jsonl
├── evt-sbom-show-<timestamp>Z.xes
└── evt-sbom-show-<timestamp>Z.jsonl
```
<!-- END custom:evidence -->

<!-- BEGIN custom:exit-codes -->
## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Command completed (including graceful-degradation `WARN` cases) |
| 1 | Unexpected internal error |
| 2 | Invalid workspace — `Cargo.toml` not found |
<!-- END custom:exit-codes -->

<!-- BEGIN custom:examples -->
## Examples

```sh
# Generate SBOM (requires cargo-cyclonedx)
cargo cicd sbom generate

# View the generated SBOM
cargo cicd sbom show

# Install cargo-cyclonedx then generate
cargo install cargo-cyclonedx
cargo cicd sbom generate

# Check the full sbom.json
cat sbom.json | jq .
```
<!-- END custom:examples -->
