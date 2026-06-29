# Architecture Requirements Document (ARD) — cargo-cicd

**Status**: Proposal  
**Version**: v1.0.0  
**Author**: teamwork_preview_explorer_docs_update  

---

## 1. System Architecture

```text
                     USER & IDE SURFACE (CLI / LSP)
+------------------------------------------------------------------------+
|                                                                        |
|    cargo-cicd CLI Command                 star-toml LSP / diagnostics  |
|          |                                            |                |
+----------|--------------------------------------------|----------------+
           |                                            |
           |                                            |
           v                                            v
==========================================================================
1. OPERATIONAL LAW LAYER (star-toml Schema & Validation Rules)
- Defines constraints on cicd.toml (e.g. allowed targets, port ranges, limits)
- Path policies: Sandbox, RelativeOnly, BlockForbidden
- Custom Validate implementations on structs
==========================================================================
           |
           | Loader admission resolves: q_config = 1
           v
==========================================================================
2. PLANNING LAYER (Engine State Initialization)
- Gathers workspace symbols, toolchain state, changes, and dirty files
- Constructs the raw execution DAG (conforming to cicd-process.ttl)
- Produces EngineState aggregate containing the AdmittedConfig<T> envelope
==========================================================================
           |
           | Formulates target sequence
           v
==========================================================================
3. EXECUTION LAYER (Runners & Adapters)
- Spawns subprocesses (cargo build, cargo test, trybuild)
- Emits real-time ProcessEvents (XES log appends)
- Replaces/subtracted keyed receipts in target/cargo-cicd/evidence/
==========================================================================
           |
           | Compiles trace output
           v
==========================================================================
4. VERIFICATION LAYER (wpm Oracle / Conformance Check)
- Evaluates XES logs against ontology choose-graphs (cicd-process.ttl)
- Performs ReceiptDoctor checks (wpm receipt doctor --strict)
- Computes Conformance score and checks for deceptive traces
==========================================================================
           |
           | Yields Verdict (Accept / Refused / NotAvailable)
           v
==========================================================================
5. STANDING LAYER (Final Gate / AdmittedConfig Witness)
- Calculates final standing bit (q_standing = q_config ∧ q_verification)
- Emits signed, BLAKE3 hash-bound execution witness
- Gates final execution (e.g. publish proceeds only if q_standing = 1)
==========================================================================
```

---

## 2. Architecture Layers

### 2.1 Operational Law Layer
The bedrock of the system. Ingests raw workspace configuration (`cicd.toml`) and environment variables using the `star-toml` loader pipeline. Validates structural and semantic rules statically before any other logic compiles.

### 2.2 Planning Layer
Inspects workspace metadata, git files, and IDE symbols to build an in-memory execution plan. Aggregates all context into the `EngineState` root, incorporating the `AdmittedConfig<T>` wrapper.

### 2.3 Execution Layer
Interacts with the shell and compilers via isolated adapters. Spawns cargo operations and writes structured XES events representing discrete operations. No business decisions are made here.

### 2.4 Verification Layer
Analyzes execution outputs. Uses the wasm4pm oracle (`wpm`) to compare execution traces against the declared OWL/PROV ontology choice graph (`cicd-process.ttl`). Ensures no out-of-order or skipped steps exist.

### 2.5 Standing Layer
The final gateway. Combines configuration admission standing ($q_{config} = 1$) and process verification standing ($q_{verification} = 1$) to compute the sovereign standing bit ($q_{standing} = q_{config} \wedge q_{verification}$). Emits the BLAKE3 witness hash representing the absolute state proof.

---

## 3. Authority Model & Security Model

### 3.1 Authority Model
Sovereignty is distributed across two authorities:
1. **The Policy Authority (Config Admission)**: Governed by the `star-toml` schema and Custom Validators. The user is allowed to adjust configurations (e.g. target sizes, test directories), but the values must strictly fall within boundaries declared by the compile-time schema.
2. **The Adjudication Authority (Process Verification)**: Governed by the wasm4pm oracle (`wpm`). It audits the historical activity execution trace. The CLI is merely a supplicant submitting a receipt; it has no power to grant itself permission to publish.

### 3.2 Security Model
* **Path Sandboxing**: The system restricts all file path resolutions to the workspace root using `star_toml::PathPolicy::Sandbox`. Attempts to reference external toolchains, log targets, or project files outside this box trigger immediate refusal.
* **Deterministic Serialization**: Configurations are serialized into canonical, alphabetically sorted TOML before hashing. This ensures two semantically identical configs always yield the same cryptographic digest.
* **Cryptographic Bindings**: Receipts are bound to:
  * Schema Digest
  * Canonical Configuration Digest
  * Process Trace Digest (XES logs)
  * BLAKE3 Witness Signatures

---

## 4. Core Invariants

To guarantee system stability, `cargo-cicd` refactors its core invariants under `star-toml`'s strict typestates:

1. **Public Crate Boundary**: Only deserialized, frozen, and fully admitted configuration wrappers (`AdmittedConfig<T>`) can cross the boundary between `cargo-cicd-cli` and `cargo-cicd-core`.
2. **Unconditional Evidence Logging**: Every command execution must register start and complete timestamps, appending them directly to the `events.jsonl` pipeline trace.
3. **No Silent Verdict Fallback**: Missing key-value pairs or missing signature blocks in the oracle's verification logs are treated as immediate structural defects (`Refused`).
4. **Keyed Receipt Subtraction**: The system prevents phantom receipts. Emitting a new receipt replaces the old one for that specific command key; old receipts cannot linger as valid proof of the current state.
5. **Strict Schema Admission**: The loader runs with strict unknown field rejection enabled. Unknown properties in `cicd.toml` trigger immediate exit code `2` (Invalid Workspace).
6. **Path Traversal Prevention**: Any user-defined file path is evaluated against `star-toml` relative paths; absolute paths or directory traversals (`..`) are denied admission.
7. **Publish Evidence Gate**: Crates may only be uploaded to registry endpoints if the current witness configuration has standing ($q_{config} = 1$) and the wasm4pm oracle returns `Accept`.

---

## 5. Chatman's Law

> **"The code is not the system; the system is the bounded space of paths admitted by the law, witnessed by the build, and proven by the verification loop."**

In traditional CI/CD, the code repository is assumed to be the system. Chatman's Law rejects this assumption. 

Code in isolation has no authority. A system is only realized when:
1. **The Law (Operational Law)** defines the boundaries of valid configuration states.
2. **The Build (Witnessed Build)** produces deterministic, hash-bound configurations and artifacts.
3. **The Loop (Verification Loop)** continuously executes and certifies that runtime operations align with the declared process ontology.

`cargo-cicd` refactored with `star-toml` is the physical manifestation of Chatman's Law.
