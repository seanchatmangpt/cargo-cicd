# Product Requirements Document (PRD) — cargo-cicd as an Operational Substrate

**Status**: Proposal  
**Version**: v1.0.0  
**Author**: teamwork_preview_explorer_docs_update  

---

## 1. Vision & Mission

### 1.1 Vision
> **"Make Cargo the operational substrate for software engineering."**

Cargo should no longer be viewed merely as a package manager and compiler runner. Instead, Cargo must become the sovereign execution container and admission authority that guarantees code safety, operational conformance, and process provenance.

### 1.2 Mission
To transition `cargo-cicd` from a local workflow assistant into a strict admission gate. By integrating `star-toml`, `cargo-cicd` will load and execute local policies as immutable operational law, recording cryptographic proof of execution (receipts) to certify workspace readiness before any changes are committed or published to registry platforms.

---

## 2. Positioning & Product Principles

### 2.1 Positioning
`cargo-cicd` acts as a localized admission controller for Rust workspaces. It sits between the raw text configuration files (`cicd.toml`), local git workspaces, and remote registries. Rather than deferring verification to remote CI systems, `cargo-cicd` enforces conformance locally on the engineer's workstation, generating cryptographically verified execution receipts that serve as sovereign proof of quality.

### 2.2 Product Principles

* **Configuration is Operational Law**: A configuration file is not a passive bag of properties; it is the boundary of permitted system execution. If a configuration cannot be admitted under the schema's invariants, the system must not start.
* **Zero-Trust Edge Adjudication**: Developers must not push unverified code. All verification gates (lints, unit tests, trybuild type checks) must execute locally first, producing an immutable trace.
* **Typestate Conformance**: Configuration must flow through a deterministic, compiler-checked pipeline from raw strings to a frozen, witnessed envelope (`AdmittedConfig<T>`).
* **Unforgeable Proof of Process**: Every pipeline run must generate a process-mining compliant receipt (XES/OCEL 2.0) that binds the code digest, configuration digest, and execution log under a BLAKE3 signature.

---

## 3. Supported Surfaces & Workflows

### 3.1 Supported Surfaces

1. **Command Line Interface (CLI)**: A Rust binary (`cargo-cicd`) executing subcommand nouns (`status`, `target`, `test`, `publish`, etc.) with strict, predictable exit codes.
2. **Language Server Protocol (LSP)**: Integrates with IDEs using the `star-toml-lsp` protocol to expose live diagnostics, autocomplete, hover card schema explanations, and instant verification feedback.
3. **CI/CD Integrations**: Non-interactive actions (e.g., GitHub Actions, GitLab CI) that parse receipts, verify signatures, and block build pipelines if process logs do not conform to the declared OWL/PROV process ontology.

### 3.2 Target Workflows

```
   [ Developer edits cicd.toml ]
                 │
                 ▼
     [ LSP checks cicd.toml ] ──► Errors/Warnings: DOC-xxx, SCH-JSON-xxx
                 │
                 ▼
    [ cargo cicd workspace doctor ] ──► Strict Admission (q_config = 1)
                 │
                 ▼
        [ Run Tests & Lints ] ──► Emits XES/OCEL 2.0 process traces
                 │
                 ▼
     [ wpm receipt doctor ] ──► Audits traces against cicd-process.ttl
                 │
                 ▼
    [ cargo cicd publish run ] ──► Verifies BLAKE3 witness & verdict;
                                   Publishes only if Admitted
```

* **Workflow 1: Local Edit Loop with LSP**: As a developer edits `cicd.toml`, the LSP verifies the schema and rules. Warnings (e.g., missing descriptions, missing default values) or Fatal errors (e.g., directory traversal paths, invalid ports) are surfaced instantly in the editor.
* **Workflow 2: Pre-Commit Admission**: When running `cargo cicd status` or git hooks, the `star-toml` loader runs in strict mode. If the configuration fails validation, the exit code is non-zero, preventing the commit.
* **Workflow 3: Publish Receipt Gate**: Prior to publishing, `cargo-cicd` generates a final execution receipt, sends it to the wasm4pm oracle for adjudication, checks the signature, and only proceeds to `cargo publish` if the verdict is explicitly `Accept`.

---

## 4. Security Philosophy

`cargo-cicd` assumes a zero-trust model towards local environments:
* **Sandboxed Path Resolution**: Any file path reference specified in `cicd.toml` (e.g. build targets, artifact directories) must be verified against `star-toml` path policies (`Sandbox` or `RelativeOnly`). Absolute paths or directory-traversal strings (`../../etc/passwd`) are rejected at load-time.
* **Strict Deserialization**: The loader refuses any configuration with unknown or redundant properties. This prevents configuration smuggling attacks where flags are injected into unparsed fields to bypass gates.
* **Cryptographic Witness Chains**: Every `AdmittedConfig` computes a BLAKE3 witness digest combining the schema definition, the configuration inputs, and environment variable hashes. This digest is hard-bound to the final publication receipt.

---

## 5. Success Metrics

1. **Zero Unwitnessed Releases**: 100% of crates published to crates.io from workspaces utilizing `cargo-cicd` must have an associated, valid `AdmittedConfig` witness receipt.
2. **Fitness Score = 1.0**: The process fitness score calculated by the `wpm` (wasm4pm) oracle must be exactly 1.0, indicating the execution order strictly adhered to the ontology choice-graph (`cicd-process.ttl`).
3. **100% Diagnostic Coverage**: Every failure to load, merge, or validate configuration must yield a structured error code (matching `DOC-xxx` or `SCH-JSON-xxx` definitions) with actionable repair hints, minimizing developer troubleshooting time.
